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
from typing import Callable, Optional, Tuple


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

# --------------------------------------------------------------------------- #
# RUNG 19 — super-equilibrium O (lifting the equilibrium-O lower bound).       #
# Every NO number since rung 7 reads the rung-6 EQUILIBRIUM [O] into the       #
# Zeldovich rate, so it is a LOWER BOUND. Fluent (Theory Guide §9.1.3) offers  #
# a PARTIAL-EQUILIBRIUM O closure (Westenberg 1971, adds the 3-body            #
# O+O+M⇌O2+M) that sits ABOVE equilibrium O. Both share the same [O2]^0.5, so  #
# their RATIO is dimensionless and T-ONLY — no absolute-magnitude sourcing:    #
#   [O]_eq = C1·T^-0.5·[O2]^0.5·exp(-θ1/T),  [O]_pe = C2·T^+0.5·[O2]^0.5·exp(-θ2/T)  #
#   m(T) = [O]_pe/[O]_eq = (C2/C1)·T·exp((θ1-θ2)/T)  ∈ [1.16,1.50] over 1800-2400 K   #
# We lift OUR OWN rung-6 comp["O"] by m(T) inside the rung-7 integrator; m≡1   #
# ⇒ bit-for-bit rung 7. The lift is T-DRIVEN (φ-independent) — WEAKEST in the  #
# O2-depleted rich primary, so it does NOT match the naive "rich explosion"    #
# intuition. Constants TRANSCRIBED from the standard published forms (image-   #
# locked sources), NOT digit-verified; cross-validated by the equilibrium-O    #
# units gate (Westenberg [O]_eq / comp["O"] ∈ [0.94,0.99], test_rung19).       #
# See docs/rung19-spec.md and docs/plans/rung19-anchor-superequilibrium-prompt.md §B.  #
# --------------------------------------------------------------------------- #
_WESTENBERG_C1, _WESTENBERG_TH1 = 3.970e5, 31090.0   # equilibrium-O correlation (θ in K)
_WESTENBERG_C2, _WESTENBERG_TH2 = 36.64, 27123.0     # partial-equilibrium-O correlation
# RUNG 20 — flame-band floor for the super-eq O lift THROUGH the quench. m(T)=A·T·exp(B/T) with
# B=θ1-θ2≈3967 K DIVERGES as T→0 (m(1500 K)≈1.9, m(1200 K)≈3), so lifting [O] on a cooling quench
# path that reaches T_mix≈Tt4 would inject an out-of-band multiplier. The Westenberg partial-eq
# closure is a FLAME model (T≳1500 K) anyway, so we freeze m at m(max(T, floor)) below the band —
# which also keeps the standing 1≤m≤2 assert honest along the whole trajectory. (docs/rung20-spec.md)
_SUPER_EQ_T_FLOOR = 1500.0


def _super_eq_o_multiplier(T: float) -> float:
    """Super-equilibrium O multiplier m(T)=[O]_pe/[O]_eq=(C2/C1)·T·exp((θ1-θ2)/T) — the Westenberg
    partial-equilibrium O over equilibrium O (the shared [O2]^0.5 cancels ⇒ DIMENSIONLESS, T-ONLY).
    ∈ [1.16,1.50] over the flame band, DECREASING in T (→1 as T→∞: the partial-eq pool relaxes to
    equilibrium once the fast H-atom shuffle equilibrates). φ-INDEPENDENT: the lift is T-driven, NOT
    rich-driven — it is WEAKEST in the O2-starved rich primary, where thermal NO has already died.
    m≡1 recovers rung 7 exactly (super_eq_o=False). Rung 19; docs/rung19-spec.md."""
    return (_WESTENBERG_C2 / _WESTENBERG_C1) * T * math.exp(
        (_WESTENBERG_TH1 - _WESTENBERG_TH2) / T)


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
    ei_no: float         # emission index, g NO / kg fuel (thermal, m-lifted if super_eq_o)
    # RUNG 19 — the two lower-bound-lifting channels (both 1.0/0.0 for the rung-7 baseline).
    o_multiplier: float = 1.0    # super-eq O multiplier m(T) applied to [O] (1.0 ⇒ bit-for-bit rung 7)
    ei_no_prompt: float = 0.0    # ADDITIVE prompt (Fenimore) EI, g NO/kg fuel (0.0 ⇒ thermal only)

    @property
    def ei_no_total(self) -> float:
        """Total EI = thermal (m-lifted) + prompt, g NO/kg fuel. Rung 19: the equilibrium-O
        lower bound lifted two ways — a COMPUTED T-driven super-eq-O factor already folded into
        `ei_no` (via the lifted [O]), plus the IMPOSED additive prompt bump `ei_no_prompt`."""
        return self.ei_no + self.ei_no_prompt

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
                nsteps: int = 4000, o_multiplier: float = 1.0) -> NOxState:
    """Kinetic NO after residence time tau on the frozen pool `comp` at (T, p).

    One-equation extended-Zeldovich model (Heywood/Turns), QSS on N, REVERSE-RATE form
    for R2/R3 so equilibrium [N] is never needed (uses the pool's own O, H):
        d[NO]/dt = 2 R1 (1 - a^2)/(1 + a R1/(R2+R3)),  a = [NO]/[NO]_e
        R1 = k1f[O][N2],  R2 = k2r[NO]_e[O],  R3 = k3r[NO]_e[H]
    a=0 -> rate=2 R1 (initial rate); a->1 -> rate=0 (saturates at [NO]_e, so tau->inf
    recovers the equilibrium NO — an internal consistency gate). RK4 from [NO]=0.

    RUNG 19 — `o_multiplier` (default 1.0 ⇒ byte-identical rung 7) lifts the pool's [O] by the
    super-equilibrium factor m(T) (`_super_eq_o_multiplier`) BEFORE forming R1/R2. In the
    kinetically-limited (frozen) regime the NO stays far below the [NO]_e ceiling, so the rate ∝
    [O] and x_no scales ~linearly with m — a faster FORMATION, not a higher equilibrium (the
    ceiling [NO]_e is a thermodynamic quantity, independent of the O-atom closure, so the clamp
    still binds at the same value). super_eq_o thus lifts the equilibrium-O lower bound.
    """
    # K-CHECK (rung-7 standing assert, on every diagnostic run): the transcribed rate
    # constants must agree with the a6/a7 thermo at the evaluation T (the twin of rung 6's
    # atom-balance assert). A gross transcription slip is orders of magnitude off.
    kr = _kcheck_ratio(T)
    assert 0.90 < kr < 1.15, f"Zeldovich K-check off: ratio {kr:.4f} at T={T}"

    ntot = sum(comp.values())
    conc = p / (_Ru * T)                         # total molar concentration, mol/m^3
    x = {s: comp[s] / ntot for s in comp}
    cO = x.get("O", 0.0) * conc * o_multiplier   # rung 19: super-eq O lift (m=1.0 ⇒ rung 7)
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
                    char_time=(cNOe / (2.0 * R1) if R1 > 0.0 else math.inf), ei_no=ei,
                    o_multiplier=o_multiplier)


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

    def H_prod_at(T: float) -> float:                    # products' scale-A enthalpy, ↑ in T
        comp = _equilibrium_composition(far_p, T, p)
        return sum(comp[s] * _h_molar_A(s, T) for s in comp)

    lo, hi = 800.0, 3200.0
    for _ in range(100):
        T = 0.5 * (lo + hi)
        lo, hi = (lo, T) if H_prod_at(T) > H_react else (T, hi)
        if hi - lo < 1e-6:                               # ~31 iters to 1e-6 K; below any anchor
            break
    T = 0.5 * (lo + hi)
    # Bracket guard (post-loop, not an endpoint eval — `_equilibrium_composition` DIVERGES at the
    # cold 800 K edge, so we can't probe there; a root outside [800,3200] instead pins the bisection
    # against an edge, which this catches). Any real flame sits well inside.
    assert 801.0 < T < 3199.0, \
        f"_primary_aft: flame temp {T:.1f} K pinned at [800,3200] K bracket edge (far_p={far_p:.4f})"
    return T


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

    def H_prod_at(T: float) -> float:                    # re-equilibrated pool enthalpy, ↑ in T
        comp = _equilibrium_composition(far_ov, T, p)
        return sum(comp[s] * _h_molar_A(s, T) for s in comp)

    lo, hi = 700.0, 3200.0
    for _ in range(100):
        T = 0.5 * (lo + hi)
        lo, hi = (lo, T) if H_prod_at(T) > H_mix else (T, hi)
        if hi - lo < 1e-6:                               # ~31 iters to 1e-6 K; below any anchor
            break
    T = 0.5 * (lo + hi)
    # Bracket guard (post-loop; cold-edge eval diverges — see `_primary_aft`).
    assert 701.0 < T < 3199.0, \
        f"_mixed_out_T: mix temp {T:.1f} K pinned at [700,3200] K bracket edge (far_ov={far_ov:.4f})"
    return T


# --- Rung-10 finite-rate quench (secondary-zone Zeldovich in the cooling gas) --------- #
def _quench_trajectory(comp_prim: dict, T_prim: float, alpha: float, far_ov: float,
                       T_dilution: float, p: float, ngrid: int = 240) -> list:
    """Fast-chemistry dilution trajectory for the finite quench (rung 10).

    Rung 9's mix-out is the IDEAL (infinitely-fast) quench: NO frozen at the primary value.
    Here the quench air is added over a finite time, so we resolve the mix in a parameter
    β ∈ [0,1] (dilution fraction). The air present at β is a(β)=α+β(1−α) mol per mol
    TOTAL(final) air, so the LOCAL fuel/air ratio far_local = far_ov/a sweeps far_p → far_ov
    — through STOICHIOMETRIC for a rich primary (the peak of the NO bell). At each β the
    majors + T are instantaneous equilibrium (the rung-8 re-equilibrating `_mixed_out_T` on
    the CURRENT air, basis 1 mol current air), so [O],[N2],[H],[NO]_e and T are functions of
    β ALONE; NO is the one SLOW variable integrated separately (`_quench_no`). `V` is the
    pool volume on the FINAL basis (a·ntot_local mol → V), so extensive NO moles ↔ conc.

    The rung-7 K-check + trace guards bind along the WHOLE trajectory (every T the quench
    visits), not just the primary — the transcribed rates stay tied to the a6/a7 thermo."""
    tab = []
    for i in range(ngrid):
        b = i / (ngrid - 1)
        a = alpha + b * (1.0 - alpha)                 # mol current-air / mol total-final-air
        far_local = far_ov / a                        # far_p (β=0) → far_ov (β=1)
        alpha_local = alpha / a                        # fraction of CURRENT air that is primary
        T_local = _mixed_out_T(comp_prim, T_prim, alpha_local, far_local, T_dilution, p)
        comp_local = _equilibrium_composition(far_local, T_local, p)
        ntot_local = sum(comp_local.values())          # moles per mol current-air
        conc = p / (_Ru * T_local)                     # total molar conc, mol/m^3
        cO = comp_local.get("O", 0.0) / ntot_local * conc
        cN2 = comp_local["N2"] / ntot_local * conc
        cH = comp_local.get("H", 0.0) / ntot_local * conc
        x_no_e = _equilibrium_no_fraction(comp_local, T_local)
        cNOe = x_no_e * conc
        V = a * ntot_local * _Ru * T_local / p         # volume on the FINAL (total-air) basis
        kr = _kcheck_ratio(T_local)                    # K-check binds at EVERY trajectory T
        assert 0.90 < kr < 1.15, f"quench K-check off: ratio {kr:.4f} at T={T_local:.1f}"
        assert x_no_e < 0.02, f"NO not trace on quench path (x_NO_e={x_no_e:.4g}) at T={T_local:.1f}"
        tab.append(dict(a=a, T=T_local, cO=cO, cN2=cN2, cH=cH, cNOe=cNOe,
                        ntot_local=ntot_local, V=V))
    return tab


def _quench_no(comp_prim: dict, T_prim: float, alpha: float, far_ov: float,
               T_dilution: float, p: float, n_no_initial: float, tau_q: float,
               nsteps: int = 2000, ngrid: int = 240, tab: Optional[list] = None,
               schedule: Optional[Callable[[float], float]] = None,
               super_eq_o: bool = False) -> dict:
    """Finite-rate quench NO integrator (rung 10; schedule-aware for rung 11). CLAMP-FREE.

    Integrates the extended-Zeldovich rate (the SAME reverse-rate one-equation form as
    `_thermal_no`) along the `_quench_trajectory` cooling/mixing path, starting from the
    primary's kinetic NO (`n_no_initial`, the rung-9 frozen value). Two differences from
    `_thermal_no`, both load-bearing:

      * NO is EXTENSIVE — moles per mol total-final-air. Mixing dilution air changes the
        volume (V(β)) but conserves NO moles; only chemistry (the Zeldovich rate) changes
        them. So we integrate dn_NO/dt = rate([NO]=n_NO/V)·V.
      * The cNO≤cNOe CAP IS DROPPED. On a cooling path NO is legitimately
        super-equilibrium and frozen (Heywood); the cap would delete exactly that NO — a
        plausible-but-wrong low number with the asserts still green. The (1−a²) factor
        already goes NEGATIVE when a=[NO]/[NO]_e>1 (super-eq NO decomposes) and the
        Arrhenius constants freeze it out as T falls, so the form self-limits. This is a
        SEPARATE integrator; `_thermal_no` stays byte-identical (its rung-6..9 reduce gates
        depend on its exact capped RK4 trajectory). See docs/rung10-spec.md § the clamp trap.

    A slow quench dwells near the stoichiometric crossing (the NO-bell peak) and RE-MAKES
    the NO a rich primary avoided; a fast quench escapes past the peak — the RQL hazard.

    The trajectory (majors + T as a function of β alone) is τ_q-INDEPENDENT — the fast
    chemistry doesn't know how fast the mixing is — so a caller sweeping τ_q at fixed φ_p can
    build it ONCE and pass it as `tab` (the main.py panel / tests do this; a bare `zoned_nox`
    call rebuilds it).

    RUNG 11 — `schedule` decouples the DILUTION FRACTION β from time. Rung 10's β = t/τ_q
    (linear) doubled the time fraction as the β index; a physical jet-entrainment schedule
    remaps it, β = schedule(t/τ_q). We index the trajectory on β = schedule(tfrac) but still
    step `dt` in REAL time for the Zeldovich accumulation — conflating the two (rung 10 got
    away with it only because its schedule was the identity) would silently reproduce rung-10
    behaviour under a rung-11 label. `schedule=None` (default) is the identity → BYTE-IDENTICAL
    rung 10 (its rung-6..10 reduce gates depend on the exact capped trajectory). See
    docs/rung11-spec.md / `JetMixing`."""
    if tab is None:
        tab = _quench_trajectory(comp_prim, T_prim, alpha, far_ov, T_dilution, p, ngrid=ngrid)

    def interp(key: str, tfrac: float) -> float:
        x = tfrac * (len(tab) - 1)
        i = min(int(x), len(tab) - 2)
        w = x - i
        return tab[i][key] * (1.0 - w) + tab[i + 1][key] * w

    max_a = 0.0   # max [NO]/[NO]_e over the path: <1 ⇒ clamp dormant; >1 ⇒ super-eq regime

    def dn_dt(tfrac: float, n_no: float) -> float:
        nonlocal max_a
        cNOe = interp("cNOe", tfrac)
        if cNOe <= 0.0:
            return 0.0
        T = interp("T", tfrac); V = interp("V", tfrac)
        cO = interp("cO", tfrac); cN2 = interp("cN2", tfrac); cH = interp("cH", tfrac)
        if super_eq_o:                          # rung 20: lift [O] by m(T) INSIDE the re-making (the
            # deferred rung-19 seam — the finite-quench NO rode on equilibrium O). m(T) multiplies cO
            # so it scales R1 (formation) AND R2 (reverse) alike, exactly as _thermal_no's o_multiplier
            # does on the primary. Floor T at the flame band (m diverges as T→0 on the cool tail).
            m = _super_eq_o_multiplier(max(T, _SUPER_EQ_T_FLOOR))
            assert 1.0 <= m <= 2.0, (
                f"quench super-eq O multiplier m={m:.3f} at T={T:.0f} K outside [1,2] — the "
                "Westenberg partial-eq closure is a flame model (floored at T≳1500 K)"
            )
            cO *= m
        R1 = _k_zeldovich("1f", T) * cO * cN2
        R2 = _k_zeldovich("2r", T) * cNOe * cO
        R3 = _k_zeldovich("3r", T) * cNOe * cH
        beta = R1 / (R2 + R3) if (R2 + R3) > 0.0 else 0.0
        a = (n_no / V) / cNOe
        max_a = max(max_a, a)
        return 2.0 * R1 * (1.0 - a * a) / (1.0 + beta * a) * V     # d(n_NO)/dt = rate·V

    # β↔time map: identity (β = t/τ_q, rung-10 linear) unless a rung-11 mixing schedule
    # remaps it. When schedule is None the calls below are byte-identical to rung 10.
    sched = schedule if schedule is not None else (lambda x: x)
    n_no = n_no_initial
    dt = tau_q / nsteps
    t = 0.0
    for _ in range(nsteps):
        b1 = sched(min(t / tau_q, 1.0))
        b2 = sched(min((t + 0.5 * dt) / tau_q, 1.0))
        b3 = sched(min((t + dt) / tau_q, 1.0))
        k1 = dn_dt(b1, n_no)
        k2 = dn_dt(b2, n_no + 0.5 * dt * k1)
        k3 = dn_dt(b2, n_no + 0.5 * dt * k2)
        k4 = dn_dt(b3, n_no + dt * k3)
        n_no += dt / 6.0 * (k1 + 2 * k2 + 2 * k3 + k4)
        if n_no < 0.0:
            n_no = 0.0                                  # guard negatives ONLY (no eq cap)
        t += dt

    ntot_mix = tab[-1]["a"] * tab[-1]["ntot_local"]     # moles per mol total-final-air (=ntot(far_ov,T_mix))
    x_no_mix = n_no / ntot_mix
    n_fuel = far_ov * _M_AIR / _M_CH2
    ei = 1000.0 * (n_no * _M_NO) / (n_fuel * _M_CH2_KG) if n_fuel > 0.0 else 0.0
    T_peak = max(r["T"] for r in tab)
    return dict(ei=ei, x_no_mix=x_no_mix, n_no=n_no, T_peak=T_peak, max_a=max_a)


