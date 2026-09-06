---
name: rust-port-slice-ag-preflight
description: "Slice AG (rungs 75/76) pre-flight — a claim can be true of one population and false of the one beside it, and the section contradicted itself before anyone looked"
metadata: 
  node_type: memory
  type: project
  originSessionId: b06dd9a2-1415-4f04-bf1d-8a0cc913c936
  modified: 2026-09-06T14:43:10.452Z
---

Slice AG's pre-flight (plan § 5.31, rungs 75/76, `AntiWindupTransient` + `SensedCapTransient`).
Fourteen probes; scripts and `measurements.md` in `M:\claud_projects\temp\slice-ag-preflight\`.

**THE PROCESS LESSON: *what supplies the value?* has a sibling — *what POPULATION is the value
drawn from, and did the claim sweep that one or a different one?*** I drafted a verdict that a
shipped comment in `tests/test_rung76.py` was false in both clauses, from a margin sweep run
against the MARCH. The same section, three paragraphs earlier, already carried a reading of the
READERS at the identical arm and margin that answered the opposite way (60 of 60 against 0 of
1 366). **The contradiction was inside my own section before anyone looked for it**, and I was
about to delete a true statement. Run properly, `cap_gains` at that arm goes 2/8 → 4/8 → 8/8 →
**0 of 8 live cells** over margins 0.05/0.10/0.20/0.40, so the comment's `~0.20` cliff is real on
the population it was written about and absent on the one standing next to it. The repair became
ADDITIVE — 12 lines, 0 removed, both populations named — instead of a deletion.

The advisor caught this by reading my own two numbers against each other. Extends
[[instrument-fed-by-what-it-certifies]] and AF's standing item; see
[[rust-port-slice-af-preflight]].

**Three more, worth carrying:**

* **A spy that evaluates the thing it counts double-counts it.** My `_cap_fuel` spy computed what
  the sensed branch *would* say in order to compare it, and that computation is a call — so the
  dispatch count came back at exactly 2× on a method with one call site. Caught by the ratio being
  an exact integer, not by anything failing.
* **A `sum()` census by CONTENT, not by call site, named the CPython exemption to the exact key
  in advance.** Nine `sum()` calls across the two classes: five sum a literal 1, two sum three
  floats, exactly two sum a 341-long trajectory. Measured across PyPy and CPython, **2 of 83 273
  keys differ and they are precisely those two** (`cap_bill/fuel_int/0` and `/1`). This is AF's
  falsified-`sum()`-census lesson applied one slice later and paying off.
* **A fallibility claim verified at ONE call site is not verified.** The crate declares
  `windup_tau` infallible because *no caller catches*; there are three call sites and the third is
  inside a `try/except AssertionError`, so the premise is FALSE. The conclusion survives for a
  different, measured reason: the catching caller computes `math.log(tau_t/(tau+tau_t))` before
  the march, so every input that would trip the refusal raises `ValueError`/`ZeroDivisionError`
  first, and what that `except` actually absorbs is a different assert entirely. Slice L's rule is
  per call site — so sweep the sites, not the argument.

**And one defect of my own tooling, not the port's:** a patch script sliced the plan between
`s.index(A)` and `s.index(B)` where `B`'s heading text also occurs in an earlier slice's section,
so it duplicated ~8 600 lines. `git diff --stat` caught it. **In a 21 000-line append-only plan,
never anchor a splice on a heading pattern that repeats per slice** — rewrite the whole section
and append it, or anchor on something unique to the section.

**What the pre-flight settled, in one line each:** the phase table's `0 ADD` for AG is the first
of the last four cell counts to be right as written; AF's owed `_with_*` re-run comes back
negative (no third field-divergent name); the inherited `sensed_cap` unreachability closes by
arming the accel arm but the value break is decided by the min-select one level down, so the gate
must be three-sided; and the step count is pre-registered at **7** against the method-count law's
**6**, which makes the count itself the test.
