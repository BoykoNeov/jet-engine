---
name: rust-port-slice-ah-step6b-oracle
description: "Slice AH oracle (6b) — a CORRECT zero-exemption CPython arm makes AG's 'the goldens differ somewhere' provenance gate unsatisfiable; carry provenance in a dedicated sentinel key"
metadata:
  type: project
---

Slice AH step 6 (b), 2026-09-26: `dump_slice_ah.py` + `slice_ah_oracle.rs`, 29 288 keys,
bit-identical to PyPy and CPython with NO exemption. Plan § 5.32.6 (i)/(j).

**The lesson: a gate that infers a property from a SIDE EFFECT stops working when the side effect
correctly goes to zero.** AG's `the_two_goldens_are_not_the_same_file` proved the CPython golden
was a real run by asserting SOME key differed, which was true only because AG's arithmetic had 406
compensated-`sum` divergences. Here the call graph reaches no float `sum` at all, so the correct
answer is zero differences, which that gate would read as "copied file". Fixed with a key built to
carry the fact: `_interp/sum_probe` = bits of `sum([1e16, 1.0, -1e16])` per interpreter (CPython
1.0, PyPy 0.0), excluded from comparisons by name and asserted per arm.

**Why:** an instrument that works only while an unrelated quantity is nonzero has an expiry date
set by that quantity, and a better port (or a cleaner slice) is exactly what expires it.
**How to apply:** when a gate's claim is about PROVENANCE or LIVENESS, give it its own witness
(sentinel key, counter), never "something else differed". Same step, same shape: section P's
gauged trajectory is bit-identical to the ungauged one (the gauge is inert), so dropping the gauge
moved 0 keys until a `gauged_hits` counter was emitted beside it, and `gauged_binds` (exactly 0)
still cannot see it. Also: the CPython prediction was written from a RUNTIME `builtins.sum`
census over the drive, not from the pre-flight's class-scoped reading, which is AG's P2 miss avoided.
See [[rust-port-slice-ah-step6]], [[instrument-fed-by-what-it-certifies]], [[rust-port-slice-t-step1]].
