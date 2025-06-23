//! Utility components used throughout the application

use leptos::*;
use crate::types::*;

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
    view! {
        <div class="pool-card">
            <div class="pool-header">
                <h4 class="pool-title">{pool.asset_symbol}</h4>
                <span class="pool-denomination">
                    {format!("{:.8}", pool.denomination as f64 / 100_000_000.0)}
                </span>
            </div>
            
            <div class="pool-stats">
                <div class="stat">
                    <span class="stat-label">"Anonymity Set"</span>
                    <span class="stat-value">{pool.anonymity_set}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">"Total Deposits"</span>
                    <span class="stat-value">{pool.total_deposits}</span>
                </div>
                <div class="stat">
                    <span class="stat-label">"Created"</span>
                    <span class="stat-value">
                        {format_timestamp(pool.created_at)}
                    </span>
                </div>
            </div>
            
            <div class="pool-actions">
                <button class="btn btn-primary btn-sm">
                    "View Details"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn NoteCard(note: DepositNote) -> impl IntoView {
    let commitment_preview = format!("{}...", &note.commitment[..16]);
    let commitment_for_log = note.commitment.clone();
    let note_for_copy = note.clone();
    
    view! {
        <div class="note-card">
            <div class="note-header">
                <h4 class="note-title">{note.asset_id.to_string()}</h4>
                <span class="note-amount">
                    {format!("{:.8}", note.denomination as f64 / 100_000_000.0)}
                </span>
            </div>
            
            <div class="note-details">
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
                        // Navigate to withdraw page with this note
                        log::info!("Using note for withdrawal: {}", commitment_for_log);
                    }
                >
                    "Use for Withdrawal"
                </button>
                
                <button
                    class="btn btn-secondary btn-sm"
                    on:click=move |_| {
                        let note_json = serde_json::to_string_pretty(&note_for_copy).unwrap_or_default();
                        copy_to_clipboard(&note_json);
                    }
                >
                    "Copy"
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
    // In a real implementation, format the timestamp properly
    format!("{:.0} seconds ago", js_sys::Date::now() / 1000.0 - timestamp)
}

fn copy_to_clipboard(text: &str) {
    // In a real implementation, use the Clipboard API
    log::info!("Copying to clipboard: {}", text);
}