# Configuration

**Winscroller 2d Appview** reads its settings from:
```text
%APPDATA%\Winscroller2dAppview\config.toml
```

Right-click the notification tray icon and select **⚙ Edit Configuration** to open it in Notepad.

## Schema Reference

| Key | Default | Range / Values | Description |
|---|---|---|---|
| `columns` | `7` | `>= 3` | Grid columns per page |
| `rows` | `5` | `>= 2` | Grid rows per page |
| `icon_size` | `76.0` | `64.0` – `96.0` | Base icon size in pixels |
| `show_labels` | `true` | `true` / `false` | Show app names under icons |
| `backdrop` | `"Wallpaper"` | `"Wallpaper"`, `"Centered"` | Backdrop surface style |
| `centered` | `false` | `true` / `false` | Floating card (`true`) vs Fullscreen Launchpad (`false`) |
| `card_width` | `0.75` | `0.40` – `0.95` | Floating card screen width fraction |
| `card_height` | `0.80` | `0.40` – `0.95` | Floating card screen height fraction |
| `glow` | `true` | `true` / `false` | Search pill focus glow |
| `accent_color` | `"#89b4fa"` | `#RRGGBB` / `#RRGGBBAA` | Accent highlight hex color |
| `hotkey` | `"Alt+Space"` | `"Alt+Space"` | Global fallback hotkey |

## Example Profiles

### Centered Floating Card
```toml
columns = 6
rows = 4
icon_size = 78.0
centered = true
backdrop = "Centered"
card_width = 0.72
card_height = 0.76
accent_color = "#cba6f7"
```

### Ultrawide Fullscreen Launchpad
```toml
columns = 9
rows = 5
icon_size = 84.0
centered = false
backdrop = "Wallpaper"
accent_color = "#00f5d4"
```
