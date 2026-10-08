"""Branch census over the step-2 probe's TSV: which branches the five calls reached."""
import collections
import re
import struct
import sys

rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1], encoding="utf-8")]
by_leaf = collections.defaultdict(collections.Counter)
for p, t in rows:
    leaf = re.sub(r"\.\d+", ".#", p)
    by_leaf[leaf][t if not t.startswith("f:") else ("f:NaN" if t[2:5] in ("7ff", "fff") and t != "f:7ff0000000000000" else "f")] += 1

want = ["n_edge", "worst_miss", "min_margin", "agreement", "skipped.switch", "skipped.regime",
        "control_clip_shared", "tau_f_inert.", "predicted", "measured", "riding4_valid",
        "census.", "by_authority.", "zeros", "mask_leak", "vacuous", "all_differenced",
        "ever_two", "n_invalid", "control_all_gov", "all_fuel", "fuel_side", "masked",
        "authority", "@rig", "march_identical", "n_scored", "n_riding4"]
for leaf in sorted(by_leaf):
    if any(w in leaf for w in want):
        c = by_leaf[leaf]
        print("%-60s %s" % (leaf, dict(c.most_common(8))))
