CREATE TABLE IF NOT EXISTS pings (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    document_id BIGINT NOT NULL,
    -- value
    ping TIMESTAMPTZ NOT NULL,
    -- references
    FOREIGN KEY (document_id) REFERENCES accounts (id) ON DELETE CASCADE,
    UNIQUE (document_id, ping)
);

CREATE INDEX IF NOT EXISTS idx_pings_document_id ON minutes_per_pings (document_id);
