//! ZKane WASM Integration Tests
//! 
//! Tests for the WASM bindings and browser compatibility.

use wasm_bindgen_test::*;
use zkane_common::{Secret, Nullifier, Commitment};
use alkanes_support::id::AlkaneId;
use crate::tests::helpers::*;

// Configure for browser testing
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_crypto_functions() {
    // Test secret generation
    let secret_hex = zkane_wasm::generate_random_secret();
    assert_eq!(secret_hex.len(), 64); // 32 bytes * 2 hex chars
    
    // Test nullifier generation
    let nullifier_hex = zkane_wasm::generate_random_nullifier();
    assert_eq!(nullifier_hex.len(), 64); // 32 bytes * 2 hex chars
    
    // Test commitment generation
    let commitment_hex = zkane_wasm::generate_commitment_from_secret_nullifier(
        &secret_hex,
        &nullifier_hex,
    ).unwrap();
    assert_eq!(commitment_hex.len(), 64); // 32 bytes * 2 hex chars
    
    // Test nullifier hash generation
    let nullifier_hash_hex = zkane_wasm::generate_nullifier_hash_from_nullifier(
        &nullifier_hex,
    ).unwrap();
    assert_eq!(nullifier_hash_hex.len(), 64); // 32 bytes * 2 hex chars
    
    zkane_wasm::log("✅ WASM crypto functions working correctly");
}

#[wasm_bindgen_test]
fn test_wasm_deposit_note_creation() {
    let asset_id = AlkaneId { block: 2, tx: 1 };
    let denomination = "1000000";
    
    // Create deposit note
    let deposit_note = zkane_wasm::create_deposit_note(&asset_id, denomination).unwrap();
    
    // Verify all fields are present and valid
    assert_eq!(deposit_note.secret().len(), 64);
    assert_eq!(deposit_note.nullifier().len(), 64);
    assert_eq!(deposit_note.commitment().len(), 64);
    assert_eq!(deposit_note.asset_id().block(), 2);
    assert_eq!(deposit_note.asset_id().tx(), 1);
    assert_eq!(deposit_note.denomination(), "1000000");
    
    // Verify deposit note validity
    let is_valid = zkane_wasm::verify_deposit_note_validity(&deposit_note).unwrap();
    assert!(is_valid);
    
    zkane_wasm::log("✅ WASM deposit note creation working correctly");
}

#[wasm_bindgen_test]
fn test_wasm_transaction_output_validation() {
    let outputs_json = r#"[
        {
            "value": 546,
            "script_pubkey": "76a914..."
        },
        {
            "value": 1000000,
            "script_pubkey": "a914..."
        }
    ]"#;
    
    let outputs_hash = zkane_wasm::hash_transaction_outputs(outputs_json).unwrap();
    assert_eq!(outputs_hash.len(), 64); // 32 bytes * 2 hex chars
    
    // Test with same inputs should produce same hash
    let outputs_hash2 = zkane_wasm::hash_transaction_outputs(outputs_json).unwrap();
    assert_eq!(outputs_hash, outputs_hash2);
    
    zkane_wasm::log("✅ WASM transaction output validation working correctly");
}

