# Open Issues Digest (Verified Audit Findings & Real App Bugs)

This document records the actual risks, code debt, and production bugs identified across audits and WebView2 real-app testing. All file references and line numbers are verified against the repository tree.

---

## 1. Verified Real App Bugs (Found in 67-Video Test & Remediated)

The following six bugs were detected during real-application end-to-end execution against `F:\Videos` and WebView2 CDP automation, and have all been fixed with regression tests:

| ID | Location | Summary | Resolution Status |
| :--- | :--- | :--- | :--- |
| **BUG-01** | `src-tauri/src/domain/format.rs:118` | `GameFormat::Classic90` serialized as `CLASSIC90`, causing deserialization failure `"unknown variant CLASSIC_90"`. | **RESOLVED**: Fixed serde rename attribute and added roundtrip tests in `format.rs` and `serde_wire_names.rs`. |
| **BUG-02** | `src/features/settings/SettingsPage.tsx` | Changing song count sent previous count due to React state closure stale binding. | **RESOLVED**: Fixed input change handler state capture. |
| **BUG-03** | `src-tauri/src/services/ffmpeg.rs:652` | Temp file lacked `.jpg` extension, causing FFmpeg to fail format deduction, plus audio longer than video failed seek. | **RESOLVED**: Added `.jpg` suffix to temp file and earlier frame fallback. |
| **BUG-04** | `src-tauri/src/commands/presentation.rs:17` | `open_presentation` ran synchronously on the main thread, deadlocking all IPC while window initialized. | **RESOLVED**: Converted to async command and spawned auto-open on background worker thread. |
| **BUG-05** | `src/components/layout/Sidebar.tsx` | Language change event failed to trigger sidebar re-render. | **RESOLVED**: Subscribed sidebar and translated views to i18n instance identity change. |
| **BUG-06** | `src/components/layout/Sidebar.tsx:112` | Sidebar footer displayed hardcoded string `v0.1.0`. | **RESOLVED**: Replaced with dynamic application version string. |

---

## 2. Verified Architectural & Code Risks (With Real Source Paths)

| ID | Category | Verified File & Line | Severity | Description | Remediation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **AUD-01** | Backend | `src-tauri/src/services/ffmpeg.rs:20-21`, `358-366` | **RISK** | `PINNED_FFMPEG_SHA256` is hardcoded to a specific Gyan essentials build hash. Upstream rebuilds or Gyan mirror rotations will cause `FFMPEG_CHECKSUM_MISMATCH`. | Download `.zip.sha256` dynamically alongside archive or offer user confirmation on verified official domain. |
| **AUD-02** | Backend | `src-tauri/src/db/repositories/ticket.rs:24-28` | **RISK** | `generate_batch` accepts up to `10_000` tickets synchronously in SQLite transaction, which causes momentary main thread blockage and memory spikes. | Chunk ticket inserts into transactions of 500 tickets with event progress streaming. |
| **AUD-03** | Frontend Glue | `src/features/game-controller/HostProjectorCard.tsx:76-91` | **RISK** | `listenToPresentationAutoplayBlocked` and `listenToPresentationSuggested` call `then()` to store cleanup closures, but unmounting before the promise resolves can leave dangling listeners. | Track `isMounted` ref flag or invoke `cleanup()` immediately if unmounted before promise resolution (matching pattern in `useMediaLibrary.ts:68-74`). |
| **AUD-04** | Localization | `src/lib/api/errors.ts:8-13` | **RISK** | Error codes added in Waves 3-5 (`ADMIN_PIN_REQUIRED`, `ADMIN_PIN_RATE_LIMITED`, `FFMPEG_DOWNLOAD_FAILED`, `FFMPEG_CHECKSUM_MISMATCH`, `CANNOT_RESTORE_ACTIVE_GAME`) lack entries in `src/locales/en.json::errors` and fall back to raw English Rust error strings. | Add localized entries under `errors.*` in `en.json` and `gu.json`. |
| **AUD-05** | UI Accessibility | `src/features/game-controller/QuickClaim.tsx:34-38` | **RISK** | Combobox `<select>` dropdown for pattern scheme selection lacked an explicit `aria-label` or `<label>` element. | **RESOLVED**: Added `aria-label={t("claims.selectScheme") ?? "Select scheme"}` and verified key parity. |

---

## 3. Verification Standards
Every finding in this digest has been cross-checked by direct file inspection (`Test-Path`, `Get-Content`). Future audit batches must verify line numbers directly from the tree before submission.
