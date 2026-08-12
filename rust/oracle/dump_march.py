"""THE ORACLE, phase 4 slice F — every rung-25/26 MARCH value the Rust must reproduce.

The eighth in the family (`dump_gas.py` → `dump_cycle.py` → `dump_nox.py` → `dump_quench.py` →
`dump_pdf.py` → `dump_spatial.py` → `dump_nozzle.py` → here). A separate file again, for the usual
reason: the earlier TSVs stay frozen as their own audit trail, and each dump's regeneration cost
stays proportional to what it certifies.

WHAT IS ACTUALLY NEW HERE, and therefore what the sweep is built around:

  * A MARCH, not a solve. Rungs 7–24 evaluate closures and root-find; these two INTEGRATE, 100 or
    400 steps deep, each step re-solving the equilibrium composition and then bisecting a
    temperature. Error does not merely appear — it ACCUMULATES, and the exit state is 400 chained
    solves downstream of the entry. That is what makes bit-equality here a stronger statement than
    it was in slice E, where every gated quantity was a single solve.

  * `dS` LEADS, and it is the most drift-sensitive quantity in the port so far. It is
    `S_exit − S_entry`: a difference of two molar entropies that legitimately lands NEGATIVE in 13
    of 70 cells at the shipped `nstep` (§ 4.11 probe 3). Its SIGN is not fixed, so it is a near-
    total cancellation, and slice 5's lesson — a finite difference inherits its drift from the
    quantity differenced — says a relative bar on it would be measuring the wrong thing. Under
    bit-equality that same property makes it the best detector in the dump.

  * THREE BISECTION TOLERANCES IN ONE SLICE: `1e-11·Tm` (the energy bisection of both marches),
    `1e-10·T` (`_equilibrate_hp`), and `1e-13·Tm` (slice E's `_expand_nozzle`, reached through
    (F)/(R)/(I)). All three share slice E's named loop shape — counted `range(200)`, midpoint at
    the TOP, bracket updated, break on THIS iteration's PRE-update midpoint, result recomputed
    from the final bracket AFTER the loop. Transcribing them uniformly is the silent defect this
    dump exists to catch, and it is why `equilibrate_hp` is dumped DIRECTLY as well as through
    `_irreversible_fast_expand`.

  * A CLAIM OF EXACTNESS THAT SURVIVED. `_freeze_out_expand` says a CONSTANT `da_local_fn`
    reproduces `_finite_rate_expand(Da)` "to the ULP". § 4.11 probe 4 measured 40/40 bit-exact —
    the first such claim in this lineage to survive, after slices C, D and E each corrected one.
    The `red/` block dumps BOTH sides so the Python's own equality is in the TSV; the Rust-side
    equality is a separate gate in `tests/rung26.rs`, because a Python↔Rust dump cannot see a
    loop-shape error transcribed identically into both copies.

  * THE SPECIES ORDER, DUMPED AS DATA. § 4.11 probe 5 measured `_equilibrium_composition`'s key
    order input-independent over 112 states, so the `.get(sp, 0.0)` zero-fill in the two hand-built
    relaxation dictionaries has no reachable instance. The order is still load-bearing — both
    accumulations run in it and float addition is not associative — so `order/` records it as
    INDICES rather than leaving the Rust to retype a list by hand.

THE SIZING LEVER. `_expand_nozzle`'s (F) and (R), and `_irreversible_fast_expand`'s (I), depend
only on `(far, Tt4, pt4, Tt9, pt9, p9)` — NOT on `Da`, NOT on `nstep`, NOT on `rate_scale`. So the
three reference states are ONE call each per design point and the whole Da × nstep ladder sweeps
against them. Without the lever every Da point would pay three nozzle solves that cannot move.

THE DISCRETE KEYS, and what each is honestly worth:

  * `census/negative_ds` — how many cells of a fixed Da × Tt4 ladder produce a NEGATIVE `dS`. This
    one IS live and no tolerance expresses it: it is the count of cells where a physically
    non-negative quantity comes out the other side of the truncation with the wrong sign, and it
    moves with `nstep`. 13 of 70 at the probe's sweep.
  * `census/frozen_from_entry` — how many design points on the Tt4 ladder never switch the
    relaxation on at all (`Da_entry < 1`). This is rung 26's own dormant-lean claim as an integer:
    2 of 5 on the ladder below, and it MOVES with Tt4, which is the rung.
  * `order/<species>` — the index of each species in `_equilibrium_composition`'s returned order.

WHAT IS DELIBERATELY *NOT* DUMPED, AND WHERE IT WENT INSTEAD. The energy bisection's halving count.
An earlier draft of this header described an `iters/…/min|max` class and the dump never emitted one
— `_finite_rate_expand` returns four values and cannot report a count without instrumenting the
source, so the class was documentation for a gate that did not exist. Note that the Rust gate's
key-COUNT guard could not have caught that: neither side emitted the class, so the totals matched.

The count is worth keeping, but not here. Slice E classified its own `iters` as a NAMING key —
the value is already gated at bit-equality, so a mis-shaped loop is caught by the value and the
count only improves the failure message. There is however one thing the count says that no dumped
value can: **whether the loop CONVERGED at all.** `used == 200` means the bracket never met its
stopping rule, and that is invisible in the result, because `0.5*(lo+hi)` off an unconverged
bracket is a perfectly plausible temperature. A Python↔Rust dump cannot express it either, since
both sides would agree on the same unconverged number. So the count lives on `march::March` and is
gated in `tests/rung25.rs::the_energy_bisection_converges_far_inside_its_cap`, against the 36–37
halvings § 4.11 probe 1 measured across all 70 marches.

Regenerate with:
    py -3                     rust/oracle/dump_march.py rust/oracle/march_cpython.tsv
    .venv\\Scripts\\python.exe rust/oracle/dump_march.py rust/oracle/march_pypy.tsv
"""
import math
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import gas as G
from turbojet.engine import FlightCondition, build_turbojet
from turbojet.gas import (
    Gas, FiniteRate, FreezeOut,
    _equilibrium_composition, _expand_nozzle, _finite_rate_expand, _freeze_out_expand,
    _irreversible_fast_expand, _equilibrate_hp, _tau_chem_recomb,
    _mix_h_abs_B,
)

