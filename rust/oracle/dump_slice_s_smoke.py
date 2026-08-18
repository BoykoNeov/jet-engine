"""SLICE S step 1 — the SMOKE dump for rungs 43 + 45 (`TwoSpoolFuelTransient`).

Not the slice's oracle (that is step 4, on both suites' full grids). This exists to catch a
structural mistake BEFORE the 20 Python gates are ported on top of it at steps 2 and 3 — and
§ 5.16's four probes named the mistakes in advance, each of which the shipped Rust deliberately
does NOT make:

  1. `_close_fuel` ported by analogy to rung 40's `_close`. They differ in SIX places (three-arm
     high wall vs two, `f_cap = 0.065` + an `f_floor` vs neither, `step = 0.04` vs `0.02`, a
     last-negative/first-positive scan vs break-at-first-success, `ghi` produced inside the try vs
     outside, and no sign filter on the final guard) and rung 35's single-spool fuel closure is a
     THIRD thing again. Sections A and B drive the closure directly and the census reads out all
     three high-wall arms — and section L exists because A–K never bind the third one;
  2. `round_ties_even` replaced by `f64::round`. Rung 43's ramps put `8.25/0.02 = 412.5` EXACTLY
     on the tie, where Python gives 412 and Rust's `round` gives 413. § 5.16 measured every
     reported value BLIND to the extra step, so section F runs AT `r = 0.25` and dumps `npts`
     beside the values: the length is the only key that can see it;
  3. the four Illinois tolerances collapsed to one. `1e-12` closes the flow, `1e-9` the topping
     set point, `1e-13` BOTH legs — sections C and I are where a shared constant would show;
  4. the float-IDENTITY branches turned into arithmetic. `faded` at `w >= 1.0` returns the cap
     ITSELF, the two legs return `mf_sched` ITSELF when dormant, and `_release_weight`
     short-circuits to exactly 1.0 / 0.0 for a FALSY `tau_rel` — which is `None` **or** `0.0`.
     Section I arms all ten cases and reads out the three weight arms;
  5. `_integrate_fuel_asym`'s 16-key point compared against a 14-key struct. Section I enumerates
     Python's key set PER ROUTE and asserts each count, because a field missing from that route
     would otherwise be missing from the comparison too.

THE CELLS, each touching a path ONCE:

  A  the TWO SUITES' GASES, and the closure driven DIRECTLY — all 23 of `_close_fuel`'s dict keys
     by name, at two speeds. `test_rung43.py:62` hard-codes `R_c = 286.9` and `test_rung45.py:83`
     derives `286.857142857…`; § 5.16 measured their whole 400-key fuel-path dump BIT-IDENTICAL,
     with the difference reaching exactly one channel — the static/exhaust conversion, i.e. the
     THRUST. So this section carries both recipes' `R_c`/`R_t` bits AND a thrust key per recipe;
     without the thrust key a bit-exact dump certifies nothing about which gas the port used.
  B  `_instant_fuel` — all 45 keys, the count asserted from PYTHON's own `len()` rather than
     derived from 23 + 23.
  C  `equilibrium_fuel` on CPG (including the explicit-`start` signature branch), and then on the
     three non-equilibrium TPG gases as a SEPARATE arm with the Newton PASS COUNT as a discrete
     key. § 5.16 probe 3 measured that count swinging 16-fold between interpreters on a residual
     that plateaus just under an ABSOLUTE bar, which makes it the sharpest detector in the slice.
  D  `fuel_for_Tt4`, the DERIVED accel schedule's whole table, and `_interp` on all three arms.
  E  `integrate_fuel` bare — EVERY point, EVERY field, on a short march.
  F  `ramp_excursion_fuel` at the `r = 0.25` TIE and at a non-tie control, with `npts`.
  G  `freeze_channels`, `constant_speed_excursion_fuel`, and `collapse_exponent` — whose argmin
     sits on a PLATEAU (every currency's minimum attained by two adjacent `q` at a gap of exactly
     0.000e+00) and whose first-of-equals tie-break rung 43's own gate 9 cannot see.
  H  RUNG 45 — `phi_excursion_fuel` and `transient_surge_margin_fuel` against armed maps.
  I  THE ARMED LIMITER CASES — probe 4 (B)'s NINE, plus a TENTH this dump adds because the
     nine between them never CONTEST a `min`. Probe 2 measured that NO phase-6 gate arms a single
     one of these keywords, so these sections are the entire coverage of ~40 % of the source.
  J  THE REFUSAL, reached through an ordinary entry point on `Gas.reacting_equilibrium()` — where
     the error that ESCAPES is the BRACKET one, naming a cause that is not the actual one.
  K  the `lp_disabled` REDUCE against a bare rung-35 `SpoolTransient`, which must be BIT-identical.
  L  THE THIRD HIGH-WALL ARM, which sections A–K never bind — and, past it, the only CPG cell in
     this file where the closure FAILS TO BRACKET.

WHAT PYTHON CANNOT SEE, AND WHY IT IS NOT FAKED HERE. The closure SWALLOWS every failure of its
own bracket scan, so the march-in advances and their classification cannot be counted from
outside the body — and copying the body into this file to count them would make the dump's
arithmetic a COPY rather than the shipped code (slice R's rule, and slice O's). Those stay
Rust-side counters, gated against numbers measured by `probe_s6.py`, which instruments the
SHIPPED body by textual substitution: **46 swallowed on the equilibrium gas — 38 the refusal and
8 `inverse: root not bracketed` out of the HPC ideal temperature** — and 0 on every CPG grid.
Section J compares what Python CAN see there: which error escapes.

THE CENSUS KEYS ARE SENSITIVE TO STATEMENT POSITION between `emit_census` boundaries — a call
added between two of them silently moves twenty keys and needs a golden regeneration. That is not
a defect; it is how this dump caught the Rust resetting its counters one statement too early and
hiding 39 Illinois calls that Python's `E/bare` section legitimately carries.

Regenerate with:
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_s_smoke.py > rust/oracle/slice_s_smoke_pypy.tsv
"""
import os
import struct
import sys
from fractions import Fraction

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import engine as E                                          # noqa: E402
from turbojet.engine import (AsymmetricLag, ComponentMap,                 # noqa: E402
                             FlightCondition, SpoolTransient, SurgeLimiter,
                             TwoSpoolFuelTransient, build_turbojet,
                             build_two_spool_turbojet)
