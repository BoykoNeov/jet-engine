# Rung 104 scratch archive — what the anchor's givens were measured with

Archived 2026-10-20 before any rung-104 code entered the repo (the anchor: `docs/plans/rung104-anchor-turbine-law.md`).

* `probe/` — G1–G4, against the repo at `33250a2` (outputs `g1.txt` … `g4.txt`).
* `probe2/` — P5–P10, against a throwaway worktree of `33250a2` carrying the PROTOTYPE:
  `worktree-probe.patch` (its diff to `33250a2`) plus `turbine_law.rs.prototype` (the new, untracked module).
  The prototype's default law came from the env var `R104_LAW` so the whole gate could run under each law.
* `p5_before_fix.txt` — P5 before the joint-search repair (anchor G7); `p5_slowsearch.txt` / `p7_slowsearch.txt` —
  P5/P7 before the searches were started from design values (anchor COST); `p5.txt`, `p7.txt` — after both.
* `p6*.txt` — the Controls timings (anchor COST): `p6` first (single runs), `p6b` ten runs with call counts, `p6c` after
  the design starts, `p6d` after the below-choke false-position search.
* `p9.txt` — the three-way comparison below choke (old model / rung 97's branch / each law) — anchor G9.
* `preflight.txt` — the scored cells' classes only (anchor § 3).
* `gate_pin.log` — the whole gate with the prototype on the pin (anchor G5).
* `rung104_pin.txt` — R1b's record of the OLD model, made by `probe/src/pin_golden.rs` against the repo at `33250a2`
  (no rung-104 code in it); `preflight_h2.txt` — H2's cells, class only; `p11.txt` — the C3 call count.
* NB `gate_pin.log` ran on an EARLIER state of the prototype than `worktree-probe.patch` (before G7's repair and the
  COST search changes); both changes touch only the laws' paths.
