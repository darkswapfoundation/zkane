//! Service layer for ZKane Frontend application

use crate::types::*;
use crate::wasm_bindings::*;
use leptos::*;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone)]
pub struct ZKaneService;

impl ZKaneService {
    pub fn new() -> Self {
        Self
    }

    /// Create a new deposit note using integrated WASM bindings
    pub async fn create_deposit(
        &self,
        asset_id: AlkaneId,
        amount: u128,
    ) -> Result<DepositNote, ZKaneError> {
        let amount_str = amount.to_string();
        let wasm_asset_id = WasmAlkaneId::from(&asset_id);
        
        let wasm_note = create_deposit_note(&wasm_asset_id, &amount_str)
            .map_err(|e| ZKaneError::WasmError(format!("{:?}", e)))?;
            
        let js_note = JsDepositNote::from(wasm_note);
        Ok(DepositNote::from_js(js_note))
    }

    /// Generate withdrawal proof using integrated WASM bindings
    pub async fn generate_withdrawal_proof(
        &self,
        deposit_note: &DepositNote,
        recipient_outputs: &[TxOutput],
        merkle_path: &MerklePath,
    ) -> Result<WithdrawalProof, ZKaneError> {
        // Hash the transaction outputs
        let outputs_json = serde_json::to_string(recipient_outputs)
            .map_err(|e| ZKaneError::SerializationError(e.to_string()))?;
        
        let outputs_hash = hash_transaction_outputs(&outputs_json)
            .map_err(|e| ZKaneError::WasmError(format!("{:?}", e)))?;

        // Generate the proof using integrated WASM bindings
        let merkle_path_json = serde_json::to_string(merkle_path)
            .map_err(|e| ZKaneError::SerializationError(e.to_string()))?;

        let proof_hex = generate_withdrawal_proof_placeholder(
            &deposit_note.secret,
            &deposit_note.nullifier,
            &merkle_path_json,
            &outputs_hash,
        ).map_err(|e| ZKaneError::ProofGenerationFailed(format!("{:?}", e)))?;

        // Generate nullifier hash
        let nullifier_hash = generate_nullifier_hash_from_nullifier(&deposit_note.nullifier)
            .map_err(|e| ZKaneError::WasmError(format!("{:?}", e)))?;

        Ok(WithdrawalProof {
            proof: proof_hex,
            merkle_root: merkle_path.root.clone(),
            nullifier_hash: nullifier_hash.clone(),
            outputs_hash: outputs_hash.clone(),
            public_inputs: vec![
                merkle_path.root.clone(),
                nullifier_hash.clone(),
                outputs_hash.clone(),
            ],
        })
    }

    /// Verify a deposit note is valid
    pub async fn verify_deposit_note(&self, note: &DepositNote) -> Result<bool, ZKaneError> {
        let wasm_asset_id = WasmAlkaneId::from(&note.asset_id);
        let wasm_note = WasmDepositNote::new(
            note.secret.clone(),
            note.nullifier.clone(),
            note.commitment.clone(),
            wasm_asset_id,
            note.denomination.to_string(),
            note.leaf_index,
        );

        let is_valid = verify_deposit_note_validity(&wasm_note)
            .map_err(|e| ZKaneError::WasmError(format!("{:?}", e)))?;

        Ok(is_valid)
    }

    /// Generate pool ID for asset/denomination pair
    pub fn generate_pool_id(&self, asset_id: &AlkaneId, denomination: u128) -> Result<AlkaneId, ZKaneError> {
        let wasm_asset_id = WasmAlkaneId::from(asset_id);
        let wasm_pool_id = generate_pool_id(&wasm_asset_id, &denomination.to_string())
            .map_err(|e| ZKaneError::WasmError(format!("{:?}", e)))?;

        Ok(AlkaneId::from(wasm_pool_id))
    }
}

use deezel_web::{
    wallet_provider::{BrowserWalletProvider, WalletConnector, WalletInfo},
    AlkanesProvider, DeezelError, WalletProvider,
};

#[derive(Clone)]
pub struct WalletService {
    pub connector: WalletConnector,
    pub available_wallets: RwSignal<Vec<WalletInfo>>,
    pub connected_wallet: RwSignal<Option<BrowserWalletProvider>>,
}

impl WalletService {
    pub fn new() -> Self {
        Self {
            connector: WalletConnector::new(),
            available_wallets: create_rw_signal(Vec::new()),
            connected_wallet: create_rw_signal(None),
        }
    }

    pub async fn detect_wallets(&self) {
        if let Ok(wallets) = self.connector.detect_wallets().await {
            self.available_wallets.set(wallets);
        }
    }

