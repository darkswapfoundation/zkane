
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
                        <img src="/assets/logo.svg" alt="ZKane" class="brand-logo"/>
                        <span class="brand-text">"ZKane"</span>
                    </A>
                    <span class="brand-subtitle">"Privacy Pool"</span>
                </div>

                <nav class="header-nav">
                    <A href="/deposit" class="nav-link">
                        <span class="nav-icon">"💰"</span>
                        "Deposit"
                    </A>
                    <A href="/withdraw" class="nav-link">
                        <span class="nav-icon">"🔒"</span>
                        "Withdraw"
                    </A>
                    <A href="/pools" class="nav-link">
                        <span class="nav-icon">"🏊"</span>
                        "Pools"
                    </A>
                    <A href="/history" class="nav-link">
                        <span class="nav-icon">"📜"</span>
                        "History"
                    </A>
                </nav>

                <div class="header-actions">
                    <ThemeToggle/>
                    <A href="/settings" class="action-button">
                        <span class="action-icon">"⚙️"</span>
                    </A>
                    <A href="/help" class="action-button">
                        <span class="action-icon">"❓"</span>
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
                    Theme::Light => "☀️",
                    Theme::Dark => "🌙",
                    Theme::Auto => "🌓",
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
                    <h4>"ZKane Privacy Pool"</h4>
                    <p>"Privacy-preserving alkanes transactions using zero-knowledge proofs"</p>
                </div>
                
                <div class="footer-section">
                    <h4>"Resources"</h4>
                    <ul class="footer-links">
                        <li><a href="/help" class="footer-link">"Documentation"</a></li>
                        <li><a href="/about" class="footer-link">"About"</a></li>
                        <li><a href="https://github.com/zkane-project" target="_blank" class="footer-link">"GitHub"</a></li>
                    </ul>
                </div>
                
                <div class="footer-section">
                    <h4>"Community"</h4>
                    <ul class="footer-links">
                        <li><a href="https://discord.gg/zkane" target="_blank" class="footer-link">"Discord"</a></li>
                        <li><a href="https://twitter.com/zkane_project" target="_blank" class="footer-link">"Twitter"</a></li>
                        <li><a href="https://t.me/zkane" target="_blank" class="footer-link">"Telegram"</a></li>
                    </ul>
                </div>
                
                <div class="footer-section">
                    <h4>"Legal"</h4>
                    <ul class="footer-links">
                        <li><a href="/privacy" class="footer-link">"Privacy Policy"</a></li>
                        <li><a href="/terms" class="footer-link">"Terms of Service"</a></li>
                    </ul>
                </div>
            </div>
            
            <div class="footer-bottom">
                <p>"© 2024 ZKane Project. Built with ❤️ and Rust."</p>
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
                    <h1 class="hero-title">"Privacy-First Alkanes Transactions"</h1>
                    <p class="hero-subtitle">
                        "Use zero-knowledge proofs to transact alkanes assets privately and securely"
                    </p>
                    <div class="hero-actions">
                        <A href="/deposit" class="btn btn-primary btn-large">
                            "Create Deposit"
                        </A>
                        <A href="/pools" class="btn btn-secondary btn-large">
                            "Browse Pools"
                        </A>
                    </div>
                </div>
                <div class="hero-visual">
                    <div class="privacy-illustration">
                        <div class="privacy-node">"🔒"</div>
                        <div class="privacy-node">"🔐"</div>
                        <div class="privacy-node">"🛡️"</div>
                    </div>
                </div>
            </div>

            <div class="features-section">
                <h2>"Why Choose ZKane?"</h2>
                <div class="features-grid">
                    <FeatureCard 
                        icon="🔒"
                        title="Privacy Preserving"
                        description="Zero-knowledge proofs ensure your transaction history remains private"
                    />
                    <FeatureCard 
                        icon="⚡"
                        title="Fast & Efficient"
                        description="Built with Rust and WebAssembly for optimal performance"
                    />
                    <FeatureCard 
                        icon="🪙"
                        title="Multi-Asset Support"
                        description="Works with any alkanes-based asset on Bitcoin"
                    />
                    <FeatureCard 
                        icon="🛡️"
                        title="Secure by Design"
                        description="Cryptographic guarantees with open-source transparency"
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
            async move {
                alkanes_service.get_privacy_pools().await
            }
        }
    );

    view! {
        <div class="stats-section">
            <h2>"Network Statistics"</h2>
            <Suspense fallback=|| view! { <div class="loading">"Loading stats..."</div> }>
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
                                            value=total_pools.to_string()
                                            label="Active Pools"
                                            icon="🏊"
                                        />
                                        <StatCard 
                                            value=total_deposits.to_string()
                                            label="Total Deposits"
                                            icon="💰"
                                        />
                                        <StatCard 
                                            value=avg_anonymity.to_string()
                                            label="Avg Anonymity Set"
                                            icon="🔒"
                                        />
                                        <StatCard
                                            value="100%".to_string()
                                            label="Privacy Guaranteed"
                                            icon="🛡️"
                                        />
                                    </div>
                                }.into_any()
                            },
                            Err(_) => view! {
                                <div class="error">"Failed to load statistics"</div>
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
                <p>"Deposit alkanes assets into a privacy pool for anonymous transactions"</p>
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
                <h1>"Private Withdrawal"</h1>
                <p>"Withdraw your assets privately using zero-knowledge proofs"</p>
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
                <h1>"Privacy Pools"</h1>
                <p>"Browse available privacy pools and their statistics"</p>
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
                <h1>"Transaction History"</h1>
                <p>"View your saved deposit notes and transaction history"</p>
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
                <h1>"Settings"</h1>
                <p>"Customize your ZKane experience"</p>
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
                <h1>"Help & Documentation"</h1>
                <p>"Learn how to use ZKane privacy pools safely and effectively"</p>
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
                <h1>"About ZKane"</h1>
                <p>"Privacy-preserving alkanes transactions using zero-knowledge proofs"</p>
            </div>
            <AboutComponent/>
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