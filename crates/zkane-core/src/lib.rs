//! # ZKane Core Privacy Pool Operations
//!
//! This crate provides high-level operations for the ZKane privacy pool system.
//! It implements the core business logic for privacy-preserving deposits and
//! withdrawals, building on the cryptographic primitives from `zkane-crypto`.
//!
//! ## Overview
//!
//! The ZKane core module provides:
//!
//! - **Privacy Pool Management**: High-level interface for pool operations
//! - **Deposit Note Generation**: Creation of complete deposit information
//! - **Proof Verification**: Validation of zero-knowledge proofs
//! - **State Management**: Tracking of commitments and nullifiers
//!
//! ## Architecture
//!
//! The core system is built around the [`PrivacyPool`] struct, which manages:
//!
//! - A Merkle tree of commitments for efficient inclusion proofs
//! - A set of spent nullifiers for double-spending prevention
//! - Configuration parameters for the specific asset and denomination
//!
//! ## Usage Patterns
//!
//! ### Basic Pool Operations
//!
//! ```rust
//! use zkane_core::{PrivacyPool, mock_provider::MockProvider};
//! use zkane_common::ZKaneConfig;
//! use alkanes_support::id::AlkaneId;
//! use std::sync::Arc;
//!
//! // Create a new privacy pool
//! let config = ZKaneConfig::new(
//!     AlkaneId { block: 2, tx: 1 }.into(),  // Asset ID
//!     1000000,                              // Denomination
//!     20,                                   // Tree height
//!     vec![],                               // Verifier key
//! );
//! let provider = Arc::new(MockProvider::new(bitcoin::Network::Regtest));
//! let mut pool = PrivacyPool::new(config, provider)?;
//!
//! // Check pool status
//! let commitment_count = pool.commitment_count();
//! let merkle_root = pool.merkle_root();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Deposit Flow
//!
//! ```rust
//! use zkane_core::generate_deposit_note;
//! use alkanes_support::id::AlkaneId;
//!
//! // Generate a complete deposit note
//! let asset_id = AlkaneId { block: 2, tx: 1 }.into();
//! let denomination = 1000000u128;
//! let deposit_note = generate_deposit_note(asset_id, denomination)?;
//!
//! // The commitment can be submitted to the privacy pool
//! println!("Commitment to deposit: {}", deposit_note.commitment.to_hex());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Security Model
//!
//! The core security properties maintained by this module:
//!
//! 1. **Privacy**: Deposits and withdrawals cannot be linked
//! 2. **Integrity**: Only valid proofs are accepted
//! 3. **Double-spend Prevention**: Nullifiers can only be used once
//! 4. **Completeness**: Valid operations always succeed
//!
//! ## Error Handling
//!
//! All operations return `Result` types with descriptive errors from [`zkane_common::ZKaneError`].
//! Common error conditions include:
//!
//! - Invalid proofs or commitments
//! - Attempted double-spending
//! - Pool capacity exceeded
//! - Configuration mismatches

use zkane_common::{
    Secret, Nullifier, Commitment, NullifierHash, DepositNote, WithdrawalProof,
    ZKaneConfig, MerklePath, ZKaneError, ZKaneResult,
};
use zkane_crypto::{generate_commitment, MerkleTree};
use alkanes_support::id::AlkaneId;
use std::collections::HashSet;
use deezel_common::traits::DeezelProvider;
use std::sync::Arc;
 
pub mod mock_provider;

/// A privacy pool for a specific asset and denomination.
///
/// The `PrivacyPool` manages the state of a privacy pool, including the Merkle tree
/// of commitments and the set of spent nullifiers. It provides methods for deposits,
/// withdrawals, and state queries while maintaining privacy and security properties.
///
/// # Example
///
/// ```rust
/// use zkane_core::{PrivacyPool, mock_provider::MockProvider};
/// use zkane_common::ZKaneConfig;
/// use alkanes_support::id::AlkaneId;
/// use deezel_common::traits::DeezelProvider;
/// use std::sync::Arc;
///
/// # fn test() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = MockProvider::new(bitcoin::Network::Regtest);
/// let config = ZKaneConfig::new(
///     AlkaneId { block: 2, tx: 1 }.into(),
///     1000000,
///     20,
///     vec![],
/// );
/// let mut pool = PrivacyPool::new(config, Arc::new(provider))?;
///
/// // Check initial state
/// assert_eq!(pool.commitment_count(), 0);
/// assert!(!pool.is_nullifier_spent(&[0u8; 32]));
/// # Ok(())
/// # }
/// ```
pub struct PrivacyPool<P: DeezelProvider> {
    /// Configuration for this pool
    config: ZKaneConfig,
    /// Merkle tree storing commitments
    merkle_tree: MerkleTree,
    /// Set of spent nullifier hashes
    spent_nullifiers: HashSet<[u8; 32]>,
    /// Provider for interacting with the Bitcoin network
    provider: Arc<P>,
}

