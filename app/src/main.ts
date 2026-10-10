import { createBackend, type Link, type Settings, type Status } from "./backend";
import { setLanguage, t } from "./i18n";
import { checkForUpdate } from "./update";
import "./fonts.css";
import "./style.css";

const backend = createBackend();
let version = "";
let settings: Settings = { saveLogs: false, checkUpdates: true, introSeen: true };
let lastLogId = 0;
let hotkey = "F8";

function el<T extends HTMLElement = HTMLElement>(id: string): T {
  const node = document.getElementById(id);
  if (!node) throw new Error(`missing #${id}`);
  return node as T;
}

// ---------- tabs (WAI-ARIA tabs pattern with arrow keys) ----------
const tabs = Array.from(document.querySelectorAll<HTMLButtonElement>('[role="tab"]'));
function select(tab: HTMLButtonElement): void {
  for (const other of tabs) {
    const selected = other === tab;
    other.setAttribute("aria-selected", String(selected));
    other.tabIndex = selected ? 0 : -1;
    el(other.getAttribute("aria-controls") ?? "").hidden = !selected;
  }
}
tabs.forEach((tab, i) => {
  tab.addEventListener("click", () => select(tab));
  tab.addEventListener("keydown", (e) => {
    const step = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (!step) return;
    const next = tabs[(i + step + tabs.length) % tabs.length];
    select(next);
    next.focus();
  });
});

// ---------- dialogs ----------
function confirmDialog(text: string): Promise<boolean> {
  const dialog = el<HTMLDialogElement>("confirm");
  el("confirm-text").textContent = text;
  dialog.showModal();
  return new Promise((resolve) => {
    const done = (answer: boolean) => () => {
      dialog.close();
      resolve(answer);
    };
    el("confirm-yes").onclick = done(true);
    el("confirm-no").onclick = done(false);
    dialog.oncancel = done(false);
  });
}

// ---------- fishing ----------
function hint(s: Status): string {
  const key = s.hotkey ?? "Start";
  if (!s.game) return t("hint.noGame");
  if (s.state === "paused") return t("hint.paused", { key });
  if (s.state === "idle") return t(s.game.edition === "bedrock" ? "hint.bedrockIdle" : "hint.idle", { key });
  return t("hint.running", { key });
}

function render(s: Status): void {
  hotkey = s.hotkey ?? hotkey;
  document.body.dataset.state = s.state;
  el("state").textContent = t(`state.${s.state}`);
  el("hint").textContent = hint(s);
  for (const key of ["casts", "bites", "timeouts", "resyncs"] as const) el(key).textContent = String(s[key]);
  el("game").textContent = s.game ? `${s.game.edition === "java" ? "Java" : "Bedrock"} - ${s.game.title}` : t("fishing.notFound");
  el("focus").textContent = t(s.focused ? "fishing.inGame" : "fishing.notInGame");
  el("fps").textContent = s.captureFps === null ? "-" : `${Math.round(s.captureFps)} fps`;
  const d = s.detection;
  el("detection").textContent = d.methods.length
    ? d.methods
        .map((m) => (m === "subtitles" ? t("fishing.lookSubtitle") + (d.ocrLanguage ? ` (${t("fishing.ocr", { lang: d.ocrLanguage })})` : "") : t("fishing.lookPack")))
        .join(t("fishing.or"))
    : "-";
  el("last-bite").textContent =
    d.lastBiteSecondsAgo === null ? "-" : t("fishing.secondsAgo", { n: Math.round(d.lastBiteSecondsAgo), source: d.lastBiteSource ?? "" });
  const error = el("error");
  error.textContent = s.lastError ?? "";
  error.hidden = !s.lastError;
  const noBite = el("no-bite");
  noBite.textContent = s.noBiteHint && s.game ? t(`noBite.${s.game.edition}`) : "";
  noBite.hidden = !noBite.textContent;
}

