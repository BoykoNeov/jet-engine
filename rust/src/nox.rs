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
//! - **rung 10** — the quench takes TIME. As the dilution air mixes in, the local mixture
//!   sweeps through STOICHIOMETRIC, so a rich primary's temperature RISES through the NO-bell
//!   peak and the same Zeldovich rate RE-MAKES the NO it avoided. `docs/rung10-spec.md`.
//! - **rung 11** — the quench time stops being a free knob: [`JetMixing`] DERIVES it from the
//!   jet momentum-flux ratio J, with an entrainment schedule replacing rung 10's linear one.
//!   Mean-field, so EI falls MONOTONICALLY in J — no mixing optimum. `docs/rung11-spec.md`.
//! - **rung 12** — [`Unmixedness`] splits the flow into a mean-field bulk and an under-mixed
//!   core that lingers, and the NO-vs-J curve TURNS BACK UP: the Holdeman optimum, recovered
//!   and pinned AT `C_opt`. `docs/rung12-spec.md`.
//! - **rung 20** — rung 19's super-equilibrium [O] threaded THROUGH the quench, closing the
//!   seam where the finite-quench numbers still rode on equilibrium O. `docs/rung20-spec.md`.
//! - **rung 13** — [`MixingPdf`] replaces rung-12's two lumps with a CONTINUOUS mean-preserving
//!   β-PDF of mixture fraction over the ideal bell. A MECHANISM SEPARATION: composition variance
//!   pins the optimum's LOCATION, and rung-12's dwell made the climb. `docs/rung13-spec.md`.
//! - **rung 15** — [`QuenchPdf`] carries that β-PDF THROUGH the quench, so the two mechanisms
//!   COMBINE: a FINITE floor at `C_opt`, and the far flank climbs again. `docs/rung15-spec.md`.
//! - **rung 16** — [`PocketQuenchPdf`] retires rung-15's linearisation by quenching EACH pocket
//!   separately, so the dwell acts inside the cooling chemistry and term 2 goes SUBLINEAR.
//!   `docs/rung16-spec.md`.
//! - **rung 18** — [`TransportedPdf`] derives the width from a variance-decay ODE instead of the
//!   kink. Load-bearing result NEGATIVE: a 0-D transport cannot derive the optimum's location.
//!   `docs/rung18-spec.md`.
//! - **rung 21** — the super-equilibrium [O] threaded through the ideal-bell integrals too, which
//!   discharges rung 20's forbid guard. Shape-preserving. `docs/rung21-spec.md`.
//!
//! # What the port had to be careful about here
//!
//! Every composition is an ORDERED slice, never a map, because Python sums its dicts in
//! insertion order and float addition is not associative. Through rung 20 every composition
//! still comes from [`gas::equilibrium_composition`] or [`gas::air_mole_fractions`], so no new
//! ordering is introduced anywhere in this file.
//!
//! Slice A predicted the trap would fire in rung 10's quench trajectory. **It does not, and
//! the check is worth recording rather than repeating**: the trajectory builds a record of
//! SCALARS read by field ([`QuenchPoint`]), rung 12's two-stream split is a scalar mass
//! weighting, and the only composition either of them sums is the equilibrium solver's own
//! output. The Python sites that DO assemble a composition by hand sit in the nozzle marches
//! (`gas.py:1963` / `gas.py:2255`) — rungs 25/26, which is phase 4's problem, not this file's.
//!
//! The Zeldovich integrator is fixed-step RK4 with no adaptive control, so it carries no
//! stopping rule. What it does carry is 4000 accumulations, and `c += dt/6.0 * (…)` is a
//! different function in the last bit from `(…) * dt/6.0`. The two bisections
//! ([`primary_aft`], [`mixed_out_t`]) are where the stopping rules live, and their inner
//! evaluation is the 8-species Newton — the deepest solver nesting in the project. Rung 10
//! wraps a THIRD loop around that pair: one trajectory is `ngrid` mix-out bisections, each a
//! bisection over a Newton.

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
    let t = primary_aft_raw(far_p, p, t_air, hf_fuel);
    // Bracket guard (post-loop, not an endpoint eval — `equilibrium_composition` DIVERGES at
    // the cold 800 K edge, so we cannot probe there; a root outside [800,3200] instead pins
    // the bisection against an edge, which this catches). Any real flame sits well inside.
    assert!(
        801.0 < t && t < 3199.0,
        "primary_aft: flame temp {t:.1} K pinned at [800,3200] K bracket edge (far_p={far_p:.4})"
    );
    t
}

/// [`primary_aft`] with the bracket guard as a `None` instead of a panic — **the flammability
/// limit as a BRANCH** (rung 13).
///
/// The ideal bell is sampled from ξ≈0 upward, and its cold end is not a small number: below the
/// flammability limit the bisection pins against its 800 K edge and there is no flame at all.
/// Python expresses that as `try: _primary_aft(...) except AssertionError: return 0.0`, which is
/// a *catch*, so nothing in the Python distinguishes "no flame" from "some other assertion in
/// the equilibrium solver". Rust cannot catch a panic and should not try; this splits the guard
/// out instead, which is narrower than Python's `except` by exactly the solver's own asserts.
///
/// That difference is MEASURED rather than assumed: the oracle dumps the index of the first
/// burnable node on every bell grid it builds (`bell/*/first_burnable`), so if the two languages
/// ever took the zero branch a different number of times, the gate names the grid. Measured on
/// the shipped grids: 1 node at the subsonic design point, 0 at the hotter supersonic one — so
/// both the taken and the never-taken case are covered.
pub fn try_primary_aft(far_p: f64, p: f64, t_air: f64, hf_fuel: f64) -> Option<f64> {
    let t = primary_aft_raw(far_p, p, t_air, hf_fuel);
    if 801.0 < t && t < 3199.0 {
        Some(t)
    } else {
        None
    }
}

/// The bisection itself, shared by [`primary_aft`] and [`try_primary_aft`] so the two cannot
/// drift apart — they differ ONLY in what they do with a pinned bracket.
fn primary_aft_raw(far_p: f64, p: f64, t_air: f64, hf_fuel: f64) -> f64 {
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
    0.5 * (lo + hi)
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

// --------------------------------------------------------------------------------------
// RUNG 10 — the FINITE-RATE quench (secondary-zone Zeldovich in the cooling gas).
//
// Rung 9's mix-out is the IDEAL (infinitely-fast) quench: NO frozen at the primary value.
// Here the quench air is added over a finite time, so the LOCAL mixture sweeps far_p → far_ov
// — through STOICHIOMETRIC for a rich primary — and a rich primary's temperature RISES through
// the NO-bell peak on the way down, RE-MAKING the NO it avoided. `docs/rung10-spec.md`.
// --------------------------------------------------------------------------------------

/// One point on the fast-chemistry dilution trajectory, at dilution fraction β.
///
/// A record of SCALARS, read by field and never summed — which is why the composition-ORDER
/// hazard the port plan flagged for this slice does not actually fire here (see the module
/// header). Every composition on this path still comes from [`equilibrium_composition`].
#[derive(Debug, Clone, Copy)]
pub struct QuenchPoint {
    /// mol current-air per mol total-FINAL-air, `a(β) = α + β(1−α)`
    pub a: f64,
    /// local instantaneous-equilibrium temperature, K
    pub t: f64,
    /// [O], mol/m³ (the rung-6 equilibrium pool's; rung 20 lifts it INSIDE the integrator)
    pub c_o: f64,
    /// [N₂], mol/m³
    pub c_n2: f64,
    /// [H], mol/m³
    pub c_h: f64,
    /// [NO]_e, mol/m³ — the thermodynamic ceiling, untouched by rung 20
    pub c_noe: f64,
    /// moles per mol CURRENT air
    pub ntot_local: f64,
    /// pool volume on the FINAL (total-air) basis, m³ — so extensive NO moles ↔ concentration
    pub v: f64,
}

/// Fast-chemistry dilution trajectory for the finite quench (rung 10).
///
/// The quench is resolved in a parameter β ∈ [0,1] (dilution fraction). The air present at β
/// is `a(β) = α + β(1−α)` mol per mol TOTAL(final) air, so the LOCAL fuel/air ratio
/// `far_local = far_ov/a` sweeps far_p → far_ov — through STOICHIOMETRIC for a rich primary
/// (the peak of the NO bell). At each β the majors + T are instantaneous equilibrium (the
/// rung-8 re-equilibrating [`mixed_out_t`] on the CURRENT air, basis 1 mol current air), so
/// [O], [N₂], [H], [NO]_e and T are functions of β ALONE; NO is the one SLOW variable,
/// integrated separately by [`quench_no`].
///
/// The rung-7 K-check + trace guards bind along the WHOLE trajectory (every T the quench
/// visits), not just the primary — the transcribed rates stay tied to the a6/a7 thermo.
///
/// **This is the expensive object in the slice** — every point re-equilibrates the diluting
/// majors through a bisection whose inner evaluation is the 8-species Newton. It takes no
/// `tau_q` and no jet config, so ONE trajectory serves an entire τ_q sweep, an entire J sweep,
/// and rung 12's bulk/core pair; [`quench_no`] accepts it prebuilt for exactly that reason.
pub fn quench_trajectory(
    comp_prim: &[(&str, f64)],
    t_prim: f64,
    alpha: f64,
    far_ov: f64,
    t_dilution: f64,
    p: f64,
    ngrid: usize,
) -> Vec<QuenchPoint> {
    let mut tab = Vec::with_capacity(ngrid);
    for i in 0..ngrid {
        let b = i as f64 / (ngrid - 1) as f64;
        let a = alpha + b * (1.0 - alpha); // mol current-air / mol total-final-air
        let far_local = far_ov / a; // far_p (β=0) → far_ov (β=1)
        let alpha_local = alpha / a; // fraction of CURRENT air that is primary
        let t_local = mixed_out_t(comp_prim, t_prim, alpha_local, far_local, t_dilution, p);
        let comp_local = equilibrium_composition(far_local, t_local, p);
        let ntot_local: f64 = comp_local.iter().map(|&(_, v)| v).sum(); // per mol current-air
        let conc = p / (RU * t_local); // total molar conc, mol/m³
        let c_o = maybe(&comp_local, "O") / ntot_local * conc;
        let c_n2 = need(&comp_local, "N2") / ntot_local * conc;
        let c_h = maybe(&comp_local, "H") / ntot_local * conc;
        let x_no_e = equilibrium_no_fraction(&comp_local, t_local);
        let c_noe = x_no_e * conc;
        let v = a * ntot_local * RU * t_local / p; // volume on the FINAL (total-air) basis
        let kr = kcheck_ratio(t_local); // K-check binds at EVERY trajectory T
        assert!(
            0.90 < kr && kr < 1.15,
            "quench K-check off: ratio {kr:.4} at T={t_local:.1}"
        );
        assert!(
            x_no_e < 0.02,
            "NO not trace on quench path (x_NO_e={x_no_e:.4e}) at T={t_local:.1}"
        );
        tab.push(QuenchPoint { a, t: t_local, c_o, c_n2, c_h, c_noe, ntot_local, v });
    }
    tab
}

/// Linear interpolation on the trajectory at time-fraction `tfrac`, Python's `interp` exactly.
///
/// `int(x)` truncates toward zero and the index is clamped to `len−2`, so `tfrac = 1` lands on
/// the last interval with `w = 1` rather than running off the end. Rust's `as usize` also
/// truncates toward zero (and saturates at 0 below it), which is the same function on this
/// domain — β never leaves [0,1].
fn interp(tab: &[QuenchPoint], tfrac: f64, get: impl Fn(&QuenchPoint) -> f64) -> f64 {
    let x = tfrac * (tab.len() - 1) as f64;
    let i = (x as usize).min(tab.len() - 2);
    let w = x - i as f64;
    get(&tab[i]) * (1.0 - w) + get(&tab[i + 1]) * w
}

/// What [`quench_no`] returns — Python's result dict, as a struct.
#[derive(Debug, Clone, Copy)]
pub struct QuenchResult {
    /// EI_NO re-made along the finite quench, g NO / kg fuel
    pub ei: f64,
    /// NO mole fraction frozen at the end of the quench
    pub x_no_mix: f64,
    /// NO moles per mol total-final-air
    pub n_no: f64,
    /// peak T along the path, K — `> T_primary` for a RICH primary (the smoking gun)
    pub t_peak: f64,
    /// max `[NO]/[NO]_e` reached; `< 1` ⇒ the dropped equilibrium clamp is dormant
    pub max_a: f64,
}

/// Numerical + closure knobs for [`quench_no`] — Python's keyword tail.
pub struct QuenchOpts<'a> {
    /// RK4 steps in REAL time
    pub nsteps: usize,
    /// trajectory points, when one has to be built
    pub ngrid: usize,
    /// a PREBUILT trajectory; `None` builds one (the τ_q-independence lever)
    pub tab: Option<&'a [QuenchPoint]>,
    /// RUNG 11 — the β(t/τ_q) entrainment schedule; `None` is the identity ⇒ rung 10
    pub schedule: Option<&'a dyn Fn(f64) -> f64>,
    /// RUNG 20 — lift [O] by m(T) along the cooling path; `false` ⇒ bit-for-bit rung 10/11
    pub super_eq_o: bool,
}

