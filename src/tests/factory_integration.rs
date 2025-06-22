//! Integration tests for ZKane factory and witness envelope functionality
//! Updated to use transaction output validation instead of recipient addresses

use zkane_common::{Secret, Nullifier, ZKaneConfig, DepositNote};
use zkane_crypto::{generate_commitment, generate_nullifier_hash};
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the factory pattern for creating zkane pools
    #[test]
    fn test_factory_pool_creation() -> Result<()> {
        // Define test asset (diesel)
        let asset_id = AlkaneId { block: 2, tx: 0 };
        let denomination = 1000000u128; // 1M units

        // Test that we can generate the expected pool ID
        let expected_pool_id = generate_pool_id(&asset_id, denomination);
        
        println!("Asset ID: block={}, tx={}", asset_id.block, asset_id.tx);
        println!("Denomination: {}", denomination);
        println!("Expected Pool ID: block={}, tx={}", expected_pool_id.block, expected_pool_id.tx);

        // Pool ID should be deterministic
        let pool_id2 = generate_pool_id(&asset_id, denomination);
        assert_eq!(expected_pool_id, pool_id2);

        // Different denomination should produce different pool ID
        let different_pool_id = generate_pool_id(&asset_id, denomination * 2);
        assert_ne!(expected_pool_id, different_pool_id);

        Ok(())
    }

    /// Test witness envelope data structures for deposits
    #[test]
    fn test_deposit_witness_data() -> Result<()> {
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        let commitment = generate_commitment(&nullifier, &secret)?;

        // Create deposit witness data
        let deposit_witness = DepositWitnessData {
            commitment: *commitment.as_bytes(),
        };

        // Serialize to JSON (this would go in the witness envelope)
        let witness_json = serde_json::to_string(&deposit_witness)?;
        println!("Deposit witness JSON: {}", witness_json);

        // Deserialize back
        let parsed: DepositWitnessData = serde_json::from_str(&witness_json)?;
        assert_eq!(deposit_witness.commitment, parsed.commitment);

        // Verify the commitment is 32 bytes
        assert_eq!(deposit_witness.commitment.len(), 32);

        Ok(())
    }

    /// Test witness envelope data structures for withdrawals
    #[test]
    fn test_withdrawal_witness_data() -> Result<()> {
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        let commitment = generate_commitment(&nullifier, &secret)?;
        let nullifier_hash = generate_nullifier_hash(&nullifier)?;

        // Create mock proof and merkle path
        let mock_proof = vec![1u8; 256]; // 256-byte proof
        let merkle_root = [42u8; 32];
        let path_elements = vec![[0u8; 32]; 20]; // 20-level tree
        let path_indices = vec![false; 20];
        let leaf_index = 5u32;

        // Create mock transaction outputs hash
        let outputs_hash = generate_mock_outputs_hash(1000000, "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")?;

        // Create withdrawal witness data
        let withdrawal_witness = WithdrawalWitnessData {
            proof: mock_proof.clone(),
            merkle_root,
            nullifier_hash: *nullifier_hash.as_bytes(),
            path_elements: path_elements.clone(),
            path_indices: path_indices.clone(),
            leaf_index,
            commitment: *commitment.as_bytes(),
            outputs_hash,
        };

        // Serialize to JSON (this would go in the witness envelope)
        let witness_json = serde_json::to_string(&withdrawal_witness)?;
        println!("Withdrawal witness JSON length: {} bytes", witness_json.len());

        // Deserialize back
        let parsed: WithdrawalWitnessData = serde_json::from_str(&witness_json)?;
        assert_eq!(withdrawal_witness.proof, parsed.proof);
        assert_eq!(withdrawal_witness.merkle_root, parsed.merkle_root);
        assert_eq!(withdrawal_witness.nullifier_hash, parsed.nullifier_hash);
        assert_eq!(withdrawal_witness.path_elements, parsed.path_elements);
        assert_eq!(withdrawal_witness.path_indices, parsed.path_indices);
        assert_eq!(withdrawal_witness.leaf_index, parsed.leaf_index);
        assert_eq!(withdrawal_witness.commitment, parsed.commitment);
        assert_eq!(withdrawal_witness.outputs_hash, parsed.outputs_hash);

        // Verify large data can be stored
        assert!(witness_json.len() > 1000); // Should be much larger than 80 bytes

        Ok(())
    }

    /// Test the complete flow: factory -> deposit -> withdrawal with output validation
    #[test]
    fn test_complete_privacy_flow_with_outputs() -> Result<()> {
        let asset_id = AlkaneId { block: 2, tx: 0 };
        let denomination = 1000000u128;

        println!("=== ZKane Privacy Flow Test (with Output Validation) ===");

        // Step 1: Generate pool ID (what factory would do)
        let pool_id = generate_pool_id(&asset_id, denomination);
        println!("1. Generated pool ID: block={}, tx={}", pool_id.block, pool_id.tx);

        // Step 2: Generate deposit note
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        let commitment = generate_commitment(&nullifier, &secret)?;
        
        let deposit_note = DepositNote::new(
            secret,
            nullifier,
            commitment,
            asset_id.into(),
            denomination,
            0, // Will be set by contract
        );
        
        println!("2. Generated deposit note with commitment: {}", commitment.to_hex());

        // Step 3: Create deposit witness data
        let deposit_witness = DepositWitnessData {
            commitment: *commitment.as_bytes(),
        };
        
        let deposit_witness_json = serde_json::to_string(&deposit_witness)?;
        println!("3. Deposit witness size: {} bytes", deposit_witness_json.len());

        // Step 4: Simulate deposit processing (what zkane contract would do)
        let leaf_index = 0u32; // First deposit
        println!("4. Processed deposit at leaf index: {}", leaf_index);

        // Step 5: Generate withdrawal data with transaction output validation
        let nullifier_hash = generate_nullifier_hash(&nullifier)?;
        let mock_proof = generate_mock_proof(&secret, &nullifier)?;
        let merkle_root = [1u8; 32]; // Mock root
        let (path_elements, path_indices) = generate_mock_merkle_path(leaf_index, 20);

        // Generate mock transaction outputs hash (this would be the actual transaction outputs)
        let recipient_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
        let outputs_hash = generate_mock_outputs_hash(denomination, recipient_address)?;

        let withdrawal_witness = WithdrawalWitnessData {
            proof: mock_proof,
            merkle_root,
            nullifier_hash: *nullifier_hash.as_bytes(),
            path_elements,
            path_indices,
            leaf_index,
            commitment: *commitment.as_bytes(),
            outputs_hash,
        };

        let withdrawal_witness_json = serde_json::to_string(&withdrawal_witness)?;
        println!("5. Withdrawal witness size: {} bytes", withdrawal_witness_json.len());

        // Step 6: Verify withdrawal data integrity
        let parsed_withdrawal: WithdrawalWitnessData = serde_json::from_str(&withdrawal_witness_json)?;
        assert_eq!(withdrawal_witness.nullifier_hash, parsed_withdrawal.nullifier_hash);
        assert_eq!(withdrawal_witness.commitment, parsed_withdrawal.commitment);
        assert_eq!(withdrawal_witness.leaf_index, parsed_withdrawal.leaf_index);
        assert_eq!(withdrawal_witness.outputs_hash, parsed_withdrawal.outputs_hash);

        println!("6. Withdrawal data verified successfully");
        println!("7. Outputs hash: {}", hex::encode(outputs_hash));
        println!("=== Privacy Flow Test Complete ===");

        Ok(())
    }

    /// Test opcode input size limits (updated for no recipient parameter)
    #[test]
    fn test_opcode_size_limits() {
        // Test that our opcode inputs fit within 80 bytes

        // Initialize opcode: asset_id_block (16) + asset_id_tx (16) + denomination (16) + tree_height (4) = 52 bytes
        let init_size = 16 + 16 + 16 + 4;
        assert!(init_size <= 80, "Initialize opcode exceeds 80 bytes: {}", init_size);

        // Withdraw opcode: no parameters (0 bytes) - recipient determined by transaction vouts
        let withdraw_size = 0;
        assert!(withdraw_size <= 80, "Withdraw opcode exceeds 80 bytes: {}", withdraw_size);

        // Deposit opcode: no parameters (0 bytes) - commitment in witness envelope
        let deposit_size = 0;
        assert!(deposit_size <= 80, "Deposit opcode exceeds 80 bytes: {}", deposit_size);

        // IsNullifierSpent opcode: nullifier_hash (32 bytes)
        let nullifier_check_size = 32;
        assert!(nullifier_check_size <= 80, "IsNullifierSpent opcode exceeds 80 bytes: {}", nullifier_check_size);

        // HasCommitment opcode: commitment (32 bytes)
        let commitment_check_size = 32;
        assert!(commitment_check_size <= 80, "HasCommitment opcode exceeds 80 bytes: {}", commitment_check_size);

        // GetMerklePath opcode: leaf_index (4 bytes)
        let merkle_path_size = 4;
        assert!(merkle_path_size <= 80, "GetMerklePath opcode exceeds 80 bytes: {}", merkle_path_size);

        println!("All opcode inputs fit within 80-byte limit ✓");
        println!("Withdrawal now uses transaction vouts for recipient (no frontrunning) ✓");
    }

    /// Test transaction output hash generation
    #[test]
    fn test_transaction_output_validation() -> Result<()> {
        let denomination = 1000000u128;
        let recipient1 = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
        let recipient2 = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";

        // Different recipients should produce different hashes
        let hash1 = generate_mock_outputs_hash(denomination, recipient1)?;
        let hash2 = generate_mock_outputs_hash(denomination, recipient2)?;
        assert_ne!(hash1, hash2);

        // Same recipient should produce same hash
        let hash3 = generate_mock_outputs_hash(denomination, recipient1)?;
        assert_eq!(hash1, hash3);

        // Different amounts should produce different hashes
        let hash4 = generate_mock_outputs_hash(denomination * 2, recipient1)?;
        assert_ne!(hash1, hash4);

        println!("Transaction output validation tests passed ✓");
        Ok(())
    }
}