T0 = time.time()
ROWS = []


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


# --- the design points, and the knobs every section shares ------------------------------------
FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
              eta_t=0.90, eta_m=0.99, pi_n=0.98)
PI_C = 10.0

# (tag, Tt4). Tags are LITERAL strings, never a formatted float, so the Rust side cannot disagree
# about how Python spells a number in a key. The ladder is WIDER than the rung-25/26 suites' own
# {1500, 1800, 2200}: `cold` sits below where the relaxation ever switches on and `vhot` above
# where the suite stops, which is what makes the two censuses below able to move.
DPS = [
    ("cold", 1300.0),
    ("dp", 1500.0),
    ("warm", 1800.0),
    ("hot", 2200.0),
    ("vhot", 2300.0),
]

# The Da ladder — wider than the suite's {0.3, 1, 3, 10, 30} at BOTH ends. 0.03 is the near-frozen
# corner where § 4.11 probe 3 found the worst 2nd-law truncation; 300 is deep into the asymptote
# where the march converges on the closed-form (I) ceiling.
DA_LADDER = [("da003", 0.03), ("da03", 0.3), ("da1", 1.0), ("da3", 3.0),
             ("da10", 10.0), ("da30", 30.0), ("da300", 300.0)]

# ONE `Gas` PER DESIGN POINT, not one shared. The equilibrium section caches the burn condition
# it was frozen at and asserts on reuse, so a shared instance fails at the second Tt4 — the same
# construction the rung-25/26 suites use.
STATES = {}
for tag, tt4 in DPS:
    g = Gas.reacting_equilibrium()
    r = build_turbojet(g, PI_C, tt4, FLIGHT.p0, **LOSSES).run(FLIGHT, 1.0)
    st4, st9 = r.stations["4"], r.stations["9"]
    STATES[tag] = dict(gas=g, far=st4.far, Tt4=st4.Tt, pt4=st4.pt,
                       Tt9=st9.Tt, pt9=st9.pt, p9=r.p9)

