//! Working fluid: the flow state, and the dual-section gas.
//!
//! Port of `turbojet/gas.py` rungs 1-6 (phase 1 of docs/plans/todo-rust-port.md). The
//! derivations live in `docs/rung3-variable-cp.md`, `docs/rung4-reacting-products.md`,
//! `docs/rung5-fork-b.md` and `docs/rung6-spec.md`; what follows is the arithmetic and the
//! reasons a line is shaped the way it is.
//!
//! RUNG 3 -- variable cp(T): the thermally-perfect gas. Rungs 1-2 modelled each section as
//! *calorically* perfect (constant gamma, cp, R). Rung 3 lets cp vary with temperature, which
//! is the first rung where the isentropic power law Tt/Tt = (pt/pt)^g STOPS being exact, so
//! the cycle moves to the gas-table property functions:
//!
//! ```text
//! h(T)   = ∫_0^T cp dT'          enthalpy,          J/kg
//! phi(T) = ∫ cp/T dT'            entropy function,  J/(kg K)
//! pr(T)  = exp(phi(T)/R)         reduced pressure,  dimensionless
//! T_from_h, T_from_pr            the two inverses
//! ```
//!
//! For any process 1->2, ds = phi(T2)-phi(T1) - R ln(p2/p1). Set ds = 0 and it collapses to
//! p2/p1 = pr(T2)/pr(T1) -- every isentropic step in the cycle is ONE pr ratio.
//!
//! THE LOAD-BEARING DESIGN DECISION (rung3 spec § the trap). Rungs 1-2 reproduce their
//! tables to the digit using g = (gamma-1)/gamma with gamma = 1.4 and a rounded R = 287 that
//! is ~0.05 % off R = (gamma-1)/gamma * cp. The thermally-perfect pr = exp(phi/R) has the
//! constant-cp limit pr = T^(R/cp), exponent 287/1004 -- the same 0.05 % apart. So routing a
//! constant-cp gas through the integral path lands ~3e-4 off. Hence: a CALORICALLY-perfect
//! section keeps the rung-1/2 CLOSED FORMS exactly, a THERMALLY-perfect one INTEGRATES.
//!
//! ---------------------------------------------------------------------------------------
//! TWO PORTING RULES, both learned the expensive way (todo-rust-port.md § 1):
//!
//! 1. FLOAT MULTIPLICATION IS NOT ASSOCIATIVE. Python's `A[2] * T ** 2` is `A2 * (T*T)`.
//!    Written in Rust as `A[2] * T * T` it parses as `(A2 * T) * T` -- a DIFFERENT number.
//!    Every power is therefore precomputed into a named binding and multiplied once.
//! 2. NEVER `powf` FOR AN INTEGER POWER. LLVM strength-reduces `x.powf(0.5)` to `sqrt`,
//!    which differs in the last bit ~1 in 2500. Integer powers are explicit products.

use std::cell::RefCell;

// --------------------------------------------------------------------------------------
// Constants and the NASA 7-coefficient species data.
// Cp_molar(T)/Ru = a1 + a2 T + a3 T^2 + a4 T^3 + a5 T^4, two ranges joined at 1000 K.
// --------------------------------------------------------------------------------------

/// Universal gas constant, J/(mol K).
pub const RU: f64 = 8.314462618;
/// Low/high polynomial join, K.
pub const T_BREAK: f64 = 1000.0;

/// `(name, molar mass g/mol, A_low, A_high)`.
///
/// ORDER IS LOAD-BEARING for anything that sums over a mixture: Python iterates its dicts in
/// insertion order, and floating-point addition is not associative, so a reordered table
/// changes the last bits of every mole-weighted coefficient. Kept in `gas.py`'s order.
pub struct Species {
    pub name: &'static str,
    pub m: f64,
    pub a_low: [f64; 5],
    pub a_high: [f64; 5],
}

pub const SPECIES: &[Species] = &[
    Species { name: "N2", m: 28.0134,
        a_low: [3.298677, 1.4082404e-3, -3.963222e-6, 5.641515e-9, -2.444854e-12],
        a_high: [2.92664, 1.4879768e-3, -5.68476e-7, 1.0097038e-10, -6.753351e-15] },
    Species { name: "O2", m: 31.9988,
        a_low: [3.78245636, -2.99673416e-3, 9.84730201e-6, -9.68129509e-9, 3.24372837e-12],
        a_high: [3.28253784, 1.48308754e-3, -7.57966669e-7, 2.09470555e-10, -2.16717794e-14] },
    Species { name: "Ar", m: 39.948,
        a_low: [2.5, 0.0, 0.0, 0.0, 0.0],
        a_high: [2.5, 0.0, 0.0, 0.0, 0.0] },
    Species { name: "CO2", m: 44.0095,
        a_low: [2.35677352, 8.98459677e-3, -7.12356269e-6, 2.45919022e-9, -1.43699548e-13],
        a_high: [3.85746029, 4.41437026e-3, -2.21481404e-6, 5.23490188e-10, -4.72084164e-14] },
    Species { name: "H2O", m: 18.01528,
        a_low: [4.19864056, -2.03643410e-3, 6.52040211e-6, -5.48797062e-9, 1.77197817e-12],
        a_high: [3.03399249, 2.17691804e-3, -1.64072518e-7, -9.70419870e-11, 1.68200992e-14] },
    // Rung 6 -- the five dissociation species. Same GRI-Mech 3.0 source; unused by rungs 1-5.
    Species { name: "CO", m: 28.0101,
        a_low: [3.57953347, -6.10353680e-4, 1.01681433e-6, 9.07005884e-10, -9.04424499e-13],
        a_high: [2.71518561, 2.06252743e-3, -9.98825771e-7, 2.30053008e-10, -2.03647716e-14] },
    Species { name: "H2", m: 2.01588,
        a_low: [2.34433112, 7.98052075e-3, -1.94781510e-5, 2.01572094e-8, -7.37611761e-12],
        a_high: [3.33727920, -4.94024731e-5, 4.99456778e-7, -1.79566394e-10, 2.00255376e-14] },
    Species { name: "OH", m: 17.00734,
        a_low: [3.99201543, -2.40131752e-3, 4.61793841e-6, -3.88113333e-9, 1.36411470e-12],
        a_high: [3.09288767, 5.48429716e-4, 1.26505228e-7, -8.79461556e-11, 1.17412376e-14] },
    Species { name: "O", m: 15.9994,
        a_low: [3.16826710, -3.27931884e-3, 6.64306396e-6, -6.12806624e-9, 2.11265971e-12],
        a_high: [2.56942078, -8.59741137e-5, 4.19484589e-8, -1.00177799e-11, 1.22833691e-15] },
    Species { name: "H", m: 1.00794,
        a_low: [2.50000000, 7.05332819e-13, -1.99591964e-15, 2.30081632e-18, -9.27732332e-22],
        a_high: [2.50000001, -2.30842973e-11, 1.61561948e-14, -4.73515235e-18, 4.98197357e-22] },
    // Rung 7 -- thermal NOx. Inert to rungs 1-6.
    Species { name: "NO", m: 30.0061,
        a_low: [4.21859896, -4.63988124e-3, 1.10443049e-5, -9.34055507e-9, 2.80554874e-12],
        a_high: [3.26071234, 1.19101135e-3, -4.29122646e-7, 6.94481463e-11, -4.03295681e-15] },
    Species { name: "N", m: 14.0067,
        a_low: [2.50000000, 0.0, 0.0, 0.0, 0.0],
        a_high: [2.41594290, 1.74890650e-4, -1.19023690e-7, 3.02262450e-11, -2.03609820e-15] },
];

