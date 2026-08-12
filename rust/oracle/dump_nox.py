"""THE ORACLE, phase 3 slice A — dump every rung-7/8/9/19 NOx value the Rust must reproduce.

The third in the family (`dump_gas.py` → `dump_cycle.py` → here), and the first to probe a
DIAGNOSTIC layer rather than the cycle. Everything below rides the rung-6 equilibrium solve
that phase 1 measured bit-exact, so what is genuinely new here is:

  * the extended-Zeldovich integrator — fixed-step RK4, 4000 steps, NO adaptive control, so
    (unlike the burner) it carries no stopping rule at all;
  * TWO new solvers that DO — `_primary_aft` and `_mixed_out_T`, both bisections on a
    monotone scale-A enthalpy whose INNER evaluation is the 8-species Newton. That nesting is
    the deepest in the project, and phase 1 named stopping rules as the port's whole residual
    risk (§ 4.1), so the sweep below exists to give those two roots a real spread.

Sized the way `dump_cycle.py` sized the burner: by DISTINCT ROOTS, not by row count. A
headline like "the AFT reproduces 300/300" is worthless if the 300 are six roots repeated;
`nox_oracle.rs` asserts the distinct-root counts so they cannot silently collapse.

SHAPE KEYS. Rung 9's finding is WHERE the EI-vs-φ bell peaks, not what its peak is, which
puts it on the plan's § 4.2 register of claims a tolerance cannot cover. So the dump carries
the bell's ARGMAX as its own key, beside the whole curve. If the bar holds at bit-equality
the argmax costs nothing; if phase 3 ever falls back to a tolerance, it is the only thing
that makes "did the peak MOVE?" answerable at all.

Single-use by design (docs/plans/todo-rust-port.md): it validates the Rust and is deleted at
phase 8. It reaches into `turbojet.gas`'s private names on purpose — it is not an API
consumer, it is a reference dump.

Output is TSV, one row per value:  key <TAB> u64-bits <TAB> repr

Run under BOTH interpreters. The project already ships on two (the gate runs PyPy, the
fingerprint goldens are CPython), so whatever PyPy and CPython disagree by is a deviation the
project ALREADY tolerates -- that gap is the principled tolerance floor, not an invented one.

    C:\\Python314\\python.exe rust/oracle/dump_nox.py rust/oracle/nox_cpython.tsv
    .venv\\Scripts\\python.exe  rust/oracle/dump_nox.py rust/oracle/nox_pypy.tsv
"""
import os
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

from turbojet import gas as G
from turbojet.engine import FlightCondition, build_turbojet

ROWS = []


def put(key, value):
    """Record one float. Rejects non-finite so a NaN cannot slip through as 'equal'."""
    v = float(value)
    assert v == v and abs(v) != float("inf"), f"{key} is not finite: {v}"
    ROWS.append((key, struct.unpack("<Q", struct.pack("<d", v))[0], repr(v)))


# --- the grids --------------------------------------------------------------------------
# The FLAME band, because that is the only place NO chemistry is evaluated. 1500.0 sits in
# the grid exactly: it is `_SUPER_EQ_T_FLOOR`, which rung 20 will need and which is a natural
# place for a comparison to be spelled one way in one language and another in the other.
# 1000.0 is the polynomial join — every `_g_molar` under `_kp_no` crosses it.
T_FLAME = [1000.0, 1200.0, 1400.0, 1500.0, 1600.0, 1800.0, 2000.0, 2200.0, 2400.0,
           2441.540385130793, 2600.0, 2800.0, 3000.0]
# Local fuel/air ratios: lean cycle values up through the RICH primary at the soot bound
# φ=2 (rung 9's scope). f_stoich ≈ 0.0677, so 0.1354 is φ=2 exactly.
FAR_LOCAL = [0.005, 0.01, 0.0204, 0.02717919071928212, 0.04, 0.0677, 0.08, 0.1, 0.1354]
# Pressures: the combustor operating range, from a low-altitude idle to a high-π_c cruise.
P_GRID = [101325.0, 300000.0, 747441.4730230813, 1.5e6, 3.0e6]


# --- SECTION 1: the pure T-functions (no solver, no composition) --------------------------
# k(T) = A·T^n·exp(-θ/T)·1e-6, four LEFT-ASSOCIATED factors. The `n` here are 0.0 and 1.0,
# where pow is exact for any spelling — but the transcription is `T ** n` off a table, so the
# Rust spells it `powp(t, n)` and this grid is what would catch it if a future n were not.
for key in ("1f", "1r", "2f", "2r", "3f", "3r"):
    for T in T_FLAME:
        put(f"kz/{key}/{T!r}", G._k_zeldovich(key, T))
