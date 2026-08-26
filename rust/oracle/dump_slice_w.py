"""SLICE W step 4 -- **THE ORACLE** for rungs 62 + 63 (`BleedSchedule`, `ScheduledBleedTransient`),
over the TWO SUITES' OWN grids plus the three arms no suite reaches.

Step 3 ported 88 gates, one to one, and then measured what they can see. The answer is the
reason this file exists: **five of six injected defects pass all 88**, two of them moving 312
and 151 gate-visible readings. Every one of those gates is RELATIONAL -- it asserts a relation
among values THIS interpreter computed -- so an arithmetic divergence between Rust and Python
moves both sides of every relation and leaves all 88 green. **This file is the instrument that
establishes agreement with Python**, and both suite headers say so.

# THE GRIDS, WITH PROVENANCE -- DO NOT "FIX" ONE TO MATCH ANOTHER

Unlike slice V, the two suites here share a knee and differ only in the march step:

| section | `n_lo` | `ds` | provenance |
|---|---|---|---|
| A -- the schedule type      | 0.65 | --    | `test_rung62.py:57` |
| B -- `b_of` on a machine    | 0.65 | --    | `test_rung62.py:57` |
| C -- the forward closure    | 0.65 | --    | `test_rung62.py` / `test_rung63.py` |
| D -- rung 62's readers      | 0.65 | 0.01  | `test_rung62.py:55` |
| E -- rung 63's readers      | 0.65 | 0.005 | `test_rung63.py:45` |
| F -- `_isolating` / `_legs` | 0.65 | 0.01  | `test_rung62.py:55` |
| G -- the `at_stator` trap   | 0.65 | --    | `test_rung63.py`, gate 2 |
| **H -- the eight readers**  | 0.65 | 0.01 / 0.005 | **ADDED** -- see below |
| J -- the REDUCE (control)   | 0.65 | 0.01  | `test_rung62.py:55` |
| **K -- the dispatch census**| 0.65 | 0.02  | **ADDED** -- s 5.21 (v)'s `probe_w3.py` workload |

**`N_LO` IS 0.65 AND THAT IS NOT RUNG 57's 0.75574.** `test_rung62.py:57` says why in its own
comment: 0.75574 leaves the bleed CLIPPED at `b_max`, where `db/dn = 0` and there is no loop to
measure. One `sat` arming at 0.75574 is carried in section D and **labelled ADDED**, because the
saturated corner is a real state of the schedule and no suite exercises it.

# `_cpg` IS COPIED CHARACTER FOR CHARACTER, AND THAT IS LOAD-BEARING

`R_c` is DERIVED as `(gamma_c - 1.0)/gamma_c * cp_c`. `1.4 - 1.0` is `0.3999999999999999` in
IEEE-754, so re-spelling it `0.4/1.4` builds a gas one ULP away and drifts EVERY number in this
file. Step 2's first smoke run did exactly that and failed 243 of 522 keys.

# THE THREE **ADDED** SECTIONS -- LABELLED, SO A SUPERSET CANNOT PASS AS A PORT

* **H -- THE EIGHT INHERITED `at_stator` READERS.** s 5.21 (ii) measured that ONLY ONE of them
  (`schedule_invariance`) is called anywhere in either suite; the other seven are 0 and 0. Their
  rung-62 behaviour is UNGATED in Python, and this is the section that gates it. Two things make
  the obvious version of it blind and are done differently here:

    - **SIX OF THE EIGHT REFUSE A BLEED-ONLY MACHINE.** `credit_decomposition`,
      `composite_credit`, `engagement_shift`, `matched_credit`, `set_point_bands` and
      `floor_composite` all assert `_is_armed() or vsv_lp or vsv_hp` -- a STATOR arming. So the
      section runs them on a machine carrying BOTH devices, and records the six refusals on a
      bleed-only machine as their own keys. Running them bare would have emitted nothing and
      looked like coverage.
    - **THREE OF THEM PASS STATOR ARGUMENTS TO `at_stator` INTERNALLY** (`credit_decomposition`'s
      `v_at_min` sibling, `engagement_shift`'s keyword sweep, `set_point_bands`' ladder), so
      rung 62's cell must carry `self`'s VALVE while honouring the PASSED setting. Get that
      wrong for the arg-passing readers and the one gated reader still passes. `H/at_stator/*`
      dumps the five argument shapes directly beside them.

* **K -- THE DISPATCH CENSUS.** Integer counts, not values: s 5.21 (v)'s four reduced/bled pairs
  plus `b_of`'s CALL COUNT and its three classifications. Step 3 finding 3 measured that a
  `_powers` "simplified" to re-read `b_of` moves the CALL COUNT and leaves all eight pairs
  untouched, so the pairs alone are the instrument that measured nothing. Both are here, and
  `slice_w_dispatch.rs` reads them from this golden BY KEY rather than from a typed literal.

  **WHAT IS NOT IN IT, AND WHY.** The Rust `Census` also carries `close_bracket_fails`,
  `close_fuel_bracket_fails`, `march_in_advances`, `fuel_march_in_advances`, `lo_floor_hits`
  and the three `hi_wall_*` counters. Those tick INSIDE rung 62's bled bodies, at points with no
  Python method boundary to wrap, so counting them here would mean instrumenting the Python
  source -- which this dump must not do. Their effect is carried by value keys (the bracket
  walls decide `m_lp`), and they are named here rather than silently omitted.

* **D's `sat` ARMING** -- the saturated knee, above.

# THE CONDITIONAL BLOCKS EMIT A PRESENCE FLAG

`leg_retiming` returns `audits` only when an `accel` leg is passed, and `channels` only then
too. A block that BOTH sides skip is an agreement on ABSENCE, which is exactly the shape an
oracle cannot see. Every such block emits `<path>/*_present` first, so a silently skipped reader
lands in the Rust's `bad` list as NO GOLDEN.

# THE CPython ARM HAS NO TOLERANCE TIER

Every cell in this file is **CPG** -- both suites' `_cpg()`. A float drifting between
interpreters is therefore a DEFECT, not content, and `slice_w_oracle.rs` panics on it.

Regenerate the goldens with:
    .venv\\Scripts\\python.exe rust\\oracle\\dump_slice_w.py > rust\\oracle\\slice_w_pypy.tsv
    C:\\Python314\\python.exe  rust\\oracle\\dump_slice_w.py > rust\\oracle\\slice_w_cpython.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from turbojet.gas import Gas                                                      # noqa: E402
from turbojet.engine import (                                                     # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap, ScheduledStatorTransient,
    StatorSchedule, ScheduledBleedTransient, BleedSchedule, SurgeLimiter, IncidenceLimiter,
)

OUT = []


NAN_BITS = 0x7FF8000000000000


def f(key, x):
    """A float, as its IEEE-754 bit pattern -- **except a NaN, which is CANONICALISED.**

    66 keys in this file are legitimately NaN: `s_eng` is `nan` by construction wherever a leg
    never crosses (Python says so at `floor_dichotomy`), and `erosion` is `0/0` on an HP spool
    the LP lever does not reach. A NaN's bit pattern is NOT portable -- CPython's `float('nan')`
    is the positive quiet NaN while `0.0/0.0` on x86-64 unwinds NEGATIVE -- so comparing the
    raw bits would make the oracle fail on the SIGN OF A NAN, which carries no meaning. Both
    sides canonicalise, and the comparison then says what it means: *both are NaN*."""
    v = float(x)
    OUT.append((key, NAN_BITS if v != v
                else struct.unpack("<Q", struct.pack("<d", v))[0]))


def d(key, n):
    """An integer, as an unsigned 64-bit word.

    `sign_bleed` / `sign_stator` are **-1 or +1**, and the whole file is `key<TAB>u64`, so a
    signed value is written in two's complement -- which is exactly what the Rust's `as u64`
    on an `i32` produces. Masking rather than widening the format keeps every line parseable
    by one `parse::<u64>()`."""
    OUT.append((key, int(n) & 0xFFFFFFFFFFFFFFFF))


def b(key, flag):
    OUT.append((key, 1 if flag else 0))


def tag(key):
    """A key whose PRESENCE is the value -- a discrete label (`lever/bleed`, `leg/accel`).
    Asking the golden for a label the run did not take is a missing-key failure, which IS the
    assertion; no string ever has to be compared."""
    OUT.append((key, 1))



def guard_armed(m):
    """Python's COMPOSITE guard `self._is_armed() or self.vsv_lp or self.vsv_hp` -- the
    expression six of section H's eight readers open with, and **a DIFFERENT PREDICATE from
    `_is_armed()` itself**, which is scheduled-only.

    They are dumped as two keys because they differ on exactly one input shape -- a CONSTANT
    stator with no schedule -- and that shape appears in this file only in section H (b)'s
    argument sweep. Emitting one key for both is how the first writing of this file compared
    Python's scheduled-only method against the Rust's composite `is_armed()` and read a NAMING
    difference as a port defect: 4 keys of 100-odd flipped, and the other ~100 agreed because
    the two predicates coincide everywhere else."""
    return bool(m._is_armed() or m.vsv_lp or m.vsv_hp)


# ============================================================================== the shared grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, SETTLE = 1000.0, 1400.0, 1.2
# BOTH suites' knee. **NOT rung 57's 0.75574** -- see the header.
N_LO, V, B, MARGIN = 0.65, 0.20, 0.10, 0.25
N_LO_SAT = 0.75574          # ADDED: rung 57's knee, where the bleed clips at b_max
# Section H's own margin.  `matched_credit` REFUSES the both-scheduled machine at 0.25 -- its
# rung-59 clamp audit finds the schedule consulted outside the derived bracket at 3 of 210
# cutting points.  That refusal is recorded as its own key (H/clamp_refusal) rather than
# tuned away, and the reader is then RUN at 0.40, where the bracket contains the march.
MARGIN_H = 0.40
DS_62 = 0.01                # `test_rung62.py:55`
DS_63 = 0.005               # `test_rung63.py:45`
DS_CENSUS = 0.02            # ADDED: s 5.21 (v)'s `probe_w3.py` workload
RATES = (0.10, 0.25, 0.50, 1.00, 2.00)
SM_GRID = (0.34, 0.36, 0.40, 0.43, 0.46)

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
TILT_LP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
TILT_HP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """BOTH suites' `_cpg`, character for character. `R_c` is DERIVED; `0.4/1.4` is a
    DIFFERENT NUMBER and it drifts every key in this file (step 2, finding 1)."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def bt(lp=LP, hp=HP, **kw):
    return ScheduledBleedTransient(DESIGN, FLIGHT, 1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


BLEED_KW = dict(bleed_sched=BleedSchedule(B, N_LO))
STAT_KW = dict(vsv_sched_lp=StatorSchedule(V, N_LO))
CONST_KW = dict(bleed=B)
BOTH_KW = {**BLEED_KW, **STAT_KW}
BOTH_CONST_KW = dict(bleed=B, vsv_lp=V)

# ---------------------------------------------------------------------------- the put_* helpers
PT_KEYS = ("s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp",
           "mdot_air", "sp_thrust", "mf", "mf_sched")
CELL_KEYS = ("m_i", "m_i_grid", "m_phi", "s", "v", "s_grid", "min_phi", "nu0", "nu_lp_end",
             "nu_hp_end", "Tt4_peak", "fuel_removed", "s_eng")
CLOSE_KEYS = ("m_lp", "m_imp", "m_hp", "phi_lp", "phi_hp", "n_lp", "n_hp", "tau_lpc",
              "tau_hpc", "Tt25", "Tt3", "pi_lpc", "pi_hpc", "pt4", "f", "eta_lpc", "eta_hpc",
              "mdot_air", "mdot4")
EQ_KEYS = ("Tt2", "Tt25", "Tt3", "Tt4", "Tt45", "Tt5", "f", "mdot_air", "mdot4", "nu_lp",
           "nu_hp", "n_lp", "n_hp", "phi_lp", "phi_hp", "pi_lpc", "pi_hpc", "pi_hpt", "pi_lpt",
           "slip", "sp_thrust", "pt4", "M9", "eta_lpc", "eta_hpc", "m_lp", "m_hp")
TAIL_KEYS = ("Phi_lp", "Phi_hp", "Tt45", "Tt5", "tau_hpt", "tau_lpt", "pi_hpt", "pi_lpt",
             "eta_hpt", "eta_lpt", "nu_hpt", "nu_lpt", "sp_thrust", "M9", "slip")
CHAIN_KEYS = ("Tt4", "d_Tt25", "d_Tt3", "d_f", "d_mfp", "d_ratio", "d_kappa", "d_n_hp",
              "d_nu_lp")


def put_traj(p, traj, stride):
    d(f"{p}/npts", len(traj))
    for i in range(0, len(traj), stride):
        for k in PT_KEYS:
            f(f"{p}/{i}/{k}", traj[i][k])


def put_read(p, rd):
    d(f"{p}/npts", rd["npts"])
    for sp in ("lp", "hp"):
        r = rd[sp]
        for k in ("m_phi", "m_i", "T_c", "min_phi"):
            f(f"{p}/{sp}/{k}", r[k])
        for k in ("s", "phi", "v", "nu_lp", "nu_hp"):
            f(f"{p}/{sp}/at/{k}", r["at"][k])


def put_cell(p, c):
    for k in CELL_KEYS:
        f(f"{p}/{k}", c[k])
    d(f"{p}/npts", c["npts"])
    d(f"{p}/prof_len", len(c["prof"]))
    for i in (0, len(c["prof"]) // 2, len(c["prof"]) - 1):
        f(f"{p}/prof/{i}/s", c["prof"][i][0])
        f(f"{p}/prof/{i}/m", c["prof"][i][1])


def put_accel(p, a):
    d(f"{p}/n", len(a.n_H))
    f(f"{p}/margin", a.margin)
    for i in range(len(a.n_H)):
        f(f"{p}/n_H/{i}", a.n_H[i])
        f(f"{p}/kappa/{i}", a.kappa[i])


def put_audit(p, au):
    for k in ("lo", "hi", "n_min", "n_max", "cut_lo", "cut_hi"):
        f(f"{p}/{k}", au[k])
    d(f"{p}/n_cuts", au["n_cuts"])
    d(f"{p}/clamped", au["clamped"])


def put_pin(p, au):
    for k in ("m_set", "m_min", "residual", "s_eng", "removed"):
        f(f"{p}/{k}", au[k])
    for k in ("pinned", "dormant", "from_zero", "admissible"):
        b(f"{p}/{k}", au[k])


def put_chain(p, chain):
    d(f"{p}/n", len(chain))
    for i, row in enumerate(chain):
        for k in CHAIN_KEYS:
            f(f"{p}/{i}/{k}", row[k])


def put_legs(p, r):
    for k in ("reference", "start", "ramp", "full", "self_cancel", "surrendered",
              "share_start", "loop", "nu0_ref", "nu0_armed", "cmd_ramp", "cmd_full",
              "s_ref", "s_ramp", "s_full", "r"):
        f(f"{p}/{k}", r[k])
    tag(f"{p}/lever/{r['lever']}")
    tag(f"{p}/spool/{r['spool']}")


# =============================================================================================
# A -- THE SCHEDULE TYPE.  The deliberate TWIN of rung 57's `StatorSchedule`: same functional
# form, same two shapes, same corner assert.  It is NOT factored against it -- the rung compares
# two DEVICES, and one generic `Schedule` with a shape enum would make it compare two spellings.
# =============================================================================================
for shape in ("smooth", "linear"):
    s = BleedSchedule(B, N_LO, shape=shape)
    f(f"A/{shape}/b_max", s.b_max)
    f(f"A/{shape}/n_ref", s.n_ref)
    for n in (0.30, 0.40, 0.50, 0.60, N_LO, 0.68, 0.70, 0.75, 0.80, 0.90, 0.95, 0.999,
              1.0, 1.05, 1.30):
        f(f"A/{shape}/b_of_n/{n:.3f}", s(n))
# The corner `__post_init__` pins, and the degenerate schedule the reduce gate builds.
f("A/corner/exact_zero_at_one", BleedSchedule(B, N_LO)(1.0))
f("A/corner/exact_zero_above", BleedSchedule(B, N_LO)(1.4))
f("A/corner/at_knee", BleedSchedule(B, N_LO)(N_LO))
f("A/bmax0/at_lo", BleedSchedule(0.0, N_LO)(N_LO))
f("A/bmax0/at_zero", BleedSchedule(0.0, N_LO)(0.0))
# ADDED: the SATURATED knee -- rung 57's 0.75574, where the schedule clips at b_max over the
# whole band the suites march.  `test_rung62.py:57` names this as the artifact it moved off.
for n in (0.60, 0.70, N_LO_SAT, 0.80, 0.90):
    f(f"A/sat/b_of_n/{n:.5f}", BleedSchedule(B, N_LO_SAT)(n))

# =============================================================================================
# B -- `b_of` AND `_armed_bleed` ON A MACHINE.  The three legs (constant / scheduled / bare) and
# the `Tt2` REFERRAL the schedule reads through -- `b_of` corrects `nu_lp` to a PHYSICAL speed.
# =============================================================================================
for tg, kw in (("const", CONST_KW), ("sched", BLEED_KW), ("bare", {}),
               ("bmax0", dict(bleed_sched=BleedSchedule(0.0, N_LO))),
               ("both", BOTH_KW)):
    m = bt(**kw)
    b(f"B/{tg}/armed", m._armed_bleed())
    b(f"B/{tg}/is_scheduled_stator", m._is_armed())
    b(f"B/{tg}/guard_armed_stator", guard_armed(m))
    f(f"B/{tg}/Tt2_d", m.Tt2_d)
    for nu in (0.50, 0.60, 0.70, 0.75, 0.80, 0.90, 1.00, 1.10):
        f(f"B/{tg}/b_of_design_Tt2/{nu:.2f}", m.b_of(nu))
        f(f"B/{tg}/b_of_Tt2_280/{nu:.2f}", m.b_of(nu, 280.0))
        f(f"B/{tg}/b_of_Tt2_240/{nu:.2f}", m.b_of(nu, 240.0))

# =============================================================================================
# C -- THE FORWARD CLOSURE.  `_close`, `_powers`, `_instant_tail`, `_instant_fuel`, `_close_fuel`
# and `equilibrium`, on every arming the two suites build and on both map shapes.
#
# `mdot_face` is the key the trial-vs-imposed shadowing would move and it reaches the answer
# ONLY through `_powers` -- Python's `_close` binds a LOCAL `mdot_face` (the m_lp-derived TRIAL
# face flow) and returns a dict key of the same name holding `mdot_imp/(1-b)` (the IMPOSED one).
# They agree only AT the root, so a converged closure hides the swap.
# =============================================================================================
for shape, (lp, hp) in (("shaped", (LP, HP)), ("tilted", (TILT_LP, TILT_HP))):
    for tg, kw in (("bare", {}), ("const010", dict(bleed=0.10)), ("const030", dict(bleed=0.30)),
                   ("sched", BLEED_KW), ("stat", STAT_KW), ("both", BOTH_KW)):
        m = bt(lp, hp, **kw)
        Tt2, pt2, V0 = m._inlet(FLIGHT)
        f(f"C/{shape}/{tg}/inlet/Tt2", Tt2)
        f(f"C/{shape}/{tg}/inlet/pt2", pt2)
        f(f"C/{shape}/{tg}/inlet/V0", V0)
        for Tt4 in (1000.0, 1200.0, 1500.0):
            e = m.equilibrium(FLIGHT, Tt4)
            for k in EQ_KEYS:
                f(f"C/{shape}/{tg}/eq/{Tt4:.0f}/{k}", e[k])
            tag(f"C/{shape}/{tg}/eq/{Tt4:.0f}/branch/{e['branch']}")
            # THE ABSENT KEY ITSELF -- the thing s 5.21 (v) says no VALUE key can see. Rung
            # 40's and rung 57's closures return a dict with NO `bleed` key at all; the Rust
            # spells it `Option<f64>`, and `.unwrap_or(0.0)` IS `.get(_, 0.0)`.
            b(f"C/{shape}/{tg}/eq/{Tt4:.0f}/has_bleed_key", "bleed" in e)
            b(f"C/{shape}/{tg}/eq/{Tt4:.0f}/has_mdot_face_key", "mdot_face" in e)
            if "bleed" in e:
                f(f"C/{shape}/{tg}/eq/{Tt4:.0f}/bleed", e["bleed"])
                f(f"C/{shape}/{tg}/eq/{Tt4:.0f}/mdot_face", e["mdot_face"])
        for (nu_lp, nu_hp, Tt4) in ((0.80, 0.85, 1200.0), (0.85, 0.88, 1200.0),
                                    (0.95, 0.97, 1400.0)):
            p = f"C/{shape}/{tg}/close/{nu_lp:.2f}_{Tt4:.0f}"
            c = m._close(nu_lp, nu_hp, Tt4, Tt2, pt2)
            for k in CLOSE_KEYS:
                f(f"{p}/{k}", c[k])
            b(f"{p}/has_bleed_key", "bleed" in c)
            b(f"{p}/has_mdot_face_key", "mdot_face" in c)
            if "bleed" in c:
                f(f"{p}/bleed", c["bleed"])
                f(f"{p}/mdot_face", c["mdot_face"])
            # `_powers` -- the site rung 40 pulled out of the tail for the Newton's inner loop.
            # PRESENCE first: an aborting closure and a skipped block read alike.
            b(f"{p}/powers_present", True)
            p_lp, p_hp = m._powers(c, FLIGHT, nu_lp, nu_hp, Tt4)
            f(f"{p}/powers/Phi_lp", p_lp)
            f(f"{p}/powers/Phi_hp", p_hp)
            t = m._instant_tail(FLIGHT, c, nu_lp, nu_hp, Tt4, V0)
            for k in TAIL_KEYS:
                f(f"{p}/tail/{k}", t[k])
            tag(f"{p}/tail/branch/{t['branch']}")
            b(f"{p}/tail/has_sp_thrust_inlet", "sp_thrust_inlet" in t)
            if "sp_thrust_inlet" in t:
                f(f"{p}/tail/sp_thrust_inlet", t["sp_thrust_inlet"])
            # and the two split sites must AGREE -- rung 62's own gate-2 witness.
            inst = m._instant(FLIGHT, nu_lp, nu_hp, Tt4)
            b(f"{p}/powers_match_tail", p_lp == inst["Phi_lp"] and p_hp == inst["Phi_hp"])

# --- the FUEL closure, with its own bracket.  b = 0.30 is where the walls' 1/(1-b) is what
# --- keeps the scan OUTSIDE the physical root.
for tg, kw in (("bare", {}), ("b010", dict(bleed=0.10)), ("b030", dict(bleed=0.30)),
               ("sched", BLEED_KW)):
    m = bt(**kw)
    Tt2, pt2, _ = m._inlet(FLIGHT)
    for Tt4 in (1000.0, 1200.0):
        mf = m.fuel_for_Tt4(FLIGHT, Tt4)
        f(f"C/fuel/{tg}/{Tt4:.0f}/mf", mf)
        eq = m.equilibrium(FLIGHT, Tt4)
        c = m._close_fuel(eq["nu_lp"], eq["nu_hp"], mf, Tt2, pt2)
        for k in CLOSE_KEYS + ("Tt4",):
            f(f"C/fuel/{tg}/{Tt4:.0f}/{k}", c[k])
        # **THE TWO KEYS ARE INDEPENDENT, AND THAT IS THE POINT.** Rung 40's `_close_fuel`
        # ALREADY returns `mdot_air_face` (its own trial face flow); only rung 62's adds
        # `bleed`. So an `Option` on one says nothing about the other, and a dump that
        # guarded both on one flag would never read the bled `bleed` at all.
        b(f"C/fuel/{tg}/{Tt4:.0f}/has_bleed_key", "bleed" in c)
        b(f"C/fuel/{tg}/{Tt4:.0f}/has_face_key", "mdot_air_face" in c)
        if "mdot_air_face" in c:
            f(f"C/fuel/{tg}/{Tt4:.0f}/mdot_air_face", c["mdot_air_face"])
        if "bleed" in c:
            f(f"C/fuel/{tg}/{Tt4:.0f}/bleed", c["bleed"])
            f(f"C/fuel/{tg}/{Tt4:.0f}/mdot_face", c["mdot_face"])
        i = m._instant_fuel(FLIGHT, 0.85, 0.88, mf)
        for k in ("Tt4", "Phi_lp", "Phi_hp", "sp_thrust", "f", "n_hp", "pt4", "mdot_air"):
            f(f"C/fuel/{tg}/{Tt4:.0f}/instant/{k}", i[k])

# =============================================================================================
# D -- RUNG 62's READERS, on `test_rung62.py`'s grid (ds = 0.01).
# =============================================================================================
# --- the two loop-gain FACTORS the headline's SIGN argument rests on
for row in bt().loop_factors(FLIGHT, (900.0, 1100.0, 1300.0, 1500.0), db=0.10, dv=0.20):
    p = f"D/lf/{row['Tt4']:.0f}"
    for k in ("Tt4", "n_bare", "dn_db", "dn_dv"):
        f(f"{p}/{k}", row[k])
    d(f"{p}/sign_bleed", row["sign_bleed"])
    d(f"{p}/sign_stator", row["sign_stator"])

# --- THE HEADLINE: `loop_decomposition` on an ARMED machine.  Its reference is `at_lever()`,
# --- NOT `_isolating()` -- a different path from `marginal_loop` below.
for ln, kw in (("bled", BLEED_KW), ("stat", STAT_KW), ("const", CONST_KW), ("both", BOTH_KW),
               # ADDED -- the SATURATED schedule at rung 57's knee; no suite arms it.
               #
               # **AND IT IS NOT DEGENERATE, WHICH WAS MEASURED RATHER THAN ASSUMED.**  The
               # obvious worry about this arming is that it reduces to the CONSTANT one -- at
               # 0.75574 the bleed machine idles at 0.737, BELOW the knee, so the schedule
               # starts the ramp clipped at `b_max` with `db/dn = 0` and no loop to close.
               # Diffed key for key against `D/ld/const/*`: they agree on 21 of 54 keys and
               # those 21 are the LEVER-INDEPENDENT ones (`reference`, `nu0_ref`, `s_ref`, `r`
               # and the two labels).  Every rung-62 headline quantity differs, because the
               # ramp carries `n` back OVER the knee and the schedule comes off the clip
               # mid-march.  `self_cancel` lands strictly BETWEEN the two legs at all three
               # rates -- const 1.0205 / 1.0289 / 1.0349, sat 1.0480 / 1.0417 / 1.0378, bled
               # 1.0990 / 1.0971 / 1.0930 -- so the loop-attributable part (sat - const against
               # bled - const) is 35 % / 19 % / 5 %.  `test_rung62.py:57`'s reason for moving
               # the knee to 0.65 is CONFIRMED and given a number: the placement ATTENUATES the
               # loop rather than removing it, and it attenuates it more the faster the ramp.
               ("sat", dict(bleed_sched=BleedSchedule(B, N_LO_SAT)))):
    for r in (0.25, 0.50, 1.00):
        put_legs(f"D/ld/{ln}/{r:.2f}",
                 bt(**kw).loop_decomposition(FLIGHT, LO, HI, r=r, s_settle=SETTLE, ds=DS_62))

# --- the MARGINAL loop: one lever's own loop with a NEIGHBOUR carried on both sides
for ln, lever, nb in (("bled", BLEED_KW, None), ("stat", STAT_KW, None),
                      ("const", CONST_KW, None),
                      ("bled_nb_stat", BLEED_KW, STAT_KW),
                      ("stat_nb_bled", STAT_KW, BLEED_KW),
                      ("stat_nb_const", STAT_KW, CONST_KW)):
    for r in (0.25, 0.50, 1.00):
        put_legs(f"D/ml/{ln}/{r:.2f}",
                 bt().marginal_loop(FLIGHT, LO, HI, lever, neighbour=nb, r=r,
                                    s_settle=SETTLE, ds=DS_62))

# --- what the schedule actually COMMANDS over the ramp (the level a constant leg must match)
for ln, kw in (("bled", BLEED_KW), ("stat", STAT_KW), ("const", CONST_KW)):
    for r in (0.25, 0.50, 1.00):
        c = bt(**kw).commanded_level(FLIGHT, LO, HI, r=r, s_settle=SETTLE, ds=DS_62)
        p = f"D/cl/{ln}/{r:.2f}"
        for k in ("at_min", "mean", "peak", "s_min"):
            f(f"{p}/{k}", c[k])
        tag(f"{p}/lever/{c['lever']}")

# --- the PAIR: four cells, credit AND cost, in both currencies
for r in (0.25, 0.50, 1.00):
    pi = bt().pair_interaction(FLIGHT, LO, HI, STAT_KW, BLEED_KW, r=r, s_settle=SETTLE,
                               ds=DS_62)
    p = f"D/pi/{r:.2f}"
    for k in ("credit_a", "credit_b", "credit_pair", "credit_sum", "interaction",
              "interaction_frac", "cost_a", "cost_b", "cost_pair", "cost_interaction", "r"):
        f(f"{p}/{k}", pi[k])
    tag(f"{p}/spool/{pi['spool']}")

# --- the ramp-rate CONTROL: credit per unit CONSTANT setting against ramp rate
for ln, lever, setting in (("bleed", CONST_KW, B), ("stat", dict(vsv_lp=V), V)):
    for row in bt().clock_sweep(FLIGHT, LO, HI, lever, setting, rates=RATES,
                                s_settle=SETTLE, ds=DS_62):
        p = f"D/cs/{ln}/{row['r']:.2f}"
        for k in ("r", "bare", "credit", "per_setting"):
            f(f"{p}/{k}", row[k])

# =============================================================================================
# E -- RUNG 63's READERS, on `test_rung63.py`'s grid (ds = 0.005).
# =============================================================================================
_M63 = bt()
LEG = _M63.accel_schedule(FLIGHT, LO, HI, MARGIN, 13)
put_accel("E/leg", LEG)

for ln, lever, nb in (("bled", BLEED_KW, None), ("stat", STAT_KW, None),
                      ("bled_nb_stat", BLEED_KW, STAT_KW)):
    # --- THE RUNG: does the lever RE-TIME the fuel leg?
    for r in (0.25, 0.50, 1.00):
        lr = _M63.leg_retiming(FLIGHT, LO, HI, lever, accel=LEG, r=r, s_settle=SETTLE,
                               ds=DS_63, neighbour=nb)
        p = f"E/lr/{ln}/{r:.2f}"
        for k in ("ref_limited", "ref_dormant", "armed_limited", "armed_dormant",
                  "d_limited", "d_dormant", "rel_limited", "rel_dormant", "r", "ds"):
            f(f"{p}/{k}", lr[k])
        tag(f"{p}/leg/{lr['leg']}")
        # PRESENCE FIRST -- `audits` and `channels` exist only when an `accel` leg is passed,
        # and a block BOTH sides skip is an agreement on ABSENCE, not on a value.
        b(f"{p}/audits_present", bool(lr["audits"]))
        for side in ("ref", "armed"):
            put_audit(f"{p}/audit/{side}", lr["audits"][side])
        b(f"{p}/channels_present", lr["channels"] is not None)
        ch = lr["channels"]
        for k in ("s_at", "d_kappa", "d_pt3", "d_cap", "d_mf_sched", "d_g"):
            f(f"{p}/ch/{k}", ch[k])
        for side in ("ref", "armed"):
            for k in ("s", "n_hp", "pt3", "cap", "kappa", "mf_sched", "g"):
                f(f"{p}/ch/{side}/{k}", ch[side][k])
    # --- the NO-ACCEL control: a `Tt4_max` leg, so `audits`/`channels` are ABSENT on both sides
    lr0 = _M63.leg_retiming(FLIGHT, LO, HI, lever, Tt4_max=1350.0, r=0.5, s_settle=SETTLE,
                            ds=DS_63, neighbour=nb)
    p = f"E/lr0/{ln}"
    for k in ("ref_limited", "ref_dormant", "armed_limited", "armed_dormant",
              "d_limited", "d_dormant", "rel_limited", "rel_dormant"):
        f(f"{p}/{k}", lr0[k])
    tag(f"{p}/leg/{lr0['leg']}")
    b(f"{p}/audits_present", bool(lr0["audits"]))
    b(f"{p}/channels_present", lr0["channels"] is not None)

    # --- THE MECHANISM: the leg's two SENSED INPUTS, with a GENUINELY BARE reference
    si = _M63.sensed_inputs(FLIGHT, LO, HI, lever, margin=MARGIN, n=13, neighbour=nb)
    p = f"E/si/{ln}"
    for k in ("d_ordinate", "d_abscissa", "signed_ordinate", "signed_abscissa", "d_mfp"):
        f(f"{p}/{k}", si[k])
    b(f"{p}/ordinate_identical", si["ordinate_identical"])
    b(f"{p}/abscissa_identical", si["abscissa_identical"])
    put_accel(f"{p}/reference", si["reference"])
    put_accel(f"{p}/armed", si["armed"])
    put_chain(f"{p}/chain", si["chain"])

    # --- rung 59's SPLICE, for a lever that moves BOTH halves of the table
    md = _M63.matched_leg_deltas(FLIGHT, LO, HI, lever, margin=MARGIN, r=0.5,
                                 s_settle=SETTLE, ds=DS_63, n=13, neighbour=nb)
    p = f"E/md/{ln}"
    for k in ("delta_match", "delta_index", "delta_value", "margin", "r", "ds"):
        f(f"{p}/{k}", md[k])
    d(f"{p}/clamped", md["clamped"])
    for c in ("bare_leg", "matched", "reindexed", "revalued"):
        put_cell(f"{p}/{c}", md["cells"][c])
        put_audit(f"{p}/audit/{c}", md["audits"][c])

    # --- the FORWARD arrow: rung 58's mixed second difference on `at_lever` siblings
    for r in (0.25, 0.50, 1.00):
        lc = _M63.lever_composite(FLIGHT, LO, HI, lever, accel=LEG, r=r, s_settle=SETTLE,
                                  ds=DS_63, neighbour=nb)
        p = f"E/lc/{ln}/{r:.2f}"
        for k in ("credit_bare", "credit_fuel", "interaction", "share", "predicted",
                  "profile_bare", "profile_fuel", "recovered", "relocation",
                  "relocation_bare", "removed_bare", "removed_armed", "r", "ds"):
            f(f"{p}/{k}", lc[k])
        tag(f"{p}/leg/{lc['leg']}")
        for c in ("neither", "lever", "fuel", "both"):
            put_cell(f"{p}/{c}", lc["cells"][c])

    # --- THE `_surge_fuel` PATH: a phi FLOOR beside the valve, swept over the set point.
    # --- This is the ONLY rung-63 reader that reaches `_surge_fuel`, which is what a
    # --- `..R43`-vs-`..R57_FUEL` table spread decides.
    fd = _M63.floor_dichotomy(FLIGHT, LO, HI, lever, SM_GRID, spool="lp", r=0.5,
                              s_settle=SETTLE, ds=DS_63, neighbour=nb)
    p = f"E/fd/{ln}"
    for k in ("phi_surge", "min_phi_ref", "min_phi_armed", "r", "ds"):
        f(f"{p}/{k}", fd[k])
    f(f"{p}/band_lo", fd["band"][0])
    f(f"{p}/band_hi", fd["band"][1])
    d(f"{p}/rows", len(fd["rows"]))
    for i, row in enumerate(fd["rows"]):
        # THE ROWS ARE THE FLOOR-ARMED CELLS.  `min_phi_ref`/`min_phi_armed` above come from
        # the leg-FREE cells and can never see a leg; `row["sm"]` is the input grid echoed
        # back.  Dumping only those would be structurally blind to every `_surge_fuel` defect
        # while LOOKING like coverage of one -- step 3's finding 5, item 2.
        for k in ("sm", "phi_lim", "m_i_fuel", "m_i_both", "min_phi_fuel", "min_phi_both",
                  "removed_fuel", "removed_both", "credit"):
            f(f"{p}/row{i}/{k}", row[k])
        b(f"{p}/row{i}/disarmed", row["disarmed"])

# =============================================================================================
# F -- `_isolating` AND `_legs`.  The sibling PAIR every rung-63 reader is built on, and the
# generalised START / RAMP / FULL the rung's headline reads.
# =============================================================================================
for tg, lever, nb in (("plain", BLEED_KW, None),
                      ("nb_stat", BLEED_KW, STAT_KW),
                      ("stat_nb_bled", STAT_KW, BLEED_KW),
                      ("nb_const", STAT_KW, CONST_KW)):
    ref, armed = bt()._isolating(lever, neighbour=nb)
    p = f"F/iso/{tg}"
    b(f"{p}/ref_armed_bleed", ref._armed_bleed())
    b(f"{p}/armed_armed_bleed", armed._armed_bleed())
    b(f"{p}/ref_is_scheduled_stator", ref._is_armed())
    b(f"{p}/armed_is_scheduled_stator", armed._is_armed())
    b(f"{p}/ref_guard_armed_stator", guard_armed(ref))
    b(f"{p}/armed_guard_armed_stator", guard_armed(armed))
    f(f"{p}/ref_b_of_080", ref.b_of(0.80))
    f(f"{p}/armed_b_of_080", armed.b_of(0.80))
    f(f"{p}/ref_vsv_lp", ref.vsv_lp)
    f(f"{p}/armed_vsv_lp", armed.vsv_lp)

# `_legs` called directly with the reference NAMED -- the shape `marginal_loop` builds and
# `loop_decomposition` does not.
_ref, _armed = bt()._isolating(BLEED_KW, neighbour=STAT_KW)
put_legs("F/legs/bled_nb_stat",
         _armed._legs(FLIGHT, _ref, LO, HI, 0.5, SETTLE, DS_62, "lp"))
put_legs("F/legs/bled_nb_stat_hp",
         _armed._legs(FLIGHT, _ref, LO, HI, 0.5, SETTLE, DS_62, "hp"))
# and with a LEG threaded through all four marches -- rung 63's extension of it
put_legs("F/legs/with_accel",
         _armed._legs(FLIGHT, _ref, LO, HI, 0.5, SETTLE, DS_63, "lp", accel=LEG))

# =============================================================================================
# G -- THE `at_stator` TRAP.  s 5.21 (ii): rung 62 overrode `at_stator` so a rung-57 reader on a
# bleed-armed machine differences against a sibling CARRYING THIS MACHINE'S VALVE, which makes
# rung 59's `schedule_invariance` compare the plant with ITSELF and report rung 59's exact
# headline WHILE MEASURING NOTHING.  The gate exists to pin that counterfeit.  Left as rung
# 57's, the two identities read False/False at 9.543e-3 and 1.019e-2.
# =============================================================================================
for tg, kw in (("sched", BLEED_KW), ("const", CONST_KW), ("both", BOTH_KW)):
    m = bt(**kw)
    sib = m.at_stator()
    p = f"G/trap/{tg}"
    b(f"{p}/sibling_armed_bleed", sib._armed_bleed())
    b(f"{p}/sibling_is_scheduled", sib.bleed_sched is not None)
    b(f"{p}/sibling_is_scheduled_stator", sib._is_armed())
    b(f"{p}/sibling_guard_armed_stator", guard_armed(sib))
    f(f"{p}/sibling_bleed", sib.bleed)
    f(f"{p}/sibling_b_of_080", sib.b_of(0.80))
    inv = m.schedule_invariance(FLIGHT, LO, HI, MARGIN, 13)
    b(f"{p}/ordinate_identical", inv["ordinate_identical"])
    b(f"{p}/abscissa_identical", inv["abscissa_identical"])
    f(f"{p}/d_ordinate", inv["d_ordinate"])
    f(f"{p}/d_abscissa", inv["d_abscissa"])
    put_accel(f"{p}/inv_bare", inv["bare"])
    put_accel(f"{p}/inv_matched", inv["matched"])
    put_chain(f"{p}/inv_chain", inv["chain"])
# The HONEST reader beside it: rung 63's `sensed_inputs` differences against a valve-SHUT
# sibling and reports the two numbers the counterfeit hides.
hon = bt().sensed_inputs(FLIGHT, LO, HI, BLEED_KW, margin=MARGIN, n=13)
for k in ("d_ordinate", "d_abscissa", "signed_ordinate", "signed_abscissa", "d_mfp"):
    f(f"G/honest/{k}", hon[k])
b("G/honest/ordinate_identical", hon["ordinate_identical"])
b("G/honest/abscissa_identical", hon["abscissa_identical"])

# =============================================================================================
# H -- **ADDED**: THE EIGHT INHERITED `at_stator` READERS ON A BLEED-ARMED MACHINE.
#
# s 5.21 (ii)'s step-4 checklist item (a).  ONLY `schedule_invariance` is called anywhere in
# either suite (once, in `test_rung63.py`); the other seven are 0 calls and 0 gates, so their
# rung-62 behaviour is UNGATED in Python and this section is where it is gated.
#
# SIX OF THE EIGHT REFUSE A BLEED-ONLY MACHINE -- they assert a STATOR arming.  So they run
# here on a machine carrying BOTH devices, and the six refusals are recorded as their own keys.
# Running them on a bleed-only machine would have emitted nothing and looked like coverage.
# =============================================================================================
# --- (a) the six REFUSALS on a bleed-only machine, and the two non-refusals beside them.  A
# --- discrete parity fact, and the reason the readers below are run on a both-armed machine.
for name, call in (
        ("credit_decomposition",
         lambda m: m.credit_decomposition(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_62)),
        ("composite_credit",
         lambda m: m.composite_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_63,
                                      accel=LEG)),
        ("engagement_shift",
         lambda m: m.engagement_shift(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_63,
                                      accel=LEG)),
        ("matched_credit",
         lambda m: m.matched_credit(FLIGHT, LO, HI, MARGIN, r=0.5, s_settle=SETTLE, ds=DS_63)),
        ("set_point_bands",
         lambda m: m.set_point_bands(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_63)),
        ("floor_composite",
         lambda m: m.floor_composite(FLIGHT, LO, HI, SurgeLimiter(spool="lp", phi_lim=0.60),
                                     r=0.5, s_settle=SETTLE, ds=DS_63)),
        ("stator_credit",
         lambda m: m.stator_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_62)),
        ("schedule_invariance",
         lambda m: m.schedule_invariance(FLIGHT, LO, HI, MARGIN, 13))):
    try:
        call(bt(**BLEED_KW))
        b(f"H/refuses_bleed_only/{name}", False)
    except AssertionError:
        b(f"H/refuses_bleed_only/{name}", True)