pub fn species(name: &str) -> &'static Species {
    SPECIES.iter().find(|s| s.name == name).expect("unknown species")
}

/// Cold section: dry air mole fractions. Sums to 0.9996, NOT 1 -- see [`air_mole_fractions`].
pub const AIR: &[(&str, f64)] = &[("N2", 0.7808), ("O2", 0.2095), ("Ar", 0.0093)];

/// Hot section (rung 3 only): a FIXED lean-products mixture, per mole of fuel for a Jet-A
/// surrogate burned at far ~= 0.030. Unanchored by design and stated plainly; rung 4's
/// reacting gas computes the composition from f instead.
pub const PRODUCTS: &[(&str, f64)] =
    &[("N2", 150.74), ("O2", 22.69), ("Ar", 1.803), ("CO2", 12.0), ("H2O", 11.5)];

/// (CH2)n repeat-unit molar mass, g/mol.
pub const M_CH2: f64 = 12.011 + 2.0 * 1.008;

/// Dry-air mole fractions renormalised to sum EXACTLY 1.
///
/// Load-bearing: [`AIR`] sums to 0.9996, and the stoichiometry is "per 1 mol of dry air", so
/// the un-normalised base drifts the product fractions off the anchor's values.
pub fn air_mole_fractions() -> Vec<(&'static str, f64)> {
    let xsum: f64 = AIR.iter().map(|&(_, v)| v).sum();
    AIR.iter().map(|&(s, v)| (s, v / xsum)).collect()
}

/// Mean molar mass of dry air, g/mol (~28.96), from the normalised fractions.
pub fn m_air() -> f64 {
    air_mole_fractions().iter().map(|&(s, x)| x * species(s).m).sum()
}

/// Stoichiometric fuel/air ratio: the f at which excess O2 hits zero (~0.0677).
pub fn f_stoich() -> f64 {
    let x_o2 = air_mole_fractions().iter().find(|&&(s, _)| s == "O2").unwrap().1;
    (x_o2 / 1.5) * M_CH2 / m_air()
}

// --------------------------------------------------------------------------------------
// RUNG 5 -- Fork B: formation-enthalpy bookkeeping.
//
// Rung 4 stayed Fork A (fixed hPR, sensible h(0)=0 datum). Fork B carries each species'
// standard formation enthalpy so the burner's heat release is DERIVED from an absolute-
// enthalpy balance instead of assumed. The formation constant is exactly the NASA-7 a6 term
// that rungs 3/4 dropped; since `antideriv_h` is the polynomial part with NO constant,
// `Ru * a6` is the additive formation offset. It CANCELS in every enthalpy DIFFERENCE
// (turbine, nozzle), so only the burner's cross-section subtraction ever sees it.
// --------------------------------------------------------------------------------------

pub const T_REF: f64 = 298.15;
/// Standard-state pressure, 1 bar -- the `(p/p0)^dnu` factor in Kp.
pub const P_REF: f64 = 100000.0;

/// Standard molar enthalpies of formation at 298.15 K, J/mol (CODATA/JANAF). Elements are
/// the reference datum (0); H2O is GAS (vapour), hence LHV rather than HHV.
pub const HF298: &[(&str, f64)] = &[
    ("N2", 0.0), ("O2", 0.0), ("Ar", 0.0), ("CO2", -393520.0), ("H2O", -241826.0),
    ("CO", -110527.0), ("H2", 0.0), ("OH", 38987.0), ("O", 249180.0), ("H", 217998.0),
    // ΔHf°(NO) carries a real ~1 kJ/mol literature spread; the K-check confirms JANAF.
    ("NO", 90291.0), ("N", 472680.0),
];

/// Standard molar entropies at 298.15 K, J/(mol K). The twin of [`HF298`]: rung 5 derives the
/// formation constant a6 from HF298, rung 6 derives the absolute-entropy constant a7 from
/// this, so `g0 = h0 - T s0 -> Kp`. Consumed ONLY by the equilibrium solve -- the additive a7
/// cancels in every pr ratio, exactly as a6 cancels in enthalpy differences.
pub const S298: &[(&str, f64)] = &[
    ("N2", 191.609), ("O2", 205.152), ("Ar", 154.846), ("CO2", 213.785), ("H2O", 188.835),
    ("CO", 197.660), ("H2", 130.680), ("OH", 183.708), ("O", 161.058), ("H", 114.716),
    ("NO", 210.758), ("N", 153.30),
];

pub fn hf298(name: &str) -> f64 {
    HF298.iter().find(|&&(s, _)| s == name).expect("no ΔHf for species").1
}
pub fn s298(name: &str) -> f64 {
    S298.iter().find(|&&(s, _)| s == name).expect("no S° for species").1
}

