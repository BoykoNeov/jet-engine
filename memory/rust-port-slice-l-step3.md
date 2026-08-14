---
name: rust-port-slice-l-step3
description: "Slice L step 3 (rung 42, the bleed valve) — my smoke check witnessed ONE of the three methods the slice's own headline names, and a planned commit split turned out vacuous"
metadata: 
  node_type: memory
  type: project
  originSessionId: a55aa7ad-be31-4488-80c7-dbdf415c2bdd
  modified: 2026-08-14T14:34:52.476Z
---

Rung 42's interstage bleed valve ported into `rust/src/bleed.rs` (commit `e4fd2b6`). 448 Rust
tests green, every oracle bit-identical, 314 smoke rows bit-for-bit vs PyPy. Three process
lessons, none of them about the physics:

**1. A SMOKE CHECK THAT COVERS ONE OF THE THREE THINGS THE HEADLINE CLAIMS IS NOT A CHECK.** The
slice's claim — written into the crate's own header — is that all three of rung 41's schedule
methods reach rung 42's body through the hook. My first pass exercised `surge_margin` only,
because that is the one `bleed_trade` happens to call. The other two were the ones with untested
output: `flow_coefficient_turn`'s branch can FLIP under bleed and it carries the nullable columns,
and the two skipping methods are where I deliberately diverged from the source's spelling. The
advisor caught it. **How to apply:** when a claim names N things, count how many the check
touches before calling it wide — the widening is cheap, and the first pass covering 1 of 3 was
not visible from a clean diff. Same family as [[rust-port-inside-outside-exactness]] and
§ 5.8.2 (c)'s "a bit-exact agreement bounds the columns PRINTED, not the fields RETURNED".

**2. MEASURE THE DETECTOR, NOT JUST THE RESULT — IT WAS 2 %.** "0 mismatches over 314 rows" is an
observation until the check is calibrated ([[rust-port-slice-j]]: a 7 252-key bit-exact oracle
passed a deliberately mis-spelled square). Flipping ONE of three `(1-b)` associations —
algebraically identical, a different double — moved **7 of the 314 rows**, all at the sweep's
edges. So the defect class IS caught, by 2 % of the grid: a handful of cells would have passed it.
**How to apply:** flip one thing deliberately, confirm the diff goes nonzero, revert. It costs one
rebuild and converts a claim into a bounded one. See [[golden-fingerprint-gate]].

**3. A PLANNED COMMIT SPLIT CAN BE VACUOUS ONCE THE TYPE IS RIGHT.** The plan said the bleed
booking fields "go on the result as their own commit" — which presumes they land on the SHARED
result type, i.e. a change to gated code needing its own commit. They do not: Python never
*constructs* a bleed result at `b = 0`, so its dataclass defaults are unreachable and the caller
reads the ABSENCE through `getattr`. An `Option` on a NEW type makes the wrong write unwritable
and touches no gated type — so there was nothing to split. **How to apply:** a plan's commit
structure encodes an assumption about which code a change reaches; re-check it after the design
settles rather than following it mechanically.

Also worth keeping: the `b = 0` row is **vacuous for the thing it looks like it tests** —
`st_inlet == specific_thrust` and `mdot_core == mdot_air` numerically there, so every spelling
agrees. Gate the `None`-ness as a type instead, and sweep at `b > 0`.

Full record: `docs/plans/todo-rust-port.md` § 5.8.3 (a)–(h). Step 4 (the oracle + the two rung
suites) remains, and § 5.8.1's dump grid needs widening — it sweeps rung 41's schedules on
rung-39 matchers only. Related: [[rust-port-slice-l-step1]], [[rust-port-slice-k]],
[[rust-port-copy-vs-rederivation]].
