// Island core: единый типизированный event bus поверх Tauri emit.
// Имена событий — контракт (docs/ipc.md). Payload — serde-структуры dto.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub dpi: f64,
    pub is_primary: bool,
    pub island_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IslandState {
    Idle,
    Hover,
    Expanded,
    Activity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorNear {
    pub monitor_id: u32,
    pub x: f64,
    pub y: f64,
    /// расстояние от курсора до центра острова, physical px
    pub distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullscreenChanged {
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherVisibility {
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipKind {
    Text,
    Url,
    Image,
    Files,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem {
    pub id: i64,
    pub kind: ClipKind,
    /// расшифрованное превью (не более 500 символов), для UI; секреты фильтруются до записи
    pub preview: String,
    pub source_app: Option<String>,
    pub is_pinned: bool,
    pub created_at: i64, // unix ms
}

/// событие clipboard:new — без полного содержимого
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardNew {
    pub item: ClipItem,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShelfStatus {
    #[default]
    Ok,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfItem {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub mime: Option<String>,
    pub is_dir: bool,
    #[serde(default)]
    pub status: ShelfStatus,
    pub added_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiToken {
    pub stream_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDone {
    pub stream_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiError {
    pub stream_id: Option<String>,
    pub code: String,
    pub message_key: String,
    #[serde(default)]
    pub params: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptureKind {
    Screenshot,
    Ocr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResult {
    pub kind: CaptureKind,
    pub path: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityKind {
    Download,
    Record,
    Mic,
    Stt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityUpdate {
    pub kind: ActivityKind,
    pub active: bool,
    pub label: Option<String>,
    pub bytes_done: Option<u64>,
    pub bytes_per_sec: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SttPhase {
    Idle,
    Listening,
    Processing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SttState {
    pub state: SttPhase,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaStatus {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaUpdate {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub thumbnail_b64: Option<String>,
    pub status: MediaStatus,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsChanged {
    pub key: String,
    pub value: serde_json::Value,
}

/// Реестр имён событий — единственный источник истины (см. docs/ipc.md).
pub mod events {
    pub const CURSOR_NEAR: &str = "cursor:near";
    pub const FULLSCREEN_CHANGED: &str = "fullscreen:changed";
    pub const MONITOR_CHANGED: &str = "monitor:changed";
    pub const LAUNCHER_VISIBILITY: &str = "launcher:visibility";
    pub const AI_TOKEN: &str = "ai:token";
    pub const AI_DONE: &str = "ai:done";
    pub const AI_ERROR: &str = "ai:error";
    pub const CAPTURE_RESULT: &str = "capture:result";
    pub const SHELF_CHANGED: &str = "shelf:changed";
    pub const CLIPBOARD_NEW: &str = "clipboard:new";
    pub const ACTIVITY_UPDATE: &str = "activity:update";
    pub const STT_STATE: &str = "stt:state";
    pub const MEDIA_UPDATE: &str = "media:update";
    pub const WEATHER_UPDATE: &str = "weather:update";
    pub const SETTINGS_CHANGED: &str = "settings:changed";
}