/// (CH2)n repeat-unit molar mass, kg/mol.
pub const M_CH2_KG: f64 = M_CH2 / 1000.0;
/// Mattingly's assumed heating value, J/kg -- the ONE calibration input's anchor.
pub const HPR_MATTINGLY: f64 = 42.8e6;

/// Fuel ΔHf298, J/mol, pinned so the DERIVED LHV equals Mattingly's assumed 42.8 MJ/kg.
pub fn hf_fuel_default() -> f64 {
    HPR_MATTINGLY * M_CH2_KG + hf298("CO2") + hf298("H2O")
}

/// Derived lower heating value, J/kg. `CH2 + 1.5 O2 -> CO2 + H2O(gas)`:
/// `LHV = (ΔHf(CH2) − ΔHf(CO2) − ΔHf(H2O,gas)) / M_CH2` (reactants − products, O2 = 0).
pub fn lhv_from_fuel(hf_fuel_molar: f64) -> f64 {
    (hf_fuel_molar - hf298("CO2") - hf298("H2O")) / M_CH2_KG
}

/// Mass-specific formation enthalpy of the lean-combustion products at f, J/kg.
///
/// `hf_prod = Σ nᵢ ΔHf,i / Σ nᵢ Mᵢ`. Only CO2 and H2O carry formation (N2/O2/Ar are elements).
/// This is the additive offset the burner adds to the SENSIBLE h_t to get the absolute one.
pub fn formation_products_mass(f: f64) -> f64 {
    let comp = products_composition(f);
    let mut h = 0.0f64; // J per mol-air basis
    let mut m = 0.0f64; // kg per mol-air basis
    for &(s, n) in &comp {
        h += n * hf298(s);
        m += n * species(s).m / 1000.0;
    }
    h / m
}

// --------------------------------------------------------------------------------------
// Composition and mole-weighting.
// --------------------------------------------------------------------------------------

/// Lean-complete-combustion product mole numbers per 1 mol dry air, from f.
///
/// `CH2 + 1.5 O2 -> CO2 + H2O`, dry-air oxidiser. `n_fuel = f * M_air / M_CH2` is the mol of
/// (CH2) burned per mol of air. N2 and Ar are inert and pass through; each mol of fuel makes
/// one CO2 and one H2O and consumes 1.5 O2. Returns UNNORMALISED mole numbers.
///
/// The return ORDER (N2, Ar, CO2, H2O, O2) reproduces the Python dict's insertion order,
/// because [`mixture`] sums over it and float addition is not associative.
pub fn products_composition(f: f64) -> Vec<(&'static str, f64)> {
    let x = air_mole_fractions();
    let xg = |name: &str| x.iter().find(|&&(s, _)| s == name).unwrap().1;
    let n_fuel = f * m_air() / M_CH2;
    let comp = vec![
        ("N2", xg("N2")),
        ("Ar", xg("Ar")),
        ("CO2", n_fuel),
        ("H2O", n_fuel),
        ("O2", xg("O2") - 1.5 * n_fuel),
    ];
    let o2 = comp[4].1;
    // LEAN GUARD (rung-4 conservation assert): rich f is out of scope and must TRIP, not
    // silently produce a negative O2 mole number.
    assert!(o2 > 0.0,
        "rich mixture f={f:.4} >= f_stoich={:.4}: excess O2 <= 0 (rich combustion / \
         dissociation is rung 5, out of rung-4 scope)", f_stoich());
    // ATOM CONSERVATION: C, H, O balance, built once per composition.
    assert!((comp[2].1 - n_fuel).abs() < 1e-12, "C balance");
    assert!((2.0 * comp[3].1 - 2.0 * n_fuel).abs() < 1e-12, "H balance");
    let o_in = 2.0 * xg("O2");
    let o_out = 2.0 * comp[2].1 + comp[3].1 + 2.0 * comp[4].1;
    assert!((o_out - o_in).abs() < 1e-12, "O balance");
    comp
}

/// Mole-weight a species mixture into `(A_low, A_high, R)`.
///
/// `cp_mass(T) = R Σ x_i (Cp_i/Ru)(T)` with `R = Ru/M_mix`, so the mixture's effective Cp/R
/// coefficients are just the mole-weighted species coefficients -- cp is linear in them.
pub fn mixture(fractions: &[(&str, f64)]) -> ([f64; 5], [f64; 5], f64) {
    let xsum: f64 = fractions.iter().map(|&(_, v)| v).sum();
    let x: Vec<(&str, f64)> = fractions.iter().map(|&(s, v)| (s, v / xsum)).collect();
    let m: f64 = x.iter().map(|&(s, xi)| xi * species(s).m).sum();
    let r = RU / (m / 1000.0);
    let mut a_low = [0.0f64; 5];
    let mut a_high = [0.0f64; 5];
    for k in 0..5 {
        // Summed in table order, one accumulator, to match the Python generator expression.
        let mut lo = 0.0f64;
        let mut hi = 0.0f64;
        for &(s, xi) in &x {
            lo += xi * species(s).a_low[k];
            hi += xi * species(s).a_high[k];
        }
        a_low[k] = lo;
        a_high[k] = hi;
    }
    (a_low, a_high, r)
}

// --------------------------------------------------------------------------------------
// The NASA polynomial and its two antiderivatives.
//
// Every power is bound to a name and multiplied ONCE -- see porting rule 1 in the module
// header. `A[2] * T * T` would be `(A2*T)*T`, which is not what Python computes.
// --------------------------------------------------------------------------------------

/// `cp(T)/R = A1 + A2 T + A3 T^2 + A4 T^3 + A5 T^4`.
pub fn poly(a: &[f64; 5], t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    a[0] + a[1] * t + a[2] * t2 + a[3] * t3 + a[4] * t4
}

/// `∫_0^T (cp/R) dT'` -- the enthalpy antiderivative through the origin.
///
/// Datum h(0) = 0, zero integration constant. Load-bearing: the burner is the one place
/// enthalpy crosses the cold->hot boundary, so the relative datum does NOT cancel for a dual
/// gas. Datum-0 makes a flat-cp thermally-perfect section reduce to EXACTLY cp*T.
pub fn antideriv_h(a: &[f64; 5], t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    a[0] * t + a[1] * t2 / 2.0 + a[2] * t3 / 3.0 + a[3] * t4 / 4.0 + a[4] * t5 / 5.0
}

