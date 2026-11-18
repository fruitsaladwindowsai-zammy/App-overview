# Configuration Guide

Config file path: `%APPDATA%\Winscroller2dAppview\config.toml`  
Quick access: **Right-click Tray Icon → ⚙ Edit Configuration**

```toml
columns = 7
rows = 5
icon_size = 76.0
show_labels = true
backdrop = "Wallpaper"    # "Wallpaper" or "Centered"
centered = false          # true = Floating glass card, false = Fullscreen Launchpad
card_width = 0.75
card_height = 0.80
glow = true
accent_color = "#89b4fa"
hotkey = "Alt+Space"
```

| Field | Range / Notes |
|---|---|
| `columns`, `rows` | Minimum `3×2`. Use `9×5` for ultrawide displays or `6×4` for centered card mode. |
| `icon_size` | Clamped to `64.0..=96.0`. Hovered icons scale `1.10x`. |
| `accent_color` | Accepts `#RRGGBB` or `#RRGGBBAA` hex strings. |
