"""THE ORACLE, phase 5 slice K — every rung-38/39 value the Rust must reproduce.

WHAT IS NEW HERE, and why each thing is dumped rather than asserted:

  * A THIRD CHOKED THROAT. Rung 31 captured two areas from one design run; rung 38 captures
    three (`A4`, `A45`, `A8`) and chains the same (★) solver twice. All three are dumped, because
    a wrong area is a silent multiplier on every number downstream of it.

  * A JOINT LOOP THAT CAPS FAR MORE OFTEN THAN SLICE I's. The single-spool `(f, pt4)` fixed
    point exhausted its 200-pass cap on two cells; this one does on 23 of 105 matched cells, and
    when it caps BOTH halves of the stopping rule cycle together (§ 5.7 (b)). The per-cell pass
    count is therefore dumped as a value: a divergence shows up as a COUNT rather than as an
    unexplained last-bits drift.

  * AND WHETHER IT CAPS IS INTERPRETER-DEPENDENT — 8 vs 200 at 29 of 126 cells, every one on the
    equilibrium gas (§ 5.7 (c)). So `two_spool_oracle.rs` bit-gates the pass count against PyPy
    ONLY, and the CPython arm excludes it explicitly rather than by a loose tolerance.

  * THE ENVELOPE, AS DATA, ON A GRID EXTENDED BY ONE COLUMN. Slice I's grid starts at M0 = 0.3.
    `M0 = 0` is added here because it is excluded by a SOLVER ROUND-TRIP rather than by physics
    — `T_from_h_c(h_c(250))` lands three ulps low on the integral gases, failing the FIRST clause
    of a two-clause ram assert while the second is exact — and a Rust round-trip landing on the
    other side of exactness would move an envelope boundary no value comparison can see.

  * TWO NEW ABORT CODES, APPENDED. Slice I's table is reused verbatim and extended by 7 and 8;
    renumbering would silently re-label every existing cell.

Regenerate with:
    py -3                     rust/oracle/dump_two_spool.py rust/oracle/two_spool_cpython.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_two_spool.py rust/oracle/two_spool_pypy.tsv
"""
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.engine import (  # noqa: E402
    ComponentMap, FlightCondition, MapMatcher, OffDesignMatcher, TwoSpoolMapMatcher,
    TwoSpoolMatcher, build_turbojet, build_two_spool_turbojet,
)
from turbojet.gas import Gas  # noqa: E402

T0 = time.time()
ROWS = []


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)

# THE GRID. Slice I's, plus the M0 = 0 column (see the header).
M0S = [0.0, 0.3, 0.5, 0.85, 1.2, 1.6, 2.0]
TT4S = [400.0, 500.0, 600.0, 650.0, 900.0, 1100.0, 1500.0]
# Rung 39 multiplies by map SHAPE, so only two shapes get the full grid; the other three get the
# two flight Machs that matter and every throttle. Stated here because a census read off one grid
# and gated on another is how slice J's P2 got its number wrong.
M0S_NARROW = [0.85, 1.6]

# Rung 39's OWN shapes, copied verbatim from tests/test_rung39.py — note `l`, which slice K put
# on the Rust `ComponentMap` precisely because these set it (§ 5.7 (a)).
SHAPES = [
    ("flat", ComponentMap(), ComponentMap(), True),
    ("mixed", ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
     ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0), True),
    ("flow_dom", ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
     ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7), False),
    ("press_dom", ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
     ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0), False),
    ("tilted", ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85),
     ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85), False),
    # a_t != 0: the ONLY shape that makes the outer turbine-efficiency loop run more than once.
    ("turb", ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7, a_t=0.02),
     ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7, a_t=0.02), False),
]

