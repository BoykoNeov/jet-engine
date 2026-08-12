"""THE ORACLE, phase 3 slice B — every rung-10/11/12/20 finite-quench value the Rust must
reproduce.

The fourth in the family (`dump_gas.py` → `dump_cycle.py` → `dump_nox.py` → here), and a
SEPARATE file from slice A's rather than an extension of it, for a cost reason: a quench
trajectory is `ngrid` mix-out bisections each re-solving the 8-species Newton ~31 times, so
folding these rows into `dump_nox.py` would drag minutes of trajectory builds along every time
the cheap rung-7/8/9/19 keys were regenerated. Each dump's regeneration cost stays proportional
to what it certifies, and slice A's committed TSVs stay frozen as their own audit trail.

WHAT IS ACTUALLY NEW HERE, and therefore what the sweep is built around:

  * `_quench_trajectory` — a THIRD loop around slice A's deepest nesting: ngrid mix-out
    bisections, each a bisection over the 8-species Newton. Every point is a DISTINCT root
    (each sits at its own `far_local`), so the trajectory dump is `ngrid` distinct roots per
    (design point, φ_p) pair and `quench_oracle.rs` asserts that count.
  * `_quench_no` — a clamp-free RK4 in REAL time indexed on a β schedule. No stopping rule, so
    what it measures is accumulation order: `t += dt` over `nsteps`, `n_no += dt/6·(…)`, and
    `(t + 0.5·dt)/tau_q` — each a different function in the last bit from its obvious rewrite.
  * `JetMixing.schedule` — `(1 − tfrac) ** shape_n` with a float ATTRIBUTE exponent. PyPy
    rewrites `x ** 2` (integer literal) into a multiply and does NOT rewrite this, so the Rust
    must reach libm `pow`. SECTION 1 answers that in isolation, in milliseconds, instead of
    letting it surface as "EI differs" three sections later.
  * `JetMixing.tau_q` / `Unmixedness.C` — `math.sqrt`, which is the sqrt instruction and NOT
    Python's `** 0.5`. This is the INVERSE of phase 2's trap; applying "always powp"
    mechanically would get it backwards, so both are dumped.

THE SIZING LEVER. `_quench_trajectory` takes no `tau_q`, no J and no schedule — the fast
chemistry does not know how fast the mixing is. So ONE trajectory per (design point, φ_p) pair
serves the entire rung-10 τ_q sweep, the entire rung-11 J sweep, rung 12's bulk/core pair and
rung 20's lifted arm. Five trajectories carry every sweep below. `_quench_no` takes `tab=` for
exactly this reason, and the Python tests' `_reusable_traj` is the same pattern.

SHAPE KEYS. Slice A measured that the two interpreters can disagree on an extremum's VALUE
while agreeing on its LOCATION exactly, and its finding 1 says every later slice with a
location claim must dump its argmax. Two apply here and both are dumped as their own keys:
  * rung 10's smoking gun is WHERE along the mixing path the temperature peaks (at the
    stoichiometric crossing for a rich primary; at β=0 for a lean one);
  * rung 12's whole claim is WHERE the emissions minimum sits (AT the Holdeman optimum), and
    that it SHIFTS as (H/S)² — so the argmin is dumped at two jet spacings, which makes the
    shift checkable as a relationship between two locations rather than as two values.
Both grids are deliberately coarse (β on 1/32ths, J in factor-of-2 steps) so the extremum sits
several steps clear of its neighbours: a fine grid makes an argmax a coin-flip between adjacent
cells and turns a real detector into a flaky one.

Single-use by design (docs/plans/todo-rust-port.md): it validates the Rust and is deleted at
phase 8. It reaches into `turbojet.gas`'s private names on purpose — it is not an API consumer,
it is a reference dump.

Output is TSV, one row per value:  key <TAB> u64-bits <TAB> repr

Run under BOTH interpreters — whatever PyPy and CPython disagree by is a deviation the project
ALREADY tolerates, and that gap is the principled tolerance floor rather than an invented one.

    C:\\Python314\\python.exe rust/oracle/dump_quench.py rust/oracle/quench_cpython.tsv
    .venv\\Scripts\\python.exe  rust/oracle/dump_quench.py rust/oracle/quench_pypy.tsv
"""
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


