# Troubleshooting & FAQ

### 1. 4-finger swipe up still activates Windows Task View over an Admin terminal
Windows UIPI blocks non-elevated `WH_KEYBOARD_LL` hooks when a High Integrity (Administrator) window holds focus. Launch `winscroller-2d-appview.exe` as Administrator if you frequently focus elevated shells.

### 2. How do I force-rebuild the icon cache?
```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\Winscroller2dAppview\icon_cache_v1"
```
Then restart Winscroller 2d Appview.

### 3. Newly installed app doesn't appear yet
Right-click the system tray icon and select **🔄 Reload Installed Apps** to re-scan `Get-StartApps` and Start Menu `.lnk` folders.

### 4. How do I exit cleanly?
Right-click the notification tray icon and choose **❌ Exit Winscroller 2d Appview**.