/// `∫ (cp/R)/T' dT' = phi(T)/R`. Datum arbitrary -- it cancels in every pr ratio.
pub fn antideriv_phi(a: &[f64; 5], t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    a[0] * t.ln() + a[1] * t + a[2] * t2 / 2.0 + a[3] * t3 / 3.0 + a[4] * t4 / 4.0
}

/// Invert a monotone-increasing `f` to `f(T) = target`: safeguarded Newton.
///
/// `cp(T) > 0` makes both h and pr strictly increasing, so the root is unique and bracketed;
/// a Newton step that leaves the bracket falls back to bisection.
///
/// **This function's `tol` dominates the whole gas layer's reproducibility.** Measured across
/// the two interpreters the project already ships on, forward arithmetic agrees to <= 2e-14
/// while these inverses spread to 9.9e-12 -- three orders of magnitude more, and set entirely
/// by the stopping rule below, not by arithmetic. Ported iterate-for-iterate for that reason.
pub fn solve<F, G>(f: F, fprime: G, target: f64, lo0: f64, hi0: f64, tol: f64) -> f64
where
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    let (mut lo, mut hi) = (lo0, hi0);
    assert!(f(lo) - target <= 0.0 && 0.0 <= f(hi) - target, "inverse: root not bracketed");
    let mut x = 0.5 * (lo + hi);
    for _ in 0..100 {
        let fx = f(x) - target;
        if fx > 0.0 { hi = x; } else { lo = x; }
        let dfx = fprime(x);
        let mut xn = if dfx > 0.0 { x - fx / dfx } else { 0.5 * (lo + hi) };
        if !(lo < xn && xn < hi) {
            xn = 0.5 * (lo + hi); // bisection fallback
        }
        if (xn - x).abs() <= tol * x {
            return xn;
        }
        x = xn;
    }
    x
}

/// The default bracket and tolerance of `gas.py`'s `_solve`.
pub const SOLVE_LO: f64 = 150.0;
pub const SOLVE_HI: f64 = 4000.0;
pub const SOLVE_TOL: f64 = 1e-11;

// --------------------------------------------------------------------------------------
// Sections. Each answers the same seven questions; only `Reacting` consults `far`.
// --------------------------------------------------------------------------------------

/// A calorically-perfect section: constant (gamma, cp, R), closed-form math.
///
/// Keeps the rung-1/2 forms bit-for-bit (`h = cp T`, `pr = T^(1/g)`), so reduce-to-ideal
/// reproduces the prior tables to the digit.
#[derive(Clone, Copy, Debug)]
pub struct CpgSection {
    pub gamma: f64,
    pub cp_const: f64,
    pub r: f64,
    /// Isentropic exponent `(gamma-1)/gamma`.
    pub g: f64,
}

impl CpgSection {
    pub fn new(gamma: f64, cp: f64, r: f64) -> Self {
        CpgSection { gamma, cp_const: cp, r, g: (gamma - 1.0) / gamma }
    }
    pub fn cp(&self, _t: f64) -> f64 { self.cp_const }
    pub fn h(&self, t: f64) -> f64 { self.cp_const * t }
    /// `T^(1/g)` -- i.e. `T^(cp/R)` in the closed-form limit.
    pub fn pr(&self, t: f64) -> f64 { t.powf(1.0 / self.g) }
    pub fn t_from_h(&self, h: f64) -> f64 { h / self.cp_const }
    pub fn t_from_pr(&self, pr: f64) -> f64 { pr.powf(self.g) }
    pub fn gamma_at(&self, _t: f64) -> f64 { self.gamma }
    pub fn r_at(&self) -> f64 { self.r }
}

/// A thermally-perfect section: cp(T) from a mole-weighted NASA polynomial.
///
/// h and phi are analytic and piecewise across the 1000 K join, so they are continuous by
/// construction; the inverses are numerical. R is constant.
#[derive(Clone, Copy, Debug)]
pub struct TpgSection {
    pub a_low: [f64; 5],
    pub a_high: [f64; 5],
    pub r: f64,
}

impl TpgSection {
    pub fn new(a_low: [f64; 5], a_high: [f64; 5], r: f64) -> Self {
        TpgSection { a_low, a_high, r }
    }

    fn a(&self, t: f64) -> &[f64; 5] {
        if t <= T_BREAK { &self.a_low } else { &self.a_high }
    }

    pub fn cp(&self, t: f64) -> f64 {
        self.r * poly(self.a(t), t)
    }

    /// `∫_0^T cp dT'`, datum h(0) = 0, continuous across the 1000 K join.
    pub fn h(&self, t: f64) -> f64 {
        if t <= T_BREAK {
            return self.r * antideriv_h(&self.a_low, t);
        }
        let h_break = antideriv_h(&self.a_low, T_BREAK);
        self.r * (h_break + antideriv_h(&self.a_high, t) - antideriv_h(&self.a_high, T_BREAK))
    }

    /// `phi(T)/R`, continuous across the join (datum arbitrary).
    pub fn phi(&self, t: f64) -> f64 {
        if t <= T_BREAK {
            return antideriv_phi(&self.a_low, t);
        }
        let p_break = antideriv_phi(&self.a_low, T_BREAK);
        p_break + antideriv_phi(&self.a_high, t) - antideriv_phi(&self.a_high, T_BREAK)
    }

    pub fn pr(&self, t: f64) -> f64 { self.phi(t).exp() }

    pub fn t_from_h(&self, h_target: f64) -> f64 {
        // dh/dT = cp(T)
        let t = solve(|x| self.h(x), |x| self.cp(x), h_target, SOLVE_LO, SOLVE_HI, SOLVE_TOL);
        // Round-trip inverse -- a STANDING conservation assert (rung-3 gate 2).
        assert!((self.h(t) - h_target).abs() <= 1e-6 * h_target.abs() + 1e-3,
                "T_from_h round-trip");
        t
    }

