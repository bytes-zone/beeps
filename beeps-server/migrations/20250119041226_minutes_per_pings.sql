CREATE TABLE IF NOT EXISTS minutes_per_pings (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    document_id BIGINT NOT NULL,
    -- value
    minutes_per_ping BIGINT NOT NULL,
    -- clock
    clock TIMESTAMPTZ NOT NULL,
    counter BIGINT NOT NULL,
    node_id BIGINT NOT NULL,
    -- references
    FOREIGN KEY (document_id) REFERENCES accounts (id) ON DELETE CASCADE,
    UNIQUE (document_id, clock, counter, node_id)
);

CREATE INDEX IF NOT EXISTS idx_minutes_per_pings_document_id ON minutes_per_pings (document_id);
