import os, sys, struct
REPO = r"W:\Claude_projects\jet engine"
sys.path.insert(0, REPO); sys.path.insert(0, os.path.join(REPO, "tests"))
import turbojet.engine as E
from test_rung84 import _rig, _kw, build_two_spool_turbojet, _cpg, PI_LPC, PI_HPC, TT4, FLIGHT, REAL
dsg = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
hx = lambda x: "%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]
LOG = []
orig = E._illinois


TARGET = "3fe1e919fdf4ec59 3fe330c8126fcda1 bfad8a24e69e79c0 3f8c7cdaa7d93340"
CAUGHT = []


def logged(f, a, b, fa, fb, tol=1e-10, maxit=100):
    if "%s %s %s %s" % (hx(a), hx(b), hx(fa), hx(fb)) == TARGET and not CAUGHT:
        cells = {n: c.cell_contents for n, c in zip(f.__code__.co_freevars, f.__closure__ or ())}
        CAUGHT.append(cells)
    LOG.append("CALL %s %s %s %s %s" % (hx(a), hx(b), hx(fa), hx(fb), hx(tol)))

    hit = bool(CAUGHT) and len(CAUGHT) == 1 and "%s %s %s %s" % (hx(a), hx(b), hx(fa), hx(fb)) == TARGET

    def g(x):
        if hit and hx(x) == "3fe2f28cabcfb8ba" and len(CAUGHT) == 1:
            ev = CAUGHT[0]["ev"]
            CAUGHT.append(ev(x))
            me = {n: c.cell_contents for n, c in zip(ev.__code__.co_freevars, ev.__closure__)}["self"]
            mp = me.map_hp
            print("MAP", type(mp).__name__, {k: (hx(v) if isinstance(v, float) else v) for k, v in vars(mp).items()})
            print("BASE", hx(me.eta_hpc), type(me).__mro__[0].__name__)
            ph, nh = CAUGHT[1]["phi_hp"], CAUGHT[1]["n_hp"]
            print("EVAL", hx(mp.eta_c_at(me.eta_hpc, ph, nh)), "method", type(mp).eta_c_at)
        y = f(x)
        LOG.append("EV %s %s" % (hx(x), hx(y)))
        return y
    return orig(g, a, b, fa, fb, tol, maxit)


E._illinois = logged
kw = _kw(0.25); kw.pop("ds"); kw.pop("r")
m = _rig(dsg)
m._split_march(kw["flight"], kw["Tt4_lo"], kw["Tt4_hi"], kw["Tt4_max"], kw["phi_lim"],
               kw["phi_air"], "demand", (0.0190 + (0.0206 - 0.0190) * 4 / 8.0, 0.05, 0.05, 0.05),
               0.25, kw["s_settle"], 0.005, kw["v_max"], kw["inc"])
open(sys.argv[1], "w", newline="\n").write("\n".join(LOG) + "\n")
print(len(LOG))
cells = CAUGHT[0]
print("g freevars", sorted(cells))
ev = cells["ev"]
evc = {n: c.cell_contents for n, c in zip(ev.__code__.co_freevars, ev.__closure__ or ())}
for k, v in sorted(evc.items()):
    if isinstance(v, float): print("in", k, hx(v), repr(v))
    else: print("in", k, type(v).__name__)
out = CAUGHT[1]
for k, v in out.items():
    if isinstance(v, float): print("ZZ", k, hx(v), repr(v))
