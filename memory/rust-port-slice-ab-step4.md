---
name: rust-port-slice-ab-step4
description: Slice AB step 4 — a cross-interpreter exemption measured between the two DUMPS was 67 names wider than the one measured against the PORT
metadata: 
  node_type: memory
  type: project
  originSessionId: 57f6e146-1b56-46c8-a924-d490be2f24f2
  modified: 2026-08-28T09:11:55.347Z
---

Slice AB step 4 (rung 69's oracle, 2026-08-27) built `EXEMPT` — the key names the CPython arm may
differ on — the obvious way first: diff `slice_ab_pypy.tsv` against `slice_ab_cpython.tsv`. That
gives **261 names**. The Rust run measures **194**. The 67-name gap is every `c1`-rooted key in the
dump's section I, which replays the root finder on **coefficients read back from the golden** — so
in the CPython arm Rust is fed CPython's own diverging sum and the difference cancels.

Two more things the same step measured, both worth carrying:

* **The oracle's DECLARED EXTRA table found the only port defect** — `0.5 * z` in Python is a full
  complex product, not a scaling, so its cross term `0.0 * z.im` flips the sign of a zero real
  part. One key of 15 957. The suite's own grid never lands on it; the hand-chosen triples did.
  Step 2 had spelled the sign-of-zero decision out for the ADDITION on the line above and stopped —
  reasoning about one operation of an expression says nothing about the next.
* **CPython 3.14 changed what mixed float/complex arithmetic means on a signed zero** (real-operand
  fast paths in `__rsub__`/`__rmul__` that drop the zero cross-terms). 134 of the 194 exempt names
  are that, not the `sum()` the pre-registration predicted. The port is held to PyPy.

**Why:** an exemption is a licence to ignore a disagreement, so it must be measured against the
thing it licenses. A diff between two *reference* implementations answers a different question than
a diff between the *port* and one of them, and the two differ exactly where the harness feeds a
reference's own values back in. Declaring the wider set would have marked 67 keys allowed-to-differ
that must not, with only the both-directions equality assert standing between that and a silent
hole. [[rust-port-slice-z-step4]] is the same lesson one level in — count vs names; this is *whose
diff*.

**How to apply:** build the exempt list by running the gate and reading what IT measured, never by
diffing the two goldens. Keep the set-equality assertion in both directions so a name that stops
drifting fails too. And whenever a dump section reads any value back out of the golden as an
INPUT, say so on the section — that section's arm is testing something narrower than the others,
and the exemption will not match the dump diff there.

Related: [[rust-port-slice-ab-step3]], [[rust-port-slice-aa-steps2345]], [[golden-fingerprint-gate]].
