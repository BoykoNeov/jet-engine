"""SLICE W step 2 — the SMOKE dump for rungs 62-63 (`ScheduledBleedTransient`).

Not the slice's oracle (that is step 4, on both suites' grids). This exists to catch a
structural mistake BEFORE the 88 Python gates are ported on top of it at step 3 — and
§ 5.21's five probes named the mistakes in advance, each of which the shipped Rust
deliberately does NOT make:

  1. **`at_stator` LEFT AS RUNG 57's.** § 5.21 (ii) measured that flipping the rung-63
     counterfeit gate's two identities from True/True to False/False, at 9.5e-3 and 1.0e-2.
     Section E dumps `at_stator()`'s valve arming AND the inherited `schedule_invariance`
     run on a bleed-armed machine, so a bare-sibling port shows as a wrong number rather
     than as a missing method.
  2. **`_powers`/`_instant_tail` DISPATCHING ON `b_of` INSTEAD OF THE CLOSURE'S OWN KEY.**
     Python reads `c.get("bleed", 0.0)` — an ABSENT key on rung 40/57 closures. Both
     spellings agree wherever b is 0, so no value key can see the difference. Section G
     carries the four reduced/bled COUNTS, which is the only instrument that can.
  3. **THE `1/(1-b)` DROPPED FROM THE FUEL BRACKET WALLS.** `f_cap`/`f_floor` are
     CORE-referenced, so the FACE-flow walls they imply carry it; without it the scan starts
     INSIDE the physical root at large b. Section D marches the fuel closure at b = 0.30.
  4. **`mdot_face` READ AS THE TRIAL FACE FLOW.** Python's dict key is `mdot_imp/(1-b)` and
     shadows a local of the same name three lines up. They agree only AT the root, so a
     converged closure hides it — section C dumps `_powers`' own output, which is where the
     wrong one would bite.
  5. **`R62_FUEL` SPREAD FROM `..R43` INSTEAD OF `..R57_FUEL`.** Rung 62 does not override
     `_surge_fuel`, so the wrong spread silently drops rung 60's floor-resolving body.
     Section F runs a phi-floor leg on a bleed-armed machine.

Every float is emitted as its IEEE-754 bit pattern, so the comparison is bit-equality and not
a tolerance. Regenerate with:
    .venv\\Scripts\\python.exe rust\\oracle\\dump_slice_w_smoke.py > rust\\oracle\\slice_w_smoke_pypy.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from turbojet.gas import Gas                                                      # noqa: E402
from turbojet.engine import (                                                     # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap, ScheduledStatorTransient,
    StatorSchedule, ScheduledBleedTransient, BleedSchedule, SurgeLimiter,
)

OUT = []


def f(key, x):
    OUT.append((key, struct.unpack("<Q", struct.pack("<d", float(x)))[0]))


def d(key, n):
    OUT.append((key, int(n)))


def b(key, flag):
    OUT.append((key, 1 if flag else 0))


# ---------------------------------------------------------------------------- the grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE = 1000.0, 1400.0, 0.01, 1.2
N_LO, V, B = 0.65, 0.20, 0.10
LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """`test_rung62.py`'s own `_cpg`, spelled CHARACTER FOR CHARACTER.

    `R_c` is DERIVED as `(gamma_c - 1.0)/gamma_c * cp_c`, and that is not a formality:
    `1.4 - 1.0` is `0.3999999999999999` in IEEE-754, so re-spelling it as `0.4/1.4` builds a
    gas one ULP away and drifts EVERY number in this dump — including section H's reduce,
    which never touches the valve. The first writing of this file did exactly that."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def bt(**kw):
    return ScheduledBleedTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw)


SCHED = BleedSchedule(B, N_LO)
STAT = StatorSchedule(V, N_LO)