#[wasm_bindgen_test]
fn test_wasm_witness_envelope_generation() {
    let commitment_hex = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    
    // Test deposit witness generation
    let deposit_witness = zkane_wasm::generate_deposit_witness(commitment_hex).unwrap();
    assert!(!deposit_witness.is_empty());
    
    // Parse as JSON to verify structure
    let witness_json: serde_json::Value = serde_json::from_str(&deposit_witness).unwrap();
    assert!(witness_json["commitment"].is_string());
    
    // Test withdrawal witness generation
    let proof_hex = "42".repeat(256); // 256 bytes of 0x42
    let merkle_root_hex = "42".repeat(32); // 32 bytes of 0x42
    let nullifier_hash_hex = "43".repeat(32); // 32 bytes of 0x43
    let path_elements_json = r#"["elem1", "elem2", "elem3"]"#;
    let path_indices_json = r#"[false, true, false]"#;
    let outputs_hash_hex = "44".repeat(32); // 32 bytes of 0x44
    
    let withdrawal_witness = zkane_wasm::generate_withdrawal_witness(
        &proof_hex,
        &merkle_root_hex,
        &nullifier_hash_hex,
        path_elements_json,
        path_indices_json,
        0,
        commitment_hex,
        &outputs_hash_hex,
    ).unwrap();
    
    assert!(!withdrawal_witness.is_empty());
    
    // Parse as JSON to verify structure
    let withdrawal_json: serde_json::Value = serde_json::from_str(&withdrawal_witness).unwrap();
    assert!(withdrawal_json["proof"].is_string());
    assert!(withdrawal_json["merkle_root"].is_string());
    assert!(withdrawal_json["nullifier_hash"].is_string());
    assert!(withdrawal_json["path_elements"].is_array());
    assert!(withdrawal_json["path_indices"].is_array());
    assert!(withdrawal_json["leaf_index"].is_number());
    assert!(withdrawal_json["commitment"].is_string());
    assert!(withdrawal_json["outputs_hash"].is_string());
    
    zkane_wasm::log("✅ WASM witness envelope generation working correctly");
}

#[wasm_bindgen_test]
fn test_wasm_pool_id_generation() {
    let asset_id = AlkaneId { block: 2, tx: 1 };
    let denomination = "1000000";
    
    let pool_id = zkane_wasm::generate_pool_id(&asset_id, denomination).unwrap();
    
    // Pool ID should be deterministic
    let pool_id2 = zkane_wasm::generate_pool_id(&asset_id, denomination).unwrap();
    assert_eq!(pool_id.block(), pool_id2.block());
    assert_eq!(pool_id.tx(), pool_id2.tx());
    
    // Different inputs should produce different pool IDs
    let different_asset = AlkaneId { block: 3, tx: 1 };
    let different_pool_id = zkane_wasm::generate_pool_id(&different_asset, denomination).unwrap();
    assert!(pool_id.tx() != different_pool_id.tx());
    
    zkane_wasm::log("✅ WASM pool ID generation working correctly");
}

#[wasm_bindgen_test]
fn test_wasm_proof_generation_placeholder() {
    let secret_hex = zkane_wasm::generate_random_secret();
    let nullifier_hex = zkane_wasm::generate_random_nullifier();
    let merkle_path_json = r#"{
        "elements": ["elem1", "elem2"],
        "indices": [false, true]
    }"#;
    let outputs_hash_hex = "42".repeat(32);
    
    let proof_hex = zkane_wasm::generate_withdrawal_proof_placeholder(
        &secret_hex,
        &nullifier_hex,
        merkle_path_json,
        &outputs_hash_hex,
    ).unwrap();
    
    // Should produce a deterministic proof for same inputs
    let proof_hex2 = zkane_wasm::generate_withdrawal_proof_placeholder(
        &secret_hex,
        &nullifier_hex,
        merkle_path_json,
        &outputs_hash_hex,
    ).unwrap();
    assert_eq!(proof_hex, proof_hex2);
    
    // Should be 256 bytes (512 hex chars)
    assert_eq!(proof_hex.len(), 512);
    
    zkane_wasm::log("✅ WASM proof generation placeholder working correctly");
}

#[wasm_bindgen_test]
fn test_wasm_utility_functions() {
    // Test hex validation
    let valid_hex = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    assert!(zkane_wasm::is_valid_hex(valid_hex, 32));
    assert!(!zkane_wasm::is_valid_hex(valid_hex, 16)); // Wrong length
    assert!(!zkane_wasm::is_valid_hex("invalid_hex", 32)); // Invalid hex
    
    // Test version info
    let version = zkane_wasm::get_version();
    assert!(!version.is_empty());
    
    // Test system info
    let info = zkane_wasm::get_zkane_info();
    assert!(!info.is_null());
    
    // Test timestamp
    let timestamp = zkane_wasm::get_timestamp();
    assert!(timestamp > 0.0);
    
    zkane_wasm::log("✅ WASM utility functions working correctly");
}

