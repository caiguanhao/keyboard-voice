//! Reads the keys that egui/winit never reports — Print Screen, Scroll Lock,
//! Pause, Caps Lock, Num Lock, the Windows/Super key, and (on keyboards whose
//! firmware emits a scancode for it) Fn — straight from evdev on Linux.
//! evdev sits below X11 and Wayland, so this works in any desktop session.
//! Everywhere else this is a no-op stub.

use crate::keymap::KeySpec;
use eframe::egui::Context;
use std::sync::mpsc::Receiver;

pub struct SysKeys {
    receiver: Receiver<KeySpec>,
}

impl SysKeys {
    pub fn try_recv(&self) -> Option<KeySpec> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(not(target_os = "linux"))]
pub fn start(_ctx: Context) -> (SysKeys, Option<String>) {
    let (_, receiver) = std::sync::mpsc::channel();
    (SysKeys { receiver }, None)
}

#[cfg(target_os = "linux")]
pub use linux::start;

#[cfg(target_os = "linux")]
mod linux {
    use super::SysKeys;
    use crate::keymap::{system_key_spec, KeySpec, SystemKey};
    use eframe::egui::Context;
    use evdev::{Device, EventSummary, KeyCode};
    use std::{
        collections::HashSet,
        path::PathBuf,
        sync::{
            mpsc::{channel, Sender},
            Arc, Mutex,
        },
        thread,
        time::Duration,
    };

    const RESCAN_INTERVAL: Duration = Duration::from_secs(3);

    /// Keycodes this app can display. A device offering any of them is treated
    /// as a keyboard worth reading.
    const TARGET_KEYS: &[KeyCode] = &[
        KeyCode::KEY_SYSRQ,
        KeyCode::KEY_PRINT,
        KeyCode::KEY_SCROLLLOCK,
        KeyCode::KEY_PAUSE,
        KeyCode::KEY_CAPSLOCK,
        KeyCode::KEY_NUMLOCK,
        KeyCode::KEY_LEFTMETA,
        KeyCode::KEY_RIGHTMETA,
        KeyCode::KEY_FN,
    ];

    pub fn start(ctx: Context) -> (SysKeys, Option<String>) {
        let (sender, receiver) = channel::<KeySpec>();
        let opened = Arc::new(Mutex::new(HashSet::new()));
        let debug = std::env::var_os("KEYBOARD_DEBUG").is_some();
        let first_scan = scan(&sender, &opened, debug, &ctx);
        let warning = if first_scan == 0 {
            Some(
                "Special keys unavailable: cannot read /dev/input (add the user to the `input` group)"
                    .to_owned(),
            )
        } else {
            None
        };
        thread::Builder::new()
            .name("keyboard-voice-syskeys".into())
            .spawn(move || loop {
                thread::sleep(RESCAN_INTERVAL);
                scan(&sender, &opened, debug, &ctx);
            })
            .expect("spawn syskeys thread");
        (SysKeys { receiver }, warning)
    }

    /// Opens every not-yet-opened keyboard device and spawns a reader thread
    /// for it. Returns how many keyboards are open after the scan.
    fn scan(
        sender: &Sender<KeySpec>,
        opened: &Arc<Mutex<HashSet<PathBuf>>>,
        debug: bool,
        ctx: &Context,
    ) -> usize {
        let mut count = 0;
        for (path, device) in evdev::enumerate() {
            let has_target_key = device
                .supported_keys()
                .is_some_and(|keys| TARGET_KEYS.iter().any(|key| keys.contains(*key)));
            if !has_target_key {
                continue;
            }
            if !opened.lock().unwrap().insert(path.clone()) {
                count += 1; // already open
                continue;
            }
            if debug {
                eprintln!(
                    "keyboard-voice debug: reading special keys from {}",
                    path.display()
                );
            }
            let sender = sender.clone();
            let opened_clone = Arc::clone(opened);
            let reader_ctx = ctx.clone();
            let spawned = thread::Builder::new()
                .name(format!("keyboard-voice-syskeys-{}", path.display()))
                .spawn({
                    let path = path.clone();
                    move || {
                        read_device(device, &sender, debug, &reader_ctx);
                        opened_clone.lock().unwrap().remove(&path);
                    }
                });
            match spawned {
                Ok(_) => count += 1,
                Err(error) => {
                    opened.lock().unwrap().remove(&path);
                    eprintln!(
                        "keyboard-voice: could not spawn reader for {}: {error}",
                        path.display()
                    );
                }
            }
        }
        count
    }

    fn read_device(mut device: Device, sender: &Sender<KeySpec>, debug: bool, ctx: &Context) {
        loop {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        match event.destructure() {
                            EventSummary::Key(_, code, value) => {
                                if debug && value == 1 {
                                    eprintln!("keyboard-voice debug: key {code:?} pressed");
                                }
                                // Value 1 is a press; ignore releases (0) and repeats (2).
                                if value == 1 {
                                    if let Some(key) = system_key_for(code) {
                                        if sender.send(system_key_spec(key)).is_err() {
                                            return; // the app is gone
                                        }
                                        // evdev events don't wake egui on their own.
                                        ctx.request_repaint();
                                    }
                                }
                            }
                            // Scan events carry the raw scancode of unknown keys.
                            EventSummary::Misc(_, code, value) if debug => {
                                eprintln!("keyboard-voice debug: misc {code:?} value {value}");
                            }
                            _ => {}
                        }
                    }
                }
                // Unplugged or revoked; the rescan picks it up if it returns.
                Err(_) => return,
            }
        }
    }

    fn system_key_for(code: KeyCode) -> Option<SystemKey> {
        Some(match code {
            // Most keyboards report Print Screen as KEY_SYSRQ; some emit the
            // HID consumer-page "AC Print" code instead.
            KeyCode::KEY_SYSRQ | KeyCode::KEY_PRINT => SystemKey::PrintScreen,
            KeyCode::KEY_SCROLLLOCK => SystemKey::ScrollLock,
            KeyCode::KEY_PAUSE => SystemKey::Pause,
            KeyCode::KEY_CAPSLOCK => SystemKey::CapsLock,
            KeyCode::KEY_NUMLOCK => SystemKey::NumLock,
            KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA => SystemKey::Super,
            KeyCode::KEY_FN => SystemKey::Function,
            _ => return None,
        })
    }
}