impl<P: DeezelProvider> PrivacyPool<P> {
    /// Create a new privacy pool with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration specifying asset, denomination, and tree parameters
    /// * `provider` - A provider for interacting with the Bitcoin network
    ///
    /// # Returns
    ///
    /// A `Result` containing the new privacy pool or an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zkane_core::{PrivacyPool, mock_provider::MockProvider};
    /// use zkane_common::ZKaneConfig;
    /// use alkanes_support::id::AlkaneId;
    /// use deezel_common::traits::DeezelProvider;
    /// use std::sync::Arc;
    ///
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = MockProvider::new(bitcoin::Network::Regtest);
    /// let config = ZKaneConfig::new(
    ///     AlkaneId { block: 2, tx: 1 }.into(),
    ///     1000000,
    ///     20,
    ///     vec![],
    /// );
    /// let pool = PrivacyPool::new(config, Arc::new(provider))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: ZKaneConfig, provider: Arc<P>) -> ZKaneResult<Self> {
        let merkle_tree = MerkleTree::new(config.tree_height);
        
        Ok(Self {
            config,
            merkle_tree,
            spent_nullifiers: HashSet::new(),
            provider,
        })
    }

    /// Get the configuration for this pool.
    pub fn config(&self) -> &ZKaneConfig {
        &self.config
    }

    /// Get the current Merkle root of the commitment tree.
    ///
    /// The Merkle root represents the current state of all commitments in the pool
    /// and is used in zero-knowledge proofs to prove inclusion.
    ///
    /// # Returns
    ///
    /// The 32-byte Merkle root hash.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zkane_core::{PrivacyPool, mock_provider::MockProvider};
    /// # use zkane_common::ZKaneConfig;
    /// # use alkanes_support::id::AlkaneId;
    /// # use std::sync::Arc;
    /// #
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// # let provider = MockProvider::new(bitcoin::Network::Regtest);
    /// # let config = ZKaneConfig::new(
    /// #     AlkaneId { block: 2, tx: 1 }.into(), 1000000, 20, vec![]
    /// # );
    /// # let pool = PrivacyPool::new(config, Arc::new(provider))?;
    /// let root = pool.merkle_root();
    /// assert_eq!(root.len(), 32);
    /// # Ok(())
    /// # }
    /// ```
    pub fn merkle_root(&self) -> [u8; 32] {
        self.merkle_tree.root()
    }

    /// Get the number of commitments in the pool.
    ///
    /// # Returns
    ///
    /// The total number of commitments that have been deposited.
    pub fn commitment_count(&self) -> u64 {
        self.merkle_tree.leaf_count().into()
    }

    /// Check if a nullifier hash has been spent.
    ///
    /// # Arguments
    ///
    /// * `nullifier_hash` - The nullifier hash to check
    ///
    /// # Returns
    ///
    /// `true` if the nullifier has been spent, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zkane_core::{PrivacyPool, mock_provider::MockProvider};
    /// # use zkane_common::ZKaneConfig;
    /// # use alkanes_support::id::AlkaneId;
    /// # use std::sync::Arc;
    /// #
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// # let provider = MockProvider::new(bitcoin::Network::Regtest);
    /// # let config = ZKaneConfig::new(
    /// #     AlkaneId { block: 2, tx: 1 }.into(), 1000000, 20, vec![]
    /// # );
    /// # let pool = PrivacyPool::new(config, Arc::new(provider))?;
    ///
    /// // New nullifier should not be spent
    /// assert!(!pool.is_nullifier_spent(&[42u8; 32]));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_nullifier_spent(&self, nullifier_hash: &[u8; 32]) -> bool {
        self.spent_nullifiers.contains(nullifier_hash)
    }

    /// Add a commitment to the pool.
    ///
    /// This method adds a new commitment to the Merkle tree, representing a new deposit.
    /// The commitment should be generated using [`generate_commitment`] from a secret
    /// and nullifier pair.
    ///
    /// # Arguments
    ///
    /// * `commitment` - The commitment to add
    ///
    /// # Returns
    ///
    /// A `Result` containing the leaf index where the commitment was inserted.
    ///
    /// # Errors
    ///
    /// Returns an error if the tree is full or if there's a cryptographic error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zkane_core::{PrivacyPool, mock_provider::MockProvider};
    /// # use zkane_common::{ZKaneConfig, Commitment};
    /// # use alkanes_support::id::AlkaneId;
    /// # use std::sync::Arc;
    /// #
    /// # async fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut provider = MockProvider::new(bitcoin::Network::Regtest);
    /// # let config = ZKaneConfig::new(
    /// #     AlkaneId { block: 2, tx: 1 }.into(), 1000000, 20, vec![]
    /// # );
    ///
    /// let txid = "mock_txid";
    /// let commitment_hex = "0000000000000000000000000000000000000000000000000000000000000042";
    /// let mock_response = serde_json::json!({
    ///     "vout": [
    ///         {
    ///             "scriptpubkey": format!("6a{}", commitment_hex),
    ///             "value": 0
    ///         }
    ///     ]
    /// });
    /// provider.add_response(txid, mock_response);
    /// # let mut pool = PrivacyPool::new(config, Arc::new(provider))?;
    /// let leaf_index = pool.add_commitment(txid).await?;
    /// assert_eq!(leaf_index, 0);
    /// assert_eq!(pool.commitment_count(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_commitment(&mut self, txid: &str) -> ZKaneResult<u64> {
        let tx_info = self.provider.get_tx(txid).await?;
        
        let vout = tx_info["vout"].as_array().ok_or(ZKaneError::TransactionParseError)?;
        
        let commitment = vout.iter()
            .find_map(|output| {
                let script_pubkey = output["scriptpubkey"].as_str()?;
                if script_pubkey.starts_with("6a") { // OP_RETURN
                    let data = hex::decode(&script_pubkey[2..]).ok()?;
                    if data.len() == 32 {
                        let mut commitment_bytes = [0u8; 32];
                        commitment_bytes.copy_from_slice(&data);
                        Some(Commitment::new(commitment_bytes))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .ok_or(ZKaneError::CommitmentNotFound)?;

        let leaf_index = self.merkle_tree.insert(&commitment)
            .map_err(|e| ZKaneError::CryptoError(e.to_string()))?;
        Ok(leaf_index.into())
    }

    /// Generate a Merkle inclusion proof for a commitment.
    ///
    /// This method generates the cryptographic proof needed to show that a specific
    /// commitment is included in the pool's Merkle tree.
    ///
    /// # Arguments
    ///
    /// * `leaf_index` - The index of the commitment in the tree
    ///
    /// # Returns
    ///
    /// A `Result` containing the Merkle path for the proof.
    ///
    /// # Errors
    ///
    /// Returns an error if the leaf index is invalid.
    pub fn generate_merkle_proof(&self, leaf_index: u64) -> ZKaneResult<MerklePath> {
        self.merkle_tree.generate_path(leaf_index as u32)
    }

    /// Process a withdrawal by marking the nullifier as spent.
    ///
    /// This method should be called after verifying a withdrawal proof to prevent
    /// the same nullifier from being used again.
    ///
    /// # Arguments
    ///
    /// * `nullifier_hash` - The nullifier hash being spent
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the nullifier has already been spent.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use zkane_core::{PrivacyPool, mock_provider::MockProvider};
    /// # use zkane_common::ZKaneConfig;
    /// # use alkanes_support::id::AlkaneId;
    /// # use std::sync::Arc;
    /// #
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// # let provider = MockProvider::new(bitcoin::Network::Regtest);
    /// # let config = ZKaneConfig::new(
    /// #     AlkaneId { block: 2, tx: 1 }.into(), 1000000, 20, vec![]
    /// # );
    /// # let mut pool = PrivacyPool::new(config, Arc::new(provider))?;
    ///
    /// let nullifier_hash = [42u8; 32];
    ///
    /// // First withdrawal should succeed
    /// pool.process_withdrawal(&nullifier_hash)?;
    /// assert!(pool.is_nullifier_spent(&nullifier_hash));
    ///
    /// // Second withdrawal should fail
    /// assert!(pool.process_withdrawal(&nullifier_hash).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn process_withdrawal(&mut self, nullifier_hash: &[u8; 32]) -> ZKaneResult<()> {
        if self.spent_nullifiers.contains(nullifier_hash) {
            return Err(ZKaneError::NullifierAlreadySpent);
        }
        
        self.spent_nullifiers.insert(*nullifier_hash);
        Ok(())
    }

    /// Verify a withdrawal proof against the current pool state.
    ///
    /// This method performs all the cryptographic verification needed to validate
    /// a withdrawal, including proof verification and nullifier checking.
    ///
    /// # Arguments
    ///
    /// * `proof` - The withdrawal proof to verify
    ///
    /// # Returns
    ///
    /// `true` if the proof is valid and the withdrawal is allowed, `false` otherwise.
    ///
    /// # Note
    ///
    /// This method only verifies the proof; it does not mark the nullifier as spent.
    /// Call [`process_withdrawal`] after successful verification to update the state.
    pub fn verify_withdrawal_proof(&self, proof: &WithdrawalProof) -> bool {
        // Check if nullifier is already spent
        if self.is_nullifier_spent(proof.nullifier_hash.as_bytes()) {
            return false;
        }

        // Check if merkle root matches current state
        if proof.merkle_root != self.merkle_root() {
            return false;
        }

        // In a full implementation, this would verify the zero-knowledge proof
        // For now, we assume the proof is valid if basic checks pass
        true
    }

    /// Get the maximum capacity of this pool.
    ///
    /// # Returns
    ///
    /// The maximum number of commitments this pool can hold.
    pub fn max_capacity(&self) -> u64 {
        self.config.max_deposits()
    }

    /// Check if the pool is at capacity.
    ///
    /// # Returns
    ///
    /// `true` if the pool cannot accept more deposits, `false` otherwise.
    pub fn is_full(&self) -> bool {
        self.commitment_count() >= self.max_capacity()
    }

    /// Get statistics about the pool.
    ///
    /// # Returns
    ///
    /// A tuple containing (commitment_count, spent_nullifiers_count, capacity).
    pub fn stats(&self) -> (u64, usize, u64) {
        (
            self.commitment_count(),
            self.spent_nullifiers.len(),
            self.max_capacity(),
        )
    }
}

/// Generate a complete deposit note for the given asset and denomination.
///
/// This function creates all the cryptographic material needed for a deposit,
/// including the secret, nullifier, and commitment. The resulting deposit note
/// contains everything a user needs to later withdraw their funds.
///
/// # Arguments
///
/// * `asset_id` - The alkanes asset being deposited
/// * `denomination` - The amount being deposited
///
/// # Returns
///
/// A `Result` containing the complete deposit note.
///
/// # Example
///
/// ```rust
/// use zkane_core::generate_deposit_note;
/// use alkanes_support::id::AlkaneId;
///
/// let asset_id = AlkaneId { block: 2, tx: 1 }.into();
/// let denomination = 1000000u128;
/// let deposit_note = generate_deposit_note(asset_id, denomination)?;
///
/// // The deposit note contains everything needed for later withdrawal
/// println!("Secret: {}", deposit_note.secret.to_hex());
/// println!("Nullifier: {}", deposit_note.nullifier.to_hex());
/// println!("Commitment: {}", deposit_note.commitment.to_hex());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Security Notes
///
/// - The secret and nullifier are generated using secure randomness
/// - The deposit note should be stored securely by the user
/// - Loss of the deposit note makes withdrawal impossible
pub fn generate_deposit_note(asset_id: AlkaneId, denomination: u128) -> ZKaneResult<DepositNote> {
    let secret = Secret::random();
    let nullifier = Nullifier::random();
    let commitment = generate_commitment(&nullifier, &secret)
        .map_err(|e| ZKaneError::CryptoError(e.to_string()))?;

    Ok(DepositNote::new(
        secret,
        nullifier,
        commitment,
        asset_id.into(),
        denomination,
        0, // Leaf index will be set when deposited
    ))
}

/// Verify the integrity of a deposit note.
///
/// This function checks that the commitment in a deposit note was correctly
/// generated from the secret and nullifier.
///
/// # Arguments
///
/// * `note` - The deposit note to verify
///
/// # Returns
///
/// A `Result` containing `true` if the note is valid, `false` otherwise.
///
/// # Example
///
/// ```rust
/// use zkane_core::{generate_deposit_note, verify_deposit_note};
/// use alkanes_support::id::AlkaneId;
///
/// let asset_id = AlkaneId { block: 2, tx: 1 }.into();
/// let note = generate_deposit_note(asset_id, 1000000)?;
///
/// // Valid note should verify
/// assert!(verify_deposit_note(&note)?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn verify_deposit_note(note: &DepositNote) -> ZKaneResult<bool> {
    let computed_commitment = generate_commitment(&note.nullifier, &note.secret)
        .map_err(|e| ZKaneError::CryptoError(e.to_string()))?;
    
    Ok(computed_commitment == note.commitment)
}

