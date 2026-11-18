// Automated Runtime Prerequisite Checker, Auto-Installer & Windows Autostart Manager
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::Config;
use crate::icon_extractor::get_cache_dir;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const AUTOSTART_REG_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const AUTOSTART_VALUE_NAME: &str = "Winscroller2dAppview";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrereqStatus {
    Present,
    Installed,
    Warning(String),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrereqCheck {
    pub name: String,
    pub status: PrereqStatus,
    pub details: String,
}

#[derive(Clone, Debug)]
pub struct PrereqReport {
    pub checks: Vec<PrereqCheck>,
    pub all_ready: bool,
}

impl PrereqReport {
    pub fn from_checks(checks: Vec<PrereqCheck>) -> Self {
        let all_ready = checks
            .iter()
            .all(|c| !matches!(c.status, PrereqStatus::Failed(_)));
        Self { checks, all_ready }
    }

    pub fn format_summary(&self) -> String {
        let mut out = String::from("Winscroller 2d Appview — Prerequisite Check Report\n");
        out.push_str("==================================================\n");
        for check in &self.checks {
            let badge = match &check.status {
                PrereqStatus::Present => "[OK]",
                PrereqStatus::Installed => "[INSTALLED]",
                PrereqStatus::Warning(_) => "[WARN]",
                PrereqStatus::Failed(_) => "[FAIL]",
            };
            out.push_str(&format!("{badge:<12} {} — {}\n", check.name, check.details));
            match &check.status {
                PrereqStatus::Warning(msg) | PrereqStatus::Failed(msg) => {
                    out.push_str(&format!("             Note: {msg}\n"));
                }
                _ => {}
            }
        }
        out.push_str(&format!(
            "Overall status: {}\n",
            if self.all_ready {
                "READY"
            } else {
                "ACTION REQUIRED"
            }
        ));
        out
    }
}

fn app_local_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Winscroller2dAppview");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn prereq_stamp_path() -> PathBuf {
    app_local_dir().join("prereqs_ok_v1.txt")
}

pub fn prereq_log_path() -> PathBuf {
    app_local_dir().join("prereqs.log")
}

fn system32_dir() -> PathBuf {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    PathBuf::from(root).join("System32")
}

fn has_vc_runtime(sys32: &Path) -> bool {
    sys32.join("vcruntime140.dll").exists() && sys32.join("msvcp140.dll").exists()
}

fn has_opengl_runtime(sys32: &Path) -> bool {
    sys32.join("opengl32.dll").exists()
}

fn run_hidden(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .is_ok_and(|s| s.success())
}

fn install_vc_redist() -> bool {
    // 1. Primary path: silent winget install
    if run_hidden(
        "winget",
        &[
            "install",
            "--id",
            "Microsoft.VCRedist.2015+.x64",
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ],
    ) {
        return true;
    }

    // 2. Fallback path: official Microsoft VC++ 2015-2022 x64 redistributable
    let ps_cmd = r"$out = Join-Path $env:TEMP 'vc_redist.x64.exe'; Invoke-WebRequest -Uri 'https://aka.ms/vs/17/release/vc_redist.x64.exe' -OutFile $out -UseBasicParsing; Start-Process -FilePath $out -ArgumentList '/install','/quiet','/norestart' -Wait";
    run_hidden(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", ps_cmd],
    )
}

fn install_opengl_compat_pack() -> bool {
    run_hidden(
        "winget",
        &[
            "install",
            "--id",
            "9NQPSL29BFFF",
            "--source",
            "msstore",
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ],
    )
}

fn check_precision_touchpad() -> bool {
    Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\PrecisionTouchPad\Status",
            "/v",
            "Enabled",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .is_ok_and(|out| {
            out.status.success() && String::from_utf8_lossy(&out.stdout).contains("0x1")
        })
}

