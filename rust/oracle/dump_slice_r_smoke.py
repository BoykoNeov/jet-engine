"""SLICE R step 1 — the SMOKE dump for rungs 40 + 44 (`TwoSpoolTransient`).

Not the slice's oracle (that is step 4, on both suites' full grids). This exists to catch a
structural mistake BEFORE the 16 Python gates are ported on top of it at steps 2 and 3 — and
§ 5.15 named the mistakes in advance, each of which the shipped Rust deliberately does NOT make:

  1. the `steady` memo keyed on the FLOAT BITS instead of on `round(Tt4, 3)`. Probe 1 measured the
     one collision that exists moving **0 reported values**, so no value key can see this — which
     is why section H dumps the memo's KEY SEQUENCE as its own oracle keys;
  2. `int(round(s_end/ds))` replaced by a truncation. `1.2/0.05 = 23.99999999999999644729`, so the
     round gives 24 and a truncation gives 23 — a whole missing step. Sections F and G therefore
     run AT `s_end = 1.2, ds = 0.05` (rung 40 gate 7's own pair, and the only one of the four in
     use where the round is not exact): an oracle built from the other three would be as blind as
     the gates are;
  3. rung 40's LINEAR running-line reference (`slip_excursion`) unified with rung 44's PER-INSTANT
     one (`phi_excursion`). Probe 5 measured the two extrema agreeing to seven figures while the
     POINTWISE gap reaches 5 %, so section G dumps both references pointwise;
  4. the march routed through rung 34's marcher, which would convert a raise into a truncation;
  5. `equilibrium`'s `best` tracking elided, or its noise-floor exit treated as a rescue path —
     probe 3 measured it as the ORDINARY exit on the reacting gas (6 of 12 cells).

THE CELLS, each touching a path ONCE:

  A  `_close` driven DIRECTLY at two speeds — all 21 of its dict keys, by name.
  B  `_instant` (= `_close` + `_instant_tail`) — all 44 keys, on CPG and on the REACTING gas.
  C  `equilibrium` on both gases, at temperatures probe 3 measured taking DIFFERENT exit branches,
     with the exit kind and the pass count as DISCRETE keys (probe 4 measured both flipping
     between interpreters while every rung-40 gate passes on either).
  D  `lead_threshold` — the `== 1` identity on flat+CPG and the shaped value beside it. It reaches
     rung 39's `match` through the INHERITED table.
  E  `jacobian` / `eigenvalues` / `oscillatory_band` / `damping_ratio_max` — BOTH eigenvalue arms
     and both `b*c` signs, counted ON THIS GRID (gate 5's 245/7 belong to gate 5's grid).
  F  `integrate` at `s_end = 1.2, ds = 0.05` — every point, every field.
  G  `slip_excursion` at gate 7's own call, plus the two running-line references POINTWISE.
  H  `phi_excursion` at `r_ramp = 5.0, s_end = 6.0` — the case where the memo COLLISION fires —
     and `transient_surge_margin` against an armed surge line.
  I  the `lp_disabled` REDUCE against a bare rung-34 `SpoolTransient`, which must be BIT-identical.

WHAT PYTHON CANNOT SEE, AND WHY IT IS NOT FAKED HERE. Three of the Rust's counters are for arms
INSIDE `_close` that Python swallows: the low-wall march-in advances, the non-real guard firing
inside that loop, and the `g` failures they come from. A wrapper cannot count a failure the shipped
body catches, and copying the body into this file to count them would make the dump's arithmetic a
COPY rather than the shipped code. So those three stay Rust-side counters gated against ZERO, with
probe 5's body-copy measurement (0 advances in 6 339 calls) as their provenance; everything Python
CAN see is compared. The two instruments below that recompute anything — the high-wall arm and the
eigenvalue arm — recompute a SCALAR the shipped body also computes, feed no dumped value, and are
labelled at their definition.

Regenerate with:
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_r_smoke.py > rust/oracle/slice_r_smoke_pypy.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import engine as E                                          # noqa: E402
from turbojet.engine import (FlightCondition, build_turbojet,             # noqa: E402
                             build_two_spool_turbojet, ComponentMap,
                             SpoolTransient, TwoSpoolTransient)
from turbojet.gas import Gas                                              # noqa: E402

ROWS = []


def put(key, value):
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putd(key, n):
    n = int(n)
    ROWS.append((key, n, str(n)))


# ================================================================= the instruments
CENSUS = {}


def reset_census():
    CENSUS.clear()
    CENSUS.update(close_calls=0, close_bracket_fails=0, close_nonreal_propagated=0,
                  powers_calls=0, match_calls=0, instant_calls=0, integrate_calls=0,
                  hi_wall_literal=0, hi_wall_map=0, eig_real=0, eig_complex=0,
                  illinois_calls=0, illinois_evals=0, illinois_exhausted=0)
    MATCH_TT4.clear()
    EQ_POWERS.clear()


MATCH_TT4 = []        # every Tt4 handed to `match`, in call order
LAST_TRAJ = []        # the points of the most recent `integrate` call
EQ_POWERS = []        # `_powers` calls per `equilibrium` call, in call order

reset_census()

_ILL = E._illinois


def _ill(f, a, b, fa, fb, tol=1e-10, maxit=100):
    """The shipped Illinois, counted. `illinois_exhausted` is the arm slice Q measured at 103 of
    109 on rung 37's grid and probe 2 measured at 0 of 20 847 here — the SAME counter on the
    opposite population, quoted with its grid and never merged with slice Q's."""
    n = [0]

    def counting(x):
        v = f(x)
        n[0] += 1
        CENSUS["illinois_evals"] += 1
        return v
    CENSUS["illinois_calls"] += 1
    try:
        return _ILL(counting, a, b, fa, fb, tol, maxit)
    finally:
        if n[0] >= maxit:
            CENSUS["illinois_exhausted"] += 1