from turbojet.gas import Gas                                              # noqa: E402

ROWS = []


def put(key, value):
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putd(key, n):
    n = int(n)
    ROWS.append((key, n, str(n)))


def fnv1a(name):
    """FNV-1a over an ASCII name — so an extra key RENAMED or REORDERED in the port fails a value
    comparison instead of riding in a text column the comparator never reads."""
    h = 0xCBF29CE484222325
    for ch in name.encode("ascii"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def kind_of(exc):
    """Which arm produced an `AssertionError`, by the substrings the Rust classifies on too."""
    s = str(exc)
    return (0 if "non-equilibrium" in s else 1 if "inverse: root not bracketed" in s
            else 2 if "off-map compressor trial" in s else 3 if "does not bracket" in s else 4)


# ================================================================= the instruments
CENSUS = {}
# TWO OF THE RUST COUNTERS ARE DELIBERATELY ABSENT HERE. `der_calls` counts a LOCAL closure Python
# offers no handle on, and `march_points` counts an append this file already sees as `npts`.
# Emitting them as constant zeros would make the comparison a gate that FAILS on a correct port;
# leaving them out makes the Rust side state its own relation to the keys that ARE here
# (`march_points == npts`, `der_calls == 4*npts` on an unbroken march), which is a claim about the
# marcher's shape rather than a copied number.
KEYS = dict(close_calls=0, instant_calls=0, eq_calls=0, eq_passes=0, march_calls=0,
            topping_calls=0, sched_calls=0, sched_dormant=0,
            surge_calls=0, surge_dormant=0, rw_calls=0, rw_one=0, rw_interior=0, rw_zero=0,
            hi_wall_literal=0, hi_wall_map=0, hi_wall_hi0=0,
            illinois_calls=0, illinois_evals=0, illinois_exhausted=0)


def reset_census():
    CENSUS.clear()
    CENSUS.update(KEYS)
    INSTANT_N[:] = [0]


INSTANT_N = [0]        # `_instant_fuel` calls, for recovering the Newton pass count

reset_census()

_ILL = E._illinois


def _ill(f, a, b, fa, fb, tol=1e-10, maxit=100):
    """The shipped Illinois, counted. `illinois_exhausted` is the same counter slice Q measured at
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
    """Counts the call, classifies the THREE-ARM HIGH WALL, and calls the SHIPPED body.

    INSTRUMENT NOTE, and it is the same one slice R's dump wrote for rung 40's two-arm wall:
    `n_lp` and `hi0` are recomputed here — a handful of operations the body also performs —
    purely to classify which arm of `min(2.5, phi_max*n_L, hi0)` binds. They feed NO dumped
    value; every number in this file comes out of the shipped body.

    IT EARNS THE DUPLICATION BECAUSE NOTHING ELSE CAN SEE THE THIRD ARM. That arm is rung 43's
    most prominent departure from rung 40's closure, and a partition-sum check
    (`literal + map + hi0 == calls`) passes identically whether it binds or is ABSENT — so a port
    that dropped it would be invisible. Classified here, it becomes a compared key.

    Python's `min(a, b, c)` is a FOLD, not a pairwise `min`, and is spelled as one: first wins
    on a tie.
    """
    CENSUS["close_calls"] += 1
    n_lp = nu_lp * (self.Tt2_d / Tt2) ** 0.5
    hi0 = mdot_fuel * Tt2 ** 0.5 / (0.004 * self.mcorr_lp_d * pt2)
    cap, arm = 2.5, "hi_wall_literal"
    wall_map = self.map_lp.phi_max() * n_lp
    if wall_map < cap:
        cap, arm = wall_map, "hi_wall_map"
    if hi0 < cap:
        arm = "hi_wall_hi0"
    CENSUS[arm] += 1
    return _CLOSE_FUEL(self, nu_lp, nu_hp, mdot_fuel, Tt2, pt2)


TwoSpoolFuelTransient._close_fuel = _close_fuel_counted

_count(TwoSpoolFuelTransient, "_topping_fuel", "topping_calls")
_count(TwoSpoolFuelTransient, "integrate_fuel", "march_calls")


def _instant_extra(self, out, a, k):
    INSTANT_N[0] += 1


_count(TwoSpoolFuelTransient, "_instant_fuel", "instant_calls", _instant_extra)


def _leg_extra(key):
    def extra(self, out, a, k):
        # The DORMANT branch returns `mf_sched` ITSELF — a float compared with itself, which is
        # what § 5.16 measured as an EXACT structural zero. `is` catches it without arithmetic.
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
    """Records the Newton PASS COUNT, recovered from `_instant_fuel` calls rather than from inside
    the loop: each pass costs 3 (the residual + two Jacobian columns), the check that EXITS costs
    the first of those, and the returned instant costs one more —

        p completed passes  =>  3p + 2 calls

    and `3p + 2` is never a multiple of 3, so the recovery is unambiguous. § 5.16 probe 3 measured
    this count differing between interpreters in 6 of 6 TPG cells (PyPy 2 against CPython 33 in
    one), which is why it is a DISCRETE key and not a diagnostic.
    """
    n0 = INSTANT_N[0]
    CENSUS["eq_calls"] += 1
    out = _EQ(self, flight, mdot_fuel, start)
    if getattr(self, "_degenerate", None) is not None:
        # lp_disabled FORWARDS to rung 35's own solve, which never touches this class's
        # `_instant_fuel` — there is no two-shaft Newton to count. Section K's gate is the
        # bit-equality against a bare SpoolTransient, not a pass count.
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


# ================================================================= the dict emitters
# THE KEY LISTS COME FROM PYTHON'S OWN DICTS, never from the Rust struct: a field forgotten in the
# port must show up as a MISSING comparison, and it only can if the dump enumerates the source.
CLOSE_KEYS = 23          # rung 40's 21 + Tt4 + mdot_air_face
INSTANT_KEYS = 45        # rung 40's 44 + mdot_air_face


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


POINT_KEYS_BARE = 14
POINT_KEYS_ASYM = 16
BARE_ORDER = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
              "mdot_air", "sp_thrust", "branch", "mf", "mf_sched")


def put_point(prefix, p):
    """One marched point, with the ROUTE's key set enumerated and its count asserted."""
    extra = sorted(set(p) - set(BARE_ORDER))
    assert not (set(BARE_ORDER) - set(p)), sorted(set(BARE_ORDER) - set(p))
    for k in BARE_ORDER:
        if k == "branch":
            putd(f"{prefix}/branch_choked", 1 if p[k] == "choked" else 0)
        else:
            put(f"{prefix}/{k}", p[k])
    for k in extra:
        put(f"{prefix}/{k}", p[k])
    return len(p), tuple(extra)


def put_traj(prefix, pts):
    putd(f"{prefix}/npts", len(pts))
    nkeys, extra = None, None
    for ip, p in enumerate(pts):
        nkeys, extra = put_point(f"{prefix}/{ip}", p)
    putd(f"{prefix}/point_keys", nkeys)
    putd(f"{prefix}/extra_keys", len(extra))
    # THE EXTRA KEYS' NAMES, as a value the comparator can actually CHECK. Emitting the name in
    # the text column alone would leave these rows uncompared — and a route whose extra fields
    # were named or ORDERED differently is exactly what this is here to catch.
    for ie, k in enumerate(extra):
        putd(f"{prefix}/extra/{ie}", fnv1a(k))
    return nkeys


# ======================================================================== the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
SINGLE = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.92,
              eta_m=0.99, pi_n=0.98, nozzle_convergent=True)

LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
LO, HI = 1250.0, 1450.0          # rung 35's own step — apples-to-apples


def gas43():
    """`test_rung43.py:62` — `R_c` HARD-CODED at 286.9."""
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


def gas45():
    """`test_rung45.py:83` — `R_c` DERIVED as `(g-1)/g*cp = 286.857142857…`.

    The two gases' whole fuel-path dump is bit-identical; only the thrust key witnesses the
    difference, which is why each recipe carries one below."""
    gc, cpc, gt, cpt = 1.4, 1004.0, 1.3, 1239.0
    return Gas(gamma_c=gc, cp_c=cpc, R_c=(gc - 1.0) / gc * cpc,
               gamma_t=gt, cp_t=cpt, R_t=(gt - 1.0) / gt * cpt, hPR=42.8e6)


def design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def ft(gas, ml=LP_SHAPED, mh=HP_SHAPED, rho=1.0):
    return TwoSpoolFuelTransient(design(gas), FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=rho)


F43 = ft(gas43())
F45 = ft(gas45())

# ------------------------------------------- A: the two gases, and the closure driven DIRECTLY
for tag, g in (("r43", gas43()), ("r45", gas45())):
    put(f"A/{tag}/R_c", g.R_c)
    put(f"A/{tag}/R_t", g.R_t)
    put(f"A/{tag}/cp_c", g.cp_c)
    put(f"A/{tag}/gamma_c", g.gamma_c)
