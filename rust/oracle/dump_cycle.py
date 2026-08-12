"""THE ORACLE, phase 2 — dump every design-point CYCLE value the Rust port must reproduce.

The twin of `dump_gas.py`, one layer up. Where that one probed gas properties at a grid of
(T, far), this one runs the WHOLE design cycle — freestream, five components, shaft balance,
scoring — over the gas ladder and the loss configurations rungs 1-6 actually exercise, and
dumps every station total and every performance number.

Single-use by design (docs/plans/todo-rust-port.md): it validates the Rust and is deleted at
phase 8. It imports the production `build_turbojet` on purpose — it is not an API consumer,
it is a reference dump.

WHY THIS EXISTS RATHER THAN JUST PORTING THE TESTS. The rungs 1-6 suites check the cycle to
~0.1 % against published tables, because that is what a textbook anchor can carry. That
tolerance is three orders of magnitude looser than the port's real question, which phase 1
answered as "the arithmetic is PyPy's; the residual risk is SOLVER STOPPING RULES"
(§ 4.1). Phase 2 introduces exactly two new solvers -- the burner's `f = g(f)` fixed point
and rung 6's bisection on the absolute-enthalpy balance -- so the phase gate has to measure
at the bit, not at 0.1 %.

Output is TSV, one row per value:  key <TAB> u64-bits <TAB> repr

Run under BOTH interpreters. The project already ships on two (the gate runs PyPy, the
fingerprint goldens are CPython), so whatever PyPy and CPython disagree by is a deviation the
project ALREADY tolerates -- that gap is the principled tolerance floor, not a number picked
out of the air.

    C:\\Python314\\python.exe rust/oracle/dump_cycle.py rust/oracle/cycle_cpython.tsv
    .venv\\Scripts\\python.exe  rust/oracle/dump_cycle.py rust/oracle/cycle_pypy.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.components import ram_recovery
from turbojet.engine import FlightCondition, build_turbojet
from turbojet.gas import Gas

ROWS = []


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


# --- the flight conditions the rungs 1-6 suites use ------------------------------------
FLIGHT_R1 = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)     # rung-1 design point
FLIGHT_MATT = FlightCondition(T0=216.7, p0=50_000.0, M0=2.0)    # Mattingly Ex 7.1
FLIGHT_FB = FlightCondition(T0=216.7, p0=18_750.0, M0=2.0)      # rungs 5/6 lossy supersonic

# Mattingly Ex 7.1 losses, shared by the isentropic and polytropic spellings.
MATT_COMMON = dict(pi_d=0.95 * ram_recovery(2.0), eta_b=0.98, pi_b=0.94,
                   eta_m=0.99, pi_n=0.96, p_exit=50_000.0 / 0.5)
# The rungs 5/6 design point (docs: test_forkb / test_rung6 _DESIGN).
FB_DESIGN = dict(pi_c=10.0, p_ambient=18_750.0, pi_d=0.95, eta_c=0.90, eta_b=0.98,
                 pi_b=0.95, eta_t=0.90, eta_m=0.99, pi_n=0.97)
# The rung-2 "losses on" configuration.
R1_LOSSY = dict(pi_d=0.95, eta_c=0.88, eta_b=0.99, pi_b=0.95, eta_t=0.90,
                eta_m=0.99, pi_n=0.98)


def flat(cp, R):
    """A constant-cp polynomial (A_low == A_high == cp/R): a TPG section whose cp(T) happens
    to be flat. Rung 3's gate 3 uses it to drive the integral path to a known answer."""
    return ((cp / R, 0.0, 0.0, 0.0, 0.0), (cp / R, 0.0, 0.0, 0.0, 0.0))


def dump_case(tag, gas, flight, mdot, pi_c, Tt4, p_ambient, **kw):
    """Run one design point and record every number it produces."""
    result = build_turbojet(gas, pi_c=pi_c, Tt4=Tt4, p_ambient=p_ambient, **kw).run(flight, mdot)
    for label, st in result.stations.items():
        put(f"{tag}/st{label}/Tt", st.Tt)
        put(f"{tag}/st{label}/pt", st.pt)
        put(f"{tag}/st{label}/mdot", st.mdot)
        put(f"{tag}/st{label}/far", st.far)
    put(f"{tag}/V0", result.V0)
    put(f"{tag}/V9", result.V9)
    put(f"{tag}/M9", result.M9)
    put(f"{tag}/T9", result.T9)
    put(f"{tag}/p9", result.p9)
    p = result.performance
    put(f"{tag}/F", p.specific_thrust)
    put(f"{tag}/tsfc", p.tsfc)
    put(f"{tag}/eta_brayton", p.eta_brayton)
    put(f"{tag}/eta_thermal", p.eta_thermal)
    put(f"{tag}/eta_propulsive", p.eta_propulsive)
    put(f"{tag}/eta_overall", p.eta_overall)


