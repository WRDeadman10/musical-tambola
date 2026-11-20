# Database Schema & Persistence — Media Tambola

## Storage Engine
- **Engine**: SQLite via `rusqlite` (bundled feature) in Rust.
- **Location**: `%APPDATA%/<bundle-identifier>/data/tambola.sqlite`.
- **Modes**:
  - `PRAGMA journal_mode = WAL;`
  - `PRAGMA foreign_keys = ON;`
  - `PRAGMA synchronous = NORMAL;`

## Tables Schema

### 1. `app_settings`
Key-value configuration store for application settings.
- `key` TEXT PRIMARY KEY
- `value` TEXT NOT NULL
- `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP

### 2. `media_assets`
Stores media slot configurations for numbers 1 through 90.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `number` INTEGER NOT NULL UNIQUE CHECK(number >= 1 AND number <= 90)
- `media_type` TEXT NOT NULL CHECK(media_type IN ('IMAGE', 'VIDEO'))
- `source_type` TEXT NOT NULL CHECK(source_type IN ('LOCAL', 'URL'))
- `path_or_url` TEXT NOT NULL
- `poster_path` TEXT
- `title_gujarati` TEXT NOT NULL
- `is_valid` BOOLEAN NOT NULL DEFAULT 0
- `last_validated_at` DATETIME
- `validation_error` TEXT
- `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP
- `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP

### 3. `ticket_batches`
Logical grouping of generated tickets.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `batch_code` TEXT NOT NULL UNIQUE
- `name` TEXT NOT NULL
- `ticket_count` INTEGER NOT NULL
- `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP

### 4. `tickets`
Tambola ticket definitions.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `batch_id` INTEGER NOT NULL REFERENCES ticket_batches(id) ON DELETE CASCADE
- `ticket_code` TEXT NOT NULL UNIQUE
- `layout_hash` TEXT NOT NULL UNIQUE
- `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP

### 5. `ticket_cells`
15 filled cells per ticket (3 rows x 9 columns grid).
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `ticket_id` INTEGER NOT NULL REFERENCES tickets(id) ON DELETE CASCADE
- `row_idx` INTEGER NOT NULL CHECK(row_idx >= 0 AND row_idx <= 2)
- `col_idx` INTEGER NOT NULL CHECK(col_idx >= 0 AND col_idx <= 8)
- `number` INTEGER NOT NULL CHECK(number >= 1 AND number <= 90)
- UNIQUE(ticket_id, row_idx, col_idx)
- UNIQUE(ticket_id, number)

### 6. `prize_schemes`
Data-driven configuration of winning patterns.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `name` TEXT NOT NULL
- `scheme_type` TEXT NOT NULL CHECK(scheme_type IN ('EARLY_FIVE', 'FOUR_CORNERS', 'TOP_LINE', 'MIDDLE_LINE', 'BOTTOM_LINE', 'FULL_HOUSE'))
- `rule_config_json` TEXT NOT NULL
- `max_winners` INTEGER NOT NULL DEFAULT 1
- `is_active` BOOLEAN NOT NULL DEFAULT 1
- `display_order` INTEGER NOT NULL DEFAULT 0
- `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP

