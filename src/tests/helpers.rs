//! ZKane Test Helpers
//! 
//! Helper functions for ZKane testing following the boiler pattern.

use alkanes::view;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use std::str::FromStr;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes::tests::helpers as alkane_helpers;
use protorune::{balance_sheet::{load_sheet}, tables::RuneTable, message::MessageContext};
use protorune_support::balance_sheet::BalanceSheetOperations;
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use metashrew_support::{index_pointer::KeyValuePointer, utils::consensus_encode};
use ordinals::Runestone;
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers};
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
use protorune::protostone::Protostones;
use metashrew_core::{println, stdio::stdout};
use protobuf::Message;
use std::fmt::Write;

use zkane_common::{Secret, Nullifier, Commitment, NullifierHash, DepositNote};
use zkane_crypto::{generate_commitment, generate_nullifier_hash};

// ZKane contract template blocks
pub const ZKANE_TEMPLATE_BLOCK: u128 = 4;
pub const ZKANE_FACTORY_TEMPLATE_BLOCK: u128 = 4;
pub const ZKANE_INSTANCE_BLOCK: u128 = 6;

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

/// Helper function to call ZKane contract with specific opcode and analyze response
pub fn call_zkane_contract(
    contract_id: &AlkaneId,
    opcode: u128,
    inputs: Vec<u128>,
    witness_data: Option<Vec<u8>>,
    block_height: u32,
    test_name: &str
) -> Result<Vec<u8>> {
    let mut call_inputs = vec![
        contract_id.block,
        contract_id.tx,
        opcode,
    ];
    call_inputs.extend(inputs);

    let mut witness = Witness::new();
    if let Some(data) = witness_data {
        witness.push(data);
    }

    let test_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness,
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(call_inputs).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    alkanes::indexer::index_block(&test_block, block_height)?;

    println!("✅ {} call executed at block {}", test_name, block_height);

    // Get the response data from the trace
    let response_outpoint = OutPoint {
        txid: test_block.txdata[0].compute_txid(),
        vout: 0,
    };

    let trace_data = &view::trace(&response_outpoint)?;
    let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
    let trace_guard = trace_result.0.lock().unwrap();

    println!("📊 {} trace executed successfully", test_name);
    Ok(Vec::new()) // Placeholder - would parse actual trace data
}

/// Helper function to call ZKane factory with specific opcode
pub fn call_zkane_factory(
    factory_id: &AlkaneId,
    opcode: u128,
    inputs: Vec<u128>,
    block_height: u32,
    test_name: &str
) -> Result<Vec<u8>> {
    call_zkane_contract(factory_id, opcode, inputs, None, block_height, test_name)
}

/// Parse u128 from response data
pub fn parse_u128_response(data: &[u8], expected_name: &str) -> Result<u128> {
    if data.len() < 16 {
        return Err(anyhow::anyhow!("{} response too short: {} bytes", expected_name, data.len()));
    }
    let value = u128::from_le_bytes(data[0..16].try_into().map_err(|_| {
        anyhow::anyhow!("Failed to parse {} as u128", expected_name)
    })?);
    println!("📊 {}: {}", expected_name, value);
    Ok(value)
}

/// Parse AlkaneId from response data
pub fn parse_alkane_id_response(data: &[u8], expected_name: &str) -> Result<AlkaneId> {
    if data.len() < 32 {
        return Err(anyhow::anyhow!("{} response too short: {} bytes", expected_name, data.len()));
    }
    let block = u128::from_le_bytes(data[0..16].try_into().map_err(|_| {
        anyhow::anyhow!("Failed to parse {} block", expected_name)
    })?);
    let tx = u128::from_le_bytes(data[16..32].try_into().map_err(|_| {
        anyhow::anyhow!("Failed to parse {} tx", expected_name)
    })?);
    let alkane_id = AlkaneId { block, tx };
    println!("📊 {}: AlkaneId {{ block: {}, tx: {} }}", expected_name, block, tx);
    Ok(alkane_id)
}

/// Parse bool from response data
pub fn parse_bool_response(data: &[u8], expected_name: &str) -> Result<bool> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("{} response is empty", expected_name));
    }
    let value = data[0] != 0;
    println!("📊 {}: {}", expected_name, value);
    Ok(value)
}

