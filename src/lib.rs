//! ZKane Privacy Pool System
//! 
//! A privacy pool implementation for alkanes assets using zero-knowledge proofs.
//! This system allows users to deposit alkanes assets and withdraw them from different
//! addresses, breaking on-chain links between deposits and withdrawals.
//!
//! ## Features
//!
//! - **Privacy-preserving transactions**: Uses zero-knowledge proofs to hide transaction links
//! - **Multi-asset support**: Works with any alkanes asset
//! - **Factory pattern**: Automatic pool creation for asset/denomination pairs
//! - **Witness envelopes**: Efficient storage of large proof data
//! - **Transaction output validation**: Prevents frontrunning attacks
//! - **Browser compatibility**: WASM bindings for dapp integration
//!
//! ## Architecture
//!
//! The ZKane system consists of several components:
//!
//! - **ZKane Contract**: Core privacy pool contract implementing deposits and withdrawals
//! - **ZKane Factory**: Factory contract for creating and managing multiple pools
//! - **Noir Circuits**: Zero-knowledge proof circuits for withdrawal validation
//! - **WASM Bindings**: Browser-compatible API for dapp integration
//!
//! ## Usage
//!
//! ### Basic Deposit Flow
//!
//! ```rust,no_run
//! use zkane_common::{Secret, Nullifier};
//! use zkane_crypto::generate_commitment;
//! use zkane_core::generate_deposit_note;
//! use alkanes_support::id::AlkaneId;
//!
//! // Generate deposit note
//! let asset_id = AlkaneId { block: 2, tx: 1 };
//! let denomination = 1000000u128;
//! let deposit_note = generate_deposit_note(asset_id, denomination)?;
//!
//! // Use deposit_note.commitment for the deposit transaction
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Basic Withdrawal Flow
//!
//! ```rust,no_run
//! use zkane_crypto::generate_nullifier_hash;
//!
//! // Generate nullifier hash for withdrawal
//! let nullifier_hash = generate_nullifier_hash(&deposit_note.nullifier)?;
//!
//! // Generate ZK proof (using Noir circuit)
//! // Submit withdrawal transaction with proof
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Re-export core types and functions
pub use zkane_common::*;
pub use zkane_crypto::*;
pub use zkane_core::*;

// Re-export WASM bindings when feature is enabled
#[cfg(feature = "wasm")]
#[cfg(feature = "wasm")]
pub use zkane_wasm;

// Error types
pub use anyhow::{Error, Result};

/// ZKane system version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// ZKane system description
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Get ZKane system information
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        name: "ZKane Privacy Pool".to_string(),
        version: VERSION.to_string(),
        description: DESCRIPTION.to_string(),
        features: vec![
            "Privacy-preserving transactions".to_string(),
            "Multi-asset support".to_string(),
            "Zero-knowledge proofs".to_string(),
            "Factory pattern".to_string(),
            "Witness envelope support".to_string(),
            "Transaction output validation".to_string(),
            "Browser compatibility".to_string(),
        ],
    }
}

/// System information structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
}

/// ZKane configuration constants
pub mod config {
    use alkanes_support::id::AlkaneId;

    /// Block number for ZKane contract templates
    pub const ZKANE_TEMPLATE_BLOCK: u128 = 4;
    
    /// Block number for ZKane factory templates
    pub const ZKANE_FACTORY_TEMPLATE_BLOCK: u128 = 4;
    
    /// Block number for ZKane instances
    pub const ZKANE_INSTANCE_BLOCK: u128 = 6;
    
    /// Default Merkle tree depth
    pub const DEFAULT_MERKLE_DEPTH: usize = 20;
    
    /// Maximum number of commitments per pool
    pub const MAX_COMMITMENTS: u64 = 1_048_576; // 2^20
    
    /// Minimum denomination for privacy pools
    pub const MIN_DENOMINATION: u128 = 1000;
    
    /// ZKane contract template ID
    pub const ZKANE_TEMPLATE_ID: AlkaneId = AlkaneId {
        block: ZKANE_TEMPLATE_BLOCK,
        tx: 0x1000,
    };
    
    /// ZKane factory template ID
    pub const ZKANE_FACTORY_TEMPLATE_ID: AlkaneId = AlkaneId {
        block: ZKANE_FACTORY_TEMPLATE_BLOCK,
        tx: 0x2000,
    };
}

