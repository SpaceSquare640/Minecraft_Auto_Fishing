// Simulated backend for the website App Preview and for UI work in a normal browser.
// It follows the same rules as crates/engine: cast, wait for a bite, reel, recast.
// Nothing is installed, saved or sent anywhere.
import type { Backend, EngineState, Link, LogEntry, Settings, Status } from "./backend";

const SETTLE_MS = 3000;
const RECAST_MS = 600;
const BITE_MIN_MS = 3000;
const BITE_MAX_MS = 8000;
const LINKS: Record<Link, string> = {
  github: "https://github.com/SpaceSquare640/Minecraft_Auto_Fishing",
  releases: "https://github.com/SpaceSquare640/Minecraft_Auto_Fishing/releases",
  discord: "https://discord.gg/aaUQVJeCgC",
  website: "https://spacesquare640.github.io/Minecraft_Auto_Fishing/",
};

export function demoBackend(): Backend {
  let state: EngineState = "idle";
  let casts = 0;
  let bites = 0;
  let lastBite: number | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let settings: Settings = { saveLogs: false, checkUpdates: false, introSeen: true };
  const log: LogEntry[] = [];

  const add = (text: string): void => {
    const now = new Date();
    log.push({ id: log.length + 1, time: now.toTimeString().slice(0, 8), text });
  };
  const enter = (next: EngineState): void => {
    add(`State: ${state} -> ${next}`);
    state = next;
  };
  const later = (ms: number, next: () => void): void => {
    clearTimeout(timer);
    timer = setTimeout(next, ms);
  };
  const cast = (): void => {
    enter("casting");
    casts += 1;
    later(SETTLE_MS, wait);
  };
  const wait = (): void => {
    enter("waiting");
    later(BITE_MIN_MS + Math.random() * (BITE_MAX_MS - BITE_MIN_MS), () => {
      bites += 1;
      lastBite = Date.now();
      add("Bite seen (subtitles, 0.98)");
      enter("reeling");
      later(RECAST_MS, cast);
    });
  };

  const snapshot = (): Status => ({
    state,
    casts,
    bites,
    timeouts: 0,
    resyncs: 0,
    game: { edition: "java", title: "Minecraft (preview)" },
    focused: true,
    captureFps: 60,
    hotkey: "F8",
    lastError: null,
    detection: {
      methods: ["subtitles", "resource pack"],
      ocrLanguage: "en-US",
      lastBiteSource: lastBite === null ? null : "subtitles",
      lastBiteSecondsAgo: lastBite === null ? null : (Date.now() - lastBite) / 1000,
    },
  });

  add("App Preview started");
  return {
    kind: "preview",
    appInfo: async () => ({ version: "0.1.0" }),
    status: async () => snapshot(),
    start: async () => {
      if (state === "idle" || state === "paused") cast();
      return snapshot();
    },
    stop: async () => {
      clearTimeout(timer);
      if (state !== "idle") enter("idle");
      return snapshot();
    },
    logSince: async (id) => log.filter((e) => e.id > id),
    getSettings: async () => settings,
    setSettings: async (next) => (settings = next),
    openLink: async (target) => {
      window.open(LINKS[target], "_blank", "noopener");
    },
    openLogFolder: async () => undefined,
    installPack: async () => "Preview only: nothing is installed.",
  };
}
