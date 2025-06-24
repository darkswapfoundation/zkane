#![allow(dead_code)]

use wasm_bindgen_test::*;
use leptos::*;
use wasm_bindgen::JsCast;

wasm_bindgen_test_configure!(run_in_browser);

// Simple component that doesn't use complex services
#[component]
fn SimpleTestComponent() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    
    view! {
        <div>
            <button on:click=move |_| set_count.update(|n| *n += 1)>
                "Count: " {count}
            </button>
        </div>
    }
}

#[wasm_bindgen_test]
async fn test_simple_component() {
    let document = web_sys::window().unwrap().document().unwrap();
    let test_wrapper = document.create_element("section").unwrap();
    let _ = document.body().unwrap().append_child(&test_wrapper);

    // Mount a simple component without complex services
    let _dispose = mount_to(
        test_wrapper.clone().unchecked_into(),
        || view! { <SimpleTestComponent/> },
    );

    // Wait for the reactive system to settle
    gloo_timers::future::TimeoutFuture::new(100).await;

    // Check that the component rendered
    let content = test_wrapper.inner_html();
    assert!(!content.is_empty(), "Component should render content");
    assert!(content.contains("Count:"), "Should contain count text");
}

#[wasm_bindgen_test]
fn test_basic_wasm() {
    // Test that WASM is working
    assert_eq!(2 + 2, 4);
}

#[wasm_bindgen_test]
fn test_web_apis() {
    // Test that we can access web APIs
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    assert!(document.body().is_some());
}