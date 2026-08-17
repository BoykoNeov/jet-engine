"""SLICE N step 2 — the smoke dump for `StageStack` ALONE.

Not the slice's oracle (that is step 4, on the 640-cell / 160-row grid). This exists to catch a
structural mistake — a swapped ladder, a `** k` spelled as a repeated multiply, a floor applied
after the value it guards, a capacity profile that round-trips row 0 through the bisection —
BEFORE rung 55's matcher is written on top of it at step 3.

IT ENUMERATES THE METHODS STEP 2 SHIPS AND COVERS EACH. Slice L step 3's smoke reached 1 of the
3 methods its own headline named; slice M's needed `currency_split` twice because at v = 0 it
could not discriminate the sibling constructor. The specific trap here: a stack built with
`vsv_stages = None` NEVER touches `cmap_axial`, so both second branches of `psi_at`/`vsv_at` are
dead — cells B and C exist to reach them, with a MOVED stator so `cmap.vsv != cmap_axial.vsv`.

The stacks are built from LITERAL (tau_d, pi_d, eta_d, kc) rather than through a matcher, so this
gate is about `StageStack` and nothing else; the values are printed by `#anchor` rows, taken from
a real two-spool design point, and copied into the Rust as literals.

Run:  .venv\Scripts\pypy.exe M:\claud_projects\temp\slice_n\smoke55.py > <tsv>
"""
import os
import sys

sys.path.insert(0, r"M:\claud_projects\jet engine")

from turbojet.gas import Gas
from turbojet.engine import (
    FlightCondition, build_two_spool_turbojet, ComponentMap, StageStack, StageStackMatcher,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)

OUT = []


def emit(key, val):
    OUT.append("%s\t%s" % (key, float(val).hex()))


