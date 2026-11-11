# Rung 108 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\rung108-probe`, deleted after the rung; its
manifest is `Cargo.toml.txt`). `lib.rs` is a PROTOTYPE built from public pieces only: rung 106's `governor_lag::try_der`
copied with the one line `dg = (r − g)/lag.tau(r, g)` (rung 52's two constants on the redline clip), the demand either
rung 95's governor or the open ramp (`Demand::Ramp`); `march(sv, att, rel, ds)` drives it from a Controls request. Every
script printed REACHABILITY, step-check gaps, cost or the ledger's relative miss only — never an overshoot, a thrust, a
time, a stall margin or a cut size (anchor `docs/plans/rung108-anchor-two-speed-limiter.md` § 0; checked on review).

- `sweep.rs` → `sweep.txt` — G1/G2/G3: 576 runs (72 settings × `τ_att` {0.01, 0.02, 0.05, 0.1} × `τ_rel` {0.1, 0.8}):
  bit-equality with the page's route at equal constants (`same=`), run lengths, step-check gaps at steps 1, 2, 4 against
  4× finer, attack / release edge counts, lean-wall holds, seconds.
- `base.rs` → `base.txt` — G3: rung 106's SHIPPED single lag (0.1 / 0.2) and rung 95's instant route at one step on the 63
  governed settings — the pre-existing roughness beside the floor / schedule (to 6.2e−4).
- `timing.rs` → `timing.txt` — G4: one-thread native cost, rung 106 against the two-speed march at (0.02, 0.2) on 0.005.
- `open2.rs` → `open2.txt` — G2/G3: 900 open runs (3 cases × 3 flights × 4 maps × 5 × 5 constants): stops (`NOTE` lines —
  the flat-LP floor runs only) and the worst one-step gap per constant pair.
- `ledger.rs` → `ledger.txt` — G5: the ledger's relative miss per phase (attack, release), at the run's step and half it.
