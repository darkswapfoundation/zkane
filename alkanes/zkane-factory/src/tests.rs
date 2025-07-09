use super::*;
use wasm_bindgen_test::*;
use alkanes_runtime::test_utils::MockContext;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_initialize_factory() {
    // 🎯 ZKANE CHADSON: Initialize mock context for wasm-bindgen-test
    // This provides the necessary host environment for the contract to execute.
    let mut context = MockContext::new();
    context.setup();

    let factory = ZKaneFactory::default();
    let result = factory.initialize();
    
    // Teardown the mock context
    context.teardown();

    assert!(result.is_ok());
}