# --- (a2) A SECOND refusal SHAPE, and it is not an arming one.  On the both-SCHEDULED machine
# --- `matched_credit` at margin 0.25 trips rung 59's own CLAMP AUDIT: the schedule is consulted
# --- outside the derived bracket at 3 of 210 cutting points, so the number would be an envelope
# --- edge rather than the derived shape.  Recorded rather than tuned away -- and it is why (c)
# --- runs that one reader at MARGIN_H.
for mtg, mkw in (("sched", {**BLEED_KW, **STAT_KW}), ("const", dict(bleed=B, vsv_lp=V))):
    try:
        bt(**mkw).matched_credit(FLIGHT, LO, HI, MARGIN, r=0.5, s_settle=SETTLE, ds=DS_63,
                                 n=13)
        b(f"H/clamp_refusal/{mtg}", False)
    except AssertionError:
        b(f"H/clamp_refusal/{mtg}", True)

# --- (b) `at_stator` ITSELF, with the FIVE argument shapes.  Three of the eight readers pass
# --- stator arguments to it internally, so the cell must carry `self`'s VALVE while honouring
# --- the PASSED setting.  Get that wrong for the arg-passing readers and the ONE gated reader
# --- still passes.
for tg, kw in (("none", {}), ("vsv_lp", dict(vsv_lp=V)), ("vsv_hp", dict(vsv_hp=0.10)),
               ("sched_lp", dict(vsv_sched_lp=StatorSchedule(V, N_LO))),
               ("sched_hp", dict(vsv_sched_hp=StatorSchedule(V, N_LO)))):
    for mtg, mkw in (("sched", BLEED_KW), ("const", CONST_KW)):
        sib = bt(**mkw).at_stator(**kw)
        p = f"H/at_stator/{mtg}/{tg}"
        b(f"{p}/armed_bleed", sib._armed_bleed())
        b(f"{p}/is_scheduled_stator", sib._is_armed())
        b(f"{p}/guard_armed_stator", guard_armed(sib))
        f(f"{p}/bleed", sib.bleed)
        f(f"{p}/b_of_080", sib.b_of(0.80))
        f(f"{p}/vsv_lp", sib.vsv_lp)
        f(f"{p}/vsv_hp", sib.vsv_hp)
        b(f"{p}/sched_lp_present", sib.vsv_sched_lp is not None)
        b(f"{p}/sched_hp_present", sib.vsv_sched_hp is not None)
        f(f"{p}/v_of_lp_080", sib.v_of("lp", 0.80, 0.85))
        f(f"{p}/v_of_hp_085", sib.v_of("hp", 0.80, 0.85))

