---
name: claude-md-is-a-reference
description: "CLAUDE.md must stay a compact reference/index, not a per-rung handout; a guard test enforces a size budget"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 910011bc-f896-4434-8ff0-820ec360a406
  modified: 2026-07-28T06:06:07.236Z
---

CLAUDE.md is a REFERENCE / index — one line per rung, one line per deferred-seam
entry. Rung detail (derivation, assumptions, concessions, reduce contract, gates)
lives in `docs/rungN-spec.md`, NOT in CLAUDE.md.

**Why:** CLAUDE.md loads into context every session, so its size is a real recurring
cost. It has bloated **twice** by accreting a full essay per rung (reached ~200 KB /
1343 lines before the 2026-07-23 cleanup back to ~22 KB / ~198 lines). A passive
"one-line map not the handout" banner was ignored across sessions — including by me —
so prose alone does not hold the line.

**The rung TABLE must be the only section that grows per rung.** The 2026-07-28
re-compaction (29,989 → 26,276 B) found the actual bloat mechanism: growth was
~750 B/rung while a table row is only ~250 B, because FOUR sites grew per rung.
Three were pure duplication of the table and are now collapsed to rung-count-invariant
forms — (1) the "Current scope" paragraph's matcher-ladder narration, (2) the
§ Layout `engine.py` per-rung method-name inventory, (3) the per-seam "Built" list
(now one line: "seams 25–N closed by the same-numbered rung"). **If the guard trips
again, check those three FIRST — a re-grown narration there is the bug, and the fix
is deletion, not a bump.** Rows 1–48 are already at the ~230-char one-line budget;
compressing them is shaving real content, not fixing bloat.

**How to apply:** When adding a rung, add ONE table row + a couple of one-line status
entries; put everything else in the spec. The mechanical backstop is
`tests/test_claude_md_reference.py` (byte + line budget, runs in the fast `pytest`
subset). If it trips because content was written as an essay, move it to the spec —
do NOT raise the budget; bump the budget only for genuine one-line-per-rung growth.
The facts that live ONLY in CLAUDE.md (OPEN seams, NEGATIVE-result docs) must keep
their doc pointer when compressing. See [[session-end-routine]], [[always-commit-and-push]].
