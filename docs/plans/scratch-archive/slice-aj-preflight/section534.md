
### 5.34 SLICE AJ (rungs 81–84, `AuthorityClockTransient` … `StaircaseLawTransient`) — PRE-REGISTERED, nine probes MEASURED first

Phase 7 is authorised and slice AI closed at § 5.33.7 owing nothing blocking. It booked **one
probe to this pre-flight by name** — § 5.33.7 (e): *delete rung 80's `at_lever` in `engine.py`, run
`test_rung80.py`…`test_rung84.py` and the r80–r84 kernels* — which is § 5.33 (ii)'s question
(*does any rung-81–84 reader depend on the rebuilt machine being its own rung?*) in its sharpest
form. It is run first, and it answers that question and corrects the slice that booked it.

Probes live in `W:\temp\claude\slice-aj-preflight\` (`predictions.md` written before each ran).
`engine.py:N` here is **unwatched** — `tests/test_rust_line_citations.py` guards `rust/` only.

#### (i) THE LEADING FINDING — **NO RUNG-81–84 READER DEPENDS ON THE RIG's CLASS, AND PYTHON's `test_rung80.py` IS NOT BLIND TO THE DELETION: § 5.33.7 (e)'s *"faithfully blind, not under-ported"* IS REFUTED — THE RUST `rung80.rs` IS UNDER-PORTED AT ONE TEST**

The mutation ran on two full copies of `turbojet/` + `tests/` (`ctl\`, `mut\`; in `mut\` lines
`21429–21449`, rung 80's `at_lever`, deleted and checked by AST), the live repo untouched. Both
suites ran `-n 4` under a pytest plugin (`plugin\ajprobe.py`) with **positive controls asserted at
configure time** in every process: the engine file loaded is the copy's own; `'at_lever' in
SplitWallTransient.__dict__` is `True` in `ctl`, `False` in `mut`; `StaircaseLawTransient.at_lever`
resolves to `SplitWallTransient.at_lever` in `ctl` and `StateCoordinateTransient.at_lever` in `mut`.

| | control | mutated |
|---|---|---|
| rigs built by an `at_lever` (by the machine's class) | **all `SplitWallTransient`** (1 312) + 2 `StateCoordinateTransient` | **all `StateCoordinateTransient`** (1 228) |
| a rig that calls `at_lever` again (rig of a rig) | 0 | 0 |
| a rung-81+ method called ON a rig | 0 | 0 |
| distinct (definer, method, receiver) among rung-81+ calls | 28 | 28 — the same set |
| `test_rung81.py`…`test_rung84.py` | 45 passed | **45 passed** |
| `test_rung80.py` | 12 passed | **11 passed, 1 FAILED** — `test_the_knob_is_loud` |
| kernels r80, r81, r81m, r82, r82r, r82t, ctl vs mut, exact `float.hex` | — | **0 differ** of 1 092 / 11 462 / 784 / 153 / 111 / 176 |

The two control-run `StateCoordinateTransient` rigs are `tests/test_rung80.py:98`'s reduce test,
which builds a rung-79 machine on purpose (`_rig(design, StateCoordinateTransient).coord_scan`).

**The failure:** `tests/test_rung80.py:126` reads the walls back as `rig._walls_of(rig, surge)` — a
static method looked up ON THE BUILT RIG. Under the mutation the rig is a rung-79 class, which has
no `_walls_of`, and the lookup dies: `AttributeError: 'StateCoordinateTransient' object has no
attribute '_walls_of'`. § 5.33.7 (e) grepped `test_rung80.py` for `at_lever`, `isinstance` and a
carry test, found none, and concluded the Python suite is blind and `rung80.rs`'s 16/16 under the
Rust mutation *"faithfully blind, not under-ported"*. **Both halves are wrong.** The catcher is
incidental — a method lookup on the rig, not a test about the rig — which is exactly what a grep
for the words of a carry test cannot find. `rust/tests/rung80.rs:310` ports it as the free function
`walls_of(&built, …)`, which accepts any core, so the Rust test cannot fail the way Python's does.
**Booked for step 1:** `rung80.rs::the_knob_is_loud` gains an assert that `built`'s lever/triple
tables are `R80`'s (the only spelling this crate has for Python's attribute lookup on the rig's
class), with a declared-difference note in its header; and the memory entry that repeats the claim
(`rust-port-slice-ai-step7.md`) is corrected, not appended to.

**Why rungs 81–84 survive, by mechanism, not by luck:** every reader that marches reads the rig
through methods whose definer is the SAME on a rung-79 and a rung-80 class. Rung 81's
`authority_mask` is the one reader that dispatches on the rig itself (`engine.py:22099–22103` —
`m._with_share`, `m._quad_gains_at`, `m._charpoly4`, `m._jac4`, `m._quartic_roots_c`, with `m` the rig
`_split_march` returns); all five resolve to rungs 72/73 on both classes. Rung-80 methods
(`_walls_of`, `_split_row`, …) are called on `self`, the top-level machine. An AST scan of
`engine.py:21795–23071` finds **20 references to ladder methods, receivers `self` (15) and `m` (5),
none other** — no unreached branch does what the test did.

**The entry control the advisor asked for** — a zero diff is two findings (§ 5.33 (x) 1), so every
kernel was rerun in `mut\` with a counter on every `at_lever` definer (`kcount.py`):

| kernel | rung-79 `at_lever` entries (mutated) — the rebuild path RAN |
|---|---|
| r80 | 34 |
| r81 | 42 |
| r81m | 2 |
| r82 | 81 |
| r82r | 18 |
| r82t | 78 |

**So the six identical kernels are *ran, no difference*, not *never entered*.**

**The answer to § 5.33 (ii) at rungs 81–84:** no reader, suite value or kernel value depends on the
rebuilt machine being rung 80's; exactly one TEST does, structurally. For the port that makes
`split_wall.rs:44–45`'s booking — *"its `R81`…`R84` lever tables must carry [`R80`]'s `at_lever`"* —
**moot: AJ builds NO `R81`…`R84` tables at all** (§ (ii)), and the readers take an `R80` machine.
The sentence is rewritten at step 1.

#### (ii) THE CELL CENSUS — **0 ADD, 0 SWAP, 0 tables: four reader-only rungs**

Probe 1 (AST, slice AC's predicate): **24 methods over the 4 classes, every one single-definer in
the MRO** — rung 81 5, rung 82 9, rung 83 4, rung 84 6; seven of them `staticmethod`s. **No class
after rung 80 defines `at_lever`, `_shared_rig`, `_cap_fuel`, `_with_coord` or any other cell.**
`_scan` has a second definer, `VariableStatorMatcher` (rung 53), which is NOT an ancestor — a name
reused across unrelated hierarchies, not a cell (the port names them apart: `stator.rs:1060`'s
`scan` vs this slice's).

**Runtime, over the four suites (control):** 129 (definer, method) pairs reached; **all 24 rung-81+
methods reached** (0 unreached). The inherited path: rung 80 `_split_march` (1 306 entries), rung 74
`_demand_tau` (1 655 476), rung 72 `_riding4` (1 304), `_jac4` / `_charpoly4` / `_quartic_roots_c`
(133 each), `_with_share` (135), and **`_quad_gains_at` → rung 73's** (`AppliedReferenceTransient`,
135) — its two definers make it the one inherited CELL on the path, already a table field.

**Therefore: `TripleHooks` stays at 18 fields; `LeverHooks` unchanged; no carriers** (§ (iv));
**no `R81`…`R84`** — a Python machine of class 81–84 is an `R80` core plus free functions, since
nothing in the tables differs. The one dispatch the port must get right is `authority_mask`'s five
calls ON THE RIG: they go through the rig core's tables (`m`), not the caller's. With no swaps the
two are the same pointers, so no value gate can see a port that uses the wrong one; step 7 owes an
injection on the rig's table to show it is the rig's that is read.

#### (iii) BOOKED ITEMS, AND EVERY SITE WHERE THE CRATE ALREADY DESCRIBES RUNGS 81–84

| # | where | claim | verdict |
|---|---|---|---|
| A | § 5.33 (ii), (xi), § 5.33.7 (e) | does any rung-81–84 reader depend on the rig being its own rung? run the deletion through 84 | **§ (i)**: no reader; one rung-80 TEST, structurally — and it refutes § 5.33.7 (e)'s *"faithfully blind"* |
| B | `split_wall.rs:44–45` | rigs are `SplitWallTransient` by class (TRUE, measured); *"R81…R84 lever tables must carry R80's at_lever"* | **MOOT** — no such tables (§ (ii)). Rewritten at step 1 |
| C | `state_coordinate.rs:96–97` | *"the plant bumps on EVERY incidence call — including every rung-80–84 march slice AJ will port"* | **OVERSTATED IN SCOPE.** Every rung-81–84 reader takes `inc: bool = False` and threads it to `_split_march`; every call site in `tests/test_rung81–84.py` and in the r80–r82t kernels passes `inc=False`; measured, `inc=False` at **all 1 306** `_split_march` entries and `_phi_ref == "phi"` at **all 4 801 992** `_cap_fuel` entries. No shipped rung-80–84 march touches rung 79's counters. **The thread-local DECISION stands** (any incidence march in the binary still would); the sentence is re-scoped at step 1 |
| D | `sensed_cap.rs:596` | bracketing buys *a root exists*, never `G' > 0` — *"That distinction is rung 83's whole subject"* | **MISATTRIBUTED.** Root-vs-slope (uniqueness) is **rung 78's** subject. Rung 83's is sign-change-vs-root — on a `min`-built residual a bracket does not even buy a root, which REFUTES the sentence's premise there; on rung 76's continuous `G` the premise holds. Corrected at step 1 |
| E | `rung41.rs:457`, `slice_l_oracle.rs:42`, `oracle/dump_slice_l.py:32` | *"rung 83's identity round-trip sold as verification"* | TRUE — rung 83's process lesson, cited as such |