    pub async fn connect(&self, wallet_info: WalletInfo) -> Result<(), DeezelError> {
        // TODO: Make these URLs configurable
        let provider = BrowserWalletProvider::connect(
            wallet_info,
            "http://localhost:8332".to_string(),
            "http://localhost:8080".to_string(),
            "regtest".to_string(),
        )
        .await?;
        self.connected_wallet.set(Some(provider));
        Ok(())
    }

    pub async fn disconnect(&self) {
        if let Some(mut wallet) = self.connected_wallet.get_untracked() {
            let _ = wallet.disconnect().await;
            self.connected_wallet.set(None);
        }
    }
}

#[derive(Clone)]
pub struct AlkanesService;

impl AlkanesService {
    pub fn new() -> Self {
        Self
    }

    /// Get available alkane assets for the user
    pub async fn get_user_assets(
        &self,
        wallet_provider: &BrowserWalletProvider,
        _address: &str,
    ) -> Result<Vec<AssetBalance>, ZKaneError> {
        let balances = AlkanesProvider::get_balance(wallet_provider, None)
            .await
            .map_err(|e| ZKaneError::WasmError(e.to_string()))?;

        let mut asset_balances = Vec::new();
        for balance in balances {
            asset_balances.push(AssetBalance {
                asset_id: AlkaneId::from_str(&balance.alkane_id).unwrap_or_default(),
                symbol: balance.symbol,
                name: balance.name,
                balance: balance.balance,
                decimals: balance.decimals,
                icon_url: None,
            });
        }
        Ok(asset_balances)
    }

    /// Get privacy pools for assets
    pub async fn get_privacy_pools(
        &self,
        wallet_provider: &BrowserWalletProvider,
    ) -> Result<Vec<PoolInfo>, ZKaneError> {
        let result = wallet_provider
            .call(
                &wallet_provider.web_provider().metashrew_rpc_url(),
                "get_privacy_pools",
                serde_json::Value::Null,
                1,
            )
            .await
            .map_err(|e| ZKaneError::WasmError(e.to_string()))?;

        serde_json::from_value(result).map_err(|e| ZKaneError::SerializationError(e.to_string()))
    }

    /// Create deposit transaction
    pub async fn create_deposit_transaction(
        &self,
        wallet_provider: &BrowserWalletProvider,
        asset_id: &AlkaneId,
        amount: u128,
        pool_id: &AlkaneId,
        commitment: &str,
    ) -> Result<TransactionRequest, ZKaneError> {
        let witness_data = generate_deposit_witness(commitment)
            .map_err(|e| ZKaneError::WasmError(format!("{:?}", e)))?;

        let params = deezel_common::SendParams {
            outputs: vec![(pool_id.to_string(), amount)],
            fee_rate: 10,
            ..Default::default()
        };

        let tx_hex = wallet_provider
            .create_transaction(params)
            .await
            .map_err(|e| ZKaneError::WasmError(e.to_string()))?;

        Ok(TransactionRequest {
            tx_hex,
            witness_data,
            fee_rate: 10,
        })
    }

    /// Create withdrawal transaction
    pub async fn create_withdrawal_transaction(
        &self,
        wallet_provider: &BrowserWalletProvider,
        proof: &WithdrawalProof,
        outputs: &[TxOutput],
    ) -> Result<TransactionRequest, ZKaneError> {
        let witness_data = generate_withdrawal_witness(
            &proof.proof,
            &proof.merkle_root,
            &proof.nullifier_hash,
            &serde_json::to_string(&vec!["0xabcd"]).unwrap(), // Mock path elements
            &serde_json::to_string(&vec![false]).unwrap(), // Mock path indices
            0, // Mock leaf index
            &"0x1234".repeat(16), // Mock commitment
            &proof.outputs_hash,
        )
        .map_err(|e| ZKaneError::WasmError(format!("{:?}", e)))?;

        let send_outputs = outputs
            .iter()
            .map(|o| (o.script_pubkey.clone(), o.value))
            .collect();

        let params = deezel_common::SendParams {
            outputs: send_outputs,
            fee_rate: 10,
            ..Default::default()
        };

        let tx_hex = wallet_provider
            .create_transaction(params)
            .await
            .map_err(|e| ZKaneError::WasmError(e.to_string()))?;

        Ok(TransactionRequest {
            tx_hex,
            witness_data,
            fee_rate: 10,
        })
    }

    /// Broadcast transaction
    pub async fn broadcast_transaction(
        &self,
        wallet_provider: &BrowserWalletProvider,
        tx_request: &TransactionRequest,
    ) -> Result<TransactionResponse, ZKaneError> {
        let signed_tx = wallet_provider
            .sign_transaction(tx_request.tx_hex.clone())
            .await
            .map_err(|e| ZKaneError::WasmError(e.to_string()))?;

        let txid = wallet_provider
            .broadcast_transaction(signed_tx)
            .await
            .map_err(|e| ZKaneError::WasmError(e.to_string()))?;

        Ok(TransactionResponse {
            txid,
            status: TransactionStatus::Pending,
            confirmations: 0,
        })
    }

