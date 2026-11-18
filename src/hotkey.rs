// Winscroller 2d Appview - Precision Touchpad Gesture and Global Hotkey Engine
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, GetAsyncKeyState, RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS,
    KEYEVENTF_KEYUP, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, VK_ESCAPE, VK_LWIN,
    VK_RWIN, VK_SPACE, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetClassNameW, GetMessageW, PostMessageW, SetWindowsHookExW,
    UnhookWindowsHookEx, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND, HHOOK, KBDLLHOOKSTRUCT, MSG,
    WH_KEYBOARD_LL, WINEVENT_OUTOFCONTEXT, WM_CLOSE, WM_HOTKEY, WM_KEYDOWN, WM_SYSKEYDOWN,
};

#[allow(clippy::upper_case_acronyms)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HWINEVENTHOOK(pub isize);

type WinEventProc = Option<
    unsafe extern "system" fn(
        h_hook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        id_object: i32,
        id_child: i32,
        id_thread: u32,
        event_time: u32,
    ),
>;

#[link(name = "user32")]
extern "system" {
    fn SetWinEventHook(
        event_min: u32,
        event_max: u32,
        hmod_proc: HMODULE,
        lpfn_proc: WinEventProc,
        id_process: u32,
        id_thread: u32,
        dw_flags: u32,
    ) -> HWINEVENTHOOK;

    fn UnhookWinEvent(h_hook: HWINEVENTHOOK) -> windows::Win32::Foundation::BOOL;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureCommand {
    Toggle,
    Dismiss,
}

type GestureCallback = Box<dyn Fn(GestureCommand) + Send + 'static>;
static GESTURE_CALLBACK: Mutex<Option<GestureCallback>> = Mutex::new(None);
pub static IS_APP_VISIBLE: AtomicBool = AtomicBool::new(true);

/// Parses a human-readable hotkey string (e.g. `"Alt+Space"`, `"Ctrl+Shift+A"`, `"Win+F2"`)
/// into Win32 `RegisterHotKey` modifier flags and virtual key code.
/// Falls back to `Alt+Space` if the string cannot be parsed.
pub fn parse_hotkey(spec: &str) -> (HOT_KEY_MODIFIERS, u32) {
    let mut mods = MOD_NOREPEAT;
    let mut vk: Option<u32> = None;

    for raw_part in spec.split('+') {
        let part = raw_part.trim().to_ascii_lowercase();
        match part.as_str() {
            "alt" => mods |= MOD_ALT,
            "ctrl" | "control" => mods |= MOD_CONTROL,
            "shift" => mods |= MOD_SHIFT,
            "win" | "super" | "meta" => mods |= MOD_WIN,
            "space" => vk = Some(VK_SPACE.0 as u32),
            "tab" => vk = Some(VK_TAB.0 as u32),
            s if s.len() == 1 => {
                let ch = s.as_bytes()[0].to_ascii_uppercase();
                if ch.is_ascii_alphanumeric() {
                    vk = Some(ch as u32);
                }
            }
            s if s.starts_with('f') && s.len() <= 3 => {
                if let Ok(num) = s[1..].parse::<u32>() {
                    if (1..=12).contains(&num) {
                        vk = Some(0x70 + (num - 1)); // VK_F1..VK_F12
                    }
                }
            }
            _ => {}
        }
    }

    match vk {
        Some(code) if mods != MOD_NOREPEAT => (mods, code),
        _ => (MOD_ALT | MOD_NOREPEAT, VK_SPACE.0 as u32),
    }
}

fn notify_gesture(cmd: GestureCommand) {
    if let Ok(guard) = GESTURE_CALLBACK.lock() {
        if let Some(ref cb) = *guard {
            cb(cmd);
        }
    }
}

// 1. Low-Level Keyboard Hook: Intercepts Windows Precision Touchpad gesture injections
unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 && (wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN) {
        let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let win_pressed =
            GetAsyncKeyState(VK_LWIN.0 as i32) < 0 || GetAsyncKeyState(VK_RWIN.0 as i32) < 0;
        let ctrl_pressed = GetAsyncKeyState(0x11 /* VK_CONTROL */) < 0;

        // Windows 10/11 Precision Touchpad: 4-finger swipe up generates Win + Tab (or Ctrl + Win + Up)
        if (kbd.vkCode == VK_TAB.0 as u32 && win_pressed)
            || (kbd.vkCode == 0x26 /* VK_UP */ && win_pressed && ctrl_pressed)
        {
            notify_gesture(GestureCommand::Toggle);
            return LRESULT(1); // Suppress Windows Task View so Winscroller 2d Appview appears seamlessly!
        }

        // Windows 10/11 Precision Touchpad: 4-finger swipe down generates Win + D (or Ctrl + Win + Down)
        if ((kbd.vkCode == 0x44 /* 'D' */ && win_pressed)
            || (kbd.vkCode == 0x28 /* VK_DOWN */ && win_pressed && ctrl_pressed))
            && IS_APP_VISIBLE.load(Ordering::Relaxed)
        {
            notify_gesture(GestureCommand::Dismiss);
            return LRESULT(1); // Suppress Desktop minimization while Winscroller dismisses
        }
    }

    CallNextHookEx(HHOOK(0), code, wparam, lparam)
}

