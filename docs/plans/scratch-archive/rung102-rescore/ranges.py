import re,sys
D="W:/temp/claude/rung102-rescore"
pats={"C1 F ratio":r"C1: .*F ratio ([\d.]+)","C2 dnuL %":r"C2: .*dnuL/nuL ([-\d.]+) %","C3 dnuH %":r"C3: .*dnuH/nuH ([-\d.]+) %",
"C4 dTt4":r"C4: .*dTt4 ([-\d.]+) K","C5 dF %":r"C5: .*\(([-\d.]+) %\)","C10 dnuH":r"C10: .*dnuH ([-\d.]+)","C10 dnuL":r"C10: .*dnuL ([-\d.]+)",
"C10 match":r"C10: .*match F ratio ([\d.]+)","C11 dnuL %":r"C11: .*dnuL/nuL ([-\d.]+) %","C11 dnuH %":r"C11: .*dnuH/nuH ([-\d.]+) %","C12 maxTt4":r"C12: .*max Tt4 ([\d.]+)",
"C7 LPflow end-ratio":None,"C8 HPflow d":None,"C8 HPpress d":None}
for law in ["pin","ellipse","two","one"]:
    txt=open(f"{D}/score_c_{law}.txt",encoding="utf-8").read()
    out=[]
    for k,p in pats.items():
        if p is None: continue
        v=[float(x) for x in re.findall(p,txt)]
        if v: out.append(f"{k} {min(v):.4g}..{max(v):.4g}")
    for lab,p in [("C7 LPflow lit-unlit",r"end LP flow margin ([-\d.]+) vs ([-\d.]+)"),("C7 LPpress lit-unlit",r"end LP pressure margin ([-\d.]+) vs ([-\d.]+)"),
                  ("C8 HPflow lit-unlit",r"end HP flow margin ([-\d.]+) vs ([-\d.]+)"),("C8 HPpress lit-unlit",r"end HP pressure margin ([-\d.]+) vs ([-\d.]+)")]:
        v=[float(a)-float(b) for a,b in re.findall(p,txt)]
        out.append(f"{lab} {min(v):+.3f}..{max(v):+.3f}")
    print("==",law); print("\n".join("   "+o for o in out))
