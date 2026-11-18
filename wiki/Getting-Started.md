# Getting Started

## Prerequisites (Windows 10 / 11 x64)

On a clean Windows PC, install Git, Rust, and C++ Build Tools via `winget` in an Administrator PowerShell:

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

## Add to Windows Startup

```powershell
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\Winscroller2dAppview.lnk")
$Shortcut.TargetPath = (Resolve-Path ".\target\release\winscroller-2d-appview.exe").Path
$Shortcut.WorkingDirectory = (Resolve-Path ".\target\release").Path
$Shortcut.Save()
```

Once running, look for the **Winscroller 2d Appview** icon in your notification tray and swipe **4 fingers UP** (or press `Alt + Space`)!
