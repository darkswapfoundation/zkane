//! # ZKane Common Types and Structures
//!
//! This crate provides the core types and data structures used throughout
//! the ZKane privacy pool system. It defines the fundamental cryptographic
//! primitives and data structures that enable privacy-preserving transactions.
//!
//! ## Overview
//!
//! ZKane is a privacy pool system for alkanes assets that uses zero-knowledge proofs
//! to break the link between deposits and withdrawals. This crate contains the
//! fundamental types that represent the cryptographic primitives used in the system.
//!
//! ## Core Types
//!
//! - [`Secret`] - 32-byte secret values for commitment generation
//! - [`Nullifier`] - 32-byte nullifier values for double-spending prevention
//! - [`Commitment`] - 32-byte commitment values for privacy preservation
//! - [`NullifierHash`] - 32-byte hashed nullifiers for public verification
//! - [`DepositNote`] - Complete deposit information for users
//! - [`WithdrawalProof`] - Zero-knowledge proof data for withdrawals
//! - [`ZKaneConfig`] - Configuration for privacy pools
//! - [`MerklePath`] - Merkle tree inclusion proofs
//!
//! ## Privacy Model
//!
//! The privacy model is based on commitments that hide the relationship between
//! deposits and withdrawals:
//!
//! 1. **Deposit**: User generates a secret and nullifier, computes a commitment,
//!    and deposits assets along with the commitment to the privacy pool.
//!
//! 2. **Withdrawal**: User generates a zero-knowledge proof that they know a
//!    secret and nullifier for a commitment in the pool, without revealing
//!    which specific commitment.
//!
//! ## Example Usage
//!
//! ```rust
//! use zkane_common::{Secret, Nullifier, DepositNote};
//! use alkanes_support::id::AlkaneId;
//!
//! // Generate cryptographic values for a deposit
//! let secret = Secret::random();
//! let nullifier = Nullifier::random();
//!
//! // Create a deposit note (commitment would be calculated separately)
//! let asset_id = AlkaneId { block: 2, tx: 1 };
//! let deposit_note = DepositNote::random(asset_id, 1000000);
//!
//! // Access the underlying bytes for cryptographic operations
//! let secret_bytes = secret.as_bytes();
//! let nullifier_bytes = nullifier.as_bytes();
//! ```
//!
//! ## Security Considerations
//!
//! - **Secret Management**: Secrets must be kept private and never transmitted in plaintext
//! - **Nullifier Uniqueness**: Each nullifier can only be used once to prevent double-spending
//! - **Commitment Binding**: Commitments cryptographically bind secrets and nullifiers
//! - **Random Generation**: All cryptographic values should use secure randomness

use anyhow::Result;
use serde::{Deserialize, Serialize};
use alkanes_support::id::AlkaneId;

/// A commitment to a secret value in the privacy pool.
///
/// Commitments are cryptographic bindings of secrets and nullifiers that hide
/// the actual values while allowing zero-knowledge proofs of knowledge.
/// They are stored publicly in the privacy pool's Merkle tree.
///
/// # Security Properties
///
/// - **Hiding**: The commitment reveals no information about the secret or nullifier
/// - **Binding**: It's computationally infeasible to find different secret/nullifier
///   pairs that produce the same commitment
/// - **Deterministic**: Same secret/nullifier pair always produces same commitment
///
/// # Example
///
/// ```rust
/// use zkane_common::Commitment;
///
/// // Create from bytes (typically from cryptographic hash function)
/// let bytes = [0u8; 32];
/// let commitment = Commitment::new(bytes);
///
/// // Convert to hex for display/storage
/// let hex_string = commitment.to_hex();
///
/// // Parse from hex
/// let parsed = Commitment::from_hex(&hex_string).unwrap();
/// assert_eq!(commitment, parsed);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Commitment(pub [u8; 32]);

