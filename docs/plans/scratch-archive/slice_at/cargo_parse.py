import re,json
D="W:/temp/claude/slice_at/"
runs=[l.strip() for l in open(D+"cargo_list.err",encoding="utf-8",errors="replace") if re.match(r"\s*(Running|Doc-tests)",l)]
blocks=[];cur=[]
for l in open(D+"cargo_list.txt",encoding="utf-8",errors="replace"):
    l=l.rstrip("\n")
    m=re.match(r"^(\d+) tests?, (\d+) benchmarks",l)
    if m: blocks.append((cur,int(m.group(1))));cur=[];continue
    if l.endswith(": test"): cur.append(l[:-6])
tot=0;per={}
for r,(names,n) in zip(runs,blocks):
    assert len(names)==n,(r,len(names),n)
    m=re.search(r"tests.([A-Za-z0-9_]+)\.rs",r); key=m.group(1) if m else " ".join(r.split()[:3])
    per[key]=names; tot+=n
print(len(runs),len(blocks),"total",tot)
for k,v in per.items():
    if not re.match(r"^[a-z0-9_]+$",k) or any("::" in x for x in v): print(k,len(v),[x for x in v if "::" in x][:4])
json.dump(per,open(D+"cargo_tests.json","w"),indent=0)
