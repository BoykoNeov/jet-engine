"""THE ORACLE, phase 3 slice D — every rung-22/23/24 resolved-cross-plane value the Rust must
reproduce.

The sixth in the family (`dump_gas.py` → `dump_cycle.py` → `dump_nox.py` → `dump_quench.py` →
`dump_pdf.py` → here). A separate file again, for the usual reason: the earlier TSVs stay frozen
as their own audit trail, and each dump's regeneration cost stays proportional to what it
certifies.

WHAT IS ACTUALLY NEW HERE, and therefore what the sweep is built around:

  * A RESOLVED y-z FIELD with a MEAN-PRESERVING BISECTION inside it. `_spatial_segregation` walks
    60 fixed halvings of an air-delivery scale so `⟨ξ⟩ = ξ̄` exactly, and every bisection step
    evaluates an `ny·nz` double loop. The bracket the halvings walk is decided by a `>` on an
    accumulated mean, so a single last-bit difference in that accumulation can send one step the
    other way and move `s` by ~1e-15 — which is exactly the shape slice B's trajectory roots had.
  * THREE FUNCTIONS THAT COMPUTE THE SAME `g` BY THREE ROUTES, and — measured, § 2 — only TWO of
    them agree bit-for-bit. Rung 23 reaches the terminal field through a time development whose
    `frac = 1.0` scalings (`1.0**(1/3)`, `sqrt(1.0)`) are exactly 1, and accumulates FLAT like
    rung 22, so it is EXACT. Rung 24 accumulates its mean HIERARCHICALLY
    (`sum(sum(r) for r in xi)`) while its mean-square two lines later is FLAT, so it is NOT. The
    Python's docstrings claim the OPPOSITE pairing and its own gate asserts `< 1e-9` on both,
    which is why the source cannot see the difference. Both are dumped, with different bars.
  * A DATA-DEPENDENT KNOT COUNT. Rungs 23 and 24 bin a `(ξ, τ)` cloud into `max(8, ny//2)` bins
    and keep only the NON-EMPTY ones, so the interpolator's knot count is an OUTPUT of the field.
    A count off by one reshapes the whole spectrum and no tolerance on τ would name it, so it is
    dumped as its own DISCRETE key — the family slice C's first-burnable-node index opened.
  * A HEAVILY-TAKEN SCHEME BRANCH. Rung 24 sends any cell with `u < 1e-8` to an analytic
    stagnant limit. Measured, 18–50 % of cells take it, because the β-clip creates large
    exactly-flat plateaus where `|∇ξ|² = 0` — so unlike rung 20's flame-band floor this is the
    OPPOSITE of dormant, and its count is dumped per J.

THE SIZING LEVER. The cross-plane itself is cheap: one field is `O(60·ny·nz)` for the bisection
plus `O(ny·nz)` for the moments, all of it plain arithmetic with no solver. So the FIELD sweeps
run wide (three J decades × three grids × several geometries) and cost seconds. What is expensive
is the end-to-end `zoned_nox` for rungs 23/24, because each builds TWO per-pocket banks (the
correlated one and its matched-mean twin) and every rich pocket in a bank is a full `_quench_no`.
Those run at the small `NB16/NG16/NS16` resolutions in § 7, and § 4–6 certify the spectra that
feed them separately — so a defect localises to the field, the spectrum, or the chemistry rather
than to "⟨EI⟩ differs".

SHAPE KEYS. Slice B measured a location key REFUTING the claim it was dumped to confirm, and
slice C measured one sitting ON a scheme boundary. Four locations are dumped here, all on the
COARSE grid discipline slice C forced:

  * rung 22's `g`-minimum over a J sweep, at three geometries. The Python's own helper sweeps 49
    or 81 log-spaced points over J ∈ [1, 400], which puts neighbours 3.8–6.4 % apart in `C`
    around a QUADRATIC minimum — the configuration slice C says not to ship a location key on.
    This dump uses rung 24's own house grid instead (J ∈ {4, 9, 16, 36, 64}, ~1.8–2.25× apart,
    with `C_opt` landing exactly on a node) and dumps the fine sweep as VALUES only.
  * rung 24's `F(C)` minimum, and — the KILL TEST — the `⟨|∇ξ|²⟩` MAXIMUM. `u` carries an
    explicit `1/var` and rung 22 already mins `g` at `C_opt`, so "argmin F == argmin g" is a
    TELL. `⟨|∇ξ|²⟩` carries no `g` algebraically, so it is the g-free witness, and it is
    rebuilt here from the field WITHOUT any variance normalisation rather than read off a
    production accessor — an accessor would make the witness compare production to itself.
  * the stagnant-cell count's minimum, which lands at `C_opt` too. Dumped, but recorded in the
    Rust gate as CORROBORATION and not as a second kill test: `u` carries the same `1/var`.
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


def argmin(vals):
    return min(range(len(vals)), key=lambda i: vals[i])


def argmax(vals):
    return max(range(len(vals)), key=lambda i: vals[i])


# --- resolutions ------------------------------------------------------------------------------
TAU = 3e-3
HF = G._HF_FUEL_DEFAULT
PHI_P = 1.5      # the RQL rich primary every mixing rung anchors on
CE = 0.20        # the ANCHORED jet-entrainment regime (rungs 11-16)
S0, H0 = 0.0625, 0.10        # the shipped spacing and duct height ⇒ C_opt=2.5 at J=16
KP, KY, KZ = 0.316, 0.28, 0.28
# The cross-plane grids. 16 is deliberately below anything the Python ships: it is the grid where
# the non-empty-bin count has the most room to differ, and the field is pure arithmetic so a
# coarse grid certifies the same spellings a fine one would.
NY_SWEEP = (16, 32, 48)
NT = 24                       # rung-23 dwell time steps
# The end-to-end rung-23/24 chemistry in § 7 — each bank is n_bell pockets, each a full quench,
# and each closure builds TWO banks (correlated + matched-mean twin).
NB16, NQ16, NG16, NS16 = 20, 64, 9, 100
NGRID, NSTEPS = 17, 200
# The COARSE location grid — rung 24's own, C_opt landing exactly on the J=16 node.
J_COARSE = (4.0, 9.0, 16.0, 36.0, 64.0)


def mix(J, H=H0):
    return G.JetMixing(J=J, H=H, C_e=CE, shape_n=2.0)


# ==============================================================================================
# SECTION 1 — the SOLVER-FREE cross-plane algebra.
# ==============================================================================================
# Every spelling question answered in isolation, in milliseconds: the penetration's fractional
# power, the Holdeman group's sqrt, the derived optimum, and the plume's Gaussian sums with their
# wall images. A defect here would otherwise surface six sections later as "g differs".

put("alg/f_stoich", G._F_STOICH)
put("alg/xi_soot", (2.0 * G._F_STOICH) / (1.0 + 2.0 * G._F_STOICH))

for kp in (0.25, 0.316, 0.40):
    # C_opt = 1/(4k_p²) — the DERIVED optimum. There is no C_opt knob anywhere in rungs 22/23/24;
    # this is the whole signature of the inversion of rung 18.
    put(f"alg/c_opt/kp{kp}", 1.0 / (4.0 * kp * kp))

for J in (1.0, 4.0, 16.0, 100.0, 400.0):
    # δ = k_p·√(S·H)·J^(1/4): a real `pow` with a fractional exponent beside a real `sqrt`. The
    # port must NOT fold J**0.25 into sqrt(sqrt(J)) — a different function.
    put(f"alg/delta/J{J}", KP * math.sqrt(S0 * H0) * J ** 0.25)
    put(f"alg/quarter/J{J}", J ** 0.25)
    put(f"alg/C/J{J}", (S0 / H0) * math.sqrt(J))
for frac in (1.0 / 24.0, 0.5, 23.0 / 24.0, 1.0):
    # rung 23's time development: (t/τ)^(1/3) and √(t/τ). At frac=1.0 BOTH must be exactly 1.0 —
    # that exactness is why rung 23's terminal field reproduces rung 22's bit-for-bit.
    put(f"alg/cbrt/{frac:.6f}", frac ** (1.0 / 3.0))
    put(f"alg/sqrtf/{frac:.6f}", math.sqrt(frac))

FLIGHT_SUB = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
FLIGHT_SUP = FlightCondition(T0=216.7, p0=18_750.0, M0=2.0)
LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)

DESIGN = {}
for name, flight, pi_c, Tt4, mdot in (("dp1", FLIGHT_SUB, 10.0, 1500.0, 50.0),
                                      ("dp4", FLIGHT_SUP, 12.0, 1800.0, 50.0)):
    r = build_turbojet(G.Gas.reacting_equilibrium(), pi_c, Tt4, flight.p0,
                       **LOSSES).run(flight, mdot)
    st3, st4 = r.stations["3"], r.stations["4"]
    DESIGN[name] = (st3.Tt, st4.Tt, st4.far, st4.pt)
    put(f"dp/{name}/Tt3", st3.Tt)
    put(f"dp/{name}/Tt4", st4.Tt)
    put(f"dp/{name}/far", st4.far)
    put(f"dp/{name}/pt4", st4.pt)
    put(f"dp/{name}/xibar", st4.far / (1.0 + st4.far))

GAS = G.Gas.reacting_equilibrium()
FAR1 = DESIGN["dp1"][2]
FAR4 = DESIGN["dp4"][2]

# the two-stream ceiling every resolved width must stay under (rung 18's DERIVED bound)
for name, far in (("dp1", FAR1), ("dp4", FAR4)):
    put(f"ceil/{name}", G._two_stream_ceiling(far, PHI_P))


# ==============================================================================================
# SECTION 2 — the RESOLVED WIDTH g, and the THREE ROUTES TO IT.
# ==============================================================================================
# The headline measurement of this slice. `_spatial_segregation` (rung 22), the terminal field of
# `_spatial_dwell_field` (rung 23) and the field of `_spatial_local_field` (rung 24) all compute
# the SAME g. Rung 23 is EXACTLY equal to rung 22; rung 24 is NOT, and the entire difference is
# one hierarchical sum. Both differences are dumped as their own keys so the Rust asserts the
# EXACT one exactly and the inexact one to a bar — and so a port that "tidies" the hierarchical
# sum into a flat one FAILS here rather than silently improving on the source.

TAU_MIX_REF = 1.0e-3     # a fixed dwell scale for the field sweeps (τ_mix cancels out of rung 24)

for ny in NY_SWEEP:
    for J in (1.0, 4.0, 16.0, 100.0, 400.0):
        g22 = G._spatial_segregation(FAR1, PHI_P, S0, H0, J, k_p=KP, k_y=KY, k_z=KZ, ny=ny, nz=ny)
        g23, _tau23 = G._spatial_dwell_field(FAR1, PHI_P, S0, H0, J, TAU_MIX_REF,
                                             k_p=KP, k_y=KY, k_z=KZ, ny=ny, nz=ny, nt=NT)
        g24, _tau24, _F = G._spatial_local_field(FAR1, PHI_P, S0, H0, J, TAU_MIX_REF,
                                                 k_p=KP, k_y=KY, k_z=KZ, ny=ny, nz=ny)
        tag = f"n{ny}/J{J}"
        put(f"g22/{tag}", g22)
        put(f"g23/{tag}", g23)
        put(f"g24/{tag}", g24)
        # The reduce residuals, as VALUES. `d23` must be exactly +0.0 everywhere; `d24` must not
        # be, and its magnitude is the bar the Rust gate uses.
        put(f"d23/{tag}", g23 - g22)
        put(f"d24/{tag}", g24 - g22)

# The two geometries that exercise the (H/S)² collapse — a HALVED spacing and a DOUBLED height,
# both of which must move J_opt by ×4 while leaving the minimum's DEPTH alone.
for label, S, H in (("halfS", S0 / 2.0, H0), ("dblH", S0, 2.0 * H0), ("both", 2.0 * S0, 2.0 * H0)):
    for J in (4.0, 16.0, 64.0, 256.0):
        put(f"g22/{label}/J{J}",
            G._spatial_segregation(FAR1, PHI_P, S, H, J, k_p=KP, k_y=KY, k_z=KZ, ny=32, nz=32))

# The hotter supersonic design point — a different ξ̄, so a different β̄ and a different bisection.
for J in (4.0, 16.0, 64.0):
    put(f"g22/dp4/J{J}",
        G._spatial_segregation(FAR4, PHI_P, S0, H0, J, k_p=KP, k_y=KY, k_z=KZ, ny=32, nz=32))


# ==============================================================================================
# SECTION 3 — the LOCATION keys, on the COARSE grid.
# ==============================================================================================
# Rung 22's optimum is found by GRID ARGMIN, so the grid is part of the finding. The Python's own
# helper uses 49/81 log-spaced points over J ∈ [1,400]; around a quadratic minimum that puts
# neighbours 3.8–6.4 % apart in C, and slice C measured what happens when a location key sits
# that close to its neighbours. The house grid below is rung 24's, and the CLEARANCE of the
# winning cell over both neighbours is dumped beside the index so the Rust gate can assert the
# margin rather than trust it.


def coarse_argmin_g(far, S, H, kp=KP, ny=32):
    gs = [G._spatial_segregation(far, PHI_P, S, H, J, k_p=kp, k_y=KY, k_z=KZ, ny=ny, nz=ny)
          for J in J_COARSE]
    i = argmin(gs)
    return gs, i


for label, S, H, kp in (("base", S0, H0, KP),
                        ("halfS", S0 / 2.0, H0, KP),
                        ("dblH", S0, 2.0 * H0, KP),
                        ("kp25", S0, H0, 0.25),
                        ("kp40", S0, H0, 0.40)):
    # halfS/dblH shift J_opt by (H/S)² = ×4, so they need their own grid to have the optimum on a
    # node — the same coarse SHAPE, scaled. kp moves C_opt instead, so it keeps the base grid.
    grid = tuple(J * 4.0 for J in J_COARSE) if label in ("halfS", "dblH") else J_COARSE
    gs = [G._spatial_segregation(FAR1, PHI_P, S, H, J, k_p=kp, k_y=KY, k_z=KZ, ny=32, nz=32)
          for J in grid]
    i = argmin(gs)
    put(f"loc/g/{label}/idx", float(i))
    put(f"loc/g/{label}/J", grid[i])
    put(f"loc/g/{label}/C", (S / H) * math.sqrt(grid[i]))
    put(f"loc/g/{label}/gmin", gs[i])
    # the clearance of the winning cell over each neighbour, as a RATIO — the margin a last-bit
    # difference would have to cross to relocate the argmin.
    if 0 < i < len(gs) - 1:
        put(f"loc/g/{label}/clear_lo", gs[i - 1] / gs[i])
        put(f"loc/g/{label}/clear_hi", gs[i + 1] / gs[i])
    for k, gv in enumerate(gs):
        put(f"loc/g/{label}/v{k}", gv)

# The FINE sweep the Python's own helper runs — dumped as VALUES ONLY, never as an argmin, which
# is the whole point of the coarse grid above.
J_FINE = [1.0 * 400.0 ** (i / 48.0) for i in range(49)]
for k in (0, 12, 24, 30, 36, 48):
    put(f"fine/J{k}", J_FINE[k])
    put(f"fine/g{k}", G._spatial_segregation(FAR1, PHI_P, S0, H0, J_FINE[k],
                                             k_p=KP, k_y=KY, k_z=KZ, ny=32, nz=32))


# ==============================================================================================
# SECTION 4 — rung 24's F(C), and the G-FREE WITNESS (the circularity kill test).
# ==============================================================================================
# F = ⟨τ⟩/τ_mix is a PURE field functional — τ_mix cancels out of u = σ²|∇ξ|²/(2var) — so F at two
# very different τ_mix must be BIT-IDENTICAL, which is dumped as its own pair of keys rather than
# argued from the algebra. F's minimum sits at C_opt; so does g's, and that coincidence is a TELL
# because u carries an explicit 1/var. The witness that breaks the circularity is ⟨|∇ξ|²⟩, which
# carries no g at all — rebuilt below from the field WITHOUT variance normalisation, because
# reading it off a production accessor would compare production to itself.


def mean_grad_sq(far, S, H, J, kp=KP, ny=32, nz=32):
    """⟨|∇ξ|²⟩ of the terminal field, rebuilt with NO variance normalisation — the g-free witness."""
    xibar = far / (1.0 + far)
    far_p = PHI_P * G._F_STOICH
    xi_p = far_p / (1.0 + far_p)
    delta = kp * math.sqrt(S * H) * J ** 0.25
    sig_y, sig_z = KY * H, KZ * S
    ys = [(i + 0.5) * H / ny for i in range(ny)]
    zs = [(j + 0.5) * S / nz for j in range(nz)]
    ay = [sum(math.exp(-((y - c) ** 2) / (2 * sig_y * sig_y))
              for c in (-delta, delta, 2 * H - delta, 2 * H + delta)) for y in ys]
    az = [sum(math.exp(-((z - S / 2 - m * S) ** 2) / (2 * sig_z * sig_z))
              for m in (-1, 0, 1)) for z in zs]
    may, maz = sum(ay) / ny, sum(az) / nz
    ayh, azh = [a / may for a in ay], [a / maz for a in az]
    beta_bar = (xi_p - xibar) / xi_p

    def mean_at(s):
        return sum(xi_p * (1.0 - min(1.0, max(0.0, s * beta_bar * a * b)))
                   for a in ayh for b in azh) / (ny * nz)

    lo, hi = 0.0, 50.0
    for _ in range(60):
        s = 0.5 * (lo + hi)
        if mean_at(s) > xibar:
            lo = s
        else:
            hi = s
    s_star = 0.5 * (lo + hi)
    xi = [[xi_p * (1.0 - min(1.0, max(0.0, s_star * beta_bar * a * b))) for b in azh] for a in ayh]
    dy, dz = H / ny, S / nz
    tot = 0.0
    for i in range(ny):
        im, ip = max(0, i - 1), min(ny - 1, i + 1)
        for j in range(nz):
            jm, jp = (j - 1) % nz, (j + 1) % nz
            gy = (xi[ip][j] - xi[im][j]) / ((ip - im) * dy)
            gz = (xi[i][jp] - xi[i][jm]) / (2 * dz)
            tot += gy * gy + gz * gz
    return tot / (ny * nz)


F_COARSE = []
for J in J_COARSE:
    _g, _t, F = G._spatial_local_field(FAR1, PHI_P, S0, H0, J, TAU_MIX_REF,
                                       k_p=KP, k_y=KY, k_z=KZ, ny=32, nz=32)
    F_COARSE.append(F)
    put(f"F/base/J{J}", F)
iF = argmin(F_COARSE)
put("loc/F/base/idx", float(iF))
put("loc/F/base/J", J_COARSE[iF])
put("loc/F/base/C", (S0 / H0) * math.sqrt(J_COARSE[iF]))
put("loc/F/base/clear_lo", F_COARSE[iF - 1] / F_COARSE[iF])
put("loc/F/base/clear_hi", F_COARSE[iF + 1] / F_COARSE[iF])
# the U's DEPTH — the ~39 % the headline weighs against τ_mix's ~20× swing
put("F/base/depth", max(F_COARSE) / min(F_COARSE))

# τ_mix CANCELS — F at two τ_mix three decades apart must be bit-identical.
for J in (4.0, 16.0, 64.0):
    _g, _t, Fa = G._spatial_local_field(FAR1, PHI_P, S0, H0, J, 1.0e-4,
                                        k_p=KP, k_y=KY, k_z=KZ, ny=32, nz=32)
    _g, _t, Fb = G._spatial_local_field(FAR1, PHI_P, S0, H0, J, 1.0e-1,
                                        k_p=KP, k_y=KY, k_z=KZ, ny=32, nz=32)
    put(f"F/cancel/J{J}/a", Fa)
    put(f"F/cancel/J{J}/b", Fb)
    put(f"F/cancel/J{J}/d", Fb - Fa)      # must be exactly +0.0

# THE KILL TEST — ⟨|∇ξ|²⟩ carries no g, and it is MAXIMAL at C_opt.
GRADS = [mean_grad_sq(FAR1, S0, H0, J) for J in J_COARSE]
for J, gv in zip(J_COARSE, GRADS):
    put(f"grad/base/J{J}", gv)
iG = argmax(GRADS)
put("loc/grad/base/idx", float(iG))
put("loc/grad/base/J", J_COARSE[iG])
put("loc/grad/base/clear_lo", GRADS[iG] / GRADS[iG - 1])
put("loc/grad/base/clear_hi", GRADS[iG] / GRADS[iG + 1])

# The (H/S)² shift, inherited by the DWELL — rung 22's signature carried into rung 24's F.
for label, S, grid in (("halfS", S0 / 2.0, tuple(J * 4.0 for J in J_COARSE)),
                       ("base", S0, J_COARSE)):
    Fs = [G._spatial_local_field(FAR1, PHI_P, S, H0, J, TAU_MIX_REF,
                                 k_p=KP, k_y=KY, k_z=KZ, ny=32, nz=32)[2] for J in grid]
    i = argmin(Fs)
    put(f"loc/Fshift/{label}/idx", float(i))
    put(f"loc/Fshift/{label}/J", grid[i])
    put(f"loc/Fshift/{label}/C", (S / H0) * math.sqrt(grid[i]))


# ==============================================================================================
# SECTION 5 — the DISCRETE keys: knot counts and the stagnant-branch census.
# ==============================================================================================
# Neither is a value. The knot count is an OUTPUT of the binner (only non-empty bins become
# knots), and the stagnant count is the census of a scheme branch. Both are integers a 15th-digit
# shift can move, and neither is visible to any tolerance on τ or F.


def knot_count(tau_of_xi):
    """The non-empty-bin count, read off the closure's own captured knot list."""
    return len(tau_of_xi.__closure__[0].cell_contents)     # `centers`


