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

1. **Fix the bisection** — ✅ **DONE** (see § Outcome below). Measured **5.4×** on the march and
   **28:18 → 13:09 (2.15×)** on the full gate — better than the 2.7× the secant experiment
   projected, because the CPG root is *closed-form*, not merely faster to converge to.
2. ~~`pytest -n auto`~~ — **ALREADY DONE.** `pytest.ini` runs `-n auto --dist load
   --maxschedchunk=1` with LPT ordering from learned durations. There is no parallelism win left
   to take. See the budget below.
3. **PyPy** — ✅ **MEASURED** (see § Outcome of item 2 below). **5.1–5.3×** on the full gate,
   zero code change, and — the load-bearing result — **bit-identical trajectories**.
4. **Rust/PyO3 on `_sonic_throat` alone** — justified only after (1); the kernel is ~40 lines,
   so this is a targeted extension, not a rewrite.

## Where the 28 minutes actually goes (measured full gate, 2026-07)

973 passed in **1698 s (28:18)** wall — already under `-n auto`.
Learned durations (`.pytest_cache/v/durations/call`, 984 entries):

| | |
|---|---|
| workers | **8** — the machine is 8 physical / 16 logical, and xdist's `-n auto` counts PHYSICAL |
| sum of learned durations | **229 min** |
| parallel wall clock | 28.3 min |
| longest single test | 374 s (6.2 min) — the LPT makespan floor |
| top-10 tests' share of the sum | 20% (the load is broad, not pole-bound) |
| **rung ≥ 30 files** (the `_sonic_throat` path) | **165 min — 72%** |
| rungs 7–29 (mixing-PDF / NOx diagnostics) | 64 min — 28% |

⚠ **The 229 min is NOT true serial CPU time, and no "effective parallelism" ratio should be
computed from it.** Those durations were themselves learned *during 8-way parallel runs*, so each
is inflated by memory-bandwidth and SMT contention. 229/28.3 = 8.1× on 8 workers would be >100%
efficiency — an impossibility that is the tell. Read 229 min as an **upper bound** on serial time
and use it only for *relative* shares between files (which are inflated alike). The wall-clock
figures below are the load-bearing ones.

Heaviest files: rung63 (1532 s), rung24 (1314 s), rung23 (886 s), rung59 (843 s), rung66 (681 s).

**Consequence.** The wall clock is throughput-bound (broad load on 16 cores), not pole-bound, so
the only remaining lever is **less total CPU work** — which is exactly item (1). A 2.7× on the
`_sonic_throat` path would cut ~140 of the 165 structural minutes to ~52, i.e. serial sum
229 → ~141 min, projecting a wall clock near **17–18 min**. Rungs 23/24 are untouched by it —
they are mixing-PDF integrals in `gas.py`, a separate kernel.

## Outcome of item 1 — measured, not projected

`_sonic_throat` now dispatches on `gas.hot_is_cpg` to the residual's exact linear root
`T* = h_t(Tt9)/(cp + ½γR)`; the TPG/reacting loop is factored out unchanged as
`_sonic_throat_bisect`. Derivation, the two deliberate choices, and the gate repair are in
`docs/rung30-spec.md` § "The CPG branch is SOLVED, not searched".

| | before | after |
|---|---|---|
| heaviest rung-66 `_stator_march` | 12.52 s | **2.32 s — 5.38×** |
| full gate (`pytest --runslow`, 16 cores) | 28:18 | **13:09 — 2.15×** |

The gate beat the 17–18 min projection because the projection assumed a 2.7× secant; the CPG
root is closed-form, so the ~45 residual evaluations per call go to **zero**, not to ~5.

**Numerical consequence.** Every CPG result moves by 1–2 ulp (max 2.4e-15 relative over a
341-point march). Safe for the reduce spine *by structure*: `test_reduce_*` / `*_bit_for_bit`
compare two code paths under the **same** solver, so both sides move together.

