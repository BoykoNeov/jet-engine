"""Lift steps 2–4's flatteners out of the four slice_aj_*.rs files into tests/slice_aj_flat/mod.rs."""
import re
import sys

T = r"W:\Claude_projects\jet engine\rust\tests"
FLAT_MARK = "// ---------------------------------------------------------------------------- the flattening"
CMP_MARK = "// ---------------------------------------------------------------------------- the comparison"


def read(name):
    with open("%s\\%s.rs" % (T, name), encoding="utf-8", newline="") as fh:
        return fh.read()


def block_end(lines, i):
    """Index of the column-0 `}` closing the item that starts at line i."""
    for j in range(i, len(lines)):
        if lines[j] == "}":
            return j
    raise AssertionError("no closing brace from line %d" % i)


def cut(lines, start_pat, with_docs=True):
    """Remove the first item whose line starts with start_pat (plus its doc comment above)."""
    i = next(k for k, l in enumerate(lines) if l.startswith(start_pat))
    j = block_end(lines, i)
    while with_docs and i > 0 and lines[i - 1].startswith("///"):
        i -= 1
    removed = lines[i:j + 1]
    del lines[i:j + 1]
    return removed


shared_parts = []
for name in ("slice_aj_clock", "slice_aj_threshold", "slice_aj_corrector", "slice_aj_staircase"):
    text = read(name)
    assert "\r\n" not in text, "%s has CRLF" % name
    lines = text.split("\n")
    a = lines.index(FLAT_MARK)
    b = lines.index(CMP_MARK)
    sec = lines[a + 1:b]
    keep_local = []
    # the Flat struct + its first impl — replaced by the shared superset
    i = sec.index("#[derive(Default)]")
    assert sec[i + 1].startswith("struct Flat(")
    j = block_end(sec, i + 2)
    del sec[i:j + 1]
    if name == "slice_aj_staircase":
        k = next(n for n, l in enumerate(sec) if l.startswith("const SPACING"))
        keep_local = sec[k - 1:k + 1]
        assert keep_local[0].startswith("///"), keep_local
        del sec[k - 1:k + 1]
        cut(sec, "impl Flat {", with_docs=False)
    if name == "slice_aj_corrector":
        cut(sec, "fn scan(")                     # identical to slice_aj_threshold's
    if name == "slice_aj_threshold":
        cut(sec, "fn rig_after(")                # identical to slice_aj_clock's
    body = "\n".join(sec).strip("\n")
    body = re.sub(r"(?m)^fn ", "pub fn ", body)
    assert "struct Flat" not in body and "impl Flat" not in body, name
    shared_parts.append("// ======================================== from `%s.rs` (step %s)\n\n%s\n"
                        % (name, {"slice_aj_clock": 2, "slice_aj_threshold": 3,
                                  "slice_aj_corrector": 4, "slice_aj_staircase": 4}[name], body))
    new_sec = [
        FLAT_MARK,
        "//",
        "// Every flattener lives in `tests/slice_aj_flat/mod.rs` since step 6: this file's gates are",
        "// what VERIFY them, and `slice_aj_oracle.rs` reuses them, so no struct has two descriptions.",
        "",
        "mod slice_aj_flat;",
        "use slice_aj_flat::*;",
        "",
    ] + ((keep_local + [""]) if keep_local else [])
    lines[a:b] = new_sec
    with open("%s\\%s.rs" % (T, name), "w", encoding="utf-8", newline="") as fh:
        fh.write("\n".join(lines))
    print(name, "moved", len(sec), "lines")

with open(r"W:\temp\claude\slice-aj-step6\shared_body.rs", "w", encoding="utf-8", newline="") as fh:
    fh.write("\n".join(shared_parts))
print("ok")
