# Changelog

All notable changes to **Winscroller 2d Appview** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-18

### Added
- **Core Native Launcher & 2D Paginated Grid (`src/ui.rs`, `src/model.rs`)**:
  - Borderless, fullscreen, always-on-top `egui` overlay with zero taskbar button (`with_taskbar(false)`).
  - Screen-filling application grid (92% width up to 1800px) and optional centered floating glass card presentation mode.
  - Borderless icon selection highlight with 1.10x hover/focus scaling, soft ambient glass squircle wash strictly behind the icon glyph, and subtle pill badge for selected labels.
  - Continuous multi-page sliding (`page_offset: f32`) with distance-based alpha fading, two-finger horizontal trackpad scrolling, pointer drag swiping, velocity fling snapping, and continuously morphing pagination pills.
  - Real-time damped harmonic oscillator spring & bounce physics for both open/dismiss (`stiffness = 220.0`, `damping = 17.5`) and page snapping (`stiffness = 180.0`, `damping = 19.0`).
  - Alfred-style multi-tier fuzzy search scoring engine (exact leading prefix `+5000`, word prefixes `+1200/+1000`, camelCase initials `+900`, AppID prefixes `+600`, substrings `+500`, keywords `+300`) with blinking caret and live app counter.
- **Precision Touchpad Gestures & Windows Task View Interception (`src/hotkey.rs`)**:
  - Low-level keyboard hook (`WH_KEYBOARD_LL`) initialized with `GetModuleHandleW(None)` intercepting 4-finger swipe up (`Win+Tab` / `Ctrl+Win+Up`) to suppress Windows Task View and summon Winscroller 2d Appview.
  - Out-of-context `SetWinEventHook` fallback detecting `MultitaskingViewBackgroundHost` and `XamlExplorerHostIslandWindow` to close Task View immediately if triggered directly by DWM.
  - 4-finger swipe down interception (`Win+D` / `Ctrl+Win+Down`) suppressing desktop minimization while visible and initiating smooth spring dismissal.
  - Global `Alt+Space` fallback hotkey via `RegisterHotKey`.
- **High-DPI Unplated Icon Extraction & Wallpaper Blur (`src/icon_extractor.rs`, `src/wallpaper.rs`)**:
  - Unplated 256×256 Shell COM extraction (`SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP`) with direct screen DC `GetDIBits`.
  - GDI 0-alpha transparency repair eliminating solid black bounding boxes on legacy Win32 apps (`cmd.exe`, `magnify.exe`, etc.).
  - Automatic transparent-margin bounding box cropping, Lanczos3 90%-fill resampling, and persistent PNG disk caching in `%LOCALAPPDATA%\Winscroller2dAppview\icon_cache_v1\`.
  - Background `TranscodedWallpaper` loader with 640×360 downscale, 2-pass separable $O(W \cdot H)$ box blur, and dark frosted acrylic vignette (`rgba(14, 18, 30, 0.55)`).
- **App Scanner, Launcher & System Tray Daemon (`src/app_scanner.rs`, `src/launcher.rs`, `src/tray.rs`, `src/config.rs`)**:
  - Dual scanner combining UWP/MSIX (`Get-StartApps`) and Start Menu `.lnk` shortcuts with noise filtering for administrative tools and uninstallers.
  - Native activation via `explorer.exe shell:AppsFolder\...` and Win32 `ShellExecuteW`.
  - Native Win32 `Shell_NotifyIconW` system tray icon with right-click menu (Open, Edit Configuration, Reload Installed Apps, Exit).
  - Persistent TOML configuration at `%APPDATA%\Winscroller2dAppview\config.toml`.
- **Documentation, Wiki, Website & CI/CD**:
  - Added `README.md`, `KEYBINDINGS.md`, `CONFIGURATION.md`, `USAGE.md`, `RELEASE_NOTES.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `LICENSE`, `winscroller-2d-appview.example.toml`, mdBook documentation (`docs/`), GitHub Wiki (`wiki/`), interactive showcase site (`website/`), custom SVG concept diagrams, and GitHub Actions workflows (CI, GitHub Pages, Release).
