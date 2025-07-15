//! Type definitions for ZKane Frontend application

use serde::{Deserialize, Serialize};

// Local AlkaneId definition for frontend compatibility
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AlkaneId {
    pub block: u128,
    pub tx: u128,
}

impl std::fmt::Display for AlkaneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.block, self.tx)
    }
}

impl AlkaneId {
    pub fn new(block: u128, tx: u128) -> Self {
        Self { block, tx }
    }
}

use deezel_common::alkanes::AlkaneId as WasmAlkaneId;

impl From<&WasmAlkaneId> for AlkaneId {
    fn from(alkane_id: &WasmAlkaneId) -> Self {
        Self {
            block: alkane_id.block as u128,
            tx: alkane_id.tx as u128,
        }
    }
}

// JsDepositNote for frontend compatibility
#[derive(Clone, Debug)]
pub struct JsDepositNote {
    secret: String,
    nullifier: String,
    commitment: String,
    asset_id: AlkaneId,
    denomination: String,
    leaf_index: u32,
}

impl JsDepositNote {
    pub fn new(
        secret: String,
        nullifier: String,
        commitment: String,
        asset_id: AlkaneId,
        denomination: String,
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

    pub fn secret(&self) -> String { self.secret.clone() }
    pub fn nullifier(&self) -> String { self.nullifier.clone() }
    pub fn commitment(&self) -> String { self.commitment.clone() }
    pub fn asset_id(&self) -> &AlkaneId { &self.asset_id }
    pub fn denomination(&self) -> String { self.denomination.clone() }
    pub fn leaf_index(&self) -> u32 { self.leaf_index }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetBalance {
    pub asset_id: AlkaneId,
    pub symbol: String,
    pub name: String,
    pub balance: u128,
    pub decimals: u8,
    pub icon_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepositNote {
    pub secret: String,
    pub nullifier: String,
    pub commitment: String,
    pub asset_id: AlkaneId,
    pub denomination: u128,
    pub leaf_index: u32,
    pub created_at: f64, // timestamp
}

impl DepositNote {
    pub fn from_js(js_note: JsDepositNote) -> Self {
        Self {
            secret: js_note.secret(),
            nullifier: js_note.nullifier(),
            commitment: js_note.commitment(),
            asset_id: js_note.asset_id().clone(),
            denomination: js_note.denomination().parse().unwrap_or(0),
            leaf_index: js_note.leaf_index(),
            created_at: js_sys::Date::now(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawalProof {
    pub proof: String,
    pub merkle_root: String,
    pub nullifier_hash: String,
    pub outputs_hash: String,
    pub public_inputs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerklePath {
    pub root: String,
    pub elements: Vec<String>,
    pub indices: Vec<bool>,
    pub leaf_index: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: u128,
    pub script_pubkey: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolInfo {
    pub pool_id: AlkaneId,
    pub asset_id: AlkaneId,
    pub asset_symbol: String,
    pub denomination: u128,
    pub total_deposits: u64,
    pub anonymity_set: u64,
    pub created_at: f64,
    pub last_deposit: f64,
}

impl PoolInfo {
    pub fn anonymity_level(&self) -> AnonymityLevel {
        match self.anonymity_set {
            0..=9 => AnonymityLevel::VeryLow,
            10..=49 => AnonymityLevel::Low,
            50..=99 => AnonymityLevel::Medium,
            100..=499 => AnonymityLevel::High,
            _ => AnonymityLevel::VeryHigh,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnonymityLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl AnonymityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnonymityLevel::VeryLow => "Very Low",
            AnonymityLevel::Low => "Low",
            AnonymityLevel::Medium => "Medium",
            AnonymityLevel::High => "High",
            AnonymityLevel::VeryHigh => "Very High",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            AnonymityLevel::VeryLow => "anonymity-very-low",
            AnonymityLevel::Low => "anonymity-low",
            AnonymityLevel::Medium => "anonymity-medium",
            AnonymityLevel::High => "anonymity-high",
            AnonymityLevel::VeryHigh => "anonymity-very-high",
        }
    }
}

#[derive(Clone, Debug)]
pub enum DepositStatus {
    Idle,
    ValidatingAmount,
    CreatingNote,
    BuildingTransaction,
    WaitingForSignature,
    Broadcasting,
    Complete(DepositNote),
    Error(String),
}

#[derive(Clone, Debug)]
pub enum WithdrawalStatus {
    Idle,
    ParsingNote,
    ValidatingRecipient,
    FetchingMerklePath,
    GeneratingProof,
    BuildingTransaction,
    WaitingForSignature,
    Broadcasting,
    Complete(WithdrawalProof),
    Error(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
pub enum ZKaneError {
    #[error("WASM operation failed: {0}")]
    WasmError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid amount")]
    InvalidAmount,
    
    #[error("Invalid deposit note")]
    InvalidDepositNote,
    
    #[error("No asset selected")]
    NoAssetSelected,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Invalid recipient address")]
    InvalidRecipient,
    
    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub tx_hex: String,
    pub witness_data: String,
    pub fee_rate: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub txid: String,
    pub status: TransactionStatus,
    pub confirmations: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub network: String,
    pub indexer_url: String,
    pub default_fee_rate: u64,
    pub min_anonymity_set: u64,
    pub supported_assets: Vec<AlkaneId>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            indexer_url: "https://api.zkane.org".to_string(),
            default_fee_rate: 10, // sat/vB
            min_anonymity_set: 10,
            supported_assets: vec![
                AlkaneId { block: 1, tx: 1 }, // Example asset
            ],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: Theme,
    pub currency: Currency,
    pub auto_save_notes: bool,
    pub show_advanced_options: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Currency {
    BTC,
    USD,
    EUR,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::Auto,
            currency: Currency::BTC,
            auto_save_notes: true,
            show_advanced_options: false,
        }
    }
}

// Notification system
#[derive(Clone, Debug)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: f64,
    pub auto_dismiss: bool,
    pub timeout: Option<u64>, // timeout in milliseconds
}

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationType {
    Success,
    Warning,
    Error,
    Info,
}

impl Notification {
    pub fn success(title: &str, message: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            message: message.to_string(),
            notification_type: NotificationType::Success,
            timestamp: js_sys::Date::now(),
            auto_dismiss: true,
            timeout: Some(5000), // 5 seconds
        }
    }

    pub fn error(title: &str, message: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            message: message.to_string(),
            notification_type: NotificationType::Error,
            timestamp: js_sys::Date::now(),
            auto_dismiss: false,
            timeout: None, // No auto-dismiss for errors
        }
    }

    pub fn warning(title: &str, message: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            message: message.to_string(),
            notification_type: NotificationType::Warning,
            timestamp: js_sys::Date::now(),
            auto_dismiss: true,
            timeout: Some(7000), // 7 seconds
        }
    }

    pub fn info(title: &str, message: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            message: message.to_string(),
            notification_type: NotificationType::Info,
            timestamp: js_sys::Date::now(),
            auto_dismiss: true,
            timeout: Some(4000), // 4 seconds
        }
    }
}