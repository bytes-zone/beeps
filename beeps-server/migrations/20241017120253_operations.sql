CREATE TABLE operations (
    id SERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    counter BIGINT NOT NULL,
    node SMALLINT NOT NULL,
    op JSONB NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id)
);

CREATE INDEX idx_document_node_timestamp_counter_desc ON operations (document_id, node, timestamp DESC, counter DESC);
