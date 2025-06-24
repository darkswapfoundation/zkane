#![allow(dead_code)]

use wasm_bindgen_test::*;
use leptos::*;
use wasm_bindgen::JsCast;

wasm_bindgen_test_configure!(run_in_browser);

// Define types locally to avoid importing from the main library
#[derive(Clone, Debug, PartialEq)]
pub struct TestAlkaneId {
    pub block: u128,
    pub tx: u128,
}

impl TestAlkaneId {
    pub fn new(block: u128, tx: u128) -> Self {
        Self { block, tx }
    }
}

impl std::fmt::Display for TestAlkaneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.block, self.tx)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TestTheme {
    Light,
    Dark,
    Auto,
}

// Simple, isolated components that don't depend on the main library
#[component]
fn IsolatedCounter(
    #[prop(default = 0)] initial_value: i32,
    #[prop(default = 1)] step: i32,
) -> impl IntoView {
    let (value, set_value) = create_signal(initial_value);

    view! {
        <div class="isolated-counter">
            <button on:click=move |_| set_value.set(0)>"Clear"</button>
            <button on:click=move |_| set_value.update(|v| *v -= step)>"-" {step}</button>
            <span class="counter-value">"Value: " {value}</span>
            <button on:click=move |_| set_value.update(|v| *v += step)>"+" {step}</button>
        </div>
    }
}

