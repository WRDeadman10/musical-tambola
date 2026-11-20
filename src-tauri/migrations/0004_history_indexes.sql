-- Migration 0004: Performance indexes for history and winners queries

CREATE INDEX IF NOT EXISTS idx_claims_status ON claims(status);
CREATE INDEX IF NOT EXISTS idx_claims_ticket_code_submitted ON claims(ticket_code_submitted);
CREATE INDEX IF NOT EXISTS idx_claims_claimed_at ON claims(claimed_at);

CREATE INDEX IF NOT EXISTS idx_winners_awarded_at ON winners(awarded_at);
CREATE INDEX IF NOT EXISTS idx_winners_claim_id ON winners(claim_id);