# ==============================================================================================
# 0. THE SPECIES ORDER, AS DATA (§ 4.11 probe 5)
#
#    `sps = list(comp)` pins the iteration order of both hand-built relaxation dictionaries, and
#    `sum(comp1.values())` / `_mix_h_abs_B` then accumulate in it. Float addition is not
#    associative, so this is load-bearing even though the zero-fill branch it guards is
#    unreachable. Dumped as indices so the Rust reads the order rather than retyping it.
# ==============================================================================================
_order_ref = list(_equilibrium_composition(STATES["dp"]["far"], STATES["dp"]["Tt4"],
                                           STATES["dp"]["pt4"]))
for i, sp in enumerate(_order_ref):
    put(f"order/{sp}", float(i))
put("order/n_species", float(len(_order_ref)))

# The claim probe 5 made, re-asserted HERE so a future gas-table edit that reorders the species
# fails the dump rather than silently changing every accumulation downstream of it.
for tag, tt4 in DPS:
    s = STATES[tag]
    for T in (700.0, 1200.0, 1800.0, 2400.0):
        for p in (2.0e4, 1.0e5, 2.5e6):
            assert list(_equilibrium_composition(s["far"], T, p)) == _order_ref, \
                f"species order moved at far={s['far']}, T={T}, p={p}"

# ==============================================================================================
# 1. THE THREE REFERENCE STATES — (F) frozen, (R) reversible, (I) irreversible-fast
#
#    (F) and (R) are slice E's `_expand_nozzle` reached through a phase-4 caller, which is the
#    first thing outside slice E to exercise it. (I) adds `_equilibrate_hp` — the slice's third
#    bisection tolerance, and the one whose bracket § 4.11 probe 2 measured 37x wider than the
#    root offset it has to contain.
# ==============================================================================================
for tag, _ in DPS:
    s = STATES[tag]
    comp_entry = _equilibrium_composition(s["far"], s["Tt4"], s["pt4"])
    for sp, n in comp_entry.items():
        put(f"entry/{tag}/comp/{sp}", n)

    T9f, V9f, _ = _expand_nozzle(comp_entry, s["far"], s["Tt9"], s["pt9"], s["p9"],
                                 shifting=False)
    T9r, V9r, _ = _expand_nozzle(comp_entry, s["far"], s["Tt9"], s["pt9"], s["p9"],
                                 shifting=True)
    T9i, V9i, comp9i, T_star = _irreversible_fast_expand(comp_entry, s["far"], s["Tt9"],
                                                         s["pt9"], s["p9"])
    put(f"ref/{tag}/T9_frozen", T9f)
    put(f"ref/{tag}/V9_frozen", V9f)
    put(f"ref/{tag}/T9_reversible", T9r)
    put(f"ref/{tag}/V9_reversible", V9r)
    put(f"ref/{tag}/T9_irrev_fast", T9i)
    put(f"ref/{tag}/V9_irrev_fast", V9i)
    put(f"ref/{tag}/T_star", T_star)
    for sp, n in comp9i.items():
        put(f"ref/{tag}/comp_irrev/{sp}", n)

    # `_equilibrate_hp` DIRECTLY, at the same arguments `_irreversible_fast_expand` uses. Dumped
    # separately because a bisection reached only through a caller is a bisection whose own
    # stopping rule is never pinned: if the 1e-10 were transcribed as 1e-11 the composite would
    # still be gated, but the failure would read as a nozzle defect.
    H_entry = _mix_h_abs_B(comp_entry, s["Tt9"])
    comp_star, T_star_direct = _equilibrate_hp(s["far"], H_entry, s["pt9"],
                                               s["Tt9"] - 100.0, s["Tt9"] + 800.0)
    put(f"eqhp/{tag}/T_star", T_star_direct)
    put(f"eqhp/{tag}/H_entry", H_entry)
    for sp, n in comp_star.items():
        put(f"eqhp/{tag}/comp/{sp}", n)
    assert T_star_direct == T_star, "the direct equilibrate_hp disagrees with the composite"

    # The bracket-headroom facts probe 2 measured, as VALUES rather than prose: a later reader
    # tempted to tighten this bracket moves every V9_irrev_fast in the slice.
    put(f"eqhp/{tag}/rise", T_star - s["Tt9"])