    pub fn t_from_pr(&self, pr_target: f64) -> f64 {
        let target = pr_target.ln(); // solve Phi(T) = ln(pr)
        // dPhi/dT = (cp/R)/T
        let t = solve(|x| self.phi(x), |x| poly(self.a(x), x) / x,
                      target, SOLVE_LO, SOLVE_HI, SOLVE_TOL);
        assert!((self.phi(t) - target).abs() <= 1e-9, "T_from_pr round-trip");
        t
    }

    /// `gamma = cp/(cp - R)`.
    pub fn gamma_at(&self, t: f64) -> f64 {
        let cp = self.cp(t);
        cp / (cp - self.r)
    }

    pub fn r_at(&self) -> f64 { self.r }
}

/// A hot section whose composition -- and thus cp(T), R, gamma(T) -- tracks f (rung 4).
///
/// Each distinct f defines a lean-combustion product mixture; mole-weighting it yields the
/// very same `(A_low, A_high, R)` a frozen [`TpgSection`] takes, so this builds -- and
/// MEMOISES -- one section per f and delegates. The integral machinery, the guarded-Newton
/// inverses and their standing round-trip asserts are inherited unchanged.
///
/// Memoisation: the burner's fixed point evaluates at a few nearby f while it converges, then
/// the whole cycle downstream calls at ONE fixed f. Keyed on the exact bit pattern of f (which
/// is what `FlowState.far` carries and threads verbatim), so it is a memo cache and not hidden
/// state -- the same f always maps to the same section.
#[derive(Debug, Default)]
pub struct ReactingSection {
    cache: RefCell<Vec<(u64, TpgSection)>>,
}

impl Clone for ReactingSection {
    /// A clone starts with an EMPTY cache on purpose: the cache is a pure function of f, so
    /// copying it would be correct but pointless, and an empty one keeps `Clone` cheap.
    fn clone(&self) -> Self { ReactingSection::default() }
}

impl ReactingSection {
    pub fn new() -> Self { Self::default() }

    pub fn section_for(&self, far: f64) -> TpgSection {
        let key = far.to_bits();
        if let Some(&(_, sec)) = self.cache.borrow().iter().find(|&&(k, _)| k == key) {
            return sec;
        }
        let (a_low, a_high, r) = mixture(&products_composition(far));
        let sec = TpgSection::new(a_low, a_high, r);
        self.cache.borrow_mut().push((key, sec));
        sec
    }

    pub fn cp(&self, t: f64, far: f64) -> f64 { self.section_for(far).cp(t) }
    pub fn h(&self, t: f64, far: f64) -> f64 { self.section_for(far).h(t) }
    pub fn pr(&self, t: f64, far: f64) -> f64 { self.section_for(far).pr(t) }
    pub fn t_from_h(&self, h: f64, far: f64) -> f64 { self.section_for(far).t_from_h(h) }
    pub fn t_from_pr(&self, pr: f64, far: f64) -> f64 { self.section_for(far).t_from_pr(pr) }
    pub fn gamma_at(&self, t: f64, far: f64) -> f64 { self.section_for(far).gamma_at(t) }
    pub fn r_at(&self, far: f64) -> f64 { self.section_for(far).r }
}

// --------------------------------------------------------------------------------------
// RUNG 6 — high-temperature dissociation + chemical equilibrium.
//
// The complete-combustion `products_composition(f)` is replaced (for the equilibrium gas)
// by a T,p-coupled solve: 3 element balances (C, H, O) + 5 reaction Kp relations for the 8
// reacting mole numbers. Kp needs g0 = h0 − T s0 on absolute enthalpy (a6) AND absolute
// entropy (a7).
//
// THE DATUM RULE, and it is a requirement rather than a taste: the Kp solve uses SCALE A
// (a6-at-298.15, formation) or the reaction ΔG° is simply wrong; the cycle burner's energy
// balance uses SCALE B (0 K sensible + formation) so it reduces to Fork B exactly. Only the
// datum-free composition — the mole numbers — crosses between them.
// --------------------------------------------------------------------------------------

/// The 8 unknowns. N2 and Ar are inert and carried separately.
///
/// ORDER IS LOAD-BEARING: it fixes the residual vector, the Jacobian columns and — through
/// [`equilibrium_composition`] — the order [`mixture`] later sums in.
pub const SP_REACT: [&str; 8] = ["CO2", "H2O", "CO", "H2", "OH", "O", "H", "O2"];

/// Atom counts (C, H, O) per species, in [`SP_REACT`] order.
pub const ELEM: [[f64; 3]; 8] = [
    [1.0, 0.0, 2.0], // CO2
    [0.0, 2.0, 1.0], // H2O
    [1.0, 0.0, 1.0], // CO
    [0.0, 2.0, 0.0], // H2
    [0.0, 1.0, 1.0], // OH
    [0.0, 0.0, 1.0], // O
    [0.0, 1.0, 0.0], // H
    [0.0, 0.0, 2.0], // O2
];

/// The five basis dissociation reactions (products positive), as `(species index, nu)`:
///
/// ```text
/// CO2 -> CO + 1/2 O2      H2O -> H2 + 1/2 O2      H2O -> OH + 1/2 H2
/// 1/2 O2 -> O             1/2 H2 -> H
/// ```
///
/// Held as ordered slices, not maps: Python iterates its dicts in insertion order and float
/// addition is not associative, so the term order is part of the arithmetic.
pub const REACTIONS: [&[(usize, f64)]; 5] = [
    &[(0, -1.0), (2, 1.0), (7, 0.5)],
    &[(1, -1.0), (3, 1.0), (7, 0.5)],
    &[(1, -1.0), (4, 1.0), (3, 0.5)],
    &[(7, -0.5), (5, 1.0)],
    &[(3, -0.5), (6, 1.0)],
];

/// `∫_0^T (cp/Ru) dT'` for one species (dimensionless), across the 1000 K join.
pub fn sens_h(sp: &str, t: f64) -> f64 {
    let s = species(sp);
    if t <= T_BREAK {
        return antideriv_h(&s.a_low, t);
    }
    antideriv_h(&s.a_low, T_BREAK) + antideriv_h(&s.a_high, t) - antideriv_h(&s.a_high, T_BREAK)
}

