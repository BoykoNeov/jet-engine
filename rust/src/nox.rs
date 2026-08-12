//! Rungs 7 / 8 / 9 / 19 — thermal NOx as a DECOUPLED diagnostic layer on the rung-6 cycle.
//!
//! NO is a trace (ppm) species, so nothing here feeds the cycle: the design run stays
//! bit-for-bit rung 6. What changes between rungs is only WHERE the chemistry is evaluated.
//!
//! - **rung 7** — the extended Zeldovich mechanism on the mixed-out station-4 pool. NO does
//!   *not* equilibrate; it is rate-limited and frozen far below its equilibrium value, which
//!   INVERTS rung 6's whole premise. `docs/rung7-spec.md`.
//! - **rung 8** — two zones. All the fuel plus a fraction α of the air burn in a hot,
//!   near-stoichiometric PRIMARY; the rest dilutes back to Tt4. EI_NO lifts out of the
//!   mixed-out ~zero into the ICAO band. `docs/rung8-spec.md`.
//! - **rung 9** — the primary is allowed RICH (φ ≤ 2). EI_NO forms a bell PEAKING near
//!   stoichiometric and collapsing on the rich flank, which is why real low-NOx combustors
//!   burn rich. `docs/rung9-spec.md`.
//! - **rung 19** — the equilibrium-[O] LOWER BOUND every NO number since rung 7 carried,
//!   lifted two ways: a computed T-driven super-equilibrium factor and an imposed prompt
//!   (Fenimore) bump. Both refute "the rich primary explodes", from opposite directions.
//!   `docs/rung19-spec.md`.
//!
//! # What the port had to be careful about here
//!
//! Every composition is an ORDERED slice, never a map, because Python sums its dicts in
//! insertion order and float addition is not associative. In this slice every composition
//! comes from [`gas::equilibrium_composition`] or [`gas::air_mole_fractions`], so no new
//! ordering is introduced — but rung 10's quench trajectory BUILDS one, and that is where
//! the next slice has to look.
//!
//! The Zeldovich integrator is fixed-step RK4 with no adaptive control, so it carries no
//! stopping rule. What it does carry is 4000 accumulations, and `c += dt/6.0 * (…)` is a
//! different function in the last bit from `(…) * dt/6.0`. The two bisections
//! ([`primary_aft`], [`mixed_out_t`]) are where the stopping rules live, and their inner
//! evaluation is the 8-species Newton — the deepest solver nesting in the project.

use crate::gas::{
    self, air_mole_fractions, equilibrium_composition, g_molar, h_molar_a, m_air, powp, species,
    Gas, M_CH2, M_CH2_KG, RU,
};

/// Extended Zeldovich mechanism (Hanson & Salimian 1984, as tabulated in Turns):
///
/// ```text
/// 1: O + N2 <=> NO + N     2: N + O2 <=> NO + O     3: N + OH <=> NO + H
/// ```
///
/// `k = A·T^n·exp(−θ/T)`, native cm³/mol/s → SI m³/(mol·s) via `·1e-6`. Every reaction is
/// mole-conserving (Δν = 0), so `Kc = Kp` with no `(p/p0)` factor.
///
/// Held as an ordered table rather than a map: the key order is the order the K-check
/// multiplies in, and that is part of the arithmetic.
pub const ZELDOVICH: &[(&str, f64, f64, f64)] = &[
    ("1f", 1.8e14, 0.0, 38370.0),
    ("1r", 3.8e13, 0.0, 425.0),
    ("2f", 1.8e10, 1.0, 4680.0),
    ("2r", 3.8e9, 1.0, 20820.0),
    ("3f", 7.1e13, 0.0, 450.0),
    ("3r", 1.7e14, 0.0, 24560.0),
];

/// NO molar mass, kg/mol.
pub fn m_no() -> f64 {
    species("NO").m / 1000.0
}

// --------------------------------------------------------------------------------------
// RUNG 19 — super-equilibrium O (lifting the equilibrium-O lower bound).
//
// Every NO number since rung 7 reads the rung-6 EQUILIBRIUM [O] into the Zeldovich rate, so
// it is a LOWER BOUND. Fluent (Theory Guide § 9.1.3) offers a PARTIAL-EQUILIBRIUM O closure
// (Westenberg 1971, adding the 3-body O+O+M ⇌ O2+M) that sits ABOVE equilibrium O. Both
// share the same [O2]^0.5, so their RATIO is dimensionless and T-ONLY — no absolute-magnitude
// sourcing is needed:
//
//     [O]_eq = C1·T^−0.5·[O2]^0.5·exp(−θ1/T)
//     [O]_pe = C2·T^+0.5·[O2]^0.5·exp(−θ2/T)
//     m(T)   = [O]_pe/[O]_eq = (C2/C1)·T·exp((θ1−θ2)/T)  ∈ [1.16, 1.50] over 1800–2400 K
//
// We lift OUR OWN rung-6 comp["O"] by m(T) inside the rung-7 integrator; m ≡ 1 ⇒ bit-for-bit
// rung 7. The lift is T-DRIVEN (φ-independent) — WEAKEST in the O2-depleted rich primary, so
// it does NOT match the naive "rich explosion" intuition. Constants TRANSCRIBED from the
// standard published forms (image-locked sources), NOT digit-verified; cross-validated by the
// equilibrium-O units gate. `docs/rung19-spec.md`.
// --------------------------------------------------------------------------------------
pub const WESTENBERG_C1: f64 = 3.970e5;
pub const WESTENBERG_TH1: f64 = 31090.0;
pub const WESTENBERG_C2: f64 = 36.64;
pub const WESTENBERG_TH2: f64 = 27123.0;

