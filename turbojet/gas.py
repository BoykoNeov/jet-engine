"""Working-fluid model and the flow state that travels through the engine.

RUNG 4 — reacting products: the hot-section composition (and thus cp_t, R_t,
gamma_t) tracks the fuel/air ratio f. Rung 3 let cp vary with T but froze the
products at one lean mixture; rung 4 computes the product mole numbers from f by
explicit (CH2)n lean-complete-combustion stoichiometry and mole-weights the SAME
NASA-7 species, so the property functions gain an f argument on the hot section.
The rung-3 frozen paths (Gas(), Gas.thermally_perfect()) are untouched — the
reacting gas is a separate Gas.reacting() path. See docs/rung4-reacting-products.md.

RUNG 3 — variable cp(T): the thermally-perfect gas. Rungs 1-2 modeled each
section as *calorically* perfect (constant gamma, cp, R). Rung 3 lets cp vary
with temperature (still an ideal gas p = rho R T, R constant per section, but
cp = cp(T) and therefore gamma = gamma(T)). This is the first rung where the
isentropic power law Tt/Tt = (pt/pt)^g STOPS being exact, so the cycle moves
from the algebraic cp*T / pi^g forms to the gas-table PROPERTY FUNCTIONS:

    h(T)        = int_0^T cp dT'              enthalpy,            J/kg
    phi(T)      = int    cp/T dT'             entropy function,    J/(kg K)
    pr(T)       = exp(phi(T)/R)               reduced pressure,    dimensionless
    T_from_h, T_from_pr                       the two inverses

For ANY process between states 1,2:  ds = phi(T2)-phi(T1) - R ln(p2/p1). Set
ds = 0 (isentropic) and it collapses to p2/p1 = pr(T2)/pr(T1) — so every
isentropic pressure<->temperature step in the cycle is ONE pr ratio. See
docs/rung3-variable-cp.md.

THE LOAD-BEARING DESIGN DECISION (docs/rung3-variable-cp.md § the trap). Rungs
1-2 reproduce their tables TO THE DIGIT using g = (gamma-1)/gamma with gamma=1.4
(exponent 0.28571) and a rounded R=287 that is ~0.05% off the relation
R = (gamma-1)/gamma * cp. But the thermally-perfect pr = exp(phi/R) has the
constant-cp limit pr = T^(R/cp) with exponent R/cp = 287/1004 = 0.28586 — those
two exponents differ by that same 0.05%. So routing a constant-cp gas through the
integral path lands ~3e-4 off the gamma-based answer. The fix:

  - a CALORICALLY-perfect section keeps the rung-1/2 CLOSED FORMS exactly
    (h = cp*T, pr = T^(1/g), closed-form inverses) — bit-for-bit prior behavior;
  - a THERMALLY-perfect section INTEGRATES (h, phi analytic from the polynomial;
    inverses by safeguarded Newton). R/cp(T)-based, as physics dictates.

The branch hides inside Gas: a section is CPG if it carries a constant triple,
TPG if it carries a cp(T) model. Components call the one property interface and
never see which — EXCEPT the two velocity<->enthalpy coupling stations (0 and 9),
where the rounded-R trap forces an explicit closed-form branch (see engine.py
freestream and components.py Nozzle). The defaults make a single-gas CPG instance
that reproduces rung 1 exactly, so the reduce-to-ideal gate just uses `Gas()`.
"""
import math
from dataclasses import dataclass, replace
from typing import Optional, Tuple


@dataclass
class FlowState:
    """Gas state at a station, carried in TOTAL (stagnation) quantities.

    Cycle analysis works in totals because they already fold in the kinetic
    energy of the flow; we only convert to static at the nozzle exit. See
    SPEC.md § Conventions.
    """

    Tt: float            # total temperature, K
    pt: float            # total pressure, Pa
    mdot: float          # mass flow, kg/s
    far: float = 0.0     # fuel-air ratio carried downstream of the burner


# --------------------------------------------------------------------------- #
# NASA 7-coefficient species data (rung 3 only — unused by CPG sections).      #
# Cp_molar(T)/Ru = a1 + a2 T + a3 T^2 + a4 T^3 + a5 T^4, two temperature       #
# ranges joined at 1000 K. Standard NASA/GRI-Mech values; certified by the     #
# air-table + Cengel/Mattingly anchors in tests/test_variable_cp.py, so a      #
# transcription slip cannot pass silently (docs/plans/rung3-anchor-cengel.md). #
# --------------------------------------------------------------------------- #

_Ru = 8.314462618        # universal gas constant, J/(mol K)
_T_BREAK = 1000.0        # K, low/high polynomial join

# (molar mass g/mol, low-range 200-1000 K, high-range 1000+ K) per species.
_SPECIES = {
    "N2": (28.0134,
           (3.298677, 1.4082404e-3, -3.963222e-6, 5.641515e-9, -2.444854e-12),
           (2.92664, 1.4879768e-3, -5.68476e-7, 1.0097038e-10, -6.753351e-15)),
    "O2": (31.9988,
           (3.78245636, -2.99673416e-3, 9.84730201e-6, -9.68129509e-9, 3.24372837e-12),
           (3.28253784, 1.48308754e-3, -7.57966669e-7, 2.09470555e-10, -2.16717794e-14)),
    "Ar": (39.948, (2.5, 0.0, 0.0, 0.0, 0.0), (2.5, 0.0, 0.0, 0.0, 0.0)),
    "CO2": (44.0095,
            (2.35677352, 8.98459677e-3, -7.12356269e-6, 2.45919022e-9, -1.43699548e-13),
            (3.85746029, 4.41437026e-3, -2.21481404e-6, 5.23490188e-10, -4.72084164e-14)),
    "H2O": (18.01528,
            (4.19864056, -2.03643410e-3, 6.52040211e-6, -5.48797062e-9, 1.77197817e-12),
            (3.03399249, 2.17691804e-3, -1.64072518e-7, -9.70419870e-11, 1.68200992e-14)),
    # RUNG 6 — the five dissociation species (CO, H2, OH, O, H). Same GRI-Mech 3.0
    # source as the five above; unused by rungs 1-5 (frozen/Fork-A/Fork-B never form
    # them). Certified by the a6/a7 self-checks + the CEA methane-AFT equilibrium
    # anchor (docs/plans/rung6-anchor-equilibrium.md § 1, 4).
    "CO": (28.0101,
           (3.57953347, -6.10353680e-4, 1.01681433e-6, 9.07005884e-10, -9.04424499e-13),
           (2.71518561, 2.06252743e-3, -9.98825771e-7, 2.30053008e-10, -2.03647716e-14)),
    "H2": (2.01588,
           (2.34433112, 7.98052075e-3, -1.94781510e-5, 2.01572094e-8, -7.37611761e-12),
           (3.33727920, -4.94024731e-5, 4.99456778e-7, -1.79566394e-10, 2.00255376e-14)),
    "OH": (17.00734,
           (3.99201543, -2.40131752e-3, 4.61793841e-6, -3.88113333e-9, 1.36411470e-12),
           (3.09288767, 5.48429716e-4, 1.26505228e-7, -8.79461556e-11, 1.17412376e-14)),
    "O": (15.9994,
          (3.16826710, -3.27931884e-3, 6.64306396e-6, -6.12806624e-9, 2.11265971e-12),
          (2.56942078, -8.59741137e-5, 4.19484589e-8, -1.00177799e-11, 1.22833691e-15)),
    "H": (1.00794,
          (2.50000000, 7.05332819e-13, -1.99591964e-15, 2.30081632e-18, -9.27732332e-22),
          (2.50000001, -2.30842973e-11, 1.61561948e-14, -4.73515235e-18, 4.98197357e-22)),
    # RUNG 7 — thermal NOx: NO and the N atom (the extended Zeldovich mechanism). Same
    # GRI-Mech 3.0 source; INERT to rungs 1-6 (nothing references them — _equil_solve
    # uses its fixed 8-species C/H/O basis, _mixture only the composition it is handed).
    # Certified by the a6/a7 self-check + the thermo-kinetic K-check (docs/plans/rung7-anchor-nox.md).
    "NO": (30.0061,
           (4.21859896, -4.63988124e-3, 1.10443049e-5, -9.34055507e-9, 2.80554874e-12),
           (3.26071234, 1.19101135e-3, -4.29122646e-7, 6.94481463e-11, -4.03295681e-15)),
    "N": (14.0067,
          (2.50000000, 0.0, 0.0, 0.0, 0.0),
          (2.41594290, 1.74890650e-4, -1.19023690e-7, 3.02262450e-11, -2.03609820e-15)),
}

# Cold section = dry air (mole fractions). R_air falls out at ~287.1, consistent
# with the rounded 287 of rungs 1-2.
_AIR = {"N2": 0.7808, "O2": 0.2095, "Ar": 0.0093}

# Hot section = a FIXED-composition lean combustion products mixture (NOT
# composition tracking — that is the next rung; docs/rung3-variable-cp.md § scope).
# Mole counts per mole of fuel for C12H23 (a Jet-A surrogate, stoich far=0.0684)
# burned LEAN at far ~= 0.030 (equivalence ratio ~0.44), with dry air as oxidizer:
#   C12H23 + 40.44 (O2 + 3.727 N2 + 0.0444 Ar)
#        -> 12 CO2 + 11.5 H2O + 22.69 O2(excess) + 150.74 N2 + 1.80 Ar
# The products are UNANCHORED (no external number pins them — Cengel/Mattingly are
# single-gas air); the composition is fixed and stated plainly. The temperature
# dependence alone lands cp_t(T) in the rung-2 cp_t~=1239 neighborhood (cp rises
# ~1241 @1240 K to ~1311 @1800 K); R_t ~= 287.1 falls out of the molar mass. USED
# ONLY by Gas.thermally_perfect() (rung 3); rung 4's Gas.reacting() computes the
# composition from f instead (see _products_composition below).
_PRODUCTS = {"N2": 150.74, "O2": 22.69, "Ar": 1.803, "CO2": 12.0, "H2O": 11.5}