# --------------------------------------------------------------------------- the anchor
# The design point the literals come from. Printed so the Rust's copies can be checked against
# the machine that produced them rather than trusted.
design = build_two_spool_turbojet(Gas.reacting_equilibrium(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)
MAP_LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
MAP_HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
mm = StageStackMatcher(design, FLIGHT, 1.0, map_lp=MAP_LP, map_hp=MAP_HP)
KC = mm.gas.gamma_c / (mm.gas.gamma_c - 1.0)
for name, v in (("tau_lpc_d", mm.tau_lpc_d), ("pi_lpc_design", mm.pi_lpc_design),
                ("eta_lpc", mm.eta_lpc), ("tau_hpc_d", mm.tau_hpc_d),
                ("pi_hpc_design", mm.pi_hpc_design), ("eta_hpc", mm.eta_hpc), ("kc", KC)):
    emit("anchor/" + name, v)

TAU_LP, PI_LP, ETA_LP = mm.tau_lpc_d, mm.pi_lpc_design, mm.eta_lpc
TAU_HP, PI_HP, ETA_HP = mm.tau_hpc_d, mm.pi_hpc_design, mm.eta_hpc

# --------------------------------------------------------------------------- the four cells
#
#  A  K=8  dT   lumped lever      derived   moved stator   -- cmap_axial DEAD (that is the point)
#  B  K=8  tau  front-row lever   derived   moved stator   -- reaches cmap_axial on rows 1..7
#  C  K=4  dT   two front rows    uniform   opened stator  -- the other profile, v < 0
#  D  K=1  dT   lumped lever      derived   design setting -- the reduce + the solve_n dispatch
#
CELLS = [
    ("A", dict(K=8, cmap=ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7, vsv=0.25,
                                      capacity=0.80),
               tau_d=TAU_LP, pi_d=PI_LP, eta_d=ETA_LP, kc=KC, split="dT",
               vsv_stages=None, cap_profile="derived")),
    ("B", dict(K=8, cmap=ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7, vsv=0.25,
                                      capacity=0.80),
               tau_d=TAU_LP, pi_d=PI_LP, eta_d=ETA_LP, kc=KC, split="tau",
               vsv_stages=1, cap_profile="derived")),
    ("C", dict(K=4, cmap=ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0, vsv=-0.15,
                                      capacity=0.85),
               tau_d=TAU_HP, pi_d=PI_HP, eta_d=ETA_HP, kc=KC, split="dT",
               vsv_stages=2, cap_profile="uniform")),
    ("D", dict(K=1, cmap=ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7, vsv=0.0,
                                      capacity=0.80),
               tau_d=TAU_LP, pi_d=PI_LP, eta_d=ETA_LP, kc=KC, split="dT",
               vsv_stages=None, cap_profile="derived")),
    # E/F — the DESIGN SETTING on the UNIFORM profile, which is where § 5.10 (iv)'s degenerate
    # argmin lives. Nothing is moved, so at (m, n) = (1, 1) every row sits at phi_k = 1 and the
    # per-row margins collapse onto ONE value to the bit — except where the march's own
    # `th *= tau_k` accumulation has drifted from the ladder `theta_d` it divides by.
    ("E", dict(K=8, cmap=ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7, vsv=0.0,
                                      capacity=0.80),
               tau_d=TAU_LP, pi_d=PI_LP, eta_d=ETA_LP, kc=KC, split="dT",
               vsv_stages=None, cap_profile="uniform")),
    ("F", dict(K=4, cmap=ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7, vsv=0.0,
                                      capacity=0.80),
               tau_d=TAU_LP, pi_d=PI_LP, eta_d=ETA_LP, kc=KC, split="dT",
               vsv_stages=None, cap_profile="uniform")),
    # G — THE POWER-SPELLING DISCRIMINATOR, and it is here because the cell that was supposed to
    # do this job was nearly blind. `_ladder_T`'s "tau" arm is the ONE place in rungs 55/56 where
    # Python raises to a VARIABLE integer exponent (`r ** k`), so it is the file's only genuine
    # power-spelling choice — `pow(r, k)` against a running product against the "simplify the two
    # powers into one" `tau ** (k/K)`. Measured over 109 650 (tau, K, k) cells: the spellings
    # differ on 34.8 % and 65.5 % of them respectively. But at cell B's own (tau_lp, K = 8) only
    # rows 7-8 separate `pow` from the product, by ONE bit — and at (tau_lp, K = 4) NOTHING
    # separates them at all. K = 16 separates on 8 rows against the product and 14 against the
    # single power, so the rule is pinned rather than incidentally satisfied.
    ("G", dict(K=16, cmap=ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7, vsv=0.10,
                                       capacity=0.80),
               tau_d=TAU_LP, pi_d=PI_LP, eta_d=ETA_LP, kc=KC, split="tau",
               vsv_stages=3, cap_profile="derived")),
]

# Face points: one AT design, one throttled back, one deep enough that the low-n end of
# `solve_n`'s own bracket clamps (which is how `_T_FLOOR` gets witnessed here rather than only
# at the oracle).
FACES = [("design", 1.0, 1.0), ("part", 0.82, 0.90), ("deep", 0.55, 0.72)]

