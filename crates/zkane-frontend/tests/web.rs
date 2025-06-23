#![allow(dead_code)]

use zkane_frontend::*;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

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
fn test_basic_functionality() {
    // Test that WASM is working
    assert_eq!(2 + 2, 4);
    
    // Test that we can access web APIs
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    assert!(document.body().is_some());
}

#[wasm_bindgen_test]
fn test_console_logging() {
    // Test that console logging works
    web_sys::console::log_1(&"Test log message from WASM".into());
    
    // If we get here without panicking, logging works
    assert!(true);
}

#[wasm_bindgen_test]
fn test_json_serialization() {
    // Test basic JSON operations
    let test_data = serde_json::json!({
        "test": "value",
        "number": 42
    });
    
    let serialized = serde_json::to_string(&test_data).unwrap();
    assert!(serialized.contains("test"));
    assert!(serialized.contains("42"));
    
    let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized["test"], "value");
    assert_eq!(deserialized["number"], 42);
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
    let zkane_service = ZKaneService::new();
    let alkanes_service = AlkanesService::new();
    let notification_service = NotificationService::new();
    let storage_service = StorageService::new();
    
    // If we get here, the services were created successfully
    assert!(true);
}

#[wasm_bindgen_test]
fn test_notification_service() {
    let notification_service = NotificationService::new();
    
    // Test creating different types of notifications
    notification_service.success("Test Success", "This is a success message");
    notification_service.error("Test Error", "This is an error message");
    notification_service.warning("Test Warning", "This is a warning message");
    notification_service.info("Test Info", "This is an info message");
    
    // Check that notifications were created
    let notifications = notification_service.notifications.get();
    assert_eq!(notifications.len(), 4);
    
    // Test dismissing a notification
    if let Some(first_notification) = notifications.first() {
        notification_service.dismiss(&first_notification.id);
        let updated_notifications = notification_service.notifications.get();
        assert_eq!(updated_notifications.len(), 3);
    }
}