def _ideal_bell_ei(far_local: float, p: float, Tt3: float, hf_fuel: float, tau: float,
                   super_eq_o: bool = False) -> float:
    """Rung-9 IDEAL primary EI_NO (g NO/kg fuel) at a LOCAL fuel/air ratio — the bell EI(φ),
    sampled for the rung-13 PDF closure (docs/rung13-spec.md). Runs the same primitives as the
    zoned primary (`_primary_aft` → `_equilibrium_composition` → `_thermal_no`) at the local φ.

    Returns 0 outside the valid window: φ>2 (soot bound — the 5-species basis is invalid AND the
    O-starved pool makes ≈0 NO anyway) or too lean to burn (flame pinned at the cold bracket edge,
    `_primary_aft` raises). NO finite quench here — this ISOLATES composition variance on the ideal
    bell (carrying it through the finite quench is the rung-15 seam)."""
    phi = far_local / _F_STOICH
    if far_local <= 0.0 or phi > 2.0 + 1e-9:
        return 0.0
    try:
        T_p = _primary_aft(far_local, p, Tt3, hf_fuel)
    except AssertionError:
        return 0.0                      # too lean to burn (cold-bracket-edge flame)
    comp = _equilibrium_composition(far_local, T_p, p)
    # RUNG 20 — super_eq_o lifts this local bell's [O] by m(T_p) (default False ⇒ rung-13/15/18
    # ideal-bell integrals are UNTOUCHED, staying equilibrium-O lower bounds). Only the rung-16
    # per-pocket lean/tail branch passes True, keeping ei_no_pocket_quench internally consistent.
    m = _super_eq_o_multiplier(max(T_p, _SUPER_EQ_T_FLOOR)) if super_eq_o else 1.0
    return _thermal_no(comp, T_p, p, tau, far_local, o_multiplier=m).ei_no


def _beta_pdf_nodes_weights(xibar: float, g_seg: float, n_quad: int = 200):
    """Regime-aware, mean-preserving quadrature of a β-PDF of mixture fraction ξ (docs/rung13-spec.md
    §quadrature). Mean ξ̄, normalized variance (segregation) g∈(0,1): σ²=g·ξ̄·(1−ξ̄), shape params
    a=ξ̄·(1/g−1), b=(1−ξ̄)·(1/g−1). Returns (nodes ξ_i, normalized weights w_i).

    A LEAN mean gives a<1, so P_β∝ξ^(a−1) has an integrable SINGULARITY at ξ→0 that a naive uniform-
    in-ξ midpoint rule mis-weights (⟨ξ⟩ drifts off ξ̄, the integral never converges). REGIME-AWARE
    fix: for a<1 substitute u=ξ^a (uniform-in-u — the Jacobian cancels ξ^(a−1) EXACTLY, leaving the
    bounded weight (1−ξ)^(b−1), b≥1); for a≥1 (near-delta, no singularity) window a uniform-in-ξ grid
    over [0, ξ̄+8σ]. The mean AND variance are ASSERTED against their targets — that check is the
    deliverable more than the number is."""
    inv = 1.0 / g_seg - 1.0
    a, b = xibar * inv, (1.0 - xibar) * inv
    assert a > 0.0 and b >= 1.0, (
        f"β-PDF shape (a={a:.3f}, b={b:.3f}) outside a>0,b≥1 — the quadrature needs a non-singular "
        f"(1−ξ) tail (b≥1 holds for a lean mean until g≈0.49, well past g_max)."
    )
    if a < 1.0:                          # singular lean-mean regime — u=ξ^a cancels ξ^(a−1)
        u = [(i + 0.5) / n_quad for i in range(n_quad)]
        nodes = [uu ** (1.0 / a) for uu in u]
        logw = [(b - 1.0) * math.log(1.0 - x) for x in nodes]
    else:                                # near-delta (a≥1): bounded density, CENTER the window on the
        # mass (a ±8σ band around the exact mean ξ̄), not [0, …] — otherwise as g→0 the peak narrows
        # to a sliver near ξ̄ while the nodes stay spread over [0, ξ̄], mis-resolving it (⟨ξ⟩ drifts,
        # the assertion fires). Centering keeps ~constant nodes-per-σ at every g down to the delta floor.
        sigma = math.sqrt(g_seg * xibar * (1.0 - xibar))
        lo = max(1e-12, xibar - 8.0 * sigma)
        hi = min(1.0 - 1e-12, xibar + 8.0 * sigma)
        nodes = [lo + (hi - lo) * (i + 0.5) / n_quad for i in range(n_quad)]
        logw = [(a - 1.0) * math.log(x) + (b - 1.0) * math.log(1.0 - x) for x in nodes]
    m = max(logw)
    ww = [math.exp(l - m) for l in logw]
    s = sum(ww)
    w = [x / s for x in ww]

    # mean-preservation gate (THE deliverable): the closure must integrate at the specified mean.
    mean_xi = sum(wi * x for wi, x in zip(w, nodes))
    var_xi = sum(wi * (x - xibar) ** 2 for wi, x in zip(w, nodes))
    var_tgt = g_seg * xibar * (1.0 - xibar)
    assert abs(mean_xi - xibar) <= 0.01 * xibar, (
        f"β-PDF quadrature drifted the mean: ⟨ξ⟩={mean_xi:.6f} vs ξ̄={xibar:.6f} (>1%) — "
        f"the mean-preserving closure must integrate at ξ̄."
    )
    assert abs(var_xi - var_tgt) <= 0.05 * var_tgt, (
        f"β-PDF quadrature variance off target: {var_xi:.3e} vs {var_tgt:.3e} (>5%)."
    )
    return nodes, w


def _bell_interpolator(p: float, Tt3: float, hf_fuel: float, tau: float, n_bell: int = 200):
    """Build the IDEAL primary bell EI(ξ) ONCE on a fixed fine ξ-grid (ξ from ~0 up to the φ=2 soot
    bound) and return a smooth linear interpolator ξ↦EI. The bell is equilibrium-heavy, so a J-sweep
    reuses ONE bell (EI(ξ) is smooth) rather than re-solving per PDF node."""
    xi_max = (2.0 * _F_STOICH) / (1.0 + 2.0 * _F_STOICH)
    xi_ref = [xi_max * (i + 0.5) / n_bell for i in range(n_bell)]
    ei_ref = [_ideal_bell_ei(x / (1.0 - x), p, Tt3, hf_fuel, tau) for x in xi_ref]

    def bell(xi: float) -> float:
        if xi <= xi_ref[0]:
            return ei_ref[0]
        if xi >= xi_ref[-1]:
            return 0.0                  # beyond φ=2 (soot-rich) ⇒ EI≈0
        lo, hi = 0, n_bell - 1
        while hi - lo > 1:
            mid = (lo + hi) // 2
            if xi_ref[mid] <= xi:
                lo = mid
            else:
                hi = mid
        t = (xi - xi_ref[lo]) / (xi_ref[hi] - xi_ref[lo])
        return ei_ref[lo] + t * (ei_ref[hi] - ei_ref[lo])

    return bell


def _pdf_mean_ei(far_overall: float, Tt3: float, p: float, hf_fuel: float, tau: float,
                 g_seg: float, n_bell: int = 200, n_quad: int = 200) -> float:
    """⟨EI⟩ = ∫₀¹ EI_bell(φ(ξ))·P_β(ξ; ξ̄, g) dξ — the rung-13 resolved-mixing-PDF closure
    (docs/rung13-spec.md). A mean-preserving β-PDF of mixture fraction ξ=far/(1+far) at the OVERALL
    mean ξ̄=far_overall/(1+far_overall). g→0 ⇒ a delta at ξ̄ ⇒ the well-mixed point value. Builds the
    equilibrium-heavy bell once (`_bell_interpolator`) and integrates it against the regime-aware,
    mean-preserving quadrature (`_beta_pdf_nodes_weights`)."""
    xibar = far_overall / (1.0 + far_overall)
    if g_seg <= 1e-9:
        return _ideal_bell_ei(far_overall, p, Tt3, hf_fuel, tau)   # delta ⇒ well-mixed point value
    bell = _bell_interpolator(p, Tt3, hf_fuel, tau, n_bell=n_bell)
    nodes, w = _beta_pdf_nodes_weights(xibar, g_seg, n_quad=n_quad)
    return sum(wi * bell(x) for wi, x in zip(w, nodes))


def _pocket_quench_mean_ei(far_overall: float, Tt3: float, p: float, hf_fuel: float, tau_ref: float,
                           tau_core: float, g_seg: float, n_bell: int = 120, n_quad: int = 160,
                           quench_ngrid: int = 240, quench_nsteps: int = 2000,
                           super_eq_o: bool = False):
    """⟨EI_pocket_quench(ξ; τ_core)⟩ over a mean-preserving β-PDF — the rung-16 PER-POCKET
    PDF-through-quench closure (docs/rung16-spec.md). The rung-16 upgrade of `_pdf_mean_ei`:
    where rung 15 integrates the CONSTANT-T ideal bell and multiplies by a scalar dwell factor
    D(u)=τ_core/τ_ref (a LINEARISATION, exact only while EI ∝ τ), rung 16 carries EACH rich-of-mean
    pocket through its OWN finite quench — `_quench_no` on the pocket's cooling/mixing trajectory at
    the dwell τ_core — so the dwell enters INSIDE the chemistry. A pocket that lingers COOLS as it
    re-makes NO through the stoichiometric crossing, so ⟨EI⟩ is SUBLINEAR in τ_core (vs rung-15's
    linear D(u)·EI): the cooling-limited dwell erodes the far-over-penetration flank.

    Pocket bookkeeping (a mixture-fraction ξ ⇒ local far_local=ξ/(1−ξ), α=far_overall/far_local):
      • RICH of the overall mean (far_local > far_overall), burnable, φ≤2 → its own `_quench_no`
        (it dilutes DOWN through stoich toward the overall mean → the rung-10 re-making).
      • LEAN of the mean / φ>2 / too-lean-to-burn → `_ideal_bell_ei` (0 above φ2 — the soot-bound
        scope, IDENTICAL to rung-15's `_pdf_mean_ei`): a lean pocket only gets leaner on dilution,
        never re-crosses stoich, so it has NO finite quench. Keeping this branch bit-identical to
        rung 15 is what makes the reduce (pocket_quench→rung 15) and the φ>2 tail treatment exact.

    Values are built on a fixed ξ-grid over the burnable window [0, ξ(φ=2)] and interpolated against
    the regime-aware β-PDF quadrature (`_beta_pdf_nodes_weights`); the φ>2 tail is 0 (rung-15 scope).
    g→0 ⇒ a delta at ξ̄ ⇒ the single pocket-at-the-mean quench (≈0 at a lean mean ⇒ the finite bulk
    floor dominates). Returns (⟨EI⟩ g NO/kg fuel, max_a) — max_a folds into the clamp-dormancy gate."""
    xibar = far_overall / (1.0 + far_overall)
    xi_max = (2.0 * _F_STOICH) / (1.0 + 2.0 * _F_STOICH)          # ξ at the soot bound φ=2
    xi_grid = [xi_max * (i + 0.5) / n_bell for i in range(n_bell)]
    vals, max_a = [], 0.0
    for xi in xi_grid:
        far_local = xi / (1.0 - xi)
        if far_local < far_overall or far_local / _F_STOICH > 2.0 + 1e-9 or far_local <= 0.0:
            vals.append(_ideal_bell_ei(far_local, p, Tt3, hf_fuel, tau_ref,   # lean/tail: rung-15 bell
                                       super_eq_o=super_eq_o))                # (lifted with the pocket)
            continue
        try:
            T_p = _primary_aft(far_local, p, Tt3, hf_fuel)
        except AssertionError:
            vals.append(0.0)                                       # too lean to burn (cold-edge flame)
            continue
        alpha = far_overall / far_local                           # ≤ 1 (rich-of-mean pocket)
        comp = _equilibrium_composition(far_local, T_p, p)
        # RUNG 20 — lift the pocket's initial [O] (m at its OWN flame T_p) AND its quench re-making,
        # so every EI in the β-PDF integral carries the same closure (no half-eq-O hybrid).
        m0 = _super_eq_o_multiplier(max(T_p, _SUPER_EQ_T_FLOOR)) if super_eq_o else 1.0
        n0 = alpha * _thermal_no(comp, T_p, p, tau_ref, far_local, o_multiplier=m0).x_no * sum(comp.values())
        q = _quench_no(comp, T_p, alpha, far_overall, Tt3, p, n0, tau_core,
                       nsteps=quench_nsteps, ngrid=quench_ngrid, super_eq_o=super_eq_o)
        vals.append(q["ei"])
        max_a = max(max_a, q["max_a"])

    def qb(xi: float) -> float:
        if xi <= xi_grid[0]:
            return vals[0]
        if xi >= xi_grid[-1]:
            return 0.0                                            # φ>2 tail: rung-15 soot-bound scope
        lo, hi = 0, n_bell - 1
        while hi - lo > 1:
            mid = (lo + hi) // 2
            if xi_grid[mid] <= xi:
                lo = mid
            else:
                hi = mid
        t = (xi - xi_grid[lo]) / (xi_grid[hi] - xi_grid[lo])
        return vals[lo] + t * (vals[hi] - vals[lo])

    if g_seg <= 1e-9:
        return qb(xibar), max_a                                  # delta ⇒ single pocket-at-the-mean
    nodes, w = _beta_pdf_nodes_weights(xibar, g_seg, n_quad=n_quad)
    return sum(wi * qb(x) for wi, x in zip(w, nodes)), max_a


def _two_stream_ceiling(far_overall: float, phi_primary: float) -> float:
    """Rung-18 DERIVED injection ceiling g_ceiling = (ξ_p − ξ̄)/(1 − ξ̄) — the maximum normalized
    variance of a two-delta PDF on {0 (dilution air), ξ_p (rich-primary products)} at the fixed
    overall mean ξ̄ (docs/rung18-spec.md). Set by the PRIMARY RICHNESS φ_p, NOT a free knob — the one
    quantity rung 18 DERIVES rather than fits (it exposes rung-13's g_max=0.3 as ~4.4× too large).

    A two-delta at {0, ξ_p} with mean ξ̄ carries mass ξ̄/ξ_p at ξ_p; its variance is ξ̄(ξ_p−ξ̄), so the
    normalized segregation g=σ²/[ξ̄(1−ξ̄)]=(ξ_p−ξ̄)/(1−ξ̄). Requires φ_p>φ_overall (a rich primary
    diluting down to a leaner mean — the RQL geometry); asserts g_ceiling∈(0,1)."""
    xibar = far_overall / (1.0 + far_overall)
    far_p = phi_primary * _F_STOICH
    xi_p = far_p / (1.0 + far_p)
    g_ceiling = (xi_p - xibar) / (1.0 - xibar)
    assert 0.0 < g_ceiling < 1.0, (
        f"two-stream ceiling g={g_ceiling:.4f} outside (0,1): the primary (φ_p={phi_primary}, "
        f"ξ_p={xi_p:.4f}) must be RICHER than the overall mean (ξ̄={xibar:.4f}) — the RQL geometry."
    )
    return g_ceiling


def _transport_variance(g_ceiling: float, omega: float, tau: float,
                        c_phi: float = 2.0, nsteps: int = 400) -> float:
    """Rung-18 transported segregation: integrate the mixture-fraction variance DECAY ODE
    dg/dt = −C_φ·ω·g over the residence [0, τ] from the injection ceiling g_ceiling
    (docs/rung18-spec.md). ω is the turbulent mixing frequency, C_φ≈2 the canonical
    mechanical-to-scalar timescale ratio (scalar dissipation). Returns the residual width g(τ).

    Analytic for constant ω (g_ceiling·exp(−C_φ·ω·τ)), but INTEGRATED numerically so the
    negative-result gate (docs/rung18-spec.md §1) can drive it with ANY ω(J)/ω(C): a MEAN-FIELD
    ω(J) gives a monotone/flat g(J) (no interior optimum — the rung-11/12 variance seam); an
    interior optimum appears ONLY once ω is given a SPATIAL coverage dependence ω(C=(S/H)√J) — i.e.
    once the jet spacing S is injected. So this ODE cannot DERIVE the Holdeman optimum; the optimum
    LOCATION is imposed through the caller's coverage ω(C)."""
    dt = tau / nsteps
    g = g_ceiling
    denom = 1.0 + c_phi * omega * dt            # backward (implicit) Euler on the linear decay —
    for _ in range(nsteps):                     # unconditionally stable AND positivity-preserving for
        g /= denom                              # any dt (forward Euler would go negative at C_φ·ω·dt>1)
    assert 0.0 < g <= g_ceiling + 1e-12, (
        f"transported variance g={g:.4e} left (0, g_ceiling={g_ceiling:.4e}] — the decay ODE must "
        f"stay bounded by the injection ceiling (check C_φ·ω·τ ≥ 0 and dt small enough)."
    )
    return g


