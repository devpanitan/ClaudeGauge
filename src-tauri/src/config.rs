//! Config file handling: %APPDATA%\ClaudeGauge\config.json
//!
//! The file is created with defaults on first run. `limit` is optional:
//! when null, no percentage is shown (the cap is never guessed); set a
//! positive number to display `%` and the progress bar.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Thresholds {
    pub warn: f64,
    pub danger: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_poll")]
    pub poll_interval: u64,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_position")]
    pub position: Position,
    /// None => no percentage shown; Some(n) => show `%` against this cap.
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default = "default_thresholds")]
    pub thresholds: Thresholds,
}

fn default_poll() -> u64 {
    30
}
fn default_mode() -> String {
    "full".to_string()
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_position() -> Position {
    Position { x: 1600, y: 40 }
}
fn default_thresholds() -> Thresholds {
    Thresholds {
        warn: 60.0,
        danger: 85.0,
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            poll_interval: default_poll(),
            mode: default_mode(),
            theme: default_theme(),
            position: default_position(),
            limit: None,
            thresholds: default_thresholds(),
        }
    }
}

/// Directory that holds config.json (created if missing).
pub fn config_dir() -> PathBuf {
    // Windows: %APPDATA%\ClaudeGauge
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("ClaudeGauge");
    }
    // Fallback for dev on other platforms.
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config").join("ClaudeGauge");
    }
    PathBuf::from(".").join("ClaudeGauge")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Load config, writing a default file the first time.
pub fn load() -> Config {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str::<Config>(&text).unwrap_or_else(|_| {
            let cfg = Config::default();
            let _ = save(&cfg);
            cfg
        }),
        Err(_) => {
            let cfg = Config::default();
            let _ = save(&cfg);
            cfg
        }
    }
}

pub fn save(cfg: &Config) -> std::io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let text = serde_json::to_string_pretty(cfg).unwrap_or_default();
    fs::write(config_path(), text)
}

/// Persist a new window position without disturbing other fields.
pub fn save_position(x: i32, y: i32) {
    let mut cfg = load();
    cfg.position = Position { x, y };
    let _ = save(&cfg);
}

/// Persist the selected theme (validated against known names).
pub fn set_theme(theme: &str) {
    let valid = matches!(theme, "dark" | "light" | "glass");
    let mut cfg = load();
    cfg.theme = if valid { theme.to_string() } else { default_theme() };
    let _ = save(&cfg);
}

/// Set (or clear) the user token limit, keeping other fields intact.
/// Only strictly-positive values are accepted; anything else clears the
/// limit back to `null` (percentage will not be shown).
pub fn set_limit(value: Option<u64>) {
    let mut cfg = load();
    cfg.limit = match value {
        Some(v) if v > 0 => Some(v),
        _ => None,
    };
    let _ = save(&cfg);
}
