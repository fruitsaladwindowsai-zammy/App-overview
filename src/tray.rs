// Windows System Notification Tray with Embedded Brand Icon
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, GetSystemMetrics, LoadIconW, LoadImageW,
    PostQuitMessage, RegisterClassW, SetForegroundWindow, TrackPopupMenu, HICON, HMENU,
    IDI_APPLICATION, IMAGE_ICON, LR_SHARED, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG,
    SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
    WINDOW_EX_STYLE, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP, WM_USER,
    WNDCLASSW, WS_OVERLAPPED,
};

pub const APP_ICON_RES_ID: usize = 1;

const WM_TRAYICON: u32 = WM_USER + 101;
const IDM_OPEN: usize = 1001;
const IDM_SETTINGS: usize = 1002;
const IDM_RELOAD: usize = 1003;
const IDM_EXIT: usize = 1004;
const IDM_AUTOSTART: usize = 1005;
const IDM_PREREQS: usize = 1006;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    Toggle,
    Reload,
    Exit,
}

struct TrayState {
    sender: Sender<TrayCommand>,
    hotkey_label: String,
    config_path: PathBuf,
}

static TRAY_STATE: Mutex<Option<TrayState>> = Mutex::new(None);

/// Loads the embedded Winscroller 2d Appview brand icon (`winscroller-2d-appview.ico` resource ID 1),
/// falling back to the standard Windows application icon if unavailable.
pub fn load_brand_icon(small: bool) -> HICON {
    unsafe {
        let hinst = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let (cx, cy) = if small {
            (GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON))
        } else {
            (GetSystemMetrics(SM_CXICON), GetSystemMetrics(SM_CYICON))
        };
        if let Ok(handle) = LoadImageW(
            hinst,
            PCWSTR(APP_ICON_RES_ID as *const u16),
            IMAGE_ICON,
            cx,
            cy,
            LR_SHARED,
        ) {
            if handle.0 != 0 {
                return HICON(handle.0);
            }
        }
        LoadIconW(None, IDI_APPLICATION).unwrap_or(HICON(0))
    }
}

