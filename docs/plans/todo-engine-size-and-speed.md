# Two engineering questions: `engine.py`'s size, and a Rust rewrite

Not a rung, not a seam. Measured 2026-07-30 on the rung-66 tree. Both answers turned out
to be the opposite of the question's framing, so the numbers are recorded here.

## Q1 — is `engine.py` (11 312 lines) a maintenance problem? Is a split worth it?

**Measured shape**

| | lines |
|---|---|
| total | 11 312 |
| docstring | ~3 604 |
| comment | ~1 109 |
| blank | 1 322 |
| **actual code** | **~5 277** |

42% of the file is prose. The largest classes are `TwoSpoolFuelTransient` (1 488),
`ScheduledStatorTransient` (1 022), `ScheduledBleedTransient` (857), `SpoolTransient` (717),
`TwoLagCascadeTransient` (652).

**The finding: a split cannot reduce the coupling, because the coupling is not the file.**
It is a 9-deep linear inheritance chain

```
TwoSpoolMatcher → TwoSpoolMapMatcher → TwoSpoolTransient → TwoSpoolFuelTransient
  → ScheduledStatorTransient → ScheduledBleedTransient → LimitedBleedTransient
  → LaggedBleedTransient → TwoLagCascadeTransient
```

plus one diamond (`StatorBleedMatcher(TwoSpoolBleedMatcher, VariableStatorMatcher)`).
Moving those classes into eight files leaves every `super()` call exactly where it was and
adds eight import edges. **That chain IS the reduce-to-prior contract made executable** — each
rung subclasses its predecessor and overrides one thing, which is what makes
`test_reduce_*_bit_for_bit` a two-line assertion. Refactoring it to composition would delete
the project's spine. So: no restructuring.

**What a split WOULD buy is navigation, not maintainability** — reading the tail of the chain
out of an 11k-line file, every rung session. That is a real cost, and it is a *pure file move*:

- **175 files import from `turbojet.engine`** ⇒ `engine.py` must survive as a re-export shim so
  not one import line changes anywhere.
- Cut/paste only — no renames, no signature changes. The reduce spine
  (`test_reduce_*`, `test_cycle_untouched_*`, `*_bit_for_bit`) is what proves the move inert.
- `CLAUDE.md` § Layout maps names to `engine.py` and is under a byte-budget guard test.
- It rewrites `git blame` boundaries on the file that is the record of 66 rungs.

Verdict: **optional, low value, non-zero risk.** Not recommended now.

## Q2 — would a Rust rewrite buy meaningful performance?

**The profile says the question is misaimed.** One representative transient march
(`TwoLagCascadeTransient._stator_march`, rung-66 settings), 5.3 s clean / 18.1 s under cProfile:

| function | share |
|---|---|
| `components.py:_sonic_throat` (cum) | **85%** |
| ├ its `resid` closure | 12 115 710 calls |
| └ `gas.h_t` / `gamma_t_at` (CPG closed form) | the rest |

Corroborated at 24× the scale on the whole of `test_reduce_valve_lag_alone_is_rung65_bit_for_bit`
(417 s under cProfile, 1.70e9 calls): `_sonic_throat` cum = **88%**, 6 627 268 calls,
298 227 060 `resid` evaluations — the same 45-per-solve.

269 238 `_sonic_throat` calls ⇒ **~45 residual evaluations per solve.** That is because the
solver is a **bisection to `1e-13*Tt`** (`components.py:517`). On a CPG gas the residual
`cp·(Tt−T) − ½·γ·R·T` is **linear in T** — the code bisects 45 times to find a linear root.

Two things this rules out:

- **Memoization is dead.** `choked_mfp` is a pure function of `(gas, Tt, far)`, but measured
  argument reuse is **1.1×** (234 789 unique of 269 242). A cache buys nothing.
- **The equilibrium gas is not the cost.** A full `reacting_equilibrium` design-point run is
  **60 ms** (`_equil_solve` + `_gauss_solve` dominate, but the total is negligible). The
  expensive gas is not on the expensive path; rungs 31–66 run on CPG.

**Measured lever, pure Python:** replacing the bisection with a secant/Newton iteration
(finite-difference derivative, same bracket, same tolerance) gave

```
baseline       5.31 s
newton-secant  1.94 s      → 2.74x on the whole march
```

An analytic derivative, or the CPG closed form `T*/Tt = 2/(γ+1)` the docstring already says it
reproduces to machine precision, would do better still. `_sonic_throat`/`choked_mfp` has 35
call sites and is on every path from rung 30 up.

**Ladder, in effort order (Rust last):**

1. **Fix the bisection** — ~2.7× measured, ~40 lines, no new dependency. Must be gated on the
   full reduce spine: it changes last bits, so every `*_bit_for_bit` test is the real cost.
   (A CPG-exact closed form would be bit-*different* from bisection; this is the blocker,
   not the physics.)
2. ~~`pytest -n auto`~~ — **ALREADY DONE.** `pytest.ini` runs `-n auto --dist load
   --maxschedchunk=1` with LPT ordering from learned durations. There is no parallelism win left
   to take. See the budget below.
3. **PyPy** — unusually viable *because* there is **no numpy anywhere** in this repo. Pure
   stdlib scalar float loops is PyPy's best case (typ. 5–15×, zero code change). matplotlib is
   the friction, but `main.py` is its only consumer — the suite could run on PyPy.
4. **Rust/PyO3 on `_sonic_throat` alone** — justified only after (1); the kernel is ~40 lines,
   so this is a targeted extension, not a rewrite.

## Where the 28 minutes actually goes (measured full gate, 2026-07)

973 passed in **1698 s (28:18)** wall — already under `-n auto` on **16 cores**.
Learned durations (`.pytest_cache/v/durations/call`, 984 entries):

| | |
|---|---|
| serial sum (total CPU) | **229 min** |
| parallel wall clock | 28.3 min ⇒ **8.1× effective** on 16 workers |
| longest single test | 374 s (6.2 min) — the LPT makespan floor |
| top-10 tests' share of the sum | 20% (the load is broad, not pole-bound) |
| **rung ≥ 30 files** (the `_sonic_throat` path) | **165 min — 72%** |
| rungs 7–29 (mixing-PDF / NOx diagnostics) | 64 min — 28% |

Heaviest files: rung63 (1532 s), rung24 (1314 s), rung23 (886 s), rung59 (843 s), rung66 (681 s).

**Consequence.** The wall clock is throughput-bound (broad load on 16 cores), not pole-bound, so
the only remaining lever is **less total CPU work** — which is exactly item (1). A 2.7× on the
`_sonic_throat` path would cut ~140 of the 165 structural minutes to ~52, i.e. serial sum
229 → ~141 min, projecting a wall clock near **17–18 min**. Rungs 23/24 are untouched by it —
they are mixing-PDF integrals in `gas.py`, a separate kernel.

**A full Rust rewrite is refused on the project's own terms.** `CLAUDE.md`: *"The deliverable
is understanding, not the tool."* The 42% prose ratio is the evidence — this is a document as
much as a program, and the reduce spine is a Python-inheritance artifact. A rewrite trades the
deliverable for wall-clock that items 1–3 already buy.