# --------------------------------------------------------------------------- #
# RUNG 4 — reacting products: composition (and therefore cp_t, R_t, gamma_t)   #
# tracks the fuel/air ratio f. Explicit lean-complete-combustion stoichiometry #
# of a (CH2)n hydrocarbon (Jet-A ~= C12H23 ~= (CH2)n) in dry air:              #
#     CH2 + 1.5 O2 -> CO2 + H2O   (per mol fuel), burned LEAN (f < f_stoich).  #
# The product mole numbers are computed deterministically from f and fed to    #
# the SAME _mixture() mole-weighting the frozen rung-3 gas uses — only the      #
# composition now depends on f. See docs/rung4-reacting-products.md.           #
# --------------------------------------------------------------------------- #

# (CH2)n repeat unit molar mass, g/mol (C 12.011 + 2 H 1.008).
_M_CH2 = 12.011 + 2 * 1.008

# --------------------------------------------------------------------------- #
# RUNG 5 — Fork B: formation-enthalpy bookkeeping. Rung 4 stayed Fork A (fixed  #
# hPR, sensible h(0)=0 datum). Fork B carries each species' standard formation  #
# enthalpy so the burner's heat release is DERIVED from an absolute-enthalpy    #
# balance instead of assumed. The formation constant is exactly the NASA-7 a6   #
# term (H/RuT = ...poly... + a6/T) that rungs 3/4 dropped; since _antideriv_h   #
# is the polynomial part with NO constant, Ru*a6 is the additive formation      #
# offset. It CANCELS in every enthalpy DIFFERENCE (turbine, nozzle), so only    #
# the burner's cross-section subtraction sees it. See docs/rung5-fork-b.md and  #
# docs/plans/rung5-anchor-formation.md.                                         #
# --------------------------------------------------------------------------- #

# Standard molar enthalpies of formation at 298.15 K, J/mol (CODATA/JANAF).
# Elements are the reference datum (0); H2O is GAS (vapour) -> LHV, not liquid.
# The five dissociation species (CO/H2/OH/O/H) are added for rung 6; H2 is an
# element (0). OH carries ~1.5 kJ/mol literature spread (docs/plans/rung6-anchor § 1).
_T_REF = 298.15
_HF298 = {"N2": 0.0, "O2": 0.0, "Ar": 0.0, "CO2": -393520.0, "H2O": -241826.0,
          "CO": -110527.0, "H2": 0.0, "OH": 38987.0, "O": 249180.0, "H": 217998.0,
          # Rung 7 (JANAF). ΔHf°(NO) carries a real ~1 kJ/mol literature spread — GRI-Mech
          # assumes ~91.27 vs our JANAF 90.291; the K-check confirms JANAF (rung7-anchor § 1).
          "NO": 90291.0, "N": 472680.0}

# RUNG 6 — standard molar ENTROPIES at 298.15 K, J/(mol K) (CODATA/JANAF). The twin
# of _HF298: rung 5 derived the formation constant a6 from _HF298; rung 6 derives the
# absolute-entropy constant a7 from _S298 (a7 = S/Ru - antideriv_phi(A_low, 298.15)),
# so g0(T)=h0-T*s0 -> Kp. Consumed ONLY by the equilibrium/Kp solve, never downstream
# pr (the additive a7 cancels in every pr ratio, as a6 cancels in enthalpy diffs).
_S298 = {"N2": 191.609, "O2": 205.152, "Ar": 154.846, "CO2": 213.785, "H2O": 188.835,
         "CO": 197.660, "H2": 130.680, "OH": 183.708, "O": 161.058, "H": 114.716,
         "NO": 210.758, "N": 153.30}    # rung 7 (JANAF)

_P_REF = 100000.0        # standard-state pressure, 1 bar (the Kp (p/p0)^dnu factor)

# Fuel formation enthalpy is the ONE calibration input (it carries the same
# information hPR did). Default pinned so the DERIVED LHV of CH2 + 1.5 O2 -> CO2
# + H2O(gas) equals Mattingly's assumed hPR = 42.8 MJ/kg (=> ~ -34.99 kJ/mol,
# physically reasonable for a liquid-HC (CH2) unit). See rung5 anchor § 2.
_HPR_MATTINGLY = 42.8e6                                   # J/kg
_M_CH2_KG = _M_CH2 / 1000.0                               # kg/mol
_HF_FUEL_DEFAULT = _HPR_MATTINGLY * _M_CH2_KG + _HF298["CO2"] + _HF298["H2O"]  # J/mol


def _formation_products_mass(f: float) -> float:
    """Mass-specific formation enthalpy of the lean-combustion products at f, J/kg.

    Mole-weight the per-species ΔHf298 over the f-dependent composition and divide
    by the product mass: hf_prod = Σ nᵢ ΔHf,i / Σ nᵢ Mᵢ. Only CO2 and H2O carry
    formation (N2/O2/Ar are elements, 0). This is the additive offset the burner
    adds to the SENSIBLE h_t to get the absolute (formation + sensible) enthalpy.
    """
    comp = _products_composition(f)
    H = sum(comp[s] * _HF298[s] for s in comp)               # J per mol-air basis
    m = sum(comp[s] * _SPECIES[s][0] / 1000.0 for s in comp)  # kg per mol-air basis
    return H / m


def _lhv_from_fuel(hf_fuel_molar: float) -> float:
    """Derived lower heating value, J/kg. CH2 + 1.5 O2 -> CO2 + H2O(gas):
    LHV = (ΔHf(CH2) − ΔHf(CO2) − ΔHf(H2O,gas)) / M_CH2  (reactants − products, O2=0).
    """
    return (hf_fuel_molar - _HF298["CO2"] - _HF298["H2O"]) / _M_CH2_KG


def _air_mole_fractions() -> dict:
    """Dry-air mole fractions renormalized to sum EXACTLY 1.

    _AIR sums to 0.9996; the stoichiometry ("per 1 mol of dry air") needs a true
    unit basis, so normalize first. This is load-bearing: the un-normalized 0.9996
    base drifts the product fractions off Mattingly's Ex 6.3 values.
    """
    xsum = sum(_AIR.values())
    return {s: v / xsum for s, v in _AIR.items()}


# Mean molar mass of dry air, g/mol (~28.96), from the normalized fractions.
_M_AIR = sum(x * _SPECIES[s][0] for s, x in _air_mole_fractions().items())

# Stoichiometric fuel/air ratio: the f at which excess O2 hits zero. Solving
# x_O2 - 1.5*n_fuel = 0 with n_fuel = f*M_air/M_CH2 gives f_stoich below (~0.0677,
# vs Mattingly's f_max = 0.0676, 0.15%). Lean combustion (rung-4 scope) is f < this.
_F_STOICH = (_air_mole_fractions()["O2"] / 1.5) * _M_CH2 / _M_AIR


def _products_composition(f: float) -> dict:
    """Lean-complete-combustion product mole numbers per 1 mol dry air, from f.

    CH2 + 1.5 O2 -> CO2 + H2O, dry air oxidizer. n_fuel = f*M_air/M_CH2 is the mol
    of (CH2) burned per mol of air (f = m_fuel/m_air => mol ratio scales by the mass
    ratio M_air/M_CH2). N2 and Ar are inert and pass through; each mol fuel makes one
    CO2 and one H2O and consumes 1.5 O2 from the air's 0.2095. Returns UNNORMALIZED
    mole numbers (per mol air) — _mixture() normalizes. See docs/rung4 § the model.
    """
    x = _air_mole_fractions()
    n_fuel = f * _M_AIR / _M_CH2                     # mol (CH2) per mol air
    comp = {
        "N2": x["N2"],                               # inert, passes through
        "Ar": x["Ar"],                               # inert, passes through
        "CO2": n_fuel,                               # 1 per mol fuel
        "H2O": n_fuel,                               # 1 per mol fuel
        "O2": x["O2"] - 1.5 * n_fuel,                # excess (lean => > 0)
    }
    # LEAN GUARD (rung-4 conservation assert): rich f is out of scope and must
    # trip, not silently produce a negative O2 mole number.
    assert comp["O2"] > 0.0, (
        f"rich mixture f={f:.4f} >= f_stoich={_F_STOICH:.4f}: excess O2 <= 0 "
        "(rich combustion / dissociation is rung 5, out of rung-4 scope)"
    )
    # ATOM CONSERVATION (built once per composition): C, H, O balance. Reactants
    # per mol air: C = H/2 = n_fuel (from CH2), O = 2*x_O2 (from air O2). Products:
    # C in CO2, H in H2O (2 each), O in 2*CO2 + H2O + 2*O2_excess.
    assert abs(comp["CO2"] - n_fuel) < 1e-12, "C balance"
    assert abs(2 * comp["H2O"] - 2 * n_fuel) < 1e-12, "H balance"
    o_in = 2 * x["O2"]
    o_out = 2 * comp["CO2"] + comp["H2O"] + 2 * comp["O2"]
    assert abs(o_out - o_in) < 1e-12, "O balance"
    return comp


def _mixture(fractions: dict) -> Tuple[Tuple[float, ...], Tuple[float, ...], float]:
    """Mole-weight a species mixture into (A_low, A_high, R).

    cp_mass(T) = R * sum_i x_i (Cp_i/Ru)(T) with R = Ru/M_mix, so the mixture's
    effective Cp/R polynomial coefficients are just the mole-weighted species
    coefficients A[k] = sum_i x_i a_i[k] (cp is linear in the coefficients).
    """
    xsum = sum(fractions.values())
    x = {s: f / xsum for s, f in fractions.items()}          # normalize to sum 1
    M = sum(x[s] * _SPECIES[s][0] for s in x)                # g/mol
    R = _Ru / (M / 1000.0)                                   # J/(kg K)
    A_low = tuple(sum(x[s] * _SPECIES[s][1][k] for s in x) for k in range(5))
    A_high = tuple(sum(x[s] * _SPECIES[s][2][k] for s in x) for k in range(5))
    return A_low, A_high, R


