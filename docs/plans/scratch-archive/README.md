# Scratch archive — the cited parts of `W:\temp\claude`

Committed specs, plans, tests and memory notes cite files under `W:\temp\claude\<folder>\…`
as the probe or record behind a measured claim (e.g. `rust/tests/rung82.rs` cites
`W:\temp\claude\slice-aj-step5\probe_voids.py`). Those temp folders were deleted on 2026-10-08.
This folder keeps what they cited.

**Path mapping.** `W:\temp\claude\<folder>\<file>` is now `docs/plans/scratch-archive/<folder>/<file>`.
The citations themselves were left as written.

**What was kept**, per cited folder: its top-level scripts and notes (`.py .md .rs .toml .ps1 .cmd
.mjs .js .sh .html`, each ≤ 200 KB), plus `src/` of the scratch crates (`jet-*`) and
`solver_audit/rust/examples/` (the probes `todo-solver-tolerance-audit.md` names).

**What was NOT kept** (regenerable or bulky):
- run logs and captured outputs (`.out .err .log .txt .tsv .json`), e.g. `r55.tsv`,
  `main_pypy_stdout_lf.txt`, `probe_run.tsv`. Rerun the archived probe to regenerate one.
- copies of the repo or the Python model: `slice_au\wt`, `slice-aj-preflight\ctl|mut`,
  `solver_audit\rust\src` and `fixcheck\`. Use git history and the `python-final` tag.
- build output (`target\`), browser profiles, and folders that were never cited.

**Added 2026-10-10 — slice 5's burner probes** (`jet-combustor-timing`, plan § 13.2; `jet-burner-map`, the
crash map and grid check, § 13.9). These two also keep their small summary `.txt` files (`fast/curve/pocket.txt`
= the crash map's outcome classes and timings, `grid2.txt` = the fixed grid against the model's defaults;
`gas/ref/shape/time.txt` = § 13.2's measurements), because § 13.9 quotes numbers from them. The raw `.tsv`
outputs (14 MB) were not kept.

Scripts here ran against the model **as it was** on their date (many drive the Python at tag
`python-final`). Nothing in this folder is built, run or tested by the crate.
