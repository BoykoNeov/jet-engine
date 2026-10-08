import io
M = r"C:/Users/boiko/.claude/projects/W--Claude-projects-jet-engine/memory/rust-port-status.md"
s = io.open(M, encoding="utf-8", newline="").read()
lines = s.split("\n")
idx = [i for i, l in enumerate(lines) if l.startswith("| AH | 77+78 |")]
assert len(idx) == 1
row = ("| AI | 79+80 | **2026-09-29, seven steps** — `state_coordinate.rs` + `split_wall.rs` 2 195 lines (P1 "
       "refuted: per-LINE), 44 ported gates, **37 945 oracle keys** bit-exact vs PyPy (CPython: 227 differ, "
       "all on the two pre-predicted `sum()` paths), **9 dispatch gates**; full gate **172 blocks / 1 776 "
       "passed / 0 failed**, pytest 1 387 |")
lines.insert(idx[0] + 1, row)
io.open(M, "w", encoding="utf-8", newline="").write("\n".join(lines))