/// `∫ (cp/Ru)/T' dT'` for one species (dimensionless), across the join.
pub fn sens_phi(sp: &str, t: f64) -> f64 {
    let s = species(sp);
    if t <= T_BREAK {
        return antideriv_phi(&s.a_low, t);
    }
    antideriv_phi(&s.a_low, T_BREAK) + antideriv_phi(&s.a_high, t)
        - antideriv_phi(&s.a_high, T_BREAK)
}

/// Formation constant: `H(298.15) = ΔHf` ⇒ `a6 = ΔHf/Ru − antideriv_h(A_low, 298.15)`.
pub fn a6_of(sp: &str) -> f64 {
    hf298(sp) / RU - antideriv_h(&species(sp).a_low, T_REF)
}

/// Absolute-entropy constant: `S(298.15) = S298` ⇒ `a7 = S298/Ru − antideriv_phi(A_low, 298.15)`.
pub fn a7_of(sp: &str) -> f64 {
    s298(sp) / RU - antideriv_phi(&species(sp).a_low, T_REF)
}

/// SCALE A absolute molar enthalpy (a6-at-298.15, formation), J/mol. Kp and AFT only.
pub fn h_molar_a(sp: &str, t: f64) -> f64 {
    RU * (sens_h(sp, t) + a6_of(sp))
}

/// Absolute standard-state molar entropy `s0(T)` at p0 = 1 bar, J/(mol K). Kp only.
pub fn s_molar(sp: &str, t: f64) -> f64 {
    RU * (sens_phi(sp, t) + a7_of(sp))
}

/// Absolute standard-state Gibbs energy `g0(T) = h0 − T s0`, J/mol (scale A). Kp only.
pub fn g_molar(sp: &str, t: f64) -> f64 {
    h_molar_a(sp, t) - t * s_molar(sp, t)
}

/// SCALE B absolute molar enthalpy (0 K sensible + formation), J/mol.
///
/// The burner's ENERGY-balance datum — it matches production Fork B, so the cycle reduces
/// to it exactly.
pub fn h_molar_b(sp: &str, t: f64) -> f64 {
    RU * sens_h(sp, t) + hf298(sp)
}

/// `ln Kp(T) = −ΔG°(T)/(Ru T)`, with `ΔG° = Σ nu g0` (scale A, a datum-free reaction constant).
pub fn ln_kp(rxn: &[(usize, f64)], t: f64) -> f64 {
    let mut dg0 = 0.0f64;
    for &(i, nu) in rxn {
        dg0 += nu * g_molar(SP_REACT[i], t);
    }
    -dg0 / (RU * t)
}

/// Solve `A x = b` by Gauss-Jordan elimination with partial pivoting (small dense system).
///
/// The pivot search keeps Python's FIRST-maximum tie-break. Rust's `max_by` returns the LAST
/// maximum, which would pick a different row on a tie and take the Newton down a different
/// (still convergent, but not bit-identical) path.
pub fn gauss_solve(a: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
    let n = a.len();
    let mut m: Vec<Vec<f64>> = a
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let mut r = row.clone();
            r.push(b[i]);
            r
        })
        .collect();
    for c in 0..n {
        let mut piv_row = c;
        let mut best = m[c][c].abs();
        for r in (c + 1)..n {
            if m[r][c].abs() > best {
                best = m[r][c].abs();
                piv_row = r;
            }
        }
        m.swap(c, piv_row);
        let piv = m[c][c];
        let prow = m[c].clone();
        for r in 0..n {
            if r != c && m[r][c] != 0.0 {
                let fac = m[r][c] / piv;
                for k in c..=n {
                    m[r][k] -= fac * prow[k];
                }
            }
        }
    }
    (0..n).map(|i| m[i][n] / m[i][i]).collect()
}

