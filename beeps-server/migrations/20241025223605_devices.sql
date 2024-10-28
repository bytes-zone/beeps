CREATE TABLE devices (
    id SERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id),
    UNIQUE (document_id, name)
);
