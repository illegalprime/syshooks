extern crate notify_rust;

use self::notify_rust::{
    Notification,
    NotificationHint,
};

pub use self::notify_rust::Error;

pub enum Volume {
    Muted,
    Percent(u32),
}

pub fn show_volume(percent: Volume) -> Result<(), Error> {
    let icon = match percent {
        Volume::Muted => "notification-audio-volume-muted",
        Volume::Percent(x) if x == 0 => "notification-audio-volume-off",
        Volume::Percent(x) if x < 33 => "notification-audio-volume-low",
        Volume::Percent(x) if x < 67 => "notification-audio-volume-medium",
        _ => "notification-audio-volume-high",
    };

    let value = match percent {
        Volume::Muted => 0,
        Volume::Percent(p) => p,
    };

    let _ = Notification::new()
        .summary(" ")
        .icon(icon)
        .hint(NotificationHint::SoundName("audio-volume-change".to_string()))
        .hint(NotificationHint::Custom("synchronous".to_string(), "volume".to_string()))
        .hint(NotificationHint::CustomInt("value".to_string(), value as i32))
        .show()?;
    Ok(())
}
