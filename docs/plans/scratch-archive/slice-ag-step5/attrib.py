"""Re-run the three GRID-SENSITIVE injections and bucket the differing keys by (inc, margin).

The first sweep's attribution counted prefixes `E/0.1/` and `F/0.1/`; adding the `inc` axis renamed
every key to `E/i0/0.1/` and the counters silently read 0 everywhere. A stale instrument of exactly
the class this step keeps finding, so it is fixed here rather than in prose.
"""
import hashlib
import io
import subprocess

SRC = r"M:\claud_projects\jet engine\rust\src\sensed_cap.rs"
MANI = r"M:\claud_projects\jet engine\rust\Cargo.toml"
PRISTINE = io.open(SRC, encoding="utf-8", newline="").read()
SHA = hashlib.sha256(PRISTINE.encode("utf-8")).hexdigest()
print("pristine sha256 %s" % SHA)

PY = dict(l.rstrip("\n").split("\t")
          for l in io.open(r"W:\temp\claude\slice-ag-step5\py.tsv") if "\t" in l)

INJ = [
    (3, "accel_binds folds min instead of max",
     "            accel_binds: (if cap_a2 > cap_a { cap_a2 } else { cap_a }) < cap_s,",
     "            accel_binds: (if cap_a2 < cap_a { cap_a2 } else { cap_a }) < cap_s,"),
    (10, "row_err's two targets INVERTED",
     "                            let tgt = if sched { x.c } else { x.c - 1.0 };",
     "                            let tgt = if sched { x.c - 1.0 } else { x.c };"),
    (15, "fixed_point reads solve(q), not sensed(q, w0)",
     "            fixed_point: (sensed(q, w0) - w0).abs(),",
     "            fixed_point: (solve(q) - w0).abs(),"),
    (6, "tau_auth / tau_masked SWAPPED",
     "            tau_auth: if auth == Authority::Fuel { tau_f } else { taus.1 },\n"
     "            tau_masked: if masked == Authority::Fuel { tau_f } else { taus.1 },",
     "            tau_auth: if masked == Authority::Fuel { tau_f } else { taus.1 },\n"
     "            tau_masked: if auth == Authority::Fuel { tau_f } else { taus.1 },"),
]


def bucket(k):
    """`E/i0/0.1/...` -> `i0/0.1`; `G/i1/sched|none/...` -> `i1/G`; `H/i0/0.05/...` -> `i0/0.05`."""
    p = k.split("/")
    if len(p) < 3:
        return "top"
    if p[0] == "G":
        return "%s/G" % p[1]
    return "%s/%s" % (p[1], p[2])


for n, name, old, new in INJ:
    assert PRISTINE.count(old) == 1, (n, name)
    io.open(SRC, "w", encoding="utf-8", newline="").write(PRISTINE.replace(old, new))
    r = subprocess.run(
        'cargo test --release --manifest-path "%s" --test slice_ag_step5_drive' % MANI,
        shell=True, capture_output=True, text=True)
    assert "test result: ok" in r.stdout, (n, r.stdout[-400:] + r.stderr[-400:])
    rs = dict(l.rstrip("\n").split("\t")
              for l in io.open(r"W:\temp\claude\slice-ag-step5\rs.tsv") if "\t" in l)
    d = [k for k in PY if k not in rs or PY[k] != rs[k]]
    counts = {}
    for k in d:
        counts[bucket(k)] = counts.get(bucket(k), 0) + 1
    print("%2d  %-45s %3d keys  %s"
          % (n, name, len(d), "  ".join("%s=%d" % kv for kv in sorted(counts.items()))))
    io.open(SRC, "w", encoding="utf-8", newline="").write(PRISTINE)
    assert hashlib.sha256(
        io.open(SRC, encoding="utf-8", newline="").read().encode("utf-8")).hexdigest() == SHA