impl Commitment {
    /// Create a new commitment from 32 bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The 32-byte commitment value
    ///
    /// # Example
    ///
    /// ```rust
    /// use zkane_common::Commitment;
    ///
    /// let bytes = [42u8; 32];
    /// let commitment = Commitment::new(bytes);
    /// ```
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the commitment as a byte array reference.
    ///
    /// # Returns
    ///
    /// A reference to the underlying 32-byte array
    ///
    /// # Example
    ///
    /// ```rust
    /// use zkane_common::Commitment;
    ///
    /// let commitment = Commitment::new([1u8; 32]);
    /// let bytes = commitment.as_bytes();
    /// assert_eq!(bytes.len(), 32);
    /// ```
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert the commitment to a hexadecimal string.
    ///
    /// # Returns
    ///
    /// A 64-character hexadecimal string representation
    ///
    /// # Example
    ///
    /// ```rust
    /// use zkane_common::Commitment;
    ///
    /// let commitment = Commitment::new([255u8; 32]);
    /// let hex = commitment.to_hex();
    /// assert_eq!(hex.len(), 64);
    /// assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    /// ```
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse a commitment from a hexadecimal string.
    ///
    /// # Arguments
    ///
    /// * `hex_str` - A 64-character hexadecimal string
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed commitment or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The string is not valid hexadecimal
    /// - The string does not represent exactly 32 bytes
    ///
    /// # Example
    ///
    /// ```rust
    /// use zkane_common::Commitment;
    ///
    /// let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    /// let commitment = Commitment::from_hex(hex).unwrap();
    /// assert_eq!(commitment.to_hex(), hex);
    /// ```
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid commitment length: expected 32 bytes, got {}", bytes.len()));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
}

/// A nullifier hash to prevent double spending.
///
/// Nullifier hashes are derived from nullifiers and are published during
/// withdrawals to prevent the same deposit from being withdrawn multiple times.
/// The hash prevents linking the withdrawal to the original nullifier while
/// still enabling double-spend detection.
///
/// # Security Properties
///
/// - **One-way**: Computationally infeasible to derive nullifier from hash
/// - **Deterministic**: Same nullifier always produces same hash
/// - **Collision-resistant**: Different nullifiers produce different hashes
///
/// # Example
///
/// ```rust
/// use zkane_common::NullifierHash;
///
/// // Create from hash output
/// let hash_bytes = [0u8; 32];
/// let nullifier_hash = NullifierHash::new(hash_bytes);
///
/// // Convert to hex for storage/transmission
/// let hex = nullifier_hash.to_hex();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NullifierHash(pub [u8; 32]);

impl NullifierHash {
    /// Create a new nullifier hash from 32 bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The 32-byte hash value
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the nullifier hash as a byte array reference.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert the nullifier hash to a hexadecimal string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse a nullifier hash from a hexadecimal string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not valid hex or not 32 bytes.
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid nullifier hash length: expected 32 bytes, got {}", bytes.len()));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
}

/// A secret value used to generate commitments.
///
/// Secrets are randomly generated 32-byte values that, combined with nullifiers,
/// create unique commitments for privacy pool deposits. The secret must be kept
/// private by the user to enable future withdrawals.
///
/// # Security Requirements
///
/// - **Randomness**: Must be generated using cryptographically secure randomness
/// - **Confidentiality**: Must never be shared or transmitted in plaintext
/// - **Persistence**: Must be stored securely for future withdrawal
///
/// # Example
///
/// ```rust
/// use zkane_common::Secret;
///
/// // Generate a cryptographically secure random secret
/// let secret = Secret::random();
///
/// // Create from existing bytes (e.g., derived from seed)
/// let bytes = [42u8; 32];
/// let secret = Secret::new(bytes);
///
/// // Access bytes for cryptographic operations
/// let secret_bytes = secret.as_bytes();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Secret(pub [u8; 32]);

impl Secret {
    /// Create a new secret from 32 bytes.
    ///
    /// # Security Warning
    ///
    /// Ensure the bytes come from a cryptographically secure source.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Generate a cryptographically secure random secret.
    ///
    /// Uses the system's secure random number generator to create
    /// a 32-byte secret suitable for cryptographic use.
    ///
    /// # Example
    ///
    /// ```rust
    /// use zkane_common::Secret;
    ///
    /// let secret1 = Secret::random();
    /// let secret2 = Secret::random();
    /// // Secrets should be different (with overwhelming probability)
    /// assert_ne!(secret1, secret2);
    /// ```
    pub fn random() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Get the secret as a byte array reference.
    ///
    /// # Security Warning
    ///
    /// Handle the returned bytes carefully to avoid leaking the secret.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert the secret to a hexadecimal string.
    ///
    /// # Security Warning
    ///
    /// The returned string contains the secret in plaintext.
    /// Only use for secure storage or transmission over encrypted channels.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse a secret from a hexadecimal string.
    ///
    /// # Security Warning
    ///
    /// Ensure the hex string comes from a trusted source and is transmitted securely.
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid secret length: expected 32 bytes, got {}", bytes.len()));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
}

