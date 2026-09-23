---
name: rust-port-slice-ah-step5
description: "Slice AH step 5 — rung 78's last bodies ported bit-exact, and the PREVIOUS step had shipped the citation guard red while its own section said 5 of 5: it ran the guard, then kept editing docs"
metadata:
  type: project
---

Rung 78 §§ 4–5 ported (`gauge_vs_device`, `phi_at` under the new `PhiAtFreeze`, `c_on_frozen`,
`gauge_march`); `residual_gauge.rs` 1 165 → 1 691, module total 2 899 (inside P1's band, but
reached by DOC lines, not the +320 of code step 4 extrapolated). `tests/slice_ah_march.rs`
234 lines / 4 gates. Every rung-78 body is now ported. Plan § 5.32.5, 2026-09-23.

**The lesson: run a count guard AFTER the step's LAST doc edit, not merely "inside the step".**
Step 5's citation census read 205 against step 4's blessed 196; only 3 sites were step 5's. A
snapshot of step 4's own commit reads 202, so step 4 re-blessed, then added six more citing doc
lines, and pushed the count gate RED while § 5.32.4 (g) said "5 of 5". Step 3's paragraph had
said "run it inside the step" — step 4 obeyed that and still shipped red. No citation was ever
wrong (all nine hit already-pinned lines); only the count.

**Why:** a guard run is a measurement of the tree AT THAT MOMENT; every later edit voids it,
and doc edits feel too harmless to re-run for.
**How to apply:** make the citation guard (and any census) the last command before `git add`,
after all doc/comment edits — and when a census disagrees, snapshot the previous commit
(`git archive` + `tar --force-local` on Windows) to split the arrears from this step's growth.

Also measured, each one lesson already on file:
- Bit-exact on 129 of 129 returned values (float.hex vs a Python run), not just the suite's
  one-sided bars.
- `PhiAtFreeze` is the first freeze guard with a REACHABLE nest (20 per call), so § 5.26 (iii)
  item B, owed as a panic message, landed as its DOC. It is also the first guard restoring `prev`
  where Python clobbers to `None`; only a hand-built nest can see the policy.
- Injection sweep 7 defects: 5 caught, 2 (closed block drops the stator freeze; schedule built
  under a gauge) measured as exact no-ops on all 129 values — recorded, not gated.
- § 5's docstring claim that `c` drifts so `k·c` sweeps a range is FALSE here (7e-9 drift);
  only `hits`/`binds` carry information. See [[rust-port-slice-ah-step4]],
  [[rust-port-ported-test-vacuity]], [[rust-port-documented-gate-that-doesnt-exist]].
