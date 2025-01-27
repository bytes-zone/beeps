CREATE TABLE IF NOT EXISTS minutes_per_pings (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    document_id BIGINT NOT NULL,
    -- value
    minutes_per_ping INTEGER NOT NULL,
    -- clock
    timestamp TIMESTAMPTZ NOT NULL,
    counter INTEGER NOT NULL,
    node INTEGER NOT NULL,
    -- references
    FOREIGN KEY (document_id) REFERENCES accounts (id) ON DELETE CASCADE,
    UNIQUE (document_id, timestamp, counter, node)
);

CREATE INDEX IF NOT EXISTS idx_minutes_per_pings_document_id ON minutes_per_pings (document_id);