E._illinois = _ill

_CLOSE = TwoSpoolTransient._close


def _close(self, nu_lp, nu_hp, Tt4, Tt2, pt2):
    """Counts the call, classifies the HIGH WALL's `min`, and calls the SHIPPED body.

    INSTRUMENT NOTE: `n_lp` is recomputed here — three operations the body also performs — purely
    to classify which arm of `min(2.5, phi_max*n_L)` binds. It feeds NO dumped value; every number
    in this file comes out of the shipped body below. Probe 2 measured this `min` as the one
    genuinely contested branch in the class (1 221 literal / 5 118 map), so it is worth a census
    where the swallowed arms are not.
    """
    CENSUS["close_calls"] += 1
    n_lp = nu_lp * (self.Tt2_d / Tt2) ** 0.5
    if 2.5 <= self.map_lp.phi_max() * n_lp:
        CENSUS["hi_wall_literal"] += 1
    else:
        CENSUS["hi_wall_map"] += 1
    try:
        return _CLOSE(self, nu_lp, nu_hp, Tt4, Tt2, pt2)
    except AssertionError as ex:
        if "does not bracket" in str(ex):
            CENSUS["close_bracket_fails"] += 1
        elif "off-map compressor trial" in str(ex):
            CENSUS["close_nonreal_propagated"] += 1
        raise


TwoSpoolTransient._close = _close

_POWERS = TwoSpoolTransient._powers


def _powers(self, c, flight, nu_lp, nu_hp, Tt4):
    CENSUS["powers_calls"] += 1
    return _POWERS(self, c, flight, nu_lp, nu_hp, Tt4)


TwoSpoolTransient._powers = _powers

_INSTANT = TwoSpoolTransient._instant


def _instant(self, flight, nu_lp, nu_hp, Tt4):
    CENSUS["instant_calls"] += 1
    return _INSTANT(self, flight, nu_lp, nu_hp, Tt4)


