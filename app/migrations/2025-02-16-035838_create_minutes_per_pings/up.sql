CREATE TABLE IF NOT EXISTS minutes_per_pings (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- value
    minutes_per_ping INTEGER NOT NULL,
    -- clock
    timestamp TIMESTAMP NOT NULL,
    counter INTEGER NOT NULL,
    node INTEGER NOT NULL,
    -- references
    UNIQUE (timestamp, counter, node)
);
