import io

t = io.open(r"M:\claud_projects\jet engine\turbojet\engine.py", encoding="utf-8").read().split("\n")
# `_cap_rows` (def at 19320) through the end of `solve_gain` (last line 19619).
seg = t[19319:19619]
tot = len(seg)
blank = sum(1 for l in seg if not l.strip())
comment = sum(1 for l in seg if l.strip().startswith("#"))
# docstrings: crude but adequate -- count lines inside triple-quoted runs
doc, inside = 0, False
for l in seg:
    q = l.count('"""')
    if inside:
        doc += 1
        if q:
            inside = False
        continue
    if q == 1:
        doc += 1
        inside = True
    elif q >= 2:
        doc += 1
print("python 19320-19619: total %d, blank %d, comment %d, docstring %d, CODE %d"
      % (tot, blank, comment, doc, tot - blank - comment - doc))

r = io.open(r"W:\temp\claude\slice-ag-step5\append.rs", encoding="utf-8").read().split("\n")
rt = len(r)
rb = sum(1 for l in r if not l.strip())
rc = sum(1 for l in r if l.lstrip().startswith("//"))
print("rust added: total %d, blank %d, comment %d, CODE %d" % (rt, rb, rc, rt - rb - rc))