def _poly(A: Tuple[float, ...], T: float) -> float:
    """cp(T)/R = A1 + A2 T + A3 T^2 + A4 T^3 + A5 T^4."""
    return A[0] + A[1] * T + A[2] * T ** 2 + A[3] * T ** 3 + A[4] * T ** 4


def _antideriv_h(A: Tuple[float, ...], T: float) -> float:
    """int_0^T (cp/R) dT' — the enthalpy antiderivative through the origin.

    Datum h(0)=0 (zero integration constant). This is load-bearing: the burner is
    the one place enthalpy crosses the cold->hot section boundary, so the relative
    datum does NOT cancel for a dual gas. Datum-0 makes a flat-cp TPG section
    reduce to EXACTLY cp*T (the rung-2 convention). See docs/rung3-variable-cp.md.
    """
    return (A[0] * T + A[1] * T ** 2 / 2 + A[2] * T ** 3 / 3
            + A[3] * T ** 4 / 4 + A[4] * T ** 5 / 5)


def _antideriv_phi(A: Tuple[float, ...], T: float) -> float:
    """int (cp/R)/T' dT' = phi(T)/R. Datum arbitrary (cancels in every pr ratio)."""
    return (A[0] * math.log(T) + A[1] * T + A[2] * T ** 2 / 2
            + A[3] * T ** 3 / 3 + A[4] * T ** 4 / 4)


def _solve(f, fprime, target: float, lo: float = 150.0, hi: float = 4000.0,
           tol: float = 1e-11) -> float:
    """Invert a monotone-increasing f to f(T)=target: safeguarded Newton.

    cp(T) > 0 makes both h and pr strictly increasing, so the root is unique and
    bracketed by [lo, hi]; a Newton step that leaves the bracket falls back to a
    bisection step. Converges in a handful of iterations.
    """
    assert f(lo) - target <= 0.0 <= f(hi) - target, "inverse: root not bracketed"
    x = 0.5 * (lo + hi)
    for _ in range(100):
        fx = f(x) - target
        if fx > 0.0:
            hi = x
        else:
            lo = x
        dfx = fprime(x)
        xn = x - fx / dfx if dfx > 0.0 else 0.5 * (lo + hi)   # Newton, guarded
        if not (lo < xn < hi):
            xn = 0.5 * (lo + hi)                              # bisection fallback
        if abs(xn - x) <= tol * x:
            return xn
        x = xn
    return x


class _CPGSection:
    """A calorically-perfect section: constant (gamma, cp, R), closed-form math.

    Keeps the rung-1/2 forms bit-for-bit (h = cp*T, pr = T^(1/g)), so the
    reduce-to-ideal gate reproduces the prior tables to the digit.
    """

    is_cpg = True

    def __init__(self, gamma: float, cp: float, R: float):
        self.gamma, self._cp, self.R = gamma, cp, R
        self.g = (gamma - 1.0) / gamma           # isentropic exponent (gamma-1)/gamma

    # Every section method carries an ignored far=0.0 so the three section kinds
    # share one interface (docs/rung4 § property interface): only _ReactingSection
    # consults far. CPG/frozen-TPG are composition-independent, so they discard it.
    def cp(self, T: float, far: float = 0.0) -> float:
        return self._cp                          # constant, by definition of CPG
    def h(self, T: float, far: float = 0.0) -> float:
        return self._cp * T
    def pr(self, T: float, far: float = 0.0) -> float:
        return T ** (1.0 / self.g)               # T^(cp/R) in the closed-form limit
    def T_from_h(self, h: float, far: float = 0.0) -> float:
        return h / self._cp
    def T_from_pr(self, pr: float, far: float = 0.0) -> float:
        return pr ** self.g
    def gamma_at(self, T: float, far: float = 0.0) -> float:
        return self.gamma
    def R_at(self, far: float = 0.0) -> float:
        return self.R


class _TPGSection:
    """A thermally-perfect section: cp(T) from a mole-weighted NASA polynomial.

    h and phi are analytic (piecewise across the 1000 K join, so they are
    continuous by construction); the inverses are numerical. R is constant.
    """

    is_cpg = False

    def __init__(self, coeffs: Tuple[Tuple[float, ...], Tuple[float, ...]], R: float):
        self.A_low, self.A_high = coeffs
        self.R = R

    def _A(self, T: float) -> Tuple[float, ...]:
        return self.A_low if T <= _T_BREAK else self.A_high

    # Ignored far=0.0 on every public method — see _CPGSection note. A frozen-TPG
    # section's cp(T) does not depend on composition; _ReactingSection is the one
    # that picks a per-f _TPGSection and delegates.
    def cp(self, T: float, far: float = 0.0) -> float:
        return self.R * _poly(self._A(T), T)

    def h(self, T: float, far: float = 0.0) -> float:
        """int_0^T cp dT', datum h(0)=0, continuous across the 1000 K join."""
        if T <= _T_BREAK:
            return self.R * _antideriv_h(self.A_low, T)
        h_break = _antideriv_h(self.A_low, _T_BREAK)         # low range up to 1000
        return self.R * (h_break + _antideriv_h(self.A_high, T)
                         - _antideriv_h(self.A_high, _T_BREAK))   # then high range above

    def _Phi(self, T: float) -> float:
        """phi(T)/R, continuous across the join (datum arbitrary)."""
        if T <= _T_BREAK:
            return _antideriv_phi(self.A_low, T)
        p_break = _antideriv_phi(self.A_low, _T_BREAK)
        return p_break + _antideriv_phi(self.A_high, T) - _antideriv_phi(self.A_high, _T_BREAK)

    def pr(self, T: float, far: float = 0.0) -> float:
        return math.exp(self._Phi(T))

    def T_from_h(self, h_target: float, far: float = 0.0) -> float:
        T = _solve(self.h, self.cp, h_target)                # dh/dT = cp(T)
        # Round-trip inverse — a STANDING conservation assert (rung-3 gate 2).
        assert abs(self.h(T) - h_target) <= 1e-6 * abs(h_target) + 1e-3, "T_from_h round-trip"
        return T

    def T_from_pr(self, pr_target: float, far: float = 0.0) -> float:
        target = math.log(pr_target)                         # solve Phi(T) = ln(pr)
        T = _solve(self._Phi, lambda t: _poly(self._A(t), t) / t, target)  # dPhi/dT=(cp/R)/T
        assert abs(self._Phi(T) - target) <= 1e-9, "T_from_pr round-trip"
        return T

    def gamma_at(self, T: float, far: float = 0.0) -> float:
        cp = self.cp(T)
        return cp / (cp - self.R)                            # gamma = cp/(cp - R)

    def R_at(self, far: float = 0.0) -> float:
        return self.R


class _ReactingSection:
    """A hot section whose composition — and thus cp(T), R, gamma(T) — tracks f.

    RUNG 4. Each distinct f defines a lean-combustion product mixture
    (_products_composition); mole-weighting it with _mixture() yields the very same
    (A_low, A_high, R) a frozen _TPGSection takes, so this section just builds — and
    MEMOIZES — one _TPGSection per f and delegates every property call to it. The
    integral pr=exp(phi/R) machinery, the guarded-Newton inverses, and their standing
    round-trip asserts are inherited UNCHANGED from _TPGSection; only the mixture
    coefficients now depend on f (docs/rung4-reacting-products.md § the model).

    Memoization: the burner's fixed point evaluates h(Tt4, f) at a few nearby f while
    it converges, then the whole cycle downstream calls at ONE fixed f. Caching the
    per-f _TPGSection (keyed on the exact float f, which is stored in FlowState.far
    and threaded verbatim) makes those downstream calls free. Pure/deterministic: the
    same f always maps to the same section — this is a memo cache, not hidden state.
    """

    is_cpg = False

    def __init__(self):
        self._cache: dict = {}

    def _for(self, far: float) -> _TPGSection:
        sec = self._cache.get(far)
        if sec is None:
            A_low, A_high, R = _mixture(_products_composition(far))
            sec = _TPGSection((A_low, A_high), R)
            self._cache[far] = sec
        return sec

    def cp(self, T: float, far: float = 0.0) -> float:
        return self._for(far).cp(T)
    def h(self, T: float, far: float = 0.0) -> float:
        return self._for(far).h(T)
    def pr(self, T: float, far: float = 0.0) -> float:
        return self._for(far).pr(T)
    def T_from_h(self, h: float, far: float = 0.0) -> float:
        return self._for(far).T_from_h(h)
    def T_from_pr(self, pr: float, far: float = 0.0) -> float:
        return self._for(far).T_from_pr(pr)
    def gamma_at(self, T: float, far: float = 0.0) -> float:
        return self._for(far).gamma_at(T)
    def R_at(self, far: float = 0.0) -> float:
        return self._for(far).R


# --------------------------------------------------------------------------- #
# RUNG 6 — high-temperature dissociation + chemical equilibrium. The complete- #
# combustion _products_composition(f) is replaced (for the equilibrium gas) by #
# a T,p-coupled equilibrium solve: 3 element balances (C,H,O) + 5 reaction Kp   #
# relations for the 8 reacting mole numbers. Kp needs g0=h0-T*s0 on absolute    #
# enthalpy (a6) AND absolute entropy (a7). See docs/rung6-spec.md and           #
# docs/plans/rung6-anchor-equilibrium.md. The datum rule (anchor § 2b): the Kp  #
# solve uses SCALE A (a6-at-298.15, formation) — REQUIRED, not a choice, or the #
# reaction dG0 is wrong; the cycle-burner energy balance uses SCALE B (0K-      #
# sensible + formation, production Fork B) so it reduces to Fork B exactly. Only #
# the datum-free composition (mole numbers) crosses between them.               #
# --------------------------------------------------------------------------- #

