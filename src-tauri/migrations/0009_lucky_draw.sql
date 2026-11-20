-- Migration 0009: Lucky Draw
CREATE TABLE IF NOT EXISTS lucky_draws (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source TEXT NOT NULL CHECK(source IN ('NAMES', 'TICKETS')),
    batch_id INTEGER REFERENCES ticket_batches(id),
    exclude_previous_winners BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS lucky_draw_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    draw_id INTEGER NOT NULL REFERENCES lucky_draws(id) ON DELETE CASCADE,
    entry_value TEXT NOT NULL,
    is_winner BOOLEAN NOT NULL DEFAULT 0,
    drawn_at DATETIME
);

CREATE TABLE IF NOT EXISTS lucky_draw_winners (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    draw_id INTEGER NOT NULL REFERENCES lucky_draws(id) ON DELETE CASCADE,
    winner TEXT NOT NULL,
    draw_position INTEGER NOT NULL,
    drawn_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_lucky_draw_entries_draw ON lucky_draw_entries(draw_id, is_winner);
CREATE INDEX IF NOT EXISTS idx_lucky_draw_winners_draw ON lucky_draw_winners(draw_id);
