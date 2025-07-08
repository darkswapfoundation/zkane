
//! Main application component for ZKane Frontend

use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use crate::components::*;
use crate::services::*;
use crate::types::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    
    // Global services
    let notification_service = NotificationService::new();
    let storage_service = StorageService::new();
    let zkane_service = ZKaneService::new();
    let alkanes_service = AlkanesService::new();
    let wallet_service = WalletService::new();

    // Detect wallets on startup
    let wallet_service_clone = wallet_service.clone();
    create_resource(|| (), move |_| {
        let wallet_service = wallet_service_clone.clone();
        async move {
            wallet_service.detect_wallets().await;
        }
    });

    // Global state
    let (app_config, _set_app_config) = create_signal(AppConfig::default());
    let (user_preferences, set_user_preferences) = create_signal(UserPreferences::default());
    
    // Load user preferences from storage
    let storage_service_clone = storage_service.clone();
    spawn_local(async move {
        if let Ok(prefs) = storage_service_clone.load_preferences() {
            set_user_preferences.set(prefs);
        }
    });
    
    // Provide services to child components
    provide_context(notification_service.clone());
    provide_context(storage_service);
    provide_context(zkane_service);
    provide_context(alkanes_service);
    provide_context(wallet_service.clone());
    provide_context(app_config);
    provide_context(user_preferences);
    provide_context(set_user_preferences);

    view! {
        <Html lang="en" dir="ltr" attr:data-theme=move || {
            match user_preferences.get().theme {
                Theme::Light => "light",
                Theme::Dark => "dark",
                Theme::Auto => "auto",
            }
        }/>
        
        <Title text="ZKane Privacy Pool"/>
        <Meta name="description" content="Privacy-preserving alkanes transactions using zero-knowledge proofs"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>
        <Meta charset="utf-8"/>
        
        <Link rel="icon" type_="image/png" href="/assets/favicon.png"/>

        <Router>
            <div class="app">
                <Header/>
                <NotificationContainer/>
                
                <main class="main-content">
                    {move || {
                        if wallet_service.connected_wallet.get().is_some() {
                            view! {
                                <Routes>
                                    <Route path="/" view=HomePage/>
                                    <Route path="/deposit" view=DepositPage/>
                                    <Route path="/withdraw" view=WithdrawPage/>
                                    <Route path="/pools" view=PoolsPage/>
                                    <Route path="/history" view=HistoryPage/>
                                    <Route path="/settings" view=SettingsPage/>
                                    <Route path="/help" view=HelpPage/>
                                    <Route path="/about" view=AboutPage/>
                                </Routes>
                            }.into_view()
                        } else {
                            view! { <WalletNotConnected/> }.into_view()
                        }
                    }}
                </main>

                <Footer/>
            </div>
        </Router>
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="header-content">
                <div class="header-brand">
                    <A href="/" class="brand-link">
                        <div class="brand-logo">"⬢"</div>
                        <span class="brand-text">"ZKane"</span>
                    </A>
                    <span class="brand-subtitle">"Privacy Infrastructure"</span>
                </div>

                <nav class="header-nav">
                    <A href="/deposit" class="nav-link">
                        <span class="nav-icon">"⬇"</span>
                        "Deposit"
                    </A>
                    <A href="/withdraw" class="nav-link">
                        <span class="nav-icon">"⬆"</span>
                        "Withdraw"
                    </A>
                    <A href="/pools" class="nav-link">
                        <span class="nav-icon">"◯"</span>
                        "Pools"
                    </A>
                    <A href="/history" class="nav-link">
                        <span class="nav-icon">"▤"</span>
                        "History"
                    </A>
                </nav>

                <div class="header-actions">
                    <WalletConnectorComponent />
                    <div class="action-button" title="Network Status">
                        <span class="action-icon">"🌐"</span>
                    </div>
                    <ThemeToggle/>
                    <A href="/settings" class="action-button">
                        <span class="action-icon">"⚙️"</span>
                    </A>
                    <A href="/help" class="action-button">
                        <span class="action-icon">"?"</span>
                    </A>
                </div>
            </div>
        </header>
    }
}

