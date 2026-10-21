# Rung 102 Part C re-scored under rung 104's LP-turbine law (2026-10-21)

`docs/rung102-spec.md` § Part C re-scored. A scratch crate (`Cargo.toml.txt`, against the repo at `4c40a71` plus nothing:
the probe only reads) built three bins; the crate itself was temporary and is deleted.

- `score_c.rs` — `../rung102/score_c.rs` with ONE change: the LP turbine's law is `argv[1]` (`pin`, `ellipse`, `two`,
  `one`), set on the defaults every request is built from. Output `score_c_<law>.txt` (stdout; the `done` lines went
  to stderr, not kept).
- `tt4hold.rs` — `../rung102/tt4hold.rs`, the same one change. Output `tt4hold_<law>.txt`.
- `page_default.rs` → `page_default.txt` — the Controls page's DEFAULT request, fitted unlit vs lit held, per law:
  the numbers the page's afterburner note now states.
- `runall.sh` — the four laws in turn, below-normal. `cmp.py` — line-by-line compare (it drops the `LAW` header):
  `score_c_pin.txt` and `tt4hold_pin.txt` against `../rung102/score_c.txt` / `tt4hold.txt`: 0 differing lines each.
  `ranges.py` — each claim's measured range per law (the spec's table).

The paths inside `cmp.py`, `ranges.py` and `runall.sh` point at the scratch crate `W:	emp\claudeung102-rescore`, deleted
after the run; point them at this folder to re-run the comparisons.
