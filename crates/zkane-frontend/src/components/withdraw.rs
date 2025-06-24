//! Withdraw component and related UI elements

use leptos::*;
use wasm_bindgen::JsCast;
use gloo_file::callbacks::read_as_text;
use crate::types::*;

#[component]
pub fn NoteInput(
    note_json: ReadSignal<String>,
    set_note_json: WriteSignal<String>,
    parse_note: impl Fn() + 'static + Clone,
    parsed_note: ReadSignal<Option<DepositNote>>,
) -> impl IntoView {
    view! {
        <div class="note-input">
            <label class="form-label">"Deposit Note"</label>
            <div class="note-input-group">
                <textarea 
                    class="form-textarea"
                    placeholder="Paste your deposit note JSON here..."
                    rows="8"
                    prop:value=note_json
                    on:input={
                        let parse_note = parse_note.clone();
                        move |ev| {
                            set_note_json.set(event_target_value(&ev));
                            parse_note();
                        }
                    }
                ></textarea>
                
                <div class="note-actions">
                    <input 
                        type="file"
                        accept=".json"
                        style="display: none"
                        id="note-file-input"
                        on:change=move |ev| {
                            // Handle file upload
                            if let Some(file) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                .and_then(|input| input.files())
                                .and_then(|files| files.get(0)) {
                                
                                let set_note_json = set_note_json.clone();
                                let parse_note = parse_note.clone();
                                
                                let file_blob = gloo_file::Blob::from(file);
                                read_as_text(&file_blob, move |result| {
                                    if let Ok(text) = result {
                                        set_note_json.set(text);
                                        parse_note();
                                    }
                                });
                            }
                        }
                    />
                    
                    <button 
                        type="button"
                        class="btn btn-secondary btn-sm"
                        on:click=move |_| {
                            if let Some(input) = document().get_element_by_id("note-file-input") {
                                input.dyn_into::<web_sys::HtmlElement>().unwrap().click();
                            }
                        }
                    >
                        "Load from File"
                    </button>
                    
                    <button 
                        type="button"
                        class="btn btn-secondary btn-sm"
                        prop:disabled=move || note_json.get().is_empty()
                        on:click=move |_| {
                            set_note_json.set(String::new());
                        }
                    >
                        "Clear"
                    </button>
                </div>
            </div>
            
            {move || {
                match parsed_note.get() {
                    Some(note) => {
                        view! {
                            <div class="note-preview">
                                <div class="note-status success">
                                    <span class="status-icon">"✅"</span>
                                    <span>"Valid deposit note loaded"</span>
                                </div>
                                <div class="note-details">
                                    <div class="detail-row">
                                        <span class="detail-label">"Asset:"</span>
                                        <span class="detail-value">{note.asset_id.to_string()}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="detail-label">"Amount:"</span>
                                        <span class="detail-value">{format!("{:.8}", note.denomination as f64 / 100_000_000.0)}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="detail-label">"Commitment:"</span>
                                        <span class="detail-value commitment">{format!("{}...", &note.commitment[..16])}</span>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    },
                    None => {
                        if !note_json.get().is_empty() {
                            view! {
                                <div class="note-status error">
                                    <span class="status-icon">"❌"</span>
                                    <span>"Invalid deposit note format"</span>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }
                }
            }}
        </div>
    }
}

#[component]
pub fn RecipientInput(
    recipient: ReadSignal<String>,
    set_recipient: WriteSignal<String>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let is_valid_address = move || {
        let addr = recipient.get();
        !addr.is_empty() && validate_bitcoin_address(&addr)
    };

    view! {
        <div class="recipient-input">
            <label class="form-label">"Recipient Address"</label>
            <div class="input-group">
                <input 
                    type="text"
                    class="form-input"
                    class:valid=is_valid_address
                    class:invalid=move || !recipient.get().is_empty() && !is_valid_address()
                    placeholder="Enter Bitcoin address..."
                    prop:value=recipient
                    prop:disabled=disabled
                    on:input=move |ev| {
                        set_recipient.set(event_target_value(&ev));
                    }
                />
                
                {move || {
                    if !recipient.get().is_empty() {
                        if is_valid_address() {
                            Some(view! {
                                <span class="input-status success">"✅"</span>
                            })
                        } else {
                            Some(view! {
                                <span class="input-status error">"❌"</span>
                            })
                        }
                    } else {
                        None
                    }
                }}
            </div>
            
            <div class="input-help">
                <small class="help-text">
                    "Enter a valid Bitcoin address where you want to receive the withdrawn funds"
                </small>
            </div>
        </div>
    }
}

#[component]
pub fn WithdrawActions(
    withdraw_action: Action<(), ()>,
    withdrawal_status: ReadSignal<WithdrawalStatus>,
    parsed_note: ReadSignal<Option<DepositNote>>,
    recipient: ReadSignal<String>,
) -> impl IntoView {
    let can_withdraw = move || {
        parsed_note.get().is_some() && 
        !recipient.get().is_empty() && 
        validate_bitcoin_address(&recipient.get()) &&
        matches!(withdrawal_status.get(), WithdrawalStatus::Idle)
    };

    view! {
        <div class="withdraw-actions">
            <button 
                type="button"
                class="btn btn-primary btn-lg"
                prop:disabled=move || !can_withdraw()
                on:click=move |_| {
                    withdraw_action.dispatch(());
                }
            >
                {move || {
                    match withdrawal_status.get() {
                        WithdrawalStatus::Idle => "Generate Withdrawal Proof",
                        WithdrawalStatus::ParsingNote => "Parsing Note...",
                        WithdrawalStatus::ValidatingRecipient => "Validating Address...",
                        WithdrawalStatus::FetchingMerklePath => "Fetching Merkle Path...",
                        WithdrawalStatus::GeneratingProof => "Generating Proof...",
                        WithdrawalStatus::BuildingTransaction => "Building Transaction...",
                        WithdrawalStatus::WaitingForSignature => "Waiting for Signature...",
                        WithdrawalStatus::Broadcasting => "Broadcasting...",
                        WithdrawalStatus::Complete(_) => "Proof Generated",
                        WithdrawalStatus::Error(_) => "Try Again",
                    }
                }}
            </button>
            
            {move || {
                match withdrawal_status.get() {
                    WithdrawalStatus::ParsingNote | 
                    WithdrawalStatus::ValidatingRecipient | 
                    WithdrawalStatus::GeneratingProof => {
                        Some(view! {
                            <div class="progress-indicator">
                                <div class="spinner"></div>
                                <span>
                                    {match withdrawal_status.get() {
                                        WithdrawalStatus::ParsingNote => "Parsing deposit note...",
                                        WithdrawalStatus::ValidatingRecipient => "Validating recipient address...",
                                        WithdrawalStatus::GeneratingProof => "Generating zero-knowledge proof...",
                                        _ => ""
                                    }}
                                </span>
                            </div>
                        })
                    },
                    _ => None
                }
            }}
            
            {move || {
                if matches!(withdrawal_status.get(), WithdrawalStatus::GeneratingProof) {
                    Some(view! {
                        <div class="proof-progress">
                            <div class="progress-bar">
                                <div class="progress-fill"></div>
                            </div>
                            <p class="progress-text">
                                "This may take a few minutes. Please do not close this tab."
                            </p>
                        </div>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}

#[component]
pub fn WithdrawResult(
    status: ReadSignal<WithdrawalStatus>,
    generated_proof: ReadSignal<Option<WithdrawalProof>>,
) -> impl IntoView {
    view! {
        <div class="withdraw-result">
            {move || {
                match status.get() {
                    WithdrawalStatus::Complete(proof) => {
                        let proof_clone1 = proof.clone();
                        let proof_clone2 = proof.clone();
                        let proof_clone3 = proof.clone();
                        let nullifier_hash_preview = format!("{}...", &proof.nullifier_hash[..16]);
                        let merkle_root_preview = format!("{}...", &proof.merkle_root[..16]);
                        let proof_len = proof.proof.len();
                        
                        Some(view! {
                            <div class="success-result">
                                <div class="success-header">
                                    <span class="success-icon">"✅"</span>
                                    <h4>"Withdrawal Proof Generated Successfully"</h4>
                                </div>
                                
                                <div class="proof-display">
                                    <label>"Your Withdrawal Proof:"</label>
                                    <textarea
                                        class="proof-textarea"
                                        readonly
                                        prop:value=move || {
                                            serde_json::to_string_pretty(&proof_clone1).unwrap_or_default()
                                        }
                                    ></textarea>
                                </div>
                                
                                <div class="proof-actions">
                                    <button
                                        type="button"
                                        class="btn btn-secondary"
                                        on:click=move |_| {
                                            let proof_json = serde_json::to_string_pretty(&proof_clone2).unwrap_or_default();
                                            copy_to_clipboard(&proof_json);
                                        }
                                    >
                                        "Copy Proof"
                                    </button>
                                    
                                    <button
                                        type="button"
                                        class="btn btn-secondary"
                                        on:click=move |_| {
                                            let proof_json = serde_json::to_string_pretty(&proof_clone3).unwrap_or_default();
                                            download_as_file(&proof_json, &format!("zkane-withdrawal-proof-{}.json", proof_clone3.nullifier_hash));
                                        }
                                    >
                                        "Download Proof"
                                    </button>
                                    
                                    <button
                                        type="button"
                                        class="btn btn-primary"
                                        on:click=move |_| {
                                            // In a real implementation, this would submit the transaction
                                            log::info!("Submitting withdrawal transaction with proof");
                                        }
                                    >
                                        "Submit Transaction"
                                    </button>
                                </div>
                                
                                <div class="proof-details">
                                    <h5>"Proof Details:"</h5>
                                    <div class="detail-grid">
                                        <div class="detail-row">
                                            <span class="detail-label">"Nullifier Hash:"</span>
                                            <span class="detail-value monospace">{nullifier_hash_preview}</span>
                                        </div>
                                        <div class="detail-row">
                                            <span class="detail-label">"Root:"</span>
                                            <span class="detail-value monospace">{merkle_root_preview}</span>
                                        </div>
                                        <div class="detail-row">
                                            <span class="detail-label">"Proof Size:"</span>
                                            <span class="detail-value">{proof_len}" bytes"</span>
                                        </div>
                                    </div>
                                </div>
                                
                                <div class="next-steps">
                                    <h5>"Next Steps:"</h5>
                                    <ol>
                                        <li>"Copy or download your withdrawal proof"</li>
                                        <li>"Submit the transaction to the Bitcoin network"</li>
                                        <li>"Wait for confirmation (usually 10-60 minutes)"</li>
                                        <li>"Your funds will be available at the recipient address"</li>
                                    </ol>
                                </div>
                            </div>
                        })
                    },
                    WithdrawalStatus::Error(error) => {
                        Some(view! {
                            <div class="error-result">
                                <div class="error-header">
                                    <span class="error-icon">"❌"</span>
                                    <h4>"Withdrawal Failed"</h4>
                                </div>
                                <p class="error-message">{error}</p>
                                
                                <div class="error-help">
                                    <h5>"Common Issues:"</h5>
                                    <ul>
                                        <li>"Invalid deposit note format"</li>
                                        <li>"Invalid recipient address"</li>
                                        <li>"Network connectivity issues"</li>
                                        <li>"Note already used for withdrawal"</li>
                                    </ul>
                                </div>
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
fn validate_bitcoin_address(address: &str) -> bool {
    // Basic validation - in production, use a proper Bitcoin address validator
    !address.is_empty() && 
    address.len() >= 26 && 
    address.len() <= 62 &&
    (address.starts_with('1') || address.starts_with('3') || address.starts_with("bc1"))
}

fn copy_to_clipboard(text: &str) {
    // In a real implementation, use the Clipboard API
    log::info!("Copying to clipboard: {}", text);
}

fn download_as_file(content: &str, filename: &str) {
    // In a real implementation, create a download link
    log::info!("Downloading file: {} with content: {}", filename, content);
}