---
name: rust-port-slice-ae-step2
description: "A mutation sweep from the previous session was still running, still rewriting the working tree, and the backup I took against exactly that was itself mutated — at byte-identical size"
metadata: 
  node_type: memory
  type: project
  originSessionId: ce3a0446-018b-4636-9ce0-39b61e4af5e4
  modified: 2026-09-01T18:35:22.670Z
---

Slice AE step 2 (rung 73 Rust port) shipped the slice's one added cell (`TripleHooks` 13 → 14,
`quad_gains_at`), rung 73's gains body and **all five public readers** in
`M:\claud_projects\jet engine\rust\src\applied_reference.rs` (**401 → 1 663 lines**, both counts
read off the tree — and the write-up first said 1 653, because the count was taken before this
same step's last doc-comment edit landed: **a measured number carries a timestamp as well as a
value**). **`Rust == PyPy` on all 5 066 keys, bit for bit.** Plan § 5.29.2.

**THE LESSON: a background instrument outlives the session that launched it, and its output does
not.** The step's mutation sweep had been launched in the background at the end of the previous
session and was **still alive** — writing a mutated source file, building, restoring, looping —
while its stdout went to a terminal that no longer existed. Three things followed, and the ORDER
OF DISCOVERY is the lesson: the first is what exposed the sweep at all, and the other two were
found by measuring afterwards, not by noticing:

1. **It deleted the artifact I had just produced.** Its `run()` begins `os.remove(dump)`, so a
   dump this session drove — printed as written, exit 0 — was gone before anything compared it.
2. **The source was mutated at rest, and the timestamp had an innocent explanation ready** (later
   than the baseline: exactly what a doc-comment edit looks like).
3. **The backup taken to guard against this was itself contaminated — and its SIZE MATCHED the
   clean file byte for byte**, because the mutation it carried swapped `{ 0 } else { 1 }` for
   `{ 1 } else { 0 }`. A size that matches is not a checksum.

**How to apply:** before reading, copying or trusting any file a sweep can write, check for the
sweep's PROCESS by PID (`Get-CimInstance Win32_Process` for the command line — two other `cargo`
runs on this machine belonged to a different repository, and a name-matched kill would have taken
them). Recovery, with nothing committed to fall back on, came from **two independently mutated
snapshots reverted by their own anchors, byte-identical afterwards** — the agreement is the
evidence; either revert alone is a hope.

**THE SAME TOOL DID A SECOND INVISIBLE THING: A THREE-LINE DIFF THAT REWROTE 1 569 LINES.** It
reads with universal newlines and writes with `newline=None`, so on Windows every file it touched
came back **CRLF** in full. With git's `text=auto` that is normalised away on read, so the diff,
the diffstat and `git show` all reported exactly the three intended lines. The full gate caught it
at target 86 of 139: a slice-AC gate that `include_str!`s a source file and scopes its search with a
newline-brace-newline pattern extracted 65 145 chars instead of 4 229 and fired *"the scope slipped past the function
body"*. Proven not to be the code change by converting **HEAD's own bytes** to CRLF (fails
identically) and the worktree's to LF (passes identically). Census: **171 files, 141 LF, 30 CRLF** — and that
census mis-named its own population (167 Rust, 4 Python, from a stray glob); the Rust split was
137 LF / 30 CRLF, and **21 files are still CRLF today**, so the trap is dormant, not gone. Of the
six `include_str!` sites reading a `.rs` file, **only that one is line-ending dependent** —
checked, not assumed, and `src/` has **zero** such sites, which is why the bit-exact dump driven
on the CRLF tree transfers to the LF one without a re-run.

**How to apply:** a tool that rewrites a source file must read AND write with `newline=""`, and
assert the restore is **byte for byte**, not merely that the file is back. And when a gate that
reads raw source bytes fails on a step that added three lines, suspect the ENCODING before the
lines — `git diff` is normalised and cannot see it.

**AND A DUMP-ONLY MUTATION SWEEP ANSWERS THE WRONG QUESTION.** Every mutation here was scored on
**two seats** — does it move a value in the 5 066-key dump, and would a shipped gate have caught
it. Six of nine move a value; exactly **one** moves a gate. That is not a hole, it is the step's
shape (step 2 ships no gate file, so the only gates are step 1's plumbing gates, blind to the
gains chain by construction) — but only the second seat makes the sentence sayable instead of
assumable. The five value-only mutations are pre-registered for step 3's ported gates.
[[rust-port-slice-ae-step1]]'s M11 was re-run here rather than cited: **M11 alone moves 0 keys and
0 gates; M11b, deleting BOTH law-carries, moves 122 and kills one** — so the shipped docstring is
true of the pair and false of the member, and M11's zero was a property of the other carrier.

**A GATE NAMED FOR TWO CARRIERS COULD ONLY FAIL ON ONE.**
`at_lever_and_the_rig_both_carry_the_reference` survives deleting the rig's copy and dies only
when both go — a one-sided detector wearing a two-sided name, visible only to the both-copies
mutation.

**AND A ZERO ON THE VALUE SEAT IS NOT "THE DEFENCE IS UNNECESSARY".** The `-0.0`/`+0.0` set-keying
mutation moved 0 keys, so the dump's raw bits were read: **101 keys are exactly `-0.0`** (all of
one field), 925 are `+0.0`, and inside the four sets that mutation re-keys `+0.0` appears twice
and `-0.0` never. Unobservable **on this grid**, measured — [[rust-port-slice-ac-step5]]'s j10
shape — and booked to the oracle rather than shrugged at.

**A NUMBER'S HEADING CAN NAME A DIFFERENT OBJECT FROM THE THING MEASURED.** A probe's `29/25` vs
`28/24` per-arm pair sat under a heading that said "arms" while the neighbouring prose said 28 and
24; both were right about different objects. Splitting the counter per law showed the ARM list is
four wider at every point on both arms, and the seventh evaluation is rung 69's `_manifold_v`
falling through to its parent when the incidence limiter is disarmed (70 of 70 against 0 of 31) —
present under both bodies, so it cancels.

See [[rust-port-status]], [[rust-port-slice-ae-preflight]], [[windows-tooling-file-hazards]].
