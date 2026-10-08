import io
M = "C:/Users/boiko/.claude/projects/W--Claude-projects-jet-engine/memory/"
G = "W:/Claude_projects/jet engine/"


def rd(p):
    with io.open(p, encoding="utf-8", newline="") as fh:
        return fh.read()


def wr(p, s):
    with io.open(p, "w", encoding="utf-8", newline="") as fh:
        fh.write(s)


def edit(p, pairs):
    s = rd(p)
    for old, new in pairs:
        assert s.count(old) == 1, (p, old[:60], s.count(old))
        s = s.replace(old, new)
    wr(p, s)


# 0. hazards file -> uniform CRLF
p = M + "windows-tooling-file-hazards.md"
wr(p, rd(p).replace("\r\n", "\n").replace("\n", "\r\n"))

# 1. the plan
edit(G + "docs/plans/todo-rust-port.md", [
    ("AND RUNG 80's `at_lever` IS VISIBLE TO POINTER IDENTITY AND TO NOTHING ELSE — IN EITHER LANGUAGE**",
     "AND RUNG 80's `at_lever` IS INVISIBLE TO EVERY VALUE-BEARING SEAT — CAUGHT ONLY BY STRUCTURAL GATES, MEASURED IN RUST THROUGH RUNG 80**"),
    ("   row enters neither at those seats. A nonzero `at_lever` count there does NOT mean *ran and made\n"
     "   no difference*; the difference was built and then never consulted.",
     "   row enters neither at those seats. A nonzero `at_lever` count there does NOT mean *ran and made\n"
     "   no difference*; the difference was built and then never consulted. *Never dispatched* is an\n"
     "   INFERENCE across rows — the `Count` row is a different rig from the `AtLever` row — sound\n"
     "   because the dispatch sites belong to the READERS, not to the tables."),
    ("* `split_saturation`'s 10 931 is 3 more than 8 × 1 366: one of its eight marches is three calls\n"
     "  longer. Pinned, not explained.",
     "* `split_saturation`'s 10 931 is 3 more than 8 × 1 366. **First written as \"one march is three\n"
     "  calls longer\" — which the arithmetic refutes** (a point is 4 entries; the advisor's catch). A\n"
     "  throwaway per-march probe measured it: all eight marches are 341 points; the THREE highest\n"
     "  walls (0.855, 0.86, 0.88) each enter `cap_fuel` 1 367 times, the five lower 1 366. One extra\n"
     "  entry per march at the top walls, from a site not traced."),
    ("at all (grepped), so the ported suite is faithfully blind, not under-ported. The override is\n"
     "redundant on every shipped path in PYTHON too — `_shared_rig` re-sets `_sm_air` off `self` — so\n"
     "rung 80's `at_lever` is visible to Rust pointer identity and to nothing value-bearing in either\n"
     "language. This measures slice AJ's booking (§ 5.33 (ii)) at its first rung: at rung 80 no reader\n"
     "depends on the rebuilt machine being its own rung. Whether one at 81–84 does stays AJ's question.\n"
     "The Python half is inferred from the grep and the port's fidelity, not re-run.",
     "at all (grepped), so the ported suite is faithfully blind, not under-ported. **What caught the\n"
     "mutation is STRUCTURAL, and not only pointers**: of the 11 failures, the pointer census, the\n"
     "install proof and the mechanism gate compare pointers, but `slice_ai_cells.rs`'s carry gate and\n"
     "this file's rebuild-helper proof READ BACK the sibling's `sm_air` knob. No value-bearing seat —\n"
     "the oracle, either ported suite — sees it. **Measured in Rust, through rung 80 only.** That\n"
     "Python's `test_rung80.py` is blind too is INFERRED (the grep, the port's fidelity), not run; and\n"
     "Python rungs 81–84 INHERIT `SplitWallTransient.at_lever`, so deleting it there changes the class\n"
     "of every rig those rungs build — their suites and kernels were never run under it, and the Rust\n"
     "port cannot yet. **Booked as slice AJ's first pre-flight probe**: delete the method in\n"
     "`engine.py`, run `test_rung80.py`…`test_rung84.py` and kernels r80–r84, restore by checkout +\n"
     "hash. That is § 5.33 (ii)'s booked question in its sharpest form."),
    ("**SLICE AI IS CLOSED.** Next: slice AJ (rungs 81–84), which owes its pre-flight — including § 5.33\n"
     "(ii)'s booking, now measured at rung 80 by (e).",
     "**SLICE AI IS CLOSED.** Next: slice AJ (rungs 81–84), which owes its pre-flight — opening with\n"
     "(e)'s booked Python mutation, § 5.33 (ii)'s question asked at rungs 81–84."),
])

