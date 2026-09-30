"""SLICE AJ step 6 — **THE ORACLE for rungs 81, 82, 83 AND 84**, run once on PyPy and once on CPython.

    .venv\\Scripts\\python.exe rust/oracle/dump_slice_aj.py rust/oracle/slice_aj_pypy.tsv
    C:\\Python314\\python.exe   rust/oracle/dump_slice_aj.py rust/oracle/slice_aj_cpython.tsv

Launch each through PowerShell `Start-Process -FilePath <interpreter>`, never a quoted `cmd start`
(plan § 5.34.4 (d): `start` ate the interpreter path and the "PyPy" file was CPython's). The
ENTRY CONTROL below refuses the mismatch by itself: the first line printed is `sys.version`, and
an output whose name says `pypy` / `cpython` is refused off that interpreter.

# THE FORMAT IS STEPS 2–4's, NOT SLICES AH/AI's — AND WHY THAT STILL CATCHES A FORGOTTEN KEY

One line per value, `path <TAB> token`, IN PYTHON's OWN ORDER: a dict writes `keys:N` then its
items in insertion order, a list `len:N` then its items, a float its IEEE bits (`f:nan` for a
NaN), `n` for `None`. `rust/tests/slice_aj_oracle.rs` writes the same lines from its own structs
through `tests/slice_aj_flat/mod.rs` — the flatteners steps 2–4 verified against their own
oracles, lifted into one module so this file adds no second description of a struct. AH/AI walked
their keys here and HAND-LISTED them in Rust so a key both sides forgot failed by name; in this
format Python writes EVERY key it returns and the Rust side must write the same path at the same
position, so a field the port forgot fails at its path, and a field out of order fails too — the
order AH/AI's sorted walk could not see.

# WHAT THIS ADDS THAT STEPS 2–4's ORACLES DO NOT

1. **CPYTHON.** Steps 2–4 were PyPy only. § 5.34 (ix) P2 predicted this oracle *"bit-exact on
   CPython except `authority_mask`'s `c0`/`c1`"*; step 4 (§ 5.34.4 (d)) already REFUTED that at two
   of its readings, by `ComponentMap.eta_c_at`'s `(n - 1.0) ** 2` (CPython's `pow` misrounds a
   square where PyPy multiplies) amplified by the Illinois solve's `tol = 1e-12`. Those two
   readings are kept VERBATIM here (`p_scan_p4`, `p_secant_iter_v4`) as the CPython arm's positive
   controls. The CPython prediction lives in `slice_aj_oracle.rs`' header and in
   `W:/temp/claude/slice-aj-step6/predictions.md`, written before the CPython file was opened.
2. **THE INCIDENCE STATOR ARM** (`i_*` readings). Every rung-81–84 call in the four suites, in
   steps 2–5's oracles and in the r80–r82t kernels passes `inc = False` (§ 5.34 (iii) C), so no
   gate in either language could see a port that DROPS the flag. The arm is aimed where it is
   LIVE, measured on PyPy before this file was written (`W:/temp/claude/slice-aj-step6/
   probe_inc.py`): at `r = 0.25` the four-loop window is open and NON-MONOTONE in `tau_f`
   (`n_fuel` 4, 10, 4, 3 at `0.004`, `0.02`, `0.05`, `0.30`); at `r >= 0.35` it is mostly empty and
   at rung 81's default `r = 0.5` every row is zero. The bisecting readers VOID on this arm (the
   detector is already on at `0.004`), so it carries one void reading and otherwise the direct
   readers inside `[0.004, 0.05]`.
3. **THE PLANT UNDER THE READERS** (`p_plant`, `i_plant`): `_scan_cells` — the march every
   rung-82/83/84 reader reduces — walked every fifth point WHOLE plus min/max/last/neg of every
   float column (AH/AI's section P), with `_riding4`'s points as trajectory INDICES (`id(p)`,
   what `riding4_idx` ports) and every scored cell.
4. **A LADDER WHOSE `n - 1` IS NOT A POWER OF TWO** (`p_shape_n4`) — aimed at step 4's coverage gap
   C6 (`lo + (hi-lo)*i/(n-1)` vs `lo + i*step`), and it MISSES: the two spellings are bit-identical
   at all four points of this ladder, and of `i_shape`'s. Separation depends on `(lo, hi, n)`
   JOINTLY, not on `n` (271 of 589 nearby ladders separate, e.g. `[0.004, 0.024]` at `n = 11`).
   C6 stays OPEN; the injection survived (plan § 5.34.6 (e)).

# P2's BUDGET

§ 5.34 (ix) P2: the dump cannot replay the suites' grids (rung 84's suite alone is ~1 200 s), so
it takes ONE ramp and ONE `ds` pair per reader and says so: `threshold_law` runs `rs = (0.25,)` at
`ds = 0.005` with `ds_fine = 0.0025`; `threshold_terms` three walls; `lattice_count` the same
window at both `ds`. Rung 81's two readers are cheap and run at their fixtures' own defaults.

# THE INTERPRETER SENTINEL

`_interp.sum_probe`, AH's: `sum([1e16, 1.0, -1e16])` is `1.0` under CPython 3.12+ and `0.0` under
PyPy's naive fold. Excluded from every comparison by name, asserted per arm.
"""
import os
import platform
import struct
import sys
import time

