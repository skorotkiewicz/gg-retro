-- Add migration script here

CREATE TABLE users (
    uin INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Set the autoincrement to start at 100000000 (9 digits)
INSERT INTO sqlite_sequence (name, seq) VALUES ('users', 999999);
