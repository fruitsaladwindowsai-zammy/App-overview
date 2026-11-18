# Usage Guide — Winscroller 2d Appview

This guide covers daily workflows, touchpad gestures, keyboard navigation, background tray management, and tips for getting the most out of **Winscroller 2d Appview**.

---

## 1. Launching the Background Daemon

Start `winscroller-2d-appview.exe` once per Windows session (or configure autostart via `shell:startup`):

```powershell
& "d:\Projects\App overview\target\release\winscroller-2d-appview.exe"
```

Because the binary uses `#![windows_subsystem = "windows"]` and `with_taskbar(false)`:
- No console window appears.
- No taskbar icon clutters your Windows taskbar.
- A notification icon appears in your **System Tray** (bottom-right near the clock).
- On first launch, Winscroller scans installed UWP/MSIX and Start Menu `.lnk` applications and caches normalized 256×256 → target-size icons to `%LOCALAPPDATA%\Winscroller2dAppview\icon_cache_v1\`.

---

## 2. Summoning & Dismissing the Overview

### Summoning
- **4-Finger Swipe Up** (Default Touchpad Gesture): Place four fingers on your Windows Precision Touchpad and swipe upward. Winscroller intercepts Windows Task View and pops open with an underdamped spring bounce.
- **<kbd>Alt</kbd> + <kbd>Space</kbd>**: Works globally from any desktop window.
- **Tray Icon Click**: Left-click or double-click the notification tray icon.

### Dismissing
- **4-Finger Swipe Down**: Swipe four fingers downward on your touchpad. Winscroller eats the `Win+D` sequence so your open apps stay visible while Winscroller springs away.
- **<kbd>Esc</kbd> Key**: Clears your active search query first; pressing <kbd>Esc</kbd> on an empty search box smoothly dismisses the overlay.
- **Downward Drag / Scroll**: Drag downward (`> 55px`) anywhere on the canvas or scroll down past the final page.
- **Click Empty Wallpaper**: Left-clicking anywhere outside the search capsule and app icons dismisses the view.

---

## 3. Browsing Pages with Continuous Swipes

When your installed applications span multiple pages (e.g., 73 apps across 3 pages on a 7×5 grid):
- **Two-Finger Horizontal Swipe**: Slide two fingers left or right on your touchpad. Pages move continuously 1:1 with your fingers and fade smoothly as they exit the viewport.
- **Fling Momentum**: A quick flick in either direction exceeds the velocity threshold (`|v| > 1.2`) and glides cleanly into the next or previous page with spring-damper settling.
- **Mouse / Touch Drag**: Hold left mouse button (or single touch) anywhere on the grid and drag horizontally to pan pages.
- **Pagination Pills**: Click any pill/dot at the bottom center of the screen to spring directly to that page.

---

## 4. Instant Fuzzy Search Workflow

You do not need to click the search capsule first — **just start typing** as soon as the overlay opens:

| What You Type | What Matches First | Scoring Mechanism |
|---|---|---|
| `chr` | **Google Chrome** / **Chromium** | Whole-query prefix bonus (`+5000`) |
| `vsc` | **Visual Studio Code** | Acronym / camelCase initials (`+900`) |
| `cp` | **Control Panel** | Initials prefix (`+900`) |
| `cmd` | **Command Prompt** | AppID word prefix (`+600`) |
| `fox` | **Firefox** | Substring match (`+500`) |
| `win term` | **Windows Terminal** | Multi-term AND prefix scoring |

Press <kbd>Enter</kbd> immediately to launch the highlighted top match and dismiss the overview.

---

## 5. System Tray Context Menu

Right-click the **Winscroller 2d Appview** icon in the Windows notification tray to access:

1. **🚀 Open Winscroller 2d Appview (`Alt+Space`)** — Toggles the fullscreen/floating launcher.
2. **⚙ Edit Configuration** — Opens `%APPDATA%\Winscroller2dAppview\config.toml` in Notepad.
3. **🔄 Reload Installed Apps** — Re-scans UWP/MSIX packages and Start Menu `.lnk` shortcuts after installing or uninstalling software.
4. **❌ Exit Winscroller 2d Appview** — Cleanly unhooks keyboard/WinEvent hooks, removes the tray icon, and exits.

---

## 6. Refreshing Icons & Wallpaper

- **Wallpaper Updates**: Winscroller reads `%APPDATA%\Microsoft\Windows\Themes\TranscodedWallpaper` on startup. If you change your Windows desktop wallpaper, restart Winscroller from the tray to pick up the new frosted background.
- **Icon Cache Refresh**: If an app updates its icon, clear `%LOCALAPPDATA%\Winscroller2dAppview\icon_cache_v1\` and relaunch Winscroller to trigger fresh high-DPI extraction.
