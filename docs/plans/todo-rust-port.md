# The Rust port — plan

**Status: PHASES 0–3 COMPLETE AND GREEN — slices A (rungs 7/8/9/19), B (10/11/12/20),
C (13/15/16/18/21), D (22/23/24) and E (14/17, the nozzle strand) all shipped.
PHASE 4 (the nozzle & turbine marches, rungs 25–30) was AUTHORISED 2026-08-12 and runs in three
slices: F (25/26) is SHIPPED — § 4.11 pre-registration, § 4.12 measurement — and G (27/28) and
H (29/30) are next. 330 Rust tests.**
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

**THE PORT DECISIONS TAKEN, all four as pre-registered in § 4.11**: a new `march.rs` (rung 30 will
go to `components.rs`); `choked_mfp` deferred to phase 5, checked against the tests rather than
assumed; the two non-oracle reduces gated as named tests; and the three bisection tolerances
transcribed separately, with `equilibrate_hp` dumped DIRECTLY as well as through its caller so a
mis-transcribed `1e-10` names the equilibration instead of reading as a nozzle defect.

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
| **4** | Nozzle & turbine marches, rungs 25–30 — own convergence behaviour, hence separate. **AUTHORISED 2026-08-12; in three DEPENDENCY slices — F (25/26) SHIPPED, G (27/28), H (29/30)** | 2–3 | ✅ slice F: `march_oracle.rs` (**912/912** bit-exact vs PyPy, on 49 distinct march exit roots) + 2 rung suites (32 tests) in a new `march.rs`; the FOURTH "exactly"-class claim and the FIRST to survive — because it compares a COPY, not a rederivation (§ 4.12) |
| **5** | Steady matchers — rungs 31–33, 38–39, 42, 53–56, 61. **Contains the diamond** (§ 6) | 4–6 | per-rung tests pass |
| **6** | Transients — rungs 34–37, 40, 43–52 (the fuel-side limiter family) | 4–6 | per-rung tests pass |
| **7** | **The ladder, rungs 57–84** — the `Hooks` table from § 2, one module per rung | 5–8 | 28/28 reduce-to-prior bit-exact |
| **8** | `main.py` replacement; adjudicate the fragile rungs; re-anchor the fingerprint; **delete the Python** | 2–3 | full suite green on Rust alone |

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

---

## 6. Named risks

- **The diamond.** `StatorBleedMatcher(TwoSpoolBleedMatcher, VariableStatorMatcher)` — rung 61,
  two parents, resolved by Python's method order. Neither a `Prev` chain nor a linear table
  handles multiple inheritance. **Action:** before phase 4, write down what Python's resolution
  order actually produces, flatten by hand, gate with rung 61's existing reduce test.
- **`main.py`.** 5,236 lines, almost all formatted tables plus one chart. Least valuable to
  port, most tedious (Rust's formatting is clumsier). Three ways: port straight; emit data and
  plot with a Rust crate; or keep one Python script for the chart — which **violates the
  oracle-only rule**, so it needs an explicit decision (§ 9).
- **Test-suite mapping.** `pytest.mark.parametrize` → loops or a declarative macro. Module
  fixtures → `OnceLock` statics, built **once per process** rather than once per worker — your
  own notes record per-worker fixture rebuilding adding 2:37 to a 2:59 gate, and that cost
  disappears rather than shrinking. `-m "not slow"` → `#[ignore]` + `cargo test -- --ignored`.
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

**Decision 1 is REVISED by § 4.2**: phases 0–2 are held to bit-equality, not to a tolerance,
because it was measured achievable (100 % on both oracles) and because a tolerance bar let a
real defect ride for a whole phase. Later phases may fall back to Option B, with the deviation
distribution published here.

### Consequences for the phase table

Phase 8's `main.py` row is now "Rust CLI prints the tables and dumps plot JSON; port the
chart script; verify it is fast" rather than a three-way choice.