# The five basis dissociation reactions (products positive), {species: nu}.
#   CO2 -> CO + 1/2 O2 ; H2O -> H2 + 1/2 O2 ; H2O -> OH + 1/2 H2 ; 1/2 O2 -> O ; 1/2 H2 -> H
_REACTIONS = (
    {"CO2": -1.0, "CO": 1.0, "O2": 0.5},
    {"H2O": -1.0, "H2": 1.0, "O2": 0.5},
    {"H2O": -1.0, "OH": 1.0, "H2": 0.5},
    {"O2": -0.5, "O": 1.0},
    {"H2": -0.5, "H": 1.0},
)
_SP_REACT = ("CO2", "H2O", "CO", "H2", "OH", "O", "H", "O2")   # the 8 unknowns (N2/Ar inert)
_ELEM = {"CO2": (1, 0, 2), "H2O": (0, 2, 1), "CO": (1, 0, 1), "H2": (0, 2, 0),
         "OH": (0, 1, 1), "O": (0, 0, 1), "H": (0, 1, 0), "O2": (0, 0, 2)}   # (C,H,O)


def _sens_h(sp: str, T: float) -> float:
    """int_0^T (cp/Ru) dT' for one species (dimensionless), across the 1000 K join.
    Molar sensible enthalpy = Ru * this (0 K datum, as _TPGSection.h uses per mass)."""
    A_low, A_high = _SPECIES[sp][1], _SPECIES[sp][2]
    if T <= _T_BREAK:
        return _antideriv_h(A_low, T)
    return (_antideriv_h(A_low, _T_BREAK)
            + _antideriv_h(A_high, T) - _antideriv_h(A_high, _T_BREAK))


def _sens_phi(sp: str, T: float) -> float:
    """int (cp/Ru)/T' dT' for one species (dimensionless), across the join. Molar
    sensible entropy = Ru * this; absolute adds Ru*a7."""
    A_low, A_high = _SPECIES[sp][1], _SPECIES[sp][2]
    if T <= _T_BREAK:
        return _antideriv_phi(A_low, T)
    return (_antideriv_phi(A_low, _T_BREAK)
            + _antideriv_phi(A_high, T) - _antideriv_phi(A_high, _T_BREAK))


def _a6_of(sp: str) -> float:
    """Formation constant: H(298.15)=dHf => a6 = dHf/Ru - antideriv_h(A_low, 298.15)."""
    return _HF298[sp] / _Ru - _antideriv_h(_SPECIES[sp][1], _T_REF)


def _a7_of(sp: str) -> float:
    """Absolute-entropy constant: S(298.15)=S298 => a7 = S298/Ru - antideriv_phi(A_low,298.15)."""
    return _S298[sp] / _Ru - _antideriv_phi(_SPECIES[sp][1], _T_REF)


def _h_molar_A(sp: str, T: float) -> float:
    """SCALE A absolute molar enthalpy (a6-at-298.15, formation), J/mol. Kp + AFT only."""
    return _Ru * (_sens_h(sp, T) + _a6_of(sp))


def _s_molar(sp: str, T: float) -> float:
    """Absolute standard-state molar entropy s0(T) at p0=1 bar, J/(mol K). Kp only."""
    return _Ru * (_sens_phi(sp, T) + _a7_of(sp))


def _g_molar(sp: str, T: float) -> float:
    """Absolute standard-state Gibbs energy g0(T)=h0-T*s0, J/mol (scale A). Kp only."""
    return _h_molar_A(sp, T) - T * _s_molar(sp, T)


def _h_molar_B(sp: str, T: float) -> float:
    """SCALE B absolute molar enthalpy (0K-sensible + formation), J/mol. The burner's
    ENERGY-balance datum — matches production Fork B so the cycle reduces to it exactly."""
    return _Ru * _sens_h(sp, T) + _HF298[sp]


def _lnKp(rxn: dict, T: float) -> float:
    """ln Kp(T) = -dG0(T)/(Ru T), dG0 = sum nu*g0 (scale A, datum-free reaction constant)."""
    dG0 = sum(nu * _g_molar(sp, T) for sp, nu in rxn.items())
    return -dG0 / (_Ru * T)


def _gauss_solve(A, b):
    """Solve A x = b by Gaussian elimination with partial pivoting (small dense system)."""
    n = len(A)
    M = [row[:] + [b[i]] for i, row in enumerate(A)]
    for c in range(n):
        piv_row = max(range(c, n), key=lambda r: abs(M[r][c]))
        M[c], M[piv_row] = M[piv_row], M[c]
        piv = M[c][c]
        for r in range(n):
            if r != c and M[r][c] != 0.0:
                fac = M[r][c] / piv
                for k in range(c, n + 1):
                    M[r][k] -= fac * M[c][k]
    return [M[i][n] / M[i][i] for i in range(n)]


def _equil_solve(bC: float, bH: float, bO: float, n_inert: float,
                 T: float, p: float) -> dict:
    """Core equilibrium solve: mole numbers of the 8 reacting species at (T, p) given
    C/H/O atom totals and inert moles. Damped Newton in y=ln(n) (keeps n>0), seeded
    from complete combustion; 3 element balances (C,H,O) + 5 reaction Kp equations.

    Reaction eq r:  sum_i nu_ri (y_i - ln n_tot) + dnu_r ln(p/p0) - lnKp_r = 0,
    with n_tot including the inert species so mole FRACTIONS x_i = n_i/n_tot are right.
    Basis-agnostic (any hydrocarbon C_bC H_bH), so tests reuse it for the methane anchor.

    RUNG 9 — the seed BRANCHES on the O-balance sign. The 8-species system (CO/H2 are
    already unknowns; reactions 1+2 span the water-gas shift) is complete lean OR rich;
    only the SEED must know which side it is on. Lean (`bO >= 2bC + bH/2`, the full-
    oxidation O demand) keeps the byte-identical rung-6 expression, so every rung-1..8
    path — all lean (cycle burner φ≈0.41) — takes an unchanged Newton trajectory and
    reduce-to-rung-8 is bit-for-bit by construction. Rich (`bO <` that) swaps in an
    O-limited seed: water first (H2O favored), all C→CO, then upgrade CO→CO2 with the
    leftover O. The lean seed's `O2 = (bO−2bC−bH/2)/2` goes negative when rich (floored
    to 1e-8) and grossly violates the O balance, so damped Newton is fragile there;
    the O-limited seed converges cleanly to the φ_p≤2 soot bound (docs/rung9-spec.md).
    """
    if bO >= 2.0 * bC + bH / 2.0:
        # LEAN — byte-identical to the rung-6 seed (C->CO2, H->H2O, leftover O2; radicals
        # tiny). Untouched so the whole lean cycle keeps its exact Newton path (reduce gate).
        seed = {"CO2": max(bC, 1e-12), "H2O": max(bH / 2.0, 1e-12),
                "CO": 1e-8, "H2": 1e-8, "OH": 1e-8, "O": 1e-9, "H": 1e-9,
                "O2": max((bO - 2.0 * bC - bH / 2.0) / 2.0, 1e-8)}
    else:
        # RICH (rung 9) — O-limited allocation, atom-conserving: water first (min(bH/2, bO)),
        # then all C->CO, upgrade CO->CO2 with the O left over; any H beyond the O supply
        # stays H2. O2 and radicals tiny (equilibrium O2 IS ~0 rich). See docs/rung9-spec.md.
        n_h2o = min(bH / 2.0, bO)
        o_left = bO - n_h2o
        n_co2 = min(bC, max(o_left - bC, 0.0))   # leftover O upgrades CO->CO2
        seed = {"CO2": max(n_co2, 1e-12), "H2O": max(n_h2o, 1e-12),
                "CO": max(bC - n_co2, 1e-12), "H2": max(bH / 2.0 - n_h2o, 1e-12),
                "OH": 1e-8, "O": 1e-9, "H": 1e-9, "O2": 1e-8}
    y = {s: math.log(seed[s]) for s in _SP_REACT}
    lnKp = [_lnKp(r, T) for r in _REACTIONS]
    lnpr = math.log(p / _P_REF)

    converged = False
    for _ in range(200):
        nv = {s: math.exp(y[s]) for s in _SP_REACT}
        ntot = sum(nv.values()) + n_inert
        F = [sum(_ELEM[s][k] * nv[s] for s in _SP_REACT) - b
             for k, b in enumerate((bC, bH, bO))]
        for r, K in zip(_REACTIONS, lnKp):
            dnu = sum(r.values())
            F.append(sum(nu * (y[s] - math.log(ntot)) for s, nu in r.items()) + dnu * lnpr - K)
        # Jacobian dF/dy (y_j = ln n_j; dn_j/dy_j = n_j).
        J = [[0.0] * 8 for _ in range(8)]
        for j, sj in enumerate(_SP_REACT):
            for k in range(3):                       # element rows
                J[k][j] = _ELEM[sj][k] * nv[sj]
        for ri, r in enumerate(_REACTIONS):          # reaction rows
            dnu = sum(r.values())
            for j, sj in enumerate(_SP_REACT):
                J[3 + ri][j] = r.get(sj, 0.0) - dnu * (nv[sj] / ntot)
        dy = _gauss_solve(J, [-v for v in F])
        step = max(abs(d) for d in dy)
        scale = 1.0 if step < 1.0 else 1.0 / step    # damping: cap the log-step at 1
        for j, sj in enumerate(_SP_REACT):
            y[sj] = max(y[sj] + scale * dy[j], -80.0)   # floor: n >= ~1e-35 (trace species)
        if step * scale < 1e-13:
            converged = True
            break

    # CONVERGENCE (rung-6 standing assert, the Newton twin of the burner's fixed-point
    # `else: assert False`): the atom balances below can hold with the log-Kp residuals
    # still open, so guard the FULL solve explicitly (measured ~10-20 steps, far under 200).
    assert converged, f"equilibrium Newton did not converge in 200 steps at (T={T}, p={p})"

    comp = {s: math.exp(y[s]) for s in _SP_REACT}
    # ATOM CONSERVATION (rung-6 standing assert): the solver enforces C,H,O as
    # equations, so a converged run closes them — this catches a non-converged solve.
    assert abs(sum(_ELEM[s][0] * comp[s] for s in _SP_REACT) - bC) < 1e-9 * (bC + 1e-9), "C balance"
    assert abs(sum(_ELEM[s][1] * comp[s] for s in _SP_REACT) - bH) < 1e-9 * (bH + 1e-9), "H balance"
    assert abs(sum(_ELEM[s][2] * comp[s] for s in _SP_REACT) - bO) < 1e-9 * bO, "O balance"
    return comp


