"""THE ORACLE, phase 3 slice C — every rung-13/15/16/18/21 mixing-PDF value the Rust must
reproduce.

The fifth in the family (`dump_gas.py` → `dump_cycle.py` → `dump_nox.py` → `dump_quench.py` →
here). A separate file again, and for the usual reason: slice B's committed TSVs stay frozen as
their own audit trail, and each dump's regeneration cost stays proportional to what it certifies.

WHAT IS ACTUALLY NEW HERE, and therefore what the sweep is built around:

  * `_beta_pdf_nodes_weights` — a REGIME-SWITCHING quadrature. For a lean mean the β-PDF shape
    parameter `a = ξ̄·(1/g − 1)` crosses 1 as `g` rises, and the code changes integration scheme
    at exactly that crossing: `a < 1` substitutes `u = ξ^a` (a `powp` with a computed exponent),
    `a ≥ 1` windows a uniform grid on `ξ̄ ± 8σ` (a `math.sqrt`). BOTH branches are dumped, and
    the boundary is dumped from both sides, because a port that gets one branch right and the
    other wrong looks correct across most of a J sweep.
  * `_transport_variance` — `nsteps` REPEATED DIVISIONS by `1 + C_φ·ω·dt`. Analytically this is
    `g_ceiling·exp(−C_φ·ω·τ)` and the docstring says so; numerically it is NOT, and the two are
    dumped side by side so a "simplification" to the closed form fails here rather than as a
    rung-18 basin that is subtly the wrong depth.
  * `_ideal_bell_ei` / `_bell_interpolator` — the bell every ideal-bell closure (13/15/18/21/22)
    integrates against. Its LEAN end is a BRANCH, not a value: below the flammability limit
    `_primary_aft` pins against its cold bracket edge and Python catches the AssertionError to
    return 0. The index of the first burnable node is dumped as its own key, so the Rust's
    fallible-AFT arm is measured against the Python's `try/except` rather than assumed equal.
  * `_pocket_quench_mean_ei` — the expensive one: `n_bell` pockets, each rich-of-mean pocket a
    full `_quench_no` that builds its OWN mix-out trajectory (no `tab` sharing is possible —
    every pocket sits at its own `far_local` with its own `alpha`).

THE SIZING LEVERS, and there are two — a different pair from slice B's.

  1. THE BELL IS `g`-INDEPENDENT AND `J`-INDEPENDENT. `_bell_interpolator` depends only on
     (p, Tt3, hf, τ, super_eq_o), so FOUR bells (2 design points × 2 super-eq arms) carry every
     g sweep, every J sweep and both mean positions in sections 4–7. Production's `_pdf_mean_ei`
     rebuilds the bell per call, so the wide sweeps run the hoisted-bell helper — which is the
     SAME arithmetic (`sum(wᵢ·bell(ξᵢ))`) and is what the Python's own tests use; § 4 then pins
     a handful of direct `_pdf_mean_ei` calls against it so the hoist is measured, not assumed.
  2. RUNG 16's POCKET GRID IS `g`-INDEPENDENT. In `_pocket_quench_mean_ei` the `vals` array
     depends on `tau_core` only (through `tau_pocket`); `g_seg` enters nowhere before the final
     β-quadrature. So the Rust splits the function in two and a `g` sweep at fixed `τ_core` is
     free. The dump cannot exploit that against Python's monolithic version, so § 8 sweeps
     `τ_core` widely and `g` narrowly — and dumps four SINGLE pockets by hand, which localises a
     defect to a pocket instead of to the integral.

SHAPE KEYS. Slices A and B both measured that the two interpreters can disagree on an extremum's
VALUE while agreeing on its LOCATION exactly, and slice B measured a location key REFUTING the
claim it was dumped to confirm. Four locations are dumped here:
  * rung 13's emissions minimum over a J sweep, at FOUR spacings — deliberately wider than the
    Python's own gate, which samples two, because a gate that samples where a claim is true
    cannot discover where it stops being true (slice B, rung 12).
  * rung 13's HUMP in `⟨EI⟩(g)`. This one needs care: the hump peak sits at g ≈ 0.02 and the
    quadrature's branch boundary at this design point is g = ξ̄/(1+ξ̄) ≈ 0.0258 — right on top of
    it. Measured across the boundary the curve is not even locally monotone (g=0.026 reads ABOVE
    g=0.025 by 0.03 %, the two schemes disagreeing). So the argmax grid is COARSE and its peak
    cell clears both neighbours by ~19 %, which no branch artefact of that size can move; the
    boundary itself is dumped separately, as VALUES, in § 1d.
  * rung 21's shape-preservation claim: the argmin with the super-eq-O lift ON must be the SAME
    INDEX as with it OFF. A location key pre-registered to CONFIRM.
  * rung 18's transported-width basin, and the argmin of its mean-field twin (which must sit at
    an END — the rung's whole NEGATIVE result).
NOT dumped: a global argmin for RUNG 16. Its own spec (`gas.py` `PocketQuenchPDF`, "NOT
CLAIMED") declines that location — it flips across the quadrature, the φ>2 tail and the `C_e`
regime — so a key on it would fail for a reason that is not a defect. Rung 16 gets the structure
its spec DOES certify: the excess vanishing at `C_opt`, both flanks up, and the sublinearity
RATIO (two values from the same run, the reduce-spine shape).

THE `g` SWEEP IS CAPPED AT 0.40, and the cap is measured rather than aesthetic.
`_beta_pdf_nodes_weights` asserts `b ≥ 1`, i.e. `g ≤ (1−ξ̄)/(2−ξ̄)`: 0.4933 at dp1's lean mean,
0.4835 at the stoichiometric mean, 0.489 at dp4's. 0.40 clears all three. Sweeping past it is
not a port test, it is the Python's own guard firing in both languages.

THE `n_quad` FLOOR IS ALSO MEASURED, and finding it cost this dump its first run. The closure's
OTHER standing guard — mean preservation to 1 % — is `n_quad`-SENSITIVE, and the first draft of
this file swept at `n_quad = 64` and crashed inside the Python. Characterised: at a LEAN mean the
guard REJECTS `g = 0.026` (the first point past the `a = 1` branch switch) and `g = 0.40` (the top
of the range) for every `n_quad ≤ 100`, and accepts both from 112 up — 8.2e-3 and 9.4e-3 at 112,
falling to 4.0e-3 and 5.7e-3 at 160. The Python's own gate samples `n_quad = 160` and `g ≤ 0.30`,
so it sits inside the accepted region and nothing there could see this. **That is not a port
defect and it is not worked around**: the sweep runs at `n_quad = 160` (the Python tests' own
value), the convergence ladder is dumped as its own keys, and `pdf_oracle.rs` asserts the guard
FIRES at `n_quad = 64` — the Rust reproduces the rejection, not just the acceptance.

Single-use by design (docs/plans/todo-rust-port.md): it validates the Rust and is deleted at
phase 8. It reaches into `turbojet.gas`'s private names on purpose — it is not an API consumer,
it is a reference dump.

Output is TSV, one row per value:  key <TAB> u64-bits <TAB> repr

Run under BOTH interpreters — whatever PyPy and CPython disagree by is a deviation the project
ALREADY tolerates, and that gap is the principled tolerance floor rather than an invented one.

    C:\\Python314\\python.exe rust/oracle/dump_pdf.py rust/oracle/pdf_cpython.tsv
    .venv\\Scripts\\python.exe  rust/oracle/dump_pdf.py rust/oracle/pdf_pypy.tsv
"""
import math
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import gas as G
from turbojet.engine import FlightCondition, build_turbojet

