use anyhow::Result;
use deezel_common::{
    provider::ConcreteProvider,
    traits::{BitcoinRpcProvider, WalletProvider},
    WalletConfig,
};
use bitcoin::Network;
use serde_json::json;
use std::sync::Arc;
use wiremock::matchers::{body_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};
use zkane_core::PrivacyPool;
use zkane_common::{SerializableAlkaneId, ZKaneConfig};

#[tokio::test]
async fn test_get_block_count_with_mock() -> Result<()> {
    // Arrange: Start a mock server
    let server = MockServer::start().await;

    // Arrange: Create a mock response for the `getblockcount` RPC call
    let mock_response = json!({
        "jsonrpc": "2.0",
        "result": 800000,
        "id": 1
    });

    // Arrange: Set up the mock expectation on the server
    Mock::given(method("POST"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "method": "getblockcount",
            "params": serde_json::Value::Null,
            "id": 1
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    // Arrange: Create a ConcreteProvider pointing to the mock server
    let provider = ConcreteProvider::new(
        server.uri(),   // bitcoin_rpc_url
        "".to_string(), // metashrew_rpc_url
        "".to_string(), // sandshrew_rpc_url
        None,           // esplora_url
        "regtest".to_string(),
        None, // wallet_path
        None, // http_client
    )
    .await?;

    // Act: Call the method under test
    let block_count_result = provider.get_block_count().await;

    // Assert: Check that the call was successful and returned the correct value
    assert!(block_count_result.is_ok());
    assert_eq!(block_count_result.unwrap(), 800000);

    Ok(())
}

#[tokio::test]
async fn test_privacy_pool_with_concrete_provider() -> Result<()> {
    // Arrange: Start a mock server
    let server = MockServer::start().await;
    let txid = "some_txid";
    let commitment_hex = "0000000000000000000000000000000000000000000000000000000000000042";

    // Arrange: Mock the get_tx response for the commitment
    Mock::given(method("GET"))
        .and(wiremock::matchers::path(format!("/tx/{}", txid)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "vout": [
                {
                    "scriptpubkey_asm": format!("OP_RETURN {}", commitment_hex),
                    "scriptpubkey": format!("6a{}", commitment_hex),
                    "value": 0
                }
            ]
        })))
        .mount(&server)
        .await;
    
    // Arrange: Mock the get_block_count response
    let mock_block_count_response = json!({
        "jsonrpc": "2.0",
        "result": 100,
        "id": 1
    });

    Mock::given(method("POST"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "method": "getblockcount",
            "params": serde_json::Value::Null,
            "id": 1
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_block_count_response))
        .mount(&server)
        .await;

    // Arrange: Create a ConcreteProvider pointing to the mock server
    let provider = Arc::new(
        ConcreteProvider::new(
            server.uri(), // bitcoin_rpc_url
            "".to_string(), // metashrew_rpc_url
            "".to_string(), // sandshrew_rpc_url
            Some(server.uri()), // esplora_url
            "regtest".to_string(),
            None, // wallet_path
            None, // http_client
        )
        .await?,
    );

    // Arrange: Create the PrivacyPool
    let config = ZKaneConfig::new(
        SerializableAlkaneId { block: 1, tx: 1 },
        1000,
        20,
        vec![],
    );
    let mut pool = PrivacyPool::new(config, provider.clone())?;

    // Act: Add a commitment
    let result = pool.add_commitment(txid).await;

    // Assert
    assert!(result.is_ok(), "add_commitment failed: {:?}", result.err());
    assert_eq!(result.unwrap(), 0); // First leaf index is 0
    assert_eq!(pool.commitment_count(), 1);

    Ok(())
}

#[tokio::test]
async fn test_wallet_creation() -> Result<()> {
    // Arrange
    let server = MockServer::start().await;
    let provider = ConcreteProvider::new(
        server.uri(),   // bitcoin_rpc_url
        "".to_string(), // metashrew_rpc_url
        "".to_string(), // sandshrew_rpc_url
        None,           // esplora_url
        "regtest".to_string(),
        None, // wallet_path
        None, // http_client
    )
    .await?;

    let wallet_config = WalletConfig {
        wallet_path: "/tmp/test_wallet.json".to_string(),
        network: Network::Regtest,
        bitcoin_rpc_url: server.uri(),
        metashrew_rpc_url: "".to_string(),
        network_params: None,
    };

    // Act
    let wallet_info = provider.create_wallet(wallet_config, None, Some("password".to_string())).await?;

    // Assert
    assert!(wallet_info.mnemonic.is_some());
    let mnemonic = wallet_info.mnemonic.unwrap();
    let word_count = mnemonic.split_whitespace().count();
    assert_eq!(word_count, 24);

    Ok(())
}