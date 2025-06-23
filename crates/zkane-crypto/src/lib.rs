//! # ZKane Cryptographic Primitives
//!
//! This crate provides the cryptographic building blocks for the ZKane privacy pool system.
//! It implements the core cryptographic operations needed for privacy-preserving transactions,
//! including hash functions, Merkle trees, and commitment schemes.
//!
//! ## Overview
//!
//! The ZKane cryptographic system is built on the following primitives:
//!
//! - **Poseidon Hash Function**: A zero-knowledge friendly hash function optimized for
//!   use in arithmetic circuits
//! - **Merkle Trees**: Binary trees for efficient commitment storage and inclusion proofs
//! - **Commitment Scheme**: Cryptographic commitments that hide secrets while enabling
//!   zero-knowledge proofs
//!
//! ## Security Model
//!
//! The cryptographic security of ZKane relies on:
//!
//! 1. **Hash Function Security**: Poseidon provides collision resistance and preimage resistance
//! 2. **Commitment Hiding**: Commitments reveal no information about the underlying secret
//! 3. **Commitment Binding**: Computationally infeasible to find different inputs producing
//!    the same commitment
//! 4. **Merkle Tree Integrity**: Tree structure ensures tamper-evident commitment storage
//!
//! ## Example Usage
//!
//! ```rust
//! use zkane_crypto::{generate_commitment, generate_nullifier_hash, MerkleTree};
//! use zkane_common::{Secret, Nullifier};
//!
//! // Generate cryptographic values
//! let secret = Secret::random();
//! let nullifier = Nullifier::random();
//!
//! // Create commitment
//! let commitment = generate_commitment(&nullifier, &secret)?;
//!
//! // Generate nullifier hash for public use
//! let nullifier_hash = generate_nullifier_hash(&nullifier)?;
//!
//! // Create and use Merkle tree
//! let mut tree = MerkleTree::new(20); // 20-level tree
//! let leaf_index = tree.insert(&commitment)?;
//! let merkle_root = tree.root();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Performance Considerations
//!
//! - **Poseidon Hashing**: Optimized for zero-knowledge circuits but slower than SHA-256
//!   for general use
//! - **Merkle Tree Operations**: O(log n) insertion and proof generation
//! - **Memory Usage**: Trees store all intermediate nodes for efficient proof generation
//!
//! ## Zero-Knowledge Compatibility
//!
//! All cryptographic primitives in this crate are designed to be compatible with
//! zero-knowledge proof systems, particularly Noir circuits. The Poseidon hash function
//! is specifically chosen for its efficiency in arithmetic circuits.

pub mod hash;
pub mod poseidon;
pub mod merkle;

use anyhow::Result;
use zkane_common::{Secret, Nullifier, Commitment, NullifierHash};

pub use hash::*;
pub use poseidon::*;
pub use merkle::*;

/// Generate a commitment from a nullifier and secret.
///
/// This function creates a cryptographic commitment that binds a nullifier and secret
/// together while hiding both values. The commitment can be safely published without
/// revealing the underlying secret or nullifier.
///
/// # Cryptographic Properties
///
/// - **Hiding**: The commitment reveals no information about the secret or nullifier
/// - **Binding**: It's computationally infeasible to find different secret/nullifier
///   pairs that produce the same commitment
/// - **Deterministic**: Same inputs always produce the same commitment
///
/// # Arguments
///
/// * `nullifier` - The nullifier value (will be revealed during withdrawal)
/// * `secret` - The secret value (must remain private)
///
/// # Returns
///
/// A `Result` containing the commitment or an error if the cryptographic operation fails.
///
/// # Example
///
/// ```rust
/// use zkane_crypto::generate_commitment;
/// use zkane_common::{Secret, Nullifier};
///
/// let secret = Secret::random();
/// let nullifier = Nullifier::random();
/// let commitment = generate_commitment(&nullifier, &secret)?;
///
/// // The commitment can be safely published
/// println!("Commitment: {}", commitment.to_hex());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Security Notes
///
/// - The secret must be kept private to maintain privacy
/// - The nullifier will be revealed during withdrawal
/// - Both values should be generated using secure randomness
pub fn generate_commitment(nullifier: &Nullifier, secret: &Secret) -> Result<Commitment> {
    let hash_result = poseidon_hash_two(nullifier.as_bytes(), secret.as_bytes())?;
    Ok(Commitment::new(hash_result))
}

/// Generate a nullifier hash from a nullifier.
///
/// This function creates a one-way hash of a nullifier that can be safely published
/// during withdrawal to prevent double-spending. The hash prevents linking the
/// withdrawal back to the original nullifier while still enabling duplicate detection.
///
/// # Cryptographic Properties
///
/// - **One-way**: Computationally infeasible to derive the nullifier from its hash
/// - **Deterministic**: Same nullifier always produces the same hash
/// - **Collision-resistant**: Different nullifiers produce different hashes
///
/// # Arguments
///
/// * `nullifier` - The nullifier to hash
///
/// # Returns
///
/// A `Result` containing the nullifier hash or an error.
///
/// # Example
///
/// ```rust
/// use zkane_crypto::generate_nullifier_hash;
/// use zkane_common::Nullifier;
///
/// let nullifier = Nullifier::random();
/// let nullifier_hash = generate_nullifier_hash(&nullifier)?;
///
/// // The hash can be safely published during withdrawal
/// println!("Nullifier hash: {}", nullifier_hash.to_hex());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Security Notes
///
/// - The original nullifier should be kept private until withdrawal
/// - The hash is published during withdrawal to prevent double-spending
/// - Multiple withdrawals with the same nullifier hash will be rejected
pub fn generate_nullifier_hash(nullifier: &Nullifier) -> Result<NullifierHash> {
    let hash_result = poseidon_hash_single(nullifier.as_bytes())?;
    Ok(NullifierHash::new(hash_result))
}

