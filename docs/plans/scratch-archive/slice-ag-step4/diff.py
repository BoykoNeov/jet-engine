import io
import sys

py = dict(l.rstrip("\n").split("\t")
          for l in io.open(r"W:\temp\claude\slice-ag-step4\py.tsv") if "\t" in l)
rs = dict(l.rstrip("\n").split("\t")
          for l in io.open(r"W:\temp\claude\slice-ag-step4\rs.tsv") if "\t" in l)
only_py = sorted(set(py) - set(rs))
only_rs = sorted(set(rs) - set(py))
diff = sorted(k for k in py if k in rs and py[k] != rs[k])
print("py keys %d  rs keys %d" % (len(py), len(rs)))
print("only py %d %s" % (len(only_py), only_py[:8]))
print("only rs %d %s" % (len(only_rs), only_rs[:8]))
print("differing %d" % len(diff))
for k in diff[:40]:
    print("   %-40s py=%s rs=%s" % (k, py[k], rs[k]))
sys.exit(1 if (only_py or only_rs or diff) else 0)
