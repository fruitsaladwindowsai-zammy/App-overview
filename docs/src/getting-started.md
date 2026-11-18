# Getting Started

This chapter walks you through installing, launching, and verifying **Winscroller 2d Appview** on Windows 10 or Windows 11.

## System Requirements
- **OS**: Windows 10 (1809 or newer) or Windows 11 (64-bit)
- **GPU**: DirectX / OpenGL compatible display adapter (used by `eframe`/`egui`)
- **Input**: Windows Precision Touchpad (for 4-finger & 2-finger gestures) and/or keyboard + mouse

## Option 1: Pre-Compiled Release Binary
1. Download `winscroller-2d-appview-x86_64-pc-windows-msvc.zip` from the [GitHub Releases](https://github.com/fruitsaladwindowsai-zammy/App-overview/releases) page.
2. Extract to a permanent folder (e.g., `C:\Tools\Winscroller2dAppview\`).
3. Double-click `winscroller-2d-appview.exe`.

## Option 2: Build from Source on a Clean Windows Machine
Open an **Administrator PowerShell** prompt and install prerequisites:
```powershell
winget install --id Git.Git --accept-package-agreements --accept-source-agreements
winget install --id Rustlang.Rustup --accept-package-agreements --accept-source-agreements
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```
Restart PowerShell, then clone and build:
```powershell
git clone https://github.com/fruitsaladwindowsai-zammy/App-overview.git
cd "App-overview"
cargo build --release
& ".\target\release\winscroller-2d-appview.exe"
```

## First Launch Verification
1. Look in the Windows Notification Tray (bottom-right near the clock) for the **Winscroller 2d Appview** icon.
2. Swipe **4 fingers UP** on your touchpad (or press `Alt + Space`).
3. Confirm the fullscreen frosted overlay springs smoothly into view with your installed apps.
4. Swipe **two fingers left/right** to slide smoothly between pages.
5. Swipe **4 fingers DOWN** or press `Esc` to dismiss cleanly.
