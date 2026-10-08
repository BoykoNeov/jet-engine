"""Key-by-key diff of the two step-5 drives. Both sides emit `key\\tu64`, the u64 being an
IEEE-754 bit pattern for a float key and a plain integer / FNV-1a hash otherwise."""
import io
import sys

BASE = r"W:\temp\claude\slice-ag-step5"


def load(p):
    out = {}
    for line in io.open(p, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        k, v = line.split("\t")
        out[k] = int(v)
    return out


py = load(BASE + r"\py.tsv")
rs = load(BASE + r"\rs.tsv")
only_py = sorted(set(py) - set(rs))
only_rs = sorted(set(rs) - set(py))
both = sorted(set(py) & set(rs))
bad = [k for k in both if py[k] != rs[k]]
print("py keys      %d" % len(py))
print("rs keys      %d" % len(rs))
print("shared       %d" % len(both))
print("only in py   %d" % len(only_py))
print("only in rs   %d" % len(only_rs))
print("MISMATCHED   %d" % len(bad))
for k in only_py[:20]:
    print("  PY-ONLY %s" % k)
for k in only_rs[:20]:
    print("  RS-ONLY %s" % k)
import struct
for k in bad[:40]:
    a = struct.unpack("<d", struct.pack("<Q", py[k]))[0]
    b = struct.unpack("<d", struct.pack("<Q", rs[k]))[0]
    print("  DIFF %-44s py=%-24r rs=%-24r  (%d vs %d)" % (k, a, b, py[k], rs[k]))
sys.exit(1 if (bad or only_py or only_rs) else 0)
