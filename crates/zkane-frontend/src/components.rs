//! UI Components for ZKane Frontend application

mod deposit;
mod withdraw;
mod pool_list;
mod history;
mod settings;
mod help;
mod about;
mod notifications;
mod utils;

pub use deposit::*;
pub use withdraw::*;
pub use pool_list::*;
pub use history::*;
pub use settings::*;
pub use help::*;
pub use about::*;
pub use notifications::*;
pub use utils::*;

use leptos::*;
use crate::types::*;
use crate::services::*;

#[component]
pub fn DepositComponent() -> impl IntoView {
    let zkane_service = expect_context::<ZKaneService>();
    let alkanes_service = expect_context::<AlkanesService>();
    let notification_service = expect_context::<NotificationService>();
    let storage_service = expect_context::<StorageService>();
    
    // State
    let (selected_asset, set_selected_asset) = create_signal(None::<AssetBalance>);
    let (deposit_amount, set_deposit_amount) = create_signal(String::new());
    let (deposit_status, set_deposit_status) = create_signal(DepositStatus::Idle);
    let (created_note, set_created_note) = create_signal(None::<DepositNote>);
    
    // Load user assets
    let alkanes_service_for_assets = alkanes_service.clone();
    let user_assets = Resource::new(
        || (),
        move |_| {
            let alkanes_service = alkanes_service_for_assets.clone();
            async move {
                alkanes_service.get_user_assets("user_address").await
            }
        },
    );

    // Deposit action
    let deposit_action = Action::new({
        let zkane_service = zkane_service.clone();
        let alkanes_service = alkanes_service.clone();
        let notification_service = notification_service.clone();
        let storage_service = storage_service.clone();
        move |_: &()| {
            let zkane_service = zkane_service.clone();
            let alkanes_service = alkanes_service.clone();
            let notification_service = notification_service.clone();
            let storage_service = storage_service.clone();
            let selected_asset = selected_asset.get();
            let amount_str = deposit_amount.get();
            
            async move {
                if let Some(asset) = selected_asset {
                    set_deposit_status.set(DepositStatus::ValidatingAmount);
                    
                    // Validate amount
                    let amount = match parse_amount(&amount_str, asset.decimals) {
                        Ok(amt) if amt <= asset.balance => amt,
                        Ok(_) => {
                            set_deposit_status.set(DepositStatus::Error("Insufficient balance".to_string()));
                            notification_service.error("Invalid Amount", "Insufficient balance");
                            return;
                        },
                        Err(e) => {
                            set_deposit_status.set(DepositStatus::Error(e.clone()));
                            notification_service.error("Invalid Amount", &e);
                            return;
                        }
                    };
                    
                    set_deposit_status.set(DepositStatus::CreatingNote);
                    
                    // Create deposit note
                    match zkane_service.create_deposit(asset.asset_id.clone(), amount).await {
                        Ok(note) => {
                            set_created_note.set(Some(note.clone()));
                            set_deposit_status.set(DepositStatus::Complete(note.clone()));
                            
                            // Save note to storage if auto-save is enabled
                            if let Err(e) = storage_service.save_deposit_note(&note) {
                                log::warn!("Failed to save deposit note: {:?}", e);
                            }
                            
                            notification_service.success(
                                "Deposit Note Created",
                                "Your deposit note has been created successfully. Save it securely!"
                            );
                        },
                        Err(e) => {
                            let error_msg = format!("Failed to create deposit note: {:?}", e);
                            set_deposit_status.set(DepositStatus::Error(error_msg.clone()));
                            notification_service.error("Deposit Failed", &error_msg);
                        }
                    }
                } else {
                    set_deposit_status.set(DepositStatus::Error("No asset selected".to_string()));
                    notification_service.error("No Asset Selected", "Please select an asset to deposit");
                }
            }
        }
    });

    view! {
        <div class="deposit-component">
            <AssetSelector 
                assets=user_assets
                selected_asset=selected_asset
                set_selected_asset=set_selected_asset
            />
            
            <AmountInput 
                amount=deposit_amount
                set_amount=set_deposit_amount
                selected_asset=selected_asset
                disabled=Signal::derive(move || !matches!(deposit_status.get(), DepositStatus::Idle))
            />
            
            <DepositActions 
                deposit_action=deposit_action
                deposit_status=deposit_status
                selected_asset=selected_asset
                amount=deposit_amount
            />
            
            <DepositResult
                status=deposit_status
                created_note=created_note
                storage_service=storage_service.clone()
            />
        </div>
    }
}