#[allow(dead_code)]
pub struct TrayManager {
    running: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl TrayManager {
    pub fn start(sender: Sender<TrayCommand>, hotkey_label: String, config_path: PathBuf) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        if let Ok(mut guard) = TRAY_STATE.lock() {
            *guard = Some(TrayState {
                sender,
                hotkey_label,
                config_path,
            });
        }

        thread::spawn(move || unsafe {
            let class_name: Vec<u16> = "WinscrollerTrayMsgClass\0".encode_utf16().collect();
            let window_name: Vec<u16> = "WinscrollerTrayWindow\0".encode_utf16().collect();

            unsafe extern "system" fn wnd_proc(
                hwnd: HWND,
                msg: u32,
                wparam: WPARAM,
                lparam: LPARAM,
            ) -> LRESULT {
                match msg {
                    WM_TRAYICON => {
                        let event = lparam.0 as u32;
                        if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                            if let Ok(guard) = TRAY_STATE.lock() {
                                if let Some(ref state) = *guard {
                                    let _ = state.sender.send(TrayCommand::Toggle);
                                }
                            }
                        } else if event == WM_RBUTTONUP {
                            let mut cursor = POINT::default();
                            let _ = GetCursorPos(&mut cursor);

                            let hotkey = TRAY_STATE
                                .lock()
                                .ok()
                                .and_then(|g| g.as_ref().map(|s| s.hotkey_label.clone()))
                                .unwrap_or_else(|| "Alt+Space".to_string());

                            let menu = CreatePopupMenu();
                            if let Ok(hmenu) = menu {
                                let open_title: Vec<u16> =
                                    format!("Open Winscroller 2d Appview\t{hotkey}\0")
                                        .encode_utf16()
                                        .collect();
                                let config_title: Vec<u16> = "Edit Configuration (config.toml)\0"
                                    .encode_utf16()
                                    .collect();
                                let reload_title: Vec<u16> =
                                    "Reload Apps & Config\0".encode_utf16().collect();
                                let autostart_title: Vec<u16> =
                                    "Run at Windows Startup\0".encode_utf16().collect();
                                let prereqs_title: Vec<u16> =
                                    "Check & Install Prerequisites\0".encode_utf16().collect();
                                let exit_title: Vec<u16> =
                                    "Exit Winscroller 2d Appview\0".encode_utf16().collect();

                                let autostart_flag = if crate::prereqs::is_autostart_enabled() {
                                    MF_STRING | MF_CHECKED
                                } else {
                                    MF_STRING | MF_UNCHECKED
                                };

                                let _ = AppendMenuW(
                                    hmenu,
                                    MF_STRING,
                                    IDM_OPEN,
                                    PCWSTR(open_title.as_ptr()),
                                );
                                let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
                                let _ = AppendMenuW(
                                    hmenu,
                                    MF_STRING,
                                    IDM_SETTINGS,
                                    PCWSTR(config_title.as_ptr()),
                                );
                                let _ = AppendMenuW(
                                    hmenu,
                                    MF_STRING,
                                    IDM_RELOAD,
                                    PCWSTR(reload_title.as_ptr()),
                                );
                                let _ = AppendMenuW(
                                    hmenu,
                                    autostart_flag,
                                    IDM_AUTOSTART,
                                    PCWSTR(autostart_title.as_ptr()),
                                );
                                let _ = AppendMenuW(
                                    hmenu,
                                    MF_STRING,
                                    IDM_PREREQS,
                                    PCWSTR(prereqs_title.as_ptr()),
                                );
                                let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
                                let _ = AppendMenuW(
                                    hmenu,
                                    MF_STRING,
                                    IDM_EXIT,
                                    PCWSTR(exit_title.as_ptr()),
                                );

                                let _ = SetForegroundWindow(hwnd);
                                let _ = TrackPopupMenu(
                                    hmenu,
                                    TPM_LEFTALIGN | TPM_BOTTOMALIGN,
                                    cursor.x,
                                    cursor.y,
                                    0,
                                    hwnd,
                                    None,
                                );
                                let _ = DestroyMenu(hmenu);
                            }
                        }
                        LRESULT(0)
                    }
                    WM_COMMAND => {
                        let id = wparam.0 & 0xffff;
                        if let Ok(guard) = TRAY_STATE.lock() {
                            if let Some(ref state) = *guard {
                                match id {
                                    IDM_OPEN => {
                                        let _ = state.sender.send(TrayCommand::Toggle);
                                    }
                                    IDM_SETTINGS => {
                                        let _ = std::process::Command::new("notepad.exe")
                                            .arg(&state.config_path)
                                            .spawn();
                                    }
                                    IDM_RELOAD => {
                                        let _ = state.sender.send(TrayCommand::Reload);
                                    }
                                    IDM_AUTOSTART => {
                                        let next = !crate::prereqs::is_autostart_enabled();
                                        let _ = crate::prereqs::set_autostart(next);
                                    }
                                    IDM_PREREQS => {
                                        let cfg_path = state.config_path.clone();
                                        thread::spawn(move || {
                                            let _ = crate::prereqs::ensure_prerequisites(
                                                &cfg_path, true,
                                            );
                                            let _ = std::process::Command::new("notepad.exe")
                                                .arg(crate::prereqs::prereq_log_path())
                                                .spawn();
                                        });
                                    }
                                    IDM_EXIT => {
                                        let _ = state.sender.send(TrayCommand::Exit);
                                        PostQuitMessage(0);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        LRESULT(0)
                    }
                    WM_DESTROY => {
                        PostQuitMessage(0);
                        LRESULT(0)
                    }
                    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            }

            let wc = WNDCLASSW {
                lpfnWndProc: Some(wnd_proc),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };

            let _ = RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(window_name.as_ptr()),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                HWND(0),
                HMENU(0),
                None,
                None,
            );

            let icon: HICON = load_brand_icon(true);

            let mut tip = [0u16; 128];
            let tip_str: Vec<u16> = "Winscroller 2d Appview — Active\0".encode_utf16().collect();
            for (i, c) in tip_str.iter().enumerate().take(127) {
                tip[i] = *c;
            }

            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 100,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: icon,
                szTip: tip,
                ..Default::default()
            };

            let _ = Shell_NotifyIconW(NIM_ADD, &nid);

            let mut msg = MSG::default();
            while running_clone.load(Ordering::Relaxed) {
                if GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                    let _ = DispatchMessageW(&msg);
                }
            }

            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
            let _ = DestroyWindow(hwnd);
        });

        Self { running }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
