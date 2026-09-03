---
name: rust-port-slice-ae-step4
description: "Slice AE step 4 — the one aggregate detector that fired, I tried to exempt with a structural argument that cancels arithmetically; and a success line that had been counting golden reads as comparisons"
metadata:
  node_type: memory
  type: project
---

Slice AE step 4 (rung 73, the oracle — `rust/tests/slice_ae_oracle.rs`, 7 gates over 76 770 keys).
Plan § 5.29.4. See [[rust-port-status]], [[rust-port-slice-ae-step3]].

**THE LESSON: when the only detector that fired is an aggregate, the reflex is to explain why it
cannot match — and my explanation was arithmetically false.** `M/n_pos_zero` counts `+0.0` slots
over every key before section M. The port read 8 164, CPython's golden 8 166. I argued it was an
unavoidable hybrid: the census mixes keys the port computes (PyPy arithmetic) with keys it reads
out of the arm's golden, so it "cannot equal either pure number", therefore exempt. **Work the sum
and the hybrid half cancels exactly.** The read half contributes the same term to both the port's
census and CPython's own, so the residue is a statement about the self-computed half ALONE: a
deficit of exactly 2, which resolves by name to `E/row/6/pole_72` and `F/row/15/pole_72`, each
carrying `1.776e-16` where CPython has an exact `0.0` — **both already sitting in the drift list I
had not yet read**, because the flips assert aborts before drifts are ever printed. Exempting it
would have suppressed the only signal I had.

**Why to apply it:** an aggregate is the one kind of key whose disagreement invites a story instead
of a lookup, because it has no single owner to point at. That is exactly backwards — an aggregate
is a detector, and a detector that fires is a request to go read what it detected.

**How to apply it:** never admit an aggregate to an exemption until the things it aggregates are
named. Before that, make the instrument capable of showing them: `finish` asserted in severity
order and each assert printed only its own set (and only the first 12 of it), so on a run with two
kinds of disagreement the later sets were unreachable. The repair is an env-gated dump of all four
sets written BEFORE the first assert — reading, never a change to an assert, and never a reordering
of them, since the order encodes severity and reordering reads later as tuning the gate to pass.

## The green was 71 044 checks and the line said 76 770

`input_*` records a key in `seen` without ever comparing it. `seen.len()` was being printed as
"values compared", so **5 726 declared-grid inputs — sections J and K's grid, stride, replay
coordinates and path counts — were being counted as checks.** An input cannot disagree with the
file it was read from. The line now reports compared / read / total separately. Same shape as
[[rust-port-slice-z-step4]]: a count that names one population and measures another.

## A fixture typed from memory, and the error the compiler could not catch

The oracle had been written and committed **without ever being compiled**. Five errors, four loud
(`ComponentMap::new`, a `TwoSpoolLosses` missing two fields, `QuadGains` imported from the wrong
module, `h_pr` for `hpr`). **The fifth was silent: the gas omitted `r_c` and `r_t`**, which
`GasSpec::default()` supplies — it would have compiled, run a DIFFERENT GAS, and disagreed with
PyPy on every key at once, which reads like a broken port and not a broken fixture. All six
spellings are now copied verbatim from the green `rust/tests/rung73.rs` rather than recalled.
**A sibling that already passes is the fixture's source of truth; memory is not.**

## What went right, kept because each was measured

* **The port needed ZERO fixes.** Bit-exact against PyPy on all 71 044 compared keys, first run
  that ever compiled. Every defect this step found was in the instrument.
* **P9 CONFIRMED whole** — 683 names AND the composition (450 self-computed + 233 `L/cp4/*/out` +
  0 from `L/qr/*/out`) AND all seven section counts. Its sharpest clause carried the most: 698
  golden differences under `L/qr/*/out` contribute **nothing**, because those coefficients are
  declared inputs, so fed CPython's own the port reproduces CPython's roots — **the quartic root
  finder is thereby MEASURED interpreter-portable**, leaving `_charpoly4`'s `sum()` as the sole
  origin from a second independent direction.
* **Golden-vs-golden was 891 names wider in L alone** (1 574 vs 683), which is
  [[rust-port-slice-ab-step4]]'s finding again with a mechanism this time.
* **The real/complex classification of a quartic root is interpreter-dependent.** 63 keys flip
  `+0.0`-ness between the goldens, netting the -9 that separates the two censuses: the 2
  `*/pole_72` above and **61 imaginary parts of quartic roots**. Driven wholly by the coefficients
  upstream, never by the solver.
* **The exemption was measured, then gated twice** — `finish` asserts it against a fresh
  measurement in both directions, and a second gate asserts it against P9's predicted table, so
  neither list can be quietly re-fitted to the other.
* **The shipped PyPy golden reproduces byte-for-byte**, and the 36-byte gap against a scratch copy
  is explained rather than bounded: that copy predates the two's-complement mask in the dumper's
  `d()`. Committing the goldens gives the git baseline step 2 lacked when a mutation sweep
  contaminated a backup at byte-identical size.
* **`the_reference_census_is_this_grids_own` compares the dump against itself and nothing else** —
  every value it reads is a golden. Now says so, and names where the port DOES independently
  re-derive the replay path (from its own `applied_clip`, against `k_path[p] / k_stride`).
* **I nearly re-committed § 5.29.3 (j)'s own defect**: launching the gate as
  `cargo test > out & echo exit=%ERRORLEVEL%`, where cmd expands `%ERRORLEVEL%` at parse time and
  returns `echo`'s status. Killed by the PID I had captured and relaunched so the exit code is
  cargo's own. **The step whose plan section records an unread exit code is the step most likely
  to produce another one** - and it produced one anyway. The relaunch made `$p.ExitCode` cargo's
  own and then **nothing ever wrote it to a file**; `$p` died with the launching shell and Windows
  keeps no status for a dead PID, so the pre-registered `exit` row is the ONE row of seven that
  reads NOT READ. The other six hit exactly (141 / 1 / 142 / 142 of 142 / 1 458 / 0). Exit 0 is
  DERIVABLE from 0 `error[E` + 0 `FAILED` + 142 of 142 ok, and a derivation is not a reading.
  **A status is measured when it is ON DISK: the same command that redirects stdout must write
  the code beside it.** Fixing the SOURCE of a number is not the same as fixing its CAPTURE.
* **Repairing one lesson opened another, FOURTH instance.** Admitting the aggregate to `EXEMPT`
  made the file's rule 3 (*"a discrete key that flips is never a rounding"*) false by one name, so
  the doc and the assert message described different properties - slice AC step 2's finding,
  created here by the exemption itself, so **the audit belongs to whoever WRITES an exemption**.
  The first repair then stated the exception in prose and enforced nothing: `raw()` tests `EXEMPT`
  **before** `discrete`, so a second discrete exempt name is absorbed on the branch above and the
  runtime could not have caught it. `Cmp::discrete_exempt` now records exempt-and-discrete keys and
  `finish()` asserts the set is that one name - and **the gate proves it can see by passing**,
  since a dead recorder yields the empty set and fails the equality.
* **The plan's own claim about the gate was a quote I had not read.** I wrote that the oracle's
  block shows `test result: ok. 7 passed`; my `sed` window had stopped two lines short, and the
  `grep -c` I leaned on returned **5** - five binaries have seven tests, so the count attributes
  none of them. It is at line 1 846, `finished in 110.80s`, unique among the five. Written three
  lines under my own sentence that a derivation is not a reading.
