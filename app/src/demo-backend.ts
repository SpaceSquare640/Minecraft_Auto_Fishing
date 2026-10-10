// Simulated backend for the website App Preview and for UI work in a normal browser.
// It follows the same rules as crates/engine: cast, wait for a bite, reel, recast.
import type { Backend, EngineState, Status } from "./backend";

const SETTLE_MS = 1500;
const RECAST_MS = 600;
const BITE_MIN_MS = 3000;
const BITE_MAX_MS = 8000;

export function demoBackend(): Backend {
  let state: EngineState = "idle";
  let casts = 0;
  let bites = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  const later = (ms: number, next: () => void): void => {
    clearTimeout(timer);
    timer = setTimeout(next, ms);
  };

  const cast = (): void => {
    state = "casting";
    casts += 1;
    later(SETTLE_MS, wait);
  };

  const wait = (): void => {
    state = "waiting";
    later(BITE_MIN_MS + Math.random() * (BITE_MAX_MS - BITE_MIN_MS), () => {
      bites += 1;
      state = "reeling";
      later(RECAST_MS, cast);
    });
  };

  const snapshot = (): Status => ({
    state,
    casts,
    bites,
    timeouts: 0,
    game: { edition: "java", title: "Minecraft (preview)" },
    focused: true,
    captureFps: 60,
    hotkey: "F8",
    lastError: null,
  });

  return {
    kind: "preview",
    status: async () => snapshot(),
    start: async () => {
      if (state === "idle" || state === "paused") cast();
      return snapshot();
    },
    stop: async () => {
      clearTimeout(timer);
      state = "idle";
      return snapshot();
    },
  };
}