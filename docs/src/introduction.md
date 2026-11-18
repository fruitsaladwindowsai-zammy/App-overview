# Introduction

Welcome to the official documentation for **Winscroller 2d Appview** — a 100% native Rust Launchpad-style 2D application overview for Windows 10 and Windows 11.

![2D Launchpad Grid & Borderless Icon Halo](./assets/concept-launchpad-grid.svg)

## Why Winscroller 2d Appview?

The default Windows Start Menu and Task View break flow with cramped vertical lists, heavy web-view lag, and abrupt cuts. **Winscroller 2d Appview** reimagines application launching on Windows as an instant, fluid, hardware-accelerated spatial canvas:

- **Native 4-Finger Touchpad Control**: Swipe 4 fingers UP to summon the overview (automatically suppressing Windows Task View) and 4 fingers DOWN or press `Esc` to dismiss smoothly.
- **Continuous Two-Finger Page Scrolling**: Slide seamlessly between app pages with sub-pixel tracking, fling velocity detection, and damped harmonic spring snapping.
- **Borderless Launchpad Aesthetics**: Selected icons scale up **1.10x** over a soft, borderless frosted halo without bulky bounding boxes or borders.
- **Crystal-Clear Unplated Icons**: Extracts 256×256 Shell COM icons without Windows Start Menu plates, fixes legacy 0-alpha GDI black boxes, auto-crops transparent margins, and caches Lanczos3-scaled PNGs.
- **Dynamic Wallpaper Frosted Blur**: Automatically blurs your active Windows `TranscodedWallpaper` with a fast 2-pass box blur and dark acrylic vignette.
- **Zero Taskbar Clutter**: Lives silently in the Windows notification tray with a native right-click menu and global `Alt+Space` fallback.