### 7. `games`
Session records for Tambola games.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `game_code` TEXT NOT NULL UNIQUE
- `status` TEXT NOT NULL CHECK(status IN ('READY', 'ACTIVE', 'PAUSED', 'COMPLETED', 'ENDED'))
- `ticket_batch_id` INTEGER NOT NULL REFERENCES ticket_batches(id)
- `draw_sequence_json` TEXT NOT NULL
- `current_draw_position` INTEGER NOT NULL DEFAULT 0
- `started_at` DATETIME
- `ended_at` DATETIME
- `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP

Partial unique index ensures at most one game in ACTIVE or PAUSED:
`CREATE UNIQUE INDEX idx_single_active_or_paused_game ON games((1)) WHERE status IN ('ACTIVE', 'PAUSED');`

### 8. `game_tickets`
Snapshot link of tickets participating in an active game.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `game_id` INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE
- `ticket_id` INTEGER NOT NULL REFERENCES tickets(id)
- UNIQUE(game_id, ticket_id)

### 9. `game_media_snapshots`
Snapshot of media slot states when a game initializes.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `game_id` INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE
- `number` INTEGER NOT NULL CHECK(number >= 1 AND number <= 90)
- `media_type` TEXT NOT NULL
- `source_type` TEXT NOT NULL
- `path_or_url` TEXT NOT NULL
- `poster_path` TEXT
- `title_gujarati` TEXT NOT NULL
- UNIQUE(game_id, number)

### 10. `game_schemes`
Snapshot of prize schemes active when a game initializes.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `game_id` INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE
- `scheme_id` INTEGER NOT NULL REFERENCES prize_schemes(id)
- `name` TEXT NOT NULL
- `scheme_type` TEXT NOT NULL
- `rule_config_json` TEXT NOT NULL
- `max_winners` INTEGER NOT NULL
- UNIQUE(game_id, scheme_id)

### 11. `game_calls`
Chronological history of drawn numbers for a game.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `game_id` INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE
- `draw_position` INTEGER NOT NULL CHECK(draw_position >= 1 AND draw_position <= 90)
- `number` INTEGER NOT NULL CHECK(number >= 1 AND number <= 90)
- `drawn_at` DATETIME DEFAULT CURRENT_TIMESTAMP
- UNIQUE(game_id, number)
- UNIQUE(game_id, draw_position)

### 12. `claims` (Updated in Migration 0003)
Authoritative audit trail of every submitted claim, accepted or rejected.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `game_id` INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE
- `ticket_id` INTEGER REFERENCES tickets(id) (NULL for unknown/unregistered tickets)
- `ticket_code_submitted` TEXT NOT NULL (raw or normalized code entered by user)
- `scheme_id` INTEGER (NULL if scheme was deleted or not in master)
- `current_draw_position` INTEGER NOT NULL
- `qualification_position` INTEGER
- `status` TEXT NOT NULL CHECK(status IN ('ACCEPTED', 'REJECTED'))
- `rejection_code` TEXT (e.g. `LATE_CLAIM`, `ONLY_FULL_HOUSE_ELIGIBLE`, etc.)
- `rejection_reason` TEXT (canonical English description)
- `claimed_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP

Indexes:
- `idx_claims_game_id` ON `claims(game_id)`
- `idx_claims_ticket_id` ON `claims(ticket_id)`
- `idx_claims_scheme_id` ON `claims(scheme_id)`

### 13. `winners` (Updated in Migration 0003)
Accepted prize allocations.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `game_id` INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE
- `scheme_id` INTEGER NOT NULL REFERENCES prize_schemes(id)
- `ticket_id` INTEGER NOT NULL REFERENCES tickets(id)
- `claim_id` INTEGER NOT NULL REFERENCES claims(id)
- `is_full_house` BOOLEAN NOT NULL
- `winner_position` INTEGER NOT NULL
- `awarded_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP

Constraints & Indexes:
- `UNIQUE(game_id, scheme_id, ticket_id)`: A ticket can only win a given scheme once.
- `idx_winners_game_scheme_pos` UNIQUE ON `winners(game_id, scheme_id, winner_position)`: Winner placement ranks cannot collide.
- `idx_winners_single_full_house_per_ticket` UNIQUE ON `winners(game_id, ticket_id) WHERE is_full_house = 1`: At most one Full House win per ticket per game.
- `idx_winners_single_non_full_house_per_ticket` UNIQUE ON `winners(game_id, ticket_id) WHERE is_full_house = 0`: At most one non-Full-House win per ticket per game.
- `idx_winners_game_id` ON `winners(game_id)`

## Migrations Strategy
Migrations are managed in Rust using embedded SQL files tracked in a `schema_migrations` table with version and applied_at timestamps:
1. `0001_init.sql`: Core application tables, seed schemes, and initial settings.
2. `0002_schemes_update.sql`: Added `description` and `updated_at` to `prize_schemes` with NOCASE unique index.
3. `0003_claims_rebuild.sql`: Rebuilt `claims` table to record all attempts (including unknown tickets and schemes) with status and rejection codes; added database-level guards on `winners` table to enforce single non-Full-House, single Full House, and winner rank uniqueness.
4. `0004_history_indexes.sql`: Performance indexes for winner history and claim audit trail queries: `idx_claims_status`, `idx_claims_ticket_code_submitted`, `idx_claims_claimed_at`, `idx_winners_awarded_at`, `idx_winners_claim_id`.
