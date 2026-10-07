// core/secrets.rs — хранение секретов (API-ключи) вне БД/конфигов/логов.
// Windows: Credential Manager через keyring; entry "island/gemini_api_key".
// Вне Windows (только для unit-тестов ядра): in-memory fallback, НЕ для продакшена.

use keyring::Entry;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecretError {
    #[error("keyring unavailable: {0}")]
    Keyring(String),
}

const SERVICE: &str = "island";

/// Имена секретеов — стабильные ключи контракта.
pub mod names {
    pub const GEMINI_API_KEY: &str = "gemini_api_key";
}

pub struct SecretsStore;

impl SecretsStore {
    fn entry(name: &str) -> Result<Entry, SecretError> {
        Entry::new(SERVICE, name).map_err(|e| SecretError::Keyring(e.to_string()))
    }

    /// Сохранить секрет. Значение никогда не возвращается наружу через UI-состояния.
    pub fn set(name: &str, value: &str) -> Result<(), SecretError> {
        #[cfg(target_os = "windows")]
        {
            if value.is_empty() {
                Self::delete(name)?;
                return Ok(());
            }
            Self::entry(name)?
                .set_password(value)
                .map_err(|e| SecretError::Keyring(e.to_string()))
        }
        #[cfg(not(target_os = "windows"))]
        {
            // Тестовый fallback: только для проверки логики вызовов на CI без Windows.
            crate::core::secrets::test_backend::with_store(|s| {
                if value.is_empty() {
                    s.remove(name);
                } else {
                    s.insert(name.to_string(), value.to_string());
                }
            });
            Ok(())
        }
    }

    pub fn get(name: &str) -> Result<Option<String>, SecretError> {
        #[cfg(target_os = "windows")]
        {
            match Self::entry(name)?.get_password() {
                Ok(v) => Ok(Some(v)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(SecretError::Keyring(e.to_string())),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(crate::core::secrets::test_backend::with_store(|s| s.get(name).cloned()))
        }
    }

    pub fn exists(name: &str) -> Result<bool, SecretError> {
        Ok(Self::get(name)?.is_some())
    }

    pub fn delete(name: &str) -> Result<(), SecretError> {
        #[cfg(target_os = "windows")]
        {
            match Self::entry(name)?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(SecretError::Keyring(e.to_string())),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            crate::core::secrets::test_backend::with_store(|s| {
                s.remove(name);
            });
            Ok(())
        }
    }
}

/// In-memory бэкенд для тестов на не-Windows (не компилируется в Windows-сборку).
#[cfg(not(target_os = "windows"))]
pub mod test_backend {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

    pub fn with_store<R>(f: impl FnOnce(&mut HashMap<String, String>) -> R) -> R {
        let m = STORE.get_or_init(|| Mutex::new(HashMap::new()));
        let mut guard = m.lock().unwrap();
        f(&mut guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_delete_roundtrip() {
        SecretsStore::set(names::GEMINI_API_KEY, "test-key-123").unwrap();
        assert!(SecretsStore::exists(names::GEMINI_API_KEY).unwrap());
        assert_eq!(
            SecretsStore::get(names::GEMINI_API_KEY).unwrap().as_deref(),
            Some("test-key-123")
        );
        SecretsStore::delete(names::GEMINI_API_KEY).unwrap();
        assert!(!SecretsStore::exists(names::GEMINI_API_KEY).unwrap());
    }

    #[test]
    fn empty_value_deletes() {
        SecretsStore::set(names::GEMINI_API_KEY, "k").unwrap();
        SecretsStore::set(names::GEMINI_API_KEY, "").unwrap();
        assert!(!SecretsStore::exists(names::GEMINI_API_KEY).unwrap());
    }
}