Enumerated by grepping the whole crate (`src`, `tests`, `oracle`) for the four class names, their
public method names, `rung 8[1-4]`, `81–84` and `slice AJ` — § 5.33 (x) 2's repaired scope.

#### (iv) PROCESS-GLOBAL STATE AND CARRIERS — **none**

Rungs 81–84 hold **0 attribute writes** (`self.x =`, `m.x =`, `Cls.x =`, `type(self).x =`), **0
`assert`s, 0 `raise`s**, and one scope entry (`m._with_share` in `authority_mask`, rung 72's
existing `ShareScope`). No class state, no new core field, no refusal that panics: every failure
mode here is a RETURNED string (§ (vi)).

#### (v) THE ARITHMETIC SURFACE — **six `round`s, one float sum, three keyed orderings, `id()`, and four `%g` messages**

* **`round(x, 6)` ×2** (`:22256`, `:22876` — `c.tau_f/tau_f`) and **`round(s, 9)` ×2** (`:22880`,
  `:22881`): the crate's format-and-parse spelling (`round6`-style, as `round10`/`round12`). A
  229 990-case stress (random + near-half adversarial) found **0 mismatches** between Rust
  `format!("{:.N}")`-then-parse and Python `round` on PyPy 3.11.15 AND CPython 3.14.3; for `n ≥ 1`
  an exact decimal tie cannot exist in a binary float. Every argument the four suites produce was
  recorded (`E.round` shadowed in the module globals, positive control asserted): **2 073 records,
  0 Rust mismatches.** `round(.,6)` always returned `3.0` (κ = 3).
