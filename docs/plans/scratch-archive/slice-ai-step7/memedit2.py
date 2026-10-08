import io
M = r"C:/Users/boiko/.claude/projects/W--Claude-projects-jet-engine/memory/"
def edit(name, old, new):
    p = M + name
    s = io.open(p, encoding="utf-8", newline="").read()
    assert s.count(old) == 1, (name, s.count(old))
    io.open(p, "w", encoding="utf-8", newline="").write(s.replace(old, new))
edit("rust-port-status.md",
     "(injected 999.0: 0 keys). ",
     "(injected 999.0: 0 keys). **STEP 7 DONE — SLICE AI COMPLETE 2026-09-29** (§ 5.33.7) — "
     "`slice_ai_dispatch.rs`, 9 gates, 32 s, rows on parallel threads with NO lock (thread-local "
     "counters); every verdict row as pre-registered. Rung 79's three table swaps are ONE deletion "
     "to every seat; THREE kinds of silence (ran / never entered / built-but-never-dispatched); "
     "rung 80's `at_lever` visible to pointer identity alone (source mutation: `rung80.rs` 16/16, "
     "oracle 0 keys — I predicted rung80 would fail). P1 REFUTED: 2 195 lines, per-LINE. "
     "See [[rust-port-slice-ai-step7]]. **NEXT: slice AJ (rungs 81–84) needs its pre-flight.** ")
edit("rust-port-status.md",
     "## Slice AI (rungs 79 + 80, `StateCoordinateTransient` + `SplitWallTransient`) — IN FLIGHT, steps 1–2 done, step 3 next",
     "## Slice AI (rungs 79 + 80, `StateCoordinateTransient` + `SplitWallTransient`) — CLOSED 2026-09-29, all seven steps (see the phase line above for steps 3–7)")
