# Winscroller 2d Appview v0.1.0 — Release Notes

**Winscroller 2d Appview v0.1.0** is the inaugural public release of our native Rust fullscreen & floating 2D Launchpad application overview for Windows 10 and Windows 11.

---

## Highlights

### 🖐️ Native 4-Finger Precision Touchpad Gestures (Task View Replaced)
- **4-Finger Swipe Up** summons **Winscroller 2d Appview** seamlessly instead of Windows Task View.
- Dual-layer suppression (`WH_KEYBOARD_LL` + `SetWinEventHook` for `MultitaskingViewBackgroundHost` / `XamlExplorerHostIslandWindow`) stops Windows App View from flickering or taking focus.
- **4-Finger Swipe Down**, <kbd>Esc</kbd>, downward swipe, or clicking the blurred backdrop smoothly dismisses the launcher without minimizing your open desktop windows.
- Global <kbd>Alt</kbd> + <kbd>Space</kbd> hotkey and system tray icon provide instant fallback triggers on desktops without touchpads.

### 🌊 Continuous Two-Finger Swiping & Underdamped Spring Physics
- Swipe two fingers horizontally across the trackpad (or drag with mouse/touch) to glide continuously between pages with sub-pixel smoothness.
- Velocity-aware fling snapping and real-time damped harmonic oscillator physics ($F = -kx - cv$) give both open/dismiss (`0.91 → 1.025 → 1.00`) and page transitions an organic, snappy bounce.
- Bottom pagination pills dynamically morph width and tint in sync with fractional `page_offset`.

### ✨ Borderless Selection & Frosted Wallpaper Blur
- **Zero-Border Selection**: Removed harsh bounding cards and stroke outlines. Selected icons scale cleanly to **1.10x** over a soft, borderless frosted glass halo strictly behind the icon glyph.
- **Dynamic Frosted Backdrop**: Reads your live Windows `TranscodedWallpaper`, downscales to 640×360, runs a 2-pass separable box blur, and applies a dark acrylic vignette (`rgba(14, 18, 30, 0.55)`).

### 🎨 Crystal-Clear Unplated Icons
- Extracts unplated 256×256 icons via `IShellItemImageFactory` (`SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP`).
- Repairs legacy Win32 0-alpha GDI bitmaps so apps like Command Prompt and Magnifier render with clean transparency instead of solid black boxes.
- Auto-crops transparent padding and scales with Lanczos3 resampling into `%LOCALAPPDATA%\Winscroller2dAppview\icon_cache_v1\`.

### 🔍 Alfred-Style Instant Fuzzy Search
- Start typing anywhere to filter all installed Win32 and UWP/MSIX apps in sub-millisecond time.
- Supports leading prefix boosts, camelCase initials (`vsc` → Visual Studio Code, `cp` → Control Panel), AppID prefixes (`cmd`), and multi-term queries.

---

## Package Contents (`winscroller-2d-appview-x86_64-pc-windows-msvc.zip`)
- `winscroller-2d-appview.exe` — Standalone native Windows x64 binary (no runtime dependencies)
- `winscroller-2d-appview.example.toml` — Documented configuration template
- `README.md`, `KEYBINDINGS.md`, `CONFIGURATION.md`, `LICENSE`

---

## Quick Start
1. Extract the archive to a permanent folder (e.g., `C:\Tools\Winscroller2dAppview\`).
2. Double-click `winscroller-2d-appview.exe`.
3. Look for the **Winscroller 2d Appview** icon in your Windows notification tray (near the clock).
4. Swipe **4 fingers UP** on your touchpad or press <kbd>Alt</kbd> + <kbd>Space</kbd> to toggle!
