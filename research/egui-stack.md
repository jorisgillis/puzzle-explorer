# Research: pin eframe/egui versions and prove a hello-world builds and runs here

Ticket: #9 (child of the wayfinder map #2). Question: which eframe/egui
versions build with the installed cargo/rustc 1.98.1, is crates.io reachable
from this environment, and does a minimal eframe app compile and open a
window on this (Wayland) session — or must it run under XWayland? Plus
display quirks relevant to the sudoku UI.

## Environment facts (verified 2026-11-13)

- `cargo 1.98.1 (797e8a9bc 2026-08-05)`, `rustc 1.98.1 (48a229cea 2026-09-01)`
  at `/home/joris/.cargo/bin/`.
- Session: `XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-1`,
  `DISPLAY=:0` with XWayland sockets (`/tmp/.X11-unix/X0`, `X1`) present.
- GPU: AMD Radeon RX 480 (polaris10), Mesa 25.3.6 (RADV Vulkan + radeonsi GL
  4.6 Core). `glxinfo -B` works **only with a clean `LD_LIBRARY_PATH`** (see
  quirk below).

## crates.io reachability

Yes. The sparse index `https://index.crates.io/config.json` returns 200,
`static.crates.io` serves `.crate` files (200), and `cargo add`/`cargo build`
resolve and download fine: `cargo add eframe` locked **426 packages** to
latest Rust-1.98.1-compatible versions. (The REST API
`crates.io/api/v1/crates/eframe` returns 403 without a `User-Agent` header —
that is normal; with a UA it works and reports `max_version: 0.37.0`. Cargo
itself is unaffected.)

## Versions that build with rustc 1.98.1

Latest published **eframe/egui 0.37.0** builds cleanly with rustc 1.98.1.
Declared MSRVs from the crates.io index (all ≤ 1.98.1, so all compatible):

| eframe | rust-version (MSRV) |
|--------|---------------------|
| 0.37.0 | 1.95 ← latest, **works** |
| 0.36.x | 1.95 |
| 0.35.0 | 1.92 |
| 0.34.x | 1.92 |
| 0.33.x | 1.88 |
| 0.32.x | 1.85 |

Exact versions locked by the probe (`/tmp/egui-probe/Cargo.lock`):
**eframe 0.37.0, egui 0.37.0, wgpu 30.0.2, winit 0.30.13** (426 crates total
with default features). No fallback was needed; 0.37.0 is the pin.

**API note (0.37 breaking change):** the `eframe::App` trait no longer has
`fn update(&mut self, ctx, frame)`. It is now `fn ui(&mut self, ui: &mut
egui::Ui, frame: &mut Frame)` (plus an optional `fn logic(&mut self, ctx,
frame)` for when the window is hidden). Also, `ui()` is only called when a
repaint is requested — a static UI renders a few frames and then stops; call
`ctx.request_repaint()` if you need continuous animation. `run_native` still
takes `Box::new(|_cc| Ok(Box::new(MyApp)))`.

Recommended dependency block for the app crate (matches ticket #10):

```toml
[dependencies]
eframe = "0.37"   # default features: wgpu renderer + wayland + x11 + winit
 egui = "0.37"    # pin both; eframe re-exports egui, but depend on it directly
```

(Optional glow backend instead of/in addition to wgpu: `eframe = { version =
"0.37", features = ["glow"] }` — verified to build and run; select at
runtime with `NativeOptions { renderer: eframe::Renderer::Glow, .. }`.)

## Build proof and timings (scratch dir `/tmp/egui-probe`, outside the repo)

`cargo init --bin`, `cargo add eframe egui@0.37 env_logger`, hello-world
`main.rs` (400x300 window, heading + frame counter, closes itself after 150
frames via `ViewportCommand::Close`).

- `cargo build` (debug, from scratch, 426 crates): **~1m28s wall**
  (469s user, 577% CPU). Glow variant (extra crates): ~1m40s wall.
- `cargo build --release` from scratch: **~3m20s wall**; binary 30 MB.
- Both profiles compile with zero errors on rustc 1.98.1.

## Run proof on this session's display

Hello-world **opens a window and renders** (probe logged frames 30..150,
then closed itself; exit code 0, no errors):

- **Default path = native Wayland, no XWayland needed.** With
  `WAYLAND_DISPLAY=wayland-1` set, winit picks the Wayland backend.
  `WAYLAND_DEBUG=1` shows a real `xdg_toplevel` created and `wl_buffer`
  commits (600x450 px = 400x300 logical at 1.5 scale). wgpu renders on the
  **RADV "AMD Radeon RX 480" Vulkan adapter** (also llvmpipe and GL 4.6
  adapters available).
- **XWayland path also works:** `env -u WAYLAND_DISPLAY WINIT_UNIX_BACKEND=x11`
  → `xlsclients` lists `0x1200002 "egui-probe hello-world" 600x450+1620+842`
  (a mapped X window via XWayland), 150 frames, exit 0.
- **Glow renderer also works** on the same session (150 frames, exit 0).
- Release binary behaves identically (150 frames, exit 0).

## Display quirk that matters: polluted `LD_LIBRARY_PATH` breaks the GPU

This session's shell has
`LD_LIBRARY_PATH=/home/joris/.local/share/zed/external_agents/registry/mistral-vibe/.../_internal`,
which bundles an **older `libstdc++.so.6`**. Under it, every system Vulkan
ICD fails to load (`GLIBCXX_3.4.29/3.4.32 not found`), `glxinfo` fails
("couldn't find RGB GLX visual"), and the eframe app **exits with
`Wgpu(RequestAdapterError(NotFound ...))` — "No suitable graphics adapter
found"** (even with `WGPU_BACKEND=gl`: "Failed to create surface").

**Fix: run the app with a clean environment, e.g.
`env -u LD_LIBRARY_PATH ./target/debug/sudoku-ui`** (or strip the vibe
`_internal` entry from `LD_LIBRARY_PATH`). With that one change, both the
Wayland and XWayland paths above work.

## UI constraint for the sudoku app (from map #2 / Racket experience)

Popup dialogs / separate egui viewports are known to **hang in this
environment** (popup windows hung the Racket GUI). Build the UI with
**in-window panels only** — `egui::CentralPanel` + `SidePanel`/`TopBottomPanel`,
and modal flows as an in-window `egui::Window` (or an `Area` overlay), never
`ctx.show_viewport_deferred` / OS-level popup windows.

## Reproduction

```sh
mkdir /tmp/egui-probe && cd /tmp/egui-probe
cargo init --bin --name egui-probe
cargo add eframe egui@0.37 env_logger
# write the hello-world main.rs (App::ui + request_repaint + Close at 150 frames)
cargo build                                  # ~1.5 min debug
env -u LD_LIBRARY_PATH ./target/debug/egui-probe   # renders 150 frames, exit 0
```

Sources: crates.io sparse index + REST API (version/MSRV data), eframe
0.37.0 source in the local cargo registry (`src/epi.rs` for the `App` trait,
`Cargo.toml` for features), probe runs `run1..run10` logs in `/tmp/egui-probe`,
`glxinfo`/`xlsclients`/`xprop`/`WAYLAND_DEBUG=1` output on this session, and
issue #2 environment notes.
