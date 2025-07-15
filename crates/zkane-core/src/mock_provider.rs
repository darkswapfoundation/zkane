use deezel_common::{
    alkanes::{
        types::{EnhancedExecuteParams, EnhancedExecuteResult},
        AlkanesInspectConfig, AlkanesInspectResult, AlkaneBalance,
    },
    ord::{
        AddressInfo as OrdAddressInfo, Block as OrdBlock, Blocks as OrdBlocks,
        Children as OrdChildren, Inscription as OrdInscription, Inscriptions as OrdInscriptions,
        Output as OrdOutput, ParentInscriptions as OrdParents, RuneInfo as OrdRuneInfo,
        Runes as OrdRunes, SatResponse as OrdSat, TxInfo as OrdTxInfo,
    },
    traits::*,
    *,
};
use alkanes_support::proto::alkanes as alkanes_pb;
use async_trait::async_trait;
use bitcoin::{
    secp256k1::{schnorr, All, Secp256k1},
    Network, OutPoint, Transaction, TxOut,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use protorune_support::proto::protorune as protorune_pb;

#[derive(Clone)]
pub struct MockProvider {
    pub responses: Arc<Mutex<HashMap<String, JsonValue>>>,
    secp: Secp256k1<All>,
    network: Network,
}

impl MockProvider {
    pub fn new(network: Network) -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            secp: Secp256k1::new(),
            network,
        }
    }

    pub fn add_response(&mut self, txid: &str, response: JsonValue) {
        self.responses.lock().unwrap().insert(txid.to_string(), response);
    }
}

#[async_trait(?Send)]
impl JsonRpcProvider for MockProvider {
    async fn call(
        &self,
        _url: &str,
        _method: &str,
        _params: JsonValue,
        _id: u64,
    ) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_bytecode(&self, _block: &str, _tx: &str) -> Result<String> {
        Ok(String::new())
    }
}

#[async_trait(?Send)]
impl StorageProvider for MockProvider {
    async fn read(&self, _key: &str) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
    async fn write(&self, _key: &str, _data: &[u8]) -> Result<()> {
        Ok(())
    }
    async fn exists(&self, _key: &str) -> Result<bool> {
        Ok(false)
    }
    async fn delete(&self, _key: &str) -> Result<()> {
        Ok(())
    }
    async fn list_keys(&self, _prefix: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
    fn storage_type(&self) -> &'static str {
        "mock"
    }
}

#[async_trait(?Send)]
impl NetworkProvider for MockProvider {
    async fn get(&self, _url: &str) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
    async fn post(&self, _url: &str, _body: &[u8], _content_type: &str) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
    async fn is_reachable(&self, _url: &str) -> bool {
        true
    }
}

#[async_trait(?Send)]
impl CryptoProvider for MockProvider {
    fn random_bytes(&self, _len: usize) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
    fn sha256(&self, _data: &[u8]) -> Result<[u8; 32]> {
        Ok([0; 32])
    }
    fn sha3_256(&self, _data: &[u8]) -> Result<[u8; 32]> {
        Ok([0; 32])
    }
    async fn encrypt_aes_gcm(&self, _data: &[u8], _key: &[u8], _nonce: &[u8]) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
    async fn decrypt_aes_gcm(&self, _data: &[u8], _key: &[u8], _nonce: &[u8]) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
    async fn pbkdf2_derive(
        &self,
        _password: &[u8],
        _salt: &[u8],
        _iterations: u32,
        _key_len: usize,
    ) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
}

