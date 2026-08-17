---
name: rust-port-slice-n-step2
description: "The value dump passed first try and taught nothing — every finding came from the gates BESIDE it, and three of the four were my own instruments measuring nothing"
metadata: 
  node_type: memory
  type: project
  originSessionId: 7980985e-f9c7-444f-b487-a7a57da0c8b7
  modified: 2026-08-17T09:53:21.371Z
---

Slice N step 2 (`stage.rs` — rung 55/56's `StageStack`) shipped 2026-08-17. The oracle dump was
**1 337 keys bit-exact on the first run**. That is the least informative part of the step.

**1. A DEAD GUARD'S THRESHOLD IS WORTH MORE THAN ITS COUNT.** The plan recorded `_P_FLOOR` as
*0 firings in 521 649 marches* — a claim about a sweep. The floor it shadows (`_T_FLOOR`) is
applied to `tau_k` *before* the guarded value is formed, so the second guard can only fire when
`e > (1−P_FLOOR)/(1−T_FLOOR) = 1.001` **exactly** — a threshold in the two constants alone,
independent of every input. **How to apply:** when a guard measures dead, look for what makes it
dead before quoting the count; a derived threshold generalises and a sweep number does not.

**2. THREE INSTRUMENTS I WROTE MEASURED NOTHING, AND EACH WAS FOUND ONLY BY RUNNING IT.**
* The floor grid probed `march(0.9, 0.1)` on the guess that a bisection's LOW bracket end is the
  non-physical one. It is the HIGH one — that cell clamps **nothing**, while `march(8, 2)` clamps
  7 of 8 stages. Shipped as first written, the whole clamp branch would have read as covered.
* The argmin-tie gate asserted the tie on a cell carrying a MOVED stator, where the march is not
  at the stack's design point — refuted by its own printed numbers.
* The `Split::Tau` arm's doc comment claimed the grid measured the file's one power-spelling
  choice. It nearly didn't: the spellings differ on 34.8 % of a 109 650-cell scan, but at the
  cell I had shipped only 2 of 9 rows separated them by one bit — and at the neighbouring `K = 4`
  **none did**. *A divergence rate over a swept grid is not the rate at the cell you shipped.*
  Same family as [[rust-port-slice-j]].

**3. WRITE THE CENSUS COUNTERS INTO THE SHIPPED CODE AT THE STEP THAT SHIPS THE CODE.** Three of
the four census bars a later step owes can only be observed from inside these functions. Adding
them later means editing code the next step is already built on — [[rust-port-slice-l-step4]]'s
ripple. They paid immediately (48 / 48 bisection passes, 51 marches per solve).

**4. A LEDGER YOU HAVE JUST CLEARED IS THE EASIEST ONE TO DIRTY AGAIN.** Step 1's whole finding
was one stray skipped test; step 2 put one straight back — a ```` ```ignore ```` doc-block in a
crate with zero doc-tests and 42 ```` ```text ```` ones. Caught only by reading the *ignored*
column of a run whose exit code was 0. Mirror of [[rust-port-slice-n-step1]].

**5. THE REVIEW CAUGHT WHAT THE GREEN RUN COULD NOT: a two-armed fallible twin with NEITHER arm
executed.** The step's own pre-registered headline was that this function needs exactly two abort
reasons, and no test reached either. **How to apply:** when a step introduces error paths, gate
one and *book* the other; an untested arm nobody wrote down is
[[rust-port-documented-gate-that-doesnt-exist]].