**What it exposed (worth more than the speed).** The gate came back `972 passed, 1 failed`, and
the one failure was not a regression:

- `test_rung48.py::test_decel_never_fires_bit_for_bit_rung45` compared a min-select against its
  own reference point at margin 0, i.e. asserted `(a/b)·b == a` in binary floating point.
- Measured on the **unmodified** tree, that gate trips at `HI` = 1398, 1395, 1350 and passes at
  1400, 1399, 1390, 1380 — a **~40% pre-existing flake** that happened to be green at the one
  start temperature the suite used. The change merely re-rolled the die.
- In every case the discrepancy is confined to **row 0 alone** (225/226 rows bit-identical), on
  both trees — so the contract holds everywhere it actually claims to.
- Repaired by construction, not by tolerance: bit-exactness kept, comparison starts at row 1,
  row 0 gated as the ulp artifact it is. Gate 6 already owned the round-trip identity at its
  honest 2e-3 tolerance, so no coverage was lost.

**Coverage repair.** Rung 30's gate 2a justified itself as *"two genuinely different code paths"*
but ran on a CPG gas — after the branch it would have compared a closed form against itself and
gone on passing. It is now a **three-way** agreement (bisection ≡ linear root ≡ textbook
`2/(γ+1)`) with the bisection called explicitly, which is strictly stronger than what it replaced.

**A second target, measured and declined.** `engine.py:_solve_choked_turbine` also bisects ~46
times (cum 283 s of a 417 s profile). It does **not** get the same fix: its residual runs through
`pr_t`, powers and `τ^0.5`, so it is not linear and has no closed form — and 4.6M of the 6.6M
`_sonic_throat` calls came from *inside* its residual, so item 1 already gutted its cost. A
secant there would change last bits on thirty rungs of anchored numbers for a much smaller win.
Left open deliberately.

## Outcome of item 2 — measured, not projected

PyPy 3.11 (v7.3.23) installed **outside the repo** at `M:\claud_projects\temp\pypy`, with its own
`cache_dir` so the two interpreters cannot poison each other's learned durations. **Not adopted** —
nothing in the tree points at it; `requirements.txt` and `pytest.ini` are untouched.

The first comparison (PyPy 1:55 vs CPython 13:17 ≈ 6.9×) was **confounded**: PyPy's `-n auto`
resolved to 16 **logical** CPUs, CPython's to 8 **physical** ones. Re-run at matched worker counts:

| workers | CPython 3.14 | PyPy 3.11 | ratio |
|---|---|---|---|
| `-n 8` | 13:17 (797 s) | **2:37** (157 s) | **5.08×** |
| `-n 16` | 10:13 (613 s) | **1:55** (115 s) | **5.32×** |
| single `_stator_march`, 1 thread | 2.32 s | 0.622 s | 3.74× |

**5.1–5.3×, consistent at both worker counts** — the honest figure; the 6.9× was half interpreter,
half worker count. It is *larger* on the whole gate than on one march (3.74×) because JIT warmup
amortises across 973 tests instead of being paid once per process.

**The result that matters: trajectories are BIT-IDENTICAL across interpreters.** A 341-point rung-66
march compares exactly equal, float for float. For a project whose spine is anchored numbers and
`*_bit_for_bit` gates, that is the precondition — without it the speed would be unusable.

> **⚠ SCOPE CORRECTION (2026-07-31).** That sentence is true of the kernel it was measured on and
> was written without saying so. The march above is a **CPG / `_sonic_throat`** path. Re-measured
> across six more kernels: the **CPG ladder is bit-identical** (96 off-design matcher values,
> 6 152 rung-66 trajectory floats, every discrete argmin / limiter-edge row), but **71% of 133
> equilibrium-side values DIFFER** — rungs 3–30 diagnostics, worst **3.7e-6** relative, on a
> difference-of-near-equals quantity that is unchanged at 3 significant figures. Cause:
> `expm1` / `log1p` / `erf` differ by 1 ulp and naive `sum()` reassociates (`exp`, `log`, `sqrt`,
> `**`, `fsum` all agree). Read the claim as **"the CPG structural ladder is bit-identical"**, not
> "the interpreter is". Full evidence and the adoption plan: **`docs/plans/todo-pypy-switch.md`**.

