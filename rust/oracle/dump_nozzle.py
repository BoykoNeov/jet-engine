"""THE ORACLE, phase 3 slice E — every rung-14/17 NOZZLE-STRAND value the Rust must reproduce.

The seventh in the family (`dump_gas.py` → `dump_cycle.py` → `dump_nox.py` → `dump_quench.py` →
`dump_pdf.py` → `dump_spatial.py` → here). A separate file again, for the usual reason: the earlier
TSVs stay frozen as their own audit trail, and each dump's regeneration cost stays proportional to
what it certifies.

WHAT IS ACTUALLY NEW HERE, and therefore what the sweep is built around:

  * A THIRD ROUTE TO A NUMBER THE CYCLE ALREADY COMPUTES. `_expand_nozzle`'s FROZEN branch is the
    production nozzle re-derived on the molar entropy scale — `Σ nᵢ[s0ᵢ(T) − Ru ln(xᵢ p/p0)]`
    bisected against the entry entropy, where production runs `t_from_pr`'s safeguarded Newton on
    `antideriv_phi` of the MOLE-WEIGHTED coefficients. The docstring says the two agree "EXACTLY".
    **Measured (§ 4.9 of the plan): they never do.** Worst 2.46e-11 m/s in V9 over eight design
    points, 0/8 bit-equal — and driving the bisection to FULL convergence (all 200 halvings, no
    stopping rule) leaves 2.05e-12 K standing. So the stopping rule is a factor 4–8 and the FLOOR
    is the route. The `conv/` keys carry that converged residual so the port must reproduce the
    inexactness, exactly as slice D gates rung 24's hierarchical sum from both sides.
  * A BISECTION WHOSE LOOP SHAPE IS THE HAZARD. `for _ in range(200)`, midpoint at the TOP, bracket
    updated, then `if hi - lo <= 1e-13 * T: break` on THIS iteration's PRE-update midpoint, and
    `T9 = 0.5*(lo+hi)` computed AFTER the loop from the final bracket. An idiomatic
    `while hi-lo > tol` rewrite gets three things wrong at once and each is worth one bracket
    quantum. The `iters/` keys are the halving count — see § the discrete keys for what they are
    and are NOT.
  * A GUARD THAT IS ACTUALLY REACHABLE. `_expand_nozzle`'s post-loop assert says the 500 K bracket
    floor "never happens here (every exit sits >700 K)". True at shipped conditions, and MEASURED
    to fire below `p9/pt9` = 0.025016 at the cool design point and 0.002608 at the hot one — 6.4×
    and 44.9× below where the engine runs. So unlike rung 20's dormant flame-band floor (which
    needed a second, cooler design point before its gate meant anything) and unlike slice D's knot
    count (which could not fire at all), this one is dumped from BOTH sides: a census of a fixed
    back-pressure ladder counting how many rungs of it the guard rejects.
  * A LADDER OF THREE COMBUSTOR-MIXING FIDELITIES SHARING ONE NOZZLE DENOMINATOR — and the ratio
    that makes the whole rung interesting is a COMPARISON, not a value. `a_mixed < a_bulk <
    a_pocket` with `a_mixed < 1 < a_bulk` is the headline; each `a` is a mole fraction over the
    same `x_no_e(T9)`.

THE SIZING LEVER, and it is the same shape slice B's was. `nozzle_flow` reads only
`(far, Tt4, pt4, Tt9, pt9, p9)` — NOT `phi_primary`, NOT `mixing`, NOT `pocket_quench`, NOT
`super_eq_o`, NOT any grid. So T9 and the COMMON clamp denominator `x_no_e(T9)` are ONE call for
the entire φ_p × J × C_e × super_eq_o sweep, and the three NO numerators sweep against a cached
denominator. Measured on PyPy: `nozzle_flow` 8 ms, a bulk `zoned_nox` 0.13 s, a full
`exhaust_no_clamp` 1.95 s at the source's own coarse grids. Without the lever the band-edge sweep
in § 6 pays a nozzle solve per J point that cannot move.

THE DISCRETE KEYS, and what each one is honestly worth:

  * `guard/…/fires` — the number of back-pressure ratios in a fixed ladder at which the frozen
    branch's floor guard rejects. LIVE: it moves with the design point (6 of 12 at the cool point,
    3 of 12 at the hot one) and no tolerance on T9 expresses "the solve was refused". The census
    runs on the FROZEN branch alone, deliberately: the shifting branch would additionally reach the
    equilibrium Newton's own asserts below the floor, and then the count would be measuring two
    guards at once.
  * `iters/…` — the bisection's halving count. **A NAMING KEY, NOT AN INDEPENDENT DISCRIMINATOR,
    and it is recorded as one.** T9 is gated at BIT-EQUALITY, so a mis-shaped loop is already
    detected by the value; what the count adds is that the failure reads "the loop ran 47 halvings
    instead of 44" rather than "T9 differs in the last two bits". Slice D's knot count could claim
    more than this (nothing else could see it); this one cannot, so it does not claim it. To keep
    the transcription from drifting, the loop below ASSERTS it reproduces `_expand_nozzle`'s own T9
    bit-for-bit before the count is recorded.
  * `edge/…/first_dormant` — the index of the first J in a fixed ladder at which `a_bulk` falls
    below 1. This is the rung-17 firing band EDGE, which the source states exists and never
    measures. It is a live integer: it sits at index 7 for C_e=0.20 and index 8 for C_e=0.15 on the
    ladder below.

THE ONE PLACE THIS DUMP CONTRADICTS THE SOURCE'S OWN READING (§ 4.9 probe 4). The
`exhaust_no_clamp` docstring says a fast enough quench (J→∞) drives `a_bulk → a_mixed < 1`
(dormant). Correct — and INCOMPLETE as a statement about the ladder: `a_pocket` RISES over the same
sweep (11.06 → 12.82 → 14.34 at J = 225 / 4000 / 16000) because `ei_no_pocket_quench` = the
mean-field bulk (riding `τ_mean ∝ 1/√J`, collapsing) + a β-PDF integral at
`τ_core = τ_res(1+b_u·u)`, which `PocketQuenchPDF.core_dwell`'s own docstring calls an ABSOLUTE
residence whose penalty "survives J→∞". Both branches are dumped so the port gates the
reconciliation rather than the half of it the source's tests cover.
"""

