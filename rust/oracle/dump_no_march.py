"""THE ORACLE, phase 4 slice G — every rung-27/28 NO-MARCH value the Rust must reproduce.

The ninth in the family, and a separate file from `dump_march.py` for the usual reason: slice F's
TSV stays frozen as its own audit trail, and each dump's regeneration cost stays proportional to
what it certifies.

WHAT IS NEW HERE, and therefore what the sweep is built around:

  * TWO CLOCKS THAT DISAGREE BY CONSTRUCTION. Rung 26's recombination clock is Ea=0 (so k RISES on
    cooling) and termolecular (c_tot^2); rung 27's NO-destruction clock is Arrhenius (k CRATERS on
    cooling) and bimolecular (c_tot^1). Both of the NO clock's factors AGREE — both drive freezing
    — so its kill test INVERTS rung 26's. `clock/` dumps both kill hooks so the inversion is data.

  * A SECOND `pow` RULE, IN THE OPPOSITE DIRECTION. `_tau_no_exact` spells `(1+beta*a) ** 2` with
    an INTEGER exponent, which may be a product; `_tau_chem_recomb` 400 lines away spells
    `T ** _N_HOHM` with a float CONSTANT, which must reach libm pow. Both live in one Rust module,
    so `exact/` isolates the integer one exactly as slice F's `clock/` isolated the float one.

  * A DORMANT GUARD. `coupled_no_freeze_out_nozzle` ends its beta sweep with
    `if not isfinite(tau_ratio_min): tau_ratio_min = 1.0`. Measured 0 of 55 sampled cells reach it,
    and `_tau_no_exact` returns finite even at 400 K. So it is dumped from the ACCEPTING side only
    here (nothing else is honest), and the REFUSING side is gated in `tests/rung28.rs` by forcing
    the radicals to zero — the rung-20 gate-5 lesson.

  * A REDUCE THE DUMP DELIBERATELY DOES NOT CARRY. Rung 28's structural reduce (feed the frozen
    trajectory to `_coupled_no_march`, recover `_no_freeze_out_expand`) is gated RUST-vs-RUST in
    `tests/rung28.rs`. A Python<->Rust dump compares values and cannot see a loop-shape error
    transcribed identically into both copies. Both sides ARE dumped so the Python's own equality
    is pinned as data and the Rust gate is checking the same claim, not a weaker one.

WHAT IS *NOT* DUMPED, AND WHY. The `max_a` ARGMAX. It is a genuine tripwire — a march that peaked
mid-path would be a real defect and no value key names it — but the Python returns `max_a` without
an index and cannot report one without instrumenting the source. Dumping a class only the Rust can
produce is exactly the mistake slice F's finding 5 records, so it is gated Rust-side in
`tests/rung27.rs` instead. Measured constant (at the exit) over 5 design points x 4 rate scales
spanning 1e-12 to 1e12, including cells where NO is 97% relaxed.

Regenerate with:
    py -3                     rust/oracle/dump_no_march.py rust/oracle/no_march_cpython.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_no_march.py rust/oracle/no_march_pypy.tsv
"""
import math
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.engine import FlightCondition, build_turbojet
from turbojet.gas import (
    Gas, NOFreezeOut, CoupledNOFreezeOut,
    _equilibrium_composition, _no_freeze_out_expand, _frozen_no_trajectory, _coupled_no_march,
    _tau_no_destroy, _tau_no_exact,
)

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
PHI_P = 1.0          # the rung-27/28 suites' own primary equivalence ratio

DPS = [("cold", 1300.0), ("dp", 1500.0), ("warm", 1800.0), ("hot", 2200.0), ("vhot", 2300.0)]

STATES = {}
for tag, tt4 in DPS:
    g = Gas.reacting_equilibrium()
    r = build_turbojet(g, PI_C, tt4, FLIGHT.p0, **LOSSES).run(FLIGHT, 1.0)
    st3, st4, st9 = r.stations["3"], r.stations["4"], r.stations["9"]
    STATES[tag] = dict(gas=g, far=st4.far, Tt3=st3.Tt, Tt4=st4.Tt, pt4=st4.pt,
                       Tt9=st9.Tt, pt9=st9.pt, p9=r.p9)

# ==============================================================================================
# 1. THE ANCHORED NO CLOCK, STANDALONE — including both kill hooks
#
#    Solver-free, so it isolates the Zeldovich reverse rates and the bimolecular density law from
#    everything downstream. The pressure ladder is off-round for the reason slice F's is: an arm
#    whose tau depends on a single ratio silently collapses cells when the grid repeats it.
# ==============================================================================================
_clock_comp = _equilibrium_composition(STATES["hot"]["far"], STATES["hot"]["Tt4"],
                                       STATES["hot"]["pt4"])