**Side finding, free and interpreter-independent.** `-n 16` (logical) beats `-n 8` (physical) by
**1.30×** on CPython and 1.36× on PyPy. xdist's `-n auto` counts physical cores, so the stock gate
leaves that on the table. `-n 16` is a one-flag change to `pytest.ini` — **not taken**: the user
reported the machine going sluggish under it, and 1.3× is not worth the interactive box.

## The interactive-headroom fix (2026-07-30) — priority, not fewer workers

`-n auto` packs a worker onto every physical core, so a gate saturates the machine whatever the
interpreter. The fix in `conftest.py` is `_run_at_below_normal_priority()`, called from
`pytest_configure` — which runs in the controller **and** in every xdist worker, so one call
covers the fleet (verified: all 9 processes report `BelowNormal` mid-run).

**Why priority and not `-n 6`.** A below-normal process is preempted only when something else
actually wants the CPU, so an idle machine still runs the gate at **full speed** — the cost is
zero when nobody is competing. Surrendering a worker pays unconditionally.

**Honest scope.** This governs CPU scheduling only. Memory-bandwidth and L3 contention are not
priority-governed, so a packed run can still feel heavy. If it does, `-n 6` is the lever priority
cannot pull. Opt out entirely with `JET_TEST_NICE=0`.

**A coupling to watch (not blocking, but nobody was watching it).** Every run rewrites
`.pytest_cache/v/durations/call`, and `conftest.py`'s `SLOW_SECONDS = 8.0` is a **threshold on
those numbers**. Those durations were always contention-inflated; running below-normal makes the
inflation depend on *what else the box is doing*. So a fast non-spine test that gets preempted past
8.0 s can be tagged `slow` and silently drop out of the bare `pytest` subset. **If the fast subset
ever shrinks unexpectedly, this is why** — the fix is to re-learn durations on an idle machine.
For the same reason the `~5 min` / `~6–16 min` budgets in `CLAUDE.md` are left alone: they are
stale-HIGH (the safe direction) since the rung-30 fix, and re-measuring them under load would write
a contention number in place of an honest one. The `~13 min` full-gate figure was measured on an
idle box at normal priority — that is the one to keep.

**The trap it nearly shipped as.** The obvious spelling
`kernel32.SetPriorityClass(kernel32.GetCurrentProcess(), …)` returns **0** on 64-bit Windows and
raises nothing: `GetCurrentProcess()` returns the pseudo-handle `(HANDLE)-1`, and ctypes' default
`restype` of `c_int` truncates it, so the call gets a bad handle and fails **silently**. Measured —
processes stayed at `Normal` while the code "succeeded". `restype = c_void_p` is load-bearing.
Because a silent no-op is the failure mode, the helper *verifies* with `GetPriorityClass` and the
report header prints a **WARNING** when the drop did not take.

**Friction, both small.** (a) `tests/test_rung28.py` imports `main`, which imports matplotlib, so a
bare PyPy gate errors at collection (953 passed, 1 error) until `pip install matplotlib` is run into
PyPy — my earlier claim that "no test imports matplotlib" was wrong, it enters one level down.
(b) The learned-duration cache is per-interpreter and must not be shared.

**Combined with item 1:** the full gate went **28:18 → 1:55, ~14.8×**, with no algorithmic change
beyond one closed-form root and no change to any anchored number.

**A full Rust rewrite is refused on the project's own terms.** `CLAUDE.md`: *"The deliverable
is understanding, not the tool."* The 42% prose ratio is the evidence — this is a document as
much as a program, and the reduce spine is a Python-inheritance artifact. A rewrite trades the
deliverable for wall-clock that items 1–3 already buy.