for T in T_FLAME:
    put(f"kcheck/{T!r}", G._kcheck_ratio(T))          # the rung-7 standing assert's own number
    put(f"kpno/{T!r}", G._kp_no(T))                   # Kp(½N₂+½O₂⇌NO) off the a6/a7 substrate
    put(f"mO/{T!r}", G._super_eq_o_multiplier(T))     # rung 19's Westenberg ratio
    put(f"hairA/{T!r}", G._h_air_molar_A(T))          # scale-A molar enthalpy of 1 mol air

# --- SECTION 2: equilibrium NO on the frozen rung-6 pool ---------------------------------
# x_NO_e = Kp·√(x_N2·x_O2) — the first place a composition is SUMMED (`ntot`), so it is the
# first place the species ORDER of the ported composition shows up in the last bit.
for far in FAR_LOCAL:
    for T in (1600.0, 2000.0, 2400.0, 2800.0):
        comp = G._equilibrium_composition(far, T, 1.5e6)
        put(f"xnoeq/{far!r}/{T!r}", G._equilibrium_no_fraction(comp, T))
        put(f"ntot/{far!r}/{T!r}", sum(comp.values()))

# --- SECTION 3: the extended-Zeldovich integrator, driven directly ------------------------
# Every NOxState field, over (far, T, p, tau, m). The RK4 has no stopping rule, so what this
# measures is ACCUMULATION ORDER: 4000 steps of `cNO += dt/6·(k1+2k2+2k3+k4)`, which is a
# different function from `(k1+2k2+2k3+k4)·dt/6` in the last bit and drifts over 4000 of them.
#
# THE GRID IS CLIPPED, and by the model's OWN declared scope rather than by convenience:
# `_thermal_no` asserts x_NO_e < 0.02 (NO must be trace for the decoupled diagnostic to be
# valid), and section 2 above measures exactly where that binds — the LEAN-and-HOT corner
# (far 0.027 at 2800 K gives 0.0282). That is an envelope limit of the Python being ported,
# not something the port introduces, so the corner is left OUT of the integrator grid and
# left IN section 2's `xnoeq` rows, where the guard's location is itself oracle-pinned.
TNO_CASES = []
for far in (0.01, 0.02717919071928212, 0.0677, 0.1354):
    for T in (1600.0, 2000.0, 2400.0) + ((2800.0,) if far >= 0.0677 else ()):
        for p in (101325.0, 1.5e6):
            TNO_CASES.append((far, T, p, 3e-3, 1.0))
# The rung-19 m-lifted arm, on the same pool: m enters BEFORE R1/R2 are formed, so it is not
# a post-hoc scale factor and has to be measured, not inferred from the m=1 rows.
for far in (0.02717919071928212, 0.0677, 0.1354):
    for T in (2000.0, 2400.0):
        TNO_CASES.append((far, T, 1.5e6, 3e-3, G._super_eq_o_multiplier(T)))
# Residence time is the one un-anchored knob rung 7 declares; sweep it two decades so the
# integrator is exercised from "barely started" to "near the [NO]_e ceiling" (where the
# clamp branch inside the loop actually fires).
for tau in (1e-4, 3e-4, 1e-3, 1e-2, 3e-2, 1e-1):
    TNO_CASES.append((0.0677, 2400.0, 1.5e6, tau, 1.0))

for far, T, p, tau, m in TNO_CASES:
    tag = f"{far!r}/{T!r}/{p!r}/{tau!r}/{m!r}"
    comp = G._equilibrium_composition(far, T, p)
    n = G._thermal_no(comp, T, p, tau, far, o_multiplier=m)
    put(f"tno/{tag}/x_no", n.x_no)
    put(f"tno/{tag}/x_no_eq", n.x_no_eq)
    put(f"tno/{tag}/initial_rate", n.initial_rate)
    put(f"tno/{tag}/char_time", n.char_time)
    put(f"tno/{tag}/ei_no", n.ei_no)
    put(f"tno/{tag}/frac_eq", n.fraction_of_equil)

