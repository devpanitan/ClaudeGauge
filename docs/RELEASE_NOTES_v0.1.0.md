# ClaudeGauge v0.1.0

The first release — a tiny always-on-top desktop widget for Windows that shows
your Claude Code token usage for the current 5-hour window.

## What it does

Keeps an at-a-glance widget floating on your desktop so you can see how many
tokens you've used in the active 5-hour window, how fast you're burning them,
and when the window resets — without breaking flow.

## Highlights

- **Exact token count** for the current 5-hour block (read locally via `ccusage`)
- **Optional %, progress bar, and `used / limit`** once you set a token limit —
  green → yellow (60%) → red (85%)
- **Burn rate + reset countdown** in the footer
- **Always-on-top, transparent, frameless**; drag it anywhere and it remembers
  where you put it
- **Three themes** — Dark / Light / Glass (palette icon cycles them)
- **System tray** + **CLI**: `ccgauge start | stop | toggle`, and
  `ccgauge config set limit 2m`
- Set the limit right from the widget via the ⚙ gear

## Requirements

- Windows 10 / 11
- Node.js on your PATH (the widget calls `ccusage` through `npx`)
- Prior Claude Code usage on this machine (so there are logs in
  `%USERPROFILE%\.claude\` to read)

## Install

Download `ClaudeGauge_0.1.0_x64_en-US.msi` from the assets below, double-click
to install, and launch ClaudeGauge.

## Known limitations

- **You set the token cap yourself.** Anthropic tunes the real 5-hour limit
  server-side, so ClaudeGauge doesn't guess it. Until you set a `limit`, the
  widget shows the exact token count and burn rate but no `%`.
- **Node.js required** — v0.1.0 shells out to `ccusage` via `npx`. Bundling
  ccusage to drop the Node dependency is planned for a later version.
- Windows only for now.

---

Not an official Anthropic tool. Reads local usage only — nothing leaves your
machine, and it never touches your credentials.
