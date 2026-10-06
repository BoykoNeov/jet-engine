---
name: sandbox-slice1
description: "Web sandbox slice 1 SHIPPED 2026-10-06 (docs/sandbox/) — process lessons: screenshots found two layout bugs 122 green checks missed; a browser-vs-native bar measured on a 2-output spike was 4 decades too tight; tie a tolerance to its MECHANISM and compare zero-crossing quantities absolutely"
metadata:
  node_type: memory
  type: project
  originSessionId: 37d8e91c-92f4-4849-9e51-b3819f0a71fe
  modified: 2026-10-06T17:39:05.178Z
---

Slice 1 of the web sandbox ([[sandbox-direction]]) shipped 2026-10-06: `docs/sandbox/turbojet-sandbox.html`,
one offline file with the Rust model as embedded wasm. Plan `docs/plans/sandbox-plan.md`; how it is
built and gated: `docs/sandbox/README.md`. Next: slices 2 (rung 85 blade knobs) and 3 (off-design),
user said "both, either order".

**Process lessons.**
- **LOOK at the page.** 122 checks were green (Rust joints, wasm-vs-native grid, a 10-step headless
  Chrome drive) while the page had two visible bugs: `display: grid` on `.knob` overrode the `hidden`
  attribute (both efficiency spellings shown), and on a phone the tiles/results column sized to their
  min-content and overflowed. Only CDP screenshots (light, dark, 390 px) found them. A behaviour test
  is blind to layout; take screenshots before calling a page done. Same family as
  [[visuals-model-binding]]: "in sync" is not "right".
- **A bar measured on a toy is not a bar.** The spike (2 outputs, mostly closed-form) showed
  browser≡native to ~1e-15; the real 91-design grid showed 2.35e-11 on every TABLE gas. The cause is
  mechanical: `gas::SOLVE_TOL` = 1e-11, and the two builds' Newtons stop at different iterates inside
  it. So the bar is 100× SOLVE_TOL (1e-9), the perfect gas keeps 1e-13 — measure on the real grid,
  then STATE the mechanism the bar answers to ([[rust-port-measure-before-registering]]).
- **Compare a quantity that can be exactly 0 absolutely.** Entropy is a difference from the ambient
  datum; the exhaust curve ends ON it (native 0, browser −2e-13 → "100 % relative"). Classify by
  what the number IS, not by a key-name regex that missed the curve arrays.
- **Tamper once to prove the check can fail** — altering one native TSFC turned the grid red
  ([[instrument-fed-by-what-it-certifies]]).
- **The perfect gas's ideal compressor is not vertical on T–s**: rung 1's rounded constants
  (cp 1004 vs Rγ/(γ−1) 1004.5) leave exactly `(cp − Rγ/(γ−1))·ln(Tb/Ta)`; the test asserts that
  closed form instead of loosening the bar.
- **Backslash hazards re-hit 4×** despite [[windows-tooling-file-hazards]] — see that file.
