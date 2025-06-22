//! ZKane Test Suite
//! 
//! Comprehensive test suite for ZKane privacy pool system following
//! the boiler pattern for alkanes contract testing.

pub mod helpers;
pub mod zkane_integration;
pub mod factory_integration;
pub mod privacy_pool_tests;
pub mod wasm_integration;

// Re-export common test utilities
pub use helpers::*;