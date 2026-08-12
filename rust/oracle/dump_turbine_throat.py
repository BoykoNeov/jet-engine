"""THE ORACLE, phase 4 slice H — every rung-29/30 value the Rust must reproduce.

The tenth and last of phase 4's family. Rungs 29 (the shifting turbine) and 30 (the choked
convergent nozzle) share no code with each other or with slices F/G, which is exactly why they are
one slice: neither depends on the other, so they could go anywhere, and putting them together
keeps the dependency-ordered slices before them clean.

WHAT IS NEW HERE:

  * A TWO-LEVEL SOLVE. `_work_limited_expand` bisects p5 on the OUTSIDE and T5 at constant entropy
    on the INSIDE, so every outer step pays a full inner bisection. It is the only nested root-find
    in the port so far, and the outer bracket's stopping rule (1e-12*p) is a FOURTH tolerance
    beside the three slice F catalogued.

  * TWO CODE PATHS ONTO ONE PHYSICAL CONDITION. `_sonic_throat` takes a CLOSED FORM on a CPG gas
    and a BISECTION otherwise. Rung 30's gate 2a compares them — and it runs on a CPG gas, so
    without the explicit `_sonic_throat_bisect` entry point it would compare the closed form
    against itself. Both are dumped, on the same gas, so the agreement is data rather than an
    assertion about it.

  * A `** 0.5` THAT IS *NOT* A sqrt. `_sonic_throat` spells V* as `(...) ** 0.5`, a libm pow call
    that differs from sqrt about one point in 670 — the trap phase 2 was caught by. Slice F's
    rung-26 clock spells `math.sqrt(J)`, which really is the sqrt instruction. Both live in phase
    4, and `throat/` is what would localise a port that applied either rule by habit.

  * A BOUND WHOSE HEADLINE IS A COMPARISON OF TWO CURRENCIES. Rung 29's finding is RATIO != ENERGY:
    the frozen station-5 pool is super-equilibrium by a large RATIO, but what a shifting turbine
    can EXPLOIT is the radical INVENTORY, which is far smaller. Both are dumped so the inversion is
    readable off the TSV rather than off prose.

Regenerate with:
    py -3                     rust/oracle/dump_turbine_throat.py rust/oracle/tt_cpython.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_turbine_throat.py rust/oracle/tt_pypy.tsv
"""
import math
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.components import Nozzle, _sonic_throat, _sonic_throat_bisect
from turbojet.engine import FlightCondition, build_turbojet
from turbojet.gas import Gas, FlowState, _equilibrium_composition, _work_limited_expand

T0 = time.time()
ROWS = []


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
              eta_t=0.90, eta_m=0.99, pi_n=0.98)
PI_C = 10.0
DPS = [("cold", 1300.0), ("dp", 1500.0), ("warm", 1800.0), ("hot", 2200.0), ("vhot", 2300.0)]

# The CPG dual gas the rung-30 component gates use — where `_sonic_throat` takes its CLOSED FORM.
CPG = Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=1.3, cp_t=1239.0, R_t=285.9, hPR=42.8e6)

STATES = {}
for tag, tt4 in DPS:
    g = Gas.reacting_equilibrium()
    r = build_turbojet(g, PI_C, tt4, FLIGHT.p0, **LOSSES).run(FLIGHT, 1.0)
    st3, st4, st5, st9 = (r.stations["3"], r.stations["4"], r.stations["5"], r.stations["9"])
    # The shaft-set enthalpy drop, exactly as `Engine.run` hands it to the Turbine.
    delta_h = (g.h_c(st3.Tt) - g.h_c(r.stations["2"].Tt)) / (0.99 * (1.0 + st4.far))
    STATES[tag] = dict(gas=g, far=st4.far, Tt4=st4.Tt, pt4=st4.pt, Tt5=st5.Tt, pt5=st5.pt,
                       Tt9=st9.Tt, pt9=st9.pt, p9=r.p9, delta_h=delta_h)

