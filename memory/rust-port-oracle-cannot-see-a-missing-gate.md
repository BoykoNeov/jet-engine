---
name: rust-port-oracle-cannot-see-a-missing-gate
description: "A 100%-bit-exact oracle is silent on COVERAGE — enumerate the source test file's gates and diff before calling a rung ported"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 519adbe9-fbbb-49f5-9712-29d4dd7fed36
  modified: 2026-08-12T18:18:39.805Z
---

Phase 4 slice H shipped **6 of rung 29's 16 gates** and looked complete: its oracle read 270/270
bit-exact and every ported test passed. The ten missing ones were two whole families — the `π_c`
and `M0` margin sweeps that re-check the rung's "one design point" concession on the axes it named.

**The cause was a dropped step, not a wrong judgement.** Slices F and G each began by reading
`test_rungN.py` and enumerating its gates. Slice H was written from `gas.py`'s docstrings, the
spec, and the *header comment* of one test file — so `test_rung29.py`, the largest test file in the
phase at 422 lines, was never opened. It received the smallest Rust suite in the phase.

**Why nothing caught it:** an oracle gates VALUES. Bit-equality proves the port computes what the
Python computes; it is completely silent on whether anything asserts what that computation is
*for*. The oracle's own key-count guard compares the dump against the test, so it cannot see a
claim absent from both. Same blind spot as [[rust-port-documented-gate-that-doesnt-exist]] from the
opposite side: there a gate was described and absent, here a gate existed in the source and was
never transcribed.

**How to apply:** before calling a rung ported, run `grep -n "^def test" tests/test_rungN.py` and
diff it against the Rust test names. One command, and it is the ONLY detector for this class. Read
the test file itself, never just its header — a header summarises gates, and summaries drop clauses
(slice H also lost two clauses of rung 30's gate 3 that way). Treat "the oracle is 100 %" as
evidence about arithmetic and nothing else. Related: [[rust-port-ported-test-vacuity]],
[[rust-port-guessed-census-bars]].
