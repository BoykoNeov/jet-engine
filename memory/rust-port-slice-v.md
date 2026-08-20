---
name: rust-port-slice-v
description: "Phase 7 authorised; slice V's pre-registration refuted the pre-flight's own reading of slice V — a census matches a SHAPE, so a bare permanent assignment was invisible to one built on try/finally"
metadata: 
  node_type: memory
  type: project
  originSessionId: 43899c52-0135-4f27-b65c-1efeb8c4ef1b
  modified: 2026-08-20T13:23:40.355Z
---

**PHASE 7 AUTHORISED 2026-08-20** ("start phase 7"), the authorisation
[[rust-port-phase7-preflight]] said was owed. Slice V (rungs 57–60, `ScheduledStatorTransient`)
is pre-registered in `docs/plans/todo-rust-port.md` § 5.20 off six probes measured first —
`probe_p7{l,m,n,o,p}.py` + `plugin_scoped_arm.py` in `M:\claud_projects\temp\rust-phase7\`.

**THE PRE-FLIGHT'S OWN SLICE-V READING WAS WRONG TWICE, AND BOTH ERRORS HAVE ONE CAUSE: A
CENSUS MATCHES A SHAPE, NOT A QUESTION.**

1. § 5.19 (x) called slice V *"the one slice that CHANGES A GATED SIGNATURE"* — the `&Scope` on
   `try_close`. § 5.19 (iv) had measured **which** cells read a dynamically-scoped state field
   and never **at which rung the first such reader sits**. Emitted: the earliest is rung 64
   (slice X); `try_close`'s is rung 65 (slice Y). Rungs 57–60 read **no** scoped field of either
   kind. `Scope` cannot even be *defined* at V — its fields are the nine state-kind fields, first
   read at rung 64 — so writing it here threads a dead, ungateable parameter through two slices.
   Generalises [[rust-port-guessed-census-bars]]: the measurement was right, the *predicate* was
   one question short.

2. **The shape the census could not see.** § 5.19 (iv) classified the phase's dynamic scope by
   one pattern: save a field, set it, restore in a `finally`. Rung 57's `_arm` assigns
   `self.map_lp`/`map_hp` and **never restores** — from inside `_close`/`_close_fuel`, which in
   Rust are shipped `&self` cells with a live `R40` table and two dispatch gates. Invisible to a
   `try/finally` detector. This, not `Scope`, is slice V's structural content.

**THE SOURCE STATES THE HAZARD TWICE AND NEVER CONNECTS THE TWO.** `_arm`: *"a pure function of
(nu_L, nu_H, Tt2) — no history, no latch"*. `v_of`: readers avoid `self.map_*` *"which `_arm`
leaves at whatever the LAST sub-step happened to be"*. Both hold only if no reader outside the
close path OBSERVES the staleness — **reachability, not purity**, which is [[rust-port-slice-i]]'s
lesson on a pair of docstrings instead of on a handler. Neither docstring makes that claim, and
[[rust-port-slice-l-step4]] is why a shipped-source claim is not evidence.

**MEASURED BY MAKING PYTHON BEHAVE LIKE THE PORT.** The natural port is a locally-armed core, so
Python was patched to restore the maps after each close and the two modes diffed: **920 262
close calls, 208 125 leaving `map_lp` mutated, 59/59 gates green BOTH ways** — and readers after
a march differ by up to **15.4 % on `margin_min_lp`**, the transient LP surge margin, which is
the currency rung 57's headline is stated in. Third suite in this port found blind to a large
error, and the first where the blind spot is the **object's own state** rather than a reader's
coordinate. The constant-setting arm differs by **exactly zero** (rung 53's constant is applied
in `__init__`; only a schedule reaches `_arm`), so no reduce gate can witness it either.

**THE MARCH ITSELF IS BIT-IDENTICAL, AND THAT IS ALGEBRA, NOT AN UNREACHED PATH** — the crucial
check, because a zero from an unexercised path reads like a zero from an inert one. The stale map
**is** read 723 times mid-march (`_instant_tail` 687, `_powers` 36), every read is `eta_t_at`,
`with_vsv` sets only `vsv`, and `eta_t_at` reads `a_t`. **`vsv` cannot reach `a_t`**, so the two
shipped cells are invariant *by construction*. A field can be stale on 723 reads and inert on all
of them: staleness bites only through the CHANNELS the mutation drives.

**AND THE CARRIER'S PRICE WAS MEASURED BEFORE IT WAS BUILT**, because stating one without a price
is the pre-flight's own defect 2 recurring. `Cell<ComponentMap>` is right (Python's shape exactly,
`ComponentMap` is `Copy`) but `map_lp`/`map_hp` are `pub` fields on **`TwoSpoolMapCore` — rung 39,
PHASE 5** — so it reaches **43 `src` sites over six files in three phases** plus 16 in tests.
That is a third distinct reason slice V is the risk, and none of the three is § 5.19's.

**ALSO DECIDED AT STEP 1:** the two `⚠` notes in `fuel_transient.rs` booked `try_close_fuel` /
`try_surge_fuel` into `TwoSpoolTransientHooks` — **wrong table.** That one is carried on
`TwoSpoolTransientCore`; a cell typed `fn(&FuelTransientCore, …)` there would make rung 40's table
name rung 43's type and hand every rung-40 object a cell it can never call. A new
`FuelTransientHooks` + `R43` at rung 43's own composition level, as `stator.rs` carries two tables
rather than merging them. Both bodies moved out of the `impl` **verbatim** — verified by
normalising and diffing against `git show HEAD`, not by eye — so step 1a changes **zero executable
lines**.

**Why:** every error here is a correct measurement answering a slightly different question than
the one that mattered. **How to apply:** when a plan hands you a classification, ask what shape
its detector matched and what a *different* shape would look like; and when a plan says a slice
owes a cost, check *when* the first thing that needs it appears before paying it.
Related: [[rust-port-measure-before-registering]], [[rust-port-slice-u-step4]],
[[rust-port-oracle-cannot-see-a-missing-gate]].
