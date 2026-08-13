"""THE ORACLE, phase 5 slice I — every rung-31/33 value the Rust must reproduce.

The first slice of phase 5, and the first anywhere in the port whose subject is a SOLVE OVER A
SOLVE: `OffDesignMatcher.match` runs a joint `(f, pt4)` fixed point whose every pass drives a
turbine bisection, and rung 33's branch wraps ANOTHER root find around all of it.

WHAT IS NEW HERE, and why each thing is dumped rather than asserted:

  * A MARCH THAT WALKS PAST FAILURES. `_match_subsonic` steps each bracket inward while
    catching `AssertionError` — the first code in the port that treats a raise as control flow.
    Which trials raise DECIDES the bracket and therefore the root, so the rejection counts and
    the bracket endpoints are dumped as values, not left implied by pi_t. The march LOOP is
    replicated here and in the Rust gate; what it drives is entirely shipped code, which is
    where the rejection set is actually decided.

  * A LOOP THAT DOES NOT CONVERGE. The joint fixed point exhausts its 200-pass cap on the
    production gas at the two hottest throttles and falls out with NO assert, so the answer is
    the 200th iterate of a fixed count. Every pass has to reproduce bit-for-bit; the per-cell
    `_solve_turbine` call count is dumped so a divergence shows up as a COUNT rather than as an
    unexplained last-bits drift.

  * A COUNT MEASURED THROUGH THE SHIPPED LOOP. `_solve_turbine`'s map evaluations per call are
    counted by OVERRIDING `_tau_t_of_pi_t` in a delegating subclass — the loop itself is
    untouched. A copy of the loop with a counter in it would gate the copy.

  * `choked_mfp` GETS ITS OWN GATE AT LAST. It was held out of phase 2 and again out of phase 4
    slice H on the ground that no rungs 1-30 test referenced it. Rung 31 is where it is used, so
    this is where it is pinned — including the `Tt ** 0.5` that is a libm `pow` and not a sqrt
    (pre-registered as P4).

  * THE ENVELOPE, AS DATA. 38 of the 126 cells abort. They are dumped as a per-cell abort CODE
    rather than skipped, so "the Rust matched fewer cells" is a gate failure instead of a
    silently shorter file.

Regenerate with:
    py -3                     rust/oracle/dump_offdesign.py rust/oracle/offdesign_cpython.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_offdesign.py rust/oracle/offdesign_pypy.tsv
"""
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.components import choked_mfp, ram_recovery
from turbojet.engine import FlightCondition, build_turbojet, OffDesignMatcher
from turbojet.gas import Gas

T0 = time.time()
ROWS = []


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)


def cpg_gas():
    """The SELF-CONSISTENT CPG dual gas: R_t = (g-1)/g*cp_t EXACTLY.

    NOT slice H's `CPG`, which rounds R_t to 285.9. Both rung-31 gate 2 and rung-33 gate 4
    compare the sonic-throat SOLVER against a closed form, and that identity holds only when
    the constants satisfy the perfect-gas relation exactly. Copying slice H's helper forward
    would break both gates for a reason that looks like a solver artefact.
    """
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


GASES = [("cpg", cpg_gas), ("tpg", Gas.thermally_perfect), ("eq", Gas.reacting_equilibrium)]

# THE GRID, WRITTEN DOWN — which is the point. The pre-registration quoted "930 low / 616 high
# raises" from a sweep whose grid was never recorded, so the number could not be reproduced and
# had to be re-measured (§ 5.4 (i)). A count without its grid is not a measurement.
M0S = [0.3, 0.5, 0.85, 1.2, 1.6, 2.0]
TT4S = [400.0, 500.0, 600.0, 650.0, 900.0, 1100.0, 1500.0]

# Abort codes. Numeric because the TSV carries floats, and CONTIGUOUS from 1 so that a Rust
# side that aborts for a different reason lands on a different number rather than on "nonzero".
ABORT = {
    "": 0.0,                                    # matched
    "SUB-IDLE": 1.0,
    "efficiency cascade": 2.0,
    "inverse: root not bracketed": 3.0,
    "equilibrium Newton": 4.0,
    "off-design burner f did not converge": 5.0,
    "nozzle back-pressure": 6.0,
}


def abort_code(msg):
    for tag, code in ABORT.items():
        if tag and tag in msg:
            return code
    raise AssertionError(f"UNCLASSIFIED abort, add it to ABORT: {msg[:120]}")


