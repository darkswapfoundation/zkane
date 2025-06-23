//! Test helper functions and utilities

use zkane_common::*;
use zkane_crypto::*;
use zkane_core::*;

/// Create a test configuration for ZKane
pub fn create_test_config() -> ZKaneConfig {
    ZKaneConfig::new(
        SerializableAlkaneId { block: 2, tx: 1 },
        1000000,  // 1 BTC denomination
        20,       // Tree depth
        vec![],   // Empty verifier key
    )
}

/// Generate test deposit note
pub fn create_test_deposit_note() -> DepositNote {
    let asset_id = SerializableAlkaneId { block: 2, tx: 1 };
    let denomination = 1000000u128;
    DepositNote::random(asset_id, denomination)
}

/// Create test commitment
pub fn create_test_commitment() -> Commitment {
    let secret = Secret::random();
    let nullifier = Nullifier::random();
    generate_commitment(&nullifier, &secret).unwrap()
}

/// Create test merkle path
pub fn create_test_merkle_path() -> MerklePath {
    MerklePath {
        elements: vec![
            [0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90,
             0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90],
            [0x98, 0x76, 0x54, 0x32, 0x10, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10, 0xfe, 0xdc, 0xba,
             0x98, 0x76, 0x54, 0x32, 0x10, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10, 0xfe, 0xdc, 0xba],
        ],
        indices: vec![false, true],
    }
}

/// Create test withdrawal proof
pub fn create_test_withdrawal_proof() -> WithdrawalProof {
    WithdrawalProof {
        proof: vec![1, 2, 3, 4, 5], // Mock proof data
        nullifier_hash: NullifierHash([0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef,
                                       0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef]),
        merkle_root: [0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef,
                      0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef],
        recipient: 1000000u128, // 1 BTC in satoshis
    }
}

/// Assert that two commitments are equal
pub fn assert_commitments_equal(a: &Commitment, b: &Commitment) {
    assert_eq!(a.as_bytes(), b.as_bytes(), "Commitments should be equal");
}

/// Assert that a commitment is valid
pub fn assert_commitment_valid(commitment: &Commitment) {
    assert!(!commitment.as_bytes().iter().all(|&b| b == 0), "Commitment should not be all zeros");
    assert_eq!(commitment.as_bytes().len(), 32, "Commitment should be 32 bytes");
}

/// Assert that a nullifier hash is valid
pub fn assert_nullifier_hash_valid(hash: &NullifierHash) {
    assert!(!hash.as_bytes().iter().all(|&b| b == 0), "Nullifier hash should not be all zeros");
    assert_eq!(hash.as_bytes().len(), 32, "Nullifier hash should be 32 bytes");
}

/// Create a test privacy pool
pub fn create_test_privacy_pool() -> Result<PrivacyPool, ZKaneError> {
    let config = create_test_config();
    PrivacyPool::new(config)
}

/// Add test commitments to a pool
pub fn add_test_commitments_to_pool(pool: &mut PrivacyPool, count: usize) -> Result<Vec<Commitment>, ZKaneError> {
    let mut commitments = Vec::new();
    
    for _ in 0..count {
        let commitment = create_test_commitment();
        pool.add_commitment(&commitment)?;
        commitments.push(commitment);
    }
    
    Ok(commitments)
}

/// Verify that a pool has the expected state
pub fn assert_pool_state(pool: &PrivacyPool, expected_commitment_count: usize) {
    assert_eq!(pool.commitment_count(), expected_commitment_count as u64,
               "Pool should have {} commitments", expected_commitment_count);
    
    if expected_commitment_count > 0 {
        let root = pool.merkle_root();
        assert!(!root.iter().all(|&b| b == 0), "Merkle root should not be all zeros");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_functions() {
        // Test config creation
        let config = create_test_config();
        assert_eq!(config.denomination, 1000000);
        assert_eq!(config.tree_height, 20);

        // Test deposit note creation
        let note = create_test_deposit_note();
        assert_eq!(note.denomination, 1000000);
        assert!(!note.secret.0.is_empty());

        // Test commitment creation
        let commitment = create_test_commitment();
        assert_commitment_valid(&commitment);

        // Test pool creation
        let mut pool = create_test_privacy_pool().unwrap();
        assert_eq!(pool.commitment_count(), 0);

        // Test adding commitments
        let commitments = add_test_commitments_to_pool(&mut pool, 3).unwrap();
        assert_eq!(commitments.len(), 3);
        assert_pool_state(&pool, 3);
    }
}