def _equilibrium_composition(f: float, T: float, p: float) -> dict:
    """Equilibrium mole numbers per mol dry air at (f, T, p) for the (CH2)n fuel.
    Returns the 8 reacting species + the inert N2/Ar. Wraps _equil_solve with the
    (CH2)n atom basis: n_fuel=f*M_air/M_CH2 => C=n_fuel, H=2*n_fuel, O=2*x_O2."""
    x = _air_mole_fractions()
    n_fuel = f * _M_AIR / _M_CH2
    comp = _equil_solve(n_fuel, 2.0 * n_fuel, 2.0 * x["O2"], x["N2"] + x["Ar"], T, p)
    comp["N2"], comp["Ar"] = x["N2"], x["Ar"]
    return comp


class _EquilibriumSection:
    """A hot section whose composition is the EQUILIBRIUM mixture at the burner's
    (Tt4, pt4), FROZEN through the turbine and nozzle (rung 6, frozen-downstream).

    Like _ReactingSection it delegates every property call to a memoized per-far
    _TPGSection built via _mixture() (so R_t tracks the dissociation mole-count shift)
    — but the composition comes from _equilibrium_composition(far, Tt4, pt4), so the
    burner must freeze() it before any downstream call. The (Tt4, pt4) is baked in at
    freeze time; downstream calls key on far alone (the frozen mixture is independent
    of the evaluation T — the turbine asks at Tt5, the nozzle at T9). Reusing one Gas
    across two burn configs with the same far but different (Tt4,pt4) trips the guard
    (restores "pure function of far for a fixed burn config" — no hidden state).
    """

    is_cpg = False

    def __init__(self):
        self._cache: dict = {}          # far -> _TPGSection (frozen station-4 mixture)
        self._comp: dict = {}           # far -> composition dict (diagnostics/asserts)
        self._burn: Optional[Tuple[float, float]] = None   # (Tt4, pt4) of first freeze

    def freeze(self, far: float, T_burn: float, p_burn: float) -> dict:
        if self._burn is None:
            self._burn = (T_burn, p_burn)
        else:
            assert (abs(self._burn[0] - T_burn) < 1e-9 * T_burn
                    and abs(self._burn[1] - p_burn) < 1e-6 * p_burn), (
                "equilibrium section: burn condition changed on a reused Gas "
                f"(had {self._burn}, got {(T_burn, p_burn)})"
            )
        if far not in self._cache:
            comp = _equilibrium_composition(far, T_burn, p_burn)
            A_low, A_high, R = _mixture(comp)
            self._cache[far] = _TPGSection((A_low, A_high), R)
            self._comp[far] = comp
        return self._comp[far]

    def _for(self, far: float) -> _TPGSection:
        sec = self._cache.get(far)
        assert sec is not None, (
            f"equilibrium hot section not frozen for far={far}: the burner must run "
            "(freeze the station-4 mixture) before any downstream property call"
        )
        return sec

    def cp(self, T: float, far: float = 0.0) -> float:
        return self._for(far).cp(T)
    def h(self, T: float, far: float = 0.0) -> float:
        return self._for(far).h(T)
    def pr(self, T: float, far: float = 0.0) -> float:
        return self._for(far).pr(T)
    def T_from_h(self, h: float, far: float = 0.0) -> float:
        return self._for(far).T_from_h(h)
    def T_from_pr(self, pr: float, far: float = 0.0) -> float:
        return self._for(far).T_from_pr(pr)
    def gamma_at(self, T: float, far: float = 0.0) -> float:
        return self._for(far).gamma_at(T)
    def R_at(self, far: float = 0.0) -> float:
        return self._for(far).R


# --------------------------------------------------------------------------- #
# RUNG 7 — thermal NOx: the extended Zeldovich mechanism (kinetically-limited). #
# NO is a TRACE (ppm) species, so it is a DECOUPLED diagnostic layer on top of  #
# the rung-6 cycle — the cycle stays bit-for-bit rung 6. Unlike the major       #
# species, NO does NOT reach equilibrium in a combustor: it is rate-limited,    #
# kinetically frozen far below its equilibrium value at realistic residence     #
# times (the lesson INVERTS rung 6). See docs/rung7-spec.md and                 #
# docs/plans/rung7-anchor-nox.md. Everything here reads the FROZEN rung-6 pool   #
# (O, O2, OH, H, N2) and reuses the scale-A _g_molar (a6+a7) substrate for Kp —  #
# NO/N are NEVER added to _equil_solve (keeps reduce-to-rung-6 automatic).       #
# --------------------------------------------------------------------------- #

# Extended Zeldovich mechanism (Hanson & Salimian 1984, as tabulated in Turns):
#   1: O + N2 <=> NO + N   2: N + O2 <=> NO + O   3: N + OH <=> NO + H
# k = A * T^n * exp(-theta/T), native cm^3/mol/s -> SI m^3/(mol s) via *1e-6.
# Every reaction is mole-conserving (Dnu=0), so Kc=Kp with NO (p/p0) factor.
_ZELDOVICH = {
    "1f": (1.8e14, 0.0, 38370.0), "1r": (3.8e13, 0.0, 425.0),
    "2f": (1.8e10, 1.0, 4680.0),  "2r": (3.8e9, 1.0, 20820.0),
    "3f": (7.1e13, 0.0, 450.0),   "3r": (1.7e14, 0.0, 24560.0),
}
_M_NO = _SPECIES["NO"][0] / 1000.0     # kg/mol


def _k_zeldovich(key: str, T: float) -> float:
    """Zeldovich rate constant k(T) in SI m^3/(mol s)."""
    A, n, theta = _ZELDOVICH[key]
    return A * (T ** n) * math.exp(-theta / T) * 1e-6


def _kp_no(T: float) -> float:
    """Kp(1/2 N2 + 1/2 O2 <=> NO) = exp(-dG0/RuT), dG0 = g(NO) - 1/2 g(N2) - 1/2 g(O2).
    Dnu=0 => NO (p/p0) factor: equilibrium NO is PRESSURE-INDEPENDENT (inverts rung 6)."""
    dG0 = _g_molar("NO", T) - 0.5 * _g_molar("N2", T) - 0.5 * _g_molar("O2", T)
    return math.exp(-dG0 / (_Ru * T))


def _equilibrium_no_fraction(comp: dict, T: float) -> float:
    """Superimposed equilibrium NO mole fraction from the frozen rung-6 mixture `comp`
    (mole numbers per mol air). NO is trace: it does NOT perturb `comp`. x_NO_e =
    Kp_NO * sqrt(x_N2 x_O2) using that mixture's own N2/O2 (N is negligible, ~1e-5 of NO)."""
    ntot = sum(comp.values())
    return _kp_no(T) * math.sqrt(comp["N2"] / ntot * comp["O2"] / ntot)


def _kcheck_ratio(T: float) -> float:
    """Thermo-kinetic K-check: (k1f k2f)/(k1r k2r) vs Kc(N2+O2<=>2NO)=exp(-dG0/RuT),
    dG0 = 2 g(NO) - g(N2) - g(O2). Reactions 1+2 sum to N2+O2<=>2NO, so detailed
    balance ties the transcribed RATE CONSTANTS to the a6/a7 THERMO (N cancels). The
    ratio is dimensionless (the 1e-6 SI factor cancels). Measured ~1.035-1.044."""
    kc_rate = ((_k_zeldovich("1f", T) * _k_zeldovich("2f", T))
               / (_k_zeldovich("1r", T) * _k_zeldovich("2r", T)))
    dG0 = 2 * _g_molar("NO", T) - _g_molar("N2", T) - _g_molar("O2", T)
    return kc_rate / math.exp(-dG0 / (_Ru * T))


@dataclass
class NOxState:
    """Thermal-NO diagnostic at one (frozen pool, T, p, tau). A pure DIAGNOSTIC — it
    never feeds the cycle. Mole fractions; rates in mol/m^3/s; EI in g NO / kg fuel."""
    x_no: float          # kinetic NO mole fraction after residence time tau
    x_no_eq: float       # equilibrium NO mole fraction (the ceiling)
    initial_rate: float  # d[NO]/dt at t=0 = 2 k1f [O]_e [N2]_e, mol/m^3/s
    char_time: float     # tau_NO = [NO]_e / initial_rate, s (>> residence => frozen)
    ei_no: float         # emission index, g NO / kg fuel

    @property
    def ppm(self) -> float:
        return self.x_no * 1e6

    @property
    def ppm_eq(self) -> float:
        return self.x_no_eq * 1e6

    @property
    def fraction_of_equil(self) -> float:
        return self.x_no / self.x_no_eq


