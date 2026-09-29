"""SLICE AI step 6 — **THE ORACLE for rungs 79 AND 80**, run once on PyPy and once on CPython.

Emits `key<TAB>u64` to a file, sorted, with the tally on stderr. Every float is its IEEE-754 bit
pattern, every string an FNV-1a 64-bit hash, every `None` a PRESENCE FLAG beside the value it
stands in for, and every sample-shaped reading carries its ROW COUNT.

    pypy   rust/oracle/dump_slice_ai.py rust/oracle/slice_ai_pypy.tsv
    python rust/oracle/dump_slice_ai.py rust/oracle/slice_ai_cpython.tsv

Slice AH's dumper is the template (`dump_slice_ah.py`): the key set is WALKED here and
HAND-LISTED on the Rust side, and `slice_ai_oracle.rs` compares the two key SETS before any value,
so a key both emitters forgot is a named failure rather than a shared silence. What is different
at this slice is below.

# EVERY READER ON BOTH STATOR ARMS — AND RUNG 80's INCIDENCE ARM HAS NEVER RUN ANYWHERE

`tests/test_rung79.py` and `tests/test_rung80.py` build every receiver with the `phi` stator
(`stator_lim`) and call every reader at `inc = False`. § 5.31.5 (k): a grid NARROWED from the
suite's is the same defect as one COPIED from it. So every reader here runs on both arms, and no
Python test had built a rung-80 machine with an incidence stator before this file.

**WHAT THE INCIDENCE ARM DOES NOT REACH, MEASURED:** `_walls_of`'s incidence branch (the wall read
back through `phi_lim_at`). Every rig arms the VALVE and `phi_air` takes the valve's wall first, so
the stator's wall enters no reported field. Injected as `999.0` in the port, it moved 0 of this
file's keys (plan § 5.33.6). The arm is still a different plant — its rung-79 scans find 2 riding
points where the `phi` arm finds 10, and its rung-80 gain cells are mostly vacuous.

# SECTION I — § 5.33 (i)'s ARM: RUNG 74's `demand_gains` ON RUNG-78/79/80 MACHINES

`demand_gains` is single-definer at rung 74 and dispatches `_with_coord`, which from rung 79 on
lands on rung 79's body: it writes `_phi_ref = "demand"`, a value the class never declares, and
the incidence branch then runs on every `_cap_fuel` call inside the scope — and short-circuits to
`_surge_fuel` every time. The pre-flight measured it value-INVISIBLE (0 of 196 keys move against
rung 78) and counter-VISIBLE (128 hits at `phi_lim = 0.80`, 80 at `0.76`, `fb_inc = calls_inc`).
`demand_gains` does not RETURN the counters, so each call is bracketed by a snapshot of rung 79's
six class counters and the DELTAS are emitted beside the reading. The Rust side resets its
thread-locals and reads them after, which is the same delta. Settings are the pre-flight's own
(`tests/test_numeric_fingerprint.py`'s `_S3_*`: `ds = 0.005`, `every = 4`), so its measurements
are this section's predictions. Rung 78 is the machine the break is introduced against; rung 80
inherits it.

# SECTION P — THE PLANTS, WITH THEIR COUNTERS BESIDE THEM

`coord_march` returns aggregates of its two marches and none of their points, and rung 80's
readers return one row per march. So P marches rung 79's INCIDENCE plant (`coord_march`'s own
`traj1`: `_cap_march` under `_with_coord("incidence")`, accel armed from the `phi` scope) and rung
80's split plant (`_split_march` at `phi_lim = 0.75`, `phi_air = 0.77`, in both coordinates), and
walks every fifth point WHOLE plus min/max/last of every float column — AH's section P, derived
from the point rather than named. Rung 79's six counter DELTAS ride beside each march: on the
incidence march they are the plant's own knob counts, and on the split marches they must be zero
(rung 80 marches in `phi` reference throughout — slice AI step 4's measurement).

**Rung 78's process-global `_gauge_hits` / `_gauge_binds` are NOT emitted**, though rung 79's
`phi` arm dispatches into rung 78's body: in the port they are process-global statics, the race
AH step 7 found, and a value keyed off them here would be a value another test thread can move.

# THE INTERPRETER SENTINEL

`_interp/sum_probe`, AH's: `sum([1e16, 1.0, -1e16])` is `1.0` under CPython 3.12+ and `0.0` under
PyPy's naive fold. Excluded from every comparison by name and asserted per arm.

# THE CPYTHON PREDICTION

Written in `slice_ai_oracle.rs`'s header from a RUNTIME `builtins.sum` census over THIS drive
(`W:/temp/claude/slice-ai-step6/probe_drive.py`), taken before either golden was compared.
"""
import os
import struct
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, REPO)

