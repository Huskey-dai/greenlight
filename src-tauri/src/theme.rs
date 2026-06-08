/// Detect the OS theme (dark/light) and emit changes to the frontend.
use tauri::Emitter;
use tauri::Listener;

/// Start listening for OS theme changes and emit them to the frontend.
pub fn start_theme_listener(app: &tauri::AppHandle) {
    let app_handle = app.clone();
    // Tauri's WebviewWindow emits ThemeChanged events
    // We listen for these and forward them via our custom event
    let _ = app.listen("tauri://theme-changed", move |event| {
        let _ = app_handle.emit("theme-changed", event.payload());
    });
}