# --------------------------------------------------------------------------- #
# RUNG 14 — equilibrium-vs-frozen NOZZLE FLOW (the rung-6 cycle-side seam).     #
# The production nozzle FREEZES the station-4 equilibrium mixture through the   #
# whole expansion (rungs 6-13, `_EquilibriumSection`). Real nozzle flow lies    #
# between FROZEN (chemistry infinitely slow — composition fixed) and            #
# EQUILIBRIUM / SHIFTING (chemistry infinitely fast — composition = eq(T,p) at  #
# every point). As the exhaust expands and cools, the dissociated radicals      #
# CO/H₂/OH/O/H RECOMBINE to CO₂/H₂O, releasing chemical energy that partly       #
# converts to kinetic energy → a HIGHER V9 → more thrust. So equilibrium is an   #
# UPPER bound on nozzle performance and frozen a LOWER one; the real nozzle sits #
# between. This is a pure DIAGNOSTIC beside the cycle (like the rung-7..13 NOx   #
# layer): the production nozzle stays FROZEN, so the cycle is bit-for-bit rung 6.#
# See docs/rung14-spec.md and docs/plans/rung14-anchor-nozzle.md.               #
# --------------------------------------------------------------------------- #

_T_EXIT_FLOOR = 500.0   # nozzle-exit bisection floor, K. The equilibrium Newton diverges below
                        # ~500 K (radical mole numbers underflow the log-space floor); every real
                        # exit sits >700 K and bisection never probes below its lower bound, so
                        # this is a bracket GUARD (like `_primary_aft`), not an operating limit.


def _mix_entropy_molar(comp: dict, T: float, p: float) -> float:
    """Absolute mixture entropy per mol dry air, J/(mol·air K), at (T, p):

        S = Σ_i n_i [ s0_i(T) − Ru·ln(x_i · p/p0) ],   x_i = n_i/n_tot.

    s0_i is the absolute standard-state entropy `_s_molar` (the a7 constant, rung 6); the
    −Ru·ln(x_i·p/p0) term is the partial-pressure/mixing correction (p0 = _P_REF = 1 bar). For a
    FIXED composition the Σ n_i·(−Ru·ln x_i) mixing part is invariant, so an isentropic drop
    collapses to the pr-ratio relation (== the frozen production nozzle); for a SHIFTING mixture
    the mole fractions change and the mixing term is live (docs/rung14-spec.md § the entropy)."""
    ntot = sum(comp.values())
    S = 0.0
    for sp, n in comp.items():
        if n <= 0.0:
            continue
        S += n * (_s_molar(sp, T) - _Ru * math.log((n / ntot) * p / _P_REF))
    return S


def _mix_mass_per_air(comp: dict) -> float:
    """Mixture mass per mol dry air, kg. INVARIANT under recombination (atoms conserved), so the
    frozen and equilibrium exits share it — the two V9 = √(2 ΔH/m) denominators are identical."""
    return sum(n * _SPECIES[sp][0] / 1000.0 for sp, n in comp.items())


def _mix_h_abs_B(comp: dict, T: float) -> float:
    """Absolute (scale-B: 0K-sensible + formation) mixture enthalpy per mol air, J/mol·air."""
    return sum(n * _h_molar_B(sp, T) for sp, n in comp.items())


def _expand_nozzle(comp_entry: dict, far: float, Tt9: float, pt9: float, p9: float,
                   shifting: bool) -> Tuple[float, float, dict]:
    """One reversible-adiabatic (isentropic) nozzle expansion of the exhaust from the entry TOTAL
    state (Tt9, pt9) to the back-pressure p9. Returns (T9, V9, comp9). Rung 14.

    COMMON PHYSICAL ENTRY (the fair-bracket choice, advisor-confirmed): BOTH modes start from the
    SAME gas — the frozen station-4 mixture `comp_entry` at (Tt9, pt9) — so the stagnation enthalpy
    H_entry and entropy S_entry are IDENTICAL and the recombination benefit shows up as V9, not as
    a shifted entry state. We do NOT re-equilibrate the entry: that would lower H_entry (eq is more
    recombined) and bracket two DIFFERENT gases. The sliver of entry irreversibility from switching
    chemistry on at the throat is the stated 'infinite-rate-at-the-nozzle' approximation (spec §).

      FROZEN      (shifting=False): composition held = comp_entry → reduces to the production
                  nozzle's pr-ratio expansion EXACTLY (the fixed-comp mixing term cancels).
      EQUILIBRIUM (shifting=True) : composition = _equilibrium_composition(far, T, p9) at each T.

    Isentropic: solve T9 with S_mix(comp(T9), T9, p9) = S_mix(comp_entry, Tt9, pt9). Then
    V9 = √(2·(H_entry − H_abs(comp9, T9))/m) on ABSOLUTE (scale-B) enthalpy so the recombination
    (formation) energy appears, with m the recombination-invariant mass per mol air."""
    S_entry = _mix_entropy_molar(comp_entry, Tt9, pt9)
    H_entry = _mix_h_abs_B(comp_entry, Tt9)
    m = _mix_mass_per_air(comp_entry)

    def comp_at(T: float) -> dict:
        return _equilibrium_composition(far, T, p9) if shifting else comp_entry

    # S(T) rises with T at fixed p (higher s0, and the shift toward dissociation only adds moles),
    # so bisect on the entropy residual: at p9 < pt9 the gas is over-entropic at Tt9, cool it to
    # bring S back to S_entry (⇒ T9 < Tt9 — the expansion trades enthalpy for kinetic energy).
    lo, hi = _T_EXIT_FLOOR, Tt9
    for _ in range(200):
        T = 0.5 * (lo + hi)
        if _mix_entropy_molar(comp_at(T), T, p9) > S_entry:
            hi = T
        else:
            lo = T
        if hi - lo <= 1e-13 * T:     # near-machine, so frozen reproduces the production nozzle exactly
            break
    T9 = 0.5 * (lo + hi)
    # Bracket guard (post-loop, like `_primary_aft`): a root pinned at the cold floor means the
    # expansion wanted a colder exit than the equilibrium solve can resolve. Never happens here
    # (every exit sits >700 K), but guard it rather than trust a bracket edge.
    assert T9 > _T_EXIT_FLOOR + 1.0, \
        f"nozzle exit T={T9:.1f} K pinned at the {_T_EXIT_FLOOR:.0f} K bracket floor (far={far:.4f})"
    comp9 = comp_at(T9)
    V9 = math.sqrt(2.0 * (H_entry - _mix_h_abs_B(comp9, T9)) / m)
    return T9, V9, comp9


def _nozzle_clamp_diag(comp_entry: dict, Tt9: float, T9: float,
                       x_no_frozen: Optional[float]) -> dict:
    """The dropped-clamp corollary (rung 14; docs/rung14-spec.md § where the clamp earns its keep).

    On the nozzle cooling path the COMPOSITION is frozen, but the equilibrium NO mole fraction
    x_NO_e = Kp_NO(T)·√(x_N2·x_O2) COLLAPSES as T falls (Kp_NO = exp(−ΔG°/RuT) is steeply
    T-dependent). So exhaust NO frozen at the combustor value becomes SUPER-EQUILIBRIUM (a =
    [NO]/[NO]_e > 1): rung 7's cNO≤cNOe clamp would DELETE it — a plausible-but-wrong low number
    with every assert still green. Rung 10 DROPPED that clamp (super-eq NO freezes on cooling —
    Heywood) and proved it DORMANT on the combustor quench (max_a=0.677<1). HERE it fires (a≫1) —
    this is exactly the near-stoich exhaust-cooling regime rung 10 flagged. x_NO_e is monotone in
    T (frozen x_N2/x_O2), so the coldest point T9 gives the trajectory-max a. The COLLAPSE RATIO is
    frozen-NO-independent (the certifiable core); max_a needs a frozen exhaust NO — the caller
    passes the physically-realistic rung-8 zoned (ICAO-band) value."""
    xe_entry = _equilibrium_no_fraction(comp_entry, Tt9)
    xe_exit = _equilibrium_no_fraction(comp_entry, T9)     # coldest → smallest x_NO_e → max a
    return dict(x_no_e_entry=xe_entry, x_no_e_exit=xe_exit,
                no_collapse_ratio=xe_entry / xe_exit,
                max_a=(x_no_frozen / xe_exit) if x_no_frozen is not None else None)


@dataclass
class JetMixing:
    """Rung-11 jet-in-crossflow mixing config — the PHYSICAL dilution-air entrainment model
    that retires rung 10's free τ_q + linear-schedule knobs (docs/rung11-spec.md). A MEAN-FIELD
    model: a single well-mixed core diluting on a mean β(t); it derives the quench RATE from jet
    momentum but CANNOT produce a mixing OPTIMUM (that is a spatial-variance effect — an
    over-penetrating jet leaves an un-mixed hot near-stoich core — deferred to rung 12). So the
    J-sweep is MONOTONE by construction, not by accident.

    The one design knob is the momentum-flux ratio J = ρ_j U_j²/(ρ_c U_c²); H/U_c/C_e/shape_n are
    order-of-magnitude / un-anchored (like α, φ_p, τ), so the ABSOLUTE τ_q is un-pinned — what is
    certified is the √J SCALING and the monotone direction (docs/plans/rung11-anchor-mixing.md)."""
    J: float                # jet-to-crossflow momentum-flux ratio (THE design knob)
    H: float = 0.10         # dilution-zone duct height, m (the cross-stream mixing length)
    U_c: float = 75.0       # bulk crossflow velocity, m/s
    C_e: float = 0.15       # entrainment constant, O(0.1); folds penetration + entrainment + ρ ratio
    shape_n: float = 2.0    # entrainment schedule exponent (1 = linear/rung-10; >1 decelerating)

    def __post_init__(self):
        for name, v in (("J", self.J), ("H", self.H), ("U_c", self.U_c),
                        ("C_e", self.C_e), ("shape_n", self.shape_n)):
            assert v > 0.0, f"JetMixing.{name}={v} must be positive"

    @property
    def tau_q(self) -> float:
        """Derived quench time τ_q = H/(C_e·√J·U_c) — monotone-DECREASING in J (a higher-momentum
        jet penetrates and entrains faster; penetration ∝ √J). 'Quick quench' = high jet momentum.
        For physical J∈[4,100] this lands in the RQL sub-ms–few-ms quench band."""
        return self.H / (self.C_e * math.sqrt(self.J) * self.U_c)

    def schedule(self, tfrac: float) -> float:
        """The entrainment schedule β(t/τ_q) = 1 − (1 − t/τ_q)^shape_n — reaches β=1 EXACTLY at
        tfrac=1 (no endpoint trap). shape_n=1 ⇒ β=tfrac (linear = constant entrainment = rung 10,
        the reduce); shape_n>1 ⇒ concave/decelerating (fast near the jet where shear + gradient are
        strong, slowing as the concentration difference collapses).

        shape_n==1 returns the IDENTITY exactly (not 1−(1−x)^1, which drifts a ULP) so a
        JetMixing(shape_n=1) is BYTE-IDENTICAL to the rung-10 linear path at the derived τ_q."""
        if self.shape_n == 1.0:
            return tfrac
        return 1.0 - (1.0 - tfrac) ** self.shape_n


@dataclass
class Unmixedness:
    """Rung-12 spatial-unmixedness (two-stream) model — the VARIANCE layer rung 11 deferred.
    It rides ON a `JetMixing` (which supplies J and the duct height H) and finally makes the
    NO-vs-J curve TURN BACK UP, recovering the classic Holdeman dilution-jet optimum AT C_opt.

    Rung 11 was MEAN-FIELD: one well-mixed core diluting on a mean β(t), so its J-sweep is
    monotone (a stronger jet only ever re-makes LESS NO). But a real dilution jet has an
    OPTIMUM at the Holdeman group C=(S/H)√J ≈ C_opt≈2.5 (a *uniformity* criterion): UNDER-
    penetration (low C) leaves the jet near the wall and the core un-mixed; OVER-penetration
    (high C) slams the air onto the far wall / collides jets in the centre — BOTH leave a hot,
    near-stoichiometric core that misses the fast jet mixing and lingers. That un-mixed core is
    a spatial-VARIANCE effect a mean field cannot represent (docs/rung11-spec.md § the ceiling).

    THE MODEL (a two-stream split, mass-weighted; the CORE carries the off-optimum penalty, TWO WAYS):
        * a BULK fraction (1−w) quenched at the rung-11 jet time τ_mean(J)=H/(C_e√J·U_c) — the
          mean-field flow (∝1/√J: what a mean field says, monotone-falling — the fixed REFERENCE,
          NOT a function of C_opt);
        * an UNDER-MIXED CORE fraction w that MISSES the jet and lingers, quenched at a dwell
          τ_core(C)=τ_res·(1+b_u·u) — an ABSOLUTE residence (~τ_res, NOT the vanishing jet time,
          so its NO penalty survives J→∞) that GROWS off-optimum (a worse-placed jet strands the
          pocket longer). So off-optimum the core worsens TWO ways: more gas segregates (w↑) AND it
          lingers longer (τ_core↑);
        * the unmixedness u(C)=|ln(C/C_opt)| drives both — the segregated fraction
          w(C)=min(w_max, k_u·u) and the core dwell — and is KINKED at C_opt (an L1 distance, →0
          at C_opt on BOTH flanks; under- and over-penetration penalised alike).

        EI_total = (1−w)·EI(τ_mean) + w·EI(τ_core)

    THE PIN (why the min lands AT C_opt, not above it). With a SMOOTH (parabolic) w the turn-up
    would drift to a stronger jet than C_opt — the still-falling mean-field bulk pulls it right.
    The KINK gives w a non-zero slope at C_opt, so the turn-up starts THERE the moment the
    unmixedness beats the penetration benefit: k_u·[EI(τ_core)−EI(τ_mean)] > EI(τ_mean) at C_opt.
    That inequality is exactly the condition an emissions optimum EXISTS at the uniformity optimum
    (if penetration always won, there would be no optimum). So the EI-min pins at C_opt for ALL S
    ⇒ J_min = J_opt = (C_opt·H/S)², which shifts EXACTLY as (H/S)² — the Holdeman group made
    literal (docs/plans/rung12-anchor-unmixedness.md §1).

    THE PAYOFF (docs/rung12-spec.md gates): EI_NO(J) FALLS then RISES — an interior minimum AT
    C_opt, the recovered Holdeman optimum — and its location shifts as (H/S)² with the spacing.
    `unmixedness=None` (default) is the exact rung-11 mean-field path; k_u=0 is bit-for-bit rung 11
    at every J. Like C_e/τ_q, the ABSOLUTE knobs (S, τ_res, k_u, b_u, w_max) are order-of-magnitude
    / un-anchored — what is certified is the TURN-UP, the optimum AT C_opt, and the (H/S)² shift."""
    S: float = 0.0625          # dilution-jet spacing, m (cross-stream spacing of adjacent jets)
    C_opt: float = 2.5         # Holdeman uniformity optimum of C=(S/H)√J (best cross-plane mixing)
    tau_res: float = 2.5e-3    # core dwell AT the optimum, s (the absolute dilution-zone residence)
    k_u: float = 2.5           # core-fraction sensitivity to unmixedness (the kink that PINS the min at C_opt)
    b_u: float = 1.0           # core-dwell growth off-optimum (keeps the over-penetration flank rising)
    w_max: float = 0.7         # cap on the segregated core fraction (0<w_max≤1)

    def __post_init__(self):
        for name, v in (("S", self.S), ("C_opt", self.C_opt), ("tau_res", self.tau_res),
                        ("w_max", self.w_max)):
            assert v > 0.0, f"Unmixedness.{name}={v} must be positive"
        assert self.k_u >= 0.0, f"Unmixedness.k_u={self.k_u} must be ≥ 0 (0 ⇒ reduce to rung 11)"
        assert self.b_u >= 0.0, f"Unmixedness.b_u={self.b_u} must be ≥ 0"
        assert self.w_max <= 1.0, f"Unmixedness.w_max={self.w_max} must be ≤ 1 (a mass fraction)"

    def C(self, mixing: "JetMixing") -> float:
        """The Holdeman momentum-flux/geometry group C=(S/H)√J — jet penetration ∝ √J scaled by
        the spacing/height ratio. Uses the paired JetMixing's H and J (the same jet that set τ_q)."""
        return (self.S / mixing.H) * math.sqrt(mixing.J)

    def _u(self, C: float) -> float:
        """The unmixedness u(C)=|ln(C/C_opt)| — an L1 (KINKED) distance from the Holdeman optimum,
        0 at C_opt, symmetric in ln C. The kink (non-zero slope at C_opt) is what pins the EI-min
        AT C_opt rather than at a stronger jet (a smooth parabola would let it drift; class § THE PIN)."""
        return abs(math.log(C / self.C_opt))

    def core_fraction(self, C: float) -> float:
        """The un-mixed (segregated) core mass fraction w(C)=min(w_max, k_u·u) — the UNMIXEDNESS.
        Zero at C_opt (perfect tiling), rising on BOTH flanks (kinked → non-zero slope at C_opt,
        so the turn-up starts at C_opt). Capped at w_max."""
        return min(self.w_max, self.k_u * self._u(C))

    def core_dwell(self, C: float) -> float:
        """The under-mixed core's quench dwell τ_core(C)=τ_res·(1+b_u·u) — an ABSOLUTE residence
        (does NOT ride the vanishing jet time τ_mean∝1/√J, so its NO penalty survives J→∞) that
        GROWS off-optimum (a worse-placed jet strands the pocket longer). =τ_res at C_opt."""
        return self.tau_res * (1.0 + self.b_u * self._u(C))


