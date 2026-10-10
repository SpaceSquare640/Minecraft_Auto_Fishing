import { invoke } from "@tauri-apps/api/core";
import "./style.css";

interface Status {
  state: "idle" | "casting" | "waiting" | "reeling" | "paused";
  casts: number;
  bites: number;
  timeouts: number;
}

function el(id: string): HTMLElement {
  const node = document.getElementById(id);
  if (!node) throw new Error(`missing #${id}`);
  return node;
}

function render(s: Status): void {
  el("state").textContent = s.state;
  el("casts").textContent = String(s.casts);
  el("bites").textContent = String(s.bites);
  el("timeouts").textContent = String(s.timeouts);
}

async function call(command: "start" | "stop" | "status"): Promise<void> {
  render(await invoke<Status>(command));
}

el("start").addEventListener("click", () => void call("start"));
el("stop").addEventListener("click", () => void call("stop"));
void call("status");
setInterval(() => void call("status"), 500);