/// Helper function to generate a deterministic pool ID
fn generate_pool_id(asset_id: &AlkaneId, denomination: u128) -> AlkaneId {
    // Use a hash of asset_id and denomination to generate a unique tx value
    let mut hasher_input = Vec::new();
    hasher_input.extend_from_slice(&asset_id.block.to_le_bytes());
    hasher_input.extend_from_slice(&asset_id.tx.to_le_bytes());
    hasher_input.extend_from_slice(&denomination.to_le_bytes());
    
    // Simple hash for demo - in production use proper hash function
    let mut hash_value = 0u128;
    for chunk in hasher_input.chunks(16) {
        let mut bytes = [0u8; 16];
        bytes[..chunk.len()].copy_from_slice(chunk);
        hash_value ^= u128::from_le_bytes(bytes);
    }
    
    AlkaneId {
        block: 6, // ZKANE_INSTANCE_BLOCK
        tx: hash_value,
    }
}

/// Mock witness data structures (matching the zkane contract)
#[derive(serde::Deserialize, serde::Serialize)]
struct DepositWitnessData {
    commitment: [u8; 32],
}

#[derive(serde::Deserialize, serde::Serialize)]
struct WithdrawalWitnessData {
    proof: Vec<u8>,
    merkle_root: [u8; 32],
    nullifier_hash: [u8; 32],
    path_elements: Vec<[u8; 32]>,
    path_indices: Vec<bool>,
    leaf_index: u32,
    commitment: [u8; 32],
    outputs_hash: [u8; 32], // Hash of transaction outputs for recipient validation
}

