//! # ZKane WASM Bindings for Browser DApp Integration
//!
//! This crate provides WebAssembly (WASM) bindings for all ZKane functionality needed
//! to build a browser-based privacy pool decentralized application (dapp). It exposes
//! a JavaScript-compatible API that enables privacy-preserving transactions directly
//! in web browsers.
//!
//! ## Overview
//!
//! The ZKane WASM bindings provide a complete interface for:
//!
//! - **Cryptographic Operations**: Secret/nullifier generation, commitment creation
//! - **Deposit Management**: Creating and validating deposit notes
//! - **Withdrawal Processing**: Proof generation and verification
//! - **Pool Management**: Pool ID generation and registry operations
//! - **Transaction Validation**: Output hash calculation for recipient validation
//! - **Witness Envelope Support**: Large data storage for Bitcoin transactions
//!
//! ## Browser Compatibility
//!
//! This crate is designed to work in modern web browsers with WebAssembly support:
//!
//! - Chrome 57+, Firefox 52+, Safari 11+, Edge 16+
//! - Requires `wasm-bindgen` and `wasm-pack` for building
//! - Compatible with modern JavaScript frameworks (React, Vue, Angular)
//! - Supports both ES modules and CommonJS
//!
//! ## Usage in JavaScript
//!
//! ```javascript
//! import init, {
//!     create_deposit_note,
//!     generate_withdrawal_proof_placeholder,
//!     hash_transaction_outputs
//! } from './pkg/zkane_wasm.js';
//!
//! // Initialize the WASM module
//! await init();
//!
//! // Create a deposit note
//! const assetId = { block: 2, tx: 1 };
//! const denomination = "1000000";
//! const depositNote = create_deposit_note(assetId, denomination);
//!
//! console.log("Deposit created:", depositNote.commitment());
//! ```
//!
//! ## Security Considerations
//!
//! - **Client-Side Cryptography**: All cryptographic operations run in the browser
//! - **Memory Safety**: WASM provides memory isolation from JavaScript
//! - **Secure Random Generation**: Uses browser's crypto.getRandomValues()
//! - **No Network Dependencies**: All operations work offline
//!
//! ## Performance Notes
//!
//! - **WASM Overhead**: Some performance cost compared to native implementations
//! - **Memory Usage**: Efficient memory management with automatic cleanup
//! - **Bundle Size**: Optimized for minimal WASM binary size
//! - **Initialization**: One-time WASM module initialization required

use wasm_bindgen::prelude::*;
use zkane_common::{
    Secret, Nullifier, Commitment, NullifierHash, DepositNote, 
    WithdrawalProof, ZKaneConfig, MerklePath
};
use zkane_crypto::{generate_commitment, generate_nullifier_hash, MerkleTree};
use zkane_core::{PrivacyPool, generate_deposit_note, verify_deposit_note};
use alkanes_support::id::AlkaneId;
use serde::{Deserialize, Serialize};
use js_sys::Promise;
use wasm_bindgen_futures::future_to_promise;

// Set up console error panic hook for better debugging
#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();
}

// Utility macro for error handling
macro_rules! js_error {
    ($msg:expr) => {
        JsValue::from_str(&format!("ZKane Error: {}", $msg))
    };
}

// ============================================================================
// Core Types for JavaScript Interop
// ============================================================================

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct JsAlkaneId {
    block: u64, // Use u64 for JS compatibility
    tx: u64,
}

#[wasm_bindgen]
impl JsAlkaneId {
    #[wasm_bindgen(constructor)]
    pub fn new(block: u64, tx: u64) -> JsAlkaneId {
        JsAlkaneId { block, tx }
    }

    #[wasm_bindgen(getter)]
    pub fn block(&self) -> u64 {
        self.block
    }

    #[wasm_bindgen(getter)]
    pub fn tx(&self) -> u64 {
        self.tx
    }
}

impl From<JsAlkaneId> for AlkaneId {
    fn from(js_id: JsAlkaneId) -> Self {
        AlkaneId {
            block: js_id.block as u128,
            tx: js_id.tx as u128,
        }
    }
}

