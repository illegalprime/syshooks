extern crate notify_rust;

use self::notify_rust::{
    Notification,
    NotificationHint,
};

pub use self::notify_rust::{
    NotificationHandle,
    Error,
};

pub fn show_brightness(percent: u32) -> Result<NotificationHandle, Error> {
    let icon = match percent {
        x if x <= 33 => "notification-display-brightness-low",
        x if x <= 67 => "notification-display-brightness-medium",
        x if x <= 99 => "notification-display-brightness-high",
        _ => "notification-display-brightness-full",
    };

    Notification::new()
        .summary(" ")
        .icon(icon)
        .hint(NotificationHint::CustomInt("value".to_string(), percent as i32))
        .hint(NotificationHint::Custom(
                "x-canonical-private-synchronous".to_string(),
                "brightness".to_string()))
        .show()
}
