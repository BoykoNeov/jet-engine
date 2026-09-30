"""Plan § 5.34.6 (e): where step 4's C6 mutant — `residual_shape`'s ladder respelled
`lo + i*step` (step = (hi-lo)/(n-1)) instead of `lo + (hi-lo)*i/(n-1)` — yields a DIFFERENT point.

Pure float arithmetic, interpreter-independent (no `**`, no `sum`). The grid is the one the
figure was first computed on: lo in {0.004, 0.01, 0.016, 0.019, 0.02, 0.03, 0.037}, hi in
{0.024, 0.03, 0.05, 0.1, 0.3}, hi > lo, n = 3..21. Measured 2026-09-30: 271 of 589 ladders
separate the spellings (e.g. [0.004, 0.024] at n = 11, point 9); the oracle's two n = 4 ladders
([0.016, 0.024] and [0.004, 0.05]) separate at NO point — which is why the mutant survived it.

A separating point is a DIFFERENT INPUT to `corrector_read`; that it moves a reported value is a
prediction, not shown here.

    python rust/oracle/probe_slice_aj_step6_ladder.py
"""
import itertools


def sep(lo, hi, n):
    step = (hi - lo) / (n - 1)
    return [i for i in range(n) if lo + (hi - lo) * i / (n - 1) != lo + i * step]


tot = hit = 0
for lo, hi in itertools.product([0.004, 0.01, 0.016, 0.019, 0.02, 0.03, 0.037],
                                [0.024, 0.03, 0.05, 0.1, 0.3]):
    if hi <= lo:
        continue
    for n in range(3, 22):
        tot += 1
        hit += bool(sep(lo, hi, n))
print("separating ladders:", hit, "of", tot)
print("example [0.004, 0.024] n=11:", sep(0.004, 0.024, 11))
print("oracle's n=4 ladders:", sep(0.016, 0.024, 4), sep(0.004, 0.05, 4))
