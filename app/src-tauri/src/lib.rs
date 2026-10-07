// Island — entry point приложения (Tauri 2).
// S1: каркас — core (db/settings/secrets/bus), ipc-команды настроек/секретов, трей, single-instance.
// Модули M1–M9 подключаются по мере реализации (см. docs/ipc.md «Статус»).

mod core;
mod ipc;

#[cfg(target_os = "windows")]
mod windows_platform;

use std::sync::Arc;

use tauri::{Manager, RunEvent, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::core::bus::{EventBus, TauriEmitter};
use crate::core::db::Db;
use crate::core::settings::SettingsStore;

/// Общее состояние приложения для команд.
pub struct AppState {
    pub db: Arc<Db>,
    pub settings: Arc<SettingsStore>,
    pub bus: Arc<EventBus>,
}

fn app_data_dir(app: &tauri::AppHandle) -> std::path::PathBuf {
    // %LOCALAPPDATA%\Island на Windows; fallback — стандартный dir из Tauri.
    let base = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("./island-data"));
    if std::env::var_os("ISLAND_DATA_DIR").is_some() {
        // тестовый/переносной режим
        return std::path::PathBuf::from(std::env::var_os("ISLAND_DATA_DIR").unwrap());
    }
    base
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // второй экземпляр — показать острова (реализация M1 в S2)
            log::info!("second instance ignored");
            let _ = app;
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().max_file_size(5_000_000).build())
        .setup(|app| {
            let handle = app.handle().clone();
            let dir = app_data_dir(&handle);
            let db = Arc::new(Db::open(&dir.join("island.db"))?);
            let settings = Arc::new(SettingsStore::new(db.clone()));
            let bus = Arc::new(EventBus(Arc::new(TauriEmitter(handle.clone()))));
            app.manage(AppState {
                db,
                settings: settings.clone(),
                bus: bus.clone(),
            });

            // Язык из настроек применяется в UI при загрузке (i18n читает settings_get).

            // Остров-заглушка каркаса: невидимое окно предпрогрева WebView2 + трей.
            // Реальные per-monitor острова — модуль overlay (S2).
            let _warm = WebviewWindowBuilder::new(&handle, "island-warm", tauri::WebviewUrl::App("index.html".into()))
                .visible(false)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .inner_size(320.0, 40.0)
                .build()?;

            ipc::tray::build_tray(app)?;

            // Событие старта настроек → подписка модулей будет в S2+.
            log::info!(
                "island setup complete, data_dir={}",
                dir.display()
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::settings::settings_get,
            ipc::settings::settings_set,
            ipc::settings::settings_all,
            ipc::settings::settings_reset,
            ipc::secrets_cmd::secret_set,
            ipc::secrets_cmd::secret_exists,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // остров/лончер не закрывают приложение — только скрытие (выход через трей)
                let label = window.label();
                if label.starts_with("island") || label == "launcher" || label == "capture" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("fatal: failed to build Island app")
        .run(|app_handle, event| match event {
            RunEvent::ExitRequested { .. } => {
                log::info!("exit requested: flushing state");
                let _ = app_handle;
            }
            _ => {}
        });
}