async function refresh(): Promise<void> {
  render(await backend.status());
  const entries = await backend.logSince(lastLogId);
  if (entries.length === 0) {
    if (lastLogId === 0) el("events").replaceChildren(Object.assign(document.createElement("li"), { textContent: t("fishing.noEvents") }));
    return;
  }
  const list = el("events");
  if (lastLogId === 0) list.replaceChildren();
  for (const entry of entries) {
    const item = document.createElement("li");
    const time = document.createElement("time");
    time.textContent = entry.time;
    item.append(time, ` ${entry.text}`);
    list.prepend(item);
  }
  while (list.children.length > 50) list.lastElementChild?.remove();
  lastLogId = entries[entries.length - 1].id;
}

el("start").addEventListener("click", async () => render(await backend.start()));
el("stop").addEventListener("click", async () => render(await backend.stop()));

// ---------- setup ----------
async function install(edition: "java" | "bedrock"): Promise<void> {
  if (edition === "java" && !(await confirmDialog(t("setup.javaConfirm")))) return;
  const result = el("pack-result");
  try {
    result.textContent = await backend.installPack(edition);
  } catch (e) {
    result.textContent = String(e);
  }
}
el("install-java").addEventListener("click", () => void install("java"));
el("install-bedrock").addEventListener("click", () => void install("bedrock"));

// ---------- settings ----------
async function save(change: Partial<Settings>): Promise<void> {
  try {
    settings = await backend.setSettings({ ...settings, ...change });
    el("settings-result").textContent = t("settings.saved");
  } catch (e) {
    el("settings-result").textContent = String(e);
  }
}
el<HTMLInputElement>("save-logs").addEventListener("change", (e) => void save({ saveLogs: (e.target as HTMLInputElement).checked }));
el<HTMLInputElement>("check-updates").addEventListener("change", (e) => void save({ checkUpdates: (e.target as HTMLInputElement).checked }));
el("open-logs").addEventListener("click", () => void backend.openLogFolder());

// ---------- about ----------
async function updateCheck(): Promise<void> {
  const result = el("update-result");
  const download = el("download");
  result.textContent = t("about.checking");
  const outcome = await checkForUpdate(version);
  download.hidden = outcome.kind !== "newer";
  result.textContent =
    outcome.kind === "newer" ? t("about.newVersion", { v: outcome.version })
    : outcome.kind === "latest" ? t("about.upToDate")
    : outcome.kind === "none" ? t("about.noReleases")
    : outcome.kind === "limited" ? t("about.limited")
    : t("about.failed");
}
el("check-now").addEventListener("click", () => void updateCheck());
el("download").addEventListener("click", () => void backend.openLink("releases"));
document.querySelectorAll<HTMLButtonElement>("[data-link]").forEach((button) => {
  button.addEventListener("click", () => void backend.openLink(button.dataset.link as Link));
});

// ---------- start-up ----------
async function init(): Promise<void> {
  setLanguage("en"); // English only for now; more languages plug into i18n.ts
  version = (await backend.appInfo()).version;
  el("version").textContent = t("about.version", { v: version });
  el("about-version").textContent = t("about.version", { v: version });
  el("build").textContent = t(backend.kind === "app" ? "build.app" : "build.preview");
  settings = await backend.getSettings();
  el<HTMLInputElement>("save-logs").checked = settings.saveLogs;
  el<HTMLInputElement>("check-updates").checked = settings.checkUpdates;
  await refresh();
  el("setup-common").textContent = t("setup.common", { key: hotkey });
  el("setup-bedrock3").textContent = t("setup.bedrock3", { key: hotkey });

  if (!settings.introSeen) {
    const intro = el<HTMLDialogElement>("intro");
    intro.showModal();
    await new Promise<void>((resolve) => {
      el("intro-ok").onclick = () => {
        intro.close();
        resolve();
      };
      intro.oncancel = () => resolve();
    });
    await save({ introSeen: true });
  }
  if (settings.checkUpdates) void updateCheck();
  setInterval(() => void refresh(), 250);
}
void init();
