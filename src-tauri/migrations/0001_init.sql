-- 0001_init.sql: Media Tambola Baseline Schema

-- 1. Application Settings
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. Media Assets for Numbers 1 through 90
CREATE TABLE IF NOT EXISTS media_assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    number INTEGER NOT NULL UNIQUE CHECK(number >= 1 AND number <= 90),
    media_type TEXT NOT NULL CHECK(media_type IN ('IMAGE', 'VIDEO')),
    source_type TEXT NOT NULL CHECK(source_type IN ('LOCAL', 'URL')),
    path_or_url TEXT NOT NULL,
    poster_path TEXT,
    title_gujarati TEXT NOT NULL,
    is_valid BOOLEAN NOT NULL DEFAULT 0,
    last_validated_at DATETIME,
    validation_error TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3. Ticket Batches
CREATE TABLE IF NOT EXISTS ticket_batches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    ticket_count INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 4. Tickets
CREATE TABLE IF NOT EXISTS tickets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_id INTEGER NOT NULL REFERENCES ticket_batches(id) ON DELETE CASCADE,
    ticket_code TEXT NOT NULL UNIQUE,
    layout_hash TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_tickets_batch_id ON tickets(batch_id);

-- 5. Ticket Cells (15 cells per ticket in a 3x9 grid)
CREATE TABLE IF NOT EXISTS ticket_cells (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticket_id INTEGER NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    row_idx INTEGER NOT NULL CHECK(row_idx >= 0 AND row_idx <= 2),
    col_idx INTEGER NOT NULL CHECK(col_idx >= 0 AND col_idx <= 8),
    number INTEGER NOT NULL CHECK(number >= 1 AND number <= 90),
    UNIQUE(ticket_id, row_idx, col_idx),
    UNIQUE(ticket_id, number)
);
CREATE INDEX IF NOT EXISTS idx_ticket_cells_ticket_id ON ticket_cells(ticket_id);
CREATE INDEX IF NOT EXISTS idx_ticket_cells_number ON ticket_cells(number);

-- 6. Prize Schemes
CREATE TABLE IF NOT EXISTS prize_schemes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    scheme_type TEXT NOT NULL CHECK(scheme_type IN ('EARLY_FIVE', 'FOUR_CORNERS', 'TOP_LINE', 'MIDDLE_LINE', 'BOTTOM_LINE', 'FULL_HOUSE')),
    rule_config_json TEXT NOT NULL DEFAULT '{}',
    max_winners INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 7. Games Session
CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_code TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK(status IN ('READY', 'ACTIVE', 'PAUSED', 'COMPLETED', 'ENDED')),
    ticket_batch_id INTEGER NOT NULL REFERENCES ticket_batches(id),
    draw_sequence_json TEXT NOT NULL,
    current_draw_position INTEGER NOT NULL DEFAULT 0,
    started_at DATETIME,
    ended_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_games_batch_id ON games(ticket_batch_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_single_active_or_paused_game ON games((1)) WHERE status IN ('ACTIVE', 'PAUSED');

-- 8. Game Tickets Mapping
CREATE TABLE IF NOT EXISTS game_tickets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    ticket_id INTEGER NOT NULL REFERENCES tickets(id),
    UNIQUE(game_id, ticket_id)
);
CREATE INDEX IF NOT EXISTS idx_game_tickets_game_id ON game_tickets(game_id);
CREATE INDEX IF NOT EXISTS idx_game_tickets_ticket_id ON game_tickets(ticket_id);

-- 9. Game Media Snapshots
CREATE TABLE IF NOT EXISTS game_media_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    number INTEGER NOT NULL CHECK(number >= 1 AND number <= 90),
    media_type TEXT NOT NULL,
    source_type TEXT NOT NULL,
    path_or_url TEXT NOT NULL,
    poster_path TEXT,
    title_gujarati TEXT NOT NULL,
    UNIQUE(game_id, number)
);
CREATE INDEX IF NOT EXISTS idx_game_media_snapshots_game ON game_media_snapshots(game_id);

-- 10. Game Schemes Snapshots
CREATE TABLE IF NOT EXISTS game_schemes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    scheme_id INTEGER NOT NULL REFERENCES prize_schemes(id),
    name TEXT NOT NULL,
    scheme_type TEXT NOT NULL,
    rule_config_json TEXT NOT NULL,
    max_winners INTEGER NOT NULL,
    UNIQUE(game_id, scheme_id)
);
CREATE INDEX IF NOT EXISTS idx_game_schemes_game ON game_schemes(game_id);

-- 11. Game Number Calls (History of Draws)
CREATE TABLE IF NOT EXISTS game_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    draw_position INTEGER NOT NULL CHECK(draw_position >= 1 AND draw_position <= 90),
    number INTEGER NOT NULL CHECK(number >= 1 AND number <= 90),
    drawn_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(game_id, number),
    UNIQUE(game_id, draw_position)
);
CREATE INDEX IF NOT EXISTS idx_game_calls_game_pos ON game_calls(game_id, draw_position);
CREATE INDEX IF NOT EXISTS idx_game_calls_game_num ON game_calls(game_id, number);

-- 12. Claims Audit Trail
CREATE TABLE IF NOT EXISTS claims (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    ticket_id INTEGER NOT NULL REFERENCES tickets(id),
    scheme_id INTEGER NOT NULL REFERENCES prize_schemes(id),
    draw_position INTEGER NOT NULL,
    is_accepted BOOLEAN NOT NULL,
    rejection_reason TEXT,
    claimed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_claims_game_id ON claims(game_id);
CREATE INDEX IF NOT EXISTS idx_claims_ticket_id ON claims(ticket_id);
CREATE INDEX IF NOT EXISTS idx_claims_scheme_id ON claims(scheme_id);

-- 13. Winners
CREATE TABLE IF NOT EXISTS winners (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    scheme_id INTEGER NOT NULL REFERENCES prize_schemes(id),
    ticket_id INTEGER NOT NULL REFERENCES tickets(id),
    claim_id INTEGER NOT NULL REFERENCES claims(id),
    is_full_house BOOLEAN NOT NULL,
    winner_position INTEGER NOT NULL,
    awarded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(game_id, scheme_id, ticket_id)
);
CREATE INDEX IF NOT EXISTS idx_winners_game_scheme ON winners(game_id, scheme_id);
CREATE INDEX IF NOT EXISTS idx_winners_ticket ON winners(ticket_id);

-- Seed Default Prize Schemes
INSERT OR IGNORE INTO prize_schemes (id, name, scheme_type, rule_config_json, max_winners, is_active, display_order)
VALUES
    (1, 'Early Five', 'EARLY_FIVE', '{}', 1, 1, 1),
    (2, 'Four Corners', 'FOUR_CORNERS', '{}', 1, 1, 2),
    (3, 'Top Line', 'TOP_LINE', '{}', 1, 1, 3),
    (4, 'Middle Line', 'MIDDLE_LINE', '{}', 1, 1, 4),
    (5, 'Bottom Line', 'BOTTOM_LINE', '{}', 1, 1, 5),
    (6, 'Full House', 'FULL_HOUSE', '{}', 1, 1, 6);

-- Seed Initial App Settings
INSERT OR IGNORE INTO app_settings (key, value)
VALUES
    ('language', 'gu'),
    ('theme', 'dark'),
    ('default_video_volume', '80'),
    ('video_autoplay', 'false'),
    ('show_internal_numbers', 'false');
