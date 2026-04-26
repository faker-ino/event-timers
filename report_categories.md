# Report Categories

Batch triage date: 2026-04-26

## Feature Requests / Enhancements

| Priority | Request | User | Notes |
| --- | --- | --- | --- |
| P1 | Add actual time display to time interval markers | Unknown first user | Marker spacing works, but the user also wants "the time as well." Clarify whether this means labels like `13:00`, `14:00` on ruler interval markers. |
| P1 | Limit "happening now" when an event is too far in the past | QuantumOGRE | Current notification can show old events as "happening now." Add a max grace window or cutoff. |
| P2 | Add a favorite event feature | gartner | Right-click event, favorite it, and give all matching instances a golden border or similar visual highlight. |
| P2 | Highlight tracked or favorite items in the main window | gartner | Similar to the tracked-item mini-window, but visible directly in the main schedule list. |
| P2 | First-time instructional popup for event setup | You | One-time onboarding popup to teach users how to configure events. |
| P3 | Improve discoverability of tracked-items mini-window | Sakuna | After seeing a screenshot, asked how to set it up. The feature exists but is not obvious. |

## Bug Reports / Suspicious Behavior

| Priority | Issue | User | Notes |
| --- | --- | --- | --- |
| P1 | False or incorrect meta event shown for Tangled Depths | Sakuna | User said no meta was happening in Tangled Depths and shared a screenshot. Needs reproduction details. |
| P2 | "Happening now" covers events too far in the past | QuantumOGRE | Could be treated as either a bug or a UX logic flaw. |

## UX / Documentation Issues

| Priority | Issue | Evidence | Suggested Fix |
| --- | --- | --- | --- |
| P2 | Tracked-items setup is unclear | Sakuna asked how to set it up | Add a tooltip, help button, onboarding popup, or context-menu shortcut. |
| P2 | Difference between tracked items, favorites, highlights, and notifications is unclear | gartner was unsure whether favorites should be a tracked-item enhancement | Define these categories clearly in the UI and settings. |

## Suggested Tracker Buckets

### Timeline / Schedule Display

- Add clock labels to interval markers.
- Verify marker labels do not clutter dense layouts.

### Notification Logic

- Add max "happening now" grace period.
- Example setting: show "happening now" for `X` minutes after start.
- Consider separate handling for long-duration events and short events.

### Event Correctness

- Investigate Tangled Depths false positive.
- Needed context: screenshot time, selected filters, map/event source, timezone, and whether this was an API/data issue or local schedule calculation issue.

### Favorites / Tracking

- Add right-click "Favorite event."
- Apply a golden border or visual accent to all matching future instances.
- Decide whether favorites are separate from tracked items or tracked items with stronger visual styling.

### Onboarding / Discoverability

- Add first-run setup popup.
- Add a "How to track events" hint.
- Expose tracked-items window from context menu or settings.

## Practical Priority Order

1. Fix "happening now" cutoff.
2. Investigate Tangled Depths false meta.
3. Add time labels to interval markers.
4. Improve tracked-items discoverability.
5. Add favorite/highlight system.
6. Add first-run event setup popup.
