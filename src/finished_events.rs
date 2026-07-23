use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashSet;

use crate::config::TrackedEventId;

/// GW2's daily reset lands exactly on the UTC calendar day boundary (00:00 UTC),
/// which is also where Unix day numbers roll over, so no timezone math is needed.
fn reset_day(unix_time: i64) -> i64 {
    unix_time.div_euclid(86400)
}

/// Tracks which events the user has marked (manually or via API sync) as finished
/// for the current GW2 daily reset period. Rather than tagging each entry with the
/// day of the specific occurrence it was marked against (fragile: depends on every
/// caller recomputing that occurrence's start time correctly), the whole set is
/// wiped the moment real wall-clock time crosses into a new reset day. This is
/// simpler and self-heals any stale entries left over from a previous bug.
#[derive(Debug, Default)]
pub struct FinishedEventsState {
    finished: HashSet<TrackedEventId>,
    last_reset_day: Option<i64>,
}

impl FinishedEventsState {
    /// Clears all markers if real time has moved past the reset day they were set on.
    fn roll_reset(&mut self, current_time: i64) {
        let today = reset_day(current_time);
        if self.last_reset_day != Some(today) {
            self.finished.clear();
            self.last_reset_day = Some(today);
        }
    }
}

pub static FINISHED_EVENTS: Lazy<Mutex<FinishedEventsState>> =
    Lazy::new(|| Mutex::new(FinishedEventsState::default()));

pub fn is_event_finished(track_name: &str, event_name: &str, current_time: i64) -> bool {
    let event_id = TrackedEventId::new(track_name, event_name);
    let mut state = FINISHED_EVENTS.lock();
    state.roll_reset(current_time);
    state.finished.contains(&event_id)
}

/// Marks an event finished for today, without toggling it back off if it's
/// already marked (used for API-driven auto-marking).
pub fn set_event_finished(track_name: &str, event_name: &str, current_time: i64) {
    let event_id = TrackedEventId::new(track_name, event_name);
    let mut state = FINISHED_EVENTS.lock();
    state.roll_reset(current_time);
    state.finished.insert(event_id);
}

pub fn toggle_event_finished(track_name: &str, event_name: &str, current_time: i64) {
    let event_id = TrackedEventId::new(track_name, event_name);
    let mut state = FINISHED_EVENTS.lock();
    state.roll_reset(current_time);
    if !state.finished.remove(&event_id) {
        state.finished.insert(event_id);
    }
}

/// Snapshot of currently-finished events for persisting to disk, alongside the
/// reset day they belong to (so a stale snapshot can be recognized on load).
pub fn export_finished(current_time: i64) -> (HashSet<TrackedEventId>, i64) {
    let mut state = FINISHED_EVENTS.lock();
    state.roll_reset(current_time);
    (state.finished.clone(), state.last_reset_day.unwrap_or(reset_day(current_time)))
}

/// Restores finished-event markers loaded from disk, discarding them outright
/// if they were saved on a prior reset day.
pub fn import_finished(entries: HashSet<TrackedEventId>, saved_day: i64, current_time: i64) {
    let mut state = FINISHED_EVENTS.lock();
    let today = reset_day(current_time);
    state.finished = if saved_day == today { entries } else { HashSet::new() };
    state.last_reset_day = Some(today);
}
