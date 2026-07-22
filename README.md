# Event-Timers

This is a personal fork of [qjv/event-timers](https://github.com/qjv/event-timers) by hvaren, extended by faker-ino.

## Changes in this fork

- **Mark events as finished** — right-click any event occurrence to mark it finished; it's shown grayed out with a red X. Marks automatically clear at GW2's daily reset (00:00 UTC).
- **GW2 API sync** — optionally enter an API key (needs only the "progression" permission) in Settings to automatically mark today's completed world bosses and map-chest metas as finished, so you don't have to mark them yourself. Syncs on load and every 5 minutes, or on demand via "Sync Now".
- **Hide event viewer in competitive modes** — setting to auto-hide the event timer window while in PvP/WvW, and also in Obsidian Sanctum, Edge of the Mists, and Armistice Bastion.
- **Hide event viewer on loading/login screens** — new setting (on by default) to auto-hide the window on the login/character-select screen and while a map is loading.
- **Auto-update disabled** — this build no longer checks the upstream repo for updates, so it won't be silently overwritten by upstream releases.



