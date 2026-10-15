# Rung 90 — scratch scripts and the scored outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\jet-rung90-probe`, deleted after the rung).

- `givens.rs` — anchor § 0: the c = 0 running line and rung 42's `b` response against the linear model (existing code only, run before any customer-bleed code existed).
- `score.rs` — every pre-registered quantity: part A (steady, flat + `flow/press`) → `score_a.txt`; part B (rung 64's valve rig, 6 cells × 3 demands × 6 marches) → `score_b.txt`.
- `a5.rs` — A5 re-read: `score.rs`'s off-line bracket (Tt4 ± 300 K) reached unchoked points below 1000 K and printed NaN; this brackets upward only (results in `docs/rung90-spec.md` § 3).
- `amount.rs` — after scoring: is the amount mode's refusal at w = 0.3 real? (yes: 50.2 % of the air at 1400 K).
- `stops.rs` — after scoring: which Controls requests the amount mode refuses or stops.
- `crash.rs` — the sandbox crash map (1 000 random Controls requests, each carrying a customer bleed) → `crash.txt` (counts in `docs/plans/sandbox-plan.md` § 19).