class Counting(OffDesignMatcher):
    """`OffDesignMatcher` with two counters and NO arithmetic of its own.

    Both overrides delegate immediately, so every number below is the shipped class's. This is
    how the map-evaluation count is observed inside the shipped bisection instead of inside a
    copy of it — the Rust does the same thing with a `Cell<u64>` in `tau_t_of_pi_t`.
    """

    def __init__(self, *a, **k):
        super().__init__(*a, **k)
        self.n_tau = 0
        self.n_solve_turbine = 0
        self.n_subsonic_op = 0

    def _tau_t_of_pi_t(self, *a, **k):
        self.n_tau += 1
        return super()._tau_t_of_pi_t(*a, **k)

    def _solve_turbine(self, *a, **k):
        self.n_solve_turbine += 1
        return super()._solve_turbine(*a, **k)

    def _subsonic_operating(self, *a, **k):
        self.n_subsonic_op += 1
        return super()._subsonic_operating(*a, **k)


MATCHERS = {}
for gname, gfac in GASES:
    design = build_turbojet(gfac(), PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    MATCHERS[gname] = Counting(design, FLIGHT, 1.0)

# ==============================================================================================
# 1. THE HARDWARE CAPTURE — A4/A8 and the design reference the whole rung hangs off
# ==============================================================================================
for gname, _ in GASES:
    m = MATCHERS[gname]
    put(f"hw/{gname}/A4", m.A4)
    put(f"hw/{gname}/A8", m.A8)
    put(f"hw/{gname}/f_design", m.f_design)
    put(f"hw/{gname}/pi_d_max", m.pi_d_max)
    put(f"hw/{gname}/pi_d_design", m.pi_d_design)
    for st in ("2", "3", "4", "5", "9"):
        s = m.ref.stations[st]
        put(f"hw/{gname}/ref{st}/Tt", s.Tt)
        put(f"hw/{gname}/ref{st}/pt", s.pt)
    put(f"hw/{gname}/ref/A4_over_A8", m.A4 / m.A8)

# ==============================================================================================
# 2. `choked_mfp` — rung 31's own component function, gated for the first time
#
#    Its whole load-bearing property is that MFP* depends on Tt and COMPOSITION only, with no
#    pressure in it at all. That is why the same value can serve as a fixed hardware constant
#    off design. Swept over Tt x far so the far dependence is exercised too: on the CPG gas it
#    must be exactly flat in far, on the reacting one it must not be — and the CONTRAST is what
#    makes the far axis a measurement rather than a column of repeats.
# ==============================================================================================
MFP_T = [400.0, 650.0, 900.0, 1262.0, 1500.0, 1800.0]
MFP_F = [0.0, 0.005, 0.0272, 0.045]
for gname, _ in GASES:
    gas = MATCHERS[gname].gas
    for i, Tt in enumerate(MFP_T):
        for j, far in enumerate(MFP_F):
            if gname == "eq":
                # An equilibrium gas answers only at the far its burner FROZE, so the sweep
                # over far is not available on it. Its single frozen value is dumped instead —
                # recorded here rather than silently skipped.
                continue
            put(f"mfp/{gname}/{i}/{j}", choked_mfp(gas, Tt, far))
    if gname == "eq":
        f = MATCHERS[gname].f_design
        for i, Tt in enumerate(MFP_T):
            put(f"mfp/{gname}/{i}/frozen", choked_mfp(gas, Tt, f))

# ==============================================================================================
# 3. THE MATCHED GRID — every cell, and for the 38 that abort, WHY
# ==============================================================================================
n_choked = n_subsonic = n_abort = 0
BRANCH_OF = {}
for gname, _ in GASES:
    m = MATCHERS[gname]
    for M0 in M0S:
        flight = FlightCondition(T0=250.0, p0=FLIGHT.p0, M0=M0)
        mtag = f"{M0:.2f}"
        for Tt4 in TT4S:
            tag = f"{gname}/{mtag}/{Tt4:.0f}"
            m.n_solve_turbine = 0
            try:
                od = m.match(flight, Tt4)
            except AssertionError as e:
                put(f"cell/{tag}/abort", abort_code(str(e).split("\n")[0]))
                n_abort += 1
                continue
            put(f"cell/{tag}/abort", ABORT[""])
            put(f"cell/{tag}/branch", 0.0 if od.branch == "choked" else 1.0)
            BRANCH_OF[tag] = od.branch
            # THE COUNT (P1): how many turbine solves the joint fixed point paid for. 1/7/7 on
            # CPG and 200 where the stopping rule is unmeetable — the table § 5.4 (b) measured.
            put(f"cell/{tag}/n_solve_turbine", float(m.n_solve_turbine))
            for name, v in (("pi_c", od.pi_c), ("tau_c", od.tau_c), ("tau_t", od.tau_t),
                            ("pi_t", od.pi_t), ("mdot_air", od.mdot_air),
                            ("mdot_ratio", od.mdot_ratio), ("thrust", od.thrust),
                            ("V0", od.V0), ("V9", od.V9), ("M9", od.M9),
                            ("T9", od.T9), ("p9", od.p9),
                            ("F_over_mdot", od.performance.specific_thrust),
                            ("tsfc", od.performance.tsfc),
                            ("eta_th", od.performance.eta_thermal),
                            ("eta_p", od.performance.eta_propulsive)):
                put(f"cell/{tag}/{name}", v)
            for st in ("2", "3", "4", "5", "9"):
                s = od.stations[st]
                put(f"cell/{tag}/s{st}/Tt", s.Tt)
                put(f"cell/{tag}/s{st}/pt", s.pt)
            put(f"cell/{tag}/s4/far", od.stations["4"].far)
            if od.branch == "choked":
                n_choked += 1
            else:
                n_subsonic += 1

put("census/matched_choked", float(n_choked))
put("census/matched_subsonic", float(n_subsonic))
put("census/aborted", float(n_abort))

# ==============================================================================================
# 4. THE TURBINE SOLVE'S MAP-EVALUATION COUNT (P1) — measured, then written
#
#    § 5.4 (c) predicted a fixed count with no spread. It is counted here per SOLVE (not per
#    match) by taking the difference across one call on a freshly-seeded matcher, and the bar
#    below is whatever this run measures — the standing rule after five typed count bars in
#    phase 4 came out wrong five times.
# ==============================================================================================
tau_per_solve = set()
for gname, _ in GASES:
    m = MATCHERS[gname]
    gas = m.gas
    for Tt4 in (1500.0, 1100.0, 900.0, 650.0):
        wg = m._working_gas(m.f_design, Tt4, m.pi_b * m.pi_c_design * 4.0e5)
        before = m.n_tau
        pi_t, tau_t, Tt5 = m._solve_turbine(wg, Tt4, m.f_design)
        tau_per_solve.add(m.n_tau - before)
        put(f"turb/{gname}/{Tt4:.0f}/pi_t", pi_t)
        put(f"turb/{gname}/{Tt4:.0f}/tau_t", tau_t)
        put(f"turb/{gname}/{Tt4:.0f}/Tt5", Tt5)
        put(f"turb/{gname}/{Tt4:.0f}/n_tau", float(m.n_tau - before))
assert len(tau_per_solve) == 1, f"the map-evaluation count SPREADS: {sorted(tau_per_solve)}"
put("census/tau_evals_per_solve", float(next(iter(tau_per_solve))))

# ==============================================================================================
# 5. THE BRACKET MARCH — the rejection sets and endpoints that DECIDE the subsonic root
#
#    The loop below is a replica of `_match_subsonic`'s two marches; the Rust gate carries the
#    same replica. What it drives — `_subsonic_operating` and everything under it — is shipped
#    code, and that is where the rejection set is decided. Run on every cell, not only the ones
#    that dispatch, so the arm covers the whole grid rather than the 14 subsonic outcomes.
# ==============================================================================================
n_lo_tot = n_hi_tot = 0
for gname, _ in GASES:
    m = MATCHERS[gname]
    for M0 in M0S:
        flight = FlightCondition(T0=250.0, p0=FLIGHT.p0, M0=M0)
        state0, _V0 = m._fs_engine.freestream(flight, m.mdot_air_design)
        Tt2, pt2 = state0.Tt, m.pi_d_max * ram_recovery(M0) * state0.pt
        for Tt4 in TT4S:
            tag = f"{gname}/{M0:.2f}/{Tt4:.0f}"

            def resid(pi_t):
                return m._subsonic_operating(flight, Tt4, Tt2, pt2, flight.p0, pi_t)["resid"]

            lo, rlo, n_lo = None, None, 0
            pt = 0.15
            while pt < 0.95:
                try:
                    rlo = resid(pt); lo = pt; break
                except AssertionError:
                    n_lo += 1
                    pt += 0.02
            hi, rhi, n_hi = None, None, 0
            pt = 0.9995
            while lo is not None and pt > lo:
                try:
                    rhi = resid(pt); hi = pt; break
                except AssertionError:
                    n_hi += 1
                    pt -= 0.02
            n_lo_tot += n_lo
            n_hi_tot += n_hi
            put(f"brk/{tag}/n_lo", float(n_lo))
            put(f"brk/{tag}/n_hi", float(n_hi))
            put(f"brk/{tag}/found_lo", 1.0 if lo is not None else 0.0)
            put(f"brk/{tag}/found_hi", 1.0 if hi is not None else 0.0)
            if lo is not None:
                put(f"brk/{tag}/lo", lo)
                # `rlo` is the residual at the INITIAL low endpoint and the bisection never
                # refreshes it when the root falls in the low half, so it rides through the
                # whole solve. Dumped for that reason: it is state, not a byproduct.
                put(f"brk/{tag}/rlo", rlo)
            if hi is not None:
                put(f"brk/{tag}/hi", hi)
                put(f"brk/{tag}/rhi", rhi)
                put(f"brk/{tag}/straddles", 1.0 if rlo * rhi < 0.0 else 0.0)

put("census/march_reject_lo", float(n_lo_tot))
put("census/march_reject_hi", float(n_hi_tot))

# ==============================================================================================
# 6. DISTINCT-VALUE COUNTS — measured first, then asserted (phase 4's five-for-five lesson)
# ==============================================================================================
def distinct_by_branch(gas, branch, suffix):
    """Distinct BIT PATTERNS of one quantity over one gas's cells on one branch.

    Split per (gas, branch) rather than aggregated, because the aggregate hides exactly the
    thing worth counting — and because phase 4's five wrong count bars were all aggregates.
    """
    return {bits for k, bits, _ in ROWS
            if k.startswith(f"cell/{gas}/") and k.endswith(suffix)
            and BRANCH_OF.get(k[len("cell/"):-len(suffix)]) == branch}


# THE TWO RUNGS, STATED AS COUNTS OVER BIT PATTERNS — which is a strictly stronger claim than
# either suite's `< 1e-9`, and the only form in which "constant" and "varies" are the same kind
# of statement.
#
#   RUNG 31: (★) is pure GEOMETRY, so on a calorically-perfect gas the choked tau_t is
#            machine-constant along the whole throttle AND flight-Mach sweep -> ONE pattern.
#   RUNG 33: the subsonic coupling runs through pi_c (structural), so it SURVIVES CPG ->
#            every subsonic cell its own value. The INVERSION, in the same currency.
#   The reacting gas collapses NEITHER, which is what makes the CPG collapse a measurement
#   about the pin rather than about the sweep being too narrow to resolve anything.
for gas, _ in GASES:
    for branch in ("choked", "subsonic"):
        for q in ("/tau_t", "/pi_t", "/pi_c"):
            n = len(distinct_by_branch(gas, branch, q))
            put(f"roots/{gas}/{branch}{q}_distinct", float(n))

CPG_CHOKED_TAU = len(distinct_by_branch("cpg", "choked", "/tau_t"))
CPG_SUB_TAU = len(distinct_by_branch("cpg", "subsonic", "/tau_t"))
EQ_CHOKED_TAU = len(distinct_by_branch("eq", "choked", "/tau_t"))
N_CPG_CHOKED = sum(1 for t, b in BRANCH_OF.items() if b == "choked" and t.startswith("cpg/"))
N_EQ_CHOKED = sum(1 for t, b in BRANCH_OF.items() if b == "choked" and t.startswith("eq/"))
# Bars MEASURED on this run, then written — never guessed while the gate was being typed.
assert CPG_CHOKED_TAU == 1, f"rung 31: CPG choked tau_t took {CPG_CHOKED_TAU} values, not 1"
assert CPG_SUB_TAU > 1, "rung 33: CPG subsonic tau_t must VARY (the inversion)"
assert EQ_CHOKED_TAU == N_EQ_CHOKED, "the reacting gas must not collapse anywhere"

print(f"[3] cells: {n_choked} choked, {n_subsonic} subsonic, {n_abort} aborted")
print(f"[4] map evaluations per turbine solve: {sorted(tau_per_solve)}")
print(f"[5] march rejections: {n_lo_tot} low, {n_hi_tot} high")
print(f"[6] tau_t patterns — cpg choked {CPG_CHOKED_TAU}/{N_CPG_CHOKED} (rung 31), "
      f"cpg subsonic {CPG_SUB_TAU} (rung 33), eq choked {EQ_CHOKED_TAU}/{N_EQ_CHOKED}")

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-5I off-design matcher oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
