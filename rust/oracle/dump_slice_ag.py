"""SLICE AG step 6 — **THE ORACLE for rungs 75 AND 76**, run once on PyPy and once on CPython.

Emits `key<TAB>u64` to a file, sorted, with the tally on stderr. Every float is its IEEE-754 bit
pattern, every string an FNV-1a 64-bit hash, every `None` a PRESENCE FLAG beside the value it
stands in for, and every sample-shaped reading carries its ROW COUNT.

    pypy   rust/oracle/dump_slice_ag.py rust/oracle/slice_ag_pypy.tsv
    python rust/oracle/dump_slice_ag.py rust/oracle/slice_ag_cpython.tsv

# THE KEY SET IS **WALKED**, NOT TRANSCRIBED, AND THAT IS THIS DUMPER'S ONE DESIGN DIFFERENCE

Slices AA-AF each hand-listed both sides. A key present in the Python reading and forgotten in
BOTH emitters is then invisible to every check either side runs -- which is
`rust-port-documented-gate-that-doesnt-exist` in its oracle-shaped form: *a count guard is blind
to a class absent from BOTH sides.*

So the Python side here does not name keys at all. `walk()` descends the returned dict and emits
whatever is in it; the Rust side still hand-lists, because it has structs and no reflection. **The
two key SETS are then compared for equality by `slice_ag_oracle.rs` before any value is**, so a
key the port forgot to emit is a NAMED FAILURE rather than a silence. That check is the whole
reason for the asymmetry, and it only works in this direction: a generic Rust walker would
reintroduce the shared blindness.

# THE GRIDS ARE THE SHIPPED READERS' OWN DEFAULTS, AND THE `inc` AXIS IS MANDATORY

§ 5.27.6 (i) burned a slice on transcribing one reader's stride into another, so nothing below
passes an argument the reader defaults -- except the two the suite itself sweeps (`margin`, and
`windup_gains`' reversed `tau_ts`), which are here BECAUSE the suite sweeps them.

**AND EVERY READER IS DRIVEN ON BOTH STATOR ARMS.** § 5.31.5 (k) is the reason and it is recent:
that step's own drive passed `inc = False` everywhere, so 1 814 keys were produced with rung 69's
incidence-referenced plant entirely untouched -- while `test_rung76.py` sweeps `for inc in (False,
True)` in three of its tests. Re-driven with the axis added, the golden went to 2 691 keys. A grid
NARROWED from the suite's is the same defect as a grid COPIED from it, one step earlier.

# WHAT THE FOLDS CANNOT DO, AND THE SECTION THAT DOES IT

**AN AGGREGATE IS LOSSY, SO SECTION H EMITS THE MARCHES THEMSELVES.** A-G are the readers' folds
over trajectories; AD step 5 found its six drifting keys at 2 points of 1 302 only because the
equivalent section existed. H walks `_windup_march` and `_cap_march` on both stator arms and emits
every fifth point WHOLE, plus the `min`, `max` and LAST of every float column over ALL points.

**That backstop is real and it is weaker than "nothing can hide"** -- AD step 5's close-out
measured a hidden-point defect moving 0 of 54 116 keys against a control that moved 10. What H
pins is one point in five plus both extremes and the endpoint. No more, and it is written that way
round deliberately.

**A CELL CAN BREAK BY EMPTYING THE SAMPLE**, so every list emits its length before its members and
every `None` a flag of its own. `contraction_law` legitimately returns `measured=None` on the arms
the raised cap still cannot reach -- that is § 2's finding, not a gap -- and `device_control`'s
`cutting_output` can be `None` on a cell where nothing cuts. `None == None` agrees perfectly and
measures nothing, so section Z emits the COUNT of them as a key of its own.

# THE THREE HAZARDS THESE TWO RUNGS SUPPLY

* **SIGNED ZERO.** Bit patterns are emitted, so `-0.0` and `+0.0` are DIFFERENT values for the
  same key; section Z counts both, because AE step 4 found 63 keys flipping `+0.0`-ness between
  the two goldens. Rung 75's `mask_leak`/`track_leak` and rung 76's `masked_moved`/`gov_row` are
  exact zeros on every shipped cell, which is exactly the population where the sign bit is free
  to differ without any relative bar noticing.
* **A REACHABLE `nan`.** `solve_gain`'s `gain` is `dD/dS` and Python writes `nan` where `dS` is
  exactly zero. Step 5 measured that at **0 of 36 rows**, so it is unreachable on this plant --
  but `nan != nan`, so a comparison by VALUE would report a difference on a key both sides
  computed identically. Bit patterns are compared, so a `nan` matches a `nan` with the same
  payload; section Z counts them so an oracle that started producing them cannot do so silently.
* **THE ONE UNTAGGED MESSAGE.** `engine.py:19523` -- *the two cap laws marched different grids* --
  is the only assert in either class that does not open with its rung number (§ 5.31 (v): 4 of 4
  tagged at rung 75, 4 of 5 here). It is not reachable from any grid below, so nothing here
  witnesses it; that is P6's subject and step 7's, and it is named here so the absence is a
  recorded decision rather than an oversight.

# P2, PRE-REGISTERED HERE BEFORE THE CPYTHON ARM HAS EVER RUN

§ 5.31 (iv) drove ONE reader end to end plus the marches and measured the CPython arm needing an
exemption for exactly **two keys of 83 273** -- `cap_bill`'s `fuel_int/0` and `/1`, the two
`sum()` calls in either class that add a 341-long trajectory rather than a literal `1`. CPython
3.12+'s `sum` is Neumaier-compensated; PyPy's and Rust's are naive left folds.

**THE PREDICTION: the exemption list stays exactly `F/*/fuel_int/0` and `F/*/fuel_int/1`**, across
every cell this dumper drives -- which is more readers than that measurement covered
(`windup_gains`, `contraction_law`, `device_control`, `windup_bill`, `cap_gains` and `solve_gain`
were all outside it).

**THE FALSIFIER, stated so the prediction can lose**: if the CPython arm differs on a key outside
that set, P2 is wrong and the cause is not summation order. Section H's plant keys are NOT
exempted in advance -- AF's P2 was falsified by 49 keys in a reader nobody had suspected, and
pre-exempting a whole section is how that happens twice.

**SCORED AFTER THE ARM RAN: P2 IS FALSIFIED, 398 KEYS WIDE.**  406 of 38 100 differ.  All eight
`fuel_int` keys it named DO differ, so the CAUSE is confirmed and the falsifier's own wording --
*the cause is not summation order* -- is the half that was wrong: it IS summation order, in a
place the prediction's reasoning excluded.  **P2 sized the exemption by the LENGTH of the sum**,
exempting the ones that add 341 things and holding a short one safe.  `_charpoly4`
(Faddeev-LeVerrier, `engine.py:16235` / `16237`) is built on two `sum()` calls of **FOUR TERMS**,
a matrix product and a trace, and both diverge -- compensated summation parts company with a naive
fold as soon as the addends have mixed magnitudes, which a Jacobian's characteristic polynomial
has by construction.  **And the blast radius is set by SHARING, not by SIZE**: the suspected
341-term sum sits in one reader and moved 8 keys, while the unsuspected 4-term one sits in a
SHARED static helper and moved 398 across three sections and six readers.  Section H, the one
place this file refused to pre-exempt, is CLEAN.
"""
import os
import struct
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, REPO)

