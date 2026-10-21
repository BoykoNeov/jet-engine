# Rung 34's low-speed wall — what the repair's numbers were measured with (2026-10-21)

Scratch crate (`Cargo.toml` points at the repo; during the work it was built against the unchanged repo for "before"
and against a throwaway worktree for each trial). Spec: `docs/rung34-spec.md` § The low-speed wall; sandbox plan § 42.

* `fc.rs` — the Fly-it crash map: 800 seeded requests (seed 34; 70 % fly / running point, 30 % slam; random gas, map,
  flight, throttle 550–1500 K, 10 % afterburner) through the page's own `sandbox::call`. Outputs (4.7 MB each, not
  kept — rerun): before, 46 refusals, every other reply the baseline; trial "everywhere", 18 refusals run, 395 of
  754 running replies differ at ≤ 1e-9; the shipped "old search first", the 754 byte-identical, the same 18 run, 28
  refusals byte-identical but one (a lit afterburner whose dry point now solves; same plain words).
* `trial-everywhere.diff` — the rejected trial: the step lifted inside every instant. Its gate failures: 1 446
  `combustor_oracle`, 2 783 `spool_oracle`, 335 `slice_q_smoke` keys, five `slice_p_smoke` tests (the Illinois count
  190 vs 227), and `sandbox_fly`'s "No shaft speed" case (now "spool down"). Largest smooth move ~4e-9 relative.
* `q7.rs` — the cause: at `ν` 0.30–0.70 the choked-star nozzle refuses while the subsonic solve runs.
* `q8.rs` — the page map's `Φ` maximum over a fine speed grid at the throttles that still refuse (650/675 K: none
  positive), and the flat map vs rung 33's working root (to ~1e-13).
* `q9.rs` — the 18 requests that now run: each on the stable point (`Φ` ±0.01 around it); the words of the 28 left.
  Reads `now-run.tsv` / `fc-*.tsv` (regenerate with `fc`).
* `q10.rs` — 3 000 wide random fly requests (throttle 300–900 K): which plain words each refusal shows; "No shaft
  speed…" stays reachable.
* `q11.rs` — the lowest running throttle on the page map, hot day, bisected: 728.0 → 675.4 K (thermally perfect),
  736.5 → 679.6 K (perfect); every throttle above runs.
* `q12.rs` / `q13.rs` — Fly it with an afterburner: its copied search kept the wall; then its subsonic solve's lower end
  (the choke root) did too. Unlit, 725 K now runs on that day; 700 K is below idle there (`Φ` at best −0.004).
* `q14.rs` — q10's sweep one line per request, run on the shipped code and on each version: the repair (before the
  below-idle rule) turned 82 refusals into runs and 20 ("spool down" 11, "no shaft speed" 9) into "the efficiency
  bookkeeping failed" (65 → 85); with the rule all 85 read below idle and nothing else moves.
* `q15.rs` — those 85 operating points: every one a settled shaft with NEGATIVE thrust (Mach 0.5–2), where the scoring's
  efficiency check cannot pass (its tolerance is a fraction of a negative number).
* `q16.rs` — round-number below-idle cases for the test (sea level, Mach 0.6, 450 K, perfect gas).
* `q17.rs` — the running-line chart (`running_point_json` over `running_line_grid`) at six flights, two gases, three
  maps, shipped code vs the repair: identical (600 points, none with negative thrust; the grid's 75 K steps miss
  the throttles the repair opens).
* The slams: in `fc`'s replies an `"ok":1` slam can carry a stop. With the first commit three throttle-up slams from a
  low start stopped at step 1 (the march kept the wall); with the page's march switch two run to the end and one
  stops at the fuel cap at step 3 (a fast fuel-metered ramp); no other reply moves; no slam endpoint has negative
  thrust (555 endpoints).