def stagnant_cells(far, S, H, J, ny=32, nz=32):
    """How many cells take rung 24's `u < 1e-8` analytic stagnant limit."""
    xibar = far / (1.0 + far)
    far_p = PHI_P * G._F_STOICH
    xi_p = far_p / (1.0 + far_p)
    delta = KP * math.sqrt(S * H) * J ** 0.25
    sig_y, sig_z = KY * H, KZ * S
    ys = [(i + 0.5) * H / ny for i in range(ny)]
    zs = [(j + 0.5) * S / nz for j in range(nz)]
    ay = [sum(math.exp(-((y - c) ** 2) / (2 * sig_y * sig_y))
              for c in (-delta, delta, 2 * H - delta, 2 * H + delta)) for y in ys]
    az = [sum(math.exp(-((z - S / 2 - m * S) ** 2) / (2 * sig_z * sig_z))
              for m in (-1, 0, 1)) for z in zs]
    may, maz = sum(ay) / ny, sum(az) / nz
    ayh, azh = [a / may for a in ay], [a / maz for a in az]
    beta_bar = (xi_p - xibar) / xi_p

    def mean_at(s):
        t = 0.0
        for a in ayh:
            sa = s * beta_bar * a
            for b in azh:
                t += xi_p * (1.0 - min(1.0, max(0.0, sa * b)))
        return t / (ny * nz)

    lo, hi = 0.0, 50.0
    for _ in range(60):
        s = 0.5 * (lo + hi)
        if mean_at(s) > xibar:
            lo = s
        else:
            hi = s
    s_star = 0.5 * (lo + hi)
    xi = [[xi_p * (1.0 - min(1.0, max(0.0, s_star * beta_bar * a * b))) for b in azh] for a in ayh]
    mean = sum(sum(r) for r in xi) / (ny * nz)          # HIERARCHICAL, as production
    meansq = sum(v * v for r in xi for v in r) / (ny * nz)   # FLAT, as production
    var = max(meansq - mean * mean, 0.0)
    dy, dz = H / ny, S / nz
    n = 0
    for i in range(ny):
        im, ip = max(0, i - 1), min(ny - 1, i + 1)
        for j in range(nz):
            jm, jp = (j - 1) % nz, (j + 1) % nz
            gy = (xi[ip][j] - xi[im][j]) / ((ip - im) * dy)
            gz = (xi[i][jp] - xi[i][jm]) / (2 * dz)
            if sig_y * sig_y * (gy * gy + gz * gz) / (2.0 * max(var, 1e-30)) < 1e-8:
                n += 1
    return n