Tt2, pt2, V0 = F43._inlet(FLIGHT)
put("A/tt2", Tt2)
put("A/pt2", pt2)
put("A/v0", V0)
reset_census()
for ic, (nu_lp, nu_hp, mf) in enumerate(((1.0, 1.0, 0.020), (0.92, 0.96, 0.017))):
    put_close(f"A/{ic}", F43._close_fuel(nu_lp, nu_hp, mf, Tt2, pt2))
emit_census("A")

# THE ONE CHANNEL THAT WITNESSES THE GAS. `R_c` reaches the fuel path only through the
# static/exhaust conversion; every speed, temperature, pressure ratio, flow coefficient and
# applied fuel is bit-identical between the two recipes. Without this key a 400-key bit-exact
# dump certifies nothing about which gas the port built.
for tag, f in (("r43", F43), ("r45", F45)):
    i = f._instant_fuel(FLIGHT, 1.0, 1.0, 0.020)
    put(f"A/{tag}/sp_thrust", i["sp_thrust"])
    put(f"A/{tag}/Tt4", i["Tt4"])
    put(f"A/{tag}/nu_lpt", i["nu_lpt"])
emit_census("A/thrust")

# --------------------------------------------------------------- B: the instant, all 45 keys
for ic, (nu_lp, nu_hp, mf) in enumerate(((1.0, 1.0, 0.020), (0.94, 0.97, 0.0235))):
    put_instant(f"B/{ic}", F43._instant_fuel(FLIGHT, nu_lp, nu_hp, mf))
emit_census("B")

# --------------------------------------------------------- C: the 2-D Newton at fixed FUEL
# THE REDUCE (gate 1): a rung-40 Tt4-control point, re-reached through the forward BURNER.
for it, Tt4 in enumerate((1500.0, 1300.0, 1100.0)):
    eq0 = F43.equilibrium(FLIGHT, Tt4)
    mf = eq0["f"] * eq0["mdot_air"]
    put(f"C/cpg/{it}/mf", mf)
    fq = F43.equilibrium_fuel(FLIGHT, mf)
    for k in ("nu_lp", "nu_hp", "Tt4", "pi_lpc", "pi_hpc", "Phi_lp", "Phi_hp", "mdot_air"):
        put(f"C/cpg/{it}/{k}", fq[k])
    putd(f"C/cpg/{it}/passes", LAST_PASSES[0])
# …and the ONE shipped signature branch nothing above takes: an explicit START.
eq = F43.equilibrium_fuel(FLIGHT, F43.fuel_for_Tt4(FLIGHT, 1300.0), start=(0.90, 0.95))
put("C/start/nu_lp", eq["nu_lp"])
put("C/start/nu_hp", eq["nu_hp"])
put("C/start/Tt4", eq["Tt4"])
putd("C/start/passes", LAST_PASSES[0])
emit_census("C/cpg")

# THE TPG ARM — probe 3's detector, and the slice's one genuine exposure. The residual plateaus
# just under an ABSOLUTE 1e-12 bar (worst accepted 9.29e-13, i.e. 8 % under a bar the shipped
# comment calls "comfortably under"), so a last-bit difference anywhere upstream does not DRIFT
# the exit — it re-rolls how many passes squeak under, 16-fold. Dumped and gated at bit-equality
# against PyPy; EXCLUDED from step 4's CPython bar as a declared fragile set.
for name, g in (("tpg", Gas.thermally_perfect()), ("reacting", Gas.reacting()),
                ("forkb", Gas.reacting_forkb())):
    f = ft(g)
    for it, Tt4 in enumerate((1400.0, 1450.0)):
        mf = f.fuel_for_Tt4(FLIGHT, Tt4)
        eq = f.equilibrium_fuel(FLIGHT, mf)
        put(f"C/{name}/{it}/mf", mf)
        for k in ("nu_lp", "nu_hp", "Tt4", "Phi_lp", "Phi_hp"):
            put(f"C/{name}/{it}/{k}", eq[k])
        putd(f"C/{name}/{it}/passes", LAST_PASSES[0])
    emit_census(f"C/{name}")