/// Flame-band floor for the super-eq O lift THROUGH the quench (rung 20).
///
/// `m(T) = A·T·exp(B/T)` with `B = θ1 − θ2 ≈ 3967 K` DIVERGES as `T → 0` (m(1500 K) ≈ 1.9,
/// m(1200 K) ≈ 3), so lifting [O] on a cooling quench path that reaches `T_mix ≈ Tt4` would
/// inject an out-of-band multiplier. The Westenberg partial-eq closure is a FLAME model
/// (T ≳ 1500 K) anyway. Rung 20 consumes this; rung 19's primary is always well above it.
pub const SUPER_EQ_T_FLOOR: f64 = 1500.0;

/// Super-equilibrium O multiplier `m(T) = [O]_pe/[O]_eq = (C2/C1)·T·exp((θ1−θ2)/T)`.
///
/// The shared `[O2]^0.5` cancels, so this is DIMENSIONLESS and T-ONLY. ∈ [1.16, 1.50] over
/// the flame band, DECREASING in T (→ 1 as T → ∞: the partial-eq pool relaxes to equilibrium
/// once the fast H-atom shuffle equilibrates). φ-INDEPENDENT: the lift is T-driven, NOT
/// rich-driven — it is WEAKEST in the O2-starved rich primary, where thermal NO has already
/// died. `m ≡ 1` recovers rung 7 exactly. Rung 19.
pub fn super_eq_o_multiplier(t: f64) -> f64 {
    (WESTENBERG_C2 / WESTENBERG_C1) * t * ((WESTENBERG_TH1 - WESTENBERG_TH2) / t).exp()
}

/// Zeldovich rate constant `k(T)` in SI m³/(mol·s).
///
/// The four factors are LEFT-ASSOCIATED exactly as Python spells them, and `T ** n` is a libm
/// `pow` call off the table — hence [`powp`], not a product chain. The tabulated `n` are 0.0
/// and 1.0, where `pow` is exact for any spelling; the faithful transcription costs nothing
/// and is what keeps a future non-integral `n` honest.
pub fn k_zeldovich(key: &str, t: f64) -> f64 {
    let &(_, a, n, theta) = ZELDOVICH
        .iter()
        .find(|&&(k, _, _, _)| k == key)
        .expect("unknown Zeldovich reaction");
    a * powp(t, n) * (-theta / t).exp() * 1e-6
}

/// `Kp(½N₂ + ½O₂ ⇌ NO) = exp(−ΔG°/RuT)`, `ΔG° = g(NO) − ½g(N₂) − ½g(O₂)`.
///
/// Δν = 0 ⇒ no `(p/p0)` factor: equilibrium NO is PRESSURE-INDEPENDENT, which inverts rung 6
/// (where pressure suppresses dissociation).
pub fn kp_no(t: f64) -> f64 {
    let d_g0 = g_molar("NO", t) - 0.5 * g_molar("N2", t) - 0.5 * g_molar("O2", t);
    (-d_g0 / (RU * t)).exp()
}

/// Superimposed equilibrium NO mole fraction from the frozen rung-6 mixture `comp` (mole
/// numbers per mol air).
///
/// NO is trace: it does NOT perturb `comp`. `x_NO_e = Kp_NO·√(x_N2·x_O2)` using that
/// mixture's own N₂/O₂ (N is negligible, ~1e-5 of NO).
pub fn equilibrium_no_fraction(comp: &[(&str, f64)], t: f64) -> f64 {
    let ntot: f64 = comp.iter().map(|&(_, v)| v).sum();
    // The division order is Python's: ((N2/ntot)·O2)/ntot, not (N2·O2)/(ntot·ntot).
    kp_no(t) * (need(comp, "N2") / ntot * need(comp, "O2") / ntot).sqrt()
}

/// Thermo-kinetic K-check: `(k1f·k2f)/(k1r·k2r)` against `Kc(N₂+O₂ ⇌ 2NO) = exp(−ΔG°/RuT)`
/// with `ΔG° = 2g(NO) − g(N₂) − g(O₂)`.
///
/// Reactions 1+2 sum to N₂+O₂ ⇌ 2NO, so detailed balance ties the transcribed RATE CONSTANTS
/// to the a6/a7 THERMO (N cancels). Dimensionless — the `1e-6` SI factor cancels. Measured
/// ~1.035–1.044, and it is asserted on every diagnostic run: a gross transcription slip is
/// orders of magnitude off. This is the twin of rung 6's atom-balance assert.
pub fn kcheck_ratio(t: f64) -> f64 {
    let kc_rate =
        (k_zeldovich("1f", t) * k_zeldovich("2f", t)) / (k_zeldovich("1r", t) * k_zeldovich("2r", t));
    let d_g0 = 2.0 * g_molar("NO", t) - g_molar("N2", t) - g_molar("O2", t);
    kc_rate / (-d_g0 / (RU * t)).exp()
}

/// `comp[name]` — Python's dict indexing, so a missing species is a bug, not a zero.
fn need(comp: &[(&str, f64)], name: &str) -> f64 {
    comp.iter()
        .find(|&&(s, _)| s == name)
        .unwrap_or_else(|| panic!("species {name} absent from composition"))
        .1
}

