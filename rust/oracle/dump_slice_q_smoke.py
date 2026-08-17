"""SLICE Q step 1 — the SMOKE dump for rung 37 (`CombustorTransient`).

Not the slice's oracle (that is step 3, on the full gate grid). This exists to catch a structural
mistake — a power block copied from `_instant_tail` instead of written on the absolute flows, a
`pi_c` returned from the recomputed root instead of the required back-pressure, the two
`equilibrium_soak` loops unified, a march routed through `spool.rs::march` — BEFORE the seven
Python gates are ported on top of it at step 2.

The cells are chosen to touch every path ONCE:

  A  `_pic_of_m` / `_pic_band` driven DIRECTLY at two speeds and three flows, including both band
     endpoints. § 5.14 probe 3 measured that `phi_max*n` binds the ceiling in 15 of 15 cells and
     the literal 2.5 never, so the ceiling is dumped as a value rather than trusted.
  B  the BACK-PRESSURE invert — the map's THIRD use — driven at a pt4 built from a known flow, so
     the returned `m` is checkable against the flow that produced it.
  C  `_plenum_state` OFF equilibrium at three pt4 around the steady one, which is the only place
     `mdot_c != mdot_NGV` and `dpt4_ds != 0` are both live.
  D  `_plenum_pt4_at` — the site whose Illinois EXHAUSTS `maxit` on 94.5 % of its calls (§ 5.14
     probe 2). Its `illinois_exhausted` count is the whole detector for the return arm.
  E  `equilibrium_plenum` beside `equilibrium_fuel`: the non-tautological reduce, two closures.
  F  `plenum_frozen_peak` on two shapes x two fill clocks — a 151-step RK4 with no `try`.
  G  `_close_compressor_fuel_soak` at three metal temperatures, including one ABOVE the burner
     exit (the reslam sign, where the sink runs backwards).
  H  `_instant_soak`, which reuses rung 34's `_instant_tail` unchanged — the opposite of the
     plenum, whose power block could not.
  I  `equilibrium_soak`, whose two fixed-point loops differ by one line.
  J  `soak_excursion` cold and hot beside `adiabatic_excursion`, at `s_end = 3.0` so the
     adiabatic reaches 99 % of the speed rise and the cold one does NOT — both arms of `t_accel`
     in one smoke.
  K  the both-OFF REDUCE against a bare `SpoolTransient`, which must be BIT-identical.

Regenerate with:
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_q_smoke.py > rust/oracle/slice_q_smoke_pypy.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import engine as E                                        # noqa: E402
from turbojet.components import ram_recovery                            # noqa: E402
from turbojet.engine import (FlightCondition, build_turbojet,           # noqa: E402
                             ComponentMap, SpoolTransient, CombustorTransient)
from turbojet.gas import Gas                                            # noqa: E402

ROWS = []


def put(key, value):
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


def putd(key, n):
    n = int(n)
    ROWS.append((key, n, str(n)))


# ================================================================= the instruments
# Every one WRAPS shipped code; none copies a loop.
CENSUS = {}


def reset_census():
    CENSUS.clear()
    CENSUS.update(illinois_calls=0, illinois_evals=0, illinois_exhausted=0,
                  backpressure_calls=0, backpressure_bracket_fails=0,
                  pt4_at_calls=0, pt4_at_bracket_fails=0, pt4_at_floor_fails=0,
                  soak_close_calls=0, soak_close_bracket_fails=0,
                  plenum_state_calls=0, instant_soak_calls=0)


reset_census()
_ILL = E._illinois


def _ill(f, a, b, fa, fb, tol=1e-10, maxit=100):
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


def wrap(cls, name, call_key, fail_key=None, fail_match=None):
    orig = getattr(cls, name)

    def wrapped(self, *a, **k):
        CENSUS[call_key] += 1
        try:
            return orig(self, *a, **k)
        except AssertionError as ex:
            if fail_key and (fail_match is None or fail_match in str(ex)):
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
FLOW = ComponentMap.surge_flow()
PRESS = ComponentMap.surge_pressure()


def build(cmap, **kw):
    gas = Gas.thermally_perfect()
    eng = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
    return CombustorTransient(eng, FLIGHT, 1.0, comp_map=cmap, **kw)


def face(ct, nu):
    """The four lines `_plenum_state`, `_plenum_pt4_at` and `_instant_soak` each open with."""
    pi_d = ct.pi_d_max * ram_recovery(FLIGHT.M0)
    state0, V0 = ct._fs_engine.freestream(FLIGHT, ct.mdot_air_design)
    Tt2, pt2 = state0.Tt, pi_d * state0.pt
    return Tt2, pt2, nu * (ct.Tt2_d / Tt2) ** 0.5


# --------------------------------------------------------------- A: the speed line read as pi_c(m)
CTP = build(FLOW, plenum_ratio=0.05)
for iu, nu in enumerate((0.85, 1.0)):
    Tt2, pt2, n = face(CTP, nu)
    m_lo, pic_max, m_hi, pic_min = CTP._pic_band(FLOW, n, Tt2)
    put(f"A/band/{iu}/m_lo", m_lo)
    put(f"A/band/{iu}/pic_max", pic_max)
    put(f"A/band/{iu}/m_hi", m_hi)
    put(f"A/band/{iu}/pic_min", pic_min)
    for im, m in enumerate((m_lo, 0.5 * (m_lo + m_hi), m_hi)):
        pic, phi, tau_c, Tt3, eta_c = CTP._pic_of_m(FLOW, n, Tt2, m)
        put(f"A/pic/{iu}/{im}/pi_c", pic)
        put(f"A/pic/{iu}/{im}/flowcoef", phi)
        put(f"A/pic/{iu}/{im}/tau_c", tau_c)
        put(f"A/pic/{iu}/{im}/tt3", Tt3)
        put(f"A/pic/{iu}/{im}/eta_c", eta_c)
emit_census("A")

# --------------------------------------------------------------- B: the back-pressure invert
for iu, nu in enumerate((0.85, 1.0)):
    Tt2, pt2, n = face(CTP, nu)
    m_lo, _, m_hi, _ = CTP._pic_band(FLOW, n, Tt2)
    m_mid = 0.5 * (m_lo + m_hi)
    pt4 = CTP._pic_of_m(FLOW, n, Tt2, m_mid)[0] * CTP.pi_b * pt2
    c = CTP._compressor_from_backpressure(FLOW, n, Tt2, pt2, pt4)
    for k in ("m", "phi", "tau_c", "Tt3", "eta_c", "pi_c"):
        put(f"B/{iu}/{k}", c[k])
    put(f"B/{iu}/m_target", m_mid)
emit_census("B")

# --------------------------------------------------------------- C: the decoupled instant
MF14 = CTP._fuel_for_Tt4(FLIGHT, 1400.0, FLOW)
NU0 = CTP.equilibrium_fuel(FLIGHT, MF14, FLOW)["nu"]
PT4_S = CTP._plenum_pt4_at(FLIGHT, NU0, MF14, FLOW)
put("C/nu0", NU0)
put("C/pt4_steady", PT4_S)
for ip, scale in enumerate((0.97, 1.0, 1.03)):
    s = CTP._plenum_state(FLIGHT, NU0, PT4_S * scale, MF14, FLOW)
    for k in ("nu", "pt4", "Tt4", "pi_c", "phi", "f", "mdot_c", "mdot_ngv",
              "Phi", "dpt4_ds", "tau_t", "Tt3"):
        put(f"C/{ip}/{k}", s[k])
    put(f"C/{ip}/split", (s["mdot_c"] + MF14 - s["mdot_ngv"]) / s["mdot_ngv"])
emit_census("C")

# --------------------------------------------------------------- D: the exhausting root find
for iu, nu in enumerate((0.85, 1.0)):
    put(f"D/{iu}/pt4", CTP._plenum_pt4_at(FLIGHT, nu, MF14, FLOW))
emit_census("D")

# --------------------------------------------------------------- E: the non-tautological reduce
for ish, (sh, cmap) in enumerate((("flow", FLOW), ("press", PRESS))):
    ct = build(cmap, plenum_ratio=0.05)
    for it, Tt4 in enumerate((1400.0, 1100.0)):
        mf = ct._fuel_for_Tt4(FLIGHT, Tt4, cmap)
        a = ct.equilibrium_plenum(FLIGHT, mf, cmap)
        b = ct.equilibrium_fuel(FLIGHT, mf, cmap)
        for k in ("nu", "pt4", "Tt4", "pi_c", "phi", "f", "mdot_c", "mdot_ngv",
                  "Phi", "dpt4_ds", "tau_t", "Tt3"):
            put(f"E/{sh}/{it}/{k}", a[k])
        put(f"E/{sh}/{it}/rung35_nu", b["nu"])
        put(f"E/{sh}/{it}/rung35_pi_c", b["pi_c"])
        put(f"E/{sh}/{it}/massbal_rel", (a["mdot_c"] + mf - a["mdot_ngv"]) / a["mdot_ngv"])
emit_census("E")

# --------------------------------------------------------------- F: the plenum march
for ish, (sh, cmap) in enumerate((("flow", FLOW), ("press", PRESS))):
    for iv, r_v in enumerate((0.03, 0.1)):
        ct = build(cmap, plenum_ratio=r_v)
        r = ct.plenum_frozen_peak(FLIGHT, 1100.0, 1400.0, cmap)
        for k in ("E0", "peak", "peak_minus_E0", "split_max", "nu0", "r_v"):
            put(f"F/{sh}/{iv}/{k}", r[k])
emit_census("F")

# --------------------------------------------------------------- G: the soak closure
CTS = build(FLOW, soak_gain=0.1, soak_ratio=3.0)
MFS = CTS._fuel_for_Tt4(FLIGHT, 1400.0, FLOW)
NUS = CTS.equilibrium_fuel(FLIGHT, MFS, FLOW)["nu"]
put("G/nu", NUS)
Tt2, pt2, n = face(CTS, NUS)
put("G/tt2", Tt2)
put("G/pt2", pt2)
put("G/n", n)
for im, Tm in enumerate((1100.0, 1400.0, 1600.0)):     # 1600 is ABOVE the burner exit
    c = CTS._close_compressor_fuel_soak(Tt2, pt2, FLOW, n, MFS, Tm)
    for k in ("m", "m_imp", "phi", "tau_c", "eta_c", "Tt3", "Tt4_b", "Tt4_t",
              "pi_c", "pt4", "f", "mdot4", "mdot_air"):
        put(f"G/{im}/{k}", c[k])
emit_census("G")

# --------------------------------------------------------------- H: the soak instant
for im, Tm in enumerate((1100.0, 1400.0, 1600.0)):
    i = CTS._instant_soak(FLIGHT, NUS, MFS, Tm, FLOW)
    for k in ("nu", "Tt4", "pi_c", "tau_c", "eta_c", "eta_t", "m", "n", "flowcoef", "mdot_air",
              "f", "pi_t", "tau_t", "Tt3", "Tt5", "nu_t", "p_net_spec", "Phi", "sp_thrust",
              "thrust", "M9", "pt9_over_p0", "Tt4_burner", "dTm_ds"):
        put(f"H/{im}/{k}", i[k])
    putd(f"H/{im}/branch_choked", 1 if i["branch"] == "choked" else 0)
emit_census("H")

# --------------------------------------------------------------- I: the two-loop equilibrium
for ish, (sh, cmap) in enumerate((("flow", FLOW), ("press", PRESS))):
    ct = build(cmap, soak_gain=0.1, soak_ratio=3.0)
    for it, Tt4 in enumerate((1400.0, 1100.0)):
        mf = ct._fuel_for_Tt4(FLIGHT, Tt4, cmap)
        a = ct.equilibrium_soak(FLIGHT, mf, cmap)
        b = ct.equilibrium_fuel(FLIGHT, mf, cmap)
        for k in ("nu", "Tt4", "pi_c", "Phi", "tau_t", "Tt4_burner", "dTm_ds", "mdot_air"):
            put(f"I/{sh}/{it}/{k}", a[k])
        put(f"I/{sh}/{it}/rung35_nu", b["nu"])
        put(f"I/{sh}/{it}/rung35_pi_c", b["pi_c"])
emit_census("I")

# --------------------------------------------------------------- J: the two-state march
# s_end = 3.0 so the ADIABATIC reaches 99 % of the speed rise (t ~ 2.15) and the COLD one does
# not — both arms of `t_accel` in one smoke.
for name, res in (("cold", CTS.soak_excursion(FLIGHT, 1100.0, 1400.0, "cold", FLOW, s_end=3.0)),
                  ("hot", CTS.soak_excursion(FLIGHT, 1100.0, 1400.0, "hot", FLOW, s_end=3.0)),
                  ("adiab", CTS.adiabatic_excursion(FLIGHT, 1100.0, 1400.0, FLOW, s_end=3.0))):
    put(f"J/{name}/e_surge", res["E_surge"])
    put(f"J/{name}/nu0", res["nu0"])
    put(f"J/{name}/nu_final", res["nu_final"])
    putd(f"J/{name}/t_accel_is_none", 1 if res["t_accel"] is None else 0)
    put(f"J/{name}/t_accel", 0.0 if res["t_accel"] is None else res["t_accel"])
emit_census("J")

# --------------------------------------------------------------- K: the both-OFF reduce
gas = Gas.thermally_perfect()
eng = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
CT_OFF = CombustorTransient(eng, FLIGHT, 1.0, comp_map=FLOW)
eng2 = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL)
ST = SpoolTransient(eng2, FLIGHT, 1.0, comp_map=FLOW)
put("K/plenum_K", CT_OFF._plenum_K)
put("K/pt4_d", CT_OFF.pt4_d)
put("K/mdot4_d", CT_OFF.mdot4_d)
for it, Tt4 in enumerate((1500.0, 1200.0, 900.0)):
    mf = ST._fuel_for_Tt4(FLIGHT, Tt4)
    a = CT_OFF.equilibrium_fuel(FLIGHT, mf)
    b = ST.equilibrium_fuel(FLIGHT, mf)
    for k in ("nu", "pi_c", "tau_t", "Tt4", "mdot_air"):
        put(f"K/{it}/ct_{k}", a[k])
        put(f"K/{it}/st_{k}", b[k])
emit_census("K")

# =========================================================================== emit
out = sys.stdout
out.write("# slice Q smoke — rung 37 CombustorTransient — key\tu64 bits (or an integer)\trepr\n")
for key, bits, text in ROWS:
    out.write(f"{key}\t{bits}\t{text}\n")
sys.stderr.write(f"[dump_slice_q_smoke] {len(ROWS)} values\n")
