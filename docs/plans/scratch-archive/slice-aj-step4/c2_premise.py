import struct
T = r"W:/Claude_projects/jet engine/rust/oracle/slice_aj_step4_pypy.tsv"
def f(tok): return struct.unpack("<d", struct.pack("<Q", int(tok[2:], 16)))[0]
d = {}
for line in open(T):
    p, t = line.rstrip("\n").split("\t")
    d[p] = t
for name in ("read_ctl", "read_v1"):
    k = d.get(name + ".kappa"); print(name, "kappa", k, "F", d.get(name + ".F"), "g", d.get(name + ".g"))
    if not (k and k.startswith("f:") and d[name + ".h"].startswith("f:")): continue
    tau = f(d[name + ".tau_f"]); t = f(d[name + ".tau_hat_min"]); kap = f(k); h = f(d[name + ".h"])
    F83 = t / kap; g83 = F83 - tau
    g84 = h / kap; F84 = g84 + tau
    print("  F83==F84", F83 == F84, "g83==g84", g83 == g84, F83, F84, g83, g84)
