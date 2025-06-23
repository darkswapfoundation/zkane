#![allow(dead_code)]

use wasm_bindgen_test::*;

// Import the basic functions we can test
use zkane_frontend::{health_check, get_app_version};

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