// 2. WinEvent Hook: Intercepts Windows Task View / App View if activated directly by DWM
unsafe extern "system" fn task_view_interceptor_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if hwnd.0 == 0 || id_object != 0 {
        return;
    }

    let mut buf = [0u16; 128];
    let len = GetClassNameW(hwnd, &mut buf);
    if len > 0 {
        let class = String::from_utf16_lossy(&buf[..len as usize]);
        // Detect Windows Task View window classes
        if class.contains("MultitaskingViewBackgroundHost")
            || class.contains("XamlExplorerHostIslandWindow")
        {
            // Dismiss Windows Task View immediately
            let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            keybd_event(VK_ESCAPE.0 as u8, 0, Default::default(), 0);
            keybd_event(VK_ESCAPE.0 as u8, 0, KEYEVENTF_KEYUP, 0);

            // Open Winscroller 2d Appview
            notify_gesture(GestureCommand::Toggle);
        }
    }
}

#[allow(dead_code)]
pub struct GlobalHotkeyListener {
    running: Arc<AtomicBool>,
}

impl GlobalHotkeyListener {
    pub fn start<F>(hotkey_spec: &str, callback: F) -> Self
    where
        F: Fn(GestureCommand) + Send + 'static,
    {
        if let Ok(mut guard) = GESTURE_CALLBACK.lock() {
            *guard = Some(Box::new(callback));
        }

        let (modifiers, vk) = parse_hotkey(hotkey_spec);
        let hotkey_label = hotkey_spec.to_string();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        thread::spawn(move || unsafe {
            let hotkey_id = 1;

            let registered = RegisterHotKey(HWND(0), hotkey_id, modifiers, vk);
            if !registered.as_bool() {
                eprintln!(
                    "Warning: Could not register {} hotkey (may already be in use)",
                    hotkey_label
                );
            }

            // Get valid module handle so global hook succeeds on Windows
            let hmod = GetModuleHandleW(PCWSTR::null()).unwrap_or(HMODULE(0));

            // Install low-level keyboard hook to capture 4-finger touchpad gestures & suppress Task View
            let kbd_hook =
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), hmod, 0);

            // Install WinEvent hook to instantly intercept Task View window if DWM creates it
            let event_hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_OBJECT_SHOW,
                HMODULE(0),
                Some(task_view_interceptor_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            );

            let mut msg = MSG::default();
            while running_clone.load(Ordering::Relaxed) {
                if GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                    if msg.message == WM_HOTKEY && msg.wParam == WPARAM(hotkey_id as usize) {
                        notify_gesture(GestureCommand::Toggle);
                    }
                    let _ = DispatchMessageW(&msg);
                }
            }

            if event_hook.0 != 0 {
                let _ = UnhookWinEvent(event_hook);
            }
            if let Ok(h) = kbd_hook {
                let _ = UnhookWindowsHookEx(h);
            }
            let _ = UnregisterHotKey(HWND(0), hotkey_id);
        });

        Self { running }
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_alt_space() {
        let (mods, vk) = parse_hotkey("Alt+Space");
        assert_eq!(mods, MOD_ALT | MOD_NOREPEAT);
        assert_eq!(vk, VK_SPACE.0 as u32);
    }

    #[test]
    fn test_parse_ctrl_shift_letter_and_fn_keys() {
        let (mods, vk) = parse_hotkey("Ctrl+Shift+A");
        assert_eq!(mods, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT);
        assert_eq!(vk, b'A' as u32);

        let (mods_f, vk_f) = parse_hotkey("Win+F2");
        assert_eq!(mods_f, MOD_WIN | MOD_NOREPEAT);
        assert_eq!(vk_f, 0x71);
    }

    #[test]
    fn test_parse_invalid_falls_back_to_alt_space() {
        let (mods, vk) = parse_hotkey("invalid");
        assert_eq!(mods, MOD_ALT | MOD_NOREPEAT);
        assert_eq!(vk, VK_SPACE.0 as u32);
    }
}