#[async_trait(?Send)]
impl PgpProvider for MockProvider {
    async fn generate_keypair(
        &self,
        _user_id: &str,
        _passphrase: Option<&str>,
    ) -> Result<PgpKeyPair> {
        unimplemented!()
    }
    async fn import_key(&self, _armored_key: &str) -> Result<PgpKey> {
        unimplemented!()
    }
    async fn export_key(&self, _key: &PgpKey, _include_private: bool) -> Result<String> {
        unimplemented!()
    }
    async fn encrypt(
        &self,
        _data: &[u8],
        _recipient_keys: &[PgpKey],
        _armor: bool,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }
    async fn decrypt(
        &self,
        _encrypted_data: &[u8],
        _private_key: &PgpKey,
        _passphrase: Option<&str>,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }
    async fn sign(
        &self,
        _data: &[u8],
        _private_key: &PgpKey,
        _passphrase: Option<&str>,
        _armor: bool,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }
    async fn verify(
        &self,
        _data: &[u8],
        _signature: &[u8],
        _public_key: &PgpKey,
    ) -> Result<bool> {
        unimplemented!()
    }
    async fn encrypt_and_sign(
        &self,
        _data: &[u8],
        _recipient_keys: &[PgpKey],
        _signing_key: &PgpKey,
        _passphrase: Option<&str>,
        _armor: bool,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }
    async fn decrypt_and_verify(
        &self,
        _encrypted_data: &[u8],
        _private_key: &PgpKey,
        _sender_public_key: &PgpKey,
        _passphrase: Option<&str>,
    ) -> Result<PgpDecryptResult> {
        unimplemented!()
    }
    async fn list_pgp_keys(&self) -> Result<Vec<PgpKeyInfo>> {
        unimplemented!()
    }
    async fn get_key(&self, _identifier: &str) -> Result<Option<PgpKey>> {
        unimplemented!()
    }
    async fn delete_key(&self, _identifier: &str) -> Result<()> {
        unimplemented!()
    }
    async fn change_passphrase(
        &self,
        _key: &PgpKey,
        _old_passphrase: Option<&str>,
        _new_passphrase: Option<&str>,
    ) -> Result<PgpKey> {
        unimplemented!()
    }
}

#[async_trait(?Send)]
impl TimeProvider for MockProvider {
    fn now_secs(&self) -> u64 {
        0
    }
    fn now_millis(&self) -> u64 {
        0
    }
    async fn sleep_ms(&self, _ms: u64) {}
}

