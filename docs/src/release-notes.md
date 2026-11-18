# Release Notes (v0.1.0)

**Released**: November 2026  
**Target Platform**: Windows 10 (1809+) & Windows 11 (`x86_64`)

## Major Features
- **Project Identity**: Renamed and packaged cleanly as **Winscroller 2d Appview** (`winscroller-2d-appview.exe`).
- **Dual-Layer 4-Finger Touchpad Engine**:
  - `WH_KEYBOARD_LL` with valid `GetModuleHandleW(None)` suppresses `Win+Tab` / `Ctrl+Win+Up` and opens Winscroller instead of Windows Task View.
  - `SetWinEventHook` catches `MultitaskingViewBackgroundHost` / `XamlExplorerHostIslandWindow` if opened directly by DWM.
  - 4-finger swipe down (`Win+D`) dismisses Winscroller without minimizing desktop windows.
- **Borderless Icon Selection**: Pure ambient translucent halo behind the icon (`1.10x` scale, zero border stroke).
- **Continuous Two-Finger Page Swiping**: Sub-pixel horizontal scrolling, velocity fling detection, spring-damper snapping, and morphing pagination pills.
- **Crystal-Clear Unplated Icons**: `SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP`, GDI 0-alpha transparency fix, auto-crop, and Lanczos3 PNG caching.
- **Dynamic Wallpaper Blur**: Fast 2-pass separable box blur on Windows `TranscodedWallpaper`.
