//! History component for deposit note management

use leptos::*;
use leptos_router::*;
use wasm_bindgen_futures::spawn_local;
use crate::types::*;
use crate::services::*;
use crate::components::utils::*;

#[component]
pub fn HistoryComponent() -> impl IntoView {
    let storage_service = expect_context::<StorageService>();
    let notification_service = expect_context::<NotificationService>();
    
    // State for managing notes
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);
    
    // Preload test deposit notes on component mount
    let storage_service_for_preload = storage_service.clone();
    let set_refresh_trigger_for_preload = set_refresh_trigger;
    create_effect(move |_| {
        let storage_service = storage_service_for_preload.clone();
        spawn_local(async move {
            if let Err(e) = storage_service.preload_test_deposit_notes() {
                log::warn!("Failed to preload test deposit notes: {:?}", e);
            } else {
                // Trigger refresh after preloading
                set_refresh_trigger_for_preload.update(|n| *n += 1);
            }
        });
    });
    
    // Load saved deposit notes with refresh capability
    let saved_notes = Resource::new(
        move || refresh_trigger.get(),
        move |_| {
            let storage_service = storage_service.clone();
            async move {
                storage_service.load_deposit_notes()
            }
        },
    );

    view! {
        <div class="history-component">
            <div class="history-header">
                <h3>"Saved Deposit Notes"</h3>
                <div class="history-actions">
                    <button 
                        class="btn btn-secondary"
                        on:click=move |_| {
                            set_refresh_trigger.update(|n| *n += 1);
                            notification_service.info("Refreshed", "Deposit notes refreshed");
                        }
                    >
                        "🔄 Refresh"
                    </button>
                </div>
            </div>
            
            <Suspense fallback=|| view! { <LoadingSpinner message="Loading history..."/> }>
                {move || {
                    saved_notes.get().map(|result| -> leptos::View {
                        match result {
                            Ok(notes) => {
                                if notes.is_empty() {
                                    view! {
                                        <EmptyState
                                            icon="📜"
                                            title="No Deposit Notes"
                                            message="You haven't created any deposit notes yet. Create your first deposit to get started!"
                                        />
                                    }.into_view()
                                } else {
                                    view! {
                                        <div class="notes-list">
                                            <div class="notes-summary">
                                                <p class="summary-text">
                                                    {format!("You have {} saved deposit note{}", 
                                                        notes.len(), 
                                                        if notes.len() == 1 { "" } else { "s" }
                                                    )}
                                                </p>
                                            </div>
                                            
                                            <div class="notes-grid">
                                                {notes.into_iter().map(|note| {
                                                    view! {
                                                        <NoteCard note=note/>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    }.into_view()
                                }
                            },
                            Err(e) => view! {
                                <ErrorState
                                    title="Failed to Load History"
                                    message=format!("Error loading deposit notes: {:?}", e)
                                />
                            }.into_view()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn EnhancedNoteCard(
    note: DepositNote,
    on_use_for_withdrawal: impl Fn(DepositNote) + 'static + Clone,
    on_delete: impl Fn(String) + 'static + Clone,
) -> impl IntoView {
    let storage_service = expect_context::<StorageService>();
    let notification_service = expect_context::<NotificationService>();
    
    let commitment_preview = format!("{}...", &note.commitment[..16]);
    let asset_symbol = storage_service.get_asset_symbol(&note.asset_id);
    let note_for_copy = note.clone();
    let note_for_withdrawal = note.clone();
    let note_for_delete = note.clone();
    
    view! {
        <div class="enhanced-note-card">
            <div class="note-header">
                <div class="note-title-section">
                    <h4 class="note-title">{asset_symbol.clone()}</h4>
                    <span class="note-amount">
                        {format!("{:.8} {}", note.denomination as f64 / 100_000_000.0, asset_symbol)}
                    </span>
                </div>
                <div class="note-status">
                    <span class="status-badge status-active">"Active"</span>
                </div>
            </div>
            
            <div class="note-details">
                <div class="detail-row">
                    <span class="detail-label">"Asset ID"</span>
                    <span class="detail-value">{note.asset_id.to_string()}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">"Commitment"</span>
                    <span class="detail-value monospace">
                        {commitment_preview}
                    </span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">"Leaf Index"</span>
                    <span class="detail-value">{note.leaf_index}</span>
                </div>
                <div class="detail-row">
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
                        on_use_for_withdrawal(note_for_withdrawal.clone());
                    }
                    title="Load this note for withdrawal"
                >
                    "⬆ Use for Withdrawal"
                </button>
                
                <button
                    class="btn btn-secondary btn-sm"
                    on:click=move |_| {
                        let note_json = serde_json::to_string_pretty(&note_for_copy).unwrap_or_default();
                        copy_to_clipboard(&note_json);
                        notification_service.success("Copied", "Deposit note copied to clipboard");
                    }
                    title="Copy note JSON to clipboard"
                >
                    "📋 Copy Note"
                </button>
                
                <button
                    class="btn btn-danger btn-sm"
                    on:click=move |_| {
                        on_delete(note_for_delete.commitment.clone());
                    }
                    title="Delete this deposit note"
                >
                    "🗑 Delete"
                </button>
            </div>
        </div>
    }
}

// Utility functions
fn format_timestamp(timestamp: f64) -> String {
    let now = js_sys::Date::now();
    let diff = (now - timestamp) / 1000.0; // Convert to seconds
    
    if diff < 60.0 {
        "Just now".to_string()
    } else if diff < 3600.0 {
        format!("{:.0} minutes ago", diff / 60.0)
    } else if diff < 86400.0 {
        format!("{:.0} hours ago", diff / 3600.0)
    } else {
        format!("{:.0} days ago", diff / 86400.0)
    }
}

fn copy_to_clipboard(text: &str) {
    // For now, just log the action. In a real implementation, use the Clipboard API
    log::info!("Copying to clipboard: {}", text);
    // TODO: Implement proper clipboard functionality with web_sys
}