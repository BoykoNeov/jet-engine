"""SLICE AH step 6 (b) — **THE ORACLE for rungs 77 AND 78**, run once on PyPy and once on CPython.

Emits `key<TAB>u64` to a file, sorted, with the tally on stderr. Every float is its IEEE-754 bit
pattern, every string an FNV-1a 64-bit hash, every `None` a PRESENCE FLAG beside the value it
stands in for, and every sample-shaped reading carries its ROW COUNT.

    pypy   rust/oracle/dump_slice_ah.py rust/oracle/slice_ah_pypy.tsv
    python rust/oracle/dump_slice_ah.py rust/oracle/slice_ah_cpython.tsv

Slice AG's dumper is the template (`dump_slice_ag.py`): the key set is WALKED here and
HAND-LISTED on the Rust side, and `slice_ah_oracle.rs` compares the two key SETS before any value,
so a key both emitters forgot is a named failure rather than a shared silence. What is different
at this slice is below.

# EVERY PER-ARM READER IS DRIVEN ON BOTH STATOR ARMS, THOUGH NEITHER SUITE SWEEPS ONE

`tests/test_rung77.py` and `tests/test_rung78.py` call eight of the nine readers at `inc = False`
only; `stiffness_ledger` is the one that sweeps `arms=(False, True)` itself. § 5.31.5 (k) is the
reason this file does not copy that: a grid NARROWED from the suite's is the same defect as a grid
COPIED from it, and 1 814 of AG's keys were once produced with rung 69's incidence plant never
touched. So A-C and E-H run both arms; D runs once, at its own defaults, which already cover both.

# THE PLANT SECTION IS **CLIP**, NOT DEMAND

AG's section H walked DEMAND-coordinate marches through a hand-typed 28-column list. Every rung-77
and rung-78 reader except `gauge_march` reads `_ledger_march`, which marches **CLIP** — a
30-key point with no `w_*`/`cap_*`/`lag_coord`, where AG's list would have raised a `KeyError` on
the first point. So section P walks each strided point GENERICALLY, and its column folds range
over the float keys of the point itself rather than over a list: **the set is DERIVED, not
named** (§ 5.32 (viii) defect 5). `P/*/gauged` is the one DEMAND march — rung 78's gauged plant at
`mult = 2.0`, the trajectory whose worst deviation section H reports as a single number.

# THE FLOAT-KEYED DICTS

`gauge_scan`'s `ks` and `root_census`' `cells` are keyed by the `mult` FLOAT, and `walk` spells a
key segment with `%s`, i.e. `repr`: `0.0`, `2.0`, `-0.5`. Rust's `{}` prints `0` and `2`. The Rust
side formats them as Python does; a slip there is two missing and two extra keys, which the
key-set gate names before any value is read.

# THE INTERPRETER SENTINEL, AND WHY IT EXISTS

AG's `the_two_goldens_are_not_the_same_file` asserts the two goldens DIFFER somewhere — which was
true there because 406 keys differed. **If this slice's CPython arm differs on nothing, that gate
cannot tell a correct zero from a copied file.** So each arm emits `_interp/sum_probe`: the bits of
`sum([1e16, 1.0, -1e16])`, which is `1.0` under CPython 3.12+'s compensated `sum` and `0.0` under
PyPy's naive fold (and Rust's). The Rust side excludes `_interp/*` by name from every comparison and
asserts the two goldens DISAGREE on it — the instrument proves it can see the one difference it
exists to see, rather than the gate inferring provenance from whatever else happened to differ.
It is written with a raw append, not `f()`, so it does not enter section Z's census.

# PRE-REGISTERED BEFORE ANY GOLDEN WAS COMPARED — AND WRITTEN FROM A DERIVED SET, NOT A NAMED ONE

§ 5.32 (iv) found no float `sum()` INSIDE the two classes, and said in so many words that it was
not a claim about the call graph. AG's P2 is why that distinction matters: it sized its exemption
from the sums it had reasoned about and missed `_charpoly4`, a four-term sum in a SHARED helper,
by 398 keys. So the prediction below is not read off § (iv). It is read off a RUNTIME census:
`builtins.sum` wrapped for one PyPy run of THIS drive, recording every call by its caller's
`file:line` and whether any element is a float (`W:/temp/claude/slice_ah_oracle/probe_drive.py`).

**MEASURED:** outside this file, the drive reaches exactly ONE `sum()` site, `engine.py:20462`,
which is `gauge_scan`'s `n_bad` count over integer literals, i.e. no float summation anywhere in
the call graph the readers and plants reach. (This file's own two sites are the sentinel above and
`emit_march`'s integer `neg` count.)

**THE PREDICTION: the CPython golden is bit-identical to the PyPy golden on EVERY key except
`_interp/sum_probe`**, so `slice_ah_oracle.rs` carries NO exemption at all, the first oracle since
slice V without one.
**THE FALSIFIER:** any other key that differs. If one does, the divergence is not summation, or the
census missed a path (a `sum` bound under another name, or a site this drive never reached), and
the gate is not relaxed to fit.

**AND P3 IS MEASURED ON A DRIVE THAT VISITS ITS SITES.** The same run re-ran the pre-flight's
per-site nest counter (probe 13, positive control first): **110 nests in 811 465 `_b_state` sets**
(the same count on `_v_state`), at all five static sites. `leg_slopes` → `_c_at` 12 + 60 (via
`stiffness_ledger`), `gauge_scan` 12, `root_census` 12, `gauge_march` 2, and **`gauge_vs_device` →
`_phi_at` 12**. That last one is the site safe ONLY by the dead-window criterion, and it is the one
a zero-visit drive would have left unscored. A bit-identical PyPy arm is therefore P3 CONFIRMED,
not P3 not-yet-tested.
"""
import os
import struct
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, REPO)

