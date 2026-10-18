---
name: rung99-speed-axis
description: "Rung 99 (Mach axis on the control tables) process lessons — compute a derivation from the model's own state, not a hand estimate (advisor caught a coefficient slip); a quantity zero in algebra needs an absolute bar; compile tests before a 15-min gate; JS template literals eat backslashes"
metadata:
  node_type: memory
  type: project
  originSessionId: 88c81e76-8dcf-41fb-90c3-5b92afee2032
  modified: 2026-10-18T13:09:07.119Z
---

Rung 99 (2026-10-18): `mach_axis.rs` — eight grid slices blended at equal reading, or the thrust recomputed (formula);
anchor `7113ed3`; 116 HIT / 0 MISS of 116. Follows [[rung99-decisions]], [[rung98-corrected-tables]].

**Lesson 1 — derive from the model's own state, not a hand estimate.** My first anchor draft estimated the bend of
thrust-vs-Mach by hand (`d²/dM²(1+0.2M²)^−3.5 ∝ 2.32M²−1`, a factoring slip; true `1.6M²−1`) and froze a "2× margin" at
Mach 1.4. The advisor's pre-freeze review caught it. Re-derived ANALYTICALLY from engine state + the model's nozzle area
+ its inlet (G5, a state print, no table read): the margin was 1.1–1.4× at the low throttle and zero at the high one,
so that cell was unscored — it came out −0.04 %. Disclose that such a print is close to the predicted quantity: the
116/116 then tests only what it neglects (temperature residual, calibrated end), measured within 0.02 points of it.

**Lesson 2 — a quantity that is zero in algebra needs an absolute bar.** The formula's gap "rest" is 0 while choked and
~1e-13 noise in arithmetic; the browser comparator's RELATIVE bar read two noise values as 190 % apart. Gave it its own
absolute rule in `check.mjs` (worst 8.1e-13, bar 1e-10), documented beside the entropy/settled-gap rules.

**Lesson 3 — `cargo test --no-run` before launching the gate.** A settings round-trip test lists every field by hand;
the new field broke its compile and the first 15-minute gate died at the build. Compile the tests first.

**Lesson 4 (tooling) — browser.mjs code inside a JS template literal loses single backslashes** (`\s` → `s`, `\.` →
`.`): write `\\s`, `\\.`, as the surrounding checks do. Python heredoc edits add their own escaping layer on top — check
the written file.

**Census held exactly as pre-registered** (2 of 83 check requests; crash map 76 diffs, 0 outside the class); three
newly refused crash requests were each read — slow slices' own reach (a fixed customer amount; rows with no operating
point at Mach 0.3/0.6), not bugs.