# --- resolutions ---------------------------------------------------------------------------
# NGRID=33 puts β on exact 1/32ths — dyadic, so β itself carries no representation difference
# and what the trajectory dump measures is the mix-out root, not the grid. It is also coarse
# enough for the argmax-β detector to sit clear of its neighbours (the resolution note in
# `zoned_nox` says the SHAPE is settled by ~32 points; the 240 production default is for the
# anchor's digits, and a 240-point build here would cost minutes per interpreter).
NGRID = 33
# NSTEPS: the RK4 has no stopping rule, so more steps buy accumulation depth, not accuracy of a
# root. 800 is the sweep default; the anchor rows below re-run a few points at the production
# 2000 so the deeper accumulation is measured too rather than assumed to behave the same.
NSTEPS = 800
NSTEPS_DEEP = 2000
TAU = 3e-3
HF = G._HF_FUEL_DEFAULT


# --- SECTION 1: the mixing ALGEBRA — no solver, no composition, no integrator ---------------
# This section exists to answer the power-spelling question IN ISOLATION. If the Rust spells
# `(1-x)**n` as a product chain, or `math.sqrt(J)` as `powp(J, 0.5)`, these rows say so
# directly, in milliseconds, instead of the defect surfacing as an EI mismatch after a
# trajectory build. Every value below is pure algebra.
SHAPE_N = [1.0, 1.5, 2.0, 2.5, 3.0, 4.0]
TFRAC = [0.0, 0.125, 0.25, 0.3333333333333333, 0.5, 0.625, 0.75, 0.875, 0.9, 1.0]
J_GRID = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0]   # factor-of-2: contains 4 and 16 EXACTLY

for n in SHAPE_N:
    jm = G.JetMixing(J=16.0, shape_n=n)
    for x in TFRAC:
        put(f"sched/{n!r}/{x!r}", jm.schedule(x))
# tau_q over J, and over the other three knobs at fixed J — H/C_e/U_c enter linearly, so a
# transposed factor shows up here and nowhere else.
for j in J_GRID:
    put(f"tauq/J/{j!r}", G.JetMixing(J=j).tau_q)
for h in (0.05, 0.10, 0.20):
    put(f"tauq/H/{h!r}", G.JetMixing(J=16.0, H=h).tau_q)
for ce in (0.10, 0.15, 0.25):
    put(f"tauq/Ce/{ce!r}", G.JetMixing(J=16.0, C_e=ce).tau_q)
for uc in (50.0, 75.0, 120.0):
    put(f"tauq/Uc/{uc!r}", G.JetMixing(J=16.0, U_c=uc).tau_q)

# The rung-12 group and its three derived quantities. The `min(w_max, ·)` cap and the |ln|
# KINK are both here: the C grid straddles C_opt on both flanks and reaches far enough out that
# w saturates, which is exactly the kind of clamp a port puts on the wrong side of a multiply.
for s in (0.0625, 0.125):
    um = G.Unmixedness(S=s)
    for j in J_GRID:
        jm = G.JetMixing(J=j)
        C = um.C(jm)
        put(f"holdeman/{s!r}/{j!r}/C", C)
        put(f"holdeman/{s!r}/{j!r}/u", um._u(C))
        put(f"holdeman/{s!r}/{j!r}/w", um.core_fraction(C))
        put(f"holdeman/{s!r}/{j!r}/tcore", um.core_dwell(C))
# k_u=0 (the rung-11 reduce) and a w_max that binds early — both are branch selectors.
for ku, wmax in ((0.0, 0.7), (2.5, 0.2), (5.0, 0.7)):
    um = G.Unmixedness(k_u=ku, w_max=wmax)
    for j in (1.0, 16.0, 128.0):
        put(f"holdeman/ku{ku!r}w{wmax!r}/{j!r}/w", um.core_fraction(um.C(G.JetMixing(J=j))))

