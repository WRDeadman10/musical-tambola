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

---

## ADR-005: Schema Migration Immutability and Final In-Place Edit Note

### Context
During Phase 2 implementation, `0001_init.sql` was amended in place to support `REMOTE` source types and temporarily contained a stray `'URL'` value. In production and after tag/release, editing historical migration files destroys reproducibility and breaks checksum tracking.

### Decision
`0001_init.sql` CHECK constraint is strictly set to `CHECK(source_type IN ('LOCAL', 'REMOTE'))`. Per Lead review (MSG-019), because the application has not yet been released or deployed to users, this represents the final in-place edit allowed inside `0001_init.sql`. All future schema modifications across subsequent phases must strictly be committed as new, sequential, immutable SQL migration files (e.g. `0002_*.sql`).

### Consequences
- **Pros**:
  - Guarantees deterministic database state across all development and production environments.
  - Aligns with standard migration lifecycle best practices.
- **Cons**:
  - Schema changes require creating new migration files rather than editing initial schema definitions.

---

## ADR-006: Constrained Randomized Ticket Generation and Layout Hashing

### Context
Tambola tickets require a valid 3x9 grid (15 numbers, 5 per row, 1–3 per column, column ranges, and ascending sorted column values). Randomly placing numbers without constraints leads to invalid configurations, high rejection rates, or biased distributions. Duplicate ticket layouts must be prevented globally across all batches.

### Decision
1. **Constrained Layout Synthesis**: Use randomized backtracking to generate a 3x9 binary presence mask where each row has exactly 5 occupied cells and each column has 1–3 occupied cells.
2. **Column Number Selection**: Draw $k$ distinct numbers uniformly at random from each column's strict range using `RandomSource` (`OsRng` in production, `DeterministicRandomSource` in tests), sort them ascending, and assign them to occupied row cells top-to-bottom.
3. **Canonical Layout Hashing**: Compute a deterministic row-major string representation and SHA-256 hash (`layout_hash`). Uniqueness is enforced atomically in SQLite (`UNIQUE(layout_hash)`).
4. **Collision Handling**: Maintain a retry budget (50 per ticket, $10 \times count$ overall). If budget is exhausted, the transaction is rolled back completely.

5. **Database Transaction & Mutex Locking**:
   - `generate_batch` operates within a single SQLite transaction and acquires the database lock for the duration of batch generation.
   - During generation of a large batch (up to 10,000 tickets), concurrent reader or media protocol requests will wait on the mutex.
   - For V1 desktop usage, this mutex exclusivity is acceptable given the 10,000 ticket cap and guarantees absolute consistency and monotonic code assignment without interleaving or partial writes.

### Consequences
- **Pros**:
  - 100% of generated tickets pass full Tambola validation before insertion.
  - Zero duplicate tickets across games and batches.
  - Testable and reproducible with deterministic RNG seeds.
  - Atomic generation: no partial batches or corrupted sequences.
- **Cons**:
  - Media protocol requests are blocked during large batch generation (acceptable for V1 offline desktop usage).

---

## ADR-007: Stateless Winning Scheme Evaluators and Qualification Logic

### Context
Phase 4 introduces customizable winning schemes (Early Five, Four Corners, Top Line, Middle Line, Bottom Line, Full House) with maximum winner quotas, active flags, and qualification rules. Claim validation requires calculating the exact chronological draw position at which a ticket qualified to strictly enforce the "no late claims" rule.

### Decision
1. **Stateless, Pure Evaluators**:
   - Evaluator implementations (`LineEvaluator`, `FullHouseEvaluator`, `FourCornersEvaluator`, `EarlyFiveEvaluator`) implement a common `SchemeEvaluator` trait without any database dependencies or side effects.
   - Dynamic code evaluation (`eval` or script interpreters) is strictly prohibited.
2. **Four Corners Convention**:
   - Isolated explicitly in `FourCornersEvaluator`: defined as the leftmost (index 0) and rightmost (index 4) occupied cells of Row 0 and Row 2.
3. **Early Five Qualification Position**:
   - Gathers all ticket numbers present in `call_position_by_number`. If count < 5, ticket is not qualified (`None`). If count $\ge 5$, draw positions are sorted ascending and the 5th smallest ($P_4$) is authoritative.
4. **Lines and Full House Position**:
   - Qualification position is the maximum call position among required numbers:
     $$\max_{n \in \text{required}} (\text{call\_position}[n])$$
5. **Full House Identification**:
   - Authoritatively identified via `SchemeType::FullHouse` enum check, never by matching user-editable string names.
