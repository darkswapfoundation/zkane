//! Test modules for ZKane

// Standard test modules - aligned with boiler pattern
pub mod std;
pub mod zkane_withdrawal_verification_test;
pub mod helpers;

// Legacy modules (temporarily disabled)
// pub mod crypto_tests;  // Tests are in individual crate modules
// pub mod core_tests;    // Tests are in individual crate modules
// pub mod wasm_integration;  // Temporarily disabled due to WASM dependency issues
// pub mod end_to_end_flow_tests;  // Temporarily disabled due to alkanes dependency issues
// pub mod frontend_integration_tests;  // Frontend not in workspace
// pub mod frontend_component_tests;    // Frontend not in workspace

// Re-export test utilities
pub use std::*;
pub use zkane_withdrawal_verification_test::*;
pub use helpers::*;
// pub use crypto_tests::*;
// pub use core_tests::*;
// pub use wasm_integration::*;  // Disabled
// pub use end_to_end_flow_tests::*;  // Disabled
// pub use frontend_integration_tests::*;
// pub use frontend_component_tests::*;