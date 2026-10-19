# Low-start repair probes (sandbox plan § 37; archived here — the worktree and scratch crate they ran in were temporary and are deleted)

Base commit: `534f69e` (the code before the repair). The `eq_*` / `loaded` probes ran as test files in a throwaway
worktree of `534f69e`, calling `try_equilibrium` with EXPLICIT starts (the repair did not exist there). The rest are
`[[bin]]`s of one scratch crate (`crate_Cargo.toml.txt`), built against the repo after the repair and — for the
"before" files — against that worktree.

- `eq_start.rs` → `eq_start.txt` — the full-speed start vs the steady matcher on the probe grid: which refusals a lower
  start rescues (31 at first, `B … rescued`).
- `eq_ladder.rs` → `eq_ladder.txt` — the ladder 0.9 … 0.4: where it solves it lands on the steady point.
- `eq_unique.rs` → `eq_unique.txt` — (a) the full-speed path already returns 41 below-idle points; (b) every low start
  that solves reaches one root (worst spread printed); the below-idle rescue candidates the filter refuses.
- `loaded.rs` → `loaded.txt` — the customer-bleed engine at 11 km, Mach 1.6/2: the steady matcher accepts CHOKED
  below-idle points (−7 to −16 N), which is why the gate's `judge` asks only for running steady points to be solved.
- `red_check.txt` — the gate with the low start switched off by a temporary env flag (since removed).
- `census_tail.txt` — the end of the full-gate census with the repair on (only the stale page and one Controls test).
- `design_low.rs` → `design_low_before.txt` / `design_low_after.txt` — the design flight, flat-LP, 440–540 K: steady
  matcher, equilibrium, page. 480–510 K were refused before and run after, at the steady speeds; 470 K and below are
  below idle (restored source: the scratch file was reused for the next probe).
- `floors.rs` → `floors_before.txt` / `floors_after.txt` — the page's lowest starting throttle per map and flight
  (after: with the refusal just below each floor — every one now "below idle").
- `thrust_floor.rs` → `thrust_floor.txt` — thrust per kg/s at and above each fast floor: at Mach 2.6 (and Mach 2 on
  two maps) the full-speed start already accepts below-idle points (restored source, as above).
- `lever_cases.rs` → `lever_cases.txt` — the two lever cases `sandbox_controls` held refused ("does not settle", the
  acceleration gap): every start that settles lands on one root; both run now.
- `crash.rs` → `crash_summary.txt` — seed 36, 1 200 random Controls requests through `sandbox::call`, before vs after
  (`crash.rs` is the HP-wall repair's, unchanged). `crash_moved.tsv` (index, old reply, new reply) — the 15 that moved;
  `crash_offmap_remaining.tsv` (index, request, reply) — the 3 still "off the maps".
