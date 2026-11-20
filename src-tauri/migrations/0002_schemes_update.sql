-- Migration 0002: Add description and updated_at to prize_schemes and add case-insensitive unique index on name
ALTER TABLE prize_schemes ADD COLUMN description TEXT NOT NULL DEFAULT '';
ALTER TABLE prize_schemes ADD COLUMN updated_at DATETIME NOT NULL DEFAULT '';
UPDATE prize_schemes SET updated_at = created_at WHERE updated_at = '';

-- Case-insensitive unique index on prize_schemes name
CREATE UNIQUE INDEX IF NOT EXISTS idx_prize_schemes_name_nocase ON prize_schemes(name COLLATE NOCASE);