    /// Get transaction status
    pub async fn get_transaction_status(
        &self,
        wallet_provider: &BrowserWalletProvider,
        txid: &str,
    ) -> Result<TransactionResponse, ZKaneError> {
        let status = wallet_provider
            .get_tx_status(txid)
            .await
            .map_err(|e| ZKaneError::WasmError(e.to_string()))?;

        let confirmed = status
            .get("confirmed")
            .and_then(|c| c.as_bool())
            .unwrap_or(false);
        let block_height = status.get("block_height").and_then(|h| h.as_u64());

        let (status, confirmations) = if confirmed {
            let tip = wallet_provider.get_blocks_tip_height().await.unwrap_or(0);
            (
                TransactionStatus::Confirmed,
                tip.saturating_sub(block_height.unwrap_or(tip)) as u32,
            )
        } else {
            (TransactionStatus::Pending, 0)
        };

        Ok(TransactionResponse {
            txid: txid.to_string(),
            status,
            confirmations,
        })
    }
}

#[derive(Clone)]
pub struct NotificationService {
    pub notifications: RwSignal<Vec<Notification>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            notifications: RwSignal::new(Vec::new()),
        }
    }

    pub fn add_notification(&self, notification: Notification) {
        self.notifications.update(|notifications| {
            notifications.push(notification.clone());
            
            // Auto-dismiss if configured
            if notification.auto_dismiss {
                let notifications_signal = self.notifications;
                let notification_id = notification.id.clone();
                
                spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(5000).await;
                    notifications_signal.update(|notifications| {
                        notifications.retain(|n| n.id != notification_id);
                    });
                });
            }
        });
    }

    pub fn remove_notification(&self, id: &str) {
        self.notifications.update(|notifications| {
            notifications.retain(|n| n.id != id);
        });
    }

    pub fn clear_all(&self) {
        self.notifications.update(|notifications| {
            notifications.clear();
        });
    }

    pub fn get_notifications(&self) -> ReadSignal<Vec<Notification>> {
        self.notifications.read_only()
    }

    pub fn success(&self, title: &str, message: &str) {
        self.add_notification(Notification::success(title, message));
    }

    pub fn error(&self, title: &str, message: &str) {
        self.add_notification(Notification::error(title, message));
    }

    pub fn warning(&self, title: &str, message: &str) {
        self.add_notification(Notification::warning(title, message));
    }

    pub fn info(&self, title: &str, message: &str) {
        self.add_notification(Notification::info(title, message));
    }

    pub fn dismiss(&self, id: &str) {
        self.remove_notification(id);
    }
}

#[derive(Clone)]
pub struct StorageService;

impl StorageService {
    pub fn new() -> Self {
        Self
    }

    /// Preload test deposit notes for demonstration purposes
    pub fn preload_test_deposit_notes(&self) -> Result<(), ZKaneError> {
        // Check if test notes already exist to avoid duplicates
        let existing_notes = self.load_deposit_notes().unwrap_or_default();
        if !existing_notes.is_empty() {
            return Ok(()); // Already have notes, don't add test data
        }

        let test_notes = vec![
            DepositNote {
                secret: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
                nullifier: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
                commitment: "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string(),
                asset_id: AlkaneId { block: 1, tx: 1 }, // ALKS
                denomination: 100000000, // 1 ALKS
                leaf_index: 42,
                created_at: js_sys::Date::now() - 86400000.0 * 2.0, // 2 days ago
            },
            DepositNote {
                secret: "0x2345678901bcdef02345678901bcdef02345678901bcdef02345678901bcdef0".to_string(),
                nullifier: "0xbcdef02345678901bcdef02345678901bcdef02345678901bcdef02345678901".to_string(),
                commitment: "0x8765432109edcba98765432109edcba98765432109edcba98765432109edcba9".to_string(),
                asset_id: AlkaneId { block: 2, tx: 1 }, // TEST
                denomination: 1000000000, // 10 TEST
                leaf_index: 123,
                created_at: js_sys::Date::now() - 3600000.0 * 6.0, // 6 hours ago
            },
            DepositNote {
                secret: "0x3456789012cdef123456789012cdef123456789012cdef123456789012cdef12".to_string(),
                nullifier: "0xcdef123456789012cdef123456789012cdef123456789012cdef123456789012".to_string(),
                commitment: "0x7654321098dcba87654321098dcba87654321098dcba87654321098dcba876".to_string(),
                asset_id: AlkaneId { block: 1, tx: 1 }, // ALKS
                denomination: 1000000000, // 10 ALKS
                leaf_index: 256,
                created_at: js_sys::Date::now() - 1800000.0, // 30 minutes ago
            },
            DepositNote {
                secret: "0x4567890123def234567890123def234567890123def234567890123def2345".to_string(),
                nullifier: "0xdef234567890123def234567890123def234567890123def234567890123de".to_string(),
                commitment: "0x6543210987cba96543210987cba96543210987cba96543210987cba96543".to_string(),
                asset_id: AlkaneId { block: 3, tx: 1 }, // PRIV
                denomination: 50000000, // 0.5 PRIV
                leaf_index: 789,
                created_at: js_sys::Date::now() - 86400000.0 * 7.0, // 1 week ago
            },
            DepositNote {
                secret: "0x5678901234ef345678901234ef345678901234ef345678901234ef345678901".to_string(),
                nullifier: "0xef345678901234ef345678901234ef345678901234ef345678901234ef3456".to_string(),
                commitment: "0x5432109876ba95432109876ba95432109876ba95432109876ba95432109876".to_string(),
                asset_id: AlkaneId { block: 2, tx: 1 }, // TEST
                denomination: 500000000, // 5 TEST
                leaf_index: 1024,
                created_at: js_sys::Date::now() - 900000.0, // 15 minutes ago
            },
        ];

        // Save each test note
        for note in test_notes {
            self.save_deposit_note(&note)?;
        }

        Ok(())
    }

