"""THE ORACLE, phase 6 slice P — every rung-34/35/36 value the Rust must reproduce.

The port's FIRST ODE. `SpoolTransient` throws away the shaft balance every steady matcher is
built around, replaces it with a forward compressor closure, and marches the leftover power
imbalance under a fixed-step RK4. Rung 35 re-controls the same plant on FUEL so `Tt4` becomes an
output; rung 36 hangs a read-only surge line beside the running line.

WHAT IS NEW HERE, and why each thing is dumped rather than asserted:

  * A TRAJECTORY WHOSE LENGTH IS AN OUTPUT. Both marches `break` when any RK sub-stage leaves the
    valid region, so how many points come back is decided by the physics, not by `s_end/ds`.
    S 5.13 probe 5 measured it varying by MAP SHAPE on the spool-down — 66 of 161 steps, 81 of
    161, and one full. Every march therefore dumps `n_pts` as a discrete key BEFORE any value.

  * A BRANCH ON WHICH THE VIRTUAL HOOK IS DEAD. `_instant_tail` solves the choked (star) geometry
    through `_solve_turbine`, dispatches on the nozzle, and on the SUBSONIC branch re-solves
    `pi_t` from nozzle continuity — discarding the hook's answer entirely. Slice P measured the
    consequence: swapping rung 34's Illinois for rung 31's bisection leaves the subsonic cells
    bit-identical and ALL 19 ported Python gates passing. So the hook is gated by a COUNT.

  * TWO ARMS OF ONE `>`, 185 FIRINGS APART. The `M9 > 0.985` guard decides whether a failed
    subsonic bracket is the continuous choke boundary (absorb) or a real solve gap (RAISE). A
    port that swapped them moves no value key at all, so both are dumped as counts.

  * A ROOT FINDER WHOSE DELICATE DETAILS ARE NOT VALUES. Injecting a reordered convergence test
    into `_illinois` leaves every value bit-exact and changes only how many residual evaluations
    happen; exhausting `maxit` is unreachable. Both are dumped as counts. Measured by WRAPPING
    THE RESIDUAL, never by copying the loop — a copy would gate the copy.

  * `ComponentMap.phi_max`, OWED SINCE SLICE M AND DEAD IN TWO OF THREE ARMS. A rung-34 march
    reaches only `flat5` and `quadratic`, and never a nonzero `vsv`. The per-arm tallies are
    dumped so "the linear arm is unreachable" is a MEASUREMENT the Rust reproduces rather than a
    sentence in a comment.

Regenerate with:
    .venv\\Scripts\\python.exe rust/oracle/dump_spool.py rust/oracle/spool_pypy.tsv
    C:\\Python314\\python.exe   rust/oracle/dump_spool.py rust/oracle/spool_cpython.tsv
"""
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import engine as E                                        # noqa: E402
from turbojet.engine import (FlightCondition, build_turbojet,           # noqa: E402
                             ComponentMap, SpoolTransient)
from turbojet.gas import Gas                                            # noqa: E402

T0 = time.time()
ROWS = []


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putd(key, n):
    """Record one DISCRETE key — a count, a length, a branch index. Never a rounded float."""
    n = int(n)
    ROWS.append((key, n, str(n)))


# ================================================================= the instruments
# Every one of these WRAPS shipped code. None copies a loop, because a copied loop gates the
# copy (the port's standing rule, and the reason `_illinois`'s evaluation count is measured on
# the RESIDUAL rather than on a re-implementation of the iteration).

CENSUS = {}


def reset_census():
    CENSUS.clear()
    CENSUS.update(illinois_calls=0, illinois_evals=0, illinois_exhausted=0,
                  r34_solve_turbine=0, subsonic_raises=0, subsonic_escalations=0,
                  phi_max_flat5=0, phi_max_quadratic=0, phi_max_linear=0, phi_max_swirled=0)


reset_census()

_ILLINOIS = E._illinois


