import io
M = r"C:/Users/boiko/.claude/projects/W--Claude-projects-jet-engine/memory/"
def edit(name, old, new):
    p = M + name
    s = io.open(p, encoding="utf-8", newline="").read()
    assert s.count(old) == 1, (name, s.count(old))
    io.open(p, "w", encoding="utf-8", newline="").write(s.replace(old, new))
edit("MEMORY.md",
     "slice AI (79/80) steps 1–6 DONE, step 7 next",
     "slice AI (79/80) COMPLETE in 7 steps; slice AJ (81–84) next, needs its pre-flight")
edit("rust-port-slice-index.md",
     "[step 6](rust-port-slice-ai-step6.md) —",
     "[step 6](rust-port-slice-ai-step6.md) [step 7](rust-port-slice-ai-step7.md) —")
edit("rust-port-slice-index.md",
     "where a fault reaches an output is measured by injecting into it, not by reasoning (one lesson per file)",
     "where a fault reaches an output is measured by injecting into it, not by reasoning; and a call count says a constructor RAN, not that what it built was ever dispatched (one lesson per file)")
