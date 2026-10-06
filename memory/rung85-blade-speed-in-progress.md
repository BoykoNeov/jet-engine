---
name: rung85-blade-speed-in-progress
description: "Rung 85 (blade-speed walls) is IN DESIGN, no code — the anchor doc docs/plans/rung85-anchor-blade-speed.md carries the probes, sources, user decisions and the open items; the pre-check found the stack's K and C were two disguised, inconsistent blade speeds"
metadata:
  node_type: memory
  type: project
  originSessionId: edd8c029-bea9-4b2c-99e0-c4ce222591f5
  modified: 2026-10-06T06:31:44.008Z
---

**State (2026-10-06):** design only; nothing in `rust/` touched. The anchor doc
`docs/plans/rung85-anchor-blade-speed.md` is the record — read it first.

**User decisions:** (1) BOTH walls — airflow (front-row relative tip Mach) and strength
(Ti-6Al-4V blade-root pull, 14 CFR 33.27's 120 %) — whichever binds; (2) the design flow
coefficient `Φ_d` becomes a DESIGN INPUT (Rotor 37 mean-line 0.537, NASA TP 1659 Table II),
`Φ_d` = 1 the bit-for-bit reduce to rungs 55/56.

**Settled 2026-10-06 (anchor § 6.1):** the incidence loops (60/69–71) are untouched by `Φ_d` —
they are FLOOR solves, both sides of `M_i ≥ m_lim` scale by `1/Φ_d`, so no "gain change" existed;
`v_max` is map units (rung 57's "20°" was a slip, fixed); rung 54's throat is steady-core only.

**Step 2 done 2026-10-06 (anchor § 1):** Ti-6Al-4V from the Rolled Alloys AMS 4911 sheet
(plate stand-in for forgings, disclosed); airflow level 1.3 from Biollo & Benini 2011 (review
chapter — the textbook was NOT read first-hand); 1.4/1.5 disclosed levels, Rotor 37 an example.
**User decision 2026-10-06 (anchor § 3):** the airflow level is an efficiency TARGET, not a hard
wall ⇒ it SIZES the design only; the strength redline is the ONLY off-design wall; tip Mach off
design is a READING. D4/Q5 rewritten; D1–D5 settled, D6 (golden) open.

**Step 3 done 2026-10-06 (anchor § 4 + § 6.2): predictions REGISTERED, before any code** —
P0 (`Φ_d` can't reach the speed bills, `==`), P1/P2 LP all-rows at the walls' K = 2/3 (BANDS),
P3 monotone in K, P4–P6 HP levers (P5 the riskiest), P7 shape (`tilted`), P8 tip Mach falls
while N rises; voids V1–V5; NEGATIVE rule: no default-shape crossing ⇒ a negative doc. § 6.2
is the per-cell redline table every bar reads (12 cells → 4 distinct LP/HP machine pairs).
**Amended same day, pre-code (§ 4.4):** the user's SANDBOX decision ([[sandbox-direction]])
retires the NEGATIVE gate — rung 85 ships as KNOBS, all five shapes equal; advisor fixes: P0 is
settled by PATH (`Φ_d` never enters the schedule residual; V5 retired), P3 off the 1500 point,
P7 on each shape's own redlines (§ 6.3 — `tilted` is `l` = 0.85, tightest LP `R` 1.337), P8
against the formula, A6 bare-speed note, A7 the conventions paragraph is the table's record.

**Next:** D6 — a Rust-owned CLI golden, its own commit (first rung with no Python segment); then
code; then score § 4 in `docs/rung85-spec.md`.

**Why — process lessons:** (a) a dimensionless model can still carry a physical quantity
IMPLIED by several inputs — convert each to that quantity before adding a wall on it (P-A: `K`
and `C` implied 158 vs 220 m/s on the LP spool); (b) the advisor caught a COST compared to a
LEVEL (rung 53's +66.7 % is a part-power change, not N/N_d) and a wrong "both walls scale with
N" (pre-swirl); (c) probe the side-effects of a new wall on the neighbouring disclosed constant
before registering anything (P-C: `φ_d` = 1 forced C ≈ 0.97–0.998); (d) at step 3 the draft
headline prediction ("the lumped stator crosses the redline") was ALREADY decided — the other way —
by published numbers once they were converted to the wall's own reference (rung 53 lumped
N_L/N_d = 1.26 vs the smallest LP redline 1.395). Convert every published bill to the new
wall's denominator before writing a bar; and when an interpolation lands ON a bar (P1: 1.40
vs 1.395), register a BAND with both outcomes named, never a direction.

**How to apply:** § 4 is now FROZEN — score it, never edit a bar after code runs; take no credit
for § 0's or § 4.0's pre-check numbers. See [[rung56-per-row-capacity]], [[per-row-blading-negative]].
