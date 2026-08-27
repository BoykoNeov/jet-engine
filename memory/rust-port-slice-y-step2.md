---
name: rust-port-slice-y-step2
description: Slice Y step 2 — a few-ulp mismatch that looked like a port defect was the probe harness spelling a gas constant differently from every suite; re-run the committed dump first
metadata: 
  node_type: memory
  type: project
  originSessionId: cc4ae781-acf8-4650-a2a9-99b2cb27665e
  modified: 2026-08-26T18:44:55.031Z
---

The rung-65 port's first smoke run missed by 5–7 ulps. Bisecting the trajectory put the first
divergence *before* the march, in shipped rung-64 code that slice X had certified bit-exact over
318 keys — i.e. it looked like a regression in already-gated code.

**IT WAS THE PROBE, AND THE ONE-COMMAND CHECK THAT PROVES IT SHOULD COME FIRST.** Re-running the
nearest committed dump (`rust/oracle/dump_slice_x.py`) against its TSV gave **0 of 318 differing**,
which clears the shipped code immediately. The cause was in my own probe files: they wrote the gas
constant as `R_c = 0.4/1.4*1004.0` where every suite and every dump writes
`(1.4 - 1.0)/1.4*1004.0`. **`1.4 - 1.0` is not the double nearest `0.4`.** One ulp in `R_c` moved
`nu0` by seven ulps and the march's minimum by five. Six probe files carried it.

**Why:** this is [[rust-port-slice-s-step3]]'s lesson one level down — there the probe's header
claimed the suites' *grid* and its code ran another; here the grid was right and the *gas* was not.
A hand-copied constant is a re-derivation, and re-derivations of floating-point expressions are not
equal to copies of them.

**How to apply:** copy constant EXPRESSIONS from the suite character-for-character, never simplify
them (`(1.4 - 1.0)/1.4` is not `0.4/1.4`). And when a port disagrees with an anchor by a few ulps,
re-run the nearest committed dump against its TSV BEFORE touching the port — a green diff moves the
search into the harness in one command.

Related: [[rust-port-slice-y-step1]], [[rust-port-slice-y-step3]], [[golden-fingerprint-gate]].
