---
name: rust-port-phase7-preflight
description: "Phase 7's pre-flight — the plan stated the same SET twice and nobody diffed the two; then every defect in the write-up came from hand-writing a table a probe could have emitted"
metadata: 
  node_type: memory
  type: project
  originSessionId: 93914f47-6512-4eec-8e05-0ec7b9ad9ce0
  modified: 2026-08-20T12:45:13.793Z
---

Phase 7 (the ladder: rungs 57–60 and 62–84, 27 rungs) had its **pre-flight authorised alone on
2026-08-20**, repeating phase 5's two-step exactly — measure, land, re-decide. Written to
`docs/plans/todo-rust-port.md` § 5.19; eleven probes in `M:\claud_projects\temp\rust-phase7\`.
**Phase 7 itself is NOT authorised.**

**THE PRE-FLIGHT'S DELIVERABLE WAS A SET THE PLAN HAD CLAIMED TWICE AND NEVER COUNTED.** § 2's
architecture table named **eight** hooks; § 5.12's measured crossing list named **six**; the
intersection was **three**. Nobody had noticed the two lists disagreed, because they answer
different questions — § 2 asked "what is hot", § 5.12 asked "what crosses the phase boundary", and
**neither is "what are the table's fields"**. Enumerated: **38 names = 28 new cells + 8 already
shipped + 2 Rust deletes**, a **36-field table**. That also refutes § 3's *"the trait is ~8–10
methods, not 40"* at its own lower bound, for one reason worth carrying: § 3 checked *defined
exactly once* and never checked *overridden at least once*. Generalises [[rust-port-slice-k]] — a
scope list is only as good as an enumeration — to a claim that was *measured*, just never on the
right predicate.

**§ 2's TABLE ALSO CARRIED § 5.12's OWN BUG SINCE THE SPIKE.** `_instant_fuel` is one of its eight
hooks and **is not a hook**: its two definitions are on `SpoolTransient` and
`TwoSpoolFuelTransient`, which are **siblings**. That is the exact scoping error
[[rust-port-phase6-preflight]] made, caught, and wrote up — sitting undetected one section earlier
the whole time.

**TWO STRUCTURAL SHAPES § 2 DOES NOT KNOW ABOUT, AND BOTH WOULD HAVE COMPILED CLEAN.**

- **16 `super(LimitedBleedTransient, self)` sites** over rungs 65–75 pin **rung 62** regardless of
  leaf depth. § 2's whole spelling is `..R63` + "call the parent"; nine rungs down, *the parent* and
  *the pin* are different functions. And the pin is on the **function**, never the table —
  `r62_close_fuel(&R62, …)` compiles and silently freezes the ladder, which is the failure mode
  [[rust-port-ladder-architecture]] records the generics arrangement producing (0.018 % off, clean
  build).
- **23 dynamically-scoped fields behind 52 save/set/restore `try/finally` guards.** Rung 80's own
  docstring calls itself *"this family's THIRTEENTH reload"* — documented in the source, invisible
  to the plan. **The classification is what mattered, and I nearly skipped it**: 12 are config, but
  **9 are assigned INSIDE a march** — the current RK4 state component, passed by dynamic scope so
  hook signatures need not change, and read by hook cells. A field that is both config and state
  cannot ride `Config { f: v, ..*cfg }`. That measurement decides the hook parameter type, so it is
  upstream of every slice boundary; the advisor blocked the slice plan on it and was right.

**THE DECISION IT BOUGHT: the port already had TWO answers to this shape and neither was named.**
`rho` (r40) was ported by threading a parameter into its one reader; `bleed` (r42) by giving the
reader `&mut self` and restoring in straight-line code. Both work because those readers sit at the
TOP of the chain. So: the 12 config-kind fields take the `&mut self` precedent (no table change at
all), the 9 state-kind ones take a `Scope` parameter on **7 of the 36 cells**. That closes § 6's
narrowed-config-view question — **but not at zero cost**: one of the seven is
`TwoSpoolTransientHooks::try_close`, a *shipped* phase-6 cell with a live table and a dispatch gate,
so slice V changes a gated signature. *Two questions that look like separate design choices can be
one decision; ask whether they are before pricing either — and then check the price is really zero.*

**ALSO MEASURED:** § 2's *"Rust deletes at_lever/_shared_rig outright"* is refuted as written — from
rung 73 those forwarders **post-assign ten private attributes that are not constructor parameters**,
which is what rung 80's docstring means by "THE EIGHTEENTH INSTANCE of the trap". Template-method
hazard is **0**, reached through **two detector defects in a row** on the arm that measured zero
(343 false sites, then 51, from a "supplied" set blind first to instance attributes and then to
dataclass annotations) — a zero from a blind instrument reads exactly like a zero from a live one.
§ 6's runtime-introspection table names **four** tests; there are **eight**, and one it missed
(`test_rung79.py:133`, `co_consts is not None`) **cannot fail**.

**SIZING:** 15 362 source lines / 548 collected tests / 27 files = **4.34× phase 6's source, 3.49×
its tests**, with **263 of 548 carrying `slow`** against phase 6's 10 of 157. Fifteen slices (V…AJ),
**15–20 sessions against the table's 5–8**. The phase table's gate — *"27/27 reduce-to-prior
bit-exact"* — is a spine, not a gate; what the phase owes (oracles, the 548 gates, dispatch gates)
is now in the row.

**AND THE SECOND LESSON, WHICH COST FOUR DEFECTS: EVERY ERROR IN THE WRITE-UP WAS A TABLE TYPED
FROM PROBE OUTPUT RATHER THAN EMITTED BY IT.** The measurements were all correct. The tables built
from them were not: a cell column listing `_closer` (defined once, so not a hook at all) and
`at_lever`/`at_stator` (the very names the section two screens above says Rust DELETES) — self-
contradictory, and the arithmetic gave it away first by summing to 31 against a stated 30; a
`Scope` cost stated as zero without checking the shipped tables; a field that fell between two
probes because one of them scanned only top-level statements and the guard was nested in a `for`
(repairing it took the census from 23 guards/19 fields to **52/23**); and a `slow` ratio counting
**decorators** and calling them tests ([[rungs72-77-march-audit]]'s *a counter is only as good as
the noun it counts*, on my own summary). **Where a probe PRINTED the answer the section was right;
where I read probe output and typed a table it was wrong four times out of four.** The rule taken
into the phase: **if a slice's table can be emitted, emit it** — § 5.10 step 4's *emit and compare,
do not restate*, applied to plan prose instead of to a census.

**Why:** a plan's architecture section is written before anyone looks, so it is a hypothesis even
when it cites a measurement — the spike measured *call rates* honestly and the *set* never at all.
**How to apply:** when a plan states a set twice, diff the two statements before trusting either;
when a pre-flight's job is "what are the fields", enumerate on the predicate the fields are defined
by, not the one an earlier section happened to measure; and never hand-transcribe a table you could
generate. Related: [[rust-port-phase5-preflight]], [[rust-port-measure-before-registering]],
[[rust-port-oracle-cannot-see-a-missing-gate]], [[rust-port-guessed-census-bars]].
