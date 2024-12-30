CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    password TEXT NOT NULL,
    UNIQUE (email)
);
