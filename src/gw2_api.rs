use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::time::{Duration, Instant};

use crate::finished_events::set_event_finished;
use crate::time_utils::get_current_unix_time;

const WORLDBOSSES_URL: &str = "https://api.guildwars2.com/v2/account/worldbosses";
const MAPCHESTS_URL: &str = "https://api.guildwars2.com/v2/account/mapchests";

/// Maps a GW2 API world-boss id (from /v2/account/worldbosses) to the
/// (track, event) pair it represents in event_tracks.json.
const WORLD_BOSS_EVENTS: &[(&str, &str, &str)] = &[
    ("admiral_taidha_covington", "World Bosses", "Admiral Taidha Covington"),
    ("svanir_shaman_chief", "World Bosses", "Svanir Shaman Chief"),
    ("megadestroyer", "World Bosses", "Megadestroyer"),
    ("shadow_behemoth", "World Bosses", "Shadow Behemoth"),
    ("the_shatterer", "World Bosses", "The Shatterer"),
    ("great_jungle_wurm", "World Bosses", "Great Jungle Wurm"),
    ("modniir_ulgoth", "World Bosses", "Modniir Ulgoth"),
    ("fire_elemental", "World Bosses", "Fire Elemental"),
    ("inquest_golem_mark_ii", "World Bosses", "Golem Mark II"),
    ("claw_of_jormag", "World Bosses", "Claw of Jormag"),
    ("karka_queen", "Hard World Bosses", "Karka Queen"),
    ("tequatl_the_sunless", "Hard World Bosses", "Tequatl"),
    ("triple_trouble_wurm", "Hard World Bosses", "Triple Trouble"),
    ("drakkar", "Bjora Marches", "Champion of the Ice Dragon"),
    ("mists_and_monsters_titans", "Janthir Syntri", "Of Mists and Monsters"),
];

/// Maps a GW2 API map-chest id (from /v2/account/mapchests) to the specific
/// (track, event) that grants it. Most maps run several sub-events per meta
/// cycle, but only one is the actual chest trigger; these were pinned down by
/// cross-referencing each chest's known reward-cycle timing against the wiki
/// page for each candidate sub-event (matching UTC schedules + wiki links from
/// third-party GW2 event trackers). Entries marked "best guess" didn't have a
/// confirmed timing match and should be treated as unverified.
const MAP_CHEST_EVENTS: &[(&str, &str, &str)] = &[
    ("amnytas_heros_choice_chest", "Amnytas", "The Defense of Amnytas"),
    ("auric_basin_heros_choice_chest", "Auric Basin", "Battle in Tarir"),
    // Convergences rotate daily between two maps; only one is active per day,
    // so marking both under the shared chest id is harmless, not a guess.
    ("convergence_heros_choice_chest", "Convergences", "Outer Nayos"),
    ("convergence_heros_choice_chest", "Convergences", "Mount Balrior"),
    ("crystal_oasis_heros_choice_chest", "Crystal Oasis", "Casino Blitz"),
    ("domain_of_vabbi_heros_choice_chest", "Domain of Vabbi", "Forged with Fire"),
    // best guess: not timing-confirmed
    ("dragons_end_heros_choice_chest", "Dragon's End", "The Battle for the Jade Sea"),
    ("dragons_stand_heros_choice_chest", "Dragon's Stand", "Advancing on the Blighting Towers"),
    // best guess: not timing-confirmed
    ("echovald_wilds_heros_choice_chest", "The Echovald Wilds", "The Gang War of Echovald"),
    // best guess: not timing-confirmed
    ("elon_riverlands_heros_choice_chest", "Elon Riverlands", "Doppelganger"),
    ("new_kaineng_city_heros_choice_chest", "New Kaineng City", "Kaineng Blackout"),
    ("seitung_province_heros_choice_chest", "Seitung Province", "Aetherblade Assault"),
    ("skywatch_archipelago_heros_choice_chest", "Skywatch Archipelago", "Unlocking the Wizard's Tower"),
    ("tangled_depths_heros_choice_chest", "Tangled Depths", "King of the Jungle"),
    // best guess: not timing-confirmed
    ("the_desolation_heros_choice_chest", "The Desolation", "Maws of Torment"),
    ("verdant_brink_heros_choice_chest", "Verdant Brink", "Night Bosses"),
    // Not currently tracked by this addon (no matching map track exists yet):
    // citadel_of_zakiros_heros_choice_chest, gyala_delve_heros_choice_chest,
    // inner_nayos_heros_choice_chest, wild_island_heros_choice_chest
];

/// Human-readable result of the most recent sync attempt, shown in Settings.
pub static SYNC_STATUS: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static LAST_AUTO_SYNC: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));

/// Called periodically from the background thread; only performs an actual
/// API sync once `interval` has elapsed since the last automatic attempt.
pub fn maybe_auto_sync(api_key: &str, interval: Duration) {
    if api_key.trim().is_empty() {
        return;
    }

    let mut last = LAST_AUTO_SYNC.lock();
    let due = match *last {
        Some(t) => t.elapsed() >= interval,
        None => true,
    };
    if !due {
        return;
    }
    *last = Some(Instant::now());
    drop(last);

    sync_blocking(api_key);
}

/// Spawns a background thread to sync immediately, bypassing the auto-sync interval.
pub fn sync_now(api_key: String) {
    std::thread::spawn(move || {
        sync_blocking(&api_key);
    });
}

fn sync_blocking(api_key: &str) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            nexus::log::log(
                nexus::log::LogLevel::Critical,
                "Event Timers",
                &format!("Failed to create Tokio runtime for GW2 API sync: {}", e),
            );
            return;
        }
    };

    runtime.block_on(async {
        let client = reqwest::Client::new();
        let bosses = fetch_ids(&client, WORLDBOSSES_URL, api_key).await;
        let chests = fetch_ids(&client, MAPCHESTS_URL, api_key).await;

        let (bosses, chests) = match (bosses, chests) {
            (Ok(b), Ok(c)) => (b, c),
            (Err(e), _) | (_, Err(e)) => {
                let msg = format!("GW2 API sync failed: {}", e);
                nexus::log::log(nexus::log::LogLevel::Warning, "Event Timers", &msg);
                *SYNC_STATUS.lock() = msg;
                return;
            }
        };

        let now = get_current_unix_time();
        let mut marked = 0;

        for id in &bosses {
            for &(api_id, track, event) in WORLD_BOSS_EVENTS {
                if id.as_str() == api_id {
                    set_event_finished(track, event, now, now);
                    marked += 1;
                }
            }
        }

        for id in &chests {
            for &(api_id, track, event) in MAP_CHEST_EVENTS {
                if id.as_str() == api_id {
                    set_event_finished(track, event, now, now);
                    marked += 1;
                }
            }
        }

        let msg = format!(
            "GW2 API sync OK: {} boss(es), {} chest(s) reported by API, {} event(s) marked finished.",
            bosses.len(),
            chests.len(),
            marked
        );
        nexus::log::log(nexus::log::LogLevel::Info, "Event Timers", &msg);
        *SYNC_STATUS.lock() = msg;
    });
}

async fn fetch_ids(
    client: &reqwest::Client,
    url: &str,
    api_key: &str,
) -> Result<Vec<String>, String> {
    let response = client
        .get(url)
        .query(&[("access_token", api_key)])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("{} - {}", status, body));
    }

    response
        .json::<Vec<String>>()
        .await
        .map_err(|e| e.to_string())
}
