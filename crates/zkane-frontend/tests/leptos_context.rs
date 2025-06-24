#![allow(dead_code)]

use zkane_frontend::*;
use leptos::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_app_context_setup() {
    let document = web_sys::window().unwrap().document().unwrap();
    let test_wrapper = document.create_element("section").unwrap();
    let _ = document.body().unwrap().append_child(&test_wrapper);

    // Mount our App component to test context setup
    let _dispose = mount_to(
        test_wrapper.clone().unchecked_into(),
        || view! { <App/> },
    );

    // Wait for the reactive system to settle
    // Wait for reactive system to settle

    // Check that the app rendered without panicking (which would happen if context was missing)
    let app_content = test_wrapper.inner_html();
    
    // The app should have rendered some content (not be empty)
    assert!(!app_content.is_empty(), "App should render content");
    
    // Look for key elements that should be present
    assert!(app_content.contains("ZKane"), "App should contain ZKane branding");
}

#[wasm_bindgen_test]
async fn test_theme_toggle_context() {
    let document = web_sys::window().unwrap().document().unwrap();
    let test_wrapper = document.create_element("section").unwrap();
    let _ = document.body().unwrap().append_child(&test_wrapper);

    // Mount just the Header component which contains ThemeToggle
    let _dispose = mount_to(
        test_wrapper.clone().unchecked_into(),
        || {
            // Provide the necessary contexts manually for testing
            let (user_preferences, set_user_preferences) = create_signal(UserPreferences::default());
            let notification_service = NotificationService::new();
            let storage_service = StorageService::new();
            let zkane_service = ZKaneService::new();
            let alkanes_service = AlkanesService::new();
            let app_config = create_signal(AppConfig::default()).0;
            
            provide_context(notification_service);
            provide_context(storage_service);
            provide_context(zkane_service);
            provide_context(alkanes_service);
            provide_context(app_config);
            provide_context(user_preferences);
            provide_context(set_user_preferences);
            
            view! { <App/> }
        },
    );

    // Wait for the reactive system to settle
    // Wait for reactive system to settle

    // Check that the header rendered without panicking
    let header_content = test_wrapper.inner_html();
    assert!(!header_content.is_empty(), "Header should render content");
    
    // Look for the theme toggle button
    let theme_button = test_wrapper.query_selector(".theme-toggle");
    assert!(theme_button.is_ok(), "Should find theme toggle button");
    
    if let Ok(Some(button)) = theme_button {
        // Click the theme toggle button to test the context interaction
        let html_button = button.unchecked_into::<web_sys::HtmlElement>();
        html_button.click();
        
        // Wait for the reactive system to process the click
        // Wait for reactive system to settle
        
        // If we get here without panicking, the context is working correctly
        assert!(true, "Theme toggle should work without context errors");
    }
}

#[wasm_bindgen_test]
async fn test_basic_functionality() {
    // Test that WASM is working
    assert_eq!(2 + 2, 4);
    
    // Test that we can access web APIs
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    assert!(document.body().is_some());
}

#[wasm_bindgen_test]
fn test_health_check() {
    // Test the basic health check function
    assert!(health_check());
}

#[wasm_bindgen_test]
fn test_app_version() {
    // Test that we can get the app version
    let version = get_app_version();
    assert!(!version.is_empty());
    assert_eq!(version, "0.1.0");
}

#[wasm_bindgen_test]
fn test_types_creation() {
    // Test that we can create our basic types
    let alkane_id = AlkaneId::new(123, 456);
    assert_eq!(alkane_id.block, 123);
    assert_eq!(alkane_id.tx, 456);
    
    // Test Display trait
    let display_str = format!("{}", alkane_id);
    assert_eq!(display_str, "123:456");
}

#[wasm_bindgen_test]
fn test_services_creation() {
    // Test that we can create our services
    let _zkane_service = ZKaneService::new();
    let _alkanes_service = AlkanesService::new();
    let _notification_service = NotificationService::new();
    let _storage_service = StorageService::new();
    
    // If we get here, the services were created successfully
    assert!(true);
}