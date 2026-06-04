use crate::state::{aggregate_state, AppState, SessionState};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

/// Setup the system tray icon and context menu.
pub fn setup_tray(
    app: &tauri::App,
    shared_state: Arc<RwLock<AppState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let default_icon = crate::icons::icon_for_state(&SessionState::Idle)?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("Greenlight: IDLE")
        .icon(default_icon)
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("popup") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("popup") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    // Update tray icon when state changes
    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        let mut prev_state = String::new();
        loop {
            let app_state = shared_state.read().await;
            let agg = aggregate_state(&app_state.sessions);
            let count = app_state.sessions.len();
            let new_state = format!("{:?}{}", agg, count);
            drop(app_state);

            if new_state != prev_state {
                prev_state = new_state.clone();
                let state_str = crate::state::state_to_string(&agg);
                let tooltip = format!("Greenlight: {} ({} session{})", state_str, count, if count == 1 { "" } else { "s" });

                if let Some(tray) = app_handle.tray_by_id("main-tray") {
                    match crate::icons::icon_for_state(&agg) {
                        Ok(icon) => {
                            let _ = tray.set_icon(Some(icon));
                        }
                        Err(e) => log::error!("Failed to load icon for state {agg:?}: {e}"),
                    }
                    let _ = tray.set_tooltip(Some(&tooltip));
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    Ok(())
}