# --- (c) THE EIGHT READERS, run.  Machine: bleed schedule + stator schedule, and the
# --- constant/constant pair beside it, so the arg-passing sweeps inside three of them are
# --- exercised on both arming modes.
CK = ("bare", "armed", "pointwise", "credit", "credit_pointwise", "erosion", "closed_form",
      "v_at_min", "s_at_min", "s_at_min_bare", "nu0_bare", "nu0_armed", "min_phi_bare",
      "min_phi_armed", "m_phi_bare", "m_phi_armed", "r")
DK = ("bare", "start", "ramp", "full", "share_start", "share_ramp", "self_cancel",
      "nu0_bare", "nu0_armed")
HK = ("predicted", "profile_bare", "profile_fuel", "credit_bare", "credit_fuel",
      "interaction", "share", "v_bare", "v_fuel", "v_ratio", "relocation", "relocation_bare",
      "leg_cost_bare", "leg_cost_armed", "fuel_removed_bare", "fuel_removed_armed", "r", "ds")
EK = ("bare_limited", "bare_dormant", "armed_limited", "armed_dormant", "d_limited",
      "d_dormant", "rel_limited", "rel_dormant", "r", "ds")
MK = ("credit_bare", "interaction_bare_leg", "interaction_matched", "delta_match",
      "delta_index", "delta_value", "abscissa_share", "ordinate_share", "share_bare_leg",
      "share_matched", "s_eng_bare_leg", "s_eng_matched", "removed_bare_leg",
      "removed_matched", "relocation", "d_ordinate", "d_abscissa", "margin", "r", "ds")
