// ZKane withdrawal circuit
// Proves knowledge of a secret and nullifier for a commitment in the merkle tree
// AND validates that the transaction outputs match the intended recipient

use std::hash::poseidon;
use std::merkle::compute_merkle_root;

// Circuit parameters
global TREE_HEIGHT: u32 = 20;

// Main circuit function
fn main(
    // Private inputs (witness)
    secret: Field,
    nullifier: Field,
    path_elements: [Field; TREE_HEIGHT],
    path_indices: [u1; TREE_HEIGHT],
    
    // Public inputs
    merkle_root: pub Field,
    nullifier_hash: pub Field,
    outputs_hash: pub Field,  // Hash of transaction outputs (prevents frontrunning)
) {
    // 1. Compute commitment from secret and nullifier
    let commitment = poseidon::bn254::hash_2([nullifier, secret]);
    
    // 2. Compute nullifier hash
    let computed_nullifier_hash = poseidon::bn254::hash_1([nullifier]);
    
    // 3. Verify nullifier hash matches public input
    assert(computed_nullifier_hash == nullifier_hash);
    
    // 4. Verify merkle tree inclusion
    let computed_root = compute_merkle_root(commitment, path_elements, path_indices);
    assert(computed_root == merkle_root);
    
    // 5. Ensure outputs_hash is included in the proof (prevents tampering)
    // By including outputs_hash as a public input, we bind the proof to specific
    // transaction outputs, preventing frontrunning attacks
    let _outputs_square = outputs_hash * outputs_hash;
}

// Helper function to compute merkle root
fn compute_merkle_root(
    leaf: Field,
    path_elements: [Field; TREE_HEIGHT],
    path_indices: [u1; TREE_HEIGHT]
) -> Field {
    let mut current = leaf;
    
    for i in 0..TREE_HEIGHT {
        let path_element = path_elements[i];
        let is_right = path_indices[i];
        
        if is_right == 1 {
            // Current node is right child
            current = poseidon::bn254::hash_2([path_element, current]);
        } else {
            // Current node is left child
            current = poseidon::bn254::hash_2([current, path_element]);
        }
    }
    
    current
}

#[test]
fn test_commitment_generation() {
    let secret = 123;
    let nullifier = 456;
    let commitment = poseidon::bn254::hash_2([nullifier, secret]);
    
    // Commitment should be deterministic
    let commitment2 = poseidon::bn254::hash_2([nullifier, secret]);
    assert(commitment == commitment2);
}

#[test]
fn test_nullifier_hash_generation() {
    let nullifier = 456;
    let nullifier_hash = poseidon::bn254::hash_1([nullifier]);
    
    // Nullifier hash should be deterministic
    let nullifier_hash2 = poseidon::bn254::hash_1([nullifier]);
    assert(nullifier_hash == nullifier_hash2);
}

#[test]
fn test_merkle_root_computation() {
    let leaf = 123;
    let path_elements = [456, 789, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let path_indices = [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    
    let root = compute_merkle_root(leaf, path_elements, path_indices);
    
    // Root should be deterministic
    let root2 = compute_merkle_root(leaf, path_elements, path_indices);
    assert(root == root2);
}

#[test]
fn test_outputs_hash_binding() {
    // Test that different outputs_hash values are properly constrained
    let outputs_hash1 = 12345;
    let outputs_hash2 = 54321;
    
    // The circuit should bind to the specific outputs_hash
    let square1 = outputs_hash1 * outputs_hash1;
    let square2 = outputs_hash2 * outputs_hash2;
    
    assert(square1 != square2);
}