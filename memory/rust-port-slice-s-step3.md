---
name: rust-port-slice-s-step3
description: "An injection that compiled and applied could not have moved anything; and a suite's blindest gate was the one no injection was aimed at"
metadata: 
  node_type: memory
  type: project
  originSessionId: c19578b9-e1b7-420f-b913-b38a3cfcc4bd
  modified: 2026-08-19T06:24:06.085Z
---

Slice S step 3 of the Rust port (2026-08-19) — `tests/rung45.rs`, 10 tests / 0 failed, name diff
726 → 736 with 0 removals, exactly as pre-registered.

**The lesson: COMPILING IS NOT EXPRESSING.** The file passed 10/10 first try, so thirteen
injections were run to give it teeth. One of them — seed `ext_lp` with `phi_surge * 1e-9` to break
the read-only claim — applied, compiled, and reported **zero gates firing**. The march loop is
`if e_lp.abs() > ext_lp.abs()`, so the first point overwrote the seed: it could not have moved
anything. I had written [[rust-port-slice-s-step1]]'s *"two injections reporting nothing moved
could not have moved anything"* myself, two steps earlier in the same slice, and still built the
instrument that repeats it. The harness's "applied?" column checked `count(old) == 1` and a clean
build — neither is evidence of numerical expression.

**How to apply:** an injection harness needs a THIRD check beside "it applied" and "it compiled":
that the perturbed quantity is still perturbed at the point the gate reads it. Prefer perturbing
the RESULT over seeding an accumulator, and when a zero comes back, ask what overwrites the change
before assuming the gate is blind. Keep the dud in the harness as a labelled row rather than
deleting it — it is the only thing that makes the neighbouring zeros readable. Same family as
[[rust-port-slice-r-step3]] and [[rust-port-slice-s-step2]]'s line-number injection.

**A second, cheaper lesson: measure a zero before calling it a hole.** Dropping the `r` from the
march bound `r + s_settle` fired nothing. Measured rather than filed as a gap: it moves `npts`
(351/326/316/306 → 301) and moves the physics by **exactly zero, bit-for-bit at all four ramp
rates** — the minimum is attained during the ramp, never inside the settle tail. So the suite is not
blind; the change is physically inert, and the only channel that witnesses it is a key no gate
reads. That is a concrete instruction to the value oracle, not a defect.

**And a third: the source's own gate armed its object to dodge a different assert.** Python's
`lp_disabled` test builds two objects, the second with both maps armed, and asserts only
`pytest.raises(AssertionError)`. Measured: `transient_surge_margin_fuel` reads its surge-line assert
FIRST, so the arming is what makes that half test the assert it names — unarmed, it would have
passed having exercised a completely different one. The port asserts WHICH refusal escapes.
See also [[rust-port-ported-test-vacuity]]: rung 55's inherited gate had to be REBUILT rather than
ported, because Rust's by-value constructors delete the shared-object channel its `==` depended on.