/// A nullifier value used to generate nullifier hashes.
///
/// Nullifiers are randomly generated values that are combined with secrets
/// to create commitments. During withdrawal, the nullifier is revealed
/// (as a hash) to prevent double-spending while maintaining privacy.
///
/// # Security Properties
///
/// - **Uniqueness**: Each nullifier should be used only once
/// - **Randomness**: Must be generated using secure randomness
/// - **Binding**: Cryptographically bound to the secret in the commitment
///
/// # Example
///
/// ```rust
/// use zkane_common::Nullifier;
///
/// // Generate a random nullifier
/// let nullifier = Nullifier::random();
///
/// // Create from specific bytes
/// let bytes = [1u8; 32];
/// let nullifier = Nullifier::new(bytes);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nullifier(pub [u8; 32]);

impl Nullifier {
    /// Create a new nullifier from 32 bytes.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Generate a cryptographically secure random nullifier.
    pub fn random() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Get the nullifier as a byte array reference.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert the nullifier to a hexadecimal string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse a nullifier from a hexadecimal string.
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid nullifier length: expected 32 bytes, got {}", bytes.len()));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
}

/// Configuration for a ZKane privacy pool.
///
/// This structure contains all the parameters needed to configure and operate
/// a privacy pool for a specific asset and denomination combination.
///
/// # Example
///
/// ```rust
/// use zkane_common::ZKaneConfig;
/// use alkanes_support::id::AlkaneId;
///
/// let config = ZKaneConfig::new(
///     AlkaneId { block: 2, tx: 1 },  // Asset ID
///     1000000,                        // 1M unit denomination
///     20,                            // 20-level Merkle tree (1M max deposits)
///     vec![0u8; 32],                 // Verifier key (placeholder)
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKaneConfig {
    /// The alkane asset ID this pool accepts
    pub asset_id: AlkaneId,
    /// The denomination (fixed amount) for deposits/withdrawals
    pub denomination: u128,
    /// The merkle tree height (determines max number of deposits)
    pub tree_height: u32,
    /// The verifier key for proof verification
    pub verifier_key: Vec<u8>,
}

impl ZKaneConfig {
    /// Create a new ZKane configuration.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The alkanes asset this pool will accept
    /// * `denomination` - Fixed amount for all deposits/withdrawals
    /// * `tree_height` - Merkle tree height (max deposits = 2^height)
    /// * `verifier_key` - Cryptographic key for proof verification
    pub fn new(
        asset_id: AlkaneId,
        denomination: u128,
        tree_height: u32,
        verifier_key: Vec<u8>,
    ) -> Self {
        Self {
            asset_id,
            denomination,
            tree_height,
            verifier_key,
        }
    }

    /// Get the maximum number of deposits this pool can handle.
    ///
    /// # Returns
    ///
    /// The maximum number of deposits (2^tree_height)
    pub fn max_deposits(&self) -> u64 {
        1u64 << self.tree_height
    }
}

/// A deposit note containing the secret information needed for withdrawal.
///
/// This structure contains all the information a user needs to store
/// in order to withdraw their deposit from the privacy pool at a later time.
/// It should be kept secure and private.
///
/// # Security Warning
///
/// This structure contains sensitive cryptographic material. It should be:
/// - Stored securely (encrypted at rest)
/// - Transmitted only over secure channels
/// - Never logged or exposed in plaintext
///
/// # Example
///
/// ```rust
/// use zkane_common::{DepositNote, Secret, Nullifier, Commitment};
/// use alkanes_support::id::AlkaneId;
///
/// // Create a deposit note with specific values
/// let secret = Secret::random();
/// let nullifier = Nullifier::random();
/// let commitment = Commitment::new([0u8; 32]); // Would be calculated
/// let asset_id = AlkaneId { block: 2, tx: 1 };
///
/// let note = DepositNote::new(
///     secret,
///     nullifier,
///     commitment,
///     asset_id,
///     1000000,  // denomination
///     0,        // leaf index (set during deposit)
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositNote {
    /// The secret value (keep private!)
    pub secret: Secret,
    /// The nullifier value (keep private!)
    pub nullifier: Nullifier,
    /// The commitment (public, stored in pool)
    pub commitment: Commitment,
    /// The asset ID for this deposit
    pub asset_id: AlkaneId,
    /// The denomination of this deposit
    pub denomination: u128,
    /// The leaf index in the merkle tree (set during deposit)
    pub leaf_index: u32,
}

