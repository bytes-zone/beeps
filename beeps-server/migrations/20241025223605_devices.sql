CREATE TABLE devices (
    id SERIAL PRIMARY KEY,
    document_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    node_id INT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id),
    UNIQUE (document_id, node_id)
);

CREATE INDEX idx_document_id_node_id ON devices (document_id, node_id);
