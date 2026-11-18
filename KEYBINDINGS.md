# Keybindings, Touchpad Gestures & Mouse Controls

Complete input reference for **Winscroller 2d Appview**.

---

## Global Triggers (Works Anywhere in Windows)

| Input | Action | Implementation |
|---|---|---|
| **4-Finger Swipe Up** | Toggle Winscroller 2d Appview (suppresses Task View) | `WH_KEYBOARD_LL` (`Win+Tab` / `Ctrl+Win+Up`) + `SetWinEventHook` |
| **4-Finger Swipe Down** | Dismiss Winscroller 2d Appview (when visible) | `WH_KEYBOARD_LL` (`Win+D` / `Ctrl+Win+Down`) |
| <kbd>Alt</kbd> + <kbd>Space</kbd> | Toggle Winscroller 2d Appview | Win32 `RegisterHotKey` (`MOD_ALT \| MOD_NOREPEAT`) |
| **Left-Click / Double-Click Tray** | Toggle Winscroller 2d Appview | `Shell_NotifyIconW` (`WM_LBUTTONUP` / `WM_LBUTTONDBLCLK`) |
| **Right-Click Tray** | Open native context menu | `TrackPopupMenu` (`WM_RBUTTONUP`) |

> [!NOTE]
> When **Winscroller 2d Appview** is **hidden**, `Win+D` (4-finger swipe down) passes through untouched to Windows so your normal Show Desktop gesture still works. It is only intercepted while Winscroller is open.

---

## Touchpad & Pointer Gestures (Inside Overview)

| Gesture / Action | Behavior |
|---|---|
| **2-Finger Swipe Left / Right** | Smooth continuous horizontal page sliding with momentum and spring-damper snap |
| **Pointer / Touch Drag Left / Right** | Pan across pages smoothly; fast flicks (`|velocity| > 1.2`) fling to next/previous page |
| **Pointer / Touch Drag Down (`> 55px`)** | Dismisses Winscroller smoothly with bounce |
| **Mouse Wheel Up / Down** | Navigates to previous / next page (scrolling down past the last page dismisses) |
| **Hover Icon** | Updates active selection, scales icon **1.10x**, displays borderless ambient halo |
| **Left-Click Icon** | Launches the application and initiates smooth spring dismissal |
| **Left-Click Pagination Pill** | Jumps smoothly to the clicked page with spring interpolation |
| **Left-Click Empty Wallpaper** | Dismisses the overview smoothly |

---

## Keyboard Shortcuts (Inside Overview)

| Key | Behavior |
|---|---|
| **Any Printable Character** | Appends to live fuzzy search query and resets view to page 1 |
| <kbd>Backspace</kbd> | Deletes last character from search query and re-filters results |
| <kbd>Esc</kbd> | Clears active search query; if query is already empty, smoothly dismisses overview |
| <kbd>Enter</kbd> | Launches the currently selected application and dismisses overview |
| <kbd>→</kbd> (**Right Arrow**) | Moves selection right; automatically rolls over to the next page at column boundary |
| <kbd>←</kbd> (**Left Arrow**) | Moves selection left; automatically rolls back to the previous page at column 0 |
| <kbd>↓</kbd> (**Down Arrow**) | Moves selection down one row within the current page |
| <kbd>↑</kbd> (**Up Arrow**) | Moves selection up one row within the current page |
| <kbd>PageDown</kbd> | Springs to the next page and selects its first icon |
| <kbd>PageUp</kbd> | Springs to the previous page and selects its first icon |
