-- 001_init.sql — базовая схема Island (ТЗ §5.4). Миграция N001.
-- Версионирование: таблица schema_migrations ведётся приложением (core/db.rs).

CREATE TABLE IF NOT EXISTS settings (
    key        TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL            -- unix ms
);

CREATE TABLE IF NOT EXISTS clipboard_history (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    kind         TEXT NOT NULL CHECK (kind IN ('text','url','image','files')),
    content_enc  BLOB,                     -- AES-256-GCM: текст/URL/JSON путей (NULL для image)
    blob_path    TEXT,                     -- зашифрованный PNG в кэше (для image)
    content_hash TEXT NOT NULL,            -- SHA-256 hex для дедупликации
    preview      TEXT NOT NULL DEFAULT '', -- нешифрованное превью <=500 символов (без секретов: фильтруется до записи)
    source_app   TEXT,
    is_pinned    INTEGER NOT NULL DEFAULT 0,
    created_at   INTEGER NOT NULL,
    expires_at   INTEGER
);
CREATE INDEX IF NOT EXISTS idx_clip_hash ON clipboard_history(content_hash);
CREATE INDEX IF NOT EXISTS idx_clip_created ON clipboard_history(created_at);

CREATE TABLE IF NOT EXISTS shelf_files (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    path     TEXT NOT NULL UNIQUE,
    name     TEXT NOT NULL,
    size     INTEGER NOT NULL,
    mime     TEXT,
    is_dir   INTEGER NOT NULL DEFAULT 0,
    status   TEXT NOT NULL DEFAULT 'ok' CHECK (status IN ('ok','missing')),
    added_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_sessions (
    id          TEXT PRIMARY KEY,          -- uuid
    title       TEXT NOT NULL,
    provider    TEXT NOT NULL DEFAULT 'gemini',
    model       TEXT,
    created_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_messages (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES ai_sessions(id) ON DELETE CASCADE,
    role       TEXT NOT NULL CHECK (role IN ('user','model','system')),
    content    TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ai_msg_session ON ai_messages(session_id, created_at);

CREATE TABLE IF NOT EXISTS ai_documents (
    id                    TEXT PRIMARY KEY, -- uuid
    path                  TEXT NOT NULL,
    hash                  TEXT NOT NULL,
    mime                  TEXT NOT NULL,
    size                  INTEGER NOT NULL,
    gemini_file_uri       TEXT,
    gemini_file_expires_at INTEGER,
    created_at            INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_cache (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    prompt_hash TEXT NOT NULL,
    provider    TEXT NOT NULL,
    model       TEXT NOT NULL,
    response    TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    ttl         INTEGER NOT NULL DEFAULT 0   -- сек; 0 = без истечения
);
CREATE INDEX IF NOT EXISTS idx_ai_cache_prompt ON ai_cache(prompt_hash);

CREATE TABLE IF NOT EXISTS app_index (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    target        TEXT NOT NULL UNIQUE,     -- путь или AUMID
    icon_path     TEXT,
    launch_count  INTEGER NOT NULL DEFAULT 0,
    last_launched INTEGER
);
CREATE INDEX IF NOT EXISTS idx_app_name ON app_index(name);
CREATE INDEX IF NOT EXISTS idx_app_launch ON app_index(launch_count);

CREATE TABLE IF NOT EXISTS weather_cache (
    location     TEXT PRIMARY KEY,
    payload_json TEXT NOT NULL,
    fetched_at   INTEGER NOT NULL
);