/// `comp.get(name, 0.0)` — for the species the integrator tolerates being absent (O, H are
/// present only in a DISSOCIATING pool, so a frozen rung-4 mixture legitimately lacks them).
fn maybe(comp: &[(&str, f64)], name: &str) -> f64 {
    comp.iter().find(|&&(s, _)| s == name).map_or(0.0, |&(_, v)| v)
}

/// Thermal-NO diagnostic at one (frozen pool, T, p, τ). A pure DIAGNOSTIC — it never feeds
/// the cycle. Mole fractions; rates in mol/m³/s; EI in g NO / kg fuel.
#[derive(Debug, Clone, Copy)]
pub struct NoxState {
    /// kinetic NO mole fraction after residence time τ
    pub x_no: f64,
    /// equilibrium NO mole fraction (the ceiling)
    pub x_no_eq: f64,
    /// `d[NO]/dt` at t=0 = `2·k1f·[O]_e·[N2]_e`, mol/m³/s
    pub initial_rate: f64,
    /// `τ_NO = [NO]_e / initial_rate`, s (≫ residence ⇒ frozen)
    pub char_time: f64,
    /// emission index, g NO / kg fuel (thermal, m-lifted when `super_eq_o`)
    pub ei_no: f64,
    /// RUNG 19 — super-eq O multiplier m(T) applied to [O] (1.0 ⇒ bit-for-bit rung 7)
    pub o_multiplier: f64,
    /// RUNG 19 — additive prompt (Fenimore) EI, g NO/kg fuel (0.0 ⇒ thermal only)
    pub ei_no_prompt: f64,
}

impl NoxState {
    /// Total EI = thermal (m-lifted) + prompt, g NO/kg fuel.
    ///
    /// Rung 19: the equilibrium-O lower bound lifted two ways — a COMPUTED T-driven
    /// super-eq-O factor already folded into `ei_no` (via the lifted [O]), plus the IMPOSED
    /// additive prompt bump.
    pub fn ei_no_total(&self) -> f64 {
        self.ei_no + self.ei_no_prompt
    }
    pub fn ppm(&self) -> f64 {
        self.x_no * 1e6
    }
    pub fn ppm_eq(&self) -> f64 {
        self.x_no_eq * 1e6
    }
    pub fn fraction_of_equil(&self) -> f64 {
        self.x_no / self.x_no_eq
    }
}

/// Kinetic NO after residence time `tau` on the frozen pool `comp` at `(T, p)`.
///
/// One-equation extended-Zeldovich model (Heywood/Turns), QSS on N, REVERSE-RATE form for
/// R2/R3 so equilibrium [N] is never needed (uses the pool's own O, H):
///
/// ```text
/// d[NO]/dt = 2·R1·(1 − a²)/(1 + a·R1/(R2+R3)),   a = [NO]/[NO]_e
/// R1 = k1f[O][N2],   R2 = k2r[NO]_e[O],   R3 = k3r[NO]_e[H]
/// ```
///
/// `a = 0` → rate = 2R1 (the initial rate); `a → 1` → rate = 0, so the integrator saturates
/// at `[NO]_e` and `τ → ∞` recovers the equilibrium NO — an internal consistency gate. RK4
/// from `[NO] = 0`.
///
/// RUNG 19 — `o_multiplier` (default 1.0 ⇒ byte-identical rung 7) lifts the pool's [O] by the
/// super-equilibrium factor m(T) BEFORE forming R1/R2. In the kinetically-limited (frozen)
/// regime the NO stays far below the `[NO]_e` ceiling, so the rate ∝ [O] and `x_no` scales
/// ~linearly with m — a faster FORMATION, not a higher equilibrium (the ceiling is a
/// thermodynamic quantity, independent of the O-atom closure, so the clamp still binds at the
/// same value). `super_eq_o` thus lifts the equilibrium-O lower bound.
pub fn thermal_no(
    comp: &[(&str, f64)],
    t: f64,
    p: f64,
    tau: f64,
    far: f64,
    nsteps: usize,
    o_multiplier: f64,
) -> NoxState {
    // K-CHECK (rung-7 standing assert, on every diagnostic run): the transcribed rate
    // constants must agree with the a6/a7 thermo at the evaluation T. The twin of rung 6's
    // atom-balance assert.
    let kr = kcheck_ratio(t);
    assert!(
        0.90 < kr && kr < 1.15,
        "Zeldovich K-check off: ratio {kr:.4} at T={t}"
    );

    let ntot: f64 = comp.iter().map(|&(_, v)| v).sum();
    let conc = p / (RU * t); // total molar concentration, mol/m³
                             // Python builds `x = {s: comp[s]/ntot}` first, so every concentration is
                             // (n_i/ntot)·conc — NOT (n_i·conc)/ntot, which differs in the last bit.
    let c_o = maybe(comp, "O") / ntot * conc * o_multiplier; // rung 19: m=1.0 ⇒ rung 7
    let c_n2 = need(comp, "N2") / ntot * conc;
    let c_h = maybe(comp, "H") / ntot * conc;
    let x_no_eq = equilibrium_no_fraction(comp, t);
    let c_noe = x_no_eq * conc;
    // TRACE guard (rung 7): NO must be trace for the decoupled-diagnostic assumption.
    assert!(
        x_no_eq < 0.02,
        "NO not trace (x_NO_e={x_no_eq:.4e}) — decoupling invalid"
    );

    let r1 = k_zeldovich("1f", t) * c_o * c_n2;
    let r2 = k_zeldovich("2r", t) * c_noe * c_o;
    let r3 = k_zeldovich("3r", t) * c_noe * c_h;
    let beta = if (r2 + r3) > 0.0 { r1 / (r2 + r3) } else { 0.0 };

    let rate = |c_no: f64| {
        let a = c_no / c_noe;
        2.0 * r1 * (1.0 - a * a) / (1.0 + beta * a)
    };

    let dt = tau / nsteps as f64;
    let mut c_no = 0.0f64;
    for _ in 0..nsteps {
        let k1 = rate(c_no);
        let k2 = rate(c_no + 0.5 * dt * k1);
        let k3 = rate(c_no + 0.5 * dt * k2);
        let k4 = rate(c_no + dt * k3);
        // The accumulation is a SPELLING: `dt/6.0` multiplies the sum from the left, and the
        // sum itself associates left. `(…)·dt/6.0` is a different function in the last bit,
        // and this loop runs it 4000 times.
        c_no += dt / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
        if c_no > c_noe {
            c_no = c_noe; // clamp: never overshoot equilibrium
        }
    }
    // STANDING assert: the integrator stays in [0, [NO]_e].
    assert!(
        -1e-12 <= c_no && c_no <= c_noe * (1.0 + 1e-9),
        "kinetic NO out of [0,eq]: {c_no} vs {c_noe}"
    );

    let x_no = c_no / conc;
    // Emission index, g NO / kg fuel: NO moles per mol air = x_no·ntot; fuel mass = n_fuel·M_CH2.
    let n_fuel = far * m_air() / M_CH2;
    let ei = if n_fuel > 0.0 {
        1000.0 * (x_no * ntot * m_no()) / (n_fuel * M_CH2_KG)
    } else {
        0.0
    };
    NoxState {
        x_no,
        x_no_eq,
        initial_rate: 2.0 * r1,
        char_time: if r1 > 0.0 { c_noe / (2.0 * r1) } else { f64::INFINITY },
        ei_no: ei,
        o_multiplier,
        ei_no_prompt: 0.0,
    }
}

