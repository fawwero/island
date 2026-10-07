// types.ts — зеркало Rust-dto (app/src-tauri/src/core/events.rs).
// Контракт: docs/ipc.md. Изменять только синхронно с Rust в одном PR.

export type IslandState = "idle" | "hover" | "expanded" | "activity";
export type ClipKind = "text" | "url" | "image" | "files";
export type ShelfStatus = "ok" | "missing";
export type CaptureKind = "screenshot" | "ocr";
export type ActivityKind = "download" | "record" | "mic" | "stt";
export type SttPhase = "idle" | "listening" | "processing";
export type MediaStatus = "playing" | "paused" | "stopped";

export interface MonitorInfo {
  id: number;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  dpi: number;
  isPrimary: boolean;
  islandEnabled: boolean;
}

export interface CursorNear {
  monitorId: number;
  x: number;
  y: number;
  distance: number;
}

export interface FullscreenChanged {
  active: boolean;
}

export interface LauncherVisibility {
  visible: boolean;
}

export interface ClipItem {
  id: number;
  kind: ClipKind;
  preview: string;
  sourceApp: string | null;
  isPinned: boolean;
  createdAt: number;
}

export interface ClipboardNew {
  item: ClipItem;
}

export interface ShelfItem {
  id: number;
  path: string;
  name: string;
  size: number;
  mime: string | null;
  isDir: boolean;
  status: ShelfStatus;
  addedAt: number;
}

export interface AiToken {
  streamId: string;
  text: string;
}

export interface AiDone {
  streamId: string;
}

export interface AiError {
  streamId: string | null;
  code: string;
  messageKey: string;
  params: Record<string, unknown>;
}

export interface CaptureResult {
  kind: CaptureKind;
  path: string | null;
  text: string | null;
}

export interface ActivityUpdate {
  kind: ActivityKind;
  active: boolean;
  label: string | null;
  bytesDone: number | null;
  bytesPerSec: number | null;
}

export interface SttState {
  state: SttPhase;
  text: string | null;
}

export interface MediaUpdate {
  title: string;
  artist: string;
  album: string | null;
  thumbnailB64: string | null;
  status: MediaStatus;
  positionMs: number;
  durationMs: number | null;
}

export interface SettingsChanged {
  key: string;
  value: unknown;
}

// Единый формат ошибки invoke (core/error.rs).
export interface AppErrorPayload {
  code: string;
  messageKey: string;
  params: Record<string, unknown>;
}

export function isAppErrorPayload(e: unknown): e is AppErrorPayload {
  return (
    typeof e === "object" &&
    e !== null &&
    "code" in e &&
    "messageKey" in e &&
    "params" in e
  );
}
