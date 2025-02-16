CREATE TABLE IF NOT EXISTS tags (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- value
    ping TIMESTAMP NOT NULL,
    tag TEXT,
    -- clock
    timestamp TIMESTAMP NOT NULL,
    counter INTEGER NOT NULL,
    node INTEGER NOT NULL,
    -- references
    UNIQUE (timestamp, counter, node)
);