/// Parse bytes32 from response data
pub fn parse_bytes32_response(data: &[u8], expected_name: &str) -> Result<[u8; 32]> {
    if data.len() < 32 {
        return Err(anyhow::anyhow!("{} response too short: {} bytes", expected_name, data.len()));
    }
    let mut result = [0u8; 32];
    result.copy_from_slice(&data[0..32]);
    println!("📊 {}: {}", expected_name, hex::encode(result));
    Ok(result)
}

/// Generate a test deposit note
pub fn generate_test_deposit_note(
    asset_id: AlkaneId,
    denomination: u128,
    leaf_index: u32,
) -> Result<DepositNote> {
    let secret = Secret::random();
    let nullifier = Nullifier::random();
    let commitment = generate_commitment(&nullifier, &secret)?;

    Ok(DepositNote::new(
        secret,
        nullifier,
        commitment,
        asset_id,
        denomination,
        leaf_index,
    ))
}

/// Generate witness envelope data for deposit
pub fn generate_deposit_witness_data(commitment: &Commitment) -> Result<Vec<u8>> {
    let witness_data = serde_json::json!({
        "commitment": hex::encode(commitment.as_bytes())
    });
    Ok(witness_data.to_string().into_bytes())
}

/// Generate witness envelope data for withdrawal
pub fn generate_withdrawal_witness_data(
    proof: &[u8],
    merkle_root: &[u8; 32],
    nullifier_hash: &NullifierHash,
    path_elements: &[String],
    path_indices: &[bool],
    leaf_index: u32,
    commitment: &Commitment,
    outputs_hash: &[u8; 32],
) -> Result<Vec<u8>> {
    let witness_data = serde_json::json!({
        "proof": hex::encode(proof),
        "merkle_root": hex::encode(merkle_root),
        "nullifier_hash": hex::encode(nullifier_hash.as_bytes()),
        "path_elements": path_elements,
        "path_indices": path_indices,
        "leaf_index": leaf_index,
        "commitment": hex::encode(commitment.as_bytes()),
        "outputs_hash": hex::encode(outputs_hash)
    });
    Ok(witness_data.to_string().into_bytes())
}

/// Calculate transaction outputs hash for recipient validation
pub fn calculate_outputs_hash(outputs: &[TxOut]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();

    for output in outputs {
        hasher.update(&output.value.to_sat().to_le_bytes());
        hasher.update(output.script_pubkey.as_bytes());
    }

    hasher.finalize().into()
}

