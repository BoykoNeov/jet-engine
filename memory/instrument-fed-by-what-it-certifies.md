---
name: instrument-fed-by-what-it-certifies
description: "The defect class that hit slices AC, AD and AE in a row — a gate, control or fixture whose input is supplied by the very thing it is meant to certify, so it passes by agreeing with itself"
metadata:
  node_type: memory
  type: feedback
---

**One defect class arrived three consecutive slices, and each time it survived to the LAST step.**

* **AC** — a gate that computed my own formula twice and compared the two.
* **AD** — a gate that compared the plant against the function that produced it.
* **AE step 5** — an *install proof* that passed because the machine held the pointer my own
  fixture had handed the builder three lines earlier; it failed only on the one row where the
  crate supplied its own.

Same shape every time: **the instrument is fed by the thing it certifies**, so green means
"consistent with itself" and nothing else. Every one was caught by a mutation sweep at the final
step — never by the pre-flight, even though each pre-flight has a § (x) *defects in this
pre-flight's own instruments* section that does work (AE caught five there before writing a line).
§ (x) is asking the wrong question.

**Why:** review reads gates as claims and fixtures as plumbing. A fixture line that supplies the
value under test, or re-implements the statement a mutation deletes, is invisible to that reading
and produces silent green in the one file whose whole purpose is coverage. Three occurrences is a
pattern, not luck.

**How to apply — promote it from this file into every pre-flight's § (x), as a named item:**

1. Of every planned gate, control, fixture and install proof, ask **what supplies the value under
   test?** If the answer is the code under test, or my own fixture, the gate is void before it is
   written.
2. Score the mutation sweep on **every binary in the slice, not just the new one**. That is what
   turned both of AE's blind spots from suspicions into numbers.
3. An install proof must be independent of what it certifies *by construction* — prefer an
   address/identity comparison against a source the fixture never touches, under an exhaustive
   destructuring so a new field is a compile error.

Related: [[rust-port-slice-ac-step7]], [[rust-port-slice-ad-step6]], [[rust-port-slice-ae-step5]],
[[rust-port-slice-w-step3]] (make the instrument prove it can SEE).
