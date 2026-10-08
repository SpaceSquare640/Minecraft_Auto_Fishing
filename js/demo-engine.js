// Fishing demo state machine. No DOM access: timers, clock and randomness are injected,
// so the rules can be tested in Node with a fake clock (tests/demo-engine.test.js).
//
//   idle -> casting -> waiting -> bite -> reeling -> caught
//                         |          \-> missed (manual, too slow)
//                         \-> early (manual, reeled in before the bite)
//   Auto mode reacts to the bite by itself and recasts after each catch.
(function (root, factory) {
  if (typeof module === "object" && module.exports) module.exports = factory();
  else root.DemoEngine = factory();
})(typeof self !== "undefined" ? self : this, function () {
  "use strict";

  var CONFIG = {
    castMs: 600,
    waitMinMs: 2000,
    waitMaxMs: 6000,
    biteWindowMs: 1000,      // manual reaction window; 0.8 s proved too tight for touch input
    autoReactMinMs: 200,
    autoReactMaxMs: 300,
    reelMs: 600,
    autoRecastMs: 1000,
    bubbleLeadMs: 1200       // bubbles approach the bobber this long before a bite
  };

  // Approximates Minecraft's fish loot weights.
  var FISH = [
    { id: "cod", weight: 60 },
    { id: "salmon", weight: 25 },
    { id: "pufferfish", weight: 13 },
    { id: "tropical", weight: 2 }
  ];

  function pickFish(r) {
    var total = FISH.reduce(function (s, f) { return s + f.weight; }, 0);
    var x = r * total;
    for (var i = 0; i < FISH.length; i++) { if ((x -= FISH[i].weight) < 0) return FISH[i].id; }
    return FISH[FISH.length - 1].id;
  }

  function createEngine(deps) {
    var cfg = Object.assign({}, CONFIG, deps.config);
    var schedule = deps.schedule, cancel = deps.cancel, now = deps.now, random = deps.random;
    var emit = deps.onEvent || function () {};
    var timers = [];

    var state = {
      mode: "manual",
      phase: "idle",
      paused: false,
      nearBite: false,
      lastFish: null,
      lastReactionMs: null,
      stats: {
        manual: { caught: 0, missed: 0, bestMs: null },
        auto: { caught: 0, missed: 0 }
      }
    };
    var biteAt = 0;

    function later(ms, fn) { timers.push(schedule(fn, ms)); }
    function clearTimers() { timers.forEach(cancel); timers = []; }
    function between(min, max) { return Math.round(min + random() * (max - min)); }

    function set(phase, data) {
      state.phase = phase;
      if (phase !== "waiting") state.nearBite = false;
      emit(phase, data || {});
    }

    function cast() {
      clearTimers();
      state.lastFish = null;
      set("casting");
      later(cfg.castMs, startWaiting);
    }

    function startWaiting() {
      set("waiting");
      var wait = between(cfg.waitMinMs, cfg.waitMaxMs);
      if (wait > cfg.bubbleLeadMs) {
        later(wait - cfg.bubbleLeadMs, function () { state.nearBite = true; emit("nearBite", {}); });
      }
      later(wait, bite);
    }

    function bite() {
      biteAt = now();
      set("bite");
      if (state.mode === "auto") {
        later(between(cfg.autoReactMinMs, cfg.autoReactMaxMs), function () { reel(now() - biteAt); });
      } else {
        later(cfg.biteWindowMs, function () {
          state.stats.manual.missed++;
          set("missed");
        });
      }
    }

    function reel(reactionMs) {
      clearTimers();
      state.lastReactionMs = reactionMs;
      set("reeling", { ms: reactionMs });
      later(cfg.reelMs, function () {
        var stats = state.stats[state.mode];
        state.lastFish = pickFish(random());
        stats.caught++;
        if (state.mode === "manual" && (stats.bestMs === null || reactionMs < stats.bestMs)) stats.bestMs = reactionMs;
        set("caught", { fish: state.lastFish, ms: reactionMs });
        if (state.mode === "auto") later(cfg.autoRecastMs, cast);
      });
    }

    // Primary button. Manual: cast / reel in. Auto: pause / resume.
    function action() {
      if (state.mode === "auto") return state.paused ? resume() : pause();
      switch (state.phase) {
        case "idle": case "caught": case "missed": case "early": return cast();
        case "waiting":
          clearTimers();
          state.stats.manual.missed++;
          return set("early");
        case "bite": return reel(now() - biteAt);
        default: return; // casting / reeling: ignore extra presses
      }
    }

    // Stop without counting anything (used when the demo scrolls out of view).
    function halt() { clearTimers(); set("idle"); }

    function pause() {
      if (state.mode !== "auto" || state.paused) return;
      state.paused = true;
      halt();
      emit("paused", {});
    }

    function resume() {
      if (state.mode !== "auto" || !state.paused) return;
      state.paused = false;
      emit("resumed", {});
      cast();
    }

    function setMode(mode) {
      if (mode === state.mode) return;
      clearTimers();
      state.mode = mode;
      state.paused = false;
      emit("mode", { mode: mode });
      if (mode === "auto") cast(); else set("idle");
    }

    return {
      state: state,
      action: action,
      setMode: setMode,
      pause: pause,
      resume: resume,
      halt: halt,
      pendingTimers: function () { return timers.length; }
    };
  }

  return { createEngine: createEngine, pickFish: pickFish, CONFIG: CONFIG, FISH: FISH };
});