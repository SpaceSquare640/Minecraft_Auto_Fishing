# Change Log

All notable changes to the Minecraft Auto Fishing tool are documented here.

## 0.1.0-alpha.2 — 2026-10-10 (test pre-release, Windows)

Second test build of the Windows app, for testing only. Download it from [GitHub Releases](https://github.com/SpaceSquare640/Minecraft_Auto_Fishing/releases). The installer is not code-signed yet, so Windows SmartScreen or Microsoft Defender may warn you.

- Bedrock Edition: the first cast after you return to the game (for example after closing the pause menu with Esc) should no longer be missed (not verified in game yet). Bedrock ignores the first mouse button event after its window gets the focus back, so the app first sends a mouse release that does nothing in the game, and it waits half a second after F8 — 2026-10-10
- The Windows installer shows the app icon instead of the default installer icon — 2026-10-10
- Settings and log files moved out of the install folder, to `%APPDATA%\io.github.spacesquare640.minecraftautofishing` and `%LOCALAPPDATA%\io.github.spacesquare640.minecraftautofishing\logs`. Uninstalling with "Delete the application data" now removes them. Your settings are carried over; old log files in `%LOCALAPPDATA%\Minecraft Auto Fishing\logs` are no longer used and can be deleted — 2026-10-10

## 0.1.0-alpha.1 — 2026-10-10 (test pre-release, Windows)

First test build of the Windows app, for testing only. Download it from [GitHub Releases](https://github.com/SpaceSquare640/Minecraft_Auto_Fishing/releases). The installer is not code-signed yet, so Windows SmartScreen may warn you (More info › Run anyway).

- First Windows app
  - Bite detection: Java Edition reads the bite subtitle in your game language; Bedrock Edition uses the new Bite Caption resource pack.
  - F8 starts and stops fishing; the app pauses when Minecraft is not the active window.
  - Setup tab installs the Bite Caption resource pack for Java or Bedrock.
  - Optional log files (off by default, kept 7 days) and an update check at startup (can be turned off).
  - If no bite subtitle shows up for several waits in a row, the app suggests what to check (pack, subtitles, casting distance).
  - Bedrock Edition: clicks hold the mouse button briefly, so the game does not miss them (being tested).
  - Pressing F8 after a stop always casts first (reel in before you start).
  - Turning on log files also saves the events of the current session.

## Earlier

- Licensed under GPL-3.0 (`LICENSE` added) — 2026-10-08
