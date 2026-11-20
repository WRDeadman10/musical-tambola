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