impl Default for QuenchOpts<'_> {
    fn default() -> Self {
        Self { nsteps: 2000, ngrid: 240, tab: None, schedule: None, super_eq_o: false }
    }
}

/// Finite-rate quench NO integrator (rung 10; schedule-aware for rung 11). CLAMP-FREE.
///
/// Integrates the extended-Zeldovich rate (the SAME reverse-rate one-equation form as
/// [`thermal_no`]) along the [`quench_trajectory`] cooling/mixing path, starting from the
/// primary's kinetic NO. Two differences from [`thermal_no`], both load-bearing:
///
/// * **NO is EXTENSIVE** — moles per mol total-final-air. Mixing dilution air changes the
///   volume `V(β)` but conserves NO moles; only chemistry changes them. So this integrates
///   `dn_NO/dt = rate([NO] = n_NO/V)·V`.
/// * **The `cNO ≤ cNOe` CAP IS DROPPED.** On a cooling path NO is legitimately
///   super-equilibrium and frozen (Heywood); the cap would delete exactly that NO — a
///   plausible-but-wrong low number with the asserts still green. The `(1−a²)` factor already
///   goes NEGATIVE when `a > 1` (super-eq NO decomposes) and the Arrhenius constants freeze it
///   out as T falls, so the form self-limits. This is a SEPARATE integrator; [`thermal_no`]
///   stays byte-identical, because its rung-6..9 reduce gates depend on its exact capped RK4
///   trajectory. `docs/rung10-spec.md` § the clamp trap.
///
/// A slow quench dwells near the stoichiometric crossing (the NO-bell peak) and RE-MAKES the
/// NO a rich primary avoided; a fast quench escapes past the peak — the RQL hazard.
///
/// RUNG 11 — `schedule` decouples the DILUTION FRACTION β from time. Rung 10's `β = t/τ_q` is
/// linear; a physical jet-entrainment schedule remaps it, `β = schedule(t/τ_q)`. The
/// trajectory is indexed on β but `dt` still steps in REAL time for the Zeldovich accumulation
/// — conflating the two (which rung 10 got away with only because its schedule was the
/// identity) would silently reproduce rung-10 behaviour under a rung-11 label.
///
/// **The time variable ACCUMULATES** (`t += dt`), and `i as f64 * dt` is a different number
/// after 2000 steps. Likewise `(t + 0.5·dt)/tau_q` is not `t/tau_q + 0.5·dt/tau_q`.
pub fn quench_no(
    comp_prim: &[(&str, f64)],
    t_prim: f64,
    alpha: f64,
    far_ov: f64,
    t_dilution: f64,
    p: f64,
    n_no_initial: f64,
    tau_q: f64,
    o: QuenchOpts<'_>,
) -> QuenchResult {
    let built;
    let tab: &[QuenchPoint] = match o.tab {
        Some(t) => t,
        None => {
            built =
                quench_trajectory(comp_prim, t_prim, alpha, far_ov, t_dilution, p, o.ngrid);
            &built
        }
    };

    let mut max_a = 0.0f64; // max [NO]/[NO]_e over the path: <1 ⇒ clamp dormant

    // `max_a` is updated AFTER the `cNOe <= 0` early return and on ALL FOUR RK4 trial states,
    // including `n_no + dt·k3` — rung 10's dormancy gate reads it, so the update points are
    // part of the contract, not bookkeeping.
    let dn_dt = |tfrac: f64, n_no: f64, max_a: &mut f64| -> f64 {
        let c_noe = interp(tab, tfrac, |r| r.c_noe);
        if c_noe <= 0.0 {
            return 0.0;
        }
        let t = interp(tab, tfrac, |r| r.t);
        let v = interp(tab, tfrac, |r| r.v);
        let mut c_o = interp(tab, tfrac, |r| r.c_o);
        let c_n2 = interp(tab, tfrac, |r| r.c_n2);
        let c_h = interp(tab, tfrac, |r| r.c_h);
        if o.super_eq_o {
            // RUNG 20 — lift [O] by m(T) INSIDE the re-making (the deferred rung-19 seam: the
            // finite-quench NO rode on equilibrium O). m multiplies c_o, so it scales R1
            // (formation) AND R2 (reverse) alike, exactly as `thermal_no`'s o_multiplier does
            // on the primary. Floor T at the flame band — m diverges as T → 0 on the cool tail.
            let m = super_eq_o_multiplier(t.max(SUPER_EQ_T_FLOOR));
            assert!(
                (1.0..=2.0).contains(&m),
                "quench super-eq O multiplier m={m:.3} at T={t:.0} K outside [1,2] — the \
                 Westenberg partial-eq closure is a flame model (floored at T≳1500 K)"
            );
            c_o *= m;
        }
        let r1 = k_zeldovich("1f", t) * c_o * c_n2;
        let r2 = k_zeldovich("2r", t) * c_noe * c_o;
        let r3 = k_zeldovich("3r", t) * c_noe * c_h;
        let beta = if (r2 + r3) > 0.0 { r1 / (r2 + r3) } else { 0.0 };
        let a = (n_no / v) / c_noe;
        if a > *max_a {
            *max_a = a;
        }
        2.0 * r1 * (1.0 - a * a) / (1.0 + beta * a) * v // d(n_NO)/dt = rate·V
    };

    // β↔time map: identity (β = t/τ_q, rung-10 linear) unless a rung-11 mixing schedule remaps
    // it. With `schedule: None` the calls below are byte-identical to rung 10.
    let identity = |x: f64| x;
    let sched: &dyn Fn(f64) -> f64 = o.schedule.unwrap_or(&identity);

    let mut n_no = n_no_initial;
    let dt = tau_q / o.nsteps as f64;
    let mut t = 0.0f64;
    for _ in 0..o.nsteps {
        let b1 = sched((t / tau_q).min(1.0));
        let b2 = sched(((t + 0.5 * dt) / tau_q).min(1.0));
        let b3 = sched(((t + dt) / tau_q).min(1.0));
        let k1 = dn_dt(b1, n_no, &mut max_a);
        let k2 = dn_dt(b2, n_no + 0.5 * dt * k1, &mut max_a);
        let k3 = dn_dt(b2, n_no + 0.5 * dt * k2, &mut max_a);
        let k4 = dn_dt(b3, n_no + dt * k3, &mut max_a);
        n_no += dt / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
        if n_no < 0.0 {
            n_no = 0.0; // guard negatives ONLY (no equilibrium cap)
        }
        t += dt;
    }

    let last = tab[tab.len() - 1];
    let ntot_mix = last.a * last.ntot_local; // per mol total-final-air (= ntot(far_ov, T_mix))
    let x_no_mix = n_no / ntot_mix;
    let n_fuel = far_ov * m_air() / M_CH2;
    let ei = if n_fuel > 0.0 {
        1000.0 * (n_no * m_no()) / (n_fuel * M_CH2_KG)
    } else {
        0.0
    };
    // Python's `max(r["T"] for r in tab)`: a left fold that keeps the FIRST maximum, so a
    // lean/stoich (monotone-falling) trajectory reports `tab[0].t` exactly.
    let mut t_peak = tab[0].t;
    for r in &tab[1..] {
        if r.t > t_peak {
            t_peak = r.t;
        }
    }
    QuenchResult { ei, x_no_mix, n_no, t_peak, max_a }
}

// --------------------------------------------------------------------------------------
// RUNGS 13 / 15 / 16 / 18 / 21 — the mixture-fraction PDF family.
//
// Rung 12 parameterised the segregation as TWO LUMPS: a mean-field bulk and an under-mixed
// core, mass-weighted by a kinked `w(C)`. Rungs 13-18 replace that with a CONTINUOUS
// distribution — a mean-preserving β-PDF of mixture fraction ξ = far/(1+far), integrated
// against the ideal primary bell EI(φ) — and then ask, one rung at a time, what the width of
// that distribution should be and what the pockets inside it should be carried through.
//
//   13  the bell integrated against a β-PDF whose width is the SAME kinked `g(C)`
//   15  that integral SCALED by rung-12's dwell (a linearisation: exact only while EI ∝ τ)
//   16  each rich pocket through its OWN quench instead, so the dwell acts inside the cooling
//   18  the width from a variance-DECAY ODE off a DERIVED ceiling, instead of the kink
//   21  rung 19's super-equilibrium [O] threaded through ALL of the above
//
// The load-bearing arithmetic is `beta_pdf_nodes_weights`, and it is REGIME-SWITCHING: the
// shape parameter `a = ξ̄(1/g − 1)` crosses 1 as the width grows, and the quadrature changes
// scheme at exactly that crossing. A port that gets one branch right and the other wrong looks
// correct across most of a J sweep and wrong in a narrow band around the optimum.
// --------------------------------------------------------------------------------------

/// Rung-9 IDEAL primary EI_NO (g NO/kg fuel) at a LOCAL fuel/air ratio — the bell EI(φ) the
/// rung-13 PDF closure samples.
///
/// Runs the same primitives as the zoned primary ([`primary_aft`] → [`equilibrium_composition`]
/// → [`thermal_no`]) at the local φ. Returns 0 outside the valid window: φ>2 (the soot bound —
/// the 5-species basis is invalid AND the O-starved pool makes ≈0 NO anyway) or too lean to
/// burn (see [`try_primary_aft`]). No finite quench here — this ISOLATES composition variance
/// on the ideal bell; carrying it through the finite quench is rung 15's seam.
///
/// RUNG 20/21 — `super_eq_o` lifts this local bell's [O] by `m(T_p)`. `false` ⇒ the rung-13/15/18
/// integrals are UNTOUCHED, staying equilibrium-O lower bounds. `docs/rung21-spec.md`.
pub fn ideal_bell_ei(
    far_local: f64,
    p: f64,
    tt3: f64,
    hf_fuel: f64,
    tau: f64,
    super_eq_o: bool,
) -> f64 {
    let phi = far_local / gas::f_stoich();
    if far_local <= 0.0 || phi > 2.0 + 1e-9 {
        return 0.0;
    }
    let Some(t_p) = try_primary_aft(far_local, p, tt3, hf_fuel) else {
        return 0.0; // too lean to burn (cold-bracket-edge flame)
    };
    let comp = equilibrium_composition(far_local, t_p, p);
    let m = if super_eq_o {
        super_eq_o_multiplier(t_p.max(SUPER_EQ_T_FLOOR))
    } else {
        1.0
    };
    thermal_no(&comp, t_p, p, tau, far_local, 4000, m).ei_no
}