// --- Rung-8 two-zone (primary → dilution) NOx helpers (all on rung-6/7 primitives) -------

/// Scale-A molar enthalpy of 1 mol air at T (summed over air species, formation datum).
///
/// N₂/O₂/Ar carry zero formation enthalpy; the trace CO₂ in air carries its ΔHf298. The same
/// absolute (scale-A) datum the rung-6 AFT diagnostic used — no new convention.
pub fn h_air_molar_a(t: f64) -> f64 {
    air_mole_fractions().iter().map(|&(s, x)| x * h_molar_a(s, t)).sum()
}

/// `Σ nᵢ·h̄ᵢ_A(T)` over a composition, in ITS order. Both bisections below call this at every
/// trial temperature, and the order is the composition's, never a sorted one.
fn h_prod_scale_a(comp: &[(&str, f64)], t: f64) -> f64 {
    comp.iter().map(|&(s, n)| n * h_molar_a(s, t)).sum()
}

/// Adiabatic flame temperature of (fuel + 1 mol primary air), air PREHEATED to `t_air`.
///
/// Bisection on T so the equilibrium products' scale-A enthalpy equals the reactants':
///
/// ```text
/// Σ nᵢ(far_p, T)·h̄ᵢ_A(T) = h̄_air_A(T_air) + n_fuel·hf_fuel
/// ```
///
/// Rung 8: the primary burns all the fuel with only its share of the air, preheated to the
/// ACTUAL compressor-exit Tt3 (not 298 K). Preheating from Tt3 is what ties the primary flame
/// to the running cycle and makes the α→1 reduce-to-rung-7 gate exact.
///
/// The inner `h_prod_scale_a` re-solves the 8-species equilibrium Newton at every trial T, so
/// this is a bisection wrapped around a Newton — the deepest solver nesting in the project.
pub fn primary_aft(far_p: f64, p: f64, t_air: f64, hf_fuel: f64) -> f64 {
    let n_fuel = far_p * m_air() / M_CH2;
    let h_react = h_air_molar_a(t_air) + n_fuel * hf_fuel;

    let (mut lo, mut hi) = (800.0f64, 3200.0f64);
    for _ in 0..100 {
        let t = 0.5 * (lo + hi);
        // products' scale-A enthalpy, monotone ↑ in T
        if h_prod_scale_a(&equilibrium_composition(far_p, t, p), t) > h_react {
            hi = t;
        } else {
            lo = t;
        }
        if hi - lo < 1e-6 {
            // ~31 iters to 1e-6 K; below any anchor
            break;
        }
    }
    let t = 0.5 * (lo + hi);
    // Bracket guard (post-loop, not an endpoint eval — `equilibrium_composition` DIVERGES at
    // the cold 800 K edge, so we cannot probe there; a root outside [800,3200] instead pins
    // the bisection against an edge, which this catches). Any real flame sits well inside.
    assert!(
        801.0 < t && t < 3199.0,
        "primary_aft: flame temp {t:.1} K pinned at [800,3200] K bracket edge (far_p={far_p:.4})"
    );
    t
}