@dataclass
class MixingPDF:
    """Rung-13 resolved-mixing-PDF config — a mean-preserving β-PDF of mixture fraction that
    replaces rung-12's parameterised SEGREGATION (w(C)) with a CONTINUOUS distribution. Rides ON a
    `JetMixing` (needs J and the duct height H for the Holdeman group C=(S/H)√J) and is MUTUALLY
    EXCLUSIVE with `unmixedness` (two closures of the SAME segregation physics).

    A MECHANISM SEPARATION (the sharp result — NOT a rung-12 "turn-up" reproduction). Rung-12's
    over-penetration CLIMB came from the DWELL effect (an absolute, off-optimum-growing τ_core
    surviving J→∞ — a TIME mechanism). This rung isolates the COMPOSITION mechanism and DROPS the
    quench chain, so it structurally CANNOT climb: it pins the optimum LOCATION (min AT C_opt) but
    the far-over-penetration flank DESCENDS (the humped ⟨EI⟩(g), below). Composition variance pins
    the optimum; the dwell effect makes the climb; combining them (the PDF through the quench) is
    rung 15. This replaces the segregation parameterisation, NOT the dwell — complementary, not a
    superset.

    THE LESSON (framed correctly — NOT generic "convexity/Jensen"). The NO-vs-φ bell is convex on
    its flanks but CONCAVE at the peak, so there is no global convexity to invoke. The honest
    statement: NO production is sharply PEAKED at stoich, so spreading the local φ around a fixed
    mean (unmixedness) RAISES the mean NO whenever the mean is OFF-stoich — the stoich-ward tail
    samples the peak while the mean itself sits in a low-EI wing — most strongly the leaner the
    mean, and it REVERSES sign at a stoich mean (spreading then moves mass OFF the peak, lowering
    NO). Our combustor mean is LEAN (the dilution zone conserves mass ⇒ the mean mixture fraction
    is the OVERALL value, and the Holdeman group is a dilution-jet quantity), so segregation raises
    NO and the optimum is where segregation is least (C_opt).

    THE MODEL: ⟨EI⟩ = ∫ EI_bell(φ(ξ))·P_β(ξ; ξ̄, g) dξ — ξ̄ the overall mixture fraction, the width
    the SEGREGATION g(C)=min(g_max, k_g·|ln(C/C_opt)|), the SAME kinked L1 distance from C_opt that
    drove rung-12's w(C). g=0 at C_opt (⇒ a delta ⇒ the well-mixed point value); rising (kinked) on
    both flanks; capped at g_max. So ⟨EI⟩(g(C)) collapses to the well-mixed value AT C_opt (a sharp
    notch) with BOTH immediate flanks lifting by orders — the Holdeman optimum LOCATION from a
    CONTINUOUS PDF, J_min=J_opt shifting as (H/S)². (⟨EI⟩(g) is HUMPED — peak at low g, descending as
    the β-PDF goes bimodal — so the far-over-penetration flank descends, per the separation above.)

    HONEST SCOPE: this ISOLATES composition variance on the IDEAL bell — the finite-quench dwell
    chain (rungs 10–12) is dropped, so the optimum minimum is the well-mixed LEAN value ≈0 (a
    modeling boundary, NOT a physics claim; bulk / primary-history NO is out of scope). Carrying the
    PDF THROUGH the rung-12 quench is the rung-15 seam. `pdf=None` (default) is the exact rung-12
    path; g→0 is the point value. Like the rung-9..12 knobs, S/k_g/g_max are order-of-magnitude;
    C_opt≈2.5 is Holdeman's value — what is CERTIFIED is the optimum pinned AT C_opt (both flanks
    up), the (H/S)² shift, and the SIGN of the effect (and its reversal at a stoich mean)."""
    S: float = 0.0625          # dilution-jet spacing, m (forms the Holdeman group with H and J)
    C_opt: float = 2.5         # Holdeman uniformity optimum of C=(S/H)√J (the segregation minimum)
    k_g: float = 0.3           # segregation sensitivity to the kinked distance |ln(C/C_opt)|
    g_max: float = 0.3         # cap on the segregation (0<g_max<1; past the ⟨EI⟩(g) hump ⇒ far flank descends)
    n_bell: int = 200          # fixed reference-bell grid points (EI(ξ) built ONCE, interpolated)
    n_quad: int = 200          # β-PDF quadrature nodes

    def __post_init__(self):
        for name, v in (("S", self.S), ("C_opt", self.C_opt)):
            assert v > 0.0, f"MixingPDF.{name}={v} must be positive"
        assert self.k_g >= 0.0, f"MixingPDF.k_g={self.k_g} must be ≥ 0 (0 ⇒ g≡0 ⇒ well-mixed point)"
        assert 0.0 < self.g_max < 1.0, f"MixingPDF.g_max={self.g_max} must be in (0,1)"
        assert self.n_bell > 1 and self.n_quad > 1, "MixingPDF grid sizes (n_bell, n_quad) must be > 1"

    def C(self, mixing: "JetMixing") -> float:
        """The Holdeman momentum-flux/geometry group C=(S/H)√J (identical to Unmixedness.C — the
        same jet that set τ_q). Uses the paired JetMixing's H and J."""
        return (self.S / mixing.H) * math.sqrt(mixing.J)

    def segregation(self, C: float) -> float:
        """The β-PDF segregation width g(C)=min(g_max, k_g·|ln(C/C_opt)|) — KINKED at C_opt (0 there,
        rising on BOTH flanks with a non-zero slope, so the emissions minimum pins AT C_opt), capped
        at g_max. g→0 ⇒ a delta at ξ̄ ⇒ the well-mixed point value EI(φ_overall)."""
        return min(self.g_max, self.k_g * abs(math.log(C / self.C_opt)))


@dataclass
class QuenchPDF:
    """Rung-15 PDF-THROUGH-QUENCH config — carries rung-13's resolved mixture-fraction β-PDF THROUGH the
    rung-10/12 finite-quench dwell chain, so the two mixing mechanisms finally COMBINE: the COMPOSITION
    variance (rung 13, which pinned the optimum LOCATION but on the ideal bell dropped the quench → its
    optimum collapsed to ≈0) AND the DWELL (rung 12, an absolute off-optimum-growing residence → the
    over-penetration flank CLIMBS). Rides ON a `JetMixing` (needs J and H for the Holdeman group
    C=(S/H)√J AND the derived τ_mean for the floor), MUTUALLY EXCLUSIVE with both `pdf` and
    `unmixedness` (three closures of the SAME variance physics).

    THE CONSTRUCTION (additive — mean-field FLOOR + resolved composition FLUCTUATION × dwell):
        <EI>_15 = EI_bulk_quench(τ_mean)          # term 1: rung-11 mean field (the FINITE floor, all C)
                + D(u) · <EI_bell>(g)             # term 2: rung-13 β-PDF integral × a rung-12 dwell
    with g(C)=min(g_max, k_g·|ln(C/C_opt)|) (rung-13 segregation), u(C)=|ln(C/C_opt)| (rung-12
    unmixedness), and the dwell factor D(u)=τ_res·(1+b_u·u)/τ_ref — τ_core(u)/τ_ref, an ABSOLUTE
    off-optimum-growing residence (rung-12 core_dwell) rescaling the reference-τ bell EI to the pocket's
    actual lingering dwell (exact while EI ∝ τ, the dormant clamp). τ_ref is the zoned_nox residence
    `tau` at which the bell is built (so the two stay locked).

    DISTINGUISHABLE FROM BOTH PARENTS: the finite floor + the climbing far flank are NOT rung 13 (whose
    optimum is ≈0 and whose far flank descends as the β-PDF goes bimodal); the STOICH-MEAN SIGN REVERSAL
    (term 2 samples the NONLINEAR, peaked bell) is NOT reproducible by rung-12's lumped dwell — the
    discriminator that catches the naïve "dwell-only PDF through the quench" trap (which, because
    EI_quench(τ) is LINEAR, collapses to rung-12's mean and rides the wrong sign; docs/rung15-spec.md).
    `pdf_quench=None` is the exact rung-13 path; at C_opt (g→0) <EI> = the finite bulk quench NO
    (ei_no_quenched), NOT ≈0.

    Like the rung-9..13 knobs, S/k_g/g_max/τ_res/b_u are order-of-magnitude; C_opt≈2.5 is Holdeman's
    value; b_u=3 (larger than rung-12's default) pins the min AT C_opt because term 2's <EI_bell> is a
    weaker lever than rung-12's EI(τ_core) (docs/plans/rung15-anchor-pdf-quench.md §2c). Certified: the
    finite floor, the min AT C_opt (both flanks up, far flank CLIMBING), the (H/S)² shift, the sign
    reversal."""
    S: float = 0.0625          # dilution-jet spacing, m (forms the Holdeman group with H and J)
    C_opt: float = 2.5         # Holdeman uniformity optimum of C=(S/H)√J (segregation + dwell minimum)
    k_g: float = 0.3           # β-PDF segregation sensitivity to |ln(C/C_opt)| (rung-13 width)
    g_max: float = 0.3         # cap on the segregation (0<g_max<1; rung-13 bimodal bound)
    tau_res: float = 2.5e-3    # under-mixed-pocket dwell AT the optimum, s (absolute residence; rung 12)
    b_u: float = 3.0           # off-optimum dwell growth (pins the min at C_opt; > rung-12's default)
    n_bell: int = 200          # reference-bell grid points (EI(ξ) built ONCE, interpolated; rung 13)
    n_quad: int = 200          # β-PDF quadrature nodes (rung 13)

    def __post_init__(self):
        for name, v in (("S", self.S), ("C_opt", self.C_opt), ("tau_res", self.tau_res)):
            assert v > 0.0, f"QuenchPDF.{name}={v} must be positive"
        assert self.k_g >= 0.0, f"QuenchPDF.k_g={self.k_g} must be ≥ 0 (0 ⇒ g≡0 ⇒ floor only)"
        assert self.b_u >= 0.0, f"QuenchPDF.b_u={self.b_u} must be ≥ 0"
        assert 0.0 < self.g_max < 1.0, f"QuenchPDF.g_max={self.g_max} must be in (0,1)"
        assert self.n_bell > 1 and self.n_quad > 1, "QuenchPDF grid sizes (n_bell, n_quad) must be > 1"

    def C(self, mixing: "JetMixing") -> float:
        """The Holdeman momentum-flux/geometry group C=(S/H)√J (identical to MixingPDF/Unmixedness.C —
        the same jet that set τ_q). Uses the paired JetMixing's H and J."""
        return (self.S / mixing.H) * math.sqrt(mixing.J)

    def _u(self, C: float) -> float:
        """The unmixedness u(C)=|ln(C/C_opt)| — the KINKED L1 distance from the Holdeman optimum (0 at
        C_opt), driving BOTH the β-PDF width g and the dwell growth (rung-12 + rung-13 kinks, unified)."""
        return abs(math.log(C / self.C_opt))

    def segregation(self, C: float) -> float:
        """The rung-13 β-PDF segregation width g(C)=min(g_max, k_g·u) — KINKED at C_opt (0 there),
        capped at g_max. Sets term 2's composition variance (the nonlinear-bell integral)."""
        return min(self.g_max, self.k_g * self._u(C))

    def dwell_factor(self, C: float, tau_ref: float) -> float:
        """The dwell factor D(u)=τ_res·(1+b_u·u)/τ_ref — the segregated pocket's ABSOLUTE residence
        τ_core(u)=τ_res·(1+b_u·u) (rung-12 core_dwell, GROWS off-optimum, survives J→∞) relative to the
        bell's reference residence τ_ref. Rescales the reference-τ bell EI to the pocket's lingering
        dwell (EI ∝ τ, dormant clamp). Its off-optimum growth is what makes the far flank CLIMB."""
        return self.tau_res * (1.0 + self.b_u * self._u(C)) / tau_ref


@dataclass
class PocketQuenchPDF:
    """Rung-16 PER-POCKET PDF-through-quench config — RETIRES rung-15's one acknowledged
    linearisation. Rung 15 (`QuenchPDF`) carried the composition β-PDF through the dwell as
    term 2 = D(u)·⟨EI_bell⟩(g): the CONSTANT-T ideal bell × a SCALAR dwell factor D(u)=τ_core/τ_ref
    — exact only while EI ∝ τ (the dormant clamp), which ignores that a lingering pocket COOLS.
    Rung 16 carries EACH rich-of-mean pocket through its OWN finite quench (`_quench_no` at the
    dwell τ_core), so the dwell enters INSIDE the chemistry. Same knobs, same rides-on-`JetMixing`,
    same Holdeman group C=(S/H)√J; MUTUALLY EXCLUSIVE with `pdf`, `pdf_quench` AND `unmixedness`
    (four closures of the SAME variance physics — a ≤1-of-four guard).

    THE CONSTRUCTION (additive, mirroring rung 15 — only term 2's INTERNALS change):
        ⟨EI⟩_16 = EI_bulk_quench(τ_mean)                      # term 1: rung-11 mean field (finite floor)
                + ⟨EI_pocket_quench(ξ; τ_core(C))⟩_g          # term 2: PER-POCKET quench β-PDF integral
    with g(C)=min(g_max, k_g·|ln(C/C_opt)|) (rung-13 segregation), τ_core(C)=τ_res·(1+b_u·|ln(C/C_opt)|)
    (rung-12 absolute dwell, now applied INSIDE each pocket's quench, not as a ratio).

    THE ROBUST LESSON (docs/rung16-spec.md; certified in tests/test_rung16.py):
      • SUBLINEAR DWELL (the mechanism): because each pocket cools through its quench, term 2 grows
        SUBLINEARLY in τ_core (far-flank term2 ratio ≈ ×1.30 across J=144→625 vs rung-15's ×1.51 =
        the dwell ratio EXACTLY — the linearisation made visible).
      • FAR-FLANK EROSION (the headline): the cooling-limited dwell erodes rung-15's over-penetration
        secondary basin by ~18–32%, into NEAR-DEGENERACY with the sharp C_opt notch. The composition
        excess still → 0 AT C_opt (g→0), both immediate flanks up (the notch survives).
      • NOT CLAIMED: which of the two near-degenerate optima is GLOBALLY lowest — it flips sign across
        the β-PDF quadrature (~5%), the φ>2 tail treatment, and the C_e regime (2%→21% over 0.20→0.15),
        all comparable to the margin. Rung 16 quantifies the linearisation error; it does NOT relocate
        the optimum. (Tests assert erosion/sublinearity/near-degeneracy — never a global-min location.)

    `pocket_quench=None` is the exact rung-15 path; at C_opt (g→0) ⟨EI⟩_16 = the finite bulk quench NO
    (ei_no_quenched), NOT ≈0. The clamp-free per-pocket integrator is the CAPABILITY (a pocket that
    goes super-equilibrium rolls over) — DORMANT at the anchored design point (max_a≈0.79 < 1, like
    rung 10's own 0.677); the rung-15↔16 difference here is COOLING within the dwell, not super-eq.

    n_bell/n_quad default SMALLER than rung 15's (each n_bell node is a full `_quench_no` — the
    diagnostic is ~n_bell× costlier than rung 15's single bell integral); the far-basin value is
    converged only to ~5% (the near-degeneracy sits at that resolution)."""
    S: float = 0.0625          # dilution-jet spacing, m (forms the Holdeman group with H and J)
    C_opt: float = 2.5         # Holdeman uniformity optimum of C=(S/H)√J (segregation minimum)
    k_g: float = 0.3           # β-PDF segregation sensitivity to |ln(C/C_opt)| (rung-13 width)
    g_max: float = 0.3         # cap on the segregation (0<g_max<1; rung-13 bimodal bound)
    tau_res: float = 2.5e-3    # under-mixed-pocket dwell AT the optimum, s (absolute; rung 12)
    b_u: float = 3.0           # off-optimum dwell growth (rung-12 core_dwell slope)
    n_bell: int = 120          # per-pocket ξ-grid points (each a full _quench_no — cost driver)
    n_quad: int = 160          # β-PDF quadrature nodes (rung 13)

    def __post_init__(self):
        for name, v in (("S", self.S), ("C_opt", self.C_opt), ("tau_res", self.tau_res)):
            assert v > 0.0, f"PocketQuenchPDF.{name}={v} must be positive"
        assert self.k_g >= 0.0, f"PocketQuenchPDF.k_g={self.k_g} must be ≥ 0 (0 ⇒ g≡0 ⇒ floor only)"
        assert self.b_u >= 0.0, f"PocketQuenchPDF.b_u={self.b_u} must be ≥ 0"
        assert 0.0 < self.g_max < 1.0, f"PocketQuenchPDF.g_max={self.g_max} must be in (0,1)"
        assert self.n_bell > 1 and self.n_quad > 1, "PocketQuenchPDF grid sizes must be > 1"

    def C(self, mixing: "JetMixing") -> float:
        """The Holdeman momentum-flux/geometry group C=(S/H)√J (identical to QuenchPDF.C — the same
        jet that set τ_q). Uses the paired JetMixing's H and J."""
        return (self.S / mixing.H) * math.sqrt(mixing.J)

    def _u(self, C: float) -> float:
        """The unmixedness u(C)=|ln(C/C_opt)| — the KINKED L1 distance from the Holdeman optimum (0 at
        C_opt), driving BOTH the β-PDF width g and the dwell growth (rung-12 + rung-13 kinks)."""
        return abs(math.log(C / self.C_opt))

    def segregation(self, C: float) -> float:
        """The rung-13 β-PDF segregation width g(C)=min(g_max, k_g·u) — KINKED at C_opt (0 there),
        capped at g_max. Sets term 2's composition variance (the per-pocket quench integral width)."""
        return min(self.g_max, self.k_g * self._u(C))

    def core_dwell(self, C: float) -> float:
        """The ABSOLUTE per-pocket dwell τ_core(C)=τ_res·(1+b_u·u) (rung-12 core_dwell) — GROWS
        off-optimum, survives J→∞. Unlike rung 15's `dwell_factor` (a τ_core/τ_ref RATIO multiplying a
        constant-T bell), rung 16 passes this τ_core straight INTO each pocket's `_quench_no`, so the
        dwell acts through the cooling chemistry (⇒ SUBLINEAR, the rung-16 correction)."""
        return self.tau_res * (1.0 + self.b_u * self._u(C))


