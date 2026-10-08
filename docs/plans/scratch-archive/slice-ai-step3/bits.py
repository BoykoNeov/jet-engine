import json, struct
d = json.load(open(r"W:\temp\claude\slice-ai-step3\py.json"))
b = lambda h: "0x%016x" % struct.unpack("<Q", struct.pack("<d", float.fromhex(h)))[0]
m = d["march"]
for k in ("gap_min", "gap_med", "gap_max"):
    print(k, b(m[k]))
f = d["forced"]
for k in ("d_forced", "d_forced_med", "d_shipped"):
    print(k, b(f[k]))
for r in f["rows"]:
    print("    [" + ", ".join(b(r[k]) for k in ("s", "w_phi", "w_inc", "d_forced")) + "],", r["same_float"])
print("first", [b(x) if isinstance(x, str) else x for x in d["log_first"]])
print("last", [b(x) if isinstance(x, str) else x for x in d["log_last"]])