for tag, kw in CELLS:
    st = StageStack(**kw)
    K = st.K
    p = "cell%s/" % tag
    emit(p + "e_d", st.e_d)
    emit(p + "vsv_stages", st.vsv_stages)
    emit(p + "cmap_vsv", st.cmap.vsv)
    emit(p + "cmap_axial_vsv", st.cmap_axial.vsv)
    for k in range(K + 1):
        emit(p + "theta_d/%d" % k, st.theta_d[k])
        emit(p + "varpi_d/%d" % k, st.varpi_d[k])
    for k in range(K):
        emit(p + "psi_at/%d" % k, st.psi_at(k, 0.93))
        emit(p + "vsv_at/%d" % k, st.vsv_at(k))
        emit(p + "throat_ratio/%d" % k, st.stage_throat_ratio(k))
        emit(p + "capacity/%d" % k, st.stage_capacity(k))
        emit(p + "throat_loading/%d" % k, st.stage_throat_loading(k, 0.97))
        emit(p + "capacity_margin/%d" % k, st.stage_capacity_margin(k, 0.97))

    for fname, m, n in FACES:
        q = p + fname + "/"
        r = st.march(m, n, st.eta_d)
        emit(q + "tau", r["tau"])
        emit(q + "pi_internal", r["pi_internal"])
        emit(q + "e", r["e"])
        emit(q + "clamped", r["clamped"])
        for k in range(K):
            emit(q + "phi/%d" % k, r["phis"][k])
            emit(q + "n_k/%d" % k, r["n_ks"][k])
            emit(q + "tau_k/%d" % k, r["taus"][k])
        emit(q + "tau_of", st.tau_of(m, n, st.eta_d))
        emit(q + "lumped_tau", st.lumped_tau(m, n))
        # The face identity rung 56's currency rides on: m_k = phi_k * n_k.
        emit(q + "m_k_last", r["phis"][K - 1] * r["n_ks"][K - 1])
        # RUNG 56's per-row margin at the MARCHED flow — the reading § 5.10 (iv)'s argmin is
        # taken over. The index is emitted beside the values because when they agree to the bit
        # the index is the ONLY thing that discriminates, and a float dump cannot see it.
        mk = [r["phis"][k] * r["n_ks"][k] for k in range(K)]
        mg = [st.stage_capacity_margin(k, mk[k]) for k in range(K)]
        for k in range(K):
            emit(q + "row_margin/%d" % k, mg[k])
        emit(q + "row_argmin", min(range(K), key=lambda i: mg[i]))

    # THE TWO FLOORS. Measured, not guessed: it is the HIGH m/n end that clamps, not the low
    # one — `march(0.9, 0.1)` clamps NOTHING (n_k^2 is tiny, so `tau_k` stays near 1), while
    # `march(8, 2)` drives 7 of 8 stages to `_T_FLOOR`. And `_P_FLOOR` is dead for a DERIVED
    # reason, not a grid one: `tau_k` is floored BEFORE `base` is formed, so
    # `base >= 1 - e*(1 - _T_FLOOR)`, and `base < _P_FLOOR` needs
    # `e > (1 - _P_FLOOR)/(1 - _T_FLOOR) = 1.001` EXACTLY — a threshold in the two constants
    # alone, independent of the map, the split, K and the design point. At `e` just past it both
    # floors fire on the SAME stages and the shared counter doubles.
    lo = st.march(0.9, 0.1, st.eta_d)
    emit(p + "bracket_lo/tau", lo["tau"])
    emit(p + "bracket_lo/clamped", lo["clamped"])
    t_only = st.march(8.0, 2.0, st.eta_d)
    emit(p + "clamp_T/tau", t_only["tau"])
    emit(p + "clamp_T/clamped", t_only["clamped"])
    for k in range(K):
        emit(p + "clamp_T/tau_k/%d" % k, t_only["taus"][k])
    eta_hi = 0.99 * st.eta_d / 0.90        # scaled so e clears 1.001 on either spool's eta_d
    both = st.march(8.0, 2.0, eta_hi)
    emit(p + "clamp_TP/eta_live", eta_hi)
    emit(p + "clamp_TP/e", both["e"])
    emit(p + "clamp_TP/tau", both["tau"])
    emit(p + "clamp_TP/pi_internal", both["pi_internal"])
    emit(p + "clamp_TP/clamped", both["clamped"])

    # The speed-line inversion itself, at two pinned works. At K = 1 this DISPATCHES to
    # `ComponentMap.solve_n`, which is what makes the reduce bit-for-bit.
    for lbl, m, tau_c in (("at_design", 1.0, st.tau_d),
                          ("throttled", 0.86, 1.0 + 0.80 * (st.tau_d - 1.0))):
        emit(p + "solve_n/" + lbl, st.solve_n(m, tau_c, st.eta_d))

    # A stack at a MOVED efficiency, so `e = e_d*(eta_live/eta_d)` is not the identity.
    emit(p + "march_eta_lo/tau", st.march(0.95, 0.96, st.eta_d * 0.97)["tau"])
    emit(p + "march_eta_lo/e", st.march(0.95, 0.96, st.eta_d * 0.97)["e"])

sys.stdout.write("# slice N step 2 — StageStack smoke, PyPy\n")
sys.stdout.write("\n".join(OUT) + "\n")
