# Winscroller 2d Appview Documentation (mdBook)

This directory contains the [mdBook](https://rust-lang.github.io/mdBook/) documentation source deployed automatically to GitHub Pages via `.github/workflows/gh-pages.yml`.

## Preview Locally

```powershell
cargo install mdbook
mdbook serve docs --open
```

## Build Static HTML

```powershell
mdbook build docs
# Output is written to docs/book/
```