# ------------------------------------------------- D: fuel_for_Tt4, the schedule, and _interp
for it, Tt4 in enumerate((LO, HI, 1500.0)):
    put(f"D/mf/{it}", F43.fuel_for_Tt4(FLIGHT, Tt4))
emit_census("D/mf")

ACC = F43.accel_schedule(FLIGHT, LO, HI, margin=0.15, n=5)
put("D/acc/margin", ACC.margin)
putd("D/acc/n", len(ACC.n_H))
for i, (x, y) in enumerate(zip(ACC.n_H, ACC.kappa)):
    put(f"D/acc/n_H/{i}", x)
    put(f"D/acc/kappa/{i}", y)
# `cap` on all three arms — BELOW the table, INSIDE it, ABOVE it.
for i, n_h in enumerate((ACC.n_H[0] - 0.05, 0.5 * (ACC.n_H[0] + ACC.n_H[-1]),
                         ACC.n_H[-1] + 0.05)):
    put(f"D/acc/cap/{i}", ACC.cap(n_h, 250_000.0))
# `_interp` is a SECOND COPY on purpose (the two-spool chain deliberately does not inherit
# SpoolTransient's). All three arms plus the exact-node case.
XS, YS = [1.0, 2.0, 3.0, 4.0], [10.0, 20.0, 15.0, 40.0]
for i, x in enumerate((0.5, 1.0, 1.5, 2.5, 3.999, 4.0, 9.0)):
    put(f"D/interp/{i}", TwoSpoolFuelTransient._interp(XS, YS, x))
emit_census("D")

# ------------------------------------------------------- E: the bare march, EVERY point
MF_LO = F43.fuel_for_Tt4(FLIGHT, LO)
MF_HI = F43.fuel_for_Tt4(FLIGHT, HI)
EQ0 = F43.equilibrium(FLIGHT, LO)
NU0 = (EQ0["nu_lp"], EQ0["nu_hp"])
put("E/mf_lo", MF_LO)
put("E/mf_hi", MF_HI)
put("E/nu0_lp", NU0[0])
put("E/nu0_hp", NU0[1])


def ramp(r):
    def sched(s):
        if s <= 0.0:
            return MF_LO
        if s >= r:
            return MF_HI
        return MF_LO + (MF_HI - MF_LO) * (s / r)
    return sched


put_traj("E/bare", F43.integrate_fuel(FLIGHT, ramp(0.5), NU0, 1.0, 0.05))
emit_census("E/bare")
put_traj("E/freeze_lp", F43.integrate_fuel(FLIGHT, ramp(0.5), NU0, 1.0, 0.05, freeze="lp"))
emit_census("E/freeze_lp")
put_traj("E/freeze_hp", F43.integrate_fuel(FLIGHT, ramp(0.5), NU0, 1.0, 0.05, freeze="hp"))
emit_census("E/freeze_hp")

# ------------------------------------------------------------ F: the ramp, AT THE TIE
# r = 0.25 => s_end = 8.25, ds = 0.02 => 8.25/0.02 = 412.5 EXACTLY. Python's half-to-EVEN round
# gives 412; a naive f64::round gives 413 — a whole extra marched point. § 5.16 measured
# Tt4_peak / X / E_temp_H / E_temp_L / complete BIT-IDENTICAL across that difference (the peak is
# attained at point 13 of 412, 3 % in, where the ramp ends), so `npts` is the ONLY key that can
# see it. It is dumped beside the values it is blind to.
#
# The QUOTIENT is exactly 412.5 — a fact about the float division's RESULT, not about the two
# operands as exact rationals (`Fraction(8.25)/Fraction(0.02)` is a 19-digit mess, because 0.02 is
# not a dyadic). And `int(412.5) == 412` too, so a TRUNCATION agrees with Python here: the naive
# test for this hazard reports agreement on precisely the case that matters.
assert Fraction(8.25 / 0.02) == Fraction(825, 2), "the tie is not exact"
assert int(8.25 / 0.02) == 412 == round(8.25 / 0.02), "the truncation AGREES here"
assert Fraction(8.30 / 0.02) == Fraction(415), "the control is not an exact integer"
putd("F/tie/steps_python", int(round(8.25 / 0.02)))
for tag, r in (("tie", 0.25), ("ctl", 0.30)):
    ex = F43.ramp_excursion_fuel(FLIGHT, LO, HI, r=r)
    for k in ("r", "rho", "Tt4_peak", "X", "E_temp_H", "E_temp_L"):
        put(f"F/{tag}/{k}", ex[k])
    putd(f"F/{tag}/complete", 1 if ex["complete"] else 0)
    putd(f"F/{tag}/npts", len(ex["traj"]))
    put(f"F/{tag}/s_last", ex["traj"][-1]["s"])
    put(f"F/{tag}/Tt4_last", ex["traj"][-1]["Tt4"])
    emit_census(f"F/{tag}")

