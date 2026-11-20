-- Migration 0007: Pattern engine, masks, and multiple Full House schemes
ALTER TABLE prize_schemes ADD COLUMN pattern_kind TEXT NOT NULL DEFAULT 'MASK';
ALTER TABLE prize_schemes ADD COLUMN mask_json TEXT;
ALTER TABLE prize_schemes ADD COLUMN early_n INTEGER;
ALTER TABLE prize_schemes ADD COLUMN is_full_house INTEGER NOT NULL DEFAULT 0;

-- Backfill existing 6 schemes
UPDATE prize_schemes SET
    pattern_kind = 'EARLY_N',
    mask_json = '[[false,false,false,false,false],[false,false,false,false,false],[false,false,false,false,false]]',
    early_n = 5,
    is_full_house = 0
WHERE scheme_type = 'EARLY_FIVE';

UPDATE prize_schemes SET
    pattern_kind = 'MASK',
    mask_json = '[[true,false,false,false,true],[false,false,false,false,false],[true,false,false,false,true]]',
    early_n = NULL,
    is_full_house = 0
WHERE scheme_type = 'FOUR_CORNERS';

UPDATE prize_schemes SET
    pattern_kind = 'MASK',
    mask_json = '[[true,true,true,true,true],[false,false,false,false,false],[false,false,false,false,false]]',
    early_n = NULL,
    is_full_house = 0
WHERE scheme_type = 'TOP_LINE';

UPDATE prize_schemes SET
    pattern_kind = 'MASK',
    mask_json = '[[false,false,false,false,false],[true,true,true,true,true],[false,false,false,false,false]]',
    early_n = NULL,
    is_full_house = 0
WHERE scheme_type = 'MIDDLE_LINE';

UPDATE prize_schemes SET
    pattern_kind = 'MASK',
    mask_json = '[[false,false,false,false,false],[false,false,false,false,false],[true,true,true,true,true]]',
    early_n = NULL,
    is_full_house = 0
WHERE scheme_type = 'BOTTOM_LINE';

UPDATE prize_schemes SET
    pattern_kind = 'FULL_HOUSE',
    mask_json = '[[true,true,true,true,true],[true,true,true,true,true],[true,true,true,true,true]]',
    early_n = NULL,
    is_full_house = 1
WHERE scheme_type = 'FULL_HOUSE';

-- Seed preset schemes (inactive by default)
INSERT OR IGNORE INTO prize_schemes (
    scheme_type, name, description, max_winners, is_active, display_order, pattern_kind, mask_json, early_n, is_full_house
) VALUES
('EARLY_FIVE', 'Early 7', 'Any 7 numbers on the ticket are called.', 1, 0, 7, 'EARLY_N', '[[false,false,false,false,false],[false,false,false,false,false],[false,false,false,false,false]]', 7, 0),
('EARLY_FIVE', 'Early 10', 'Any 10 numbers on the ticket are called.', 1, 0, 8, 'EARLY_N', '[[false,false,false,false,false],[false,false,false,false,false],[false,false,false,false,false]]', 10, 0),
('MIDDLE_LINE', 'Pattern H', 'Columns 1 and 5 all rows, plus middle row.', 1, 0, 9, 'MASK', '[[true,false,false,false,true],[true,true,true,true,true],[true,false,false,false,true]]', NULL, 0),
('MIDDLE_LINE', 'Pattern Plus', 'Middle row and middle column.', 1, 0, 10, 'MASK', '[[false,false,true,false,false],[true,true,true,true,true],[false,false,true,false,false]]', NULL, 0),
('TOP_LINE', 'Pattern V', 'V-shaped pattern across the grid.', 1, 0, 11, 'MASK', '[[true,false,false,false,true],[false,true,false,true,false],[false,false,true,false,false]]', NULL, 0),
('TOP_LINE', 'Pattern A', 'A-shaped pattern across the grid.', 1, 0, 12, 'MASK', '[[false,false,true,false,false],[true,true,true,true,true],[true,false,false,false,true]]', NULL, 0),
('FOUR_CORNERS', 'Photo Frame', 'Outer border cells (photo frame).', 1, 0, 13, 'MASK', '[[true,true,true,true,true],[true,false,false,false,true],[true,true,true,true,true]]', NULL, 0),
('TOP_LINE', 'Breakfast', 'Logical columns 1 and 2 (left two columns).', 1, 0, 14, 'MASK', '[[true,true,false,false,false],[true,true,false,false,false],[true,true,false,false,false]]', NULL, 0),
('MIDDLE_LINE', 'Lunch', 'Logical column 3 (center column).', 1, 0, 15, 'MASK', '[[false,false,true,false,false],[false,false,true,false,false],[false,false,true,false,false]]', NULL, 0),
('BOTTOM_LINE', 'Dinner', 'Logical columns 4 and 5 (right two columns).', 1, 0, 16, 'MASK', '[[false,false,false,true,true],[false,false,false,true,true],[false,false,false,true,true]]', NULL, 0),
('FULL_HOUSE', 'Full House 2', 'Second Full House award (all 15 numbers).', 1, 0, 17, 'FULL_HOUSE', '[[true,true,true,true,true],[true,true,true,true,true],[true,true,true,true,true]]', NULL, 1);

-- Expand game_schemes to preserve immutable snapshot of pattern fields
ALTER TABLE game_schemes ADD COLUMN pattern_kind TEXT NOT NULL DEFAULT 'MASK';
ALTER TABLE game_schemes ADD COLUMN mask_json TEXT;
ALTER TABLE game_schemes ADD COLUMN early_n INTEGER;
ALTER TABLE game_schemes ADD COLUMN is_full_house INTEGER NOT NULL DEFAULT 0;