BK = ("gap_phi", "gap_m", "gap_phi_bands", "gap_m_bands", "credit", "excursion", "criterion",
      "identity_residual", "overlap_lo", "overlap_hi", "r", "ds")
FK = ("credit_bare", "credit_fuel", "interaction", "pinned_prediction", "pinned_residual",
      "s_eng_bare", "s_eng_armed", "d_s_eng", "removed_bare", "removed_armed", "v_at_min",
      "r", "ds")

for mtg, mkw in (("sched", BOTH_KW), ("const", BOTH_CONST_KW)):
    m = bt(**mkw)
    # 1 -- stator_credit (the ONE with no arming assert)
    for sp in ("lp", "hp"):
        c = m.stator_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_62, spool=sp)
        p = f"H/{mtg}/credit_{sp}"
        for k in CK:
            f(f"{p}/{k}", c[k])
        b(f"{p}/pointwise_exact", c["pointwise_exact"])
    # 2 -- credit_decomposition (passes `vsv_lp=v_at_min` to `at_stator` internally)
    dc = m.credit_decomposition(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_62)
    for k in DK:
        f(f"H/{mtg}/dec/{k}", dc[k])
    # 3 -- composite_credit
    cc = m.composite_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_63, accel=LEG)
    for k in HK:
        f(f"H/{mtg}/comp/{k}", cc[k])
    tag(f"H/{mtg}/comp/leg/{cc['leg']}")
    for _c in ("neither", "stator", "fuel", "both"):
        put_cell(f"H/{mtg}/comp/{_c}", cc["cells"][_c])
    # 4 -- engagement_shift (sweeps `at_stator(**kw)` internally)
    es = m.engagement_shift(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_63, accel=LEG)
    for k in EK:
        f(f"H/{mtg}/eng/{k}", es[k])
    tag(f"H/{mtg}/eng/leg/{es['leg']}")
    # 5 -- schedule_invariance (the ONE the suites call)
    inv = m.schedule_invariance(FLIGHT, LO, HI, MARGIN, 13)
    b(f"H/{mtg}/inv/ordinate_identical", inv["ordinate_identical"])
    b(f"H/{mtg}/inv/abscissa_identical", inv["abscissa_identical"])
    f(f"H/{mtg}/inv/d_ordinate", inv["d_ordinate"])
    f(f"H/{mtg}/inv/d_abscissa", inv["d_abscissa"])
    put_chain(f"H/{mtg}/inv/chain", inv["chain"])
    # 6 -- matched_credit
    # MARGIN_H, not MARGIN -- see (a2): 0.25 REFUSES this machine, by design.
    mc = m.matched_credit(FLIGHT, LO, HI, MARGIN_H, r=0.5, s_settle=SETTLE, ds=DS_63,
                          n=13)
    for k in MK:
        f(f"H/{mtg}/matched/{k}", mc[k])
    b(f"H/{mtg}/matched/ordinate_identical", mc["ordinate_identical"])
    b(f"H/{mtg}/matched/abscissa_identical", mc["abscissa_identical"])
    for _c in ("neither", "stator", "fuel", "both_bare_leg", "both_matched",
               "both_reindexed", "both_revalued"):
        put_cell(f"H/{mtg}/matched/{_c}", mc["cells"][_c])
    for _a in ("fuel", "both_bare_leg", "both_matched"):
        put_audit(f"H/{mtg}/matched/audit/{_a}", mc["audits"][_a])
    # 7 -- set_point_bands (sweeps `at_stator(**kw)` over a ramp-rate ladder internally)
    sb = m.set_point_bands(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_63)
    for k in BK:
        f(f"H/{mtg}/bands/{k}", sb[k])
    b(f"H/{mtg}/bands/phi_admissible", sb["phi_admissible"])
    b(f"H/{mtg}/bands/m_admissible", sb["m_admissible"])
    for side in ("bare", "armed"):
        for k in ("phi_0", "phi_min", "phi_exc", "m_0", "m_min", "m_exc", "T_c", "v_0"):
            f(f"H/{mtg}/bands/{side}/{k}", sb[side][k])
    # 8 -- floor_composite, on BOTH floor kinds
    for ftg, floor in (("phi", SurgeLimiter(spool="lp", phi_lim=0.60)),
                       ("inc", IncidenceLimiter(spool="lp", m_lim=0.500))):
        fc = m.floor_composite(FLIGHT, LO, HI, floor, r=0.5, s_settle=SETTLE, ds=DS_63)
        p = f"H/{mtg}/floor_{ftg}"
        for k in FK:
            f(f"{p}/{k}", fc[k])
        tag(f"{p}/regime/{fc['regime']}")
        tag(f"{p}/kind/{fc['floor']}")
        b(f"{p}/admissible", fc["admissible"])
        for _c in ("neither", "stator", "fuel", "both"):
            put_cell(f"{p}/{_c}", fc["cells"][_c])
        for _a in ("fuel", "both"):
            put_pin(f"{p}/audit/{_a}", fc["audits"][_a])

