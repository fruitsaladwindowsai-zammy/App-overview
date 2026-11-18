# Configuration Reference (`config.toml`)

**Winscroller 2d Appview** automatically generates and loads its TOML configuration file at:

```text
%APPDATA%\Winscroller2dAppview\config.toml
```

You can edit it quickly by right-clicking the **Winscroller 2d Appview** system tray icon and clicking **⚙ Edit Configuration**, or start from [winscroller-2d-appview.example.toml](winscroller-2d-appview.example.toml).

---

## Complete Field Reference

| Key | Type | Default | Valid Range / Values | Description |
|---|---|---|---|---|
| `columns` | `usize` | `7` | `>= 3` | Number of app icon columns per page. The grid spans 92% of display width (up to 1800px). |
| `rows` | `usize` | `5` | `>= 2` | Number of app icon rows per page (`columns * rows` = items per page). |
| `icon_size` | `f32` | `76.0` | `64.0` – `96.0` | Base logical icon dimension in pixels. Also controls the cached PNG icon extraction resolution. |
| `show_labels` | `bool` | `true` | `true` / `false` | Shows truncated application names (`<= 18` chars) beneath each icon with contrast drop shadows. |
| `backdrop` | `String` | `"Wallpaper"` | `"Wallpaper"`, `"Centered"` | Controls full frosted wallpaper overlay vs. centered floating card mode. |
| `centered` | `bool` | `false` | `true` / `false` | When `true`, displays a rounded floating dark glass card over the dimmed desktop wallpaper. |
| `card_width` | `f32` | `0.75` | `0.40` – `0.95` | Fractional display width of the floating card when `centered = true`. |
| `card_height` | `f32` | `0.80` | `0.40` – `0.95` | Fractional display height of the floating card when `centered = true`. |
| `glow` | `bool` | `true` | `true` / `false` | Enables accent focus glow ring around the search pill when typing. |
| `accent_color` | `String` | `"#89b4fa"` | `#RRGGBB` or `#RRGGBBAA` | Hex accent tint for search focus border, blinking caret, and active pagination pill. |
| `hotkey` | `String` | `"Alt+Space"` | `"Alt+Space"` | Label for the global fallback keyboard shortcut registered via `RegisterHotKey`. |

---

## Ready-to-Use Profiles

### 1. Fullscreen macOS Launchpad (Default — 1080p / 1440p)
```toml
columns = 7
rows = 5
icon_size = 76.0
show_labels = true
backdrop = "Wallpaper"
centered = false
card_width = 0.75
card_height = 0.80
glow = true
accent_color = "#89b4fa"
hotkey = "Alt+Space"
```

### 2. Ultrawide Monitor Dense Grid (21:9 / 3440×1440)
```toml
columns = 9
rows = 5
icon_size = 84.0
show_labels = true
backdrop = "Wallpaper"
centered = false
glow = true
accent_color = "#00f5d4"
hotkey = "Alt+Space"
```

### 3. Centered Floating Glass Canvas (Spotlight / GNOME Hybrid)
```toml
columns = 6
rows = 4
icon_size = 78.0
show_labels = true
backdrop = "Centered"
centered = true
card_width = 0.72
card_height = 0.76
glow = true
accent_color = "#cba6f7"
hotkey = "Alt+Space"
```

### 4. Minimal Icon-Only Dock Grid
```toml
columns = 8
rows = 4
icon_size = 88.0
show_labels = false
backdrop = "Wallpaper"
centered = false
glow = true
accent_color = "#a6e3a1"
hotkey = "Alt+Space"
```

---

## Storage Locations

| Path | Purpose |
|---|---|
| `%APPDATA%\Winscroller2dAppview\config.toml` | Persistent user configuration file |
| `%LOCALAPPDATA%\Winscroller2dAppview\icon_cache_v1\` | Pre-normalized, unplated PNG icon cache (`{hash}_{size}.png`) |
| `%APPDATA%\Microsoft\Windows\Themes\TranscodedWallpaper` | Read-only source for dynamic desktop wallpaper blur |