TwoSpoolTransient._instant = _instant

_MATCH = TwoSpoolTransient.match


def _match(self, flight, Tt4, *a, **k):
    """Records every `Tt4` the INHERITED rung-39 match is asked for.

    This is how the `steady` memo's miss sequence is recovered without reaching inside a closure:
    `_ramp_march` calls `match` ONCE for its start point and the memo calls it exactly ONCE PER
    MISS thereafter, in order.
    """
    CENSUS["match_calls"] += 1
    MATCH_TT4.append(float(Tt4))
    return _MATCH(self, flight, Tt4, *a, **k)


TwoSpoolTransient.match = _match

_INTEGRATE = TwoSpoolTransient.integrate


def _integrate(self, flight, schedule, nu0, s_end, ds):
    """Counts the march and KEEPS its points, so section H can build the memo's key sequence under
    BOTH keying schemes off the same trajectory — the exact-float one is not reachable any other
    way, since the shipped closure only ever calls `match` on a MISS."""
    CENSUS["integrate_calls"] += 1
    out = _INTEGRATE(self, flight, schedule, nu0, s_end, ds)
    LAST_TRAJ[:] = out
    return out


TwoSpoolTransient.integrate = _integrate

_EIG = TwoSpoolTransient.__dict__["eigenvalues"].__func__


def _eigenvalues(J):
    """INSTRUMENT NOTE: recomputes the discriminant (four operations) to classify the arm, then
    returns the SHIPPED body's result. Counted on THIS grid — gate 5's 245 real / 7 complex are
    gate 5's grid's numbers and are not restated here."""
    tr = J[0][0] + J[1][1]
    det = J[0][0] * J[1][1] - J[0][1] * J[1][0]
    if tr * tr - 4.0 * det >= 0.0:
        CENSUS["eig_real"] += 1
    else:
        CENSUS["eig_complex"] += 1
    return _EIG(J)


TwoSpoolTransient.eigenvalues = staticmethod(_eigenvalues)

_EQ = TwoSpoolTransient.equilibrium


def _equilibrium(self, flight, Tt4, start=None):
    """Records `_powers` calls per equilibrium, which is what makes the EXIT KIND and the PASS
    COUNT recoverable without reaching into the loop: each Newton pass calls `_powers` three times
    (residual + two Jacobian columns) and the primary return costs ONE more, so

        primary at pass k  =>  3k + 1 calls        noise (all 80 passes)  =>  240 calls

    and `3k + 1 = 240` has no integer solution, so the classification is unambiguous.
    """
    n0 = CENSUS["powers_calls"]
    try:
        return _EQ(self, flight, Tt4, start)
    finally:
        EQ_POWERS.append(CENSUS["powers_calls"] - n0)


TwoSpoolTransient.equilibrium = _equilibrium


def eq_kind_and_iters(n_powers):
    """(exit kind, pass count) from the `_powers` call count — see `_equilibrium`."""
    if n_powers == 3 * TwoSpoolTransient._EQ_MAX:
        return 1, TwoSpoolTransient._EQ_MAX          # 1 == the NOISE-FLOOR exit
    assert n_powers % 3 == 1, ("a primary exit costs 3k+1 _powers calls", n_powers)
    return 0, (n_powers - 1) // 3


def emit_census(prefix):
    for k in sorted(CENSUS):
        putd(f"census/{prefix}/{k}", CENSUS[k])
    reset_census()


# ================================================================= the dict emitters
# THE KEY LISTS COME FROM PYTHON'S OWN DICTS, never from the Rust struct: a field forgotten in the
# port must show up as a MISSING comparison, and it only can if the dump enumerates the source.
CLOSE_KEYS = 21
INSTANT_KEYS = 44


