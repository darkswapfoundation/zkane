//! ZKane Privacy Pool Alkane Contract
//!
//! A privacy pool implementation for alkanes assets using zero-knowledge proofs.
//! Users can deposit alkanes assets and withdraw them from different addresses,
//! breaking the on-chain link between deposits and withdrawals.
//!
//! IMPORTANT: Recipients are determined by Bitcoin transaction vouts, not by the contract.
//! The ZK proof validates that the transaction structure matches the intended recipient.

use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::response::CallResponse;
use alkanes_support::context::Context;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::witness::find_witness_payload;
use alkanes_support::id::AlkaneId;
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::utils::consensus_decode;
use zkane_common::{Commitment, NullifierHash, WithdrawalProof, ZKaneConfig};
use zkane_crypto::{generate_commitment, generate_nullifier_hash, verify_merkle_path};
use anyhow::{anyhow, Result};
use bitcoin::{Transaction, TxOut};
use std::io::Cursor;
use std::sync::Arc;

#[cfg(test)]
pub mod tests;

/// ZKane privacy pool contract
#[derive(Default)]
pub struct ZKaneContract {
    /// Whether the contract has been initialized
    initialized: bool,
}

/// Witness envelope data structures
#[derive(serde::Deserialize, serde::Serialize)]
struct DepositWitnessData {
    /// The commitment to deposit (32 bytes)
    commitment: [u8; 32],
}

#[derive(serde::Deserialize, serde::Serialize)]
struct WithdrawalWitnessData {
    /// The zero-knowledge proof (variable size)
    /// This proof validates:
    /// 1. Knowledge of secret and nullifier for a commitment in the tree
    /// 2. The transaction outputs match the intended recipient
    proof: Vec<u8>,
    /// The merkle root (32 bytes)
    merkle_root: [u8; 32],
    /// The nullifier hash (32 bytes)
    nullifier_hash: [u8; 32],
    /// Merkle path elements (variable size)
    path_elements: Vec<[u8; 32]>,
    /// Merkle path indices (variable size)
    path_indices: Vec<bool>,
    /// The leaf index of the commitment
    leaf_index: u32,
    /// The original commitment being withdrawn (32 bytes)
    commitment: [u8; 32],
    /// Hash of the transaction outputs (for recipient validation)
    /// This prevents frontrunning by binding the proof to specific outputs
    outputs_hash: [u8; 32],
}

/// Message enum for opcode-based dispatch
/// All opcodes are limited to 80 bytes of input data
#[derive(MessageDispatch)]
enum ZKaneMessage {
    /// Initialize the privacy pool
    /// Input: asset_id_block (16) + asset_id_tx (16) + denomination (16) + tree_height (4) = 52 bytes
    #[opcode(0)]
    Initialize {
        /// The alkane asset ID this pool accepts (block part)
        asset_id_block: u128,
        /// The alkane asset ID this pool accepts (tx part)
        asset_id_tx: u128,
        /// The denomination (fixed amount) for deposits/withdrawals
        denomination: u128,
        /// The merkle tree height
        tree_height: u32,
    },

    /// Deposit alkanes into the privacy pool
    /// Uses witness envelope for commitment data
    /// Input: empty (commitment comes from witness)
    #[opcode(1)]
    Deposit,

    /// Withdraw alkanes from the privacy pool
    /// Uses witness envelope for proof and merkle path data
    /// The recipient is determined by the transaction vouts, not by contract parameters
    /// Input: empty (all data comes from witness and transaction structure)
    #[opcode(2)]
    Withdraw,

    /// Get the current merkle root
    #[opcode(10)]
    #[returns(Vec<u8>)]
    GetRoot,

    /// Get the number of deposits
    #[opcode(11)]
    #[returns(u128)]
    GetDepositCount,

    /// Check if a nullifier hash has been spent
    /// Input: nullifier_hash (32 bytes)
    #[opcode(12)]
    #[returns(bool)]
    IsNullifierSpent {
        nullifier_hash: [u8; 32],
    },

    /// Get the pool configuration
    #[opcode(13)]
    #[returns(Vec<u8>)]
    GetConfig,

    /// Get the denomination
    #[opcode(14)]
    #[returns(u128)]
    GetDenomination,

    /// Get the asset ID
    #[opcode(15)]
    #[returns(Vec<u8>)]
    GetAssetId,

    /// Get merkle path for a leaf index
    /// Input: leaf_index (4 bytes)
    #[opcode(16)]
    #[returns(Vec<u8>)]
    GetMerklePath {
        leaf_index: u32,
    },

