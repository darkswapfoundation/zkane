//! Deposit component and related UI elements

use leptos::*;
use crate::types::*;
use crate::services::*;

#[component]
pub fn AssetSelector(
    assets: Resource<(), Result<Vec<AssetBalance>, ZKaneError>>,
    selected_asset: ReadSignal<Option<AssetBalance>>,
    set_selected_asset: WriteSignal<Option<AssetBalance>>,
) -> impl IntoView {
    view! {
        <div class="asset-selector">
            <label class="form-label">"Select Asset"</label>
            <Suspense fallback=|| view! { <div class="loading">"Loading assets..."</div> }>
                {move || {
                    assets.get().map(|result| {
                        match result {
                            Ok(assets) => {
                                if assets.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <p>"No assets available"</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    let assets_for_change = assets.clone();
                                    view! {
                                        <select
                                            class="form-select"
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev);
                                                if value.is_empty() {
                                                    set_selected_asset.set(None);
                                                } else {
                                                    if let Some(asset) = assets_for_change.iter().find(|a| a.asset_id.to_string() == value) {
                                                        set_selected_asset.set(Some(asset.clone()));
                                                    }
                                                }
                                            }
                                        >
                                            <option value="">"Select an asset..."</option>
                                            {assets.into_iter().map(|asset| {
                                                view! {
                                                    <option value=asset.asset_id.to_string()>
                                                        {format!("{} ({:.8})", asset.symbol, asset.balance as f64 / 10f64.powi(asset.decimals as i32))}
                                                    </option>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    }.into_any()
                                }
                            },
                            Err(e) => view! {
                                <div class="error-state">
                                    <p>"Failed to load assets: " {format!("{:?}", e)}</p>
                                </div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
            
            {move || {
                selected_asset.get().map(|asset| {
                    view! {
                        <div class="asset-info">
                            <div class="asset-details">
                                <span class="asset-symbol">{asset.symbol}</span>
                                <span class="asset-balance">
                                    "Balance: " {format!("{:.8}", asset.balance as f64 / 10f64.powi(asset.decimals as i32))}
                                </span>
                            </div>
                        </div>
                    }
                })
            }}
        </div>
    }
}

#[component]
pub fn AmountInput(
    amount: ReadSignal<String>,
    set_amount: WriteSignal<String>,
    selected_asset: ReadSignal<Option<AssetBalance>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="amount-input">
            <label class="form-label">"Amount"</label>
            <div class="input-group">
                <input 
                    type="text"
                    class="form-input"
                    placeholder="0.00000000"
                    prop:value=amount
                    prop:disabled=disabled
                    on:input=move |ev| {
                        set_amount.set(event_target_value(&ev));
                    }
                />
                {move || {
                    selected_asset.get().map(|asset| {
                        view! {
                            <span class="input-addon">{asset.symbol}</span>
                        }
                    })
                }}
            </div>
            
            {move || {
                selected_asset.get().map(|asset| {
                    let max_amount = asset.balance as f64 / 10f64.powi(asset.decimals as i32);
                    view! {
                        <div class="amount-helpers">
                            <button 
                                type="button"
                                class="btn btn-link btn-sm"
                                prop:disabled=disabled
                                on:click=move |_| {
                                    set_amount.set(format!("{:.8}", max_amount));
                                }
                            >
                                "Max"
                            </button>
                            <span class="max-amount">
                                "Max: " {format!("{:.8}", max_amount)}
                            </span>
                        </div>
                    }
                })
            }}
        </div>
    }
}

#[component]
pub fn DepositActions(
    deposit_action: Action<(), ()>,
    deposit_status: ReadSignal<DepositStatus>,
    selected_asset: ReadSignal<Option<AssetBalance>>,
    amount: ReadSignal<String>,
) -> impl IntoView {
    let can_deposit = move || {
        selected_asset.get().is_some() && 
        !amount.get().is_empty() && 
        matches!(deposit_status.get(), DepositStatus::Idle)
    };

    view! {
        <div class="deposit-actions">
            <button 
                type="button"
                class="btn btn-primary btn-lg"
                prop:disabled=move || !can_deposit()
                on:click=move |_| {
                    deposit_action.dispatch(());
                }
            >
                {move || {
                    match deposit_status.get() {
                        DepositStatus::Idle => "Create Deposit Note",
                        DepositStatus::ValidatingAmount => "Validating...",
                        DepositStatus::CreatingNote => "Creating Note...",
                        DepositStatus::BuildingTransaction => "Building Transaction...",
                        DepositStatus::WaitingForSignature => "Waiting for Signature...",
                        DepositStatus::Broadcasting => "Broadcasting...",
                        DepositStatus::Complete(_) => "Note Created",
                        DepositStatus::Error(_) => "Try Again",
                    }
                }}
            </button>
            
            {move || {
                match deposit_status.get() {
                    DepositStatus::ValidatingAmount | DepositStatus::CreatingNote => {
                        Some(view! {
                            <div class="progress-indicator">
                                <div class="spinner"></div>
                                <span>
                                    {match deposit_status.get() {
                                        DepositStatus::ValidatingAmount => "Validating amount...",
                                        DepositStatus::CreatingNote => "Creating deposit note...",
                                        _ => ""
                                    }}
                                </span>
                            </div>
                        })
                    },
                    _ => None
                }
            }}
        </div>
    }
}

#[component]
pub fn DepositResult(
    status: ReadSignal<DepositStatus>,
    created_note: ReadSignal<Option<DepositNote>>,
    storage_service: StorageService,
) -> impl IntoView {
    view! {
        <div class="deposit-result">
            {move || {
                match status.get() {
                    DepositStatus::Complete(note) => {
                        let note_clone1 = note.clone();
                        let note_clone2 = note.clone();
                        let note_clone3 = note.clone();
                        Some(view! {
                            <div class="success-result">
                                <div class="success-header">
                                    <span class="success-icon">"✅"</span>
                                    <h4>"Deposit Note Created Successfully"</h4>
                                </div>
                                
                                <div class="note-display">
                                    <label class="note-label">"Your Deposit Note (Save This Securely!):"</label>
                                    <textarea
                                        class="note-textarea"
                                        readonly
                                        prop:value=move || {
                                            serde_json::to_string_pretty(&note_clone1).unwrap_or_default()
                                        }
                                    ></textarea>
                                </div>
                                
                                <div class="note-actions">
                                    <button
                                        type="button"
                                        class="btn btn-secondary"
                                        on:click=move |_| {
                                            let note_json = serde_json::to_string_pretty(&note_clone2).unwrap_or_default();
                                            copy_to_clipboard(&note_json);
                                        }
                                    >
                                        "Copy to Clipboard"
                                    </button>
                                    
                                    <button
                                        type="button"
                                        class="btn btn-secondary"
                                        on:click=move |_| {
                                            let note_json = serde_json::to_string_pretty(&note_clone3).unwrap_or_default();
                                            download_as_file(&note_json, &format!("zkane-deposit-{}.json", note_clone3.commitment));
                                        }
                                    >
                                        "Download as File"
                                    </button>
                                </div>
                                
                                <div class="security-warning">
                                    <span class="warning-icon">"⚠️"</span>
                                    <div class="warning-text">
                                        <strong>"Important Security Notice:"</strong>
                                        <ul>
                                            <li>"This note contains the secret needed to withdraw your funds"</li>
                                            <li>"Store it securely and never share it with anyone"</li>
                                            <li>"If you lose this note, your funds cannot be recovered"</li>
                                        </ul>
                                    </div>
                                </div>
                            </div>
                        })
                    },
                    DepositStatus::Error(error) => {
                        Some(view! {
                            <div class="error-result">
                                <div class="error-header">
                                    <span class="error-icon">"❌"</span>
                                    <h4>"Deposit Failed"</h4>
                                </div>
                                <p class="error-message">{error}</p>
                            </div>
                        })
                    },
                    _ => None
                }
            }}
        </div>
    }
}

// Utility functions
fn copy_to_clipboard(text: &str) {
    // In a real implementation, use the Clipboard API
    log::info!("Copying to clipboard: {}", text);
}

fn download_as_file(content: &str, filename: &str) {
    // In a real implementation, create a download link
    log::info!("Downloading file: {} with content: {}", filename, content);
}