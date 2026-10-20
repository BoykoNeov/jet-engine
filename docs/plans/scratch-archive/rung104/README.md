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

**Added at the rung's close (2026-10-20), against the repo with rung 104's code (`probe2/Cargo.toml` points at it):**

* `p11b.txt` / `p11c.txt` — the C3 call count again: after the start-from-design repair, then after the polished
  temperature inversions (two rows 3–5.5x the ellipse, ≤ 0.21 s a point — spec § 1).
* `p12` — H2's equilibrium-gas cells, class only (pre-flight); `p13` — the two-row flux's smoothness at 1e-12 steps
  (what led to the polishing); `p18` — rung 103's R6 on the cooled variant at the unchoke boundary (the boundary fix).
* `p14` — are the control rungs' failures dust? (per-step pin-vs-ellipse difference ≤ 5e-15, all choked — the case for
  the shortcut; its output was read in session, not saved).
* The sort: `p15.txt` rung 103's sign, `p16.txt` rung 102 Part C, `p17.txt` rung 102's two stops, `p19.txt` the moved
  witnesses (low start, customer bleed, altitude, Blades, lean wall), `p20.txt` rung 97's numbers, `p21.txt` which CLI
  panels differ from their Python segment.
* `r9_r2_.txt` / `r9_r4_.txt` — R9, the red check (the joint search forced back to the pin's HP ratio): R2 passes, R4
  fails.
* `gate_r1a.log` — R1a, the whole gate built with `pin-default`.
* `p22.txt` — the close-out check of the 264 retired perfect-gas keys: where a map term moves `η_LPT` off design the law
  differs from the pin by 2–12 parts per million (real, not search rounding), the same on the pages' surge maps.
