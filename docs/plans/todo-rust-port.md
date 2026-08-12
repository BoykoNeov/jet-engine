# The Rust port — plan

**Status: PHASES 0–2 COMPLETE AND GREEN; PHASE 3 IN PROGRESS — SLICES A (rungs 7/8/9/19) AND
B (rungs 10/11/12/20) DONE.**
The architecture is settled by measurement (§ 1–2); the three forks were answered on 2026-08-12
(§ 9); phases 0–2 were then built and gated (§ 4.1, § 4.2). Phase 1 was the first deliberate
stopping point because it is where the arithmetic risk concentrates; phase 2 was authorised
separately and **corrected phase 1's central diagnosis** — see § 4.2, which is the answer to
the question phase 1 thought it had already answered. Phase 3 was authorised on 2026-08-12 and
is being taken in slices, because at 2,745 source lines and 204 tests it is the largest phase
in the port; § 4.3 records slice A, § 4.4 slice B.

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
| the PDF family | 13, 15, 16, 18, **21** | rung 13's `_bell_interpolator`; **21 is the same lift through the ideal-bell integrals** |
| the nozzle strand | 14, 17 | independent of the mixing closures — portable at any point |
| the resolved cross-plane | 22, 23, 24 | rung 13's bell; otherwise self-contained |

Slice B leaves the quench machinery (`quench_trajectory` / `quench_no` / `JetMixing` /
`Unmixedness`) in place, which is what rungs 15 and 16 build their dwell chain on — so the PDF
family is now the natural next slice, and the nozzle strand stays portable at any point.

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

### The rungs where a tolerance is NOT a valid substitute

The finding is a **count**, and a count jumps discontinuously:

| rung | the claim | why a tolerance does not cover it |
|---|---|---|
| **9** | the EI bell PEAKS near φ≈0.95 | **MEASURED (§ 4.3):** the two interpreters disagree on the peak VALUE and agree on its LOCATION — so the argmax is the only key that answers "did the finding move?" |
| **10** | `T(β)` peaks AT the stoich crossing | **MEASURED (§ 4.4):** an interior argmax for a rich primary, index 0 for a lean one. A trajectory run backwards leaves every individual temperature plausible and breaks only this. |
| **12** | the EI-min is pinned **AT** `C_opt` | **MEASURED (§ 4.4), and the claim NARROWED by measuring it:** the argmin is the finding, and it moves off `C_opt` past `S/S_x ≈ 1.2`. A value tolerance would have reported "agrees" on every one of those points. |
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
| **3** | NOx & mixing, rungs 7–24. **RISK-BEARING — not bulk.** These are phase 1's largest *consumer*: every one rides the equilibrium solve and `Kp = exp(−ΔG°/RuT)`, and their findings are *shapes* (the bell's peak, the minimum pinned at `C_opt`, monotone-vs-turns-back-up) that a last-digit shift in an exponential can move. Deliberately placed straight after phase 1 as the **first real test of whether the transcendental arithmetic holds**. **IN PROGRESS — slices A (7/8/9/19) and B (10/11/12/20) DONE**, § 4.3–4.4; the remaining slices are grouped in § 4.3 by DEPENDENCY, not by number (rung 21 belongs with 13–18) | 4–6 | ✅ slice A: `nox_oracle.rs` (**1806/1806** bit-exact vs PyPy on 22+22 distinct solver roots) + 4 rung suites (43 tests) · ✅ slice B: `quench_oracle.rs` (**2507/2507**, on 165 distinct trajectory roots) + 4 rung suites (39 tests); extremum *locations* dumped as their own keys, and one of them NARROWED a shipped claim (§ 4.4) |
| **4** | Nozzle & turbine marches, rungs 25–30 — own convergence behaviour, hence separate | 2–3 | per-rung tests pass |
| **5** | Steady matchers — rungs 31–33, 38–39, 42, 53–56, 61. **Contains the diamond** (§ 6) | 4–6 | per-rung tests pass |
| **6** | Transients — rungs 34–37, 40, 43–52 (the fuel-side limiter family) | 4–6 | per-rung tests pass |
| **7** | **The ladder, rungs 57–84** — the `Hooks` table from § 2, one module per rung | 5–8 | 28/28 reduce-to-prior bit-exact |
| **8** | `main.py` replacement; adjudicate the fragile rungs; re-anchor the fingerprint; **delete the Python** | 2–3 | full suite green on Rust alone |

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
   closures), so it ships one green gate at a time rather than as one landing. Slices A (§ 4.3)
   and B (§ 4.4) are done. No further authorisation is needed inside phase 3; the next re-decide
   point is **before phase 5**, which contains the diamond (§ 6).

**Decision 1 is REVISED by § 4.2**: phases 0–2 are held to bit-equality, not to a tolerance,
because it was measured achievable (100 % on both oracles) and because a tolerance bar let a
real defect ride for a whole phase. Later phases may fall back to Option B, with the deviation
distribution published here.

### Consequences for the phase table

Phase 8's `main.py` row is now "Rust CLI prints the tables and dumps plot JSON; port the
chart script; verify it is fast" rather than a three-way choice.