# ==================================================================== A -- the SCHEDULE type
# The deliberate TWIN of rung 57's. Both shapes, both clip arms, and the EXACT zero at n_ref
# that `__post_init__` asserts.
for shape in ("smooth", "linear"):
    s = BleedSchedule(B, N_LO, shape=shape)
    for n in (0.40, 0.60, N_LO, 0.70, 0.80, 0.90, 0.999, 1.0, 1.05, 1.30):
        f(f"A/{shape}/b_of_n/{n:.3f}", s(n))
f("A/corner/exact_zero", BleedSchedule(B, N_LO)(1.0))
f("A/bmax0/at_lo", BleedSchedule(0.0, N_LO)(N_LO))

# ==================================================================== B -- `b_of` on a MACHINE
# The CONSTANT leg, the SCHEDULED leg, and the Tt2 referral the schedule reads through.
for tag, kw in (("const", dict(bleed=B)), ("sched", dict(bleed_sched=SCHED)),
                ("bare", {})):
    m = bt(**kw)
    b(f"B/{tag}/armed", m._armed_bleed())
    for nu in (0.60, 0.75, 0.90, 1.00, 1.10):
        f(f"B/{tag}/b_of_design_Tt2/{nu:.2f}", m.b_of(nu))
        f(f"B/{tag}/b_of_Tt2_280/{nu:.2f}", m.b_of(nu, 280.0))

# ==================================================================== C -- the TWO CLOSURES
# The bled bodies, and `_powers`/`_instant_tail` on top of them. `mdot_face` is the key the
# trial-vs-imposed shadowing would move, and it reaches the answer only through `_powers`.
KEYS = ("m_lp", "m_imp", "m_hp", "phi_lp", "phi_hp", "n_lp", "n_hp", "tau_lpc", "tau_hpc",
        "Tt25", "Tt3", "pi_lpc", "pi_hpc", "pt4", "f", "eta_lpc", "eta_hpc", "mdot_air",
        "mdot4", "bleed", "mdot_face")
for tag, kw in (("const", dict(bleed=B)), ("sched", dict(bleed_sched=SCHED)),
                ("both", dict(bleed_sched=SCHED, vsv_sched_lp=STAT))):
    m = bt(**kw)
    Tt2, pt2, V0 = m._inlet(FLIGHT)
    for (nu_lp, nu_hp, Tt4) in ((0.80, 0.85, 1200.0), (0.95, 0.97, 1400.0)):
        c = m._close(nu_lp, nu_hp, Tt4, Tt2, pt2)
        for k in KEYS:
            f(f"C/{tag}/close/{nu_lp:.2f}_{Tt4:.0f}/{k}", c[k])
        p = m._powers(c, FLIGHT, nu_lp, nu_hp, Tt4)
        f(f"C/{tag}/powers/{nu_lp:.2f}_{Tt4:.0f}/Phi_lp", p[0])
        f(f"C/{tag}/powers/{nu_lp:.2f}_{Tt4:.0f}/Phi_hp", p[1])
        t = m._instant_tail(FLIGHT, c, nu_lp, nu_hp, Tt4, V0)
        for k in ("Phi_lp", "Phi_hp", "Tt45", "Tt5", "tau_hpt", "tau_lpt", "pi_hpt", "pi_lpt",
                  "sp_thrust", "sp_thrust_inlet", "M9"):
            f(f"C/{tag}/tail/{nu_lp:.2f}_{Tt4:.0f}/{k}", t[k])
        b(f"C/{tag}/tail/{nu_lp:.2f}_{Tt4:.0f}/choked", t["branch"] == "choked")

# ==================================================================== D -- the FUEL closure
# Including b = 0.30, where the bracket walls' 1/(1-b) is what keeps the scan outside the root.
for tag, bb in (("b010", 0.10), ("b030", 0.30)):
    m = bt(bleed=bb)
    Tt2, pt2, _ = m._inlet(FLIGHT)
    eq = m.equilibrium(FLIGHT, 1200.0)
    mf = eq["f"] * eq["mdot_air"]
    c = m._close_fuel(eq["nu_lp"], eq["nu_hp"], mf, Tt2, pt2)
    for k in KEYS + ("Tt4", "mdot_air_face"):
        f(f"D/{tag}/close_fuel/{k}", c[k])

