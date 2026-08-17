"""THE ORACLE, phase 6 slice Q — every rung-37 value the Rust must reproduce.

`CombustorTransient` splits rung 34's one bundled concession into two clocks. A PLENUM makes
`pt4` a state and CONFIRMS the concession; a HEAT-SOAK metal temperature is a second STATE and
CORRECTS it. Both default OFF and reduce to rung 35 by exact dispatch.

WHAT IS NEW HERE, and why each thing is dumped rather than asserted:

  * A QUANTITY THE RUNG'S OWN GATES CANNOT SEE. `_plenum_state`'s `Phi` is read at exactly ONE
    site — `equilibrium_plenum`'s residual — and the difference between the honest two-mass-flow
    power and a per-unit-air copy of `_instant_tail`'s is
    `eta_m*dh_t*(mdot_ngv - mdot_c*(1+f))`, which vanishes EXACTLY at the plenum's steady
    condition. Step 2 measured the consequence: that injection fails 0 of the 7 ported gates and
    60 of the smoke's 517 values. So section B dumps `Phi` OFF equilibrium, at three pressures
    around the steady one, which is the only place in the project where it is observable at all.

  * AN ILLINOIS ARM SLICE P SHIPPED AS UNREACHABLE. `_plenum_pt4_at` passes `_N_TOL = 1e-12` as
    an ABSOLUTE bracket width on a `pt4` of order 1e5 Pa, so 94.5 % of its calls exhaust `maxit`
    and return `b`. Slice P measured that arm at ZERO firings and could close the blind spot only
    with a counter. Every section here emits `illinois_exhausted`, and section B's is the
    detector for which endpoint the arm returns.

  * THREE MARCHES WITH NO `try`. Rung 34's marcher breaks out when a stage leaves the valid
    region, making trajectory LENGTH an output; all three of rung 37's run their step count
    unconditionally and let a failure propagate. Nothing fails on this grid, so the difference is
    LATENT — and the evaluation COUNT per section is the certificate that every step ran, since a
    failure would have aborted the march rather than shortened it.

  * A CEILING WHOSE SECOND ARM IS OFF THE OPERATING GRID. `_pic_band`'s `min(2.5, phi_max*n)`
    takes the map arm in every cell any gate reaches. Section A drives it up to `nu = 1.3`, where
    the literal 2.5 binds, so "the 2.5 never binds" is a MEASUREMENT with both sides present
    rather than a silence.

  * THE SOAK CLOSURE'S BRACKET, LIVE FROM ONE CALLER AND DEAD FROM THE OTHER. Fallibility is per
    call site: `equilibrium_soak`'s march-in absorbs failures, `soak_excursion`'s RK stages have
    no `try` at all. The two are separated by section, not summed.

Regenerate with:
    .venv\\Scripts\\python.exe rust/oracle/dump_combustor.py rust/oracle/combustor_pypy.tsv
    C:\\Python314\\python.exe   rust/oracle/dump_combustor.py rust/oracle/combustor_cpython.tsv
"""
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import engine as E                                        # noqa: E402
from turbojet.components import ram_recovery                            # noqa: E402
from turbojet.engine import (FlightCondition, build_turbojet,           # noqa: E402
                             ComponentMap, SpoolTransient, CombustorTransient)
from turbojet.gas import Gas                                            # noqa: E402

T0 = time.time()
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
    CENSUS.update(illinois_calls=0, illinois_evals=0, illinois_exhausted=0,
                  r34_solve_turbine=0, subsonic_raises=0, subsonic_escalations=0,
                  phi_max_flat5=0, phi_max_quadratic=0, phi_max_linear=0, phi_max_swirled=0,
                  backpressure_calls=0, backpressure_bracket_fails=0,
                  pt4_at_calls=0, pt4_at_bracket_fails=0, pt4_at_floor_fails=0,
                  soak_close_calls=0, soak_close_bracket_fails=0,
                  plenum_state_calls=0, instant_soak_calls=0)


reset_census()
_ILL = E._illinois