impl DepositNote {
    /// Create a new deposit note with all parameters.
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret value (keep private)
    /// * `nullifier` - The nullifier value (keep private)
    /// * `commitment` - The commitment (public)
    /// * `asset_id` - The asset being deposited
    /// * `denomination` - The amount being deposited
    /// * `leaf_index` - Position in the Merkle tree
    pub fn new(
        secret: Secret,
        nullifier: Nullifier,
        commitment: Commitment,
        asset_id: AlkaneId,
        denomination: u128,
        leaf_index: u32,
    ) -> Self {
        Self {
            secret,
            nullifier,
            commitment,
            asset_id,
            denomination,
            leaf_index,
        }
    }

    /// Generate a random deposit note for testing purposes.
    ///
    /// # Warning
    ///
    /// This method creates a placeholder commitment. In production,
    /// the commitment should be calculated using the proper hash function
    /// with the secret and nullifier.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The asset for this deposit
    /// * `denomination` - The amount for this deposit
    pub fn random(asset_id: AlkaneId, denomination: u128) -> Self {
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        // Note: commitment should be calculated using proper hash function
        let commitment = Commitment::new([0u8; 32]); // Placeholder
        
        Self {
            secret,
            nullifier,
            commitment,
            asset_id,
            denomination,
            leaf_index: 0, // Will be set when deposited
        }
    }
}

/// Merkle tree path for proving inclusion.
///
/// This structure represents a path from a leaf to the root of a Merkle tree,
/// containing all the sibling hashes needed to verify that a specific
/// commitment is included in the tree.
///
/// # Example
///
/// ```rust
/// use zkane_common::MerklePath;
///
/// // Create a path with sibling hashes and directions
/// let elements = vec![[1u8; 32], [2u8; 32], [3u8; 32]];
/// let indices = vec![false, true, false]; // left, right, left
/// let path = MerklePath::new(elements, indices).unwrap();
///
/// assert_eq!(path.len(), 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerklePath {
    /// The path elements (sibling hashes at each level)
    pub elements: Vec<[u8; 32]>,
    /// The path indices (false = left, true = right)
    pub indices: Vec<bool>,
}

impl MerklePath {
    /// Create a new merkle path.
    ///
    /// # Arguments
    ///
    /// * `elements` - Sibling hashes at each level of the tree
    /// * `indices` - Direction at each level (false = left, true = right)
    ///
    /// # Errors
    ///
    /// Returns an error if elements and indices have different lengths.
    pub fn new(elements: Vec<[u8; 32]>, indices: Vec<bool>) -> Result<Self> {
        if elements.len() != indices.len() {
            return Err(anyhow::anyhow!(
                "Path elements and indices must have same length: {} vs {}",
                elements.len(),
                indices.len()
            ));
        }
        Ok(Self { elements, indices })
    }

    /// Get the path length (number of levels).
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get the tree height this path represents.
    ///
    /// This is the same as the path length.
    pub fn tree_height(&self) -> u32 {
        self.elements.len() as u32
    }
}

/// Zero-knowledge proof for withdrawal.
///
/// This structure contains all the data needed to verify a withdrawal
/// from the privacy pool, including the cryptographic proof and
/// associated public inputs.
///
/// # Example
///
/// ```rust
/// use zkane_common::{WithdrawalProof, NullifierHash};
///
/// let proof = WithdrawalProof::new(
///     vec![0u8; 256],                    // Proof bytes
///     [1u8; 32],                         // Merkle root
///     NullifierHash::new([2u8; 32]),     // Nullifier hash
///     12345,                             // Recipient
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalProof {
    /// The zero-knowledge proof bytes
    pub proof: Vec<u8>,
    /// The merkle root at time of proof generation
    pub merkle_root: [u8; 32],
    /// The nullifier hash being revealed
    pub nullifier_hash: NullifierHash,
    /// The recipient address (as u128 for alkanes compatibility)
    pub recipient: u128,
}

