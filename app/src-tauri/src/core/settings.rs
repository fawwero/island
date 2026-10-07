// core/settings.rs — хранилище настроек (таблица settings) со значениями по умолчанию.
// Ключи вида "domain.param" (например "hotkeys.launcher", "clipboard.limit").
// Ядро не зависит от Tauri: событие об изменении возвращается вызывающему слою.

use std::sync::Arc;

use rusqlite::OptionalExtension;
use serde_json::Value;

use crate::core::db::Db;

pub const SCHEMA_VERSION_KEY: &str = "settings.schema_version";
pub const CURRENT_SCHEMA_VERSION: i64 = 1;

/// Значения по умолчанию (ТЗ §5.4, ARCHITECTURE §8). Полное покрытие MVP-модулей.
pub fn defaults() -> Vec<(&'static str, Value)> {
    vec![
        // Общие
        (SCHEMA_VERSION_KEY, Value::from(CURRENT_SCHEMA_VERSION)),
        ("general.language", Value::from("ru")),
        ("general.autostart", Value::from(false)),
        ("general.tray_paused", Value::from(false)),
        // Поведение острова
        ("island.hover_distance_px", Value::from(120.0)),
        ("island.click_through", Value::from(true)),
        ("island.fps_limit", Value::from(60)),
        ("island.autohide_fullscreen", Value::from(true)),
        ("island.game_blocklist", Value::from(Vec::<String>::new())),
        // Хоткеи (дефолты из финализации ТЗ 9.3)
        ("hotkeys.launcher", Value::from("Alt+Space")),
        ("hotkeys.screenshot", Value::from("Ctrl+Alt+S")),
        ("hotkeys.ocr", Value::from("Ctrl+Alt+X")),
        ("hotkeys.dictation", Value::from("Win+Alt+V")),
        ("hotkeys.dictation_fallback", Value::from("Ctrl+Alt+D")),
        // Модули (вкл/выкл)
        ("modules.clipboard", Value::from(true)),
        ("modules.launcher", Value::from(true)),
        ("modules.capture", Value::from(true)),
        // Лончер
        ("launcher.index_start_menu", Value::from(true)),
        ("launcher.index_uwp", Value::from(true)),
        // Буфер обмена (M6 MVP: текст/URL)
        ("clipboard.max_items", Value::from(200)),
        ("clipboard.retention_days", Value::from(30)),
        ("clipboard.max_text_bytes", Value::from(1_048_576)), // 1 МБ
        ("clipboard.process_blacklist", Value::from(Vec::<String>::new())),
        ("clipboard.paused", Value::from(false)),
        // ИИ (M3)
        ("ai.provider", Value::from("gemini")),
        ("ai.model", Value::from("")),            // "" => автоподбор
        ("ai.model_fallback", Value::from("gemini-2.5-flash")),
        ("ai.consent_free_tier", Value::from(false)), // предупреждение показано?
        // Прочее (используется в полной версии; ключи фиксируем сейчас ради стабильности схемы)
        ("shelf.max_items", Value::from(100)),
        ("weather.city", Value::from("")),
        ("weather.units", Value::from("c")),
        ("residents.resident_id", Value::from("none")),
        ("power.battery_saver", Value::from(false)),
    ]
}

pub struct SettingsStore {
    db: Arc<Db>,
}

impl SettingsStore {
    pub fn new(db: Arc<Db>) -> Self {
        let store = Self { db };
        store.ensure_defaults();
        store
    }

    /// Вставляет отсутствующие ключи со значениями по умолчанию (идемпотентно).
    fn ensure_defaults(&self) {
        let conn = self.db.conn.lock().unwrap();
        let now = now_ms();
        for (key, value) in defaults() {
            let exists: Option<i64> = conn
                .query_row(
                    "SELECT 1 FROM settings WHERE key = ?1",
                    rusqlite::params![key],
                    |r| r.get(0),
                )
                .optional()
                .unwrap();
            if exists.is_none() {
                conn.execute(
                    "INSERT INTO settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![key, value.to_string(), now],
                )
                .unwrap();
            }
        }
    }

