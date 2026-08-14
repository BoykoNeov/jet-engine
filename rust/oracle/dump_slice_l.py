"""THE ORACLE, phase 5 slice L — every rung-41/42 value the Rust must reproduce.

WHAT IS NEW HERE, and why each thing is dumped rather than asserted:

  * A DISPATCH, NOT A NUMBER. Rung 42 is the port's first override of a live virtual slot:
    rung 41's three schedule methods call `self.match`, and on a rung-42 object that must reach
    rung 42's body. Naming rung 39's function in the Rust hook table compiles and returns
    numbers. So § 4 sweeps all three methods on a RUNG-42 core at `b > 0` — the narrowing
    § 5.8's step-4 line called out: § 5.8.1's grid ran them on rung-39 matchers only, and a dump
    built on that witnesses the dispatch through `surge_margin` alone. The discriminating
    question for every key below is *could rung 39's body have produced this?*

  * A MISSING VALUE AND A MISSING OBJECT, BOTH OF WHICH A FLOAT DUMP IS BLIND TO.
    `flow_coefficient_turn` returns `kind="RAIL"` with `pi_star`/`star_form` = None and
    `gamma_c`/`far` ABSENT from the dict; and `match` at `b = 0` returns rung 39's object, which
    has no booking attributes at all. Both are dumped as an explicit DISCRIMINANT plus a
    DECLARED SENTINEL (`NULL`, below), and the branch COUNTS are gated — never by omitting the
    key on the null rows, which would leave the key-count guard blind to a class absent from
    both sides.

  * THE MEMO KEY SEQUENCE (P4). `cache[key] = self.match(flight, key)` passes the ROUNDED value
    on as the throttle, so the rounding moves VALUES. `round(x, 6)` is correctly-rounded
    half-to-EVEN and the naive `(x*1e6).round()/1e6` is not — demonstrably so at 350.0078125.
    Every key a turn matched at is dumped in call order.

  * A CENSUS THAT MEASURES PHYSICS. Rung 42's UNCHOKED count rises with `b` (23/23/24/25 at
    b = 0.00/0.02/0.05/0.10) — that is rung 42's own gate 6, *opening the valve shrinks the
    choked envelope*, read as a count. Its zero rows are dumped as explicit 0s.

  * WHAT IS DELIBERATELY *NOT* HERE: the refinement count 33. Python cannot instrument the
    shipped body's two phases apart from outside, so its arm would be a transcription and the
    comparison would be self-confirming (rung 83's identity-round-trip shape). The 33 is gated
    in `rung41.rs` against the arithmetic instead: ceil(ln(1e-5/20)/ln(0.618...)) + 2.

Regenerate with:
    py -3                     rust/oracle/dump_slice_l.py rust/oracle/slice_l_cpython.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_l.py rust/oracle/slice_l_pypy.tsv
"""
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.engine import (  # noqa: E402
    ComponentMap, FlightCondition, TwoSpoolBleedMatcher, TwoSpoolMapMatcher,
    build_two_spool_turbojet,
)
from turbojet.gas import Gas  # noqa: E402

T0 = time.time()
ROWS = []

# THE DECLARED SENTINEL for every nullable column. Chosen impossible, not merely unusual: the
# four `RAIL`-nulled turn fields are a pressure ratio (>1), a `1+eta*(tau-1)` form (>1), a heat
# capacity ratio (>1) and a fuel-air ratio (>0); the four booking fields are a fraction in
# [0, 0.5), a mass flow, a specific thrust and a TSFC, all positive on the swept grid. A negative
# value cannot be produced by any of them, so a Rust row writing a real number where Python
# writes None can never compare equal by accident.
NULL = -1.0

# Discriminant codes. Kept as small integers so `quant_of` in the gate can bar them at 0.0.
KIND = {"MIN": 0.0, "RAIL": 1.0}
BINDING = {"lp": 0.0, "hp": 1.0}


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
FLIGHT_M16 = FlightCondition(T0=250.0, p0=50_000.0, M0=1.60)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)

