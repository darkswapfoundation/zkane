//! Testable version of ZKane Frontend - simplified for wasm-pack testing
//! 
//! This module provides a simplified, testable version of the frontend
//! that avoids the complex service dependencies that cause wasm-bindgen issues.

use leptos::*;
use wasm_bindgen::prelude::*;

// Re-export basic types that are needed for testing
pub use crate::types::{AlkaneId, UserPreferences, Theme};

// Simple, testable components without heavy service dependencies
#[component]
pub fn SimpleCounter(
    #[prop(default = 0)] initial_value: i32,
    #[prop(default = 1)] step: i32,
) -> impl IntoView {
    let (value, set_value) = create_signal(initial_value);

    view! {
        <div class="simple-counter">
            <button on:click=move |_| set_value.set(0)>"Clear"</button>
            <button on:click=move |_| set_value.update(|v| *v -= step)>"-" {step}</button>
            <span class="counter-value">"Value: " {value}</span>
            <button on:click=move |_| set_value.update(|v| *v += step)>"+" {step}</button>
        </div>
    }
}

#[component]
pub fn SimpleThemeToggle(
    #[prop(default = Theme::Auto)] initial_theme: Theme,
) -> impl IntoView {
    let (theme, set_theme) = create_signal(initial_theme);

    let toggle_theme = move |_| {
        set_theme.update(|t| {
            *t = match *t {
                Theme::Light => Theme::Dark,
                Theme::Dark => Theme::Auto,
                Theme::Auto => Theme::Light,
            }
        });
    };

    view! {
        <button 
            class="theme-toggle"
            on:click=toggle_theme
            title=move || format!("Current theme: {:?}", theme.get())
        >
            {move || match theme.get() {
                Theme::Light => "☀️",
                Theme::Dark => "🌙",
                Theme::Auto => "🌓",
            }}
        </button>
    }
}

#[component]
pub fn SimpleAssetDisplay(
    asset_id: AlkaneId,
    #[prop(default = "Unknown".to_string())] symbol: String,
    #[prop(default = 0)] balance: u128,
) -> impl IntoView {
    view! {
        <div class="asset-display">
            <div class="asset-id">{asset_id.to_string()}</div>
            <div class="asset-symbol">{symbol}</div>
            <div class="asset-balance">{balance.to_string()}</div>
        </div>
    }
}

#[component]
pub fn SimpleNotification(
    title: String,
    message: String,
    #[prop(default = "info".to_string())] notification_type: String,
) -> impl IntoView {
    let (visible, set_visible) = create_signal(true);

    view! {
        <div 
            class=format!("notification notification-{}", notification_type)
            style:display=move || if visible.get() { "block" } else { "none" }
        >
            <div class="notification-title">{title}</div>
            <div class="notification-message">{message}</div>
            <button 
                class="notification-close"
                on:click=move |_| set_visible.set(false)
            >
                "×"
            </button>
        </div>
    }
}

#[component]
pub fn SimpleApp() -> impl IntoView {
    view! {
        <div class="simple-app">
            <header class="simple-header">
                <h1>"ZKane Privacy Pool"</h1>
                <SimpleThemeToggle/>
            </header>
            
            <main class="simple-main">
                <SimpleCounter initial_value=0 step=1/>
                
                <div class="asset-list">
                    <SimpleAssetDisplay 
                        asset_id=AlkaneId::new(1, 1)
                        symbol="ALKS".to_string()
                        balance=1000000000
                    />
                    <SimpleAssetDisplay 
                        asset_id=AlkaneId::new(2, 1)
                        symbol="TEST".to_string()
                        balance=5000000000
                    />
                </div>
                
                <SimpleNotification 
                    title="Welcome".to_string()
                    message="ZKane Privacy Pool is ready!".to_string()
                    notification_type="success".to_string()
                />
            </main>
        </div>
    }
}

// Export functions for testing
#[wasm_bindgen]
pub fn get_simple_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn simple_health_check() -> bool {
    true
}

// Simple initialization for testing
#[wasm_bindgen]
pub fn init_simple_app() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);
    
    log::info!("🚀 ZKane Simple App initialized for testing...");
    
    mount_to_body(|| {
        view! { <SimpleApp/> }
    });
}