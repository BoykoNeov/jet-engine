# Altitude on the Controls view — scratch scripts and their outputs (sandbox plan § 27)

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\jet-altitude-probe`, deleted after the work).

- `sim.rs` → `sim.txt` — where the design flight sits in the standard atmosphere (5 579.3 m, −1.92 K); the pressure
  similarity at ×½ / ×2 through `build` (bit-exact: powers of two); the default move from −1 to 20 km.
- `stepcheck.rs` → `stepcheck.txt` — every step-checked knob at its design-flight limit, 0.02 against 0.005, at the design
  flight, sea level −40 / 0 / +40 K and 15 km, on two maps.
- `govgrid.rs` → `govgrid.txt` — the governor's gain and time constant over a flight grid, with the end airflow over the
  design's beside (the error follows it).
- `rulecheck.rs` → `rulecheck.txt` — the density rule checked AT the limits it sets: 720 governed runs, 0–20 km, ±40 K,
  four maps, four laws; the worst 25 and the runs above 1e−4 that never touch a fuel edge.
- `crash.rs` → `crash.txt` (seed 14, −1 … 25 km, ±45 K) and `crash_wide.txt` (seed 15, 400 requests, −5 … 47 km,
  ±100 K) — the crash map through `sandbox::call`; refusals tallied by cause and altitude band.
- `lowest.rs` → `lowest.txt` — the lowest throttle with a steady point per map and flight, and with a fixed generator or
  customer amount (the measurement behind the reworded "off the maps" words).
- `highend.rs` → `highend.txt` — the HIGHEST workable throttle in cold air high up (the off-map words' other side);
  `crash.rs` re-run → `crash_detail.txt` lists each off-map and does-not-settle refusal with its throttles, air and loads.
