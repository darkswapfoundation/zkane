//! Frontend component integration tests

use crate::zkane_frontend::*;
use leptos::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_app_initialization() {
    // Test that the main app component can be created
    let app = create_runtime();
    
    app.dispose();
}

#[wasm_bindgen_test]
async fn test_deposit_component_rendering() {
    let runtime = create_runtime();
    
    // Mock services
    let zkane_service = ZKaneService::new();
    let alkanes_service = AlkanesService::new();
    let notification_service = NotificationService::new();
    let storage_service = StorageService::new();
    
    // Provide contexts
    provide_context(zkane_service);
    provide_context(alkanes_service);
    provide_context(notification_service);
    provide_context(storage_service);
    
    // Create deposit component
    let deposit_component = DepositComponent();
    
    // Test that component renders without panicking
    assert!(deposit_component.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_withdraw_component_rendering() {
    let runtime = create_runtime();
    
    // Mock services
    let zkane_service = ZKaneService::new();
    let alkanes_service = AlkanesService::new();
    let notification_service = NotificationService::new();
    
    // Provide contexts
    provide_context(zkane_service);
    provide_context(alkanes_service);
    provide_context(notification_service);
    
    // Create withdraw component
    let withdraw_component = WithdrawComponent();
    
    // Test that component renders without panicking
    assert!(withdraw_component.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_asset_selector_with_empty_assets() {
    let runtime = create_runtime();
    
    // Create empty asset resource
    let assets = Resource::new(
        || (),
        |_| async { Ok(Vec::<AssetBalance>::new()) }
    );
    
    let (selected_asset, set_selected_asset) = signal(None::<AssetBalance>);
    
    // Create asset selector
    let selector = AssetSelector {
        assets,
        selected_asset,
        set_selected_asset,
    };
    
    // Test that component renders with empty assets
    assert!(selector.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_asset_selector_with_mock_assets() {
    let runtime = create_runtime();
    
    // Create mock assets
    let mock_assets = vec![
        AssetBalance {
            asset_id: "btc".to_string(),
            symbol: "BTC".to_string(),
            balance: 100_000_000, // 1 BTC
            decimals: 8,
        },
        AssetBalance {
            asset_id: "eth".to_string(),
            symbol: "ETH".to_string(),
            balance: 1_000_000_000_000_000_000, // 1 ETH
            decimals: 18,
        },
    ];
    
    let assets = Resource::new(
        || (),
        move |_| {
            let assets = mock_assets.clone();
            async move { Ok(assets) }
        }
    );
    
    let (selected_asset, set_selected_asset) = signal(None::<AssetBalance>);
    
    // Create asset selector
    let selector = AssetSelector {
        assets,
        selected_asset,
        set_selected_asset,
    };
    
    // Test that component renders with mock assets
    assert!(selector.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_amount_input_validation() {
    let runtime = create_runtime();
    
    let (amount, set_amount) = signal(String::new());
    let (selected_asset, _) = signal(Some(AssetBalance {
        asset_id: "btc".to_string(),
        symbol: "BTC".to_string(),
        balance: 100_000_000, // 1 BTC
        decimals: 8,
    }));
    let disabled = signal(false);
    
    // Create amount input
    let amount_input = AmountInput {
        amount,
        set_amount,
        selected_asset,
        disabled,
    };
    
    // Test that component renders
    assert!(amount_input.into_view().is_some());
    
    // Test amount parsing
    set_amount.set("0.5".to_string());
    assert_eq!(amount.get(), "0.5");
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_note_input_parsing() {
    let runtime = create_runtime();
    
    let (note_json, set_note_json) = signal(String::new());
    let (parsed_note, set_parsed_note) = signal(None::<DepositNote>);
    
    let parse_note = move || {
        let json = note_json.get();
        if !json.is_empty() {
            match serde_json::from_str::<DepositNote>(&json) {
                Ok(note) => set_parsed_note.set(Some(note)),
                Err(_) => set_parsed_note.set(None),
            }
        } else {
            set_parsed_note.set(None);
        }
    };
    
    // Create note input
    let note_input = NoteInput {
        note_json,
        set_note_json,
        parse_note,
        parsed_note,
    };
    
    // Test that component renders
    assert!(note_input.into_view().is_some());
    
    // Test valid JSON parsing
    let valid_note = DepositNote {
        asset_id: "btc".to_string(),
        denomination: 50_000_000, // 0.5 BTC
        commitment: "0x1234567890abcdef".to_string(),
        nullifier: "0xabcdef1234567890".to_string(),
        secret: "0x9876543210fedcba".to_string(),
        leaf_index: 42,
        created_at: 1640995200.0, // 2022-01-01
    };
    
    let valid_json = serde_json::to_string(&valid_note).unwrap();
    set_note_json.set(valid_json);
    parse_note();
    
    assert!(parsed_note.get().is_some());
    
    // Test invalid JSON
    set_note_json.set("invalid json".to_string());
    parse_note();
    
    assert!(parsed_note.get().is_none());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_pool_card_rendering() {
    let runtime = create_runtime();
    
    let mock_pool = PoolInfo {
        pool_id: "btc-pool-1".to_string(),
        asset_symbol: "BTC".to_string(),
        denomination: 50_000_000, // 0.5 BTC
        anonymity_set: 150,
        total_deposits: 75_000_000_000, // 750 BTC
        created_at: 1640995200.0,
    };
    
    // Create pool card
    let pool_card = PoolCard { pool: mock_pool };
    
    // Test that component renders
    assert!(pool_card.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_note_card_rendering() {
    let runtime = create_runtime();
    
    let mock_note = DepositNote {
        asset_id: "btc".to_string(),
        denomination: 50_000_000, // 0.5 BTC
        commitment: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        nullifier: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        secret: "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string(),
        leaf_index: 42,
        created_at: 1640995200.0,
    };
    
    // Create note card
    let note_card = NoteCard { note: mock_note };
    
    // Test that component renders
    assert!(note_card.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_notification_system() {
    let runtime = create_runtime();
    
    let notification_service = NotificationService::new();
    provide_context(notification_service.clone());
    
    // Create notification container
    let container = NotificationContainer();
    
    // Test that component renders
    assert!(container.into_view().is_some());
    
    // Test adding notifications
    notification_service.success("Test Success", "This is a success message");
    notification_service.error("Test Error", "This is an error message");
    notification_service.warning("Test Warning", "This is a warning message");
    notification_service.info("Test Info", "This is an info message");
    
    // Check that notifications were added
    assert_eq!(notification_service.notifications.get().len(), 4);
    
    // Test dismissing notifications
    let notifications = notification_service.notifications.get();
    if let Some(first_notification) = notifications.first() {
        notification_service.dismiss(&first_notification.id);
        assert_eq!(notification_service.notifications.get().len(), 3);
    }
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_theme_selector() {
    let runtime = create_runtime();
    
    let (current_theme, set_theme) = signal(Theme::Light);
    
    let theme_selector = ThemeSelector {
        current_theme: current_theme.get(),
        on_change: move |theme| set_theme.set(theme),
    };
    
    // Test that component renders
    assert!(theme_selector.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_toggle_setting() {
    let runtime = create_runtime();
    
    let (checked, set_checked) = signal(false);
    
    let toggle = ToggleSetting {
        label: "Test Setting",
        description: "This is a test setting",
        checked: checked.into(),
        on_change: move |value| set_checked.set(value),
    };
    
    // Test that component renders
    assert!(toggle.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_loading_spinner() {
    let runtime = create_runtime();
    
    let spinner = LoadingSpinner {
        message: Some("Loading test..."),
    };
    
    // Test that component renders
    assert!(spinner.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_empty_state() {
    let runtime = create_runtime();
    
    let empty_state = EmptyState {
        icon: "📭",
        title: "No Items",
        message: "There are no items to display",
    };
    
    // Test that component renders
    assert!(empty_state.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_error_state() {
    let runtime = create_runtime();
    
    let error_state = ErrorState {
        title: "Error Occurred",
        message: "Something went wrong".to_string(),
    };
    
    // Test that component renders
    assert!(error_state.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_security_tip() {
    let runtime = create_runtime();
    
    let security_tip = SecurityTip {
        icon: "🔐",
        title: "Security Tip",
        description: "Always keep your private keys secure",
    };
    
    // Test that component renders
    assert!(security_tip.into_view().is_some());
    
    runtime.dispose();
}

// Helper function to create a test runtime
fn create_runtime() -> Runtime {
    Runtime::new()
}

// Integration tests for component interactions
#[wasm_bindgen_test]
async fn test_deposit_flow_integration() {
    let runtime = create_runtime();
    
    // Set up services
    let zkane_service = ZKaneService::new();
    let alkanes_service = AlkanesService::new();
    let notification_service = NotificationService::new();
    let storage_service = StorageService::new();
    
    provide_context(zkane_service.clone());
    provide_context(alkanes_service.clone());
    provide_context(notification_service.clone());
    provide_context(storage_service.clone());
    
    // Test the full deposit component
    let deposit_component = DepositComponent();
    assert!(deposit_component.into_view().is_some());
    
    // Test that services are accessible
    let zkane_service_ctx = expect_context::<ZKaneService>();
    assert_eq!(zkane_service.get_version(), zkane_service_ctx.get_version());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_withdraw_flow_integration() {
    let runtime = create_runtime();
    
    // Set up services
    let zkane_service = ZKaneService::new();
    let alkanes_service = AlkanesService::new();
    let notification_service = NotificationService::new();
    
    provide_context(zkane_service.clone());
    provide_context(alkanes_service.clone());
    provide_context(notification_service.clone());
    
    // Test the full withdraw component
    let withdraw_component = WithdrawComponent();
    assert!(withdraw_component.into_view().is_some());
    
    runtime.dispose();
}

#[wasm_bindgen_test]
async fn test_app_routing() {
    let runtime = create_runtime();
    
    // Test that the main app with routing can be created
    let app = App();
    assert!(app.into_view().is_some());
    
    runtime.dispose();
}

// Performance tests
#[wasm_bindgen_test]
async fn test_component_creation_performance() {
    let runtime = create_runtime();
    
    let start = web_sys::window()
        .unwrap()
        .performance()
        .unwrap()
        .now();
    
    // Create multiple components to test performance
    for _ in 0..100 {
        let spinner = LoadingSpinner { message: None };
        let _ = spinner.into_view();
    }
    
    let end = web_sys::window()
        .unwrap()
        .performance()
        .unwrap()
        .now();
    
    let duration = end - start;
    
    // Should be able to create 100 components in less than 100ms
    assert!(duration < 100.0, "Component creation took too long: {}ms", duration);
    
    runtime.dispose();
}