print(sys.version.replace("\n", " "), flush=True)

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, REPO)

from turbojet.gas import Gas                                              # noqa: E402
from turbojet.engine import (                                             # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    AuthorityClockTransient, ThresholdLawTransient, CorrectorLawTransient,
    StaircaseLawTransient, BleedLimiter, StatorIncidenceLimiter, StatorLimiter,
)

# ---------------------------------------------------------- THE ENTRY CONTROL (§ 5.34.4 (d))
PATH = sys.argv[1] if len(sys.argv) > 1 else None
assert PATH, "usage: dump_slice_aj.py <out.tsv>"
_IMPL = platform.python_implementation()
_base = os.path.basename(PATH).lower()
assert not ("pypy" in _base and _IMPL != "PyPy"), "a *pypy* output written by %s" % _IMPL
assert not ("cpython" in _base and _IMPL != "CPython"), "a *cpython* output written by %s" % _IMPL

# ------------------------------------------------ `tests/test_rung81.py`…`test_rung84.py`'s grid
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, TT4_MAX = 1000.0, 1400.0, 1200.0
B, V_MAX = 0.10, 0.20
TAU, TAU_S = 0.05, 0.05
PHI_FUEL, PHI_AIR = 0.75, 0.77
SETTLE, DS, DS_FINE = 1.2, 0.005, 0.0025
BRACKET = (0.004, 0.30)
#: rung 83 § 3.2's jump (`r = 0.25`) and § 3.4's crossing (`r = 0.35`) — steps 4/5's literals.
JL, JH = 0.0197750, 0.0197875
CL, CH = 0.037000, 0.037333
T_START = (BRACKET[0] * BRACKET[1]) ** 0.5
#: `test_p5`'s spacing formula at `ds = 0.005`.
SPACING = 0.005 * (0.024 - 0.016) / 0.005938
#: The incidence arm's ramp — the one where its window is live (see the header).
R_INC = 0.25
#: The plant walk's clocks: a `tau_f` inside both arms' live windows.
PLANT_TAUS = (0.02, 0.05, 0.05, 0.05)
P_STRIDE = 5

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


DESIGN = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                  nozzle_convergent=True, **REAL)


def rig(cls, inc):
    """The four suites' `_rig`, with the stator arm made explicit (slice AI's dumper)."""
    sm = 0.80 / FLOOR - 1.0
    m = cls(DESIGN, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                        else None),
            stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m


def kw(r, inc, ds=DS, phi_lim=PHI_FUEL):
    """Rungs 83/84's `**kw` — every `_threshold_scan` argument except `tau_f`."""
    return dict(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=phi_lim,
                phi_air=PHI_AIR, tau_gov=0.05, tau_q=0.05, tau_s=0.05, r=r, s_settle=SETTLE,
                ds=ds, v_max=V_MAX, inc=inc)


# ------------------------------------------------------------- steps 3/4's `tok` and `flat`
def tok(x):
    if x is None:
        return "n"
    if isinstance(x, bool):
        return "b:%d" % int(x)
    if isinstance(x, int):
        return "i:%d" % x
    if isinstance(x, float):
        if x != x:
            return "f:nan"
        return "f:%016x" % struct.unpack("<Q", struct.pack("<d", x))[0]
    if isinstance(x, str):
        return "s:" + x
    raise TypeError((type(x), x))