# --- ram_recovery: all three branches, including the M0 = 5 join ------------------------
for m0 in (0.0, 0.5, 0.85, 1.0, 1.5, 2.0, 3.0, 5.0, 5.0001, 6.0, 8.0):
    put(f"ram/{m0!r}", ram_recovery(m0))

# --- RUNGS 1-2: the calorically-perfect single gas ---------------------------------------
dump_case("r1_ideal", Gas(), FLIGHT_R1, 1.0, 10.0, 1500.0, 50_000.0)
dump_case("r2_lossy", Gas(), FLIGHT_R1, 1.0, 10.0, 1500.0, 50_000.0, **R1_LOSSY)
# The unified() collapse: a genuinely DUAL gas flattened back onto the cold section. The
# rung-2 gate-1 lever, and the only place the port's struct-update `unified` is exercised.
dump_case("r2_unified", Gas(gamma_t=1.3, cp_t=1239.0, R_t=285.9).unified(),
          FLIGHT_R1, 1.0, 10.0, 1500.0, 50_000.0)

# --- RUNG 2: the Mattingly dual-gas anchor, both efficiency spellings ---------------------
MATT_GAS = dict(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=1.3, cp_t=1239.0, R_t=285.9,
                hPR=42.8e6)
dump_case("r2_matt_iso", Gas(**MATT_GAS), FLIGHT_MATT, 1.0, 10.0, 1800.0, 50_000.0,
          eta_c=0.8641, eta_t=0.9099, **MATT_COMMON)
# RUNG 2b: the SAME case with the polytropic knob fed directly. The two must agree to ~1e-9
# by an algebraic identity, so dumping both makes that identity checkable in Rust as well.
dump_case("r2b_matt_poly", Gas(**MATT_GAS), FLIGHT_MATT, 1.0, 10.0, 1800.0, 50_000.0,
          e_c=0.9, e_t=0.9, **MATT_COMMON)

# --- RUNG 3: the thermally-perfect gas, and the flat-cp integral path ---------------------
dump_case("r3_ideal", Gas.thermally_perfect(), FLIGHT_R1, 1.0, 10.0, 1500.0, 50_000.0)
dump_case("r3_lossy", Gas.thermally_perfect(), FLIGHT_R1, 1.0, 10.0, 1500.0, 50_000.0,
          **R1_LOSSY)
dump_case("r3_flat", Gas(R_c=286.9, R_t=285.9, hPR=42.8e6,
                         cp_c_coeffs=flat(1004.0, 286.9), cp_t_coeffs=flat(1239.0, 285.9)),
          FLIGHT_MATT, 1.0, 10.0, 1800.0, 50_000.0,
          eta_c=0.8641, eta_t=0.9099, **MATT_COMMON)

# --- RUNG 4: the reacting gas — the burner's fixed point becomes live ---------------------
dump_case("r4_ideal", Gas.reacting(), FLIGHT_R1, 1.0, 10.0, 1500.0, 50_000.0)
dump_case("r4_cold", Gas.reacting(), FLIGHT_R1, 1.0, 10.0, 1400.0, 50_000.0)
dump_case("r4_hot", Gas.reacting(), FLIGHT_R1, 1.0, 10.0, 1700.0, 50_000.0)
dump_case("r4_forkA_fb", Gas.reacting(hPR=42.8e6), FLIGHT_FB, 50.0, Tt4=1800.0, **FB_DESIGN)

# --- RUNG 5: Fork B — the derived heat release -------------------------------------------
dump_case("r5_forkb", Gas.reacting_forkb(), FLIGHT_FB, 50.0, Tt4=1800.0, **FB_DESIGN)
# A LOWER-LHV fuel: the one case where the calibration input is off its default, so the
# derived-heat-release path is proven live rather than merely consistent.
dump_case("r5_lean_fuel", Gas.reacting_forkb(hf_fuel_molar=-50_000.0), FLIGHT_FB, 50.0,
          Tt4=1800.0, **FB_DESIGN)

# --- RUNG 6: chemical equilibrium — the bisection burner ----------------------------------
# The design point, plus the two cold-Tt4 points GATE 1 uses for the reduce-to-rung-5 seam.
# Every one of these runs ~33 bisection steps, each a full 8-species damped Newton, so this
# is where a stopping-rule difference between the two languages would surface if anywhere.
dump_case("r6_design", Gas.reacting_equilibrium(), FLIGHT_FB, 50.0, Tt4=1800.0, **FB_DESIGN)
dump_case("r6_cold1000", Gas.reacting_equilibrium(), FLIGHT_FB, 50.0, Tt4=1000.0, **FB_DESIGN)
dump_case("r6_cold1400", Gas.reacting_equilibrium(), FLIGHT_FB, 50.0, Tt4=1400.0, **FB_DESIGN)
# The Fork-B twins of those two cold points, so the SEAM ITSELF (fE - fB) is an oracle value
# and not merely a pair of independently-checked ones.
dump_case("r5_cold1000", Gas.reacting_forkb(), FLIGHT_FB, 50.0, Tt4=1000.0, **FB_DESIGN)
dump_case("r5_cold1400", Gas.reacting_forkb(), FLIGHT_FB, 50.0, Tt4=1400.0, **FB_DESIGN)