from turbojet.gas import Gas                                              # noqa: E402
from turbojet.engine import (                                             # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    ResidualGaugeTransient, StateCoordinateTransient, SplitWallTransient,
    BleedLimiter, StatorIncidenceLimiter, StatorLimiter,
)

# ------------------------------------- `tests/test_rung79.py` / `test_rung80.py`'s shared grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI = 1000.0, 1400.0
DS, SETTLE, R = 0.005, 1.2, 0.5
B, V_MAX, TT4_MAX = 0.10, 0.20, 1200.0
TAU, TAU_S = 0.05, 0.05
TAUS = (0.05, 0.05, 0.05, 0.05)
#: Both suites' receiver wall: rung 79's `PHI_JAC`, rung 80's literal `0.80`.
PHI_RIG = 0.80
#: Rung 80's `PHI_FUEL` — every rung-80 fixture's fuel wall.
PHI_FUEL = 0.75
#: `tests/test_rung80.py`'s `gains` / `gains_demand` walls — WIDER than the reader's default.
GAINS_AIRS = (None, 0.77, 0.80)

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

#: Section I: the pre-flight's settings (`_S3_*`, `every = 4`) at its two walls.
I_WALLS = (0.80, 0.76)
I_EVERY = 4
#: Section P's stride, and the split plant's two walls.
P_STRIDE = 5
P_SPLIT_AIR = 0.77

CTR = ("_coord_hits", "_coord_binds", "_coord_fb_phi", "_coord_fb_inc",
       "_coord_calls_phi", "_coord_calls_inc")
CTR_KEYS = ("hits", "binds", "fb_phi", "fb_inc", "calls_phi", "calls_inc")

OUT = []
N_NONE = [0]
N_NEG_ZERO = [0]
N_POS_ZERO = [0]
N_NAN = [0]
TIMES = []


def f(key, x):
    v = float(x)
    bits = struct.unpack("<Q", struct.pack("<d", v))[0]
    if v == 0.0:
        (N_NEG_ZERO if bits else N_POS_ZERO)[0] += 1
    elif v != v:
        N_NAN[0] += 1
    OUT.append((key, bits))


# THE THREE COUNTERS PROVE THEY CAN SEE, BEFORE ANYTHING REAL IS EMITTED — AG's self-test.
_before = (N_POS_ZERO[0], N_NEG_ZERO[0], N_NAN[0], len(OUT))
f("_selftest/pos_zero", 0.0)
f("_selftest/neg_zero", -0.0)
f("_selftest/nan", float("nan"))
f("_selftest/ordinary", 1.5)
assert (N_POS_ZERO[0], N_NEG_ZERO[0], N_NAN[0]) == (_before[0] + 1, _before[1] + 1,
                                                    _before[2] + 1),     "a census counter did not move on a value that is exactly what it counts"
assert OUT[-3][1] == 0x8000000000000000, "-0.0 did not survive as the negative zero"
assert OUT[-4][1] == 0, "+0.0 did not survive as the positive zero"
N_POS_ZERO[0], N_NEG_ZERO[0], N_NAN[0] = _before[0], _before[1], _before[2]
del OUT[_before[3]:]
assert not OUT, "the self-test left a key behind"

# THE INTERPRETER SENTINEL — see the header. A RAW append: outside section Z's census.
OUT.append(("_interp/sum_probe",
            struct.unpack("<Q", struct.pack("<d", float(sum([1e16, 1.0, -1e16]))))[0]))


def d(key, n):
    OUT.append((key, int(n) & 0xFFFFFFFFFFFFFFFF))


def bo(key, x):
    OUT.append((key, 1 if x else 0))