impl WithdrawalProof {
    /// Create a new withdrawal proof.
    ///
    /// # Arguments
    ///
    /// * `proof` - The zero-knowledge proof bytes
    /// * `merkle_root` - The Merkle root when proof was generated
    /// * `nullifier_hash` - The nullifier hash being spent
    /// * `recipient` - The recipient address
    pub fn new(
        proof: Vec<u8>,
        merkle_root: [u8; 32],
        nullifier_hash: NullifierHash,
        recipient: u128,
    ) -> Self {
        Self {
            proof,
            merkle_root,
            nullifier_hash,
            recipient,
        }
    }

    /// Get the size of the proof in bytes.
    pub fn proof_size(&self) -> usize {
        self.proof.len()
    }
}

/// Error types for ZKane operations.
///
/// This enum represents all the possible errors that can occur
/// during ZKane privacy pool operations.
#[derive(Debug, thiserror::Error)]
pub enum ZKaneError {
    /// Invalid commitment format or value
    #[error("Invalid commitment: {0}")]
    InvalidCommitment(String),
    
    /// Invalid nullifier format or value
    #[error("Invalid nullifier: {0}")]
    InvalidNullifier(String),
    
    /// Invalid zero-knowledge proof
    #[error("Invalid proof: {0}")]
    InvalidProof(String),
    
    /// Attempt to spend an already spent nullifier
    #[error("Nullifier already spent")]
    NullifierAlreadySpent,
    
    /// Merkle root doesn't match expected value
    #[error("Invalid merkle root")]
    InvalidMerkleRoot,
    
    /// Denomination doesn't match pool requirements
    #[error("Invalid denomination")]
    InvalidDenomination,
    
    /// Merkle tree has reached maximum capacity
    #[error("Tree is full")]
    TreeFull,
    
    /// General cryptographic operation error
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
}

/// Result type for ZKane operations.
///
/// This is a convenience type alias for `Result<T, ZKaneError>`.
pub type ZKaneResult<T> = std::result::Result<T, ZKaneError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_hex_roundtrip() {
        let original = Commitment::new([1u8; 32]);
        let hex = original.to_hex();
        let parsed = Commitment::from_hex(&hex).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_secret_random() {
        let secret1 = Secret::random();
        let secret2 = Secret::random();
        assert_ne!(secret1, secret2);
    }

    #[test]
    fn test_nullifier_random() {
        let nullifier1 = Nullifier::random();
        let nullifier2 = Nullifier::random();
        assert_ne!(nullifier1, nullifier2);
    }

    #[test]
    fn test_merkle_path_validation() {
        let elements = vec![[1u8; 32], [2u8; 32]];
        let indices = vec![true, false];
        let path = MerklePath::new(elements, indices).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path.tree_height(), 2);

        // Test mismatched lengths
        let elements = vec![[1u8; 32]];
        let indices = vec![true, false];
        assert!(MerklePath::new(elements, indices).is_err());
    }

    #[test]
    fn test_zkane_config_max_deposits() {
        let config = ZKaneConfig::new(
            AlkaneId { block: 1, tx: 1 },
            1000,
            10,
            vec![],
        );
        assert_eq!(config.max_deposits(), 1024); // 2^10
    }

    #[test]
    fn test_deposit_note_creation() {
        let secret = Secret::random();
        let nullifier = Nullifier::random();
        let commitment = Commitment::new([42u8; 32]);
        let asset_id = AlkaneId { block: 2, tx: 1 };

        let note = DepositNote::new(
            secret,
            nullifier,
            commitment,
            asset_id,
            1000000,
            5,
        );

        assert_eq!(note.asset_id, asset_id);
        assert_eq!(note.denomination, 1000000);
        assert_eq!(note.leaf_index, 5);
    }

    #[test]
    fn test_withdrawal_proof_creation() {
        let proof_bytes = vec![1, 2, 3, 4];
        let merkle_root = [42u8; 32];
        let nullifier_hash = NullifierHash::new([1u8; 32]);
        let recipient = 12345u128;

        let proof = WithdrawalProof::new(
            proof_bytes.clone(),
            merkle_root,
            nullifier_hash,
            recipient,
        );

        assert_eq!(proof.proof, proof_bytes);
        assert_eq!(proof.merkle_root, merkle_root);
        assert_eq!(proof.nullifier_hash, nullifier_hash);
        assert_eq!(proof.recipient, recipient);
        assert_eq!(proof.proof_size(), 4);
    }
}