/// Core equilibrium solve: mole numbers of the 8 reacting species at `(T, p)`, given C/H/O
/// atom totals and inert moles.
///
/// Damped Newton in `y = ln n` (which keeps `n > 0`), seeded from complete combustion; 3
/// element balances plus 5 reaction Kp equations. Reaction `r` reads
///
/// ```text
/// Σ nu_ri (y_i − ln n_tot) + dnu_r ln(p/p0) − lnKp_r = 0
/// ```
///
/// with `n_tot` INCLUDING the inert species, so the mole fractions `x_i = n_i/n_tot` are
/// right. Basis-agnostic (any `C_bC H_bH`), so the tests reuse it for the methane anchor.
///
/// RUNG 9 — the seed BRANCHES on the O-balance sign. The 8-species system is complete lean
/// OR rich; only the SEED must know which side it is on. The lean branch keeps the
/// byte-identical rung-6 expression, so every rung-1..8 path — all lean — takes an unchanged
/// Newton trajectory and reduce-to-rung-8 is bit-for-bit by construction.
pub fn equil_solve(
    b_c: f64, b_h: f64, b_o: f64, n_inert: f64, t: f64, p: f64,
) -> [f64; 8] {
    // Seed, in SP_REACT order: CO2, H2O, CO, H2, OH, O, H, O2.
    let seed: [f64; 8] = if b_o >= 2.0 * b_c + b_h / 2.0 {
        // LEAN — byte-identical to the rung-6 seed (C->CO2, H->H2O, leftover O2; radicals
        // tiny). Untouched so the whole lean cycle keeps its exact Newton path.
        [
            b_c.max(1e-12),
            (b_h / 2.0).max(1e-12),
            1e-8,
            1e-8,
            1e-8,
            1e-9,
            1e-9,
            ((b_o - 2.0 * b_c - b_h / 2.0) / 2.0).max(1e-8),
        ]
    } else {
        // RICH (rung 9) — O-limited allocation, atom-conserving: water first, then all
        // C->CO, upgrade CO->CO2 with the O left over; any H beyond the O supply stays H2.
        // The lean seed's O2 goes negative when rich and grossly violates the O balance, so
        // damped Newton is fragile there; this one converges cleanly to the soot bound.
        let n_h2o = (b_h / 2.0).min(b_o);
        let o_left = b_o - n_h2o;
        let n_co2 = b_c.min((o_left - b_c).max(0.0));
        [
            n_co2.max(1e-12),
            n_h2o.max(1e-12),
            (b_c - n_co2).max(1e-12),
            (b_h / 2.0 - n_h2o).max(1e-12),
            1e-8,
            1e-9,
            1e-9,
            1e-8,
        ]
    };

    let mut y: [f64; 8] = [0.0; 8];
    for j in 0..8 {
        y[j] = seed[j].ln();
    }
    let lnkp: Vec<f64> = REACTIONS.iter().map(|r| ln_kp(r, t)).collect();
    let lnpr = (p / P_REF).ln();

    let mut converged = false;
    for _ in 0..200 {
        let mut nv = [0.0f64; 8];
        for j in 0..8 {
            nv[j] = y[j].exp();
        }
        let mut ntot = 0.0f64;
        for j in 0..8 {
            ntot += nv[j];
        }
        ntot += n_inert;
        let ln_ntot = ntot.ln();

        // Residuals: 3 element balances, then the 5 reaction relations.
        let mut f = [0.0f64; 8];
        for (k, b) in [b_c, b_h, b_o].iter().enumerate() {
            let mut s = 0.0f64;
            for j in 0..8 {
                s += ELEM[j][k] * nv[j];
            }
            f[k] = s - b;
        }
        for (ri, r) in REACTIONS.iter().enumerate() {
            let dnu: f64 = r.iter().map(|&(_, nu)| nu).sum();
            let mut s = 0.0f64;
            for &(j, nu) in r.iter() {
                s += nu * (y[j] - ln_ntot);
            }
            f[3 + ri] = s + dnu * lnpr - lnkp[ri];
        }

        // Jacobian dF/dy (y_j = ln n_j, so dn_j/dy_j = n_j).
        let mut jac: Vec<Vec<f64>> = vec![vec![0.0; 8]; 8];
        for j in 0..8 {
            for k in 0..3 {
                jac[k][j] = ELEM[j][k] * nv[j]; // element rows
            }
        }
        for (ri, r) in REACTIONS.iter().enumerate() {
            let dnu: f64 = r.iter().map(|&(_, nu)| nu).sum();
            for j in 0..8 {
                let nu = r.iter().find(|&&(i, _)| i == j).map_or(0.0, |&(_, v)| v);
                jac[3 + ri][j] = nu - dnu * (nv[j] / ntot);
            }
        }

        let neg: Vec<f64> = f.iter().map(|v| -v).collect();
        let dy = gauss_solve(&jac, &neg);
        let mut step = f64::NEG_INFINITY;
        for d in &dy {
            step = step.max(d.abs());
        }
        // Damping: cap the log-step at 1.
        let scale = if step < 1.0 { 1.0 } else { 1.0 / step };
        for j in 0..8 {
            // Floor: n >= ~1e-35, for the trace species.
            y[j] = (y[j] + scale * dy[j]).max(-80.0);
        }
        if step * scale < 1e-13 {
            converged = true;
            break;
        }
    }

    // CONVERGENCE (rung-6 standing assert, the Newton twin of the burner's fixed-point
    // `else: assert False`): the atom balances below can hold with the log-Kp residuals still
    // open, so guard the FULL solve explicitly. Measured ~10-20 steps, far under 200.
    assert!(converged,
            "equilibrium Newton did not converge in 200 steps at (T={t}, p={p})");

    let mut comp = [0.0f64; 8];
    for j in 0..8 {
        comp[j] = y[j].exp();
    }
    // ATOM CONSERVATION (rung-6 standing assert): the solver enforces C, H, O as equations,
    // so a converged run closes them — this catches a non-converged solve.
    let bal = |k: usize| -> f64 { (0..8).map(|j| ELEM[j][k] * comp[j]).sum() };
    assert!((bal(0) - b_c).abs() < 1e-9 * (b_c + 1e-9), "C balance");
    assert!((bal(1) - b_h).abs() < 1e-9 * (b_h + 1e-9), "H balance");
    assert!((bal(2) - b_o).abs() < 1e-9 * b_o, "O balance");
    comp
}

