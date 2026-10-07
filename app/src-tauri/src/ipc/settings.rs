// ipc/settings.rs — команды настроек (M11). Тонкие обёртки над SettingsStore.
// Контракт: docs/ipc.md. Ошибки — AppError ({code, messageKey, params}).

use serde_json::Value;
use tauri::{Manager, State};

use crate::core::error::AppError;
use crate::AppState;

#[tauri::command]
pub fn settings_get(state: State<AppState>, key: String) -> Result<Value, AppError> {
    Ok(state.settings.get(&key))
}

#[tauri::command]
pub fn settings_set(
    state: State<AppState>,
    key: String,
    value: Value,
) -> Result<bool, AppError> {
    // валидация типа по схеме defaults (поверхностная: JSON-значение обязательно)
    if value.is_null() && !matches!(key.as_str(), _) {
        return Err(AppError::new("invalid_value", "errors.invalidValue"));
    }
    let changed = state.settings.set(&key, value.clone());
    if changed {
        state.bus.settings_changed(&key, value);
    }
    Ok(changed)
}

#[tauri::command]
pub fn settings_all(state: State<AppState>) -> Result<Value, AppError> {
    Ok(Value::Object(state.settings.all()))
}

#[tauri::command]
pub fn settings_reset(app: tauri::AppHandle, state: State<AppState>) -> Result<(), AppError> {
    state.settings.reset_to_defaults();
    use tauri::Emitter;
    let _ = app.emit("settings:changed", serde_json::json!({ "key": "*", "value": null }));
    Ok(())
}