# ==============================================================================================
# 1. RUNG 29 — the shifting-turbine bracket, and BOTH currencies of the finding
# ==============================================================================================
earned = 0
for tag, _ in DPS:
    s = STATES[tag]
    st = s["gas"].shifting_turbine(s["far"], s["Tt4"], s["pt4"], s["delta_h"])
    put(f"r29/{tag}/T5_frozen", st.T5_frozen)
    put(f"r29/{tag}/p5_frozen", st.p5_frozen)
    put(f"r29/{tag}/T5_shifting", st.T5_shifting)
    put(f"r29/{tag}/p5_shifting", st.p5_shifting)
    put(f"r29/{tag}/delta_h", st.delta_h)
    put(f"r29/{tag}/dT5", st.dT5)
    put(f"r29/{tag}/dT5_fraction", st.dT5_fraction)
    put(f"r29/{tag}/dp5_fraction", st.dp5_fraction)
    put(f"r29/{tag}/super_eq_ratio_max", st.super_eq_ratio_max)
    put(f"r29/{tag}/radical_inventory", st.radical_inventory)
    if st.frozen_turbine_earned:
        earned += 1

# LIVE discrete key: how many design points on the ladder call the frozen turbine EARNED. It MOVES
# with Tt4 — that motion IS rung 29's verdict, and no tolerance on dT5 expresses "the bound was
# called defensible here and not there".
put("census/frozen_turbine_earned", float(earned))

# The FROZEN branch of the solver, which production never takes (`shifting_turbine` delegates its
# frozen bound to the closed form). Dumped so the solver path itself is pinned: without this the
# `shifting=False` arm would be dead code in the port with nothing to compare it against.
for tag in ("dp", "hot"):
    s = STATES[tag]
    ce = _equilibrium_composition(s["far"], s["Tt4"], s["pt4"])
    from turbojet.gas import _mix_mass_per_air
    m = _mix_mass_per_air(ce)
    T5f, p5f, _ = _work_limited_expand(ce, s["far"], s["Tt4"], s["pt4"], s["delta_h"] * m, False)
    put(f"r29solve/{tag}/T5_frozen_solved", T5f)
    put(f"r29solve/{tag}/p5_frozen_solved", p5f)

# ==============================================================================================
# 2. RUNG 30 — the sonic throat, on BOTH code paths
#
#    On the CPG gas `_sonic_throat` takes the closed form; `_sonic_throat_bisect` is called
#    EXPLICITLY on the same gas so the two paths are compared rather than one being compared with
#    itself. That is gate 2a's whole content, and it only works because the split exists.
# ==============================================================================================
for i, Tt9 in enumerate([900.0, 1100.0, 1262.0, 1500.0, 1800.0, 2000.0]):
    for j, pt9 in enumerate([1.2e5, 3.4e5, 7.3e5, 1.9e6]):
        Ts, ps, Vs = _sonic_throat(CPG, Tt9, pt9, 0.0)
        put(f"throat/cpg/{i}/{j}/Tstar", Ts)
        put(f"throat/cpg/{i}/{j}/pstar", ps)
        put(f"throat/cpg/{i}/{j}/Vstar", Vs)
        # the SEARCHED root on the same gas — gate 2a's second path
        Tb = _sonic_throat_bisect(CPG, Tt9, 0.0, CPG.h_t(Tt9, 0.0), CPG.R_t_at(0.0))
        put(f"throat/bisect/{i}/{j}/Tstar", Tb)
        put(f"throat/bisect/{i}/{j}/gap", Tb - Ts)

# The REACTING gas takes the bisection branch — the real design-point path.
for tag, _ in DPS:
    s = STATES[tag]
    Ts, ps, Vs = _sonic_throat(s["gas"], s["Tt9"], s["pt9"], s["far"])
    put(f"throat/react/{tag}/Tstar", Ts)
    put(f"throat/react/{tag}/pstar", ps)
    put(f"throat/react/{tag}/Vstar", Vs)
    put(f"throat/react/{tag}/T_ratio", Ts / s["Tt9"])
    put(f"throat/react/{tag}/p_ratio", ps / s["pt9"])

