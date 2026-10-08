// Run: node --test "tests/*.test.js"
const test = require("node:test");
const assert = require("node:assert/strict");
const { createEngine, pickFish, CONFIG } = require("../js/demo-engine.js");

// Deterministic virtual clock: timers only fire when advance() is called.
function fakeClock() {
  let t = 0, nextId = 1, queue = [];
  return {
    now: () => t,
    schedule: (fn, ms) => { const id = nextId++; queue.push({ id, at: t + ms, fn }); return id; },
    cancel: (id) => { queue = queue.filter((q) => q.id !== id); },
    advance(ms) {
      const end = t + ms;
      for (;;) {
        queue.sort((a, b) => a.at - b.at || a.id - b.id);
        const next = queue[0];
        if (!next || next.at > end) break;
        queue.shift(); t = next.at; next.fn();
      }
      t = end;
    },
    pending: () => queue.length
  };
}

function setup(random = () => 0) {
  const clock = fakeClock();
  const events = [];
  const engine = createEngine({ ...clock, random, onEvent: (type, data) => events.push({ type, data }) });
  return { clock, engine, events };
}

const WAIT = CONFIG.waitMinMs; // random() = 0 -> shortest wait

test("manual: reeling in within the window catches a fish and records reaction time", () => {
  const { clock, engine } = setup();
  engine.action();                              // cast
  assert.equal(engine.state.phase, "casting");
  clock.advance(CONFIG.castMs);
  assert.equal(engine.state.phase, "waiting");
  clock.advance(WAIT);
  assert.equal(engine.state.phase, "bite");
  clock.advance(400);
  engine.action();                              // reel in
  assert.equal(engine.state.phase, "reeling");
  clock.advance(CONFIG.reelMs);
  assert.equal(engine.state.phase, "caught");
  assert.equal(engine.state.stats.manual.caught, 1);
  assert.equal(engine.state.stats.manual.bestMs, 400);
  assert.equal(engine.state.lastFish, "cod");
  assert.equal(clock.pending(), 0, "manual mode does not recast by itself");
});

test("manual: too slow counts as missed", () => {
  const { clock, engine } = setup();
  engine.action();
  clock.advance(CONFIG.castMs + WAIT + CONFIG.biteWindowMs);
  assert.equal(engine.state.phase, "missed");
  assert.equal(engine.state.stats.manual.missed, 1);
  assert.equal(engine.state.stats.manual.caught, 0);
});

test("manual: reeling in before the bite counts as early and cancels the bite", () => {
  const { clock, engine } = setup();
  engine.action();
  clock.advance(CONFIG.castMs + 500);
  engine.action();
  assert.equal(engine.state.phase, "early");
  assert.equal(engine.state.stats.manual.missed, 1);
  clock.advance(20000);
  assert.equal(engine.state.phase, "early", "no stale bite timer fires");
});

test("manual: extra presses while casting or reeling are ignored", () => {
  const { clock, engine } = setup();
  engine.action(); engine.action(); engine.action();
  assert.equal(engine.state.phase, "casting");
  assert.equal(clock.pending(), 1);
});

test("auto: loops on its own, never misses, best time stays a manual stat", () => {
  const { clock, engine } = setup(() => 0.5);
  engine.setMode("auto");
  clock.advance(120000);
  assert.ok(engine.state.stats.auto.caught >= 10, `caught ${engine.state.stats.auto.caught}`);
  assert.equal(engine.state.stats.auto.missed, 0);
  assert.equal(engine.state.stats.manual.caught, 0);
});

test("auto: pause stops everything, resume continues", () => {
  const { clock, engine } = setup();
  engine.setMode("auto");
  clock.advance(CONFIG.castMs + 100);
  engine.action();                              // pause
  assert.equal(engine.state.paused, true);
  assert.equal(clock.pending(), 0);
  const caught = engine.state.stats.auto.caught;
  clock.advance(60000);
  assert.equal(engine.state.stats.auto.caught, caught);
  engine.action();                              // resume
  assert.equal(engine.state.paused, false);
  clock.advance(60000);
  assert.ok(engine.state.stats.auto.caught > caught);
});

test("switching modes clears pending timers (no double casts)", () => {
  const { clock, engine } = setup();
  engine.action();
  clock.advance(CONFIG.castMs + 100);           // manual, waiting
  engine.setMode("auto");
  engine.setMode("manual");
  assert.equal(engine.state.phase, "idle");
  assert.equal(clock.pending(), 0);
  clock.advance(60000);
  assert.equal(engine.state.phase, "idle");
});

test("halt returns to idle without counting a miss", () => {
  const { clock, engine } = setup();
  engine.action();
  clock.advance(CONFIG.castMs + WAIT);          // bite
  engine.halt();
  assert.equal(engine.state.phase, "idle");
  assert.equal(engine.state.stats.manual.missed, 0);
  assert.equal(clock.pending(), 0);
});

test("bubbles appear shortly before the bite", () => {
  const { clock, engine, events } = setup();
  engine.action();
  clock.advance(CONFIG.castMs + WAIT - CONFIG.bubbleLeadMs);
  assert.equal(engine.state.nearBite, true);
  assert.ok(events.some((e) => e.type === "nearBite"));
});

test("fish table follows the weights", () => {
  assert.equal(pickFish(0), "cod");
  assert.equal(pickFish(0.6), "salmon");
  assert.equal(pickFish(0.85), "pufferfish");
  assert.equal(pickFish(0.99), "tropical");
});