# --- THE SOLVER SWEEP: enough DISTINCT roots to carry a claim about the solvers ----------
#
# The cases above are the ones the rungs 1-6 suites actually run, and between them they
# produce only 8 distinct fixed-point roots and 3 distinct bisection roots. Phase 1 named
# solver stopping rules as the port's whole residual risk, so "the burner reproduces
# bit-for-bit" cannot rest on three numbers -- a small integer count cannot carry a rate.
#
# This sweep moves every knob the burner's root depends on: the pressure ratio and combustion
# efficiency (which set the residual's scale), Tt4 (which sets its slope), the ambient level
# and flight Mach (which set Tt3 and pt4, so the equilibrium composition at each trial f), and
# mdot (which is a pure scale and therefore a control -- if it ever moved f, that alone would
# be a finding). Each case is run on BOTH the rung-5 fixed point and the rung-6 bisection, so
# the two solvers are exercised on identical operating points and can be compared directly.
# The `fp` column says whether the rung-5 FIXED-POINT arm runs at this point; see case "g".
SWEEP = [
    #  tag       pi_c   Tt4    eta_b  T0      p0        M0    mdot   fp
    # M0 must stay > 0: at static the propulsive efficiency is 0/0 and `_score`'s cascade
    # assert compares 0 < 0. That is a real scope limit of the design-point scorer, not a
    # sweep detail -- a static design point is a rung nobody has built.
    ("a",         6.0, 1450.0, 0.99,  288.15, 101325.0, 0.20,   3.0, True),
    ("b",         8.0, 1500.0, 1.00,  250.0,   50000.0, 0.85,   1.0, True),
    ("c",        12.0, 1600.0, 0.97,  230.0,   40000.0, 1.20,  20.0, True),
    ("d",        16.0, 1700.0, 0.99,  216.7,   30000.0, 1.60,  75.0, True),
    ("e",        20.0, 1900.0, 0.96,  216.7,   18750.0, 2.00,  50.0, True),
    ("f",        25.0, 2000.0, 0.98,  220.0,   26000.0, 2.40, 120.0, True),
    # The HOT, low-pressure-ratio corner: f = 0.052, well above every other point here and
    # ~77 % of stoichiometric. The rung-6 bisection handles it; the rung-5 FIXED POINT does
    # not -- production's Fork-B closure assert (`1e-6 * rhs`) fires. That is a pre-existing
    # envelope limit of the Python being ported, NOT something the port introduces, so the
    # case is kept for the bisection alone rather than deleted: it is the only root in the
    # sweep from a genuinely different regime, and dropping it would narrow exactly the
    # spread this sweep exists to widen.
    ("g",        10.0, 2100.0, 0.95,  288.15, 101325.0, 0.30,   5.0, False),
    ("h",        30.0, 1650.0, 1.00,  240.0,   40000.0, 0.90, 200.0, True),
    ("i",        18.0, 1750.0, 0.94,  216.7,   22000.0, 1.80,  90.0, True),
    ("j",        14.0, 1550.0, 0.985, 260.0,   60000.0, 0.60,  12.0, True),
    ("k",        22.0, 1850.0, 0.975, 210.0,   15000.0, 2.20,  65.0, True),
    ("l",         9.0, 1350.0, 0.93,  270.0,   70000.0, 0.45,   8.0, True),
]

for tag, pi_c, Tt4, eta_b, T0, p0, M0, mdot, fp in SWEEP:
    flight = FlightCondition(T0=T0, p0=p0, M0=M0)
    common = dict(pi_c=pi_c, Tt4=Tt4, p_ambient=p0, pi_d=0.95 * ram_recovery(M0),
                  eta_c=0.90, eta_b=eta_b, pi_b=0.95, eta_t=0.90, eta_m=0.99, pi_n=0.97)
    if fp:
        # The rung-4/5 FIXED POINT (relative-residual stopping rule).
        dump_case(f"sweep5_{tag}", Gas.reacting_forkb(), flight, mdot, **common)
    # The rung-6 BISECTION (bracket-width stopping rule), same operating point.
    dump_case(f"sweep6_{tag}", Gas.reacting_equilibrium(), flight, mdot, **common)


with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-2 cycle oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]}")