# ==================================================================== E -- THE `at_stator` TRAP
# § 5.21 (ii). The override must hand back a sibling CARRYING THIS MACHINE'S VALVE, so rung
# 59's inherited reader compares the plant with itself and returns its exact identity.
m = bt(bleed_sched=SCHED)
sib = m.at_stator()
b("E/at_stator/sibling_armed", sib._armed_bleed())
b("E/at_stator/sibling_is_scheduled", sib.bleed_sched is not None)
f("E/at_stator/sibling_bleed", sib.bleed)
# ...and with a STATOR argument passed, which is how three of the eight inherited readers
# spell it: the valve must survive AND the passed setting must land.
sib_v = m.at_stator(vsv_lp=V)
b("E/at_stator_v/sibling_armed", sib_v._armed_bleed())
f("E/at_stator_v/vsv_lp", sib_v.vsv_lp)
trap = m.schedule_invariance(FLIGHT, LO, HI, 0.25, n=5)
b("E/trap/ordinate_identical", trap["ordinate_identical"])
b("E/trap/abscissa_identical", trap["abscissa_identical"])
f("E/trap/d_ordinate", trap["d_ordinate"])
f("E/trap/d_abscissa", trap["d_abscissa"])
# The HONEST reader beside it, which differences against a valve-SHUT sibling.
honest = bt().sensed_inputs(FLIGHT, LO, HI, dict(bleed_sched=SCHED), margin=0.25, n=5)
f("E/honest/d_ordinate", honest["d_ordinate"])
f("E/honest/d_abscissa", honest["d_abscissa"])
f("E/honest/signed_ordinate", honest["signed_ordinate"])
f("E/honest/signed_abscissa", honest["signed_abscissa"])
f("E/honest/d_mfp", honest["d_mfp"])

# ==================================================================== F -- the INHERITED leg
# `R62_FUEL` must spread from rung 57's table: rung 62 does not override `_surge_fuel`, so the
# floor a leg resolves is rung 60's, not rung 49's.
m = bt(bleed_sched=SCHED)
lim = SurgeLimiter.from_margin(LP, "lp", 0.40)
cell = m._cell(FLIGHT, LO, HI, 0.5, SETTLE, 0.02, "lp", None, lim, None)
for k in ("m_i", "m_phi", "s", "min_phi", "fuel_removed", "Tt4_peak"):
    f(f"F/floor_leg/{k}", cell[k])
d("F/floor_leg/npts", cell["npts"])

# ==================================================================== G -- THE DISPATCH COUNTS
# § 5.21 (v). The ONLY instrument that can see `_powers` re-reading `b_of`: both spellings
# agree wherever b is 0, so every value key above is blind to it.
def counts(tag, **kw):
    m = bt(**kw)
    t = dict(close_red=0, close_bled=0, fuel_red=0, fuel_bled=0,
             pow_red=0, pow_bled=0, tail_red=0, tail_bled=0)
    rc, rcf, rp, rt = m._close, m._close_fuel, m._powers, m._instant_tail

    def w_close(nu_lp, nu_hp, Tt4, Tt2, pt2):
        t["close_red" if m.b_of(nu_lp, Tt2) == 0.0 else "close_bled"] += 1
        return rc(nu_lp, nu_hp, Tt4, Tt2, pt2)

    def w_fuel(nu_lp, nu_hp, mf, Tt2, pt2):
        t["fuel_red" if m.b_of(nu_lp, Tt2) == 0.0 else "fuel_bled"] += 1
        return rcf(nu_lp, nu_hp, mf, Tt2, pt2)

    def w_pow(c, flight, nu_lp, nu_hp, Tt4):
        t["pow_red" if c.get("bleed", 0.0) == 0.0 else "pow_bled"] += 1
        return rp(c, flight, nu_lp, nu_hp, Tt4)

    def w_tail(flight, c, nu_lp, nu_hp, Tt4, V0):
        t["tail_red" if c.get("bleed", 0.0) == 0.0 else "tail_bled"] += 1
        return rt(flight, c, nu_lp, nu_hp, Tt4, V0)

    m._close, m._close_fuel, m._powers, m._instant_tail = w_close, w_fuel, w_pow, w_tail
    m.equilibrium(FLIGHT, LO)
    m._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, 0.02)
    for k, n in t.items():
        d(f"G/{tag}/{k}", n)


