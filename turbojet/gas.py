"""Working-fluid model and the flow state that travels through the engine.

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
# ~1241 @1240 K to ~1311 @1800 K); R_t ~= 287.1 falls out of the molar mass.
_PRODUCTS = {"N2": 150.74, "O2": 22.69, "Ar": 1.803, "CO2": 12.0, "H2O": 11.5}


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

    def cp(self, T: float) -> float:
        return self._cp                          # constant, by definition of CPG
    def h(self, T: float) -> float:
        return self._cp * T
    def pr(self, T: float) -> float:
        return T ** (1.0 / self.g)               # T^(cp/R) in the closed-form limit
    def T_from_h(self, h: float) -> float:
        return h / self._cp
    def T_from_pr(self, pr: float) -> float:
        return pr ** self.g
    def gamma_at(self, T: float) -> float:
        return self.gamma


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

    def cp(self, T: float) -> float:
        return self.R * _poly(self._A(T), T)

    def h(self, T: float) -> float:
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

    def pr(self, T: float) -> float:
        return math.exp(self._Phi(T))

    def T_from_h(self, h_target: float) -> float:
        T = _solve(self.h, self.cp, h_target)                # dh/dT = cp(T)
        # Round-trip inverse — a STANDING conservation assert (rung-3 gate 2).
        assert abs(self.h(T) - h_target) <= 1e-6 * abs(h_target) + 1e-3, "T_from_h round-trip"
        return T

    def T_from_pr(self, pr_target: float) -> float:
        target = math.log(pr_target)                         # solve Phi(T) = ln(pr)
        T = _solve(self._Phi, lambda t: _poly(self._A(t), t) / t, target)  # dPhi/dT=(cp/R)/T
        assert abs(self._Phi(T) - target) <= 1e-9, "T_from_pr round-trip"
        return T

    def gamma_at(self, T: float) -> float:
        cp = self.cp(T)
        return cp / (cp - self.R)                            # gamma = cp/(cp - R)


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

    def __post_init__(self):
        self._cold = (_TPGSection(self.cp_c_coeffs, self.R_c) if self.cp_c_coeffs
                      else _CPGSection(self.gamma_c, self.cp_c, self.R_c))
        self._hot = (_TPGSection(self.cp_t_coeffs, self.R_t) if self.cp_t_coeffs
                     else _CPGSection(self.gamma_t, self.cp_t, self.R_t))

    # --- classmethod factory: the thermally-perfect dual gas ------------------

    @classmethod
    def thermally_perfect(cls, hPR: float = 42.8e6) -> "Gas":
        """NASA-air cold section + lean-products hot section (rung 3)."""
        Alo_c, Ahi_c, R_c = _mixture(_AIR)
        Alo_t, Ahi_t, R_t = _mixture(_PRODUCTS)
        return cls(R_c=R_c, R_t=R_t, hPR=hPR,
                   cp_c_coeffs=(Alo_c, Ahi_c), cp_t_coeffs=(Alo_t, Ahi_t))

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
        return self.cp_t_coeffs is None

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

    def cp_t_at(self, T: float) -> float:
        return self._hot.cp(T)
    def h_t(self, T: float) -> float:
        return self._hot.h(T)
    def pr_t(self, T: float) -> float:
        return self._hot.pr(T)
    def T_from_h_t(self, h: float) -> float:
        return self._hot.T_from_h(h)
    def T_from_pr_t(self, pr: float) -> float:
        return self._hot.T_from_pr(pr)
    def gamma_t_at(self, T: float) -> float:
        return self._hot.gamma_at(T)

    def unified(self) -> "Gas":
        """Return a copy with the hot section collapsed onto the cold section.

        The lever for the reduce-to-ideal gate (docs/rung2-spec.md § Verification
        gates): collapsing the WHOLE section hot->cold — the (gamma, cp, R) triple
        AND any TPG cp(T) model — is what lets the machinery reproduce the rung-1
        table to the digit. If the hot section stayed different, its exponent would
        tilt the turbine and nozzle legs and the digits would drift.
        """
        return replace(self, gamma_t=self.gamma_c, cp_t=self.cp_c, R_t=self.R_c,
                       cp_t_coeffs=self.cp_c_coeffs)
