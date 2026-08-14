"""THE ORACLE, phase 5 slice J — every rung-32 value the Rust must reproduce.

Rung 32 is a SOLVE AROUND SLICE I's SOLVE: an outer secant on `eta_c` whose every pass runs the
whole rung-31 joint `(f, pt4)` fixed point, which itself drives a turbine bisection per pass.
Three nested loops, and the outermost one is new.

WHAT IS DUMPED HERE THAT SLICE I's DUMP COULD NOT BE, and why:

  * THE MAP ITSELF, AWAY FROM ANY CYCLE. `psi`, `eta_c_at`, `eta_t_at` and `solve_n` are swept
    standalone over a written-down grid, because the Rust carries only rung 32's FIVE fields
    where the Python's dataclass carries ten (`l`, `phi_surge`, `vsv`, `capacity` belong to
    rungs 34/36/53/54 and default to zero). That subset is the port's one deliberate structural
    difference in this slice, so it is measured on the arithmetic rather than argued from
    algebra — an omitted `- 0.0 * x` is exact, but "algebraically inert" and "arithmetically
    inert" have already come apart three times in this port (see `nox`'s cross-plane note).

  * `solve_n`'s RESIDUAL-EVALUATION COUNT, per call. The bracket is a fixed `[0.1, 2.0]` and the
    break `hi - lo <= 1e-14` is ABSOLUTE, so the count cannot depend on the data — it is a
    naming key exactly as slice I's 47 is, and it is counted by overriding `psi` in a delegating
    subclass (one `psi` call per residual) rather than by copying the loop.

  * THE OUTER SECANT's PASS COUNT, per cell. Slice I measured a discrete instability — the inner
    fixed point's pass count flips 7 <-> 200 between CPython and PyPy on the equilibrium gas —
    and rung 32 runs that loop once per outer pass. Whether the flip REACHES the outer count is
    exactly the thing a value gate cannot see, so it is dumped as a count.

  * THE REDUCE, AS A PER-CELL BIT FLAG. A flat map must give rung 31 back bit-for-bit, but only
    ON THE CHOKED BRANCH: rung 32 predates rung 33 and does not dispatch, so below the unchoke
    boundary the two matchers are solving DIFFERENT problems and the reduce is not merely
    inexact, it is not claimed. The flag and the branch are dumped side by side so the
    condition is in the data instead of in a comment.

Regenerate with:
    py -3                     rust/oracle/dump_map.py rust/oracle/map_cpython.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_map.py rust/oracle/map_pypy.tsv
"""
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.engine import (
    ComponentMap, FlightCondition, MapMatcher, OffDesignMatcher, build_turbojet,
)
from turbojet.gas import Gas

T0 = time.time()
ROWS = []


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def bits(v):
    return struct.unpack("<Q", struct.pack("<d", float(v)))[0]


FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)


def cpg_gas():
    """The SELF-CONSISTENT CPG dual gas: R_t = (g-1)/g*cp_t EXACTLY — slice I's helper, kept
    identical so a rung-32 number can be compared against a rung-31 one on the SAME hardware."""
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


GASES = [("cpg", cpg_gas), ("tpg", Gas.thermally_perfect), ("eq", Gas.reacting_equilibrium)]

SHAPES = [("flat", ComponentMap.flat()),
          ("flow", ComponentMap.flow_dominated()),
          ("press", ComponentMap.pressure_dominated()),
          ("tilt", ComponentMap.tilted())]

# TWO MORE SHAPES, STANDALONE ONLY — they run no cycle solve, and they exist because the four
# above do not cover the coefficients rung 32's OWN Python gates use. Gate 5 builds
# `ComponentMap(a=0.25, b=0.05, sigma=0.3, a_t=0.5)`, and no shape above has `a_t` past 0.02;
# gate 6 sweeps `sigma` up to 1.0, and no shape above has `sigma` past 0.6. Without these the
# "50 evaluations, zero spread" claim and `eta_t_at`'s curvature would be pinned only on a band
# narrower than the gates that rely on them.
MAP_ONLY = [("gate5", ComponentMap(a=0.25, b=0.05, sigma=0.3, a_t=0.5)),
            ("sig1", ComponentMap(sigma=1.0))]