from turbojet.gas import Gas                                              # noqa: E402
from turbojet.engine import (                                             # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    AntiWindupTransient, SensedCapTransient,
    BleedLimiter, StatorIncidenceLimiter, StatorLimiter,
)

# ------------------------------------- `tests/test_rung75.py` / `test_rung76.py`'s shared grid
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
PHI_JAC, PHI_BOTH = 0.80, 0.76
TAU_T, TAU_T_FAST = 0.05, 0.0125
MARGIN, MARGIN_HI = 0.10, 0.20

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

#: Section H's stride. Every fifth point WHOLE — see the header on what that does and does not pin.
H_STRIDE = 5

OUT = []
#: Section Z's censuses, accumulated as the emitters run rather than recomputed afterwards.
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


# -------------------------------------------------------------------------------------
# THE THREE COUNTERS PROVE THEY CAN SEE, BEFORE ANYTHING REAL IS EMITTED.
#
# Slice W step 3: *every one a zero nobody measured, so make the instrument prove it can
# SEE.*  `Z/n_neg_zero` and `Z/n_nan` come back 0 on this plant, and a 0 from a counter
# that has never incremented is indistinguishable from a counter that CANNOT.  The trap is
# specific and one line wide: `-0.0 == 0.0` is `True` in Python, so a classifier written
# `if x == 0.0: pos_zero += 1` would fold every negative zero into the positive count and
# `n_neg_zero` would be unreachable BY CONSTRUCTION -- a gate agreeing with itself about a
# sign-bit difference that would then cross both arms invisibly.
#
# `f()` branches on `bits`, not on the comparison, so it is written correctly.  This proves
# it, rather than trusting the reading: four values through the real `f()`, each counter
# asserted to move by exactly one, then every side effect undone.
# -------------------------------------------------------------------------------------
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
    """Emit whatever is here, by SHAPE — the header's one design difference.

    `bool` is tested before `int` because `bool` IS an `int` in Python and a `d()` of `True`
    would lose the distinction the Rust side spells as two different types.

    **THE CONTAINER LENGTH IS `#len` AND NOT `n`, BECAUSE THREE READERS PUBLISH A FIELD CALLED
    `n`.** `contraction_law`, `cap_bill` and `solve_gain` each return one, so a walker spelling
    its own length as `/n` would write two different numbers to the same key -- caught by the
    duplicate-key assert at the bottom, which is why that assert is there and not a comment.
    """
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