@dataclass
class TransportedPDF:
    """Rung-18 TRANSPORTED-variance config — the honest LIMIT of the deferred 'transported PDF' seam
    (docs/rung18-spec.md). Rungs 12–17 IMPOSE the β-PDF width as a kinked g(C)=min(g_max, k_g·|ln(C/
    C_opt)|). This config instead solves g(C) as the residual of a mixture-fraction variance DECAY ODE
    dg/dt=−C_φ·ω(C)·g (`_transport_variance`) from a DERIVED two-stream ceiling (`_two_stream_ceiling`),
    then feeds it through the rung-13 IDEAL BELL (`_pdf_mean_ei`). Rides ON a `JetMixing` (needs J and H
    for the Holdeman group C=(S/H)√J), and is ≤1-of-FIVE mutually exclusive with `unmixedness`/`pdf`/
    `pdf_quench`/`pocket_quench` (five closures of the SAME variance physics).

    THE LOAD-BEARING RESULT IS NEGATIVE (and stronger for it). A 0-D variance transport CANNOT DERIVE
    the C_opt optimum: with any MEAN-FIELD ω(J) (or the trajectory τ_q(J)∝1/√J) the residual g(J) is
    MONOTONE/FLAT — no interior optimum (the optimum needs ω peaked at a specific PENETRATION, i.e. the
    SPATIAL spacing S via C=(S/H)√J, absent from the mean-field trajectory). This is rung-11's own
    result: 'mean-field ⇒ no mixing optimum (the variance seam, rung 12)'. So the coverage
    ω(C)=ω_opt·exp(−ln²(C/C_opt)/2w_cov²) below is an EXPLICITLY IMPOSED spatial closure — the honest
    successor of rung-13's kinked g(C), NOT a derivation. A transported/CFD PDF that predicts the
    spatial pattern (and rung-17's firing MAGNITUDE) stays the DEFERRED ceiling.

    WHAT TRANSPORT LEGITIMATELY ADDS (certified; docs/rung18-spec.md gates):
      • a DERIVED two-stream ceiling g_ceiling=(ξ_p−ξ̄)/(1−ξ̄) from φ_p (NOT a free knob) — exposes
        rung-13's g_max=0.3 as ~4.4× too large (φ_p=1.5 ⇒ g_ceiling=0.0675);
      • a RESIDUAL floor g(C_opt)=g_ceiling·exp(−Da_opt)>0 (perfect mixing never reached ⇒ the emissions
        optimum is ELEVATED off the well-mixed value, not the kink's touch-the-floor ≈0);
      • KINK-is-non-generic — the transported g is SMOOTH (both one-sided slopes →0 at C_opt) vs the
        imposed corner (±k_g/C_opt); the sharpness of the pin was the artifact, not its location;
      • the canonical C_φ≈2 scalar-dissipation decay law.

    `transported=None` (default) is the exact prior path; Da_opt→∞ ⇒ g(C_opt)→0 ⇒ the kinked notch (the
    well-mixed point value — the reduce). Like the rung-9..16 knobs, S/Da_opt/w_cov are order-of-
    magnitude; C_opt≈2.5 is Holdeman's, C_φ=2 is anchored; g_ceiling is DERIVED. Certified: the derived
    ceiling, the residual floor, the smoothness (kink-non-genericity), and the NEGATIVE result (optimum
    ⟺ the spatial ω(C))."""
    S: float = 0.0625          # dilution-jet spacing, m (forms the Holdeman group with H and J)
    C_opt: float = 2.5         # Holdeman uniformity optimum of C=(S/H)√J (the coverage peak)
    C_phi: float = 2.0         # scalar-dissipation constant (mechanical-to-scalar timescale ratio; anchored)
    Da_opt: float = 2.0        # optimum Damköhler C_φ·ω_opt·τ (e-folds of variance the best jet decays)
    w_cov: float = 1.0         # coverage width in ln(C/C_opt) (sets the basin breadth; IMPOSED spatial)
    tau_mix: float = 2.5e-3    # mixing residence for the ODE, s (folds into Da; g depends only on the product)
    n_bell: int = 200          # ideal-bell grid points (EI(ξ) built ONCE, interpolated; rung 13)
    n_quad: int = 200          # β-PDF quadrature nodes (rung 13)
    n_ode: int = 400           # variance-ODE integration steps

    def __post_init__(self):
        for name, v in (("S", self.S), ("C_opt", self.C_opt), ("C_phi", self.C_phi),
                        ("Da_opt", self.Da_opt), ("w_cov", self.w_cov), ("tau_mix", self.tau_mix)):
            assert v > 0.0, f"TransportedPDF.{name}={v} must be positive"
        assert self.n_bell > 1 and self.n_quad > 1 and self.n_ode > 1, \
            "TransportedPDF grid sizes (n_bell, n_quad, n_ode) must be > 1"

    def C(self, mixing: "JetMixing") -> float:
        """The Holdeman momentum-flux/geometry group C=(S/H)√J (identical to MixingPDF/PocketQuenchPDF.C
        — the same jet that set τ_q). Uses the paired JetMixing's H and J."""
        return (self.S / mixing.H) * math.sqrt(mixing.J)

    def coverage_omega(self, C: float) -> float:
        """The IMPOSED spatial coverage ω(C)=ω_opt·exp(−ln²(C/C_opt)/2w_cov²), peaked at C_opt (best
        cross-plane tiling ⇒ fastest scalar dissipation). SMOOTH (analytic max ⇒ zero slope at C_opt),
        NOT the kink. ω_opt is folded via Da_opt=C_φ·ω_opt·τ_mix, so this returns ω_opt·(the coverage
        shape). This is the one thing a 0-D transport CANNOT derive (the spatial S enters here); it is
        the explicit successor of rung-13's kinked g(C)."""
        omega_opt = self.Da_opt / (self.C_phi * self.tau_mix)     # from Da_opt = C_φ·ω_opt·τ_mix
        lnr = math.log(C / self.C_opt)
        return omega_opt * math.exp(-lnr * lnr / (2.0 * self.w_cov * self.w_cov))

    def segregation(self, C: float, far_overall: float, phi_primary: float) -> float:
        """The TRANSPORTED width g(C): integrate dg/dt=−C_φ·ω(C)·g from the DERIVED two-stream ceiling.
        A smooth basin (min AT C_opt, from the imposed coverage) ELEVATED off zero by the residual
        g(C_opt)=g_ceiling·exp(−Da_opt)>0. Returns (g, g_ceiling)."""
        g_ceiling = _two_stream_ceiling(far_overall, phi_primary)
        g = _transport_variance(g_ceiling, self.coverage_omega(C), self.tau_mix,
                                c_phi=self.C_phi, nsteps=self.n_ode)
        return g, g_ceiling


@dataclass
class PromptNO:
    """Rung-19 prompt-NO (Fenimore) config — De Soete's (1975) global-rate CORRECTION FACTOR
    reduced to its fitted, rich-peaking φ-shape (docs/rung19-spec.md). An IMPOSED trace channel
    ADDED beside thermal NO; it is the RICH-SPECIFIC lift of the equilibrium-O lower bound (the
    complement of super-eq O, which is T-driven and modest — the naive 'rich explosion' intuition
    fails BOTH ways, and prompt SURVIVES where thermal dies on the rich flank).

      EI_prompt(φ,T) = scale · max(f(φ,n), 0) · exp(−Ea/RuT)
        f(φ,n) = 4.75 + 0.0819·n − 23.2·φ + 32·φ² − 12.2·φ³      (De Soete, valid φ∈[0.6,1.6])

    THE MAGNITUDE IS IMPOSED, not derived — a 0-D burnt pool has no flame structure (thin-zone
    fuel loading, flame-front residence) to anchor the absolute g/kg, so the `scale` is back-solved
    from a REFERENCE-POINT EI `peak_ei` imposed at (`phi_ref`, `T_ref`). `T_ref` is set to a
    REALISTIC near-peak primary AFT (~2400 K) so the reference is physical and the DELIVERED prompt
    peak lands near `peak_ei` (~2 g/kg, the ~1–5 g/kg literature band). NOTE the delivered EI still
    tracks the LOCAL primary T: because prompt carries exp(−Ea/RuT), a hotter primary (T_p > T_ref)
    nudges the delivered EI ABOVE `peak_ei` — `peak_ei` is the reference value, not a hard cap. This
    is HARDER than rung 7's band and is the rung-18-flavored concession: only the φ-SHAPE and the
    directional prompt/thermal ratio are certified, NOT the number. (An imposed closure — rung-7 honesty.)

    The burnt-pool [O2]^a·[FUEL] factors of the full De Soete rate are DROPPED (they double-count O2
    depletion on an already-burnt pool and flip the shape lean-peaking — check_prompt_no.py); the
    rich-peak lives ONLY in De Soete's fitted f(φ). f<0 past φ≈1.65 ⇒ CLAMPED at 0: φ>1.6 is out of
    De Soete's validity, so the deep-rich flank up to the soot bound φ=2 is OUTSIDE the prompt model
    (flagged, not modelled). `prompt=None` ⇒ no additive term (code-path-identical to the prior rung).

    THE T-SENSITIVITY DISCRIMINATOR: prompt carries a SINGLE Arrhenius exp; thermal a DOUBLE
    (k1f·[O]_eq, itself ∝exp). Measured 2000→2400 K at stoich: thermal ×566, prompt ×21 — prompt is
    ~27× milder, the quantitative face of 'survives where thermal dies'."""
    peak_ei: float = 2.0       # IMPOSED reference prompt EI at (phi_ref, T_ref), g NO/kg fuel — the magnitude concession
    n_carbon: float = 12.0     # fuel carbon number (C12 Jet-A surrogate on the per-(CH2) basis; a modeling choice)
    Ea: float = 303474.0       # De Soete activation energy, J/mol (Fluent modified De Soete; transcribed)
    T_ref: float = 2400.0      # reference flame T at which peak_ei is imposed, K (a realistic near-peak primary AFT)
    phi_ref: float = 1.24      # f(φ) cubic-maximum location (where the reference EI is imposed)
    phi_valid_max: float = 1.6 # De Soete φ-validity ceiling; above this f(φ) is extrapolation (flagged)

    def __post_init__(self):
        for name, v in (("peak_ei", self.peak_ei), ("n_carbon", self.n_carbon),
                        ("Ea", self.Ea), ("T_ref", self.T_ref), ("phi_ref", self.phi_ref)):
            assert v > 0.0, f"PromptNO.{name}={v} must be positive"
        assert self.f_correction(self.phi_ref) > 0.0, \
            f"PromptNO.phi_ref={self.phi_ref} sits where f(φ)≤0 — cannot calibrate the scale there"

    def f_correction(self, phi: float) -> float:
        """De Soete's fitted correction factor f(φ,n)=4.75+0.0819n−23.2φ+32φ²−12.2φ³ — a cubic in φ
        peaking slightly rich (~φ=1.24) and going NEGATIVE past φ≈1.65 (the validity ceiling). This
        is where the rich-peaking prompt SHAPE lives; the magnitude is the imposed `scale`."""
        n = self.n_carbon
        return 4.75 + 0.0819 * n - 23.2 * phi + 32.0 * phi ** 2 - 12.2 * phi ** 3

    @property
    def scale(self) -> float:
        """The imposed EI prefactor, back-solved so EI_prompt(phi_ref, T_ref) == peak_ei. Makes the
        one un-derivable magnitude TRANSPARENT (a physical reference EI ~2 g/kg at a realistic primary
        AFT) rather than an opaque pre-exponential. This is the rung-19 concession made legible."""
        return self.peak_ei / (self.f_correction(self.phi_ref)
                               * math.exp(-self.Ea / (_Ru * self.T_ref)))

    def ei_prompt(self, phi: float, T: float) -> float:
        """Imposed prompt EI, g NO/kg fuel = scale·max(f(φ),0)·exp(−Ea/RuT). Clamped ≥0 (f<0 for
        φ>~1.65 ⇒ no negative prompt). Equals `peak_ei` at (phi_ref, T_ref) and rises above it where
        the local primary T exceeds T_ref (the single exp; `peak_ei` is the reference, not a cap). The
        SINGLE exp is why prompt is far less T-sensitive than the double-exp thermal — 'survives
        where thermal dies'."""
        return self.scale * max(self.f_correction(phi), 0.0) * math.exp(-self.Ea / (_Ru * T))


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
    # RUNG 10 — finite-rate quench (all None for the IDEAL quench, tau_q=None → bit-for-bit
    # rung 9). Set only when zoned_nox is called with a finite tau_q.
    tau_q: Optional[float] = None          # quench (dilution-mixing) time, s
    ei_no_quenched: Optional[float] = None # EI_NO re-made along the finite quench, g NO/kg fuel
    x_no_quenched: Optional[float] = None  # NO mole fraction frozen at the end of the finite quench
    T_peak: Optional[float] = None         # peak T along the quench path (> T_primary for a rich primary)
    max_a_quench: Optional[float] = None   # max [NO]/[NO]_e along the path; <1 ⇒ clamp dormant
    # RUNG 11 — the physical jet-entrainment mixing model. Set (with tau_q = the DERIVED
    # τ_q=H/(C_e√J·U_c)) when zoned_nox is called with a `mixing` config; None otherwise.
    mixing: Optional[JetMixing] = None     # the jet-in-crossflow config that DERIVED tau_q + schedule
    # RUNG 12 — spatial unmixedness (the two-stream variance layer). Set when zoned_nox is called
    # with an `unmixedness` config (requires `mixing`); all None for the rung-11 mean field.
    # `ei_no_quenched` then holds the mean-field BULK (rung-11) EI and `ei_no_unmixed` the two-
    # stream total — the one that TURNS BACK UP in J, recovering the Holdeman optimum.
    unmixedness: Optional["Unmixedness"] = None  # the variance config that split bulk/core
    C_holdeman: Optional[float] = None     # the Holdeman group C=(S/H)√J at this J
    w_core: Optional[float] = None         # the un-mixed core mass fraction w(C) (0 at C_opt)
    ei_no_unmixed: Optional[float] = None  # two-stream EI_NO, g/kg — (1−w)·EI(τ_mean)+w·EI(τ_core)
    ei_no_core: Optional[float] = None     # the lingering core's EI_NO at τ_core(C) (the penalty source)
    # RUNG 13 — the resolved mixing PDF (replaces rung-12's parameterised SEGREGATION with a
    # continuous distribution; drops the dwell). Set when zoned_nox is called with a `pdf` config
    # (requires `mixing`, mutually exclusive with `unmixedness`); None otherwise. `ei_no_quenched`
    # still holds the mean-field bulk reference; `ei_no_pdf` is the β-PDF integral over the IDEAL
    # bell — a sharp emissions minimum PINNED AT C_opt (both flanks up), from a continuous
    # distribution rather than two lumps (the far-over-penetration flank descends — the humped
    # ⟨EI⟩(g); the over-penetration climb is rung-12's dwell effect, absent here).
    pdf: Optional["MixingPDF"] = None      # the β-PDF config used
    g_seg: Optional[float] = None          # the segregation width g(C) (0 at C_opt); reused by rung 15
    ei_no_pdf: Optional[float] = None      # ⟨EI⟩ over the β-PDF of the ideal bell, g/kg — min AT C_opt
    # RUNG 15 — the PDF THROUGH the finite quench (composition variance AND dwell, COMBINED). Set when
    # zoned_nox is called with a `pdf_quench` config (requires `mixing`, mutually exclusive with `pdf`
    # and `unmixedness`); None otherwise. `ei_no_quenched` holds term 1 (the rung-11 mean-field bulk
    # quench, the FINITE floor rung 13 lacked); `ei_no_pdf_quench` = term1 + term2 — the combined result
    # (min AT C_opt with a FINITE floor, far flank CLIMBING — distinguishable from BOTH rung 12 and 13).
    pdf_quench: Optional["QuenchPDF"] = None  # the PDF-through-quench config used (C_holdeman/g_seg reused)
    ei_no_pdf_excess: Optional[float] = None  # term 2 = D(u)·⟨EI_bell⟩(g), g/kg — resolved composition×dwell
    ei_no_pdf_quench: Optional[float] = None  # term1 + term2, g/kg — the combined result (the finite floor)
    # RUNG 16 — the PDF through the finite quench, PER POCKET (retires rung-15's linearised dwell). Set
    # when zoned_nox is called with a `pocket_quench` config (requires `mixing`, ≤1-of-four with pdf/
    # pdf_quench/unmixedness); None otherwise. `ei_no_quenched` still holds term 1 (the mean-field bulk
    # floor); `ei_no_pocket_excess` is term 2 = ⟨EI_pocket_quench(ξ;τ_core)⟩_g (each rich pocket carried
    # through its OWN quench — SUBLINEAR in τ_core, the cooling correction); `ei_no_pocket_quench` =
    # term1 + term2 (erodes rung-15's far flank into near-degeneracy with the C_opt notch).
    pocket_quench: Optional["PocketQuenchPDF"] = None  # the per-pocket PDF-through-quench config used
    ei_no_pocket_excess: Optional[float] = None  # term 2 — the per-pocket quench β-PDF integral, g/kg
    ei_no_pocket_quench: Optional[float] = None  # term1 + term2, g/kg — the combined result (rung 16)
    # RUNG 18 — the TRANSPORTED-variance closure (what 0-D CAN/CANNOT derive). Set when zoned_nox is
    # called with a `transported` config (requires `mixing`, ≤1-of-five with the other four); None
    # otherwise. The segregation width g(C) is no longer the imposed kink but the residual of a variance
    # ODE dg/dt=−C_φ·ω(C)·g from a DERIVED two-stream ceiling — a smooth basin ELEVATED off the
    # well-mixed value (the residual floor g(C_opt)>0). The `C_opt` LOCATION still rides on the IMPOSED
    # spatial coverage ω(C) (a 0-D transport cannot derive it — the rung-11/12 variance seam). Fed
    # through the rung-13 ideal bell. `ei_no_pdf` is NOT set here (different closure); `g_seg` reused.
    transported: Optional["TransportedPDF"] = None  # the transported-variance config used
    g_ceiling: Optional[float] = None      # DERIVED two-stream injection ceiling (ξ_p−ξ̄)/(1−ξ̄) from φ_p
    g_transported: Optional[float] = None  # the ODE-residual width g(C) (≤ g_ceiling; >0 at C_opt)
    ei_no_transported: Optional[float] = None  # ⟨EI⟩ over the β-PDF of the ideal bell at g_transported
    # RUNG 19/20 — the two lower-bound-lifting channels (both off ⇒ bit-for-bit the prior rung).
    # super_eq_o lifts the [O] by m(T) inside the Zeldovich integrator; prompt ADDS the imposed De Soete
    # φ-bump `ei_no_prompt`. RUNG 19 lifted only the PRIMARY (folded into `primary.ei_no`/`x_no_mix`).
    # RUNG 20 threads super_eq_o THROUGH the finite-quench re-making too: `ei_no_quenched`, the rung-12
    # core, the rung-16 per-pocket, and the rung-17 clamp now carry the SAME m(T)-lift along the cooling
    # path (`super_eq_o` here is the single flag for both). The lift is bounded & peak-concentrated
    # (≈m(T_peak)) — the ideal-bell composition integrals (pdf/pdf_quench/transported) DELIBERATELY stay
    # equilibrium-O (forbidden to combine — see the zoned_nox guard; docs/rung20-spec.md).
    super_eq_o: bool = False               # whether the [O] was super-eq-lifted (primary AND, rung 20, the quench)
    o_multiplier: float = 1.0              # the m(T_p) applied to the primary O (1.0 ⇒ rung 7 baseline)
    prompt: Optional["PromptNO"] = None    # the imposed prompt-NO config, if any
    ei_no_prompt: float = 0.0              # additive primary prompt EI, g NO/kg fuel (0.0 ⇒ thermal only)

    @property
    def ei_no_total(self) -> float:
        """Total primary EI = thermal (super-eq-O-lifted) + prompt, g NO/kg fuel (rung 19). The
        equilibrium-O lower bound (`ei_no`) lifted the two certified ways — the T-driven super-eq-O
        factor already in `ei_no` via the lifted [O], plus the IMPOSED additive prompt bump."""
        return self.ei_no + self.ei_no_prompt

    @property
    def ei_no_quenched_total(self) -> Optional[float]:
        """RUNG 20 — total finite-quench EI = re-made thermal (`ei_no_quenched`, super-eq-O-lifted when
        super_eq_o) + the invariant prompt (`ei_no_prompt`), g NO/kg fuel; None for the ideal quench.

        Prompt rides the quench UNCHANGED: EI is per-kg-fuel and prompt is a flame-front phenomenon set
        at the primary, so dilution lowers its mole fraction but not its emission index. We ADD it here
        rather than inject prompt moles into `_quench_no`'s cooling chemistry ON PURPOSE — prompt's
        MAGNITUDE is imposed (rung-19 concession), so running an un-certified number through Zeldovich
        destruction would be false precision. (This is why prompt is kept OUT of the rung-17 clamp `a`,
        which stays a certified-thermal margin.) docs/rung20-spec.md."""
        if self.ei_no_quenched is None:
            return None
        return self.ei_no_quenched + self.ei_no_prompt

    @property
    def ei_no(self) -> float:
        """Emission index, g NO / kg fuel — set in the primary, conserved through dilution
        (α cancels: NO moles and fuel moles both scale with the primary air fraction).

        This is the IDEAL-quench (rung 9) EI. For the finite quench (rung 10) use
        `ei_no_quenched`, which RE-MAKES NO at the stoichiometric crossing the dilution passes
        through. At the lean main.py design point `ei_no_quenched ≥ ei_no` because NO stays
        BELOW equilibrium the whole way (max_a_quench < 1 — the documented dormancy); in
        GENERAL the dropped clamp permits NO to DECREASE where the inherited NO is
        super-equilibrium on the cooling path (a>1), so ≥ is not guaranteed off this point."""
        return self.primary.ei_no

    @property
    def ppm_primary(self) -> float:
        return self.primary.x_no * 1e6

    @property
    def ppm_mix(self) -> float:
        return self.x_no_mix * 1e6


