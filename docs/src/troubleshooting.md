# Troubleshooting & FAQ

## 4-Finger Gesture Opens Windows Task View
1. Open Windows **Settings → Bluetooth & devices → Touchpad → Four-finger gestures** and verify 4-finger gestures are enabled.
2. Make sure only one instance of `winscroller-2d-appview.exe` is running (and exit any older `savanna-win.exe` instance if present).
3. If an elevated (Administrator) window currently has focus, Windows UIPI prevents a standard-privilege `WH_KEYBOARD_LL` hook from intercepting keys over that window. Run Winscroller 2d Appview elevated if you frequently focus Administrator terminals.

## Refreshing Stale or Missing Icons
Delete the cached icon folder and relaunch:
```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\Winscroller2dAppview\icon_cache_v1"
```
You can also right-click the tray icon and choose **🔄 Reload Installed Apps** to rescan newly installed programs.

## Wallpaper Didn't Update After Changing Desktop Background
Winscroller reads `%APPDATA%\Microsoft\Windows\Themes\TranscodedWallpaper` asynchronously at startup. Exit and relaunch from the tray icon to refresh the blurred wallpaper texture.

## Exiting Completely
Right-click the **Winscroller 2d Appview** system tray icon and choose **❌ Exit Winscroller 2d Appview**, or terminate `winscroller-2d-appview.exe` in Task Manager.