/// Verifies all runtime prerequisites on startup and automatically installs any missing
/// Windows redistributables. Uses a fast-path stamp after the first clean run so subsequent
/// launches have near-zero overhead. Pass `force = true` to run every check unconditionally.
pub fn ensure_prerequisites(config_path: &Path, force: bool) -> PrereqReport {
    let sys32 = system32_dir();
    let stamp = prereq_stamp_path();

    // Fast path: already verified previously and core DLLs + config still exist
    if !force
        && stamp.exists()
        && has_vc_runtime(&sys32)
        && has_opengl_runtime(&sys32)
        && config_path.exists()
    {
        return PrereqReport::from_checks(vec![PrereqCheck {
            name: "Fast-Path Verification".to_string(),
            status: PrereqStatus::Present,
            details: "Previously verified prerequisites and system runtimes present".to_string(),
        }]);
    }

    let mut checks = Vec::new();

    // 1. App directories & default configuration
    let cache_dir = get_cache_dir();
    let cfg_ready = if config_path.exists() {
        true
    } else {
        Config::default().save_to(config_path).is_ok()
    };
    checks.push(PrereqCheck {
        name: "App Config & Icon Cache".to_string(),
        status: if cfg_ready && cache_dir.exists() {
            PrereqStatus::Present
        } else {
            PrereqStatus::Failed("Could not initialize AppData directories".to_string())
        },
        details: format!(
            "config={}, cache={}",
            config_path.display(),
            cache_dir.display()
        ),
    });

    // 2. Visual C++ 2015-2022 x64 Redistributable
    if has_vc_runtime(&sys32) {
        checks.push(PrereqCheck {
            name: "Visual C++ x64 Runtime".to_string(),
            status: PrereqStatus::Present,
            details: "vcruntime140.dll & msvcp140.dll available".to_string(),
        });
    } else {
        let installed = install_vc_redist() && has_vc_runtime(&sys32);
        checks.push(PrereqCheck {
            name: "Visual C++ x64 Runtime".to_string(),
            status: if installed {
                PrereqStatus::Installed
            } else {
                PrereqStatus::Warning(
                    "Auto-install did not detect vcruntime140.dll; install Microsoft.VCRedist.2015+.x64 manually if needed".to_string(),
                )
            },
            details: if installed {
                "Automatically installed Microsoft.VCRedist.2015+.x64".to_string()
            } else {
                "Attempted silent install via winget / Microsoft installer".to_string()
            },
        });
    }

    // 3. OpenGL GPU Runtime
    if has_opengl_runtime(&sys32) {
        checks.push(PrereqCheck {
            name: "OpenGL Graphics Runtime".to_string(),
            status: PrereqStatus::Present,
            details: "opengl32.dll available".to_string(),
        });
    } else {
        let installed = install_opengl_compat_pack() && has_opengl_runtime(&sys32);
        checks.push(PrereqCheck {
            name: "OpenGL Graphics Runtime".to_string(),
            status: if installed {
                PrereqStatus::Installed
            } else {
                PrereqStatus::Failed("opengl32.dll missing; update GPU drivers".to_string())
            },
            details: "Required for hardware-accelerated egui rendering".to_string(),
        });
    }

    // 4. Windows PowerShell (for UWP/MSIX Get-StartApps discovery)
    let ps_ok = run_hidden(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$PSVersionTable.PSVersion.Major",
        ],
    );
    checks.push(PrereqCheck {
        name: "Windows PowerShell".to_string(),
        status: if ps_ok {
            PrereqStatus::Present
        } else {
            PrereqStatus::Warning(
                "UWP app scanning will fall back to Start Menu .lnk shortcuts".to_string(),
            )
        },
        details: "Used for UWP/MSIX Get-StartApps discovery".to_string(),
    });

    // 5. Windows Precision Touchpad status (informational)
    let ptp = check_precision_touchpad();
    checks.push(PrereqCheck {
        name: "Windows Precision Touchpad".to_string(),
        status: if ptp {
            PrereqStatus::Present
        } else {
            PrereqStatus::Warning(
                "Precision Touchpad not detected; Alt+Space hotkey & mouse gestures active"
                    .to_string(),
            )
        },
        details: if ptp {
            "Precision Touchpad active (4-finger & 2-finger gestures ready)".to_string()
        } else {
            "Desktop / non-PTP input mode".to_string()
        },
    });

    let report = PrereqReport::from_checks(checks);
    let summary = report.format_summary();
    let _ = fs::write(prereq_log_path(), &summary);
    if report.all_ready {
        let _ = fs::write(&stamp, "ok\n");
    }
    report
}

/// Returns `true` if Winscroller 2d Appview is registered to start automatically at Windows logon.
pub fn is_autostart_enabled() -> bool {
    Command::new("reg")
        .args(["query", AUTOSTART_REG_KEY, "/v", AUTOSTART_VALUE_NAME])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Enables or disables automatic startup at Windows logon via `HKCU\...\Run`.
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let quoted = format!("\"{}\"", exe.display());
        let ok = run_hidden(
            "reg",
            &[
                "add",
                AUTOSTART_REG_KEY,
                "/v",
                AUTOSTART_VALUE_NAME,
                "/t",
                "REG_SZ",
                "/d",
                &quoted,
                "/f",
            ],
        );
        if ok {
            Ok(())
        } else {
            Err("Failed to register Windows Run startup key".to_string())
        }
    } else {
        let ok = run_hidden(
            "reg",
            &[
                "delete",
                AUTOSTART_REG_KEY,
                "/v",
                AUTOSTART_VALUE_NAME,
                "/f",
            ],
        );
        if ok || !is_autostart_enabled() {
            Ok(())
        } else {
            Err("Failed to remove Windows Run startup key".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_ready_with_present_installed_and_warnings() {
        let report = PrereqReport::from_checks(vec![
            PrereqCheck {
                name: "VC++".to_string(),
                status: PrereqStatus::Installed,
                details: "Installed silently".to_string(),
            },
            PrereqCheck {
                name: "Touchpad".to_string(),
                status: PrereqStatus::Warning("Desktop mode".to_string()),
                details: "Mouse fallback".to_string(),
            },
        ]);
        assert!(report.all_ready);
        let text = report.format_summary();
        assert!(text.contains("[INSTALLED]"));
        assert!(text.contains("[WARN]"));
        assert!(text.contains("READY"));
    }

    #[test]
    fn test_report_not_ready_when_any_check_fails() {
        let report = PrereqReport::from_checks(vec![PrereqCheck {
            name: "OpenGL".to_string(),
            status: PrereqStatus::Failed("Missing opengl32.dll".to_string()),
            details: "GPU driver needed".to_string(),
        }]);
        assert!(!report.all_ready);
        assert!(report.format_summary().contains("ACTION REQUIRED"));
    }
}