def rig(cls, phi_lim, inc=False):
    """The two suites' `_rig(design, cls, _sm(phi_lim), inc=inc)` — the RECEIVER.

    The readers build their own rig from `sm`; this is the machine they are called ON, and its
    `_ic_cap` is what `_windup_march`/`_cap_march` carry, so the receiver's state is load-bearing
    for `contraction_law` in particular.
    """
    sm = phi_lim / FLOOR - 1.0
    m = cls(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                        else None),
            stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law = "demand", "sched"
    m._windup_law, m._tau_t = "none", None
    return m


def arms(tag, inc):
    """The two per-arm key prefixes, and the arm flag emitted beside them.

    § 5.31.5 (k) renamed every key when the axis was added and the sweep's per-margin counters
    silently read zero on all sixteen rows. So the arm is part of the key from the start here,
    not appended to an existing scheme later.

    **The flag is `arm_inc` and not `inc`** because `device_control` and `windup_bill` each
    publish an `inc` field of their own at exactly this depth, and the two would collide.

    **CALL THIS ONCE PER ARM, NOT ONCE PER CELL.**  It EMITS, so a call inside an inner
    loop writes `arm_inc` once per inner iteration -- sections E/F/G each have a second
    axis and each did exactly that on the first drive.  The duplicate-key assert caught
    all three; the point is that a helper which both BUILDS a prefix and EMITS a value
    is only safe at the depth its emission belongs to.  Hoist it, and index off the
    string it returns.
    """
    p = "%s/i%d" % (tag, 1 if inc else 0)
    bo(p + "/arm_inc", inc)
    return p


# =====================================================================================
# A -- `windup_gains`.  The reader's OWN defaults (`refs=("applied","sched")`,
# `tau_ts=(0.05, 0.0125)`), plus the suite's REVERSED tuple as a separate cell.
#
# The reversal is not cosmetic: `ratios` divides the FIRST cell's reading by the LAST,
# so the two orders produce reciprocal ratios (2.5 against 0.4) off the same cells.
# `rung75.rs`'s header records that its own gate reads no ratio there, which is exactly
# why the orientation needs pinning HERE.
# =====================================================================================
for inc in (False, True):
    p = arms("A", inc)
    m = rig(AntiWindupTransient, PHI_JAC, inc)
    walk(p + "/fwd", m.windup_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, inc=inc))
    walk(p + "/rev", m.windup_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC,
                                    tau_ts=(TAU_T_FAST, TAU_T), inc=inc))

# =====================================================================================
# B -- `contraction_law`.  The reader's OWN six-clock default, which is WIDER than the
# suite's four: the two fastest are where `measured` legitimately comes back `None`, and
# a `None` on both sides is exactly what section Z's counter exists for.
# =====================================================================================
for inc in (False, True):
    p = arms("B", inc)
    walk(p, rig(AntiWindupTransient, PHI_BOTH, inc)
             .contraction_law(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, inc=inc))