* **`round(s, 9)` IS a merge key.** Within one march, 0 merges (minimum spacing `1.25e-3`), so
  `:22879`'s *"9 places cannot merge two distinct march points"* holds; ACROSS the three `ds`
  marches, 339 distinct values collapse to 195 — the intended cross-`ds` merge. The Rust key is
  the rounded `f64`'s bits, and the merge must be reproduced bit for bit.
* **`int(round(edge/ds))` ×2** (`:22897–22898`): 1-argument `round` is ties-to-even and returns an
  int → `round_ties_even` + cast. Values measured: 16, 17, 33, 65, 290 and `309.99999999999994`
  (max deviation from an integer `5.7e-14`); **0 exact `.5` ties.**
* **One float sum:** `rate = sum(1.0/t for t in tt)` (`:22104`, 4 terms) — `authority_mask` only,
  used as the zero-root bar `1e-4·rate`. `sum(x["n_scored"] …)` (`:22014`) is an integer sum.
  Naive fold vs CPython's compensated `sum()` can differ in the last ulp; the path is the r81m
  kernel's, see below.
* **Keyed orderings:** `min(hats, key=…)` (`:22255`) and `min(summ, key=summ.get)` (`:22883`)
  return the FIRST minimum in iteration order — `summ` is a dict, so INSERTION order: the Rust map
  is insertion-ordered (a `Vec` of pairs), never a `HashMap`/`BTreeMap`. `sorted(…, key=dist)`
  (`:22495`) is stable → `sort_by` (stable), never `sort_unstable_by`.
* **`id(p)`** (`:21941`, `:21944`, `:22218`, `:22220`) maps `_riding4`'s points back to trajectory
  INDICES. `shared_actuator.rs`'s `riding4` returns CLONES, so the port needs an index-returning
  variant (or `riding4` re-expressed over indices). A float-equality match on `s` would be a
  different claim and is refused.
* **`float('nan')`** (`:22414`) is a returned VALUE in `_threshold_row`; the oracle must encode NaN
  explicitly.
