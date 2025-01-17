CREATE TABLE IF NOT EXISTS documents (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    owner_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    FOREIGN KEY (owner_id) REFERENCES accounts (id) ON DELETE CASCADE
);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_documents_owner_id ON documents (owner_id);

CREATE TRIGGER update_documents_updated_at BEFORE
UPDATE ON documents FOR EACH ROW EXECUTE FUNCTION update_updated_at_column ();
