---
name: rung105-flight-wiring
description: "2026-10-21 repair (not a rung), rung 105's first seam: the HP vane law's flight pressure wired through rung 37's soak and the two-shaft fuel rigs (43–91), checked at each closure's ENTRY for the same compressor face; rung 102's one-shaft afterburner refuses the law up front — lessons: re-run on the base commit before saying where a failure came from (a cause from R1b's lost messages was refuted by R1b's own header); a panic-message list misses tests a sibling's quiet hook silenced; check a refusal before the scan that would re-word it; cargo test with several --test stops at the first failing binary"
metadata:
  node_type: memory
  type: project
  originSessionId: ee684fdf-ba80-4ca5-9ee0-05449763a01e
  modified: 2026-10-21T14:09:13.361Z
---

Repair (not a rung), 2026-10-21, after [[rung105-hp-vane]] and [[rung34-low-speed-wall-repair]]. Spec:
`docs/rung105-spec.md` § The paths wired after the rung. Gate `rust/tests/rung105_flight.rs`. Census archived in
`docs/plans/scratch-archive/rung105-flight/census.txt` (law as default: 198 failed vs R1b's 199; the 15 swallowed
refusals → 0; no new crash). The user asked only "What is next" → "1"; no scope decisions were needed.

**Lesson 1 — re-run on the base commit before saying where a failure came from.** Five failures were absent from R1b's
list. I first blamed lost messages in R1b's run, but R1b's own header says only ONE of its failures had no message, so
that could not explain five; the advisor caught it after the push. What was measured: on `fc69393` (this change's base,
a worktree with `hp-law-default`) they fail identically, so they came in between R1b and `fc69393`. Separately, eight
failures in THIS census had no message (a sibling test's process-wide quiet panic hook ate them): compare censuses per
program by FAILED test names, and re-run any program whose failed count ≠ its panic count alone (`--test-threads=1`).
Don't group a check you never ran on the base with ones you did (the page check was).

**Lesson 2 — check a refusal at the closure's ENTRY, before any trial-skipping scan.** The old sticky value was read
inside each trial, so the scan skipped the refusal and said "does not bracket". Reading it once at entry and returning
with `?` keeps the law's words. Store the pressure WITH the inputs it belongs to (`tt2`, `pt2`, compared to the bit),
so a sibling machine or another flight's leftover is refused, not used. Every refusal it fired was a MISSING flight:
tests that read the inlet on one machine and close on a fresh sibling ([[instrument-fed-by-what-it-certifies]]).

**Lesson 3 — `cargo test --test a --test b …` stops at the first failing binary.** Add `--no-fail-fast` when the point
is to see every one fail; stopping the Bash wrapper left no orphan here, but check by command line
([[windows-tooling-file-hazards]]).

**Lesson 4 — "law = pin" on the perfect gas is bit-exact only at design efficiency**; off the design speeds the shortcut
is off and the law's search lands within ~1e-13 (rung 57's toggle: 9.3e-14). Bar such gates at R2's 1e-12, not bits.
