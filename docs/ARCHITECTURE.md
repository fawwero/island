# Island — Архитектура (v1.0, зафиксирована)

Основание: ТЗ Island v1.1 (корень репозитория), включая раздел «Ответы и финализация открытых вопросов (9.3)».
Документ обязателен к соблюдению; изменения — только через ADR в `docs/adr/`.

## 1. Финальные решения по открытым вопросам ТЗ (закрыты)

| Вопрос | Решение |
| --- | --- |
| Лицензия | **MIT** (файл `LICENSE`); dual MIT/Apache-2.0 допускается для зависимостей, код приложения — MIT |
| Хоткеи скриншота/OCR | По умолчанию: **Ctrl+Alt+S** (скриншот), **Ctrl+Alt+X** (OCR). Win+Shift+S не перехватываем (занят Snipping Tool). Переназначаются в M11 |
| Полка файлов | Хранение путей в SQLite, очистка таблицы при каждом старте Rust-бэкенда; потеря списка при перезапуске подтверждена владельцем |
| Модель Gemini | Автоподбор: `GET /v1beta/models`, фильтр по маске `*flash*`, исключение deprecated; fallback — `gemini-2.5-flash`; выбор сохраняется в `settings.ai.model` |
| Страница настроек | Отдельное окно Tauri (не модалка в острове): сайдбар категорий слева + область контента справа; категории: Общие, Мониторы, Поведение, Хоткеи, Модули, Лончер, ИИ, Буфер, Полка, Индикаторы, Медиа, Погода, Жилец, Питание, Данные |
| Спрайты жильцов | Каталог `assets/residents/` + `LICENSES.md`; линтер CI проверяет соответствие (полная версия, S11) |

## 2. Стеки и версии (фиксируются в lock-файлах и rust-toolchain.toml)

| Слой | Выбор | Обоснование |
| --- | --- | --- |
| Язык бэкенда | Rust stable (`rust-toolchain.toml`: channel = "stable") | Требование Tauri 2 |
| Каркас | Tauri 2.x, crate `tauri` с feature `protocol-asset` | Оверлеи, плагины, ACL-capabilities |
| Рендерер | WebView2 Evergreen (штатный для Windows) | Системный на Win11 |
| Фронтенд | React 19 + TypeScript 5.x + Vite 6 | Реактивный UI, экосистема |
| Анимации/стили | Motion (`motion/react`) + Tailwind CSS 4.x | Spring-«жидкость», layout-анимации |
| State (UI) | Zustand | Лёгкий, без boilerplate |
| БД | rusqlite (bundled) из Rust + версионируемые миграции SQL | Полный контроль шифрования и WAL; плагин sql не даёт шифрования на уровне приложения |
| Секреты | keyring (Windows Credential Manager) | API-ключ вне БД/логов/UI |
| Шифрование буфера | aes-gcm 0.10 (AES-256-GCM), ключ обёрнут DPAPI (windows crate) | Требование M6 |
| WinAPI/WinRT | `windows` 0.58+ (WinRT: SMTC, OCR, Graphics.Capture), `windows-sys` допустим для чистого Win32 | См. отклонение в ТЗ §4 |
| Наблюдение папок | notify 6.x (ReadDirectoryChangesW) | Event-driven |
| HTTP/SSE | reqwest 0.12 (rustls) + futures-util | Gemini streaming; TLS без OpenSSL |
| Локализация UI | i18next + react-i18next, ресурсы RU/EN | Переключение без перезапуска |
| Глоб. хоткеи | tauri-plugin-global-shortcut (RegisterHotKey) | Безопасно для античитов |
| Прочее плагины | single-instance, autostart, dialog, log, deep-link (не нужен) — только разрешённые capability | MVP |
| Сборка/дистрибуция | GitHub Actions windows-latest: NSIS-установщик + portable ZIP в GitHub Releases | ТЗ §1, решение 18 |

Точные версии фиксируются в `Cargo.lock` / `package-lock.json` при скаффолдинге (S1).

## 3. Структура монорепозитория

