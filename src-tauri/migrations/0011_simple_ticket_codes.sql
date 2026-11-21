-- 0011_simple_ticket_codes.sql: Backfill ticket codes to simple sequential numbers (1, 2, 3...)

-- Update tickets table stripping 'T-' and leading zeros if numeric
UPDATE tickets
SET ticket_code = CAST(SUBSTR(ticket_code, 3) AS TEXT)
WHERE ticket_code LIKE 'T-%' AND SUBSTR(ticket_code, 3) GLOB '[0-9]*';

-- Update claims table ticket_code_submitted
UPDATE claims
SET ticket_code_submitted = CAST(SUBSTR(ticket_code_submitted, 3) AS TEXT)
WHERE ticket_code_submitted LIKE 'T-%' AND SUBSTR(ticket_code_submitted, 3) GLOB '[0-9]*';