#[component]
pub fn WithdrawComponent() -> impl IntoView {
    let zkane_service = expect_context::<ZKaneService>();
    let alkanes_service = expect_context::<AlkanesService>();
    let notification_service = expect_context::<NotificationService>();
    
    // State
    let (deposit_note_json, set_deposit_note_json) = create_signal(String::new());
    let (recipient_address, set_recipient_address) = create_signal(String::new());
    let (withdrawal_status, set_withdrawal_status) = create_signal(WithdrawalStatus::Idle);
    let (parsed_note, set_parsed_note) = create_signal(None::<DepositNote>);
    let (generated_proof, set_generated_proof) = create_signal(None::<WithdrawalProof>);

    // Parse note when JSON changes
    let notification_service_for_parse = notification_service.clone();
    let parse_note = move || {
        let json = deposit_note_json.get();
        if !json.is_empty() {
            match serde_json::from_str::<DepositNote>(&json) {
                Ok(note) => {
                    set_parsed_note.set(Some(note));
                    notification_service_for_parse.info("Note Parsed", "Deposit note loaded successfully");
                },
                Err(_) => {
                    set_parsed_note.set(None);
                    notification_service_for_parse.error("Invalid Note", "Failed to parse deposit note JSON");
                }
            }
        } else {
            set_parsed_note.set(None);
        }
    };

    // Withdrawal action
    let withdraw_action = Action::new(move |_: &()| {
        let zkane_service = zkane_service.clone();
        let notification_service = notification_service.clone();
        let note_json = deposit_note_json.get();
        let recipient = recipient_address.get();
        
        async move {
            set_withdrawal_status.set(WithdrawalStatus::ParsingNote);
            
            // Parse and validate deposit note
            let deposit_note = match serde_json::from_str::<DepositNote>(&note_json) {
                Ok(note) => note,
                Err(_) => {
                    set_withdrawal_status.set(WithdrawalStatus::Error("Invalid deposit note".to_string()));
                    notification_service.error("Invalid Note", "Failed to parse deposit note");
                    return;
                }
            };
            
            set_withdrawal_status.set(WithdrawalStatus::ValidatingRecipient);
            
            // Validate recipient address
            if !validate_bitcoin_address(&recipient) {
                set_withdrawal_status.set(WithdrawalStatus::Error("Invalid recipient address".to_string()));
                notification_service.error("Invalid Address", "Please enter a valid Bitcoin address");
                return;
            }
            
            set_withdrawal_status.set(WithdrawalStatus::GeneratingProof);
            
            // Create transaction outputs
            let outputs = vec![TxOutput {
                value: deposit_note.denomination,
                script_pubkey: recipient.clone(),
            }];
            
            // Mock merkle path (in production, fetch from indexer)
            let merkle_path = MerklePath {
                root: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
                elements: vec!["0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string()],
                indices: vec![false],
                leaf_index: deposit_note.leaf_index,
            };
            
            // Generate withdrawal proof
            match zkane_service.generate_withdrawal_proof(&deposit_note, &outputs, &merkle_path).await {
                Ok(proof) => {
                    set_generated_proof.set(Some(proof.clone()));
                    set_withdrawal_status.set(WithdrawalStatus::Complete(proof));
                    notification_service.success(
                        "Proof Generated",
                        "Withdrawal proof generated successfully"
                    );
                },
                Err(e) => {
                    let error_msg = format!("Failed to generate proof: {:?}", e);
                    set_withdrawal_status.set(WithdrawalStatus::Error(error_msg.clone()));
                    notification_service.error("Proof Generation Failed", &error_msg);
                }
            }
        }
    });

    view! {
        <div class="withdraw-component">
            <NoteInput 
                note_json=deposit_note_json
                set_note_json=set_deposit_note_json
                parse_note=parse_note
                parsed_note=parsed_note
            />
            
            <RecipientInput 
                recipient=recipient_address
                set_recipient=set_recipient_address
                disabled=Signal::derive(move || parsed_note.get().is_none())
            />
            
            <WithdrawActions 
                withdraw_action=withdraw_action
                withdrawal_status=withdrawal_status
                parsed_note=parsed_note
                recipient=recipient_address
            />
            
            <WithdrawResult 
                status=withdrawal_status
                generated_proof=generated_proof
            />
        </div>
    }
}

#[component]
pub fn PoolListComponent() -> impl IntoView {
    let alkanes_service = expect_context::<AlkanesService>();
    
    // State
    let (filter_asset, set_filter_asset) = create_signal(String::new());
    let (sort_by, set_sort_by) = create_signal("anonymity_set".to_string());
    let (sort_desc, set_sort_desc) = create_signal(true);
    
    // Load privacy pools
    let pools = Resource::new(
        || (),
        move |_| {
            let alkanes_service = alkanes_service.clone();
            async move {
                alkanes_service.get_privacy_pools().await
            }
        },
    );

    view! {
        <div class="pool-list-component">
            <PoolFilters 
                filter_asset=filter_asset
                set_filter_asset=set_filter_asset
                sort_by=sort_by
                set_sort_by=set_sort_by
                sort_desc=sort_desc
                set_sort_desc=set_sort_desc
            />
            
            <Suspense fallback=|| view! { <LoadingSpinner message="Loading pools..."/> }>
                {move || {
                    pools.get().map(|result| -> leptos::View {
                        match result {
                            Ok(pools) => {
                                let filtered_pools = filter_and_sort_pools(
                                    pools,
                                    &filter_asset.get(),
                                    &sort_by.get(),
                                    sort_desc.get()
                                );
                                
                                if filtered_pools.is_empty() {
                                    view! {
                                        <EmptyState
                                            icon="🏊"
                                            title="No Pools Found"
                                            message="No privacy pools match your current filters"
                                        />
                                    }.into_view()
                                } else {
                                    view! {
                                        <div class="pools-grid">
                                            {filtered_pools.into_iter().map(|pool| {
                                                view! { <PoolCard pool=pool/> }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_view()
                                }
                            },
                            Err(e) => view! {
                                <ErrorState
                                    title="Failed to Load Pools"
                                    message=format!("Error: {:?}", e)
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
pub fn HistoryComponent() -> impl IntoView {
    let storage_service = expect_context::<StorageService>();
    let notification_service = expect_context::<NotificationService>();
    
    // Load saved deposit notes
    let saved_notes = Resource::new(
        || (),
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
                <button 
                    class="btn btn-secondary"
                    on:click=move |_| {
                        saved_notes.refetch();
                        notification_service.info("Refreshed", "Deposit notes refreshed");
                    }
                >
                    "Refresh"
                </button>
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
                                            title="No History"
                                            message="You haven't created any deposit notes yet"
                                        />
                                    }.into_view()
                                } else {
                                    view! {
                                        <div class="notes-list">
                                            {notes.into_iter().map(|note| {
                                                view! { <NoteCard note=note/> }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_view()
                                }
                            },
                            Err(e) => view! {
                                <ErrorState
                                    title="Failed to Load History"
                                    message=format!("Error: {:?}", e)
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
pub fn SettingsComponent() -> impl IntoView {
    let user_preferences = expect_context::<ReadSignal<UserPreferences>>();
    let set_user_preferences = expect_context::<WriteSignal<UserPreferences>>();
    let storage_service = expect_context::<StorageService>();
    let notification_service = expect_context::<NotificationService>();

    let save_preferences = {
        let user_preferences = user_preferences;
        let storage_service = storage_service.clone();
        let notification_service = notification_service.clone();
        move || {
            let prefs = user_preferences.get();
            match storage_service.save_preferences(&prefs) {
                Ok(_) => notification_service.success("Settings Saved", "Your preferences have been saved"),
                Err(e) => notification_service.error("Save Failed", &format!("Failed to save settings: {:?}", e)),
            }
        }
    };

    view! {
        <div class="settings-component">
            <div class="settings-section">
                <h3>"Appearance"</h3>
                <ThemeSelector
                    current_theme=user_preferences.get().theme
                    on_change={
                        let save_preferences = save_preferences.clone();
                        move |theme| {
                            set_user_preferences.update(|prefs| prefs.theme = theme);
                            save_preferences();
                        }
                    }
                />
            </div>
            
            <div class="settings-section">
                <h3>"Privacy"</h3>
                <ToggleSetting
                    label="Auto-save deposit notes"
                    description="Automatically save deposit notes to local storage"
                    checked=Signal::derive(move || user_preferences.get().auto_save_notes)
                    on_change={
                        let save_preferences = save_preferences.clone();
                        move |checked| {
                            set_user_preferences.update(|prefs| prefs.auto_save_notes = checked);
                            save_preferences();
                        }
                    }
                />
            </div>
            
            <div class="settings-section">
                <h3>"Advanced"</h3>
                <ToggleSetting
                    label="Show advanced options"
                    description="Display advanced configuration options"
                    checked=Signal::derive(move || user_preferences.get().show_advanced_options)
                    on_change={
                        let save_preferences = save_preferences.clone();
                        move |checked| {
                            set_user_preferences.update(|prefs| prefs.show_advanced_options = checked);
                            save_preferences();
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
pub fn HelpComponent() -> impl IntoView {
    view! {
        <div class="help-component">
            <div class="help-section">
                <h3>"Getting Started"</h3>
                <div class="help-content">
                    <h4>"1. Understanding Privacy Pools"</h4>
                    <p>
                        "Privacy pools allow you to deposit alkanes assets and later withdraw them to a different address, "
                        "breaking the link between your deposit and withdrawal transactions."
                    </p>

                    <h4>"2. Making a Deposit"</h4>
                    <ol>
                        <li>"Select an alkanes asset from your wallet"</li>
                        <li>"Choose an amount to deposit (this determines which pool you join)"</li>
                        <li>"Create a deposit note - save this securely!"</li>
                        <li>"Send the transaction to deposit your assets"</li>
                    </ol>

                    <h4>"3. Making a Withdrawal"</h4>
                    <ol>
                        <li>"Load your saved deposit note"</li>
                        <li>"Enter the recipient Bitcoin address"</li>
                        <li>"Generate a zero-knowledge proof"</li>
                        <li>"Submit the withdrawal transaction"</li>
                    </ol>
                </div>
            </div>

            <div class="help-section">
                <h3>"Security Best Practices"</h3>
                <div class="help-content">
                    <SecurityTip 
                        icon="🔐"
                        title="Secure Your Deposit Notes"
                        description="Your deposit note contains the secret information needed to withdraw your funds. Store it securely and never share it with anyone."
                    />
                    
                    <SecurityTip 
                        icon="🌐"
                        title="Use Different Networks"
                        description="For maximum privacy, use different network connections (VPN, Tor) when making deposits versus withdrawals."
                    />
                    
                    <SecurityTip 
                        icon="⏰"
                        title="Wait Between Transactions"
                        description="Wait for more deposits to join your pool before withdrawing. Larger anonymity sets provide better privacy."
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AboutComponent() -> impl IntoView {
    view! {
        <div class="about-component">
            <div class="about-section">
                <h3>"About ZKane"</h3>
                <p>
                    "ZKane is a privacy-preserving protocol for alkanes assets on Bitcoin. "
                    "Using zero-knowledge proofs, ZKane enables anonymous transactions while "
                    "maintaining the security and decentralization of the Bitcoin network."
                </p>
            </div>
            
            <div class="about-section">
                <h3>"Technology"</h3>
                <ul>
                    <li>"Built with Rust for performance and safety"</li>
                    <li>"WebAssembly for browser compatibility"</li>
                    <li>"Zero-knowledge proofs using Noir"</li>
                    <li>"Alkanes protocol integration"</li>
                    <li>"Leptos for reactive UI"</li>
                </ul>
            </div>
            
            <div class="about-section">
                <h3>"Version Information"</h3>
                <p>"Version: " {crate::get_app_version()}</p>
                <p>"Built with ❤️ by the ZKane team"</p>
            </div>
        </div>
    }
}

// Helper functions
fn filter_and_sort_pools(
    mut pools: Vec<PoolInfo>,
    filter_asset: &str,
    sort_by: &str,
    sort_desc: bool,
) -> Vec<PoolInfo> {
    // Filter by asset
    if !filter_asset.is_empty() {
        pools.retain(|pool| pool.asset_symbol.to_lowercase().contains(&filter_asset.to_lowercase()));
    }
    
    // Sort pools
    pools.sort_by(|a, b| {
        let ordering = match sort_by {
            "anonymity_set" => a.anonymity_set.cmp(&b.anonymity_set),
            "total_deposits" => a.total_deposits.cmp(&b.total_deposits),
            "denomination" => a.denomination.cmp(&b.denomination),
            "created_at" => a.created_at.partial_cmp(&b.created_at).unwrap_or(std::cmp::Ordering::Equal),
            _ => std::cmp::Ordering::Equal,
        };
        
        if sort_desc { ordering.reverse() } else { ordering }
    });
    
    pools
}

fn parse_amount(amount_str: &str, decimals: u8) -> Result<u128, String> {
    if amount_str.is_empty() {
        return Err("Amount cannot be empty".to_string());
    }

    let parsed: f64 = amount_str.parse()
        .map_err(|_| "Invalid amount format".to_string())?;
    
    if parsed <= 0.0 {
        return Err("Amount must be greater than zero".to_string());
    }
    
    let multiplier = 10u128.pow(decimals as u32) as f64;
    let amount = (parsed * multiplier) as u128;
    
    Ok(amount)
}

fn validate_bitcoin_address(address: &str) -> bool {
    !address.is_empty() && address.len() >= 26 && address.len() <= 62
}