# =============================================================================================
# J -- THE REDUCE, AND IT IS THIS FILE'S CONTROL SECTION.
#
# b == 0 dispatches to rung 57's own body VERBATIM at every state, so an unbled machine is rung
# 57 (hence rungs 43-52) bit-for-bit.  This is a code path rung 62 NEVER ENTERS, which is what
# makes it the control: a disagreement reaching HERE is the GRID's, not the port's.  Step 2's
# first smoke run failed 100 keys in this section on a gas one ULP away.
# =============================================================================================
RKEYS = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "pi_lpc", "pi_hpc",
         "Phi_lp", "Phi_hp", "sp_thrust", "m_lp", "m_hp", "Tt25", "Tt3")
for tg, kw57, kw62 in (("bare", {}, {}),
                       ("vconst", dict(vsv_lp=V), dict(vsv_lp=V)),
                       ("vsched", dict(vsv_sched_lp=StatorSchedule(V, N_LO)),
                        dict(vsv_sched_lp=StatorSchedule(V, N_LO))),
                       ("bmax0", {}, dict(bleed_sched=BleedSchedule(0.0, N_LO)))):
    a = ScheduledStatorTransient(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw57)
    c = bt(**kw62)
    for Tt4 in (1000.0, 1200.0, 1400.0):
        ea, ec = a.equilibrium(FLIGHT, Tt4), c.equilibrium(FLIGHT, Tt4)
        for k in RKEYS:
            assert ea[k] == ec[k], f"{tg} {Tt4} {k}: {ea[k]!r} != {ec[k]!r}"
            f(f"J/{tg}/{Tt4:.0f}/{k}", ec[k])
    # and the FUEL path, which has its own dispatch and its own bracket
    mf = a.fuel_for_Tt4(FLIGHT, 1200.0)
    f(f"J/{tg}/fuel/mf", mf)
    b(f"J/{tg}/fuel/mf_identical", mf == c.fuel_for_Tt4(FLIGHT, 1200.0))
    ia, ic = a._instant_fuel(FLIGHT, 0.85, 0.88, mf), c._instant_fuel(FLIGHT, 0.85, 0.88, mf)
    for k in RKEYS:
        assert ia[k] == ic[k], f"{tg} fuel {k}"
        f(f"J/{tg}/fuel/{k}", ic[k])
    # the MARCH, which is where a per-call dispatch could drift without a value moving
    _traj = c._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_62)[0]
    put_traj(f"J/{tg}/march", _traj, stride=29)
    put_read(f"J/{tg}/read", c._read(_traj))

