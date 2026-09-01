# Parklife

A macOS desktop app for blurring a region of an image and saving the result as a new file.

Open an image, drag to select a region, apply blur, and save a copy. Blur is done in Rust
(the `image` crate), so large images stay responsive.

> Status: early scaffold. The blur workflow is not implemented yet — the current build is
> the Tauri starter template.

## Stack

- [Tauri](https://tauri.app) 2.11.5 (Rust backend)
- React 19 + TypeScript + Vite 7 (frontend)
- pnpm

## Prerequisites

- Rust toolchain (`rustup`)
- Node.js + pnpm
- macOS with Xcode command line tools

## Development

```sh
pnpm install
pnpm tauri dev      # run the app
```

Other commands:

```sh
pnpm build          # frontend build only (tsc + vite)
pnpm tauri build    # production bundle

cd src-tauri
cargo test          # Rust tests
cargo clippy
```

See [CLAUDE.md](./CLAUDE.md) for architecture notes and repo-specific setup details.
