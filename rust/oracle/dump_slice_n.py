"""SLICE N's value oracle — rungs 55 + 56, on the grid § 5.10 PRE-REGISTERED and no other.

    fast:  2 gases (cpg, tpg) x 5 disclosed shapes x K{2,4,8,16} x split{dT,tau}
           x cap_profile{derived,uniform} x 4 throttles          = 640 cells, x2 spools
    sched: 2 gases x 5 shapes x 2 spools x vsv_stages{None,1} x 4 throttles = 160 rows
    equil: the eq gas on `stage_throat_margin` ONLY (§ 5.10 (v)), its own process/file

THE GRID IS THE PRE-REGISTERED ONE, NOT A NEIGHBOURING ONE (§ 5.7 (e)). Every census in
§ 5.10 (i)/(ii)/(iii)/(iv)/(vi) was measured on one of these three sweeps, so a dump that swept
a slightly different one would leave each of those bars asserting over unmeasured cells.

`CAP = 0.60` IS TAKEN FROM `probe_n3.py`, AND ITS STATED PROVENANCE IS WRONG. probe_n1's comment
calls it *"rung 54's disclosed capacity constant, as the rung-56 tests carry it"*; the rung-56
tests carry `CAP = 0.90` (`tests/test_rung56.py:48`) and 0.60 appears nowhere in rungs 53-56 as a
capacity. It is kept anyway — the (iv) census was MEASURED at 0.60, so changing it here would
silently re-point every one of those bars at cells nobody has looked at. The constant is
arbitrary-but-pre-registered, which is a different justification from the one the probe wrote
down, and § 5.10 records the correction.

THE CENSUS IS EMITTED AS KEYS, NOT QUOTED AS PROSE. § 5.10 (iii)'s `3 204 / 521 649` and (vi)'s
`120 / 4 360` were measured on the PROBES' grids (probe_n1 sweeps K in {2,4,8} -- THREE values --
and no `cap_profile` axis, i.e. 240 cells and 120 stacks, not 640 and 320). Restating them here
would be slice L step 4's copied bar. So Python COUNTS on this grid and the Rust compares its own
`take_census()` against those counts.

    `clamped` IS A SUM PYTHON CANNOT SPLIT. `StageStack.march` adds both floors into one counter,
    which is § 5.10 (iii)'s own point, so the dump emits the SUM and the Rust asserts
    `t_floor + p_floor == sum` beside `p_floor == 0`. The split is the port's, gated at step 2
    from the derived threshold `e > 1.001`; it is not something this file can witness.

Usage:
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_n.py fast  rust/oracle/slice_n_pypy.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_slice_n.py equil rust/oracle/slice_n_eq_pypy.tsv
    py -3                     rust/oracle/dump_slice_n.py lean  rust/oracle/slice_n_cpython.tsv
"""
import os
import sys
import traceback

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet.engine import (  # noqa: E402
    ComponentMap, FlightCondition, StageStack, StageStackMatcher, build_two_spool_turbojet,
)
from turbojet.gas import Gas  # noqa: E402

ARM = sys.argv[1] if len(sys.argv) > 1 else "fast"
OUT = sys.argv[2] if len(sys.argv) > 2 else None

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
CAP = 0.60          # see the module note -- pre-registered, NOT the rung-56 suite's 0.90

# Rung 53's OWN five disclosed shapes, verbatim from tests/test_rung53.py::SHAPES, which is what
# probe_n1/n3 swept. The names use '_' because '/' is this file's key separator.
SHAPES = [
    ("flow_press", ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7),
                   ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)),
    ("press_flow", ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    ("tilted",     ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85),
                   ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)),
    ("steep",      ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2),
                   ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)),
    ("flat_eta",   ComponentMap(sigma=0.1, l=0.7), ComponentMap(sigma=0.1, l=1.0)),
]
SHAPE_IX = {name: i for i, (name, _, _) in enumerate(SHAPES)}
SPOOLS = ("lp", "hp")
K_GRID = (2, 4, 8, 16)
SPLITS = ("dT", "tau")
PROFILES = ("derived", "uniform")
THROTTLE_FAST = (1500.0, 1200.0, 1000.0, 800.0)

# The equilibrium arm — `stage_throat_margin` ONLY, per § 5.10 (v): it contains no scan, so it
# stays at 0.1-2.4 s per cell, while ONE `stage_incidence_schedule` row costs 36.9 s. The
# schedule's equilibrium coverage is DELIBERATELY ABSENT with that number beside it; a two-cell
# schedule arm would be a bar over unmeasured cells (§ 5.8.1 (i)).
K_EQ = (2, 8)
THROTTLE_EQ = (1500.0, 1200.0)