#[wasm_bindgen_test]
fn test_wasm_error_handling() {
    // Test invalid hex input
    let result = zkane_wasm::generate_commitment_from_secret_nullifier(
        "invalid_hex",
        "also_invalid",
    );
    assert!(result.is_err());
    
    // Test invalid length
    let short_hex = "1234"; // Too short
    let normal_hex = zkane_wasm::generate_random_secret();
    let result = zkane_wasm::generate_commitment_from_secret_nullifier(
        &short_hex,
        &normal_hex,
    );
    assert!(result.is_err());
    
    // Test invalid JSON
    let result = zkane_wasm::hash_transaction_outputs("invalid_json");
    assert!(result.is_err());
    
    zkane_wasm::log("✅ WASM error handling working correctly");
}

#[wasm_bindgen_test]
fn test_wasm_end_to_end_flow() {
    zkane_wasm::log("🚀 Starting WASM end-to-end flow test");
    
    // Step 1: Create deposit note
    let asset_id = AlkaneId { block: 2, tx: 1 };
    let denomination = "1000000";
    let deposit_note = zkane_wasm::create_deposit_note(&asset_id, denomination).unwrap();
    
    zkane_wasm::log("✅ Step 1: Deposit note created");
    
    // Step 2: Generate deposit witness
    let deposit_witness = zkane_wasm::generate_deposit_witness(&deposit_note.commitment()).unwrap();
    
    zkane_wasm::log("✅ Step 2: Deposit witness generated");
    
    // Step 3: Calculate transaction outputs hash
    let outputs_json = r#"[
        {
            "value": 546,
            "script_pubkey": "76a914abc123def456789abc123def456789abc123def488ac"
        }
    ]"#;
    let outputs_hash = zkane_wasm::hash_transaction_outputs(outputs_json).unwrap();
    
    zkane_wasm::log("✅ Step 3: Transaction outputs hash calculated");
    
    // Step 4: Generate nullifier hash
    let nullifier_hash = zkane_wasm::generate_nullifier_hash_from_nullifier(
        &deposit_note.nullifier()
    ).unwrap();
    
    zkane_wasm::log("✅ Step 4: Nullifier hash generated");
    
    // Step 5: Generate withdrawal proof (placeholder)
    let merkle_path_json = r#"{
        "elements": ["elem1", "elem2", "elem3"],
        "indices": [false, true, false]
    }"#;
    let proof = zkane_wasm::generate_withdrawal_proof_placeholder(
        &deposit_note.secret(),
        &deposit_note.nullifier(),
        merkle_path_json,
        &outputs_hash,
    ).unwrap();
    
    zkane_wasm::log("✅ Step 5: Withdrawal proof generated");
    
    // Step 6: Generate withdrawal witness
    let merkle_root = "42".repeat(32);
    let path_elements_json = r#"["elem1", "elem2", "elem3"]"#;
    let path_indices_json = r#"[false, true, false]"#;
    
    let withdrawal_witness = zkane_wasm::generate_withdrawal_witness(
        &proof,
        &merkle_root,
        &nullifier_hash,
        path_elements_json,
        path_indices_json,
        0,
        &deposit_note.commitment(),
        &outputs_hash,
    ).unwrap();
    
    zkane_wasm::log("✅ Step 6: Withdrawal witness generated");
    
    // Step 7: Verify all data is consistent
    assert!(!deposit_witness.is_empty());
    assert!(!withdrawal_witness.is_empty());
    assert_eq!(outputs_hash.len(), 64);
    assert_eq!(proof.len(), 512);
    
    zkane_wasm::log("✅ Step 7: Data consistency verified");
    
    zkane_wasm::log("🎉 WASM end-to-end flow test completed successfully!");
}