def _illinois_counted(f, a, b, fa, fb, tol=1e-10, maxit=100):
    """Count residual evaluations by wrapping `f`, and infer exhaustion from the count.

    A call that exhausts `maxit` performs exactly `maxit` evaluations (one per iteration, and it
    never returns early); a converging call performs strictly fewer. So the exhaustion arm is
    observable WITHOUT re-implementing the loop — a copy of the loop would gate the copy.

    **THE FIRST DRAFT OF THIS WRAPPER WAS WRONG, AND THE ORACLE IS WHAT FOUND IT.** It tallied
    into the census AFTER `_ILLINOIS` returned:

        n = [0]
        def counting(x):  n[0] += 1;  return f(x)
        out = _ILLINOIS(counting, ...)
        CENSUS["illinois_evals"] += n[0]        # <-- never runs if the call raised

    An Illinois whose residual raises mid-search — which happens inside every bracket march, and
    is CONTROL FLOW here, not an error — propagates out, so that last line is skipped and the
    call's evaluations are lost entirely. Rust's counter increments in the loop, so it keeps them.
    Result: 7 299 of 7 300 keys bit-exact and `census/equilibria/illinois_evals` off by **9** —
    Rust 16 761, this script 16 752. Every VALUE agreed; only the count did.

    The port's counting is the more informative of the two (it does not discard a partial search),
    so the instrument is aligned to it rather than the reverse: tally inside the wrapper, AFTER
    the residual returns, so a raising evaluation is uncounted on both sides and a partial search
    is counted on both. Fourth instance in this port of a measuring pass finding the defect in
    the INSTRUMENT rather than in the code under test.
    """
    n = [0]

    def counting(x):
        v = f(x)                       # a raise here is counted by NEITHER side
        n[0] += 1
        CENSUS["illinois_evals"] += 1  # tallied HERE, so an aborted search keeps its partial count
        return v

    CENSUS["illinois_calls"] += 1
    try:
        return _ILLINOIS(counting, a, b, fa, fb, tol, maxit)
    finally:
        if n[0] >= maxit:
            CENSUS["illinois_exhausted"] += 1


E._illinois = _illinois_counted

_SOLVE_T = SpoolTransient._solve_turbine


def _solve_t_counted(self, gas, Tt4, f, eta_t=None):
    CENSUS["r34_solve_turbine"] += 1
    return _SOLVE_T(self, gas, Tt4, f, eta_t)


SpoolTransient._solve_turbine = _solve_t_counted

_SUB = SpoolTransient._turbine_subsonic


def _sub_counted(self, *a, **k):
    try:
        return _SUB(self, *a, **k)
    except AssertionError:
        CENSUS["subsonic_raises"] += 1
        raise


SpoolTransient._turbine_subsonic = _sub_counted

_TAIL = SpoolTransient._instant_tail


def _tail_counted(self, *a, **k):
    try:
        return _TAIL(self, *a, **k)
    except AssertionError as ex:
        if "failed to bracket AWAY" in str(ex):
            CENSUS["subsonic_escalations"] += 1
        raise


SpoolTransient._instant_tail = _tail_counted

_PHI_MAX = ComponentMap.phi_max


def _phi_max_counted(self, psi_floor=0.1):
    A = self.vsv * (1.0 + self.l)
    if A != 0.0:
        CENSUS["phi_max_swirled"] += 1
    elif self.sigma == 0.0 and self.l == 0.0:
        CENSUS["phi_max_flat5"] += 1
    elif self.sigma == 0.0:
        CENSUS["phi_max_linear"] += 1
    else:
        CENSUS["phi_max_quadratic"] += 1
    return _PHI_MAX(self, psi_floor)


ComponentMap.phi_max = _phi_max_counted


def emit_census(prefix):
    """Write the census under `prefix/` and RESET it. Each section's counts are its own."""
    for k in sorted(CENSUS):
        putd(f"census/{prefix}/{k}", CENSUS[k])
    reset_census()


# ======================================================================== the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C, TT4 = 10.0, 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)

SHAPES = [
    ("flat", ComponentMap.flat()),
    ("flow_dom", ComponentMap.flow_dominated()),
    ("press_dom", ComponentMap.pressure_dominated()),
    ("tilted", ComponentMap.tilted()),
    ("surge_flow", ComponentMap.surge_flow()),
    ("surge_pressure", ComponentMap.surge_pressure()),
    ("surge_tilted", ComponentMap.surge_tilted()),
]
THROTTLES = [1500.0, 1300.0, 1100.0, 900.0, 700.0, 520.0]
BRANCH_INDEX = {"choked": 0, "subsonic": 1}

EQ_KEYS = ("nu", "n", "pi_c", "tau_c", "mdot_air", "f", "pi_t", "tau_t", "Tt3", "Tt5",
           "flowcoef", "Phi", "sp_thrust", "M9", "pt9_over_p0", "eta_c", "eta_t", "nu_t",
           "p_net_spec", "m", "thrust", "Tt2", "pt2", "V0")
