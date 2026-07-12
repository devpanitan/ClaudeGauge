// Hide the console window in release; keep it in debug so `println!` of the
// UsageState (M1 deliverable) is visible while developing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod usage;

use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, WindowEvent,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // `claudegauge config set <key> <value>` writes config and exits without
    // launching the GUI. A running widget re-reads config on its next poll.
    if handle_config_cli(&args) {
        return;
    }

    // First-instance CLI arg: start (default) | stop | toggle.
    let subcommand = args.get(1).cloned().unwrap_or_default();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_limit, get_theme, set_theme])
        // Route `claudegauge <cmd>` from a second launch to the running one.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let cmd = argv.get(1).map(String::as_str).unwrap_or("start");
            control(app, cmd);
        }))
        .setup(move |app| {
            let handle = app.handle().clone();
            let win = app
                .get_webview_window("main")
                .expect("main window missing");

            // Restore last saved position before showing.
            let cfg = config::load();
            let _ = win.set_position(tauri::PhysicalPosition::new(
                cfg.position.x,
                cfg.position.y,
            ));

            // Honor the initial subcommand. If this process is the first
            // instance, `stop` has nothing to stop, so just exit; anything
            // else shows the widget.
            if subcommand == "stop" {
                app.handle().exit(0);
            } else {
                let _ = win.show();
            }

            // Persist position whenever the user drags the widget.
            win.on_window_event(|event| {
                if let WindowEvent::Moved(pos) = event {
                    config::save_position(pos.x, pos.y);
                }
            });

            build_tray(app)?;

            // Poll ccusage on a background thread; emit + print each tick.
            std::thread::spawn(move || loop {
                let cfg = config::load();
                let state = usage::fetch(&cfg);
                // Debug-only console trace; release builds stay silent (no
                // usage data written to stdout).
                #[cfg(debug_assertions)]
                if let Ok(json) = serde_json::to_string(&state) {
                    println!("{json}");
                }
                let _ = handle.emit("usage", &state);
                let secs = cfg.poll_interval.max(5);
                std::thread::sleep(Duration::from_secs(secs));
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ClaudeGauge");
}

/// Apply a control command to the running instance.
fn control(app: &tauri::AppHandle, cmd: &str) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    match cmd {
        "stop" | "quit" => app.exit(0),
        "start" | "show" => {
            let _ = win.show();
            let _ = win.set_focus();
        }
        "hide" => {
            let _ = win.hide();
        }
        "toggle" => {
            if win.is_visible().unwrap_or(false) {
                let _ = win.hide();
            } else {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
        _ => {
            let _ = win.show();
        }
    }
}

/// Tauri command: set/clear the token limit from the widget input.
/// Pass `null`/`0` to clear (percentage hidden).
#[tauri::command]
fn set_limit(limit: Option<u64>) {
    config::set_limit(limit);
}

/// Tauri command: read the saved theme so the widget can apply it on load.
#[tauri::command]
fn get_theme() -> String {
    config::load().theme
}

/// Tauri command: persist the selected theme (dark | light | glass).
#[tauri::command]
fn set_theme(theme: String) {
    config::set_theme(&theme);
}

/// Handle `config set <key> <value>` CLI form. Returns true if this was a
/// config command (caller should exit). Non-config args return false.
fn handle_config_cli(args: &[String]) -> bool {
    if args.get(1).map(String::as_str) != Some("config") {
        return false;
    }
    // config set <key> <value>
    let key = args.get(3).map(String::as_str).unwrap_or("");
    let raw = args.get(4).map(String::as_str).unwrap_or("");
    let is_set = args.get(2).map(String::as_str) == Some("set");

    match (is_set, key) {
        (true, "limit") => match parse_human_number(raw) {
            Some(n) if n > 0 => {
                config::set_limit(Some(n));
                println!("limit set to {n}");
            }
            _ => {
                config::set_limit(None);
                println!("limit cleared (invalid or non-positive value)");
            }
        },
        (true, "poll") => {
            if let Ok(secs) = raw.parse::<u64>() {
                let mut cfg = config::load();
                cfg.poll_interval = secs.max(5);
                let _ = config::save(&cfg);
                println!("poll interval set to {}s", cfg.poll_interval);
            } else {
                println!("invalid poll value");
            }
        }
        _ => {
            println!("usage: claudegauge config set <limit|poll> <value>");
        }
    }
    true
}

/// Parse a human-friendly number: "2000000", "2,000,000", "2m", "500k".
fn parse_human_number(s: &str) -> Option<u64> {
    let t = s.trim().replace([',', '_'], "").to_lowercase();
    if t.is_empty() {
        return None;
    }
    let (num, mult) = if let Some(stripped) = t.strip_suffix('m') {
        (stripped, 1_000_000.0)
    } else if let Some(stripped) = t.strip_suffix('k') {
        (stripped, 1_000.0)
    } else {
        (t.as_str(), 1.0)
    };
    num.parse::<f64>()
        .ok()
        .map(|v| (v * mult).round() as u64)
        .filter(|v| *v > 0)
}

/// System tray with show/hide + quit.
fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("ClaudeGauge")
        .menu(&menu)
        .on_menu_event(|app, event| control(app, event.id.as_ref()))
        .build(app)?;

    Ok(())
}
