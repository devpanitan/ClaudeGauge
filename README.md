# ClaudeGauge

A tiny always-on-top desktop widget for **Windows** that shows your **Claude
Code** token usage for the current 5-hour window — how much you've used, how
much is left, and when it resets.

> ⚠️ **Not an official Anthropic tool.** All data stays on your machine.
> ClaudeGauge only reads local usage via [`ccusage`](https://github.com/ryoppippi/ccusage);
> it never sends anything off your computer and never touches your credentials.

> 📸 **Screenshot / GIF goes here.** Drop a capture of the widget (ideally the
> three themes, or a short GIF of it live) at `docs/screenshot.png` and replace
> this line with:
> `![ClaudeGauge widget](docs/screenshot.png)`

## What it shows

- **tokens used** for the current 5-hour window — always exact
- **% + a colored progress bar** (green → yellow 60% → red 85%) once you set a
  token limit; the burn rate shows instead until then
- **reset countdown** until the window rolls over
- transparent, borderless, always-on-top; drag it anywhere and it remembers
  where you put it; **Dark / Light / Glass** themes

## How it works

```
Claude Code (~/.claude/*.jsonl)
        │
   ccusage blocks --json      ◄── polled every 30s (configurable)
        │
   Rust backend (Tauri)  →  UsageState { usedTokens, cap, percent, resetIn }
        │
   Transparent always-on-top widget  +  system tray  +  CLI
```

ClaudeGauge does **not** parse Claude Code's JSONL itself — `ccusage` owns
that. The backend just polls it, maps the JSON into a display state, and emits
it to the widget.

### A note on the percentage

Anthropic tunes the real 5-hour token limit server-side, so it isn't fixed —
and guessing it produced misleading numbers. So ClaudeGauge **does not guess**.
The **exact token count** is always the star of the card. The percentage and
progress bar only appear once *you* set a `limit`:

- **no limit set** → shows tokens used + a button to set one; no `%`
- **limit set** → shows `%`, a colored progress bar, and `used / limit`

Set your limit whenever you like (see CLI below, or click the button on the
widget). It persists across restarts.

## Install — just want to use it (most people)

No cloning, no Rust. Just:

1. Go to the [**Releases**](https://github.com/devpanitan/ClaudeGauge/releases)
   page and download `ClaudeGauge_x.x.x_x64_en-US.msi`.
2. Double-click the `.msi` to install.
3. Launch **ClaudeGauge** — the widget appears. Drag it wherever you like.

**Before it can show anything, you need:**

- **Windows 10 / 11**
- **[Node.js](https://nodejs.org/)** installed and on your PATH — the widget
  runs `ccusage` through `npx`, which needs Node.
- **You've used Claude Code before** on this machine, so there are local usage
  logs in `%USERPROFILE%\.claude\` to read. No logs = nothing to show.

> First launch shows tokens used but no `%` — that's expected. Set a token
> limit to unlock the percentage and progress bar (see below).

### Set your token limit

Anthropic doesn't publish a fixed 5-hour cap, so you set your own to get a `%`.
Either:

- **On the widget:** click the ⚙ gear (or the "ตั้ง token limit" button) and
  type a number — `2000000`, `2m`, and `500k` all work.
- **From a terminal:**

  ```bash
  ccgauge config set limit 2000000
  ```

It's saved and persists across restarts.

## Install — you want to change the code (developers)

```bash
git clone https://github.com/devpanitan/ClaudeGauge.git
cd ClaudeGauge

npm install

# one-time: generate icon binaries from the source logo
npm run tauri icon src-tauri/icons/icon.svg

# run in dev (hot-reload widget; prints UsageState JSON to the console)
npm run tauri dev

# produce the installer yourself (.msi in src-tauri/target/release/bundle/msi/)
npm run tauri build
```

Building from source also needs the
[Rust toolchain](https://www.rust-lang.org/tools/install) and the
[Tauri v2 prerequisites](https://tauri.app/start/prerequisites/) (WebView2 is
preinstalled on Windows 11).

## CLI

The installed executable is also the CLI. Copy `scripts/ccgauge.cmd` somewhere
on your PATH (edit the exe path inside if needed), then:

```bash
ccgauge start                     # show the widget
ccgauge stop                      # quit the running widget
ccgauge toggle                    # show/hide
ccgauge hide                      # hide (tray stays)
ccgauge config set limit 2000000  # set the 5h cap -> % + bar appear
ccgauge config set limit 2m       # same, shorthand (k / m accepted)
ccgauge config set poll 60        # poll every 60s
```

Setting a limit writes to the config file; a running widget picks it up on its
next poll. You can also click **"ตั้ง token limit เพื่อดู %"** on the widget
and type the number inline.

A second launch is routed to the already-running instance, so `stop`/`toggle`
act on the live widget rather than opening a new one.

## Config

`%APPDATA%\ClaudeGauge\config.json` (created on first run). See
[`config.example.json`](config.example.json) for a documented template:

```json
{
  "pollInterval": 30,
  "mode": "full",
  "theme": "dark",
  "position": { "x": 1600, "y": 40 },
  "limit": null,
  "thresholds": { "warn": 60, "danger": 85 }
}
```

- `pollInterval` — seconds between ccusage polls (minimum 5)
- `theme` — `dark`, `light`, or `glass`; also cycled by the palette icon on the widget
- `limit` — the 5h token cap; a positive number shows `%` + bar, `null` hides them
- `thresholds` — percentages at which the bar turns yellow / red
- `position` — updated automatically when you drag the widget

## Status

MVP (milestones M1–M2): data pipeline, always-on-top widget, drag + remembered
position, colored progress bar, tray, and CLI start/stop/toggle. Display modes,
threshold notifications, and a winget manifest are planned next (M3–M4).

## Privacy

Reads local usage logs only. No API keys, no external services (besides
`ccusage` running locally), no web-app scraping.

## License

[MIT](LICENSE)
