# The Rust port — plan

**Status: DECIDED — phases 0–1 AUTHORISED, the rest is not.** The architecture is settled by
measurement (§ 1–2); the three open forks were answered on 2026-08-12 (§ 9). Build phases 0
and 1, then stop and re-decide: phase 1 is where the arithmetic risk concentrates, so it is
the deliberate early exit.

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

### The rungs where a tolerance is NOT a valid substitute

The finding is a **count**, and a count jumps discontinuously:

| rung | the claim | why a tolerance does not cover it |
|---|---|---|
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
| **0** | Cargo workspace; the oracle bridge (Python dumps JSON at named probes, a Rust test reads and compares); the per-quantity tolerance policy | 1 | bridge round-trips on a design run |
| **1** | `gas.rs` — `FlowState`, dual-section `Gas`, CPG closed form, TPG NASA integrals, reacting, Fork B, equilibrium Newton (rungs 1–6) | 3–5 | design-point station table matches oracle |
| **2** | `components.rs` + `engine.rs` design point — shaft balance, `_score`; conservation checks as `assert!` (they run on **every** execution, per the working contract) | 1–2 | rungs 1–6 tests pass |
| **3** | NOx & mixing, rungs 7–24. **RISK-BEARING — not bulk.** These are phase 1's largest *consumer*: every one rides the equilibrium solve and `Kp = exp(−ΔG°/RuT)`, and their findings are *shapes* (the bell's peak, the minimum pinned at `C_opt`, monotone-vs-turns-back-up) that a last-digit shift in an exponential can move. Deliberately placed straight after phase 1 as the **first real test of whether the transcendental arithmetic holds** | 4–6 | per-rung tests pass; extremum *locations* re-checked, not just values |
| **4** | Nozzle & turbine marches, rungs 25–30 — own convergence behaviour, hence separate | 2–3 | per-rung tests pass |
| **5** | Steady matchers — rungs 31–33, 38–39, 42, 53–56, 61. **Contains the diamond** (§ 6) | 4–6 | per-rung tests pass |
| **6** | Transients — rungs 34–37, 40, 43–52 (the fuel-side limiter family) | 4–6 | per-rung tests pass |
| **7** | **The ladder, rungs 57–84** — the `Hooks` table from § 2, one module per rung | 5–8 | 28/28 reduce-to-prior bit-exact |
| **8** | `main.py` replacement; adjudicate the fragile rungs; re-anchor the fingerprint; **delete the Python** | 2–3 | full suite green on Rust alone |

**Total: 26–40 focused sessions.** The risk is **not** evenly spread and is not only at the
ends: it concentrates in phase 1's transcendental arithmetic, in **phase 3 which consumes it**,
and in the final count-based adjudication. Phases 4–7 are grinding but low-risk.

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

### Consequences for the phase table

Phase 8's `main.py` row is now "Rust CLI prints the tables and dumps plot JSON; port the
chart script; verify it is fast" rather than a three-way choice.
