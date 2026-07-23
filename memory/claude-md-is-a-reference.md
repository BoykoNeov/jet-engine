---
name: claude-md-is-a-reference
description: "CLAUDE.md must stay a compact reference/index, not a per-rung handout; a guard test enforces a size budget"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 910011bc-f896-4434-8ff0-820ec360a406
  modified: 2026-07-23T02:23:43.433Z
---

CLAUDE.md is a REFERENCE / index — one line per rung, one line per deferred-seam
entry. Rung detail (derivation, assumptions, concessions, reduce contract, gates)
lives in `docs/rungN-spec.md`, NOT in CLAUDE.md.

**Why:** CLAUDE.md loads into context every session, so its size is a real recurring
cost. It has bloated **twice** by accreting a full essay per rung (reached ~200 KB /
1343 lines before the 2026-07-23 cleanup back to ~22 KB / ~198 lines). A passive
"one-line map not the handout" banner was ignored across sessions — including by me —
so prose alone does not hold the line.

**How to apply:** When adding a rung, add ONE table row + a couple of one-line status
entries; put everything else in the spec. The mechanical backstop is
`tests/test_claude_md_reference.py` (byte + line budget, runs in the fast `pytest`
subset). If it trips because content was written as an essay, move it to the spec —
do NOT raise the budget; bump the budget only for genuine one-line-per-rung growth.
The facts that live ONLY in CLAUDE.md (OPEN seams, NEGATIVE-result docs) must keep
their doc pointer when compressing. See [[session-end-routine]], [[always-commit-and-push]].