    /// Save deposit note to local storage
    pub fn save_deposit_note(&self, note: &DepositNote) -> Result<(), ZKaneError> {
        let storage = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| ZKaneError::WasmError("Local storage not available".to_string()))?;

        let key = format!("zkane_deposit_note_{}", note.commitment);
        let value = serde_json::to_string(note)
            .map_err(|e| ZKaneError::SerializationError(e.to_string()))?;

        storage.set_item(&key, &value)
            .map_err(|e| ZKaneError::WasmError(format!("Failed to save note: {:?}", e)))?;

        Ok(())
    }

    /// Load deposit notes from local storage
    pub fn load_deposit_notes(&self) -> Result<Vec<DepositNote>, ZKaneError> {
        let storage = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| ZKaneError::WasmError("Local storage not available".to_string()))?;

        let mut notes = Vec::new();
        let length = storage.length()
            .map_err(|e| ZKaneError::WasmError(format!("Failed to get storage length: {:?}", e)))?;

        for i in 0..length {
            if let Ok(Some(key)) = storage.key(i) {
                if key.starts_with("zkane_deposit_note_") {
                    if let Ok(Some(value)) = storage.get_item(&key) {
                        if let Ok(note) = serde_json::from_str::<DepositNote>(&value) {
                            notes.push(note);
                        }
                    }
                }
            }
        }

        // Sort notes by creation date (newest first)
        notes.sort_by(|a, b| b.created_at.partial_cmp(&a.created_at).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(notes)
    }

    /// Delete a specific deposit note from local storage
    pub fn delete_deposit_note(&self, commitment: &str) -> Result<(), ZKaneError> {
        let storage = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| ZKaneError::WasmError("Local storage not available".to_string()))?;

        let key = format!("zkane_deposit_note_{}", commitment);
        storage.remove_item(&key)
            .map_err(|e| ZKaneError::WasmError(format!("Failed to delete note: {:?}", e)))?;

        Ok(())
    }

    /// Get asset symbol for a given asset ID (helper for display)
    pub fn get_asset_symbol(&self, asset_id: &AlkaneId) -> String {
        // In a real implementation, this would query the asset registry
        // For now, return mock data based on known assets
        match (asset_id.block, asset_id.tx) {
            (1, 1) => "ALKS".to_string(),
            (2, 1) => "TEST".to_string(),
            (3, 1) => "PRIV".to_string(),
            _ => format!("{}:{}", asset_id.block, asset_id.tx),
        }
    }

    /// Save user preferences
    pub fn save_preferences(&self, preferences: &UserPreferences) -> Result<(), ZKaneError> {
        let storage = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| ZKaneError::WasmError("Local storage not available".to_string()))?;

        let value = serde_json::to_string(preferences)
            .map_err(|e| ZKaneError::SerializationError(e.to_string()))?;

        storage.set_item("zkane_preferences", &value)
            .map_err(|e| ZKaneError::WasmError(format!("Failed to save preferences: {:?}", e)))?;

        Ok(())
    }

    /// Load user preferences
    pub fn load_preferences(&self) -> Result<UserPreferences, ZKaneError> {
        let storage = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| ZKaneError::WasmError("Local storage not available".to_string()))?;

        match storage.get_item("zkane_preferences") {
            Ok(Some(value)) => {
                serde_json::from_str(&value)
                    .map_err(|e| ZKaneError::SerializationError(e.to_string()))
            },
            _ => Ok(UserPreferences::default()),
        }
    }
}