def _ill(f, a, b, fa, fb, tol=1e-10, maxit=100):
    """Count residual evaluations by wrapping `f`; infer exhaustion from the count.

    A call that exhausts `maxit` performs exactly `maxit` evaluations and never returns early, so
    the arm is observable WITHOUT re-implementing the loop — a copy would gate the copy. The tally
    happens INSIDE the wrapper, after the residual returns, so a search aborted by a raising
    residual keeps its partial count on both sides (`dump_spool.py`'s repaired shape, inherited
    rather than re-derived).
    """
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

_ST = SpoolTransient._solve_turbine


def _st(self, gas, Tt4, f, eta_t=None):
    CENSUS["r34_solve_turbine"] += 1
    return _ST(self, gas, Tt4, f, eta_t)


SpoolTransient._solve_turbine = _st

_SUB = SpoolTransient._turbine_subsonic


def _sub(self, *a, **k):
    try:
        return _SUB(self, *a, **k)
    except AssertionError:
        CENSUS["subsonic_raises"] += 1
        raise


SpoolTransient._turbine_subsonic = _sub

_TAIL = SpoolTransient._instant_tail


def _tail(self, *a, **k):
    try:
        return _TAIL(self, *a, **k)
    except AssertionError as ex:
        if "failed to bracket AWAY" in str(ex):
            CENSUS["subsonic_escalations"] += 1
        raise


SpoolTransient._instant_tail = _tail


_PHI_MAX = ComponentMap.phi_max


def _phi_max_counted(self, psi_floor=0.1):
    """Tally which arithmetic ARM of `phi_max` each call takes.

    § 5.14 prediction 9: rung 37 reaches only the QUADRATIC arm, because its whole grid is surge
    shapes — a DIFFERENT census from slice P's, which had `flat5` live at 5 258 because its grid
    included flat maps. Emitted per section so the two are never merged. That the other three arms
    are reachable at all is slice P's gate (`spool_oracle.rs`'s direct section drives all four);
    what is asserted here is that this rung never takes them.
    """
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


def wrap(cls, name, call_key, fail_key=None):
    orig = getattr(cls, name)

    def wrapped(self, *a, **k):
        CENSUS[call_key] += 1
        try:
            return orig(self, *a, **k)
        except AssertionError:
            if fail_key:
                CENSUS[fail_key] += 1
            raise
    setattr(cls, name, wrapped)


wrap(CombustorTransient, "_compressor_from_backpressure",
     "backpressure_calls", "backpressure_bracket_fails")
wrap(CombustorTransient, "_close_compressor_fuel_soak",
     "soak_close_calls", "soak_close_bracket_fails")
wrap(CombustorTransient, "_plenum_state", "plenum_state_calls")
wrap(CombustorTransient, "_instant_soak", "instant_soak_calls")

_P4 = CombustorTransient._plenum_pt4_at


def _p4(self, *a, **k):
    CENSUS["pt4_at_calls"] += 1
    try:
        return _P4(self, *a, **k)
    except AssertionError as ex:
        if "flow floor above the map ceiling" in str(ex):
            CENSUS["pt4_at_floor_fails"] += 1
        elif "mass balance does not bracket" in str(ex):
            CENSUS["pt4_at_bracket_fails"] += 1
        raise


CombustorTransient._plenum_pt4_at = _p4


def emit_census(prefix):
    for k in sorted(CENSUS):
        putd(f"census/{prefix}/{k}", CENSUS[k])
    reset_census()


# ======================================================================== the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C, TT4 = 10.0, 1500.0
REAL = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)
SHAPES = [("flow", ComponentMap.surge_flow()),
          ("press", ComponentMap.surge_pressure()),
          ("tilt", ComponentMap.surge_tilted())]

PLENUM_STATE_KEYS = ("nu", "pt4", "Tt4", "pi_c", "phi", "f", "mdot_c", "mdot_ngv",
                     "Phi", "dpt4_ds", "tau_t", "Tt3")
INSTANT_KEYS = ("nu", "Tt4", "pi_c", "tau_c", "eta_c", "eta_t", "m", "n", "flowcoef",
                "mdot_air", "f", "pi_t", "tau_t", "Tt3", "Tt5", "nu_t", "p_net_spec",
                "Phi", "sp_thrust", "thrust", "M9", "pt9_over_p0")


