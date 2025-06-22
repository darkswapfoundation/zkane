//! Poseidon hash function implementation for ZKane
//! 
//! This is a simplified implementation for demonstration purposes.
//! In production, you would use a proper Poseidon implementation
//! that matches the one used in your Noir circuits.

use anyhow::Result;
use ark_ff::{PrimeField, Field, Zero};
use ark_bn254::Fr as Bn254Fr;
use ark_std::vec::Vec;

/// Poseidon hash function using BN254 scalar field
/// 
/// Note: This is a placeholder implementation. In production, you should use
/// a proper Poseidon implementation that matches your Noir circuit exactly.
pub fn poseidon_hash(input: &[u8]) -> Result<[u8; 32]> {
    // Convert bytes to field elements
    let field_elements = bytes_to_field_elements(input)?;
    
    // Apply Poseidon permutation (simplified)
    let result = poseidon_permutation(&field_elements)?;
    
    // Convert back to bytes
    field_element_to_bytes(&result)
}

/// Convert bytes to BN254 field elements
fn bytes_to_field_elements(input: &[u8]) -> Result<Vec<Bn254Fr>> {
    let mut elements = Vec::new();
    
    // Process input in 31-byte chunks (to stay within field size)
    for chunk in input.chunks(31) {
        let mut bytes = [0u8; 32];
        bytes[1..chunk.len() + 1].copy_from_slice(chunk);
        
        let element = Bn254Fr::from_le_bytes_mod_order(&bytes);
        elements.push(element);
    }
    
    // Ensure we have at least one element
    if elements.is_empty() {
        elements.push(Bn254Fr::zero());
    }
    
    Ok(elements)
}

/// Convert a field element back to bytes
fn field_element_to_bytes(element: &Bn254Fr) -> Result<[u8; 32]> {
    use ark_serialize::CanonicalSerialize;
    
    let mut bytes = Vec::new();
    element.serialize_compressed(&mut bytes)?;
    
    // Pad or truncate to 32 bytes
    let mut result = [0u8; 32];
    let len = std::cmp::min(bytes.len(), 32);
    result[..len].copy_from_slice(&bytes[..len]);
    
    Ok(result)
}

/// Simplified Poseidon permutation
/// 
/// This is a placeholder implementation. In production, you need to use
/// the exact same Poseidon parameters and implementation as your Noir circuit.
fn poseidon_permutation(input: &[Bn254Fr]) -> Result<Bn254Fr> {
    // For now, just sum all elements and square the result
    // This is NOT a secure hash function - just a placeholder
    let mut result = Bn254Fr::zero();
    
    for element in input {
        result += element;
    }
    
    // Apply some simple operations to mix the input
    result = result.square();
    result += Bn254Fr::from(1u64);
    result = result.square();
    
    Ok(result)
}

/// Poseidon hash for two field elements (common case)
pub fn poseidon_hash_two(left: &[u8; 32], right: &[u8; 32]) -> Result<[u8; 32]> {
    let mut input = Vec::with_capacity(64);
    input.extend_from_slice(left);
    input.extend_from_slice(right);
    poseidon_hash(&input)
}

/// Poseidon hash for a single 32-byte input
pub fn poseidon_hash_single(input: &[u8; 32]) -> Result<[u8; 32]> {
    poseidon_hash(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poseidon_hash_deterministic() {
        let input = b"hello world";
        let hash1 = poseidon_hash(input).unwrap();
        let hash2 = poseidon_hash(input).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_poseidon_hash_different_inputs() {
        let input1 = b"hello world";
        let input2 = b"hello world!";
        
        let hash1 = poseidon_hash(input1).unwrap();
        let hash2 = poseidon_hash(input2).unwrap();
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_poseidon_hash_two() {
        let left = [1u8; 32];
        let right = [2u8; 32];
        
        let hash1 = poseidon_hash_two(&left, &right).unwrap();
        let hash2 = poseidon_hash_two(&left, &right).unwrap();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_poseidon_hash_single() {
        let input = [42u8; 32];
        
        let hash1 = poseidon_hash_single(&input).unwrap();
        let hash2 = poseidon_hash_single(&input).unwrap();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_bytes_to_field_elements() {
        let input = b"test";
        let elements = bytes_to_field_elements(input).unwrap();
        assert!(!elements.is_empty());
    }

    #[test]
    fn test_field_element_to_bytes() {
        let element = Bn254Fr::from(42u64);
        let bytes = field_element_to_bytes(&element).unwrap();
        assert_eq!(bytes.len(), 32);
    }
}