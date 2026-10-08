"""Emit (float.hex, '%g' % x) for the py_g gate. Run on PyPy and CPython; outputs must match."""
import random, struct

vals = [0.004, 0.3, 1e-12, 0.0198, 1.0, 1e-7,
        1e-4, 1e-5, 9.99999e-5, 9.999995e-5, 9.9999951e-5, 0.000099999949,
        123456.0, 1234567.0, 999999.0, 999999.5, 999999.4, 99999.95, 100000.0, 100.0, 10.0,
        0.0, -0.0, -1.0, -0.004, -1e-7, 5e-324, 2.2250738585072014e-308, 1.7976931348623157e308,
        1234565.0, 1000005.0, 9999995.0, 1234575.0, 12345650.0, 0.5, 2.5, 1.5, 0.125, 0.0001,
        3.0, 33.0, 0.05, 0.08, 0.2, 0.35, 0.76, 1e-9, 1e16, 1e22, 1e100, 1e-300, 123.456789,
        float('nan'), float('inf'), float('-inf')]
# exact 6-significant-digit ties: the 7th significant digit is exactly 5 and the value is an
# exact binary integer, so no representation error hides the tie
for n in range(100000, 1000000, 7919):
    for k in range(0, 4):
        vals.append((n * 10 + 5) * 10.0 ** k)
rng = random.Random(84)
for _ in range(4000):
    x = struct.unpack('<d', struct.pack('<Q', rng.getrandbits(64)))[0]
    vals.append(x)
for _ in range(4000):
    vals.append(rng.uniform(-1, 1) * 10.0 ** rng.randint(-12, 12))
for _ in range(2000):  # short decimals, the kind a knob carries
    vals.append(round(rng.uniform(0, 2), rng.randint(1, 8)))
for x in vals:
    h = 'nan' if x != x else x.hex()
    print(h, '%g' % x)
