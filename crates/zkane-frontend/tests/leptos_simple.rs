#![allow(dead_code)]

use zkane_frontend::*;
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
    // Test that we can create our services without context
    let _zkane_service = ZKaneService::new();
    let _alkanes_service = AlkanesService::new();
    let _notification_service = NotificationService::new();
    let _storage_service = StorageService::new();
    
    // If we get here, the services were created successfully
    assert!(true);
}