# puzzle-explorer docs

Explainer pages about the algorithms behind this project — written for anyone
curious, playful in tone, but every factual claim is cited or linked to code.

## Pages

- [solvers.md](solvers.md) — Which Sudoku-solving algorithm is fastest, and how do naive backtracking, propagation + MRV, and DLX compare?
- [generation.md](generation.md) — How do you generate a Sudoku puzzle with exactly one solution?
- [complexity.md](complexity.md) — Is Sudoku NP-complete?

## Assets

- [assets/benchmark-results.csv](assets/benchmark-results.csv) — raw per-run benchmark rows (Racket harness)
- [assets/benchmark-table.md](assets/benchmark-table.md) — the aggregated results table embedded in [solvers.md](solvers.md)
- `assets/<page>-<slug>.png` — hand-made goofy images, one per page section that wants one

## The explainer template

Every page under `docs/` follows this section order (locked by ticket #38):

1. **Question** — the page title phrased as the question it answers
2. **TL;DR** — the answer in 2-4 sentences
3. **Deep dive** — the full explanation
4. **Complexity callout** — the asymptotic / NP-hardness angle (see [complexity.md](complexity.md))
5. **Diagram** — mermaid block or ASCII art, always with alt text
6. **Worked example** — a concrete puzzle walked through
7. **Goofy image** — a hand-made PNG from `assets/<page>-<slug>.png`
8. **See the code** — relative links into the repo (`../sudoku/...`, `../rust/...`)
9. **Further reading** — primary sources with URLs/DOIs

Conventions: mermaid renders inline on GitHub and ASCII is allowed (9x9 grids
read well in ASCII); every diagram gets alt text. Tone is public and playful,
but jokes never obscure a claim — every factual claim carries a citation or a
code link. Images are original and committed (never hotlinked).