# 2. the test file (comments only)
edit(G + "rust/tests/slice_ai_dispatch.rs", [
    ("//!   because they call the plant with an explicit coordinate and never touch the rig's table.\n"
     "//! * **Rung 80's `at_lever` is visible to pointer identity alone** — silent at all nine seats, on\n"
     "//!   values and counters, while entered at every one: `r80_shared_rig` re-reads `sm_air` off the\n"
     "//!   CORE. The closing source mutation (plan § 5.33.7) is that claim tested on the whole crate.",
     "//!   because they call the plant with an explicit coordinate and never touch the rig's table\n"
     "//!   (*never dispatched* is read off the `Count` row — an inference across rows, sound because\n"
     "//!   the dispatch sites belong to the readers, not the tables).\n"
     "//! * **Rung 80's `at_lever` is invisible to every value-bearing seat** — silent at all nine seats,\n"
     "//!   on values and counters, while entered at every one: `r80_shared_rig` re-reads `sm_air` off\n"
     "//!   the CORE. The closing source mutation (plan § 5.33.7 (e)) is caught only by STRUCTURAL gates\n"
     "//!   — pointer identity AND a readback of the sibling's `sm_air`. Measured in Rust through rung 80."),
    ("///   reader dispatches that rig's `shared_rig` again. So rung 80's `at_lever` re-aim is visible to\n"
     "///   pointer identity and to nothing else — AH P2's shape, one rung on (the closing mutation).",
     "///   reader dispatches that rig's `shared_rig` again. So rung 80's `at_lever` re-aim is invisible\n"
     "///   to every value-bearing seat and caught only structurally — pointer identity and the\n"
     "///   sibling-knob readback (the closing mutation, plan § 5.33.7 (e))."),
    ("/// * **`split_saturation` enters `cap_fuel` 10 931 times, not 8 × 1 366 = 10 928** — one of its eight\n"
     "///   marches is three calls longer. Pinned, not explained.",
     "/// * **`split_saturation` enters `cap_fuel` 10 931 times, not 8 × 1 366 = 10 928.** Not a longer\n"
     "///   march (a point is 4 entries): a throwaway probe measured all eight at 341 points, with the\n"
     "///   three highest walls (0.855, 0.86, 0.88) at 1 367 entries each. One extra entry per march at\n"
     "///   the top walls, from a site not traced."),
])

# 3. memory
edit(M + "rust-port-slice-ai-step7.md", [
    ("and rung 80's at_lever is visible to pointer identity alone, in both languages\"",
     "and rung 80's at_lever is invisible to every value-bearing seat, caught only structurally (measured in Rust through rung 80)\""),
    ("- The closing mutation (delete `at_lever: r80_at_lever,`) was caught by pointer gates only; I\n"
     "  predicted the ported `rung80.rs` would fail and it was 16/16 — `test_rung80.py` has no carry gate.",
     "- The closing mutation (delete `at_lever: r80_at_lever,`) was caught only by STRUCTURAL gates —\n"
     "  pointer identity AND a readback of the sibling's `sm_air` (I first wrote \"pointers only\"; the\n"
     "  advisor caught it from my own table). I predicted the ported `rung80.rs` would fail: 16/16, since\n"
     "  `test_rung80.py` has no carry gate. The Python side and rungs 81–84 were NOT run — booked as\n"
     "  slice AJ's first pre-flight probe.\n"
     "- `split_saturation`'s +3 `cap_fuel` entries: I first wrote \"one march is 3 calls longer\", which\n"
     "  the arithmetic refutes (a point = 4 entries); a probe found +1 at each of the 3 top walls."),
])
edit(M + "rust-port-status.md", [
    ("rung 80's `at_lever` visible to pointer identity alone (source mutation: `rung80.rs` 16/16, "
     "oracle 0 keys — I predicted rung80 would fail).",
     "rung 80's `at_lever` invisible to every value-bearing seat, caught only structurally (source "
     "mutation: `rung80.rs` 16/16, oracle 0 keys — I predicted rung80 would fail); Python side and "
     "rungs 81–84 booked as AJ's first pre-flight probe."),
])
print("ok")
