---
name: perf-sonic-throat-and-pypy
description: "The 2026-07-30 perf investigation: _sonic_throat's CPG branch is a CLOSED-FORM root (5.4x on a march, 28:18 -> 13:17 on the gate) and PyPy is a further 5.1-5.3x. Its bit-identity is SCOPED to the CPG ladder (corrected 2026-07-31). Also the lesson that a profile's hot function may be hot for an algorithmic reason, not a language reason."
metadata: 
  node_type: memory
  type: project
  originSessionId: 9815da7f-03c3-438c-b234-613c89605708
  modified: 2026-07-31T06:31:03.587Z
---

Asked whether `engine.py` should be split and whether a **Rust rewrite** would buy speed. Both
answers inverted the question's framing. Full write-up: `docs/plans/todo-engine-size-and-speed.md`.

**The headline lesson: the profile said "85% in one function", and the right reading was
ALGORITHMIC, not linguistic.** `components.py:_sonic_throat` bisected ~45 times to `1e-13*Tt` —
on a CPG gas its residual `h_t(Tt) − h_t(T) − ½γRT` is **exactly linear in T**, so the root is
`T* = h_t(Tt9)/(cp + ½γR)` in closed form. A Rust port would have made 45 wasted iterations fast;
deleting them made them free. **Before porting a hot kernel, ask what it is computing, not how
fast the language computes it.** Rungs 31–66 all run CPG, so this is the whole structural path.

**Measured (2026-07-30, 973 tests):**
- heaviest rung-66 `_stator_march`: 12.52 s → 2.32 s (**5.38×**)
- full gate `--runslow`: 28:18 → 13:17 (**2.15×**), CPython `-n 8`
- PyPy 3.11 on top: **5.08× at `-n 8`, 5.32× at `-n 16`** — combined **28:18 → 1:55, ~14.8×**

**PyPy is NOT adopted** — it lives at `M:\claud_projects\temp\pypy` with its own `cache_dir`;
`requirements.txt`/`pytest.ini` untouched. Preconditions if it ever is: matplotlib must be
installed into it (`test_rung28` → `main` → matplotlib, one level down — I wrongly claimed no test
imports it), and the learned-duration cache must not be shared between interpreters.

**⚠ CORRECTED 2026-07-31 — "BIT-IDENTICAL across interpreters" is TRUE but SCOPED, and I stated
it here without the scope.** The evidence was ONE 341-point rung-66 march, which is a CPG /
`_sonic_throat` path. Measured across six more kernels: the **CPG ladder is bit-identical**
(96 matcher values + 6 138 trajectory floats + every discrete argmin/edge row), but **71% of 133
equilibrium-side values DIFFER** (rungs 3–30 diagnostics; worst 3.7e-6 relative, on a
difference-of-near-equals). Mechanism: `expm1`/`log1p`/`erf` differ by 1 ulp and naive `sum()`
reassociates — primitives the NASA-integral / equilibrium kernels use and the CPG closed forms
do not. **The generalising lesson: one bit-identical sample licenses a claim about that KERNEL,
never about the interpreter.** Plan + evidence: `docs/plans/todo-pypy-switch.md`. The detector
that hole demanded is now built — see [[golden-fingerprint-gate]].

**Two confounds I caught only by checking, both of which would have shipped a wrong number:**
1. PyPy's `-n auto` resolved to 16 **logical** CPUs, CPython's to 8 **physical** → the first
   "6.9×" was half worker count. Matched runs gave the honest 5.1–5.3×.
2. `.pytest_cache` learned durations are recorded **during 8-way parallel runs**, so their sum is
   contention-inflated. 229 min / 28.3 min = 8.1× on 8 workers is >100% efficiency — the
   impossibility is the tell. Never compute an effective-parallelism ratio from them.

**What the change EXPOSED is worth more than the speed.** The gate came back 972/1, and the one
failure was a **~40% pre-existing flake**: `test_rung48.py::test_decel_never_fires_bit_for_bit_rung45`
compared a min-select against its own reference point at margin 0, i.e. asserted `(a/b)·b == a` in
binary FP. Proven pre-existing by sweeping `HI` on the **stashed** tree (fails at 1398/1395/1350,
passes at 1400/1399/1390/1380) — it was green only at the one start temperature the suite used.
Repaired **by construction** (compare from row 1; row 0 gated as the ulp artifact it is), never by
loosening a tolerance. **A `*_bit_for_bit` gate that reconstructs a value instead of reusing it is
a latent flake** — see [[always-commit-and-push]] for why the full gate is what catches these.

**A near-miss I caught myself:** the branch would have silently gutted rung 30's gate 2a, which
justifies itself as *"two genuinely different code paths"* but runs on a CPG gas — it would have
compared the closed form against itself **and gone on passing**. Repaired by factoring the loop
out as an explicitly-callable `_sonic_throat_bisect`. **When you replace a branch, check whether a
test's claimed independence was riding on the branch you just removed.**

**Declined, deliberately:** `_solve_choked_turbine` also bisects ~46× but its residual runs through
`pr_t`, powers and `τ^0.5` — not linear, no closed form — and 4.6M of the 6.6M `_sonic_throat`
calls came from *inside* it, so item 1 already gutted its cost. **A full Rust rewrite is refused on
the project's own terms** (`CLAUDE.md`: *"the deliverable is understanding, not the tool"*; 42% of
`engine.py` is prose). Rust/PyO3 on `_sonic_throat` alone is **still open — the user deferred it**.
**Splitting `engine.py` was assessed and NOT recommended**: the coupling is a 9-deep inheritance
chain that IS the reduce-to-prior contract made executable, and a split moves files without
touching it. See [[test-suite-speed-policy]], [[claude-md-is-a-reference]].
