// core/bus.rs — типизированная обёртка над Tauri EventTarget::emit.
// Модули логики держат Arc<dyn Emitter>, что позволяет мокать события в тестах.

use serde::Serialize;

use crate::core::events::*;

pub trait Emitter: Send + Sync {
    fn emit(&self, event: &str, payload: &Value2);
}

/// Универсальный payload-обёртка (уже сериализуемое dto).
pub type Value2 = serde_json::Value;

fn to_payload<T: Serialize>(v: &T) -> Value2 {
    serde_json::to_value(v).expect("dto must serialize")
}

/// Реализация поверх tauri::AppHandle.
pub struct TauriEmitter(pub tauri::AppHandle);

impl Emitter for TauriEmitter {
    fn emit(&self, event: &str, payload: &Value2) {
        use tauri::Emitter as _;
        if let Err(e) = self.0.emit(event, payload.clone()) {
            log::warn!("emit {event} failed: {e}");
        }
    }
}

/// Мок для unit-тестов: собирает события в вектор.
#[cfg(test)]
pub struct CollectingEmitter(pub std::sync::Mutex<Vec<(String, Value2)>>);

#[cfg(test)]
impl Emitter for CollectingEmitter {
    fn emit(&self, event: &str, payload: &Value2) {
        self.0
            .lock()
            .unwrap()
            .push((event.to_string(), payload.clone()));
    }
}

/// Хелперы доменных событий — единые точки формирования payload.
pub struct EventBus(pub std::sync::Arc<dyn Emitter>);

impl EventBus {
    pub fn cursor_near(&self, m: u32, x: f64, y: f64, distance: f64) {
        self.emit_typed(
            events::CURSOR_NEAR,
            &CursorNear {
                monitor_id: m,
                x,
                y,
                distance,
            },
        );
    }
    pub fn fullscreen_changed(&self, active: bool) {
        self.emit_typed(events::FULLSCREEN_CHANGED, &FullscreenChanged { active });
    }
    pub fn monitor_changed(&self, monitors: Vec<MonitorInfo>) {
        self.emit_typed(events::MONITOR_CHANGED, &monitors);
    }
    pub fn launcher_visibility(&self, visible: bool) {
        self.emit_typed(events::LAUNCHER_VISIBILITY, &LauncherVisibility { visible });
    }
    pub fn ai_token(&self, stream_id: String, text: String) {
        self.emit_typed(events::AI_TOKEN, &AiToken { stream_id, text });
    }
    pub fn ai_done(&self, stream_id: String) {
        self.emit_typed(events::AI_DONE, &AiDone { stream_id });
    }
    pub fn ai_error(&self, err: crate::core::error::AppError) {
        let e = AiError {
            stream_id: None,
            code: err.code.to_string(),
            message_key: err.message_key.to_string(),
            params: err.params,
        };
        self.emit_typed(events::AI_ERROR, &e);
    }
    pub fn capture_result(&self, r: CaptureResult) {
        self.emit_typed(events::CAPTURE_RESULT, &r);
    }
    pub fn shelf_changed(&self, items: Vec<ShelfItem>) {
        self.emit_typed(events::SHELF_CHANGED, &items);
    }
    pub fn clipboard_new(&self, item: ClipItem) {
        self.emit_typed(events::CLIPBOARD_NEW, &ClipboardNew { item });
    }
    pub fn activity_update(&self, a: ActivityUpdate) {
        self.emit_typed(events::ACTIVITY_UPDATE, &a);
    }
    pub fn stt_state(&self, s: SttState) {
        self.emit_typed(events::STT_STATE, &s);
    }
    pub fn media_update(&self, m: MediaUpdate) {
        self.emit_typed(events::MEDIA_UPDATE, &m);
    }
    pub fn settings_changed(&self, key: &str, value: Value2) {
        self.emit_typed(
            events::SETTINGS_CHANGED,
            &SettingsChanged {
                key: key.to_string(),
                value,
            },
        );
    }

    fn emit_typed<T: Serialize>(&self, event: &str, payload: &T) {
        self.0.emit(event, &to_payload(payload));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_emits_typed_payload_shape() {
        let em = std::sync::Arc::new(CollectingEmitter(std::sync::Mutex::new(vec![])));
        let bus = EventBus(em.clone());
        bus.settings_changed("general.language", serde_json::json!("en"));
        let evs = em.0.lock().unwrap();
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].0, "settings:changed");
        assert_eq!(evs[0].1["key"], "general.language");
        assert_eq!(evs[0].1["value"], "en");
    }
}