# =============================================================================================
# K -- **ADDED**: THE DISPATCH CENSUS.  Integer counts, not values.
#
# s 5.21 (v)'s four reduced/bled pairs, over ONE `equilibrium(FLIGHT, LO)` plus ONE
# `_stator_march` at ds = 0.02 -- `probe_w3.py`'s workload EXACTLY.  Step 3's finding 5 is what
# fixes the Tt4: at 1200 the BARE machine's Newton takes three closes fewer (62 against 65)
# while the scheduled one is unchanged, so a census on the wrong throttle reproduces one row of
# the pre-registered table and misses another by 3 -- which reads as a port defect.
#
# `b_of`'s CALL COUNT is here for step 3's finding 3: a `_powers` "simplified" to re-read
# `b_of` moves the CALL COUNT (409 -> 818) and leaves ALL EIGHT PAIRS UNTOUCHED, because the
# two spellings agree at EVERY call on this plant.  P4 named the pairs as the instrument; they
# are as blind as the value keys, and the call count is what betrays the re-read.
#
# **THE CLASS METHOD IS PATCHED, NOT THE INSTANCE.**  Rust's counters are a thread-local
# GLOBAL, so they count `b_of` on every object alive during the workload.  An instance-level
# patch would count one object's calls and silently under-report any sibling's.  The classifier
# inside the `_close` wrappers calls the SAVED ORIGINAL, so it never inflates the count it is
# reading beside.
# =============================================================================================
_ORIG_B_OF = ScheduledBleedTransient.b_of
_ORIG_AT_LEVER = ScheduledBleedTransient.at_lever
_ORIG_AT_STATOR = ScheduledBleedTransient.at_stator
_ORIG_ISOLATING = ScheduledBleedTransient._isolating
_ORIG_LEGS = ScheduledBleedTransient._legs
_ORIG_CLOSE = ScheduledBleedTransient._close
_ORIG_CLOSE_FUEL = ScheduledBleedTransient._close_fuel
_ORIG_POWERS = ScheduledBleedTransient._powers
_ORIG_TAIL = ScheduledBleedTransient._instant_tail
COUNTERS = ("close_reduced", "close_bled", "close_fuel_reduced", "close_fuel_bled",
            "powers_reduced", "powers_bled", "tail_reduced", "tail_bled",
            "b_of_calls", "b_of_constant", "b_of_sched_zero", "b_of_sched_open",
            "at_lever_calls", "at_stator_r62", "isolating_calls", "legs_calls",
            "legs_lever_bleed")