def put_close(prefix, c):
    assert len(c) == CLOSE_KEYS, (len(c), sorted(c))
    assert isinstance(c["wgas"], Gas)
    for k in sorted(c):
        if k != "wgas":
            put(f"{prefix}/{k}", c[k])


def put_instant(prefix, i):
    assert len(i) == INSTANT_KEYS, (len(i), sorted(i))
    assert isinstance(i["wgas"], Gas)
    for k in sorted(i):
        if k == "wgas":
            continue
        if k == "branch":
            putd(f"{prefix}/branch_choked", 1 if i[k] == "choked" else 0)
        else:
            put(f"{prefix}/{k}", i[k])


# ======================================================================== the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)

FLAT = ComponentMap.flat()
LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)


def cpg_gas():
    """The suites' self-consistent CPG dual gas: R_t = (g-1)/g*cp_t exactly."""
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


def design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def tt(gas, ml=LP_SHAPED, mh=HP_SHAPED, rho=1.0):
    return TwoSpoolTransient(design(gas), FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=rho)


T_CPG = tt(cpg_gas())

# --------------------------------------------------------------- A: the forward closure, direct
Tt2, pt2, V0 = T_CPG._inlet(FLIGHT)
put("A/tt2", Tt2)
put("A/pt2", pt2)
put("A/v0", V0)
for ic, (nu_lp, nu_hp) in enumerate(((1.0, 1.0), (0.92, 0.96))):
    put_close(f"A/{ic}", T_CPG._close(nu_lp, nu_hp, 1200.0, Tt2, pt2))
emit_census("A")

# --------------------------------------------------------------- B: the instant, both gases
for ic, (nu_lp, nu_hp, Tt4) in enumerate(((1.0, 1.0, 1200.0), (0.92, 0.96, 1350.0))):
    put_instant(f"B/cpg/{ic}", T_CPG._instant(FLIGHT, nu_lp, nu_hp, Tt4))
emit_census("B/cpg")

T_RE = tt(Gas.reacting_equilibrium())
put_instant("B/re/0", T_RE._instant(FLIGHT, 1.0, 1.0, 1500.0))
emit_census("B/re")

# --------------------------------------------------------------- C: the 2-D Newton, both exits
# Probe 3: on the reacting gas 1500 exits PRIMARY and 1450 through the NOISE floor, on this very
# map pair — so both arms of the return are live in three cells.
for it, Tt4 in enumerate((1500.0, 1200.0)):
    eq = T_CPG.equilibrium(FLIGHT, Tt4)
    put_instant(f"C/cpg/{it}", eq)
    kind, iters = eq_kind_and_iters(EQ_POWERS[-1])
    putd(f"C/cpg/{it}/exit_noise", kind)
    putd(f"C/cpg/{it}/passes", iters)
    putd(f"C/cpg/{it}/powers_calls", EQ_POWERS[-1])
# ...and the ONE shipped signature branch nothing above takes: an explicit START. It also gives the
# pass count a second population, which is the key probe 4 measured flipping between interpreters.
eq = T_CPG.equilibrium(FLIGHT, 1200.0, start=(0.90, 0.95))
put("C/start/nu_lp", eq["nu_lp"])
put("C/start/nu_hp", eq["nu_hp"])
put("C/start/pi_lpc", eq["pi_lpc"])
put("C/start/Phi_lp", eq["Phi_lp"])
put("C/start/Phi_hp", eq["Phi_hp"])
kind, iters = eq_kind_and_iters(EQ_POWERS[-1])
putd("C/start/exit_noise", kind)
putd("C/start/passes", iters)
putd("C/start/powers_calls", EQ_POWERS[-1])
emit_census("C/cpg")

