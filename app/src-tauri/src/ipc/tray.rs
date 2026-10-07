// ipc/tray.rs — иконка в трее (ТЗ §3 «Трей»): Показать/Скрыть, Пауза хоткеев, Настройки, Выход.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Emitter, Manager};

pub fn build_tray(app: &mut App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Показать/Скрыть острова", true, None::<&str>)?;
    let pause_hotkeys = MenuItem::with_id(app, "pause_hotkeys", "Пауза хоткеев", true, None::<&str>)?;
    let pause_clipboard =
        MenuItem::with_id(app, "pause_clipboard", "Пауза записи буфера", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Настройки", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &pause_hotkeys, &pause_clipboard, &settings, &quit])?;

    let icon = app.default_window_icon().cloned().ok_or_else(|| {
        std::io::Error::other("missing default window icon (icons/icon.png required)")
    })?;

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Island")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                for w in app.webview_windows().values() {
                    if w.label().starts_with("island") {
                        let _ = if w.is_visible().unwrap_or(false) { w.hide() } else { w.show() };
                    }
                }
            }
            "pause_hotkeys" => {
                let st = app.state::<crate::AppState>();
                let paused = st.settings.get("general.tray_paused").as_bool() != Some(true);
                if st.settings.set("general.tray_paused", serde_json::json!(paused)) {
                    st.bus.settings_changed("general.tray_paused", serde_json::json!(paused));
                }
            }
            "pause_clipboard" => {
                let st = app.state::<crate::AppState>();
                let paused = st.settings.get("clipboard.paused").as_bool() != Some(true);
                if st.settings.set("clipboard.paused", serde_json::json!(paused)) {
                    st.bus.settings_changed("clipboard.paused", serde_json::json!(paused));
                }
            }
            "settings" => {
                // окно настроек создаётся лениво (S5); пока — событие для warm-окна
                let _ = app.emit("settings:open", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}
