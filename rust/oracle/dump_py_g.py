"""THE `%g` ORACLE, slice AJ step 1 — Python's `'%g' % x` for every value `py_g` must reproduce.

Rungs 82–84 return four messages built with `%g` (`engine.py:22305`, `:22784`, `:23030`,
`:23061`), and the crate had no `%g` formatter — only `py_e` (`%.Ne`) and `py_repr`. This dump
is what `demand_coordinate::py_g` is gated against, in `rust/tests/slice_aj_plumbing.rs`.

Output is TSV, one row per value:  u64-bits <TAB> '%g' % x
Bits, because the input is what must be exact; the output is compared as a string.

THE SET, and why each group is in it:
  * P3's driven arguments (`0.004`, `0.3`, `1e-12`, `0.0198`, `1`) and the shipped default
    `eps = 1e-7`, which Python prints `1e-07` — the two-digit exponent Rust's `{:e}` lacks.
  * both notation boundaries (`1e-4` / `1e-5`, `123456` / `1234567`) and the values that
    CARRY across one when rounded to six digits (`999999.5` -> `1e+06`, `9.999995e-05`).
  * EXACT decimal ties at six significant digits — integers whose seventh digit is exactly 5.
    `round(x, n >= 1)` can never meet an exact tie in a binary float, so the pre-flight's
    229 990-case rounding stress (plan § 5.34 (v)) says nothing about them; `%g` on an integer
    can. Python rounds them half-to-even.
  * zero, negative zero (`-0`), negatives, subnormals, the extremes, NaN and both infinities.
  * 10 000 random values: raw bit patterns, log-uniform magnitudes, short decimals.

Run under BOTH interpreters; the two outputs were byte-identical when this was written.

    C:\\Python314\\python.exe rust/oracle/dump_py_g.py rust/oracle/py_g_cpython.tsv
    .venv\\Scripts\\python.exe  rust/oracle/dump_py_g.py rust/oracle/py_g_pypy.tsv
"""
import random
import struct
import sys

VALS = [0.004, 0.3, 1e-12, 0.0198, 1.0, 1e-7,
        1e-4, 1e-5, 9.99999e-5, 9.999995e-5, 9.9999951e-5, 0.000099999949,
        123456.0, 1234567.0, 999999.0, 999999.5, 999999.4, 99999.95, 100000.0, 100.0, 10.0,
        0.0, -0.0, -1.0, -0.004, -1e-7, 5e-324, 2.2250738585072014e-308, 1.7976931348623157e308,
        1234565.0, 1000005.0, 9999995.0, 1234575.0, 12345650.0, 0.5, 2.5, 1.5, 0.125, 0.0001,
        3.0, 33.0, 0.05, 0.08, 0.2, 0.35, 0.76, 1e-9, 1e16, 1e22, 1e100, 1e-300, 123.456789,
        float("nan"), float("inf"), float("-inf")]

# exact six-significant-digit ties: the seventh significant digit is exactly 5 and the value is
# an exact binary integer, so no representation error can hide the tie
for n in range(100000, 1000000, 7919):
    for k in range(0, 4):
        VALS.append((n * 10 + 5) * 10.0 ** k)

rng = random.Random(84)
for _ in range(4000):
    VALS.append(struct.unpack("<d", struct.pack("<Q", rng.getrandbits(64)))[0])
for _ in range(4000):
    VALS.append(rng.uniform(-1, 1) * 10.0 ** rng.randint(-12, 12))
for _ in range(2000):  # short decimals, the kind a knob carries
    VALS.append(round(rng.uniform(0, 2), rng.randint(1, 8)))

with open(sys.argv[1], "w", newline="\n", encoding="ascii") as out:
    for x in VALS:
        bits = struct.unpack("<Q", struct.pack("<d", x))[0]
        out.write("%d\t%s\n" % (bits, "%g" % x))
