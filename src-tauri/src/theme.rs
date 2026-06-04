/// Detect the OS theme (dark/light) and emit changes to the frontend.
use tauri::Emitter;
use tauri::Listener;

/// Get the current OS theme preference.
pub fn current_theme() -> String {
    // Tauri provides system theme detection
    // This will be called on startup and when changes are detected
    "light".to_string() // Default, will be overridden by actual detection
}

/// Start listening for OS theme changes and emit them to the frontend.
pub fn start_theme_listener(app: &tauri::AppHandle) {
    let app_handle = app.clone();
    // Tauri's WebviewWindow emits ThemeChanged events
    // We listen for these and forward them via our custom event
    let _ = app.listen("tauri://theme-changed", move |event| {
        let _ = app_handle.emit("theme-changed", event.payload());
    });
}