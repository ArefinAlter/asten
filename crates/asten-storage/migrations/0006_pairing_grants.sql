-- Pairing codes record the kind of token they grant. Existing codes grant
-- guest tokens, which is what they always did.
ALTER TABLE pairing_codes ADD COLUMN grants TEXT NOT NULL DEFAULT 'guest'
    CHECK (grants IN ('desk', 'guest', 'session'));
