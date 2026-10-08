// Connects DemoEngine (rules) to the page: scene, lever, button, result text, stats and log.
(function () {
  "use strict";

  var root = document.getElementById("demo");
  if (!root || !window.DemoEngine) return;

  var scene = root.querySelector(".demo__scene");
  var line = scene.querySelector(".scene__line");
  var actionBtn = root.querySelector("[data-demo-action]");
  var resultEl = root.querySelector("[data-demo-result]");
  var logEl = root.querySelector("[data-demo-log]");
  var announcer = root.querySelector("[data-demo-announcer]");
  var MAX_LOG = 6;
  var ROD_TIP = "114,2";
  var BOBBER_TOP = { rest: 25, bite: 29 };   // keeps the line attached when the bobber sinks

  function t(key, params) { return window.I18n.t(document.documentElement.lang, key, params); }
  function fishName(id) { return t("demo.fish." + id); }

  var engine = window.DemoEngine.createEngine({
    schedule: function (fn, ms) { return setTimeout(fn, ms); },
    cancel: clearTimeout,
    now: function () { return performance.now(); },
    random: Math.random,
    onEvent: onEvent
  });
  var state = engine.state;

  function log(key, params) {
    var li = document.createElement("li");
    li.textContent = t("demo.log." + key, params);
    logEl.appendChild(li);
    while (logEl.children.length > MAX_LOG) logEl.removeChild(logEl.firstChild);
  }

  // Polite announcements for things a sighted player would notice instantly.
  // Auto mode announces only start/pause, so a screen reader is not flooded every few seconds.
  function announce(text) {
    announcer.textContent = "";
    setTimeout(function () { announcer.textContent = text; }, 50);   // clear-then-set so repeats are re-read
  }

  function resultText() {
    if (state.mode === "auto") return t(state.paused ? "demo.result.autoPaused" : "demo.result.autoRunning");
    switch (state.phase) {
      case "casting": case "waiting": return t("demo.result.waiting");
      case "bite": case "reeling": return t("demo.result.bite");
      case "caught": return t("demo.result.caught", { fish: fishName(state.lastFish), ms: Math.round(state.lastReactionMs) });
      case "missed": return t("demo.result.missed");
      case "early": return t("demo.result.early");
      default: return t("demo.result.intro");
    }
  }

  function actionLabel() {
    if (state.mode === "auto") return t(state.paused ? "demo.action.resume" : "demo.action.pause");
    switch (state.phase) {
      case "casting": return t("demo.action.casting");
      case "waiting": return t("demo.action.reel");
      case "bite": return t("demo.action.reelNow");
      case "reeling": return t("demo.action.reeling");
      default: return t("demo.action.cast");
    }
  }

  function setStat(name, value) { root.querySelector('[data-stat="' + name + '"]').textContent = value; }

  function render() {
    scene.setAttribute("data-phase", state.phase);
    scene.setAttribute("data-mode", state.mode);
    scene.setAttribute("data-near-bite", String(state.nearBite));
    if (state.lastFish) scene.setAttribute("data-fish", state.lastFish);
    line.setAttribute("points", ROD_TIP + " 60," + (state.phase === "bite" ? BOBBER_TOP.bite : BOBBER_TOP.rest));

    actionBtn.textContent = actionLabel();
    actionBtn.setAttribute("data-state", state.mode === "manual" && state.phase === "bite" ? "urgent" : "");
    resultEl.textContent = resultText();

    var s = state.stats, none = t("demo.stats.none");
    setStat("manual.caught", s.manual.caught);
    setStat("manual.missed", s.manual.missed);
    setStat("manual.best", s.manual.bestMs === null ? none : t("demo.stats.ms", { ms: Math.round(s.manual.bestMs) }));
    setStat("auto.caught", s.auto.caught);
    setStat("auto.missed", s.auto.missed);
    setStat("auto.best", none);
  }

  function onEvent(type, data) {
    var manual = state.mode === "manual";
    switch (type) {
      case "casting": log("cast"); break;
      case "waiting": log("waiting"); break;
      case "bite": log("bite"); if (manual) announce(t("demo.result.bite")); break;
      case "reeling": log("reeling", { ms: Math.round(data.ms) }); break;
      case "caught": log("caught", { fish: fishName(data.fish) }); if (manual) announce(resultText()); break;
      case "missed": log("missed"); announce(resultText()); break;
      case "early": log("early"); announce(resultText()); break;
      case "paused": log("paused"); announce(t("demo.result.autoPaused")); break;
      case "resumed": log("resumed"); announce(t("demo.result.autoRunning")); break;
      case "mode":
        log(data.mode === "auto" ? "modeAuto" : "modeManual");
        if (data.mode === "auto") announce(t("demo.result.autoRunning"));
        break;
    }
    render();
  }

  actionBtn.addEventListener("click", function () { engine.action(); });
  scene.addEventListener("click", function () { if (state.mode === "manual") engine.action(); });
  root.querySelectorAll('input[name="demo-mode"]').forEach(function (radio) {
    radio.addEventListener("change", function () { if (radio.checked) engine.setMode(radio.value); });
  });

  // Stop while off screen or in a background tab: saves CPU/battery and avoids
  // counting misses the player could not see.
  var inView = true, suspendedAuto = false;
  function updateVisibility() {
    var visible = inView && !document.hidden;
    if (!visible) {
      if (state.mode === "auto" && !state.paused) { suspendedAuto = true; engine.pause(); }
      else if (state.mode === "manual" && /^(casting|waiting|bite|reeling)$/.test(state.phase)) engine.halt();
    } else if (suspendedAuto) {
      suspendedAuto = false;
      if (state.mode === "auto" && state.paused) engine.resume();
    }
  }
  if ("IntersectionObserver" in window) {
    new IntersectionObserver(function (entries) {
      inView = entries[0].isIntersecting;
      updateVisibility();
    }, { threshold: 0.2 }).observe(scene);
  }
  document.addEventListener("visibilitychange", updateVisibility);

  render();
})();