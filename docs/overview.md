# Overview

Minecraft Auto Fishing is an open-source tool that will automate the repetitive part of fishing in Minecraft: watching the bobber, reeling in at the right moment and casting again.

> **Status:** In development. Nothing described below is released yet; it describes the planned behavior.

## Supported platforms

| Edition | Platforms |
|---|---|
| Minecraft: Java Edition | Windows, macOS, Linux |
| Minecraft: Bedrock Edition | Windows 10 / 11 |

Consoles (Xbox, PlayStation, Nintendo Switch) and mobile devices (iOS, Android) are not supported. Supported game versions will be listed here at release.

## How it will work

1. **Start** — hold a fishing rod, face the water and press **F8** in the game (or **Start** in the app).
2. **Cast** — the tool casts the fishing rod.
3. **Detect a bite** — it reads the game's on-screen subtitles and waits for the bite subtitle.
4. **Reel in** — it reels in immediately, then casts again and repeats.

Press **F8** again to stop. The tool also pauses when Minecraft is not the active window, and only sends clicks to Minecraft.

### Bite detection

| Edition | Method | What you need to do |
|---|---|---|
| Java Edition | Reads the bite subtitle in your game language | Turn on subtitles (Options › Accessibility › Show Subtitles) |
| Bedrock Edition | Reads the subtitle added by the **Bite Caption** resource pack | Install the pack, turn it on in Global Resources and turn on captions |

Bedrock uses the same subtitle for every splash, so the resource pack gives the bite its own subtitle. The pack only changes a subtitle; it does not change gameplay. Java players can use the pack too, for example when Windows has no text-recognition language for their game language. The app can install the pack for you from its **Setup** tab.

Stay close to the water and do not cast too far: a bite that is too far away makes no sound and no subtitle.

### App settings

- **Save logs to files** — off by default. Logs stay on your PC for 7 days and contain only app events: no keystrokes, screenshots or audio.
- **Check for updates when the app starts** — on by default. The app asks GitHub for the latest release; like any website, GitHub sees your IP address.

## What it is not

- **Not a mod.** It runs outside the game and does not change the game itself. The optional resource pack only adds a subtitle.
- **Not affiliated with Mojang or Microsoft.** Minecraft is a trademark of Mojang Studios.

## Before you use it

Many multiplayer servers do not allow automated or AFK fishing. Check the rules of the server you play on before using any automation tool.

## Repository layout

| Branch | Purpose |
|---|---|
| `Source_Code` (default) | Source code of the tool, these docs and the tool's change log |
| `Minecraft_Auto_Fishing_Website_Preview` | The project website (GitHub Pages) |
