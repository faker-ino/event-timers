use nexus::data_link::mumble::{get_mumble_link, map_type, UiState};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::time::{Duration, Instant};

/// Whether the game currently reports being in a competitive mode (PvP or WvW).
/// Returns `false` if the Mumble link isn't available (e.g. game not running).
pub fn is_in_competitive_mode() -> bool {
    get_mumble_link()
        .map(|link| link.read_ui_state().contains(UiState::IS_IN_COMPETITIVE_MODE))
        .unwrap_or(false)
}

/// Whether the player is currently on one of the maps the viewer should always
/// be hidden on: Obsidian Sanctum, Edge of the Mists, or Armistice Bastion.
pub fn is_in_hidden_map() -> bool {
    get_mumble_link()
        .map(|link| {
            matches!(
                link.read_map_type(),
                map_type::WVW_OBSIDIAN_SANCTUM | map_type::WVW_EDGE_OF_THE_MISTS | map_type::WVW_LOUNGE
            )
        })
        .unwrap_or(false)
}

/// How long Mumble's `ui_tick` must stay unchanged before we consider the
/// game to be on a loading screen (the game stops updating Mumble Link while
/// a map is loading, so the tick visibly stalls).
const LOADING_STALL_THRESHOLD: Duration = Duration::from_millis(500);

struct TickWatch {
    last_tick: u32,
    since: Instant,
}

static TICK_WATCH: Lazy<Mutex<Option<TickWatch>>> = Lazy::new(|| Mutex::new(None));

/// Whether the game is currently sitting on the login/character-select
/// screen or a map loading screen.
///
/// Detected purely from Mumble Link, so it self-corrects every frame and
/// can never get stuck hidden: `map_id == 0` catches character select,
/// and a stalled `ui_tick` (the game stops pushing Mumble updates while a
/// map loads) catches loading screens.
pub fn is_in_loading_or_login_screen() -> bool {
    let Some(link) = get_mumble_link() else {
        return false;
    };

    if link.read_map_id() == 0 {
        return true;
    }

    let tick = link.read_ui_tick();
    let mut watch = TICK_WATCH.lock();
    match watch.as_mut() {
        Some(w) if w.last_tick == tick => w.since.elapsed() >= LOADING_STALL_THRESHOLD,
        _ => {
            *watch = Some(TickWatch {
                last_tick: tick,
                since: Instant::now(),
            });
            false
        }
    }
}