STAG = []
for J in J_COARSE:
    c = stagnant_cells(FAR1, S0, H0, J)
    STAG.append(c)
    put(f"stag/base/J{J}", float(c))
put("loc/stag/base/idx", float(argmin(STAG)))
put("loc/stag/base/J", J_COARSE[argmin(STAG)])
# the branch is NOT dormant — this fraction is the evidence, and the Rust gate asserts a band
put("stag/base/frac_min", min(STAG) / (32.0 * 32.0))
put("stag/base/frac_max", max(STAG) / (32.0 * 32.0))
for ny in NY_SWEEP:
    put(f"stag/n{ny}/J16", float(stagnant_cells(FAR1, S0, H0, 16.0, ny=ny, nz=ny)))

for ny in NY_SWEEP:
    for J in (4.0, 16.0, 100.0):
        _g23, t23 = G._spatial_dwell_field(FAR1, PHI_P, S0, H0, J, TAU_MIX_REF,
                                           k_p=KP, k_y=KY, k_z=KZ, ny=ny, nz=ny, nt=NT)
        _g24, t24, _F = G._spatial_local_field(FAR1, PHI_P, S0, H0, J, TAU_MIX_REF,
                                               k_p=KP, k_y=KY, k_z=KZ, ny=ny, nz=ny)
        put(f"knots/r23/n{ny}/J{J}", float(knot_count(t23)))
        put(f"knots/r24/n{ny}/J{J}", float(knot_count(t24)))


