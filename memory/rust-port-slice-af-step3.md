---
name: rust-port-slice-af-step3
description: "Slice AF step 3 — a gate read a split produced by the PREVIOUS step's code as its own clamp working, when that clamp fires 0 of 340 times; and the first point variant whose inherited keys change SIGN"
metadata: 
  node_type: memory
  type: project
  originSessionId: b1bef680-e0b2-46ed-8351-f2f5ecf6e9e2
  modified: 2026-09-04T15:50:14.411Z
---

Slice AF step 3 (rung 74, `DemandCoordinateTransient`). Plan § 5.30.3. The six-state demand march
`_integrate_fuel_demand`, its joint initial condition, and `PointExtra::Demand`. See
[[rust-port-status]], [[rust-port-slice-af-step2]], [[rust-port-slice-t-step1]],
[[instrument-fed-by-what-it-certifies]].

**THE LESSON, and it extends the standing question a second time.** [[rust-port-slice-af-step2]]
added *what supplies the POINT the test is driven at?* to *what supplies the value under test?*
This step adds the third: **what supplies the DIFFERENCE the gate reads?** The gate for this
step's one blocking risk — *do not copy the parent's floor onto the new state* — asserted that the
new clamp BITES, reading a 20-vs-0 split between two arms as its own code working. It was not:
the split is produced by a body ported at the PREVIOUS step, which caps the target upstream, and
the clamp under test **fires 0 of 340 times**. A gate whose content is *these two arms differ* has
to ask which of the two arms' changes produced the difference before it can name what it holds.

Caught by measuring the thing the gate was about to assert, in the source language, before
believing it. The repaired gate holds what is actually live — the `if latched` GUARD (dropping it
kills) and the initial condition's own stop — and BOOKS the clamp's arithmetic as a dead site,
naming the one operating shape that would reach it (a decreasing schedule) and the second shipped
refusal that closes that route.

## The first point variant whose inherited keys change SIGN

A new trajectory-point variant means widening every reader that matches on the old ones — 32 sites
in `src`, of which the compiler names only 7 and **25 sit behind a `_ =>` wildcard**. That much is
the previous slice's recorded audit, one variant on. What is new: **every earlier variant floored
its clip states, so eight inherited arms read `x > 0` as *is this leg live*. This one records
unfloored projections, and the negative region is reachable** — 21 of 341 points on one arm and 0
on its sibling. The source language reads the dict key and gets the negative number, so admitting
is faithful and refusing would be stricter than the source; but the two arms now hand the same
reader two different SIGN SETS. **Widening a match arm is not the whole job — ask what the new
variant's DOMAIN is, at each arm, not just whether the key is present.**

## Two gate defects the mutation sweep found, both of the same family

* **A gate that could not separate two live terms.** It asserted a maximum over a trajectory, and
  the initial condition shifts the very first point by exactly the amount the derivative term does
  — so deleting the derivative term still satisfied it. Repaired by reading the LAST point: with
  thirty-four time constants between, a shift still present at the end can only be held by a term
  in the derivative. **A max over a trajectory is not an isolation.**
* **`is_finite()` is satisfied by a zero fallback.** The reader gate asserted the widened readers
  ANSWER; a mutation making one answer `(0.0, 0.0)` passed, and passed the `>= 0` beside it.
  Repaired by comparing each reader's return against the point's own destructured keys — a
  different code path from the reader — plus an assertion that the compared values are non-zero.
  [[rust-port-slice-t-step1]]'s exact-zero class met FORWARDS instead of after the fact.

## A BOOKED BLINDNESS THAT WAS FALSE — and it shipped in the first commit

The reader gate carried a comment saying one widened reader *could not* be value-gated on this
arm, because the actuator it reads sits within `1.8e-15` of its design setting. **It was asserted,
not measured, and it was wrong.** Typed as a mutation — the widened arm replaced by a `0.0`
fallback — it is KILLED, at `v = -3.544154491931811e-17`: an exact comparison sees a value that
small as readily as it sees `1e-2`, and "essentially zero" is not zero.

**A claimed blind spot is a claim, and it needs a mutation like any other.** Two of the three
gate defects this step already repaired were the same shape (an `is_finite()` a zero satisfies; a
max over a trajectory that could not isolate a derivative term), which makes it the step's own
recurring defect rather than an incident. It is also step 1's recorded lesson with the survivor
imaginary too: there, a CORRECT survivor was defended by a FALSE comment and the comment shipped;
here the comment shipped and there was no survivor at all.

Caught by a reviewer asking *is that measured?* about a sentence, one commit after it landed.

## Two mispredictions that were the SWEEP working, and the fact behind them

Two mutations predicted KILL survived and were RIGHT to: both were typed on one of the two
fuel-side legs, and **reachability at this rung is per-LEG**. On the measured arm that leg never
rides above the schedule (0 of 341 against its sibling's 21), so the clamp has nothing to cut on
it; and it HOLDS the actuator on 340 of 341 points, so the applied-reference body takes its
float-identity branch. The twins typed on the OTHER leg both kill. **A mutation on a two-legged
plant has to be typed on the leg that is live, and which leg that is is a measurement.**

## A dead hook is not a blind spot if you can INJECT it

Three ported sites were unreachable at this rung because a dispatched hook returns `None` there.
Rather than book three blind survivors, the hook was injected through the table and the march
measured — and the measurement reproduced the NEXT rung's own headline a slice early (the device
is identically zero on whichever leg holds the actuator, so it disarms itself there). **Prefer
injecting a dead hook over booking its sites: it turns three unfalsifiable claims into one
measurement, and it can find the next rung's finding for free.**

## The restore that the build could not see

`shutil.copy2` preserves mtime. A mutation sweep that WRITES a mutant (new timestamp, rebuilt) and
then RESTORES with `copy2` hands back a file OLDER than the artifact just built from the mutant —
so the run ends with the source reading clean and **the test binary compiled from the last
mutation**. The next run's baseline check then measures that stale binary and calls the unmutated
tree red. Use `copyfile` + `utime(None)`.

**And the check that did NOT catch it is the more useful half.** My own "is the tree clean?" grep
sampled two of twenty-four mutation markers and reported clean. **A leftover-check that samples the
mutation set is not a check** — the thing that actually caught it was the next run's baseline
assertion, which is why a sweep must verify the unmutated tree is green before it scores anything.

## And two tooling repeats, both caught by guards installed one step earlier

* A patch script written as a shell heredoc lost its line continuations TWICE; both times the
  substitution-count guard reported `matches=0` and wrote nothing. Write the patch to a FILE when
  any needle carries a backslash. Belongs with [[windows-tooling-file-hazards]].
* **The sweep's own record was truncated by a `head -40`**, so half its verdicts were never
  printed and the reported exit code was the filter's. [[rust-port-slice-af-step2]]'s *a gate's
  exit code does not survive a pipe*, one step later on a different command: a sweep that mutates
  the working tree writes to a LOG, never into a pipe that can close early.