def cpg_gas():
    gc, cc, gt, ct = 1.4, 1004.0, 1.3, 1239.0
    return Gas(gamma_c=gc, cp_c=cc, R_c=(gc - 1.0) / gc * cc,
               gamma_t=gt, cp_t=ct, R_t=(gt - 1.0) / gt * ct, hPR=42.8e6)


GASES = {"cpg": cpg_gas, "tpg": Gas.thermally_perfect, "eq": Gas.reacting_equilibrium}
ARMS = {"fast": ["cpg", "tpg"], "lean": ["cpg", "tpg"], "equil": ["eq"]}[ARM]

# THE `lean` ARM IS `fast` WITHOUT THE PER-ROW KEYS, AND IT IS WHAT THE CPYTHON ARM RUNS.
#
# Per-row keys are 30 960 of `fast`'s 72 360 and 2.1 MB of its 4.84 — 43 % of the file. They are
# there for COVERAGE against PyPy: an aggregate is an argmin or a face read, so a stack whose
# row 3 is wrong can still produce the right `binds`. The CPython arm answers a different
# question — how much of the dump is interpreter-STABLE, i.e. how strong the bit claim against
# PyPy is — and an argmin over all K rows already moves when a row drifts. So the interpreter arm
# sweeps the SAME 640 cells and 160 schedule rows and omits the projections, and its bar is over
# exactly what it dumped. The omission is stated with its size rather than left to be noticed.
ROWS = ARM != "lean"

out = []


def put(k, v):
    out.append((k, float(v)))


def flag(k, b):
    out.append((k, 1.0 if b else 0.0))


# =========================================================================================
# THE CENSUS — pure delegating wrappers, so the counts cost arithmetic nothing
# =========================================================================================
#
# Every wrapper below returns the original method's value unchanged; none of them reads or
# writes a float. That is what makes it legitimate to count DURING the value sweep rather than
# in a second pass — a second pass would have to re-run the grid and could drift from it.

STATS = {}
FIRE = {}
FIRST_CLAMPED = {}
FIRST_CLAMPED_WHERE = {}
CURRENT = {}          # the cell the sweep is inside, so a firing can be ATTRIBUTED
_orig = {}


def _bump(k, n=1):
    STATS[k] = STATS.get(k, 0) + n


def _fire(k):
    """Firings are tallied PER ARM. The fast arm has no caught scope at all, so a non-zero
    `fire/fast/*` would mean a raise that the sweep only survived by accident — which is a
    different fact from the schedule arm's, and a single global counter would hide it."""
    k = "%s/%s" % (ARM_TAG[0], k)
    FIRE[k] = FIRE.get(k, 0) + 1


ARM_TAG = ["fast"]
FIRE_KEYS = ("bracket", "clamped_root", "other", "map_bracket")


def instrument():
    _orig["post"] = StageStack.__post_init__
    _orig["march"] = StageStack.march
    _orig["solve_n"] = StageStack.solve_n
    _orig["caps"] = StageStack.capacities
    _orig["map_solve_n"] = ComponentMap.solve_n

    def post(self):
        _bump("stacks_built")
        return _orig["post"](self)

    def march(self, m, n, eta_live):
        r = _orig["march"](self, m, n, eta_live)
        _bump("marches")
        _bump("clamped_total", r["clamped"])
        if r["clamped"]:
            _bump("marches_clamped")
        return r

    def solve_n(self, m, tau_c, eta_live):
        _bump("solve_n_calls")
        if self.K == 1:
            _bump("solve_n_k1")
        try:
            return _orig["solve_n"](self, m, tau_c, eta_live)
        except AssertionError as e:
            # § 5.10 (i)'s frame census, taken at the RAISE rather than inferred from the
            # message: `extract_tb(...)[-1]` is the innermost frame, which is the whole point.
            tb = traceback.extract_tb(e.__traceback__)[-1]
            txt = str(e)
            if "bracket fails" in txt:
                _fire("bracket")
            elif "clamped" in txt:
                _fire("clamped_root")
                # The triple the Rust gate needs to reach this arm DIRECTLY. The schedule's
                # `except AssertionError: break` swallows WHICH arm fired, so reproducing the
                # firings does not gate the arm — only re-entering `try_solve_n` at these
                # arguments does (§ 5.10's `slice_n_deferrals_so_far` item 2b).
                FIRST_CLAMPED.setdefault("m", m)
                FIRST_CLAMPED.setdefault("tau_c", tau_c)
                FIRST_CLAMPED.setdefault("eta_live", eta_live)
                FIRST_CLAMPED.setdefault("K", float(self.K))
                FIRST_CLAMPED.setdefault("vsv", self.cmap.vsv)
                FIRST_CLAMPED.setdefault("tau_d", self.tau_d)
                FIRST_CLAMPED.setdefault("pi_d", self.pi_d)
                FIRST_CLAMPED.setdefault("eta_d", self.eta_d)
                FIRST_CLAMPED.setdefault("e_d", self.e_d)
                if not FIRST_CLAMPED_WHERE:
                    FIRST_CLAMPED_WHERE.update(CURRENT)
            else:
                _fire("other")
            _fire("frame:%s:%d" % (tb.name, tb.lineno))
            raise

    def caps(self):
        _bump("capacities_hits" if self._C_ks is not None else "capacities_built")
        return _orig["caps"](self)

    def map_solve_n(self, m, tau_c, tau_d):
        _bump("map_solve_n_calls")
        try:
            return _orig["map_solve_n"](self, m, tau_c, tau_d)
        except AssertionError:
            _fire("map_bracket")
            raise

    StageStack.__post_init__ = post
    StageStack.march = march
    StageStack.solve_n = solve_n
    StageStack.capacities = caps
    ComponentMap.solve_n = map_solve_n