impl LogProvider for MockProvider {
    fn debug(&self, _message: &str) {}
    fn info(&self, _message: &str) {}
    fn warn(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
}

#[async_trait(?Send)]
impl WalletProvider for MockProvider {
    async fn create_wallet(
        &self,
        _config: WalletConfig,
        _mnemonic: Option<String>,
        _passphrase: Option<String>,
    ) -> Result<WalletInfo> {
        unimplemented!()
    }
    async fn load_wallet(
        &self,
        _config: WalletConfig,
        _passphrase: Option<String>,
    ) -> Result<WalletInfo> {
        unimplemented!()
    }
    async fn get_balance(&self, _addresses: Option<Vec<String>>) -> Result<WalletBalance> {
        unimplemented!()
    }
    async fn get_address(&self) -> Result<String> {
        unimplemented!()
    }
    async fn get_addresses(&self, _count: u32) -> Result<Vec<AddressInfo>> {
        unimplemented!()
    }
    async fn send(&self, _params: SendParams) -> Result<String> {
        unimplemented!()
    }
    async fn get_utxos(
        &self,
        _include_frozen: bool,
        _addresses: Option<Vec<String>>,
    ) -> Result<Vec<(OutPoint, UtxoInfo)>> {
        unimplemented!()
    }
    async fn get_history(
        &self,
        _count: u32,
        _address: Option<String>,
    ) -> Result<Vec<TransactionInfo>> {
        unimplemented!()
    }
    async fn freeze_utxo(&self, _utxo: String, _reason: Option<String>) -> Result<()> {
        unimplemented!()
    }
    async fn unfreeze_utxo(&self, _utxo: String) -> Result<()> {
        unimplemented!()
    }
    async fn create_transaction(&self, _params: SendParams) -> Result<String> {
        unimplemented!()
    }
    async fn sign_transaction(&self, _tx_hex: String) -> Result<String> {
        unimplemented!()
    }
    async fn broadcast_transaction(&self, _tx_hex: String) -> Result<String> {
        unimplemented!()
    }
    async fn estimate_fee(&self, _target: u32) -> Result<FeeEstimate> {
        unimplemented!()
    }
    async fn get_fee_rates(&self) -> Result<FeeRates> {
        unimplemented!()
    }
    async fn sync(&self) -> Result<()> {
        unimplemented!()
    }
    async fn backup(&self) -> Result<String> {
        unimplemented!()
    }
    async fn get_mnemonic(&self) -> Result<Option<String>> {
        unimplemented!()
    }
    fn get_network(&self) -> Network {
        self.network
    }
    async fn get_internal_key(&self) -> Result<bitcoin::XOnlyPublicKey> {
        unimplemented!()
    }
    async fn sign_psbt(&self, _psbt: &bitcoin::psbt::Psbt) -> Result<bitcoin::psbt::Psbt> {
        unimplemented!()
    }
    async fn get_keypair(&self) -> Result<bitcoin::secp256k1::Keypair> {
        unimplemented!()
    }
    fn set_passphrase(&mut self, _passphrase: Option<String>) {
        unimplemented!()
    }
}

#[async_trait(?Send)]
impl AddressResolver for MockProvider {
    async fn resolve_all_identifiers(&self, _input: &str) -> Result<String> {
        unimplemented!()
    }
    fn contains_identifiers(&self, _input: &str) -> bool {
        unimplemented!()
    }
    async fn get_address(&self, _address_type: &str, _index: u32) -> Result<String> {
        unimplemented!()
    }
    async fn list_identifiers(&self) -> Result<Vec<String>> {
        unimplemented!()
    }
}

#[async_trait(?Send)]
impl BitcoinRpcProvider for MockProvider {
    async fn get_block_count(&self) -> Result<u64> {
        Ok(0)
    }
    async fn generate_to_address(&self, _nblocks: u32, _address: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_new_address(&self) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_transaction_hex(&self, _txid: &str) -> Result<String> {
        Ok(String::new())
    }
    async fn get_block(&self, _hash: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_block_hash(&self, _height: u64) -> Result<String> {
        Ok(String::new())
    }
    async fn send_raw_transaction(&self, _tx_hex: &str) -> Result<String> {
        Ok(String::new())
    }
    async fn get_mempool_info(&self) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn estimate_smart_fee(&self, _target: u32) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_esplora_blocks_tip_height(&self) -> Result<u64> {
        Ok(0)
    }
    async fn trace_transaction(
        &self,
        _txid: &str,
        _vout: u32,
        _block: Option<&str>,
        _tx: Option<&str>,
    ) -> Result<serde_json::Value> {
        Ok(JsonValue::Null)
    }
}

#[async_trait(?Send)]
impl MetashrewRpcProvider for MockProvider {
    async fn get_metashrew_height(&self) -> Result<u64> {
        Ok(0)
    }
    async fn get_contract_meta(&self, _block: &str, _tx: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn trace_outpoint(&self, _txid: &str, _vout: u32) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_spendables_by_address(&self, _address: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_protorunes_by_address(&self, _address: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_protorunes_by_outpoint(&self, _txid: &str, _vout: u32) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
}

#[async_trait(?Send)]
impl EsploraProvider for MockProvider {
    async fn get_blocks_tip_hash(&self) -> Result<String> {
        Ok(String::new())
    }
    async fn get_blocks_tip_height(&self) -> Result<u64> {
        Ok(0)
    }
    async fn get_blocks(&self, _start_height: Option<u64>) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_block_by_height(&self, _height: u64) -> Result<String> {
        Ok(String::new())
    }
    async fn get_block(&self, _hash: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_block_status(&self, _hash: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_block_txids(&self, _hash: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_block_header(&self, _hash: &str) -> Result<String> {
        Ok(String::new())
    }
    async fn get_block_raw(&self, _hash: &str) -> Result<String> {
        Ok(String::new())
    }
    async fn get_block_txid(&self, _hash: &str, _index: u32) -> Result<String> {
        Ok(String::new())
    }
    async fn get_block_txs(&self, _hash: &str, _start_index: Option<u32>) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_address_info(&self, _address: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_address(&self, _address: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_address_txs(&self, _address: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_address_txs_chain(
        &self,
        _address: &str,
        _last_seen_txid: Option<&str>,
    ) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_address_txs_mempool(&self, _address: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_address_utxo(&self, _address: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_address_prefix(&self, _prefix: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_tx(&self, txid: &str) -> Result<JsonValue> {
        let responses = self.responses.lock().unwrap();
        responses
            .get(txid)
            .cloned()
            .ok_or_else(|| DeezelError::JsonRpc(format!("No mock response for txid: {}", txid)))
    }
    async fn get_tx_hex(&self, _txid: &str) -> Result<String> {
        Ok(String::new())
    }
    async fn get_tx_raw(&self, _txid: &str) -> Result<String> {
        Ok(String::new())
    }
    async fn get_tx_status(&self, _txid: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_tx_merkle_proof(&self, _txid: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_tx_merkleblock_proof(&self, _txid: &str) -> Result<String> {
        Ok(String::new())
    }
    async fn get_tx_outspend(&self, _txid: &str, _index: u32) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_tx_outspends(&self, _txid: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn broadcast(&self, _tx_hex: &str) -> Result<String> {
        Ok(String::new())
    }
    async fn get_mempool(&self) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_mempool_txids(&self) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_mempool_recent(&self) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn get_fee_estimates(&self) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
}

#[async_trait(?Send)]
impl RunestoneProvider for MockProvider {
    async fn decode_runestone(&self, _tx: &Transaction) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn format_runestone_with_decoded_messages(
        &self,
        _tx: &Transaction,
    ) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
    async fn analyze_runestone(&self, _txid: &str) -> Result<JsonValue> {
        Ok(JsonValue::Null)
    }
}

#[async_trait(?Send)]
impl OrdProvider for MockProvider {
    async fn get_inscription(&self, _inscription_id: &str) -> Result<OrdInscription> {
        unimplemented!()
    }
    async fn get_inscriptions_in_block(&self, _block_hash: &str) -> Result<OrdInscriptions> {
        unimplemented!()
    }
    async fn get_ord_address_info(&self, _address: &str) -> Result<OrdAddressInfo> {
        unimplemented!()
    }
    async fn get_block_info(&self, _query: &str) -> Result<OrdBlock> {
        unimplemented!()
    }
    async fn get_ord_block_count(&self) -> Result<u64> {
        unimplemented!()
    }
    async fn get_ord_blocks(&self) -> Result<OrdBlocks> {
        unimplemented!()
    }
    async fn get_children(
        &self,
        _inscription_id: &str,
        _page: Option<u32>,
    ) -> Result<OrdChildren> {
        unimplemented!()
    }
    async fn get_content(&self, _inscription_id: &str) -> Result<Vec<u8>> {
        unimplemented!()
    }
    async fn get_inscriptions(&self, _page: Option<u32>) -> Result<OrdInscriptions> {
        unimplemented!()
    }
    async fn get_output(&self, _output: &str) -> Result<OrdOutput> {
        unimplemented!()
    }
    async fn get_parents(
        &self,
        _inscription_id: &str,
        _page: Option<u32>,
    ) -> Result<OrdParents> {
        unimplemented!()
    }
    async fn get_rune(&self, _rune: &str) -> Result<OrdRuneInfo> {
        unimplemented!()
    }
    async fn get_runes(&self, _page: Option<u32>) -> Result<OrdRunes> {
        unimplemented!()
    }
    async fn get_sat(&self, _sat: u64) -> Result<OrdSat> {
        unimplemented!()
    }
    async fn get_tx_info(&self, _txid: &str) -> Result<OrdTxInfo> {
        unimplemented!()
    }
}

#[async_trait(?Send)]
impl AlkanesProvider for MockProvider {
    async fn execute(&self, _params: EnhancedExecuteParams) -> Result<EnhancedExecuteResult> {
        unimplemented!()
    }
    async fn protorunes_by_address(&self, _address: &str) -> Result<JsonValue> {
        unimplemented!()
    }
    async fn protorunes_by_outpoint(
        &self,
        _txid: &str,
        _vout: u32,
    ) -> Result<protorune_pb::OutpointResponse> {
        unimplemented!()
    }
    async fn simulate(&self, _contract_id: &str, _params: Option<&str>) -> Result<JsonValue> {
        unimplemented!()
    }
    async fn trace(&self, _outpoint: &str) -> Result<alkanes_pb::Trace> {
        unimplemented!()
    }
    async fn get_block(&self, _height: u64) -> Result<alkanes_pb::BlockResponse> {
        unimplemented!()
    }
    async fn sequence(&self, _txid: &str, _vout: u32) -> Result<JsonValue> {
        unimplemented!()
    }
    async fn spendables_by_address(&self, _address: &str) -> Result<JsonValue> {
        unimplemented!()
    }
    async fn trace_block(&self, _height: u64) -> Result<alkanes_pb::Trace> {
        unimplemented!()
    }
    async fn get_bytecode(&self, _alkane_id: &str) -> Result<String> {
        unimplemented!()
    }
    async fn inspect(
        &self,
        _target: &str,
        _config: AlkanesInspectConfig,
    ) -> Result<AlkanesInspectResult> {
        unimplemented!()
    }
    async fn get_balance(&self, _address: Option<&str>) -> Result<Vec<AlkaneBalance>> {
        unimplemented!()
    }
}

#[async_trait(?Send)]
impl MonitorProvider for MockProvider {
    async fn monitor_blocks(&self, _start: Option<u64>) -> Result<()> {
        unimplemented!()
    }
    async fn get_block_events(&self, _height: u64) -> Result<Vec<BlockEvent>> {
        unimplemented!()
    }
}

#[async_trait(?Send)]
impl KeystoreProvider for MockProvider {
    async fn derive_addresses(
        &self,
        _master_public_key: &str,
        _network: Network,
        _script_types: &[&str],
        _start_index: u32,
        _count: u32,
    ) -> Result<Vec<KeystoreAddress>> {
        unimplemented!()
    }
    async fn get_default_addresses(
        &self,
        _master_public_key: &str,
        _network: Network,
    ) -> Result<Vec<KeystoreAddress>> {
        unimplemented!()
    }
    fn parse_address_range(&self, _range_spec: &str) -> Result<(String, u32, u32)> {
        unimplemented!()
    }
    async fn get_keystore_info(
        &self,
        _master_public_key: &str,
        _master_fingerprint: &str,
        _created_at: u64,
        _version: &str,
    ) -> Result<KeystoreInfo> {
        unimplemented!()
    }
}

#[async_trait(?Send)]
impl DeezelProvider for MockProvider {
    fn provider_name(&self) -> &str {
        "mock"
    }
    fn clone_box(&self) -> Box<dyn DeezelProvider> {
        Box::new(self.clone())
    }
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    fn secp(&self) -> &Secp256k1<All> {
        &self.secp
    }
    async fn get_utxo(&self, _outpoint: &OutPoint) -> Result<Option<TxOut>> {
        unimplemented!()
    }
    async fn sign_taproot_script_spend(
        &self,
        _sighash: bitcoin::secp256k1::Message,
    ) -> Result<schnorr::Signature> {
        unimplemented!()
    }
}