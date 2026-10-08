import struct, sys
f = lambda h: struct.unpack("<d", bytes.fromhex(h)[::-1])[0]
hx = lambda x: "%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]
v = f("3feaef3f5fea9684") - 1.0
def sq(x):
    return x ** 2
seen = []
for i in range(20000):
    r = sq(v)
    if not seen or seen[-1][1] != hx(r):
        seen.append((i, hx(r)))
print("v*v", hx(v * v), "transitions", seen)
