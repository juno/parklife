# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this app is

Parklife is a macOS desktop app (Tauri 2.11.5). The goal: open an image file, select a
region, apply blur to that region, and save the result under a new name. Image processing
runs in Rust via the `image` crate (`imageops::blur`), not in the frontend.

As of the initial commit this is the unmodified `create-tauri-app` react-ts scaffold plus
the version pin and the `image` dependency — the blur feature is not implemented yet. The
`greet` command in `src-tauri/src/lib.rs` and the demo UI in `src/App.tsx` are placeholder
scaffold code to be replaced.

## Commands

Run from the repo root:

- `pnpm tauri dev` — run the app (starts Vite on :1420, then the Tauri window; Rust build errors surface here)
- `pnpm build` — frontend only: `tsc && vite build` into `dist/`
- `pnpm tauri build` — production bundle

Rust (run from `src-tauri/`):

- `cargo check` / `cargo build` — first build compiles ~500 crates (~1 min)
- `cargo test` — run all Rust tests; single test: `cargo test <name>` or `cargo test --test <file>`
- `cargo clippy`

## Architecture

**Two halves, one bridge.** The React frontend (`src/`) and the Rust backend (`src-tauri/src/`)
are separate builds. They talk only through Tauri commands: frontend calls
`invoke("command_name", { args })` from `@tauri-apps/api/core`; Rust exposes `#[tauri::command]`
functions registered in `tauri::generate_handler![...]` in `lib.rs`. Adding a feature that
needs native work (file dialogs, image processing, disk I/O) means: new command in `lib.rs`,
add it to the handler list, call it via `invoke` from the frontend.

**Rust entry point is split.** `src-tauri/src/main.rs` is a thin shim that calls
`parklife_lib::run()`; all real setup (builder, plugins, handler registration) lives in
`lib.rs`. The `_lib` suffix on the crate name is a Windows naming workaround — don't rename it.

**Permissions are opt-in per capability.** `src-tauri/capabilities/default.json` lists what the
`main` window is allowed to do. Tauri commands and plugin APIs are denied unless their
permission (e.g. `dialog:allow-open`, `fs:allow-write-file`) is added here. When a frontend
call fails with a permissions error, this file is why.

**Config.** `src-tauri/tauri.conf.json` — app metadata (`productName`, `identifier`,
window size/title), bundle targets, and the `build` hooks that wire `pnpm dev`/`pnpm build`
into `tauri dev`/`tauri build`. `vite.config.ts` is pinned to port 1420 with `strictPort`
because Tauri expects a fixed dev URL.

## Repo-specific setup notes

- **Tauri is pinned** to `=2.11.5` in `src-tauri/Cargo.toml` and locked in `Cargo.lock`.
  Keep the exact-version pin; `tauri-build` tracks its own version (2.6.3).
- **`pnpm-workspace.yaml`** carries two deliberate settings: `onlyBuiltDependencies: [esbuild]`
  (pnpm 11 blocks build scripts by default) and `verifyDepsBeforeRun: false` (the pre-script
  auto-install otherwise exits non-zero on the ignored-build warning). esbuild ships prebuilt
  binaries, so the "ignored build" warning is harmless.
- Package manager is **pnpm**. `pnpm-lock.yaml` and `src-tauri/Cargo.lock` are committed.
