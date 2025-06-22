//! Hash function implementations for ZKane

use anyhow::Result;
use sha2::{Digest, Sha256};
use blake2::{Blake2b512, Blake2s256};

/// SHA-256 hash function
pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

/// Blake2s-256 hash function
pub fn blake2s(input: &[u8]) -> [u8; 32] {
    let mut hasher = Blake2s256::new();
    hasher.update(input);
    hasher.finalize().into()
}

/// Blake2b-512 hash function
pub fn blake2b(input: &[u8]) -> [u8; 64] {
    let mut hasher = Blake2b512::new();
    hasher.update(input);
    hasher.finalize().into()
}

/// Merkle tree hash function (using Blake2s for efficiency)
pub fn merkle_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut input = Vec::with_capacity(64);
    input.extend_from_slice(left);
    input.extend_from_slice(right);
    blake2s(&input)
}

/// Hash a leaf value for merkle tree inclusion
pub fn hash_leaf(leaf: &[u8; 32]) -> [u8; 32] {
    // Prefix with 0x00 to distinguish from internal nodes
    let mut input = Vec::with_capacity(33);
    input.push(0x00);
    input.extend_from_slice(leaf);
    blake2s(&input)
}

/// Hash an internal node for merkle tree
pub fn hash_internal(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    // Prefix with 0x01 to distinguish from leaf nodes
    let mut input = Vec::with_capacity(65);
    input.push(0x01);
    input.extend_from_slice(left);
    input.extend_from_slice(right);
    blake2s(&input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_lit::hex;

    #[test]
    fn test_sha256() {
        let input = b"hello world";
        let expected = hex!("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
        assert_eq!(sha256(input), expected);
    }

    #[test]
    fn test_blake2s() {
        let input = b"hello world";
        let result = blake2s(input);
        // Blake2s should produce deterministic output
        assert_eq!(blake2s(input), result);
    }

    #[test]
    fn test_merkle_hash_deterministic() {
        let left = [1u8; 32];
        let right = [2u8; 32];
        
        let hash1 = merkle_hash(&left, &right);
        let hash2 = merkle_hash(&left, &right);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_merkle_hash_different_order() {
        let left = [1u8; 32];
        let right = [2u8; 32];
        
        let hash1 = merkle_hash(&left, &right);
        let hash2 = merkle_hash(&right, &left);
        
        // Different order should produce different hash
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_leaf_vs_internal_hash() {
        let data = [1u8; 32];
        let other = [2u8; 32];
        
        let leaf_hash = hash_leaf(&data);
        let internal_hash = hash_internal(&data, &other);
        
        // Leaf and internal hashes should be different even with same input
        assert_ne!(leaf_hash, internal_hash);
    }
}