CLOCK_T = [800.0, 1100.0, 1400.0, 1700.0, 2000.0, 2300.0]
CLOCK_P = [2.3e4, 5.7e4, 1.43e5, 6.1e5, 2.37e6]
for i, T in enumerate(CLOCK_T):
    for j, p in enumerate(CLOCK_P):
        put(f"clock/free/{i}/{j}", _tau_no_destroy(_clock_comp, T, p))
        put(f"clock/killT/{i}/{j}", _tau_no_destroy(_clock_comp, T, p, kill_T=1800.0))
        put(f"clock/killc/{i}/{j}", _tau_no_destroy(_clock_comp, T, p, kill_c=1.0e-2))

# The [O]=[H]=0 branch returns +inf, which `put` refuses by design, so it is dumped as a PREDICATE.
_no_rad = {sp: (0.0 if sp in ("O", "H") else n) for sp, n in _clock_comp.items()}
put("clock/no_radicals_is_inf",
    1.0 if _tau_no_destroy(_no_rad, 1800.0, 1.0e5) == float("inf") else 0.0)

# ==============================================================================================
# 2. THE EXACT LINEARISED CLOCK — the beta repair's own arithmetic
#
#    This is where `(1+beta*a) ** 2` lives, the one INTEGER exponent in the slice.
# ==============================================================================================
for tag, _ in DPS:
    s = STATES[tag]
    ce = _equilibrium_composition(s["far"], s["Tt4"], s["pt4"])
    zn = s["gas"].zoned_nox(s["far"], s["Tt3"], s["Tt4"], s["pt4"], PHI_P)
    traj = _frozen_no_trajectory(ce, s["Tt9"], s["pt9"], s["p9"], 400)
    for i in range(11):
        st = traj[min(i * 400 // 10, 400)]
        tau_e, beta_i, a_i = _tau_no_exact(st[3], st[2], st[1], zn.x_no_mix)
        tau_s = _tau_no_destroy(st[3], st[2], st[1])
        put(f"exact/{tag}/{i}/tau", tau_e)
        put(f"exact/{tag}/{i}/beta", beta_i)
        put(f"exact/{tag}/{i}/a", a_i)
        put(f"exact/{tag}/{i}/surrogate", tau_s)
        put(f"exact/{tag}/{i}/ratio", tau_e / tau_s)
    put(f"traj/{tag}/T_exit", traj[-1][2])
    put(f"traj/{tag}/T_mid", traj[200][2])
    put(f"traj/{tag}/p_mid", traj[200][1])
    put(f"traj/{tag}/x_no_frozen", zn.x_no_mix)

# ==============================================================================================
# 3. RUNG 27 — the NO march, at the anchored rate and at both rate_scale limits
# ==============================================================================================
frozen_from_entry = 0
for tag, _ in DPS:
    s = STATES[tag]
    for ltag, rs in (("anchored", 1.0), ("slow", 1e-12), ("fast", 1e12), ("mid", 1e6)):
        st = s["gas"].no_freeze_out_nozzle(s["far"], s["Tt3"], s["Tt4"], s["pt4"], s["Tt9"],
                                           s["pt9"], s["p9"], PHI_P, NOFreezeOut(rate_scale=rs))
        put(f"r27/{tag}/{ltag}/Da_entry", st.Da_entry)
        put(f"r27/{tag}/{ltag}/Da_exit", st.Da_exit)
        put(f"r27/{tag}/{ltag}/x_no_relaxed", st.x_no_relaxed)
        put(f"r27/{tag}/{ltag}/max_a", st.max_a)
        put(f"r27/{tag}/{ltag}/relaxed_fraction", st.relaxed_fraction)
        if ltag == "anchored":
            put(f"r27/{tag}/T9_frozen", st.T9_frozen)
            put(f"r27/{tag}/x_no_frozen", st.x_no_frozen)
            put(f"r27/{tag}/x_no_e_entry", st.x_no_e_entry)
            put(f"r27/{tag}/x_no_e_exit", st.x_no_e_exit)
            put(f"r27/{tag}/max_a_frozen", st.max_a_frozen)
            if st.frozen_from_entry:
                frozen_from_entry += 1

# LIVE discrete key — rung 27's headline as an integer. It is 5 of 5 (unlike rung 26's major pool,
# which is 2 of 5 on the same ladder), and that CONTRAST is the rung.
put("census/no_frozen_from_entry", float(frozen_from_entry))

# ==============================================================================================
# 4. RUNG 28 — the coupled march, its uncoupled reduce, and the channel decomposition
# ==============================================================================================
for tag, _ in DPS:
    s = STATES[tag]
    for ctag, couple in (("coupled", True), ("uncoupled", False)):
        st = s["gas"].coupled_no_freeze_out_nozzle(
            s["far"], s["Tt3"], s["Tt4"], s["pt4"], s["Tt9"], s["pt9"], s["p9"], PHI_P,
            CoupledNOFreezeOut(), couple=couple)
        k = f"r28/{tag}/{ctag}"
        put(f"{k}/T9_pool", st.T9_pool)
        put(f"{k}/s_freeze_pool", st.s_freeze_pool)
        put(f"{k}/Da_entry", st.Da_entry)
        put(f"{k}/Da_exit_frozen", st.Da_exit_frozen)
        put(f"{k}/Da_exit_depletion", st.Da_exit_depletion)
        put(f"{k}/Da_exit_heat", st.Da_exit_heat)
        put(f"{k}/Da_exit_coupled", st.Da_exit_coupled)
        put(f"{k}/x_radical_entry", st.x_radical_entry)
        put(f"{k}/x_radical_exit_pool", st.x_radical_exit_pool)
        put(f"{k}/x_no_relaxed", st.x_no_relaxed)
        put(f"{k}/x_no_e_exit", st.x_no_e_exit)
        put(f"{k}/max_a", st.max_a)
        put(f"{k}/a_entry", st.a_entry)
        put(f"{k}/a_exit", st.a_exit)
        put(f"{k}/beta_max", st.beta_max)
        put(f"{k}/tau_ratio_min", st.tau_ratio_min)
        put(f"{k}/depletion_factor", st.depletion_factor)
        put(f"{k}/heat_release_factor", st.heat_release_factor)
        put(f"{k}/net_factor", st.net_factor)
        put(f"{k}/channel_ratio", st.channel_ratio)

# The pool_rate_scale limit — the STRUCTURAL gate (channel 1 unbounded, channel 2 saturating).
for tag in ("dp", "hot"):
    s = STATES[tag]
    for ptag, prs in (("poolfast", 1e6), ("poolslow", 1e-6)):
        st = s["gas"].coupled_no_freeze_out_nozzle(
            s["far"], s["Tt3"], s["Tt4"], s["pt4"], s["Tt9"], s["pt9"], s["p9"], PHI_P,
            CoupledNOFreezeOut(pool_rate_scale=prs))
        put(f"r28lim/{tag}/{ptag}/depletion_factor", st.depletion_factor)
        put(f"r28lim/{tag}/{ptag}/heat_release_factor", st.heat_release_factor)
        put(f"r28lim/{tag}/{ptag}/net_factor", st.net_factor)

# ==============================================================================================
# 5. THE STRUCTURAL REDUCE — both sides, in the TSV
#
#    Gated RUST-vs-RUST in `tests/rung28.rs`; carried here so the Rust gate is checking the same
#    claim the Python makes and not a weaker one.
# ==============================================================================================
for tag in ("dp", "hot"):
    s = STATES[tag]
    ce = _equilibrium_composition(s["far"], s["Tt4"], s["pt4"])
    zn = s["gas"].zoned_nox(s["far"], s["Tt3"], s["Tt4"], s["pt4"], PHI_P)
    nf = s["gas"].nozzle_flow(s["far"], s["Tt4"], s["pt4"], s["Tt9"], s["pt9"], s["p9"],
                              x_no_frozen=zn.x_no_mix)
    tau_res = 0.5 / (0.6 * nf.V9_frozen)

    def _da_no(comp, T, p):
        return tau_res / _tau_no_destroy(comp, T, p)

    for nstep in (100, 400):
        a = _no_freeze_out_expand(ce, s["far"], s["Tt9"], s["pt9"], s["p9"], zn.x_no_mix,
                                  _da_no, nstep)
        traj = _frozen_no_trajectory(ce, s["Tt9"], s["pt9"], s["p9"], nstep)
        b = _coupled_no_march(traj, traj, zn.x_no_mix, _da_no)
        for i, name in enumerate(("T9", "x_no", "x_no_e_exit", "max_a", "Da_entry", "Da_exit")):
            put(f"red/{tag}/{nstep}/r27_{name}", a[i])
            put(f"red/{tag}/{nstep}/r28_{name}", b[i])
        assert all(x == y for x, y in zip(a, b)), \
            f"the STRUCTURAL reduce is NOT bit-exact at {tag}/{nstep} — prediction 2 said it is"

# ==============================================================================================
# 6. DISTINCT-VALUE COUNTS — measured, then asserted. Slice F's lesson: a guessed count bar was
#    wrong, and the shortfall was the physics rather than a defect.
# ==============================================================================================
for arm, want in (("free", 30), ("killT", 30), ("killc", 6)):
    vals = {bits for key, bits, _ in ROWS if key.startswith(f"clock/{arm}/")}
    assert len(vals) == want, f"clock/{arm}: {len(vals)} distinct, expected {want}"
    put(f"roots/clock_{arm}_distinct", float(len(vals)))
march_roots = {bits for key, bits, _ in ROWS if key.startswith("r27/") and "/max_a" in key}
assert len(march_roots) >= 15, f"only {len(march_roots)} distinct rung-27 clamp values"
put("roots/r27_max_a_distinct", float(len(march_roots)))
print(f"[6] distinct: clock arms 30/30/6, rung-27 clamp values {len(march_roots)}")

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-4G NO-march oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