# ==============================================================================================
# SECTION 6 — the τ(ξ) SPECTRA themselves.
# ==============================================================================================
# The interpolator is what feeds the per-pocket bank, so it is dumped directly at sample ξ —
# including BOTH flat-extrapolated ends, where a knot-count difference would show first. Rung
# 23's spectrum is an arrival-time DEFICIT bounded by [0, τ_mix]; rung 24's is the analytic
# τ_mix·[1 − 1/E + 1/u]. They are DIFFERENT functions of the same field and both are dumped, so a
# port that wires one into the other's slot fails here and not as a corr_ratio 3 % off.

XI_MAX = (2.0 * G._F_STOICH) / (1.0 + 2.0 * G._F_STOICH)
for J in (4.0, 16.0, 64.0):
    _g23, t23 = G._spatial_dwell_field(FAR1, PHI_P, S0, H0, J, TAU_MIX_REF,
                                       k_p=KP, k_y=KY, k_z=KZ, ny=32, nz=32, nt=NT)
    _g24, t24, F24 = G._spatial_local_field(FAR1, PHI_P, S0, H0, J, TAU_MIX_REF,
                                            k_p=KP, k_y=KY, k_z=KZ, ny=32, nz=32)
    for k in range(9):
        xi = XI_MAX * k / 8.0            # k=0 and k=8 exercise both flat extrapolations
        put(f"tau23/J{J}/x{k}", t23(xi))
        put(f"tau24/J{J}/x{k}", t24(xi))
    # ⟨τ⟩ over the β-PDF — the matched-mean scalar the correlation-off twin runs at. This is the
    # bridge from the spectrum to the chemistry, so it is dumped before § 7 spends the quenches.
    xibar = FAR1 / (1.0 + FAR1)
    for tag, tf, gseg in (("r23", t23, _g23), ("r24", t24, _g24)):
        nodes, wts = G._beta_pdf_nodes_weights(xibar, gseg, n_quad=NQ16)
        put(f"taumean/{tag}/J{J}", sum(wi * tf(x) for wi, x in zip(wts, nodes)))


