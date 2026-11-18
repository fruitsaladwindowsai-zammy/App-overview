// App scanner for Windows installed applications
use serde::Deserialize;
use std::collections::HashSet;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

use crate::model::AppEntry;

#[derive(Deserialize, Debug)]
struct StartAppRaw {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "AppID")]
    app_id: String,
}

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn scan_installed_apps() -> Vec<AppEntry> {
    let mut apps: Vec<AppEntry> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut seen_names: HashSet<String> = HashSet::new();

    // 1. Scan via Get-StartApps (covers UWP, MSIX, and registered desktop apps)
    if let Ok(output) = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-StartApps | ConvertTo-Json -Compress",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        if output.status.success() {
            let json_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(parsed) = serde_json::from_str::<Vec<StartAppRaw>>(&json_str) {
                for item in parsed {
                    if should_filter_app(&item.name, &item.app_id) {
                        continue;
                    }

                    let norm_name = item.name.trim().to_lowercase();
                    if seen_names.contains(&norm_name) {
                        continue;
                    }
                    seen_names.insert(norm_name);
                    seen_ids.insert(item.app_id.clone());

                    let is_uwp = item.app_id.contains('!') || item.app_id.starts_with("Microsoft.");
                    let target = format!("shell:AppsFolder\\{}", item.app_id);

                    apps.push(AppEntry {
                        id: item.app_id.clone(),
                        name: item.name.trim().to_string(),
                        target,
                        is_uwp,
                        keywords: vec![item.name.clone()],
                    });
                }
            }
        }
    }

    // 2. Scan Start Menu .lnk shortcuts (User + All Users)
    let start_menu_dirs = get_start_menu_directories();
    for dir in start_menu_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let norm_name = stem.trim().to_lowercase();
                    if should_filter_app(stem, &path.to_string_lossy()) {
                        continue;
                    }

                    if seen_names.contains(&norm_name) {
                        continue;
                    }
                    seen_names.insert(norm_name);

                    let path_str = path.to_string_lossy().to_string();
                    apps.push(AppEntry {
                        id: path_str.clone(),
                        name: stem.trim().to_string(),
                        target: path_str,
                        is_uwp: false,
                        keywords: vec![stem.to_string()],
                    });
                }
            }
        }
    }

    // Sort alphabetically by name
    apps.sort_by_key(|a| a.name.to_lowercase());

    apps
}

fn should_filter_app(name: &str, target_or_id: &str) -> bool {
    let lower_name = name.to_lowercase();
    let lower_target = target_or_id.to_lowercase();

    // Filter administrative tools / msc console scripts / installers
    if lower_target.ends_with(".msc") {
        return true;
    }

    // Filter uninstaller and diagnostic clutter
    let blocked_exact = [
        "administrative tools",
        "dfrgui",
        "character map",
        "disk cleanup",
        "event viewer",
        "computer management",
        "local security policy",
        "performance monitor",
        "resource monitor",
        "task scheduler",
        "services",
        "system information",
        "windows tools",
        "click to do",
    ];

    for blocked in blocked_exact {
        if lower_name == blocked {
            return true;
        }
    }

    let blocked_keywords = [
        "uninstall",
        "remove ",
        "documentation",
        "readme",
        "help",
        "manual",
        "release notes",
        "license",
        "configuration tool",
        "setup wizard",
        "iscsi initiator",
        "odbc data",
        "component services",
        "system configuration",
        "defragment",
        "recovery drive",
        "memory diagnostic",
        "private browsing",
        "guest mode",
        "diagnostics",
    ];

    for blocked in blocked_keywords {
        if lower_name.contains(blocked) {
            return true;
        }
    }

    false
}

fn get_start_menu_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User Start Menu
    if let Some(app_data) = dirs::data_dir() {
        dirs.push(app_data.join("Microsoft\\Windows\\Start Menu\\Programs"));
    }

    // System Start Menu
    if let Ok(program_data) = std::env::var("ProgramData") {
        dirs.push(PathBuf::from(program_data).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }

    dirs
}
