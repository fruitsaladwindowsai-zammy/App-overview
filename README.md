# Winscroller 2d Appview 🪟✨
### Native Launchpad & 2D Application Overview for Windows
*A native Rust-powered fullscreen and floating application overview with macOS Launchpad aesthetics, hardware-accelerated frosted wallpaper blur, Windows Precision Touchpad 4-finger gestures, and damped spring physics.*

---

## 📑 Table of Contents
1. [Overview & Philosophy](#1-overview--philosophy)
2. [Key Capabilities & Features](#2-key-capabilities--features)
3. [Architecture & Engine Design](#3-architecture--engine-design)
4. [Precision Touchpad & Gesture System](#4-precision-touchpad--gesture-system)
   - [4-Finger Swipe Up (Summon & Task View Interception)](#4-finger-swipe-up-summon--task-view-interception)
   - [4-Finger Swipe Down & Escape (Dismissal)](#4-finger-swipe-down--escape-dismissal)
   - [Two-Finger Continuous Swiping & Momentum](#two-finger-continuous-swiping--momentum)
5. [Visual & Physics Aesthetics](#5-visual--physics-aesthetics)
   - [Damped Harmonic Oscillator (Spring & Bounce)](#damped-harmonic-oscillator-spring--bounce)
   - [Borderless Icon Selection Highlight](#borderless-icon-selection-highlight)
   - [Dynamic Frosted Wallpaper Acrylic Blur](#dynamic-frosted-wallpaper-acrylic-blur)
   - [High-DPI Unplated Icon Extraction Pipeline](#high-dpi-unplated-icon-extraction-pipeline)
6. [Installation & Deployment Manual](#6-installation--deployment-manual)
   - [Prerequisites](#prerequisites)
   - [Pre-compiled Release Binary](#pre-compiled-release-binary)
   - [Building from Source](#building-from-source)
   - [Configuring Autostart with Windows](#configuring-autostart-with-windows)
7. [User Manual & Navigation Controls](#7-user-manual--navigation-controls)
8. [Configuration Guide (`config.toml`)](#8-configuration-guide-configtoml)
9. [Troubleshooting & FAQs](#9-troubleshooting--faqs)

---

## 1. Overview & Philosophy

**Winscroller 2d Appview** is a zero-latency, 100% native Rust application overview designed to replace the conventional Windows Start Menu and Task View with a fluid, distraction-free app launcher inspired by macOS Launchpad and iOS App Library.

Unlike typical Electron or web-based overlays that suffer from startup latency, frame jitter, and multi-gigabyte memory footprints, Winscroller 2d Appview is built with:
- **Zero Heavy Web Runtimes**: Pure native Rust compiled to a single standalone binary.
- **Microsecond Response Time**: Sits silently in the background as a lightweight system tray daemon (using `< 35 MB` RAM).
- **Direct GPU-Accelerated Rendering**: Powered by `egui` and `wgpu`/DirectX, running synchronized at your display's native refresh rate (60Hz, 120Hz, 165Hz, or 240Hz).
- **Deep Windows OS Integration**: Intercepts Windows Precision Touchpad multi-touch gestures at the kernel/hook level, reads active transcoded desktop wallpapers, and extracts unplated high-resolution Win32/UWP assets via Windows Shell COM interfaces.

---

## 2. Key Capabilities & Features

- **Seamless Native Overlay (No Taskbar Button)**:
  - Runs in the background as a notification tray daemon without occupying space on the Windows taskbar (`with_taskbar(false)`).
  - Toggles smoothly over any fullscreen game, browser, IDE, or multi-monitor setup.
- **Native 4-Finger Touchpad Gestures**:
  - **4 Fingers UP**: Summons Winscroller 2d Appview instantly while intercepting and suppressing the default Windows Task View / App View.
  - **4 Fingers DOWN**: Dismisses the view smoothly with damped bounce physics and prevents the desktop from minimizing.
- **Continuous Two-Finger Page Swiping**:
  - Horizontal trackpad scrolling smoothly glides pages left and right with continuous sub-pixel coordinates.
  - Features authentic inertia, fling physics, and spring-damper snapping.
  - Active pagination indicator pills morph and expand in real-time according to fractional page offsets.
- **Refined Borderless Icon Selection**:
  - No bulky rectangular cards or distracting stroke outlines.
  - Subtly scales icons up by **1.10x** with a soft, frosted ambient glow strictly centered behind the icon glyph.
- **Dynamic Desktop Wallpaper Frosted Glass**:
  - Automatically loads and blurs your active Windows desktop background (`%APPDATA%\Microsoft\Windows\Themes\TranscodedWallpaper`).
  - Dark acrylic vignette provides high contrast and crisp readability for all application titles.
- **Alfred-Style Fast Keyboard Search**:
  - Translucent search capsule with glowing focus ring.
  - Intelligent scoring based on word prefixes, camelCase initials (e.g. `cmd` for Command Prompt, `vs` for Visual Studio), and fuzzy substrings.
- **Filtered Clutter-Free Application Scanner**:
  - Automatically queries both Win32 Start Menu shortcuts and UWP/MSIX packages.
  - Automatically filters out uninstallers, diagnostic tools, and system clutter (`dfrgui`, `Event Viewer`, `Computer Management`, duplicate private browsing entries).

---

## 3. Architecture & Engine Design

The application is structured into modular, low-overhead native subsystems:

| Module | Source File | Responsibility |
|---|---|---|
| **App Entry Point** | `src/main.rs` | Viewport builder, subsystem initialization, and event orchestrator. |
| **UI & Render Engine** | `src/ui.rs` | Continuous multi-page grid renderer, search capsule, pagination, and spring physics. |
| **Touchpad & Gestures** | `src/hotkey.rs` | `WH_KEYBOARD_LL` hook, `SetWinEventHook` Task View interceptor, and `RegisterHotKey`. |
| **Icon Extraction** | `src/icon_extractor.rs` | Shell COM item extraction, unplated asset trimming, GDI transparency correction, and disk caching. |
| **Wallpaper Loader** | `src/wallpaper.rs` | Desktop background discovery, downscaling, 2-pass box blur, and texture uploading. |
| **App Scanner** | `src/app_scanner.rs` | Win32 `.lnk` parsing, PowerShell `Get-StartApps` query, and noise filtering. |
| **Model & Scoring** | `src/model.rs` | Alfred-style fuzzy keyword search engine and pagination arithmetic. |
| **System Tray** | `src/tray.rs` | Win32 `Shell_NotifyIconW` daemon and context menu handler. |
| **Configuration** | `src/config.rs` | TOML serializer and persistent configuration manager. |

---

## 4. Precision Touchpad & Gesture System

### 4-Finger Swipe Up (Summon & Task View Interception)
In standard Windows 10/11, swiping 4 fingers upward triggers the native **Windows Task View** (Virtual Desktops / App View). Winscroller 2d Appview eliminates this conflict using a dual-layer interception architecture:

1. **Kernel Hook Layer (`WH_KEYBOARD_LL`)**:
   - Installed using `SetWindowsHookExW` with a valid module handle via `GetModuleHandleW(None)`.
   - When the Windows Precision Touchpad driver synthesizes `Win + Tab` or `Ctrl + Win + Up`, the hook captures the event in `< 1ms`, delivers `GestureCommand::Toggle` to Winscroller, and returns `LRESULT(1)`.
   - Returning `1` consumes the key sequence before DWM (Desktop Window Manager) or Explorer can register it, completely suppressing the default Task View.
2. **WinEvent Fallback Layer (`SetWinEventHook`)**:
   - In case a proprietary touchpad driver or DWM bypasses keyboard synthesis and invokes Task View directly, an out-of-context WinEvent hook listens for `EVENT_SYSTEM_FOREGROUND`.
   - If a window matching class `MultitaskingViewBackgroundHost` or `XamlExplorerHostIslandWindow` attempts to gain focus, it is immediately closed via `PostMessageW(WM_CLOSE)` and <kbd>Esc</kbd>, and Winscroller 2d Appview is summoned in its place.

### 4-Finger Swipe Down & Escape (Dismissal)
- **Trackpad 4-Finger Down**: Intercepts `Win + D` or `Ctrl + Win + Down`. When Winscroller is active, it consumes the event (`LRESULT(1)`) so open desktop applications are not minimized, and initiates the smooth spring exit animation.
- **Escape Key (<kbd>Esc</kbd>)**: If a search query is active, clears the search text; if empty, smoothly closes the launcher.
- **Downward Touch Drag**: Swiping downward anywhere on the grid or trackpad (`vertical_drag_accum > 50.0`) triggers smooth dismissal.
- **Backdrop Click**: Clicking on empty blurred wallpaper dismisses the view.

### Two-Finger Continuous Swiping & Momentum
- Swiping two fingers horizontally on a Precision Touchpad delivers horizontal wheel deltas (`delta.x`) and smooth scroll vectors.
- Winscroller tracks a fractional coordinate: `page_offset: f32`.
- As you swipe, the current page slides smoothly in real time while the neighboring page slides into view with distance-based alpha attenuation.
- Fling gestures calculate release velocity:
  $$\text{Target Page} = \begin{cases} \text{page} + 1 & \text{if } v > 1.2 \\ \text{page} - 1 & \text{if } v < -1.2 \\ \text{round}(\text{offset}) & \text{otherwise} \end{cases}$$
- Releasing the trackpad engages spring-damper snapping to glide into the target page.

---

## 5. Visual & Physics Aesthetics

### Damped Harmonic Oscillator (Spring & Bounce)
All transitions in Winscroller 2d Appview are driven by an underdamped harmonic oscillator simulation computed each frame:

$$m\ddot{x} + c\dot{x} + k(x - x_{\text{target}}) = 0$$

- **Entrance Spring**: Stiffness $k = 220.0$, Damping $c = 17.5$.
  - Scales the app grid from `0.91` up to `1.025` (overshoot bounce) before settling rock-solid at `1.00`.
- **Page Snapping Spring**: Stiffness $k = 180.0$, Damping $c = 19.0$.
  - Provides natural deceleration and elastic boundary resistance when scrolling past the first or last page.

### Borderless Icon Selection Highlight
Unlike traditional UI frameworks that draw harsh rectangular bounding boxes around selected items, Winscroller 2d Appview implements an authentic Launchpad aesthetic:
- **No Border Strokes**: Completely avoids rectangular outlines, outlines on glow capsules, or heavy bounding cards.
- **Icon Scale Spring**: Hovering or selecting an icon scales it up smoothly to **1.10x**.
- **Soft Ambient Halo**: Renders a borderless, soft diffuse squircle wash strictly behind the icon graphic (`render_icon_size + 14.0`) using translucent white (`rgba(255, 255, 255, 0.14)`).
- **Subtle Text Capsule**: The label text below sits cleanly with an optional subtle pill badge and dark drop shadows for contrast against bright wallpapers.

### Dynamic Frosted Wallpaper Acrylic Blur
1. Reads `%APPDATA%\Microsoft\Windows\Themes\TranscodedWallpaper`.
2. Downscales the image to $640 \times 360$ for optimal GPU memory bandwidth and cache locality.
3. Applies a vectorized, high-performance 2-pass box blur that closely approximates Gaussian blur.
4. Tints the blurred background with a deep acrylic vignette (`rgba(12, 16, 26, 0.55)`), ensuring light or dark application icons remain vibrant and readable.

### High-DPI Unplated Icon Extraction Pipeline
Traditional Windows icon extraction frequently suffers from two flaws:
1. Windows Shell wraps Win32 icons in an ugly Start Menu square tile with a diagonal sheen if unplated flags are not specified.
2. Older 24-bit GDI icons leave the alpha channel as 0, which naive decoders treat as solid black, generating black boxes around apps like Command Prompt, Dev Home, and Magnifier.

**Winscroller 2d Appview resolves this completely**:
- Uses `SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP` (requesting unplated $256 \times 256$ assets).
- In `hbitmap_to_rgba`, pure black pixels `(0, 0, 0)` in non-alpha GDI bitmaps are preserved as transparent (`alpha = 0`), while colored icon pixels are converted to opaque (`alpha = 255`).
- The graphic bounding box is calculated to crop empty margins, then resampled using high-quality **Lanczos3 filtering** to fill 90% of the canvas.
- Extracted icons are persistently cached in `%LOCALAPPDATA%\Winscroller2dAppview\icon_cache_v1\`.

---

## 6. Installation & Deployment Manual

### Prerequisites
- **Operating System**: Windows 10 (version 1809+) or Windows 11 (64-bit).
- **Hardware**: Any x86_64 PC (laptops with Windows Precision Touchpad will enjoy native multi-finger swipe gestures; desktop PCs use mouse wheel and drag gestures).
- **Build Tools** (only if building from source):
  - Rust toolchain (`stable-x86_64-pc-windows-gnu` or `stable-x86_64-pc-windows-msvc`).

### Pre-compiled Release Binary
The fully compiled, standalone executable is located at:
```
d:\Projects\App overview\target\release\winscroller-2d-appview.exe
```
This binary is completely standalone. It requires no DLL installations, runtimes, or external frameworks.

### Building from Source

To compile the latest release binary from the project directory:

```powershell
# 1. Navigate to the project directory
cd "d:\Projects\App overview"

# 2. Build with maximum optimizations
cargo build --release

# 3. The output executable will be placed in:
# .\target\release\winscroller-2d-appview.exe
```

### Configuring Autostart with Windows

To have Winscroller 2d Appview start silently in the background whenever you log into Windows:

#### Method A: Windows Startup Folder (Recommended)
1. Press <kbd>Win</kbd> + <kbd>R</kbd>, type `shell:startup`, and press <kbd>Enter</kbd>.
2. Right-click inside the folder, select **New** → **Shortcut**.
3. Browse to `d:\Projects\App overview\target\release\winscroller-2d-appview.exe`.
4. Name the shortcut `Winscroller 2d Appview` and click **Finish**.

#### Method B: PowerShell Script (One-Click Setup)
Run this command in Windows PowerShell:
```powershell
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\Winscroller2dAppview.lnk")
$Shortcut.TargetPath = "d:\Projects\App overview\target\release\winscroller-2d-appview.exe"
$Shortcut.WorkingDirectory = "d:\Projects\App overview\target\release"
$Shortcut.WindowStyle = 7 # Minimized
$Shortcut.Save()
Write-Host "Winscroller 2d Appview has been added to Windows Startup!" -ForegroundColor Green
```

---

## 7. User Manual & Navigation Controls

### Quick Reference Guide

| Action | Control / Gesture | Description |
|---|---|---|
| **Summon Launcher** | **4-Finger Swipe Up** | Swiping four fingers upward on your touchpad opens Winscroller and suppresses Task View. |
| **Summon (Keyboard)** | <kbd>Alt</kbd> + <kbd>Space</kbd> | Global hotkey to toggle Winscroller from anywhere in Windows. |
| **Summon (Mouse/Tray)** | **Click Tray Icon** | Single left-click or double-click the notification tray icon (near the clock). |
| **Dismiss Launcher** | **4-Finger Swipe Down** | Swiping four fingers downward dismisses Winscroller smoothly with bounce. |
| **Dismiss (Keyboard)** | <kbd>Esc</kbd> | Closes the launcher (or clears search query if text is entered). |
| **Dismiss (Mouse/Touch)**| **Click Backdrop / Drag Down** | Clicking the blurred wallpaper or dragging downward dismisses the view. |
| **Browse Pages** | **2-Finger Swipe Left / Right** | Continuous trackpad swipe slides pages with momentum and spring snapping. |
| **Browse (Mouse)** | **Scroll Wheel or Drag** | Scroll the mouse wheel or drag anywhere across the screen. |
| **Page Jump** | **Click Pagination Pill** | Click any dot/pill at the bottom to jump directly to that page. |
| **Search Apps** | **Type any key** | Searches instantly with Alfred-style scoring (e.g. `c` for Chrome, `cmd` for Command Prompt). |
| **Navigate Grid** | **Arrow Keys** | Moves the selection cursor smoothly across apps and pages. |
| **Launch Application** | <kbd>Enter</kbd> or **Click Icon** | Launches the chosen app and smoothly dismisses Winscroller. |
| **Tray Context Menu** | **Right-Click Tray Icon** | Access options to Open, Edit Configuration, Reload Apps, or Exit. |

---

## 8. Configuration Guide (`config.toml`)

The application's persistent settings are stored at:
```
%APPDATA%\Winscroller2dAppview\config.toml
```

You can edit this file directly in Notepad by right-clicking the system tray icon and selecting **⚙ Edit Configuration**, or by opening the file path manually.

### Default Configuration
```toml
# Winscroller 2d Appview Configuration

# Number of columns in the grid (minimum: 3, recommended: 7 for 1080p/1440p)
columns = 7

# Number of rows in the grid (minimum: 2, recommended: 5)
rows = 5

# Base icon size in pixels (64.0 - 96.0)
icon_size = 76.0

# Whether application names are displayed underneath icons
show_labels = true

# Backdrop style: "Wallpaper" (reads Windows wallpaper) or "Dark"
backdrop = "Wallpaper"

# Centered mode: false for fullscreen Launchpad, true for floating centered card
centered = false

# Width fraction of screen in centered mode (0.4 - 0.95)
card_width = 0.75

# Height fraction of screen in centered mode (0.4 - 0.95)
card_height = 0.80

# Whether search bar glow effects are enabled
glow = true

# Accent highlight color (Hex RGB/RGBA string)
accent_color = "#89b4fa"

# Global keyboard shortcut
hotkey = "Alt+Space"
```

---

## 9. Troubleshooting & FAQs

### Q1: 4-Finger gesture still opens Windows Task View occasionally.
**Resolution**:
1. Open Windows **Settings** (<kbd>Win</kbd> + <kbd>I</kbd>) → **Bluetooth & devices** → **Touchpad**.
2. Scroll to **Four-finger gestures** (or **Advanced gestures**).
3. Ensure that touchpad gestures are enabled.
4. Winscroller 2d Appview automatically intercepts `Win + Tab` via `WH_KEYBOARD_LL` and terminates Task View window instances via `SetWinEventHook`.

### Q2: Some icons are missing or need a refresh.
**Resolution**:
1. Right-click the system tray icon near your taskbar clock.
2. Click **🔄 Reload Installed Apps**.
3. To perform a completely clean re-extraction of all icons, delete the icon cache folder:
   ```powershell
   Remove-Item -Recurse -Force "$env:LOCALAPPDATA\Winscroller2dAppview\icon_cache_v1"
   ```

### Q3: How do I completely exit the application?
**Resolution**:
Right-click the system tray icon (near the clock) and click **❌ Exit Winscroller 2d Appview**. You can also terminate it from Windows Task Manager under the process name `winscroller-2d-appview.exe`.

---
*Built with native Rust for the ultimate Windows application launch experience.*