# ==============================================================================================
# 2. THE FINITE-RATE MARCH (rung 25) — the Da ladder at nstep=100, plus a 400 refinement
#
#    nstep=100 is the config MINIMUM and the resolution at which the 2nd-law floor has the least
#    margin, so it is where the sweep is broad. The 400 block is the shipped default and is run
#    at three design points to certify the resolution axis itself.
# ==============================================================================================
neg_ds = 0
for tag, _ in DPS:
    s = STATES[tag]
    comp_entry = _equilibrium_composition(s["far"], s["Tt4"], s["pt4"])
    for dtag, da in DA_LADDER:
        T9, V9, comp9, dS = _finite_rate_expand(comp_entry, s["far"], s["Tt9"], s["pt9"],
                                                s["p9"], da, 100)
        put(f"fr100/{tag}/{dtag}/T9", T9)
        put(f"fr100/{tag}/{dtag}/V9", V9)
        put(f"fr100/{tag}/{dtag}/dS", dS)
        for sp, n in comp9.items():
            put(f"fr100/{tag}/{dtag}/comp/{sp}", n)
        if dS < 0.0:
            neg_ds += 1

for tag in ("dp", "warm", "hot"):
    s = STATES[tag]
    comp_entry = _equilibrium_composition(s["far"], s["Tt4"], s["pt4"])
    for dtag, da in [("da03", 0.3), ("da3", 3.0), ("da30", 30.0)]:
        T9, V9, comp9, dS = _finite_rate_expand(comp_entry, s["far"], s["Tt9"], s["pt9"],
                                                s["p9"], da, 400)
        put(f"fr400/{tag}/{dtag}/T9", T9)
        put(f"fr400/{tag}/{dtag}/V9", V9)
        put(f"fr400/{tag}/{dtag}/dS", dS)
        if dS < 0.0:
            neg_ds += 1

# LIVE discrete key: the count of cells where a physically non-negative quantity comes out of the
# truncation with the WRONG SIGN. No tolerance on dS expresses this, and it moves with nstep.
put("census/negative_ds", float(neg_ds))

# ==============================================================================================
# 3. THE ANCHORED CLOCK (rung 26), STANDALONE — including both kill-test hooks
#
#    Solver-free, so it is the cheapest block here and the one that isolates `powp(T, -2.0)`.
#    `_N_HOHM` is a float CONSTANT, so Python reaches libm `pow`; a Rust port that spells it
#    `1.0/(T*T)` is algebraically identical and arithmetically different, and NOTHING else in
#    this dump would localise that to the clock.
# ==============================================================================================
_clock_comp = _equilibrium_composition(STATES["hot"]["far"], STATES["hot"]["Tt4"],
                                       STATES["hot"]["pt4"])
# THE PRESSURE LADDER IS CHOSEN SO THE ARM COUNTS BELOW ARE STRUCTURAL, NOT ACCIDENTAL.
# τ_free ∝ T⁴/p² (`Ea=0` ⇒ `k ∝ T^-n = T²`, times the density² term `∝ (p/T)²`), and with `k(T)`
# pinned τ_killT ∝ (T/p)². So each arm's distinct count is the number of distinct values of ONE
# ratio, and a grid with a repeated ratio silently loses a cell. The round ladder
# {2e4, 5e4, 1.5e5, 6e5, 2.5e6} does exactly that — `(800 K, 2e4 Pa)` and `(2000 K, 5e4 Pa)` share
# `p/T = 25`, so killT held 29 values where the bar said 30. These values are off-round for that
# reason: all 30 `p/T` and all 30 `T²/p` are distinct as exact rationals, so the counts cannot ride
# on a floating-point coincidence that might not survive a change of interpreter.
for i, T in enumerate([800.0, 1100.0, 1400.0, 1700.0, 2000.0, 2300.0]):
    for j, p in enumerate([2.3e4, 5.7e4, 1.43e5, 6.1e5, 2.37e6]):
        put(f"clock/free/{i}/{j}", _tau_chem_recomb(_clock_comp, T, p))
        # kill_T pins k(T) (density alone drives); kill_M pins the density (temperature alone).
        put(f"clock/killT/{i}/{j}", _tau_chem_recomb(_clock_comp, T, p, kill_T=1800.0))
        put(f"clock/killM/{i}/{j}", _tau_chem_recomb(_clock_comp, T, p, kill_M=1.0e-5))

# The x_OH <= 0 branch returns +inf, which `put` refuses by design — so it is dumped as the
# PREDICATE instead. A frozen (zero-radical) mixture is the reduce that makes Da_local -> 0.
_no_oh = {sp: (0.0 if sp == "OH" else n) for sp, n in _clock_comp.items()}
put("clock/no_oh_is_inf", 1.0 if _tau_chem_recomb(_no_oh, 1800.0, 1.0e5) == float("inf") else 0.0)

