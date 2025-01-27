CREATE TABLE IF NOT EXISTS tags (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    document_id BIGINT NOT NULL,
    -- value
    ping TIMESTAMPTZ NOT NULL,
    tag TEXT,
    -- clock
    timestamp TIMESTAMPTZ NOT NULL,
    counter INTEGER NOT NULL,
    node INTEGER NOT NULL,
    -- references
    FOREIGN KEY (document_id) REFERENCES accounts (id) ON DELETE CASCADE,
    UNIQUE (document_id, timestamp, counter, node)
);

CREATE INDEX IF NOT EXISTS idx_tags_document_id ON minutes_per_pings (document_id);
