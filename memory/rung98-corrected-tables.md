---
name: rung98-corrected-tables
description: "Rung 98 (δ/θ-corrected control tables) process lessons — a table read outside its rows is a clamp (H4's MISS); a default that hides the rung must go to the user before the anchor freezes; a bash var inside a cmd string stays literal (again)"
metadata:
  node_type: memory
  type: project
  originSessionId: 08fc14b8-3417-46cb-88af-c1084de6a116
  modified: 2026-10-18T08:30:04.226Z
---

Rung 98 (2026-10-18): `corrected_tables.rs` — three tables made at the design flight, carried by `δ`/`θ`; anchor
`897001b`; 68 HIT / 4 MISS of 72. Follows [[rung98-decisions]], [[rung97-decisions]], [[rung96-inferred-thrust]].

**Lesson 1 — a table read OUTSIDE its rows is a clamp, not the curve.** H4 derived the uncorrected proportional law's
error from the steady-fuel exponent (k ≈ 4.7, which held) and predicted "settles faster in hot air". It settled slower:
the rows sit where the CORRECTED reading falls, so the uncorrected one ran past the top row at 111/126 points, and rung
95's steady table holds its end value there. Before deriving how a misread table errs, check whether the misread lands
inside its rows, and what the table does beyond them (rung 96's extrapolates, rung 95's and rung 48's clamp).

**Lesson 2 — a default that hides the rung is the user's call, before the freeze.** The advisor's "this flight"
default kept saved runs bit for bit but left the user's two switches inert; a side note flagged it; asked, the user
chose "design flight". Pre-register what the default moves and census it by running (1 of 77 check requests moved;
the crash map's 129 diffs all inside the predicted class).

**Lesson 3 — the advisor's pre-freeze review caught three freezable errors:** scoring flights never checked for TABLE
reach (only their own ends), a 32-cell prediction that was construction (moved to a gate), and two guessed bars
(derive from the solvers' tolerances; probe the identity on every variant, printing only its spread).

**Tooling:** `cmd //c "... W:\temp\claude\\$C\..."` in a bash loop left a literal `$C` AGAIN (see
[[windows-tooling-file-hazards]]) — spell paths out. A perl `-0` regex with `/s` and `.*` ate a whole file — use `-p`
line mode or the Edit tool. A path with a space breaks `--manifest-path` in `cmd start`: use the repo-relative
`rust\Cargo.toml` (the Bash cwd is the repo).
