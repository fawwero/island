// ipc/secrets_cmd.rs — команды секретов (M3/M11).
// Значение секрета НИКОГДА не возвращается в UI (ТЗ §3 «Безопасность: ключи»).

use crate::core::error::AppError;
use crate::core::secrets::{names as secret_names, SecretsStore};

/// Разрешённые имена секретов — белый список (защита от произвольных записей в Credential Manager).
const ALLOWED: &[&str] = &[secret_names::GEMINI_API_KEY];

fn check_name(name: &str) -> Result<(), AppError> {
    if ALLOWED.contains(&name) {
        Ok(())
    } else {
        Err(AppError::new("invalid_value", "errors.secretName"))
    }
}

#[tauri::command]
pub fn secret_set(name: String, value: String) -> Result<(), AppError> {
    check_name(&name)?;
    SecretsStore::set(&name, &value)?;
    Ok(())
}

#[tauri::command]
pub fn secret_exists(name: String) -> Result<bool, AppError> {
    check_name(&name)?;
    Ok(SecretsStore::exists(&name)?)
}
