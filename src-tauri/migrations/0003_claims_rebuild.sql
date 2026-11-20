-- Migration 0003: Rebuild claims table and add winners constraints and indexes

CREATE TABLE IF NOT EXISTS claims_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    ticket_id INTEGER REFERENCES tickets(id),
    ticket_code_submitted TEXT NOT NULL,
    scheme_id INTEGER,
    current_draw_position INTEGER NOT NULL,
    qualification_position INTEGER,
    status TEXT NOT NULL CHECK (status IN ('ACCEPTED', 'REJECTED')),
    rejection_code TEXT,
    rejection_reason TEXT,
    claimed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Copy existing claims if any
INSERT INTO claims_new (
    id, game_id, ticket_id, ticket_code_submitted, scheme_id, current_draw_position,
    qualification_position, status, rejection_code, rejection_reason, claimed_at
)
SELECT
    c.id,
    c.game_id,
    c.ticket_id,
    COALESCE(t.ticket_code, 'UNKNOWN'),
    c.scheme_id,
    c.draw_position,
    CASE WHEN c.is_accepted THEN c.draw_position ELSE NULL END,
    CASE WHEN c.is_accepted THEN 'ACCEPTED' ELSE 'REJECTED' END,
    CASE WHEN c.is_accepted THEN NULL ELSE 'LEGACY_REJECT' END,
    c.rejection_reason,
    c.claimed_at
FROM claims c
LEFT JOIN tickets t ON c.ticket_id = t.id;

DROP TABLE claims;
ALTER TABLE claims_new RENAME TO claims;

-- Recreate indexes on claims
CREATE INDEX IF NOT EXISTS idx_claims_game_id ON claims(game_id);
CREATE INDEX IF NOT EXISTS idx_claims_ticket_id ON claims(ticket_id);
CREATE INDEX IF NOT EXISTS idx_claims_scheme_id ON claims(scheme_id);

-- Winners constraints & indexes
CREATE UNIQUE INDEX IF NOT EXISTS idx_winners_game_scheme_pos ON winners(game_id, scheme_id, winner_position);
CREATE UNIQUE INDEX IF NOT EXISTS idx_winners_single_full_house_per_ticket ON winners(game_id, ticket_id) WHERE is_full_house = 1;
CREATE UNIQUE INDEX IF NOT EXISTS idx_winners_single_non_full_house_per_ticket ON winners(game_id, ticket_id) WHERE is_full_house = 0;
CREATE INDEX IF NOT EXISTS idx_winners_game_id ON winners(game_id);
