// Application launcher for Windows
use std::os::windows::process::CommandExt;
use std::process::Command;
use windows::core::HSTRING;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn launch_app(target: &str) -> bool {
    if target.starts_with("shell:AppsFolder\\") {
        // Use explorer.exe to activate the UWP or desktop app from AppsFolder
        let status = Command::new("explorer.exe")
            .arg(target)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
        return status.is_ok();
    }

    // Classic ShellExecuteW for .lnk or .exe files
    unsafe {
        let op = HSTRING::from("open");
        let file = HSTRING::from(target);

        let result = ShellExecuteW(
            HWND(0),
            &op,
            &file,
            None,
            None,
            SW_SHOWNORMAL,
        );

        // ShellExecute returns an HINSTANCE > 32 on success
        (result.0 as usize) > 32
    }
}