def census(tg, workload="march", **kw):
    """**EVERY WRAPPER IS ON THE CLASS, NOT ON ONE INSTANCE.**

    Rust's counters are a thread-local GLOBAL: `bump(&CLOSE_BLED)` fires wherever the cell
    runs, on `self` or on any sibling alive at the time.  An instance-level wrapper would count
    one object and silently under-report every sibling -- which is invisible on the `march`
    workload (it builds none) and wrong by a factor on the `siblings` one (it builds four
    marching machines).  Patching the class is what makes the two censuses comparable.

    The classifiers call the SAVED ORIGINAL `b_of`, so they never inflate the call count they
    are recorded beside."""
    t = dict.fromkeys(COUNTERS, 0)

    def counted_b_of(self, nu_lp, Tt2=None):
        t["b_of_calls"] += 1
        v = _ORIG_B_OF(self, nu_lp, Tt2)
        if self.bleed_sched is None:
            t["b_of_constant"] += 1
        else:
            t["b_of_sched_zero" if v == 0.0 else "b_of_sched_open"] += 1
        return v

    def counted_at_lever(self, *a, **k):
        t["at_lever_calls"] += 1
        return _ORIG_AT_LEVER(self, *a, **k)

    def counted_at_stator(self, *a, **k):
        t["at_stator_r62"] += 1
        return _ORIG_AT_STATOR(self, *a, **k)

    def counted_isolating(self, *a, **k):
        t["isolating_calls"] += 1
        return _ORIG_ISOLATING(self, *a, **k)

    def counted_legs(self, *a, **k):
        t["legs_calls"] += 1
        r = _ORIG_LEGS(self, *a, **k)
        if r["lever"] == "bleed":
            t["legs_lever_bleed"] += 1
        return r

    def counted_close(self, nu_lp, nu_hp, Tt4, Tt2, pt2):
        t["close_reduced" if _ORIG_B_OF(self, nu_lp, Tt2) == 0.0 else "close_bled"] += 1
        return _ORIG_CLOSE(self, nu_lp, nu_hp, Tt4, Tt2, pt2)

    def counted_close_fuel(self, nu_lp, nu_hp, mf, Tt2, pt2):
        t["close_fuel_reduced" if _ORIG_B_OF(self, nu_lp, Tt2) == 0.0
          else "close_fuel_bled"] += 1
        return _ORIG_CLOSE_FUEL(self, nu_lp, nu_hp, mf, Tt2, pt2)

    def counted_powers(self, c, flight, nu_lp, nu_hp, Tt4):
        t["powers_reduced" if c.get("bleed", 0.0) == 0.0 else "powers_bled"] += 1
        return _ORIG_POWERS(self, c, flight, nu_lp, nu_hp, Tt4)

    def counted_tail(self, flight, c, nu_lp, nu_hp, Tt4, V0):
        t["tail_reduced" if c.get("bleed", 0.0) == 0.0 else "tail_bled"] += 1
        return _ORIG_TAIL(self, flight, c, nu_lp, nu_hp, Tt4, V0)

    ScheduledBleedTransient.b_of = counted_b_of
    ScheduledBleedTransient.at_lever = counted_at_lever
    ScheduledBleedTransient.at_stator = counted_at_stator
    ScheduledBleedTransient._isolating = counted_isolating
    ScheduledBleedTransient._legs = counted_legs
    ScheduledBleedTransient._close = counted_close
    ScheduledBleedTransient._close_fuel = counted_close_fuel
    ScheduledBleedTransient._powers = counted_powers
    ScheduledBleedTransient._instant_tail = counted_tail
    try:
        m = bt(**kw)
        if workload == "march":
            m.equilibrium(FLIGHT, LO)
            m._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS_CENSUS)
        else:
            # THE SECOND WORKLOAD, and it exists because the first one reported FOUR ZEROS
            # NOBODY LOOKED AT.  `equilibrium` + `_stator_march` never construct a sibling, so
            # `at_lever`, `at_stator`, `_isolating` and `_legs` all read 0 there -- and a dead
            # counter and an untaken path are the same character.  These two readers DO reach
            # them: `loop_decomposition` goes through `at_lever()`, `marginal_loop` through
            # `_isolating()`, and both end in `_legs`.  Non-zero HERE is what makes the zeros
            # above measured zeros.
            m.loop_decomposition(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS_CENSUS)
            m.marginal_loop(FLIGHT, LO, HI, BLEED_KW, r=0.5, s_settle=SETTLE, ds=DS_CENSUS)
            # ...and `schedule_invariance`, purely so `at_stator_r62` is measured SOMEWHERE.
            # It is the FIFTH counter that would otherwise be zero in all seven censuses:
            # nothing above it constructs a rung-57 sibling, and `at_stator` is the cell
            # s 5.21 (ii) forced the whole module layout for.  A counter no workload reaches
            # cannot report an inert one.
            m.schedule_invariance(FLIGHT, LO, HI, MARGIN, 5)
    finally:
        ScheduledBleedTransient.b_of = _ORIG_B_OF
        ScheduledBleedTransient.at_lever = _ORIG_AT_LEVER
        ScheduledBleedTransient.at_stator = _ORIG_AT_STATOR
        ScheduledBleedTransient._isolating = _ORIG_ISOLATING
        ScheduledBleedTransient._legs = _ORIG_LEGS
        ScheduledBleedTransient._close = _ORIG_CLOSE
        ScheduledBleedTransient._close_fuel = _ORIG_CLOSE_FUEL
        ScheduledBleedTransient._powers = _ORIG_POWERS
        ScheduledBleedTransient._instant_tail = _ORIG_TAIL
    for k in COUNTERS:
        d(f"K/{tg}/{k}", t[k])


census("bare")
census("stator", **STAT_KW)
census("sched", **BLEED_KW)
census("const", **CONST_KW)
census("both", **BOTH_KW)
# The SIBLING workload -- `loop_decomposition` + `marginal_loop`, which is where `at_lever`,
# `at_stator`, `_isolating` and `_legs` are actually reached.  Without it, four of the census's
# seventeen counters are zeros nobody measured.
census("sib_sched", workload="siblings", **BLEED_KW)
census("sib_const", workload="siblings", **CONST_KW)

# ---------------------------------------------------------------------------- emit
print("# slice W step 4 -- rungs 62-63 ORACLE. key<TAB>u64 (float keys are IEEE-754 bits).")
seen = set()
for k, v in OUT:
    assert k not in seen, f"duplicate key {k}"
    seen.add(k)
    print(f"{k}\t{v}")
print(f"# {len(OUT)} keys", file=sys.stderr)
