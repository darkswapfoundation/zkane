//! Notification component and related UI elements

use leptos::*;
use crate::types::*;
use crate::services::NotificationService;

#[component]
pub fn NotificationContainer() -> impl IntoView {
    let notification_service = expect_context::<NotificationService>();
    let notifications = notification_service.notifications;

    view! {
        <div class="notification-container">
            <For
                each=move || notifications.get()
                key=|notification| notification.id.clone()
                children=move |notification| {
                    view! {
                        <NotificationItem notification=notification/>
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn NotificationItem(notification: Notification) -> impl IntoView {
    let notification_service = expect_context::<NotificationService>();
    let id = notification.id.clone();

    // TODO: Auto-dismiss after timeout - simplified for now
    // This can be implemented later with proper timeout handling
    let _timeout = notification.timeout; // Acknowledge the field exists

    view! {
        <div
            class="notification"
            class:notification-success=matches!(notification.notification_type, NotificationType::Success)
            class:notification-error=matches!(notification.notification_type, NotificationType::Error)
            class:notification-warning=matches!(notification.notification_type, NotificationType::Warning)
            class:notification-info=matches!(notification.notification_type, NotificationType::Info)
        >
            <div class="notification-icon">
                {match notification.notification_type {
                    NotificationType::Success => "✅",
                    NotificationType::Error => "❌",
                    NotificationType::Warning => "⚠️",
                    NotificationType::Info => "ℹ️",
                }}
            </div>
            
            <div class="notification-content">
                <div class="notification-title">{notification.title}</div>
                <div class="notification-message">{notification.message}</div>
            </div>
            
            <button
                class="notification-close"
                on:click=move |_| {
                    notification_service.dismiss(&id);
                }
            >
                "×"
            </button>
        </div>
    }
}