@dataclass
class NozzleFlowState:
    """Frozen-vs-equilibrium nozzle-flow diagnostic (rung 14; docs/rung14-spec.md). A pure
    diagnostic BESIDE the cycle — the production nozzle stays FROZEN, so the cycle is bit-for-bit
    rung 6. Velocities m/s, temperatures K, mole fractions dimensionless; compositions are mole
    numbers per mol dry air.

    THE THRUST BRACKET (major species): V9_frozen ≤ V9(real) ≤ V9_equilibrium. The gap `dV9` is the
    recombination energy a shifting expansion recovers (CO/H₂/OH/O/H → CO₂/H₂O on cooling). It is
    NEGLIGIBLE at the cool lean design point (dissociation ≈ 0) and grows with combustor temperature.
    THE CLAMP COROLLARY (NO): on the same cooling path equilibrium NO collapses (`no_collapse_ratio`),
    so any realistic frozen exhaust NO is super-equilibrium and rung 7's dropped clamp would fire
    (`max_a` ≫ 1, vs rung 10's dormant 0.677 — see `_nozzle_clamp_diag`)."""
    # --- the thrust bracket (major species) ---
    T9_frozen: float                  # frozen exit static temperature (== production nozzle), K
    T9_equilibrium: float             # shifting-equilibrium exit static temperature, K (> frozen)
    V9_frozen: float                  # frozen exhaust velocity (== production nozzle), m/s
    V9_equilibrium: float             # shifting-equilibrium exhaust velocity, m/s (≥ frozen)
    co_fraction_entry: float          # CO/(CO+CO2) at the nozzle entry — the dissociation content
    comp_entry: dict                  # frozen station-4 mixture (mole numbers per mol air)
    comp_exit_eq: dict                # shifted equilibrium exit mixture
    # --- the dropped-clamp corollary (NO) ---
    x_no_e_entry: float               # equilibrium NO mole fraction at the nozzle entry (Tt9)
    x_no_e_exit: float                # equilibrium NO at the exit (T9_frozen) — collapsed
    no_collapse_ratio: float          # x_no_e_entry / x_no_e_exit (frozen-NO-independent)
    x_no_frozen: Optional[float] = None   # the caller's frozen exhaust NO (e.g. rung-8 zoned); None if absent
    max_a: Optional[float] = None     # x_no_frozen / x_no_e_exit; > 1 ⇒ the dropped clamp fires

    @property
    def dV9(self) -> float:
        """Extra exhaust velocity a shifting (equilibrium) expansion recovers over frozen, m/s."""
        return self.V9_equilibrium - self.V9_frozen

    @property
    def dV9_frac(self) -> float:
        """`dV9` as a fraction of the frozen exhaust VELOCITY. The specific-THRUST delta is modestly
        larger — ΔF/F = ΔV9/(V9 − V0/(1+f)), so the flight ram term (V0) lifts it ~1.2–1.35× at
        M0=0.85 (≈0.56% vs the ≈0.46% velocity fraction at the hot anchor)."""
        return self.dV9 / self.V9_frozen

    @property
    def clamp_fires(self) -> bool:
        """Would rung 7's dropped NO clamp fire on the nozzle cooling path? (max_a > 1.)"""
        return self.max_a is not None and self.max_a > 1.0


