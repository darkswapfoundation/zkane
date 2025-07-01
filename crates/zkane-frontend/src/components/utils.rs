//! Utility components used throughout the application

use leptos::*;
use crate::types::*;
use crate::services::*;

#[component]
pub fn LoadingSpinner(
    #[prop(optional)] message: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="loading-spinner">
            <div class="spinner"></div>
            {message.map(|msg| view! { <span class="loading-message">{msg}</span> })}
        </div>
    }
}

#[component]
pub fn EmptyState(
    icon: &'static str,
    title: &'static str,
    message: &'static str,
) -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-icon">{icon}</div>
            <h3 class="empty-title">{title}</h3>
            <p class="empty-message">{message}</p>
        </div>
    }
}

#[component]
pub fn ErrorState(
    title: &'static str,
    message: String,
) -> impl IntoView {
    view! {
        <div class="error-state">
            <div class="error-icon">"❌"</div>
            <h3 class="error-title">{title}</h3>
            <p class="error-message">{message}</p>
        </div>
    }
}

#[component]
pub fn SecurityTip(
    icon: &'static str,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <div class="security-tip">
            <div class="tip-icon">{icon}</div>
            <div class="tip-content">
                <h4 class="tip-title">{title}</h4>
                <p class="tip-description">{description}</p>
            </div>
        </div>
    }
}

#[component]
pub fn ThemeSelector(
    current_theme: Theme,
    on_change: impl Fn(Theme) + 'static,
) -> impl IntoView {
    let theme_light = current_theme.clone();
    let theme_dark = current_theme.clone();
    let theme_auto = current_theme.clone();
    
    view! {
        <div class="theme-selector">
            <label class="form-label">"Theme"</label>
            <select
                class="form-select"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    match value.as_str() {
                        "light" => on_change(Theme::Light),
                        "dark" => on_change(Theme::Dark),
                        "auto" => on_change(Theme::Auto),
                        _ => {}
                    }
                }
            >
                <option value="light" selected=move || matches!(theme_light, Theme::Light)>
                    "Light"
                </option>
                <option value="dark" selected=move || matches!(theme_dark, Theme::Dark)>
                    "Dark"
                </option>
                <option value="auto" selected=move || matches!(theme_auto, Theme::Auto)>
                    "Auto"
                </option>
            </select>
        </div>
    }
}

#[component]
pub fn ToggleSetting(
    label: &'static str,
    description: &'static str,
    checked: Signal<bool>,
    on_change: impl Fn(bool) + 'static,
) -> impl IntoView {
    view! {
        <div class="toggle-setting">
            <div class="setting-info">
                <label class="setting-label">{label}</label>
                <p class="setting-description">{description}</p>
            </div>
            <div class="setting-control">
                <input 
                    type="checkbox"
                    class="toggle-input"
                    prop:checked=checked
                    on:change=move |ev| {
                        let checked = event_target_checked(&ev);
                        on_change(checked);
                    }
                />
                <span class="toggle-slider"></span>
            </div>
        </div>
    }
}