print(f"  section 1 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# --- SECTION 2: the design points (derived from REAL engine runs, never hardcoded) ----------
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

GAS = G.Gas.reacting_equilibrium()

# --- SECTION 3: the TRAJECTORIES — ngrid distinct mix-out roots apiece ----------------------
# Five (design point, φ_p) pairs. φ_p=0.8 is LEAN (the trajectory only cools: T_peak must be
# T(β=0)); 1.0 sits at the bell peak; 1.5 and 2.0 are RICH, where the local mixture sweeps UP
# through stoichiometric — the smoking gun. dp4 repeats φ=1.5 at a different (Tt3, Tt4, far, p)
# so the trajectory's dependence on the design point is measured and not assumed.
#
# Every one of the ngrid points is a DISTINCT mix-out root (its own far_local), so this section
# alone is 5·33 = 165 distinct bisections-over-a-Newton. `quench_oracle.rs` asserts that count
# so it cannot silently collapse to a handful of repeated roots.
TRAJ = {}       # (dp, phi) -> dict(comp, T_p, alpha, n0, tab, ei9, far_p)


def trajectory(dp, phi_p):
    key = (dp, phi_p)
    if key not in TRAJ:
        Tt3, Tt4, far, p = DESIGN[dp]
        far_p = phi_p * G._F_STOICH
        alpha = far / far_p
        T_p = G._primary_aft(far_p, p, Tt3, HF)
        comp = G._equilibrium_composition(far_p, T_p, p)
        nox = G._thermal_no(comp, T_p, p, TAU, far_p)
        n0 = alpha * nox.x_no * sum(comp.values())
        tab = G._quench_trajectory(comp, T_p, alpha, far, Tt3, p, ngrid=NGRID)
        TRAJ[key] = dict(comp=comp, T_p=T_p, alpha=alpha, n0=n0, tab=tab,
                         ei9=nox.ei_no, far_p=far_p)
        print(f"  traj {dp}/{phi_p} built at {time.time() - T0:.1f}s")
    return TRAJ[key]


TRAJ_CASES = [("dp1", 0.8), ("dp1", 1.0), ("dp1", 1.5), ("dp1", 2.0), ("dp4", 1.5)]
for dp, phi_p in TRAJ_CASES:
    t = trajectory(dp, phi_p)
    tag = f"{dp}/{phi_p!r}"
    put(f"traj/{tag}/T_p", t["T_p"])
    put(f"traj/{tag}/alpha", t["alpha"])
    put(f"traj/{tag}/n0", t["n0"])
    put(f"traj/{tag}/ei9", t["ei9"])
    for i, row in enumerate(t["tab"]):
        for k in ("a", "T", "cO", "cN2", "cH", "cNOe", "ntot_local", "V"):
            put(f"traj/{tag}/{i}/{k}", row[k])
    # SHAPE KEY — rung 10's smoking gun is a LOCATION: where along the mixing path the
    # temperature peaks. For a RICH primary that is the stoichiometric crossing, several β
    # steps in; for a LEAN one it is β=0 and the trajectory only cools. `max()` keeps the FIRST
    # maximum, so the lean case reports index 0 exactly — which is what makes this a detector
    # for "the trajectory runs the wrong way" rather than a value comparison.
    Ts = [row["T"] for row in t["tab"]]
    put(f"traj/{tag}/argmax_i", float(max(range(len(Ts)), key=lambda i: Ts[i])))
    put(f"traj/{tag}/T_peak", max(Ts))
    put(f"traj/{tag}/T_end", Ts[-1])

print(f"  section 3 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# --- SECTION 4: RUNG 10 — the τ_q sweep on a prebuilt trajectory ----------------------------
# The rung's claim is that EI rises MONOTONICALLY with τ_q — a slow quench dwells at the stoich
# crossing and re-makes the NO a rich primary avoided. Five decades of τ_q, all five fields, on
# the trajectory built above (τ_q-independent, so this is free).
TAU_Q = [1e-5, 3e-5, 1e-4, 3e-4, 1e-3, 3e-3, 1e-2]


def dump_quench(tag, t, dp, tau_q, *, schedule=None, super_eq_o=False, nsteps=NSTEPS):
    Tt3, _Tt4, far, p = DESIGN[dp]
    q = G._quench_no(t["comp"], t["T_p"], t["alpha"], far, Tt3, p, t["n0"], tau_q,
                     nsteps=nsteps, ngrid=NGRID, tab=t["tab"], schedule=schedule,
                     super_eq_o=super_eq_o)
    for k in ("ei", "x_no_mix", "n_no", "T_peak", "max_a"):
        put(f"{tag}/{k}", q[k])
    return q


for dp, phi_p in TRAJ_CASES:
    t = trajectory(dp, phi_p)
    for tau_q in TAU_Q:
        dump_quench(f"r10/{dp}/{phi_p!r}/{tau_q!r}", t, dp, tau_q)
# The DEEP-accumulation arm: the same points at the production 2000 steps. The RK4 carries no
# stopping rule, so this measures 2000 accumulations rather than 800 — the one thing that
# genuinely differs between the two, and the reason it is dumped rather than inferred.
for tau_q in (1e-4, 1e-3, 3e-3):
    dump_quench(f"r10deep/dp1/1.5/{tau_q!r}", trajectory("dp1", 1.5), "dp1", tau_q,
                nsteps=NSTEPS_DEEP)

print(f"  section 4 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# --- SECTION 5: RUNG 11 — the J sweep (DERIVED τ_q + entrainment schedule) ------------------
# `mixing` replaces rung 10's free time with τ_q(J) AND a decelerating schedule. Both change at
# once, which is why the shape_n=1 row matters: at shape_n=1 the schedule is the IDENTITY, so
# rung 11 at the derived τ_q must be BYTE-IDENTICAL to the rung-10 path at that same τ_q. That
# is the reduce contract, and it is dumped as a pair of keys rather than asserted only in Rust.
for dp, phi_p in (("dp1", 1.5), ("dp1", 1.0)):
    t = trajectory(dp, phi_p)
    for j in J_GRID:
        jm = G.JetMixing(J=j)
        put(f"r11/{dp}/{phi_p!r}/{j!r}/tau_q", jm.tau_q)
        dump_quench(f"r11/{dp}/{phi_p!r}/{j!r}", t, dp, jm.tau_q, schedule=jm.schedule)
# the shape_n reduce pair, at one J
T15 = trajectory("dp1", 1.5)
JM1 = G.JetMixing(J=16.0, shape_n=1.0)
dump_quench("r11/reduce/sched1", T15, "dp1", JM1.tau_q, schedule=JM1.schedule)
dump_quench("r11/reduce/linear", T15, "dp1", JM1.tau_q, schedule=None)
# and the shape_n sensitivity at fixed τ_q — isolates the SCHEDULE from the TIME.
for n in (1.5, 2.0, 3.0):
    jm = G.JetMixing(J=16.0, shape_n=n)
    dump_quench(f"r11/shape/{n!r}", T15, "dp1", jm.tau_q, schedule=jm.schedule)

print(f"  section 5 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# --- SECTION 6: RUNG 12 — the two-stream split, and WHERE its minimum sits ------------------
# THE SHAPE KEY OF THE SLICE. Rung 12's claim is a LOCATION: EI_NO falls then RISES, with the
# minimum pinned AT the Holdeman optimum C_opt, so J_min = (C_opt·H/S)², shifting as (H/S)².
#
# (a) the ABSOLUTE sweep at the default spacing: the fall-then-rise curve on a factor-of-2 J
#     grid that contains J_opt=16 exactly, so the argmin is a clean index several steps clear
#     of its neighbours.
for s in (0.0625,):
    um = G.Unmixedness(S=s)
    best_j, best_ei = None, float("inf")
    for j in J_GRID:
        jm = G.JetMixing(J=j)
        C = um.C(jm)
        w = um.core_fraction(C)
        tag = f"r12/{s!r}/{j!r}"
        put(f"{tag}/C", C)
        put(f"{tag}/w", w)
        qb = dump_quench(f"{tag}/bulk", T15, "dp1", jm.tau_q, schedule=jm.schedule)
        qc = dump_quench(f"{tag}/core", T15, "dp1", um.core_dwell(C), schedule=jm.schedule)
        ei = (1.0 - w) * qb["ei"] + w * qc["ei"]
        put(f"{tag}/ei_unmixed", ei)
        if ei < best_ei:
            best_j, best_ei = j, ei
    put(f"r12/{s!r}/argmin_J", best_j)
    put(f"r12/{s!r}/min_ei", best_ei)

# (b) the (H/S)² SHIFT, on the J grid RELATIVE to each spacing's own J_opt — the same grid the
#     Python's own gate 3 uses, `[J_opt/4, J_opt/2, J_opt, 2·J_opt, 4·J_opt]`, so the argmin
#     landing at INDEX 2 IS the claim and the shift is a relationship between two LOCATIONS
#     (J_opt = 25 at S=0.05, 16 at S=0.0625; 25/16 = (0.0625/0.05)² exactly) rather than a
#     comparison of two values.
#
# (c) AND THE BOUNDARY, which is why this section is not just (b) twice. Rung 12's docstring
#     claims the min pins at C_opt "for ALL S". IT DOES NOT, and the port's shape key is what
#     found it: at S=0.125 the argmin sits at INDEX 3 (2·J_opt), not 2. The Python's own gate
#     never sees this because it tests only S ∈ {0.0625, 0.05}, both inside the valid band.
#
#     The mechanism is the docstring's OWN pin inequality, k_u·[EI(τ_core) − EI(τ_mean)] >
#     EI(τ_mean) evaluated at C_opt (where w=0). At the optimum τ_mean = S/(C_e·C_opt·U_c)
#     GROWS with the spacing, so a wide enough spacing makes the mean-field bulk quench SLOWER
#     than the "lingering" core — the model's premise inverts and the core becomes a relief
#     rather than a penalty. Both sides of that inequality are dumped at every spacing below, so
#     the Rust asserts the CONDITION and not the over-stated claim.
#
#     MEASURED, and worth stating precisely because the inequality is CONSERVATIVE: it goes
#     false at S=0.0625 while the pin survives to S=0.08 and breaks between 0.08 and 0.09. The
#     inequality assumes EI ∝ τ (that is what turns dE/dlnJ into −E/2); EI is SUBLINEAR in
#     dwell at these times, so the bulk falls more slowly than assumed and the pin holds longer
#     than the algebra predicts. The shipped default S=0.0625 is inside the band, but only by
#     about 1.4× — which is the useful thing to know and was invisible before this sweep.
SHIFT_S = [0.05, 0.0625, 0.08, 0.09, 0.125]
for s in SHIFT_S:
    um = G.Unmixedness(S=s)
    j_opt = (um.C_opt * G.JetMixing(J=1.0).H / um.S) ** 2
    put(f"r12shift/{s!r}/J_opt", j_opt)
    js = [j_opt / 4, j_opt / 2, j_opt, 2 * j_opt, 4 * j_opt]   # C = C_opt·{.5,.707,1,1.41,2}
    eis = []
    for k, j in enumerate(js):
        jm = G.JetMixing(J=j)
        C = um.C(jm)
        w = um.core_fraction(C)
        tag = f"r12shift/{s!r}/{k}"
        put(f"{tag}/J", j)
        put(f"{tag}/C", C)
        put(f"{tag}/w", w)
        qb = dump_quench(f"{tag}/bulk", T15, "dp1", jm.tau_q, schedule=jm.schedule)
        qc = dump_quench(f"{tag}/core", T15, "dp1", um.core_dwell(C), schedule=jm.schedule)
        ei = (1.0 - w) * qb["ei"] + w * qc["ei"]
        put(f"{tag}/ei_unmixed", ei)
        eis.append(ei)
    imin = min(range(len(eis)), key=lambda i: eis[i])
    put(f"r12shift/{s!r}/argmin_i", float(imin))       # 2 ⇒ pinned AT C_opt
    # the pin inequality's two sides, at C_opt itself (w=0 there, so these ARE the endpoints
    # the docstring's condition compares). `tau_mean_opt` is dumped beside them because its
    # crossing of tau_res is the physical reading of the same thing.
    jm_opt = G.JetMixing(J=j_opt)
    e_m = dump_quench(f"r12shift/{s!r}/pin_Em", T15, "dp1", jm_opt.tau_q,
                      schedule=jm_opt.schedule)["ei"]
    e_c = dump_quench(f"r12shift/{s!r}/pin_Ec", T15, "dp1", um.tau_res,
                      schedule=jm_opt.schedule)["ei"]
    put(f"r12shift/{s!r}/tau_mean_opt", jm_opt.tau_q)
    put(f"r12shift/{s!r}/pin_lhs", um.k_u * (e_c - e_m))
    put(f"r12shift/{s!r}/pin_rhs", e_m)
# k_u=0 — the exact rung-11 reduce. w=0 ⇒ ei_unmixed == the bulk EI, and note this is NOT a
# short-circuit: the core integration still RUNS at τ_core and must not trip an assert. That is
# why the core EI is dumped here too, at a dwell no other row uses.
UM0 = G.Unmixedness(k_u=0.0)
JM16 = G.JetMixing(J=16.0)
C0 = UM0.C(JM16)
put("r12/ku0/C", C0)
put("r12/ku0/w", UM0.core_fraction(C0))
QB0 = dump_quench("r12/ku0/bulk", T15, "dp1", JM16.tau_q, schedule=JM16.schedule)
QC0 = dump_quench("r12/ku0/core", T15, "dp1", UM0.core_dwell(C0), schedule=JM16.schedule)
put("r12/ku0/ei_unmixed", (1.0 - UM0.core_fraction(C0)) * QB0["ei"]
    + UM0.core_fraction(C0) * QC0["ei"])

print(f"  section 6 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# --- SECTION 7: RUNG 20 — the super-eq O lift THROUGH the quench ----------------------------
# m(T) multiplies [O] INSIDE the re-making, so it scales formation and the reverse alike and is
# NOT a post-hoc factor on the m=1 answer — it has to be measured. The floor at 1500 K matters
# on the cool tail of every trajectory (m diverges as T→0), and `_SUPER_EQ_T_FLOOR` sits in the
# grid exactly, so a comparison spelled `<` in one language and `<=` in the other would show.
for dp, phi_p in (("dp1", 1.0), ("dp1", 1.5), ("dp4", 1.5)):
    t = trajectory(dp, phi_p)
    for tau_q in (1e-4, 1e-3, 3e-3):
        dump_quench(f"r20/{dp}/{phi_p!r}/{tau_q!r}", t, dp, tau_q, super_eq_o=True)
# the lift THROUGH a rung-11 jet, and through the rung-12 core dwell — the two places rung 20
# threads that rung 19 did not reach.
JM16 = G.JetMixing(J=16.0)
dump_quench("r20/jet/J16", T15, "dp1", JM16.tau_q, schedule=JM16.schedule, super_eq_o=True)
UM = G.Unmixedness()
dump_quench("r20/core/S0625", T15, "dp1", UM.core_dwell(UM.C(JM16)),
            schedule=JM16.schedule, super_eq_o=True)

print(f"  section 7 done at {time.time() - T0:.1f}s ({len(ROWS)} rows)")

# --- SECTION 8: the PUBLIC entry point — Gas.zoned_nox with the new knobs -------------------
# Sections 3–7 drive the private functions, which is how the sweeps stay affordable; this
# section certifies the WIRING that connects them — that `zoned_nox` picks the right τ_q, hands
# `_quench_no` the right schedule, shares one trajectory between bulk and core, and leaves the
# rung-9 scalars untouched. Each call here rebuilds its own trajectory, so the list is short by
# design.
def dump_zoned(tag, far, Tt3, Tt4, p, phi, **kw):
    z = GAS.zoned_nox(far, Tt3, Tt4, p, phi, tau=TAU, quench_ngrid=NGRID,
                      quench_nsteps=NSTEPS, **kw)
    put(f"zn/{tag}/ei_no", z.ei_no)             # rung-9 scalar: must be untouched by the quench
    put(f"zn/{tag}/x_no_mix", z.x_no_mix)
    put(f"zn/{tag}/T_primary", z.T_primary)
    put(f"zn/{tag}/T_mix", z.T_mix)
    put(f"zn/{tag}/tau_q", z.tau_q)
    put(f"zn/{tag}/ei_quenched", z.ei_no_quenched)
    put(f"zn/{tag}/x_quenched", z.x_no_quenched)
    put(f"zn/{tag}/T_peak", z.T_peak)
    put(f"zn/{tag}/max_a", z.max_a_quench)
    if z.ei_no_unmixed is not None:
        put(f"zn/{tag}/C_holdeman", z.C_holdeman)
        put(f"zn/{tag}/w_core", z.w_core)
        put(f"zn/{tag}/ei_core", z.ei_no_core)
        put(f"zn/{tag}/ei_unmixed", z.ei_no_unmixed)
    return z


TT3_1, TT4_1, FAR_1, PT4_1 = DESIGN["dp1"]
dump_zoned("r10/tq1e-3", FAR_1, TT3_1, TT4_1, PT4_1, 1.5, tau_q=1e-3)
dump_zoned("r10/tq3e-3", FAR_1, TT3_1, TT4_1, PT4_1, 1.5, tau_q=3e-3)
dump_zoned("r10/lean", FAR_1, TT3_1, TT4_1, PT4_1, 0.9, tau_q=1e-3)
dump_zoned("r11/J16", FAR_1, TT3_1, TT4_1, PT4_1, 1.5, mixing=G.JetMixing(J=16.0))
dump_zoned("r11/J64", FAR_1, TT3_1, TT4_1, PT4_1, 1.5, mixing=G.JetMixing(J=64.0))
dump_zoned("r12/J16", FAR_1, TT3_1, TT4_1, PT4_1, 1.5,
           mixing=G.JetMixing(J=16.0), unmixedness=G.Unmixedness())
dump_zoned("r12/J128", FAR_1, TT3_1, TT4_1, PT4_1, 1.5,
           mixing=G.JetMixing(J=128.0), unmixedness=G.Unmixedness())
dump_zoned("r20/J16", FAR_1, TT3_1, TT4_1, PT4_1, 1.5,
           mixing=G.JetMixing(J=16.0), unmixedness=G.Unmixedness(), super_eq_o=True)

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-3B finite-quench oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