# ==============================================================================================
# SECTION 7 — END TO END through `zoned_nox`, on the real chemistry.
# ==============================================================================================
# The three closures wired into production, at the small resolutions, including the MATCHED-MEAN
# TWINS that are rungs 23/24's whole instrument: the same integral at the scalar ⟨τ⟩ removes the
# ξ–τ correlation, so `corr_ratio` isolates it. A τ(ξ) accidentally wired flat gives exactly 1.0,
# which is why the ratio is dumped rather than only the correlated EI.

SP_S = G.SpatialPDF(S=S0, ny=24, nz=24, n_bell=NB16, n_quad=NQ16)
SD_S = G.SpatialDwellPDF(S=S0, ny=20, nz=20, nt=16, n_bell=NB16, n_quad=NQ16)
SL_S = G.SpatialLocalPDF(S=S0, ny=20, nz=20, n_bell=NB16, n_quad=NQ16)


def dump_zoned(tag, dp, **kw):
    Tt3, Tt4, far, p = DESIGN[dp]
    z = GAS.zoned_nox(far, Tt3, Tt4, p, PHI_P, tau=TAU, quench_ngrid=NGRID,
                      quench_nsteps=NSTEPS, **kw)
    put(f"zn/{tag}/ei_no", z.ei_no)
    put(f"zn/{tag}/ei_quenched", z.ei_no_quenched)
    put(f"zn/{tag}/max_a", z.max_a_quench)
    put(f"zn/{tag}/C_holdeman", z.C_holdeman)
    put(f"zn/{tag}/g_seg", z.g_seg)
    put(f"zn/{tag}/g_ceiling", z.g_ceiling)
    if z.ei_no_spatial is not None:
        put(f"zn/{tag}/g_spatial", z.g_spatial)
        put(f"zn/{tag}/ei_spatial", z.ei_no_spatial)
    if z.ei_no_spatial_dwell is not None:
        put(f"zn/{tag}/g_dwell", z.g_spatial_dwell)
        put(f"zn/{tag}/tau_mean", z.tau_mean_dwell)
        put(f"zn/{tag}/ei_excess", z.ei_no_spatial_dwell_excess)
        put(f"zn/{tag}/ei_dwell", z.ei_no_spatial_dwell)
        put(f"zn/{tag}/ei_meanfield", z.ei_no_spatial_dwell_meanfield)
        put(f"zn/{tag}/corr_ratio", z.corr_ratio)
    if z.ei_no_spatial_local is not None:
        put(f"zn/{tag}/g_local", z.g_spatial_local)
        put(f"zn/{tag}/f_shape", z.f_shape)
        put(f"zn/{tag}/tau_mean", z.tau_mean_local)
        put(f"zn/{tag}/ei_excess", z.ei_no_spatial_local_excess)
        put(f"zn/{tag}/ei_local", z.ei_no_spatial_local)
        put(f"zn/{tag}/ei_meanfield", z.ei_no_spatial_local_meanfield)
        put(f"zn/{tag}/corr_ratio", z.corr_ratio_local)
    print(f"  zoned {tag} at {time.time() - T0:.1f}s")
    return z


