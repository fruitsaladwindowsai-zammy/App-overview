# Contributing to Winscroller 2d Appview

Thank you for your interest in contributing to **Winscroller 2d Appview**! Every bug report, gesture improvement, documentation tweak, and pull request is welcome.

---

## Development Setup

### Prerequisites
- Windows 10 (1809+) or Windows 11 (x86_64)
- Rust stable toolchain (`rustup` — MSVC or GNU + `rust-lld`)
- Optional: `mdbook` (`cargo install mdbook`) for previewing the `docs/` book locally

### Local Verification Commands
Before opening a pull request, please verify all checks pass cleanly:

```powershell
# 1. Check formatting
cargo fmt -- --check

# 2. Run Clippy lints
cargo clippy --all-targets

# 3. Run unit tests (fuzzy scoring, pagination, grid navigation)
cargo test

# 4. Verify optimized release build
cargo build --release
```

---

## Project Layout

| Path | Purpose |
|---|---|
| `src/main.rs` | Binary entry point, viewport setup (`with_taskbar(false)`), channel wiring |
| `src/ui.rs` | `eframe::App` renderer, spring physics, continuous multi-page swipe, borderless halo |
| `src/hotkey.rs` | `WH_KEYBOARD_LL` hook, `SetWinEventHook` Task View suppressor, `RegisterHotKey` |
| `src/icon_extractor.rs` | Shell COM `IShellItemImageFactory`, GDI alpha repair, auto-crop, Lanczos3 cache |
| `src/wallpaper.rs` | `TranscodedWallpaper` 2-pass separable box blur + acrylic vignette |
| `src/app_scanner.rs` | `Get-StartApps` UWP/MSIX + Start Menu `.lnk` scanner & clutter filter |
| `src/model.rs` | Alfred-style search scoring, pagination arithmetic, and unit tests |
| `src/launcher.rs` | `explorer.exe shell:AppsFolder\...` and Win32 `ShellExecuteW` activation |
| `src/tray.rs` | Native Win32 `Shell_NotifyIconW` system tray icon & context menu |
| `src/config.rs` | TOML config persistence |
| `docs/` | mdBook documentation site |
| `wiki/` | GitHub Wiki pages |
| `website/` | Interactive showcase website & live simulator |

---

## Coding Guidelines

- **Zero Unsafe Leaks**: Every Win32 GDI handle (`HBITMAP`, `HDC`), hook (`HHOOK`, `HWINEVENTHOOK`), and COM session (`CoInitializeEx` / `CoUninitialize`) must be paired with proper cleanup.
- **Unmultiplied Color Safety**: Always use `egui::Color32::from_rgba_unmultiplied` (never `from_rgba_premultiplied` with unmultiplied RGB values) to avoid additive white clamping.
- **Keep Idle Footprint Minimal**: When `is_visible == false`, avoid high-frequency repaints (`request_repaint_after(100ms)`).
- **Update Docs**: Any new config field, gesture, or shortcut should update `KEYBINDINGS.md`, `CONFIGURATION.md`, `winscroller-2d-appview.example.toml`, `docs/src/`, and `CHANGELOG.md`.

---

## Pull Request Process
1. Fork the repository and create a feature branch (`git checkout -b feat/my-feature`).
2. Keep commits focused and descriptive.
3. Run the verification checklist above.
4. Open a pull request using the repository PR template.