#[component]
fn ThemeToggle() -> impl IntoView {
    let user_preferences = expect_context::<ReadSignal<UserPreferences>>();
    let set_user_preferences = expect_context::<WriteSignal<UserPreferences>>();
    let storage_service = expect_context::<StorageService>();

    let toggle_theme = move |_| {
        set_user_preferences.update(|prefs| {
            prefs.theme = match prefs.theme {
                Theme::Light => Theme::Dark,
                Theme::Dark => Theme::Auto,
                Theme::Auto => Theme::Light,
            };
        });
        
        // Save to storage
        let prefs = user_preferences.get();
        let _ = storage_service.save_preferences(&prefs);
    };

    view! {
        <button
            class="theme-toggle action-button"
            on:click=toggle_theme
            title=move || {
                match user_preferences.get().theme {
                    Theme::Light => "Switch to Dark Theme",
                    Theme::Dark => "Switch to Auto Theme",
                    Theme::Auto => "Switch to Light Theme",
                }
            }
        >
            <span class="action-icon">
                {move || match user_preferences.get().theme {
                    Theme::Light => "☀",
                    Theme::Dark => "🌙",
                    Theme::Auto => "◐",
                }}
            </span>
        </button>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="footer-content">
                <div class="footer-section">
                    <h4>"ZKane Protocol"</h4>
                    <p>"Enterprise privacy infrastructure for Bitcoin alkanes using audited zero-knowledge cryptography"</p>
                </div>
                
                <div class="footer-section">
                    <h4>"Developer Resources"</h4>
                    <ul class="footer-links">
                        <li><a href="/help" class="footer-link">"Documentation"</a></li>
                        <li><a href="/about" class="footer-link">"Protocol Specification"</a></li>
                        <li><a href="https://github.com/zkane-project" target="_blank" class="footer-link">"GitHub Repository"</a></li>
                        <li><a href="https://docs.zkane.io" target="_blank" class="footer-link">"API Reference"</a></li>
                    </ul>
                </div>
                
                <div class="footer-section">
                    <h4>"Security & Audits"</h4>
                    <ul class="footer-links">
                        <li><a href="https://audits.zkane.io" target="_blank" class="footer-link">"Security Audits"</a></li>
                        <li><a href="https://bug-bounty.zkane.io" target="_blank" class="footer-link">"Bug Bounty Program"</a></li>
                        <li><a href="/security" class="footer-link">"Security Best Practices"</a></li>
                        <li><a href="https://transparency.zkane.io" target="_blank" class="footer-link">"Transparency Reports"</a></li>
                    </ul>
                </div>
                
                <div class="footer-section">
                    <h4>"Enterprise & Legal"</h4>
                    <ul class="footer-links">
                        <li><a href="https://enterprise.zkane.io" target="_blank" class="footer-link">"Enterprise Solutions"</a></li>
                        <li><a href="/privacy" class="footer-link">"Privacy Policy"</a></li>
                        <li><a href="/terms" class="footer-link">"Terms of Service"</a></li>
                        <li><a href="/compliance" class="footer-link">"Compliance Framework"</a></li>
                    </ul>
                </div>
            </div>
            
            <div class="footer-bottom">
                <p>"© 2024 ZKane Protocol. Open-source privacy infrastructure built with Rust."</p>
                <p class="footer-version">"v" {crate::get_app_version()}</p>
            </div>
        </footer>
    }
}

