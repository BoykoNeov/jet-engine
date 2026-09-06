---
name: rust-port-slice-af-usage-guard
description: "Slice AF's last debt — a census's SKIP category was hiding a worse defect than the fails it counted, and the guard built to stop it rebuilt the same blind spot one layer down"
metadata: 
  node_type: memory
  type: project
  originSessionId: 4c1ab6dd-a64e-489b-ac0f-dcd24b896287
  modified: 2026-09-06T08:25:17.658Z
---

Slice AF's closing commit (rung 74, `docs/plans/todo-rust-port.md` § 5.30.7) fixed the seven
`Usage:`-block calls step 5 (a) had booked and shipped `tests/test_usage_blocks.py` to stop the
class regrowing. Two lessons, and the second is about my own instrument.

**A SKIP CATEGORY CAN HIDE A WORSE DEFECT THAN THE ONE BEING COUNTED.** Step 5 (a)'s census
reported *10 bind fails, 5 skipped (`**kw`, or an argument list that is not Python)*. The fails
were the headline for a whole step; the skip line's second clause was **six documented statements
that do not parse at all** — a line that cannot be TYPED, which is strictly worse than a call that
raises when you type it. One of them names a parameter (`p0`) the builder does not have
(`p_ambient`), so it carries a second defect underneath the first. **They hid because the census
parsed each block WHOLE**: measured, 0 of the six blocks parse, and they document **19 further
calls** that therefore went unchecked too. Ask what a census is DECLINING to check, and what that
declining costs — a skip is not a smaller failure, it is an unmeasured one.

**AND THE GUARD I WROTE TO FIX THAT REBUILT IT ONE LAYER DOWN.** My first `_statements` joined a
bad line forward until the group parsed — and when no group ever parsed, emitted the whole
remainder as ONE casualty, so an untypable line still swallowed everything after it. The only
thing that caught it was the file's own *can it see a break* gate (a synthetic module carrying one
of each defect, asserted by kind AND count): on the repaired tree every block parses, so the
difference is invisible to every other gate in the file. **A can-it-see gate is not ceremony —
it is the only test that runs the instrument's failure path once the tree is clean**
([[instrument-fed-by-what-it-certifies]]).

**Three shapes worth keeping.** (1) The three phantom readers were RETARGETED, never deleted, and
each target came from a source that is not the name — a trailing comment, a section header, the
rung spec's own reader list; AD's pre-flight had guessed from naming alone and named one that was
already on its own line in the same block. (2) The two not-Python spellings each had a CORRECT
TWIN a few hundred lines away, so [[rust-port-slice-af-step6]]'s *look for one construct spelled
two ways* decided both repairs with no argument from first principles — second instance in two
commits. (3) The bars are read off the run: this checker counts 166 calls where step 5 (a)'s
counted 102, because it resolves constructors and builders too, so the old table is not reused —
the two agree only where both measure, the seven fails by name.

Related: [[rust-port-slice-af-step6]], [[rust-port-slice-af-step5a]], [[golden-fingerprint-gate]].