/// Regime-aware, mean-preserving quadrature of a β-PDF of mixture fraction ξ (rung 13).
///
/// Mean ξ̄, normalized variance (segregation) `g ∈ (0,1)`: `σ² = g·ξ̄(1−ξ̄)`, shape parameters
/// `a = ξ̄(1/g − 1)`, `b = (1−ξ̄)(1/g − 1)`. Returns (nodes ξᵢ, normalized weights wᵢ).
///
/// **The two regimes are different integration schemes, and the switch is at `a = 1`.**
/// A LEAN mean gives `a < 1`, so `P_β ∝ ξ^(a−1)` has an integrable SINGULARITY at ξ→0 that a
/// naive uniform-in-ξ midpoint rule mis-weights (⟨ξ⟩ drifts off ξ̄ and the integral never
/// converges). The fix substitutes `u = ξ^a` — uniform in u, the Jacobian cancelling `ξ^(a−1)`
/// EXACTLY and leaving the bounded weight `(1−ξ)^(b−1)`, `b ≥ 1`. For `a ≥ 1` (near-delta, no
/// singularity) it windows a uniform-in-ξ grid over `ξ̄ ± 8σ` instead — CENTERED on the mass,
/// because as g→0 the peak narrows to a sliver near ξ̄ and a `[0, …]` window would mis-resolve it.
///
/// **The mean and variance are ASSERTED against their targets — that check is the deliverable
/// more than the number is.** Both guards are live constraints on the caller, not decoration:
///
/// * `b ≥ 1` caps the width at `g ≤ (1−ξ̄)/(2−ξ̄)` — 0.493 at the shipped lean mean.
/// * the 1 % mean-preservation bar is `n_quad`-SENSITIVE, which the oracle measured rather than
///   assumed: at a lean mean it REJECTS `g = 0.026` (the first point past the `a = 1` switch)
///   and `g = 0.40` for every `n_quad ≤ 100`, and accepts both from 112 up. The Python's own
///   gate samples `n_quad = 160`, so nothing there could see it.
///
/// Three spellings here are not interchangeable, and each is the opposite of its neighbour:
/// `powp` for the COMPUTED exponent `1/a`, `.sqrt()` for σ (the sqrt instruction, NOT `powp`),
/// and `d * d` for the variance check's `** 2` (an integer literal, which PyPy rewrites into a
/// multiply). `(hi − lo)·(i + 0.5)/n` is `((hi−lo)·(i+0.5))/n`, not `(hi−lo)·((i+0.5)/n)`.
pub fn beta_pdf_nodes_weights(xibar: f64, g_seg: f64, n_quad: usize) -> (Vec<f64>, Vec<f64>) {
    let inv = 1.0 / g_seg - 1.0;
    let (a, b) = (xibar * inv, (1.0 - xibar) * inv);
    assert!(
        a > 0.0 && b >= 1.0,
        "β-PDF shape (a={a:.3}, b={b:.3}) outside a>0,b≥1 — the quadrature needs a non-singular \
         (1−ξ) tail (b≥1 holds for a lean mean until g≈0.49, well past g_max)."
    );
    let n = n_quad as f64;
    let (nodes, logw): (Vec<f64>, Vec<f64>) = if a < 1.0 {
        // singular lean-mean regime — u = ξ^a cancels ξ^(a−1)
        let nodes: Vec<f64> =
            (0..n_quad).map(|i| powp((i as f64 + 0.5) / n, 1.0 / a)).collect();
        let logw = nodes.iter().map(|&x| (b - 1.0) * (1.0 - x).ln()).collect();
        (nodes, logw)
    } else {
        // near-delta (a ≥ 1): bounded density, so CENTER the window on the mass
        let sigma = (g_seg * xibar * (1.0 - xibar)).sqrt();
        let lo = 1e-12f64.max(xibar - 8.0 * sigma);
        let hi = (1.0 - 1e-12f64).min(xibar + 8.0 * sigma);
        let nodes: Vec<f64> =
            (0..n_quad).map(|i| lo + (hi - lo) * (i as f64 + 0.5) / n).collect();
        let logw = nodes
            .iter()
            .map(|&x| (a - 1.0) * x.ln() + (b - 1.0) * (1.0 - x).ln())
            .collect();
        (nodes, logw)
    };
    let m = logw.iter().fold(f64::NEG_INFINITY, |acc, &v| acc.max(v));
    let ww: Vec<f64> = logw.iter().map(|&l| (l - m).exp()).collect();
    let s: f64 = ww.iter().fold(0.0, |acc, &v| acc + v);
    let w: Vec<f64> = ww.iter().map(|&x| x / s).collect();

    // mean-preservation gate (THE deliverable): the closure must integrate at the specified mean
    let mean_xi = w.iter().zip(nodes.iter()).fold(0.0, |acc, (&wi, &x)| acc + wi * x);
    let var_xi = w.iter().zip(nodes.iter()).fold(0.0, |acc, (&wi, &x)| {
        let d = x - xibar;
        acc + wi * (d * d)
    });
    let var_tgt = g_seg * xibar * (1.0 - xibar);
    assert!(
        (mean_xi - xibar).abs() <= 0.01 * xibar,
        "β-PDF quadrature drifted the mean: ⟨ξ⟩={mean_xi:.6} vs ξ̄={xibar:.6} (>1%) — the \
         mean-preserving closure must integrate at ξ̄ (raise n_quad: the bar needs ≥112 near \
         the a=1 switch and at the top of the g range)."
    );
    assert!(
        (var_xi - var_tgt).abs() <= 0.05 * var_tgt,
        "β-PDF quadrature variance off target: {var_xi:.3e} vs {var_tgt:.3e} (>5%)."
    );
    (nodes, w)
}

/// The IDEAL primary bell EI(ξ) on a fixed fine ξ-grid, with a linear interpolator.
///
/// Python returns a closure over its two reference arrays; Rust returns the arrays, which is
/// the same object with the capture made visible. Built ONCE and reused: the bell is
/// equilibrium-heavy (every node an AFT bisection plus a 4000-step RK4) and depends on
/// **neither `g` nor `J`**, so one bell serves an entire segregation sweep and an entire jet
/// sweep. That is the sizing lever this whole slice rests on.
#[derive(Debug, Clone)]
pub struct Bell {
    /// the reference ξ grid, from ~0 up to the φ=2 soot bound
    pub xi_ref: Vec<f64>,
    /// EI_NO at each reference ξ, g NO/kg fuel (0 below the flammability limit)
    pub ei_ref: Vec<f64>,
}

