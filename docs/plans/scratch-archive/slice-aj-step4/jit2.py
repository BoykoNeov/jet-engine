import struct, sys, random
sys.path.insert(0, r"W:\Claude_projects\jet engine")
from turbojet.engine import ComponentMap
f = lambda h: struct.unpack("<d", bytes.fromhex(h)[::-1])[0]
hx = lambda x: "%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(0.55)
phi, n = f("3fed1578425ec2e5"), f("3feaef3f5fea9684")
print("cold", hx(HP.eta_c_at(0.88, phi, n)))
random.seed(1)
acc = 0.0
for i in range(200000):
    acc += HP.eta_c_at(0.88, 0.8 + 0.2 * random.random(), 0.8 + 0.1 * random.random())
out = set()
for i in range(5000):
    out.add(hx(HP.eta_c_at(0.88, phi, n)))
print("hot", out)
# count mismatches u**2 vs u*u over the warm loop's kind of inputs, interpreted vs jitted
def sq(x): return x ** 2
bad = 0
for i in range(200000):
    x = random.random() - 0.5
    if sq(x) != x * x: bad += 1
print("hot sq != mul:", bad, "of 200000")
