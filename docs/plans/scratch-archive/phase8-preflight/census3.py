import re, pathlib
R = pathlib.Path(r"W:\Claude_projects\jet engine")
alltxt = "\n".join(f.read_text(encoding="utf-8") for f in (R/"rust/tests").glob("*.rs"))
alias = {"test_stations":"rung1","test_polytropic":"rung2b","test_variable_cp":"rung3","test_reacting":"rung4","test_forkb":"rung5"}
tot=hit_own=hit_any=0; miss=[]
for f in sorted((R/"tests").glob("test_*.py")):
    k=f.stem; m=re.match(r"test_rung(\d+b?)$",k); tgt=alias.get(k) or (("rung"+m.group(1)) if m else None)
    if not tgt: continue
    own=(R/"rust/tests"/(tgt+".rs")).read_text(encoding="utf-8")
    for name in re.findall(r"^\s*def (test_\w+)", f.read_text(encoding="utf-8"), re.M):
        tot+=1; bare=name[5:]
        if name in own or re.search(r"\b"+re.escape(bare)+r"\b", own): hit_own+=1
        elif name in alltxt: hit_any+=1
        else: miss.append((k,name))
print("defs",tot,"named in own rust file",hit_own,"named elsewhere",hit_any,"unnamed",len(miss))
import collections; c=collections.Counter(k for k,_ in miss); print(sorted(c.items(), key=lambda x:-x[1])[:40])
