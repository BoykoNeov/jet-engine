import re, sys
exec(open(r"W:\temp\claude\rung86-probe\score_c.py", encoding="utf-8").read().split("for head, cells, ex in blocks:")[0])
inc_n = phi_n = phi_skipped_len = 0
for head, cells, ex in blocks:
    inc = "IncidenceFloor" in head
    pairs = [((0,0),(0,1)),((1,0),(1,1)),((0,0),(1,0)),((0,1),(1,1))] if inc else [((0,0),(1,0)),((0,1),(1,1))]
    for off, on in pairs:
        a, b = cells[off], cells[on]
        bind = lambda c: c["frac"] == c["frac"] and c["frac"] > 0
        if not (bind(a) and bind(b)): continue
        if a["n"] != b["n"]:
            if not inc: phi_skipped_len += 1
            continue
        if inc: inc_n += 1
        else: phi_n += 1
print("incidence", inc_n, "phi", phi_n, "phi pairs both binding but unequal length", phi_skipped_len)
shorter = longer = 0
for head, cells, ex in blocks:
    if "PhiFloor" not in head: continue
    for off, on in [((0,0),(1,0)),((0,1),(1,1))]:
        a, b = cells[off], cells[on]
        bind = lambda c: c["frac"] == c["frac"] and c["frac"] > 0
        if bind(a) and bind(b) and a["n"] != b["n"]:
            if b["n"] < a["n"]: shorter += 1
            else: longer += 1
print("stator-on run shorter", shorter, "longer", longer)
