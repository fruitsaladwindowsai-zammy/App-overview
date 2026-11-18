# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

---

## Reporting a Vulnerability

Security issues and vulnerabilities should be reported responsibly. Please do **not** open public GitHub issues for security vulnerabilities.

### Reporting Process
1. Open a private security advisory on GitHub via **Security** > **Advisories** > **Report a vulnerability**.
2. Include:
   * Description of the vulnerability and impact.
   * Minimal reproduction steps or proof-of-concept (PoC).
   * Your Windows build version (e.g., Windows 11 23H2 / 24H2).
3. We will acknowledge receipt of your report within 48 hours and work on a coordinated fix and advisory.

---

## Hooks, Privacy & Integrity Model

- **Zero Telemetry / Zero Network**: **Winscroller 2d Appview** performs zero network connections, listens on no TCP/UDP ports, and transmits no analytics or keystrokes.
- **Scoped Keyboard Hook (`WH_KEYBOARD_LL`)**: The low-level hook in `src/hotkey.rs` inspects only the exact virtual key codes needed to intercept Windows Precision Touchpad 4-finger gestures (`Win+Tab`, `Ctrl+Win+Up`, `Win+D`, `Ctrl+Win+Down`). All other keystrokes are immediately passed unmodified via `CallNextHookEx` and are never logged or stored.
- **Local-Only Storage**: Configuration and extracted icon caches reside exclusively in the current user's `%APPDATA%\Winscroller2dAppview\` and `%LOCALAPPDATA%\Winscroller2dAppview\` directories.
