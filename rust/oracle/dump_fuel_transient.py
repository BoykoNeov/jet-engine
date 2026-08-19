"""SLICE S step 4 — THE ORACLE for rungs 43 + 45 (`TwoSpoolFuelTransient`), over BOTH suites' grids.

Step 1's smoke touched every path ONCE (5 536 values, twelve sections) on cells chosen to reach a
path rather than to reproduce a gate. This dump runs the grids the two Python suites actually
sweep — `test_rung43.py`'s ten gates and `test_rung45.py`'s nine — because steps 2 and 3 both
measured that those gates cannot see several things the port could get wrong:

  * **seven of rung 43's eleven gates are blind to a wrong LP derivative** (step 2 finding 2), and
    rung 45 has NO dynamical reduce at all (step 3 finding 4). Both suites are signs, orderings
    and spreads; this file is the numbers underneath them.
  * **`npts` is the ONLY channel that witnesses the march bound.** Step 3 finding 3 measured
    dropping the `r` from `r + s_settle` leaving `min_phi_lp` BIT-IDENTICAL at all four ramp
    rates while `npts` moved 351/326/316/306 → 301. § 5.16 booked `npts` to this step as an
    oracle key; it is emitted on EVERY marched cell, not one.
  * **`s_lp`/`s_hp`, `min_phi_hp`, `ratio` and `E_temp_*`** are returned by the shipped methods
    and read by NOTHING in either suite — carried on every excursion cell here.

**THE CENSUS WAS MEASURED ON THIS GRID BEFORE A GATE WAS WRITTEN, AND FOUR REGISTERED NUMBERS
DIED.** § 5.16's predictions 4 and 6 quote a census taken by `probe_s2.py`, whose header calls its
grid "rungs 43 and 45's OWN grids" and which is in fact a cross-product of its own choosing
(3 shapes × 4 `rho` × 4 `r`, then 4 shapes × 3 `rho` × 5 ramps). On the grids the suites really
sweep:

  | quantity                        | § 5.16 registered | MEASURED here |
  |---------------------------------|-------------------|---------------|
  | `integrate_fuel` calls          | 162               | **143**       |
  | …of them on the `412.5` tie     | 21                | **52**        |
  | high wall `literal / map / hi0` | 24 033 / 200 193 / 3 663 | **1 398 / 228 801 / 1 210** |
  | CPG floats moving under CPython  | **0** (prediction 2) | **15**, all libm — see the Rust |

(This dump's own cells give **1 398 / 223 890 / 1 210** over 140 marches: it folds rung 43 gate
10's `freeze_channels` call into section F rather than repeating it, which costs 3 marches and
4 911 map-arm calls and no literal or `hi0` ones. Both numbers are stated because quoting either
for the other is the mistake this whole table is about.)

An instrument's own docstring is not evidence about what it measured: probe 2's grid is neither a
superset nor a subset of the suites', and the header saying otherwise is why nobody re-derived it.
Fourth time in one slice that a census turned out to be a property of the grid rather than of the
code. The tie
population is the one that matters: prediction 4 sized the `round_ties_even` exposure at 13 % of
marches and it is **36 %** — every `r = 0.25` cell of rung 43's gates 5, 6, 7, 9 and 10 lands on
`8.25 / 0.02 = 412.5` exactly.

**AND THE HIGH WALL'S TWO RARE ARMS ARE REACHED BY EXACTLY ONE CELL OF THE TWENTY GATES.** All
1 398 literal hits and all 1 210 `hi0` hits come from `test_rung45.py`'s **`hp-only` shape**, whose
LP map is `ComponentMap.flat()`: a flat map has no `phi_max` ceiling, so `2.5` binds on the accel
(1 301) and the decel's low fuel drops `hi0` below both (1 207). Drop that one cell and two of the
three arms go dead — which is why the census is emitted PER CELL there and not per section.

THE THREE ARMS:

  main    the CPG grids of both suites — sections A…S. ~12 s on PyPy.
  gas     the NON-CPG gases — the three admitted TPG ones (probe 3's detector) and the one
          `Gas.reacting_equilibrium()` the fuel path REFUSES. Named `gas` rather than slice R's
          `equil` because here the equilibrium gas is the one that produces no values at all.
  cpython main + gas in ONE file, for the interpreter arm.

**WHY THE TPG HALF IS A DECLARED FRAGILE SET AND THE CPG HALF IS NOT.** § 5.16 probe 1 dumped the
fuel path on all three admitted gases and measured **391 of 398 float keys moving** CPython vs
PyPy at ~1e-10 relative, with every arming predicate, branch verdict and iteration count
IDENTICAL; probe 3 measured `equilibrium_fuel`'s Newton pass count swinging 16-fold on the same
gases, because its `1e-12` bar is ABSOLUTE and sits below the gas sub-solve's own noise. So the
TPG keys are gated bit-exact against PyPy and published as a deviation distribution against
CPython, NEVER summed with the CPG half.

Regenerate with:
    .venv\\Scripts\\python.exe rust/oracle/dump_fuel_transient.py main    rust/oracle/fuel_transient_pypy.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_fuel_transient.py gas     rust/oracle/fuel_transient_gas_pypy.tsv
    C:\\Python314\\python.exe  rust/oracle/dump_fuel_transient.py cpython rust/oracle/fuel_transient_cpython.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import engine as E                                          # noqa: E402
from turbojet.engine import (ComponentMap, FlightCondition,               # noqa: E402
                             SpoolTransient, TwoSpoolFuelTransient,
                             TwoSpoolTransient, build_turbojet,
                             build_two_spool_turbojet)
from turbojet.gas import Gas                                              # noqa: E402

ARM = sys.argv[1] if len(sys.argv) > 1 else "main"
OUT = sys.argv[2] if len(sys.argv) > 2 else None
assert ARM in ("main", "gas", "cpython"), ARM

ROWS = []


def put(key, value):
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putd(key, n):
    """A discrete key. NEGATIVE values are written as their two's-complement `u64`, because the
    Rust comparator parses this column as a `u64` and a bare `-1` is a parse error rather than a
    mismatch — a golden that CRASHES the reader is worse than one that disagrees with it. The
    text column keeps the signed spelling, so `-1` stays readable at a glance."""
    n = int(n)
    ROWS.append((key, n if n >= 0 else n + (1 << 64), str(n)))


# ================================================================= the instruments
# Identical in intent to `dump_slice_s_smoke.py`'s, and identical in one further respect: every
# NUMBER comes out of the SHIPPED body. The ONE wrapper that recomputes anything (`_close_fuel`'s
# wall classifier) recomputes three scalars the body also computes, feeds no dumped value, and
# says so at its definition.
CENSUS = {}
# THREE RUST COUNTERS ARE DELIBERATELY ABSENT HERE, and the Rust states its own relation to the
# keys that ARE, which is a claim about the marcher's shape rather than a copied number:
#     der_calls == 4 * march_points        (RK4, four `der` per loop iteration, one point each)
#     rw_calls  == der_calls               (`_release_weight` is called unconditionally in `der`)
#     march_points == the sum of this section's `…/npts` keys — EXCEPT section F, where
#         `freeze_channels` runs THREE marches per cell and reports only their peak
#         temperatures, so no length is available to publish. Rung 44's own lengths are spelled
#         `npts44` precisely so they cannot join that sum: they come off a DIFFERENT class's
#         marcher and this counter never sees them.
# `der_calls` counts a LOCAL closure Python offers no handle on; emitting a constant zero for it
# would make the comparison a gate that FAILS on a correct port.
KEYS = dict(close_calls=0, instant_calls=0, eq_calls=0, eq_passes=0,
            march_calls=0, march_points=0,
            topping_calls=0, sched_calls=0, sched_dormant=0, surge_calls=0, surge_dormant=0,
            rw_calls=0, rw_one=0, rw_interior=0, rw_zero=0,
            lo_floor_hits=0, hi_wall_literal=0, hi_wall_map=0, hi_wall_hi0=0,
            interp_low=0, interp_mid=0, interp_high=0,
            illinois_calls=0, illinois_evals=0, illinois_exhausted=0)


def reset_census():
    CENSUS.clear()
    CENSUS.update(KEYS)
    INSTANT_N[:] = [0]


INSTANT_N = [0]

reset_census()

_ILL = E._illinois


def _ill(f, a, b, fa, fb, tol=1e-10, maxit=100):
    """The shipped Illinois, counted. `illinois_exhausted` is the counter slice Q measured at
    103 of 109 on rung 37's grid and slice R at 0 of 20 847 on rung 40's — quoted here with THIS
    grid attached, and never merged with either."""
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


def _count(cls, name, key, extra=None):
    """Wrap a method to bump `key`, calling the SHIPPED body. No arithmetic is duplicated."""
    orig = getattr(cls, name)

    def wrapped(self, *a, **k):
        CENSUS[key] += 1
        out = orig(self, *a, **k)
        if extra is not None:
            extra(self, out, a, k)
        return out
    setattr(cls, name, wrapped)
    return orig


_CLOSE_FUEL = TwoSpoolFuelTransient._close_fuel


def _close_fuel_counted(self, nu_lp, nu_hp, mdot_fuel, Tt2, pt2):
    """Counts the call, classifies the THREE-ARM HIGH WALL and the LOW floor, and calls the
    SHIPPED body.

    `n_lp`, `lo0` and `hi0` are recomputed here — a handful of operations the body also performs
    — purely to say WHICH arm of `min(2.5, phi_max*n_L, hi0)` and of `max(lo0, 0.02)` binds. They
    feed NO dumped value.

    IT EARNS THE DUPLICATION BECAUSE NOTHING ELSE CAN SEE THE ARMS. A partition-sum check
    (`literal + map + hi0 == calls`) passes identically whether the third arm binds or is ABSENT
    FROM THE SOURCE — step 1 finding 4 caught exactly that. Classified here, each arm becomes a
    compared key, and on this grid the split is the finding: 1 398 / 228 801 / 1 210, with both
    rare arms confined to ONE cell of the twenty gates.

    Python's `min(a, b, c)` is a FOLD, not a pairwise `min`, and is spelled as one: first wins on
    a tie.
    """
    CENSUS["close_calls"] += 1
    n_lp = nu_lp * (self.Tt2_d / Tt2) ** 0.5
    lo0 = mdot_fuel * Tt2 ** 0.5 / (0.065 * self.mcorr_lp_d * pt2)
    hi0 = mdot_fuel * Tt2 ** 0.5 / (0.004 * self.mcorr_lp_d * pt2)
    if lo0 < 0.02:
        CENSUS["lo_floor_hits"] += 1
    cap, arm = 2.5, "hi_wall_literal"
    wall_map = self.map_lp.phi_max() * n_lp
    if wall_map < cap:
        cap, arm = wall_map, "hi_wall_map"
    if hi0 < cap:
        arm = "hi_wall_hi0"
    CENSUS[arm] += 1
    return _CLOSE_FUEL(self, nu_lp, nu_hp, mdot_fuel, Tt2, pt2)


TwoSpoolFuelTransient._close_fuel = _close_fuel_counted

_INTERP = TwoSpoolFuelTransient._interp


def _interp_counted(xs, ys, x):
    """Classifies `_interp`'s three arms. Called only from `ramp_excursion_fuel`'s two
    running-line lookups, and the LOW/HIGH clamps are where a march that has left the sampled
    band shows up — a port that fell through to `ys[-1]` instead of clamping would return the
    same number on the interior and a different one at both ends."""
    if x <= xs[0]:
        CENSUS["interp_low"] += 1
    elif x >= xs[-1]:
        CENSUS["interp_high"] += 1
    else:
        CENSUS["interp_mid"] += 1
    return _INTERP(xs, ys, x)


TwoSpoolFuelTransient._interp = staticmethod(_interp_counted)


def _march_extra(self, out, a, k):
    CENSUS["march_points"] += len(out)


_count(TwoSpoolFuelTransient, "integrate_fuel", "march_calls", _march_extra)
_count(TwoSpoolFuelTransient, "_topping_fuel", "topping_calls")


def _instant_extra(self, out, a, k):
    INSTANT_N[0] += 1


_count(TwoSpoolFuelTransient, "_instant_fuel", "instant_calls", _instant_extra)


def _leg_extra(key):
    def extra(self, out, a, k):
        # The DORMANT branch returns `mf_sched` ITSELF — a float compared with itself. `is`
        # catches it without arithmetic.
        if out is a[3]:
            CENSUS[key] += 1
    return extra


_count(TwoSpoolFuelTransient, "_sched_fuel", "sched_calls", _leg_extra("sched_dormant"))
_count(TwoSpoolFuelTransient, "_surge_fuel", "surge_calls", _leg_extra("surge_dormant"))

_RW = E._release_weight


def _rw(s, s_off, tau_rel):
    w = _RW(s, s_off, tau_rel)
    CENSUS["rw_calls"] += 1
    if w >= 1.0:
        CENSUS["rw_one"] += 1
    elif w == 0.0:
        CENSUS["rw_zero"] += 1
    else:
        CENSUS["rw_interior"] += 1
    return w


E._release_weight = _rw

_EQ = TwoSpoolFuelTransient.equilibrium_fuel
LAST_PASSES = [0]


def _equilibrium_fuel(self, flight, mdot_fuel, start=None):
    """Records the Newton PASS COUNT, recovered from `_instant_fuel` calls rather than from
    inside the loop: each pass costs 3 (the residual + two Jacobian columns), the check that
    EXITS costs the first of those, and the returned instant costs one more — `3p + 2`, never a
    multiple of 3, so the recovery is unambiguous."""
    n0 = INSTANT_N[0]
    # COUNTED ONLY ON THE TWO-SHAFT PATH. Python's wrapper sits on the class method, which the
    # DEGENERATE object enters too and then forwards to rung 35's own solve; Rust's counter lives
    # inside `FuelTransientCore::try_equilibrium_fuel`, which that forward never reaches. Counting
    # the forward here would make section C's `eq_calls` a gate that fails on a correct port.
    if getattr(self, "_degenerate", None) is None:
        CENSUS["eq_calls"] += 1
    out = _EQ(self, flight, mdot_fuel, start)
    if getattr(self, "_degenerate", None) is not None:
        # lp_disabled FORWARDS to rung 35's own solve, which never touches this class's
        # `_instant_fuel`. Section C's gate is the bit-equality against a bare SpoolTransient.
        LAST_PASSES[:] = [-1]
        return out
    n = INSTANT_N[0] - n0
    assert n % 3 == 2, ("an equilibrium_fuel exit costs 3p+2 _instant_fuel calls", n)
    CENSUS["eq_passes"] += (n - 2) // 3
    LAST_PASSES[:] = [(n - 2) // 3]
    return out


TwoSpoolFuelTransient.equilibrium_fuel = _equilibrium_fuel


def emit_census(prefix):
    for k in sorted(CENSUS):
        putd(f"census/{prefix}/{k}", CENSUS[k])
    reset_census()


def kind_of(exc):
    """Which arm produced an `AssertionError`, by the substrings the Rust classifies on too.

    0–4 are `fuel_transient.rs`'s `FuelAbort` variants verbatim. 5 and 6 are the two ENUM-level
    refusals rung 45's degenerate gate reaches — a distinction that only exists because they are
    raised by DIFFERENT statements and section K measures which one wins."""
    s = str(exc)
    return (0 if "non-equilibrium" in s else 1 if "inverse: root not bracketed" in s
            else 2 if "off-map compressor trial" in s else 3 if "does not bracket" in s
            else 5 if "needs a surge line on BOTH maps" in s
            else 6 if "inherently two-shaft" in s else 4)


# ================================================================= the dict emitters
# THE KEY LISTS COME FROM PYTHON'S OWN DICTS, never from the Rust struct: a field forgotten in the
# port must show up as a MISSING comparison, and it only can if the dump enumerates the source.
EQF_KEYS = 45          # `equilibrium_fuel` returns an `_instant_fuel` dict
POINT_KEYS = 14


def put_eqf(prefix, d, passes=True):
    assert len(d) == EQF_KEYS, (len(d), sorted(d))
    assert isinstance(d["wgas"], Gas)
    for k in sorted(d):
        if k == "wgas":
            continue
        if k == "branch":
            putd(f"{prefix}/branch_choked", 1 if d[k] == "choked" else 0)
        else:
            put(f"{prefix}/{k}", d[k])
    if passes:
        putd(f"{prefix}/passes", LAST_PASSES[0])


def put_point(prefix, p):
    assert len(p) == POINT_KEYS, (len(p), sorted(p))
    for k in sorted(p):
        if k == "branch":
            putd(f"{prefix}/branch_choked", 1 if p[k] == "choked" else 0)
        else:
            put(f"{prefix}/{k}", p[k])


def put_ramp(prefix, e):
    """RUNG 43's `ramp_excursion_fuel` — all seven reported fields, `npts`, and the LOCATION of
    the peak plus the peak point ITSELF.

    The location key is why this is more than a summary. § 5.16 prediction 4 measured the peak
    attained at point 13 of 413 — 3 % into a march that is 95 % settling tail — so every reported
    field of this cell is decided by a handful of early points, and a port whose interior drifted
    after them would agree on all seven. `i_peak` pins WHERE, and the dumped point pins WHAT."""
    assert len(e) == 8, sorted(e)
    put(f"{prefix}/r", e["r"])
    put(f"{prefix}/rho", e["rho"])
    put(f"{prefix}/Tt4_peak", e["Tt4_peak"])
    put(f"{prefix}/X", e["X"])
    put(f"{prefix}/E_temp_H", e["E_temp_H"])
    put(f"{prefix}/E_temp_L", e["E_temp_L"])
    putd(f"{prefix}/complete", 1 if e["complete"] else 0)
    traj = e["traj"]
    putd(f"{prefix}/npts", len(traj))
    ip = max(range(len(traj)), key=lambda i: traj[i]["Tt4"])
    putd(f"{prefix}/i_peak", ip)
    put_point(f"{prefix}/peak", traj[ip])
    put_point(f"{prefix}/last", traj[-1])


def put_exc(prefix, ex):
    """RUNG 45's `phi_excursion_fuel` — all NINE keys, including the four no Python gate reads
    (`s_lp`/`s_hp`, `min_phi_hp`, `ratio`)."""
    assert len(ex) == 9, sorted(ex)
    for k in ("ext_lp", "ext_hp", "s_lp", "s_hp", "min_phi_lp", "min_phi_hp",
              "Tt4_peak", "ratio"):
        put(f"{prefix}/{k}", ex[k])
    putd(f"{prefix}/npts", ex["npts"])


def put_sm(prefix, sm):
    assert len(sm) == 11, sorted(sm)
    for k in ("margin_min_lp", "margin_min_hp", "steady_min_lp", "steady_min_hp",
              "min_phi_lp", "min_phi_hp", "phi_surge_lp", "phi_surge_hp"):
        put(f"{prefix}/{k}", sm[k])
    putd(f"{prefix}/crossed_lp", 1 if sm["crossed_lp"] else 0)
    putd(f"{prefix}/crossed_hp", 1 if sm["crossed_hp"] else 0)
    putd(f"{prefix}/npts", sm["npts"])


def put_fc(prefix, fc):
    assert len(fc) == 7, sorted(fc)
    for k in ("both", "lp", "hp", "d_lp", "d_hp", "r", "rho"):
        put(f"{prefix}/{k}", fc[k])


def put_cs(prefix, cs):
    assert len(cs) == 5, sorted(cs)
    for k in ("Tt4_peak", "E_temp", "E_lp", "E_hp", "f"):
        put(f"{prefix}/{k}", cs[k])


# ======================================================================== the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
# `test_rung43.py`'s SINGLE and `test_rung45.py`'s differ in TWO places, and both are copied from
# their own suite rather than from each other (step 3's `DTT4` lesson, applied to a losses dict).
SINGLE43 = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.92,
                eta_m=0.99, pi_n=0.98, nozzle_convergent=True)
SINGLE45 = dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92,
                eta_m=0.99, pi_n=0.98)

LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
TILTED = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)
FLAT = ComponentMap.flat()
SHAPES43 = {
    "flow/press": (LP_SHAPED, HP_SHAPED),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (TILTED, TILTED),
}
# `test_rung45.py` adds rung 40's DISCRIMINATOR shape — and it is the ONLY cell in either suite
# that reaches two of the high wall's three arms.
SHAPES45 = dict(SHAPES43)
SHAPES45["hp-only"] = (FLAT, HP_SHAPED)
LO, HI = 1250.0, 1450.0


def gas43():
    """`test_rung43.py:62` — `R_c` HARD-CODED at 286.9."""
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


def gas45():
    """`test_rung45.py:83` — `R_c` DERIVED as `(g-1)/g*cp = 286.857142857…`."""
    gc, cpc, gt, cpt = 1.4, 1004.0, 1.3, 1239.0
    return Gas(gamma_c=gc, cp_c=cpc, R_c=(gc - 1.0) / gc * cpc,
               gamma_t=gt, cp_t=cpt, R_t=(gt - 1.0) / gt * cpt, hPR=42.8e6)


def design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


D43 = design(gas43())
D45 = design(gas45())


def ft43(ml=LP_SHAPED, mh=HP_SHAPED, rho=1.0):
    return TwoSpoolFuelTransient(D43, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=rho)


def ft45(ml=LP_SHAPED, mh=HP_SHAPED, rho=1.0, lp_disabled=False):
    return TwoSpoolFuelTransient(D45, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=rho,
                                 lp_disabled=lp_disabled)


if ARM in ("main", "cpython"):
    # ------------------------------------------------------- A: the two suites' GASES
    # `R_c` reaches the fuel path through the static/exhaust conversion ALONE — every speed,
    # temperature, pressure ratio, flow coefficient and applied fuel is bit-identical between the
    # two recipes (§ 5.16 probe 1, 400 keys). So the bits certify the CONSTANT and only a THRUST
    # key certifies that the constant reached the physics. Both are here, per recipe.
    for tag, g in (("r43", gas43()), ("r45", gas45())):
        put(f"A/{tag}/R_c", g.R_c)
        put(f"A/{tag}/R_t", g.R_t)
        put(f"A/{tag}/cp_c", g.cp_c)
        put(f"A/{tag}/gamma_c", g.gamma_c)
    reset_census()
    for tag, f in (("r43", ft43()), ("r45", ft45())):
        i = f._instant_fuel(FLIGHT, 1.0, 1.0, 0.020)
        put(f"A/{tag}/sp_thrust", i["sp_thrust"])
        put(f"A/{tag}/Tt4", i["Tt4"])
        put(f"A/{tag}/nu_lpt", i["nu_lpt"])
    emit_census("A")

    # ------------------------------------------ B: RUNG 43 GATE 1 — control invariance
    # The reduce, non-tautological: a rung-40 Tt4-control point re-reached through the forward
    # BURNER. All 45 keys of the returned instant, plus the Newton pass count.
    f = ft43()
    for it, Tt4 in enumerate((1500.0, 1300.0, 1100.0)):
        eq = f.equilibrium(FLIGHT, Tt4)
        mf = eq["f"] * eq["mdot_air"]
        put(f"B/{it}/mf", mf)
        for k in ("nu_lp", "nu_hp", "pi_lpc", "pi_hpc", "Tt4", "mdot_air", "f",
                  "tau_hpt", "tau_lpt", "sp_thrust"):
            put(f"B/{it}/eq40/{k}", eq[k])
        put_eqf(f"B/{it}/eqf", f.equilibrium_fuel(FLIGHT, mf))
    emit_census("B")

    # ---------------------------------------------- C: RUNG 43 GATE 2 — lp_disabled
    # The degenerate object is built from the SINGLE-spool engine in this suite, and from a
    # TWO-spool one in `test_rung45.py`. Both constructions are dumped (C here, K below), because
    # Python is duck-typed across that difference and Rust is not.
    single43 = build_turbojet(gas43(), PI_HPC, TT4, FLIGHT.p0, **SINGLE43)
    st = SpoolTransient(single43, FLIGHT, 1.0, comp_map=HP_SHAPED)
    deg = TwoSpoolFuelTransient(single43, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED,
                                lp_disabled=True)
    reset_census()
    for it, Tt4 in enumerate((1500.0, 1300.0, 1150.0)):
        mf = st._fuel_for_Tt4(FLIGHT, Tt4)
        put(f"C/{it}/mf", mf)
        a, b = st.equilibrium_fuel(FLIGHT, mf), deg.equilibrium_fuel(FLIGHT, mf)
        for k in ("nu", "pi_c", "Tt4", "mdot_air", "f", "tau_t", "sp_thrust"):
            put(f"C/{it}/rung35/{k}", a[k])
            put(f"C/{it}/deg/{k}", b[k])
        putd(f"C/{it}/deg_passes", LAST_PASSES[0])
    emit_census("C")

    # ------------------------------------- D: RUNG 43 GATE 3 — rung 40's control untouched
    t40 = TwoSpoolTransient(D43, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED)
    f = ft43()
    put_cs("D/exercise", f.constant_speed_excursion_fuel(FLIGHT, LO, HI))
    for it, Tt4 in enumerate((1500.0, 1300.0, 1150.0)):
        a, b = t40.equilibrium(FLIGHT, Tt4), f.equilibrium(FLIGHT, Tt4)
        for k in ("nu_lp", "nu_hp", "pi_lpc", "pi_hpc", "Tt4", "mdot_air", "f",
                  "tau_hpt", "tau_lpt", "sp_thrust"):
            put(f"D/{it}/t40/{k}", a[k])
            put(f"D/{it}/ft/{k}", b[k])
    emit_census("D")

    # -------------------------------------------- E: RUNG 43 GATE 4 — the DYNAMICAL reduce
    # The one gate in either suite that asserts a NUMBER (step 2 finding 2), and the longest
    # march in the file — 701 points at `s_end = 14.0`.
    f = ft43()
    mf_hi = f.fuel_for_Tt4(FLIGHT, HI)
    eq_hi, eq_lo = f.equilibrium(FLIGHT, HI), f.equilibrium(FLIGHT, LO)
    put("E/mf_hi", mf_hi)
    for k in ("nu_lp", "nu_hp", "Tt4"):
        put(f"E/eq_hi/{k}", eq_hi[k])
        put(f"E/eq_lo/{k}", eq_lo[k])
    traj = f.integrate_fuel(FLIGHT, lambda s: mf_hi,
                            (eq_lo["nu_lp"], eq_lo["nu_hp"]), 14.0, 0.02)
    putd("E/npts", len(traj))
    put_point("E/first", traj[0])
    put_point("E/last", traj[-1])
    # …and the interior, every 100th point: the settle is where a wrong derivative that still
    # lands on the right fixed point would hide, and gate 4 reads only `traj[-1]`.
    for ip in range(0, len(traj), 100):
        put_point(f"E/at/{ip}", traj[ip])
    emit_census("E")

    # ----------------------------------------------- F: RUNG 43 GATE 5 — freeze_channels
    # 12 cells, each THREE marches (both / LP frozen / HP frozen). The `r = 0.25` half sits on
    # the `412.5` tie.
    for name in ("flow/press", "tilted"):
        ml, mh = SHAPES43[name]
        for rho in (0.5, 1.0, 2.0):
            g = ft43(ml, mh, rho)
            for r in (0.25, 1.0):
                put_fc(f"F/{name}/{rho}/{r}", g.freeze_channels(FLIGHT, LO, HI, r))
    emit_census("F")

    # ------------------------------------------------ G: RUNG 43 GATE 6 — the rho-free ceiling
    for name in ("flow/press", "tilted"):
        ml, mh = SHAPES43[name]
        for r in (0.25, 1.0):
            for rho in (1.0, 7.0, 50.0):
                put_ramp(f"G/{name}/{r}/ceil/{rho}",
                         ft43(ml, mh, rho).ramp_excursion_fuel(FLIGHT, LO, HI, r, freeze="lp"))
            for rho in (1.0, 8.0, 32.0):
                put_ramp(f"G/{name}/{r}/free/{rho}",
                         ft43(ml, mh, rho).ramp_excursion_fuel(FLIGHT, LO, HI, r))
    emit_census("G")

    # ------------------------------------------------- H: RUNG 43 GATE 7 — rho-monotonicity
    for name, (ml, mh) in SHAPES43.items():
        for r in (0.25, 1.0):
            for rho in (0.25, 0.5, 1.0, 2.0, 4.0):
                put_ramp(f"H/{name}/{r}/{rho}",
                         ft43(ml, mh, rho).ramp_excursion_fuel(FLIGHT, LO, HI, r))
    emit_census("H")

    # ----------------------------------------- I: RUNG 43 GATE 8 — the r -> 0 limit, rho-free
    for name, (ml, mh) in SHAPES43.items():
        put_cs(f"I/{name}/base", ft43(ml, mh).constant_speed_excursion_fuel(FLIGHT, LO, HI))
        put_cs(f"I/{name}/rho0.2", ft43(ml, mh, 0.2).constant_speed_excursion_fuel(FLIGHT, LO, HI))
        put_cs(f"I/{name}/rho5.0", ft43(ml, mh, 5.0).constant_speed_excursion_fuel(FLIGHT, LO, HI))
    emit_census("I")

    # ---------------------------------------- J: RUNG 43 GATE 9 — the withdrawn clock, on the TIE
    # All four `r = 0.25` cells here land on `8.25/0.02 = 412.5` exactly, where `round_ties_even`
    # gives 412 and `f64::round` 413 — and § 5.16 prediction 4 measured every reported value
    # BLIND to the extra point. `npts` is the only key that sees it.
    ml, mh = SHAPES43["flow/press"]
    pts = []
    for rho in (0.25, 1.0, 4.0, 8.0):
        g = ft43(ml, mh, rho)
        for r in (0.25, 0.5, 1.0, 2.0):
            e = g.ramp_excursion_fuel(FLIGHT, LO, HI, r)
            put_ramp(f"J/{rho}/{r}", e)
            if e["complete"]:
                pts.append((r, rho, e))
    putd("J/n_points", len(pts))
    for key in ("E_temp_H", "X", "E_temp_L"):
        q, sp = TwoSpoolFuelTransient.collapse_exponent(pts, key)
        put(f"J/collapse/{key}/q", q)
        put(f"J/collapse/{key}/spread", sp)
        # …and the WHOLE scored curve, which is what makes the argmin's TIE visible: the
        # minimum is attained by two adjacent `q` at a gap of exactly 0.000e+00, and rung 43's
        # own gate 9 cannot see which of the two the fold returns.
        for i in range(0, 25):
            _, s_i = TwoSpoolFuelTransient.collapse_exponent(pts, key, q=i / 20.0)
            put(f"J/collapse/{key}/score/{i}", s_i)
    emit_census("J")

    # ============================================================== RUNG 45
    # ------------------------------------------------ K: GATE 1 — read-only, and lp_disabled
    for name in ("flow/press", "tilted"):
        ml, mh = SHAPES45[name]
        bare = ft45(ml, mh)
        armed = ft45(ml.with_phi_surge(0.70), mh.with_phi_surge(0.55))
        for tag, o in (("bare", bare), ("armed", armed)):
            mfh = o.fuel_for_Tt4(FLIGHT, HI)
            e0 = o.equilibrium(FLIGHT, LO)
            t = o.integrate_fuel(FLIGHT, lambda s, m=mfh: m,
                                 (e0["nu_lp"], e0["nu_hp"]), 2.0, 0.02)
            putd(f"K/{name}/{tag}/npts", len(t))
            put_point(f"K/{name}/{tag}/last", t[-1])
            put_eqf(f"K/{name}/{tag}/eqf", o.equilibrium_fuel(FLIGHT, mfh))
    # …and the SECOND lp_disabled construction — from a TWO-spool design, which `test_rung43.py`
    # never builds. Both rung-45 methods must REFUSE it, and the escaping error's identity is the
    # only thing there is to compare, so it is compared.
    #
    # ONLY THE ARMED OBJECT IS DUMPED, AND THE UNARMED ONE IS A DISCLOSED PORT DIVERGENCE. On an
    # UNARMED degenerate object Python's `transient_surge_margin_fuel` raises the SURGE-LINE
    # assert (kind 5), because its own body reads `self.map_lp/map_hp` and checks `phi_surge`
    # BEFORE `_fuel_ramp_march`'s two-shaft refusal (kind 6) can fire. Rust cannot reproduce that
    # order and it is not a bug in the port: step 2 finding 4 recorded that EVERY `lp_disabled`
    # constructor in the project takes `map_hp` ALONE, so the degenerate variant has no `map_lp`
    # to read a `phi_surge` off and must refuse on degeneracy first. `fuel_transient_oracle.rs`
    # asserts the Rust side of that divergence directly, beside this sentence's twin.
    o = ft45(LP_SHAPED.with_phi_surge(0.6), HP_SHAPED.with_phi_surge(0.55), lp_disabled=True)
    for meth in ("phi_excursion_fuel", "transient_surge_margin_fuel"):
        try:
            getattr(o, meth)(FLIGHT, 1000.0, 1400.0)
            putd(f"K/deg/armed/{meth}", -1)
        except AssertionError as exc:
            putd(f"K/deg/armed/{meth}", kind_of(exc))
    # …and the UNARMED object's, recorded so the divergence has a NUMBER in the golden rather
    # than only a sentence. The Rust never compares this key; it asserts its own answer (6) and
    # names this one (5). It is spelled `pyonly` so the comparator's never-compared half stays a
    # real gate everywhere else.
    o = ft45(LP_SHAPED, HP_SHAPED, lp_disabled=True)
    for meth in ("phi_excursion_fuel", "transient_surge_margin_fuel"):
        try:
            getattr(o, meth)(FLIGHT, 1000.0, 1400.0)
            putd(f"K/deg/pyonly/{meth}", -1)
        except AssertionError as exc:
            putd(f"K/deg/pyonly/{meth}", kind_of(exc))
    emit_census("K")

    # ------------------------------- L: GATE 2 — the four shapes, PER-CELL census
    # THE ONLY CELL IN EITHER SUITE THAT REACHES TWO OF THE HIGH WALL'S THREE ARMS is `hp-only`,
    # whose LP map is `ComponentMap.flat()`. The census is emitted per cell here, not per
    # section, because a section total would let one shape's 228 801 map hits bury the other's
    # 1 301 literal ones.
    for name, (ml, mh) in SHAPES45.items():
        o = ft45(ml, mh)
        t2 = TwoSpoolTransient(D45, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        put_exc(f"L/{name}/acc", o.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=0.5))
        emit_census(f"L/{name}/acc")
        put_exc(f"L/{name}/dec", o.phi_excursion_fuel(FLIGHT, 1400.0, 1000.0, r=0.5))
        emit_census(f"L/{name}/dec")
        # rung 44's SAME-MAPS comparison. `400.0` is a DELTA against rung 45's ENDPOINT `1400.0`
        # — step 3's `DTT4` hazard, which exactly one of `rung45.rs`'s ten tests catches.
        ex44 = t2.phi_excursion(FLIGHT, 1000.0, 400.0, r_ramp=0.5)
        for k in ("ext_lp", "ext_hp", "min_phi_lp", "min_phi_hp"):
            put(f"L/{name}/t40/{k}", ex44[k])
        putd(f"L/{name}/t40/npts44", ex44["npts"])
        emit_census(f"L/{name}/t40")

    # ------------------------------------------- M: GATE 3(a) — the five-rho sweep
    for rho in (0.2, 0.5, 1.0, 2.0, 5.0):
        put_exc(f"M/{rho}", ft45(LP_SHAPED, HP_SHAPED, rho)
                .phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=0.5))
    emit_census("M")

    # ------------------------ N: GATE 3(b) — the 19-point running line and the OUTPUT reference
    # The most bespoke computation in either suite (step 3 finding 4): a hand-written 19-point
    # interp, its own 326-step march per `rho`, and three spreads gated only by `< 0.02`,
    # `> 0.20` and an ordering. Every one of the 19 grid values is a key.
    ftg = ft45(LP_SHAPED, HP_SHAPED)
    grid = [1000.0 + 50.0 * k for k in range(19)]
    ys_l = [ftg.equilibrium(FLIGHT, T)["phi_lp"] for T in grid]
    for ik, y in enumerate(ys_l):
        put(f"N/grid/{ik}", y)
    emit_census("N/grid")

    def interp(x):
        if x <= grid[0]:
            return ys_l[0]
        if x >= grid[-1]:
            return ys_l[-1]
        for i in range(len(grid) - 1):
            if grid[i] <= x <= grid[i + 1]:
                t = (x - grid[i]) / (grid[i + 1] - grid[i])
                return ys_l[i] + t * (ys_l[i + 1] - ys_l[i])
        return ys_l[-1]

    for rho in (0.2, 1.0, 5.0):
        o = ft45(LP_SHAPED, HP_SHAPED, rho)
        put_exc(f"N/{rho}/exc", o.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=0.5))
        mf_lo, mf_hi2 = o.fuel_for_Tt4(FLIGHT, 1000.0), o.fuel_for_Tt4(FLIGHT, 1400.0)
        put(f"N/{rho}/mf_lo", mf_lo)
        put(f"N/{rho}/mf_hi", mf_hi2)
        eq0 = o.equilibrium(FLIGHT, 1000.0)
        nu0 = (eq0["nu_lp"], eq0["nu_hp"])

        def sched(s, a=mf_lo, b=mf_hi2):
            return a + (b - a) * min(1.0, s / 0.5)

        t = o.integrate_fuel(FLIGHT, sched, nu0, 6.5, 0.02)
        putd(f"N/{rho}/out/npts", len(t))
        oe, i_oe = 0.0, -1
        for ip, p in enumerate(t):
            e_lp = p["phi_lp"] - interp(p["Tt4"])
            if abs(e_lp) > abs(oe):
                oe, i_oe = e_lp, ip
        put(f"N/{rho}/out_ext", oe)
        putd(f"N/{rho}/i_out_ext", i_oe)
        put_point(f"N/{rho}/out_at", t[i_oe])
        emit_census(f"N/{rho}")

    # ----------------------------------- O: GATE 4 — fuel vs Tt4 control at three ramp rates
    o = ft45(LP_SHAPED, HP_SHAPED)
    t2 = TwoSpoolTransient(D45, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
    for r in (1.0, 0.5, 0.3):
        put_exc(f"O/fuel/{r}", o.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=r))
        ex44 = t2.phi_excursion(FLIGHT, 1000.0, 400.0, r_ramp=r)
        put(f"O/t40/{r}/min_phi_lp", ex44["min_phi_lp"])
        putd(f"O/t40/{r}/npts44", ex44["npts"])
    emit_census("O")

    # ------------------------------------------------ P: GATE 5 — the ramp-rate sweep
    # THE `npts` GATE. Step 3 finding 3 measured `min_phi_lp` BIT-IDENTICAL at all four rates
    # with the march bound's `r` deleted, and only these four lengths (351/326/316/306 vs a flat
    # 301) witnessing it.
    o = ft45(LP_SHAPED, HP_SHAPED)
    for r in (1.0, 0.5, 0.3, 0.1):
        put_exc(f"P/{r}", o.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=r))
    emit_census("P")

    # ---------------------------------------------------- Q: GATE 6 — the crossing
    o = ft45(LP_SHAPED.with_phi_surge(0.746), HP_SHAPED.with_phi_surge(0.55))
    put_sm("Q/acc", o.transient_surge_margin_fuel(FLIGHT, 1000.0, 1400.0, r=0.3))
    bare = ft45(LP_SHAPED, HP_SHAPED)
    try:
        bare.transient_surge_margin_fuel(FLIGHT, 1000.0, 1400.0)
        putd("Q/unarmed", -1)
    except AssertionError as exc:
        putd("Q/unarmed", kind_of(exc))
    emit_census("Q")

    # ----------------------------- R: GATE 1/cycle's own fuel-path calls
    # The `Gas.reacting_equilibrium()` design run those two gates sandwich is `cycle_oracle.rs`'s
    # channel, not this file's — step 3 recorded that nothing in the fuel path can perturb a
    # single-spool cycle. What IS this file's is the pair of calls made BETWEEN the two runs.
    o = ft45(LP_SHAPED.with_phi_surge(0.60), HP_SHAPED.with_phi_surge(0.55))
    put_exc("R/exc", o.phi_excursion_fuel(FLIGHT, 1000.0, 1300.0, r=0.5))
    put_sm("R/sm", o.transient_surge_margin_fuel(FLIGHT, 1000.0, 1300.0, r=0.5))
    emit_census("R")

    # ---------------------------------------------- S: the accel schedule and `_interp`
    # `accel_schedule` is reached by NO phase-6 gate (probe 2), so the smoke is its only
    # coverage; it is repeated here on the oracle's own object because `_interp`'s three arms are
    # counted in this file and section H is the only thing that drives them.
    f = ft43()
    acc = f.accel_schedule(FLIGHT, LO, HI, margin=0.15, n=5)
    putd("S/n", len(acc.n_H))
    for i, (nh, kappa) in enumerate(zip(acc.n_H, acc.kappa)):
        put(f"S/n_H/{i}", nh)
        put(f"S/kappa/{i}", kappa)
    put("S/margin", acc.margin)
    # …and the table READ, on all three of `cap`'s arms — a SEPARATE function from `_interp`
    # in Python (their fall-throughs differ) and kept separate in Rust.
    for i, nh in enumerate((acc.n_H[0] * 0.9, acc.n_H[0], 0.5 * (acc.n_H[0] + acc.n_H[-1]),
                            acc.n_H[-1], acc.n_H[-1] * 1.1)):
        put(f"S/read/{i}", acc.cap(nh, 250_000.0))
    emit_census("S")

if ARM in ("gas", "cpython"):
    # ============================================== THE NON-CPG GASES
    # An ADDED arm: neither suite runs a TPG gas through the fuel path. It is here because
    # § 5.16 probe 3 measured `equilibrium_fuel`'s Newton pass count — an ABSOLUTE `1e-12` bar
    # sitting under the gas sub-solve's own ~1e-10 noise — swinging 16-fold between interpreters,
    # which makes it the sharpest single detector in the slice. Gated bit-exact against PyPy;
    # published as a deviation distribution against CPython and NEVER summed with the CPG half.
    for name, factory in (("tpg", Gas.thermally_perfect), ("reacting", Gas.reacting),
                          ("forkb", Gas.reacting_forkb)):
        g = factory()
        putd(f"T/{name}/equilibrium_flag", 1 if g.equilibrium else 0)
        d = build_two_spool_turbojet(g, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                     nozzle_convergent=True, **REAL)
        f = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
        reset_census()
        for it, Tt4 in enumerate((1300.0, 1400.0, 1450.0, 1500.0)):
            mf = f.fuel_for_Tt4(FLIGHT, Tt4)
            put(f"T/{name}/{it}/mf", mf)
            put_eqf(f"T/{name}/{it}/eqf", f.equilibrium_fuel(FLIGHT, mf))
        emit_census(f"T/{name}/eqf")
        # …and a SHORT march, so the fragile set covers the marcher and not only the Newton.
        mf0 = f.fuel_for_Tt4(FLIGHT, 1300.0)
        mf1 = f.fuel_for_Tt4(FLIGHT, 1450.0)
        eq0 = f.equilibrium(FLIGHT, 1300.0)

        def sched(s, a=mf0, b=mf1):
            return a + (b - a) * min(1.0, s / 0.5)

        t = f.integrate_fuel(FLIGHT, sched, (eq0["nu_lp"], eq0["nu_hp"]), 1.0, 0.02)
        putd(f"T/{name}/npts", len(t))
        for ip in range(0, len(t), 10):
            put_point(f"T/{name}/at/{ip}", t[ip])
        put_point(f"T/{name}/last", t[-1])
        emit_census(f"T/{name}/march")

    # ----------------------------------------- U: THE REFUSAL, on the gas the fuel path REJECTS
    # `_tt4_from_f` asserts `not self.gas.equilibrium`, and `_close_fuel` SWALLOWS that assertion
    # inside its bracket scan — so what ESCAPES is the BRACKET error, naming a cause that is not
    # the actual one. What Python CAN see is which error escapes; the 46 swallowed advances
    # (38 the refusal, 8 `inverse: root not bracketed`, 0 off-map) are Rust-side counters, gated
    # against `probe_s6.py`'s instrumentation of the SHIPPED body.
    geq = Gas.reacting_equilibrium()
    putd("U/equilibrium_flag", 1 if geq.equilibrium else 0)
    deq = build_two_spool_turbojet(geq, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                   nozzle_convergent=True, **REAL)
    feq = TwoSpoolFuelTransient(deq, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
    reset_census()
    # A census PER CALL, because `probe_s6.py` measured the 46 on ONE call and a section total
    # would let the second call's advances hide inside it — the same argument section L makes for
    # the high wall's rare arms, one section on.
    #
    # AND THE FIRST CELL IS THE SMOKE'S OWN FUEL FLOW, DELIBERATELY. § 5.16 and step 1 both record
    # the swallowed advances as "46 — 38 the refusal, 8 `inverse`", which is true of
    # `fuel_for_Tt4(1400)` and of no other flow: at 0.020 the same 46 splits **39 / 7**, and at
    # 0.017 it is **47 = 40 / 7**. The split is a property of the CELL — where the trial band that
    # runs the HP face past `psi < 0` starts relative to the march-in grid — so all three are
    # driven here and each is gated on its own numbers. *A census is a property of the grid, for
    # the fourth time in this slice.*
    mfeq = feq.fuel_for_Tt4(FLIGHT, 1400.0)     # Tt4-control: allowed on EVERY gas
    put("U/mf_smoke", mfeq)
    emit_census("U/setup")
    for it, mf in enumerate((mfeq, 0.020, 0.017)):
        try:
            feq.equilibrium_fuel(FLIGHT, mf)
            putd(f"U/eqf/{it}", -1)
        except AssertionError as exc:
            putd(f"U/eqf/{it}", kind_of(exc))
        emit_census(f"U/{it}")
    # …and the DIRECT poke, which raises the refusal ITSELF rather than the bracket error.
    try:
        feq._tt4_from_f(700.0, 0.02)
        putd("U/direct", -1)
    except AssertionError as exc:
        putd("U/direct", kind_of(exc))
    emit_census("U/direct")

# =========================================================================== emit
out = open(OUT, "w", encoding="utf-8", newline="\n") if OUT else sys.stdout
out.write(f"# slice S oracle ({ARM}) — rungs 43+45 TwoSpoolFuelTransient — "
          "key\tu64 bits (or an integer)\trepr\n")
for key, bits, text in ROWS:
    out.write(f"{key}\t{bits}\t{text}\n")
if OUT:
    out.close()
sys.stderr.write(f"[dump_fuel_transient/{ARM}] {len(ROWS)} values\n")