# --- SECTION 4: PromptNO — algebra only, no solver ---------------------------------------
# f(φ) is a cubic that goes NEGATIVE past φ≈1.65, and `ei_prompt` clamps at 0. The grid
# straddles that sign change on purpose: a max(·, 0) is exactly the kind of thing a port
# spells as a clamp on the wrong side of the multiply.
PROMPTS = [("dflt", G.PromptNO()),
           ("c8", G.PromptNO(n_carbon=8.0)),
           ("peak5", G.PromptNO(peak_ei=5.0)),
           ("tref2200", G.PromptNO(T_ref=2200.0))]
for name, pr in PROMPTS:
    put(f"prompt/{name}/scale", pr.scale)
    for phi in (0.6, 0.8, 1.0, 1.2, 1.24, 1.4, 1.6, 1.65, 1.7, 2.0):
        put(f"prompt/{name}/f/{phi!r}", pr.f_correction(phi))
        for T in (2000.0, 2400.0, 2441.540385130793):
            put(f"prompt/{name}/ei/{phi!r}/{T!r}", pr.ei_prompt(phi, T))

# --- SECTION 5: Gas.thermal_nox — the rung-7 public entry point ---------------------------
GAS = G.Gas.reacting_equilibrium()
for far in (0.01, 0.02717919071928212, 0.0677):
    for T in (1600.0, 2000.0, 2400.0):
        for seo, pmt in ((False, None), (True, None), (False, G.PromptNO()),
                         (True, G.PromptNO())):
            tag = f"{far!r}/{T!r}/{int(seo)}{int(pmt is not None)}"
            n = GAS.thermal_nox(far, T, 1.5e6, 3e-3, super_eq_o=seo, prompt=pmt)
            put(f"tnox/{tag}/x_no", n.x_no)
            put(f"tnox/{tag}/ei_no", n.ei_no)
            put(f"tnox/{tag}/ei_prompt", n.ei_no_prompt)
            put(f"tnox/{tag}/ei_total", n.ei_no_total)
            put(f"tnox/{tag}/o_mult", n.o_multiplier)
            put(f"tnox/{tag}/ppm", n.ppm)
            put(f"tnox/{tag}/ppm_eq", n.ppm_eq)
# The EXPLICIT-φ branch. `thermal_nox`'s `phi` argument overrides the derived far/f_stoich for
# the prompt term ONLY, so without a case that sets it the branch ships unmeasured — the same
# objection phase 2 raised against porting rung 30's choked nozzle into a phase whose gates
# could not see it. Two φ per point: one that must MOVE the prompt off its derived value, and
# the derived value itself passed explicitly, which must reproduce the default arm exactly.
for far in (0.02717919071928212, 0.0677):
    for phi in (0.8, 1.2, 1.6, far / G._F_STOICH):
        n = GAS.thermal_nox(far, 2200.0, 1.5e6, 3e-3, prompt=G.PromptNO(), phi=phi)
        tag = f"{far!r}/{phi!r}"
        put(f"tnoxphi/{tag}/ei_prompt", n.ei_no_prompt)
        put(f"tnoxphi/{tag}/ei_total", n.ei_no_total)

# --- SECTION 6: the design points the zoned diagnostic actually runs on -------------------
# Derived from REAL equilibrium-engine runs, never hardcoded — the mix-out gate
# (|T_mix − Tt4| < 5 %) is a statement about a CONSISTENT (Tt3, Tt4, far) triple, so an
# invented one would either trip the gate or, worse, sit just inside it while being physically
# incoherent. Four points, chosen to move Tt3, Tt4, far and p independently.
FLIGHT_SUB = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
FLIGHT_SUP = FlightCondition(T0=216.7, p0=18_750.0, M0=2.0)
LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)

DESIGN = []      # (name, Tt3, Tt4, far, pt4)
for name, flight, pi_c, Tt4, mdot in (("dp1", FLIGHT_SUB, 10.0, 1500.0, 50.0),
                                      ("dp2", FLIGHT_SUB, 20.0, 1500.0, 50.0),
                                      ("dp3", FLIGHT_SUB, 10.0, 1700.0, 50.0),
                                      ("dp4", FLIGHT_SUP, 12.0, 1800.0, 50.0)):
    r = build_turbojet(G.Gas.reacting_equilibrium(), pi_c, Tt4, flight.p0,
                       **LOSSES).run(flight, mdot)
    st3, st4 = r.stations["3"], r.stations["4"]
    DESIGN.append((name, st3.Tt, st4.Tt, st4.far, st4.pt))
    put(f"dp/{name}/Tt3", st3.Tt)
    put(f"dp/{name}/Tt4", st4.Tt)
    put(f"dp/{name}/far", st4.far)
    put(f"dp/{name}/pt4", st4.pt)

