"""Guard: every `engine.py` LINE CITATION in the Rust port still points where it did.

The port's doc comments cite the Python they port by line — `engine.py:17818`,
and inside the same comment block the bare `` `18653` `` form for a second site.
Those citations are the only mechanical link between a Rust body and the source
it is a port of, and they are the one kind of documentation here that goes wrong
WITHOUT ANYONE TOUCHING IT: an edit anywhere above the cited line moves it, and
the comment goes on naming a number that now points at something else entirely.

WHAT THE DECAY LOOKED LIKE WHEN IT WAS FIRST MEASURED (slice AG step 1):

  * **18 of 50 citations were stale.** Thirteen by exactly `+2` — commit
    `a592a0d`, which added `tests/test_usage_blocks.py` and edited 34 lines of
    `engine.py` on the way — one by `+14` from an older shift, and four more of
    the bare form that a first sweep for `engine.py:` did not even look at.
  * The correction that FOUND it was itself stale. Slice AG's own step 1 wrote
    *the only writer is rung 75's `_with_ic_cap` (`engine.py:19022`)* is wrong —
    and the wrong half is the METHOD NAME. `19022` pointed at exactly the right
    line when it was written; `a592a0d` then pushed that line to `19024`.
  * A citation written in this slice, in a NEW file, inherited a stale number by
    being copied from the comment it was correcting.

So: the numbers are right when written and rot silently afterwards, and a reader
who follows one lands on a plausible-looking wrong line. That is worse than a
missing citation, because it reads as confirmation.

HOW THIS GUARD WORKS. `tests/golden/rust_engine_citations.json` records, for
each cited line, the TEXT that line held when a human last verified the citation.
The gate is that `engine.py` still says the same thing at the same number. It
detects DRIFT, not wrongness — a citation blessed while pointing at the wrong
line stays wrong, and no instrument can know what a comment MEANT to point at.
That is why blessing is a deliberate act with a printed diff, not a `-u` flag.

IF THIS TEST FAILS:
  * a DRIFT means `engine.py` moved under a citation. The failure message names
    the line the recorded text is at NOW; that number is the repair. Fix the
    COMMENT — the Python is not wrong for having moved.
  * an UNBLESSED citation means a new one was written. Verify by hand that it
    points where the sentence says, then run this file directly to re-bless.
  * a COUNT change means citations were added or removed. Same: run this file
    directly and type the numbers off ITS OWN run.
  * a citation quoted as HISTORY — a comment that says *this used to read `N`* —
    belongs in `HISTORICAL` below, which is frozen for the reason the AF guard's
    skip list is: an unchecked line is not a lesser failure than a failing one.

WHAT THIS GUARD DOES NOT DO: it does not read the sentence around the citation,
so it cannot tell you that `engine.py:17713` is quoted as *the line that copies
`_ref_law`* when it copies something else. It answers one question only, and it
answers it every run: has the ground moved since somebody looked?
"""
import io
import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENGINE_PY = os.path.join(ROOT, "turbojet", "engine.py")
RUST_SRC = os.path.join(ROOT, "rust", "src")
ANCHORS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "golden",
                       "rust_engine_citations.json")

# --- the census, MEASURED (run this file directly; it prints all three) --------------------
# Read off the run in the commit that repaired the 18 stale citations, on the
# repaired tree. `SITES` counts every occurrence, so a line citing two numbers
# counts twice; `LINES` counts the distinct `engine.py` lines they point at.
#
# RE-BLESSED AT SLICE AG STEP 5 (10/59/47 -> 10/63/51): four new citations, all step 5's own
# (`18789` is `_rhs_gains_at`'s defaults line, `19341` the plain `_lag_coord` write, `19352` the
# `accel` argument `_cap_rows` passes where rung 75 passes `None`, `19523` the untagged grid
# assert). NO ARREARS this time -- step 4's procedural repair held, and this guard was run BEFORE
# the step shipped rather than after. **AND IT PAID ON ITS FIRST INTENDED USE**: the fourth
# citation was typed as `19353`, which is `g = read("sensed")`; the argument the sentence is about
# is one line up. A citation that is wrong AT BIRTH is the one failure this guard's own docstring
# says it cannot catch -- and it caught this one, because a NEW anchor prints the line it lands on
# and the printed text did not match the sentence.
#
# RE-BLESSED AT SLICE AG STEP 4 (9/50/38 -> 10/59/47), AND **THE GUARD HAD BEEN RED FOR TWO
# STEPS BEFORE ANYONE LOOKED.** Step 2 cited `engine.py:18694` in `windup_march`'s doc and
# never re-blessed; steps 2 and 3 both reported `cargo test` alone and neither ran `pytest`,
# so an instrument shipped one step earlier TO CATCH DOCUMENTATION DECAY went two steps
# unconsulted while failing. Of step 4's nine new anchors, eight are step 4's own
# (`4694`, `19283`, `19301`, `19351`, `19373`, `19375`, `19591`, `19598`) and one is step
# 2's arrears. Both asserts that would have caught it — the blessed-set equality and this
# census — were already here and green-by-not-being-run.
FILES = 10
SITES = 63
LINES = 51