/// Equilibrium mole numbers per mol dry air at `(f, T, p)` for the (CH2)n fuel.
///
/// Returns the 8 reacting species followed by the inert N2 and Ar — the order
/// [`mixture`] will sum in.
pub fn equilibrium_composition(f: f64, t: f64, p: f64) -> Vec<(&'static str, f64)> {
    let x = air_mole_fractions();
    let xg = |name: &str| x.iter().find(|&&(s, _)| s == name).unwrap().1;
    let n_fuel = f * m_air() / M_CH2;
    let comp = equil_solve(n_fuel, 2.0 * n_fuel, 2.0 * xg("O2"), xg("N2") + xg("Ar"), t, p);
    let mut out: Vec<(&'static str, f64)> =
        SP_REACT.iter().enumerate().map(|(j, &s)| (s, comp[j])).collect();
    out.push(("N2", xg("N2")));
    out.push(("Ar", xg("Ar")));
    out
}

/// A hot section whose composition is the EQUILIBRIUM mixture at the burner's `(Tt4, pt4)`,
/// FROZEN through the turbine and nozzle (rung 6, frozen-downstream).
///
/// Like [`ReactingSection`] it delegates to a memoised per-`far` [`TpgSection`], so `R_t`
/// tracks the dissociation mole-count shift — but the composition comes from the equilibrium
/// solve, so **the burner must [`freeze`](Self::freeze) before any downstream call.** The
/// `(Tt4, pt4)` is baked in at freeze time; downstream calls key on `far` alone, because the
/// frozen mixture does not depend on the evaluation temperature (the turbine asks at Tt5, the
/// nozzle at T9).
///
/// Reusing one gas across two burn configs at the same `far` but different `(Tt4, pt4)` trips
/// the guard. That restores "pure function of far for a fixed burn config" — no hidden state.
#[derive(Debug, Default)]
pub struct EquilibriumSection {
    cache: RefCell<Vec<(u64, TpgSection)>>,
    comp: RefCell<Vec<(u64, Vec<(&'static str, f64)>)>>,
    burn: RefCell<Option<(f64, f64)>>,
}

impl Clone for EquilibriumSection {
    fn clone(&self) -> Self { EquilibriumSection::default() }
}

impl EquilibriumSection {
    pub fn new() -> Self { Self::default() }

    pub fn freeze(&self, far: f64, t_burn: f64, p_burn: f64) -> Vec<(&'static str, f64)> {
        {
            let mut burn = self.burn.borrow_mut();
            match *burn {
                None => *burn = Some((t_burn, p_burn)),
                Some((t0, p0)) => assert!(
                    (t0 - t_burn).abs() < 1e-9 * t_burn && (p0 - p_burn).abs() < 1e-6 * p_burn,
                    "equilibrium section: burn condition changed on a reused Gas \
                     (had {t0}, {p0}; got {t_burn}, {p_burn})"),
            }
        }
        let key = far.to_bits();
        if !self.cache.borrow().iter().any(|&(k, _)| k == key) {
            let comp = equilibrium_composition(far, t_burn, p_burn);
            let (a_low, a_high, r) = mixture(&comp);
            self.cache.borrow_mut().push((key, TpgSection::new(a_low, a_high, r)));
            self.comp.borrow_mut().push((key, comp));
        }
        self.comp.borrow().iter().find(|&&(k, _)| k == key).unwrap().1.clone()
    }

    fn section_for(&self, far: f64) -> TpgSection {
        let key = far.to_bits();
        let hit = self.cache.borrow().iter().find(|&&(k, _)| k == key).map(|&(_, s)| s);
        hit.unwrap_or_else(|| panic!(
            "equilibrium hot section not frozen for far={far}: the burner must run \
             (freeze the station-4 mixture) before any downstream property call"))
    }

    pub fn cp(&self, t: f64, far: f64) -> f64 { self.section_for(far).cp(t) }
    pub fn h(&self, t: f64, far: f64) -> f64 { self.section_for(far).h(t) }
    pub fn pr(&self, t: f64, far: f64) -> f64 { self.section_for(far).pr(t) }
    pub fn t_from_h(&self, h: f64, far: f64) -> f64 { self.section_for(far).t_from_h(h) }
    pub fn t_from_pr(&self, pr: f64, far: f64) -> f64 { self.section_for(far).t_from_pr(pr) }
    pub fn gamma_at(&self, t: f64, far: f64) -> f64 { self.section_for(far).gamma_at(t) }
    pub fn r_at(&self, far: f64) -> f64 { self.section_for(far).r }
}

/// The one property interface components call. They never see which kind they hold.
///
/// Python spells this as three classes with a shared duck-typed interface plus an ignored
/// `far=0.0` on the composition-independent ones; an enum says the same thing to the compiler.
#[derive(Debug, Clone)]
pub enum Section {
    Cpg(CpgSection),
    Tpg(TpgSection),
    Reacting(ReactingSection),
    Equilibrium(EquilibriumSection),
}

impl Section {
    pub fn is_cpg(&self) -> bool { matches!(self, Section::Cpg(_)) }

    pub fn cp(&self, t: f64, far: f64) -> f64 {
        match self {
            Section::Cpg(s) => s.cp(t),
            Section::Tpg(s) => s.cp(t),
            Section::Reacting(s) => s.cp(t, far),
            Section::Equilibrium(s) => s.cp(t, far),
        }
    }
    pub fn h(&self, t: f64, far: f64) -> f64 {
        match self {
            Section::Cpg(s) => s.h(t),
            Section::Tpg(s) => s.h(t),
            Section::Reacting(s) => s.h(t, far),
            Section::Equilibrium(s) => s.h(t, far),
        }
    }
    pub fn pr(&self, t: f64, far: f64) -> f64 {
        match self {
            Section::Cpg(s) => s.pr(t),
            Section::Tpg(s) => s.pr(t),
            Section::Reacting(s) => s.pr(t, far),
            Section::Equilibrium(s) => s.pr(t, far),
        }
    }
    pub fn t_from_h(&self, h: f64, far: f64) -> f64 {
        match self {
            Section::Cpg(s) => s.t_from_h(h),
            Section::Tpg(s) => s.t_from_h(h),
            Section::Reacting(s) => s.t_from_h(h, far),
            Section::Equilibrium(s) => s.t_from_h(h, far),
        }
    }
    pub fn t_from_pr(&self, pr: f64, far: f64) -> f64 {
        match self {
            Section::Cpg(s) => s.t_from_pr(pr),
            Section::Tpg(s) => s.t_from_pr(pr),
            Section::Reacting(s) => s.t_from_pr(pr, far),
            Section::Equilibrium(s) => s.t_from_pr(pr, far),
        }
    }
    pub fn gamma_at(&self, t: f64, far: f64) -> f64 {
        match self {
            Section::Cpg(s) => s.gamma_at(t),
            Section::Tpg(s) => s.gamma_at(t),
            Section::Reacting(s) => s.gamma_at(t, far),
            Section::Equilibrium(s) => s.gamma_at(t, far),
        }
    }
    pub fn r_at(&self, far: f64) -> f64 {
        match self {
            Section::Cpg(s) => s.r_at(),
            Section::Tpg(s) => s.r_at(),
            Section::Reacting(s) => s.r_at(far),
            Section::Equilibrium(s) => s.r_at(far),
        }
    }
}

// --------------------------------------------------------------------------------------
// The flow state.
// --------------------------------------------------------------------------------------

/// Gas state at a station, carried in TOTAL (stagnation) quantities.
///
/// Cycle analysis works in totals because they already fold in the kinetic energy of the
/// flow; we only convert to static at the nozzle exit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowState {
    /// Total temperature, K.
    pub tt: f64,
    /// Total pressure, Pa.
    pub pt: f64,
    /// Mass flow, kg/s.
    pub mdot: f64,
    /// Fuel-air ratio, carried downstream of the burner.
    pub far: f64,
}

impl FlowState {
    pub fn new(tt: f64, pt: f64, mdot: f64) -> Self {
        FlowState { tt, pt, mdot, far: 0.0 }
    }
}
