CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS history (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    content_type TEXT NOT NULL,
    content TEXT NOT NULL,
    favicon TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);