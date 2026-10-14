import re, sys
txt = open(sys.argv[1], encoding="utf-8").read().splitlines()
blocks = []
i = 0
while i < len(txt):
    L = txt[i]
    if L.startswith(("shaped", "tilted")):
        head = L
        cells = {}
        extra = {}
        j = i + 1
        while j < len(txt) and txt[j].startswith("    "):
            t = txt[j].strip()
            m = re.match(r"S(\d)B(\d) free m_i=(\S+) s=(\S+) tt4stencil=(\S+) minphi=(\S+) \| leg m_i=(\S+) s=(\S+) phi@min=(\S+) v=(\S+) removed=(\S+) sched=(\S+) frac=(\S+) s_eng=(\S+) npts=(\d+)", t)
            if m:
                g = m.groups()
                cells[(int(g[0]), int(g[1]))] = dict(fm=float(g[2]), fs=float(g[3]), fphi=float(g[5]), lm=float(g[6]), ls=float(g[7]),
                                                      phimin=float(g[8]), v=float(g[9]), frac=float(g[12]), n=int(g[14]))
            m = re.match(r"set_point=(\S+) T_c=(\S+)", t)
            if m:
                extra["sp"], extra["tc"] = float(m.group(1)), float(m.group(2))
            m = re.match(r"ledger=.* T_ledger=(\S+) dT=(\S+)", t)
            if m:
                extra["dT"] = float(m.group(2))
            j += 1
        blocks.append((head, cells, extra))
        i = j
    else:
        i += 1

for head, cells, ex in blocks:
    T = re.search(r"T=(\S+)", head).group(1)
    reached = re.search(r"reached=(\S+)", head).group(1)
    npts = re.search(r"npts=\[(.*?)\]", head).group(1)
    key = head.split(" T=")[0]
    out = [f"{key:60s} T={T} reached={reached}"]
    if "IncidenceFloor" in head:
        worst = max(abs(c["lm"] - max(c["fm"], ex["sp"])) if c["lm"] == c["lm"] else float("nan") for c in cells.values())
        out.append(f"C1 worst={worst:.2e} dT={ex.get('dT')}")
    else:
        sp, tc = ex["sp"], ex["tc"]
        msgs = []
        for (s, b), c in sorted(cells.items()):
            binds = c["fphi"] < sp
            if not binds:
                ok = abs(c["lm"] - c["fm"]) < 1e-12
                msgs.append(f"S{s}B{b}:free{'ok' if ok else 'MOVED'}")
            else:
                a = abs(c["phimin"] - sp)
                bb = abs(c["lm"] - (tc - 1 / sp + c["v"]))
                msgs.append(f"S{s}B{b}:bind a={a:.1e} b={bb:.1e} n={c['n']}")
        out.append(" ".join(msgs))
    # C3 fractions
    fr = {k: (c["frac"], c["n"]) for k, c in cells.items()}
    out.append("frac " + " ".join(f"S{s}B{b}={fr[(s,b)][0]:.4e}/{fr[(s,b)][1]}" for (s, b) in sorted(fr)))
    print(" | ".join(out))

# ---- C3 and step-size, appended
print("=== C3 (equal-length, both binding) ===")
hits = misses = 0
for head, cells, ex in blocks:
    inc = "IncidenceFloor" in head
    def binds(c):
        return c["frac"] > 0 if c["frac"] == c["frac"] else False
    pairs = [((0,0),(0,1),"B"), ((1,0),(1,1),"B"), ((0,0),(1,0),"S"), ((0,1),(1,1),"S")]
    for off, on, lev in pairs:
        if not inc and lev == "B":
            continue
        a, b = cells[off], cells[on]
        if a["n"] != b["n"] or not (binds(a) and binds(b)):
            continue
        ok = (b["frac"] < a["frac"]) if inc else (b["frac"] > a["frac"])
        hits += ok; misses += (not ok)
        if not ok:
            print("MISS", head.split(" T=")[0], off, on, a["frac"], b["frac"])
print("C3 hits", hits, "misses", misses)
print("=== step size: T at ds 0.005 vs 0.0025 ===")
byk = {}
for head, cells, ex in blocks:
    k = re.sub(r" ds=\S+", "", head.split(" T=")[0])
    byk.setdefault(k, []).append((float(re.search(r"T=(\S+)", head).group(1)), cells))
for k, v in byk.items():
    if len(v) == 2:
        (t1, c1), (t2, c2) = v
        if "PhiFloor" in k:
            bb = []
            for key in [(1,0),(1,1)]:
                for c, tag in [(c1, "a"), (c2, "b")]:
                    pass
        print(f"{k:55s} {t1:+.6e} {t2:+.6e} rel={(t2-t1)/t1 if t1 else 0:+.3f}")