# THE GRID. Slice K's 147 cells (3 gases x 7 M0 x 7 Tt4), inherited wholesale; the BLEED axis is
# what slice L adds. Written down because a census read off one grid and gated on another is how
# § 5.6's P2 got its number wrong.
M0S = [0.0, 0.3, 0.5, 0.85, 1.2, 1.6, 2.0]
TT4S = [400.0, 500.0, 600.0, 650.0, 900.0, 1100.0, 1500.0]
BLEEDS = [0.00, 0.02, 0.05, 0.10]

# Rung 41's OWN throttle grid for the schedules, verbatim from the slice's probe — 13 points, of
# which 10 survive on most (gas, shape, floor) cells. The SKIP is control flow, so the length of
# each returned list is itself a measurement.
GRID41 = [1500.0, 1300.0, 1100.0, 950.0, 900.0, 850.0, 800.0, 750.0, 700.0, 650.0,
          600.0, 500.0, 400.0]

# Rung 41's OWN shape pairs, verbatim from tests/test_rung41.py.
LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
TILTED = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)
STEEP = ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)
SHAPES41 = [
    ("flow_press", LP_SHAPED, HP_SHAPED),
    ("press_flow", ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
     ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    ("tilted", TILTED, TILTED),
    ("steep", STEEP, STEEP),
]
FLOORS = [0.50, 0.55]

# Abort codes. SLICE I's TABLE as slice K appended to it — never renumbered, because the Rust
# compares the code and a renumber would silently re-label every cell in the older dumps. Rung
# 42's own messages fall into the existing classes by construction: "rung-42 bleed match
# unphysical" -> 9, its UNCHOKED guard -> 8, its LP secant -> 11, its turbine loop -> 12.
ABORT = {
    "": 0.0,
    "SUB-IDLE": 1.0,
    "efficiency cascade": 2.0,
    "inverse: root not bracketed": 3.0,
    "equilibrium Newton": 4.0,
    "off-design burner f did not converge": 5.0,
    "nozzle back-pressure": 6.0,
    "ram must not cool/depressurize": 7.0,
    "UNCHOKED": 8.0,
    "unphysical": 9.0,
    "does not straddle": 10.0,
    "efficiency secant did not converge": 11.0,
    "turbine-efficiency loop did not converge": 12.0,
    "speed-line bracket fails": 13.0,
    "shaft does not close": 14.0,
}


def abort_code(msg):
    for tag, code in ABORT.items():
        if tag and tag in msg:
            return code
    raise AssertionError(f"UNCLASSIFIED abort, add it to ABORT: {msg[:120]}")


def cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """The SELF-CONSISTENT CPG dual gas: R = (g-1)/g*cp EXACTLY — slices I/K's helper, and for
    its reason (a rounded R breaks the closed forms the rung gates compare the solver against)."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


GASES = [("cpg", cpg_gas), ("tpg", Gas.thermally_perfect), ("eq", Gas.reacting_equilibrium)]


def design_of(gas, pi_lpc=PI_LPC, pi_hpc=PI_HPC, real=None):
    return build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **(real or REAL))


class Count42(TwoSpoolBleedMatcher):
    """Rung 42 with the joint-loop counter and NO arithmetic of its own.

    BOTH cascades are counted into one `n_pass`, deliberately: at `b = 0` the match dispatches to
    rung 39's body and runs `_cascade_map`, at `b > 0` it runs `_cascade_bleed`, and the Rust
    feeds both into the SAME `counters::bump_cascade`. One cell only ever runs one matcher, so the
    two never mix inside a reading."""

    def reset(self):
        self.n_pass = 0
        return self

    def _cascade_map(self, *a, **k):
        self.n_pass += 1
        return super()._cascade_map(*a, **k)

    def _cascade_bleed(self, *a, **k):
        self.n_pass += 1
        return super()._cascade_bleed(*a, **k)


class KeyLog(TwoSpoolBleedMatcher):
    """P4's instrument: every throttle `match` was CALLED at, in order.

    `flow_coefficient_turn`'s memo calls `self.match(flight, key)` only on a MISS, so this records
    the miss sequence — which is exactly what P4 is about, since the key passed IS the throttle
    solved. Recorded at ENTRY so a FAILING match (the one that ends the coarse scan) is recorded
    too; the Rust's `note_key` sits on the same branch, before `try_match_point`."""

    def reset(self):
        self.keys = []
        return self

    def match(self, flight, Tt4):
        self.keys.append(float(Tt4))
        return super().match(flight, Tt4)


def put_turn(tag, t):
    """One `flow_coefficient_turn` result — the DISCRIMINANT first, then every field, with the
    four `RAIL`-nulled ones written as the declared sentinel rather than omitted."""
    put(f"{tag}/kind", KIND[t["kind"]])
    put(f"{tag}/Tt4_star", t["Tt4_star"])
    put(f"{tag}/phi_star", t["phi_star"])
    put(f"{tag}/closed_form", t["closed_form"])
    put(f"{tag}/band_lo", t["band"][0])
    put(f"{tag}/band_hi", t["band"][1])
    for k in ("pi_star", "star_form"):
        v = t[k]
        put(f"{tag}/{k}", NULL if v is None else v)
    for k in ("gamma_c", "far"):        # ABSENT from the RAIL dict, not merely None
        put(f"{tag}/{k}", t.get(k, NULL))


SM_FIELDS = ("Tt4", "x_lp", "x_hp", "phi_lp", "phi_hp", "n_lp", "n_hp", "pi_lpc", "pi_hpc",
             "slip", "SM_lp", "SM_hp")
RL_FIELDS = ("Tt4", "x_lp", "x_hp", "phi_lp", "phi_hp", "n_lp", "n_hp", "pi_lpc", "pi_hpc")


def put_schedules(tag, mm, flight):
    """`surge_margin_schedule` + `running_line_map`, EVERY field of EVERY row plus the LENGTHS.

    The lengths are the SKIP census — control flow, not error handling — and § 5.8.2 (c)'s lesson
    applies to the fields: `running_line_map`'s output feeds nothing downstream, so a transposed
    `x_lp`/`x_hp` or `pi_lpc`/`pi_hpc` pair would be revealed by no other number in the port."""
    sched = mm.surge_margin_schedule(flight, GRID41)
    put(f"{tag}/sched/n", float(len(sched)))
    for i, r in enumerate(sched):
        for k in SM_FIELDS:
            put(f"{tag}/sched/{i}/{k}", r[k])
        put(f"{tag}/sched/{i}/binding", BINDING[r["binding"]])
    rl = mm.running_line_map(flight, GRID41)
    put(f"{tag}/runline/n", float(len(rl)))
    for i, r in enumerate(rl):
        for k in RL_FIELDS:
            put(f"{tag}/runline/{i}/{k}", r[k])
    return len(sched), len(rl)


# ==============================================================================================
# 1. RUNG 41's ZERO-NEW-CONSTANT CLOSED FORM, and `phi_surge`'s arrival on the map
#
#    pi* = gamma_c^(gamma_c/(gamma_c-1)) from the COLD gamma alone. Cheap, and it pins the one
#    arithmetic the value grid below could absorb: a `powf`-vs-`powp` spelling in the exponent.
# ==============================================================================================
DESIGNS = {}
for gname, gmk in GASES:
    DESIGNS[gname] = design_of(gmk())
    mm = TwoSpoolMapMatcher(DESIGNS[gname], FLIGHT, 1.0)
    put(f"pistar/{gname}", mm.critical_flow_turn_pi())
for i, gc in enumerate((1.30, 1.35, 1.40, 1.45)):
    d = design_of(cpg_gas(gamma_c=gc))
    put(f"pistar/gamma_c/{i}", TwoSpoolMapMatcher(d, FLIGHT, 1.0).critical_flow_turn_pi())
# `phi_surge` is carried and NEVER read by the solver — the field arrived in this slice, so the
# ONE thing to pin is that `with_phi_surge` copies the map and moves nothing else.
_bare = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
_armed = _bare.with_phi_surge(0.55)
put("phi_surge/carried", _armed.phi_surge)
for j, phi in enumerate((0.55, 0.85, 1.0, 1.2)):
    put(f"phi_surge/psi_bare/{j}", _bare.psi(phi))
    put(f"phi_surge/psi_armed/{j}", _armed.psi(phi))

print(f"[1] closed forms, {time.time() - T0:.1f}s")

# ==============================================================================================
# 2. RUNG 42's GRID — 147 cells PER BLEED LEVEL, and for every abort, WHY
#
#    ON THE `mixed` SHAPE PAIR (LP_SHAPED, HP_SHAPED), WHICH IS NOT A FREE CHOICE. Every number
#    § 5.8.1 (v) pre-registered — 67/67/66/65 matched, UNCHOKED 23/23/24/25 — was measured on this
#    pair, and P7's census half is that `b` = 0 reproduces slice K's rung-39 row EXACTLY, which is
#    slice K's `mixed` row (one of its two shapes getting the full M0 grid). Swept on FLAT maps
#    instead this dump read 68/68/68/67 with UNCHOKED flat at 23 — numbers that are perfectly
#    correct and answer a different question, and that would have looked like a REFUTATION of
#    rung 42's own gate 6. § 5.7 (e)'s rule, caught in the act: a bar is measured on the grid it
#    will be gated on, never read off a neighbouring one.
# ==============================================================================================
CENSUS = {}
FLAT = ComponentMap()
for b in BLEEDS:
    CENSUS[b] = dict((k, 0) for k in ABORT.values())
    n_ok = 0
    for gname, _ in GASES:
        d = DESIGNS[gname]
        for M0 in M0S:
            flight = FlightCondition(T0=250.0, p0=FLIGHT.p0, M0=M0)
            for Tt4 in TT4S:
                tag = f"r42/{b:.2f}/{gname}/{M0:.2f}/{Tt4:.0f}"
                m = Count42(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED,
                            bleed=b).reset()
                try:
                    od = m.match(flight, Tt4)
                except AssertionError as e:
                    code = abort_code(str(e).split("\n")[0])
                    put(f"{tag}/abort", code)
                    CENSUS[b][code] += 1
                    continue
                put(f"{tag}/abort", ABORT[""])
                CENSUS[b][0.0] += 1
                n_ok += 1
                # THE PASS COUNT — bit-gated against PyPy only (§ 5.7 (c)): the joint loop's
                # stopping rule is unmeetable by a hair on the integral gases.
                put(f"{tag}/n_pass", float(m.n_pass))
                for name, v in (("pi_lpc", od.pi_lpc), ("pi_hpc", od.pi_hpc),
                                ("eta_lpc", od.eta_lpc), ("eta_hpc", od.eta_hpc),
                                ("eta_hpt", od.eta_hpt), ("eta_lpt", od.eta_lpt),
                                ("n_lp", od.n_lp), ("n_hp", od.n_hp),
                                ("N_lp_ratio", od.N_lp_ratio), ("N_hp_ratio", od.N_hp_ratio),
                                ("slip", od.slip), ("phi_lp", od.phi_lp),
                                ("phi_hp", od.phi_hp), ("nu_hpt", od.nu_hpt),
                                ("nu_lpt", od.nu_lpt), ("tau_hpt", od.tau_hpt),
                                ("tau_lpt", od.tau_lpt), ("tau_lpc", od.tau_lpc),
                                ("tau_hpc", od.tau_hpc), ("pi_hpt", od.pi_hpt),
                                ("pi_lpt", od.pi_lpt), ("mdot_air", od.mdot_air),
                                ("mdot_ratio", od.mdot_ratio), ("thrust", od.thrust),
                                ("V0", od.V0), ("V9", od.V9), ("M9", od.M9),
                                ("T9", od.T9), ("p9", od.p9),
                                ("F_over_mdot", od.performance.specific_thrust),
                                ("tsfc", od.performance.tsfc),
                                ("eta_th", od.performance.eta_thermal),
                                ("eta_p", od.performance.eta_propulsive)):
                    put(f"{tag}/{name}", v)
                for st in ("2", "25", "3", "4", "45", "5", "9"):
                    s = od.stations[st]
                    put(f"{tag}/s{st}/Tt", s.Tt)
                    put(f"{tag}/s{st}/pt", s.pt)
                # mdot IS the extraction's only visible trace — nothing downstream reads it
                # (`_score` never touches mass flow), so it is dumped where it can be seen.
                for st in ("2", "25", "3", "4"):
                    put(f"{tag}/s{st}/mdot", od.stations[st].mdot)
                put(f"{tag}/s4/far", od.stations["4"].far)
                # THE BOOKING, as a discriminant plus the declared sentinel: Python never
                # CONSTRUCTS a TwoSpoolBleedResult at b = 0, so these four attributes are ABSENT
                # there and `bleed_trade` reads that absence through `getattr`.
                has = 0.0 if hasattr(od, "st_inlet") else 1.0
                put(f"{tag}/booking_absent", has)
                for k in ("bleed", "mdot_core", "st_inlet", "tsfc_inlet"):
                    put(f"{tag}/{k}", getattr(od, k, NULL))
    for code in sorted(set(ABORT.values())):
        put(f"census/r42/{b:.2f}/abort_code/{code:.0f}", float(CENSUS[b][code]))
    print(f"[2] b={b:.2f}: {n_ok} matched of {len(GASES) * len(M0S) * len(TT4S)}, "
          f"UNCHOKED {CENSUS[b][8.0]:.0f}, {time.time() - T0:.1f}s")

# ==============================================================================================
# 3. RUNG 41's SCHEDULES on RUNG-39 matchers — 3 gases x 4 shapes x 2 floors
#
#    The baseline half: what the methods do with no valve anywhere. § 4 is what makes it a
#    DISPATCH test rather than a transcription test.
# ==============================================================================================
for gname, _ in GASES:
    d = DESIGNS[gname]
    for sname, ml, mh in SHAPES41:
        for floor in FLOORS:
            mm = TwoSpoolMapMatcher(d, FLIGHT, 1.0, map_lp=ml.with_phi_surge(floor),
                                    map_hp=mh.with_phi_surge(floor))
            put_schedules(f"r41/{gname}/{sname}/{floor:.2f}", mm, FLIGHT)
            # gate 2's non-tautological reproduction, dumped per cell at one throttle: the
            # margin is measured on the very forward map that sets that spool's running line.
            try:
                od = mm.match(FLIGHT, 1100.0)
            except AssertionError:
                continue
            for sp in ("lp", "hp"):
                put(f"r41/{gname}/{sname}/{floor:.2f}/pi_shipped_{sp}",
                    mm._pi_c_spool_shipped(od, sp))

print(f"[3] rung 41 schedules on rung-39 matchers, {time.time() - T0:.1f}s")

# ==============================================================================================
# 4. THE NARROWING FIX — ALL THREE rung-41 methods on a RUNG-42 CORE, at b > 0
#
#    § 5.8's step-4 line: § 5.8.1's grid swept rung 41's schedules on rung-39 matchers ONLY, so a
#    dump built on it witnesses the dispatch through `surge_margin` alone. Every key here is one
#    rung 39's body CANNOT have produced, because the valve moves the point it is read at.
#
#    The cells sit at the sweep's EDGES on purpose: § 5.8.3 (h) measured the check's sensitivity
#    to a mis-associated `(1-b)` at ~2 % of rows, and the rows that moved were at M0 = 1.60 or on
#    the equilibrium gas. A comfortable mid-band cell would have passed the defect.
# ==============================================================================================
# The branch census is kept PER BLOCK, not pooled. § 5.8.1 (viii)'s registered numbers are
# "60 MIN / 20 RAIL over the shaped grid" and "16 of 19 `lp` cases RAIL on the flat one" — two
# different populations, and a single pooled counter answers neither. (First draft pooled them
# and reported 32/22 over 54 runs, a number no prediction is written against.)
N42 = {"MIN": 0, "RAIL": 0}
NFLAT = {"MIN": 0, "RAIL": 0}
NFLAT_LP_RAIL = 0
N_ENDED_ON_ABORT = 0
N_TURNS = 0
for gname, _ in GASES:
    d = DESIGNS[gname]
    for b in (0.0, 0.10):
        for flname, fl in (("0.85", FLIGHT), ("1.60", FLIGHT_M16)):
            bm = TwoSpoolBleedMatcher(d, FLIGHT, 1.0,
                                      map_lp=LP_SHAPED.with_phi_surge(0.55),
                                      map_hp=HP_SHAPED.with_phi_surge(0.55), bleed=b)
            tag = f"r42sched/{gname}/{b:.2f}/{flname}"
            ns, nr = put_schedules(tag, bm, fl)
            print(f"    {tag}: sched {ns}/{len(GRID41)}  runline {nr}/{len(GRID41)}")
    # THE TURNS. Both spools, both bleed levels, at the design flight; the M0 = 1.60 edge is
    # swept on the CPG gas only, where a turn run is cheap enough to afford both spools.
    for b in (0.0, 0.10):
        for spool in ("hp", "lp"):
            bm = TwoSpoolBleedMatcher(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED,
                                      bleed=b)
            t = bm.flow_coefficient_turn(FLIGHT, spool)
            put_turn(f"r42turn/{gname}/{b:.2f}/{spool}", t)
            N42[t["kind"]] += 1
            N_TURNS += 1
            N_ENDED_ON_ABORT += 1 if t["band"][0] > 350.0 + 10.0 else 0

for b in (0.0, 0.10):
    for spool in ("hp", "lp"):
        bm = TwoSpoolBleedMatcher(DESIGNS["cpg"], FLIGHT, 1.0, map_lp=LP_SHAPED,
                                  map_hp=HP_SHAPED, bleed=b)
        t = bm.flow_coefficient_turn(FLIGHT_M16, spool)
        put_turn(f"r42turn/cpg/M16/{b:.2f}/{spool}", t)
        N42[t["kind"]] += 1
        N_TURNS += 1
        N_ENDED_ON_ABORT += 1 if t["band"][0] > 350.0 + 10.0 else 0

# `bleed_trade` — the one path where the dispatch was ALREADY witnessed (via `surge_margin`), on
# the armed maps that make both margins present. At b = 0 the row reads the CORE numbers through
# the absent booking, which is the `getattr` fallback expressed as data.
for gname, _ in GASES:
    bm = TwoSpoolBleedMatcher(DESIGNS[gname], FLIGHT, 1.0,
                              map_lp=LP_SHAPED.with_phi_surge(0.55),
                              map_hp=HP_SHAPED.with_phi_surge(0.55), bleed=0.0)
    for Tt4 in (1500.0, 1300.0, 1100.0):
        rows = bm.bleed_trade(FLIGHT, Tt4, bleeds=(0.0, 0.05, 0.10))
        put(f"trade/{gname}/{Tt4:.0f}/n", float(len(rows)))
        for i, r in enumerate(rows):
            for k in ("bleed", "Tt4", "phi_lp", "phi_hp", "n_lp", "n_hp", "pi_lpc", "pi_hpc",
                      "Tt25", "slip", "mdot_air", "thrust", "st_inlet", "tsfc",
                      "SM_lp", "SM_hp"):
                put(f"trade/{gname}/{Tt4:.0f}/{i}/{k}", r[k])
        # the valve is RESTORED by the `finally` — dumped, because a Rust port that mutates and
        # forgets would leave every later reading on the wrong machine.
        put(f"trade/{gname}/{Tt4:.0f}/bleed_after", bm.bleed)

print(f"[4] the narrowing fix — three methods on a rung-42 core, {time.time() - T0:.1f}s")

# ==============================================================================================
# 5. `flow_coefficient_turn` ON FLAT MAPS — gate 5's OWN 19 cases x both spools (P3, P9)
#
#    Gate 5 only ever calls the `hp` spool; the `lp` column here is WIDER than any shipped gate,
#    deliberately, because P9's RAIL branch is where the LP spool normally lives (16 of 19).
# ==============================================================================================
CASES = []
for nm, kw in (("base", dict()),
               ("split_4.5x4", dict(pi_lpc=4.5, pi_hpc=4.0)),
               ("split_2.25x8", dict(pi_lpc=2.25, pi_hpc=8.0)),
               ("eta_hpc_.80", dict(real=dict(REAL, eta_hpc=0.80))),
               ("eta_hpc_.95", dict(real=dict(REAL, eta_hpc=0.95))),
               ("eta_hpt_.85", dict(real=dict(REAL, eta_hpt=0.85))),
               ("eta_lpc_.80", dict(real=dict(REAL, eta_lpc=0.80)))):
    CASES.append((nm, cpg_gas(), kw, FLIGHT))
for gc in (1.30, 1.35, 1.40, 1.45):
    CASES.append((f"gamma_c_{gc}", cpg_gas(gamma_c=gc), dict(), FLIGHT))
CASES.append(("gamma_t_1.25", cpg_gas(gamma_t=1.25), dict(), FLIGHT))
CASES.append(("cp_t_1300", cpg_gas(cp_t=1300.0), dict(), FLIGHT))
for i, hPR in enumerate((4.28e8, 4.28e9, 4.28e10)):
    CASES.append((f"hPR_{i}", cpg_gas(hPR=hPR), dict(), FLIGHT))
CASES.append(("M0_1.60", cpg_gas(), dict(), FLIGHT_M16))
CASES.append(("tpg", Gas.thermally_perfect(), dict(), FLIGHT))
CASES.append(("tpg_M0_1.60", Gas.thermally_perfect(), dict(), FLIGHT_M16))

for nm, gas, kw, fl in CASES:
    d = design_of(gas, **kw)
    for spool in ("hp", "lp"):
        # NOTE the matcher is built at FLIGHT and matched at `fl` — gate 5's own construction.
        mm = TwoSpoolMapMatcher(d, FLIGHT, 1.0)
        t = mm.flow_coefficient_turn(fl, spool)
        put_turn(f"turn/{nm}/{spool}", t)
        NFLAT[t["kind"]] += 1
        if spool == "lp" and t["kind"] == "RAIL":
            NFLAT_LP_RAIL += 1
        N_TURNS += 1
        N_ENDED_ON_ABORT += 1 if t["band"][0] > 350.0 + 10.0 else 0

# P9's branch COUNT — the gate asserts this and not only the numbers, because a float dump is
# structurally blind to a RAIL row whose null columns carry a plausible number.
put("census/turn/n", float(N_TURNS))
put("census/turn42/MIN", float(N42["MIN"]))
put("census/turn42/RAIL", float(N42["RAIL"]))
put("census/turnflat/MIN", float(NFLAT["MIN"]))
put("census/turnflat/RAIL", float(NFLAT["RAIL"]))
# § 5.8.1 (viii)'s own number: the RAIL branch is where the LP spool normally LIVES, so it is
# counted on its own rather than left inside the pooled RAIL total.
put("census/turnflat/lp_RAIL", float(NFLAT_LP_RAIL))
# P3's SECOND half, as data: the coarse scan always ends on the ABORT, never on `Tt4_lo`, so the
# runnable band's low end is set by the choked envelope and the parameter is DEAD.
put("census/turn/ended_on_abort", float(N_ENDED_ON_ABORT))

print(f"[5] flat-map turns: {NFLAT['MIN']} MIN / {NFLAT['RAIL']} RAIL "
      f"(lp RAIL {NFLAT_LP_RAIL} of {len(CASES)}); rung-42-core turns: {N42['MIN']} MIN / "
      f"{N42['RAIL']} RAIL; ended-on-abort {N_ENDED_ON_ABORT} of {N_TURNS}, "
      f"{time.time() - T0:.1f}s")

# ==============================================================================================
# 6. THE MEMO KEY SEQUENCE (P4) — dumped as discrete oracle values
#
#    Four representative turns: a MIN and a RAIL on the rung-39 path, and BOTH on a rung-42 core
#    with the valve open (where the branch can FLIP, because bleed moves phi and therefore the
#    argmin index — so the two key sequences are not the same object).
# ==============================================================================================
for tag, gname, bleed, spool in (("r39_hp", "cpg", 0.0, "hp"), ("r39_lp", "cpg", 0.0, "lp"),
                                 ("r42_hp", "cpg", 0.10, "hp"), ("r42_lp", "cpg", 0.10, "lp")):
    kl = KeyLog(DESIGNS[gname], FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED,
                bleed=bleed).reset()
    t = kl.flow_coefficient_turn(FLIGHT, spool)
    put(f"keys/{tag}/kind", KIND[t["kind"]])
    put(f"keys/{tag}/n", float(len(kl.keys)))
    for i, k in enumerate(kl.keys):
        put(f"keys/{tag}/{i}", k)
    print(f"    keys/{tag}: {len(kl.keys)} keys, {t['kind']}")

print(f"[6] memo key sequences, {time.time() - T0:.1f}s")

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-5L rung-41/42 surge-line + bleed oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