```
island/                      # корень репозитория
├── LICENSE  README.md       # MIT, описание
├── rust-toolchain.toml      # stable
├── .github/workflows/       # ci.yml (check/test/lint), release.yml (win x64 bundle)
├── docs/                    # ТЗ, ARCHITECTURE.md, adr/, sprints/
├── app/                     # Tauri-приложение (workspace root для Rust)
│   ├── package.json         # npm workspace root (frontend dev/build)
│   ├── src-tauri/           # Rust: crate island
│   │   ├── Cargo.toml       # [dependencies] crossterm? нет; windows cfg(target-os="windows")
│   │   ├── tauri.conf.json  # identifier com.island.app, windows описываются кодом
│   │   ├── capabilities/default.json
│   │   ├── migrations/*.sql # версионируемые миграции
│   │   └── src/
│   │       ├── main.rs lib.rs
│   │       ├── core/        # db, settings, bus (typed events), secrets, errors
│   │       ├── modules/     # overlay launcher ai capture shelf clipboard activity media weather residents
│   │       └── ipc/         # #[tauri::command] thin wrappers -> modules
│   └── src/                 # React: ui/ (остров, лончер, селектор, настройки), i18n/, store/, types.ts
└── assets/                  # спрайты (S11), иконки
```

Модуль ≠ окно: Rust-модули инкапсулируют логику; окна создаются модулем `overlay`/ipc.

## 4. Слои и границы ответственности

```
React UI (тонкий) --invoke--> ipc::* commands --> modules/* (логика) --> core/* (db/settings/secrets/bus)
React UI <--Tauri events (typed bus)--- modules/* <-- WinAPI/WinRT/notify/reqwest
```

Правила:
1. Вся системная и сетевая логика — только в Rust. UI не имеет прямого доступа к fs/http/shell (capability-минимализм).
2. Каждый module реализует публичный трейт-фасад (`LauncherApi`, `ClipboardApi`, ...) — точечная тестируемость и замена.
3. События — типизированный перечень (см. §6), имена стабильны, payload — serde-структуры из `ipc/dto.rs`, общие типы генерируются в TS (`ts-rs`).
4. Ошибки: единый `core::error::AppError`, маппится в `{ code, message_key, params }` для i18n в UI.
5. Платформенные части изолированы `#[cfg(target_os = "windows")]`; non-Windows компилируется и запускает unit-тесты ядра (CI на ubuntu может гонять логику без окон).

## 5. Окна (все создаются программно, #[tauri::Builder] без статических окон)

| Window label | Назначение | Ключевые флаги |
| --- | --- | --- |
| `island-{monitor_id}` (N шт.) | Остров на мониторе | transparent, decorations:false, always_on_top, skip_taskbar, visible_on_all_workspaces(false), focused(false), shadow:false; далее WS_EX_NOACTIVATE + динамический WS_EX_TRANSPARENT/LAYERED hit-test |
| `launcher` | Лончер | как остров + фокус по вызову, ленивое создание |
| `capture` | Селектор области | полноэкранный per-monitor, прозрачный, topmost |
| `settings` | Настройки (M11) | обычное окно с декорациями |

Клик-_through_: Rust подписывается на глобальное движение курсора (WM_MOUSEMOVE через скрытый message-only window либо GetCursorPos таймер ≥120 Гц только во время активности — см. ADR-003), делает hit-test по форме острова и вызывает `SetWindowLongPtrW(GWL_EXSTYLE)` с флагами. В idle — полностью click-through.

Полноэкранный режим: `SHQueryUserNotificationState` опрашивается event-триггерно (change of foreground hook запрещён → polling 1 Гц только когда остров видим, согласно ADR-003) → событие `fullscreen:changed` → скрытие/возврат.

## 6. IPC-контракт (единый источник истины)

Команды и события — ровно по таблице ТЗ §5.2/§5.3 (+дополнения v1.1). Реестр поддерживается в `docs/ipc.md` и дублируется типами `src/ipc/dto.rs` (Rust) → `app/src/types.ts` (генерация ts-rs в pre-build).

События: cursor:near, fullscreen:changed, monitor:changed, launcher:visibility, ai:token|done|error, capture:result, shelf:changed, clipboard:new, activity:update, stt:state, media:update, weather:update, settings:changed.

## 7. Схема БД (SQLite, файл `%LOCALAPPDATA%\Island\island.db`, WAL)

