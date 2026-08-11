---
name: two-indexes-one-spine
description: "CLAUDE.md's rung table and MEMORY.md's shipped-rung list index the SAME spine — the table carries the physical verdict, memory carries the process lesson, and neither restates the other"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bcd8aa28-c502-420c-8a1c-f9d85cd7a8c9
  modified: 2026-08-11T22:06:21.980Z
---

The project keeps **two** indexes over the same 80+ rung spine, and **both load into every
session**: CLAUDE.md's rung table (the physical claim, one row per rung) and this memory
directory's shipped-rung list (the process lesson, one line per rung). Measured 2026-08-12 over
rungs 75–83, the two overlapped **25–59% by content word, averaging ~42%** — and the shared part
was always the same thing: memory's line re-opening with CLAUDE.md's headline sentence.

**Why:** they are two payloads, not one, so merging them would lose something — but they are read
side by side in one context, so a clause present in both is paid for twice every session and
navigates nothing. The duplication is also self-reinforcing: each new rung's memory entry was
written by copying the shape of the last one.

**How to apply:**

- **A rung memory entry's hook is the process lesson**, in positive form: what went wrong, what it
  corrected, what to do differently next time. Never the physical headline — CLAUDE.md's row
  already carries that, in the same context.
- **CLAUDE.md's row is the physical claim** — name, HEADLINE, cross-rung verdict, ≤350 bytes. Never
  the process story; that is memory's and the spec's.
- **Both indexes are grouped by the same families, in numeric order** — the families line sits above
  CLAUDE.md's rung table and the `###` sub-headings mirror it here. Numeric order is also the
  dependency order (each rung reduces to its predecessor), so **never reorder either index by
  theme** — group inside the existing order. When a family label and a rung disagree, name the
  exception in the label (rung 61 is steady, not transient); do not smooth it.
- **A new rung extends the last family's range** — two characters — rather than adding a line.
  That is what keeps the grouping O(1) as the project grows.

**The startup cost is not the reason.** Measured the same day: the three always-loaded files
(project CLAUDE.md, global CLAUDE.md, this index) came to ~63 KB, roughly 2% of context, growing
~470 B/rung. That is affordable for a hundred more rungs. The reason to keep both indexes tight is
**findability** — at 80+ entries a flat chronological list has to be read in full to be used.

Related: [[claude-md-is-a-reference]] (the size guard and its budget), [[session-end-routine]]
(when these files get updated).
