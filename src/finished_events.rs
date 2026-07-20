use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashSet;

use crate::config::TrackedEventId;

/// GW2's daily reset lands exactly on the UTC calendar day boundary (00:00 UTC),
/// which is also where Unix day numbers roll over, so no timezone math is needed.
fn reset_day(unix_time: i64) -> i64 {
    unix_time.div_euclid(86400)
}

/// Identifies one occurrence of a recurring event, by which GW2 reset-day it falls on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FinishedOccurrence {
    event_id: TrackedEventId,
    day: i64,
}

/// Tracks which events the user has manually marked as finished for the current
/// GW2 daily reset period. Session-only: intentionally not persisted, since entries
/// are only meaningful until the next reset.
#[derive(Debug, Default)]
pub struct FinishedEventsState {
    finished: HashSet<FinishedOccurrence>,
}

impl FinishedEventsState {
    /// Drop markers from days before the current GW2 reset day
    fn cleanup_old(&mut self, current_time: i64) {
        let today = reset_day(current_time);
        self.finished.retain(|o| o.day >= today);
    }
}

pub static FINISHED_EVENTS: Lazy<Mutex<FinishedEventsState>> =
    Lazy::new(|| Mutex::new(FinishedEventsState::default()));

pub fn is_event_finished(track_name: &str, event_name: &str, start_time: i64) -> bool {
    let event_id = TrackedEventId::new(track_name, event_name);
    let day = reset_day(start_time);
    FINISHED_EVENTS
        .lock()
        .finished
        .contains(&FinishedOccurrence { event_id, day })
}

pub fn toggle_event_finished(
    track_name: &str,
    event_name: &str,
    start_time: i64,
    current_time: i64,
) {
    let event_id = TrackedEventId::new(track_name, event_name);
    let mut state = FINISHED_EVENTS.lock();
    state.cleanup_old(current_time);

    let key = FinishedOccurrence { event_id, day: reset_day(start_time) };
    if !state.finished.remove(&key) {
        state.finished.insert(key);
    }
}
