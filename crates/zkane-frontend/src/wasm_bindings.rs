//! WASM bindings for ZKane functionality integrated directly into the frontend
//!
//! This module provides the WASM interface with simplified implementations
//! to avoid compilation issues with alkanes/metashrew dependencies.

use wasm_bindgen::prelude::*;
use serde::Deserialize;
use crate::types::*;
use sha2::{Digest, Sha256};

// Utility macro for error handling
macro_rules! js_error {
    ($msg:expr) => {
        JsValue::from_str(&format!("ZKane Error: {}", $msg))
    };
}

// ============================================================================
// Core WASM-bindgen Types for JavaScript Interop
// ============================================================================

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WasmAlkaneId {
    block: u64, // Use u64 for JS compatibility
    tx: u64,
}

#[wasm_bindgen]
impl WasmAlkaneId {
    #[wasm_bindgen(constructor)]
    pub fn new(block: u64, tx: u64) -> WasmAlkaneId {
        WasmAlkaneId { block, tx }
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

impl From<&AlkaneId> for WasmAlkaneId {
    fn from(id: &AlkaneId) -> Self {
        WasmAlkaneId {
            block: id.block as u64,
            tx: id.tx as u64,
        }
    }
}

impl From<WasmAlkaneId> for AlkaneId {
    fn from(id: WasmAlkaneId) -> Self {
        AlkaneId {
            block: id.block as u128,
            tx: id.tx as u128,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WasmDepositNote {
    secret: String,
    nullifier: String,
    commitment: String,
    asset_id: WasmAlkaneId,
    denomination: String,
    leaf_index: u32,
}

#[wasm_bindgen]
impl WasmDepositNote {
    #[wasm_bindgen(constructor)]
    pub fn new(
        secret: String,
        nullifier: String,
        commitment: String,
        asset_id: WasmAlkaneId,
        denomination: String,
        leaf_index: u32,
    ) -> WasmDepositNote {
        WasmDepositNote {
            secret,
            nullifier,
            commitment,
            asset_id,
            denomination,
            leaf_index,
        }
    }

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
    pub fn asset_id(&self) -> WasmAlkaneId {
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

impl From<WasmDepositNote> for JsDepositNote {
    fn from(wasm_note: WasmDepositNote) -> Self {
        JsDepositNote::new(
            wasm_note.secret,
            wasm_note.nullifier,
            wasm_note.commitment,
            wasm_note.asset_id.into(),
            wasm_note.denomination,
            wasm_note.leaf_index,
        )
    }
}

// ============================================================================
// Cryptographic Functions (Simplified for WASM compatibility)
// ============================================================================

/// Generate a random secret (32 bytes as hex string)
#[wasm_bindgen]
pub fn generate_random_secret() -> String {
    let mut secret = [0u8; 32];
    getrandom::getrandom(&mut secret).expect("Failed to generate random bytes");
    hex::encode(secret)
}

/// Generate a random nullifier (32 bytes as hex string)
#[wasm_bindgen]
pub fn generate_random_nullifier() -> String {
    let mut nullifier = [0u8; 32];
    getrandom::getrandom(&mut nullifier).expect("Failed to generate random bytes");
    hex::encode(nullifier)
}

/// Generate a commitment from secret and nullifier (simplified using SHA256)
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

    // Simplified commitment generation using SHA256
    let mut hasher = Sha256::new();
    hasher.update(&secret_bytes);
    hasher.update(&nullifier_bytes);
    hasher.update(b"commitment"); // Domain separator
    let commitment: [u8; 32] = hasher.finalize().into();

    Ok(hex::encode(commitment))
}

/// Generate a nullifier hash from nullifier (simplified using SHA256)
#[wasm_bindgen]
pub fn generate_nullifier_hash_from_nullifier(nullifier_hex: &str) -> Result<String, JsValue> {
    let nullifier_bytes = hex::decode(nullifier_hex)
        .map_err(|e| js_error!(format!("Invalid nullifier hex: {}", e)))?;

    if nullifier_bytes.len() != 32 {
        return Err(js_error!("Nullifier must be 32 bytes"));
    }

    // Simplified nullifier hash using SHA256
    let mut hasher = Sha256::new();
    hasher.update(&nullifier_bytes);
    hasher.update(b"nullifier_hash"); // Domain separator
    let nullifier_hash: [u8; 32] = hasher.finalize().into();

    Ok(hex::encode(nullifier_hash))
}

// ============================================================================
// Deposit Note Management (Simplified)
// ============================================================================

/// Generate a complete deposit note (simplified implementation)
#[wasm_bindgen]
pub fn create_deposit_note(
    asset_id: &WasmAlkaneId,
    denomination: &str,
) -> Result<WasmDepositNote, JsValue> {
    let denom: u128 = denomination.parse()
        .map_err(|e| js_error!(format!("Invalid denomination: {}", e)))?;

    // Generate random secret and nullifier
    let secret = generate_random_secret();
    let nullifier = generate_random_nullifier();
    
    // Generate commitment
    let commitment = generate_commitment_from_secret_nullifier(&secret, &nullifier)?;

    Ok(WasmDepositNote::new(
        secret,
        nullifier,
        commitment,
        asset_id.clone(),
        denom.to_string(),
        0, // Placeholder leaf index
    ))
}

/// Verify that a deposit note is valid (simplified implementation)
#[wasm_bindgen]
pub fn verify_deposit_note_validity(note: &WasmDepositNote) -> Result<bool, JsValue> {
    // Verify that the commitment matches the secret and nullifier
    let expected_commitment = generate_commitment_from_secret_nullifier(
        &note.secret, 
        &note.nullifier
    )?;
    
    Ok(expected_commitment == note.commitment)
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

    // Use SHA256 for output hashing
    let mut hasher = Sha256::new();
    for output in outputs {
        hasher.update(&output.value.to_le_bytes());
        hasher.update(output.script_pubkey.as_bytes());
    }

    let hash: [u8; 32] = hasher.finalize().into();
    Ok(hex::encode(hash))
}

// ============================================================================
// Pool ID Generation (Simplified)
// ============================================================================

/// Generate deterministic pool ID for asset/denomination pair
#[wasm_bindgen]
pub fn generate_pool_id(asset_id: &WasmAlkaneId, denomination: &str) -> Result<WasmAlkaneId, JsValue> {
    let denom: u128 = denomination.parse()
        .map_err(|e| js_error!(format!("Invalid denomination: {}", e)))?;

    // Use same logic as factory contract for deterministic pool ID generation
    let mut hasher_input = Vec::new();
    hasher_input.extend_from_slice(&asset_id.block.to_le_bytes());
    hasher_input.extend_from_slice(&asset_id.tx.to_le_bytes());
    hasher_input.extend_from_slice(&denom.to_le_bytes());
    
    let mut hash_value = 0u128;
    for chunk in hasher_input.chunks(16) {
        let mut bytes = [0u8; 16];
        bytes[..chunk.len()].copy_from_slice(chunk);
        hash_value ^= u128::from_le_bytes(bytes);
    }
    
    let pool_id = WasmAlkaneId {
        block: 6, // ZKANE_INSTANCE_BLOCK
        tx: hash_value as u64, // Truncate for JS compatibility
    };

    Ok(pool_id)
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

    let witness_data = serde_json::json!({
        "commitment": commitment_hex
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

/// Get the ZKane frontend version
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get ZKane system information
#[wasm_bindgen]
pub fn get_zkane_info() -> JsValue {
    let info = serde_json::json!({
        "name": "ZKane Privacy Pool Frontend",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Privacy pool for alkanes assets using zero-knowledge proofs",
        "features": [
            "Privacy-preserving transactions",
            "Multi-asset support",
            "Zero-knowledge proofs",
            "Factory pattern",
            "Witness envelope support",
            "Transaction output validation",
            "Direct core integration",
            "Simplified WASM implementation"
        ]
    });

    serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
}