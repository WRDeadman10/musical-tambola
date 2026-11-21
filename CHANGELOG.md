# Changelog

All notable changes to the Media Tambola project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.3] - 2026-11-21

### Added
- **UI redesign**: new design tokens (surfaces, accent, gold), restyled buttons/cards/badges/tabs, collapsible icon sidebar, host cockpit (stage, tabs for Claim / Forecast / Projector / Lucky draw, always-visible action bar), redesigned projector scenes, light theme (applied at startup; the projector stays dark).
- **Simple ticket numbers**: tickets are numbered 1, 2, 3 per game; claims accept the number (old `T-000001` codes still work).
- **Win forecast** (admin only): which ticket wins which prize at which draw, with a per-ticket check.
- **Projector video follows the host**: the Audience Preview is the only video control; pause, play and seek are mirrored.
- **Settings**: "Reset settings" and "Delete all data" (typed confirmation, admin PIN, automatic backup).
- **Finished-game screen** with "Start a new game" and a winners link.

### Fixed
- Switching back to Classic from Song Grid failed; the Song Grid song count could not be changed.
- Every video preview failed (temp file extension) and videos with longer audio than video.
- Opening the projector froze the app (window created on the main thread).
- Event branding settings (accent colour, sounds) were saved under names the backend rejects and started blank.
- Sidebar kept the old language; the saved theme was not applied at startup.
- `get_game_win_forecast` and `reset_all_data` were missing from the permission lists.
- Printed ticket grid overflowed its template region and printed thumbnails behind the text.

## [1.0.2] - 2026-11-21

### Fixed
- **Language Switch Reactivity**:
  - Ensured translator function re-evaluates identity on language change so sidebar, settings, and host controls instantly update (`090c1a6`).
- **Owner Testing IPC Contracts**:
  - Repaired Song Grid row loading, history parameter serialization, and startup-settings retrieval contracts (`a85afd5`).

### Added
- **Host Control Panel Live Controls & Projector Scenes (Wave 5)**:
  - Real-time presentation reducer, countdown ticker, and reactive projector scenes (`d782f00`, `9000507`, `bf51666`).
- **FFmpeg Preview Engine & Download Manager (BT-35)**:
  - Local and PATH FFmpeg detection.
  - Pinned Gyan Essentials download streaming with SHA-256 validation, 150MB zip cap, and zip slip protection.
  - Video preview poster generation (2 workers, midpoint frame, DB mutex non-blocking, poster progress events) (`e44ad84`, `75699cf`).
- **Ticket Codes & Template Printing (BT-38)**:
  - Formatted sequential ticket code (`T-{:06}`) per ticket with prominent display.
  - PrintableTicket 2:1 template layout with dynamic media title loading and `@media print` A4 pagination (`5d70211`).
- **Presentation System Hardening (BT-36)**:
  - Default hidden song titles on game start and draw transitions.
  - Presentation splash mode support.
  - Event-driven presentation lifecycle notifications (`presentation-status-changed`).
  - Auto-open presentation window on multi-monitor setups (`75699cf`).
- **Game Screen Viewport Fit & Form Polish (BT-39)**:
  - 3-column scrollable host layout with live status and compact song strip.
  - Media Library slot count synchronized with active game format (`b84fa03`).

## [1.0.1] - 2026-11-21

### Added
- **Game Formats & Song Grid (BT-20 / BT-29)**:
  - Added support for `SONG_GRID` mode (3×5 grid with 15 numbers, 20–45 configurable song counts) alongside `CLASSIC_90` (3×9 grid).
  - Format-specific ticket generation, monotonic codes, layout hashing, and batch badges.
- **Pattern Engine & Custom Schemes (BT-21 / BT-28)**:
  - 3×5 pattern mask evaluators for customizable winning combinations.
  - Preset gallery including Pattern H, Photo Frame, Plus, V, A, Breakfast, Top/Middle/Bottom Line, Four Corners, and Early N (5, 7, 10).
  - Support for multiple Full House tiers with `is_full_house` exclusion flags.
- **Dual-Window Presentation System (BT-26 / BT-23 / U1)**:
  - Dedicated fullscreen Presentation window for secondary displays / projectors.
  - Multi-monitor auto-detection with recommended projector prompt and display persistence.
  - Read-only ACL capability isolation (`capabilities/presentation.json`) with `require_main_window` defence-in-depth on all mutating backend commands.
  - Projector scenes: idle poster, animated media playback, answer banner, number badge, ticker strip, and prize announcements.
- **Media Folder Import & Auto Posters (BT-25 / BT-24)**:
  - Automated directory scanning (ASC, DESC, Number, Random) with 5,000-file safety cap, symlink traversal prevention, and magic-byte sniff validation.
  - Auto-generated video frame posters stored with SHA-8 hashing under `%APPDATA%/posters/`.
- **Ticket Printing & Visual Templates (BT-30 / W3-1)**:
  - Visual 1800×900 template canvas with user-uploaded background templates.
  - Aspect ratio (2:1), format (PNG/JPEG), and size (≤5 MB) validation with SHA-256 deduplicated storage.
  - Print preview, pagination, and range filtering.
- **Lucky Draw Engine (W3-2 / U2)**:
  - Autonomous Lucky Draw supporting both `NAMES` and `TICKETS` participant pools.
  - Cryptographically secure `OsRng` draw without replacement and toggleable exclusion of previous winners.
  - Projector winner announcement integration.
- **Event Branding & Procedural Sound Effects (W3-3 / U3)**:
  - Event branding settings for custom idle poster and live scrolling ticker banner.
  - 5 bundled procedural sound effects (`draw`, `drumroll`, `win`, `loss`, `tick`) with selectable host/projector audio routing.
- **Data Management, Safe Restore & Admin PIN (W3-4 / U4)**:
  - SQLite online backup API generating compressed zip archives containing `tambola.sqlite`, `manifest.json`, `posters/`, and `templates/`.
  - Zip bomb & path safety: 20,000 entry cap, 2 GB uncompressed cap, strict path allowlists rejecting traversal.
  - Atomic restore staging with `PRAGMA integrity_check`, WAL checkpointing (`PRAGMA wal_checkpoint(TRUNCATE)`), automated rollback snapshot (`pre_restore_<timestamp>/`), and restart marker.
  - Argon2id password hashing for Admin PIN with 5 attempts/minute rate limiting and 10-minute session unlock guarding setup modifications during active games.
- **Song Title Import (W3-5 / U5)**:
  - Universal spreadsheet & delimited file importer supporting CSV, XLSX, XLS, and ODS formats via `calamine 0.38` and `csv 1.3`.
  - Two-stage workflow: non-mutating preview with slot range, duplicate, and character limit validation plus `outOfFormatRange` warnings, followed by transactional apply.

### Changed
- Refactored Host Control Panel into responsive 3-column layout with quick-claim desk and countdown timers (BT-22).
- Upgraded acceptance test suite in `tests/acceptance.rs` to cover both Classic 90 and Song Grid end-to-end lifecycles.

## [1.0.0] - 2026-11-20

### Added
- Initial production release candidate of Media Tambola.
- Core 90-number draw engine with Fisher-Yates shuffle.
- Authoritative claim validation engine and winner allocation.
- Media manager with local and remote asset validation and custom `media://` streaming protocol.
- Bilingual English and Gujarati localized interface.
- SQLite embedded storage with versioned migrations and zero raw database error leaks.
- Windows NSIS installer bundle.