impl Bell {
    /// ξ ↦ EI, linear between reference nodes. Three branches, all of them reachable: below the
    /// first node the bell is flat, at or beyond the last it is 0 (past φ=2, soot-rich), and
    /// between them a binary search plus a lerp.
    pub fn at(&self, xi: f64) -> f64 {
        let n = self.xi_ref.len();
        if xi <= self.xi_ref[0] {
            return self.ei_ref[0];
        }
        if xi >= self.xi_ref[n - 1] {
            return 0.0; // beyond φ=2 (soot-rich) ⇒ EI≈0
        }
        let (mut lo, mut hi) = (0usize, n - 1);
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if self.xi_ref[mid] <= xi {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let t = (xi - self.xi_ref[lo]) / (self.xi_ref[hi] - self.xi_ref[lo]);
        self.ei_ref[lo] + t * (self.ei_ref[hi] - self.ei_ref[lo])
    }
}

/// ξ at the φ=2 soot bound — the top of every ξ-grid in this family.
pub fn xi_soot_bound() -> f64 {
    (2.0 * gas::f_stoich()) / (1.0 + 2.0 * gas::f_stoich())
}

/// Build the ideal bell (rung 13; rung 21's `super_eq_o` lifts every node).
///
/// The grid is `xi_max·(i + 0.5)/n_bell` — `(xi_max·(i+0.5))/n_bell`, left to right, and NOT
/// `xi_max·((i+0.5)/n_bell)`. Three grid formulas in this family have three different shapes;
/// transcribing them by eye is how one of them ends up a bit off.
pub fn bell_interpolator(
    p: f64,
    tt3: f64,
    hf_fuel: f64,
    tau: f64,
    n_bell: usize,
    super_eq_o: bool,
) -> Bell {
    let xi_max = xi_soot_bound();
    let xi_ref: Vec<f64> =
        (0..n_bell).map(|i| xi_max * (i as f64 + 0.5) / n_bell as f64).collect();
    let ei_ref = xi_ref
        .iter()
        .map(|&x| ideal_bell_ei(x / (1.0 - x), p, tt3, hf_fuel, tau, super_eq_o))
        .collect();
    Bell { xi_ref, ei_ref }
}

/// `⟨EI⟩ = ∫₀¹ EI_bell(φ(ξ))·P_β(ξ; ξ̄, g) dξ` on a PREBUILT bell — the hoisted form.
///
/// This is [`pdf_mean_ei`] with the bell lifted out of the call, which is what makes a sweep
/// affordable, and it is what the Python's own rung-13/21 tests use. **One documented
/// difference from the wrapper:** the `g → 0` delta short-circuit returns the INTERPOLANT at ξ̄,
/// where production returns the EXACT [`ideal_bell_ei`]. The Python has the same split and its
/// `test_reduce_g_to_zero_is_well_mixed_point_value` is careful about which one it compares to.
pub fn pdf_mean_ei_on_bell(bell: &Bell, xibar: f64, g_seg: f64, n_quad: usize) -> f64 {
    if g_seg <= 1e-9 {
        return bell.at(xibar); // delta ⇒ well-mixed point value
    }
    let (nodes, w) = beta_pdf_nodes_weights(xibar, g_seg, n_quad);
    w.iter().zip(nodes.iter()).fold(0.0, |acc, (&wi, &x)| acc + wi * bell.at(x))
}

/// `⟨EI⟩` over the mean-preserving β-PDF of the ideal bell — the rung-13 closure, and the one
/// rungs 18 and 22 reuse verbatim with only the SOURCE of `g` changed.
///
/// A β-PDF of mixture fraction at the OVERALL mean `ξ̄ = far/(1+far)`; `g → 0` is a delta at ξ̄,
/// i.e. the well-mixed point value. Builds the bell (the expensive part) and integrates it
/// against the regime-aware quadrature.
///
/// RUNG 21 — `super_eq_o` threads the Westenberg m(T) lift through BOTH the delta short-circuit
/// AND the built bell, so the reduce stays consistent in the limit.
#[allow(clippy::too_many_arguments)]
pub fn pdf_mean_ei(
    far_overall: f64,
    tt3: f64,
    p: f64,
    hf_fuel: f64,
    tau: f64,
    g_seg: f64,
    n_bell: usize,
    n_quad: usize,
    super_eq_o: bool,
) -> f64 {
    let xibar = far_overall / (1.0 + far_overall);
    if g_seg <= 1e-9 {
        // delta ⇒ well-mixed point value, from the EXACT bell rather than the interpolant
        return ideal_bell_ei(far_overall, p, tt3, hf_fuel, tau, super_eq_o);
    }
    let bell = bell_interpolator(p, tt3, hf_fuel, tau, n_bell, super_eq_o);
    let (nodes, w) = beta_pdf_nodes_weights(xibar, g_seg, n_quad);
    w.iter().zip(nodes.iter()).fold(0.0, |acc, (&wi, &x)| acc + wi * bell.at(x))
}

/// Rung 16's per-pocket bank: EI for every pocket on the ξ-grid, at ONE dwell.
///
/// **This is the slice's expensive object and its own sizing lever.** Each rich-of-mean pocket
/// is a full [`quench_no`] that builds its OWN mix-out trajectory — no `tab` sharing is
/// possible, because every pocket sits at its own `far_local` with its own `alpha`. But the
/// bank depends on `tau_core` and **not on `g_seg`**, which enters nowhere before the final
/// β-quadrature. So the Python's monolithic function is split in two here: build the bank once
/// per dwell, then [`pocket_quench_integrate`] over as many widths as you like for free. The
/// Python cannot do that and rebuilds 24 quenches per call.
#[derive(Debug, Clone)]
pub struct PocketGrid {
    /// the ξ grid, up to the φ=2 soot bound
    pub xi_grid: Vec<f64>,
    /// EI at each pocket, g NO/kg fuel
    pub vals: Vec<f64>,
    /// max `[NO]/[NO]_e` over every pocket's quench; `< 1` ⇒ the dropped clamp stayed dormant
    pub max_a: f64,
}

impl PocketGrid {
    /// ξ ↦ EI over the pocket bank — the same three-branch interpolator as [`Bell::at`], on a
    /// different pair of arrays. (Kept as its own body rather than shared, because the Python
    /// writes it twice too and the φ>2 tail is a rung-15-scope decision, not a lerp detail.)
    pub fn at(&self, xi: f64) -> f64 {
        let n = self.xi_grid.len();
        if xi <= self.xi_grid[0] {
            return self.vals[0];
        }
        if xi >= self.xi_grid[n - 1] {
            return 0.0; // φ>2 tail: rung-15 soot-bound scope
        }
        let (mut lo, mut hi) = (0usize, n - 1);
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if self.xi_grid[mid] <= xi {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let t = (xi - self.xi_grid[lo]) / (self.xi_grid[hi] - self.xi_grid[lo]);
        self.vals[lo] + t * (self.vals[hi] - self.vals[lo])
    }
}

/// Numerical knobs for the per-pocket bank — Python's keyword tail on `_pocket_quench_mean_ei`.
#[derive(Debug, Clone, Copy)]
pub struct PocketOpts {
    /// ξ-grid points; each one is a pocket, and each rich pocket a full quench — the cost driver
    pub n_bell: usize,
    /// trajectory points inside each pocket's quench
    pub quench_ngrid: usize,
    /// RK4 steps inside each pocket's quench
    pub quench_nsteps: usize,
    /// RUNG 20/21 — lift each pocket's initial [O] AND its quench re-making
    pub super_eq_o: bool,
}

impl Default for PocketOpts {
    fn default() -> Self {
        Self { n_bell: 120, quench_ngrid: 240, quench_nsteps: 2000, super_eq_o: false }
    }
}

/// Build the rung-16 per-pocket bank at a single dwell `tau_core`.
///
/// Pocket bookkeeping (a mixture fraction ξ ⇒ local `far_local = ξ/(1−ξ)`, `α = far_ov/far_local`):
///
/// * **RICH of the overall mean**, burnable, φ≤2 → its own [`quench_no`]: it dilutes DOWN through
///   stoichiometric toward the mean, which is the rung-10 re-making.
/// * **LEAN of the mean / φ>2 / too lean to burn** → [`ideal_bell_ei`] (0 above φ2). A lean
///   pocket only gets leaner on dilution and never re-crosses stoich, so it has no finite
///   quench. Keeping this branch bit-identical to rung 15's is what makes the reduce exact.
///
/// `n0 = α · x_no · Σn` is `(α·x_no)·Σn`, left to right; float multiply is not associative and
/// this is the seed the whole pocket integration starts from.
pub fn pocket_quench_grid(
    far_overall: f64,
    tt3: f64,
    p: f64,
    hf_fuel: f64,
    tau_ref: f64,
    tau_core: f64,
    o: PocketOpts,
) -> PocketGrid {
    let xi_max = xi_soot_bound(); // ξ at the soot bound φ=2
    let xi_grid: Vec<f64> =
        (0..o.n_bell).map(|i| xi_max * (i as f64 + 0.5) / o.n_bell as f64).collect();
    let mut vals = Vec::with_capacity(o.n_bell);
    let mut max_a = 0.0f64;
    for &xi in &xi_grid {
        let far_local = xi / (1.0 - xi);
        if far_local < far_overall
            || far_local / gas::f_stoich() > 2.0 + 1e-9
            || far_local <= 0.0
        {
            // lean/tail: the rung-15 bell, lifted WITH the pocket so there is no half-eq-O hybrid
            vals.push(ideal_bell_ei(far_local, p, tt3, hf_fuel, tau_ref, o.super_eq_o));
            continue;
        }
        let Some(t_p) = try_primary_aft(far_local, p, tt3, hf_fuel) else {
            vals.push(0.0); // too lean to burn (cold-edge flame)
            continue;
        };
        let alpha = far_overall / far_local; // ≤ 1 (a rich-of-mean pocket)
        let comp = equilibrium_composition(far_local, t_p, p);
        // RUNG 20 — lift the pocket's initial [O] at its OWN flame T_p, and (below) its quench
        // re-making too, so every EI in the β-PDF integral carries the same closure.
        let m0 = if o.super_eq_o {
            super_eq_o_multiplier(t_p.max(SUPER_EQ_T_FLOOR))
        } else {
            1.0
        };
        let ntot: f64 = comp.iter().map(|&(_, v)| v).sum();
        let n0 = alpha * thermal_no(&comp, t_p, p, tau_ref, far_local, 4000, m0).x_no * ntot;
        let q = quench_no(
            &comp,
            t_p,
            alpha,
            far_overall,
            tt3,
            p,
            n0,
            tau_core,
            QuenchOpts {
                nsteps: o.quench_nsteps,
                ngrid: o.quench_ngrid,
                tab: None,
                schedule: None,
                super_eq_o: o.super_eq_o,
            },
        );
        vals.push(q.ei);
        max_a = max_a.max(q.max_a);
    }
    PocketGrid { xi_grid, vals, max_a }
}

/// Integrate a prebuilt [`PocketGrid`] against the β-PDF — the cheap half of rung 16.
///
/// `g → 0` ⇒ a delta at ξ̄ ⇒ the single pocket AT the mean (≈0 at a lean mean, which is why the
/// finite bulk floor dominates there).
pub fn pocket_quench_integrate(
    grid: &PocketGrid,
    far_overall: f64,
    g_seg: f64,
    n_quad: usize,
) -> f64 {
    let xibar = far_overall / (1.0 + far_overall);
    if g_seg <= 1e-9 {
        return grid.at(xibar); // delta ⇒ a single pocket at the mean
    }
    let (nodes, w) = beta_pdf_nodes_weights(xibar, g_seg, n_quad);
    w.iter().zip(nodes.iter()).fold(0.0, |acc, (&wi, &x)| acc + wi * grid.at(x))
}

/// `⟨EI_pocket_quench(ξ; τ_core)⟩` over the β-PDF — rung 16's closure, as one call.
///
/// The rung-16 upgrade of [`pdf_mean_ei`]: where rung 15 integrates the CONSTANT-T ideal bell
/// and multiplies by a scalar dwell factor `D(u) = τ_core/τ_ref` (a LINEARISATION, exact only
/// while EI ∝ τ), rung 16 carries EACH rich-of-mean pocket through its OWN finite quench at the
/// dwell τ_core, so the dwell enters INSIDE the chemistry. A pocket that lingers COOLS as it
/// re-makes NO through the stoichiometric crossing, so ⟨EI⟩ is SUBLINEAR in τ_core — the
/// cooling-limited dwell erodes rung-15's far-over-penetration flank.
///
/// Returns `(⟨EI⟩ g NO/kg fuel, max_a)`; `max_a` folds into the clamp-dormancy gate.
#[allow(clippy::too_many_arguments)]
pub fn pocket_quench_mean_ei(
    far_overall: f64,
    tt3: f64,
    p: f64,
    hf_fuel: f64,
    tau_ref: f64,
    tau_core: f64,
    g_seg: f64,
    n_quad: usize,
    o: PocketOpts,
) -> (f64, f64) {
    let grid = pocket_quench_grid(far_overall, tt3, p, hf_fuel, tau_ref, tau_core, o);
    (pocket_quench_integrate(&grid, far_overall, g_seg, n_quad), grid.max_a)
}

/// Rung-18 DERIVED injection ceiling `g_ceiling = (ξ_p − ξ̄)/(1 − ξ̄)`.
///
/// The maximum normalized variance of a two-delta PDF on {0 (dilution air), ξ_p (rich-primary
/// products)} at the fixed overall mean ξ̄. Set by the PRIMARY RICHNESS φ_p, **NOT a free knob**
/// — the one quantity rung 18 DERIVES rather than fits, and it exposes rung-13's `g_max = 0.3`
/// as ~4.4× too large (φ_p=1.5 ⇒ 0.0675).
///
/// A two-delta at {0, ξ_p} with mean ξ̄ carries mass ξ̄/ξ_p at ξ_p; its variance is ξ̄(ξ_p−ξ̄), so
/// the normalized segregation is `(ξ_p−ξ̄)/(1−ξ̄)`. Requires `φ_p > φ_overall` (a rich primary
/// diluting down to a leaner mean — the RQL geometry), which the assert enforces.
pub fn two_stream_ceiling(far_overall: f64, phi_primary: f64) -> f64 {
    let xibar = far_overall / (1.0 + far_overall);
    let far_p = phi_primary * gas::f_stoich();
    let xi_p = far_p / (1.0 + far_p);
    let g_ceiling = (xi_p - xibar) / (1.0 - xibar);
    assert!(
        0.0 < g_ceiling && g_ceiling < 1.0,
        "two-stream ceiling g={g_ceiling:.4} outside (0,1): the primary (φ_p={phi_primary}, \
         ξ_p={xi_p:.4}) must be RICHER than the overall mean (ξ̄={xibar:.4}) — the RQL geometry."
    );
    g_ceiling
}

/// Rung-18 transported segregation: the mixture-fraction variance DECAY ODE
/// `dg/dt = −C_φ·ω·g` integrated over the residence `[0, τ]` from the injection ceiling.
///
/// ω is the turbulent mixing frequency and `C_φ ≈ 2` the canonical mechanical-to-scalar
/// timescale ratio (scalar dissipation). Returns the residual width `g(τ)`.
///
/// **Analytic for constant ω — and deliberately NOT written that way.** `g_ceiling·exp(−C_φ·ω·τ)`
/// is the closed form, but the negative-result gate has to be able to drive this with ANY ω, so
/// it is integrated numerically: backward (implicit) Euler on the linear decay, which is
/// unconditionally stable AND positivity-preserving for any dt (forward Euler goes negative once
/// `C_φ·ω·dt > 1`). The loop of `nsteps` divisions is ~1 % from the closed form at the shipped
/// settings, and the oracle dumps both side by side so a "simplification" fails loudly here
/// instead of quietly making rung 18's basin the wrong depth.
///
/// The physics result this exists to state is NEGATIVE: a mean-field ω(J) gives a monotone/flat
/// g(J) — no interior optimum. An optimum appears ONLY once ω is given a SPATIAL coverage
/// dependence ω(C = (S/H)√J), i.e. once the jet spacing S is injected. So this ODE cannot DERIVE
/// the Holdeman optimum; the location is imposed through the caller's ω(C). `docs/rung18-spec.md`.
pub fn transport_variance(
    g_ceiling: f64,
    omega: f64,
    tau: f64,
    c_phi: f64,
    nsteps: usize,
) -> f64 {
    let dt = tau / nsteps as f64;
    let mut g = g_ceiling;
    let denom = 1.0 + c_phi * omega * dt;
    for _ in 0..nsteps {
        g /= denom;
    }
    assert!(
        0.0 < g && g <= g_ceiling + 1e-12,
        "transported variance g={g:.4e} left (0, g_ceiling={g_ceiling:.4e}] — the decay ODE must \
         stay bounded by the injection ceiling (check C_φ·ω·τ ≥ 0 and dt small enough)."
    );
    g
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

/// Rung-11 jet-in-crossflow mixing config — the PHYSICAL dilution-air entrainment model that
/// retires rung 10's free `τ_q` + linear-schedule knobs. `docs/rung11-spec.md`.
///
/// A MEAN-FIELD model: a single well-mixed core diluting on a mean β(t). It DERIVES the quench
/// RATE from jet momentum but CANNOT produce a mixing OPTIMUM — that is a spatial-variance
/// effect (an over-penetrating jet leaves an un-mixed hot near-stoich core), deferred to rung
/// 12. So the J-sweep is MONOTONE **by construction**, not by accident.
///
/// The one design knob is the momentum-flux ratio `J = ρ_j U_j²/(ρ_c U_c²)`; `H`/`U_c`/`C_e`/
/// `shape_n` are order-of-magnitude / un-anchored (like α, φ_p, τ), so the ABSOLUTE `τ_q` is
/// un-pinned — what is certified is the √J SCALING and the monotone direction.
#[derive(Debug, Clone, Copy)]
pub struct JetMixing {
    /// jet-to-crossflow momentum-flux ratio (THE design knob)
    pub j: f64,
    /// dilution-zone duct height, m (the cross-stream mixing length)
    pub h: f64,
    /// bulk crossflow velocity, m/s
    pub u_c: f64,
    /// entrainment constant, O(0.1); folds penetration + entrainment + density ratio
    pub c_e: f64,
    /// entrainment schedule exponent (1 = linear/rung-10; >1 decelerating)
    pub shape_n: f64,
}

impl Default for JetMixing {
    /// Python's dataclass defaults. `J` has none there (it is the required design knob), so the
    /// value here is a placeholder every construction site is expected to overwrite; the
    /// `..Default::default()` idiom is what keeps the other four honest.
    fn default() -> Self {
        Self { j: 1.0, h: 0.10, u_c: 75.0, c_e: 0.15, shape_n: 2.0 }
    }
}

impl JetMixing {
    /// Python's `__post_init__`; see [`PromptNo::validate`] for why it is a method here.
    pub fn validate(&self) {
        for (name, v) in [
            ("J", self.j),
            ("H", self.h),
            ("U_c", self.u_c),
            ("C_e", self.c_e),
            ("shape_n", self.shape_n),
        ] {
            assert!(v > 0.0, "JetMixing.{name}={v} must be positive");
        }
    }

    /// Derived quench time `τ_q = H/(C_e·√J·U_c)` — monotone-DECREASING in J (a
    /// higher-momentum jet penetrates and entrains faster; penetration ∝ √J). "Quick quench"
    /// = high jet momentum. For physical `J ∈ [4,100]` this lands in the RQL sub-ms–few-ms band.
    ///
    /// **`sqrt`, not `powp`.** Python spells this `math.sqrt(self.J)`, which IS the sqrt
    /// instruction — the exact inverse of phase 2's trap, where Python's `x ** 0.5` was a libm
    /// `pow` call that differed from `sqrt` about 1 point in 670. Applying "always `powp`"
    /// mechanically would get this one backwards.
    pub fn tau_q(&self) -> f64 {
        self.h / (self.c_e * self.j.sqrt() * self.u_c)
    }

    /// The entrainment schedule `β(t/τ_q) = 1 − (1 − t/τ_q)^shape_n` — reaches β=1 EXACTLY at
    /// `tfrac=1` (no endpoint trap). `shape_n=1` ⇒ `β = tfrac` (linear = constant entrainment =
    /// rung 10, the reduce); `shape_n>1` ⇒ concave/decelerating (fast near the jet where shear
    /// and gradient are strong, slowing as the concentration difference collapses).
    ///
    /// `shape_n == 1` returns the IDENTITY exactly — not `1−(1−x)^1`, which drifts a ULP — so a
    /// `shape_n=1` jet is BYTE-IDENTICAL to the rung-10 linear path at the derived `τ_q`.
    ///
    /// The exponent is a float FIELD, not an integer literal, so PyPy's `x ** 2` → multiply
    /// rewrite does not apply and this reaches libm `pow`: hence [`powp`]. That is measured by
    /// the oracle's own schedule block, not reasoned about — it is the first non-literal float
    /// exponent in a hot path in the project.
    pub fn schedule(&self, tfrac: f64) -> f64 {
        if self.shape_n == 1.0 {
            return tfrac;
        }
        1.0 - powp(1.0 - tfrac, self.shape_n)
    }
}

/// Rung-12 spatial-unmixedness (two-stream) model — the VARIANCE layer rung 11 deferred. It
/// rides ON a [`JetMixing`] (which supplies J and the duct height H) and finally makes the
/// NO-vs-J curve TURN BACK UP, recovering the classic Holdeman dilution-jet optimum AT `C_opt`.
///
/// Rung 11 was MEAN-FIELD, so its J-sweep is monotone (a stronger jet only ever re-makes LESS
/// NO). But a real dilution jet has an OPTIMUM at the Holdeman group `C = (S/H)√J ≈ 2.5` (a
/// *uniformity* criterion): UNDER-penetration leaves the jet near the wall and the core
/// un-mixed; OVER-penetration slams the air onto the far wall / collides jets in the centre —
/// BOTH leave a hot, near-stoichiometric core that misses the fast jet mixing and lingers.
///
/// THE MODEL (a two-stream split, mass-weighted; the CORE carries the off-optimum penalty TWO
/// WAYS):
///
/// * a BULK fraction `(1−w)` quenched at the rung-11 jet time `τ_mean(J)` — the mean-field
///   flow (∝ 1/√J: monotone-falling, the fixed REFERENCE, NOT a function of `C_opt`);
/// * an UNDER-MIXED CORE fraction `w` that MISSES the jet and lingers, quenched at a dwell
///   `τ_core(C) = τ_res·(1 + b_u·u)` — an ABSOLUTE residence, NOT the vanishing jet time, so
///   its NO penalty survives J→∞ — that GROWS off-optimum;
/// * the unmixedness `u(C) = |ln(C/C_opt)|` drives both and is KINKED at `C_opt`.
///
/// ```text
/// EI_total = (1−w)·EI(τ_mean) + w·EI(τ_core)
/// ```
///
/// THE PIN: with a SMOOTH (parabolic) `w` the turn-up would drift to a stronger jet than
/// `C_opt` — the still-falling mean-field bulk pulls it right. The KINK gives `w` a non-zero
/// slope at `C_opt`, so the turn-up starts THERE. The EI-min therefore pins at `C_opt` for ALL
/// `S` ⇒ `J_min = J_opt = (C_opt·H/S)²`, shifting EXACTLY as `(H/S)²` — the Holdeman group made
/// literal. `docs/rung12-spec.md`.
///
/// Like `C_e`/`τ_q`, the ABSOLUTE knobs (`S`, `τ_res`, `k_u`, `b_u`, `w_max`) are
/// order-of-magnitude / un-anchored — what is certified is the TURN-UP, the optimum AT `C_opt`,
/// and the `(H/S)²` shift.
#[derive(Debug, Clone, Copy)]
pub struct Unmixedness {
    /// dilution-jet spacing, m (cross-stream spacing of adjacent jets)
    pub s: f64,
    /// Holdeman uniformity optimum of `C = (S/H)√J` (best cross-plane mixing)
    pub c_opt: f64,
    /// core dwell AT the optimum, s (the absolute dilution-zone residence)
    pub tau_res: f64,
    /// core-fraction sensitivity to unmixedness (the kink that PINS the min at `C_opt`)
    pub k_u: f64,
    /// core-dwell growth off-optimum (keeps the over-penetration flank rising)
    pub b_u: f64,
    /// cap on the segregated core fraction (`0 < w_max ≤ 1`)
    pub w_max: f64,
}

impl Default for Unmixedness {
    fn default() -> Self {
        Self { s: 0.0625, c_opt: 2.5, tau_res: 2.5e-3, k_u: 2.5, b_u: 1.0, w_max: 0.7 }
    }
}

impl Unmixedness {
    /// Python's `__post_init__`.
    pub fn validate(&self) {
        for (name, v) in
            [("S", self.s), ("C_opt", self.c_opt), ("tau_res", self.tau_res), ("w_max", self.w_max)]
        {
            assert!(v > 0.0, "Unmixedness.{name}={v} must be positive");
        }
        assert!(self.k_u >= 0.0, "Unmixedness.k_u={} must be ≥ 0 (0 ⇒ reduce to rung 11)", self.k_u);
        assert!(self.b_u >= 0.0, "Unmixedness.b_u={} must be ≥ 0", self.b_u);
        assert!(
            self.w_max <= 1.0,
            "Unmixedness.w_max={} must be ≤ 1 (a mass fraction)",
            self.w_max
        );
    }

    /// The Holdeman momentum-flux/geometry group `C = (S/H)√J` — jet penetration ∝ √J scaled by
    /// the spacing/height ratio. Uses the paired [`JetMixing`]'s `H` and `J` (the same jet that
    /// set `τ_q`). `sqrt`, not `powp`, for the reason in [`JetMixing::tau_q`].
    pub fn c(&self, mixing: &JetMixing) -> f64 {
        (self.s / mixing.h) * mixing.j.sqrt()
    }

    /// The unmixedness `u(C) = |ln(C/C_opt)|` — an L1 (KINKED) distance from the Holdeman
    /// optimum, 0 at `C_opt`, symmetric in `ln C`. The kink is what pins the EI-min AT `C_opt`
    /// rather than at a stronger jet (a smooth parabola would let it drift).
    pub fn u(&self, c: f64) -> f64 {
        (c / self.c_opt).ln().abs()
    }

    /// The un-mixed (segregated) core mass fraction `w(C) = min(w_max, k_u·u)`. Zero at `C_opt`
    /// (perfect tiling), rising on BOTH flanks, capped at `w_max`.
    pub fn core_fraction(&self, c: f64) -> f64 {
        self.w_max.min(self.k_u * self.u(c))
    }

    /// The under-mixed core's quench dwell `τ_core(C) = τ_res·(1 + b_u·u)` — an ABSOLUTE
    /// residence (it does NOT ride the vanishing jet time `τ_mean ∝ 1/√J`, so its NO penalty
    /// survives J→∞) that GROWS off-optimum. Equals `τ_res` at `C_opt`.
    pub fn core_dwell(&self, c: f64) -> f64 {
        self.tau_res * (1.0 + self.b_u * self.u(c))
    }
}

/// Rung-13 resolved-mixing-PDF config — a mean-preserving β-PDF of mixture fraction that
/// replaces rung-12's parameterised SEGREGATION (`w(C)`) with a CONTINUOUS distribution.
///
/// Rides ON a [`JetMixing`] (it needs J and the duct height H for the Holdeman group
/// `C = (S/H)√J`) and is MUTUALLY EXCLUSIVE with [`Unmixedness`] — two closures of the same
/// segregation physics.
///
/// **A MECHANISM SEPARATION, not a rung-12 reproduction.** Rung-12's over-penetration CLIMB came
/// from the DWELL effect — an absolute, off-optimum-growing `τ_core` surviving J→∞, a TIME
/// mechanism. This rung isolates the COMPOSITION mechanism and DROPS the quench chain, so it
/// structurally CANNOT climb: it pins the optimum LOCATION (min AT `C_opt`) while the
/// far-over-penetration flank DESCENDS. Composition variance pins the optimum; the dwell makes
/// the climb; combining them is rung 15.
///
/// **The lesson, framed correctly — NOT generic "convexity/Jensen".** The NO-vs-φ bell is convex
/// on its flanks but CONCAVE at the peak, so there is no global convexity to invoke. The honest
/// statement: NO production is sharply PEAKED at stoich, so spreading the local φ around a fixed
/// mean RAISES the mean NO whenever the mean is OFF-stoich — the stoich-ward tail samples the
/// peak while the mean itself sits in a low-EI wing — most strongly the leaner the mean, and it
/// REVERSES SIGN at a stoichiometric mean. Our combustor mean is LEAN, so segregation raises NO
/// and the optimum is where segregation is least.
///
/// `docs/rung13-spec.md`. Like the rung-9..12 knobs, `S`/`k_g`/`g_max` are order-of-magnitude;
/// `C_opt ≈ 2.5` is Holdeman's value. What is CERTIFIED is the optimum pinned AT `C_opt` (both
/// flanks up), the `(H/S)²` shift, and the SIGN of the effect and its reversal at a stoich mean.
#[derive(Debug, Clone, Copy)]
pub struct MixingPdf {
    /// dilution-jet spacing, m (forms the Holdeman group with H and J)
    pub s: f64,
    /// Holdeman uniformity optimum of `C = (S/H)√J` (the segregation minimum)
    pub c_opt: f64,
    /// segregation sensitivity to the kinked distance `|ln(C/C_opt)|`
    pub k_g: f64,
    /// cap on the segregation (`0 < g_max < 1`; past the ⟨EI⟩(g) hump ⇒ the far flank descends)
    pub g_max: f64,
    /// reference-bell grid points — EI(ξ) is built ONCE and interpolated
    pub n_bell: usize,
    /// β-PDF quadrature nodes
    pub n_quad: usize,
}

impl Default for MixingPdf {
    fn default() -> Self {
        Self { s: 0.0625, c_opt: 2.5, k_g: 0.3, g_max: 0.3, n_bell: 200, n_quad: 200 }
    }
}

impl MixingPdf {
    /// Python's `__post_init__`.
    pub fn validate(&self) {
        for (name, v) in [("S", self.s), ("C_opt", self.c_opt)] {
            assert!(v > 0.0, "MixingPDF.{name}={v} must be positive");
        }
        assert!(
            self.k_g >= 0.0,
            "MixingPDF.k_g={} must be ≥ 0 (0 ⇒ g≡0 ⇒ well-mixed point)",
            self.k_g
        );
        assert!(
            0.0 < self.g_max && self.g_max < 1.0,
            "MixingPDF.g_max={} must be in (0,1)",
            self.g_max
        );
        assert!(
            self.n_bell > 1 && self.n_quad > 1,
            "MixingPDF grid sizes (n_bell, n_quad) must be > 1"
        );
    }

    /// The Holdeman group `C = (S/H)√J` — identical to [`Unmixedness::c`], the same jet that set
    /// `τ_q`. `sqrt`, not `powp`, for the reason in [`JetMixing::tau_q`].
    pub fn c(&self, mixing: &JetMixing) -> f64 {
        (self.s / mixing.h) * mixing.j.sqrt()
    }

    /// The β-PDF segregation width `g(C) = min(g_max, k_g·|ln(C/C_opt)|)` — KINKED at `C_opt`
    /// (0 there, rising on BOTH flanks with a non-zero slope, which is what pins the emissions
    /// minimum AT `C_opt`), capped at `g_max`. `g → 0` ⇒ a delta at ξ̄ ⇒ the well-mixed point.
    pub fn segregation(&self, c: f64) -> f64 {
        self.g_max.min(self.k_g * (c / self.c_opt).ln().abs())
    }
}

/// Rung-15 PDF-THROUGH-QUENCH config — rung-13's β-PDF carried THROUGH the rung-10/12 dwell
/// chain, so the two mixing mechanisms finally COMBINE.
///
/// ```text
/// ⟨EI⟩₁₅ = EI_bulk_quench(τ_mean)   # term 1: the rung-11 mean field — a FINITE floor, all C
///        + D(u) · ⟨EI_bell⟩(g)      # term 2: the rung-13 integral × a rung-12 dwell
/// ```
///
/// with `g(C) = min(g_max, k_g·u)` (rung-13 segregation), `u(C) = |ln(C/C_opt)|` (rung-12
/// unmixedness), and the dwell factor `D(u) = τ_res(1 + b_u·u)/τ_ref` — an ABSOLUTE
/// off-optimum-growing residence rescaling the reference-τ bell EI to the pocket's actual
/// lingering dwell (exact while EI ∝ τ, the dormant clamp). `τ_ref` is the `zoned_nox` residence
/// at which the bell is built, so the two stay locked.
///
/// **Distinguishable from BOTH parents**: the finite floor and the climbing far flank are NOT
/// rung 13 (whose optimum is ≈0 and whose far flank descends); the STOICH-MEAN SIGN REVERSAL is
/// NOT reproducible by rung-12's lumped dwell, which is the discriminator that catches the naïve
/// "dwell-only PDF through the quench" trap. `b_u = 3` is larger than rung-12's default because
/// term 2's `⟨EI_bell⟩` is a weaker lever than rung-12's `EI(τ_core)`. `docs/rung15-spec.md`.
#[derive(Debug, Clone, Copy)]
pub struct QuenchPdf {
    /// dilution-jet spacing, m
    pub s: f64,
    /// Holdeman uniformity optimum of `C = (S/H)√J` (segregation AND dwell minimum)
    pub c_opt: f64,
    /// β-PDF segregation sensitivity to `|ln(C/C_opt)|` (the rung-13 width)
    pub k_g: f64,
    /// cap on the segregation (`0 < g_max < 1`; the rung-13 bimodal bound)
    pub g_max: f64,
    /// under-mixed-pocket dwell AT the optimum, s (an absolute residence; rung 12)
    pub tau_res: f64,
    /// off-optimum dwell growth — pins the min at `C_opt`; larger than rung-12's default
    pub b_u: f64,
    /// reference-bell grid points (rung 13)
    pub n_bell: usize,
    /// β-PDF quadrature nodes (rung 13)
    pub n_quad: usize,
}

impl Default for QuenchPdf {
    fn default() -> Self {
        Self {
            s: 0.0625,
            c_opt: 2.5,
            k_g: 0.3,
            g_max: 0.3,
            tau_res: 2.5e-3,
            b_u: 3.0,
            n_bell: 200,
            n_quad: 200,
        }
    }
}

impl QuenchPdf {
    /// Python's `__post_init__`.
    pub fn validate(&self) {
        for (name, v) in [("S", self.s), ("C_opt", self.c_opt), ("tau_res", self.tau_res)] {
            assert!(v > 0.0, "QuenchPDF.{name}={v} must be positive");
        }
        assert!(self.k_g >= 0.0, "QuenchPDF.k_g={} must be ≥ 0 (0 ⇒ floor only)", self.k_g);
        assert!(self.b_u >= 0.0, "QuenchPDF.b_u={} must be ≥ 0", self.b_u);
        assert!(
            0.0 < self.g_max && self.g_max < 1.0,
            "QuenchPDF.g_max={} must be in (0,1)",
            self.g_max
        );
        assert!(
            self.n_bell > 1 && self.n_quad > 1,
            "QuenchPDF grid sizes (n_bell, n_quad) must be > 1"
        );
    }

    /// The Holdeman group `C = (S/H)√J` — identical to [`MixingPdf::c`].
    pub fn c(&self, mixing: &JetMixing) -> f64 {
        (self.s / mixing.h) * mixing.j.sqrt()
    }

    /// The unmixedness `u(C) = |ln(C/C_opt)|` — the KINKED L1 distance from the optimum, driving
    /// BOTH the β-PDF width and the dwell growth (rungs 12 and 13's kinks, unified).
    pub fn u(&self, c: f64) -> f64 {
        (c / self.c_opt).ln().abs()
    }

    /// The rung-13 β-PDF segregation width `g(C) = min(g_max, k_g·u)`.
    pub fn segregation(&self, c: f64) -> f64 {
        self.g_max.min(self.k_g * self.u(c))
    }

    /// The dwell factor `D(u) = τ_res(1 + b_u·u)/τ_ref` — the segregated pocket's ABSOLUTE
    /// residence relative to the bell's reference residence. Its off-optimum growth is what makes
    /// the far flank CLIMB.
    pub fn dwell_factor(&self, c: f64, tau_ref: f64) -> f64 {
        self.tau_res * (1.0 + self.b_u * self.u(c)) / tau_ref
    }
}

/// Rung-16 PER-POCKET PDF-through-quench config — RETIRES rung-15's one acknowledged
/// linearisation.
///
/// Rung 15 carried the composition β-PDF through the dwell as `term 2 = D(u)·⟨EI_bell⟩(g)`: the
/// CONSTANT-T ideal bell times a SCALAR dwell factor, exact only while EI ∝ τ — which ignores
/// that a lingering pocket COOLS. Rung 16 carries EACH rich-of-mean pocket through its OWN finite
/// quench at the dwell `τ_core`, so the dwell acts INSIDE the chemistry. Same knobs, same
/// rides-on-`JetMixing`, same Holdeman group.
///
/// **The robust lesson** (`docs/rung16-spec.md`):
/// * SUBLINEAR DWELL (the mechanism) — term 2 grows sublinearly in `τ_core`, against rung-15's
///   linear `D(u)·EI` whose growth IS the dwell ratio exactly. The linearisation made visible.
/// * FAR-FLANK EROSION (the headline) — the cooling-limited dwell erodes rung-15's
///   over-penetration secondary basin into NEAR-DEGENERACY with the sharp `C_opt` notch, which
///   survives (the composition excess still → 0 at `C_opt`, both immediate flanks up).
/// * **NOT CLAIMED: which of the two near-degenerate optima is GLOBALLY lowest.** It flips across
///   the β-PDF quadrature, the φ>2 tail treatment and the `C_e` regime, all comparable to the
///   margin. Rung 16 quantifies the linearisation error; it does not relocate the optimum — so no
///   gate here asserts a global-min LOCATION, and the oracle deliberately dumps no argmin for it.
///
/// The defaults are SMALLER than rung 15's because each `n_bell` node is a full quench.
#[derive(Debug, Clone, Copy)]
pub struct PocketQuenchPdf {
    /// dilution-jet spacing, m
    pub s: f64,
    /// Holdeman uniformity optimum of `C = (S/H)√J`
    pub c_opt: f64,
    /// β-PDF segregation sensitivity to `|ln(C/C_opt)|` (the rung-13 width)
    pub k_g: f64,
    /// cap on the segregation (`0 < g_max < 1`)
    pub g_max: f64,
    /// under-mixed-pocket dwell AT the optimum, s (absolute; rung 12)
    pub tau_res: f64,
    /// off-optimum dwell growth (the rung-12 `core_dwell` slope)
    pub b_u: f64,
    /// per-pocket ξ-grid points — each one a full quench, so this is THE cost driver
    pub n_bell: usize,
    /// β-PDF quadrature nodes (rung 13)
    pub n_quad: usize,
}

impl Default for PocketQuenchPdf {
    fn default() -> Self {
        Self {
            s: 0.0625,
            c_opt: 2.5,
            k_g: 0.3,
            g_max: 0.3,
            tau_res: 2.5e-3,
            b_u: 3.0,
            n_bell: 120,
            n_quad: 160,
        }
    }
}

impl PocketQuenchPdf {
    /// Python's `__post_init__`.
    pub fn validate(&self) {
        for (name, v) in [("S", self.s), ("C_opt", self.c_opt), ("tau_res", self.tau_res)] {
            assert!(v > 0.0, "PocketQuenchPDF.{name}={v} must be positive");
        }
        assert!(
            self.k_g >= 0.0,
            "PocketQuenchPDF.k_g={} must be ≥ 0 (0 ⇒ floor only)",
            self.k_g
        );
        assert!(self.b_u >= 0.0, "PocketQuenchPDF.b_u={} must be ≥ 0", self.b_u);
        assert!(
            0.0 < self.g_max && self.g_max < 1.0,
            "PocketQuenchPDF.g_max={} must be in (0,1)",
            self.g_max
        );
        assert!(self.n_bell > 1 && self.n_quad > 1, "PocketQuenchPDF grid sizes must be > 1");
    }

    /// The Holdeman group `C = (S/H)√J` — identical to [`QuenchPdf::c`].
    pub fn c(&self, mixing: &JetMixing) -> f64 {
        (self.s / mixing.h) * mixing.j.sqrt()
    }

    /// The unmixedness `u(C) = |ln(C/C_opt)|`.
    pub fn u(&self, c: f64) -> f64 {
        (c / self.c_opt).ln().abs()
    }

    /// The rung-13 β-PDF segregation width `g(C) = min(g_max, k_g·u)`.
    pub fn segregation(&self, c: f64) -> f64 {
        self.g_max.min(self.k_g * self.u(c))
    }

    /// The ABSOLUTE per-pocket dwell `τ_core(C) = τ_res(1 + b_u·u)`. Unlike rung 15's
    /// [`QuenchPdf::dwell_factor`] — a `τ_core/τ_ref` RATIO multiplying a constant-T bell — this
    /// goes straight INTO each pocket's quench, so the dwell acts through the cooling chemistry.
    pub fn core_dwell(&self, c: f64) -> f64 {
        self.tau_res * (1.0 + self.b_u * self.u(c))
    }
}

/// Rung-18 TRANSPORTED-variance config — the honest LIMIT of the deferred "transported PDF" seam.
///
/// Rungs 12–17 IMPOSE the β-PDF width as a kinked `g(C) = min(g_max, k_g·|ln(C/C_opt)|)`. This
/// config instead solves `g(C)` as the residual of a variance DECAY ODE
/// (`dg/dt = −C_φ·ω(C)·g`, [`transport_variance`]) from a DERIVED two-stream ceiling
/// ([`two_stream_ceiling`]), then feeds it through the same rung-13 ideal bell.
///
/// **THE LOAD-BEARING RESULT IS NEGATIVE, and stronger for it.** A 0-D variance transport CANNOT
/// DERIVE the `C_opt` optimum: with any MEAN-FIELD ω(J) the residual g(J) is monotone or flat —
/// no interior optimum, because an optimum needs ω peaked at a specific PENETRATION, i.e. the
/// SPATIAL spacing S, which a mean-field trajectory does not contain. So the coverage
/// `ω(C) = ω_opt·exp(−ln²(C/C_opt)/2w_cov²)` below is an EXPLICITLY IMPOSED spatial closure — the
/// honest successor of rung-13's kinked g(C), NOT a derivation.
///
/// **What transport legitimately DOES add** (certified): a DERIVED ceiling from φ_p, exposing
/// rung-13's `g_max = 0.3` as ~4.4× too large; a RESIDUAL floor `g(C_opt) = g_ceiling·e^(−Da_opt)
/// > 0`, so the emissions optimum is ELEVATED off the well-mixed value rather than touching it;
/// and SMOOTHNESS — both one-sided slopes vanish at `C_opt`, so the kink's sharpness was the
/// artifact, not its location. `docs/rung18-spec.md`.
#[derive(Debug, Clone, Copy)]
pub struct TransportedPdf {
    /// dilution-jet spacing, m
    pub s: f64,
    /// Holdeman uniformity optimum of `C = (S/H)√J` (here, the COVERAGE peak)
    pub c_opt: f64,
    /// scalar-dissipation constant (mechanical-to-scalar timescale ratio; anchored ≈2)
    pub c_phi: f64,
    /// optimum Damköhler `C_φ·ω_opt·τ` — e-folds of variance the best jet decays
    pub da_opt: f64,
    /// coverage width in `ln(C/C_opt)` — sets the basin breadth; the IMPOSED spatial part
    pub w_cov: f64,
    /// mixing residence for the ODE, s (folds into Da; g depends only on the product)
    pub tau_mix: f64,
    /// ideal-bell grid points (rung 13)
    pub n_bell: usize,
    /// β-PDF quadrature nodes (rung 13)
    pub n_quad: usize,
    /// variance-ODE integration steps
    pub n_ode: usize,
}

impl Default for TransportedPdf {
    fn default() -> Self {
        Self {
            s: 0.0625,
            c_opt: 2.5,
            c_phi: 2.0,
            da_opt: 2.0,
            w_cov: 1.0,
            tau_mix: 2.5e-3,
            n_bell: 200,
            n_quad: 200,
            n_ode: 400,
        }
    }
}

impl TransportedPdf {
    /// Python's `__post_init__`.
    pub fn validate(&self) {
        for (name, v) in [
            ("S", self.s),
            ("C_opt", self.c_opt),
            ("C_phi", self.c_phi),
            ("Da_opt", self.da_opt),
            ("w_cov", self.w_cov),
            ("tau_mix", self.tau_mix),
        ] {
            assert!(v > 0.0, "TransportedPDF.{name}={v} must be positive");
        }
        assert!(
            self.n_bell > 1 && self.n_quad > 1 && self.n_ode > 1,
            "TransportedPDF grid sizes (n_bell, n_quad, n_ode) must be > 1"
        );
    }

    /// The Holdeman group `C = (S/H)√J` — identical to [`MixingPdf::c`].
    pub fn c(&self, mixing: &JetMixing) -> f64 {
        (self.s / mixing.h) * mixing.j.sqrt()
    }

    /// The IMPOSED spatial coverage `ω(C) = ω_opt·exp(−ln²(C/C_opt)/2w_cov²)`, peaked at `C_opt`
    /// (the best cross-plane tiling ⇒ fastest scalar dissipation) and SMOOTH — an analytic
    /// maximum, so zero slope at `C_opt`, NOT the kink. `ω_opt` is folded in via
    /// `Da_opt = C_φ·ω_opt·τ_mix`.
    ///
    /// **This is the one thing a 0-D transport cannot derive** — the spatial S enters here, and
    /// nowhere else — so it is the explicit successor of rung-13's kinked g(C), stated as an
    /// imposition rather than smuggled in.
    pub fn coverage_omega(&self, c: f64) -> f64 {
        let omega_opt = self.da_opt / (self.c_phi * self.tau_mix); // from Da_opt = C_φ·ω_opt·τ_mix
        let lnr = (c / self.c_opt).ln();
        omega_opt * (-lnr * lnr / (2.0 * self.w_cov * self.w_cov)).exp()
    }

    /// The TRANSPORTED width `g(C)`: integrate the decay ODE from the DERIVED two-stream ceiling.
    /// A smooth basin (min AT `C_opt`, from the imposed coverage) ELEVATED off zero by the
    /// residual `g(C_opt) = g_ceiling·e^(−Da_opt) > 0`. Returns `(g, g_ceiling)`.
    pub fn segregation(
        &self,
        c: f64,
        far_overall: f64,
        phi_primary: f64,
    ) -> (f64, f64) {
        let g_ceiling = two_stream_ceiling(far_overall, phi_primary);
        let g = transport_variance(
            g_ceiling,
            self.coverage_omega(c),
            self.tau_mix,
            self.c_phi,
            self.n_ode,
        );
        (g, g_ceiling)
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
    /// rung 19 + 20: lift the primary [O] by m(T_p), AND the quench path's by m(T(β))
    pub super_eq_o: bool,
    /// rung 19: the imposed prompt bump at `phi_primary`
    pub prompt: Option<PromptNo>,
    /// RK4 steps for the primary Zeldovich integrator
    pub nsteps: usize,
    /// rung 10: a FINITE quench time, s. `None` ⇒ the IDEAL quench ⇒ bit-for-bit rung 9.
    /// MUTUALLY EXCLUSIVE with `mixing`.
    pub tau_q: Option<f64>,
    /// rung 11: DERIVE `τ_q` and the entrainment schedule from jet-in-crossflow physics
    pub mixing: Option<JetMixing>,
    /// rung 12: the two-stream spatial-variance layer. REQUIRES `mixing`.
    pub unmixedness: Option<Unmixedness>,
    /// rung 13: the resolved mixing β-PDF on the IDEAL bell. REQUIRES `mixing`.
    pub pdf: Option<MixingPdf>,
    /// rung 15: that β-PDF THROUGH the quench, with a LINEARISED dwell. REQUIRES `mixing`.
    pub pdf_quench: Option<QuenchPdf>,
    /// rung 16: the same, PER POCKET — the dwell inside the chemistry. REQUIRES `mixing`.
    pub pocket_quench: Option<PocketQuenchPdf>,
    /// rung 18: the width from a variance-decay ODE instead of the kink. REQUIRES `mixing`.
    pub transported: Option<TransportedPdf>,
    /// finite-quench trajectory points — a pure cost/accuracy knob
    pub quench_ngrid: usize,
    /// finite-quench RK4 steps — a pure cost/accuracy knob
    pub quench_nsteps: usize,
}

impl Default for ZonedNoxOpts {
    fn default() -> Self {
        Self {
            tau: 3e-3,
            super_eq_o: false,
            prompt: None,
            nsteps: 4000,
            tau_q: None,
            mixing: None,
            unmixedness: None,
            pdf: None,
            pdf_quench: None,
            pocket_quench: None,
            transported: None,
            quench_ngrid: 240,
            quench_nsteps: 2000,
        }
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
    // RUNG 10 — the finite-rate quench. ALL `None` for the ideal quench (`tau_q: None`,
    // `mixing: None`), which is the short-circuit that makes the reduce exact.
    /// quench (dilution-mixing) time, s — the free rung-10 knob, or rung 11's DERIVED one
    pub tau_q: Option<f64>,
    /// EI_NO re-made along the finite quench, g NO/kg fuel — the MEAN-FIELD (rung-11) bulk
    pub ei_no_quenched: Option<f64>,
    /// NO mole fraction frozen at the end of the finite quench
    pub x_no_quenched: Option<f64>,
    /// peak T along the quench path, K — `> T_primary` for a RICH primary
    pub t_peak: Option<f64>,
    /// max `[NO]/[NO]_e` along the path; `< 1` ⇒ the dropped clamp is dormant
    pub max_a_quench: Option<f64>,
    /// RUNG 11 — the jet-in-crossflow config that DERIVED `tau_q` + the schedule
    pub mixing: Option<JetMixing>,
    // RUNG 12 — spatial unmixedness. `ei_no_quenched` then holds the mean-field BULK and
    // `ei_no_unmixed` the two-stream total — the one that TURNS BACK UP in J.
    /// the variance config that split bulk/core
    pub unmixedness: Option<Unmixedness>,
    /// the Holdeman group `C = (S/H)√J` at this J
    pub c_holdeman: Option<f64>,
    /// the un-mixed core mass fraction `w(C)` (0 at `C_opt`)
    pub w_core: Option<f64>,
    /// two-stream EI_NO, g/kg — `(1−w)·EI(τ_mean) + w·EI(τ_core)`
    pub ei_no_unmixed: Option<f64>,
    /// the lingering core's EI_NO at `τ_core(C)` — the penalty source
    pub ei_no_core: Option<f64>,
    // RUNG 13 — the resolved mixing PDF. `ei_no_quenched` still holds the mean-field bulk
    // reference; `ei_no_pdf` is the β-PDF integral over the IDEAL bell, whose minimum is
    // PINNED AT `C_opt` (both flanks up) — the optimum LOCATION from a continuous distribution
    // rather than two lumps. Its far-over-penetration flank DESCENDS; the climb was rung-12's
    // dwell, which this rung deliberately drops.
    /// the β-PDF config used
    pub pdf: Option<MixingPdf>,
    /// the segregation width `g(C)` (0 at `C_opt`) — reused by rungs 15/16/18
    pub g_seg: Option<f64>,
    /// ⟨EI⟩ over the β-PDF of the ideal bell, g/kg — min AT `C_opt`
    pub ei_no_pdf: Option<f64>,
    // RUNG 15 — the PDF THROUGH the finite quench (composition variance AND dwell, COMBINED).
    // `ei_no_quenched` is term 1, the FINITE floor rung 13 lacked.
    /// the PDF-through-quench config used (`c_holdeman`/`g_seg` reused)
    pub pdf_quench: Option<QuenchPdf>,
    /// term 2 = `D(u)·⟨EI_bell⟩(g)`, g/kg — resolved composition × a LINEARISED dwell
    pub ei_no_pdf_excess: Option<f64>,
    /// term1 + term2, g/kg — the combined result (a finite floor, the far flank CLIMBING)
    pub ei_no_pdf_quench: Option<f64>,
    // RUNG 16 — the same, PER POCKET: each rich pocket through its OWN quench, so the dwell acts
    // inside the cooling chemistry and term 2 is SUBLINEAR in `τ_core`.
    /// the per-pocket PDF-through-quench config used
    pub pocket_quench: Option<PocketQuenchPdf>,
    /// term 2 — the per-pocket quench β-PDF integral, g/kg
    pub ei_no_pocket_excess: Option<f64>,
    /// term1 + term2, g/kg — erodes rung-15's far flank into near-degeneracy with the notch
    pub ei_no_pocket_quench: Option<f64>,
    // RUNG 18 — the TRANSPORTED-variance closure. The width is no longer the imposed kink but
    // the residual of a decay ODE from a DERIVED ceiling: a SMOOTH basin ELEVATED off the
    // well-mixed value. `ei_no_pdf` is NOT set here (a different closure); `g_seg` is reused.
    /// the transported-variance config used
    pub transported: Option<TransportedPdf>,
    /// the DERIVED two-stream injection ceiling `(ξ_p−ξ̄)/(1−ξ̄)` from φ_p
    pub g_ceiling: Option<f64>,
    /// the ODE-residual width `g(C)` — `≤ g_ceiling`, and `> 0` even at `C_opt`
    pub g_transported: Option<f64>,
    /// ⟨EI⟩ over the β-PDF of the ideal bell at `g_transported`
    pub ei_no_transported: Option<f64>,
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
    ///
    /// RUNG 10 — pass a finite `tau_q` to resolve the quench in TIME instead of collapsing it
    /// to an instant. As the dilution air mixes in, the LOCAL mixture sweeps
    /// `far_p → f_stoich → far_ov`, so a RICH primary's temperature RISES through the NO-bell
    /// peak and the extended-Zeldovich rate RE-MAKES NO along that path. A SLOW quench dwells
    /// at stoich and re-makes the NO the rich primary avoided; a FAST quench escapes past the
    /// peak — the whole point of "quick"-quench, and rung 9's rich-flank collapse is therefore
    /// CONTINGENT on it. `tau_q: None` is the IDEAL quench — the EXACT rung-9 path.
    ///
    /// RUNG 11 — pass a `mixing` config INSTEAD of `tau_q` to DERIVE the quench from
    /// jet-in-crossflow physics. `ei_no_quenched` falls MONOTONICALLY as J rises. Mean-field:
    /// it derives the quench RATE but has no mixing OPTIMUM.
    ///
    /// RUNG 12 — pass an `unmixedness` config (REQUIRES `mixing`) to add the two-stream
    /// VARIANCE layer, making the NO-vs-J curve TURN BACK UP and recovering the Holdeman
    /// optimum AT `C_opt`. `ei_no_quenched` still holds the monotone mean-field BULK reference;
    /// `ei_no_unmixed` is the two-stream total. `k_u = 0` is bit-for-bit rung 11.
    ///
    /// RUNG 20 — `super_eq_o` now ALSO lifts [O] along the quench path, so the finite-quench
    /// fields stop riding on the equilibrium-O lower bound. `false` ⇒ byte-identical rung 10/11.
    ///
    /// `quench_ngrid`/`quench_nsteps` are pure cost/accuracy knobs, used only on a finite
    /// quench. The 240 default reproduces the anchor's worked example; the SHAPE (peak
    /// temperature, monotonicity) is settled by ~32 points, so tests run coarse — a 240-point
    /// trajectory is 240 mix-out bisections, each re-solving the 8-species Newton ~31 times.
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
        assert!(
            !(o.tau_q.is_some() && o.mixing.is_some()),
            "pass EITHER tau_q (rung-10 free time + linear schedule) OR mixing (rung-11 \
             jet-entrainment: DERIVES τ_q + a decelerating schedule) — mutually exclusive."
        );
        assert!(
            !(o.unmixedness.is_some() && o.mixing.is_none()),
            "unmixedness (rung-12 spatial variance) REQUIRES a `mixing` config — it needs the \
             jet's J and duct H for the Holdeman group C=(S/H)√J and the mean-field bulk τ_mean."
        );
        assert!(
            !(o.pdf.is_some() && o.mixing.is_none()),
            "pdf (rung-13 resolved mixing PDF) REQUIRES a `mixing` config — it needs the jet's J \
             and duct H for the Holdeman group C=(S/H)√J that sets the β-PDF width g(C)."
        );
        assert!(
            !(o.pdf_quench.is_some() && o.mixing.is_none()),
            "pdf_quench (rung-15 PDF through the quench) REQUIRES a `mixing` config — it needs \
             the Holdeman group C=(S/H)√J AND the derived τ_mean for the mean-field floor."
        );
        assert!(
            !(o.pocket_quench.is_some() && o.mixing.is_none()),
            "pocket_quench (rung-16 PER-POCKET PDF through the quench) REQUIRES a `mixing` \
             config — it needs the Holdeman group C=(S/H)√J AND the derived τ_mean floor."
        );
        assert!(
            !(o.transported.is_some() && o.mixing.is_none()),
            "transported (rung-18 transported-variance closure) REQUIRES a `mixing` config — it \
             needs the Holdeman group C=(S/H)√J that the imposed coverage ω(C) rides on."
        );
        // AT MOST ONE closure. Slice B's version of this file recorded that the check was
        // DELIBERATELY omitted while `unmixedness` was the only closure ported, because
        // comparing one Option against one cannot fail and a bar that cannot fail is not a bar
        // (rungs 78/79's vacuity lesson) — and promised it would arrive WITH the second closure.
        // Five are ported now, so it is live, and it is the same guard the Python writes over
        // its eight. Rungs 22/23/24's three spatial closures join this count in a later slice.
        let closures = [
            o.unmixedness.is_some(),
            o.pdf.is_some(),
            o.pdf_quench.is_some(),
            o.pocket_quench.is_some(),
            o.transported.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count();
        assert!(
            closures <= 1,
            "pass AT MOST ONE of unmixedness (rung-12 two-stream) / pdf (rung-13 β-PDF on the \
             ideal bell) / pdf_quench (rung-15 β-PDF THROUGH the quench, LINEARISED dwell) / \
             pocket_quench (rung-16 PER-POCKET quench, SCALAR dwell) / transported (rung-18 \
             transported variance) — closures of the SAME variance physics, {closures} given."
        );
        // RUNG 21 — the rung-20 forbid guard is DISCHARGED. `super_eq_o` now threads the SAME
        // Westenberg m(T) lift through the ideal-bell composition integrals too — rung 13's
        // `pdf`, rung 15's term 2, rung 18's `transported` — so `pdf_quench` is no longer a
        // half-lifted HYBRID: term 1 (lifted by rung 20) and term 2 (lifted here) both carry
        // m(T) and the sum is internally consistent. No forbid: `super_eq_o` combines with every
        // closure. The ideal-bell lift is peak-concentrated and BELOW the primary's, because the
        // integral is EI-weighted onto the near-stoich peak where m is at its MINIMUM — the
        // rung-20 inversion generalised to composition variance. `docs/rung21-spec.md`.
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

        let mut state = ZonedNoxState {
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
            tau_q: None,
            ei_no_quenched: None,
            x_no_quenched: None,
            t_peak: None,
            max_a_quench: None,
            mixing: None,
            unmixedness: None,
            c_holdeman: None,
            w_core: None,
            ei_no_unmixed: None,
            ei_no_core: None,
            pdf: None,
            g_seg: None,
            ei_no_pdf: None,
            pdf_quench: None,
            ei_no_pdf_excess: None,
            ei_no_pdf_quench: None,
            pocket_quench: None,
            ei_no_pocket_excess: None,
            ei_no_pocket_quench: None,
            transported: None,
            g_ceiling: None,
            g_transported: None,
            ei_no_transported: None,
        };
        if o.tau_q.is_none() && o.mixing.is_none() {
            return state; // IDEAL quench — bit-for-bit rung 9
        }

        // RUNG 10/11 — the finite-rate quench: re-integrate NO (clamp-free) through the
        // cooling/mixing trajectory, starting from the primary's frozen NO. A pure diagnostic —
        // NO/N still never enter the equilibrium solve, so the cycle stays bit-for-bit rung 6.
        // Rung 10 = a free `tau_q` + linear schedule; rung 11 = `mixing` DERIVES
        // `τ_q = H/(C_e√J·U_c)` and a decelerating entrainment schedule (`mixing: None` ⇒
        // `schedule: None` ⇒ byte-identical rung 10).
        let (tau_q_eff, sched_fn): (f64, Option<Box<dyn Fn(f64) -> f64>>) = match o.mixing {
            Some(m) => {
                m.validate();
                (m.tau_q(), Some(Box::new(move |x| m.schedule(x))))
            }
            None => (o.tau_q.expect("checked above"), None),
        };
        assert!(
            tau_q_eff > 0.0,
            "tau_q {tau_q_eff} must be positive (or None for the ideal quench)"
        );

        // Rung 12 shares ONE τ_q-independent trajectory between the mean-field bulk and the
        // under-mixed core (both traverse the same β-path, differing only in τ). The
        // mean-field-only path lets `quench_no` build its own → byte-identical rung 10/11.
        let tab: Option<Vec<QuenchPoint>> = o.unmixedness.map(|_| {
            quench_trajectory(&comp_p, t_p, alpha, far, tt3, p, o.quench_ngrid)
        });
        let sched_ref = sched_fn.as_deref();
        // RUNG 20 — `super_eq_o` lifts [O] INSIDE this re-making by m(T) along the cooling path
        // (`false` ⇒ byte-identical rung 10/11), so `ei_no_quenched` and the rung-12 core carry
        // the same lift the rung-19 primary already did — closing the "finite-quench fields ride
        // on equilibrium O" lower-bound seam. The lift is MODEST & PEAK-CONCENTRATED: the
        // Zeldovich re-making peaks at the hottest stoich crossing where m(T) is at its MINIMUM,
        // so the effective lift ≈ m(T_peak), even smaller than the rung-19 primary lift.
        // `docs/rung20-spec.md`. The NO ceiling `[NO]_e` is a THERMODYNAMIC quantity, untouched.
        let q = quench_no(
            &comp_p,
            t_p,
            alpha,
            far,
            tt3,
            p,
            n_no_total,
            tau_q_eff,
            QuenchOpts {
                nsteps: o.quench_nsteps,
                ngrid: o.quench_ngrid,
                tab: tab.as_deref(),
                schedule: sched_ref,
                super_eq_o: o.super_eq_o,
            },
        );
        state.tau_q = Some(tau_q_eff);
        state.mixing = o.mixing;
        state.ei_no_quenched = Some(q.ei); // the MEAN-FIELD (rung-11) bulk EI
        state.x_no_quenched = Some(q.x_no_mix);
        state.t_peak = Some(q.t_peak);
        state.max_a_quench = Some(q.max_a);

        // The five closures below are mutually exclusive (asserted above) and each returns, so
        // the order is the Python's own branch order and not a precedence. Rungs 22/23/24's
        // spatial closures sit between `pocket_quench` and `transported` in the Python; they
        // arrive in a later slice and slot into the same place.
        let mixing_cfg = o.mixing;

        if let Some(pdf) = o.pdf {
            // RUNG 13 — the RESOLVED MIXING PDF, replacing rung-12's parameterised segregation
            // with a continuous distribution: integrate the IDEAL primary bell EI(φ) over a
            // mean-preserving β-PDF of mixture fraction whose single width is the segregation
            // `g(C) = min(g_max, k_g·|ln(C/C_opt)|)` — KINKED at the Holdeman optimum, so
            // ⟨EI⟩(g(C)) collapses to the well-mixed value AT `C_opt` (g=0 ⇒ delta ⇒ point
            // value) with both flanks lifting. Isolates the COMPOSITION mechanism and drops the
            // dwell chain, so it pins the optimum but does NOT climb. A pure diagnostic: NO/N
            // still never enter the equilibrium solve, so the cycle stays bit-for-bit rung 6.
            pdf.validate();
            let mixing = mixing_cfg.expect("pdf REQUIRES mixing — asserted above");
            let c = pdf.c(&mixing);
            let g_seg = pdf.segregation(c);
            state.pdf = Some(pdf);
            state.c_holdeman = Some(c);
            state.g_seg = Some(g_seg);
            state.ei_no_pdf = Some(pdf_mean_ei(
                far, tt3, p, hf_fuel, o.tau, g_seg, pdf.n_bell, pdf.n_quad,
                o.super_eq_o, // rung 21: lift the ideal bell
            ));
            return state;
        }

        if let Some(qp) = o.pdf_quench {
            // RUNG 15 — the PDF THROUGH the finite quench: carry rung-13's β-PDF through the
            // rung-10/12 dwell chain, COMBINING composition variance with the dwell. Additive:
            //   term 1 = `ei_no_quenched` (the rung-11 mean-field bulk — the FINITE floor rung
            //            13 lacked, present at all C);
            //   term 2 = D(u)·⟨EI_bell⟩(g) (the rung-13 integral REUSED VERBATIM, scaled by the
            //            off-optimum-growing dwell factor; EI ∝ τ, the dormant clamp).
            // The bell's reference residence IS `o.tau`, so the two stay locked. At `C_opt`
            // (g→0) term 2 → 0 and ⟨EI⟩ is the FINITE bulk NO, not rung-13's ≈0; off-optimum the
            // NONLINEAR bell keeps the stoich-mean SIGN REVERSAL that a lumped-dwell rung 12
            // cannot, and the dwell growth makes the far flank CLIMB.
            qp.validate();
            let mixing = mixing_cfg.expect("pdf_quench REQUIRES mixing — asserted above");
            let c = qp.c(&mixing);
            let g_seg = qp.segregation(c);
            let bell_mean_ei = pdf_mean_ei(
                far, tt3, p, hf_fuel, o.tau, g_seg, qp.n_bell, qp.n_quad,
                o.super_eq_o, // rung 21: lift term 2's ideal bell
            );
            let term2 = qp.dwell_factor(c, o.tau) * bell_mean_ei;
            state.pdf_quench = Some(qp);
            state.c_holdeman = Some(c);
            state.g_seg = Some(g_seg);
            state.ei_no_pdf_excess = Some(term2);
            state.ei_no_pdf_quench = Some(q.ei + term2); // term1 + term2
            return state;
        }

        if let Some(pq) = o.pocket_quench {
            // RUNG 16 — the PDF through the finite quench, PER POCKET, retiring rung-15's
            // linearised dwell. Rung 15 scaled the CONSTANT-T bell by a scalar `D(u)`, exact only
            // while EI ∝ τ. Rung 16 carries EACH rich-of-mean β-PDF pocket through its OWN finite
            // quench at the dwell `τ_core(C)`, so the dwell acts INSIDE the cooling chemistry.
            // Additive, mirroring rung 15 — only term 2's internals change. Because a lingering
            // pocket COOLS, term 2 is SUBLINEAR in `τ_core`, which erodes the over-penetration
            // flank into near-degeneracy with the `C_opt` notch. At `C_opt` (g→0) term 2 is the
            // single lean pocket at ξ̄ ≈ 0, so the total is the finite bulk floor.
            pq.validate();
            let mixing = mixing_cfg.expect("pocket_quench REQUIRES mixing — asserted above");
            let c = pq.c(&mixing);
            let g_seg = pq.segregation(c);
            let tau_core = pq.core_dwell(c);
            let (excess, pocket_max_a) = pocket_quench_mean_ei(
                far,
                tt3,
                p,
                hf_fuel,
                o.tau,
                tau_core,
                g_seg,
                pq.n_quad,
                PocketOpts {
                    n_bell: pq.n_bell,
                    quench_ngrid: o.quench_ngrid,
                    quench_nsteps: o.quench_nsteps,
                    super_eq_o: o.super_eq_o, // rung 20: lift each pocket's re-making
                },
            );
            state.pocket_quench = Some(pq);
            state.c_holdeman = Some(c);
            state.g_seg = Some(g_seg);
            state.ei_no_pocket_excess = Some(excess);
            state.ei_no_pocket_quench = Some(q.ei + excess); // term1 + term2
            // the dormancy gate spans the pockets as well as the bulk
            state.max_a_quench = Some(q.max_a.max(pocket_max_a));
            return state;
        }

        if let Some(tr) = o.transported {
            // RUNG 18 — the TRANSPORTED-variance closure. The β-PDF width `g(C)` is no longer the
            // imposed kink but the residual of a variance DECAY ODE from a DERIVED two-stream
            // ceiling, fed through the SAME rung-13 ideal bell. What it adds over the kink: a
            // ceiling DERIVED from φ_p rather than the free `g_max`; a RESIDUAL floor
            // `g(C_opt) > 0`, so the optimum is ELEVATED off the well-mixed value; and a SMOOTH
            // basin, so the kink's sharpness was the artifact and not its location. The `C_opt`
            // LOCATION still rides on the IMPOSED spatial coverage ω(C) — a 0-D transport cannot
            // derive it, which is this rung's load-bearing NEGATIVE result.
            tr.validate();
            let mixing = mixing_cfg.expect("transported REQUIRES mixing — asserted above");
            let c = tr.c(&mixing);
            let (g_seg, g_ceiling) = tr.segregation(c, far, phi_primary);
            state.transported = Some(tr);
            state.c_holdeman = Some(c);
            state.g_ceiling = Some(g_ceiling);
            state.g_transported = Some(g_seg);
            state.g_seg = Some(g_seg);
            state.ei_no_transported = Some(pdf_mean_ei(
                far, tt3, p, hf_fuel, o.tau, g_seg, tr.n_bell, tr.n_quad,
                o.super_eq_o, // rung 21: the same lift threads the same bell
            ));
            return state;
        }

        let Some(um) = o.unmixedness else {
            return state; // rung 10/11 mean field — untouched
        };

        // RUNG 12 — the under-mixed CORE (spatial variance): the SAME cooling trajectory, but
        // the core misses the jet and quenches at an ABSOLUTE residence `τ_core(C)` (NOT the
        // vanishing jet time, so its NO penalty survives J→∞) that GROWS off-optimum.
        // Mass-weight bulk/core by the KINKED segregated fraction `w(C)`, whose non-zero slope
        // at `C_opt` PINS the EI-min AT the Holdeman optimum (`J_min = J_opt`, shifting as
        // `(H/S)²`) → EI_NO turns back up. Still a pure diagnostic: cycle bit-for-bit rung 6.
        um.validate();
        let mixing = o.mixing.expect("unmixedness REQUIRES mixing — asserted above");
        let c = um.c(&mixing);
        let w = um.core_fraction(c);
        let qc = quench_no(
            &comp_p,
            t_p,
            alpha,
            far,
            tt3,
            p,
            n_no_total,
            um.core_dwell(c),
            QuenchOpts {
                nsteps: o.quench_nsteps,
                ngrid: o.quench_ngrid,
                tab: tab.as_deref(),
                schedule: sched_ref,
                super_eq_o: o.super_eq_o, // rung 20: lift the lingering core's re-making too
            },
        );
        state.unmixedness = Some(um);
        state.c_holdeman = Some(c);
        state.w_core = Some(w);
        state.ei_no_core = Some(qc.ei); // the lingering-core EI at τ_core(C)
        state.ei_no_unmixed = Some((1.0 - w) * q.ei + w * qc.ei);
        // the dormancy gate spans BOTH streams
        state.max_a_quench = Some(q.max_a.max(qc.max_a));
        state
    }
}