CENSUS_KEYS = ("stacks_built", "marches", "marches_clamped", "clamped_total", "solve_n_calls",
               "solve_n_k1", "map_solve_n_calls", "capacities_built", "capacities_hits")


def emit_census(tag):
    for k in CENSUS_KEYS:
        put("census/%s/%s" % (tag, k), STATS.get(k, 0))
    STATS.clear()


# =========================================================================================
# BUILDERS
# =========================================================================================

def design(gname):
    return build_two_spool_turbojet(GASES[gname](), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def maps(ml, mh, capacity):
    a_l, a_h = ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR)
    if capacity:
        a_l, a_h = a_l.with_capacity(CAP), a_h.with_capacity(CAP)
    return a_l, a_h


# The per-spool aggregate field sets, named once so the Rust reader can be a transcription.
THR_KEYS = ("vsv", "m", "n", "capacity_front", "tan_b1_crit", "binds", "m_c_worst", "x_worst",
            "c_min_worst", "m_c_face", "x_face", "amplification", "inc_worst", "m_i_worst")
THR_FLAGS = ("chokes", "rear_binds", "front_binds")
MAR_KEYS = ("vsv", "phi_face", "n", "m", "tan_b1_crit", "worst", "m_i_worst", "m_i_face",
            "rear_excess", "phi_front", "phi_rear")
GAP_KEYS = ("m", "n", "tau_lumped", "tau_marched", "gap", "gap_frac")
WALK_KEYS = ("binds", "m_c_worst", "m_c_face", "amplification", "inc_worst", "m_i_worst",
             "c_min_worst", "m", "n", "vsv")
SHIFT_KEYS = ("n_lumped", "n_stacked", "d_n", "phi_lumped", "phi_stacked", "d_phi",
              "pi_lumped", "pi_stacked", "d_pi")
SCHED_KEYS = ("vsv_star", "residual", "tan_b1", "tan_b1_design", "phi_stage", "phi_stage_bare",
              "m_i", "m_i_bare", "m_i_worst", "worst", "n", "n_bare", "d_n", "rear_excess")
# The WIDE per-row set, on a NAMED subgrid (K = 8 at the design throttle, BOTH profiles) rather
# than everywhere: `capacity`/`area` are throttle-INDEPENDENT, so sweeping them over all four
# throttles would multiply the file by 4 for nothing. Both profiles are kept because `capacity`
# is the ONE reading the profile axis moves — the uniform branch would otherwise be dumped
# nowhere.
WIDE_KEYS = ("phi", "n", "vsv", "m_k", "capacity", "area", "throat_loading", "c_min")


