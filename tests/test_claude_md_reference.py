"""Guard: CLAUDE.md is a REFERENCE / index, not a handout.

CLAUDE.md is loaded into context at the start of every session, so its size is a
recurring, real cost. It has twice grown into a multi-hundred-KB document by
accreting a full essay per rung — duplicating content that already lives in each
rung's spec. This test is the mechanical backstop for the "one-line map, not the
handout" rule stated in the file's own banner.

If this test FAILS:
  * If it tripped because a rung (or a seam) was written up as an ESSAY here —
    that is the failure mode this guard exists to catch. Move the detail into the
    rung's spec (docs/rungN-spec.md) and leave a ONE-LINE hook here. Do NOT raise
    the budget.
  * If it tripped because the project legitimately grew — many new rungs, each a
    genuine single line in the table — then and only then bump the budget below,
    deliberately, in the same commit that adds the content.

The budget is generous relative to the compressed size (~26 KB / ~205 lines at
rung 54; 30.3 KB / 232 lines at rung 64): headroom for real one-line-per-rung
growth, but ~7x tighter than the essay-bloat it replaced, so an essay-explosion
trips it an order of magnitude earlier than last time (which reached ~200 KB).
The bump at rung 64 was the first, and it bought ~2 rungs of headroom on purpose:
a guard that forces a conscious decision every couple of rungs is working. If a
bump ever buys many rungs at once, that is the essay failure mode wearing a
budget's clothes.

On the shape of legitimate growth: after the rung-54 re-compaction, the rung
TABLE is the only section that grows per rung (~250 bytes/row). The three other
sites that used to grow in lockstep with it — the "Current scope" ladder
narration, the § Layout `engine.py` method inventory, and the per-seam "Built"
list — were each collapsed to a form that is invariant in the rung count, and
they should stay that way. If this guard trips again, check those three FIRST:
a re-grown narration there is the bug, and the fix is deletion, not a bump.
"""
import os

_HERE = os.path.dirname(os.path.abspath(__file__))
_CLAUDE_MD = os.path.join(_HERE, os.pardir, "CLAUDE.md")

# Deliberate budget. Bytes is the primary guard (the failure mode is prose volume,
# not line count); lines is a secondary sanity cap. Bump ONLY for real content
# growth, never to make an essay fit — see the module docstring.
# Bumped at rung 64 (30,274 B): one genuine row, deliberately. Bumped again 2026-07-31
# (31,259 B) for the golden-fingerprint gate's § Layout entry — NOT a rung, and not table
# growth: a third non-rung test file, beside the two § Layout already names. The prior budget
# had 33 bytes left, so rung 67's row trips it regardless; taking the bump here for a reason
# that can be named beats taking it later for a row that cannot. Bumped at rung 67 (31,583 B)
# for exactly the row that comment predicted -- ONE table line at 345 B (inside the 350 B
# per-row rule) plus its § Layout ladder entry, and PAID FOR in part by rewriting the CASCADE A
# open-seam entry it closes. Real one-line-per-rung growth, which is what a bump is for.
MAX_BYTES = 31_700
MAX_LINES = 300


def _read():
    with open(_CLAUDE_MD, "rb") as fh:
        raw = fh.read()
    return raw, raw.decode("utf-8").count("\n") + 1


def test_claude_md_within_byte_budget():
    raw, _ = _read()
    assert len(raw) <= MAX_BYTES, (
        f"CLAUDE.md is {len(raw):,} bytes, over the {MAX_BYTES:,}-byte budget. "
        "CLAUDE.md is a reference/index — rung detail belongs in docs/rungN-spec.md, "
        "not here. Move the detail out (do NOT raise the budget) unless this is real "
        "one-line-per-rung growth. See this test's module docstring."
    )


def test_claude_md_within_line_budget():
    _, lines = _read()
    assert lines <= MAX_LINES, (
        f"CLAUDE.md is {lines:,} lines, over the {MAX_LINES}-line budget. "
        "Keep the rung table to one line per rung and 'Deferred seams' to one line "
        "per entry; the detail lives in the specs. See this test's module docstring."
    )


if __name__ == "__main__":
    test_claude_md_within_byte_budget()
    test_claude_md_within_line_budget()
    print("CLAUDE.md within budget:", os.path.getsize(_CLAUDE_MD), "bytes")