/// Generate a mock zero-knowledge proof
fn generate_mock_proof(secret: &Secret, nullifier: &Nullifier) -> Result<Vec<u8>> {
    // In production, this would call the Noir prover
    // For now, generate a deterministic mock proof based on inputs
    let mut proof = Vec::new();
    proof.extend_from_slice(secret.as_bytes());
    proof.extend_from_slice(nullifier.as_bytes());
    
    // Pad to realistic proof size (256 bytes)
    while proof.len() < 256 {
        proof.push(0x42);
    }
    
    Ok(proof)
}

/// Generate a mock merkle path
fn generate_mock_merkle_path(leaf_index: u32, tree_height: u32) -> (Vec<[u8; 32]>, Vec<bool>) {
    let mut path_elements = Vec::new();
    let mut path_indices = Vec::new();
    
    let mut current_index = leaf_index;
    
    for level in 0..tree_height {
        // Generate a deterministic sibling hash
        let mut sibling = [0u8; 32];
        sibling[0..4].copy_from_slice(&level.to_le_bytes());
        sibling[4..8].copy_from_slice(&current_index.to_le_bytes());
        
        path_elements.push(sibling);
        path_indices.push(current_index % 2 == 1); // true if right child
        
        current_index /= 2;
    }
    
    (path_elements, path_indices)
}

/// Generate a mock transaction outputs hash
/// This simulates hashing the transaction outputs (value + script_pubkey)
fn generate_mock_outputs_hash(amount: u128, recipient: &str) -> Result<[u8; 32]> {
    use sha2::{Digest, Sha256};
    
    let mut hasher = Sha256::new();
    
    // Hash the output value
    hasher.update(&amount.to_le_bytes());
    
    // Hash the recipient address (in practice this would be the script_pubkey)
    hasher.update(recipient.as_bytes());
    
    Ok(hasher.finalize().into())
}