counts("bare")
counts("stator", vsv_sched_lp=STAT)
counts("sched", bleed_sched=SCHED)
counts("both", bleed_sched=SCHED, vsv_sched_lp=STAT)

# ==================================================================== H -- THE REDUCE
# b == 0 dispatches to rung 57's own body VERBATIM at every state, so an unbled machine is
# rung 57 (hence rungs 43-52) bit-for-bit on every recorded key.
RKEYS = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "pi_lpc", "pi_hpc",
         "Phi_lp", "Phi_hp", "sp_thrust", "m_lp", "m_hp", "Tt25", "Tt3")
for tag, kw57, kw62 in (("bare", {}, {}),
                        ("vconst", dict(vsv_lp=V), dict(vsv_lp=V)),
                        ("vsched", dict(vsv_sched_lp=StatorSchedule(V, N_LO)),
                         dict(vsv_sched_lp=StatorSchedule(V, N_LO))),
                        ("bmax0", {}, dict(bleed_sched=BleedSchedule(0.0, N_LO)))):
    a = ScheduledStatorTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw57)
    c = bt(**kw62)
    for Tt4 in (1000.0, 1400.0):
        ea, ec = a.equilibrium(FLIGHT, Tt4), c.equilibrium(FLIGHT, Tt4)
        for k in RKEYS:
            assert ea[k] == ec[k], f"{tag} {Tt4} {k}: {ea[k]!r} != {ec[k]!r}"
            f(f"H/{tag}/{Tt4:.0f}/{k}", ec[k])

# ==================================================================== J -- `_legs`, THE RUNG
# The loop a state-fed schedule closes on itself. `self_cancel > 1` is the headline.
for tag, kw in (("bleed", dict(bleed_sched=SCHED)), ("stator", dict(vsv_sched_lp=STAT))):
    m = bt(**kw)
    r = m.loop_decomposition(FLIGHT, LO, HI, r=0.5, ds=0.02)
    for k in ("reference", "start", "ramp", "full", "self_cancel", "surrendered",
              "share_start", "loop", "nu0_ref", "nu0_armed", "cmd_ramp", "cmd_full",
              "s_ref", "s_ramp", "s_full"):
        f(f"J/{tag}/{k}", r[k])
    b(f"J/{tag}/lever_is_bleed", r["lever"] == "bleed")

# ==================================================================== K -- `_isolating`
m = bt()
ref, armed = m._isolating(dict(bleed_sched=SCHED))
b("K/plain/ref_armed", ref._armed_bleed())
b("K/plain/armed_armed", armed._armed_bleed())
ref2, armed2 = m._isolating(dict(bleed_sched=SCHED), neighbour=dict(vsv_sched_lp=STAT))
b("K/neighbour/ref_armed", ref2._armed_bleed())
b("K/neighbour/armed_armed", armed2._armed_bleed())
b("K/neighbour/ref_is_armed_stator", ref2._is_armed())
b("K/neighbour/armed_is_armed_stator", armed2._is_armed())

# ---------------------------------------------------------------------------- emit
print("# slice W step 2 -- rungs 62-63 SMOKE. key<TAB>u64 (float keys are IEEE-754 bits).")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys", file=sys.stderr)
