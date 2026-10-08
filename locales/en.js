// English strings. To add a language, copy this file to locales/<lang>.js,
// translate the values, and load it in index.html before js/i18n.js.
window.LOCALES = window.LOCALES || {};
window.LOCALES.en = {
  meta: {
    title: "Minecraft Auto Fishing",
    docsTitle: "Docs · Minecraft Auto Fishing",
    changelogTitle: "Changelog · Minecraft Auto Fishing",
    description: "An open-source tool in development that will watch for a bite, reel in and recast for you in Minecraft. Runs outside the game, not a mod."
  },
  nav: {
    label: "Main",
    brand: "Auto Fishing",
    home: "Home",
    docs: "Docs",
    changelog: "Changelog",
    github: "GitHub"
  },
  docs: {
    title: "Documentation",
    lead: "What the tool does, how it will work and what to check before using it. This page is loaded live from the repository."
  },
  changelog: {
    title: "Changelog",
    lead: "Release history, loaded live from the repository.",
    tool: "Tool releases",
    site: "Website updates"
  },
  loader: {
    loading: "Loading…",
    error: "This document could not be loaded.",
    viewOnGithub: "View it on GitHub"
  },
  a11y: {
    skip: "Skip to content",
    newTab: "(opens in a new tab)"
  },
  hero: {
    status: "In development",
    title: "Minecraft Auto Fishing",
    tagline: "Stop staring at the bobber.",
    description: "An open-source tool, currently in development, that will watch for a bite, reel in and recast for you. It runs outside the game and is not a mod.",
    ctaDemo: "Try the demo",
    ctaGithub: "View on GitHub",
    iconAlt: "Project icon: a pixel-art fishing rod with a gear, a red bobber and a hooked fish"
  },
  demo: {
    title: "Interactive demo",
    lead: "Fish by hand first: cast, wait for the bobber to dip, and reel in fast. Then flip the lever to Auto and watch the tool do it for you.",
    disclaimer: "Simulation, not the actual tool. Timings are illustrative.",
    modeLabel: "Mode",
    manual: "Manual",
    auto: "Auto",
    sceneHint: "Pond scene",
    action: {
      cast: "Cast",
      casting: "Casting…",
      reel: "Reel in",
      reelNow: "Reel in!",
      reeling: "Reeling…",
      pause: "Pause",
      resume: "Resume"
    },
    result: {
      intro: "Press Cast, then reel in as soon as the bobber dips.",
      waiting: "Watch the bobber…",
      bite: "Bite!",
      caught: "Caught {fish} in {ms} ms.",
      missed: "Too slow. The fish got away.",
      early: "Too early. Nothing on the hook.",
      autoRunning: "Auto mode is fishing on its own.",
      autoPaused: "Auto mode paused."
    },
    stats: {
      caption: "Results: manual vs. auto",
      metric: "Result",
      caught: "Caught",
      missed: "Missed",
      best: "Best reaction",
      ms: "{ms} ms",
      none: "—"
    },
    log: {
      label: "Activity log",
      cast: "Casting…",
      waiting: "Watching the bobber…",
      bite: "Bite detected",
      reeling: "Reeling in ({ms} ms after the bite)",
      caught: "Caught {fish}",
      missed: "Missed: too slow",
      early: "Reeled in too early",
      paused: "Paused",
      resumed: "Resumed",
      modeManual: "Switched to Manual",
      modeAuto: "Switched to Auto"
    },
    fish: {
      cod: "Raw Cod",
      salmon: "Raw Salmon",
      pufferfish: "Pufferfish",
      tropical: "Tropical Fish"
    }
  }
};