    /// Получить значение: сначала таблица, затем default, иначе Null.
    pub fn get(&self, key: &str) -> Value {
        let conn = self.db.conn.lock().unwrap();
        let raw: Option<String> = conn
            .query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                rusqlite::params![key],
                |r| r.get(0),
            )
            .optional()
            .unwrap();
        match raw {
            Some(s) => serde_json::from_str(&s).unwrap_or(Value::Null),
            None => defaults()
                .into_iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v)
                .unwrap_or(Value::Null),
        }
    }

    /// Установить значение. Возвращает true, если значение изменилось (для emit события).
    pub fn set(&self, key: &str, value: Value) -> bool {
        let old = self.get(key);
        if old == value {
            return false;
        }
        let conn = self.db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
            rusqlite::params![key, value.to_string(), now_ms()],
        )
        .unwrap();
        true
    }

    /// Все настройки (для UI/экспорта). Секретов здесь нет по построению (ключи — в Credential Manager).
    pub fn all(&self) -> serde_json::Map<String, Value> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value_json FROM settings ORDER BY key").unwrap();
        let mut map = serde_json::Map::new();
        let rows = stmt
            .query_map([], |r| {
                let k: String = r.get(0)?;
                let v: String = r.get(1)?;
                Ok((k, v))
            })
            .unwrap();
        for row in rows {
            let (k, v) = row.unwrap();
            if let Ok(parsed) = serde_json::from_str::<Value>(&v) {
                map.insert(k, parsed);
            }
        }
        for (k, v) in defaults() {
            map.entry(k).or_insert(v);
        }
        map
    }

    /// Сброс к умолчаниям (удаляет пользовательские значения, затем пересоздаёт defaults).
    pub fn reset_to_defaults(&self) {
        {
            let conn = self.db.conn.lock().unwrap();
            conn.execute("DELETE FROM settings", []).unwrap();
        }
        self.ensure_defaults();
    }

    /// Валидация при импорте: принимаем только известные ключи и JSON-совместимые значения.
    pub fn import_filtered(&self, doc: Value) -> usize {
        let mut count = 0;
        if let Value::Object(map) = doc {
            let allowed: Vec<&str> = defaults().iter().map(|(k, _)| *k).collect();
            for (k, v) in map {
                if allowed.contains(&k.as_str()) && self.set(&k, v) {
                    count += 1;
                }
            }
        }
        count
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn store() -> SettingsStore {
        SettingsStore::new(Arc::new(Db::open_in_memory().unwrap()))
    }

    #[test]
    fn defaults_present_after_init() {
        let s = store();
        assert_eq!(s.get("general.language"), json!("ru"));
        assert_eq!(s.get("hotkeys.launcher"), json!("Alt+Space"));
        assert_eq!(s.get("clipboard.max_items"), json!(200));
        assert_eq!(s.get("unknown.key"), Value::Null);
    }

    #[test]
    fn set_get_and_change_flag() {
        let s = store();
        assert!(s.set("general.language", json!("en")));
        assert_eq!(s.get("general.language"), json!("en"));
        assert!(!s.set("general.language", json!("en"))); // без изменения — false
    }

    #[test]
    fn reset_restores_defaults() {
        let s = store();
        s.set("clipboard.max_items", json!(50));
        s.reset_to_defaults();
        assert_eq!(s.get("clipboard.max_items"), json!(200));
    }

    #[test]
    fn import_filters_unknown_keys() {
        let s = store();
        let n = s.import_filtered(json!({
            "general.language": "en",
            "evil.secret_key": "x",
            "clipboard.max_items": 300
        }));
        assert_eq!(n, 2);
        assert_eq!(s.get("general.language"), json!("en"));
        assert_eq!(s.get("evil.secret_key"), Value::Null);
    }

    #[test]
    fn export_contains_no_secrets_marker() {
        // контракт: в таблице settings физически нет ключей ai.api_key (он в keyring)
        let s = store();
        let all = s.all();
        assert!(!all.contains_key("ai.api_key"));
    }
}
