# Implementations, layout, and tests

Two implementations live side by side, held to identical behavior by golden
parity fixtures: **Racket** (`sudoku/`) and **Rust + egui** (`rust/`).

## Layout

```
sudoku/           Racket implementation
  info.rkt        package metadata (installable via raco pkg)
  grid.rkt        9x9 grid model, validation, string/pretty rendering
  solver.rkt      MRV backtracking solver, constraint propagation, solution counting
  generator.rkt   complete-grid generation + clue removal with uniqueness guarantee
  complexity.rkt  complexity metrics and 0-100 difficulty score
  ui.rkt          native racket/gui front end
  main.rkt        CLI entry point
rust/             Rust implementation (cargo workspace)
  core/           sudoku-core lib crate: grid, solver, generator, complexity
  app/            sudoku bin crate: clap CLI + egui GUI
  SPEC.md         core crate spec
  SPEC-UI.md      egui app behavior spec (ported from sudoku/ui.rkt)
tests/            rackunit test suites (raco test tests/)
docs/             detailed guides (playing, solving, this file)
assets/           images used by the README
```

## Requirements

- Racket (developed against v9.3). The GUI additionally needs the platform's
  GTK3/X11 libraries, which any desktop Racket install provides.
- Rust (stable toolchain) for the `rust/` workspace.

## Tests

```
raco test tests/
```

## The Rust twin (rust/)

Same capabilities as the Racket app: egui GUI with the same board interaction,
colors, buttons and live analysis panel, and the same CLI subcommands. The
core logic (MRV solver with propagation, uniqueness-checked generator,
complexity metrics) lives in the `sudoku-core` library crate and is covered by
unit tests plus golden parity fixtures measured from the Racket
implementation (`rust/core/tests/fixtures/parity.json`) — two languages,
one truth.

Build, run, test:

```
cd rust
cargo build            # builds sudoku-core + the sudoku app
cargo run -- ui        # GUI (on Wayland: env -u LD_LIBRARY_PATH cargo run -- ui)
cargo run -- generate hard
cargo run -- solve  "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
cargo run -- analyze "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
cargo test --workspace # fmt/clippy/test also run in CI (.github/workflows/rust.yml)
```

Deliberate divergences from the Racket version (documented in the parity
fixtures): the Rust solution counter is correct where Racket's `for/and`
short-circuit undercounts (so `unique` is `true` for puzzles like
"hard-17"/"escargot" where Racket reports `#f`), and CLI booleans print as
`true`/`false` instead of `#t`/`#f`.

## See also

- [GLOSSARY.md](../GLOSSARY.md) — shared vocabulary for the project.
- [rust/SPEC.md](../rust/SPEC.md), [rust/SPEC-UI.md](../rust/SPEC-UI.md) — specs the Rust crates are built against.
