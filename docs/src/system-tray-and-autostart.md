# System Tray & Windows Autostart

## Native Notification Tray Daemon (`src/tray.rs`)
Winscroller 2d Appview runs with `#![windows_subsystem = "windows"]` and `with_taskbar(false)` so it never clutters your Windows taskbar. Instead, it registers a lightweight Win32 `Shell_NotifyIconW` tray icon:

- **Left-Click / Double-Click**: Instantly toggles Winscroller 2d Appview.
- **Right-Click Menu**:
  - **🚀 Open Winscroller 2d Appview (`Alt+Space`)**
  - **⚙ Edit Configuration** (opens `config.toml` in Notepad)
  - **🔄 Reload Installed Apps** (re-scans Start Menu & UWP packages)
  - **❌ Exit Winscroller 2d Appview** (cleanly unhooks and terminates)

## Configure Automatic Startup with Windows

### Method 1: One-Line PowerShell Shortcut (Recommended)
```powershell
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\Winscroller2dAppview.lnk")
$Shortcut.TargetPath = "d:\Projects\App overview\target\release\winscroller-2d-appview.exe"
$Shortcut.WorkingDirectory = "d:\Projects\App overview\target\release"
$Shortcut.Save()
```

### Method 2: Manual `shell:startup` Folder
1. Press `Win + R`, enter `shell:startup`, press `Enter`.
2. Right-click → **New → Shortcut** → point to `winscroller-2d-appview.exe`.