ROWS = []
T0 = time.time()


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def argmin(vals):
    return min(range(len(vals)), key=lambda i: vals[i])


def argmax(vals):
    return max(range(len(vals)), key=lambda i: vals[i])


# --- resolutions ----------------------------------------------------------------------------
TAU = 3e-3
HF = G._HF_FUEL_DEFAULT
# NB: the ideal-bell reference grid. 32 is coarse for the physics (production uses 120-200) but
# this dump certifies ARITHMETIC, not shape convergence, and every node is a `_primary_aft`
# bisection plus a 4000-step RK4. It is also coarse enough that the lean unburnable run is
# several nodes long, so the first-burnable-index key has something to say.
NB = 32
# NQ: the β-PDF quadrature. 160 is the Python rung-13/21 tests' own value, and it is a FLOOR
# here, not a taste — see the `n_quad` note in the module docstring. It costs nothing: the
# quadrature touches no solver, and the bell it integrates against is an interpolator.
NQ = 160
# Rung 16's per-pocket integral. Each rich-of-mean pocket is a full `_quench_no` over its own
# 9-point trajectory; 24 pockets cost ~1.2 s on PyPy and ~12 s on CPython per τ_core.
NB16, NQ16, NG16, NS16 = 24, 64, 9, 100
# The finite-quench resolution for the end-to-end `zoned_nox` calls in § 9.
NGRID, NSTEPS = 17, 200
PHI_P = 1.5      # the RQL rich primary every mixing rung anchors on
CE = 0.20        # the ANCHORED jet-entrainment regime (rungs 11-16)


def mix(J, **kw):
    return G.JetMixing(J=J, C_e=CE, shape_n=2.0, **kw)


def j_opt(S, C_opt=2.5, H=None):
    """J where C=(S/H)√J = C_opt — the uniformity optimum every kinked closure pins to."""
    H = G.JetMixing(J=1.0).H if H is None else H
    return (C_opt * H / S) ** 2


# =============================================================================================
# SECTION 1 — the SOLVER-FREE algebra. No compositions, no bells, no integrators.
# =============================================================================================
# This section exists to answer every spelling and ordering question IN ISOLATION, in
# milliseconds, instead of letting a defect surface as "⟨EI⟩ differs" six sections later. It is
# slice B's section 1, aimed at this slice's new arithmetic.
#
# The traps it is built to catch, one per group:
#   * `uu ** (1.0/a)` — a COMPUTED float exponent. PyPy does not rewrite it; the Rust must reach
#     libm `pow` (`powp`), not `x.powf` folded to something else and not a product chain.
#   * `(x - xibar) ** 2` — an INTEGER LITERAL exponent, which PyPy DOES rewrite into a multiply.
#     The two halves of the rule point opposite ways in the SAME function.
#   * `math.sqrt(...)` in the `a ≥ 1` window and in every config's `C()` — the sqrt instruction,
#     NOT `** 0.5`. Slice B's finding 2; applying "always powp" mechanically gets these backwards.
#   * `xi_max * (i + 0.5) / n_bell` — `(xi_max·(i+0.5))/n_bell`, not `xi_max·((i+0.5)/n_bell)`.
#     Three grid formulas in this slice, each a different shape; § 1e pins all three.
#   * `_transport_variance`'s loop of divisions vs its own analytic closed form.

# --- 1a: the four configs' derived quantities ------------------------------------------------
# The C grid straddles C_opt on BOTH flanks (the |ln| kink is not differentiable there) and
# reaches far enough out that the `min(g_max, ·)` cap binds — both are branch selectors.
C_FACTORS = [0.01, 0.25, 0.5, 0.8, 1.0, 1.25, 2.0, 4.0, 100.0]
J_GRID = [1.0, 4.0, 9.0, 16.0, 25.0, 64.0, 100.0, 225.0, 625.0]
S_GRID = [0.0800, 0.0625, 0.0500, 0.0400]

for S in S_GRID:
    pdf = G.MixingPDF(S=S)
    qp = G.QuenchPDF(S=S)
    pq = G.PocketQuenchPDF(S=S)
    tr = G.TransportedPDF(S=S)
    for j in J_GRID:
        m = mix(j)
        # `C` is the same formula on all four configs; dumping all four is not redundant — it is
        # what catches a transposed S/H in ONE of them (they are four separate Python methods).
        put(f"cfg/{S!r}/{j!r}/C_pdf", pdf.C(m))
        put(f"cfg/{S!r}/{j!r}/C_qp", qp.C(m))
        put(f"cfg/{S!r}/{j!r}/C_pq", pq.C(m))
        put(f"cfg/{S!r}/{j!r}/C_tr", tr.C(m))
    for f in C_FACTORS:
        C = 2.5 * f
        put(f"cfg/{S!r}/C{f!r}/g_pdf", pdf.segregation(C))
        put(f"cfg/{S!r}/C{f!r}/u_qp", qp._u(C))
        put(f"cfg/{S!r}/C{f!r}/g_qp", qp.segregation(C))
        put(f"cfg/{S!r}/C{f!r}/D_qp", qp.dwell_factor(C, TAU))
        put(f"cfg/{S!r}/C{f!r}/g_pq", pq.segregation(C))
        put(f"cfg/{S!r}/C{f!r}/tcore_pq", pq.core_dwell(C))
        put(f"cfg/{S!r}/C{f!r}/omega_tr", tr.coverage_omega(C))

