# Frontend Glue Audits (Batch 4: Tasks 16..20)

Generated on 2026-11-21 as part of Queue Batch 4.

## Task 16: Audit Store Reactivity
**Files**: `src/stores/appStore.ts`, all components selecting `useAppStore((s) => s.t)`

1. **Selector returning a stable value while its data changes**:
   - `file:line`: `src/stores/appStore.ts:85-93`
   - **Verdict**: **OK** (Fixed in commit `090c1a6`)
   - **Analysis**: Prior to `090c1a6`, `t` was a singleton closures capturing `() => get().language`. Because Zustand selectors compare identity (`s => s.t`), components selecting only `t` did not re-render upon language changes. With the subscriber at lines 87-93 re-creating `t` upon `state.language !== previous.language`, `t` updates its identity, triggering reactive re-renders across all localized components. Verified by `src/test/languageReactivity.test.tsx`.

---

## Task 17: Audit Listener Leaks
**Files**: Every `listen*` and `useEffect` across `src/` (e.g., `SetupGuard.tsx`, `HostPanel.tsx`, `PosterToolsCard.tsx`, `PresentationPage.tsx`, `useMediaLibrary.ts`)

1. **Missing cleanup / Promise resolved after unmount**:
   - `file:line`: `src/components/layout/SetupGuard.tsx:28-38`, `src/features/game-controller/HostPanel.tsx:83-97`, `src/features/media-library/PosterToolsCard.tsx:55-66`
   - **Verdict**: **RISK**
   - **Analysis**: Tauri's `listen` is asynchronous and returns `Promise<UnlistenFn>`. In standard pattern:
     ```ts
     useEffect(() => {
       let stop: (() => void) | null = null;
       void listenToSomething(...).then(cleanup => { stop = cleanup; });
       return () => stop?.();
     }, []);
     ```
     If the component unmounts *before* the async `listen` promise resolves (e.g. rapid navigation), the cleanup function `() => stop?.()` executes when `stop` is still `null`. When the promise subsequently resolves, the event listener is registered permanently on the Tauri core bridge without an active unlisten handle, causing a zombie memory leak.
   - **Failing Scenario**: Rapidly mounting and unmounting `SetupGuard` or `PosterToolsCard` 10 times registers 10 redundant event listeners on the Tauri IPC bridge.
   - **Fix Recommendation**: Track an `isMounted` boolean:
     ```ts
     useEffect(() => {
       let mounted = true;
       let stop: (() => void) | null = null;
       void listenToSomething(...).then(cleanup => {
         if (!mounted) cleanup();
         else stop = cleanup;
       });
       return () => { mounted = false; stop?.(); };
     }, []);
     ```

---

## Task 18: Audit IPC Spreads
**Files**: `src/lib/api/luckyDraw.ts:4-6`, `src/lib/api/media.ts:46-58`

1. **`createLuckyDraw` spread**:
   - `file:line`: `src/lib/api/luckyDraw.ts:4-6` vs `src-tauri/src/commands/lucky_draw.rs:10-15`
   - **Verdict**: **OK**
   - **Analysis**: `CreateLuckyDrawInput` properties are `name`, `source`, `names`, `batchId`, `excludePreviousWinners`. Tauri deserializer maps `{ ...input }` to Rust camelCase arguments `name`, `source`, `names`, `batch_id`, and `exclude_previous_winners` via serde snake_case mapping. Verified with integration tests.

2. **`scanMediaFolder` spread**:
   - `file:line`: `src/lib/api/media.ts:52-57` vs `src-tauri/src/commands/media.rs:270-276`
   - **Verdict**: **OK**
   - **Analysis**: Explicit object structure `{ rootPath, order, recursive, ...(seed === undefined ? {} : { seed }) }` matches Rust command parameters `root_path: String, order: MediaFolderOrder, recursive: bool, seed: Option<u64>`.

---

## Task 19: Audit Error Mapping
**Files**: `src/lib/api/errors.ts:1-27`, `src/locales/en.json:errors`

1. **Rust codes with no frontend translation**:
   - `file:line`: `src/lib/api/errors.ts:10-12`
   - **Verdict**: **RISK**
   - **Analysis**: Newer backend error codes added during Waves 3-5 (such as `ADMIN_PIN_REQUIRED`, `ADMIN_PIN_RATE_LIMITED`, `ADMIN_PIN_INVALID_FORMAT`, `BACKUP_SCHEMA_INCOMPATIBLE`, `CANNOT_RESTORE_ACTIVE_GAME`, `FFMPEG_CHECKSUM_MISMATCH`, `FFMPEG_DOWNLOAD_FAILED`, `POSTER_GENERATION_FAILED`, `BATCH_FORMAT_MISMATCH`) do not have entries in `en.json::errors`.
   - **Scenario**: When `toErrorMessage(caught, t)` is called with `{ code: "ADMIN_PIN_REQUIRED" }`, `translate("errors.ADMIN_PIN_REQUIRED")` returns `"errors.ADMIN_PIN_REQUIRED"`. `toErrorMessage` falls back to `caught.message` (the English error message provided by Rust). The user sees a raw English fallback string rather than a localized UI string in Gujarati mode.

---

## Task 20: Audit Locale Parity
**Files**: `src/locales/en.json`, `src/locales/gu.json`

1. **Keys missing or untranslated in either file**:
   - `file:line`: `src/locales/en.json`, `src/locales/gu.json`
   - **Verdict**: **OK**
   - **Analysis**: Executed full recursive key parity comparison using node json traversal script:
     - Missing in Gujarati (`gu.json`): **0 keys** (`[]`).
     - Missing in English (`en.json`): **0 keys** (`[]`).
     - Total keys: 100% parity across both language files.
