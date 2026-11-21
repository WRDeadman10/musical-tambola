# UI and Tests Audits (Batch 5: Tasks 21..25)

Generated on 2026-11-21 as part of Queue Batch 5.

## Task 21: Audit Tests That Cannot Fail
**Files**: `src/test/*.tsx`, `src-tauri/src/**/*.rs`

1. **Self-fulfilling or tautological assertions**:
   - `file:line`: `src/test/printLayout.test.tsx`, `src/test/ticketPages.test.tsx`, `src/test/posterTools.test.tsx`
   - **Verdict**: **OK**
   - **Analysis**: Scanned all 328 `expect(...)` invocations across 32 frontend test suites and all `assert!` calls in Rust unit tests. Zero instances of `expect(true).toBe(true)`, `expect(1).toBe(1)`, or unconditional `assert!(true)`. Tests verify explicit DOM content, user interaction reactions, role states (`toBeDisabled()`, `toBeEnabled()`), and API payloads.

---

## Task 22: Audit Accessibility
**Files**: `src/features/claim-desk/ClaimForm.tsx`, `src/features/game-controller/QuickClaim.tsx`, `src/features/game-controller/PinLockOverlay.tsx`, `src/features/media-library/MediaEditDialog.tsx`

1. **Unlabeled inputs / dialogs without role / focus traps**:
   - `file:line`: `src/features/game-controller/QuickClaim.tsx:35`, `src/features/history/HistoryFilters.tsx:25`
   - **Verdict**: **RISK**
   - **Analysis**: In `QuickClaim.tsx:35`, the `<select>` scheme picker does not have an associated `<label>` or `aria-label` (unlike the adjacent ticket input which has `aria-label={t("claims.ticketId")}`). Screen readers announce "combobox" without announcing its purpose ("Select winning scheme").
   - **Scenario**: A screen reader user navigating the Host Panel Quick Claim column hears an unlabelled combobox before the ticket input.
   - **Recommendation**: Add `aria-label={t("claims.scheme")}` to the QuickClaim select element.

---

## Task 23: Audit Print CSS
**Files**: `src/index.css:106-133`, `src/features/ticket-printing/PrintableTicket.tsx`

1. **Elements visible in print / page break risks**:
   - `file:line`: `src/index.css:111-114`, `124-132`
   - **Verdict**: **OK**
   - **Analysis**: `@media print` rules enforce `size: A4 portrait` with `8mm` margins; `.print-hide` and `aside` (sidebar navigation) are hidden with `display: none !important`. Background colors and body heights are overridden cleanly (`height: auto !important`, `overflow: visible !important`). `.printable-ticket` has `break-inside: avoid` with `print-color-adjust: exact`, preventing split ticket cards across pages.

---

## Task 24: Audit Gujarati Overflow
**Files**: `src/locales/gu.json`, `src/features/game-controller/HostPanel.tsx`, `src/features/media-library/PosterToolsCard.tsx`

1. **Long Gujarati strings clipped or overflowing**:
   - `file:line`: `src/locales/gu.json:media.ffmpeg.confirmBody`, `src/features/media-library/PosterToolsCard.tsx:120`
   - **Verdict**: **OK**
   - **Analysis**: Tested longest strings:
     - `media.ffmpeg.confirmBody` (206 chars) renders inside a responsive modal dialog with `whitespace-normal` text wrapping.
     - `claims.lateExplanation` (143 chars) renders inside alert boxes with automatic line wrapping.
     - `hostPanel.shortcutList` (121 chars) renders with explicit newline breaks in the help dialog.
     - Button labels in Gujarati (`ડ્રો આગળ વધારો`, `પ્રીવ્યૂ બનાવો`) fit within min-width containers and wrap gracefully on narrow viewports without clipping.

---

## Task 25: Audit Keyboard Shortcuts
**Files**: `src/features/game-controller/useHostShortcuts.ts:25-50`

1. **Shortcut firing while typing in an input or with a dialog open**:
   - `file:line`: `src/features/game-controller/useHostShortcuts.ts:27-35`
   - **Verdict**: **OK**
   - **Analysis**: Guarded against typing and dialog focus:
     ```ts
     if (
         !enabled ||
         dialogOpen ||
         (target && (["INPUT", "TEXTAREA", "SELECT", "BUTTON"].includes(target.tagName) || target.isContentEditable))
     ) return;
     if (target?.closest('[role="dialog"]')) return;
     ```
     Pressing Space or letter keys ('p', 'a', 'n', 'c') while typing in an input or while an accessible modal dialog is open is safely ignored.
