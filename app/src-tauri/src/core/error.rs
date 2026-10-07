// core/error.rs — единая ошибка приложения, сериализуемая для UI в { code, messageKey, params }.

use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub struct AppError {
    pub code: &'static str,
    pub message_key: &'static str,
    pub params: Map<String, Value>,
    pub detail: Option<String>, // только в логи, не в UI
}

impl AppError {
    pub fn new(code: &'static str, message_key: &'static str) -> Self {
        Self {
            code,
            message_key,
            params: Map::new(),
            detail: None,
        }
    }
    pub fn with(mut self, key: &str, value: Value) -> Self {
        self.params.insert(key.into(), value);
        self
    }
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message_key)?;
        if let Some(d) = &self.detail {
            write!(f, " ({d})")?;
        }
        Ok(())
    }
}

impl std::error::Error for AppError {}

/// Сериализация строго по контракту (ARCHITECTURE §4.4).
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("AppError", 3)?;
        st.serialize_field("code", self.code)?;
        st.serialize_field("messageKey", self.message_key)?;
        st.serialize_field("params", &self.params)?;
        st.end()
    }
}

impl From<crate::core::db::DbError> for AppError {
    fn from(e: crate::core::db::DbError) -> Self {
        AppError::new("db", "errors.db").detail(e.to_string())
    }
}
impl From<crate::core::secrets::SecretError> for AppError {
    fn from(e: crate::core::secrets::SecretError) -> Self {
        AppError::new("secret", "errors.secret").detail(e.to_string())
    }
}

/// Коды ошибок — контракт с i18n ключами UI.
pub mod codes {
    pub const DB: &str = "db";
    pub const SECRET: &str = "secret";
    pub const HOTKEY_CONFLICT: &str = "hotkey_conflict";
    pub const NOT_FOUND: &str = "not_found";
    pub const INVALID_VALUE: &str = "invalid_value";
    pub const AI_NO_KEY: &str = "ai_no_key";
    pub const AI_NO_NETWORK: &str = "ai_no_network";
    pub const AI_RATE_LIMIT: &str = "ai_rate_limit";
    pub const AI_MODEL_UNAVAILABLE: &str = "ai_model_unavailable";
    pub const OCR_NO_LANGUAGE: &str = "ocr_no_language";
    pub const CAPTURE_PROTECTED: &str = "capture_protected";
}
