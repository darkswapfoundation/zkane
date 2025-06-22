//! ZKane Test Suite
//! 
//! Comprehensive test suite for ZKane privacy pool system following
//! the boiler pattern for alkanes contract testing.

pub mod helpers;
pub mod zkane_integration;
pub mod factory_integration;
pub mod privacy_pool_tests;
// pub mod wasm_integration; // Disabled - zkane_wasm crate not available

// Enhanced test modules following boiler and oyl-protocol patterns
pub mod comprehensive_zkane_tests;
pub mod edge_case_tests;
pub mod security_tests;
pub mod performance_tests;
pub mod end_to_end_flow_tests;

// Minimal WASM-compatible tests
pub mod zkane_wasm_minimal;

// Re-export common test utilities
pub use helpers::*;