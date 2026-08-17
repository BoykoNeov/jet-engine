# The Rust port — plan

**Status: PHASES 0–3 COMPLETE AND GREEN — slices A (rungs 7/8/9/19), B (10/11/12/20),
C (13/15/16/18/21), D (22/23/24) and E (14/17, the nozzle strand) all shipped.
PHASE 4 (the nozzle & turbine marches, rungs 25–30) was AUTHORISED 2026-08-12 and is COMPLETE —
three dependency slices, F (25/26) § 4.11/4.12, G (27/28) § 4.13/4.14, H (29/30) § 4.15, all
bit-exact.
**PHASE 5 (the steady matchers) was AUTHORISED 2026-08-13 and is IN PROGRESS, in SLICES.** Its
**PRE-FLIGHT inheritance census (§ 5.3) came first and is DONE**: the rung-61 diamond is
discharged, and the phase's real structural requirement is a FIVE-name virtual-dispatch set —
one of them claimed by PHASE 6 — plus one live constant shadow. **SLICE I (rungs 31/33) IS
SHIPPED — 3951/3951 bit-exact; 415 Rust tests in all** (§ 5.5); it also carries the port's first
fallible paths and its first virtual hook, and § 5.4 (i) records how the fallible design's
first answer was wrong. **PHASE 5 IS COMPLETE** — seven slices, I·J·K·L·M·N·O, the last of them
shipping rung 61's diamond on 2026-08-17.
**PHASE 6 (the 15 transient rungs) WAS AUTHORISED 2026-08-17.** Its **PRE-FLIGHT (§ 5.12) came
first and is DONE**: the § 5.3 census run in the opposite direction — phase 6 as the ancestor
side — found **six names crossing into phase 7**, all on the two-spool chain, zero constant
shadows, and no sibling dispatch owed. It also settled two things the slice plan cannot discover
later: `integrate_fuel` is ported **entire** (rungs 46–52 are keywords on one method, not seven
classes), and the `min`-select hazard the phase table's "low-risk" label would have missed is
**refuted by measurement** — the legs never contend, so the discrete content is the arming
predicates, not an argmin. **The next authorisation point is before phase 7.**
The architecture is settled by measurement (§ 1–2); the three forks were answered on 2026-08-12
(§ 9); phases 0–2 were then built and gated (§ 4.1, § 4.2). Phase 1 was the first deliberate
stopping point because it is where the arithmetic risk concentrates; phase 2 was authorised
separately and **corrected phase 1's central diagnosis** — see § 4.2, which is the answer to
the question phase 1 thought it had already answered. Phase 3 was authorised on 2026-08-12 and
was taken in slices, because at 2,745 source lines and 204 tests it is the largest phase in the
port; § 4.3 records slice A, § 4.4 slice B, § 4.5 slice C, § 4.7–4.8 slice D, § 4.9–4.10 slice E.

**The ask.** The whole project in Rust — engine *and* all tests. Python may survive only as a
**single-use oracle**: a reference implementation the Rust is validated against, then deleted.

**The standing constraint** (`CLAUDE.md`): *the deliverable is understanding, not the tool.*
The ~21,400 lines of derivation comments and 190 spec documents are the asset. A port that
runs faster and reads worse has lost.

---

## 1. What the spike settled

`M:\claud_projects\temp\rust-spike` — 28 rungs, 9 hooks, no physics, override counts copied
from the measured ones in `engine.py`. Full detail in its `RESULTS.md`.

| question | answer |
|---|---|
| Does `const R58: Hooks = Hooks { close_fuel: r58_close_fuel, ..R57 };` chain 28 deep? | **Yes.** A rung is literally the fields it replaces. |
| Does reduce-to-prior survive? | **28 / 28 bit-for-bit.** The `None` arm calls the parent's function, so there is no arithmetic to drift. |
| Is every rung separately runnable? | **Yes** — 27 / 27 successive rungs move the answer. |
| What does the indirection cost? | **0.5 %** at the real hook rates — inside the drift check's own 0.1 %. Worst case (a ~10-flop hook at 3.1 M calls) ≈ **1.1 ns/call**, ~2 %. |
| Compile time at depth 28? | **7.1 s** clean with LTO, 6.6 s incremental, 510 KB binary. |
| Can the parent be handed to a solver as a value? | **Yes** — a table field already *is* a `fn`. No closure, no allocation, no generics. |
| Can Rust replace the `inspect`-based tests? | **Yes** — `include_str!` reads sibling source at *compile* time. |

**Two findings that change how the port is written:**

- **The leaf-dispatch trap is silent.** Python resolves `self._instant_fuel` inside a rung-57
  body to the *leaf's* override. The compile-time-generics arrangement that drops the leaf
  type parameter resolves it to rung 57's own — it **compiled** and returned a number 0.018 %
  different. Nothing warned. The function-pointer table cannot make this mistake, because the
  leaf's table is the argument.
- **Never force-inline down the ladder.** A rung's fallback appears at two call sites (`None`
  and `Some`); with `#[inline(always)]` at depth 28 that is 2²⁷ expansions. The spike's first
  build ran > 10 min in codegen and was killed. Removing the attribute → 7.1 s.

---

## 2. The architecture: a `const Hooks` table per rung

```rust
pub struct Hooks {
    close_fuel:   fn(&Hooks, &mut State, &Config) -> f64,
    cap_fuel:     fn(&Hooks, &mut State, &Config) -> f64,
    instant_fuel: fn(&Hooks, &State,     &Config, f64) -> Instant,
    // ~8-10 total; see § 3 for why it is not 40
}

// rung 64's bleed limiter. The reduce contract is the SAME LINE Python writes.
fn r64_close_fuel(h: &Hooks, st: &mut State, cfg: &Config) -> f64 {
    let Some(lim) = cfg.bleed_lim else { return r63_close_fuel(h, st, cfg) };
    solve_b(closer(r63_close_fuel, h, st, cfg), lim).0    // parent passed as a VALUE
}

pub const R63: Hooks = Hooks { close_fuel: r63_close_fuel, ..R62 };
pub const R64: Hooks = Hooks { close_fuel: r64_close_fuel, ..R63 };   // <- the whole rung
```

`h` — the **leaf's** table — rides through every call, so a rung-57 body asking for
`instant_fuel` still gets rung 84's, exactly like Python's `self`.

Toggling is **one branch at the top of the program**, not per timestep:

```rust
let hooks = match rung { 57 => &R57, /* ... */ 84 => &R84, _ => bail!() };
march(hooks, &cfg);        // from here on, a straight loop
```

That is what makes lean execution and 28 live histories compatible: the measured hook call
rates put all of the ladder's *depth* in cold code.

| hook | calls per rung-84 march | places redefined |
|---|---|---|
| `_close_fuel` | 62,670 | 5 |
| `_instant_fuel` | 26,217 | 2 |
| `_cap_fuel` | 3,498 | 3 |
| `_close` | 240 | 5 |
| `_shared_rig` | 8 | 8 |
| `_stator_march` | 5 | 5 |
| `integrate_fuel` | 3 | 13 |
| `at_lever` | 1 | 18 |

The columns are inversely related. Nothing redefined more than five times is hot.

### Layout — one file per rung

```
src/rungs/rung61.rs    impl + readers + the derivation comments + `pub const R61`
src/rungs/rung84.rs
```

`git log src/rungs/rung84.rs` then *is* the rung. This is strictly better than today, where
all 28 classes share one 20,545-line file.

### What Rust deletes outright

`at_lever` (18 copies) and `_shared_rig` (8 copies) exist only to copy every field forward
into a sibling machine. Rung 80's docstring calls it *"THE EIGHTEENTH INSTANCE of the trap
rungs 61–79 each hit"*, and says dropping one field would have *"returned this rung's own
predicted result having measured rung 79."* In Rust that is `Config { vsv_lp, ..self.cfg }`
— **and forgetting a field is not expressible.** Twenty-six overrides and a recurring bug
class gone.

### Rules that follow

1. Hooks take `&Config`, never a positional knob list. Otherwise every signature is the union
   of all 28 rungs' knobs and adding rung 85 edits all 28 files.
2. No `#[inline(always)]` on any rung function. Where both branches call the parent exactly
   once, hoist to a single call site.
3. One flat `State` struct. A disabled state has a derivative of exactly zero — bit-exact
   under fixed-step RK4, which `_rk4_floor_shared`'s `ds·Σ(1/τ) ≤ 2` assertion confirms this
   is. **Watch:** that assertion sums over *armed* legs; it must keep doing so.

---

## 3. Inventory

| | code lines | comments | files |
|---|---|---|---|
| `gas.py` | 2,422 | 2,156 | 1 |
| `components.py` | 262 | 373 | 1 |
| `engine.py` | 12,585 | 7,960 | 1 |
| `main.py` | 5,236 | 934 | 1 |
| `tests/` | 17,399 | 8,997 | 89 |
| **total** | **37,904** | **20,420** | 93 |

58 classes, 768 methods, 1,180 test functions (1,355 tests), a 28-deep ladder, plus 190
markdown docs (34,515 lines) that do **not** change.

**Dependencies: `math` and `cmath` only.** `matplotlib` appears in `main.py` and
`docs/visuals/` alone. There is no library ecosystem to replace — unusually clean.

**The trait is ~8–10 methods, not 40.** 264 methods are defined exactly once (each rung's own
diagnostics — free functions, not hooks); 26 more are sibling constructors that Rust deletes;
`integrate_fuel` is 2 real bodies plus 10 thin delegating wrappers.

---

## 4. The bit-exactness decision (OPEN — § 9)

`tests/golden/numeric_fingerprint.json` holds 2.2 MB of **CPython** values. Rust will not
naturally reproduce them digit-for-digit.

**Option A — demand digit-for-digit.** Reachable in principle: the earlier cffi experiment hit
100.00 % on three kernels across ~3.0 M values. It took three bug hunts to get there — LLVM
strength-reducing `x.powf(0.5)` to `sqrt` (wrong in the last bit ~1 in 2,500), a wrong name
binding, and `σ·(d·d)` vs `(σ·d)·d` (float multiply is not associative; wrong 45 times in
32,508). Those were the *simplest* pieces. Scaled to the equilibrium solve and the NASA
integrals — full of `exp`/`log`/`pow`, whose last digit belongs to the C math library — this
is expensive and may be unreachable.

**Option B — Python as oracle, then re-anchor (RECOMMENDED).** Run both, require agreement to
a declared tolerance across every golden key, publish the deviation distribution, adjudicate
the fragile rungs individually, *then* freeze Rust's values and delete the Python. The
existing CPython file stays in git history as the audit trail.

Why B is safe here: nearly every test compares two quantities from the **same run** — that is
what reduce-to-prior means — and those survive any faithful port untouched. The spike showed
the reduce contract is *stronger* in Rust (28/28 bit-exact by construction, § 1). Only one
test file holds absolute values.

### 4.1 What phase 1 MEASURED — the bar is far easier to clear than feared

> **⚠ READ § 4.2 FIRST. This section's numbers and its central diagnosis were both superseded
> by phase 2.** Its *conclusion* — the bar is far easier to clear than feared — survives and
> in fact strengthened to 100 %. Its *reason* for the residual 36 misses (a solver stopping
> rule) was wrong: the cause was a 1-ULP transcription defect in the polynomial spelling, which
> the stopping rule then amplified. Kept unedited below as the audit trail, because the way a
> tolerance bar hid a real defect is itself the finding.

The tolerance policy is not invented. The project already ships on two interpreters (the gate
runs PyPy, the fingerprint goldens are CPython), so whatever those two disagree by is a
deviation the project ALREADY tolerates. The oracle therefore dumps under both, and the gap
sets each bar. Over all 3232 gas values, rungs 1–6:

| | bit-identical | |
|---|---|---|
| Rust vs **CPython** 3.14.3 | 1880 / 3232 | 58.17 % |
| Rust vs **PyPy** 3.11.15 | **3196 / 3232** | **98.89 %** |

**The second line is the finding.** Every forward quantity is **100 % bit-exact against
PyPy** — `cp`, `h`, `pr`, `gamma`, `R`, the mole-weighted coefficients, the whole equilibrium
substrate (`a6`, `a7`, the four molar functions, `lnKp`), the dense Gaussian elimination, and
the **8-species equilibrium composition itself** — with `exp` and `log` included. Rust's
floating-point arithmetic is not a source of drift here. It *is* PyPy's; CPython's libm is
the outlier.

The only spread left is the two safeguarded-Newton inverses (`T_from_h` 373/400, `T_from_pr`
391/400), and that is not arithmetic either: it is `_solve`'s own `tol = 1e-11` relative
stopping rule landing on a marginally different iterate — **three orders of magnitude above
every other term in the gas layer.**

Three consequences for the rest of the port:

1. **Option B is cheaper than budgeted.** The re-anchor is not "Rust's numbers replace
   CPython's"; it is "Rust's numbers are PyPy's numbers", and the project's gate already runs
   on PyPy. The fingerprint's existing tolerances already absorb this exact gap.
2. **The residual risk is the SOLVERS, not the arithmetic.** Every remaining phase should
   budget for stopping-rule reproducibility and not for last-bit polynomial drift.
3. **The rung-6 result de-risks phase 3.** The equilibrium solve was named the highest-risk
   piece and came back bit-exact, which is the substrate rungs 7–24 sit on.

### 4.2 What phase 2 MEASURED — § 4.1's diagnosis was WRONG, and the bar is 100 %

Phase 2 ported `components.py` and `engine.py`'s design point, and built the cycle oracle that
gates them (`rust/oracle/dump_cycle.py`, `rust/tests/cycle_oracle.rs`) — the twin of phase 1's
gas oracle, one layer up: the whole design cycle, across the gas ladder and the loss
configurations rungs 1–6 exercise. It found a defect in phase 1's shipped code.

**§ 4.1 attributed the residual 36/3232 to the solvers. It was arithmetic.** `poly`,
`antideriv_h` and `antideriv_phi` spelled Python's `T ** 3` … `T ** 5` as product chains.
Measured over 6013 points against both interpreters, `x*x*x` reproduces `x ** 3` only
4519/6013 and `x*x*x*x` reproduces `x ** 4` only 3975/6013 — Python's `**` is a libm `pow`
call, and a product chain is a *different function* in the last bit. The error lands in the
high-order terms, ~1e-20 relative to the sum, so it tipped the last bit only occasionally —
which is why phase 1's own oracle passed at 100 % on enthalpy. **The safeguarded Newton then
amplified it to 1e-11 through its `tol = 1e-11` stopping rule, which is exactly what made it
look like a stopping-rule artefact.** The gate ran a whole phase on that misreading.

The rule that fixes it is **split**, and neither simplification works:

| | spelling | gas oracle vs PyPy |
|---|---|---|
| the SQUARE | `t * t` | **3232 / 3232** ✅ |
| | `powp(t, 2.0)` | 3230 / 3232 |
| the CUBE and above | product chains | 3196 / 3232 (phase 1) |
| | `powp(t, 3.0/4.0/5.0)` | **3232 / 3232** ✅ |

The asymmetry is PyPy's, not Rust's: its JIT rewrites `x ** 2` into a multiply and does **not**
rewrite higher powers, so reproducing PyPy means doing exactly the same. A third trap sits
beside it: written as the obvious `x.powf(0.5)`, LLVM folds the call to `sqrt` — measured
identical on all 4012 grid points — while Python's `x ** 0.5` differs from `sqrt` about 1 point
in 670. Hence `gas::powp`, whose `black_box` on the exponent defeats the fold.
`rust/tests/porting_rules.rs` keeps all three detectors honest, oracle-free.

**Result after the fix, and the new bar:**

| | bit-identical vs PyPy | vs CPython |
|---|---|---|
| gas oracle (3232 values) | **3232 / 3232 (100 %)** | 1883 / 3232 |
| cycle oracle (1481 values) | **1481 / 1481 (100 %)** | 903 / 1481 |

The two Python interpreters agree with each other on only **64 %** of the cycle values, with
gaps to 4.9e-11 — so "Rust IS PyPy" is now a stronger statement than "Python is Python".

**Both new solvers reproduce bit-for-bit**, and that claim is sized deliberately. A headline
like "`far`: 114/114" is 19 measurements wearing a 114 costume — stations 0/2/3 are
structurally zero and 4/5/9 repeat one number. So the oracle carries a **solver sweep** moving
every knob the burner's root depends on (`pi_c` 6→30, `Tt4` 1000→2100 K, `eta_b` 0.93→1.00,
`M0` 0.2→2.4, `p0` 15→101 kPa, `mdot` 1→200). The claim rests on **19 distinct fixed-point
roots and 15 distinct bisection roots**, all bit-exact, and `cycle_oracle.rs` asserts that
distinct-root count so it cannot silently collapse.

**THE BAR IS NOW BIT-EQUALITY FOR PHASES 0–2, AND THAT REVISES § 9 DECISION 1.** Both oracles
assert `exact == total` on the PyPy arm. § 9 chose Option B (tolerance) on the grounds that
Option A was "expensive and may be unreachable"; phases 0–2 measured it to be *neither*, and a
tolerance bar demonstrably cannot tell a real defect from acceptable noise — for a whole phase
it did not. **This is not a blanket adoption of Option A.** A later phase — phase 3's mixing
PDFs and bell integrals bring new solvers — may legitimately fail to reach 100 %. If it does,
that phase falls back to § 4.1's tolerance policy **with its deviation distribution published
in this document**, never by silently loosening the bar. A future session that sees 3231/3232
should read the assertion message, which says the same thing.

**The price, measured and accepted: 2.1× on a rung-5 Fork-B cycle (6.2 → 12.9 µs), 1.36× on a
rung-6 equilibrium cycle (850 → 1157 µs).** It is confined to the three polynomial functions
and therefore to REAL-GAS paths: `CpgSection` is closed-form and never touches them, and the
deepest transient ladders (rungs 66–84, where the wall-clock actually is) build bare CPG gases.
The steady matchers and lower transients (31, 34, 40, 57) do build equilibrium gases and will
pay. § 6's "2–4 minutes" estimate should be read with that in mind — it was an estimate before
this cost existed. If a later phase becomes speed-bound, the trade is reversible in one
function, at a known price in bit-exactness.

### 4.3 What phase 3 SLICE A MEASURED — the bar holds where the plan said it would break

Slice A is rungs **7 / 8 / 9 / 19**: the extended-Zeldovich integrator, the two-zone
primary→dilution split, the rich-primary bell, and rung 19's two channels for lifting the
equilibrium-[O] lower bound. It ships `rust/src/nox.rs`, the oracle `rust/oracle/dump_nox.py`,
and five gates (`nox_oracle.rs` plus the four rung suites, 47 tests).

**The remaining slices, grouped by DEPENDENCY rather than by number** — the numbering is
misleading here, because rungs 20 and 21 are super-eq O threaded through machinery that arrives
much later than rung 19:

| slice | rungs | what it needs first |
|---|---|---|
| ~~the finite-rate quench~~ | ~~10, 11, 12, **20**~~ | **DONE — § 4.4** |
| ~~the PDF family~~ | ~~13, 15, 16, 18, **21**~~ | **DONE — § 4.5** |
| ~~the resolved cross-plane~~ | ~~22, 23, 24~~ | **DONE — § 4.7 (pre-registration), § 4.8 (measurement)** |
| ~~the nozzle strand~~ | ~~14, 17~~ | **DONE — § 4.9 (pre-registration), § 4.10 (measurement). PHASE 3 COMPLETE.** It lands next to phase 4, whose rung 25 brackets against rung 14's bounds |

Slice B leaves the quench machinery (`quench_trajectory` / `quench_no` / `JetMixing` /
`Unmixedness`) in place, which is what rungs 15 and 16 build their dwell chain on — so the PDF
family was the natural next slice, and the nozzle strand stays portable at any point. Slice C in
turn leaves `bell_interpolator`, `beta_pdf_nodes_weights` and `pocket_quench_grid` in place,
which is the entire substrate rungs 22/23/24 need: they change only the SOURCE of `g` (and, for
23/24, add a per-pocket `τ(ξ)` where the bank currently takes one scalar).

| | bit-identical vs PyPy | vs CPython |
|---|---|---|
| NOx oracle (1806 values) | **1806 / 1806 (100 %)** | 1159 / 1806 (**64.2 %**) |

**The bar holds, and it holds exactly where phase 3 was rated risk-bearing.** § 5's
reconciliation narrowed the risk to "watch the stopping rules, not the polynomials", and slice
A is where the two hardest stopping rules in the project live: `primary_aft` and `mixed_out_t`
are bisections whose INNER evaluation is the 8-species equilibrium Newton. Both reproduce
bit-for-bit, on **22 distinct AFT roots and 22 distinct mix-out roots** (asserted, so they
cannot silently collapse). The 4000-step RK4 integrator reproduces too, which was never in
doubt — it has no adaptive control, so it carries no stopping rule at all, only accumulation
order. CPython↔PyPy agree on 64.3 %, the same ~64 % the cycle oracle found.

**Three findings that change how the remaining slices are written:**

1. **SHAPE KEYS EARN THEIR COST, and the measurement says so.** Rung 9's claim is *where* the
   EI-vs-φ bell peaks. CPython and PyPy disagree on the peak **value** in the last bit and
   agree on the peak **location** exactly. A value gate would have called that a deviation; the
   argmax key says the finding did not move. Every remaining slice with a location claim (rungs
   12, 22, 24 above all) should dump its argmax beside its curve.
2. **SPLIT-INDEPENDENCE IS NOT A BIT-EQUALITY.** α cancels ALGEBRAICALLY, but `α·far_p = far_ov`
   holds only to rounding, so the bisection's final sign test can land on the other side. The
   spread is 0.0 K at two design points and **5.821e-7 K** at the other two — which is
   `2500/2³²` **exactly**, one quantum of the `[700, 3200]` bracket after the 32 halvings the
   `hi−lo < 1e-6` rule allows. Not drift: one grid step. This is the shape of thing to expect
   wherever a later rung asserts an analytic cancellation through a solver.
3. **The composition-ORDER trap did not fire in slice A, because slice A never builds one.**
   Every composition here comes from `equilibrium_composition` or `air_mole_fractions`. Rung
   10's `_quench_trajectory` and rung 12's two-stream split DO build new ones, whose insertion
   order is the code's order rather than `SP_REACT`'s. **That is the first thing the next slice
   must enumerate**, before a line of Rust: a wrong order is one ULP in `ntot`, invisible in
   `x_no`, and then amplified by a bisection — exactly the shape of phase 2's defect.

   > **CORRECTED BY SLICE B — this prediction was filed one phase early, and the discriminating
   > question was the wrong one.** "Does a rung build a dict?" is not the hazard; "is any
   > composition SUMMED or ITERATED in an order other than `SP_REACT`'s?" is. Enumerated over
   > `gas.py:999–1820` before writing any Rust: `_quench_trajectory` builds a record of SCALARS
   > read by field and never summed, rung 12's two-stream split is a scalar mass weighting, and
   > the only composition either sums is `_equilibrium_composition`'s own output. The sites that
   > genuinely assemble a composition by hand (`comp1 = {sp: …}`) are `gas.py:1963` and
   > `gas.py:2255` — **rungs 25/26, which is PHASE 4's problem.** The warning stands; it belongs
   > against the nozzle marches, not here.

### 4.4 What phase 3 SLICE B MEASURED — 100 % again, and a shipped claim narrowed

Slice B is rungs **10 / 11 / 12 / 20**: the finite-rate quench, the jet-entrainment model that
derives its time, the two-stream variance layer that recovers the Holdeman optimum, and rung
19's super-equilibrium O threaded through the quench. It ships the quench machinery in
`rust/src/nox.rs`, the oracle `rust/oracle/dump_quench.py`, and five gates — the four rung
suites (39 tests) plus `quench_oracle.rs` (5), 44 in all.

| | bit-identical vs PyPy | vs CPython |
|---|---|---|
| quench oracle (2507 values) | **2507 / 2507 (100 %)** | 1462 / 2507 (**58.3 %**) |

**THE SIZING LEVER, and it is what made the slice affordable.** `_quench_trajectory` takes no
`tau_q`, no `J` and no schedule — the fast chemistry does not know how fast the mixing is. So
ONE trajectory per (design point, φ_p) serves the entire τ_q sweep, the entire J sweep, rung
12's bulk/core pair and rung 20's lifted arm. **Five trajectories carry all 2507 values**; the
whole dump runs in 2.2 s on PyPy and 21 s on CPython. Sized without that lever, the same
coverage is minutes per interpreter, and rung 12's location claim is the first thing thinned out.

**The bars split exactly as slice A's did, and one class got STRONGER.** Three classes are
bit-identical between the two interpreters — the mixing algebra (272 keys), the shape locations
(16), and the **bisection roots (352)**. That last one is 33 trajectory points × 5 cases of
bisection-over-a-Newton agreeing to the last bit on both interpreters, where slice A could say
it of 22 roots. The classes that disagree are the ones with an accumulated iterate
(kinetic 3.8e-15, equilibrium 2.3e-15, design point 1.4e-15).

**Three findings:**

1. **THE SHAPE KEYS PAID FOR THEMSELVES ON THE FIRST RUN.** All 16 location keys are
   bit-identical across interpreters while 42 % of the values are not — slice A's finding 1,
   reproduced. And one of them **disagreed with the theory it was dumped to confirm**, which is
   the whole reason to dump locations rather than values: see 3.
2. **`math.sqrt` IS NOT `powp`, and the two hazards point OPPOSITE WAYS.** `JetMixing.tau_q`
   and `Unmixedness.C` call `math.sqrt`, which is the sqrt instruction — the inverse of phase
   2's trap, where Python's `x ** 0.5` was a `pow` call that differed from `sqrt` about 1 point
   in 670. Applying "always `powp`" mechanically gets these backwards. Meanwhile
   `JetMixing.schedule`'s `(1−x) ** shape_n` has a float ATTRIBUTE exponent, which PyPy does
   NOT rewrite into a multiply, so it DOES need `powp`. The oracle answers both in its own
   solver-free section rather than letting either surface as "EI differs" three sections later.
3. **RUNG 12's "the min pins at `C_opt` for ALL S" IS OVER-STATED, and the port's own shape key
   is what found it.** The Python's gate 3 tests only `S ∈ {0.0625, 0.05}`, both inside the
   valid band, so nothing there could see it. The boundary has a closed form in the model's own
   knobs: at the optimum `τ_mean(J_opt) = S/(C_e·C_opt·U_c)` — **`H` cancels** — and the
   "lingering" core stops being a penalty once that reaches `τ_res`. Write
   `S_x = τ_res·C_e·C_opt·U_c`.

   **The group is earned by the `τ_res` sweep, and which sweep earns it is the methodological
   point.** `C_e` and `U_c` reach the model only through `τ_mean`, so they are ONE lever wearing
   two names — a `C_e` sweep alone is a weak test, and it was the first evidence offered here.
   `τ_res` is the discriminator: it sits in `S_x` AND directly in `τ_core`, so if its appearance
   in `S_x` were doing double duty the ratio would drift. MEASURED: **the boundary is
   `pinned ≤ 1.15`, `broken ≥ 1.20`, across `τ_res` (4×, moving `S_x` 0.035 → 0.141 m), `C_e`
   (1.33×) and `H` (4×, cancels).** The excess over the derived 1.0 is the class docstring's own
   inequality being CONSERVATIVE — it assumes `EI ∝ τ`, and EI is sublinear in dwell, so the
   bulk falls more slowly than the algebra predicts. **The transition is BRACKETED in
   (1.15, 1.20], not resolved**, so the gate bars `≤ 1.15` and `≥ 1.30` and leaves the gap
   unasserted: an iff at 1.2 would put the two rows nearest the edge on a coefficient nobody
   located. The shipped default sits at `S/S_x` = 0.89 — inside the band by about 1.3×. Gated as
   `rung12.rs::the_pin_at_c_opt_has_a_spacing_limit`; recorded in `docs/rung12-spec.md`. **The
   Python's `Unmixedness` docstring still says "for ALL S" and is the one place this correction
   has not been written, because editing the ported source mid-port needs its own decision.**

Two smaller things the slice recorded rather than worked around:

- **An invented bar failed within the hour.** `design_point` was set to 1e-15 in
  `quench_oracle.rs` by analogy instead of by measurement, and one key missed at 1.36e-15. Slice
  A puts the same keys at 1e-12; this now does too, and the failure is written into the
  function's doc comment rather than quietly overwritten — § 4.2's whole lesson is that an
  unmeasured bar cannot tell a defect from noise.
- **Rung 20's flame-band floor is DORMANT at the shipped design point, by 17 K.** The
  trajectory bottoms out at 1517 K against a 1500 K floor; the clip binds only below about
  Tt4 = 1480 K. So the rung-20 floor gate runs a SECOND, cooler design point, because asserting
  it at the design point alone is a gate on a branch nothing takes.

### 4.5 What phase 3 SLICE C MEASURED — 100 % again, and the source's OWN guard has a floor

Slice C is rungs **13 / 15 / 16 / 18 / 21**: the mixture-fraction β-PDF on the ideal bell, that
PDF carried through the quench, the same per pocket, the width taken instead from a
variance-decay ODE, and rung 19's super-equilibrium [O] threaded through all of them. It ships the
PDF family in `rust/src/nox.rs`, the oracle `rust/oracle/dump_pdf.py`, and six gates — the five
rung suites (59 tests) plus `pdf_oracle.rs` (6). The Rust suite is now **217 tests**.

| | bit-identical vs PyPy | vs CPython |
|---|---|---|
| PDF oracle (2448 values) | **2448 / 2448 (100 %)** | 1727 / 2448 (**70.55 %**) |

The CPython agreement is higher than slice B's 58.3 % for a structural reason, not a lucky one: a
larger share of this slice is solver-free algebra. **The split is the same one every slice has
found** — the classes that are EXACTLY equal across interpreters are the ones with no accumulated
iterate. Worst disagreement anywhere on the dump: 4.9e-15.

| class | keys | CPython↔PyPy | what it is |
|---|---|---|---|
| `mixing_algebra` | 594 | **0.00e0** | the configs, the derived ceiling, the variance ODE, a finite difference of two ODE runs |
| `shape_location` | 48 | **0.00e0** | every argmax / argmin / branch index |
| `quadrature` | 703 | 9.7e-16 | β-PDF nodes, weights, achieved mean and variance |
| `bell` | 357 | 4.9e-15 | the ideal bell and its interpolant |
| `pdf_integral` | 568 | ≤1.3e-15 | ⟨EI⟩ over the PDF, and rungs 15/18/21's arithmetic on it |
| `kinetic` | 166 | ≤1.9e-15 | rung 16's per-pocket quenches and the public wiring |

**`mixing_algebra` is a WIDER class here than in slice B, and that is worth recording.** It now
contains `_transport_variance` — 400 repeated divisions — and a finite difference of two of those
runs at `eps = 1e-5`. Both are bit-identical on both interpreters. A loop of divisions is exactly
the shape one would expect to accumulate a difference, and it does not, because every iterate is
a division by the SAME constant.

**Five findings, and the first one is the reason to sweep wider than the source.**

1. **THE SOURCE'S OWN GUARD HAS AN `n_quad` FLOOR, AND THE PYTHON'S GATE SITS INSIDE IT.**
   `_beta_pdf_nodes_weights` asserts that the quadrature integrates at the specified mean to 1 %.
   That bar is `n_quad`-SENSITIVE. Measured at a lean mean: it **REJECTS** `g = 0.026` (the first
   sweep point past the `a = 1` regime switch) and `g = 0.40` (the top of the range) for every
   `n_quad ≤ 100`, and accepts both from 112 up — 8.2e-3 and 9.4e-3 at 112, falling to 4.0e-3 and
   5.7e-3 at 160. The Python's own gate samples `n_quad = 160` and `g ≤ 0.30`, so nothing there
   could see it. **Found by the dump's first run crashing inside the Python**, and the temptation
   was to lower the sweep and move on. Instead the convergence ladder is dumped as its own keys
   and `pdf_oracle.rs` asserts the guard **FIRES** at `n_quad ∈ {64, 100}` as well as passing at
   {112, 160} — so the port reproduces the rejection, not only the acceptance. This is the third
   consecutive slice where sweeping past the source's own gate found something the source's gate
   structurally could not.

2. **A LOCATION KEY CAN SIT ON A SCHEME BOUNDARY, AND THE FIX IS A COARSER GRID, NOT A FINER ONE.**
   The quadrature is REGIME-SWITCHING: `a = ξ̄(1/g − 1)` crosses 1 at `g = ξ̄/(1+ξ̄) ≈ 0.0258` for
   the shipped lean mean, and the two sides are different integration schemes. Rung 13's hump peaks
   at `g ≈ 0.02` — right on top of it. Measured across the boundary the curve is not even locally
   monotone: `g = 0.026` reads **0.03 % ABOVE** `g = 0.025`, the two schemes disagreeing. So the
   argmax grid is deliberately coarse, with its peak cell clearing both neighbours by ~20 %; the
   boundary itself is dumped separately as VALUES. At the hotter design point the tight side of the
   margin is 2.8 %, and it is checkable that the tight comparison lies WITHIN one branch while the
   branch-straddling one has 33 % — which is the configuration to want, and worth verifying rather
   than assuming when a grid is chosen.

3. **A PRE-REGISTERED CONFIRMATION HELD, AND ALL 48 LOCATION KEYS AGREED ACROSS INTERPRETERS.**
   Rung 21's claim is that the super-equilibrium-O lift is SHAPE-PRESERVING. Registered before the
   dump ran, at four spacings × two design points — wider than the Python's own two — the argmin is
   the SAME INDEX with the lift on and off in all eight cases, while every value moves (asserted
   separately, so the equality is not a tautology). Slice B's *refutation* was the exception, not
   the rule; the point stands that which one fires is unpredictable, so budget for both.

4. **A LOCATION THE SOURCE EXPLICITLY DECLINES MUST NOT BE DUMPED.** Rung 16's own docstring says
   which of its two near-degenerate optima is globally lowest is NOT CLAIMED — it flips across the
   β-PDF quadrature, the φ>2 tail treatment and the `C_e` regime, all comparable to the margin. An
   argmin key there would have failed for a reason that is not a defect, which is the same family
   as slice B's dormant-guard trap. What is dumped instead is the structure the rung does certify:
   the excess vanishing at `C_opt`, both flanks up, and the SUBLINEARITY ratio — two values from
   the same sweep, compared against the dwell ratio from that same sweep rather than a remembered
   constant. **`rust/tests/rung16.rs` contains no argmin assertion, deliberately.**

5. **A "HELPER MATCHES PRODUCTION" PIN BECOMES A TAUTOLOGY UNDER A BETTER FACTORISATION.** The
   Python's rung-16 suite pins a cached-trajectory re-implementation against production, which is
   a real check there. In Rust the two are the SAME call, because production is already split into
   a `τ_core`-dependent pocket bank and a cheap β-integration over it — so transcribing that test
   would compare a function to itself (rungs 78/79's vacuity trap, in a new costume). It is
   replaced by two gates that say something: production equals term 1 (built independently on the
   shared trajectory) plus term 2, and **the bank is `g`-independent BIT-EXACTLY** at three widths.
   The Python cannot state the second one at all.

Three smaller things the slice recorded rather than worked around:

- **The bell's LEAN END is a BRANCH, and the two languages express it differently.** Below the
  flammability limit `_primary_aft` pins against its cold bracket edge; Python catches the
  AssertionError, Rust splits the guard out as `try_primary_aft`, which is NARROWER than Python's
  `except` by exactly the equilibrium solver's own asserts. Rather than assume a `try/except` and
  an `Option` agree, the oracle dumps the index of the first burnable node on every bell grid:
  1 at the subsonic design point, 0 at the hotter supersonic one, so both the taken and the
  never-taken case are covered and any divergence names the grid.
- **The `≤1-of-N` closure guard is now a REAL bar.** `nox.rs` recorded in slice B that the check
  was deliberately omitted while one closure was ported — a bar that cannot fail is not a bar —
  and promised it would arrive with the second. Five are ported, so it is live and gated in three
  of the five rung suites.
- **TWO STALE PYTHON DOCSTRINGS, RECORDED AND NOT EDITED.** `gas.py:3355–3356` still says the
  ideal-bell integrals "DELIBERATELY stay equilibrium-O (forbidden to combine)", and `gas.py:4360`
  still says the PDF fields "stay equilibrium-O (threading the lift THROUGH the quench is a
  deferred seam)". **Rung 21 discharged both**, and the code at `gas.py:4422–4430` says so
  explicitly. This is the same call slice B made about rung 12's over-stated `Unmixedness`
  docstring: editing the ported source mid-port needs its own decision, and an edit made now would
  put the oracle and its source out of step for a reason unrelated to the port. Recorded in BOTH
  places, as slice B's was — here, and in `docs/rung21-spec.md`, because someone reading the rung's
  own spec to understand the physics never opens this file.

### 4.6 Every bit-equality number in § 4.1–4.5 is OPTIMISATION-INDEPENDENT

Each slice's percentage was measured on ONE build. That is a weaker claim than it reads as: an
unqualified "2448/2448 bit-exact" asserts a property of the *arithmetic*, but a single run only
establishes it for one codegen. Re-run after slice C at **three optimisation levels** — `opt-level
= 0`, the `[profile.test]` `opt-level = 2`, and release's `opt-level = 3` + `lto = "thin"` +
`codegen-units = 1` — all five oracle gates (`gas` / `cycle` / `nox` / `quench` / `pdf`, 11,474
values) pass identically at every one. `opt-level = 0` is 8.3× slower on `pdf_oracle` (43.7 s vs
5.3 s), so the codegen really is different.

This is the expected result and worth having anyway: Rust does not enable FP contraction, so
`a*b + c` may not become an FMA and the optimiser may not reassociate — the guarantee that makes
the port's whole bit-equality programme possible is a LANGUAGE property, not a lucky build. The
one-line consequence: **the percentages may be quoted unqualified**, and a future disagreement that
appears only at one optimisation level is a compiler bug, not a porting defect.

The same pass re-checked something § 4.5's stale-docstring entry above does NOT cover: whether the
**Rust** doc comments transcribed the stale Python claim. They did not — `nox.rs` states the forbid
guard DISCHARGED, and every "stays equilibrium-O" in the Rust is conditioned on `super_eq_o =
false`, which is the reduce contract rather than the stale assertion. The no-edit policy is
correctly scoped to the oracle: the Rust is not the oracle, so text there can and should be fixed
on sight.

### 4.8 What phase 3 SLICE D MEASURED — 100 % again, and TWO "exact" claims corrected

Slice D is rungs **22 / 23 / 24**: the resolved y-z cross-plane, that plane developed in TIME so
each pocket carries its own dwell, and the same plane with each cell relaxing at its own
gradient-derived rate. It ships the closures in `rust/src/nox.rs`, the oracle
`rust/oracle/dump_spatial.py`, and four gates — three rung suites (43 tests) plus
`spatial_oracle.rs` (5). The Rust suite is now **265 tests**.

| | bit-identical vs PyPy | vs CPython |
|---|---|---|
| spatial oracle (462 values) | **462 / 462 (100 %)** | 241 / 462 (**52.2 %**) |

**All 10 location keys and all 28 DISCRETE keys are identical on BOTH interpreters** — the split
every slice has found, now with a new class on the exact-zero side.

| class | keys | CPython↔PyPy | what it is |
|---|---|---|---|
| `cross_plane_algebra` | 30 | **0.00e0** | penetration, the Holdeman group, the derived optimum |
| `shape_location` | 10 | **0.00e0** | every argmin / argmax index |
| `discrete` | 28 | **0.00e0** | τ(ξ) knot counts and the stagnant-branch census |
| `residual` (ABS) | 33 | 2.19e-16 | the two reduce residuals and the τ_mix cancellation |
| `dp` / `zn` | 150 | ≤4.4e-15 | the design points and the public wiring |
| `field` | 151 | ≤6.1e-13 | the resolved fields and everything read off them |
| `spectrum` | 60 | ≤1.0e-12 | the τ(ξ) interpolators and their β-PDF means |

**THE HEADLINE IS THAT TWO SOURCE CLAIMS OF EXACTNESS ARE BACKWARDS OR WRONG, FOR ONE REASON.**
Rung 24 applies an operation INSIDE an accumulation and removes it OUTSIDE, twice:

1. **The `g` reduce** — § 4.7's pre-registered measurement, confirmed by the port. Rung 23 is
   EXACT, rung 24 is not, and the whole ~1e-17 is `sum(sum(r) for r in xi)` against rung 22's flat
   pass. Substituting a flat mean reproduces rung 22 bit-for-bit at every point measured.
2. **The `τ_mix` cancellation** — found DURING the port, not pre-registered. The docstring says
   `⟨τ⟩ = τ_mix·F(C)` **EXACTLY**; production forms `Σ(τ_mix·X_i)` and divides the mean by `τ_mix`
   afterwards instead of scaling once, so `F` moves by up to 2.4e-14 relative across three decades
   of `τ_mix`. Algebraically exact, arithmetically not.

Both are gated **from both sides** — the port must reproduce the inexactness, because tidying
either into its exact form would be MORE accurate than the source and is therefore a defect.

**FIVE FURTHER FINDINGS.**

3. **RUNG 24's `u < 1e-8` BRANCH IS THE INVERSE OF RUNG 20's FLOOR.** Rung 20's flame-band clip
   never binds at the shipped design point and needed a second, cooler one before its gate meant
   anything. This one is taken by **18–50 % of cells** at every J, because the β-clip creates
   large exactly-flat plateaus where `|∇ξ|²` is precisely zero. Its census is a DISCRETE key,
   U-shaped with its minimum at `C_opt` — recorded as CORROBORATION of `F`'s U and explicitly NOT
   as a second kill test, because `u` carries the same `1/var` coupling that makes
   "argmin F == argmin g" circular. The g-free witness stays `⟨|∇ξ|²⟩`.
4. **ONE OF THE TWO DISCRETE KEYS IS A TRIPWIRE, NOT A DISCRIMINATOR — and it is recorded as one.**
   The τ(ξ) binner keeps only NON-EMPTY bins, so its knot count *could* be an output of the field.
   Measured across five grids (16→96) and six decades of `J` (0.05→5000): **every bin is non-empty
   at every point**, so the count is always exactly `max(8, ny/2)`. The Gaussian-plume field covers
   its own ξ range densely and nothing in the shipped regime empties a bin. The key is kept —
   a count off by one would reshape the whole spectrum invisibly to any τ tolerance — but the
   comment justifying it originally claimed the coarse grid was chosen because the count "has the
   most room to differ" there, which the measurement does not support. Corrected in the dump and
   in the gate. Same "a bar that cannot fail is not a bar" shape as the ≤1-of-N guard slice B
   deferred, and the honest move is to name it rather than to leave the justification stronger
   than the evidence. (The stagnant census, by contrast, is live: it moves with `J` and bottoms at
   `C_opt`. And the mean-preservation guard DOES fire out at `J ≈ 5e4`, where the over-penetrating
   plume needs an air scale past the bisection's ceiling of 50.)
5. **A RELATIVE BAR IS THE WRONG CURRENCY FOR A RESIDUAL, AND IT FAILED LOUDLY.** The CPython arm
   reported a worst relative disagreement of **1.60** on `d24/n16/J1.0` — a difference of two
   nearly-equal numbers, where the operands' last bits set the whole answer. Those keys are now
   compared ABSOLUTELY (measured worst 2.19e-16). Same lesson the golden fingerprint gate learned
   from the other side.
6. **AN INVENTED BAR FAILED INSIDE THE HOUR, TWICE.** A 1e-14 relative bar on the rung-24 residual
   (measured 1.10e-14) and a 1.05 clearance bar on the g-free witness's peak (measured **1.0137** —
   the peak clears its low neighbour by 1.4 %, the thinnest margin any location key in this slice
   carries). The second fix is the instructive one: the peak was widened onto a **4× grid** where
   the clearances become 19.5 % and 47.8 %, rather than the bar being loosened. **A location key
   that sits close to its neighbours needs a coarser grid, never a looser bar** — slice C's rule,
   applied a second time and now on a kill test rather than on a hump.
7. **MY OWN FIRST GATE OVER-CLAIMED, AND THE WIDER SWEEP CAUGHT IT.** The first version of the
   reduce gate swept 3 J × 3 grids, found rung 24 inexact 9/9, and stated "never bit-equal" as a
   law. The oracle's wider sweep (adding `ny=16` and `J ∈ {1, 400}`) found **two points where the
   two summation orders round together**. Fifth consecutive slice where sweeping past the first
   gate written changed what could be claimed.

**THE THIRD AND FOURTH VACUITY CASES, both declined rather than transcribed.** § 4.7 pre-registered
`test_no_C_opt_knob_it_is_derived` (a `TypeError` that in Rust is a compile error); rung 23's
`test_helper_matches_production` is the fourth, and the direct twin of rung 16's from slice C. Both
are replaced by statements the Python cannot make: `c_opt()` TRACKING `1/(4k_p²)` as `k_p` moves
with the argmin following it, and — for rung 23 — that a CONSTANT `TauSpectrum` reproduces the
scalar path bit-for-bit (the new `Dwell` enum's own reduce) and that the matched-mean arm IS rung
16's closure at a derived scalar. All four can fail; the transcriptions could not.

**A GRID CONVENTION AND A CROSS-SLICE INTERACTION, both recorded because they will recur.** The
three configs ship DIFFERENT default grids (22 → 48/48, 23 → 40/40, 24 → 48/48), so any comparison
of two closures' `g` at defaults fails for a reason that is not a defect; every such comparison
passes grids explicitly and says so. And **slice C's `n_quad` floor binds inside slice D**: the two
gates that sweep `J` wide enough for the resolved width to reach the top of the `g` range panic
inside the quadrature's own mean-preservation guard at `n_quad = 64`, and run at 160.

**One more grid distinction, which is the rung's own physics rather than a porting detail.** Rung
22's WIDTH minimum is broad, so a ~2× coarse grid locates it safely; its EMISSIONS minimum is
NARROW, because the derived floor sits just below the ideal-bell hump peak. Probed at J=36 instead
of 25 the emissions reading is monotone (1.1860 / 1.1767 / 1.1639) — a true statement about a
different question. Two quantities, two grids, and the reason is in the spec.

### 4.7 SLICE D (rungs 22/23/24) — PRE-REGISTERED before any code was written

Recorded here **before** the port, because three of the four decisions below are vacuity traps of a
kind slices B and C already paid for, and one is a source claim that measurement **inverted**.

**The already-measured inversion — WHICH REDUCE IS EXACT IS BACKWARDS FROM THE DOCSTRINGS.**
Rung 24's `_spatial_local_field` docstring says its `g` is "IDENTICAL BY CONSTRUCTION — not to a
tolerance (contrast rung-23's `_spatial_dwell_field`, which re-derives it through a time development
and matches to <1%)". Measured at J ∈ {4, 16, 100} × ny=nz ∈ {32, 40, 48}, nine points each:

| | vs `_spatial_segregation` | |
|---|---|---|
| rung 23 (`_spatial_dwell_field`) | **exactly equal, 9/9** | the one the docstring only claims to <1% |
| rung 24 (`_spatial_local_field`) | **never equal**, ≤1.9e-16 | the one the docstring claims by construction |

Both directions are wrong, and the mechanism is **summation order**, not physics:

- Rung 23 reaches the terminal field through `_plume(1.0)` — `delta_f * 1.0**(1/3)` and
  `sig_f * sqrt(1.0)` are bit-identical to rung 22's, and it then accumulates `sxi`/`sxi2` in the
  same flat single pass. Exact, at every grid.
- Rung 24 builds the field as a list of rows and takes `sum(sum(r) for r in xi)` — a **hierarchical**
  sum (row partials, then summed) where rung 22 runs one flat accumulator. Same value in exact
  arithmetic, different rounding. Hence ~1e-17.

Three consequences, all load-bearing for the port:
1. **The Rust must reproduce the hierarchical sum**, not "tidy it up" into a flat one. A flat sum
   would be more accurate and would be a DEFECT — it is exactly the class of transcription error
   § 4.2 caught only because the bar is bit-equality. `sum(sum(r) for r in xi)` ports as
   `xi.iter().map(|r| r.iter().sum::<f64>()).sum::<f64>()`.
2. **The two reduces get DIFFERENT bars**: rung 23's is asserted bit-exact (tighter than the
   Python's own `< 1e-9`), rung 24's needs a tolerance. The source asserts `< 1e-9` on both, which
   is why its own gate cannot see that they differ in kind.
3. **The rung-24 docstring is wrong as written** — a different category from § 4.5's two, which were
   merely stale. Recorded, not edited (the oracle policy); the *Rust* doc comment states the
   measured position instead.

**The four gate decisions.**

1. **Rung 22's `C_opt` is located by GRID ARGMIN**, not a root-find — `_argmin_C` sweeps J
   log-spaced over [1, 400]. At the Python's `npts` of 49/81 the neighbours sit ~3.8–6.4% apart in
   `C` around a quadratic minimum, which is the configuration § 4.5 finding 2 says not to ship a
   location key on. Rung 24's own gates already show the house style — `J ∈ {4, 9, 16, 36, 64}`,
   ~1.8–2.25× apart, with `C_opt = 2.5` landing exactly on a node. The rung-22 location key uses
   that grid shape; the fine sweep is dumped as VALUES.
2. **Rung 24's finding is a SPLIT** — `F(C)` U-shaped with an interior minimum, `⟨EI⟩(J)` monotone.
   Both halves are asserted off the SAME sweep, plus the part that makes the pair non-vacuous: the
   two argmins must land at DIFFERENT J (F interior, ⟨EI⟩ at an end). Without that, one quantity
   wired into both slots passes.
3. **Rung 23's matched-mean twin is already in production** (`ei_no_spatial_dwell_meanfield`,
   `corr_ratio`), so the kill test is a sign assert — plus a bar that the two terms differ by more
   than the tolerance floor. A τ(ξ) accidentally wired flat gives `corr_ratio` exactly 1.0, which
   would otherwise read as "no correlation effect" rather than "instrument dead".
4. **Rung 24's Python suite has NO `at_most_one_closure` gate.** All three Rust suites get the
   ≤1-of-EIGHT guard, so the rung-24 one sweeps past the source's gate — the fourth consecutive
   slice to do so.

**THE SUMMATION SHAPE IS PER-LINE, NOT PER-FUNCTION — and the rule above would break the other
moment.** Rung 24 computes its two moments two lines apart in OPPOSITE shapes:

```python
mean   = sum(sum(r) for r in xi) / (ny * nz)             # HIERARCHICAL — row partials, then summed
meansq = sum(v * v for r in xi for v in r) / (ny * nz)   # FLAT — one accumulator over all cells
```

Rung 22 does both flat in one pass, so only the MEAN drifts. Substituting a flat mean reproduces
rung 22 **bit-for-bit at every J and grid measured**, and the hierarchical one never does — so the
entire ~1e-17 is that one line, pinned rather than inferred. Porting "reproduce the hierarchical
sum" as a blanket rule would make `meansq` hierarchical too and inject a SECOND defect on the other
moment. Every accumulator is ported per line. (Rung 23's are all flat, matching rung 22 — which is
*why* it comes out exact.)

**THE DUMP'S GRID CONVENTION, STATED because the next slice will hit it.** The three configs carry
DIFFERENT default grids — `SpatialPDF` 48/48, `SpatialDwellPDF` 40/40, `SpatialLocalPDF` 48/48 — so
any key cross-asserting rung 23's `g` against rung 22's *at defaults* fails for a reason that is not
a defect. The Python's own tests dodge this by passing grids explicitly on both sides, which makes
the mismatch inert in the source's gates but NOT in a dump built from default configs. **The dump
passes grids explicitly wherever two closures' `g` are compared**, and the cross-closure keys live
in one class so the bar is chosen once.

**TWO DISCRETE KEYS, the family slice C's first-burnable-node index opened.** Neither is a value:
- **The non-empty bin count in the τ(ξ) binner** (rungs 23 and 24). `centers`/`taus` are built only
  over bins with `cnts[b] > 0`, so the knot count is DATA-dependent; a count off by one changes the
  interpolator's whole shape and no tolerance on τ would name it. Dumped per J, both rungs.
- **The count of cells taking rung 24's `u < 1e-8` stagnant branch.** Measured, and it is emphatically
  NOT dormant — **18–50 % of cells** take it (`ny=nz=32`: 304/1024 at J=4, **180/1024 at J=16**,
  494/1024 at J=100), because the β-clip creates large exactly-flat plateaus where `|∇ξ|² = 0`. So
  this is the opposite of rung 20's dormant floor: a heavily-taken branch, worth a key on its own.
  Its count is **U-shaped with the minimum at `C_opt`**, which corroborates F's U — but it is NOT a
  second kill test, because `u` carries the same explicit `1/var` coupling. The g-free witness stays
  `⟨|∇ξ|²⟩`.

**THE THIRD VACUITY CASE, AND THE PATTERN NOW HAS A NAME.** `test_no_C_opt_knob_it_is_derived`
asserts that `SpatialPDF(C_opt=2.5)` raises `TypeError`. In Rust an unknown struct field is a
COMPILE error and the crate has no dependencies by decision (§ 3), so there is no `trybuild` and any
runtime transcription measures literally nothing. It is not ported; the Rust asserts the derivation
instead — `C_opt() == 1/(4k_p²)` across several `k_p`, and the argmin TRACKING it as `k_p` moves.
With rung 16's cached-helper test (§ 4.5) and rung 23's `test_helper_matches_production`, that is
**three instances across two slices of one pattern: the source's test guards something the target's
type system or factorisation already guarantees.** A faithful port of such a test is a green test
that measures nothing. The rule: **ask what a ported test could still FAIL for in the new code.**

### 4.10 What phase 3 SLICE E MEASURED — 100 % again, PHASE 3 COMPLETE, and a THIRD "exactly" corrected

Slice E is rungs **14 / 17**: the frozen↔shifting-equilibrium nozzle bracket, and the
combustor-mixing-fidelity ladder of the dropped-NO-clamp margin. It ships the nozzle strand in
`rust/src/nox.rs`, the oracle `rust/oracle/dump_nozzle.py`, and four gates — two rung suites
(24 tests) plus `nozzle_oracle.rs` (4) — and discharges three gates `rung20.rs` had deferred.
**The Rust suite is now 296 tests, and phase 3 is complete.**

| | bit-identical vs PyPy | vs CPython |
|---|---|---|
| nozzle oracle (513 values) | **513 / 513 (100 %)** | 151 / 513 (**29.4 %**) |

**The CPython agreement is the LOWEST of any slice** — 29.2 %, against slice D's 52.2 % and slice
C's 70.6 % — and structurally so, not by luck. Almost every key here rides a bisection root whose
inner evaluation is the 8-species Newton, and one whole class is a difference of near-equal
numbers. The classes that are EXACTLY equal on both interpreters are the ones every slice has
found: the discrete counts (22 keys, 0.00e0) and the solver-free composition sums (22 keys,
3.5e-15). This is the fifth consecutive slice with that same split.

| class | keys | CPython↔PyPy | what it is |
|---|---|---|---|
| `discrete` | 24 | **0.00e0** | halving counts, the guard census (BOTH branches — § 4.10.1 b), the band-edge index, root counts |
| `prim/comp` | 22 | 3.5e-15 | the station-4 composition and its mass per mol air |
| `conv` / `bp` | 46 | ≤7.6e-12 | the converged frozen root, and the back-pressure ladder |
| `dp` / `prim/thermo` / `prim/x_no_e` | 78 | ≤5.7e-11 | the cycle inputs, the molar S/H sums, equilibrium NO |
| `clamp` / `r17` / `r14c` / `edge` | 194 | ≤9.6e-11 | the clamp diagnostics and the whole rung-17 ladder |
| `nz/bracket` | 56 | ≤9.9e-11 | T9, V9, dV9 for both bracket limits |
| `nz/exit_eq` | 80 | ≤3.6e-10 | the shifted exit composition — worst key is H, a ~1e-24 trace |
| `residual` (ABS) | 17 | 6.0e-11 | the frozen-reduce residual, at both stopping rules |

**FINDING 1 — A THIRD SOURCE CLAIM OF EXACTNESS IS ALGEBRAIC ONLY, AND THIS TIME THE MECHANISM IS
TWO ROUTES RATHER THAN TWO SUMMATION SHAPES.** `_expand_nozzle`'s docstring says its frozen branch
"reduces to the production nozzle's pr-ratio expansion **EXACTLY** (the fixed-comp mixing term
cancels)", and `test_rung14.py` gates that at `< 1e-6` absolute. Measured over eight design points:
**0 / 8 bit-equal**, worst 2.46e-11 m/s in V9 (1.75e-14 relative) and 2.80e-11 K in T9 (2.36e-14).
The source's bar sits about six orders above the number it gates.

The residual then **SPLITS**, which is the part worth having. Re-running the frozen bisection with
its `hi − lo ≤ 1e-13·T` stopping rule made a knob — down to `0`, all 200 halvings — the residual
falls to **2.05e-12 K (2.5e-15 relative)** and **stops there**. So the stopping rule is a factor
4–8 and the FLOOR is the route: a molar entropy sum `Σ nᵢ[s0ᵢ(T) − Ru ln(xᵢp/p0)]` against
production's `t_from_pr` Newton on `antideriv_phi` of the mole-weighted coefficients. The mixing
term does cancel; two ways of adding the same numbers do not.

Gated **from both sides**, as slice D's two were: the reduce is asserted at 1e-12 relative (40×
over measured), the converged residual at 1e-13 and asserted NON-zero, and the bit-equal COUNT is
asserted at 0 of 8 rather than "never" — slice D's finding 7 is the reason a count and not a law.
Recorded in `docs/rung14-spec.md`; the Python is not edited (the standing oracle policy).

**FINDING 2 — RUNG 17's FIRING BAND EDGE IS LOCATED, AND THE LADDER DOES NOT GO DORMANT WITH THE
BULK.** The source is explicit that the firing is in-band and not universal: "as the quench gets
FAST (J→∞) `x_no_quenched→x_no_mix` … so `a_bulk→a_mixed<1` (dormant)". It never measures where,
and its suite tests one J. Bisected on the sign of `a_bulk − 1`:

| `C_e` | `a_bulk` at the RQL J = 225 | `a_bulk` crosses 1 at |
|---|---|---|
| 0.15 | 4.356 | **J ≈ 3978–4000** |
| 0.20 | 3.272 | **J ≈ 2457–2470** |

The firing survives ~11× past the shipped band and the edge moves 1.6× on an un-pinned entrainment
scale. **But the natural reading of that caveat — that J→∞ sends the whole ladder dormant — is
wrong:**

| `J` | `a_mixed` | `a_bulk` | `a_pocket` | gap | `hides_super_eq` | `ladder_monotone` |
|---|---|---|---|---|---|---|
| 225 (RQL) | 0.01582 | 3.272 | **11.06** | 3.38 | true | true |
| 4 000 | 0.01582 | 0.789 | **12.82** | 16.26 | **false** | true |
| 16 000 | 0.01582 | 0.402 | **14.34** | 35.65 | **false** | true |

**The mechanism is the source's own, one rung down**, and the gate checks it term by term rather
than asserting the outcome: `ei_no_pocket_quench` = term 1 (the mean-field bulk, riding
`τ_mean ∝ 1/√J`, which collapses) + term 2 (the β-PDF integral at `τ_core = τ_res(1+b_u·u)`, which
`PocketQuenchPDF.core_dwell`'s **own docstring** calls an ABSOLUTE residence whose NO penalty
"survives J→∞", with `u` growing off-optimum). Measured term 2 = 0.646 → 0.997 → 1.155 g/kg. So
`hides_super_eq` is a statement about `a_bulk` and `ladder_monotone` is the statement about the
ladder — a distinction the rung's own prose blurs. Checked against the obvious alternative
explanation: the segregation `g` is PINNED at `g_max` = 0.3 from J ≈ 225 upward and unpinned at
J = 25, so the clip is exercised on both sides and the rise is not the width moving. Recorded in
`docs/rung17-spec.md`. **Sixth consecutive slice where sweeping past the source's own gate found
something the source's gate structurally could not.**

**FINDING 3 — A DORMANT GUARD THAT IS ACTUALLY REACHABLE, WHICH IS THE BEST CASE OF A FAMILY THE
PORT KEEPS MEETING.** `_expand_nozzle`'s post-loop assert says the 500 K exit-bracket floor "never
happens here (every exit sits >700 K)". True at shipped conditions; measured, it fires below
`p9/pt9` = **0.025016** at the cool design point and **0.002608** at the hot one, 6.4× and 44.9×
below where the engine runs. Compare the family: rung 20's flame-band floor was dormant and needed
a SECOND, cooler design point before its gate meant anything (§ 4.4); slice D's knot count could
not fire at all and had to be labelled a tripwire (§ 4.8). This one is gated from both sides at
both design points, and its census over a fixed back-pressure ladder (4 of 12 rejected at the cool
point, 1 of 12 at the hot one) is a live integer that MOVES.

**FINDING 4 — THE EIGHTH VACUITY CASE IS THE FIRST THE PORT CREATED RATHER THAN INHERITED.**
§ 4.9 decision 3 pre-registered replacing the Python's monkey-patch gate — which rebinds the global
`_equilibrium_composition` to a constant and asserts the shifting branch then equals the frozen one
bit-for-bit — with a closure passed into the expansion body, and called that "strictly stronger …
it still fails if the branches ever diverge in bookkeeping". **The second half is false, and it is
false because of the first half of the same decision:** once `shifting` is consumed by the closure
builder there is no branch left inside the body, so the constant-closure call and the frozen call
are the SAME CALL. Slice C's finding 5 in a new costume, and the third time the port has met it —
but the first time the port's own factorisation, rather than Rust's type system, dissolved a test
that was real in Python. The shipped gate keeps the tautology only as the SETUP for an arm that
CAN fail: fed a DIFFERENT constant pool (the shifted exit mixture), the same call must MOVE both
T9 and V9 — which is what catches a body that ignored `comp_at` and would silently collapse the
whole bracket. **The rule survives; what failed was asking it before the design was fixed instead
of after.**

**FINDING 5 — ONE OF `rung20.rs`'s THREE DEFERRED GATES WAS NEVER THIS STRAND'S.** Slice B's
`rung20.rs` deferred gates 4/5/6 to "the nozzle strand and the PDF family". Gate 4 (the clamp
margins rise while the denominator is untouched) is genuinely slice E's, and lands here with gate
1's clamp half. Gate 6's ideal-bell half was slice C's and its quench half was always portable.
**Gate 5 — prompt riding the dilution — needed `ei_no_quenched_total`, three lines with no nozzle
code in them, and was portable in slice B.** It is added here and labelled a slice-B omission, and
the header comment that mis-attributed it is corrected in the same edit. Recorded because the
"don't ship a test whose subject is absent" rule is right and its failure mode is the inverse:
a portable gate parked behind an unrelated dependency, which nothing re-checks.

Two smaller things the slice recorded rather than worked around:

- **THE `residual` CLASS IS COMPARED ABSOLUTELY, AND THE CPython ARM ON IT IS A SANITY CHECK
  RATHER THAN A DISCRIMINATOR.** These keys are differences of near-equal numbers, and measured
  across the two interpreters one of them SIGN-FLIPS (−2.27e-13 against +2.27e-13, relative
  disagreement 2.00) while the absolute spread stays at 6.0e-11. Slice D's finding 5 from the same
  side. The bar covers values as small as 2.3e-13, so on those keys the CPython arm constrains
  almost nothing and it is the PyPy arm's bit-equality that pins them — stated in the gate rather
  than left for a reader to work out.
- **THE HALVING COUNT IS A NAMING KEY, NOT A DISCRIMINATOR, AND IT SAYS SO.** Slice D kept its
  knot count on the argument that nothing else could see a change; that argument does NOT hold
  here, because T9 is gated at bit-equality and a mis-shaped loop moves T9. What the count adds is
  that the failure reads "47 halvings instead of 44" rather than "T9 differs in the last two bits".
  Kept for that, labelled as that.

**THE SIZING LEVER, and it is the same shape slice B's was.** `nozzle_flow` reads only
`(far, Tt4, pt4, Tt9, pt9, p9)` — no `phi_primary`, no mixing config, no grid — so T9 and the
COMMON clamp denominator `x_no_e(T9)` are ONE call for the entire φ_p × J × C_e × super_eq_o
sweep. Measured on PyPy: `nozzle_flow` 8 ms, a bulk `zoned_nox` 0.13 s, a full `exhaust_no_clamp`
1.95 s at the source's own coarse grids. The whole dump is 17 s on PyPy and 134 s on CPython.
Without the lever the band-edge sweep pays a nozzle solve per J point that cannot move.

#### 4.10.1 POST-SHIP REVIEW — the lever was ARGUED not measured, and one shipped REASON was wrong

Slice E shipped before its closing review ran (the advisor was unavailable twice). Run afterwards,
it raised four items; two were real, and both are now fixed on top of `e077517`. The oracle grew
511 → **513** keys and is **513/513** bit-exact against PyPy (151/513, 29.4 %, against CPython —
the two new keys are discrete, so exact on both).

**(a) THE ONE ASSUMPTION BIT-EQUALITY IS STRUCTURALLY BLIND TO — and the obvious check for it is
VACUOUS.** The sizing lever above is an argument, not a measurement: nothing verified that hoisting
the nozzle solve out of the sweep is legitimate. It matters more than a normal untested assumption,
because **both** the Python dump and the Rust gate hoist, so an invalid hoist bakes the same stale
denominator into both references and 513/513 passes with both of them wrong. The obvious repair —
re-solve `nozzle_flow` inside the loop and compare — **cannot detect it**: `nozzle_flow` takes no
jet argument in either language, so that compares a pure function to itself. That is vacuity case
#8's shape arriving in the fix rather than in the test, and it was caught before it was written.
The check that BITES is against the **un-hoisted route**: `exhaust_no_clamp` does take the mixing
config and builds its own nozzle internally, and it is run at seven points the sweep shares.
Measured: the denominator and T9 are **bit-identical across all seven** on both interpreters, and
the five shared `a_bulk` values agree bit-for-bit between the hoisted and un-hoisted routes while
being five DISTINCT numbers. Now asserted in both languages, with the distinctness asserted too so
the check cannot go quietly vacuous.

**(b) A SHIPPED REASON REFUTED — the guard census was frozen-only for a FALSE reason, and the fix
ADDS a measurement.** `try_expand_nozzle`'s doc said the census runs on the frozen branch alone
because the shifting branch "would additionally reach the equilibrium Newton's asserts below the
floor, so the count would measure two guards at once". Swept over the back-pressure ladder at both
design points: **false**. The bisection brackets T into `[500 K, Tt9]`, which is inside the Newton's
converging range, so the Newton is never reached out-of-range and the floor guard is the only one
either branch can fire — same guard, same ladder position, same message. The choice was harmless;
its stated reason was not. The census now runs on **both** branches (`guard/…/fires_shifting`), the
Python asserts the message identifies the floor guard, and the Rust asserts the two branches agree
**ratio by ratio** rather than merely in count. The type-level narrowing of `Option` against
Python's `except AssertionError` is real but has **no reachable instance from this entry point**; a
phase-4 caller expanding outside this bracket would be the first, and the doc now says so.

**(c) THE DEFERRAL SWEEP, RUN AND CLEAN.** § 4.10 finding 5 recorded a portable gate parked behind
an unrelated dependency, and phase 3's completion is exactly when to check for more: every
intra-phase-3 dependency now exists. Swept every Rust test and source file for deferral markers —
five hits, all decisions rather than parked work (three vacuity-register entries, one rung-30 item
correctly beyond phase 3, one *physics* deferral inherited from the source). Rung 20's gate 5 was
the only instance.

**(d) THE BLUR FIXED WHERE A READER MEETS IT.** Finding 2 diagnosed rung 17's prose as conflating
the two margins, then appended a SHARPENING block *below* the conflating sentence. Both earlier
passages now name `a_bulk` as the un-pinned one and point forward; `CLAUDE.md`'s rung-17 row goes
from "bulk + per-pocket FIRE" to "bulk fires IN-BAND, per-pocket everywhere" (+19 B against 61 B of
headroom; the row's no-numbers rule keeps `J ≈ 2460` out of it).

**STILL OPEN, and deliberately NOT started: a vacuity retro-audit of slices A–D.** Case #8's
mechanism is the port's own factorisation dissolving a comparison, and slices A–D pre-registered
*before* measuring — so the rule that caught it here was applied to the source's design and never
re-applied to theirs. That is an audit, not a check, and phase 4 is unauthorised; it is listed here
as a candidate rather than run.

### 4.9 SLICE E (rungs 14/17, the NOZZLE STRAND) — PRE-REGISTERED, and the three bars MEASURED first

Recorded **before** any Rust was written, and — the change from § 4.7 — **after** the measurements the
decisions depend on. Slice D pre-registered one already-measured inversion; this slice makes that the
rule, because every gate below would otherwise have transcribed a bar nobody had checked. Four
probes ran on the Python first (`PyPy`, the gate interpreter); all four moved a gate.

**PROBE 1 — the frozen reduce is NOT exact, and the source's own bar is five orders too slack.**
`_expand_nozzle`'s frozen branch docstring says it "reduces to the production nozzle's pr-ratio
expansion **EXACTLY** (the fixed-comp mixing term cancels)", and `test_rung14.py` gates it at
`< 1e-6` absolute. Measured over Tt4 ∈ {1300, 1500, 1800, 2200} × losses {on, off}:

| | worst |
|---|---|
| `\|V9_frozen − V9_production\|` | 2.46e-11 m/s (1.75e-14 relative) |
| `\|T9_frozen − T9_production\|` | 2.80e-11 K (2.36e-14 relative) |
| bit-equal | **0 / 8** |

**And the residual SPLITS, which is the part worth having.** Re-running the frozen bisection with its
`hi − lo ≤ 1e-13·T` stopping rule made a knob — 1e-13 → 1e-14 → 1e-15 → 1e-16 → 0 (all 200 halvings)
— the residual falls to **2.05e-12 K (2.5e-15 relative)** at Tt4=1500 and 2.27e-12 K (1.8e-15) at
2200, **and stops there.** So the shipped stopping rule contributes a factor 4–8, and the irreducible
floor is the **ROUTE**: the molar entropy sum `Σ nᵢ[s0ᵢ(T) − Ru ln(xᵢp/p0)]` against production's
`t_from_pr` Newton on `antideriv_phi` of the mole-weighted coefficients. Algebraically the same
number; arithmetically two different functions, exactly as `x*x*x` is not `x ** 3` (§ 4.2).

**The "EXACTLY" is therefore ALGEBRAIC — the third of that family, after slice D's two** (§ 4.8), and
the first where the inexactness is two different *routes* rather than two different *summation
shapes*. Consequences, all pre-registered:

1. The rung-14 reduce gate is **1e-12 RELATIVE** (≈40× over the measured 2.4e-14), not the source's
   1e-6 absolute. A bar six orders above the thing it measures cannot tell a defect from noise, which
   is § 4.2's whole lesson.
2. A **SECOND** gate states what the Python cannot: rebuilt independently on the public
   `mix_entropy_molar` with the bracket driven to full convergence, the residual is **still nonzero**
   and ≤1e-13 relative. The Python's tolerance is a literal inside a private function, so its suite
   cannot ask this question at all.
3. That second gate is asserted **only at the two design points it was measured at**, and the oracle
   dumps the converged residual at all eight so the bit-equal COUNT is reported rather than assumed.
   Slice D's finding 7 — a first gate that stated "never bit-equal" as a law and was refuted by the
   wider sweep — is the reason this is a count and not a claim.

**PROBE 2 — the 500 K exit-bracket floor is REACHABLE, at BOTH shipped design points.**
`_expand_nozzle`'s post-loop guard says "Never happens here (every exit sits >700 K)". True of shipped
conditions, and the port now says where it stops being true — bracketed on `p9/pt9`:

| design point | Tt9 | guard FIRES below | shipped `p9/pt9` | margin |
|---|---|---|---|---|
| Tt4 = 1500 K | 1262.7 K | **0.025016** | 0.1591 | 6.4× |
| Tt4 = 2200 K | 1991.8 K | **0.002608** | 0.1170 | 44.9× |

This is the **best-behaved** member of the dormant-guard family the port keeps meeting: rung 20's
flame-band floor needed a second, cooler design point before its gate meant anything (§ 4.4), and
slice D's knot count could not fire at all and had to be labelled a tripwire (§ 4.8 finding 4). This
one is gated **from both sides at both design points** — a `should_panic` at a `p9` past the edge and
a pass at the shipped one — so neither half is a bar that cannot fail.

**PROBE 3 — rung 17's firing band edge, LOCATED. The source states it and never measures it.**
The `exhaust_no_clamp` docstring says the firing "holds across the RQL J-band but is NOT universal —
as the quench gets FAST (J→∞) `x_no_quenched→x_no_mix` … so `a_bulk→a_mixed<1` (dormant)". Measured,
by bisecting `J` on the sign of `a_bulk − 1`:

| `C_e` | `a_bulk` at the RQL J = 225 | `a_bulk` crosses 1 at |
|---|---|---|
| 0.15 | 4.356 | **J ≈ 3978–4000** |
| 0.20 | 3.272 | **J ≈ 2457–2470** |

So the firing survives ~11× past the shipped band, and the edge MOVES by 1.6× on an un-pinned
entrainment scale — which is the source's "rides on un-pinned mixing scales" made numerical.

**PROBE 4 — and the ladder does NOT go dormant with the bulk. This is the slice's finding.**
The natural reading of that caveat is that `J→∞` sends the whole ladder dormant. It does not:

| `J` | `a_mixed` | `a_bulk` | `a_pocket` | pocket/bulk gap | `hides_super_eq` | `ladder_monotone` |
|---|---|---|---|---|---|---|
| 225 (RQL) | 0.01582 | 3.272 | **11.06** | 3.38 | true | true |
| 4 000 | 0.01582 | 0.789 | **12.82** | 16.26 | **false** | true |
| 16 000 | 0.01582 | 0.402 | **14.34** | 35.65 | **false** | true |

`a_pocket` RISES while `a_bulk` falls through 1, so past the crossing the rung-17 headline predicate
`hides_super_eq` (defined on `a_bulk`) goes FALSE while the ORDERING survives everywhere measured.
**The mechanism is the source's own, one rung down:** `ei_no_pocket_quench` = term 1 (the mean-field
bulk, riding `τ_mean ∝ 1/√J`, which collapses) + term 2 (the β-PDF integral at
`τ_core = τ_res(1+b_u·u)`, which `PocketQuenchPDF.core_dwell`'s docstring calls an ABSOLUTE residence
whose "NO penalty survives J→∞", and `u` GROWS off-optimum). Measured term 2 = 0.646 → 0.997 → 1.155
g/kg across those three J. **So rung 17's J→∞ caveat is exactly right about `a_bulk` and incomplete
as a statement about the LADDER**, and rung 16 already contains the reason. The port gates the
reconciliation; the correction is recorded in `docs/rung17-spec.md`, not edited into the Python
(the standing oracle policy, § 4.5).

Checked while measuring it, because the alternative explanation is a width effect: the segregation
`g` is **pinned at `g_max` = 0.3 from J ≈ 225 upward** (raw `k_g·|C−C_opt|` = 2.06 at J=225, 22.97 at
J=16 000) and is NOT pinned at J = 25 (0.1875). So the clip is exercised on both sides and the
`a_pocket` rise is not the width moving.

**PROBE 5 — `dV9_frac` is monotone on a grid 3.7× finer than the source's.** `test_rung14.py` asserts
`fracs[0] < fracs[1] < fracs[2]` on Tt4 ∈ {1500, 1800, 2200}. Measured strictly monotone on **11
points** over 1300–2300 K, 9.86e-6 → 7.90e-3, with `co_fraction_entry` 1.44e-7 at the cool end. The
hot-anchor band `3e-3 < dV9_frac < 8e-3` holds (4.38e-3 at 2200 K). Nothing moved; recorded because
a monotonicity claim on three points is the shape slice B narrowed rung 12 on, and this one survives.

**THE SIX PORT DECISIONS.**

1. **It lands in `nox.rs`, not a new module.** Rung 17 is nine NO fields plus `T9`/`x_no_e_exit`, and
   is three `zoned_nox` calls; `nozzle_clamp_diag` calls `equilibrium_no_fraction`, which already
   lives there. A separate module buys a circular dependency or a 200-line orphan. Phase 4's marches
   (rungs 25–30, six rungs) are the second consumer of `mix_entropy_molar`/`mix_mass_per_air`/
   `mix_h_abs_b` and are where that module decision belongs — **not pre-built for a phase that has
   not been scoped.** Recorded so phase 4 does not read this as settled against it.
2. **THE BISECTION'S LOOP SHAPE IS A NAMED HAZARD, and it is three hazards.** `_expand_nozzle` runs
   `for _ in range(200)`, takes the midpoint at the TOP, updates the bracket, then breaks on
   `hi − lo <= 1e-13 * T` where `T` is **this iteration's pre-update midpoint** — and computes
   `T9 = 0.5*(lo+hi)` **after** the loop, from the final bracket. An idiomatic `while hi-lo > tol`
   rewrite gets all three wrong (it can do zero halvings, it tests a different midpoint, and it
   returns the last `T` rather than the final bracket's), and each is worth one bracket quantum —
   invisible to any tolerance, which is slice A's `2500/2³²` shape. **Checked while registering it:
   `primary_aft` and `mixed_out_t`, ported in slices A/B, both have the shape right** (`for _ in
   0..100`, midpoint at the top, break after the update, `0.5*(lo+hi)` post-loop). The discipline is
   holding; this records the evidence rather than the intention. Note also `<=`, not `<`.
3. **The monkey-patch becomes a CLOSURE, and production must go through it.** `test_rung14.py`'s
   gate 1b rebinds the module-global `_equilibrium_composition` to freeze the shift, then asserts the
   shifting branch equals the frozen one bit-for-bit. Rust gets `expand_nozzle_with(comp_at: impl
   Fn(f64) -> Vec<(&'static str, f64)>, …)`, and `expand_nozzle(comp_entry, far, …, shifting)` is a
   thin wrapper that builds the closure and calls it. **If the two were separate paths the test would
   compare the closure path to the bool path and prove nothing about production** — the same trap as
   slice C's helper-matches-production.

   > **CORRECTED DURING THE PORT — and the correction is vacuity case #8.** This entry originally
   > said the closure version is "strictly stronger than the Python's: no global mutation, and it
   > still fails if the branches ever diverge in bookkeeping." The second half is false, and it is
   > false *because* of the first decision in this same paragraph. Once `shifting` is consumed by
   > the closure builder there is no branch left inside the expansion body, so
   > `expand_nozzle_with(constant closure)` and `expand_nozzle(shifting = false)` are the SAME CALL
   > and asserting they agree compares a function to itself. Slice C's finding 5 in a new costume
   > — "a better factorisation turns the source's real pin into a self-comparison" — and the third
   > time the port has met it. **The shipped gate keeps the tautological half only as the setup for
   > an arm that CAN fail: fed a DIFFERENT constant pool (the shifted exit mixture), the same call
   > must MOVE both T9 and V9.** That is what catches a body which ignored `comp_at` and read
   > `comp_entry` throughout — a defect that would silently collapse the whole rung-14 bracket, and
   > which the tautological half passes happily.
4. **VACUITY CASES FIVE, SIX, SEVEN AND EIGHT — the register is now five slices long, and
   case #8 is one this slice's own design CREATED (see decision 3's correction).**
   * **#5, rung 14:** `assert nf.comp_exit_eq is not nf.comp_entry` — a Python identity check, put
     there to catch "a silent solver return of the entry pool". In Rust `comp_exit_eq` is a `Vec`
     and is always a fresh allocation, so the transcription is green by construction. Replaced by the
     physics it was proxying: the exit pool differs from the entry pool **as values**, with CO down
     and CO2 up.
   * **#6, rung 17:** `test_identity_is_witnessed_not_a_test` — and its own file header says it:
     "witnessed, not gated … Reported; NOT a discriminating test." It compares `a_pocket/a_bulk` to
     `gap_pocket_over_bulk`, both built from the same two EIs over the same `xe`. It cannot fail in
     Python either; in Rust it is the same tautology in a new costume. Replaced by a round-trip that
     CAN fail: `kappa · ei_no_quenched` must reproduce `x_no_bulk_quench`, and `a_pocket` must equal
     `x_no_pocket / xe` with `xe` taken from an INDEPENDENT `nozzle_flow` call rather than off the
     same state.
   * **#7, rung 17:** `test_requires_both_configs` passes `mixing=None` / `pocket_quench=None` and
     asserts each is rejected. Rust takes both **by value**, not as `Option`, so the requirement is a
     COMPILE error — rung 22's `TypeError` probe in a new costume (§ 4.8). Not ported; the Rust
     asserts the guard that remains runtime, `p9 ≤ pt9`, and the equilibrium-gas requirement.
   With rung 16's cached helper (§ 4.5), rung 23's `test_helper_matches_production` and rung 22's
   `TypeError` probe (§ 4.8), that is **eight instances across three slices**, and the register has
   not gone a slice without gaining one. Case #8 is the first the PORT created rather than
   inherited, which is worth separating: #5–#7 are the source's tests meeting Rust's type system,
   #8 is a factorisation of mine dissolving a test that was real in Python. The rule
   ("ask what a ported test could still FAIL for in the new code") catches both, but only if it is
   re-asked AFTER the design is chosen — here it was not, and the pre-registration said the
   opposite for a day.
5. **RUNG 20's DEFERRED GATE 5 IS NOT THIS SLICE'S, and the comment blaming the nozzle strand is
   wrong.** `rust/tests/rung20.rs`'s header defers gates 4/5/6 to "the nozzle strand and the PDF
   family". Gate 4 (the clamp margins rise, denominator untouched) and gate 1's clamp half ARE slice
   E's and land here. Gate 5 is the prompt-through-dilution invariant, which needs
   `ei_no_quenched_total` — a three-line `Option<f64>` on `ZonedNoxState` with nothing to do with the
   nozzle. **It was portable in slice B and was bundled into the deferral by mistake.** It is added
   here and labelled as a slice-B omission; the header comment is corrected in the same edit.
   Leaving a known-portable gate unported is the "shipping untested code" objection inverted.
6. **THE SIZING LEVER, and it is the same shape slice B's was.** `nozzle_flow` reads only
   `(far, Tt4, pt4, Tt9, pt9, p9)` — **not** `phi_primary`, `mixing`, `pocket_quench`, `super_eq_o`
   or any grid. So `T9` and the common clamp denominator `x_no_e(T9)` are ONE call for the entire
   `φ_p × J × C_e × super_eq_o` sweep, and the three NO numerators sweep against a cached
   denominator. Measured on PyPy: `nozzle_flow` 8 ms, a bulk `zoned_nox` 0.13 s, a full
   `exhaust_no_clamp` 1.95 s at the source's own coarse grids (`n_bell` 20, `n_quad` 64, `ngrid` 24,
   `nsteps` 200). Without the lever every J point on the band-edge sweep costs a nozzle solve it does
   not need.

### 4.12 What phase 4 SLICE F MEASURED — 100 % again, a claim of exactness that SURVIVED, and a test bar that does not

**912 / 912 bit-exact against PyPy**, first run, including exit states 400 chained solves deep.
The Rust suite is now **330 tests**. Slice F is rungs 25/26; slices G (27/28) and H (29/30) follow.

**FINDING 1 — the marches AMPLIFY the interpreter gap, which is what makes the 100 % sharp.**
Against **CPython** the same dump is only **493 / 912** bit-identical (54.1 %), and the split by
quantity is the story: `clock/` (solver-free, closed form) is 90/90 identical, while **velocity is
3 / 88** and temperature 56/92. A march is not a solve — it re-solves the equilibrium composition
and bisects a temperature 100 or 400 times over, so a last-bit difference at step 1 is still being
carried at step 400. **Bit-equality on these keys is therefore a much stronger statement than the
same words were in slice E**, where the quantities were single solves. It also disposes of the
standing worry that a 100 % result might mean the sweep is insensitive: on the very same keys, two
Pythons disagree.

**FINDING 2 — the FOURTH "exactly"-class claim in this lineage, and the first to SURVIVE.**
§ 4.11 probe 4 measured `_freeze_out_expand`'s "reproduces `_finite_rate_expand(Da)` **to the
ULP**" at 40/40 bit-exact on the Python, and the Rust reproduces it at 40/40 between its own two
functions (`tests/rung26.rs::constant_da_local_is_rung25_bit_for_bit`, against the Python suite's
6 cells). Slices C, D and E each corrected a claim of exactly this shape — 3 for 3 — so the prior
was that this would be the fourth correction. **It is not, and the reason is structural rather
than lucky:** the two loops are a deliberate verbatim duplication, so the reduce compares two
identical instruction sequences rather than two routes to the same number. Slice E's corrected
claim failed precisely because it compared two ROUTES (the entropy scale against the production
nozzle's Newton). **The distinction to carry forward: "exactly" survives a COPY and does not
survive a REDERIVATION.**

> **AND THE DUPLICATION IS LOAD-BEARING IN THE PORT, NOT JUST THE SOURCE.** Factoring the two
> Rust loops into one generic body would compile, pass the oracle at 912/912, and silently turn
> this gate into a self-comparison — vacuity case #8, whose mechanism is the port's own
> factorisation dissolving the source's pin. `march.rs`'s header says so at the top, because the
> next reader's instinct will be to remove the copy.

**FINDING 3 — a shipped TEST BAR that only holds where it was evaluated.** `test_rung26.py`'s
2nd-law gate asserts `dS_freeze > -1e-6` at `Tt4 ∈ {1500, 1800, 2200}`. Swept over the oracle's
five design points:

| `Tt4` | 1300 | 1500 | 1800 | 2200 | 2300 |
|---|---|---|---|---|---|
| `dS_freeze` | **−2.077e-05** | +1.211e-04 | +3.202e-03 | +3.947e-02 | +7.056e-02 |
| `Da_entry` | 0.0654 | 0.3098 | 1.4465 | 4.5012 | 5.1538 |

**At 1300 K the bar fails by 20×.** Not a port defect — the oracle gates `fz/cold/dS` bit-exactly
and it passes, so the Python computes the same number; the bar was simply never evaluated below
1500 K. The mechanism is the one § 4.11 probe 3 identified from the other side: the anchored clock
never switches on at 1300 K (`Da_entry` = 0.065), so there is almost no relaxation and therefore
almost no entropy to produce, and the trapezoid truncation is then LARGER than the physical signal
and sets its sign. `-1e-6` sits in the gap between the two regimes.

The Rust suite ships the three statements that are actually true, separately rather than conflated
into one threshold: `dS` clears the code's OWN floor (`DS_FLOOR` = −5e-3) everywhere, with the
worst point clearing by 240×; `dS > 1e-4` strictly wherever `Da_entry > 1`, i.e. wherever there is
relaxation to produce it; and **`dS` is MONOTONE in `Tt4` across the whole ladder** — which the
Python never asserts and which is a far sharper detector than any floor, since it fails if a march
goes wrong anywhere on the ladder rather than only below a bound. **The Python's bar is left
as-is**: no shipped CLAIM moves (rung 26's headline is the moving freeze point, untouched), and
editing the source's tests is not the port's remit. It is recorded here so phase 8's adjudication
does not meet it cold.

**FINDING 4 — a distinct-value bar I guessed was wrong, and the shortfall was the physics.** The
dump's first draft asserted one lumped "≥ 80 distinct clock values" over the anchored clock's three
arms and failed at 66. The arms are not comparable: with the density pinned, `τ_chem` loses its
only pressure dependence, so that arm's 6 × 5 grid holds **6** values, not 30 — the kill test
working exactly as designed. A lumped bar hid a real structural fact behind a threshold nobody had
checked. The dump now gates each arm at its own count (`free` 30, `killT` 30, `killM` 6) and, since
`τ_free ∝ T⁴/p²` and `τ_killT ∝ (T/p)²`, **the pressure ladder was moved off round numbers** so
those counts are structural: on `{2e4, 5e4, …}` the cells `(800 K, 2e4 Pa)` and `(2000 K, 5e4 Pa)`
share `p/T` = 25 and `killT` silently held 29. Encoding *that* as the bar would have pinned a
floating-point coincidence that need not survive a change of interpreter. **This is the
measure-before-registering rule failing at the one place slice E did not apply it — to the dump's
own census bars — and it is the second time in this port a "count" bar has been the thing that was
wrong.**

**FINDING 5 (post-ship review) — A KEY CLASS DOCUMENTED TWICE, COMPUTED, AND EMITTED BY NOBODY.**
The dump's header described an `iters/…/min|max` discrete-key class in two passages, and
`March` computed both fields every step behind a paragraph justifying them. Nothing emitted them
and nothing read them; the Python cannot report a halving count without instrumenting the source,
so the class was documentation for a gate that did not exist. **The oracle's own key-COUNT guard
could not catch it** — it asserts `rust.len() == oracle.len()`, and with the class absent from
BOTH sides the totals matched at 912. A guard against drift between two sides is blind to
something missing from both.

Resolved by making the counts earn their place rather than deleting them, because there is exactly
one thing they say that no dumped value can: **whether the loop converged.** `used == 200` means
the bracket never met its stopping rule, and that is invisible in the result — `0.5*(lo+hi)` off an
unconverged bracket is a plausible temperature, and a Python↔Rust dump would see both sides agree
on the same unconverged number. The counts now gate a band (measured 36–37; the gate is 20 < n <
60) in `tests/rung25.rs::the_energy_bisection_converges_far_inside_its_cap`, which also catches
the inverted stopping rule that would break out immediately.

**THE PORT DECISIONS TAKEN, all four as pre-registered in § 4.11**: a new `march.rs` (rung 30 will
go to `components.rs`); `choked_mfp` deferred to phase 5, checked against the tests rather than
assumed; the two non-oracle reduces gated as named tests; and the three bisection tolerances
transcribed separately, with `equilibrate_hp` dumped DIRECTLY as well as through its caller so a
mis-transcribed `1e-10` names the equilibration instead of reading as a nozzle defect.

### 4.14 What phase 4 SLICE G MEASURED — 100 % again, the DISCRIMINATOR held, and a third fitted bar

**776 / 776 bit-exact against PyPy**, first run. The Rust suite is now **361 tests**.

**FINDING 1 — the sharpest dump in the port, by a wide margin.** Against **CPython** only
**62 of 776** values are bit-identical — **8.0 %**, against slice F's 54 % and slice E's far higher.
Every quantity here is a ratio of Arrhenius rates evaluated on a marched trajectory, so there is
nothing insensitive in the sweep to inflate the agreement: `Da` 0/106 identical, `a` 0/75, `max_a`
0/43, the clock 0/90. The two interpreters disagree on essentially every value, and Rust matches
PyPy on every one.

**FINDING 2 — slice F's DISCRIMINATOR made two predictions and both held.** § 4.13 registered
`_frozen_T` and rung 28's structural reduce as COPY-class *before* measuring, on the rule that an
"exactly" claim survives a copied instruction sequence and dies on a rederivation. Both survived
(5/5 and 10/10, worst |Δ| exactly 0). The second one sharpens the rule: it holds **despite** rung
27 computing `_equilibrium_no_fraction` once per step where rung 28 computes it twice — same
function, same arguments, same bits. **A COPY is about the arithmetic performed, not the syntax.**

**FINDING 3 — a THIRD bar fitted to the points it was evaluated at.** Rung 27's linearity gate
asserts `|a(2x)/a(x) − 2| < 1e-9`, which is exact — but only because the Python drives it at a
literal ZERO rate, where `x_no` never moves. At the ANCHORED rate the relaxation step is AFFINE,
not linear (the `relax·x_no_e` term does not scale with the input), and the departure measures:

| `Tt4` | 1500 | 1800 | 2200 | 2300 |
|---|---|---|---|---|
| departure at the zoned NO level | 3.11e-10 | 8.81e-07 | 6.76e-04 | 2.18e-03 |

**Nine orders across the ladder**, rising monotonically with the residual relaxation. The source
states the proportionality with neither qualification. The Rust gates both halves — the exact
claim in the reduce, and the SHAPE of the departure at the anchored rate — rather than a
threshold, for the reason § 4.12 finding 3 gives. (A first draft of that arm fitted a bar and
failed, at the *wrong NO level*: the departure is ~10× larger at `x_no` = 1e-4 than at the zoned
value, because the affine offset matters more when the level is smaller. **That is the third time
in two slices a guessed bar has been the thing that was wrong** — the measure-before-registering
rule keeps failing at the same place, which is bars invented while writing a gate rather than
before it.)

**FINDING 4 — the dormant guard, and the argmax that is honestly a tripwire.** The β repair's
`if not isfinite: 1.0` fallback is DORMANT at every shipped condition (0 of 55 sampled cells;
`_tau_no_exact` returns finite even at 400 K), so `tests/rung28.rs` reaches it from the REFUSING
side by removing the radicals — the rung-20 gate-5 lesson applied without being asked. And the
`max_a` ARGMAX is constant at the exit across 5 design points × 4 rate scales spanning `1e-12 …
1e12`, including cells where NO is 97 % relaxed, so the source's "a relaxed one may peak earlier"
hedge is unrealised across the whole reachable range. It is gated Rust-side and labelled a
TRIPWIRE — **not** dumped, because the Python cannot produce an index without instrumentation and
dumping a one-sided class is § 4.12 finding 5 repeated on purpose.

**AND THE SHIPPED CODE CONFIRMS ITS OWN ERRATUM FROM AN UNEXPECTED SIDE.** Rung 27's
`relaxed_fraction` comes out **NEGATIVE** at the hot anchored points (−1.4e-07 at 1800 K, −4.0e-04
at 2200, −1.3e-03 at 2300). A negative relaxation means NO *formed* rather than being destroyed —
exactly rung 28's erratum, that `a < 1` at the entry so NO arrives SUB-equilibrium and initially
tries to form. Rung 27's own output says so and nothing in its suite reads it.

### 4.15 SLICE H (rungs 29/30) — PHASE 4 COMPLETE, and the one slice that was NOT pre-registered

**270 / 270 bit-exact against PyPy.** The Rust suite is **397 tests**, and **phase 4 is complete**:
slices F (25/26), G (27/28) and H (29/30) all shipped — in ONE session against the 2–3 budgeted.

**AND THE FIRST VERSION OF THIS SLICE SHIPPED 6 OF RUNG 29's 16 GATES.** Slices F and G each began
by reading `test_rungN.py` and enumerating its gates; slice H read `gas.py`'s docstrings, the spec,
and `test_rung30.py`'s *header*, and **never opened `test_rung29.py` at all** — which is the
largest test file in the phase at 422 lines. It got the smallest Rust suite (7 tests, against rung
25's 17), and the ten missing gates were the entire `π_c`-margin and `M0`-margin families: the two
sweeps that re-check rung 29's "one design point" concession on the axes it named.

**No self-check in this port could have caught that, and the reason is worth keeping.** The oracle
came back 270/270 and the phase looked complete. **An oracle gates VALUES; a missing gate is a
missing CLAIM, not a wrong number.** Bit-equality says the port computes what the Python computes —
it is silent on whether anything asserts what that computation is *for*. The only detector is
enumerating the source's gates and diffing, which costs one `grep`.

**AND THE SAME ENUMERATION THEN FOUND SIX MORE IN SLICE G.** Running the check across all six
rungs rather than only the one that failed: rungs 25, 26 and 27 were complete by name (10 gates
each, all covered), but `test_rung28.py` has **20 gates and the port held 14** — missing the whole
β-MARGIN family, which is rung 28's own seam re-checked. Those are now ported too, including the
three sharpest: β is **EXACTLY pressure-invariant** (gated at `1e-12` relative over a 640×
pressure span — a claim of exactness that a looser bar could not tell from "roughly flat"), its
whole-plane maximum is **INTERIOR** rather than a scan edge (the ridge must strictly beat both
flanks), and `τ_exact/τ_surrogate` matches its hand-derived closed form pointwise, which is the
only non-tautological check on the algebra the entire repair rests on.

**The pattern in the two misses is worth naming: they were the two LARGEST test files** (422 and
~400 lines), i.e. exactly the ones where reading the source docstrings feels like enough.

The ten rung-29 gates are now ported, along with two clauses of rung 30's gate 3 that the header
summary carried and the first pass dropped (`p* > 3·p0`, and the thrust-loss magnitude). Two bars were also
tightened from round thresholds to the rung's own numbers — `rescued > 0.5` → `0.80–0.94` against
the documented 87 %, and net loss `< 0.15` → `0.03–0.10` against the documented 5–8 %; both pass,
and both would previously have passed on a model with substantially wrong physics.

> **A vacuity the re-aim created, caught on the way in.** The Python runs the two margin sweeps
> through two helpers and gates that they agree at the point they share. This port has ONE
> parameterised helper, so transcribing that assertion compares a function with itself — it passed,
> and meant nothing. It now compares the sweep helper against the design-point construction, which
> really are two different paths here. **That is vacuity case #8 for the fourth time, and every
> instance has been created by the port's own factorisation rather than inherited from the source.**

**THIS SLICE BROKE THE RULE THE OTHER TWO FOLLOWED, and it is recorded because the cost is
visible.** Slices E, F and G each probed the Python first and wrote the bars down before any code.
Slice H went straight from reading the source to porting, and the three census bars were therefore
invented while writing the dump rather than measured before it. All three were wrong:

* `>= 24 distinct sonic-throat roots` — **actual 17**, because `T*` is PRESSURE-INDEPENDENT. The
  residual `h_t(Tt9) − h_t(T*) = ½γ_t(T*)R T*` contains no pressure at all; `pt9` enters only
  through `p*`. So a 6 × 4 grid holds SIX roots per arm. **That is the property rung 31's
  `choked_mfp` is built on ("MFP* is a function of Tt and composition ALONE") showing up one rung
  early** — a real structural fact the guessed bar was hiding.
* `p*` distinct = 24 — **actual 19**. On a CPG gas `T*/Tt9` is constant, so `p* = pt9 · const` and
  the structural count is 4; the constant is reached through `pr_t(T*)/pr_t(Tt9)`, which differs in
  the last bits per `Tt9`, giving 19 by coincidence. **Dropped rather than pinned** — encoding 19
  would fix a floating-point accident that need not survive a change of interpreter, which is the
  same trap slice F avoided by moving its clock ladder off round numbers.
* A fixed ambient ladder for the nozzle census — **impossible at some design points**, because
  `pt9` moves with `Tt4` and 3.0e5 Pa is subcritical at one point and ABOVE `pt9` at another. Now
  set as fractions of the local `pt9`, so both branches are live everywhere.

**Counting the whole phase: five guessed bars, five wrong, and in four cases the shortfall was the
PHYSICS rather than a defect.** The rule is not "measure more"; it is that a bar written while
authoring a gate is a guess wearing a gate's clothes, and the place it keeps happening is census
counts — the one class where being wrong looks like a failing test rather than a wrong number.

**THE FINDINGS.**

1. **RATIO ≠ ENERGY is stronger than the source states it: the two are ANTI-CORRELATED.** Rung 29
   says the frozen station-5 pool is super-equilibrium by a large ratio while the energy available
   is small. Measured across the ladder, the ratio does not merely fail to predict the energy — it
   moves the OTHER WAY:

   | `Tt4` | 1300 | 1500 | 1800 | 2200 | 2300 | span |
   |---|---|---|---|---|---|---|
   | `ΔT5/T5` | 1.69e-05 | 1.07e-04 | 7.65e-04 | 6.13e-03 | 1.04e-02 | **×616 (rises)** |
   | super-eq ratio | 993.5 | 109.4 | 17.7 | 5.21 | 4.25 | **÷234 (FALLS)** |
   | radical inventory | 3.85e-06 | 3.18e-05 | 3.11e-04 | 2.28e-03 | 3.19e-03 | ×828 (rises) |

   The ratio is largest (993×) exactly where the bracket is worth least (1.7e-05), and the
   INVENTORY — what a shifting expansion can actually burn — tracks the energy to within a factor
   1.3 while the ratio moves 234× the other way. `tests/rung29.rs` gates the anti-correlation
   rather than bounds on each series, because a threshold on either alone would pass on a model
   where both rose together, which is exactly the reading rung 29 corrected.
2. **THREE SPELLINGS OF "RAISE TO A POWER" NOW LIVE IN ONE PHASE, and each site takes a different
   one.** `sonic_throat`'s `V* = (...) ** 0.5` is a libm `pow` (differs from `sqrt` about one point
   in 670 — phase 2's trap); rung 26's `math.sqrt(J)` really is the sqrt instruction; rung 28's
   `(1+βa)²` is an integer power that may be a product. Any one rule applied by habit to all three
   is a silent one-bit defect, and the three are within 500 lines of each other in `march.rs` and
   `components.rs`. Each site now restates which rule applies and why.
3. **The two-path gate is protected AND the protection is checked.** Rung 30's gate 2a calls itself
   "two genuinely different code paths onto the same M=1 condition" — but it runs on a CPG gas,
   where `sonic_throat` takes the closed form, so without the separate `sonic_throat_bisect` entry
   point it would compare the closed form with itself. `tests/rung30.rs` asserts the two paths
   AGREE to the bisection's band **and DISAGREE somewhere** — the second half being what proves the
   dispatch has not collapsed. That is the shape `porting_rules.rs` uses to prove `powp` is a real
   call, reused where the same vacuity could arise.
4. **`choked_mfp` stayed out, as pre-registered in § 4.11**, and the check was against the tests
   rather than the plan's word: no rung-30 test references it, so it waits for phase 5 where
   `test_rung31.py` can exercise it.

### 4.13 SLICE G (rungs 27/28, the NO MARCHES) — PRE-REGISTERED, and the first slice to register PREDICTIONS

Slice F's finding 2 produced a **discriminator**, so this slice registers predictions rather than
bare measurements: *an "exactly" claim survives a COPY of an instruction sequence and dies on a
REDERIVATION of the same quantity.* Slice G contains two COPY-class claims and no rederivation, so
**both were predicted to SURVIVE before measuring** — and a refutation would have been worth more
than the confirmation, because it would have broken the rule one slice after it was formed.

**PREDICTION 1 — `_frozen_T` (gas.py:2310, 2357). SURVIVES, 5/5.** The docstring says it is
"byte-identical to `_expand_nozzle(shifting=False)`'s bisection (same bracket, same 1e-13 tol), so
the exit T9 matches `nozzle_flow` bit-for-bit (the reduce hinge)". It is the same loop, not a
second route. Measured bit-exact at every design point, both through the public method and through
`_frozen_no_trajectory` directly, `ΔT` exactly 0.

**PREDICTION 2 — rung 28's structural reduce. SURVIVES, 10/10, worst |Δ| exactly 0.** Feeding
`_frozen_no_trajectory` to `_coupled_no_march` reproduces `_no_freeze_out_expand` — "the two march
the identical expression sequence". It survives *despite* an asymmetry that looks like it should
break it: rung 27 computes `_equilibrium_no_fraction` ONCE per step and uses the value twice, while
rung 28 calls it TWICE. Same function, same arguments, so the same bits — which is precisely what
the rule is about, and a useful sharpening of it: **a COPY is about the arithmetic performed, not
the syntax.**

**MEASUREMENT 3 — the `max_a` ARGMAX is CONSTANT, and the source hedges a case that never
occurs.** `_no_freeze_out_expand`'s comment says "equilibrium NO is monotone in T, so a frozen NO
peaks at the cold exit; **a relaxed one may peak earlier**". Measured over 5 design points × 4
rate scales spanning `1e-12 … 1e12` (`nstep` = 400, so the exit is index 400): the argmax is **400
in all 20 cells**, including those where NO is 70–97 % relaxed. The hedge is unrealised across the
whole reachable range. **So the argmax is a TRIPWIRE, not a discriminator** — the slice-E
classification, and the same honesty slice F's `iters` eventually got. **And, like `iters`, it is
gated RUST-SIDE rather than dumped**: the Python returns `max_a` without an index and cannot report
one without instrumenting the source, so a dumped `argmax/` class would be the § 4.12 finding-5
mistake repeated deliberately. `tests/rung27.rs` asserts the peak is at the exit across the same
20 cells, which is a claim about the port that the oracle could not carry anyway.

> Also measured, and it CONFIRMS rung 28's erratum from an unexpected side: the relaxed fraction is
> **NEGATIVE** at the hot anchored points (−1.4e-07 at 1800 K, −4.0e-04 at 2200, −1.3e-03 at 2300).
> A negative relaxation means NO *formed* rather than being destroyed — exactly the erratum's point
> that `a < 1` at the entry, so NO arrives SUB-equilibrium and initially tries to form. The
> shipped code's own output shows it.

**MEASUREMENT 4 — the β repair's fallback is DORMANT, so the port gates it from the REFUSING
side.** `coupled_no_freeze_out_nozzle` ends its β sweep with `if not math.isfinite(tau_ratio_min):
tau_ratio_min = 1.0`. Measured across the ladder: **0 degenerate samples of 55**, the fallback
never fires, and `_tau_no_exact` returns finite everywhere — even at 400 K, which was the obvious
candidate for degeneracy. That is the rung-20 gate-5 shape (a guard gated only from the accepting
side proves nothing). Forced, the branch IS reachable: zeroing `[O]` and `[H]` gives `(inf, 0, 0)`
from `_tau_no_exact` and `inf` from `_tau_no_destroy`. **The Rust gates both sides.**

**MEASUREMENT 5 — β and the bound, as numbers.** β spans 0.0022 → 0.5429 over the path × ladder,
with the max at `Tt4` = 2300 K — consistent with the 0.5444 plane maximum the source documents, and
comfortably under the 1.0 the whole repair rides on. `τ_exact/τ_surrogate` measures **1.04 … 3.26,
never below 1**, so the surrogate is a genuine lower bound on the relaxation time in every sampled
cell, which is what rung 27's bound claim needed.

**THE PORT DECISIONS.**

1. **Slice G lands in `march.rs` beside slice F** — the module decision is spent, and rung 28
   consumes slice F's `freeze_out_expand` record directly.
2. **`(1.0 + beta*a) ** 2` is an INTEGER exponent**, so it is the one place in this slice that
   spells as a PRODUCT (`x * x`) rather than [`powp`] — the exact inverse of rung 26's float
   `N_HOHM = -2.0`. Both spellings appear within 400 lines of each other, which is why the rule is
   worth restating at each site rather than once per module.
3. **Both COPY-class reduces are gated RUST-vs-RUST**, not through the oracle, for the reason
   slice F established: a Python↔Rust dump compares values and cannot see a loop-shape error
   transcribed identically into both copies.
4. **The dormant fallback is gated from both sides**, and the argmax is dumped as a tripwire with
   its constancy stated rather than implied.

### 4.11 SLICE F (rungs 25/26, the RECOMBINATION MARCHES) — PRE-REGISTERED, five probes MEASURED first

**PHASE 4 WAS AUTHORISED 2026-08-12** (the standing re-decide point in § 9 decision 3 was *before*
phase 4; it is now spent, and the next one is before phase 5). Phase 4 is rungs 25–30 and runs in
**three slices, grouped by DEPENDENCY** exactly as phase 3 was:

| slice | rungs | what it is | depends on |
|---|---|---|---|
| **F** | 25, 26 | the MAJOR-POOL marches: the Damköhler flow between rung-14's bounds, its closed-form fast ceiling, and the anchored GRI-Mech clock that lets the relaxation shut off | slice E's `expand_nozzle` + the three mixture helpers |
| **G** | 27, 28 | the NO marches: a scalar relaxation on the FROZEN path, then that clock re-read on slice F's RELAXING pool | F (rung 28 only — see below) |
| **H** | 29, 30 | the work-limited turbine expansion, and the sonic throat of a convergent nozzle | neither |

**RUNG 27 DOES NOT DEPEND ON RUNG 26**, and an earlier draft of this table said it did.
`_no_freeze_out_expand` marches a single scalar along rung-14's FROZEN isentropic path and never
touches `_freeze_out_expand`. Only rung 28 needs rung 26 — it reads the `record` trajectory. F→G
is still the right order, but for rung 28's sake alone.

**THE PROBES RAN FIRST, AS SLICE E MADE THE RULE** (`M:\claud_projects\temp\rust-phase4\probe_slice_f.py`,
PyPy, the gate interpreter). All five targets are things the source states in WORDS and never
numbers, and each one sets a bar below. The sweep is deliberately wider than the source's own gates:
`Tt4 ∈ {1300, 1500, 1800, 2200, 2300}` against the tests' {1500, 1800, 2200}, and
`Da ∈ {0.03 … 300}` against their {0.3 … 30}.

**PROBE 1 — the `T + 50` upper bracket is SOUND, and its word-bound is now a number.**
`_finite_rate_expand:1967` justifies the temperature bisection's upper bracket with *"bounded by the
whole entry re-equilibration ~10s of K, so T+50 never clips the root"* — a bound with no measurement
behind it. Measured over 70 marches: the largest single-step temperature RISE is **12.956 K**, at
`Tt4 = 2300, Da = 300, nstep = 100, step 0` — the hottest, fastest, coarsest corner, and the FIRST
step, which is exactly where the recombination reheat is concentrated. Headroom to the top of the
bracket never falls below **37.04 K**. Probe 2 measures the re-equilibration the docstring appeals to
at **21.40 K**, so the stated chain (rise ≤ re-equilibration ≪ 50) holds with 3.9× margin on the
measured rise and 2.3× on its own bound. **Nothing moves; the bar is that the bracket is transcribed
as `T + 50.0` literally**, because it sets the bisection's iterate sequence and any narrowing changes
every bit downstream.

> Also measured, and load-bearing for the port: the bisection uses **36–37** of its `range(200)`
> halvings at every one of the 70 marches. The counted loop's cap is never reached, so the `break`
> always fires — the Rust may not rely on that, but the oracle records the count as a NAMING key
> (slice E's classification: T9 is already gated at bit-equality, so the count only makes a
> shape error read "41 halvings instead of 37").

**PROBE 2 — `_equilibrate_hp`'s bracket is over-wide, and ASYMMETRIC IN THE UNUSED DIRECTION.**
`_irreversible_fast_expand:2028` brackets the constant-(H,p) root at `[Tt9 − 100, Tt9 + 800]`.
Measured: the root is **always above `Tt9`**, by 0.02 K at `Tt4` = 1300 rising monotonically to
**21.40 K** at 2300. So the 100 K below `Tt9` is never entered at all, and at most **2.7 %** of the
800 K above it is used. That is consistent with the docstring's physics (recombination reheats, so
`T* > Tt9`) — the measurement adds that the bracket is ~37× wider than the largest root offset.
**No change: same transcription bar as probe 1.** Recorded because a later reader tempted to tighten
this bracket would silently move every `V9_irrev_fast` in the slice.

**PROBE 3 — the 2nd-law floor's comment is CONFIRMED, both of its numbers.** `_DS_FLOOR = -5e-3`
(gas.py:1922) is justified by *"at the config minimum nstep (100) the worst trapezoid-truncation dS
is ~−1e-3 (frozen limit, 2nd-order → 0); this floor clears that yet catches a pathologically coarse
grid (nstep ≈ 10 ⇒ dS ≈ −0.1)"*. Measured:

| claim | measured |
|---|---|
| worst dS at `nstep = 100` is ~−1e-3 | **−5.366e-04**, margin 4.46e-03 over the floor |
| …and it is the FROZEN limit | **yes** — at `Da = 0.03`, the lowest rate probed; 8 of the 13 negative cells sit at `Tt4` = 1300 |
| `nstep ≈ 10` ⇒ dS ≈ −0.1 | **−3.2e-02 … −7.3e-02**, and the assert FIRES at all three rates |

The assert does not fire at `nstep = 20` (dS = −2.2e-04 at `Da` = 30, −2.6e-03 at 300 — negative but
inside the floor), so the config's own `nstep ≥ 100` guard carries **5×** margin over where the floor
actually bites. **This is the first probe in this lineage that CONFIRMS the source rather than
correcting it, and it is recorded as such.**

> **`dS` LEADS THE ORACLE.** It is a difference of two molar entropies that legitimately lands
> NEGATIVE in **13 of 70** sweep cells at the shipped `nstep`, i.e. a near-total cancellation whose
> sign is not even fixed. That makes it the most drift-sensitive quantity in the slice and therefore
> the arm to lead with — slice 5's lesson (a finite difference inherits its drift from the quantity
> differenced) and slice 4's (lead with the reader that bypasses the short-circuit), applied here.

**PROBE 4 — rung 26's "to the ULP" reduce HOLDS. The first exactness claim in this lineage to
survive.** `_freeze_out_expand:2216` says that with a CONSTANT `da_local_fn` it "reproduces
`_finite_rate_expand(Da)` **to the ULP**". Slices C, D and E each corrected a claim of this exact
shape (three for three), so this was pre-registered as the likely fourth. Measured over
`Tt4 × Da × nstep` = 5 × 4 × 2 = **40 cells**: **40/40 bit-exact**, worst |ΔV9| exactly **0.0**, and
the exit composition bit-identical species by species and in the same order. It survives `Da = 300`,
which the source's own gate (Da ∈ {0.5, 2, 10}, one design point) never reaches. **GATE: the Rust
must reproduce this as an equality between two RUST functions, not via the oracle** — the Python↔Rust
dump cannot see a loop-shape error transcribed identically into both copies, and this reduce can.
That is one of the two non-oracle gates in this slice.

**PROBE 5 — the composition-ORDER hazard filed against these exact two lines is NOT LIVE, and the
order is still load-bearing.** § 4.3's slice-A note, corrected by slice B, ends by naming
`gas.py:1963` and `gas.py:2255` — the two hand-built `comp1 = {sp: …}` dictionaries — as **"phase 4's
problem"**. It is now phase 4, and the question is answered by dumping rather than by reading:

* `sps = list(comp)` and `list(n_eq)` are the **same list in the same order**, at every step.
* `_equilibrium_composition` returns `[CO2, H2O, CO, H2, OH, O, H, O2, N2, Ar]` for **112 of 112**
  probed `(far, T, p)` combinations spanning `far ∈ [0.010, 0.045]`, `T ∈ [700, 2400] K`,
  `p ∈ [2e4, 2.5e6] Pa`. The key order is input-independent.
* Therefore `n_eq.get(sp, 0.0)` **never fills a zero** and nothing in `n_eq` is **ever dropped**. The
  silent-zero branch has no reachable instance from these entry points.

**The hazard is discharged — but the ORDER is not free**, because `sum(comp1.values())` and
`mix_h_abs_b` both accumulate in it and floating-point addition is not associative. **GATE: the
oracle dumps the ordered species list as data**, and the Rust slice is built from that dump rather
than from a list retyped by hand.

**THE PORT DECISIONS.**

1. **A NEW `march.rs`, and this is the module decision slice E parked.** Slice E's port decision 1
   put rung 17 in `nox.rs` and said explicitly that phase 4's marches "are where that module
   decision belongs — not pre-built for a phase that has not been scoped", so this is not read as
   settled either way. It is settled now, and **against** `nox.rs`: that file is already **4,349
   lines** and slices F+G would push it past 5,500. The dependency is strictly ONE-WAY — the marches
   consume `mix_entropy_molar` / `mix_mass_per_air` / `mix_h_abs_b` / `expand_nozzle` /
   `equilibrium_no_fraction`, all already `pub`, and nothing in rungs 7–24 consumes a march — so
   there is no circular dependency to buy, which was slice E's stated reason against splitting. Rung
   29's `shifting_turbine` is a `Gas` method, and an inherent `impl Gas` block is legal from any
   module of the defining crate (`nox.rs` already has two). **Rung 30 goes to `components.rs`
   regardless**, being a `Nozzle` branch.
2. **`choked_mfp` IS NOT PORTED IN THIS PHASE.** § 5.2 assigns it to rung 31 and phase 5. Checked
   against the tests rather than assumed: **no rung-30 test references it** (the only hit anywhere in
   `tests/` is a comment in `test_numeric_fingerprint.py`). Shipping it under a gate made only of
   rung-30 tests is the "untested code" case the plan warns about, so it waits for phase 5.
3. **TWO GATES ARE NOT THE ORACLE.** Probe 4's constant-rate reduce (rung 26 → rung 25) and rung
   28's structural reduce (`_frozen_no_trajectory` → `_coupled_no_march` reproduces
   `_no_freeze_out_expand`, slice G) are equalities between two RUST functions. A Python↔Rust dump is
   blind to a loop-shape error made identically in both copies; these are not. They run as named
   gates in the rung suites, not as a byproduct of the oracle.
4. **THREE BISECTION TOLERANCES IN ONE SLICE, and transcribing them uniformly is the defect.**
   `1e-11 * Tm` (rungs 25/26's energy bisection), `1e-10 * T` (`_equilibrate_hp`), `1e-13 * Tm`
   (rung 27's `_frozen_T`, and slice E's `expand_nozzle` already carries it). All three share slice
   E's named loop shape — counted `range(200)`, midpoint at the TOP, bracket updated, break on
   **this iteration's pre-update** midpoint, result recomputed from the final bracket AFTER the loop.
   Each is transcribed literally and separately.
5. **THE SLICE A–D VACUITY RETRO-AUDIT STAYS DEFERRED, deliberately.** § 4.10 (a) lists it as a
   *candidate*, not as parked work. Folding an audit of four shipped slices into a porting phase
   blurs two deliverables and would make phase 4's own gate ambiguous. It is not forgotten; it is
   not phase 4.

### The rungs where a tolerance is NOT a valid substitute

The finding is a **count**, and a count jumps discontinuously:

| rung | the claim | why a tolerance does not cover it |
|---|---|---|
| **9** | the EI bell PEAKS near φ≈0.95 | **MEASURED (§ 4.3):** the two interpreters disagree on the peak VALUE and agree on its LOCATION — so the argmax is the only key that answers "did the finding move?" |
| **10** | `T(β)` peaks AT the stoich crossing | **MEASURED (§ 4.4):** an interior argmax for a rich primary, index 0 for a lean one. A trajectory run backwards leaves every individual temperature plausible and breaks only this. |
| **12** | the EI-min is pinned **AT** `C_opt` | **MEASURED (§ 4.4), and the claim NARROWED by measuring it:** the argmin is the finding, and it moves off `C_opt` past `S/S_x ≈ 1.2`. A value tolerance would have reported "agrees" on every one of those points. |
| **13** | ⟨EI⟩(g) is HUMPED | **MEASURED (§ 4.5):** the peak is an interior argmax sitting one grid cell from the quadrature's own `a = 1` scheme switch, across which the curve is not locally monotone. The claim is the LOCATION; the neighbouring values differ by ~3 %. |
| **13** | the min is pinned AT `C_opt`, shifting as `(H/S)²` | **MEASURED (§ 4.5)** at four spacings: the finding is that two argmins stand in a fixed relation, which no per-value tolerance expresses. |
| **16** | which near-degenerate optimum is lowest | **THE INVERSE CASE, and it belongs here for that reason:** the rung DECLINES this location, so it must NOT be gated — a key on it fails for reasons that are not defects. The gates assert the sublinearity RATIO instead. |
| **14** | the dropped clamp FIRES on the cooling path | **MEASURED (§ 4.9):** a THRESHOLD, `max_a > 1`. It rides ~250× clear at φ_p=1.0 and ~0.016 at φ_p=1.5, so the two SIDES are what the rung says — a value tolerance on `max_a` expresses neither. |
| **17** | the fidelity ORDERING `a_mixed < a_bulk < a_pocket` | **MEASURED (§ 4.9):** three comparisons, not three values — and the one that BREAKS out of band is `a_bulk > 1`, at `J ≈ 2460`, while the ordering itself survives to J = 16 000. A tolerance loose enough to pass the moved magnitudes cannot see either. |
| **18** | mean-field ω has NO interior optimum | the finding is that an argmin sits at an END for three ω-shapes and INSIDE for the spatial one — four locations and a contrast, not four values. |
| **21** | the O-lift is SHAPE-PRESERVING | **MEASURED (§ 4.5):** the claim is that two argmins are EQUAL while every value between them moves. A value tolerance loose enough to pass the moved values cannot see the equal locations at all. |
| **83** | 1 of 5 ramps has no root | a 15th-digit shift can cost a fourth ramp its root |
| **84** | a minimum over a *marched* grid | by construction a reading that a tiny shift relocates to the next step |
| **81** | 0 of 1,364 floats moved | **bit-equality itself is the finding** |
| **78 / 79** | counts of exact zeros | ditto |

**This table is NOT complete.** It is drawn from the reader-only tail, because that is where
the counts are most obvious. The same objection covers any finding whose claim is *where* an
extremum sits rather than what a value is — rung 12's "min pinned **AT** `C_opt`", rung 22's
"`C_opt` **emerges as an OUTPUT**", rung 24's "`F(C)` U-shaped but `⟨EI⟩` monotone". **Phase 3
will extend this list**; treat it as a running register, not a fixed one.

Each entry needs individual adjudication *before* re-anchoring — decide whether the claim
survives and what the Rust test should assert. Budget ~10–15 rungs of this. It is a schedule
item, not a footnote.

---

## 5. Phases

Each phase ports **code and its tests together** and is gated against the oracle before the
next starts. The tree is green at every phase boundary; there is no big-bang cutover.

| # | scope | sessions | gate |
|---|---|---|---|
| **0** | ~~Cargo crate; the oracle bridge; the per-quantity tolerance policy~~ | **DONE** | ✅ 3232 values round-trip; policy DERIVED from the CPython↔PyPy gap, not invented |
| **1** | ~~`gas.rs` — `FlowState`, CPG closed form, TPG NASA integrals, reacting section, Fork B, equilibrium Newton (rungs 1–6)~~ | **DONE** | ✅ **two** gates: `gas_oracle.rs` (values, **3232/3232** bit-exact vs PyPy after phase 2's fix — § 4.1 shipped it at 3196, § 4.2 says why) and `gas_spine.rs` (**reduce-to-prior**, 6 tests — § 5.1) |
| **2** | ~~`components.rs` + `engine.rs` design point — shaft balance, `_score`; conservation checks as `assert!`~~ | **DONE** | ✅ **three** gates: `cycle_oracle.rs` (1481/1481 bit-exact vs PyPy, on 19+15 distinct solver roots — § 4.2), the 8 ported rung suites (39 tests, rungs 1–6, incl. rung 6's GATE 1), and `porting_rules.rs` |
| **3** | NOx & mixing, rungs 7–24. **RISK-BEARING — not bulk.** These are phase 1's largest *consumer*: every one rides the equilibrium solve and `Kp = exp(−ΔG°/RuT)`, and their findings are *shapes* (the bell's peak, the minimum pinned at `C_opt`, monotone-vs-turns-back-up) that a last-digit shift in an exponential can move. Deliberately placed straight after phase 1 as the **first real test of whether the transcendental arithmetic holds**. **DONE — slices A (7/8/9/19), B (10/11/12/20), C (13/15/16/18/21), D (22/23/24) and E (14/17) all shipped**, § 4.3–4.10; the slices are grouped in § 4.3 by DEPENDENCY, not by number | 4–6 | ✅ slice A: `nox_oracle.rs` (**1806/1806** bit-exact vs PyPy on 22+22 distinct solver roots) + 4 rung suites (43 tests) · ✅ slice B: `quench_oracle.rs` (**2507/2507**, on 165 distinct trajectory roots) + 4 rung suites (39 tests), one location key NARROWING a shipped claim · ✅ slice C: `pdf_oracle.rs` (**2448/2448**, both quadrature branches asserted exercised) + 5 rung suites (59 tests); the source's own mean-preservation guard found to have an `n_quad` FLOOR, and the port gates the REJECTION as well as the acceptance (§ 4.5) · ✅ slice D: `spatial_oracle.rs` (**462/462**, incl. 28 DISCRETE keys) + 3 rung suites (43 tests); TWO source claims of exactness CORRECTED — rung 24 applies an operation inside an accumulation and removes it outside, twice (§ 4.8) | · ✅ slice E: `nozzle_oracle.rs` (**513/513**, incl. 24 DISCRETE keys) + 2 rung suites (24 tests) + 3 gates `rung20.rs` had deferred; a THIRD claim of exactness corrected (the frozen reduce is algebraic only, and its floor is the entropy ROUTE, not the bisection's stopping rule) and rung 17's firing band edge LOCATED — past it the bulk margin goes dormant while the per-pocket one RISES (§ 4.10) |
| **4** | Nozzle & turbine marches, rungs 25–30 — own convergence behaviour, hence separate. ~~**AUTHORISED 2026-08-12; three DEPENDENCY slices**~~ **DONE** | 2–3 | ✅ slice F: `march_oracle.rs` (**912/912** bit-exact vs PyPy, on 49 distinct march exit roots) + 2 rung suites (32 tests) in a new `march.rs`; the FOURTH "exactly"-class claim and the FIRST to survive — because it compares a COPY, not a rederivation (§ 4.12) · ✅ slice G: `no_march_oracle.rs` (**776/776**, and only 8.0 % CPython-identical — the sharpest dump in the port) + 2 rung suites (28 tests); slice F's discriminator made TWO pre-registered predictions and both HELD (§ 4.14) · ✅ slice H: `tt_oracle.rs` (**270/270**) + 2 rung suites (14 tests); RATIO ≠ ENERGY measured ANTI-correlated, and the one slice not pre-registered — all three of its guessed census bars were wrong (§ 4.15) |
| **5** | ~~Steady matchers — rungs 31–33, 38–39, **41**, 42, 53–56, 61~~ **DONE 2026-08-17, seven slices (I·J·K·L·M·N·O).** ~~**Contains the diamond** (§ 6)~~ **PRE-FLIGHT DONE (§ 5.3); AUTHORISED 2026-08-13.** The diamond is discharged; the phase's structural content is the **five-name virtual set** (`_solve_turbine` — claimed by PHASE 6 — `match`, `_hp_eta_loop`, `_lp_eta_loop`, `at_setting`) and `_INC_MAX`'s live shadow | 4–6 | ✅ slice I (rungs 31/33): `offdesign_oracle.rs` (**3951/3951** bit-exact vs PyPy, incl. **961 discrete** keys) + 2 rung suites (17 tests) in a new `matcher.rs`; the crate's FIRST fallible paths, its FIRST virtual hook, and the two rungs re-gated as counts over BIT PATTERNS (§ 5.5) · ✅ slice J (rung 32): `map_oracle.rs` (**7 252/7 252**) + `rung32.rs`; the oracle found BLIND to a mis-spelled square, so the rule is gated directly (§ 5.6) · ✅ slice K (rungs 38/39): `two_spool_oracle.rs` (**11 812/11 812**, and only 46.3 % CPython-identical) + 2 rung suites (19 tests) in a new `two_spool.rs`; all six predictions held, and the CPython arm REFUTED an assertion inherited from slice I — the pass-count instability needs a SOLVER-derived property, not the equilibrium gas (§ 5.7) · ✅ slice L (rungs 41/42): `slice_l_oracle.rs` (**25 458 keys**) + 2 rung suites (12 + 12 tests) in a new `bleed.rs`, plus the crate's first FALLIBLE TWINS; nine predictions all settled, and a claim the SHIPPED SOURCE carried found wrong (§ 5.8) · ✅ slice M (rungs 53/54): `slice_m_oracle.rs` + 2 rung suites (24 + 25 tests) in a new `stator.rs`, all six steps shipped; its probe **OVERTURNED slice J's `solve_n` zero-firing verdict**, and a bar asserted in a shipped doc comment was refuted by a third of the dump (§ 5.9) · ✅ **PHASE 5 COMPLETE** — the last two slices were N (55/56) and O (61), sized in § 5.9; **N is PRE-REGISTERED (§ 5.10), 10 predictions**, and its probes **REFUTE § 5.9 (c) twice** — reading a method's body cannot see its state's CARRIER — **steps 1–4 SHIPPED** (step 1: P2 held at 535 names, `diff` empty, and the ±1 that would not reconcile was slice I's last `#[ignore]` surviving a rule slice M had already retired · step 2: `stage.rs` + `slice_n_smoke.rs`, **1 337 keys bit-exact first run** over seven enumerated cells, and it **CORRECTS § 5.10 (iii)** — `_P_FLOOR`'s deadness is a DERIVED threshold `e > 1.001` in the two floor constants, not a property of the sweep · step 3: `StageStackCore` + `R55`/`R55_TWO`, and the carrier lesson RECURRED — a FOURTH gated-code edit, at step 3, in a file step 1 never opened, because the plan asked what carrier `at_setting` needs and never what carrier the EFFICIENCY-LOOP hook needs; plus an `_INC_MAX` shadow § 5.3's pre-flight had called correctly and the porting slice mis-spelled · step 4: `slice_n_oracle.rs`, **72 520 keys bit-exact first run** + a 5 649-key equilibrium arm and a 41 560-key CPython one, and § 5.10's own censuses turned out to be measured on TWO grids — (i)/(iv) reproduced to the firing, (iii)/(vi) came from a 240-cell probe sweep, so the census is EMITTED and compared rather than restated; the CPython arm's *discrete → bits* tier had to SPLIT, because all 520 interpreter flips are argmin indices at the design throttle) |
| **6** | Transients — rungs 34–37, 40, 43–52 (the fuel-side limiter family). **AUTHORISED 2026-08-17; PRE-FLIGHT DONE (§ 5.12).** The phase's structural content is a **six-name virtual set** — `integrate_fuel`, `_close`, `_close_fuel`, `_surge_fuel`, `_instant_tail`, `_powers` — **every one of which crosses into phase 7**, so there is no phase-6-internal hook and the `Hooks` table appears at slice R, not P. Six slices, ordered by `_degenerate`: **P** (34/35/36 `SpoolTransient`) · **Q** (37) · **R** (40/44) · **S** (43/45, `integrate_fuel` ENTIRE) · **T** (46/47/48 gates) · **U** (49/50/51/52 gates). The `4257–4506` object block spans two phases — `IncidenceLimiter` is rung **60** | 4–6 (**light** — 156 tests over 15 files, and phase 3 took five slices for 204) | per-rung tests pass |
| **7** | **The ladder, rungs 57–60 and 62–84** — the `Hooks` table from § 2, one module per rung. (**61 is PHASE 5's**, not this phase's — it is the steady `StatorBleedMatcher`, and it was double-listed here until the slice-K audit) | 5–8 | 27/27 reduce-to-prior bit-exact |
| **8** | `main.py` replacement; adjudicate the fragile rungs; re-anchor the fingerprint; **delete the Python** | 2–3 | full suite green on Rust alone |

**THE TABLE'S OWN COVERAGE WAS NEVER AUDITED UNTIL SLICE K, AND IT HAD BOTH FAILURE MODES.**
Enumerating rungs 1–84 across the eight rows found **rung 41 in NO phase** (5 stopped at 39 and
jumped to 42; 6 listed 34–37, 40, 43–52) and **rung 61 in TWO** (named by 5, swept up by 7's
"57–84"). Neither is cosmetic: rung 41's code is *inside* `TwoSpoolMapMatcher`, the class slice K
ports, so an unassigned rung would have been ported by accident or dropped by accident — and the
three `self.match` call sites the § 5.3 census used to justify a virtual hook are **all three
rung-41 methods** (`surge_margin` 3174, `running_line_map` 3226, `flow_coefficient_turn` 3242),
which is what decides slice K's boundary (§ 5.7). Both rows are corrected above. **The lesson is
the port's own restated at the plan level: a scope list is a claim about a SET, and it is only as
good as an enumeration over that set — nobody had ever counted.**

**PHASE 3's estimate held: 4–6 sessions were budgeted and five slices shipped.** Its
risk-bearing label was earned in a way the estimate did not anticipate — the arithmetic never
failed (five oracles, 11,985 values, 100 % bit-exact against PyPy at every one), and every
finding came instead from sweeping past the source's own gates. Six slices, six such findings.

**Total: 26–40 focused sessions**, as estimated *before* phase 1 ran. The risk is **not**
evenly spread and is not only at the ends: it concentrates in phase 1's transcendental
arithmetic, in **phase 3 which consumes it**, and in the final count-based adjudication.
Phases 4–7 are grinding but low-risk.

**Reconciling that against what phase 1 actually measured (§ 4.1):** phase 3 was rated
risk-bearing because rungs 7–24 ride the equilibrium solve and their findings are extremum
*locations*. That solve has now come back **bit-exact**, which removes the *arithmetic* half of
phase 3's risk but **not** the *shape* half — an extremum's location can still move on a
solver's stopping rule, and phase 3's own mixing PDFs and bell integrals bring new solvers.
So phase 3 keeps its 4–6 sessions and its risk-bearing label, with the reason narrowed: **watch
the stopping rules, not the polynomials.** No other estimate changes.

### 5.1 What phase 1's spine gate does and does not cover

`gas_spine.rs` ports the reduce-to-prior contract that `gas_oracle.rs` cannot see — a port can
agree at every probe point and still be structurally wrong. Six tests, all green:

- the calorically-perfect section keeps `h = cp T` and `pr = T^(1/g)` **bit-for-bit**, so
  reduce-to-ideal still reproduces rung 1's table to the digit;
- **the trap, measured**: the closed-form and integral exponents differ by 4.98e-4, moving `pr`
  by 1.35e-2 while leaving `h` at 3.1e-16 — so the branch split is load-bearing for pressure
  and harmless for enthalpy, exactly as rung 3's docstring claims;
- Fork B's derived LHV round-trips the assumed 42.8 MJ/kg with gap **exactly 0**;
- a6/a7 self-check (GATE 3) — evaluating back at 298.15 K returns ΔHf and S298;
- pressure suppresses dissociation (0.101 → 0.056 → 0.039 at 1/5/13 atm);
- **rung 6 reduces to rung 4 as it cools**: worst composition gap 4.0e-2 → 7.5e-3 → 6.4e-4 →
  1.1e-5 → **4.8e-8** over 2400→900 K, strictly shrinking. The *shrinking* is the assertion, not
  the smallness — a constant offset would betray a scale-A datum leaking into a scale-B
  balance, and "the gap is small" passes happily on a constant 1 %.

**NOT covered, deliberately:** rung 6's GATE 1 — the cold-`Tt4` *cycle* reduce (`fE == fB` to
1e-6) — needs `build_turbojet`, so it lands in **phase 2** with the components. Phase 1's gate
is the gas layer's spine, not the cycle's.

### 5.2 What phase 2 shipped

`rust/src/components.rs` (the five components, every conservation assert, `ram_recovery`),
`rust/src/engine.rs` (freestream, the shaft balance and its closure check, `score`,
`build_turbojet`), and the `Gas`/`GasSpec` pair in `gas.rs` that phase 1 had left out — the
dual-section object the components actually hold, its four factories, and `unified()`.

Three notes on shape, each a place Rust says something the Python could only comment:

- **`GasSpec` + struct-update syntax replaces `dataclasses.replace`.** § 2 records eighteen
  hand-written field-copy sites in the Python and a rung-80 docstring calling one omission
  *"THE EIGHTEENTH INSTANCE of the trap"*. `..self.spec` makes forgetting a field inexpressible.
- **`Losses::default()` is the reduce-to-ideal gate**, the way Python's ideal keyword defaults
  are: the no-argument call IS the rung-1 engine.
- **`Component` is an enum, not a trait object.** Python dispatches with `isinstance` because
  Turbine and Nozzle deliberately diverge from `apply(state, gas)`; the enum turns that
  divergence into a `match` the compiler checks, so a sixth component cannot be added without
  deciding how the engine drives it.

**Deliberately NOT ported yet:** rung 30's choked convergent nozzle (`_sonic_throat`) and rung
31's `choked_mfp`, though both live in `components.py`. They belong to phases 4 and 5, where
their own gates run; shipping them into a phase gated only by the rungs 1–6 suites would ship
untested code. The Python grew that branch AT rung 30 too.

The 39 ported tests also record two ENVELOPE limits of the Python being ported, neither
introduced by the port: the design-point scorer's efficiency cascade is `0/0` at `M0 = 0`, and
production's Fork-B closure assert fires at `f ≈ 0.052` (~77 % of stoichiometric). Both are
documented at their sweep entries rather than worked around.

### 5.3 PHASE 5 PRE-FLIGHT — the inheritance census, MEASURED

**Authorised on its own, ahead of the build**, because § 6 named the rung-61 diamond as the
phase's structural risk and prescribed the action *"write down what Python's resolution order
actually produces, flatten by hand, gate with rung 61's existing reduce test."* Two probes, one
interpreter each (`M:\claud_projects\temp\rung61-diamond\probe_phase5{,b}.py`).

**The scope was widened before the first line was written, and that was the load-bearing
decision.** `engine.rs` has no matcher classes at all, so **phase 5 is the first phase in the
port to meet Python inheritance** — the diamond is one edge out of nine, and it turns out to be
the *easy* one. Clearing it alone would have been a false clearance. The census below covers
every phase-5 edge.

**1. The diamond is ONE colliding name, and it is `__init__`.** `TwoSpoolBleedMatcher` defines
5 names, `VariableStatorMatcher` 22; the intersection is `{__init__}`. **Measured over class
DATA as well as callables** — the first probe filtered `vars()` to callables, and a constant
resolves by the same MRO, so a same-named constant with different values would have made the
order load-bearing in a second place. It does not: rung 42 defines **zero** constants. The MRO
is `StatorBleedMatcher → TwoSpoolBleedMatcher → VariableStatorMatcher → TwoSpoolMapMatcher →
TwoSpoolMatcher → object`, and **the flattened linear chain is that list**.

**2. Why the flattening is safe is a CONJUNCTION of two independent facts**, and that — not
"we flattened it and the numbers agreed" — is the finding. (a) `VariableStatorMatcher`
overrides **nothing** on the plant: none of its 16 methods shadows a `TwoSpoolMapMatcher` name,
so it is a pure extension that acts only through constructor state. (b) The one place order
*would* have bitten was **opted out of by hand** — rung 61 calls `VariableStatorMatcher.__init__`
explicitly (`engine.py:8480`) and contains no bare `super()` at all. § 6's fear that *"neither a
`Prev` chain nor a linear table handles multiple inheritance"* was written before anyone looked;
the MRO is doing almost no work here.

**3. A `super()` target DOES move between cells — and the moving one is never traversed.**
From `TwoSpoolBleedMatcher`, `super().__init__` reaches `TwoSpoolMapMatcher` when rung 42 is the
concrete class and `VariableStatorMatcher` when rung 61 is: **different function objects**.
`super().match` reaches `TwoSpoolMapMatcher` **under both** — the same object — because rung 53
has no `match` to intercept it. Since rung 61 bypasses the constructor chain, **no live call
site traverses a moving edge.** The consequence is therefore *weaker* than feared, and it is a
correction to the expectation that rung 42 would have to take its parent from the instantiating
cell: it does not. Rung 42's `match` may name its parent directly. The single requirement is
that **rung 61 must not inherit rung 42's constructor**, which is one line of Rust, not an
architecture.

**4. Virtual dispatch across phase 5 is SIX overrides on FIVE names, and the set is NOT closed —
one name crosses into phase 6.** The pattern a plain Rust `struct` + `impl` cannot express is a
child overriding a method an *ancestor calls on `self`*. Every other override is a leaf and needs
nothing. Swept with the ancestors restricted to the phase-5 set and the **descendants opened to
all 58 classes in `engine.py`** — the first pass restricted both sides and therefore answered only
*"does phase 5 need hooks for phase 5?"*:

| name | overridden by | called on `self` by | sites |
|---|---|---|---|
| **`_solve_turbine`** | **`SpoolTransient` — rung 34, PHASE 6** | `OffDesignMatcher` | 1 |
| `match` | `TwoSpoolBleedMatcher` | `TwoSpoolMapMatcher` | 3 (`engine.py:3193, 3233, 3261`) |
| `_hp_eta_loop` | `StageStackMatcher` | `TwoSpoolMapMatcher` | 1 (`2992`) |
| `_lp_eta_loop` | `StageStackMatcher` | `TwoSpoolMapMatcher` | 1 (`2994`) |
| `at_setting` | `StageStackMatcher`, `StatorBleedMatcher` | `VariableStatorMatcher` | **13** (`6169`–`6549`) |

**`_solve_turbine` is the one that decides the architecture for slice one.** It is rung 31's
method, it is called on `self` inside rung 31's own body, and the class that overrides it is
**rung 34's `SpoolTransient` — phase 6**. So `OffDesignMatcher`'s ported body must already be
hookable at that name on the day it ships, or phase 6 refactors phase-5 code that is already
gated. This is precisely the failure the phase gating exists to prevent, and a census restricted
to phase 5 could not see it. **32 further names are called on `self` and overridden by nothing at
all** — those are plain functions, no hook, no indirection.

**`at_setting` is the heavy one** by an order of magnitude, and is a *sibling constructor*
returning the concrete type — the `Self`-returning shape a const-`Hooks` table handles free and a
hardcoded `Prev` chain does not.

**4b. SIBLING DISPATCH is live, and a `self.X` scan is structurally blind to it.** `at_setting`
/`at_point`/`at_bleed` hand back a concrete-typed sibling and the code then calls methods on *that*
receiver. Six distinct names are invoked that way; five resolve to a single definition and are
safe, but **`match` is called on a sibling at 4 sites** (`8612, 8614, 8629, 8669`) and `match` is
overridden. It adds no new name to the set above — it adds the requirement that the sibling
constructor return **the instantiating cell**, not a fixed type, which is exactly why rung 61
overrides `at_setting` at all.

**5. The hazard the override census could not see, checked separately and ABSENT.** An ancestor
calling `self.X()` where `X` is supplied only by a descendant (the abstract-hook / template-method
shape) is invisible to a both-define-it diff. Swept over all thirteen phase-5 classes:
**none** — every name read off `self` resolves on the defining class's own MRO.

**6. ONE LIVE CONSTANT SHADOW, and it changes a solver's iteration cap.**
`StageStackMatcher._INC_MAX = 200` shadows `VariableStatorMatcher._INC_MAX = 80`, and rung 53's
`incidence_schedule` (`6309`) and `_schedule_root` (`6506`) read `self._INC_MAX` — **neither is
overridden by rung 55**, so on a rung-55 object rung 53's own inherited loops run to 200. In Rust
that cap must be a per-cell parameter, never a literal in the ported body. `_INC_TOL` is
redeclared at the **identical** value (`1e-12`) — inert, but preserve it so a reader does not
infer a difference that is not there. Swept downstream against all 58 classes as well: **those
two are the only constant shadows that exist**, and nothing in the transient ladder shadows a
phase-5 constant.

**7. THE KILL TEST — the source's claim at `engine.py:8477` CONFIRMED, and SHARPENED.** The
comment says a co-operative `super()` chain *"would silently leave the stators at the design
setting — a wrong number with no exception."* Built and run at `(v_lp, v_hp, b) = (0.20, 0.05,
0.10)`: **no exception**, `map_lp.vsv == 0.0`, `map_hp.vsv == 0.0`, and `vsv_lp` present as an
attribute holding `0.0` — every defence the class has, passed. The sharpening is *where* the
damage lands:

| quantity | co-operative | explicit | relative |
|---|---|---|---|
| `phi_lp` | 1.077620292521086 | 0.9406801067799551 | **1.46e-1** |
| `n_lp` | 0.9760038748343771 | 1.11715064924026 | **1.26e-1** |
| `phi_hp` | 1.0080636642767797 | 0.974412076584007 | 3.45e-2 |
| `n_hp` | 1.0190804141746266 | 1.0535518977105405 | 3.27e-2 |
| `thrust` | 658.9560907209022 | 658.3093942088998 | 9.82e-4 |
| `pi_lpc` | 2.7239332275073793 | 2.7235207024697132 | **1.52e-4** |

The error is **concentrated in exactly the two quantities rung 61 is about** — the margin
coordinate `φ` and the shaft speed `N`, 13–15 % — while thrust moves 0.1 % and the pressure
ratio 0.015 %. "Plausible numbers" is literally right: every headline cycle number a casual
check would look at is intact. **It is deliberately NOT counted in the port's "exactly"-claim
ledger** (slices D, E and F): those were *exactness* claims — `== 0`, bit-for-bit — and this is a
*hazard* claim, an assertion about what would go wrong on a road not taken. Different category,
different test, its own line.

**What was NOT built, and why.** A source-level re-parenting harness (recompile rung 42's and
rung 61's bodies under a linear base, diff the outputs against the diamond) was designed and
then **dropped before it was written**. It compares *the same source text*, so bit-for-bit
agreement is near-guaranteed and therefore near-zero evidence — slice F's own lesson, that an
exactness claim survives a copied instruction sequence and dies on a rederivation, applies to
the port's own scaffolding too. It also fights the code: `at_point` hard-constructs
`StatorBleedMatcher` **by name** (`8504`), so the flat class's siblings would be real diamond
objects and `test_trap_at_setting_carries_the_bleed`'s `isinstance` assert would pass vacuously
or fail spuriously. Walking the MRO and walking the flattened order agree *by construction* —
that arm is void, and the questions that are not void are 3, 4, 5 and 7 above.

**Verdict: § 6's diamond is DISCHARGED as a risk, and phase 5's real structural requirement is
the FIVE-name virtual set — one of which is claimed by phase 6 — not the two-parent join. The
census was widened twice, and both widenings paid: the first turned "the diamond" into "nine
edges", the second turned "closed inside phase 5" into `_solve_turbine`.**

### 5.4 SLICE I (rungs 31 + 33, `OffDesignMatcher`) — PRE-REGISTERED, five probes MEASURED first

**The slice.** Rungs 31 and 33 are **one Python class** — rung 33 is a second `match` branch on
rung 31's object, dispatched when the choked solve leaves the nozzle subsonic — so they are one
slice by construction, not by grouping. Rung 32 (`ComponentMap` + `MapMatcher`) is slice J.
Ported alongside: `choked_mfp`, deliberately deferred out of phase 2 (§ 5.2) and out of phase 4
(§ 4.15) as *"phase 5's, where its own gates run."* Its gates now run.

**Probed BEFORE registering** (the standing lesson), and the first probe's headline was WRONG:

**(a) THE CAUGHT-EXCEPTION WALL IS LIVE — after a first sweep said it was dead.**
`_match_subsonic` marches both brackets inward while catching `AssertionError` from deep inside
the component stack. Swept at one flight Mach and three throttles: **zero raises**, which would
have meant the Rust needs no fallible path at all. Swept at **3 gases × 6 flight Machs × 7
throttles = 126 bracket pairs**: **930 raises on the low march and 616 on the high march.** The
arm carries the majority of the low-throttle envelope. *A dead-guard finding is only as wide as
the sweep behind it* — the same lesson as § 5.3's second widening, twice in one session.
**Consequence:** `subsonic_operating` must be fallible in Rust; a panic cannot be marched past.

**(b) THE JOINT `(f, pt4)` FIXED POINT DOES NOT CONVERGE ON THE PRODUCTION GAS.** Its docstring
says seeding from the design point *"converges in a few passes."* Measured, as `_solve_turbine`
calls per `match`:

| gas | Tt4 = 1500 | 1100 | 900 |
|---|---|---|---|
| CPG | 1 | 7 | 7 |
| thermally-perfect | 2 | 7 | 7 |
| **reacting-equilibrium** | **200** | **200** | 7 |

200 is `_MAX`. The loop **exhausts its cap and falls out without ever setting `done`, and there
is no assert** — unlike `_solve_f` three methods above it, which raises when it fails to
converge. So the production gas silently returns the 200th iterate at the two hottest throttles.
The numbers are right (`pi_c` reduces to 10.0 to 1e-8 at design), so this is a **convergence-test
failure, not a wrong answer** — but the returned value is *the last iterate of a fixed count*,
which makes bit-exact reproduction of all 200 passes a hard requirement rather than a nicety.

**(c) THE TURBINE BISECTION IS EXACTLY 47 EVALUATIONS, ALWAYS.** `_solve_turbine` bisects `pi_t`
on `(0.02, 0.999)` to `hi - lo <= _TOL`, and `_TOL = 1e-13` is **absolute** there while the same
constant is used **relatively** in the `f` fixed point — the docstring calls it *"fixed-point /
bisection relative tolerance"*, which is half right. 2 bracket evaluations + 44 halvings
(`ceil(log2(0.979/1e-13)) = 44`) + 1 final = **47**, and it measured 47 at every call with no
spread.

**(d) THE BRANCH MAP, so the gate cannot silently exercise one path.** Choked down to Tt4 ≈ 650,
subsonic below: CPG flips between 600 and 550, thermally-perfect and reacting between 650 and
600. The CPG unchoke boundary bisects cleanly to **Tt4 = 592.036674415 K**. On the two variable-cp
gases the boundary is **NOT cleanly locatable**: inside a narrow band the subsonic re-solve lands
back above ambient and trips its own `p9 = p0` assert, so dispatch and re-solve disagree there.
Recorded as a scope edge of the Python being ported; not introduced by the port, and not repaired
by it.

**(e) ENVELOPE LIMITS THAT ARE NOT RUNG 31/33's, kept out of the oracle.** `M0 = 0` trips the
freestream's own ram assert on a variable-cp gas (phase 2 already recorded the design-point half
of this). `M0 = 2.0` at low throttle trips **the efficiency cascade in `_score`** at Tt4 = 650 and
**the equilibrium Newton's 200-step cap** at Tt4 = 550 — neither is a matching failure, and both
are upstream of this slice.

**PRE-REGISTERED PREDICTIONS** (falsifiable, recorded before the Rust exists):

1. **P1 — the counts port exactly.** Rust reproduces 47 turbine-residual evaluations per solve and
   the 1/7/7 · 2/7/7 · 200/200/7 joint-loop table, cell for cell. A count that differs means the
   arithmetic diverged even where the value gate still passes.
2. **P2 — the raise counts port exactly.** The fallible Rust path raises on the same 930 low /
   616 high bracket samples. If Rust's count differs, the wall sits somewhere else and (a)'s
   correction is itself wrong.
3. ~~**P3 — the 200-cap exhaustion is a LAST-BITS LIMIT CYCLE, not slow convergence.**~~
   **REFUTED IN ITS DICHOTOMY, and the real answer is sharper — see (g).**
4. **P4 — `Tt ** 0.5` is a `pow`, not a `sqrt`.** Both `choked_mfp` and the `tau_t ** 0.5` in the
   turbine residual must use the port's `powp`. This is the trap that hid for a whole phase
   (§ 4.15); registering it means a miss shows up as an oracle failure, not as a tolerance.

**(f) THE FALLIBLE-PATH DESIGN, decided by measurement rather than by taste.** — **SUPERSEDED
BY (i): its conclusion is right for the route it sampled and WRONG for the one it did not.**
Kept verbatim below, because what it got wrong is the finding. The bracket march
walks past **three** assert sites, not one: the equilibrium Newton's 200-step cap (`gas.py:651`),
the nozzle's `p9 <= pt9` precondition (`components.py:683`), and the matcher's own `f` fixed point
(`engine.py:500`). Two are cheap to make fallible in Rust; the third sits in phase-1 **gated**
code and would force a signature change through it. So the question measured first was whether it
needs to be fallible at all — dumped every `(f, Tt4, pt4)` reaching `freeze_equilibrium` over
the march, 8,289 calls:

| axis | raising calls (129) | succeeding calls (8,160) | separates? |
|---|---|---|---|
| `f` | 0.0271792 … 0.0271792 | 0.000102 … 0.0271792 | no — ranges overlap |
| `pt4` | 4.96e5 … 1.91e6 | 5.03e4 … 1.91e6 | no — ranges overlap |
| **`Tt4`** | **400 only** | **500, 600, 900, 1500** | **YES, cleanly — no value on both sides** |

**Every one of the 129 raises is at `Tt4 = 400 K`**, and at that throttle the Newton fails for
*every* `pi_t`, so the whole low bracket is unevaluable and the outer sub-idle assert fires
anyway. `Tt4 = 400` is below the modelled envelope, not an interior condition. **Decision: make
the two cheap sites fallible** — `Nozzle::try_apply` returning `Result` with `apply` delegating,
and a `Result` on the matcher's own `solve_f` — **and leave `gas.rs` untouched.** The documented
consequence, stated rather than hidden: below the envelope the two implementations **both abort,
with different messages** (Python raises the outer "does not bracket … SUB-IDLE" assert, Rust
panics inside the equilibrium Newton). Neither returns a number, so no result diverges. **This is
NOT a fitted `Tt4 < 500` screen** — the Rust does not test the throttle at all; it simply lets an
out-of-envelope condition abort, which is what Python does one frame later.

**P2 is re-scoped accordingly:** the raise counts to reproduce are the **in-envelope** ones (the
nozzle and burner sites), where the equilibrium site contributes zero. A Rust rejection set that
differs even by one trial moves the bracket and therefore the root, so this is a value gate, not
a bookkeeping one.

**(g) WHAT THE 200-CAP ACTUALLY IS — P3's dichotomy was FALSE, both ways.** The loop's `done`
needs `|Δf| <= 1e-13·f` **AND** `|Δpt4| <= 1e-13·pt4`, and the two hot throttles fail it for
**different reasons**:

- **Tt4 = 1500:** `f` is **exactly constant** across all 200 passes — one distinct value, `Δf = 0`.
  So the `f` half of the test passes trivially and it is the **`pt4` half** that never settles.
- **Tt4 = 1100:** `f` falls into a **two-value cycle** between adjacent representable numbers,
  `|Δf| = 2e-13` absolute on `f ≈ 0.0272` — about `7e-12` relative, roughly **70× above** the
  `1e-13` bar it is tested against. It cannot ever pass.

Neither is "slow convergence" and neither is a plain limit cycle: **the stopping rule is
unmeetable at this tolerance**, by a different mechanism at each throttle. The bearing on the
port is that the returned value is the 200th iterate of a fixed count, so all 200 passes must
reproduce bit-for-bit — and the bearing on the *project* is that a later rung cannot treat this
loop's output as converged to its own stated tolerance.

**(h) A SECOND PHASE-6 INBOX ITEM, recorded now while it is cheap.** § 5.3 left phase 6 owed
`_solve_turbine`. Add this: `TwoSpoolTransient` drives `self.match(...)` inside loops at **five
sites**, so a matcher that silently runs 200 non-converging passes is a per-timestep cost *and* a
"the state advances on an unconverged iterate" question. Measured here for the **single-spool**
joint loop only; whether the two-spool matcher shares the behaviour is slice K's to establish,
not assumed from this one.

**(i) (f) IS CORRECTED, AND THE REASON IS THAT `except AssertionError` IS BARE.** (f) reasoned
about three named assert *sites*. But `_match_subsonic`'s `except AssertionError` names nothing —
it catches **every** assert reachable from `resid`, so the question is not "which three did we
list" but "which are reachable". Re-swept with the caught exception's **message** recorded
(3 gases × 6 flight Machs × 7 throttles; `M:\claud_projects\temp\rust-slice-i\probe_raises*.py`),
596 raises fell into exactly **three families** — and the split by *call route* is what refutes (f):

| family | raises | route into it | fires on cells that BRACKET? |
|---|---|---|---|
| the equilibrium Newton, via `Gas.freeze_equilibrium` | 40 | **(f) sampled this one** | no — `Tt4 = 400` only, and no `pi_t` is evaluable there |
| the equilibrium Newton, via `Burner._solve_equilibrium` → `_equilibrium_composition` | **26** | **(f) never sampled it** | **YES — 5 cells at `Tt4` = 500 / 600 / 650** |
| the matcher's own `f` fixed point (`engine.py:500`) | 172 | — | yes |
| the nozzle's asserts (`components.py`) | 132 | — | yes |

**The second row is the correction.** `_solve_f`'s equilibrium branch reaches the composition
solve **directly** and never touches `freeze_equilibrium`, so (f)'s 8,289-call dump was blind to
it by construction — the same shape as § 4.16's *"an oracle cannot see a missing gate"*, one level
down. Those 26 raises are **in-envelope and load-bearing**: they move the low bracket from 0.15 to
0.19 / 0.35 / 0.17 / 0.33 / 0.23 on five cells that then return a matched point. A Rust that panics
there returns nothing where Python returns a number. **So `gas.rs` DOES need a fallible path** —
but as an **additive `try_` twin** whose panicking original delegates to it, which is not the
signature change through gated code that (f) refused: **no phase-1 gate sees any change at all.**

**THE RULE, stated once and applied per site:** *an assert becomes fallible iff it is reachable
from inside `resid` during the bracket march* — which is exactly Python's `try` scope. Its two
edges were both measured rather than assumed:

- **`_sonic_throat`'s two bracket asserts: 0 fires in 111,775 calls.** Reachable, never taken.
  Left as panics — a fallible path with no reachable failure is a gate that measures nothing
  (§ 4.9's rule), and the consequence is stated instead: if one ever fired, Rust would abort where
  Python marched on.
- **`_solve`'s `inverse: root not bracketed`: 6 fires in 225,410 calls — and every one aborts its
  CELL rather than being marched past.** Established as a *superset* argument, not an absence:
  the message sweep runs the subsonic march on all 126 cells unconditionally, where `match` runs
  it only after unchoke, so its march coverage strictly contains the full sweep's — and it saw
  three families, not four. All six sit at `Tt4 = 400` on the thermally-perfect gas. Left a panic;
  making it fallible would infect every property-interface call site in the crate for a guard that
  provably cannot be marched past.

**THE ENVELOPE MAP, so the oracle excludes cells with a reason rather than silently.** Over the
126 cells: **88 match (74 choked, 14 subsonic), 38 abort** — `Tt4 = 400` is outside the envelope
on every gas, and the aborts above it are the `_score` efficiency cascade (11) and SUB-IDLE (11),
both of which (e) had already placed upstream of this slice.

**P2 IS RE-SCOPED A SECOND TIME, and the reason is worth more than the number.** (a)'s
"930 low / 616 high" is not reproducible, because **the sweep grid behind it was never written
down** — this sweep, at the same 3 × 6 × 7 shape, measures 550 / 46. So P2 becomes: *the Rust
reproduces the per-cell rejection counts and bracket endpoints of the grid recorded in the
oracle*, and the grid is now in the dump where it can be read. A count without its grid is not a
measurement.

### 5.5 What phase 5 SLICE I MEASURED — 100 % again, and every pre-registered prediction held

Slice I ships `rust/src/matcher.rs` (`OffDesignMatcher`, both branches, `MatcherHooks`), the
oracle `rust/oracle/dump_offdesign.py`, and three gates — `offdesign_oracle.rs` plus the two
ported rung suites `rung31.rs` (8 tests) and `rung33.rs` (9). It also ships the crate's first
fallible paths (§ (i)) and `nozzle_convergent` on `Losses`. **The Rust suite is now 415 tests.**

| | bit-identical vs PyPy | vs CPython |
|---|---|---|
| off-design oracle (3,951 values) | **3,951 / 3,951 (100 %)** | 2,387 / 3,951 (**60.4 %**) |

| class | keys | CPython↔PyPy | what it is |
|---|---|---|---|
| `discrete` | 873 | **0.00e0** | the branch labels, the abort codes, the per-cell march-rejection tallies, the 47, the root counts |
| `step` | 238 | **0.00e0** | the marched bracket endpoints, `0.15 + k·0.02` |
| `value` | 2,514 | ≤2.6e-10 | the matched grid — `pi_c`, `tau_t`, `pi_t`, the stations, thrust, the efficiencies |
| `residual` (ABS) | 238 | ≤1.9e-10 | the (★★) mass-continuity residual at both endpoints |
| **`loopcount`** | 88 | **18 cells DISAGREE** | the joint fixed point's pass count — see finding 4 |

**ALL FOUR SURVIVING PREDICTIONS HELD, and two of them are only meaningful because they are
COUNTS.** P1 has two halves and BOTH were checked, which is worth spelling out because the gate
alone only proves the first: the gate shows Rust == Python, and P1 also asserted Python == a
specific table. The turbine solve takes **47** map evaluations at every call with no spread; and
read off the oracle at the design Mach, the joint-loop table is

| gas | `Tt4` = 1500 | 1100 | 900 |
|---|---|---|---|
| calorically perfect | **1** | **7** | **7** |
| thermally perfect | **2** | **7** | **7** |
| reacting equilibrium | **200** | **200** | **7** |

— cell for cell the `1/7/7 · 2/7/7 · 200/200/7` § (b) predicted, including the 200s, where the
answer *is* the 200th iterate of a non-converging loop. P2 (as re-scoped): **550 low / 46 high**
march rejections, per cell, on the grid now recorded in the oracle. P4: the three `Tt ** 0.5`
sites are libm `pow`s, and would have shown as value drift if not. (P3 was already refuted both
ways by § (g).)

*A near-miss worth recording, because it is finding 2's own error committed one paragraph above
finding 2:* the first version of this section claimed P1 "held" on the strength of the gate
alone, which measures agreement with Python and says nothing about agreement with the predicted
table. The table was then read off the oracle and does match — but the claim was written before
it was checked. **A gate that pins A to B is not evidence about B's own value.**

**FINDING 1 — THE TWO RUNGS, RESTATED AS A COUNT OVER BIT PATTERNS, WHICH IS STRICTLY STRONGER
THAN EITHER SUITE'S OWN GATE.** Both Python suites express rung 31's pin and rung 33's inversion
as tolerances (`|tau_t − tau_t0| < 1e-9`, `spread > 1e-3`). A tolerance cannot distinguish *the
pin holds* from *the pin nearly holds*, and the two claims are not even in the same currency.
Counted over distinct `f64` bit patterns instead, on the same 3-gas × 6-Mach × 7-throttle grid:

| gas | choked `tau_t` | subsonic `tau_t` |
|---|---|---|
| calorically perfect | **1 pattern / 26 cells** | **4 / 4** |
| thermally perfect | 24 / 24 | 5 / 5 |
| reacting equilibrium | 24 / 24 | 5 / 5 |

So (★) is not merely tight on a CPG gas — it is *exact*, one value across the whole throttle AND
flight-Mach sweep — while the subsonic branch on the same gas moves at every point. **The pairing
is what makes it a measurement**: the second column alone would pass on a sweep too narrow to
resolve anything, and the first alone would pass on a solver that had stopped responding to its
inputs. `rung31.rs::the_pin_is_exact_not_merely_tight` gates both halves together.

**FINDING 2 — A COUNT WITHOUT ITS GRID IS NOT A MEASUREMENT.** § (a) recorded "930 raises on the
low march and 616 on the high" but not the sweep that produced them, so the number could not be
reproduced, could not be ported as a prediction, and had to be re-measured from scratch; the same
3 × 6 × 7 shape gives 550 / 46. This is the fourth distinct way the port has been bitten by a
claim whose *scope* was left implicit — after the dead-guard sweeps (§ 5.3, twice), the census
restricted to its own phase, and § (i)'s dump that could not see a route. The grid now lives in
the oracle file beside the counts, where a reader gets both or neither.

**FINDING 3 — THE ENVELOPE IS GATED AS DATA, NOT AS A SKIP LIST.** 38 of the 126 cells abort, for
five distinct reasons. Each is dumped as a numeric abort CODE, so a Rust that aborted for a
*different* reason — or matched a cell Python could not — fails on that key instead of silently
producing a shorter file. All 38 agree, which is what makes the fallible-path design of § (i)
verified rather than merely argued: the rejection set, the resulting bracket, and the reason each
cell dies are all reproduced exactly.

**FINDING 4 — THE ONE QUANTITY IN THE PORT THAT IS NOT INTERPRETER-INVARIANT, AND IT IS THE
SHARPEST CONFIRMATION OF § (g) AVAILABLE.** Every slice so far has found the discrete keys
exactly equal on both interpreters — counts, indices, censuses — because integers do not drift.
Here **one** discrete class does not: the joint `(f, pt4)` fixed point's pass count differs
between CPython and PyPy on **18 of 88 cells**, flipping **7 ↔ 200 in BOTH directions**, and
every disagreeing cell is on the equilibrium gas. That is § (g)'s "the stopping rule is
unmeetable" caught in the act — the test fails by such a hair that last-bit arithmetic decides
which side of it a cell lands on.

Two things make it a finding rather than noise, and both are asserted in the gate rather than
narrated: the instability is **confined to the equilibrium gas** (a calorically-perfect cell
flipping would mean something else entirely), and **it never moves a value** — the same cells'
`pi_c`, `tau_t` and thrust still agree to 2.6e-10. A cell that ran 200 passes and one that ran 7
land on the same answer, so the loop *is* converged in any physical sense and only the stopping
TEST fails to say so. The class is bit-gated on PyPy, where it is prediction P1, and explicitly
not compared on CPython, with the disagreement count printed on both arms.

**This sharpens the phase-6 inbox item at § (h).** `TwoSpoolTransient` drives `match()` in loops
at five sites; a 28× cost swing that flips on last-bit arithmetic is not only a per-timestep
expense but a reproducibility hazard for anything that counts work.

**WHAT SLICE J OWES, WRITTEN DOWN RATHER THAN LEFT TO BE NOTICED.** `test_rung33.py`'s gate 7 has
a second half asserting that rung 32's `MapMatcher` does NOT inherit rung 33's dispatch. Rung 32
does not exist in the Rust yet, so there is nothing for that claim to be true or false of; it is
recorded as `rung33.rs::slice_j_deferrals`, which gates the part that IS checkable today.

### 5.6 SLICE J (rung 32, `ComponentMap` + `MapMatcher`) — PRE-REGISTERED, eight probes MEASURED first

**The slice.** Rung 32 is `ComponentMap` (an analytic compressor efficiency island + speed lines
and a near-flat turbine map) and `MapMatcher`, which subclasses slice I's `OffDesignMatcher` and
adds exactly one thing: the component efficiencies stop being held at design and are read off the
map at the operating point, closed by an OUTER secant, with the shaft speed `N` attached from the
speed-line inversion. Everything else — the choke machinery, `solve_turbine`, `solve_f`,
`working_gas` — is slice I's, unchanged.

**THE FIRST QUESTION WAS A BLOCKING ONE, AND IT DECIDED THE SHAPE OF A FUNCTION.**
`_operating_point` calls `self._solve_turbine`. `class SpoolTransient(MapMatcher)` — rung 34,
phase 6 — **overrides `_solve_turbine` and overrides neither `_operating_point` nor `match`**, so
rung 32's operating point is a **SECOND live site** for the virtual hook slice I shipped, and it
is a site § 5.3's census could not name because the census enumerated (name, ancestor, descendant)
triples rather than call sites *inside* a class that does not yet exist in the Rust. Calling
`r31_solve_turbine` directly here would compile and return a silently different number in phase 6
— § 1's leaf-dispatch trap, which is the one failure mode the const-table architecture exists to
make impossible. `operating_point` therefore dispatches through `self.solve_turbine`, and that is
settled before the function is written rather than after it is gated.

**Probed BEFORE registering** (the standing lesson), on both interpreters, and two of the eight
probes changed a claim that was about to be registered wrong.

**(a) BOTH OF RUNG 32'S OWN RAISE SITES ARE DEAD — swept 8× wider than the gates, because a
narrow sweep is exactly how slice I's (a) got its first answer wrong.** `ComponentMap.solve_n`
asserts its bisection bracket straddles the root; the outer secant has a `for…else raise` at
`_ETA_MAX = 80`. Swept at **3 gases × 6 flight Machs × 9 throttles × 5 map shapes = 810 cells**:
the bracket assert would fire **0 times**, the secant exhausts its cap **0 times**. The sweep
does produce 135 raises — and **all 135 are at `M0 = 0.0`, all 135 are `Inlet`'s "ram must not
cool/depressurize" or the CPG efficiency cascade, and `OffDesignMatcher.match` raises the
identical assert at the identical cells**. They are rung 31's static-flight edge, not rung 32's.
**Consequence: `MapMatcher` needs no fallible path.** That is the OPPOSITE of slice I's answer to
the same question, and it is not a weaker sweep — it is a genuinely different structure, because
nothing in rung 32 marches a bracket past a failure.

**(b) `solve_n` COSTS EXACTLY 48 RESIDUAL EVALUATIONS AT EVERY CALL** — `min = max = 48`,
one distinct value over **269 calls**, on both interpreters. Bracket `[0.1, 2.0]`, break
`hi - lo <= 1e-14` **absolutely** ⇒ `ceil(log2(1.9 / 1e-14)) = 48`. The direct analogue of slice
I's P1 (47 for the turbine solve) and, like it, a count that differs means the arithmetic diverged
somewhere a value gate still passes.

**(c) THE `eta_c` CLAMP NEVER BITES**, 0 times over the 99-cell instrumented sweep, so the
`min(max(x, 0.3), 1.0)` spelling is **moot** — Python's `min`/`max` and Rust's `f64::min`/`max`
disagree on NaN, and the port would have had to choose. Recorded rather than reasoned about: the
Rust uses the Python's argument order and says at the site that the branch is unreached, so a
future rung that DOES reach it inherits a decision that was made deliberately.

**(d) THE OMITTED LATER-RUNG TERMS IN `psi` ARE BIT-INERT — MEASURED, NOT ARGUED.**
`ComponentMap` carries fields for rungs 34 (`l`), 36 (`phi_surge`), 53 (`vsv`) and 54
(`capacity`); slice J ports **only rung 32's five** (`a, b, c, sigma, a_t`), so its `psi` is
`1 - sigma*(phi-1)^2` where the Python's is `… - l*(phi-1)`. Over **26 900 evaluations on the
real swept `phi`**, the two spellings are bit-identical **26 900 / 26 900** on both interpreters.
`is_flat` and `phi_max` are **not ported and the reason is written down**: `is_flat` is called by
no engine code at all (only by rung-36/41/53/54 *tests*), and `phi_max` only by the rung-34/40/43
forward transient closures — phase 6. Neither is reachable from any rung-32 gate.

**(e) THE FLAT-MAP REDUCE IS BIT-EXACT, WHERE THE PYTHON GATES IT AT `1e-9` — AND THE FIRST
STATEMENT OF THAT CLAIM WAS AN OVER-CLAIM.** With a flat map `eta_c_at` returns its base
untouched, the residual is exactly `0.0`, the secant breaks on pass 1, and the operating point is
computed at the design efficiencies — so every rung-31 quantity should be bit-identical, not
merely close. Measured on the reacting gas at gate 1's three throttles: **10/10 scalars, all six
stations' `Tt` and `pt`, and `specific_thrust`, bit-equal, on both interpreters.** Then swept to
**3 gases × 3 Machs × 6 throttles**, and it FAILS on two cells — `(thermally-perfect, M0 = 0.85,
Tt4 = 600)` and `(equilibrium, M0 = 0.85, Tt4 = 600)` — in **nine scalars and four stations at
once**. Those are precisely the cells where **rung 31 dispatches to the subsonic branch and rung
32 does not**, so the comparison there is not a reduce at all: it is (f) below, seen from the
other side. **The registered claim is therefore conditional on the branch**, and the Python's
gate 1 never met the condition because all three of its throttles are choked.

**(f) THE RUNG-33 IOU DISCHARGES AS A CONTRADICTION, NOT AS AN ABSENCE.** At `Tt4 = 560` on the
CPG gas, `OffDesignMatcher.match` returns `branch = 'subsonic'`. `MapMatcher.match` at the same
cell **returns** — it does not abort — with `nozzle_choked = False` **and `branch = 'choked'`**:
rung 32 predates rung 33 in the ladder, never got the dispatch, and inherits
`OffDesignResult`'s default label, so the result carries a branch label its own flag contradicts.
That is a sharper discharge of `rung33.rs::slice_j_deferrals` than "does not inherit", and it is
the same fact (e)'s two failing cells report.

**(g) THE INNER LOOP'S DISCRETE INSTABILITY IS CONTAINED — IT DOES NOT AMPLIFY THROUGH THE OUTER
SECANT. The first measurement of this could not have detected it.** § 5.5 found the joint
`(f, pt4)` pass count flipping `7 ↔ 200` between CPython and PyPy on 18 of 88 cells, **every
disagreeing cell on the equilibrium gas**; rung 32 wraps that loop in an outer secant, so the
question is whether the flip propagates outward. The first sweep compared 99 cells of outer
counts across interpreters, found **99/99 identical** — and was **swept on the thermally-perfect
gas, where the flip does not live either**. Re-measured on the equilibrium gas, 3 shapes × 4
throttles: the inner counts disagree between interpreters in **8 of 12 cells** (e.g. `tilted`,
`Tt4 = 1100`: PyPy `[200, 7, 9, 200, 200]` vs CPython `[8, 200, 8, 200, 9]`) while the outer
secant count is identical in **12 of 12** (`1, 5, 5, 6` on both). A cell that ran 200 inner passes
and one that ran 8 hand the secant the same operating point to well inside its own `1e-11`
tolerance, so the outer count is a **usable** key exactly where the inner one is not.

**(h) THE OUTER SECANT IS 1 PASS ON EVERY FLAT MAP, INCLUDING THE `sigma` VARIANTS.** Gate 6
sweeps `sigma ∈ {0, 0.3, 0.6, 1.0}` expecting an `N` schedule that genuinely moves with it — and
`sigma` enters `solve_n` only, never `eta_c_at`, so all four run the secant exactly once. The
gate's "not a tautology" bar (`spread > 1e-4`) is therefore testing the speed-line inversion in
isolation, with the efficiency loop inert. Worth saying because it means gate 6 does **not**
exercise the machinery gate 3 does.

**The predictions, registered before the Rust is written.** Each names what would refute it.

* **P1 — the reduce is `==`, not a tolerance, ON THE CHOKED BRANCH.** With a flat map, every
  quantity `OffDesignMatcher` also computes is bit-identical between `MapMatcher::match` and
  `match_point` at every cell where the rung-31 answer has `nozzle_choked == true`. The map
  read-offs `n_corr`, `N_ratio`, `flowcoef`, `nu_t` are **explicitly outside** the claim — they
  come from `solve_n`, arithmetic rung 31 never runs. *Refuted by:* any choked cell needing a
  tolerance; or by the claim holding on an UNCHOKED cell, which would mean the Rust inherited a
  dispatch the Python does not have.
* **P2 — `solve_n` takes exactly 48 residual evaluations, with zero spread, at every call in the
  dump.** *Refuted by:* any other count.
* **P3 — the outer secant count is interpreter-stable where the inner is not.** Over the oracle's
  own grid the outer count agrees CPython-vs-PyPy at 100 %, on cells where the inner counts do
  not. *Refuted by:* one disagreeing outer count. This EXTENDS § 5.5's finding rather than
  re-running it — § 5.5 established the instability, P3 bounds its reach.
* **P4 — `MapMatcher` needs no fallible path**, i.e. no `try_` twin is added to the crate by this
  slice. *Refuted by:* any cell in the dump grid at `M0 > 0` that raises from `solve_n`'s bracket
  or the secant's cap.
* **P5 — rung 32 does NOT inherit rung 33's dispatch, and says so contradictorily**: at
  `Tt4 = 560` CPG the Rust returns `nozzle_choked == false` with `branch == Choked`, where slice
  I's matcher returns `Branch::Subsonic`. *Refuted by:* an abort, or a subsonic answer.
* **P6 — `n_corr` is the LEAST cross-interpreter-stable quantity in the slice, and it is worst AT
  THE DESIGN POINT.** Three points say so (`Tt4 = 1500`: 1.5e-11; `1200`: 9.1e-13; `900`:
  1.6e-12), and the mechanism would be that `solve_n`'s target `(tau_c-1)/(tau_c_d-1) → 1` at
  design, so the bisection resolves `n - 1` against a target whose own deviation from 1 is what
  carries the error. *Refuted by:* the design point not being the worst cell on the full grid —
  in which case the conditioning story is wrong and the three points were a coincidence, which is
  precisely why this is registered rather than asserted.

**THE ONE REFACTOR THIS SLICE MAKES TO GATED CODE, and how it is kept honest.**
`OffDesignMatcher::rebuild` hardcodes `self.eta_c` / `self.eta_t`; rung 32 needs the same
instruction sequence at the map's efficiencies. The Python duplicates the whole rebuild inside
`MapMatcher.match`. The port instead **parameterises `rebuild` with `(eta_c, eta_t)`** — the port
already shares that function between rungs 31 and 33 on exactly this reasoning, and slice F's
*don't factor a deliberate duplication away* was about two loops that only LOOK alike, which this
is not. What makes it safe is not the argument: `offdesign_oracle`, `rung31` and `rung33` are
re-run afterwards and must be **bit-identical**, or the refactor is reverted.

**Sizing.** Gate 1 on the reacting gas at three throttles; gates 3–7 on the thermally-perfect gas,
as the Python's own `_fast_matchers` does — the outer secant multiplies the equilibrium re-freeze
per inner pass, and an equilibrium sweep across all shapes is the slice's cost trap.

#### What phase 5 SLICE J MEASURED — SHIPPED, **7 252 / 7 252 bit-exact** vs PyPy

`rust/src/map.rs` + `rust/oracle/dump_map.py` + `rust/tests/map_oracle.rs` + `rust/tests/rung32.rs`.
Against CPython the same dump is **69.2 %** identical: bit-exact on the calorically-perfect gas
(2 588 / 2 588) and ~26 % on the thermally-perfect and equilibrium ones, because every cell is a
three-deep nest of loops whose last bits are carried outward.

**The verdicts, one per registered prediction.**

* **P1 — CONFIRMED, and its conditional half now has EVIDENCE rather than a qualifier.** 28 of 28
  choked cells reduce bit-for-bit against `OffDesignMatcher` on a flat map, across three gases.
  The registered refutation clause ("the claim holding on an UNCHOKED cell") is not merely absent
  — it is measured: **0 of 4** subsonic cells reduce bit-exactly, and the gate asserts
  `n_sub_bitequal < n_sub`. That assertion was missing from the first draft, which asserted only
  the choked half; "bit-exact on the choked branch" would then have been a qualifier with nothing
  behind it, and a subsonic cell that happened to land back on rung 32's answer would have read as
  support for the claim.
* **P2 — CORRECTED: 50, not 48.** Same loop, same zero spread (one distinct count over 120 swept
  calls on both interpreters), but the registration counted the bisection steps and forgot the two
  bracket-endpoint evaluations that decide the assert. `ceil(log2(1.9/1e-14)) = 48` steps **+ 2**.
  The claim survives; the number a gate compares against has to be the one the instrument reads.
* **P3 — CONFIRMED, cleanly.** The outer secant's pass count agrees CPython-vs-PyPy on **144 of
  144** cells; the inner turbine-solve total disagrees on **5 of 144**, and every one of those five
  is on the equilibrium gas. So § 5.5's discrete instability is real and CONTAINED: a cell that ran
  200 inner passes and one that ran 8 hand the secant the same operating point to well inside its
  `1e-11` tolerance. The outer count is bit-gated on both arms; the inner one on neither but PyPy.
* **P4 — CONFIRMED.** Rung 32's three own raise sites (the secant cap, the physicality assert, the
  speed-line bracket) fire **0 times** across the 152-cell grid — dumped as explicit zero counts
  under `census/abort_code/7..9` rather than left as an absence — and **0 times** over 120
  standalone `solve_n` calls that now include `sigma = 1.0`. All 8 aborted cells are code 2, rung
  31's efficiency cascade. **No `try_` twin is added by this slice.**
* **P5 — CONFIRMED as the predicted contradiction.** At `Tt4 = 560` on the reacting gas rung 32
  returns `nozzle_choked = false` with `branch = Choked`; rung 31's own matcher at the same cell
  returns `Subsonic`. Discharged at `rung32.rs::rung33_gate7_second_half_map_does_not_inherit_subsonic`;
  `rung33.rs::slice_j_deferrals` is KEPT, because it carries the contrast the rung-32 file cannot.
* **P6 — REFUTED, BOTH HALVES.** `n_corr` is **not** the least cross-interpreter-stable quantity:
  it ranks **21st of 38** at 2.68e-11, an order of magnitude better than `F_over_mdot` at 2.95e-10.
  And it is **not** worst at the design point: its worst three cells are all at `Tt4 = 1100`, at
  every flight Mach, on the FLAT map. The registered mechanism (the `solve_n` target → 1 at design,
  so the bisection resolves `n - 1` against a target whose own deviation carries the error) is
  therefore wrong, and the three probe points that suggested it — 1500, 1200, 900 — were a
  coincidence of a grid that **never sampled 1100**. The lesson is the port's own, restated: the
  argmax that pays is the one that disagrees, and a location claim must be swept wider than the
  points that suggested it.

**AND THE HEADLINE ALMOST SHIPPED IN THE WRONG CURRENCY.** Rung 32's finding is that the compressor
WORK is choke-pinned and map-free while `pi_c` and `mdot` are not. The port's usual way to say
"constant vs varies" is a count of distinct bit patterns — and that count is a perfect
NON-discriminator here: `tau_c`'s bits move across the four map shapes in **every one of the 32**
non-equilibrium cells, exactly as `pi_c`'s do. `tau_c` is map-free STRUCTURALLY (no map coefficient
enters the shaft balance that sets it) but it is reached through a fixed point whose other variables
do move with the map, and a converged iterate carries its history in the last bits. The claim is
about MAGNITUDE and always was — Python's gate 4 bar is `1e-4`, not zero. Gated as a relative spread
across shapes instead: **`3.65e-6` for `tau_c` against `3.76e-2` for `pi_c`**, and the assertion is
the RATIO (measured `1.03e4`, barred at `1e3`) rather than the direction `tau_c < pi_c`, which would
still pass with the map-freeness entirely gone.

**THE ORACLE IS BLIND TO THE SQUARE-SPELLING RULE, AND THAT WAS MEASURED RATHER THAN SUSPECTED.**
`porting_rules.rs` was checked for whether it covers a new module and the answer is that it reads
no source at all — it is a BEHAVIOURAL test that the spellings remain distinguishable, so no module
is inside or outside its coverage. Which raised the real question: does anything gate `map.rs`'s
spelling? `psi` was mis-spelled `powp(u, 2.0)` on purpose and the 7 252-key bit-equality oracle
**passed both arms**. The reason is in `porting_rules.rs`'s own printout — `pow(x, 2)` differs from
`x*x` at **1 point in 4012**, and the oracle sweeps 60 `psi` evaluations, so its power to catch this
is ~1.5 %. All three squares in the file are now gated DIRECTLY at
`rung32.rs::the_three_squares_are_multiplies_not_pow_calls`: a 40 000-point grid, the shipped
function asserted equal to the multiply spelling at every point, plus a vacuity guard that the grid
can tell the spellings apart at all (it does, at 19/17/22 points). Re-applying the mis-spelling now
fails it. **The general lesson: "100 % bit-exact" bounds the CELLS the oracle visited, not the RULES
it can discriminate — and a rule whose two spellings differ once in 4 000 needs its own grid.**
(The first draft of that gate FAILED against correct code, because it reconstructed `u` as the
sweep variable rather than as `(1.0 + step) - 1.0`, which does not round-trip.)

**Three further coverage gaps closed after the first green run**, all of the same shape — an
instrument that looked complete: (i) the oracle's four shapes did not include `a_t = 0.5` or
`sigma = 1.0`, the
coefficients rung 32's OWN gates 5 and 6 use, so the count and curvature claims were pinned on a
band narrower than the gates relying on them; they are now in the standalone sweeps, which cost no
cycle solve. (ii) A `CELL_Q` name list justified in a comment as keeping three arms in step was
read only by a `debug_assert_eq!` — compiled out in `--release`, the only profile the gate runs in.
Deleted. (iv) `MapMatcher::match_point` — the no-argument form that reads the constructor's stored
map — had NO coverage at all, because every gate and the whole oracle pass a map explicitly. The
rung-33 discharge now goes through it, which is also what the Python's own gate does. (iii) The
Python suite's three easiest-to-drop bars were copied verbatim: gate 4's
`dpc > 30*rel` is CONDITIONAL on `Tt4 <= 1100`, gate 6's spread bar is TWO-SIDED (`< 0.05` **and**
`> 1e-4`, and the lower half is what stops the robustness claim being a tautology about a quantity
that never moved), and gate 1's specific-thrust bar is ABSOLUTE where its neighbours are relative.

### 5.7 SLICE K (rungs 38 + 39, the TWO-SPOOL matchers) — PRE-REGISTERED, four probes MEASURED first

**The slice.** `build_two_spool_turbojet` / `TwoSpoolEngine` / `TwoSpoolResult`, rung 38's
`TwoSpoolMatcher` (the `(★)` choke solver parameterised to serve *both* turbines, the burner
`f`-solve, the triangular `_cascade`, `match`) and rung 39's `TwoSpoolMapMatcher`
(`_secant`, `_hp_eta_loop`, `_lp_eta_loop`, `_cascade_map`, `match`). Both `lp_disabled` reduce
paths dispatch into already-shipped code — rung 38's to slice I's `OffDesignMatcher`, rung 39's to
slice J's `MapMatcher` — so the whole four-rung ladder (flat+disabled → 31, shaped+disabled → 32,
flat two-spool → 38, shaped two-spool → 39) closes inside this slice.

**THE BOUNDARY IS DECIDED BY THE HOOK, AND THE HOOK IS DECIDED BY RUNG 41.** § 5.3's census
justified a virtual `match` by three `self.match` call sites at `engine.py:3193, 3233, 3261`.
Enumerating their enclosing `def`s puts **all three inside rung-41 methods** — `surge_margin`
(3174), `running_line_map` (3226), `flow_coefficient_turn` (3242) — and rung 41 was in **no
phase at all** until the audit recorded above § 5.1. So:

* **Slice K = 38 + 39, the rung-41 surge methods EXCLUDED.** Measured, not assumed: rung 38's and
  rung 39's own suites (6 + 10 tests) reference no surge method and no `phi_surge`.
* **Slice L = 41 + 42.** Rung 42 *overrides* `match`; rung 41 *calls* it on `self`. They are the
  hook's live pair, and gating dispatch needs both. Rung 41's gates 1–2 reach `TwoSpoolTransient`
  (phase 6), so part of it defers — the `rung33.rs::slice_j_deferrals` precedent, reused.
* **Consequence, NAMED rather than left to be noticed: slice K ships three hooks with ZERO live
  call sites** (`match`, `_hp_eta_loop`, `_lp_eta_loop`; the eta loops are overridden by rung 55,
  phase 7). That is the same standing as slice I's `solve_turbine`, which is unexercised until
  phase 6 — an architectural requirement of § 5.3's census, not an instrument. The distinction
  from slice J's deleted `debug_assert_eq!` is that a hook's job is to exist on the day the
  overriding rung lands; an instrument's job is to fire.

**PROBED BEFORE REGISTERING** (`M:\claud_projects\temp\rust-slice-k\probe_{slice_k,counts,envelope}.py`),
and the first probe's grid was wrong in exactly the way the port keeps re-learning:

**(a) `l` IS NOT INERT HERE, AND THAT IS A SLICE-J TOUCH.** § 5.6 (d) recorded rung 34's linear
loading slope `l` as absent from the Rust `ComponentMap` because it is `0.0` at every rung-32
call. **Rung 39's own test shapes set it** — `SHAPES_C` in `tests/test_rung39.py` carries
`l = 0.7 / 0.85 / 1.0` on 2 of 2 maps in 5 of its 6 shapes, and over a `phi ∈ [0.6, 1.3]` sweep
the two `psi` spellings differ by **27 % to 43 % relative**, not in the last bit. Porting rung 39's
gates on `l = 0` shapes instead would narrow the band the source itself gates on — the vacuity
trap of § 4.9. So `l` is added to `ComponentMap` now, under slice J's own discipline: `map_oracle`,
`rung32`, `offdesign_oracle`, `rung31` and `rung33` are re-run and must be **bit-identical** or
the change is reverted (registered as P5). `phi_surge` is **not** added — it is rung 41's, and it
follows the surge methods into slice L.

**(b) THE INHERITED QUESTION (§ 5.4 (h)) — ANSWERED, AND IT INVERTS THE SINGLE-SPOOL ANSWER.**
Slice I left "whether the two-spool matcher shares the 200-pass behaviour is slice K's to
establish". It does, and **far more widely**: on a 126-cell grid (3 gases × 6 flight Machs ×
7 throttles from 900 to 1500 K) **23 of 105 matched cells** exhaust the cap on rung 38 and 10 of
72 on rung 39 — against slice I's *two* cells on the single spool. All but one are the
equilibrium gas (the exception is `tpg, M0 = 1.60, Tt4 = 1500`). **The mechanism is the
inversion.** Slice I found the two halves of `done` failing for *different* reasons at different
throttles (`f` exactly constant at 1500; a two-value `f` cycle at 1100). Here
`n_distinct_f == n_distinct_pt4` in **every one of the 33 capped cells** — the two halves cycle
*together*, over 2 to 6 distinct values, with `|Δf|/f` between 5.4e-13 and 2.8e-11 and
`|Δpt4|/pt4` about 3× that. Both sit above the `1e-13` bar, so the rule is unmeetable — but by
ONE mechanism on two spools where it was two mechanisms on one.

**(c) THE PASS COUNT IS NOT CROSS-INTERPRETER STABLE, AND IT IS NOT A WOBBLE — IT IS 8 vs 200.**
CPython and PyPy disagree on the joint-loop pass count at **29 of 126** rung-38 cells and **19 of
144** rung-39 cells, every one on the equilibrium gas, and the disagreement is between converging
in ~8 passes and never converging at all. Whether this loop terminates is an interpreter
property. Three consequences, and the second is a correction of a shipped instrument's reading:

1. The outer pass count may be gated **against PyPy only**, never on both arms.
2. § 5.6's P3 said the *outer* secant count agrees on 144 of 144 cells while only the *inner*
   turbine count disagrees, and read that as the discrete instability being "contained". It is
   contained **in rung 32**, not in general: the two-spool outer loop is the same class of
   count and it is the *least* stable quantity in this slice.
3. The cells that cap must reproduce **200 passes bit-for-bit against PyPy**, exactly as slice I's
   two did — registered as P2.

**(d) THE BISECTION IS ONE NUMBER, AND THERE ARE THREE OF IT.** `_solve_choked_turbine`'s
bracket is `(0.02, 0.999)` against `_TOL = 1e-13` absolute ⇒ `ceil(log2(0.979 / 1e-13)) = 44`
iterations. Measured over **10 502 calls** on both interpreters: **one** distinct value, and the
counter reads **47** — because `tau_of` runs once per residual evaluation *and once more after the
loop*, and there are 46 residual evaluations (44 + the two bracket endpoints). § 5.6's P2 was
CORRECTED for exactly this reason one slice ago; the number a gate compares against must be the
one its instrument reads, so the gate names which of 44 / 46 / 47 it counts.

**(e) THE ENVELOPE — AND MY OWN FIRST GRID REPEATED THE PORT'S OLDEST MISTAKE.** On the probe's
first grid (`Tt4 ≥ 900`) **not one** of rungs 38/39's own asserts fired, which reads as "the
scope guard is dead". On slice I's inherited grid rung 38's **UNCHOKE scope guard fires on a
broad band along the cold/slow edge**, not in a corner. *A count without its grid is not a
measurement*, and the instrument that produced the wrong reading was mine. Slice I's grid is
inherited wholesale and **extended by the `M0 = 0` column of (f)**, giving
`M0 ∈ {0, 0.3, 0.5, 0.85, 1.2, 1.6, 2.0} × Tt4 ∈ {400, 500, 600, 650, 900, 1100, 1500} × 3 gases
= 147 cells`. **The census was then re-measured on THAT grid, and the first number was wrong:**
`UNCHOKED` is **23**, not the 20 seen on 126 cells. Reading a census off the probe's grid and
gating it on the dump's is precisely how § 5.6's P2 got its number wrong one slice ago. Full
census, rung 38: **68 matched / 79 abort** — `UNCHOKED` 23, `root not bracketed` 18,
`ram must not cool` 14, `efficiency cascade` 12, `equilibrium Newton` 7, `nozzle back-pressure` 3,
`burner f` 2. Five of rungs 38/39's own asserts fire **zero** times (the `(★)` bracket straddle,
the physicality check, both efficiency secants, the outer turbine-efficiency loop, `solve_n`'s
speed-line bracket, and `TwoSpoolEngine.run`'s two shaft-closure checks) — dumped as explicit
zeros, per § 5.6's P4. **Rung 39 with shaped maps differs from rung 38 on exactly ONE cell of
147**: `tpg, M0 = 1.60, Tt4 = 600` matches on rung 38 and hits the efficiency cascade on rung 39.
The map moves one cell across the envelope, and nothing else. Two families are NEW to slice I's
`ABORT` table (`ram must not cool`, `UNCHOKED`); they are **appended**, never renumbered, because
the Rust compares the code.

**(f) `M0 = 0` IS EXCLUDED BY A SOLVER ROUND-TRIP, NOT BY PHYSICS — AND SLICE I NEVER SAW IT**
(its grid started at 0.3). At `M0 = 0` the freestream's two-clause assert `Tt0 >= T0 and
pt0 >= p0` fails on its **first** clause for the thermally-perfect and equilibrium gases and
passes entirely on the calorically-perfect one: `T_from_h_c(h_c(250.0))` returns
**249.99999999999994**, three ulps low, while the pressure clause is exact (`pr_c` ratio `1.0`).
The CPG branch is a closed form and round-trips exactly, so its `M0 = 0` cells get as far as
`_score` and abort there instead. The column is put **on** the dump grid rather than trimmed off
it, because a Rust round-trip landing on the other side of exactness would move an envelope
boundary that no value comparison can see (P6).

**(g) THE `_secant` CLAMP IS DEAD.** `min(max(nxt, 0.3), 1.0)` binds **0 times** across all 72
matched cells of the 144-cell shaped grid (6 map shapes × 3 gases × 2 Machs × 4 throttles).
Ported as written — it is a guard on a path this grid does not reach — but recorded as dead so a
reader does not infer it is load-bearing, and so slice L knows what it is looking at.

**(h) THE SLICE CONTAINS NO SQUARES — BUT THAT IS THE WRONG READING OF SLICE J's LESSON.** All
**14** `**` operators in the ported range are `** 0.5`, and the `l` term added under (a) is
linear, so no new square enters. The first draft of this paragraph stopped there and concluded
that `rung32.rs::the_three_squares_are_multiplies_not_pow_calls` "has nothing to cover here".
**That is wrong, and it is wrong in slice J's own way.** (a) *changes `psi`* — the one function
slice J proved the 7 252-key oracle is blind to, because it sweeps 60 evaluations of a rule whose
two spellings differ once in 4 012 points. Python's body is
`1.0 - sigma*(phi-1)**2 - l*(phi-1)`, and float subtraction is not associative: writing
`1.0 - l*u - sigma*(u*u)` is algebraically identical, differs in the last bit, and would pass
every oracle key. The existing gate cannot see it either — its `cmap` leaves `l` at `0.0`, so the
term is not present to be reordered. It is extended with an `l != 0` arm asserting the
left-to-right order, carrying its own vacuity guard that the reordered spelling is distinguishable
on the grid at all. **The rule slice J actually established is not "gate squares" — it is "gate any
change to `psi` DIRECTLY, because the oracle cannot see one."**

#### The predictions

* **P1 — THE ONE ARROW, AND THE STRUCTURAL HALF STATED NARROWLY ENOUGH TO BE FALSE.** Rung 39's
  finding A is that `_hp_eta_loop` is closed: perturbing `eta_lpc` leaves `pi_hpc` bit-for-bit
  unchanged, while perturbing `eta_hpc` moves `pi_lpc`. **The first draft of this prediction said
  "the HP loop's signature carries no LP quantity, so the closure is a compile-time fact" — and
  that is FALSE on code already read**: the signature takes `Tt25`, which the LP energy balance
  produced, and `self` reaches `eta_lpc` in Rust exactly as in Python. A prediction that can be
  "confirmed" without checking anything is the rung-61 trap (a derived quantity whose binding
  constant is mine). The claim is therefore the narrow one rung 39's gate 4 actually tests: **no LP
  EFFICIENCY and no LP PRESSURE RATIO enters the HP loop** — `Tt25` does, and it is map-free by
  rung 38's energy cascade. The Rust makes that checkable by taking the HP loop's inputs as
  explicit scalars rather than off `self`, so `eta_lpc` is not in scope inside it.
  *Refuted by:* any `eta_lpc` perturbation moving a single bit of `pi_hpc`.
* **P2 — THE CAPPED CELLS COME BACK BIT-EXACT AGAINST PYPY, ALL 200 PASSES.** *Refuted by:* any
  cell whose Rust joint-loop pass count differs from PyPy's, or whose value differs in a bit.
  This is the slice's real risk: (c) shows the count is a knife edge between 8 and 200, so a
  single last-bit divergence anywhere in the cascade shows up as a 25× count difference rather
  than as a small value drift.
* **P3 — THE BISECTION COUNT IS ONE DISTINCT VALUE ON THE DUMP GRID**, and the gate names which
  of 44 iterations / 46 residual evaluations / 47 `tau_of` calls it reads. *Refuted by:* two
  distinct values, on either interpreter.
* **P4 — RUNG 38's AND 39's OWN ASSERTS, AS EXPLICIT COUNTS ON THE 147-CELL DUMP GRID**:
  `UNCHOKED` exactly **23**; the `(★)` bracket, physicality, both secants, the turbine-eta loop, the
  speed-line bracket and both shaft-closure checks exactly **0**, dumped as zeros under
  `census/abort_code/…` rather than left as an absence. *Refuted by:* any nonzero, which would
  mean the Rust reaches a failure mode the Python does not.
* **P5 — ADDING `l` MOVES NO SHIPPED BIT.** `map_oracle` (7 252 keys), `offdesign_oracle`
  (3 951), `rung31`, `rung32` and `rung33` re-run bit-identical after the `ComponentMap` change.
  *Refuted by:* one changed bit ⇒ revert, per slice J's own rule for a refactor to gated code.
* **P6 — THE `M0 = 0` ENVELOPE IS GAS-SPLIT IN RUST TOO**: the TPG and equilibrium cells abort on
  the ram clause and the CPG ones get past it. *Refuted by:* a Rust round-trip that is exact on
  the integral gases (the cell would then match, and no value comparison would notice).

**Module decision.** A new `rust/src/two_spool.rs`. `matcher.rs` is 843 lines and `map.rs` 554;
rung 38 + 39 is ~940 lines of Python and would push `matcher.rs` past 1 900. The dependency is
strictly one-way (two-spool consumes `OffDesignMatcher`, `MapMatcher`, `ComponentMap`,
`choked_mfp`, the components and the gas; nothing consumes it until slice L), which is the same
test slice F applied when it split `march.rs` out.

**THE ONE LOOP-SHAPE DECISION THAT DECIDES A REDUCE GATE.** Rung 39's gate 1 — flat maps
reproduce rung 38 bit-for-bit — does not hold because flat maps make the arithmetic agree. It
holds because both efficiency loops **test the residual BEFORE ever calling `_secant`**, so on a
flat map they return having done no secant arithmetic at all (measured: `hp_passes` includes 0 on
27 of the shaped grid's cells), and because the outer turbine-efficiency loop returns on its first
pass when `a_t == 0`. **A `do`-while shape converges to the same place, looks correct, and breaks
the bit-for-bit reduce** for a reason that reads as a solver artefact rather than as a
transcription error. Both loops are written check-first, and gate 1 is what witnesses it.

**Sizing.** The oracle runs the 147-cell grid on rung 38 and on rung 39 at the four `SHAPES_C`
shapes plus flat; the equilibrium gas re-freezes its mixture once per joint-loop pass, and a
200-pass cell is the cost trap — the probe's 126 rung-38 cells took 55 s on PyPy and 162 s on
CPython.

#### What phase 5 SLICE K MEASURED — SHIPPED, **11 812 / 11 812 bit-exact** vs PyPy

`rust/src/two_spool.rs` + `rust/oracle/dump_two_spool.py` + `rust/tests/two_spool_oracle.rs` +
`rust/tests/rung38.rs` (9 tests) + `rung39.rs` (10). Against CPython the same dump is **46.3 %**
identical — the sharpest value dump in phase 5, because every cell is a joint fixed point over a
turbine bisection over a gas solve.

**The verdicts, one per registered prediction. All six land; two of them corrected something.**

* **P1 — CONFIRMED, on the narrowed claim, and the port states it STRUCTURALLY.** Perturbing
  `eta_lpc` leaves `pi_hpc`, `Tt3` and `Tt25` bit-for-bit unchanged across 4 shape pairs × 3
  throttles × 2 gases, while `eta_hpc` moves `pi_lpc` negatively by more than `1e-5`; the
  turbine efficiencies move both. Beyond the numbers, `hp_eta_loop_closed` is a FREE function
  whose parameters are a gas, geometry, burner constants and HP-side scalars — so "no LP
  efficiency, no LP pressure ratio" is a scope fact a reviewer can check without running
  anything, and `rung38.rs::gate3_structural_the_hp_leaf_takes_no_lp_quantity` fails to compile
  if that signature is ever widened for convenience.
* **P2 — CONFIRMED, including every capped cell.** 11 812 of 11 812 keys bit-identical to PyPy,
  and the 302 pass-count keys are inside that total — so the 200-pass cells reproduce all 200
  passes. This was the slice's stated risk: with the count a knife edge between ~8 and 200, one
  wrong bit anywhere in the cascade would have surfaced as a 25× count difference, not a drift.
* **P3 — CONFIRMED at the noun the instrument reads.** `47` `tau_of` calls per (★) solve, ONE
  distinct value over 1 800 solves on both interpreters — 44 bisection iterations + 2 bracket
  endpoints + 1 final `tau_of`. Registered that way *because* § 5.6's P2 was corrected one slice
  ago for counting the steps and forgetting the endpoints.
* **P4 — CONFIRMED, exactly.** `UNCHOKED` = **23** on the 147-cell rung-38 grid (67 on rung 39's
  larger one), and rung 38/39's six other asserts — the physicality check, the (★) bracket, both
  efficiency secants, the outer turbine-eta loop, `solve_n`'s speed-line bracket and both
  shaft-closure checks — fire **0** times, dumped as explicit zeros under
  `census/r3*/abort_code/9..14` rather than left as an absence.
* **P5 — CONFIRMED.** Adding `l` left `map_oracle` (7 252 keys), `offdesign_oracle` (3 951),
  `rung31`, `rung32` and `rung33` bit-identical. Shipped as its own commit ahead of the module,
  so a failure would have been isolated rather than entangled with 1 000 new lines.
* **P6 — CONFIRMED.** Rust's `t_from_h_c(h_c(250.0))` returns 249.999999999999943 on both
  integral gases and exactly 250.0 on the closed-form one, so the `M0 = 0` column aborts on the
  ram clause for `tpg`/`eq` (14 cells) and gets as far as `score` for `cpg` — the gas-split
  envelope, reproduced. Gated at `rung38.rs::envelope_m0_zero_is_a_round_trip_not_a_physics_boundary`,
  which also pins that the PRESSURE clause is the exact one, so the abort's REASON cannot change
  while the cell still aborts.

**AND THE CPYTHON ARM REFUTED AN ASSERTION SLICE K INHERITED — WHICH IS THE SLICE'S OWN FINDING.**
`offdesign_oracle.rs` states the pass-count instability is *"a property of the EQUILIBRIUM gas's
unmeetable stopping rule"* and asserts every flip is an `eq` cell. Copied forward, that assertion
FAILED here: of 81 flips, **13 are on the THERMALLY-PERFECT gas**. The common factor is not the
composition, it is the **ROUTE TO A PROPERTY** — `tpg` and `eq` both reach `cp` through an
integral and a root-find, so last-bit arithmetic can tip an unmeetable stopping rule either way;
the calorically-perfect gas is closed-form and flips **nowhere**, on either grid. The gate now
asserts that (no `cpg` flip, ever), and slice I's comment is corrected in place with its
assertion left standing — the rung-28 shape, applied to the port's own instruments.

**TWO MORE INSTRUMENT DEFECTS, BOTH THE SHAPE SLICE J FOUND, AND NEITHER TOUCHING A NUMBER.**

1. `bisect/n_solves_swept` was classed `discrete` because its key looked like a census number. It
   is `2 × cascade_calls` summed over the sweep, so it inherits the joint loop's
   interpreter-dependence exactly, and the CPython arm failed on a key that was behaving
   correctly. **A quantity's class comes from what PRODUCES it, never from what it is called.**
2. **A KEY NAMED FOR A DEFECT IT CANNOT DETECT.** The gate's own module note claimed
   `hp_passes_max`/`lp_passes_max` witness the check-first loop shape — *"a `do`-while would make
   them ≥ 1 where the flat cells measure 0"*. They would not: they are **maxima over the whole
   grid**, set by the SHAPED cells at 4, so flipping every flat cell from 0 to 1 leaves them
   unmoved. The asymmetry was visible in the dump itself — the outer turbine loop got a `_min`
   *and* a `_max`, the eta loops only a `_max`. Both minima are now dumped and gated (`0` today);
   the maxima are kept for what they DO witness, that `ETA_MAX = 80` is nowhere near approached.
   This is `docs`' own *"a documented gate that doesn't exist"* entry, one slice later, and the
   code was never in doubt — `rung39.rs::gate1_reduce_flat_maps_is_rung38` catches a `do`-while
   bit-for-bit. **The instrument was.**

Also removed post-ship: a `Matched::SingleMap` variant that nothing constructs (rung 39 dispatches
through its own enum) — slice J's deleted-`debug_assert_eq!` class; and one `tt.sqrt()` in the new
rung-38 structural gate where every shipped site spells it `powp(tt, 0.5)`. Harmless where it sat,
and exactly the line slice L would have copied.

---

### 5.8 SLICE L (rungs 41 + 42) — **COMPLETE: steps 1–4 SHIPPED**; pre-registered (§ 5.8.1), nine predictions, ALL SETTLED

Handoff state as of commit `ff784ed`. **448 Rust tests green, all three existing oracles
bit-identical.** Read this before resuming — the probes are done and their numbers are here, so
do NOT re-measure them.

**What is DONE — step 1, the fallible twins.** Rung 41's `surge_margin_schedule`,
`running_line_map` and `flow_coefficient_turn` wrap `self.match` in `except AssertionError` and
**skip** the failing point, so every assert reachable inside `match` sits in a caught scope that
Rust cannot express with a panic. The crate rule (`gas.rs::Abort`) is *an assert becomes an
`Abort` iff it is reachable from inside the march's residual* — here, from inside the caught
scope. Which asserts those are was **measured** on a 147-cell grid (3 gases × 7 `M0` × 7 `Tt4`),
swept at **two bleed settings** (`b` = 0.0 and 0.10) for 294 runs in all, recording the innermost
raising frame. *(The parenthesis originally read "3 gases × 2 bleeds × 7 × 7 = 147", which
multiplies to 294; the grid is 147 cells PER bleed setting, which is also how the `b`-dependent
row below reads — corrected when § 5.8.1 went to pin these as gate bars.)*

| site | firings | verdict |
|------|---------|---------|
| `_score` efficiency cascade | 27 | → `Abort` |
| `gas._solve`, via `t_from_pr_t` **only** | 36 | → `Abort` |
| `_equil_solve` Newton | 14 | → `Abort` |
| `_solve_f` burner | 3 | → `Abort` |
| nozzle back-pressure | 6 | → `Abort` |
| `freestream` ram-must-not-cool | 28 | → `Abort` |
| rung-39 UNCHOKED scope guard | 23 (25 at `b`=0.10) | → `Abort` |
| everything else, incl. `t_from_h`→`solve` | **0** | keeps `assert!` |

The zero-firing rule holds: *a fallible path with no reachable failure is a gate that measures
nothing.* `t_from_h` reaches the same `solve` and fires **0** times, so the adjudication is per
**call site**, not per function. This **overturns § 5.4 (i)'s** "`solve` stays a panic" — by
measurement, which is the only thing allowed to overturn it.

Shape shipped: an **additive twin** — `try_x` holds the body and returns `Result<_, Abort>`, `x`
delegates and panics with the identical message. No already-gated caller changed and no shipped
bit moved. The hook table's entry is now `try_match_point`, so rung 42's override and rung 41's
caller meet at **one** fallible dispatch point. Rung 38's `match_point` stays infallible on
purpose (its scope guard is not in the caught chain).

**What the probes ALSO settled — reuse, do not re-run:**

- **The dispatch is live and gateable.** In Python, `bleed_trade → surge_margin → self.match →
  rung 42's override`, and the surge margins **move with bleed**. No value key can fake it, so
  slice L owes a `gate_the_dispatch_is_live`.
- **Rung 42's UNCHOKED count rises 23 → 25 when the valve opens.** That is rung 42's own gate 6,
  *opening the valve shrinks the choked envelope*, expressed as a census count — a count that
  measures physics, not plumbing.
- **Rung 41's own guards never fire** on the swept grid (floors to 0.65, four map shapes):
  neither `already at/over surge` nor `_pi_c_spool`'s `tau <= 1`. Rung 42's own asserts (secant,
  turbine loop, unphysical) likewise 0. Its constructor guard `0 <= bleed < 0.5` **does** fire.
  Per the zero rule, all of those stay `assert!`.

**What REMAINS, in order** (step 1 of this list is **DONE** — it is § 5.8.1):

1. ~~**Write the pre-registration**~~ — **DONE, § 5.8.1**, nine predictions. It gained two
   things the list above did not name: a P5-style *"the refactor moves no shipped bit"* extended
   to cover **`phi_surge`'s arrival on `ComponentMap`** (§ 5.7 (a) deferred that field to this
   slice, and adding it is a change to already-gated code), and a corrected **deferral** list —
   gate 7b reaches `SpoolTransient` (**single**-spool, rungs 34/36), not `TwoSpoolTransient`, and
   the cycle test **splits** rather than defers.
2. ~~**Port rung 41**~~ — **DONE**, commits `13e02e6` (the `phi_surge` field, alone) and
   `af0d3cb` (the six methods, into `two_spool.rs`). 448 Rust tests green; smoke-checked
   **bit-for-bit against PyPy over 58 rows** before committing — both branches of the turn, both
   spools, both flight Machs, flat *and* shaped maps, all three gases for the schedules. It
   produced **two corrections to § 5.8.1**, both in § 5.8.2 below: P4 is now **closed by
   construction** rather than gated, and the deferral table's `is_flat` line is **withdrawn**.
3. ~~**Port rung 42**~~ — **DONE**, in a new `bleed.rs`: `TwoSpoolBleedMatcher` with its own
   `R42` hook-table entry, `lp_eta_loop_bleed`, `try_cascade_bleed`, a **separate** rebuild body
   *and a separate result assembly*, and `bleed_trade`. 448 Rust tests green with every oracle
   bit-identical; smoke-checked **bit-for-bit against PyPy over 314 rows**, at `b > 0`, with the
   check's own sensitivity measured. **P1, P2 and P7 are settled early** (§ 5.8.3). The planned
   second commit for "the booking fields on the result" turned out **VACUOUS** — § 5.8.3 (a).
4. ~~**The oracle and the suites**~~ — **DONE**, § 5.8.4. `dump_slice_l.py` (25 458 keys) +
   `slice_l_oracle.rs`, and `rung41.rs` / `rung42.rs`. The step-4 line said "the 9 + 10 Python
   gates"; the files have **12 + 12** test functions under **8 + 8** documented headings, and the
   roster is now DATA in each suite. The narrowing was fixed as required (§ 5.8.4 (b)), and the
   `(1-b)` detector re-measured on the dump — **1.00 % of keys, and the `value` bar is blind to
   all but one of them** (§ 5.8.4 (e)). The CPython arm then caught that bar being **copied from
   slice K rather than measured here**, and the 34 keys it failed on split into a flat extremum's
   LOCATION and a pass-count flip's price — § 5.8.4 (h).

### 5.8.4 SLICE L step 4 — the oracle, the suites, and a claim the SHIPPED SOURCE carried

**481 Rust tests green (479 before `slice_l_oracle.rs`'s two arms); the PyPy arm is
25 458 / 25 458 bit-identical and the CPython arm passes at measured bars.** Six of the nine
predictions are settled here (P3, P4, P5, P6, P8, P9), joining step 3's P1/P2/P7. Two of them
moved, and one correction lands in shipped code rather than in a plan.

**(a) THE PLAN'S GATE COUNT WAS WRONG IN BOTH DIRECTIONS, AND THE ROSTER IS NOW DATA.** § 5.8's
step-4 line said "the 9 + 10 Python gates". `tests/test_rung41.py` has **12** `def test_`
functions under **8** documented gate headings; `tests/test_rung42.py` likewise has **12** under
**8**. Neither file reconciles with 9 or with 10, and the numbers are not two views of one thing —
the headings group functions unevenly (gate 5 is two functions, gate 1 is three). Enumerated
before any Rust was written, per `docs`' *slice K* entry (the phase table's scope list had never
been enumerated: it dropped one rung and double-counted another) and *an oracle cannot see a
MISSING GATE* (grep the source's gates and diff; never port from a header).

Settled: **10 of rung 41's 12 port** — gate 1b defers whole to phase 6 (`TwoSpoolTransient`),
gate 7b defers whole (`SpoolTransient`, **single**-spool), and gate 1c SPLITS, its bit-for-bit
cycle halves porting now. **All 12 of rung 42's port**, because that file's cycle gate reaches
`build_turbojet` and the bleed matcher only — no transient anywhere. Both rosters are asserted as
arrays (`slice_l_deferrals`, `slice_l_roster`) so the count is auditable rather than prose.

**(b) THE CENSUS WAS FIRST MEASURED ON THE WRONG MAPS, AND IT LOOKED LIKE A REFUTATION.** The
first dump swept rung 42's 147-cell grid on **FLAT** maps and read **68 / 68 / 68 / 67** matched
with UNCHOKED flat at **23 / 23 / 23 / 23**. Against § 5.8.1 (v)'s registered 67 / 67 / 66 / 65
and 23 / 23 / 24 / 25 that reads as P6 refuted — *the UNCHOKED column does not rise*, which is
rung 42's own gate 6 failing.

It was the grid. Every pre-registered number was probed on the **`mixed` pair**
`(LP_SHAPED, HP_SHAPED)`, and P7's census half is that `b = 0` reproduces slice K's rung-39 row —
which is slice K's `mixed` row, one of its two shapes getting the full `M0` grid. Re-swept there,
the dump reproduces the table **to the cell**: 67 / 67 / 66 / 65 matched, UNCHOKED **23 / 23 / 24
/ 25**, and the `b = 0` row's `efficiency cascade` count is **13** — slice K's own one-cell
difference from rung 38. This is § 5.7 (e)'s rule caught in the act rather than quoted: *a bar is
measured on the grid it will be gated on, never read off a neighbouring one.* The flat numbers
are perfectly correct and answer a different question, which is exactly what makes the trap
survivable-looking.

**(c) A CLAIM IN THE SHIPPED SOURCE WAS FALSE, AND THE GATE WRITTEN FOR IT PASSED THE DEFECT.**
`two_spool.rs` asserted, in the golden section's own comment, that the CHECK-FIRST shape is
load-bearing — "a `do`-while converges to the same place and makes the refinement count 34 instead
of 33". P5's gate was written on that claim. **Injected, the `do`-while makes 33.** Same
`tt4_star`, same everything: 479 tests and all four value oracles pass unchanged.

The reason is structural, and it is the mirror of the place the distinction IS live. The golden
section's bracket is **always `2 * coarse = 20` wide on entry**, so the stopping rule cannot be met
before the first pass — and meeting it on entry is the ONLY thing that separates check-first from
`do`-while. Rung 39's efficiency loops differ precisely because a FLAT map meets their residual on
entry, which is what `rung39.rs::the_efficiency_loops_test_before_they_step` gates. Carrying the
claim across from one loop to the other was the error.

Both the source comment and the test's doc are corrected, and P5 is re-registered against what it
demonstrably catches, measured the same way: a changed **stopping rule** (`1e-6` gives 37) or
bracket width. It is a gate on the scan PARAMETERS reaching the loop, not on the loop's shape. The
count 33 itself stands, and stays out of the value dump — Python cannot instrument the shipped
body's two phases from outside, so its arm would be a transcription and the comparison
self-confirming (rung 83's *identity round-trip*).

**(d) THE DISPATCH GATE IS THREE TESTS, AND IT IS THE ONLY THING THAT CATCHES ITS DEFECT.** § 5.8
owed a `gate_the_dispatch_is_live`. Written first as one test with three legs, then split, because
pointing `R42`'s hook at `R39.try_match_point` fires the FIRST leg and the other two never run —
so "all three schedule methods witness the dispatch" would have been a claim the calibration never
touched. Split, each fails on its own.

The wider reading is the one worth keeping: with rung 42's physics replaced wholesale by rung 39's,
**14 of `rung42.rs`'s 17 tests still pass**. Every value gate reaches `try_match_bleed` directly,
because that is the entry point carrying the booking; only rung 41's methods go through the hook.
A suite can port a rung's entire physics faithfully and still not witness the one thing the port
added.

**(e) THE DETECTOR WAS MEASURED, AND THE `value` BAR IS BLIND TO THE DEFECT IT WAS SIZED FOR.**
Step 3 measured the `(1-b)` mis-association at ~2 % of its 314 smoke rows. Re-measured on the
25 458-key dump: **254 keys move — 1.00 % — and exactly ONE exceeds the `value` bar. It is not a
value.** It is an `n_pass`, the joint loop taking 12 passes instead of 11. The worst VALUE
deviation over the whole sweep is **2.05e-9**, inside the 1e-8 relative bar.

So the PyPy arm's bit-equality is not belt-and-braces, it is the detector: toleranced, this gate
would catch the defect on 1 key of 25 458 instead of 254 — and that one key only exists because
the pass count is dumped at all. The worst-moved value sits at `M0 = 1.60` on the thermally-perfect
gas, which is where § 5.8.3 (h) said this class surfaces, so the deliberately-edge-placed cells
earned their place.

**(f) P4 COST A SECOND TOUCH OF GATED CODE, TAKEN DELIBERATELY.** § 5.8.2 warned that adding an
instrument in step 4 means touching gated code twice, and shipped `refine_calls` in step 2 to avoid
it. P4 needed one anyway: a memo-key recorder in `counters`, on `od_index`'s MISS branch. The
alternative was transcribing the golden-section abscissa arithmetic into the gate, which gates the
copy; and proxying the sequence through `tt4_star` is a gate that might not exist, since two branch
sequences can close on the same midpoint. Its own commit, P2's bill paid: 448 green, four oracles
bit-identical. Measured sequences: **124 keys (MIN) / 90 (RAIL)** on the rung-39 path, **121 / 88**
on a rung-42 core at `b = 0.10` — different objects, as the branch flip predicts.

**(g) ONE THING § 5.8.3 PREDICTED THAT THE DUMP DOES NOT WITNESS.** Step 3 reasoned that
`flow_coefficient_turn`'s `MIN`/`RAIL` branch **can flip under bleed**, since bleed moves `phi` and
therefore the argmin index. Read cell by cell rather than as the pooled `10 MIN / 6 RAIL`, it does
not: `Hp` is MIN in all 8 rung-42-core turns, `Lp` is RAIL in all 6 at `M0 = 0.85` and MIN in both
at `M0 = 1.60`, and `kind` is **identical at `b = 0.00` and `b = 0.10` in every one of the eight
(gas, spool, flight) cells**. The branch driver on this grid is the FLIGHT MACH, not the valve.
The concern was still worth acting on — it is why `kind` is dumped per cell, so the claim is gated
rather than assumed — but a pooled count invites reading a bleed-driven flip into a number that
contains none (`docs`' *guessed census bars*). Recorded so no later slice inherits it as
established.

Two dead axes were also turned into gates rather than left as duplicate work. `p3`'s floor sweep
ran `flow_coefficient_turn` at floors 0.0 and 0.55, which that method never reads — 8 duplicate
runs witnessing nothing. § 5.8.1 (viii) had MEASURED the invariance (bit-identical `Tt4_star` at
both floors), so it is now asserted: the same content as gate 1, read through a different method.

**What the rest of the dump settled.** P3: **0 refinement aborts** anywhere on the dump grid,
including the rung-42 cores § 5.8.1's probes never swept, and `ended_on_abort` is **54 of 54** — so
`Tt4_lo = 350` is dead, gated absolutely in `rung41.rs` (`band_lo > Tt4_lo + coarse`) since no
comparison can see a parameter both sides read. P9: the discriminant + declared sentinel
(`NULL = -1.0`, impossible for all eight nullable columns) with the branch counts per BLOCK —
**16 of 19 `lp` cases RAIL** on the flat set, § 5.8.1 (viii)'s number to the cell. Pooling the two
blocks' counters was the first draft's error and reported 32/22 over 54 runs, a number no
prediction is written against.

**(h) THE `value` BAR WAS COPIED FROM SLICE K, AND THE CPYTHON ARM CAUGHT IT.** 1e-8 relative was
taken over wholesale; on this dump it failed on **34 of 23 772 value keys**. The 34 are two
disjoint populations with nothing between them, and the accounting closes exactly — which is the
reason to trust the split rather than widen one number:

*(A) The LOCATION of a flat extremum — 28 keys = 7 turn cells × 4 fields.* Out of the SAME golden
section, in the SAME cell: `phi_star` (what the turn is worth) agrees to **4.07e-11**, `Tt4_star`
(where it is) to **7.39e-6**, `far` (read at that location) to **1.58e-5**. At an interior maximum
the objective's slope is zero, so noise in the objective — the inner matcher's own convergence,
not machine epsilon — buys a first-order move in the abscissa and none in the ordinate. The
bracket is driven to 1e-5 K by a stopping rule the objective's noise floor cannot support, so the
last refinements resolve nothing and the two interpreters settle anywhere in the ε-optimal set.
**This INVERTS `docs`' *shape keys* entry** (*a peak's VALUE drifts and its LOCATION does not*):
that was an argmax over a discrete GRID, which quantises the answer and snaps both interpreters to
one node. This one is over a CONTINUUM. Same word, opposite conditioning — the grid was doing the
work. NOT claimed: √(4.07e-11) = 6.4e-6 lands within 16 % of the worst `Tt4_star` and looks like
the √ε law, but per CELL the ratio runs 0.07–6.17, a spread of ~90. Two maxima meeting, not a law
(rung 66's *check where an extremum sits before quoting it*). Only the per-cell ORDERING is
asserted. These four fields are now their own quantity class at a **measured** 1e-4, and the file
says out loud that at that bar they are not meaningfully gated against CPython — the PyPy
bit-equality arm is their gate.

*(B) A pass-count flip costs ONE DECADE — 6 keys.* Value keys in cells whose joint loop ran a
different number of passes reach **1.55e-8**; in cells that ran the same count they stop at
**1.03e-9**. The first version held the flipped half to 1e-8 and fired, proving only that the bar
had been inherited. Both populations are now measured and BOTH bars asserted, because the content
is the separation: if the unflipped half ever reaches the flipped half's bar, the flip is not what
drives the drift and the section measures nothing.

**CPG is clean on both.** Zero calorically-perfect turn cells moved a location; zero
calorically-perfect pass counts flipped. Slice K established that invariant on the loop count
alone — a second, unrelated phenomenon now obeys it, for the same reason: a closed form has no
root-find beneath it for last-bit arithmetic to tip. Both halves are asserted, so a future slice
that breaks either learns it here.

### 5.8.3 SLICE L step 3 — the rung-42 port, and a PLANNED COMMIT that turned out vacuous

Rung 42 lives in a new `bleed.rs`, as § 5.8.1's module decision said it would. Three of the nine
predictions are now settled, and the step produced one correction to its own plan.

**(a) THE SECOND COMMIT IS VACUOUS, BECAUSE THE ABSENCE IS A TYPE.** § 5.8's step-3 line said the
bleed booking fields "go on the result as their own commit" — which presumes they land on the
SHARED `TwoSpoolMapResult` and are therefore a change to gated code. They do not. Python never
*constructs* a `TwoSpoolBleedResult` at `b = 0`; it returns rung 39's object, whose four booking
attributes do not exist, and `bleed_trade` reads that **absence** through
`getattr(od, "st_inlet", od.performance.specific_thrust)`. So the dataclass's `st_inlet = 0.0`
default is **unreachable**, and a port that always built the struct would write `0.0` into the
`b = 0` row where Python writes the core specific thrust. An `Option<BleedBooking>` on the new
type makes that unwritable and touches no gated type at all — so there is nothing to split off.
This is § 5.8.1's **P9 shape applied a second time in the same slice**, to a missing OBJECT rather
than a missing value.

**(b) THE `b = 0` ROW IS VACUOUS FOR THE THING IT LOOKS LIKE IT TESTS — REGISTERED, NOT DISCOVERED
AFTERWARDS.** At `b = 0`, `st_inlet == specific_thrust` and `mdot_core == mdot_air` NUMERICALLY.
So the row a value comparison would naturally reach for cannot discriminate a wrongly-built
booking, a swapped `st_inlet`/`tsfc_inlet` pair, or a defaulted field — every spelling agrees
there. The smoke sweep is therefore at `b > 0` only, and the `None`-ness is gated where it can be
seen: as a type assertion in Rust, beside P7's bit-equality. Symmetric with § 5.8.2 (d)'s two
notes: a check that passes under both spellings is not a check.

**(c) THE FIRST OVERRIDE NEEDED THE DESCENDANT'S STATE ON THE SHARED CORE.** A `fn`-pointer table
hands its hook `&TwoSpoolMapCore`, so `bleed` has to be a field there rather than on the rung-42
leaf — which is what Python's `self` is anyway: inside rung 39's methods it IS the rung-42 object
carrying `self.bleed`. Named in the field's doc so a reader does not infer rung 39 has a valve,
and explicitly **not** the `l` mistake of slice J → K: the consumer ships in the same slice.

**(d) THE HOOK ROUTING IS ASYMMETRIC, AND BOTH DIRECTIONS ARE LOAD-BEARING.** `_hp_eta_loop` is
called **VERBATIM** by rung 42's cascade, so the Rust must reach it THROUGH the table — naming
`r39_hp_eta_loop` would compile and silently freeze the slot rung 55 overrides in phase 7.
Conversely `_lp_eta_loop_bleed` is a **NEW METHOD NAME in Python, not an override**, so `R42`'s
`lp_eta_loop` slot stays rung 39's function; putting the bleed body there would stop rungs 39/41's
suites witnessing the unchanged one, which is the very thing the source's docstring preserves.

**(e) THE SCOPE GUARD IS WHERE COPYING THE SOURCE GIVES THE WRONG RUST.** Python spells rung 42's
UNCHOKED guard as an `assert`, exactly as it spells rung 39's — and rung 39's is already an
`Abort` here, because rung 41's schedule methods SKIP such a point. Rung 42's `match` is reached
through the SAME hook from the SAME methods, so a panic would kill a `surge_margin_schedule` on a
bleed matcher where Python skips. Ported as `Err`, and the smoke grid reaches it: **28 of 144
cells**. Every other rung-42 guard stays an `assert!` on the zero-firing rule — including the
constructor's range check, which is the one measured to FIRE.

**(f) THE COUNTERS WIDENED TO `pub(crate)`, DELIBERATELY.** Rung 42's cascade feeds rung 38/39's
accumulators, because `cascade_calls` per cell is how the oracle sees a joint loop **cap at 200**
(`r38/n_pass`, `r39/n_pass`), and § 5.8.1 (v)'s rung-42 census counts exactly that per bleed
level. `reset()` is per cell, so one cell runs one matcher and the rungs never mix inside a
reading. Shipped **beside the code**, per § 5.8.2's rule: adding an instrument in step 4 means
touching gated code twice.

**What the smoke check compared, and why it is that wide.** § 5.8.2 (c)'s lesson applied UP FRONT
rather than after a clean pass: **every** field of `TwoSpoolBleedResult`, of its rung-39 base and
of its `Performance`, plus **all 8 stations × 4 fields** — 79 columns per matched row. `mdot` is
in there because it is the ONLY place the extraction is visible at all (nothing downstream reads
it: `try_score` never touches mass flow), and the same-typed adjacent pairs `phi_lp`/`phi_hp`,
`n_lp`/`n_hp`, `pi_lpc`/`pi_hpc`, `st_inlet`/`tsfc_inlet` are exactly the transpositions no other
number would reveal. **314 rows, 0 mismatches** — 116 matched cells over 3 gases × 2 map shapes ×
2 flight Machs × 3 bleeds × 4 throttles, 28 aborts, 24 `bleed_trade` rows, 6 valve-restored
checks, and the schedule block below.

**(g) `bleed_trade` WITNESSES ONE OF THE THREE SCHEDULE METHODS, AND THE FIRST PASS STOPPED
THERE.** The slice's claim — written into `two_spool.rs`'s own header — is that `surge_margin`,
`running_line_map` AND `flow_coefficient_turn` all reach rung 42's body through the hook. Only the
first is on `bleed_trade`'s path, and the other two are the ones with output nothing else covers:
`flow_coefficient_turn` returns the type carrying P9's nullable columns, and its `MIN`/`RAIL`
branch can FLIP under bleed, because bleed moves `phi` and therefore the argmin index. Widened to
all three on a rung-42 core, at **both** bleed levels: **both branches are exercised at `b > 0`**
(`Hp` `MIN` / `Lp` `RAIL` on all three gases, with the four `NULL` columns compared as such), and
the skip census is the physics rather than the plumbing — **10 of 13 survive at `b = 0` on every
gas, 9 of 13 on TPG and equilibrium at `b = 0.10`**, CPG holding at 10. That is rung 42's gate 6,
*opening the valve shrinks the choked envelope*, read through rung 41's skip, reproduced
bit-identically **including the count**. The `b = 0` figure also re-confirms § 5.8.1 (vi) on a
shape pair it did not sweep.

**(h) THE CHECK'S SENSITIVITY WAS MEASURED, NOT ASSUMED — AND IT IS 2 %.** `docs`' *slice J*
entry is that a 7 252-key bit-exact oracle passed a deliberately mis-spelled square, so "0
mismatches" is an observation until the detector is calibrated. Flipping ONE of the three `(1-b)`
associations (`eta_m * (1+f) * (1-b)` for `eta_m * (1-b) * (1+f)`, algebraically identical and a
different double) moves **7 of the 314 rows**: 5 matched cells, 1 `running_line_map` row and 1
`surge_margin_schedule` row, all at the sweep's edges (`M0 = 1.60`, or the equilibrium gas at
1300 K). Reverted, and the diff returns to zero. So the defect class IS caught — but by ~2 % of
the grid, which is the argument for the sweep's width: a handful of cells would have passed it.

**Three predictions settled early.**

* **P1 — CONFIRMED, on both sides.** `bleed_trade → surge_margin → self.match → R42` reproduces
  § 5.8.1 (vii)'s numbers to the digit (`SM_L` 0.1089 → 0.1248 → 0.1430, `SM_H` 0.5237 → 0.5338 →
  0.5444), and the Rust carries its refutation condition as its own test: margins invariant in `b`
  would mean the hook slot never got `R42`, which compiles and returns numbers.
* **P2 — HELD.** Three additive changes to gated code (the `bleed` field, a `try_freestream_at`
  accessor, the counter visibility) and **448 tests green with every oracle bit-identical**.
* **P7 — HELD, as a VALUE.** `b = 0` against a `TwoSpoolMapMatcher` on the same design and maps:
  bit-equal over all 8 stations × 4 fields and 13 further quantities, on 18 cells, with the
  booking `None`. Spelled `R39.try_match_point` and not `core.try_match_point` — this is Python's
  `super().match(...)`, a NON-virtual call, and routing it back through the table would recurse.

### 5.8.2 SLICE L step 2 — what the PORT found that the PRE-REGISTRATION could not

Two of § 5.8.1's nine predictions changed shape while being implemented. Neither is a result being
retro-fitted — both are the *instrument* turning out to be settleable by argument where the
pre-registration could only measure.

**(a) P4 IS CLOSED BY CONSTRUCTION, NOT MERELY GATED.** § 5.8.1 (ii) measured Python's `round(x, 6)`
and the naive `(x * 1e6).round() / 1e6` agreeing on all 4 216 live keys and on a 600 000-point
synthetic sweep, and — correctly — refused to call that a proof: at the estimated ~2e-7 divergence
rate the sweep expects ≈0.1 events, so a measured zero is consistent with a real divergence it
missed. That is why the registered instrument became the **key sequence** rather than the spelling.

Writing the port settled it outright, in two steps.

* **Format-and-parse IS Python's algorithm.** CPython's `double_round` calls `_Py_dg_dtoa(x, mode 3,
  ndigits)` for the correctly-rounded, half-to-**even** decimal string and `_Py_dg_strtod` to convert
  back; PyPy's `rfloat.round_double` does the same two steps. Rust's `{:.6}` is exact and rounds
  half-to-even and `str::parse::<f64>` is correctly rounded, so `round6` is not an approximation of
  `round(x, 6)` — it is the same pair of operations.
* **The naive spelling is DEMONSTRABLY WRONG on reachable inputs.** An exact tie at the 6th decimal
  needs `x = (2j+1)/(2·10^6)`; for a dyadic `x = m/2^k` that forces `k = 7` with `m` odd, so the ties
  are exactly the **odd multiples of `1/128`** — representable, and inside the [350, 1500] band the
  coarse scan sweeps. Verified on `x = 350.0078125`: PyPy and `round6` both give `350.007812`
  (half to even) where the naive spelling gives `350.007813`.

So the divergence class is closed in the library, and **P4's key dump becomes a regression guard
rather than the primary defence.** The general lesson is the port's own *measure before registering*
rule read from the other end: a measured zero bounds a *rate*, and when the failure class is small
enough to ENUMERATE, enumerating it beats sweeping for it. § 5.8.1 (ii) sized the rate correctly and
still could not see that the tie set was `{odd/128}`.

**(b) P8's `is_flat` LINE IS WITHDRAWN — the deferral table named a gate that should not exist.**
§ 5.8.1's table said rung 41 gate 1b's closing `ComponentMap.flat().with_phi_surge(0.6).is_flat()`
**ports now**, since it needs no transient. It needs no transient and it should still not be
written, for two reasons found while writing the port:

1. **The predicate would be Python's minus a term.** Python's `is_flat` reads `vsv == 0.0`, and
   `vsv` is rung 53's — slice M's. A Rust `is_flat` without that conjunct is inert exactly as far as
   today's sweep and no further: **the `l` mistake of slice J → K, repeated on a predicate instead of
   a field** (§ 5.7 (a)).
2. **There is no flat-reduce BRANCH for it to guard.** Python's flatness is a claim about which
   fields one predicate reads; in Rust the reduce is STRUCTURAL — `psi` returns `1.0`,
   `eta_c_at` returns its base — so `is_flat()` could return `true` while the reduce is broken, and
   vice versa. The predicate and the property are not the same object here.

P8's *content* is therefore gated where it lives, as a **value**: a flat map carrying a floor and one
without produce a bit-identical matched point (rung 41 gate 1's actual body). `is_flat` lands with
`vsv`, in slice M. Recorded in `map.rs`'s field-subset note, which is where a reader looking for the
field will be.

**(c) THE SMOKE CHECK WAS WIDENED AFTER IT PASSED, AND THAT IS THE POINT.** The first pass printed
**7 of 11** `FlowTurn` fields, **4 of 13** `SurgeMargin` fields and **0 of 9** `RunningLinePoint`
fields — 20 uncompared `f64`s behind a clean diff. The failure mode that leaves open is the **SWAP**:
`x_lp`/`x_hp`, `n_lp`/`n_hp`, `pi_lpc`/`pi_hpc` are same-typed ADJACENT entries in a struct literal,
and `running_line_map`'s output feeds nothing downstream, so no other number in the port would
reveal a transposition. `band` has the same property — written once, read by nobody. Re-run on every
field of all three types: **88 rows, 0 mismatches**, including the four-field `NULL` sentinel on the
`RAIL` rows. The lesson is `docs`' *oracle cannot see a missing gate* entry applied to a hand-written
check: **a bit-exact agreement bounds the columns PRINTED, not the fields RETURNED**, and the fields
most likely to be wrong are exactly the ones nothing downstream reads.

**(d) TWO VACUITY NOTES, REGISTERED SYMMETRICALLY.** § 5.8.1 (iv) already recorded that the argmin's
ties are unreachable (0 across 118 runs), so the first-wins rule is written to match the source and
cannot be discriminated by any gate here. The same is true of `binding`: **every one of the 30 swept
schedule rows came back `lp`**, so neither the `hp` arm nor the `<=` tie rule is exercised. That is
not a grid deficiency — LP-always-binding IS rung 41's headline — so it is recorded beside the field
rather than left for a reader to assume the branch was tested.

**What step 2 confirmed unchanged.** P5's 33 refinement calls reproduced on every `MIN` run and 0 on
every `RAIL` one, with the counter shipped **beside the code** rather than in step 4's gate — adding
it later would mean touching gated code twice. **But the Python arm of P5 is permanently the
`probe_l3` transcription**, because Python cannot instrument the shipped body's two phases apart
from outside; the load-bearing leg is the arithmetic (`ceil(ln(1e-5/20)/ln(0.618…)) + 2 = 33`), which
is interpreter-independent. **So step 4's `dump_slice_l.py` must NOT emit a literal 33** — that would
make the oracle gate self-confirming, rung 83's *identity round-trip sold as verification* shape.
Gate the 33 in `rung41.rs` against the arithmetic and keep it out of the value dump. P9 is carried by
the **type**: `FlowTurn` uses `Option` for the four fields Python's `RAIL` dict nulls or omits, so
writing `0.0` where Python writes `None` is unwritable rather than merely gated. The § 5.8.1 (vi)
skip census reproduced exactly (10 of 13 on CPG/TPG/eq at these shapes).

**Two divergences recorded rather than fixed**, both named in the source:

* On **unarmed** maps Python's `surge_margin_schedule` catches the `phi_surge > 0` assert at every
  point and returns `[]`; the Rust panics on the first. Reachable by CONSTRUCTION, not by throttle,
  so no swept grid produces it — and adding a `Result` would re-open an adjudication a measurement
  already settled (§ 5.8's zero-firing verdict).
* `Spool` as an enum, and the six methods living on `TwoSpoolMapCore` rather than on the matcher
  enum, make two Python asserts unreachable **by type**. Note the second changes the *exception*: a
  rung-41 method on a `lp_disabled` Python matcher raises `AttributeError` from the rung-32 delegate,
  not `AssertionError`, so it was never catchable by the schedule methods anyway.

Then **slices M / N / O**: rungs 53–56 and 61, the airflow levers.

**Probe scripts** live in `M:\claud_projects\temp\rust-slice-l\` (`probe_slice_l.py`,
`probe_sites.py`, `probe_solve_chain.py`, § 5.8.1's `probe_l2.py` / `probe_l3.py`, and the two
smoke checks — step 2's `smoke_ref.py` and step 3's `smoke42.py` with its Rust half `smoke42.rs`,
kept there rather than in `rust/tests/` because the gates are step 4's).

### 5.8.1 SLICE L — PRE-REGISTERED, and a SECOND probe measured first

**These are not re-measurements of § 5.8.** That handoff settled the seven fallibility sites and
three facts, and they stand. What follows is a set of *different* questions, each of which decides
either a Rust type signature or a gate bar — and the port's own rule is that a bar is measured on
the grid it will be gated on, never read off a neighbouring one (§ 5.7 (e), and § 5.6's P2 before
it). `probe_l2.py`, PyPy.

**(i) THE GOLDEN SECTION'S NAKED `phi` NEVER RAISES — AND THE COARSE SCAN'S GUARD IS 100 % LIVE.**
`flow_coefficient_turn` wraps only the **coarse scan** in `except AssertionError: break`; the
`phi(c)` / `phi(d)` calls inside the golden section are unguarded, so a raise there propagates out
of the method. Measured on **two grids**, because the first one did not back the bar:

* **`probe_l2.py` — 38 runs, 726 refinement calls, 0 aborts, coarse guard 38 of 38.** The case set
  is gate 5's own (7 design/efficiency variants + 4 `gamma_c` + `gamma_t` + `cp_t` + 3 `hPR` +
  `M0` = 1.60 = 17) **plus 2 thermally-perfect cases gate 5 does not carry**, × both spools. Note
  gate 5 only ever calls the `"hp"` spool, so **the `lp` column here is wider than any shipped
  gate** — that is deliberate, and it is why the count's authority is this enumeration and not the
  gate's.
* **`probe_l3.py` — 80 runs, 1 980 refinement calls, 0 aborts, coarse guard 80 of 80.** Every one
  of those matchers is built with **rung 41's own four shapes plus flat, at floors 0.0 and 0.55,
  at both flight Machs**. This grid exists because the first was measured on **flat** maps only
  (gate 5's own construction, no `map_lp` / `map_hp`), so it backed nothing about the shaped cells
  a dump would naturally sweep — and the gap decides a **type signature**, not merely a bar: an
  infallible `flow_coefficient_turn` meeting a shaped-map refinement abort panics exactly where
  Python skips, and **no value comparison can see it.**

**2 706 refinement calls over 118 runs, 0 aborts, and the coarse guard live on every single one.**
Two consequences, and the second was not anticipated:

* `flow_coefficient_turn` stays **infallible** in Rust. A `Result` there would be a control-flow
  path with no reachable failure — the `Abort` rule's own words, and slice K (g)'s dead-clamp
  precedent. The live fallible call is `try_match_point` *inside the coarse scan*.
* **`Tt4_lo = 350.0` is DEAD on this grid.** The scan always terminates on the abort, never on
  `T > Tt4_lo`, so the runnable band's low end is set by the envelope and not by the parameter.
  Ported as written, recorded as dead so no reader infers it is load-bearing.

**(ii) THE MEMO KEY IS A THROTTLE, NOT A CACHE KEY — AND THE CACHE IS ALL BUT DEAD.**
`cache[key] = self.match(flight, key)` passes the **rounded** value, so `round(float(T), 6)` sets
the throttle actually matched. It moves **values**, not merely cache identity, and that is the
faithfulness trap § 5.8 flagged. Measured: **10 cache hits against 4 206 misses** across the 38
runs — every hit is the closing `od_at(Tstar)` landing on an abscissa already visited. So the
rounding is **not** a caching device, and the port must not be written as though it were.

On the spelling: Python's `round(x, 6)` is correctly-rounded decimal with half-to-**even**; the
naive Rust `(x * 1e6).round() / 1e6` is half-**away-from-zero** *and* carries the multiply's own
error. They agreed on **all 4 216 keys reached** and on a **600 000-point synthetic sweep** of
golden-section abscissae over [350, 1500]. **That zero is not a proof, and this registration says
so:** at these magnitudes a double's ulp is ≈2.3e-13, so the two spellings can differ only when the
exact value lands within ≈1e-7 of a rounding boundary in the 7th decimal — about 2e-7 per call,
i.e. ≈0.1 expected events in 600 000. A measured zero at that rate is consistent with a real
divergence the sweep missed. **The instrument is therefore the KEY SEQUENCE, not the spelling**:
every memo key is dumped as a discrete oracle value (P4), so a divergent rounding shows up as a
changed key rather than as a silently different throttle.

**(iii) THE REFINEMENT COUNT IS ONE NUMBER, AND IT IS PREDICTABLE FROM THE ARITHMETIC.** Every
`MIN` run makes exactly **33** refinement `phi` calls — on flat maps *and* on all four shapes at
both floors (`probe_l3.py`: 60 `MIN` runs × 33 = 1 980 exactly) — 2 initial plus 31 loop passes, and the
arithmetic reproduces it: the bracket is 2 × `coarse` = 20 wide, the stop is `b - a < 1e-5`, and
`ceil(ln(1e-5 / 20) / ln(0.618…)) = 31`. This is the *golden-gate-slice-6* rule reused — a count
that can be **predicted** from the arithmetic is a stronger bar than one merely observed. **The
gate names its instrument**, because the memo makes three counts differ: on a `MIN` run,
refinement `phi` calls = **33**, total `match` calls = **116–140** (case-dependent, since the coarse
scan's length rides the envelope), and loop passes = **32**, of which 31 call `phi` and the last
only tests the stopping rule. § 5.7 (d) corrected a
bar one slice ago for exactly this; the number a gate compares against must be the one its
instrument reads.

**(iv) THE ARGMIN's TIES ARE UNREACHABLE, SO THE RULE IS REGISTERED RATHER THAN MEASURED.**
`min(range(len(vals)), key=…)` returns the **first** minimal index; Rust's `min_by` also returns
the first, but a hand-rolled fold with `<=` flips it. Ties measured: **0** across all 38 runs. The
port writes the first-wins spelling and carries an **explicit vacuity note** — no gate on this grid
can distinguish the two, which is the honest reading of a rule the data cannot test.

**(v) RUNG 42's CENSUS, ON THE DUMP GRID, PER BLEED LEVEL.** Grid: slice K's 147 cells
(3 gases × 7 `M0` × 7 `Tt4`), inherited wholesale; the **bleed axis is new**. Codes are slice I's
table as slice K appended to it — never renumbered.

| `b` | matched | cascade (2) | bracket (3) | Newton (4) | burner `f` (5) | nozzle (6) | ram (7) | **UNCHOKED (8)** | 200-pass cells |
|-----|---------|-------------|-------------|------------|----------------|------------|---------|------------------|----------------|
| 0.00 | 67 | 13 | 18 | 7 | 2 | 3 | 14 | **23** | 10 |
| 0.02 | 67 | 14 | 18 | 7 | 1 | 3 | 14 | **23** | 13 |
| 0.05 | 66 | 14 | 18 | 7 | 1 | 3 | 14 | **24** | 17 |
| 0.10 | 65 | 14 | 18 | 7 | 1 | 3 | 14 | **25** | 12 |

Every row sums to 147. Two readings, and both become gates:

* **`b` = 0 reproduces slice K's rung-39 census EXACTLY** — 67 matched with cascade 13, which is
  slice K's own one-cell difference from rung 38 (`tpg, M0 = 1.60, Tt4 = 600`). The reduce
  dispatch is therefore witnessed as a **census** as well as a value (P7).
* **The UNCHOKED column rises monotonically with `b`.** That is rung 42's own gate 6 — *opening
  the valve shrinks the choked envelope* — expressed as a count that measures **physics**, not
  plumbing. It is the counterexample to `docs`' *guessed census bars* entry: a count bar earns its
  place when the source has a claim it can refute.

**(vi) RUNG 41's SCHEDULES SKIP, AND THE SKIP CENSUS IS GAS-DEPENDENT, NOT SHAPE-DEPENDENT.** Over
4 map shapes × 2 floors × 3 gases on a 13-point throttle grid: **10 of 13 survive** on all 16
CPG/TPG combinations and on 6 of 8 equilibrium ones; **9 of 13** on the equilibrium `steep` pair.
The skip *reasons* split cleanly by gas (CPG: 3 × UNCHOKED · TPG: 1 × UNCHOKED + 2 × bracket ·
eq: 1 × UNCHOKED + 1 × bracket + 1 × Newton) and are **identical across all four shapes and both
floors**. Neither of rung 41's own asserts appears anywhere in that census — § 5.8's zero-firing
verdict, confirmed a second time on a wider grid.

**(vii) THE DISPATCH IS LIVE, AND ITS SIGN IS THE PHYSICS.** `bleed_trade → surge_margin →
self.match → rung 42's override`: both margins rise monotonically with `b` at all three throttles
on both gases (CPG at 1200 K: `SM_L` 0.1089 → 0.1248 → 0.1430; `SM_H` 0.5237 → 0.5338 → 0.5444).
No value key can fake this, because the margins are computed *from* a match only the override
supplies — which is why slice L owes a dispatch gate rather than a value one.

**(viii) THE TWO RETURN SHAPES DIFFER, AND `RAIL` IS THE MAJORITY BRANCH ON THE `lp` SPOOL.**
`flow_coefficient_turn` returns `kind="MIN"` with `pi_star`, `star_form`, `gamma_c` and `far`, or
`kind="RAIL"` with `pi_star=None`, `star_form=None` and **`gamma_c`/`far` absent from the dict
altogether**. Measured: **60 `MIN` / 20 `RAIL` over `probe_l3.py`'s 80 runs**, and on `probe_l2.py`'s
flat-map set **16 of 19 `lp` cases RAIL** — so the null branch is not a corner, it is where the LP
spool normally lives. **A dump of floats is blind to the difference**: a Rust port writing `0.0`
where Python writes `None` produces a column that compares equal and means something else. The
dump therefore carries an explicit **discriminant column** (`kind`, as a code) and a **declared
sentinel** for the three nullable columns, and the gate asserts the branch *count*, not just the
values. This is P8's rule — *a value oracle cannot see a boolean's field set* — applied to a
**missing value** instead of a missing field.

One thing that fell out of `probe_l3.py` for free and belongs to P2: the surge floor changes
**no** turn. Every `(gas, shape, M0, spool)` cell returns bit-identical `Tt4_star` at `floor = 0.0`
and `floor = 0.55`, which is `phi_surge`-is-a-pure-diagnostic measured on the running line rather
than argued from the docstring.

#### The predictions

* **P1 — THE DISPATCH IS LIVE, AND THE GATE IS A SIGN, NOT A VALUE.** Rung 41's `surge_margin`,
  called through rung 42's `bleed_trade`, reaches rung 42's overriding `try_match_point`, so both
  margins move monotonically upward with `b`. *Refuted by:* margins invariant in `b` (the hook
  would then be dispatching to rung 39's body), or a non-monotone column.
* **P2 — THE REFACTOR AND `phi_surge` MOVE NO SHIPPED BIT.** Two changes to already-gated code
  land in this slice: step 1's fallible twins (already shipped and already satisfying this) and
  **`phi_surge`'s arrival on `ComponentMap`**, which § 5.7 (a) deferred here on purpose.
  `map_oracle` (7 252 keys), `offdesign_oracle` (3 951), `two_spool_oracle` (11 812) and the
  `rung31` / `rung32` / `rung33` / `rung38` / `rung39` suites re-run **bit-identical**.
  *Refuted by:* one changed bit ⇒ **revert**, per slice J's own rule for a refactor to gated code.
* **P3 — `flow_coefficient_turn` IS INFALLIBLE, AND ITS COARSE GUARD IS THE LIVE ONE.** The
  refinement makes **0** aborts over **2 706 calls in 118 runs — flat maps AND all four shaped
  pairs at both floors** — while the coarse scan aborts on **118 of 118**, so `Tt4_lo` is never
  what ends the band. The shaped half of that grid is what makes the bar cover the cells the dump
  sweeps; measured on flat maps alone it would have been a bar over unmeasured cells.
  *Refuted by:* any refinement abort on the dump grid (the method would then need a `Result` and
  the Rust would be making a live failure un-catchable), or any run whose scan ends on
  `T > Tt4_lo` instead.
* **P4 — THE MEMO KEY SEQUENCE REPRODUCES BIT-FOR-BIT, DUMPED AS DISCRETE ORACLE KEYS.** Not the
  rounding *spelling* — the sequence of keys the method actually matches at, which is what the
  rounding decides. *Refuted by:* one key differing in a bit, which is the only way a divergent
  `round` can be caught before it becomes a silently different throttle.
* **P5 — THE REFINEMENT COUNT IS EXACTLY 33 ON EVERY `MIN` RUN**, and the gate states that it
  counts **refinement `phi` calls** — not `match` calls, not loop passes. PyPy arm only, per
  slice K's P2 on count stability. *Refuted by:* any second distinct value.
* **P6 — RUNG 42's CENSUS IS (v)'s TABLE, PER BLEED, AS EXPLICIT COUNTS INCLUDING THE ZEROS.**
  UNCHOKED **23 / 23 / 24 / 25** at `b` = 0.00 / 0.02 / 0.05 / 0.10; rung 42's own secant,
  turbine-loop and unphysical asserts dumped as explicit **0**s (§ 5.6's P4 discipline).
  *Refuted by:* any nonzero on the three zero rows, or a UNCHOKED column that does not rise.
* **P7 — `b` = 0 IS RUNG 39, AS BOTH A VALUE AND A CENSUS.** The `bleed == 0.0` branch forwards
  to rung 39's `match` verbatim, so every matched cell is bit-identical to `two_spool_oracle`'s
  and the whole census row equals slice K's rung-39 row (67 matched, cascade 13).
  *Refuted by:* one differing bit, or one cell landing in a different abort class.
* **P8 — THE SURGE FLOOR IS INVISIBLE TO FLATNESS, AND THAT RULE IS GATED DIRECTLY.** Python's
  `is_flat` deliberately excludes `phi_surge` (it is a pure diagnostic and enters no solver), and
  a value oracle **structurally cannot see a boolean's field set** — slice J's "gate the rule
  directly, because the oracle is blind to it" generalised from a mis-spelled square to a
  predicate. Note `is_flat` is **not currently ported** (`map.rs` § *The field subset*: it is
  called by no engine code, only by rung-36/41/53/54 tests), so this prediction is what decides
  whether the slice adds it or gates the flat-reduce dispatch instead. *Refuted by:* a flat map
  carrying a surge floor failing to take the flat-reduce path.
* **P9 — THE `RAIL` BRANCH IS DUMPED AS A DISCRIMINANT PLUS A DECLARED SENTINEL, AND ITS COUNT IS
  THE GATE.** 60 `MIN` / 20 `RAIL` on the shaped grid; 16 of 19 `lp` cases RAIL on the flat one.
  The three nullable columns (`pi_star`, `star_form`, and the two `MIN`-only fields) never carry a
  bare `0.0`. *Refuted by:* a Rust `RAIL` row whose null columns compare equal to a real `MIN`
  value — the failure a float dump cannot see, which is why the branch **count** is asserted
  rather than only the numbers.

#### The deferrals — and one of them is a SPLIT, not a defer

Rung 41's suite is 9 gates; **two defer whole and one splits.** § 5.8's list named the wrong class
for the second, which matters because the two transients are different rungs in the same phase:

| gate | reaches | verdict |
|------|---------|---------|
| 1b `test_reduce_transient_untouched_by_surge_line_bit_for_bit` | `TwoSpoolTransient` (rung 40) | **DEFER → phase 6** — *except its last line* |
| 1b's closing `is_flat` assertion | `ComponentMap` only | **PORTS NOW** — it needs no transient at all, and it is P8's gate |
| 7b `test_rung36_verdict_survives_but_its_mechanism_is_corrected` | `SpoolTransient.surge_margin_channels` (rungs 34/36, **single**-spool) | **DEFER → phase 6.** § 5.8 called this `TwoSpoolTransient`; phase 6 covers both so the verdict held, but the noun was wrong |
| 1c `test_cycle_untouched_rung6_bit_for_bit` | `build_turbojet(…).run()` **and** a `SpoolTransient` construction | **SPLITS** — the bit-for-bit cycle halves port now; only the interleaved transient construction waits |

The IOU is written into `rung41.rs` as a `slice_l_deferrals` test, the
`rung33.rs::slice_j_deferrals` precedent reused for the third time.

#### Module decision, and sizing

**Rung 41 goes INTO `two_spool.rs`** — its six methods are methods on `TwoSpoolMapMatcher`, the
struct that file owns, and the whole point of slice K's hook was that rung 41's callers meet rung
42's override at one dispatch point. **Rung 42 gets a new `bleed.rs`**, carrying
`TwoSpoolBleedMatcher`, `lp_eta_loop_bleed`, `cascade_bleed`, its **separate** rebuild body and
`bleed_trade`, plus its own `R42` hook-table entry. Rung 41 adds ~160 Python lines and rung 42
~230; keeping both would push `two_spool.rs` past 2 100 lines, and slice K split `two_spool.rs`
out of `matcher.rs` at exactly that bar. The dependency stays one-way (`bleed.rs` consumes
`two_spool.rs`, nothing consumes `bleed.rs` until phase 6).

**The one shape decision that decides a reduce gate**, stated before it can be got wrong: rung
42's rebuild is a **deliberate duplication** of rung 39's, not rung 39's with a `b` parameter
threaded through it. `docs`' *COPY vs REDERIVATION* entry is the reason — an "exactly" claim
survives a copied instruction sequence and dies on a second derivation — and the Python says so
in its own docstring (`_hp_eta_loop` is called **verbatim**; `_cascade_map` and `_lp_eta_loop` are
left **literally unchanged** so rungs 39/41's suites keep witnessing them). Factoring the
duplication away would look like a cleanup and break P7 bit-for-bit.

**Sizing.** The oracle runs 147 cells × 4 bleed levels on rung 42 (≈5 min on PyPy — the measured
probe cost was 296 s for the same sweep plus rung 41's schedules), plus rung 41's schedule methods
over 4 shapes × 2 floors × 3 gases and `flow_coefficient_turn` over gate 5's 19 cases × 2 spools
(≈8 s flat, ≈79 s over the shaped grid). The **200-pass cells are the cost trap**, and the bleed axis makes more of them: 10 → 17
as the valve opens.

### 5.9 SLICE M (rungs 53 + 54, `VariableStatorMatcher`) — PRE-REGISTERED, nine probes MEASURED first

**PHASE 5's REMAINDER IS THREE SLICES, NOT ONE.** The phase-5 row lists `53–56, 61` as one
block and the memory index carried "slice M (53–56, 61) is next"; **neither was ever sized.**
Enumerated, that block is **~1 600 Python lines and 103 test gates** — 22 · 21 · 18 · 21 · 21,
**counted** with `grep -c "^def test"` and not estimated, because a section whose whole point is
that nobody counted must not itself carry a guess (the first draft of this table said "~106",
"19" and "22", and all three were wrong) — against slice L's ~390 lines and 19 gates, which
still needed four steps and four commits. It splits on DEPENDENCY,
the way § 4.3 grouped phase 3:

| slice | scope | Python | gates | depends on |
|-------|-------|--------|-------|------------|
| **M** | 53 + 54 — `VariableStatorMatcher` + `ComponentMap`'s `vsv`/`capacity` channels | ~585 | 22 + 21 | slice L |
| **N** | 55 + 56 — `StageStack` + `StageStackMatcher` | ~685 | 18 + 21 | **M** (`test_rung56.py::test_reduce_K1_is_rung54_throat_margin_bit_for_bit`) |
| **O** | 61 — `StatorBleedMatcher` | ~305 | 21 | **M** *and* slice L's `bleed.rs` (it is the § 6 diamond, `TwoSpoolBleedMatcher` × `VariableStatorMatcher`) |

53 and 54 are **inseparable**: 54's `throat_margin` extends 53's `stator_margin` row in place,
and 54's `_schedule_root` is the documented immune replacement for 53's `incidence_schedule`
ladder — the two docstrings cross-reference each other. Slice boundaries are free inside an
authorised phase, so this is a SIZING correction, not a scope change, and needs no fresh
authorisation. The phase-5 row's scope list is unchanged.

`probe_m1.py`, PyPy. Grid: **2 gases (CPG, TPG) × rung 53/54's own five disclosed shapes ×
4 throttles (1500/1200/1000/800) × 2 spools = 80 cells**, which is the grid the slice-M dump
will sweep — never a neighbouring one (§ 5.7 (e)).

**(i) THERE IS EXACTLY ONE CAUGHT SCOPE IN RUNGS 53/54, AND IT OVERTURNS A RECORDED SLICE-J
VERDICT.** `grep except` over the two rungs' 585 lines returns **one** hit: rung 54's `_scan`
wraps `at_setting(v,·).throat_margin(…)` in `except AssertionError: break`. Reproduced line for
line with the innermost raising frame recorded:

| gas arm | cells | how the walk ends | innermost raising frame |
|---------|-------|-------------------|-------------------------|
| CPG + TPG | **80 of 80** | broke on `AssertionError` | **`ComponentMap.solve_n`, the speed-line bracket — every single one** |
| **equilibrium** | **20 of 20** | broke on `AssertionError` | **`solve_n`, again — every single one** |
| either | 0 | ran to `_V_MAX` | — |

Cross-tabbed by the spool being swept, the split is **40 / 40** on the first arm and **10 / 10**
on the second, **with NO crossover in either**: sweeping the LP stator kills `_lp_eta_loop`'s
`solve_n`, sweeping the HP kills `_hp_eta_loop`'s.

**THE EQUILIBRIUM ARM IS NOT A FORMALITY, AND IT WAS RUN BECAUSE THE FIRST GRID COULD NOT BACK
THE CLAIM.** `_scan`'s caught chain also contains `_equil_solve`'s Newton and `_solve_f`'s
burner, and slice L's own table records those two firing **14** and **3** times inside a caught
scope on ITS grid. Had either been the innermost frame here, **two more call sites would need
fallible twins** — a TYPE SIGNATURE, discovered only after `stator.rs` had been written against a
two-site shape. Neither appears: **100 cells, 100 `solve_n`, 50/50 by spool.** This is § 5.8.1
(i)'s discipline reused — that probe was re-run on shaped maps for exactly the same reason, and
the rule is *a bar measured on a narrower grid than the dump sweeps is a bar over unmeasured
cells.* That is not
plumbing — it is the lever unloading its own speed line until the map stops being valid, which is
what `_scan`'s docstring calls "the SOLVE itself gives out".

**`map.rs`'s shipped note says the opposite, and slice M overturns it by measurement** — the only
thing allowed to. It reads: *"[`solve_n`]'s bracket assert … would fire **0** times … so it stays
an `assert!` and not an `Abort`: nothing catches it"*, measured over slice J's 810 cells (§ 5.6
(a)). Slice J's grid had no `_scan` in it, because rung 54 did not exist in the Rust yet. This is
**the second time** a slice has overturned a predecessor's fallibility verdict on a wider grid —
slice L's step 1 did it to § 5.4 (i)'s "`solve` stays a panic" — and it is the same lesson twice:
**a zero-firing verdict is a claim about the GRID, and it expires when a new caller arrives.**
Per-call-site, as slice L established:

| `solve_n` call site | reached by slice M's caught scope? | verdict |
|---------------------|-----------------------------------|---------|
| `two_spool.rs:1408` — rung 39 `_hp_eta_loop` | **yes, 40 firings** | → `try_` twin |
| `two_spool.rs:1443` — rung 39 `_lp_eta_loop` | **yes, 40 firings** | → `try_` twin |
| `map.rs:511` — rung 32 `operating_point` | no | keeps `assert!` |
| `bleed.rs:158` — rung 42 `lp_eta_loop_bleed` | no | keeps `assert!` |

**(ii) `_V_MAX = 8.0` IS DEAD.** No walk reaches it. The break settings span **1.16 – 3.36**
(26 distinct values), and `n_scan` runs **29 – 84**. Ported as written and **recorded dead**, so
no reader infers it is load-bearing — slice L's `Tt4_lo = 350.0` precedent, second instance.

**(iii) RUNG 53's OWN FLOOR ASSERT NEVER FIRES.** `stator_margin`'s
`assert phi_s < phi_op` ("the running line has crossed its OWN floor"), called OUTSIDE any catch
over `v ∈ {−0.30, −0.10, 0, 0.10, 0.30, 0.60, 1.00}` × 5 shapes × 4 throttles × 2 gases ×
2 spools: **560 clean calls, 0 raises.** Per the zero rule it stays `assert!`.

**(iv) `_INC_MAX = 80` IS A DEAD SHADOW ON BOTH ROOT-FINDERS, AND SO IS THE SECOND STOP CLAUSE.**
The phase-5 row flags "`_INC_MAX`'s live shadow"; measured, it is not live.

| root-finder | bisection passes | stop clause |
|-------------|------------------|-------------|
| rung 53 `incidence_schedule` (doubling ladder) | 30 · 32 · 33 · 34 · 35 · **36 max** | `\|r\| <= _INC_TOL` on **all 42** |
| rung 54 `_schedule_root` (bracketed off the scan) | 26 · 28 · 29 · 30 · 31 · 32 · **33 max** | `\|r\| <= _INC_TOL` on **all 54** |

**Said precisely, because the loose phrasing invites a deletion:** the width test is the SECOND
disjunct of one `if abs(r) <= _INC_TOL or hi - lo <= 1e-14`, so what is measured is that the
tolerance clause **always short-circuits it** — the width clause is *unreached*, which is not the
same as *inert*. **Both disjuncts port.** (`docs`' *golden gate slice 4* entry is this lesson
already: lead with the reader that BYPASSES the short-circuit.)

Neither reaches 80. Unlike § 5.8.1 (iii)'s 33, these counts are **not** predictable from the
arithmetic — the ladder's bracket width is data-dependent — so the gate asserts the measured SET,
names its instrument (**bisection passes, not residual evaluations, not ladder steps**), and is
**PyPy-arm only**, per slice K's P2 on count stability.

**(v) THE TWO SCHEDULE CENSUSES.** Rung 53's ladder over the 80 cells: **20 "residual already
zero"** (exactly the design throttle — 2 gases × 5 shapes × 2 spools), **42 roots**, **18 bracket
ASSERTS** that propagate uncaught. Ladder steps ∈ {0, 3, 4, 5, 6}. Rung 54's bracket branch:
**54 bracketed · 20 "no crossing → 0.0" · 6 "no crossing → None"**, and `schedule_throat` returns
**74 `exists=True` / 6 `exists=False`**.

**(vi) THREE FIELD-SET SPLITS, AND A FLOAT DUMP IS STRUCTURALLY BLIND TO ALL THREE.** Slice L's
P9 rule (*a value oracle cannot see a missing value*), now three times over in one slice:

| object | branch | the difference |
|--------|--------|----------------|
| `throat_margin` | `capacity > 0` | **16 keys vs 19** — `choked`, `m_c`, `throat_mach_design` appear only with a throat model (80 rows at 16, 160 at 19) |
| `authority_ceiling` | `v_ch is None` | **86 of 240 cells** return `m_i_at_throat=None` where the rest return a float |
| `schedule_throat` | `exists=False` | **6 of 80 rows** null `vsv_star`/`throat_loading`/`c_min` **and drop** `tan_b1`, `m`, `phi_op`, `n`, `m_i`, `m_phi` outright |

Each gets a discriminant column plus a declared sentinel, and the gate asserts the branch COUNT.

**(vii) THE `binds` CENSUS MEASURES PHYSICS, NOT PLUMBING.** `authority_ceiling`'s verdict —
which of the three ceilings stops the stator first — over 240 cells (80 × 3 capacities):

| `C` | throat | peak | edge |
|-----|--------|------|------|
| 0.00 | **0** | 48 | 32 |
| 0.80 | **54** | 26 | 0 |
| 0.90 | **66** | 14 | 0 |

`peak_interior` is **144 True / 96 False**. The throat column rising with `C` is rung 54's own
claim — *a tighter throat binds earlier* — expressed as a count the source could refute, which is
what earns a count bar its place (`docs`' *guessed census bars* entry, and § 5.8.1 (v)'s
precedent).

**(viii) A CLAIM THE SHIPPED SOURCE CARRIES IS NARROWER THAN IT STATES — AND THE NARROWING IS A
PARAMETER THE DOCSTRING NEVER NAMES.** `incidence_schedule`'s docstring says the doubling ladder
"can step OVER the root and out the far side — reporting the schedule unreachable when it exists
(measured: the `steep` shape at Tt4=1200, root at v* = 0.909)". **At the method's own default
`v_hi = 1.0` it does no such thing**: on that exact cell it returns **0.9090766150970013**,
agreeing with rung 54's scan-bracketed root to 1e-10. The failure needs the cap RAISED, and the
source's own test supplies one — `test_rung54.py::test_rung54_root_finds_a_schedule_rung53s_doubling_ladder_walks_over`
passes `v_hi = 0.98 * v_edge = 1.725`. Swept over caps {1.0, 0.98·v_edge, v_edge, 2.0, 4.0} on
3 shapes × 2 spools:

* the walk-over reproduces on **exactly 1 of 6 cells** (`steep`/LP), and there only at caps ≥ 1.725;
* **`steep`/HP has an interior peak too and never walks over at ANY cap.** So an interior peak is
  **necessary but not sufficient** — the peak must lie *between the root and the cap*. The
  docstring names neither condition.

**AND `docs/rung54-spec.md` CARRIES IT WORSE — CHECKED, NOT ASSUMED.** Its § at lines 168–172
prints a ladder TRACE: *"rung 53's method walks 0.05 → … → 1.6 (residual +1.64e-2, already
climbing) and asserts the schedule unreachable."* **That trace is unreachable at the shipped
default.** The ladder's step is `hi = min(2*hi, cap)` with `cap = v_hi = 1.0`, so the sequence is
0.05 · 0.1 · 0.2 · 0.4 · 0.8 · **1.0** and stops — reaching **1.6 requires `v_hi >= 1.6`**, i.e.
a cap 60 % above the default the spec never mentions. So the spec does not merely omit the
condition, it publishes a walk the method cannot take as called. **This is a source correction
slice M owes**, in the same class as slice L step 4's, and it is recorded here rather than left
as a passing remark because *asserting a defect in a file one has not opened* is precisely the
copied-bar failure `docs`' *slice L step 4* entry records.

The port's job is to **reproduce both sides**, not to repair either: a more careful Rust bracket
would look like an improvement and be a silent divergence.

**(ix) SLICE M DEFERS ZERO TEST GATES — THE FIRST PHASE-5 SLICE WITH NONE, AND IT IS ENUMERATED
RATHER THAN ASSUMED.** *(Read precisely: this is about the 43 gates in the two Python suites, all
of which port now. It is NOT the same object as P10's IOU, which is a gate that **cannot exist
yet** — no descendant exists to witness `at_setting`'s dispatch until slice N. `slice_m_deferrals`
holds that one entry and no deferred gate.)* `grep -n "Transient\|StageStack"` over both suites returns **no hit in
either** (the only `build_turbojet` calls are each suite's own single-spool "cycle untouched"
gate, which ports now). Every one of the 43 gates is steady and reachable from slice L's ladder.
This is checked BY ENUMERATION because `docs`' *an oracle cannot see a MISSING GATE* entry says a
bit-exact dump says nothing about coverage.

#### The predictions

* **P1 — THE CAUGHT SCOPE REACHES `solve_n` AND NOTHING ELSE, PER CALL SITE, ON ALL THREE GASES.**
  **100 of 100 cells** abort (80 CPG+TPG + 20 equilibrium), the innermost frame is `solve_n`'s
  bracket every time, and the firing loop tracks the swept spool **50/50** with no crossover. The two rung-39 sites get `try_` twins; rung 32's and rung
  42's keep `assert!`. *Refuted by:* any cell whose innermost frame is not `solve_n`, any
  crossover between swept spool and firing loop, or any walk that reaches `_V_MAX`.
* **P2 — THE REFACTOR AND THE TWO NEW MAP FIELDS MOVE NO SHIPPED BIT.** Three changes to
  already-gated code land here: `vsv` and `capacity` arriving on `ComponentMap`, and the
  `try_solve_n` twins on two call sites. `map_oracle` (7 252), `offdesign_oracle` (3 951),
  `two_spool_oracle` (11 812), `slice_l_oracle` (25 458) and the `rung31`/`rung32`/`rung33`/
  `rung38`/`rung39`/`rung41`/`rung42` suites re-run **bit-identical**. *Refuted by:* one changed
  bit ⇒ **revert**, per slice J's rule for a refactor to gated code. Note this is the THIRD
  instance of that rule and the largest re-run set the port has had.
  **The BASELINE is established BEFORE the first edit, not reconstructed after a failure:**
  `cargo test --release` on the pre-slice-M tree is **481 passed / 0 failed across 56 suites,
  exit 0** (`baseline_cargo.txt`). A prediction whose refutation costs a revert is worthless
  without a recorded "before" — otherwise a red suite after the edit cannot be told from drift
  that was already there.
* **P3 — THE `vsv == 0` AND `capacity == 0` EARLY RETURNS ARE PORTED AS EARLY RETURNS.**
  `psi` and `phi_max` both return before touching the swirl term at `vsv == 0.0`, and
  `phi_surge_at` returns the FIELD. Writing `base - 0.0*(1+l)*phi` instead is an assumed
  algebraic no-op, which is the *power-spelling* entry's exact failure class. *Refuted by:* any
  rung ≤ 52 key moving in P2's re-run.
* **P4 — `_V_MAX = 8.0` AND `_INC_MAX = 80` ARE DEAD, AND SO IS `hi − lo <= 1e-14`; ALL THREE
  ARE PORTED AS WRITTEN AND RECORDED DEAD.** *Refuted by:* any walk terminating on `_V_MAX`, any
  bisection reaching 80 passes, or any stop on the width clause.
* **P5 — THE ROOT-FINDER PASS COUNTS ARE (iv)'s SETS, AND THE GATE NAMES ITS INSTRUMENT.**
  Rung 53 ∈ {30, 32, 33, 34, 35, 36}; rung 54 ∈ {26, 28, 29, 30, 31, 32, 33}. Bisection passes —
  **not** residual evaluations, **not** ladder steps; § 5.7 (d) corrected a bar one slice ago for
  exactly this. PyPy arm only. *Refuted by:* any count outside its set.
* **P6 — THE THREE FIELD-SET SPLITS ARE DUMPED AS DISCRIMINANTS PLUS DECLARED SENTINELS, AND THE
  BRANCH COUNTS ARE THE GATE.** 80/160 on `throat_margin`'s capacity branch, 86/154 on
  `authority_ceiling`'s `v_ch`, 74/6 on `schedule_throat`'s `exists`. No nullable column ever
  carries a bare `0.0`. *Refuted by:* a Rust null row comparing equal to a real value — the
  failure a float dump cannot see.
* **P7 — THE `binds` CENSUS IS (vii)'s TABLE, INCLUDING THE ZEROS, AND THE THROAT COLUMN RISES
  WITH `C`.** 0 / 54 / 66 at C = 0.00 / 0.80 / 0.90. *Refuted by:* a nonzero throat count at
  C = 0, or a throat column that does not rise.
* **P8 — THE WALK-OVER IS CAP-CONDITIONAL, AND BOTH SIDES ARE GATED.** At `v_hi = 1.0` the Rust
  ladder returns `steep`/LP/1200's root bit-for-bit; at `v_hi = 0.98·v_edge` it raises. The
  necessary-but-not-sufficient reading of `peak_interior` is gated by `steep`/HP, which has an
  interior peak and never walks over. *Refuted by:* a Rust ladder that succeeds at the raised cap
  (it would have silently repaired the defect) or fails at the default one.
* **P9 — THE TWO REDUCE CONTRACTS ARE DIFFERENT SHAPES AND BOTH PORT.** Rung 53's is an
  **IDENTITY** — at `vsv == 0` the stored maps are the SAME OBJECTS and `match` is rung 39's
  inherited method, so there is no rung-53 path to skip. Rung 54's is an **INVARIANCE OVER `C`** —
  every matched field bit-identical at every capacity, at a MOVED stator, which is strictly
  stronger. **The identity half SPLITS on what Rust can express, and the split is decided here
  rather than discovered mid-port:** Python's gate is two assertions, and only one of them
  survives.
  * `VariableStatorMatcher.match is TwoSpoolMapMatcher.match` (`test_rung53.py:98`) — *"there is
    no rung-53 code path to skip"* — ports **EXACTLY**, as raw **fn-pointer equality between the
    `R53` and `R39` hook-table entries**. This is the half that carries the actual claim, and the
    hook table is what makes it expressible.
  * `m.map_lp is LP` ports **WEAKER**. `ComponentMap` is `#[derive(Copy)]`, so it has no object
    identity to compare and none is meaningful for a value type; the honest Rust is field-wise
    `==` plus `vsv == 0.0`. **That weakening is stated in the gate's own text**, not only here —
    a reduce gate that silently answers a smaller question is the `docs` *ported test can go
    VACUOUS* failure.

  *Refuted by:* any matched field moving with `C`, or the two hook entries differing.
* **P10 — `at_setting` ENTERS THE HOOK TABLE IN THIS SLICE, WITH ITS DISPATCH GATE OWED TO SLICE
  N.** It is overridden at **three** levels (53, 55, 61) and `stator_sweep`, `currency_split`,
  `incidence_schedule`, `_scan` and `schedule_throat` all reach it through `self`. Hardcoding
  rung 53's body would compile, return numbers, and return the WRONG ones under slice N — the
  failure `map.rs`'s own note warns about for `solve_turbine`. Slice M has no descendant to
  witness the dispatch with, so the gate is an **IOU** in `rung53.rs::slice_m_deferrals` (the
  `slice_j_deferrals` / `slice_l_deferrals` precedent, fourth use), NOT a silent omission.

#### Module decision, and sizing

**A new `stator.rs`**, carrying `VariableStatorMatcher` with its own `R53`/`R54` hook-table
entries, `at_setting` as a virtual, rung 53's four reading methods and rung 54's six. `two_spool.rs`
is already 2 025 lines and slice K split it out of `matcher.rs` at exactly that bar. `ComponentMap`'s
own rung-53/54 channels (`with_vsv`, `tan_beta1`, `tan_beta1_crit`, `phi_surge_at`, `with_capacity`,
`throat_ratio`, `throat_loading`, `capacity_margin`, `chokes`, `design_throat_mach`) go into
`map.rs` beside the fields they read, and `map.rs`'s *field subset* note — which already records
`vsv` as owed to this slice — is discharged there.

**The one shape decision that decides a reduce gate**, stated before it can be got wrong:
`throat_margin` **mutates rung 53's returned row in place** and returns the same map. Rebuilding it
as a fresh struct with the union of both field sets would look like a cleanup and would destroy
P6's 16-vs-19 discriminant, which is the only instrument that can see the capacity branch at all.

**Sizing.** The `_scan` walk is the cost trap: 80 cells × 29–84 settings × a full two-spool match
each, measured at **17.5 s** for the abort census and **71.7 s** for the three-capacity `binds`
sweep on CPG+TPG. Rung 53's ladder and rung 54's root are cheap by comparison (8.7 s and 24.9 s
over the same 80 cells).

**The equilibrium gas is the cost, and it is now measured rather than guessed: 887 s for 20
cells, ≈ 44 s per `_scan`** — about 200× the CPG/TPG cell. A full 40-cell equilibrium arm is
therefore ≈ 30 min, and the first attempt at this measurement **was killed at a 15-minute cap
with its output still buffered, having recorded nothing** — so the re-run prints per cell,
unbuffered. Two consequences for the dump: the equilibrium arm is sampled at 2 throttles rather
than 4, and any `_scan`-driven dump column states its own grid.

#### The steps — SIX, and step 1 is the whole revert unit

Slice L's shape (four steps, four commits) with two additions: the gated-code refactor is
ISOLATED into step 1 so P2's "one changed bit ⇒ revert" has a clean unit to revert, and the
source correction slice M owes (§ 5.9 (viii)) is its own docs-only step at the end.

| step | scope | gate |
|------|-------|------|
| **1** | **ALL changes to already-gated code**: `vsv` + `capacity` on `ComponentMap`, its ten rung-53/54 channels, `is_flat`, the `psi` early return, and the two `try_solve_n` twins with the hook-table signature change they force | **P2** — the four oracles and the seven suites re-run bit-identical |
| **2** | `stator.rs` rung 53: `VariableStatorCore`, `R53`, `at_setting` as the table's fourth slot, `stator_margin`, `stator_sweep`, `currency_split`, `throttle_currency`, `incidence_schedule`; the `slice_m_deferrals` IOU (P10) | compiles + a smoke check against one dumped cell |
| **3** | `stator.rs` rung 54: `throat_margin`, `throat_sweep`, `_scan`, `_interp`, `_cross`, `authority_ceiling`, `_schedule_root`, `schedule_throat`, `R54` | same |
| **4** | the slice-M oracle — Python dump LAUNCHED FIRST (see below), Rust reader written while it runs; P4/P5/P6/P7's census bars | `slice_m_oracle.rs` bit-exact |
| **5** | the two suites, `rung53.rs` (22) + `rung54.rs` (21), incl. P8's cap-conditional walk-over and P9's two reduce contracts | 43 gates green |
| **6** | the SOURCE correction: `docs/rung54-spec.md` §'s unreachable trace, and `incidence_schedule`'s docstring condition | docs-only, **no gate** (§ Commands' rule) |

**(a) P2's REVERT UNIT IS WIDER THAN P2's OWN TEXT, AND THE EXTRA PART IS IN TWO MORE FILES.**
P2 enumerates "`vsv` and `capacity` arriving on `ComponentMap`, and the `try_solve_n` twins on
two call sites". But `solve_n`'s bracket assert fires INSIDE `r39_hp_eta_loop` /
`r39_lp_eta_loop`, so a fallible twin at those sites turns those two hook entries from `EtaLoop`
to `Result<EtaLoop, Abort>` — a change to the PUBLIC `TwoSpoolHooks` signature, which ripples
into `try_match_point`'s body and into `bleed.rs`, whose cascade calls `hp_eta_loop_closed`
verbatim THROUGH the table. So the unit is **two fields + two twins + one hook-table signature
across `map.rs`, `two_spool.rs` and `bleed.rs`.** Recorded here because "revert" needs to know
what is in the revert; it is the same *fallibility is per CALL SITE* discipline as slice L step 1,
one level up — there the twin was per site, here the SIGNATURE is per table.

**(b) THE BASELINE IS SUFFICIENT AS RECORDED, AND THAT WAS CHECKED RATHER THAN ASSUMED.** P2's
"before" is a COUNT (481/0, 56 suites), while the claim is bit-identity of four oracles. Those
are the same object here only because every oracle compares against a **committed `.tsv` golden
loaded with `include_str!`** — the goldens are Python-generated, in git, and untouched by this
slice, so a passing `cargo test` IS the bit-identity check. Had the comparison been external, the
pre-edit dumps would have to be saved BEFORE the first edit, because once the two fields land a
"before" cannot be regenerated without reverting — which is the exact failure the baseline note
was written to prevent.

**(c) `at_setting`'s SIGNATURE IS DECIDED HERE AND IS ADDITIVE FOR N AND O.** With const
fn-pointer tables (§ 4.1) and three overriding rungs, the carrier is a **descendant enum field**
on `VariableStatorCore` (`Plain` / slice N's `Stack{…}` / slice O's `Bleed{…}`), and the hook is
`fn(&VariableStatorCore, f64, f64) -> VariableStatorCore`. **The condition that makes this
survive was checked, not hoped:** rung 55's and rung 61's `at_setting` bodies read only fields of
`self` (`K_lp`/`K_hp`/`split`/`vsv_stages_*`/`cap_profile`; `bleed`) and re-construct their own
type — neither reads anything slice M cannot store. So slices N and O add a VARIANT and a TABLE
ENTRY and change no signature. Stated now because the alternative is discovering it mid-slice-N
and paying for two more gated-code refactors.

> **REFUTED BY SLICE N's PROBES — § 5.10.** The literal claim holds (`at_setting`'s signature is
> untouched) but the conclusion does not: rung 55's state is the two BUILT stacks, whose `Vec<f64>`
> ladders cost `Descendant` its `Copy`, and its overrides live on rung 39's `TwoSpoolHooks`, which
> `with_hooks` hardcodes as `&R39`. **Reading a method's body tells you what state it READS; it
> cannot tell you what the state's CARRIER costs.**

**(d) P6's SENTINEL IS DECIDED BEFORE THE DUMP IS WRITTEN.** The three field-set splits become
`Option` in Rust and the dump needs a scalar column. `NaN` **fails open** — a null row and a live
row both compare unequal, so a diff cannot see the very class of error P6 exists to catch — and
`0.0` is what P6's own refutation clause names. The form is therefore a **presence column gated
on its measured count** (80/160, 86/154, 74/6) beside a value column the reader asserts is never
read when absent. `throat_margin`'s 16-vs-19 split is the same mechanism: emit the three throat
keys CONDITIONALLY and gate the row counts.

**(e) `is_flat` MUST DISCRIMINATE IN BOTH DIRECTIONS OR IT IS VACUOUS.** The content of Python's
predicate is an ASYMMETRY — `vsv` is part of flatness (it enters `psi`), `capacity` is not (a
pure diagnostic, like `phi_surge`). A gate asserting only `flat().is_flat() == true` passes with
the `vsv` conjunct MISSING, which is slice J→K's `l` mistake repeated on a predicate and is what
`map.rs`'s own note warns about. The gate asserts **`vsv != 0` ⇒ false** and **`capacity != 0` ⇒
still true**.

**(f) STEP 4 LAUNCHES THE PYTHON DUMP FIRST.** At ≈44 s per `_scan` the equilibrium arm is ~15
min even sampled at 2 throttles, and the first attempt at that measurement already died at a
15-minute cap with its output buffered. The dump is started DETACHED and UNBUFFERED as step 4's
first action, and the Rust reader is written while it runs — not after.

#### Step 1 — SHIPPED. **P2 and P3 HOLD; the revert unit reached a TEST, which (a) did not say**

`cargo test --release`: **481 passed / 0 failed across 56 suites, exit 0** — every digit the
pre-edit baseline's, so all four value oracles and all seven suites re-run bit-identical and **P2
is settled without a revert.** P3 rides on the same run: `vsv` is now a live field in `psi`, and
the only thing standing between it and every rung ≤ 52 number is the `vsv == 0.0` early return.

**What the port found that (a) did not.** (a) enumerated the revert unit as three SOURCE files.
It is four files, and the fourth is a GATE: `rung38.rs`'s "no LP quantity is a parameter" test
calls `hp_eta_loop_closed` DIRECTLY — it is a signature test, so a signature change reaches it by
construction. It now `.expect`s, with the reason in the gate: an `Err` there would mean the HP
leaf stopped closing on a flat map, which is a different failure from the one its assertion
describes. `rung32.rs`'s ordering arm needed the two new fields spelled out, and `vsv: 0.0` is
load-bearing there rather than padding — it is what sends `psi` down the early return, so the arm
really does compare orderings of the TWO-term expression it names.

**Two zero-firing verdicts were re-dated rather than deleted.** `map.rs`'s "0 of 810 cells, so it
stays an `assert!`" is left standing beside the correction, because it is still true of the grid
that measured it; and `bleed.rs`'s `lp_eta_loop_bleed` keeps its panicking `solve_n` **with the
expiry written into the module note** — rung 61 overrides `at_setting` to keep the valve open
through every sweep, which puts that site inside `_scan`'s catch for the first time. Slice O
measures it; it does not inherit the paragraph.

One thing shipped that step 1 did NOT gate, and it is named so it cannot be forgotten: `phi_max`
is still unported (phase 6), and **its rung-53 early return is owed with it** — porting that body
without the `vsv == 0.0` branch would be P3's failure one phase late.

#### Steps 2–3 — SHIPPED. Both smoke grids were BLIND to branches they were written for

Rung 53 (step 2) and rung 54 (step 3) each ported bit-for-bit on the first run — 169 keys, then
421 — but **in both cases the first draft of the smoke check could not see the failure it
existed to catch**, and both gaps were found by testing the instrument rather than the code.

* **Step 2.** `currency_split`'s two legs hold the OTHER spool at `self`'s setting, unlike every
  other sweep in the file. The first arm ran it only at `v = 0`, where *"hold at self's setting"*
  and *"pin to zero"* are the SAME instruction. A second arm at `vsv = (0.15, 0.10)` discriminates;
  **verified by mutating the leg to the wrong constructor** (it fails at `splitmv/lp/d_phi_op`, in
  the 4th decimal) and reverting.
* **Step 3.** On the two default maps `peak_interior` is False, every schedule EXISTS and `v_ch` is
  present whenever `C > 0` — so 369 keys asserted while the parabolic peak refinement, the
  schedule's `found: None` and the `v_ch: None`-WITH-a-throat-model branches went entirely
  unmeasured. A probe over 3 shapes × 2 spools × 4 throttles located all three in the `steep`
  shape.

**The rule that generalises: a value grid chosen for coverage of CELLS is not a grid chosen for
coverage of BRANCHES, and only the second one tests an `Option`.** Both gaps were invisible to
the key count, which is what makes them worth recording.

`at_setting` is a HOOK that REBUILDS (see `stator.rs`'s module note); `Engine` and `TwoSpoolEngine`
gained `Clone` for it. Python's `r_hi is not None` guard is UNREACHABLE — the loop assigns before
every exit — so the port breaks out of the loop WITH the value, making the unset state
unrepresentable rather than merely unreached.

#### Step 4 — the oracle. **EVERY PRE-REGISTERED CENSUS REPRODUCED, AND ONE BAR IS OFF BY A STEP**

10 950 keys over the pre-registered 80 cells. Measured against § 5.9's bars:

| bar | pre-registered | measured | |
|---|---|---|---|
| `binds` at `C` = 0.00 / 0.80 / 0.90 (throat) | 0 / 54 / 66 | **0 / 54 / 66** | ✔ |
| `peak_interior` | 144 / 96 | **144 / 96** | ✔ |
| `v_ch is None` | 86 of 240 | **86 of 240** | ✔ |
| schedule `exists` | 74 / 6 | **74 / 6** | ✔ |
| rung 53's ladder ASSERTS | 18 | **18** (62 bracket) | ✔ |
| `n_scan` | 29–84, 26 distinct | **29–84, 26 distinct** | ✔ |
| break settings | **1.16 – 3.36** | **1.120 – 3.320** | ✘ |

**(a) THE BREAK-SETTING RANGE IS THE FAILING SETTING, NOT THE SURVIVING ONE — EXACTLY ONE SCAN
STEP APART.** § 5.9 (ii) records "the break settings span 1.16 – 3.36 (26 distinct values)". The
count is right and the range is one `V_STEP` high: `1.120 + 0.04 = 1.16` and `3.320 + 0.04 = 3.36`,
and `29 * 0.04 = 1.16`, `84 * 0.04 = 3.36`. So the probe recorded the setting the walk DIED at,
while `rows[-1]["vsv"]` — which every rung-54 reader consumes as `v_edge`, and which
`authority_ceiling` divides by in `setting_cut` — is the last setting that SURVIVED. **A gate
written from the plan's number would have been off by one step on a quantity that is a
denominator.** Caught only because the dump re-measured instead of copying; this is the *slice L
step 4* lesson (a copied bar) landing on the plan's own text rather than on the source's.

**(b) THE 16-vs-19 BAR COUNTS A THIRD BRANCH THIS DUMP DOES NOT SWEEP.** § 5.9 (vi) says "80 rows
at 16, 160 at 19"; the dump measures **80 / 80**, because it sweeps the capacity branch at
`C ∈ {0, 0.80}` and the probe swept `{0, 0.80, 0.90}`. Both fully exercise the SPLIT — `C = 0.90`
is the same branch as `C = 0.80` — so the gate asserts its own grid's 80/80 and says why it
differs. Per *guessed census bars*: prefer the per-arm count actually measured over a total whose
enumeration is not stated.

**(c) THE EQUILIBRIUM ARM WAS RE-SIZED, AND THE SIZING ERROR IS THE PLAN'S.** § 5.9 sized it at
"887 s for 20 cells, ≈ 44 s per `_scan`" — a figure that assumes ONE scan per cell. The full
per-cell body runs **five** (the bare scan, one inside each of three `authority_ceiling` calls,
one inside `schedule_throat`), i.e. ~75 min, which would put a quarter-hour into the Rust gate.
The equilibrium arm therefore dumps the margin rows, ONE scan and the throat rows, and nothing
that re-scans. **What it is FOR survives intact**: P1's claim is that the caught scope reaches
`solve_n` on all three gases, and the scan IS the caught scope — its LENGTH is the witness. The
`binds` census and the schedule split are cpg/tpg columns and the gate says so.

**(d) P5 HAD NO INSTRUMENT AND NOW HAS ONE.** The pass-count sets are inside the shipped loops and
`psi_calls` cannot serve — both root-finders reach `psi` through a FULL two-spool match, so a
`psi` tally measures the plant, not the search. Two thread-locals in `stator.rs`, incremented once
per BISECTION PASS (not per residual evaluation, not per ladder step) and read-and-reset so a
caller can attribute passes to one root.

#### Step 6 — the source correction, VERIFIED RATHER THAN COPIED

`docs/rung54-spec.md` § P-C3 published a ladder trace the method **cannot take as called**, and
`incidence_schedule`'s docstring stated the walk-over without the condition that produces it.
Both now carry the measurement, and all three of its claims were re-run directly rather than
copied from § 5.9 (viii): at `v_hi = 1.0` the `steep`/LP/1200 cell returns
**0.9090766150970013**, agreeing with rung 54's scan-bracketed **0.9090766151249412** to 2.8e-11;
it walks over at caps ≥ 1.725 and at no smaller cap; and `steep`/HP has an interior peak yet never
walks over at ANY cap in {1.0, 0.98·v_edge, v_edge, 2.0, 4.0}.

#### Step 5 — the two suites. **43/43 GATES PORTED; THE `slow` MAPPING IN § 6 IS WRONG**

`rung53.rs` (24 tests) and `rung54.rs` (25 tests), both green first run. The counts exceed
Python's 22 + 21 because **four gates had to SPLIT**, and every split is forced rather than
stylistic: Rust's `#[should_panic]` is per-test, so Python's `pytest.raises` inside a loop over
three refused capacities becomes three tests, and two more refusals that sat inside a value gate
become their own. The coverage check is therefore a **name → parameter-set diff, never a count**
(slice L's guessed-bars lesson, and the fifth time a count bar would have been wrong): all 22 and
all 21 Python names are present, with every `parametrize` set carried whole.

**§ 6's test-suite mapping is corrected by measurement.** It prescribes
`-m "not slow"` → `#[ignore]` + `cargo test -- --ignored`. That is wrong, and this slice is the
first evidence: Python marks **13** of these 43 gates `slow`, and the ported 49 tests — the 13
included, which between them run 10 shape×spool cells, 12 full matches, 8 bisected schedule
points and two 5-shape × 6-throttle authority scans — finish in **0.82 s + 1.70 s**. The marker
records a COST that did not survive the port, not a claim. Carrying it would deselect 13 real
gates from the default run to save ~2 s, which is precisely the silent deselection
`conftest.py`'s one-gate policy exists to forbid. **Rule for the remaining slices: port the
gate, drop the marker, and re-introduce `#[ignore]` only against a MEASURED cost.**

Three Python assertions are about Python and not about the physics; each is ported to a
substitute or booked, never dropped silently (`rung53.rs::slice_m_deferrals`):

| Python asserts | Rust | why |
|---|---|---|
| `m.map_lp is LP` (object identity) | bit equality of all 9 fields | `ComponentMap` is `Copy` — "the same object" is not a thing a Rust map can be |
| `VariableStatorMatcher.match is TwoSpoolMapMatcher.match` | `hooks.hp_eta_loop as usize == R39.hp_eta_loop as usize` | no inheritance; delegation is the twin. Comparing two `R53` entries would be slice L's self-comparison and would pass on any table |
| `raises(…, match="lp_disabled")` | **unrepresentable** — the parameter does not exist | a type-level refusal is strictly stronger than a runtime one, so nothing is owed |

**Two genuine debts, both booked:** `ComponentMap::phi_max` (a third of rung 53's expression
gate and part of rung 54's `C=0` gate — the symbol is read only by the rung-34/40/43 FORWARD
transient closures, so it is **phase 6's**, with the Python assertion quoted at the deferral so
the port need not re-derive it); and the `StatorHooks` **dispatch**, which slice M cannot witness
because the table has one entry and `Descendant` one variant — **owed to slice N**, with the
arity pinned by an exhaustive `match` so a second variant arriving without a dispatch gate fails
to compile rather than passing quietly.

Three map factories were missing from Rust and are added as literal table rows: `surge_flow`,
`surge_pressure`, `surge_tilted`. Slice K had wanted `surge_pressure` and spelled it inline in
`rung39.rs:170`; that inline copy is now redundant.

### 5.10 SLICE N (rungs 55 + 56, `StageStack` + `StageStackMatcher`) — PRE-REGISTERED, three probes MEASURED first

**§ 5.9 (c) IS REFUTED, TWICE, AND THAT IS THIS SECTION'S FIRST FINDING.** Slice M decided
`at_setting`'s carrier ahead of time and wrote: *"rung 55's and rung 61's `at_setting` bodies read
only fields of `self` … so slices N and O add a VARIANT and a TABLE ENTRY and change no
signature."* The literal claim survives — `StatorHooks::at_setting`'s signature is untouched — but
the conclusion it was written to support does not. **Two gated-code edits are forced, and neither
is visible from the method bodies (c) inspected:**

**§ 5.11 (v) CORRECTS THE SECOND HALF OF THIS: SLICE O ADDED NO VARIANT.** Rung 61's `at_setting` reads only the bleed fraction, which has lived on `TwoSpoolMapCore` since slice L — so slice O is a TABLE ENTRY and `Descendant::Plain`. The prediction was right about slice N and wrong about slice O, for the same one-level-down carrier reason that cost slice N two gated-code edits.

1. **`Descendant` cannot stay `Copy`.** Rung 55's state is not the five scalars (c) enumerated;
   it is the two **built** `StageStack` objects, and a stack carries `theta_d`, `varpi_d` and
   `_C_ks` — three runtime-length `Vec<f64>` ladders. `#[derive(Clone, Copy)]` on `Descendant`
   goes, and `r53_at_setting` (`stator.rs:487`) gains a `.clone()`.
2. **`VariableStatorCore::with_hooks` hardcodes `&R39`** (`stator.rs:340`). Rung 55 overrides
   `_hp_eta_loop`/`_lp_eta_loop`, which live on rung 39's `TwoSpoolHooks`, not on `StatorHooks` —
   so rung 53's constructor has to take a **second** `&'static TwoSpoolHooks` and thread it down.

**The lesson, stated at the level it generalises to: reading a method's body tells you what state
it READS; it cannot tell you what the state's CARRIER costs.** (c) checked the former and
concluded the latter. It is the same shape as *a scope list is only as good as an enumeration
over that set* — a claim about a set, verified on a proxy for the set.

**AND THE THIRD FINDING IS THE OPPOSITE SIGN — the first time a predecessor's refactor PREVENTED
a slice's churn.** Slice N's new raising site sits inside the two efficiency loops, and slice M
step 1 already turned those two hook entries into `Result<EtaLoop, Abort>` for its own reasons
(§ 5.9 (a)). `two_spool.rs:1103`'s doc comment even names *"Rung 55's `StageStackMatcher`"* as the
overrider. So the expensive half of this slice — a public hook-table signature change — is
**already paid for**, and slice N inherits it rather than repeating slice J→M's and slice I→L's
*a zero-firing verdict expires when a new caller arrives*.

#### The probes — `probe_n1/n2/n3.py`, PyPy

Grid: **2 gases (CPG, TPG) × the same five disclosed shapes × 4 throttles (1500/1200/1000/800) ×
2 spools**, extended by rung 55/56's own axes — `K ∈ {2,4,8,16}`, `split ∈ {dT,tau}`,
`cap_profile ∈ {derived,uniform}`, `vsv_stages ∈ {None,1}`. **640 cells** for the throat census,
**160 rows** for the schedule census. Never a neighbouring grid (§ 5.7 (e)).

> **STEP 4's PROVENANCE NOTE ON `CAP = 0.60`.** `probe_n1` calls it *"rung 54's disclosed capacity
> constant, as the rung-56 tests carry it"*. Those tests carry **0.90** (`test_rung56.py:48`) and
> `0.60` appears nowhere in rungs 53–56 as a capacity. It is KEPT, because (iv) was measured at
> 0.60 and moving it would re-point every one of those bars at unmeasured cells — so the constant
> is **arbitrary-but-pre-registered**, which is a different justification from the one written
> down beside it.

**(i) THERE IS EXACTLY ONE CAUGHT SCOPE IN RUNGS 55/56, AND IT RAISES FROM A *NEW* FRAME WITH
*TWO* REASONS.** `grep except` over the two rungs' 685 lines returns **one** hit: rung 55's
`stage_incidence_schedule` wraps `resid(x, Tt4)` in `except AssertionError: break`
(`engine.py:7331`) — structurally rung 54's `_scan`, which is why nothing may be inherited from
it. Reproduced with the innermost raising frame recorded:

| innermost raising frame | firings | what it is |
|---|---|---|
| `engine.py:6936` `StageStack.solve_n` | **39 of 40** | the stack speed-line **bracket** assert |
| `engine.py:6950` `StageStack.solve_n` | **1 of 40** | the **clamped-root** assert |
| `ComponentMap.solve_n` | **0** | slice M's frame — **never reached** here |

**Slice M's answer does not carry over, and the 1-of-40 is the reason this was measured rather
than assumed.** With both spools stacked, `ComponentMap.solve_n` is called **zero** times in a
match — `StageStack.solve_n` replaces it — so the frame slice M found 100/100 times is absent.
The fallible twin is therefore `StageStack::try_solve_n` with **two** abort reasons, both live,
and the rare one is the one a smoke grid would miss (slice M steps 2/3's lesson, third instance).

**(ii) WHICH CELLS BREAK, AND IT IS RUNG 55's OWN HEADLINE APPEARING IN THE FAILURE CENSUS.**
160 schedule rows: **120 reached, 40 not.** All 40 non-reached rows are on the **lumped** lever
(`vsv_stages = None`, every stage moves); **all 80 front-row-lever rows (`vsv_stages = 1`) reach.**
None is at the design throttle. That is rung 55's P3 — *the front-row lever's cost collapses* —
showing up as *the lumped lever runs out of map validity where the front-row lever never does*,
and it is a discriminator no value oracle produces.

**(iii) CONSTANT LIVENESS — five measured, three DEAD, and one dead one has no gate that could
tell.**

| constant | verdict | measurement |
|---|---|---|
| `_E_TOL` = 1e-14 | **LIVE** | `_stage_eta` ends on it at **exactly 48 passes**, every one of 120 |
| `_N_TOL` = 1e-14 | **LIVE** | `StageStack.solve_n` ends on it at **exactly 48 passes**, all 10 219 |
| `_stage_eta`'s 300 cap | **DEAD** | reached 0 times |
| `solve_n`'s 200 cap | **DEAD** | reached 0 times |
| `_T_FLOOR` = 1e-3 | **LIVE** | fires **3 204** times in 521 649 marches |
| `_P_FLOOR` = 1e-6 | **DEAD** | fires **0** times in 521 649 marches |
| `v_hi` = 4.0 | **DEAD** | no scan reaches it (slice M's `V_MAX` precedent, second instance) |
| `_M_of_nu`'s range guard | **LATENT-ONLY** | worst `nu²` on the grid is **2.7 %** of the limit |

> **CORRECTED BY STEP 2 — `_P_FLOOR`'s DEADNESS IS DERIVED, AND THE `0 / 521 649` BELOW IS ONLY
> THE GRID HALF OF IT.** `tau_k` is floored BEFORE `base = 1 + e*(tau_k − 1)` is formed, so
> `base ≥ 1 − e*(1 − T_FLOOR)` and `base < P_FLOOR` requires `e > (1 − P_FLOOR)/(1 − T_FLOOR)
> = 1.001` **exactly** — a threshold in the two floor constants alone, independent of map, split,
> `K` and design point. The count is zero because `e = e_d*(eta_live/eta_d)` sits just under it,
> not because the sweep missed a corner. Pushed past it, both floors fire on the SAME stages and
> the shared counter reads exactly double. *A dead guard's threshold is worth more than its
> count.*

> **CORRECTED BY STEP 4 — EVERY COUNT IN THE TABLE ABOVE IS OVER A 240-CELL GRID, NOT THE DUMP's
> 640.** `probe_n1`'s liveness sweep runs `K ∈ {2,4,8}` — **three** values — with **no
> `cap_profile` axis, so its 120 stacks, 10 219 `solve_n` calls and 521 649 marches are not this
> slice's grid. Measured on the dump's: **320 stacks, 65 002 + 292 500 `solve_n` calls,
> 3 317 982 + 14 926 093 marches, 320 profiles built against 24 000 cache hits.** The VERDICTS
> (live / dead / latent) all survive; the numbers do not transfer. So step 4 EMITS the census as
> dump keys and compares `take_census()` against them rather than quoting any figure here — and
> § 5.10 (iv), which WAS measured on the 640-cell grid (`probe_n3`), reproduced exactly.
> *Two censuses in one section can be measured on two grids, and the section reads as one.*

**`_P_FLOOR` IS A DEAD GUARD THAT NO GATE CAN SEE, AND THAT IS WHY IT GETS ITS OWN.** Python's
`march` adds both floors into ONE `clamped` counter, and both readers of that counter —
`solve_n`'s root assert and `test_rung55.py:191` — read only the sum. So a Rust gate on
`clamped == 0` cannot distinguish the guard that fires 3 204 times from the one that never fires,
and the single clamped abort in (i) came through `_T_FLOOR`. Ported as written, recorded dead
with its `0 / 521 649`, and **gated per floor** so the two can never be conflated — *make a dead
key earn its place* rather than delete it. `_M_of_nu`'s guard is the source's own declared latent
defect (*"gate the latent defect, not just the exercised path"*) and needs a hand-built profile to
reach, so it is a `#[should_panic]` and not a value gate.

**(iv) THE `binds` / `inc_worst` CENSUS SPLITS INTO TWO POPULATIONS, AND THE ACCOUNTING CLOSES
EXACTLY.** 640 cells × 2 spools = 1 280 half-rows per currency. Front/rear/interior:

| | front | rear | interior |
|---|---|---|---|
| `binds`, derived profile | 240 | 400 | 0 |
| `binds`, uniform profile | 50 | 587 | **3** |
| `inc_worst`, either profile | 1 182 | 88 | **10** |

**All 13 interior readings are ONE population: HP spool, CPG gas, `Tt4` = 1500 — the DESIGN
throttle — and they are decided by the LAST BIT.** At design every row sits at `phi_k = 1`, so
the per-row margins are equal to **1–2 ULP** (spread 2.2e-16 on a value of 0.818) and several rows
are **bit-identical**: one K=8 cell has rows 5, 6 and 7 exactly equal. The argmin is therefore not
physics but a **tie-break**, and the port must reproduce Python's: `min(range(n), key=…)` returns
the **first** minimum. Rust's `Iterator::min_by` also returns the first of equal elements, so the
idiomatic spelling agrees — but a `fold` with `<=`, or `max_by` anywhere near it, would not, and
**a value oracle would be blind to it because the values agree to the bit while the INDEX flips.**
This is the *location keys REFUTE* lesson at ULP scale, and it gets a dedicated gate.

Consequently the pre-registered bar is **not** "`binds ∈ {front, rear}`": it is *front-or-rear at
every off-design throttle (627 of 640 half-rows), with the 13 exceptions all at design, all HP,
all CPG, and all inside 2 ULP* — a **degenerate argmin, not a third physical class**.

**(v) COST — the equilibrium split is clean, and it decides the arms rather than being guessed.**
CPG/TPG is cheap throughout: 640 throat cells in ~5 s, `running_line_shift` 11 ms,
`throat_walk` 9 ms, a K=16 `stage_throat_margin` 6 ms. On the **equilibrium** gas
`stage_throat_margin` stays cheap (**0.1–2.4 s** per cell) because it contains no scan, while
**one** `stage_incidence_schedule` row costs **36.9 s** — slice M's `_scan` cost, unchanged.
So the equilibrium arm covers `stage_throat_margin` **only**, and the schedule's equilibrium
coverage is **deliberately absent with that number beside it**; a two-cell schedule arm would be
*a bar over unmeasured cells*, which is the thing § 5.8.1 (i) forbids.

**(vi) STACK CONSTRUCTION — the number that decides the `Descendant` shape.** A `K=8` match builds
the stacks **twice** (constructor only) and then runs **6 464 marches** and **64**
`StageStack.solve_n` calls against them; `capacities()` is built 120 times against **4 360** cache
hits.

> **CORRECTED BY STEP 4, LIKE (iii) — THESE ARE `probe_n1`'s NUMBERS AND `probe_n1` IS NOT THIS
> GRID.** The per-match figures (2 / 6 464 / 64) are one `K=8` cell at `Tt4` = 1200; the
> `120 / 4 360` is the whole 240-cell liveness sweep, which runs `K ∈ {2,4,8}` with no
> `cap_profile` axis. **Two grids in one sentence.** On the dump's 640 cells: **320 stacks built,
> 320 capacity profiles built against 24 000 cache hits.** Step 3's FINDING 2 leaned on
> `120 / 4 360` as the witness that there is exactly ONE stack object per spool per matcher — that
> argument is unaffected (it is about the ratio being large, not about its value), but the numbers
> it quotes are a different sweep's. A correction attached to one section leaves the other
> standing, which is step 1's own lesson.

#### The pre-registered predictions

* **P1 — THE CAUGHT SCOPE NEEDS A TWO-REASON FALLIBLE TWIN, AND NOTHING ELSE DOES.**
  `StageStack::try_solve_n` returns `Err` on the bracket and on the clamped root; every other
  assert in rungs 55/56 stays a panic. *Refuted by:* any Rust abort reaching
  `stage_incidence_schedule`'s scan from a third frame, or either reason never firing on the dump
  grid.
* **P2 — THE GATED-CODE REVERT UNIT IS EXACTLY THREE EDITS IN ONE FILE.** `Descendant` loses
  `Copy`; `r53_at_setting` gains `.clone()`; `VariableStatorCore::with_hooks` takes a
  `&'static TwoSpoolHooks`. Baseline: `cargo test --release` re-runs **bit-identical** — every
  value oracle and every suite, at the pass/fail counts RECORDED as step 1's first action rather
  than quoted here. **The RIPPLE was counted rather than hoped, because § 5.9 (a) got burned on
  exactly this** — it enumerated three source files and the unit turned out to be four, the fourth
  a *gate*. `VariableStatorCore::with_hooks` has **three** call sites in all: its definition,
  `new`'s `Self::with_hooks`, and `r53_at_setting` — **all three in `stator.rs`**, and no test file
  calls it. *Refuted by:* one changed digit, or a fourth site anywhere.
  **(b) of § 5.9 applies unchanged — the oracles compare committed `include_str!`
  goldens, so a passing `cargo test` IS the bit-identity check.**
* **P3 — `R55` ENTERS BOTH TABLES, AND THE DISPATCH GATE MUST ASSERT IN BOTH DIRECTIONS.**
  A rung-53 core still satisfies `hooks.hp_eta_loop as usize == R39.hp_eta_loop as usize`; a
  **stacked** core does **not**. A one-directional gate passes on any table — slice M (e)'s
  `is_flat` failure mode. This discharges `rung53.rs::slice_m_deferrals` item 3. *Refuted by:*
  either direction failing, or the `Descendant` match ceasing to be exhaustive.
* **P4 — THE REDUCE IS AN IDENTITY AT `K = 1`, AND IT IS ALREADY MEASURED.** 120 comparisons of
  rung 56's `stage_throat_margin` against rung 54's `throat_margin` (`m`, `n`, `m_c_face`,
  `x_face`, and row 0's `m_c`/`throat_loading`/`c_min`/`area`/`capacity`): **zero mismatches**,
  with `amplification == 1.0` and `work_gap == 0.0` **exactly**, on both gases at three stator
  settings. *Refuted by:* any non-bit-equal field, or a non-exact 1.0/0.0.
* **P5 — `StageStack::solve_n` DISPATCHES AT `K = 1` AND IS THEREFORE BIT-FOR-BIT, NOT MERELY
  TIGHT.** *Refuted by:* the Rust computing the one-stage march instead of calling
  `ComponentMap::solve_n`.
* **P6 — THE ARGMIN TIE-BREAK IS A GATE, NOT AN IDIOM.** The 13 design-throttle cells are
  reproduced index-for-index, **and the rule is pinned on a CONSTRUCTED exact tie as well** — a
  hand-built row vector with several bit-identical minima, asserted to return the FIRST. On the
  measured cells alone the rule would be *incidentally* satisfied rather than pinned, which is the
  self-comparison failure mode. *Refuted by:* any index differing while the values agree to the bit
  — which is exactly the failure a value oracle cannot see.
* **P7 — THE `vsv_stages` SPLIT SURVIVES THE PORT AS A COUNT.** 120/160 reached, the 40 misses all
  on `vsv_stages = None`, none at `Tt4 = 1500`. *Refuted by:* any front-row-lever row failing to
  reach, or a lumped-lever miss at design.
* **P8 — `_P_FLOOR` NEVER FIRES AND `_T_FLOOR` ALWAYS CAN.** *Refuted by:* a `_P_FLOOR` firing on
  the dump grid (which would make the source's own shared counter load-bearing after all).
* **P9 — THE ONE-SIDED STACK IS A CONTROLLED EXPERIMENT, AND THE CONTROL IS ONE-WAY — MEASURED,
  TWO-SIDED, AND IT REPRODUCES RUNG 39's ARROW FROM A NEW LEVER.** The first draft of this bar
  asserted the dispatch (`stack_hp is None` ⇒ `super()._hp_eta_loop`) and then *inferred* that the
  HP fields stay put, which is a claim about the converged point and was not measured. Measured,
  on 40 points (2 gases × 5 shapes × 4 throttles):

  | armed | HP fields (`n_hp`, `phi_hp`, `eta_hpc`, `pi_hpc`) | LP fields | thrust |
  |---|---|---|---|
  | `K_lp = 8, K_hp = 1` | **bit-identical, 40 of 40** | move (24–35 of 40) | moves 24/40 |
  | `K_lp = 1, K_hp = 8` | move (38 of 40) | **move, 24 of 40** | moves 24/40 |

  So stacking the LP spool is a controlled experiment on the HP one and **the reverse is not** —
  which is rung 39's own headline (*the map opens ONE arrow HP→LP; `π_LPC` cancels*) arriving from
  a lever rung 39 never had. `thrust` is excluded from the invariant clause on purpose: it is a
  whole-engine quantity and moves in both arms. *Refuted by:* any HP field moving in the first
  arm, or the second arm's LP fields NOT moving (which would make the bar vacuous rather than
  wrong).
* **P10 — RUNG 55's `lp_disabled` REFUSAL IS UNREPRESENTABLE, LIKE RUNG 53's.** Rust has no such
  parameter, so `assert not (lp_disabled and K > 1)` has nothing to witness. Booked in
  `slice_n_deferrals`, **not owed** — the type-level refusal is strictly stronger (slice M's
  precedent, second use).

#### Module decision, and sizing

**A new `stage.rs`**, carrying `StageStack` (the ladders, the march, the two solvers, rung 56's
per-row throat) and `StageStackCore` with its `R55` entries in **both** tables. `stator.rs` is
1 196 lines and slice K's split bar was 2 025; rung 55/56's ~685 Python lines land near 900 Rust,
so folding them in would cross it. The three module-level helpers `_mfp_frac` / `_nu_of_M` /
`_M_of_nu` go into `map.rs` beside `design_throat_mach`, which is the relation they were factored
out of and the only existing consumer.

**`Descendant::Stack` CARRIES THE BUILT STACKS, and the rejected alternative is recorded with its
number so slice O does not re-litigate it.** The alternative — keep `Descendant` `Copy` by storing
only the six scalars and rebuilding the stack on demand — is *correct* (the bisection is
deterministic, so the bits are identical), but at `K = 8` it rebuilds once per `solve_n` call
against 6 464 marches, **≈ 50 % on a match**, and it makes the measured `capacities()` cache
(120 built / 4 360 hits) vacuous. The other alternative — `stack_lp`/`stack_hp` as fields on
`VariableStatorCore`, the rung-42 `bleed`-on-the-shared-core precedent — is refused because
`Descendant` was invented **in slice M** precisely to stop descendant state landing on rung 53's
struct.

**THE CAPACITY CACHE IS LAZY, AND THE MEASUREMENT FOUND A SECOND ASSERT THAT SHADOWS THE FIRST**
(`probe_n4.py`, raised in review of step 1). The eager build is the obvious Rust move once
`Descendant` is `Clone` — every reader takes `&self`, so a `Vec` filled in the constructor costs
nothing to hand out. It is wrong, and § 5.10 (ii)'s own census is what witnesses it: 80 of the 160
schedule rows are built with `capacity = False` — **step 4 measured 160 of 160**, since
`probe_n3` sets it on every matcher, so the hazard is WIDER than this note claimed and the
conclusion stands a fortiori — so an eager profile panics where Python is silent,
and the tempting repair — add `.with_capacity()` to those maps — would **silently widen the port's
grid past the source's**. Measured on a capacity-free map:

| call | Python |
|---|---|
| `StageStack(K=8, …)` construction | **OK** (`_C_ks` is `None`) |
| `march` / `tau_of` / `solve_n` / `lumped_tau` | **OK** |
| `stage_throat_ratio(k)` / `stage_throat_loading(k, m_k)` | **OK** — rung 56's *area* half needs no `C` |
| `capacities()` / `stage_capacity(k)` / `stage_capacity_margin(k, m_k)` | **RAISES**, at FIRST READ |

So the cache is a `OnceCell<Vec<f64>>`-shaped thing, not a constructor field, and the assert lives
at first read. **AND THE STACK'S OWN ASSERT IS UNREACHABLE THROUGH THE MATCHER.** Through
`StageStackMatcher` the same capacity-free maps give `match`, `stage_margin`, `work_gap` and
`stage_incidence_schedule` all **OK**, and `stage_throat_margin` raises **from its own `cmap`
assert**, before it ever touches the stack. Two asserts carry the same sentence and the outer one
always wins — so a Rust gate driven through the matcher would gate the matcher's guard while
reading as though it gated the stack's. `StageStack::capacities()` is called **directly** on a
capacity-free stack instead: *a documented gate that doesn't exist*, caught before it was written
rather than after.

**AND `r53_at_setting`'s `.clone()` IS RIGHT FOR `Plain` AND WOULD BE WRONG FOR `Stack`.** A cloned
stack carries ladders built from the OLD map at the OLD setting — precisely the failure this
module's own note predicted when it made `at_setting` a hook (*"a copy-and-swap in rung 53's body
would leave slice N with a stack built at the wrong setting"*). It is safe only because `R55`
overrides `at_setting` and rebuilds, and **P3's table-identity gate cannot see that**: an `R55`
whose `at_setting` entry was left pointing at `r53_at_setting` still satisfies every fn-pointer
comparison P3 makes on the *eta-loop* entries. The discriminator is a value, not a pointer — and
**measuring it found that most of the obvious choices are VACUOUS.** At `vsv_lp` 0.0 → 0.20 on a
`K = 8` stack:

| read | base | moved | usable as the discriminator? |
|---|---|---|---|
| `stack_lp.cmap.vsv` | `0.0` | **`0.2`** | **YES** — the only field that moves |
| `stack_lp.cmap_axial.vsv` | `0.0` | `0.0` | no, by construction (`replace(cmap, vsv=0.0)`) — but it PAIRS with the row above into a two-sided bar |
| `stack_lp.theta_d[1]` | 1.051213625922528 | 1.051213625922528 | **NO — bit-identical** |
| `stack_lp.e_d` | 0.91239605960857439 | 0.91239605960857439 | **NO — bit-identical** |

The design ladder is map-INDEPENDENT (it is built from `tau_d`/`pi_d`/`eta_d`/`kc`, none of which
the stator touches), so a "the stack was rebuilt" gate written on `theta_d` or `e_d` would pass on
a stack that was never rebuilt at all — *a ported test can go VACUOUS*, caught before writing. The
bar is the pair (`cmap.vsv == moved` **and** `cmap_axial.vsv == 0.0`), and it joins P3 rather than
replacing it.

**The one shape decision that decides a gate**, stated before it can be got wrong: rung 56's
per-row read **must not** be factorised into rung 54's face read. `stage_throat_margin` at `K = 1`
takes the `stack is None` branch and calls `cmap.throat_ratio()`/`throat_loading`/`capacity_margin`
**verbatim**, which is what makes P4 an identity rather than an algebraic re-derivation — slice
D/E's *an "exactly" claim survives a copied instruction sequence and dies on a second derivation*.

#### The steps — SIX, and step 1 is again the whole revert unit

| step | scope | gate |
|------|-------|------|
| **1** | **ALL changes to already-gated code**: `Descendant` loses `Copy`, `r53_at_setting`'s `.clone()`, and the `&'static TwoSpoolHooks` threaded through `VariableStatorCore::with_hooks`; plus the three `map.rs` helpers | ✅ **P2 HELD** — see below |
| **2** | `stage.rs`: `StageStack` — the two design ladders, `_stage_eta`, `march` with BOTH floors counted, `tau_of`, `lumped_tau`, `solve_n` + `try_solve_n` (P1), and rung 56's `capacities`/`stage_*` row reads | ✅ **`slice_n_smoke.rs`, 1 337 keys bit-exact on SEVEN cells + 10 non-value gates**; crate **546 run, 0 skipped** — see below |
| **3** | `StageStackCore`: `R55` in both tables, `at_stages`, the two stacked eta loops, `stage_margin`, `stage_throat_margin`, `throat_walk`, `work_gap`, `running_line_shift`, `stage_incidence_schedule`; the P3 dispatch gate discharging `slice_m_deferrals` item 3 | ✅ **crate 551 run, 0 skipped**; `slice_n_smoke.rs` still 1 337 keys bit-exact, 11 → 14 gates; `rung53.rs` 24 → 26 — see below |
| **4** | the slice-N oracle — Python dump LAUNCHED FIRST, Rust reader written while it runs; the (i)/(iii)/(iv)/(vi) census bars, and the equilibrium arm on `stage_throat_margin` only | ✅ **`slice_n_oracle.rs`, 72 520 keys bit-exact first run** + a 5 649-key equilibrium arm + a 41 560-key CPython arm; 5 gates, crate **556 run, 0 skipped**. (i)/(iv) reproduced EXACTLY; (iii)/(vi) were measured on the PROBES' grid, so the census is EMITTED and compared — see below |
| **5** | the two suites, `rung55.rs` (18) + `rung56.rs` (21), incl. P4/P5's reduce contracts, P6's tie-break and P8's per-floor split | ✅ **`rung55.rs` 20 + `rung56.rs` 23 = 43 gates**, crate **556 → 599 run, 0 ignored**, as a `--list` **name diff: 43 additions, 0 removals**. Four detectors measured; TWO of the source's own gates found vacuous, and a FIFTH gated-code edit landed here — see below |
| **6** | the source corrections this slice owes (§ (iii)'s dead constants, if the specs assert otherwise) | ✅ docs-only, **no gate**. The dead constants turned out NOT to be asserted anywhere in the two specs; what the slice actually owed was **three vacuous gates and one over-stated reason**, all found at step 5 — `docs/rung55-spec.md` § Verification gates + § the row-count table, `docs/rung56-spec.md` § Verification gates |

##### STEP 1 — SHIPPED. P2 held in BOTH currencies, and the count that did not reconcile was a FINDING

**P2 predicted three edits in one file and a bit-identical re-run; both halves held.** The
code-only diff (comments stripped) is exactly `Descendant`'s `Copy` → `Clone`, `with_hooks`'s
second table, `r53_at_setting`'s two additions, and the three new `map.rs` helpers — **no fourth
site anywhere**, which is the clause § 5.9 (a) got burned on.

| | pre-edit | post-edit |
|---|---|---|
| `cargo test --release` | exit **0** | exit **0** |
| binaries | 60 | 60 |
| passed / failed / ignored | — | **534 / 0 / 1** |
| `--list` names | **535** | **535**, `diff` EMPTY |

**AND THE COUNT-VS-COUNT CHECK IS WHY THE NAME DIFF WAS RUN AT ALL.** The first baseline was
captured through `| tail -80`, which kept the exit code and ate every per-target total — so the
recovery was an inventory of test NAMES rather than a number, which is the stronger instrument and
the one this project's own rule already demanded (*coverage is a name → parameter-set diff, never a
count*). Exit-0 proves nothing FAILED; only the diff proves nothing VANISHED.

**THE ±1 WAS THE FINDING.** 535 listed against the 534 the slice-M ledger shipped: `--list` counts
an `#[ignore]`d test and a run does not. The crate had **exactly one**, `rung31.rs`'s
`gate4_running_line_and_direction`, carried over by slice I as the port's spelling of
`@pytest.mark.slow` (`tests/test_rung31.py:120`). **Slice M step 5 retired that mapping** — *port
the gate, DROP the marker, re-introduce `#[ignore]` only against a MEASURED cost* — but retired it
going FORWARD, so the one pre-existing instance kept deselecting itself for two slices while the
rule that forbade it was already in the repo. Measured: **2.27 s**, in a gate whose slowest single
target is **246 s**. Marker dropped; `rung31.rs` is now 8 passed / 0 ignored and the crate is
**535 run, 0 skipped**. *A correction applied only forward is a correction that leaves its own
precedent standing* — and the only thing that surfaced it was a count that refused to reconcile.

##### STEP 2 — SHIPPED. The value dump held first try; every finding came from the gates BESIDE it

`stage.rs` carries `StageStack` whole: both work splits, both capacity profiles, the lazy per-row
capacity cache, the two-reason fallible twin, and rung 56's four row reads. The dump —
`slice_n_smoke.rs` against `oracle/slice_n_smoke_pypy.tsv` — is **1 337 keys bit-exact on the first
run**, over seven cells chosen by ENUMERATING the methods step 2 ships rather than by picking a
representative one: `K` of 8/8/4/1/8/4/16, both splits, both profiles, the lumped and the front-row
lever, a closed stator and an opened one, and `solve_n` on **both** the `K = 1` dispatch and the
`K > 1` bisection. That enumeration is what caught the module's dead branch: **a stack with
`vsv_stages = None` never touches `cmap_axial`**, so a smoke built only on the default lever
leaves both second branches of `psi_at`/`vsv_at` unwitnessed.

**THE INSTRUMENTS SHIPPED WITH THE CODE, NOT WITH THE GATES THAT READ THEM.** § 5.10's (iii),
(vi) and P8 can only be observed from inside these functions, so `take_census` — bisection passes
in both loops, marches, `solve_n` calls, constructions, cache builds/hits, and a PER-FLOOR split
of the clamp counter — is in `stage.rs` from the start. Retrofitting it at step 4 would have meant
editing step-2 code that step 3 was already built on, which is § 5.9 (a)'s ripple exactly. It
immediately paid: `stage_eta` **48** passes, `try_solve_n` **48**, and a `solve_n` call costs
**51 marches** — 2 bracket endpoints, 48 residuals, and the one extra march the clamped-root check
runs. Both 48s are *predictable from the arithmetic* (absolute break over a fixed bracket), like
`map.rs`'s, so they are gated rather than merely recorded.

**FINDING 1 — `_P_FLOOR` IS DEAD FOR A DERIVED REASON, AND § 5.10 (iii) HAD ONLY THE GRID ONE.**
The pre-registration recorded *0 firings in 521 649 marches*, which is a claim about a sweep.
`tau_k` is floored **before** `base = 1 + e*(tau_k − 1)` is formed, so `base ≥ 1 − e*(1 − T_FLOOR)`
and `base < P_FLOOR` requires

```text
    e > (1 − P_FLOOR)/(1 − T_FLOOR) = (1 − 1e-6)/(1 − 1e-3) = 1.001   EXACTLY
```

— a threshold in the two floor constants **alone**: independent of the map, the split, `K` and
the design point. Since `e = e_d*(eta_live/eta_d)`, that sits just above any physical live
efficiency, which is *why* the count is zero. Pushed past it deliberately, both floors fire on the
**same** stages and the shared counter reads exactly double (7 → 14 at `K = 8`, 4 → 8 at `K = 4`)
— § 5.10 (iii)'s *the two can never be conflated* shown rather than asserted. **A dead guard's
threshold is worth more than its count**, and the count is what a sweep gives you.

**FINDING 2 — THE FIRST FLOOR GRID CLAMPED NOTHING, AND THE GUESS IT WAS BUILT ON WAS BACKWARDS.**
It probed `march(0.9, 0.1)` on the reading that `solve_n`'s LOW bracket end is the non-physical
one. It is the HIGH `m/n` end: at `n = 0.1` the `n_k²` factor is tiny so `tau_k` stays near 1 and
**nothing** clamps, while `march(8, 2)` drives **7 of 8** stages to `_T_FLOOR`. Had the grid been
shipped as first written, the whole clamp branch — and with it the fallible twin's rarer abort
reason — would have read as covered while never executing once.

**FINDING 3 — THE ARGMIN TIE IS AT THE DESIGN SETTING, AND THE FIRST GATE ASSERTED IT SOMEWHERE
ELSE AND WAS REFUTED BY ITS OWN NUMBERS.** § 5.10 (iv) locates the degeneracy at
`phi_k = 1`; the first gate looked for it on a cell carrying a MOVED stator, where `psi(1) ≠ 1`,
so the rows separate by whole percent (`[0.140, 0.243, 0.331, 0.348]`) and there is no tie at all.
Measured at the design setting on the uniform profile:

| | rows `0..K−2` | last row | argmin |
|---|---|---|---|
| `K = 8` | **bit-identical, all 7** | LOWER by one step in the last place | **7** |
| `K = 4` | **bit-identical, all 3** | HIGHER by one step | **0** |

The whole spread is `< 1e-15` and the argmin lands on **opposite ends of the stack** depending
only on which way the march's own `th *= tau_k` accumulation drifted from the ladder `theta_d` it
divides by. Both cells are in the dump, so the index is compared against PyPy rather than
restated. And **the tie-break RULE is pinned on a constructed tie, not on these cells** — cell F's
argmin is 0 because row 0 is genuinely tied-and-first and cell E's is 7 because row 7 is genuinely
smaller, so neither run discriminates *first-of-equals* from *last-of-equals*; the `<=` fold that
would return the LAST is written out beside it and asserted to differ (§ 5.10 P6's
self-comparison clause).

**FINDING 4 — THE FILE'S ONE POWER-SPELLING DECISION WAS ASSERTED, NOT GATED, AND THE CELL MEANT
TO GATE IT WAS NEARLY BLIND.** `_ladder_T`'s `"tau"` arm is the **only** place in rungs 55/56 where
Python raises to a VARIABLE integer exponent (`r ** k`); every other `**` in the two rungs is `0.5`
or `kc`, neither of which has an alternative spelling. So it is the file's single genuine choice —
`pow(r, k)` against a running product against the tempting "simplify the two powers into one"
`tau ** (k/K)` — and the first draft's doc comment claimed the smoke grid measured it. It did not.
Scanned over **109 650** `(tau, K, k)` cells the spellings differ on **34.8 %** and **65.5 %**
respectively, but that rate does not transfer to a chosen cell: at cell B's own `(tau_lp, K = 8)`
only rows 7–8 separate `pow` from the product, **by one bit**, and at `(tau_lp, K = 4)` — the
neighbouring choice — **nothing separates them at all.** A cell G at `K = 16` was added (8 rows
against the product, 14 against the single power), and then **the detector was measured rather
than trusted**: re-spelled as a running product, the dump fails at `cellB/theta_d/7`,
`0x3ff595ff5c0b5e4d` vs `…4e`. *Exactness bounds the CELLS visited, not the RULES discriminated* —
slice J's lesson, and the rate over a swept grid is not the rate at the cell you shipped.

**FINDING 5 — STEP 1's OWN LESSON CAME BACK THE NEXT DAY, FROM THE OTHER DIRECTION.** The full
run came back **545 passed / 0 failed / 1 IGNORED**, on a crate step 1 had just cleared to
*535 run, 0 skipped*. The skipped item was a ```` ```ignore ```` doc-block in this step's own
`StageStackSpec` note — the crate has **zero** doc-tests and spells all 42 of its other code
blocks ```` ```text ````, so the one deviation both broke the convention and put a deselected
item back in the ledger. It is now `text`, and the crate is ~~**545 run, 0 skipped**~~ **546 run, 0 skipped** — see step 3's postscript, which measured it. Step 1's
finding was *a correction applied only forward leaves its own precedent standing*; this is the
mirror — **a ledger you have just cleared is the easiest one to dirty again**, and the only thing
that caught it was reading the ignored column of a run whose exit code was 0.

**P1's TWIN GOT ITS FIRST ARM EXECUTED, AND THE SECOND ONE BOOKED.** `try_solve_n` shipped with
two `Err` returns — P1's entire content — and the first draft reached **neither**, which is slice
L step 3's shape at the step that introduces the machinery. The **bracket** arm is now gated,
from the same numbers the floors were measured with (at `m = 8` the stack cannot do the design
work at either bracket end, so no sign change exists), together with the assertion that the
panicking half produces the *same string*. The **clamped-root** arm — § 5.10 (i)'s 1-in-40 — needs
a root that exists whose march still clamps; that is booked in `slice_n_deferrals_so_far` and owed
by step 4's oracle, where the dump grid produces it naturally.

**Three deferrals booked** (`slice_n_deferrals_so_far`, which step 5's suites take over): the
`split` and `cap_profile` string asserts are **unrepresentable** — both are enums, so there is no
invalid value to reject — `0 <= vsv_stages` is half unrepresentable, `usize` carrying the lower
bound while the `<= K` half stays a live, gated assert; and the clamped-root `Err` arm above.

**Rules slice M established that slice N inherits, so they are not re-litigated:** port the gate
and **drop the `slow` marker** — Python marks 6 of these 39 `slow` (5 + 1), and `#[ignore]` returns only
against a MEASURED cost; coverage is a **name → parameter-set diff**, never a count (slice M's
22 + 21 became 24 + 25 from four forced `#[should_panic]` splits, and rung 55's
`test_capacity_style_guards_reject_nonsense` plus rung 56's profile refusals will split the same
way); and `ComponentMap::phi_max` stays booked to **phase 6**, unchanged from slice M's ledger.

**ONE MORE GATE IS OWED TO PHASE 6, AND THE FIRST DRAFT BOOKED IT UNDER THE WRONG NAME.** Rung 55's
`test_cycle_untouched_transient_ladder_is_bit_for_bit_unstacked` was written here as a `phi_max`
consequence. It is not: read, the gate runs a **rung-43 fuel transient twice** — once before a
stage stack is live on the same hardware, once after — and demands the two point lists are
`==`. Its content is *absence of leakage*, and what makes it unportable today is that
`TwoSpoolFuelTransient` **does not exist in Rust yet** (phase 6), not `phi_max`. Booked in
`slice_n_deferrals` under that reason, with the assertion quoted, so phase 6 does not re-derive
it — and recorded here because *a deferral filed against the wrong cause is a deferral nobody can
discharge.*

##### STEP 3 — SHIPPED. The carrier lesson repeated ONE LEVEL DOWN, and the plan's own step table is what it refuted

`stage.rs` now carries `StageStackCore` whole: `R55` in the stator table, `R55_TWO` in rung 39's,
the two stacked efficiency loops, `at_stages`, and all six reading methods (`stage_margin`,
`stage_throat_margin`, `throat_walk`, `work_gap`, `running_line_shift`,
`stage_incidence_schedule`). Measured: the crate is **551 run / 0 failed / 0 ignored** over
61 targets; `slice_n_smoke.rs` is still 1 337 keys bit-exact and grew 11 → 14 gates, and
`rung53.rs` grew 24 → 26.

**FINDING 1 — THE FOURTH GATED-CODE EDIT LANDS AT STEP 3, IN A FILE STEP 1 NEVER OPENED, AND IT
IS § 5.10's OWN HEADLINE ONE LEVEL DOWN.** § 5.10's step table called step 1 *"ALL changes to
already-gated code"* and P2 enumerated three edits in `stator.rs`. **P2's literal text survives**
— `VariableStatorCore::with_hooks` still has exactly three call sites, all in `stator.rs`. The
table row does not: `TwoSpoolMapCore` gains `stack_lp`/`stack_hp` in `two_spool.rs`.

The reason is structural and was visible in the signature all along. Rung 55's real overrides are
the two **efficiency loops**, whose hook `self` is `&TwoSpoolMapCore` — the INNER core. It has no
path up to `VariableStatorCore::descendant`, where slice M put the descendant state. So the built
stacks must hang off the inner core, on `bleed`'s precedent, whose doc comment already states the
crate's rule: *the port's `fn`-pointer tables put the descendant's state on the shared core rather
than on the leaf.* § 5.10 asked what carrier `at_setting` needs and **never asked what carrier the
next hook needs**. It is § 5.9 (a)'s burn again (three source files enumerated, the unit was four)
and this slice's own lesson recursing: **a carrier claim checked on ONE hook says nothing about
the next hook's.**

**FINDING 2 — `Descendant`'s `Copy` DROP WAS NOT FORCED, AND § 5.9 (c) IS REFUTED FOR A DIFFERENT
REASON THAN STEP 1 RECORDED.** With the stacks on the inner core, `Descendant::Stack` carries
exactly the six scalars (c) predicted — `k_lp`/`k_hp`/`split`/`vsv_stages_*`/`cap_profile` — every
one of them `Copy`. So on *that* type (c)'s prediction held, and step 1's `Copy` → `Clone` edit
bought nothing. It is **left as `Clone` rather than reverted** (strictly weaker, nothing reads it
as a claim, and a revert is churn in gated code), and `stator.rs`'s doc comment — which
attributed the drop to `Vec<f64>` ladders that will never live there — is corrected in place. The
carrier decision is also what makes the census honest: `StageStack`'s per-row cache is a
`OnceCell`, and § 5.10 (vi)'s **120 built / 4 360 hits** is only Python's number if there is
exactly ONE stack object per spool per matcher for the readers to share. Two by-value copies —
one on the enum, one on the core — would let readers split across them and the build count would
stop being the source's.

**FINDING 3 — THE PRE-FLIGHT GOT THE `_INC_MAX` SHADOW RIGHT AND THE SLICE THAT PORTED THE CODE
GOT IT WRONG, IN THE EXACT SPELLING THE PRE-FLIGHT FORBADE.** § 5.3 item 6 identified it
correctly and prescribed the fix: *"`StageStackMatcher._INC_MAX = 200` shadows
`VariableStatorMatcher._INC_MAX = 80` ... In Rust that cap must be a per-cell parameter, never a
literal in the ported body."* Slice M shipped `for _ in 0..Self::INC_MAX` — a literal in the
ported body, in both loops — and its doc comment attributed the shadow to **rung 61**. Read:
`StatorBleedMatcher` declares `_B_TOL`/`_B_MAX`/`_B_CAP`/`_B_STEP` and **no** `_INC_MAX`;
`StageStackMatcher` declares it at `engine.py:7282`. So the shadow was live at slice N all along,
and the comment pointed the next reader at the wrong slice to find it.

**What licensed the slip is § 5.9 (iv), and it is TRUE — the two senses of "live" are different.**
(iv) measured the cap **never reached**: rung 53's ladder ends on `|r| <= _INC_TOL` at 30–36
passes and rung 54's bracketed root at 26–33, on all 42 + 54 roots. That is a claim about whether
the cap is *hit*, and from it "the shadow is not live" reads as settled. But a shadow that cannot
change a number still decides which constant the body names, and a body that names the wrong one
is wrong wherever the grid later moves. **A dead constant's SPELLING still has to be right** —
the same shape as this slice's own *a dead guard's threshold is worth more than its count*.

Ported as `VariableStatorCore::inc_max()`, a two-arm read on the descendant tag — Python's MRO
decision expressed as data — and read from all three sites: rung 55's own
`stage_incidence_schedule` and the two inherited rung-53/54 loops. **No value moves**, exactly as
(iv) predicts, so the gate asserts the dispatch and not an outcome.

**And it is LATENT on the shipped suite besides**, which is the sharper half:
`test_rung55.py:438` runs `incidence_schedule` on a genuine rung-53 matcher, so no shipped Python
test drives an inherited schedule on a stacked object. A wrong cap here would be invisible to
every value oracle the port will ever build. Rung 55's other two constants were checked and are
NOT shadows — `_INC_TOL` is a re-declaration at the same 1e-12, and `_V_SCAN = 0.05` is a NEW name
beside `_V_STEP = 0.04`, not a redefinition of it.

**FINDING 4 — THE ARITY PIN FIRED, WHICH IS THE ONLY WAY A READER LEARNS IT WAS LOAD-BEARING.**
`rung53.rs::slice_m_deferrals` shipped an exhaustive `match` over `Descendant` with a comment
saying a second variant would break compilation. It did — `error[E0004]: non-exhaustive patterns:
Descendant::Stack { .. } not covered`, before a single new gate had been written. A pin that has
never fired and a pin that cannot fire read identically in the source.

**P3's GATE NEEDED THREE CLAUSES AND A VALUE HALF, AND BOTH HALVES WERE MEASURED RATHER THAN
TRUSTED.** `the_stacked_dispatch_is_live` discharges `slice_m_deferrals` item 3. Three pointer
clauses, because a one-directional gate passes on ANY table (slice M (e)'s `is_flat` failure): a
rung-53 core's eta loops **are** `R39`'s, a stacked core's are **not**, and a stacked core's
`try_match_point` **is** `R39`'s — that third clause catching an accidental `match` override, the
failure that would make rung 55 a new matcher rather than rung 39's with one inversion swapped.
It is true by construction as well as by assertion, because `R55_TWO` names
`R39.try_match_point` rather than re-spelling it (and `r39_try_match_point` is private, so
reaching it would have been a FIFTH gated-code edit).

Then the half the pointers cannot see: **an `R55` whose `at_setting` entry was left pointing at
`r53_at_setting` satisfies every pointer clause above** and hands back a sibling with
`stack_lp: None` — a silently UNSTACKED machine producing plausible numbers. § 5.10 measured which
reads discriminate it; both failure modes were then INJECTED and the gate watched to fail:
pointing `R55.at_setting` at rung 53's body fails the `assert_ne!`, and a rebuild that skips the
stack construction fails at *"the sibling must still be STACKED"*. The vacuous reads are asserted
EQUAL in the same gate (`theta_d`, `e_d` are bit-identical across a stator move, because the
design ladder is map-independent), so the vacuity is recorded rather than rediscovered by whoever
writes a weaker gate next.

**AND THE SIX READERS WOULD OTHERWISE HAVE SHIPPED UNEXECUTED.** Step 4 owns the oracle and step 5
the suites, so nothing in the crate called `stage_margin`, `stage_throat_margin`, `throat_walk`,
`work_gap`, `running_line_shift` or `stage_incidence_schedule`. That is slice L step 3's shape —
*my smoke check witnessed 1 of the 3 methods the slice's own headline names* — pre-empted rather
than repeated: `every_step3_reader_is_reached_and_the_k1_branches_are_exact` enumerates the six and
asserts only what is STRUCTURAL, i.e. what holds by which BRANCH runs rather than by arithmetic
agreeing — `rear_excess` and `work_gap.gap` exact zeros at `K = 1`, `amplification` exactly 1.0,
`running_line_shift` an exact zero against its own `at_stages(1, 1)` sibling. P4's field-for-field
identity and P5's dispatch stay step 5's; stating weakened versions of them here would be the
*ported test goes VACUOUS* trap in reverse — a weak gate standing where the strong one is owed.

**ONE DIVERGENCE RECORDED, NOT REPAIRED.** Rung 55's efficiency-loop non-convergence is an
`AssertionError` in Python, and `stage_incidence_schedule` catches `AssertionError` — so Python
would swallow it as a map-validity edge. Rust's is a `panic!` (rung 39's own spelling) and is not
catchable. § 5.10 (i) measured the caught frames on 40 firings and this was not among them (39
bracket, 1 clamped root, 0 elsewhere), so it is unobservable on the grid — but P1's refutation
clause is *"any Rust abort reaching the scan from a third frame"*, and this is where a third frame
would come from. Written into `r55_hp_eta_loop`'s doc so step 4's oracle can attribute it if it
ever appears.

**TWO GATES CAME OUT OF REVIEW, AND ONE OF THEM COVERS THE ONLY SPELLING IN STEP 3 THAT COULD HAVE
DIVERGED SILENTLY.** `stage_margin` computes each row's stall floor as
`cmap.phi_surge / (1 + v_k*cmap.phi_surge)` — off the map's `phi_surge` FIELD, which by rung 53's
rule *is* the design-setting anchor — and **not** via `ComponentMap::phi_surge_at`, which reads the
MAP's own `vsv`. On the lumped lever the two agree for every row and the choice is invisible. On
the FRONT-ROW lever at a moved setting they do not: the rear rows carry `v_k = 0` while
`cmap.vsv != 0`, so the map-level reader would put the rear rows' floor at the front rows' value.
The reachability gate reads `m_i` / `worst` / `rear_excess` and would never have seen it. Gated
now, and the detector MEASURED: re-spelled as `phi_surge_at()`, the new gate fails on the rear
row's floor. That is step 2's own lesson — *a smoke grid on the default lever leaves the second
branch dead* — arriving one method further along.

The second is wording, and it is the same failure with the sign flipped. `slice_m_deferrals` item
3 covered `StatorHooks` dispatch for **both** coming descendants; the first draft struck it whole.
Rung 61's override does not exist yet, so it is now marked *discharged for rung 55, still owed for
rung 61* — **a deferral closed on behalf of a rung that has not shipped is as unfindable as one
filed against the wrong cause.**

**POSTSCRIPT — STEP 2 SHIPPED THE SAME COUNT THREE TIMES AND GOT IT WRONG ONCE, INSIDE ITS OWN
SECTION.** Its table row says *546 run, 0 skipped*, its prose says *545*, and the same commit's
`MEMORY.md` says *546*. Measured here: **551** at step 3, against `#[test]` counts taken from git
at HEAD — `rung53.rs` 24 → 26 and `slice_n_smoke.rs` 11 → 14, exactly +5 — so the baseline was
**546** and the prose sentence is the odd one out. Three hand-written copies, none derived from
another, is a disagreement waiting to happen, and a majority vote is not what settled it: the
`#[test]` counts did. **Exactly one record should be a measurement and the rest should quote it**
— the rule this project already applies to a quoted gate TIME, arriving on a gate COUNT.


### 5.11 SLICE O (rung 61, `StatorBleedMatcher`) — PRE-REGISTERED, four probes MEASURED first

**The last slice in phase 5**, and the one § 6 named as the phase's structural risk before
§ 5.3's pre-flight discharged it. One rung, 306 source lines, 18 Python gates. Four steps, not
six — slice J's shape (one rung), not slice N's.

Probes: `M:\claud_projects\temp\slice_o\probe_o{1,2}.py` plus two inline checks.


##### STEP 2 — SHIPPED. **P1 IS REFUTED, BY A DEFERRAL SLICE L HAD WRITTEN DOWN AND DATED**

`rust/oracle/dump_slice_o.py` + `rust/tests/slice_o_oracle.rs`. **8 256 distinct keys bit-exact
against PyPy on the fast arm** (640 root-finder cells over 2 gases × 5 shapes × 4 throttles ×
4 settings × 2 spools × 2 targets) and **204 on the reacting arm** — the latter **first run, no
edits**. The fast arm needed the edit below before it could run at all.

**P1 IS REFUTED — a gated-code edit WAS needed, in `bleed.rs`, and it is not the one P1 was
about.** P1 predicted no `Descendant` variant, no new field, no gated-code edit; the first two
held (step 1), the third did not. Rung 42's bled LP efficiency loop called the PANICKING
`solve_n` while rung 39's twin called `try_solve_n`, so it could not be caught. Converting it,
propagating with `?`, and tallying the abort is the slice's ONE gated-code edit.

**AND SLICE L HAD ALREADY CALLED IT, IN THE FILE THE EDIT LANDS IN.** `bleed.rs`'s module note
says, verbatim: *"`lp_eta_loop_bleed`'s own `solve_n` is NOT converted: no rung-53/54 walk
reaches it, because rung 53's `at_setting` builds a valve-shut sibling. **Rung 61's
`StatorBleedMatcher` overrides `at_setting` precisely so the valve stays OPEN through every
sweep** — which puts this call site inside `_scan`'s catch for the first time. Slice O must
MEASURE that site, not inherit this paragraph."* Every clause of that is right, including which
rung would come due.

Three things follow, and the ordering of the first is the honest part:

1. **THE GATE FOUND IT BEFORE THE PARAGRAPH WAS READ.** The oracle panicked, the backtrace named
   `lp_eta_loop_bleed` eleven frames down, and only then did the module note turn out to have
   predicted it. A deferral written down is worth exactly the grep that finds it; what actually
   reaches you is a failure. That is an argument for **writing the deferral into the code the
   next slice will run**, which is where slice L put it — not for trusting that it will be read.
2. **NO RUNG-42 GATE COULD HAVE SEEN IT, AND NEITHER COULD A RUNG-54 ONE.** Rung 42's readers
   never walk until refusal; rung 54's walk never ran on an open valve. The defect exists in
   neither rung — **it is created by the COMPOSITION**, which is what rung 61 is. This is the
   first defect in the port located in an edge rather than a node.
3. **WHAT RETIRES A ZERO-FIRING COUNT IS A COUNT.** Slice L's verdict was *measured 0 firings*,
   so the discharge is `counters::lp_bleed_aborts()`, tallied at the raise and gated **> 0** in
   `slice_o_oracle.rs`. A comment saying "now it fires" would be the same species of claim that
   needed replacing.

**THE READ/KEY ACCOUNTING CLOSED ON TWO POPULATIONS, AND NAMING ONE WAS NOT ENOUGH.** The dump
writes 8 700 lines over 8 256 distinct keys. The first reconciliation pin claimed the only
duplicate was `goal` on a solved cell and **failed at 444 against 384**; the missing 60 are
`price_split`'s `gap_present`, emitted once explicitly and again inside its own
`(b_phi, b_m_phi, gap)` loop — and 60 is exactly `2 gases × 5 shapes × 2 spools × 3 settings`.
Slice L step 4's lesson, second instance: **a residual that does not close is a population you
have not found**, and only an exact bar says so — an approximate one absorbs all 60 silently.

**THE CENSUS BARS § 5.11 REGISTERED, ALL HELD ON THE ORACLE'S OWN GRID** (not restated from the
probe's — the cpg half of this grid *is* `probe_o1.py`'s, to the value):

| § 5.11 | bar | result |
|---|---|---|
| (i) | `_feasible` refusals | **0** of every call in 640 cells |
| (i) | `choked envelope` / `stator infeasible` branches | **0** each — both dead |
| (i) | `valve authority exhausted` + solved | **= 640**, the two live outcomes partition the grid |
| (ii) | `hi - lo <= 1e-15` arm | **0** — the dead arm of the compound condition |
| (ii) | `_B_MAX = 80` | never reached |
| (iii) | exact `b* == 0.0` | **0** — the truthiness trap is latent |
| (iii) | `ratio` present | **0** — every shipped row is mixed |
| (iv) | `b_last` present ⟺ exhausted branch | exact |

And one identity is asserted per cell rather than in aggregate:
`feasible = 2 + walk_steps + bisect_passes`, which is what makes the dumped count pin the PATH
and not merely the endpoint — a bisection reads only signs, so a walk that visits different
valve positions can still land on the same `b*` (slice N's vacuous-dispatch failure, in the one
place it would have applied here).


##### STEP 3 — SHIPPED. **TWO DETECTORS FIRED, AND ONE FIRED ON A GATE THAT WAS NOT AIMED AT IT**

`rust/tests/rung61.rs` — **23 gates from Python's 18**, all green. The extra five are three
`parametrize` families split into named gates plus the gas arms of gate 1 split into two, and the
ledger.

**ALL 23 PASSED FIRST RUN, WHICH BY THIS PROJECT'S RULE PROVES NOTHING**, so two defects were
injected into `stator_bleed.rs`:

| detector | what it removes | result |
|---|---|---|
| A | `with_b_cap` made a no-op | `gate8_the_ceiling_is_cap_dependent` **FAILS** |
| B | `r61_at_setting` reverted to rung 53's body (the sibling loses the valve) | `gate2_trap_at_setting_carries_the_bleed` **and** `gate6_seam_as_posed_valve_shrinks_the_stators_authority` both **FAIL** |

**DETECTOR B FIRING TWICE IS THE USEFUL PART.** Gate 2 is *aimed* at that defect; gate 6 is a
physics gate about the valve pre-spending the incidence budget, and it catches the same injury
independently because a valve-less sibling makes `authority_with_bleed`'s three rows identical.
An independent witness is worth more than a second assertion in the same gate — and it is the
converse of slice N's finding, where a gate that named a defect could not see it.

**ONE GATE IS STRENGTHENED PAST THE SOURCE, AND FOR THE VACUITY REASON.** Python's trap gate
asserts `isinstance(sib, StatorBleedMatcher)`. In Rust a sibling from `at_setting` is a
`VariableStatorCore` whichever table built it, so an `isinstance` port would compare a type to
itself — *a ported test can go VACUOUS*, caught before shipping rather than after. The
discriminant is the TABLE POINTER and the carried valve position, and those are what the Rust
asserts.

**THE TWO TOLERANCE-STYLE BARS ARE MEASURED AND RECORDED, NOT RE-FITTED:**

| gate | bar (the SOURCE's) | measured worst | headroom |
|---|---|---|---|
| 3, retention | `≥ 0.70` | **0.73339** | 4.8 % |
| 6, credit interaction | `< 0.03` | **0.01686** | 1.8× |

Recorded at each gate so a later change that erodes 0.733 to 0.71 reads as erosion rather than as
a pass — and deliberately NOT tightened onto the port's own measurement, which is how a gate stops
testing the claim it names.

**A SECOND REACH-INSIDE, FOUND AT THE START THIS TIME.** `test_rung61.py:370` writes
`m._B_CAP = cap` — slice N's `_V_SCAN` again, and slice N's memory entry prescribes the fix:
`grep '\._[A-Z_]* *='` over the suite, at PRE-FLIGHT. Run here before a line of the suite was
written, it returned exactly that one hit. `StatorBleedCore` gained `with_b_cap`, which is **not**
a gated-code edit — the constant lives in slice O's own new file. The override deliberately does
not propagate to siblings, which is faithful: Python rebinds an instance attribute and
`at_point` builds fresh objects that read the class one, and no sibling ever runs a walk.


##### STEP 4 — SHIPPED, docs-only. **SLICE O COMPLETE, AND WITH IT PHASE 5**

Three corrections, each to a place that carried a claim slice O falsified or sharpened:

1. **`rust/src/stator.rs`'s `Descendant` doc** said *"slices N and O add a VARIANT and a TABLE
   ENTRY"*. Half wrong, corrected in place with the reason: slice O adds a table entry and
   `Descendant::Plain`, because the carrier already lived one level down. **This plan's § 5.9
   carried the same sentence** and now carries the correction beside it.
2. **`docs/rung61-spec.md`** gains a *What the RUST PORT measured* section — the dead `try/except`
   and its confirmed-but-rescoped docstring claim, the two dead things of different kinds, the
   latent truthiness trap, and the two measured bar headrooms. **No verdict and no number of the
   rung moves**, which is the sentence that matters: the port re-ran the rung's own instruments
   against a bit-exact oracle and swept past its grid, and the physics stood.
3. **`rust/src/bleed.rs`'s module note** records the discharge of its own IOU — see step 2.

**AND ONE INSTRUMENT WAS RENAMED BEFORE IT COULD BECOME THE FOURTH INSTANCE.** The abort tally was
read with `.max()` inside the per-shape loop while `counters::reset()` runs once at sweep start —
so it was a *cumulative* total reported under a maximum's name. The `> 0` bar held either way,
which is exactly why it would have survived: **an instrument reporting something other than its
name is this port's most repeated defect**, and this slice's own headline is *what retires a
zero-firing count is a count*. Read once, at the end, as `lp_bleed_aborts_cumulative`.

**THE SLICE'S END STATE, MEASURED AS A NAME DIFF** (the step table's currency since step 1, never
a count): **599 → 628 run, 0 ignored — 29 additions, 0 removals.** The 29 are 4 smoke + 2 oracle
+ 23 `rung61.rs`.

---

**PHASE 5 IS COMPLETE.** Rungs 31–33, 38–39, 41, 42, 53–56 and 61, in seven slices (I · J · K ·
L · M · N · O). The phase's own named risk — the rung-61 diamond — cost one line of Rust
(`&R42` in one constructor) and produced no finding at all; **everything the phase actually
taught came from the five-name virtual set, the constant shadows, and the composition edges**,
which is what § 5.3's pre-flight said after it was widened twice. The next authorisation point is
**before phase 6**.

---

#### (i) THE LEADING FINDING — `_feasible` EXISTS TO SWALLOW REFUSALS, AND ON THE SHIPPED GRID IT SWALLOWS NONE

`StatorBleedMatcher._feasible` wraps a sibling's `stator_margin` in a bare
`except AssertionError: return None`, and its docstring justifies itself: *"Rung 42's valve
SHRINKS the choked envelope while rung 53's setting unloads the speed line, so the feasible set
is bounded on BOTH axes — by different mechanisms."* Measured over the suite's own grid — five
map shapes × four `Tt4` × four `v` × two spools × two targets, **320 `compensating_bleed` calls
and 10 613 `_feasible` calls**:

| | count |
|---|---:|
| `_feasible` calls | 10 613 |
| …that returned a row | **10 613** |
| …that swallowed an assertion | **0** |

**The claim is CONFIRMED and its SCOPE CORRECTED — not refuted.** A wide sweep (1 760 cells,
`Tt4` 600–2100, `v` 0–5, `b` 0–0.49) refuses 756 of them, under exactly **two** distinct
messages, and each attaches to the axis the docstring names:

| axis, swept alone | bound | mechanism |
|---|---|---|
| `v` at `b = 0`, `Tt4 = 1500` | between 1.2 and 1.3 | **speed-line bracket** (`ComponentMap.solve_n`) |
| `b` at `v = 0`, `Tt4 = 1500` | **none — clean to 0.49** | — |
| `b` at `v = 0`, `Tt4 = 700` | 0.49 | **choked envelope** |

So both mechanisms are real, they are the two the docstring names, and the `b` axis binds only
near the throttle edge. **What is corrected is where they live: entirely outside every shipped
test.** The consequence for the port is concrete and is a *reachability* statement, not a
site list (slice I's rule): of `compensating_bleed`'s three `None`-returning branches, **two are
dead on the whole Python suite** — `"stator setting infeasible with the valve shut"` and
`"choked envelope closed before the target"` — and only `"valve authority exhausted (b >= cap)"`
is live, at **124 of 320 calls**. This is a *hazard*-class claim about a road not taken, in
§ 5.3 item 7's category, and is deliberately NOT counted in the port's "exactly"-claim ledger.

#### (ii) TWO DEAD THINGS, AND THEY ARE DIFFERENT ANIMALS

Registered separately on purpose — merging them into one "dead constants" bullet is what makes
the second invisible.

- **`_B_MAX = 80` is a DEAD CAP.** Measured 22–30 bisection passes over all 196 solved calls.
  Same shape as `_INC_MAX` (§ 5.9 (iv)) and slice L's `Tt4_lo`: port as written, record dead.
- **`hi - lo <= 1e-15` is a DEAD ARM OF A COMPOUND CONDITION.** The exit test is
  `abs(r) <= self._B_TOL or hi - lo <= 1e-15`, and **196 of 196** solved calls exit on the
  first disjunct. The second guards nothing on this grid. Slice N's *an arm that guarded
  nothing* is the precedent, and it is the one where **porting the spelling still matters** —
  a `||` whose right side is dropped is a different function wherever the grid later moves.

#### (iii) THE TRUTHINESS TRAP — LATENT, AND PORTED AS WRITTEN

`compensability` writes `row["ratio"] = (bh / bl) if (bl and bh) else None`. Python truthiness
makes an **exact `0.0`** behave as absent, which `is not None` would not. Measured: **no
`b_star == 0.0` on any grid swept** (196 solved values, min `8.54e-3`, max `4.49e-1`), and on
the shipped throttle band every row is *mixed* — `b_lp` finite, `b_hp` `None` — so `ratio` is
`None` for the stated physical reason and the trap never decides anything.

Ported faithfully as `l != 0.0 && h != 0.0` inside the `(Some, Some)` arm, and recorded latent.
**Fourth instance of *a dead thing's spelling still has to be right*** (slice N step 3).

#### (iv) THE RETURN SHAPE IS AN ENUM, NOT A STRUCT OF `Option`s — A PORT DECISION, REGISTERED

`compensating_bleed` returns **three different key sets**, measured:

| branch | carries |
|---|---|
| solved | `b_star`, `goal`, `resid`, `bare_phi`, `bare_m_phi`, `bare_m_i` |
| `"valve authority exhausted"` / `"choked envelope closed"` | `b_star=None`, `goal`, **`b_last`**, **`resid_last`** |
| `"stator setting infeasible"` | `b_star=None`, `goal` — and **neither `b_last` nor `resid_last`** |

`compensability` reads `c.get("resid_last")`, which tolerates that third shape; a Rust struct
with `Option` fields would let a caller read a field Python would have raised `KeyError` on, so
the port is a three-variant enum. Registered as the decision, not discovered later.

#### (v) THE PLAN'S OWN WRITTEN EXPECTATION IS WRONG — `Descendant::Bleed` IS NOT NEEDED

Two shipped places predict slice O adds a `Descendant` variant: `stator.rs`'s enum doc (*"slices
N and O add a VARIANT and a TABLE ENTRY"*) and this plan at § 5.9. **Corrected here, with the
reason, before any code is written.** Rung 61's `at_setting` reads exactly one piece of state —
`self.bleed` — and `bleed` has been a field on `TwoSpoolMapCore` since slice L (`two_spool.rs`),
because rung 42 needed it there. So rung 61 adds:

- **one `StatorHooks` table**, `R61`, whose `at_setting` is `at_point(v_lp, v_hp, self.bleed)`;
- **`core.hooks = &R42`** on the inner core, which is what makes rung 53's readers see the bled
  `match` — `TwoSpoolMapCore::try_match_point` already dispatches through that pointer;
- **`Descendant::Plain`**, which is also correct for `inc_max()`: rung 61 declares no `_INC_MAX`,
  so it inherits rung 53's 80, and `Plain` returns exactly that.

Nothing else. **§ 5.9 (c)'s scalar-variant prediction is not merely refuted here — the variant
is not needed at all**, because the carrier it would have duplicated already exists one level
down. That is the *same* one-level-down carrier fact slice N was burned by twice, arriving for
once in the project's favour.

#### (vi) THE ONE QUESTION A BODY-READ CANNOT ANSWER, AND IT IS CHECKED

Registering (v) from a body-read is exactly slice N step 3's trap, so the carrier question is
separated out and **measured**: `VariableStatorCore::with_hooks` builds a fresh
`TwoSpoolMapCore` that initialises `bleed: 0.0`, so `R61`'s `at_setting` must restore the valve
after construction. Does that need a `bleed` parameter on `with_hooks` — a **sixth gated-code
edit** — or does build-then-set suffice?

**Build-then-set suffices, and the hazard that would have killed it is checked ABSENT.** If the
design capture dispatched through `core.hooks` even once, `&R42` at `bleed: 0.0` is a *different
function object* than `&R39` and the design references would shift — a value-level difference no
signature inspection shows. Read: `TwoSpoolMapCore::with_hooks` calls `TwoSpoolCore::new` and
reads stations off `base.reference`. **It never calls `try_match_point`**, so the table pointer
is inert at construction. And it is faithful besides — Python's rung 61 sets `self.bleed` *after*
`VariableStatorMatcher.__init__` returns, and no constructor in the chain calls `self.match`.

#### (vii) THE REACTING ARM IS FEASIBLE AND MUST BE SMALL — MEASURED, NOT ASSUMED

Gate 1 requires the two-axis reduce bit-for-bit on the **reacting** gas as well as the fast one.
Measured at the gate's own points (3 corners × 2 `Tt4` × 17 fields):

| gas | mismatches | wall clock |
|---|---:|---:|
| `thermally_perfect` | **0** of 102 | 0.6 s |
| `reacting_equilibrium` | **0** of 102 | 12.6 s |

…and **one** `compensating_bleed` call on the reacting gas costs **54.5 s**. So the reacting arm
is a REDUCE arm only — corners, never a `b*` sweep — the same split slice N's 5 649-key
equilibrium arm took against its 72 520-key fast one. Sized before the oracle is written, which
is the point of measuring it here.

---

#### PREDICTIONS — registered before any code

| # | prediction | falsified by |
|---|---|---|
| **P1** | No `Descendant` variant, no new field, no gated-code edit outside `stator.rs`'s new table. | any edit to `bleed.rs` / `two_spool.rs` / `stage.rs` — **REFUTED at step 2**: the first two clauses held, the third did not (`lp_eta_loop_bleed` had to become fallible). Slice L had named this exact site, this exact rung, in `bleed.rs`'s own module note. |
| **P2** | `R61`'s `at_setting` + `core.hooks = &R42` reproduce every rung-53/54 reader on a bled machine bit-for-bit. | any oracle key off |
| **P3** | The oracle is bit-exact against PyPy **first run** (six slices, six times). | a single key off |
| **P4** | The two-axis reduce holds on the reacting gas as well as the fast one, at `==`. | any field differing |
| **P5** | `_B_MAX` never reached and the `1e-15` arm never taken, on the ORACLE's grid too — not just the probe's. | a census key disagreeing |
| **P6** | The `at_setting`-drops-the-bleed trap (Python gate 2) is witnessable in Rust **only** structurally — a sibling built through `R53`'s body comes back with `bleed == 0.0` while every headline value stays plausible. | the value dump catching it |
| **P7** | Rung 61's `lp_disabled` assert is **unrepresentable** in Rust — `VariableStatorCore` holds the non-degenerate `TwoSpoolMapCore` directly, so there is no `lp_disabled` rung-61 object to reject. Ported as a deferral, not as a gate. | a representable path |
| **P8** | The 18 Python gates port to **≥ 18** Rust gates, as a `--list` name diff with 0 removals. | a removal — **HELD at 23** |

#### THE FOUR STEPS

| # | step | gate |
|---|---|---|
| 1 | ✅ `stator_bleed.rs` + `R61`; **ZERO gated-code edits** — P1 held | ✅ 599 → 603 run, 0 ignored; name diff **4 additions, 0 removals** |
| 2 | ✅ `dump_slice_o.py` + `slice_o_oracle.rs`; **P1 REFUTED** — one gated-code edit in `bleed.rs`, which slice L had written down and dated | ✅ **8 256** distinct keys bit-exact (fast) + **204** (reacting, first run) |
| 3 | ✅ `rung61.rs` — **23 gates from 18**; two detectors injected, both fired, one on a gate not aimed at it | ✅ 23/23 green; both tolerance bars measured |
| 4 | ✅ docs — the spec's *What the RUST PORT measured*, the `stator.rs` + § 5.9 corrections, and one instrument renamed before it became the fourth instance | ✅ docs-only; end state **599 → 628 run, 0 ignored** as a name diff (29 additions, 0 removals) |


##### STEP 1 — SHIPPED. **THE SMOKE GATE'S FIRST FAILURE WAS A DEFECT IN THE INSTRUMENT, NOT THE PORT**

`rust/src/stator_bleed.rs` (794 lines) + `rust/tests/slice_o_smoke.rs` (4 gates). Crate
**599 → 603 run, 0 ignored**, as a `--list` **name diff: 4 additions, 0 removals**.

**P1 HELD, EXACTLY AS § 5.11 (v) CORRECTED IT.** The module adds one `StatorHooks` table
(`R61`), sets `core.hooks = &R42` on the inner core, and takes `Descendant::Plain`. `git diff`
over `src/` is **one line** outside the new file — `pub mod stator_bleed;` in `lib.rs`. **No
`Descendant` variant, no new field, no gated-code edit** — the first slice since I to touch
nothing that was already gated, and the reason is § 5.11 (v)'s: the carrier already existed one
level down. Slice N's twice-burned lesson, arriving in the port's favour.

**THE FINDING — A HAND-TYPED CONSTANT IN A PROBE SILENTLY CHANGED THE GAS, AND THE SMOKE GATE
REPORTED IT AS A PORT DEFECT.** 11 of the first 12 dumped values were bit-exact and `thrust_comp`
was off by **exactly 1 ULP**. Chasing it down through the decomposition found `v0` — the
freestream velocity, a *phase-2* quantity — differing by 1 ULP, which no shipped oracle could
have allowed. The cause was in the dump script: it built the gas with `R_c = .4/1.4*1004.` where
`test_rung61.py::_cpg_gas` computes `R_c = (gamma_c - 1.0)/gamma_c * cp_c`, and **`1.4 - 1.0` is
not the literal `0.4`** (`0.3999999999999999` against `0.4000000000000000222…`). Re-dumped with
the suite's own recipe, all 16 values match to the bit.

Three things follow, and the third is the one worth carrying:

1. `probe_o1.py` — whose census went into § 5.11 (i)/(ii) — had used the correct recipe already,
   so those counts stand. The wide sweep in § 5.11 (i) had not, so it was **re-run**: 1 760 cells,
   756 refused, 578 speed-line / 178 choked — *identical*, and the `v` bound tightened from
   "between 1.2 and 1.4" to **between 1.2 and 1.3**.
2. A 1-ULP gas difference moved a thrust by 1 ULP and left `phi`, `n`, `b*` and both identity
   residuals **bit-identical** — so the sensitivity is real but narrow, which is why it survived
   eleven values before showing.
3. **A GO/NO-GO GATE THAT FAILS HAS NOT NECESSARILY FOUND WHAT IT NAMES.** Slice M's two
   measuring passes found defects in the instrument rather than the port; this is the third
   instance, and the first where the instrument was a *constant* rather than a reader. The rule
   the port already had — *never retype a decimal, dump the bits* — does not cover this, because
   the defect was upstream of the dump: **the INPUTS have to be built by the source's own
   expression, not by an equivalent-looking one.** An oracle inherits every constant its dump
   script types.

**THE STRUCTURAL GATE IS THE ONE NO VALUE CAN REPLACE.**
`at_setting_carries_the_bleed_and_rung53s_body_would_not` asserts on the STATE — that the
dispatched sibling keeps `bleed = 0.10` and keeps rung 42's inner table — because § 5.3 item 7
measured what the failure looks like: 13–15 % on `φ` and `N`, **0.1 % on thrust**. Every headline
number a value gate would look at stays plausible.

### 5.12 PHASE 6 PRE-FLIGHT — the census in the OTHER direction, MEASURED

**AUTHORISED 2026-08-17** ("work on Phase 6 (the 15 transient rungs)"), which is the fresh
authorisation § 5's row 5 said was owed before this phase.

Phase 5's pre-flight (§ 5.3) swept with phase 5 as the *descendant* side and found **one** name
crossing forward. Phase 6 is the mirror at ten times the size and the roles are swapped: **every
class in phase 7 descends from `TwoSpoolFuelTransient`.** So the same sweep is run again with the
phase-6 set as ANCESTORS and all 58 classes opened on the descendant side — § 5.3's second
widening, applied to the phase that widening was *about*.

**THE FIRST RUN OF THE CENSUS WAS WRONG, AND THE ERROR IS ONE THE PHASE-5 SWEEP COULD NOT MAKE.**
It asked *"is this name redefined by any class descending from ANY phase-6 class"* and returned
**32** names. But `SpoolTransient` (single-spool) and `TwoSpoolFuelTransient` (two-spool) are
**siblings**, not a chain — a same-named method on one is not an override of the other, and 26 of
the 32 were exactly that. Phase 5's set had one root, so the bug had nowhere to appear. Re-scoped
so an override must be a class whose MRO contains *the ancestor that makes the call*:

| name | called on `self` by | sites | overridden by |
|---|---|---|---|
| **`integrate_fuel`** | `SpoolTransient.ramp_excursion_fuel`, `TwoSpoolFuelTransient.{ramp_excursion_fuel, _fuel_ramp_march}` | 2 | **11 phase-7 classes** (`LaggedBleedTransient` … `SensedCapTransient`) |
| **`_close`** | `TwoSpoolTransient.{_instant, equilibrium}` | 2 | 4 (`ScheduledStatorTransient`, `ScheduledBleedTransient`, `LimitedBleedTransient`, `LaggedBleedTransient`) |
| **`_close_fuel`** | `TwoSpoolFuelTransient._instant_fuel` | 1 | the same 4 |
| **`_surge_fuel`** | `TwoSpoolFuelTransient.{integrate_fuel, _integrate_fuel_lagged, _integrate_fuel_asym}` | 3 | `ScheduledStatorTransient` |
| **`_instant_tail`** | `TwoSpoolTransient._instant`, `TwoSpoolFuelTransient._instant_fuel` | 2 | `ScheduledBleedTransient` |
| **`_powers`** | `TwoSpoolTransient.equilibrium` | 1 | `ScheduledBleedTransient` |

**SIX names, all on the two-spool chain, and every one of them CROSSES INTO PHASE 7** — there is
not a single phase-6-internal hook. `SpoolTransient` and `CombustorTransient` need **none**:
nothing downstream inherits from them (`CombustorTransient` has 0 subclasses, `SpoolTransient`
has 1 and it is `CombustorTransient` itself, which overrides nothing that rung 34 calls on
`self`). So slices P and Q ship as plain `struct` + `impl`, and the `Hooks` table appears at
slice R.

**THE OTHER THREE ARMS, EACH MEASURED RATHER THAN ASSUMED:**

- **Template-method hazard (§ 5.3 arm 5): ABSENT, 0 sites.** No phase-6 body reads a `self.X`
  that its own MRO does not supply.
- **Constant shadows (the `_INC_MAX` shape): 0 pairs.** No descendant of any phase-6 class
  redefines a constant that class declares, and no phase-6 class shadows a phase-5 one. Four
  class constants ARE read off `self` inside phase-6 bodies and must be per-cell parameters
  rather than literals on the same rule: `_EQ_MAX = 80`, `_EQ_TOL = 1e-12`, `_N_TOL = 1e-12`,
  `_PHI_FLOOR = 0.3`.
- **Sibling-receiver dispatch: live, but entirely PHASE-7-INTERNAL.** Phase 6 defines no
  `at_*` constructor at all — `at_lever`/`at_stator` are rung 57's and below. Two overridden
  names *are* invoked on a sibling result (`_stator_march` at 17 sites, `equilibrium` at 2),
  and both callers are phase-7 classes. **Nothing is owed by phase 6 here**, which is the one
  arm where phase 5 was the harder case.

**THE SCOPE LIST, ENUMERATED — and the 4257–4506 object block spans TWO phases.** Slice K's audit
found rung 41 in no phase and rung 61 in two, on the lesson that *a scope list is a claim about a
SET and nobody had ever counted*. Counted here: five objects sit in that block, and the middle
one is not phase 6's.

| object | line | rung | phase |
|---|---|---|---|
| `AccelSchedule` | 4257 | 48 | **6** |
| `SurgeLimiter` | 4304 | 49 | **6** |
| `IncidenceLimiter` | 4349 | **60** | **7** |
| `_release_weight` | 4426 | 51 | **6** |
| `AsymmetricLag` | 4454 | 52 | **6** |

**RUNGS 46–52 DO NOT PARTITION THE WAY EVERY PRIOR SLICE DID, AND THE SOURCE SETTLES IT.** Every
slice so far was *a class*. These seven are **keyword arguments on one method**: `integrate_fuel`
(4834–5029) plus the two bodies it dispatches to, `_integrate_fuel_lagged` (5030) and
`_integrate_fuel_asym` (5105). A slice-per-rung would write partial bodies that never existed in
Python, and each rung's reduce contract is *`X=None` ⇒ the prior rung bit-for-bit* — a claim about
the WHOLE body, testable only once it is whole. **So `integrate_fuel` is ported ENTIRE in one
slice, with all seven legs, and only the GATES are sliced.** Decided here, not discovered at
slice T.

**THE SLICE ORDER IS FORCED BY `_degenerate`, not chosen.** `TwoSpoolTransient.__init__` (3416)
constructs a `SpoolTransient` as `self._degenerate` when `lp_disabled`, and both
`TwoSpoolFuelTransient.integrate_fuel` (4961) and `.equilibrium_fuel` (4794) delegate to it. The
single-spool transient is therefore a *component* of the two-spool one, not a parallel branch.

| slice | rungs | class | depends on |
|---|---|---|---|
| **P** | 34, 35, 36 | `SpoolTransient` | phase 5 slice I/J (`MapMatcher`, the `_solve_turbine` hook) |
| **Q** | 37 | `CombustorTransient` | P |
| **R** | 40, 44 | `TwoSpoolTransient` | P (`_degenerate`), slice K |
| **S** | 43, 45 | `TwoSpoolFuelTransient` — the whole `integrate_fuel` | P, R |
| **T** | 46, 47, 48 | gates only: governor, `τ_gov`, `AccelSchedule` | S |
| **U** | 49, 50, 51, 52 | gates only: `SurgeLimiter`, `s_off`, `τ_rel`, `AsymmetricLag` | S |

**THE ARITHMETIC RISK WAS PREDICTED IN THE WRONG PLACE, AND THE PROBE REFUTED IT.** The phase
table calls phases 4–7 *"grinding but low-risk"*; the one shape that would break that is a
`min`-select inside an RK4 march, because a last-bit difference there does not drift — it flips
which leg is authoritative and the flip propagates for every remaining step. `der` (4972) collects
up to three caps and applies `min(caps)`, so the prediction was that the oracle must dump an
argmin index (slice N's FINDING 6, one level up). **Measured over 78 cases** — ramp rate
{0.15, 0.5, 2.0} × redline {none, 1480, 1420} × accel margin {none, 0.05, 0.15} × φ floor
{none, 0.75, 0.77}, `M:\claud_projects\temp\rust-phase6\probe_minselect2.py`:

**not one `der` in any case had two live caps.** 0 contested selections out of ~600 000. The
legs never contend, so `min(caps)` is never a comparison and there is no argmin to flip. The
discrete content is one level down, in the two **arming predicates** — `i["Tt4"] > Tt4_max`
(4991) and `c < mf` (4997) — each evaluated in every RK sub-step. Those are what the oracle
must emit, as a per-leg decision sequence, not an index.

**AND THE CPython ARM SAYS NOTHING YET, WHICH IS ITSELF THE FINDING.** Dumped both decision
sequences and the full trajectory bits over six armed configurations and diffed CPython 3.14
against PyPy: **9 376 keys, 100 % identical** — no arming flip, no trajectory bit. Next to slice
G's 8.0 % and slice K's 46.3 % that reads as a clean result, and it is not one: the probe runs a
**CPG** gas, whose property calls are closed-form, so there is almost nothing for the two
interpreters to disagree about. **The CPython arm is only a detector on the reacting/equilibrium
gas**, and slice S owes that measurement before it can claim the arming decisions are stable.
Recorded now so the 100 % is not quoted later as coverage it does not have — *an oracle cannot
see a missing gate*, applied to the instrument instead of the gate.

**THE DEFERRAL INBOX, COLLECTED ONCE AND ASSIGNED.** Six items are owed to this phase; each is
booked to the slice that can discharge it, so none ships orphaned:

| owed item | recorded at | slice |
|---|---|---|
| `test_reduce_transient_untouched_by_surge_line_bit_for_bit` | `rung41.rs` roster #2 | **R** (rung 40) |
| `test_rung36_verdict_survives_but_its_mechanism_is_corrected` | `rung41.rs` roster #12 | **P** (`SpoolTransient.surge_margin_channels`, single-spool 34/36) |
| `ComponentMap::phi_max` **and its rung-53 early return** | `rung53.rs` item 1, `map.rs:101` | **P** (rung 34's forward closures are its only reader) |
| `test_cycle_untouched_transient_ladder_…_unstacked` | `rung55.rs` item 5, `stage.rs:870` | **S** (needs `TwoSpoolFuelTransient`) |

Slice O's lesson applies to every deferral this phase *creates*: **write it where the next
slice's compiler and tests will hit it**, not only into a paragraph — what reached slice O was a
panic with a backtrace, and the note that had predicted it correctly was read second.

**REPRESENTATION, SETTLED ONCE.** Schedules (`fuel_schedule`, `schedule`, the internal `sched`
closures at 3828/5295) are all `f64 -> f64` closures capturing local values; they become
`&dyn Fn(f64) -> f64`. `_closer(self, method, *args)` (10041) is phase 7's and forwards varargs —
**not** phase 6's problem, and it is flagged here only so slice S does not design for it.

**SIZING, HONESTLY.** ~3 540 source lines and 5 036 test lines over **156 Python tests** in 15
files. Phase 3 was 2 745 lines / 204 tests and took five slices; phase 5 was seven. **The table's
"4–6 sessions" is light for six slices** — say so rather than discover it. Nine of the 156 tests
carry `slow` (rungs 37, 40, 46, 52); slice M's rule stands — port the gate, drop the marker,
re-introduce `#[ignore]` only against a MEASURED Rust cost. Phase 6 is the first phase where one
might genuinely be earned (4 000-step marches inside sweeps), so measure rather than assume in
either direction.

### 5.13 SLICE P (rungs 34/35/36, `SpoolTransient`) — PRE-REGISTERED, five probes MEASURED first

The port's first ODE. `SpoolTransient` (720 lines, `engine.py:1292–2010`) subclasses rung 32's
`MapMatcher`, replaces the steady shaft balance with a FORWARD closure, and marches
`dν/ds = Φ(ν, Tt4(s))` under RK4. Rung 35 re-controls the same plant on FUEL (`Tt4` becomes an
output); rung 36 hangs a read-only surge line beside it. Three Python suites, **19 tests**, zero
`slow` markers.

`M:\claud_projects\temp\rust-phase6\probe_p.py`, five arms, run over seven map shapes × six
throttles plus fifteen marches.

**PROBE 1 — `ComponentMap::phi_max`'s DEFERRAL NOTE DESCRIBES A CONTROL FLOW THAT DOES NOT
EXIST.** The symbol is owed to this slice (`rung53.rs` item 1 / `map.rs:101`) because rung 34's
two compressor closures are its only callers. Both records say the rung-53 repair is an **early
return at `vsv == 0.0`, "exactly as `psi` does"**. Read: `psi` does have one (`engine.py:855`);
**`phi_max` does not.** It folds the swirl amplitude `A = vsv·(1+l)` into the *coefficients* —
the flat guard becomes `sigma == 0 and l == 0 and A == 0`, `rhs = 1 − A − psi_floor`, and the
linear coefficient is `lin = l + A`, so `A` appears in three places and none of them is a branch.
The quoted assertion is the rung-34 form (`rhs = 1 − 0.1`, `lin = l`) — **correct exactly where
slice P lives and wrong everywhere else.** Copying it would have shipped a `phi_max` with a
spurious early return, observationally identical on every call this slice makes and wrong for
rung 53. *Slice O's lesson with the failure mode moved: the deferral was placed where the next
slice would read it, and its CONTENT was the defect.*

Measured: **16 508 calls, `vsv == 0.0` at every one**, and only two of the three arithmetic
branches live — `flat5` (σ = l = 0 ⇒ 5.0) **5 258**, `quadratic` **11 250**, the `sigma == 0,
l ≠ 0` **linear** branch **0**. Six distinct return values over six `(σ, l)` pairs. So the dead
branch and the dead `A` both have to be SPELLED right against a Python that never exercises them
here — slice N step 3's rule, third instance.

**PROBE 2 — THE `_solve_turbine` OVERRIDE IS LOAD-BEARING, AND THE SOURCE'S OWN TOLERANCE CLAIM
IS 1.7× LIGHT.** This is the name § 5.3's census made phase 5 ship hookable; slice P is what
finally fires it. `engine.py:1322` says the Illinois override finds the *"same root as the
inherited bisection to ~1e-11"*. Measured over **14 002 paired calls** (both solvers run on
identical inputs):

| quantity | max relative | median | bit-identical |
|---|---|---|---|
| `pi_t` | **8.950e-12** | 1.069e-12 | 8 / 14 002 |
| `tau_t` | **1.707e-11** | 4.477e-13 | 187 / 14 002 |

The claim holds for the ROOT and is exceeded by the quantity DERIVED from it — `tau_t` comes back
1.9× outside the stated band, because `_tau_t_of_pi_t` amplifies. Rung 31's bracket never failed
where rung 34's succeeded (0 of 14 002), so the two solvers agree on the domain as well as the
root. **The porting consequence is a discriminator:** calling `r31_solve_turbine` from a rung-34
body would move `pi_t` by ~9e-12, which the ORACLE sees and the SUITE does not — rung 34's own
reduce gates are written at `1e-8`. Registered as a gate in its own right, not left to the dump.

**PROBE 3 — THE ASSERT CENSUS: FIVE SITES FIRE, AND THE RAREST ONE IS THE ESCALATION GUARD.**
Slice I's rule (*an assert becomes fallible iff it is reachable from inside a bracket march's
`resid`*) applied to rung 34's four `try` scopes. Firings, deduplicated to the raising frame:

| site | fires | reached from |
|---|---|---|
| `_close_compressor:1390` — the flow bracket | 1 312 | `_find_equilibrium_nu`'s march-in, `integrate`'s step |
| `_solve_f:500` — the matcher's `f` fixed point | 1 194 | inherited, slice I already fallible |
| `components.py apply:683` — the nozzle | 983 | inherited, slice I already fallible |
| `_turbine_subsonic:1432` — the subsonic bracket | 187 | `_instant_tail`'s fallback `try` |
| **`_instant_tail:1488` — the `M9 > 0.985` escalation** | **2** | nothing catches it |

**Both arms of the subsonic fallback are LIVE on this grid** — 185 of the 187 bracket failures are
the legitimate `M9 → 1` boundary and get absorbed, and **2 escalate**, which is the branch the
source's comment says must raise rather than hide under a *"subsonic"* label. A port that turned
that guard into a silent fallback would pass every value gate. Two firings out of ~16 000 instants
is the whole detector, so the dump carries the escalation as a COUNT.

**PROBE 4 — TWO CLAMPS LIVE, ONE DEAD.** `_interp`'s low clamp fires **15**, its high clamp
**34**, interior **2 612** — both edges exercised, so the ported `_interp` is gated three ways.
`integrate`'s `nu = max(0.2, …)` sub-idle floor: **0 hits** in every march run here. Dead, spelled
anyway, and recorded so a later slice does not read its absence as absence of the code.

**PROBE 5 — THE MARCHES TERMINATE EARLY, AND ONLY ON THE SPOOL-DOWN.** `except AssertionError:
break` is not decoration: of fifteen marches, the three accel ramps and the fuel ramp run to full
length on every shape, and the **fuel-cut spool-down stops early on two shapes of three** —
`surge_flow` at 66 of 161 steps, `flat` at 81 of 161, `flow_dominated` full. So the trajectory
LENGTH is a discrete output that varies by map shape, and the oracle emits it as a key.

**THE TEN PREDICTIONS, registered before any Rust is written:**

1. The oracle comes back **100 % bit-exact against PyPy** — the RK4 has no adaptive control, so
   it carries accumulation order and no stopping rule (§ 4.3's precedent), and every solver under
   it is an Illinois already proven exact in slices I/J.
2. `r34_solve_turbine` fires **> 0** and `r31_solve_turbine` fires **0** on a `SpoolTransient`
   march — the count, not the values, because slice N FINDING 3 caught a hook that compiled and
   was never reached.
3. Injecting `r31_solve_turbine` in rung 34's body moves **> 0 oracle keys and 0 rung-suite
   gates**, at ~9e-12 — probe 2's discriminator, measured as a detector.
4. `phi_max`'s **linear** branch stays at 0 firings across the whole dump, and the `A ≠ 0` path
   stays unreachable — both gated against Python directly rather than through the march.
5. The escalation guard fires **exactly 2** times on the pre-registered grid, and the fallback
   **185**; a port that swaps them changes no value key.
6. Trajectory LENGTH differs by map shape on the spool-down and is bit-reproduced.
7. Rung 34's gate 6 (`solve_n(m, tau_c_forward(n, m)) == n`) is **exact**, not merely tight —
   slice J ported `solve_n` and this is its exact inverse. If it is not exact, the reason is
   slice J's mis-spelled-square shape and it gets its own gate.
8. `SpoolTransient` and `CombustorTransient` need **no `Hooks` table** — § 5.12's census says
   nothing downstream overrides a name rung 34 calls on `self`. If slice P finds one, § 5.12's
   arm 1 is wrong and the phase table's slice split moves.
9. The rung-41 gate deferred to here (`test_rung36_verdict_survives_but_its_mechanism_is_corrected`,
   `surge_margin_channels`) ports whole; `rung41.rs`'s roster count moves 10 → 11.
10. Zero `slow` markers are earned: rungs 34/35/36 carry none in Python and the Rust marches are
    the same arithmetic 6–8× faster.

---

## 6. Named risks

##### STEP 4 — SHIPPED. **THE SECTION'S OWN CENSUSES WERE MEASURED ON TWO DIFFERENT GRIDS**

`slice_n_oracle.rs` is **72 520 keys bit-exact against PyPy on the first run**, over the
pre-registered 640 cells and 160 schedule rows, plus a **5 649-key equilibrium arm** and a
**41 560-key CPython arm**. Five gates; the crate is **556 run / 0 failed / 0 ignored** over
**62 targets** — the 556 measured with `cargo test --release -- --list` on a crate with zero
`#[ignore]`, not carried over from step 3's 551 + 5 (step 2's postscript: *exactly one record
should be a measurement and the rest should quote it*).

**FINDING 1 — § 5.10's OWN CENSUSES SPLIT BY PROVENANCE, AND ITS PROSE DOES NOT SAY WHICH GRID
EACH CAME FROM.** (iv)'s `binds`/`inc_worst` table reproduced **exactly** — 240/400/0 derived,
50/587/3 uniform, 1 182/88/10 either — and all 13 interior readings are HP, CPG, `Tt4` = 1500,
with worst margin spread **4.441e-16**. That is because `probe_n3` swept *this* grid. (iii)'s
`3 204 / 521 649` and (vi)'s `120 built / 4 360 hits / 6 464 marches` did not: `probe_n1` sweeps
`K ∈ {2,4,8}` — **three** values — with **no `cap_profile` axis**, i.e. 240 cells and 120 stacks
against the dump's 640 and 320. Measured here: **320 stacks, 3 317 982 marches, 320 profiles
built against 24 000 cache hits.** So the census is **emitted as dump keys and compared**, never
restated — *slice L step 4*'s copied bar, pre-empted. **Two censuses in one section can be
measured on two grids, and the section reads as though they share one.**

**AND THE ONE CENSUS PYTHON CANNOT SPLIT IS DECLARED RATHER THAN FAKED.** `march` adds both
floors into one `clamped`, which is (iii)'s point, so the dump emits the SUM (49 173 on the fast
arm, 399 967 on the schedule one) and the Rust asserts `t_floor + p_floor == sum` beside
`p_floor == 0`. The split stays step 2's derived-threshold gate.

**FINDING 2 — § 5.10 (i)'s FRAME CENSUS REPRODUCED TO THE FIRING, AND THAT DISCHARGED THE LAST
DEFERRAL.** 40 caught firings: **39 bracket, 1 clamped root, 0 `ComponentMap.solve_n`** — the
pre-registered table, and identical under CPython. `slice_n_deferrals_so_far` item 2b was owed
here and the catch is what made it hard: `except AssertionError: break` swallows *which* arm
fired, so reproducing the schedule gates nothing. The dump classifies each firing **at the raise**
and carries the `(m, tau_c, eta_live)` triple plus the cell of the clamped-root one, so
`the_clamped_root_arm_is_reached_from_the_dump_grid` rebuilds that stack — checked against its
own `tau_d`/`e_d` bits — and re-enters `try_solve_n` directly. **Both arms of P1's twin are now
executed.**

**FINDING 3 — `StageStack::solve_n`'s `K = 1` DISPATCH IS UNREACHABLE THROUGH THE MATCHER, ON ALL
THREE ARMS.** `solve_n_k1` = **0**, everywhere, including the 3 368 `ComponentMap::solve_n` calls
`running_line_shift`'s `K = 1` baseline makes. The reason is structural: `StageStackCore` builds a
stack only where `K > 1`, so at `K = 1` the efficiency loop **is** rung 39's and calls the map
directly — the `if k == 1` branch inside the stack's own solver is reached only from a
hand-built stack (step 2's smoke cell D). § 5.10 P5 says the reduce is *"bit-for-bit, not merely
tight"* **because** it dispatches; that is true of the OBJECT and vacuous on the MATCHER, where
no stack exists to dispatch. P5 is sharpened, not refuted — and the count is what surfaced it.

**FINDING 4 — THE PASS-COUNT IDENTITY NEEDED THE ABORT CENSUS, AND THE TWO ABORT REASONS ENTER IT
WITH DIFFERENT WEIGHTS.** Step 2 gated `solve_n`'s 48 bisections as *predictable from the
arithmetic*. Written here as `passes == 48 × calls` it is **wrong by 1 872**: a BRACKET abort
returns before the loop and runs **zero** passes, while the CLAMPED-ROOT abort runs the full 48
and aborts after it. So the identity is `48 × (calls − K1 dispatches − bracket aborts)`, and it
reads the firing census to get there. A derived bar still has to be derived on the code path that
actually runs.

**FINDING 5 — A FLAG THAT DISARMED ITSELF, AND A COUNT COULD NOT SEE IT.** The CPython arm is
`fast` minus the per-row projections, switched by a `rows` parameter. That parameter is
**shadowed** by `rows = m.throat_walk(...)` inside the same function, so from the second subgrid
cell onward the flag was a non-empty list — truthy — and the per-row keys came back. The arm
emitted **71 504** keys instead of 41 560, which next to `fast`'s 72 520 reads as *about the
same*. Only a key-**SET** diff found it: 856 missing, every one in the FIRST shape. This
project's *coverage is a name → parameter-set diff, never a count*, arriving on a dump script
rather than on a test list. The `fast` golden was unaffected (`rows = True` stays truthy) and was
re-run and diffed hex-for-hex to prove it.

**FINDING 6 — BOTH DETECTORS MEASURED, AND THE SECOND ONE IS WHAT JUSTIFIES 43 % OF THE FILE.**
The dump passed first run, which is step 2's warning exactly, so each gate was watched to fail:

| injected defect | keys differing | of which |
|---|---|---|
| `argmin` re-spelled as a `<=` fold (LAST of equals) | **47** of 72 520 | **all** index/flag — **no value key moved** |
| every INTERIOR row's `m_c` nudged 2 ULP | **7 046** | **7 040 per-row, 6 aggregate** |

The first is § 5.10 P6 *shown*: a value oracle is blind, the index keys are not. The second is
why both argmin currencies are dumped for **every row of every half-row** rather than for the
winning row — without them this defect shows on **6 keys of 72 520**, and had the three
interior-`binds` half-rows not existed, on **zero**.

**FINDING 7 — THE CPYTHON TIER SLICE M's CLASSIFIER WOULD HAVE GOT WRONG, AND IT IS (iv)'s
DEGENERACY ARRIVING ACROSS INTERPRETERS.** Slice M's rule was *discrete → bits on every
interpreter, because a difference is a BRANCH and not drift*. Measured: **520 discrete keys differ
and every single one is at `Tt4` = 1500**, where every `phi_k = 1` collapses the per-row margins
to 1–2 ULP and which row wins is decided by the last bit of the march's own accumulation. That is
drift wearing a branch's clothes. So the tier **splits**: argmin keys held to bits **off** design
(**7 481**, zero differing) and **counted** at design (2 280 keys, **520** flips, pinned).
Everything else discrete — `reached`, `chokes`, `n_rows`, `n_caps`, `index`, `K`, `vsv_stages` —
is bit-exact everywhere. The continuous tiers are slice M's two, re-measured: `|Δ| ≤ 1e-10` first
(the smallest live magnitude among differing keys is a converged `residual` at **1.5e-14**), then
relative `≤ 1e-7` with the measured worst **2.889e-9** pinned at 4e-9 — and it is a `d_n`, a finite
difference, slice M's amplification lesson holding.

**AND THE ONLY THING THAT FAILED ON THAT ARM'S FIRST RUN WAS A COUNT I HAD DERIVED OVER THE WRONG
POPULATION.** Every value tier passed; the two argmin bars read 7 481 / 2 280 and the measured
numbers are **4 680 / 1 560**. The 7 481 came from *all discrete keys minus the design ones*,
which sweeps in `reached`, `chokes`, `n_rows`, `n_caps`, `index`, `K` and `vsv_stages` — 3 681
keys that are not argmins at all (and none of which differs). **A count derived from a SUPERSET is
a guess**, which is this port's *guessed census bars* lesson arriving on a population instead of
on a magnitude — and the enumeration that fixed it is the same one that produces the bar:
4 × 1 280 + 640 + 2 × 160 + 160 = 6 240, of which 1 560 sit at design.

**AND THE CENSUS IS NOT INTERPRETER-INVARIANT WHILE THE FIRING CENSUS IS.** CPython marches
3 346 848 against PyPy's 3 317 982 and builds 10 702 schedule stacks against 10 664 — the scan
walks a different number of steps — so the census equality is deliberately **not** re-asserted on
that arm (slice K's P2). The **branch** counts are: `binds`/`inc_worst`/reached all reproduce, and
so does 39 + 1 + 0.

**FINDING 8 — TWO OF THE SECTION'S OWN SENTENCES ARE WRONG, AND BOTH IN THE SAFE DIRECTION.**
§ 5.10's laziness note says *"80 of the 160 schedule rows are built with `capacity = False`"*;
measured, it is **160 of 160** — `probe_n3` sets it on every matcher — so the eager-build hazard
it describes is **wider** than the note claimed and its conclusion stands a fortiori. And
`CAP = 0.60`'s provenance is misattributed: `probe_n1` calls it *"rung 54's disclosed capacity
constant, as the rung-56 tests carry it"*, but `tests/test_rung56.py:48` carries **0.90** and
`0.60` appears nowhere in rungs 53–56 as a capacity. It is **kept** — (iv) was measured at 0.60
and moving it would re-point every one of those bars at unmeasured cells (§ 5.7 (e)) — so the
constant is *arbitrary-but-pre-registered*, which is a different justification from the one the
probe wrote down.

**FINDING 9 — THE COMPLETENESS ASSERT WAS SUPPLYING ITS OWN ALIBI, AND IT CERTIFIED SEVEN KEYS
NOBODY READ.** `seen + census_keys == want.len()` counted the census and firing keys **by
prefix**, so any of them was "covered" whether or not a gate had looked at it. Nine were not:
`marches_clamped` and `map_solve_n_calls` on all three arms, `fire/sched/clamped_root`, and the
whole `fire/clamped/*` sample the other gate consumes from its own load. That is
*a documented gate that doesn't exist* with the accounting standing in for the gate. Replaced by a
`HashSet` of keys a gate actually consumed, asserted EQUAL to the census/firing key set — and the
detector was measured: stop reading one key and it names it. Each orphan then got the bar it can
carry. `map_solve_n_calls` is § 5.10 (i)'s **third row** and is now gated (zero on the two fully
stacked arms, non-zero on the fast arm, whose `running_line_shift` builds a `K = 1` baseline);
`marches_clamped` counts **marches** where `StackCensus` counts **stages**, so Rust cannot
reproduce it without a step-2 edit — kept, with the RELATION as its bar (`0 < mc <= total`: 49 173
stage firings over 22 565 marches is ~2.2 stages each, not one pathological march) and the reason
written down.

**FINDING 10 — § 5.10 (ii)'s 40 AND § 5.10 (i)'s 40 ARE THE SAME 40, AND NOTHING SAID SO.** (ii)
counts schedule rows that did not reach; (i) counts firings of the caught scope. They are
independent: a row can also fail by walking the scan to `v_hi` with **no sign change**, which is a
map-shape fact and not a raise. `bracket + clamped_root == rows − reached` is now asserted, and it
holds — so on this grid every non-reached row is a raise. Two numbers that agree are not the same
number until something says they must be.

**ONE READER'S GRID IS NAMED RATHER THAN IMPLIED, AND ONE INVARIANCE REPLACES A DUPLICATE.**
`throat_walk` and `running_line_shift` are swept on `K = 8`, derived, both spools, all shapes —
`throat_walk` because it is a projection of rows the sweep already dumps (what it gates is the row
ASSEMBLY), `running_line_shift` because it re-matches a `K = 1` sibling per throttle and is the one
reader that doubles its cell's cost. And `cap_profile` is read **only** by `capacities`, so
`stage_margin`/`work_gap`/the matched point are bit-identical across it: the dump carries them on
`derived` alone and `the_capacity_profile_cannot_reach_any_rung_55_reading` asserts the identity
instead — with a `moved > 0` clause so it cannot become a comparison of two identical objects.
That is a **stronger** statement than dumping the duplicate, and it saved 3.4 MB.

##### STEP 5 — SHIPPED. **THE SOURCE'S OWN GATES WERE THE THING THAT NEEDED MEASURING**

`rung55.rs` (20 gates) and `rung56.rs` (23) port `test_rung55.py`'s 18 and `test_rung56.py`'s 21.
The crate is **599 run / 0 failed / 0 ignored** over 64 targets, measured with
`cargo test --release -- --list` and reconciled as a **name diff against the step-4 baseline: 43
additions, 0 removals** — the step table's own currency, because *exit-0 proves nothing FAILED;
only the diff proves nothing VANISHED* (step 1). The ignored column was read explicitly, which is
step 2's ```` ```ignore ```` trap; both new files spell all their fenced blocks `text`.

**THE NAME → PARAMETER-SET DIFF, which is the artifact the rule actually asks for.** A count that
reconciles is not one:

| Python gate | its parameter set | Rust |
|---|---|---|
| `test_reduce_K1_is_bit_for_bit_rung53` | `(vl,vh)` × 4 × 4 throttles × 19 fields | 1, loops |
| `test_reduce_stack_object_dispatches_at_K1` | 3 `(m, tau)` points | 1, **+ census clauses** |
| `test_stack_reproduces_rung2b_polytropic_efficiency` | `kc` = 3.5, `K` ∈ {1,2,4,8,16,32} | 1 |
| `test_reduce_K1_on_the_reacting_equilibrium_gas` | 2 throttles, equilibrium gas | 1 |
| `test_design_ladder_is_exact_for_every_K_and_split` | `K` ∈ {2,4,8,16} × 2 splits × 2 spools | 1, loops |
| `test_front_stage_phi_is_the_face_phi` | `K` ∈ {4,8} × 4 throttles × 2 spools | 1 |
| `test_capacity_style_guards_reject_nonsense` | THREE refusals | **2** + ledger item 1 |
| `test_marched_work_differs_…_throttle_depth` | 4 throttles × 2 spools, `K` ∈ {1,8} | 1 |
| `test_p1_running_line_shift_sign_and_monotonicity` | 5 shapes | 1, loops |
| `test_p1_is_paid_in_shaft_speed_not_performance` | 2 shapes × 4 throttles | 1 |
| `test_p4_front_stalls_while_the_rear_chokes` | 2 throttles × 2 spools × 8 rows | 1 |
| `test_p5_shift_converges_in_K` | 3 throttles × `K` ∈ {1,2,4,8,16} | 1 |
| `test_p6_verdicts_survive_the_work_split` | 3 throttles × 2 spools | 1, **+ two `assert_ne!`** |
| `test_cycle_untouched_transient_ladder_…_unstacked` | — | **DEFERRED, phase 6** |
| `test_p3_front_row_lever_cost_factorises` | `K` ∈ {2,4,8,16} | 1 |
| `test_p3_row_count_has_an_interior_optimum` | rows 1…6 at `_V_SCAN` = 0.01 | 1 |
| `test_p3_all_rows_schedule_ceases_to_exist_deep_off_design` | 4 throttles | 1 |
| `test_cycle_untouched_default_design_run_…_rung6` (×2 files) | 5 stations | 1 each |
| — | — | **+1 ADDED**: `…the_scan_step_is_an_instance_value…` |
| `test_reduce_invariance_over_capacity_and_profile` | 3 `(vl,vh)` × **9 cases** × 4 throttles × 19 | 1 |
| `test_reduce_K1_is_rung54_throat_margin_bit_for_bit` | 2 settings × 2 profiles × 4 throttles × 2 spools | 1 |
| `test_reduce_stack_capacities_at_K1` | 2 profiles × 3 gammas | 1 |
| `test_derived_profile_is_the_ladder_…_front_row` | 2 spools × 8 rows | 1 |
| `test_uniform_profile_is_the_disclosed_alternative` | TWO refusals + the uniform read | **2** + ledger item 1 |
| `test_hp_profile_falls_harder_than_lp` | 2 spools | 1 |
| `test_per_row_corrected_flow_is_phi_times_n_…` | `vsv_stages` = 3 at `v` = 0.40, 8 rows | 1 |
| `test_design_tie_is_a_tolerance_not_an_identity` | `K` ∈ {2,4,8,16} × 2 spools | 1 |
| `test_amplification_is_the_non_tautology_gate` | 5 shapes × 2 splits × 2 spools | 1 |
| `test_uniform_profile_amplifies_harder_than_derived` | 2 throttles × 2 spools | 1 |
| `test_binding_row_migrates_front_to_rear` | 5 shapes × 2 splits × 2 spools × 8-point walk | 1 |
| `test_uniform_profile_binds_at_the_rear_…` | 5 shapes × 2 spools × 7 throttles | 1 |
| `test_K_is_a_resolution_increments_shrink` | 2 spools × `K` ∈ {1,2,4,8,16,32} | 1 |
| `test_split_is_load_bearing_but_carries_no_sign` | 5 shapes × 2 spools × 2 throttles | 1 |
| `test_two_constraints_opposite_ends_and_opposite_spools` | 5 shapes × 2 throttles | 1 |
| `test_rung54s_hp_throat_claim_is_corrected_by_resolution` | 3 throttles, both profiles | 1 |
| `test_capacity_channel_stays_diagnostic_only` | `C` ∈ {0.99, 0.30}, 19 fields | 1 |
| `test_front_row_lever_debits_the_row_it_does_not_move` | 2 throttles × `v` ∈ {0, 0.20, 0.3536, 0.60} | 1 |
| `test_positional_advantage_is_currency_dependent` | 3 `v`, two currencies | 1 |
| `test_lever_relocates_the_binding_row_to_itself_…` | 2 throttles × 3 `v` | 1 |
| — | — | **+1 ADDED**: `…mach_guard_is_latent_not_absent` |

18 + 21 = 39 Python names → 37 ported (one deferred) + 4 from two splits + 2 added = **43**.

**FINDING 1 — THE FIFTH GATED-CODE EDIT LANDS AT STEP 5, AND § 5.10's STEP TABLE IS NOW REFUTED
A THIRD TIME.** `test_rung55.py:481` sets `m._V_SCAN = 0.01` — a Python CLASS attribute overridden
per instance. Rust's associated const cannot be, so `StageStackCore` gains a private `v_scan`
field, a `with_v_scan` builder, and two `Self::V_SCAN` → `self.v_scan` reads. The step table calls
step 1 *"ALL changes to already-gated code"*; step 3 refuted that once (`TwoSpoolMapCore`'s two
stack fields), and this is the second refutation from a direction neither step looked. Stated at
the level it generalises: **porting the CODE does not bound the edits the TESTS force** — a suite
can reach inside a constant, and no amount of reading the source module will show you that.
`grep '\._[A-Z_]* *='` over the four suites 53–56 returns exactly this one override and one
*assertion* (`_V_STEP == 0.04`), so the scan was cheap and should have run at pre-flight.

The edit is **value-neutral on everything already gated**: `slice_n_oracle.rs`'s 72 520 keys,
`slice_n_smoke.rs`'s 1 337, `rung53.rs` and `rung54.rs` all re-ran bit-identical.

**FINDING 2 — AND THE OVERRIDE MOVES NO VERDICT, WHICH THE OBVIOUS GATE COULD NOT HAVE TOLD ME.**
The natural place to gate `with_v_scan` is the row-count experiment that needs it, and that gate
is **vacuous**: measured, `test_p3_row_count_has_an_interior_optimum` passes unchanged at the
default 0.05. Measured in Python at both steps, `rows = 1..6`:

```text
    v* at 0.01   0x1.6a19e5f8b8522p-2 …      RELIEF  0x1.867f37d1b88f3p-4 …
    v* at 0.05   0x1.6a19e5f8b999ap-2 …      RELIEF  0x1.867f37d1b8b57p-4 …
```

— the roots differ from about the 11th decimal (the bisection stops on `INC_TOL = 1e-12` in the
RESIDUAL, so a different bracket lands on a different root) and every bar in the experiment is
orders above that. So the edit is justified by FAITHFULNESS, not by a failing gate, and it gets
its own gate (`test_the_scan_step_is_an_instance_value_and_it_moves_the_root`) asserting the field
is LIVE, that the default IS the const, and that the move is below every bar. **A dead knob's
spelling still has to be right** — this slice's `_P_FLOOR`/`_INC_MAX` lesson, third instance, now
on a knob whose deadness is a VERDICT's rather than a value's. It also **refutes the source's own
stated reason** for the finer scan (*"the reversal was first seen at a coarse scan and could have
been a bracket artifact"*): it was not.

**FINDING 3 — TWO OF THE SOURCE'S OWN GATES ARE VACUOUS, AND ONLY INJECTING DEFECTS FOUND THEM.**
Both suites passed on the first run, which is step 2's warning, so four detectors were injected
into `stage.rs` and each gate watched:

| injected defect | what fired | what did NOT |
|---|---|---|
| `K = 1` dispatch deleted (`if false && …`) | **NOTHING** — until the census clause was added | the whole of `test_reduce_stack_object_dispatches_at_K1` |
| `K = 1` throat row RE-DERIVED instead of copied | `test_reduce_k1_is_rung54_throat_margin_bit_for_bit`, by **1 ULP** | — |
| `Split::Tau`'s ladder collapsed onto `Split::DT` | rung 56's `test_split_is_load_bearing…` at `rel = 0.0000` | **rung 55's `test_p6_verdicts_survive_the_work_split`** |
| the `v_scan` override removed | `test_the_scan_step_is_an_instance_value…` | `test_p3_row_count_has_an_interior_optimum` |

* **`test_reduce_stack_object_dispatches_at_K1` cannot see the dispatch.** Its docstring says the
  value equality shows *"it is the same code and not merely the same algebra"*. It does not: the
  fall-through bisects the SAME bracket `[0.1, 2.0]` to the SAME `1e-14`, and its residual
  `tau_of − tau_c` differs from the map's `psi*n² − target` by a POSITIVE affine factor, which a
  bisection reading only SIGNS cannot see — same `lo`/`hi` sequence, same `0.5*(lo+hi)`, same
  bits. Gated structurally instead, off `stage.rs`'s own census: a dispatched call runs **0**
  bisection passes and **0** marches; the fallen-through one runs **144** (3 × 48). *A documented
  gate that doesn't exist*, this time found in the SOURCE rather than in the port.
* **`test_p6_verdicts_survive_the_work_split` cannot see a DEAD split.** Every clause it makes is
  an upper bound, and every one is satisfied at `x == y` — so it cannot distinguish P6's claim
  (*disclosed, and no verdict rides on it*) from *the disclosed choice is dead code*. Two
  `assert_ne!` clauses close it, on the in-repo precedent that already exists for exactly this
  shape (step 4's `moved > 0`; Python's own rung-56 P4 gate ends with `!=`). Detector re-measured:
  the collapsed split now fails it. **A "nothing rides on this knob" gate is vacuous unless
  something else says the knob is LIVE** — and rungs 55/56 are two-sided only when read TOGETHER,
  which is not how a suite is read.
* `test_rung55.py:498`'s cost clause is vacuous too, and differently: `cost ==
  dict(sorted(cost.items()))` compares a dict to a re-ORDERED copy of itself and `dict.__eq__`
  ignores order, so it is `True` for any curve, the `or` short-circuits, and the monotonicity it
  looks like it gates is never evaluated. Measured on PyPy before porting — the costs ARE
  ascending (0.0230, 0.0529, 0.0931, 0.1509, 0.2432, 0.4309) — so the Rust asserts the LIVE half
  alone. The port is STRONGER here, deliberately and with the reason written at the gate.

**THE `slow` MARKERS, DROPPED AGAINST A MEASUREMENT.** Python marks six of these 39 `slow`
(5 in rung 55, 1 in rung 56). Measured: **`rung55.rs` 20 gates in 1.34 s, `rung56.rs` 23 in
0.25 s** — the whole of both suites is under two seconds against a gate whose slowest single
target is 246 s. Slice M's rule applies unchanged: port the gate, DROP the marker, re-introduce
`#[ignore]` only against a MEASURED cost. Nothing here earns one.

**ONE FRAGILITY WRITTEN DOWN RATHER THAN DISCOVERED LATER.** `take_census` reads AND RESETS a
thread-local, so the dispatch gate is correct only while it is the sole census consumer in that
binary. A second reader in `rung55.rs` would steal its tallies and the failure would read as a
physics disagreement rather than a harness one. Noted at the gate; if one is ever needed, the two
must be serialised.

**WHAT STEP 6 INHERITED.** `slice_n_deferrals` (in `rung55.rs`) is the ledger of record from here;
`slice_n_smoke.rs`'s `slice_n_deferrals_so_far` stays where it is, with its own outcomes, as the
trail between steps.

##### STEP 6 — SHIPPED, docs-only. **THE STEP WAS SCOPED AT THE WRONG DEFECT**

The step table wrote step 6 as *"§ (iii)'s dead constants, if the specs assert otherwise"*. Read:
`grep` over `docs/rung55-spec.md` and `docs/rung56-spec.md` for `_P_FLOOR`, `_T_FLOOR`, the two
loop caps and `v_hi` returns **nothing** — neither spec mentions a single one of them, so the
correction the step was named for **does not exist**. What the slice actually owed was found at
step 5 and could not have been anticipated at pre-registration, because it is not about the code
at all: **three of the source's own gates assert nothing, and one spec sentence over-states why a
knob is there.** All four are written into the two specs, quoting the injected defect that
exposed each:

* `rung55-spec.md` § Verification gates — gates 1, 7 and 9's vacuity, each with its detector.
* `rung55-spec.md` § the row-count table — `_V_SCAN` = 0.01 CONFIRMS the reversal rather than
  rescuing it; the curve is scan-step-invariant to ~1e-11.
* `rung56-spec.md` § Verification gates — its gate 7 is the one that DOES catch a dead split
  (and rung 55's does not), and its gate 1 is genuinely tight at 1 ULP.

The Python suites are left untouched: repairing the source's gates is a change to the oracle's
tests, which § 8 puts outside the port. Recorded rather than done, with the measurement beside
each, so whoever takes it is not re-deriving it. **A step named after a predicted defect finds
the predicted defect or nothing** — the useful ones arrived from the step before it.


- ~~**The diamond.**~~ **DISCHARGED — § 5.3.** `StatorBleedMatcher(TwoSpoolBleedMatcher,
  VariableStatorMatcher)` collides on **one** name (`__init__`, no constant), rung 61 **opts out
  of the MRO by hand**, and rung 53 shadows nothing on the plant — so the flattened linear chain
  IS the MRO. The prescribed action was carried out and **widened**: `engine.rs` has no matchers,
  so phase 5 is the port's FIRST meeting with Python inheritance, and the diamond is the easy
  edge. **The real requirement is the FIVE-name virtual set** (`_solve_turbine`, `match`,
  `_hp_eta_loop`, `_lp_eta_loop`, `at_setting`) **plus one live constant shadow** (`_INC_MAX`
  80→200, read by inherited rung-53 solver loops). **NOT closed:** `_solve_turbine` is rung 31's,
  called on `self` in rung 31's body, and overridden by **phase 6's** rung 34 — so phase 5's
  first slice must ship it hookable or phase 6 refactors gated code.
- **`main.py`.** 5,236 lines, almost all formatted tables plus one chart. Least valuable to
  port, most tedious (Rust's formatting is clumsier). Three ways: port straight; emit data and
  plot with a Rust crate; or keep one Python script for the chart — which **violates the
  oracle-only rule**, so it needs an explicit decision (§ 9).
- **Test-suite mapping.** `pytest.mark.parametrize` → loops or a declarative macro. Module
  fixtures → `OnceLock` statics, built **once per process** rather than once per worker — your
  own notes record per-worker fixture rebuilding adding 2:37 to a 2:59 gate, and that cost
  disappears rather than shrinking. ~~`-m "not slow"` → `#[ignore]` + `cargo test -- --ignored`.~~
  **CORRECTED by slice M's step 5 — measure, don't map.** The `slow` marker records a COST, and
  the cost does not survive the port: slice M's 13 marked gates run in ~2 s total. Port the gate,
  DROP the marker, and re-introduce `#[ignore]` only against a measured cost — carrying it
  forward would silently deselect real gates to save seconds.
- **Speed, honestly bounded.** The measured 2.84× (PyPy) / 8.30× (CPython) was *three kernels
  in one march* and must not be extrapolated. Estimate — **labelled an estimate** — the gate
  lands around 2–4 minutes against today's ~17:21.

### The four runtime-introspection tests, one by one

| test | what it asserts | replacement |
|---|---|---|
| `test_rung71.py:241`, `test_rung73.py:477`, `test_rung72.py:414` | a parameter (`s_off`, `tau_rel`) is **absent** from a method's signature | Pass that hook a **narrowed config view** — a struct holding only the fields it may read. "Not reachable" becomes a compile error: strictly stronger than the test. **Cost, decided in phase 7 not discovered in it:** a hook taking a narrowed view cannot share a `Hooks` field type with one taking `&Config`, so either the table carries per-hook parameter types (fine — just more struct) or these three fall back to `include_str!` like the row below. |
| `test_rung73.py:488` | `src.count("g_own + req - clip") == 1` | `include_str!` + `.matches().count()`. **Verified in the spike** (16,430 bytes at compile time, count assertion passed). Stronger than the original, which re-reads from disk at import-cached line numbers. |
| `test_numeric_fingerprint.py:2193` | every golden kernel is reached by some test | Register keys through a macro that records them — the check becomes structural rather than textual. `include_str!` is the fallback. |

---

## 7. What carries the understanding

The 20,420 comment lines transcribe across unchanged; the 190 spec documents do not move at
all. Two genuine gains: the "components are pure functions" contract becomes compiler-enforced,
and one file per rung makes each rung's diff its own history. One genuine loss: every optional
lever becomes an explicit type, which is noisier to read than `None`.

**The port should be judged on whether the Rust reads as well as the Python, not on the clock.**

---

## 8. What is NOT in scope

Nothing on `CLAUDE.md`'s open list gets built during the port. No rung 85. The port is a
translation with a bit-exactness contract, not a re-foundation — mixing the two would make
every disagreement with the oracle ambiguous.

---

## 9. Decisions — ANSWERED 2026-08-12

1. **The bit-exactness bar → OPTION B.** Python is the oracle; agreement is required to a
   declared tolerance across every golden key, the deviation distribution is published, the
   fragile rungs of § 4 are adjudicated individually, and only then are Rust's values frozen.
   The CPython fingerprint stays in git history as the audit trail.
2. **`main.py` → SPLIT, on the rule "the ENGINE is pure Rust".** Everything computational,
   including the station tables the working contract requires every run to print, is Rust.
   Rust also emits the plot's data as JSON. **One small Python script owns the matplotlib
   chart only** — it does no physics, reads no engine code, and is the single permitted
   exception to oracle-only. *To verify in phase 8: that it stays fast (the assumption behind
   the decision).*
3. **Go → PHASES 0–1 ONLY.** Build the scaffolding, the oracle bridge and the gas core, then
   stop and re-decide before phase 3 (the first heavy consumer of that arithmetic).
   **Superseded 2026-08-12: phase 2 was authorised and is complete.** The stop-and-re-decide
   point is now **before phase 3**, which is where it was always going to matter.
   **Superseded again 2026-08-12: phase 3 was authorised and is under way, in SLICES.** The
   phase is the port's largest (2,745 source lines, 204 tests, eight mutually-exclusive mixing
   closures), so it shipped one green gate at a time rather than as one landing. **All five slices
   are now done — A (§ 4.3), B (§ 4.4), C (§ 4.5), D (§ 4.7–4.8) and E (§ 4.9–4.10) — and PHASE 3
   IS COMPLETE.** No further authorisation was needed inside phase 3; **phase 4 (the nozzle &
   turbine marches, rungs 25–30) has not been authorised and is the next thing to ask about.** The
   standing re-decide point remains **before phase 5**, which contains the diamond (§ 6).
   **Superseded 2026-08-13, in TWO steps.** The re-decide point was reached and taken in the order
   the plan asked for: first the **PRE-FLIGHT ALONE** was authorised — § 6's prescribed diamond
   action, which discharged it and found the phase's real structural content (§ 5.3) — and only
   then, on that evidence, **PHASE 5 ITSELF was authorised on 2026-08-13, in SLICES**, on the
   phase-3/4 pattern (pre-register, port code with its tests, gate every value, ship one green
   slice at a time). Slices are free inside the phase; **the next authorisation point is before
   phase 6.**

**Decision 1 is REVISED by § 4.2**: phases 0–2 are held to bit-equality, not to a tolerance,
because it was measured achievable (100 % on both oracles) and because a tolerance bar let a
real defect ride for a whole phase. Later phases may fall back to Option B, with the deviation
distribution published here.

### Consequences for the phase table

Phase 8's `main.py` row is now "Rust CLI prints the tables and dumps plot JSON; port the
chart script; verify it is fast" rather than a three-way choice.