6. **Database Migration 0002**:
   - Added `description` and `updated_at` to `prize_schemes` and created a case-insensitive unique index `idx_prize_schemes_name_nocase` on `prize_schemes(name COLLATE NOCASE)`.
   - `updated_at` added with constant default `DEFAULT ''` and backfilled from `created_at` to comply with SQLite's non-constant default restrictions on `ALTER TABLE ADD COLUMN`.

### Consequences
- **Pros**:
  - Highly testable, zero side-effect qualification engine.
  - Chronological draw order invariant guaranteed (number magnitudes never compared).
  - Schema immutability preserved via versioned migration `0002_schemes_update.sql`.
- **Cons**:
  - New scheme types require extending the Rust `SchemeType` enum and registry.

---

## ADR-008: Game Session Lifecycle, Snapshot Immutability, and Crash Recovery

### Context
Phase 5 introduces the Game Controller and Draw Engine. Game execution requires strict immutability: changes to live media assets or prize schemes must never alter or corrupt games already in progress. In addition, games must survive application crashes and power loss with 100% deterministic recovery of past draws, current position, and future draw sequences.

### Decision
1. **Cryptographic Draw Permutation**:
   - `generate_draw_sequence` implements Fisher–Yates shuffle over numbers 1..=90 using `RandomSource` (`OsRngSource` in production, `DeterministicRandomSource` in tests).
   - The entire 90-number permutation is persisted in SQLite as JSON (`draw_sequence_json`) *before* the first draw is used.
2. **Game Media and Scheme Snapshots**:
   - Upon `create_game`, all 90 media assets are snapshotted into `game_media_snapshots`, and all currently active prize schemes are snapshotted into `game_schemes`.
   - The media protocol serves snapshot media via `http://media.localhost/game/<gameId>/<number>` and `/thumb`.
   - Mutations to `media_assets` or `prize_schemes` during or after game creation have zero effect on active or historic games.
3. **State Machine & Atomic Transactions**:
   - Only one game may be in `ACTIVE` or `PAUSED` state across the database, enforced by partial unique index `idx_single_active_or_paused_game`.
   - State transitions: `create_game` (ACTIVE, position 1), `draw_next` (ACTIVE -> ACTIVE, or COMPLETED at 90), `pause_game` (ACTIVE -> PAUSED), `resume_game` (PAUSED -> ACTIVE), `end_game` (ACTIVE|PAUSED -> ENDED).
   - Lifecycle operations (`pause_game`, `resume_game`, `end_game`) use single atomic conditional SQL `UPDATE ... WHERE id = ?1 AND status = '...'` with rows-affected verification to eliminate check-then-act races.
   - `end_game` explicitly rejects games that are already in `COMPLETED` or `ENDED` status.
   - `draw_next` is strictly idempotent: takes `expected_position`. If `current_draw_position != expected_position`, returns the existing `GameState` unchanged without drawing another number.
   - Persisted draw sequence JSON is strictly validated as a 90-element permutation of 1..=90 both during `draw_next` and during game state restoration (`get_game`/`get_active_game`), failing gracefully with clear errors if corrupted.
4. **Deterministic Crash Recovery**:
   - `get_active_game` reconstructs `GameState` purely from persisted SQLite records without regenerating or altering the draw sequence.

### Consequences
- **Pros**:
  - Complete crash recovery: dropping connections and restarting the application resumes exact game state.
  - Snapshot immutability: games are self-contained and auditable.
  - Zero chance of double draws or sequence desynchronization.
- **Cons**:
  - Snapshotting 90 media rows adds a minor one-time disk cost per game session (~few kilobytes).

---

## ADR-009: Claim Validation Engine, Rejection Codes, and Winner Allocation Guards

### Context
Phase 6 introduces the Claim Validator and Winner Allocation engine (spec §9, §10, §11, §12, §18). In high-pressure live Tambola games, claims may be submitted concurrently, out-of-order, late, or for ineligible schemes. The application must enforce:
1. Pure, testable decision logic decoupled from I/O.
2. Complete auditability: every claim attempt (accepted or rejected, known or unknown ticket) must be permanently persisted.
3. Multi-layer guards: application-level evaluation plus database-level UNIQUE constraints to prevent race conditions or double allocations.
4. User-friendly frontend localization: return typed error codes and canonical English text, avoiding raw database errors.