import math
import os
import struct
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
from turbojet import gas as G
from turbojet.engine import FlightCondition, build_turbojet

ROWS = []
T0 = time.time()


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
TAU = 3e-3
HF = G._HF_FUEL_DEFAULT
PHI_P = 1.5          # the RQL rich primary — where the mixed-out shortcut HIDES the NO (rung 17)
CE = 0.20            # the ANCHORED jet-entrainment regime (rungs 11-16)
# The rung-17 suite's own coarse grids: per-pocket quench is the cost driver, DIRECTION not digits.
NB, NQ = 20, 64      # per-pocket ξ-grid / β-PDF quadrature nodes
NG, NSTEPS = 24, 200 # finite-quench trajectory points / RK4 steps

# (tag, Tt4, losses, mdot). Tags are LITERAL strings, never a formatted float, so the Rust side
# cannot disagree about how Python spells a number in a key.
DPS = [
    ("cool", 1300.0, True, 1.0),      # the dissociation→0 reduce (rung-14 gate 2)
    ("dp", 1500.0, True, 1.0),        # the shipped design point
    ("warm", 1800.0, True, 1.0),
    ("hot", 2200.0, True, 1.0),
    ("cool0", 1300.0, False, 1.0),    # losses OFF — the rung-14 magnitude gate runs on these
    ("dp0", 1500.0, False, 1.0),
    ("warm0", 1800.0, False, 1.0),
    ("hot0", 2200.0, False, 1.0),
]
SPECIES_ORDER = list(G._SP_REACT) + ["N2", "Ar"]

_CACHE = {}


def dp(tag):
    """Build the equilibrium engine once per design point and read what rung 14/17 ride on."""
    if tag not in _CACHE:
        _, Tt4, losses, mdot = next(d for d in DPS if d[0] == tag)
        g = G.Gas.reacting_equilibrium()
        kw = LOSSES if losses else {}
        r = build_turbojet(g, PI_C, Tt4, FLIGHT.p0, **kw).run(FLIGHT, mdot)
        s3, s4, s9 = r.stations["3"], r.stations["4"], r.stations["9"]
        _CACHE[tag] = dict(g=g, r=r, far=s4.far, Tt3=s3.Tt, Tt4=s4.Tt, pt4=s4.pt,
                           Tt9=s9.Tt, pt9=s9.pt, p9=r.p9, V9=r.V9, T9=r.T9)
    return _CACHE[tag]