# Citations quoted as HISTORY: a comment that reports what an earlier comment
# SAID, where the number is part of the quotation and must not track the file.
# Both entries were created by the commit that wrote this guard — one is the
# stale `17816` slice AF wrote and slice AG quotes while correcting it, the other
# the `19022` that named the right line under the numbering of its own day.
# Growing this set is a decision, not a fix.
HISTORICAL = frozenset({
    ("three_loop.rs", 17816),
    ("two_spool_transient.rs", 19022),
})


# --- the instrument ----------------------------------------------------------------------

def _blocks(lines):
    """Contiguous runs of `//`-comment lines, as `(start_index, text)`."""
    out, i = [], 0
    while i < len(lines):
        if lines[i].lstrip().startswith("//"):
            j = i
            while j < len(lines) and lines[j].lstrip().startswith("//"):
                j += 1
            out.append((i, "\n".join(lines[i:j])))
            i = j
        else:
            i += 1
    return out


def citations(src_dir=RUST_SRC, n_engine_lines=None):
    """Every `engine.py` line citation in the port, as `(file, number, form)`.

    TWO forms, and the second is why the first sweep of this defect under-counted
    by four: `engine.py:N` anywhere in a comment, and a bare `` `N` `` inside a
    comment BLOCK that names `engine.py` somewhere. The block scope is what makes
    the bare form decidable — `(`engine.py:17967`/`17991`)` cites two lines and
    only one of them carries the file name.
    """
    out = []
    for fn in sorted(os.listdir(src_dir)):
        if not fn.endswith(".rs"):
            continue
        text = io.open(os.path.join(src_dir, fn), encoding="utf-8").read()
        for _, block in _blocks(text.split("\n")):
            if "engine.py" not in block:
                continue
            for m in re.finditer(r"engine\.py:(\d+)", block):
                out.append((fn, int(m.group(1)), "engine.py:N"))
            for m in re.finditer(r"`(\d{4,5})`", block):
                n = int(m.group(1))
                if n_engine_lines is None or 1000 <= n <= n_engine_lines:
                    out.append((fn, n, "bare"))
    return out


def _engine_lines(path=ENGINE_PY):
    return io.open(path, encoding="utf-8").read().split("\n")


def census(src_dir=RUST_SRC, engine_path=ENGINE_PY, historical=HISTORICAL):
    """`(files, sites, distinct_lines, checked)` — `checked` drops the history."""
    lines = _engine_lines(engine_path)
    found = citations(src_dir, len(lines))
    checked = [c for c in found if (c[0], c[1]) not in historical]
    return (len({c[0] for c in found}), len(found),
            len({c[1] for c in checked}), checked)


def drift(anchors, src_dir=RUST_SRC, engine_path=ENGINE_PY, historical=HISTORICAL):
    """Citations whose line no longer says what was blessed, and where it went.

    Each record is `(files, n, blessed, now, moved_to)`. `moved_to` is the line
    the blessed text sits at today — the repair — or `None` if the text is gone,
    which means the Python was edited rather than merely moved.
    """
    lines = _engine_lines(engine_path)
    _, _, _, checked = census(src_dir, engine_path, historical)
    by_line = {}
    for fn, n, _form in checked:
        by_line.setdefault(n, set()).add(fn)
    bad = []
    for n in sorted(by_line):
        key = str(n)
        if key not in anchors:
            bad.append((sorted(by_line[n]), n, None, _at(lines, n), None))
            continue
        blessed = anchors[key]
        if _at(lines, n) == blessed:
            continue
        moved = next((k + 1 for k, ln in enumerate(lines) if ln.strip() == blessed
                      and abs(k + 1 - n) <= 200), None)
        bad.append((sorted(by_line[n]), n, blessed, _at(lines, n), moved))
    return bad


def _at(lines, n):
    return lines[n - 1].strip() if 0 < n <= len(lines) else "<out of range>"


def _load():
    return json.load(io.open(ANCHORS, encoding="utf-8"))["lines"]


# --- the gates ---------------------------------------------------------------------------

