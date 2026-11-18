# Architecture

**Winscroller 2d Appview** follows a clean, multi-threaded actor-channel design where blocking Windows COM/Shell/GDI work and low-level Win32 message loops never block the GPU rendering thread.

## Thread Topology

1. **Main UI Thread (`eframe` / `egui` — `src/main.rs`, `src/ui.rs`)**:
   - Owns the borderless transparent viewport (`with_taskbar(false)`, `with_always_on_top()`, `with_maximized(true)`).
   - Runs discrete spring physics (`anim_progress`, `page_offset`), handles pointer/keyboard/wheel events, and draws the frosted wallpaper, search capsule, multi-page icon grid, and pagination pills.
   - Polls non-blocking `mpsc::Receiver` channels for tray commands, gesture commands, loaded icons, and blurred wallpaper.

2. **Global Gesture & Hotkey Thread (`src/hotkey.rs`)**:
   - Registers `Alt+Space` via `RegisterHotKey`.
   - Installs `WH_KEYBOARD_LL` (`SetWindowsHookExW`) with `GetModuleHandleW(None)`.
   - Installs `SetWinEventHook` for `EVENT_SYSTEM_FOREGROUND`..`EVENT_OBJECT_SHOW`.
   - Runs a blocking Win32 `GetMessageW` loop and dispatches `GestureCommand::{Toggle, Dismiss}`.

3. **Notification Tray Thread (`src/tray.rs`)**:
   - Creates a hidden message window (`WinscrollerTrayMsgClass`) and adds a tray icon via `Shell_NotifyIconW`.
   - Dispatches `TrayCommand::{Toggle, Reload, Exit}` over `mpsc::Sender`.

4. **Background Icon Loader Thread (`src/icon_extractor.rs`)**:
   - Initializes STA COM (`CoInitializeEx(COINIT_APARTMENTTHREADED)`).
   - Iterates discovered `AppEntry` items, reading from `%LOCALAPPDATA%\Winscroller2dAppview\icon_cache_v1\` or extracting via `IShellItemImageFactory`, normalizing with Lanczos3, and streaming `LoadedIcon` frames to the UI thread.

5. **Background Wallpaper Thread (`src/wallpaper.rs`)**:
   - Decodes `TranscodedWallpaper`, downscales to 640×360, runs 2-pass $O(W \cdot H)$ box blur + acrylic tint, and streams `BlurredWallpaper` once ready.
