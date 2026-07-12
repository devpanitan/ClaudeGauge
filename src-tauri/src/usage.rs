//! Data engine: wrap `ccusage blocks --json`, map into `UsageState`.
//!
//! We never parse Claude Code's JSONL ourselves — ccusage owns that. This
//! module only shells out, deserializes, and computes the display state.
//!
//! M2.5 design: `usedTokens` is the exact source of truth. We do NOT guess the
//! 5-hour cap anymore (auto-derivation produced nonsense like ~24.8M). The cap
//! is whatever the user configured; `percent` is only computed when a `limit`
//! is set, and is `null` otherwise.

use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::process::Command;

// ---- ccusage JSON (only the fields we need; everything else ignored) ----

#[derive(Deserialize, Default)]
struct CcOut {
    #[serde(default)]
    blocks: Vec<Block>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Block {
    #[serde(default)]
    is_active: bool,
    #[serde(default)]
    is_gap: bool,
    #[serde(default)]
    end_time: Option<String>,
    #[serde(default)]
    total_tokens: u64,
    #[serde(default)]
    token_counts: Option<TokenCounts>,
    #[serde(default)]
    burn_rate: Option<BurnRate>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TokenCounts {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct BurnRate {
    #[serde(default)]
    tokens_per_minute: f64,
}

impl Block {
    /// Tokens used in this block. Prefer ccusage's `totalTokens`; if absent
    /// (older/edge output), sum the component counts.
    fn used(&self) -> u64 {
        if self.total_tokens > 0 {
            return self.total_tokens;
        }
        match &self.token_counts {
            Some(tc) => {
                tc.input_tokens
                    + tc.output_tokens
                    + tc.cache_creation_input_tokens
                    + tc.cache_read_input_tokens
            }
            None => 0,
        }
    }

    fn burn(&self) -> f64 {
        self.burn_rate
            .as_ref()
            .map(|b| b.tokens_per_minute)
            .unwrap_or(0.0)
    }
}

// ---- Public display state emitted to the frontend ----

#[derive(Serialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UsageState {
    /// Exact token count from ccusage — always trustworthy.
    pub used_tokens: u64,
    /// User-configured cap; `null` when unset.
    pub limit: Option<u64>,
    /// `usedTokens / limit * 100`, or `null` when no limit is set.
    pub percent: Option<f64>,
    /// Tokens per minute for the active block.
    pub burn_rate: f64,
    pub reset_in_seconds: i64,
    pub is_active: bool,
    /// Populated only when ccusage could not be read.
    pub error: Option<String>,
}

/// Fetch usage once. Never panics; failures surface via `error`.
pub fn fetch(cfg: &Config) -> UsageState {
    match run_ccusage() {
        Ok(raw) => match serde_json::from_str::<CcOut>(&raw) {
            Ok(parsed) => map_state(&parsed, cfg, now_rfc3339()),
            Err(e) => UsageState {
                limit: cfg.limit,
                error: Some(format!("parse error: {e}")),
                ..Default::default()
            },
        },
        Err(e) => UsageState {
            limit: cfg.limit,
            error: Some(e),
            ..Default::default()
        },
    }
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Pure mapping from parsed ccusage output to display state.
/// `now` is injected (RFC3339) so this is unit-testable.
fn map_state(out: &CcOut, cfg: &Config, now: String) -> UsageState {
    let active = out.blocks.iter().find(|b| b.is_active && !b.is_gap);
    let limit = cfg.limit.filter(|l| *l > 0);

    match active {
        None => UsageState {
            used_tokens: 0,
            limit,
            percent: None,
            burn_rate: 0.0,
            reset_in_seconds: 0,
            is_active: false,
            error: None,
        },
        Some(b) => {
            let used = b.used();
            // Percent only when a valid limit exists — never guessed.
            let percent = limit.map(|l| (used as f64 / l as f64) * 100.0);
            let reset_in = match &b.end_time {
                Some(end) => seconds_until(end, &now),
                None => 0,
            };
            UsageState {
                used_tokens: used,
                limit,
                percent,
                burn_rate: b.burn(),
                reset_in_seconds: reset_in,
                is_active: true,
                error: None,
            }
        }
    }
}

/// Seconds from `now` until `end` (both RFC3339). Clamped at 0.
fn seconds_until(end: &str, now: &str) -> i64 {
    let end = chrono::DateTime::parse_from_rfc3339(end);
    let now = chrono::DateTime::parse_from_rfc3339(now);
    match (end, now) {
        (Ok(e), Ok(n)) => (e.timestamp() - n.timestamp()).max(0),
        _ => 0,
    }
}

/// Run `ccusage blocks --json`, cross-platform. Returns stdout or an error.
fn run_ccusage() -> Result<String, String> {
    let output = build_command()
        .output()
        .map_err(|e| format!("spawn failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let head = stderr.lines().next().unwrap_or("ccusage failed");
        return Err(head.to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(target_os = "windows")]
fn build_command() -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // npx resolves to npx.cmd on Windows, so go through cmd.
    let mut c = Command::new("cmd");
    c.args(["/C", "npx", "ccusage@latest", "blocks", "--json"]);
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

#[cfg(not(target_os = "windows"))]
fn build_command() -> Command {
    let mut c = Command::new("npx");
    c.args(["ccusage@latest", "blocks", "--json"]);
    c
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> CcOut {
        serde_json::from_str(raw).unwrap()
    }

    const SAMPLE: &str = r#"{
      "blocks": [
        {
          "isActive": false, "isGap": false,
          "endTime": "2026-07-12T05:00:00.000Z",
          "totalTokens": 180000
        },
        {
          "isActive": true, "isGap": false,
          "endTime": "2026-07-12T15:00:00.000Z",
          "tokenCounts": {
            "inputTokens": 40000, "outputTokens": 5000,
            "cacheCreationInputTokens": 4000, "cacheReadInputTokens": 1000
          },
          "burnRate": { "tokensPerMinute": 142.5 }
        }
      ]
    }"#;

    #[test]
    fn no_limit_means_null_percent_but_exact_used() {
        let cfg = Config::default(); // limit = None
        let s = map_state(&parse(SAMPLE), &cfg, "2026-07-12T11:00:00.000Z".into());

        assert!(s.is_active);
        assert_eq!(s.used_tokens, 50000); // summed from tokenCounts, exact
        assert_eq!(s.limit, None);
        assert_eq!(s.percent, None); // never guessed
        assert!((s.burn_rate - 142.5).abs() < 0.001);
        assert_eq!(s.reset_in_seconds, 4 * 3600); // 11:00 -> 15:00
        assert!(s.error.is_none());
    }

    #[test]
    fn limit_set_produces_percent() {
        let mut cfg = Config::default();
        cfg.limit = Some(100000);
        let s = map_state(&parse(SAMPLE), &cfg, "2026-07-12T14:59:00.000Z".into());
        assert_eq!(s.limit, Some(100000));
        assert!((s.percent.unwrap() - 50.0).abs() < 0.001);
        assert_eq!(s.reset_in_seconds, 60);
    }

    #[test]
    fn zero_limit_is_treated_as_unset() {
        let mut cfg = Config::default();
        cfg.limit = Some(0);
        let s = map_state(&parse(SAMPLE), &cfg, "2026-07-12T11:00:00.000Z".into());
        assert_eq!(s.limit, None);
        assert_eq!(s.percent, None);
    }

    #[test]
    fn no_active_block_reports_idle() {
        let raw = r#"{ "blocks": [
          { "isActive": false, "isGap": false, "totalTokens": 120000,
            "endTime": "2026-07-12T05:00:00.000Z" }
        ] }"#;
        let mut cfg = Config::default();
        cfg.limit = Some(200000);
        let s = map_state(&parse(raw), &cfg, now_rfc3339());
        assert!(!s.is_active);
        assert_eq!(s.used_tokens, 0);
        assert_eq!(s.percent, None); // idle -> no percent
        assert_eq!(s.limit, Some(200000)); // limit still surfaced
    }

    #[test]
    fn countdown_never_negative() {
        assert_eq!(
            seconds_until("2026-07-12T05:00:00.000Z", "2026-07-12T09:00:00.000Z"),
            0
        );
    }
}
