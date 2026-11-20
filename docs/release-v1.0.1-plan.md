# Release v1.0.1 Plan — Feature parity with the client's current app + best-in-class host UX

Status: APPROVED by Owner (2026-11-20). Lead: Claude. Implementers: AG (backend / Rust), Codex (frontend).
Reference: the client's existing app ("Unique Digital Tambola / Musical Tambola", TIG branding) observed in the Owner's screen recording.

## 1. Goals

1. Every feature the client uses today exists in Media Tambola.
2. The host experience is dramatically easier than the reference app: one control panel, few clicks, big targets, keyboard-first, no hidden menus, nothing that can wreck a live game by accident.
3. The audience screen (projector) is separate from the host screen and always looks clean.

## 2. Owner decisions (binding)

| Topic | Decision |
|---|---|
| Ticket format | BOTH formats, chosen per game: **Classic 90** (3×9, 15 numbers, 1–90) and **Song Grid** (3×5, all 15 cells filled, N songs, e.g. 45). |
| Late claims | **Always rejected.** No host override. The host and claimant must be clearly told the claim is late, with the qualification draw and current draw shown. |
| Prize patterns | **Visual pattern editor** with presets (H, +, V, A, Photo Frame, Breakfast/Lunch/Dinner, Corners, Rows, Early N, Full House). The host can edit presets or create new patterns by clicking cells. |
| Language | One language at a time (English or Gujarati), switched in Settings. Everything new must be fully bilingual. |

## 3. Feature map (reference app → v1.0.1)

| Reference app feature | v1.0.1 deliverable | Owner |
|---|---|---|
| 45 songs, 3×5 tickets, song titles on tickets | Song Grid format: N configurable (15–90), 3×5 tickets, text-title ticket style | AG + Codex |
| Many prizes (Breakfast, Dinner, Early 5/7/10, H, +, V, A, Photo Frame, Corner, rows, multiple Full Houses) | Mask-based pattern engine + editor + presets; multiple Full House schemes (FH 1, FH 2 …) | AG + Codex |
| Prize announcement screen (pattern with ticks + big prize name) | Projector "Prize Announcement" scene, triggered from the host panel or automatically on an accepted claim | Codex |
| Game screen: title banner, number box, Q.NO, pause, countdown, Show Ans, Hide number | Projector scene with title banner (show/hide answer), number badge (toggle), draw counter, countdown timer; all driven from the host panel | AG (events) + Codex |
| Splash number card between songs ("43") | Optional reveal sequence: number card → media; per-game setting | Codex |
| Lucky-draw sound, random-number sound | Sound effects (draw, reveal, accepted, rejected, lucky draw) with volume and mute; bundled royalty-free sounds | AG (assets/settings) + Codex |
| Print Ticket, Print Preview, Direct Print | Ticket sheet printing: preview, N tickets per A4, branding header, both formats, print or save as PDF | Codex (+ AG data query) |
| Lucky Draw (names or ticket numbers, Big Lucky Draw) | Lucky Draw: pool from names list, ticket codes, or a CSV; animated draw on the projector; draw history saved | AG + Codex |
| Ticket search with prize select (Bogi Tickets) | Already in the Claim Desk; moves into the host panel as an inline quick-claim box | Codex |
| Folder of 01…NN media, prize images by name | Folder import: pick a folder and auto-map 01..NN files to slots; preview before applying | AG + Codex |
| Song/question list, Excel import | CSV/Excel (.xlsx) import of titles by number | AG + Codex |
| Main-screen poster, ticket-search background, themes, ticker, event name | Branding: event name, logo/poster for the idle screen, ticket-print header, scrolling ticker text, accent theme | AG + Codex |
| Restore Data, Password, Setting Admin | Backup / Restore (DB + media manifest) and an optional admin PIN that locks setup screens during a live game | AG + Codex |
| Previous Answer / Answer List | Already present (draw history); exposed in the host panel | Codex |
| KBC Rapid Round quiz | Deferred to v1.1 (separate game mode); confirm with Owner | — |
| Multi-monitor (projector) | Separate presentation window; host picks the display and it goes fullscreen there | AG + Codex |

## 4. UX principles for the Host Control Panel

- **One screen to run the whole event.** The left column is a small preview of what the audience sees. The centre has huge controls. The right column holds the quick claim box, winners per prize, and the recent draw list.
- **Primary action is always obvious.** NEXT is the largest button. Destructive actions (End Game) are small and separated, and always confirmed.
- **Keyboard first.**
  - Space/→ Next (never while typing)
  - P pause/resume
  - A show/hide answer
  - N toggle number
  - C focus the claim box
  - L lucky draw
  - F11 projector fullscreen
  - ? opens a shortcut cheat sheet
- **Live state everywhere.** Status chips show ACTIVE/PAUSED, draw x/N, countdown and projector connected.
- **No dead ends.** Every error says what to do next, in the selected language.
- **Projector stays clean.** No menus, cursors or admin chrome; smooth transitions; readable from the back of a hall (minimum 48px titles at 1080p).
- **Safe live operation.** While a game is ACTIVE, setup screens are read-only (or PIN-locked), so a mis-click cannot change media or prizes mid-game.

## 5. Work breakdown and order

Wave 1 (start now):
- BT-20 (AG) Game formats: CLASSIC_90 + SONG_GRID; songCount per format; Song Grid ticket generator; format snapshot on the game; media slot count follows the format.
- BT-22 (AG) Pattern engine: mask-based patterns, presets, multiple Full House schemes, claim integration, migrations.
- BT-21 (Codex) Host Control Panel redesign on the existing APIs (layout, shortcuts, quick claim, status chips).

Wave 2:
- BT-24 (AG) Presentation window + state/command events (show/hide answer, number, countdown, splash, announce prize, lucky draw), display selection.
- BT-23 (Codex) Projector scenes: idle poster, media + banner, number badge, countdown, splash, prize announcement, lucky draw animation, ticker.
- BT-25 (Codex) Pattern editor UI and the format choice in the New Game wizard / Media Library.

Wave 3:
- BT-26 (AG) Folder import, CSV/XLSX import, backup/restore, admin PIN, sound settings, lucky-draw backend, branding settings.
- BT-27 (Codex) Ticket printing, Lucky Draw UI, Branding UI, Import/Backup UI, sounds.

Wave 4:
- BT-28 (AG + Codex) Full regression: acceptance tests for both formats, Playwright flows, installer v1.0.1.

## 6. Non-negotiables carried from v1.0.0

- Late claims are always rejected (chronological draw position). Eligibility rules are unchanged; Full House is identified by an is_full_house flag on the pattern, never by its name.
- Snapshots are taken at game start: format, media, patterns and limits.
- All new behaviour has real tests; frontend tests must render components.
- English and Gujarati parity for every string.