# ------------------------------------------- G: the mechanism, the r->0 limit, and the plateau
FZ = F43.freeze_channels(FLIGHT, LO, HI, r=0.5, s_settle=2.0)
for k in ("both", "lp", "hp", "d_lp", "d_hp", "r", "rho"):
    put(f"G/freeze/{k}", FZ[k])
emit_census("G/freeze")

CS = F43.constant_speed_excursion_fuel(FLIGHT, LO, HI)
for k in ("Tt4_peak", "E_temp", "E_lp", "E_hp", "f"):
    put(f"G/const/{k}", CS[k])
emit_census("G/const")

# THE PLATEAU. Every currency's argmin is a TIE of two adjacent q at a gap of exactly 0.000e+00,
# and gate 9 (which asserts q_H < q_X < q_L with a gap > 0.3) is satisfied by a last-of-equals
# tie-break just as well. Python's `min` keeps the FIRST; only these keys say so.
#
# **THE GRID IS GATE 9's OWN, and the first one tried was the lesson.** A cheaper grid (2 shapes x
# 3 rho x 3 r at s_settle = 2.0) reported `tied = 0` on all three currencies: the score's plateau
# is a property of the GRID, not of the method, so a tie-break gate written against that grid
# COULD NOT FIRE and would have shipped as a gate that has never fired once.
PTS = []
for rho in (0.25, 1.0, 4.0, 8.0):
    f = ft(gas43(), LP_SHAPED, HP_SHAPED, rho=rho)
    for r in (0.25, 0.5, 1.0, 2.0):
        ex = f.ramp_excursion_fuel(FLIGHT, LO, HI, r=r)
        if ex["complete"]:
            PTS.append((r, rho, ex))
assert len(PTS) >= 12, len(PTS)
putd("G/pts/n", len(PTS))
for ip, (r, rho, ex) in enumerate(PTS):
    put(f"G/pts/{ip}/r", r)
    put(f"G/pts/{ip}/rho", rho)
    for k in ("X", "E_temp_H", "E_temp_L"):
        put(f"G/pts/{ip}/{k}", ex[k])
for k in ("X", "E_temp_H", "E_temp_L"):
    q, sp = TwoSpoolFuelTransient.collapse_exponent(PTS, k)
    put(f"G/collapse/{k}/q", q)
    put(f"G/collapse/{k}/spread", sp)
    # The TIE, made explicit: the neighbouring exponent scores the SAME to the bit.
    _, sp_next = TwoSpoolFuelTransient.collapse_exponent(PTS, k, q=q + 0.05)
    put(f"G/collapse/{k}/spread_next", sp_next)
    putd(f"G/collapse/{k}/tied", 1 if sp_next == sp else 0)
    for iq, qq in enumerate((0.0, 1.0)):
        _, s0 = TwoSpoolFuelTransient.collapse_exponent(PTS, k, q=qq)
        put(f"G/collapse/{k}/at/{iq}", s0)
emit_census("G")

# --------------------------------------------------------------- H: RUNG 45, the surge line
ARMED = ft(gas45(), LP_SHAPED.with_phi_surge(0.86), HP_SHAPED.with_phi_surge(0.90))
ex = ARMED.phi_excursion_fuel(FLIGHT, LO, HI, r=0.5, s_settle=1.0)
for k in ("ext_lp", "ext_hp", "s_lp", "s_hp", "min_phi_lp", "min_phi_hp", "Tt4_peak", "ratio"):
    put(f"H/exc/{k}", ex[k])
putd("H/exc/npts", ex["npts"])
emit_census("H/exc")

sm = ARMED.transient_surge_margin_fuel(FLIGHT, LO, HI, r=0.5, s_settle=1.0)
for k in ("margin_min_lp", "margin_min_hp", "steady_min_lp", "steady_min_hp",
          "min_phi_lp", "min_phi_hp", "phi_surge_lp", "phi_surge_hp"):
    put(f"H/sm/{k}", sm[k])
putd("H/sm/crossed_lp", 1 if sm["crossed_lp"] else 0)
putd("H/sm/crossed_hp", 1 if sm["crossed_hp"] else 0)
putd("H/sm/npts", sm["npts"])
emit_census("H/sm")

