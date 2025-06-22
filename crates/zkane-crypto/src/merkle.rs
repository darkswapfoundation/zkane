//! Merkle tree implementation for ZKane privacy pools

use anyhow::Result;
use zkane_common::{Commitment, MerklePath, ZKaneError, ZKaneResult};
use crate::hash::{hash_leaf, hash_internal};
use std::collections::HashMap;

/// A sparse Merkle tree for storing commitments
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// The height of the tree (number of levels)
    height: u32,
    /// The current number of leaves
    leaf_count: u32,
    /// Cache of computed hashes at each level
    /// Key: (level, index), Value: hash
    cache: HashMap<(u32, u32), [u8; 32]>,
    /// The zero hashes for each level (for sparse tree optimization)
    zero_hashes: Vec<[u8; 32]>,
}

impl MerkleTree {
    /// Create a new merkle tree with the given height
    pub fn new(height: u32) -> Self {
        let zero_hashes = Self::compute_zero_hashes(height);
        
        Self {
            height,
            leaf_count: 0,
            cache: HashMap::new(),
            zero_hashes,
        }
    }

    /// Compute the zero hashes for each level of the tree
    fn compute_zero_hashes(height: u32) -> Vec<[u8; 32]> {
        let mut zero_hashes = Vec::with_capacity(height as usize + 1);
        
        // Level 0 (leaves): hash of zero
        let zero_leaf = [0u8; 32];
        zero_hashes.push(hash_leaf(&zero_leaf));
        
        // Higher levels: hash of two zero hashes from previous level
        for i in 1..=height {
            let prev_zero = zero_hashes[(i - 1) as usize];
            let current_zero = hash_internal(&prev_zero, &prev_zero);
            zero_hashes.push(current_zero);
        }
        
        zero_hashes
    }

    /// Insert a commitment into the tree and return its leaf index
    pub fn insert(&mut self, commitment: &Commitment) -> ZKaneResult<u32> {
        if self.leaf_count >= (1u32 << self.height) {
            return Err(ZKaneError::TreeFull);
        }

        let leaf_index = self.leaf_count;
        let leaf_hash = hash_leaf(commitment.as_bytes());
        
        // Store the leaf
        self.cache.insert((0, leaf_index), leaf_hash);
        
        // Update the tree by recomputing hashes up to the root
        self.update_path(leaf_index, leaf_hash);
        
        self.leaf_count += 1;
        Ok(leaf_index)
    }

    /// Update the tree along the path from a leaf to the root
    fn update_path(&mut self, leaf_index: u32, leaf_hash: [u8; 32]) {
        let mut current_hash = leaf_hash;
        let mut current_index = leaf_index;
        
        for level in 1..=self.height {
            let parent_index = current_index / 2;
            let is_right_child = current_index % 2 == 1;
            
            let sibling_hash = if is_right_child {
                // We are the right child, get left sibling
                let sibling_index = current_index - 1;
                self.get_hash(level - 1, sibling_index)
            } else {
                // We are the left child, get right sibling
                let sibling_index = current_index + 1;
                self.get_hash(level - 1, sibling_index)
            };
            
            let parent_hash = if is_right_child {
                hash_internal(&sibling_hash, &current_hash)
            } else {
                hash_internal(&current_hash, &sibling_hash)
            };
            
            self.cache.insert((level, parent_index), parent_hash);
            
            current_hash = parent_hash;
            current_index = parent_index;
        }
    }

    /// Get the hash at a specific level and index
    fn get_hash(&self, level: u32, index: u32) -> [u8; 32] {
        if let Some(&hash) = self.cache.get(&(level, index)) {
            hash
        } else {
            // Return the appropriate zero hash for this level
            self.zero_hashes[level as usize]
        }
    }

    /// Get the current root hash of the tree
    pub fn root(&self) -> [u8; 32] {
        if self.leaf_count == 0 {
            return self.zero_hashes[self.height as usize];
        }
        
        self.get_hash(self.height, 0)
    }

    /// Generate a merkle path for the given leaf index
    pub fn generate_path(&self, leaf_index: u32) -> ZKaneResult<MerklePath> {
        if leaf_index >= self.leaf_count {
            return Err(ZKaneError::InvalidCommitment("Leaf index out of bounds".to_string()));
        }

        let mut elements = Vec::new();
        let mut indices = Vec::new();
        let mut current_index = leaf_index;
        
        for level in 0..self.height {
            let is_right_child = current_index % 2 == 1;
            let sibling_index = if is_right_child {
                current_index - 1
            } else {
                current_index + 1
            };
            
            let sibling_hash = self.get_hash(level, sibling_index);
            elements.push(sibling_hash);
            indices.push(is_right_child);
            
            current_index /= 2;
        }
        
        MerklePath::new(elements, indices)
    }