from turbojet.gas import Gas                                              # noqa: E402
from turbojet.engine import (                                             # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    StiffnessLedgerTransient, ResidualGaugeTransient,
    BleedLimiter, StatorIncidenceLimiter, StatorLimiter,
)

# ------------------------------------- `tests/test_rung77.py` / `test_rung78.py`'s shared grid
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
PHI_JAC = 0.80
MARGIN = 0.10

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

#: Section P's stride. Every fifth point WHOLE, plus min/max/last of every float column.
P_STRIDE = 5
#: Section P's gauged march: `gauge_march`'s own `2.0`, the multiple past the singular gauge.
P_GAUGE_MULT = 2.0

OUT = []
N_NONE = [0]
N_NEG_ZERO = [0]
N_POS_ZERO = [0]
N_NAN = [0]


def f(key, x):
    v = float(x)
    bits = struct.unpack("<Q", struct.pack("<d", v))[0]
    if v == 0.0:
        (N_NEG_ZERO if bits else N_POS_ZERO)[0] += 1
    elif v != v:
        N_NAN[0] += 1
    OUT.append((key, bits))


# THE THREE COUNTERS PROVE THEY CAN SEE, BEFORE ANYTHING REAL IS EMITTED — AG's self-test, verbatim
# in effect: `-0.0 == 0.0` is `True`, so a classifier branching on the comparison instead of the
# sign bit would make `n_neg_zero` unreachable by construction.
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
    """Emit whatever is here, by SHAPE. AG's walker: `bool` before `int`, `#len` not `n`."""
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
    """The two suites' `_rig` — the RECEIVER, at `PHI_JAC`, with the stator arm made explicit.

    Both suites build every reader's receiver at `sm = PHI_JAC / FLOOR - 1`, and set the same four
    knobs by plain assignment: `demand` / `sched` / `none` / `solve`.
    """
    sm = PHI_JAC / FLOOR - 1.0
    m = cls(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                        else None),
            stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law, m._windup_law, m._tau_t = "demand", "sched", "none", None
    m._cap_law = "solve"
    return m


def arms(tag, inc):
    """The per-arm prefix, and the arm flag beside it — CALL ONCE PER ARM (AG § 5.31.6 (c) 1).

    `arm_inc`, not `inc`: every reader here publishes an `inc` field at exactly this depth.
    """
    p = "%s/i%d" % (tag, 1 if inc else 0)
    bo(p + "/arm_inc", inc)
    return p


# =====================================================================================
# A-C -- RUNG 77's three per-arm readers, at their OWN defaults beyond the suite's
# `phi_lim`/`margin`, on a rung-77 receiver.
# =====================================================================================
for inc in (False, True):
    p = arms("A", inc)
    walk(p, rig(StiffnessLedgerTransient, inc).leg_slopes(
        FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=MARGIN, inc=inc))

