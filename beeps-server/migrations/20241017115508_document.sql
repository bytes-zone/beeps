CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);
