---
name: rust-port-phase7-preflight
description: "Phase 7's pre-flight — the deliverable was a SET the plan had claimed twice without ever enumerating, and both claims were wrong in different directions"
metadata: 
  node_type: memory
  type: project
  originSessionId: 93914f47-6512-4eec-8e05-0ec7b9ad9ce0
  modified: 2026-08-20T12:35:29.219Z
---

Phase 7 (the ladder: rungs 57–60 and 62–84, 27 rungs) had its **pre-flight authorised alone on
2026-08-20**, repeating phase 5's two-step exactly — measure, land, re-decide. Written to
`docs/plans/todo-rust-port.md` § 5.19; nine probes in `M:\claud_projects\temp\rust-phase7\`.
**Phase 7 itself is NOT authorised.**

**THE PRE-FLIGHT'S DELIVERABLE WAS A SET THE PLAN HAD CLAIMED TWICE AND NEVER COUNTED.** § 2's
architecture table named **eight** hooks; § 5.12's measured crossing list named **six**;
the intersection was **three**. Nobody had noticed the two lists disagreed, because they answer
different questions — § 2 asked "what is hot", § 5.12 asked "what crosses the phase boundary", and
**neither is "what are the table's fields"**. Enumerated: **38 names**, which also refutes § 3's
*"the trait is ~8–10 methods, not 40"* at its own lower bound. § 3's reasoning was sound and its
conclusion wrong for one reason worth carrying: it checked *defined exactly once* and never checked
*overridden at least once*. Generalises [[rust-port-slice-k]] — a scope list is only as good as an
enumeration — to a claim that was *measured*, just never on the right predicate.

**§ 2's TABLE ALSO CARRIED § 5.12's OWN BUG SINCE THE SPIKE.** `_instant_fuel` is one of its eight
hooks and **is not a hook**: its two definitions are on `SpoolTransient` and
`TwoSpoolFuelTransient`, which are **siblings**. That is the exact scoping error
[[rust-port-phase6-preflight]] made, caught, and wrote up — sitting undetected one section earlier
the whole time.

**TWO STRUCTURAL SHAPES § 2 DOES NOT KNOW ABOUT, AND BOTH WOULD HAVE COMPILED CLEAN.**

- **16 `super(LimitedBleedTransient, self)` sites** over rungs 65–75 pin **rung 62** regardless of
  leaf depth. § 2's whole spelling is `..R63` + "call the parent"; nine rungs down, *the parent*
  and *the pin* are different functions. And the pin is on the **function**, never the table —
  `r62_close_fuel(&R62, …)` compiles and silently freezes the ladder, which is the failure mode
  [[rust-port-ladder-architecture]] records the generics arrangement producing (0.018 % off, clean
  build).
- **19 dynamically-scoped fields behind 23 save/set/restore `try/finally` guards.** Rung 80's own
  docstring calls itself *"this family's THIRTEENTH reload"* — the pattern was documented in the
  source and invisible to the plan. **The classification is what mattered, and I nearly skipped
  it**: 11 are config, but **7 are assigned INSIDE a march** — they are the current RK4 state
  component, passed by dynamic scope so hook signatures need not change, and read by three hook
  cells. A field that is both config and state cannot ride `Config { f: v, ..*cfg }`. That
  measurement decides the hook parameter type, so it is upstream of every slice boundary; the
  advisor blocked the slice plan on it and was right.

**THE DECISION IT BOUGHT: one `Scope` parameter closes a question § 6 had booked as a COST.** § 6
said the narrowed-config-view for rungs 71/72/73's signature-absence tests might force per-hook
parameter types or a fallback to `include_str!`. It costs **zero** — the `Scope` struct phase 7
needs anyway simply does not carry `s_off`/`tau_rel`. *Two questions that look like separate design
choices can be one decision; ask whether they are before pricing either.*

**ALSO MEASURED:** § 2's *"Rust deletes `at_lever`/`_shared_rig` outright"* is refuted as written —
from rung 73 those forwarders **post-assign ten private attributes that are not constructor
parameters**, and *that* is what rung 80's docstring means by "THE EIGHTEENTH INSTANCE of the
trap". Template-method hazard is **0**, and getting there took **two detector defects in a row** on
the arm that measured zero (343 false sites, then 51, both from a "supplied" set blind to instance
attributes and then to dataclass annotations) — a zero from a blind instrument reads exactly like a
zero from a live one. § 6's runtime-introspection table named **four** tests; there are **eight**,
and one of the four it missed (`test_rung79.py:133`, `co_consts is not None`) **cannot fail**.

**SIZING, SAID OUT LOUD RATHER THAN DISCOVERED:** 15 362 source lines / 488 tests / 27 files =
**4.34× phase 6's source, 3.13× its tests**, and **219 of 488 tests carry `slow`** against phase
6's 9. Fifteen slices (V…AJ), **15–20 sessions against the table's 5–8**. The phase table's gate —
*"27/27 reduce-to-prior bit-exact"* — is weaker than every phase since 1; what the phase owes
(oracles, the 488 gates, dispatch gates) is now written into the row.

**Why:** a plan's own architecture section is written before anyone looks, so it is a hypothesis
even when it cites a measurement — the spike measured *call rates* honestly and the *set* never at
all. **How to apply:** when a plan states a set twice, diff the two statements before trusting
either; and when a pre-flight's job is "what are the fields", the predicate to enumerate is the one
the fields are defined by, not the one an earlier section happened to measure. Related:
[[rust-port-phase5-preflight]], [[rust-port-measure-before-registering]],
[[rust-port-oracle-cannot-see-a-missing-gate]].