def _thermal_no(comp: dict, T: float, p: float, tau: float, far: float,
                nsteps: int = 4000) -> NOxState:
    """Kinetic NO after residence time tau on the frozen pool `comp` at (T, p).

    One-equation extended-Zeldovich model (Heywood/Turns), QSS on N, REVERSE-RATE form
    for R2/R3 so equilibrium [N] is never needed (uses the pool's own O, H):
        d[NO]/dt = 2 R1 (1 - a^2)/(1 + a R1/(R2+R3)),  a = [NO]/[NO]_e
        R1 = k1f[O][N2],  R2 = k2r[NO]_e[O],  R3 = k3r[NO]_e[H]
    a=0 -> rate=2 R1 (initial rate); a->1 -> rate=0 (saturates at [NO]_e, so tau->inf
    recovers the equilibrium NO — an internal consistency gate). RK4 from [NO]=0.
    """
    # K-CHECK (rung-7 standing assert, on every diagnostic run): the transcribed rate
    # constants must agree with the a6/a7 thermo at the evaluation T (the twin of rung 6's
    # atom-balance assert). A gross transcription slip is orders of magnitude off.
    kr = _kcheck_ratio(T)
    assert 0.90 < kr < 1.15, f"Zeldovich K-check off: ratio {kr:.4f} at T={T}"

    ntot = sum(comp.values())
    conc = p / (_Ru * T)                         # total molar concentration, mol/m^3
    x = {s: comp[s] / ntot for s in comp}
    cO = x.get("O", 0.0) * conc
    cN2 = x["N2"] * conc
    cH = x.get("H", 0.0) * conc
    x_no_eq = _equilibrium_no_fraction(comp, T)
    cNOe = x_no_eq * conc
    # TRACE guard (rung-7): NO must be trace for the decoupled-diagnostic assumption.
    assert x_no_eq < 0.02, f"NO not trace (x_NO_e={x_no_eq:.4g}) — decoupling invalid"

    R1 = _k_zeldovich("1f", T) * cO * cN2
    R2 = _k_zeldovich("2r", T) * cNOe * cO
    R3 = _k_zeldovich("3r", T) * cNOe * cH
    beta = R1 / (R2 + R3) if (R2 + R3) > 0.0 else 0.0

    def rate(cNO: float) -> float:
        a = cNO / cNOe
        return 2.0 * R1 * (1.0 - a * a) / (1.0 + beta * a)

    dt = tau / nsteps
    cNO = 0.0
    for _ in range(nsteps):
        k1 = rate(cNO)
        k2 = rate(cNO + 0.5 * dt * k1)
        k3 = rate(cNO + 0.5 * dt * k2)
        k4 = rate(cNO + dt * k3)
        cNO += dt / 6.0 * (k1 + 2 * k2 + 2 * k3 + k4)
        if cNO > cNOe:
            cNO = cNOe                           # clamp: never overshoot equilibrium
    # STANDING assert: the integrator stays in [0, [NO]_e].
    assert -1e-12 <= cNO <= cNOe * (1.0 + 1e-9), f"kinetic NO out of [0,eq]: {cNO} vs {cNOe}"

    x_no = cNO / conc
    # Emission index, g NO / kg fuel: NO moles per mol air = x_no*ntot; fuel mass = n_fuel*M_CH2.
    n_fuel = far * _M_AIR / _M_CH2
    ei = 1000.0 * (x_no * ntot * _M_NO) / (n_fuel * _M_CH2_KG) if n_fuel > 0.0 else 0.0
    return NOxState(x_no=x_no, x_no_eq=x_no_eq, initial_rate=2.0 * R1,
                    char_time=(cNOe / (2.0 * R1) if R1 > 0.0 else math.inf), ei_no=ei)


# --- Rung-8 two-zone (primary -> dilution) NOx helpers (all on rung-6/7 primitives) --- #
def _h_air_molar_A(T: float) -> float:
    """Scale-A molar enthalpy of 1 mol air at T (sum over air species, formation datum).
    N2/O2/Ar carry zero formation enthalpy; the trace CO2 in air carries its ΔHf298. The
    same absolute (scale-A) datum the rung-6 AFT diagnostic used — no new convention."""
    return sum(x * _h_molar_A(s, T) for s, x in _air_mole_fractions().items())


def _primary_aft(far_p: float, p: float, T_air: float, hf_fuel: float) -> float:
    """Adiabatic flame temp of (fuel + 1 mol primary air), air PREHEATED to T_air.

    Bisection on T so the equilibrium products' scale-A enthalpy equals the reactants':
        Σ nᵢ(far_p, T)·h̄ᵢ_A(T) = h̄_air_A(T_air) + n_fuel·hf_fuel.
    Rung-8: the primary burns all the fuel with only its share of the air, preheated to
    the ACTUAL compressor-exit Tt3 (not 298 K). Preheating from Tt3 is what ties the
    primary flame to the running cycle and makes the α→1 reduce-to-rung-7 gate exact
    (docs/rung8-spec.md § reduce gate). Promotes the test-only rung-6 `_aft` helper to a
    reusable primary-AFT — but starting from Tt3, not 298 K."""
    n_fuel = far_p * _M_AIR / _M_CH2
    H_react = _h_air_molar_A(T_air) + n_fuel * hf_fuel
    lo, hi = 800.0, 3200.0
    for _ in range(100):
        T = 0.5 * (lo + hi)
        comp = _equilibrium_composition(far_p, T, p)
        H_prod = sum(comp[s] * _h_molar_A(s, T) for s in comp)
        lo, hi = (lo, T) if H_prod > H_react else (T, hi)
    return 0.5 * (lo + hi)


def _mixed_out_T(comp_prim: dict, T_prim: float, alpha: float,
                 far_ov: float, T_dilution: float, p: float) -> float:
    """Mixed-out temperature after adding (1−α) mol dilution air at T_dilution to α mol
    primary products and RE-EQUILIBRATING the major species at the overall far_ov.

    Basis: 1 mol TOTAL air. Primary products are per-mol-PRIMARY-air, so scale by α.
    Enthalpy is conserved; bisection on T_mix so the re-equilibrated pool's scale-A
    enthalpy equals α·H_primary + (1−α)·H_dilution_air. Re-equilibrating (NOT freezing)
    the dissociated primary majors RELEASES the stored dissociation energy, so T_mix
    returns to ≈ Tt4 — the rung-8 conservation gate (docs/rung8-spec.md § mix-out). By
    α·far_p = far_ov, α cancels in the balance, so T_mix is split-independent by
    construction (the overall adiabatic flame temp from Tt3)."""
    H_prim = alpha * sum(comp_prim[s] * _h_molar_A(s, T_prim) for s in comp_prim)
    H_dil = (1.0 - alpha) * _h_air_molar_A(T_dilution)
    H_mix = H_prim + H_dil
    lo, hi = 700.0, 3200.0
    for _ in range(100):
        T = 0.5 * (lo + hi)
        comp = _equilibrium_composition(far_ov, T, p)
        H_prod = sum(comp[s] * _h_molar_A(s, T) for s in comp)
        lo, hi = (lo, T) if H_prod > H_mix else (T, hi)
    return 0.5 * (lo + hi)


@dataclass
class ZonedNOxState:
    """Two-zone (primary → dilution) thermal-NO diagnostic (rung 8). A pure DIAGNOSTIC —
    like NOxState it never feeds the cycle. NO is set in the hot primary and FROZEN through
    the dilution that cools the gas to T_mix ≈ Tt4; EI_NO is a per-kg-fuel quantity set in
    the primary (dilution lowers the mole FRACTION, not the emission INDEX)."""
    phi_primary: float   # primary equivalence ratio (≤ 2, lean-to-rich RQL scope; rung 9)
    far_primary: float   # primary fuel/air ratio = phi_primary * f_stoich
    alpha: float         # fraction of the air routed to the primary (≤ 1)
    T_primary: float     # adiabatic primary flame temperature, K (from Tt3)
    T_mix: float         # mixed-out temperature after dilution, K (≈ Tt4)
    primary: NOxState    # the rung-7 NO diagnostic evaluated ON the hot primary pool
    x_no_mix: float      # NO mole fraction after dilution (frozen moles / mixed total moles)

    @property
    def ei_no(self) -> float:
        """Emission index, g NO / kg fuel — set in the primary, conserved through dilution
        (α cancels: NO moles and fuel moles both scale with the primary air fraction)."""
        return self.primary.ei_no

    @property
    def ppm_primary(self) -> float:
        return self.primary.x_no * 1e6

    @property
    def ppm_mix(self) -> float:
        return self.x_no_mix * 1e6


