//! Core privacy pool logic for ZKane

use anyhow::Result;
use zkane_common::{
    Commitment, DepositNote, NullifierHash, MerklePath, Secret, Nullifier, 
    WithdrawalProof, ZKaneConfig, ZKaneError, ZKaneResult
};
use zkane_crypto::{
    generate_commitment, generate_nullifier_hash, MerkleTree, verify_merkle_path
};
use alkanes_support::id::AlkaneId;
use std::collections::HashSet;

pub mod pool;
pub mod prover;

pub use pool::*;
pub use prover::*;

/// A privacy pool for a specific alkanes asset
#[derive(Debug, Clone)]
pub struct PrivacyPool {
    /// Configuration for this pool
    config: ZKaneConfig,
    /// Merkle tree of commitments
    merkle_tree: MerkleTree,
    /// Set of spent nullifier hashes
    spent_nullifiers: HashSet<NullifierHash>,
    /// Current merkle root
    current_root: [u8; 32],
}

impl PrivacyPool {
    /// Create a new privacy pool with the given configuration
    pub fn new(config: ZKaneConfig) -> Self {
        let merkle_tree = MerkleTree::new(config.tree_height);
        let current_root = merkle_tree.root();
        
        Self {
            config,
            merkle_tree,
            spent_nullifiers: HashSet::new(),
            current_root,
        }
    }

    /// Get the pool configuration
    pub fn config(&self) -> &ZKaneConfig {
        &self.config
    }

    /// Get the current merkle root
    pub fn root(&self) -> [u8; 32] {
        self.current_root
    }

    /// Get the number of deposits in the pool
    pub fn deposit_count(&self) -> u32 {
        self.merkle_tree.leaf_count()
    }

    /// Check if the pool is full
    pub fn is_full(&self) -> bool {
        self.merkle_tree.is_full()
    }

    /// Check if a nullifier hash has been spent
    pub fn is_nullifier_spent(&self, nullifier_hash: &NullifierHash) -> bool {
        self.spent_nullifiers.contains(nullifier_hash)
    }

    /// Process a deposit and return a deposit note
    pub fn deposit(&mut self, secret: Secret, nullifier: Nullifier) -> ZKaneResult<DepositNote> {
        // Generate commitment
        let commitment = generate_commitment(&nullifier, &secret)?;
        
        // Insert into merkle tree
        let leaf_index = self.merkle_tree.insert(&commitment)?;
        
        // Update current root
        self.current_root = self.merkle_tree.root();
        
        // Create deposit note
        let deposit_note = DepositNote::new(
            secret,
            nullifier,
            commitment,
            self.config.asset_id.clone(),
            self.config.denomination,
            leaf_index,
        );
        
        Ok(deposit_note)
    }

    /// Generate a merkle path for a given leaf index
    pub fn generate_merkle_path(&self, leaf_index: u32) -> ZKaneResult<MerklePath> {
        self.merkle_tree.generate_path(leaf_index)
            .map_err(|e| ZKaneError::CryptoError(format!("Failed to generate merkle path: {}", e)))
    }

    /// Verify a withdrawal proof
    pub fn verify_withdrawal(
        &self,
        proof: &WithdrawalProof,
        commitment: &Commitment,
        leaf_index: u32,
        merkle_path: &MerklePath,
    ) -> ZKaneResult<bool> {
        // Check if nullifier is already spent
        if self.is_nullifier_spent(&proof.nullifier_hash) {
            return Err(ZKaneError::NullifierAlreadySpent);
        }

        // Verify merkle path
        let path_valid = verify_merkle_path(
            commitment,
            leaf_index,
            merkle_path,
            &proof.merkle_root,
            self.config.tree_height,
        ).map_err(|e| ZKaneError::CryptoError(format!("Merkle path verification failed: {}", e)))?;

        if !path_valid {
            return Ok(false);
        }

        // Check if the merkle root is valid (should be a recent root)
        if proof.merkle_root != self.current_root {
            return Err(ZKaneError::InvalidMerkleRoot);
        }

        // TODO: Verify the zero-knowledge proof
        // This would involve calling the Noir verifier with the proof
        // For now, we'll assume the proof is valid if we reach this point
        
        Ok(true)
    }

    /// Process a withdrawal (mark nullifier as spent)
    pub fn withdraw(&mut self, nullifier_hash: NullifierHash) -> ZKaneResult<()> {
        if self.is_nullifier_spent(&nullifier_hash) {
            return Err(ZKaneError::NullifierAlreadySpent);
        }

        self.spent_nullifiers.insert(nullifier_hash);
        Ok(())
    }

    /// Get all spent nullifier hashes
    pub fn spent_nullifiers(&self) -> &HashSet<NullifierHash> {
        &self.spent_nullifiers
    }