# --------------------------------------------- I: THE NINE ARMED CASES — the slice's coverage
# § 5.16 probe 4 (B) armed all seven limiter keywords for the first time. Its own summary table
# LOST the ninth row while its prose counted nine ("six of the nine"); recovered from the probe:
# the composite ALL case routes through `_integrate_fuel_lagged` WITH both min-select legs, which
# is the only case exercising that twin's `faded` (which references `mf_sched`, not `mf`) beside
# a sequential no-filter min-select. It is the most composed route in the family.
MF0 = F43.fuel_for_Tt4(FLIGHT, 1000.0)
MF1 = F43.fuel_for_Tt4(FLIGHT, 1400.0)
EQA = F43.equilibrium(FLIGHT, 1000.0)
NUA = (EQA["nu_lp"], EQA["nu_hp"])
ACCA = F43.accel_schedule(FLIGHT, 1000.0, 1400.0, margin=0.15, n=5)
SUA = SurgeLimiter(spool="lp", phi_lim=0.75)
put("I/mf0", MF0)
put("I/mf1", MF1)
put("I/nu0_lp", NUA[0])
put("I/nu0_hp", NUA[1])
emit_census("I/setup")


def sched_a(s):
    return MF0 + (MF1 - MF0) * min(1.0, s / 0.5)


CASES = (
    ("bare", {}),
    ("r46", dict(Tt4_max=1380.0)),
    ("r47", dict(Tt4_max=1380.0, tau_gov=0.2)),
    ("r48", dict(accel=ACCA)),
    ("r49", dict(surge=SUA)),
    ("r50", dict(surge=SUA, s_off=0.4)),
    ("r51", dict(surge=SUA, s_off=0.4, tau_rel=0.3)),
    ("r52", dict(surge=SUA, lag=AsymmetricLag(tau_att=0.02, tau_rel=0.3))),
    ("all", dict(Tt4_max=1380.0, tau_gov=0.2, accel=ACCA, surge=SUA)),
    # A TENTH CASE, and it is an ADDITION to probe 4 (B)'s nine rather than one of them. The
    # nine between them never CONTEST a `min`: the eight single-leg cases build at most one cap,
    # and the composite ALL routes to `_integrate_fuel_lagged`, whose min-select is SEQUENTIAL
    # and builds no `caps` list at all. So the bare marcher's `caps = [c for c in caps if
    # c < mf]` / `min(caps)` — the only place in the family where two legs contend for the same
    # actuator — was reached by nothing. Probe 2 measured `der` building ZERO caps 227 856 times
    # out of 227 856 across both suites' full grids, so this dump is the only coverage that
    # machinery has anywhere in phase 6. Dropping `tau_gov` from the composite is what puts it on
    # the bare route with all three legs armed.
    ("contest", dict(Tt4_max=1380.0, accel=ACCA, surge=SUA)),
)
for tag, kw in CASES:
    pts = F43.integrate_fuel(FLIGHT, sched_a, NUA, 1.0, 0.05, **kw)
    nkeys = put_traj(f"I/{tag}", pts)
    # The per-ROUTE key count, asserted from PYTHON's dict rather than from the Rust struct.
    assert nkeys in (POINT_KEYS_BARE, POINT_KEYS_ASYM), (tag, nkeys)
    assert (nkeys == POINT_KEYS_ASYM) == ("lag" in kw), (tag, nkeys)
    putd(f"I/{tag}/lagged_route", 1 if ("tau_gov" in kw and "Tt4_max" in kw) else 0)
    putd(f"I/{tag}/clipped", sum(1 for p in pts if p["mf"] < p["mf_sched"]))
    emit_census(f"I/{tag}")

# The rung-50/51 forced release with a FALSY tau_rel — `not tau_rel` is true for `None` AND for
# `0.0`, and both take the IDENTICAL step branch. `tau_rel=0.0` is a live signature the Python
# accepts (`tau_rel >= 0.0`) and nothing has ever passed.
for i, tr in enumerate((None, 0.0)):
    pts = F43.integrate_fuel(FLIGHT, sched_a, NUA, 0.6, 0.05, surge=SUA, s_off=0.4, tau_rel=tr)
    put_traj(f"I/falsy/{i}", pts)
    emit_census(f"I/falsy/{i}")
for i, s in enumerate((0.0, 0.2, 0.39999, 0.4, 0.55, 0.7, 1.0)):
    put(f"I/rw/step/{i}", E._release_weight(s, 0.4, None))
    put(f"I/rw/zero/{i}", E._release_weight(s, 0.4, 0.0))
    put(f"I/rw/fade/{i}", E._release_weight(s, 0.4, 0.3))
    put(f"I/rw/none/{i}", E._release_weight(s, None, 0.3))
