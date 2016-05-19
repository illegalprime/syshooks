use std::env::args;
use std::process::exit;
use std::io::{
    stderr,
    Write,
};

mod notify;
mod manage;
mod audio;

use manage::brightness::Brightness;
use manage::brightness::xcb::XcbBrightness;
use manage::volume::Mixer;

use notify::volume::Volume;
use notify::volume;

use audio::notifications;

fn help() -> ! {
    let name = args().nth(0).unwrap_or_else(|| {
        "pleb_ui".to_string()
    });
    println!(r#"{0} USAGE
    {0} brightness {{up|down|set}} <percent>
    {0}            get

    {0} volume {{up|down|set}} <percent>
    {0} volume toggle-mute

    {0} {{-h|--help}}"#, name);
    exit(255);
}

fn main() {
    match args().nth(1).as_ref().map(|s| s as &str) {
        Some("volume") => set_volume(),
        Some("brightness") => set_brightness(),
        _ => help(),
    };
}

fn set_volume() {
    let mut args = args().skip(2);

    let (mult, set, toggle_mute) = if let Some(arg) = args.next() {
        if arg == "up" {
            (1.0, false, false)
        } else if arg == "down" {
            (-1.0, false, false)
        } else if arg == "set" {
            (1.0, true, false)
        } else if arg == "toggle-mute" {
            (1.0, false, true)
        } else {
            help();
        }
    } else {
        help();
    };

    let master = match Mixer::new("default", "Master") {
        Ok(m) => m,
        Err(e) => {
            writeln!(stderr(), "There was an error opening the alsa mixer: {:?}", e).ok();
            exit(1)
        },
    };

    if toggle_mute {
        if master.toggle_mute().is_err() {
            writeln!(stderr(), "This mixer cannot be muted / unmuted!").ok();
        }
    } else {
        let percent: f32 = args.next()
            .and_then(|p| p.parse().ok())
            .unwrap_or_else(|| {
                help()
            });
        if set {
            master.set_volume(percent / 100.0);
        } else {
            master.change_volume_clip(percent * mult / 100.0);
        }
    }

    let vol_status = if master.is_muted() {
        Volume::Muted
    } else {
        Volume::Percent((master.volume() * 100.0) as u32)
    };

    if let Err(e) = volume::show_volume(vol_status) {
        writeln!(stderr(), "Error showing volume notification: {}", e).ok();
    }
    if let Err(_) = notifications::volume_change() {
        writeln!(stderr(), "Could not find volume change audio clip").ok();
    }
}

fn set_brightness() {
    let mut args = args().skip(2);

    let command = args.next();

    let percent: Option<f64> = args.next().and_then(|p| p.parse().ok());

    let bright_control = XcbBrightness::connect();

    let result = match (command.as_ref().map(|a| a as &str), percent) {
        (Some("up"),   Some(p)) => bright_control.change_n_clip(p),
        (Some("down"), Some(p)) => bright_control.change_n_clip(-1.0 * p),
        (Some("set"),  Some(p)) => bright_control.set(p),
        (Some("get"),  None)    => {
            let current = match bright_control.current() {
                Ok(c) => c,
                Err(e) => {
                    writeln!(stderr(), "Could not get brightness: {}", e).ok();
                    exit(3)
                },
            };
            println!("{}", current);
            return;
        },
        _ => help(),
    };

    if let Err(e) = result {
        writeln!(stderr(), "Error during operation: {}", e).ok();
    }

    let current = match bright_control.current() {
        Ok(c) => c as u32,
        Err(e) => {
            writeln!(stderr(), "Could not get brightness: {}", e).ok();
            exit(4)
        },
    };

    if let Err(e) = notify::brightness::show_brightness(current) {
        writeln!(stderr(), "Error showing volume notification: {}", e).ok();
    }
}
