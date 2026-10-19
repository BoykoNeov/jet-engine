# HP-wall repair probes (sandbox plan § 36; archived here — the scratch crates and worktree they ran in were temporary and are deleted)

Base commit: `070b729` (the code before the repair); the repair is `a1df329`. Two scratch crates, each a set of
`[[bin]]`s against the repo (`probe_Cargo.toml.txt`, `page_Cargo.toml.txt`); the "before" runs of the page crate were
the same sources built against a throwaway worktree of `070b729`.

- `probe.rs` → `probe_before.txt` / `probe_after.txt` — rung 40's equilibrium (OLD, from full speed), rung 39's steady
  matcher with rung 103's switch (ST), and the equilibrium SEEDED at the steady root (SEED), on the four Controls maps ×
  four flights × 550–1500 K. Before: seeded at the root it still failed (the finding that moved the search from the
  Newton's start to the closure). After: 21 of 37 failures solve, all within 6e-13 of the steady speeds.
- `gscan.rs` — the closure residual `g(m)` over the whole bracket. The archived source is its LAST version (the
  full-speed start at 550/600 K, standard day); the first version scanned at the steady root's speeds (flat-LP, design
  flight 700/800 K, standard day 700 K, +40 K 900 K) and showed `g` turning back negative near `m ≈ 2.4`, where `Tt3 < 0`.
- `page.rs` → `page_before.txt` / `page_after.txt` — the Controls page's own `build` + `march`, flat-LP, four closures
  (plain / bleed lever / customer / cooling), two flights, 600–800 K: 31 of 40 refused before, 0 after.
- `crash.rs` → `crash_summary.txt` — seed 36, 1 200 random Controls requests through `sandbox::call`, before vs after.
  `crash_moved_words.tsv` (index, old reply, new reply) — the 19 refusals whose words changed;
  `crash_offmap_remaining.tsv` (index, request, new reply) — the 18 still "off the maps".
- `classify.rs` → `classify.txt` — those 18 against the steady matcher and the bare / seeded equilibrium.
- `thrust.rs` → `thrust.txt` — the 8 where the steady matcher fails its efficiency check: specific thrust from rung
  103's probe joint loop (copied from `../rung103/probe1_census.rs`) — all negative, nozzle choked.
- `floor.rs` → `floor.txt` — the lowest throttle that solves per map at Mach 0.85 / 1.6 / 2.0 / 2.6 (design-flight air)
  and the 11 km range, before and after: the numbers in the Controls refusal words.
- `check.rs` → `check.txt` — the flat-LP edge for the page tests (500/510 K refused, 520 K runs).