# --- SECTION 7: the two new SOLVERS, swept for DISTINCT ROOTS -----------------------------
# `_primary_aft` — bisection on Σnᵢ(far_p,T)·h̄ᵢ_A(T) = h̄_air_A(T_air) + n_fuel·hf_fuel, the
# inner Σ re-solving the 8-species Newton at every trial T (~31 of them). Its root depends on
# far_p (through both sides), p (through the composition only), T_air (the preheat) and
# hf_fuel. This sweep moves all four, so no two rows share a root by construction.
HF = G._HF_FUEL_DEFAULT
AFT_CASES = []
for phi in (0.45, 0.6, 0.8, 0.95, 1.0, 1.2, 1.5, 2.0):
    AFT_CASES.append((phi * G._F_STOICH, 1.5e6, 583.5049266125288, HF))
for T_air in (400.0, 500.0, 583.5049266125288, 700.0, 850.0, 1000.0):
    AFT_CASES.append((1.0 * G._F_STOICH, 1.5e6, T_air, HF))
for p in P_GRID:
    AFT_CASES.append((0.9 * G._F_STOICH, p, 650.0, HF))
for hf in (HF, -50_000.0, -1e5, 0.0):
    AFT_CASES.append((0.85 * G._F_STOICH, 1.2e6, 620.0, hf))

# dedupe: the four sub-sweeps cross at their common centre (φ=1 at the dp1 preheat), and an
# identical argument tuple is literally the same root, not a second one.
AFT_CASES = list(dict.fromkeys(AFT_CASES))
for far_p, p, T_air, hf in AFT_CASES:
    put(f"aft/{far_p!r}/{p!r}/{T_air!r}/{hf!r}", G._primary_aft(far_p, p, T_air, hf))

# `_mixed_out_T` — the twin bisection, on the DILUTED pool. Its root is the overall adiabatic
# flame temperature from T_dilution, so it is split-independent (α cancels): that invariance
# is itself a rung-8 gate, and dumping several α at fixed (far_ov, T_dil, p) makes it
# checkable in Rust as a bit-equality rather than as a tolerance.
MIX_CASES = []


def mix_case(tag, far_ov, T_dil, p, phi):
    """Build a CONSISTENT (primary, split) pair for one mix-out root: α·far_p = far_ov is what
    makes α cancel, so an invented α would be measuring a balance the model never solves."""
    far_p = phi * G._F_STOICH
    alpha = far_ov / far_p
    if alpha > 1.0:
        return
    T_p = G._primary_aft(far_p, p, T_dil, HF)
    MIX_CASES.append((tag, G._equilibrium_composition(far_p, T_p, p), T_p, alpha,
                      far_ov, T_dil, p))


# (a) the SPLIT-INDEPENDENCE rows: three φ at one design point should give the SAME root,
# because α cancels out of the balance. Deliberately redundant, and it is a gate not a waste —
# but NOT a bit-equality, which is what measuring it taught. α cancels ALGEBRAICALLY; in
# floating point α·far_p = far_ov holds only to rounding, so the target moves in the last bits
# and the bisection's FINAL sign test can land on the other side. Measured: 0.0 K at two design
# points and 5.821e-7 K at the other two — and that is `2500/2**32` EXACTLY, one quantum of the
# [700,3200] bracket after the 32 halvings the `hi-lo < 1e-6` rule allows. `nox_oracle.rs`
# asserts one quantum, which is the tightest true statement (Python's own gate says 1e-3 K).
for name, Tt3, Tt4, far, pt4 in DESIGN:
    for phi in (0.6, 0.95, 1.4):
        mix_case(f"{name}/{phi!r}", far, Tt3, pt4, phi)
# (b) the DISTINCT-ROOT rows: move each argument the root actually depends on, one at a time.
for far_ov in (0.012, 0.018, 0.022, 0.030, 0.035, 0.045):
    mix_case(f"far/{far_ov!r}", far_ov, 650.0, 1.2e6, 0.95)
for T_dil in (400.0, 500.0, 700.0, 850.0, 1000.0):
    mix_case(f"tdil/{T_dil!r}", 0.025, T_dil, 1.2e6, 0.95)
for p in P_GRID:
    mix_case(f"p/{p!r}", 0.025, 650.0, p, 0.95)