#[component]
fn IsolatedThemeToggle(
    #[prop(default = TestTheme::Auto)] initial_theme: TestTheme,
) -> impl IntoView {
    let (theme, set_theme) = create_signal(initial_theme);

    let toggle_theme = move |_| {
        set_theme.update(|t| {
            *t = match *t {
                TestTheme::Light => TestTheme::Dark,
                TestTheme::Dark => TestTheme::Auto,
                TestTheme::Auto => TestTheme::Light,
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
                TestTheme::Light => "☀️",
                TestTheme::Dark => "🌙",
                TestTheme::Auto => "🌓",
            }}
        </button>
    }
}

#[component]
fn IsolatedAssetDisplay(
    asset_id: TestAlkaneId,
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
fn IsolatedApp() -> impl IntoView {
    view! {
        <div class="isolated-app">
            <header class="isolated-header">
                <h1>"ZKane Privacy Pool - Test Version"</h1>
                <IsolatedThemeToggle/>
            </header>
            
            <main class="isolated-main">
                <IsolatedCounter initial_value=0 step=1/>
                
                <div class="asset-list">
                    <IsolatedAssetDisplay 
                        asset_id=TestAlkaneId::new(1, 1)
                        symbol="ALKS".to_string()
                        balance=1000000000
                    />
                    <IsolatedAssetDisplay 
                        asset_id=TestAlkaneId::new(2, 1)
                        symbol="TEST".to_string()
                        balance=5000000000
                    />
                </div>
            </main>
        </div>
    }
}

// Tests that don't import anything from the main library
#[wasm_bindgen_test]
fn test_basic_wasm() {
    assert_eq!(2 + 2, 4);
}

#[wasm_bindgen_test]
fn test_web_apis() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    assert!(document.body().is_some());
}

#[wasm_bindgen_test]
fn test_alkane_id_creation() {
    let alkane_id = TestAlkaneId::new(123, 456);
    assert_eq!(alkane_id.block, 123);
    assert_eq!(alkane_id.tx, 456);
    
    let display_str = format!("{}", alkane_id);
    assert_eq!(display_str, "123:456");
}

#[wasm_bindgen_test]
fn test_theme_enum() {
    let light = TestTheme::Light;
    let dark = TestTheme::Dark;
    let auto = TestTheme::Auto;
    
    assert_ne!(light, dark);
    assert_ne!(dark, auto);
    assert_ne!(auto, light);
}

#[wasm_bindgen_test]
async fn test_isolated_counter() {
    let document = web_sys::window().unwrap().document().unwrap();
    let test_wrapper = document.create_element("section").unwrap();
    let _ = document.body().unwrap().append_child(&test_wrapper);

    let _dispose = mount_to(
        test_wrapper.clone().unchecked_into(),
        || view! { <IsolatedCounter initial_value=5 step=2/> },
    );

    // Wait for rendering
    gloo_timers::future::TimeoutFuture::new(100).await;

    let content = test_wrapper.inner_html();
    assert!(!content.is_empty(), "Counter should render content");
    assert!(content.contains("Value: 5"), "Should show initial value");
    assert!(content.contains("+2"), "Should show step value");
    assert!(content.contains("-2"), "Should show step value");
}

#[wasm_bindgen_test]
async fn test_counter_interaction() {
    let document = web_sys::window().unwrap().document().unwrap();
    let test_wrapper = document.create_element("section").unwrap();
    let _ = document.body().unwrap().append_child(&test_wrapper);

    let _dispose = mount_to(
        test_wrapper.clone().unchecked_into(),
        || view! { <IsolatedCounter initial_value=0 step=1/> },
    );

    // Wait for rendering
    gloo_timers::future::TimeoutFuture::new(100).await;

    // Find the increment button (last button)
    let buttons = test_wrapper.query_selector_all("button").unwrap();
    let inc_button = buttons.get(2).unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
    
    // Click increment button
    inc_button.click();
    
    // Wait for reactive update
    gloo_timers::future::TimeoutFuture::new(100).await;
    
    let content = test_wrapper.inner_html();
    assert!(content.contains("Value: 1"), "Should increment to 1");
}

#[wasm_bindgen_test]
async fn test_isolated_theme_toggle() {
    let document = web_sys::window().unwrap().document().unwrap();
    let test_wrapper = document.create_element("section").unwrap();
    let _ = document.body().unwrap().append_child(&test_wrapper);

    let _dispose = mount_to(
        test_wrapper.clone().unchecked_into(),
        || view! { <IsolatedThemeToggle initial_theme=TestTheme::Light/> },
    );

    // Wait for rendering
    gloo_timers::future::TimeoutFuture::new(100).await;

    let content = test_wrapper.inner_html();
    assert!(!content.is_empty(), "Theme toggle should render");
    assert!(content.contains("☀️"), "Should show light theme icon");
}

#[wasm_bindgen_test]
async fn test_isolated_asset_display() {
    let document = web_sys::window().unwrap().document().unwrap();
    let test_wrapper = document.create_element("section").unwrap();
    let _ = document.body().unwrap().append_child(&test_wrapper);

    let asset_id = TestAlkaneId::new(123, 456);
    let _dispose = mount_to(
        test_wrapper.clone().unchecked_into(),
        || view! { 
            <IsolatedAssetDisplay 
                asset_id=asset_id
                symbol="TEST".to_string()
                balance=1000000000
            /> 
        },
    );

    // Wait for rendering
    gloo_timers::future::TimeoutFuture::new(100).await;

    let content = test_wrapper.inner_html();
    assert!(!content.is_empty(), "Asset display should render");
    assert!(content.contains("123:456"), "Should show asset ID");
    assert!(content.contains("TEST"), "Should show symbol");
    assert!(content.contains("1000000000"), "Should show balance");
}

#[wasm_bindgen_test]
async fn test_isolated_app() {
    let document = web_sys::window().unwrap().document().unwrap();
    let test_wrapper = document.create_element("section").unwrap();
    let _ = document.body().unwrap().append_child(&test_wrapper);

    let _dispose = mount_to(
        test_wrapper.clone().unchecked_into(),
        || view! { <IsolatedApp/> },
    );

    // Wait for rendering
    gloo_timers::future::TimeoutFuture::new(100).await;

    let content = test_wrapper.inner_html();
    assert!(!content.is_empty(), "Isolated app should render");
    assert!(content.contains("ZKane Privacy Pool - Test Version"), "Should show app title");
    assert!(content.contains("Value: 0"), "Should show counter");
    assert!(content.contains("ALKS"), "Should show asset");
}