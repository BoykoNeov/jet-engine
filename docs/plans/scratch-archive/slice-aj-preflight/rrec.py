import json, struct, subprocess, collections, math
from decimal import Decimal
J = json.load(open(r"W:/temp/claude/slice-aj-preflight/p_drive.json"))
rec = J["rec"]
bits = lambda x: struct.unpack("<Q", struct.pack("<d", x))[0]
lines, meta = [], []
for xh, nd, outr in rec:
    x = float.fromhex(xh)
    lines.append("%016x %d" % (bits(x), -1 if nd is None else nd)); meta.append((x, nd, outr))
res = subprocess.run([r"W:/temp/claude/slice-aj-preflight/rcheck/rcheck.exe"], input="\n".join(lines), capture_output=True, text=True).stdout.split("\n")
bad = collections.Counter(); kinds = collections.Counter(); ties = 0; distinct = collections.defaultdict(set)
for (x, nd, outr), line in zip(meta, res):
    kinds[nd] += 1; distinct[nd].add(x)
    r = line.split()[2]
    if nd is None:
        py = int(outr)
        if int(r) != py: bad[nd] += 1
        if (Decimal(x) - Decimal(math.floor(x))) == Decimal("0.5"): ties += 1
    else:
        if int(r, 16) != bits(float.fromhex(outr)): bad[nd] += 1
print("records by ndigits:", dict(kinds), "| distinct args:", {k: len(v) for k, v in distinct.items()})
print("rust mismatches:", dict(bad), "| exact .5 ties at round(x):", ties)
# how far is edge/ds from an integer (the on-grid test uses 1e-9)?
dev = [abs(x - round(x)) for x, nd, _ in meta if nd is None]
print("round(edge/ds): max |x - round(x)| =", max(dev), " values:", sorted({x for x, nd, _ in meta if nd is None})[:6])
# does round(s,9) merge two distinct march points?
s9 = [x for x, nd, _ in meta if nd == 9]
print("round(s,9) args", len(s9), "distinct", len(set(s9)), "distinct after rounding", len({round(v, 9) for v in set(s9)}))
print("round(.,6) outputs:", sorted({round(x, 6) for x, nd, _ in meta if nd == 6}))
# segment: each edge_read = [n_scored x nd=9 (summands)] + [1 x nd=9 (edge)] + [2 x nd=None]
segs, cur = [], []
for xh, nd, outr in rec:
    if nd == 6:
        continue
    cur.append((float.fromhex(xh) if xh.startswith(("0x", "-0x")) else xh, nd))
    if nd is None and len(cur) >= 2 and cur[-2][1] is None:
        segs.append(cur); cur = []
merges = 0; cross_gap = []
for sg in segs:
    summ = [x for x, nd in sg[:-3] if nd == 9]
    if len({round(v, 9) for v in summ}) != len(set(summ)): merges += 1
    ss = sorted(set(summ))
    if len(ss) > 1: cross_gap.append(min(b - a for a, b in zip(ss, ss[1:])))
print("sizes", [len(s) for s in segs]); print("edge_read segments:", len(segs), "| segments with an in-march merge:", merges,
      "| min spacing within a march:", min(cross_gap))