* **`%g`:** four returned messages (`:22305`, `:22784`, `:23030`, `:23061`). The crate has **no
  `%g` formatter** (it has `py_e`, `py_repr`). A new helper is owed, pinned on the driven values
  (`0.004`, `0.3`, `1e-12`, `0.0198`, `1`) AND on the shipped default `eps = 1e-7`, which Python
  prints `1e-07` — the two-digit exponent Rust's `{:e}` does not produce.

**The CPython exemption, pre-registered from the kernels already shipped:** control (PyPy) vs the
committed CPython golden differs on **r80: 37 keys and r81m: 155 keys, every one a `c0`/`c1`
charpoly coefficient** (`gc.arms…` / `m.arms…cells[…].c0|c1`); **r81, r82, r82r, r82t: 0.** No
`zeros` key differs. So the exemption is `_charpoly4`'s path, not `:22104`'s sum. AJ's own oracle
arms (§ (ix) P2) are predicted exact on both interpreters except `authority_mask`'s `c0`/`c1`.

#### (vi) THE VOIDS — **twelve returned strings, ten driven; the suites assert three; two NAME THE WRONG CAUSE**

Driven directly (`p_drive.py`), exact strings:

| code | method | driven by | string |
|---|---|---|---|
| V1 | `_bisect`, `root_class` | `r = 1.0` (no four-loop window) | `V1: four-loop window empty at a bracket end` |
| V1 | `_bisect` | — | `V1: window closed mid-bisection` — **not driven** |
| V3 | `_bisect` | `r = 0.35`, `phi_lim = 0.760` | `V3: threshold not strictly inside [0.004, 0.3]` |
| V4 | `corrector_step` | `r = 1.0` | `V4: kappa impure` |
| V4 | `corrector_secant` | `r = 1.0` | `V4: kappa impure at a start point` |
| V4 | `corrector_secant` | — | `V4: kappa impure at an iterate` — **not driven** |
| V5 | `corrector_step` | `c = 1` | `V5: \|1-c\| below 1e-12` |
| S2 | `corrector_secant` | identical starts (2 marches) | `S2: flat pair, \|dg\| < 1e-12` |
| V3 | `lattice_count` | `r = 1.0` | `V3: an edge off the march grid` |
| V2 | `staircase_number` | `r = 1.0` | `V2: kappa impure at an end of the bracket` |
| V5 | `staircase_number` | `tau_lo = tau_hi` | `V5: no edge move in [0.0198, 0.0198]` |
| V6 | `root_class` | `eps = 1.0` | `V6: bracket narrower than 1` |

The suites assert V1 (as the `window_open` flag), V3 (truthiness) and V5 (substring `"V5"`).

**Two voids misname their cause.** At `r = 1.0` there is no four-loop window, so κ is EMPTY, not
impure, and no edge exists to be off the grid — yet `corrector_step`/`corrector_secant` say *kappa
impure* and `lattice_count` says *an edge off the march grid*. These are Python's strings: **the
port reproduces them word for word** and a gate pins each at `r = 1.0`; the misnaming is recorded
here, not repaired in Rust.

#### (vii) NESTS — **zero**

The runtime plugin (positive control asserted: `(2, 1, 2, 1)`) over all five suites: `_b_state` set
15 057 908 times, `_v_state` 15 060 258, **0 nests** of either, in the engine or the tests. (The
mutated run did not install these hooks — its zeros are *not instrumented*, not measurements; the
same holds for its `inc` and `phi_ref` tallies.)

#### (viii) SIZING — **P1 can separate per-LINE from per-FIELD for the first time**

| slice | rungs | total | methods | body | dict fields | distinct (method, key) |
|---|---|---|---|---|---|---|
| AG | 75+76 | 1 073 | 25 | 628 | 217 | 193 |
| AH | 77+78 | 1 125 | 25 | 636 | 211 | 165 |
| AI | 79+80 | 1 037 | 23 | 530 | 185 | 175 |
| **AJ** | **81–84** | **1 271** | **24** | **653** | **548** | **455** |

Rung by rung: 81 193 body, 82 259, 83 78, 84 123. AG–AI all sit near 0.34 dict fields per body
line, so a per-line and a per-field cost model predicted the same thing and AI's P1 could not tell
them apart. AJ is 0.84 — **2.6× denser in returned structure** — and the two models now disagree by
a factor of two: AI's 2 195 Rust lines per 530 body lines scales to **≈ 2 700**; per distinct key
(2 195 / 175) to **≈ 5 700**.

#### (ix) PREDICTIONS — pre-registered, settled at the last step