@dataclass
class ExhaustNOxClampState:
    """Rung-17 combustor-mixing-fidelity ladder of the exhaust-NO clamp margin (docs/rung17-spec.md).

    A pure DIAGNOSTIC composing rung 16 (the per-pocket exhaust NO) with rung 14 (the nozzle cooling
    collapse) at a RICH RQL primary — the cycle stays bit-for-bit rung 6. It asks ONE question three
    ways: carry the exhaust NO from three combustor-mixing-fidelity models through the SAME rung-14
    nozzle collapse to T9 and read the dropped-clamp margin a=[NO]/[NO]_e(T9):

      MIXED-OUT   (rung 8,  x_no_mix)        — the standard shortcut; at a RICH primary it reads
                                               DORMANT (a<1): mixing-out HIDES the super-eq NO.
      BULK QUENCH (rung 11, x_no_quenched)   — the dilution re-making restored; FIRES (a>1).
      PER-POCKET  (rung 16, β-PDF segregation) — the segregation-raised mean; FIRES harder.

    The LOAD-BEARING content splits in two (different strengths — do NOT bundle them):
      (a) the ORDERING a_mixed ≤ a_bulk ≤ a_pocket is STRUCTURAL — the clamp-free quench only ADDS NO to
          the mixed-out pool (x_no_quenched ≥ x_no_mix in the dormant regime) and the per-pocket excess
          is additive (x_no_pocket = x_no_bulk + κ·⟨EI⟩_pocket, ⟨EI⟩ ≥ 0). Plus a_mixed<1 is robust (a
          rich primary makes ≈0 NO). THIS is the certified claim.
      (b) the FIRING (a_bulk>1, a_pocket>1) is the UN-PINNED threshold: it holds across the RQL J-band
          but is NOT universal — as the quench gets FAST (J→∞) x_no_quenched→x_no_mix (the rung-10
          τ_q→0 reduce: fast quench = ideal quench = mixed-out) so a_bulk→a_mixed<1 (dormant). Every
          firing MAGNITUDE and the gap ride on un-pinned mixing scales (C_e, τ_res, H, J).
    So the HEADLINE — mixing-out HIDES super-eq NO the fuller models reveal — rides on the IN-BAND
    firing, and that in-band firing IS the lesson (not a universal claim; docs/rung17-spec.md § scope).
    The pocket/bulk RATIO equals rung-16's station-4 gap EXACTLY — the nozzle denominator x_no_e(T9) is
    COMMON and cancels, so the nozzle is a NO-OP on the ratio (a SYNTHESIS of rungs 11/16/14, not new
    physics; STATED, not gated).

    CONTRAST rung 14 (docs/rung14-spec.md): rung 14 fires on the φ_p=1.0 mixed-out number (a≈250) — the
    ZONED-vs-UNZONED axis. Rung 17 is the MIXING-FIDELITY axis at the RICH φ_p=1.5 primary, where the
    mixed-out number is deceptively DORMANT (a≈0.02). NOT a contradiction — the same lesson from the
    other side: what NO actually leaves the engine depends on how faithfully the combustor mixing is
    modeled; the crude mixed-out shortcut is UNCONSERVATIVE exactly where the primary runs rich."""
    phi_primary: float               # the rich RQL primary equivalence ratio (the regime that hides NO)
    T9: float                        # frozen nozzle-exit static temperature, K (the cooling endpoint)
    x_no_e_exit: float               # equilibrium NO at T9 — the COMMON clamp denominator (rung 14)
    no_collapse_ratio: float         # x_no_e(Tt9)/x_no_e(T9) — the cooling collapse (frozen-NO-independent)
    # the three exhaust-NO models (mole fractions) and their clamp margins a=[NO]/[NO]_e(T9):
    x_no_mixed_out: float            # rung-8 mixed-out exhaust NO
    x_no_bulk_quench: float          # rung-11 mean-field bulk-quench exhaust NO
    x_no_pocket: float               # rung-16 per-pocket β-PDF-mean exhaust NO (= κ·ei_no_pocket_quench)
    a_mixed_out: float               # x_no_mixed_out/x_no_e_exit  (< 1 ⇒ mixing-out HIDES the super-eq NO)
    a_bulk_quench: float             # x_no_bulk_quench/x_no_e_exit (> 1 ⇒ the quench re-making fires it)
    a_pocket: float                  # x_no_pocket/x_no_e_exit     (> 1, larger ⇒ segregation raises it more)
    # station-4 quantities behind the pocket rung (transparency; the gap is rung-16's, itself scale-dependent):
    ei_no_quenched: float            # rung-11 bulk-quench EI (term 1), g NO/kg fuel
    ei_no_pocket_quench: float       # rung-16 per-pocket EI (term1+term2), g NO/kg fuel
    gap_pocket_over_bulk: float      # ei_no_pocket_quench/ei_no_quenched ≡ a_pocket/a_bulk (nozzle cancels)
    max_a_quench: float              # rung-16 STATION-4 clamp dormancy (max [NO]/[NO]_e over the pockets)

    @property
    def hides_super_eq(self) -> bool:
        """The rung-17 headline: does mixing-out HIDE the super-equilibrium exhaust NO the fuller
        models reveal? (mixed-out dormant AND the bulk quench fires.)"""
        return self.a_mixed_out < 1.0 < self.a_bulk_quench

    @property
    def ladder_monotone(self) -> bool:
        """The load-bearing direction: a_mixed_out < a_bulk_quench < a_pocket (fidelity → more NO)."""
        return self.a_mixed_out < self.a_bulk_quench < self.a_pocket


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
    def thermal_nox(self, far: float, T: float, p: float, tau: float = 3e-3,
                    super_eq_o: bool = False, prompt: Optional["PromptNO"] = None,
                    phi: Optional[float] = None) -> "NOxState":
        """Thermal-NO diagnostic on the equilibrium pool at (far, T, p) after residence
        time tau (default 3 ms, a typical gas-turbine primary-zone residence — an
        UN-ANCHORED knob, stated like the specified exit pressure). Solves the rung-6
        equilibrium composition (scale-A, datum-free mole numbers), superimposes
        equilibrium NO, and integrates the extended Zeldovich mechanism. Trace species
        => this does NOT affect the cycle; it is a pure diagnostic (docs/rung7-spec.md).

        RUNG 19 — lift the equilibrium-O LOWER BOUND two ways (docs/rung19-spec.md):
          super_eq_o=True  — lift the pool's [O] by the Westenberg partial-equilibrium
                             multiplier m(T)=`_super_eq_o_multiplier(T)` inside the integrator
                             (a T-driven ~1.16–1.50× factor). super_eq_o=False ⇒ m=1 ⇒ bit-for-bit
                             rung 7.
          prompt=PromptNO() — ADD the imposed De Soete prompt (Fenimore) EI at the local φ
                             (defaults to far/f_stoich): `ei_no_prompt`, an additive rich-peaking
                             bump. prompt=None ⇒ no term. Both off ⇒ the exact prior code path.
        The SUMMED trace guard spans both channels (thermal-lifted + prompt); NO stays trace, so
        the cycle is untouched."""
        comp = _equilibrium_composition(far, T, p)
        m = _super_eq_o_multiplier(T) if super_eq_o else 1.0
        assert 1.0 <= m <= 2.0, (
            f"super-eq O multiplier m={m:.3f} at T={T:.0f} K outside the flame-band bound [1,2] — "
            "the Westenberg partial-eq/eq ratio is a FLAME model (T≳1500 K)"
        )
        nox = _thermal_no(comp, T, p, tau, far, o_multiplier=m)
        if prompt is not None:
            phi_local = phi if phi is not None else far / _F_STOICH
            nox.ei_no_prompt = prompt.ei_prompt(phi_local, T)
        # SUMMED trace guard (rung 19): thermal (m-lifted) + prompt must stay trace. x_no ∝ EI at
        # fixed far, so convert the prompt EI to a mole fraction via the thermal x_no/EI ratio.
        x_no_prompt = (nox.ei_no_prompt / nox.ei_no * nox.x_no) if nox.ei_no > 0.0 else 0.0
        assert nox.x_no + x_no_prompt < 0.02, (
            f"summed NO not trace (x_NO_thermal+prompt={nox.x_no + x_no_prompt:.4g}) — "
            "decoupling invalid"
        )
        return nox

    # --- Rung-8 two-zone combustor NOx diagnostic (DECOUPLED; never feeds the cycle) -----
    def zoned_nox(self, far: float, Tt3: float, Tt4: float, p: float,
                  phi_primary: float, tau: float = 3e-3,
                  tau_q: Optional[float] = None,
                  mixing: Optional["JetMixing"] = None,
                  unmixedness: Optional["Unmixedness"] = None,
                  pdf: Optional["MixingPDF"] = None,
                  pdf_quench: Optional["QuenchPDF"] = None,
                  pocket_quench: Optional["PocketQuenchPDF"] = None,
                  transported: Optional["TransportedPDF"] = None,
                  super_eq_o: bool = False, prompt: Optional["PromptNO"] = None,
                  quench_ngrid: int = 240, quench_nsteps: int = 2000) -> "ZonedNOxState":
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
        primary value. Held below soot onset (phi_primary ≤ 2.0; docs/rung9-spec.md).

        RUNG 10 — the FINITE-RATE quench (docs/rung10-spec.md). Pass a finite `tau_q` (the
        quench/dilution-mixing time, s) to resolve the quench in TIME instead of collapsing it
        to an instant. As the dilution air mixes in over `tau_q`, the LOCAL mixture sweeps
        far_p → f_stoich → far_ov — through STOICHIOMETRIC — so a RICH primary's temperature
        RISES through the NO-bell peak on the way down, and the extended-Zeldovich rate RE-MAKES
        NO along that path (a clamp-free integrator; super-equilibrium NO on cooling must not be
        capped — Heywood). A SLOW quench dwells at stoich and re-makes the NO the rich primary
        avoided; a FAST quench escapes past the peak — the whole point of "quick"-quench. The
        rung-9 rich-flank collapse is thus CONTINGENT on a fast quench (`ei_no_quenched` fills
        the rich flank back in). `tau_q=None` (default) is the IDEAL quench — the EXACT rung-9
        path, so every existing call stays bit-for-bit rung 9 (hence bit-for-bit rung 6).

        RUNG 11 — the PHYSICAL jet-entrainment quench (docs/rung11-spec.md). Pass a `mixing`
        (a `JetMixing` config) INSTEAD of `tau_q` to DERIVE the quench from jet-in-crossflow
        physics: τ_q = H/(C_e·√J·U_c) from the momentum-flux ratio J (so "quick quench" = a
        high-momentum jet), and a decelerating ENTRAINMENT schedule β(t)=1−(1−t/τ_q)^n instead
        of rung 10's linear one. `mixing` and `tau_q` are MUTUALLY EXCLUSIVE (assert one-or-the-
        other; like the isentropic/polytropic knob). EI_NO_quenched falls MONOTONICALLY as J
        rises (a stronger jet escapes the stoich peak faster → less re-made NO). This is a
        MEAN-FIELD model (one well-mixed core): it derives the quench RATE but has no mixing
        OPTIMUM (a variance effect — deferred to rung 12). `mixing=None` (default) keeps the
        exact rung-9/10 paths, so every existing call is untouched.

        RUNG 12 — SPATIAL UNMIXEDNESS (docs/rung12-spec.md). Pass an `unmixedness` (an
        `Unmixedness` config; REQUIRES `mixing`) to add the two-stream VARIANCE layer rung 11
        deferred, finally making the NO-vs-J curve TURN BACK UP and recovering the Holdeman
        dilution-jet optimum AT C_opt≈2.5. The flow splits into a mean-field BULK (fraction 1−w,
        quenched at the rung-11 jet time τ_mean — the still-falling reference) and an UNDER-MIXED
        CORE (fraction w=w(C), quenched at an ABSOLUTE dwell τ_core(C)=τ_res·(1+b_u·u) that misses
        the fast jet and grows off-optimum). The unmixedness u(C)=|ln(C/C_opt)| is KINKED at the
        Holdeman group C_opt (both w and τ_core →0-penalty there), so `ei_no_unmixed` = (1−w)·
        EI(τ_mean)+w·EI(τ_core) FALLS to an interior minimum AT C_opt then RISES — the min pinned
        at C_opt (J_min=J_opt, shifting as (H/S)²). `unmixedness=None` (default) is the exact
        rung-11 mean field; k_u=0 is bit-for-bit rung 11. NO is still trace → cycle bit-for-bit
        rung 6. `ei_no_quenched` still holds the mean-field BULK (the monotone reference).

        RUNG 13 — the RESOLVED MIXING PDF (docs/rung13-spec.md). Pass a `pdf` (a `MixingPDF` config;
        REQUIRES `mixing`, MUTUALLY EXCLUSIVE with `unmixedness`) to replace rung-12's parameterised
        SEGREGATION with a CONTINUOUS mean-preserving β-PDF of mixture fraction. `ei_no_pdf` =
        ∫ EI_bell(φ(ξ))·P_β(ξ; ξ̄, g(C)) dξ integrates the IDEAL primary bell over the distribution,
        its width the segregation g(C)=min(g_max, k_g·|ln(C/C_opt)|) (the same kinked Holdeman
        distance as rung-12's w). The lesson, stated correctly: NO is sharply PEAKED at stoich, so
        segregation RAISES the mean NO whenever the mean is OFF-stoich (our lean dilution mean),
        reversing sign at a stoich mean — NOT generic "convexity". ⟨EI⟩(g(C)) collapses to the well-
        mixed lean value AT C_opt (g=0 ⇒ delta) with BOTH immediate flanks lifting — the Holdeman
        optimum LOCATION recovered from a continuous PDF (J_min=J_opt, shifting as (H/S)²). A
        MECHANISM SEPARATION, not a rung-12 reproduction: this isolates the COMPOSITION mechanism and
        drops the finite-quench dwell chain, so it pins the optimum but CANNOT climb — the far-over-
        penetration flank DESCENDS (the humped ⟨EI⟩(g), β-PDF going bimodal). Rung-12's over-
        penetration climb was the DWELL effect; combining both (the PDF through the quench) is the
        rung-15 seam (and the min ≈0 here — dropping the bulk floor — is that scope boundary).
        `pdf=None` (default) is the exact rung-12 path; g→0 is the well-mixed point value. NO still
        trace → cycle bit-for-bit rung 6. `ei_no_quenched` still holds the mean-field bulk reference.

        RUNG 15 — the PDF THROUGH the finite quench (docs/rung15-spec.md). Pass a `pdf_quench` (a
        `QuenchPDF` config; REQUIRES `mixing`, MUTUALLY EXCLUSIVE with `pdf` and `unmixedness`) to
        finally COMBINE the two mixing mechanisms rungs 12–13 kept isolated: the COMPOSITION variance
        (rung 13) AND the DWELL (rung 12). Additive — `ei_no_pdf_quench` = term1 + term2 where term1 =
        `ei_no_quenched` (the rung-11 mean-field bulk quench, the FINITE floor rung 13 dropped) and
        term2 = D(u)·⟨EI_bell⟩(g) (the rung-13 β-PDF integral over the ideal bell, reused verbatim,
        scaled by an off-optimum-growing dwell factor D(u)=τ_res(1+b_u·u)/tau — τ_core/τ_ref, rescaling
        the reference-τ bell EI to the pocket's lingering dwell; EI ∝ τ, dormant clamp). The result: the
        ≈0 rung-13 optimum floor BECOMES the finite bulk quench NO (at C_opt, g→0 ⇒ term2→0 ⇒
        ei_no_pdf_quench = ei_no_quenched, NOT ≈0), the min is PINNED AT C_opt (J_min=J_opt, (H/S)²
        shift), BOTH flanks lift, and the far over-penetration flank CLIMBS again (the dwell restored,
        surviving J→∞) — distinguishable from BOTH parents. The nonlinear bell keeps the STOICH-MEAN
        SIGN REVERSAL (the discriminator a lumped-dwell rung-12-in-disguise fails). `pdf_quench=None`
        (default) is the exact rung-13 path. NO still trace → cycle bit-for-bit rung 6.

        RUNG 19 — lift the equilibrium-O LOWER BOUND on the PRIMARY (docs/rung19-spec.md). Every NO
        number since rung 7 read the rung-6 EQUILIBRIUM [O], so it is a lower bound. Two knobs lift it,
        and the load-bearing result is that BOTH contradict the naive "the rich primary explodes"
        intuition, from opposite directions:
          super_eq_o=True  — lift the primary [O] by the Westenberg partial-equilibrium multiplier
                             m(T_p)∈[1.16,1.50] inside the rung-7 integrator. T-DRIVEN, not rich-driven
                             (φ-independent; WEAKEST in the O2-starved rich primary). super_eq_o=False
                             ⇒ m=1 ⇒ bit-for-bit the prior rung.
          prompt=PromptNO() — ADD the imposed De Soete prompt (Fenimore) φ-bump `ei_no_prompt` at
                             phi_primary. RICH-SPECIFIC: it SURVIVES where thermal dies (prompt/thermal
                             grows monotonically rich, 0.4→455 across φ=1.0→1.5), and is ~27× less
                             T-sensitive (single vs double Arrhenius exp). prompt=None ⇒ no term.
        Both act ONLY on the primary diagnostic (`ei_no`/`x_no_mix`, and `ei_no_total`=`ei_no`+prompt);
        the finite-quench/PDF fields stay equilibrium-O (threading the lift THROUGH the quench is a
        deferred seam). The SUMMED trace guard spans both channels; NO stays trace ⇒ cycle bit-for-bit
        rung 6. The prompt MAGNITUDE is IMPOSED (a 0-D pool has no flame structure); only the φ-shape
        and the directional prompt/thermal ratio are certified (docs/rung19-spec.md § the concessions).

        `quench_ngrid`/`quench_nsteps` are the finite-quench numerical resolution (trajectory
        points / RK4 steps) — pure cost/accuracy knobs, only used when `tau_q` is finite. The 240
        default reproduces the anchor's worked example (docs/plans/rung10-anchor-quench.md); EI_NO
        is within ~0.1 % by ngrid≈120 and the SHAPE (T_peak, monotonicity) by ngrid≈32, so tests
        and `main.py` pass a smaller ngrid to stay interactive (a 240-point trajectory is ~25 s —
        each point re-equilibrates the diluting majors via a bisection `_mixed_out_T`)."""
        assert 0.0 < phi_primary <= 2.0 + 1e-9, (
            f"phi_primary {phi_primary} outside (0, 2] — the 5-species (no soot / no C(s)) "
            "basis is valid only below soot onset (~φ2; graphite onset is φ3). Rich RQL scope."
        )
        assert not (tau_q is not None and mixing is not None), (
            "pass EITHER tau_q (rung-10 free time + linear schedule) OR mixing (rung-11 "
            "jet-entrainment: DERIVES τ_q + a decelerating schedule) — they are mutually exclusive."
        )
        assert not (unmixedness is not None and mixing is None), (
            "unmixedness (rung-12 spatial variance) REQUIRES a `mixing` config — it needs the jet's "
            "J and duct H for the Holdeman group C=(S/H)√J and the mean-field bulk τ_mean."
        )
        assert not (pdf is not None and mixing is None), (
            "pdf (rung-13 resolved mixing PDF) REQUIRES a `mixing` config — it needs the jet's J and "
            "duct H for the Holdeman group C=(S/H)√J that sets the β-PDF segregation width g(C)."
        )
        assert not (pdf_quench is not None and mixing is None), (
            "pdf_quench (rung-15 PDF through the quench) REQUIRES a `mixing` config — it needs the jet's "
            "J and duct H for the Holdeman group C=(S/H)√J AND the derived τ_mean for the mean-field floor."
        )
        assert not (pocket_quench is not None and mixing is None), (
            "pocket_quench (rung-16 PER-POCKET PDF through the quench) REQUIRES a `mixing` config — it "
            "needs the jet's J and duct H for the Holdeman group C=(S/H)√J AND the derived τ_mean floor."
        )
        assert not (transported is not None and mixing is None), (
            "transported (rung-18 transported-variance closure) REQUIRES a `mixing` config — it needs the "
            "jet's J and duct H for the Holdeman group C=(S/H)√J that the imposed coverage ω(C) rides on."
        )
        assert sum(x is not None for x in (unmixedness, pdf, pdf_quench, pocket_quench, transported)) <= 1, (
            "pass AT MOST ONE of unmixedness (rung-12 two-stream) / pdf (rung-13 β-PDF on the ideal bell) "
            "/ pdf_quench (rung-15 β-PDF THROUGH the quench, LINEARISED dwell) / pocket_quench (rung-16 "
            "PER-POCKET quench) / transported (rung-18 transported variance) — FIVE closures of the SAME "
            "variance physics."
        )
        assert not (super_eq_o and (pdf is not None or pdf_quench is not None or transported is not None)), (
            "RUNG 20 — super_eq_o threads the lower-bound lift ONLY through the _quench_no re-making "
            "(the mean-field bulk `ei_no_quenched`, the rung-12 core, the rung-16 per-pocket, and the "
            "rung-17 clamp). The ideal-bell composition integrals — pdf (rung 13), pdf_quench (rung-15 "
            "term 2), transported (rung 18) — DELIBERATELY stay equilibrium-O lower bounds (docs/"
            "rung20-spec.md): pdf_quench would become a half-lifted HYBRID (lifted term1 + eq-O term2). "
            "Combine super_eq_o with mixing / unmixedness / pocket_quench only."
        )
        hf_fuel = (self.hf_fuel_molar if self.hf_fuel_molar is not None
                   else _HF_FUEL_DEFAULT)   # 0.0 is a valid ΔHf (elements are the datum)
        far_p = phi_primary * _F_STOICH
        alpha = far / far_p                              # fraction of the air in the primary
        assert alpha <= 1.0 + 1e-9, \
            f"primary air fraction α={alpha:.4f} > 1 — overall mixture leaner than the primary"

        T_p = _primary_aft(far_p, p, Tt3, hf_fuel)
        comp_p = _equilibrium_composition(far_p, T_p, p)
        # RUNG 19 — lift the equilibrium-O LOWER BOUND on the PRIMARY (super_eq_o=False, prompt=None
        # ⇒ m=1, no prompt ⇒ the integrator call is byte-identical to the prior rung). super-eq O is
        # a T-driven m(T_p)~1.16–1.50× on the primary [O]; prompt is the imposed De Soete φ-bump at
        # phi_primary. Both act ONLY on the primary diagnostic (nox / x_no_mix); the finite-quench
        # and PDF fields below stay equilibrium-O (the through-the-quench lift is a deferred seam).
        m_p = _super_eq_o_multiplier(T_p) if super_eq_o else 1.0
        assert 1.0 <= m_p <= 2.0, (
            f"super-eq O multiplier m={m_p:.3f} at primary T={T_p:.0f} K outside the flame-band "
            "bound [1,2] — the Westenberg partial-eq/eq ratio is a FLAME model (T≳1500 K)"
        )
        nox = _thermal_no(comp_p, T_p, p, tau, far_p, o_multiplier=m_p)   # rung 7/19 integrator
        ei_no_prompt = prompt.ei_prompt(phi_primary, T_p) if prompt is not None else 0.0
        nox.ei_no_prompt = ei_no_prompt
        # SUMMED trace guard (rung 19): the primary thermal (m-lifted) + prompt must stay trace.
        x_no_prompt_p = (ei_no_prompt / nox.ei_no * nox.x_no) if nox.ei_no > 0.0 else 0.0
        assert nox.x_no + x_no_prompt_p < 0.02, (
            f"summed primary NO not trace (x_NO_thermal+prompt={nox.x_no + x_no_prompt_p:.4g}) — "
            "decoupling invalid"
        )

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

        state = ZonedNOxState(phi_primary=phi_primary, far_primary=far_p, alpha=alpha,
                              T_primary=T_p, T_mix=T_mix, primary=nox, x_no_mix=x_no_mix,
                              super_eq_o=super_eq_o, o_multiplier=m_p, prompt=prompt,
                              ei_no_prompt=ei_no_prompt)
        if tau_q is None and mixing is None:
            return state                                 # IDEAL quench — bit-for-bit rung 9

        # RUNG 10/11 — finite-rate quench: re-integrate NO (clamp-free) through the cooling/mixing
        # trajectory, starting from the primary's frozen NO (n_no_total). A pure diagnostic —
        # NO/N still never enter _equil_solve, so the cycle stays bit-for-bit rung 6. Rung 10 = a
        # free `tau_q` + linear schedule; rung 11 = `mixing` DERIVES τ_q (=H/(C_e√J·U_c)) and a
        # decelerating entrainment schedule (mixing=None ⇒ schedule=None ⇒ byte-identical rung 10).
        if mixing is not None:
            tau_q_eff, schedule = mixing.tau_q, mixing.schedule      # rung 11: derived from jet J
        else:
            tau_q_eff, schedule = tau_q, None                        # rung 10: the free-time path
        assert tau_q_eff > 0.0, f"tau_q {tau_q_eff} must be positive (or None for the ideal quench)"

        # Rung 12 shares ONE τ_q-independent trajectory between the mean-field bulk and the
        # under-mixed core (both traverse the same β-path, differ only in τ). The mean-field-only
        # path (unmixedness=None) lets _quench_no build its own tab → byte-identical rung 10/11.
        tab = (_quench_trajectory(comp_p, T_p, alpha, far, Tt3, p, ngrid=quench_ngrid)
               if unmixedness is not None else None)
        # RUNG 20 — super_eq_o lifts the [O] INSIDE this re-making by m(T) along the cooling path
        # (default False ⇒ byte-identical rung 10/11). So `ei_no_quenched` (the mean-field bulk) and,
        # below, the rung-12 core / rung-16 per-pocket carry the same lift the rung-19 primary already
        # did — closing the "finite-quench fields ride on equilibrium O" lower-bound seam. The lift is
        # MODEST & PEAK-CONCENTRATED: the Zeldovich re-making peaks at the hottest stoich crossing where
        # m(T) is at its MINIMUM (~1.14), so the effective lift ≈ m(T_peak) — even smaller than the
        # rung-19 primary lift (docs/rung20-spec.md). The NO ceiling [NO]_e is a THERMODYNAMIC quantity,
        # untouched, so the rung-14/17 clamp DENOMINATOR is unchanged: only the numerator (and hence a) rises.
        q = _quench_no(comp_p, T_p, alpha, far, Tt3, p, n_no_total, tau_q_eff,
                       nsteps=quench_nsteps, ngrid=quench_ngrid, tab=tab, schedule=schedule,
                       super_eq_o=super_eq_o)
        state.tau_q = tau_q_eff
        state.mixing = mixing
        state.ei_no_quenched = q["ei"]                    # the MEAN-FIELD (rung-11) bulk EI
        state.x_no_quenched = q["x_no_mix"]
        state.T_peak = q["T_peak"]
        state.max_a_quench = q["max_a"]

        if pdf is not None:
            # RUNG 13 — the RESOLVED MIXING PDF (replaces rung-12's parameterised SEGREGATION with a
            # continuous distribution): integrate the IDEAL primary bell EI(φ) over a mean-preserving
            # β-PDF of mixture fraction whose single width is the segregation g(C)=min(g_max,
            # k_g·|ln(C/C_opt)|) — KINKED at the Holdeman optimum, so ⟨EI⟩(g(C)) collapses to the
            # well-mixed value AT C_opt (g=0 ⇒ delta ⇒ point value) with both flanks lifting: the
            # optimum LOCATION from a continuous distribution. Isolates the COMPOSITION mechanism
            # (drops the dwell chain) ⇒ it pins the optimum but does NOT climb — the far flank
            # descends (humped ⟨EI⟩(g)); the climb is rung-12's dwell (rung-15 combines them). A pure
            # diagnostic: NO/N still never enter _equil_solve, cycle bit-for-bit rung 6.
            C = pdf.C(mixing)
            g_seg = pdf.segregation(C)
            state.pdf = pdf
            state.C_holdeman = C
            state.g_seg = g_seg
            state.ei_no_pdf = _pdf_mean_ei(far, Tt3, p, hf_fuel, tau, g_seg,
                                           n_bell=pdf.n_bell, n_quad=pdf.n_quad)
            return state

        if pdf_quench is not None:
            # RUNG 15 — the PDF THROUGH the finite quench: carry rung-13's resolved β-PDF through the
            # rung-10/12 dwell chain, COMBINING the composition variance with the dwell. Additive:
            #   term 1 = ei_no_quenched  (the rung-11 mean-field bulk quench q["ei"] — the FINITE floor
            #            rung 13 lacked, present at all C);
            #   term 2 = D(u)·⟨EI_bell⟩(g)  (the rung-13 β-PDF integral over the IDEAL bell — reused
            #            verbatim — scaled by the off-optimum-growing dwell factor D(u)=τ_res(1+b_u·u)/τ,
            #            rescaling the reference-τ bell EI to the pocket's actual lingering dwell; EI ∝ τ,
            #            the dormant clamp). The bell reference IS the residence `tau`, so the two lock.
            # At C_opt (g→0) term 2 → 0 and ⟨EI⟩ = the FINITE bulk NO (NOT rung-13's ≈0); off-optimum the
            # NONLINEAR bell keeps the STOICH-MEAN SIGN REVERSAL (a lumped-dwell rung 12 cannot) and the
            # dwell growth makes the far flank CLIMB. A pure diagnostic: NO/N never enter _equil_solve, so
            # the cycle stays bit-for-bit rung 6.
            C = pdf_quench.C(mixing)
            g_seg = pdf_quench.segregation(C)
            bell_mean_ei = _pdf_mean_ei(far, Tt3, p, hf_fuel, tau, g_seg,
                                        n_bell=pdf_quench.n_bell, n_quad=pdf_quench.n_quad)
            term2 = pdf_quench.dwell_factor(C, tau) * bell_mean_ei
            state.pdf_quench = pdf_quench
            state.C_holdeman = C
            state.g_seg = g_seg
            state.ei_no_pdf_excess = term2
            state.ei_no_pdf_quench = q["ei"] + term2       # term1 (ei_no_quenched) + term2
            return state

        if pocket_quench is not None:
            # RUNG 16 — the PDF THROUGH the finite quench, PER POCKET (retires rung-15's linearised
            # dwell). Rung 15 scaled the CONSTANT-T ideal bell by a scalar D(u)=τ_core/τ_ref (exact only
            # while EI ∝ τ). Rung 16 carries EACH rich-of-mean β-PDF pocket through its OWN finite quench
            # (`_quench_no` at the dwell τ_core(C)), so the dwell acts INSIDE the cooling chemistry.
            # Additive, mirroring rung 15 — only term 2's internals change:
            #   term 1 = ei_no_quenched  (the rung-11 mean-field bulk quench q["ei"] — the finite floor,
            #            unchanged, present at all C);
            #   term 2 = ⟨EI_pocket_quench(ξ; τ_core(C))⟩_g  (the β-PDF integral of the PER-POCKET quench;
            #            lean-of-mean / φ>2 pockets reuse the rung-13 ideal bell — 0 above φ2 — since they
            #            never re-cross stoich). Because a lingering pocket COOLS, term 2 is SUBLINEAR in
            #            τ_core (vs rung-15's linear D(u)·EI) → the far-over-penetration flank ERODES into
            #            near-degeneracy with the C_opt notch. At C_opt (g→0) term 2 → the single lean
            #            pocket at ξ̄ ≈ 0 ⇒ ei_no_pocket_quench = the finite bulk floor.
            # A pure diagnostic: NO/N never enter _equil_solve, so the cycle stays bit-for-bit rung 6.
            C = pocket_quench.C(mixing)
            g_seg = pocket_quench.segregation(C)
            tau_core = pocket_quench.core_dwell(C)
            excess, pocket_max_a = _pocket_quench_mean_ei(
                far, Tt3, p, hf_fuel, tau, tau_core, g_seg,
                n_bell=pocket_quench.n_bell, n_quad=pocket_quench.n_quad,
                quench_ngrid=quench_ngrid, quench_nsteps=quench_nsteps,
                super_eq_o=super_eq_o)          # rung 20: lift each pocket's re-making (see the bulk q above)
            state.pocket_quench = pocket_quench
            state.C_holdeman = C
            state.g_seg = g_seg
            state.ei_no_pocket_excess = excess
            state.ei_no_pocket_quench = q["ei"] + excess   # term1 (ei_no_quenched) + term2
            state.max_a_quench = max(state.max_a_quench, pocket_max_a)   # dormancy gate spans pockets
            return state

        if transported is not None:
            # RUNG 18 — the TRANSPORTED-variance closure (docs/rung18-spec.md). The β-PDF width g(C) is
            # no longer the imposed kink but the residual of a variance DECAY ODE dg/dt=−C_φ·ω(C)·g
            # (`_transport_variance`) from a DERIVED two-stream ceiling (`_two_stream_ceiling`, from φ_p),
            # fed through the SAME rung-13 ideal bell (`_pdf_mean_ei`). What it adds vs rung 13's kink:
            #   • g_ceiling is DERIVED from φ_p (not the free g_max=0.3, ~4.4× too large);
            #   • g(C_opt)=g_ceiling·exp(−Da_opt) > 0 — a RESIDUAL floor (perfect mixing never reached),
            #     so the emissions optimum is ELEVATED off the well-mixed value (not the kink's ≈0);
            #   • g(C) is SMOOTH (both one-sided slopes →0 at C_opt) — the kink's sharpness was the artifact.
            # The C_opt LOCATION still rides on the IMPOSED spatial coverage ω(C) — a 0-D transport CANNOT
            # derive it (mean-field ω(J) ⇒ monotone g(J); the optimum needs the spatial S — the rung-11/12
            # variance seam). A pure diagnostic: NO/N never enter _equil_solve, cycle bit-for-bit rung 6.
            C = transported.C(mixing)
            g_seg, g_ceiling = transported.segregation(C, far, phi_primary)
            state.transported = transported
            state.C_holdeman = C
            state.g_ceiling = g_ceiling
            state.g_transported = g_seg
            state.g_seg = g_seg
            state.ei_no_transported = _pdf_mean_ei(far, Tt3, p, hf_fuel, tau, g_seg,
                                                   n_bell=transported.n_bell, n_quad=transported.n_quad)
            return state

        if unmixedness is None:
            return state                                  # rung 10/11 mean field — untouched

        # RUNG 12 — the under-mixed CORE (spatial variance): the SAME cooling trajectory, but the
        # core misses the jet and quenches at an ABSOLUTE residence τ_core(C)=τ_res·(1+b_u·u) (NOT
        # the vanishing jet time, so its NO penalty survives J→∞) that GROWS off-optimum. Mass-weight
        # bulk/core by the KINKED segregated fraction w(C)=k_u·|ln(C/C_opt)|, whose non-zero slope at
        # C_opt PINS the EI-min AT the Holdeman optimum (J_min=J_opt, shifting as (H/S)²) → EI_NO
        # turns back up. A pure diagnostic: NO/N still never enter _equil_solve, cycle bit-for-bit rung 6.
        C = unmixedness.C(mixing)
        w = unmixedness.core_fraction(C)
        qc = _quench_no(comp_p, T_p, alpha, far, Tt3, p, n_no_total, unmixedness.core_dwell(C),
                        nsteps=quench_nsteps, ngrid=quench_ngrid, tab=tab, schedule=schedule,
                        super_eq_o=super_eq_o)   # rung 20: lift the lingering core's re-making too
        state.unmixedness = unmixedness
        state.C_holdeman = C
        state.w_core = w
        state.ei_no_core = qc["ei"]                    # the lingering-core EI at τ_core(C)
        state.ei_no_unmixed = (1.0 - w) * q["ei"] + w * qc["ei"]   # ei_no_quenched (q) is the mean-field bulk
        state.max_a_quench = max(state.max_a_quench, qc["max_a"])   # dormancy gate spans BOTH streams
        return state

    # --- Rung-14 equilibrium-vs-frozen nozzle-flow diagnostic (DECOUPLED; never feeds the cycle) ---
    def nozzle_flow(self, far: float, Tt4: float, pt4: float,
                    Tt9: float, pt9: float, p9: float,
                    x_no_frozen: Optional[float] = None) -> "NozzleFlowState":
        """Frozen-vs-equilibrium nozzle-flow diagnostic (rung 14; docs/rung14-spec.md).

        The production nozzle FREEZES the station-4 equilibrium mixture through the whole expansion
        (rungs 6-13). This DIAGNOSTIC re-runs the hot expansion two ways from the SAME physical entry
        gas — the frozen station-4 mixture at the nozzle-entry TOTAL state (Tt9, pt9) — expanding
        isentropically to the back-pressure p9 (pass the run's Tt9=Tt5, pt9=π_n·pt5, p9=p_exit):

          FROZEN      — composition held at the station-4 mixture (== the production nozzle, exact).
          EQUILIBRIUM — composition RE-EQUILIBRATES at each (T, p): CO/H₂/OH/O/H recombine on cooling,
                        releasing chemical energy → a higher exit velocity (the UPPER bound).

        Returns the [V9_frozen, V9_equilibrium] thrust bracket (the real nozzle sits between; the gap
        is negligible at the cool lean design point and grows with combustor T) plus the dropped-clamp
        corollary: pass a frozen exhaust NO (`x_no_frozen`, e.g. the rung-8 zoned mole fraction) to get
        `max_a` — on the cooling path equilibrium NO collapses, so the frozen NO is super-equilibrium
        and rung 7's clamp would fire (max_a ≫ 1, vs rung 10's dormant 0.677).

        A pure diagnostic: it only READS (far, Tt4, pt4, Tt9, pt9, p9) and touches no cycle path, so
        the cycle stays bit-for-bit rung 6. Requires the equilibrium (rung-6) gas."""
        assert self.equilibrium, \
            "nozzle_flow: needs the rung-6 equilibrium gas (Gas.reacting_equilibrium())"
        assert p9 <= pt9 * (1.0 + 1e-12), \
            f"nozzle_flow: back-pressure p9={p9:.0f} Pa exceeds pt9={pt9:.0f} Pa (cannot expand to it)"
        comp_entry = _equilibrium_composition(far, Tt4, pt4)     # the FROZEN station-4 mixture

        T9f, V9f, _ = _expand_nozzle(comp_entry, far, Tt9, pt9, p9, shifting=False)
        T9e, V9e, comp9 = _expand_nozzle(comp_entry, far, Tt9, pt9, p9, shifting=True)

        nCO, nCO2 = comp_entry.get("CO", 0.0), comp_entry.get("CO2", 0.0)
        co_frac = nCO / (nCO + nCO2) if (nCO + nCO2) > 0.0 else 0.0

        clamp = _nozzle_clamp_diag(comp_entry, Tt9, T9f, x_no_frozen)   # clamp rides the FROZEN exit
        return NozzleFlowState(
            T9_frozen=T9f, T9_equilibrium=T9e, V9_frozen=V9f, V9_equilibrium=V9e,
            co_fraction_entry=co_frac, comp_entry=comp_entry, comp_exit_eq=comp9,
            x_no_e_entry=clamp["x_no_e_entry"], x_no_e_exit=clamp["x_no_e_exit"],
            no_collapse_ratio=clamp["no_collapse_ratio"],
            x_no_frozen=x_no_frozen, max_a=clamp["max_a"],
        )

    # --- Rung-17 combustor-mixing-fidelity ladder of the dropped-clamp margin (DECOUPLED) --------- #
    def exhaust_no_clamp(self, far: float, Tt3: float, Tt4: float, p: float,
                         Tt9: float, pt9: float, p9: float,
                         phi_primary: float, mixing: "JetMixing",
                         pocket_quench: "PocketQuenchPDF",
                         tau: float = 3e-3, super_eq_o: bool = False,
                         quench_ngrid: int = 240, quench_nsteps: int = 2000) -> "ExhaustNOxClampState":
        """Rung-17 combustor-mixing-fidelity ladder of the dropped-clamp margin (docs/rung17-spec.md).

        Composes rung 16 (the per-pocket exhaust NO) with rung 14 (the nozzle cooling collapse) at a
        RICH RQL primary. Carries THREE mixing-fidelity models of the exhaust NO — mixed-out (rung 8),
        mean-field bulk quench (rung 11), per-pocket β-PDF mean (rung 16) — through the SAME rung-14
        nozzle expansion to T9, and reads the dropped-clamp margin a=[NO]/[NO]_e(T9) for each.

        THE HEADLINE: at a rich primary the MIXED-OUT number reads DORMANT (a<1) — mixing-out HIDES the
        super-equilibrium exhaust NO — while the fuller models FIRE (a>1). This is the counterpoint to
        rung 14's φ_p=1.0 clamp corollary (which fires ON the mixed-out number, a≈250): the same lesson
        from the RICH side, where the crude mixed-out shortcut is deceptively low. The ORDERING
        a_mixed≤a_bulk≤a_pocket is STRUCTURAL (the quench only ADDS NO; the per-pocket excess is
        additive) and a_mixed<1 is robust — that is the certified claim. The FIRING (a_bulk,a_pocket>1)
        holds across the RQL J-band but is NOT universal: a fast enough quench (J→∞) drives
        x_no_quenched→x_no_mix (the rung-10 τ_q→0 reduce) so even a_bulk→a_mixed<1 (dormant). Every
        firing magnitude and the gap ride on un-pinned mixing scales (C_e, τ_res, H, J). The
        a_pocket/a_bulk ratio equals rung-16's station-4 gap by construction (the x_no_e(T9) denominator
        is common and cancels — a stated synthesis, not new physics).

        RUNG 20 — `super_eq_o=True` lifts the equilibrium-O LOWER BOUND on all three numerators (the
        mixed-out via the rung-19 primary [O] lift, the bulk/per-pocket via the rung-20 quench re-making
        lift). The shared denominator x_no_e(T9) is a thermodynamic ceiling, UNCHANGED, so every margin a
        RISES — this is the discharge of the rung-17 "every a is a lower bound" caveat. The lift is
        modest (~m(T_peak)≈1.14 through the quench), so the ORDERING and the in-band firing survive; the
        clamp still does NOT fire at station 4 (max_a_quench<1 — super-eq O is not the burner-clamp lever;
        a slow-enough freeze is, a separate seam). super_eq_o=False (default) ⇒ bit-for-bit rung 17.

        A pure diagnostic: it only READS state (through zoned_nox + nozzle_flow, both untouched) and
        feeds the cycle nothing, so the cycle stays bit-for-bit rung 6. Requires the equilibrium
        (rung-6) gas and both a `mixing` and a `pocket_quench` (the bulk and per-pocket rungs need the
        jet). Pass the run's Tt9=Tt5, pt9=π_n·pt5, p9=p_exit (same as nozzle_flow)."""
        assert self.equilibrium, \
            "exhaust_no_clamp: needs the rung-6 equilibrium gas (Gas.reacting_equilibrium())"
        assert mixing is not None and pocket_quench is not None, \
            "exhaust_no_clamp: needs both a JetMixing and a PocketQuenchPDF (the bulk + per-pocket rungs)"

        # The three exhaust-NO models — read straight off the rung-8/11/16 diagnostics, untouched.
        # RUNG 20 — super_eq_o lifts the equilibrium-O LOWER BOUND on all three numerators: the mixed-out
        # via the rung-19 primary [O] lift, the bulk/per-pocket via the rung-20 quench re-making lift. The
        # common denominator x_no_e(T9) is a THERMODYNAMIC ceiling (Kp_NO·√(x_N2·x_O2)), NOT set by the
        # O-atom closure, so it is UNCHANGED — every margin a rises, confirming rung 17's a were lower
        # bounds. The lift is bounded (~m(T_peak)≈1.14 on the quench), so the firing/ordering are robust.
        zn_mixed = self.zoned_nox(far, Tt3, Tt4, p, phi_primary, tau, super_eq_o=super_eq_o)  # rung 8
        zn_bulk = self.zoned_nox(far, Tt3, Tt4, p, phi_primary, tau, mixing=mixing, super_eq_o=super_eq_o,
                                 quench_ngrid=quench_ngrid, quench_nsteps=quench_nsteps)  # rung 11 (bulk quench)
        zn_pkt = self.zoned_nox(far, Tt3, Tt4, p, phi_primary, tau,
                                mixing=mixing, pocket_quench=pocket_quench, super_eq_o=super_eq_o,
                                quench_ngrid=quench_ngrid, quench_nsteps=quench_nsteps)   # rung 16 (per-pocket)
        x_no_mixed = zn_mixed.x_no_mix
        x_no_bulk = zn_bulk.x_no_quenched
        # x_no ∝ EI at fixed overall far (same n_tot and n_fuel per mol air), so κ = x_no/EI is COMMON to
        # the bulk and every pocket → the per-pocket β-PDF-mean mole fraction is κ·⟨EI⟩_pocket. That same
        # proportionality is exactly WHY the nozzle is a no-op on the pocket/bulk ratio: a_pocket/a_bulk =
        # ⟨EI⟩_pocket/EI_bulk = rung-16's station-4 gap. A STATED identity, not a gate — the x_no_e(T9)
        # denominator cancels by construction, so no computation could make it false (docs/rung17-spec.md).
        kappa = x_no_bulk / zn_bulk.ei_no_quenched
        x_no_pkt = kappa * zn_pkt.ei_no_pocket_quench

        # ONE rung-14 nozzle expansion → T9, the COMMON denominator x_no_e(T9), and the collapse ratio.
        nf = self.nozzle_flow(far, Tt4, p, Tt9, pt9, p9, x_no_frozen=x_no_bulk)
        xe = nf.x_no_e_exit                              # the common clamp denominator for all three

        return ExhaustNOxClampState(
            phi_primary=phi_primary, T9=nf.T9_frozen, x_no_e_exit=xe,
            no_collapse_ratio=nf.no_collapse_ratio,
            x_no_mixed_out=x_no_mixed, x_no_bulk_quench=x_no_bulk, x_no_pocket=x_no_pkt,
            a_mixed_out=x_no_mixed / xe, a_bulk_quench=nf.max_a, a_pocket=x_no_pkt / xe,
            ei_no_quenched=zn_bulk.ei_no_quenched,
            ei_no_pocket_quench=zn_pkt.ei_no_pocket_quench,
            gap_pocket_over_bulk=zn_pkt.ei_no_pocket_quench / zn_bulk.ei_no_quenched,
            max_a_quench=zn_pkt.max_a_quench,
        )

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
