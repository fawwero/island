# Island — IPC-контракт (v1.0, S1)

Единый источник истины по командам и событиям. Основание: ТЗ §5.2/§5.3 (+дополнения v1.1).
Rust-dto: `app/src-tauri/src/core/events.rs`. TS-типы зеркалируются вручную до подключения ts-rs (S2, задача в бэклоге) — файл `app/src/types.ts`.
Любое изменение контракта = ADR + обновление этого файла + обоих языков в одном PR.

## Команды (invoke)

| Команда | Вход | Выход | Модуль | Статус |
| --- | --- | --- | --- | --- |
| overlay_set_interactive | { interactive: bool } | — | M1 | S2 |
| overlay_set_state | { state: 'idle'\|'hover'\|'expanded' } | — | M1 | S2 |
| monitor_list | — | MonitorInfo[] | M1 | S1 (реализация S2) |
| launcher_toggle | — | — | M2 | S3 |
| launcher_search | { query: string } | LaunchResult[] | M2 | S3 |
| launcher_run | { resultId: number } | — | M2 | S3 |
| ai_send_message | { sessionId, text, docIds? } | { streamId } | M3 | S5 |
| ai_cancel | { streamId } | — | M3 | S5 |
| ai_attach_document | { path } | { docId } | M3 | S7 |
| ai_upload_document | { docId } | { geminiFileUri } | M3 | S7 |
| ai_set_provider | { provider, model, endpoint? } | — | M3 | S5 |
| secret_set | { name, value } | — | M3 | S5 |
| secret_exists | { name } | boolean | M3 | S1 |
| capture_start | { mode: 'screenshot'\|'ocr' } | — | M4 | S4 |
| capture_cancel | — | — | M4 | S4 |
| shelf_add / shelf_remove / shelf_list / shelf_clear / shelf_start_drag | см. ТЗ | ShelfItem[] / — | M5 | S8 |
| clipboard_list | { query?, limit, offset } | ClipItem[] | M6 | S4 |
| clipboard_paste | { id } | — | M6 | S4 |
| clipboard_pin / clipboard_delete / clipboard_clear | { id } / — | — | M6 | S4 |
| clipboard_pause | { paused: bool } | — | M6 | S4 |
| media_control | { action: 'play'\|'pause'\|'next'\|'prev' } | — | M8 | S9 |
| stt_start / stt_stop | — | — | M7 | S10 |
| weather_get | { force?: bool } | Forecast | M9 | S10 |
| settings_get | { key } | json | M11 | S1 |
| settings_set | { key, value } | boolean (changed) | M11 | S1 |
| settings_reset | — | — | M11 | S5 |
| settings_export / settings_import | { path } / { json } | — / number | M11 | S5 |
| app_check_update | — | UpdateInfo | общий | S12 (Could) |

## События (emit, Rust → UI)

| Событие | Payload | Источник | Статус |
| --- | --- | --- | --- |
| cursor:near | CursorNear{monitorId,x,y,distance} | M1 | S2 |
| fullscreen:changed | { active } | M1 | S2 |
| monitor:changed | MonitorInfo[] | M1 | S2 |
| launcher:visibility | { visible } | M2 | S3 |
| ai:token / ai:done / ai:error | AiToken/AiDone/AiError | M3 | S5 |
| capture:result | CaptureResult | M4 | S4 |
| shelf:changed | ShelfItem[] | M5 | S8 |
| clipboard:new | ClipboardNew{item} | M6 | S3 |
| activity:update | ActivityUpdate | M7 | S9/S10 |
| stt:state | SttState | M7 | S10 |
| media:update | MediaUpdate | M8 | S9 |
| weather:update | Forecast | M9 | S10 |
| settings:changed | SettingsChanged{key,value} | все | S1 |

## Правила payload
1. Все поля camelCase (`#[serde(rename_all = "camelCase")]`).
2. Ошибки invoke: `{ code, messageKey, params }` (core/error.rs); содержимое буфера/API-ключи в payload событий запрещены (только preview ≤500 символов после фильтра секретов).
3. Секреты: get-команда для значения секрета из UI не существует; только `secret_exists`.