* **P1 — the port follows the BODY, not the field count.** Python's body lines already contain the
  dict literals; Rust adds a struct declaration per returned shape (≈ one line per distinct key)
  on top of the per-line cost. **P1 predicts the four modules together land in 2 700 – 3 700
  (`wc -l`, AH's count); above 4 000 the per-field mechanism wins.**
* **P2 — the oracle.** No r83/r84 kernel exists; step 6 writes a `dump_slice_aj.py` over all four
  rungs. Predicted bit-exact on PyPy, and on CPython except `authority_mask`'s `c0`/`c1` (§ (v)).
  **Budgeted before it is written:** the cached per-test call times for the four suites are 6.4 s
  (81), 119.6 s (82), 105.8 s (83) and **1 176 s (84)** — rung 84's p6 alone 345 s — so the dump
  cannot replay the suites' grids; it takes one `ds` pair and one ramp per reader, and says so.
* **P3 — every void string reproduces byte for byte**, the two misnamed ones included, and the new
  `%g` helper matches Python on the five driven arguments and on `1e-07`. Falsified by any byte.
* **P4 — the rig dispatch is the RIG's.** An injection that swaps `quad_gains_at` on the RIG's
  table alone moves `authority_mask`; the same swap on the caller's table alone moves nothing.
  Falsified either way.
* **P5 — Python's deletion result reproduces in Rust through rung 84**: with `at_lever: r80_at_lever,`
  deleted, every AJ value and oracle key is unchanged, and the ported `rung80.rs::the_knob_is_loud`
  now FAILS (step 1's pointer assert) — the Rust suite catches exactly what Python's does.
* **P6 — 0 ADD, 0 tables, `TripleHooks` at 18**, all steps.
* **P7 — the step count is not scored.** The planned list: **1** plumbing — four modules, no tables,
  the `%g` helper, `riding4` by index, items B/C/D corrected, the `rung80.rs` pointer assert
  (§ (i)); **2** rung 81 (`_central`, `_criterion_at`, `authority_clock`, `_tau_f_inert`,
  `authority_mask`); **3** rung 82 (nine methods); **4** rungs 83 + 84 (ten methods, 201 body
  lines between them); **5** the four ported suites plus the ten driven voids written from the
  source; **6** the oracle; **7** P4's rig-dispatch injection and P5's closing deletion — the
  dispatch content a slice with no swaps still has.

**Naming, decided now:** `fuel_transient::classify` and `stator.rs`'s `scan` exist; this slice's
`classify` and `_scan` are named in their own modules and called fully qualified, with no bare
`use` of either.

#### (x) DEFECTS IN THIS PRE-FLIGHT's OWN INSTRUMENTS

1. **The kernel dumps carried a LOAD control, not an ENTRY control.** `at_lever_in_r80` proved
   which copy was imported, not that the rebuild path ran; *0 differ* alone could have meant
   *never entered*. The advisor's catch; `kcount.py`'s rerun (§ (i)) is the repair.
2. **The mutated suite ran the light plugin**, so its `inc`, `phi_ref` and nest tallies are zero by
   construction. First read from the merged output as measurements; they are reported as *not
   instrumented* (§ (vii)).
3. **Call COUNTS differ between the two runs** (`threshold_law` 4 vs 3, `authority_mask` 1 vs 2,
   `_scan` 648 vs 567). Checked per worker, not assumed: `threshold_law` and `authority_mask` are
   module-scoped fixtures (`tests/test_rung82.py:84`, `tests/test_rung81.py:78`), and each worker
   that ran a test needing one called it exactly once (control 1, 1, 1, 1; mutated 1, 0, 1, 1). A
   module fixture is rebuilt PER WORKER, so only the SETS compare across runs — and they are equal.
4. **The rig tracker counted rung-81+ methods on a rig, and missed rung-72 methods on one.**
   `r81p_on_rig = 0` is true and was read as *nothing dispatches on the rig*; `authority_mask`
   dispatches five inherited methods there (§ (ii)). The AST receiver scan found it. A census
   scoped to NEW methods cannot see an OLD method called on a new object.
5. **Run times from this pre-flight are not costs.** The two suites (35:28 and 30:36) competed
   with twelve kernel processes and one was wrapped by the census; P2's budget uses the cached
   per-test durations instead.

#### (xi) WHAT SLICE AJ OWES BEFORE ITS STEP 1 — **nothing blocking**

Step 1 carries § (i)'s `rung80.rs` pointer assert and the memory correction, § (iii)'s items B, C
and D, the `%g` helper and `riding4` by index. Nothing is booked forward past rung 84.
