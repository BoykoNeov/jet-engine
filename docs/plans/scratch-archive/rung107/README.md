# Rung 107 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung107-probe`, deleted after the rung; its
manifest is `Cargo.toml.txt`). `lib.rs` is a PROTOTYPE built from public pieces only: rung 95's governor as the demand of
rung 52's asymmetric (fast-attack / slow-release) lag on the schedule and the floor, in `integrate_fuel_asym`'s order —
the redline and the metal leg instant on top (here in sequence; the frozen device solves them together, anchor § 0), an
applied fuel below the lean wall held there on the governed path; `open = true` hands in the open ramp instead. Every
script printed REACHABILITY, step-check gaps or the ledger's miss only (anchor `docs/plans/rung107-anchor-governor-release.md`
§ 0).

- `check52.rs` → `check52.txt` — G1: the prototype's open path IS the Controls view's rung-52 route, bit for bit (64 runs).
- `reach.rs` → `reach.txt` — G2/G3: 560 governed runs at the first lag pairs (mostly attacks now refused): run length,
  stops, the governor's clamps, the lean hold, which leg held, releases / re-attacks, the cut asked for at s = 0,
  bit-equality with rung 95 before engagement, cost. (Its fast ramp 0.15 was refused by the page: not a whole step.)
- `reach2.rs` → `reach2.txt` — G3′: the same in the ADMITTED region (attack 0.2 / 0.4 × release 0.1–1.6, HP gain 10, the
  fast ramp 0.16), 1 344 runs — the advisor's check.
- `step.rs` → `step_*.txt` — the first step check (step 0.02 against 0.005), each law at RUNG 95's edges and at the page
  defaults, attack 0.02–0.2 × release 0.02–0.8, three flights, per case (schedule / floor on press/flow / both + redline).
- `base.rs` → `base_*.txt` — the baselines: rung 95 with the INSTANT schedule / floor, the governor-off release (rung 52),
  and the governed release at slow attacks, the same comparison.
- `edge.rs` → `edge_*.txt` — the fine sweep at rung 106's per-law edges: attack 0.04–0.5 × release 0.02–0.8 — the table the
  user chose the knobs' floors from (attack ≥ 0.2, release ≥ 0.1).
- `edge16.rs` → `edge16_*.txt` — the same past the slider: attack 0.2 / 0.4 × release 1.0 / 1.6.
- `ledger.rs` → `ledger.txt` — D1's per-phase miss on recorded points, the schedule (four maps) and the floor (three),
  at steps 0.02 and 0.01 — the measurement the gate's bars were frozen from.
- `score.rs` → `score.txt` — the SCORING on the shipped code (anchor § 3): H1–H6 cell by cell with the vacuity rules
  written in its header before the first run, then the reported quantities (crossing times, D1's let-through and
  withheld fuel, floor + schedule + redline, the other laws at release 0.4, the floor cells with the governor OFF, a
  longer settle, cost).
- `crash.rs` → `crash.txt` — the crash map: 1 000 random governed requests with the release on (seed 13), through
  `sandbox::call`, each admitted run re-read through the library to count releases and re-attacks.
- `sagcause.rs` → `sagcause.txt` — off protocol: which edge of the lag makes the sag (attack and release varied
  separately, steps 0.02 and 0.005) — the decomposition the headline rests on.