# Abort codes. SLICE I's TABLE, APPENDED TO — never renumbered, because the Rust compares the
# code and a renumber would silently re-label every cell in the older dump.
ABORT = {
    "": 0.0,
    "SUB-IDLE": 1.0,
    "efficiency cascade": 2.0,
    "inverse: root not bracketed": 3.0,
    "equilibrium Newton": 4.0,
    "off-design burner f did not converge": 5.0,
    "nozzle back-pressure": 6.0,
    "ram must not cool/depressurize": 7.0,     # NEW — the M0 = 0 round-trip (§ 5.7 (f))
    "UNCHOKED": 8.0,                            # NEW — rung 38's own scope guard
    "unphysical": 9.0,                          # rung 38/39's own physicality check
    "does not straddle": 10.0,                  # the (★) bracket
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


def cpg_gas():
    """The SELF-CONSISTENT CPG dual gas: R_t = (g-1)/g*cp_t EXACTLY — slice I's helper, and for
    its reason (a rounded R_t breaks the closed forms two gates compare the solver against)."""
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp,
               R_t=(g - 1.0) / g * cp, hPR=42.8e6)


GASES = [("cpg", cpg_gas), ("tpg", Gas.thermally_perfect), ("eq", Gas.reacting_equilibrium)]


class TauCount(object):
    """Delegates EVERYTHING; counts `T_from_pr_t`, which the (★) residual's `tau_of` calls
    exactly once. The shipped bisection is untouched, so the count is the shipped loop's — the
    same discipline slice I used with a counting subclass, applied where the thing to count is a
    CLOSURE and cannot be overridden."""

    def __init__(self, inner, sink):
        object.__setattr__(self, "_inner", inner)
        object.__setattr__(self, "_sink", sink)

    def T_from_pr_t(self, *a, **k):
        self._sink[0] += 1
        return self._inner.T_from_pr_t(*a, **k)

    def __getattr__(self, name):
        return getattr(object.__getattribute__(self, "_inner"), name)


class Count38(TwoSpoolMatcher):
    """Rung 38 with counters and NO arithmetic of its own."""

    def reset(self):
        self.n_pass = 0
        self.tau_per_solve = []
        return self

    def _cascade(self, *a, **k):
        self.n_pass += 1
        return super()._cascade(*a, **k)

    def _solve_choked_turbine(self, gas, *a, **k):
        sink = [0]
        out = super()._solve_choked_turbine(TauCount(gas, sink), *a, **k)
        self.tau_per_solve.append(sink[0])
        return out


class Count39(TwoSpoolMapMatcher):
    """Rung 39 likewise: joint passes, outer turbine passes, per-loop secant passes, clamps."""

    def reset(self):
        self.n_pass = 0
        self.n_turb = []
        self.hp_passes = []
        self.lp_passes = []
        self.n_secant = 0
        self.n_clamp = 0
        return self

    def _cascade_map(self, *a, **k):
        self.n_pass += 1
        self._turb_this = 0
        out = super()._cascade_map(*a, **k)
        self.n_turb.append(self._turb_this)
        return out

    def _solve_choked_turbine(self, gas, *a, **k):
        if a and a[2] == self.A4:      # the HP call: one per outer turbine-efficiency pass
            self._turb_this += 1
        return super()._solve_choked_turbine(gas, *a, **k)

    def _hp_eta_loop(self, *a, **k):
        n0 = self.n_secant
        out = super()._hp_eta_loop(*a, **k)
        self.hp_passes.append(self.n_secant - n0)
        return out

    def _lp_eta_loop(self, *a, **k):
        n0 = self.n_secant
        out = super()._lp_eta_loop(*a, **k)
        self.lp_passes.append(self.n_secant - n0)
        return out

    def _secant(self, eta, eta_prev, R, R_prev, target):
        self.n_secant += 1
        raw = target if (eta_prev is None or abs(R - R_prev) < 1e-300) \
            else eta - R * (eta - eta_prev) / (R - R_prev)
        if raw < 0.3 or raw > 1.0:
            self.n_clamp += 1
        return TwoSpoolMapMatcher._secant(eta, eta_prev, R, R_prev, target)