def build(cmap, **kw):
    gas = Gas.thermally_perfect()
    eng = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    return CombustorTransient(eng, FLIGHT, 1.0, comp_map=cmap, **kw)


def face(ct, nu):
    pi_d = ct.pi_d_max * ram_recovery(FLIGHT.M0)
    state0, _ = ct._fs_engine.freestream(FLIGHT, ct.mdot_air_design)
    Tt2, pt2 = state0.Tt, pi_d * state0.pt
    return Tt2, pt2, nu * (ct.Tt2_d / Tt2) ** 0.5


# ==================================================== A: the speed line, and BOTH ceiling arms
# nu = 1.3 is where `min(2.5, phi_max*n)` takes the LITERAL arm. No gate ever reaches it, which
# is exactly why it is here: a zero on the other arm is evidence only when both are present.
for sh, cmap in SHAPES:
    ct = build(cmap, plenum_ratio=0.05)
    for nu in (0.7, 0.85, 1.0, 1.3):
        Tt2, pt2, n = face(ct, nu)
        m_lo, pic_max, m_hi, pic_min = ct._pic_band(cmap, n, Tt2)
        tag = f"A/{sh}/{nu}"
        put(f"{tag}/m_lo", m_lo)
        put(f"{tag}/pic_max", pic_max)
        put(f"{tag}/m_hi", m_hi)
        put(f"{tag}/pic_min", pic_min)
        putd(f"{tag}/ceiling_is_the_literal", 1 if 2.5 <= cmap.phi_max() * n else 0)
        for im, frac in enumerate((0.0, 0.25, 0.5, 0.75, 1.0)):
            m = m_lo + frac * (m_hi - m_lo)
            pic, phi, tau_c, Tt3, eta_c = ct._pic_of_m(cmap, n, Tt2, m)
            put(f"{tag}/{im}/pi_c", pic)
            put(f"{tag}/{im}/flowcoef", phi)
            put(f"{tag}/{im}/tau_c", tau_c)
            put(f"{tag}/{im}/tt3", Tt3)
            put(f"{tag}/{im}/eta_c", eta_c)
emit_census("A")

# ============================== B: the DECOUPLED instant — the ONLY place `Phi` is observable
for sh, cmap in SHAPES:
    ct = build(cmap, plenum_ratio=0.05)
    for Tt4 in (1400.0, 1100.0):
        mf = ct._fuel_for_Tt4(FLIGHT, Tt4, cmap)
        nu0 = ct.equilibrium_fuel(FLIGHT, mf, cmap)["nu"]
        pt4_s = ct._plenum_pt4_at(FLIGHT, nu0, mf, cmap)
        put(f"B/{sh}/{Tt4:.0f}/nu0", nu0)
        put(f"B/{sh}/{Tt4:.0f}/pt4_steady", pt4_s)
        for ip, scale in enumerate((0.94, 0.97, 1.0, 1.03, 1.06)):
            s = ct._plenum_state(FLIGHT, nu0, pt4_s * scale, mf, cmap)
            for k in PLENUM_STATE_KEYS:
                put(f"B/{sh}/{Tt4:.0f}/{ip}/{k}", s[k])
            put(f"B/{sh}/{Tt4:.0f}/{ip}/split",
                (s["mdot_c"] + mf - s["mdot_ngv"]) / s["mdot_ngv"])
        # the back-pressure invert on its own, at the steady pressure
        Tt2, pt2, n = face(ct, nu0)
        c = ct._compressor_from_backpressure(cmap, n, Tt2, pt2, pt4_s)
        for k in ("m", "phi", "tau_c", "Tt3", "eta_c", "pi_c"):
            put(f"B/{sh}/{Tt4:.0f}/bp_{k}", c[k])
emit_census("B")

