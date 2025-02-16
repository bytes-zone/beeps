CREATE TABLE IF NOT EXISTS pings (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- value
    ping TIMESTAMP NOT NULL,
    -- references
    UNIQUE (ping)
);