def s(key, text):
    h = 0xCBF29CE484222325
    for ch in text.encode("utf-8"):
        h = ((h ^ ch) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    OUT.append((key, h))


def walk(key, obj):
    """Emit whatever is here, by SHAPE. AG's walker: `bool` before `int`, `#len` not `n`.

    A NEGATIVE int is emitted as its 64-bit two's complement (`d`'s mask) — `coord_march`'s
    `br_phi`/`br_inc` are `calls - fb`, which is not a count by construction."""
    if obj is None:
        bo(key + "?", False)
        N_NONE[0] += 1
        return
    if isinstance(obj, bool):
        bo(key + "?", True)
        bo(key, obj)
    elif isinstance(obj, int):
        bo(key + "?", True)
        d(key, obj)
    elif isinstance(obj, float):
        bo(key + "?", True)
        f(key, obj)
    elif isinstance(obj, str):
        bo(key + "?", True)
        s(key, obj)
    elif isinstance(obj, (list, tuple)):
        bo(key + "?", True)
        d(key + "/#len", len(obj))
        for i, x in enumerate(obj):
            walk("%s/%d" % (key, i), x)
    elif isinstance(obj, dict):
        bo(key + "?", True)
        d(key + "/#len", len(obj))
        # `%s` of a key: `split_gains`' `authority` dict can be keyed by `None` -> `None`
        for k in sorted(obj, key=str):
            walk("%s/%s" % (key, k), obj[k])
    else:
        raise AssertionError("unhandled type %r at %s" % (type(obj), key))


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def rig(cls, inc):
    """Both suites' `_rig`, with the stator arm made explicit.

    `sm = 0.80 / FLOOR - 1` and the four knobs `demand` / `sched` / `none` / `solve` — which are
    also the fingerprint's `_s3_rig` after its two assignments, because `_windup_law = "none"`
    and `_cap_law = "solve"` are the class defaults. So ONE rig serves section I too.
    """
    sm = PHI_RIG / FLOOR - 1.0
    m = cls(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                        else None),
            stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m


def arms(tag, inc):
    """The per-arm prefix, and the arm flag beside it — CALL ONCE PER ARM (AG § 5.31.6 (c) 1)."""
    p = "%s/i%d" % (tag, 1 if inc else 0)
    bo(p + "/arm_inc", inc)
    return p


def ctr():
    return tuple(getattr(StateCoordinateTransient, k) for k in CTR)


def emit_ctr(tag, c0, c1):
    for k, a, b in zip(CTR_KEYS, c0, c1):
        d("%s/%s" % (tag, k), b - a)


class timed:
    def __init__(self, name):
        self.name = name

    def __enter__(self):
        self.t0 = time.time()

    def __exit__(self, *a):
        TIMES.append((self.name, time.time() - self.t0))
        sys.stderr.write("  %-26s %7.1fs\n" % (self.name, TIMES[-1][1]))
        sys.stderr.flush()


# =====================================================================================
# A-D -- RUNG 79's four readers, at their own defaults, on a rung-79 receiver.
# =====================================================================================
for inc in (False, True):
    with timed("A coord_scan i%d" % inc):
        walk(arms("A", inc), rig(StateCoordinateTransient, inc).coord_scan(
            FLIGHT, LO, HI, TT4_MAX, inc=inc))
for inc in (False, True):
    with timed("B coord_census i%d" % inc):
        walk(arms("B", inc), rig(StateCoordinateTransient, inc).coord_census(
            FLIGHT, LO, HI, TT4_MAX, inc=inc))
for inc in (False, True):
    with timed("C coord_march i%d" % inc):
        walk(arms("C", inc), rig(StateCoordinateTransient, inc).coord_march(
            FLIGHT, LO, HI, TT4_MAX, inc=inc))
for inc in (False, True):
    with timed("D coord_forced i%d" % inc):
        walk(arms("D", inc), rig(StateCoordinateTransient, inc).coord_forced(
            FLIGHT, LO, HI, TT4_MAX, inc=inc))

# =====================================================================================
# E-J -- RUNG 80's four readers at the suite's arguments, on a rung-80 receiver.
# `split_saturation` is called by no Python test; it runs at its own eight default walls.
# =====================================================================================
for inc in (False, True):
    with timed("E split_liveness i%d" % inc):
        walk(arms("E", inc), rig(SplitWallTransient, inc).split_liveness(
            FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL, inc=inc))
for inc in (False, True):
    with timed("F split_arrest i%d" % inc):
        walk(arms("F", inc), rig(SplitWallTransient, inc).split_arrest(
            FLIGHT, LO, HI, TT4_MAX, phi_lim_lo=PHI_FUEL, inc=inc))
for inc in (False, True):
    with timed("G split_saturation i%d" % inc):
        walk(arms("G", inc), rig(SplitWallTransient, inc).split_saturation(
            FLIGHT, LO, HI, TT4_MAX, inc=inc))
for inc in (False, True):
    with timed("H split_gains clip i%d" % inc):
        walk(arms("H", inc), rig(SplitWallTransient, inc).split_gains(
            FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL, phi_airs=GAINS_AIRS, coord="clip",
            inc=inc))
for inc in (False, True):
    with timed("J split_gains demand i%d" % inc):
        walk(arms("J", inc), rig(SplitWallTransient, inc).split_gains(
            FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL, phi_airs=GAINS_AIRS, coord="demand",
            inc=inc))

# =====================================================================================
# I -- § 5.33 (i): rung 74's `demand_gains` on rung-78/79/80 machines, counters beside.
# =====================================================================================
for tag, cls in (("r78", ResidualGaugeTransient), ("r79", StateCoordinateTransient),
                 ("r80", SplitWallTransient)):
    for wall in I_WALLS:
        for inc in (False, True):
            p = arms("I/%s/w%s" % (tag, wall), inc)
            with timed("I %s %s i%d" % (tag, wall, inc)):
                c0 = ctr()
                g = rig(cls, inc).demand_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=wall,
                                               taus=TAUS, inc=inc, r=R, s_settle=SETTLE,
                                               ds=DS, v_max=V_MAX, every=I_EVERY)
                c1 = ctr()
            walk(p + "/g", g)
            emit_ctr(p + "/ctr", c0, c1)


