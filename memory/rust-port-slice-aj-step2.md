---
name: rust-port-slice-aj-step2
description: "Slice AJ step 2 (rung 81's five readers) — a reviewer's 'don't merge these two lookups' named a difference of FORM: on the shipped domain the two are algebraically one function, so both merge-injections survived by construction; derive the identity before booking a gate seat"
metadata:
  node_type: memory
  type: project
  originSessionId: c14405f8-ddd5-496b-b034-53ed9e781f29
  modified: 2026-09-29T13:56:06.321Z
---

Slice AJ step 2 COMPLETE 2026-09-29 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.34.2). `rust/src/authority_clock.rs` (815 lines): `central`, `criterion_at`, `authority_clock`,
`tau_f_inert`, `authority_mask`. Gate `rust/tests/slice_aj_clock.rs`, 5 readings bit for bit against
`rust/oracle/probe_slice_aj_step2.py` → `slice_aj_step2_pypy.tsv` (16 119 lines). Green first run.
Sweep 16 injections, 16 of 16 predicted. Probes in `W:\temp\claude\slice-aj-step2\`.
Next: step 3 = rung 82's nine readers in `threshold_law.rs`.

**THE LESSON: before keeping two code paths apart "because merging them would be a departure",
check whether they are the same function on the domain the code actually sees.** The advisor warned
not to merge `_demand_tau(lag, cap, w)` and `lag.tau(required, g)`. On a `demand` point
`required = mf_sched - cap` and `g = mf_sched - w`, so `required > g ⇔ w > cap` — the swap IS the
change of variables. I predicted both merge-injections would SURVIVE from that algebra, and they did.

**Why:** a distinction that no value can see costs a gate seat that would be vacuous, and a doc
sentence implying a value would move is a small false claim.

**How to apply:** when a warning says "these differ", derive whether they differ on the shipped
inputs; if they are identical there, keep Python's spelling, say in the doc that no gate can pin it,
and don't book an injection for it at the dispatch step.

Also:
- A flat `path<TAB>token` TSV (dict = `keys:N` + items in order) pins key ORDER and presence, not
  just values — used instead of spliced constants because 1 054 cells × 10 fields was too many.
- A deliberate extra reading (`demand-latched`) was the ONLY catcher of the branch-on-variant defect
  (J1). The fixture settings could not see it; pick probe calls by the branches they reach.
- P1 read early: rung 81 tracks the per-LINE model (≈4.2 Rust lines per Python body line).

Related: [[rust-port-slice-aj-step1]], [[rust-port-slice-aj-preflight]], [[rust-port-status]].