# The knobs that only appear in ONE place each — a transposed factor shows up here and nowhere
# else. `k_g=0` and a `g_max` that binds early are branch selectors, like slice B's `k_u=0`.
for kg, gmax in ((0.0, 0.3), (0.3, 0.05), (0.9, 0.3)):
    pdf = G.MixingPDF(k_g=kg, g_max=gmax)
    for f in (0.25, 1.0, 4.0):
        put(f"cfg/kg{kg!r}gm{gmax!r}/{f!r}/g", pdf.segregation(2.5 * f))
for tres, bu in ((1.0e-3, 3.0), (2.5e-3, 0.0), (5.0e-3, 6.0)):
    qp, pq = G.QuenchPDF(tau_res=tres, b_u=bu), G.PocketQuenchPDF(tau_res=tres, b_u=bu)
    for f in (0.25, 1.0, 4.0):
        put(f"cfg/tr{tres!r}bu{bu!r}/{f!r}/D", qp.dwell_factor(2.5 * f, TAU))
        put(f"cfg/tr{tres!r}bu{bu!r}/{f!r}/tcore", pq.core_dwell(2.5 * f))
for da, wc, cphi, tmix in ((2.0, 1.0, 2.0, 2.5e-3), (0.5, 0.5, 2.0, 2.5e-3),
                           (6.0, 2.0, 3.0, 1.0e-3)):
    tr = G.TransportedPDF(Da_opt=da, w_cov=wc, C_phi=cphi, tau_mix=tmix)
    for f in (0.25, 1.0, 4.0):
        put(f"cfg/da{da!r}w{wc!r}c{cphi!r}t{tmix!r}/{f!r}/omega", tr.coverage_omega(2.5 * f))

# --- 1b: the DERIVED two-stream ceiling ------------------------------------------------------
# Rung 18's one derived quantity. Pure composition algebra — no J, no C_e, no solver.
CEIL_FAR = [0.0150, 0.0271791907192821, 0.0350, 0.0450]
for far in CEIL_FAR:
    for phi_p in (0.8, 1.0, 1.2, 1.5, 1.8, 2.0):
        if phi_p * G._F_STOICH <= far:
            continue                       # the RQL guard: the primary must be RICHER than the mean
        put(f"ceil/{far!r}/{phi_p!r}", G._two_stream_ceiling(far, phi_p))

# --- 1c: the variance-decay ODE, and its own analytic twin -----------------------------------
# `nsteps` repeated divisions. The `an` key is the CLOSED FORM the docstring names; the two must
# NOT agree bit-for-bit, and dumping both is what makes a "simplification" to `exp` fail loudly.
for gc in (0.0675, 0.0200, 0.3000):
    for om in (0.0, 50.0, 250.0, 1000.0):
        for tau in (1.0e-3, 2.5e-3):
            for nst in (50, 200, 400):
                put(f"ode/{gc!r}/{om!r}/{tau!r}/{nst}",
                    G._transport_variance(gc, om, tau, c_phi=2.0, nsteps=nst))
            put(f"ode/{gc!r}/{om!r}/{tau!r}/analytic", gc * math.exp(-2.0 * om * tau))
for cphi in (1.0, 2.0, 3.5):
    put(f"ode/cphi{cphi!r}", G._transport_variance(0.0675, 250.0, 2.5e-3, c_phi=cphi, nsteps=400))

# --- 1d: the β-PDF quadrature, BOTH branches, and the boundary from both sides ---------------
# `a = ξ̄(1/g − 1)` crosses 1 at g = ξ̄/(1+ξ̄). At dp1's lean mean that is g ≈ 0.02578, so the
# grid below straddles it deliberately: 0.0257 is the LAST `a ≥ 1` (windowed) point and 0.026 the
# first `a < 1` (u-substituted) one. The two schemes are different functions; the port must
# switch at the same place, and `nodes`/`weights`/mean/variance are dumped so a wrong switch
# shows up as a node mismatch rather than as a 0.03 % ⟨EI⟩ drift.
XIBAR_LEAN = 0.0271791907192821 / (1.0 + 0.0271791907192821)
XIBAR_ST = G._F_STOICH / (1.0 + G._F_STOICH)
G_GRID = [0.0005, 0.004, 0.01, 0.02, 0.0257, 0.026, 0.05, 0.12, 0.24, 0.40]


