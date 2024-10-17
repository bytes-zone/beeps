CREATE TABLE operations (
    id SERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    counter BIGINT NOT NULL,
    node SMALLINT NOT NULL,
    op JSONB NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

CREATE INDEX idx_account_node_timestamp_counter_desc ON operations (account_id, node, timestamp DESC, counter DESC);
