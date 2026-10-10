// The UI talks to a Backend, never to Tauri directly, so the same UI runs in the app
// (Rust backend) and on the website as an App Preview (simulated backend, M6).
import { invoke } from "@tauri-apps/api/core";
import { demoBackend } from "./demo-backend";

export type EngineState = "idle" | "casting" | "waiting" | "reeling" | "paused";

export interface Status {
  state: EngineState;
  casts: number;
  bites: number;
  timeouts: number;
  game: { edition: "java" | "bedrock"; title: string } | null;
  focused: boolean;
  captureFps: number | null;
  hotkey: string | null;
  lastError: string | null;
}

export interface Backend {
  readonly kind: "app" | "preview";
  status(): Promise<Status>;
  start(): Promise<Status>;
  stop(): Promise<Status>;
}

const tauriBackend: Backend = {
  kind: "app",
  status: () => invoke<Status>("status"),
  start: () => invoke<Status>("start"),
  stop: () => invoke<Status>("stop"),
};

/** Inside the desktop app Tauri injects its IPC bridge; anywhere else, use the simulation. */
export function createBackend(): Backend {
  return "__TAURI_INTERNALS__" in window ? tauriBackend : demoBackend();
}