#[component]
pub fn PoolCard(pool: PoolInfo) -> impl IntoView {
    let asset_symbol = pool.asset_symbol.clone();
    let pool_status = if pool.anonymity_set > 10 { "Active" } else { "Building" };
    let privacy_score = calculate_privacy_score(pool.anonymity_set as u32, pool.total_deposits as u32);
    
    view! {
        <div class="pool-card enhanced-pool-card">
            <div class="pool-card-corners">
                <div class="pool-corner-left">
                    <span class="pool-type-icon" title="Privacy Pool">
                        "⊚"
                    </span>
                    <span class="pool-status-indicator" title=format!("Pool Status: {}", pool_status)>
                        {if pool.anonymity_set > 10 { "✓" } else { "⏳" }}
                    </span>
                </div>
                <div class="pool-corner-right">
                    <span class="pool-privacy-score" title=format!("Privacy Score: {}/100", privacy_score)>
                        {format!("{}%", privacy_score)}
                    </span>
                    <span class="pool-reference" title=format!("Pool ID: {}", &pool.asset_symbol)>
                        {format!("#{}", &pool.asset_symbol[..3].to_uppercase())}
                    </span>
                </div>
            </div>
            
            <div class="pool-header">
                <div class="pool-title-section">
                    <h4 class="pool-title">{asset_symbol.clone()}</h4>
                    <span class="pool-denomination">
                        {format!("{:.8} {}", pool.denomination as f64 / 100_000_000.0, asset_symbol)}
                    </span>
                </div>
            </div>
            
            <div class="pool-status-section">
                <span class="status-badge" class:status-active={pool.anonymity_set > 10} class:status-building={pool.anonymity_set <= 10}>
                    {pool_status}
                </span>
            </div>
            
            <div class="pool-details">
                <div class="detail-row">
                    <span class="detail-label">"Anonymity Set"</span>
                    <span class="detail-value">{pool.anonymity_set}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">"Total Deposits"</span>
                    <span class="detail-value">{pool.total_deposits}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">"Privacy Score"</span>
                    <span class="detail-value">
                        {format!("{}/100", privacy_score)}
                    </span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">"Created"</span>
                    <span class="detail-value">
                        {format_timestamp(pool.created_at)}
                    </span>
                </div>
            </div>
            
            <div class="pool-actions">
                <button class="btn btn-primary btn-sm" title="View detailed pool information">
                    "📊 View Details"
                </button>
                <button class="btn btn-secondary btn-sm" title="Join this privacy pool">
                    "🔒 Join Pool"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn NoteCard(
    note: DepositNote,
    #[prop(optional)] on_use_for_withdrawal: Option<Box<dyn Fn(DepositNote) + 'static>>,
    #[prop(optional)] on_delete: Option<Box<dyn Fn(String) + 'static>>,
) -> impl IntoView {
    let storage_service = expect_context::<StorageService>();
    let notification_service = expect_context::<NotificationService>();
    
    let commitment_preview = format!("{}...", &note.commitment[..16]);
    let asset_symbol = storage_service.get_asset_symbol(&note.asset_id);
    let note_for_copy = note.clone();
    let note_for_withdrawal = note.clone();
    let note_for_delete = note.clone();
    
    // Clone notification service for each closure
    let notification_service_copy = notification_service.clone();
    let notification_service_delete = notification_service.clone();
    let storage_service_delete = storage_service.clone();
    
    view! {
        <div class="note-card">
            <div class="note-card-corners">
                <div class="note-corner-left">
                    <span class="asset-icon" title=format!("{} Asset", asset_symbol)>
                        {get_asset_icon(&asset_symbol)}
                    </span>
                    <span class="note-status-indicator" title="Ready for withdrawal">
                        "✓"
                    </span>
                </div>
                <div class="note-corner-right">
                    <span class="note-timestamp" title=format!("Created {}", format_timestamp(note.created_at))>
                        {format_relative_time(note.created_at)}
                    </span>
                    <span class="note-reference" title=format!("Note ID: {}", &note.commitment[..8])>
                        {format!("#{}", &note.commitment[..6])}
                    </span>
                </div>
            </div>
            <div class="note-header">
                <h4 class="note-title">{asset_symbol.clone()}</h4>
                <span class="note-amount">
                    {format!("{:.8} {}", note.denomination as f64 / 100_000_000.0, asset_symbol)}
                </span>
            </div>
            
            <div class="note-details">
                <div class="detail">
                    <span class="detail-label">"Asset ID"</span>
                    <span class="detail-value">{note.asset_id.to_string()}</span>
                </div>
                <div class="detail">
                    <span class="detail-label">"Commitment"</span>
                    <span class="detail-value monospace">
                        {commitment_preview}
                    </span>
                </div>
                <div class="detail">
                    <span class="detail-label">"Leaf Index"</span>
                    <span class="detail-value">{note.leaf_index}</span>
                </div>
                <div class="detail">
                    <span class="detail-label">"Created"</span>
                    <span class="detail-value">
                        {format_timestamp(note.created_at)}
                    </span>
                </div>
            </div>
            
            <div class="note-actions">
                <button
                    class="btn btn-primary btn-sm"
                    on:click=move |_| {
                        if let Some(callback) = &on_use_for_withdrawal {
                            callback(note_for_withdrawal.clone());
                        } else {
                            // Default behavior: navigate to withdraw page
                            let note_json = serde_json::to_string_pretty(&note_for_withdrawal).unwrap_or_default();
                            // Store the note in session storage for the withdraw page to pick up
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.session_storage() {
                                    let _ = storage.set_item("zkane_prefill_note", &note_json);
                                }
                            }
                            
                            // Use Leptos router navigation
                            let navigate = leptos_router::use_navigate();
                            navigate("/withdraw", Default::default());
                        }
                    }
                >
                    "⬆️ Use for Withdrawal"
                </button>
                
                <button
                    class="btn btn-secondary btn-sm"
                    on:click=move |_| {
                        let note_json = serde_json::to_string_pretty(&note_for_copy).unwrap_or_default();
                        let notification_service = notification_service_copy.clone();
                        
                        copy_to_clipboard_with_feedback(&note_json, notification_service);
                    }
                >
                    "📋 Copy Note"
                </button>
                
                <button
                    class="btn btn-danger btn-sm"
                    on:click=move |_| {
                        if let Some(callback) = &on_delete {
                            callback(note_for_delete.commitment.clone());
                        } else {
                            // Default behavior: show confirmation and delete
                            if confirm_delete_note() {
                                match storage_service_delete.delete_deposit_note(&note_for_delete.commitment) {
                                    Ok(_) => {
                                        notification_service_delete.success("Deleted", "Deposit note deleted successfully");
                                        // Trigger page refresh
                                        web_sys::window().unwrap().location().reload().unwrap();
                                    },
                                    Err(e) => {
                                        notification_service_delete.error("Delete Failed", &format!("Failed to delete note: {:?}", e));
                                    }
                                }
                            }
                        }
                    }
                >
                    "🗑️ Delete"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn PoolFilters(
    filter_asset: ReadSignal<String>,
    set_filter_asset: WriteSignal<String>,
    sort_by: ReadSignal<String>,
    set_sort_by: WriteSignal<String>,
    sort_desc: ReadSignal<bool>,
    set_sort_desc: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="pool-filters">
            <div class="filter-group">
                <label class="filter-label">"Filter by Asset"</label>
                <input 
                    type="text"
                    class="form-input"
                    placeholder="Enter asset symbol..."
                    prop:value=filter_asset
                    on:input=move |ev| {
                        set_filter_asset.set(event_target_value(&ev));
                    }
                />
            </div>
            
            <div class="filter-group">
                <label class="filter-label">"Sort by"</label>
                <select 
                    class="form-select"
                    on:change=move |ev| {
                        set_sort_by.set(event_target_value(&ev));
                    }
                >
                    <option value="anonymity_set" selected=move || sort_by.get() == "anonymity_set">
                        "Anonymity Set"
                    </option>
                    <option value="total_deposits" selected=move || sort_by.get() == "total_deposits">
                        "Total Deposits"
                    </option>
                    <option value="denomination" selected=move || sort_by.get() == "denomination">
                        "Denomination"
                    </option>
                    <option value="created_at" selected=move || sort_by.get() == "created_at">
                        "Created Date"
                    </option>
                </select>
            </div>
            
            <div class="filter-group">
                <button 
                    type="button"
                    class="btn btn-secondary"
                    class:active=sort_desc
                    on:click=move |_| {
                        set_sort_desc.update(|desc| *desc = !*desc);
                    }
                >
                    {move || if sort_desc.get() { "↓ Desc" } else { "↑ Asc" }}
                </button>
            </div>
        </div>
    }
}

// Utility functions
fn format_timestamp(timestamp: f64) -> String {
    // Format timestamp without showing seconds - use relative time format
    format_relative_time(timestamp)
}

fn format_relative_time(timestamp: f64) -> String {
    let now = js_sys::Date::now() / 1000.0;
    let diff = now - timestamp;
    
    // Ensure we never show seconds - always round up to minutes
    if diff < 60.0 {
        "now".to_string()
    } else if diff < 3600.0 {
        // Convert to minutes, ensuring we never show 0 minutes for anything >= 60 seconds
        let minutes = ((diff / 60.0).floor() as u32).max(1);
        format!("{}m", minutes)
    } else if diff < 86400.0 {
        // Convert to hours, ensuring we never show 0 hours for anything >= 3600 seconds
        let hours = ((diff / 3600.0).floor() as u32).max(1);
        format!("{}h", hours)
    } else {
        // Convert to days, ensuring we never show 0 days for anything >= 86400 seconds
        let days = ((diff / 86400.0).floor() as u32).max(1);
        format!("{}d", days)
    }
}

fn get_asset_icon(asset_symbol: &str) -> &'static str {
    match asset_symbol {
        "ALKS" => "⟐",
        "BTC" => "₿",
        "TEST" => "⧬",
        "ETH" => "Ξ",
        "USDT" => "₮",
        "USDC" => "◎",
        _ => "◈"
    }
}

async fn copy_to_clipboard_async(text: &str) -> Result<(), String> {
    use wasm_bindgen_futures::JsFuture;
    
    let window = web_sys::window().ok_or("No window object")?;
    let navigator = window.navigator();
    
    // Get the clipboard API
    let clipboard = navigator.clipboard();
    let promise = clipboard.write_text(text);
    
    match JsFuture::from(promise).await {
        Ok(_) => {
            log::info!("Successfully copied to clipboard");
            Ok(())
        },
        Err(e) => {
            log::error!("Failed to copy to clipboard: {:?}", e);
            Err(format!("Clipboard write failed: {:?}", e))
        }
    }
}

fn copy_to_clipboard_with_feedback(text: &str, notification_service: NotificationService) {
    let text_to_copy = text.to_string();
    
    // Spawn the async operation
    wasm_bindgen_futures::spawn_local(async move {
        match copy_to_clipboard_async(&text_to_copy).await {
            Ok(_) => {
                notification_service.success("Copied", "Deposit note copied to clipboard");
            },
            Err(e) => {
                log::error!("Clipboard copy failed: {}", e);
                notification_service.error("Copy Failed", &format!("Failed to copy to clipboard: {}", e));
            }
        }
    });
}


fn confirm_delete_note() -> bool {
    if let Some(window) = web_sys::window() {
        window.confirm_with_message("Are you sure you want to delete this deposit note? This action cannot be undone.").unwrap_or(false)
    } else {
        false
    }
}

fn calculate_privacy_score(anonymity_set: u32, total_deposits: u32) -> u32 {
    // Calculate privacy score based on anonymity set size and total deposits
    // Higher anonymity set = better privacy
    // More total deposits = more activity and mixing
    let anonymity_score = (anonymity_set.min(100) as f32 * 0.7) as u32;
    let activity_score = ((total_deposits.min(50) as f32 / 50.0) * 30.0) as u32;
    
    (anonymity_score + activity_score).min(100)
}