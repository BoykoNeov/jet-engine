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

The budget is generous relative to the compressed size (~22 KB / ~200 lines at
rung 43): headroom for real one-line-per-rung growth, but ~10x tighter than the
essay-bloat it replaced, so an essay-explosion trips it an order of magnitude
earlier than last time (which reached ~200 KB).
"""
import os

_HERE = os.path.dirname(os.path.abspath(__file__))
_CLAUDE_MD = os.path.join(_HERE, os.pardir, "CLAUDE.md")

# Deliberate budget. Bytes is the primary guard (the failure mode is prose volume,
# not line count); lines is a secondary sanity cap. Bump ONLY for real content
# growth, never to make an essay fit — see the module docstring.
MAX_BYTES = 30_000
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
