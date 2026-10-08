
### 5.33 SLICE AI (rungs 79 + 80, `StateCoordinateTransient` + `SplitWallTransient`) — PRE-REGISTERED, eleven probes MEASURED first

Phase 7 is authorised (§ 1's phase table) and slice AH closed at § 5.32.7 owing nothing, so this
slice needs no authorisation and inherits no debt from its predecessor. It inherits **four items
booked to it BY NAME** by earlier slices, and — the advisor's catch, and the enumeration the last
two pre-flights did not run — **twelve sites where the crate's own doc comments already describe
rungs 79–80**, each a sentence about code not yet written. § (iii) checks every one.

Probes live in `W:\temp\claude\slice-ai-preflight\`. `engine.py:N` in this section is
**unwatched** — `tests/test_rust_line_citations.py` guards `rust/` only (§ 5.32 (iii)).

#### (i) THE LEADING FINDING — **THE VALUE BREAK BOOKED HERE SINCE SLICE AE IS REAL AND REACHED, AND TWO SEPARATE MECHANISMS HIDE IT: 0 OF 196 VALUES MOVE, 128 BRANCH ENTRIES DO, AND THE ONLY INSTRUMENTS THAT SEE IT ARE RUNG 79's PROCESS-GLOBAL COUNTERS**

`_with_coord` has two definers — rung 74's writes `_lag_coord`, rung 79's writes `_phi_ref` — and
§ 5.29 (v) / § 5.30 (ii) booked *"the value break can first be observed at slice AI"*. Probe 4 finds
**five** dispatch sites of the setter in the whole of `engine.py` — four in rung 79's own
`coord_march`, on a rung-79 receiver (three calls, and one bound-method REFERENCE handed to
`_with_probe` at `:21172`, which the first AST census of CALLS missed and the runtime counter did
not — § (x) defect 5), and one in **rung 74's `demand_gains`** (`engine.py:18278`), which pins
`m._lag_coord = "clip"` by ASSIGNMENT (`:18269`) and then enters
`m._with_coord("demand", m._demand_gains_at, …)` — a DISPATCH. `demand_gains` is single-definer, so
every rung from 75 to 84 inherits it, and on a rung-79 or rung-80 machine the dispatch lands on rung
79's body.

Probe 5 runs `demand_gains` at the fingerprint's r74 settings (`every = 4`) on machines of four
rungs, spying on the scope and snapshotting rung 79's six class counters around each run. **Diffed
against rung 78**, where the break is introduced, not against 74 — the advisor's correction, since
rungs 75–78 carry their own class defaults:

| machine | `_with_coord` body run | `(_lag_coord, _phi_ref)` inside the scope | rung-79 counters moved | keys ≠ rung 78 |
|---|---|---|---|---|
| 74 | 74's | `("demand", —)` | 0 | 0 / 196 |
| 78 | 74's | `("demand", —)` | 0 | — (baseline) |
| **79** | **79's** | **`("clip", "demand")`** | **hits 128, binds 128, `calls_inc` 128, `fb_inc` 128** | **0 / 196** |
| **80** | **79's** | **`("clip", "demand")`** | the same | **0 / 196** |

**Both fields end the scope wrong, the new branch runs 128 times, and not one number moves.** Two
masks, each already on record for its own reason:

1. **`_lag_coord` stays `"clip"` where rung 74 meant `"demand"`** — invisible because the
   coordinate's only reader, `_demand_target`, tests `== "demand-latched"`, so `clip` and `demand`
   take the same branch (§ 5.30 (i)'s *three-valued tag read by a two-valued test*).
2. **`_phi_ref` becomes `"demand"`** — a THIRD value rung 79's class docstring does not declare
   (`"phi" | "incidence"`). `_cap_fuel` tests `== "phi"`, so everything else is incidence, and the
   incidence branch is entered 128 times. **But `fb_inc = calls_inc = 128`**: every one of those
   calls short-circuited to `_surge_fuel`, which brackets its own HARDCODED `phi` residual (rung 79
   § 5.1). The coordinated residual was never bracketed once.

**So the booking resolves, but not the way it was written.** The break IS observable at AI — by a
field read inside the scope and by rung 79's counters — and it is **not** observable by any value
at the shipped settings. A reading of `0 / 196` alone would have reported it INERT; probe 5's first
run did exactly that (§ (x) defect 1).

**Three consequences, each pre-registered:**

* **(a) The port's gate for the break is a COUNTER gate plus a field readback**, and those counters
  are process-global in Python — which is what forces § (iv)'s decision rather than leaving it to
  taste. The instrument that sees this slice's leading hazard is the one with the known race.
* **(b) `_phi_ref` is a `Cell<&'static str>` with Python's `== "phi"` test and everything else
  treated as incidence — NOT an enum.** `"demand"` is reached on a callable path; a two-variant
  enum, or a `match` with an `unreachable!()` arm, turns Python's silent proceed into a panic —
  slice AF step 6's *"the port refuses more than Python does"*. `lag_coord`'s own doc
  (`two_spool_transient.rs:733`) states the same decision for the same reason. And the third value
  reaches a REFUSAL: with `_gauge_k ≠ 1`, a rung-74 reader on a rung-79 machine raises rung 79's
  `:20910` (probe 6 drives it). The Rust must reach that refusal, not a different panic.
* **(c) A latent Python defect, recorded and not repaired.** `demand_gains` on any machine of rung
  79 or later runs with both coordinate fields wrong. No shipped test and no fingerprint kernel
  drives it (`test_rung74.py` and `kernel_r74` build rung-74 machines), and probe 7 counts the rung
  79/80 suites' dispatches: **four, every one rung 79's own body from `coord_march`** — zero from
  `demand_gains`. The port is a translation, and because `CoordScope` goes THROUGH the table, re-aiming `with_coord` at rung 79 reproduces the
  Python exactly — which is what the four-site rule (§ 5.30.6 (ix)) exists to guarantee.

#### (ii) THE CELL CENSUS — **0 ADD; six swaps, all already fields; and the zero is durable because nothing after rung 80 overrides anything at all**

Probe 1, under slice AC's repaired predicate (overridden **and** substitutable):

* **21 distinct method names** — rung 79 has 13, rung 80 has 10; `at_lever` and `_shared_rig` are
  in both.
* **6 substitutable SWAPS over 4 names, every one already a field**: `at_lever` (79, 80 —
  `LeverHooks::at_lever`), `_shared_rig` (79, 80 — `TripleHooks::shared_rig`), `_cap_fuel` (79 —
  `TripleHooks::cap_fuel`) and **`_with_coord` (79 — `TripleHooks::with_coord`, the name reuse the
  cell was built in advance for)**.
* **1 INCOMPATIBLE name reuse**: `split_gains`, rung 70 → 80 — booking B, § (iii).
* **16 single-definers**, which are not cells. ⇒ **0 ADD. `TripleHooks` stays at 18 fields.**

**`_with_coord`'s verdict prints `RENAMED?`, not `SAME`**: the second parameter is `coord` at rung
74 and `ref` at rung 79. All five dispatch sites pass it POSITIONALLY, so the pair is substitutable and
the swap is legal. `demand_coordinate.rs:350` states this correctly (*"one parameter renamed"*);
`three_loop.rs:618`'s *"an IDENTICAL signature"* is imprecise in exactly this — recorded as item H and
corrected at step 1.

**Durable, and more strongly than at AH.** The four classes after rung 80 redefine **none** of this
slice's 21 names — and define **no `at_lever` at all** (probe 8). The carry chain ends here:

| rung | class | carries | NEW |
|---|---|---|---|
| 78 | `ResidualGaugeTransient` | 7 | `_gauge_k` |
| **79** | **`StateCoordinateTransient`** | **8** | **`_phi_ref`** |
| **80** | **`SplitWallTransient`** | **9** | **`_sm_air`** |
| 81–84 | — | *inherit rung 80's* | — |

**Booked forward to slice AJ, by measurement:** every rig a rung-81–84 machine builds through
`_shared_rig` → `at_lever` is a `SplitWallTransient` **by class**. The Rust `R81`…`R84` lever
tables must reproduce that by carrying `r80_at_lever`, and AJ's pre-flight owes the question of
whether any rung-81–84 reader depends on the rebuilt machine being its own rung.

**Two new core carriers**, not table fields: `phi_ref` (`Cell<&'static str>`, default `"phi"`) and
`sm_air` (`Cell<Option<f64>>`, default `None`). **The setter rule** (`CoordScope`'s doc: dispatch
the setter iff a later rung overrides `_with_*` onto a DIFFERENT field) applied to the three new
scopes: `_with_coord` at 79 is the rule's own case and stays dispatched; `_with_air` and
`_with_probe` are single-definer over all 58 classes, so their guards write directly (`ShareScope`
/ `_with_windup`'s precedent). The plain assignments — `_phi_ref` at `:20962`/`:20970`, `_sm_air`
at `:21447`/`:21464` — port as plain `set`s (§ 5.30.6's four-site rule, extended to two new fields).

#### (iii) FOUR ITEMS BOOKED BY NAME, AND TWELVE SITES WHERE THE CRATE ALREADY DESCRIBES CODE NOT YET WRITTEN

| # | where | claim | verdict |
|---|---|---|---|
| A | § 5.29 (v), § 5.30 (ii), § 5.30.6 (ix) | `_with_coord`'s second definer writes another field; the four-site rule comes due; the value break is first observable here | **§ (i)**: real, reached, masked twice. Gate by counters + readback |
| B | § 5.27 (iii) P3 | rung 80's `split_gains` ports as a different function; falsified only if a caller must choose | **DISCHARGED.** Rung 70's only caller is `rung67_control` (`:14558`), driven only by `tests/test_rung70.py:298` on a rung-70 machine. Python's `TypeError` on a rung-80 machine is **unreachable by shipped code**; the Rust returning a value there is a divergence on an unreachable path, which `cross_split.rs:1093` already says |
| C | § 5.32.2, `sensed_cap.rs:633`, `stiffness_ledger.rs:208` | rung 79's `slope` has an expression-first `1e-9` fold at `:21029` | **STILL at `:21029`**, and it has **nine** siblings in the same class — § (v) |
| L | § 5.30.1 (h)'s survivor table | the injection *`CoordScope::drop` writes `lag_coord` DIRECTLY, bypassing the cell* SURVIVES every rung-74 gate, because there the cell and the field are one; *"only rung 79 separates them, and that is slice AI's gate to write"* | **OWED, and § (i) says which instrument.** On an R79 machine the guard's `set` dispatches to rung 79 (writes `phi_ref`, hands back the displaced `"phi"`) and the defective `drop` writes that `"phi"` into `lag_coord`. After the scope Python leaves `("clip", "phi")`; the survivor leaves **`("phi", "demand")`** — an out-of-set coordinate and a stuck reference. By § (i)'s two masks the demand path's VALUES may well stay put, so the killing gate is the **post-scope field readback**, not a value diff. Predicted in P2, never measured in Rust |
| D | `residual_gauge.rs:1214` | *"a reader from rung 79 onward holds no measurement at all"* (of `_b_state`/`_v_state` nests) | **MEASURED — § (vii)**: 0 nests in `engine.py` at 79/80, statically and at runtime; but **4 at runtime in the SUITE** — a test helper the static census cannot see |
| E | `two_spool_transient.rs:858` | `_with_gov`'s three call sites are *"`split_gains` (rung 70) and two inherited readers at rungs 80/81"* | **FALSE ON LOCATION.** The other two are `shared_cells` (rung 72, `:16591`) and `applied_cells` (rung 73, `:17269`). The count (3) and *all pass a literal `None`* are right. **Corrected at step 1** |
| F | `gas.rs:1220` | a rung-80 docstring calls a dropped field *"THE EIGHTEENTH INSTANCE of the trap"* | TRUE (`:21433`) |
| G | `anti_windup.rs:379`, `:855` | `_with_coord` has two definers, 74 and 79; a dispatched pin would hit the second | TRUE |
| H | `three_loop.rs:618` | the two `_with_coord` definers have an *IDENTICAL signature* | **IMPRECISE** — one parameter is renamed (§ (ii)); substitutable because every call is positional. Corrected at step 1 |
| I | `demand_coordinate.rs:57–67` | `phi_ref` stays unborn until this slice; no slot here is one a later slice can re-aim | TRUE — no `phi_ref` or `sm_air` carrier exists in `rust/src` |
| J | `demand_coordinate.rs:767`, `:800` | `_cap_fuel` has three definers (74, 78, 79), so an inherited reader must go through the table | TRUE — probe 1: 3 definers |
| K | `two_spool_transient.rs:733` | `lag_coord` is a string cell and not an enum, so an out-of-set value reaches Python's refusal | TRUE, and it now **extends to `phi_ref`** — § (i)(b) |

A, B, C and L are the plan's bookings; the twelve crate sites are C's two, D, E, F, G's two, H, I,
J's two and K. One of the twelve is false (E) and one imprecise (H).

#### (iv) PROCESS-GLOBAL STATE — **SIX COUNTERS, A FLAG AND A LOG ON THE CLASS, ALL WRITTEN BY THE PLANT, AND ALL SIX COUNTERS RETURNED BY A READER AS ORACLE VALUES**

Probe 3 lists **sixteen class-level writes** in rung 79 (rung 80 has none): `_coord_hits` /
`_coord_binds` bumped in `_cap_fuel`; `_coord_fb_phi` / `_coord_fb_inc` in `_phi_cap`'s `shipped`
closure; `_coord_calls_phi` / `_coord_calls_inc` in `_phi_cap`; all six zeroed and read back by
`coord_march`, which RETURNS them (`hits`, `binds`, `calls_*`, `fb_*`, `br_*`); and `_coord_probe` /
`_coord_log` set and restored by `_with_probe`, with `_cap_fuel` appending to the log whenever the
flag is up. Rung 78 had two such counters; rung 79 has **six, plus a log**.

**THE DECISION: `thread_local!`, not `static`s — made on three measured grounds, and rung 78's
statics are knowingly left as they are.**

1. **The library spawns no threads.** `rust/src` has zero `thread::spawn`, zero `rayon`, zero
   `par_iter` (probe 11). A reader's march therefore runs on its caller's thread, so per-thread
   counters equal Python's per-process ones for every value any reader returns. **Falsified the
   day a march crosses a thread** — P3.
2. **A `static` would be corrupted by any other test in the binary that marches an incidence
   machine**, and the plant bumps on EVERY `_phi_ref != "phi"` march — including every rung-80–84
   incidence march slice AJ will port. § 5.32.7 (c)'s race, at six counters instead of two, with a
   lock that would have to spread into every later test file. **A lock protects the resource you
   wrap it around; a thread-local removes the shared resource.**
3. **The crate's own counters are already thread-local** — nine modules (`three_loop.rs`,
   `cross_split.rs`, `full_split.rs`, …) use `thread_local!` + `bump`. Rung 78's `GAUGE_HITS` /
   `GAUGE_BINDS` are the only process-global pair, and their module doc gives the reason (a per-core
   `Cell` would be Python's rejected instance attribute) — a reason a thread-local satisfies too.

**Rung 78's pair is not converted.** They are shipped, AH step 7's `Mutex` protects them, and
changing them moves no value. The crate will carry two conventions for one Python construct, and
the rung-79 module doc says so and why.

**The flag and the log MUST NOT be core fields.** Rung 79's own `_with_probe` docstring records
that exact shape shipping once: the flag written on the INSTANCE, `_cap_march` building a NEW rig
through `at_lever`, and the log coming back EMPTY while `hits`/`binds` read 1366/1366. A core
field is the Rust spelling of that instance attribute.

#### (v) THE ARITHMETIC SURFACE — **one float sum, precedented thirteen times; ten expression-first folds; and an upper median**

Probe 10 lists every site for a hand reading (AH § (iv)'s lesson: at small n the list IS the
measurement):

* **`sum()` — 10 sites, 9 of them integer COUNTS** (`sum(1 for …)`). **ONE float sum**: rung 80's
  `rate = sum(1.0 / t for t in tt)` (`:21750`), four terms, feeding only the threshold
  `abs(z) < 1e-4 * rate` in the `zeros` count. The same expression appears **13 times** in
  `engine.py` and is already ported as a naive left fold at rungs 72, 73, 75 and 76
  (`shared_actuator.rs:2275`, `applied_reference.rs:1229`, …). CPython's compensated `sum` can
  differ in the last bit; that moves `zeros` only if a root sits within an ulp of the threshold.
  **No `**`, no `pow`** in either class.
* **Expression-first `max(abs(x), c)` folds: TEN, all in rung 79** — booking C's `1e-9` at
  `:21029`, eight `1e-30`s (`:21044`, `:21075`, `:21159`, `:21161`, `:21187`, `:21188`, `:21338`,
  `:21339`) and one `1e-12` (`:21205`). Python's `max(nan, c)` returns `nan` and `f64::max` returns
  `c`; the crate's rule for this spelling is § 5.32.2's, and it applies to all ten, not to the one
  booked.
* **`max([…] + […])` with no `default`** (`sched_moved`, `:21159`) raises on empty in Python; the
  `split_*` readers use `default=None`/`0`. Every argument is an `abs` or a count, so slice AA's
  *`fold(0.0, max)` ≠ `max(default=)` when all are negative* has no surface.
* **`sorted(gaps)[len(gaps) // 2]`** (`coord_march`'s `gap_med`, `d_med`) is the UPPER median — an
  index, not an average. Ported as an index.

#### (vi) THE REFUSALS — **all SEVEN reachable; the suite asserts two, one of them through a needle matching eight sites; and the guard written to catch a WIRING bug fires on a LEGAL input two rounding steps wide**

Probe 6 drives every one (on PyPy and, by § (x) defect 3's accident, on CPython too — identical):

| site | method | reached by | suite |
|---|---|---|---|
| `:20910` | `_cap_fuel` | `_phi_ref = "incidence"` × `_gauge_k = 2` — **and by the third value `"demand"`** | yes — needle `"REFUSED"`, **8** occurrences in `engine.py` |
| `:21285` | `_forced_cap` | binding bracket never turns (`phi_lim = 50`, direct call) | no |
| `:21301` | `_forced_cap` | slack bracket never turns (`phi_lim = 1e-3`, direct call) | no |
| `:21467` | `_shared_rig` | `sm_air < sm` | yes — sharp needle |
| **`:21486`** | **`_shared_rig`** | **`sm_air = sm + 1 or 2 ulp` — an ADMISSIBLE input** | no |
| `:21551` | `_split_row` | a valve-less machine, direct call | no |
| `:21647` | `split_arrest` | walls not bracketed (`phi_lim_lo = 0.78`) | no |

**`:21486` is the finding.** Its message says it catches *"the split did not reach the plant"* —
a plumbing defect, a dropped keyword. It also fires on a legal input. At the fingerprint's
`sm = 0.4545454545454546` and `phi_surge = 0.55`:

| `sm_air` | built `phi_air` | result |
|---|---|---|
| `sm + 1 ulp` | `0.8` | REFUSED |
| `sm + 2 ulp` | `0.8` | REFUSED |
| `sm + 3 ulp` | `0.8000000000000002` | marches |

`1 + sm_air` has four times `sm`'s ulp, so the first two steps round away before the multiply and
the two walls come out as one float. Python's reading of its guard stands — a split that does not
separate the walls must not march as one. **But the band is a float fact, and the port must
reproduce it bit for bit**: the Rust wall product must round identically. A gate at `+1`, `+2`
(refuse) and `+3` (march) ulp, written from the source.

**The ported needles must be sharper than the suite's.** `"REFUSED"` matches eight sites; each of
the seven messages carries a phrase unique in `engine.py`, and the ported gates use those.

#### (vii) THE `_b_state` / `_v_state` NEST — **zero in the ENGINE at rungs 79/80, and FOUR in the SUITE, at a site no static census of `engine.py` can reach**

Probe 2 reruns § 5.32's probe 14 (the freezing set DERIVED, 37 methods). The ladder's nests are
still the six at rungs 77/78; **rungs 79/80 add none**. Rung 79's four freezing methods
(`_coord_at` with its two inner closures, `coord_census`, `coord_forced`) open their block, call
only non-freezers inside it, and close it. Probe 7 — AH's runtime plugin, positive control kept and passing
(`b_sets=2 b_nests=1 v_sets=2 v_nests=1`), `-n0`, over `tests/test_rung79.py` +
`tests/test_rung80.py` on PyPy, 37 passed — **disagrees**:

| field | sets | nests | where |
|---|---|---|---|
| `_b_state` | 1 139 854 | **4** | `StateCoordinateTransient <- _c_at:19309`, and NO other `engine.py` frame |
| `_v_state` | 1 139 952 | **4** | the same |

A nest whose only `engine.py` frame is the callee means **the caller is the test file**.
`tests/test_rung79.py:305–320`'s helper `_a_cap` freezes `m._b_state, m._v_state = q, v` itself
and then calls `m._c_at(…, q, v)` twice inside that block. The first call is the nest (the field
is already non-`None`); `_c_at`'s own `finally` thaws the plant; the second call re-freezes from
`None` and is not one. Four calls of `_a_cap` ⇒ four nests.

**It is AH § (i)'s SITE 1 shape** (`leg_slopes`: nest, never re-freeze, safe by statement order),
handed the enclosing `(q, v)`, with a DEAD window: between the first `_c_at`'s return and the
second's own set, nothing reads the plant. So restore-to-`None` and restore-previous agree on
every value here — **but the ported `rung79.rs` must port `_a_cap` WITH its outer guard**, and it
will be the first ported TEST in the phase that holds a guard across a call to a guarded reader.

**Item D is closed by measurement, and not the way the static probe said.** From rung 79 on the
ENGINE holds zero nests; the SUITE holds four, at one site. § 5.32 (viii) defect 5 said *a scope
error deletes the evidence of itself*: the static census ranges over `engine.py`, the port ports
the tests too, and the site sits in exactly the file the census never opened. The runtime counter
found it because it counts where the SET happens, not where the source is.

#### (viii) SIZING, AND WHY P1 CAN FINALLY SEPARATE ITS TWO MECHANISMS

Probe 9, AH's rule (non-blank, non-comment, minus docstrings):

| slice | rungs | classes | total | methods | body |
|---|---|---|---|---|---|
| AG | 75+76 | 2 | 1 073 | 25 | 628 |
| AH | 77+78 | 2 | 1 125 | 25 | 636 |
| **AI** | **79+80** | **2** | **1 037** | **23** | **530** |

Rung 79 is 598 / 13 / 301, rung 80 439 / 10 / 229. **AI is AH's class count at 83 % of its body.**
That is the separation § 5.32 (vii) P1 wanted and could not get: AH's two mechanisms were 140 lines
apart, AI's are ~480 — § (ix) P1.

#### (ix) PREDICTIONS — pre-registered, settled at the last step

* **P1 — the Rust line count separates per-CLASS from per-LINE cost.** Per-class (AH landed on it):
  the two modules ≈ **2 900** (AG 2 930, AH 2 899). Per-line: 2 899 × 530 / 636 ≈ **2 420**.
  **P1 predicts per-class: `state_coordinate.rs` + `split_wall.rs` in 2 700 – 3 100.** Below 2 600
  the per-line mechanism wins. The ratio is reported and not scored (§ 5.32 (vi)).
* **P2 — the `with_coord` re-aim is invisible to every VALUE and visible to COUNTERS and FIELDS.**
  The Rust oracle's `demand_gains` on an R79 machine reproduces § (i): every value key bit-identical
  to R78, `calls_inc = fb_inc = hits = binds = 128` at `every = 4`, `(lag_coord, phi_ref) =
  ("clip", "demand")` inside the scope and `("clip", "phi")` after it. And item L's survivor,
  injected, is caught by the post-scope readback and by NO value key. Falsified by any value key
  moving under the shipped table, any counter differing, or the survivor moving a value.
* **P3 — thread-local instruments are value-identical to Python under the default PARALLEL harness,
  with no lock.** Every counter a reader returns matches the oracle bit for bit. Falsified by any
  counter mismatch in a parallel `cargo test`.
* **P4 — the third value reaches the refusal, not a panic.** On an R79 machine with `gauge_k = 2`,
  `demand_gains` stops at `:20910`'s message. Falsified by any other panic or by a return.
* **P5 — `:21486`'s ulp band reproduces**: `+1`, `+2` refuse, `+3` marches. Falsified by any shift.
* **P6 — 0 ADD; `TripleHooks` ends the slice at 18 fields**; two core carriers added.
* **P7 — the step count is NOT scored.** § 5.32.7 (f) found it a bookkeeping choice. The PLANNED
  list, on AH's shape: **1** plumbing — the two modules, `R79`/`R80` tables (six re-aims), the
  `phi_ref`/`sm_air` carriers, the thread-local instruments, items E and H corrected,
  `slice_ai_cells.rs` with pointer identity both ways; **2** rung 79 §§ 1–4 (`_phi_residual`,
  `_phi_cap`, `_cap_fuel`, `_coord_at`, `coord_scan`, `coord_census`); **3** rung 79 §§ 5–5.2
  (`_with_probe`, `coord_march`, `_forced_cap`, `coord_forced`); **4** rung 80 whole; **5** the two
  ported suites plus the seven refusal gates written from the source, `_a_cap` ported WITH its
  outer guard (§ (vii)); **6** the oracle, carrying
  § (i)'s `demand_gains`-on-R79 arm; **7** the dispatch gates.

**Naming, decided now:** Python rung 79's public `coord_march` would share its bare Rust name with
`demand_coordinate::coord_march`, which is rung 74's `_coord_march` — and rung 80's `_split_march`
calls the rung-74 one. The rung-79 function keeps Python's name in `state_coordinate.rs`, and
`split_wall.rs` names the rung-74 one **fully qualified at every call**, with no `use` of either
bare name. `split_gains` follows the same rule (`cross_split::` vs `split_wall::`).

#### (x) DEFECTS IN THIS PRE-FLIGHT'S OWN INSTRUMENTS

1. **Probe 5's first run reported `0 / 64` against rung 74, with no counter** — which reads as *the
   break is inert*. The rerun (the advisor's four additions) found it REACHED 128 times and masked.
   **A zero diff is two findings, and only a counter splits them** — § 5.31.7's counting-pointer
   lesson, arriving as this slice's leading finding rather than after it.
2. **The booked-item enumeration first grepped only the PLAN.** The crate's own forward claims —
   twelve sites describing unwritten code — were outside it, and one of them (E) is false. The
   advisor's catch; § (iii) is the repaired enumeration.
3. **Three background runs were launched with `cmd //c start … "<python path>" …`, and `start` takes
   its first quoted argument as a WINDOW TITLE.** Two runs therefore executed the `.py` through the
   file association — `C:\WINDOWS\py.exe`, system CPython, not the PyPy venv — and the third
   (`-m pytest`) never started, sitting on a hidden error until it was killed by its captured PID.
   The two that ran were re-run under PyPy with `start ""`: **identical output**, so the accident
   became a free two-interpreter cross-check. Recorded in `windows-tooling-file-hazards`.
4. **The static nest probe reported ZERO at rungs 79/80, and the runtime probe found FOUR** — in
   `tests/test_rung79.py`, which the static census never reads (§ (vii)). It is § 5.32 (viii)
   defect 5 one slice on, from a new direction: there the census's set of CALLEES was named rather
   than derived; here its set of FILES was. The static probe has two further blind spots, neither
   hiding anything here: it only recognises `x.method(…)` calls (rung 79's inner `solve` / `slope`
   are called by bare name), and it keys freezers by NAME, so two inner functions called `solve`
   collapse to one. **The runtime counter is the measurement; the static census is the
   cross-check, and it is scoped to one file of the two the port ports.**
5. **Probe 4's `_with_coord` census counted CALLS and missed a REFERENCE.** `coord_march` hands
   `self._with_coord` to `_with_probe` as a value (`:21172`); an `ast.Call` walk cannot see it, and
   probe 7's runtime wrapper counted it (`from _with_probe:20997`). The miss changed no verdict —
   the site is rung 79's own body on a rung-79 machine — but a census of dispatch sites that
   ranges over call syntax is scoped by SYNTAX, which is § 5.31.6's *a scanner is a claim about
   syntax as well as scope*, again.

#### (xi) WHAT SLICE AI OWES BEFORE ITS STEP 1 — **nothing blocking**

Step 1's own deliverables carry items E and H (two doc corrections in shipped files), the § (iv)
decision stated in `state_coordinate.rs`'s module doc beside rung 78's contrary one, and `phi_ref`
as a string cell with the `== "phi"` test (§ (i)(b)). Booked forward to slice AJ: § (ii)'s *rungs
81–84 build rung-80 machines*.

