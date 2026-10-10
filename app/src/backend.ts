// The UI talks to a Backend, never to Tauri directly, so the same UI runs in the app
// (Rust backend) and on the website as an App Preview (simulated backend, M6).
import { invoke } from "@tauri-apps/api/core";
import { demoBackend } from "./demo-backend";

export type EngineState = "idle" | "casting" | "waiting" | "reeling" | "paused";
export type Link = "github" | "discord" | "website" | "releases";

export interface Status {
  state: EngineState;
  casts: number;
  bites: number;
  timeouts: number;
  resyncs: number;
  game: { edition: "java" | "bedrock"; title: string } | null;
  focused: boolean;
  captureFps: number | null;
  hotkey: string | null;
  lastError: string | null;
  detection: {
    methods: string[];
    ocrLanguage: string | null;
    lastBiteSource: string | null;
    lastBiteSecondsAgo: number | null;
  };
}

export interface Settings {
  saveLogs: boolean;
  checkUpdates: boolean;
  introSeen: boolean;
}

export interface LogEntry {
  id: number;
  time: string;
  text: string;
}

export interface Backend {
  readonly kind: "app" | "preview";
  appInfo(): Promise<{ version: string }>;
  status(): Promise<Status>;
  start(): Promise<Status>;
  stop(): Promise<Status>;
  logSince(id: number): Promise<LogEntry[]>;
  getSettings(): Promise<Settings>;
  setSettings(settings: Settings): Promise<Settings>;
  openLink(target: Link): Promise<void>;
  openLogFolder(): Promise<void>;
  installPack(edition: "java" | "bedrock"): Promise<string>;
}

const tauriBackend: Backend = {
  kind: "app",
  appInfo: () => invoke("app_info"),
  status: () => invoke("status"),
  start: () => invoke("start"),
  stop: () => invoke("stop"),
  logSince: (id) => invoke("log_since", { id }),
  getSettings: () => invoke("get_settings"),
  setSettings: (settings) => invoke("set_settings", { settings }),
  openLink: (target) => invoke("open_link", { target }),
  openLogFolder: () => invoke("open_log_folder"),
  installPack: (edition) => invoke("install_pack", { edition }),
};

/** Inside the desktop app Tauri injects its IPC bridge; anywhere else, use the simulation. */
export function createBackend(): Backend {
  return "__TAURI_INTERNALS__" in window ? tauriBackend : demoBackend();
}