def design_of(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


# ==============================================================================================
# 1. THE CAPTURED HARDWARE — three throats, and rung 39's per-face design references
# ==============================================================================================
DESIGNS, M38, M39 = {}, {}, {}
for gname, gmk in GASES:
    gas = gmk()
    d = design_of(gas)
    DESIGNS[gname] = d
    m = Count38(d, FLIGHT, 1.0)
    M38[gname] = m
    put(f"hw/{gname}/A4", m.A4)
    put(f"hw/{gname}/A45", m.A45)        # THE THIRD THROAT — rung 38's structural novelty
    put(f"hw/{gname}/A8", m.A8)
    put(f"hw/{gname}/f_design", m.f_design)
    put(f"hw/{gname}/pi_d_max", m.pi_d_max)
    # The design run itself: both shaft-closure asserts fire here.
    ref = m.ref
    for st in ("2", "25", "3", "4", "45", "5", "9"):
        put(f"design/{gname}/s{st}/Tt", ref.stations[st].Tt)
        put(f"design/{gname}/s{st}/pt", ref.stations[st].pt)
    put(f"design/{gname}/F_over_mdot", ref.performance.specific_thrust)
    put(f"design/{gname}/tsfc", ref.performance.tsfc)

    mm = Count39(d, FLIGHT, 1.0, map_lp=ComponentMap(), map_hp=ComponentMap())
    M39[gname] = mm
    put(f"faces/{gname}/mcorr_lp_d", mm.mcorr_lp_d)
    put(f"faces/{gname}/mcorr_hp_d", mm.mcorr_hp_d)
    put(f"faces/{gname}/tau_lpc_d", mm.tau_lpc_d)
    put(f"faces/{gname}/tau_hpc_d", mm.tau_hpc_d)
    put(f"faces/{gname}/Tt2_d", mm.Tt2_d)
    put(f"faces/{gname}/Tt25_d", mm.Tt25_d)
    put(f"faces/{gname}/Tt4_d", mm.Tt4_d)
    put(f"faces/{gname}/Tt45_d", mm.Tt45_d)

print(f"[1] hardware captured, {time.time() - T0:.1f}s")

# ==============================================================================================
# 2. THE RUNG-38 GRID — 147 cells, and for every abort, WHY
# ==============================================================================================
CENSUS38 = dict((k, 0) for k in ABORT.values())
n38_ok = 0
for gname, _ in GASES:
    d = DESIGNS[gname]
    for M0 in M0S:
        flight = FlightCondition(T0=250.0, p0=FLIGHT.p0, M0=M0)
        for Tt4 in TT4S:
            tag = f"{gname}/{M0:.2f}/{Tt4:.0f}"
            m = Count38(d, FLIGHT, 1.0).reset()
            try:
                od = m.match(flight, Tt4)
            except AssertionError as e:
                code = abort_code(str(e).split("\n")[0])
                put(f"r38/{tag}/abort", code)
                CENSUS38[code] += 1
                continue
            put(f"r38/{tag}/abort", ABORT[""])
            CENSUS38[0.0] += 1
            n38_ok += 1
            # THE PASS COUNT — bit-gated against PyPy only (§ 5.7 (c)).
            put(f"r38/{tag}/n_pass", float(m.n_pass))
            for name, v in (("pi_lpc", od.pi_lpc), ("pi_hpc", od.pi_hpc),
                            ("tau_lpc", od.tau_lpc), ("tau_hpc", od.tau_hpc),
                            ("tau_hpt", od.tau_hpt), ("pi_hpt", od.pi_hpt),
                            ("tau_lpt", od.tau_lpt), ("pi_lpt", od.pi_lpt),
                            ("mdot_air", od.mdot_air), ("mdot_ratio", od.mdot_ratio),
                            ("thrust", od.thrust), ("V0", od.V0), ("V9", od.V9),
                            ("M9", od.M9), ("T9", od.T9), ("p9", od.p9),
                            ("F_over_mdot", od.performance.specific_thrust),
                            ("tsfc", od.performance.tsfc),
                            ("eta_th", od.performance.eta_thermal),
                            ("eta_p", od.performance.eta_propulsive)):
                put(f"r38/{tag}/{name}", v)
            for st in ("2", "25", "3", "4", "45", "5", "9"):
                s = od.stations[st]
                put(f"r38/{tag}/s{st}/Tt", s.Tt)
                put(f"r38/{tag}/s{st}/pt", s.pt)
            put(f"r38/{tag}/s4/far", od.stations["4"].far)

for code in sorted(set(ABORT.values())):
    put(f"census/r38/abort_code/{code:.0f}", float(CENSUS38[code]))
print(f"[2] rung 38: {n38_ok} matched of {len(GASES) * len(M0S) * len(TT4S)}, "
      f"{time.time() - T0:.1f}s")

# ==============================================================================================
# 3. THE RUNG-39 GRID — the same cells, per map shape
# ==============================================================================================
CENSUS39 = dict((k, 0) for k in ABORT.values())
n39_ok = 0
TURB_PASSES, HP_PASSES, LP_PASSES, CLAMPS = set(), set(), set(), 0
for gname, _ in GASES:
    d = DESIGNS[gname]
    for sname, mlp, mhp, wide in SHAPES:
        for M0 in (M0S if wide else M0S_NARROW):
            flight = FlightCondition(T0=250.0, p0=FLIGHT.p0, M0=M0)
            for Tt4 in TT4S:
                tag = f"{gname}/{sname}/{M0:.2f}/{Tt4:.0f}"
                mm = Count39(d, FLIGHT, 1.0, map_lp=mlp, map_hp=mhp).reset()
                try:
                    od = mm.match(flight, Tt4)
                except AssertionError as e:
                    code = abort_code(str(e).split("\n")[0])
                    put(f"r39/{tag}/abort", code)
                    CENSUS39[code] += 1
                    continue
                put(f"r39/{tag}/abort", ABORT[""])
                CENSUS39[0.0] += 1
                n39_ok += 1
                TURB_PASSES.update(mm.n_turb)
                HP_PASSES.update(mm.hp_passes)
                LP_PASSES.update(mm.lp_passes)
                CLAMPS += mm.n_clamp
                put(f"r39/{tag}/n_pass", float(mm.n_pass))
                for name, v in (("pi_lpc", od.pi_lpc), ("pi_hpc", od.pi_hpc),
                                ("eta_lpc", od.eta_lpc), ("eta_hpc", od.eta_hpc),
                                ("eta_hpt", od.eta_hpt), ("eta_lpt", od.eta_lpt),
                                ("n_lp", od.n_lp), ("n_hp", od.n_hp),
                                ("N_lp_ratio", od.N_lp_ratio), ("N_hp_ratio", od.N_hp_ratio),
                                ("slip", od.slip), ("phi_lp", od.phi_lp),
                                ("phi_hp", od.phi_hp), ("nu_hpt", od.nu_hpt),
                                ("nu_lpt", od.nu_lpt), ("tau_hpt", od.tau_hpt),
                                ("tau_lpt", od.tau_lpt), ("mdot_air", od.mdot_air),
                                ("thrust", od.thrust), ("V9", od.V9), ("T9", od.T9),
                                ("p9", od.p9),
                                ("F_over_mdot", od.performance.specific_thrust),
                                ("tsfc", od.performance.tsfc)):
                    put(f"r39/{tag}/{name}", v)
                for st in ("25", "3", "4", "45", "5"):
                    s = od.stations[st]
                    put(f"r39/{tag}/s{st}/Tt", s.Tt)
                    put(f"r39/{tag}/s{st}/pt", s.pt)

for code in sorted(set(ABORT.values())):
    put(f"census/r39/abort_code/{code:.0f}", float(CENSUS39[code]))
put("census/r39/turb_passes_min", float(min(TURB_PASSES)))
put("census/r39/turb_passes_max", float(max(TURB_PASSES)))
# The MAXIMA witness that the ETA_MAX = 80 cap is nowhere near approached. The MINIMA are
# what witness the CHECK-FIRST loop shape rung 39's flat-map reduce depends on: a flat map
# passes the residual on entry, so the secant is called ZERO times. A `do`-while would move
# the minimum to 1 and leave the maximum at 4 — so the maxima alone are BLIND to it.
put("census/r39/hp_passes_max", float(max(HP_PASSES)))
put("census/r39/lp_passes_max", float(max(LP_PASSES)))
put("census/r39/hp_passes_min", float(min(HP_PASSES)))
put("census/r39/lp_passes_min", float(min(LP_PASSES)))
# THE DEAD CLAMP, dumped as an explicit zero rather than left as an absence (§ 5.7 (g)).
put("census/r39/secant_clamp_hits", float(CLAMPS))
print(f"[3] rung 39: {n39_ok} matched, turb passes {sorted(TURB_PASSES)}, "
      f"hp {sorted(HP_PASSES)}, lp {sorted(LP_PASSES)}, clamps {CLAMPS}, "
      f"{time.time() - T0:.1f}s")

# ==============================================================================================
# 4. THE (★) BISECTION's COST (P3) — measured per SOLVE, on a stated grid
#
#    ceil(log2(0.979 / 1e-13)) = 44 iterations, + the two bracket endpoints = 46 residual
#    evaluations, + one more `tau_of` after the loop = 47, which is what the instrument reads.
#    § 5.6's P2 was corrected for exactly this reason one slice ago, so the gate names its noun.
# ==============================================================================================
TAU_PER_SOLVE = set()
n_solves = 0
for gname, _ in GASES:
    d = DESIGNS[gname]
    for M0 in (0.85, 1.6):
        flight = FlightCondition(T0=250.0, p0=FLIGHT.p0, M0=M0)
        for Tt4 in (900.0, 1100.0, 1500.0):
            m = Count38(d, FLIGHT, 1.0).reset()
            try:
                m.match(flight, Tt4)
            except AssertionError:
                continue
            TAU_PER_SOLVE.update(m.tau_per_solve)
            n_solves += len(m.tau_per_solve)
assert len(TAU_PER_SOLVE) == 1, f"the (star) bisection cost SPREAD: {sorted(TAU_PER_SOLVE)}"
put("bisect/tau_of_calls_per_solve", float(next(iter(TAU_PER_SOLVE))))
put("bisect/n_solves_swept", float(n_solves))
print(f"[4] (star) bisection: {sorted(TAU_PER_SOLVE)} tau_of calls per solve "
      f"over {n_solves} solves")

# ==============================================================================================
# 5. THE REDUCE LADDER — one dispatch closes four rungs
#
#    flat + lp_disabled -> rung 31, shaped + lp_disabled -> rung 32. Both are EXACT DISPATCH
#    (no LP hardware is built), so the values below must equal the single-spool matchers' own.
# ==============================================================================================
single = build_turbojet(Gas.reacting_equilibrium(), PI_HPC, TT4, FLIGHT.p0,
                        pi_d=REAL["pi_d"], eta_c=REAL["eta_hpc"], eta_b=REAL["eta_b"],
                        pi_b=REAL["pi_b"], eta_t=REAL["eta_hpt"], eta_m=REAL["eta_m"],
                        pi_n=REAL["pi_n"], nozzle_convergent=True)
shape = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
for Tt4 in (900.0, 1100.0, 1500.0):
    tag = f"{Tt4:.0f}"
    a = TwoSpoolMatcher(single, FLIGHT, 1.0, lp_disabled=True).match(FLIGHT, Tt4)
    b = OffDesignMatcher(single, FLIGHT, 1.0).match(FLIGHT, Tt4)
    put(f"reduce/r38_disabled/{tag}/pi_c", a.pi_c)
    put(f"reduce/r31/{tag}/pi_c", b.pi_c)
    put(f"reduce/r38_disabled/{tag}/thrust", a.thrust)
    put(f"reduce/r31/{tag}/thrust", b.thrust)
    c = TwoSpoolMapMatcher(single, FLIGHT, 1.0, map_hp=shape,
                           lp_disabled=True).match(FLIGHT, Tt4)
    e = MapMatcher(single, FLIGHT, 1.0, comp_map=shape).match(FLIGHT, Tt4)
    put(f"reduce/r39_disabled/{tag}/pi_c", c.pi_c)
    put(f"reduce/r32/{tag}/pi_c", e.pi_c)
    put(f"reduce/r39_disabled/{tag}/eta_c", c.eta_c)
    put(f"reduce/r32/{tag}/eta_c", e.eta_c)
    put(f"reduce/r39_disabled/{tag}/n_corr", c.n_corr)
    put(f"reduce/r32/{tag}/n_corr", e.n_corr)

# ==============================================================================================
# 6. THE ISOLATED CASCADE (rung 38 gate 3's protocol) — a fixed (Tt2, Tt4, f), so the outer
#    joint loop cannot confound a reading, and BOTH cascades side by side.
# ==============================================================================================
from turbojet.engine import ram_recovery  # noqa: E402

for gname, _ in GASES:
    d = DESIGNS[gname]
    m = TwoSpoolMatcher(d, FLIGHT, 1.0)
    mm = TwoSpoolMapMatcher(d, FLIGHT, 1.0,
                            map_lp=ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
                            map_hp=ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0))
    for Tt4 in (900.0, 1200.0, 1500.0):
        od = m.match(FLIGHT, Tt4)
        state0, _ = m._fs_engine.freestream(FLIGHT, m.mdot_air_design)
        Tt2 = state0.Tt
        pt2 = m.pi_d_max * ram_recovery(FLIGHT.M0) * state0.pt
        f = od.stations["4"].far
        pt4 = m.pi_b * od.pi_hpc * od.pi_lpc * pt2
        wgas = m._working_gas(f, Tt4, pt4)
        tag = f"{gname}/{Tt4:.0f}"
        put(f"iso/{tag}/Tt2", Tt2)
        put(f"iso/{tag}/pt2", pt2)
        put(f"iso/{tag}/f", f)
        put(f"iso/{tag}/pt4", pt4)
        c = m._cascade(wgas, Tt2, Tt4, f)
        for k in sorted(c):
            put(f"iso/{tag}/r38/{k}", c[k])
        wgas2 = mm._working_gas(f, Tt4, pt4)
        cm = mm._cascade_map(wgas2, Tt2, pt2, Tt4, f)
        for k in sorted(cm):
            put(f"iso/{tag}/r39/{k}", cm[k])

# ==============================================================================================
# 7. STANDALONE `psi` / `solve_n` WITH `l` — the term slice K put on the Rust map (§ 5.7 (a))
#
#    Cheap (no cycle solve), and it pins the ONE thing the value oracle above cannot see: the
#    l-term's ARITHMETIC, on the very coefficients rung 39's own shapes use.
# ==============================================================================================
for i, cm in enumerate((ComponentMap(sigma=0.1, l=0.7), ComponentMap(sigma=0.1, l=1.0),
                        ComponentMap(sigma=0.2, l=0.85), ComponentMap(sigma=0.3),
                        ComponentMap())):
    for j, phi in enumerate((0.55, 0.7, 0.85, 0.95, 1.0, 1.05, 1.2, 1.45)):
        put(f"psi/{i}/{j}", cm.psi(phi))
    for j, (mm_, tau, tau_d) in enumerate(((0.8, 1.9, 2.0), (1.0, 2.0, 2.0),
                                           (1.1, 2.15, 2.0), (0.6, 1.5, 2.0))):
        put(f"solve_n/{i}/{j}", cm.solve_n(mm_, tau, tau_d))

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-5K two-spool matcher oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
