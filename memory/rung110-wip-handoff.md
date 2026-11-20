---
name: rung110-wip-handoff
description: "Rung 110 (turbofan flown) paused AGAIN 2026-11-20 (session 2): gates 1-5 green, gate 4b one convergence failure, full gate NOT run, all UNCOMMITTED on main — what is done, what is next"
metadata:
  node_type: memory
  type: project
  originSessionId: e6648d71-da72-47be-b1d8-c7aee1ca569c
  modified: 2026-11-20T09:07:28.311Z
---

**Paused 2026-11-20 (second session) at the user's "wrap up".** Still UNCOMMITTED on `main` (not verified by the
full gate): `rust/src/two_spool.rs`, `matcher.rs` (NEW this session), `flown_turbofan.rs`, `bleed.rs`,
`customer_bleed.rs`, `shaft_offtake.rs`, `stator.rs`, `turbine_cooling.rs`, `two_spool_transient.rs`, `lib.rs`, and
`rust/tests/rung110.rs` (new).

**Verified this session:** the no-fan path BEFORE the nozzle-rule change — rung38/39/40/41/42/53/90/91/92/97/102/103/
104/105, two_spool_oracle, two_spool_transient_oracle, cli_golden all green. Gates in `rung110.rs`: 1 (90 pts bit-equal
vs rung 38's own loop), 2 (90 pts bit-equal vs rung 31), 3a (44 designs ≤2e-10), 3b (44 designs, choked/unchoked core,
unchoked fan, ≤2.7e-10), 4 (90 pts ≤1e-6), 5 (model vs book's equations exact (1+f) 1.1e-11; re-derivation vs printed
0.117 %) — green.

**Changed this session (each a finding for the spec):**
- `FAN_TOL = 1e-10` for the joint (f, pt4) loop with a fan: table-gas inversions scatter `pt4_new` ~2e-11 (measured,
  loop gain ~0.3), so `TOL` 1e-13 made a 2-cycle for ever. With a fan, an exhausted loop is now an `Abort` (no fan:
  old fall-through kept, oracle-gated).
- D8 shaft speeds (√Δh_s ratio, the FROZEN definition) are `n_fan`/`n_hp` and are what P1/P6 score; the matcher's own
  speeds kept as `n_*_map`. (Advisor caught my post-freeze switch.)
- User decision: ONE nozzle rule — every engine (two-shaft `two_spool.rs` AND one-shaft `matcher.rs:376`) sizes A8 at
  the design's actual exit when unchoked at design. Before: bypass 0 vs 1e-9 jumped up to ~23 % thrust on low-π_c
  designs, and no-fan unchoked designs missed their own design point. **The full gate has NOT run since: if ANY
  recorded result moves, STOP and show the user before committing (their condition).**
- The one-shaft end now flies on rung 31 (`Shafts::One(OffDesignMatcher)`, standing on), not rung 32 on a flat map:
  rung 32 has no subsonic branch (map.rs § 3), so below choke it solved a choked nozzle (0.5 % off rung 31 at Mach 0).
  The anchor already said rung 31; last session's code had wired rung 32.
- `TwoSpoolEngine::run` at exactly v0 == 0 scores with `score_or_static` (every such call panicked before).
- User decision on gate 5: the frozen "model within 0.3 % of printed" MISSED (thrust 0.367 %) → printed, recorded as a
  MISS; asserted instead: model's reference τtH equals the τtH the printed τf, τcH, f, ηmH ask for (book prints 0.7580,
  0.2 % of HP work short) + the exact-(1+f) match. Spec must carry the breakdown (exact (1+f) ≈ −0.09 %, reference
  ≈ −0.12 % on F/ṁ0; ṁ0 0.13 %).
- A dry fan nozzle at design: worded assert in `TwoSpoolCore::new`.

**Next, in order:**
1. Gate 4b (`gate4b_unchoked_designs_return_home_and_bypass_zero_has_no_step`): home-bar now 1e-7 (worst seen 2.3e-10;
   reason in the comment) — passes; the FAILURE is equilibrium, design M0 π_c 2 π_f 1.2, flown to M0.5 Tt4 700: α=1e-9
   "did not settle" while α=0 runs. Diagnose the joint loop there (gain near 1? fan-split bracket?) — the named open
   risk. Don't widen the throttle grid to dodge it.
2. Full gate `rust\test-all.ps1` (matcher.rs changed ⇒ oracles). Then gates 6–8, P1–P8 (210 cells), CLI panel +
   golden, `sandbox-wasm/build.ps1`, gate, commit + push; then spec / CLAUDE.md row / notes / splice.

Kept on purpose: `W:\temp\claude\rung110-flown-turbofan\probe\` and `book\` (gate 5 sources, OCR). Related:
[[rung110-decisions]], [[needless-precision]].