/// Mixed-out temperature after adding `(1−α)` mol dilution air at `t_dilution` to `α` mol
/// primary products and RE-EQUILIBRATING the major species at the overall `far_ov`.
///
/// Basis: 1 mol TOTAL air. Primary products are per-mol-PRIMARY-air, so scale by α. Enthalpy
/// is conserved; bisection on `T_mix` so the re-equilibrated pool's scale-A enthalpy equals
/// `α·H_primary + (1−α)·H_dilution_air`. Re-equilibrating (NOT freezing) the dissociated
/// primary majors RELEASES the stored dissociation energy, so `T_mix` returns to ≈ Tt4 — the
/// rung-8 conservation gate.
///
/// By `α·far_p = far_ov`, α cancels in the balance, so `T_mix` is split-independent by
/// construction (it is the overall adiabatic flame temperature from Tt3).
pub fn mixed_out_t(
    comp_prim: &[(&str, f64)],
    t_prim: f64,
    alpha: f64,
    far_ov: f64,
    t_dilution: f64,
    p: f64,
) -> f64 {
    let h_prim = alpha * h_prod_scale_a(comp_prim, t_prim);
    let h_dil = (1.0 - alpha) * h_air_molar_a(t_dilution);
    let h_mix = h_prim + h_dil;

    let (mut lo, mut hi) = (700.0f64, 3200.0f64);
    for _ in 0..100 {
        let t = 0.5 * (lo + hi);
        // re-equilibrated pool enthalpy, monotone ↑ in T
        if h_prod_scale_a(&equilibrium_composition(far_ov, t, p), t) > h_mix {
            hi = t;
        } else {
            lo = t;
        }
        if hi - lo < 1e-6 {
            break;
        }
    }
    let t = 0.5 * (lo + hi);
    // Bracket guard (post-loop; cold-edge eval diverges — see `primary_aft`).
    assert!(
        701.0 < t && t < 3199.0,
        "mixed_out_t: mix temp {t:.1} K pinned at [700,3200] K bracket edge (far_ov={far_ov:.4})"
    );
    t
}

/// Rung-19 prompt-NO (Fenimore) config — De Soete's (1975) global-rate CORRECTION FACTOR
/// reduced to its fitted, rich-peaking φ-shape.
///
/// An IMPOSED trace channel ADDED beside thermal NO; it is the RICH-SPECIFIC lift of the
/// equilibrium-O lower bound (the complement of super-eq O, which is T-driven and modest —
/// the naive "rich explosion" intuition fails BOTH ways, and prompt SURVIVES where thermal
/// dies on the rich flank).
///
/// ```text
/// EI_prompt(φ,T) = scale · max(f(φ,n), 0) · exp(−Ea/RuT)
///   f(φ,n) = 4.75 + 0.0819·n − 23.2·φ + 32·φ² − 12.2·φ³      (De Soete, valid φ∈[0.6,1.6])
/// ```
///
/// THE MAGNITUDE IS IMPOSED, not derived — a 0-D burnt pool has no flame structure (thin-zone
/// fuel loading, flame-front residence) to anchor the absolute g/kg, so `scale` is back-solved
/// from a REFERENCE-POINT EI `peak_ei` imposed at `(phi_ref, T_ref)`. `T_ref` is a REALISTIC
/// near-peak primary AFT (~2400 K) so the reference is physical and the delivered prompt peak
/// lands near `peak_ei` (~2 g/kg, the ~1–5 g/kg literature band). The delivered EI still
/// tracks the LOCAL primary T: prompt carries `exp(−Ea/RuT)`, so a hotter primary nudges it
/// ABOVE `peak_ei` — a reference value, not a hard cap. Only the φ-SHAPE and the directional
/// prompt/thermal ratio are certified, NOT the number.
///
/// The burnt-pool `[O2]^a·[FUEL]` factors of the full De Soete rate are DROPPED (they
/// double-count O2 depletion on an already-burnt pool and flip the shape lean-peaking); the
/// rich peak lives ONLY in De Soete's fitted f(φ). `f < 0` past φ≈1.65 ⇒ CLAMPED at 0: the
/// deep-rich flank up to the soot bound φ=2 is OUTSIDE the prompt model (flagged, not
/// modelled).
///
/// THE T-SENSITIVITY DISCRIMINATOR: prompt carries a SINGLE Arrhenius exp; thermal a DOUBLE
/// (k1f·[O]_eq, itself ∝ exp). Measured 2000→2400 K at stoich: thermal ×566, prompt ×21 —
/// prompt is ~27× milder, the quantitative face of "survives where thermal dies".
#[derive(Debug, Clone, Copy)]
pub struct PromptNo {
    /// IMPOSED reference prompt EI at `(phi_ref, T_ref)`, g NO/kg fuel — the magnitude concession
    pub peak_ei: f64,
    /// fuel carbon number (C12 Jet-A surrogate on the per-(CH2) basis; a modeling choice)
    pub n_carbon: f64,
    /// De Soete activation energy, J/mol (Fluent modified De Soete; transcribed)
    pub ea: f64,
    /// reference flame T at which `peak_ei` is imposed, K (a realistic near-peak primary AFT)
    pub t_ref: f64,
    /// f(φ) cubic-maximum location (where the reference EI is imposed)
    pub phi_ref: f64,
    /// De Soete φ-validity ceiling; above this f(φ) is extrapolation (flagged)
    pub phi_valid_max: f64,
}

impl Default for PromptNo {
    fn default() -> Self {
        Self {
            peak_ei: 2.0,
            n_carbon: 12.0,
            ea: 303474.0,
            t_ref: 2400.0,
            phi_ref: 1.24,
            phi_valid_max: 1.6,
        }
    }
}