# =====================================================================================
# C -- `device_control`.  The reader's OWN defaults, both references.
# =====================================================================================
for inc in (False, True):
    p = arms("C", inc)
    walk(p, rig(AntiWindupTransient, PHI_BOTH, inc)
             .device_control(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, inc=inc))

# =====================================================================================
# D -- `windup_bill`.  The reader's OWN eight-clock default, which is wider than the
# suite's five AND STARTS ON THE ADMISSIBILITY BOUNDARY: `tau_ts[0] = 0.00625` is what
# `WINDUP_TAU_GRID_FLOOR` evaluates to at this grid, so whether `_rk4_floor_shared`
# admits EQUALITY is a value this section pins rather than a detail.
# =====================================================================================
for inc in (False, True):
    p = arms("D", inc)
    walk(p, rig(AntiWindupTransient, PHI_BOTH, inc)
             .windup_bill(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, inc=inc))

# =====================================================================================
# E -- `cap_gains`, at BOTH margins.  `0.10` is the suite's; `0.20` is § 5.31.5 (a)'s
# second one, where `accel_binds` actually discards rows and a governor-authoritative
# cell exists at all -- the two expressions the suite's own grid leaves dark.
# =====================================================================================
for inc in (False, True):
    base = arms("E", inc)
    for mi, margin in enumerate((MARGIN, MARGIN_HI)):
        p = "%s/m%d" % (base, mi)
        # `grid_margin`, not `margin`: both readers RETURN a `margin` field at this depth.
        f(p + "/grid_margin", margin)
        walk(p, rig(SensedCapTransient, PHI_JAC, inc)
                 .cap_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=margin, inc=inc))

# =====================================================================================
# F -- `cap_bill`, both references.  **THE TWO `fuel_int` KEYS ARE P2's WHOLE SUBJECT**
# and they are emitted per cell, so the prediction can be scored per cell rather than
# in aggregate.
#
# **THE TWO REFERENCE ARMS ARE NOT SYMMETRIC, AND THE FIRST DRIVE OF THIS SECTION FOUND
# OUT THE HARD WAY.** `cap_bill` marches the DEMAND coordinate directly -- unlike
# `cap_rows` and `solve_gain`, which march the CLIP plant and read Jacobians afterwards --
# so `("applied", "none")` is rung 74 s 4's cell with NO INTERIOR EQUILIBRIUM and the
# march refuses (`the joint initial condition did not converge`, residual 2.898e-03, which
# is rung 74's own reported number and rung 75 s 2's whole subject). The applied arm is
# reachable ONLY with rung 75's device armed, so it is driven at `law="track"`. Widening a
# grid along one axis can require arming a DIFFERENT one, and a refusal three rungs old is
# what says so.
#
# The two trajectories the reader returns are NOT walked here -- they are section H's,
# strided. Walking two 341-point marches whole would add ~24 000 keys for a backstop
# section H already provides at one point in five plus both extremes. Their LENGTHS are
# emitted, because `cap_bill` asserts the two grids match and a pair that both went empty
# would satisfy every fold above it.
# =====================================================================================
for inc in (False, True):
    base = arms("F", inc)
    for ref, law in (("sched", "none"), ("applied", "track")):
        p = "%s/%s" % (base, ref)
        out = rig(SensedCapTransient, PHI_BOTH, inc).cap_bill(
            FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, ref=ref, law=law, inc=inc)
        # `traj` is ONE key holding BOTH marches (`dict(solve=a, sensed=b)`) -- not two fields.
        # The port splits it into two struct fields; the ORACLE follows PYTHON's shape, because
        # the key set is what the two sides are compared on.
        traj = out.pop("traj")
        for k in ("solve", "sensed"):
            d("%s/traj/%s/n" % (p, k), len(traj[k]))
        walk(p, out)

# =====================================================================================
# G -- `solve_gain`, on the suite's own THREE margins x both arms.  § 5.31.5 (d)'s
# population: `fixed_point` is exactly `+0.0` at 18 of these 36 rows, which is why
# `rung76.rs` adds a gate that scores the identity only where it is NOT already exact.
# =====================================================================================
for inc in (False, True):
    base = arms("G", inc)
    for gi, margin in enumerate((0.05, MARGIN, 0.40)):
        p = "%s/m%d" % (base, gi)
        # `grid_margin`, not `margin`: both readers RETURN a `margin` field at this depth.
        f(p + "/grid_margin", margin)
        walk(p, rig(SensedCapTransient, PHI_JAC, inc)
                 .solve_gain(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=margin, inc=inc))