# ==============================================================================================
# 3. RUNG 30 — the convergent NOZZLE, both branches: choked and subcritical
#
#    The subcritical branch is the REDUCE: a convergent nozzle below the critical ratio must be
#    bit-for-bit the shipped specified-exit-pressure nozzle at the same condition.
# ==============================================================================================
choked = 0
for tag, _ in DPS:
    s = STATES[tag]
    st5 = FlowState(Tt=s["Tt5"], pt=s["pt5"], mdot=1.0, far=s["far"])
    # The ambient ladder is set as a FRACTION of the local pt9, not as fixed pressures. A fixed
    # ladder cannot straddle the critical ratio at every design point — pt9 moves with Tt4, so
    # 3.0e5 Pa is subcritical at one point and ABOVE pt9 (an impossible expansion) at another,
    # which is what a first draft did. Fractions guarantee both branches at every point, which is
    # what makes the choked census below able to move for the right reason.
    _pt9 = 0.98 * s["pt5"]
    for ptag, p0 in (("design", FLIGHT.p0), ("sub", 0.80 * _pt9), ("deep", 0.05 * _pt9)):
        conv = Nozzle(p_ambient=p0, pi_n=0.98, convergent=True)
        ex = conv.apply(st5, s["gas"])
        put(f"nozzle/{tag}/{ptag}/M9", ex.M9)
        put(f"nozzle/{tag}/{ptag}/T9", ex.T9)
        put(f"nozzle/{tag}/{ptag}/V9", ex.V9)
        put(f"nozzle/{tag}/{ptag}/p9", ex.p9)
        if ex.M9 > 0.999999999:
            choked += 1
        else:
            # The REDUCE: unchoked, it must be the default nozzle at the same back-pressure.
            plain = Nozzle(p_ambient=p0, pi_n=0.98, p_exit=p0)
            ref = plain.apply(st5, s["gas"])
            assert ex.M9 == ref.M9 and ex.T9 == ref.T9 and ex.V9 == ref.V9 and ex.p9 == ref.p9, \
                f"the subcritical convergent branch is NOT the default nozzle at {tag}/{ptag}"
            put(f"nozzle/{tag}/{ptag}/ref_V9", ref.V9)

# LIVE discrete key: how many (design point x ambient) cells CHOKE. It has to move with the
# back-pressure or the branch would be untested on one side.
put("census/choked_cells", float(choked))

# ==============================================================================================
# 4. DISTINCT-VALUE COUNTS — measured first, then asserted (slice F's lesson, twice learned)
# ==============================================================================================
# THE SONIC-THROAT ROOT IS PRESSURE-INDEPENDENT, so the 6 x 4 grid holds SIX roots, not 24.
# `h_t(Tt9) - h_t(T*) = 1/2 gamma_t(T*) R T*` contains no pressure at all — pt9 enters only
# through `p* = pt9 * pr_t(T*)/pr_t(Tt9)`. That is the same property rung 31's `choked_mfp` is
# built on ("MFP* is a function of Tt and composition ALONE"), showing up here one rung early.
#
# A first draft asserted ">= 24 distinct sonic-throat roots" and failed at 17. That is the THIRD
# guessed count bar in phase 4 to be wrong, and the third time the shortfall was the physics
# rather than a defect (slice F's killM arm, slice F's killT ratio collision, this). The rule
# keeps failing in the same place: bars invented while WRITING a gate instead of measured before
# it. Each arm is now counted separately, and `p*` is counted too — it DOES vary with pt9, so the
# contrast between the two counts is what makes the invariance a measurement.
for arm, want in (("cpg", 6), ("bisect", 6), ("react", 5)):
    vals = {bits for key, bits, _ in ROWS
            if key.startswith(f"throat/{arm}/") and key.endswith("Tstar")}
    assert len(vals) == want, f"throat/{arm} Tstar: {len(vals)} distinct, expected {want}"
    put(f"roots/throat_{arm}_tstar_distinct", float(len(vals)))
# `p*` is DELIBERATELY NOT COUNTED, and the reason is the same one that moved the clock ladder off
# round numbers in slice F. On a CPG gas `T*/Tt9` is a constant, so `p* = pt9 * const` and the
# structural count would be 4 — but the constant is reached through `pr_t(T*)/pr_t(Tt9)`, which
# differs in the last bits from one Tt9 to the next, so the measured count is 19: five accidental
# collisions out of 24. Pinning 19 would pin a floating-point coincidence that need not survive a
# change of interpreter. The VALUES are gated at bit-equality regardless, which is the real claim;
# a count here would add nothing but fragility.

t5_roots = {bits for key, bits, _ in ROWS if key.startswith("r29/") and "/T5_" in key}
assert len(t5_roots) >= 9, f"only {len(t5_roots)} distinct turbine exit roots"
put("roots/t5_distinct", float(len(t5_roots)))
print(f"[4] throat Tstar 6/6/5 (pressure-INDEPENDENT), pstar 24 (it varies); "
      f"turbine exits {len(t5_roots)}, choked cells {choked}, earned {earned}")

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-4H turbine/throat oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