for tag, comp_p, T_p, alpha, far_ov, T_dil, p in MIX_CASES:
    put(f"mixT/{tag}", G._mixed_out_T(comp_p, T_p, alpha, far_ov, T_dil, p))

# --- SECTION 8: Gas.zoned_nox — the whole rung-8/9/19 chain -------------------------------
def dump_zoned(tag, gas, far, Tt3, Tt4, p, phi, **kw):
    z = gas.zoned_nox(far, Tt3, Tt4, p, phi, tau=3e-3, **kw)
    put(f"zoned/{tag}/far_primary", z.far_primary)
    put(f"zoned/{tag}/alpha", z.alpha)
    put(f"zoned/{tag}/T_primary", z.T_primary)
    put(f"zoned/{tag}/T_mix", z.T_mix)
    put(f"zoned/{tag}/x_no_mix", z.x_no_mix)
    put(f"zoned/{tag}/ei_no", z.ei_no)
    put(f"zoned/{tag}/ei_prompt", z.ei_no_prompt)
    put(f"zoned/{tag}/ei_total", z.ei_no_total)
    put(f"zoned/{tag}/o_mult", z.o_multiplier)
    put(f"zoned/{tag}/ppm_primary", z.ppm_primary)
    put(f"zoned/{tag}/ppm_mix", z.ppm_mix)
    put(f"zoned/{tag}/primary_x_no_eq", z.primary.x_no_eq)
    put(f"zoned/{tag}/primary_char_time", z.primary.char_time)
    return z


PHI_SWEEP = (0.45, 0.6, 0.7, 0.8, 0.9, 0.95, 1.0, 1.1, 1.2, 1.4, 1.6, 2.0)
for name, Tt3, Tt4, far, pt4 in DESIGN:
    for phi in PHI_SWEEP:
        if far / (phi * G._F_STOICH) > 1.0:
            continue                       # leaner overall than the primary — α>1, out of scope
        dump_zoned(f"{name}/{phi!r}", GAS, far, Tt3, Tt4, pt4, phi)

# The rung-19 arms on the design point, both channels and both together.
_, TT3_1, TT4_1, FAR_1, PT4_1 = DESIGN[0]
for phi in (0.8, 0.95, 1.2, 1.5):
    for label, kw in (("seo", dict(super_eq_o=True)),
                      ("pmt", dict(prompt=G.PromptNO())),
                      ("both", dict(super_eq_o=True, prompt=G.PromptNO()))):
        dump_zoned(f"r19/{label}/{phi!r}", GAS, FAR_1, TT3_1, TT4_1, PT4_1, phi, **kw)

# --- SECTION 9: SHAPE KEYS — where the rung-9 bell PEAKS ----------------------------------
# Rung 9's claim is a LOCATION ("EI_NO peaks near φ≈0.95 and collapses rich"), and § 4.2 of
# the plan says a tolerance cannot cover a location. So the argmax is dumped as its own value.
# The grid is deliberately coarse enough (Δφ=0.01) that the peak sits several steps clear of
# its neighbours: a fine grid would make the argmax a coin-flip between adjacent cells and
# turn a real detector into a flaky one. The whole curve goes out beside it so that IF a later
# phase falls back to a tolerance, the adjudication has the shape and not just the verdict.
#
# MEASURED, and it is the reason this section is worth its cost: CPython and PyPy disagree on
# the peak VALUE in the last bit and agree on the peak LOCATION exactly. A value gate would
# have called that a deviation; the shape key says the finding did not move.
#
# dp1 and dp3 produce the SAME bell, which is not redundancy but an invariant: EI is set in
# the PRIMARY, so it depends on (far_p, p, Tt3, tau) and NOT on Tt4 or the overall far. dp3
# differs from dp1 in exactly Tt4, so an identical bell IS rung 8's "dilution lowers the mole
# fraction, not the emission index", read at the bit. `nox_oracle.rs` asserts it.
BELL_PHI = [0.85 + 0.01 * i for i in range(26)]        # 0.85 … 1.10
for name, Tt3, Tt4, far, pt4 in DESIGN:
    best_phi, best_ei = None, -1.0
    for phi in BELL_PHI:
        ei = GAS.zoned_nox(far, Tt3, Tt4, pt4, phi, tau=3e-3).ei_no
        put(f"bell/{name}/{phi!r}", ei)
        if ei > best_ei:
            best_phi, best_ei = phi, ei
    put(f"bell/{name}/argmax_phi", best_phi)
    put(f"bell/{name}/peak_ei", best_ei)


with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-3A NOx oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]}")
