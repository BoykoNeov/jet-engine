import struct, sys, os
sys.path.insert(0, r"W:\Claude_projects\jet engine")
from turbojet.engine import ComponentMap
f = lambda h: struct.unpack("<d", bytes.fromhex(h)[::-1])[0]
hx = lambda x: "%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(0.55)
print(type(HP).__name__, HP.a, HP.b, HP.c)
phi, n = f("3fed1578425ec2e5"), f("3feaef3f5fea9684")
print("py eta_c_at   ", hx(HP.eta_c_at(0.88, phi, n)))
u, v = phi - 1.0, n - 1.0
print("u*u spelling  ", hx(0.88 - HP.a * (u * u) - HP.b * (v * v) - HP.c * u * v))
print("u**2 == u*u   ", u ** 2 == u * u, v ** 2 == v * v, hx(u**2), hx(u*u), hx(v**2), hx(v*v))
print("want py 3fec04ba258ccb0d? got above; rust 3fec04ba258ccb0e")