def sweep_fast(gases, ks, splits, profiles, throttles, subgrids, rows=True):
    """The throat/margin sweep. `subgrids` switches the two readers that are NOT swept whole;
    `rows` switches the PER-ROW keys, which the CPython arm does not carry (see `lean` below)."""
    n_cell = 0
    for gname in gases:
        d = design(gname)
        for sname, ml0, mh0 in SHAPES:
            a_l, a_h = maps(ml0, mh0, True)
            for K in ks:
                for split in splits:
                    for prof in profiles:
                        cfg = "%s/%s/K%d/%s/%s" % (gname, sname, K, split, prof)
                        m = StageStackMatcher(d, FLIGHT, 1.0, map_lp=a_l, map_hp=a_h,
                                              K_lp=K, K_hp=K, split=split, cap_profile=prof)
                        for tt4 in throttles:
                            n_cell += 1
                            cell = "%s/%.0f" % (cfg, tt4)
                            print("[cell %d] %s" % (n_cell, cell), file=sys.stderr, flush=True)
                            r = m.stage_throat_margin(FLIGHT, tt4)
                            for spool in SPOOLS:
                                s = r[spool]
                                p = "thr/%s/%s" % (cell, spool)
                                for k in THR_KEYS:
                                    put("%s/%s" % (p, k), s[k])
                                for k in THR_FLAGS:
                                    flag("%s/%s" % (p, k), s[k])
                                put("%s/n_rows" % p, len(s["stages"]))
                                # BOTH argmin currencies on EVERY row — the census reads only
                                # the two indices, and a tie-break flip is invisible in an
                                # aggregate that agrees to the bit.
                                for st in (s["stages"] if rows else ()):
                                    q = "%s/r%d" % (p, st["stage"])
                                    put("%s/m_c" % q, st["m_c"])
                                    put("%s/m_i" % q, st["m_i"])
                                if rows and K == 8 and tt4 == 1500.0:
                                    for st in s["stages"]:
                                        q = "%s/r%d" % (p, st["stage"])
                                        for k in WIDE_KEYS:
                                            put("%s/%s" % (q, k), st[k])
                                        flag("%s/chokes" % q, st["chokes"])

                            # The profile axis is INERT for every rung-55 reading: `capacities`
                            # is touched only by rung 56's capacity currency, so `stage_margin`,
                            # `work_gap` and the matched point itself are bit-identical across
                            # it. Dumped on `derived` only and the invariance GATED in Rust —
                            # dumping the duplicate would double the file to restate it.
                            if prof != "derived":
                                continue
                            a = m.stage_margin(FLIGHT, tt4)
                            for spool in SPOOLS:
                                s = a[spool]
                                p = "mar/%s/%s" % (cell, spool)
                                for k in MAR_KEYS:
                                    put("%s/%s" % (p, k), s[k])
                                for st in (s["stages"] if rows else ()):
                                    put("%s/r%d/m_phi" % (p, st["stage"]), st["m_phi"])
                                    if tt4 == 1500.0:
                                        put("%s/r%d/phi_surge" % (p, st["stage"]),
                                            st["phi_surge"])
                            w = m.work_gap(FLIGHT, tt4)
                            for spool in SPOOLS:
                                for k in GAP_KEYS:
                                    put("gap/%s/%s/%s" % (cell, spool, k), w[spool][k])

                        if not (subgrids and K == 8 and prof == "derived"):
                            continue
                        # `throat_walk` is a PROJECTION of the rows above onto one spool, so
                        # what it gates is the row ASSEMBLY, not new arithmetic. Its subgrid is
                        # named rather than implied: K = 8, derived, both spools, all shapes.
                        for spool in SPOOLS:
                            # NOT `rows` — that is this function's own PARAMETER, and binding a
                            # walk list to it silently re-armed the per-row keys after the first
                            # subgrid cell. The `lean` arm came out at 71 504 keys against
                            # `fast`'s 72 360, which reads as "about the same"; only a key-SET
                            # diff showed the 856 missing were all in the FIRST shape.
                            walk_rows = m.throat_walk(FLIGHT, throttles, spool=spool)
                            for i, row in enumerate(walk_rows):
                                # KEYED BY THROTTLE, WITH THE ROW INDEX AS A VALUE. The index
                                # alone would hide the throttle from every reader of the key,
                                # and the CPython arm needs it: the argmin is a last-bit
                                # tie-break AT DESIGN and physics everywhere else. Dumping
                                # `index` keeps the grid ORDER gated, which keying by throttle
                                # would otherwise drop.
                                p = "walk/%s/%s/%.0f" % (cfg, spool, row["Tt4"])
                                put("%s/index" % p, i)
                                for k in WALK_KEYS:
                                    put("%s/%s" % (p, k), row[k])
                                flag("%s/chokes" % p, row["chokes"])
                                put("%s/n_caps" % p, len(row["capacities"]))
                                for k in ("capacities", "throat_loadings", "margins"):
                                    put("%s/%s_first" % (p, k), row[k][0])
                                    put("%s/%s_last" % (p, k), row[k][-1])
                        # `running_line_shift` re-matches a K = 1 sibling per throttle, so it is
                        # the one reader whose cost doubles the cell it sits on. Same named
                        # subgrid.
                        for i, row in enumerate(m.running_line_shift(FLIGHT, throttles)):
                            p = "shift/%s/%d" % (cfg, i)
                            for spool in SPOOLS:
                                for k in SHIFT_KEYS:
                                    put("%s/%s/%s" % (p, spool, k), row[spool][k])
                            for k in ("thrust_lumped", "thrust_stacked", "d_thrust"):
                                put("%s/%s" % (p, k), row[k])
    return n_cell


