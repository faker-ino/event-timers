// Add to lib.rs

use nexus::{
    gui::{register_render, render, RenderType},
    keybind::register_keybind_with_string,
    quick_access::{add_quick_access, remove_quick_access},
    texture::load_texture_from_memory,
    AddonFlags,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::ffi::c_char;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

mod config;
mod finished_events;
mod game_state;
mod json_loader;
mod notification_logic;
mod notifications;
mod time_utils;
mod ui;

use config::{load_user_config, save_user_config, RUNTIME_CONFIG};
use notification_logic::update_notifications;
use ui::{
    check_for_event_tracks_update, render_main_window, render_settings,
    render_toast_notifications, render_upcoming_panel,
};

// Embed icon files directly in the binary
const QA_ICON: &[u8] = include_bytes!("../qa_icon.png");
const QA_ICON_HOVER: &[u8] = include_bytes!("../qa_icon_hovered.png");
const NOTIFICATION_TICK_ACTIVE_MS: u64 = 250;
const NOTIFICATION_TICK_IDLE_MS: u64 = 1500;
const QUICK_ACCESS_ID: &str = "EVENT_TIMERS_QA";

static BG_STOP: AtomicBool = AtomicBool::new(false);
static BG_THREAD: Lazy<Mutex<Option<thread::JoinHandle<()>>>> = Lazy::new(|| Mutex::new(None));
// Tracks whether the quick access shortcut is currently registered with Nexus,
// so its visibility can be toggled live from settings / competitive-mode state.
static QUICK_ACCESS_ADDED: AtomicBool = AtomicBool::new(false);

extern "C-unwind" fn toggle_window_keybind(_identifier: *const c_char, is_release: bool) {
    if !is_release {
        let mut config = RUNTIME_CONFIG.lock();
        config.show_main_window = !config.show_main_window;
    }
}

extern "C-unwind" fn toggle_toasts_keybind(_identifier: *const c_char, is_release: bool) {
    if !is_release {
        let mut config = RUNTIME_CONFIG.lock();
        config.notification_config.toast_enabled = !config.notification_config.toast_enabled;
    }
}

extern "C-unwind" fn toggle_upcoming_panel_keybind(_identifier: *const c_char, is_release: bool) {
    if !is_release {
        let mut config = RUNTIME_CONFIG.lock();
        config.notification_config.upcoming_panel_enabled = !config.notification_config.upcoming_panel_enabled;
    }
}

nexus::export! {
    name: "Event Timers",
    signature: -0x45564E54,
    load,
    unload,
    flags: AddonFlags::None,
}

fn load() {
    load_user_config();
    BG_STOP.store(false, Ordering::Relaxed);
    
    // Check for event_tracks.json updates on load
    check_for_event_tracks_update();
    
    // Load quick access textures once; the shortcut itself is added/removed
    // reactively every frame by sync_quick_access_visibility().
    load_texture_from_memory("EVENT_TIMERS_QA_ICON", QA_ICON, None);
    load_texture_from_memory("EVENT_TIMERS_QA_ICON_HOVER", QA_ICON_HOVER, None);

    register_keybind_with_string("Toggle Event Timers", toggle_window_keybind, "ALT+E")
        .revert_on_unload();

    register_keybind_with_string("Toggle Toast Notifications", toggle_toasts_keybind, "")
        .revert_on_unload();

    register_keybind_with_string("Toggle Upcoming Panel", toggle_upcoming_panel_keybind, "")
        .revert_on_unload();

    register_render(RenderType::Render, render!(|ui| {
        sync_quick_access_visibility();
        render_main_window(ui);
        let (toast_enabled, upcoming_enabled) = {
            let config = RUNTIME_CONFIG.lock();
            (
                config.notification_config.toast_enabled,
                config.notification_config.upcoming_panel_enabled,
            )
        };
        if toast_enabled {
            render_toast_notifications(ui);
        }
        if upcoming_enabled {
            render_upcoming_panel(ui);
        }
    }))
    .revert_on_unload();
    
    register_render(RenderType::OptionsRender, render!(|ui| {
        render_settings(ui);
    }))
    .revert_on_unload();

    let handle = thread::Builder::new()
        .name("event-timers-notifications".to_string())
        .spawn(|| {
            while !BG_STOP.load(Ordering::Relaxed) {
                let (has_targets, any_surface_enabled) = {
                    let config = RUNTIME_CONFIG.lock();
                    let has_targets =
                        !config.tracked_events.is_empty() || !config.oneshot_events.is_empty();
                    let any_surface_enabled = config.notification_config.toast_enabled
                        || config.notification_config.upcoming_panel_enabled
                        || config.show_main_window;
                    (has_targets, any_surface_enabled)
                };
                if has_targets && any_surface_enabled {
                    update_notifications();
                }
                let sleep_ms = if has_targets && any_surface_enabled {
                    NOTIFICATION_TICK_ACTIVE_MS
                } else {
                    NOTIFICATION_TICK_IDLE_MS
                };
                thread::sleep(Duration::from_millis(sleep_ms));
            }
        });
    if let Ok(h) = handle {
        *BG_THREAD.lock() = Some(h);
    }
}

/// Adds or removes the quick access shortcut to match current settings.
/// Called every frame so toggling "Show quick access icon" takes effect live.
fn sync_quick_access_visibility() {
    let desired = RUNTIME_CONFIG.lock().show_quick_access_icon;
    let currently_added = QUICK_ACCESS_ADDED.load(Ordering::Relaxed);

    if desired == currently_added {
        return;
    }

    if desired {
        add_quick_access(
            QUICK_ACCESS_ID,
            "EVENT_TIMERS_QA_ICON",
            "EVENT_TIMERS_QA_ICON_HOVER",
            "Toggle Event Timers",
            "Toggle Event Timers Window",
        )
        .leak();
    } else {
        remove_quick_access(QUICK_ACCESS_ID);
    }
    QUICK_ACCESS_ADDED.store(desired, Ordering::Relaxed);
}

fn unload() {
    BG_STOP.store(true, Ordering::Relaxed);
    if let Some(h) = BG_THREAD.lock().take() {
        let _ = h.join();
    }
    if QUICK_ACCESS_ADDED.swap(false, Ordering::Relaxed) {
        remove_quick_access(QUICK_ACCESS_ID);
    }
    save_user_config();
}