# ==============================================================================================
# 4. FREEZE-OUT (rung 26) — the anchored march, and the two rate_scale limits
# ==============================================================================================
frozen_from_entry = 0
for tag, _ in DPS:
    s = STATES[tag]
    st = s["gas"].freeze_out_nozzle(s["far"], s["Tt4"], s["pt4"], s["Tt9"], s["pt9"], s["p9"],
                               FreezeOut())
    put(f"fz/{tag}/T9", st.T9_freeze)
    put(f"fz/{tag}/V9", st.V9_freeze)
    put(f"fz/{tag}/dS", st.dS_freeze)
    put(f"fz/{tag}/s_freeze", st.s_freeze)
    put(f"fz/{tag}/Da_entry", st.Da_entry)
    put(f"fz/{tag}/Da_exit", st.Da_exit)
    put(f"fz/{tag}/co_entry", st.co_fraction_entry)
    put(f"fz/{tag}/co_exit", st.co_fraction_freeze_exit)
    put(f"fz/{tag}/bracket_filled", st.bracket_filled)
    if st.frozen_from_entry:
        frozen_from_entry += 1

# LIVE discrete key: rung 26's own dormant-lean claim as an integer, and it MOVES with Tt4.
put("census/frozen_from_entry", float(frozen_from_entry))

# The two limit gates. rate_scale -> 0 drives Da_local -> 0 (frozen, F); -> inf drives the
# irreversible-fast ceiling (I). Neither is the rung-25 reduce — the schedule still varies with
# (T, p) — which is exactly why the CONSTANT-Da reduce below is a separate block.
for tag in ("dp", "hot"):
    s = STATES[tag]
    for ltag, rs in (("slow", 1e-5), ("fast", 1e5)):
        st = s["gas"].freeze_out_nozzle(s["far"], s["Tt4"], s["pt4"], s["Tt9"], s["pt9"], s["p9"],
                                   FreezeOut(rate_scale=rs))
        put(f"fzlim/{tag}/{ltag}/V9", st.V9_freeze)
        put(f"fzlim/{tag}/{ltag}/T9", st.T9_freeze)
        put(f"fzlim/{tag}/{ltag}/s_freeze", st.s_freeze)

# ==============================================================================================
# 5. THE CONSTANT-Da REDUCE (§ 4.11 probe 4) — BOTH sides, in the TSV
#
#    `_freeze_out_expand` with a constant `da_local_fn` reproduces `_finite_rate_expand(Da)` to
#    the ULP. Measured 40/40 on the Python. The RUST-side equality is gated in `tests/rung26.rs`
#    rather than here, because a Python<->Rust dump compares VALUES and cannot see a loop-shape
#    error transcribed identically into both copies. What this block adds is that the Python's own
#    equality is pinned as data, so the Rust gate is checking the same claim and not a weaker one.
# ==============================================================================================
for tag in ("dp", "hot"):
    s = STATES[tag]
    comp_entry = _equilibrium_composition(s["far"], s["Tt4"], s["pt4"])
    for dtag, da in [("da05", 0.5), ("da2", 2.0), ("da300", 300.0)]:
        a = _finite_rate_expand(comp_entry, s["far"], s["Tt9"], s["pt9"], s["p9"], da, 100)
        b = _freeze_out_expand(comp_entry, s["far"], s["Tt9"], s["pt9"], s["p9"],
                               (lambda d: (lambda c, T, p: d))(da), 100)
        put(f"red/{tag}/{dtag}/fr_T9", a[0])
        put(f"red/{tag}/{dtag}/fz_T9", b[0])
        put(f"red/{tag}/{dtag}/fr_V9", a[1])
        put(f"red/{tag}/{dtag}/fz_V9", b[1])
        put(f"red/{tag}/{dtag}/fr_dS", a[3])
        put(f"red/{tag}/{dtag}/fz_dS", b[3])
        assert a[0] == b[0] and a[1] == b[1] and a[3] == b[3], \
            f"the constant-Da reduce is NOT bit-exact at {tag}/{dtag} — probe 4 said it is"
        assert all(a[2][sp] == b[2][sp] for sp in a[2]) and list(a[2]) == list(b[2]), \
            f"the constant-Da reduce's COMPOSITION differs at {tag}/{dtag}"
        # A constant Da_local never crosses 1 downward unless it starts below it, so s_freeze is
        # a two-valued function of Da here — dumped because it is the one output the reduce does
        # NOT share with rung 25, and therefore the one that proves the two ran different code.
        put(f"red/{tag}/{dtag}/fz_s_freeze", b[4])

