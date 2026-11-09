# Moved numbers inside refusal messages: largest absolute and relative change per pair.
import sys, re
def rows(p): return [l.rstrip('\n').split('\t') for l in open(p, encoding='utf-8')]
A, B = rows(sys.argv[1]), rows(sys.argv[2])
num = re.compile(r'-?\d+(?:\.\d+)?')
worst = []
for a, b in zip(A, B):
    if a[2] == b[2] or '"ok":1' in a[2]: continue
    na, nb = [float(x) for x in num.findall(a[2])], [float(x) for x in num.findall(b[2])]
    ch = [(abs(x - y), x, y) for x, y in zip(na, nb) if x != y]
    if ch: worst.append((max(ch), a[0]))
worst.sort(reverse=True)
for (d, x, y), i in worst[:8]: print(f'#{i}: {x} -> {y} (diff {d:g})')
print(len(worst), 'refusals with moved numbers')