    /// Check if a merkle root is valid (current or recent)
    pub fn is_valid_root(&self, root: &[u8; 32]) -> bool {
        // For now, only accept the current root
        // In a production system, you might want to accept recent roots
        // to handle race conditions
        root == &self.current_root
    }
}

/// Generate a random deposit note for the given asset and denomination
pub fn generate_deposit_note(asset_id: AlkaneId, denomination: u128) -> ZKaneResult<DepositNote> {
    let secret = Secret::random();
    let nullifier = Nullifier::random();
    let commitment = generate_commitment(&nullifier, &secret)?;
    
    Ok(DepositNote::new(
        secret,
        nullifier,
        commitment,
        asset_id,
        denomination,
        0, // Leaf index will be set when deposited
    ))
}

/// Verify that a deposit note is valid (commitment matches secret and nullifier)
pub fn verify_deposit_note(note: &DepositNote) -> ZKaneResult<bool> {
    let expected_commitment = generate_commitment(&note.nullifier, &note.secret)?;
    Ok(note.commitment == expected_commitment)
}

/// Generate nullifier hash from a deposit note
pub fn get_nullifier_hash(note: &DepositNote) -> ZKaneResult<NullifierHash> {
    generate_nullifier_hash(&note.nullifier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkane_common::{Secret, Nullifier};

    fn create_test_config() -> ZKaneConfig {
        ZKaneConfig::new(
            AlkaneId { block: 1, tx: 1 },
            1000000, // 1M denomination
            20,      // Tree height
            vec![],  // Empty verifier key for testing
        )
    }

    #[test]
    fn test_privacy_pool_creation() {
        let config = create_test_config();
        let pool = PrivacyPool::new(config.clone());
        
        assert_eq!(pool.config().denomination, 1000000);
        assert_eq!(pool.config().tree_height, 20);
        assert_eq!(pool.deposit_count(), 0);
        assert!(!pool.is_full());
    }

    #[test]
    fn test_deposit() {
        let config = create_test_config();
        let mut pool = PrivacyPool::new(config);
        
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        
        let deposit_note = pool.deposit(secret, nullifier).unwrap();
        
        assert_eq!(deposit_note.secret, secret);
        assert_eq!(deposit_note.nullifier, nullifier);
        assert_eq!(deposit_note.leaf_index, 0);
        assert_eq!(pool.deposit_count(), 1);
    }

    #[test]
    fn test_multiple_deposits() {
        let config = create_test_config();
        let mut pool = PrivacyPool::new(config);
        
        for i in 0..5 {
            let secret = Secret::random();
            let nullifier = Nullifier::random();
            
            let deposit_note = pool.deposit(secret, nullifier).unwrap();
            assert_eq!(deposit_note.leaf_index, i);
        }
        
        assert_eq!(pool.deposit_count(), 5);
    }

    #[test]
    fn test_nullifier_spending() {
        let config = create_test_config();
        let mut pool = PrivacyPool::new(config);
        
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        let nullifier_hash = generate_nullifier_hash(&nullifier).unwrap();
        
        // Initially not spent
        assert!(!pool.is_nullifier_spent(&nullifier_hash));
        
        // Mark as spent
        pool.withdraw(nullifier_hash).unwrap();
        assert!(pool.is_nullifier_spent(&nullifier_hash));
        
        // Try to spend again - should fail
        assert!(pool.withdraw(nullifier_hash).is_err());
    }

    #[test]
    fn test_merkle_path_generation() {
        let config = create_test_config();
        let mut pool = PrivacyPool::new(config);
        
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        
        let deposit_note = pool.deposit(secret, nullifier).unwrap();
        let merkle_path = pool.generate_merkle_path(deposit_note.leaf_index).unwrap();
        
        assert_eq!(merkle_path.len(), 20); // Tree height
    }

    #[test]
    fn test_generate_deposit_note() {
        let asset_id = AlkaneId { block: 1, tx: 1 };
        let denomination = 1000000;
        
        let note = generate_deposit_note(asset_id.clone(), denomination).unwrap();
        
        assert_eq!(note.asset_id, asset_id);
        assert_eq!(note.denomination, denomination);
        assert!(verify_deposit_note(&note).unwrap());
    }

    #[test]
    fn test_nullifier_hash_generation() {
        let asset_id = AlkaneId { block: 1, tx: 1 };
        let note = generate_deposit_note(asset_id, 1000000).unwrap();
        
        let nullifier_hash = get_nullifier_hash(&note).unwrap();
        let expected_hash = generate_nullifier_hash(&note.nullifier).unwrap();
        
        assert_eq!(nullifier_hash, expected_hash);
    }
}