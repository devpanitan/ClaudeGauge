import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

// Shape emitted by the Rust backend (see src-tauri/src/usage.rs).
interface UsageState {
  usedTokens: number;
  limit: number | null;
  percent: number | null;
  burnRate: number;
  resetInSeconds: number;
  isActive: boolean;
  error: string | null;
}

const el = {
  widget: document.getElementById("widget") as HTMLDivElement,
  used: document.getElementById("used") as HTMLSpanElement,
  percent: document.getElementById("percent") as HTMLSpanElement,
  bar: document.getElementById("bar") as HTMLDivElement,
  footLeft: document.getElementById("foot-left") as HTMLSpanElement,
  footRight: document.getElementById("foot-right") as HTMLSpanElement,
  footRightTxt: document.getElementById("foot-right-txt") as HTMLSpanElement,
  cta: document.getElementById("cta") as HTMLButtonElement,
  gear: document.getElementById("gear") as HTMLButtonElement,
  palette: document.getElementById("palette") as HTMLButtonElement,
  input: document.getElementById("limit-input") as HTMLInputElement,
  saveBtn: document.getElementById("save-btn") as HTMLButtonElement,
};

const WARN = 60;
const DANGER = 85;

// 27900 -> "27.9k", 2000000 -> "2.0M", 940 -> "940"
function fmtTokens(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + "M";
  if (n >= 1_000) return (n / 1_000).toFixed(n >= 100_000 ? 0 : 1) + "k";
  return String(Math.round(n));
}

function fmtCountdown(seconds: number): string {
  if (seconds <= 0) return "0:00";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return h > 0
    ? `${h}:${String(m).padStart(2, "0")}`
    : `${m}:${String(s).padStart(2, "0")}`;
}

// Accept "2000000", "2,000,000", "2m", "500k".
function parseHumanNumber(raw: string): number | null {
  const t = raw.trim().replace(/[,_\s]/g, "").toLowerCase();
  if (!t) return null;
  let mult = 1;
  let num = t;
  if (t.endsWith("m")) {
    mult = 1_000_000;
    num = t.slice(0, -1);
  } else if (t.endsWith("k")) {
    mult = 1_000;
    num = t.slice(0, -1);
  }
  const v = Number(num);
  if (!Number.isFinite(v) || v <= 0) return null;
  return Math.round(v * mult);
}

let latest: UsageState | null = null;

function render(state: UsageState): void {
  el.widget.classList.remove(
    "warn",
    "danger",
    "error",
    "idle",
    "has-limit",
    "no-limit"
  );

  if (state.error) {
    el.widget.classList.add("error", "no-limit");
    el.used.textContent = "ccusage?";
    el.percent.textContent = "";
    el.footLeft.textContent = state.error.slice(0, 34);
    el.footRight.classList.add("is-hidden");
    return;
  }

  // hero: tokens used is always shown
  el.used.textContent = state.isActive ? fmtTokens(state.usedTokens) : "idle";

  const hasLimit = state.limit != null && state.limit > 0;
  el.widget.classList.add(hasLimit ? "has-limit" : "no-limit");
  if (!state.isActive) el.widget.classList.add("idle");

  if (hasLimit && state.percent != null) {
    const pct = Math.max(0, Math.min(100, state.percent));
    if (pct >= DANGER) el.widget.classList.add("danger");
    else if (pct >= WARN) el.widget.classList.add("warn");
    el.percent.textContent = `${pct.toFixed(0)}%`;
    el.bar.style.width = `${pct}%`;
    el.footLeft.textContent = `${fmtTokens(state.usedTokens)} / ${fmtTokens(
      state.limit as number
    )}`;
  } else {
    el.percent.textContent = "";
    el.bar.style.width = "0%";
    // state A footer: burn rate
    el.footLeft.textContent = state.isActive
      ? `${Math.round(state.burnRate)} tok/min`
      : "no active 5h block";
  }

  if (state.isActive && state.resetInSeconds > 0) {
    el.footRight.classList.remove("is-hidden");
    el.footRightTxt.textContent = fmtCountdown(state.resetInSeconds);
  } else {
    el.footRight.classList.add("is-hidden");
  }
}

// Tick the countdown locally between backend polls so it feels live.
setInterval(() => {
  if (latest && latest.isActive && !latest.error && latest.resetInSeconds > 0) {
    latest.resetInSeconds -= 1;
    el.footRightTxt.textContent = fmtCountdown(latest.resetInSeconds);
  }
}, 1000);

listen<UsageState>("usage", (event) => {
  latest = event.payload;
  // don't clobber the widget while the user is typing a limit
  if (!el.widget.classList.contains("editing")) render(latest);
});

// ---- limit editing ----
function openEditor(): void {
  el.widget.classList.add("editing");
  if (latest?.limit) el.input.value = String(latest.limit);
  el.input.focus();
  el.input.select();
}
function closeEditor(): void {
  el.widget.classList.remove("editing");
  if (latest) render(latest);
}
async function saveLimit(): Promise<void> {
  const value = parseHumanNumber(el.input.value);
  await invoke("set_limit", { limit: value }); // null clears it
  if (latest) {
    latest.limit = value;
    latest.percent =
      value && latest.isActive ? (latest.usedTokens / value) * 100 : null;
  }
  closeEditor();
}

el.cta.addEventListener("click", (e) => {
  e.stopPropagation();
  openEditor();
});
el.gear.addEventListener("click", (e) => {
  e.stopPropagation();
  if (el.widget.classList.contains("editing")) closeEditor();
  else openEditor();
});
el.saveBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  void saveLimit();
});
el.input.addEventListener("keydown", (e) => {
  if (e.key === "Enter") void saveLimit();
  if (e.key === "Escape") closeEditor();
});

// ---- theme cycling (Dark -> Light -> Glass), persisted in config ----
const THEMES = ["dark", "light", "glass"] as const;
type Theme = (typeof THEMES)[number];
let theme: Theme = "dark";

function applyTheme(t: Theme): void {
  theme = t;
  document.documentElement.setAttribute("data-theme", t);
}

// Load the saved theme on startup so the widget opens in the user's choice.
invoke<string>("get_theme")
  .then((t) => {
    if ((THEMES as readonly string[]).includes(t)) applyTheme(t as Theme);
  })
  .catch(() => {});

el.palette.addEventListener("click", (e) => {
  e.stopPropagation();
  const next = THEMES[(THEMES.indexOf(theme) + 1) % THEMES.length];
  applyTheme(next);
  void invoke("set_theme", { theme: next });
});

// ---- drag to move (Rust persists position); skip interactive controls ----
const appWindow = getCurrentWindow();
el.widget.addEventListener("mousedown", (e) => {
  const target = e.target as HTMLElement;
  if (e.button === 0 && !target.closest(".no-drag")) {
    void appWindow.startDragging();
  }
});
