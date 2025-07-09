use leptos::*;
use wasm_bindgen_test::*;
use zkane_frontend::components::DepositComponent;
use zkane_frontend::services::{AlkanesService, NotificationService, StorageService, WalletService, ZKaneService};
use zkane_frontend::types::UserPreferences;

wasm_bindgen_test_configure!(run_in_browser);

/// A helper function to render a component with all necessary services provided.
fn with_services<F, IV>(f: F)
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    // Create mock services
    let (user_preferences, _) = create_signal(UserPreferences::default());
    let notification_service = NotificationService::new();
    let storage_service = StorageService::new();
    let wallet_service = WalletService::new();
    let alkanes_service = AlkanesService::new();
    let zkane_service = ZKaneService::new();

    // Provide contexts
    provide_context(user_preferences);
    provide_context(notification_service);
    provide_context(storage_service);
    provide_context(wallet_service);
    provide_context(alkanes_service);
    provide_context(zkane_service);

    // Mount the component
    mount_to_body(f);
}

#[wasm_bindgen_test]
fn test_deposit_component_renders_with_services() {
    // This test ensures that the DepositComponent can render without panicking
    // when all of its required services are provided in the context.
    with_services(|| {
        view! { <DepositComponent /> }
    });

    // If we reach here without panicking, the test is considered a success.
    // We can add more assertions later to check for specific elements.
}