for it, Tt4 in enumerate((1500.0, 1450.0)):
    eq = T_RE.equilibrium(FLIGHT, Tt4)
    put(f"C/re/{it}/nu_lp", eq["nu_lp"])
    put(f"C/re/{it}/nu_hp", eq["nu_hp"])
    put(f"C/re/{it}/Phi_lp", eq["Phi_lp"])
    put(f"C/re/{it}/Phi_hp", eq["Phi_hp"])
    put(f"C/re/{it}/pi_lpc", eq["pi_lpc"])
    put(f"C/re/{it}/pi_hpc", eq["pi_hpc"])
    put(f"C/re/{it}/mdot_air", eq["mdot_air"])
    kind, iters = eq_kind_and_iters(EQ_POWERS[-1])
    putd(f"C/re/{it}/exit_noise", kind)
    putd(f"C/re/{it}/passes", iters)
    putd(f"C/re/{it}/powers_calls", EQ_POWERS[-1])
emit_census("C/re")

# --------------------------------------------------------------- D: sigma_crit, through `match`
T_FLAT = tt(cpg_gas(), FLAT, FLAT)
put("D/flat_identity", T_FLAT.lead_threshold(FLIGHT, 1100.0, d=25.0))
put("D/shaped", T_CPG.lead_threshold(FLIGHT, 1100.0))
od = T_CPG.match(FLIGHT, 1100.0)
put("D/shaped_at_nu", T_CPG.lead_threshold(FLIGHT, 1100.0, d=5.0,
                                           nu=(od.N_lp_ratio, od.N_hp_ratio)))
put("D/nu_lp", od.N_lp_ratio)
put("D/nu_hp", od.N_hp_ratio)
emit_census("D")

# --------------------------------------------------------------- E: the 2x2 and its two arms
NU_E = (od.N_lp_ratio, od.N_hp_ratio)
J = T_CPG.jacobian(FLIGHT, 1100.0, nu=NU_E)
for r in range(2):
    for cc in range(2):
        put(f"E/J/{r}{cc}", J[r][cc])
band = T_CPG.oscillatory_band(FLIGHT, 1100.0, nu=NU_E)
putd("E/band_is_none", 0 if band is not None else 1)
put("E/band_lo", band[0])
put("E/band_hi", band[1])
put("E/damping_max", T_CPG.damping_ratio_max(FLIGHT, 1100.0, nu=NU_E))
# BOTH eigenvalue arms on THIS grid: rho=1 is real, and rho inside the band is complex.
for ir, rho in enumerate((1.0, (band[0] * band[1]) ** 0.5)):
    Jr = [[J[0][0] / rho, J[0][1] / rho], [J[1][0], J[1][1]]]
    lo, hi = T_CPG.eigenvalues(Jr)
    put(f"E/eig/{ir}/lo", lo)
    put(f"E/eig/{ir}/hi", hi)
    put(f"E/eig/{ir}/rho", rho)
# The flat-LP discriminator: b*c >= 0, so NO band and zero damping.
od_f = T_FLAT.match(FLIGHT, 1100.0)
NU_F = (od_f.N_lp_ratio, od_f.N_hp_ratio)
putd("E/flat/band_is_none", 1 if T_FLAT.oscillatory_band(FLIGHT, 1100.0, nu=NU_F) is None else 0)
put("E/flat/damping_max", T_FLAT.damping_ratio_max(FLIGHT, 1100.0, nu=NU_F))
emit_census("E")

# --------------------------------------------------------------- F: the march, EVERY point
# s_end = 1.2, ds = 0.05 — rung 40 gate 7's pair, and the ONLY one of the four in use where
# int(round(s_end/ds)) is not exact (23.99999999999999644729 -> 24, truncation -> 23).
TT4_LO, DTT4, R_RAMP, S_END, DS = 1100.0, 50.0, 0.5, 1.2, 0.05
od_lo = T_CPG.match(FLIGHT, TT4_LO)
NU0 = (od_lo.N_lp_ratio, od_lo.N_hp_ratio)


def ramp(t):
    return TT4_LO + DTT4 * min(1.0, t / R_RAMP)