ALL_SHAPES = SHAPES + MAP_ONLY

# THE GRID, WRITTEN DOWN (slice I's lesson: a count without its grid is not a measurement).
# The equilibrium gas gets a NARROWER one — and that is a cost decision, stated rather than
# hidden: an equilibrium working gas is re-frozen inside every inner pass, the inner loop runs
# its full 200-pass cap there, and rung 32 multiplies that by the outer secant. The Python's own
# rung-32 suite makes the same call (`_fast_matchers` runs gates 3-7 thermally-perfect).
#
# `Tt4 = 500/600` are in the grid for ONE reason: that is where rung 31 dispatches to rung 33's
# subsonic branch and rung 32 does not, so it is the only place the reduce's CONDITION is
# exercised rather than assumed. A grid that stopped at 650 would report a clean 100 % reduce
# and would never have touched the half of the claim that is interesting.
M0S = [0.3, 0.85, 1.6]
TT4S = [500.0, 600.0, 650.0, 900.0, 1100.0, 1500.0]
EQ_M0S = [0.85]
EQ_TT4S = [600.0, 900.0, 1100.0, 1500.0]
EQ_SHAPES = ["flat", "flow"]

ABORT = {
    "": 0.0,
    "SUB-IDLE": 1.0,
    "efficiency cascade": 2.0,
    "inverse: root not bracketed": 3.0,
    "equilibrium Newton": 4.0,
    "off-design burner f did not converge": 5.0,
    "nozzle back-pressure": 6.0,
    "map match did not converge": 7.0,
    "map match unphysical": 8.0,
    "speed-line bracket fails": 9.0,
}


def abort_code(msg):
    for tag, code in ABORT.items():
        if tag and tag in msg:
            return code
    raise AssertionError(f"UNCLASSIFIED abort, add it to ABORT: {msg[:120]}")


# ==============================================================================================
# 0. THE COUNTERS — all three by DELEGATING overrides, none by a copy of a loop
# ==============================================================================================
class CountingMap(ComponentMap):
    """`ComponentMap` that counts `psi` calls. `solve_n`'s residual `g` calls `psi` exactly once,
    so this counts residual evaluations INSIDE the shipped bisection rather than inside a copy."""

    n_psi = 0

    def psi(self, phi):
        CountingMap.n_psi += 1
        return super().psi(phi)


class Counting(MapMatcher):
    """`MapMatcher` with three counters and NO arithmetic of its own."""

    def __init__(self, *a, **k):
        super().__init__(*a, **k)
        self.n_outer = 0
        self.n_solve_turbine = 0

    def _operating_point(self, *a, **k):
        self.n_outer += 1
        return super()._operating_point(*a, **k)

    def _solve_turbine(self, *a, **k):
        self.n_solve_turbine += 1
        return super()._solve_turbine(*a, **k)


