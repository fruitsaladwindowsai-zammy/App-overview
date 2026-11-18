# Building from Source

## Toolchain Options on Windows

### Option A: MSVC Toolchain (Standard)
Install Rust (`x86_64-pc-windows-msvc`) and Visual Studio C++ Build Tools:
```powershell
winget install --id Rustlang.Rustup
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
cargo build --release
```

### Option B: GNU Toolchain (`x86_64-pc-windows-gnu`) + `rust-lld`
This repository includes `.cargo/config.toml` configured with `rustflags = ["-C", "linker=rust-lld"]` so builds on the Windows GNU toolchain link cleanly without `raw-dylib` IAT corruption.

## Build & Verification Commands
```powershell
# Format check
cargo fmt -- --check

# Clippy static analysis
cargo clippy --all-targets

# Unit tests (search scoring, pagination, grid navigation)
cargo test

# Optimized release build
cargo build --release
```

The standalone executable is generated at:
```text
target\release\winscroller-2d-appview.exe
```