# ==============================================================================================
# 1. THE MIXTURE PRIMITIVES, SOLVER-FREE — so a defect localises to a sum, not to "V9 differs".
#
# All three are plain Σ over the composition in SP_REACT + N2/Ar order. `_mix_entropy_molar` is
# the one with real structure: it SKIPS n ≤ 0 and takes `log((n/ntot)*p/p0)` — one `math.log` per
# species, and the mixing term is what cancels at fixed composition (which is WHY the frozen
# branch reduces to a pr-ratio at all).
# ==============================================================================================
print("[1] mixture primitives")
for tag in ("dp", "hot"):
    d = dp(tag)
    comp = G._equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    for sp in SPECIES_ORDER:
        put(f"prim/{tag}/comp/{sp}", comp[sp])
    put(f"prim/{tag}/mass_per_air", G._mix_mass_per_air(comp))
    # Sweep T and p independently: the entropy carries BOTH, the enthalpy only T.
    for ttag, T in (("entry", d["Tt9"]), ("mid", 900.0), ("exit", 700.0)):
        put(f"prim/{tag}/h_absB/{ttag}", G._mix_h_abs_B(comp, T))
        for ptag, p in (("pt9", d["pt9"]), ("p9", d["p9"]), ("bar", G._P_REF)):
            put(f"prim/{tag}/S/{ttag}/{ptag}", G._mix_entropy_molar(comp, T, p))
    # The equilibrium-NO fraction the clamp denominator is built from (rung 7 primitive, re-read
    # here at nozzle temperatures — three decades colder than anything rung 7 evaluates it at).
    for ttag, T in (("entry", d["Tt9"]), ("mid", 900.0), ("exit", 700.0)):
        put(f"prim/{tag}/x_no_e/{ttag}", G._equilibrium_no_fraction(comp, T))