# =====================================================================================
# P -- THE PLANTS THEMSELVES, walked GENERICALLY -- AH's section P.
# =====================================================================================
def emit_march(tag, traj):
    d(tag + "/n", len(traj))
    # An EMPTY march would agree with itself on every key below (§ 5.27 (ii)).
    assert traj, "an empty march is not a measurement: %s" % tag
    for i in range(0, len(traj), P_STRIDE):
        walk("%s/p%d" % (tag, i), traj[i])
    # THE COLUMNS ARE DERIVED FROM THE POINT, NOT NAMED: every key whose value is a float.
    cols = sorted(k for k, v in traj[0].items() if isinstance(v, float))
    d(tag + "/#cols", len(cols))
    for k in cols:
        col = [p[k] for p in traj]
        assert all(isinstance(x, float) for x in col), "column %s changes type mid-march" % k
        f("%s/col/%s/min" % (tag, k), min(col))
        f("%s/col/%s/max" % (tag, k), max(col))
        f("%s/col/%s/last" % (tag, k), col[-1])
        d("%s/col/%s/neg" % (tag, k), sum(1 for x in col if x < 0.0))


for inc in (False, True):
    p = arms("P", inc)
    with timed("P coord plant i%d" % inc):
        # rung 79's INCIDENCE plant -- `coord_march`'s `traj1`, WITHOUT the probe (the probe
        # appends a log and bumps nothing; `coord_march` reads its log in section C).
        m = rig(StateCoordinateTransient, inc)
        sm = PHI_RIG / m.map_lp_design.phi_surge - 1.0
        acc = m._with_coord("phi", m.accel_for, FLIGHT, LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc,
                            0.10)
        c0 = ctr()
        traj = m._with_coord("incidence", m._cap_march, FLIGHT, LO, HI, TT4_MAX, sm, TAUS, R,
                             SETTLE, DS, V_MAX, inc, "demand", "sched", "none", None, "solve",
                             acc)[3]
        c1 = ctr()
    emit_ctr(p + "/coord/ctr", c0, c1)
    emit_march(p + "/coord", traj)
    for coord in ("demand", "clip"):
        with timed("P split %s i%d" % (coord, inc)):
            m = rig(SplitWallTransient, inc)
            c0 = ctr()
            traj = m._split_march(FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, P_SPLIT_AIR, coord, TAUS, R,
                                  SETTLE, DS, V_MAX, inc)[3]
            c1 = ctr()
        emit_ctr("%s/split_%s/ctr" % (p, coord), c0, c1)
        emit_march("%s/split_%s" % (p, coord), traj)

# =====================================================================================
# Z -- THE CENSUSES.
# =====================================================================================
d("Z/n_none", N_NONE[0])
d("Z/n_neg_zero", N_NEG_ZERO[0])
d("Z/n_pos_zero", N_POS_ZERO[0])
d("Z/n_nan", N_NAN[0])

seen = {}
for k, v in OUT:
    assert k not in seen, "DUPLICATE KEY %s" % k
    seen[k] = v
path = sys.argv[1] if len(sys.argv) > 1 else None
assert path, "usage: dump_slice_ai.py <out.tsv>"
with open(path, "w", encoding="utf-8", newline="\n") as fh:
    for k in sorted(seen):
        fh.write("%s\t%d\n" % (k, seen[k]))
sys.stderr.write("keys %d -> %s\n" % (len(seen), path))
sys.stderr.write("  none %d   neg_zero %d   pos_zero %d   nan %d\n"
                 % (N_NONE[0], N_NEG_ZERO[0], N_POS_ZERO[0], N_NAN[0]))
sys.stderr.write("  total %.1fs\n" % sum(t for _, t in TIMES))
