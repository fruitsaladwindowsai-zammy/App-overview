// Winscroller 2d Appview - Native Launchpad-style Application Launcher for Windows
#![windows_subsystem = "windows"]

mod app_scanner;
mod config;
mod hotkey;
mod icon_extractor;
mod launcher;
mod model;
mod tray;
mod ui;
mod wallpaper;

use std::sync::mpsc::channel;
use config::Config;
use ui::SavannaApp;

fn main() -> eframe::Result<()> {
    let config = Config::load();

    // 1. Initialize global hotkey and 4-finger touchpad gesture listener
    let (hotkey_tx, hotkey_rx) = channel();
    let _hotkey_listener = hotkey::GlobalHotkeyListener::start(move |cmd| {
        let _ = hotkey_tx.send(cmd);
    });

    // 2. Initialize notification tray icon manager
    let (tray_tx, tray_rx) = channel();
    let _tray_manager = tray::TrayManager::start(tray_tx);

    // 3. Viewport setup: borderless, transparent, maximized, no taskbar button
    let viewport = egui::ViewportBuilder::default()
        .with_title("Winscroller 2d Appview")
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_maximized(true)
        .with_taskbar(false);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Winscroller 2d Appview",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(SavannaApp::new(cc, config, tray_rx, hotkey_rx)))
        }),
    )
}