MATCHERS = {}
R31 = {}
for gname, gfac in GASES:
    design = build_turbojet(gfac(), PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    MATCHERS[gname] = Counting(design, FLIGHT, 1.0, comp_map=ComponentMap.flat())
    r31_design = build_turbojet(gfac(), PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    R31[gname] = OffDesignMatcher(r31_design, FLIGHT, 1.0)

# ==============================================================================================
# 1. THE DESIGN REFERENCES the map coordinates are normalised on
#
#    Four scalars captured in the constructor. They are the ONLY state rung 32 adds to slice I's
#    hardware capture, and every corrected coordinate below divides by one of them — so a wrong
#    one moves `m`, `n`, `flowcoef` and `nu_t` together and would otherwise read as a solver
#    artefact rather than as a captured constant.
# ==============================================================================================
for gname, _ in GASES:
    m = MATCHERS[gname]
    put(f"ref/{gname}/Tt2_d", m.Tt2_d)
    put(f"ref/{gname}/mdot_corr_d", m.mdot_corr_d)
    put(f"ref/{gname}/tau_c_d", m.tau_c_d)
    put(f"ref/{gname}/Tt4_d", m.Tt4_d)

# ==============================================================================================
# 2. THE MAP, STANDALONE — the field subset measured on the arithmetic
#
#    `psi`, `eta_c_at` and `eta_t_at` over a grid that reaches well outside the swept operating
#    band, so the Rust's five-field spelling is pinned where the cycle never takes it. The
#    Python evaluates `1 - sigma*(phi-1)**2 - l*(phi-1)` and the Rust `1 - sigma*(phi-1)^2`;
#    both interpreters constant-fold `** 2` to a multiply, and `l` is 0.0 at every rung-32 call.
# ==============================================================================================
PHIS = [0.20, 0.55, 0.80, 0.95, 0.999, 1.0, 1.001, 1.05, 1.30, 1.90]
NS = [0.30, 0.60, 0.85, 0.98, 1.0, 1.02, 1.15, 1.60]
NU_TS = [0.40, 0.75, 0.95, 1.0, 1.05, 1.40]
for sname, cmap in ALL_SHAPES:
    for i, phi in enumerate(PHIS):
        put(f"map/{sname}/psi/{i}", cmap.psi(phi))
    for i, phi in enumerate(PHIS):
        for j, n in enumerate(NS):
            put(f"map/{sname}/eta_c/{i}/{j}", cmap.eta_c_at(0.88, phi, n))
    for i, nu in enumerate(NU_TS):
        put(f"map/{sname}/eta_t/{i}", cmap.eta_t_at(0.90, nu))

# ==============================================================================================
# 3. `solve_n` — the speed-line inversion, swept standalone, WITH its evaluation count
#
#    Swept on a synthetic (m, tau_c) grid rather than only at the operating points, because the
#    bisection's cost claim ("48 evaluations, zero spread") is a claim about the ABSOLUTE break
#    `hi - lo <= 1e-14` on a fixed bracket, and a sweep confined to the running line could not
#    distinguish it from a claim about the data.
# ==============================================================================================
SOLVE_M = [0.55, 0.75, 0.90, 1.0, 1.10, 1.25]
SOLVE_TAU = [1.35, 1.60, 1.90, 2.20, 2.55]
TAU_C_D = 2.2044318861866967          # a fixed reference, so the sweep is gas-independent
#    Every cell carries an `ok` flag and the value only when the bracket held. `sigma = 1.0`
#    makes `psi` go NEGATIVE well away from design, so whether `[0.1, 2.0]` still straddles the
#    root is a property of the coefficients — dumping it as a flag makes a bracket failure a
#    matched key instead of a dead script, and puts rung 32's own raise site in the data.
n_evals = set()
n_brk_fail = 0
for sname, cmap in ALL_SHAPES:
    cm = CountingMap(a=cmap.a, b=cmap.b, c=cmap.c, sigma=cmap.sigma, a_t=cmap.a_t)
    for i, mm_ in enumerate(SOLVE_M):
        for j, tc in enumerate(SOLVE_TAU):
            CountingMap.n_psi = 0
            try:
                n = cm.solve_n(mm_, tc, TAU_C_D)
            except AssertionError:
                put(f"solven/{sname}/{i}/{j}/ok", 0.0)
                n_brk_fail += 1
                continue
            n_evals.add(CountingMap.n_psi)
            put(f"solven/{sname}/{i}/{j}/ok", 1.0)
            put(f"solven/{sname}/{i}/{j}", n)
put("census/solve_n_evals_min", float(min(n_evals)))
put("census/solve_n_evals_max", float(max(n_evals)))
put("census/solve_n_eval_patterns", float(len(n_evals)))
put("census/solve_n_bracket_failures", float(n_brk_fail))

# ==============================================================================================
# 4. THE MATCHED GRID — every cell, on every shape, with both loop counts
# ==============================================================================================
n_cells = n_abort = 0
CELL_BITS = {}
CELL_VALS = {}
ABORTS_SEEN = {}
BRANCH31 = {}
for gname, _ in GASES:
    m = MATCHERS[gname]
    m0s = EQ_M0S if gname == "eq" else M0S
    tt4s = EQ_TT4S if gname == "eq" else TT4S
    shapes = [(s, c) for s, c in SHAPES if gname != "eq" or s in EQ_SHAPES]
    for sname, cmap in shapes:
        for M0 in m0s:
            flight = FlightCondition(T0=250.0, p0=FLIGHT.p0, M0=M0)
            for Tt4 in tt4s:
                tag = f"{gname}/{sname}/{M0:.2f}/{Tt4:.0f}"
                m.n_outer = 0
                m.n_solve_turbine = 0
                try:
                    od = m.match(flight, Tt4, comp_map=cmap)
                except AssertionError as e:
                    code = abort_code(str(e).split("\n")[0])
                    put(f"cell/{tag}/abort", code)
                    ABORTS_SEEN[code] = ABORTS_SEEN.get(code, 0) + 1
                    n_abort += 1
                    continue
                put(f"cell/{tag}/abort", ABORT[""])
                n_cells += 1
                # THE TWO COUNTS. `n_outer` is the outer secant's pass count (slice I's 7<->200
                # inner flip either reaches it or does not); `n_solve_turbine` is the total
                # inner cost, outer x inner, so a divergence anywhere shows as a COUNT.
                put(f"cell/{tag}/n_outer", float(m.n_outer))
                put(f"cell/{tag}/n_solve_turbine", float(m.n_solve_turbine))
                # `branch` is dumped even though rung 32 never sets it: it is ALWAYS "choked",
                # including below the unchoke boundary where `nozzle_choked` says otherwise.
                # That contradiction is rung 33's gate 7 second half and it is data here.
                put(f"cell/{tag}/branch", 0.0 if od.branch == "choked" else 1.0)
                put(f"cell/{tag}/nozzle_choked", 1.0 if od.nozzle_choked else 0.0)
                vals = (("eta_c", od.eta_c), ("eta_t", od.eta_t), ("n_corr", od.n_corr),
                        ("N_ratio", od.N_ratio), ("flowcoef", od.flowcoef), ("nu_t", od.nu_t),
                        ("pi_c", od.pi_c), ("tau_c", od.tau_c), ("tau_t", od.tau_t),
                        ("pi_t", od.pi_t), ("mdot_air", od.mdot_air),
                        ("mdot_ratio", od.mdot_ratio), ("thrust", od.thrust),
                        ("V0", od.V0), ("V9", od.V9), ("M9", od.M9),
                        ("T9", od.T9), ("p9", od.p9),
                        ("F_over_mdot", od.performance.specific_thrust),
                        ("tsfc", od.performance.tsfc),
                        ("eta_th", od.performance.eta_thermal),
                        ("eta_p", od.performance.eta_propulsive))
                for name, v in vals:
                    put(f"cell/{tag}/{name}", v)
                for st in ("2", "3", "4", "5", "9"):
                    s = od.stations[st]
                    put(f"cell/{tag}/s{st}/Tt", s.Tt)
                    put(f"cell/{tag}/s{st}/pt", s.pt)
                put(f"cell/{tag}/s4/far", od.stations["4"].far)
                CELL_BITS[tag] = {k: bits(v) for k, v in vals}
                CELL_VALS[tag] = {k: float(v) for k, v in vals}

put("census/matched", float(n_cells))
put("census/aborted", float(n_abort))
# The abort MIX, not just the total: an abort code is what tells rung 32's own two raise sites
# (codes 7-9) apart from rung 31's envelope edges, and § 5.6 (a) predicted rung 32's are dead.
for code in sorted(ABORT.values()):
    if code != 0.0:
        put(f"census/abort_code/{code:.0f}", float(ABORTS_SEEN.get(code, 0)))

# ==============================================================================================
# 5. THE REDUCE — flat map vs rung 31, PER CELL, CONDITIONED ON THE BRANCH
#
#    A flat map makes the outer secant inert on pass 1, so rung 32 must hand back rung 31's
#    numbers BIT-FOR-BIT — but only where rung 31 stays on its choked branch. Below the unchoke
#    boundary rung 31 dispatches to rung 33's subsonic solve and rung 32 does not, so the two
#    are answering different questions and the reduce is NOT claimed there. Both the flag and
#    rung 31's branch are dumped, so the Rust gate re-derives the condition from the data.
# ==============================================================================================
n_reduce_choked = n_reduce_eq = n_reduce_sub = n_reduce_sub_eq = 0
for gname, _ in GASES:
    r31 = R31[gname]
    m0s = EQ_M0S if gname == "eq" else M0S
    tt4s = EQ_TT4S if gname == "eq" else TT4S
    for M0 in m0s:
        flight = FlightCondition(T0=250.0, p0=FLIGHT.p0, M0=M0)
        for Tt4 in tt4s:
            tag = f"{gname}/flat/{M0:.2f}/{Tt4:.0f}"
            if tag not in CELL_BITS:
                continue
            try:
                od31 = r31.match(flight, Tt4)
            except AssertionError:
                put(f"red/{tag}/r31_ok", 0.0)
                continue
            put(f"red/{tag}/r31_ok", 1.0)
            put(f"red/{tag}/r31_branch", 0.0 if od31.branch == "choked" else 1.0)
            BRANCH31[tag] = od31.branch
            same = {}
            for name, v in (("pi_c", od31.pi_c), ("tau_c", od31.tau_c), ("tau_t", od31.tau_t),
                            ("pi_t", od31.pi_t), ("mdot_air", od31.mdot_air),
                            ("thrust", od31.thrust), ("V9", od31.V9), ("T9", od31.T9),
                            ("p9", od31.p9)):
                same[name] = 1.0 if bits(v) == CELL_BITS[tag][name] else 0.0
                put(f"red/{tag}/{name}_same", same[name])
            allsame = 1.0 if all(x == 1.0 for x in same.values()) else 0.0
            put(f"red/{tag}/all_same", allsame)
            if od31.branch == "choked":
                n_reduce_choked += 1
                n_reduce_eq += int(allsame)
            else:
                n_reduce_sub += 1
                n_reduce_sub_eq += int(allsame)

put("census/reduce_choked_cells", float(n_reduce_choked))
put("census/reduce_choked_bitequal", float(n_reduce_eq))
put("census/reduce_subsonic_cells", float(n_reduce_sub))
put("census/reduce_subsonic_bitequal", float(n_reduce_sub_eq))
# MEASURED on this run, then asserted — never the other way round (phase 4's five-for-five).
assert n_reduce_eq == n_reduce_choked, \
    f"the flat-map reduce is NOT bit-exact on the choked branch: {n_reduce_eq}/{n_reduce_choked}"
assert n_reduce_sub > 0, "the grid must contain subsonic cells, or the CONDITION is untested"
# AND THE OTHER HALF, which is the one that makes "on the choked branch" a claim rather than a
# qualifier: below the unchoke boundary the two matchers are solving different problems, so they
# must NOT agree. Without this the conditional form of the reduce would be untested — a cell
# where rung 33 happened to land back on rung 32's answer would read as support for it.
assert n_reduce_sub_eq < n_reduce_sub, \
    (f"every subsonic cell reduced bit-exactly too ({n_reduce_sub_eq}/{n_reduce_sub}) — then the "
     "branch CONDITION on the reduce has no evidence behind it and must not be claimed")

# ==============================================================================================
# 6. THE SHAPE SPREAD — rung 32's headline, and the currency it is NOT true in
#
#    RUNG 32's headline: `tau_c` (the compressor WORK) is choke-pinned and MAP-FREE, while
#    `pi_c` and `mdot` are NOT. The obvious way to say that in this port's usual currency would
#    be a count of DISTINCT BIT PATTERNS across the four map shapes — one for `tau_c`, several
#    for `pi_c`. THAT WAS MEASURED FIRST AND IT IS FALSE: `tau_c`'s bit pattern moves across
#    shapes in every one of the 32 non-equilibrium cells, exactly as `pi_c`'s does. The count is
#    a perfect non-discriminator here.
#
#    It is false for a reason worth writing down rather than working around. `tau_c` is map-free
#    STRUCTURALLY — no map coefficient appears in the shaft balance that sets it — but it is
#    reached through a fixed point whose OTHER variables (`eta_c`, hence `pi_c`, hence `pt4`,
#    hence `f`) do move with the map, and a converged iterate carries its history in the last
#    bits. So the claim is about MAGNITUDE and always was: Python's gate 4 bar is `1e-4`, not
#    zero. The relative spread across shapes is dumped per cell instead — a float, computed from
#    bit-equal inputs, so it stays a bit-exact quantity while measuring the right thing.
# ==============================================================================================
def spread_across_shapes(gas, M0, Tt4, quantity):
    """Relative peak-to-peak of one quantity across the four map shapes at one cell."""
    vals = []
    for sname, _ in SHAPES:
        t = f"{gas}/{sname}/{M0:.2f}/{Tt4:.0f}"
        if t in CELL_VALS:
            vals.append(CELL_VALS[t][quantity])
    if len(vals) < 2:
        return None, len(vals)
    lo, hi = min(vals), max(vals)
    return (hi - lo) / (0.5 * (hi + lo)), len(vals)


worst = {}
for gname, _ in GASES:
    if gname == "eq":
        continue
    for M0 in M0S:
        for Tt4 in TT4S:
            for q in ("tau_c", "pi_c", "mdot_air", "n_corr", "eta_c", "thrust"):
                s, k = spread_across_shapes(gname, M0, Tt4, q)
                if s is None:
                    continue
                put(f"shapes/{gname}/{M0:.2f}/{Tt4:.0f}/{q}_spread", s)
                put(f"shapes/{gname}/{M0:.2f}/{Tt4:.0f}/{q}_n", float(k))
                worst[q] = max(worst.get(q, 0.0), s)
for q in sorted(worst):
    put(f"census/worst_shape_spread/{q}", worst[q])
put("census/map_free_ratio", worst["pi_c"] / worst["tau_c"])
# MEASURED, then written — and as a RATIO, not as a direction. `tau_c < pi_c` would still pass
# with tau_c at 3.7e-2, i.e. with the map-freeness gone entirely; it names a direction where the
# measurement is a POINT. The ratio measures 1.03e4, so the bar below has ~10x headroom and a
# tenfold degradation of the pin fails it.
assert worst["pi_c"] / worst["tau_c"] > 1.0e3, \
    (f"rung 32: the WORK must be the map-free one by orders, not by a hair — "
     f"pi_c/tau_c spread ratio is {worst['pi_c'] / worst['tau_c']:.3e}")

print(f"[3] solve_n residual evaluations: {sorted(n_evals)}")
print(f"[4] cells: {n_cells} matched, {n_abort} aborted")
print(f"[5] reduce: {n_reduce_eq}/{n_reduce_choked} bit-equal on choked, "
      f"{n_reduce_sub} subsonic cells NOT claimed")
print("[6] worst relative spread across the four shapes: "
      + ", ".join(f"{q} {worst[q]:.3e}" for q in sorted(worst)))

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-5J component-map matcher oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, b, text in ROWS:
        fh.write(f"{key}\t{b}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