PT_KEYS = ("s", "nu", "Tt4", "pi_c", "tau_c", "mdot_air", "f", "tau_t", "Phi", "sp_thrust",
           "M9", "pt9_over_p0")


def st(cmap, gas=None):
    g = gas if gas is not None else Gas.thermally_perfect()
    return SpoolTransient(
        build_turbojet(g, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL),
        FLIGHT, 1.0, comp_map=cmap)


def dump_traj(prefix, traj):
    """Length FIRST (it is an output), then a fixed sample of the points.

    Sampling every 7th point rather than all of them keeps the file readable while still riding
    the whole accumulation: point 140 of a 161-step march has had 140 RK4 steps of error
    accumulation behind it, so a last-bit divergence anywhere upstream reaches it.
    """
    putd(f"{prefix}/n_pts", len(traj))
    for i in range(0, len(traj), 7):
        p = traj[i]
        putd(f"{prefix}/{i}/branch", BRANCH_INDEX[p.branch])
        for k in PT_KEYS:
            put(f"{prefix}/{i}/{k}", getattr(p, k))
    if traj:
        p = traj[-1]                                # the terminal state, always
        putd(f"{prefix}/last/branch", BRANCH_INDEX[p.branch])
        for k in PT_KEYS:
            put(f"{prefix}/last/{k}", getattr(p, k))


# ---------------------------------------------------------- section 1: equilibria
reset_census()
n_choked = n_subsonic = 0
for name, cmap in SHAPES:
    s = st(cmap)
    for Tt4 in THROTTLES:
        eq = s.equilibrium(FLIGHT, Tt4)
        tag = f"eq/{name}/{Tt4:.0f}"
        putd(f"{tag}/branch", BRANCH_INDEX[eq["branch"]])
        n_choked += eq["branch"] == "choked"
        n_subsonic += eq["branch"] == "subsonic"
        for k in EQ_KEYS:
            put(f"{tag}/{k}", eq[k])
emit_census("equilibria")
putd("cells/choked", n_choked)
putd("cells/subsonic", n_subsonic)

# ------------------------------------------------------ section 2: the Tt4 marches
reset_census()
for name in ("surge_flow", "flow_dom", "flat"):
    cmap = dict(SHAPES)[name]
    s = st(cmap)
    for r in (0.1, 1.0, 5.0):
        d = s.ramp_excursion(FLIGHT, 1100.0, 1450.0, r, s_settle=8.0, ds=0.05)
        put(f"ramp/{name}/{r}/E", d["E"])
        put(f"ramp/{name}/{r}/nu0", d["nu0"])
        dump_traj(f"ramp/{name}/{r}", d["traj"])
    # the fuel-cut spool-down: the march whose LENGTH varies by shape
    nu0 = s.equilibrium(FLIGHT, 1100.0)["nu"]
    traj = s.integrate(FLIGHT, lambda x: 600.0, nu0, 8.0, 0.05)
    dump_traj(f"spooldown/{name}", traj)
    putd(f"spooldown/{name}/nu_floor_hits", sum(1 for p in traj if p.nu == 0.2))
    put(f"const_speed/{name}", s.constant_speed_excursion(FLIGHT, 1100.0, 1450.0))
emit_census("tt4_marches")

# ------------------------------------------------- section 3: rung 35, fuel control
reset_census()
for name in ("surge_flow", "surge_tilted", "flow_dom"):
    cmap = dict(SHAPES)[name]
    s = st(cmap)
    for Tt4 in (1400.0, 1100.0):
        mf = s._fuel_for_Tt4(FLIGHT, Tt4)
        put(f"fuel/{name}/{Tt4:.0f}/mf", mf)
        eq = s.equilibrium_fuel(FLIGHT, mf)
        for k in EQ_KEYS:
            put(f"fuel/{name}/{Tt4:.0f}/{k}", eq[k])
        put(f"fuel/{name}/{Tt4:.0f}/Tt4_out", eq["Tt4"])
    d = s.ramp_excursion_fuel(FLIGHT, 1250.0, 1450.0, 1.0, s_settle=6.0, ds=0.05)
    for k in ("E_surge", "E_temp", "Tt4_peak", "nu0"):
        put(f"fuelramp/{name}/{k}", d[k])
    dump_traj(f"fuelramp/{name}", d["traj"])
    cs = s.constant_speed_excursion_fuel(FLIGHT, 1250.0, 1450.0)
    for k in ("E_surge0", "E_temp0", "Tt4_peak", "Tt4_target"):
        put(f"fuelstep/{name}/{k}", cs[k])
    for Tt3, f in ((650.0, 0.020), (700.0, 0.025), (600.0, 0.030)):
        put(f"tt4_from_f/{name}/{Tt3:.0f}/{f}", s._tt4_from_f(Tt3, f))