# ==================================================== C: the plenum EQUILIBRIUM (gate 2's grid)
for sh, cmap in SHAPES:
    ct = build(cmap, plenum_ratio=0.05)
    for Tt4 in (1400.0, 1100.0, 900.0):
        mf = ct._fuel_for_Tt4(FLIGHT, Tt4, cmap)
        a = ct.equilibrium_plenum(FLIGHT, mf, cmap)
        b = ct.equilibrium_fuel(FLIGHT, mf, cmap)
        tag = f"C/{sh}/{Tt4:.0f}"
        for k in PLENUM_STATE_KEYS:
            put(f"{tag}/{k}", a[k])
        put(f"{tag}/rung35_nu", b["nu"])
        put(f"{tag}/rung35_pi_c", b["pi_c"])
        put(f"{tag}/rung35_tau_t", b["tau_t"])
        put(f"{tag}/massbal_rel", (a["mdot_c"] + mf - a["mdot_ngv"]) / a["mdot_ngv"])
        put(f"{tag}/mf", mf)
emit_census("C")

# ==================================================== D: the plenum MARCH (gate 3's grid)
for sh, cmap in SHAPES:
    for r_v in (0.03, 0.1):
        ct = build(cmap, plenum_ratio=r_v)
        r = ct.plenum_frozen_peak(FLIGHT, 1100.0, 1400.0, cmap)
        tag = f"D/{sh}/{r_v}"
        for k in ("E0", "peak", "peak_minus_E0", "split_max", "nu0", "r_v"):
            put(f"{tag}/{k}", r[k])
emit_census("D")

# ==================================================== E: the soak CLOSURE and INSTANT, directly
for sh, cmap in SHAPES:
    ct = build(cmap, soak_gain=0.1, soak_ratio=3.0)
    mf = ct._fuel_for_Tt4(FLIGHT, 1400.0, cmap)
    nu = ct.equilibrium_fuel(FLIGHT, mf, cmap)["nu"]
    Tt2, pt2, n = face(ct, nu)
    put(f"E/{sh}/nu", nu)
    put(f"E/{sh}/tt2", Tt2)
    put(f"E/{sh}/pt2", pt2)
    put(f"E/{sh}/n", n)
    # 1600 K is ABOVE the burner exit, so the sink runs backwards — the reslam sign.
    for im, Tm in enumerate((1000.0, 1250.0, 1450.0, 1600.0)):
        c = ct._close_compressor_fuel_soak(Tt2, pt2, cmap, n, mf, Tm)
        for k in ("m", "m_imp", "phi", "tau_c", "eta_c", "Tt3", "Tt4_b", "Tt4_t",
                  "pi_c", "pt4", "f", "mdot4", "mdot_air"):
            put(f"E/{sh}/{im}/{k}", c[k])
        i = ct._instant_soak(FLIGHT, nu, mf, Tm, cmap)
        for k in INSTANT_KEYS:
            put(f"E/{sh}/{im}/i_{k}", i[k])
        put(f"E/{sh}/{im}/Tt4_burner", i["Tt4_burner"])
        put(f"E/{sh}/{im}/dTm_ds", i["dTm_ds"])
        putd(f"E/{sh}/{im}/branch_choked", 1 if i["branch"] == "choked" else 0)
emit_census("E")

# ==================================================== F: the soak EQUILIBRIUM (gate 4's grid)
for sh, cmap in SHAPES:
    ct = build(cmap, soak_gain=0.1, soak_ratio=3.0)
    for Tt4 in (1400.0, 1100.0):
        mf = ct._fuel_for_Tt4(FLIGHT, Tt4, cmap)
        a = ct.equilibrium_soak(FLIGHT, mf, cmap)
        b = ct.equilibrium_fuel(FLIGHT, mf, cmap)
        tag = f"F/{sh}/{Tt4:.0f}"
        for k in INSTANT_KEYS:
            put(f"{tag}/{k}", a[k])
        put(f"{tag}/Tt4_burner", a["Tt4_burner"])
        put(f"{tag}/dTm_ds", a["dTm_ds"])
        put(f"{tag}/rung35_nu", b["nu"])
        put(f"{tag}/rung35_pi_c", b["pi_c"])
        put(f"{tag}/rung35_tau_t", b["tau_t"])
emit_census("F")