impl From<AlkaneId> for JsAlkaneId {
    fn from(id: AlkaneId) -> Self {
        JsAlkaneId {
            block: id.block as u64,
            tx: id.tx as u64,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct JsDepositNote {
    secret: String,
    nullifier: String,
    commitment: String,
    asset_id: JsAlkaneId,
    denomination: String, // Use string for large numbers
    leaf_index: u32,
}

#[wasm_bindgen]
impl JsDepositNote {
    #[wasm_bindgen(getter)]
    pub fn secret(&self) -> String {
        self.secret.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn nullifier(&self) -> String {
        self.nullifier.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn commitment(&self) -> String {
        self.commitment.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn asset_id(&self) -> JsAlkaneId {
        self.asset_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn denomination(&self) -> String {
        self.denomination.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn leaf_index(&self) -> u32 {
        self.leaf_index
    }
}

// ============================================================================
// Cryptographic Functions
// ============================================================================

/// Generate a random secret (32 bytes as hex string)
#[wasm_bindgen]
pub fn generate_random_secret() -> String {
    let secret = Secret::random();
    hex::encode(secret.as_bytes())
}

/// Generate a random nullifier (32 bytes as hex string)
#[wasm_bindgen]
pub fn generate_random_nullifier() -> String {
    let nullifier = Nullifier::random();
    hex::encode(nullifier.as_bytes())
}

/// Generate a commitment from secret and nullifier
#[wasm_bindgen]
pub fn generate_commitment_from_secret_nullifier(
    secret_hex: &str,
    nullifier_hex: &str,
) -> Result<String, JsValue> {
    let secret_bytes = hex::decode(secret_hex)
        .map_err(|e| js_error!(format!("Invalid secret hex: {}", e)))?;
    let nullifier_bytes = hex::decode(nullifier_hex)
        .map_err(|e| js_error!(format!("Invalid nullifier hex: {}", e)))?;

    if secret_bytes.len() != 32 {
        return Err(js_error!("Secret must be 32 bytes"));
    }
    if nullifier_bytes.len() != 32 {
        return Err(js_error!("Nullifier must be 32 bytes"));
    }

    let mut secret_array = [0u8; 32];
    let mut nullifier_array = [0u8; 32];
    secret_array.copy_from_slice(&secret_bytes);
    nullifier_array.copy_from_slice(&nullifier_bytes);

    let secret = Secret::new(secret_array);
    let nullifier = Nullifier::new(nullifier_array);

    let commitment = generate_commitment(&nullifier, &secret)
        .map_err(|e| js_error!(format!("Failed to generate commitment: {}", e)))?;

    Ok(hex::encode(commitment.as_bytes()))
}

/// Generate a nullifier hash from nullifier
#[wasm_bindgen]
pub fn generate_nullifier_hash_from_nullifier(nullifier_hex: &str) -> Result<String, JsValue> {
    let nullifier_bytes = hex::decode(nullifier_hex)
        .map_err(|e| js_error!(format!("Invalid nullifier hex: {}", e)))?;

    if nullifier_bytes.len() != 32 {
        return Err(js_error!("Nullifier must be 32 bytes"));
    }

    let mut nullifier_array = [0u8; 32];
    nullifier_array.copy_from_slice(&nullifier_bytes);
    let nullifier = Nullifier::new(nullifier_array);

    let nullifier_hash = generate_nullifier_hash(&nullifier)
        .map_err(|e| js_error!(format!("Failed to generate nullifier hash: {}", e)))?;

    Ok(hex::encode(nullifier_hash.as_bytes()))
}

// ============================================================================
// Deposit Note Management
// ============================================================================

/// Generate a complete deposit note for the given asset and denomination
#[wasm_bindgen]
pub fn create_deposit_note(
    asset_id: &JsAlkaneId,
    denomination: &str,
) -> Result<JsDepositNote, JsValue> {
    let alkane_id: AlkaneId = asset_id.clone().into();
    let denom: u128 = denomination.parse()
        .map_err(|e| js_error!(format!("Invalid denomination: {}", e)))?;

    let deposit_note = generate_deposit_note(alkane_id, denom)
        .map_err(|e| js_error!(format!("Failed to generate deposit note: {}", e)))?;

    Ok(JsDepositNote {
        secret: hex::encode(deposit_note.secret.as_bytes()),
        nullifier: hex::encode(deposit_note.nullifier.as_bytes()),
        commitment: hex::encode(deposit_note.commitment.as_bytes()),
        asset_id: JsAlkaneId::from(alkanes_support::id::AlkaneId::from(deposit_note.asset_id)),
        denomination: deposit_note.denomination.to_string(),
        leaf_index: deposit_note.leaf_index,
    })
}

/// Verify that a deposit note is valid
#[wasm_bindgen]
pub fn verify_deposit_note_validity(note: &JsDepositNote) -> Result<bool, JsValue> {
    // Parse the note back to internal format
    let secret_bytes = hex::decode(&note.secret)
        .map_err(|e| js_error!(format!("Invalid secret hex: {}", e)))?;
    let nullifier_bytes = hex::decode(&note.nullifier)
        .map_err(|e| js_error!(format!("Invalid nullifier hex: {}", e)))?;
    let commitment_bytes = hex::decode(&note.commitment)
        .map_err(|e| js_error!(format!("Invalid commitment hex: {}", e)))?;

    if secret_bytes.len() != 32 || nullifier_bytes.len() != 32 || commitment_bytes.len() != 32 {
        return Err(js_error!("Invalid byte lengths"));
    }

    let mut secret_array = [0u8; 32];
    let mut nullifier_array = [0u8; 32];
    let mut commitment_array = [0u8; 32];
    secret_array.copy_from_slice(&secret_bytes);
    nullifier_array.copy_from_slice(&nullifier_bytes);
    commitment_array.copy_from_slice(&commitment_bytes);

    let secret = Secret::new(secret_array);
    let nullifier = Nullifier::new(nullifier_array);
    let commitment = Commitment::new(commitment_array);
    let asset_id: AlkaneId = note.asset_id.clone().into();
    let denomination: u128 = note.denomination.parse()
        .map_err(|e| js_error!(format!("Invalid denomination: {}", e)))?;

    let internal_note = DepositNote::new(
        secret,
        nullifier,
        commitment,
        asset_id.into(),
        denomination,
        note.leaf_index,
    );

    let is_valid = verify_deposit_note(&internal_note)
        .map_err(|e| js_error!(format!("Failed to verify deposit note: {}", e)))?;

    Ok(is_valid)
}

// ============================================================================
// Transaction Output Validation
// ============================================================================

/// Hash transaction outputs for recipient validation
#[wasm_bindgen]
pub fn hash_transaction_outputs(outputs_json: &str) -> Result<String, JsValue> {
    #[derive(Deserialize)]
    struct TxOutput {
        value: u64,
        script_pubkey: String,
    }

    let outputs: Vec<TxOutput> = serde_json::from_str(outputs_json)
        .map_err(|e| js_error!(format!("Invalid outputs JSON: {}", e)))?;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();

    for output in outputs {
        hasher.update(&output.value.to_le_bytes());
        hasher.update(output.script_pubkey.as_bytes());
    }

    let hash: [u8; 32] = hasher.finalize().into();
    Ok(hex::encode(hash))
}

// ============================================================================
// Witness Envelope Generation
// ============================================================================

/// Generate deposit witness envelope data
#[wasm_bindgen]
pub fn generate_deposit_witness(commitment_hex: &str) -> Result<String, JsValue> {
    let commitment_bytes = hex::decode(commitment_hex)
        .map_err(|e| js_error!(format!("Invalid commitment hex: {}", e)))?;

    if commitment_bytes.len() != 32 {
        return Err(js_error!("Commitment must be 32 bytes"));
    }

    let mut commitment_array = [0u8; 32];
    commitment_array.copy_from_slice(&commitment_bytes);

    let witness_data = serde_json::json!({
        "commitment": hex::encode(commitment_array)
    });

    Ok(witness_data.to_string())
}

/// Generate withdrawal witness envelope data
#[wasm_bindgen]
pub fn generate_withdrawal_witness(
    proof_hex: &str,
    merkle_root_hex: &str,
    nullifier_hash_hex: &str,
    path_elements_json: &str,
    path_indices_json: &str,
    leaf_index: u32,
    commitment_hex: &str,
    outputs_hash_hex: &str,
) -> Result<String, JsValue> {
    // Parse all inputs
    let proof = hex::decode(proof_hex)
        .map_err(|e| js_error!(format!("Invalid proof hex: {}", e)))?;
    
    let merkle_root = hex::decode(merkle_root_hex)
        .map_err(|e| js_error!(format!("Invalid merkle root hex: {}", e)))?;
    
    let nullifier_hash = hex::decode(nullifier_hash_hex)
        .map_err(|e| js_error!(format!("Invalid nullifier hash hex: {}", e)))?;
    
    let commitment = hex::decode(commitment_hex)
        .map_err(|e| js_error!(format!("Invalid commitment hex: {}", e)))?;
    
    let outputs_hash = hex::decode(outputs_hash_hex)
        .map_err(|e| js_error!(format!("Invalid outputs hash hex: {}", e)))?;

    // Parse path elements and indices
    let path_elements: Vec<String> = serde_json::from_str(path_elements_json)
        .map_err(|e| js_error!(format!("Invalid path elements JSON: {}", e)))?;
    
    let path_indices: Vec<bool> = serde_json::from_str(path_indices_json)
        .map_err(|e| js_error!(format!("Invalid path indices JSON: {}", e)))?;

    // Validate lengths
    if merkle_root.len() != 32 || nullifier_hash.len() != 32 || 
       commitment.len() != 32 || outputs_hash.len() != 32 {
        return Err(js_error!("Hash values must be 32 bytes"));
    }

    let witness_data = serde_json::json!({
        "proof": hex::encode(proof),
        "merkle_root": hex::encode(merkle_root),
        "nullifier_hash": hex::encode(nullifier_hash),
        "path_elements": path_elements,
        "path_indices": path_indices,
        "leaf_index": leaf_index,
        "commitment": hex::encode(commitment),
        "outputs_hash": hex::encode(outputs_hash)
    });

    Ok(witness_data.to_string())
}

// ============================================================================
// Pool ID Generation (Factory Pattern)
// ============================================================================

/// Generate deterministic pool ID for asset/denomination pair
#[wasm_bindgen]
pub fn generate_pool_id(asset_id: &JsAlkaneId, denomination: &str) -> Result<JsAlkaneId, JsValue> {
    let alkane_id: AlkaneId = asset_id.clone().into();
    let denom: u128 = denomination.parse()
        .map_err(|e| js_error!(format!("Invalid denomination: {}", e)))?;

    // Use same logic as factory contract
    let mut hasher_input = Vec::new();
    hasher_input.extend_from_slice(&alkane_id.block.to_le_bytes());
    hasher_input.extend_from_slice(&alkane_id.tx.to_le_bytes());
    hasher_input.extend_from_slice(&denom.to_le_bytes());
    
    let mut hash_value = 0u128;
    for chunk in hasher_input.chunks(16) {
        let mut bytes = [0u8; 16];
        bytes[..chunk.len()].copy_from_slice(chunk);
        hash_value ^= u128::from_le_bytes(bytes);
    }
    
    let pool_id = AlkaneId {
        block: 6, // ZKANE_INSTANCE_BLOCK
        tx: hash_value,
    };

    Ok(pool_id.into())
}

// ============================================================================
// Proof Generation (Placeholder for Noir Integration)
// ============================================================================

/// Generate a withdrawal proof (placeholder - would integrate with Noir)
#[wasm_bindgen]
pub fn generate_withdrawal_proof_placeholder(
    secret_hex: &str,
    nullifier_hex: &str,
    merkle_path_json: &str,
    outputs_hash_hex: &str,
) -> Result<String, JsValue> {
    // This is a placeholder implementation
    // In production, this would call the Noir prover
    
    let secret = hex::decode(secret_hex)
        .map_err(|e| js_error!(format!("Invalid secret hex: {}", e)))?;
    let nullifier = hex::decode(nullifier_hex)
        .map_err(|e| js_error!(format!("Invalid nullifier hex: {}", e)))?;
    let outputs_hash = hex::decode(outputs_hash_hex)
        .map_err(|e| js_error!(format!("Invalid outputs hash hex: {}", e)))?;

    if secret.len() != 32 || nullifier.len() != 32 || outputs_hash.len() != 32 {
        return Err(js_error!("Invalid input lengths"));
    }

    // Generate a deterministic mock proof
    let mut proof = Vec::new();
    proof.extend_from_slice(&secret);
    proof.extend_from_slice(&nullifier);
    proof.extend_from_slice(&outputs_hash);
    
    // Pad to realistic proof size (256 bytes)
    while proof.len() < 256 {
        proof.push(0x42);
    }

    Ok(hex::encode(proof))
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Log a message to the browser console
#[wasm_bindgen]
pub fn log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

/// Get the current timestamp
#[wasm_bindgen]
pub fn get_timestamp() -> f64 {
    js_sys::Date::now()
}

/// Validate hex string
#[wasm_bindgen]
pub fn is_valid_hex(hex_str: &str, expected_length: usize) -> bool {
    if let Ok(bytes) = hex::decode(hex_str) {
        bytes.len() == expected_length
    } else {
        false
    }
}

// ============================================================================
// Version Information
// ============================================================================

/// Get the ZKane WASM version
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get ZKane system information
#[wasm_bindgen]
pub fn get_zkane_info() -> JsValue {
    let info = serde_json::json!({
        "name": "ZKane Privacy Pool",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Privacy pool for alkanes assets using zero-knowledge proofs",
        "features": [
            "Privacy-preserving transactions",
            "Multi-asset support",
            "Zero-knowledge proofs",
            "Factory pattern",
            "Witness envelope support",
            "Transaction output validation"
        ]
    });

    serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
}

// ============================================================================
// Error Handling
// ============================================================================

#[wasm_bindgen]
pub struct ZKaneError {
    message: String,
}

#[wasm_bindgen]
impl ZKaneError {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl From<anyhow::Error> for ZKaneError {
    fn from(err: anyhow::Error) -> Self {
        ZKaneError {
            message: err.to_string(),
        }
    }
}