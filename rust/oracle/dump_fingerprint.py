"""SLICE AS — the numeric fingerprint's 45 kernels, recorded on PyPy in the FINGERPRINT's OWN key space.

    .venv\\Scripts\\python.exe rust/oracle/dump_fingerprint.py rust/oracle/fingerprint_pypy.tsv

Launch through PowerShell `Start-Process -FilePath <interpreter>`, never a quoted `cmd start`
(plan § 5.34.4 (d)). The ENTRY CONTROL below refuses an output whose name says `pypy` written by
any other interpreter, and the first line printed is `sys.version`.

WHAT THIS IS. `tests/test_numeric_fingerprint.py` is the project's only ABSOLUTE-value gate: 45
kernels whose values are compared against a committed CPython golden. Plan § 8.1 (vi) re-anchors it
on Rust. This script does NOT re-implement a kernel — it imports that module and calls its own
`KERNELS` table, so every key name (the `dir()` walk of `_floats_of`, the `:g` spellings, `_flat`'s
`[i]` / `#n` / `.re` / `.im`, r76's three dropped `_UNSTABLE` keys) is the gate's own. The output
serves three purposes:

1. THE KEY SET the Rust port must reproduce, exactly — not reconstructed by hand.
2. THE PyPy-vs-CPython DEVIATION, measured directly against `tests/golden/numeric_fingerprint.json`
   under the module's own `TOL` / `ABS_TOL` and `_close` (printed at the end, per kernel).
3. THE BIT-EXACT TARGET for `rust/tests/fingerprint.rs`: Rust == PyPy is the port's spine, so a
   wrong key or a wrong value fails there at its name rather than as a tolerance miss.

FORMAT: one line per value, `kernel <TAB> key <TAB> token`, kernels in `KERNELS` order and keys
sorted. Tokens: `f:<16 hex digits>` (IEEE bits; `f:nan` for a NaN), `i:<int>`, `b:0|1`, `s:<str>`,
`n` (None), and `j:<json>` for a list (only r66's `keys` and `edges.*`, ints / strings).
After the kernels, two pseudo-kernels `_tol` / `_abs_tol` carry the module's own `TOL` /
`ABS_TOL` per kernel (an absent `ABS_TOL` entry written as `0.0`, which is what `.get` returns).
"""
import json
import os
import platform
import struct
import sys
import time

print(sys.version.replace("\n", " "), flush=True)

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, REPO)
sys.path.insert(0, os.path.join(REPO, "tests"))

PATH = sys.argv[1] if len(sys.argv) > 1 else None
assert PATH, "usage: dump_fingerprint.py <out.tsv>"
_IMPL = platform.python_implementation()
_base = os.path.basename(PATH).lower()
assert not ("pypy" in _base and _IMPL != "PyPy"), "a *pypy* output written by %s" % _IMPL
assert not ("cpython" in _base and _IMPL != "CPython"), "a *cpython* output written by %s" % _IMPL

import test_numeric_fingerprint as fp  # noqa: E402


def tok(x):
    if x is None:
        return "n"
    if isinstance(x, bool):
        return "b:%d" % int(x)
    if isinstance(x, int):
        return "i:%d" % x
    if isinstance(x, float):
        if x != x:
            return "f:nan"
        return "f:%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]
    if isinstance(x, str):
        assert "\t" not in x and "\n" not in x, x
        return "s:" + x
    if isinstance(x, list):
        assert all(isinstance(y, (int, str)) and not isinstance(y, bool) for y in x), x
        return "j:" + json.dumps(x)
    raise TypeError((type(x), x))


golden = fp._load_golden()["kernels"]
lines, report = [], []
t_all = time.time()
for name, fn in fp.KERNELS.items():
    t0 = time.time()
    got = fn()
    dt = time.time() - t0
    for k in sorted(got):
        assert "\t" not in k and "\n" not in k, k
        lines.append("%s\t%s\t%s" % (name, k, tok(got[k])))
    # THE DEVIATION, under the gate's own comparison — the same `_close` the pytest gate runs.
    want = {k: fp._decode(v) for k, v in golden[name].items()}
    assert set(want) == set(got), (name, sorted(set(want) ^ set(got))[:8])
    tol, atol = fp.TOL[name], fp.ABS_TOL.get(name, 0.0)
    n_diff = n_bad = 0
    max_rel = max_abs = 0.0
    for k in want:
        a, b = got[k], want[k]
        if a == b:
            continue
        n_diff += 1
        ok, _ = fp._close(a, b, tol, atol)
        n_bad += not ok
        if isinstance(a, float) and isinstance(b, float):
            max_abs = max(max_abs, abs(a - b))
            max_rel = max(max_rel, abs(a - b) / abs(b) if b != 0.0 else abs(a))
    report.append((name, len(want), n_diff, n_bad, max_rel, max_abs, tol, atol, dt))
    print("  %-5s %6d keys  %5d differ  %d beyond  rel %.2e  abs %.2e  (%.1f s)"
          % (name, len(want), n_diff, n_bad, max_rel, max_abs, dt), flush=True)

# THE TOLERANCE TABLES, from the module itself — so the Rust transcription is checked against the
# source rather than against a second hand copy, and survives the Python module's deletion.
for name in fp.KERNELS:
    lines.append("_tol\t%s\t%s" % (name, tok(float(fp.TOL[name]))))
    lines.append("_abs_tol\t%s\t%s" % (name, tok(float(fp.ABS_TOL.get(name, 0.0)))))
assert set(fp.TOL) == set(fp.KERNELS) and set(fp.ABS_TOL) <= set(fp.KERNELS)

with open(PATH, "w", encoding="utf-8", newline="\n") as fh:
    fh.write("\n".join(lines) + "\n")
    fh.flush()
    os.fsync(fh.fileno())

print("wrote %d values over %d kernels in %.1f s; %d beyond tolerance"
      % (len(lines), len(report), time.time() - t_all, sum(r[3] for r in report)), flush=True)
