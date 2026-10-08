import random, struct, subprocess, sys
random.seed(84)
xs = []
for _ in range(100000):
    xs.append((random.uniform(0.0, 4.0), 6))
    xs.append((random.uniform(0.0, 2.0), 9))
# adversarial: values within 1 ulp of a decimal half-point at 6 and 9 places
import math
for k in range(1, 3000):
    for nd in (6, 9):
        h = (k + 0.5) / 10**nd
        for d in (-2, -1, 0, 1, 2):
            v = h
            for _ in range(abs(d)):
                v = math.nextafter(v, math.inf if d > 0 else -math.inf)
            xs.append((v, nd))
inp = "\n".join("%016x %d" % (struct.unpack("<Q", struct.pack("<d", x))[0], nd) for x, nd in xs)
res = subprocess.run([r"W:/temp/claude/slice-aj-preflight/rcheck/rcheck.exe"], input=inp, capture_output=True, text=True).stdout.split("\n")
bad = 0
for (x, nd), line in zip(xs, res):
    rb = int(line.split()[2], 16)
    py = struct.unpack("<Q", struct.pack("<d", round(x, nd)))[0]
    if rb != py:
        bad += 1
        if bad <= 5: print("MISMATCH", repr(x), nd, repr(round(x, nd)))
print("stress cases:", len(xs), "mismatches:", bad, "| python:", sys.implementation.name, sys.version.split()[0])