def flat(path, x, out):
    if isinstance(x, set):
        x = sorted(x)
    # a FLOAT-keyed dict (`summands`) is its list of `[key, value]` pairs — by the FIELD's name,
    # since an empty one has no key to test (step 4's first-run defect)
    if isinstance(x, dict) and (path.rsplit(".", 1)[-1] == "summands"
                                or any(not isinstance(k, str) for k in x)):
        x = list(x.items())
    if isinstance(x, (list, tuple)):
        out.append((path, "len:%d" % len(x)))
        for i, y in enumerate(x):
            flat("%s.%d" % (path, i), y, out)
    elif isinstance(x, dict):
        out.append((path, "keys:%d" % len(x)))
        for k, v in x.items():
            flat("%s.%s" % (path, k), v, out)
    else:
        out.append((path, tok(x)))


OUT = []
# THE INTERPRETER SENTINEL — see the header.
OUT.append(("_interp.sum_probe", tok(float(sum([1e16, 1.0, -1e16])))))


def plant(m, tau_f, r, inc):
    """`_scan_cells` — the march, `_riding4` as INDICES, the scored cells — reduced for a walk."""
    traj, ride, cells = m._scan_cells(FLIGHT, LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, tau_f,
                                      PLANT_TAUS[1], PLANT_TAUS[2], PLANT_TAUS[3], r, SETTLE,
                                      DS, V_MAX, inc)
    # An EMPTY march would agree with itself on every key below (§ 5.27 (ii)).
    assert traj, "an empty march is not a measurement"
    index = {id(p): i for i, p in enumerate(traj)}
    cols = [k for k, v in traj[0].items() if isinstance(v, float)]
    for p in traj:
        assert [k for k, v in p.items() if isinstance(v, float)] == cols, "a column moved"
    col = {k: [p[k] for p in traj] for k in cols}
    return dict(
        n=len(traj),
        points=[(i, traj[i]) for i in range(0, len(traj), P_STRIDE)],
        cols={k: dict(min=min(v), max=max(v), last=v[-1], neg=sum(1 for x in v if x < 0.0))
              for k, v in col.items()},
        ride=[index[id(p)] for p in ride],
        cells=cells,
    )


# (name, the rung's own class, arm, call). A `rig` is built FRESH per reading, as steps 2–4 do.
C81, C82, C83, C84 = (AuthorityClockTransient, ThresholdLawTransient, CorrectorLawTransient,
                      StaircaseLawTransient)