def sweep_schedule(gases, throttles):
    """§ 5.10 (i)/(ii)'s arm — the ONE caught scope in rungs 55/56, and the 120/160 split.

    CAPACITY-FREE MAPS ON ALL 160 ROWS, which is `probe_n3.probe_scan_cells` verbatim. § 5.10's
    laziness note says *"80 of the 160 schedule rows are built with capacity = False"*; measured,
    it is 160 of 160 — the eager-build hazard that note describes is therefore wider than the
    note claimed, not narrower, and the conclusion stands a fortiori.
    """
    n_row = 0
    for gname in gases:
        d = design(gname)
        for sname, ml0, mh0 in SHAPES:
            a_l, a_h = maps(ml0, mh0, False)
            for spool in SPOOLS:
                for vs in (None, 1):
                    kw = {"vsv_stages_lp" if spool == "lp" else "vsv_stages_hp": vs}
                    m = StageStackMatcher(d, FLIGHT, 1.0, map_lp=a_l, map_hp=a_h,
                                          K_lp=8, K_hp=8, **kw)
                    tag = "%s/%s/%s/vs%s" % (gname, sname, spool, "N" if vs is None else vs)
                    print("[sched] %s" % tag, file=sys.stderr, flush=True)
                    CURRENT.clear()
                    CURRENT.update(gas=gases.index(gname), shape=SHAPE_IX[sname],
                                   spool=SPOOLS.index(spool),
                                   vs=-1.0 if vs is None else float(vs))
                    rows = m.stage_incidence_schedule(FLIGHT, throttles, spool=spool)
                    for row in rows:
                        n_row += 1
                        p = "sched/%s/%.0f" % (tag, row["Tt4"])
                        flag("%s/reached" % p, row["reached"])
                        put("%s/K" % p, row["K"])
                        flag("%s/has_vsv_stages" % p, row["vsv_stages"] is not None)
                        put("%s/vsv_stages" % p, -1.0 if row["vsv_stages"] is None
                            else row["vsv_stages"])
                        for k in SCHED_KEYS:
                            put("%s/%s" % (p, k), row[k])
    return n_row


# =========================================================================================
# THE RUN
# =========================================================================================

instrument()

if ARM == "equil":
    # `stage_throat_margin` only, on the derived profile — § 5.10 (v)'s sized arm.
    ARM_TAG[0] = "equil"
    cells = sweep_fast(ARMS, K_EQ, SPLITS, ("derived",), THROTTLE_EQ, subgrids=False)
    emit_census("equil")
    print("# equil: %d cells" % cells, file=sys.stderr, flush=True)
else:
    cells = sweep_fast(ARMS, K_GRID, SPLITS, PROFILES, THROTTLE_FAST, subgrids=True, rows=ROWS)
    emit_census("fast")
    ARM_TAG[0] = "sched"
    rows = sweep_schedule(ARMS, THROTTLE_FAST)
    emit_census("sched")
    # § 5.10 (i)'s frame census, and the deferral-2b triple beside it.
    for tag in ("fast", "sched"):
        for k in FIRE_KEYS:
            put("fire/%s/%s" % (tag, k), FIRE.get("%s/%s" % (tag, k), 0))
    put("fire/has_clamped_sample", 1.0 if FIRST_CLAMPED else 0.0)
    for k, v in sorted(FIRST_CLAMPED.items()):
        put("fire/clamped/%s" % k, v)
    for k, v in sorted(FIRST_CLAMPED_WHERE.items()):
        put("fire/clamped/at_%s" % k, v)
    print("# fast: %d cells, %d schedule rows" % (cells, rows), file=sys.stderr, flush=True)
    print("# frames: %s" % sorted(FIRE.items()), file=sys.stderr, flush=True)

if OUT:
    with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
        for k, v in out:
            fh.write("%s\t%s\n" % (k, v.hex()))
else:
    for k, v in out:
        print("%s\t%s" % (k, v.hex()))
print("# %d keys" % len(out), file=sys.stderr, flush=True)
