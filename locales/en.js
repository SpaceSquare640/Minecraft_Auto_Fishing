// English strings. To add a language, copy this file to locales/<lang>.js,
// translate the values, and load it in index.html before js/i18n.js.
window.LOCALES = window.LOCALES || {};
window.LOCALES.en = {
  meta: {
    title: "Minecraft Auto Fishing",
    docsTitle: "Docs · Minecraft Auto Fishing",
    changelogTitle: "Changelog · Minecraft Auto Fishing",
    notFoundTitle: "Page not found · Minecraft Auto Fishing",
    description: "An open-source auto-fishing tool in development for Minecraft: Java Edition (Windows, macOS, Linux) and Bedrock Edition (Windows). It runs outside the game and is not a mod."
  },
  nav: {
    label: "Main",
    brand: "Auto Fishing",
    home: "Home",
    docs: "Docs",
    changelog: "Changelog",
    github: "GitHub",
    discord: "Discord"
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
  notFound: {
    title: "Page not found",
    text: "This page doesn't exist, or it has moved. Try the home page, the docs, or ask on Discord.",
    home: "Back to the home page"
  },
  loader: {
    loading: "Loading…",
    error: "This document could not be loaded.",
    viewOnGithub: "View it on GitHub"
  },
  how: {
    title: "How it works",
    lead: "Four steps, repeated for as long as you let it run.",
    more: "Read the full docs →",
    steps: {
      cast: { title: "1. Cast", text: "The tool casts your fishing rod." },
      detect: { title: "2. Detect a bite", text: "It watches for the moment a fish bites. Detection method: to be decided." },
      reel: { title: "3. Reel in", text: "It reels in right away, before the fish gets away." },
      recast: { title: "4. Recast", text: "It casts again and keeps going until you stop it." }
    }
  },
  features: {
    title: "Features",
    status: { planned: "Planned", available: "Available" },
    items: {
      reel: { title: "Auto reel-in", text: "Reels in the moment a fish bites." },
      recast: { title: "Auto recast", text: "Casts again after every catch, with no input from you." },
      outside: { title: "Runs outside the game", text: "Not a mod, so no game files are changed. For Java Edition (Windows, macOS, Linux) and Bedrock Edition (Windows)." },
      open: { title: "Open source", text: "Free to read, change and share under GPL-3.0." }
    }
  },
  appPreview: {
    title: "App preview",
    lead: "This is the real app interface, running on a simulated game: try Start, the tabs and the settings. Nothing is installed and no clicks are sent anywhere.",
    frameTitle: "Minecraft Auto Fishing app preview",
    open: "Open the app preview in a full page"
  },
  start: {
    title: "Getting started",
    text: "A test build for Windows is available on GitHub Releases, and setup steps are in the Docs. The installer is not code-signed yet, so Windows may show a warning. Join the community on Discord for updates and questions.",
    download: "Download test build (Windows)",
    discord: "Join the community on Discord",
    cta: "Follow on GitHub"
  },
  faq: {
    title: "FAQ",
    items: {
      mod: { q: "Is it a mod?", a: "No. It runs outside Minecraft and does not change any game files." },
      servers: { q: "Can I use it on multiplayer servers?", a: "Many servers do not allow automated or AFK fishing. Check the rules of the server you play on first." },
      official: { q: "Is it made by Mojang or Microsoft?", a: "No. This is an independent fan project and is not affiliated with Mojang or Microsoft." },
      release: { q: "When will it be released?", a: "A first test build for Windows is available on GitHub Releases. Progress toward a stable release is posted on the Changelog page." },
      versions: { q: "Which Minecraft editions and platforms will it support?", a: "Minecraft: Java Edition on Windows, macOS and Linux, and Minecraft: Bedrock Edition on Windows. Consoles and mobile devices are not supported. Supported game versions will be listed in the docs at release." },
      free: { q: "Is it free?", a: "Yes. It is open source under the GPL-3.0 license." }
    }
  },
  footer: {
    name: "Minecraft Auto Fishing",
    disclaimer: "Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.",
    navLabel: "Footer",
    license: "License: GPL-3.0",
    copyright: "© 2026 SpaceSquare640 · Licensed under GPL-3.0",
    credits: "UI components adapted from",
    backToTop: "Back to top"
  },
  a11y: {
    skip: "Skip to content",
    newTab: "(opens in a new tab)"
  },
  hero: {
    status: "In development",
    title: "Minecraft Auto Fishing",
    tagline: "Stop staring at the bobber.",
    description: "An open-source tool, currently in development, that will watch for a bite, reel in and recast for you in Minecraft: Java Edition and Bedrock Edition on PC. It runs outside the game and is not a mod.",
    ctaDemo: "Try the demo",
    ctaGithub: "View on GitHub",
    ctaDiscord: "Join Discord",
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