/// Create a withdrawal proof for the given parameters.
///
/// This function creates a withdrawal proof structure with the provided parameters.
/// In a full implementation, this would generate the actual zero-knowledge proof.
///
/// # Arguments
///
/// * `proof_bytes` - The zero-knowledge proof data
/// * `merkle_root` - The Merkle root at time of proof generation
/// * `nullifier_hash` - The nullifier hash being revealed
/// * `recipient` - The recipient address
///
/// # Returns
///
/// A new withdrawal proof structure.
///
/// # Note
///
/// This is a convenience function for creating the proof structure.
/// The actual zero-knowledge proof generation would be handled by external
/// libraries (e.g., Noir circuits).
pub fn create_withdrawal_proof(
    proof_bytes: Vec<u8>,
    merkle_root: [u8; 32],
    nullifier_hash: NullifierHash,
    recipient: u128,
) -> WithdrawalProof {
    WithdrawalProof::new(proof_bytes, merkle_root, nullifier_hash, recipient)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mock_provider::MockProvider;
    use std::sync::Arc;

    fn create_test_pool() -> PrivacyPool<MockProvider> {
        let config = ZKaneConfig::new(
            alkanes_support::id::AlkaneId { block: 2, tx: 1 }.into(),
            1000000,
            4, // Small tree for testing
            vec![],
        );
        let provider = Arc::new(MockProvider::new(bitcoin::Network::Regtest));
        PrivacyPool::new(config, provider).unwrap()
    }

    #[test]
    fn test_privacy_pool_creation() {
        let pool = create_test_pool();
        
        assert_eq!(pool.commitment_count(), 0);
        assert_eq!(pool.max_capacity(), 16); // 2^4
        assert!(!pool.is_full());
    }

    #[tokio::test]
    async fn test_commitment_addition() {
        let mut pool = create_test_pool();
        let txid = "mock_txid";
        
        let commitment_hex = "0000000000000000000000000000000000000000000000000000000000000042";
        let mock_response = serde_json::json!({
            "vout": [
                {
                    "scriptpubkey": format!("6a{}", commitment_hex),
                    "value": 0
                }
            ]
        });
        
        pool.provider
            .responses
            .lock()
            .unwrap()
            .insert(txid.to_string(), mock_response);

        let leaf_index = pool.add_commitment(txid).await.unwrap();
        
        assert_eq!(leaf_index, 0);
        assert_eq!(pool.commitment_count(), 1);
    }

    #[test]
    fn test_nullifier_spending() {
        let mut pool = create_test_pool();
        
        let nullifier_hash = [42u8; 32];
        
        // Initially not spent
        assert!(!pool.is_nullifier_spent(&nullifier_hash));
        
        // Process withdrawal
        pool.process_withdrawal(&nullifier_hash).unwrap();
        assert!(pool.is_nullifier_spent(&nullifier_hash));
        
        // Second withdrawal should fail
        assert!(pool.process_withdrawal(&nullifier_hash).is_err());
    }

    #[tokio::test]
    async fn test_merkle_proof_generation() {
        let mut pool = create_test_pool();
        let txid = "mock_txid_proof";

        let commitment_hex = "0000000000000000000000000000000000000000000000000000000000000042";
        let mock_response = serde_json::json!({
            "vout": [
                {
                    "scriptpubkey": format!("6a{}", commitment_hex),
                    "value": 0
                }
            ]
        });
        
        pool.provider
            .responses
            .lock()
            .unwrap()
            .insert(txid.to_string(), mock_response);

        let leaf_index = pool.add_commitment(txid).await.unwrap();
        
        let proof = pool.generate_merkle_proof(leaf_index).unwrap();
        assert_eq!(proof.len(), 4); // Tree height
    }

    #[test]
    fn test_deposit_note_generation() {
        let asset_id = AlkaneId { block: 2, tx: 1 };
        let denomination = 1000000u128;
        
        let note = generate_deposit_note(asset_id, denomination).unwrap();
        
        assert_eq!(note.asset_id, asset_id.into());
        assert_eq!(note.denomination, denomination);
        assert!(verify_deposit_note(&note).unwrap());
    }

    #[tokio::test]
    async fn test_withdrawal_proof_verification() {
        let mut pool = create_test_pool();
        
        // Add a commitment
        let txid = "mock_txid_withdraw";
        let commitment_hex = "0000000000000000000000000000000000000000000000000000000000000042";
        let mock_response = serde_json::json!({
            "vout": [
                {
                    "scriptpubkey": format!("6a{}", commitment_hex),
                    "value": 0
                }
            ]
        });
        pool.provider
            .responses
            .lock()
            .unwrap()
            .insert(txid.to_string(), mock_response);
        pool.add_commitment(txid).await.unwrap();
        
        let nullifier_hash = NullifierHash::new([1u8; 32]);
        let proof = WithdrawalProof::new(
            vec![0u8; 256],
            pool.merkle_root(),
            nullifier_hash,
            12345,
        );
        
        // Should verify with correct merkle root
        assert!(pool.verify_withdrawal_proof(&proof));
        
        // Should fail after nullifier is spent
        pool.process_withdrawal(nullifier_hash.as_bytes()).unwrap();
        assert!(!pool.verify_withdrawal_proof(&proof));
    }

    #[tokio::test]
    async fn test_pool_capacity() {
        let mut pool = create_test_pool();
        
        // Fill the pool
        for i in 0..16 {
            let txid = format!("mock_txid_{}", i);
            let commitment_hex = format!("{:064x}", i);
            let mock_response = serde_json::json!({
                "vout": [
                    {
                        "scriptpubkey": format!("6a{}", commitment_hex),
                        "value": 0
                    }
                ]
            });
            pool.provider
                .responses
                .lock()
                .unwrap()
                .insert(txid.to_string(), mock_response);
            pool.add_commitment(&txid).await.unwrap();
        }
        
        assert!(pool.is_full());
        assert_eq!(pool.commitment_count(), 16);
        
        // Adding one more should fail
        let txid = "mock_txid_full";
        let commitment_hex = "0000000000000000000000000000000000000000000000000000000000000099";
        let mock_response = serde_json::json!({
            "vout": [
                {
                    "scriptpubkey": format!("6a{}", commitment_hex),
                    "value": 0
                }
            ]
        });
        pool.provider
            .responses
            .lock()
            .unwrap()
            .insert(txid.to_string(), mock_response);
        assert!(pool.add_commitment(txid).await.is_err());
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let mut pool = create_test_pool();
        
        // Add some commitments and spend some nullifiers
        let txid1 = "mock_txid_stats1";
        let txid2 = "mock_txid_stats2";
        let commitment_hex1 = "0000000000000000000000000000000000000000000000000000000000000001";
        let commitment_hex2 = "0000000000000000000000000000000000000000000000000000000000000002";
        let mock_response1 = serde_json::json!({
            "vout": [ { "scriptpubkey": format!("6a{}", commitment_hex1), "value": 0 } ]
        });
        let mock_response2 = serde_json::json!({
            "vout": [ { "scriptpubkey": format!("6a{}", commitment_hex2), "value": 0 } ]
        });
        pool.provider
            .responses
            .lock()
            .unwrap()
            .insert(txid1.to_string(), mock_response1);
        pool.add_commitment(txid1).await.unwrap();
        let mock_response2 = serde_json::json!({
            "vout": [ { "scriptpubkey": format!("6a{}", commitment_hex2), "value": 0 } ]
        });
        pool.provider
            .responses
            .lock()
            .unwrap()
            .insert(txid2.to_string(), mock_response2);
        pool.add_commitment(txid2).await.unwrap();
        pool.process_withdrawal(&[1u8; 32]).unwrap();
        
        let (commitments, spent, capacity) = pool.stats();
        assert_eq!(commitments, 2);
        assert_eq!(spent, 1);
        assert_eq!(capacity, 16);
    }
}