impl PromptNo {
    /// Python's `__post_init__`. Rust cannot run a constructor on struct-literal syntax, so
    /// this is called at every point of use — which is what the Python dataclass achieves and
    /// is cheap beside an equilibrium solve.
    pub fn validate(&self) {
        for (name, v) in [
            ("peak_ei", self.peak_ei),
            ("n_carbon", self.n_carbon),
            ("Ea", self.ea),
            ("T_ref", self.t_ref),
            ("phi_ref", self.phi_ref),
        ] {
            assert!(v > 0.0, "PromptNo.{name}={v} must be positive");
        }
        assert!(
            self.f_correction(self.phi_ref) > 0.0,
            "PromptNo.phi_ref={} sits where f(φ)≤0 — cannot calibrate the scale there",
            self.phi_ref
        );
    }

    /// De Soete's fitted correction factor `f(φ,n) = 4.75 + 0.0819n − 23.2φ + 32φ² − 12.2φ³`.
    ///
    /// A cubic in φ peaking slightly rich (~φ=1.24) and going NEGATIVE past φ≈1.65 (the
    /// validity ceiling). This is where the rich-peaking prompt SHAPE lives; the magnitude is
    /// the imposed `scale`.
    pub fn f_correction(&self, phi: f64) -> f64 {
        let n = self.n_carbon;
        // The square is a multiply and the cube is a `pow` — PyPy's JIT rewrites `x ** 2` and
        // does not rewrite `x ** 3`, so reproducing PyPy means spelling them differently.
        4.75 + 0.0819 * n - 23.2 * phi + 32.0 * (phi * phi) - 12.2 * powp(phi, 3.0)
    }

    /// The imposed EI prefactor, back-solved so `EI_prompt(phi_ref, T_ref) == peak_ei`.
    ///
    /// Makes the one un-derivable magnitude TRANSPARENT (a physical reference EI ~2 g/kg at a
    /// realistic primary AFT) rather than an opaque pre-exponential. The rung-19 concession
    /// made legible.
    pub fn scale(&self) -> f64 {
        self.peak_ei / (self.f_correction(self.phi_ref) * (-self.ea / (RU * self.t_ref)).exp())
    }

    /// Imposed prompt EI, g NO/kg fuel = `scale·max(f(φ),0)·exp(−Ea/RuT)`.
    ///
    /// Clamped ≥ 0 (f < 0 for φ > ~1.65 ⇒ no negative prompt). Equals `peak_ei` at
    /// `(phi_ref, T_ref)` and rises above it where the local primary T exceeds `T_ref`. The
    /// SINGLE exp is why prompt is far less T-sensitive than the double-exp thermal.
    pub fn ei_prompt(&self, phi: f64, t: f64) -> f64 {
        self.scale() * self.f_correction(phi).max(0.0) * (-self.ea / (RU * t)).exp()
    }
}

/// Knobs for [`Gas::thermal_nox`] — rung 7's residence time plus rung 19's two channels.
///
/// A struct rather than a parameter list, and deliberately so: § 2 of the port plan makes it
/// a RULE that a hook takes a config, because otherwise every later rung that adds a knob
/// edits every existing call site. `..Default::default()` is Python's keyword defaults.
#[derive(Debug, Clone, Copy)]
pub struct ThermalNoxOpts {
    /// residence time, s — an UN-ANCHORED knob, stated like the specified exit pressure
    pub tau: f64,
    /// rung 19: lift [O] by the Westenberg m(T) (false ⇒ bit-for-bit rung 7)
    pub super_eq_o: bool,
    /// rung 19: add the imposed De Soete prompt bump (None ⇒ no term)
    pub prompt: Option<PromptNo>,
    /// the local φ the prompt is evaluated at (None ⇒ `far / f_stoich`)
    pub phi: Option<f64>,
    /// RK4 steps for the Zeldovich integrator
    pub nsteps: usize,
}

impl Default for ThermalNoxOpts {
    fn default() -> Self {
        Self { tau: 3e-3, super_eq_o: false, prompt: None, phi: None, nsteps: 4000 }
    }
}

/// Knobs for [`Gas::zoned_nox`].
///
/// Rungs 10–24 each append ONE field here and leave every existing call site untouched —
/// which is the whole reason it is a struct. Python's equivalent is a twelve-`Option`
/// parameter list whose mutual-exclusion rules are re-asserted at the top of the method.
#[derive(Debug, Clone, Copy)]
pub struct ZonedNoxOpts {
    /// primary-zone residence time, s
    pub tau: f64,
    /// rung 19: lift the primary [O] by the Westenberg m(T_p)
    pub super_eq_o: bool,
    /// rung 19: the imposed prompt bump at `phi_primary`
    pub prompt: Option<PromptNo>,
    /// RK4 steps for the primary Zeldovich integrator
    pub nsteps: usize,
}

impl Default for ZonedNoxOpts {
    fn default() -> Self {
        Self { tau: 3e-3, super_eq_o: false, prompt: None, nsteps: 4000 }
    }
}