### Decision
1. **Pure Decision Engine (`domain/claims/claim_validation.rs`)**:
   - `evaluate_claim` is a pure function with zero database or network dependencies.
   - Strictly enforces a 10-step check order:
     1. Active/Paused game status check (`NO_ACTIVE_GAME`, `GAME_NOT_ACCEPTING_CLAIMS`).
     2. Ticket existence and game batch mapping (`TICKET_NOT_FOUND`, `TICKET_NOT_IN_GAME`).
     3. Scheme game snapshot check (`SCHEME_NOT_IN_GAME`).
     4. Pattern qualification evaluation via `SchemeRegistry` (`NOT_COMPLETED`).
     5. Strict late-claim enforcement: accept only when $qualification\_position == current\_draw\_position$; if $qualification\_position < current\_draw\_position$, reject as `LATE_CLAIM`.
     6. Duplicate scheme win prevention (`ALREADY_WON_SCHEME`).
     7. Post-Full-House preclusion (`ALREADY_WON_FULL_HOUSE`).
     8. Single non-Full-House rule (`ONLY_FULL_HOUSE_ELIGIBLE`).
     9. Capacity check ($accepted\_count \ge max\_winners$) -> `WINNER_LIMIT_REACHED`.
     10. Acceptance ($winner\_position = accepted\_count + 1$).
2. **Migration 0003 & Table Rebuild**:
   - `claims` rebuilt to include `status` (`ACCEPTED`/`REJECTED`), `rejection_code`, `rejection_reason`, `ticket_code_submitted`, and `qualification_position`. `ticket_id` and `scheme_id` are nullable so invalid submissions are preserved in the audit trail.
   - `winners` enforced with SQLite-level partial unique indexes:
     - `idx_winners_game_scheme_pos` (unique rank per scheme).
     - `idx_winners_single_full_house_per_ticket` (at most one Full House).
     - `idx_winners_single_non_full_house_per_ticket` (at most one non-Full-House).
3. **Atomic Service Transactions & Concurrency**:
   - `ClaimRepository::submit_claim` executes inside a single `BEGIN IMMEDIATE` transaction.
   - If an insert into `winners` encounters a race condition or constraint violation, the award is rolled back, the claim is converted to `REJECTED` with code `WINNER_LIMIT_REACHED`, and committed safely without raising raw SQL errors.
4. **Ticket Code Normalization**:
   - Accepts shorthand and user variations ("1", "000001", "T-1", "t-000001") and canonicalizes them to 6-digit padded format ("T-000001").

### Consequences
- **Pros**:
  - Complete crash and concurrency safety: zero chance of over-allocating winners or double prize payouts.
  - Transparent audit trail: every submission is recorded.
  - Portable, decoupled pure logic covered by unit tests.
- **Cons**:
  - Rebuilding `claims` table required versioned Migration 0003.

---

## ADR-010: Permanent Game History Preservation and Snapshot Integrity

### Context
Phase 7 introduces Winner History, Claim Audit History, and Dashboard Statistics (spec §4, §12). Tambola prize awards, game sessions, and claim audit trails represent historical event records. Accidental or intentional hard-deletion of games, claims, or winners would compromise prize auditing, dispute resolution, and historical game records. Furthermore, if a prize scheme is subsequently renamed in the administrative master table (`prize_schemes`), historical winner records must reflect the scheme name active when the game was played.

### Decision
1. **Never Delete History Guarantee**:
   - Games, claims, and winners must never be hard-deleted through any API or user command.
   - Foreign key `ON DELETE CASCADE` on `claims` and `winners` exists purely as relational hygiene; no command exposes game deletion.
   - Deletion of `ticket_batches` is explicitly guarded: attempting to delete a batch whose tickets were assigned to games, claims, or winners returns an error (`TicketRepository::delete_batch`).
2. **Snapshot Name Integrity**:
   - All history and winner queries join against the immutable game snapshot (`game_schemes`) first: `COALESCE(gs.name, ps.name)`.
   - Modifying or renaming a master prize scheme in `prize_schemes` does not alter the historical scheme name displayed on past winners or claims.
3. **Optimized Read-Only Queries**:
   - Migration 0004 introduces compound indexes (`idx_claims_status`, `idx_claims_ticket_code_submitted`, `idx_claims_claimed_at`, `idx_winners_awarded_at`, `idx_winners_claim_id`) for parameterized multi-attribute filtering and pagination.

### Consequences
- **Pros**:
  - Irreversible auditability: past tournament outcomes and prize claims cannot be erased or distorted by configuration updates.
  - Snapshot fidelity: historic game records match exactly what players and hosts saw during gameplay.
  - Query performance: pagination and date-range filters remain performant across thousands of historical records.
- **Cons**:
  - Storage space is permanent (negligible for SQLite with local tambola sessions).