// Page components
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="page home-page">
            <div class="hero-section">
                <div class="hero-content">
                    <h1 class="hero-title">"Enterprise Privacy Infrastructure"</h1>
                    <p class="hero-subtitle">
                        "Zero-knowledge privacy pools for Bitcoin alkanes. Mathematical privacy, enterprise-grade security, self-sovereign control."
                    </p>
                    <div class="hero-actions">
                        <A href="/deposit" class="btn btn-primary btn-large">
                            "Create Private Deposit"
                        </A>
                        <A href="/pools" class="btn btn-secondary btn-large">
                            "Explore Privacy Pools"
                        </A>
                    </div>
                    <div class="hero-trust-indicators">
                        <span class="trust-badge">"Audited"</span>
                        <span class="trust-separator">"•"</span>
                        <span class="trust-badge">"Open Source"</span>
                        <span class="trust-separator">"•"</span>
                        <span class="trust-badge">"Non-custodial"</span>
                    </div>
                </div>
                <div class="hero-visual">
                    <div class="privacy-illustration">
                        <div class="privacy-node">"⬢"</div>
                        <div class="privacy-node">"◆"</div>
                        <div class="privacy-node">"⬟"</div>
                    </div>
                </div>
            </div>

            <div class="features-section">
                <h2>"Enterprise-Grade Privacy Technology"</h2>
                <div class="features-grid">
                    <FeatureCard
                        icon="∮"
                        title="Mathematical Privacy"
                        description="Cryptographic guarantees, not obfuscation. Zero-knowledge proofs ensure mathematical certainty of privacy."
                    />
                    <FeatureCard
                        icon="₿"
                        title="Bitcoin Native"
                        description="Built for Bitcoin's newest metaprotocol. Seamless integration with alkanes infrastructure."
                    />
                    <FeatureCard
                        icon="⬢"
                        title="Enterprise Ready"
                        description="Institutional-grade security and compliance. Audited smart contracts and battle-tested cryptography."
                    />
                    <FeatureCard
                        icon="🔑"
                        title="Self-Sovereign"
                        description="Your keys, your privacy, your control. No trusted third parties or custodial risks."
                    />
                </div>
            </div>

            <QuickStats/>
        </div>
    }
}

#[component]
fn FeatureCard(
    icon: &'static str,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <div class="feature-card">
            <div class="feature-icon">{icon}</div>
            <h3 class="feature-title">{title}</h3>
            <p class="feature-description">{description}</p>
        </div>
    }
}