/// Two-zone (primary → dilution) thermal-NO diagnostic (rung 8).
///
/// Like [`NoxState`] it never feeds the cycle. NO is set in the hot primary and FROZEN
/// through the dilution that cools the gas to `T_mix ≈ Tt4`; EI_NO is a per-kg-fuel quantity
/// set in the primary — **dilution lowers the mole FRACTION, not the emission INDEX.**
#[derive(Debug, Clone, Copy)]
pub struct ZonedNoxState {
    /// primary equivalence ratio (≤ 2, lean-to-rich RQL scope; rung 9)
    pub phi_primary: f64,
    /// primary fuel/air ratio = `phi_primary · f_stoich`
    pub far_primary: f64,
    /// fraction of the air routed to the primary (≤ 1)
    pub alpha: f64,
    /// adiabatic primary flame temperature, K (from Tt3)
    pub t_primary: f64,
    /// mixed-out temperature after dilution, K (≈ Tt4)
    pub t_mix: f64,
    /// the rung-7 NO diagnostic evaluated ON the hot primary pool
    pub primary: NoxState,
    /// NO mole fraction after dilution (frozen moles / mixed total moles)
    pub x_no_mix: f64,
    /// RUNG 19 — whether the [O] was super-eq-lifted
    pub super_eq_o: bool,
    /// RUNG 19 — the m(T_p) applied to the primary O (1.0 ⇒ rung 7 baseline)
    pub o_multiplier: f64,
    /// RUNG 19 — the imposed prompt-NO config, if any
    pub prompt: Option<PromptNo>,
    /// RUNG 19 — additive primary prompt EI, g NO/kg fuel (0.0 ⇒ thermal only)
    pub ei_no_prompt: f64,
}

impl ZonedNoxState {
    /// Emission index, g NO / kg fuel — set in the primary, conserved through dilution (α
    /// cancels: NO moles and fuel moles both scale with the primary air fraction).
    ///
    /// This is the IDEAL-quench (rung 9) EI. Rung 10's finite quench RE-MAKES NO at the
    /// stoichiometric crossing the dilution passes through, and gets its own field.
    pub fn ei_no(&self) -> f64 {
        self.primary.ei_no
    }
    /// Total primary EI = thermal (super-eq-O-lifted) + prompt, g NO/kg fuel (rung 19).
    pub fn ei_no_total(&self) -> f64 {
        self.ei_no() + self.ei_no_prompt
    }
    pub fn ppm_primary(&self) -> f64 {
        self.primary.x_no * 1e6
    }
    pub fn ppm_mix(&self) -> f64 {
        self.x_no_mix * 1e6
    }
}

impl Gas {
    /// Thermal-NO diagnostic on the equilibrium pool at `(far, T, p)` after residence time
    /// `tau` (default 3 ms, a typical gas-turbine primary-zone residence — an UN-ANCHORED
    /// knob, stated like the specified exit pressure).
    ///
    /// Solves the rung-6 equilibrium composition (scale-A, datum-free mole numbers),
    /// superimposes equilibrium NO, and integrates the extended Zeldovich mechanism. Trace
    /// species ⇒ this does NOT affect the cycle; it is a pure diagnostic.
    ///
    /// RUNG 19 lifts the equilibrium-O LOWER BOUND two ways — `super_eq_o` scales the pool's
    /// [O] by the Westenberg m(T) inside the integrator, and `prompt` ADDS the imposed De
    /// Soete bump at the local φ (defaulting to `far / f_stoich`). Both off ⇒ the exact prior
    /// code path. The SUMMED trace guard spans both channels.
    pub fn thermal_nox(&self, far: f64, t: f64, p: f64, o: ThermalNoxOpts) -> NoxState {
        let comp = equilibrium_composition(far, t, p);
        let m = if o.super_eq_o { super_eq_o_multiplier(t) } else { 1.0 };
        assert!(
            (1.0..=2.0).contains(&m),
            "super-eq O multiplier m={m:.3} at T={t:.0} K outside the flame-band bound [1,2] — \
             the Westenberg partial-eq/eq ratio is a FLAME model (T≳1500 K)"
        );
        let mut nox = thermal_no(&comp, t, p, o.tau, far, o.nsteps, m);
        if let Some(pr) = o.prompt {
            pr.validate();
            let phi_local = o.phi.unwrap_or(far / gas::f_stoich());
            nox.ei_no_prompt = pr.ei_prompt(phi_local, t);
        }
        // SUMMED trace guard (rung 19): thermal (m-lifted) + prompt must stay trace. x_no ∝ EI
        // at fixed far, so convert the prompt EI to a mole fraction via the thermal x_no/EI ratio.
        let x_no_prompt =
            if nox.ei_no > 0.0 { nox.ei_no_prompt / nox.ei_no * nox.x_no } else { 0.0 };
        assert!(
            nox.x_no + x_no_prompt < 0.02,
            "summed NO not trace (x_NO_thermal+prompt={:.4e}) — decoupling invalid",
            nox.x_no + x_no_prompt
        );
        nox
    }

