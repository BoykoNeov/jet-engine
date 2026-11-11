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

**After the build and the freeze (2026-11-11)** — two diagnostics for the pinned-trigger gate (anchor § 2's amendment);
they print overshoots, which is allowed only now, the § 3 predictions being frozen. Both drive the SHIPPED
`two_speed_clip` module, not the prototype.
- `diag.rs` → `diag.txt` — governed, shape 0, (0.02, 0.1), step 0.005: every RK4 stage with `g > 0` and `r ≤ g` before
  the first recorded crossing. None earlier than the crossing step's own k4 (it lands on the crossing point).
- `diag2.rs` → `diag2.txt` — open, step 0.02: (1) the same stage log on shape 0 at (0.02, 0.1) — the ENGAGEMENT step's k3
  at s = 0.29 and the crossing step's k4 at s = 0.52 read `τ_rel` early; (2) per cell (4 shapes × cut-in 0.02/0.05/0.1),
  the first departure from rung 47 at the cut-in time, the split across give-back 0.1/0.8, the peak's point, the
  overshoot across give-back 0.1/0.4/0.8, and rung 47's.

**After scoring (2026-11-11)** — the scored and reported numbers of `docs/rung108-spec.md` §§ 3–4, all on the SHIPPED
route.
- `score.rs` → `score.txt` — anchor § 3's whole grid on the 0.005 step (4 maps × the cut-in / give-back constants,
  governed and open, beside rung 106's single lag and rung 95's instant redline): overshoot, the cut at the crossing,
  sag, settle time, the LP's closest approach, the ledger per phase; the HIT / MISS / VACUOUS tally.
- `score2.rs` → `score2.txt` — the same governed cells with a 4 τ settle (off protocol): H5's misses do not move, H6's
  give-back 0.4 settles at 3.0–3.1 τ, 0.8 still not.
- `crash.rs` → `crash.txt` — the crash map (rung 95's instrument, seed 108, 1 000 random requests, the box and the lag
  on): panics, stop kinds, refusals by reason, minimum-fuel holds, re-attacks (`RE-ATTACK` lines), the slowest five.
- `slow_reqs.txt`, `wasmtime.mjs` → `wasmtime.txt` — those five slowest requests timed in WebAssembly through the page's
  own call path (Node's V8): 6.7–10.4 s.
- `reatt.rs` → `reatt.txt` — the crash map's 12 re-attacking requests: the clip's turns on the two-speed route (from
  its recorded clip and from the applied cut, which agree on all 12), then the applied cut on rung 47's single lag at the
  cut-in time (12 of 12 turn) and at the give-back time (8 of 12): the re-attack is the single lag's own.
