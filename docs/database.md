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

Partial unique index ensures at most one active or paused game:
`CREATE UNIQUE INDEX idx_single_active_or_paused_game ON games(status) WHERE status IN ('ACTIVE', 'PAUSED');`

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

### 12. `claims`
Audit trail of every submitted claim.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `game_id` INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE
- `ticket_id` INTEGER NOT NULL REFERENCES tickets(id)
- `scheme_id` INTEGER NOT NULL REFERENCES prize_schemes(id)
- `draw_position` INTEGER NOT NULL
- `is_accepted` BOOLEAN NOT NULL
- `rejection_reason` TEXT
- `claimed_at` DATETIME DEFAULT CURRENT_TIMESTAMP

### 13. `winners`
Accepted prize awards.
- `id` INTEGER PRIMARY KEY AUTOINCREMENT
- `game_id` INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE
- `scheme_id` INTEGER NOT NULL REFERENCES prize_schemes(id)
- `ticket_id` INTEGER NOT NULL REFERENCES tickets(id)
- `claim_id` INTEGER NOT NULL REFERENCES claims(id)
- `is_full_house` BOOLEAN NOT NULL
- `winner_position` INTEGER NOT NULL
- `awarded_at` DATETIME DEFAULT CURRENT_TIMESTAMP
- UNIQUE(game_id, scheme_id, ticket_id)

## Migrations Strategy
Migrations are managed in Rust using embedded SQL files (`src-tauri/migrations/0001_init.sql`, etc.) tracked in a `schema_migrations` table with version and applied_at timestamps.
