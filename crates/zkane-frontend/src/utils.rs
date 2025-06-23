//! Utility functions for the ZKane frontend

use wasm_bindgen::prelude::*;
use web_sys::console;

/// Log a message to the browser console
pub fn log(message: &str) {
    console::log_1(&JsValue::from_str(message));
}

/// Log an error to the browser console
pub fn log_error(message: &str) {
    console::error_1(&JsValue::from_str(message));
}

/// Format a large number with commas
pub fn format_number(num: u128) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let chars: Vec<char> = num_str.chars().collect();
    
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    
    result
}

/// Truncate a hex string for display
pub fn truncate_hex(hex: &str, start_chars: usize, end_chars: usize) -> String {
    if hex.len() <= start_chars + end_chars + 3 {
        return hex.to_string();
    }
    
    format!("{}...{}", 
        &hex[..start_chars], 
        &hex[hex.len() - end_chars..]
    )
}

/// Validate hex string format
pub fn is_valid_hex(hex: &str) -> bool {
    hex.chars().all(|c| c.is_ascii_hexdigit())
}

/// Get current timestamp in milliseconds
pub fn get_timestamp() -> u64 {
    js_sys::Date::now() as u64
}

/// Format timestamp as human readable string
pub fn format_timestamp(timestamp: u64) -> String {
    let date = js_sys::Date::new(&JsValue::from_f64(timestamp as f64));
    date.to_locale_string("en-US", &JsValue::UNDEFINED).as_string().unwrap_or_default()
}

/// Generate a random ID for UI elements
pub fn generate_id() -> String {
    format!("zkane-{}", get_timestamp())
}

/// Copy text to clipboard (placeholder - would use web APIs)
pub fn copy_to_clipboard(text: &str) -> Result<(), JsValue> {
    log(&format!("Copying to clipboard: {}", text));
    // In a real implementation, this would use the Clipboard API
    Ok(())
}

/// Show browser notification (placeholder)
pub fn show_notification(title: &str, message: &str) {
    log(&format!("Notification: {} - {}", title, message));
    // In a real implementation, this would use the Notifications API
}

/// Validate Bitcoin address format (basic check)
pub fn is_valid_bitcoin_address(address: &str) -> bool {
    // Basic validation - starts with 1, 3, or bc1
    address.starts_with('1') || address.starts_with('3') || address.starts_with("bc1")
}

/// Format Bitcoin amount with proper decimal places
pub fn format_bitcoin_amount(satoshis: u64) -> String {
    let btc = satoshis as f64 / 100_000_000.0;
    format!("{:.8} BTC", btc)
}

/// Parse denomination string to u128
pub fn parse_denomination(denom_str: &str) -> Result<u128, String> {
    denom_str.parse::<u128>().map_err(|e| format!("Invalid denomination: {}", e))
}

/// Validate denomination amount
pub fn is_valid_denomination(amount: u128) -> bool {
    amount > 0 && amount <= u128::MAX
}

/// Get error message from JsValue
pub fn get_error_message(error: &JsValue) -> String {
    error.as_string().unwrap_or_else(|| "Unknown error".to_string())
}

/// Sleep for specified milliseconds (async)
pub async fn sleep(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .unwrap();
    });
    
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}

/// Debounce function calls
pub struct Debouncer {
    timeout_id: Option<i32>,
}

impl Debouncer {
    pub fn new() -> Self {
        Self { timeout_id: None }
    }
    
    pub fn debounce<F>(&mut self, func: F, delay_ms: u32) 
    where 
        F: FnOnce() + 'static 
    {
        // Clear existing timeout
        if let Some(id) = self.timeout_id {
            web_sys::window().unwrap().clear_timeout_with_handle(id);
        }
        
        // Set new timeout
        let closure = Closure::once_into_js(func);
        let id = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(), 
                delay_ms as i32
            )
            .unwrap();
            
        self.timeout_id = Some(id);
    }
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new()
    }
}

/// Local storage utilities
pub mod storage {
    use super::*;
    
    pub fn get_item(key: &str) -> Option<String> {
        web_sys::window()?
            .local_storage().ok()??
            .get_item(key).ok()?
    }
    
    pub fn set_item(key: &str, value: &str) -> Result<(), JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("No window object"))?
            .local_storage()
            .map_err(|e| JsValue::from_str(&format!("Local storage error: {:?}", e)))?
            .ok_or_else(|| JsValue::from_str("Local storage not available"))?
            .set_item(key, value)
    }
    
    pub fn remove_item(key: &str) -> Result<(), JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("No window object"))?
            .local_storage()
            .map_err(|e| JsValue::from_str(&format!("Local storage error: {:?}", e)))?
            .ok_or_else(|| JsValue::from_str("Local storage not available"))?
            .remove_item(key)
    }
}

/// URL utilities
pub mod url {
    use super::*;
    
    pub fn get_current_url() -> String {
        web_sys::window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_default()
    }
    
    pub fn get_query_param(param: &str) -> Option<String> {
        let url = web_sys::Url::new(&get_current_url()).ok()?;
        let params = url.search_params();
        params.get(param)
    }
    
    pub fn set_query_param(param: &str, value: &str) {
        if let Some(window) = web_sys::window() {
            if let Ok(url) = web_sys::Url::new(&get_current_url()) {
                url.search_params().set(param, value);
                let _ = window.history().unwrap().push_state_with_url(
                    &JsValue::NULL, 
                    "", 
                    Some(&url.href())
                );
            }
        }
    }
}

/// Theme utilities
pub mod theme {
    use super::*;
    
    pub fn get_theme() -> String {
        storage::get_item("zkane-theme").unwrap_or_else(|| "dark".to_string())
    }
    
    pub fn set_theme(theme: &str) {
        let _ = storage::set_item("zkane-theme", theme);
        apply_theme(theme);
    }
    
    pub fn apply_theme(theme: &str) {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(body) = document.body() {
                let _ = body.set_attribute("data-theme", theme);
            }
        }
    }
    
    pub fn toggle_theme() {
        let current = get_theme();
        let new_theme = if current == "dark" { "light" } else { "dark" };
        set_theme(&new_theme);
    }
}