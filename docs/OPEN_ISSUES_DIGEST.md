# Open Issues Digest (Batches 2-5 Audit Findings)

This document summarizes every **BUG** and **RISK** identified during the systematic source code audits conducted in Batches 2 through 5.

---

## 1. Summary Matrix

| ID | Category | Batch | Location | Severity | Description | Status / Remediation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **AUD-01** | Backend | Batch 2 | `services/ffmpeg_downloader.rs:188` | **RISK** | SHA256 checksum verification fallback when Gyan build mirrors change hashes. | Fallback gracefully prompts manual file selection or retry. |
| **AUD-02** | Backend | Batch 3 | `domain/ticket_generation.rs:142` | **RISK** | Upper bound generation limit on massive ticket batches (>10,000) leading to memory spikes. | Bounded by UI and SQLite batch inserts. |
| **AUD-03** | Frontend Glue | Batch 4 | `src/features/claims/useClaims.ts:44` | **RISK** | Tauris `listen()` returns `Promise<UnlistenFn>`. Quick unmount before promise resolution can leave a dangling listener. | Remediated with `isMounted` ref check before cleanup. |
| **AUD-04** | Localization | Batch 4 | `src/locales/en.json` & `gu.json` | **RISK** | Wave 3–5 backend error codes (`ADMIN_PIN_REQUIRED`, `FFMPEG_CHECKSUM_MISMATCH`, etc.) lack dedicated localized string entries and fall back to English API errors. | Remediate in Batch 10 or UI error handler table. |
| **AUD-05** | UI Accessibility | Batch 5 | `src/features/claims/QuickClaim.tsx:35` | **RISK** | `<select>` scheme dropdown lacks an explicit `aria-label` or `<label htmlFor="...">`, unlike ticket input. | Add `aria-label="Select winning pattern"` to `<select>`. |

---

## 2. Detailed Findings & Recommendations

### AUD-01: FFmpeg Checksum Verification Robustness
- **Finding**: `ffmpeg_downloader.rs` verifies SHA256 against hardcoded Gyan release hashes. When Gyan updates daily or essentials builds, SHA mismatches could block download.
- **Remediation**: Allow user override with explicit prompt if SHA mismatch occurs on verified official domains, or download the checksum file dynamically from `gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip.sha256`.

### AUD-02: Massive Ticket Batch Memory Footprint
- **Finding**: Generating large ticket ranges (>10,000) in a single synchronous IPC command can cause high heap allocation and momentary main-thread stutter.
- **Remediation**: Chunk generation into 500-ticket batches or stream progress via Tauri events.

### AUD-03: Tauri Event Listener Promise Cleanup
- **Finding**: In React hooks using Tauri's `listen(event, handler)`, component unmounting before the promise resolves can bypass `unlisten()`.
- **Remediation**:
  ```typescript
  let unlisten: (() => void) | null = null;
  let isMounted: boolean = true;
  listen('event', cb).then(fn => {
    if (isMounted) { unlisten = fn; } else { fn(); }
  });
  return () => { isMounted = false; unlisten?.(); };
  ```

### AUD-04: Localization Parity for Wave 3-5 Error Enums
- **Finding**: While key parity between `en.json` and `gu.json` is 100% (499 keys), backend error enums introduced in Wave 3 to 5 (e.g., `ADMIN_PIN_REQUIRED`, `FFMPEG_DOWNLOAD_FAILED`) are currently displayed using raw backend message strings rather than mapped i18n keys.
- **Remediation**: Add error message dictionary mappings in `src/lib/api/errorMapping.ts` linking backend error codes to `i18n.t('errors.<CODE>')`.

### AUD-05: QuickClaim Dropdown Accessibility
- **Finding**: `QuickClaim.tsx` `<select>` input does not declare an `aria-label`, tripping strict screen-reader accessibility audits.
- **Remediation**: Insert `aria-label="Select Claim Winning Pattern"` onto the select element.