PTS = T_CPG.integrate(FLIGHT, ramp, NU0, S_END, DS)
putd("F/npts", len(PTS))
for ip, p in enumerate(PTS):
    for k in ("s", "nu_lp", "nu_hp", "Tt4", "slip", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
              "mdot_air", "f", "Phi_lp", "Phi_hp", "sp_thrust"):
        put(f"F/{ip}/{k}", getattr(p, k))
emit_census("F")

# --------------------------------------------------------------- G: the TWO running-line references
# Rung 40 subtracts a LINEAR interpolation between the two endpoint matches; rung 44 subtracts a
# match at the INSTANTANEOUS Tt4. Probe 5 measured the extrema agreeing to seven figures while the
# pointwise gap reaches 5 % of the extremum — so the extremum alone would pass a port that unified
# them, and both references are dumped POINTWISE.
put("G/slip_excursion", T_CPG.slip_excursion(FLIGHT, TT4_LO, DTT4, s_end=S_END, ds=DS))
# AND A NON-SATURATING RAMP, because the default one CANNOT SEE the reference choice. At
# r_ramp = 0.5 the extremum is attained at s = 0.5, where the ramp has just saturated: there
# u == 1 exactly, so the linear reference IS the endpoint match, bit for bit, and unifying the
# two references moves NOTHING. At r_ramp = 3.0 the ramp never saturates inside s_end and the
# two differ by 2.4 % (-7.1498e-4 vs -6.9835e-4). Measured by injecting the unification, which
# the r_ramp = 0.5 cell reported as ONE census key and zero values.
put("G/slip_excursion_slow", T_CPG.slip_excursion(FLIGHT, TT4_LO, DTT4, r_ramp=3.0,
                                                  s_end=S_END, ds=DS))
T_CPG.rho = 2.0
put("G/slip_excursion_rho2", T_CPG.slip_excursion(FLIGHT, TT4_LO, DTT4, s_end=S_END, ds=DS))
T_CPG.rho = 1.0
od_hi = T_CPG.match(FLIGHT, TT4_LO + DTT4)
put("G/slip_lo", od_lo.slip)
put("G/slip_hi", od_hi.slip)
for ip in (0, 4, 8, 12, 16, 20, 24):
    p = PTS[ip]
    u = (p.Tt4 - TT4_LO) / DTT4
    linear = od_lo.slip + u * (od_hi.slip - od_lo.slip)
    instant = T_CPG.match(FLIGHT, p.Tt4).slip
    put(f"G/{ip}/slip", p.slip)
    put(f"G/{ip}/ref_linear", linear)
    put(f"G/{ip}/ref_instant", instant)
    put(f"G/{ip}/err_linear", p.slip - linear)
    put(f"G/{ip}/err_instant", p.slip - instant)
emit_census("G")

# --------------------------------------------------------------- H: rung 44, and the memo's keys
# r_ramp = 5.0, s_end = 6.0 is the case (301 points) in which probe 1's ONE collision between
# distinct Tt4 floats FIRES — 1399.9999999999984 and 1400.0 share the key 1400.0, and the second
# reads the first's cached phi. It is worth 0 reported values, so the KEY SEQUENCE is dumped.
ex = T_CPG.phi_excursion(FLIGHT, 1000.0, 400.0, r_ramp=5.0, s_end=6.0)
for k in ("ext_lp", "ext_hp", "s_lp", "s_hp", "min_phi_lp", "min_phi_hp", "ratio"):
    put(f"H/exc/{k}", ex[k])
putd("H/exc/npts", ex["npts"])
MISSES = MATCH_TT4[1:]          # [0] is `_ramp_march`'s own start-point match
putd("H/exc/match_calls", len(MATCH_TT4))
putd("H/exc/steady_misses", len(MISSES))
putd("H/exc/steady_calls", 2 * ex["npts"])
for ik, x in enumerate(MISSES):
    put(f"H/exc/key/{ik}", round(x, 3))
