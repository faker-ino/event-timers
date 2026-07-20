use nexus::data_link::mumble::{get_mumble_link, UiState};

/// Whether the game currently reports being in a competitive mode (PvP or WvW).
/// Returns `false` if the Mumble link isn't available (e.g. game not running).
pub fn is_in_competitive_mode() -> bool {
    get_mumble_link()
        .map(|link| link.read_ui_state().contains(UiState::IS_IN_COMPETITIVE_MODE))
        .unwrap_or(false)
}