# =====================================================================================
# H -- THE PLANTS THEMSELVES.  `_windup_march` and `_cap_march`, both stator arms:
# every fifth point WHOLE, plus min/max/last of every float column over ALL points.
#
# A-G are folds; a difference at one point of 341 survives every one of them.
# =====================================================================================
H_FLOATS = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
            "mdot_air", "sp_thrust", "mf", "mf_sched", "g", "required", "b", "b_cmd",
            "v", "v_cmd", "ic_res", "g_fuel", "g_gov", "required_fuel", "required_gov",
            "w_fuel", "w_gov", "cap_fuel", "cap_gov")
H_INTS = ("ic_iters",)
H_STRS = ("branch", "authority", "share_law", "ic_order", "lag_coord", "v_regime")


def emit_march(tag, traj):
    d(tag + "/n", len(traj))
    # A MARCH THAT RETURNED AN EMPTY LIST WOULD AGREE WITH ITSELF ON EVERY KEY BELOW, which is
    # § 5.27 (ii)'s emptied-sample defect. Refused here rather than folded over.
    assert traj, "an empty march is not a measurement: %s" % tag
    d(tag + "/keys", len(traj[0]))
    for i in range(0, len(traj), H_STRIDE):
        p, q = traj[i], "%s/p%d" % (tag, i)
        for k in H_FLOATS:
            walk("%s/%s" % (q, k), p.get(k))
        for k in H_INTS:
            walk("%s/%s" % (q, k), p.get(k))
        for k in H_STRS:
            walk("%s/%s" % (q, k), None if p.get(k) is None else str(p[k]))
    for k in H_FLOATS:
        col = [p[k] for p in traj if k in p]
        d("%s/col/%s/n" % (tag, k), len(col))
        if not col:
            continue
        f("%s/col/%s/min" % (tag, k), min(col))
        f("%s/col/%s/max" % (tag, k), max(col))
        f("%s/col/%s/last" % (tag, k), col[-1])
    # THE SIGN CENSUS rung 74 made necessary and rungs 75/76 inherit: these four are unfloored
    # projections and go NEGATIVE, which no aggregate in A-G records.
    for k in ("g_fuel", "g_gov", "required_fuel", "required_gov"):
        col = [p[k] for p in traj if k in p]
        d("%s/neg/%s" % (tag, k), sum(1 for x in col if x < 0.0))


for inc in (False, True):
    p = arms("H", inc)
    sm = PHI_BOTH / FLOOR - 1.0
    # RUNG 75's own cell — `windup_bill`'s, not the reduce cell, which IS rung 74.
    emit_march(p + "/windup",
               rig(AntiWindupTransient, PHI_BOTH, inc)._windup_march(
                   FLIGHT, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, inc,
                   "demand", "applied", "track", TAU_T)[3])
    # RUNG 76's, both cap laws — the pair whose DIFFERENCE is the rung.
    mc = rig(SensedCapTransient, PHI_BOTH, inc)
    acc = mc.accel_for(FLIGHT, LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc, MARGIN)
    for law in ("solve", "sensed"):
        emit_march("%s/cap_%s" % (p, law),
                   mc._cap_march(FLIGHT, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, inc,
                                 "demand", "sched", "none", None, law, acc)[3])

# =====================================================================================
# Z -- THE CENSUSES.  A `None` that is `None` on both sides agrees perfectly and measures
# nothing; a `+0.0` that should be `-0.0` is invisible to every relative bar; and a `nan`
# is unreachable on this plant TODAY, which is a fact about the plant and not a guarantee.
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
assert path, "usage: dump_slice_ag.py <out.tsv>"
with open(path, "w", encoding="utf-8", newline="\n") as fh:
    for k in sorted(seen):
        fh.write("%s\t%d\n" % (k, seen[k]))
sys.stderr.write("keys %d -> %s\n" % (len(seen), path))
sys.stderr.write("  none %d   neg_zero %d   pos_zero %d   nan %d\n"
                 % (N_NONE[0], N_NEG_ZERO[0], N_POS_ZERO[0], N_NAN[0]))