    /// Verify a commitment exists
    /// Input: commitment (32 bytes)
    #[opcode(17)]
    #[returns(bool)]
    HasCommitment {
        commitment: [u8; 32],
    },
}

impl ZKaneContract {
    /// Get the pointer to the configuration
    fn config_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/config")
    }

    /// Get the configuration
    fn get_config(&self) -> Result<ZKaneConfig> {
        let data = self.config_pointer().get();
        if data.is_empty() {
            return Err(anyhow!("Contract not initialized"));
        }
        
        let config: ZKaneConfig = serde_json::from_slice(&data)?;
        Ok(config)
    }

    /// Set the configuration
    fn set_config(&self, config: &ZKaneConfig) -> Result<()> {
        let data = serde_json::to_vec(config)?;
        self.config_pointer().set(Arc::new(data));
        Ok(())
    }

    /// Get the pointer to the merkle tree root
    fn root_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/merkle_root")
    }

    /// Get the current merkle root
    fn get_root(&self) -> [u8; 32] {
        let data = self.root_pointer().get();
        if data.len() == 32 {
            let mut root = [0u8; 32];
            root.copy_from_slice(&data);
            root
        } else {
            // Return zero root if not set
            [0u8; 32]
        }
    }

    /// Set the merkle root
    fn set_root(&self, root: &[u8; 32]) {
        self.root_pointer().set(Arc::new(root.to_vec()));
    }

    /// Get the pointer to the deposit count
    fn deposit_count_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/deposit_count")
    }

    /// Get the number of deposits
    fn get_deposit_count(&self) -> u32 {
        self.deposit_count_pointer().get_value::<u32>()
    }

    /// Set the deposit count
    fn set_deposit_count(&self, count: u32) {
        self.deposit_count_pointer().set_value::<u32>(count);
    }

    /// Get the pointer to commitments
    fn commitments_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/commitments")
    }

    /// Check if a commitment exists
    fn has_commitment(&self, commitment: &[u8; 32]) -> bool {
        self.commitments_pointer()
            .select(&commitment.to_vec())
            .get_value::<u8>() == 1
    }

    /// Add a commitment
    fn add_commitment(&self, commitment: &[u8; 32]) {
        self.commitments_pointer()
            .select(&commitment.to_vec())
            .set_value::<u8>(1);
    }

    /// Get the pointer to commitment by index
    fn commitment_by_index_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/commitments_by_index")
    }

    /// Store commitment by index for merkle path generation
    fn store_commitment_by_index(&self, index: u32, commitment: &[u8; 32]) {
        self.commitment_by_index_pointer()
            .select(&index.to_le_bytes().to_vec())
            .set(Arc::new(commitment.to_vec()));
    }

    /// Get commitment by index
    fn get_commitment_by_index(&self, index: u32) -> Option<[u8; 32]> {
        let data = self.commitment_by_index_pointer()
            .select(&index.to_le_bytes().to_vec())
            .get();
        
        if data.len() == 32 {
            let mut commitment = [0u8; 32];
            commitment.copy_from_slice(&data);
            Some(commitment)
        } else {
            None
        }
    }

    /// Get the pointer to spent nullifiers
    fn nullifiers_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/nullifiers")
    }

    /// Check if a nullifier hash has been spent
    fn is_nullifier_spent(&self, nullifier_hash: &[u8; 32]) -> bool {
        self.nullifiers_pointer()
            .select(&nullifier_hash.to_vec())
            .get_value::<u8>() == 1
    }

    /// Mark a nullifier hash as spent
    fn spend_nullifier(&self, nullifier_hash: &[u8; 32]) {
        self.nullifiers_pointer()
            .select(&nullifier_hash.to_vec())
            .set_value::<u8>(1);
    }

    /// Observe initialization to prevent multiple initializations
    fn observe_initialization(&self) -> Result<()> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        if pointer.get().is_empty() {
            pointer.set_value::<u8>(1);
            Ok(())
        } else {
            Err(anyhow!("Contract already initialized"))
        }
    }

    /// Parse witness data for deposits
    fn parse_deposit_witness(&self) -> Result<DepositWitnessData> {
        let context = self.context()?;
        let tx = consensus_decode::<Transaction>(&mut Cursor::new(context.transaction()?))?;
        
        let witness_data = find_witness_payload(&tx, 0)
            .ok_or_else(|| anyhow!("No witness data found for deposit"))?;

        if witness_data.len() < 32 {
            return Err(anyhow!("Invalid witness data: too short for commitment"));
        }

        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&witness_data[0..32]);

        Ok(DepositWitnessData { commitment })
    }

    /// Parse witness data for withdrawals
    fn parse_withdrawal_witness(&self) -> Result<WithdrawalWitnessData> {
        let context = self.context()?;
        let tx = consensus_decode::<Transaction>(&mut Cursor::new(context.transaction()?))?;
        
        let witness_data = find_witness_payload(&tx, 0)
            .ok_or_else(|| anyhow!("No witness data found for withdrawal"))?;

        // Deserialize the withdrawal data from JSON
        let withdrawal_data: WithdrawalWitnessData = serde_json::from_slice(&witness_data)
            .map_err(|e| anyhow!("Failed to parse withdrawal witness data: {}", e))?;

        Ok(withdrawal_data)
    }

    /// Hash the transaction outputs for recipient validation
    fn hash_transaction_outputs(&self, tx: &Transaction) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        
        // Hash all outputs in order
        for output in &tx.output {
            hasher.update(&output.value.to_le_bytes());
            hasher.update(&output.script_pubkey.as_bytes());
        }
        
        hasher.finalize().into()
    }

    /// Validate that the transaction outputs match the expected hash
    fn validate_transaction_outputs(&self, expected_outputs_hash: &[u8; 32]) -> Result<()> {
        let context = self.context()?;
        let tx = consensus_decode::<Transaction>(&mut Cursor::new(context.transaction()?))?;
        
        let actual_outputs_hash = self.hash_transaction_outputs(&tx);
        
        if &actual_outputs_hash != expected_outputs_hash {
            return Err(anyhow!("Transaction outputs do not match proof"));
        }
        
        Ok(())
    }

    /// Generate a simple merkle path (placeholder implementation)
    fn generate_merkle_path(&self, leaf_index: u32) -> Result<Vec<u8>> {
        let config = self.get_config()?;
        let deposit_count = self.get_deposit_count();
        
        if leaf_index >= deposit_count {
            return Err(anyhow!("Leaf index out of bounds"));
        }

        // This is a simplified implementation
        // In production, you'd maintain a proper merkle tree
        let mut path_elements = Vec::new();
        let mut path_indices = Vec::new();
        
        // Generate dummy path for now
        for level in 0..config.tree_height {
            path_elements.push([0u8; 32]); // Zero hash
            path_indices.push(false); // Left side
        }

        let path_data = serde_json::json!({
            "elements": path_elements,
            "indices": path_indices
        });

        Ok(path_data.to_string().into_bytes())
    }

    /// Initialize the privacy pool
    fn initialize(
        &self,
        asset_id_block: u128,
        asset_id_tx: u128,
        denomination: u128,
        tree_height: u32,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);

        // Prevent multiple initializations
        self.observe_initialization()?;

        // Create configuration
        let asset_id = AlkaneId {
            block: asset_id_block,
            tx: asset_id_tx,
        };

        let config = ZKaneConfig::new(
            asset_id,
            denomination,
            tree_height,
            vec![], // TODO: Add verifier key
        );

        // Store configuration
        self.set_config(&config)?;

        // Initialize merkle root to zero
        self.set_root(&[0u8; 32]);

        // Initialize deposit count
        self.set_deposit_count(0);

        Ok(response)
    }

    /// Process a deposit (reads commitment from witness envelope)
    fn deposit(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Get configuration
        let config = self.get_config()?;

        // Parse witness data to get commitment
        let witness_data = self.parse_deposit_witness()?;
        let commitment = witness_data.commitment;

        // Check if commitment already exists
        if self.has_commitment(&commitment) {
            return Err(anyhow!("Commitment already exists"));
        }

        // Verify the correct amount of the correct asset was sent
        let mut received_amount = 0u128;
        for transfer in &context.incoming_alkanes.0 {
            if transfer.id == config.asset_id {
                received_amount += transfer.value;
            }
        }

        if received_amount != config.denomination {
            return Err(anyhow!(
                "Invalid deposit amount: expected {}, got {}",
                config.denomination,
                received_amount
            ));
        }

        // Add commitment to storage
        self.add_commitment(&commitment);

        // Store commitment by index for merkle path generation
        let deposit_count = self.get_deposit_count();
        self.store_commitment_by_index(deposit_count, &commitment);

        // Update deposit count
        self.set_deposit_count(deposit_count + 1);

        // TODO: Update merkle tree root properly
        // For now, we'll use a simple hash of the commitment count
        let mut new_root = [0u8; 32];
        new_root[0..4].copy_from_slice(&(deposit_count + 1).to_le_bytes());
        self.set_root(&new_root);

        // Emit deposit event
        let deposit_data = serde_json::json!({
            "type": "deposit",
            "commitment": hex::encode(commitment),
            "leaf_index": deposit_count,
            "timestamp": context.myself.block
        });

        response.data = deposit_data.to_string().into_bytes();

        Ok(response)
    }

    /// Process a withdrawal (reads proof and path from witness envelope)
    /// The recipient is determined by the Bitcoin transaction vouts, not by contract parameters
    fn withdraw(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Get configuration
        let config = self.get_config()?;

        // Parse witness data to get withdrawal information
        let witness_data = self.parse_withdrawal_witness()?;

        // Validate that the transaction outputs match the proof
        // This prevents frontrunning by binding the proof to specific outputs
        self.validate_transaction_outputs(&witness_data.outputs_hash)?;

        // Check if nullifier has already been spent
        if self.is_nullifier_spent(&witness_data.nullifier_hash) {
            return Err(anyhow!("Nullifier already spent"));
        }

        // Check if commitment exists
        if !self.has_commitment(&witness_data.commitment) {
            return Err(anyhow!("Unknown commitment"));
        }

        // Verify merkle root is valid (current root)
        let current_root = self.get_root();
        if witness_data.merkle_root != current_root {
            return Err(anyhow!("Invalid merkle root"));
        }

        // TODO: Verify the zero-knowledge proof
        // The proof should validate:
        // 1. Knowledge of secret and nullifier for the commitment
        // 2. Merkle tree inclusion
        // 3. Transaction outputs hash matches intended recipient
        // For now, we'll skip proof verification in this demo
        if witness_data.proof.is_empty() {
            return Err(anyhow!("Empty proof provided"));
        }

        // Verify merkle path (as a backup check)
        let commitment_obj = Commitment::new(witness_data.commitment);
        let path = zkane_common::MerklePath::new(
            witness_data.path_elements,
            witness_data.path_indices,
        )?;
        
        let path_valid = verify_merkle_path(
            &commitment_obj,
            witness_data.leaf_index,
            &path,
            &witness_data.merkle_root,
            config.tree_height,
        ).map_err(|e| anyhow!("Merkle path verification failed: {}", e))?;

        if !path_valid {
            return Err(anyhow!("Invalid merkle path"));
        }

        // Mark nullifier as spent
        self.spend_nullifier(&witness_data.nullifier_hash);

        // Return alkanes to be distributed according to transaction vouts
        // The actual recipient is determined by the Bitcoin transaction structure
        response.alkanes.0.push(AlkaneTransfer {
            id: config.asset_id,
            value: config.denomination,
        });

        // Emit withdrawal event
        let withdrawal_data = serde_json::json!({
            "type": "withdrawal",
            "nullifier_hash": hex::encode(witness_data.nullifier_hash),
            "outputs_hash": hex::encode(witness_data.outputs_hash),
            "timestamp": context.myself.block
        });

        response.data = withdrawal_data.to_string().into_bytes();

        Ok(response)
    }

    /// Get the current merkle root
    fn get_root_response(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let root = self.get_root();
        response.data = root.to_vec();

        Ok(response)
    }

    /// Get the deposit count
    fn get_deposit_count_response(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let count = self.get_deposit_count();
        response.data = (count as u128).to_le_bytes().to_vec();

        Ok(response)
    }

    /// Check if a nullifier hash has been spent
    fn is_nullifier_spent_response(&self, nullifier_hash: [u8; 32]) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let spent = self.is_nullifier_spent(&nullifier_hash);
        response.data = vec![if spent { 1u8 } else { 0u8 }];

        Ok(response)
    }

    /// Get the configuration
    fn get_config_response(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let config = self.get_config()?;
        response.data = serde_json::to_vec(&config)?;

        Ok(response)
    }

    /// Get the denomination
    fn get_denomination_response(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let config = self.get_config()?;
        response.data = config.denomination.to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get the asset ID
    fn get_asset_id_response(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let config = self.get_config()?;
        let mut data = Vec::new();
        data.extend_from_slice(&config.asset_id.block.to_le_bytes());
        data.extend_from_slice(&config.asset_id.tx.to_le_bytes());
        response.data = data;

        Ok(response)
    }

    /// Get merkle path for a leaf index
    fn get_merkle_path_response(&self, leaf_index: u32) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let path_data = self.generate_merkle_path(leaf_index)?;
        response.data = path_data;

        Ok(response)
    }

    /// Check if a commitment exists
    fn has_commitment_response(&self, commitment: [u8; 32]) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let exists = self.has_commitment(&commitment);
        response.data = vec![if exists { 1u8 } else { 0u8 }];

        Ok(response)
    }
}

impl AlkaneResponder for ZKaneContract {}

// Use the MessageDispatch macro for opcode handling
declare_alkane! {
    impl AlkaneResponder for ZKaneContract {
        type Message = ZKaneMessage;
    }
}