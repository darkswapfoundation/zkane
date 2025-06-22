//! Cryptographic primitives for ZKane privacy pools

use anyhow::Result;
use zkane_common::{Commitment, Nullifier, NullifierHash, Secret, ZKaneError, ZKaneResult};

pub mod hash;
pub mod merkle;
pub mod poseidon;

pub use hash::*;
pub use merkle::*;
pub use poseidon::*;

/// Generate a commitment from a nullifier and secret
pub fn generate_commitment(nullifier: &Nullifier, secret: &Secret) -> ZKaneResult<Commitment> {
    let mut input = Vec::new();
    input.extend_from_slice(nullifier.as_bytes());
    input.extend_from_slice(secret.as_bytes());
    
    let hash = poseidon_hash(&input)
        .map_err(|e| ZKaneError::CryptoError(format!("Failed to generate commitment: {}", e)))?;
    
    Ok(Commitment::new(hash))
}

/// Generate a nullifier hash from a nullifier
pub fn generate_nullifier_hash(nullifier: &Nullifier) -> ZKaneResult<NullifierHash> {
    let hash = poseidon_hash(nullifier.as_bytes())
        .map_err(|e| ZKaneError::CryptoError(format!("Failed to generate nullifier hash: {}", e)))?;
    
    Ok(NullifierHash::new(hash))
}

/// Verify that a commitment was generated from the given nullifier and secret
pub fn verify_commitment(
    commitment: &Commitment,
    nullifier: &Nullifier,
    secret: &Secret,
) -> ZKaneResult<bool> {
    let expected_commitment = generate_commitment(nullifier, secret)?;
    Ok(commitment == &expected_commitment)
}

/// Verify that a nullifier hash was generated from the given nullifier
pub fn verify_nullifier_hash(
    nullifier_hash: &NullifierHash,
    nullifier: &Nullifier,
) -> ZKaneResult<bool> {
    let expected_hash = generate_nullifier_hash(nullifier)?;
    Ok(nullifier_hash == &expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkane_common::{Nullifier, Secret};

    #[test]
    fn test_commitment_generation() {
        let nullifier = Nullifier::random();
        let secret = Secret::random();
        
        let commitment = generate_commitment(&nullifier, &secret).unwrap();
        assert!(verify_commitment(&commitment, &nullifier, &secret).unwrap());
        
        // Test with different secret should fail
        let different_secret = Secret::random();
        assert!(!verify_commitment(&commitment, &nullifier, &different_secret).unwrap());
    }

    #[test]
    fn test_nullifier_hash_generation() {
        let nullifier = Nullifier::random();
        
        let nullifier_hash = generate_nullifier_hash(&nullifier).unwrap();
        assert!(verify_nullifier_hash(&nullifier_hash, &nullifier).unwrap());
        
        // Test with different nullifier should fail
        let different_nullifier = Nullifier::random();
        assert!(!verify_nullifier_hash(&nullifier_hash, &different_nullifier).unwrap());
    }

    #[test]
    fn test_deterministic_commitment() {
        let nullifier = Nullifier::new([1u8; 32]);
        let secret = Secret::new([2u8; 32]);
        
        let commitment1 = generate_commitment(&nullifier, &secret).unwrap();
        let commitment2 = generate_commitment(&nullifier, &secret).unwrap();
        
        assert_eq!(commitment1, commitment2);
    }

    #[test]
    fn test_deterministic_nullifier_hash() {
        let nullifier = Nullifier::new([1u8; 32]);
        
        let hash1 = generate_nullifier_hash(&nullifier).unwrap();
        let hash2 = generate_nullifier_hash(&nullifier).unwrap();
        
        assert_eq!(hash1, hash2);
    }
}