/// Verify that a commitment was correctly generated from a nullifier and secret.
///
/// This function verifies the integrity of a commitment by recomputing it from
/// the provided nullifier and secret and comparing with the expected commitment.
///
/// # Arguments
///
/// * `commitment` - The commitment to verify
/// * `nullifier` - The nullifier used to generate the commitment
/// * `secret` - The secret used to generate the commitment
///
/// # Returns
///
/// `true` if the commitment is valid, `false` otherwise.
///
/// # Example
///
/// ```rust
/// use zkane_crypto::{generate_commitment, verify_commitment};
/// use zkane_common::{Secret, Nullifier};
///
/// let secret = Secret::random();
/// let nullifier = Nullifier::random();
/// let commitment = generate_commitment(&nullifier, &secret)?;
///
/// // Verify the commitment
/// assert!(verify_commitment(&commitment, &nullifier, &secret)?);
///
/// // Wrong secret should fail verification
/// let wrong_secret = Secret::random();
/// assert!(!verify_commitment(&commitment, &nullifier, &wrong_secret)?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn verify_commitment(
    commitment: &Commitment,
    nullifier: &Nullifier,
    secret: &Secret,
) -> Result<bool> {
    let computed_commitment = generate_commitment(nullifier, secret)?;
    Ok(commitment == &computed_commitment)
}

/// Verify that a nullifier hash was correctly generated from a nullifier.
///
/// This function verifies the integrity of a nullifier hash by recomputing it
/// from the provided nullifier and comparing with the expected hash.
///
/// # Arguments
///
/// * `nullifier_hash` - The nullifier hash to verify
/// * `nullifier` - The nullifier used to generate the hash
///
/// # Returns
///
/// `true` if the nullifier hash is valid, `false` otherwise.
///
/// # Example
///
/// ```rust
/// use zkane_crypto::{generate_nullifier_hash, verify_nullifier_hash};
/// use zkane_common::Nullifier;
///
/// let nullifier = Nullifier::random();
/// let nullifier_hash = generate_nullifier_hash(&nullifier)?;
///
/// // Verify the nullifier hash
/// assert!(verify_nullifier_hash(&nullifier_hash, &nullifier)?);
///
/// // Wrong nullifier should fail verification
/// let wrong_nullifier = Nullifier::random();
/// assert!(!verify_nullifier_hash(&nullifier_hash, &wrong_nullifier)?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn verify_nullifier_hash(
    nullifier_hash: &NullifierHash,
    nullifier: &Nullifier,
) -> Result<bool> {
    let computed_hash = generate_nullifier_hash(nullifier)?;
    Ok(nullifier_hash == &computed_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_generation() {
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        
        let commitment1 = generate_commitment(&nullifier, &secret).unwrap();
        let commitment2 = generate_commitment(&nullifier, &secret).unwrap();
        
        // Same inputs should produce same commitment
        assert_eq!(commitment1, commitment2);
    }

    #[test]
    fn test_commitment_verification() {
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        let commitment = generate_commitment(&nullifier, &secret).unwrap();
        
        // Correct values should verify
        assert!(verify_commitment(&commitment, &nullifier, &secret).unwrap());
        
        // Wrong secret should fail
        let wrong_secret = Secret::random();
        assert!(!verify_commitment(&commitment, &nullifier, &wrong_secret).unwrap());
        
        // Wrong nullifier should fail
        let wrong_nullifier = Nullifier::random();
        assert!(!verify_commitment(&commitment, &wrong_nullifier, &secret).unwrap());
    }

    #[test]
    fn test_nullifier_hash_generation() {
        let nullifier = Nullifier::random();
        
        let hash1 = generate_nullifier_hash(&nullifier).unwrap();
        let hash2 = generate_nullifier_hash(&nullifier).unwrap();
        
        // Same nullifier should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_nullifier_hash_verification() {
        let nullifier = Nullifier::random();
        let nullifier_hash = generate_nullifier_hash(&nullifier).unwrap();
        
        // Correct nullifier should verify
        assert!(verify_nullifier_hash(&nullifier_hash, &nullifier).unwrap());
        
        // Wrong nullifier should fail
        let wrong_nullifier = Nullifier::random();
        assert!(!verify_nullifier_hash(&nullifier_hash, &wrong_nullifier).unwrap());
    }

    #[test]
    fn test_different_inputs_produce_different_outputs() {
        let secret1 = Secret::random();
        let secret2 = Secret::random();
        let nullifier1 = Nullifier::random();
        let nullifier2 = Nullifier::random();
        
        let commitment1 = generate_commitment(&nullifier1, &secret1).unwrap();
        let commitment2 = generate_commitment(&nullifier2, &secret2).unwrap();
        
        // Different inputs should produce different commitments
        assert_ne!(commitment1, commitment2);
        
        let hash1 = generate_nullifier_hash(&nullifier1).unwrap();
        let hash2 = generate_nullifier_hash(&nullifier2).unwrap();
        
        // Different nullifiers should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_merkle_tree_integration() {
        let mut tree = MerkleTree::new(4);
        
        // Generate some commitments
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        let commitment = generate_commitment(&nullifier, &secret).unwrap();
        
        // Insert into tree
        let leaf_index = tree.insert(&commitment).unwrap();
        assert_eq!(leaf_index, 0);
        
        // Get merkle root
        let root = tree.root();
        assert_ne!(root, [0u8; 32]);
        
        // Generate proof
        let proof = tree.generate_path(leaf_index).unwrap();
        assert!(!proof.is_empty());
    }
}