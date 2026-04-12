CREATE TABLE IF NOT EXISTS settings (
    user_sub TEXT PRIMARY KEY,
    api_key TEXT NOT NULL DEFAULT '',
    base_url TEXT NOT NULL DEFAULT 'https://api.openai.com/v1',
    model TEXT NOT NULL DEFAULT 'gpt-4o-mini',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
