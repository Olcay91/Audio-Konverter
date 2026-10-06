//! Symbol im Infobereich (Tray). Existiert nur, solange „In den Infobereich
//! minimieren“ eingeschaltet ist. Beim Minimieren verschwindet das Fenster dann
//! aus der Taskleiste; laufende Konvertierungen gehen im Hintergrund weiter.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Deserialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};

const TRAY_ID: &str = "main";
const MAIN_WINDOW: &str = "main";

/// Ob beim Minimieren ins Tray versteckt wird. Wird von der Oberfläche gesetzt.
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Texte des Tray-Menüs in der Sprache der Oberfläche.
#[derive(Deserialize)]
pub struct TrayLabels {
    show: String,
    quit: String,
    tooltip: String,
}

/// Schaltet das Verhalten ein oder aus. Bei jedem Aufruf wird das Symbol neu
/// angelegt, damit ein Sprachwechsel auch das Menü übersetzt.
#[tauri::command]
pub fn set_minimize_to_tray(app: AppHandle, enabled: bool, labels: TrayLabels) -> Result<(), String> {
    ENABLED.store(enabled, Ordering::Relaxed);
    let _ = app.remove_tray_by_id(TRAY_ID);
    if !enabled {
        return Ok(());
    }
    create(&app, &labels).map_err(|e| e.to_string())
}

fn create(app: &AppHandle, labels: &TrayLabels) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", &labels.show, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", &labels.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(&labels.tooltip)
        .menu(&menu)
        // Linksklick holt das Fenster zurück, das Menü gibt es per Rechtsklick.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Holt das Hauptfenster zurück in den Vordergrund.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Minimieren erkennen: Tauri meldet es als Größenänderung, danach ist das
/// Fenster minimiert. Dann ganz verstecken, damit es nur noch im Tray steht.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if let WindowEvent::Resized(_) = event {
        if window.is_minimized().unwrap_or(false) {
            let _ = window.hide();
        }
    }
}