@dataclass
class Gas:
    """Dual-section gas. Each section is CPG (constant triple, default) or TPG (cp(T)).

    Cold section (gamma_c, cp_c, R_c) applies upstream of the burner (0->3); hot
    section (gamma_t, cp_t, R_t) applies downstream (4->9). The burner is the
    hand-off. R is NOT independent of (gamma, cp): the perfect-gas relation is
    R = (gamma-1)/gamma * cp. We keep R explicit so a CPG case can pin the exact
    constants its reference used — rung 1's table used cp=1004 and a rounded R=287
    that is ~0.05% off the relation, and reduce-to-ideal must reproduce it exactly.

    DEFAULTS = rung 1: hot == cold (one gas), gamma=1.4, cp=1004, R=287. So `Gas()`
    is the rung-1 cold-air-standard gas. For a THERMALLY-perfect dual gas (NASA
    air + lean products) use the `Gas.thermally_perfect()` factory.

    A section is TPG when its cp_*_coeffs is set ((A_low, A_high) Cp/R polynomials,
    with R taken from R_c / R_t); otherwise it is CPG built from the triple. The
    property interface (h_c/pr_c/T_from_h_c/T_from_pr_c/gamma_c_at and the _t twins)
    routes to whichever — see the module docstring § the trap.
    """

    # Cold section (stations 0 -> 3): fresh air.
    gamma_c: float = 1.4
    cp_c: float = 1004.0     # J/(kg K)  (CPG only; ignored for a TPG cold section)
    R_c: float = 287.0       # J/(kg K)
    # Hot section (stations 4 -> 9): combustion products. Defaults equal cold so an
    # unconfigured Gas behaves exactly like the rung-1 single gas.
    gamma_t: float = 1.4
    cp_t: float = 1004.0     # J/(kg K)  (CPG only)
    R_t: float = 287.0       # J/(kg K)

    hPR: float = 42.8e6      # fuel heating value, J/kg

    # TPG models (rung 3). None => the section is calorically perfect (the default,
    # so rungs 1-2 are untouched). Each is (A_low, A_high) Cp/R polynomial coeffs.
    cp_c_coeffs: Optional[Tuple[Tuple[float, ...], Tuple[float, ...]]] = None
    cp_t_coeffs: Optional[Tuple[Tuple[float, ...], Tuple[float, ...]]] = None

    # Reacting hot section (rung 4). When True the hot section's composition — and
    # therefore cp_t/R_t/gamma_t — tracks the fuel/air ratio f; the R_t/gamma_t/cp_t
    # scalars above then hold only REPRESENTATIVE values (at f_design) for diagnostics.
    # Set by the Gas.reacting() factory; the frozen paths leave it False (untouched).
    reacting_hot: bool = False

    # Fork B (rung 5). When True the burner DERIVES its heat release from formation
    # enthalpies (absolute-enthalpy balance) instead of the assumed hPR; hf_fuel_molar
    # is the single calibration input and hPR is set to the DERIVED LHV. Fork A leaves
    # both untouched. See docs/rung5-fork-b.md.
    fork_b: bool = False
    hf_fuel_molar: Optional[float] = None    # fuel ΔHf298, J/mol (Fork B only)

    # Equilibrium hot section (rung 6). When True the hot composition is the CHEMICAL-
    # EQUILIBRIUM mixture at the burner's (Tt4, pt4) — dissociation included — frozen
    # downstream. Implies fork_b (the burner works on absolute enthalpies). Set by the
    # Gas.reacting_equilibrium() factory; every lower-rung path leaves it False.
    equilibrium: bool = False

    def __post_init__(self):
        self._cold = (_TPGSection(self.cp_c_coeffs, self.R_c) if self.cp_c_coeffs
                      else _CPGSection(self.gamma_c, self.cp_c, self.R_c))
        if self.equilibrium:
            self._hot = _EquilibriumSection()                # composition = equilibrium(f,T,p) (rung 6)
        elif self.reacting_hot:
            self._hot = _ReactingSection()                   # composition tracks f (rung 4)
        elif self.cp_t_coeffs:
            self._hot = _TPGSection(self.cp_t_coeffs, self.R_t)   # frozen cp(T) (rung 3)
        else:
            self._hot = _CPGSection(self.gamma_t, self.cp_t, self.R_t)  # constant cp (rungs 1-2)

    # --- classmethod factory: the thermally-perfect dual gas ------------------

    @classmethod
    def thermally_perfect(cls, hPR: float = 42.8e6) -> "Gas":
        """NASA-air cold section + lean-products hot section (rung 3)."""
        Alo_c, Ahi_c, R_c = _mixture(_AIR)
        Alo_t, Ahi_t, R_t = _mixture(_PRODUCTS)
        return cls(R_c=R_c, R_t=R_t, hPR=hPR,
                   cp_c_coeffs=(Alo_c, Ahi_c), cp_t_coeffs=(Alo_t, Ahi_t))

    @classmethod
    def reacting(cls, f_design: float = 0.0, hPR: float = 42.8e6) -> "Gas":
        """NASA-air cold section + REACTING hot section whose composition tracks f (rung 4).

        The hot section is a _ReactingSection: at each f it builds the lean-combustion
        product mixture (_products_composition) and mole-weights it (docs/rung4). The
        R_t/gamma_t/cp_t scalars are set to REPRESENTATIVE values at f_design (used only
        for diagnostics / .unified() — every live property call routes through the
        f-parameterized interface, so f_design does not affect the cycle result). The
        cold section is the same NASA air as thermally_perfect(); the burner writes the
        converged f into FlowState.far, threaded to every hot-section call downstream.
        """
        Alo_c, Ahi_c, R_c = _mixture(_AIR)
        _, _, R_t = _mixture(_products_composition(f_design))    # representative, at f_design
        return cls(R_c=R_c, R_t=R_t, hPR=hPR,
                   cp_c_coeffs=(Alo_c, Ahi_c), reacting_hot=True)

    @classmethod
    def reacting_forkb(cls, hf_fuel_molar: float = _HF_FUEL_DEFAULT,
                       f_design: float = 0.0) -> "Gas":
        """Reacting hot section with FORMATION-ENTHALPY bookkeeping (rung 5, Fork B).

        Same reacting composition as Gas.reacting() (rung 4), but the burner derives
        its heat release from an absolute-enthalpy balance on formation enthalpies
        rather than the assumed hPR. hf_fuel_molar (the fuel ΔHf298, the ONE calibration
        input) DEFAULTS to the value that reproduces Mattingly's hPR = 42.8 MJ/kg — so
        the derived LHV falls out at 42.8 and Fork B reproduces rung-4 Fork A EXACTLY
        for complete combustion (docs/plans/rung5-anchor-formation.md § 3). hPR is set
        to the derived LHV so downstream/diagnostics see the derived value.
        """
        Alo_c, Ahi_c, R_c = _mixture(_AIR)
        _, _, R_t = _mixture(_products_composition(f_design))
        lhv = _lhv_from_fuel(hf_fuel_molar)                       # DERIVED heating value
        return cls(R_c=R_c, R_t=R_t, hPR=lhv, cp_c_coeffs=(Alo_c, Ahi_c),
                   reacting_hot=True, fork_b=True, hf_fuel_molar=hf_fuel_molar)

    @classmethod
    def reacting_equilibrium(cls, hf_fuel_molar: float = _HF_FUEL_DEFAULT,
                             f_design: float = 0.0) -> "Gas":
        """Reacting hot section with CHEMICAL-EQUILIBRIUM (dissociating) products (rung 6).

        Extends Fork B (Gas.reacting_forkb): same absolute-enthalpy burner, but the
        products DISSOCIATE — the composition at the burner's (Tt4, pt4) is the chemical-
        equilibrium mixture (CO2/H2O partly split into CO/H2/OH/O/H) solved from Kp(T),
        then FROZEN through turbine + nozzle. Reduces to Fork B exactly in the cold-Tt4
        limit (dissociation -> 0). hf_fuel_molar is the same single calibration input.
        See docs/rung6-spec.md and docs/plans/rung6-anchor-equilibrium.md.
        """
        Alo_c, Ahi_c, R_c = _mixture(_AIR)
        _, _, R_t = _mixture(_products_composition(f_design))     # representative (diagnostics)
        lhv = _lhv_from_fuel(hf_fuel_molar)
        return cls(R_c=R_c, R_t=R_t, hPR=lhv, cp_c_coeffs=(Alo_c, Ahi_c),
                   reacting_hot=True, fork_b=True, equilibrium=True,
                   hf_fuel_molar=hf_fuel_molar)

    # --- isentropic exponents (CPG closed-form helpers; used by station 0/9) ---

    @property
    def g_c(self) -> float:
        """Cold isentropic exponent g = (gamma-1)/gamma; 1/g = gamma/(gamma-1)."""
        return (self.gamma_c - 1.0) / self.gamma_c

    @property
    def g_t(self) -> float:
        """Hot isentropic exponent g = (gamma-1)/gamma."""
        return (self.gamma_t - 1.0) / self.gamma_t

    # --- the property interface (components call THESE, not cp / g directly) ---

    @property
    def cold_is_cpg(self) -> bool:
        return self.cp_c_coeffs is None

    @property
    def hot_is_cpg(self) -> bool:
        # A reacting hot section is NOT calorically perfect — it must route through
        # the TPG/integral branch (Nozzle, freestream). Guarding on reacting_hot too
        # is load-bearing: reacting sets no cp_t_coeffs, so without it the Nozzle would
        # silently take the constant-gamma CPG branch and look plausible.
        return self.cp_t_coeffs is None and not self.reacting_hot

    def cp_c_at(self, T: float) -> float:
        return self._cold.cp(T)
    def h_c(self, T: float) -> float:
        return self._cold.h(T)
    def pr_c(self, T: float) -> float:
        return self._cold.pr(T)
    def T_from_h_c(self, h: float) -> float:
        return self._cold.T_from_h(h)
    def T_from_pr_c(self, pr: float) -> float:
        return self._cold.T_from_pr(pr)
    def gamma_c_at(self, T: float) -> float:
        return self._cold.gamma_at(T)

    # Hot-section property interface. Each carries far (default 0.0), the fuel/air
    # ratio that fixes the reacting composition (rung 4). CPG/frozen-TPG sections
    # ignore it, so the signature change is additive and reduce-to-ideal is untouched;
    # a REACTING section selects its per-f mixture. Downstream of the burner, callers
    # pass state.far (docs/rung4-reacting-products.md § property interface).
    def cp_t_at(self, T: float, far: float = 0.0) -> float:
        return self._hot.cp(T, far)
    def h_t(self, T: float, far: float = 0.0) -> float:
        return self._hot.h(T, far)
    def pr_t(self, T: float, far: float = 0.0) -> float:
        return self._hot.pr(T, far)
    def T_from_h_t(self, h: float, far: float = 0.0) -> float:
        return self._hot.T_from_h(h, far)
    def T_from_pr_t(self, pr: float, far: float = 0.0) -> float:
        return self._hot.T_from_pr(pr, far)
    def gamma_t_at(self, T: float, far: float = 0.0) -> float:
        return self._hot.gamma_at(T, far)
    def R_t_at(self, far: float = 0.0) -> float:
        """Hot-section gas constant. Constant for CPG/frozen-TPG (== R_t); for a
        reacting section it decreases slightly with f (heavier products)."""
        return self._hot.R_at(far)

    # --- Fork B absolute-enthalpy interface (rung 5; the BURNER alone uses it) ---
    # These carry the formation offset a6 that the sensible h_c/h_t deliberately omit.
    # Only the burner (the one cross-section enthalpy subtraction) needs absolute
    # values; turbine/nozzle use enthalpy DIFFERENCES where the offset cancels, so
    # they stay on the sensible interface, bit-for-bit rung 4.

    @property
    def lhv(self) -> float:
        """Derived lower heating value, J/kg (Fork B). == hPR by construction; for a
        non-Fork-B gas there is no fuel enthalpy, so fall back to the assumed hPR."""
        return _lhv_from_fuel(self.hf_fuel_molar) if self.fork_b else self.hPR

    @property
    def hf_fuel_mass(self) -> float:
        """Fuel formation enthalpy per unit fuel mass, J/kg (Fork B only)."""
        assert self.fork_b and self.hf_fuel_molar is not None, "hf_fuel_mass: not Fork B"
        return self.hf_fuel_molar / _M_CH2_KG

    def hf_products_mass(self, far: float) -> float:
        """Formation enthalpy of the products at far, per unit product mass, J/kg."""
        return _formation_products_mass(far)

    def h_c_abs(self, T: float) -> float:
        """Absolute cold-air enthalpy = sensible (air is elements, formation 0)."""
        return self.h_c(T)                              # air formation datum == 0

    def h_t_abs(self, T: float, far: float) -> float:
        """Absolute hot-products enthalpy = sensible h_t + product formation offset."""
        return self.h_t(T, far) + _formation_products_mass(far)

    # --- Rung-6 equilibrium-burner interface (the BURNER alone uses it) ----------
    # The burner root-finds f on the SCALE-B absolute-enthalpy balance (per mol air),
    # with the equilibrium composition solved at each trial f. Scale B (0K-sensible +
    # formation) is production Fork B's datum, so the cycle reduces to Fork B exactly
    # when dissociation is off; the composition comes from the scale-A Kp solve (the
    # only object crossing over is datum-free mole numbers). See docs/rung6-spec.md.

    @property
    def lhv_molar(self) -> float:
        """Complete-combustion lower heating value per mol fuel, J/mol (the eta_b loss basis)."""
        return self.lhv * _M_CH2_KG

    @property
    def f_stoich_lean(self) -> float:
        """Lean stoichiometric fuel/air ratio — the burner's f-bracket upper bound."""
        return _F_STOICH

    def n_fuel_per_air(self, f: float) -> float:
        """Mol of (CH2) burned per mol dry air at fuel/air ratio f."""
        return f * _M_AIR / _M_CH2

    def equilibrium_composition(self, f: float, T: float, p: float) -> dict:
        """Chemical-equilibrium product mole numbers per mol air at (f, T, p)."""
        return _equilibrium_composition(f, T, p)

    def h_air_abs_B(self, T: float) -> float:
        """Air absolute molar enthalpy per mol air, SCALE B (formation of air = 0)."""
        return sum(x * _h_molar_B(s, T) for s, x in _air_mole_fractions().items())

    def h_products_abs_B(self, comp: dict, T: float) -> float:
        """Products absolute molar enthalpy per mol air, SCALE B, over a composition dict."""
        return sum(n * _h_molar_B(s, T) for s, n in comp.items())

    def freeze_equilibrium(self, f: float, T_burn: float, p_burn: float) -> dict:
        """Freeze the station-4 equilibrium mixture for the whole downstream cycle."""
        return self._hot.freeze(f, T_burn, p_burn)

    # --- Rung-7 thermal-NOx diagnostic (DECOUPLED; never feeds the cycle) ---------
    def thermal_nox(self, far: float, T: float, p: float, tau: float = 3e-3) -> "NOxState":
        """Thermal-NO diagnostic on the equilibrium pool at (far, T, p) after residence
        time tau (default 3 ms, a typical gas-turbine primary-zone residence — an
        UN-ANCHORED knob, stated like the specified exit pressure). Solves the rung-6
        equilibrium composition (scale-A, datum-free mole numbers), superimposes
        equilibrium NO, and integrates the extended Zeldovich mechanism. Trace species
        => this does NOT affect the cycle; it is a pure diagnostic (docs/rung7-spec.md)."""
        comp = _equilibrium_composition(far, T, p)
        return _thermal_no(comp, T, p, tau, far)

    # --- Rung-8 two-zone combustor NOx diagnostic (DECOUPLED; never feeds the cycle) -----
    def zoned_nox(self, far: float, Tt3: float, Tt4: float, p: float,
                  phi_primary: float, tau: float = 3e-3) -> "ZonedNOxState":
        """Two-zone (primary → dilution) thermal NOx (docs/rung8-spec.md). Runs the SAME
        rung-7 extended-Zeldovich integrator on a HOT, near-stoichiometric PRIMARY zone
        instead of the mixed-out station-4 pool, then dilutes back to Tt4:

          1. air split — all the fuel + a fraction α of the air enter the primary at
             far_p = phi_primary·f_stoich; α = far/far_p (≤ 1);
          2. primary AFT — adiabatic flame temp from Tt3 (scale A, equilibrium products);
          3. primary NO — rung-7 `_thermal_no` on the primary equilibrium pool at (T_p, p);
          4. dilution — add the remaining air at Tt3, re-equilibrate the MAJORS (releases
             the dissociation energy → T_mix ≈ Tt4), and FREEZE the NO moles.

        NO is trace, so the cycle stays bit-for-bit rung 6 (NO/N never enter _equil_solve);
        only WHERE the chemistry is evaluated changes. The capped mixed-out Tt4 makes almost
        no NO; the hot primary — averaged away at station 4 — is where it forms.

        RUNG 9 — the primary may now run RICH (phi_primary up to 2.0): the equilibrium pool
        carries major CO/H2 (the rung-9 branched seed) and the extended-Zeldovich integrator
        runs on it unchanged. This closes the RQL (rich-burn → quick-quench → lean-burn) story:
        EI_NO forms a bell that PEAKS near stoichiometric (φ≈0.95, ~18 g/kg) and FALLS steeply
        on the rich flank (φ=1.4 → ~0.007 g/kg) — the AFT rolls over and the O-starved pool
        crashes [O]. That rich-side collapse is WHY real low-NOx combustors burn a rich primary.
        The mix-out here is the IDEAL (infinitely-fast) quench: NO is simply frozen at the
        primary value. Finite-rate quench (NO spiking as the gas dwells at stoich while mixing)
        is the next seam. Held below soot onset (phi_primary ≤ 2.0; docs/rung9-spec.md)."""
        assert 0.0 < phi_primary <= 2.0 + 1e-9, (
            f"phi_primary {phi_primary} outside (0, 2] — the 5-species (no soot / no C(s)) "
            "basis is valid only below soot onset (~φ2; graphite onset is φ3). Rich RQL scope."
        )
        hf_fuel = self.hf_fuel_molar or _HF_FUEL_DEFAULT
        far_p = phi_primary * _F_STOICH
        alpha = far / far_p                              # fraction of the air in the primary
        assert alpha <= 1.0 + 1e-9, \
            f"primary air fraction α={alpha:.4f} > 1 — overall mixture leaner than the primary"

        T_p = _primary_aft(far_p, p, Tt3, hf_fuel)
        comp_p = _equilibrium_composition(far_p, T_p, p)
        nox = _thermal_no(comp_p, T_p, p, tau, far_p)    # rung-7 integrator, UNCHANGED

        T_mix = _mixed_out_T(comp_p, T_p, alpha, far, Tt3, p)
        # Standing conservation gate (LOOSE gross-error bound — the method does not know η_b, and
        # a frozen-majors mix-out lands only ~40 K off here, WITHIN this band; the sharp
        # split-independence + frozen-vs-re-equilibrated discriminators live in the tests, which
        # ARE what catch that bug): the re-equilibrated mix-out must return to ≈ Tt4.
        assert abs(T_mix - Tt4) < 0.05 * Tt4, \
            f"mix-out T {T_mix:.1f} K did not return to Tt4={Tt4} K (re-equilibration gate)"

        # Freeze the NO moles through dilution: NO moles per mol PRIMARY air = x_no·ntot_p;
        # scale to the total-air basis by α; divide by the mixed pool's total moles per air.
        ntot_p = sum(comp_p.values())
        n_no_total = alpha * nox.x_no * ntot_p
        ntot_mix = sum(_equilibrium_composition(far, T_mix, p).values())
        x_no_mix = n_no_total / ntot_mix

        return ZonedNOxState(phi_primary=phi_primary, far_primary=far_p, alpha=alpha,
                             T_primary=T_p, T_mix=T_mix, primary=nox, x_no_mix=x_no_mix)

    def unified(self) -> "Gas":
        """Return a copy with the hot section collapsed onto the cold section.

        The lever for the reduce-to-ideal gate (docs/rung2-spec.md § Verification
        gates): collapsing the WHOLE section hot->cold — the (gamma, cp, R) triple
        AND any TPG cp(T) model — is what lets the machinery reproduce the rung-1
        table to the digit. If the hot section stayed different, its exponent would
        tilt the turbine and nozzle legs and the digits would drift.
        """
        return replace(self, gamma_t=self.gamma_c, cp_t=self.cp_c, R_t=self.R_c,
                       cp_t_coeffs=self.cp_c_coeffs, reacting_hot=False,
                       fork_b=False, equilibrium=False, hf_fuel_molar=None)