    /// Verify a merkle path for the given commitment and leaf index
    pub fn verify_path(
        &self,
        commitment: &Commitment,
        leaf_index: u32,
        path: &MerklePath,
        expected_root: &[u8; 32],
    ) -> ZKaneResult<bool> {
        if path.len() != self.height as usize {
            return Ok(false);
        }

        let mut current_hash = hash_leaf(commitment.as_bytes());
        let mut current_index = leaf_index;
        
        for (level, (&sibling_hash, &is_right_child)) in 
            path.elements.iter().zip(path.indices.iter()).enumerate() {
            
            // Verify the index matches the path
            if (current_index % 2 == 1) != is_right_child {
                return Ok(false);
            }
            
            current_hash = if is_right_child {
                hash_internal(&sibling_hash, &current_hash)
            } else {
                hash_internal(&current_hash, &sibling_hash)
            };
            
            current_index /= 2;
        }
        
        Ok(&current_hash == expected_root)
    }

    /// Get the current number of leaves in the tree
    pub fn leaf_count(&self) -> u32 {
        self.leaf_count
    }

    /// Get the height of the tree
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Check if the tree is full
    pub fn is_full(&self) -> bool {
        self.leaf_count >= (1u32 << self.height)
    }
}

/// Verify a merkle path without needing the full tree
pub fn verify_merkle_path(
    commitment: &Commitment,
    leaf_index: u32,
    path: &MerklePath,
    root: &[u8; 32],
    tree_height: u32,
) -> ZKaneResult<bool> {
    if path.len() != tree_height as usize {
        return Ok(false);
    }

    let mut current_hash = hash_leaf(commitment.as_bytes());
    let mut current_index = leaf_index;
    
    for (&sibling_hash, &is_right_child) in 
        path.elements.iter().zip(path.indices.iter()) {
        
        // Verify the index matches the path
        if (current_index % 2 == 1) != is_right_child {
            return Ok(false);
        }
        
        current_hash = if is_right_child {
            hash_internal(&sibling_hash, &current_hash)
        } else {
            hash_internal(&current_hash, &sibling_hash)
        };
        
        current_index /= 2;
    }
    
    Ok(&current_hash == root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkane_common::Commitment;

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new(4);
        assert_eq!(tree.leaf_count(), 0);
        assert_eq!(tree.height(), 4);
        assert!(!tree.is_full());
        
        // Root should be the zero hash for level 4
        let root = tree.root();
        assert_eq!(root, tree.zero_hashes[4]);
    }

    #[test]
    fn test_single_insertion() {
        let mut tree = MerkleTree::new(4);
        let commitment = Commitment::new([1u8; 32]);
        
        let leaf_index = tree.insert(&commitment).unwrap();
        assert_eq!(leaf_index, 0);
        assert_eq!(tree.leaf_count(), 1);
        
        // Root should change after insertion
        let root = tree.root();
        assert_ne!(root, tree.zero_hashes[4]);
    }

    #[test]
    fn test_multiple_insertions() {
        let mut tree = MerkleTree::new(4);
        
        for i in 0..5 {
            let commitment = Commitment::new([i as u8; 32]);
            let leaf_index = tree.insert(&commitment).unwrap();
            assert_eq!(leaf_index, i);
        }
        
        assert_eq!(tree.leaf_count(), 5);
    }

    #[test]
    fn test_tree_full() {
        let mut tree = MerkleTree::new(2); // Can hold 4 leaves
        
        // Insert 4 commitments
        for i in 0..4 {
            let commitment = Commitment::new([i as u8; 32]);
            tree.insert(&commitment).unwrap();
        }
        
        assert!(tree.is_full());
        
        // Try to insert one more - should fail
        let commitment = Commitment::new([5u8; 32]);
        assert!(tree.insert(&commitment).is_err());
    }

    #[test]
    fn test_merkle_path_generation() {
        let mut tree = MerkleTree::new(4);
        let commitment = Commitment::new([1u8; 32]);
        
        let leaf_index = tree.insert(&commitment).unwrap();
        let path = tree.generate_path(leaf_index).unwrap();
        
        assert_eq!(path.len(), 4);
        
        // Verify the path
        let root = tree.root();
        assert!(tree.verify_path(&commitment, leaf_index, &path, &root).unwrap());
    }

    #[test]
    fn test_merkle_path_verification() {
        let mut tree = MerkleTree::new(3);
        
        // Insert multiple commitments
        let commitments: Vec<_> = (0..5).map(|i| Commitment::new([i as u8; 32])).collect();
        let mut leaf_indices = Vec::new();
        
        for commitment in &commitments {
            let leaf_index = tree.insert(commitment).unwrap();
            leaf_indices.push(leaf_index);
        }
        
        let root = tree.root();
        
        // Verify all paths
        for (i, commitment) in commitments.iter().enumerate() {
            let path = tree.generate_path(leaf_indices[i]).unwrap();
            assert!(tree.verify_path(commitment, leaf_indices[i], &path, &root).unwrap());
            
            // Also test standalone verification
            assert!(verify_merkle_path(commitment, leaf_indices[i], &path, &root, 3).unwrap());
        }
    }

    #[test]
    fn test_invalid_path_verification() {
        let mut tree = MerkleTree::new(3);
        let commitment = Commitment::new([1u8; 32]);
        
        let leaf_index = tree.insert(&commitment).unwrap();
        let mut path = tree.generate_path(leaf_index).unwrap();
        let root = tree.root();
        
        // Modify the path to make it invalid
        path.elements[0][0] ^= 1;
        
        assert!(!tree.verify_path(&commitment, leaf_index, &path, &root).unwrap());
    }
}