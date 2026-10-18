import json, sys, collections
P = sys.argv[1]
reqs = {}
for l in open(P + "/crash_reqs.txt", encoding="utf-8"):
    if l.startswith("REQ"):
        _, i, r = l.rstrip("\n").split("\t", 2); reqs[int(i)] = json.loads(r)["controls"]
old = [l.rstrip("\n").split("\t") for l in open(P + "/co_crash.tsv", encoding="utf-8")]
new = [l.rstrip("\n").split("\t") for l in open(P + "/cn_crash.tsv", encoding="utf-8")]
def uses(c):
    return (c["govern"] and c["hold"] == "thrust" and c["thrust_by"] != "true") or c["accel_on"] or (c["govern"] and c["hold"] == "speed" and c["speed_law"] == "prop")
GRID = [0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0]
def eligible(c):
    if not (uses(c) and c["table_at"] == "design" and c["speed_corr"] != "none"): return False
    if c["T0"] == 250 and c["p0"] == 50000 and c["M0"] == 0.85: return False
    if c["speed_corr"] == "grid" and c["M0"] == 0.85: return False
    return True
n, bad, cls = 0, [], collections.Counter()
for (i, h0, k0, b0), (_, h1, k1, b1) in zip(old, new):
    if h0 == h1: continue
    n += 1; c = reqs[int(i)]
    if not eligible(c): bad.append((i, k0, k1, c["speed_corr"], c["M0"]))
    cls[(c["speed_corr"], k0 + " -> " + k1)] += 1
same = sum(1 for (i, h0, _, _), (_, h1, _, _) in zip(old, new) if h0 == h1 and eligible(reqs[int(i)]))
print(n, "differ;", len(bad), "outside the class (reads a table, tables at design, speed corrected, off Mach 0.85 for the grid)")
for k, v in sorted(cls.items()): print(f"  {v:4}  {k}")
print("eligible but unchanged:", same)
for b in bad: print("BAD", b)