# ==============================================================================================
# 6. THE RUNG-25 STATE OBJECT — the assembled diagnostic, including its derived properties
# ==============================================================================================
for tag in ("dp", "hot"):
    s = STATES[tag]
    st = s["gas"].finite_rate_nozzle(s["far"], s["Tt4"], s["pt4"], s["Tt9"], s["pt9"], s["p9"],
                                FiniteRate(Da=3.0, nstep=100))
    put(f"state/{tag}/V9_frozen", st.V9_frozen)
    put(f"state/{tag}/V9_finite", st.V9_finite)
    put(f"state/{tag}/V9_irrev_fast", st.V9_irrev_fast)
    put(f"state/{tag}/V9_reversible", st.V9_reversible)
    put(f"state/{tag}/T_star_entry", st.T_star_entry)
    put(f"state/{tag}/dS_finite", st.dS_finite)
    put(f"state/{tag}/attainable_gap", st.attainable_gap)
    put(f"state/{tag}/unreachable_gap", st.unreachable_gap)
    put(f"state/{tag}/finite_filled", st.finite_filled)
    put(f"state/{tag}/co_entry", st.co_fraction_entry)
    put(f"state/{tag}/co_exit", st.co_fraction_finite_exit)

# ==============================================================================================
# 7. DISTINCT-ROOT COUNTS — asserted here so the claim cannot silently collapse into one root
#    wearing many costumes (§ 4.2's "19 measurements in a 114 costume").
# ==============================================================================================
march_roots = {bits for key, bits, _ in ROWS
               if key.startswith(("fr100/", "fr400/", "fz/")) and key.endswith("/T9")}
ref_roots = {bits for key, bits, _ in ROWS
             if key.startswith("ref/") and "/T9_" in key}
print(f"[7] distinct roots: march exits {len(march_roots)}, reference exits {len(ref_roots)}")
assert len(march_roots) >= 40, f"only {len(march_roots)} distinct march exit roots"
assert len(ref_roots) >= 15, f"only {len(ref_roots)} distinct reference exit roots"
put("roots/march_distinct", float(len(march_roots)))
put("roots/reference_distinct", float(len(ref_roots)))

# The clock's three arms are counted SEPARATELY, and the counts are MEASURED rather than guessed.
# A first draft asserted one lumped bar of 80 over all three and failed at 66 — because the killM
# arm MUST collapse: pinning the density in [OH]·[M] removes the only p-dependence the clock has,
# so its 6x5 grid holds 6 distinct values, not 30. That is the kill test working, and a lumped bar
# would have hidden a real structural fact behind a threshold nobody had checked. Per arm:
#   free   6 T x 5 p, both live                       -> 30
#   killT  k(T) pinned; c_tot(T,p) still live         -> 30  (1800 K is deliberately off the ladder,
#                                                             so no cell coincides with `free`)
#   killM  density pinned; T-dependence alone         ->  6
for arm, want in (("free", 30), ("killT", 30), ("killM", 6)):
    vals = {bits for key, bits, _ in ROWS if key.startswith(f"clock/{arm}/")}
    assert len(vals) == want, f"clock/{arm}: {len(vals)} distinct values, expected {want}"
    put(f"roots/clock_{arm}_distinct", float(len(vals)))
# The three ARMS only — `clock/no_oh_is_inf` is a predicate sharing the prefix, and lumping it in
# is what made a first draft's total read 66 when the arms held 65.
_all_clock = {bits for key, bits, _ in ROWS
              if key.startswith(("clock/free/", "clock/killT/", "clock/killM/"))}
assert len(_all_clock) == 66, f"clock arms overlap: {len(_all_clock)} distinct, expected 66"
print("[7] clock arms: free 30, killT 30, killM 6 — 66 distinct, no cross-arm collision")

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-4F recombination-march oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