emit_census("I/rw")

# ------------------------------------------------------- J: THE REFUSAL, through a real caller
# `test_rung43.py:317` pokes `_tt4_from_f` DIRECTLY, so it says nothing about reaching the refusal
# through an ordinary entry point. Traced: the assert fires inside `ev`, inside `g`, and the
# bracket scan CATCHES it — so every refusal is eaten, the march-in walks 0.04 at a time to the
# wall, and what the caller sees is the BRACKET assertion, naming a cause ("off the modeled
# speed-line region") that is not the actual one. No VALUE key exists on that path, so the gate is
# on the error's IDENTITY.
FEQ = ft(Gas.reacting_equilibrium())
MFEQ = FEQ.fuel_for_Tt4(FLIGHT, 1400.0)         # Tt4-control — allowed on every gas
put("J/mf", MFEQ)
emit_census("J/setup")

for tag, call in (("direct", lambda: FEQ._tt4_from_f(700.0, 0.025)),
                  ("instant", lambda: FEQ._instant_fuel(FLIGHT, 1.0, 1.0, MFEQ)),
                  ("equilibrium", lambda: FEQ.equilibrium_fuel(FLIGHT, MFEQ))):
    try:
        call()
        putd(f"J/{tag}/raised", 0)
        putd(f"J/{tag}/kind", -1)
    except AssertionError as exc:
        putd(f"J/{tag}/raised", 1)
        putd(f"J/{tag}/kind", kind_of(exc))
reset_census()

# ------------------------------------------------------------- K: the lp_disabled REDUCE
single = build_turbojet(gas43(), PI_HPC, TT4, FLIGHT.p0, **SINGLE)
ST = SpoolTransient(single, FLIGHT, 1.0, comp_map=HP_SHAPED)
DEG = TwoSpoolFuelTransient(single, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED,
                            lp_disabled=True)
for it, Tt4 in enumerate((1500.0, 1300.0, 1150.0)):
    mf = ST._fuel_for_Tt4(FLIGHT, Tt4)
    put(f"K/{it}/mf", mf)
    a, b = ST.equilibrium_fuel(FLIGHT, mf), DEG.equilibrium_fuel(FLIGHT, mf)
    for k in ("nu", "pi_c", "Tt4", "mdot_air", "f", "tau_t", "sp_thrust"):
        assert a[k] == b[k], (Tt4, k, a[k], b[k])
        put(f"K/{it}/{k}", a[k])
reset_census()

# ------------------------------------------- L: THE THIRD HIGH-WALL ARM, and a CPG bracket fail
# **SECTIONS A–K NEVER BIND THE THIRD ARM** — every one of their census rows reported
# `hi_wall_hi0 = 0`. So rung 43's most prominent departure from rung 40's closure was, on the
# first draft of this dump, covered by nothing but a partition sum that passes whether the arm
# binds or is ABSENT. Located rather than assumed: `hi0 = mdot_fuel*sqrt(Tt2)/(f_floor*mcorr*pt2)`
# beats `min(2.5, phi_max*n_L) = 2.1098` only for `mdot_fuel < 0.008439`, i.e. below `Tt4 ~ 930`
# on this running line. At Tt4 = 900 and 800 the arm binds AND the closure still returns a state.
for il, Tt4 in enumerate((900.0, 800.0)):
    mf = F43.fuel_for_Tt4(FLIGHT, Tt4)
    put(f"L/{il}/mf", mf)
    put_close(f"L/{il}", F43._close_fuel(1.0, 1.0, mf, Tt2, pt2))
emit_census("L")

# …and leaner still, the same arm binds and the bracket then FAILS — the only CPG cell in this
# file that reaches the closure's own "does not bracket" assert, which every section above gates
# against zero. `raised`/`kind` only: there is no state to compare.
for il, Tt4 in enumerate((700.0, 650.0)):
    mf = F43.fuel_for_Tt4(FLIGHT, Tt4)
    put(f"L/fail/{il}/mf", mf)
    try:
        F43._close_fuel(1.0, 1.0, mf, Tt2, pt2)
        putd(f"L/fail/{il}/raised", 0)
        putd(f"L/fail/{il}/kind", -1)
    except AssertionError as exc:
        putd(f"L/fail/{il}/raised", 1)
        putd(f"L/fail/{il}/kind", kind_of(exc))
emit_census("L/fail")

# =========================================================================== emit
out = sys.stdout
out.write("# slice S smoke — rungs 43+45 TwoSpoolFuelTransient — "
          "key\tu64 bits (or an integer)\trepr\n")
for key, bits, text in ROWS:
    out.write(f"{key}\t{bits}\t{text}\n")
sys.stderr.write(f"[dump_slice_s_smoke] {len(ROWS)} values\n")
