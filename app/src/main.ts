import { createBackend, type Status } from "./backend";
import "./style.css";

const backend = createBackend();

function el(id: string): HTMLElement {
  const node = document.getElementById(id);
  if (!node) throw new Error(`missing #${id}`);
  return node;
}

function hint(s: Status): string {
  if (!s.game) return "Open Minecraft (Java or Bedrock) to begin.";
  if (s.state === "paused") return `Paused. Switch to Minecraft and press ${s.hotkey ?? "Start"} to continue.`;
  if (s.state === "idle") return `Switch to Minecraft and press ${s.hotkey ?? "Start"} to start.`;
  return `Fishing. Press ${s.hotkey ?? "Stop"} in the game to stop.`;
}

function render(s: Status): void {
  el("state").textContent = s.state;
  el("game").textContent = s.game ? `${s.game.edition === "java" ? "Java" : "Bedrock"} - ${s.game.title}` : "Not found";
  el("focus").textContent = s.focused ? "In game" : "Not in game";
  el("fps").textContent = s.captureFps === null ? "-" : `${Math.round(s.captureFps)} fps`;
  el("casts").textContent = String(s.casts);
  el("bites").textContent = String(s.bites);
  el("timeouts").textContent = String(s.timeouts);
  el("hint").textContent = hint(s);
  const error = el("error");
  error.textContent = s.lastError ?? "";
  error.hidden = !s.lastError;
}

async function call(action: "status" | "start" | "stop"): Promise<void> {
  render(await backend[action]());
}

el("start").addEventListener("click", () => void call("start"));
el("stop").addEventListener("click", () => void call("stop"));
el("build").textContent =
  backend.kind === "app" ? "Development build: no bite detection yet." : "App Preview: simulated, nothing is sent to a game.";
void call("status");
setInterval(() => void call("status"), 250);