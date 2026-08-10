---
name: golden-gate-slice4
description: "Fingerprint slice 4 (rungs 78/79) — a bit-exact arm is the strongest form when the rung's finding IS a set of zeros; lead the arm with the reader that BYPASSES the plant's short-circuit"
metadata: 
  node_type: memory
  type: project
  originSessionId: cf30db3f-2b60-4f60-9eb0-f64d420e03c8
  modified: 2026-08-10T09:14:13.930Z
---

Fingerprint slice 4 shipped 2026-08-10 (commit `5b0d62a`; the rung-79 `main.py` panel that
shipped beside it is `0e568c0`): 2 arms (`r78` `gauge_scan`+`root_census`+`gauge_vs_device`,
`r79` `coord_forced`+`coord_scan`+`coord_census`+`coord_march`), 2,724 new values, golden
37→39 kernels, 18,573→21,297 values. Slice 4 declares NO settings of its own — rung 79 § 0.2
takes rung 78's verbatim, which takes rung 77's — so the two arms are differenceable against
each other and against slice 3. See [[golden-gate-slice3]], [[golden-fingerprint-gate]].

**Both arms measured EXACT** — 0 of 2,724 values differed between CPython 3.14.3 and PyPy 3.11.15
(r78: 2,393 values / 1,769 floats / **1,068 distinct**; r79: 331 / 224 / **93 distinct**). The
distinct counts are what make that exactness credible: a vacuous probe returns few distinct floats,
and the `ALL-SAME` flag did not fire. So both ship at `TOL = 0.0` (bit-equality, no `ABS_TOL` leg),
joining `cpg`/`r66`/`r68`/`r77`. No sensitivity sweep — an exact arm already sits at the ulp floor,
so there is no band to consume.

**The load-bearing choice: which reader LEADS the arm.** On rung 79's plant `_cap_free`
short-circuits to `_surge_fuel` exactly when the leg binds, so `coord_scan`/`coord_march` report
structural zeros *by construction* — an arm built on them alone would pin hundreds of values and
guard nothing about the coordinate. `coord_forced` bypasses the short-circuit and holds the only
live float differences (6.1e-15 / 7.6e-16). It leads. This is [[golden-gate-slice3]]'s vacuity
lesson one level up: not "is the arm empty?" but "is the arm's CONTENT the thing the rung claims?"

**Why bit-equality is right here specifically (r79):** its zeros ARE rung 79's finding
(`d_max = 0`, `d_set = 0`, `n_both = 0` — see [[rung79-state-coordinate]]). Loosening that arm
would loosen the CLAIM, not a noise band. That inverts the usual reading of a tight tolerance as
mere strictness.

**The one thing that went wrong:** a BACKGROUNDED regeneration got nothing from `git rev-parse
HEAD` (sandbox, or the 30 s timeout under load) and wrote `repo_sha: ""`. The writer swallowed it
(`except: pass`), so it surfaced only two steps later in `test_golden_file_declares_its_provenance`
— i.e. after the 18-minute run had already written the file. The VALUES were fine; only the
provenance was. Fixed by filling in the HEAD that was current during the run (it cannot have
changed — the run does not commit) and by making `_regenerate()` print a loud warning at the
moment the sha comes back empty, where the operator is still watching.

**NEVER EDIT A SOURCE FILE WHILE ITS SUITE IS RUNNING.** I edited this module's docstring during
the gate; `test_every_kernel_is_actually_GATED` uses `inspect.getsource`, which re-reads the file
from disk at line numbers cached at import, so the shifted lines handed its regex the wrong text
and it reported **12 long-standing kernels as ungated** on a tree that was green. The tell was that
the list included arms untouched for slices (`C`, `E`, `r12`, `r25`) — when a detector accuses
things you did not touch, suspect the detector. Re-running on a stable file: 4 passed in 2.75 s.

**How to apply:** (a) a CPython pre-check that passes proves reproduction *within tolerance*, not
bit-identity — arms with nonzero `TOL` can pass while their last bits moved, and `_regenerate()`
writes unconditionally, so the recovery from a nonzero "moved" count is `git checkout tests/golden/`,
never acceptance; (b) confirm the splice with `git diff tests/golden/` — the ONLY `-` line may be
`repo_sha`; (c) reconcile the added-LINE count against the value count before believing it (floats
are stored as a 3-line `{"f": "0x…"}` object, ints as 1 line); (d) leave `every` at the readers'
default — see [[golden-gate-slice3]] on the stride knob.