F, T = False, True
READINGS = [
    # ---------------------------------------------------------------- the `phi` stator arm
    # rung 81 — both module fixtures, verbatim (defaults otherwise)
    ("p_clock", C81, F, lambda m: m.authority_clock(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                                    phi_air=PHI_AIR)),
    ("p_mask", C81, F, lambda m: m.authority_mask(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                                  phi_air=PHI_AIR)),
    # rung 82 — one ramp and one `ds` pair; `_threshold_scan` at rung 81's ramp
    ("p_law", C82, F, lambda m: m.threshold_law(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                                phi_air=PHI_AIR, rs=(0.25,), ds_fine=DS_FINE)),
    ("p_ref", C82, F, lambda m: m.threshold_reference(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                                      phi_air=PHI_AIR)),
    ("p_terms", C82, F, lambda m: m.threshold_terms(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                                    phi_air=PHI_AIR,
                                                    phi_lims=(0.745, 0.750, 0.755))),
    ("p_scan", C82, F, lambda m: m._threshold_scan(tau_f=0.05, **kw(0.5, F))),
    # rung 83
    ("p_read", C83, F, lambda m: m.corrector_read(0.08, **kw(0.5, F))),
    ("p_shape_n4", C83, F, lambda m: m.residual_shape(0.016, 0.024, n=4, **kw(0.25, F))),
    ("p_secant", C83, F, lambda m: m.corrector_secant(T_START, 1.25 * T_START, cap=6,
                                                      bracket=BRACKET, **kw(0.35, F))),
    # step 4's `secant_iter_v4`, VERBATIM — a CPython positive control
    ("p_secant_iter_v4", C83, F, lambda m: m.corrector_secant(0.30, 0.29, cap=3, **kw(1.0, F))),
    # rung 84
    ("p_edge_lo", C84, F, lambda m: m.edge_read(JL, **kw(0.25, F))),
    ("p_edge_hi", C84, F, lambda m: m.edge_read(JH, **kw(0.25, F))),
    # step 4's `scan_p4`, VERBATIM — a CPython positive control
    ("p_scan_p4", C84, F, lambda m: m.staircase_scan(0.0190, 0.0206, 9, **kw(0.25, F))),
    ("p_lattice", C84, F, lambda m: m.lattice_count(0.016, 0.024, **kw(0.25, F))),
    ("p_lattice_fine", C84, F, lambda m: m.lattice_count(0.016, 0.024, **kw(0.25, F, DS_FINE))),
    ("p_number", C84, F, lambda m: m.staircase_number(JL, JH, spacing=SPACING, **kw(0.25, F))),
    ("p_root", C84, F, lambda m: m.root_class(bracket=BRACKET, **kw(0.35, F))),
    ("p_plant", C84, F, lambda m: plant(m, PLANT_TAUS[0], R_INC, F)),
    # ----------------------------------------------------------- the INCIDENCE stator arm
    ("i_clock", C81, T, lambda m: m.authority_clock(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                                    phi_air=PHI_AIR,
                                                    tau_fs=(0.004, 0.02, 0.05),
                                                    tau_govs=(0.02, 0.05), r=R_INC, inc=T)),
    ("i_mask", C81, T, lambda m: m.authority_mask(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                                  phi_air=PHI_AIR, r=R_INC, inc=T)),
    # the arm's one void: the detector is on at the bracket's low end
    ("i_law", C82, T, lambda m: m.threshold_law(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_FUEL,
                                                phi_air=PHI_AIR, rs=(R_INC,), ds_fine=None,
                                                inc=T)),
    ("i_scan", C82, T, lambda m: m._threshold_scan(tau_f=0.02, **kw(R_INC, T))),
    ("i_read", C83, T, lambda m: m.corrector_read(0.02, **kw(R_INC, T))),
    ("i_shape", C83, T, lambda m: m.residual_shape(0.004, 0.05, n=4, **kw(R_INC, T))),
    ("i_secant", C83, T, lambda m: m.corrector_secant(0.01, 0.0125, cap=4, bracket=(0.004, 0.05),
                                                      **kw(R_INC, T))),
    ("i_edge_lo", C84, T, lambda m: m.edge_read(0.004, **kw(R_INC, T))),
    ("i_edge_hi", C84, T, lambda m: m.edge_read(0.02, **kw(R_INC, T))),
    ("i_ladder", C84, T, lambda m: m.staircase_scan(0.004, 0.05, 5, **kw(R_INC, T))),
    ("i_lattice", C84, T, lambda m: m.lattice_count(0.004, 0.05, **kw(R_INC, T))),
    ("i_plant", C84, T, lambda m: plant(m, PLANT_TAUS[0], R_INC, T)),
]

# `classify` is a staticmethod on two `edge_read`s already in hand — read, not re-marched.
CLASSIFY = [("p_classify", "p_edge_lo", "p_edge_hi"), ("i_classify", "i_edge_lo", "i_edge_hi")]
# `corrector_step` likewise, on `p_read` (step 4's `step_ok` argument).
STEP = [("p_step", "p_read", 0.044)]

names = [n for n, *_ in READINGS]
assert len(set(names)) == len(names), "a duplicate reading name"
kept = {}
t_all = time.time()
for name, cls, inc, fn in READINGS:
    m = rig(cls, inc)
    t0 = time.time()
    res = fn(m)
    kept[name] = res
    flat(name, res, OUT)
    # the rig after the reader: `_split_march`'s `_with_air` must have restored the knob
    flat(name + "@rig", dict(lag_coord=m._lag_coord, sm_air=m._sm_air), OUT)
    print("%-18s %7.1f s  %6d lines" % (name, time.time() - t0, len(OUT)), flush=True)
for name, a, b in CLASSIFY:
    flat(name, StaircaseLawTransient.classify(kept[a], kept[b]), OUT)
for name, a, c in STEP:
    flat(name, CorrectorLawTransient.corrector_step(kept[a], c), OUT)

paths = [p for p, _ in OUT]
assert len(set(paths)) == len(paths), "a duplicate path"
with open(PATH, "w", encoding="utf-8", newline="\n") as fh:
    for p, t in OUT:
        fh.write("%s\t%s\n" % (p, t))
print("lines %d -> %s   total %.1f s   (%s)" % (len(OUT), PATH, time.time() - t_all, _IMPL),
      flush=True)