# ==================================================== G: the two-state MARCH (gate 5's grid)
for sh, cmap in SHAPES:
    for G in (0.05, 0.15):
        for r_m in (1.0, 5.0):
            ct = build(cmap, soak_gain=G, soak_ratio=r_m)
            tag = f"G/{sh}/{G}/{r_m}"
            runs = [("cold", ct.soak_excursion(FLIGHT, 1100.0, 1400.0, "cold", cmap, s_end=6.0)),
                    ("hot", ct.soak_excursion(FLIGHT, 1100.0, 1400.0, "hot", cmap, s_end=6.0)),
                    ("adiab", ct.adiabatic_excursion(FLIGHT, 1100.0, 1400.0, cmap, s_end=6.0))]
            for name, r in runs:
                put(f"{tag}/{name}/e_surge", r["E_surge"])
                put(f"{tag}/{name}/nu0", r["nu0"])
                put(f"{tag}/{name}/nu_final", r["nu_final"])
                putd(f"{tag}/{name}/t_accel_is_none", 1 if r["t_accel"] is None else 0)
                put(f"{tag}/{name}/t_accel", 0.0 if r["t_accel"] is None else r["t_accel"])
            # THE VERDICT, as a discrete key: cold < hot < adiabatic.
            putd(f"{tag}/ordering_holds",
                 1 if runs[0][1]["E_surge"] < runs[1][1]["E_surge"] < runs[2][1]["E_surge"] else 0)
emit_census("G")

# ==================================================== H: the accel LAG (gate 6's grid)
cmap = ComponentMap.surge_flow()
for G in (0.05, 0.15):
    ct = build(cmap, soak_gain=G, soak_ratio=3.0)
    for name, r in (("adiab", ct.adiabatic_excursion(FLIGHT, 1100.0, 1400.0, cmap)),
                    ("cold", ct.soak_excursion(FLIGHT, 1100.0, 1400.0, "cold", cmap)),
                    ("hot", ct.soak_excursion(FLIGHT, 1100.0, 1400.0, "hot", cmap))):
        put(f"H/{G}/{name}/e_surge", r["E_surge"])
        putd(f"H/{G}/{name}/t_accel_is_none", 1 if r["t_accel"] is None else 0)
        put(f"H/{G}/{name}/t_accel", 0.0 if r["t_accel"] is None else r["t_accel"])
emit_census("H")

# ==================================================== I: the both-OFF REDUCE
gas = Gas.thermally_perfect()
eng = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
CT_OFF = CombustorTransient(eng, FLIGHT, 1.0, comp_map=ComponentMap.surge_flow())
eng2 = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
ST = SpoolTransient(eng2, FLIGHT, 1.0, comp_map=ComponentMap.surge_flow())
put("I/plenum_K", CT_OFF._plenum_K)
put("I/pt4_d", CT_OFF.pt4_d)
put("I/mdot4_d", CT_OFF.mdot4_d)
for Tt4 in (1500.0, 1300.0, 1200.0, 1000.0, 900.0):
    mf = ST._fuel_for_Tt4(FLIGHT, Tt4)
    a = CT_OFF.equilibrium_fuel(FLIGHT, mf)
    b = ST.equilibrium_fuel(FLIGHT, mf)
    for k in ("nu", "pi_c", "tau_t", "Tt4", "mdot_air", "Phi"):
        put(f"I/{Tt4:.0f}/ct_{k}", a[k])
        put(f"I/{Tt4:.0f}/st_{k}", b[k])
    putd(f"I/{Tt4:.0f}/bit_identical",
         1 if all(a[k] == b[k] for k in ("nu", "pi_c", "tau_t", "Tt4", "mdot_air")) else 0)
emit_census("I")

# =========================================================================== emit
path = sys.argv[1] if len(sys.argv) > 1 else None
out = open(path, "w", encoding="utf-8", newline="\n") if path else sys.stdout
out.write("# phase-6Q combustor-transient oracle — key\tu64 bits (or an integer)\trepr\n")
out.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
for key, bits, text in ROWS:
    out.write(f"{key}\t{bits}\t{text}\n")
if path:
    out.close()
sys.stderr.write(f"[dump_combustor] {len(ROWS)} values in {time.time() - T0:.1f}s -> {path}\n")