put("H/exc/tt4_first_miss", MISSES[0])
put("H/exc/tt4_last_miss", MISSES[-1])

# PREDICTION 4, MEASURED. The shipped memo keys on `round(Tt4, 3)`; keying on the exact float
# would insert one MORE entry, because 1399.9999999999984 and 1400.0 share the rounded key. Both
# sequences are built from the SAME marched trajectory, so this compares the equivalence relation
# and nothing else — and the shipped sequence recovered from the `match` calls is asserted to be
# the rounded one, which is what makes the count below a measurement of the memo rather than of a
# re-implementation of it.
seen_r, seen_x, keys_r, keys_x = set(), set(), [], []
for p in LAST_TRAJ:
    kr = round(p.Tt4, 3)
    if kr not in seen_r:
        seen_r.add(kr)
        keys_r.append(kr)
    if p.Tt4 not in seen_x:
        seen_x.add(p.Tt4)
        keys_x.append(p.Tt4)
assert keys_r == [round(x, 3) for x in MISSES], "the recovered miss sequence is not the rounded one"
putd("H/exc/keys_rounded", len(keys_r))
putd("H/exc/keys_exact", len(keys_x))
putd("H/exc/collisions", len(keys_x) - len(keys_r))
emit_census("H/exc")

ARMED = tt(cpg_gas(), LP_SHAPED.with_phi_surge(0.86), HP_SHAPED.with_phi_surge(0.90))
sm = ARMED.transient_surge_margin(FLIGHT, 1000.0, 400.0, r_ramp=0.3)
for k in ("margin_min_lp", "margin_min_hp", "steady_min_lp", "steady_min_hp",
          "phi_surge_lp", "phi_surge_hp"):
    put(f"H/sm/{k}", sm[k])
putd("H/sm/crossed_lp", 1 if sm["crossed_lp"] else 0)
putd("H/sm/crossed_hp", 1 if sm["crossed_hp"] else 0)
putd("H/sm/npts", sm["npts"])
putd("H/sm/steady_misses", len(MATCH_TT4) - 1)
putd("H/sm/steady_calls", 2 * sm["npts"])
for ik, x in enumerate(MATCH_TT4[1:]):
    put(f"H/sm/key/{ik}", round(x, 3))
emit_census("H/sm")

# --------------------------------------------------------------- I: the lp_disabled REDUCE
single = build_turbojet(Gas.reacting_equilibrium(), PI_HPC, TT4, FLIGHT.p0,
                        pi_d=REAL["pi_d"], eta_c=REAL["eta_hpc"], eta_b=REAL["eta_b"],
                        pi_b=REAL["pi_b"], eta_t=REAL["eta_hpt"], eta_m=REAL["eta_m"],
                        pi_n=REAL["pi_n"], nozzle_convergent=True)
DEG = TwoSpoolTransient(single, FLIGHT, 1.0, map_hp=HP_SHAPED, lp_disabled=True)
REF = SpoolTransient(single, FLIGHT, 1.0, comp_map=HP_SHAPED)
for it, Tt4 in enumerate((1500.0, 1200.0)):
    a = DEG._degenerate.equilibrium(FLIGHT, Tt4)
    b = REF.equilibrium(FLIGHT, Tt4)
    for k in ("nu", "pi_c", "tau_c", "tau_t", "mdot_air", "f", "Phi", "sp_thrust"):
        assert a[k] == b[k], (Tt4, k, a[k], b[k])
        put(f"I/{it}/{k}", a[k])
emit_census("I")

# =========================================================================== emit
out = sys.stdout
out.write("# slice R smoke — rungs 40+44 TwoSpoolTransient — key\tu64 bits (or an integer)\trepr\n")
for key, bits, text in ROWS:
    out.write(f"{key}\t{bits}\t{text}\n")
sys.stderr.write(f"[dump_slice_r_smoke] {len(ROWS)} values\n")