#[component]
fn QuickStats() -> impl IntoView {
    let alkanes_service = expect_context::<AlkanesService>();
    
    let pools_stats = Resource::new(
        || (),
        move |_| {
            let alkanes_service = alkanes_service.clone();
            let wallet_service = expect_context::<WalletService>();
            async move {
                if let Some(wallet_provider) = wallet_service.connected_wallet.get() {
                    alkanes_service.get_privacy_pools(&wallet_provider).await
                } else {
                    Err(ZKaneError::WasmError("Wallet not connected".to_string()))
                }
            }
        }
    );

    view! {
        <div class="stats-section">
            <h2>"Network Statistics"</h2>
            <Suspense fallback=|| view! { <div class="loading">"Loading network statistics..."</div> }>
                {move || {
                    pools_stats.get().map(|result| {
                        match result {
                            Ok(pools) => {
                                let total_pools = pools.len();
                                let total_deposits: u64 = pools.iter().map(|p| p.total_deposits).sum();
                                let avg_anonymity: u64 = if total_pools > 0 {
                                    pools.iter().map(|p| p.anonymity_set).sum::<u64>() / total_pools as u64
                                } else { 0 };

                                view! {
                                    <div class="stats-grid">
                                        <StatCard
                                            value=format!("{}", total_pools)
                                            label="Active Privacy Pools"
                                            icon="⚪"
                                        />
                                        <StatCard
                                            value=format!("{}", total_deposits)
                                            label="Total Value Locked"
                                            icon="₿"
                                        />
                                        <StatCard
                                            value=format!("{}", avg_anonymity)
                                            label="Average Anonymity Set"
                                            icon="∮"
                                        />
                                        <StatCard
                                            value="99.9%".to_string()
                                            label="Network Uptime"
                                            icon="⬢"
                                        />
                                    </div>
                                }.into_any()
                            },
                            Err(_) => view! {
                                <div class="error-state">
                                    <div class="error-icon">"⚠"</div>
                                    <h3 class="error-title">"Unable to Load Statistics"</h3>
                                    <p class="error-message">"Please check your connection and try again."</p>
                                </div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn StatCard(
    value: String,
    label: &'static str,
    icon: &'static str,
) -> impl IntoView {
    view! {
        <div class="stat-card">
            <div class="stat-icon">{icon}</div>
            <div class="stat-value">{value}</div>
            <div class="stat-label">{label}</div>
        </div>
    }
}

#[component]
fn DepositPage() -> impl IntoView {
    view! {
        <div class="page deposit-page">
            <div class="page-header">
                <h1>"Create Privacy Deposit"</h1>
                <p>"Deposit alkanes assets into enterprise-grade privacy pools with mathematical anonymity guarantees"</p>
            </div>
            <DepositComponent/>
        </div>
    }
}

#[component]
fn WithdrawPage() -> impl IntoView {
    view! {
        <div class="page withdraw-page">
            <div class="page-header">
                <h1>"Zero-Knowledge Withdrawal"</h1>
                <p>"Withdraw your assets privately using cryptographic proofs. No transaction linkability, guaranteed."</p>
            </div>
            <WithdrawComponent/>
        </div>
    }
}

#[component]
fn PoolsPage() -> impl IntoView {
    view! {
        <div class="page pools-page">
            <div class="page-header">
                <h1>"Privacy Pool Discovery"</h1>
                <p>"Explore active privacy pools, anonymity sets, and network statistics for optimal privacy selection"</p>
            </div>
            <PoolListComponent/>
        </div>
    }
}

#[component]
fn HistoryPage() -> impl IntoView {
    view! {
        <div class="page history-page">
            <div class="page-header">
                <h1>"Deposit Note Management"</h1>
                <p>"Securely manage your deposit notes and track privacy pool participation history"</p>
            </div>
            <HistoryComponent/>
        </div>
    }
}

#[component]
fn SettingsPage() -> impl IntoView {
    view! {
        <div class="page settings-page">
            <div class="page-header">
                <h1>"Privacy Preferences"</h1>
                <p>"Configure your privacy settings, network preferences, and security parameters"</p>
            </div>
            <SettingsComponent/>
        </div>
    }
}

#[component]
fn HelpPage() -> impl IntoView {
    view! {
        <div class="page help-page">
            <div class="page-header">
                <h1>"Documentation & Security Guide"</h1>
                <p>"Learn zero-knowledge privacy fundamentals and best practices for secure usage"</p>
            </div>
            <HelpComponent/>
        </div>
    }
}

#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <div class="page about-page">
            <div class="page-header">
                <h1>"About ZKane Protocol"</h1>
                <p>"Enterprise privacy infrastructure for Bitcoin alkanes using audited zero-knowledge cryptography"</p>
            </div>
            <AboutComponent/>
        </div>
    }
}

#[component]
fn WalletNotConnected() -> impl IntoView {
    view! {
        <div class="page wallet-not-connected">
            <div class="page-header">
                <h1>"Connect Your Wallet"</h1>
                <p>"Please connect a wallet to use ZKane."</p>
            </div>
            <WalletConnectorComponent />
        </div>
    }
}

#[component]
fn ErrorPage(
    title: &'static str,
    message: &'static str,
    show_home_link: bool,
) -> impl IntoView {
    view! {
        <div class="page error-page">
            <div class="error-content">
                <div class="error-icon">"❌"</div>
                <h1 class="error-title">{title}</h1>
                <p class="error-message">{message}</p>
                {show_home_link.then(|| view! {
                    <A href="/" class="btn btn-primary">"Go Home"</A>
                })}
            </div>
        </div>
    }
}