for J in (4.0, 16.0, 64.0):
    dump_zoned(f"r22/J{J}", "dp1", mixing=mix(J), spatial=SP_S)
dump_zoned("r22/J16/su", "dp1", mixing=mix(16.0), spatial=SP_S, super_eq_o=True)
dump_zoned("r22/dp4", "dp4", mixing=mix(16.0), spatial=SP_S)

for J in (4.0, 16.0, 64.0):
    dump_zoned(f"r23/J{J}", "dp1", mixing=mix(J), spatial_dwell=SD_S)
dump_zoned("r23/J16/su", "dp1", mixing=mix(16.0), spatial_dwell=SD_S, super_eq_o=True)

for J in (4.0, 16.0, 64.0):
    dump_zoned(f"r24/J{J}", "dp1", mixing=mix(J), spatial_local=SL_S)
dump_zoned("r24/J16/su", "dp1", mixing=mix(16.0), spatial_local=SL_S, super_eq_o=True)

with open(sys.argv[1], "w", encoding="utf-8", newline="\n") as fh:
    fh.write("# phase-3D resolved-cross-plane oracle — key\tu64 bits\trepr\n")
    fh.write(f"# {sys.implementation.name} {sys.version.split()[0]}\n")
    for key, bits, text in ROWS:
        fh.write(f"{key}\t{bits}\t{text}\n")

keys = [k for k, _, _ in ROWS]
assert len(set(keys)) == len(keys), "duplicate key in the dump"
print(f"{sys.implementation.name} {sys.version.split()[0]}: wrote {len(ROWS)} values "
      f"to {sys.argv[1]} in {time.time() - T0:.1f}s")
