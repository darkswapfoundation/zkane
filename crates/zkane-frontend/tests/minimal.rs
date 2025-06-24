#![allow(dead_code)]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_basic_wasm() {
    // Test that WASM is working
    assert_eq!(2 + 2, 4);
}

#[wasm_bindgen_test]
fn test_web_apis() {
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