for inc in (False, True):
    p = arms("B", inc)
    walk(p, rig(StiffnessLedgerTransient, inc).set_point_gains(
        FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=MARGIN, inc=inc))

for inc in (False, True):
    p = arms("C", inc)
    walk(p, rig(StiffnessLedgerTransient, inc).singular_limit(
        FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=MARGIN, inc=inc))

# =====================================================================================
# D -- `stiffness_ledger`, ONCE, at its own 24-cell default. Its fourth positional
# argument binds a parameter the body never reads (§ 5.32.6 (e)); it is passed because the
# suite passes it, and the port has no counterpart to pass it to.
# =====================================================================================
walk("D", rig(StiffnessLedgerTransient, False).stiffness_ledger(FLIGHT, LO, HI, TT4_MAX))

# =====================================================================================
# E-H -- RUNG 78's four readers, on a rung-78 receiver.
# =====================================================================================
for inc in (False, True):
    p = arms("E", inc)
    walk(p, rig(ResidualGaugeTransient, inc).gauge_scan(FLIGHT, LO, HI, TT4_MAX, inc=inc))

for inc in (False, True):
    p = arms("F", inc)
    walk(p, rig(ResidualGaugeTransient, inc).root_census(FLIGHT, LO, HI, TT4_MAX, inc=inc))

for inc in (False, True):
    p = arms("G", inc)
    walk(p, rig(ResidualGaugeTransient, inc).gauge_vs_device(FLIGHT, LO, HI, TT4_MAX, inc=inc))

C0 = {}
for inc in (False, True):
    p = arms("H", inc)
    g = rig(ResidualGaugeTransient, inc).gauge_march(FLIGHT, LO, HI, TT4_MAX, inc=inc)
    C0[inc] = g["c0"]
    walk(p, g)


# =====================================================================================
# P -- THE PLANTS THEMSELVES, walked GENERICALLY -- see the header.
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
        # THE SIGN CENSUS, over EVERY float column rather than AG's four named ones.
        d("%s/col/%s/neg" % (tag, k), sum(1 for x in col if x < 0.0))


for inc in (False, True):
    p = arms("P", inc)
    sm = PHI_JAC / FLOOR - 1.0
    # the march EVERY reader in A-G stands on -- CLIP, accel-armed, identity gauge
    emit_march(p + "/ledger",
               rig(StiffnessLedgerTransient, inc)._ledger_march(
                   FLIGHT, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, inc, MARGIN)[3])
    # rung 78's GAUGED plant: DEMAND, `k = 2 / c0` with `c0` section H's own
    m = rig(ResidualGaugeTransient, inc)
    acc = m.accel_for(FLIGHT, LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc, MARGIN)
    k = P_GAUGE_MULT / C0[inc]
    f(p + "/gauged_k", k)
    # THE TRAJECTORY CANNOT SEE THE GAUGE, SO ITS COUNTER RIDES BESIDE IT. The gauge is INERT on
    # this march (`gauge_march`'s `worst` is exactly 0.0 at every multiple), so the gauged points
    # are bit-identical to ungauged ones -- and an injection that dropped the gauge from this very
    # march moved 0 of 29 284 keys (§ 5.32.6 (i), K5). What does move is whether the GAUGED branch
    # was entered, which is what these two counts pin.
    ResidualGaugeTransient._gauge_hits = 0
    ResidualGaugeTransient._gauge_binds = 0
    traj = m._with_gauge(k, m._cap_march, FLIGHT, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE,
                         DS, V_MAX, inc, "demand", "sched", "none", None, "solve", acc)[3]
    d(p + "/gauged_hits", ResidualGaugeTransient._gauge_hits)
    d(p + "/gauged_binds", ResidualGaugeTransient._gauge_binds)
    emit_march(p + "/gauged", traj)

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
assert path, "usage: dump_slice_ah.py <out.tsv>"
with open(path, "w", encoding="utf-8", newline="\n") as fh:
    for k in sorted(seen):
        fh.write("%s\t%d\n" % (k, seen[k]))
sys.stderr.write("keys %d -> %s\n" % (len(seen), path))
sys.stderr.write("  none %d   neg_zero %d   pos_zero %d   nan %d\n"
                 % (N_NONE[0], N_NEG_ZERO[0], N_POS_ZERO[0], N_NAN[0]))