Миграции `NNN_name.sql`, таблица `schema_migrations`. Состав — ТЗ §5.4: clipboard_history, settings, shelf_files, ai_sessions, ai_messages, ai_documents, ai_cache, app_index, weather_cache (+ai_chunks Could позже).
Шифрование: `content_enc`/PNG — AES-256-GCM; DEK хранится в Credential Manager (или DPAPI-обёртка). В экспорт настроек секреты не попадают.

## 8. Конфигурация и настройки

`settings` (SQLite, key/value JSON) + defaults в коде (`modules/settings/defaults.rs`), схема версионирована (`settings.schema_version`). Любое изменение → событие `settings:changed{key,value}` → затронутые модули реагируют без рестарта. Экспорт/импорт — JSON-файл без секретов.

## 9. Жизненный цикл и надёжность

- Single instance (plugin) — второй запуск открывает существующий.
- Tray: Показать/Скрыть острова, Пауза хоткеев, Пауза буфера, Настройки, Выход.
- Shutdown: обработка RunEvent::ExitRequested → остановка потоков (clipboard listener, hotkeys, notify), закрытие DB.
- Watchdog: родительский процесс-serve; при падении islands.exe OS снимает topmost-окна (Tauri-окна принадлежат процессу) — «осиротевших» окон не остаётся; лог panic-hook (tauri-plugin-log, ротация).
- Startup ≤2 с: предпрогрев WebView2 (скрытое окно при старте), лёгкий idle-бандл.

## 10. Безопасность (чек-лист реализации)

1. API-ключ Gemini: только Credential Manager; команды `secret_set/secret_exists` (значение не читается UI); запросы только из Rust.
2. Clipboard: фильтры форматов/процессов/эвристик секретов до записи; содержимое не логируется; AES-256-GCM.
3. Capability ACL: UI-разрешения минимальны (core:default, event, invoke custom commands); shell/fs/http — запрещены фронтенду.
4. Никаких инъекций/глобальных хуков; только документированные API.
5. Телеметрия отсутствует полностью (не opt-in, а none).

## 11. CI/CD

- `ci.yml`: push/PR → jobs: `rust-check-test` (ubuntu, native deps bundled sqlite; windows-cfg код компилируется cross-check'ом `cargo check --target x86_64-pc-windows-msvc` на windows-latest), `web-lint-typebuild` (npm ci, tsc, eslint, vitest), `fmt+clippy(-D warnings)`.
- `release.yml`: tag v* → windows-latest: `npm ci && cargo tauri build` → артефакты NSIS + portable ZIP → GitHub Releases (draft→publish).
- Линтер ассетов жильцов (S11): проверка LICENSES.md ↔ файлы.

## 12. Тестовая стратегия

| Уровень | Что | Инструмент |
| --- | --- | --- |
| Unit Rust | фильтрация буфера, эвристики секретов, fuzzy-поиск, calc/convert парсер, dto-сериализация, миграции | cargo test (ядро non-Windows) |
| Unit TS | store, компоненты панелей, форматтеры i18n | vitest + testing-library |
| Contract | IPC-dto Rust↔TS | ts-rs generation + typecheck |
| Integration Win | smoke на windows-latest: старт headless-подобный (создать/скрыть окна), DB, keyring stub | cargo test --features win-smoke |
| Rучная приёмка | матрица 8.1/8.2 ТЗ | чек-листы docs/sprints |

## 13. План спринтов

Ровно по ТЗ §6 (S1–S5 MVP, S6–S12 полная версия). Приоритеты внутри спринтов фиксирует план спринта в `docs/sprints/SN-plan.md`, отчёт — `SN-report.md`. Недельная нагрузка владельца — открытый вопрос (ТУ в ТЗ §6): план строится по объёму задач, сроки пересчитываются после первого реального спринта.

## 14. Риски архитектуры (топ-3) и снижение

1. Мультимонитор + DPI/перерисовка → все размеры в physical px, конвертация на границе, тесты на VM 2×DPI (матрица S2).
2. Click-through hit-test и «съеденные» клики → conservative region (фигураidle/hover), автотест regions, fallback-настройка «всегда интерактивно».
3. WebView2 RAM на N островов → общий user-data folder, один frontend-бандл на все окна (same URL, state per monitor id через launch args), ленивые панели.
