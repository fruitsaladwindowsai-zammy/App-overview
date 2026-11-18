// Windows System Notification Tray
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, LoadIconW, PostQuitMessage, RegisterClassW,
    SetForegroundWindow, TrackPopupMenu, HICON, HMENU, IDI_APPLICATION, MF_SEPARATOR, MF_STRING,
    MSG, TPM_BOTTOMALIGN, TPM_LEFTALIGN, WINDOW_EX_STYLE, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK,
    WM_LBUTTONUP, WM_RBUTTONUP, WM_USER, WNDCLASSW, WS_OVERLAPPED,
};

const WM_TRAYICON: u32 = WM_USER + 101;
const IDM_OPEN: usize = 1001;
const IDM_SETTINGS: usize = 1002;
const IDM_RELOAD: usize = 1003;
const IDM_EXIT: usize = 1004;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    Toggle,
    Reload,
    Exit,
}

#[allow(dead_code)]
pub struct TrayManager {
    running: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl TrayManager {
    pub fn start(sender: Sender<TrayCommand>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        thread::spawn(move || unsafe {
            let class_name: Vec<u16> = "WinscrollerTrayMsgClass\0".encode_utf16().collect();
            let window_name: Vec<u16> = "WinscrollerTrayWindow\0".encode_utf16().collect();

            // Set up static callback storage for WndProc
            static mut GLOBAL_SENDER: Option<Sender<TrayCommand>> = None;
            GLOBAL_SENDER = Some(sender);

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
                            if let Some(ref s) = GLOBAL_SENDER {
                                let _ = s.send(TrayCommand::Toggle);
                            }
                        } else if event == WM_RBUTTONUP {
                            let mut cursor = POINT::default();
                            let _ = GetCursorPos(&mut cursor);

                            let menu = CreatePopupMenu();
                            if let Ok(hmenu) = menu {
                                let open_title: Vec<u16> =
                                    "🚀 Open Winscroller 2d Appview\tAlt+Space\0"
                                        .encode_utf16()
                                        .collect();
                                let config_title: Vec<u16> =
                                    "⚙ Edit Configuration\0".encode_utf16().collect();
                                let reload_title: Vec<u16> =
                                    "🔄 Reload Installed Apps\0".encode_utf16().collect();
                                let exit_title: Vec<u16> =
                                    "❌ Exit Winscroller 2d Appview\0".encode_utf16().collect();

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
                        if let Some(ref s) = GLOBAL_SENDER {
                            match id {
                                IDM_OPEN => {
                                    let _ = s.send(TrayCommand::Toggle);
                                }
                                IDM_SETTINGS => {
                                    let cfg_path = crate::config::Config::config_path();
                                    let _ = std::process::Command::new("notepad.exe")
                                        .arg(cfg_path)
                                        .spawn();
                                }
                                IDM_RELOAD => {
                                    let _ = s.send(TrayCommand::Reload);
                                }
                                IDM_EXIT => {
                                    let _ = s.send(TrayCommand::Exit);
                                    PostQuitMessage(0);
                                }
                                _ => {}
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

            let icon: HICON = LoadIconW(None, IDI_APPLICATION).unwrap_or(HICON(0));

            let mut tip = [0u16; 128];
            let tip_str: Vec<u16> = "Winscroller 2d Appview\0".encode_utf16().collect();
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
