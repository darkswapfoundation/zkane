//! Test helpers for ZKane - aligned with boiler pattern

use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;

/// Convert a vector of u128 values into a Cellpack
/// This matches the boiler pattern exactly
pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
    Cellpack {
        target: AlkaneId {
            block: v[0],
            tx: v[1]
        },
        inputs: v[2..].into()
    }
}

/// Privacy-specific helper to generate commitment hashes
pub fn generate_commitment(secret: u128, nullifier: u128) -> u128 {
    // Simple hash function for testing - in production would use proper cryptographic hash
    secret.wrapping_mul(nullifier).wrapping_add(0xdeadbeef)
}

/// Privacy-specific helper to generate nullifier hashes
pub fn generate_nullifier(secret: u128, leaf_index: u128) -> u128 {
    // Simple hash function for testing - in production would use proper cryptographic hash
    secret.wrapping_add(leaf_index).wrapping_mul(0xcafebabe)
}

/// Generate a mock Merkle proof for testing
pub fn generate_mock_proof(leaf_index: u128, tree_depth: u32) -> Vec<u128> {
    let mut proof = Vec::new();
    let mut index = leaf_index;
    
    for i in 0..tree_depth {
        // Generate mock sibling hash
        let sibling = index.wrapping_mul(0x123456789abcdef).wrapping_add(i as u128);
        proof.push(sibling);
        index /= 2;
    }
    
    proof
}

/// Verify privacy pool parameters are valid
pub fn verify_privacy_params(
    commitment: u128,
    nullifier: u128,
    proof: &[u128],
    tree_depth: u32
) -> bool {
    // Basic validation
    commitment != 0 && 
    nullifier != 0 && 
    proof.len() == tree_depth as usize &&
    !proof.iter().any(|&x| x == 0)
}

/// Calculate expected anonymity set size for a given number of deposits
pub fn calculate_anonymity_set_size(num_deposits: u128, tree_depth: u32) -> u128 {
    let max_capacity = 2u128.pow(tree_depth);
    num_deposits.min(max_capacity)
}

/// Estimate privacy score based on anonymity set size
pub fn estimate_privacy_score(anonymity_set_size: u128) -> f64 {
    if anonymity_set_size <= 1 {
        0.0
    } else {
        (anonymity_set_size as f64).log2() / 20.0 // Normalize to 0-1 range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_cellpack() {
        let v = vec![1u128, 2u128, 3u128, 4u128];
        let cellpack = into_cellpack(v.clone());
        
        assert_eq!(cellpack.target.block, 1);
        assert_eq!(cellpack.target.tx, 2);
        assert_eq!(cellpack.inputs, vec![3u128, 4u128]);
    }

    #[test]
    fn test_generate_commitment() {
        let secret = 12345u128;
        let nullifier = 67890u128;
        let commitment = generate_commitment(secret, nullifier);
        
        // Should be deterministic
        assert_eq!(commitment, generate_commitment(secret, nullifier));
        
        // Should be different for different inputs
        assert_ne!(commitment, generate_commitment(secret + 1, nullifier));
    }

    #[test]
    fn test_generate_nullifier() {
        let secret = 12345u128;
        let leaf_index = 42u128;
        let nullifier = generate_nullifier(secret, leaf_index);
        
        // Should be deterministic
        assert_eq!(nullifier, generate_nullifier(secret, leaf_index));
        
        // Should be different for different inputs
        assert_ne!(nullifier, generate_nullifier(secret, leaf_index + 1));
    }

    #[test]
    fn test_generate_mock_proof() {
        let leaf_index = 5u128;
        let tree_depth = 10u32;
        let proof = generate_mock_proof(leaf_index, tree_depth);
        
        assert_eq!(proof.len(), tree_depth as usize);
        assert!(proof.iter().all(|&x| x != 0));
    }

    #[test]
    fn test_verify_privacy_params() {
        let commitment = 12345u128;
        let nullifier = 67890u128;
        let proof = vec![1u128, 2u128, 3u128];
        
        assert!(verify_privacy_params(commitment, nullifier, &proof, 3));
        assert!(!verify_privacy_params(0, nullifier, &proof, 3)); // Zero commitment
        assert!(!verify_privacy_params(commitment, 0, &proof, 3)); // Zero nullifier
        assert!(!verify_privacy_params(commitment, nullifier, &proof, 4)); // Wrong proof length
    }

    #[test]
    fn test_calculate_anonymity_set_size() {
        assert_eq!(calculate_anonymity_set_size(100, 10), 100); // Within capacity
        assert_eq!(calculate_anonymity_set_size(2000, 10), 1024); // Exceeds capacity (2^10 = 1024)
    }

    #[test]
    fn test_estimate_privacy_score() {
        assert_eq!(estimate_privacy_score(1), 0.0); // No privacy
        assert!(estimate_privacy_score(2) > 0.0); // Some privacy
        assert!(estimate_privacy_score(1024) > estimate_privacy_score(2)); // More privacy
    }
}