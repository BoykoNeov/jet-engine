# Rung 103 probes (archived here; the scratch crates and worktree they ran in were temporary and are deleted)

Base commit: `2688ce8`. Probe 1 is a `[[bin]]` of `probe1_Cargo.toml.txt` against the repo; probe 2's bins
(`probe2_Cargo.toml.txt`) ran against a throwaway worktree of `2688ce8` carrying `worktree-probe.patch` — a PROTOTYPE of
rung 103's device plus three probe-only switches (`guard_off`: skip the refusal, the pin's numbers; `retry_any`: retry
the branch on any pinned failure; `fixed_pi`: hold the LP turbine at a trial `π`) and the census logger. Not the shipped
code.

- `reach_census.tsv` — anchor G1: one line per unchoked refusal over the whole gate (site, `Tt4`, `M0`, `p0`, test
  program, test name), the logger enabled by `R103_CENSUS`.
- `probe1_census.rs` → `probe1_census.txt` — G2: rung 39 OLD / the pin / rung 97's helper in the joint loop / rung 40's
  equilibrium, and the residual scan over `π_LPT`.
- `variants.rs` → `variants.txt` (first run, before the retry columns) and `variants_retry.txt` (the archived source,
  with them) — G3.
- `indep.rs` → `indep.txt` — G4.
- `arrow.rs` → `arrow.txt` — G5 (choked cells only).
- `preflight.rs` → `preflight.txt` — § 3's cells: the pin side and the ON outcome's class only.

After the code (against the rung-103 repo unless noted; crate `score_Cargo.toml.txt`, the base crate
`base_Cargo.toml.txt` against a worktree of `8bf63b9`):
- `score_report.rs` → `score_report.txt` — spec § 3's reported quantities on H2's 174 cells.
- `score_cost.rs` → `score_cost.txt`; `score_count.rs` → `score_count.txt` — the cost trigger and its count.
- `score_crash.rs` → `score_crash_summary.txt` — the crash map (built against both; the raw replies are regenerable
  from the seed and were not kept).
- `score_ask.rs` — prints the page's reply to a request (used to read the changed page tests' cases).
- `gate_on_summary.txt` — the whole gate with the switch's default flipped ON (spec § 5).
