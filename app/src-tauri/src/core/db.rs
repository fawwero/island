// core/db.rs — открытие SQLite (bundled), WAL, версионируемые миграции.
// Тестируется на любой ОС (без Tauri-зависимостей) — ядро для unit-тестов CI.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("migration {name} failed: {source}")]
    Migration {
        name: String,
        source: rusqlite::Error,
    },
    #[error("unknown migration version {0}")]
    UnknownVersion(i64),
}

/// Список миграций: (version, name, sql). Порядок = порядок применения.
/// Новые миграции добавляются в конец; существующие не редактируются.
pub const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "001_init", include_str!("../../migrations/001_init.sql")),
];

pub struct Db {
    pub conn: Mutex<Connection>,
}

impl Db {
    /// Открыть (или создать) БД по пути и применить миграции.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        Self::init_conn(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// In-memory БД для тестов.
    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        Self::init_conn(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn init_conn(conn: &Connection) -> Result<(), DbError> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version    INTEGER PRIMARY KEY,
                name       TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            );",
        )?;
        migrate(conn)?;
        Ok(())
    }

    /// Текущая применённая версия схемы.
    pub fn version(&self) -> Result<i64, DbError> {
        let conn = self.conn.lock().unwrap();
        let v: i64 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )?;
        Ok(v)
    }
}

fn migrate(conn: &Connection) -> Result<(), DbError> {
    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |r| r.get(0),
    )?;
    for (version, name, sql) in MIGRATIONS {
        if *version <= current {
            continue;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        conn.execute_batch(sql)
            .map_err(|e| DbError::Migration {
                name: (*name).to_string(),
                source: e,
            })?;
        conn.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![version, name, now],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_and_are_idempotent() {
        let db = Db::open_in_memory().expect("open");
        assert_eq!(db.version().unwrap(), 1);
        // повторная инициализация не падает (идемпотентность apply)
        let conn = db.conn.into_inner().unwrap();
        super::migrate(&conn).expect("second migrate");
        let v: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 1);
    }

    #[test]
    fn all_tables_exist_after_migration() {
        let db = Db::open_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let names: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        for t in [
            "settings",
            "clipboard_history",
            "shelf_files",
            "ai_sessions",
            "ai_messages",
            "ai_documents",
            "ai_cache",
            "app_index",
            "weather_cache",
            "schema_migrations",
        ] {
            assert!(names.iter().any(|n| n == t), "missing table {t}");
        }
    }

    #[test]
    fn wal_enabled_on_file_db() {
        let dir = std::env::temp_dir().join(format!("island-db-test-{}", std::process::id()));
        let path = dir.join("island.db");
        let db = Db::open(&path).unwrap();
        let conn = db.conn.lock().unwrap();
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
        drop(conn);
        drop(db);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
