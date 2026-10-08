# Slice AG step 4 — mutation sweep, PREDICTIONS TYPED BEFORE ANY RUN

Instrument: patch `rust/src/sensed_cap.rs`, rebuild, re-run `slice_ag_step4_drive`, diff its
1 422 keys against the PyPy golden `py.tsv`. Source SHA-256 verified back to pristine after each.

| # | injection | predicted | reason typed in advance |
|---|---|---|---|
| 1 | `cap_march`'s leg takes `accel: None` | **KILLED**, large | the arm is the whole structural difference; the sensed arm collapses onto the solve arm AND the march runs uncapped |
| 2 | `cap_march` drops `cap_law.set(cap_law)` | **KILLED** | the rig keeps the caller's `solve`, so the sensed arm returns solve's floats — the `0 ADD` blindness by the other route |
| 3 | `cap_march` drops `ic_cap.set(...)` | **SURVIVES** | nothing in this drive raises the cap; only `contraction_law` does, and it is step 3's reader |
| 4 | `accel_for` calls `R75_TRIPLE.shared_rig` directly instead of through `triple_hooks()` | **SURVIVES** | a schedule is read off EQUILIBRIA and no cap law touches those — the doc says it is called through the table for the RULE, and this measures that claim rather than asserting it |
| 5 | `accel_for` passes `n = 12` | **KILLED** | `n_rows` plus all 26 table rows per arm |
| 6 | `accel_for` passes `tau_rel: tau_f` (not `3·tau_f`) | **SURVIVES** | the asymmetric lag is a MARCH parameter; equilibria do not see it |
| 7 | `accel_for` passes `tau_att: taus.1` (the governor's clock) | **SURVIVES** | same reason as 6, and the grid is uniform anyway — a hazard for step 6, not a defect here |
| 8 | `c_at`'s fold becomes `w.max(1e-9)` | **SURVIVES** | the hinge is at `1e-9` and the plant refuses every fuel below ~1e-2, so the guarded branch is unreachable — the drive already measured all four sub-hinge cases as aborts |
| 9 | `c_at` differences one-sidedly, `(cap(w+dw) − cap(w))/dw` | **KILLED** | every `C/*` value |
| 10 | `c_at`'s two state guards SWAPPED (`b_state = v`, `v_state = q`) | **KILLED** | the `b_state`/`v_state` boundary — the plant solves against a state it never had |
| 11 | `CapScope::drop` restores `CAP_LAW_SOLVE` instead of `prev` | **KILLED**, exactly 2 keys | `D/scope/after` and `D/scope/inside_outer_again`; the outer scope is `sensed` |
| 12 | `CapScope::drop` restores nothing | **KILLED**, exactly 2 keys | same two keys, same reason |
| 13 | `cap_march` passes `tau_rel: tau_f` | **KILLED** | rung 52's release rate IS a march parameter |
| 14 | `c_at` drops the `/ pi_b` (uses `pt4` as `pt3`) | **KILLED** | every `C/*` value |