# ==============================================================================================
# 2. THE FROZEN / SHIFTING EXPANSION — rung 14's bracket, at every design point.
# ==============================================================================================
print(f"[2] the expansion, 8 design points  ({time.time() - T0:.1f}s)")
for tag, *_ in DPS:
    d = dp(tag)
    nf = d["g"].nozzle_flow(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"])
    put(f"dp/{tag}/far", d["far"])
    put(f"dp/{tag}/Tt9", d["Tt9"])
    put(f"dp/{tag}/pt9", d["pt9"])
    put(f"dp/{tag}/p9", d["p9"])
    put(f"dp/{tag}/V9_cycle", d["V9"])
    put(f"dp/{tag}/T9_cycle", d["T9"])
    put(f"nz/{tag}/T9_frozen", nf.T9_frozen)
    put(f"nz/{tag}/T9_eq", nf.T9_equilibrium)
    put(f"nz/{tag}/V9_frozen", nf.V9_frozen)
    put(f"nz/{tag}/V9_eq", nf.V9_equilibrium)
    put(f"nz/{tag}/dV9", nf.dV9)
    put(f"nz/{tag}/dV9_frac", nf.dV9_frac)
    put(f"nz/{tag}/co_frac_entry", nf.co_fraction_entry)
    for sp in SPECIES_ORDER:
        put(f"nz/{tag}/exit_eq/{sp}", nf.comp_exit_eq[sp])
    # the clamp corollary — frozen-NO-INDEPENDENT half
    put(f"clamp/{tag}/x_no_e_entry", nf.x_no_e_entry)
    put(f"clamp/{tag}/x_no_e_exit", nf.x_no_e_exit)
    put(f"clamp/{tag}/collapse", nf.no_collapse_ratio)

# ==============================================================================================
# 3. THE CONVERGED-BRACKET RESIDUAL — the § 4.9 probe-1 measurement, dumped so the port must
#    reproduce the INEXACTNESS. Two numbers per design point:
#      shipped/  — production's frozen T9 minus the cycle's, at the shipped 1e-13·T stopping rule
#      conv/     — the same with the bracket driven to FULL convergence (all 200 halvings)
#    The transcribed loop ASSERTS it reproduces `_expand_nozzle` bit-for-bit at the shipped
#    tolerance before any count or converged value is recorded, so the transcription cannot drift
#    away from the function it is instrumenting.
# ==============================================================================================
print(f"[3] the frozen reduce, both stopping rules  ({time.time() - T0:.1f}s)")


def frozen_bisect(comp_entry, Tt9, pt9, p9, tol_rel):
    """`_expand_nozzle`'s FROZEN branch with the stopping rule made a knob. Returns (T9, iters).

    Transcribed LINE FOR LINE: midpoint at the top, bracket updated, break tested on THIS
    iteration's pre-update midpoint with `<=`, and T9 taken from the FINAL BRACKET after the loop.
    """
    S_entry = G._mix_entropy_molar(comp_entry, Tt9, pt9)
    lo, hi = G._T_EXIT_FLOOR, Tt9
    iters = 0
    for _ in range(200):
        T = 0.5 * (lo + hi)
        iters += 1
        if G._mix_entropy_molar(comp_entry, T, p9) > S_entry:
            hi = T
        else:
            lo = T
        if hi - lo <= tol_rel * T:
            break
    return 0.5 * (lo + hi), iters


bit_equal_shipped = 0
for tag, *_ in DPS:
    d = dp(tag)
    comp = G._equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    T9s, iters = frozen_bisect(comp, d["Tt9"], d["pt9"], d["p9"], 1e-13)
    T9prod, _, _ = G._expand_nozzle(comp, d["far"], d["Tt9"], d["pt9"], d["p9"], shifting=False)
    assert T9s == T9prod, f"{tag}: the transcribed frozen loop drifted from _expand_nozzle"
    T9c, iters_c = frozen_bisect(comp, d["Tt9"], d["pt9"], d["p9"], 0.0)
    put(f"iters/{tag}/shipped", iters)
    put(f"iters/{tag}/converged", iters_c)
    put(f"resid/{tag}/shipped", T9s - d["T9"])       # ABSOLUTE bar — a difference of near-equals
    put(f"resid/{tag}/converged", T9c - d["T9"])
    put(f"conv/{tag}/T9", T9c)
    if T9s == d["T9"]:
        bit_equal_shipped += 1
# The COUNT, not a law: slice D's finding 7 is that a first gate stating "never bit-equal" as a
# universal was refuted by a wider sweep. Dumped so the gate reports what was measured.
put("resid/bit_equal_count", float(bit_equal_shipped))

# ==============================================================================================
# 4. THE BACK-PRESSURE SWEEP AND THE GUARD CENSUS — both sides of the 500 K exit floor.
#
# The ladder is fixed and spans it: at the cool design point the guard rejects the bottom half, at
# the hot one only the bottom quarter. Values are dumped for the ratios that SOLVE; the census
# counts the ones that do not. Frozen branch only — see § the discrete keys.
# ==============================================================================================
print(f"[4] the back-pressure ladder + guard census  ({time.time() - T0:.1f}s)")
P9_RATIOS = [("r999", 0.999), ("r900", 0.9), ("r500", 0.5), ("r250", 0.25), ("r159", 0.159),
             ("r100", 0.1), ("r050", 0.05), ("r030", 0.03), ("r020", 0.02), ("r010", 0.01),
             ("r005", 0.005), ("r001", 0.001)]
for tag in ("dp", "hot"):
    d = dp(tag)
    comp = G._equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    fires = 0
    for rtag, ratio in P9_RATIOS:
        p9 = d["pt9"] * ratio
        try:
            T9f, V9f, _ = G._expand_nozzle(comp, d["far"], d["Tt9"], d["pt9"], p9, shifting=False)
        except AssertionError:
            fires += 1
            continue
        put(f"bp/{tag}/{rtag}/T9", T9f)
        put(f"bp/{tag}/{rtag}/V9", V9f)
    put(f"guard/{tag}/fires", float(fires))
    put(f"guard/{tag}/ladder", float(len(P9_RATIOS)))

# ==============================================================================================
# 5. THE RUNG-17 LADDER — three combustor-mixing fidelities through ONE nozzle denominator.
#
# THE SIZING LEVER IN USE: `nf` below is computed ONCE per design point and every `a` in this
# section divides by its `x_no_e_exit`. The three numerators are the expensive part.
# ==============================================================================================
print(f"[5] the rung-17 ladder  ({time.time() - T0:.1f}s)")
# rung 17's own design point: the shipped Tt4 = 1500 K case at mdot = 50.
g17 = G.Gas.reacting_equilibrium()
r17 = build_turbojet(g17, PI_C, 1500.0, FLIGHT.p0, **LOSSES).run(FLIGHT, 50.0)
s3, s4, s9 = r17.stations["3"], r17.stations["4"], r17.stations["9"]
D17 = dict(far=s4.far, Tt3=s3.Tt, Tt4=s4.Tt, p=s4.pt, Tt9=s9.Tt, pt9=s9.pt, p9=r17.p9)
for k, v in D17.items():
    put(f"r17/dp/{k}", v)


def mix17(J, C_e=CE):
    return G.JetMixing(J=J, C_e=C_e, shape_n=2.0)


def pq17():
    return G.PocketQuenchPDF(S=0.0625, C_opt=2.5, k_g=0.3, g_max=0.3,
                             tau_res=2.5e-3, b_u=3.0, n_bell=NB, n_quad=NQ)


NF17 = g17.nozzle_flow(D17["far"], D17["Tt4"], D17["p"], D17["Tt9"], D17["pt9"], D17["p9"])
put("r17/nozzle/T9", NF17.T9_frozen)
put("r17/nozzle/x_no_e_exit", NF17.x_no_e_exit)
put("r17/nozzle/collapse", NF17.no_collapse_ratio)


def dump_clamp(tag, phi_p, J, C_e=CE, super_eq_o=False):
    t = time.time()
    s = g17.exhaust_no_clamp(D17["far"], D17["Tt3"], D17["Tt4"], D17["p"],
                             D17["Tt9"], D17["pt9"], D17["p9"],
                             phi_primary=phi_p, mixing=mix17(J, C_e), pocket_quench=pq17(),
                             tau=TAU, super_eq_o=super_eq_o,
                             quench_ngrid=NG, quench_nsteps=NSTEPS)
    put(f"r17/{tag}/T9", s.T9)
    put(f"r17/{tag}/x_no_e_exit", s.x_no_e_exit)
    put(f"r17/{tag}/collapse", s.no_collapse_ratio)
    put(f"r17/{tag}/x_no_mixed", s.x_no_mixed_out)
    put(f"r17/{tag}/x_no_bulk", s.x_no_bulk_quench)
    put(f"r17/{tag}/x_no_pocket", s.x_no_pocket)
    put(f"r17/{tag}/a_mixed", s.a_mixed_out)
    put(f"r17/{tag}/a_bulk", s.a_bulk_quench)
    put(f"r17/{tag}/a_pocket", s.a_pocket)
    put(f"r17/{tag}/ei_bulk", s.ei_no_quenched)
    put(f"r17/{tag}/ei_pocket", s.ei_no_pocket_quench)
    put(f"r17/{tag}/gap", s.gap_pocket_over_bulk)
    put(f"r17/{tag}/max_a_quench", s.max_a_quench)
    put(f"r17/{tag}/hides", 1.0 if s.hides_super_eq else 0.0)
    put(f"r17/{tag}/monotone", 1.0 if s.ladder_monotone else 0.0)
    print(f"   clamp {tag}: a=({s.a_mixed_out:.4f},{s.a_bulk_quench:.4f},{s.a_pocket:.4f}) "
          f"[{time.time() - t:.1f}s]")
    return s


dump_clamp("J225", PHI_P, 225.0)                    # the shipped RQL point
dump_clamp("J225/ce15", PHI_P, 225.0, C_e=0.15)     # the scale-sensitivity arm
dump_clamp("J225/su", PHI_P, 225.0, super_eq_o=True)  # rung 20's deferred gate 4
dump_clamp("J25", PHI_P, 25.0)                      # BELOW the g_max clip (g = 0.1875, not pinned)
dump_clamp("J4000", PHI_P, 4000.0)                  # PAST the a_bulk crossing — hides goes FALSE
dump_clamp("J16000", PHI_P, 16000.0)                # deeper still — a_pocket still RISING
dump_clamp("phi10", 1.0, 225.0)                     # the rung-14 CONTRAST: lean primary FIRES

# The rung-14 contrast built the way the rung-17 suite builds it — mixed-out straight through the
# nozzle, no jet at all. `max_a` is `x_no_mix / x_no_e(T9)`, so this is the SAME construction the
# ladder's bottom rung is, and gate 2 of the Python suite pins the two together.
for ptag, phi in (("phi10", 1.0), ("phi15", 1.5)):
    zn = g17.zoned_nox(D17["far"], D17["Tt3"], D17["Tt4"], D17["p"], phi, TAU)
    nf = g17.nozzle_flow(D17["far"], D17["Tt4"], D17["p"], D17["Tt9"], D17["pt9"], D17["p9"],
                         x_no_frozen=zn.x_no_mix)
    put(f"r14c/{ptag}/x_no_mix", zn.x_no_mix)
    put(f"r14c/{ptag}/max_a", nf.max_a)

# ==============================================================================================
# 6. THE FIRING BAND EDGE — a_bulk vs J, on the CHEAP path.
#
# The source says the firing "holds across the RQL J-band but is NOT universal — as the quench
# gets FAST (J→∞) x_no_quenched→x_no_mix … so a_bulk→a_mixed<1". It never measures where. This
# sweeps the BULK numerator only (0.13 s a point) against the cached denominator, at two C_e, and
# dumps the INDEX of the first ladder rung at which a_bulk < 1 — a live integer that moves with an
# un-pinned entrainment scale.
# ==============================================================================================
print(f"[6] the firing band edge  ({time.time() - T0:.1f}s)")
J_LADDER = [("J25", 25.0), ("J100", 100.0), ("J225", 225.0), ("J400", 400.0), ("J625", 625.0),
            ("J1000", 1000.0), ("J2000", 2000.0), ("J2500", 2500.0), ("J4000", 4000.0),
            ("J8000", 8000.0), ("J16000", 16000.0)]
XE17 = NF17.x_no_e_exit
zn_mixed = g17.zoned_nox(D17["far"], D17["Tt3"], D17["Tt4"], D17["p"], PHI_P, TAU)
put("edge/a_mixed", zn_mixed.x_no_mix / XE17)       # jet-INDEPENDENT: one value for the whole sweep
for ctag, C_e in (("ce20", 0.20), ("ce15", 0.15)):
    first_dormant = len(J_LADDER)
    for i, (jtag, J) in enumerate(J_LADDER):
        zb = g17.zoned_nox(D17["far"], D17["Tt3"], D17["Tt4"], D17["p"], PHI_P, TAU,
                           mixing=mix17(J, C_e), quench_ngrid=NG, quench_nsteps=NSTEPS)
        a = zb.x_no_quenched / XE17
        put(f"edge/{ctag}/{jtag}/a_bulk", a)
        put(f"edge/{ctag}/{jtag}/ei_bulk", zb.ei_no_quenched)
        if a < 1.0 and first_dormant == len(J_LADDER):
            first_dormant = i
    put(f"edge/{ctag}/first_dormant", float(first_dormant))

# ==============================================================================================
# 7. DISTINCT-ROOT COUNTS — asserted here so the claim cannot silently collapse into one root
#    wearing many costumes (§ 4.2's "19 measurements in a 114 costume").
# ==============================================================================================
frozen_roots = {bits for key, bits, _ in ROWS if "/T9_frozen" in key or key.startswith("bp/")
                and key.endswith("/T9")}
eq_roots = {bits for key, bits, _ in ROWS if key.endswith("/T9_eq")}
print(f"[7] distinct roots: frozen {len(frozen_roots)}, shifting {len(eq_roots)}")
assert len(frozen_roots) >= 20, f"only {len(frozen_roots)} distinct frozen exit roots"
assert len(eq_roots) >= 8, f"only {len(eq_roots)} distinct shifting exit roots"
put("roots/frozen_distinct", float(len(frozen_roots)))
put("roots/shifting_distinct", float(len(eq_roots)))

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-3E nozzle-strand oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
