//! # ZKane Frontend - Full-stack Privacy Pool Application
//! 
//! This crate provides a complete browser-based privacy pool application built with Leptos,
//! integrating with alkanes and ZKane for privacy-preserving transactions.

use leptos::*;
use wasm_bindgen::prelude::*;

mod app;
mod components;
mod services;
mod types;
mod utils;
mod wasm_bindings;

// Testable version for wasm-pack testing
#[cfg(feature = "testable")]
mod lib_testable;

// Export main modules
pub use app::*;
pub use components::*;
pub use services::*;
pub use types::*;
pub use utils::*;

// Export testable components when feature is enabled
#[cfg(feature = "testable")]
pub use lib_testable::*;

// WASM entry point
#[cfg(not(test))]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);
    
    log::info!("🚀 ZKane Privacy Pool application starting...");
    
    mount_to_body(|| {
        view! { <App/> }
    });
}

// Export for testing
#[wasm_bindgen]
pub fn init_app() {
    #[cfg(not(test))]
    main();
    
    #[cfg(test)]
    {
        console_error_panic_hook::set_once();
        _ = console_log::init_with_level(log::Level::Debug);
        log::info!("🚀 ZKane Privacy Pool application initialized for testing...");
    }
}

// Version information
#[wasm_bindgen]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Health check for testing
#[wasm_bindgen]
pub fn health_check() -> bool {
    true
}