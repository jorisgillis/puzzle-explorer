# Research: core dependency set and versions for the `rust/` workspace

Ticket: #10 (child of the wayfinder map #2). Question: what minimal dependency
set does the planned `rust/` workspace (core crate + app crate) need, with
versions compatible with Rust 1.98 and fetchable from crates.io in this
environment?

## Method / verification performed

Scratch cargo workspace in `/tmp/deps-check` mirroring the planned layout
(`core` lib crate + `app` bin crate, plus a throwaway `clap-smoke` bin so the
app crate's heavy GUI dep didn't need compiling):

- `cargo add` / `cargo add --dry-run` for each crate → all resolve from the
  crates.io index in this environment.
- `cargo fetch` on the generated lockfile → all `.crate` files download from
  crates.io successfully.
- `cargo test -p puzzle-core` and `cargo check`/`cargo run -p clap-smoke`
  under the installed **cargo/rustc 1.98.1** → compile, tests pass, CLI runs.
  Smoke test exercised the exact APIs recommended below: `rand::rng()`,
  `RngExt::random_range(1..=9)`, `SliceRandom::shuffle(&mut rng)`,
  `StdRng::seed_from_u64(..)`, `#[derive(Serialize, Deserialize)]` +
  `serde_json::{to_string, from_str}`, and clap 4 derive
  (`#[derive(Parser)]` + `#[derive(Subcommand)]` with `generate/solve/analyze/ui`).
- MSRVs read via `cargo info <crate>` (each crate's declared `rust-version`);
  all are ≤ 1.98, so all are compatible with the installed toolchain.

## Recommended dependency blocks

### Core crate (`rust/core`, e.g. `puzzle-core`) — grid model, solver, generator, metrics

```toml
[dependencies]
rand = "0.10"                                # 0.10.3 resolved; MSRV 1.85
serde = { version = "1", features = ["derive"] }  # 1.0.229 resolved; MSRV 1.56

[dev-dependencies]
serde_json = "1"                              # 1.0.154 resolved; MSRV 1.71 — parity fixtures (tests only)
```

### App crate (`rust/app`, e.g. `puzzle-app`) — clap CLI + egui/eframe UI

```toml
[dependencies]
puzzle-core = { path = "../core" }
clap = { version = "4", features = ["derive"] }  # 4.6.7 resolved; MSRV 1.85
eframe = "0.37"                                   # 0.37.0 resolved; MSRV 1.95 — GUI stack settled by #6
```

Workspace root `rust/Cargo.toml` is just `[workspace] resolver = "2"` with the
two members; no workspace-level dependencies are needed.

## Rationale per crate

- **rand (core)** — randomness for grid generation and digit shuffling.
  `rand = "0.10"` resolves to 0.10.3 (MSRV 1.85 ≤ 1.98). Use the 0.9+/0.10 API:
  `rand::rng()` for the thread RNG, `RngExt::random_range(1..=9)` (note: in
  0.10 the `Rng` trait is renamed **`RngExt`** because rand_core renamed
  `RngCore` → `Rng`), `rand::seq::SliceRandom::shuffle(&mut rng)`, and
  `StdRng::seed_from_u64(seed)` for deterministic, seedable generation
  (needed for reproducible parity fixtures). **rand 0.9 vs 0.8** (per the
  ticket): 0.9 renamed `thread_rng()`→`rng()`, `Rng::gen()`→`random()`,
  `gen_range(a..b)`→`random_range(a..b)`, `gen_bool`→`random_bool`,
  `rand::distributions`→`rand::distr`, `Standard`→`StandardUniform`, and split
  `SliceRandom`; 0.8 (last 0.8.8, no declared MSRV) still uses
  `thread_rng()`/`gen()`/two-arg `gen_range(a, b)` and is the legacy line — do
  not use it. 0.10 keeps the 0.9 names and additionally: MSRV 1.85, edition
  2024, `OsRng`→`SysRng`, removed the `small_rng` feature and `ReseedingRng`.
  0.9.5 (MSRV 1.63) is a valid fallback if pinning to the 0.9 line, but 0.10.x
  is latest and verified here. `shuffle` on a `Vec<u8>` and seeded `StdRng`
  both compile and pass under 1.98.1 in the scratch project.
- **serde + derive (core)** — `#[derive(Serialize, Deserialize)]` on grid /
  metrics types so parity fixtures (Racket output vs Rust output) can be
  compared as JSON. serde 1.0.229, MSRV 1.56. Caveat found by compiling:
  serde only implements (de)serialization for arrays up to length 32, so an
  81-cell grid must be stored as `Vec<u8>` (serializes as a JSON array of 81
  numbers) or use a custom `#[serde(with = ...)]` module — `[u8; 81]` does
  not derive.
- **serde_json (core, dev-dependency)** — JSON (de)serialization for the
  parity-fixture tests. Kept as a dev-dependency because fixtures are only
  consumed by `cargo test`; promote to a regular dependency only if the CLI
  later gains a `--json` output mode. serde_json 1.0.154, MSRV 1.71.
- **clap 4 + derive (app)** — CLI with the same subcommands as
  `sudoku/main.rkt`: `generate <easy|medium|hard>`, `solve <81-char>`,
  `analyze <81-char>`, `ui`. clap 4.6.7 with `features = ["derive"]` (pulls
  `clap_builder` + `clap_derive`), MSRV 1.85; derive smoke test with
  `Parser`/`Subcommand` compiles and runs correctly under 1.98.1.
- **eframe (app)** — the egui GUI, per the stack settled in #6. eframe 0.37.0
  (egui 0.37.0), MSRV 1.95 ≤ 1.98, resolves and fetches from crates.io here.
  It dominates the dependency tree (~300 crates incl. winit/wgpu), which is
  another reason to keep it only in the app crate.

## Anything else truly needed?

No. For a minimal core, error handling can be plain `Result<T, E>` with a
hand-rolled error enum (no thiserror/anyhow), tests use the built-in
`cargo test` (no extra crates), and formatting/linting use the toolchain's
`rustfmt`/`clippy`. Adding thiserror/anyhow or a logging facade can be
revisited when the core grows, but nothing in the core scope (grid, MRV
solver, generator, complexity metrics, JSON fixtures) requires another
dependency.

## Summary table

| Crate | Version resolved | MSRV | In crate | Purpose |
|---|---|---|---|---|
| rand | 0.10.3 | 1.85 | core | RNG for generation/shuffle (`rng()`, `random_range`, `shuffle`, seeded `StdRng`) |
| serde (+derive) | 1.0.229 | 1.56 | core | derive (de)serialization on domain types |
| serde_json | 1.0.154 | 1.71 | core (dev) | parity-fixture JSON in tests |
| clap (+derive) | 4.6.7 | 1.85 | app | CLI subcommands parity with `main.rkt` |
| eframe | 0.37.0 | 1.95 | app | egui GUI (stack per #6) |

All versions are semver-compatible ranges (`"0.10"`, `"1"`, `"4"`, `"0.37"`)
that resolve to the versions above from crates.io in this environment with
cargo 1.98.1, and the lockfile fetches cleanly (`cargo fetch`).
