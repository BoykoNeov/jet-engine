---
name: pypy-switch-shipped
description: "The 2026-07-31 PyPy adoption (slice 4): full gate 17:27 -> 2:47 (6.2x). SLOW_SECONDS deliberately NOT rescaled — under CPython it bought TIME, under PyPy it buys DETERMINISM. psutil is load-bearing. main.py's one moved line is a printed RESIDUAL, and print precision protects states, not residuals."
metadata: 
  node_type: memory
  type: project
  originSessionId: 26837c68-090b-4326-a309-885d38daeae7
  modified: 2026-07-31T10:37:08.279Z
---

PyPy 3.11 (v7.3.23) is the project's interpreter as of 2026-07-31, via a repo venv `.venv` built
from `M:\claud_projects\tools\pypy3.11-v7.3.23-win64`. Plan + full write-up:
`docs/plans/todo-pypy-switch.md` § 4. Slices 0–3 built the detector first
([[golden-fingerprint-gate]], [[golden-gate-slice2]]); slice 5 (collapse the three-gate policy)
stays open. Corrects [[perf-sonic-throat-and-pypy]]'s "NOT adopted"; retimes
[[test-suite-speed-policy]].

**Delivered: `--runslow` 1047.5 s → 167.8 s (6.2×)** on the identical 1002 tests, bare `pytest`
79.8 s. The plan predicted "2:37–2:50" and measured 2:47.

**The judgement call worth keeping — I refused BOTH options the plan offered.** It said: rescale
`SLOW_SECONDS` 8.0 → ~1.6 s to preserve the partition, *or* accept the re-cut. Both are wrong on
the data, and the reason generalises:
- The **budget stopped binding** — every candidate threshold fits, because the full gate is now
  faster than the old fast subset. Preserving a partition optimises a constraint that expired.
- **The rescale is not even a cost cut**: at 1.2 s the duration route tags 167 tests, at 0.5 s
  249 — against CPython's 159. *More*, not fewer.
- **1–3 s is the NOISE BAND.** PyPy attributes JIT warm-up to whichever test first touches a code
  path on a worker, so trivial tests record seconds they do not cost (measured: 0.00 → 2.67 s,
  0.35 → 4.70 s). A threshold there flips tags run-to-run.
**So the constant survived unchanged, with its reason INVERTED: under CPython 8.0 bought TIME;
under PyPy it buys DETERMINISM.** Generalise: *when a speedup makes a tuned constant's original
justification vacuous, look for a NEW justification at the same value before assuming it must
move.* Keeping a number can be the active decision.

**A new, permanent property of the interpreter — disclosed, not fixed:** the learned-duration
cache is now **schedule-dependent** (the JIT-warm-up attribution above). Both the fast subset and
`--affected` read it, so a test near *any* threshold can flip side between runs. Nothing sits near
8.0 s so it does not bite today — recorded in `conftest.py` rather than omitted for being
currently harmless.

**`psutil` is LOAD-BEARING, and nothing in the tree would have revealed it.** pytest-xdist counts
PHYSICAL cores via psutil and silently falls back to `os.cpu_count()` — LOGICAL — without it. On
this 8-physical/16-logical box that turns `-n auto` into `-n 16`, a config measured (1.30×) and
explicitly DECLINED. CPython's environment happened to carry psutil, so **the decline held by
accident for months.** It is now pinned in `requirements.txt` with the reasoning inline.
Generalise: *a declined decision that holds via an undeclared transitive dependency is not
actually decided — it is lucky.*

**`main.py` diff (the thing no test covers): 151,249 bytes both, exactly ONE line moved** — the
rung-32 panel's `tau_c rel`, `7.0e-10` → `6.9e-10`. That is the only printed quantity that is a
**convergence residual** rather than a physical state. **Print precision protects a state (2–4
s.f. sits decades above ~1e-6 drift) and gives a residual NO protection, because there the
printed value IS the noise.** So the ~1e-3 print-precision sensitivity bound covers every
physical number `main.py` emits and does not extend to residuals; the claim that line supports
("map-free to ~1e-6") survives by three decades on both interpreters. The `ts_diagram.png` size
change (83,057 → 80,058 B) is a matplotlib build difference, not physics.

**The seed set was regenerated (61 → 27 pairs; 42 stale, of which 20 had always been inert
because `_is_spine` overrode them),** with a ONE-TIME filter dropping anything whose CPython
duration was < 1 s so a JIT-warm-up artefact could not freeze into the cold-cache path. That
filter is not an invariant — post-switch tests have no CPython duration to check.

**⚠ And the check I ran on it was a TAUTOLOGY, which I wrote up as validation in three places
before the advisor caught it.** A cold cache deselects 224, identical to warm — but the seed was
regenerated as *exactly* `pypy >= SLOW_SECONDS`, so cold (seed only) and warm (seed OR learned)
agree BY CONSTRUCTION. It confirms the regeneration was *applied*, not that the threshold is
*right*. **Generalise: when you build an artefact from a predicate and then test that the
artefact agrees with that predicate, you have tested your typing, not your judgement.** The
tell was available in my own table — `8.0s -> 30 (duration-only: 29)` — and I read the
coincidence as a property of the threshold when it was a property of how I had just built the
seed. The real case for 8.0 (noise band, 0-newly-slow asymmetry, rescale-is-backwards) never
needed it.

**The one remaining CPython need is NOT a violation of "no CPython dependency":** running the
project (model, plot, all three gates) is PyPy-only. *Regenerating* the goldens still needs
CPython because they are a committed CPython **anchor** — a property of the anchor, enforced by
the provenance guard's `meta.implementation`. Goldens were regenerated on CPython **before** the
switch, delta predicted first and verified after (0 moved / 2 removed / 0 added).

**Two process notes.** `/usr/bin/time` does not exist in Git Bash — a background launch using it
dies instantly and the harness still reports "completed (exit 0)" because the *pipeline*
succeeded. Second occurrence of this class this session (the first was `tasklist /FI` mangled
into a path); **verify a background run is alive before spending the window waiting on it.**
And CLAUDE.md's own `~13 min` full-gate figure was **stale, not contaminated** — CPython's real
gate was 17:27, corroborated by summed call-time ÷ 8 workers ≈ 1022 s.