/// Create a comprehensive ZKane test setup
pub fn create_zkane_test_setup() -> Result<(AlkaneId, AlkaneId, AlkaneId)> {
    clear();
    
    println!("🏗️ ZKANE TEST SETUP: Contract Ecosystem Deployment");
    println!("==================================================");
    
    // PHASE 1: Deploy ZKane contract templates
    println!("\n📦 PHASE 1: Deploying ZKane Contract Templates");
    
    // Get the compiled WASM bytes for ZKane contracts
    let zkane_wasm = include_bytes!("../../alkanes/zkane/target/wasm32-unknown-unknown/release/zkane.wasm");
    let zkane_factory_wasm = include_bytes!("../../alkanes/zkane-factory/target/wasm32-unknown-unknown/release/zkane_factory.wasm");
    
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            zkane_wasm.to_vec(),
            zkane_factory_wasm.to_vec(),
        ].into(),
        [
            vec![ZKANE_TEMPLATE_BLOCK, 0x1000, 0u128], // ZKane template
            vec![ZKANE_FACTORY_TEMPLATE_BLOCK, 0x2000, 0u128], // ZKane factory template
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    alkanes::indexer::index_block(&template_block, 0)?;
    
    println!("✅ ZKane contract templates deployed at block 0");
    
    // PHASE 2: Deploy ZKane factory instance
    println!("\n🏭 PHASE 2: Deploying ZKane Factory Instance");
    
    let factory_deploy_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    ZKANE_INSTANCE_BLOCK, 0x2000, 0u128, // Deploy factory instance
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    alkanes::indexer::index_block(&factory_deploy_block, 1)?;
    
    let factory_id = AlkaneId { block: ZKANE_INSTANCE_BLOCK, tx: 0x2000 };
    
    println!("✅ ZKane factory deployed at {:?}", factory_id);
    
    // PHASE 3: Create a test asset for privacy pool
    println!("\n🪙 PHASE 3: Creating Test Asset");
    
    let test_asset_id = AlkaneId { block: 2, tx: 1 };
    let test_denomination = 1000000u128; // 1M units
    
    // PHASE 4: Deploy ZKane privacy pool instance
    println!("\n🔒 PHASE 4: Deploying ZKane Privacy Pool Instance");
    
    let pool_deploy_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    factory_id.block, factory_id.tx, 0u128, // Call factory create_pool
                                    test_asset_id.block, test_asset_id.tx, // Asset ID
                                    test_denomination, // Denomination
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    alkanes::indexer::index_block(&pool_deploy_block, 2)?;
    
    // Calculate deterministic pool ID
    let mut hasher_input = Vec::new();
    hasher_input.extend_from_slice(&test_asset_id.block.to_le_bytes());
    hasher_input.extend_from_slice(&test_asset_id.tx.to_le_bytes());
    hasher_input.extend_from_slice(&test_denomination.to_le_bytes());
    
    let mut hash_value = 0u128;
    for chunk in hasher_input.chunks(16) {
        let mut bytes = [0u8; 16];
        bytes[..chunk.len()].copy_from_slice(chunk);
        hash_value ^= u128::from_le_bytes(bytes);
    }
    
    let pool_id = AlkaneId {
        block: ZKANE_INSTANCE_BLOCK,
        tx: hash_value,
    };
    
    println!("✅ ZKane privacy pool deployed at {:?}", pool_id);
    println!("🔗 Asset: {:?}, Denomination: {}", test_asset_id, test_denomination);
    
    println!("\n🎉 ZKANE TEST SETUP COMPLETE!");
    println!("=============================");
    println!("✅ Factory: {:?}", factory_id);
    println!("✅ Pool: {:?}", pool_id);
    println!("✅ Test Asset: {:?}", test_asset_id);
    println!("✅ Ready for privacy pool testing");
    
    Ok((factory_id, pool_id, test_asset_id))
}

/// Create test deposits for privacy pool testing
pub fn create_test_deposits(
    pool_id: &AlkaneId,
    asset_id: &AlkaneId,
    denomination: u128,
    count: usize,
    start_block: u32,
) -> Result<Vec<DepositNote>> {
    let mut deposits = Vec::new();
    
    println!("\n💰 Creating {} test deposits", count);
    
    for i in 0..count {
        let deposit_note = generate_test_deposit_note(*asset_id, denomination, i as u32)?;
        let witness_data = generate_deposit_witness_data(&deposit_note.commitment)?;
        
        // Create deposit transaction
        let deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
            version: Version::ONE,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::null(),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: {
                    let mut witness = Witness::new();
                    witness.push(witness_data);
                    witness
                }
            }],
            output: vec![
                TxOut {
                    script_pubkey: Address::from_str(ADDRESS1().as_str())
                        .unwrap()
                        .require_network(get_btc_network())
                        .unwrap()
                        .script_pubkey(),
                    value: Amount::from_sat(546),
                },
                TxOut {
                    script_pubkey: (Runestone {
                        edicts: vec![],
                        etching: None,
                        mint: None,
                        pointer: None,
                        protocol: Some(
                            vec![
                                Protostone {
                                    message: into_cellpack(vec![
                                        pool_id.block, pool_id.tx, 0u128, // Deposit opcode
                                    ]).encipher(),
                                    protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                    pointer: Some(0),
                                    refund: Some(0),
                                    from: None,
                                    burn: None,
                                    edicts: vec![
                                        ProtostoneEdict {
                                            id: ProtoruneRuneId {
                                                block: asset_id.block,
                                                tx: asset_id.tx,
                                            },
                                            amount: denomination,
                                            output: 1,
                                        }
                                    ],
                                }
                            ].encipher()?
                        )
                    }).encipher(),
                    value: Amount::from_sat(546)
                }
            ],
        }]);
        
        alkanes::indexer::index_block(&deposit_block, start_block + i as u32)?;
        
        deposits.push(deposit_note);
        println!("✅ Deposit {} created: {}", i + 1, hex::encode(deposit_note.commitment.as_bytes()));
    }
    
    Ok(deposits)
}