def dump_quad(tag, xibar, g_seg, nq):
    nodes, w = G._beta_pdf_nodes_weights(xibar, g_seg, n_quad=nq)
    inv = 1.0 / g_seg - 1.0
    put(f"quad/{tag}/a", xibar * inv)
    put(f"quad/{tag}/b", (1.0 - xibar) * inv)
    for i in sorted({0, 1, nq // 4, nq // 2, (3 * nq) // 4, nq - 2, nq - 1}):
        put(f"quad/{tag}/n{i}", nodes[i])
        put(f"quad/{tag}/w{i}", w[i])
    put(f"quad/{tag}/mean", sum(wi * x for wi, x in zip(w, nodes)))
    put(f"quad/{tag}/var", sum(wi * (x - xibar) ** 2 for wi, x in zip(w, nodes)))
    put(f"quad/{tag}/vartgt", g_seg * xibar * (1.0 - xibar))
    return nodes, w


for gs in G_GRID:
    dump_quad(f"lean/{gs!r}", XIBAR_LEAN, gs, NQ)
    dump_quad(f"stoich/{gs!r}", XIBAR_ST, gs, NQ)
# n_quad is an accumulation-depth knob: the weight normalisation sums `n_quad` exponentials.
for nq in (40, 64, 160, 200):
    dump_quad(f"nq{nq}/sing", XIBAR_LEAN, 0.12, nq)
    dump_quad(f"nq{nq}/delta", XIBAR_LEAN, 0.004, nq)
# THE MEASURED n_quad FLOOR (see the docstring). These two `g` are where the closure's own
# mean-preservation guard is marginal at a lean mean; below n_quad=112 it REJECTS them, and the
# Rust must reject them too. Dumping the ladder makes the convergence a measured quantity
# instead of a remembered one, and gives the `should_panic` side of the gate a boundary to sit
# against rather than a guess.
for nq in (112, 128, 160, 200):
    for gs in (0.026, 0.40):
        dump_quad(f"floor/{gs!r}/nq{nq}", XIBAR_LEAN, gs, nq)
# The `a ≥ 1` branch also carries a `min`/`max` window clamp (`lo` floored at 1e-12, `hi` capped
# at 1−1e-12). A near-delta g drives σ→0 and neither clamp binds; a MEAN near 1 would make `hi`
# bind, which is unreachable from a lean mean — so the clamp is exercised through a synthetic
# high mean, once, rather than left as untested code.
dump_quad("clamp/hi", 0.97, 0.0005, NQ)

print(f"  section 1 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# =============================================================================================
# SECTION 2 — the design points, derived from REAL engine runs, never hardcoded
# =============================================================================================
FLIGHT_SUB = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
FLIGHT_SUP = FlightCondition(T0=216.7, p0=18_750.0, M0=2.0)
LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)

DESIGN = {}
for name, flight, pi_c, Tt4, mdot in (("dp1", FLIGHT_SUB, 10.0, 1500.0, 50.0),
                                      ("dp4", FLIGHT_SUP, 12.0, 1800.0, 50.0)):
    r = build_turbojet(G.Gas.reacting_equilibrium(), pi_c, Tt4, flight.p0,
                       **LOSSES).run(flight, mdot)
    st3, st4 = r.stations["3"], r.stations["4"]
    DESIGN[name] = (st3.Tt, st4.Tt, st4.far, st4.pt)
    put(f"dp/{name}/Tt3", st3.Tt)
    put(f"dp/{name}/Tt4", st4.Tt)
    put(f"dp/{name}/far", st4.far)
    put(f"dp/{name}/pt4", st4.pt)
    put(f"dp/{name}/xibar", st4.far / (1.0 + st4.far))
    # the b≥1 sweep ceiling this design point imposes — the measured cap, dumped so the Rust
    # gate can assert the sweep stays under it rather than trusting the comment.
    xb = st4.far / (1.0 + st4.far)
    put(f"dp/{name}/g_bmax", (1.0 - xb) / (2.0 - xb))

GAS = G.Gas.reacting_equilibrium()

# =============================================================================================
# SECTION 3 — the IDEAL BELL (rung 13's substrate; rung 21's lift)
# =============================================================================================
# `_ideal_bell_ei` at a LOCAL far, and the interpolator built on it. Both super-eq-O arms: rung
# 21's whole content is that `super_eq_o=True` lifts every node, and `False` is bit-for-bit
# rungs 13/15/18 — so the two arms must differ everywhere the bell is nonzero and the False arm
# must reproduce the pre-rung-21 numbers exactly.
XI_MAX = (2.0 * G._F_STOICH) / (1.0 + 2.0 * G._F_STOICH)
put("bell/xi_max", XI_MAX)

BELLS = {}


def bell_of(dp, super_eq):
    key = (dp, super_eq)
    if key not in BELLS:
        Tt3, _Tt4, _far, p = DESIGN[dp]
        BELLS[key] = G._bell_interpolator(p, Tt3, HF, TAU, n_bell=NB, super_eq_o=super_eq)
        print(f"  bell {dp}/su={super_eq} built at {time.time() - T0:.1f}s")
    return BELLS[key]


for dp in ("dp1", "dp4"):
    Tt3, _Tt4, far, p = DESIGN[dp]
    for super_eq in (False, True):
        tag = f"{dp}/{'su' if super_eq else 'eq'}"
        b = bell_of(dp, super_eq)
        # The reference grid, read THROUGH the interpolator: `bell(xi_ref[i])` lands on node i
        # with lerp weight 0, so these ARE `ei_ref` — and they exercise the binary search too.
        xi_ref = [XI_MAX * (i + 0.5) / NB for i in range(NB)]
        vals = []
        for i, x in enumerate(xi_ref):
            v = b(x)
            vals.append(v)
            put(f"bell/{tag}/node{i}/xi", x)
            put(f"bell/{tag}/node{i}/ei", v)
        # SHAPE KEY — the lean end of the bell is a BRANCH, not a value: below the flammability
        # limit `_primary_aft` pins at its cold bracket edge and Python catches the AssertionError
        # to return 0. This index is where that branch stops being taken, and it is the ONLY
        # thing that measures the Rust's fallible-AFT arm against Python's `try/except`.
        first_burn = next((i for i, v in enumerate(vals) if v > 0.0), len(vals))
        put(f"bell/{tag}/first_burnable", float(first_burn))
        # SHAPE KEY — rung 9's bell PEAKS near stoichiometric, and slice A measured that the two
        # interpreters agree on the LOCATION while disagreeing on the value.
        put(f"bell/{tag}/argmax", float(argmax(vals)))
        # the interpolator's three branches: below the first node, above the last, and the lerp.
        put(f"bell/{tag}/lo_edge", b(0.0))
        put(f"bell/{tag}/lo_edge2", b(xi_ref[0] * 0.5))
        put(f"bell/{tag}/hi_edge", b(XI_MAX))
        put(f"bell/{tag}/hi_edge2", b(0.5))
        for t in (0.25, 0.5, 0.75):
            for i in (3, NB // 2, NB - 2):
                x = xi_ref[i] + t * (xi_ref[i + 1] - xi_ref[i])
                put(f"bell/{tag}/lerp{i}_{t!r}", b(x))
        # `_ideal_bell_ei` DIRECTLY, including the two zero branches the grid never reaches:
        # φ > 2 (the soot bound) and far ≤ 0.
        for fl in (0.0, 0.002, 0.010, far, 0.0500, G._F_STOICH, 0.0900, 0.1359, 0.1400, 0.2000):
            put(f"bellpt/{tag}/{fl!r}",
                G._ideal_bell_ei(fl, p, Tt3, HF, TAU, super_eq_o=super_eq))
        # τ is the bell's reference residence and enters the Zeldovich integrator linearly-ish;
        # a transposed τ would be invisible at the single production value.
        for t_res in (1.0e-3, 5.0e-3):
            put(f"bellpt/{tag}/tau{t_res!r}",
                G._ideal_bell_ei(G._F_STOICH, p, Tt3, HF, t_res, super_eq_o=super_eq))

print(f"  section 3 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# =============================================================================================
# SECTION 4 — ⟨EI⟩ over the β-PDF (rung 13's closure; rungs 18/21/22 reuse it verbatim)
# =============================================================================================


def pdf_ei(b, xibar, g_seg, nq=NQ):
    """Production's `_pdf_mean_ei` with the bell HOISTED — the identical arithmetic (the Python's
    own rung-13/21 tests use exactly this helper, and `test_zoned_nox_matches_pdf_helper` pins it
    to production). Hoisting is sizing lever 1: it turns 4 bell builds into 4, not into 200."""
    if g_seg <= 1e-9:
        return b(xibar)
    nodes, w = G._beta_pdf_nodes_weights(xibar, g_seg, n_quad=nq)
    return sum(wi * b(x) for wi, x in zip(w, nodes))


for dp in ("dp1", "dp4"):
    Tt3, _Tt4, far, p = DESIGN[dp]
    xibar = far / (1.0 + far)
    for super_eq in (False, True):
        tag = f"{dp}/{'su' if super_eq else 'eq'}"
        b = bell_of(dp, super_eq)
        for gs in G_GRID + [0.0]:
            put(f"pdfei/{tag}/lean/{gs!r}", pdf_ei(b, xibar, gs))
            put(f"pdfei/{tag}/stoich/{gs!r}", pdf_ei(b, XIBAR_ST, gs))
        # the delta short-circuit is a BRANCH on `g_seg <= 1e-9`, so both sides of it are dumped.
        for gs in (0.0, 1e-12, 1e-9, 1.0000001e-9, 1e-8):
            put(f"pdfei/{tag}/delta/{gs!r}", pdf_ei(b, xibar, gs))
        # n_quad as an accumulation-depth knob on the integral itself.
        for nq in (40, 160):
            put(f"pdfei/{tag}/nq{nq}", pdf_ei(b, xibar, 0.12, nq=nq))

# SHAPE KEY — the HUMP. ⟨EI⟩(g) peaks at moderate g and DESCENDS as the β-PDF goes bimodal, which
# is WHY rung 13's far-over-penetration flank falls. The grid is coarse ON PURPOSE: measured, the
# peak cell clears both neighbours by ~19 %, while the quadrature's branch boundary (g ≈ 0.0258
# here) perturbs neighbouring values by ~0.03 %. A finer grid would put the argmax inside that
# perturbation and turn a real detector into a flaky one.
HUMP_G = [0.005, 0.01, 0.02, 0.05, 0.12, 0.30]
for dp in ("dp1", "dp4"):
    _Tt3, _Tt4, far, _p = DESIGN[dp]
    xibar = far / (1.0 + far)
    for super_eq in (False, True):
        tag = f"{dp}/{'su' if super_eq else 'eq'}"
        b = bell_of(dp, super_eq)
        hv = [pdf_ei(b, xibar, gs) for gs in HUMP_G]
        for gs, v in zip(HUMP_G, hv):
            put(f"hump/{tag}/{gs!r}", v)
        put(f"hump/{tag}/argmax", float(argmax(hv)))
        # the MARGIN the argmax rests on — dumped so the gate can assert the detector is not
        # sitting on a coin-flip, rather than assuming it (slice B's coarse-grid rationale, made
        # into a number).
        im = argmax(hv)
        put(f"hump/{tag}/margin_lo", hv[im] / hv[im - 1])
        put(f"hump/{tag}/margin_hi", hv[im] / hv[im + 1])

# Production's `_pdf_mean_ei` itself, a handful of times, to PIN the hoisted helper to the
# wrapper. If these disagree with the `pdfei/` rows above, the hoist is not the same arithmetic
# and every sweep in this section is measuring the wrong thing.
Tt3_1, Tt4_1, FAR_1, P_1 = DESIGN["dp1"]
XIB_1 = FAR_1 / (1.0 + FAR_1)
for super_eq in (False, True):
    for gs in (0.0, 0.02, 0.12, 0.30):
        put(f"pdfmean/dp1/{'su' if super_eq else 'eq'}/{gs!r}",
            G._pdf_mean_ei(FAR_1, Tt3_1, P_1, HF, TAU, gs, n_bell=NB, n_quad=NQ,
                           super_eq_o=super_eq))

print(f"  section 4 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# =============================================================================================
# SECTION 5 — rung 13's optimum LOCATION, and rung 21's shape-preservation key
# =============================================================================================
# The emissions minimum is claimed to pin AT the Holdeman group C_opt for every spacing S, so
# J_min = J_opt shifts EXACTLY as (H/S)². Dumped at FOUR spacings — the Python's own gate samples
# two, and slice B's rung-12 lesson is that a gate sampling only where a claim holds cannot find
# where it stops holding.
#
# RUNG 21's KEY: the same argmin with the super-eq-O lift ON. The rung's claim is that the lift is
# SHAPE-PRESERVING — location, shift and sign all unmoved — so `argmin_su == argmin_eq` is a
# location key pre-registered to CONFIRM. If it ever refutes, that is the finding.
for dp in ("dp1", "dp4"):
    _Tt3, _Tt4, far, _p = DESIGN[dp]
    xibar = far / (1.0 + far)
    for S in S_GRID:
        pdf = G.MixingPDF(S=S)
        Jo = j_opt(S, pdf.C_opt)
        Js = [Jo / 4.0, Jo / 2.0, Jo, 2.0 * Jo, 4.0 * Jo]
        put(f"jsweep/{dp}/{S!r}/J_opt", Jo)
        loc = {}
        for super_eq in (False, True):
            arm = "su" if super_eq else "eq"
            b = bell_of(dp, super_eq)
            eis = []
            for J in Js:
                gs = pdf.segregation(pdf.C(mix(J)))
                v = pdf_ei(b, xibar, gs)
                eis.append(v)
                put(f"jsweep/{dp}/{S!r}/{arm}/{J!r}/g", gs)
                put(f"jsweep/{dp}/{S!r}/{arm}/{J!r}/ei", v)
            loc[arm] = argmin(eis)
            put(f"jsweep/{dp}/{S!r}/{arm}/argmin", float(loc[arm]))
            # the RATIO the pin rests on: one step either side of the optimum, same run.
            im = loc[arm]
            put(f"jsweep/{dp}/{S!r}/{arm}/lift_lo", eis[im - 1] / max(eis[im], 1e-300))
            put(f"jsweep/{dp}/{S!r}/{arm}/lift_hi", eis[im + 1] / max(eis[im], 1e-300))
        put(f"jsweep/{dp}/{S!r}/loc_agree", float(loc["eq"] == loc["su"]))

# Rung 21's MAGNITUDE claim, as a ratio of two values from the same pair of runs: the ideal-bell
# lift is peak-concentrated and BELOW the primary's, because the bell integral is weighted onto
# the near-stoich peak where m(T) is at its minimum.
for gs in (0.0, 0.02, 0.12, 0.30):
    put(f"r21/lift/dp1/{gs!r}",
        pdf_ei(bell_of("dp1", True), XIB_1, gs) / pdf_ei(bell_of("dp1", False), XIB_1, gs))
put("r21/lift/primary_m", G._super_eq_o_multiplier(G._primary_aft(PHI_P * G._F_STOICH, P_1,
                                                                 Tt3_1, HF)))
put("r21/lift/peak_m", G._super_eq_o_multiplier(G._primary_aft(G._F_STOICH, P_1, Tt3_1, HF)))

print(f"  section 5 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# =============================================================================================
# SECTION 6 — rung 18: the transported width, its basin, and the NEGATIVE result
# =============================================================================================
TR = G.TransportedPDF(S=0.0625, n_bell=NB, n_quad=NQ, n_ode=200)
for dp in ("dp1", "dp4"):
    _Tt3, _Tt4, far, _p = DESIGN[dp]
    xibar = far / (1.0 + far)
    b = bell_of(dp, False)
    gc = G._two_stream_ceiling(far, PHI_P)
    put(f"tr/{dp}/g_ceiling", gc)
    gs_list = []
    for J in J_GRID:
        C = TR.C(mix(J))
        g_seg, g_ceil = TR.segregation(C, far, PHI_P)
        gs_list.append(g_seg)
        put(f"tr/{dp}/{J!r}/C", C)
        put(f"tr/{dp}/{J!r}/omega", TR.coverage_omega(C))
        put(f"tr/{dp}/{J!r}/g", g_seg)
        put(f"tr/{dp}/{J!r}/gceil", g_ceil)
        put(f"tr/{dp}/{J!r}/ei", pdf_ei(b, xibar, g_seg))
    # SHAPE KEY — the transported basin's minimum, an INTERIOR location, from the IMPOSED
    # spatial coverage ω(C). J_GRID contains J_opt=16 exactly.
    put(f"tr/{dp}/argmin_g", float(argmin(gs_list)))
    eis = [pdf_ei(b, xibar, g) for g in gs_list]
    put(f"tr/{dp}/argmin_ei", float(argmin(eis)))
    # the RESIDUAL FLOOR: g(C_opt) = g_ceiling·exp(−Da_opt) > 0, so the optimum is ELEVATED off
    # the well-mixed point value rather than touching it (the kink's ≈0).
    g_opt, _ = TR.segregation(TR.C(mix(16.0)), far, PHI_P)
    put(f"tr/{dp}/floor_g", g_opt)
    put(f"tr/{dp}/floor_ratio", g_opt / (gc * math.exp(-TR.Da_opt)))
    put(f"tr/{dp}/elevation", pdf_ei(b, xibar, g_opt) / max(pdf_ei(b, xibar, 0.0), 1e-300))

# THE NEGATIVE RESULT, as the Python's gate states it: with any MEAN-FIELD ω(J) the residual g(J)
# is monotone-or-flat — no interior optimum. Only ω(C=(S/H)√J), i.e. once the SPATIAL spacing is
# injected, produces one. Both arms are dumped, with their argmins, so the port reproduces the
# CONTRAST and not just one side of it.
MF_J = [4.0, 9.0, 16.0, 25.0, 49.0, 100.0, 225.0, 625.0]
MF_FORMS = (("const", lambda J: 250.0),
            ("sqrtJ", lambda J: 250.0 * math.sqrt(J / 16.0)),
            ("linJ", lambda J: 250.0 * (J / 16.0)))
for name, om in MF_FORMS:
    vals = [G._transport_variance(0.0675, om(J), 2.5e-3, c_phi=2.0, nsteps=400) for J in MF_J]
    for J, v in zip(MF_J, vals):
        put(f"mf/{name}/{J!r}", v)
    put(f"mf/{name}/argmin", float(argmin(vals)))
    put(f"mf/{name}/spread", (max(vals) - min(vals)) / max(vals))
sp = []
for J in MF_J:
    C = (TR.S / G.JetMixing(J=1.0).H) * math.sqrt(J)
    v = G._transport_variance(0.0675, TR.coverage_omega(C), TR.tau_mix, c_phi=TR.C_phi, nsteps=400)
    sp.append(v)
    put(f"mf/spatial/{J!r}", v)
put("mf/spatial/argmin", float(argmin(sp)))

# SMOOTHNESS — the transported basin's one-sided slopes VANISH at C_opt; the imposed kink's do
# not (equal-and-opposite ±k_g/C_opt). A finite difference at eps=1e-5 is a DIFFERENCE of two
# nearly-equal accumulated values, so it is the most drift-sensitive quantity in this dump; that
# is exactly why it is here (slice-5 lesson: a finite difference inherits the drift of what it
# differences, so it needs an ABSOLUTE bar, not a relative one).
EPS = 1e-5
C0 = TR.C_opt


def g_tr(C):
    return G._transport_variance(0.0675, TR.coverage_omega(C), TR.tau_mix,
                                 c_phi=TR.C_phi, nsteps=400)


put("smooth/tr/slope_r", (g_tr(C0 * (1 + EPS)) - g_tr(C0)) / (EPS * C0))
put("smooth/tr/slope_l", (g_tr(C0) - g_tr(C0 * (1 - EPS))) / (EPS * C0))
KINK = G.MixingPDF(S=TR.S, C_opt=C0)
put("smooth/kink/slope_r",
    (KINK.segregation(C0 * (1 + EPS)) - KINK.segregation(C0)) / (EPS * C0))
put("smooth/kink/slope_l",
    (KINK.segregation(C0) - KINK.segregation(C0 * (1 - EPS))) / (EPS * C0))

print(f"  section 6 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# =============================================================================================
# SECTION 7 — rung 15: the dwell factor TIMES the bell integral
# =============================================================================================
# Rung 15's term 2 is `D(u)·⟨EI_bell⟩(g)` — a scalar rescale of the SAME rung-13 integral, which
# is why it needs no new solver and gets a short section. What it does need is the PRODUCT in the
# right order and the two kinks (g and D) reading the same u.
QP = G.QuenchPDF(S=0.0625, n_bell=NB, n_quad=NQ)
for dp in ("dp1", "dp4"):
    _Tt3, _Tt4, far, _p = DESIGN[dp]
    xibar = far / (1.0 + far)
    for super_eq in (False, True):
        arm = "su" if super_eq else "eq"
        b = bell_of(dp, super_eq)
        t2 = []
        for J in J_GRID:
            C = QP.C(mix(J))
            g_seg = QP.segregation(C)
            d = QP.dwell_factor(C, TAU)
            v = d * pdf_ei(b, xibar, g_seg)
            t2.append(v)
            put(f"r15/{dp}/{arm}/{J!r}/D", d)
            put(f"r15/{dp}/{arm}/{J!r}/g", g_seg)
            put(f"r15/{dp}/{arm}/{J!r}/term2", v)
        put(f"r15/{dp}/{arm}/argmin_t2", float(argmin(t2)))

print(f"  section 7 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# =============================================================================================
# SECTION 8 — rung 16: the PER-POCKET quench integral (the expensive object)
# =============================================================================================
# Sizing lever 2 in reverse: `vals` inside `_pocket_quench_mean_ei` depends on τ_core and NOT on
# g, so the τ_core sweep is what costs and the g sweep is what is nearly free — except that the
# Python rebuilds the grid on every call, so here BOTH cost. The sweep is therefore τ_core-wide
# and g-narrow, and the Rust (which splits the function) gets the cheap version.
#
# NO GLOBAL ARGMIN KEY. Rung 16's own docstring declines that location — it flips across the
# quadrature, the φ>2 tail and the C_e regime, all comparable to the margin. What IS dumped is
# the structure the rung certifies: the excess at C_opt, the flanks, and the SUBLINEARITY ratio.
# `_pocket_quench_mean_ei` is driven DIRECTLY here rather than through a `PocketQuenchPDF`: the
# config's only job is to turn a jet into (g, τ_core), and § 9 exercises that mapping end to end.
# Driving the function directly is what lets the τ_core sweep reach values no jet in the shipped
# band produces.
TAU_CORES = [2.5e-3, 4.0e-3, 6.0e-3, 1.0e-2]
for tc in TAU_CORES:
    for gs in (0.05, 0.12):
        ei, ma = G._pocket_quench_mean_ei(FAR_1, Tt3_1, P_1, HF, TAU, tc, gs,
                                          n_bell=NB16, n_quad=NQ16,
                                          quench_ngrid=NG16, quench_nsteps=NS16)
        put(f"r16/dp1/{tc!r}/{gs!r}/ei", ei)
        put(f"r16/dp1/{tc!r}/{gs!r}/max_a", ma)
    print(f"  r16 tau_core={tc!r} done at {time.time() - T0:.1f}s")
# the delta short-circuit (g→0 ⇒ one pocket AT the mean, which for a lean mean is ≈0) and the
# super-eq arm — both are branches, not values.
for gs in (0.0, 1e-12):
    ei, ma = G._pocket_quench_mean_ei(FAR_1, Tt3_1, P_1, HF, TAU, 2.5e-3, gs,
                                      n_bell=NB16, n_quad=NQ16,
                                      quench_ngrid=NG16, quench_nsteps=NS16)
    put(f"r16/dp1/delta/{gs!r}/ei", ei)
    put(f"r16/dp1/delta/{gs!r}/max_a", ma)
ei_su, ma_su = G._pocket_quench_mean_ei(FAR_1, Tt3_1, P_1, HF, TAU, 4.0e-3, 0.12,
                                        n_bell=NB16, n_quad=NQ16, quench_ngrid=NG16,
                                        quench_nsteps=NS16, super_eq_o=True)
put("r16/dp1/su/ei", ei_su)
put("r16/dp1/su/max_a", ma_su)
# THE SUBLINEARITY RATIO — two values from the SAME sweep. Rung 16's mechanism is that a
# lingering pocket COOLS, so term 2 grows SUBLINEARLY in τ_core, where rung 15's D(u)·EI grows
# EXACTLY linearly. Both ratios are dumped over the same τ_core pair, so the gate compares them
# to each other rather than to a remembered constant.
lo = G._pocket_quench_mean_ei(FAR_1, Tt3_1, P_1, HF, TAU, 2.5e-3, 0.12, n_bell=NB16,
                              n_quad=NQ16, quench_ngrid=NG16, quench_nsteps=NS16)[0]
hi = G._pocket_quench_mean_ei(FAR_1, Tt3_1, P_1, HF, TAU, 6.0e-3, 0.12, n_bell=NB16,
                              n_quad=NQ16, quench_ngrid=NG16, quench_nsteps=NS16)[0]
put("r16/sublinear/ratio16", hi / lo)
put("r16/sublinear/ratio_dwell", 6.0e-3 / 2.5e-3)

# FOUR SINGLE POCKETS BY HAND — the localisation rows. `_pocket_quench_mean_ei` returns one
# number over 24 pockets; if it disagrees, these say WHICH pocket. Each is the function's own
# rich-of-mean branch, transcribed: AFT → equilibrium → seed NO → `_quench_no` at τ_core.
XI_GRID16 = [XI_MAX * (i + 0.5) / NB16 for i in range(NB16)]
for i in (8, 12, 16, 20):
    xi = XI_GRID16[i]
    far_local = xi / (1.0 - xi)
    put(f"r16/pocket{i}/xi", xi)
    put(f"r16/pocket{i}/far_local", far_local)
    T_p = G._primary_aft(far_local, P_1, Tt3_1, HF)
    alpha = FAR_1 / far_local
    comp = G._equilibrium_composition(far_local, T_p, P_1)
    n0 = alpha * G._thermal_no(comp, T_p, P_1, TAU, far_local).x_no * sum(comp.values())
    put(f"r16/pocket{i}/T_p", T_p)
    put(f"r16/pocket{i}/alpha", alpha)
    put(f"r16/pocket{i}/n0", n0)
    q = G._quench_no(comp, T_p, alpha, FAR_1, Tt3_1, P_1, n0, 4.0e-3,
                     nsteps=NS16, ngrid=NG16)
    put(f"r16/pocket{i}/ei", q["ei"])
    put(f"r16/pocket{i}/T_peak", q["T_peak"])
    put(f"r16/pocket{i}/max_a", q["max_a"])

print(f"  section 8 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# =============================================================================================
# SECTION 9 — the PUBLIC entry point: `Gas.zoned_nox` with the four new closures
# =============================================================================================
# Sections 1-8 drive the private functions, which is how the sweeps stay affordable; this section
# certifies the WIRING — that `zoned_nox` reads C from the right config, hands `_pdf_mean_ei` the
# right g, adds term 1 and term 2 in the right order, and leaves the rung-9/10/11 scalars alone.
# Each call rebuilds its own bell and trajectory, so the list is short by design.
def dump_zoned(tag, dp, phi, **kw):
    Tt3, Tt4, far, p = DESIGN[dp]
    z = GAS.zoned_nox(far, Tt3, Tt4, p, phi, tau=TAU, quench_ngrid=NGRID,
                      quench_nsteps=NSTEPS, **kw)
    put(f"zn/{tag}/ei_no", z.ei_no)              # rung-9 scalar: untouched by every closure
    put(f"zn/{tag}/ei_quenched", z.ei_no_quenched)      # term 1 for rungs 15/16
    put(f"zn/{tag}/max_a", z.max_a_quench)
    put(f"zn/{tag}/C_holdeman", z.C_holdeman)
    put(f"zn/{tag}/g_seg", z.g_seg)
    if z.ei_no_pdf is not None:
        put(f"zn/{tag}/ei_pdf", z.ei_no_pdf)
    if z.ei_no_pdf_quench is not None:
        put(f"zn/{tag}/ei_pdf_excess", z.ei_no_pdf_excess)
        put(f"zn/{tag}/ei_pdf_quench", z.ei_no_pdf_quench)
    if z.ei_no_pocket_quench is not None:
        put(f"zn/{tag}/ei_pocket_excess", z.ei_no_pocket_excess)
        put(f"zn/{tag}/ei_pocket_quench", z.ei_no_pocket_quench)
    if z.ei_no_transported is not None:
        put(f"zn/{tag}/g_ceiling", z.g_ceiling)
        put(f"zn/{tag}/g_transported", z.g_transported)
        put(f"zn/{tag}/ei_transported", z.ei_no_transported)
    return z


PDF_S = G.MixingPDF(S=0.0625, n_bell=NB, n_quad=NQ)
QP_S = G.QuenchPDF(S=0.0625, n_bell=NB, n_quad=NQ)
PQ_S = G.PocketQuenchPDF(S=0.0625, n_bell=NB16, n_quad=NQ16)
TR_S = G.TransportedPDF(S=0.0625, n_bell=NB, n_quad=NQ, n_ode=200)

dump_zoned("r13/J9", "dp1", PHI_P, mixing=mix(9.0), pdf=PDF_S)
dump_zoned("r13/J16", "dp1", PHI_P, mixing=mix(16.0), pdf=PDF_S)     # C = C_opt exactly ⇒ g = 0
dump_zoned("r13/J36", "dp1", PHI_P, mixing=mix(36.0), pdf=PDF_S)
dump_zoned("r13/J16/su", "dp1", PHI_P, mixing=mix(16.0), pdf=PDF_S, super_eq_o=True)
dump_zoned("r13/J36/su", "dp1", PHI_P, mixing=mix(36.0), pdf=PDF_S, super_eq_o=True)
dump_zoned("r15/J9", "dp1", PHI_P, mixing=mix(9.0), pdf_quench=QP_S)
dump_zoned("r15/J16", "dp1", PHI_P, mixing=mix(16.0), pdf_quench=QP_S)
dump_zoned("r15/J64", "dp1", PHI_P, mixing=mix(64.0), pdf_quench=QP_S)
dump_zoned("r15/J64/su", "dp1", PHI_P, mixing=mix(64.0), pdf_quench=QP_S, super_eq_o=True)
dump_zoned("r16/J16", "dp1", PHI_P, mixing=mix(16.0), pocket_quench=PQ_S)
dump_zoned("r16/J64", "dp1", PHI_P, mixing=mix(64.0), pocket_quench=PQ_S)
dump_zoned("r18/J9", "dp1", PHI_P, mixing=mix(9.0), transported=TR_S)
dump_zoned("r18/J16", "dp1", PHI_P, mixing=mix(16.0), transported=TR_S)
dump_zoned("r18/J25", "dp1", PHI_P, mixing=mix(25.0), transported=TR_S)
dump_zoned("r18/J16/su", "dp1", PHI_P, mixing=mix(16.0), transported=TR_S, super_eq_o=True)
dump_zoned("r13/dp4", "dp4", PHI_P, mixing=mix(16.0), pdf=PDF_S)

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-3C mixing-PDF oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
