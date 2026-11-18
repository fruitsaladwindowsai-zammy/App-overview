// Winscroller 2d Appview - Native Launchpad-style Application Launcher for Windows
#![windows_subsystem = "windows"]

mod app_scanner;
mod config;
mod hotkey;
mod icon_extractor;
mod launcher;
mod model;
mod prereqs;
mod tray;
mod ui;
mod wallpaper;

use std::path::PathBuf;
use std::sync::mpsc::channel;

use config::Config;
use ui::SavannaApp;

const BRAND_LOGO_PNG: &[u8] = include_bytes!("../docs/assets/logo-256.png");

#[link(name = "kernel32")]
extern "system" {
    fn AttachConsole(dw_process_id: u32) -> i32;
}
const ATTACH_PARENT_PROCESS: u32 = u32::MAX;

fn attach_parent_console() {
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn load_viewport_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(BRAND_LOGO_PNG).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    })
}

fn print_help() {
    println!(
        "Winscroller 2d Appview v{}\n\
         Native Launchpad-style 2D Application Overview for Windows 10/11\n\n\
         USAGE:\n\
         \twinscroller-2d-appview [OPTIONS]\n\n\
         OPTIONS:\n\
         \t-c, --config <PATH>        Use a custom config.toml file path\n\
         \t    --init-config          Write default config.toml and exit\n\
         \t    --validate             Validate config.toml and exit\n\
         \t    --check-prereqs        Verify & auto-install runtime prerequisites and exit\n\
         \t    --install-autostart    Enable automatic startup at Windows logon\n\
         \t    --remove-autostart     Disable automatic startup at Windows logon\n\
         \t-V, --version              Print version information and exit\n\
         \t-h, --help                 Print this help message and exit",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut config_path = Config::config_path();
    let mut do_init_config = false;
    let mut do_validate = false;
    let mut do_check_prereqs = false;
    let mut autostart_action: Option<bool> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-V" | "--version" => {
                attach_parent_console();
                println!("winscroller-2d-appview {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "-h" | "--help" => {
                attach_parent_console();
                print_help();
                return Ok(());
            }
            "--init-config" => do_init_config = true,
            "--validate" => do_validate = true,
            "--check-prereqs" => do_check_prereqs = true,
            "--install-autostart" => autostart_action = Some(true),
            "--remove-autostart" => autostart_action = Some(false),
            "-c" | "--config" => {
                if let Some(next) = args.get(i + 1) {
                    config_path = PathBuf::from(next);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(enable) = autostart_action {
        attach_parent_console();
        match prereqs::set_autostart(enable) {
            Ok(()) => println!(
                "Windows Startup autostart {}",
                if enable { "enabled" } else { "disabled" }
            ),
            Err(err) => {
                eprintln!("Autostart update failed: {err}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if do_check_prereqs {
        attach_parent_console();
        let report = prereqs::ensure_prerequisites(&config_path, true);
        print!("{}", report.format_summary());
        if !report.all_ready {
            std::process::exit(1);
        }
        return Ok(());
    }

    if do_init_config {
        attach_parent_console();
        let cfg = Config::default();
        match cfg.save_to(&config_path) {
            Ok(()) => println!("Initialized default config at: {}", config_path.display()),
            Err(err) => {
                eprintln!("Failed to write config: {err}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Automatic runtime prerequisite check & self-healing installation on startup
    let _prereq_report = prereqs::ensure_prerequisites(&config_path, false);

    let config = Config::load_from(&config_path);

    if do_validate {
        attach_parent_console();
        match config.validate() {
            Ok(()) => {
                println!("Config OK: {}", config_path.display());
                return Ok(());
            }
            Err(errs) => {
                eprintln!("Config validation failed ({}):", config_path.display());
                for e in errs {
                    eprintln!("  - {e}");
                }
                std::process::exit(1);
            }
        }
    }

    // 1. Initialize global hotkey and 4-finger touchpad gesture listener
    let (hotkey_tx, hotkey_rx) = channel();
    let _hotkey_listener = hotkey::GlobalHotkeyListener::start(&config.hotkey, move |cmd| {
        let _ = hotkey_tx.send(cmd);
    });

    // 2. Initialize notification tray icon manager with embedded brand icon
    let (tray_tx, tray_rx) = channel();
    let _tray_manager =
        tray::TrayManager::start(tray_tx, config.hotkey.clone(), config_path.clone());

    // 3. Viewport setup: borderless, transparent, maximized, no taskbar button, brand icon
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Winscroller 2d Appview")
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_maximized(true)
        .with_taskbar(false);

    if let Some(icon) = load_viewport_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Winscroller 2d Appview",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(SavannaApp::new(
                cc,
                config,
                config_path,
                tray_rx,
                hotkey_rx,
            )))
        }),
    )
}