emit_census("fuel")

# ------------------------------------------------- section 4: rung 36, the surge line
reset_census()
for name in ("surge_flow", "surge_pressure", "surge_tilted"):
    base = dict(SHAPES)[name]
    for phi_s in (0.55, 0.65, 0.75):
        cm = base.with_phi_surge(phi_s)
        s = st(base)
        sched = s.surge_margin_schedule(FLIGHT, [1500.0, 1300.0, 1100.0, 900.0, 800.0, 700.0], cm)
        putd(f"sm/{name}/{phi_s}/n_rows", len(sched))
        for row in sched:
            tag = f"sm/{name}/{phi_s}/{row['Tt4']:.0f}"
            for k in ("nu", "n", "phi_op", "phi_surge", "pi_c", "SM_N", "SM_flow"):
                put(f"{tag}/{k}", row[k])
        for lo in (1400.0, 1000.0, 800.0, 700.0):
            b = s.acceleration_binding(FLIGHT, lo, 1500.0, cm)
            tag = f"ab/{name}/{phi_s}/{lo:.0f}"
            for k in ("nu0", "E0", "SM_N", "ratio", "phi_step", "phi_surge"):
                put(f"{tag}/{k}", b[k])
            putd(f"{tag}/reaches_surge", 1 if b["reaches_surge"] else 0)
            putd(f"{tag}/phi_step_le_surge", 1 if b["phi_step_le_surge"] else 0)
emit_census("surge")

# --------------------------------------- section 5: rung 41's channels (slice L's deferral)
reset_census()
for name in ("surge_flow", "surge_tilted"):
    base = dict(SHAPES)[name]
    cm = base.with_phi_surge(0.65)
    s = st(base)
    for Tt4 in (1500.0, 1300.0, 1100.0, 900.0, 800.0):
        ch = s.surge_margin_channels(FLIGHT, Tt4, cm)
        tag = f"ch/{name}/{Tt4:.0f}"
        for k in ("n", "phi_op", "pi_c", "SM_N", "SM_phi_walk", "SM_speed_line", "SM_ref"):
            put(f"{tag}/{k}", ch[k])
emit_census("channels")

# ------------------------------------- section 6: phi_max, all arms driven DIRECTLY
reset_census()
DIRECT = [
    ("flat", ComponentMap.flat()),
    ("quad", ComponentMap.surge_flow()),
    ("quad2", ComponentMap(sigma=0.2, l=0.85)),
    ("linear", ComponentMap(sigma=0.0, l=0.7)),
    ("linear2", ComponentMap(sigma=0.0, l=1.4)),
    ("swirl", ComponentMap(sigma=0.1, l=0.7).with_vsv(0.20)),
    ("swirl_lin", ComponentMap(sigma=0.0, l=0.7).with_vsv(0.10)),
    ("swirl_neg", ComponentMap(sigma=0.1, l=0.7).with_vsv(-0.15)),
]
for label, cm in DIRECT:
    for floor in (0.1, 0.2, 0.35):
        put(f"phi_max/{label}/{floor}", cm.phi_max(floor))
emit_census("phi_max_direct")

# ------------------------------------------------ section 7: the map inverse (gate 6)
for name in ("surge_flow", "surge_pressure", "surge_tilted", "flat"):
    cmap = dict(SHAPES)[name]
    s = st(cmap)
    for n in (0.6, 0.75, 0.9, 1.0, 1.1):
        for m in (0.5, 0.8, 1.0, 1.2):
            tc = s._tau_c_forward(cmap, n, m)
            put(f"inv/{name}/{n}/{m}/tau_c", tc)
            put(f"inv/{name}/{n}/{m}/n_back", cmap.solve_n(m, tc, s.tau_c_d))

# ------------------------------------------------------------------------ write
keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-6P spool-transient oracle — key\tu64 bits (or an integer)\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

print(f"[1] cells: {n_choked} choked, {n_subsonic} subsonic")
print(f"[2] {sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