def _report(rec):
    fs, n, blessed, now, moved = rec
    who = "/".join(fs)
    if blessed is None:
        return f"{who} cites {n}: NEVER BLESSED (now {now!r})"
    where = (f"the blessed text is at {moved} now, which is the repair" if moved
             else "the blessed text is GONE; the Python was edited, not moved")
    return f"{who} cites {n}: blessed {blessed!r}, now {now!r} -- {where}"


def test_every_citation_still_points_where_it_was_blessed():
    bad = drift(_load())
    assert not bad, (
        "an `engine.py` line citation in the Rust port has drifted:\n  "
        + "\n  ".join(_report(r) for r in bad)
        + "\nFix the COMMENT, then run this file directly to re-bless.")


def test_the_blessed_set_is_exactly_the_cited_set():
    """A stale anchor is an anchor for a citation nobody makes any more."""
    anchors = _load()
    _, _, _, checked = census()
    cited = {str(n) for _fn, n, _f in checked}
    assert set(anchors) == cited, (
        f"anchors moved.\n  unblessed citations: {sorted(cited - set(anchors))}\n"
        f"  anchors nothing cites: {sorted(set(anchors) - cited)}\n"
        "Run this file directly to re-bless, after checking each new citation by hand.")


def test_the_census_is_the_size_it_was_measured():
    """A guard asserting `no drift` passes when the scanner finds nothing at all."""
    files, sites, lines, _ = census()
    assert (files, sites, lines) == (FILES, SITES, LINES), (
        f"census moved: {files} files / {sites} sites / {lines} distinct lines, "
        f"expected {FILES} / {SITES} / {LINES}. If that is real growth, run this file "
        "directly and type the numbers off ITS OWN run.")


def test_the_historical_set_is_frozen_and_all_of_it_is_cited():
    """A history exemption for a citation nobody writes is an exemption for nothing."""
    found = {(fn, n) for fn, n, _f in citations(RUST_SRC, len(_engine_lines()))}
    assert HISTORICAL <= found, sorted(HISTORICAL - found)


def test_the_instrument_can_see_a_shift(tmp_path):
    """A drift detector that has never reported a drift is not a detector.

    Both halves are exercised: a line pushed DOWN by an insertion above it, which
    is the whole of what happened 13 times in the real defect, and a line whose
    text was EDITED, where there is no repair to suggest and the message has to
    say so instead of inventing one.
    """
    lines = [f"row_{i} = {i}" for i in range(1, 1201)]     # every line unique, on purpose:
    rs = tmp_path / "probe.rs"                             # a real `engine.py` repeats itself,
    rs.write_text("/// engine.py:1100 and `1150`\n", encoding="utf-8")   # and a `moved_to` for a
    anchors = {"1100": _at(lines, 1100), "1150": _at(lines, 1150)}       # repeated line is a guess.

    shifted = tmp_path / "shifted.py"
    shifted.write_text("\n".join(lines[:1000] + ["# INSERTED"] * 3 + lines[1000:]),
                       encoding="utf-8")
    got = drift(anchors, str(tmp_path), str(shifted))
    assert [(n, moved) for _fs, n, _b, _now, moved in got] == [(1100, 1103), (1150, 1153)], got

    edited = tmp_path / "edited.py"
    body = list(lines)
    body[1099] = "# THE LINE THIS CITATION POINTED AT IS GONE"
    edited.write_text("\n".join(body), encoding="utf-8")
    got = drift(anchors, str(tmp_path), str(edited))
    assert [(n, moved) for _fs, n, _b, _now, moved in got] == [(1100, None)], got


def _bless():
    lines = _engine_lines()
    files, sites, distinct, checked = census()
    anchors = {str(n): _at(lines, n) for _fn, n, _f in checked}
    old = _load() if os.path.exists(ANCHORS) else {}
    for k in sorted(set(anchors) | set(old)):
        if old.get(k) != anchors.get(k):
            print(f"  {k}: {old.get(k)!r}\n     -> {anchors.get(k)!r}")
    json.dump({"_": "Blessed by tests/test_rust_line_citations.py -- see its docstring.",
               "lines": dict(sorted(anchors.items(), key=lambda kv: int(kv[0])))},
              io.open(ANCHORS, "w", encoding="utf-8", newline="\n"), indent=1)
    return files, sites, distinct


if __name__ == "__main__":
    print("re-blessing; every changed anchor is printed above the census:")
    f, s, d = _bless()
    print(f"rust/src files with a citation : {f}")
    print(f"citation sites                 : {s}")
    print(f"distinct engine.py lines       : {d}")
    print(f"quoted as history (unchecked)  : {len(HISTORICAL)}")
