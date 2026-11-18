# Architecture & Internals

Winscroller 2d Appview separates GPU rendering from blocking Win32/COM operations across 5 dedicated threads connected via lock-free `std::sync::mpsc` channels:

1. **Main UI Thread (`src/main.rs`, `src/ui.rs`)**: `eframe` / `egui` borderless fullscreen overlay, spring physics, continuous multi-page swipe renderer.
2. **Gesture & Hotkey Thread (`src/hotkey.rs`)**: `RegisterHotKey` (`Alt+Space`), `WH_KEYBOARD_LL` hook, and `SetWinEventHook` Task View interceptor.
3. **Notification Tray Thread (`src/tray.rs`)**: Hidden Win32 message window + `Shell_NotifyIconW` + native `TrackPopupMenu`.
4. **STA COM Icon Worker (`src/icon_extractor.rs`)**: `CoInitializeEx(COINIT_APARTMENTTHREADED)` + `IShellItemImageFactory` + Lanczos3 normalization + PNG cache.
5. **Wallpaper Worker (`src/wallpaper.rs`)**: `TranscodedWallpaper` decoding, 2-pass $O(W \cdot H)$ box blur, and acrylic vignette tinting.
