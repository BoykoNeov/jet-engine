import json, sys
P = sys.argv[1]
reqs = {}
for l in (x for x in open(P + "/crash_reqs.txt") if x.startswith("REQ")):
    _, i, r = l.rstrip("\n").split("\t", 2); reqs[int(i)] = json.loads(r)["controls"]
old = [l.rstrip("\n").split("\t") for l in open(P + "/co.tsv")]
new = [l.rstrip("\n").split("\t") for l in open(P + "/cn.tsv")]
def uses(c):
    return (c["govern"] and c["hold"] == "thrust" and c["thrust_by"] != "true") or c["accel_on"] or (c["govern"] and c["hold"] == "speed" and c["speed_law"] == "prop")
bad, n, cls = [], 0, {}
for (i, h0, k0), (_, h1, k1) in zip(old, new):
    if h0 == h1: continue
    n += 1; c = reqs[int(i)]
    off = not (c["T0"] == 250 and c["p0"] == 50000 and c["M0"] == 0.85)
    ok = uses(c) and off and c["table_at"] == "design"
    if not ok: bad.append((i, k0, k1, uses(c), off, c["table_at"]))
    key = ("inferred" if c["govern"] and c["hold"] == "thrust" and c["thrust_by"] != "true" else "") + ("+accel" if c["accel_on"] else "") + ("+prop" if c["govern"] and c["hold"] == "speed" and c["speed_law"] == "prop" else "")
    cls[key] = cls.get(key, 0) + 1
# the converse: requests that use a table off design with tables at design and did NOT change
same = sum(1 for (i, h0, _), (_, h1, _) in zip(old, new) if h0 == h1 and uses(reqs[int(i)]) and reqs[int(i)]["table_at"] == "design")
print(n, "differ;", len(bad), "outside (uses a table, off design, tables at design)"); print(cls); print("unchanged though carried-eligible:", same)
for b in bad: print("BAD", b)
