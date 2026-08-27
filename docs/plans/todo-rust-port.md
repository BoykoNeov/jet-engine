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
predicates, not an argmin. **SLICE P (rungs 34/35/36, the port's first ODE) IS SHIPPED** — 7 300
oracle keys bit-exact (§ 5.13). **SLICE Q (rung 37) IS SHIPPED** — 2 066 oracle keys bit-exact
(§ 5.14); its leading result is that the `try_illinois` exhaustion arm slice P measured at ZERO
firings and could only close with a counter is here the path 94.5 % of one call site's calls
take, worth 456 oracle keys and **zero of ten gates**. **SLICE R (rungs 40/44) IS SHIPPED** — 6 853 main + 1 120 reacting oracle keys bit-exact,
four steps (§ 5.15). **SLICE S (rungs 43/45, the whole `integrate_fuel`) IS SHIPPED**
— 4 671 main + 1 133 gas oracle keys bit-exact, all five steps (§ 5.16); its leading result is that
FOUR of this section's own registered census numbers came off a probe whose header called its grid
the two suites' and was not, and § 5.12's own IOU —
the arming predicates on a gas the CPython detector can see — is **discharged with a
measured margin**: the detector moves 391 of 398 value keys on every TPG gas the fuel path
admits, and the nearest arming threshold sits **seven orders** away, so nothing flips.
**SLICE T (rungs 46/47/48) IS SHIPPED** — four steps (§ 5.17), `topping_oracle.rs` **1 729 keys**
bit-exact; the first slice whose source already shipped, so its target was slice S's ungated ~40 %.
**SLICE U (rungs 49/50/51/52) IS SHIPPED** — five steps (§ 5.18), `release_oracle.rs` **4 179 keys**
bit-exact vs PyPy *and* CPython; its leading result is that a suite whose thesis is INVARIANCE is
structurally blind to what its keys ARE, and the gate written to close the slice was itself caught
comparing my formula with my formula. **PHASE 6 IS THEREFORE COMPLETE (2026-08-20), and the next
authorisation point is PHASE 7 — it has not been authorised.**
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

> **⚠ THIS TABLE IS A SPIKE MEASUREMENT, NOT A CENSUS — CORRECTED 2026-08-20 by § 5.19 (i)–(iii).**
> Enumerated against today's `engine.py`, the hook set is **38 names**, not eight; `_instant_fuel`
> is **not a hook at all** (its two definitions are on SIBLINGS, § 5.12's own bug); the
> `..R63` struct-update spelling above **cannot express** the 16 `super(LimitedBleedTransient,
> self)` sites, which pin rung 62 regardless of depth; and *"Rust deletes `at_lever`/`_shared_rig`
> outright"* is sound only once **ten out-of-band fields this section does not know about** are in
> `Config`. The call-rate columns are still the best evidence for *hot vs cold* — which is what
> they were measured for — and nothing here changes the architecture's verdict.

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

~~**The trait is ~8–10 methods, not 40.**~~ **REFUTED 2026-08-20 — § 5.19 (i): it is 38, of which
~30 are new to phase 7.** The reasoning below is sound and the conclusion was still wrong, because
"defined exactly once" was checked and "overridden at least once" never was. 264 methods are
defined exactly once (each rung's own diagnostics — free functions, not hooks); 26 more are sibling
constructors that Rust deletes — **but not for free** (§ 5.19 (iii)); `integrate_fuel` is 2 real
bodies plus 10 thin delegating wrappers.

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
| **6** | Transients — rungs 34–37, 40, 43–52 (the fuel-side limiter family). **AUTHORISED 2026-08-17; PRE-FLIGHT DONE (§ 5.12). ALL SIX SLICES SHIPPED (§ 5.13–§ 5.18) — PHASE 6 IS COMPLETE 2026-08-20. PHASE 7 NEEDS ITS OWN AUTHORISATION.** The phase's structural content is a **six-name virtual set** — `integrate_fuel`, `_close`, `_close_fuel`, `_surge_fuel`, `_instant_tail`, `_powers` — **every one of which crosses into phase 7**, so there is no phase-6-internal hook and the `Hooks` table appears at slice R, not P. Six slices, ordered by `_degenerate`: **P** (34/35/36 `SpoolTransient`) · **Q** (37) · **R** (40/44) · **S** (43/45, `integrate_fuel` ENTIRE) · **T** (46/47/48 gates) · **U** (49/50/51/52 gates). The `4257–4506` object block spans two phases — `IncidenceLimiter` is rung **60** | 4–6 (**light** — 156 tests over 15 files, and phase 3 took five slices for 204) | ✅ slice P: `spool_oracle.rs` (**7 302** bit-exact) + 3 rung suites · ✅ slice Q: `combustor_oracle.rs` (**2 066**) + 1 suite · ✅ slice R: `two_spool_transient_oracle.rs` (**6 853 + 1 120** bit-exact vs PyPy on the first run, + a tiered CPython arm reproducing probe 4's 5-of-12 exactly) + `rung40.rs`/`rung44.rs` (17 tests) + `slice_r_smoke.rs`/`slice_r_dispatch.rs`; a ported suite found running its NEIGHBOUR's gas (§ 5.15 step 4) · ✅ slice S (43/45, `integrate_fuel` ENTIRE — the phase's largest): `fuel_transient_oracle.rs` (**4 671** CPG + **1 133** gas keys bit-exact vs PyPy on the first run) + `rung43.rs`/`rung45.rs` (20 tests) + `slice_s_smoke.rs`/`slice_s_dispatch.rs`, five steps; § 5.12's CPython-detector IOU CLOSED by measurement on the three TPG gases the fuel path admits (§ 5.16) · ✅ slice T (46/47/48 gates) § 5.17, **ALL FOUR STEPS SHIPPED** — the FIRST slice whose source already shipped, so its predictions target slice S's ungated ~40 % rather than a port. `rung46.rs` (7 fns for 6, one a DISCLOSED divergence) + `rung47.rs` (9 for 9) + `rung48.rs` (16 for 16), **0 source lines** after step 1 until step 4's one behaviour-neutral HOIST; step 2 measured the rung-47 suite to have **no value content** (two injections move 13 of 18 readings by up to 24 %, all 9 gates pass) and step 3 found its one surviving defect unobservable to the WHOLE project — every reader of `fuel_removed` is scale-invariant; step 4's `topping_oracle.rs` (**1 729** keys bit-exact vs PyPy on the first run + a CPython arm, 4 tests in 53 s) HOLDS all three of those defects and corrects the slice's OWN finding 6 — `test_rung46.py`'s `~1645` peak is right, it is measured on a different GAS · ✅ **slice U (49/50/51/52 gates) — § 5.18, four probes measured first; 63 gates + 9 readers over 418 source lines, five steps; **ALL FIVE SHIPPED — SLICE U AND PHASE 6 COMPLETE** — `rung49.rs` (17 fns for 17, 0.38 s) over `SurgeRelief`/`surge_relief`/`floor_sweep` and finding 6's `read_point`, **226 source lines added and 0 executable lines changed**, **575 keys over all 23 gate cells bit-exact vs PyPy on the first run** — and then ELEVEN injections found **five of the 25 keys have no gate in either language**, with the one-ulp boolean protected REDUNDANTLY by two spellings (breaking either alone is invisible, breaking both flips it, and all four builds are 17/17). Its refusal census came back COMPLETE (all 14 marcher refusals present in Rust) but found **three of Python's own `lp_disabled` refusals UNREACHABLE over all 255 arming combinations**, so four gates named for four rungs fire ONE assert; and — inverting slice T's finding 2 — one decision key (`both_edges_inside_ramp`) sits at **ONE ULP**. **Step 2** adds the 27-field `ReleaseRelief` + `release_relief` (complete, with rung 51's `tau_rel`) + `release_sweep` in **229 added and ZERO deleted lines**, and `rung50.rs` (15 fns for 15, 15/15 in 0.90 s), **1 323 keys over all 49 gate cells bit-exact vs PyPy on the first run**; then FOURTEEN injections found the file's ONE reader of the release edge's LOCATION sweeps **none of the two knife-edge cells**, so a `k*ds` coordinate moves `n_engaged` 8→7 and `s_rel` by a WHOLE CELL past all 15 gates. **P2 CONFIRMED by measurement** (the gate named for rung 50 fires rung 49's assert, full-string), 16 of the 27 keys have NO reader — `deficit_at_release`, the rung's own named quantity, among them — and `fuel_removed` is held only as a DIFFERENCE between two copies: break all THREE and rungs 48/49/50 are green. **Step 3** (`rate_sweep` + `deficit_curve`, **65 added / 0 deleted** — two `map` bodies and one assert, which IS P6's check) + `rung51.rs` (16 for 16, 16/16 in 0.91 s on the FIRST compile), **972 keys over 36 cells bit-exact**, cell list READ OFF the suite's own memo; its one new function is exercised only on cells CHOSEN FOR INERTNESS — dropping `tau_rel`'s forwarding moves 2 of 972 keys and both are the record echoing its own argument back. **Step 4** (34-field `LagRelief` + `FactorizationGrid` + three readers, **294 added / 0 deleted**) + `rung52.rs` (15 for 15, 15/15 in 0.75 s including all four of Python's `slow` gates, so **no `#[ignore]`**), **972 keys bit-exact**, **P5 EXACT** (`credit_spread == 0.0` bit-for-bit); ten injections and TWO wrong predictions with ONE cause — `g_at_cross` and `min_phi_hp_lag` are read only as INVARIANCES ACROSS a sweep, so **a gate that reads a key only by comparing it with itself cannot see what the key IS**, and a suite whose THESIS is invariance is structurally that shape. **Step 5**: `release_oracle.rs` + `dump_release.py`, **4 179 keys over eight sections bit-exact vs PyPy on the first run AND vs CPython (0 drifts, 0 flips, no tolerance tier)**, 5 gates, three ADDED sections (both NaN arms reached for the first time in this port, `rate_sweep` inside the window, the knife-edge coordinate) — and TWO near-vacuous gates caught in that one step: the manufactured cells were first written against a re-spelled copy of the loop (fixed by lifting `crossing_census` out, behaviour-neutrally, + a fifth gate that the reader and the manufactured cells share it) and the CPython arm routed every disagreement to a PRINTOUT and could not fail on a number |
| **7** | **The ladder, rungs 57–60 and 62–84** — the `Hooks` table from § 2, one module per rung. (**61 is PHASE 5's**, not this phase's — it is the steady `StatorBleedMatcher`, and it was double-listed here until the slice-K audit) **PRE-FLIGHT DONE (§ 5.19); PHASE 7 AUTHORISED 2026-08-20 — slice V is PRE-REGISTERED (§ 5.20), and its six probes REFUTE § 5.19 (x) on slice V: the `&Scope` lands at slice Y/AA, and V's real content is `_arm`'s PERMANENT mutation of `map_lp`/`map_hp` from inside a `&self` hook — a shape the pre-flight's `try/finally` census could not match. Carrier: `Cell<ComponentMap>`.** The phase's structural content is a **36-cell hook table** (§ 2 said 8, § 5.12 said 6, the enumeration says **38** = **28 new + 8 already shipped + 2 Rust deletes**), **16 non-zero-argument `super(LimitedBleedTransient, self)` pins** that `..R63` cannot express, and **23 dynamically-scoped fields behind 52 save/set/restore guards** — 9 of them the current RK4 state, which forces a `Scope` parameter on **7 of the 36 cells** and closes § 6's narrowed-view question at zero cost. Fifteen slices **V…AJ**, ordered by where the TABLE GROWS | ~~5–8~~ **15–20** (§ 5.19 (viii): **4.34×** phase 6's source, **3.49×** its collected tests, **263 of 548** carrying `slow` against phase 6's 10 of 157) | ~~27/27 reduce-to-prior bit-exact~~ **that is a spine, not a gate** — the phase owes per-slice **oracle dumps** bit-exact vs PyPy, the **488 ported gates**, the 27 reduce contracts, **and dispatch gates** (`slice_r_dispatch.rs`'s precedent — no value key can witness a hook table) — **and, from slice V, CARRIER gates**: § 5.20 (ii) measures a local-armed-core port moving `margin_min_lp` **15.4 %** with **59/59** ported gates green, so a manufactured-carrier-bug gate is owed beside the manufactured-cell-swap one. **SLICE V COMPLETE — all five steps SHIPPED** — step 4's `slice_v_oracle.rs` + `dump_slice_v.py` is **6 819 keys bit-exact vs PyPy on the first run AND vs CPython 3.14 (0 drifts, 0 flips, no tolerance tier)** over eight sections, and it **CATCHES both carrier injections the 59 ported gates miss** (I1 at 87 keys, I2 at 709) while REACHING the 15.431 % channel step 3 had booked forward; and step 5's `slice_v_dispatch.rs` (6 gates) DISCHARGES P5 — the manufactured carrier gate, whose four golden-free assertions survive a regenerated golden, with the `surge_margin` collapse (*a scoped port reads a SCHEDULED machine bit-for-bit as an UNSTATORED one*) as its bar-free headline. **SLICE W IS PRE-REGISTERED (§ 5.21) off five probes, and its cell census REFUTES § 5.19 (x)'s column over the WHOLE phase, not just over W: the measured total is **35** (8 shipped + 27 to build), not 28, and the four names the hand-written column missed are `at_stator` (V), `at_lever` (W — 17 overriders, 46 call sites), `_quad_gains_at` (AD) and `_with_coord` (AF). `at_stator` is NOT the inert deferral § 5.20 booked: a shipped rung-63 gate reads it directly and the port's current `-> ScheduledStatorCore` return flips its verdict from `True/True` to `False/False`.** **SLICE W COMPLETE — all five steps SHIPPED.** Step 4's `slice_w_oracle.rs` + `dump_slice_w.py` is **9 422 keys bit-exact vs PyPy AND vs CPython 3.14** over ten sections, with **ONE declared cross-interpreter exemption that is itself a finding**: CPython 3.12+'s `sum()` is Neumaier-COMPENSATED and PyPy's is naive, so 7 of the 9 `commanded_level` `mean` keys differ by LANGUAGE rather than by code — the first arm in the port to catch a divergence in the interpreter instead of the port, and it falsified a claim in the shipped Rust's own comment. Step 5's `slice_w_dispatch.rs` (5 gates, zero source lines) builds the TWO instruments step 3 measured to be blind — `b_of_calls` beside the eight inert reduced/bled pairs, and a `Floor::Incidence` cell beside the `Floor::Phi` one the suites' inputs cannot discriminate — discharges P2, and carries a **6-row mutation table** in which one of its own gates lets a HALF-APPLIED injection through. **SLICE X COMPLETE** (rung 64; 318 oracle keys bit-exact on both interpreters, and its closing step found a zero-count assertion satisfied by DELETING the branch it names). **SLICE Y COMPLETE 2026-08-27 — all five steps SHIPPED.** Step 4’s `slice_y_oracle.rs` + `dump_slice_y.py` is **35 994 keys bit-exact vs PyPy AND vs CPython 3.14 on the first run**, over nine sections, with **NOTHING coarsened** (the readers were TIMED first, so P8 is discharged by running the suite’s own grid rather than by disclosing a substitute); its six-injection census then measured WHAT THE ORACLE WOULD CATCH and found its blind spots to be **exactly** the three the pre-registration named. Step 5’s `slice_y_dispatch.rs` (8 gates, no golden read by any assertion) gates those three, REPAIRS a fourth that was a real bug (`f64::max` is not Python’s `max` — they differ on a NaN in the FIRST position, which is exactly where `laws_held` puts one), and its nine-row mutation table caught **one of its own gates testing HALF of what it named**: a save-and-restore guard whose SET and RESTORE halves were owned by two different files, with neither covering both. **SLICE Z IS PRE-REGISTERED (§ 5.24) off NINE probes** — rungs 66 + 67, **1 496 source lines (3.12× slice Y) and 38 gates**, with **0 cells added by either rung** (the emitter confirms § 5.19 (x) a third and fourth time; both swap the same three already-open cells). Its leading finding landed on its OWN INSTRUMENT: the one float `sum()` in either rung is `cross_identity`’s `P_mid`, and the probe that asked whether CPython’s compensated `sum()` diverges from PyPy’s naive one chunked the products by the `n_sample` the gate PASSES (8) rather than the count the STRIDE delivers (9). At the right width the answer INVERTS — **a probe that reads a grid parameter instead of the delivered one measures a different function, and the answer it returns can be the opposite one.** The divergence is one ulp on 1 of `cross_identity`’s 3 rows and, measured rather than feared, propagates to exactly **2 of `_window`’s 8 keys without amplifying**. **STEP 1 SHIPPED 2026-08-27** — the plumbing, the six cells, the two builders and `RINGS`, with `slice_z_smoke.rs` (9 gates) gating the reduce arms and the seven refusals BEFORE either march exists. Its finding is about the SOURCE: **a march scope consumes its own field and drops the rungs above it, so rung 67’s `assert lag is None` cannot see rung 66’s CARRIER** — arming the fuel lag through `_stator_march` on a cascade-A machine is silently DISCARDED (measured, 171/171 bit-identical, with the instrument proved able to see one rung down). **P1 is falsified at its letter** (nine exhaustive `MarchScope` literals, eight in TEST files) **and its own “55 shipped call sites” was stale** — measured 82 un-scoped / 16 scoped. **STEP 2 SHIPPED 2026-08-27** — BOTH marches and all 20 method bodies, **2 576 Rust lines against 1 496 Python: a 1.72× expansion, UNDER slice W's 2.06× and under § 5.24 (ii)'s labelled ~3 000-line estimate.** The port was compared to the source BEFORE a gate was written — a throwaway bit-emitting probe pair over both marches, **all ten readers field-for-field**, the five leaf statics, the damping ladder at all four rungs and the detector's whole table: **785 keys, bit-for-bit vs PyPy on the first run.** `slice_z_smoke.rs` goes 9 → **18 gates, 0.98 s**, and **P2's six reduce arms onto five targets are gated six-for-six** — after a FOLLOW-UP caught the published tally wrong by one: section A's `LeverArm::default()` gate is rung 43/57's VALVELESS machine, not rung 64's floored-instantaneous one, and a 19th gate now covers FLOORED + UNLAGGED + NO CLOCK. Its finding is the mirror of step 1's and this half belongs to the PORT: **a `MarchScope` field is silently IGNORED by every rung below its owner, where Python raises `TypeError`** — one struct for a parameter Python adds one-per-rung. It bit **three typed route bars**, and the third bit past the witness that caught the first two: rung 46's unlagged governor and rung 47's lagged one emit the **same fourteen keys**, so `key_count` passed on both sides of a comparison whose floats disagreed. **A ROUTE WITNESS IS NOT A RUNG WITNESS.** Also measured rather than assumed: adding an enum variant breaks the exhaustive matches loudly and leaves the `_ => panic!()` ones SILENT, and a silent one is a **NARROWING** (four arms asked by hand, two widened, two left refusing); and `joint_ic_corners`'s catch arm fires on **NO** shipped grid, so its 120-character truncation is exhibited by a floor-violating `ds` with `msg_len`/hash measured on both languages. **SLICE Z COMPLETE 2026-08-27 — all five steps SHIPPED.** Step 3's `rung66.rs` (15) + `rung67.rs` (23) are green first run at **4.56 s against PyPy's 91.07 s (20.0x)**, and its leading finding is the INSTRUMENT's: the injection harness returned a `0 passed / 0 failed` baseline and called all eight injections invisible — **including the two CONTROLS** — because `cargo test` prints `Running` on stderr and `test result:` on stdout, so a `stdout + stderr` parse detaches every result from the target that names it. Caught by the ONE echoed baseline line; repaired with a bar (one target per invocation, and a refusal to run unless the baseline reads 15/23/19). Re-run, each injection twice, three of the four survivors are **provably** invisible (their liveness markers never fire). Step 4's `slice_z_oracle.rs` + `dump_slice_z.py` is **35 335 keys bit-exact vs PyPy on the first run** and vs CPython 3.14 with a named exemption **P3 pre-registered as TWO keys and measured at EIGHT NAMES** — `P_mid` is re-published four more times, so a transcribed list would have failed on six; and the stride finding recurs INSIDE the file, on two rows that are the same clock on the same trajectory sampled 9 wide and 7 wide, of which only the 9-wide diverges. Step 5's `slice_z_dispatch.rs` (8 gates, no golden read by any assertion) gates P6/P7/P11 and P4/P5, and its eleven-row mutation table found **two vacuities in a row in ONE gate**: a central difference cannot pin the constant its branch returns, and the bar written to repair that failed on its first run because the accel arm it defended was itself DORMANT. It also reclassifies **two of § 5.24 (v)'s five dead arms as UNOBSERVABLE** — deleting them changes no output on any input, so no gate can pin them. |
| **8** | `main.py` replacement; adjudicate the fragile rungs; re-anchor the fingerprint; **delete the Python**. **BLOCKED ON ONE OPEN PORT DIVERGENCE** — `components::sonic_throat`’s bracket `assert!` is a `panic!` where Python’s is a CATCHABLE `AssertionError` that every marcher’s `except AssertionError: break` relies on; measured at slice T step 1 (§ 5.17), 28 call sites of which ≥10 already sit in fallible chains, note at the definition | 2–3 | full suite green on Rust alone |

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

**DISCHARGED EARLY, AT SLICE R — and the detector fires.** Rung 40 gate 1 runs
`Gas.reacting_equilibrium()`, so the measurement booked above did not have to wait for slice S.
§ 5.15 probe 4 re-ran the reacting arm on `equilibrium`'s exit branch: **5 of 12 cells FLIP the
branch**, 10 of 12 disagree on the iteration count, 12 of 12 on the converged speeds. The CPython
arm is a detector on this gas, with a measured sensitivity. Slice S still owes the same
measurement **on its own object** — the arming predicates — because a detector's sensitivity is a
property of the quantity it watches, not of the gas alone.

**AND SLICE S HAS NOW PAID IT — § 5.16 probe 1.** `_tt4_from_f` refuses an equilibrium gas,
so the measurement had to be re-pointed at the three TPG gases the fuel path DOES admit. On
each of them the detector moves **391 of 398** value keys, and all three arming decision
sequences plus the cap-call and point counts are **identical** — with the nearest predicate
margin measured at **5.0e-3 relative against ~1e-10 of drift**. The IOU is closed by
measurement, not by impossibility, and the 100 % above still is not coverage.
**AND SLICE S STEP 4 CARRIED IT INTO THE SHIPPED ORACLE, WHERE IT CHANGED SIZE.** Probe 1's
measurement was a probe; the three TPG gases now ride in `fuel_transient_gas_pypy.tsv` as a
declared fragile set, re-measured on the oracle's own grid: **731 of 801 float keys move, median
2.0e-12, p90 4.1e-11**, and the exit PASS COUNT moves in **12 of 12** cells. The CPG half — 3 411
float keys — moves **15**, all of them `collapse_exponent`'s libm-composite scored curve and none
of them through the plant. So the shape probe 1 booked is confirmed and the "CPG moves nothing"
half of § 5.16 prediction 2 is refuted by a class rather than a fluke. **THIS IOU IS CLOSED.**


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

##### STEP 1 — SHIPPED. **TWO OF FIVE INJECTED DEFECTS ARE INVISIBLE TO 132 BIT-EXACT VALUES**

`src/spool.rs` (the port) + `ComponentMap::phi_max` + `tests/slice_p_smoke.rs`. **132 values
bit-exact against PyPy on the first run**, over eight cells chosen to touch every path once: a
choked equilibrium, a subsonic one, an off-equilibrium instant, a 21-step RK4 trajectory, rung
35's fuel control, rung 36's margin and its compounding, rung 41's channels, all six `phi_max`
shapes driven directly, and the forward/backward map inverse. Prediction 1 held; predictions 7
(the inverse is exact) and 9 (rung 41's channels port) held; prediction 8 held — no `Hooks` table
was needed in this module.

**FINDING 1 — THE SMOKE DUMP PASSED FIRST TRY, SO EVERY GATE WAS WATCHED TO FAIL, AND TWO COULD
NOT.** Slice N step 2's warning, taken as an instruction. Five defects were injected into the
shipped code and the 132 values re-run:

| injected defect | value gates failing (of 8) |
|---|---|
| the constructor takes `R31` instead of `R34` — rung 31's bisection on a rung-34 object | **5** |
| `fa *= 0.5` dropped — plain regula falsi instead of Illinois | **6** |
| the width test reads `\|c − b\|` (the new interval) instead of `\|b − a\|` | **6** |
| **the convergence test moved BEFORE `f(c)` is evaluated** | **0** |
| **exhausting `maxit` returns `a` instead of `b`** | **0** |

The last two are invisible **and the port's own doc comment asserted they were not** — it said all
three of the function's delicate details *"change the returned bits on cells that converge on the
final pass."* They do not, and both fail the same way: the reorder returns the **identical** `c`
and differs only in whether a residual is evaluated, and the exhaustion arm is **never reached**.
They are COUNT properties, not value properties — slice N FINDING 6's shape arriving in the port's
own scaffolding rather than in the physics. **The comment is corrected against the measurement,
and the blind spots are closed rather than documented:** `counters::illinois_evals` and
`illinois_exhausted` now instrument the loop, gated against PyPy counts measured by the same
method Python measures its own (a counting copy wrapping the shipped call sites). PyPy's
**227 / 403 / 1 344 / 199** reproduce on the first run, and re-injecting the reorder now fails
**exactly one** gate — the count one.

**FINDING 2 — THE HOOK IS DEAD ON THE SUBSONIC BRANCH, AND THAT IS WHY PREDICTION 2 IS A COUNT.**
Of the eight value gates, the wrong-hook injection moves five — and the one it conspicuously does
NOT move is the **subsonic equilibrium**. The reason is structural: `_instant_tail` solves the
choked `(★)` geometry first, dispatches on the nozzle, and on the subsonic branch **re-solves
`pi_t` from nozzle continuity**, discarding the choked-star answer entirely. So on a subsonic cell
the table could be wired to anything at all. § 5.13's prediction 2 registered a firing COUNT
rather than a value comparison for a different reason (slice N's unreachable hook); it turns out
to be load-bearing for this one as well. **A hook can be simultaneously load-bearing and dead,
depending on which branch the cell takes** — and the value oracle reports the second case as
agreement.

**FINDING 3 — `phi_max`'s DEAD ARMS ARE GATED DIRECTLY, AND THE GATE NEEDED A VACUITY GUARD.**
Probe 1 measured that a rung-34 march reaches only two of the three arithmetic arms and never a
nonzero `vsv`. Six shapes are therefore driven straight at the function — flat, two quadratics,
the linear arm, and two swirled ones — and the gate ends by asserting the six values are
**DISTINCT**, because six calls that happen to return one number would pass the bit comparison
six times while testing one branch. Rung 55's `test_p6_verdicts_survive_the_work_split` precedent.

##### STEP 2 — SHIPPED. **THE THREE RUNG SUITES, AND A REGISTERED PREDICTION CONFIRMED TIGHTER
THAN PYTHON ASSERTS**

`rung34.rs` (11 tests), `rung35.rs` (5), `rung36.rs` (9) — **all 19 of the Python gates port,
none deferred**, plus three added by the port. Everything green on the first run; the four slice-P
targets together run in **1.4 s**.

**THE ROSTER, and it is symmetric in both directions.** Rung 34's eight gates, rung 35's four and
rung 36's seven all land. Three tests exist here that do not exist in Python and are listed in
each file's roster so a name diff reads correctly:
`the_hook_table_fires_and_rung31s_does_not` (the firing count),
`phi_max_is_read_at_both_flow_search_caps` (the cap is READ, not just correct), and
`the_illinois_call_and_evaluation_counts_reproduce_pypys` (step 1's two blind spots).

**PREDICTION 7 HELD, AND THE PORT ASSERTS IT 100 000× TIGHTER THAN PYTHON DOES.** Rung 34's gate 6
says `solve_n(m, tau_c_forward(n, m)) == n` and Python's bar is `1e-9`. § 5.13 registered it as
EXACT rather than tight, on the ground that slice J ported the inverse of this very equation.
Measured worst residual over 4 shapes × 5 speeds × 4 flows: **inside 1e-14**. The Python bar is
kept as the first clause and the tight one added beside it as a SECOND assertion, so a future
loosening is visible in the diff rather than absorbed into a widened tolerance.

**PREDICTION 9 DISCHARGED — `rung41.rs`'s DEFERRAL LANDS IN `rung36.rs`, NOT WHERE SLICE L FILED
IT.** Slice L's ledger called `test_rung36_verdict_survives_but_its_mechanism_is_corrected` a
`TwoSpoolTransient` gate and booked it to phase 6 on that basis; it is a SINGLE-spool gate (rungs
34/36), which is why it discharges here rather than waiting for slice R. Slice L's verdict held
while its noun was wrong — recorded at the time, and now closed. The ported gate asserts four
things Python asserts in one: rung 36's verdict survives the decomposition, BOTH channels move,
the speed-line channel thins monotonically all the way down, and at the reference throttle every
channel collapses onto the shipped margin — that last clause being the anchor without which the
decomposition would be a parallel arithmetic.

**PREDICTION 10 HELD, MEASURED — AND THE NUMBER WAS RE-TAKEN AFTER STEP 3.** Python marks one of
these 19 `slow` (rung 36's). The three Rust suites run in **1.3 s together**. That measurement
predated `spool_oracle.rs`, which did not then exist and which runs **3.9 s** on its own; the
honest figure for the slice is **all five targets in 5.3 s**, against a crate whose slowest single
target is 246 s. Slice M's rule applied unchanged: port the gate, drop the marker, re-introduce
`#[ignore]` only against a measured cost. Nothing here earns one — *stated against the number that
includes the oracle, not the one that predates it.*

**ONE GATE COULD NOT BE PORTED AS WRITTEN, AND THE FIRST ATTEMPT WAS VACUOUS.** Rung 35's gate 2
re-runs ONE engine object before and after building a transient off it. That has content in Python
because the constructor takes *that* engine and the objects are SHARED (`self.gas` **is**
`_fs_engine.gas`, and an equilibrium gas carries a frozen station-4 mixture a constructor could
reset). The Rust constructor CONSUMES its engine, so the aliasing half is the compiler's. The
first draft transcribed the shape anyway — run, build a transient off a *second* engine, run again
— which **cannot fail**: nothing connects the two and the engine is immutable. A gate that cannot
fail is worse than an absent one, because a reader diffing the suites sees a gate that looks the
same and is not. Repaired to assert what remains testable — a design run is BIT-reproducible
across an intervening transient construction *and use* — with the difference written at the gate.

##### STEP 3 — SHIPPED. **`spool_oracle.rs`: 7 300 KEYS, AND THE CPython ARM IS THE SHARPEST SINCE
SLICE G**

`oracle/dump_spool.py` + `tests/spool_oracle.rs`, seven sections over 7 map shapes × 6 throttles,
15 marches, 9 surge schedules, 2 channel decompositions and every `phi_max` arm driven directly.
**7 300 / 7 300 bit-exact against PyPy** — prediction 1 held. Six gates.

**STEP 2's WRITE-UP CALLED THE SLICE SHIPPED AND IT WAS AT STEP 2 OF FOUR.** Five of the ten
predictions were unsettled at that point and the prose did not say so: 1 and 4 needed a dump that
did not exist, 5's only clause was true by construction, 6 was ungated, and **3 had never been
run at all** — the injection was measured against the SMOKE, not against the rung suites, which is
the comparison the whole discriminator argument is about. Recorded here rather than quietly fixed,
because the failure mode is the plan's own: *a step's write-up is a claim about what is gated, and
it is only as good as an enumeration over the registered list.*

**PREDICTION 3, FINALLY RUN, AND IT IS SHARPER THAN REGISTERED.** Rung 34's Illinois turbine solve
was swapped for rung 31's bisection — a ~9e-12 disagreement — and everything re-run:

| | gates failing |
|---|---|
| the **19 ported Python gates** across `rung34.rs` + `rung35.rs` + `rung36.rs` | **0** |
| `slice_p_smoke.rs`'s bit comparisons | 6 of 9 |
| the two gates the PORT added | 2 |

Registered as *"> 0 oracle keys and 0 rung-suite gates"*; measured as **zero of nineteen**. Every
gate those three suites ship is written at `1e-6`–`1e-9`, where their physics lives, and a defect
four orders below that is invisible to all of them. The argument for an oracle is now a
measurement in this port rather than a principle inherited from it.

**PREDICTION 2's SECOND CLAUSE WAS IN THE GATE'S NAME AND NOT IN ITS BODY.**
`the_hook_table_fires_and_rung31s_does_not` could only assert the first half: `spool.rs` counted
rung 34's function and **nothing counted rung 31's**, so a table wired to BOTH would have passed.
`matcher::take_r31_calls` closes it, and the oracle asserts it over the whole grid as well.
Additive instrumentation on the precedent `OffDesignMatcher::tau_calls` and `map::psi_calls`
already set — no signature and no phase-2-to-5 gate changes.

**PREDICTIONS 4, 5 AND 6 ARE NOW GATED BY THE DUMP RATHER THAN RESTATED FROM THE PROBE.** § 5.13's
numbers (185 fallbacks, 2 escalations, 16 508 `phi_max` calls) came off `probe_p.py`'s grid, which
is **not** this one — slice N step 4's lesson taken as an instruction, so the census is emitted per
section and compared. Measured here: the LINEAR `phi_max` arm fires **0** times across all five
marching sections against 23 000-odd live calls, and **6** times in the direct section, which is
what makes the zeros evidence rather than silence. Both arms of the `M9 > 0.985` guard fire on the
marches (182 raises, 2 escalations). And the spool-down's length differs by map shape with both
edges present — one shape full, two short — so the `break` that makes LENGTH an output is not dead.

**A FOURTH INSTRUMENT DEFECT, FOUND BY THE GATE IT WAS BUILT FOR.** The first dump came back
**7 299 of 7 300**, the single miss being `census/equilibria/illinois_evals`: Rust 16 761, Python
16 752. Every value agreed; only the count moved. Cause: the Python wrapper tallied into the census
*after* `_illinois` returned, and an Illinois whose residual raises mid-search — which is CONTROL
FLOW here, not an error — propagates out and skips that line, discarding the call's evaluations
entirely. Rust's counter increments in the loop and keeps them. The instrument was aligned to the
port (the more informative of the two) rather than the reverse. Fourth instance of a measuring pass
finding the defect in the INSTRUMENT.

**THE CPython ARM: 1 652 of 7 300 identical — 22.6 %, and § 5.12's warning is vindicated.** The
pre-flight's probe got **100 %** on this same instrument and recorded it as NOT coverage, because
it ran a CPG gas whose properties are closed-form. On the thermally-perfect gas the same arm is the
sharpest detector since slice G's 8.0 %: **83.8 % of continuous keys move between two correct
implementations of one language.** The tiering is the finding —

* **530 discrete OUTPUT keys — branch labels, trajectory lengths, schedule row counts, surge
  verdicts, cell tallies — and ZERO differ.** The physics' discrete answers are interpreter-
  invariant. That is the **opposite** of slice N, where 520 discrete argmin keys flipped because
  the compared quantities had collapsed to 1–2 ULP.
* **22 of 60 census keys differ** — solver iteration counts only. How many passes it took moves;
  what it converged to does not.
* **Two structural-zero families, named so nobody re-derives them.** `Phi`/`p_net_spec` at an
  equilibrium ARE the residual driven to zero (483 keys, worst *absolute* gap 1.06e-5). And
  `E_surge` on a PEAKED map is `max(0, …)` sitting AT its clamp — rung 34's gate 4 says
  `flow_dominated` run forward gives a NEGATIVE excursion, so the accumulator never leaves 0.0:
  PyPy returns **exactly 0.0**, CPython **1.61e-11**, a 100 % relative difference out of a 1e-11
  drift. **A `max`-accumulated quantity at its floor converts arbitrary drift into total relative
  disagreement**, which is why the arm is tiered on KIND and not on one tolerance.

**THE DUMP LIVES IN-TREE.** `slice_p_smoke.rs`'s 132 goldens were produced by a script under
`M:\claud_projects\temp`, unlike every prior slice's `rust/oracle/dump_*.py`. That is a durability
defect independent of the oracle question — goldens whose producer is outside the repository
cannot be regenerated from a checkout. `oracle/dump_spool.py` carries the regenerate header.

**ONE DECISION BOOKED TO SLICE R's PRE-REGISTRATION RATHER THAN LEFT TO ARRIVE MID-PORT.**
`spool.rs::march` de-duplicates Python's two RK4 bodies (`integrate` 1588, `integrate_fuel` 1769)
into one with the instant passed in. That is defensible **here** and the reason is written at the
function: the two Python bodies are character-identical apart from the closure they call and the
`Tt4` they record, and neither carries a comment arguing for the separation — the *do not factor a
deliberate duplication away* rule targets a duplication the SOURCE argues for, and this is not one.

**It stops being free at slice R.** `TwoSpoolTransient.integrate` (3753) marches TWO states with
`rho` on the LP equation — a different signature, not the same body; `TwoSpoolFuelTransient.
integrate_fuel` (4834) is the min-select body and § 5.12 booked it as **overridden by 11 phase-7
classes**, so it must be a hook; and `_integrate_fuel_lagged`/`_integrate_fuel_asym` add a third
state. There will be pressure to widen `march` into one shared marcher across all four. **Fusing
them would put a hook's dispatch inside a body slice P's gates already cover** — § 5.3's
`_solve_turbine` argument one level up, and the exact failure the phase gating exists to prevent.
Registered as a decision slice R must make explicitly, with its reason, before any code.

**THREE RECORDS DISAGREED WITH EACH OTHER AFTER STEP 2, AND ALL THREE ARE FIXED.**
`rung41.rs`'s roster still read `ported == 10` and printed `DEFERRED -> phase 6` for the gate
`rung36.rs` had already discharged (now 11, with the last one booked explicitly to slice R);
`rung53.rs`'s ledger item 1 still carried the wrong `phi_max` assertion while step 1's write-up
said *"both notes corrected"*; and the step-2 text above read as slice-complete. *A stale
cross-reference costs more than an uncounted one* — the project's own rule, applied to the port's
own paperwork.

---

### 5.14 SLICE Q (rung 37, `CombustorTransient`) — PRE-REGISTERED, four probes MEASURED first

Rung 37 splits rung 34's one bundled concession ("no combustor volume-filling, no heat soak … they
do not change the `r` framing") into two clocks that fall on opposite sides of `tau_spool`. A
**PLENUM** makes `pt4` a state and CONFIRMS the concession; a **HEAT-SOAK** metal temperature `Tm`
is a genuine second state and CORRECTS it. `CombustorTransient` (447 lines, `engine.py:2012–2396`,
14 methods) subclasses `SpoolTransient`; one Python suite, **7 tests**, one of them `slow`.

`M:\claud_projects\temp\rust-phase6\probe_q.py`, `probe_q2.py`, `probe_q3.py` — four arms over
three surge shapes × the gate grids.

**PROBE 1 — `equilibrium_soak` CONTAINS THE SAME FIXED-POINT LOOP TWICE AND THE TWO COPIES DO NOT
AGREE.** `resid`'s inner loop (2296–2305) sets `Tm = inst["Tt4_burner"]` on the pass that
converges, *before* breaking; the outer loop (2307–2312) breaks **without** that line. So the
residual the root find sees is evaluated one fixed-point update ahead of the instant the method
returns. Measured over 6 cells, unifying them (the outer loop given the inner's terminal update):

| quantity | worst move |
|---|---|
| `nu` | **0.000e+00 — bit-identical** |
| `pi_c` | 3.098e-12 |
| `Tt4` | 9.767e-12 |
| `\|dTm\|` at the terminal pass | 1.190e-07 K |

`nu` cannot move because the outer loop sits **downstream of the root find**. Gate 4's bar is
`1e-9` relative, three orders above the largest of these. Worst outer pass count is **8** against
the `range(60)` cap, and the initial guess is a bare literal `Tm = 1500.0` at both sites — not
`Tt4_lo`, not the design `Tt4`. *This is a duplication the source does not argue for and is not
even identical; it is ported as two written-out loops with the difference stated at the site.*

**PROBE 2 — THE ILLINOIS EXHAUSTION ARM IS THE DOMINANT PATH HERE, AND SLICE P SHIPPED IT AS
UNREACHABLE.** `_plenum_pt4_at` (2172) passes `tol=self._N_TOL=1e-12` as an **absolute** bracket
width on a `pt4` of order `1e5` Pa — seventeen decades below the values it brackets. Attribution
by call site, `probe_q2.py`:

| site | calls | residual evals | **exhausted `maxit`** |
|---|---|---|---|
| `_plenum_pt4_at`'s `bal` | 109 | 10 359 | **103** |
| `_compressor_from_backpressure`'s invert | 10 918 | 93 939 | 0 |
| `_solve_turbine` (rung 34's hook) | 11 078 | 92 866 | 0 |
| every other site | — | — | 0 |

**94.5 % of that site's calls run out of iterations and return `b`.** Slice P step 1 listed
*"exhausting `maxit` returns `a` instead of `b`"* as one of two injected defects invisible to 132
bit-exact values, and closed the blind spot with `counters::illinois_exhausted`. Slice Q is where
that counter acquires a population. Injecting `return a` and re-running: `equilibrium_plenum`'s
`nu` moves **3.523e-12**, `pi_c`/`pt4` **6.292e-12** — and `plenum_frozen_peak`'s peak moves
**0.000e+00**. So the arm goes from *unreachable and uncounted* to *dominant and still four orders
below every gate the suite ships*. The count is not a substitute for a value gate here; it is the
only thing that separates the two returns.

**PROBE 3 — THE REACHABILITY CENSUS, AND WHAT THE PLENUM PATH STRUCTURALLY CANNOT REACH.**

- **NONE of rung 37's three marches carries a `try`.** `plenum_frozen_peak` (2213),
  `soak_excursion` (2347) and `adiabatic_excursion` (2382) run `n_steps + 1` unconditionally, so a
  failing RK stage **propagates** — the exact opposite of `SpoolTransient.march`, whose `except
  AssertionError: break` makes trajectory LENGTH an output (§ 5.13 probe 5). Measured over 30
  marches: **0 stage failures**. The difference is therefore **LATENT**, which is precisely why it
  must not be fused: routing these through `spool.rs::march` — private, in-crate, and right there
  — would silently convert a raise into a truncation that no value gate could ever see.
- **The soak closure's bracket assert is live from ONE of its two callers.**
  `_close_compressor_fuel_soak` fires **208 of 1 373** times from `equilibrium_soak`'s march-in
  and **0 of 11 544** from `soak_excursion`'s RK stages. Same function, opposite fallibility —
  slice L step 1's *per call site, not per function* rule with both arms in one file for the
  first time.
- **`_plenum_pt4_at`'s bracket assert fires 116 of 225**; its `m_min < m_max` floor assert **0**;
  `_compressor_from_backpressure`'s bracket assert **0 of 15 136**. Two dead, one hot.
- **The plenum instant reaches the HOOK on every call and the nozzle DISPATCH on none** —
  2 272 `_plenum_state` calls, **2 272** `_solve_turbine`, **0** `_instant_tail`, **0**
  `_turbine_subsonic`. So slice P's two rarest counters (the `M9 > 0.985` fallback and its
  escalation) are **structurally unreachable** from rung 37's plenum path, and `R34` is
  load-bearing on all of it. *`probe_q2.py`'s first version of this measurement counted
  `_instant_tail` GLOBALLY and reported 160; scoped to "while inside `_plenum_state`" it is 0.
  Fifth instance in this port of a measuring pass finding the defect in the instrument.*
- **`phi_max` reaches ONE arm.** Only `quadratic` fires (11 659 in one section); `flat5`,
  `linear` and `swirled` are all 0 — a **different** census from slice P's (5 258 `flat5`),
  because rung 37's grid is all surge shapes. Quoted with its grid, never merged with slice P's.
- **`t_accel is None` is reachable on one gate's grid and not the other's** — 4 of 24 cells at
  gate 5's `s_end=6.0`, 0 of 6 at gate 6's default `s_end=12.0`. It is an `Option`, and gate 6's
  `1e9` sentinel is Python's way of ordering a `None`.
- **`_pic_band`'s ceiling: `phi_max*n` binds 15 of 15**, the literal `2.5` never; the `_PHI_FLOOR`
  floor never sits above it. **`n_steps = int(round(10.0*r_v/ds))`** with `ds = r_v/15` evaluates
  to exactly `150.0` at every `r_v` swept — the round's tie is **measured** unreachable, and
  `round_ties_even` is spelled anyway on `march`'s precedent. **`_plenum_K == 0.0`** is reachable
  only through the both-OFF construction gates 1 and 7 build: dead in every physics path, spelled.

**PROBE 4 — `docs/rung37-spec.md`'s NUMBERS WERE TAKEN ON A GRID `test_rung37.py` NEVER RUNS, AND
THEY ALL REPRODUCE.** The spec quotes its excursion table, `split_max ≈ 22 %` and
`t_accel ≈ 5.55` vs `2.15` at `G=0.1, r_m=3`; the suite sweeps `G ∈ {0.05, 0.15} × r_m ∈ {1, 5}`.
Slice N step 4's two-grids-one-section shape, sitting in the rung's own documentation. Re-measured
on both: every spec number reproduces to its quoted precision (`t_accel` 5.549999999999988 against
2.1500000000000004; `split_max` 21.98/22.09/22.12 %; `peak − E0` between −7.3e-11 and −1.7e-8),
and `cold < hot < adiabatic` holds on all nine spec-and-test cells. **A confirmation, recorded so
the grid mismatch is not re-discovered as a defect** — the numbers are right, only their grid was
unstated.

**THE TEN PREDICTIONS, registered before any Rust is written:**

1. The oracle comes back **100 % bit-exact against PyPy** — no adaptive control, no stopping rule
   downstream of a solver slices I/J/P already proved exact.
2. **Unifying `equilibrium_soak`'s two loops fails 0 of the 7 ported Python gates and moves > 0
   oracle keys.** Slice P's prediction 3 arriving in a second slice, registered before the port
   rather than after the dump.
3. Injecting `return a` into the exhaustion arm likewise fails **0 rung-37 gates** and moves
   **> 0 oracle keys**; `illinois_exhausted` comes back **109** attributed wholly to
   `_plenum_pt4_at`, 0 elsewhere.
4. `SpoolTransient::march` is **NOT** reused. Trajectory length is a PARAMETER here, not an
   output, and the dump carries **0 stage failures** so the difference is gated by reachability.
5. `_compressor_from_backpressure`'s bracket assert and `_plenum_pt4_at`'s floor assert stay at
   **0** firings across the whole dump — gated against zero rather than left absent.
6. **No `Hooks` table** — § 5.12's arm 1, second confirmation: `CombustorTransient` has 0
   subclasses. If slice Q finds one, the pre-flight's census is wrong.
7. `_plenum_state` fires `r34_solve_turbine` **exactly once per call** and `_instant_tail` /
   `_turbine_subsonic` **zero** times, over the whole dump.
8. Gate 5 — rung 37's only `slow` marker — earns **no `#[ignore]`**, measured against the crate's
   slowest target in both directions rather than by applying slice M's rule reflexively.
9. `phi_max`'s `flat5` / `linear` / `swirled` arms stay at **0** across every marching section
   against a live `quadratic` count, and the direct section drives all four so the zeros are
   evidence rather than silence.
10. Gate 1's reduce (both OFF ⇒ rung 35) holds **bit-for-bit**, not to a tolerance — the OFF
    switches are exact dispatch, and in Rust they are `Option`s that are never read.

**THREE FUNCTIONS LOOK REUSABLE AND ARE NOT**, named here so a later reader does not undo the
duplication:

- **`pi_c_map` vs `_pic_of_m`.** `pi_c_map` (rung 36) carries a `tau_c > 1.0` guard and takes
  `phi`; `_pic_of_m` (rung 37) has no guard and takes `m`. Reusing it adds an error path Python
  does not have.
- **`try_instant_tail`'s power block vs `_plenum_state`'s.** The tail computes `pt_spec =
  eta_m*(1+f)*Δh_t` **per unit air**; the plenum computes `Pt = eta_m*mdot_ngv*Δh_t` on the
  **absolute** flows, with `Pc` on `mdot_c`. They are equal only when `mdot_ngv == mdot_air*(1+f)`
  — which is exactly the coupling the plenum exists to break. A different formula, not a refactor.
- **`f_cap = 0.05`.** A local literal in `_plenum_pt4_at`, in `_close_compressor_fuel_soak`, and
  already in `try_close_compressor_fuel`. Three copies in Python, three in Rust.

**PORT DECISIONS, REGISTERED** (slice O's precedent). `theta0` is a string with `else`-hot
semantics — any value that is not `"cold"` is hot — becoming an enum, which **narrows** the
domain; `adiabatic_excursion` returns `theta0="adiabatic"`, so the enum's third variant is
unreachable from `soak_excursion` and is a label, not an input. `_plenum_state` returns neither an
`Instant` nor a `CompState` (no thrust, no `m9`, and a `dpt4_ds` neither has), so it gets its own
struct. `_close_compressor_fuel_soak` returns a `CompState` whose `tt4` holds `Tt4_turb`, with
`tt4_b` carried beside it — the tail is handed `Tt4_turb` as its `tt4` argument, and only
`dTm/ds` and the burner balance read `Tt4_burner`.

**MODULE DECISION AND SIZING.** A new `src/combustor.rs` — the rung has its own two effects, its
own three marches and its own structs, and `spool.rs` is already 1 383 lines. `CombustorTransient`
is **composition over `SpoolTransient`** (§ 5.12's census: nothing downstream overrides anything
it calls on `self`), reaching the parent's public surface through a `pub inner`. ~450 source lines
and 180 test lines over 7 tests against slice P's 720/19 — roughly a third of slice P.

**THE FOUR STEPS.** 1: `src/combustor.rs` + `tests/slice_q_smoke.rs`, with the injections above
measured rather than reasoned. 2: `tests/rung37.rs`, the 7 Python gates. 3:
`oracle/dump_combustor.py` + `tests/combustor_oracle.rs`, PyPy and CPython arms. 4: docs.

##### STEP 1 — SHIPPED. **THE ARM SLICE P COULD NOT REACH IS WORTH 97 OF 517 VALUES HERE**

`src/combustor.rs` (the port) + `oracle/dump_slice_q_smoke.py` + `tests/slice_q_smoke.rs`.
**517 values bit-exact against PyPy on the first run**, over eleven sections chosen to touch every
path once: the speed line read as `pi_c(m)` at both band endpoints, the back-pressure invert, the
decoupled instant at three pressures around the steady one, the exhausting root find, the plenum
equilibrium beside rung 35's, a 151-step plenum march on two shapes × two fill clocks, the soak
closure at three metal temperatures **including one above the burner exit**, the soak instant, the
two-loop soak equilibrium, a two-state march reaching BOTH arms of `t_accel`, and the both-OFF
reduce. Predictions 6 (no `Hooks` table) and 10 (the reduce is BIT-identical, asserted in the test
rather than only compared to Python) held.

**FIVE DEFECTS INJECTED INTO THE SHIPPED CODE, AND ALL FIVE ARE VISIBLE** — slice N step 2's rule
that a dump passing first try means no gate has been watched to fail:

| injected defect | smoke values failing (of 517) | sections |
|---|---|---|
| the Illinois exhaustion arm returns `a` instead of `b` | **97** | C D E F + census |
| the plenum power block copied from `_instant_tail` (per unit AIR) | **60** | C E + census |
| `equilibrium_soak`'s two loops UNIFIED | **29** | I + census |
| the back-pressure invert returns the RECOMPUTED `pi_c` | **16** | B C E F |
| `dTm/ds` reads `Tt4_turb` instead of `Tt4_burner` | **9** | H I J + census |

**THE TOP ROW IS THE SLICE'S LEADING RESULT AND IT INVERTS SLICE P's.** That is the identical
injection slice P measured as **0 of 8** — invisible to 132 bit-exact values, because the arm was
never reached. One rung later the same three lines decide 97 values. *A dead arm is a property of
the GRID, not of the code*: slice P closed the blind spot with `counters::illinois_exhausted`
rather than deleting the claim, and slice Q is what that counter was for. Note also which sections
it does NOT touch — H, I and J, the heat-soak half, because the exhaustion is confined to
`_plenum_pt4_at`.

**AND THE INJECTION HARNESS'S FIRST SECTION LIST WAS WRONG.** It split the panic message at
`"values differ:\n"` and took everything after, which on this console includes cargo's own
interleaved output, so it reported `Compiling` and `Finished` as sections. Re-parsed to match only
lines of the shape `key: rust <x> != py <y>`. The COUNTS were never affected — they come from the
message header — but the section lists in the table above are the re-parsed ones. *Sixth instance
in this port of a measuring pass finding the defect in the instrument, and the second in this
slice.*

##### STEP 2 — SHIPPED. **THE RUNG'S OWN HEADLINE ARITHMETIC IS UNOBSERVABLE TO EVERY GATE IT SHIPS**

`tests/rung37.rs` — **all 7 Python gates port, none deferred**, plus 3 the port adds. Ten targets
green on the first run in **6.6 s**.

**PREDICTIONS 2 AND 3 WERE RUN AGAINST THE GATES AT THIS STEP, NOT DEFERRED TO THE DUMP.** Slice P
recorded at its step 3 that its own prediction 3 had been measured against the SMOKE and never
against the rung suites — which is the comparison the entire discriminator argument is about. Run
here instead, and a third injection was added to it:

| injected defect | smoke values (of 517) | **Python gates (of 7)** | port-added gates (of 3) |
|---|---|---|---|
| the Illinois exhaustion arm returns `a` | 97 | **0** | **0** |
| the plenum power block copied from `_instant_tail` | 60 | **0** | **0** |
| `equilibrium_soak`'s two loops UNIFIED | 29 | **0** | **0** |

Predictions 2 and 3 registered *"0 gates, > 0 oracle keys"* and both hold. **The third row is the
finding, and it is sharper than either.**

**`_plenum_state`'s `Phi` IS READ AT EXACTLY ONE SITE, AND THERE THE WRONG FORMULA AND THE RIGHT
ONE ARE IDENTICAL.** Rung 37's docstring calls this the honest part — *"the shaft power is
computed HONESTLY with the two DISTINCT mass flows: the turbine passes `mdot_NGV`, the compressor
`mdot_c` — unlike rung 34/35 where they are equal by the rigid coupling."* `Phi` is read only by
`equilibrium_plenum`'s residual (`engine.py:2185`, `combustor.rs:619`); `plenum_frozen_peak` reads
`pi_c`, `mdot_c`, `mdot_ngv` and `dpt4_ds` and never `Phi` at all. And the difference between the
two formulas is

```text
    eta_m * (h_t4 - h_t5) * (mdot_ngv - mdot_c*(1 + f)) / (P_ref * nu)
```

which vanishes when `mdot_c + mdot_fuel = mdot_ngv` — the plenum's own steady condition, and the
very root that residual is driven to. So the one place the honest power block is read is the place
where it is nearly indistinguishable from the dishonest one. **The plenum's shaft ODE is never
marched**: `pt4` is the only state the plenum integrates, on a spool frozen at `nu0`.

**"Cannot differ" was the first wording and it is wrong by a residual — corrected at step 3 by
measuring it.** The balance closes to ~1e-12 relative, not to zero, so the injection *does* move
the plenum equilibrium: **5.4e-12 in `nu`, 9.1e-12 in `pi_c`**, three orders below gate 2's `1e-9`
bar. That is a stronger statement than the exactness claim and it is the true one — the crate's own
*an "exactly" claim survives a copied instruction sequence and dies on a second derivation* rule,
turned on this plan's own prose.

This is not a defect in rung 37 — the physics is right and the claim is true — but the claim is
carried by NO gate the rung ships, and only a bit-exact dump witnesses it (60 smoke keys; 104
section-C and 30 section-B oracle keys). *A quantity computed correctly, read once, and read only
where its correctness is three orders below the bar.* Registered so slice R does not inherit the
sentence as though it were tested.

**PREDICTION 8 HELD, MEASURED IN BOTH DIRECTIONS.** Python marks gate 5 `slow`. In Rust it runs
alone in **5.95 s** — 90 % of the ten targets' 6.6 s, and **2.4 %** of the crate's slowest single
target (246 s). So it is simultaneously the dominant cost of its own file and nowhere near earning
an `#[ignore]`. Both numbers are quoted because § 5.14 registered that phase 6's marches were the
first place a marker might genuinely be earned, and *"it is small"* against *"it is 90 % of its own
file"* are different claims.

**THE THREE ADDED GATES ARE ALL COUNTS, AND ONE FAILED FIRST WITH ITS OWN ARITHMETIC WRONG.**
`the_marches_run_to_length_with_no_bracket_failure` predicted the plenum march at
`151 + 3*150 = 601` `_plenum_state` calls and measured **703**. The missing 102 is the single
`plenum_pt4_at` that sets the start pressure — 2 bracket endpoints plus `ILLINOIS_MAXIT = 100`
residual evaluations, *because that is the site whose absolute 1e-12 tolerance never converges*.
The exhaustion arm turned up as an exact integer inside a gate not written to look for it, and the
gate now asserts the decomposition rather than the total. The other two are prediction 7 (the
plenum instant reaches the hook once per call and the nozzle never) and prediction 3's count half,
written as a CONTRAST — the back-pressure invert driven hard first and exhausting nothing, then
the pressure solve exhausting — because a bare positive count would also pass a port that
exhausted everywhere.

##### STEP 3 — SHIPPED. **THREE OF THE ORACLE'S OWN GATES FAILED, AND ALL THREE WERE THE GATE'S FAULT**

`oracle/dump_combustor.py` + `tests/combustor_oracle.rs`, nine sections over the gates' own grid.
**2 066 / 2 066 bit-exact against PyPy on the first run** — prediction 1 held. Eight gates.

**PREDICTIONS 2 AND 3's SECOND CLAUSE, FINALLY RUN.** Step 2 measured the gate half; the oracle
half had never been run against a dump because there was none. Completed here:

| injected defect | smoke (of 517) | Python gates (of 7) | added (of 3) | **oracle (of 2 066)** |
|---|---|---|---|---|
| the Illinois exhaustion arm returns `a` | 97 | 0 | 0 | **456** |
| the plenum power block copied from `_instant_tail` | 60 | 0 | 0 | **142** |
| `equilibrium_soak`'s two loops UNIFIED | 29 | 0 | 0 | **133** |

Registered as *"0 gates, > 0 oracle keys"*; measured at **zero of ten against 133–456 of 2 066**.
Slice P's argument for an oracle is now a measurement in a second slice rather than a precedent
inherited from the first. (The counts are exact — they come from the failure header. The *section*
attribution is not enumerated: the assertion prints only its first twelve keys.)

**THE THREE GATE FAILURES, EACH A DIFFERENT WAY OF MEASURING THE WRONG SET.** None was a port
defect; the 2 066 values were already right.

1. **A precondition listed a section that root-finds NOTHING.** The exhaustion contrast asserts
   *"these sections root-find hard and exhaust nothing"* over `A, E, G, H`. Section A is pure
   arithmetic — `pic_band` and `pic_of_m`, no bracket anywhere — and makes **0** Illinois calls, so
   "A exhausts nothing" is the emptiest sentence in the file. The `illinois_calls > 50` clause is
   what caught it, which is that clause's entire purpose: *an exhaustion count of zero means
   nothing without a call count beside it.*
2. **A RATE whose denominator included the population that never reached the thing measured.**
   The bar asked that half of `_plenum_pt4_at`'s calls exhaust; measured 103 of **225** = 46 %,
   and failed. But **116 of those 225 fail the bracket test and never reach the Illinois** —
   `equilibrium_plenum`'s outer search probes speeds off the operable map. Against the 109 that do
   reach it the rate is 103, the **94.5 %** § 5.14 probe 2 measured, reproduced exactly. Two
   different numbers about two different sets, and only one of them is what the sentence says.
3. **A per-SECTION census answering a per-CALL-PATH question.** The gate asserted
   `subsonic_raises == 0` in the plenum sections; B tallies **1** and D **4**. Not from any plenum
   instant — from the rung-35 calls each section makes to *reach* one (`fuel_for_tt4`,
   `equilibrium_fuel`), which do go through `_instant_tail`. **It is the same conflation
   `probe_q2.py`'s global-vs-scoped instrument made at the probe stage, arriving this time in a
   GATE** — where, had the tallies happened to be zero, it would have shipped as a true-looking
   false claim. The scoped form lives in `rung37.rs`, which brackets the counters around
   `try_plenum_state` alone and gets zero; the oracle now asserts only what a section tally can
   support.

That is the third, fourth and fifth instrument defect in this slice, and the more expensive
place for it: *a probe that measures nothing wastes a run; a GATE that measures nothing ships.*

**PREDICTION 9 WAS CARRIED BY NOTHING, AND AN ENUMERATION FOUND IT BEFORE THE WRITE-UP DID.** The
first version of the dump emitted no `phi_max` arm tallies at all, so the registered claim — that
rung 37 reaches only the quadratic arm — had no gate. Slice P's step 3 named exactly this failure
mode; the fix is that the enumeration over the ten predictions was run **before** writing this
section rather than after. Now gated: `flat5`, `linear` and `swirled` at **0** in all nine
sections against a live quadratic count, and the census is quoted as **rung 37's own** — slice P's
had `flat5` at 5 258 because its grid included flat maps, and the two must never be merged. That
the three dead arms are reachable at all stays `spool_oracle.rs`'s gate; re-driving them here
would gate one fact twice.

**ONE PREDICTION WAS SHARPENED BY THE DUMP.** § 5.14 registered the flow ceiling's literal `2.5`
arm as never binding on the operating grid (probe 3: 15 of 15 took the map arm). Section A drives
`nu` to 1.3 to put the other side on the dump — and only **1 of 12** cells takes the literal arm.
`surge_flow`'s `phi_max` is large enough to clear 2.5 there; `surge_pressure`'s and
`surge_tilted`'s are not, a third of the way past design speed. The arm is **shape-dependent**, and
the gate names the cell rather than the count.

**THE CPython ARM: 452 of 2 066 identical — 21.9 %, and it reproduces slice P's tiering exactly.**
§ 5.12's pre-flight got 100 % on a CPG gas and recorded it as NOT coverage; slice P measured 22.6 %
on the thermally-perfect gas, and this is the second independent confirmation that the arm is a
sharp instrument rather than a formality.

* **83 discrete OUTPUT keys — branch labels, the `cold < hot < adiabatic` verdicts, the `t_accel`
  presence flags, the reduce flags, which ceiling arm binds — and ZERO differ.** What the physics
  DECIDES is interpreter-invariant.
* **1 576 of 1 812 continuous keys differ — 87.0 %** between two correct implementations of one
  language.
* **38 of 171 census keys differ**, all of them solver iteration counts. How many passes it took
  moves; what it converged to does not.

**AND THE WHOLE CRATE IS GREEN** — every target, 0 failed, with the two slice-Q targets adding
10.1 s (oracle) and 6.6 s (suite) beside a slowest single target of 269 s.

##### STEP 4 — SHIPPED, docs-only. **SLICE Q COMPLETE; THE TEN PREDICTIONS, ENUMERATED**

| # | registered | outcome |
|---|---|---|
| 1 | oracle 100 % bit-exact vs PyPy | **HELD** — 2 066/2 066, first run (517/517 at the smoke) |
| 2 | unifying the soak loops: 0 gates, > 0 oracle keys | **HELD, sharper** — 0 of 10, **133** keys |
| 3 | exhaustion returns `a`: 0 gates, > 0 oracle keys | **HELD, sharper** — 0 of 10, **456** keys. Its COUNT clause registered **109**; the dump gives **115** (103 + 6 + 6, sections C/B/D). 109 was probe 2's grid — slice N step 4's rule applied to this section's own registration |
| 4 | `march` not reused; 0 stage failures | **HELD — but its FIRST gate was VACUOUS.** See below |
| 5 | the two dead bracket asserts stay 0 | **HELD on every gate grid** — with live siblings in the same dump; one of the two is reachable off-grid, see below |
| 6 | no `Hooks` table | **HELD** — § 5.12's census confirmed a second time |
| 7 | the plenum instant: hook once per call, nozzle never | **HELD** — and the oracle's per-section form of it FAILED first; see step 3 |
| 8 | gate 5 earns no `#[ignore]` | **HELD** — 5.95 s alone: 90 % of its own file, 2.4 % of the crate's slowest target |
| 9 | `phi_max`: only the quadratic arm | **HELD — but it was UNGATED until an enumeration found it** |
| 10 | the both-OFF reduce is BIT-for-bit | **HELD** — asserted as bits, in the suite and the dump |

**PREDICTION 4's FIRST GATE MEASURED NOTHING, AND THE ADVISOR'S CHALLENGE IS WHAT FOUND IT.** The
prediction forbids routing rung 37's marches through `spool.rs::march`, whose `break`-on-`Err`
would convert a raise into a truncation. It was gated with evaluation COUNTS — and on a grid where
no stage fails, a truncating marcher produces the identical trajectory, the identical values *and*
the identical counts. A sixth injection (`soak_excursion` rewritten to break on `Err`) failed
**0 of 19 gates across all three slice-Q targets**. The prediction was carried by nothing, exactly
as prediction 9 had been, and the step-4 table above would have said "HELD" over an empty gate.

**The repair is a FORCING case, not a looser bar, and it corrects a second claim on the way.**
Coarsen the RK step until a stage genuinely leaves the valid region and the two implementations
become distinguishable in the only way they ever can be: `plenum_frozen_peak` at `ds_frac = 2.0`
overshoots `pt4` out of the `pic_band`, and `soak_excursion` at `ds = 2.0` drives the metal
closure off its bracket. Both RAISE in Python and PANIC in Rust; a fused marcher would return a
shorter trajectory instead. Two `#[should_panic]` gates, and re-applying the injection now fails
**exactly one** — the one written for it.

The second claim: `combustor.rs` had recorded the back-pressure invert's bracket as *"DEAD on
every grid measured"*. It is dead on every grid a GATE reaches and reachable on purpose at
`ds_frac = 2.0` — **the slice's own headline, one level down**. The comment is corrected, and the
live arm now sits on the MARCH side of the *fallibility is per call site* claim rather than only
on `equilibrium_soak`'s march-in.

**THE SLICE'S ONE-LINE RESULT.** *A dead arm is a property of the grid, not of the code.* Slice P
measured `try_illinois`'s exhaustion arm at ZERO firings, established that no value gate could see
which endpoint it returned, and closed the blind spot with a counter rather than deleting the
claim. One rung later the same three lines are the path **94.5 %** of `_plenum_pt4_at`'s calls
take — because that site passes `_N_TOL = 1e-12` as an ABSOLUTE bracket width on a `pt4` of order
`1e5` Pa — and the `a`-vs-`b` return is worth 456 of 2 066 oracle keys while remaining invisible to
all ten gates. The counter slice P could only justify on principle is what made slice Q's leading
measurement possible.

**FIVE INSTRUMENT DEFECTS IN ONE SLICE, AND THE LAST THREE WERE GATES.** `probe_q2.py`'s
`_instant_tail` counter was global where the sentence said scoped; the injection harness's section
parser swallowed cargo's own output; and then three oracle gates failed for three distinct
wrong-set reasons (a precondition naming a section that root-finds nothing, a rate whose
denominator counted calls that never reached the loop, and a per-section census asked a
per-call-path question — the *same* conflation as the first one, one stage later). *A probe that
measures nothing wastes a run; a gate that measures nothing ships.*

**NOTHING IS DEFERRED OUT OF SLICE Q, AND ONE DECISION IS STRENGTHENED FOR SLICE R.** All 7 Python
gates ported. `CombustorTransient` is a leaf — no subclasses, so nothing is owed forward. The one
thing slice R inherits is a **second, independent reason** not to fuse the marchers: § 5.13 booked
that decision on the two-spool bodies' differing signatures, and rung 37 refutes fusion from the
other side — its three marches are not `march` with a different closure, they are `march` **without
its `break`**, so routing them through it would convert a raise into a truncation no value gate
could see. Written at `combustor.rs`'s module note, where slice R's reader will hit it (slice O's
rule), not only here.

**THE SPEC'S NUMBERS WERE RE-MEASURED AND ARE RIGHT.** § 5.14 probe 4 checked
`docs/rung37-spec.md`'s excursion table, `split_max ≈ 22 %` and `t_accel ≈ 5.55` vs `2.15` — all
quoted at `G = 0.1, r_m = 3`, a grid `test_rung37.py` never runs. Every one reproduces, and
`cold < hot < adiabatic` holds on the test grid too. **A confirmation, recorded so the grid
difference is not later re-discovered as a defect** — the spec states its own grid; only the
relationship between the two grids was unstated.

### 5.15 SLICE R (rungs 40 + 44, `TwoSpoolTransient`) — PRE-REGISTERED, five probes MEASURED first

Rung 40 makes BOTH shaft speeds states under one clock ratio `rho = tau_L/tau_H`, and rung 44
marches that plant against rung 41's imposed surge line. `TwoSpoolTransient`
(`engine.py:3378–3969`, 592 lines, **17 methods**, of which rung 44 owns **three** —
`_ramp_march`, `phi_excursion`, `transient_surge_margin`) subclasses rung 39's
`TwoSpoolMapMatcher`. Two Python suites, **16 tests** (9 + 7), two of them `slow`.
*Counted, not taken from the header: § 5.12's own scope audit shipped because nobody had ever
counted, and my first reading of this class said 16 and four.*

`M:\claud_projects\temp\rust-phase6\probe_r1.py` … `probe_r5.py` — five probes over both suites'
grids. `probe_r5.py` exists because a review of the first four found **four things they had not
measured**, three of which the earlier probes had silently assumed: the march-in loop, the
Newton's damper, the `wgas` comparison the gate actually makes, and whether the cache collision
fired inside the set that priced it. All four are folded in below, at the probe each belongs to.

**PROBE 1 — THE `steady` CACHE KEY COLLIDES BETWEEN TWO DISTINCT FLOATS, AND THE COLLISION IS
WORTH NOTHING.** `_ramp_march`'s closure memoises the per-`Tt4` running-line match under
`key = round(Tt4, 3)` — a *decimal* key on a float dict, so the port cannot simply hash the bits
unless the two schemes agree. Over **31 marches / 5 141 points** on the rung-40 and rung-44 gate
grids (never the defaults — collision spacing scales with `dTt4·ds/r_ramp`, and the suites sweep
all three):

| measured | value |
|---|---|
| distinct keys | 1 021 |
| legitimate memo hits (identical `Tt4` repeated) | 4 120 |
| **collisions between DISTINCT `Tt4` floats** | **1** — `1399.9999999999984` and `1400.0` share key `1400.0` |
| smallest gap between two distinct `Tt4` in any march | **1.592e-12 K** (a collision needs `< 1e-3`) |
| **reported values moved by keying EXACTLY instead** | **0**, over 6 ramp cases × `phi_excursion` + `transient_surge_margin` |
| **and the collision FIRED inside those 6 cases** | **yes — 1**, in `r_ramp=5.0, s_end=6.0` (301 points) |

That last row is not decoration: *"0 values moved"* means nothing unless a collision occurred in
the measured set, and the first version of this probe never checked. It fires in exactly one of
the six, so the zero is a measurement of the collision's price rather than of its absence.

So the collision is real — the ramp's saturating `min(1.0, t/r_ramp)` lands one step at
`1400 − 1.6e-12` and the next at `1400.0`, and the second reads the first's cached φ — and it is
**not worth a single reported number**, because it happens at the saturated end while every
extremum is attained early in the ramp. *A bit-exact oracle over the returned dicts would
therefore be BLIND to the key scheme.* That is why the port implements the round rather than
keying on the bits, and why the dump carries the **key sequence itself** (§ predictions 4): the
equivalence relation gets its own gate instead of riding on values that cannot see it — *an
oracle cannot see a missing gate*, applied before the gate is missing rather than after.
`round(x, 3)` was checked against format-and-parse on all 11 keys that occur and on 7 adversarial
half-way ties (`0.0005`, `1399.9995`, `2.6755`, …): **0 mismatches**, so the Rust route is
validated on the population it will meet.

**PROBE 2 — THE REACHABILITY CENSUS: FOUR ARMS DEAD, ONE ARM INVERTS SLICE Q, AND ONE ROUND IS
LIVE WHERE SLICE Q'S WAS DEAD.**

- **FIVE arms measured DEAD** across 20 847 closures and 5 641 marched points: `_close`'s
  bracket assert (**0**), both of `integrate`'s `except AssertionError: break` arms (**0**
  truncations in 51 marches), the `max(0.2, ·)` speed floor (**0** points), and — added after
  the first version of this probe left it uncounted — **`_close`'s low-wall march-in loop: 0
  advances in 6 339 calls** (69 440 `g` evaluations, histogram `{0: 6339}`). The loop
  `while m < hi: try: g(m) … except AssertionError: m += 0.02` exists for an off-map bracket
  artifact that never occurs on either suite's grid. All five are ported and gated against zero
  rather than left absent. *The uncounted one was flagged in my own probe file as deferred to a
  later arm, and that arm measured something else — slice N step 2's lesson landing on my own
  instrument.*
- **The high wall's `min` has BOTH arms live**, unlike everything else here:
  `hi = min(2.5, phi_max_LP · n_L)` takes the literal `2.5` **1 221** times and the map's own
  limit **5 118**. Slice Q's comparable ceiling (`_pic_band`'s `phi_max·n` against the same
  literal `2.5`) bound **15 of 15** on one arm; here the same shape is genuinely contested, so
  it is a gated two-arm census, not a spelled-but-dead branch.
- **`illinois_exhausted` is 0 of 20 847 calls here** (190 559 residual evaluations), against
  slice Q's **103 of 109** at `_plenum_pt4_at`. Same counter, opposite population: rung 37 passed
  an ABSOLUTE `1e-12` bracket width against a `pt4` of order `1e5`, rung 40 passes the same
  literal against an `m_lp` of order 1. **Quoted with its grid and never merged with slice Q's.**
- **`int(round(s_end/ds))` is LIVE.** Of the four `(s_end, ds)` pairs the two suites use, three
  are exact and one is not: `1.2/0.05 = 23.99999999999999644729`. `round` gives 24, truncation
  gives 23 — a whole missing step. Slice Q measured this same expression's tie **unreachable** on
  rung 37's grid and spelled `round_ties_even` on principle; on rung 40's grid it is load-bearing.
  *The same expression, opposite verdicts, two slices apart — which is exactly why slice Q spelled
  it rather than simplifying it.*
- **The `lp_disabled` object never builds a two-shaft state.** `__init__` returns early before
  `super().__init__`, so only four attributes exist (`_degenerate`, `map_lp`, `map_hp`, `rho`);
  every inherited method is still *bound* and raises `AttributeError` on call — **not**
  `AssertionError`, so no caller in the ladder catches it. `map_lp` / `map_hp` / `rho` are set and
  **never read** on that path.

**PROBE 3 — `equilibrium`'s NOISE-FLOOR BRANCH IS LIVE, AND THE SHIPPED COMMENT'S CELL LIST DOES
NOT HOLD WHERE RUNG 40 READS IT.** The branch (`best[0] < 1e-8` after all 80 Newton passes) was
added by rung 43 and carries a comment asserting it is *"reached ONLY by inputs that previously
RAISED"*, naming `Tt4 = 1300/1400` as the raising pair and `1500/1450/1200` as the ones that
*"happened to squeak under"*. Re-measured on rung 40's own design and maps, 12 cells × 2 gases:

| gas | map pair | primary | **noise** | raise |
|---|---|---|---|---|
| CPG | shaped + flat | **12 of 12** | 0 | 0 |
| reacting | flow/press | 2 (`1500`, `1400`) | **4** (`1450`, `1300`, `1200`, `1100`) | 0 |
| reacting | flat | 4 | **2** (`1450`, `1300`) | 0 |

`1450` takes the noise branch on BOTH map shapes and `1200` takes it on the shaped pair, so the
comment's list is wrong here; it was written at rung 43's settings and reads as general.
**The branch is not a rescue path, it is the ordinary exit on the reacting gas** — 6 of 12 cells —
so `best` tracking is load-bearing and must be ported, not elided. The acceptance bound is not
delicate, and now that is a count rather than an assertion: the worst accepted residual is
**6.47e-11** against the `1e-8` bar, and the initial residual is `3e-2`–`3e-1`. *Slice L step 4's
shape — a claim in the SHIPPED source that does not hold where a later rung reads it.*

**And the Newton's own two discrete branches are DEAD.** § 5.12 made a headline of measuring
`der`'s `min(caps)` never contended; `equilibrium` carries a *different* `min` and a three-way
`max` — `damp = min(1.0, 0.25 / max(|dl|, |dh|, 1e-30))` — evaluated on every pass, and with an
initial residual of `3e-1` the damper is exactly where one would expect a step limiter to bind.
Measured over **102 Newton steps** on the same grids: `damp < 1.0` **0 times**, the `1e-30` floor
wins **0 times**. Both spelled, both gated against zero. *Counted because a `min` on a solver's
step is the shape § 5.12 warned about, not because anything suggested it was live.*

**PROBE 4 — THE CPython ARM IS A DETECTOR AT LAST, AND IT FLIPS A DISCRETE BRANCH.** § 5.12
dumped 9 376 keys CPython-vs-PyPy and got **100 % identical**, recording explicitly that this was
NOT coverage because the probe ran a **CPG** gas, and booking the reacting-gas measurement to
slice S. **Rung 40 gate 1 runs `Gas.reacting_equilibrium()`, so slice R can discharge it early.**
Same 12 cells, both interpreters:

| measured | CPython vs PyPy |
|---|---|
| **exit-branch classification (primary / noise)** | **5 of 12 FLIP** |
| iteration count at exit | **10 of 12 differ** (e.g. shaped `1100`: PyPy noise/80 vs CPython primary/24) |
| converged `(nu_L, nu_H)` | **12 of 12 differ**, at ~1e-11 |

The mechanism is the reacting-gas equilibrium sub-solve leaving ~1e-10 of noise in `Phi` against
an ABSOLUTE `_EQ_TOL = 1e-12`: whether a pass ever squeaks under the bar is decided below the
solver's own floor, so a last-bit difference in `exp`/`log` re-rolls it. **And the rung-40 gates
cannot see any of it** — gate 1 asserts `|Phi| < 1e-9`, three orders above the worst accepted
residual, so both interpreters pass. This is the phase's first measured detector, and it
establishes the risk to prediction 1 rather than leaving it to the dump.

**PROBE 5 — RUNGS 40 AND 44 REFERENCE TWO DIFFERENT RUNNING LINES IN THE SAME WORDS, AND THE
EXTREMUM DOES NOT NOTICE.** Both docstrings say *"referenced to the RUNNING LINE"*.
`slip_excursion` (rung 40) subtracts a **linear interpolation in `u = (Tt4−Tt4_lo)/dTt4`** between
two endpoint matches; `phi_excursion` / `transient_surge_margin` (rung 44) subtract a **match at
every instantaneous `Tt4`**. On a nonlinear steady schedule those are different objects. Measured
on the same marched trajectory, in the same variable (`slip`), so only the reference varies:

| ramp | linear-ref extremum | per-instant extremum | ratio | worst pointwise gap |
|---|---|---|---|---|
| `1100 → 1150`, `r=0.5` | −2.733947e-03 | −2.733947e-03 | 1.0000 | 1.74e-05 |
| `1000 → 1400`, `r=0.5` | −2.437231e-02 | −2.437231e-02 | 1.0000 | 1.24e-03 |
| `1000 → 1400`, `r=0.1` | −3.100231e-02 | −3.100231e-02 | 1.0000 | 1.21e-03 |

**The pointwise gap reaches 5 % of the extremum, and the extremum itself agrees to seven figures**
— because the extremum is attained early, where the steady schedule has not yet curved. So rung
40's linear reference is a bounded approximation and NOT a hidden defect, and rung 44's
construction is the general one. *Recorded as content — the docstring's own history names an early
probe that read a shape backwards from a bad reference choice.*

**THE FOUR `** 0.5` SITES THAT CAN GO NEGATIVE — Python returns a COMPLEX, Rust returns NaN.**
The class carries 14; eleven are on temperatures and pressures. The discriminating ones, with the
census that decides each (42 Jacobians × 7 shapes × 2 gases × 3 throttles):

| site | guard | measured |
|---|---|---|
| `_close`'s `g` (via `gas.pr_c`, `Tt3 < 0` off-map) | the source's own `isinstance(r, float) and r == r` | the ONE instance already patched; becomes a NaN check |
| `eigenvalues`' `disc ** 0.5` | `disc >= 0.0`, explicit | **BOTH branches live: 245 real / 7 complex** over gate 5's `rho` sweep |
| `oscillatory_band`'s `(B²−4AC) ** 0.5` | none — safe only GIVEN `a<0, d<0` | `min(B²−4AC) = 2.587e-02 > 0` |
| `damping_ratio_max`'s `(−bc/(ad)) ** 0.5` | `b*c >= 0` early return | `min(a·d) = 9.438e-01 > 0`; `a<0` **42/42**, `d<0` **42/42** |

The last two ride on a **measurement** (gate 5's sign structure), not a proof, so the derivation
`B²−4AC = 16|bc|(ad+|bc|) ≥ 0` and the signs it needs are written AT the site. A sign flip turns a
Python complex into a silent Rust NaN.

**THE TEN PREDICTIONS, registered before any Rust is written:**

1. The oracle comes back **100 % bit-exact against PyPy**, INCLUDING the reacting-gas
   `equilibrium` keys — but those are the slice's one genuine exposure (probe 4), so the dump
   carries the **exit kind and the iteration count as keys of their own**, and a divergence
   shows up as a discrete disagreement rather than a last-bit one. First slice in the port to
   pre-register *where* 100 % could fail — **and what happens if it does**, decided now rather
   than improvised at step 4: § 9's Decision 1 (Option B) is the route. The reacting-gas
   `equilibrium` keys become an individually-adjudicated fragile set with their deviation
   distribution published here, exactly as § 4's fragile rungs were, and the CPG keys stay at
   bit-equality. Phases 0–2's stricter bar is not weakened for the rest of the slice.
2. `TwoSpoolFuelTransient` (slice S) overrides **none** of `_close` / `_instant_tail` / `_powers`
   — measured statically: all four overriders (`ScheduledStatorTransient`,
   `ScheduledBleedTransient`, `LimitedBleedTransient`, `LaggedBleedTransient`) are phase 7. So
   slice R's `Hooks` table ships with **zero cells exercised inside phase 6**. If slice S needs a
   non-`R40` entry, § 5.12's census is wrong. *Slice Q's prediction 6, inverted: there the
   prediction was no table, here it is a table nothing yet uses.*
3. Because no value key can witness a table nobody overrides, the slice owes a
   **dispatch-liveness gate with the failure MANUFACTURED** — swap an entry, assert a value
   breaks — on `rung42.rs::gate_the_dispatch_is_live`'s precedent and slice Q's rule that *a gate
   that only fires on failure needs the failure manufactured*.
4. Keying the `steady` cache on the exact float moves **0 reported values** (measured), so the
   round is implemented for fidelity and the **key sequence is dumped** as its own oracle keys.
   Prediction: the key sequence differs between the two schemes in **exactly 1 position** across
   the gate grids, and every value key is identical in both.
5. `SpoolTransient::march` and `combustor.rs`'s three marches are **NOT** reused; `integrate` is
   written out. Trajectory length is an OUTPUT here (rung 34's discipline, not rung 37's), and
   the dump carries **0 truncations** so the difference is gated against zero rather than absent.
6. The four dead arms of probe 2 stay at **0** across the whole dump.
7. `illinois_exhausted` comes back **0** at rung 40's call site against slice Q's 103, and the
   two populations are reported side by side, each with its grid, never summed.
8. Replacing `int(round(s_end/ds))` with a truncation is **INVISIBLE to all 16 Python gates** —
   the one gate whose grid makes it live (rung 40 gate 7, the only user of `s_end=1.2, ds=0.05`)
   asserts a 0.2 threshold against a measured 0.398, an 2× margin — and is caught by the oracle
   through the **trajectory length AND the values**, since a march one step shorter also moves
   every extremum taken over it. **This obliges the dump to carry a section at
   `s_end=1.2, ds=0.05`**: it is the only pair of the four in use where the round is not exact,
   so an oracle built from the other three would be as blind as the gates. Registered as an
   injection to MEASURE at step 1, not to reason about.
9. Both `eigenvalues` branches are gated against their **counts** (245 real / 7 complex on gate
   5's grid), not against silence — slice N's rule that only counting catches vacuity.
10. Gate 2's reduce (`lp_disabled` ⇒ rung 34) holds **bit-for-bit** through a `Degenerate` enum
    variant, and every other method on that variant **panics**, mirroring Python's uncaught
    `AttributeError`.

**THE MARCHER-FUSION DECISION — REFUSED, WITH ITS REASON, BEFORE ANY CODE.** § 5.13 booked this
to slice R explicitly. `TwoSpoolTransient.integrate` marches a TWO-vector with `rho` dividing the
LP equation; `SpoolTransient.march` marches a scalar; `combustor.rs`'s three marches run
`n_steps + 1` unconditionally. Widening one marcher across them is **refused**, on three
independent reasons: (i) the signatures differ — a two-state right-hand side with a clock ratio on
one row is not the scalar body with a different closure; (ii) slice Q's reason — rung 37's marches
have no `break`, so routing them through a marcher that truncates on `AssertionError` converts a
raise into a silent truncation **no value gate can see**, and probe 2 measures **0** truncations
here, which makes the difference LATENT rather than absent; (iii) § 5.3's `_solve_turbine`
argument one level up — `integrate_fuel` (slice S) is a hook overridden by 11 phase-7 classes, so
a shared marcher would put a hook's dispatch inside a body slice P's gates already cover. The
three bodies stay written out.

**PORT DECISIONS, REGISTERED** (slice O's precedent).

- **Two tables, and neither hard-coded.** The transient core carries its own three-field
  `TwoSpoolTransientHooks` (`close` / `instant_tail` / `powers`, all `R40` today) AND a
  `&'static TwoSpoolHooks` for the INHERITED `match`, which `_ramp_march`, `jacobian` and
  `lead_threshold` all reach through. `R39` today; slice S and phase 7 must be able to swap
  either without surgery.
- **`rho` is a mutable field, not a constructor-only parameter** — rung 40 gates 5, 6 and 7 all
  assign `t.rho = …` on a built object, and gate 7 does it inside an 18-step bisection.
- **`oscillatory_band` / `damping_ratio_max` take `rho` as a PARAMETER** instead of Python's
  `rho0, self.rho = self.rho, 1.0` … `finally: self.rho = rho0`. This is a grep, not a probe:
  `self.rho` is read at exactly **one** site inside the `jacobian` call tree
  (`engine.py:3678`), so passing 1.0 explicitly is bit-identical by construction. The
  save/restore is a Python idiom for a `&self` method, not physics.
- **`round(Tt4, 3)` is implemented, not simplified away** — format-and-parse, validated against
  PyPy on the whole key population and on adversarial ties (probe 1).
- **`lp_disabled` is a `Degenerate(SpoolTransient)` enum variant**, on slice K's own
  `TwoSpoolMapMatcher::Degenerate` precedent; `map_lp` / `map_hp` / `rho` are set-and-unread on
  that path and are NOT carried.
- **`equilibrium` returns a struct with `PartialEq`** — rung 44 gate 1 compares two returns with
  `==` over a 44-key dict: 42 floats, one `str` branch label, and one `Gas`. Measured **on the
  comparison the gate actually makes** — two SEPARATE `TwoSpoolTransient` objects (`bare` and
  `armed`, built with differently-armed maps), not one object twice: the two `wgas` are the
  **same object** (`is` → True), because the working gas is memoised upstream of the maps. So the
  `==` does not exercise gas value equality, and Rust needs `PartialEq` only over the floats and
  the branch label. (`Gas` does define `__eq__` and the two compare equal anyway, so the decision
  is safe either way — but it is now measured rather than inferred from a one-object test.)

**MODULE DECISION AND SIZING.** A new `src/two_spool_transient.rs` — `two_spool.rs` is already
2 064 lines and this is a different plant (two states, an ODE, two diagnostics families).
Composition over rung 39's `TwoSpoolMapCore` through a `pub inner`, as `combustor.rs` composes
over `SpoolTransient`. **592 Python source lines and 815 test lines over 16 tests**, against slice
P's 720/19 and slice Q's ~450/7 — so roughly slice P's size, and the largest slice of phase 6 so
far. Both `slow` markers are measured rather than inherited: **gate 7 runs in 0.2 s** on PyPy
(20 marches, 500 points) and gate 5 is the expensive one (two gases × 7 shapes × 3 throttles of
2-D Jacobians). Slice M's rule stands — port the gate, drop the marker, re-introduce `#[ignore]`
only against a MEASURED Rust cost.

**THE DEFERRAL DUE HERE.** `rung41.rs`'s roster #2,
`test_reduce_transient_untouched_by_surge_line_bit_for_bit`, is the LAST outstanding item from
slice L and is discharged at step 3: the roster goes **11 → 12** and its printed
`STILL DEFERRED -> phase 6 slice R` line is removed. *Slice O's lesson — the note that reached the
next slice was the one written where its compiler and tests would hit it.*

**THE FOUR STEPS.** 1: `src/two_spool_transient.rs` + `oracle/dump_slice_r_smoke.py` +
`tests/slice_r_smoke.rs`, with the injections of predictions 3, 5 and 8 MEASURED rather than
reasoned. 2: `tests/rung40.rs`, the 9 Python gates. 3: `tests/rung44.rs`, the 7 Python gates, and
`rung41.rs`'s roster discharged. 4: `oracle/dump_two_spool_transient.py` +
`tests/two_spool_transient_oracle.rs`, PyPy and CPython arms — the CPython arm carrying probe 4's
measured expectation, so it is read as a detector with a known sensitivity rather than as coverage.

##### STEP 1 — SHIPPED. **PREDICTION 8 IS REFUTED IN BOTH HALVES, AND THE SUITE IS 17 TESTS, NOT 16**

`src/two_spool_transient.rs` (1 250 lines) + `oracle/dump_slice_r_smoke.py` +
`tests/slice_r_smoke.rs` + `tests/slice_r_dispatch.rs`. **1 182 values bit-exact against PyPy**, over
nine sections chosen to touch every path once, and the crate is **691 run / 0 failed** over its
whole test tree: the forward closure driven directly, the instant on BOTH gases, the 2-D Newton at temperatures probe 3 measured taking
DIFFERENT exit branches, `sigma_crit` through the inherited `match`, the 2x2 with both eigenvalue
arms and both `b*c` signs, a 25-point march dumped point-by-point, the two running-line references,
rung 44's excursion at the ramp where the memo collision fires, and the `lp_disabled` reduce.

**THE SUITE COUNT IN § 5.15's OWN OPENING IS WRONG, AND IT MOVES A STEP.** *"Two Python suites, 16
tests (9 + 7)"*, counted rather than taken from the header — collected: **17 (9 + 8)**.
Three counts are in play and the sentence needs its noun: `test_rung40.py` names **8 gates** in its
docstring, defines **8 functions** and collects **9 items** (gate 5 is parametrized by gas);
`test_rung44.py` names **6 gates**, defines **8 functions** and collects **8 items**. The plan's
"9 + 7" is item-count language with rung 44's off by one, so **step 3 owes 8 test functions / 8
collected items, covering the 6 gates its docstring names**. That opening paragraph exists because
an earlier reading of the CLASS was wrong and was fixed by counting; the count of the SUITES was
then made the same way and is off by one in the same direction. *A count is only as good
as the thing you point it at.*

**PREDICTION 8, MEASURED IN BOTH DIRECTIONS, AND WRONG IN BOTH.** It registered that replacing
`int(round(s_end/ds))` with a truncation is *"INVISIBLE to all 16 Python gates"* — the one gate
whose grid makes it live asserting a 0.2 threshold against a measured 0.398 — and that the oracle
catches it *"through the trajectory length AND the values, since a march one step shorter also
moves every extremum taken over it."*

- **The Python arm: rung 40 gate 7 FAILS.** Not at the 0.2-vs-0.398 margin the prediction reasoned
  about, but four lines earlier, at `assert elo * ehi < 0.0` — the bracket-existence check. Dropping
  the last step turns the excursion at `rho = 0.6*sigma_crit` from positive to negative
  (`-1.256e-03` and `-6.942e-03`, same sign), so the bisection has nothing to bisect. The prediction
  measured the WRONG ASSERTION's margin: it read the gate's headline bar, not the bar that runs
  first. 16 of 17 tests pass under the injection.
- **The Rust arm: the extremum the gate uses is BIT-IDENTICAL, so "and the values" is false where
  it matters.** The injection moves **14 of 1 155 compared keys, plus 19 that are never compared at
  all** (point 24's fields, which a 24-point march cannot emit). `G/slip_excursion` at `rho = 1` and
  at `rho = 2` come back bit-for-bit unchanged - those are the ramp gate 7 runs, and its extremum
  sits at saturation. What moves is `F/npts`, ten census counts, the missing-point line, and
  `G/slip_excursion_slow` - the NON-saturating cell, which exists only because a DIFFERENT injection
  demanded it (below). So the length and the census are what see a short march; the excursion value
  sees it only on a ramp § 5.15 never had a reason to run.

**NINE DEFECTS INJECTED INTO THE SHIPPED RUST, AND TWO ARE INVISIBLE TO 1 174 BIT-EXACT VALUES:**

| injected defect | smoke keys moved | where |
|---|---|---|
| `_close` returns the FACE flow as `mdot_air` | **122** | A B C E F H + census |
| `round` -> truncation in `integrate` | **14** (+19 never compared) | F G census |
| `best` captured AFTER the Newton step | the Newton stops converging | C (raises) |
| the noise-floor exit deleted | the reacting cell raises | C (raises) |
| the `steady` memo keyed on the EXACT float | the KEY SEQUENCE grows by one | H (no golden for `key/251`) |
| rung 40's LINEAR reference unified with rung 44's per-instant one | **2** | G + census |
| the high wall drops the literal `2.5` arm | **2**, both `illinois_evals` | census ONLY |
| `best` keeps the LATEST tie (`<=` for `<`) | **0** | INVISIBLE *on this grid* — the INFERENCE, not the count, is corrected at step 4 (9 keys on the reacting one) |
| the march-in ladder as `0.02*(k+1)` | **0** | INVISIBLE |

**AND TWO ROWS OF THAT TABLE ARE THE SLICE'S FIRST CONTENT.**

- **THE CONTESTED `min` IS WORTH NOTHING BUT AN ITERATION.** Probe 2 made a headline of the high
  wall being *"genuinely contested"* — 1 221 literal against 5 118 map, where rung 37's comparable
  ceiling bound 15 of 15 on one arm — and the smoke reproduces both arms (2/4 in section D, 6/9 in
  E). Dropping the literal arm entirely moves **no value at all**: `census/D/illinois_evals` 42 → 43
  and `census/E/illinois_evals` 99 → 101, and every root comes back BIT-IDENTICAL from a different
  bracket. So *"both arms are live"* is a statement about which arm is TAKEN, not about what the
  choice is WORTH — the Illinois converges to the same double either way. Probe 2's census stands;
  the importance it implies does not.
- **THE DEFAULT RAMP CANNOT SEE THE REFERENCE CHOICE, FOR AN EXACT REASON.** Probe 5 measured rung
  40's linear reference and rung 44's per-instant one agreeing to seven figures at the extremum. On
  `r_ramp = 0.5, s_end = 1.2` (gate 7's own pair) they agree **bit-for-bit**, because the extremum
  is attained at `s = 0.5`, the instant the ramp SATURATES: there `u == 1` exactly, so the linear
  interpolation IS the endpoint match. The first version of section G therefore reported the
  unification as one census key and ZERO values. A non-saturating cell (`r_ramp = 3.0`, where the
  two differ by 2.4 %: `-7.1498e-04` against `-6.9835e-04`) was added, and the injection now moves a
  VALUE. *The pointwise keys the section already carried could not do it either — they are computed
  in the dump and in the test, so they gate the arithmetic and not the shipped body's CHOICE.*

**THE THREE MANUFACTURED-FAILURE GATES SHIP** (`tests/slice_r_dispatch.rs`, one `#[test]` in its own
binary so the thread-local counters cannot be stolen):

- **prediction 3 — the dispatch is live on BOTH tables.** Each of rung 40's three cells is swapped
  for a wrapper perturbing ONE number by 1e-9 relative, and each is caught where it should be and
  nowhere else: `try_close` moves the closure AND the residuals downstream, `try_instant_tail` moves
  the thrust and NOT the closure, `powers` moves the converged speeds and NOT the closure (the
  Newton alone reads it). Then the INHERITED rung-39 cell is swapped and both `lead_threshold` and
  rung 44's steady reference move — the edge that is structurally new in this slice, and slice O's
  lesson applied before the fact rather than after.
- **prediction 5 — a truncated march is visible in the length key.** The closure is starved after 30
  calls; the march comes back shorter, exactly one truncation arm is counted, and every point it DID
  produce is bit-identical to the full march's. Without this, "0 truncations, gated against zero" is
  a gate that has never fired.
- **prediction 4 — the memo's equivalence relation is gated by the KEY SEQUENCE.** The dump builds
  both keying schemes off the SAME trajectory: **251 rounded keys, 252 exact ones, 1 collision** —
  and the shipped miss sequence, recovered from the `match` calls, is asserted to BE the rounded one.
  Keying on the bits is then caught by a golden that does not exist for `H/exc/key/251`.

**AND THE COMPARATOR ITSELF HAD A GATE THAT COULD NOT FIRE.** `Cmp::finish` asserted the VALUE
diffs before the never-compared ones, so the half that exists to catch a field missing from the PORT
was unreachable whenever any value also moved - which is exactly what a short march does. The two
are now ONE panic carrying both halves, section G reads its points with `get` (it used to PANIC at
point 24 and abort before either half printed), and the guard has been WATCHED TO FIRE: commenting
out one `c.f` line makes it report `["A/v0"]` by name. *The documented gate that does not exist, one
file down, found by the injection it was built for.*

**WHAT THE SMOKE CANNOT SEE, SAID PLAINLY.** `best`'s strict `<` versus `<=` is invisible here: on
this grid no two Newton passes leave EXACTLY equal residuals, so the tie-break is never exercised -
step 4's larger reacting grid is where it could be, and that is registered rather than assumed
covered. **DISCHARGED AT STEP 4, AND THE READING INVERTS: 22 ties on the reacting arm against 0 on
the CPG one, all in noise-exit cells, and a `<=` moves 9 keys.** The march-in ladder's spelling is likewise unwitnessed by any value, because probe 2
measured the loop dead; it is gated by its counter being zero and by nothing else, which is the
honest statement. Three Rust counters have no Python column at all - the march-in advances, the
non-real guard and the `g` failures they come from are SWALLOWED by the shipped body, and a wrapper
cannot count what the body catches; copying the body into the dump would have made the dump's
arithmetic a copy rather than the shipped code. Every dead arm named above is ASSERTED against zero
in `census`, including the Newton's damper and its `1e-30` floor, which were counted but unread
until this was written.

**PORT DECISIONS THAT SURVIVED CONTACT.** `round3` is format-and-parse; `rho` is a mutable field
with a `jacobian_at_rho` twin so the two band methods pass `1.0` explicitly; `Instant2`'s
`PartialEq` is hand-written over the 42 floats and the branch, `wgas` excluded on the measured
grounds that rung 44 gate 1's comparison never exercises it; and the two `mdot_air`s in `_close` are
two named locals — which the 118-key injection says is the single most load-bearing line in the
file. The dump enumerates PYTHON's dict keys (**21** and **44**, both asserted), and the comparator
now fails on any golden key the Rust never asked for: without that, a field missing from the port
would be missing from the comparison too.

##### STEP 2 — SHIPPED. **THE GATE THAT ANCHORS THE OBJECT IS BLIND AT THE THROTTLE IT STARTS ON**

`rust/tests/rung40.rs` (591 lines) — the 9 collected items of `tests/test_rung40.py`, **8 test
functions**, gate 5 split into a CPG and a reacting `#[test]` because Python parametrizes it.
**9 run / 0 failed in 3.0 s**, and no `src/` edit: every field the port needed (`a4`/`a45`/`a8` off
`inner.base`, the `ComponentMap` shape fields, `SpoolTransient`'s eight equilibrium names) was
already `pub`, so the step is test-only exactly as the step table assumed. Seven of the nine passed
on the first compile; the two that did not were gate 5's count bars, left at `0` on purpose so the
number had to be measured rather than transcribed.

**PREDICTION 9's BAR IS FOUR NUMBERS, NOT TWO, AND THE PLAN'S PAIR IS A SUM.** § 5.15 registered
*"245 real / 7 complex on gate 5's grid"*. That is the two gases ADDED: measured per gas, the split
is **124/2 on CPG and 121/5 on the reacting gas**, 126 each, and the two sum to exactly the
registered pair. Both halves were then re-measured on the PYTHON side (`probe_r6_eig.py`,
re-evaluating the shipped discriminant `tr² − 4·det ≥ 0` on the same `J`, since Python has no
counter) and agree cell for cell. So the bar ships as four constants plus an assertion that their
total is the grid size `7 × 3 × 6` — a count read against a grid whose size is itself asserted,
which is what [[rust-port-slice-n-step4]] cost. *Registering a sum and gating a split are not the
same measurement.*

**PREDICTION 10's SECOND HALF HAD NO PYTHON GATE, AND IS WRITTEN HERE RATHER THAN LEFT TO PROSE.**
The prediction is that the `lp_disabled` reduce holds bit-for-bit *"and every other method on that
variant panics"*. Python's gate 2 tests only the forward. The panic half now sits inside gate 2 via
`catch_unwind` — `core()` and `core_mut()` on the degenerate variant, plus the MIRROR
(`degenerate()` on the full variant), so the guard is a discriminator and not a blanket panic that
would satisfy the same assertion for the wrong reason. It goes inside gate 2 deliberately: a tenth
`#[test]` would move the collected count that step 1 had just finished correcting from 16 to 17.

**AND THE STEP'S CONTENT — GATE 3's FOUR REDUCE BARS ARE STRUCTURALLY BLIND AT ITS FIRST THROTTLE.**
Gate 3 is the non-tautological one: a bare-math CPG two-shaft closure, no `Gas`/component/
`ComponentMap`-method/solver call inside it (grepped, not assumed — the only two crate calls in its
body are the `match_point` and `lead_threshold` it compares AGAINST). To check it is not vacuous,
the map's linear loading slope `l` was deleted from the bare `psi`. It IS caught — but by the
`sigma_crit` bar, not by the four speed and pressure-ratio bars four lines above it:

| injected: `l` dropped from `bare_psi` | `Tt4 = 1500` (design, and FIRST in the sweep) | `Tt4 = 1300` |
|---|---|---|
| `\|Δnu_L\|` | **4.4e-15** — six orders INSIDE the `1e-8` bar | **2.9e-2** |
| `\|Δnu_H\|` | 7.3e-15 | 1.1e-2 |
| `\|Δpi_LPC\|/pi_LPC` | 9.9e-15 | 9.7e-4 |
| `sigma_crit`, bare vs shipped | **1.0000001 vs 1.1483** — the bar that fires | — |

At the design throttle the flow coefficient is `1` by construction, so `l·(phi − 1)` vanishes
IDENTICALLY and the whole map-shape channel drops out of the reduce bars; the off-design throttles
see it at three million times the bar, but the sweep never reaches them. This is the gate's own
docstring — *"reproducing the `==1` identity alone would only re-check the reduce"* — shown rather
than asserted, and it sharpens it: **which assertion in a two-path gate is the discriminator depends
on where the sweep STARTS, and this sweep starts at the one point where three of its four bars
cannot move.** Same family as slice J's *exactness bounds the CELLS visited, not the RULES
discriminated*, one level up: here it is the CELL ORDER that decides which rule is under test.

**THE TWO SILENT DEFAULTS PYTHON HAS AND RUST DOES NOT, WRITTEN OUT.** `lead_threshold`'s `d`
is **not uniform inside gate 4** — (a) and (b) pass `25.0`, (c) leaves the default `5.0`
(`engine.py:3644`); and gate 7's `slip_excursion` names only `s_end` and `ds`, leaving
`r_ramp = 0.5` (`engine.py:3791`). Carrying `25.0` into (c), or guessing `r_ramp`, changes the
physics without failing loudly — a defaulted argument is a value the source states exactly once,
far from the call.

**GATE 7's ASSERTION ORDER IS PRESERVED AND SAID SO AT THE SITE.** `elo * ehi < 0.0` stays four
lines ahead of the `0.2` margin, because step 1 measured that the bracket check — not the headline
margin — is what a truncated step count breaks. Merging the two would silently retire that finding.

**BOTH `slow` MARKERS DROPPED, AGAINST A MEASURED COST.** Gate 5 (`@pytest.mark.slow`, both gases)
and gate 7 run inside a **3.0 s** whole-file wall clock, so slice M's rule gives no `#[ignore]`.

**AND THE LINE COUNT IN THIS PARAGRAPH WAS WRONG TWICE, IN OPPOSITE DIRECTIONS.** It was first
written as 591 without ever being counted (step 1's *count the thing you name*, one step on),
corrected to the measured **579**, and then the two doc paragraphs this section describes pushed
the file back to **591** — so the un-measured guess is now right by coincidence and the measurement
that replaced it is stale. *A count is measured AFTER the last edit, or it is a guess about a file
that no longer exists.*

##### STEP 3 — SHIPPED. **AN INVARIANCE GATE IS SATISFIED BEST BY DELETING THE VARIABLE IT VARIES**

`tests/rung44.rs` (**607** lines, counted after the last edit) — **9 run / 0 failed**, no `src/`
edit — verified by `git diff --stat -- rust/src/` coming back empty AFTER the injection harnesses,
which had written to that file thirteen times — and `rung41.rs`'s roster discharged **11 → 12**.
The crate is **709 run / 0 failed**, summed over its 79 `test result` lines by a clean run with nothing else touching cargo — the first attempt at this number overlapped the injection harnesses and reported `684 / 1`, which is an artefact of rebuilding mid-run and not a result.

**NINE FUNCTIONS FOR AN EIGHT-FUNCTION FILE, AND THE NINTH IS THE DEFERRAL.** `test_rung44.py`
names **6 gates**, defines **8** functions and collects **8** items — no `parametrize` anywhere, so
for once two of the three counts coincide, exactly as step 1 predicted after correcting § 5.15's
"9 + 7". The ninth `#[test]` is slice L's last outstanding item,
`test_reduce_transient_untouched_by_surge_line_bit_for_bit`, which lands in `rung44.rs` rather than
`rung40.rs`: what it gates is a *surge line left unread by a transient*, which is rung 44's subject,
not rung 40's. `rung41.rs`'s roster, its header table row and its assertion message are all updated
to point at where it went, and its `for … if !p` print loop is KEPT although the list is now
all-`true` — it is what turns a re-opened deferral back into a visible line instead of a silently
shorter roster. **The IOU from slice L is now CLOSED.**

**THE PORT AGREES WITH PYTHON ON VALUES, NOT ONLY ON THE SIGNS ITS GATES ASSERT.** Every assertion
in this file is a sign, an ordering, a monotonicity or a spread — none of them pins a number, so a
port that was wrong by a few per cent would pass all nine. A throwaway probe on both sides
(`M:/claud_projects/temp/rust-phase6/probe_r44_values.py` and a Rust twin, deleted after use)
dumped the **49 floats and 11 discrete flags** these gates read — the five shape pairs' accel and
decel excursions, their ratios, their damping ratios and band-existence flags, the five-point `rho`
sweep, the six-point ramp sweep, and both margin records — and the two files **diff to nothing but
`True`/`true` and `1.0`/`1`**. So the gates are sign gates sitting on top of bit-exact values, and
that is now measured rather than assumed. **CORRECTED AT STEP 4:** that probe hard-coded
`R_c = 286.9` on BOTH sides, and `test_rung44.py` DERIVES `286.857…` — so it measured this file
against rung 40's gas, which is also the gas this file shipped with. The values were bit-exact; the
GAS was its neighbour's. See § 5.15 step 4 finding 1. *An instrument that supplies the input to both
sides cannot see a wrong input.*

**THE FINDING: AN INVARIANCE CLAIM'S CONFIRMING TEST IS MAXIMALLY SATISFIED BY DELETING THE
MECHANISM.** Ten defects were injected to size the gates. **Six** are caught by the gate whose claim they break, **two** by the step-1 smoke and no rung gate at all, and **two** by NOTHING:

| injected defect | who fails | reading |
|---|---|---|
| swap the two steady references in `phi_excursion` | gate 3, gate 4 (c) | the LP/HP split is genuinely gated |
| store `\|extremum\|` instead of the SIGNED one | gate 3, gate 2 | the sign is the load-bearing claim, and the bare reference sees it too |
| transient minimum read off the STEADY line | gate 5 | the flip is a real comparison, not a restatement |
| drop the unarmed-map refusal | gate 5 | the `pytest.raises` leg ports live |
| ramp saturates instantly (`r_ramp` dead) | gate 4 (b) | the ramp-rate monotonicity is not an artefact of the grid |
| drop `l` from the TEST's own bare loading law | gate 2 | the non-tautological reference is not vacuous **on a marched object** — step 2's injection, one rung on, and here it fires at the headline bar rather than four lines above it |
| **delete `rho` from the SHIPPED marcher** | **NOTHING, in either rung suite** | see below |
| delete `rho` from the BARE march in gate 2 | **NOTHING** | the same hole, in the reference |
| `min_phi_lp/hp` tracked as a MAX | **no rung gate** — only `slice_r_smoke` | a reported field no gate reads |
| `s_lp` (WHERE the extremum sits) frozen at 0 | **no rung gate** — only `slice_r_smoke` | likewise |
| **the forward closure READS `phi_surge`** (`1e-9` of it, in the loading law) | gate 1 **and** the rung-41 discharge | the REDUCE SPINE is a live discriminator, not a self-comparison that cannot fail |

Gate 4 (a) asserts that the excursion moves less than 5 % as `rho` sweeps 25×. Deleting `rho` from
the marcher outright sends that spread to **exactly zero**, which satisfies the bar *more
comfortably than the truth does*. The same hole sits in gate 2's own `rho`-invariance leg. Nothing
in `rung44.rs` could see either, and what caught the shipped one was
`rung40.rs::gate7_scope_sigma_crit_is_first_instant_only`, a rung away and aimed at something else.
**A gate that certifies a variable POWERLESS cannot, by construction, distinguish powerless from
ABSENT — so it needs a second bar saying the variable is READ.** Both gates now carry
`assert!(lo < hi)` beside the spread, and re-injection confirms each defect is caught by the gate
whose claim it breaks. This is a bar Python does not have; it is written here for the same reason
step 2's panic half was — the port is where the missing direction became visible.

**THE REDUCE GATES NEEDED THEIR OWN INJECTION, BECAUSE A SELF-COMPARISON CANNOT BE VALUE-CHECKED.**
Sorting the nine by what actually validates them splits them cleanly: gates 2, 3, 4 (a/b/c) and 5
are bit-checked against PyPy by the probe **and** sized by an injection, while gate 1, gate 1 (b)
and the rung-41 discharge compare two runs of the SAME code against each other — there is no
absolute value for a probe to diff, so nothing but an injection can reach them. Making the forward
closure read `phi_surge` at the `1e-9` level fails **both** gate 1 and the discharge test, so both
discriminate; the discharge test is otherwise a strict SUBSET of gate 1 apart from its floor value,
and its doc comment now says so rather than implying it adds coverage (`Instant2`'s `PartialEq`
already compares the two power residuals it names). *Rung 63's over-claim shape, caught before it
shipped this time.*

**AND GATE 1 (b) WAS WEAKER THAN PYTHON'S UNTIL IT WAS REWRITTEN.** It first built a fresh engine
for each of the two runs; Python re-runs ONE object. Rebuilding still sees a global that the
diagnostics disturb, but not a mutation of the engine itself — which is the channel the gate exists
for. It now reuses one object. Rust's `run(&self)` makes that channel hard to open at all, so the
gate is thinner here than in Python either way, and the docstring says so instead of letting a
green tick imply otherwise.

**AND THE TWO FIELDS NO RUNG GATE READS ARE COVERED, ONE LAYER DOWN.** `min_phi_lp`/`min_phi_hp`
and `s_lp`/`s_hp` are emitted by `phi_excursion` and asserted by nothing in either Python suite;
corrupting them is invisible to all 17 rung tests and caught only by step 1's smoke oracle. That is
the right architecture — a value dump is what covers reported-but-unasserted fields — but it is
worth saying which layer holds which claim, because *"the rung suites pass"* would otherwise be
read as covering the whole record.

**THE HARNESS ITSELF PRODUCED A FALSE READING FIRST.** The first injection run reported every
defect as a COMPILE ERROR (the detector matched cargo's own `error: test failed` line) and, once
that was fixed, reported a **failing baseline** and three defects landing on a gate that does not
touch them. Cause: the revert used `mv "$f.bak" "$f"`, which restores the backup's mtime, so cargo
did not rebuild and the next injection ran against the *previous* one's binary. `cp` + `touch` on
revert fixes it. *An injection harness needs its own baseline to pass before any row it prints is
evidence* — three rows here were pure carry-over from the row above.


##### STEP 4 — SHIPPED. **THE ARM STEP 1 REGISTERED AS UNWITNESSED IS REACHED, AND ONE SUITE'S GAS WAS ITS NEIGHBOUR'S**

`oracle/dump_two_spool_transient.py` (**735** lines, three arms) +
`tests/two_spool_transient_oracle.rs` (**1 046**, both counted after the last edit — step 2's
postscript). **6 853 main keys and 1 120 reacting keys BIT-EXACT against PyPy on the first run**,
over sections A–L (both suites' CPG grids) and P–S (the reacting cells), plus a tiered CPython arm.
Two `src/` edits, both instruments — `eq_ties` and `steady_tt4_all` — and one TEST edit that is a
defect fix.

**THE ARCHITECTURE DECISION WAS SPIKED BEFORE THE DUMP WAS WRITTEN, AND IT WENT THE GOOD WAY.**
§ 5.15 prediction 1 registered the reacting-gas `equilibrium` keys as the slice's one genuine
exposure and pre-committed § 9 Decision 1's **Option B** — an individually-adjudicated fragile set
with a published deviation distribution — as the route if 100 % failed. The prediction's own
mechanism makes it binary: the exit branch is decided BELOW `_EQ_TOL`, so bit-exactness gives 100 %
and one differing bit flips a discrete key. So probe 4's twelve cells were run through the shipped
Rust FIRST, against a 30-second PyPy re-run of `probe_r4.py`: **all twelve agree on the exit kind,
the pass count and both converged speeds.** The arm therefore ships at bit-equality and Option B
stays unused — *decided by a two-cell-shaped spike costing minutes, rather than discovered after
500 lines of dump.*

**FINDING 1 — `test_rung44.py` AND `test_rung40.py` RUN DIFFERENT CPG GASES, AND `rung44.rs` HAD
ITS NEIGHBOUR'S.** `test_rung40.py` hard-codes `R_c = 286.9`; `test_rung44.py` writes
`R_c = (gamma_c-1)/gamma_c*cp_c` = **286.8571428571428**. Step 3 shipped `rung44.rs` with `286.9`,
copied from the file beside it, so **every rung-44 gate ran rung 40's gas**. Nothing could see it:
every assertion in that file is a sign, an ordering, a monotonicity or a spread — step 3 said so
itself — and step 3's own value probe used `286.9` on BOTH sides, so the one instrument aimed at
values was blind to it too. All nine gates still pass on the corrected constant, which is the
measurement of how invisible it was. The Python population splits cleanly: `test_rung30/31/33/38/
39/40/43` hard-code the number and `test_rung41` onward derive it; **every other ported suite —
`rung41/42/53/54/55/56/61` — has it right**, so this is an isolated slip and not a family. Found by
enumerating each suite's grid for the oracle instead of reading a constant off its neighbour, and
the dump now carries both gases' `R_c` and `R_t` as section A so the two can never be confused
again. *Slice N step 4's shape — two censuses on two grids, read as though they shared one — here
as two SUITES on two gases.*

**FINDING 2 — THE `best` TIE-BREAK IS REACHED, AND ONLY ON THE REACTING ARM.** Step 1's postscript
was explicit: *"`best`'s strict `<` versus `<=` is invisible here: on this grid no two Newton passes
leave EXACTLY equal residuals — step 4's larger reacting grid is where it could be, and that is
registered rather than assumed covered."* The advisor's review caught that the plan's step-4 entry
did not mention it, which is how a registered IOU rides one more step. An `eq_ties` counter was
added and measured:

| arm | `equilibrium` calls | ties | where |
|---|---|---|---|
| CPG (section D) | 22 | **0** | — |
| reacting (section P) | 12 | **22** | `flat/1450` (17), `flow_press/1100` (4), `flat/1300` (1) |

All three tying cells take the **NOISE exit**, which is the only exit that READS `best`; no CPG cell
ties at all, because the residual there falls under the absolute `1e-12` bar in four or five passes
instead of plateauing for eighty. Injecting `<=` then moves **9 of the 1 120 reacting keys** — every
one in `P/flat/1300`, the single-tie cell, at the last bit. The 17-tie and 4-tie cells do NOT move,
because a tie only reaches the return when the tied residual is also the minimum. **So step 1's
injection row — *"`best` keeps the LATEST tie → 0 keys moved → INVISIBLE"* — is CORRECTED by this
grid rather than confirmed**, and the count is gated PER ARM: a sum would have let the reacting 22
hide behind a CPG zero, which is exactly what step 2 paid for registering a sum instead of a split.

**AND THE COUNTER NEEDED A SECOND ONE TO BE HONEST.** `steady_tt4` records the raw `Tt4` behind each
memo MISS, so counting its distinct values measures the rounded relation against itself and reports
0 collisions always. The counterfactual — how many entries an EXACT-float memo would hold — is the
distinct raw `Tt4` over EVERY lookup, and a rounded HIT whose raw value is new IS the collision. A
second instrument, `steady_tt4_all`, records every lookup, so the Rust reproduces Python's
`251 rounded / 252 exact / 1 collision` without re-marching — which would have doubled the census
the high wall's arm is read out of. *The first draft of that key would have passed by construction.*

**FINDING 3 — PREDICTION 9's REGISTERED PAIR IS REPRODUCED FROM AN INDEPENDENT CONSTRUCTION.**
§ 5.15 registered *"245 real / 7 complex"* and step 2 corrected it to a per-gas split of 124/2 (CPG)
and 121/5 (reacting). Sections F and R rebuild gate 5's 7 × 3 × 6 sweep from the dump's own shape
table, and land on **124/2 and 121/5 exactly** — so the two are a corroboration rather than one
number quoted twice, and each is asserted beside `eig_real + eig_complex == 7*3*6`, a count read
against a grid whose size is itself asserted.

**PREDICTIONS 6 AND 7 HOLD AT SCALE, AND THEY ARE ASSERTED PER SECTION RATHER THAN ONCE.** Across
all **22** census blocks (17 main + 5 reacting): the low-wall march-in advances, the non-real guard, both march truncation
arms, the `max(0.2, ·)` speed floor, the Newton's damper and its `1e-30` floor are **0**, and
`illinois_exhausted` is **0** at rung 40's call site — against slice Q's **103 of 109** at
`_plenum_pt4_at`, the same counter on the opposite population, reported with its grid and never
summed with slice Q's. The high wall's contested `min` takes its literal arm in **10 of the 22**, which is what the census-only defect of step 1 needs to be catchable at all.

**FINDING 4 — THE CPython ARM REPRODUCES PROBE 4 EXACTLY, AND IT CONTRADICTS **BOTH HALVES** OF
SLICE N's RULE.** `slice_n_oracle.rs` states it plainly: *"iteration counts are not
interpreter-invariant … the branch verdicts ARE, because a verdict is a comparison and not an
iteration."* This arm was built on the opposite population and both halves fail:

- **The verdicts move.** Probe 4 pre-registered 5 of 12 exit-branch flips and 10 of 12 differing
  pass counts on the reacting Newton. The dump reproduces **5/12 and 10/12**, from a grid rebuilt
  independently in Rust. Copying slice N's precedent would have shipped a gate that fails.
- **An iteration count moves where every value it produces does not.**
  `census/I/shapes/illinois_evals` reads **54 323 on PyPy and 54 322 on CPython**, and
  `census/K/illinois_evals` 38 513 against 38 512 — on the CPG half, where **all 6 410 float keys
  are bit-identical**. One convergence test lands on the other side of its bracket and the Illinois
  returns the SAME double from a different last step. Slice N's first half, holding.

**AND THE CPython DIFF SPLITS BY MECHANISM INTO FOUR CLASSES, EACH WITH ITS OWN CURRENCY:**

| class | keys | moved | worst | why |
|---|---|---|---|---|
| **CPG closed-form** | **6 410** | **0** | — | bit-equal; this is the arm's coverage half |
| reacting DIRECT | 629 | 523 | **1.06e-10 rel** | the equilibrium sub-solve's own noise |
| reacting RESIDUAL | 30 | 30 | **8.17e-11 abs** | the same noise on a quantity whose truth is ZERO |
| reacting DERIVATIVE | 357 | 357 | **6.37e-3 rel** | that noise divided by the Jacobian's `h = 1e-6` |
| thermally-perfect | **1** | 1 | 1.40e-9 rel | see below |

The residual class is why the bar is not one number. `L/1500/Phi` reads **1.281e+04 RELATIVE** and
`2.9e-11` absolute; only the second is a statement about anything, and a single relative bar loose
enough to admit it would admit everything. The derivative class is the same noise amplified a
millionfold by the finite difference, worst on `bc` because it is the product of the two SMALL
off-diagonals. *A finite difference inherits drift from the quantity differenced* — the golden-gate
lesson, arriving on an interpreter arm.

**FINDING 5 — ONE KEY IN 6 411 SAYS THE TPG TABLE GAS IS NOT INTERPRETER-STABLE EITHER.**
`E/channel/gas` is the single value in the whole main dump built on `Gas.thermally_perfect()` —
gate 4's *"the `cp(T)` curve is one of `sigma_crit`'s two breaking channels"* — and it is the ONLY
non-reacting float that moves, at 1.40e-9 relative. Every closed-form CPG key beside it is
bit-identical. So the instability is the TPG **table integrals'** `log`/`exp`, exposed by
`lead_threshold`'s own `d = 25 K` finite difference, and it is a property of the GAS rather than of
this rung. Recorded because a one-key drift in a 6 411-key bit-exact half is easy to file as a
tolerance and it is a measurement.

**TWO `src/` EDITS, AND THAT BREAKS A TWO-STEP STREAK ON PURPOSE.** Steps 2 and 3 both shipped
test-only, verified by `git diff --stat -- rust/src/` coming back empty, and both said so. Step 4
does not, and the reason is that neither instrument can live in a test file: `eq_ties` needs a site
INSIDE the Newton's loop, between the tolerance check and the `best` update, and `steady_tt4_all`
needs one inside `SteadyRef::at`. A wrapper cannot reach either — the same argument the dump makes
for its three swallowed counters. Both are counters; neither touches an arithmetic line, which
`git diff` shows directly.

**COSTS, MEASURED, AND THE CRATE'S END STATE.** The dump: **6.2 s** (main, PyPy) / ~4 min (equil,
PyPy) / ~25 min (cpython, one-off — CPython runs the main arm **7.6×** slower than PyPy, timed
rather than taken from `CLAUDE.md`'s whole-suite 6.2×). The Rust: the whole 4-test file in
**25–29 s** release, three of whose four tests re-run the reacting sweep. Slice M's rule gives no
`#[ignore]` against that, and the crate keeps its zero. **The crate is 713 run / 0 failed /
0 ignored over 80 targets**, measured on one clean run — step 3's 709 over 79 plus this file's four,
which is the arithmetic checking out rather than the number being carried forward.
`pytest` is untouched by this step: the only Python written is `rust/oracle/*.py`, and
`pytest.ini`'s `testpaths = tests` excludes it — checked, not assumed.


### 5.16 SLICE S (rungs 43 + 45, `TwoSpoolFuelTransient`) — PRE-REGISTERED, four probes MEASURED first; **ALL FIVE STEPS SHIPPED**

Rung 43 puts rung 35's FUEL control on rung 40's two-shaft plant — `Tt4` becomes an OUTPUT of a
forward burner — and rung 45 marches that plant against rung 41's imposed surge line. This is the
slice § 5.12 settled would port `integrate_fuel` **entire**, because rungs 46–52 are keywords on
one method and not seven classes.

**THE SCOPE IS SETTLED BY GREP, NOT BY THE PHASE TABLE'S PHRASE.** "The whole `integrate_fuel`"
is the METHOD entire — all seven limiter keyword arms, its two private dispatch twins
(`_integrate_fuel_lagged`, `_integrate_fuel_asym`), the three set-point solves they call
(`_topping_fuel` / `_sched_fuel` / `_surge_fuel`) and the three helper classes those need
(`AccelSchedule`, `SurgeLimiter`, `AsymmetricLag`, plus the module-level `_release_weight`). It is
NOT the thirteen rung-46–52 READER methods at `engine.py:5437–6054`. Checked rather than assumed:
each of those thirteen names was grepped across every suite, and **not one is called by
`test_rung43.py` or `test_rung45.py`** — they belong to `test_rung46…52.py`, which are slices T
and U. Shipping them here would be 618 lines carrying zero gates for two slices.

*Two by-products of that grep, recorded where slices T and U will read them:* `deficit_curve`
(`engine.py:5809`) is called by **nothing at all** — no test, no `main.py`, and its only other
mention is a sibling docstring saying the gate is *not* it; and `lag_sweep` is reached only
through `factorization_grid`. Slice U inherits both.

**COUNTED, NOT TAKEN FROM A HEADER** (slice R step 1's correction, applied before the fact).
`pytest --collect-only`: **20 items, 11 + 9**, and for once all three counts coincide — no
`parametrize` in either file, and `def test` count == item count in both. Sizing: **1 103 source
lines** (930 of the class through `transient_surge_margin_fuel`, plus 92 for `AccelSchedule` /
`SurgeLimiter`, 28 for `_release_weight` and 53 for `AsymmetricLag`) over **689 test lines**,
against slice R's 592/815/17 and slice P's 720/19. **This is the largest slice of phase 6**, and
§ 5.12's own "4–6 sessions is light for six slices" gets one more datum. It ships in **FIVE**
steps rather than four, for a reason the probes measured: ~40 % of the source is limiter
machinery that NO phase-6 rung gate reaches (probe 2), so the armed smoke sections are the only
coverage it has and are sized deliberately instead of being an afterthought.

`M:\claud_projects\temp\rust-phase6\probe_s1.py` … `probe_s4.py`. Probes 1b/1c and probe 4 exist
because a review of the first drafts found three things they had not measured — a margin that was
the solver's rather than the predicate's, four limiter keywords nothing had ever executed, and an
error path the suite tests only by bypassing it.

---

**PROBE 1 — § 5.12's IOU IS DISCHARGED, THE DETECTOR FIRES, AND WHAT IT WAS AIMED AT DOES NOT
MOVE.** § 5.12 dumped the arming predicates CPython-vs-PyPy, got **9 376 keys 100 % identical**,
and recorded explicitly that this was NOT coverage because the probe ran a **CPG** gas whose
property calls are closed-form; it booked the reacting-gas measurement here. Slice R probe 4
discharged the GAS half early and wrote that slice S still owes the same measurement **on its own
object**.

The complication that had to be settled first: `_tt4_from_f` asserts `not self.gas.equilibrium`,
so `Gas.reacting_equilibrium()` — the exact gas slice R's detector fired on — is REFUSED by the
fuel path outright. Measured: `thermally_perfect`, `reacting` and `reacting_forkb` all carry
`equilibrium = False` and are **admitted**, and all three are TPG table gases, which slice R
finding 5 measured moving under CPython. So the detector is pointable, and the IOU is live rather
than vacuous.

| admitted gas | value keys moved, CPython vs PyPy | worst DIRECT | `arm_T` / `arm_A` / `arm_S` | `n_cap_calls` | `n_pts` |
|---|---:|---:|---|---:|---:|
| CPG (both suites' recipes) | **0 of 400** | — | identical | identical | identical |
| `thermally_perfect` | **391 of 398** | 9.93e-11 rel | **identical** | **identical** | **identical** |
| `reacting` | **391 of 398** | 4.47e-11 rel | **identical** | **identical** | **identical** |
| `reacting_forkb` | **391 of 398** | 4.47e-11 rel | **identical** | **identical** | **identical** |

**AND "DID NOT MOVE" IS ONLY A DISCHARGE IF THE PREDICATE WAS EVER AT RISK**, so the margins were
measured at the predicate sites — which took two attempts, and the first attempt is the lesson.
Probe 1b timed `Tt4 − Tt4_max` at every `_instant_fuel` and reported a closest margin of **zero**;
that population includes the ~40 evaluations inside `_topping_fuel`'s own Illinois, which converge
ONTO the redline by construction. It had measured the SOLVER, not the predicate. Probe 1c locates
the two branch sites exactly (`der` calls `_instant_fuel` first and `_release_weight` next, so the
last instant before each `_release_weight` IS the predicate's):

| predicate | evals | closest margin | drift, from the table above |
|---|---:|---|---|
| `i["Tt4"] > Tt4_max` | 84 | **6.95 K on 1380 K = 5.0e-3 rel** | ~1e-10 rel |
| `c < mf`, ACTIVE leg | 217 | **1.4e-2 rel** | ~1e-10 rel |
| `c < mf`, DORMANT leg | 21 | **an EXACT structural zero** | immune |

**Seven orders of margin**, so the invariance is a measurement and not a property of a lucky grid.
And the dormant row is content in its own right: `_sched_fuel` / `_surge_fuel` return `mf_sched`
**itself**, which the source justifies purely as what makes rung 48/49's dormant reduce bit-for-bit
rather than merely equal — and that same float-identity makes `mf < mf` unflippable on any
interpreter. *A reduce discipline paying off as numerical robustness.*

**AND THE 400 CPG KEYS WERE BLIND TO A GAS DIFFERENCE THAT IS REALLY THERE.** Slice R finding 1
shipped `rung44.rs` running `rung40.rs`'s gas, because `test_rung40.py` hard-codes `R_c = 286.9`
and `test_rung41` onward derive `(γ−1)/γ·cp`. **Rungs 43 and 45 straddle that same boundary** —
`test_rung43.py:62` hard-codes `286.9`, `test_rung45.py:83` derives `286.8571428571428` — so the
hazard repeats one rung on, and it was checked before a line of Rust was written. The two gases'
**whole 400-key fuel-path dump is bit-identical**: every speed, temperature, pressure ratio, flow
coefficient, applied fuel and residual. Only one channel witnesses the difference at all, and the
probe had not dumped it:

```
R_c = 286.9              sp_thrust = 40853e9ad22b6e1f
R_c = 286.857142857…     sp_thrust = 40853ec406183610      nu_lp, Tt4: BIT-IDENTICAL
```

`R_c` reaches the fuel path ONLY through the static/exhaust conversion, i.e. the thrust key —
which is exactly the channel slice R's 1-ULP hunt landed on. So the oracle carries **both suites'
`R_c`/`R_t` bits as a section-A key AND at least one thrust key per suite**; without the thrust key
a 400-key bit-exact dump certifies nothing about which gas the port used.

---

**PROBE 2 — THE REACHABILITY CENSUS, and the headline is an EXACT TIE that the two languages'
`round` disagree about.** Instrumented by taking each body with `inspect.getsource` and inserting
counter lines by textual substitution — so the arithmetic is LITERALLY the shipped text rather
than a retyping of it (slice O's rule), with every substitution asserted to have applied (slice N
step 2's three instruments that measured nothing). Over both suites' full grids: **227 889
`_close_fuel` calls, 227 856 `der` calls, 162 marches.**

**EVERY NUMBER IN THIS PROBE IS ON A GRID THAT IS NOT THE SUITES', AND ITS HEADER SAYS OTHERWISE.**
Step 4 wrote out `test_rung43.py`'s ten gates and `test_rung45.py`'s nine as they are actually
swept and measured **231 409 closure calls and 143 marches**. The counts below are kept as this
probe's, with its grid attached, and are corrected in step 4's write-up — *an instrument's own
docstring is not evidence about what it measured.*

| arm | measured | reading |
|---|---:|---|
| `_close_fuel`'s low-wall march-in (`m += step`) | **0 advances / 227 889** | DEAD on the CPG grids — **but see probe 4 (A)** |
| `g`'s off-map `AssertionError` | **0** | DEAD here, likewise |
| the `max(lo0, 0.02)` floor | `lo0` wins **227 889 / 227 889** | DEAD |
| the high wall `min(2.5, φ_max·n_L, hi0)` | literal **24 033** / map **200 193** / **`hi0` 3 663** | **ALL THREE arms live** |
| both of `integrate_fuel`'s `except AssertionError: break` | **0 truncations / 162** | DEAD |
| `equilibrium_fuel`'s damper `min(1.0, …)` | **0 binds / 8 Newton steps** | DEAD |
| its `max(…, 1e-30)` floor | **0** | DEAD |
| its `_EQ_MAX` exhaustion → `raise` | **0** | DEAD |
| `der`'s `caps` list | **`{0: 227856}`** | see below |
| `_interp`'s three arms | low **12** / interior **2 420** / high **2 752** | all live; its fall-through `return ys[-1]` **0** |

**THE HIGH WALL'S THIRD ARM IS THE MEASUREMENT THAT MOVED WHEN THE GRID DID.** On a first,
narrower sweep `hi0` won **0** times and the honest-looking write-up would have been "rung 43 adds
a third arm to rung 40's two-way `min` and it is dead". Widened to the suites' actual `r` and `ρ`
lists it wins **3 663**. *A census is a property of the grid; slice R said so about `illinois_
exhausted` and it cost nothing there because the two grids were reported apart. Here the same
mistake would have shipped a wrong verdict about live code.*

**AND STEP 4 WIDENED THE SAME LESSON ONE MORE TURN: THIS PARAGRAPH'S GRID IS STILL NOT THE
SUITES'.** `probe_s2.py`'s header calls its sweep *"rungs 43 and 45's OWN grids"*; it is a
cross-product of its own choosing. On the grids the two suites really run, the three arms are
**1 398 / 228 801 / 1 210**, not 24 033 / 200 193 / 3 663 — and both rare arms live in ONE cell,
rung 45's `hp-only` shape, whose LP map is `flat()` so the `phi_max` ceiling never binds. See
step 4's write-up below.

**AND `der` BUILDS ZERO CAPS, 227 856 TIMES OUT OF 227 856.** Neither suite arms a single limiter
keyword — grepped, then counted: the only keyword either file passes to any fuel entry point is
`freeze="lp"`. So `_topping_fuel`, `_sched_fuel`, `_surge_fuel`, `faded`, both dispatch twins and
all three helper classes are **unreached by every gate in phase 6**. § 5.12's *"not one `der` had
two live caps, 0 contested selections out of ~600 000"* was measured over 78 deliberately armed
cases and is **entirely off this slice's grid**; it is quoted here with that grid attached and
never merged. The contested-`min` question belongs to slices T and U, and is booked to them.
Gating those counters against zero on S's grid is therefore **vacuous by construction**, and the
armed smoke sections are what covers them — which is why this slice has five steps.

**THE HEADLINE — `int(round(s_end/ds))` LANDS ON AN EXACT `.5` TIE, AND PYTHON AND RUST ROUND IT
THE OPPOSITE WAY.** Rung 43's ramps run `s_end = r + 8.0` at `ds = 0.02`, and at `r = 0.25` that
is `8.25 / 0.02 = 412.5` **exactly** (checked as a `Fraction`: `825/2`). Python's zero-digit
`round` is half-to-EVEN and gives **412**; Rust's `f64::round` is half-AWAY-FROM-ZERO and gives
**413** — a whole extra marched step. Measured across every march the two suites run: **21 of 162
land on an exact tie** *(on `probe_s2.py`'s grid; step 4 re-measured it on the suites' own and got
**52 of 143** — 36 % rather than 13 %)*, 141 on an exact integer, and **not one is inexact** (so `round` and a
truncation never disagree here — the naive test for this hazard reports agreement on precisely the
case that matters, which is how the first version of the counter got it wrong).

**THIS IS THE THIRD VERDICT ON ONE EXPRESSION, AND IT VINDICATES A DEFENSIVE SPELLING.** Slice Q
measured the tie **unreachable** on rung 37's grid and spelled `round_ties_even` on principle;
slice R measured the **truncation** live at rung 40 (`1.2/0.05 = 23.999…`) and shipped
`two_spool_transient.rs:862` as `(s_end / ds).round_ties_even() as i64`. Rung 43 is the first grid
where the two SPELLINGS THEMSELVES disagree, on the `r` value **five of the eleven rung-43 gates
use**. So slice S adds no decision here — it inherits slice Q's, and the finding is that a
spelling chosen when nothing could see it is now load-bearing 47 rungs later. *Registered as an
injection to MEASURE at step 1 (swap `round_ties_even` for `round`), not to reason about.*

---

**PROBE 3 — THE SHIPPED SOURCE'S OWN RESIDUAL-FLOOR CLAIM IS A CPG STATEMENT READ AS A GENERAL
ONE, AND ON A TPG GAS THE SOLVE IS A BIT-AMPLIFIER.** `equilibrium_fuel` carries no noise-floor
acceptance branch, unlike rung 40's `equilibrium`, and says why:

> *"No noise-floor acceptance is needed here … the fuel path REFUSES an equilibrium gas outright
> (`_tt4_from_f`), so this loop only ever runs on the non-equilibrium gases, whose residual floor
> is ~1e-14 — comfortably under the absolute `_EQ_TOL`."*

Measured at six `Tt4` on each admitted gas:

| gas | accepted `\|Φ\|` range | margin under `_EQ_TOL = 1e-12` | passes |
|---|---|---|---|
| CPG | 3.2e-16 … 1.4e-13 | **7× … 3 161×** | 2 everywhere |
| `thermally_perfect` | 1.2e-13 … **9.29e-13** | **1.1× … 8.5×** | 2 … **32** |
| `reacting` | 1.0e-13 … 7.3e-13 | **1.0× … 9.7×** | 2 … 19 |

The "~1e-14" is the CPG number. On the TPG gases the floor is **9.3e-13 — 65× worse, and 8 %
under the bar it is called "comfortably under".** No cell exhausts `_EQ_MAX`, so the claim's
CONCLUSION survives; its stated reason does not. *Slice L step 4's shape, and slice R probe 3's —
a claim in the SHIPPED source that does not hold where a later reader points it — here for the
third time, and for the first time inside the very method that makes it.*

**AND THE CPython ARM TURNS THAT MARGIN INTO A 16-FOLD SWING IN THE ITERATION COUNT.** The same
six cells, both interpreters:

| gas | cells whose PASS COUNT differs | example |
|---|---:|---|
| CPG | **0 of 6** | 2 passes on both, every cell |
| `thermally_perfect` | **6 of 6** | `Tt4 = 1400`: PyPy **2** passes, CPython **33** |
| `reacting` | **6 of 6** | `Tt4 = 1450`: PyPy **19**, CPython **9** |

The mechanism is slice R probe 4's, on a different object: an exit decided just under an ABSOLUTE
bar by a residual that plateaus, so a last-bit difference re-rolls how many passes it takes to
squeak under. **This is registered as an ARCHITECTURE fact, not a tolerance problem.** It does not
threaten Rust-vs-PyPy bit-equality — the port's whole premise is that Rust reproduces PyPy's
arithmetic bit-for-bit, and where it does the pass count is reproduced with it. What it means is
that a TPG `equilibrium_fuel` key is the sharpest single **detector** in this slice (one ULP
anywhere upstream does not drift it, it explodes it 16-fold) and simultaneously **unusable inside
the CPython arm's bit-equality bar**. So: dumped and gated at bit-equality against PyPy; excluded
from the CPython bar as a declared fragile set with its deviation published, exactly as slice R
published its four-class split. Decided here by a six-cell spike costing minutes, on slice R
step 4's precedent, rather than after 500 lines of dump.

**AND `collapse_exponent`'s ARGMIN SITS ON A PLATEAU, SO ITS TIE-BREAK DECIDES THE REPORTED
NUMBER.** The method scores 25 exponents `q = i/20` by a binned spread and takes `min`. Measured
on gate 9's own 16 points: the score is **piecewise-constant in `q`** (13 distinct bin-fill shapes
over 25 samples), and every currency's minimum is attained by **two adjacent `q`, at a gap of
exactly `0.000e+00`**:

| currency | best `q` | tied `q` | spread |
|---|---:|---|---:|
| `E_temp_H` | 0.05 | **{0.05, 0.10}** | 0.137845 |
| `X` | 0.35 | **{0.35, 0.40}** | 0.141613 |
| `E_temp_L` | 0.65 | **{0.65, 0.70}** | 0.247224 |

Python's `min` keeps the FIRST of equals; Rust's `Iterator::min_by` also keeps the first — but
`max_by` keeps the LAST, and the two are one keystroke apart. **And gate 9 cannot see the
difference**: it asserts `qH < qX < qL` and `qL − qH > 0.3`, which a last-of-equals tie-break
satisfies just as well (0.10 < 0.40 < 0.70, gap 0.60). *Slice R's `best`-tie-break shape exactly —
invisible to the rung gates, visible only to the value dump — and here it is measured BEFORE the
port rather than at step 4.* No NaN score occurs on this grid (0 of 75), so the `9e9` NaN guard in
the `key=` lambda and the `if sp else nan` fall-back are both DEAD, and both are spelled.

---

**PROBE 4 (A) — THE REFUSAL IS SWALLOWED BY THE LOOP PROBE 2 MEASURED DEAD, AND THE WRONG ERROR
ESCAPES.** `test_rung43.py:317` tests the refusal by calling `_tt4_from_f(700.0, 0.025)`
**directly**, so it says nothing about reaching it through an ordinary entry point. Traced:

| entry point, on `Gas.reacting_equilibrium()` | what escapes | refusals swallowed inside `_close_fuel` |
|---|---|---:|
| `_tt4_from_f` (the gate's own call) | **REFUSAL** — *"needs the forward burner … non-equilibrium gas"* | 0 |
| `_instant_fuel` | **BRACKET** — *"does not bracket … off the modeled speed-line region"* | **46 caught, 38 of them the refusal** |
| `equilibrium_fuel` | **BRACKET**, same message | **46 caught, 38 of them the refusal** |

`_tt4_from_f`'s assert fires inside `ev`, inside `g`, and `_close_fuel` wraps `g(m)` in
`except AssertionError: m += step; continue`. So every refusal is eaten, the march-in loop walks
`0.04` at a time to the wall, and the bracket assertion is what the caller sees — naming a cause
("off the modeled speed-line region") that is not the actual one. Two consequences, both for the
port:

- **The march-in loop's `{0: 227 889}` is a property of the CPG grid.** On the one input class
  rung 43's own suite names, it runs **46 iterations**. It is a dead arm and a hot loop in the
  same file, and the census must say which grid each number came from.
- **The Rust must reproduce the swallow.** If the refusal becomes a `panic!` or an error variant
  the march-in arm does not catch, the ordinary entry points return the wrong error — a difference
  no value key can see, because on these grids they return no value at all. Gated structurally.

**PROBE 4 (B) — ALL SEVEN LIMITER KEYWORDS EXECUTE, AND THE TWO DISPATCH TWINS EMIT DIFFERENT KEY
SETS.** Nothing had ever armed `tau_gov`, `s_off`, `tau_rel` or `lag` — ~150 source lines with no
measurement at all. Armed, one case per rung, on a 21-point march:

| armed | route | clipped points | `_release_weight` calls: `w=1` / `0<w<1` / `w=0` | per-point keys |
|---|---|---:|---|---|
| bare (43/45) | `integrate_fuel` | 0 | 84 / 0 / 0 | 14 |
| `Tt4_max` (46) | same | 16 | 84 / 0 / 0 | 14 |
| `+ tau_gov` (47) | `_integrate_fuel_lagged` | 16 | 84 / 0 / 0 | **14** |
| `accel` (48) | `integrate_fuel` | 19 | 84 / 0 / 0 | 14 |
| `surge` (49) | same | 9 | 84 / 0 / 0 | 14 |
| `+ s_off` (50) | same | 7 | **33 / 0 / 51** | 14 |
| `+ tau_rel` (51) | same | 9 | **33 / 22 / 29** | 14 |
| `lag` (52) | `_integrate_fuel_asym` | 19 | **0 / 0 / 0** | **16: `+g`, `+required`** |

Three things this settles before any code: `_release_weight` is called **unconditionally** in
`der` and short-circuits to `1.0` when `s_off is None` (84 of 84 in six of the nine cases), so the
rung-49/50 reduce really is one branch; rung 51's fade is the only case with **all three** weight
arms live; and **`_integrate_fuel_asym` returns a 16-key point where every other route returns
14**, so the dump must enumerate keys PER ROUTE (slice O's three-variant-enum decision, arriving
on a trajectory record instead of a return dict). Rung 47's twin, by contrast, adds no key at all
— its third state `g` is not recorded, which is worth knowing because rung 52's is.

---

**SLICE R's PREDICTION 2 IS CORRECTED HERE, AND THE CORRECTION IS GOOD NEWS.** It registered that
`TwoSpoolFuelTransient` overrides **none** of `_close` / `_instant_tail` / `_powers`, so slice R's
`Hooks` table "ships with zero cells exercised inside phase 6". The override half is confirmed —
this class defines `_close_fuel` and `_instant_fuel`, not the three hook names. But **overridden
and exercised are different claims**: `_instant_fuel` calls `self._instant_tail`, which IS one of
`R40`'s three cells, on this slice's hot path (227 856 `der` calls reach it). Grepped for the other
two: `_close_fuel` replaces `_close` rather than calling it, and `equilibrium_fuel` runs its own
2-D Newton rather than calling `_powers` — so slice S dispatches through **exactly one of the
three cells, and exactly one**. That retires slice R's "a table nothing yet uses" caveat instead of
carrying it forward, and it turns prediction 3's manufactured-failure gate into a real one driven
from slice S's own call path.

**THE DEFERRAL DUE HERE.** `rung55.rs` item 5 —
`test_cycle_untouched_transient_ladder_is_bit_for_bit_unstacked` — was booked to this slice by
§ 5.12's inbox because it needs `TwoSpoolFuelTransient` to exist. Discharged at step 3, with the
roster line removed. **And `stage.rs:870`'s doc comment must move with it**: it currently reads
*"`TwoSpoolFuelTransient` does not exist in Rust yet"*, which stops being true at step 1 — slice O's
lesson is that the note which reaches the next slice is the one written where the compiler and
tests hit it, and this is the same note going stale in place.

---

**THE TEN PREDICTIONS, registered before any Rust is written:**

1. The oracle comes back **100 % bit-exact against PyPy** on the CPG arm. The TPG
   `equilibrium_fuel` keys are dumped as a SEPARATE arm and are the slice's one genuine exposure
   (probe 3): the pass count is a 16-fold amplifier of a single bit, so a divergence shows up as a
   discrete disagreement rather than a last-bit one. If it fails, § 9 Decision 1 Option B is the
   route and the CPG arm keeps bit-equality — decided now, and spiked at step 1 on two cells
   before the dump is written, on slice R step 4's precedent.
2. The CPython arm is **CPG-bit-exact and TPG-fragile**, with the fragile half published as a
   deviation distribution and NEVER summed with the CPG half. Prediction: the CPG half moves
   **0** float keys and the TPG half moves the exit pass count in **≥ 10 of 12** cells.
3. Slice S dispatches through **exactly one** of `R40`'s three cells (`try_instant_tail`), and the
   manufactured-failure gate proves it BOTH ways: swapping that cell moves slice S's values, and
   swapping either of the other two moves **nothing** in this slice.
4. Replacing `round_ties_even` with `f64::round` in the inherited `integrate` lengthens **21 of
   162** marches by exactly one point, and **the trajectory-LENGTH keys and the census are the
   ONLY things that see it — every reported value is blind.** *The first draft of this prediction
   said the opposite* — that at least one extremum would move, "because rung 43's
   `E_temp_H`/`E_temp_L` are maxima over the whole trajectory rather than over a saturating tail"
   — which is slice R's prediction 8 reasoning verbatim, and slice R's prediction 8 died of it.
   So it was MEASURED before it shipped (`probe_s5.py`, the step count forced to 413 and 414 over
   3 shapes × 2 `ρ` × the tie case and a non-tie control): **`Tt4_peak`, `X`, `E_temp_H`,
   `E_temp_L` and `complete` are bit-identical in 12 of 12 cells.** The reason is structural and
   is why no rung-43 grid could ever see it: `s_settle = 8.0` against a ramp of `r ≤ 2.0` makes
   95 %+ of every march settling tail, and **the peak is attained at point 13 of 412** — 3 % in,
   at the instant the ramp ends. A naive `f64::round` would therefore give the RIGHT answer to
   every rung-43 gate and only the length would betray it, which is exactly why the length is an
   oracle key. *Registered as measured, on the plan's own precedent, rather than as reasoning that
   has already been refuted once.*
5. The four dead arms of probe 2 (march-in on the CPG grid, `g`'s off-map assert, the `lo0` floor,
   both truncation arms) and the three dead arms of `equilibrium_fuel`'s Newton stay at **0**
   across the whole CPG dump — and the march-in counter is **non-zero and equal to 46** on the
   equilibrium-gas refusal section, so the same counter is gated against zero and against a
   measured number in one dump. **The counter therefore ships INSIDE `fuel_transient.rs` from
   step 1, not bolted on at step 4**: `_close_fuel` SWALLOWS the refusal, and a wrapper cannot
   count what the body catches — the same argument slice R's dump made for its three swallowed
   counters, which cost it two `src/` edits at step 4 and broke a two-step test-only streak. One
   line, no arithmetic line touched, and `git diff` shows that directly.
6. The high wall's three arms come back **24 033 / 200 193 / 3 663** on the dump's grid, and are
   gated as a three-way split whose total is the asserted call count — never as a sum
   (slice R step 2's cost).
7. `collapse_exponent`'s argmin ties at **{0.05,0.10}, {0.35,0.40}, {0.65,0.70}** and the
   first-of-equals tie-break is gated by the reported `q` keys, since probe 3 measured gate 9
   blind to it. Injecting a last-of-equals fold moves **3** value keys and **0** rung gates.
8. Reaching the refusal through `_instant_fuel` / `equilibrium_fuel` raises the **BRACKET** error,
   not the refusal, after swallowing **38** refusals in **46** march-in iterations — reproduced in
   Rust and gated on the error's IDENTITY, because no value key exists on that path.
9. `_integrate_fuel_asym` emits **16** per-point keys where every other route emits **14**, and
   the dump enumerates Python's key set per route and asserts each count.
10. The 20 Python gates port to **≥ 20** Rust gates, as a `--list` name diff with **0 removals**,
    plus the `rung55.rs` roster discharge.

**PORT DECISIONS, REGISTERED.**

- **A new `src/fuel_transient.rs`**, composing over rung 40's `TwoSpoolTransientCore` through a
  `pub inner`, as `combustor.rs` composes over `SpoolTransient` and `two_spool_transient.rs` over
  `TwoSpoolMapCore`. `two_spool_transient.rs` is already 1 271 lines and this is a different
  control (fuel imposed, `Tt4` an output, seven limiter arms).
- **The refusal is a fallible path whose error the march-in arm CATCHES**, not a panic — probe
  4 (A). The two error kinds are distinct variants so the gate can assert which one escapes.
- **`round_ties_even` is INHERITED, not re-decided** — `two_spool_transient.rs:862` already spells
  it, and probe 2 makes it load-bearing here for the first time.
- **The trajectory point is an enum or a route-tagged record**, because `_integrate_fuel_asym`
  carries two fields no other route does (probe 4 (B)) — slice O's *"a Rust struct with `Option`
  fields would let a caller read a field Python would have raised `KeyError` on"*.
- **`_release_weight` is a free function**, called unconditionally, short-circuiting to `1.0` — the
  spelling that keeps rungs 49 and 50 on one branch.
- **The three helper classes are plain frozen records**; `AccelSchedule.cap`'s clamped linear
  interpolation and `_interp` are two SEPARATE functions in Python and stay two in Rust, because
  their fall-through arms differ and probe 3 measured `_interp`'s dead.
- **The two suites' gases are built by each suite's OWN expression** — `286.9` in `rung43.rs`,
  `(γ−1)/γ·cp` in `rung45.rs` — with the bits asserted in the dump's section A, on the measured
  grounds that only a thrust key can ever witness a mistake here.
- **Both `slow` markers**: neither suite carries one (checked — `grep pytest.mark` is empty in
  both), so slice M's rule needs no measurement to give no `#[ignore]`, but the armed smoke
  sections are new cost and are timed at step 1.

**THE FIVE STEPS.**

| # | step | gate |
|---|---|---|
| 1 | `src/fuel_transient.rs` + `oracle/dump_slice_s_smoke.py` + `tests/slice_s_smoke.rs` + `tests/slice_s_dispatch.rs`, with the injections of predictions 3, 4, 7 and 8 MEASURED rather than reasoned | smoke bit-exact; name diff, 0 removals |

**THREE THINGS SETTLED ABOUT STEP 1 BEFORE IT STARTS.** (i) The armed limiter cases are **nine
SECTIONS OF THE SAME DUMP**, not a second file — probe 4 (B) already fixes the nine and their clip
counts, and the 16-key `_integrate_fuel_asym` route has its key set **enumerated and asserted per
route**, because a field missing from that route would otherwise be missing from the comparison
too (slice R step 1's closing finding). (ii) Prediction 5's march-in counter goes in `src/` on day
one, per its own text. (iii) Prediction 3's gate asserts that swapping two of `R40`'s three cells
moves **nothing**, and *a gate whose expected result is "nothing" passes when the swap silently
fails to take*: so `try_instant_tail` is perturbed FIRST and the harness watched to report
movement, before either zero is trusted. Slice R step 3 paid for this exact thing — an injection
harness whose revert preserved mtimes reported three defects as carry-over from the row above.
| 2 | `tests/rung43.rs` — the 11 collected items | 11 run / 0 failed |
| 3 | `tests/rung45.rs` — the 9 collected items; `rung55.rs` item 5 discharged and `stage.rs:870`'s stale note corrected | **SHIPPED: 10 run / 0 failed** (9 + the discharge); roster line gone |
| 4 | `oracle/dump_fuel_transient.py` + `tests/fuel_transient_oracle.rs`, PyPy + CPython arms, the TPG arm carrying probe 3's measured expectation | **SHIPPED: 5 run / 0 failed**; 4 671 + 1 133 keys bit-exact, TPG fragile set published |
| 5 | docs — the rung-43/45 specs' *What the RUST PORT measured*, this section's corrections, and the § 5.12 IOU marked discharged | **SHIPPED**, in step 4's commit; docs-only, so no gate is owed |

---

##### STEP 1 — SHIPPED. **THIS SECTION'S OWN TABLE LOST A ROW ITS PROSE COUNTED, AND ITS 46 IS TWO ARMS**

`src/fuel_transient.rs` (2 225 lines, against 1 103 of Python), `oracle/dump_slice_s_smoke.py`, `tests/slice_s_smoke.rs`
and `tests/slice_s_dispatch.rs`. The smoke is **5 536 values bit-exact against PyPy**; the
name diff is **713 → 715, 0 removals**; both new binaries carry ONE `#[test]` each, because the
counters are thread-locals.

**FINDING 1 — PROBE 4 (B)'s TABLE HAS EIGHT ROWS AND ITS OWN PROSE SAYS NINE.** The paragraph
reads *"84 of 84 in six of the nine cases"* against a table listing bare / `Tt4_max` / `+tau_gov`
/ `accel` / `surge` / `+s_off` / `+tau_rel` / `lag` — eight, of which five are 84/84. Recovered
from `probe_s4.py` rather than reconstructed: the missing row is **`ALL (46+47+48+49)`**, 19
clipped, and it is the ONLY case routing through `_integrate_fuel_lagged` **with both min-select
legs armed** — i.e. the only one exercising that twin's `faded` (which references `mf_sched`,
where the bare marcher's same-named closure references the applied `mf`) beside a sequential,
unfiltered min-select. An eight-section dump would have under-delivered prediction 9's own gate.

**AND THE TEN CASES STILL DO NOT CONTEST A `min`.** Measured after the fact, from a gate that
failed: the eight single-leg cases build at most ONE cap, and the composite routes to the LAGGED
twin, whose min-select is sequential and builds no `caps` list at all — so `der_caps_2` and
`der_caps_3` are **zero across all nine**. The bare marcher's `caps.retain(|c| c < mf)` / `min` is
the one place in the whole family where two legs contend for the same actuator, and probe 2
measured it building zero caps **227 856 times out of 227 856** on both suites' full grids. So the
dump ships a **TENTH** case — the composite with `tau_gov` dropped, which puts all three legs on
the bare route — and it is the only coverage that machinery has anywhere in the project.

**FINDING 2 — THE 46 SWALLOWED REFUSALS ARE 38 + 8, AND THE 8 COME FROM A CALL SITE SLICE L
MEASURED AT ZERO.** This section recorded *"46 caught, 38 of them the refusal"* as one number and
left the other 8 unnamed. Measured (`probe_s6.py`, the SHIPPED body instrumented by textual
substitution): they are **`inverse: root not bracketed`, every one out of the HPC ideal-temperature
inversion** `T_from_h_c(h25 + eta_hpc*(h3 - h25))` at `engine.py:4591`, over a contiguous band of
trial flows `m_lp ∈ 1.739…2.019` where the HP face has run past `psi < 0` and the ideal enthalpy
rise goes negative. **None is the off-map/NaN guard**, which stays dead even here.

That matters beyond the census. § 5.8 recorded `t_from_h` reaching `solve` and firing **0** times,
and kept its `assert!` on that measurement — true of the call sites that existed then. Rung 43's
closure scans from the low wall to a THREE-arm high wall and reaches the band; rung 40's identical
line never does, because its march-in breaks at the FIRST success and its wall has two arms. So
`gas.rs` gains **`try_t_from_h_c`** (plus the twins beneath it), added along exactly the chain
measured to fail and nowhere else — *slice L step 1's rule, one phase on: fallibility is per CALL
SITE, not per function.* The two arms are **two counters**, never a summed 46: *a registered SUM is
not a gated SPLIT.*

**FINDING 3 — PREDICTION 3 IS TRUE OF THE FUEL PATH AND FALSE OF THE OBJECT, and the gate that
said otherwise was unearned twice over.** It registered that swapping either of rung 40's other two
hook cells "moves **nothing** in this slice". Measured: on a PURE fuel-path probe both are never
CALLED (the closure is replaced, and `equilibrium_fuel` runs its own Newton), which is a stronger
statement than "moved nothing" and is what `slice_s_dispatch.rs` now asserts. But
`ramp_excursion_fuel` builds its two fuel ENDPOINTS with `fuel_for_Tt4` and its running line with
nine more `equilibrium` calls, every one of them rung 40's `Tt4`-control path — so on the OBJECT
all three cells are live and both swaps move the answer.

The first version of that gate reported a **1-ULP** difference and read as agreement. Two reasons,
both instrument defects: it probed the object (so rung 40's Newton simply re-converged to the same
root), and it perturbed `powers` by a **SCALE**, which cannot move a root the solver drives to
zero. A third: the `try_close` injection scaled `pi_lpc`, which **nothing downstream reads**, so it
moved nothing anywhere and every "nothing moved" beside it was vacuous. *Slice R step 3's lesson
generalises — an injection harness has to be shown to express itself before any zero it reports
means anything, and "the swap took" is not the same claim as "the swap could be seen".*

**FINDING 4 — THE THIRD HIGH-WALL ARM IS DEAD ON THE SMOKE'S OWN GRID, and a partition sum cannot
see that.** `min(2.5, phi_max*n_L, hi0)` is the closure's most prominent departure from rung 40's
two-arm wall, and the first draft gated it with `literal + map + hi0 == close_calls` — which passes
identically whether `hi0` binds 3 663 times or is ABSENT FROM THE SOURCE. Every one of sections
A–K reported `hi_wall_hi0 = 0`. Located rather than assumed: `hi0` beats `min(2.5, phi_max*n_L) =
2.1098` only for `mdot_fuel < 0.008439`, i.e. below `Tt4 ≈ 930` on this running line. Section **L**
now drives the closure at `Tt4 = 900/800` (the arm binds, the closure returns a state) and at
`700/650` (the arm binds and the bracket then FAILS) — which also makes it the only CPG cell in the
file reaching the `does not bracket` assert every other section gates against zero. *Slice M's
shape: a bar asserted in a comment and refuted by measurement.*

**FINDING 5 — `collapse_exponent`'s PLATEAU IS A PROPERTY OF THE GRID, and the first grid had
none.** A cheaper grid (2 shapes × 3 `rho` × 3 `r` at `s_settle = 2.0`) reported `tied = 0` on all
three currencies, so a tie-break gate written against it **could not fire**. On gate 9's own
four-`rho` × four-`r` grid at the shipped settle the plateau is exactly as predicted — argmins at
**0.05 / 0.35 / 0.65**, each tied with its neighbour at a gap of `0.000e+00`, spreads
0.137845 / 0.141613 / 0.247224 reproduced to the bit — and `slice_s_dispatch.rs` builds the
last-of-equals fold and shows it moves the reported exponents while **gate 9's ordering, gap and
interior-exponent claims all still hold**. *Slice R's `illinois_exhausted` again: a census is a
property of the grid.*

**PREDICTION 1 SPIKED AND HELD, INCLUDING THE FRAGILE HALF.** Two TPG cells per gas before a line
of dump was written: `thermally_perfect` and `reacting` at `Tt4 = 1400/1450` come back
**bit-identical to PyPy, pass counts included — 3, 7, 3 and 32.** The 32-pass cell IS probe 3's
16-fold amplifier, so the sharpest detector in the slice fires on the port and agrees. § 9
Decision 1 Option B is not needed.

**PREDICTION 4 MEASURED, NOT REASONED.** `8.25/0.02` is exactly `412.5`; `round_ties_even` gives
412, `f64::round` 413, and `int()` gives 412 — *so the naive test for this hazard reports agreement
on precisely the case that matters.* Two marches one point apart agree **pointwise** on every
field, the peak is attained at point 13 of 413, and only `npts` moves. **PREDICTION 8 HELD** —
`_instant_fuel` and `equilibrium_fuel` raise the BRACKET error while the direct poke raises the
REFUSAL.

**TWO INSTRUMENT DEFECTS THE DUMP CAUGHT IN ITSELF.** The Rust reset its counters one statement too
early and hid **39 Illinois calls** that Python's `E/bare` section legitimately carries (the two
`fuel_for_Tt4` calls and the `equilibrium` that build the ramp sit inside that census window — *the
census keys are sensitive to statement POSITION between `emit_census` boundaries*); and
`EQ_PASSES` counted residual EVALUATIONS where Python's recovery counts passes COMPLETED, an
off-by-one-per-call the pass-count keys reported directly.

**DECIDED HERE RATHER THAN AT STEP 3: the degenerate path's EIGHT refusals live on the ENUM.**
Python's `integrate_fuel` opens with seven asserts on an `lp_disabled` object and
`_fuel_ramp_march` with an eighth; none can live on `FuelTransientCore`, which is never degenerate
by construction. `TwoSpoolFuelTransient` gains `integrate_fuel_lp_disabled` (the seven, then EXACT
dispatch — the `nu0` type changes across it, which is why it is a separate method) and enum-level
`phi_excursion_fuel` / `transient_surge_margin_fuel` (the eighth). All eight are gated NOW, beside
a BARE call asserted to be ADMITTED — *a gate whose expected result is a raise passes when
everything raises.* Both rung-45 methods take Python's four keywords rather than the whole limiter
set, because passing `s_off`/`lag`/`freeze` there is a `TypeError` in the source.

**AND SIX MORE COUNTERS SHIPPED WITH NO GATE AT ALL — finding 4, five times over.**
`topping_skips`, `topping_exhausted`, `sched_skips`, `surge_skips` and `mf_floor_hits` are exactly
the dead-arm population the project's rule says is *ported and COUNTED rather than left absent*,
and the first draft ported them without counting: a port that deleted either set-point loop's
`continue` arm, or spelled the `1e-9` fuel floor as `1e-8`, went through step 1 clean. All five
measure ZERO on every section and are now asserted; `close_g_evals` — a raw count, not a branch —
gets a structural FLOOR (`>= 2x` the successful closures, since the scan must find a negative and
then a positive) rather than being left as a number nothing reads. **`assert_dead_arms` covering
*some* of a file's counters is the same defect as a partition sum covering an arm.**

**`stage.rs:870` CORRECTED HERE, not at step 3.** Its doc comment said
*"`TwoSpoolFuelTransient` does not exist in Rust yet"*, which stops being true at step 1 — this
section said so in advance and cited slice O for why the note that reaches the next slice is the
one written where the compiler and tests hit it. The `rung55.rs` roster item still discharges at
step 3; only the falsehood is fixed now.

**STILL OWED TO STEP 3:** `test_rung45.py` builds its `lp_disabled` object from a **two-spool**
design engine where `test_rung43.py` builds one from `build_turbojet` — Python is duck-typed there
and Rust is not, so `rung45.rs` needs that resolved. And the degenerate `equilibrium_fuel` FORWARD
is the one method family in the enum group still routed around: Python's forwards to the held
single-spool object while **silently discarding a `start` the caller passed**, and section K
reaches that object directly, so the forward is not exercised AS a forward and the
argument-dropping is unported. No gate reaches it today; decide it with the rest of the enum
layer. **[STEP 2 took the `equilibrium_fuel` half of this — see below; only the `lp_disabled`
CONSTRUCTION difference is still owed.]**

---

##### STEP 2 — SHIPPED. **THE SOURCE'S ONE ρ-CONTRAST ASSERTION IS SATISFIED BY ρ-INERTNESS**

`tests/rung43.rs`, **11 `#[test]`, 11 run / 0 failed, 1.16 s** — the collected count step 2 was
sized against, and 10 gates plus the scope concession, which is a test function and not a gate. The
`--list` name diff is **715 → 726, 11 additions, 0 removals** (prediction 10's currency, half paid;
`rung45.rs`'s 9 land at step 3). One `src/` edit, **+21 lines and purely additive**:
`TwoSpoolFuelTransient::equilibrium_fuel_lp_disabled`.
Step 1 booked the degenerate `equilibrium_fuel` forward to "the rest of the enum layer"; gate 2
calls it, so it came due one step earlier than written. It carries the argument drop in its doc
comment rather than in a nicer signature — **Python's forward accepts a `start` and never passes
it on**, so the parameter is present and `_`-bound, because a Rust signature without it would make
the drop invisible at the call site.

**FINDING 1 — THE ONE ASSERTION ABOUT ρ IN GATE 5 IS SATISFIED BY REMOVING ρ FROM THE PLANT.**
Python's gate 5 closes with `ratios == sorted(ratios, reverse=True)` on `d_lp/d_hp` across
`rho ∈ {0.5, 1, 2}` — a NON-STRICT ordering on the quantity whose entire subject is that the share
MOVES. Measured, not reasoned: deleting the `/rho` from the LP ODE makes the three ratios
**bit-identical** (`0.7944667304020582` three times) and Python's assertion passes on that, while
gates 6, 7 and 9 all fail. The shipped ratios fall `1.426 → 0.794 → 0.419` (flow/press, `r=0.25`)
and `1.384 → 0.774 → 0.409` (tilted, `r=0.25`), tightest adjacent pair `1.005 → 0.728` — a 28 %
drop — so `rung43.rs` asserts the STRICT `>` beside Python's non-strict one and says in place that
it is stronger than the source, with the margin that licenses it. *Slice M's shape: a bar the
source states and does not measure.*

**FINDING 2 — SEVEN OF THE ELEVEN GATES ARE BLIND TO A WRONG LP DERIVATIVE.** Three injections,
each shown to express itself before any zero beside it was read:

| injection | what it breaks | gates that fire |
|---|---|---|
| drop `/rho` on the LP ODE | ρ leaves the plant | **5** (strict arm only), 6, 7, 9 |
| LP ODE reads `Phi_hp` | the LP spool marches on the wrong residual | **gate 4 ALONE** |
| the two `freeze` arms swapped | `freeze="lp"` holds the HP spool | 5, 6 |

The middle row is the measurement worth keeping. A gross physics defect — the LP shaft integrating
the HP power residual — is invisible to every FINDING gate in the file, because they assert signs,
orderings and a monotonicity that a wrong-but-similar derivative still satisfies. The only thing
that catches it is the DYNAMICAL reduce (gate 4), which demands the march land on the independently
solved equilibrium. *A suite of sign claims needs one gate that asserts a NUMBER; rung 43 has
exactly one, and it is carrying the file.*

**FINDING 3 — AN INJECTION APPLIED BY LINE NUMBER AFTER A `src/` EDIT HIT A DIFFERENT STATEMENT
AND REPORTED ALL-GREEN.** The first three injections were `sed '<n>s|…|…|'` against line numbers
read before the +21-line addition above; the re-run of injection A landed on
`let (k3a, k3b, _, _) =`, changed nothing, and the suite came back **11 passed** — which reads as
"gate 5's new strict arm does not fire" and is in fact "the injection was never applied". Caught by
the `sed -n '<n>p'` echo, not by the result. Every injection after it is a text substitution with
its own `assert s.count(old) == 1`. *Slice R step 3's lesson, second instance inside one slice: an
injection harness has to be shown to express itself before any zero it reports means anything —
and a line number is not a target once the file underneath it has moved.*

**FINDING 4 — ONE AXIS OF PYTHON'S GATE 2 IS UNREPRESENTABLE IN RUST, AND IT WAS DELETED AT RUNG
40 RATHER THAN HERE.** Python builds the degenerate object with BOTH maps (`map_lp=LP_SHAPED,
map_hp=HP_SHAPED`) and its `__init__` selects `map_hp`; gate 2 is what proves the selection picked
the right one of the two. **Every `lp_disabled` constructor in the port takes only `map_hp`** —
`TwoSpoolMapMatcher`'s (slice K), `TwoSpoolTransient`'s (slice R) and this slice's — so "the
constructor held the wrong one of the two maps" cannot fail by construction, in `rung43.rs` OR in
`rung40.rs`, and no slice had recorded it. The port-wide convention is not re-opened for one gate;
what step 2 owes is the honest width of what survives, and that was MEASURED, not asserted: the
CALLER's choice IS live — passing `lp_shaped()` at the call site fails the gate — but it fails by
exactly **1 ULP on `nu` at `Tt4 = 1500`**, the first and thinnest of the gate's three cells, because
the design point is where the running line barely reads the map. *The trap is `rust-port-ported-test-vacuity`'s:
a better factorisation turns the source's real pin into something closer to a self-comparison.*

**FINDING 5 — GATE 3's MUTATION CHANNEL IS SEVERED BY THE CLONE, and the first draft's comment
argued the wrong thing.** Python's gate 3 hands ONE design object to both the rung-40 transient and
the fuel transient precisely so that a MUTATION of the design by the fuel path would surface on the
rung-40 side. `rung43.rs` clones, and the first draft justified that with "the build is a pure
function of its arguments" — true, and an argument for EQUIVALENCE where the source's gate is about
SHARING. The file already carried the honest version of this sentence for gate 10 ("Rust's
`run(&self)` makes that channel hard to open in the first place, so the gate is thinner here than in
Python"); gate 3 now carries it too. No code change — but a green tick must not imply the Python
claim.

**WHAT THE FILE DELIBERATELY DOES NOT CLAIM.** Gate 9 reproduces the Python gate and no more: § 5.16
probe 3 measured it blind to `collapse_exponent`'s first-of-equals tie-break (the argmins tie with
their neighbours at a gap of exactly zero, and `0.10 < 0.40 < 0.70` satisfies every assertion just
as `0.05 < 0.35 < 0.65` does). The tie-break is pinned in `slice_s_dispatch.rs`, and the header of
`rung43.rs` says so rather than letting a green tick imply the stronger claim.

---

##### STEP 3 — SHIPPED. **AN INJECTION THAT COMPILED, APPLIED, AND COULD NOT HAVE MOVED ANYTHING**

`tests/rung45.rs`, **10 `#[test]`, 10 run / 0 failed, 0.26 s** — the 9 collected items plus rung
55's roster item 5. The `--list` name diff is **726 → 736, 10 additions, 0 removals**, which is
prediction 10's currency paid in full (step 2 paid the 11, this pays the 9 and the discharge). Two
`src/` edits, **both doc comments only**: `stage.rs`'s note and `rung55.rs`'s roster entry. No
production line moved at this step.

**THE `lp_disabled` CONSTRUCTION DIFFERENCE, OWED SINCE STEP 1, IS SETTLED — AND THE REASON IS
SHARPER THAN "PYTHON IS DUCK-TYPED".** Step 1 booked that `test_rung45.py` builds its degenerate
object from a **two-spool** design engine where `test_rung43.py` builds one from `build_turbojet`.
Measured (`probe_s7`, `probe_s7c`): **rung 45's own `SINGLE` dict has no `nozzle_convergent`**, so
`build_turbojet(**SINGLE)` is REFUSED by the rung-31 matcher outright — the source could not have
fed a single-spool engine without inventing a recipe it does not contain, and feeding the two-spool
one is the only route it had. What that produces is a HYBRID, because `OffDesignMatcher.__init__`
takes the LAST `Compressor` and the LAST `Turbine` off the roster:

| | from the TWO-spool engine | from a single-spool one |
|---|---:|---:|
| `eta_t` | 0.9 (the **LPT**'s) | 0.92 |
| `pi_c_design` | 6.0 (the **HPC**'s) | 6.0 |
| `tau_c_d` | 2.4806 (the `2→3` span, **both** compressors) | 1.7597 |
| `A4` / `A8` | 7.508e-4 / 2.276e-3 | 2.264e-3 / 3.872e-3 |
| `P_ref` | 425 338 | 218 230 |
| `pi_c` at `Tt4 = 1400` | **15.41** | **5.44** |

**None of it is READ.** `phi_excursion_fuel` refuses in the eighth assert opening
`_fuel_ramp_march`; `transient_surge_margin_fuel` reads only `self.map_lp/map_hp`, which on the
degenerate branch come from CONSTRUCTOR KWARGS. Measured: all three admissible feeds raise
**byte-identical** messages at both methods. So `rung45.rs` feeds rung 45's own `SINGLE` with
`nozzle_convergent: true` added, and that added flag is the one constant in the file that is not
the source's — disclosed in the header rather than passed off as the suite's.

**FINDING 1 — THE SOURCE ARMS ITS SECOND DEGENERATE OBJECT TO GET PAST A DIFFERENT ASSERT, AND
RUST REVERSES THE ORDER.** `test_rung45.py`'s `lp_disabled` gate builds TWO objects — the first
bare, the second `_floor(...)` on both maps — and asserts only `pytest.raises(AssertionError)` for
each. Measured (`probe_s7b`): `transient_surge_margin_fuel` reads its **surge-line** assert FIRST,
so on an UNARMED degenerate object it raises *"needs a surge line on BOTH maps"* and the two-shaft
refusal is never reached. The arming is what makes that half of the gate test the assert it names;
without it the gate would have passed having exercised a completely different one. *The project's
own "a gate whose expected result is a raise passes when everything raises" landing in the SOURCE,
not in the port.* So `rung45.rs` asserts WHICH refusal escapes (§ 5.16's registered port decision,
now measured to matter) — and injection M below shows that assertion has teeth.

Rust cannot reproduce the ORDER: the refusal lives on the enum, because the `Degenerate` variant
holds a `SpoolTransient` and no maps at all, so a bare-map `transient_surge_margin_fuel` raises the
TWO-SHAFT refusal where Python raises the SURGE-LINE one. Both inputs the source exercises agree.
The divergence is DISCLOSED in the gate's doc comment rather than repaired — repairing it means
carrying two maps on a variant with no use for them.

**FINDING 2 — THE FILE PASSED 10/10 FIRST TRY, SO THIRTEEN INJECTIONS WERE RUN, AND ONE OF THEM
WAS A DUD THAT LOOKED LIKE A HOLE.** Every injection is a text substitution with its own
`count(old) == 1` (step 2's finding 3), and every one was confirmed to COMPILE before its result
was read. The first draft of `H` seeded `ext_lp` with `map_lp.phi_surge * 1e-9` to break the
read-only claim; it applied, it compiled, it reported **zero gates firing** — and the first marched
point overwrites the seed, because the loop is `if e_lp.abs() > ext_lp.abs()`. **Compiling is not
expressing.** It is kept in the harness as row `H0`, beside the repaired `H` that perturbs the
RESULT and does fire gate 1. *Slice S step 1's own lesson, arriving two steps later inside the same
slice, on an instrument written by the person who wrote that lesson down.*

| injection | what it breaks | gates that FIRE |
|---|---|---|
| A `DTT4` spelled as an ENDPOINT | the rung-44 comparison spans 1000–2400 | gate 4 ALONE |
| B running line read at `s/2` | the COMMANDED reference is sampled wrong | gate 2 |
| C `min_phi` reads the other shaft | the surge object is the wrong spool's | gate 4 |
| D march bound loses its `r` | the settle window stops tracking the ramp | **NONE** |
| E `Tt4_peak` clipped to the command | the plant's `rho` signal is erased | gate 3(b) |
| F steady min IS the transient min | gate 6's flip compares a thing with itself | gate 6 |
| G crossing flags swapped | the crossing is reported on the wrong spool | gate 6 |
| H the RESULT reads `phi_surge` | READ-ONLY broken | gate 1/read-only |
| H0 the SEED reads `phi_surge` | **a DUD — overwritten at the first point** | **NONE** |
| I `/rho` deleted from the LP ODE | `rho` leaves the plant | gate 3(a) **and** 3(b) |
| K the fuel closure reads `stack_lp` | rung 55's scope violation, manufactured | rung-55 item 5 |
| L the ramp ignores its own rate | every `r` applies the same schedule | gate 5 |
| M the degenerate refusal's WORDING | it still fires, but stops saying which | gate 1/`lp_disabled` |
| Z `min` respelled as an `if` | NO-OP CONTROL | **NONE** |

Row A is the one worth keeping. § 5.16's port decisions never mentioned it, and the source writes
`phi_excursion(FLIGHT, 1000.0, 400.0)` beside `phi_excursion_fuel(FLIGHT, 1000.0, 1400.0)` — a
**DELTA** against an **ENDPOINT**. Porting the `400.0` as an endpoint is caught by exactly ONE of
the file's ten tests; every sign and ordering assertion in gate 2 survives it. It is now a named
constant, `DTT4 = HI - LO`, with the hazard in its doc comment.

**FINDING 3 — `D` IS NOT A HOLE IN THE SUITE; THE CHANGE IS PHYSICALLY INERT AND ONLY `npts`
WITNESSES IT.** Dropping `r` from `r + s_settle` fires nothing, so it was measured rather than
filed as a gap:

```
                   BASELINE                        INJECTED
r=1.0    npts=351  min_phi_lp=0.7511681337710964   npts=301  min_phi_lp=0.7511681337710964
r=0.5    npts=326  min_phi_lp=0.7354659814383082   npts=301  min_phi_lp=0.7354659814383082
r=0.3    npts=316  min_phi_lp=0.7188792876873181   npts=301  min_phi_lp=0.7188792876873181
r=0.1    npts=306  min_phi_lp=0.6726404680613678   npts=301  min_phi_lp=0.6726404680613678
```

**Bit-for-bit identical physics at all four ramp rates**, because the minimum is attained during the
ramp and never inside the settle tail the `r` buys. The ONLY channel that witnesses the march bound
is `npts`, and no rung-45 gate reads it. **STEP 4 OWES `npts` AS AN ORACLE KEY** — this is where a
value dump earns its keep over a suite of signs.

**FINDING 4 — THE SUITE IS NINE GATES OF SIGNS AND SPREADS, SO 64 VALUES WERE DIFFED BY HAND
BEFORE STEP 4 — AND THE FIRST 30 LEFT OUT THE GATE THAT NEEDED IT MOST.** Step 2's finding 2
measured that seven of eleven rung-43 gates are blind to a wrong LP derivative; rung 45 is
worse-placed, because it has NO dynamical reduce at all (its own docstring says the object is
anchored TRANSITIVELY through rung 43's gate 1). So `probe_s8` printed Python's numbers for gate 5's
`r` sweep, gate 3(b)'s `rho` sweep, gate 2's four shape ratios and gate 6's crossing cell — 30
values, **every one bit-identical to PyPy's `repr`.**

**The first pass skipped gate 3(a), which is the rung's own reason for existing and the most
bespoke computation in the file**: a hand-written 19-point interp, its own 325-step march per `rho`,
and three spreads gated only by `< 0.02`, `> 0.20` and an ordering — bars a wrong interp or a wrong
march endpoint lands comfortably inside. `probe_s9` covers it: the **19 grid values, the 3 march
point counts (326 each), the 9 excursions and the 3 spreads — 34 more, all bit-identical.**
`raw_min` 0.00632, `cmd_ext` 0.07861, `out_ext` 0.32860, against bars of `<0.02` / `>0.20` and the
strict ordering. *The argument that a sign-only suite needs a value diff applies hardest to the gate
the value diff leaves out.* None of this is the oracle — it is the go/no-go slice N step 5 says a
measuring pass needs before it is trusted, and it passed.

**WHAT NO INJECTION REACHES.** Thirteen injections gave nine of the ten tests teeth. The tenth,
gate 1/cycle, is unreachable from `fuel_transient.rs` at all — its channel is `engine.rs`'s design
run, and nothing in the fuel path can perturb a single-spool cycle. It carries the project-wide
rung-6 invariant and its teeth are every other file's. Recorded here and in the gate's own doc
comment rather than left to a 14-row table beside 10 tests: step 1's finding is that covering SOME
of a set is the same defect as a partition sum covering an arm.

**ONE DISCLOSED DIVERGENCE IN THE TEST HARNESS ITSELF.** `refusal()` swaps the GLOBAL panic hook to
silence the backtrace, where `rung44.rs` calls `catch_unwind` and leaves it alone. Two tests here
call it and cargo runs them on parallel threads, so the restore can race — it cannot change a
`catch_unwind` result, only interleave suppressed output. The silencing is what buys the ability to
assert WHICH refusal escaped, so the divergence is kept and named on the helper.

**STEP 3 SHIPPED NO PRODUCTION CODE, AND THAT IS A PROOF RATHER THAN AN IMPRESSION.**
`src/fuel_transient.rs` was written and reverted about fifteen times across three injection
harnesses. `git show --stat` on the step-3 commit does not list it; the two `src`-side files it does
list (`stage.rs`, `rung55.rs`) have every changed line beginning `///` or `//`. So nothing green at
step 2 can have moved at step 3, by construction — which is a cheaper and stronger statement than a
full-suite re-run, and the check that establishes it is one command.

**RUNG 55's ROSTER ITEM 5 IS DISCHARGED — BY REBUILDING IT, NOT PORTING IT.** Python's gate hands
ONE design object to a stacked matcher and a fuel transient so a WRITE by the former would surface
on the latter. `StageStackCoreSpec::new` and `TwoSpoolFuelTransient::new` each take a
`TwoSpoolEngine` **by value**, so that channel does not exist and `before == after` could not have
failed whatever the closures read — `rust-port-ported-test-vacuity`, and step 2's finding 5 is the
precedent for saying so rather than shipping the tick. The shipped gate INJECTS a live K=8
`StageStack` into the fuel transient's OWN `TwoSpoolMapCore::stack_lp`/`stack_hp` and demands the
march bit-identical, which asks the question the scope boundary is actually about — *would these
closures READ a stack sitting in the slot?* Injection K manufactures the failure and it fires. The
gate also shows its injection EXPRESSES ITSELF twice over (the slot is `Some` with `K == 8`, AND the
same stack moves `eta_lpc` on a matcher that does read it), because "nothing moved" beside an inert
stack would be worth nothing.

`stage.rs`'s note is corrected a SECOND time in two steps — step 1 killed *"does not exist in Rust
yet"*, and step 3 kills *"lands with `rung55.rs`'s roster item at slice S step 3"*, which stops
being true the moment this ships. That is the point of writing it where the compiler and the tests
hit it rather than only in this plan.

**STILL OWED TO STEP 4:** `npts` as an oracle key (finding 3), and both suites' `R_c`/`R_t` bits as a
section-A key beside at least one thrust key per suite (§ 5.16 probe 1's closing measurement — only
a thrust key can ever witness which gas the port used). **BOTH PAID — see below.**

---

##### STEP 4 — SHIPPED. **THIS SECTION'S CENSUS CAME OFF A GRID ITS OWN PROBE'S HEADER CALLED THE SUITES' AND WAS NOT**

`oracle/dump_fuel_transient.py` (three arms) and `tests/fuel_transient_oracle.rs` (five gates).
**4 671 values on the CPG arm and 1 133 on the gas arm, bit-exact against PyPy on the first run
that compared them**; the CPython arm is 5 804 values and is a DETECTOR, published below. The
`--list` name diff is **736 → 741, 5 additions, 0 removals** (prediction 10's currency, paid in
full: 11 at step 2, 10 at step 3, 5 here). **Zero `src/` edits** — prediction 5 put the march-in
counter in `fuel_transient.rs` at step 1 precisely so this step would need none, and it needed
none. Timing: **157 s IN THE FULL-SUITE RUN** (a solo `cargo test --test` of the same binary
reported 155 and 199 s across two runs — the in-suite figure is the one quoted, because the
`#[ignore]` rule is about the suite's cost and not the binary's), against a **~19 min** crate
total, `two_spool_oracle`'s 70 s and `slice_s_smoke`'s 36 s. So slice M's *"re-introduce
`#[ignore]` only against a MEASURED cost"* is measured and **not met** — no marker.

**THE HEADLINE — AN INSTRUMENT'S OWN DOCSTRING IS NOT EVIDENCE ABOUT WHAT IT MEASURED.**
`probe_s2.py`'s header says it runs *"rungs 43 and 45's OWN grids"*. It runs a cross-product of its
own choosing (3 shapes × 4 `rho` × 4 `r`, then 4 shapes × 3 `rho` × 5 ramps). Every census number
predictions 4 and 6 registered came off it, so this step wrote the grid the two suites actually
sweep and read the census out BEFORE a gate was written — on the advisor's insistence and on this
slice's own record (three census-is-a-property-of-the-grid corrections already: `illinois_exhausted`,
`collapse_exponent`'s ties, `hi_wall_hi0`). **Four registered numbers died and one held:**

| quantity | § 5.16 registered | MEASURED on the suites' grids |
|---|---|---|
| `integrate_fuel` calls | 162 | **143** (140 in the dump; gate 10's 3 fold into section F) |
| …on the `412.5` tie | 21 of 162 | **52 of 143** — 36 %, not 13 % |
| high wall `literal / map / hi0` | 24 033 / 200 193 / 3 663 | **1 398 / 228 801 / 1 210** |

The right-hand column is both suites' full sweeps. The dump's own cells give
**1 398 / 223 890 / 1 210** over 140 marches — it folds rung 43 gate 10's `freeze_channels` call
into section F rather than repeating it, which costs three marches and 4 911 map-arm calls and no
literal or `hi0` ones. Both are written down, because quoting either for the other is the mistake
this whole table is about.
| CPG float keys moving under CPython (pred. 2) | **0** | **15** |
| TPG cells moving the exit pass count (pred. 2) | >= 10 of 12 | **12 of 12** — HELD |

**FINDING 1 — THE HIGH WALL'S TWO RARE ARMS ARE REACHED BY EXACTLY ONE CELL OF THE TWENTY GATES,
AND IT IS A SHAPE.** All 1 398 literal hits and all 1 210 `hi0` hits come from `test_rung45.py`'s
**`hp-only`** shape, whose LP map is `ComponentMap.flat()`: a flat map has no `phi_max` ceiling, so
`2.5` binds on the accel (1 301 of 1 304) and the decel's low fuel drops `hi0` under both (1 207 of
1 304). Step 1 finding 4 had to ADD section L to the smoke to reach `hi0` at all and concluded the
arm binds only below `Tt4 ~ 930`; that was true of its grid and is not the general statement — the
arm is reached at `Tt4 = 1000…1400` whenever the LP map is flat. The census is therefore emitted
**per cell** in section L rather than per section, because a section total would let one shape's
228 801 map hits bury another's 1 301 literal ones. A dedicated gate asserts the localization both
ways: the other three shapes take the map arm on **every** call.

**FINDING 2 — PREDICTION 2's "0 CPG FLOAT KEYS" IS REFUTED, AND THE REFUTATION NAMES A CLASS.**
15 of 3 411 CPG float keys move CPython-vs-PyPy, worst **3.14e-16 relative — one ULP**, and every
one is `collapse_exponent`'s scored curve. The reason is not the plant: `spread(q)` is
`r / rho**q` and `log`, i.e. a composite of two LIBM calls, and libm is the one thing a port shares
with neither interpreter. **Everything that reaches the dump THROUGH THE PLANT is bit-identical.**
So a third exempt class had to be invented beside slice N's iteration counts — declared wider than
the measurement, because what makes a key exempt is BEING a fractional-power / log composite.

*And the tie survived, which is the part worth keeping.* Gate 9's argmin is a plateau at a gap of
exactly `0.000e+00`, so a one-ULP move on either tied score would BREAK the tie and hand the fold a
different exponent. `J/collapse/*/q` stays on the BIT bar and all three argmins agree across
interpreters — a measurement, not a tautology.

Slice N's other half also reappears: **1 of the 66 CPG-half iteration-count keys moves**, by one,
and it is a rung-44 `phi_excursion` call — the INHERITED class, not this slice's. Slice R found the
same shape (38 513 vs 38 512); third instance, and the class is exempted with its width printed.

**FINDING 3 — THE 46 SWALLOWED REFUSALS ARE A PROPERTY OF THE FUEL FLOW, NOT OF THE GAS.** Step 1
corrected § 5.16's "46" into "38 + 8"; this step corrects the split's SCOPE. Measured at three flows
spanning 15 %:

| `mdot_fuel` | advances | refusal | `inverse` | off-map |
|---|---:|---:|---:|---:|
| `f_eq*mdot_air_eq(1400 K)` = 0.019433 (the smoke's) | 46 | **38** | **8** | 0 |
| 0.020 | 46 | **39** | **7** | 0 |
| 0.017 | **47** | **40** | **7** | 0 |

The total barely moves; the split moves by two. What moves is where the band of trial flows that
runs the HP face past `psi < 0` starts relative to the march-in grid. Python and Rust agree cell for
cell — the divergence was in the BAR, which quoted one cell's split as the gas's. Each cell is gated
on its own numbers and the first is the smoke's own flow, so the 46/38/8 is REPRODUCED here
independently rather than quoted. *A census is a property of the grid, fourth instance in one slice.*

**FINDING 4 — A DISCLOSED PORT DIVERGENCE IN WHICH REFUSAL ESCAPES, FOUND BY ADDING A CELL NEITHER
SUITE HAS.** On an **unarmed** `lp_disabled` object Python's `transient_surge_margin_fuel` raises
the SURGE-LINE assert, not the two-shaft one: its body reads `self.map_lp/map_hp` and checks
`phi_surge` before `_fuel_ramp_march`'s refusal can fire. Rust raises the two-shaft one, and that is
not a bug — step 2 finding 4 recorded that EVERY `lp_disabled` constructor in the port takes
`map_hp` ALONE, so the degenerate variant has no `map_lp` to read a surge line off and must refuse
on degeneracy first. `test_rung45.py`'s own degenerate gate arms both maps, so nothing was hidden.
The dump records Python's answer under a `pyonly` key the comparator names and skips (count
asserted, so a third cannot join), and a Rust-side gate asserts the port's answer — **the divergence
has a gate on both sides rather than a comment on one.**

**TWO VACUITY TRAPS IN THIS STEP'S OWN INSTRUMENTS, BOTH CAUGHT BEFORE SHIPPING.**
(i) The localization gate ran the whole sweep and then dropped its comparator without calling
`finish()`, so every value diff AND the never-compared half were silently discarded — step 1
finding 3 and step 2 finding 3's shape, third instance in this slice, in a file written by the
person who wrote those down. (ii) Section F's length check recomputed the expected point count
locally as `round_ties_even((r + s_settle)/ds) + 1` and compared THAT to the marcher's counter —
which is the very expression a naive-`round` port gets wrong, so the two sides moved in lockstep and
the assertion could not fail. F now passes `None`; its lengths are gated by `census/F/march_points`
against Python's counter and by nothing local. The honest general statement is that
`march_points == sum of npts` checks the COUNTER against what the caller receives, and the ROUNDING
gate is the `npts` keys against Python.

**ONE DUMP-INSTRUMENT DECISION, NOT A PORT CHANGE.** Python's `eq_calls` wrapper sits on the class
method, which the DEGENERATE object enters before forwarding to rung 35's own solve; Rust's counter
lives inside `FuelTransientCore::try_equilibrium_fuel`, which that forward never reaches. Counting
the forward would have made section C's `eq_calls` a gate that fails on a correct port, so the dump
stops counting it. Checked rather than assumed that this cannot collide with step 1's golden: the
smoke emits **no** census section containing a degenerate `equilibrium_fuel` call
(`census/L/eq_calls` is 0), so the two dumps never define the counter differently.

**WHAT THE ORACLE ADDS OVER THE SUITES.** Every ramp cell carries `i_peak` and the peak point itself
beside the seven reported fields — prediction 4 measured the peak attained at point 13 of 413, so
all seven are decided by a handful of early points and a port whose interior drifted after them
would agree on every one. Every excursion cell carries `s_lp`/`s_hp`, `min_phi_hp` and `ratio`,
which nothing in either suite reads. Section A carries both recipes' `R_c`/`R_t` bits AND a thrust
key each. And the **gas arm is added, not ported**: neither suite runs a TPG gas through the fuel
path, and probe 3's 16-fold pass-count amplifier is the sharpest single detector in the slice —
`eq_passes` totals **157** over the three admitted gases, gated as a number because a bit-exact
value dump cannot see an iteration count.

**STEP 5 SHIPPED IN THE SAME COMMIT** — the two rung specs' *What the RUST PORT measured*, this
section's five corrections (including forward-pointers written at probe 2's own claims, where the
next reader hits them rather than only here), and § 5.12's CPython-detector IOU marked CLOSED.
`pytest` is not owed: the only Python written is `rust/oracle/*.py`, which `pytest.ini`'s
`testpaths = tests` excludes — checked, not assumed.

### 5.17 SLICE T (rungs 46 + 47 + 48, the GATES on the topping governor and the `Wf/pt3` leg) — SHIPPED, four steps; pre-registered off four probes MEASURED first

Rung 46 is the first fuel-side FEEDBACK (clip the metered fuel to hold `Tt4 ≤ Tt4_max`), rung 47
gives that governor a response LAG, and rung 48 puts the FEEDFORWARD `Wf/pt3` schedule beside it.
§ 5.12 settled that these do not partition as classes — they are keyword arms on ONE method — so
slice S shipped `integrate_fuel` **entire** and this slice is **gates only**, plus the four reader
methods those gates call.

**THIS IS THE FIRST SLICE WHOSE SOURCE ALREADY SHIPPED, AND THAT INVERTS WHAT A PRE-REGISTRATION
IS FOR.** Every prior slice registered predictions about a port about to be written. Here the plant
landed at slice S, whose own probe 2 measured that **~40 % of it is limiter machinery no phase-6
rung gate reaches**. Slice T's 31 gates are therefore the FIRST coverage on `try_topping_fuel`,
`integrate_fuel_lagged`, `AccelSchedule::cap` and `try_sched_fuel` — and slice S's 801-key /
3 411-key bit-exact oracle says nothing about whether those legs are RIGHT. That is
*an oracle cannot see a missing gate*, pointed at slice S instead of at Python. **The registered
expectation is that this slice FINDS DEFECTS IN SHIPPED SLICE-S CODE**, not that it confirms a
port. A failing gate here is the slice working.

**THE SCOPE IS SETTLED BY GREP, NOT BY THE PHASE TABLE'S PHRASE.** § 5.16 left the thirteen
rung-46–52 reader methods (`engine.py:5437–6054`) unported and named two of them as slice U's
inheritance. Each of the thirteen was grepped across every suite and `main.py`; the split is clean
and **slice T owns exactly four**:

| reader | line | read by | slice |
|---|---:|---|---|
| `topping_relief` | 5437 | `test_rung46`, `test_rung47`, `main.py` | **T** |
| `topping_command_trace` | 5480 | `test_rung47`, `main.py` | **T** |
| `schedule_relief` | 5501 | `test_rung48` | **T** |
| `engagement_sweep` | 5554 | `test_rung48`, `main.py` | **T** |
| `surge_relief` … `factorization_grid` (9) | 5579–5948 | `test_rung49…52` | U |

**COUNTED, NOT TAKEN FROM A HEADER.** `pytest --collect-only`: **31 items = 6 + 9 + 16**, and all
three counts coincide again — no `parametrize` in any of the three files and `def test` count ==
item count in each. **2 of the 31 carry `slow`**, both rung 46's
(`test_governor_holds_and_the_surge_relief_split`, `test_the_lever_fast_ramp_switches_on_lp_relief`);
rungs 47 and 48 carry none, so § 5.12's "nine slow tests in rungs 37/40/46/52" is met here by the
46 pair alone. Sizing: **~142 source lines** over **984 test lines** — the mirror image of slice S's
1 103 / 689, and the first slice of the port where the gates outweigh the code by 7×.

**THE ORDER IS FORCED, NOT CHOSEN.** Rung 48's `test_reduce_two_leg_composite_min_select` arms the
governor beside the schedule leg, so 46 and 47 land before 48.

---

#### The four probes — `probe_t1.py`, `probe_t2.py` (`M:\claud_projects\temp\rust-phase6\`)

Run on the SUITES' OWN grids (slice S step 4's correction: a probe's header claimed the suites'
grids and its code ran another — every constant below is copied from `tests/test_rung46-48.py`).

**FINDING 1 — THE HOOKS TABLE HAS THREE CELLS AND § 5.12'S CENSUS NAMED SIX CROSSING NAMES.**
`TwoSpoolTransientHooks` carries `try_close`, `try_instant_tail`, `powers`. The phase-6 pre-flight
measured **six** names called on `self` inside phase 6 and overridden in phase 7 — the other three
are **`integrate_fuel`** (overridden by **11** phase-7 classes), **`_close_fuel`** (4) and
**`_surge_fuel`** (1). Slice S ported all three as inherent methods with **no hook cell and no note
anywhere in `fuel_transient.rs`** — grepped case-insensitively for `phase[ -]7` and `defer`, **0 hits**. (The first pass of that grep also asked for `owed` and got a hit; it was the substring inside *swallowed*. Recorded because a scope claim resting on a grep is only as good as the pattern — the port has spent a slice on that lesson before.) This is
precisely the shape slice O's lesson is about: *write the deferral where the next slice's compiler
and tests will hit it, not only into a paragraph* — what reached slice O was a panic with a
backtrace, and the paragraph that had predicted it correctly was read second. **Slice T writes that
note at the three method definitions.** It does not build the cells: phase 7 is unauthorised, and a
hook with no second implementation is an unmeasured guess at what phase 7 needs.

**FINDING 2 — EVERY DISCRETE DECISION IN THE FOUR READERS IS UNCONTESTED BY FIVE TO SEVEN ORDERS.**
The advisor's concern was that the four readers are thin, so the risk concentrates in the non-float
outputs a bit-exact dump is blind to (slice J: *exactness bounds the CELLS visited, not the RULES
discriminated*). Measured over rung 46/47's full grid — 4 map shapes × {r=0.5 / redline 1480,
r=0.15 / redline 1440} × {`tau_gov` None, 0.05, 0.1, 0.2, 0.4, 0.8} = **48 cells** — and rung 48's
6-margin sweep:

| decision | bar | nearest INSIDE | nearest OUTSIDE | slack |
|---|---|---:|---:|---:|
| `held` (`Tt4_peak ≤ Tt4_max + 1e-6`) | `1e-6` | `9.09e-13` | `54.7` K | **5.5e7 ×** |
| engaged (`abs(Tt4−Tt4_max) < 1e-6`) | `1e-6` | `9.09e-13` | `1.064` | **1.06e6 ×** |
| clipped (`mf < mf_sched·(1−1e-9)`) | `1e-9` | `0.0` exactly | `3.73e-4` | **3.7e5 ×** |
| `monotone_nondecreasing` | `−1e-12` | — | step `8.88e-05` | **8.9e7 ×** |

The governor either pins the redline to **machine zero** (`|overshoot| ≤ 1.6e-12`, 8 of 48 cells,
every one of them `tau_gov=None`) or misses it by **54.7–430 K**; there is no middle. Likewise a
point is either clipped by ≥ 3.7e-4 relative or by **exactly zero**. **So the port's risk is not in
the decisions** — which is worth registering, because it means a decision that DOES flip in Rust is
a port defect and never a knife-edge.

**FINDING 3 — RUNG 48'S SIX `== 0.0` ASSERTIONS ARE EXACT BY CONSTRUCTION, NOT BY CANCELLATION.**
`test_rung48.py` asserts `relief_lp == 0.0` — an exact float equality on a difference of two
independently-marched minima — at six sites. Measured WHY: for the downstream margins the bare and
limited marches are **bit-identical for their first 14 / 16 / 20 points** (m = 0.42 / 0.45 / 0.48),
and the LP argmin sits at **index 12**, inside every one of those prefixes. The two `min` calls
therefore read the *same float from the same point*, so the difference is `0.0` for the same reason
`x − x` is. **The six assertions are one claim, not six** — that the port's engagement point matches
Python's — and they will pass or fail as a block.

**FINDING 4 — THE `s_eng = NaN` ARM IS REACHABLE AND DEAD ON THE SUITES' GRID.** `schedule_relief`
returns `eng[0] if eng else float("nan")`. Swept: `n_engaged` reaches **0 at m ≥ 0.55** (r=0.5), so
the arm is live code. But the lowest `n_engaged` any suite cell produces is **1** (rung 48 gate 12's
m=0.78 at r=0.15) — every one of the six `MARGINS` engages, 6 to 103 points. **So the NaN arm ships
ungated unless the oracle adds a cell**, which is slice Q's lesson exactly: *a dead arm is a
property of the GRID, not the code*. Registered as an ADDED cell, not a ported one.
*The comparator needs no change, and that was measured rather than assumed:* PyPy's `float("nan")`
and Rust's `f64::NAN` are both `7ff8000000000000`, so the existing `to_bits()` compare handles the
key.

**FINDING 5 — GATE 12'S "COINCIDENT MINIMA" ARE THE SAME POINT, NOT TWO POINTS AT EQUAL `s`.**
`test_fast_ramp_single_crossing_when_the_minima_coincide` asserts `abs(s_lp − s_hp) < 1e-9`.
Measured at r=0.15: both minima fall on **index 7** of the same trajectory and `|s_lp − s_hp|` is
**`0.0` exactly**. The 1e-9 bar is not measuring a near-coincidence. Neither array has a tie
(`phi_lp` gap to second-smallest `3.72e-3`, `phi_hp` `6.02e-4`), and none exists at r=0.5 either
(`9.03e-5` / `1.61e-5`). **So Python's first-on-tie `min` rule is never exercised by any suite
cell** — a `<=` spelling in the Rust argmin fold would ship undetected. That gate must be
MANUFACTURED (slice Q again), not hoped for.

**FINDING 6 — A DOCSTRING NUMBER IS STALE IN RUNG 47. ~~AND IN RUNG 46~~ — CORRECTED BY STEP 4,
READ THAT BEFORE CITING THIS.** `test_rung46.py` sites the redline "below the **~1645** bare peak";
`test_rung47.py` says "**~1670**". Measured bare `Tt4` peak, all four map shapes, both settles:
**1690.5 / 1695.4 / 1702.4 / 1703.0**.

**THE CONCLUSION DRAWN FROM THAT ROW WAS HALF WRONG, AND STEP 4 MEASURED WHY — THE GRID HERE WAS
NOT "BYTE-IDENTICAL", IT WAS A DIFFERENT GAS.** Those four numbers are the **CPG** row, which is
rung 47's and rung 48's gas. Rung 46's gates 3-6 run `Gas.thermally_perfect()`, whose peaks are
**1641.4 / 1644.9 / 1650.9 / 1651.2 K** — so `~1645` is CORRECT to 0.008 % on the shape its own
gate 6 runs, and only `~1670` is stale (by 21-33 K). The two files were never describing the same
march, which is exactly why they disagreed; the tell was real and the diagnosis inverted which file
was at fault. **Do not cite this finding's original reading — the doc correction shipped at step 4
fixes rung 47 and leaves rung 46's figure standing with its gas named.** No gate reads either
number (the redline 1480 clears every peak by >200 K), so it was a doc correction throughout.

It is still the `rung 63` lesson — *check a quoted number was taken at THIS rung's settings* — with
"settings" now measured to include the GAS.

**WHAT THE DOCSTRINGS GOT RIGHT, MEASURED SO THE CORRECTION IS NOT MISTAKEN FOR A SWEEP.** `s_lp*`
0.24 ✓, `s_hp*` 0.40 ✓, rung 47's `~55→191 K` (measured 55.6→190.9) ✓ and `~220→390 K`
(218.9→388.5) ✓, rung 48's `+0.0034` at m=0.45 (0.003385) ✓ and `+0.0075` while `relief_lp` is
already 0 (0.007493 at m=0.42) ✓. **Six of seven quoted numbers hold; the one that does not is the
one both files quote.**

**FINDING 7 — THE `m → 0` CORNER COMPLETES, IT DOES NOT REFUSE.** Rung 48 gate 11 reads the
disclosed degenerate boundary. Measured at m = 0.02 / 0.05 / 0.10: the march runs to the end every
time, with `nu_hp_end` falling `1.40e-1` / `8.66e-2` / `1.92e-3` below bare and the `Tt4` peak
de-fanged to 1084 / 1244 / 1460 K against 1695 K. **So no refusal-ordering divergence is expected
here** — unlike slice S step 3's `lp_disabled` finding, which needed a `pyonly` key. Registered as a
prediction so a Rust-side refusal is read as a defect.

---

#### The pre-registered predictions

**P1 — the slice finds ≥ 1 defect in shipped slice-S limiter code.** The four legs these gates first
reach are `try_topping_fuel`, `integrate_fuel_lagged`, `AccelSchedule::cap`, `try_sched_fuel`.
Registered as the EXPECTED outcome; if all 31 gates pass first try, that is the surprise and the
instruments get audited before the result is believed (slice N step 2's lesson — *the dump passed
first try and taught nothing*).

**P2 — the six `relief_lp == 0.0` gates pass or fail as ONE.** Finding 3's mechanism says they read
the same float twice. A split outcome would REFUTE the mechanism and is the more interesting result.

**P3 — no decision key flips.** Finding 2 measured 5–7 orders of slack on all four bars. A flip is a
port defect, not a knife-edge, and will be treated as one.

**P4 — the argmin fold needs strict `<` and no suite cell tests it.** Finding 5. A manufactured
tie-cell is written; predicted that `<=` passes all 31 ported gates without it.

**P5 — the `held` / `overshoot` split is `tau_gov`-clean.** All 8 `held=True` cells are
`tau_gov=None`; all 40 lagged cells miss. Predicted the Rust port reproduces the split exactly, so
`held` needs no tolerance of its own.

---

#### The steps

| step | content |
|---|---|
| **1** | ✅ the four readers + rung 46's 6 gates; the phase-7 deferral note at the three un-hooked methods (finding 1) |
| **2** | ✅ rung 47's 9 gates (the lagged governor + the command trace) |
| **3** | ✅ rung 48's 16 gates (the schedule leg, the crossing, the degenerate boundary) |
| **4** | ✅ the oracle: the added NaN cell (finding 4), the manufactured tie cell (finding 5), the CPython arm; the two rung specs' *What the RUST PORT measured*; finding 6's doc correction — **plus the lagged route on the SUITES' cells** (step 2) **and `fuel_removed`'s VALUE** (step 3), the two quantities this slice measured that no gate in either language can hold |

`#[ignore]` on the two `slow` gates is decided by slice M's rule against a MEASURED in-suite cost
against the crate total — the criterion the previous commit pinned — and is not pre-judged here.

##### STEP 1 — SHIPPED. **A PYTHON-CATCHABLE `assert` PANICS IN RUST, AND ONE GATE OF SIX CARRIES THE SIGN**

`fuel_transient.rs` gains the four readers (`topping_relief`, `topping_command_trace`,
`schedule_relief`, `engagement_sweep`), their three return structs, and the phase-7 deferral notes
finding 1 booked; `tests/rung46.rs` is **7 test fns for 6 Python ones** — the seventh is not a port,
see below. **327 lines added to the source, 0 changed**: the readers are pure additions over slice
S's plant, which is what "gates only" was supposed to mean and is worth stating as a measurement
rather than an intention. **`rung46.rs` is 7/7 in 10.0–11.8 s** (three runs, the spread quoted rather than the fastest), and `cargo build --release` is clean.
**THE FULL-CRATE FIGURE WAS NOT IN THAT COMMIT, AND THAT WAS THE HONEST STATE**: the run was still
in flight when the step was committed, so the commit quotes no crate number. An earlier draft of this
paragraph said *"the crate is green"* BEFORE the run finished — the same class of slip as the
`#[ignore]` range that excluded its own most recent measurement, and corrected the same way: quote
what was measured and leave a gap where nothing was. **THE GAP IS NOW FILLED BY A MEASUREMENT, NOT
CLOSED BY AN ASSUMPTION**: the run finished **exit 0 — 685 tests over 70 binaries, 0 failed, 0
ignored**, summing **606.6 s (10.1 min) of TEST EXECUTION** across the binaries. That sum is run time
only and **excludes the release build**, which is why it is not the ~19 min figure this slice has been
quoting for a cold gate — the two measure different things and neither replaces the other. Four
binaries carry **72%** of the execution (144.2 s, 129.3 s, 106.7 s, 58.0 s), so `rung46.rs`'s 10 s
is 1.7% of it and § 5.17's `#[ignore]` verdict is unchanged.

**FINDING 1 — AN `assert` PYTHON CATCHES, RUST CANNOT, FOUND BY ADDING A CELL NO SUITE HAS.**
Rung 46 gate 2 checks that an `lp_disabled` object REFUSES the governor. Python leaves the
`Tt4_max=None` route open on that same object, so the obvious neighbouring cell is to run it — and
Python returns an **empty trajectory** where Rust **PANICS**. Both are the same assert:
`_sonic_throat`'s CPG bracket check fires at the very first marched point (the gate's `mf = 0.5` is
~25× design fuel). Python's `integrate_fuel` wraps its instant call in `except AssertionError` and
`break`s on it; Rust's `march` breaks on an `Err`, but `components::sonic_throat` raises a
`panic!`, which unwinds straight past the whole fallible chain.

**THE DIVERGENCE IS A CLASS, NOT A CELL, AND IT WAS MEASURED RATHER THAN ESTIMATED.**
`choked_mfp` / `sonic_throat` have **28 call sites** in the crate. Classifying them by whether the
enclosing `fn` name starts with `try_` gives **10** — `eval_m_fuel` (`spool.rs:1041`, the route this
cell takes) among them. **That is a LOWER BOUND from a name grep and not a measurement of the
fallible set**, checked to be wrong in both directions: `map.rs`'s `operating_point` and
`two_spool_transient.rs`'s `r40_try_close` are both in fallible chains and neither matches the
prefix. Whoever owns the repair classifies by SIGNATURE. It is written as a floor rather than a
count because this port has shipped five typed count bars and all five were wrong. Each site is
**one line** from being faithful once a fallible twin exists, and converting one **cannot change
behaviour on any path where the assert does not fire** — the two spellings differ only in what
happens when it does.

**NOT FIXED HERE, AND THE REASON IS SCOPE, NOT DIFFICULTY.** The repair edits shipped phase-2/4/5/6
code across six files inside a slice whose entire content is gates, and a PARTIAL conversion would
be worse than none: it would leave some paths refusing and some panicking with no principle
separating them, which is harder to reason about than today's uniform "asserts panic". So it is
**booked as an OPEN item** and the divergence is given a gate on BOTH sides —
`disclosed_divergence_a_python_catchable_assert_panics_in_rust` asserts the escaping panic is still
the bracket assert, with an `expect` message telling the next reader to **DELETE the test**, not
update it, once the twins land. Slice S step 3 finding 4's precedent, one slice on.

**FINDING 2 — THE GATES WERE MEASURED, NOT ASSUMED, AND ONE OF SIX CARRIES THE SIGN.** All 7 passed
first run, which § 5.17 P1 registered as the outcome to distrust (*slice N step 2: the dump passed
first try and taught nothing*). Three defects injected into the shipped reader:

| injection | caught by |
|---|---|
| `relief_lp` sign flipped (`bare − top`) | **gate 6 ALONE** — 1 of 7 |
| `held`'s bar inverted (`+1e-6` → `−1e-6`) | gates 3+4+5 alone — 1 of 7 |
| the BARE march armed too (the differential destroyed) | gates 3+4+5 **and** 6 — 2 of 7 |

**The first row is the finding, and a FOURTH injection sharpens it into a mechanism.** Gates 3+4+5
assert `|relief_lp| < 1e-9` at moderate `r`, where the TPG re-measurement below puts `relief_lp` at
**exactly `0.0`** — and a sign flip on an exact zero is invisible. Flipping `relief_hp` instead, whose
bar is `> 1e-6` against measured values of 2.7e-3 to 3.6e-3, is caught by **two** gates. **So the
blindness is not the suite's, it is the LP half's specifically, and the exact zero IS the
mechanism**: the same measurement that makes rung 46's headline sharp — a machine-zero the port
reproduces bit-for-bit — is what makes that half of it unable to gate its own sign. The one gate
that can is gate 6, the fast-ramp lever, which exists to make a different point and is one of the
two `slow`-marked. A reason to keep it unmarked. Same shape as slice S step 2's *non-strict
ordering gate satisfied by deleting its own variable*.

**THE GATE GRID IS NOT THE PROBE GRID, CHECKED BEFORE THE FILE WAS WRITTEN.** § 5.17's probes ran
the CPG gas; rung 46's gates 3-6 run `Gas.thermally_perfect()`. Re-measured on TPG before porting:
`held` slack **4.1e-12 to 8.6e-12** against the `1e-6` bar, `relief_lp` **exactly `0.0`** at all
four shapes, and the mechanism ordering `1373.66 < 1480 < 1558.47` with 106 K / 78 K to spare. Same
shape as CPG, so no finding moves — but *a census is a property of the grid*, and this port has
paid for assuming otherwise.

**`#[ignore]`: MEASURED AND NOT EARNED.** `test_rung46.py` marks two of its six `slow`. The ported
file runs **10.04 s in full**, against a crate total in the ~19 min range — slice M's rule wants a
MEASURED cost and this is not one, so neither gate carries a marker. Same verdict as slice S's, on
a figure two orders smaller.

**ONE PORT DECISION WORTH NAMING.** Python reaches gate 2's first refusal two calls below
`topping_relief`, inside `phi_excursion_fuel`; Rust refuses in `TwoSpoolFuelTransient::
topping_relief` itself, so the message names the method the caller actually invoked. The gate
asserts the two-shaft wording, which both spellings carry — the assertion is on the REASON, not on
the frame.

---

##### STEP 2 — SHIPPED. **THE SUITE IS A SIGN-AND-ORDERING SUITE: TWO DEFECTS MOVE 13 OF 18 VALUES BY UP TO 24% AND ALL NINE GATES PASS**

`tests/rung47.rs` is **9 test fns for 9 Python ones**, one to one, and **0 source lines added or
changed** — every method the file calls (`integrate_fuel_lagged`, the four readers) shipped at step
1, which is what "gates only" was supposed to mean at this step and is recorded as a measurement
rather than an intention. **9/9 in 0.47 s**, against `rung46.rs`'s 10.0–11.8 s; the 21× is the GAS,
not the gate count — rung 47 is CPG throughout where rung 46 switches gates 3-6 to
`Gas.thermally_perfect()`. `test_rung47.py` marks nothing `slow`, so there is no `#[ignore]`
question to decide.

**THE VALUES WERE DIFFED BEFORE THE GATES WERE BELIEVED, BECAUSE THE BARS ARE LOOSE.** All 9
passed first run, which P1 registered as the outcome to distrust. A temporary dump of the same 18
readings `probe_t3.py` measures reproduced **every one bit-for-bit at full `repr` precision** —
`0.017664563253391514` / `0.023341360739983497` for the command-trace ends, `1480.000000000001` and
`1695.4058398939349` for the two peaks, `0.005266991602928872` for the instantaneous HP rebate. So
the port is right; the question the injections then asked is what the GATES would have caught.

**FINDING 1 — THE SUITE HAS NO VALUE CONTENT, AND THAT IS STRUCTURAL RATHER THAN AN OVERSIGHT.**
Five defects injected into `integrate_fuel_lagged`, the method this file first covers:

| injection | caught by | moves |
|---|---|---|
| `dg` SIGN flipped (`(g − required)/tau`) | gates 6+7+8 — 3 of 9 | — |
| `dg` missing its `/tau_gov` | gates 7+8 — 2 of 9 | — |
| applied fuel `mf_sched + g` | gates 6+7+8 — 3 of 9 | — |
| **`required` reads the APPLIED fuel, not the SCHEDULE** | **NOTHING — 0 of 9** | **13 of 18 values; overshoot 137.46 → 156.72 (+14%), fast-ramp overshoot 218.9 → 271.8 (+24%), LP relief 1.51e-2 → 1.25e-2 (−17%)** |
| **RK4 `g` weight — a `2` dropped from `k3g`** | **NOTHING — 0 of 9** | **13 of 18 values; overshoot 137.46 → 147.83, LP relief 1.51e-2 → 1.35e-2** |

**THE TWO SURVIVORS WERE MEASURED TO MOVE SOMETHING BEFORE BEING CALLED A HOLE** — slice S step 3's
lesson (*an injection that compiled, applied, and could not have moved anything*), applied as a
precondition this time rather than discovered afterwards. They move 13 of the 18 dumped readings by
14–24%, and every gate still passes.

**THE REASON IS COUNTABLE.** Not one of the nine gates reads an absolute number. Four (1, 2, 4, 5)
are bit-identities between two runs of the SAME code; the other five are inequalities whose
measured margins are **137× / 4.5× / 27× / 8.9e7× / 2.19×**. The tightest bar in the file is gate
8's `overshoot > 100.0` against a measured 218.9, so **a defect must move a number by more than
2.19× to be seen** — and a 24% error is two orders inside that. The suite gates the REFUTATION
(a trailing-edge tool cannot reach an early minimum) and the COST (the hold breaks, the rebate
erodes), which are claims about signs and orderings; it was never built to gate arithmetic.

**FINDING 2 — THE ONE THING IN THE CRATE THAT DOES SEE THEM IS SLICE S's SMOKE ORACLE, AND THAT WAS
CHECKED RATHER THAN ASSUMED.** Both surviving injections were run against three other targets:

| target | I4 | I5 |
|---|---|---|
| `slice_s_smoke.rs` (its `r47` / `all` cases dump the lagged route) | **FAILED** | **FAILED** |
| `fuel_transient_oracle.rs` (4 671 + 1 133 keys) | ok | ok |
| `rung43.rs` / `rung45.rs` | ok | ok |

So the big oracle does **not** cover the lagged route — its grid never arms `tau_gov` — and the
whole of the crate's value coverage on `integrate_fuel_lagged` rests on the two smoke cells slice S
added for a different reason. This is *an oracle cannot see a missing gate* read in the other
direction: the gate cannot see a wrong value, and only one instrument in the crate can.

**WHAT STEP 4 OWES, AND IT IS NOW SPECIFIC RATHER THAN GENERAL.** Slice S's two lagged cells run
`Tt4_max = 1380`, `tau_gov = 0.2`, `ds = 0.05`, `s_end = 1.0`, one map pair. Rung 47's gates run
`REDLINE = 1480`, a **five-point `tau_gov` sweep**, **four map shapes**, `ds = 0.02`, and a
**second ramp rate** `r = 0.15` where the overshoot is 2–4× larger. The step-4 oracle takes the
SUITES' cells, not the smoke ones — § 5.17's own rule that the gate grid is not the probe grid,
now owed in a third direction.

**FINDING 3 — GATE 1's COMPARISON HALVES ARE VACUOUS IN RUST, AND THE FIX IS A LINE PYTHON DOES NOT
HAVE.** `test_reduce_tau_none_bit_for_bit_rung46` compares `integrate_fuel(…, Tt4_max=REDLINE)`
against `integrate_fuel(…, Tt4_max=REDLINE, tau_gov=None)` — two calls differing by a keyword whose
default IS the value passed. Python at least re-enters the function; Rust builds ONE `FuelLimiters`
value for both, so the trajectory loop is `x == x` and the `topping_relief` pair is one call
compared with itself. *A ported test can go VACUOUS* — named in the file rather than deleted, since
deleting it would hide that Python has a gate here.

**AND THE VACUITY LEAVES A REAL HOLE, WHICH IS WHY ONE LINE WAS ADDED.** If the `(Some, Some)`
dispatch were mis-spelled so a `tau_gov = None` call fell through to the BARE march, every
assertion Python writes in that gate would still pass. The added line asserts the applied fuel was
CLIPPED below the schedule at least once, which a bare march cannot do — measured far from a
knife-edge, since the bare peak is 1695.4 K against a topped 1480.0 K. Python's own last line
(`held`, `overshoot ≤ 1e-6`) is kept as the other live half, and it is the **CPG** witness of a hold
`test_rung46.py` gates on TPG only: measured `9.09e-13` against `1e-6`, a **1.1e6×** margin.

**FINDING 4 — THIS FILE CARRIES THE LP SIGN THAT RUNG 46's COULD NOT.** Step 1 finding 2 measured
rung 46's LP half blind to its own sign, because `relief_lp` is exactly `0.0` at moderate `r` and a
sign flip on an exact zero is invisible — only its fast-ramp gate carries it, one of the two marked
`slow`. Gate 8 here asserts `0.0 < relief_lp < prev` at four `tau_gov` values on a STRICTLY POSITIVE
quantity (measured `1.51e-2 → 1.03e-2 → 6.03e-3 → 3.17e-3`), so a sign flip fails four times over —
in a gate carrying no `slow` mark. Step 1's booked gap is partially closed by the NEXT rung's suite,
not by a repair.

**TWO VACUITIES IN PYTHON's OWN SPELLING, CARRIED RATHER THAN CORRECTED.** Gate 7 seeds
`prev_ov, prev_hp = -1.0, 1.0`, so its first of five comparisons is satisfied by the seed
(`55.59 > -1.0`, `3.56e-3 < 1.0`) and the content is the four later strict steps — measured
`+39.7 / +42.2 / +32.7 / +20.7` K and `-6.87e-4 / -7.38e-4 / -7.26e-4 / -5.50e-4`. Gate 9's
`monotone_nondecreasing` is vacuously true on ≤ 1 engaged points, so it is load-bearing only beside
`n_engaged > 10` — measured **45**, spanning `s = 0.30 … 1.18`, smallest step `8.88e-5` against a
`−1e-12` bar. Both are written out in the file's doc comments; changing either would be a different
test.

**§ 5.17 FINDING 6 RE-MEASURED AT THIS FILE's OWN CELL.** `test_rung47.py` sites the bare peak at
`~1670` and `test_rung46.py` at `~1645`; gate 1's cell reads `1695.4058398939349`, matching the
probe's `flow/press` figure exactly. The doc correction stays booked to step 4; it is carried in
`rung47.rs`'s constant block so the next reader does not re-derive it.

---

##### STEP 3 — SHIPPED. **THE ONE DEFECT THAT SURVIVES IS UNOBSERVABLE TO THE WHOLE PROJECT, NOT JUST TO THIS FILE**

`tests/rung48.rs` is **16 test fns for 16 Python ones**, one to one, and **0 source lines added or
changed** — the third step running, like step 2, entirely on step 1's readers. **16/16 in 0.49 s**,
and the slice's three suites together are **32 tests in 15.4 s** (rung 46's 14.5 s is the TPG gas,
not the gate count). `test_rung48.py` marks nothing `slow`, so there is no `#[ignore]` question here
either — the slice's two slow marks were rung 46's pair, as § 5.17 counted.

**THE SWEEP IS SHARED THE WAY PYTHON SHARES IT.** Gates 8/9/10 read one `(MARGINS, r=0.5,
flow/press)` engagement sweep through a `OnceLock`, mirroring `_SWEEPS`; the other four sweep keys
have one consumer each. 12 marches become 6, and no number changes.

**FOUR OF FIVE INJECTIONS ARE CAUGHT, AND THE SURVIVOR IS THE FINDING.** Every one was checked to
MOVE a reading before its result was read — slice S step 3's precondition, applied first this time
and not discovered afterwards. The probe dumps 14 canonical readings (the derived table's ends and
middle, a `cap`, and eight `ScheduleRelief` fields).

| injection | moves | caught by |
|---|---|---|
| `cap`'s mid-branch interpolation, `−` → `+` | 9 of 14 readings | **11 of 16 gates** |
| `cap`'s margin, `1+m` → `1−m` | 9 | **10 of 16** |
| the derived table sorted DESCENDING | 13 | **8 of 16** |
| the engagement bar, `1−1e-9` → `1+1e-9` | `s_eng`, `n_engaged` | **5 of 16** |
| **the `fuel_removed` trapezoid loses its `0.5`** | **`fuel_removed` ALONE, exactly 2×** | **NOTHING — 0 of 16** |

**AND THE SURVIVOR'S REASON IS STRUCTURAL, WHICH MAKES IT A SOURCE FINDING RATHER THAN A PORT ONE.**
`fuel_removed` is read in exactly three places in the whole project — `rung48.rs`, `test_rung48.py`
and `test_rung49.py` (enumerated by grep, not assumed) — and **every one of those readings is either
`> 0.0` or a pairwise `<`**. Both predicates are invariant under multiplication by any positive
constant, so **no scaling error in that integral is observable anywhere, in either language**: the
trapezoid could be a rectangle rule and nothing would know. It is not a margin that is too loose —
there is no bar to loosen. **Only a VALUE gate can hold it, so it is booked to step 4's oracle**,
which is the same conclusion step 2 reached from the other direction and the second time this slice
has found the missing instrument to be an oracle rather than a bar.

**PREDICTION P2 WAS NEARLY MIS-MEASURED, AND THE CORRECTION IS THE INTERESTING PART.** P2 says the
six `relief == 0.0` sites (gates 8, 9, 9b, 10, 12, 13) pass or fail as ONE block, because § 5.17
finding 3 measured that the two `min` calls read the SAME float. The four injections above split the
six carrier gates every time — 5 of 6, 5 of 6, 2 of 6, 5 of 6, never all six — which looks like a
refutation and is not one: reading the panic messages shows **every one of those failures fired on a
PRECONDITION** (`the sweep must straddle the crossing`, `upstream must rebate`, `the split must
exist`), never on a zero. The carrier gates are not a block for a reason that needs no injection at
all — **the six sites live on FIVE different marches** (the shared sweep at `r = 0.5`, gate 9b's
`r = 2.0`, gate 12's `r = 0.15`, gate 13's two other map shapes) — but that says nothing about P2.

So P2 was tested with an injection aimed at its mechanism instead: **1 ulp downward on the dormant
leg return**, which is precisely the "upstream one-ULP perturbation" gate 8b exists to catch. It
moves five readings in their last two digits, and **all six zero-carrying gates fail together —
P2 HOLDS**, with gate 8b and the two bit-for-bit reduce contracts falling beside them (10 of 16).

**AND THAT CLAIM IS WITNESSED PER ASSERTION, NOT PER GATE — THE FIRST WRITE-UP OF IT WAS NOT.** The
same error this section is about nearly went into this section: the ten panic lines were captured
but four of them were truncated out of the first read, so "all six fired on the zero" rested on two
gates whose messages were never seen. Both of those sit in assertions carrying a NON-zero half as
well (gate 9's `relief_lp == 0.0 && relief_hp > 0.0`, gate 9b's zero pair after a
`fuel_removed > 0.0`), so either could have fired on the other conjunct. Re-run with every message
captured, the residuals are the witness: gate 8 line 606, gate 9 line 711 (`relief_lp = 3.33e-16`
BESIDE `relief_hp = 7.49e-3`, so the failing half is the zero), gate 9b line 686
(`(2.22e-16, −5.55e-16)` against `(0.0, 0.0)`), gate 10 line 742, gate 12 line 793, gate 13 line 815
(`−5.55e-16`). **Six for six, each on the zero itself.**

**TWO REPAIRS THE ADVISOR'S REVIEW FOUND IN THE SHIPPED FILE, BOTH IN THE SAME CLASS AS THE FINDING
ABOVE — A CHECK THAT PASSES WHILE MEASURING SOMETHING ELSE.**

1. **Gate 7 read a NEARBY point where Python reads an exact one.** Python's `ratio[s_lp]` is a DICT
   LOOKUP over the ramp prefix and raises `KeyError` if the LP minimum is not in it; the port had
   "the last value with `s <= s_lp`", which would silently substitute the ratio at `s = R`. Measured
   after the repair: the exact lookup FINDS the key, so `s_lp` is in the prefix and on a grid point
   and the two spellings agree on this cell — the divergence was latent, and it is now a panic
   instead of a substitution.
2. **Gates 8b and 9b could report a TRUNCATED march as a moved one.** `first_diff_s` zips, so a
   limited march that broke out early yields "no difference" and trips the `expect` whose message
   says the clip genuinely moved the march. One length assertion each, ahead of the diff. Python has
   the same hole; naming it cost one line.

**AND THAT INJECTION'S FIRST TWO SPELLINGS MEASURED NOTHING, WHICH IS ITSELF A MEASUREMENT.**
Perturbing the dormant return UPWARD — by `*(1.0 + EPSILON)` and by `next_up` — moved **not one of
the 14 readings** and passed 16 of 16. The reason is one line in the derivative's min-select,
`caps.retain(|&c| c < mf)`: a cap at or above the schedule is DROPPED, so an upward error in the
dormant return is filtered out exactly. **§ 5.16's "exact structural zero" is therefore one-sided** —
the dormant fast path's value is unobservable above the schedule and bit-sensitive below it. Both
dead spellings were caught by the probe precondition rather than by their green tick, which is the
whole point of running the probe first.

**CONTRACT 5's ROW-0 ARTIFACT REPRODUCES, AND A PREDICTION FROM A MECHANISM WAS STILL WRONG.** The
first draft asserted Rust's row 0 is BIT-IDENTICAL, reasoning from the float-identical dormant
return; the run refuted it in one line. Measured: the `(a/b)*b` round-trip lands **1 ulp LOW**
(`g = +3.47e-18`), so the leg fires exactly once — **904 consultations, 903 dormant** — and
**1 of 226 rows differs**, at a ratio of `−1.11e-16` against Python's `1e-12` bar. `Tt4` and `f`
move 1 ulp with it; `nu_lp`, `nu_hp` and both `phi` do not. Python's exclusion is kept exactly, and
what the port ADDS is what Python's docstring only says in prose: the discrepancy is **confined to
row 0** (asserted as a list of moved `s` values, not an eyeball), and the counter witnesses the
mechanism directly — one non-dormant call, at `s = 0`. The ulp bar is Python's own measured `0–3`
envelope, not this cell's 1, because its docstring records the die being re-rolled by an unrelated
change to the operands.

**CONTRACT 1's MONKEYPATCH BECAME A COUNTER WITH A POSITIVE CONTROL IN FRONT OF IT.** Python sets
`_sched_fuel` to a raiser; Rust cannot rebind a method, so the claim is read off
`counters::take().sched_calls == 0`, which is stronger — it witnesses that no call HAPPENED rather
than that none escaped. An `== 0` on a dead instrument is permanently green, so the gate first arms
a `m = 0.25` schedule on the same object and the same march and asserts the counter MOVES.
`slice_s_dispatch.rs`'s rule, applied without having to be re-learned.

**TWO GRIDS THAT LOOK SHAREABLE AND ARE NOT.** `test_rung48.py`'s `SHAPES` has **three** entries
against `test_rung47.py`'s **four** (no `press/flow`), and its `SETTLE` is **4.0** against rung 47's
**2.0** because the engagement sweep reads a settled `nu_hp_end`. Reusing rung 47's helper would
have failed nothing — gate 13 iterates two named keys — and would have widened the grid silently.
Written out separately, with the difference in the header.

##### STEP 4 — SHIPPED. **THE SLICE'S OWN FINDING 6 WAS MEASURED ON THE WRONG GAS, AND ONE OF THE TWO FIGURES IT CALLED STALE IS CORRECT**

`rust/tests/topping_oracle.rs` is **1 729 keys bit-exact against PyPy on the first run**, over the
three suites' grids, plus a **CPython 3.14 arm** and two rule gates; **4 tests in 46-53 s**, and
`cargo build --release` clean. `rust/oracle/dump_topping.py` produces both TSVs. **The one source
change is a HOIST**: `schedule_relief`'s nested `raw_min` becomes a module-level
`fuel_transient::first_raw_min`, unchanged line for line, so the first-on-tie rule is reachable
from a test at all. Nothing else in `fuel_transient.rs` moved, and the four ported suites are
7/9/16 green beside the oracle, unchanged.

**THE BAR WAS WRITTEN BEFORE THE INSTRUMENT, WHICH IS THE ONE PROCEDURAL CHANGE THIS STEP MAKES.**
Every prior oracle in this port was built and then handed a pass criterion. Here steps 2 and 3 had
already NAMED the three defects the crate cannot see, so those became the acceptance test
(`M:\claud_projects\temp\rust-phase6\slice_t_step4_bar.md`, written first): the oracle discharges
the step only if re-applying each one FAILS it. Key count is a by-product.

| injection | site | ported gates (31) | `topping_oracle` |
|---|---|---:|---|
| **I4** — `required` reads the APPLIED fuel, not the SCHEDULE | `integrate_fuel_lagged` | **0** | **FAILS** (both arms) |
| **I5** — the RK4 `g` weight, a `2` dropped from `k3g` | `integrate_fuel_lagged` | **0** | **FAILS** (both arms) |
| **I6** — the `fuel_removed` trapezoid loses its `0.5` | `schedule_relief` | **0** | **FAILS** (both arms) |
| **I7** — the raw-min fold `<` → `<=` (P4) | `first_raw_min` | **0** | **the TIE GATE ALONE** |

**P4 IS THEREFORE AN EXPERIMENT RATHER THAN A PREDICTION, AND IT LANDED EXACTLY AS REGISTERED.**
The non-strict fold passes all 31 ported gates — all **32** executed Rust test functions, rung 46's disclosed-divergence test being the 32nd — AND both value arms of this file's 1 729 keys —
1 729 bit-exact values are blind to it, because no marched trajectory anywhere in the slice's
grids has a tie. Only `the_raw_min_fold_is_first_on_tie`, which builds its trajectory by hand,
sees it. *Slice J's lesson stated the other way round*: exactness bounds the CELLS visited, not the
RULES discriminated — and this time the rule was hoisted out of its caller specifically so a cell
would not be needed.

**I5's FIRST SPELLING MATCHED TWICE AND APPLIED NOTHING, AND FOUR GREEN SUITES WOULD HAVE READ AS
"THE DEFECT SURVIVES".** `integrate_fuel_lagged` (`fuel_transient.rs:1922`) and rung 52's
`integrate_fuel_asym` (`:2023`) carry BYTE-IDENTICAL RK4 accumulate lines, so a text substitution
aimed at the first hits both and the harness's `count == 1` assert refused it. The only thing that
separated "no injection was applied" from "the injection was applied and nothing caught it" was
that assert — the run itself reported `4 × ok` either way. Re-run targeted by LINE, with the rung-52
twin asserted UNTOUCHED, I5 fails the oracle and passes all 31 gates. **Third time in this port
that an injection could not have moved anything; first time the harness caught it up front instead
of the write-up catching it afterwards.**

**FINDING 6 IS CORRECTED, AND IT IS THE SLICE'S OWN FINDING.** § 5.17 finding 6 measured the bare
`Tt4` peak at 1690.5 / 1695.4 / 1702.4 / 1703.0 K, concluded that `test_rung46.py`'s `~1645` and
`test_rung47.py`'s `~1670` were BOTH stale, and reasoned that "the two files cannot both be right
about the same march". Measured here on BOTH gases, because the oracle emits the peak on each:

| gas | flow/press | press/flow | tilted | hp-only |
|---|---:|---:|---:|---:|
| `Gas.thermally_perfect()` — **rung 46's** gates 3-6 | 1644.9 | 1651.2 | 1650.9 | 1641.4 |
| CPG — **rung 47's and rung 48's** whole suites | 1695.4 | 1703.0 | 1702.4 | 1690.5 |

So **`~1645` is RIGHT** (0.008 % from the shape its own gate 6 runs) and **`~1670` is wrong by
21-33 K**. Finding 6's four measured numbers are the CPG row, taken on probes that ran the CPG gas,
and compared against a comment written for a TPG one. The tell — two files quoting one march and
disagreeing — was real; the diagnosis inverted which file was at fault, because the two files were
never describing the same march. *A census is a property of the grid*, and this is the first time
in the port that "the grid" turned out to mean the GAS. Corrected in `test_rung47.py`,
`docs/rung47-spec.md` and `docs/plans/rung47-anchor-lagged-governor.md`; `test_rung46.py`'s comment
is left standing with the gas named and a "do not fix this to rung 47's number" beside it.

**THE CPYTHON ARM NEEDED A BAR THIS FILE DID NOT HAVE, AND THE FIRST RUN IS WHAT SAID SO.** Six
keys failed, all of them `overshoot` and all of them in the TPG sections: it is
`Tt4_peak_top − Tt4_max`, the instantaneous governor pins the redline to machine zero, and a
RELATIVE deviation on a quantity meant to be zero is the ratio of two rounding errors — absolute
differences of 4.5e-13 to 3.4e-12 reading as **5.6e-2 to 3.7e-1 relative**. It falls back to an
ABSOLUTE `1e-9 K`: 294× above the measured spread, 5.5e10× below the ≥ 54.7 K a structural error
produces, and 1 000× TIGHTER than the `1e-6` the whole `held` decision rides on. Slice S's
`Phi_lp`/`Phi_hp` residual rule, one slice on, and reached the same way — by running the arm rather
than by predicting it.

**THE ARM'S NUMBERS, MEASURED AND NOT REGISTERED.** **85 of 120 TPG floats move** (worst 4.08e-10
relative), **0 of 1 463 CPG floats**, **0 of 147 discrete keys**, and `overshoot`'s worst absolute
movement is 3.41e-12 K. The CPG zero is worth stating beside slice S's, whose equivalent
prediction of 0 measured **15**: those 15 were `collapse_exponent`'s `powf`/`ln` scored curve, and
nothing in these four readers computes outside the plant.

**WHAT THE FILE ADDS THAT NO SUITE CELL REACHES, EACH DECLARED RATHER THAN QUIETLY FOLDED IN.**
The `s_eng = NaN` arm (finding 4) fires at `m = 0.55` and `m = 0.60`, `r = 0.5`, where `n_engaged`
is 0 — the lowest any suite cell reaches is 1. Its gate asserts the GOLDEN's bit pattern is the
canonical quiet NaN, because what finding 4 verified was that PyPy and Rust agree on
`7ff8000000000000` and NOT that the pattern survives the TSV round-trip, which is where a NaN key
would actually be lost. Finding 7's `m = 0.02` corner reproduces exactly as registered: the march
completes (226 points, the same as bare) with `nu_hp_end` **0.8188 against bare 0.9591**. **AND THE NaN CELLS ARE NOT WHERE I6 IS CAUGHT**: at `m = 0.55` and `m = 0.60` the leg never binds, so `fuel_removed` is exactly `0.0` and doubling the trapezoid moves it by nothing — the same exact-zero blindness step 1 found on `relief_lp`, one section on. I6 is held by sections F and G, and a future reader who adds only a dormant-margin cell would be adding coverage that key does not get. Section C
is a declared SUPERSET — 4 shapes × 6 taus where the suite runs `tau = 0.2` on four shapes and the
five-tau sweep on one, so **15 of its 24 cells are new**. Sections A-D also carry the TOPPED
march's whole `phi_excursion_fuel` record, which slice S's oracle gates on the BARE configuration
only; and section N carries march LENGTHS, which no reader returns.

**THE REMAINING PREDICTIONS, ADJUDICATED ON THIS FILE'S OWN GOLDEN.** **P3 HOLDS**: **147
discrete keys — every `held`, `tau_gov_is_set`, `npts`, `n_engaged` and `monotone` in the file —
and NOT ONE flips**, on either arm, which is the direct evidence finding 2's 5-to-7 orders of slack
predicted. **P5 HOLDS EXACTLY**: of 35 `held` readings, the **11 `True` ones are precisely the 11
`tau_gov = None` cells** and all 24 lagged cells miss, so `held` needs no tolerance of its own.
**P1 was discharged at step 1** and step 4 adds nothing to it — the oracle passed first run, which
P1 registered as the outcome to distrust, and the four injections above are the audit it asked for.

**`#[ignore]`: MEASURED AND NOT EARNED, A THIRD TIME — ON THIS CRATE'S OWN TIMING, NOT STEP 1's.**
Step 1's crate summed 606.6 s over 70 binaries and **that crate no longer exists**: rung 47's and
rung 48's suites landed at steps 2 and 3 and this file at step 4, so quoting it as the denominator
would be mixing vintages, which is the mistake slice M's rule exists to prevent. Taken from the
run that closes this step — *never run the gate for timing; take it from a run already happening* —
the whole crate is **exit 0, 777 tests over 89 binaries, 0 failed, 0 ignored**, summing
**1 183.6 s (19.7 min) of test execution**, and `topping_oracle` is **46.2 s of it — 3.9 %**. Four
binaries carry **64 %** (225.3 / 204.4 / 168.5 / 161.0 s), all of them oracles. Not earned; the
crate still carries **zero** `#[ignore]` attributes. The Python gate is green beside it —
**1 364 passed in 18:45** — which is the one measurement this step could not take from a Rust run,
since the doc corrections landed in `tests/test_rung46.py` and `tests/test_rung47.py`.

---

---

### 5.18 SLICE U (rungs 49 + 50 + 51 + 52, the GATES on the `φ` floor, the forced release, its RATE and the asymmetric LAG) — PRE-REGISTERED, four probes MEASURED first

**THE LAST SLICE OF PHASE 6.** Rung 49 is the first limiter that watches the PROTECTED variable
(a `φ` floor on one spool), rung 50 forces its release edge with `s_off`, rung 51 gives that
release a RATE (`tau_rel`), and rung 52 replaces both forced edges with the realisable
`AsymmetricLag` that pins its own trigger. § 5.12 settled that these are keyword arms on ONE
method, so slice S shipped the plant entire and this slice — like slice T before it — is **gates
plus readers only**.

**THE SCOPE IS SETTLED BY § 5.17's GREP, NOT RE-DERIVED.** Slice T's reader table assigned nine of
the thirteen rung-46–52 readers here: `surge_relief` · `floor_sweep` (49) · `release_relief` ·
`release_sweep` (50) · `rate_sweep` · `deficit_curve` (51) · `lag_relief` · `lag_sweep` ·
`factorization_grid` (52), `engine.py:5579–5996`.

**COUNTED, NOT TAKEN FROM A HEADER.** `pytest --collect-only`: **63 items = 17 + 15 + 16 + 15**,
and **4 carry `slow`, all four in rung 52** — counted with `--collect-only -m slow`, not by
grepping for the decorator, because a module-level `pytestmark` would not show in a grep. Sizing:
**418 source lines** over **1 727 test lines** — gates outweigh code 4×, slice T's mirror image
again. The crate stands at **23 795 source lines over 87 test binaries** going in.

**THE ORDER IS FORCED, NOT CHOSEN.** Each rung's reduce gate differences against its predecessor
(`tau_rel=None` is bit-for-bit rung 50; `lag=None` is bit-for-bit rungs 49/50/51), so 49 → 50 →
51 → 52.

---

#### The four probes — `probe_u1.py`, `probe_u1b.py`, `probe_u2.py`, `probe_u3.py` (`M:\claud_projects\temp\rust-phase6\`)

Run on the SUITES' OWN grids. **AND THE FIRST RUN OF `probe_u2` WAS NOT** — it used `SETTLE=2.0`
for all four rung files when rungs 51 and 52 use **4.0**, and had to be re-run. Slice S step 4's
lesson landed on the very probe written to honour it; the numbers happened to be settle-invariant
(every minimum sits upstream of both march ends), which is exactly why the mistake would not have
announced itself. **Every constant below is copied from `tests/test_rung49-52.py` and the two
settle times are kept apart by name.**

**FINDING 1 — THE REFUSAL CENSUS COMES BACK COMPLETE, AND THREE OF PYTHON'S OWN REFUSALS ARE
PROVABLY UNREACHABLE.** Seven of the 63 gates are pure refusal gates, and the concern was that
slice S ported the plant without the composition asserts — *an oracle cannot see a missing gate*,
pointed at slice S a second time. Measured instead of read: all **25** refusals fire in Python
(9 composition probes over 7 asserts, 8 `lp_disabled`, 8 reader/constructor), and all **14** that
belong to the marcher are present in `fuel_transient.rs`, message for message. **No hole.**

But `probe_u1b` swept all **255** arming combinations of
`(freeze, Tt4_max, tau_gov, accel, surge, s_off, tau_rel, lag)` through the degenerate object and
counted which refusal fires:

| `lp_disabled` refusal | combinations that reach it |
|---|---:|
| `freeze` | 36 |
| `Tt4_max` / `tau_gov` | 23 |
| `accel` | 8 |
| `surge` | 4 |
| **`s_off`** (rung 50) | **0 — UNREACHABLE** |
| **`tau_rel`** (rung 51) | **0 — UNREACHABLE** |
| **`lag`** (rung 52) | **0 — UNREACHABLE** |

The mechanism is an ordering: arming `s_off`/`tau_rel`/`lag` at all requires an armed leg (the
COMPOSITION asserts refuse otherwise), and the `accel`/`surge` refusals precede them inside the
degenerate block. **So three asserts in the shipped Python source can never fire, and Rust copied
all three faithfully** — which is the right port under *COPY vs REDERIVATION*, and leaves the
defect in the source. The consequence lands on the gates: rungs 49/50/51/52's four
`test_reduce_lp_disabled_asserts` all arm `surge=`, so **all four fire the SAME rung-49 assert** —
four gates named for four different rungs are ONE claim.

**FINDING 2 — THE `armed` SEED IS A DEAD DISTINCTION, AND `n_recross` IS GATED.** `lag_relief`'s
crossing loop seeds `armed = None` and guards `if armed is False`, so the FIRST crossing is not
counted as a re-crossing. The natural Rust spelling `let mut armed = false` counts it and puts
`n_recross` one high on every row. Measured over six lag cells (rung 52's own `(tau_att, tau_rel)`
grid at both ramp rates): the first point with `g > 0` is **always still ATTACKING**
(`required > g`), so both seeds agree — `n_recross = 1` everywhere, under both spellings.
`test_rung52.py:224` asserts `n_recross == 1`, so **the wrong seed ships past the one gate that
reads the key.** It needs a MANUFACTURED trajectory, and `first_raw_min`'s tie gate in
`topping_oracle.rs` is the template. (Second trap in the same eight lines, registered so the port
does not "tidy" it: the `continue` on `g <= 0.0` means an unclipped point does NOT disarm, so
folding the guard into one `if/else` is wrong.)

**FINDING 3 — A DECISION KEY AT ONE ULP — THE OPPOSITE OF SLICE T's FINDING 2.** Slice T measured
5–7 orders of slack on all four of its bars and concluded the port's risk was not in the
decisions. Slice U's are not like that:

| decision | bar | nearest INSIDE | nearest OUTSIDE | slack |
|---|---|---:|---:|---:|
| engaged mask (`mf < mf_sched·(1−1e-9)`) | `1e-9` rel | `1.685e-3` | `0.0` exactly | 1.7e6 × / structural |
| `hold_err` (the SLIDING MODE) | `1e-9` | `7.77e-16 … 1.33e-15` | — | ~1e6 × |
| `lag_relief` `eps = 0.05` | `0.05` | `0.0505` / `0.0522` | — | **1–4 %** |
| `lag_relief` `eps = 0.01` | `0.01` | `0.00916` / `0.01020` | — | **2–8 %** |
| **`both_edges_inside_ramp`** (`eng[-1] < r`) | — | `0.06` | **`1.11e-16`** | **ONE ULP** |

Over rung 49's eight floor cells the distance `r − s_rel` is `0.06` / `0.16` inside and `−0.10` /
`−0.02` / `−0.42` / `−0.12` / `−0.02` outside — **one grid cell at the tightest, not orders** —
and the eighth (HP floor `0.8650`) is **`−1.11e-16`**: the boolean is decided by one ulp.

**IT SURVIVES ONLY BECAUSE BOTH LANGUAGES SPELL THE MARCH COORDINATE THE SAME WAY.** Python and
Rust both accumulate `s += ds` from `0.0`; summing `0.02` twenty-five times gives
`0.50000000000000011` while `25 * 0.02` gives exactly `0.5`. **A "cleaner" `s = k as f64 * ds`
flips a published boolean.**

**AND THE FIRST WRITING OF THIS FINDING NAMED A READER THAT DOES NOT EXIST.** It said `main.py`
PRINTS the one-ulp cell — grepped off the format string at `main.py:2625` rather than off the
sweep that feeds it. Read: that panel sweeps **`(0.7550, 0.7500, 0.7450, 0.7400)`, `spool="lp"`**,
and rung 49's only other reader of the boolean (gate 4) is LP-only too. The HP floors are swept by
gates 9 and 9b, which read `s_eng` / `s_rel` / `relief_other` and **never the boolean**. So
**nothing in either language reads `both_edges_inside_ramp` on the cell that sits at one ulp** —
the key's exposure is the *one grid cell* of margin on the cells that ARE read, and the ulp cell
is what shows how thin that boundary is. Registered as measured, not as gated; the note goes at
the march loop where a future reader will hit it — slice O's rule.

**FINDING 4 — BOTH NaN ARMS SHIP UNGATED; THE `Option` PAIR DOES NOT.** `surge_relief` and
`release_relief` both return `s_eng`/`s_rel` = `NaN` when nothing engages. Minimum `n_engaged`
over every rung-49 floor cell is **10**; over every rung-50 `s_off` cell it is **2**. Never zero,
so **both NaN arms need an ADDED oracle cell** — slice T's finding 4 repeating on two more
readers. Companion hole, worth the sentence because it is a real ambiguity and not just a port
note: when nothing engages `release_relief` returns `s_eng = nan` but `deficit_at_release = 0.0`,
**two different sentinels for one condition in one dict**, and `0.0` is a legitimate deficit — so
that key alone cannot separate "never engaged" from "engaged with zero deficit".

The **`Option` arms are live and need no added cell**: rung 50's accel-only sweeps (margins 0.25 /
0.15) return `spool`, `phi_lim`, `relief_watched` and `relief_other` as Python `None`, while its
surge-only sweeps return `margin` as `None`. Both arms are exercised by shipped gates, so Rust's
`Option<…>` fields are gated by construction.

**FINDING 5 — THE THREE READERS RETURN THREE DIFFERENT RECORDS, AND A KEY-COUNT CENSUS WOULD NOT
SEE IT.** The obvious design — one relief record with optional fields — is wrong, and the reason
is not just the `Option` pair. Measured with `sorted(row.keys())` on all three:

| reader | keys | its OWN keys |
|---|---:|---|
| `surge_relief` | **25** | `hold_err`, `both_edges_inside_ramp`, `s_min_other`, `Tt4_peak_lim` |
| `release_relief` | **27** | `s_off`, `deficit_at_release` |
| `lag_relief` | **34** | `s_cross`, `g_at_cross`, `required_at_cross`, `g_peak`, `n_recross`, `tau_att`, `Tt4_peak_lag`, `min_phi_{lp,hp}_lag`, and the four f-string keys `s_{eng,rel}_0.05` / `_0.01` |

**Only 15 keys are common to all three**, and five more (`ds`, `margin`, `s_min_lp`, `s_min_hp`,
`tau_rel`) are shared by rungs 50 and 52 but absent from rung 49 — which reports `s_min_other`
instead of the pair. The SUFFIX changes too (`min_phi_lp_lim` at rungs 49/50, `min_phi_lp_lag` at
rung 52), and `relief_watched`/`relief_other` are plain floats out of `surge_relief` (its `surge`
is a required positional) but `None`-able out of the other two. **A shared struct emits phantom or
renamed keys, and a key-COUNT census passes on both sides while the dump compares nothing** — the
*documented gate that doesn't exist* family. Three structs, and **25 / 27 / 34 are registered here
as measured bars** for the oracle to assert.

**FINDING 5b — THE MARCH LENGTH IS THE SAME ON ALL THREE MARCHERS, SO THE ANTI-DEFLATION PAIR IS
SAFE.** Every reader returns `nu_hp_end = lim[-1][…]`, so a one-point length difference would move
it silently. Python bounds the loop with `range(int(round(s_end / ds)) + 1)` and Rust with
`(s_end / ds).round_ties_even()` — the same banker's rounding — and measured on
`integrate_fuel`, `_lagged` and `_asym` alike: **201 points at `s_end = 4.0`, 301 at `6.0`**, last
`s` = `4.0000000000000027` / `5.99999999999996`, identical across all three. Rungs 51/52's
`6.0 / 0.02 = 299.99999999999994` rounds to 300 in both languages.

**FINDING 6 — `first_raw_min` ALREADY EXISTS AND IS ALREADY GATED, SO SLICE T's P4 IS DISCHARGED
RATHER THAN RE-REGISTERED.** It folds with strict `<` and carries its own manufactured tie gate.
The three `raw_min` closures in `surge_relief` / `release_relief` / `lag_relief` REUSE it and the
prediction is not written a second time. One API gap does remain: `SurgeLimiter::read` takes a
`FuelInstant`, but rung 49's `hold_err` reads the floored `φ` off a **`FuelPoint`** — Python's
`surge.key()` returns a dict key that serves both. Slice U adds the `FuelPoint`-side accessor.

**FINDING 7 — A DOCSTRING NUMBER IS STALE AGAIN, AND THIS TIME IT IS THE SLICE'S HEADLINE RATIO.**
`factorization_grid`'s docstring says the additive-separability residual comes back at "62-70 % of
them at both ramp rates measured", and `test_rung52.py` gate 4 says "70 % at r=0.5 against 62 % at
r=2.0". Measured on the gates' own cells, at the right settle time: **65.0 % at r=0.5 (ds=0.01)
and 58.9 % at r=2.0 (ds=0.02)**. Both clear the gate's `0.4` bar comfortably and **no gate reads
the quoted figures**.

**AND THE CORRECTION IS SHIPPED WITH ITS SEARCH STATED, BECAUSE SLICE T's SAME-NUMBERED FINDING
WAS DIAGNOSED BACKWARDS.** Two measurements sitting ~5 points below two quoted figures is more
often a different FORMULA than staleness, so four alternative denominators were tried against the
code's own `max_residual / max_main_effect`:

| denominator | r=0.5 (quoted 70 %) | r=2.0 (quoted 62 %) |
|---|---:|---:|
| `max_main_effect` — **what the code computes** | **65.0 %** | **58.9 %** |
| the `tau_att`-direction main effect alone | 90.5 % | 77.1 % |
| the `tau_rel`-direction main effect alone | 65.0 % | 58.9 % |
| mean rather than max residual | 16.3 % | 14.0 % |
| the grid corner `D(ta0,tr0)` | 69.8 % | 50.7 % |

**No single alternative reproduces both.** The corner denominator lands on 69.8 % at r=0.5 —
close enough to "70 %" to be why that half was written — but gives 50.7 % where the docstring says
62 %, so it is a coincidence and not the missing formula. The correction stands, and this is the
third outing in this port for the rung-63 lesson, *check a quoted number was taken at THIS rung's
settings*, with "settings" now measured to include the SETTLE TIME as well as slice T's GAS.

**FINDING 8 — `credit_spread == 0.0` HOLDS EXACTLY ON BOTH GRIDS.** `{0.02: 0.0, 0.20: 0.0}` at
r=2.0 and `{0.02: 0.0, 0.32: 0.0}` at r=0.5. It is ONE claim with a named mechanism (`tau_rel` is
never READ while `required > g`, so the whole pre-crossing march is bit-identical), so it passes
or fails as a block — slice T finding 3's shape, on a different quantity.

---

#### The pre-registered predictions

**P1 — the slice finds ≥ 1 defect in shipped slice-S limiter code.** Slice T's registered
expectation, repeated because its basis has not changed: § 5.16 measured ~40 % of `integrate_fuel`
to be limiter machinery no phase-6 rung gate reached, and these 63 gates are the first coverage on
`try_surge_fuel`, `release_weight`'s interior branch and `integrate_fuel_asym`. **If all 63 pass
first try, that is the surprise and the instruments get audited before the result is believed.**

**P2 — the four `lp_disabled` gates pass or fail as ONE.** Finding 1's mechanism says they all
fire the rung-49 `surge` assert. A SPLIT outcome refutes the mechanism and is the more interesting
result.

**P3 — `both_edges_inside_ramp` reproduces bit-for-bit, one-ulp cell included.** Finding 3. A flip
means the march coordinate was "cleaned up" to `k * ds`, and is to be read as that and nothing
else.

**P4 — `n_recross` passes under BOTH `armed` seeds.** Finding 2. Only the manufactured cell
separates them; predicted that `let mut armed = false` passes all 63 ported gates without it.

**P5 — `credit_spread` is EXACTLY `0.0` in Rust on both grids, with no tolerance.** Finding 8.

**P6 — rung 51's step adds NO NEW LOGIC.** `release_relief` lands COMPLETE (with `tau_rel`) at
rung 50's step, because `tau_rel` is its kwarg and not a separate path; rung 51's two readers are
loops over it. Registered as checkable at ship, the way slice T registered its zero-source-line
steps.

---

#### The steps

| step | content |
|---|---|
| **1** | ✅ `surge_relief` + `floor_sweep` + rung 49's 17 gates; the `FuelPoint`-side `φ` accessor (finding 6); the one-ulp note at the march loop (finding 3). **The sweeps are memoised from the start, not after the crate time is measured**: rung 49's 17 gates share ~3 `_SWEEPS` entries in Python, and an unmemoised port redoes 8 marches per gate — a `OnceLock` per sweep is a design decision, not a later optimisation |
| **2** | `release_relief` **COMPLETE, with `tau_rel`** + `release_sweep` + rung 50's 15 gates |
| **3** | `rate_sweep` + `deficit_curve` + rung 51's 16 gates — P6's check |
| **4** | `lag_relief` + `lag_sweep` + `factorization_grid` + rung 52's 15 gates |
| **5** | the oracle: the ADDED no-engagement cell (finding 4), the MANUFACTURED `armed`-seed cell (finding 2), the CPython arm; finding 7's doc correction; and finding 1's three unreachable asserts written up where the next reader hits them |

`#[ignore]` on the four `slow` gates is decided by slice M's rule — a MEASURED in-suite cost
against the crate total — and is not pre-judged here.

##### STEP 1 — SHIPPED. **THREE INJECTIONS INTO SHIPPED READER CODE PASS ALL 17 GATES, AND ONE PUBLISHED BOOLEAN IS PROTECTED BY TWO SPELLINGS AT ONCE**

`fuel_transient.rs` gains the 25-field `SurgeRelief`, `surge_relief`, `floor_sweep`, finding 6's
`SurgeLimiter::read_point` (the `FuelPoint`-side twin of `read`), and finding 3's note at all three
march loops. **226 lines added and ONE line deleted — and the deleted line is a DOC COMMENT**, so
zero executable lines changed: a pure addition over slice S's plant, measured with
`git diff --stat` rather than intended. `tests/rung49.rs` is **847 lines, 17 test fns for Python's
17**, and runs **17/17 in 0.38 s** with the two shared sweeps behind `OnceLock`s (8 marches instead
of 56). `cargo build --release` clean; `cargo clippy --all-targets` adds no warning at any line
this step touched (73 pre-existing, none in `rung49.rs` and none in the new source).

**THE FULL-CRATE FIGURE IS MEASURED, AND NEITHER ITS WALL CLOCK NOR A PER-BINARY TABLE IS
QUOTED.** `cargo test --release` came back **exit 0**, and the crate now stands at **794 tests
over 88 test binaries with 0 `#[ignore]`** — counted with `cargo test --release -- --list` and a
`^\s*#\[ignore` grep (the 12 plain-text hits are all prose, this file's own header among them),
not carried over from a previous count. **The count comes from `--list`, the green from the EXIT
CODE, and no binary-level breakdown was captured**: that run's stdout was piped through `tail`, so
only the doc-test tail survived. `cargo test` exits non-zero if any binary fails, so exit 0 does
cover all 88 — but this sentence is the whole of what was measured, and nothing finer should be
read into it. **No time is recorded either.** The run overlapped the injection battery, which took
the build lock repeatedly, so its elapsed figure measures contention rather than the crate.
Quoting it would have been *never run the gate for timing* broken from the other end — by quoting
a number that was genuinely measured and measures the wrong thing.

**FINDING 1 — THE 17/17 WAS CHECKED AGAINST VALUES BEFORE IT WAS BELIEVED, AND IT HELD: 575 KEYS
BIT-EXACT ON THE FIRST RUN.** P1 registered a clean first pass as the outcome to distrust, so the
gates' own cells were dumped from both languages and diffed: **200 keys over the 8 cells of the two
shared sweeps**, then **375 keys over the 15 cells the four single-consumer sweeps and gate 12
reach** — every cell any of the 17 gates touches, **23 cells, 575 keys, zero differences against
PyPy**. Three of § 5.18's pre-registered numbers were re-measured rather than copied and all three
reproduced: the dict is **25** keys by Python's own `len()`; finding 3's `r − s_rel` table is
`+0.06` / `+0.16` inside and `−0.10` / `−0.02` / `−0.42` / `−0.12` / `−0.02` outside with the
eighth cell at **`−1.11e-16`**; finding 4's minimum `n_engaged` over the rung-49 floor cells is
**10**.

**FINDING 2 — THE BAR MARGINS SPLIT, AND THIS FILE IS NOT SLICE T's.** Slice T shipped 9/9 green
while blind to a 24 % value error because no bar's margin had been measured; that lesson was
applied here before the green was accepted:

| gate | bar | worst measured | slack |
|---|---|---:|---:|
| 3 | `hold_err < 1e-9` | `8.88e-16` | 1.1e6 × |
| 3 | `\|relief_watched − (φ_lim − min φ_lp)\| < 1e-5` | `1.86e-8` | 539 × |
| 5 | `min relief_other < −0.005` | `−0.010403` | 2.1 × |
| 6 | `s_min_other − s_rel ≤ 3·ds = 0.06` | `0.040` | **1.5 × — ONE GRID CELL** |
| 9 | `\|relief_other − forecast(s_eng)\| < 2e-3` | `1.7765e-3` | **1.13 × — 11 %** |
| 10 | `\|nu_hp_end − nu_hp_end_bare\| < 5e-4` | `1.22e-5` | 41 × |
| 10 | `\|relief_other[0]\| < \|relief_other[2]\|` | `3.25e-4` vs `1.04e-2` | 32 × |
| 11 | `nu_hp_end_bare − nu_hp_end > 0.2` | `0.2198` | **1.10 × — 10 %** |
| 12 | `\|Δrelief_other\| < 0.25·\|relief_other\|` | 8.8 % | 2.8 × |
| 4 | `both_edges_inside_ramp` | `r − s_rel = −0.02 / +0.06` | **one grid cell** |

Slice T measured 5–7 ORDERS on all four of its bars and concluded the port's risk was not in the
decisions. **Four of these ten sit inside `1.5 ×`, and three are VALUE bars § 5.18 finding 3's
table does not reach** — it covered the engaged mask, `hold_err`, `lag_relief`'s `eps` and the
boolean, none of which is gate 9's forecast, gate 6's placement or gate 11's collapse. Gate 9's is
the tightest thing in the file: an 11 % error in `relief_other` or in the bare-march forecast breaks
it and nothing looser would notice.

**FINDING 3 — TWELVE INJECTIONS INTO THE SHIPPED READER, AND FIVE LIVE DEFECTS PASS ALL 17
GATES.** Bit-exactness says the port is faithful; it says nothing about which of the 17
gates has POWER. Each plausible port defect was written into `surge_relief` on purpose and the
suite asked:

| injection | moved a value? | gates that caught it |
|---|---|---|
| **I** `relief_watched` / `relief_other` swapped | yes | **8 of 17** |
| **G** `hold_err` folds over EVERY point, not the engaged window | yes | gate 3 alone |
| **A** `s_min_other` reads the BARE argmin (the mechanical rung-48 copy) | yes | **gate 6 alone** |
| **B** `s_min_other` reads the WATCHED spool | yes | **gate 6 alone** |
| **J** `nu_hp_end` read off the BARE march | yes | gate 11 alone |
| **C** `hold_err` ignores which spool is floored | yes — **HP rows only** | **NONE** |
| **E** the march coordinate becomes `k * ds` | yes — all 8 rows | **NONE** |
| **F** `fuel_removed`'s trapezoid loses its `0.5` | yes | **NONE** |
| **K** `tt4_peak_lim` read off the BARE march | yes | **NONE** |
| **L** `tt4_peak_bare` read off the LIM march | yes | **NONE** |
| **D** `both_edges_inside_ramp` uses `<=` at the ramp end | **NO — inert on all 23 cells** | — |
| **H** `both_edges_inside_ramp` drops its `0 < eng[0]` clause | **NO — inert on all 23 cells** | — |

**THE TABLE HAS A "MOVED A VALUE?" COLUMN BECAUSE THE FIRST WRITING OF IT DID NOT, AND THREE ROWS
WERE WRONG WITHOUT ONE.** A "0 gates caught it" row means two different things depending on that
column, and lumping them together over-counts the ungated surface — exactly the deflation this
port keeps having to exclude. Measured in a third round rather than reasoned:

* **C is a NO-OP on every cell gate 3 reads.** `read_point` returns `phi_lp` for an LP-watched leg,
  so the injection changes nothing on the LP sweep and moves only the four HP rows. The claim is
  therefore not "a live defect passed 17 gates" but the narrower and sharper **no gate reads
  `hold_err` on an HP-watched cell at all** — gate 3 sweeps `LP_FLOORS` only.
* **D and H are inert on the WHOLE grid, and H's inertness was a SURPRISE.** The expectation was
  that dropping `0 < eng[0]` would flip the boolean on the flat-lp cell, the only one with
  `s_eng == 0`. Measured: it does not — that cell's `s_rel = 0.72` already fails `eng[-1] < r`, so
  the dropped clause is REDUNDANT there. And the inertness is established by SWEEPING the cells
  rather than by the gates staying green: over all 23, **no cell has `s_rel == r` exactly** (so D
  cannot bite) and **exactly one has `s_eng == 0`** (so H can bite only there, and does not).
  Both guard clauses are individually unreachable on this whole grid; only their conjunction
  with the march coordinate reaches the value (finding 4).
* **L was run because K did not test what the first writing claimed.** That draft said
  "`tt4_peak_lim` and `tt4_peak_bare` entirely" off an injection that moved only `_lim`. `_bare` is
  now MEASURED ungated rather than inferred — *five typed count bars, five wrong* is this port's
  standing reason not to let an inference into a claim.

So the ungated surface after step 1 is **`hold_err` on an HP-watched cell**, **`fuel_removed`'s
VALUE**, **`tt4_peak_lim`**, **`tt4_peak_bare`**, and **the march coordinate's spelling** — five
things, all measured live. That list IS the sizing for step 5's oracle, derived rather than
guessed. Two rows are worth naming: `fuel_removed` is **slice T step 3's finding repeating on a
different reader** — every consumer is scale-invariant, so doubling it is invisible — and gate 6 is
the SOLE defender of `s_min_other`, i.e. of the key carrying the rung's own mechanism.

**FINDING 4 — THE ONE-ULP BOOLEAN IS PROTECTED BY TWO SPELLINGS AT ONCE, AND NEITHER OF THEM IS
GATED.** § 5.18 P3 said a flip of `both_edges_inside_ramp` means the march coordinate was cleaned
up to `k * ds`, and is to be read as that and nothing else. Measured on the `−1.11e-16` cell (the
HP floor `0.8650`):

| source | `s_rel` bits | boolean | gates |
|---|---|---|---|
| shipped | `3fe0000000000001` | `false` | 17/17 |
| `<=` at the ramp end (D) | `3fe0000000000001` | `false` | 17/17 |
| `k * ds` (E) | `3fe0000000000000` — **exactly `r`** | `false` | 17/17 |
| **both** | `3fe0000000000000` | **`true`** | **17/17** |

**P3 IS SHARPENED AND ITS DETECTOR IS CORRECTED.** Cleaning the coordinate alone does NOT flip the
boolean — it lands `s_rel` exactly on `r` and the STRICT `<` absorbs it — so the flip is a
TWO-fault conjunction and each fault alone is invisible. And in all four builds the suite is 17/17,
which is consistent with § 5.18's own correction that nothing in either language READS the boolean
on that cell. **The only instrument that can hold it is step 5's oracle bits**, and until then the
note at the march loop is doing the entire job. E was verified LIVE before its green was read as
evidence — all 8 probe rows moved.

**AND THE INJECTION HARNESS CAUGHT ITSELF ONCE, WHICH IS WHY IT HAS A SITE-COUNT GUARD.** Round
one's E matched **3** sites and applied NOTHING; the script reported `PATCH SITE COUNT 3 — NOT
APPLIED` rather than running and printing green. That is slice T step 4's lesson (*an injection
matching TWICE applies nothing and still reports green*) landing on the very script written after
it, and the guard is the only reason it did not become a fifth "NONE" row above. Separately, a
nested-heredoc mistake left an injected `fuel_transient.rs` on disk; it was restored from the
backup and the restore VERIFIED (`grep -c k_inj` = 0, `git diff --stat` back to 226/1, suite
re-run) rather than assumed.

**THE BATTERY IS NOW THE SLICE'S STANDING INSTRUMENT, RUN PER STEP.** Steps 2–4 re-derive
`release_relief` / `rate_sweep` / `lag_relief` from this same body, and the answer to *which
gate has power* changes as each rung's gates land — a key ungated here may acquire a reader at
rung 50, and rung 50's own new keys start with none. So the injections are re-run at every
step rather than once at step 5, and each step's section carries its own table with the
"moved a value?" column.

**PREDICTION STATUS.** **P1 holds in an unexpected direction** — no defect was found in slice S's
limiter code, but eleven injections found a defect SURFACE in this step's own new code that the
gates cannot see, which is the same warning about instrument power P1 was written to raise. **P3
sharpened** (finding 4). **P2** needs rungs 50–52 and **P4/P5/P6** their own steps.

##### STEP 2 — SHIPPED. **A DEFECT STEP 1 MEASURED INVISIBLE BECOMES VISIBLE ONE RUNG LATER, AND THE RUNG'S OWN NAMED QUANTITY HAS NO READER ANYWHERE**

`fuel_transient.rs` gains the 27-field `ReleaseRelief`, `release_relief` (complete, `tau_rel`
included — § 5.18 P6) and `release_sweep`. **229 lines added and ZERO deleted**, measured with
`git diff --stat`: a pure addition over slice S's plant, cleaner than step 1's 226/1.
`tests/rung50.rs` is 15 test fns for Python's 15 and runs **15/15 in 0.90 s** with the ONE shared
sweep behind a `OnceLock`. `cargo build --release` clean; `cargo clippy --all-targets`
reports **73** warnings, the SAME pre-existing count step 1 measured, and **none at any line this
step touched** — checked by grepping the clippy output for `rung50.rs` and for `fuel_transient.rs`
line numbers in the added range, not by eyeballing the total.

**THE FULL-CRATE FIGURE IS MEASURED AND ITS SCOPE IS STATED.** `cargo test --release` came back
**exit 0** with **809 passed / 0 failed / 0 ignored**, summed over the 91 `test result` lines — which
are **89 test binaries + the lib target + doc-tests**, counted from the run's own `Running` /
`Doc-tests` lines rather than assumed. That is step 1's **794 over 88 binaries** plus this file's
**15 over one**, exactly, so the addition is visible in the total rather than inferred from it.
**No wall clock is recorded** — the run overlapped the injection battery, which took the build
lock repeatedly, so its elapsed figure would measure contention. *Never run the gate for timing*,
and equally never quote a genuinely-measured number that measures the wrong thing.

**THE MEMO STRUCTURE IS NOT RUNG 49's, AND COPYING IT WOULD HAVE BEEN A DIFFERENT MEASUREMENT.**
Python memoises on the OFFSET TUPLE as well as the leg, so of eleven distinct sweep keys **exactly
one has two consumers** (gates 3 and 4). Two neighbours look foldable and are not: gate 9 runs the
SAME offsets and floor at `settle = 4.0`, a different march length; and gate 10's `(1.10, 1.56)`
is a SUBSET of gates 3/4's `R2_OFFS` at an otherwise identical key, which Python still recomputes.
Slicing the larger sweep would give the same numbers here and quietly stop being a port. One
`OnceLock`, not rung 49's two.

**FINDING 1 — 1 323 KEYS OVER 49 CELLS, BIT-EXACT ON THE FIRST RUN.** § 5.18 P1 registers a clean
first pass as the outcome to distrust, so step 1's move was repeated: every cell any of the 15
gates touches — eleven sweeps, gate 8's three matched-release cells and contract 1b's unforced one
— dumped from both languages and diffed. **Zero differences against PyPy.** The Python side of the
probe IMPORTS `tests/test_rung50.py` and reads its constants and its `_sweep` off the module
rather than copying them, which is slice S step 4's lesson applied to the instrument that lesson
was written about; the two SETTLE values (2.0 and gate 9's 4.0) therefore cannot drift apart the
way § 5.18's own first probe run had them.

*(The first diff came back 100 % different and was an artifact: Python's text-mode `open(p, "w")`
writes CRLF on Windows and Rust wrote LF. Recorded because "everything differs" is exactly the
shape a real port failure has, and one minute was spent on the wrong hypothesis.)*

**FINDING 2 — THE FORCED-RELEASE COMPARISON SITE IS A ONE-*CELL* KNIFE EDGE, NOT A ONE-ULP ONE —
AND IT WAS MEASURED BEFORE A GATE WAS WRITTEN.** § 5.18 finding 3 found a published boolean decided
at one ulp because both languages accumulate the march coordinate. Rung 50 adds a site rung 49 has
no analogue for: `release_weight` tests `s < s_off` against that same accumulated `s`, and
`release_sweep`'s own docstring instructs callers to pass `s_off` ON the `ds` grid — which is
precisely where accumulated `s` and a "cleaner" `k * ds` can straddle the bar. Swept over the
eighteen `s_off` values this suite passes at `ds ∈ {0.02, 0.01}`:

| site | accumulated `s` at the index | `k * ds` | last ARMED index |
|---|---|---|---:|
| `ds = 0.02`, `s_off = 0.20` | `0.19999999999999998` | `0.2` | **10 vs 9** |
| `ds = 0.02`, `s_off = 0.26` | `0.25999999999999995` | `0.26` | **13 vs 12** |
| `ds = 0.01`, `s_off = 0.10` | `0.09999999999999999` | `0.1` | 10 vs 9 |
| `ds = 0.01`, `s_off = 2.20` | `2.199999999999997` | `2.2` | 220 vs 219 |
| + two more on the RK4 sub-stage coordinates | | | |

**Six comparison sites move the release edge by a WHOLE GRID CELL** between the two spellings, and
**two are live cells of this suite** — gates 5 and 10b both read `s_off = 0.26`, gate 5 reads
`0.20`. Under the shipped accumulating spelling the leg stays armed one point LONGER. This is a
coarser hazard than the one-ulp boolean and it lands on a different site, so the note goes at
`release_relief` as well as at the march loop. Both languages accumulate, so both agree — the
point is what a tidy-up would cost, and here it would cost a gate-visible cell rather than a
key nothing reads.

**FINDING 3 — 13 OF THE 27 KEYS HAVE NO READER IN THE SUITE, AND ONE OF THEM IS THE RUNG'S OWN
NAMED QUANTITY.** Measured by stripping the docstrings out of `test_rung50.py` and counting
`["key"]` sites, then confirmed at the call sites. Unread: `deficit_at_release`, `ds`, `margin`,
`min_phi_hp_bare`, `min_phi_hp_lim`, `min_phi_lp_lim`, `n_engaged`, `phi_lim`, `r`,
`relief_other`, `relief_watched`, `rho`, `s_hp_bare`, `s_lp_bare`, `spool`, `tau_rel`.

`deficit_at_release` is the one worth naming: **gate 8's entire subject is the deficit** — its
docstring is "the debit is MONOTONE INCREASING in the deficit" — and the gate reads `fuel_removed`
as a proxy instead. The key that carries the rung's concept has no gate in either language. Its
companion is § 5.18 finding 4's ambiguity, re-measured here: with nothing engaged the key returns
`0.0` while the same record's `s_eng` / `s_rel` return `NaN` — two sentinels for one condition in
one record, and `0.0` is a legitimate deficit.

**FINDING 4 — § 5.18 P2 IS CONFIRMED BY MEASUREMENT RATHER THAN INFERENCE, AT ITS SECOND OF FOUR
DATA POINTS.** P2 says the four rungs' `lp_disabled` gates pass or fail as ONE because they all
fire the rung-49 assert. Python's own gate matches the substring `"inherently two-shaft"`, which
cannot tell four asserts apart, so `contract5_lp_disabled_refuses` asserts the **whole message**
and adds `assert!(!m.contains("rung-50"))`. It passes: the gate named for rung 50 fires the assert
named for rung 49, exactly as § 5.18 finding 1's 255-combination sweep predicted. Rung 49's own
file could not make this measurement — its gate is the one that IS named correctly.

**FINDING 5 — THE BAR MARGINS SPLIT AGAIN, AND THE TIGHTEST BAR IN THIS FILE IS 5 %.** Slice T's
lesson, third outing. Measured on the suite's own cells before the 15/15 was accepted:

| gate | bar | worst measured | slack |
|---|---|---:|---:|
| 4 | `peak/at_star > 2.5` | `2.625` | **1.05 × — 5 %** |
| 4 | `past/peak < 0.6` (collapse) | `0.472` | 1.27 × |
| 9 | `last/peak < 0.6` (the same collapse) | `0.472` | 1.27 × |
| 4 | peak `s_off > 2·s_hp*` = `1.28` | `1.56` | 1.22 × — **one grid cell** |
| 3 | `hits >= 8` (non-vacuity) | `11` | 1.38 × |
| 5 | worst `s_off < s_hp*` = `0.40` | `0.30` | 1.33 × |
| 3 | `\|s_min − s_rel\| <= 3·ds` | `0.0200` | 3.00 × |
| 10 | `r = 0.5` `ds` drift `< 0.05` | `0.0115` | 4.34 × |
| 10 | `r = 2.0` `ds` drift `< 0.02` | `0.0066` | 3.05 × |
| 9 | `\|nu_hp_end − bare\| < 5e-4` | `5.75e-6` | 87 × |
| 5 | `\|relief_lp − (phi_lim − min_phi_lp_bare)\| < 1e-5` | `8.88e-16` | 1.1e10 × |
| 7 | `relief_lp == 0.0` | `0.0` exactly | structural |
| 10 | `\|s_min_lp − s_off\| <= ds` | `0.0000` at BOTH `ds` | structural |

**Gate 4's `2.625` against a `2.5` bar is the tightest thing in the file** — a 5 % error in
`relief_hp` at either of two rows breaks it and nothing looser would notice, tighter even than
step 1's gate 9 at 11 %. Two of the bars are structurally EXACT rather than tight, and that
distinction matters: gate 10's relocation offset is `0.0000` at both `ds` values on all six rows
(the minimum lands ON the imposed release point), and gate 7's `relief_lp` is an `x − x` zero, the
same mechanism as rung 48's six sites and rung 49's gate 9.

**FINDING 6 — A DOCSTRING NUMBER IS HALF STALE, AND THE SEARCH IS STATED BECAUSE SLICE T's
SAME-NUMBERED FINDING WAS DIAGNOSED BACKWARDS.** Gate 10's docstring says the two ramp rates
converge at *"0.7 % against 2 %"*. Measured on the gate's own cells at its own settle: **0.656 % at
`r = 2.0` and 1.151 % at `r = 0.5`**. The first reproduces; the second does not. Four alternatives
were tried before the correction was written:

| candidate | `r = 0.5` (quoted 2 %) |
|---|---:|
| the gate's own cells, `relief_hp`, max — **what the gate computes** | **1.151 %** |
| the same at `settle = 4.0` | 1.151 % — settle-INVARIANT |
| at the tighter floor `0.7400` | 0.872 % |
| `relief_lp` instead of `relief_hp` | 7.808 % |
| gate 5's wider offset set | 11.544 % |

None lands on 2 %. The likeliest origin is the gate's own `tol` for `r = 2.0`, which IS `0.02` —
a bar leaking into prose beside a measured figure — but that is offered as a hypothesis, not as
the found formula. Both figures clear the gate's bars (`0.05` and `0.02`) and **no gate reads
either quoted number**. Third outing in this port for the rung-63 lesson.

**FINDING 7 — A MODULE CONSTANT IS DECLARED AND NEVER READ, IN PYTHON.** `S_LP_STAR = 0.240` on
`test_rung50.py:71` occurs exactly **once** in the file's code — its own binding — measured by
stripping docstrings and counting word occurrences. It is kept in the port under `#[allow(dead_code)]`
with the measurement in its doc comment, because the port is a copy and not a tidy-up. The whole
cost is the `allow`; the whole benefit is that **Rust says so and Python never does**.

**FINDING 8 — FOURTEEN INJECTIONS, AND THE GATE THAT DEFENDS THE RELEASE EDGE SWEEPS THE ONLY
CELLS THAT ARE NOT ON THE KNIFE EDGE.** The battery is the slice's standing instrument, re-run per
step. Twelve injections were pre-registered with predictions BEFORE the harness ran once
(`u2_injection_predictions.md`), and two more were added by what round one measured.

| injection | moved a value? | keys | gates that caught it |
|---|---|---:|---|
| **A** the march coordinate spelled `k * ds` | yes — **`n_engaged` 8→7 and `s_rel` by a WHOLE CELL** | 546 | **NONE** |
| **M** the release fires one cell late (`s <= s_off`) | yes — 16 armed-window keys | 94 | **gate 10 ALONE** |
| **C** `deficit_at_release` divides by `mf` not `mf_sched` | yes | 49 | **NONE** |
| **J** `deficit_at_release` reads `eng[0]` not `eng[-1]` | yes | 49 | **NONE** |
| **D** `relief_watched` / `relief_other` swapped | yes | 80 | **NONE** |
| **H** `nu_hp_end_bare` read off the LIM march | yes | 49 | **NONE** |
| **G** `nu_hp_end` read off the BARE march | yes | 49 | contract 1b alone |
| **B** `fuel_removed`'s trapezoid loses its `0.5` | yes, ×2 | 49 | contract 1b alone |
| **B′** the SAME defect in ALL THREE copies in the module | yes | — | **NONE — rungs 48/49/50 all green** |
| **K** `min_phi_lp_lim` / `_bare` swapped | yes | 90 | contract 1b, gate 5 |
| **E** `s_min_lp` / `s_min_hp` swapped | yes | 46 | gates 3, 10 |
| **L** `s_min_lp` reads the BARE argmin | yes | 43 | gates 3, 10 |
| **F** `s_eng` / `s_rel` swapped | yes | 98 | contract 1b, gates 3, 4, 6, 8 |
| **I** the engaged mask loses its `1e-9` slack | **NO — inert on all 49 cells** | 0 | — |

**THREE PRE-REGISTERED PREDICTIONS WERE WRONG, WHICH IS THE POINT OF WRITING THEM DOWN FIRST.**

**A was predicted CAUGHT and is not, and the mechanism is exact.** Finding 2 measured a one-CELL
knife edge and the prediction followed straight from it: two of gate 5's and gate 10b's cells sit
on it, so a `k * ds` spelling must be visible. It moves the edge exactly as measured —
`s_off = 0.26` at `rho = 0.25` and at `rho = 4.0` both go `n_engaged` 8 → 7 with `s_rel` from
`0.25999999999999995` to `0.23999999999999999`. **And no gate sees it.** `gate10` is this file's
only reader of the release edge's LOCATION (`|s_min_lp − s_off| <= ds`), and it catches the same
one-cell shift when that shift is written directly at the comparison site — injection M, which
gate 10 catches ALONE. But gate 10 sweeps `0.30 / 0.40 / 0.44` and `1.10 / 1.56`, and **not one of
those is a knife-edge cell**; the two gates that DO sweep the knife-edge cells read only "some
`relief_lp < 0`" and "the worst row is upstream of `s_hp*`", neither of which a one-cell shift
flips. **The defender and the exposure are on disjoint cells** — a shape neither a bar-margin
table nor a reader census would have produced, because both instruments are blind to WHICH CELLS a
reader reads.

**AND "546 KEYS MOVED" NEEDED A SCALE BEFORE IT COULD BE QUOTED.** Step 1's lesson was that a
"0 gates caught it" row needs a *did it move a value* column; this is that column needing a
magnitude. Of A's 546: **499 moved by less than `1e-9`** (the coordinate feeds the fuel schedule at
every RK4 stage, so everything drifts a little), 32 by more than `1e-3`, and **74 are armed-window
keys**. The honest claim is the 74 and the two `n_engaged` flips, not the 546.

**B AND G WERE PREDICTED UNGATED AND ARE CAUGHT — BY A REDUCE GATE, NOT A READER.** Slice T step 3
measured every reader of `fuel_removed` in the project to be scale-invariant, and that is still
true: gates 8 and 9 read it only through strict orderings. What catches the doubled trapezoid is
`contract1b`, which compares `release_relief(s_off=None)` against `surge_relief` field for field —
a CROSS-READER identity between two independent copies of the same integral. **The reduce spine
turns out to be a VALUE gate wherever a rung re-implements its predecessor's quantity**, which is
not what the reduce spine is for and is a real second job it does.

**BUT ONLY AS A DIFFERENCE, AND B′ MEASURES THE LIMIT.** Breaking all three identical trapezoids in
the module at once leaves rungs 48, 49 and 50 **all green**. So the protection is exactly
"the two copies agree", never "the value is right" — § 5.18 step 1 finding 4's two-spelling shape
INVERTED: there two faults were needed to MOVE a value, here two are needed to HIDE one. G's twin
H makes the same point from the other side: `nu_hp_end` off the bare march is caught and
`nu_hp_end_bare` off the lim march is not, and the ONLY difference between them is that Python's
ten-key list in contract 1b happens to name one and not the other.

**I MOVED NOTHING, AND THAT IS A STATEMENT ABOUT THE GRID.** It was registered with a question
mark for exactly this reason. No point on any of the 49 cells sits in
`[mf_sched·(1 − 1e-9), mf_sched)`, so the slack is unreachable and dropping it is inert — not a
gate's achievement.

**AND THE SITE-COUNT GUARD FIRED TWICE MORE.** `K`'s first anchor matched **3** sites (all three
readers spell `min_phi_lp_bare: mpl_b,` identically) and `M`'s was indented four spaces too deep
and matched **0**. Both printed `PATCH SITE COUNT n != 1 — NOT APPLIED` instead of running and
reporting a green. That is slice T step 4's lesson (*an injection matching twice applies nothing
and still reports green*) paying for the second slice running, and it is the only reason `K` and
`M` are not two more "NONE" rows above — `M` in particular would have been a badly wrong one,
since it is the ONE injection gate 10 catches.

**THE UNGATED SURFACE AFTER STEP 2**, derived from the table rather than guessed, and therefore the
sizing for step 5's oracle: **`deficit_at_release`** (both its value and its `eng[-1]` choice),
**`relief_watched` / `relief_other`**, **`nu_hp_end_bare`**, **`fuel_removed`'s shared SCALE** (as
opposed to the two copies' agreement), and **the march coordinate's spelling on the knife-edge
cells**. Five things, all measured live.

**PREDICTION STATUS.** **P1 holds in step 1's direction again** — no defect found in slice S's
plant, but a defect surface in this step's own new code that the gates cannot see. **P2 CONFIRMED
BY MEASUREMENT** at its second of four data points (finding 4). **P6 is on track and checkable at
step 3**: `release_relief` landed complete with `tau_rel`, and Python's `rate_sweep` and
`deficit_curve` are both plain loops over it — read in the source, to be gated when step 3 adds
zero new logic. **P3/P4/P5** need rungs 51/52.

##### STEP 3 — SHIPPED. **P6 DISCHARGED, AND THE ONE FUNCTION THE STEP ADDS IS EXERCISED ONLY ON CELLS CHOSEN FOR INERTNESS**

`fuel_transient.rs` gains `rate_sweep` and `deficit_curve`: **65 lines added and ZERO deleted**.
`tests/rung51.rs` is **16 test fns for Python's 16** and runs **16/16 in 0.91 s on the first
compile**, with Python's `_ROWS` memo as a `Mutex<HashMap>` keyed on BITS.

**§ 5.18 P6 IS DISCHARGED BY MEASUREMENT, NOT BY ASSERTION.** P6 was registered before rung 50's
step was written: `release_relief` would land COMPLETE with `tau_rel` at rung 50 because `tau_rel`
is its kwarg and not a separate path, so rung 51's two readers would be LOOPS over it. That is
exactly what shipped — **two `map` bodies and one assert, no new field, no new branch, no new
constant** — and the 65-line diff with no deletions is the check, the way slice T checked its
zero-source-line steps.

**FINDING 1 — 972 KEYS OVER 36 CELLS, BIT-EXACT ON THE FIRST RUN, AND THE CELL LIST WAS READ OFF
THE SUITE RATHER THAN ENUMERATED.** P1's distrust of a clean pass again. The probe **runs all
sixteen gates and then dumps whatever ended up in `test_rung51.py`'s own `_ROWS` memo** — 29 cells
— plus the two `rate_sweep` calls that bypass it and three `deficit_curve` rows. A hand-written
cell list is this port's standing way of under-covering a suite (slice S step 4); reading the memo
cannot miss a cell a gate visits and cannot invent one it does not. Zero differences against PyPy.

**AND THE MEMO CENSUS SAID SOMETHING THE GATES DO NOT.** All **29** memoised cells run at
`r = 2.0`. `_rel`'s defaults are `phi_lim = PHI_LIM_2` and `r = R2`, where rung 50's `_sweep`
defaults the other way, so a rung-51 gate that reads as ramp-rate-agnostic is a `r = 2.0`
measurement. The only `r = 0.5` cells in the file are contracts 1b and 4, which build their calls
by hand.

**FINDING 2 — THE BAR MARGINS, AND ONE OF THEM IS 5 %.** Measured on the suite's own cells before
the green was accepted:

| gate | bar | worst measured | slack |
|---|---|---:|---:|
| 3 | faded `>1.4×` shallower | `1.470×` | **1.05 × — 5 %** |
| 7 | relief gap `> 0.01` | `0.0131` | 1.31 × |
| 11 | `\|s_min_hp(0.02) − s_min_hp(0.01)\| <= 0.02` | `0.0100` | 2 × — **exactly ONE cell at `ds = 0.01`** |
| 7 | matched deficit `< 1e-3` | `2.17e-4` | 4.6 × |
| 11 | `relief_lp` `ds` drift `< 1 %` | `0.078 %` | 12.8 × |
| 11 | `relief_hp` `ds` drift `< 1 %` | `0.092 %` | 10.9 × |
| 3 | the two brackets AGREE within 1 % | `0.073 %` | 13.7 × |
| 10 | `\|nu_hp_end − bare\| < 5e-4` | `2.85e-6` | 176 × |
| 6 | `relief_lp == 0.0` | `0.0` exactly | structural |
| 8 | the location law | 5–10 GRID CELLS clear | — |

Gate 3's `1.470×` against a `1.4×` bar is the tightest thing in the file, the same class as rung
50's gate 4 at `2.625` against `2.5`. Two files running, the slice's tightest bar is a **ratio in
a headline claim**, not a tolerance — which is the opposite of slice T's picture and consistent
with step 2's.

**FINDING 3 — FOUR INJECTIONS, ALL FOUR PREDICTIONS HELD, AND THE INTERESTING ROW IS SHARPER THAN
PREDICTED.** Only two functions are new, so the battery is four rows rather than fourteen. All
four were written down before the harness ran once:

| injection | moved a value? | keys | gates that caught it |
|---|---|---:|---|
| **I1** `rate_sweep` DROPS `tau_rel` (always forwards `None`) | yes — **but only 2 of 972** | 2 | **NONE** |
| **I4** `deficit_curve` watches the HP spool regardless | yes | 45 | **NONE** |
| **I3** `rate_sweep`'s `tau_rel >= 0` assert deleted | **NO — unreachable** | 0 | — |
| **I2** `rate_sweep` forwards `s_off` as `None` | (the plant refuses) | — | contracts 1b **and** 4 |

**I1 IS THE FINDING, AND THE "2 KEYS" IS WHAT MAKES IT ONE.** Two moved keys is a suspiciously
small number for "the rate axis stopped being forwarded", and the difference between *the physics
was inert* and *the physics moved and nothing read it* is the whole content of the row — so it was
measured rather than reasoned. The two keys are:

```
rate_sweep/c4[1]  tau_rel  0.04 -> None
rate_sweep/c4[2]  tau_rel  0.32 -> None
```

**the record echoing its own argument back, and nothing else.** Every physical quantity was
already bit-identical. The reason is structural and is the sharp version of the claim: of the four
`rate_sweep` rows the whole suite produces, **exactly two carry a live `tau_rel`, and both are
contract 4's — whose entire claim is that `tau_rel` is INERT there** (the trigger sits past the
natural release, so there is no clip left to fade). Every gate that reads a genuinely faded march
calls `release_relief` directly through the memo helper. So **`rate_sweep`'s one job is exercised
only on cells chosen for inertness**, and the sole witness that the forwarding happened at all is
the echoed parameter. Step 5's oracle owes a `rate_sweep` cell INSIDE the window — derived from
this table, not guessed.

**I2 IS CAUGHT BY A PLANT REFUSAL, NOT BY A VALUE COMPARISON**, and the prediction was half wrong:
it was registered as "caught by contract 1b and NOT contract 4". Both fired — because
`s_off = None` with a live `tau_rel` trips `integrate_fuel`'s own `tau_rel` needs `s_off`
composition assert before any number is produced. Recorded because "two gates caught it" reads as
two independent checks and is one refusal reaching two call sites.

**I3 MOVED NOTHING, AND THAT IS ABOUT THE GRID.** No gate passes a negative `tau_rel`, so
`rate_sweep`'s only assert is unreachable from the suite — a fifth unreachable refusal in this
slice, beside § 5.18 finding 1's three `lp_disabled` ones and rung 50's `s_off > 0`.

**THE CRATE CHECK FOR THIS STEP IS NAMED RATHER THAN IMPLIED.** `cargo clippy --all-targets` is
back at **73** warnings — step 2's measured baseline — with **none at any line this step touched**;
the diff is a **pure addition** (65 insertions, 0 deletions, verified with `git diff --stat`); and
rungs 49/50/51 run **17/15/16 green**. **A full `cargo test --release` was NOT re-run here** — step
2's was 809/0/0 over 89 binaries, this step changes no existing executable line, and the full run
is taken once more after step 4. That is the whole of what was checked, and nothing finer should
be read into it.

##### STEP 4 — SHIPPED. **A SUITE WHOSE SUBJECT IS INVARIANCE IS STRUCTURALLY BLIND TO VALUES, AND THAT EXPLAINS THREE OF ITS FOUR HOLES AT ONCE**

`fuel_transient.rs` gains the 34-field `LagRelief`, `FactorizationGrid`, and `lag_relief` /
`lag_sweep` / `factorization_grid`: **294 lines added and ZERO deleted**. `tests/rung52.rs` is
**15 test fns for Python's 15** and runs **15/15 in 0.75 s on the first compile** — including all
four of the slice's `slow` gates, so **no `#[ignore]`**, decided by slice M's rule on a MEASURED
in-suite cost rather than inherited from Python's marks.

**FINDING 1 — 972 KEYS OVER 18 MEMO CELLS AND TWO FACTORIZATION GRIDS, BIT-EXACT ON THE FIRST
RUN.** The cell list is read off `test_rung52.py`'s own `_ROWS` after running all fifteen gates,
as step 3's was, and the two `factorization_grid` calls are rebuilt explicitly because their
DERIVED objects — `residual`, `credit_spread`, `max_residual`, `max_main_effect` — are the rung's
headline and no `lag_relief` row carries them. Zero differences against PyPy.

**§ 5.18 P5 HOLDS EXACTLY.** `credit_spread` is asserted `== 0.0` **bit-for-bit, with no
tolerance**, on both grids, and passes. `tau_att` owns the credit exactly in Rust as in Python.

**FINDING 2 — TEN INJECTIONS, AND THREE OF THE FOUR UNCAUGHT ONES HAVE THE *SAME* CAUSE.**

| injection | moved a value? | keys | gates that caught it |
|---|---|---:|---|
| **C** `g_at_cross` / `required_at_cross` swapped | yes | 56 | **NONE** |
| **I** `min_phi_hp_lag` / `min_phi_hp_bare` swapped | yes | 56 | **NONE** |
| **G** `residual` drops its `+ d00` term | yes | 12 | **NONE** |
| **E** `max_main_effect` drops its SECOND max | yes | 2 | **NONE** |
| **D** `s_eng_<eps>` / `s_rel_<eps>` swapped | yes | 108 | gates 1, 7 |
| **F** `credit_spread` folds the COLUMN not the row | yes | 4 | gates 3, 4 |
| **J** `lag_sweep` becomes COLUMN-major | yes | 117 | gates 3, 4 |
| **A** the `armed` seed becomes `false` | **NO — inert on all 18 cells** | 0 | — |
| **B** the `g <= 0` arm DISARMS instead of skipping | **NO — inert on all 18 cells** | 0 | — |
| **H** `g_peak` folds only the clipped points | **NO — inert on all 18 cells** | 0 | — |

**THE HEADLINE IS THAT `C` AND `I` WERE BOTH PREDICTED CAUGHT AND ARE NOT, FOR ONE REASON.** Gate
1 does read `g_at_cross` and gate 8 does read `min_phi_hp_lag` — but neither reads a VALUE. Gate 1
asserts `|g_at_cross − g_at_cross[0]| < 1e-3 · g_at_cross[0]` **across a `tau_rel` sweep**, and
gate 8 asserts `|min_phi_hp_lag − prev| < 1e-4` **across a `ds` sweep**. Both are comparisons of a
key against ITSELF AT ANOTHER CELL, and a defect applied uniformly moves every cell together and
leaves the comparison untouched.

**A GATE THAT READS A KEY ONLY BY COMPARING IT WITH ITSELF CANNOT SEE WHAT THE KEY IS.** That is
the *invariance* analogue of slice T step 3's *scale-invariance* finding, and it is not an
accident of this file: **rung 52's whole subject IS invariance** — `tau_rel` does not move the
crossing, does not move the credit, does not move the engagement edge — so its gates are
structurally invariance-shaped, and a suite built to prove that nothing moves is maximally blind
to everything being wrong by the same amount. Slice T's lesson was that every READER of a quantity
can be scale-invariant; this is the same hole reached from the suite's THESIS rather than from its
readers.

**AND THE TWO REGISTERED TRAPS IN THE CROSSING LOOP ARE BOTH DEAD.** § 5.18 finding 2 measured the
`armed` seed to be a dead distinction on every marched cell and registered the `g <= 0`
continue-versus-disarm as a second trap "so the port does not tidy it", without knowing whether it
bites. Measured here: **neither moves a single key on any of the 18 cells**. Both guard clauses in
those eight lines are individually unreachable on this whole grid — § 5.18 step 1 finding 3's `D`
and `H` rows repeating on a different reader, and the reason both spellings stay in the source is
*COPY vs REDERIVATION*, not evidence. Only a MANUFACTURED trajectory separates them, and that is
step 5's.

**`E` AND `G` ARE THE OTHER SHAPE: A BAR THAT ONLY GETS EASIER.** Gates 3 and 4 assert
`max_residual > 0.4 · max_main_effect`. Dropping the second `max` from the denominator makes the
denominator SMALLER, so the ratio RISES and the bar is cleared more comfortably; dropping the
`+ d00` interaction term changes the residual by 12 keys and still clears it. A one-sided bar
cannot see an error in the direction it already allows — which is why the oracle holds
`max_residual` and `max_main_effect` as VALUES rather than trusting the ratio.

**FINDING 3 — THE BAR MARGINS, AND GATE 8's IS EXACTLY ONE CELL BY CONSTRUCTION.** Measured on
the suite's own cells:

| gate | bar | worst measured | slack |
|---|---|---:|---:|
| 3 | `max_residual > 0.4 · max_main_effect` | `0.589` | 1.47 × |
| 3 | `\|ratio drift\| > 0.05` | `0.0738` | 1.48 × |
| 4 | the same bar at `r = 0.5` | `0.650` | 1.63 × |
| 1 | `\|Δg_at_cross\| < 1e-3` rel | `5.27e-4` | 1.90 × |
| 8 | `\|Δs_cross\| <= 2·ds` | `0.0200` / `0.0100` | **2.00 × — EXACTLY one grid cell, both times** |
| 8 | `\|Δmin_phi_hp_lag\| < 1e-4` | `5.74e-5` | 1.7 × |
| 6 | the accel COMPLETES `< 1e-5` | `1.10e-6` | 9.1 × |
| 1 | `s_rel_0.01` moved `> 0.5` | `1.180` | 2.36 × |
| 1 / 3 / 4 | `s_cross`, `relief_watched`, `credit_spread` invariant | `0.0` exactly | structural |
| 2 | the location claims | 4–52 GRID CELLS clear | — |

Gate 8's `2.00 ×` is not slack in the usual sense: the crossing moves **exactly one grid cell per
halving** at both steps, which is the docstring's own "resolution limit of *first recorded point
with `required < g`*" reproduced to the cell. The bar allows two cells and the phenomenon uses
one.

**THE UNGATED SURFACE AFTER STEP 4**, derived from the table: **`g_at_cross`**,
**`required_at_cross`**, **`min_phi_hp_lag` / `min_phi_hp_bare`** (values, not their invariance),
**`max_main_effect`'s second `max`**, **`residual`'s `d00` term**, **`g_peak`**, and **both dead
distinctions in the crossing loop**. Every one of the first six is emitted as a VALUE by step 5's
oracle; the last two need manufactured trajectories and are gated there.

**THE CRATE CHECK.** `cargo clippy --all-targets` at **73**, step 2's baseline, none at any line
this step touched; the diff is a pure addition (294 insertions, 0 deletions); rungs 49/50/51/52
run **17/15/16/15** green. The full `cargo test --release` is taken once at step 5.

##### STEP 5 — SHIPPED. **THE SLICE ENDS BY GATING WHAT ITS 63 GATES COULD NOT SEE, AND ONE OF ITS OWN GATES WAS NEARLY VACUOUS**

`rust/oracle/dump_release.py` + `tests/release_oracle.rs`: **4 179 keys over eight sections,
bit-exact against PyPy on the first run — and bit-exact against CPython too, at 0 float drifts and
0 discrete flips.** Five gates. **SLICE U IS COMPLETE, AND WITH IT PHASE 6.**

**THE ORACLE'S CONTENTS ARE DERIVED, NOT CHOSEN.** Every section answers a specific row of the
four injection batteries:

| ungated by the 63 gates | measured at | held as |
|---|---|---|
| `deficit_at_release` — the rung's OWN named quantity | step 2, C and J | a VALUE, §§ B/C |
| `relief_watched` / `relief_other` out of `release_relief` | step 2, D | a VALUE, § B |
| `nu_hp_end_bare` off the LIMITED march | step 2, H | a VALUE, §§ B–H |
| `fuel_removed`'s SCALE (held only as a DIFFERENCE by contract 1b) | step 2, B and B′ | a VALUE everywhere |
| the march coordinate's SPELLING on the knife-edge cells | step 2, A | `s_rel` / `n_engaged` BITS, § G |
| `rate_sweep` with a LIVE `tau_rel` | step 3, I1 | an ADDED cell, § F |
| `g_at_cross`, `required_at_cross`, `min_phi_hp_lag` — read only as INVARIANCES | step 4, C and I | VALUES, § D |
| `max_main_effect`, `residual` — behind a ONE-SIDED bar | step 4, E and G | VALUES, § D |
| `g_peak` | step 4, H | a VALUE, § D |
| the two dead distinctions in the crossing loop | step 4, A and B | **MANUFACTURED** trajectories |

**THREE SECTIONS ARE ADDED RATHER THAN PORTED, AND EACH IS LABELLED SO A SUPERSET CANNOT PASS AS A
PORT.** § E reaches **both NaN arms for the first time in this port** — § 5.18 finding 4 measured
the minimum `n_engaged` at **10** over every rung-49 floor cell and **2** over every rung-50 `s_off`
cell, never zero, so a dormant floor and a first-point release are added. The dump confirms it:
`n_engaged = 0` and `s_eng` = `7ff8000000000000` on both. § E also **gates the two-sentinel
ambiguity** rather than only describing it — with nothing engaged the record returns
`s_eng_is_nan = 1` **and** `deficit_at_release = 0.0`, and `0.0` is a legitimate deficit. § F adds
the `rate_sweep` cell inside the window that step 3 measured missing; § G pins the knife-edge
cells' accumulated coordinate at `0.19999999999999998` and `0.25999999999999995`.

**FINDING 1 — ONE OF THIS STEP'S OWN GATES WOULD HAVE COMPARED MY FORMULA WITH MY FORMULA, AND
CATCHING IT COST A BEHAVIOUR-NEUTRAL EXTRACTION.** The two dead rules in `lag_relief`'s crossing
loop need a manufactured trajectory, and the first writing built one against a **re-spelled copy of
the loop inside the test file**. That gate would have passed forever while the shipped loop said
something else — rung 70's *a gate computing my own formula twice*, which this project has already
been caught by once, arriving on the gate written to close the slice.

The fix is the one `first_raw_min`'s own doc comment records for the identical situation one rung
earlier: **lift the loop into a callable `crossing_census`** and let the gates hold the shipped
code. The diff is 30 deleted lines and their replacement, the oracle's 4 179 keys re-ran bit-exact
across it (which is what makes "behaviour-neutral" a measurement rather than an intention), and a
FIFTH gate — `the_reader_and_the_manufactured_gates_share_one_census` — asserts on a real marched
cell that `lag_relief`'s `n_recross` and `s_cross` come out of the very function the two
manufactured gates exercise. Without that gate the extraction could rot: a later edit could give
the reader its own loop again and the manufactured gates would keep passing on the orphan.

**AND THE MANUFACTURED CELLS DO SEPARATE THE SPELLINGS**, which is the whole point:

| trajectory | shipped `crossing_census` | the wrong spelling |
|---|---:|---:|
| first clipped point ALREADY past the crossing | `n_recross = 0` | `armed = false` seed gives **1** |
| a DORMANT point inside an armed run | `n_recross = 1` | disarming on it gives **2** |
| the shape every marched cell HAS | `1` | `1` — **they agree**, which is why no cell can hold this |

**FINDING 2 — THE CPython ARM MOVED NOTHING, AND THAT IS REPORTED AS A CONFIRMATION AND NOT AS A
DETECTION.** All 4 179 keys are bit-identical between PyPy and CPython 3.14: **0 float drifts, 0
discrete flips.** Every cell in this oracle is CPG — all four suites build `_cpg_gas()` — so this
is the prediction confirmed rather than a hole found, and it is worth saying plainly that **a
detector reporting zero has demonstrated no sensitivity on this grid**. Slice T's arm had TPG
sections that genuinely moved and could therefore quote a measured sensitivity; this one cannot,
and does not. What it does establish is that nothing in the four readers' arithmetic is
interpreter-dependent, which is the CPG expectation this port has held since phase 3. **No count
was registered in advance** — five typed count bars in this port, five wrong.

**FINDING 3 — TWO DOC CORRECTIONS AND ONE DEFECT WRITTEN UP WHERE THE NEXT READER HITS IT.**

* `factorization_grid`'s docstring said the separability residual comes back at "62-70 % of them
  at both ramp rates measured", and `test_rung52.py` gate 4 said "70 % at `r = 0.5` against 62 % at
  `r = 2.0`". **Re-measured independently at this step** rather than copied from § 5.18's
  pre-registration — **58.9 % at `r = 2.0` / `ds = 0.02` and 65.0 % at `r = 0.5` / `ds = 0.01`** —
  and the pre-registered figures reproduced exactly. Both corrected in place, with the settings
  they were taken at spelled out, because the original's error was omitting them. Both clear the
  gate's `0.4` bar and no gate reads either figure.
* § 5.18 finding 1's **three unreachable `lp_disabled` refusals** are now written up **at the
  degenerate block in `turbojet/engine.py`**, not only in this file — slice O's rule, *the note
  goes where a future reader will hit it*. The comment states the measurement (zero of 255 arming
  combinations reach them), the mechanism (arming `s_off`/`tau_rel`/`lag` requires an armed leg,
  and the `accel`/`surge` refusals precede them), the consequence (four gates named for four rungs
  are ONE claim), and **why they stay**: the ordering above them is not a contract, so a future leg
  whose composition assert lets `s_off` through changes the reachability. Kept and documented, not
  deleted — and the Rust copies all three, which is *COPY vs REDERIVATION*.

**§ 5.18's PREDICTIONS, FINAL STATUS.** **P1** held in step 1's direction at every step: no defect
was found in slice S's plant, and four batteries found defect SURFACES in the slice's own new code
that the gates cannot see. **P2 CONFIRMED BY MEASUREMENT on all four of its data points** — rungs
49/50/51/52's `lp_disabled` gates all fire the rung-49 assert, and rungs 50/51/52 assert the FULL
string so it is measured rather than inferred. **P3 sharpened** at step 1 (the one-ulp boolean is
protected by two spellings at once) and **extended** at step 2 (the forced-release comparison site
is a one-CELL knife edge, a coarser hazard at a different site). **P4 held**: `n_recross` passes
under both `armed` seeds on every marched cell, and only this step's manufactured cell separates
them. **P5 held EXACTLY**: `credit_spread == 0.0` bit-for-bit on both grids, no tolerance.
**P6 discharged** at step 3: `rate_sweep` and `deficit_curve` are two `map` bodies and one assert.

### 5.19 PHASE 7 PRE-FLIGHT — the hook table ENUMERATED, and both prior lists refuted

**AUTHORISED 2026-08-20** ("start phase 7 preflight"), which is the fresh authorisation § 5's
row 7 said was owed. **The pre-flight ALONE** — § 9's pattern, as at phase 5: measure, land the
evidence, and re-decide before a line is ported.

Phase 7 is the last code phase and the largest: **rungs 57–60 and 62–84**, one module per rung,
and the `Hooks` table § 2 has been promising since the spike. Because the phase is *terminal*
and *linear* — 23 classes in one chain, nothing crossing forward — the inheritance census of
§ 5.3 / § 5.12 answers a different question here. It is no longer *"what must the previous phase
ship hookable"*; it is **"what are the table's FIELDS"**. Nine probes,
`M:\claud_projects\temp\rust-phase7\probe_p7{a,b,c,d,e,f,g,h,i}.py`, PyPy.

#### (i) THE LEADING FINDING — **§ 2 SAYS EIGHT, § 5.12 SAYS SIX, AND THE MEASURED ANSWER IS 38**

§ 2's table names eight hooks (`_close_fuel`, `_instant_fuel`, `_cap_fuel`, `_close`,
`_shared_rig`, `_stator_march`, `integrate_fuel`, `at_lever`) with call-rate columns. § 5.12's
*measured* crossing list names six (`integrate_fuel`, `_close`, `_close_fuel`, `_surge_fuel`,
`_instant_tail`, `_powers`). **The intersection is three.** § 2's table came off the synthetic
depth-28 spike and had never been enumerated against today's 23 066-line `engine.py`; § 5.12's
list was scoped to *names crossing the phase boundary*, which is not the same set as *names the
table must carry*.

Enumerated (probe 1: a name `N` is a hook iff some class `A` reaches `self.N` and some `D` with
`A` in `D.__mro__` redefines `N`): **38 names.** § 3's *"the trait is ~8–10 methods, not 40"* is
refuted at its own lower bound — it guessed the wrong end of its own range, and § 2's **rule 1**
(hooks take `&Config`, never a positional knob list) gets *heavier* at that width, not lighter.

**BUT AN OVERRIDDEN NAME IS NOT AUTOMATICALLY A CELL, AND THE FIRST WRITING OF THIS SECTION DID
NOT CHECK — § (xi).** The discriminator, measured over all 38 (probe 6): does the override
**change behaviour**, or does it only re-type and copy fields forward? A `PURE-FORWARD` override
changes the *Python type* and nothing else, and in Rust the leaf's type is not a type at all — the
table `h` rides through and `Config { .., ..self.cfg }` carries the fields — so it needs no cell.
The census **partitions exactly**:

| | n | |
|---|---|---|
| **NEEDS A NEW CELL** | **28** | every override BEHAVIOURAL |
| already a cell in a shipped table | 8 | `match`, `at_setting`, `_solve_turbine`, `_hp_eta_loop`, `_lp_eta_loop` (phase 5); `_close`→`try_close`, `_instant_tail`→`try_instant_tail`, `_powers`→`powers` (phase 6) |
| **DELETED — Rust needs no cell** | **2** | `at_lever` (17 overriders) and `at_stator` (2), the pure sibling constructors of § (iii) |
| | **38** | **= 28 + 8 + 2, and the final table is 36 fields** |

The two deleted names are exactly the ones § (iii) is about, so the section is now consistent with
itself; `_closer` — which the first slice table listed as a cell — is **defined exactly once** and
is not in the 38 at all (§ (v)).

**And `_instant_fuel` — one of § 2's eight — IS NOT A HOOK.** It is defined in exactly two
classes, `SpoolTransient` (r34) and `TwoSpoolFuelTransient` (r43), which are **siblings**. That is
precisely the bug § 5.12's own census made and caught. § 2's table has carried it since the spike.

#### (ii) SIXTEEN `super()` SITES ARE NOT ZERO-ARGUMENT, AND `..R63` CANNOT EXPRESS THEM

83 `super()` sites; **16 are `super(LimitedBleedTransient, self)`**, spread over rungs 65–75, and
15 of the 16 reach `_close_fuel` (the last reaches `_close`). Resolved against every leaf in the
chain: **all 16 land on `ScheduledBleedTransient` — rung 62 — regardless of depth.** They are a
*static pin to one ancestor*, not "the parent".

§ 2's whole spelling is `const R64: Hooks = Hooks { close_fuel: r64_close_fuel, ..R63 }` and
`r64_close_fuel` calling `r63_close_fuel`. Nine rungs down the chain, **"the parent" and "the pin"
are different functions**, and a port that writes the former is wrong in a way nothing type-checks.
The correct shape is a *direct* call to `r62_close_fuel` — and the note that has to be at the call
site, not in this paragraph:

> **The pin is on the FUNCTION, never on the TABLE.** Python's `super(C, self).m` is still bound
> to `self`, so every `self.X` inside rung 62's body still dispatches to the **leaf**. In Rust
> that is `r62_close_fuel(h, …)` with `h` the LEAF table. `r62_close_fuel(&R62, …)` compiles,
> runs, and silently freezes the ladder at rung 62 — the exact failure § 1 records the
> compile-time-generics arrangement producing (0.018 % off, clean build, no error).

18 `super().X` sites are **VALUES**, not calls (16 `_close_fuel`, 1 `_close`, 1 `shared_bill`) —
§ 2's `closer(parent, …)` shape, confirmed live at the measured width.

#### (iii) **§ 2's "RUST DELETES `at_lever`/`_shared_rig` OUTRIGHT" IS REFUTED — MEASURED**

§ 2 claims all 26 field-forwarders (18 `at_lever` + 8 `_shared_rig` — the count is exactly right)
collapse to `Config { vsv_lp, ..self.cfg }`, *"and forgetting a field is not expressible"*. Read
body by body, that is true only of rungs 62–72. **From rung 73 the shape changes**: the method
constructs the sibling with its nine lever kwargs and then **post-assigns private attributes that
are not constructor parameters at all.**

| forwarder | out-of-band fields it must remember |
|---|---|
| r73 `at_lever` | `_ref_law` |
| r74 `at_lever` | + `_lag_coord` |
| r75 `at_lever` | + `_windup_law`, `_tau_t`, `_ic_cap` |
| r76 / r77 `at_lever` | + `_cap_law` |
| r78 `at_lever` | + `_gauge_k` |
| r79 `at_lever` | + `_phi_ref` |
| **r80 `at_lever`** | **nine, all of them** — `_sm_air` last |

**10 distinct attributes over 45 assignments** (`_gov_max` is the tenth, at rung 72's
`_shared_rig`). *That second set is what rung 80's docstring calls "THE EIGHTEENTH INSTANCE of the
trap"* — the trap is not forgetting a constructor argument (the constructor would have raised); it
is forgetting one of these silent post-assignments. § 2's claim survives in substance — a Rust
`Config` struct update carries both sets at once, so the trap is still inexpressible — but **not as
written**: the deletion is sound only once the ten out-of-band fields are *in* `Config`, and § 2
does not know they exist.

#### (iv) THE PHASE'S REAL STRUCTURAL CONTENT — **23 RELOAD GUARDS OVER 19 DYNAMICALLY-SCOPED FIELDS**

Rung 80's `_with_air` docstring calls itself *"rung 62's reason, and this family's THIRTEENTH
reload of it"*. The shape: save a field off `self`, **set** it, call a reader, restore it in a
`finally`. That is **dynamic scope over `self`** — and it is the thing in phase 7 that has no
Python-shaped answer in Rust, because a `&self` method cannot do it and the value is read *deep
inside marched hook bodies*, so it cannot simply become a parameter either.

Measured (probe 7, after the repair in § (xi)): **52 guards, 23 distinct fields.** Classified by
the enclosing function of **every** assignment — the question that decides the hook signature, and
the one this pre-flight nearly skipped:

| kind | n | fields |
|---|---|---|
| **CONFIG** — set only outside a march | 12 | `_cap_law` `_gauge_k` `_gov_max` `_ic_cap` `_lag_coord` `_phi_ref` `_ref` `_ref_law` `_share_law` `_sm_air` `_tau_t` `_windup_law` |
| **STATE** — *also assigned inside a march* | **9** | `_b0` `_b_forced` `_b_state` `_ic_order` `_lag` `_tau_gov` `_v0` `_v_forced` `_v_state` |
| **already ported**, phases 5/6 | 2 | `rho` (r40), `bleed` (r42) |

**The nine are neither config nor state — they are the CURRENT RK4 STATE COMPONENT, passed down by
dynamic scope so the hook signatures do not have to change.** Rung 68's `_integrate_fuel_triple` is
the clearest reading of it: `self._b_state, self._v_state = q, v` immediately before
`self._instant_fuel(...)`, and `= None, None` in the `finally`. `b_of` (r64), `_close` and
`_close_fuel` (r65) — **all three hook cells** — read them. The `None` is load-bearing: a reader
distinguishes *inside the trial* from *outside* it.

**AND THE PORT ALREADY HAS TWO ANSWERS TO THIS SHAPE, NEITHER OF THEM A `Scope`.** `rho` (r40) was
ported by **threading it as a parameter** into the one reader that needs it (`jacobian_at_rho`);
`bleed` (r42) was ported by giving `bleed_trade` **`&mut self`** and mutating the core field in
place — Python's shape exactly, with the `finally` as a straight-line restore. Both work because
those two readers sit at the TOP of the call chain. The split below is what decides which
precedent each of the 21 phase-7 fields takes.

**DECIDED HERE, NOT IN A SLICE — and it closes § 6's "narrowed config view vs `&Config`" question
at the same time, because they are ONE decision.**

- **The 12 CONFIG-kind take phase 5's `&mut self` precedent.** They are set outside every march, so
  the guard is a `&mut self` method that assigns, calls the reader, and restores — no new
  parameter, and **nothing in the hook table changes for them.**
- **The 9 STATE-kind take a `Scope` parameter**, because they are assigned *inside* an RK4
  derivative evaluation where `self` is behind `&self` in a hook chain and `&mut self` is not
  available. ~~*(As written.)*~~ **SUPERSEDED FOR `_b_forced` AT SLICE X — § 5.22 (v)/(vii).**
  Measured at rung 64: the set→read chain is **one** frame and that frame is `try_close_fuel`, a
  SHIPPED cell with a live table and a dispatch gate, so `&Scope` is non-additive there; and
  `_b_forced` provably never NESTS, so a `Cell<Option<f64>>` + an RAII guard whose `Drop` restores
  keeps every signature AND is stronger than the `finally` it ports. Retired for that one field
  only: `_b_state`/`_v_state` DO have a same-field nest candidate, at slice AH. `Scope` is a small by-value struct of `Option<f64>` / `Option<&str>`; Rust's scoping
  *is* the `finally`, so the restore stops being a discipline and becomes structural — strictly
  stronger than the Python.

**Which cells take the `Scope` parameter, measured rather than assumed (probe 7): SEVEN of the 36**
— the cells that read a **STATE-kind** field. `_arm`, `_close`, `_close_fuel`, `_stator_march`,
`b_of`, `integrate_fuel`, `v_of`. Fifteen cells read a scoped field at all, and the other eight of
those read only CONFIG-kind ones, which live in `Config` and need no parameter. So the table does
**not** need one uniform signature and § 2's rule 1 survives for 29 of 36 cells.

**THE EXCEPTION, AND § (x) MUST SAY SO: `_close` IS `TwoSpoolTransientHooks::try_close` — a SHIPPED
PHASE-6 CELL WITH A LIVE `R40` TABLE AND A DISPATCH GATE.** Adding `&Scope` to it is **NOT
ADDITIVE** — the distinction `two_spool.rs:53` already draws deliberately when it chose to add a
field rather than change a call site. It is the only such case: `match` and `at_setting` also read a
reloaded field, but that field is `bleed`, which phase 5 resolved by the `&mut self` route and which
needs nothing here. **Slice V's scope therefore includes a signature change to already-gated phase-6
code**, beside the two additive cells it owes.

And it answers § 6's row: the three signature-absence tests (rungs 71/72/73 assert `s_off`/`tau_rel`
are **absent** from `_stator_march`) are satisfied by `Scope` simply not carrying those two fields —
a compile error, not a runtime assertion, and `_stator_march` is one of the 11 that takes it. § 6's
fallback to `include_str!` is **not needed** for those three.

#### (v) THE MECHANICAL SWEEPS — one live shadow pair, and a hazard that measured ZERO twice

- **Constant shadows: 2 phase-7 pairs, both live.** `_LAG_OK` `False`→`True` (r64→r65) and
  `_ref_law` `'sched'`→`'applied'` (r72→r73). Both are read off `self` by inherited bodies, so
  both must be per-cell parameters and **not literals** — phase 5's `_INC_MAX` lesson at 24
  levels' depth. 33 class constants in total are read off `self` inside phase-7 bodies.
- **Template-method hazard: 0 real sites** — as in phase 6. The first detector reported **343
  sites over 43 names** and was measuring nothing: it built "supplied" from class-scope
  definitions, so every instance attribute an ancestor's `__init__` assigns looked unsupplied.
  Re-scoped over every `self.X =` along the MRO it fell to **51 sites over 9 names**, and all 51
  are in the six **`@dataclass` helpers** (`StatorSchedule`, `BleedSchedule`, `IncidenceLimiter`,
  `StatorLimiter`, `StatorIncidenceLimiter`, `BleedLimiter`), whose fields are bare annotations the
  detector still could not see. **Two detector defects, one behind the other, on the arm that
  measured zero** — recorded because a zero from a blind instrument and a zero from a live one read
  identically.
- **`_closer` is settled, and the varargs are cosmetic.** Defined **once**, at rung 64
  (`LimitedBleedTransient`, 10055), signature `(self, method, *args)`; **16 call sites, every one
  passing exactly five positional arguments** `(nu_lp, nu_hp, mf|Tt4, Tt2, pt2)`. It ports as a
  concrete five-argument function taking a `fn` — no varargs machinery. § 5.12 flagged it to this
  phase; it costs nothing.

#### (vi) THE SCOPE LIST, ENUMERATED — **and the helper block is defined OUT OF RUNG ORDER**

Slice K's lesson for the third time (*a scope list is a claim about a SET, and it is only as good
as an enumeration over that set*). Phase 7 is **29 objects**: 23 chain classes + 6 helpers. Two
chain classes cover four and two rungs — `ScheduledStatorTransient` **is** rungs 57/58/59/60,
`ScheduledBleedTransient` **is** 62/63 — so a slice-per-rung would again write partial bodies that
never existed (§ 5.12's `integrate_fuel`-entire decision, one level up).

And the helper block at **9696–9930**, sitting between rung 63's class and rung 64's, is:

| object | line | rung |
|---|---|---|
| `StatorLimiter` | 9696 | **68** |
| `StatorIncidenceLimiter` | 9764 | **69** |
| `BleedLimiter` | 9863 | **64** |

**Three objects, three rungs, in non-monotonic order** — a slice defined as "everything between
these two classes" pulls rungs 68 and 69's objects into the rung-64 slice. This is the same shape
as § 5.12's `4257–4506` block (`IncidenceLimiter` is rung 60), found again and larger.
`StatorBleedMatcher` (8478) sits inside the phase-7 line range and is **rung 61, phase 5's**,
shipped.

#### (vii) THE RUNTIME-INTROSPECTION TESTS — **§ 6 NAMES FOUR; THERE ARE EIGHT**

| site | asserts | replacement |
|---|---|---|
| `test_rung71.py:241`, `test_rung72.py:414`, `test_rung73.py:477` | `s_off`/`tau_rel` **absent** from `_stator_march`'s signature | **`Scope` — § (iv).** Compile error, not assertion |
| `test_rung73.py:488` | `src.count("g_own + req - clip") == 1` | `include_str!` + `.matches().count()` — § 6, verified in the spike |
| **`test_rung71.py:243`** | `"s_off is None and tau_rel is None" in getsource(integrate_fuel)` | `include_str!` — **not in § 6** |
| **`test_rung73.py:492`** | `psrc.count("self._reference(") == 4` | `include_str!` — **not in § 6** |
| **`test_rung71.py:190`** | `"_integrate_fuel_cross_triple" not in FullSplitTransient.__dict__`, and the two are the **same function object** | **fn-pointer equality on the two tables' cells** — strictly stronger, and structural — **not in § 6** |
| **`test_rung79.py:133`** | `G.__code__.co_consts is not None` | **NOTHING — the assertion cannot fail.** § (ix) |

#### (viii) SIZING, HONESTLY — **THE ROW'S "5–8 SESSIONS" IS OFF BY ABOUT 3×**

| | phase 6 | **phase 7** | ratio |
|---|---|---|---|
| source lines (class spans) | ~3 540 | **15 362** | **4.34×** |
| test **functions** (`def test_`) | 156 | **488** | 3.13× |
| **tests COLLECTED** by pytest | **157** | **548** | **3.49×** |
| test files | 15 | **27** | 1.80× |
| test lines | 5 036 | **11 032** | 2.19× |
| **collected tests carrying `slow`** | **10 (6.4 %)** | **263 (48.0 %)** | **26×** |

**The `slow` row counts the right noun, on the second attempt.** It was first written as *"219 of
488"*, which is a count of `pytest.mark.slow` **decorator occurrences** over a count of
**functions** — one marker on a parametrized function is one occurrence and N collected tests, so
the ratio was a function-level number stated as a test-level one. Re-measured with
`--collect-only -m slow` on both phases: **263 of 548** here against **10 of 157** there. *A counter
is only as good as the noun it counts* — this index's own lesson, on this section.

Phase 6 shipped in **six slices**, several of them five steps. Phase 7 at 4.3× the source and 3.1×
the tests is **~15 slices** (§ (x)) and — labelled an estimate — **15–20 sessions**, not 5–8. It is
the largest phase in the port by a factor of four; say so rather than discover it.

**263 `slow` gates is a MEASUREMENT OWED, and it is no longer cheap.** Slice M's rule stands —
port the gate, drop the marker, re-introduce `#[ignore]` only against a **measured** Rust cost —
but at 6.4 % that rule cost nothing to apply and at 48.0 % it decides whether the phase's gate
runs in minutes or hours. Measure it at the first slice that ports a `slow`-heavy suite, not at
the last.

**AND THE PHASE TABLE'S GATE IS WEAKER THAN EVERY PHASE SINCE 1.** The row reads *"27/27
reduce-to-prior bit-exact"* — a reduce spine and nothing else. Every shipped phase gated an
**oracle** (values, bit-exact vs PyPy, plus a CPython arm) **beside** the ported rung suites.
What phase 7 actually owes, written here so the row cannot be read as the bar:

1. a **per-slice oracle dump**, bit-exact vs PyPy, with the CPython arm on the gases that admit it;
2. the **548 ported gates**, 27 suites;
3. the **27 reduce-to-prior contracts** the row does name; and
4. **dispatch gates** — `slice_r_dispatch.rs`'s precedent. A hook table is the one thing **no value
   key can witness**: swap a cell, assert a value breaks. At 36 cells this is the phase's
   signature instrument, not an extra.

#### (ix) TWO ITEMS RECORDED, NOT FIXED — slice N step 6's precedent

§ 8 puts repairing the oracle's own tests outside the port, so both are written with their
measurement and left alone.

- **`test_rung79.py:133` cannot fail.** `assert G.__code__.co_consts is not None` — `co_consts` is
  a tuple on every code object, never `None`. The comment beside it (*"a closure over `phi_lim`,
  not over `1/phi_lim`"*) names a real property that the assertion does not test. It is this
  family's own recurring shape: **a gate that reads a key only in a way that cannot distinguish
  its values.**
- **THREE COMMENTS IN THE SHIPPED RUST CALL RUNG 55 "PHASE 7", AND SLICE N SHIPPED IT.**
  `src/two_spool.rs:48` (*"`_hp_eta_loop`/`_lp_eta_loop` overridden by rung 55, phase 7"*),
  `src/two_spool.rs:53` (*"rung 55's override reads `self.stack_hp`, a phase-7 field"*) and
  `src/bleed.rs:146` (*"since rung 55 overrides that same slot in phase 7"*). All three were
  written at slices K/L when rung 55 was ahead; `stage.rs:707`'s `R55_TWO` has occupied both slots
  since phase 5 slice N. Slice L step 4's lesson — *a claim in the SHIPPED source was false* —
  recurring in the same file, and the reason the sweep for phase-7 deferrals had to read code
  rather than the plan.

**THE DEFERRAL INBOX, CODE-RESIDENT, ASSIGNED.** Three ⚠ notes were written into
`fuel_transient.rs` by slices S/T, at the definition rather than only into a section — slice O's
rule. Each is booked to the slice that discharges it:

| owed item | recorded at | slice |
|---|---|---|
| `try_close_fuel` needs a cell — 4 classes override it | `fuel_transient.rs:1532` | **V** (rung 57) |
| `try_surge_fuel` needs a cell — `ScheduledStatorTransient` overrides it | `fuel_transient.rs:1856` | **V** (rung 57) |
| `integrate_fuel` needs a cell — 11 classes override it | `fuel_transient.rs:2010` | **Y** (rung 65) |

#### (x) THE SLICE PLAN — **ordered by where the TABLE GROWS, not by rung number**

Fifteen slices. **Step 1 of every slice is the cell addition**, so a slice that forgets a cell
fails at its own first gate rather than at a value key nine rungs downstream.

**The cell column below is EMITTED by probe 6, not hand-written** — § (xi) is the reason. A cell
must EXIST at the slice porting its earliest **caller**, and is SWAPPED at every slice porting an
**overrider**; the two rules together are what make the column sum to 28 by construction instead of
by counting.

| slice | rungs | classes | cells the slice ADDS |
|---|---|---|---|
| — | — | **phase-6 EDIT, caller r43** | **`_close_fuel`, `_surge_fuel`, `integrate_fuel`** — exactly the three code-resident ⚠ notes in `fuel_transient.rs`, opened by **V** (first two) and **Y** (the third) |
| **V** | 57–60 | `StatorSchedule`, `IncidenceLimiter`, `ScheduledStatorTransient` | **3** — `_arm`, `_stator_march`, `v_of`; **+ the two phase-6 cells above, + `try_close`'s NON-ADDITIVE `&Scope`** (§ (iv)) |
| **W** | 62–63 | `BleedSchedule`, `ScheduledBleedTransient` | **4** — `_armed_bleed`, `_isolating`, `_legs`, `b_of` (+ swaps `_instant_tail`/`_powers`) |
| **X** | 64 | `BleedLimiter`, `LimitedBleedTransient` | **1** — `b_at_point`. **MEASURED 1 at § 5.22 (iii) — the first row of this column the emitter confirms.** `_closer` is **not** a cell (defined once); this is the slice of **the rung-62 PIN**, § (ii) |
| **Y** | 65 | `LaggedBleedTransient` | **0** — but it opens phase 6's `integrate_fuel` cell, and the `Scope` STATE fields go live |
| **Z** | 66–67 | `TwoLagCascadeTransient`, `CrossLoopCascadeTransient` | **0** — swaps only |
| **AA** | 68 | `StatorLimiter`, `ThreeLoopCascadeTransient` | **9** — `_check_v0`, `_clamp_v`, `_lagged_stator`, `_manifold_v`, `_rk4_floor`, `_solve_v`, `_stator_leg`, `_triple_laws`, `_triple_rig`. **The widest step, and it is rung 68 rather than 69** — rung 68 is the CALLER of all nine and rung 69 the overrider |
| **AB** | 69 | `StatorIncidenceLimiter`, `ReferenceSplitTransient` | **1** — `_with_ref` |
| **AC** | 70–71 | `CrossSplitTransient`, `FullSplitTransient` | **1** — `split_gains` |
| **AD** | 72 | `SharedActuatorTransient` | **3** — `_reference`, `_rk4_floor_shared`, `_shared_rig` |
| **AE** | 73 | `AppliedReferenceTransient` | **0** |
| **AF** | 74 | `DemandCoordinateTransient` | **3** — `_cap_fuel`, `_sensed_cap`, `_windup_tau` |
| **AG** | 75–76 | `AntiWindupTransient`, `SensedCapTransient` | **0** |
| **AH** | 77–78 | `StiffnessLedgerTransient`, `ResidualGaugeTransient` | **0** |
| **AI** | 79–80 | `StateCoordinateTransient`, `SplitWallTransient` | **0** |
| **AJ** | 81–84 | `AuthorityClock…StaircaseLawTransient` | **0** — the reader-only rungs |
| | | | **25 + 3 phase-6 = 28** |

**Slice V is the risk, and for a reason the first writing of this table did not have: it is the one
slice that CHANGES A GATED SIGNATURE.** Opening `_close_fuel` and `_surge_fuel` in
`fuel_transient.rs` is additive; giving `TwoSpoolTransientHooks::try_close` a `&Scope` is not, and
that cell has a live table and a dispatch gate behind it. **AA is the widest** at nine cells.
**Nothing after AD adds more than three** — the table is essentially complete two thirds of the way
in, which is what makes the back half grinding rather than risky.

#### (xi) **FOUR DEFECTS IN THIS SECTION'S FIRST WRITING, AND EVERY ONE CAME FROM HAND-WRITING A TABLE THE PROBES COULD HAVE EMITTED**

The measurements in (i)–(x) were correct. The **tables built from them** were transcribed by hand,
and that step — not the measuring — is where all four defects are. Recorded rather than quietly
repaired, because the pattern is the point.

1. **The 38 was never filtered by *does this need a cell at all*, and the slice table contradicted
   § (iii) two screens above it.** `_closer` was listed as a cell slice X adds — § (v) of the same
   section measures it **defined exactly once**, so it has no overrider and is not in the 38. And
   `at_lever` / `at_stator` were listed as cells while § (iii) concludes Rust **deletes** them. The
   arithmetic gave it away before the reading did: the column summed to **31** against a stated
   ~30. **Emitting the map (probe 6) makes it reconcile by construction** — 38 = 28 + 8 + 2 — which
   is § 5.10 step 4's own lesson (*emit and compare, do not restate*) applied one level up, to a
   plan section instead of to a census.
2. **The `Scope` decision was presented as free and is NOT, for one cell.**
   `TwoSpoolTransientHooks::try_close` is shipped, has a live `R40` table and a dispatch gate, and
   reads `_b_state`. § (x) now says so. The section had the additive-vs-not distinction available —
   `two_spool.rs:53` draws it deliberately — and did not apply it to its own decision.
3. **`_ic_cap` fell between two probes and was classified by neither.** probe 2 found it as an
   out-of-band `at_lever` field; probe 3 was supposed to classify it and did not, because probe 3
   scanned only a function's **top-level** statements for the save/restore pair and
   `contraction_law`'s guard is nested inside a `for`. Re-run with `ast.walk` (probe 7), the guard
   census goes from **23 guards / 19 fields to 52 / 23** — `_b_forced`, `_ic_cap`, `_v_forced` and
   `bleed` were all invisible to it. **The headline shape survived the repair and doubled**, which
   is the only reason the defect was cheap.
4. **The `slow` ratio counted decorators and called them tests** — § (viii), corrected there.

**The common cause is one thing: every one of the four is a claim that a probe could have PRINTED
and a human typed instead.** Where the probe printed it (the 38, the 16 pins, the CONFIG/STATE
split) the section was right; where a table was assembled by reading probe output and writing prose
(the cell column, the `Scope` cost, the union of two field lists, the `slow` ratio) it was wrong
four times out of four. **The rule for the phase, not just for this section: if a slice's table can
be emitted, emit it.**

~~**STOP HERE.** The pre-flight is landed; **phase 7 itself is NOT authorised** and no line is
ported until it is.~~ **PHASE 7 AUTHORISED 2026-08-20** — § 5.20 is slice V, § 5.21 slice W, § 5.22 slice X
(rung 64, PRE-REGISTERED), and § 5.20 **REFUTES § (x)'s reading of slice V**: no cell in rungs 57–60 reads a dynamically-scoped field, so the
`&Scope` on `try_close` is **slice Y's**, not V's. What V does carry is a shape this pre-flight's
`try/finally` census could not see — see § 5.20 (i).


### 5.20 SLICE V (rungs 57–60, `ScheduledStatorTransient`) — PRE-REGISTERED, six probes MEASURED first; **ALL FIVE STEPS SHIPPED**

**PHASE 7 AUTHORISED 2026-08-20** ("start phase 7"), which is the fresh authorisation § 5's row 7
and § 5.19's closing line both said was owed. The pre-flight landed the same day and is § 5.19;
this is the first porting slice, and it does **not** match § 5.19 (x)'s description of itself.

Six probes, `M:\claud_projects\temp\rust-phase7\probe_p7{l,m,n,o,p}.py` +
`plugin_scoped_arm.py`, PyPy. **Every table below is EMITTED** — § 5.19 (xi)'s rule, which is the
one rule this phase inherited rather than discovered.

#### (i) THE LEADING FINDING — **A FIELD MUTATED PERMANENTLY FROM INSIDE A `&self` HOOK, AND § 5.19's CENSUS COULD NOT SEE IT**

§ 5.19 (iv) classified the phase's dynamic-scope problem by exactly one shape: **save a field,
set it, call a reader, restore it in a `finally`.** 52 guards, 23 fields, and a `Scope` parameter
on 7 of 36 cells. Rung 57's `_arm` is a **different shape and is not in that census**: it assigns
`self.map_lp` / `self.map_hp` and **never restores them**.

| | writes | in `__init__` | in `_arm` |
|---|---|---|---|
| `map_lp` | 6 | 4 (rungs 39/40/53/57) | **2 — rung 57 and rung 68** |
| `map_hp` | 5 | 4 (rungs 39/40/53/57) | **1 — rung 57** |

and the field is read by **21 functions over rungs 39–62** (`map_lp`, 28 reads) and **19**
(`map_hp`, 26 reads) — among them `_close`, `_instant_tail`, `_powers` and `ev`, which in Rust are
**shipped `&self` cells with a live `R40` table and two dispatch gates**. A `&self` method cannot
do this, and the value is read inside a marched hook chain, so it cannot become a parameter
either. **This, not `Scope`, is slice V's structural content.**

The source states the problem in two places and never connects them. `_arm`'s docstring: *"A pure
function of (nu_L, nu_H, Tt2) — no history, no latch, so it is RK4-legal."* `v_of`'s: the readers
go through it *"rather than through `self.map_*`, which `_arm` leaves at whatever the LAST
sub-step happened to be."* Both are true only if **no reader outside the close path ever observes
the stale map** — a claim about REACHABILITY, not about purity, and neither docstring makes it.
Slice I's lesson (*a bare `except` makes the question REACHABILITY*) on a pair of claims instead
of on a handler.

#### (ii) **MEASURED: THE STALE MAP IS REACHED, IT MOVES THE RUNG'S OWN HEADLINE NUMBER BY 15 %, AND ALL 59 GATES ARE BLIND TO IT**

The port's natural shape for rung 57 is a **locally-armed core**: `try_close` builds the armed
maps, passes them down, and the caller's core is untouched. That is Python's `_arm` with the
mutation **scoped** to the close call. So Python was made to behave that way — save the maps, run
the shipped body, restore — and the two modes compared. `plugin_scoped_arm.py`, over the
rungs-57/58/59/60 suites:

| | |
|---|---|
| `_close` + `_close_fuel` calls | **920 262** (30 514 + 889 748) |
| of those, left `map_lp` mutated | **208 125** |
| of those, left `map_hp` mutated | **0** — the suites arm no HP schedule returning non-zero |
| **gates green, BOTH modes** | **59 / 59** |

**59/59 either way is the weak instrument** — this port's own repeated lesson. Probe 5 reads the
READERS instead, after a march, and compares the BITS:

| arming | key | baseline (stale) | scoped (design) | rel % |
|---|---|---|---|---|
| lp_only | `SM_lp` | 0.06080308471 | 0.05798678588 | **4.632** |
| lp_only | `margin_min_lp` | 0.1140020369 | 0.113471511 | 0.465 |
| hp_only | `margin_min_lp` | 0.09232122145 | 0.08518277881 | **7.732** |
| hp_only | `SM_hp` | 0.4404934501 | 0.43011312 | 2.357 |
| both | `SM_lp` | 0.06087379962 | 0.05798678588 | **4.743** |
| both | **`margin_min_lp`** | 0.04623412535 | 0.03909986668 | **15.431** |
| **const_lp** (a CONSTANT setting, not a schedule) | every key | — | — | **0 — no difference at all** |

`margin_min_lp` is the transient LP surge margin — **rung 57's own currency**, the quantity its
headline is stated in. A local-armed-core port moves it by 15 % and **every one of the 59 gates
passes**. That is the third time this port has found a suite blind to a large error (slice T
finding 2 at 24 %, slice U step 2's whole-cell `s_rel`), and the first time the blind spot is in
the OBJECT'S OWN STATE rather than in a reader's coordinate.

**The constant-setting arm is EXACTLY zero** because rung 53's constant is applied in `__init__`;
only a SCHEDULE reaches `_arm`. So the reduce contract and the staleness sit on disjoint arms,
which is why no reduce gate can witness this either.

#### (iii) **WHY THE MARCH ITSELF IS BIT-IDENTICAL — AN ALGEBRAIC INVARIANCE, NOT AN UNREACHED PATH**

The trajectory is bit-for-bit identical in both modes, over all four arming configurations. That
could be *"the intra-march readers sit inside the close extent"* or *"they are never called"* — a
zero from an unexercised path reads like a zero from an inert one, so it was measured (probe 6,
one march):

| reader | inside the close extent | **OUTSIDE it, on a STALE map** |
|---|---|---|
| `_instant_tail` | 0 | **687** |
| `_powers` | 0 | **36** (+ 9 on the design map) |

So the stale map **is** read 723 times mid-march, outside the extent — and the march does not
move. The reason is provable rather than measured: **every one of those reads is `eta_t_at`**,
`with_vsv` is `replace(self, vsv=vsv)`, and `eta_t_at` is `base - a_t*(nu_t-1)^2`. **`vsv` cannot
reach `a_t`.** The two shipped cells that read the map inside the march are invariant to the
arming *by construction*.

**That is what makes slice V portable at all**, and the general form is worth stating: a field can
be stale on 723 reads and inert on all of them, because staleness bites only through the CHANNELS
the mutation actually drives — here `psi` and `phi_surge_at`, exactly the pair `with_vsv`'s own
docstring names.

#### (iv) THE CARRIER, DECIDED — **`Cell<ComponentMap>`, and it is NOT the `Scope` § 5.19 designed**

- **`try_instant_tail` and `powers` need NOTHING.** § (iii) is the licence, and it is algebraic.
  Two shipped, gated cells left alone.
- **`try_close` / `try_close_fuel` must PERSIST the arming**, because § (ii) shows the post-march
  object state is observable at 15 %. `ComponentMap` is `#[derive(Clone, Copy)]` (`map.rs:192`),
  so the carrier is **`Cell<ComponentMap>` on the core** — `&self`-compatible, no `RefCell` borrow
  panic, and Python's shape exactly: set, never restore.
- **NO `&Scope` anywhere in slice V**, and the reason is measured, not assumed — § (v).

**AND THE CARRIER'S PRICE, MEASURED BEFORE IT WAS BUILT** — because a carrier stated without a
price is § 5.19 (xi)'s defect 2 recurring on the section that records it. `map_lp` / `map_hp`
are `pub` fields on **`TwoSpoolMapCore` (`two_spool.rs:1209`) — rung 39, PHASE 5** — and turning
them into `Cell` reaches:

| | sites |
|---|---|
| `src/two_spool_transient.rs` | 11 |
| `src/two_spool.rs` | 8 |
| `src/stator.rs` · `src/fuel_transient.rs` | 7 · 7 |
| `src/bleed.rs` · `src/stage.rs` | 6 · 4 |
| **`src/` total** | **43** |
| `tests/` | 16 |

Six source files over **three phases**, all mechanical (`ComponentMap` is `Copy`, so every read
is `.get()`), and the field keeps ONE spelling by becoming an accessor `map_lp(&self)` rather
than a bare `Cell` at 43 call sites. **RECONCILED AT STEP 1b: 43 and 16 count raw occurrences
including COMMENT mentions; the CODE sites are 40 and 12** (`stator.rs:316`, `rung45.rs:70` and
`fuel_transient_oracle.rs:1327` are prose about Python's `self.map_lp`). The estimate was made
with a `grep -o` that had no comment filter and the rewrite used one, so the two disagree by
exactly the seven comment lines — stated rather than left as two numbers for one thing. **That is the real cost of slice V, and it is not the cost
§ 5.19 named.** It is still the right carrier — it is Python's shape exactly — but the slice is
"the risk" for this reason, a third distinct one.

#### (v) **§ 5.19 (x)'s "SLICE V IS THE RISK" IS REFUTED AT ITS STATED REASON**

§ 5.19 (x) says slice V *"is the one slice that CHANGES A GATED SIGNATURE"* — giving
`TwoSpoolTransientHooks::try_close` a `&Scope`. § 5.19 (iv) had measured *which cells* read a
STATE-kind field and never measured **at which RUNG the first such reader appears**. Emitted
(probe 1):

| cell | first override that READS a STATE-kind field | rung | **slice** |
|---|---|---|---|
| `b_of` | `LimitedBleedTransient` | 64 | **X** |
| `_close` | `LaggedBleedTransient` | 65 | **Y** |
| `_close_fuel` | `LaggedBleedTransient` | 65 | **Y** |
| `_stator_march` | `LaggedBleedTransient` | 65 | **Y** |
| `integrate_fuel` | `TwoLagCascadeTransient` | 66 | **Z** |
| `_arm` | `ThreeLoopCascadeTransient` | 68 | **AA** |
| `v_of` | `ThreeLoopCascadeTransient` | 68 | **AA** |
| `_surge_fuel` | — | **NEVER** | — never takes one |

**Not one of them is slice V**, and rungs 57–60 read **no dynamically-scoped field at all** —
neither of the 9 STATE-kind nor of the 12 CONFIG-kind.

`Scope` also **cannot be defined at slice V**: its fields *are* the nine state-kind fields, whose
first reader is rung 64. Writing the struct here would be a placeholder with every field unknown
until slice X, threaded dead through V and W — and a dead parameter cannot be gated, which is
slice U step 3's finding (*a function exercised only on cells chosen for INERTNESS*).

**The label may survive; the reason does not.** Slice V is the risk because of § (i)–(ii) — a
carrier no census saw, protecting a number no gate reads — not because of a signature.
§ 5.19 (iv)'s *"the advisor blocked the slice plan on it and was right"* stands as written: the
CONFIG/STATE classification was the right measurement. It was applied to the wrong slice because
nobody asked **when**.

#### (vi) THE SCOPE, ENUMERATED

Three objects, and the block is again **out of rung order** — `IncidenceLimiter` is rung **60**
and sits at 4349, inside phase 6's line range, exactly as § 5.12 recorded.

| object | lines | rung | shape |
|---|---|---|---|
| `IncidenceLimiter` | 4349–4423 | **60** | `@dataclass(frozen=True)`, 2 fields, 6 methods |
| `StatorSchedule` | 7385–7426 | 57 | `@dataclass(frozen=True)`, 4 fields, 2 methods |
| `ScheduledStatorTransient` | 7429–8451 | 57/58/59/60 | 35 methods, 923 body lines, span 1023 |

`ScheduledStatorTransient` **is all four rungs** (§ 5.19 (vi)) — 31 names NEW, 4 OVERRIDES
(`__init__`, `_close`, `_close_fuel`, `_surge_fuel`), and **7 of the 35 are themselves overridden
later**, which is where the cells come from.

**THE CELLS SLICE V OPENS — on the predicate a cell is defined by** (*some ancestor reaches it AND
some descendant redefines it*), not on the narrower *rung 57's own bodies reach it*, which misses
`_close` and `_surge_fuel` because rung 57 defines them without calling them:

| cell | defined | overridden at | what slice V does |
|---|---|---|---|
| `try_close_fuel` | r43 | 57 62 64 65 | **OPEN** (the ⚠ at `fuel_transient.rs:1532`) + swap to `r57` |
| `try_surge_fuel` | r43 | **57 only** | **OPEN** (the ⚠ at `fuel_transient.rs:1856`) + swap to `r57` |
| `try_close` | r40 | 57 62 64 65 | **SWAP only** — the cell exists, `R40` ships it |
| `_arm` | **r57** | 68 | **NEW** |
| `_stator_march` | **r57** | 65 66 67 68 | **NEW** |
| `v_of` | **r57** | 68 | **NEW** |
| `at_stator` | r57 | 62 64 | **NO CELL** — a pure sibling constructor, § 5.19 (iii)'s delete |
| `integrate_fuel` | r43 | 65…76 | **NOT slice V's** — first overrider is rung 65 (slice Y) |

**3 new cells + 2 phase-6 openings + 1 swap**, which is § 5.19 (x)'s count with its reason
corrected and `at_stator` explicitly excluded rather than silently absent.

~~Adding two FIELDS to `TwoSpoolTransientHooks` touches **7 struct literals over 3 files**
(`two_spool_transient.rs`, `slice_r_dispatch.rs`, `slice_s_dispatch.rs`)~~ — **REFUTED AT STEP 1a,
and by this section's own error**: those two cells are rung **43**'s methods and belong in rung
43's table, not rung 40's. On the right table the literal count is **ZERO** and no dispatch gate is
edited. The paragraph inherited the wrong table from the `⚠` notes without asking what type the
cells' receiver is — see step 1a below. What survives: adding a FIELD is additive where adding a
PARAMETER changes 8 signatures, and § 5.19 priced the wrong one.

#### (vii) SIZING

| | |
|---|---|
| source | 1 023 + 42 + 75 = **1 140 lines** |
| tests | 4 files, 1 350 lines, 57 `def test_`, **59 collected** |
| **collected carrying `slow`** | **29 (49.2 %)** — the phase ratio, at the first slice |

§ 5.19 (viii)'s *"measure the `slow` cost at the first slice that ports a `slow`-heavy suite, not
at the last"* — this is that slice, and the measurement is owed at step 3.

#### (viii) PRE-REGISTERED — written before a line of Rust

- **P1.** `try_instant_tail` and `powers` need no change and no `Cell` read. § (iii) is algebraic,
  so this is a prediction about the PORT, not about the arithmetic: the shipped cells compile
  untouched and `slice_r_dispatch.rs` / `slice_s_dispatch.rs` stay green.
- **P2.** With `Cell<ComponentMap>` the post-march reader values reproduce Python **bit-exact**,
  including all twelve § (ii) baseline numbers. If any needs a tolerance, the carrier is wrong.
- **P3.** The two reduce arms are bit-for-bit by DISPATCH, not by arithmetic: unarmed returns on
  `_arm`'s first line, and a `v_max = 0.0` schedule hands back **the same map object** — in Rust,
  the same `ComponentMap` VALUE under `PartialEq`, since `Copy` has no identity. **The `is`-test in
  `test_reduce_zero_schedule_bit_for_bit_and_map_identity` does not survive the port as written**
  and must be re-gated as equality-plus-a-dispatch-count, or it becomes vacuous.
- **P4.** `map_hp` is never left mutated by the four suites (0 of 920 262). **The HP arm is
  therefore UNGATED in Python**, so the port's HP path is exercised by the oracle only if the dump
  adds an HP-scheduled cell. ~~It will.~~ *"It will" is a promise with no gate behind it* — booked
  instead as **step-4 checklist item (a)**: `dump_slice_v.py` MUST carry an HP-scheduled section,
  and the step-4 write-up states its key count. A deferral with a number attached survives; one
  with an intention does not.
- **P5 — the gate this slice owes.** A dispatch-style gate that manufactures the local-armed-core
  bug (restore the map after the close) and asserts `margin_min_lp` **breaks**. Without it the port
  ships a 15 % divergence that 59 ported gates cannot see. `slice_r_dispatch.rs`'s precedent, aimed
  at a CARRIER rather than at a cell.
- **P6a — A NOTE STEP 2 MUST NOT MISS.** `_arm` has **four** call sites: rung 57's `_close` /
  `_close_fuel` (7547, 7551) and **rung 62's** (8947, 9014). Rung 62 is slice **W**, so step 2
  ports two of the four — which means `_arm` must be built as **a cell**, never inlined into
  `r57_try_close`, or slice W re-opens it. § (vi) already books it as a cell; this is the reason.
- **P6.** Steps, on slice T/U's shape: **1** cells + carrier · **2** the port + `slice_v_smoke.rs`
  · **3** the four rung suites · **4** `slice_v_oracle.rs` + `dump_slice_v.py` · **5** the
  carrier/dispatch gates (P5) and the injections. **ALL FIVE STEPS SHIPPED — SLICE V IS
  COMPLETE.** Step 3 moved the
  injections forward — it ran six against the ported suites and MEASURED P5's necessity
  (the local-armed-core bug: 0 of 59 caught, Python's `SM_lp` numbers reproduced to three
  figures). ~~plus the one channel step 3 could not reach (`margin_min_lp`, a call-order
  question, target 15.4 %)~~ **STEP 4 REACHED IT** — section A reads rung 44's
  `transient_surge_margin` one call earlier in the chain and the move is **15.431 %** exactly, so
  all twelve of § (ii)'s numbers are reproduced. ~~**Step 5's remaining job is the MANUFACTURED
  gate alone**, and it is still owed~~ — **SHIPPED**: `slice_v_dispatch.rs`, 6 gates, which
  manufactures the locally-armed-core carrier inline in Rust and catches it with **four of its
  six assertions reading nothing on disk at all**. The `--dispatch` injection run flips it on
  I1 (4 of 6 gates, the four predicted exactly), I2, I3 and I6, and MISSES I4/I5 — reported as
  misses, because a carrier gate that fired on everything would not be measuring the carrier.

**STEP-4 CHECKLIST, written here so it cannot be skipped:**
**(a)** an HP-scheduled section in the dump — P4's promise, with a stated key count.
**(b)** **§ (ii)'s TWELVE NUMBERS BECOME DUMP KEYS.** P2 — the slice's central prediction — is
stated in twelve values that today live in a plan table and two files under
`M:\claud_projects\temp\`. That is not durable in the sense the rest of this port means it, and
it is this phase's own rule (*if it can be emitted, emit it*) applied to the numbers a prediction
is written in. `dump_slice_v.py` carries a post-march reader section — `SM_lp`/`SM_hp`,
`margin_min_lp`/`margin_min_hp`, over `lp_only`/`hp_only`/`both`/`const_lp` — so P2 is checked
against a committed TSV rather than against a table someone typed from a probe.

**Predictions that would REFUTE the plan rather than the port:** P1 failing means § (iii)'s algebra
is wrong and the two shipped cells are entangled after all; P2 failing means the stale map is not
the whole of the divergence and something else in the object is mutated too.


##### STEP 1a — SHIPPED. **THE TWO `⚠` NOTES BOOKED THEIR CELLS INTO THE WRONG TABLE**

The phase-6 cell openings, alone, before the carrier — so that a 43-site refactor lands on a tree
whose hook wiring is already gated green rather than on one where both are in flight.

**THE FINDING, AND IT WAS WRITTEN INTO THE SOURCE BY SLICES S AND T.** Both `⚠` notes book their
cell into **[`TwoSpoolTransientHooks`]** — `fuel_transient.rs:1536` (*"one of the three
[`TwoSpoolTransientHooks`] has no cell for"*) and `:1858` (*"the third of § 5.12's six crossing
names with no [`TwoSpoolTransientHooks`] cell"*). **That is the wrong table**, and § 5.19 (x)
repeated it. `TwoSpoolTransientHooks` is carried on `TwoSpoolTransientCore`; a cell typed
`fn(&FuelTransientCore, …)` there would make **rung 40's table name rung 43's type** and hand
every rung-40 object two cells it can never call. The two names are rung **43**'s methods, so they
take rung 43's own table:

- **NEW `FuelTransientHooks`** — `try_close_fuel`, `try_surge_fuel` — and **`R43`**, carried on
  `FuelTransientCore` beside the inherited `TwoSpoolTransientHooks` on `inner`. One table per
  composition level, as `stator.rs` carries both `StatorHooks` and `TwoSpoolHooks` rather than
  merging them, and as § 5.19 (x)'s own `..R63` spelling assumes.
- **`with_both_hooks`** — rung 57 swaps a cell in EACH table (`try_close` in rung 40's,
  `try_close_fuel`/`try_surge_fuel` in rung 43's), so the constructor takes both.

**AND IT MAKES THE OPENING CHEAPER THAN § 5.20 (vi) PRICED IT.** That paragraph said two new
FIELDS on `TwoSpoolTransientHooks` would touch **7 struct literals over 3 files**, including
`slice_r_dispatch.rs` and `slice_s_dispatch.rs`. On the right table the count is **zero** — the
phase-6 dispatch gates are not edited at all, and `git diff` over `rust/` touches exactly one
file. **P1's first half is therefore already measured, not predicted.**

**ZERO EXECUTABLE LINES CHANGED, VERIFIED BY MACHINE.** Both bodies moved out of the `impl` to
module-level `r43_try_close_fuel` / `r43_try_surge_fuel` — `r40_try_close`'s shape — and the
methods became one-line dispatches. The move was checked by re-deriving it: take each body from
`git show HEAD`, de-indent one level, rewrite `self.`→`ft.` and `Self::`→`FuelTransientCore::`,
and compare **string-for-string** against the shipped free function. **141 → 141 and 42 → 42
lines, both VERBATIM.** Slice N step 4's rule (*emit and compare, do not restate*) applied to a
refactor instead of to a census — an eyeball on a 141-line move is not evidence.

**THE PIPE THAT ATE THE GATE.** The first `cargo test --release` was run as `cargo test … | tail
-45`, so the harness reported **`tail`'s** exit status — `0` unconditionally — and kept only the
last 45 lines of ~90 suites. It happened to have passed. **A gate read through a pipe is not a
gate**, and this is the same shape as the port's own recurring finding one level down: an
instrument that cannot express the failure it is watching for. Re-run with the log captured whole
and cargo's own status read directly.

**THE GATE, READ PROPERLY:** `cargo test --release`, whole log, cargo's own status — **`CARGO_EXIT=0`, 94 suites, 845 passed, 0 failed, 0 ignored.** The `0 ignored` is slice M's rule still holding at the phase boundary. `slice_r_dispatch.rs` and `slice_s_dispatch.rs` are green **unedited**, which is **P1's first half MEASURED**.

##### STEP 1b — SHIPPED. **THE CARRIER, AND THREE DEFECTS IN INSTALLING IT — TWO OF THEM IN MY OWN INSTRUMENTS**

`TwoSpoolMapCore`'s two map fields become `Cell<ComponentMap>` behind four accessors
(`map_lp` / `map_hp` / `set_map_lp` / `set_map_hp`), and every read becomes a call. **52 sites
rewritten** — 40 in `src` over six files, 12 in `tests` over four — plus the two write sites in
`stator.rs`'s constructor. `ComponentMap` is `Copy`, so every read is a `.get()` and nothing is
cloned.

**AND THE FIRST VISIBLE CONSEQUENCE IS ONE LINE OF `stator.rs`.** Rung 53's constructor was
`let mut core = …`; the setter takes `&self`, so the `mut` is gone. That is not cosmetic — it is
the whole reason rung 57 can arm from inside a `&self` hook cell, showing up four rungs early in
a file slice M shipped.

**THE CARRIER HAS ITS OWN WITNESS, BECAUSE THE PHASE GATE STRUCTURALLY CANNOT BE ONE.** A green
`cargo test --release` after this refactor says the 52 rewrites were behaviour-neutral, which is
what it is for, and says **nothing** about whether the carrier works — nothing in the tree writes
through it during a march until step 2, and `set_map_lp` could be `{}` with only rung 53's
constructor noticing. That is slice U step 1's finding (*bit-exact and green says nothing about
GATE POWER*) arriving at this slice's own first step. `tests/slice_v_carrier.rs`, 2 tests, asserts
the property directly: **a write through a shared `&`, from a context that does not own the core,
persists — and a downstream reader that never saw the write sees the moved map.**

**AND THE WITNESS WAS MEASURED FOR GATE POWER RATHER THAN ASSUMED TO HAVE IT.** Three carrier
bugs manufactured, all three caught:

| injection | caught by |
|---|---|
| **no-op** `set_map_lp` — installed but dead | both tests |
| **aliased** cells — one write reaches both maps | **the separation bar ONLY** |
| **restoring** carrier — a `finally` Python does not have | both tests |

**THE THREE DEFECTS, ALL RECORDED RATHER THAN QUIETLY FIXED.**

1. **A GUESSED BAR, and it is this index's own recurring shape.** The witness first asserted the
   HP margin bit-for-bit under an LP arming — *"rung 53's P5 zero, one ladder early"* — and it
   moved by **2 ULPs**. Rung 53's zero is a claim about the STEADY matcher's own lever; here the
   LP arming shifts the LP work and the HP operating point follows through the cascade. Re-run as
   a **measurement first**: `d_lp_rel = 5.632730e-1`, `d_hp_rel = 3.220528e-16`. **Fifteen orders
   of magnitude, and that SEPARATION is the assertion** — it is the only one of the three that
   catches the aliased-cells injection, which a bare `assert_ne!` on the LP side cannot see. The
   guessed bar would have failed on the clean tree AND passed the aliased bug's LP half.
2. **A HAND-TYPED FILE LIST, in the phase whose rule is "if it can be emitted, emit it".** The
   first rewrite pass ran over nine files I listed by reading an earlier grep. It missed
   `tests/rung56.rs`, and the release build caught it — but only because a private field is a
   compile error. **The list was the wrong instrument; a scan was available and I typed instead**,
   which is § 5.19 (xi) recurring one section later on its own author.
3. **AND THE SCAN THAT REPAIRED IT WAS *MORE COMPLETE AND ALSO WRONG*.** Re-run blind over all
   109 `.rs` files, it rewrote **the four accessors into themselves** — `self.map_lp.get()`
   became `self.map_lp().get()`, infinite recursion — because a global rewrite has no way to know
   which sites it has already created. It would not have compiled, so it was cheap; the lesson is
   not. **Completeness and correctness are different properties, and the second pass bought the
   first by giving up the second.**

**AND THE GATE WAS READ THROUGH A PIPE A SECOND TIME.** Step 1a recorded `| tail -45` reporting
`tail`'s status. Step 1b's first run wrote the log whole and appended `CARGO_EXIT=$?` — and the
harness *still* reported exit 0, because the trailing `echo` was the last command. **`CARGO_EXIT`
was `101`**: the `rung56.rs` miss above. The log line caught what the status could not, which is
the only reason defect 2 was found at all. **Write the status into the artefact, never read it
off the runner.**

**THE GATE:** `cargo test --release`, status written into the log — **`CARGO_EXIT=0`, 95 suites, 847 passed, 0 failed, 0 ignored** (845 + the carrier's 2). **P1 IS HALF MEASURED AND HALF ARGUED, AND THE DIFFERENCE MATTERS.** *Measured:*
`try_instant_tail` and `powers` are UNEDITED (the rewrite pass touched `rung53/56/61` and
`slice_n_smoke`, not the two dispatch files) and both phase-6 dispatch gates are green.
*Argued, not measured:* that this shows **§ (iii)'s algebra holding in the port**. It does not —
those gates run with **no stator arming anywhere in the tree**, so they witness the cells still
dispatching, not the cells being invariant UNDER an arming. The algebra is sound (`with_vsv` sets
only `vsv`; `eta_t_at` reads only `a_t`; both bodies read) but it is sound **by reading**. Its
first port-side exercise is step 2, and **step 5's injections owe it one**: arm during a march and
assert `try_instant_tail`'s output is bit-identical.

##### STEP 2 — SHIPPED. **THE CARRIER'S FIRST REAL EXERCISE — AND BOTH OF THE STEP'S OWN CLOSING GATES WERE WRITTEN VACUOUS BEFORE THEY WERE WRITTEN RIGHT**

`src/stator_transient.rs` — **2 510 lines**, rungs 57/58/59/60's whole Python class plus
`StatorSchedule` and `IncidenceLimiter`. Three NEW cells (`arm`, `v_of`, `stator_march`), the two
step 1a opened, one swap into `R40`. Gated-code edits: **130 lines over three files**
(`fuel_transient.rs`, `two_spool_transient.rs`, `lib.rs`), and **zero** in `two_spool.rs` — see
finding 1. `tests/slice_v_smoke.rs` + `oracle/dump_slice_v_smoke.py`: **1 986 keys bit-exact
against PyPy ON THE FIRST RUN**, eleven sections A–K.

**1. THE CARRIER LEVEL WAS WRONG IN THE PLAN I WROTE, AND THE ADVISOR CORRECTED IT BEFORE A LINE
EXISTED.** The state and the `arm`/`v_of` cells were headed for `TwoSpoolMapCore` — that is where
`map_lp`/`map_hp` and `tt2_d` live, i.e. everything `_arm` reads and writes, and it is where
`bleed` and `stack_lp` set the precedent. Two reasons put them on `TwoSpoolTransientCore` instead.
The binding one: `try_close` is a **rung-40** cell, so `r57_try_close` receives only
`&TwoSpoolTransientCore` — the SHALLOWEST type `_arm` must be reachable from, and reachable from
`&FuelTransientCore` through `inner` for the other two cells. The second: `TwoSpoolMapCore` is
shared with the **steady** ladder (`VariableStatorCore`, `StatorBleedMatcher`, `MapMatcher`), so a
transient-only field there is strictly wider than `bleed`/`stack_lp`, which at least have steady
consumers. **The precedent named the right SHAPE and the wrong LEVEL** — and the level is set by
the shallowest hook RECEIVER, not by where the data happens to live.

**2. A GATED SIGNATURE DID CHANGE — NOT THE ONE § 5.19 (x) NAMED, AND NOT FOR THE REASON
§ 5.20 (v) REFUTED.** § (v) refuted *"slice V is the one slice that CHANGES A GATED SIGNATURE"* at
its stated reason (`try_close` does not take a `&Scope`; that is slice Y's). It is true anyway, of
a different cell. Python's `integrate_fuel(surge=…)` is **ONE slot holding TWO types**: rung 49's
`SurgeLimiter` and rung 60's `IncidenceLimiter`. `SurgeLimiter` cannot represent the second — its
floor is a CONSTANT, an incidence floor's `phi_lim` is a function of the LIVE stator setting,
recomputed at every state — so the wire type widened to `Floor { Phi, Incidence }`, and
`FuelTransientHooks::try_surge_fuel`, its dispatcher, `r43_try_surge_fuel` and both dispatch twins
took it. **The label survives with a THIRD distinct reason**, after § (iv)'s carrier and § (v)'s
refutation.

**AND THE COST WAS CHOSEN, NOT ACCEPTED.** `FuelLimiters.surge` is spelled at **~130 already-gated
sites** across eleven test files; widening THAT field to `Option<Floor>` would have churned every
one to buy nothing. It keeps its rung-49 type, gains a rung-60 `incidence` field beside it (free —
every site already writes `..Default::default()`), and one method `floor()` recovers Python's
single slot and asserts the two exclusive. Measured cost: **95 lines in `fuel_transient.rs`, zero
test edits.** One executable line was added to a body step 1a had verified string-for-string —
`let surge = floor.phi();` — and it is called out at the site, because *zero executable lines
changed* is a claim with a date on it.

**3. AN ASSERT THAT HAD TO BE AN `Abort`, AND NO VALUE GATE WOULD HAVE FOUND IT.**
`IncidenceLimiter.phi_lim_at`'s `assert d > 0.0` is reachable from inside `der` — `try_surge_fuel`
→ rung 57's cell → `resolve_floor` → here — and Python's marcher SWALLOWS an `AssertionError` at
that depth and truncates. A Rust panic would abort a march Python completes. That is § 5.16 probe
4 (A) (the reason `try_tt4_from_f` exists) landing on a site slice V *introduces*, and it fires
only where the floor sits at or above the critical incidence, so a bit-exact dump over admissible
cells certifies nothing about it. The whole rungs-57–60 assert set was swept once under that rule:
entry-point and constructor guards stay panics; everything reachable from `der` is an `Abort`.
`Floor::phi()` is deliberately a PANIC — a rung-43 object handed an incidence floor is Python's
`AttributeError` on `surge.phi_lim`, which nothing in the ladder catches either.

**4. THE DEFAULT TABLE'S CELLS PANIC, WHICH TURNS A COMMENT INTO A GATE.** Rung 40 has no `_arm`
in Python at all: an unarmed rung-40/43 object is not a rung-57 object with the lever at zero, it
is one where the name does not exist. Defaulting `NO_STATOR`'s cells to rung 57's bodies would
silently make a rung-40 object armable — a claim no value key could witness.
`a_rung_43_object_never_dispatches_the_stator_table` marches a bare rung-43 core through the
closure, the equilibrium, an accel-limited RK4 ramp and a `phi`-floor-limited one; every one of
those would hit the panic if any rung-40/43 body carried an arming call, so a green run **is** the
unreachability claim.

**5. THE GATE WRITTEN TO CLOSE THE STEP WAS VACUOUS, AND THE WRITE-UP HAD ALREADY CALLED IT
MEASURED.** The first cut asserted `R57_TWO.try_instant_tail == R40.try_instant_tail` as raw `fn`
pointers and this section claimed *"P1's second half, MEASURED"* off it. `R57_TWO` is built with
`..R40`, so that equality is a **compile-time tautology** — no struct literal spelled that way can
make it fail — and the two inequalities beside it are tautologies too, since distinct `fn` items
always have distinct addresses. It was **restated, not measured**, and slice U step 5's finding
(*the closing step wrote TWO near-vacuous gates of its own*) landed on this slice's own closing
gate. The advisor blocked the step on it.

Replaced by `the_two_shipped_cells_are_invariant_under_an_arming`, which turns the algebra into a
number: march a SCHEDULED machine, keep the map `_arm` leaves stale, and call BOTH cells at ONE
fixed `(CloseState, nu, Tt4)` against the stale map and against the design map. **Bit-identical, on
`eta_lpt`, `eta_hpt`, `sp_thrust`, the whole `Instant2`, and both power residuals — so P1's second
half IS now measured.** The anti-vacuity half is the point: it first asserts the two maps differ
OBSERVABLY (`vsv` moved, and `psi` and `phi_surge_at` both move with it), because a zero would
otherwise read the same whether the cells are invariant or the maps are equal. The pointer
assertions survive under the honest name `the_table_spelling_inherits_rather_than_copies`.

**AND THE DETECTOR WAS MEASURED, WHICH SPLIT THE TWO INSTRUMENTS' FLOORS.** `+ vsv * 1e-9` on
`eta_lpt` inside `r40_try_instant_tail` FAILS the new gate. The same injection at **`1e-15`** does
NOT — `vsv ≈ 0.017`, so the perturbation is ~1.7e-17 against an ULP of `eta_lpt ≈ 0.9` — yet the
**value dump catches it**, because a 35-step march accumulates. *A pointwise bit gate bottoms out
at an ULP; a marched dump amplifies. Neither subsumes the other*, and the pair of numbers is
recorded so slice W does not re-derive it.

**6. TWO SMALLER REPAIRS FROM THE SAME REVIEW.** (a) `Ramp::with_r` is used once, in rung 60's
RATE ladder, and a `s_settle` accidentally scaled with `r` would move the excursion without moving
the credit — the exact pair the criterion differences. Now asserted, and **the bar is measured
rather than guessed**: the first cut demanded the credit bit-equal across the ladder and FAILED on
the clean tree at 0.73 %, because rung 57's own headline says clock-free is approximate (~1 point
over a 20× range). Measured across 4×: credit **0.73 %**, excursion **66.3 %**, contrast **91×**;
the bar sits on the contrast at 20×. (b) `READ_FOREIGN_VOF` was documented *"measured DEAD"* on the
strength of a **grep of `_read`'s call sites**, and no test read it. The doc now says what was
actually done, and section K gates the counter at zero — a counter nothing reads proves nothing.

**7. THE FINDING: THE SMOKE'S FIRST CUT WAS 1 742 KEYS BIT-EXACT AND BLIND TO THREE OF ITS OWN
BRANCHES.** It passed. Reading the flags it had just emitted — not running it — showed:

| branch | first cut | why it was blind |
|---|---|---|
| `floor_composite`'s `both_pinned` regime | never reached — BOTH cells `dormant`, regime `armed_clears` | the floor I picked (`phi_lim = 0.5964`) never binds at `ds = 0.05`, so `pinned_prediction` — rung 60's whole derived claim, EXACTLY `v` or EXACTLY `0` — was `NaN` in every cell |
| `schedule_invariance`'s tuple identity | `ordinate_identical` / `abscissa_identical` **false in every cell** | the source claims `==` only at `v = 0`; every armed cell lands ~1e-13 away, so both booleans were gated at `false` and the branch that makes them `true` never ran |
| `pin_audit`'s `pinned` / `from_zero` | both `false` everywhere | downstream of the first row |

Repaired by adding a SECOND `(v, m_lim)` pair — rung 60's own `(0.05, 0.500)` — and a
zero-schedule invariance arm: `both_pinned` now lands with `pinned_prediction` **exactly `0.05`**
(the `phi` arm, `= v`) and **exactly `0.0`** (the incidence arm), residuals `-1.80e-16` and
`-2.22e-16`; `inv_zero` lands both identities `true` with `d_ordinate = d_abscissa = 0`. **This is
slice U step 1's finding arriving on a DUMP's cell choice rather than on a suite's:** bit-exact and
green says nothing about which BRANCH ran, and the only instrument that saw it was counting the
discrete keys the dump had already emitted. Two smaller versions of the same thing in the same
pass — a hand-typed `arrow_toggle` state `(0.82, 0.93, 0.42)` that does not bracket, and
`vsv_hp = 0.20` that goes off-map — were caught by Python raising, i.e. for free; **the branch
blindness was the one that would have shipped.**

**8. WHAT THE SMOKE MEASURES THAT THE PLAN ONLY PREDICTED.** Section C dumps the LIVE map's `vsv`
AFTER a march, per arming: **`0.0170`** LP-scheduled, **`0.0111`** HP, and **`0.2` exactly** for a
CONSTANT setting — against the design `0.0`. That is § 5.20 (i)'s permanent mutation as a bit key,
so a locally-armed-core port fails on a NUMBER rather than passing in silence. Beside it, the two
channels the arming drives (`psi`, `phi_surge_at`) and the one § (iii) proves it cannot
(`eta_t_at`), all read off the stale map. **P4 is half-discharged early**: the smoke already
carries `hp_only` and `both` scheduled cells, so the HP path is exercised at step 2 rather than
waiting for step 4's checklist item (a) — which still stands for the full dump.

**P3, RE-GATED AS PRE-REGISTERED.** `_arm` hands back the SAME map OBJECT at `v == 0.0`;
`ComponentMap` is `Copy` and has no identity, so Python's `is`-test does not survive as written.
Section B's zero-schedule march is bit-for-bit AND section K reads `arm_lp_zero = 188 / 188` on
that machine (against `9 / 188` where a real schedule crosses `n_ref`), which is what the identity
claim reduces to on a value type. The counters exist because step 2 laid them, per P3's own note.

**AND ONE DEFERRAL, WITH A COUNT RATHER THAN AN INTENTION (P4's rule).** `at_stator` gets **no
cell** — § 5.19 (iii)'s pure sibling constructor — but rungs **62 and 64** override it in Python,
and **eight** reader bodies call `self.at_stator()` (`stator_credit`, `credit_decomposition`,
`composite_credit`, `engagement_shift`, `schedule_invariance`, `matched_credit`,
`set_point_bands`, `floor_composite`). Those eight are INHERITED, so a rung-64 object running the
inherited `stator_credit` would build a rung-**57** bare sibling. Rungs 57–60 construct only
rung-57 objects, so it is inert here; it is **slice W's first job** and the note lives at
`at_stator`'s own definition, not only here.

**THE GATE:**
`cargo test --release`, status written into the log — **`CARGO_EXIT=0`, 96 suites, 851 passed, 0 failed, 0 ignored** (step 1b's 95 / 847, plus this slice's one suite and its four tests). The `0 ignored` is slice M's rule still holding. **NO PYTHON SOURCE CHANGED**, so `pytest` is untouched by this step and is not re-run — `rust/oracle/dump_slice_v_smoke.py` is a dump, not a test, and lives outside `turbojet/` and `tests/`.

##### STEP 3 — SHIPPED. **THE SLICE'S OWN CARRIER BUG REPRODUCES PYTHON'S NUMBERS TO THREE FIGURES AND IS CAUGHT BY 0 OF 59 GATES — AND THE FIRST RUN THAT SAID SO WAS VOID**

`tests/rung57.rs` / `rung58.rs` / `rung59.rs` / `rung60.rs` — **16 + 15 + 12 + 16 = 59 `#[test]`
against Python's 59 collected**, one to one, and **0 source lines changed** except the one doc
comment finding 2 is about. Both counts EMITTED, not typed: `pytest --collect-only -q` over the
four files reports **59 collected from 57 `def test_`** (the +2 are rung 57's two two-way
`parametrize`s, expanded here into `_primary` / `_tilted` so the Rust count matches the
COLLECTED one), and `cargo test --release` reports 59 run, 0 ignored.

**THE `slow` COST, WHICH § 5.19 (viii) PARKED AT THIS SLICE.** 29 of the 59 carry
`@pytest.mark.slow` (49.2 %, the phase ratio) and **not one becomes `#[ignore]`** — that would
retire half the slice's gates behind a flag and break the `0 ignored` line every gate since slice
M has carried. All 59 run every time. Wall clock, per suite: **0.47 / 1.24 / 1.66 / 2.03 s = 5.40 s**
against Python's **12.84 / 11.41 / 9.73 / 8.43 = 42.41 s**. **That ratio is NOT like-for-like and
the number is quoted with its caveat**: the Python figures come off `pytest`'s own summary under
the repo's xdist `-n` default, i.e. already parallel across workers, while the Rust figures are
four separate serial `cargo test` invocations. The usable conclusion is the absolute one — a
`slow`-heavy suite costs **~2 s** ported, so the phase's 263 `slow`-carrying collected tests are
not a gate-time problem.

**FINDING 1 — A SHIPPED DOC COMMENT THAT IS TRUE OF THE SIGNATURES AND FALSE OF THREE OF THE FOUR
SUITES.** `Ramp::fine`'s comment, written at step 2, calls `ds = 0.005` *"rungs 58/59/60's
default"*. As a statement about the READER METHODS it is exactly right — `composite_credit`,
`matched_credit`, `floor_composite` and the rest all declare `ds: float = 0.005`. It is not what
the suites march on. Measured off the four files rather than read off the comment:

| suite | `DS` declared | passed explicitly? | what the reader default would have given |
|---|---|---|---|
| `test_rung57.py` | **0.01** | yes | 0.01 — agrees |
| `test_rung58.py` | **0.01** | **yes, at every call site** | 0.005 — **half the step** |
| `test_rung59.py` | **0.01** | **yes, at every call site** | 0.005 — **half the step** |
| `test_rung60.py` | **0.005** | yes | 0.005 — agrees |

Porting rung 58 or 59 through `Ramp::fine` *because its doc comment names those rungs* would have
halved their step and moved every number they assert, and nothing in either suite would have said
so — the gates are relational and a finer grid moves both sides. **The comment is corrected in
`stator_transient.rs` rather than only footnoted in a test header**, because slice W inherits it
and would be misled the same way. This is [[rust-port-slice-l-step4]]'s lesson (*a claim in the
SHIPPED source was false*) on a doc comment that was true of the thing it was looking at and false
of the thing a reader would use it for.

**FINDING 2 — THE FIRST INJECTION PASS WAS VOID, AND ITS VOIDNESS IS THIS PORT'S OWN LESSON
ARRIVING ON MY OWN INSTRUMENT.** The probe was built to dump the readings the ported gates
compare — 302 of them. Run against the two structural injections it reported
`moved 0/302, caught 0/59` for both. **`moved 0` is not a result**: an injection whose only
observable is OBJECT STATE looks identical to an injection that never applied, and the harness
could not tell them apart. Slice S step 3's *injections reporting "nothing moved" could not have
moved anything*, one level up — the earlier instance was about injections that missed, this one
is about a probe that could not see. Repaired by adding a **witness section** counted separately:
the live map's `vsv` after a march (§ 5.20 (i)'s permanent mutation), `arm`'s dispatch counters on
an HP-SCHEDULED machine that no suite builds, and the steady `surge_margin`. 342 keys, of which
**20 are witnesses and 322 are gate-visible**.

**FINDING 3 — THE CARRIER BUG REPRODUCES § 5.20 (ii)'s PYTHON MEASUREMENT TO THREE SIGNIFICANT
FIGURES, ON THE RUST SIDE, WITH ALL 59 GATES GREEN.** I1 is the local-armed-core port: save the
maps, call `arm`, restore — the shape a natural Rust port takes, and the one § 5.20 (ii)
measured in Python by patching Python to behave like it.

| arming | key | PY baseline | PY scoped | PY rel % | **RS clean** | **RS injected** | **RS rel %** |
|---|---|---|---|---|---|---|---|
| lp_only | `SM_lp` | 0.06080308471 | 0.05798678588 | 4.632 | 0.0608025 | 0.0579868 | **4.631** |
| hp_only | `SM_hp` | 0.4404934501 | 0.43011312 | 2.357 | 0.440491 | 0.430113 | **2.356** |
| both | `SM_lp` | 0.06087379962 | 0.05798678588 | 4.743 | 0.0608732 | 0.0579868 | **4.742** |

and the **live map's `vsv` moves by 100 %** on every scheduled arm (0.0171159 → 0, 0.0111379 → 0).
**Caught by 0 of 59.** The channel is named rather than inferred: `surge_margin` sits on
`TwoSpoolMapCore` and runs a STEADY match, so it never passes through rung 57's `try_close` and
nothing re-arms — it reads whatever the last sub-step left. That is the port-side version of
Python's 59/59-either-way, MEASURED rather than inherited, and it makes step 5's carrier gate
demonstrably necessary instead of merely promised.

**AND ONE HALF OF § 5.20 (ii) DID NOT REPRODUCE, WHICH IS STATED AS A BOUND AND NOT AS AN
IMMUNITY.** Python's probe also moved `margin_min_lp` by **15.4 %** (and `npts` 61 → 62 on
`both`). In this harness the Rust `transient_surge_margin_fuel` reading did **not** move, and the
reason is visible: it re-marches from `equilibrium`, every close inside which fires `arm` and
overwrites the stale map before it can reach anything. **That is a difference in CALL ORDER, not
in exposure** — Python's probe read that key in a sequence where the staleness survived to it.
Booked for step 5 with the number attached (P4's rule): the carrier gate must reach the transient
reader in an order that preserves the staleness, and 15.4 % is the target it is aiming at.

**THE INJECTION TABLE — the DID-IT-MOVE column measured FIRST, per slice T step 3's precondition.**

| injection | moved (gate-visible) | moved (witness) | worst rel | caught |
|---|---|---|---|---|
| **I1 — the LOCAL-ARMED-CORE carrier** | **0 / 302** | 9 / 40 | 1.00 (`vsv`) | **0 / 59** |
| **I2 — `arm`'s HP branch dropped** | **0 / 302** | 15 / 40 | 1.00 | **0 / 59** |
| I3 — `Shape::Smooth` `x²(3−2x)` → `x³` | 78 / 302 | 21 / 40 | 4.10 | **4 / 59** |
| I4 — `erosion` inverted | 5 / 302 | 0 / 40 | 3.89 | **5 / 59** |
| I5 — the incidence floor's lever sign | 5 / 302 | 0 / 40 | 1.8e14 | **5 / 59** |
| I6 — `arm` reads the WRONG SHAFT | 79 / 302 | 12 / 40 | 12.1 | **4 / 59** |

All six re-run TOGETHER against the final 342-key probe, so the table is internally
consistent — an earlier single-injection re-run had overwritten it, and a table whose rows
came off different baselines is not one table. Per-gate failing names, and the 302/40 split
per row, in `docs/plans/slice-v-step3-evidence.md`.

**I2's moved keys carry the one piece of evidence in the run that the HP arm is not inert.**
Dropping the HP branch moves `W/live/both/vsv_lp` — the **LP** setting — from 0.0167136 to
0.0171159, because the two schedules are coupled through the shaft state. It sits directly beside
§ 5.20 P4's finding that no suite ever leaves `map_hp` mutated: the HP path is unexercised by the
GATES and is not dynamically inert in the PLANT.

**THE BAR-MARGIN TABLE — 63 inequalities, got-vs-bar, because green says nothing about slack**
(slice T step 2: 9/9 green and blind to 24 %). **Full table committed at
`docs/plans/slice-v-step3-evidence.md`**, beside the injection table — not left in a scratch
directory, which is step-4 checklist item (b)'s own rule (*if it can be emitted, emit it*, and a
number a later step depends on has to be IN THE REPO) applied to this step's instruments. The shape:

- **Seven bars sit within 10 % of their value** and are the live ones — tightest is rung 58's
  `v_ratio > 1.10` at **+1.5 %**, then rung 60's `gap_phi_bands > 1.0` at +5.3 % and rung 57's
  `m_phi shut < bare` at +5.4 %.
- **Twelve pass at more than 5× margin**, up to **+127×** (rung 57 P5's `|d_phi_lp| > 1e-3`).
  Those are not loose magnitude tests that should be tightened — their Python docstrings say so
  explicitly (*"the gate gives the ORDER and the SIGN headroom rather than pinning the weakest
  measured row"*). Recorded as SIGN tests, so a later reader cannot mistake the slack for a bug.
- **THE TIGHTEST NON-PHYSICS BAR IS A TOLERANCE BAR, and it is one solver change from flipping.**
  Rung 59's `d_abscissa < 1e-12` on the `LP const` arm reads **7.76e-13 — 22 % of headroom**.
  Python carries the identical bar at the identical value. Named, not loosened: the bar is
  correct and the margin is the disclosure.

**THE TWO NON-STRICT ORDERINGS, PORTED AS WRITTEN AND THEN MEASURED.** Python's
`ss == sorted(ss, reverse=True)` is a `>=`, satisfied by an inert sequence, so `>=` is what runs
here. Both are in fact strictly monotone, and the smallest adjacent gap is the number that says
how much:

| sequence | values over `r` = 0.1 … 2.0 | smallest adjacent gap |
|---|---|---|
| `share_start` | +0.2705 +0.1196 +0.0297 −0.0298 −0.0681 | 3.83e-2 (**11.3 %** of the range) |
| `self_cancel` | +0.8963 +0.8029 +0.7689 +0.7563 +0.7541 | 2.27e-3 (**1.6 %** of the range) |

`self_cancel`'s tail is nearly flat, so its ordering assertion is doing almost no work at the slow
end — the claim it carries (*the surrender DEEPENS with r*) is real but is delivered almost
entirely by the first gap. **AND THAT IS DISCLOSED RATHER THAN REPAIRED.** At `r` = 1.0 → 2.0
the gate is satisfied by 2.27e-3, which a grid change could erase or flip; tightening it would
make the Rust contract STRICTER than the Python one it is porting, which is the one thing a
ported gate must not do. Python carries the identical exposure at the identical values. Left as
written, with the number recorded, so a future flip reads as the grid moving and not as the
finding failing.

**THREE `is`-IDENTITY TESTS RE-GATED, EACH SAYING WHAT IT GAVE UP.** Python asserts object
identity in three places that a `Copy` value type cannot express: rung 57's *`_arm` hands back the
SAME map object* (§ 5.20 P3), rung 58's *the wall is literally one object*, and rung 60's *a
`SurgeLimiter` passes the resolver by identity*. Each is re-gated as **equality PLUS the crate's
own dispatch counter** — `arm_lp_zero == arm_calls` with `arm_lp_moved == 0`, and
`resolve_phi == n` with `resolve_incidence == 0` — which is what the identity claim reduces to
when the alternative is a REBUILD or a CONVERSION rather than a copy. The weakening is written
into each test's own doc comment; a reduce gate that quietly answers a smaller question is
[[rust-port-ported-test-vacuity]].

**AND TWO OF RUNG 60's SIXTEEN TEST A REFUSAL THIS PORT MAKES UNREPRESENTABLE.**
`floor_composite` takes a `Floor` where Python takes any leg, and `composability_ladder` takes a
`LadderAxis` where Python takes two mutually-exclusive keyword lists — so *"hand it an
`AccelSchedule`"* and *"hand it both axes"* cannot be written down. Kept at 1:1 and re-gated as an
**exhaustive `match` over each enum**, which stops compiling if a third variant appears — the
event the Python assert exists to catch — plus a runtime half asserting what the refusal protects
(the two readers do not measure the same quantity; the two axes carry different halves of the
criterion). **Decided the opposite way from rung 57's `Shape`**, where the bad value is a STRING a
caller could supply and the port therefore kept a `try_from_str` entry point. Both decisions are
stated at their own sites rather than one being applied silently to the other.

**WHAT 59/59 GREEN DOES NOT ESTABLISH, WRITTEN INTO ALL FOUR FILE HEADERS.** Every one of these
gates is RELATIONAL — it asserts a relation among values this crate computed. A Rust/Python
arithmetic divergence moves both sides of every one of them and leaves all 59 green. That is
§ 5.20 (ii)'s own headline one level up, and the instrument that establishes agreement with Python
is **step 4's oracle**, not this file. The four suites also inherit Python's HP blindness exactly
(§ 5.20 P4: 0 of 920 262 closes ever left `map_hp` mutated), so the HP path is exercised by the
smoke's section C and by step 4's dump, not by these 59.

**THE PROBE IS PRESERVED, NOT SHIPPED.** `slice_v_probe.rs` has no assertions — it prints. A
no-assertion `#[test]` is a vacuous gate and would have added a 60th test that can never fail, so
it is removed from `rust/tests/` and kept at
**`rust/oracle/slice_v_probe.rs.keep`** (the `.keep` suffix is what stops cargo compiling it),
restored by copying it to `rust/tests/slice_v_probe.rs`. Its harness is
**`rust/oracle/inject_slice_v.py`** — it takes injection names as arguments and restores the
source in a `finally`. Both are in the repo rather than in a scratch directory **because step 5
is a different session and cites them**; leaving an instrument a later step depends on outside git
is exactly the non-durability step-4 checklist item (b) was written to stop.
**`CARGO_EXIT=0`, 100 suites, 910 passed, 0 failed, 0 ignored** (step 2's 96 / 851, plus
this step's four suites and their 59 tests). **NO PYTHON SOURCE CHANGED**, so `pytest` is
untouched by this step and is not re-run.

##### STEP 4 — SHIPPED. **A CHANNEL STEP 3 MEASURED AS UNREACHABLE WAS REACHABLE ONE READER EARLIER IN THE SAME CHAIN**

`rust/oracle/dump_slice_v.py` + `slice_v_pypy.tsv` + `slice_v_cpython.tsv` +
`rust/tests/slice_v_oracle.rs`. **6 819 keys over eight sections, bit-exact against PyPy on the
first run AND against CPython 3.14 — 0 drifts, 0 flips, no tolerance tier.** Five gates. The
crate is **101 suites / 915 passed / 0 failed / 0 ignored**, reconciled as a name diff against
step 3's baseline: `cargo test --release -- --list` reports **915** against step 3's 910, **+5
additions, 0 removals** — because *exit-0 proves nothing FAILED; only the diff proves nothing
VANISHED*. `git diff -- rust/src/` is **empty**: step 4 is test-and-oracle only, and the check was
re-run AFTER the injection harness's `finally` restored the source rather than inherited from
before it. **NO PYTHON SOURCE CHANGED**, so `pytest` is untouched and is not re-run.

Every table is EMITTED: `docs/plans/slice-v-step4-evidence.md` is **generated from the artifacts**
by a script, not transcribed — § 5.19 (xi)'s rule, and the specific defect phase 7's pre-flight
found in every prior write-up.

**FINDING 1 — THE ORACLE CATCHES BOTH CARRIER INJECTIONS THAT ALL 59 PORTED GATES MISS.** Step 3
measured I1 (the locally-armed-core carrier) and I2 (`arm`'s HP branch deleted) at **0 of 302**
gate-visible readings and **0 of 59** gates. Against the oracle:

| injection | caught by the 59 (step 3) | oracle | keys differing |
|---|---|---|---|
| I1 — the LOCAL-ARMED-CORE carrier | **0 / 59** | CAUGHT | **87** / 6 819 |
| I2 — `arm`'s HP branch dropped | **0 / 59** | CAUGHT | **709** / 6 819 |
| I3 `Shape::Smooth` cubed | 4 / 59 | CAUGHT | 1 945 |
| I4 `erosion` inverted | 5 / 59 | CAUGHT | 14 |
| I5 the incidence lever's sign | 5 / 59 | **PANIC-BEFORE-COMPARE** | n/a |
| I6 `arm` reads the WRONG SHAFT | 4 / 59 | CAUGHT | 1 440 |

**AND THAT DOES NOT DISCHARGE P5**, which is written down rather than left for a later reader to
infer: a golden-comparison gate is defeated by regenerating the golden against buggy code, and a
gate that MANUFACTURES the scoped-arm behaviour inline is not. Step 5 still owes the manufactured
carrier gate. What step 4 removes from step 5's list is only the CALL-ORDER problem — finding 2.

**I5 IS CAUGHT BY A PANIC, NOT BY A KEY**, and the harness now says which. Inverting the incidence
lever empties a trajectory and `refine_min` indexes `traj[0]`, so the run dies before the
comparator sees anything and the row reads `0 / 0`. A bare `0` there is indistinguishable from
*nothing moved* — slice S step 3's lesson, so `--oracle` mode gives it its own status.

**FINDING 2 — THE 15.4 % CHANNEL IS REACHED AT STEP 4, AND STEP 3'S BOOKING IS CORRECTED.** Step 3
reproduced § (ii)'s `SM_lp` numbers on the Rust side but recorded `margin_min_lp` as NOT moving,
because its harness read `transient_surge_margin_fuel` — which re-marches from `equilibrium`, and
every close inside that re-arms before anything stale can reach it. It booked the channel to step
5 *"with the number attached"*. **Its MEASUREMENT was right and is not retracted** — step 3 said
so itself (*"a difference in CALL ORDER, not in exposure"*) and stated the reading as a bound
rather than an immunity, which is exactly why the number survived to be aimed at. What step 4
corrects is the BOOKING that followed it. Section A reads **rung 44's**
`transient_surge_margin` instead, one call earlier in p7o's chain, and the channel is live there:

```text
A/both/tsm/margin_min_lp   baseline 0.046234125347077270  (3fa7abffd0a65440)
                           scoped   0.039099866681263640  (3fa404e5d15914e0)
                           relative 15.431 %
```

with `A/both/tsm/npts` **61 → 62**. That is § (ii)'s own figure and its own bit patterns. **So all
twelve of § (ii)'s numbers are now reproduced in Rust** — the six baseline values as committed
golden keys, the six scoped values as the I1 reading — and § (ii)'s `const_lp` row (*every key: 0,
no difference at all*) reproduces exactly: **0** of that arming's keys move under I1. The general
form: *a stale field's reach is a property of WHERE IN A READER CHAIN you look, so a channel
measured closed at one reader says nothing about the reader before it.*

**FINDING 3 — § (ii)'s TABLE WAS READ OFF A CALL WHOSE THIRD ARGUMENT IS A DELTA, GIVEN THE RAMP
TOP.** `transient_surge_margin(flight, Tt4_lo, dTt4, …)` takes an INCREMENT — `test_rung44.py`
passes `300.0` / `400.0`. `probe_p7o.py` passed `HI` = 1400.0, so every `margin_min_*` number in
§ (ii) is off a march from 1000 K to **2400 K**. Section A reproduces the call verbatim, because
P2 is checked against those numbers and a "corrected" section A would make the plan's own table
uncheckable; **section A' carries the corrected `dTt4 = HI − LO` reading beside it**, labelled
ADDED. It is not a small difference — on `both`, `0.046234` at `npts` 61 against `0.068339` at
`npts` 151. Nothing is retracted: § (ii)'s conclusions are about a DIFFERENCE between two modes on
one grid, and that survives its grid being odd.

**FINDING 4 — THE READER CHAIN IS A SEQUENCE, AND THAT IS GATED RATHER THAN COMMENTED.** Because
`arm` mutates permanently, each reader in section A leaves the map where ITS last sub-step put it.
Dropping `transient_surge_margin_fuel` from the chain moves `A/both/sm/SM_lp` from
`3faf2ad9c5223ee0` to `3fadb071a9e7f9a0` — the DESIGN value. A claim like that living only in a
doc comment is [[rust-port-slice-l-step4]]; `the_reader_chain_is_a_sequence_not_a_set` reproduces
both readings and asserts they differ, and that the skipped one IS the design reading.

**FINDING 5 — TWO OF MY OWN GATES CARRIED GUESSED BARS, AND THE SECOND WAS THE SAME SHAPE AS THE
FIRST.** The gate pinning § (ii)'s six baseline values first used a blanket RELATIVE `5e-11` and
failed at `1.2e-11` on `A/lp_only/tsm/margin_min_lp`. The plan prints **10 SIGNIFICANT figures**,
so what a printed value licenses is half a unit in its own last printed DECIMAL place — `5e-11`
near 0.11, `5e-12` near 0.046 — which is now a per-value column with the six measured misses
recorded beside it. Having fixed that, the SAME defect was still sitting in
`section_b_is_the_hp_scheduled_arm_no_suite_reaches` as `n > 400` on a measured **516**: a floor
lets the section shrink by a fifth with the gate green and the write-up's number stale, which is
exactly what P4's *"a deferral with a number attached survives"* was written against. Pinned as an
equality. *Fixing an instance of a failure mode is not the same as sweeping for it.*

**FINDING 6 — ZERO INTERPRETER DIVERGENCE, AND THE PROVENANCE IS NOW IN THE FILE.** The two
goldens agree on **all 6 819 data lines** — against slice K's 46.3 % CPython-identical, and with
no tolerance tier, since every cell in this slice is CPG. That claim needed a record it did not
have: two files generated by the SAME interpreter are byte-identical too, and nothing in the bytes
said which one made them. The dump now stamps itself (`# generated by pypy 3.11.15` /
`# generated by cpython 3.14.3`), the comparator skips `#` lines, and the identity is checkable.
*A provenance claim needs a provenance record, not a careful memory of which command was run.*

**FINDING 7 — `1.4 - 1.0` IS NOT `0.4`.** The first go/no-go against § (ii)'s bits missed by 48
ULPs, and the cause was `R_c=0.4/1.4*1004.0` where the suites write `(1.4-1.0)/1.4*1004.0`.
`1.4 - 1.0` is `0.3999999999999999`. Both the dump and the oracle carry the reason at the
constructor rather than only the correct spelling, because the wrong one looks better.

**CHECKLIST ITEMS DISCHARGED.** (a) section B — the HP-SCHEDULED machine, **516 keys**, pinned as
an equality, with a companion assertion that the arm is LIVE (an HP schedule leaves `map_hp` off
its design value after a march, and leaves `map_lp` at zero) — the event **0 of 920 262** suite
closes produce. (b) § (ii)'s twelve numbers are dump keys and injected readings, per finding 2,
against a committed TSV rather than a table someone typed from a probe.

##### STEP 5 — SHIPPED. **THE MANUFACTURED GATE, AND FOUR OF ITS SIX ASSERTIONS NEED NOTHING ON DISK**

`rust/tests/slice_v_dispatch.rs` (6 `#[test]`) + two more modes on `rust/oracle/inject_slice_v.py`
(`--dispatch` against the port, `--self` against the gate's own wrapper) + `rust/oracle/emit_slice_v_step5_evidence.py` →
`docs/plans/slice-v-step5-evidence.md`. **`git diff -- rust/src/` is EMPTY** — the whole file is
built out of already-`pub` seams (`with_all_hooks`, `FuelTransientCore`'s two fields,
`StatorArming`'s six), so the manufactured table gets in without a source change, exactly as
`slice_r_dispatch.rs` does one ladder down. **NO PYTHON SOURCE CHANGED**, so `pytest` is untouched
and is not re-run. **SLICE V IS COMPLETE.**

**WHAT P5 WAS FOR, RESTATED NOW THAT ALL FOUR INSTRUMENTS EXIST.** The slice has four things that
look at the carrier and only one of them is a gate on the carrier:

| instrument | what it establishes | what it cannot see |
|---|---|---|
| the **59 ported gates** | rungs 57–60's relations hold | the carrier — **0 / 59**, measured at step 3 |
| `slice_v_carrier.rs` | the MECHANISM is live (a `&`-write persists) | whether a marched object needs it |
| `slice_v_oracle.rs` | agreement with PyPy, 6 819 keys; catches I1 at 87 | it is a GOLDEN, and a golden is regenerable |
| **`slice_v_dispatch.rs`** | the clean/scoped DIFFERENCE, computed live | values not in its reader chain (I4, I5) |

**FINDING 1 — FOUR OF THE SIX GATES READ NOTHING ON DISK, AND THAT IS THE PART THAT IS NEW.**
Two of the six pin their clean column against `oracle/slice_v_pypy.tsv` **by key** rather than
against a literal — which is § 5.19 (xi)'s rule and also closes a hole the oracle has on its own
(regenerate the golden against buggy code and the oracle agrees; this file's live clean
computation then stops matching and it fails). The other four —
`the_hand_built_machine_is_the_shipped_one`, `a_constant_setting_is_the_negative_control_…`,
`the_scoped_port_reads_a_scheduled_machine_as_an_unstatored_one` and
`the_remarching_reader_is_immune_…` — touch no file at all: they compute both branches in Rust
and assert the difference. **Delete every golden in the repo and the carrier is still gated.**
The split is recorded per test in the evidence file rather than claimed in prose.

**FINDING 2 — THE SHARPEST STATEMENT OF THE DEFECT NEEDS NO BAR AT ALL, AND IT IS NOT A
PERCENTAGE.** Under the locally-armed-core port, `surge_margin` reads a SCHEDULED rung-57 machine
**bit-for-bit identically to a machine with no stator at all** — the same bit pattern
(`3fadb071a9e7f9a0`) for `lp_only`, `hp_only` and `both`, three armings that land in three
different places when the carrier works. `surge_margin` sits on `TwoSpoolMapCore` and runs a
STEADY match, so it never passes through rung 57's `try_close` and nothing re-arms.

Two things are asserted BESIDE it so the claim cannot be read wider than it is: `const_lp` does
**not** collapse (rung 53's constant is applied in the constructor, never through `_arm`), and
`transient_surge_margin` does **not** collapse either — it re-marches internally and re-arms
partway, so its three armings still land in three places. **The collapse is a property of the
READER, not of the bug**, which is step 4's finding 2 read from the other end.

**FINDING 3 — ONE READER IN THE SAME CHAIN, ON THE SAME OBJECT, IS COMPLETELY IMMUNE, AND THAT IS
NOW GATED FROM BOTH SIDES.** `transient_surge_margin_fuel` is bit-identical between the clean and
the scoped machine on **all four** armings, while `transient_surge_margin` — called one line
earlier on the same object — moves 15.4 %. Step 3 measured the first half and correctly booked it
as *a difference in CALL ORDER, not in exposure*; step 4 found the channel open one reader
earlier. The gate now asserts both at once, including the anti-vacuity half (*the immunity is only
a finding if the reader beside it is exposed*: exactly 3 of the 4 armings must move
`transient_surge_margin`).

**FINDING 4 — THE ANTI-VACUITY GATE HAD TO COVER THE ARMING THE OTHER FIVE NEVER USE.** The
manufactured table can only be installed by rebuilding the object, so the file's whole content
rests on the hand-built machine BEING the shipped one — asserted bit-for-bit on nine readings.
The advisor's correction before a line was written: assert it on **all four** armings, not on
`both`. `ScheduledStatorTransient::new` applies a CONSTANT setting after the design capture, and
no scheduled arming reaches that line — a botched replication of it is invisible on the three
scheduled arms and would then have hollowed out `const_lp`, which is this file's negative control.
The same shape as slice U step 4's finding, caught before it shipped instead of after.

**FINDING 5 — AND THE NEGATIVE CONTROL'S ZERO IS ASSERTED WITH ITS CALL COUNT.** *Nothing moved*
and *the wrapper was never reached* are the same reading. A file-local `thread_local!` counts the
wrapped calls; `const_lp` shows **2 388** of them and **0 of 9** readings moved. The bar is `>= 1`
and not that number, deliberately: the claim is REACHABILITY, and pinning a call count would fail
on any grid change for a reason that is not this finding.

**FINDING 6 — I TYPED A NUMBER THE TEST WAS ALREADY PRINTING, AND IT WAS WRONG BY AN ORDER OF
MAGNITUDE.** The doc comment's bar-margin table said `both/margin_min_lp` missed its bar by
`1.3e-13` (3 % of it). The gate's own stdout says **1.26e-12 — 25 %**. It came off a hand
calculation while the emitted value was one `println!` away. Corrected at both sites, and it is
§ 5.19 (xi) catching its own author for the third time in this slice (step 1b's hand-typed file
list, step 3's typed `Ramp::fine` comment, this).

**THE INJECTION RUN — PREDICTED FIRST, AND ONE PREDICTION WAS WRONG.**

| injection | predicted | measured | gates failing |
|---|---|---|---|
| **I1 — the LOCAL-ARMED-CORE carrier** | CAUGHT, 4 | **CAUGHT, 4** | the four predicted, exactly |
| **I2 — `arm`'s HP branch dropped** | CAUGHT, 3 | **CAUGHT, 4** ⚠ | + `the_remarching_reader_…` |
| I3 — `Shape::Smooth` cubed | CAUGHT, 2 | CAUGHT, 2 | |
| I4 — `erosion` inverted | MISSED | **MISSED** | not in this file's reader chain |
| I5 — the incidence lever's sign | MISSED | **MISSED** | no incidence floor is built here |
| I6 — `arm` reads the WRONG SHAFT | CAUGHT, 2 | CAUGHT, 2 | |

**I2's extra failure is the prediction being too narrow, and the reason is worth keeping.**
Dropping the HP arm makes the `hp_only` machine effectively unarmed, so its
`transient_surge_margin` stops moving between clean and scoped — and
`the_remarching_reader_…`'s anti-vacuity half (*exactly 3 of 4 armings must move*) fires on that.
The gate caught the injection through the assertion written to stop the gate itself going
vacuous. **I4 and I5 are MISSES and are reported as misses**: this file's reader chain does not
touch `erosion` and builds no incidence floor. The oracle covers both (14 keys and a
PANIC-BEFORE-COMPARE respectively), and a carrier gate that fired on everything would not be
measuring the carrier — `const_lp`'s exact zero is the same statement from the other side.

**FINDING 7 — EVERY INJECTION I HAD RUN PATCHED `src/`, AND NONE OF THEM TOUCHED THE MUTATION THE
GATE ITSELF CARRIES.** The advisor caught it after the six were green. I1–I6 all ask *does the gate
notice when the PORT is wrong?*; not one asks whether the two WRAPPER cells in
`slice_v_dispatch.rs` are themselves right. A wrapper that restored only ONE of the two maps is a
**partial carrier bug in the instrument**, and the four difference-asserting gates could have gone
on passing at their pinned values with only the HP rows moving. Booked as a fourth harness mode
(`inject_slice_v.py --self`) rather than as a one-off manual edit, so it re-runs:

| mutation of the gate's own wrapper | caught |
|---|---|
| **S1** — HP restore dropped in BOTH wrappers | **4 / 6** |
| **S2** — HP restore dropped in the FUEL wrapper only | **4 / 6** |

The same four as I1, and the advisor's prediction (*only the `hp_only` rows would fire*) is
**refuted by the measurement**: the two spools are coupled through the shaft state, so an HP-only
defect is not confined to the HP armings. That is step 3's I2 finding — *dropping the HP branch
moves the LP setting* — recurring one level up, on the instrument rather than on the port.
**A gate that manufactures a bug is itself code, and it needs the same did-it-move treatment.**

**FINDING 8 — A FAILED `io.open(path, "w")` STILL TRUNCATES, AND THE `finally` THEN COULD NOT PUT
IT BACK.** Building `--self`, a bad `newline=` argument raised `ValueError` — *after* the open had
already emptied `slice_v_dispatch.rs` to **0 bytes** — and the restore path hit the identical
error, so the guard that exists to make injections safe destroyed the file it was guarding. The
file came back off a copy made outside the harness. The harness now writes a `.bak` before it
patches, and the reason is at the writer rather than in a commit message. This is the other half
of [[windows-tooling-file-hazards]]'s recorded PyPy write hazard: that one is *the write never
lands*, this one is *the truncation lands and the write does not*.

**WHY ONE GATE IS GREEN UNDER ALL EIGHT INJECTIONS, STATED RATHER THAN LEFT TO INFER.**
`the_hand_built_machine_is_the_shipped_one` never fires above, and that is correct: it is an
equality between two objects built the same way, so a `src`-side bug moves both sides together.
Its detector is a divergence between the CONSTRUCTOR and the hand-build — an event no injection in
either set manufactures, and the reason it is written as nine readings over four armings rather
than one.

**STEP 1b's SECOND IOU, DISCHARGED AND NOT SILENTLY OMITTED.** Step 1b booked *"step 5's
injections owe § (iii) one: arm during a march and assert `try_instant_tail`'s output is
bit-identical."* That was paid at **step 2**, by `the_two_shipped_cells_are_invariant_under_an_arming`
(both cells called at one fixed state against the stale map and the design map — bit-identical on
`eta_lpt`, `eta_hpt`, `sp_thrust`, the whole `Instant2` and both power residuals, with the
anti-vacuity half asserting the two maps differ observably first). It is recorded here so slice W
does not re-derive it.

**THE GATE:** `cargo test --release`, status written into the log —
**`CARGO_EXIT=0`, 102 suites, 921 passed, 0 failed, 0 ignored** (step 4's 101 / 915, plus this
step's one suite and its six tests). Reconciled as a NAME diff and not only as a count:
`cargo test --release -- --list` reports **921**, and 921 minus this file's six names is exactly
step 4's **915** — **+6 additions, 0 removals**, because *exit-0 proves nothing FAILED; only the
diff proves nothing VANISHED*. `git diff -- rust/src/` is empty, checked AFTER the injection
harness's `finally` restored the source rather than inherited from before it.

### 5.21 SLICE W (rungs 62 + 63, `BleedSchedule` + `ScheduledBleedTransient`) — PRE-REGISTERED, five probes MEASURED first

`M:\claud_projects\temp\rust-phase7\probe_w1.py` … `probe_w5.py`, PyPy. Every table below is
**EMITTED by one of them**, not typed — § 5.19 (xi) is the reason, and this section's leading
finding is what that rule was written for.

#### (i) THE LEADING FINDING — **§ 5.19 (x)'s CELL COLUMN IS SHORT BY FOUR NAMES, AND THE CENSUS THAT SAYS SO RUNS OVER THE WHOLE PHASE, NOT OVER THIS SLICE**

§ 5.19 (x) books slice W as adding **four** cells — `_armed_bleed`, `_isolating`, `_legs`,
`b_of` — and classes `at_lever` with `at_stator` as *pure sibling constructors* needing none.
`probe_w1.py` applies § 5.19 (x)'s OWN two rules (a cell EXISTS at the slice porting its
earliest **caller**, and is SWAPPED at every slice porting an **overrider**) to
`ScheduledBleedTransient`'s body and reports:

| name | NEW at 62/63 | overridden downstream by | call sites |
|---|---|---|---|
| `at_lever` | NEW | **17 classes** (rungs 64 → 80) | **46** |
| `at_stator` | override of rung 57's | 1 (rung 64) | 12 |

**`at_lever` is the most-dispatched, most-overridden name in phase 7 and the plan gives it no
cell.** So the census was re-run over **every** remaining ladder class rather than over this
slice (`probe_w4.py`) — the whole error class in one pass, instead of nine slices inheriting it
one at a time. Emitted, against § 5.19 (x)'s hand-written column:

| slice | plan says | MEASURED | MEASURED-only (MISSED) |
|---|---|---|---|
| phase 6 | 3 | 6 | `_close`, `_instant_tail`, `_powers` — **already SHIPPED**, see below |
| **V** | 3 | **4** | **`at_stator`** |
| **W** | 4 | **5** | **`at_lever`** |
| X | 1 | 1 | — |
| Y · Z | 0 · 0 | 0 · 0 | — |
| AA | 9 | 9 | — |
| AB · AC | 1 · 1 | 1 · 1 | — |
| **AD** | 3 | **4** | **`_quad_gains_at`** |
| AE | 0 | 0 | — |
| **AF** | 3 | **4** | **`_with_coord`** |
| AG … AJ | 0 | 0 | — |
| **TOTAL** | **28** | **35** | |

The three phase-6 rows are **not misses**: `try_close`, `try_instant_tail` and `powers` already
ship in `TwoSpoolTransientHooks`, and § 5.19 (i)'s own arithmetic (*"38 = 28 new + 8 already
shipped + 2 Rust deletes"*) counts them in the 8. Netting them out, the genuine additions are
**four**: `at_stator` (V), `at_lever` (W), `_quad_gains_at` (AD), `_with_coord` (AF).

**"SUMS TO 28 BY CONSTRUCTION" IS RETIRED AND REPLACED BY AN EMITTED 35**, split by what is
on disk TODAY rather than by what the column called new: **8 already SHIPPED** (phase 6's
`_close`, `_close_fuel`, `_instant_tail`, `_powers`, `_surge_fuel` + slice V's `_arm`,
`_stator_march`, `v_of`) and **27 still to build** — `at_stator` and `integrate_fuel`, which are
*booked* at V and phase 6 but OPEN at W and Y, plus the 25 that later slices create.
§ 5.19 (xi) said every table in that section
should have been emitted rather than transcribed; this is the fifth defect from that one habit,
and the first found by running the emitter over the part of the plan the slice was not about.
[[rust-port-guessed-census-bars]] — *five typed count bars, five wrong* — one level up: the bar
here is a **membership list**, and the same rule applies to it.

#### (ii) **`at_stator` IS NOT A DEFERRAL. A SHIPPED RUNG-63 GATE READS IT DIRECTLY, AND THE PORT'S CURRENT SHAPE FLIPS ITS VERDICT**

§ 5.20's closing note booked `at_stator` as *"slice W's first job"* on the strength of a body
read: rungs 62 and 64 override it, eight inherited readers call it, and rungs 57–60 build only
rung-57 objects so *"it is inert here."* That is right about slice V and wrong about what slice W
inherits. Measured (`probe_w2.py`), on a bleed-armed machine:

| key | rung 62's override (Python) | FORCED rung-57 return — **the port's current shape** |
|---|---|---|
| `ordinate_identical` | `True` | **`False`** |
| `abscissa_identical` | `True` | **`False`** |
| `d_ordinate` | `0.0` | `9.543e-3` |
| `d_abscissa` | `0.0` | `1.019e-2` |
| `at_stator()._armed_bleed()` | `True` | **no such method** |
| `type(at_stator())` | `ScheduledBleedTransient` | `ScheduledStatorTransient` |

`tests/test_rung63.py::test_the_at_stator_trap_is_real_and_returns_rung59s_zero_for_free`
asserts **both identities**, on purpose: rung 62 overrode `at_stator` so that a rung-57 reader
on a bleed-armed machine differences against a sibling **carrying this machine's valve**, which
makes rung 59's `schedule_invariance` compare the plant with ITSELF and report rung 59's exact
headline **while measuring nothing**. The gate exists to pin that counterfeit. Under the port's
current `-> ScheduledStatorCore` shape it reads `False, False` and **the gate cannot even be
written**, because the returned type has no `armed_bleed`.

**So the return type is FORCED at step 1, and it is forced to a specific level of the tree.**
The gated reader's receiver is `&ScheduledStatorCore` and it calls `accel_schedule` on the
result, so the return type must stay `ScheduledStatorCore` — which means the valve state must
live **where a `ScheduledStatorCore` can read it without going through a rung-62 type**:
on `TwoSpoolTransientCore`, beside `StatorArming`. That single constraint decides the module
layout, and it was decided by a test rather than by taste.

**ONLY ONE OF THE EIGHT INHERITED READERS IS EXERCISED — a deferral WITH A COUNT (P4's rule).**
Emitted by grep over both suites: `schedule_invariance` **1** call in `test_rung63.py`; the
other seven (`stator_credit`, `credit_decomposition`, `composite_credit`, `engagement_shift`,
`matched_credit`, `set_point_bands`, `floor_composite`) are **0 and 0**. Their rung-62 behaviour
is UNGATED in Python. They are not all one shape: `stator_transient.rs:1740–1741`, `2031` and
`2404` pass stator **arguments**, so rung 62's cell must carry `self`'s valve *while honouring
the passed setting* — get that wrong for the arg-passing readers and the one gated reader still
passes. **Booked as step-4 oracle checklist item (a): `dump_slice_w.py` MUST carry a section
running all eight inherited readers on a bleed-armed machine, and the step-4 write-up states its
key count.**

#### (iii) **THE `at_lever` RETURN-TYPE POLICY, DECIDED NOW RATHER THAN AT SLICE X — AND THE LADDER TURNS OUT TO ADD ONLY FIVE FIELDS IN TWENTY-THREE RUNGS**

If `at_lever`'s cell were typed `-> ScheduledBleedCore`, rung 64 could not return its own type
and the port would re-open a **gated signature** at X, Y, Z, AA, AB, AC, AD, AE, AF, AG, AH and
AI — twelve repeats of the one thing § 5.19 (x) calls slice V's unique risk. `probe_w5.py`
emits what each ladder class actually adds in its own `__init__`:

| adds state | class (rung) | fields |
|---|---|---|
| 62 | `ScheduledBleedTransient` | `bleed`, `bleed_sched` |
| 64 | `LimitedBleedTransient` | `bleed_lim` (+ `_b_forced`, `_b_state`) |
| 68 | `ThreeLoopCascadeTransient` | `stator_lim` (+ `_v_forced`, `_v_state`) |
| 69 | `ReferenceSplitTransient` | `stator_inc` |
| **every other class, 65–84** | — | **nothing** |

**Five arming fields across twenty-three rungs, all plain scalars — the probe's own second table
found no class holding a lambda, a closure or a mutable default.** The four `_x_forced`/`_x_state`
names are § 5.19 (iv)'s dynamically-scoped RK4 fields, already booked there. And `at_lever`'s
keyword list grows **monotonically** — 6 at rung 62, 7 at 64, 8 at 68, 9 at 69 — and then stops:
rungs 70–80 repeat the same nine and rungs 81–84 define no `at_lever` at all.

**DECIDED: `at_lever`'s cell takes an ARM STRUCT, not a keyword list, and returns ONE type for
rungs 62–84.** A struct grows by adding a field with a `Default`, which is additive; an `fn`
pointer's parameter list does not, which is not. So the cell is
`fn(&ScheduledBleedCore, &LeverArm) -> ScheduledBleedCore`, `LeverArm` starts with rung 62's two
fields beside `StatorArm`, and every later slice adds its one field and swaps a body. **The
signature is never re-opened again.** The seventeen "overrides" become seventeen cell bodies
copying different subsets of one struct — which is what they already are in Python.

The cost is stated rather than hidden: one type can no longer refuse a rung-62 reader called on a
rung-57 object. Python cannot either — those methods are reachable on every subclass instance —
and the refusal that does exist is `loop_decomposition`'s own `assert`, which ports as a panic.
**The rung is the TABLE, not the type**, which is § 2's architecture and the reason it was
chosen.

#### (iv) **THERE IS NO NEW CARRIER — MEASURED WITH WITNESS KEYS, WHICH IS THE ONLY WAY THAT ANSWER MEANS ANYTHING**

Slice V's whole content was `_arm`'s permanent mutation of `map_lp`/`map_hp` from inside a
`&self` hook, and § 5.20 (i) is explicit that a `try/finally` census could not match it. Rung 62
*looks* pure — `b_of` has no history and no latch, and the valve is read fresh at every closure —
but "looks pure" is exactly what the earlier census said. So it was probed rather than read:
54 witness keys per machine (20 scalars, the four maps' seven settings **and their object
identities**, and the instance's own key set), snapshotted before and after a march
(`probe_w3.py`):

| machine | keys moved | what moved |
|---|---|---|
| bare | **0** | — |
| stator schedule | 2 | `map_lp.vsv` 0.0 → 0.01055, and the map's identity |
| **bleed schedule** | **0** | — |
| **bleed constant** | **0** | — |
| both levers | 2 | `map_lp.vsv` 0.0 → 0.01114, and the map's identity |

**Slice V's `Cell<ComponentMap>` carrier is the ONLY one, and rung 62 adds nothing to it** — the
two rows that move are the stator's, and they move on the *combined* machine too, at a different
value. This is a **negative measured with an instrument that has already been shown to fire**,
not a negative asserted from a body read. [[rust-port-slice-v-step3]]'s lesson (*an injection
whose only trace is object state reads exactly like one that never applied*) is why the identity
columns are in the key set at all.

#### (v) **THE REDUCE IS PER CALL AND IT IS LIVE MID-MARCH — AND THE DISPATCH IT TURNS ON HAS NO NATURAL RUST SPELLING**

Rung 62's reduce contract is *per call*: `b_of` is a pure function of the live state and each
overridden closure returns to its rung-57 parent **verbatim** whenever that value is `0.0`. The
schedule is exactly `0` at and above `n_ref`, so on a *scheduled* machine both branches fire
within one march. Emitted counts over one `equilibrium` + one `_stator_march` at `ds = 0.02`
(`probe_w3.py`):

| machine | `_close` reduced/bled | `_close_fuel` | `_powers` | `_instant_tail` |
|---|---|---|---|---|
| bare | 65 / 0 | 344 / 0 | 61 / 0 | 348 / 0 |
| stator schedule | 65 / 0 | 344 / 0 | 61 / 0 | 348 / 0 |
| **bleed schedule** | **12 / 53** | **0 / 344** | **12 / 49** | **0 / 348** |
| both | 12 / 53 | 0 / 344 | 12 / 49 | 0 / 348 |

**AND THE HAZARD THE COUNTS EXPOSE.** `_close`/`_close_fuel` dispatch on `b_of(nu_lp, Tt2)`, but
`_powers` and `_instant_tail` dispatch on **`c.get("bleed", 0.0)`** — a dict key that is
**ABSENT** from rung 57's returned closure, not present-and-zero. `CloseState` has no `bleed`
field and no `mdot_face` field (checked in `two_spool_transient.rs`), so slice W must add both,
and the "absent key" branch acquires no natural spelling: a Rust struct field is always there.
Filling `bleed` unconditionally, or "simplifying" `_powers` to re-read `b_of`, changes the
dispatch — **and no value key can see it**, because both paths agree wherever `b` is 0.
[[rust-port-slice-r-step4]]'s shape exactly (*a probe feeding BOTH sides the same wrong input
sees nothing*). **This needs a DISPATCH gate counting reduced-vs-bled per cell against
12/53, 0/344, 12/49, 0/348 — not a value key**, and it is P4 below.

#### (vi) THE SCOPE, ENUMERATED

**Source**: `BleedSchedule` **50** lines (8783–8832) + `ScheduledBleedTransient` **858** lines
(8835–9692) = **908**, all emitted by `ast`.

**Cells.** W **CREATES 5** — `at_lever`, `_armed_bleed`, `_isolating`, `_legs`, `b_of` — **OPENS
1** that slice V shipped without (`at_stator`, in `StatorTransientHooks`) and **SWAPS 4** that
already exist (`try_close`, `try_close_fuel`, `try_instant_tail`, `powers`). It does **not** swap
`try_surge_fuel`: rung 62 does not override `_surge_fuel`, so rung 62's `FuelTransientHooks` must
be spelled `{ try_close_fuel: r62_…, ..R57_FUEL }` and **never `..R43`** — inheriting the wrong
table drops rung 57's surge body silently, which is § 5.20 (v)'s carrier failure wearing a
different hat.

**Tests**: **88 collected** from **42 `def test_`** (58 from rung 62's 23, 30 from rung 63's 19)
— a 2.1× parametrize expansion against slice V's 59-from-57, so the ported gate count is **88**.
**55 of the 88 carry `slow` (62.5 %)**, above § 5.19 (viii)'s 49.2 % phase ratio; on slice V's
measured ~2 s per slow-heavy ported suite that is not a gate-time problem and **none becomes
`#[ignore]`** — slice M's `0 ignored` line holds.

**Module**: `rust/src/bleed_transient.rs`, on `stator_transient.rs`'s shape. `BleedSchedule` is
ported as its **own type**, NOT factored against `StatorSchedule`: `BleedSchedule`'s docstring
says the twinning is deliberate and load-bearing — same functional form, same two shapes, same
corner assert — because the rung's headline compares two **devices**, and one generic `Schedule`
with a shape enum would make it compare two spellings. [[rust-port-copy-vs-rederivation]].

#### (vii) PRE-REGISTERED — written before a line of Rust

- **P1.** The 88 ported gates pass with **zero tolerance tiers**, on the first oracle run, as
  slices V/U/T did. If any needs a tolerance, something in (iv)'s carrier verdict is wrong.
- **P2.** `test_the_at_stator_trap_is_real_and_returns_rung59s_zero_for_free` ports **verbatim
  in its assertions** — `ordinate_identical && abscissa_identical` on a bleed-armed machine —
  and it **FAILS on a build where `r62_at_stator` is replaced by `r57_at_stator`**, with the
  two `d_` keys landing at `9.543e-3` / `1.019e-2`. That failure is step 5's manufactured-bug
  gate, and (ii)'s table is its target.
- **P3.** The reduce is **bit-for-bit by DISPATCH**, per call and both ways: a `b = 0` machine
  is rung 57 on every recorded key, and a *scheduled* machine takes both branches inside one
  march at **12 / 53** on `_close`. The second half is the one a value gate cannot see.
- **P4 — the gate this slice owes.** A **dispatch gate** counting reduced-vs-bled per cell
  against (v)'s four pairs, on `slice_r_dispatch.rs`'s precedent. Predicted to be the only
  instrument that can see a `_powers` "simplified" to re-read `b_of`.
- **P5.** `at_lever`'s cell signature, once written with `LeverArm`, is **never re-opened by any
  later slice** — X…AI add a field and swap a body. Falsified if any of rungs 64–84 needs a
  parameter that is not expressible as a field with a `Default`.
- **P6.** Steps, on slice V's shape: **1** the cells + `LeverArm` + `CloseState`'s two new
  fields · **2** the port + `slice_w_smoke.rs` · **3** the two rung suites + the injections ·
  **4** `slice_w_oracle.rs` + `dump_slice_w.py`, carrying (ii)'s eight-reader section · **5** the
  dispatch gates (P4) and the manufactured `at_stator` bug (P2).

##### STEPS 1 + 2 — SHIPPED, AND GATED TOGETHER. **522 SMOKE KEYS BIT-EXACT ON THE FIRST REAL RUN — AND THE RUN BEFORE IT FAILED 243 OF THEM ON A CONSTANT I RE-SPELLED**

`rust/src/bleed_transient.rs` (**1 869 lines**), the cells opened in
`two_spool_transient.rs` / `stator_transient.rs` / `fuel_transient.rs`,
`rust/oracle/dump_slice_w_smoke.py` and `rust/tests/slice_w_smoke.rs`.

**WHY ONE GATE FOR TWO STEPS, STATED RATHER THAN ELIDED.** Step 1's `cargo test` was launched and
then, while it ran, step 2's edits recompiled the crate underneath it — so its binaries were no
longer the ones its result would have described. A gate whose subject changed mid-run is not a
gate, and quoting it would be exactly the *status read off the runner* hazard
[[windows-tooling-file-hazards]] already names. The two steps therefore share ONE run, and the
property step 1's own gate existed to prove — that rungs 40/43/57 are untouched — is carried
instead by **smoke section H**, which asserts the rung-57 and rung-62 objects agree
**bit-for-bit in Rust** on 15 keys × 2 throttles × 4 armings before comparing either to Python.

#### WHAT THE TWO STEPS ADDED

| | |
|---|---|
| **CELLS CREATED** | `at_lever`, `armed_bleed`, `b_of`, `isolating`, `legs` — [`LeverHooks`], with `NO_LEVER`'s five bodies PANICKING |
| **CELL OPENED** | `at_stator`, into rung 57's `StatorTransientHooks` — the one slice V shipped without |
| **CELLS SWAPPED** | `try_close`, `try_close_fuel`, `try_instant_tail`, `powers` |
| **STATE** | `LeverArming { bleed, sched }` on `TwoSpoolTransientCore`, beside `StatorArming` |
| **CLOSURE FIELDS** | `CloseState::{bleed, mdot_face}` and `Instant2::sp_thrust_inlet`, all `Option<f64>` |
| **CONSTRUCTORS** | `TwoSpoolTransientCore::with_lever_hooks`, `ScheduledStatorTransient::with_tables`, `build_scheduled_bleed` |

**`Option<f64>` IS NOT DEFENSIVENESS, IT IS THE PORT OF AN ABSENT DICT KEY.** `_powers` and
`_instant_tail` dispatch on `c.get("bleed", 0.0)`, and rung 40's and rung 57's closures return a
dict with **no `bleed` key at all**. `.unwrap_or(0.0)` IS `.get(_, 0.0)`; `mdot_face` is
`.expect`ed because Python INDEXES it and would `KeyError`. A plain `f64: 0.0` would read the same
on every machine the suites build and differently in general — § 5.21 (v).

**AND THE ONE THING THE PORT DOES THAT THE SOURCE DOES BY SHADOWING.** Python's `_close` binds a
local `mdot_face` (the `m_lp`-derived TRIAL face flow) and then, eleven lines later, returns a dict
key of the same name holding `mdot_imp/(1-b)` — the IMPOSED one. They agree only AT the root, so a
converged closure hides the swap and only `_powers` reads the key. The port names the local
`mdot_face_trial` and says so at both sites.

#### FINDING 1 — **THE FIRST SMOKE RUN FAILED 243 OF 522 KEYS, AND THE SECTION THAT LOCALISED IT WAS THE ONE WITH NO VALVE IN IT**

The failures were 1–10 ULP and they were everywhere: 71 in the bled closures, 40 in the fuel
closure, 22 in the rung's own `_legs`. Every one of those is a plausible rung-62 arithmetic slip,
and chasing them meant re-deriving `mdot4`'s spelling against rung 40's.

**Section H is what made that unnecessary.** It is the REDUCE — a `bare` machine, no valve, no
stator, a code path rung 62 never enters — and it failed 100 keys while its own in-Rust assertion
(`rung 57 == rung 62`, bit-for-bit) PASSED. A defect that reaches a path the slice does not touch
is not the slice's; it is the GRID's. The cause:

```text
test_rung62.py   R_c=(gamma_c - 1.0) / gamma_c * cp_c     ->  286.8571428571428
dump (as first written)   R_c=(0.4 / 1.4) * 1004.0        ->  286.8571428571429
```

`1.4 - 1.0` is `0.3999999999999999` in IEEE-754. **I re-spelled a derived constant as its
arithmetic value and built a gas one ULP away**, which drifted every number in the file. Fixed by
copying `test_rung62.py`'s `_cpg` character for character — and the docstring now says why, because
the next dump will be tempted the same way.

This is [[rust-port-copy-vs-rederivation]] pointed at an INSTRUMENT rather than at the port: the
rule is usually quoted about not factoring a deliberate duplication away, and here the same rule
forbids evaluating a deliberate derivation. **The generalisable half is the diagnostic one**: a
smoke dump should carry at least one section on a path the slice cannot reach, because that
section is the only one that can tell a wrong PORT from a wrong GRID. Section H was written for
the reduce and paid for itself as a control.

*Two of the five probes (`probe_w2.py`, `probe_w3.py`) carry the same wrong spelling. Their
published numbers are RELATIVE — ratios, differences and counts on one grid — so none of them
moves, and § 5.21's tables stand. Recorded rather than silently repaired.*

#### FINDING 2 — **THE PORT NEEDED NO NUMERICAL CORRECTION AT ALL**

After the grid was fixed, **522 of 522 keys were bit-exact on the first run**, with zero tolerance
tiers, over ten sections: the schedule twin's two shapes and both clip arms (A), `b_of` on all
three legs including the `Tt2` referral (B), both bled closures with `_powers` and
`_instant_tail` on top (C), the fuel closure at `b = 0.10` and `b = 0.30` (D), the `at_stator`
trap and the honest reader beside it (E), an inherited `phi`-floor leg (F), the four dispatch
count pairs (G), the reduce (H), `_legs` on both levers (J) and `_isolating`'s two siblings (K).

**THE DISPATCH COUNTS REPRODUCE § 5.21 (v) EXACTLY** — `12 / 53`, `0 / 344`, `12 / 49`, `0 / 348`
on a bleed-scheduled machine, and `65 / 0`, `344 / 0`, `61 / 0`, `348 / 0` on bare and
stator-scheduled ones. Section G is the only section that can see a `_powers` re-reading `b_of`;
sections A–F and H–K are structurally blind to it, and that is the point of counting.

**AND THE TRAP REPRODUCES.** `E/trap/ordinate_identical` and `abscissa_identical` both come back
`true` on a bleed-armed machine, with the honest `sensed_inputs` beside them at `9.543e-3` and
`1.019e-2` — § 5.21 (ii)'s two numbers, now on the Rust side. Step 5's manufactured-bug gate has
its target.

#### THE DECISION § 5.21 (iii) REGISTERED, CORRECTED IN ITS SPELLING

The registration wrote the cell as `fn(&ScheduledBleedCore, &LeverArm) -> ScheduledBleedCore`.
There is no `ScheduledBleedCore`: a newtype over [`ScheduledStatorCore`] cannot be what
`at_stator` returns (§ 5.21 (ii) forces that return type), so it would have bought a type
distinction for `at_lever` alone at the price of a `Deref` on every inherited reader. **The
decision is unchanged — one type, an arm struct, a signature no later slice re-opens — and only
its spelling moved**: `fn(&ScheduledStatorCore, &LeverArm) -> ScheduledStatorCore`. Rung 62's
readers are an `impl ScheduledStatorCore` block in `bleed_transient.rs`, exactly as Python's are
methods reachable on every subclass instance.

**THE GATE:** `cargo test --release`, status written into the log. *Recorded there as 98 suites / 910 passed; re-counted at step 3 the same log reads **103 result lines / 922 passed** — see step 3's gate note. The decomposition, not the total, is the check.*

##### STEP 3 — SHIPPED. **FIVE OF SIX INJECTIONS PASS ALL 88 GATES — AND THE INSTRUMENT THAT SAYS SO WAS WRONG FIVE TIMES, EVERY ONE OF THEM A ZERO THAT HAD NEVER BEEN MEASURED**

`tests/rung62.rs` + `tests/rung63.rs` — **58 + 30 = 88 `#[test]` against Python's 88 COLLECTED**,
one to one, zero source lines changed. `M:\claud_projects\temp\rust-phase7\inject_w.py`,
`slice_w_probe.rs.keep`, `injection_w_table.txt`, `bar_margins_w.txt`, `step3.log`.

**THE COUNT CENSUS, EMITTED ON BOTH SIDES AND PER FILE.** `pytest --collect-only -q` reports
**58** from `test_rung62.py` and **30** from `test_rung63.py` (from 23 + 19 `def test_`, a 2.1×
`parametrize` expansion) and `-m slow` reports **55 of the 88** — all three reproduce § 5.21 (vi)'s
pre-registered numbers — while `cargo test --release -- --list` reports 58 and 30. Equal totals can
still hide a missed port paired with an invented gate, so the NAMES were paired: a complete
bijection, **zero unmatched Rust names in either suite**, every low-scoring pair a `parametrize`
family member, and each family's membership checked against the Python list it came from
(`[{}, BLEED, dict(bleed=0.15)]` → `bare` / `bleed_sched` / `bleed_const`, at 0.15).
*A `grep -c '#\[test\]'` gives 59 and 31: line 4 of each file is the header docstring, which
contains the literal string.*

#### FINDING 1 — **THE FIRST THREE DEFECTS OF THIS STEP WERE ALL IN THE GATES, AND THE THIRD WAS A GATE FAILING ON THE MESSAGE IT EXISTS TO ACCEPT**

The suites did not compile. Both `catch_unwind`s in `rung63.rs` capture `&ScheduledStatorCore`,
which carries slice V's `Cell<ComponentMap>` and the gas caches — **7 errors from one site**,
repaired with `AssertUnwindSafe` on `rung54.rs:517`'s precedent. `rung62.rs` read
`od.base.pi_lpc` where the field sits one level further down. Neither is interesting.

The third is. `_isolating`'s two refusals carry **different panic payload types**: the keyed one
interpolates `{k}` and unwinds a `String`, the empty-lever one is a bare literal `assert!` and
unwinds a `&'static str`. The gate downcast only to `String`, so a **correctly matched** refusal
read back as the empty string and the test failed with `wrong refusal: ` — on the very message it
was written to accept. Repaired by porting `rung57.rs`'s `panic_text`, which tries both, with the
why written at the site. **A gate can be red for the reason it is right**, and its own failure
text says the opposite.

#### FINDING 2 — **THE INJECTION TABLE. FIVE OF SIX INJECTIONS PASS ALL 88 GATES**

Six injections over **five distinct defects** (I2 and I2b are two spellings of one), five of them
§ 5.21 (v)/P2/P4's own pre-registrations. `moved_G` counts the **871 gate-visible** probe keys —
every reader the two suites call, on their own grids; `moved_W` counts the **86 witness** keys
**no ported gate reads**; `absent` counts baseline keys the injected probe never emitted.

| injection | moved G / 871 | moved W / 86 | absent | **caught / 88** |
|---|---|---|---|---|
| **I1** `mdot_face` = the TRIAL face flow | **312** | 2 | 0 | **0** |
| **I2** `_powers`/`_instant_tail` re-read `b_of` | **0** | 7 | 0 | **0** |
| **I2b** the same re-read, PREDICATE only (P4's) | **0** | 7 | 0 | **0** |
| **I3** `R62_FUEL` spread from `..R43` | **0** | 1 | 3 | **0** |
| **I4** `at_stator` left un-overridden | 4 | 1 | 1 | **2** |
| **I5** the `1/(1-b)` off the fuel bracket walls | **151** | 2 | 0 | **0** |

**I4 REPRODUCES § 5.21 (ii)'s PYTHON MEASUREMENT TO THE DIGIT, ON THE RUST SIDE.**
`G/trap/ordinate_identical` and `abscissa_identical` flip **`true`/`true` → `false`/`false`** and
the two `d_` keys land at **`9.54314506e-3`** and **`1.01882344e-2`** against the pre-registered
`9.543e-3` / `1.019e-2`. The two gates that fire are exactly the two written for it, so **P2 is
discharged on the port** and step 5's manufactured-bug gate has a measured target rather than a
promised one.

**AND THE TWO BIG ONES ARE CAUGHT BY NOTHING.** I1 moves 312 keys and I5 moves 151, and not one
gate in either suite fires. Their real worst moves are **4.3e-12** and **2.2e-12**, medians around
**7e-15** — every gate in both suites is relational, and a shift at the twelfth figure moves both
sides of every relation it asserts. That is the disclosure now written into both file headers,
**generated from this table rather than typed**.

*The `worst_rel` column reads `1` on four rows and means nothing. The maximum lands on
`G/fd/bled/row4/credit`, a difference of two near-equal numbers whose baseline is `-2.22e-16` —
machine epsilon — so one ULP there is a "100 % relative move". The harness already excludes an
EXACT-zero baseline (before that guard the column read **1.02e+298**, from `9.54e-3 / 1e-300`); it
does not exclude a near-zero one. The absolute figures above are the honest ones, and the column
is left in with this note rather than quietly patched.*

#### FINDING 3 — **P4 NAMED THE WRONG COUNTER. THE `reduced`/`bled` PAIRS ARE BLIND TO THE DEFECT THEY WERE BUILT FOR**

§ 5.21 (v) predicted that a `_powers` "simplified" to re-read `b_of` would be visible **only** to a
dispatch gate counting reduced-vs-bled per cell, and P4 registered that gate as the one slice W
owes. Measured on both spellings — I2 rebinds `b` outright, I2b changes only the branch predicate,
which is P4's own case — on the grid that reproduces (v)'s table exactly:

* **All eight `close_*`/`close_fuel_*`/`powers_*`/`tail_*` counters DO NOT MOVE.**
  `b_of(nu_lp, Tt2)` and `c.bleed.unwrap_or(0.0)` agree at **every** call on this plant, not merely
  where `b` is 0, so the dispatch is identical and the pairs cannot see it.
* The only thing in the entire instrument that moves is **`b_of`'s CALL COUNT**: 409 → 818 on every
  machine, with `b_of_sched_zero` 12 → 24 and `b_of_sched_open` 397 → 794 — the function being
  consulted exactly twice as often.

So P4's verdict survives (*no value key can see it*) and **its instrument does not**: the pairs it
named are as blind as the 871 value keys, and what betrays the re-read is the call count of the
re-read function. [[rust-port-slice-r-step4]] is *a probe feeding BOTH sides the same wrong input
sees nothing*; this is one level up — **a counter can be blind because the two branches it
separates are never actually separated.**

#### FINDING 4 — **I3 IS INVISIBLE BECAUSE THE SUITES NEVER BUILD THE INPUT THAT SEPARATES THE TWO BODIES**

`..R43` versus `..R57_FUEL` swaps `try_surge_fuel`, and the two are genuinely different functions —
but `r57_try_surge_fuel` is a **wrapper**: it RESOLVES the floor and then delegates to
`R43.try_surge_fuel`. On a `Floor::Phi` that resolution is the identity, so the two bodies agree
exactly. **Both ported suites build only `Floor::Phi`** — once, in rung 63's
`every_march_stays_choked` — and never `Floor::Incidence`, the one input the resolution step
changes.

A zero measured on inputs that cannot discriminate is not a measurement, so the probe carries a
**detector**: one `Floor::Incidence` cell on a path no ported gate reaches. Under `..R43` it
**PANICS** (rung 43's body is handed rung 60's object and `Floor::phi()` refuses), and
`W/guard/detect` flipping `1 → 0` is the ONLY key in 957 that sees I3 at all. The finding is
therefore not *"the carrier defect is undetectable"* but **"the 88 gates are blind to it because
they never construct the floor kind that distinguishes the two bodies"** — slice U's *a function
exercised only on cells chosen for INERTNESS*, arriving on an inherited table spread.

#### FINDING 5 — **THE PROBE WAS WRONG FIVE TIMES, AND EVERY ONE WOULD HAVE PUBLISHED A ZERO THAT WAS NEVER MEASURED**

The instrument had to be repaired five times before the table above could be believed. Each is a
distinct way to report a zero nothing looked at.

1. **THE HEADER CLAIMED COVERAGE THE CODE DID NOT HAVE.** The first version said its keys were
   *"the readings the 88 ported gates actually compare"* while dumping `marginal_loop` on a BARE
   machine; the headline gates call `loop_decomposition` on an ARMED one — a different reference
   path (`bare_lever()`, not `isolating()`) — and **nine readers were missing outright**. Every
   name the suites call on a core was then enumerated off their source and dumped: 435 keys →
   **871**. [[rust-port-slice-s-step4]] verbatim, on my own probe.
2. **A SECTION THAT ECHOED ITS OWN INPUT.** `floor_dichotomy`'s rows were dumped as `row{i}/sm` —
   the `sm` grid handed in — beside `min_phi_ref`/`min_phi_armed`, which come from the leg-FREE
   cells. The section named for the floor recorded **no floor-armed reading at all**. Replaced by
   the rows' nine computed fields.
3. **THE WITNESS BLOCK RAN LAST, SO A PANIC MADE ITS ZEROS COUNTERFEIT.** I4 panics in the trap
   section; with witnesses emitted afterwards it reported `moved_W = 0` having emitted **no witness
   key at all** — 444 absent. Witnesses now emit FIRST, every section runs under a `guard(name, …)`
   that catches its own panic and emits `W/guard/<name>`, and the detector runs LAST so it can
   truncate nothing. Absent counts fell to **0 / 0 / 0 / 3 / 1 / 0** — and the two survivors are
   *reported*, not hidden: I3's 3 are the detector's own keys, and **I4's 1 is a GATE-VISIBLE key**
   (`G/trap/sibling_armed_bleed`, emitted inside the guarded closure after the four that moved), so
   `G/trap`'s coverage under I4 is 4 of 5 rather than 5 of 5.
   [[rust-port-slice-s-step3]] — *injections reporting "nothing moved" could not have moved
   anything*.
4. **THE COHERENCE CHECK COULD NOT FAIL.** The check written to prove the probe is not blind —
   *`caught > 0` must imply `moved_G > 0`* — filtered rows on `status == "OK"`. I4 is the ONLY
   injection any gate catches and its probe panicked, so the filter dropped it, examined **zero
   rows**, and printed success either way. [[rust-port-slice-v-step2]] — *both gates written to
   CLOSE a step could not fail* — reproduced by me inside the check written to close that hazard.
   It now prints the number of rows it examined: **1 injection had `caught > 0`, and it moved
   gate-visible keys.**
5. **THE WITNESS WORKLOAD WAS ON A DIFFERENT GRID FROM THE TABLE IT REPRODUCES.** § 5.21 (v)'s
   counts come off `probe_w3.py`, whose workload is `equilibrium(FLIGHT, LO)`; the probe ran
   `equilibrium(…, 1200)`. The **scheduled** row still reproduced exactly — 12/53, 0/344, 12/49,
   0/348 — while **bare** came back 62 against the pre-registered 65, which reads as a port defect
   and would have opened step 5's dispatch gate red. On `LO` all twelve pairs reproduce on all
   three machines: bare and stator **65/0 · 344/0 · 61/0 · 348/0**, bleed **12/53 · 0/344 · 12/49 ·
   0/348**. **One row of a table matching exactly is not the table matching**
   ([[rust-port-slice-n-step4]], *two censuses on two grids*), and the earlier claim that "the four
   pre-registered pairs still reproduce" was true of one machine and unverified for two.

*And a sixth, on the reading rather than the instrument: a `until grep -q …` wait-loop matched the
PREVIOUS run's output file microseconds before the new run truncated it, so a re-run's results were
read off a stale artefact and briefly believed. The fix is to key on the task's own completion, not
on a pattern in a file the job is about to overwrite — [[windows-tooling-file-hazards]]'s "status
read off the runner", in a wait-loop.*

#### THE BAR-MARGIN TABLE — **80 BARS, AND IT CAUGHT TWO OF ITS OWN TRANSCRIPTIONS BECAUSE IT RE-CHECKS THE VERDICT**

Slice T step 2's lesson (9/9 green and blind to 24 %). Computed in Python off the probe rather than
by adding a `bar!` macro to 89 assert sites, because mutating a 1:1 port costs more than the
disclosure buys — so the table has a SCOPE, declared inside the artefact rather than left implicit.

* **The tightest live bars are rung 62's published bands**: `bleed self_cancel < 1.11` at **1.0 %**
  headroom, then `> 1.08` at 1.2 %, `stator < 0.85` at 2.2 %, and `bleed nu0_armed < nu0_ref` at
  2.5 %. Rung 62's headline is gated inside a band about one part in a hundred wide on each side;
  it is not a loose sign test.
* **The two `floor band` identities read EXACTLY 0 against a `1e-12` bar** — exact in arithmetic,
  so that tolerance does no work at all. [[rust-port-slice-t-step1]]'s *an EXACT ZERO blinds its own
  gate*, recorded rather than repaired: the identity is right and the bar is simply inert.
* **Two rows came back `**VIOLATED**` on a green suite, and both were MINE.**
  `bleed_triples_the_surrender` arms the **stator** (with the bleed carried as neighbour on both
  sides), not the bleed; and the floor band is asserted only on the bleed lever, so a stator band
  row was a bar this table invented rather than one it ported. A margin table that only printed
  headroom would have published both as plausible numbers — **it is re-checking the comparison that
  turns a mis-transcription into a failure instead of into a datum.**
* **EXCLUDED, and why:** ~27 pure sign tests (`> 0.0` / `< 0.0` — a sign test has no bar, only a
  magnitude; four kept as a sample), every `to_bits()` equality (a reduce gate is exact, not
  barred), and the bars whose operands need a NEIGHBOUR lever (`beside.surrendered`,
  `sched > 2.2 × over`, the super-additivity trio) — the probe passes `neighbour = None` throughout.
  That last is the one real gap.

#### THE PROBE IS PRESERVED, NOT SHIPPED

`slice_w_probe.rs` has no assertions — it prints. A no-assertion `#[test]` is a vacuous gate and
would have added an 89th test that can never fail, so it lives at
`M:\claud_projects\temp\rust-phase7\slice_w_probe.rs.keep`, restorable by copying it back;
`inject_w.py` beside it takes injection names as arguments and restores the source in a `finally`.
**Every substitution declares the number of sites it must match**, and that check earned itself
immediately: I1's two `mdot_face` sites differ only by indentation, so the 12-space pattern is a
SUFFIX of the 16-space one, matched twice, and the injection was SKIPPED rather than half-applied
— [[rust-port-slice-t-step4]]'s *an injection matching TWICE applies nothing and still reports
green*, refused at the door.

#### WHAT STEP 5 INHERITS — TWO INSTRUMENT CORRECTIONS, REGISTERED HERE SO THEY CANNOT BE MISSED

P4 and § 5.21 (v) specify gates that this step measured to be blind. Step 5 must build them
differently:

- **The dispatch gate must count `b_of_calls`, not only the four reduced/bled pairs.** Finding 3:
  I2 and I2b move the call count (409 → 818) and leave all eight pairs untouched, so a gate on the
  pairs alone would be exactly the instrument that measured nothing here.
- **The carrier gate must build a `Floor::Incidence` cell.** Finding 4: on `Floor::Phi` — the only
  kind either suite constructs — `..R43` and `..R57_FUEL` are identical, so a `Phi`-only gate
  cannot separate them.

**THE GATE:** `cargo test --release`, whole log, status derived from the LOG BODY —
**105 result lines (103 integration suites + lib unittests + doc-tests), 1010 passed, 0 failed,
0 ignored**, with `rung62`/`rung63` present and `slice_w_probe` absent. It DECOMPOSES:
the crate before this step was 101 integration suites and **922** passed, and 922 + 58 + 30 =
**1010**. *That decomposition is the check, not a remembered total — the steps-1+2 record in this
section says "98 suites, 910 passed" and the same log re-counted now reads **103 / 922**, so the
baseline it quotes is an undercount.* **NO PYTHON SOURCE CHANGED**, so `pytest` is untouched by
this step and is not re-run.

*Derived from the body and not from the exit code because this session watched
`cargo test … | tail -60` return **exit 0 over a seven-error build failure** — a pipeline's status
is the LAST command's, and `tail` always succeeds. [[windows-tooling-file-hazards]]'s "status read
off the runner", one pipe further along.*

##### STEP 4 — SHIPPED. **9 422 KEYS BIT-EXACT AGAINST PyPy ON THE FIRST REAL RUN — AND THE TWO RUNS BEFORE IT WERE BOTH THE INSTRUMENT, ONCE AS A CORRUPTED GOLDEN AND ONCE AS A PREDICATE I HAD RENAMED MYSELF**

`rust/oracle/dump_slice_w.py`, `rust/oracle/slice_w_pypy.tsv` + `slice_w_cpython.tsv`,
`rust/tests/slice_w_oracle.rs` (**2 `#[test]`** — one arm per interpreter). **NO PYTHON SOURCE
CHANGED**, so `pytest` is untouched by this step.

Ten sections, on the two suites' own grids plus three arms no suite reaches: the schedule twin
(A), `b_of` and the `Tt2` referral (B), the forward closure on 2 map shapes × 6 armings including
`_powers`/`_instant_tail`/`_close_fuel` (C), rung 62's six readers at `ds = 0.01` (D), rung 63's
five at `ds = 0.005` (E), `_isolating`/`_legs` (F), the `at_stator` trap and the honest reader
beside it (G), **the eight inherited readers (H, ADDED)**, the REDUCE as the control section (J),
and **the dispatch census (K, ADDED)**.

#### FINDING 1 — **THE CPython ARM FOUND A DIVERGENCE IN PYTHON'S OWN `sum()`, NOT IN THE PORT**

Seven of nine `D/cl/*/mean` keys disagree between the interpreters. The mechanism was measured
rather than guessed: **CPython 3.12+ uses Neumaier COMPENSATED summation in `sum()` for floats;
PyPy's is naive left-to-right.** `commanded_level` computes `sum(vals)/len(vals)`, so on a
CONSTANT valve CPython returns exactly `0.1` (`3fb999999999999a`) and PyPy returns the accumulated
value three ULPs below it — `sum(v)/len(v)` and a hand-written loop agree on PyPy and differ on
CPython, which is the whole test.

**IT IS A FINDING, NOT A DEFECT TO ROUTE AROUND.** The crate matches PyPy, which is the project's
interpreter and the one every golden is generated on; and **no shipped gate reads `mean`** —
`test_rung62.py:374` reads `at_min`. So the divergence is real in the shipped Python and
load-bearing for nothing. The oracle carries it as its **single declared exemption**, and as a
RULE (`D/cl/` … `/mean`) rather than a hand-list, checked from both ends: the rule must cover
exactly 9 keys, and **at least one drift must land inside it** — a suppression rule that
suppresses nothing is a rule nobody has looked at since it was written.

**AND IT FALSIFIED A CLAIM IN THE SHIPPED SOURCE.** `bleed_transient.rs`'s own comment said
left-to-right accumulation "is what a plain `iter().sum()` also gives" — true of PyPy, false of
CPython 3.12+. That comment is this step's **one source line**, so the step's `git diff --
rust/src/` is one comment block and not empty. [[rust-port-slice-l-step3]] is *a claim in the
SHIPPED source was false*; this is the first time a CROSS-INTERPRETER arm is what caught one.

#### FINDING 2 — **A DISCRETE FLIP IN FOUR KEYS OF A HUNDRED, AND IT WAS MY OWN KEY RATHER THAN THE PORT**

The CPython arm's first run reported `H/at_stator/{sched,const}/{vsv_lp,vsv_hp}/is_armed_stator`
flipping `rust 1 vs cpython 0`. Cause: **Python's `_is_armed()` is SCHEDULED-ONLY, and the Rust's
`is_armed()` is the COMPOSITE guard** `_is_armed() or vsv_lp or vsv_hp` that six of section H's
readers open with — a deliberate rename, recorded in the Rust method's own doc comment, with
Python's `_is_armed()` ported as `is_scheduled()`. `r57_arm`'s early return correctly reads the
scheduled one, and every other call site is porting the composite guard: **the port is right and
the oracle key was asking two different questions on the two sides.**

The instructive part is the ARITHMETIC of the miss. Over a hundred `is_armed_stator` keys agreed
and exactly **four** flipped, because the two predicates coincide on every input except a CONSTANT
stator with no schedule — a shape that exists in this whole file only in section H (b)'s argument
sweep, which was added for a different reason. A hundred agreeing keys is what made four
disagreeing ones read as a defect. Both predicates now have their own key on both sides
(`is_scheduled_stator` and `guard_armed_stator`), so neither is a naming accident and the four
discriminating cells stay discriminating. **How to apply: when a port deliberately RENAMES a
predicate, an oracle key must name WHICH ONE it asks for — the rename is invisible on every input
where the two agree, which is almost all of them.**

#### FINDING 3 — **CHECKLIST ITEM (a)'s OBVIOUS FORM EMITS NOTHING: SIX OF THE EIGHT READERS REFUSE THE MACHINE THE ITEM NAMES**

§ 5.21 (ii) booked *"a section running all eight inherited readers on a bleed-armed machine"*.
Measured: `credit_decomposition`, `composite_credit`, `engagement_shift`, `matched_credit`,
`set_point_bands` and `floor_composite` all assert `_is_armed() or vsv_lp or vsv_hp` — a **STATOR**
arming — and a bleed-only machine has none. Written as booked, six of the eight would have raised
and the section would have carried two readers while reading like eight.

So the section arms BOTH devices and **records the six refusals as their own keys**
(`H/refuses_bleed_only/*`, with the two non-refusals beside them so the row is a measurement
rather than an assumption). On the Rust side those are `catch_unwind` readings, which makes the
refusal itself a ported fact.

**AND A SECOND REFUSAL SHAPE, WHICH IS NOT AN ARMING ONE.** On the both-SCHEDULED machine
`matched_credit` at `margin = 0.25` trips rung 59's **own clamp audit** — the schedule is consulted
outside the derived bracket at 3 of 210 cutting points, so the number would be an envelope edge
rather than the derived shape. Recorded as `H/clamp_refusal/*` and the reader then RUN at 0.40,
where the bracket contains the march. Tuning the margin silently would have hidden a real property
of the plant; the two keys make it a reading.

#### FINDING 4 — **THE CENSUS'S FIRST FORM REPORTED FIVE ZEROS NOBODY HAD LOOKED AT**

Section K reproduces § 5.21 (v)'s four reduced/bled pairs exactly on all five armings — `65/0 ·
344/0 · 61/0 · 348/0` bare and stator, `12/53 · 0/344 · 12/49 · 0/348` scheduled and both — and
`b_of_calls` at **409** with `b_of_sched_zero` 12 / `b_of_sched_open` 397, which is step 3's
Rust-side measurement reproduced independently in Python.

But five of its seventeen counters — `at_lever_calls`, `at_stator_r62`, `isolating_calls`,
`legs_calls`, `legs_lever_bleed` — read **0 on every arming**, because `equilibrium` +
`_stator_march` never constructs a sibling. A dead counter and an untaken path are the same
character. A **second workload** was added (`sib_*`: `loop_decomposition` + `marginal_loop` +
`schedule_invariance`) on which all five are non-zero, which is what makes the zeros above measured
zeros; `slice_w_dispatch.rs`'s gate 1 asserts both directions.

**AND THE PYTHON WRAPPERS HAD TO MOVE FROM THE INSTANCE TO THE CLASS.** Rust's counters are a
thread-local GLOBAL — `bump(&CLOSE_BLED)` fires wherever the cell runs, on `self` or on any sibling
alive at the time. An instance-level patch counts one object and silently under-reports every
sibling: invisible on the `march` workload (which builds none) and wrong by a factor on the
`siblings` one (which marches four machines). The classifiers call the SAVED ORIGINAL `b_of`, so
they never inflate the count they are recorded beside.

#### FINDING 5 — **THE ONE `ADDED` ARMING WHOSE VALUE I HAD ASSERTED RATHER THAN MEASURED, AND IT PAYS**

Section D carries a `sat` arming at rung 57's knee (`n_lo = 0.75574`) because the saturated corner
is a real state of the schedule and no suite exercises it. The obvious worry is that it is
DEGENERATE: at that knee the bleed machine idles at 0.737, **below** the knee, so the schedule
starts the ramp clipped at `b_max` where `db/dn = 0` — which is exactly `test_rung62.py:57`'s
stated reason for moving the knee to 0.65, and if it held over the whole march the arming would
be the CONSTANT leg under another name and the `ADDED` label would oversell it.

Diffed key for key against `D/ld/const/*`: they agree on **21 of 54**, and those 21 are the
LEVER-INDEPENDENT keys — `reference`, `nu0_ref`, `s_ref`, `r`, and the two labels. **Every rung-62
headline quantity differs**, because the ramp carries `n` back over the knee mid-march and the
schedule comes off the clip. `self_cancel` lands strictly BETWEEN the two legs at all three rates:

| `r` | const (no loop) | **sat** | bled | loop retained |
|---|---|---|---|---|
| 0.25 | 1.0205 | **1.0480** | 1.0990 | **35 %** |
| 0.50 | 1.0289 | **1.0417** | 1.0971 | **19 %** |
| 1.00 | 1.0349 | **1.0378** | 1.0930 | **5 %** |

So the suite's own reason for moving the knee is **CONFIRMED and given a number**: the placement
ATTENUATES the loop rather than removing it, and it attenuates it more the faster the ramp. The
process point is the one this whole slice keeps making — *"the saturated corner is a real state
and no suite exercises it"* was an argument for carrying the arming, not evidence that carrying it
measures anything, and one `awk` over the golden was the difference between the two.

#### FINDING 6 — **TWO RUNS LOST TO THE INSTRUMENT'S PLUMBING, AND NEITHER FAILURE NAMED A KEY**

* **A `2>&1` CORRUPTED A GOLDEN.** The dump prints its key count to stderr; redirecting stderr into
  the tsv merged `# 9396 keys` onto the END of the last data line with no newline between them.
  The Rust failed with `ParseIntError { kind: InvalidDigit }` **and no key name**, because the
  failure is in the loader, before any comparison exists. [[windows-tooling-file-hazards]]'s
  family, in a redirect: a diagnostic stream written into a data file is not a diagnostic any more.
* **A SIGNED INTEGER IN A `u64` FILE.** `sign_bleed` / `sign_stator` are `-1` or `+1`, and the
  format is `key<TAB>u64`. Python wrote `-1`; the same `ParseIntError`, again naming nothing. Fixed
  by masking to two's complement in the dump's `d()`, which is exactly what the Rust's `as u64` on
  an `i32` already produced.

#### THE TWO SMALLER DECISIONS, RECORDED RATHER THAN LEFT IMPLICIT

* **NaN IS CANONICALISED ON BOTH SIDES, AND IT IS NOT A TOLERANCE.** 66 keys are legitimately NaN
  (`s_eng` where a leg never crosses; `erosion` as `0/0` on a spool the LP lever does not reach). A
  NaN's bit pattern is not portable — CPython's `float('nan')` is the POSITIVE quiet NaN while an
  x86-64 `0.0/0.0` unwinds NEGATIVE — so comparing raw bits would fail on **the sign of a NaN**,
  which carries no meaning. Both sides emit `0x7FF8…`; every other bit is compared exactly.
* **ONE KEY IS ONE-DIRECTIONAL AND SAYS SO.** `C/fuel/*/has_face_key` is a literal `true` on the
  Rust side: `FuelCloseState::mdot_air_face` is a plain `f64` and the type cannot express absence.
  It is not vacuous — if the Python dict ever stopped writing that key the golden would read 0
  against this 1 — but it can never fail in the direction of the port, and the asymmetry is written
  at the site rather than left to look like a measurement of both sides.

**THE GATE:** **9 422 keys compared on BOTH arms, zero tolerance tiers**, green on the first run
after the two instrument repairs. The CPython arm's only disagreements are **7 of the 9 declared
`sum()` keys** -- `diff` over the two goldens returns those seven lines and nothing else.

##### STEP 5 — SHIPPED. **THE TWO INSTRUMENTS STEP 3 REGISTERED AS BLIND, PLUS P2 — AND A MUTATION TABLE IN WHICH ONE OF MY OWN GATES LETS A HALF-APPLIED INJECTION THROUGH**

`rust/tests/slice_w_dispatch.rs` — **5 `#[test]`**, **zero source lines** (every injected table is
built from already-`pub` seams: `R57`, `R43`, `R62_TWO`, `R62_FUEL`, `R62`, `with_tables`,
`LeverArming`). The `at_stator` injection needs no new item at all — `R62_STATOR` **is**
`{ at_stator: r62_at_stator, ..R57 }`, so the machine with the override removed is the machine
built on `R57` itself.

#### THE TWO CORRECTIONS STEP 3 OWED, BUILT

* **P4's instrument replaced.** § 5.21 (v) predicted the `_powers`-re-reads-`b_of` defect would be
  visible only to the four reduced/bled pairs; step 3 measured all eight counters STILL. Gate 2
  asserts **both halves**: `b_of_calls` rises by exactly `powers_total + tail_total`
  (**409 → 818** on the scheduled machine, `b_of_sched_zero` 12 → 24, `b_of_sched_open` 397 → 794),
  **and every one of the eight pairs is unchanged** — plus a file-local counter proving the two
  predicates selected the SAME branch at every call, which is *why* the pairs are blind. Without
  that second half the gate would be the instrument that measured nothing.
* **The carrier gate builds a `Floor::Incidence` cell.** Gate 4 asserts the `Phi` half AS THE
  FINDING — on a `Floor::Phi` the rung-57 wrapper's resolution is the IDENTITY, so `..R43` and
  `..R57_FUEL` agree bit for bit — and then that `Incidence` REFUSES under the wrong spread. **And
  the set point is chosen so the leg BINDS**: at `sm = 0.40` the floor is DORMANT on a bleed-armed
  plant (the valve has already lifted `min_phi` to 0.7887 against a `phi_lim` of 0.7700), so each
  cell asserts its own `fuel_removed > 0` and the two armings carry different set points — 0.43
  bare, 0.46 bled. Two inert paths agreeing is not an agreement.

#### **P2 IS DISCHARGED, AND THE WITNESS IS SHARPER THAN § 5.21 (ii) PREDICTED**

The clean side is read from the committed golden **by KEY** — the defence against a golden
regenerated on buggy code, and the one failure mode the oracle alone has. The injected side flips
`ordinate_identical` / `abscissa_identical` `true/true → false/false` with the two `d_` keys at
`9.543e-3` and `1.019e-2`, each inside the bar its own last printed decimal licenses (margins
1.45e-7 in 5.0e-7 and 1.77e-6 in 5.0e-6).

§ 5.21 (ii) recorded `at_stator()._armed_bleed()` on the un-overridden sibling as *"no such
method"* — Python's `AttributeError`. **The Rust does the same thing with a TABLE**: `r57_at_stator`
builds a core carrying `NO_LEVER`, whose `armed_bleed` cell PANICS rather than answering `false`,
with the crate's own comment saying why it declines to make a claim no value gate could see. That
refusal is the port of the AttributeError and is a STRONGER witness than a `false` would have been.

#### **THE MUTATION TABLE — AND THE ONE THING IT MEASURED THAT NOBODY ASKED IT TO**

Slice V step 5's closing lesson (*a gate that MANUFACTURES a bug is code too — inject its own
wrapper*), executed. Each injection was neutered in turn and the gate that exists to catch it had
to go red:

| mutation | gate that must catch it | verdict |
|---|---|---|
| M1 `powers` stops re-reading `b_of` | `b_of_is_the_only_counter_…` | **CAUGHT** |
| M2 `tail` stops re-reading `b_of` | `b_of_is_the_only_counter_…` | **CAUGHT** |
| M3 the `at_stator` table is NOT swapped | `the_at_stator_override_…` | **CAUGHT** |
| M4 the fuel spread is `..R57_FUEL` | `the_r43_spread_…` | **CAUGHT** |
| M5 a census reading off by one | `the_dispatch_census_…` | **CAUGHT** |
| M6 BOTH re-reads removed | as M1, **plus** `each_injection_changes_…` | **CAUGHT ×2** |

**UNDER M1 AND M2 — ONE re-read site removed — GATE 5 STAYS GREEN, AND ONLY M6 FIRES IT.** Gate 5
asserts `b_of_calls` merely ROSE; gate 2 asserts it rose by EXACTLY `powers_total + tail_total`.
With one site still re-reading the count rises anyway, so the `>` test is satisfied by a
HALF-APPLIED injection and would have reported it as applied. The exact-accounting gate is strictly
stronger. [[rust-port-slice-t-step4]] is *an injection matching TWICE applies nothing and still
reports green*; this is one notch over — **an injection applying ONCE where it should apply twice,
and the gate that cannot tell.** Recorded rather than repaired, because gate 2 already carries the
exact form and gate 5's job is the cross-channel isolation.

#### AND A SHIPPED STEP-2 SECTION WHOSE CLAIM WAS CORRECTED

`slice_w_smoke.rs`'s header item 5 said section F *"runs a `phi` floor on a bleed-armed machine"*
to catch the `..R43` spread. It cannot, for two reasons found only now: a `Floor::Phi` cannot
separate the two tables at all (step 3, finding 4), and `F/floor_leg/fuel_removed` is **exactly
0** on that grid — the valve has already disarmed the floor, so the section compares two inert
paths. The section is left as it is (its keys are real readings of an inherited leg, and its golden
is bit-exact) and its CLAIM is corrected in place, pointing at gate 4 as where that check now
lives. [[rust-port-documented-gate-that-doesnt-exist]], caught by the step that replaced it.

#### THE GATE FOR STEPS 4 + 5

`cargo test --release`, whole log, status derived from the **LOG BODY** and checked by
DECOMPOSITION rather than against a remembered total -- this section has already recorded two
wrong baselines that way. **107 result lines (105 integration suites + lib unittests +
doc-tests), 1 017 passed, 0 failed, 0 ignored.** It decomposes exactly: the crate after step 3
was 105 result lines and **1 010** passed, and `1010 + 2 (slice_w_oracle) + 5
(slice_w_dispatch) = 1017`. `rung62` (58) and `rung63` (30) are unchanged, `slice_w_smoke` still
passes after its header-only correction, and `slice_w_probe` is ABSENT from the log -- it has no
assertions and lives in the temp tree, as step 3 left it.

**`git diff -- rust/src/` IS ONE COMMENT BLOCK AND NOTHING ELSE** -- nine lines at
`bleed_transient.rs`'s `mean`, and no executable line in the crate. **NO PYTHON SOURCE CHANGED**
by either step, so `pytest` is untouched and is not re-run.


### 5.22 SLICE X (rung 64, `BleedLimiter` + `LimitedBleedTransient`) — PRE-REGISTERED, seven probes MEASURED first

`M:\claud_projects\temp\rust-phase7\probe_x1.py` … `probe_x7b.py`, PyPy — nine files, because
**two of them are repairs of an earlier probe rather than new questions**. Every table below is
**EMITTED by one of them**, § 5.19 (xi)'s rule, and this section's leading finding is what that
rule was written for.

#### (i) THE LEADING FINDING — **THE FIRST CENSUS SAID ALL EIGHT SWAPPED CELLS WERE UNGATED AND ALL EIGHT INJECTIONS INERT. BOTH HALVES WERE THE INSTRUMENT; THE TRUE COUNT IS TWO**

Slice X **SWAPS 8** inherited names and **CREATES 1**. The question that sizes its gate debt is:
*if the port forgets a swap, does anything go red?* `probe_x4` asked it by DELETING rung 64's
override, so Python's own MRO supplies the parent body — which is exactly the defect "the Rust
table spread `..R62` and forgot to name this cell". It reported **8 of 8 UNGATED, and 8 of 8
INERT**. Three defects, one behind the other:

| # | defect | what it made every row report | repair |
|---|---|---|---|
| 1 | the two harness files were written `open(…).write(…)` | **INERT** — the subprocesses read EMPTY files | `with open(…)`; PyPy does not flush on refcount — [[windows-tooling-file-hazards]] |
| 2 | the suite ran under **xdist**, whose workers re-import `turbojet.engine` in fresh processes | **GREEN** — no worker ever saw the monkeypatch | `-n0` |
| 3 | the grid was `test_rung64.py` **ALONE** | over-counts holes | rungs **62 + 63 + 64**, one run per cell with `-x` |

**Defect 3 is the one that would have survived into the write-up**, and rung 64's own module
docstring names it: *"rung 60's tautology … pins `min phi == phi_lim` to 1e-15, **which rung 63's
`floor_dichotomy` already published**. Every gate below is therefore about something the tautology
does NOT own."* The suite declines, ON PURPOSE, to gate the property most of these injections
break. "Ungated by 64, caught by 63" would not be a hole — it would be a fact about where the
defence lives. Emitted on the repaired grid (**111 gates: 58 + 30 + 23**):

| injection (`cell` NOT swapped) | pytest 62+63+64, `-x` | first failing suite | DID IT MOVE? | verdict |
|---|---|---|---|---|
| — (baseline) | **111 passed** | — | — | — |
| `b_of` | 1 failed | rung 64 | YES | caught |
| `_armed_bleed` | 1 failed | rung 64 | YES | caught |
| **`_close`** | **111 passed** | — | **YES** | **UNGATED BY ALL THREE** |
| `_close_fuel` | 1 failed | rung 64 | YES | caught |
| `at_lever` | 1 failed | rung 64 | YES | caught |
| `at_stator` | 1 failed | rung 64 | YES | caught |
| `_isolating` | 1 failed | rung 64 | YES | caught |
| **`b_at_point`** | **111 passed** | — | **YES** | **UNGATED BY ALL THREE** |

**AND THE THIRD REPAIR CHANGED THE ANSWER BY NOTHING, WHICH IS ITSELF THE RESULT.** Widening from
one suite to three moved no row: rungs 62 and 63 add **zero** defence for any of rung 64's own
swaps. That is worth having measured rather than assumed — it is the difference between "I checked
the right grid" and "I checked the grid I had".

**`_isolating` also needed its WITNESS repaired**, one level below the census: probe_x4 passed an
EMPTY neighbour, on which rung 62's `want` and rung 64's both evaluate `False`, so the two bodies
agree by construction and the row read INERT while the suite was catching it. The separating input
is a neighbour **carrying** the floor. *Inert on my input* is not *inert* — and this is the second
slice running where a did-it-move column was wrong about a cell the gates could see.

#### (ii) **THE TWO HOLES, MEASURED — AND ONE OF THEM ZEROES THE RUNG'S OWN PUBLISHED HEADLINE WHILE ITS GATE STAYS GREEN**

A hole is a finding only once the number that SHOULD have caught it is printed beside the
assertion that did not — [[rust-port-slice-s-step3]], *a zero measured before being called a hole*.

**`b_at_point` — RE-SOLVE vs RECONSTRUCT.** Its docstring says the re-solve *"is what makes the
bleed integral below a measurement and not an estimate"*. The injection is that sentence's own
refused alternative. `probe_x7`, `matched_bill` at the suite's `ds = 0.005`:

| key | clean | injected |
|---|---|---|
| `b_int` constant | 0.149876470866 | 0.149876470866 |
| `b_int` schedule | 0.0737414799619 | 0.0737414799619 |
| **`b_int` floor** | **0.0382488802883** | **0** |
| **`b_peak` floor** | **0.092024528855** | **0** |
| **`b_ratio_const`** | **0.255202701714** | **0** |
| **`b_ratio_sched`** | **0.518688807277** | **0** |
| gate 5's `assert f < s < c` | **True** | **True** |

**The two ratios are the rung's HEADLINE** — `test_rung64.py`'s own module docstring says *"the
closed loop pays 52 % of rung 62's schedule's bleed and 26 % of the state-blind law's"* — and
**no gate reads either number.** The only assertion that touches them is an ORDERING, and
`f < s < c` is satisfied by driving the SMALLEST term to zero. Slice S step 3 found *a
NON-STRICT ordering assertion satisfied by inertness*; this is one notch over — **a STRICT
ordering assertion satisfied by ZEROING its smallest term**, which is strictly harder to notice
because the inequality visibly holds.

**`_close` — the Tt4-IMPOSED closure.** It is live (48 `_solve_b` calls against `_close_fuel`'s
344 on one floored march) but a reader sees it only through the march's INITIAL CONDITION: the
ramp itself runs on `_close_fuel`, and at the settle point the floor is DORMANT
(`equilibrium(1200)` and `fuel_for_Tt4(1200)` are bit-identical either way, because
`phi_lp = 0.8598` clears the 0.80 floor there). `probe_x7b`:

| key | clean | injected | move |
|---|---|---|---|
| **`nu0_lp`** | 0.7475441088051796 | **0.7557409602636336** | **+1.1 %** |
| `nu0_hp` | 0.7901540859632924 | 0.7897924703252703 | −4.6e-2 % |
| `b_int` | 0.038246852253226965 | 0.0386686995628587 | +1.1 % |
| `nu_lp_end` | 0.9410178724957114 | 0.9412256041458492 | +2.2e-2 % |
| `thrust_end` | 606.3605896041086 | 606.3261130576382 | −5.7e-3 % |
| `min_phi_lp` | 0.7999999999999995 | 0.7999999999999992 | — (rung 60's tautology, either way) |

**A 1.1 % wrong initial condition, 111 gates green.** § 5.19 (ii) records the mis-spelled pin as
*0.018 % off with a clean build and no error*; this is the same shape two orders larger, and the
reason it hides is that every rung-64 reader **differences two cells that are equally wrong** —
`min_phi_lp` is pinned by the tautology and the bills are relative. Both holes are booked to step
3 as gates the port ADDS, not as gates it ports.

#### (iii) **THE SCOPE, ENUMERATED**

**Source** (`ast` spans): `BleedLimiter` **67** lines (9863–9929) + `LimitedBleedTransient`
**438** lines (9932–10369) = **505**.

**Cells.** X **CREATES 1** — `b_at_point`, whose two ladder call sites are rung 64's own
`_bill_cell` and rung 65's override — and **SWAPS 8**: `b_of`, `_armed_bleed`, `at_lever`,
`at_stator`, `_isolating` (rung 62's table), `try_close` / `try_close_fuel` (phase 6's), and
`__init__`, which is not a vtable cell. It defines **7 PLAIN** methods no later rung overrides,
so none needs a cell: `_solve_b`, `_closer`, `_bill_cell`, `authority_ceiling`,
`_match_open_loop`, `matched_bill`, `floor_refusal`. § 5.19 (x) said **1** and probe_w4's emitted
column says **1** — **slice X is the first slice in phase 7 whose cell count the plan got right**,
and § 5.21 (i)'s re-run over the whole ladder is why that can be stated rather than hoped.

**Tests**: `test_rung64.py`, **18 `def test_` → 23 collected** (1.3× parametrize expansion,
against slice W's 2.1×), **9 carrying `slow`**, 408 lines. The ported gate count is **23**.

**Module**: `rust/src/limited_bleed.rs`, on `bleed_transient.rs`'s shape.

#### (iv) **THE RUNG-62 PIN — SLICE X SHIPS THE MECHANISM AND CANNOT EXERCISE IT**

§ 5.19 (ii) measured 16 two-argument `super(LimitedBleedTransient, self)` sites. Re-emitted one
row per site by `probe_x1d.py` — **the first form of probe_x1 re-walked the enclosing class ONCE
PER PIN SITE and printed `{_close_fuel: 29, _close: 3}` against 16 real sites, a fourth instrument
defect in this section and the fourth of the same kind: a count assembled by a loop nobody
checked**:

| pinned ancestor | sites | methods reached | rungs that write it |
|---|---|---|---|
| `LimitedBleedTransient` | **16** | `_close_fuel` ×15, `_close` ×1 | 65, 66, 67, 68, 69, 70, 72, 74, 75 |

**It is the ONLY two-argument `super` on the whole ladder**, measured rather than assumed — and
**every one of its 16 sites is in a rung slice X does not port.** So the mechanism ships LIVE and
unexercised by any value key, which is `R40`/`R43`'s situation and takes `R40`/`R43`'s answer:
manufacture the failure. In Rust the pin is simply `r62_try_close_fuel(leaf, …)` — an `fn`, not a
method — so slice X makes rung 62's two closure bodies `pub` and carries § 5.19 (ii)'s warning
**at the call site**: `r62_try_close_fuel(&R62, …)` compiles, runs, and silently freezes the
ladder at rung 62.

#### (v) **`_b_forced`: THE SET→READ CHAIN IS ONE FRAME, AND THAT FRAME IS A SHIPPED GATED CELL**

Rung 64 is the first rung to both SET and READ one of § 5.19 (iv)'s nine STATE-kind
dynamically-scoped fields. Measured (`probe_x2`) on one floored march at `ds = 0.02` —
**1 705 `b_of` calls, and every distinct frame chain from the read back to the set**:

| n | frames, read (`b_of`) → set (`closer`) |
|---|---|
| 1 415 | `b_of` ← `_close_fuel` ← `closer` |
| 290 | `b_of` ← `_close` ← `closer` |

**Two chains, depth exactly 1 in both.** The single intervening frame is rung 62's closure body —
in Rust `try_close_fuel` / `try_close`, **shipped cells with live tables and a dispatch gate**. So
§ 5.19 (iv)'s `Scope` parameter costs a NON-ADDITIVE change to already-gated phase-6 code: the
exact cost it named for slice V, which § 5.20 refuted there and re-booked to Y, and which arrives
HERE instead.

**Nothing leaks.** `_b_forced` and `_b_state` are `None` after every march, and **0 of 12 witness
keys move** on either a bare or a floored machine — including the four map object IDENTITIES that
[[rust-port-slice-v-step3]] put in the key set. Slice V's `Cell<ComponentMap>` is still the only
carrier; rung 64 adds none.

#### (vi) **THE REDUCE IS TWO DISPATCHES DEEP, THE INNER ONE HAS THREE OUTCOMES, AND TWO BRANCHES ARE DEAD**

Emitted (`probe_x3`), one `_stator_march` per machine at `ds = 0.02`:

| machine | `b_of` calls | via `_b_forced` | via `_b_state` | via super (r62) | `b == 0` | `_solve_b` | dormant | riding | saturated | closer calls min/med/max |
|---|---|---|---|---|---|---|---|---|---|---|
| unarmed (the rung-63 reduce) | 392 | 0 | 0 | **392** | 392 | 0 | — | — | — | — |
| constant `b` (rung 42's leg) | 392 | 0 | 0 | **392** | 0 | 0 | — | — | — | — |
| schedule (rung 62's leg) | 392 | 0 | 0 | **392** | 9 | 0 | — | — | — | — |
| **FLOOR, reachable (0.80)** | **1 705** | **1 705** | 0 | **0** | 392 | **392** | 257 | 135 | **0** | 1/1/14 |
| **FLOOR, dormant (0.60)** | 392 | 392 | 0 | **0** | 392 | 392 | **392** | 0 | 0 | 1/1/1 |
| **FLOOR, over-set (0.90)** | 1 228 | 1 228 | 0 | **0** | 392 | 392 | 167 | 74 | **151** | 1/2/11 |

**THREE ROWS OF THAT TABLE ARE HAZARDS, NOT DECORATION.**

1. **`_b_state` is 0 EVERYWHERE.** The second override in `b_of` is a **dead branch at rung 64** —
   it is rung 65's, declared at 64. A port that omits it passes every slice-X gate and breaks at
   slice Y. Noted **at the definition with the count** (slice O's rule), not only here.
2. **On an ARMED machine the `super` column is EXACTLY 0.** Rung 64's `b_of` fall-through is
   unreachable whenever a floor is fitted, so the only machines exercising it are those where
   rung 64's cell and rung 62's are behaviourally identical — [[rust-port-slice-u-step3]]'s
   shape, *a function exercised only on cells chosen for INERTNESS*.
3. **The reachable floor NEVER saturates** (0 of 392). All three regimes fire in one march only
   on `authority_ceiling`'s deliberately over-set floor, at **167 / 74 / 151**. A dispatch gate
   run on the rung's headline machine alone tests two branches of three.

Split by cell on the reachable floor: `_close` **26 dormant / 22 riding**, `_close_fuel`
**231 / 113**. The outer reduce holds — an unarmed rung-64 machine's 86-point march is
**bit-identical** to the rung-63 class's on the same hardware. **No trial evaluation raised on any
of the six machines**, so a Rust straight-line restore and Python's `finally` are
indistinguishable on the suites' grid — a statement about the grid, not a licence.

#### (vii) **`_b_forced` NEVER NESTS — SO THE `Scope` DECISION IS RETIRED FOR IT, AND ONLY FOR IT**

The one thing that could sink a carrier: rung 64's `_closer` restores to **`None`**, not to the
previous value, so a guard entered while the field is already set would CLOBBER — and the port
would then owe that clobber bit-for-bit rather than a quiet repair. `probe_x6`'s runtime form
(data descriptors over all nine fields, rungs 64–84 under `-n0`) would not have finished inside
the session, and **a probe nobody waits for is not a measurement**; it was stopped and replaced by
`probe_x6b`, which asks the question more directly.

| | |
|---|---|
| guards restoring **to `None`** | **68** |
| guards restoring **to a SAVED value** | **4** — all four in `_stator_march`, at rungs **65, 66, 67, 68**, over `_b0`, `_lag`, `_tau_gov`, `_ic_order`/`_v0` |

**Python itself anticipates nesting in exactly four places, and `_b_forced` is not one of them.**

| field | guards | writers | a writer reachable from inside a guard on the SAME field? |
|---|---|---|---|
| **`_b_forced`** | 1 | 2 | **NO** |
| **`_v_forced`** | 1 | 2 | **NO** |
| `_b_state` | 31 | 32 | **YES** — rungs 77–79 readers (`set_point_gains`, `gauge_scan`, `coord_census`, `root_census`, …) → `_integrate_fuel_*` |
| `_v_state` | 24 | 25 | **YES**, same shape |

**THE DECISION, AND ITS SCOPE.** `_b_forced` gets a `Cell<Option<f64>>` on
`TwoSpoolTransientCore` plus an **RAII guard whose `Drop` restores**. No signature is re-opened —
`try_close` / `try_close_fuel` keep the shape their dispatch gate already pins — and the property
§ 5.19 (iv) chose `Scope` for is recovered in full, because **`Drop` runs on unwind**: it is at
least as strong as Python's `finally`, where a straight-line restore is strictly weaker. Slice V's
`Cell<ComponentMap>` is the precedent, at the same level of the tree.

§ 5.19 (iv) is therefore **retired for `_b_forced`, and NOT for all nine.** The
`_b_state`/`_v_state` same-field nest is a live candidate at **slice AH**; it is recorded here
rather than solved, and **the reachability graph that found it is name-based across classes — an
UPPER bound**, so it says *candidate*, never *confirmed*. Whoever ports AH owes it the runtime
form this slice could not afford.

#### (viii) **WHAT THE PORT ADDS**

| | |
|---|---|
| **CELL CREATED** | `b_at_point`, into [`LeverHooks`] |
| **CELLS SWAPPED** | `b_of`, `armed_bleed`, `at_lever`, `at_stator`, `isolating` (rung 62's table) + `try_close`, `try_close_fuel` (phase 6's) |
| **STATE** | `lim: Option<BleedLimiter>` into `LeverArming`; `b_forced` (+ `b_state`, rung 65's, DEAD here) as `Cell<Option<f64>>` on `TwoSpoolTransientCore` |
| **ARM** | `LeverArm::bleed_lim` — **slice W's P5 at its first test**: 6 → 7 keywords, a field with a `Default`, signature untouched |
| **PIN** | `r62_try_close` / `r62_try_close_fuel` made `pub`, with § 5.19 (ii)'s warning at the call site |
| **CONSTRUCTOR** | `build_limited_bleed`, on `build_scheduled_bleed`'s shape, with rung 64's THREE-way arming assert extending rung 62's two-way |

#### (ix) PRE-REGISTERED — written before a line of Rust

- **P1.** The 23 ported gates pass with **zero tolerance tiers** on the first oracle run, as
  slices T/U/V/W did. If any needs a tolerance, § (v)'s "no new carrier" verdict is wrong.
- **P2 — the reduce, by DISPATCH and per call, at THREE levels.** `bleed_lim = None` is rung 63
  bit-for-bit; a DORMANT floor reaches rung 62's body at every state (392/392 at `ds = 0.02`) and
  through it rung 57's; a REACHABLE floor takes both `_solve_b` branches inside one march at
  **257 dormant / 135 riding**. No value key sees the second or the third.
- **P3 — the SATURATED branch is reached by ONE reader.** `authority_ceiling`'s over-set floor is
  the only machine in the suite that saturates (**167 / 74 / 151**); the reachable floor is
  **0 / 392**. Gated at all three, not at the two the headline machine reaches.
- **P4 — the two DEAD branches.** `b_of`'s fall-through is unreachable on every armed machine
  (super = 0 of 1 705) and `_b_state`'s branch is dead at rung 64 entirely (0 of 1 705). Two ports
  that drop them pass every slice-X gate. Manufactured-bug gates at step 5, not value keys.
- **P5 — the two HOLES get gates the port ADDS.** `b_at_point` re-solves rather than reconstructs
  (witness: `b_ratio_const` **0.2552**, `b_ratio_sched` **0.5187** — the rung's published headline,
  currently read by nothing), and rung 64's `_close` is live in the march's INITIAL CONDITION
  (witness: `nu0_lp` **0.7475441088051796**, which the injection moves 1.1 %). Both are ABSOLUTE
  keys, because the existing gates are relative and that is exactly why they miss.
- **P6 — slice W's P5 holds at its first test.** `at_lever` goes 6 → 7 keywords and its cell
  signature `fn(&ScheduledStatorCore, &LeverArm) -> ScheduledStatorCore` is **not re-opened**.
  Reported explicitly at ship, held or not.
- **P7 — the rung-62 PIN ships live and unexercised**, and its gate is manufactured: call
  `r62_try_close_fuel` once with the LEAF table and once with `&R62`, assert the values differ.
  Falsified if they agree, which would mean the leaf table is not reached.
- **P8 — the `Cell` + `Drop` carrier costs no gated signature.** Falsified if any `try_close` /
  `try_close_fuel` signature has to change.
- **P9 — the ORACLE'S GRID IS NOT THE SUITE'S, and the header says so AT THE TOP.**
  `_match_open_loop` is a root over whole MARCHES and `matched_bill` runs two of them plus four
  `_bill_cell`s at `ds = 0.005`. If `dump_slice_x.py` coarsens, it is disclosed in the header and
  in the step-4 write-up — slice S step 4's lesson (*a probe's HEADER claimed the suites' grids and
  its code ran another*).
- **P10.** Steps, on slice W's shape: **1** the cell + `LeverArm`/`LeverArming` fields + the
  carrier + the `pub` pin · **2** the port + `slice_x_smoke.rs` · **3** the rung-64 suite + the
  injections + the two hole gates (P5) · **4** `slice_x_oracle.rs` + `dump_slice_x.py` · **5** the
  dispatch gates (P2/P3) and the manufactured bugs (P4/P7).

#### (x) THE STEP LOG

##### STEP 1 — SHIPPED. **THE DECOMPOSITION WAS STATED BEFORE THE LOG WAS OPENED, AND IT HELD**

Written before reading `cargo test --release`: *107 result lines, 1017 passed, 0 failed, 0
ignored — step 1 is purely additive (two defaulted `Option` fields, two `Cell`s nothing reads,
one cell slot nothing calls, two `fn`s made `pub`).* Measured: **107 / 1017 / 0 / 0.** Exact.

Shipped: `limited_bleed.rs` (`BleedLimiter` + its four constructors + `Regime`), `LeverArming.lim`,
`LeverArm.bleed_lim` + `merged` + `keys` + `arms_valve` + `LeverArm::floored`, the `b_at_point`
cell with `R62` pointing at the **panic** (§ (ii)'s mistake refused from the other side), the
`Cell<Option<f64>>` carriers + `ForcedBleed` RAII guard, and the two `pub fn r62_try_close*`.

**THE ONE THING THAT WENT RED, AND IT WAS RIGHT TO.** `LeverArming` gaining `lim` broke a struct
literal in `tests/slice_w_dispatch.rs`. That is a *non-exhaustive-literal* failure, i.e. the
compiler enumerating the arming modes for us — the same guarantee § 5.21 (iii) chose the field
over a parameter to get. **P8 HOLDS at step 1**: no `try_close` / `try_close_fuel` signature moved.

**A PORT DECISION WRITTEN AT THE FAILURE SITE.** `ForcedBleed::set` panics if `b_forced` is
already `Some`. **Python does not raise there — it CLOBBERS**, silently. So the panic string says
so in as many words, names the two paths that could reach it (rung 65's closures, the 16 pin
sites at rungs 66–75), and opens with *THIS PANIC IS A PORT DECISION, NOT A BUG YOU JUST
INTRODUCED*. A future porter reads the reasoning where they hit it, not in a doc they would have
to know exists.

**THE CHECK THAT CLEARED BEFORE STEP 2.** `authority_ceiling`'s `"shut"` / `"schedule"` / `"full"`
legs all go through `self.at_lever(…)`, which constructs a **`LimitedBleedTransient`** — so every
leg carries `R64`, never `R62`, and the new panic is unreachable from there. `b_at_point`'s own
`bleed_lim is None` branch covers all three. Had a bare sibling carried an `R62` table, gates 4
and 5 would have died on line one of step 2.

##### STEP 2 — SHIPPED. **318 OF 318 BIT-EXACT ON THE FIRST RUN, AND THE ONE KEY THAT DISAGREED WAS A RENAMED PREDICATE**

`limited_bleed.rs` is 919 lines: the six cell bodies, `r64_solve_b`, the four `R64*` tables,
`build_limited_bleed`, and the five reading instruments (`bill_cell`, `authority_ceiling`,
`match_open_loop`, `matched_bill`, `floor_refusal`). `slice_x_smoke.rs` + `dump_slice_x_smoke.py`
carry **318 keys in nine sections**, every float as its IEEE-754 bits.

**FINDING 1 — THE ADVISOR'S BLOCKER: STEP 1 HAD ALREADY TURNED RUNG 64's `isolating` OVERRIDE
INTO A NO-OP, AND NO GATE COULD SEE IT.** Step 1 extended the SHARED `LeverArm::arms_valve()` to
include the floor. But Python's rung-63 `want` is **two-way** (line 9440) and rung 64's is
**three-way** — and the assert's OTHER side, `ref._armed_bleed()`, is DISPATCHED and gains the
term at rung 64 by itself. So the two-way/three-way difference is the **entire content** of the
override, and extending the shared helper made `r64_isolating` textually identical to
`r62_isolating`. Two consequences, both invisible to step 1's 1017-green gate because no shipped
test hands a floored NEIGHBOUR to a rung-62 machine: § (viii)'s swapped-cell list loses a row, and
step 3's `_isolating` injection would report inert *for a reason that has nothing to do with
Python*. Repaired by splitting `arms_valve` (rung 62's, two-way) from `arms_valve_floored` (rung
64's), each carrying the other's name in its doc. **Smoke section H is the witness**: it isolates
a stator against a floored neighbour — the one case rung 63's body cannot express.

**FINDING 2 — THE COARSE SMOKE GRID FLIPS ONE OF THE RUNG'S OWN PUBLISHED CLAIMS, SO SECTION G
GATES THE FLIP.** `floor_refusal`'s `inert` — claim (i), that the composite IS the valve-alone
march — is **True at `ds` 0.005 and 0.01 and False at 0.02**. Measured: the coarse march moves the
parabola-refined `m_i` by **2.894e-04**, four orders above the reader's own 1e-14 bar, while
`min_phi` still agrees to **1.110e-16**. So the flip is the REFINED MINIMUM moving on a coarser
grid, not the physics. Left at 0.02 the file would have shipped `G/inert = 0` as a bit-exact
golden reading as a refutation of the rung. Section G runs at `ds = 0.01` and **emits the 0.02
reading beside it** (`G/coarse/*`), so the flip is gated rather than avoided — slice S step 4's
lesson (*a probe's HEADER claimed the suites' grids and its code ran another*) answered by
emitting BOTH grids instead of picking one.

**FINDING 3 — THE ONE KEY THAT DISAGREED WAS A PREDICATE RENAMED IN THE PORT.**
`H/floored_neighbour/armed_is_armed_stator`: rust 1, py 0. Python's `_is_armed()` is *schedules
only*; the port's `is_armed()` is the COMPOSITE guard `_is_armed() or vsv_lp or vsv_hp` that rungs
58–60 open with, and the true equivalent is `is_scheduled()`. **The two agree on every machine
with no constant stator** — the `ref` key three lines above is one, and it passed — so the only
key in 318 that separates them is the armed leg, which carries `vsv_lp = 0.20`. Slice W step 4's
lesson arriving again, and this time the discriminating key was in the file.

**WHAT PROBE 8 SETTLED, AND WHAT IT DID NOT HAVE TO.** The advisor flagged that `_solve_b`'s
residual cannot propagate through a plain `illinois`, so the port would need a stash (with an
iteration-count divergence to disclose) or an `expect`. Measured first: **0 aborts in 156 373
closure calls** over `tests/test_rung64.py`. And the crate ALREADY has `try_illinois`, whose
residual returns `Result` — so the port propagates exactly as Python's raise does, no stash, no
`expect`, no divergence, and the zero is carried as a gated counter rather than assumed away.
Probe 8 also measured the regime split over that suite: **23 438 dormant / 13 519 riding / 1 002
saturated** in 37 959 solves — all three regimes reached, which is P3's premise confirmed on the
suite rather than on one march.

**PROBE 9 — P9's GRID, MEASURED.** One floored `_bill_cell`: **478 outer solves / 2 068 closure
evaluations at `ds = 0.02`, 1 753 / 7 385 at 0.005**. The three top-level readers at `ds = 0.02`:
`authority_ceiling` 0.13 s, `matched_bill` 0.50 s, `floor_refusal` 0.76 s on PyPy. The advisor's
warning that `_match_open_loop`'s multiplier would surprise is **half right**: it costs whole
MARCHES per iteration but **no outer solves at all** (both matched laws are open-loop), which is
why `matched_bill` shows the same 478 as a single floored cell.

**THE SMOKE ALREADY WITNESSES HOLE 1.** § (ii)'s `_close` hole has the absolute anchor
`nu0_lp = 0.7475441088051796`, and `C/floored/nu0_lp` **is that value**, bit-for-bit, with the
unfloored `D/shut/nu0_lp = 0.7557409602636336` beside it — the 1.1 % the hole leaves wrong is now
a golden PAIR. The march's initial condition comes through the equilibrium solve, which uses
`try_close` and not `try_close_fuel`, so 318/318 green means `r64_try_close` is exercised AND
bit-correct, and step 3's hole gate has its anchor already measured.

**FOUR THINGS THE ADVISOR CAUGHT IN THE SHIPPED SOURCE.** (a) `R64`'s doc said *six of six cells
swapped* two lines above *`legs` — NOT overridden*; it is **five of six**. (b) The module note's
*0 of 1 705* fall-through zero is a **WITHIN-A-MARCH** claim — smoke section B calls `b_of`
directly four times and every one takes that branch, so a step-5 gate written from the unscoped
sentence would fail on a binary that merely read the valve; `b_of_state`'s zero has no such
caveat, which makes it the stronger of the two and they are now asserted separately. (c)
`_LAG_OK` is a Python CLASS ATTRIBUTE **rung 65 flips**, and `build_limited_bleed` hard-codes it
false — so rung 65 cannot delegate here to inherit the assert chain. That is now written **inside
the assert string**, `ForcedBleed::set`'s precedent. (d) `Census64` is thread-local with no
per-test reset, so step 5 must reset in every `#[test]` that reads it.

##### STEP 3 — SHIPPED. **THE TWO ADDED GATES ARE THE ONLY THINGS THAT CATCH THE TWO HOLES, AND THE CENSUS SAYS SO BY NAME**

`rung64.rs` carries **20 tests** — Python's 18 `def test_` (23 collected; the two parametrised
pairs and the four-way become loops) plus the **two § (ii) named**. All 20 green on the first run,
which is exactly when the instrument has to be measured, so the census was re-run in Rust.

**THE CENSUS — 8 injections, 6 targets, 0 holes, and every row MOVED.** One injection per
swapped/created cell: point rung 64's table slot at rung 62's body (for `b_at_point`, which has no
rung-62 body, at a RECONSTRUCTION — § (ii)'s own mistake). Every one is caught, and the
did-it-move column is never inferred.

**AND THE ANSWER SPLITS BY GRID, WHICH IS THE POINT.** Against **everything** (the ported gates,
the two added, and `slice_x_smoke`'s 318 bit-exact values): **0 holes**. Against the **PORTED
gates alone** — the honest analogue of Python's 111 — **both holes are still holes**:

| injection | which `rung64` tests redden | ported-only verdict |
|---|---|---|
| `b_at_point` | `the_bleed_integral_is_a_measurement_and_not_an_estimate` — **that one and no other** | **still a HOLE** |
| `try_close` | `the_equilibrium_start_is_solved_on_the_floored_plant` + `the_bleed_integral_…` — **both ADDED** | **still a HOLE** |

So § (ii)'s Python measurement reproduces exactly in Rust, and the two added gates are not
belt-and-braces: **they are the entire detector.** The other six injections redden 1–11 ported
gates each (`b_of` 11, `at_lever` 7, `try_close_fuel` 7, `armed_bleed` 2, `isolating` 1,
`at_stator` 1).

**FINDING 1 — THE PROBE CALLED FOUR ROWS "BUILD FAIL" AND THEY HAD ALL RUN.** The first census
classified any non-zero exit from the witness as a build failure and then **skipped running the
gates for that row entirely**. Four injections (`armed_bleed`, `at_lever`, `isolating`,
`at_stator`) exit 101 — a **runtime PANIC**, on rung 64's OWN assert: *"reference sibling must
carry the NEIGHBOUR's valve and nothing else; it reports armed=false against neighbour=true."*
That is the strongest detector in the whole set, and the instrument was reporting it as its own
inability to compile. Rebuilt to separate the two and to run the gates unconditionally. **A probe
that cannot tell a crash from a compile error will call its best result a failure of itself.**

**FINDING 2 — THE MODULE'S `b_of` CAVEAT IS CONFIRMED BY ARITHMETIC, NOT BY ASSERTION.** The
witness census reads `b_of_super = 478` on a run containing a floored march — which reads like a
contradiction of *0 within a march*. It is not: `close_unfloored 48 + close_fuel_unfloored 344 +
b_at_point_unfloored 86 = 478`, **exactly**, so every one is on an UNFLOORED path and none is
inside a floored solve. `b_of_forced = 2 068` is the closure-evaluation count probe 9 measured
independently. That identity is a step-5 gate: `b_of_super == close_unfloored +
close_fuel_unfloored + b_at_point_unfloored`, which no float can express.

**FINDING 3 — A COMMENT PROMISED AN ENFORCEMENT THAT DID NOT EXIST.** `the_invisible_authority_…`
freezes a 15-key sweep where Python's `isinstance(v, float)` filter auto-covers whatever a later
rung adds — and its comment claimed *"the list is checked against the struct's own count below"*
with **no such check anywhere**. [[rust-port-documented-gate-that-doesnt-exist]], and the second
false claim in shipped slice-X source after `R64`'s *six of six*. Repaired by making the compiler
enforce it: an **exhaustive destructure** of `BillCell` beside the list, so rung 65 adding a field
is a COMPILE ERROR until the field is classified as swept or excluded.

**THE REST OF THE CENSUS WITNESS, read for free.** `b_of_state = 0` and `solve_b_aborts = 0` —
both dead arms confirmed on a live grid rather than argued. `saturated = 0` beside `dormant = 314
/ riding = 164`: the headline machine reaches **two** regimes, and only `authority_ceiling`'s
deliberately over-set floor reaches the third — P3's premise, measured on the machine rather than
assumed from the enum.

##### STEP 4 — SHIPPED. **BOTH INTERPRETERS ARE BIT-EXACT, SO THERE IS NO BAR TO TYPE**

`slice_x_oracle.rs` + `dump_slice_x.py`: **1 906 keys**, the SUITE's own grid (`ds = 0.005` plus
the 0.01 / 0.0025 the suite itself refines to) and **both** map shapes, in seven sections — the
four laws' cells, `b_at_point` walked at EVERY point of a floored march, `authority_ceiling`,
`matched_bill`, `floor_refusal`, a seven-point set-point sweep reaching all three regimes by
value, and a four-point authority sweep.

**THE FINDING: the CPython arm needs NO TOLERANCE.** Diffing the two goldens directly — **0 of
1 744 float keys drifted and 0 of 162 discrete keys flipped**, PyPy 3.11.15 against CPython
3.14.3. Slice W needed exactly one exemption, for Python's built-in `sum()`, whose accumulation
order differs between interpreters; **rung 64's 441 lines of readers contain no `sum()` at all**
(grepped, not assumed) — every accumulation is an explicit `+=` trapezoid and every extremum a
`max`/`min`. So the arm asserts EXACT agreement rather than carrying a bar that suppresses
nothing. [[rust-port-guessed-census-bars]] answered by measuring and finding there was nothing to
type.

**P9 IS DISCHARGED IN THE HEADER, INCLUDING WHAT IS COARSER.** Sections B/F/G run at `ds = 0.01`
and the header says so and why — B walks every point and re-solves at each, which at 0.005 is
~700 outer solves for a reading whose content is the SHAPE of `b(s)`. Probe 9's numbers are what
the choice was made from.

**TWO INSTRUMENT DEFECTS, BOTH CAUGHT BY THE INSTRUMENTS THEMSELVES.** (i) The Rust asked for 16
`npts` keys sections C/D did not emit — the exhaustive `BillCell` destructure doing exactly its
job, and the dump gained them rather than the Rust losing them. (ii) Regenerating with `> file
2>&1` **interleaved the dump's own stderr key-count INTO the middle of a data line**, and the
loader reported only `InvalidDigit`. [[windows-tooling-file-hazards]] a third time this slice; the
loader now names the offending line and says what to do about it.

**AND A THIRD DEFECT IN THE WRITE-UP, CAUGHT ONLY BY ADDING THE TALLY UP.** As first logged, this
entry said 1 906 keys in one paragraph and `1 744 + 146` in the next — short by exactly the 16
`npts` keys defect (i) had just ADDED. The drift comparison had been run on the pre-fix 1 890-key
goldens and the numbers carried forward unchanged. **A count taken before a fix is not a count of
what shipped**, and nothing in the gate can see it: both arms assert EXACT equality, so the
interpreters really do agree on all 1 906 — it is the RECORD that was stale, in the document slice
Y reads as what was measured. Re-measured against the shipped goldens by tagging every key with
the emitter that produced it (`f` / `d` / `b`), not by classifying the file, which cannot tell
them apart — floats are stored as IEEE-754 bit patterns and look exactly like the integers.
**1 744 float + 162 discrete (100 `d` + 62 `b`) = 1 906**, 0 drifted, 0 flipped. The float half
was right all along; the 16 additions were all integer-emitted, so the discrete half took the
whole error. The commit message carries the stale pair and cannot be fixed; this is where it is
set straight.

##### STEP 5 — SHIPPED. **NINE GATES NO FLOAT CAN CARRY, AND SIX MUTATIONS PROVE THEY FAIL**

`slice_x_dispatch.rs`: the reduce-by-dispatch (P2), all three regimes (P3), the two dead branches
(P4), `R62`'s panicking slot, and **the rung-62 PIN (P7)**.

**P7 HOLDS.** The same rung-62 body, same hardware, same arguments, reached once through rung
64's table with a trial position forced and once through rung 62's own: the leaf's dispatched
`b_of` returns the trial and rung 62's returns the stored constant, so the two `phi_lp` values
DIFFER. Falsified if they agreed — which would have meant the leaf table is not reached and every
`super(LimitedBleedTransient, self)` site at rungs 65–75 would freeze silently.

**FINDING — THE `b_state` GATE WAS VACUOUS AS FIRST WRITTEN, AND ITS OWN MUTATION SAID SO.**
Asserting `b_of_state == 0` on every march is satisfied by a port that DELETED the branch: the
count stays zero either way, so the assertion could not see the defect it names.
[[rust-port-slice-u-step4]]'s *a gate comparing a key with ITSELF cannot see its value*, one shape
over. Rewritten to MANUFACTURE the branch reachable — set the carrier, assert `b_of` returns it,
assert the forced trial WINS over it (Python's own precedence), assert the guard's drop exposes
the state again rather than erasing it. The zero-count assertion is kept beside it, where it now
means *and no shipped path reaches it*.

**AND ONE GATE IS A WATCHDOG, NOT A GATE, WHICH IS NOW SAID RATHER THAN BLURRED.**
`the_abort_arms_are_carried_and_never_taken` has the same vacuity and **cannot** be repaired the
same way: an `Abort` cannot be manufactured from a test without a hook in production code, which
would be worse than the gap. So the doc states what it catches (the arms starting to fire) and
what it does not (their removal — carried instead by `r64_solve_b`'s `Result<_, Abort>` signature,
which the compiler enforces at every call site, and by the oracle's 1 906 keys).

**THE MUTATION TABLE — 6 mutations, 0 survivors**, each the realistic mistake its gate names:

| mutation | reddens |
|---|---|
| `b_of`: STATE checked before FORCED | `the_lagged_position_override_…` |
| `b_of`: the rung-65 STATE branch DELETED | `the_lagged_position_override_…` |
| `R64_FUEL.try_close_fuel` left as rung 62's | `every_fall_through_…`, `the_reduce_is_a_dispatch_…` |
| `R62`'s `b_at_point` slot defaulted to `b_of` | `a_rung62_machine_refuses_…` |
| `ForcedBleed::set` CLOBBERS silently, as Python does | `a_nested_trial_position_is_refused_…` |
| `ForcedBleed`'s `Drop` does not clear the carrier | **8 of 9** |

The last row is the one worth reading: a leaked trial position reddens **eight of the nine**
gates, which is the measurement behind choosing a destructor over a `finally` anyone can forget.
And the first two rows are the repaired `b_state` gate catching both the deletion AND the
precedence inversion — the vacuous version caught neither.


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

### 5.23 SLICE Y (rung 65, `LaggedBleedTransient`) — PRE-REGISTERED, six probes MEASURED first

`M:\claud_projects\temp\rust-phase7\probe_y1.py` … `probe_y6.py`, PyPy. Every table below is
**EMITTED** by one of them (§ 5.19 (xi)).

#### (i) THE LEADING FINDING — **THE TWO SIBLING CLOSURES ARE MIRROR-IMAGE ONE-ARMED DISPATCHES, AND EACH RUNG-65 OVERRIDE ONLY EVER TAKES THE ARM THAT EQUALS ITS PARENT'S**

Rung 65 overrides `_close` and `_close_fuel` with the SAME two-way test — *if the lag is armed
**and** the march state is live, dispatch past rung 64 to rung 63's closure; otherwise run rung
64's instantaneous root.* Read as source it is one idea written twice. Counted over rungs
62 + 63 + 64 + 65 in one process (`-n0`; xdist workers never see a monkeypatch — slice X defect 2):

| cell | `lagged=F` | **`lagged=T, state=F`** | **`lagged=T, state=T`** |
|---|---|---|---|
| `_close` | 864 | **2 064** | **0** |
| `_close_fuel` | 20 477 | **0** | **200 960** |

**Neither method's test is ever exercised both ways, and the two zeros are on OPPOSITE arms.**
The mechanism is one line of the march: `_b_state` is set only inside
`_integrate_fuel_valve_lag`'s `der`, which imposes FUEL, so every close under a live state is a
`_close_fuel`; and `_close` is reached only from the STEADY solves (`equilibrium`, and the
running line the march starts on), which never run inside a derivative evaluation.

Two consequences, and the second is the one that costs gates:

1. **Rung 65's `_close` override is a NO-OP on the entire shipped grid.** Its live branch *is*
   `super()._close`. Registered as a prediction before the injection was read, and the injected
   witness is **byte-identical to baseline in all 13 keys**.
2. **`_close_fuel`'s `and self._b_state is not None` is UNTESTED.** A port spelling it *"if
   lagged, dispatch to rung 63"* — dropping the state test — agrees on every shipped path. The
   rung's own docstring is what makes this worth writing down: it justifies that half of the test
   at length (*"Outside it — every STEADY solve — the lag is meaningless and rung 64's
   instantaneous root runs, which is what makes the initial running line identical to the machine
   this rung is compared against"*) and the mechanism it describes is real — it is just delivered
   entirely through `_close`, the sibling, and never once through the method whose docstring
   claims it.

**AND THE INSTRUMENT WAS MADE TO PROVE IT CAN SEE** ([[rust-port-slice-w-step3]] — *five of six
injections passed all 88 gates and the probe was wrong five times, every one a zero nobody
measured*). `probe_y4` re-counts the arm in isolation, confirms `fuel_for_Tt4` on a lagged machine
reaches `_close` (17 calls) and not `_close_fuel` (0), and then **exhibits a direct call that
takes the arm** — the counter reads 1. The zero is the grid's, not the counter's.

#### (ii) THE SCOPE, ENUMERATED

**Source** (`ast` spans): `LaggedBleedTransient` **479** lines (10373–10851), ONE class —
`BleedLimiter` was ported whole at slice X, `tau` included, so the rung's device needs nothing.
Against slice X's 505.

**Cells — probe 1's emitted census, and rung 65 CREATES NOTHING.** Twelve methods: **6 swaps**
(`_close`, `_close_fuel`, `_stator_march`, `at_lever`, `b_at_point`, `integrate_fuel`) and **6
plain** (`_lagged`, `_removed`, `_integrate_fuel_valve_lag`, `bandwidth_ceiling`, `marginal_mode`,
`fuel_authority`), none of which any later rung overrides. § 5.19 (x) predicted **0 new cells**
and the emitter agrees — **the second row of that column an emitter confirms**, after slice X's.

What Y does owe is the **THIRD and last code-resident ⚠ note** in `fuel_transient.rs`:
`integrate_fuel` becomes a cell, its first overrider being rung 65 (**10 overriders** in all).

**Tests**: `test_rung65.py`, **21 `def test_` → 21 collected** (no parametrize expansion at all,
against slice W's 2.1× and X's 1.3×), **9 carrying `slow` (42.9 %)**, 415 lines. Ported gate
count **21**.

**Module**: `rust/src/lagged_bleed.rs`, on `limited_bleed.rs`'s shape.

#### (iii) **OPEN THE `_stator_march` SIGNATURE ONCE, FOR FOUR RUNGS — AND § 5.19 (iv)'s `Scope` IS ALIVE AFTER ALL**

`Scope` has been retired field-by-field twice (`try_close` at slice V, `_b_forced` at slice X),
which made it look like a design dying by a thousand cuts. Probe 2 emits the signature of every
`_stator_march` in the ladder and the answer is the opposite:

| rung | args | ADDS | guard shape |
|---|---|---|---|
| 57 | 10 | *(base)* | — |
| **65** | 11 | `b0` | `prev, self._b0 = self._b0, b0` … `finally: self._b0 = prev` |
| 66 | 12 | `lag` | identical |
| 67 | 13 | `tau_gov` | identical |
| 68 | 15 | `v0`, `ic_order` | identical (`ic_order` restores through an `or`) |

**Five parameters, four rungs, one shape.** So the cell takes a `&MarchScope` — a small
`Default`-able by-value struct — added ONCE at slice Y carrying `b0`, and grown additively at
66/67/68. That is § 5.19 (iv)'s `Scope`, on the one cell the section named for it, and this slice
is where it earns its keep instead of costing one.

**THE 55 EXISTING CALL SITES DO NOT MOVE.** `stator_march` stays as an inherent method supplying
`MarchScope::DEFAULT`; `stator_march_scoped` is the one that takes it, and only rung 65's own
readers call that. The cell signature changes; no caller does. (Contrast slice V, where
`try_close`'s `&Scope` would have been non-additive at a live table — that is why it was refused
there and is affordable here.)

**AND THE GUARD SAVES THE PREVIOUS VALUE, WHICH `ForcedBleed` DOES NOT.** Slice X's RAII guard
restores to `None`, correct for `_b_forced` and **wrong** for `_b0`, whose Python is
save-and-restore-previous. Probe 3 measured the `_b0` guard's max nesting depth at **1** over the
whole four-suite grid — so on every shipped path the two spellings agree, and **no value key can
tell them apart**. `InitialBleed` therefore stores the previous value and step 5 manufactures the
nest, on `slice_x_dispatch.rs`'s precedent.

#### (iv) **`integrate_fuel`'s COST IS THE GENERIC, AND ITS SIGNATURE IS THE ONE THING THAT NEVER MOVED**

Probe 2's second table: `integrate_fuel` is overridden **10** times above rung 65 and all eleven
signatures are **character-identical** to rung 43's — no rung after 43 ever adds a parameter to
it. So unlike `_stator_march` there is nothing to open; the whole cost is that
`FuelTransientCore::integrate_fuel<S: Fn(f64) -> f64>` is **generic**, and a `fn`-pointer table
cannot hold a generic ([[rust-port-ladder-architecture]]).

The cell is therefore typed on `&dyn Fn(f64) -> f64`, the body moves to `r43_integrate_fuel`, and
the inherent generic method stays where it is as a one-line dispatcher. **~100 call sites across
`src` and `tests` are untouched**, and the `FuelTransientHooks` literals that spell every field
(`R43`, `R57_FUEL`, `R62_FUEL`, `R64_FUEL`, plus three in test files) go red — a
*non-exhaustive-literal* failure, i.e. the compiler enumerating the tables for me, exactly as
`LeverArming.lim` did at slice X step 1.

#### (v) **SLICE X's CODE-RESIDENT IOU, DISCHARGED: `_b_forced` STILL NEVER NESTS, AND THE COUNT IS 294× BIGGER**

`ForcedBleed::set`'s panic message names its own expiry: *"the paths that could break it are rung
65's closures and the 16 `super(LimitedBleedTransient, self)` pin sites at rungs 66–75. If you
are reading this from one of those, the measurement no longer holds."* Slice X's bound was a
name-based call graph over 1 705 calls — an UPPER bound, and a body read besides, which is what
slice W step 2 taught not to trust.

Re-measured at RUNTIME over rungs 62–65: **max depth 1, 0 nested events, in 501 506 trials**. The
panic stands, with a measured count instead of a graph. Rung 65's own closures are why it holds,
and the reason is structural: `der` clears `_b_state` in a `finally` **before** `command()` runs,
and `command` binds `super(LimitedBleedTransient, self)._close_fuel` — rung 63's closure — so the
trial solve never re-enters rung 64's.

#### (vi) **THE BRANCH SLICE X DECLARED DEAD IS NOW THE SECOND-BUSIEST, AND ITS GATE STAYS GREEN — CORRECTLY**

`b_of`'s `b_state` override was **0 of 1 705** at rung 64 and shipped anyway, gated by a
manufactured bug because *"a port that drops it passes every slice-X gate and breaks at slice Y"*.
Measured at rung 65: **200 960 of 908 001** (forced 501 506 / state 200 960 / super 206 535).

`slice_x_dispatch.rs::the_lagged_position_override_is_declared_live_and_unreached` asserts
`b_of_state == 0`, and it **stays green**, because its four census marches all build rung-64
machines. Predicted here rather than discovered at step 1: the gate is a statement about rung 64,
not about the counter.

#### (vii) **THE CONSTRUCTOR CHAIN IS TEN ASSERTS AND RUNG 65 MUST RE-SPELL ALL TEN TO RELAX ONE**

`LaggedBleedTransient` has no `__init__`; it inherits rung 64's, whose last assert reads
`bleed_lim is None or bleed_lim.tau is None or self._LAG_OK`, and `_LAG_OK` is a **class constant
flipped in the subclass** — one of § 5.19 (v)'s two live constant shadows, read at exactly one
site. `build_limited_bleed` hard-codes it `false` and its own comment forbids adding a bool
parameter (that would put rung 65's state in rung 64's signature). So the rung-65 builder
re-spells the chain, which probe 5 emits in execution order:

| # | class | assert |
|---|---|---|
| 1–2 | `TwoSpoolMatcher` | no polytropic knobs; `nozzle_convergent` |
| 3–6 | `ScheduledStatorTransient` | base maps unstatored; `vsv_lp` xor `sched_lp`; same on HP; `lp_disabled` carries no stator |
| 7–8 | `ScheduledBleedTransient` | `bleed` xor `bleed_sched`; `0 <= bleed < 0.5` |
| 9 | `LimitedBleedTransient` | the THREE-way arming exclusion |
| **10** | `LimitedBleedTransient` | **`tau is None or _LAG_OK` — the ONE that is relaxed** |

**THIS IS A DELIBERATE DUPLICATION AND MUST NOT BE FACTORED AWAY**
([[rust-port-copy-vs-rederivation]]). Its failure mode is that re-spelling silently drops one of
the other nine while every value key stays green, so step 3 gates **each surviving assert
individually on a rung-65 machine** rather than gating "the constructor refuses a lagged limiter".

#### (viii) THE INJECTION CENSUS — one run per rung-65 override DELETED

Slice X's shape: delete the override so Python's MRO supplies the parent body, which is exactly
the defect *"the Rust table spread `..R64` and forgot to name this cell"*. Its three repairs are
inherited (`with open(…)`, `-n0`, and the grid being rungs **62 + 63 + 64 + 65** rather than 65
alone) — **132 gates**.

**AND THE INSTRUMENT ITSELF NEEDED A FOURTH REPAIR, WHICH IS THE METHOD NOTE OF THIS SLICE.** The
first run used `-x`, as slice X's did, and reported a *first failing test* column. Three runs of
the SAME `_close_fuel` injection gave **three different answers** — `1 failed in 6.86s` (0 passed),
`plateau_breaks` (30 passed), `tau_to_zero` (22 passed). The repo's own `conftest.py` orders
collection LONGEST-FIRST off a **learned-duration cache that every run rewrites**, so under `-x`
that column is measuring the cache, not the code. The CAUGHT/UNGATED verdict survived all three;
the detail did not. Re-run without `-x`, the column becomes the **SET** of failing gates, which is
order-independent — and it is also strictly more informative. **Slice X's own census carries the
same defect**; its verdicts stand, its "first failing suite" column does not.

| injection (`cell` NOT swapped) | gates red of 132 | DID IT MOVE? | verdict |
|---|---|---|---|
| — (baseline) | **0** | — | — |
| **`_close`** | **0** | **NO — byte-identical in all 15 witness keys** | **UNGATED, and INERT** |
| `_close_fuel` | 7 | YES | caught |
| `b_at_point` | 2 | YES | caught |
| `at_lever` | 10 | YES — *after the witness was repaired* | caught |
| `integrate_fuel` | 12 | YES (raises) | caught |
| `_stator_march` | 3 | YES (`b0` refused as an unexpected keyword) | caught |

**THE ONE HOLE IS NOT A HOLE, AND THAT IS THE FINDING** — § (i). `_close`'s injection reddens
nothing *and moves nothing*, because rung 65's override only ever takes the arm that equals its
parent's. This is a strictly better outcome than slice X's two live-but-invisible holes: there is
no number to pin, only a branch to reach by hand.

**THE `at_lever` ROW IS SLICE X's `_isolating` LESSON, VERBATIM, ONE SLICE LATER.** The witness
first reported it **INERT while the suite was catching it** — because the witness only ever
marched the machine it was handed and never asked for a sibling. *Inert on my input is not inert.*
The separating key is a lagged sibling's CLASS: with rung 65's `at_lever` gone, rung 64's runs, its
builder refuses a lagged limiter, and the sibling raises instead of existing. Repaired before the
column was written down, which is the only reason it is right.

**AND THE ROW I ALMOST TYPED WAS WRONG.** The `_stator_march` figure was drafted as **8** from the
shape of the other rows while the run was still going, and measured **3**. It was left as
*(pending)* instead of typed — [[rust-port-guessed-census-bars]], *five typed count bars, five
wrong; measure counts* — and that is the only reason the table is right.

#### (ix) PRE-REGISTERED — written before a line of Rust

- **P1.** The 21 ported gates pass with **zero tolerance tiers** on the first oracle run, as
  slices T/U/V/W/X did — on the two BIT-FOR-BIT reduce arms and on every value key. The `tau → 0`
  arm is exempt BY CONSTRUCTION (P2) and its exemption is not a tolerance tier.
- **P2 — the reduce has THREE arms and the third must NOT be bit-exact.** `tau = None` (or
  `bleed_lim = None`) is rung 64 bit-for-bit **by dispatch**, at every arming mode; `b0 = None`
  passed explicitly at the value the march would have chosen is bit-for-bit; `tau → 0` CONVERGES
  and is asserted as a shrinking deviation (`dev_shrinks`, consecutive ratio 1.6–2.4 at the small
  end), never as an equality. **A tolerance appearing on arm one or two falsifies the carrier
  decision**, on slice X's P1 shape.
- **P3 — the two MIRROR ZEROS get manufactured gates, not value keys** (§ (i)). `_close` under a
  live `b_state` and `_close_fuel` without one are both unreachable on the shipped grid; each is
  gated by setting the carrier by hand and asserting the dispatch, so a port that drops either
  test fails to PASS rather than fails to compile.
- **P4 — `_b0`'s guard restores the PREVIOUS value**, gated by a manufactured nest. Falsified if a
  restore-to-`None` guard fails any gate in the slice — which § (iii) predicts it will not, and
  that prediction is the reason the manufactured gate exists.
- **P5 — the `MarchScope` costs no existing call site.** Falsified if any of the 55 shipped
  `stator_march` call sites has to change, or if any signature outside `StatorTransientHooks` and
  `FuelTransientHooks` moves.
- **P6 — `slice_x_dispatch.rs`'s `b_of_state == 0` stays GREEN** (§ (vi)), and
  `ForcedBleed::set`'s never-nests panic is NOT relaxed (§ (v)). Both reported explicitly at ship,
  held or not.
- **P7 — the ten-assert chain is gated assert-by-assert** (§ (vii)) on a rung-65 machine, and the
  count `10` is asserted so a future rung adding an eleventh cannot pass silently.
- **P8 — the ORACLE'S GRID IS THE SUITE'S, and the header says so AT THE TOP.**
  `bandwidth_ceiling` is 8 marches at `ds = 0.005` and `marginal_mode` 5 more, each with an outer
  root per sub-evaluation. If `dump_slice_y.py` coarsens, it is disclosed in the header and in the
  step-4 write-up — slice S step 4's lesson.
- **P9 — the `slow` COST IS MEASURED HERE.** § 5.19 (viii) said to measure the 263-gate `slow`
  bill at the first `slow`-heavy suite; at **9 of 21 (42.9 %)** this is it. The ported suite's
  wall-clock is reported at step 3 beside PyPy's, and `#[ignore]` is re-introduced only against a
  measured number — never against the Python marker (slice M's rule).
- **P10.** Steps, on slice X's shape: **1** the `integrate_fuel` cell + `MarchScope` + the `b0`
  carrier and its previous-value guard + the `pub` pins · **2** the port (`lagged_bleed.rs`) +
  `slice_y_smoke.rs` · **3** the 21 ported gates + the ten-assert chain (P7) + whatever § (viii)
  books as a hole · **4** `slice_y_oracle.rs` + `dump_slice_y.py`, PyPy and CPython arms · **5**
  the dispatch gates and the manufactured bugs (P3/P4/P6).

#### (x) THE STEP LOG

##### STEP 1 — SHIPPED. **THE DECOMPOSITION WAS STATED BEFORE THE COMPILER WAS READ, AND IT HELD**

Written before running `cargo test`: *step 1 is additive except for two things that must go red —
the `FuelTransientHooks` literals that spell every field (a non-exhaustive-literal failure), and
nothing else; `slice_x_dispatch.rs`'s `b_of_state == 0` stays green because its census marches all
build rung-64 machines.* Measured: **`E0063 missing field integrate_fuel` in `slice_v_dispatch.rs`
(2 literals) and nowhere else**, and the whole gate green afterwards. P6 held on both halves.

Shipped: the `integrate_fuel` cell (the third and last code-resident `⚠` note, discharged at its
definition), `r43_integrate_fuel` as a free function on `&dyn Fn`, `MarchScope` + the
`stator_march` signature opened once for rungs 65–68, `stator_march_scoped` beside the unchanged
`stator_march` (P5: **all 55 call sites untouched**), `TwoSpoolTransientCore::b0`, and the two new
guards — `MarchedBleed` (restores `None`) and `InitialBleed` (**restores the PREVIOUS value**).

**THE ONE JUDGEMENT CALL, AND IT IS THE SLICE'S ARCHITECTURE.** § 5.19 (iv)'s uniform `Scope` had
been retired field-by-field twice and looked dead. Probe 2 emitting every `_stator_march`
signature in the ladder is what revived it: rungs 65/66/67/68 add five parameters to **one** cell
in **one** shape, so a struct opened once costs one non-additive change instead of four. `Scope`
lives, on exactly the cell § 5.19 (iv) measured it onto and on no other.

##### STEP 2 — SHIPPED. **THE PORT WAS BIT-EXACT AND THE PROBE HARNESS WAS NOT**

`src/lagged_bleed.rs`, **937 lines**: the six cells, the four tables, `build_lagged_bleed`, the
march, and the three readers. `tests/slice_y_smoke.rs`, four sections.

**THE FIRST RUN FAILED, IT LOOKED EXACTLY LIKE A PORT DEFECT, AND IT WAS THE ORACLE HARNESS.**
`min_phi_lp` came out 5 ulps from the anchor and `nu0_lp` 7. Tracing back through the trajectory
put the first divergence at `nu0` — **before the march** — and then at the equilibrium solve, i.e.
in SHIPPED RUNG-64 CODE that slice X had certified bit-exact over 318 keys. Two Python
measurements of that one number also disagreed with each other.

The check that settled it: **re-run `dump_slice_x.py` against its committed TSV — 0 of 318
differed.** That proved the shipped code was innocent before a line of it was touched. The cause
was in my own probe files: they spell the gas constant `R_c = 0.4/1.4*1004.0`, where every suite
and every dump spells it `(1.4 - 1.0)/1.4*1004.0`. **`1.4 - 1.0` is not the double nearest `0.4`**,
so the probe was running a machine one ulp away from the suite's — and that one ulp moved `nu0_lp`
by seven and `min_phi_lp` by five.

Slice S step 4's lesson (*a probe's HEADER claimed the suites' grids and its code ran another*)
one level down: **not the grid, the GAS**. Six probe files carried it; all six repaired, the
witness re-measured, and the smoke anchors are now PyPy's. With the harness right, the port is
**bit-exact on the first run, zero tolerance tiers** — P1 intact on both bit-for-bit reduce arms.

**The rule this adds, and it is cheap:** when a port and an oracle disagree by a few ulps, FIRST
re-run the nearest committed dump against its TSV. A green diff moves the search into the harness
in one command; the two hours I spent bisecting a trajectory bought the same answer.

##### STEP 3 — SHIPPED. **21 PORTED GATES, ONE ADDED, AND THE `slow` BILL FINALLY MEASURED**

`tests/rung65.rs`: Python's 21 collected gates, all of them, plus § 5.23 (vii)'s constructor
chain. **21 of 22 green on the first run with zero tolerance tiers** (P1); the one failure was the
gate itself, not the port — `std::ptr::eq` on a `const` table, which is inlined at every use, so
the pointer identity it asserts is a property of the compiler and not of the machine. Replaced
with `fn_addr_eq` on the six swapped CELLS, `slice_v_smoke.rs`'s idiom, which is both reliable and
strictly stronger: it says WHICH bodies the sibling carries rather than which struct it points at.

**P9, DISCHARGED — the number § 5.19 (viii) said to take at the first `slow`-heavy suite.** Nine
of `test_rung65.py`'s 21 gates carry `slow` (42.9 %, against phase 6's 6.4 %). Ported, markers
dropped, **all 22 Rust gates run in 3.0 s against PyPy's 49.2 s for 21** — a **16×** whole-suite
speed-up, measured on the same box, single-process on both sides. No `#[ignore]` is
re-introduced: slice M's rule is satisfied by a measurement rather than by a hope, and the phase's
263 `slow` gates now have a data point rather than an estimate. At this ratio the phase's `slow`
bill is minutes, not hours, which is what § 5.19 (viii) wanted to know.

##### STEP 4 — SHIPPED. **35 994 KEYS BIT-EXACT ON BOTH INTERPRETERS, AND THE CENSUS THAT PROVES THE ORACLE CAN SEE FINDS ITS THREE BLIND SPOTS TO BE EXACTLY P3 AND P4**

`rust/oracle/dump_slice_y.py` + `rust/tests/slice_y_oracle.rs`, nine sections (A…I),
**35 994 keys**, green on the first run against PyPy **and** against CPython 3.14.3 — **0 drifts,
0 flips, no tolerance tier anywhere** (P1 intact on the oracle as well as on the ported gates).
Against slice X's 1 890, slice V's 6 819 and slice W's 9 422 this is the phase's largest dump so
far, and the reason is § (ii)'s: rung 65 adds a **STATE**, so the strongest available reading of
it is the state itself at every point of a march rather than a summary of it.

**P8 IS DISCHARGED BY NOT NEEDING IT.** Slice X coarsened three of seven sections and disclosed
each. Here **nothing is coarsened**: A/B/C/D/E run the suite's own `ds = 0.005` and F/G/H run
`0.01`, which is *also* the suite's own — its three reduce gates, its `b_at_point` gate and its
continuum-edge gate all march 0.01. That was not a hope: the four readers were **timed on PyPy
before the sections were chosen** (`bandwidth_ceiling` 3.4 s, `marginal_mode` 6.2 s,
`fuel_authority` 0.2 s, one lagged march 0.2 s), because rung 65's readers are *marches* where
rung 64's were nested root sweeps. The header states the grid line by line — slice S step 4's
lesson, *a probe's HEADER claimed the suites' grids and its code ran another*.

**ONE MAP SHAPE, AND REFUSING THE SECOND IS THE SAME RULE.** Slice X ran `shaped` and `tilted`
because rung 64's headline is a RATIO between two spools' bills. Rung 65's is a bandwidth sweep on
ONE spool's floor and `test_rung65.py` never builds a second shape, so a `tilted` arm would have
been a grid the suite does not have — the mirror image of the defect P8 exists to prevent, and it
is disclosed in the header rather than left as an absence.

**AND THE INSTRUMENT WAS MADE TO PROVE IT CAN SEE.** [[rust-port-slice-w-step3]] — *five of six
injections passed all 88 gates and the probe was wrong five times, every one a zero nobody
measured.* A green oracle says the port agrees with Python; it says **nothing** about which
defects it would catch. Six injections into the SHIPPED Rust, each re-running both arms, with the
DID-IT-MOVE column slice U step 1 said an injection table cannot do without:

| injection (one cell corrupted in `src`) | oracle | keys moved of 35 994 | pre-registered |
|---|---|---|---|
| — (baseline) | GREEN | 0 | — |
| **I1 `_close_fuel` drops `&& b_state.is_some()`** | **GREEN** | **0** | **UNGATED** — § (i)'s mirror zero |
| **I2 `_close`'s live arm deleted** | **GREEN** | **0** | **UNGATED** — the override is a no-op on the whole grid |
| I3 `b_at_point` RE-SOLVES (hands back `b_cmd`) | RED | **394** | CAUGHT — section D walks it at every point |
| I4 `stator_march` IGNORES `scope.b0` | RED | **1 280** | CAUGHT — section F marches three DIFFERENT initial positions |
| **I5 `InitialBleed` restores `None`** | **GREEN** | **0** | **UNGATED** — P4, and probe 3's depth-1 measurement is why |
| I6 `integrate_fuel` never reaches the valve-lag march | RED | — | CAUGHT — see below |

**ALL SIX PREDICTIONS HELD, AND THE THREE THAT SURVIVE ARE THE THREE STEP 5 ALREADY OWED.** I1 and
I2 are § 5.23 (i)'s two mirror zeros; I5 is P4's previous-value guard. **A 35 994-key oracle is
powerless over exactly the set the pre-registration named and over nothing else I could
construct** — which is the strongest thing this census could have returned, because it means P3
and P4 are not a hedge against an unknown but a complete accounting of the residue.

**I6's ROW HAS NO NUMBER, AND THE REASON IS REPORTED RATHER THAN GUESSED.** It reddens in **0.25 s
at `lagged_bleed.rs:221`** — `r65_b_at_point`'s own panic — inside section A's very first
`bandwidth_ceiling`, because a march that never dispatched to the valve-lag integrator carries
points with no valve state. So it is caught by the PORT'S OWN ASSERT before a single key is
compared, and the census reported `-1` rather than a count. That `-1` was re-run alone and read,
not filled in from the shape of the other rows — [[rust-port-guessed-census-bars]] and slice X
step 3's *a probe that cannot tell a crash from a compile error calls its best result its own
failure*, which is also why the harness reports `BUILD` as its own outcome.

**THE LOADER'S KEY-COUNT BAR WAS MEASURED, NOT INHERITED.** Slice X's loader asserts
`len() > 1_800` because slice X emitted 1 890. Copying that here would have passed on **5 %** of
the file — a truncated golden with 1 801 keys would have parsed clean. The bar is `> 35_000`,
taken after the first dump ran.

**TWO HAZARDS PAID FOR, BOTH FROM MEMORY RATHER THAN FROM A DEBUGGING SESSION.**

1. **THE GAS SPELLING, CHECKED BEFORE THE FIRST KEY WAS EMITTED.** Step 2 lost hours to six probe
   files spelling `R_c = 0.4/1.4*1004.0` where every suite spells `(1.4 - 1.0)/1.4*1004.0`. The
   dump was grepped for that pattern before it was ever run; the only hit is the docstring warning
   about it.
2. **THE POWERSHELL BOM, CAUGHT BY A THREE-BYTE READ.** The first PyPy arm was generated with
   PowerShell's `1>` and came out with a **UTF-8 BOM** in front of the `#` on line 1 — which makes
   `line.starts_with('#')` false, so the header would parse as a data line and the loader would
   die on `InvalidDigit`. `head -c 3` against a committed golden (`23 20 73` vs `ef bb bf`) is
   what found it, before Rust ever read the file. Both arms are now generated through a POSIX
   shell and the loader's panic message names the BOM as one of its two causes.
   [[windows-tooling-file-hazards]].

**THE ONE DIVERGENCE THE ORACLE STRUCTURALLY CANNOT SEE, NAMED IN ITS OWN HEADER.**
`marginal_mode`'s `laws_held` is `float("nan")` on a cell with no riding points; Rust's `f64::max`
DISCARDS a NaN operand where Python's `max()` does not — two different functions, and a port that
picks the wrong one is invisible unless the branch is reached. Measured on this grid: `n_ride` is
**340 / 251 / 214** on natural/lo/hi and **340** on both taucells, so the NaN path never fires.
That is a **fourth zero**, found by asking of the reader what its degenerate case is rather than
by an injection, and it goes on step 5's list beside P3 and P4.

**⚠ AND THE SENTENCE ABOVE ORIGINALLY SAID PYTHON'S `max()` "PROPAGATES", WHICH IS WRONG —
CORRECTED AT STEP 5 BY RUNNING THE THREE POSITIONS IN AN INTERPRETER.** It is POSITION-DEPENDENT:
`max` holds the first element and replaces it only on a strict `>`, and every comparison against
NaN is false, so `max(nan, 1.0, 2.0)` is `nan` while `max(1.0, nan, 2.0)` is `2.0`. The
call site puts `natural` first — the one position where the two spellings part company — so the
port was on the wrong one and it was a REAL defect, not merely an ungated branch. Left visible
here rather than silently rewritten: a guess about a language's semantics is exactly the kind of
thing this plan makes a habit of measuring.

##### STEP 5 — SHIPPED, AND THE SLICE CLOSES. **THE ORACLE'S THREE BLIND SPOTS GET THEIR GATES, A FOURTH DEFECT WAS REAL, AND MUTATING MY OWN GATES FOUND ONE OWNED BY NOBODY**

`rust/tests/slice_y_dispatch.rs`, **8 gates, zero source lines** — plus one repair in
`lagged_bleed.rs` that step 4's degenerate-case question turned up. P3, P4 and P6 discharged.

**THE FOURTH DEFECT, AND IT WAS A REAL ONE IN SHIPPED CODE.** § 5.23's pre-registration named three
things no value key could carry. Asking `marginal_mode` for its degenerate case found a fourth,
and unlike the other three it was **wrong**, not merely ungated:

```text
python: max(nan, 1.0, 2.0) -> nan      rust: NAN.max(1.0).max(2.0) -> 2.0     DIFFERENT
python: max(1.0, nan, 2.0) -> 2.0      rust: (1.0).max(NAN).max(2.0) -> 2.0   same
```

`f64::max` **discards** a NaN operand; Python's builtin holds the first element and replaces it
only on a strict `>`, and every comparison against NaN is false — so a NaN in the **FIRST**
position survives and one in any later position does not. `laws_held` is `float("nan")` on a cell
with no riding points and the aggregate is `max(natural, lo, hi)` with `natural` first: **the one
position where the two spellings part company.** The port shipped `a.max(b).max(c)`. Repaired to
`py_max3`, which is Python's algorithm rather than a name for it.

**AND "IT PROPAGATES" WAS ITSELF A WRONG GUESS** — step 4's own header said so before the three
positions were run in an interpreter. The header is corrected. Two neighbouring reductions
(`frozen`, `tau_span`) keep `f64::max` because their operands cannot be NaN, and that is stated at
the line rather than swept into a helper they do not need ([[rust-port-copy-vs-rederivation]]).

**THE FILE READS NO GOLDEN, ASKED ASSERTION BY ASSERTION.** Slice V step 5's lesson — *the four
that read nothing survive a regenerated golden* — inverted here: every gate is a counter, a
same-run difference between two dispatch arms, or a property of a shipped function, so
regenerating `slice_y_pypy.tsv` cannot make one pass or fail. That is the point of the file.

**THEN I MUTATED MY OWN GATES**, which is the step's real content. Nine injections, each re-run
against BOTH files:

| injection | `slice_y_dispatch` (8) | `slice_y_oracle` (35 994) | pre-registered |
|---|---|---|---|
| — (baseline) | GREEN | GREEN | — |
| I1 `_close_fuel` drops the state test | **RED** | GREEN | RED — this file exists for it |
| I2 `_close`'s live arm deleted | **RED** | GREEN | RED — this file exists for it |
| I3 `b_at_point` RE-SOLVES | GREEN | **RED** | either — the oracle owns it (394 keys) |
| **I4 `stator_march` IGNORES `scope.b0`** | **GREEN** ⚠ | **RED** | **RED — AND IT WAS NOT** |
| I5 `InitialBleed` restores `None` | **RED** | GREEN | RED — P4 exists for it |
| I6 `integrate_fuel` skips the valve-lag march | **RED** | **RED** | either — the oracle aborts |
| I7 `py_max3` re-spelled as `f64::max` | **RED** | GREEN | RED — the function IS the gate |
| I8 the `laws_held` CALL SITE re-spelled | **RED** (after the repair below) | GREEN | GREEN — and the hole was CLOSED rather than declared |
| I9 `MarchedBleed` restores the previous value | **RED** | **RED** | RED — opposite policies, both gated |

**I4 IS THE FINDING, AND IT IS A GATE THAT WAS TESTING HALF OF WHAT IT NAMED.** The manufactured
nest asserted that after an inner march the outer `b0` is back — and a guard that saves and
restores the outer value passes that **no matter what it SETS**. So `r65_stator_march` passing
`None` instead of `scope.b0` walked straight through it. **The two halves of ONE guard were owned
by two different files** — the oracle caught the SET at 1 280 keys, this file caught the RESTORE,
and neither caught both. A defect touching only the set would have been caught by a golden that
someone might one day regenerate; a defect touching only the restore, by nothing at all if the
nest had been written slightly differently. The gate now also asserts the nested march's first
point carries the `b0` it was handed, and I4 re-run against the strengthened gate is **RED**.

**I8 WAS DECLARED A LIMIT AND THEN CLOSED, WHICH IS THE BETTER ANSWER.** Re-spelling the
`laws_held` CALL SITE back to `f64::max` was caught by nothing: `n_ride` is 340 / 251 / 214 on
natural/lo/hi and 340 on both taucells, so **no reachable march produces a NaN in any position**
and a gate on `marginal_mode`'s OUTPUT would be satisfied by either spelling. The first write-up
of this step stopped there and disclosed the hole. It did not have to: § 6's
runtime-introspection table already sanctions `include_str!` + `.matches().count()` as the
replacement for exactly this shape of *"not reachable"* assertion, and it is already the shipped
answer at `test_rung73.py:488`'s port. Two counted assertions on the call-site text — both
MEASURED against the file before being typed — turn I8 **RED**. **A declared limit is worth
writing down; it is not worth keeping when the project already owns the instrument that removes
it.** The gate says in its own doc comment that it is textual and not a value gate, so nothing
here pretends to be a measurement it is not.

**P6, BOTH HALVES, REPORTED AT THE RUNG THE SLICE-X PANIC NAMED AS ITS OWN EXPIRY.**
`slice_x_dispatch.rs`'s `b_of_state == 0` **stays green** — its four census marches all build
rung-64 machines, exactly as § 5.23 (vi) predicted before step 1 — and this file gates the other
side: on a rung-**65** machine the branch slice X declared dead is LIVE. `ForcedBleed::set`'s
never-nests panic is **not relaxed**: a whole lagged march does not trip it (the structural reason
is rung 65's own — `der` clears `_b_state` before `command()` runs, and `command` binds rung 63's
closure, so the trial solve never re-enters rung 64's) and a manufactured nest still panics.

**THE PRE-REGISTRATION, ROLL-CALLED IN FULL — all ten, none silent.**

| | held? | where it was reported |
|---|---|---|
| **P1** zero tolerance tiers, first run | **YES** | step 3 (21 of 22 gates) and step 4 (35 994 keys, BOTH arms) |
| **P2** three reduce arms, the third NOT bit-exact | **YES, and its typed band is GATED** | the `tau → 0` arm is `dev_shrinks` with the small-end ratio asserted `1.6 < r < 2.4` at `rung65.rs:363` — **ported from Python's own suite, not typed from a probe write-up**, which is the failure mode [[rust-port-guessed-census-bars]] names. No tolerance appeared on arms one or two, so the carrier decision stands |
| **P3** the two mirror zeros get manufactured gates | **YES** | step 5, and the mutation table proves both redden |
| **P4** `_b0`'s guard restores the PREVIOUS value | **YES** | step 5 — and the gate had to be STRENGTHENED first (I4) |
| **P5** `MarchScope` costs no existing call site | **YES** | step 1: all 55 `stator_march` sites untouched |
| **P6** slice X's `b_of_state == 0` green; `ForcedBleed`'s panic not relaxed | **YES, both** | step 1 and again at step 5, each half its own gate |
| **P7** the ten-assert chain gated assert-by-assert, count pinned | **YES** | step 3's added gate |
| **P8** the oracle's grid IS the suite's, header says so | **YES — with NOTHING coarsened** | step 4 |
| **P9** the `slow` bill measured here | **YES** | step 3: **16×**, 3.0 s against 49.2 s |
| **P10** the five steps, in the stated order | **YES** | — |

**SLICE Y IS COMPLETE — all five steps SHIPPED.** Rung 65 is `lagged_bleed.rs` (six cells, four
tables, plus `py_max3`), `rung65.rs` (22 gates, 16× faster than PyPy's 21), `slice_y_smoke.rs`
(4), `slice_y_oracle.rs` (35 994 keys, two interpreters, zero tolerance tiers) and
`slice_y_dispatch.rs` (8). The next slice is **Z**.



### 5.24 SLICE Z (rungs 66 + 67, `TwoLagCascadeTransient` + `CrossLoopCascadeTransient`) — PRE-REGISTERED, nine probes MEASURED first

`M:\claud_projects\temp\rust-phase7\probe_z1.py` … `probe_z9.py`, PyPy (plus CPython 3.14 on
probe 8). Every table below is **EMITTED** by one of them (§ 5.19 (xi)).

#### (i) THE LEADING FINDING — **A PROBE THAT READS A GRID PARAMETER INSTEAD OF THE DELIVERED ONE MEASURES A DIFFERENT FUNCTION, AND THE ANSWER IT RETURNS CAN BE THE OPPOSITE ONE**

Rung 67 has exactly ONE float `sum()` — `cross_identity`'s `P_mid = sum(prods) / len(prods)`.
(The other three `sum(...)` sites in the two rungs are `sum(1 for …)`, integer counts, and rung
66 has none.) That is slice W's exemption shape verbatim: CPython 3.12+'s `sum()` is
Neumaier-COMPENSATED and PyPy's is naive, so a Rust left fold can agree with one interpreter and
not the other. Probe 6 captured the products the shipped reader actually multiplies; probe 8 asks
both interpreters for `sum()` of that same list.

**PROBE 8's FIRST ANSWER WAS "NO DIVERGENCE", AND IT WAS WRONG.** It chunked the captured
products by `n_sample = 8` — the value the gate PASSES. Probe 9 then printed the reader's own
`n_sample` back and it is **9**: `sub = ride[:: max(1, len(ride) // n_sample)]` is a STRIDE, so
the delivered count is `len(ride) // (len(ride) // n_sample)` and not `n_sample`. Chunked at 8,
probe 8 was summing windows the reader never sums. Chunked at 9 the answer inverts:

| chunk (9 wide) | PyPy `sum` | CPython 3.14 `sum` | naive left fold | verdict |
|---|---|---|---|---|
| 0 | `-0.1804633613623593` | `-0.1804633613623593` | `-0.1804633613623593` | agree |
| 1 | `-0.18349781418498853` | `-0.18349781418498856` | `-0.18349781418498853` | **CPython DIFFERS** |
| 2 | `-0.18747977463936186` | `-0.18747977463936186` | `-0.18747977463936186` | agree |
| 3 | `-0.18349781418498853` | `-0.18349781418498856` | `-0.18349781418498853` | **CPython DIFFERS** |

Chunks 1 and 3 are the SAME nine products (the `tau_gov = 0.05` row, reached twice), so **1 of
`cross_identity`'s 3 rows** carries the divergence. `P_mid` there is
`-0.02038864602055428` on PyPy and `-0.020388646020554284` on CPython — **one ulp**.

**AND THE AMPLIFICATION QUESTION IS ANSWERED BY MEASUREMENT, NOT BY FEAR** (slice M's rule).
`P_mid` feeds `self._window(P_mid)`, whose six numeric outputs run through `**0.5` twice. Both
values pushed through `_window` in ONE interpreter:

| `_window` key | ulps apart |
|---|---|
| `P` | **1** |
| `T_over_tau` | **1** |
| `k`, `zeta`, `rho_lo`, `rho_hi`, `reciprocal` | **0** |

**Two of eight keys move, by one ulp, and five absorb it exactly.** So the CPython exemption
slice Z owes at step 4 is a NAMED, COUNTED pair on one row — not a tolerance tier.

**MEASURED ON THIS GRID, NOT PROVED OF THE READER.** The chunk width is
`len(ride) // n_sample`, and probe 9 measured `len(ride)` at **135 / 97 / 91** for the three
`tau_gov` values at the suite's own `ds = 0.005`. A different `ds` gives a different stride and
possibly a different width; step 4 must re-read the delivered count rather than inherit the 9.

#### (ii) THE SCOPE, ENUMERATED — **AND "0 CELLS" IS CONFIRMED BY AN EMITTER FOR BOTH RUNGS**

**Source** (`ast` spans): `TwoLagCascadeTransient` **653** lines (10854–11506), 11 methods;
`CrossLoopCascadeTransient` **843** lines (11509–12351), 15 methods — **1 496 lines.**

**AND THE “LARGEST SO FAR” IS A MEASURED RANKING, NOT A SUPERLATIVE** —
[[rust-port-guessed-census-bars]] is five typed count bars that were all wrong, and a
phase-7 span table had never been emitted. Probe 1, over every class each phase-7 slice
ports:

| slice | Python source lines | classes |
|---|---|---|
| **Z (66–67)** | **1 496** | `TwoLagCascadeTransient` 653 + `CrossLoopCascadeTransient` 843 |
| V (57–60) | 1 140 | `ScheduledStatorTransient` 1 023 + `IncidenceLimiter` 75 + `StatorSchedule` 42 |
| W (62–63) | 908 | `ScheduledBleedTransient` 858 + `BleedSchedule` 50 |
| X (64) | 505 | `LimitedBleedTransient` 438 + `BleedLimiter` 67 |
| Y (65) | 479 | `LaggedBleedTransient` 479 |

So Z is the largest, by 31 % over V — **and the units matter**: slice W turned 908 Python
lines into a 1 869-line `bleed_transient.rs`, a 2.06× expansion. Carried across, slice Z's
Rust lands near **3 000 lines**. That is an ESTIMATE, labelled as one.

**Cells — probe 1's emitted census. BOTH rungs add ZERO and swap the SAME THREE:**

| rung | SWAPS | PLAIN (new, never overridden) | cells ADDED |
|---|---|---|---|
| 66 | `_stator_march`, `at_lever`, `integrate_fuel` | `_eig`, `_gains`, `_integrate_fuel_cascade`, `_violation`, `cascade_bill`, `cascade_identity`, `marginal_mode_cascade`, `merge_identity` (8) | **0** |
| 67 | `_stator_march`, `at_lever`, `integrate_fuel` | `_exceed`, `_gains_cross`, `_integrate_fuel_cross`, `_joint_fixed_point`, `_sign_changes`, `_window`, `cross_bill`, `cross_identity`, `detector_sensitivity`, `joint_ic_corners`, `marginal_mode_cross`, `oscillation_window` (12) | **0** |

§ 5.19 (x) predicted **0** for slice Z and the emitter agrees — **the third and fourth rows of
that column an emitter confirms**, after X's 1 and Y's 0. All three swapped cells already exist:
`at_lever` was opened at W, `_stator_march` at V, `integrate_fuel` at Y.

**Tests**: `test_rung66.py` **15 → 15** collected, **5 slow (33.3 %)**, 343 lines;
`test_rung67.py` **23 → 23**, **9 slow (39.1 %)**, 507 lines. **No parametrize expansion at all
in either** — the third slice in a row at 1.00× (W was 2.1×, X 1.3×). **Ported gate count 38.**

**Modules**: two, on `lagged_bleed.rs`'s shape. The names are **PROVISIONAL** — the crate's
convention is thematic and short (`lagged_bleed.rs`, `limited_bleed.rs`, `bleed_transient.rs`)
rather than class-name transliteration, so step 1 picks them and is not bound by a sentence
written here.

#### (iii) **`MarchScope` GROWS TWICE, ADDITIVELY, AND BOTH NEW FIELDS ARE SAVE-AND-RESTORE-PREVIOUS**

§ 5.23 (iii) built `MarchScope` carrying `b0` on the stated promise that 66/67/68 grow it
additively. **Slice Z is that promise's first test.** Probe 2 emits the ladder:

| rung | `_stator_march` args | ADDS | guard |
|---|---|---|---|
| 57–60 | 10 | *(base)* | — |
| 65 | 11 | `b0` | `prev, self._b0 = self._b0, b0` … `finally: self._b0 = prev` |
| **66** | **12** | **`lag`** | `prev, self._lag = self._lag, lag` … `finally: self._lag = prev` |
| **67** | **13** | **`tau_gov`** | `prev, self._tau_gov = self._tau_gov, tau_gov` … `finally: self._tau_gov = prev` |
| 68 | 15 | `v0`, `ic_order` | same, `ic_order` restoring through an `or` |

**The two questions were asked SEPARATELY per field** — [[rust-port-slice-n-step4]] (*a carrier
claim on ONE hook says nothing about the next*) — and both answers came back the same:
save-and-restore-**previous**, and probe 3 measured **max nesting depth 1** for `_lag` and **1**
for `_tau_gov` over the whole rung-62..67 grid, with **0 nested events** either way. So, exactly
as at slice Y for `_b0`, **no value key can tell restore-previous from restore-`None`**, and the
nest must be MANUFACTURED at step 5 — twice, once per field.

`MarchScope` stays `Copy` + `Default`: `AsymmetricLag` is already `#[derive(Clone, Copy)]`, so
`lag: Option<AsymmetricLag>` and `tau_gov: Option<f64>` are additive fields and
`MarchScope::DEFAULT` gains two `None`s.

**AND THE OTHER TWO CELLS DO NOT MOVE.** `at_lever` is 7 args at 64, 65, 66 AND 67 (68 adds
`stator_lim`); `integrate_fuel` is 13 args at 65, 66, 67 AND 68. Slice W's P5 and § 5.23 (iv)
both hold at their next test.

#### (iv) **THE TWO JOINT INITIAL CONDITIONS ARE A DELIBERATE DUPLICATION, AND THE PORT MUST NOT FACTOR THEM**

Both marches need the two laws' simultaneous equilibrium at `s = 0`, and the two rungs solve it
differently ON PURPOSE:

* **rung 66** iterates INLINE and UNDAMPED, capped at 60, and asserts on failure — because its
  own identity pins the contraction factor `|R_q C_g|` at 1, so a stall there genuinely IS the
  degeneracy the rung is about, and the assert says so.
* **rung 67** calls `_joint_fixed_point`, which sweeps `w ∈ (1.0, 0.5, 0.25)` — because on
  cascade A `|P|` is pinned by nothing, `det J = (1−P)/(t_g t_v) ≠ 0` for every `P ≠ 1`, so a
  stall would be a SOLVER failure and reporting it as a marginal mode would be a false finding.

Both docstrings say this at length. [[rust-port-copy-vs-rederivation]] is the standing rule and
this is the reflex it exists to stop: a port that gives rung 66 a call to rung 67's damped solver
is bit-exact on the shipped grid (probe 3: `w = 1.0` on 36 of 39 calls) and destroys the
distinction the two asserts are FOR.

Probe 3 also measured that the damping ladder is genuinely exercised: `w = 1.0` **36**,
`w = 0.5` **2**, `w = 0.25` **1**, with `fix_q` split **33 / 6** and `converged = True` on all
39 — so `_joint_fixed_point`'s failing assert is DEAD on the grid, and exactly one call leaves
its inner `res <= tol` (1e-12) break unfired while the outer `res <= 1e-9` fires.

#### (v) THE ARM CENSUS — **AND SLICE Y's MIRROR-ZERO SHAPE DOES *NOT* RECUR**

Probe 3, over rungs 62 + 63 + 64 + 65 + 66 + 67 in one process (`-n0`; xdist workers never see a
monkeypatch — slice X defect 2), `pytest exit = 0`:

| cell | `armed=F, arg=F` | `armed=F, arg=T` | `armed=T, arg=F` | `armed=T, arg=T` |
|---|---|---|---|---|
| r66 `integrate_fuel` | 18 | 4 | 12 | **32** |
| r67 `integrate_fuel` | 5 | 6 | 8 | **38** |

**All four arms of both dispatches are LIVE.** Slice Y's leading finding — two mirror-image
one-armed dispatches, each override only ever taking the arm that equals its parent's — is a
property of rung 65 and does not carry up. Registered here because the previous slice's headline
is exactly the thing a pre-registration is tempted to inherit.

**THE FOUR ZEROS THAT DO EXIST, AND EVERY ONE IS EXHIBITED.** Probe 7 makes each counter read 1
from a direct call, so a zero is the GRID's and not the instrument's ([[rust-port-slice-w-step3]]
— *five of six injections passed all 88 gates and the probe was wrong five times, every one a
zero nobody measured*):

| dead arm | count on the grid | exhibited by probe 7 |
|---|---|---|
| r66 `_gains` with `accel` armed | **0 of 80** (all 80 are `accel=None, surge=…`) | yes, on the arguments |
| r66 `_gains` with NEITHER leg — the `if caps else 0.0` fall-through | **0 of 80** | yes |
| r67 `_sign_changes` `peak <= 0` early return | **0 of 10** | `_sign_changes([0,0,0]) = 0` |
| r67 `_window`'s `P == 0` → `T_over_tau = inf` | **0 of 31** | `_window(0.0)['T_over_tau'] = inf` |
| r66 `_violation` with the `s > s_hi` break NOT taken | **0 of 41** | `_violation(traj, 0.55, 99.0)` |

#### (vi) **`_eig`'s COMPLEX ARM IS DEAD ON THE RUNG THAT DEFINES IT, AND LIVE ONLY THROUGH THE RUNG ABOVE**

Probe 3 counted rung 66's `_eig` at **134 real / 57 complex** and that number is a trap: `_eig`
is defined on rung 66 and CALLED by rung 67's `cross_identity` too. Probe 7 re-ran the census
over `test_rung66.py` ALONE, splitting by calling function:

```
80  _eig:cascade_identity:real=True
 0  _eig:*:real=False
```

**80 of 80 real, and the complex arm never runs on rung 66 at all** — which is not an accident
but rung 66's own headline: `det J ≡ 0` makes the spectrum exactly `{0, tr J}`, so the
discriminant `tr² − 4·det` is `tr² ≥ 0` identically. The arm is kept alive one rung up, where
`det J ≠ 0`. A port that drops it passes every rung-66 gate and breaks at rung 67 — § 5.23
(iii)'s shape, one slice later and in the opposite direction.

#### (vii) THE NUMERICS CENSUS — probe 4, and ONE ITEM IS A MEASURED NEGATIVE

| hazard | rung 66 | rung 67 | verdict |
|---|---|---|---|
| float `sum()` | 0 | **1** (`cross_identity.P_mid`) | § (i) — CPython exemption OWED |
| `sum(1 for …)` integer counts | 0 | 3 | inert |
| `** 0.5` | 2 (`_eig`) | 3 (`_window`) | `.sqrt()`, never `powf(0.5)` — `gas.rs` § 2 |
| `max(3 args)` | **1** (`marginal_mode_cascade.frozen`) | 0 | **NEGATIVE, measured** |
| `max`/`min` with `default=` | 0 | 3 | never fires — probe 9 |
| `min(seq, key=…)` | **1** (`cascade_bill`) | 0 | a LOCATION key: Python returns the FIRST minimum |
| `float('nan')` literals | 11 | 15 | reached only through empty-sequence guards |
| `int(round(s_end / ds))` | 1 | 2 | the ladder's existing step-count spelling |

**THE `py_max3` DEFECT DOES NOT RECUR, AND THAT IS A MEASUREMENT.** Rung 66's one three-argument
`max` is `frozen = max(nat["drift"], moved["lo"]["drift"], moved["hi"]["drift"])`, and `drift` is
itself a `max` over a non-empty trajectory, so it cannot be NaN — an `f64::max` chain is exact
there. Rung 66's `laws_held` is a SINGLE value, not a reduction. Probe 9 measured the empty-case
guards that could have produced a NaN and none fires on the grid: `marginal_mode_cascade`'s `on`
holds **114** points, `oscillation_window`'s `live` **3 of 3 rows**, `joint_ic_corners`'s `ok`
**4 rows with `n_live = 1`**, and `nat["removed"] = 1.36e-3 ≠ 0` so `dremoved_rel`'s division is
safe.

#### (viii) **P8 IS AFFORDABLE — THE READERS WERE TIMED BEFORE THE DUMP WAS DESIGNED**

Slice Y discharged *"the oracle's grid IS the suite's"* by timing rather than by coarsening and
disclosing. Probe 6, at each gate's OWN arguments (copied from the calling test, not chosen):

| reader | s | | reader | s |
|---|---|---|---|---|
| r66 `merge_identity` | 2.15 | | r67 `cross_identity` | 4.39 |
| r66 `cascade_identity` | 2.06 | | r67 `oscillation_window` | 9.01 |
| r66 `cascade_bill` | 2.50 | | r67 `cross_bill` | 2.92 |
| r66 `marginal_mode_cascade` | 2.07 | | r67 `marginal_mode_cross` | 3.65 |
| | | | r67 `joint_ic_corners` | 4.55 |
| | | | r67 `detector_sensitivity` | 0.01 |
| | | | **TOTAL** | **33.31** |

**33 s for all ten readers at the suites' own grids**, so nothing needs coarsening and P8 is
again discharged by not needing it.

#### (ix) THE `slow` BILL, MEASURED IN PYTHON BEFORE ANY RUST EXISTS

`test_rung66.py` **15 passed in 31.46 s**; `test_rung67.py` **23 passed in 59.61 s** — **91.07 s
for 38 gates**, of which 14 carry `slow`. Slice M's rule applies unchanged: port the gate, DROP
the marker, and re-introduce it only against a MEASURED Rust number. Slice Y's Rust suite came
in **16×** faster than its Python original; that is the prior, not a prediction.

#### (x) THE PREDICTIONS

- **P1 — `MarchScope` GROWS ADDITIVELY AND NO CALL SITE MOVES.** `lag` and `tau_gov` are added
  fields; `stator_march` keeps supplying `MarchScope::DEFAULT`; all 55 shipped call sites and
  every `stator_march_scoped` signature stay as they are. **Falsified if any existing caller has
  to change** — § 5.23 (iii)'s promise at its first test.
- **P2 — SIX reduce ARMS onto FIVE distinct TARGETS, every one bit-for-bit BY DISPATCH.**
  Rung 66 reduces to rung 64 (`_lagged()` false), rung 65 (`lag is None`) and rung 52; rung 67
  to rung 65, rung 66 (`tau_gov is None`) and rung 47 — six arms, and rung 65 is a target of
  both, so five targets. (The noun is named because the count is checkable at step 2 and
  because the previous commit was a tally that did not add up.) Probe 3 measured all four
  dispatch arms live on both cells, so unlike slice Y no reduce arm is unreachable.
- **P3 — the CPython arm carries ONE DECLARED EXEMPTION, on `cross_identity`'s `P_mid` and the
  `T_over_tau` it feeds** (§ (i)). **Both keys are VERIFIED REACHABLE**: `cross_identity`'s row
  dict spreads `win`'s keys in through `**{k: win.get(k) for k in ("rho_lo", "rho_hi", "zeta",
  "T_over_tau", "opens", "reciprocal")}`, so `T_over_tau` is a published key and not an
  internal. Falsified if a third key moves, or if the exemption needs a tolerance tier rather
  than a named key list.
- **P4 — the FIVE dead arms of § (v) get MANUFACTURED gates at step 5, not value keys**, and the
  two `_gains` arms are gated on the ARGUMENTS the way probe 7 exhibits them.
- **P5 — `_eig`'s complex arm ships LIVE and rung-66-unexercised** (§ (vi)), gated by a direct
  call with a negative discriminant. Falsified if any rung-66 value key can see it.
- **P6 — the two joint-IC solves stay SEPARATE** (§ (iv)). Falsified if the port routes rung 66
  through `_joint_fixed_point`, which would be bit-exact and wrong.
- **P7 — the `lag` and `tau_gov` guards restore the PREVIOUS value, gated by TWO manufactured
  nests** (§ (iii)). Falsified if any value key can tell restore-previous from restore-`None`.
- **P8 — THE ORACLE'S GRID IS THE SUITES', with NOTHING coarsened** (§ (viii)), and the header
  says so at the top. If anything is coarsened it is disclosed in the header AND in the step-4
  write-up — slice S step 4's lesson.
- **P9 — the `slow` cost is measured in Rust, never mapped from Python's markers** (§ (ix)).
- **P10 — five steps, on slice W's two-rung shape:** **1** the `MarchScope` fields (`lag`,
  `tau_gov`) + their `_lag`/`_tau_gov` carriers + the three cell swaps + rung 67's `_RINGS`,
  which is a class CONSTANT a gate reads and not a carrier at all · **2** BOTH ports + `slice_z_smoke.rs` —
  **the largest single body in phase 7 so far, 1 496 source lines** · **3** both rung suites
  (15 + 23) + the injections · **4** `slice_z_oracle.rs` + `dump_slice_z.py`, carrying § (i)'s
  exemption block · **5** the dispatch gates and the manufactured bugs (P4/P5/P6/P7).



##### STEP 1 — SHIPPED. **The plumbing, the refusals, and a REFUSAL THAT CANNOT SEE WHAT IT REFUSES**

`MarchScope`'s two fields, their two carriers and guards, the six cell bodies (three cells x two
rungs), the two builders, the eight tables, `RINGS`, and `tests/slice_z_smoke.rs` (**9 gates,
green**). The two marches are declared and stubbed; they are step 2. The whole Rust suite is
**green, exit 0, zero failures in every target**; `cargo clippy --all-targets` reports one error
and it is PRE-EXISTING (`stator_transient.rs:2610`, `eq_op` on a deliberate NaN test) -- verified
by stashing this step's diff and re-running, which returned the same one.

**AND THE AGGREGATE PASS COUNT OF THAT RUN IS NOT QUOTED, BECAUSE I DESTROYED IT.** The run was
piped through `sort | uniq -c | head -20`, which reports the per-target lines and drops the
total. The count is not re-derived: re-running a 10-minute gate to recover a number it already
computed is the *never run the gate for timing* rule one level over, and a total invented from the
surviving rows would be exactly the typed-instead-of-emitted tally this phase has now been caught
on three times. The verdict that matters -- exit 0, no target failed -- is intact.

**P1 IS FALSIFIED AT ITS LETTER, HELD AT ITS INTENT, AND THE BILL IS PAID ONCE.** The signature
half of P1 holds exactly -- `stator_march` and `stator_march_scoped` are character-identical to
what slice Y shipped and no un-scoped call site moved.

**AND P1's OWN "55" WAS STALE, WHICH IS A SECOND HALF TO THE VERDICT.** The number was typed into
`MarchScope`'s doc comment at slice Y and P1 inherited it without re-running the count. Measured
at slice Z over `src/` and `tests/`, comments excluded: **82** un-scoped call sites (91 once slice
Z's own file lands) and **16** scoped ones. The verdict is unchanged -- none of the 82 moved --
but this is [[rust-port-guessed-census-bars]] firing on a bar that had already been WRITTEN DOWN
once, which is the harder version: a typed count that survives a slice reads like a measurement.
Both stale spellings in `stator_transient.rs` are corrected and the count now lives in ONE place. But **adding a field to a
struct is a compile error at every EXHAUSTIVE STRUCT LITERAL of it, and the port had NINE**: one
in `src/lagged_bleed.rs` and **eight in four TEST files** (`rung65.rs` x3, `slice_y_oracle.rs` x2,
`slice_y_smoke.rs` x2, `slice_y_dispatch.rs` x1) -- which is exactly where a `src/`-only grep
would have missed them, and § 5.23 (iii)'s promise was written from a `src/` reading. All nine
took the one-token repair `..MarchScope::DEFAULT`, and a functional-update literal absorbs the
NEXT field silently, so slice AA's `v0`/`ic_order` costs zero edits at these sites. **Growth is
free from the SECOND time on, not the first** -- booked here rather than patched quietly, because
the precedent a later slice inherits from a silent fix is the wrong one.

**AND THE LEADING STEP-1 FINDING IS ABOUT THE SOURCE, NOT THE PORT: A MARCH SCOPE CONSUMES ITS
OWN FIELD AND DROPS THE RUNGS ABOVE IT, SO RUNG 67's `assert lag is None` CANNOT SEE RUNG 66's
CARRIER.** The first draft of `slice_z_smoke.rs` armed the seven refusals through
`stator_march_scoped` and could not reach a single one that names another rung's knob. The reason
is structural and it runs both ways:

* a rung-66 `_stator_march` has **no `tau_gov` parameter at all**, so cascade A is unreachable
  from a rung-66 march;
* a rung-67 `_stator_march` forwards `lag` to rung **66's** carrier -- and rung 67's own armed
  branch returns before `super()`, so that carrier is never read.

**MEASURED, not read off the body** (`probe_z10.py`, PyPy). On a rung-67 machine with the valve
floored and BOTH clocks armed through the march: it does **not** raise; `self._lag` holds the
`AsymmetricLag` at the moment the armed branch runs (1 entry, instrumented); and the trajectory is
**bit-for-bit identical, 171/171 points, to one with no `lag` passed at all**. So the fuel lag is
**SILENTLY IGNORED**. The same probe runs the identical question one rung down, where the carrier
IS read, and gets `bit-for-bit equal: False` -- so the zero above is the GRID's and not the
instrument's ([[rust-port-slice-w-step3]]'s rule, applied to my own probe before quoting it).

Python's four rung-67 refusals therefore guard the **direct** `integrate_fuel` route and only
that route; through the march, cascade B on a cascade-A machine is not refused, it is discarded.
The port reproduces this exactly (it reads `lim.lag`, the argument) and the smoke file now takes
the direct route, which is also the route Python's own suites take. **Nothing is repaired** -- a
port is a translation with a bit-exactness contract, and this is a property of rung 67 for its
spec to answer, not for the port to fix.

**THE OTHER FOUR MEASUREMENTS STEP 1 OWED, ALL EMITTED RATHER THAN TYPED:**

| question | answer | how |
|---|---|---|
| does any rung above 67 rebind `_RINGS`? | **no** -- 1 definition, 2 reads, both inside rung 67's own readers, none past 12351 | grep over all 23 066 lines + the 27 suites |
| does rung 66 or 67 add/relax a construction assert? | **no** -- neither defines `__init__`; `_LAG_OK` stays `True` by inheritance | `ast` class-body enumeration |
| does `AsymmetricLag` derive `PartialEq`? | **yes**, so `MarchScope`'s five derives survive the growth | source |
| do the two guards restore PREVIOUS? | **yes, both**, asked per field rather than inherited | probe 2/3, § (iii) |

So `RINGS` ships as a plain `pub const` (a table cell would be the shape if anything rebound it,
and the const's own doc comment says where the port is wrong if a later rung ever does), and the
two builders differ from `build_lagged_bleed` **only in the four table constants they pass** --
stated in both bodies, because a reader who finds two constructors differing by four words should
be told the SAMENESS was measured.

**THE STEP-1 GATE IS A GATE, NOT A COMPILE CHECK.** `slice_z_smoke.rs`'s nine: both rungs reduce
to rung 65 **bit-for-bit on every key of every point**, on a floored machine (171 pts) *and* on an
unfloored one (68 pts, the other branch of the same `if`); `at_lever` hands back this rung's
object, discriminated by giving all three siblings ONE arming and reading which rung's refusal
comes back; the seven refusals; `RINGS`; both carriers travel scope -> `Cell` -> guard -> the
resolving `or_else`, witnessed by reaching the step-2 stub; and **both guards restore through the
unwind their own refusal causes** -- which is free here, because `catch_unwind` refuses a borrowed
core precisely for the interior mutability those carriers are, so the `AssertUnwindSafe` is the
claim and the following bit-for-bit re-march is the check.

**P11 (NEW, for step 5) -- A MANUFACTURED GATE ON THE DISCARD.** The finding above gets an owner
rather than staying an observation in a plan: the port reproduces the silent discard only because
`r67_integrate_fuel` reads `lim.lag` (the argument) and not `ft.inner.lag.get()` (the carrier),
and **that is a one-token change no value key and no current gate would catch** -- every shipped
grid arms at most one of the two. Step 5 owes a manufactured-bug gate that pins it: arm both
clocks through the march on a rung-67 machine, assert the trajectory is bit-for-bit the
governor-only one, and assert the injected carrier-reading spelling BREAKS that. Falsified if the
discard turns out to be reachable by any value key already shipped. The seam itself -- whether
rung 67's refusal SHOULD see the carrier -- belongs to `docs/rung67-spec.md`, not to the port,
which is a translation and not a repair.

**FOUR OF STEP 1's NINE GATES ARE WRITTEN AGAINST THE STUBS AND STEP 2 MUST REWRITE THEM, NOT
ADJUST THEM.** `d_the_lag_carrier_travels...`, `d_the_tau_gov_carrier_travels...` and both halves
of `d_both_guards_restore...` assert on the panic string `"SLICE Z STEP 2"`, which step 2 deletes.
That is scheduled work, booked here so step 2 meets it as a task rather than as breakage --
slice V step 5's lesson (gates that read nothing) in its inverse form: gates that read something
about to be removed. The carrier-travel claim survives the rewrite as *the armed march produces a
trajectory the disarmed one does not*, which is strictly stronger.

**AND THE NEXT RUN RECOVERS THE COUNT FOR FREE:** run the gate as `cargo test 2>&1 | tee <file>`
and aggregate afterwards, rather than piping it through `head` live. The number is then a
by-product of a run that was happening anyway, which is the only way this project permits quoting
one.

**Precedence spelled explicitly, twice:** Python's `lag = lag if lag is not None else self._lag`
means the **ARGUMENT wins over the carrier** and the **RESOLVED** value is what `super()` receives
-- and rung 67 forwards `lag` RAW while resolving only `tau_gov`, because the lag belongs to the
rung below. On every shipped grid at most one of the two is set, so all three of those spellings
agree and no value key separates them.


##### STEP 2 — SHIPPED. **Both marches, and A FIELD A JUNIOR RUNG SILENTLY IGNORES WHERE PYTHON REFUSES THE CALL**

The two ports in full: rung 66's merged four-state march and its four readers, rung 67's cross-loop
march and its six, the six leaf statics (`eig`, `violation`, `window`, `sign_changes`,
`joint_fixed_point`, `exceed`), `detector_sensitivity`, and two new `PointExtra` routes. **20
method bodies, none deferred.** `tests/slice_z_smoke.rs` grows from 9 gates to **19, green in
0.80 s**; step 1's four stub-reading gates are REWRITTEN as booked, not adjusted.

**THE SIZE ESTIMATE WAS LABELLED AS ONE AND IT WAS HIGH.** § 5.24 (ii) carried slice W's 2.06×
across to predict *"near 3 000 lines"*. Measured: `two_lag.rs` **1 114** + `cross_loop.rs`
**1 462** = **2 576** Rust lines against 1 496 Python, a **1.72×** expansion — under slice W's
ratio, not over it. Step 2 itself added 2 140 of those plus 106 lines across three other files.

#### THE PORT WAS COMPARED AGAINST THE SOURCE BEFORE A SINGLE GATE WAS WRITTEN

Every gate step 2 can write is either a reduce arm (which agrees BY DISPATCH) or a
self-referential property of the port, so neither can catch a march that is uniformly wrong.
`probe_z_step2.py` + a throwaway Rust twin therefore emit the same **785 keys** as raw f64 BITS —
both marches at four points each, **all ten readers field-for-field**, the five leaf statics, the
damping ladder at all four of its rungs, and the ringing detector's whole table. **Bit-for-bit
identical vs PyPy on the first run, 0 keys apart.**

Coverage is wider than the four sampled points look: the readers INTEGRATE whole trajectories
(`violation`, `exceed`, `removed_over`) and reduce over every point (`drift`, `track_b`,
`min_phi`), so agreement on `cascade_bill`, `marginal_mode_cascade`, `cross_bill` and
`marginal_mode_cross` is transitively agreement on the trajectories underneath them. The probe is
deleted; step 4 owes the committed oracle at the suites' own grid.

**AND IT REPRODUCED § (i)'s STRIDE FINDING ON A THIRD GRID.** `n_sample = 12` requested delivers
**15** samples on rung 66 and **13** on rung 67 at `ds = 0.01` — the leading finding is not a
property of the row it was found on.

#### (a) THE LEADING FINDING — **A `MarchScope` FIELD IS SILENTLY IGNORED BY EVERY RUNG BELOW ITS OWNER, WHERE PYTHON RAISES `TypeError`; AND IT BIT TWICE, THE SECOND TIME PAST THE WITNESS THAT CAUGHT THE FIRST**

Step 1's finding was that a march scope CONSUMES its own field and drops the rungs above it, so
rung 67's `assert lag is None` cannot see rung 66's carrier. **Step 2 found the same structure
running downward, and this half is the PORT's rather than the source's.**

Python adds these parameters ONE PER RUNG: `LaggedBleedTransient._stator_march` has no `lag`, and
neither it nor `TwoLagCascadeTransient._stator_march` has a `tau_gov`. So `m65._stator_march(...,
lag=…)` is a **`TypeError`**. The port's `MarchScope` is ONE struct shared by every rung — § 5.19
(iv)'s deliberate choice, which opens the cell's signature once instead of four times — so the
same call SUCCEEDS and the field is discarded.

**IT WAS FOUND BY TWO OF THIS STEP'S OWN GATES FAILING, AND THE SECOND FAILURE IS THE INSTRUCTIVE
ONE:**

| draft gate | armed through | bar typed | measured | what it meant |
|---|---|---|---|---|
| `d_the_tau_gov_carrier_travels…` | the march, rung 67 | `route == 14` | **16** | disarmed still means a LAGGED VALVE, so rung 65's marcher runs |
| `e_all_three…reduce_to_rung_52` | the march, rung 65 | `route == 16` | **14** | rung 65 ignored `lag` entirely |
| `e_rung_67_reduces_to_rung_47` | the march, all three | `route == 14` | **14, and the floats still disagreed** | rungs 65/66 marched rung 46's UNLAGGED redline, rung 67 rung 47's lagged one — **and those two routes emit the SAME fourteen keys** |

So three typed count bars, all wrong ([[rust-port-guessed-census-bars]] again), and the third
wrong in a way the bar could not show. **A ROUTE WITNESS IS NOT A RUNG WITNESS**, and the map is
many-to-one in a way `route`'s own first doc comment got wrong too: **14** covers the bare march,
rung 46's unlagged redline, rung 47's lagged one, rungs 48–51's legs AND rung 64's instantaneous
valve; **16** covers rung 52's clip state and rung 65's valve state, which are two DIFFERENT pairs
of keys. Only 20 and 21 name one rung each. Both affected gates now take the DIRECT `integrate_fuel` route, which is the route
Python's own suites take, and the finding is recorded at `MarchScope` rather than in a smoke
comment. **Nothing is repaired** — a narrowed per-rung view would re-open the cell signature four
times, which is the cost the struct exists to avoid, and no shipped Python suite makes such a
call.

#### (b) **ADDING AN ENUM VARIANT BREAKS THE EXHAUSTIVE MATCHES LOUDLY AND LEAVES THE `_ => panic!()` ONES SILENT — AND A SILENT ONE IS A NARROWING**

The two new routes (`Cascade`, 20 keys; `CrossCascade`, 21 — both EMITTED, not typed) forced a
by-hand audit of every `PointExtra` match in `src/` and `tests/`, because the compiler asks about
exactly the arms that were already safe. Rungs 66/67 record `g`, `required`, `b` AND `b_cmd`, and
rung 66's docstring says so as a CLAIM — *"every rung-52 and rung-65 reader works unchanged on
it"* — so a wildcard panic would have made the port **STRICTER THAN PYTHON on a dict that carries
the key**, invisibly, with every suite green. Four arms asked, answers split two–two:

| reader | rung 66/67 dict | verdict |
|---|---|---|
| `valve_of`, `r65_b_at_point` | has `b`, `b_cmd` | **WIDENED** |
| `asym_extra` | has `g`, `required` | **WIDENED** |
| `asym_extra` on `Valve` | no `g` — Python raises | still refuses |
| `valve_of` on `Asym` | no `b` — Python raises | still refuses |

All four wildcards are now spelled as named arms so the NEXT variant breaks the build at each.
`slice_s_smoke.rs`'s key-name table broke as designed and gained both routes' SORTED extra keys.

#### (c) **THE CAUGHT-PANIC ARM FIRES ON NO SHIPPED GRID, SO IT IS EXHIBITED RATHER THAN ASSUMED**

`joint_ic_corners` is the one reader that catches; the port's equivalent is `catch_unwind` plus a
120-**character** truncation, and the doc comment claims characters and bytes agree because the
message is ASCII. Measured: **0 of 8 corners raise** at `ds = 0.01`, and the suite's own call uses
2x2 corners that all converge — so the arm, the truncation and the ASCII claim would have shipped
asserted-in-prose and measured nowhere ([[rust-port-slice-m]]'s shape), and step 4's oracle runs
the suites' grid so it would not reach them either. A `ds` outside the inherited RK4 floor makes
all eight raise inside the catch; `msg_len = 120`, `bytes = 120` and the character-weighted hash
`647 771` were measured on **both** languages and agree, so the constant is an anchor rather than
a Rust-only golden. The all-failed table also exercises Python's vacuous `all([]) == True` and two
`default=` branches § (v) counts as never firing.

**AND THE PANIC HOOK IS DELIBERATELY NOT SUPPRESSED.** Python prints nothing; Rust's default hook
writes one line per caught panic. Quieting it means a process-global `set_hook`, which races the
`take_hook`/`set_hook` pairs the test files already use — a real hazard traded for cosmetic quiet.
**No value differs; only stderr does**, and it is booked here rather than fixed silently.

#### (d) THE FOUR COPY-FROM-THE-RIGHT-ANCESTOR TRAPS, EACH SPELLED AT ITS SITE

1. Rung 66's `required` is **rung 52's** shape (raw `min(caps)`, floor on the RESULT), not rung
   65's (`retain` below `mf_sched`, fall back to `mf_sched`) — different branch structure at the
   `cap == mf_sched` boundary.
2. Rung 66's `der` re-solves `_instant_fuel` **unconditionally** after the redline block; rung
   65's re-solves only when a cap bound.
3. **Rung 67's `der` has no redline min-select at all** — the redline is carried BY the state,
   rung 47's placement, and that IS the reduce detector. A port that copies rung 66's derivative
   compiles, marches, and breaks the rung-47 arm.
4. `command` sits OUTSIDE the `_b_state` scope in both, exactly where Python's call sits after
   the `finally`.

**AND THE TWO JOINT INITIAL CONDITIONS STAY SEPARATE (P6, held).** Rung 66 iterates inline and
undamped, capped at 60; rung 67 calls `joint_fixed_point`'s damped sweep. Both bodies say why the
other one would be bit-exact and wrong.

#### (e) THE PLACEMENT AMBIGUITY, EXECUTED IN BOTH DIRECTIONS

Rung 66's own docstring records that `Tt4_max`'s placement is ambiguous, that nothing would catch
a wrong pick, and that **every shipped rung-66 diagnostic passes `Tt4_max = None`** so it never
runs. Step 2 reaches it through a direct `integrate_fuel` call and asserts the branch BITES (some
point has `mf` strictly below `mf_sched − g`), while the rung-67 march must have `mf` exactly
`max(1e-9, mf_sched − g)` at **every** point — with a liveness check beside it so that second
assertion is not `mf == mf_sched` repeated.

#### (f) PREDICTIONS, AT STEP 2

- **P2 — SIX ARMS ONTO FIVE TARGETS: GATED SIX-FOR-SIX, AND THE FIRST TALLY WAS WRONG BY ONE.**
  The published list called `a_both_rungs_reduce_to_rung_65_on_an_unfloored_machine_too` the
  **rung-64** arm. It is not: that gate builds `LeverArm::default()` — **no limiter at all** —
  which is rung 43/57's machine, where rung 64 IS the floored INSTANTANEOUS valve (a limiter
  present with `tau = None`). Python's own wording, `tau=None and lag=None`, presupposes a limiter
  that HAS a `tau` to be `None`. **Step 1's gate comment hedged it correctly** (*"lands two rungs
  lower (rung 64 / rung 43)"*) **and step 2 hardened the loose half into a COUNT** — the shape
  the commit before it was caught on, one predicate later. The uncovered combination is FLOORED +
  UNLAGGED + NO CLOCK, and nothing else in the slice reaches it: every floored fixture here is
  lagged, `e_all_three…_rung_52` builds the right machine and then arms `lag`, and the 785-key
  probe ran floored-lagged machines plus two unfloored bill cells. That path runs `r64_solve_b`,
  `ForcedBleed` and `b_of` at every closure, all of which the valveless path skips. **A 19th gate
  now covers it**, and its own `14` bar is weak — an `assert_ne!` against a valveless machine is
  what discriminates. The six: 66→65, 67→65 (floored, lagged); 66/67→64 (floored,
  INSTANTANEOUS); 66→52; 67→66; 67→47.
- **P6 — HELD.** The two IC solves are separate, and each body states the other's failure mode.
- **P9 — MEASURED, NOT MAPPED.** The whole smoke file is **0.80 s** at 19 gates; no `slow`
  marker is introduced.
- **P4/P5/P7/P11 — still step 5's**, unchanged. § (v)'s five dead arms, `_eig`'s complex arm, the
  two manufactured nests and the carrier-discard gate are NOT anticipated here.
- **P3/P8 — still step 4's.** The CPython exemption is spelled at `cross_identity` with the naive
  LEFT fold that makes it PyPy's, and the probe did not run the CPython arm.

#### (g) THE GATE, AND THE COUNT IS A BY-PRODUCT OF IT

`cargo test`, piped through `tee` rather than through `head` — which is step 1's own booked
repair, since that step destroyed its aggregate by piping a live run into `sort | uniq -c | head`.
**Exit 0. 115 targets, 1 103 tests passed, 0 failed, 0 ignored**, `slice_z_smoke` 18/18 in 1.06 s.
One doc comment was added to `cross_loop.rs` after the run started; it changes no executable line,
and `cargo build --lib` plus a re-run of `slice_z_smoke` (18/18) confirm it.

`cargo clippy --all-targets` reports **one error and it is the SAME pre-existing one** step 1
verified (`eq_op` on a deliberate NaN test) — it now reads `stator_transient.rs:2633` rather than
`:2610` because this step inserted a doc comment above it, not because a second appeared. Zero new
warnings from either new module.


##### STEP 3 — SHIPPED. **The 38 ported gates, and A CENSUS WHOSE BASELINE READ ZERO**

`tests/rung66.rs` (**15** gates) and `tests/rung67.rs` (**23**), Python's two suites gate for gate,
in their order and under their names. **Both green on the first run**, 15 in **1.26 s** and 23 in
**3.30 s** — **4.56 s against PyPy's 91.07 s, a measured 20.0x** (§ 5.24 (ix)'s prior was slice Y's
16x). **P9 held: no `slow` marker and no `#[ignore]` is introduced**, because the measured Rust
cost does not warrant one.

#### (a) THE LEADING FINDING — **THE INJECTION HARNESS REPORTED A `0 passed / 0 failed` BASELINE AND CALLED ALL EIGHT INJECTIONS "0 red"**

The census ran once and returned a clean, plausible, entirely worthless table: every injection
invisible, including the two CONTROLS chosen precisely because a ported gate exists for them.

**The cause is a two-stream parse.** `cargo test` prints `Running tests\<name>.rs` on **stderr**
and the test harness prints `test result: ok. 15 passed; 0 failed` on **stdout**. The first
version ran all three targets in ONE invocation and parsed `p.stdout + p.stderr`, which puts
*every* result line BEFORE the target line that names it — so the parser's "which target am I in"
variable was `None` for all of them, each target was recorded `[0, 0, []]`, and `sum(failed) == 0`
for every run including the controls.

**IT WAS CAUGHT BY ONE PRINTED LINE AND NOTHING ELSE.** The harness echoes its baseline, and
`{'rung66': (0, 0), ...}` beside a suite that had just been watched go 15/15 and 23/23 is the only
thing in the run that disagreed with itself. Had the baseline not been printed, eight rows of
*"no gate can see this"* would have gone into a write-up. [[rust-port-slice-w-step3]] is the same
lesson one level out — *make the instrument prove it can SEE* — and the repair is a bar rather
than a fix: the harness now runs ONE target per invocation, parses stdout alone, and **refuses to
proceed unless the baseline reads exactly `{rung66: 15, rung67: 23, slice_z_smoke: 19}`.**
A census on a broken parser measures nothing, and the only defence that scales is a counted
baseline the run must clear before it is allowed to conclude anything.

#### (b) THE CENSUS, EIGHT INJECTIONS, EACH RUN TWICE

Every injection is run in two variants, which is the three-column discipline four slices running
have needed: a **LIVENESS** build (the same edit plus a `panic!` marker at the site, or one
conditioned on the state the injection depends on) that answers *did it apply and is the site
reached*, and a **SEMANTIC** build (the edit alone) that answers *what can the gates SEE*. `red` is
over all **62** gates in the slice — 15 + 23 ported plus `slice_z_smoke`'s 19.

| # | the injection | liveness | semantic `red` | verdict |
|---|---|---|---|---|
| I1 | `eig` drops its COMPLEX arm | marker fires, **5 red — all rung 67, ZERO rung 66** | **2** (`the_window_is_log_symmetric…`, `the_sum_bound_is_measured_conservative…`) | **§ 5.24 (vi) / P5 CONFIRMED BY MEASUREMENT, and the arm is NOT unguarded** |
| I2 | rung 66's INLINE UNDAMPED joint IC routed through rung 67's DAMPED solver | marker fires, 17 red — the site is reached everywhere | **0** | **P6's blind spot, exactly as pre-registered** → step 5 |
| I3 | rung 67's `assert lag is None` reads the CARRIER | **marker NEVER FIRES** | **0** | **PROVABLY invisible**, not merely unnoticed → P11, step 5 |
| I4 | the `_lag` guard restores `None` not PREVIOUS | **marker NEVER FIRES** (`prev` is never `Some`) | **0** | **PROVABLY invisible** → P7a, step 5 |
| I5 | the `_tau_gov` guard restores `None` not PREVIOUS | **marker NEVER FIRES** | **0** | **PROVABLY invisible** → P7b, step 5 |
| I6 | `window`'s `zeta` re-spelled `sqrt(1/(1+abs P))` | site trivially live (two gates read `zeta`) | **0** | a REAL 1-ulp change nothing sees → **step 4's**, § (d) |
| I7 | `exceed` copies rung 66's break — **CONTROL** | marker fires, 5 red | **2** incl. `the_exceedance_integral_does_not_drop_its_final_cell` | **the instrument CAN see** |
| I8 | `violation` adopts `exceed`'s LOWER guard — **CONTROL** | marker fires, 7 red | **0** | **the prediction was WRONG** — § (c) |

**THE LIVENESS COLUMN IS WHERE THREE OF THE FOUR ZEROS STOP BEING GUESSES.** I3, I4 and I5 do not
merely go unnoticed: their markers never fire at all, so `ft.inner.lag.get()` is never `Some` where
rung 67's refusal reads it and neither guard's `prev` is ever `Some` anywhere in the slice. Probe
3's *max nesting depth 1, 0 nested events* is thereby re-measured in the PORT, on the port's own
grid, rather than inherited from a Python reading — and the four manufactured gates step 5 owes are
owed against a hole that has now been shown to exist rather than argued to.

**AND I1's LIVENESS RUN IS THE STRONGEST SINGLE ROW.** Five gates go red when the complex arm
panics, and **every one of them is in `rung67.rs`; not one is in `rung66.rs`.** That is § 5.24
(vi)'s *"80 of 80 real, and the complex arm never runs on rung 66 at all"* re-derived from the
Rust, by a different instrument, on the ported grid. **P5's premise therefore holds and its
conclusion needs a qualification the pre-registration did not have**: the arm is dead on rung 66
but it is *not* an unguarded blind spot, because two rung-67 gates catch a port that drops it. Step
5's manufactured gate is still owed — it names the arm directly instead of catching it through an
emergent count — but it is hardening a covered site, not closing a hole.

#### (c) **I8 — THE DROPPED CELL IS NOT SMALL, IT IS EXACTLY ZERO, AND BY THE CLAMP RATHER THAN BY DECAY**

I8 was written as a CONTROL — *"MUST redden; rung 66's currency is gated on the un-repaired
integral"* — and **0 of 62 moved.** Rather than reason about why, a throwaway Rust probe computed
`violation` both ways on the shipped cascade trajectory at all three `ds`:

| `ds` | first `s > 0.5` | that `s` | area the dropped cell would add | `phi_lim − phi_lp` there |
|---|---|---|---|---|
| 0.01 | index 50 | `5.00000000000000222e-1` | **exactly `0.0`** | `−5.256501e-3` |
| 0.005 | index 100 | `5.00000000000000333e-1` | **exactly `0.0`** | `−5.256604e-3` |
| 0.0025 | index 200 | `5.00000000000000333e-1` | **exactly `0.0`** | `−5.256628e-3` |

So **both halves of the source's own account are true and its adjective is an understatement.**
The accumulated `s` really does land a float's width past `s_hi`, so the straddling cell really is
real and really is dropped — the condition `violation`'s doc comment names, exhibited for the first
time. But the reason it does not matter is not that the integrand has *decayed* to ~0 by `s = r`:
`phi_lp` has recovered **above** `phi_lim` by then, so `max(0, phi_lim − phi_lp)` is **clamped to
zero at both ends of the cell** and the two spellings agree **bit-for-bit**, at every grid.

Two consequences, and the second is step 5's: the *"two functions, never one with a flag"* rule
that keeps `violation` and `exceed` apart is protected by **nothing at all** on the shipped grid;
and **no oracle at the suites' own grid can reach it either**, because the difference is identically
zero rather than small. It is therefore NOT a step-4 hand-off. Step 5 owes it a gate on a
CONSTRUCTED trajectory whose integrand is non-zero at `s_hi`, which is the only place the
distinction exists.

#### (d) **I6 — A ONE-ULP RE-SPELLING NOTHING SEES, AND IT FALSIFIED A CLAIM IN THE PORTED FILE'S OWN DOC COMMENT**

`window`'s `zeta` is `1.0 / (1.0 + |P|).sqrt()`. The injection is `(1.0 / (1.0 + |P|)).sqrt()` —
algebraically identical, and `cross_loop.rs`'s module doc already records that the two are **not**
bit-equal and that the spelling is not a free choice. Measured over every `P` the two window gates
evaluate: **5 of 8 differ in the last bit**, including the plant's own
`P_mid = −2.0388646020554284e-2` and the `P → 1` limit that recovers rung 66. **Zero of 62 gates
move.**

The reason is the ported gate's own bar. `the_window_formula_recovers_rung66_as_its_p_to_one_limit`
asserts `|zeta − 1/sqrt(1+|P|)| < 1e-15` against a gap of ~`1.1e-16`. **I had written into that
gate's doc comment that the line "can only catch a port that changed the SPELLING"** — the
concession offered in place of admitting it is a self-comparison. It cannot catch that either, and
the comment is corrected in place rather than left standing. [[rust-port-ported-test-vacuity]]'s
harder form: **a gate labelled as PARTLY vacuous, whose surviving half was also vacuous.**

This one IS the oracle's — a bit-exactness dump over `cross_identity`'s published `zeta` /
`T_over_tau` is exactly the instrument for a 1-ulp spelling, and § 5.24 (i)'s exemption block
already puts both keys under a microscope. Booked to step 4.

#### (e) THE THREE GATES THE ADVISOR FLAGGED AS POSSIBLE SELF-COMPARISONS, ASKED ONE BY ONE

*Does the Rust compute both sides independently, or does one side derive from the other?*

| gate | verdict |
|---|---|
| `the_cross_gains_are_reciprocals` | **INDEPENDENT.** `R_q` is a central difference of `try_sched_fuel`/`try_surge_fuel` in the valve POSITION; `C_g` one of `r64_solve_b` in the fuel CLIP. Two closures, two step sizes, neither reading the other — so `prod == 1` measures the identity. Stated at the gate. |
| `the_eigenvalues_are_real_and_the_rates_add` | **HALF TAUTOLOGY, LABELLED.** `det ≡ 0` makes the discriminant `tr² ≥ 0` identically, so `all_real` cannot fail unless `det` is wrong. `rho_err` — a measured eigenvalue against a closed form in the two clocks alone — is the half with content. |
| `the_window_formula_recovers_rung66…` | **SELF-COMPARISON, AND WEAKER THAN THAT** — § (d). The reciprocal identity, the closed branch and the two monotonicities are what it does pin. |

#### (f) PREDICTIONS, AT STEP 3

- **P9 — MEASURED, NOT MAPPED.** 4.56 s for 38 gates; no marker introduced.
- **P4/P5 — SHARPENED BY MEASUREMENT.** § 5.24 (v)'s five dead arms are still step 5's, and I1
  re-measures P5's premise in the Rust while correcting its framing (§ (b)).
- **P6/P7/P11 — CONFIRMED AS HOLES, BY LIVENESS AND NOT BY SILENCE** (I2/I3/I4/I5). Step 5's.
- **P12 (NEW, for step 5) — `violation`'s DROPPED CELL.** A gate on a constructed trajectory whose
  integrand is non-zero at `s_hi`, because § (c) shows no shipped grid and no oracle can reach it.
  Falsified if some shipped key turns out to separate the two spellings after all.
- **P3/P8 — still step 4's**, and I6 adds a named third thing for the dump to catch (§ (d)).

#### (g) THE GATE, AND THE COUNT IS A BY-PRODUCT OF IT

`cargo test 2>&1 | tee <file>`, step 2's booked idiom, so the aggregate is a by-product of a run
that was happening anyway. **Exit 0. 118 result blocks — 116 integration targets, the lib unit
tests and the doc-tests — 1 142 passed, 0 failed, 0 ignored.** Step 2 reported *115 targets, 1 103
tests*; the delta is **+2 targets** (`rung66.rs`, `rung67.rs`) and **+39 tests** (their 38, plus
the 19th `slice_z_smoke` gate the follow-up commit added after step 2's number was taken), and the
target arithmetic reconciles once the counting convention is stated rather than assumed: 117
`Running` lines (116 integration + 1 `unittests`) plus one doc-test block. **The run does NOT
include `slice_z_oracle.rs`**, which was written after it started — step 4 carries its own number.


##### STEP 4 — SHIPPED. **35 335 keys on both interpreters, and AN EXEMPTION THAT WAS TWO KEYS AND IS EIGHT**

`rust/oracle/dump_slice_z.py` + `tests/slice_z_oracle.rs`, fourteen sections A–N over both rungs.
**35 335 keys, BIT-EXACT vs PyPy on the first run**, and vs **CPython 3.14** with exactly the
declared exemption below. The Rust arm runs in **16.85 s**; the dump takes **70 s** per
interpreter.

**P8 HELD, AND BY NOT NEEDING ANYTHING.** Every argument is copied from the calling gate, never
chosen — including the two places the suites NARROW a default (`oscillation_window`'s three `rhos`
rather than the seven-wide default, `joint_ic_corners`' 2x2 corners rather than 4x2), because the
oracle mirrors GATES and a default the suites do not use is a grid they do not have. § 5.24
(viii)'s 33.31 s timing is what made that free, and the header states the grid rather than
implying it.

#### (a) THE LEADING FINDING — **P3's "TWO KEYS" WAS A COUNT OF QUANTITIES, AND THE DUMP EMITS NAMES**

§ 5.24 (i) pre-registered the CPython exemption as *"a NAMED, COUNTED pair on one row — not a
tolerance tier"*, and named `P_mid` and the `T_over_tau` it feeds. Diffed over the two arms, the
measured set is **EIGHT key names**:

```text
F/rows/1/P_mid          G/P            K/window/7/P          N/0/P_mid
F/rows/1/T_over_tau     G/window/P     K/window/7/T_over_tau
                        G/window/T_over_tau
```

Every one is `P_mid` itself or the `T_over_tau` it feeds — **so P3 holds at its intent and its
arithmetic does not.** `P_mid` is re-published under four further names: `oscillation_window`
reads it as `P` and again inside its `window` sub-dict, section K re-evaluates `_window` at it, and
section N recomputes it on a second grid. **An exempt list transcribed from § 5.24 (i) would have
carried two entries and this oracle would have failed on six more** — the same shape as the slice's
own leading finding, one level out: *a number written down once reads like a measurement.*

`zeta` does **not** move at any of the eight, which confirms § 5.24 (i)'s propagation table
(`P` 1 ulp, `T_over_tau` 1 ulp, the other five keys 0) on a second instrument and a wider grid.

**AND THE LIST IS CHECKED IN BOTH DIRECTIONS.** `Cmp::finish` asserts the exempted set EQUALS the
eight names — a ninth key fails the run, and so does one of the eight *ceasing* to drift, because
an exemption nobody re-measures is a tolerance with better manners.

#### (b) **THE STRIDE FINDING RECURS ON TWO ROWS OF ONE TRAJECTORY**

§ 5.24 (i) is that `sub = ride[::max(1, len(ride)//n_sample)]` delivers a count that is not the
requested one, and that a probe reading the request measures a different function. The dump EMITS
`n_ride` and the delivered `n_sample` at every `cross_identity` row it takes, so nothing here
inherits the 9:

| section | clock | `n_ride` | requested | **DELIVERED** |
|---|---|---|---|---|
| F | `tau_gov` 0.005 / 0.05 / 0.5 at `ds` 0.005 | 135 / 97 / 91 | 8 | **9 / 9 / 9** |
| N | `tau_gov` 0.05 at `ds` 0.01 / 0.005 / 0.0025 | 49 / 97 / 195 | 6 | **7 / 7 / 7** |

`len(ride) = 135 / 97 / 91` reproduces § 5.24 (i)'s measurement exactly. And the two arms overlap
on one row: **`F/rows/1` and `N/1` are the same clock on the same grid** — `tau_gov = 0.05`,
`ds = 0.005`, `n_ride = 97` — sampled **9** wide and **7** wide. **The 9-wide one diverges from
CPython and the 7-wide one does not.** The chunk width decides the answer, on one trajectory,
inside one file — which is the slice's leading finding exhibited rather than argued.

#### (c) THE INJECTION CENSUS, RE-RUN WITH THE ORACLE AS A FOURTH TARGET

Step 3's eight injections, over **64** gates now (15 + 23 + 19 + the oracle's 2):

| # | what it breaks | ported gates | smoke | **ORACLE** |
|---|---|---|---|---|
| I1 | `eig` drops its complex arm | 2 (rung 67 only) | — | **BOTH ARMS** |
| I2 | rung 66's IC routed through rung 67's damped solver (**P6**) | — | — | **—** |
| I3 | rung 67's refusal reads the CARRIER (**P11**) | — | — | **—** |
| I4 | the `_lag` guard restores `None` (**P7a**) | — | — | **—** |
| I5 | the `_tau_gov` guard restores `None` (**P7b**) | — | — | **—** |
| I6 | `window`'s `zeta` re-spelled | — | — | **BOTH ARMS** |
| I7 | `exceed` drops the straddling cell | 2 | — | **BOTH ARMS** |
| I8 | `violation` folded into `exceed`'s guard | — | — | **BOTH ARMS** |

**THE ORACLE CLOSES TWO OF STEP 3's FOUR BLIND SPOTS AND ONE OF THEM RETIRES A PREDICTION I MADE
ONE STEP EARLIER.** Step 3 booked **P12** — *"`violation`'s dropped cell needs a step-5 gate on a
constructed trajectory, because no shipped grid and no oracle can reach it"*. Section K runs BOTH
upper limits on a synthetic ramp whose integrand is non-zero at `s_hi`, which is exactly that
constructed trajectory, so **P12 is DISCHARGED HERE rather than owed to step 5.** The reasoning
behind it stands — on every shipped march the two spellings agree bit-for-bit — and the conclusion
about WHERE the gate belongs was wrong by one step. Recorded rather than quietly dropped.

I6 likewise: step 3 handed the `zeta` spelling to step 4 and step 4 catches it, on both arms,
because section K sweeps `_window` at all eight `P` values including the plant's own.

**THE FOUR THAT SURVIVE ARE EXACTLY THE FOUR THE PRE-REGISTRATION NAMED** — P6, P11, P7a, P7b —
and three of them are *provably* invisible rather than merely unnoticed, because their liveness
markers never fire anywhere in the slice. Step 5 has a measured list, not an argued one.

#### (d) TWO THINGS THE DUMP EMITS THAT NO GATE ASKED FOR, AND WHY

* **`E/cascade/key_count` and `L/cross/key_count`, read off the LIVE dict.** `FuelPoint::key_count`
  returns 20 and 21 from a `match`, and its own doc comment says this line is what checks them —
  [[rust-port-guessed-census-bars]], applied to a constant the port would otherwise assert about
  itself.
* **`G/n_skipped` and `J/n_failed`.** Both are **0** on this grid, and a zero left un-emitted is
  what [[rust-port-slice-w-step3]] is about: `J/all_converged` is Python's `all(...)` over the OK
  rows and is vacuously `True` if every corner failed, so the count of failures rides beside it.

#### (e) PREDICTIONS, AT STEP 4

- **P3 — HELD AT ITS INTENT, ARITHMETIC CORRECTED** (§ (a)). Two quantities, eight names, no
  tolerance tier anywhere.
- **P8 — HELD, by not needing anything** (§ above).
- **P12 — DISCHARGED AT STEP 4, NOT STEP 5** (§ (c)).
- **P4/P5/P6/P7/P11 — step 5's, now against a MEASURED blind-spot list of exactly four** (§ (c)).


##### STEP 5 — SHIPPED. **SLICE Z CLOSES. Eight manufactured gates, and MUTATION FOUND TWO VACUITIES IN A ROW IN ONE OF THEM**

`tests/slice_z_dispatch.rs` — **8 gates, 1.22 s**, plus a `CensusZ` counter pair in `src` (39 lines
in `two_lag.rs`, 1 in `cross_loop.rs`). **Nothing in the file reads a golden**: every assertion is
a counter, a same-run difference between two dispatch arms, or a property of a shipped function at
arguments the file chooses, so regenerating `slice_z_pypy.tsv` cannot make one of them pass or
fail. That is slice V step 5's lesson applied as a design rule rather than discovered again.

**THE LIST IS MEASURED.** Step 3's eight injections, re-run at step 4 with the oracle as a fourth
target, leave **exactly four** survivors across all 64 gates and all 35 335 keys — and they are
exactly the four § 5.24 pre-registered: **P6** (I2), **P11** (I3), **P7a** (I4), **P7b** (I5).
Three of the four are *provably* invisible rather than merely unnoticed, because their liveness
markers never fire: `ft.inner.lag` is never `Some` where rung 67's refusal would read it, and
neither guard's `prev` is ever `Some` anywhere in the slice. Probe 3's *max nesting depth 1, 0
nested events* is thereby re-measured in the PORT rather than inherited.

#### (a) THE LEADING FINDING — **A GATE ADDED TO REPAIR ONE VACUITY IMMEDIATELY EXPOSED A SECOND ONE IN THE SAME GATE**

Mutation MU9 broke `gains`'s empty-`caps` fall-through from `return 0.0` to `return 1.0` and
`p4_the_two_dead_gains_arms…` **stayed green**. The reason is arithmetic and it is general:
**`gains` hands back a CENTRAL DIFFERENCE, and the central difference of ANY constant is zero.**
So `assert_eq!(r_none, 0.0)` proves the empty arm returns a CONSTANT and never that the constant
is the literal `0.0` — the gate's name claimed the literal. `gains` exposes only the derivative, so
the literal is out of reach from there; the claim is re-stated at what it covers.

**AND THE BAR ADDED TO KEEP THAT HONEST FAILED ON ITS FIRST RUN, WHICH IS THE FINDING.** The repair
was `assert!(r_accel != 0.0)` — *if the accel arm is dormant too, `r_none == 0` is a property of
the DIFFERENCING and not of the empty-caps branch.* It measured **`r_accel = 0` exactly.** The
exhibit was built with `accel_schedule(..., margin = 1.10, ...)`, a cap 10 % above the steady line,
which at a mid-ramp point sits ABOVE `mf_sched`; `max(0, mf_sched − cap)` then clamps to zero on
BOTH sides of the difference. **So the accel arm — one of § 5.24 (v)'s five dead arms, gated here
precisely to exhibit it — was being exhibited DORMANT**, and the only assertion covering it,
`assert_ne!(r_accel, r_surge)`, passed for the single reason that `r_surge` is non-zero.

`margin = 0.0` is Python's own *"never exceed the steady fuel/pressure ratio"*, it binds on an
accelerating ramp, and with it the arm is live and the gate is about the arm. **Two vacuities, one
gate, found in immediate succession — the second by the bar written for the first.**
[[rust-port-ported-test-vacuity]] and slice W step 5's *assert the exact delta* meet here: a
did-it-move bar has to be checked that it CAN fire, or it is one more thing that reads as coverage.

#### (b) THE MUTATION TABLE — ELEVEN ROWS, EIGHT CAUGHT, THREE SURVIVORS AND ALL THREE EXPLAINED

| # | mutation | verdict |
|---|---|---|
| MU1 | rung 66's IC routed through the damped solver | **CAUGHT** (`p6_rung_66_never_reaches…`) |
| MU2 | rung 67's refusal reads the CARRIER | **CAUGHT** (`p11…`) |
| MU3 | the `_lag` guard restores `None` | **CAUGHT** (`p7a…`) |
| MU4 | the `_tau_gov` guard restores `None` | **CAUGHT** (`p7b…`) |
| MU5 | `eig` drops its complex arm | **CAUGHT** (`p5…`) |
| MU6 | `window`'s `P == 0` guard DELETED | **SURVIVES** — § (c) |
| MU7 | `sign_changes`'s `peak <= 0` guard DELETED | **SURVIVES** — § (c) |
| MU8 | `violation`'s upper-limit break deleted | **CAUGHT** (`p4…leaf`) |
| MU9 | `gains`'s empty-`caps` returns `1.0` | **SURVIVES** — § (a) |
| MU10 | `gains`'s valve law made to READ the fuel legs | **CAUGHT** (`p4…gains`) |
| MU11 | `joint_fixed_point`'s damping ladder truncated to `w = 1` | **CAUGHT** (`p6_the_damped_sweep…`) |

**MU10's FIRST VERSION WAS A VOID MUTATION AND WAS RE-RUN.** It bound an unused `_leak` local, which
changes no value, so its "survives" said nothing about the gate. Re-spelled to actually perturb
`big_c`'s applied fuel, it is caught. A mutation harness is code too, and a mutation that does not
mutate is the same defect one level out (slice V step 5: *a gate that MANUFACTURES a bug is code —
inject its own wrapper*).

#### (c) **TWO OF § 5.24 (v)'s FIVE "DEAD ARMS" ARE NOT DEAD, THEY ARE UNOBSERVABLE — AND NO GATE CAN FIX THAT**

MU6 and MU7 are not gate defects. Deleting the branches they name changes **no output on any
input**:

* **`window`'s `P == 0` guard.** Rust's `2π / 0.0.sqrt()` is already `+inf`, which is exactly what
  the guard writes out. The branch is load-bearing in **Python**, where float division by zero
  RAISES, and value-inert in the port.
* **`sign_changes`'s `peak <= 0` early return.** With `peak = 0` the floor is `0.0`; `x.abs() < 0.0`
  is never true and `prev != 0.0` is never true, so both spellings return `0`. Inert in **both**
  languages — the guard is defensive, not functional.

§ 5.24 (v) called five arms *dead*, meaning UNEXERCISED on the shipped grid. Two of them are
something stronger and different: **unobservable**. No manufactured gate can pin their existence,
because a gate observes values; only a counter could, and a counter on a branch that cannot change
a value is testing the source text rather than the program. The two assertions therefore claim the
VALUE Python produces there — which is the port's actual contract — and say so, instead of reading
as branch coverage. **A pre-registration that classifies arms by REACHABILITY has a second axis it
did not ask about, and slice AA's should ask it: can this branch change a value at all?**

#### (d) THE EIGHT GATES

* **P6, in two halves.** `p6_rung_66_never_reaches_the_damped_solver_and_rung_67_always_does`
  counts which solver ran (`CensusZ`), because *nothing else can tell* — I2 moved 0 of 64 gates and
  0 of 35 335 keys. **Its own first draft was one-sided** and mutation of the gate's shape caught
  that: `jfp_calls == 0` passes if rung 66's march never runs at all, so `r66_inline_ic > 0` is the
  liveness half and the rung-67 arm's `r66_inline_ic == 0` is the mirror (a port that solved the IC
  twice would pass a one-sided version). `p6_the_damped_sweep_converges_where_rung_66s_undamped_loop_cannot`
  is why that is worth counting: at `|P| = 5` the ladder converges on `w = 1/4` and the undamped
  recurrence does not, so the forbidden substitution turns rung 66's *"this IS the degeneracy
  locus"* assert into a silent success.
* **P11**, with the witness that makes it mean anything. Arming both clocks through a rung-67 march
  is bit-for-bit the governor-only march (the lag is DISCARDED, as Python discards it); and the
  same scope→carrier channel run ONE RUNG DOWN moves the marcher from 16 keys to 20 — so the zero
  above is the RUNG's and not a dead channel. [[rust-port-slice-v-step4]] in its own shape.
* **P7a / P7b**, two manufactured nests, each with a REAL march as the inner scope so the SHIPPED
  guard is nested through the SHIPPED cell, and asked separately per field.
* **P5**, `eig`'s complex arm by direct call, beside the half that makes it a statement about rung
  66: under `R_q·C_g ≡ 1` the discriminant is `tr²` **exactly** (the reciprocal pairs are exact in
  binary, so `det == 0.0` is an equality and not a tolerance) at every clock pair in a 5x4 sweep.
* **P4**, the five arms — the two `gains` ones on their arguments (§ (a)), the three leaf ones
  beside their live siblings (§ (c)). The `gains` gate also pins, bit for bit, that **`C_g` is the
  same number on all three arms**: `big_c` roots `r64_solve_b` at the applied fuel and never looks
  at `caps`, so *"neither closure knows the other exists"* becomes an equality rather than prose.
  MU10 is what proves that assertion has teeth.

#### (e) PREDICTIONS, AT STEP 5 — **THE SLICE'S LEDGER**

- **P1** — falsified at its letter, held at its intent, bill paid once (step 1).
- **P2** — six arms onto five targets, gated six-for-six after the tally was caught wrong by one.
- **P3** — held at its intent; **its arithmetic corrected from 2 keys to 8 names** (step 4 § (a)).
- **P4** — the five arms gated; **two of them reclassified from dead to unobservable** (§ (c)).
- **P5** — held; its *"unguarded"* framing corrected by step 3's I1 (two rung-67 gates catch it).
- **P6** — held, and now **counted** rather than argued.
- **P7** — held, both halves, by two manufactured nests.
- **P8** — held, by not needing anything.
- **P9** — measured, never mapped: 4.56 s for the 38 ported gates, 1.22 s for these 8.
- **P10** — five steps, as planned.
- **P11** — held; the discard is gated with a one-rung-down witness.
- **P12** — raised at step 3 and **discharged at step 4**, one step earlier than it was booked.

#### (f) THE GATE, AND THE SLICE'S CLOSING NUMBERS

`cargo test 2>&1 | tee <file>`. **Exit 0. 120 result blocks, 1 152 passed, 0 failed, 0 ignored.**
The arithmetic reconciles exactly against step 3's 118 / 1 142: `slice_z_oracle.rs` adds one target
and 2 tests, `slice_z_dispatch.rs` one target and 8.

`cargo clippy --all-targets` reports **one error and it is the SAME pre-existing one** steps 1 and
2 each verified — `eq_op` on a deliberate NaN test — now at `stator_transient.rs:2639` rather than
`:2633`, because this slice inserted lines above it and not because a second appeared. **Zero
clippy findings of any kind in the four files this slice added.**

Slice Z's totals: **2 615 Rust source lines** (`two_lag.rs` 1 114 + `cross_loop.rs` 1 462 + 39 for
`CensusZ`) against 1 496 Python — **1.75x**, still under slice W's 2.06x — plus **67 test
functions**: `slice_z_smoke.rs` 19 + `rung66.rs` 15 + `rung67.rs` 23 + `slice_z_dispatch.rs` 8 +
`slice_z_oracle.rs` 2, over **35 335 oracle keys on two interpreters.** (The first draft of this
line read *"57 gates"* — a tally TYPED rather than added up, which is the thing this phase has now
been caught on four times. It is 19 + 15 + 23 + 8 + 2.)

### ~~The four~~ **THE EIGHT** runtime-introspection tests, one by one

**CORRECTED 2026-08-20 by § 5.19 (vii) — this table named FOUR and an enumeration over the 27
phase-7 suites finds EIGHT.** The four it missed are `test_rung71.py:243`, `test_rung73.py:492`
(both `include_str!`), `test_rung71.py:190` (a `__dict__`-absence plus a same-function-object
identity — which becomes **fn-pointer equality on two tables' cells**, structurally stronger) and
`test_rung79.py:133`, whose assertion **cannot fail** and is recorded rather than replaced
(§ 5.19 (ix)). Slice K's lesson, on this table: nobody had counted.

| test | what it asserts | replacement |
|---|---|---|
| `test_rung71.py:241`, `test_rung73.py:477`, `test_rung72.py:414` | a parameter (`s_off`, `tau_rel`) is **absent** from a method's signature | Pass that hook a **narrowed config view** — a struct holding only the fields it may read. "Not reachable" becomes a compile error: strictly stronger than the test. ~~**Cost, decided in phase 7 not discovered in it:** a hook taking a narrowed view cannot share a `Hooks` field type with one taking `&Config`, so either the table carries per-hook parameter types (fine — just more struct) or these three fall back to `include_str!` like the row below.~~ **DECIDED 2026-08-20 at the pre-flight, and the cost is ZERO — § 5.19 (iv).** Phase 7 needs a third hook parameter anyway, for a reason this row could not have known: **23 fields are dynamically scoped** through `try/finally` guards, **9 of them the current RK4 state**. Seven cells take that `Scope` struct and `_stator_march` is one of them; it simply does not carry `s_off`/`tau_rel`, so the narrowed view IS a parameter that cell already takes — no per-hook parameter types, and no `include_str!` fallback. **One cost, not zero: `try_close` is a SHIPPED phase-6 cell that also takes it, so slice V changes a gated signature — § 5.19 (iv).** |
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
   **Superseded 2026-08-17: PHASE 6 was authorised**, its pre-flight ran first (§ 5.12), and all
   six slices P–U shipped; **PHASE 6 IS COMPLETE 2026-08-20.**
   **Superseded again 2026-08-20, and the phase-5 two-step is repeated exactly.** The re-decide
   point before phase 7 was reached and **only the PRE-FLIGHT was authorised** ("start phase 7
   preflight"). It is done and landed at **§ 5.19**, and it did what phase 5's did: the named
   deliverable (§ 2's eight-hook table) turned out to be the hypothesis, not the finding — the
   enumeration returns **38 names**, refutes § 3's *"~8–10 methods"* at its own lower bound, and
   finds **two structural shapes § 2 does not know about** (the 16 pinned `super()` sites and the
   23 dynamically-scoped fields behind 52 guards). **PHASE 7 ITSELF IS NOT AUTHORISED** and no line is ported
   until it is. On the evidence, the two things to weigh in that decision are the **sizing** — 15–20
   sessions, four times phase 6 — and the **gate**, which the phase table understated (§ 5.19 (viii)).

**Decision 1 is REVISED by § 4.2**: phases 0–2 are held to bit-equality, not to a tolerance,
because it was measured achievable (100 % on both oracles) and because a tolerance bar let a
real defect ride for a whole phase. Later phases may fall back to Option B, with the deviation
distribution published here.

### Consequences for the phase table

Phase 8's `main.py` row is now "Rust CLI prints the tables and dumps plot JSON; port the
chart script; verify it is fast" rather than a three-way choice.
