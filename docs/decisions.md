# Architecture Decision Records (ADRs) — Media Tambola

## ADR-001: Selection of Tauri 2.x over Electron

### Context
The original draft specification suggested Electron for the desktop wrapper. However, Electron bundles a complete Chromium runtime and Node.js process, leading to large installer binaries (>100MB), high RAM footprint, and complex native C++ module compilation issues across platforms (e.g. `better-sqlite3` native bindings rebuilding against Electron ABIs).

### Decision
We use **Tauri 2.x** with a Rust backend and the native Windows Evergreen WebView2 runtime.

### Consequences
- **Pros**:
  - Installer size reduced significantly (~10-15MB instead of 120MB+).
  - Native SQLite performance via `rusqlite` compiled directly with Rust without ABI versioning friction.
  - Native security boundary: webview has zero direct access to raw OS files or shell execution.
  - Rust provides type safety, concurrency safety, and high performance for ticket combinatorial generation.
- **Cons**:
  - Requires Rust compiler and toolchain on developer machines.

---

## ADR-002: Rust-Authoritative Game Domain and Embedded SQLite

### Context
Tambola prize claiming involves fast-paced, concurrent interactions where prize availability, eligibility constraints, and late-claim disqualifications must be enforced authoritatively without race conditions.

### Decision
All core game logic—including ticket generation, layout hashing, cryptographically secure draw sequence generation, scheme evaluation, qualification calculations, and claim acceptance—is authoritatively executed in Rust inside `BEGIN IMMEDIATE` transactions in SQLite (`rusqlite` bundled). The React layer serves solely as a presentation and user-interaction shell; it never computes winners or executes direct database mutations.

### Consequences
- **Pros**:
  - Eliminates frontend state tampering or UI inconsistencies.
  - ACID guarantees for winner slot allocation and late-claim evaluations.
  - Complete crash recovery: state can be reliably reconstructed from SQLite records.
- **Cons**:
  - All game interactions must cross the Tauri IPC boundary (minimal overhead, typed via `src/lib/api.ts`).

---

## ADR-003: Custom URI Protocol (`media://`) for Local Media Rendering

### Context
Media Tambola displays local media files (images and videos) mapped to numbers 1–90. Opening arbitrary local filesystem paths (`file://` or Tauri asset protocol across full drives) introduces security vulnerabilities and allows webview scripts to probe the host file system.

### Decision
Implement a custom Tauri URI protocol (`media://<number>` or `media://asset/<id>`) handled entirely in Rust. The handler queries the SQLite database for the authorized media record associated with that slot, verifies file existence, and streams the media content with proper MIME types.

### Consequences
- **Pros**:
  - Zero raw filesystem exposure to the webview renderer.
  - Content Security Policy (CSP) remains strict and tightly scoped.
  - Enables centralized media streaming, caching, and placeholder fallback in Rust.
- **Cons**:
  - Protocol handler must correctly handle streaming and Range requests for video playback.

---

## ADR-004: Two-Tier Media Validation Strategy and Concurrency

### Context
Tambola media assets consist of local file paths and remote URLs. Remote links may be slow, return full payloads without Range headers, point to HTML landing pages, or fail HEAD requests. Local files may have spoofed extensions or corrupt headers.

### Decision
Implement a two-tier validation pipeline in Rust (`src-tauri/src/services/media_validation.rs`):
1. **Local Media**:
   - Verify absolute file existence and readability.
   - Enforce extension allowlist (`jpg`, `jpeg`, `png`, `webp`, `gif`, `mp4`, `webm`).
   - Read initial bytes and sniff magic headers (JPEG `FFD8FF`, PNG `89504E47`, GIF `47494638`, WebP `52494646...57454250`, MP4 `ftyp` box at offset 4, WebM `1A45DFA3`). Flag mismatches between declared type (`IMAGE`/`VIDEO`) and sniffed format.
2. **Remote Media**:
   - Execute HTTP GET with `Range: bytes=0-4095` (10s timeout, max 5 redirects) to inspect headers and initial bytes without downloading the entire resource.
   - If server rejects Range with 405/501 or network-level error, fall back to `HEAD`.
   - Reject `text/html` responses immediately.
   - Validate `Content-Type` against declared `IMAGE` / `VIDEO` requirements.
3. **Concurrency & Progress**:
   - `validate_all_media` runs over all 90 slots bounded by a 4-worker semaphore, emitting `media-validation-progress` events to prevent freezing UI.

### Consequences
- **Pros**:
  - Eliminates UI stalls and minimizes bandwidth usage during batch validation.
  - Catches broken, corrupt, or spoofed assets before games begin.
  - Safe against remote servers that reject HEAD or Range requests.
- **Cons**:
  - Transient remote network failures require retry ("Validate again").
