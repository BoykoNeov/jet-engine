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
# Bumped at rung 68 (32,173 B), same shape and same reason: ONE table row at 348 B (inside the
# per-row rule), its § Layout ladder entry, and its name in the BUILT list -- PAID FOR by
# rewriting the THREE-loops open-seam entry it closes into rung 68's own, one line shorter than
# the sum of what it replaced. The three no-grow sites (scope narration, engine.py method
# inventory, per-seam BUILT list) were checked first and none of them grew.
# Raised again in the same commit (32,507 B) for something that is NOT rung growth and is
# recorded here so it cannot be mistaken for it: rung 68 found that `main.py` has carried no
# panel since rung 64, so § Layout's "one panel per rung" was false. The correction plus the
# open-task line that tracks the backfill cost ~334 B. That is a DEFECT DISCLOSURE, and the
# right move when it is paid off is to DELETE both and drop this budget back, not to keep it.
# Bumped at rung 69 (33,400 B), the same shape and the same reason as 67 and 68: ONE table row
# at 345 B (inside the per-row rule), its § Layout ladder entry, and its name in the BUILT list.
# PAID FOR only in part -- the open-seam entry rung 69 closes (THE REFERENCE SPLIT) was rewritten
# into rung 69's OWN seams, which are more numerous than what they replaced, so that rewrite cost
# ~65 B rather than refunding any. The three no-grow sites (scope narration, engine.py method
# inventory, per-seam BUILT list) were checked first and none of them grew beyond one name.
# This buys ~90 B of headroom on purpose: the next rung trips it again, which is the guard
# working. The rung-68 panel-backfill disclosure (~334 B) is still unpaid and is still the
# first thing to delete when § Open engineering tasks clears it.
# Bumped at rung 70 (33,680 B), and the ~90 B of headroom above did exactly what it was bought
# to do -- the next rung tripped it. Same shape as 67/68/69: ONE table row at 322 B (comfortably
# inside the per-row rule), its § Layout ladder entry, and its name in the BUILT list. PAID FOR
# in part: rung 70 closes TWO listed open seams (rung 68's THREE-loops-on-TWO-variables and rung
# 69's pair_RV != pair_CV, which rung 69 identified as one seam from two sides), so the "Rung
# 69's seams" open-list entry was rewritten into "Rungs 69/70's seams" -- one clause deleted,
# two added, roughly a wash rather than a refund. The three no-grow sites (scope narration,
# engine.py method inventory, per-seam BUILT list) were checked first and none grew beyond one
# name. The rung-68 panel-backfill disclosure (~334 B) is STILL unpaid: rung 70 wrote its own
# panel, but 65/66/67 still have none, so that debt does not clear here.
# Bumped at rung 71 (34,180 B), and the ~105 B of headroom above again did what it was bought to
# do. Same shape as 67/68/69/70: ONE table row at 349 B (inside the per-row rule), its § Layout
# ladder entry, and its name in the BUILT list. PAID FOR in part, and this time genuinely: rung 71
# closes the `n = m = 3` cell, which was the FIRST clause of the "Rungs 69/70's seams" open entry,
# and that entry was rewritten 20 B SHORTER than what it replaced despite naming three new seams.
# The three no-grow sites (scope narration, engine.py method inventory, per-seam BUILT list) were
# checked first and none grew beyond one name. ~85 B of headroom on purpose, so the next rung trips
# it again. The rung-68 panel-backfill disclosure (~334 B) is STILL unpaid — 65/66/67 have no
# panel and rung 71 wrote its own, so that debt is now four rungs old and is still the first thing
# to delete when § Open engineering tasks clears it.
# Bumped at rung 72 (34,790 B), and the ~85 B of headroom above again did what it was bought to
# do. Same shape as 67/68/69/70/71: ONE table row at 346 B (inside the per-row rule), its § Layout
# ladder entry, and its name in the BUILT list. PAID FOR from TWO places, and one of them settles
# an old debt: (a) the "Rungs 69/70/71's seams" open entry was rewritten as "Rungs 69–72's" and
# came out ~17 B longer despite naming three new seams and marking one CLOSED-BY-REFUTATION, so
# roughly a wash; (b) **the rung-68 panel-backfill disclosure is finally PAID OFF and DELETED** —
# § Open engineering tasks records the contract CLOSED (65/66/67 backfilled 2026-08-09), and the
# comment at the top of this block said the right move once paid was to delete it and drop the
# budget rather than keep it. Only the standing caveat survives, one line instead of two, because
# `main.py` really is covered by no test and that warning is not the debt. Refund ~67 B. The three
# no-grow sites (scope narration, engine.py method inventory, per-seam BUILT list) were checked
# first and none grew beyond one name. ~88 B of headroom on purpose, so the next rung trips it.
# Bumped at rung 73 (35,240 B): the same shape as rungs 67/68 and for the same reason -- ONE
# table row at 349 B (inside the 350 B per-row rule), its § Layout ladder entry, and its name in
# the BUILT list. PAID FOR IN PART, as that precedent requires, by rewriting the open-seam entry
# it CLOSES: the rung-69..72 line lost "an APPLIED-fuel-referenced leg" and gained rung 73's own
# sharpest seam (STATE-AS-DEMAND) in fewer words, and § Open engineering tasks' panel line lost a
# date it no longer needs. The three no-grow sites (scope narration, engine.py method inventory,
# per-seam BUILT list) were checked FIRST and none grew beyond one name and one ladder arrow.
# ~8 B of headroom on purpose: the next rung trips this, which is the guard working.
MAX_BYTES = 35_240
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
