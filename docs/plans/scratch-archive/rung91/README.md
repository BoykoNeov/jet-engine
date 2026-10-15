# Rung 91 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\jet-rung91-probe`, deleted after the rung).

- `givens.rs` → `givens.txt` — anchor § 0: the uncooled line's HP-turbine point, rung 90's measured customer response, and anchor § 1's linear model for the vane and blade streams (existing code only, run before any cooling code existed).
- `score.rs` — every pre-registered quantity: part A (steady, flat + `flow/press`, Richardson-extrapolated from shares 0.01 and 0.02) → `score_a.txt`; part B (rung 64's valve rig, 6 cells × 4 demands — none, vane 0.05, blade 0.05, customer 0.05 — × 6 marches) → `score_b.txt`.
- `peek.rs` — which refusal fires when the customer and the cooling together pass half (the root check's, in words).
- `crash.rs` — the sandbox crash map (1 000 random Controls requests, each carrying turbine cooling) → `crash.txt` (counts in `docs/plans/sandbox-plan.md` § 20).