/// Utility functions for ZKane operations
pub mod utils {
    use super::*;
    use alkanes_support::id::AlkaneId;
    use sha2::{Digest, Sha256};

    /// Generate deterministic pool ID for asset/denomination pair
    pub fn generate_pool_id(asset_id: AlkaneId, denomination: u128) -> AlkaneId {
        let mut hasher_input = Vec::new();
        hasher_input.extend_from_slice(&asset_id.block.to_le_bytes());
        hasher_input.extend_from_slice(&asset_id.tx.to_le_bytes());
        hasher_input.extend_from_slice(&denomination.to_le_bytes());
        
        let mut hash_value = 0u128;
        for chunk in hasher_input.chunks(16) {
            let mut bytes = [0u8; 16];
            bytes[..chunk.len()].copy_from_slice(chunk);
            hash_value ^= u128::from_le_bytes(bytes);
        }
        
        AlkaneId {
            block: config::ZKANE_INSTANCE_BLOCK,
            tx: hash_value,
        }
    }

    /// Calculate transaction outputs hash for recipient validation
    pub fn calculate_outputs_hash(outputs: &[(u64, Vec<u8>)]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        for (value, script_pubkey) in outputs {
            hasher.update(&value.to_le_bytes());
            hasher.update(script_pubkey);
        }
        
        hasher.finalize().into()
    }

    /// Validate hex string format
    pub fn is_valid_hex(hex_str: &str, expected_length: usize) -> bool {
        if let Ok(bytes) = hex::decode(hex_str) {
            bytes.len() == expected_length
        } else {
            false
        }
    }

    /// Convert bytes to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }

    /// Convert hex string to bytes
    pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>> {
        hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Invalid hex string: {}", e))
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::{
        config::*,
        utils::*,
        SystemInfo,
        VERSION,
        DESCRIPTION,
        get_system_info,
    };
    
    // Re-export common types
    pub use zkane_common::{
        Secret, Nullifier, Commitment, NullifierHash, DepositNote,
        WithdrawalProof, ZKaneConfig, MerklePath,
    };
    
    // Re-export crypto functions
    pub use zkane_crypto::{
        generate_commitment, generate_nullifier_hash, MerkleTree,
    };
    
    // Re-export core functions
    pub use zkane_core::{
        PrivacyPool, generate_deposit_note, verify_deposit_note,
    };
    
    // Re-export alkanes types
    pub use alkanes_support::id::AlkaneId;
    
    // Re-export error types
    pub use anyhow::{Error, Result};
}

// Test modules (only compiled in test configuration)
#[cfg(test)]
pub mod tests;

// Integration with alkanes framework
#[cfg(test)]
pub mod alkanes_integration {
    //! Integration utilities for alkanes framework
    
    use super::*;
    use alkanes_support::cellpack::Cellpack;
    use alkanes_support::id::AlkaneId;

    /// Convert vector to cellpack for alkanes messaging
    pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
        Cellpack {
            target: AlkaneId {
                block: v[0],
                tx: v[1]
            },
            inputs: v[2..].into()
        }
    }

    /// ZKane opcodes for contract interaction
    pub mod opcodes {
        /// Deposit opcode
        pub const DEPOSIT: u128 = 0;
        
        /// Withdrawal opcode
        pub const WITHDRAW: u128 = 1;
        
        /// Get merkle root opcode
        pub const GET_MERKLE_ROOT: u128 = 1;
        
        /// Get commitment count opcode
        pub const GET_COMMITMENT_COUNT: u128 = 2;
        
        /// Check nullifier spent opcode
        pub const IS_NULLIFIER_SPENT: u128 = 3;
        
        /// Factory create pool opcode
        pub const FACTORY_CREATE_POOL: u128 = 0;
        
        /// Factory get pool count opcode
        pub const FACTORY_GET_POOL_COUNT: u128 = 1;
        
        /// Factory pool exists opcode
        pub const FACTORY_POOL_EXISTS: u128 = 2;
        
        /// Factory get pool ID opcode
        pub const FACTORY_GET_POOL_ID: u128 = 3;
    }
}

// Documentation tests
#[cfg(doctest)]
mod doctests {
    //! Documentation tests to ensure examples in docs work correctly
}