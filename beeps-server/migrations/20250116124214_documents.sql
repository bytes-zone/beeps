CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    owner_id INT NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE CASCADE
);

CREATE INDEX idx_documents_owner_id ON documents (owner_id);
