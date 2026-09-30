"""Plan § 5.34.6 (c): how often CPython's `(x - 1.0) ** 2` disagrees with `(x - 1.0) * (x - 1.0)`
inside `ComponentMap.eta_c_at` / `eta_t_at` on ONE ordinary march (value-neutral: the wrapper
returns the original). Measured on CPython 3.14.3, 2026-09-30: eta_c 1 291 982 calls, 26 differ;
eta_t 55 242, 0 — so a per-reading census flags every reading and predicts nothing.

    C:\\Python314\\python.exe rust/oracle/probe_slice_aj_step6_powrate.py
"""
import sys, time, struct
print(sys.version.replace("\n", " "), flush=True)
REPO = r"W:\Claude_projects\jet engine"
sys.path.insert(0, REPO); sys.path.insert(0, REPO + r"\tests")
from test_rung84 import FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, PI_LPC, PI_HPC, TT4, REAL, _cpg, _rig
from turbojet.engine import build_two_spool_turbojet, ComponentMap
C = {"eta_c": [0, 0], "eta_t": [0, 0], "psi": [0, 0]}
oc, ot, op = ComponentMap.eta_c_at, ComponentMap.eta_t_at, ComponentMap.psi
def bits(x): return struct.unpack("<Q", struct.pack("<d", x))[0]
def eta_c(self, base, fc, n):
    v = oc(self, base, fc, n)
    a = fc - 1.0; b = n - 1.0
    w = base - self.a * (a * a) - self.b * (b * b) - self.c * a * b
    C["eta_c"][0] += 1; C["eta_c"][1] += bits(v) != bits(w)
    return v
def eta_t(self, base, nu):
    v = ot(self, base, nu); a = nu - 1.0
    w = base - self.a_t * (a * a)
    C["eta_t"][0] += 1; C["eta_t"][1] += bits(v) != bits(w)
    return v
ComponentMap.eta_c_at, ComponentMap.eta_t_at = eta_c, eta_t
D = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
m = _rig(D)
t0 = time.time()
s = m._threshold_scan(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=PHI_FUEL, phi_air=PHI_AIR,
    tau_f=0.05, tau_gov=0.05, tau_q=0.05, tau_s=0.05, r=0.35, s_settle=1.2, ds=0.005, v_max=0.20, inc=False)
print("%.1fs" % (time.time() - t0), C, s["h"].hex(), flush=True)