    /// Two-zone (primary → dilution) thermal NOx — rungs 8, 9 and 19.
    ///
    /// Runs the SAME rung-7 extended-Zeldovich integrator on a HOT, near-stoichiometric
    /// PRIMARY zone instead of the mixed-out station-4 pool, then dilutes back to Tt4:
    ///
    /// 1. **air split** — all the fuel plus a fraction α of the air enter the primary at
    ///    `far_p = phi_primary·f_stoich`; `α = far/far_p ≤ 1`;
    /// 2. **primary AFT** — adiabatic flame temperature from Tt3 (scale A, equilibrium
    ///    products);
    /// 3. **primary NO** — the rung-7 integrator on the primary equilibrium pool at `(T_p, p)`;
    /// 4. **dilution** — add the remaining air at Tt3, re-equilibrate the MAJORS (releasing
    ///    the dissociation energy, so `T_mix ≈ Tt4`), and FREEZE the NO moles.
    ///
    /// NO is trace, so the cycle stays bit-for-bit rung 6 (NO/N never enter the equilibrium
    /// solve); only WHERE the chemistry is evaluated changes. The capped mixed-out Tt4 makes
    /// almost no NO; the hot primary — averaged away at station 4 — is where it forms.
    ///
    /// RUNG 9 — the primary may run RICH (`phi_primary` up to 2.0): the equilibrium pool
    /// carries major CO/H₂ and the integrator runs on it unchanged. This closes the RQL
    /// (rich-burn → quick-quench → lean-burn) story: EI_NO forms a bell that PEAKS near
    /// stoichiometric (φ≈0.95) and FALLS steeply on the rich flank — the AFT rolls over and
    /// the O-starved pool crashes [O]. That rich-side collapse is WHY real low-NOx combustors
    /// burn a rich primary. The mix-out here is the IDEAL (infinitely-fast) quench: NO is
    /// simply frozen at the primary value. Held below soot onset (φ ≤ 2).
    ///
    /// RUNG 19 acts ONLY on the primary diagnostic, by the same two channels as
    /// [`Gas::thermal_nox`].
    pub fn zoned_nox(
        &self,
        far: f64,
        tt3: f64,
        tt4: f64,
        p: f64,
        phi_primary: f64,
        o: ZonedNoxOpts,
    ) -> ZonedNoxState {
        assert!(
            0.0 < phi_primary && phi_primary <= 2.0 + 1e-9,
            "phi_primary {phi_primary} outside (0, 2] — the 5-species (no soot / no C(s)) basis \
             is valid only below soot onset (~φ2; graphite onset is φ3). Rich RQL scope."
        );
        let hf_fuel = self.spec.hf_fuel_molar.unwrap_or_else(gas::hf_fuel_default);
        let far_p = phi_primary * gas::f_stoich();
        let alpha = far / far_p; // fraction of the air in the primary
        assert!(
            alpha <= 1.0 + 1e-9,
            "primary air fraction α={alpha:.4} > 1 — overall mixture leaner than the primary"
        );

        let t_p = primary_aft(far_p, p, tt3, hf_fuel);
        let comp_p = equilibrium_composition(far_p, t_p, p);
        // RUNG 19 — super_eq_o=false, prompt=None ⇒ m=1, no prompt ⇒ the integrator call is
        // byte-identical to the prior rung. Super-eq O is a T-driven m(T_p)≈1.16–1.50× on the
        // primary [O]; prompt is the imposed De Soete φ-bump at phi_primary.
        let m_p = if o.super_eq_o { super_eq_o_multiplier(t_p) } else { 1.0 };
        assert!(
            (1.0..=2.0).contains(&m_p),
            "super-eq O multiplier m={m_p:.3} at primary T={t_p:.0} K outside the flame-band \
             bound [1,2] — the Westenberg partial-eq/eq ratio is a FLAME model (T≳1500 K)"
        );
        let mut nox = thermal_no(&comp_p, t_p, p, o.tau, far_p, o.nsteps, m_p);
        let ei_no_prompt = match o.prompt {
            Some(pr) => {
                pr.validate();
                pr.ei_prompt(phi_primary, t_p)
            }
            None => 0.0,
        };
        nox.ei_no_prompt = ei_no_prompt;
        // SUMMED trace guard (rung 19): primary thermal (m-lifted) + prompt must stay trace.
        let x_no_prompt_p =
            if nox.ei_no > 0.0 { ei_no_prompt / nox.ei_no * nox.x_no } else { 0.0 };
        assert!(
            nox.x_no + x_no_prompt_p < 0.02,
            "summed primary NO not trace (x_NO_thermal+prompt={:.4e}) — decoupling invalid",
            nox.x_no + x_no_prompt_p
        );

        let t_mix = mixed_out_t(&comp_p, t_p, alpha, far, tt3, p);
        // Standing conservation gate (LOOSE gross-error bound — the method does not know η_b,
        // and a frozen-majors mix-out lands only ~40 K off here, WITHIN this band; the sharp
        // split-independence and frozen-vs-re-equilibrated discriminators live in the tests,
        // which ARE what catch that bug): the re-equilibrated mix-out must return to ≈ Tt4.
        assert!(
            (t_mix - tt4).abs() < 0.05 * tt4,
            "mix-out T {t_mix:.1} K did not return to Tt4={tt4} K (re-equilibration gate)"
        );

        // Freeze the NO moles through dilution: NO moles per mol PRIMARY air = x_no·ntot_p;
        // scale to the total-air basis by α; divide by the mixed pool's total moles per air.
        let ntot_p: f64 = comp_p.iter().map(|&(_, v)| v).sum();
        let n_no_total = alpha * nox.x_no * ntot_p;
        let ntot_mix: f64 =
            equilibrium_composition(far, t_mix, p).iter().map(|&(_, v)| v).sum();
        let x_no_mix = n_no_total / ntot_mix;

        ZonedNoxState {
            phi_primary,
            far_primary: far_p,
            alpha,
            t_primary: t_p,
            t_mix,
            primary: nox,
            x_no_mix,
            super_eq_o: o.super_eq_o,
            o_multiplier: m_p,
            prompt: o.prompt,
            ei_no_prompt,
        }
    }
}
