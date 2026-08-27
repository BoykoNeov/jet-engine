---
name: rust-port-slice-y-step4
description: "Slice Y step 4 — a green 35,994-key oracle says nothing about what it would CATCH, so inject; and the three defects it misses turn out to be exactly the three the pre-registration already owed"
metadata:
  node_type: memory
  type: project
---

Rung 65's oracle: 35,994 keys over nine sections, bit-exact against PyPy **and** CPython 3.14 on
the first run, zero tolerance tiers. Nothing was coarsened — the readers were TIMED first (3.4 s /
6.2 s / 0.2 s) and every section runs the suite's own step size, so the "disclose your coarsening"
promise is discharged by not needing it.

**A GREEN ORACLE MEASURES AGREEMENT, NOT COVERAGE.** Six injections into the shipped Rust, each
re-running both arms with a *did-it-move* key count: three go red (a reader handing back the
command instead of the recorded state — 394 keys; a per-march override being ignored — 1,280 keys;
a march losing its extra state — aborts at the port's own assert before key one), and **three stay
green at exactly 0 keys moved**. Those three are the two mirror-image dead branches the
pre-registration had already named and the save-and-restore-previous guard whose nesting depth was
measured at 1. So the residue the value keys cannot reach is a CLOSED, PRE-REGISTERED set, not an
unknown — which is the strongest result the census could have returned.

**A FOURTH ZERO CAME FROM ASKING A READER FOR ITS DEGENERATE CASE, NOT FROM AN INJECTION.** One
reported field is NaN when a sub-case is empty, and Rust's `f64::max` discards a NaN operand where
Python's `max` does not — two different functions. Measured: the empty sub-case never occurs on
this grid, so no value key can tell the spellings apart. (My first description of *how* they differ
was itself a guess and was wrong — see [[rust-port-slice-y-step5]], which had to run the cases in
an interpreter to get it right.)

**Two hazards paid from memory instead of from a debugging session:** the gas-constant spelling was
grepped for before the dump ever ran, and the key-count bar in the loader was taken from the
measured 35,994 rather than copied from the previous slice's 1,800 (which would have passed on 5 %
of the file).

**PowerShell's `1>` writes a UTF-8 BOM.** The first golden had `ef bb bf` in front of the `#` on
line 1, which makes a `starts_with('#')` header filter false and parses the header as data. Caught
by `head -c 3` against a committed golden before Rust ever read it. Generate dumps through a POSIX
shell.

**Why:** bit-exactness and coverage are independent properties, and the port keeps finding that the
obvious green result is silent about the one that matters.

**How to apply:** after an oracle goes green, corrupt the cells it is supposed to be watching, one
at a time, and record how many keys each moves. Predict each row first; a row that agrees with its
prediction is evidence, a row that does not is the finding. Report an unexplained result (a run
that reddens with no key count) by re-running it alone and reading it, never by filling it in from
the shape of the other rows.

Related: [[rust-port-slice-y-step3]], [[rust-port-slice-w-step3]], [[rust-port-guessed-census-bars]],
[[windows-tooling-file-hazards]], [[rust-port-slice-x-step3]].
