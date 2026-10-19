//! RUNG 102 — THE AFTERBURNER: a second burner between the turbine and the nozzle.
//!
//! **The rung** (`docs/rung102-spec.md`, anchor `docs/plans/rung102-anchor-afterburner.md`). The gas leaving the
//! turbine still carries most of its oxygen — the main burner runs lean, `φ ≈ 0.4`, so the turbine survives. Burning
//! more fuel behind the turbine heats the jet again at a lower pressure: more thrust, at a worse fuel rate. Three
//! parts, one rung: **(A)** the design point (this module's builder), **(B)** the one-shaft engine flown off design
//! with the afterburner inside the nozzle relation, **(C)** the two-shaft slam with a light-off.
//!
//! Stations (industry numbering, Mattingly Fig. 7.8): 5 turbine exit → 6 burning-zone entry → 7 afterburner exit →
//! 9 nozzle exit. 5 → 6 is a diffuser, isentropic (Mattingly's own assumption): `Tt6 = Tt5`, `pt6 = pt5`, so 6 does
//! not get its own row in the station table.
//!
//! **The fuel** (energy, 6 → 7 — the main burner's balance with vitiated air in, the fuel's own enthalpy neglected as
//! there):
//!
//! ```text
//! (1+f5)·h_t(Tt6, f5) + η_AB·(f7 − f5)·h_PR = (1+f7)·h_t(Tt7, f7)        # f7 the TOTAL fuel per unit air
//! ```
//!
//! On a calorically perfect gas this is Mattingly's (7.29) exactly, `f_AB = f7 − f5`. On the reacting gas `h_t` depends
//! on `f7`, so `f7 = g(f7)` — `g` is increasing and a contraction (the main burner's argument), so iterates seeded at
//! `f5` RISE monotonically onto the root: an iterate past the stoichiometric cap means the root is past it too, and
//! the request is refused rather than evaluated out of the gas's range. On the equilibrium gas the scale-B absolute
//! balance per mol of air, the reactant being the station-4 FROZEN mixture at `Tt6` plus the added fuel, the product the
//! equilibrium mixture at `(f7, Tt7, pt7)` — bisection on `f7 ∈ [f5, f_stoich)`.
//!
//! **The duct loss** (6 → 7): `π_AB = π_fh · π_R`, a fixed flame-holder ratio times the loss of heating a moving gas in
//! a constant-area duct (Rayleigh flow), solved from the three conservation laws for the subsonic exit state:
//!
//! ```text
//! mass       ρ6·V6·(1+f7)/(1+f5) = ρ7·V7          # the injected fuel adds mass, not axial momentum
//! momentum   p6 + ρ6·V6²         = p7 + ρ7·V7²    # no wall friction (the flame holders are π_fh)
//! energy     h7(T7) + V7²/2      = h_t(Tt7, f7)
//! ```
//!
//! Heat added to a subsonic stream accelerates it toward Mach 1; past the heat the duct can take it CHOKES — a
//! refusal. `M6 = 0` gives `π_R = 1` by dispatch: with `π_fh` 0.94 that is Mattingly's fixed `π_AB`, the anchor's path.
//!
//! **The nozzle** sees `(Tt7, pt7, f7)`; on the equilibrium gas the afterburner freezes its own mixture into the gas
//! (`gas.rs`'s guard is per fuel/air ratio since this rung). **Thrust** books the TOTAL fuel ([`try_score_total`]).
//!
//! Reduces: no afterburner is `build_turbojet` itself (by dispatch — this module is never entered); [`AbMode::Unlit`]
//! with `π_fh` 1 gives that run's station 9 and performance bit for bit.

use crate::components::{Component, Nozzle, NozzleExit};
use crate::engine::{build_turbojet, Engine, EngineResult, FlightCondition, Losses, Performance};
use crate::gas::{powp, Abort, FlowState, Gas};

/// The afterburner's knobs (`docs/rung102-spec.md` § The device).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Afterburner {
    /// Afterburner exit (nozzle entry) total temperature when lit, K.
    pub tt7: f64,
    /// Combustion efficiency of the afterburner, ≤ 1.
    pub eta_ab: f64,
    /// Flame-holder (and diffuser) total-pressure ratio, lit or not, ≤ 1.
    pub pi_fh: f64,
    /// Mach number entering the burning zone, after the diffuser. 0 ⇒ no heat-addition loss.
    pub m6: f64,
}

impl Default for Afterburner {
    /// The anchor's defaults (G5): `π_fh` 0.97 with `M6` 0.20 gives 0.944 lit at a doubling — Mattingly Problem 7.9's
    /// 0.94.
    fn default() -> Self { Afterburner { tt7: 2000.0, eta_ab: 0.95, pi_fh: 0.97, m6: 0.20 } }
}

/// Fitted-and-unlit, or burning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbMode {
    /// The duct is there (its flame holders cost `π_fh`) but no fuel burns: `Tt7 = Tt5`, `f7 = f5`.
    Unlit,
    /// Burning to [`Afterburner::tt7`].
    Lit,
}

/// What the afterburner did, beside the station table.
#[derive(Clone, Copy, Debug)]
pub struct AbDetail {
    pub mode: AbMode,
    /// Fuel/air ratio arriving (the main burner's) and leaving (the TOTAL), per unit air.
    pub f5: f64,
    pub f7: f64,
    /// `pt7/pt5` and its heat-addition part.
    pub pi_ab: f64,
    pub pi_r: f64,
    /// Mach entering and leaving the burning zone.
    pub m6: f64,
    pub m7: f64,
    /// Passes of the equilibrium gas's `pt7` ↔ composition iteration (1 on every other gas).
    pub passes: usize,
}

/// The whole run: the station table 0 2 3 4 5 7 9, performance on the total fuel, and the core run it continued.
#[derive(Clone, Debug)]
pub struct AbResult {
    pub stations: Vec<(&'static str, FlowState)>,
    pub performance: Performance,
    pub v0: f64,
    pub v9: f64,
    pub m9: f64,
    pub t9: f64,
    pub p9: f64,
    pub detail: AbDetail,
}

impl AbResult {
    pub fn station(&self, label: &str) -> &FlowState {
        self.stations.iter().find(|&&(l, _)| l == label)
            .map(|(_, s)| s)
            .unwrap_or_else(|| panic!("no station {label:?} in the table"))
    }
}

/// The equilibrium `pt7` iteration's pass cap. The anchor's § 4 trigger is 10 passes; the cap is wider so a slow case
/// is REPORTED (the pass count rides out) rather than refused.
pub const AB_PASSES_MAX: usize = 50;
/// Its stopping rule, relative on `pt7`.
pub const AB_PT7_TOL: f64 = 1e-10;
const FP_TOL: f64 = 1e-12;
const FP_MAX: usize = 100;

/// The fuel cap on EVERY gas — the main burner's own bracket. CPG and TPG keep no oxygen books, but the fuel runs out
/// all the same; one cap makes "too hot" mean the same thing on every gas.
pub fn f_cap(gas: &Gas) -> f64 { gas.f_stoich_lean() * (1.0 - 1e-6) }

fn too_hot(tt7: f64, ceiling: f64) -> Abort {
    Abort(format!("The afterburner cannot reach {tt7:.0} K: burning all the oxygen left in the gas gives at most \
                   {ceiling:.0} K here."))
}

fn too_cold(tt7: f64, arriving: f64) -> Abort {
    Abort(format!("The afterburner cannot cool the gas: it arrives at {arriving:.0} K, and {tt7:.0} K was asked."))
}

/// The fuel on a gas without equilibrium chemistry (CPG, frozen TPG, reacting): the fixed point of the energy balance.
///
/// `Ok(None)` when the root lies past the stoichiometric cap (the caller names the ceiling).
pub fn fuel_frozen(gas: &Gas, tt5: f64, f5: f64, tt7: f64, eta_ab: f64) -> Result<Option<f64>, Abort> {
    let cap = f_cap(gas);
    let h5 = gas.h_t(tt5, f5);
    let mut f = f5;
    for _ in 0..FP_MAX {
        let h7 = gas.h_t(tt7, f);
        let f_new = f5 + (1.0 + f5) * (h7 - h5) / (eta_ab * gas.hpr() - h7);
        // Monotone from below (module docs): past the cap here means past it at the root.
        if !(f_new < cap) {
            return Ok(None);
        }
        if (f_new - f).abs() <= FP_TOL * f_new {
            return Ok(Some(f_new));
        }
        f = f_new;
    }
    Err(Abort(format!("afterburner fixed point f7=g(f7) did not converge in {FP_MAX} steps")))
}

/// The hottest `Tt7` a frozen-chemistry gas reaches at the cap — the energy balance solved for `T` at `f7 = cap`.
pub fn ceiling_frozen(gas: &Gas, tt5: f64, f5: f64, eta_ab: f64) -> Result<f64, Abort> {
    let cap = f_cap(gas);
    let h = ((1.0 + f5) * gas.h_t(tt5, f5) + eta_ab * (cap - f5) * gas.hpr()) / (1.0 + cap);
    gas.try_t_from_h_t(h, cap)
}

/// The scale-B absolute residual per mol of air (reactant − product − loss) at a trial `(f7, T, p)`. Positive ⇒ the
/// reactants carry more enthalpy than the products at `T` (less fuel, or a hotter `T`, closes it).
fn eq_residual(gas: &Gas, comp4: &[(&'static str, f64)], tt5: f64, f5: f64, f7: f64, t: f64, p: f64, eta_ab: f64)
    -> Result<(f64, f64), Abort>
{
    let comp = gas.try_equilibrium_composition(f7, t, p)?;
    let dn = gas.n_fuel_per_air(f7) - gas.n_fuel_per_air(f5);
    let prod = gas.h_products_abs_b(&comp, t);
    let res = gas.h_products_abs_b(comp4, tt5) + dn * gas.hf_fuel_molar() - prod - (1.0 - eta_ab) * dn * gas.lhv_molar();
    Ok((res, prod))
}

/// [`eq_residual`] at the stoichiometric cap. The rung-6 equilibrium Newton fails at a few isolated points EXACTLY at
/// `f_stoich·(1 − 1e−6)` (measured: φ 0.999999 at 1500 K and 3.2 bar, while φ 0.999 converges everywhere) — the main
/// burner never probes its bracket's edge, the afterburner's too-hot test does. Only when the Newton fails there does
/// it step back to `1e−5`, `1e−4`, then `1e−3` below stoichiometric: a sign test, and a ceiling within about a kelvin.
fn eq_residual_at_cap(gas: &Gas, comp4: &[(&'static str, f64)], tt5: f64, f5: f64, t: f64, p: f64, eta_ab: f64)
    -> Result<f64, Abort>
{
    let mut last = None;
    for back in [1e-6, 1e-5, 1e-4, 1e-3] {
        match eq_residual(gas, comp4, tt5, f5, gas.f_stoich_lean() * (1.0 - back), t, p, eta_ab) {
            Ok((r, _)) => return Ok(r),
            Err(e) => last = Some(e),
        }
    }
    Err(last.expect("three tries"))
}

/// Bisection on `T` for the root of a residual DECREASING in `T` on `[lo, hi]`, to a few ulps: the duct's momentum
/// closure (1e−9) is read off the root, and a 1e−10 bracket left 7e−9 there.
fn bisect_t(lo: f64, hi: f64, mut r: impl FnMut(f64) -> Result<f64, Abort>) -> Result<f64, Abort> {
    let (mut lo, mut hi) = (lo, hi);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if r(mid)? > 0.0 { lo = mid } else { hi = mid }
        if hi - lo <= 1e-14 * hi { break; }
    }
    Ok(0.5 * (lo + hi))
}

/// What a lit equilibrium afterburner can be asked: `Ok(f7)`, or the refusal in words (too cold — below the frozen
/// mixture's own recombination heat — or too hot, the ceiling named at this pressure).
pub fn fuel_equilibrium(
    gas: &Gas, comp4: &[(&'static str, f64)], tt5: f64, f5: f64, tt7: f64, p7: f64, eta_ab: f64,
) -> Result<f64, Abort> {
    let cap = f_cap(gas);
    let (r_lo, _) = eq_residual(gas, comp4, tt5, f5, f5, tt7, p7, eta_ab)?;
    if r_lo > 0.0 {
        // No added fuel already leaves the products short of the reactants: the frozen mixture, re-equilibrated,
        // releases heat (anchor G2). The temperature it reaches unaided is the floor.
        let floor = bisect_t(tt5, tt5 + 200.0, |t| eq_residual(gas, comp4, tt5, f5, f5, t, p7, eta_ab).map(|r| r.0))?;
        return Err(too_cold(tt7, floor));
    }
    // Bisection on `f7 ∈ [f5, cap)`, the cap itself never asked for up front: the rung-6 Newton fails near
    // stoichiometric at low temperature and high pressure (the crash map: 1540–1575 K, 5.8–8.6 bar), where no afterburner
    // that runs ever probes. The search approaches the cap only when the root lies at or past it — then the cap is
    // tested, and the ceiling named.
    let (mut lo, mut hi) = (f5, cap);
    let (mut f, mut res, mut prod) = (f5, 0.0f64, 1.0f64);
    let mut found_hi = false;          // a trial on the lean side of the root has been seen (res ≥ 0)
    for _ in 0..FP_MAX {
        f = 0.5 * (lo + hi);
        (res, prod) = match eq_residual(gas, comp4, tt5, f5, f, tt7, p7, eta_ab) {
            Ok(rp) => rp,
            // The Newton fails only hugging stoichiometric: the root is up there, so the cap decides.
            Err(_) if !found_hi && f > cap * (1.0 - 1e-3) => break,
            Err(e) => return Err(e),
        };
        if hi - lo <= FP_TOL * f { break; }
        if res < 0.0 { lo = f } else { hi = f; found_hi = true; }
    }
    if !found_hi && eq_residual_at_cap(gas, comp4, tt5, f5, tt7, p7, eta_ab)? < 0.0 {
        let ceiling = bisect_t(tt5, 4000.0, |t| eq_residual_at_cap(gas, comp4, tt5, f5, t, p7, eta_ab))?;
        return Err(too_hot(tt7, ceiling));
    }
    if !(res.abs() < 1e-6 * prod.abs()) {
        return Err(Abort("rung-102 equilibrium afterburner balance did not close".to_string()));
    }
    Ok(f)
}

/// The static state at a total `(Tt, pt)` moving at Mach `m`, on the hot section at `far`: `(T, p, V)`.
fn static_at_mach(gas: &Gas, far: f64, tt: f64, pt: f64, m: f64) -> (f64, f64, f64) {
    let r = gas.r_t_at(far);
    let h_tt = gas.h_t(tt, far);
    let resid = |t: f64| (h_tt - gas.h_t(t, far)) - 0.5 * (m * m) * gas.gamma_t_at(t, far) * r * t;
    let t = bisect_t(0.5 * tt, tt, |t| Ok(resid(t))).expect("infallible residual");
    let v = m * powp(gas.gamma_t_at(t, far) * r * t, 0.5);
    let p = pt * gas.pr_t(t, far) / gas.pr_t(tt, far);
    (t, p, v)
}

/// The constant-area heat-addition (Rayleigh) duct, the three laws solved directly (module docs). Inlet on `gas_in` at
/// `f5`, outlet on `gas_out` at `f7` (the same `Gas` except during the equilibrium iteration). `mass_factor` is
/// `(1+f7)/(1+f5)` in production; the gate passes 1 to compare the textbook closed form.
///
/// Returns `Ok(Some((π_R, M7)))`, or `Ok(None)` when the heat would choke the duct (no subsonic exit state).
#[allow(clippy::too_many_arguments)]
pub fn rayleigh(
    gas_in: &Gas, f5: f64, gas_out: &Gas, f7: f64, tt6: f64, m6: f64, tt7: f64, mass_factor: f64,
) -> Result<Option<(f64, f64)>, Abort> {
    if m6 == 0.0 {
        return Ok(Some((1.0, 0.0)));        // a stagnant duct: heat costs no pressure (the reduce)
    }
    // The ratio is pressure-level independent (ideal gas, frozen composition): solve at pt6 = 1.
    let (t6, p6, v6) = static_at_mach(gas_in, f5, tt6, 1.0, m6);
    let rho6 = p6 / (gas_in.r_t_at(f5) * t6);
    let g7 = rho6 * v6 * mass_factor;
    let impulse = p6 + rho6 * v6 * v6;
    let r7 = gas_out.r_t_at(f7);
    let h7t = gas_out.h_t(tt7, f7);
    // The exit state is parametrised by its kinetic energy `u = V7²/2` (the static temperature follows from the energy
    // law), not by `T7`: at a slow duct `Tt7 − T7` is a millionth of `T7`, and a bracket on `T7` read the velocity to
    // only 1e−8 (the crash map's Mach-0.002 ducts failed the momentum closure that way).
    let temp = |u: f64| gas_out.try_t_from_h_t(h7t - u, f7);
    let momentum = |u: f64| -> Result<f64, Abort> { let v = powp(2.0 * u, 0.5); Ok(g7 / v * r7 * temp(u)? + g7 * v - impulse) };
    // The sonic exit: the momentum residual's minimum along the Rayleigh line.
    let t_star = bisect_t(0.5 * tt7, tt7,
                          |t| Ok((h7t - gas_out.h_t(t, f7)) - 0.5 * gas_out.gamma_t_at(t, f7) * r7 * t))?;
    let u_star = h7t - gas_out.h_t(t_star, f7);
    if momentum(u_star)? > 0.0 {
        return Ok(None);                    // thermal choke
    }
    // The subsonic root: between stagnation (where the residual runs to +∞) and the sonic point; the bracket's
    // stopping rule is relative to `u` itself, so a slow duct is resolved as finely as a fast one.
    let u7 = bisect_t(0.0, u_star, momentum)?;
    let (t7, v7) = (temp(u7)?, powp(2.0 * u7, 0.5));
    let p7 = g7 / v7 * r7 * t7;
    let m7 = v7 / powp(gas_out.gamma_t_at(t7, f7) * r7 * t7, 0.5);
    let pt7 = p7 * gas_out.pr_t(tt7, f7) / gas_out.pr_t(t7, f7);
    // Conservation, every call: momentum closes; energy and mass hold by construction (V from h, ρ from G).
    let mom = (p7 + g7 * v7 - impulse).abs();
    if !(mom <= 1e-9 * impulse) {
        return Err(Abort(format!("afterburner duct momentum does not close ({mom:.3e} of {impulse:.3e})")));
    }
    Ok(Some((pt7, m7)))
}

/// The hottest `Tt7` the duct takes before choking, by bisection on `Tt7` between the arriving gas and the request.
fn choke_ceiling(gas: &Gas, f5: f64, tt5: f64, m6: f64, tt7: f64, eta_ab: f64) -> Result<f64, Abort> {
    let (mut lo, mut hi) = (tt5, tt7);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        let ok = match fuel_frozen(gas, tt5, f5, mid, eta_ab)? {
            Some(f7) => rayleigh(gas, f5, gas, f7, tt5, m6, mid, (1.0 + f7) / (1.0 + f5))?.is_some(),
            None => false,
        };
        if ok { lo = mid } else { hi = mid }
        if hi - lo <= 0.05 { break; }
    }
    Ok(lo)
}

/// [`choke_ceiling`] on the equilibrium gas, each trial's mixture at the pressure `p` of the pass that choked (the
/// ceiling is named there; the duct's own few-percent drop moves dissociation by a few kelvin at most).
fn choke_ceiling_eq(gas: &Gas, s4: &FlowState, tt5: f64, f5: f64, ab: &Afterburner, p: f64) -> Result<f64, Abort> {
    let comp4 = gas.try_freeze_equilibrium(s4.far, s4.tt, s4.pt)?;
    let (mut lo, mut hi) = (tt5, ab.tt7);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        let ok = match fuel_equilibrium(gas, &comp4, tt5, f5, mid, p, ab.eta_ab) {
            Ok(f7) => {
                let scratch = gas.clone();
                scratch.try_freeze_equilibrium(s4.far, s4.tt, s4.pt)?;
                scratch.try_freeze_equilibrium(f7, mid, p)?;
                rayleigh(&scratch, f5, &scratch, f7, tt5, ab.m6, mid, (1.0 + f7) / (1.0 + f5))?.is_some()
            }
            // Below the recombination floor no heat is added, so nothing chokes; past the oxygen, nothing burns.
            Err(e) if e.0.contains("cannot cool") => true,
            Err(_) => false,
        };
        if ok { lo = mid } else { hi = mid }
        if hi - lo <= 0.05 { break; }
    }
    Ok(lo)
}

fn choked(tt7: f64, m6: f64, ceiling: f64) -> Abort {
    Abort(format!("Heating the gas to {tt7:.0} K would choke the afterburner duct (it enters at Mach {m6:.2}); the \
                   hottest it can take is {ceiling:.0} K."))
}

/// Station 5 → 7. `s4` is the main burner's exit (the equilibrium gas's frozen mixture is keyed there), `s5` the
/// turbine's, `mdot_air` the air (station 0). On the equilibrium gas a lit afterburner freezes its mixture into `gas`.
pub fn try_afterburn(
    gas: &Gas, s4: &FlowState, s5: &FlowState, mdot_air: f64, ab: &Afterburner, mode: AbMode,
) -> Result<(FlowState, AbDetail), Abort> {
    let (tt5, f5) = (s5.tt, s5.far);
    if mode == AbMode::Unlit {
        let s7 = FlowState { tt: tt5, pt: ab.pi_fh * s5.pt, mdot: s5.mdot, far: f5 };
        let detail = AbDetail { mode, f5, f7: f5, pi_ab: ab.pi_fh, pi_r: 1.0, m6: ab.m6, m7: ab.m6, passes: 1 };
        return Ok((s7, detail));
    }
    let tt7 = ab.tt7;
    if !(tt7 > tt5) {
        return Err(too_cold(tt7, tt5));
    }
    let (f7, pi_r, m7, passes) = if gas.is_equilibrium() {
        let comp4 = gas.try_freeze_equilibrium(s4.far, s4.tt, s4.pt)?;
        let mut p = ab.pi_fh * s5.pt;
        let mut out = None;
        for pass in 1..=AB_PASSES_MAX {
            let f7 = fuel_equilibrium(gas, &comp4, tt5, f5, tt7, p, ab.eta_ab)?;
            // The station-7 properties at this trial pressure, on a scratch copy (a clone starts with empty memos),
            // so the real gas freezes once, at the converged condition.
            let scratch = gas.clone();
            scratch.try_freeze_equilibrium(s4.far, s4.tt, s4.pt)?;
            scratch.try_freeze_equilibrium(f7, tt7, p)?;
            let Some((pi_r, m7)) = rayleigh(&scratch, f5, &scratch, f7, tt5, ab.m6, tt7, (1.0 + f7) / (1.0 + f5))?
            else {
                return Err(choked(tt7, ab.m6, choke_ceiling_eq(gas, s4, tt5, f5, ab, p)?));
            };
            let p_new = ab.pi_fh * pi_r * s5.pt;
            let done = (p_new - p).abs() <= AB_PT7_TOL * p;
            p = p_new;
            out = Some((f7, pi_r, m7, pass));
            if done { break; }
        }
        let (f7, pi_r, m7, passes) = out.expect("at least one pass");
        if passes == AB_PASSES_MAX {
            return Err(Abort(format!("afterburner pt7 iteration did not settle in {AB_PASSES_MAX} passes")));
        }
        (f7, pi_r, m7, passes)
    } else {
        let Some(f7) = fuel_frozen(gas, tt5, f5, tt7, ab.eta_ab)? else {
            return Err(too_hot(tt7, ceiling_frozen(gas, tt5, f5, ab.eta_ab)?));
        };
        let Some((pi_r, m7)) = rayleigh(gas, f5, gas, f7, tt5, ab.m6, tt7, (1.0 + f7) / (1.0 + f5))? else {
            return Err(choked(tt7, ab.m6, choke_ceiling(gas, f5, tt5, ab.m6, tt7, ab.eta_ab)?));
        };
        (f7, pi_r, m7, 1)
    };
    let pi_ab = ab.pi_fh * pi_r;
    let s7 = FlowState { tt: tt7, pt: pi_ab * s5.pt, mdot: mdot_air * (1.0 + f7), far: f7 };

    // Conservation, every run (contract #4).
    if gas.is_equilibrium() {
        let comp4 = gas.try_freeze_equilibrium(s4.far, s4.tt, s4.pt)?;
        let comp7 = gas.try_freeze_equilibrium(f7, tt7, s7.pt)?;
        let dn = gas.n_fuel_per_air(f7) - gas.n_fuel_per_air(f5);
        let react = gas.h_products_abs_b(&comp4, tt5) + dn * gas.hf_fuel_molar();
        let prod = gas.h_products_abs_b(&comp7, tt7);
        let loss = (1.0 - ab.eta_ab) * dn * gas.lhv_molar();
        if !((react - (prod + loss)).abs() < 1e-6 * prod.abs()) {
            return Err(Abort("rung-102 equilibrium afterburner balance: reactants != products + loss".to_string()));
        }
    } else {
        let lhs = (1.0 + f5) * gas.h_t(tt5, f5) + ab.eta_ab * (f7 - f5) * gas.hpr();
        let rhs = (1.0 + f7) * gas.h_t(tt7, f7);
        if !((lhs - rhs).abs() < 1e-9 * rhs.abs()) {
            return Err(Abort("rung-102 afterburner energy balance violated".to_string()));
        }
    }
    if !((s7.mdot - s5.mdot * (1.0 + f7) / (1.0 + f5)).abs() < 1e-9 * s7.mdot) {
        return Err(Abort("rung-102 afterburner mass: mdot7 != mdot5*(1+f7)/(1+f5)".to_string()));
    }
    let detail = AbDetail { mode, f5, f7, pi_ab, pi_r, m6: ab.m6, m7, passes };
    Ok((s7, detail))
}

/// [`crate::engine::try_score`] with the TOTAL fuel passed in — a deliberate COPY (same lines, same order), because the
/// shipped scorer reads `f` at station "4" and its callers' tables keep that meaning. `f_total = station 4's far`
/// reproduces it bit for bit (a gate).
#[allow(clippy::too_many_arguments)]
pub fn try_score_total(
    gas: &Gas, stations: &[(&'static str, FlowState)], f_total: f64, v0: f64,
    t9: f64, v9: f64, p9: f64, p0: f64, hpr: f64,
) -> Result<Performance, Abort> {
    let at = |label: &str| stations.iter().find(|&&(l, _)| l == label).map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("station {label:?} not reached yet"));
    let f = f_total;
    let pressure_thrust = (1.0 + f) * gas.r_t_at(f) * t9 * (1.0 - p0 / p9) / v9;
    let specific_thrust = (1.0 + f) * v9 - v0 + pressure_thrust;
    let tsfc = f / specific_thrust;
    let eta_brayton = 1.0 - at("2").tt / at("3").tt;
    let ke_net = (1.0 + f) * (v9 * v9) - (v0 * v0);
    let eta_thermal = ke_net / (2.0 * f * hpr);
    let eta_propulsive = (specific_thrust * v0) / (0.5 * ke_net);
    let eta_overall = (specific_thrust * v0) / (f * hpr);
    if !((eta_overall - eta_thermal * eta_propulsive).abs() < 1e-9 * eta_overall) {
        return Err(Abort(
            "efficiency cascade eta_o == eta_thermal*eta_p must hold under the KE definition".to_string()));
    }
    Ok(Performance { specific_thrust, tsfc, eta_brayton, eta_thermal, eta_propulsive, eta_overall })
}

/// The design-point afterburning turbojet: `build_turbojet`'s engine, run as shipped, continued from station 5.
#[derive(Clone)]
pub struct AfterburningTurbojet {
    /// The shipped engine — its run IS `build_turbojet(…).run(…)`, and its station 9 the dry, no-afterburner engine.
    pub engine: Engine,
    pub ab: Afterburner,
    /// The same nozzle the shipped engine ends with.
    pub nozzle: Nozzle,
}

/// Wire the afterburning turbojet (Part A).
pub fn build_afterburning_turbojet(
    gas: Gas, pi_c: f64, tt4: f64, p_ambient: f64, losses: Losses, ab: Afterburner,
) -> AfterburningTurbojet {
    let engine = build_turbojet(gas, pi_c, tt4, p_ambient, losses);
    let nozzle = match engine.components.last() {
        Some((_, Component::Nozzle(n))) => *n,
        _ => unreachable!("build_turbojet ends with its nozzle"),
    };
    AfterburningTurbojet { engine, ab, nozzle }
}

impl AfterburningTurbojet {
    /// The core alone: the shipped run (no afterburner).
    pub fn run_core(&self, flight: &FlightCondition, mdot: f64) -> EngineResult { self.engine.run(flight, mdot) }

    /// Continue a core run through the afterburner and the nozzle.
    pub fn try_continue(&self, core: &EngineResult, mode: AbMode) -> Result<AbResult, Abort> {
        let gas = &self.engine.gas;
        let (s4, s5) = (*core.station("4"), *core.station("5"));
        let mdot_air = core.station("0").mdot;
        let (s7, detail) = try_afterburn(gas, &s4, &s5, mdot_air, &self.ab, mode)?;
        let exit: NozzleExit = self.nozzle.try_apply(&s7, gas)?;
        let mut stations: Vec<(&'static str, FlowState)> =
            core.stations.iter().copied().filter(|&(l, _)| l != "9").collect();
        stations.push(("7", s7));
        stations.push(("9", exit.state));
        let p0 = self.nozzle.p_ambient;
        let performance = try_score_total(gas, &stations, s7.far, core.v0, exit.t9, exit.v9, exit.p9, p0, gas.hpr())?;
        Ok(AbResult { stations, performance, v0: core.v0, v9: exit.v9, m9: exit.m9, t9: exit.t9, p9: exit.p9, detail })
    }

    /// Run the design point with the afterburner in `mode`.
    pub fn try_run(&self, flight: &FlightCondition, mdot: f64, mode: AbMode) -> Result<AbResult, Abort> {
        let core = self.run_core(flight, mdot);
        self.try_continue(&core, mode)
    }

    pub fn run(&self, flight: &FlightCondition, mdot: f64, mode: AbMode) -> AbResult {
        self.try_run(flight, mdot, mode).unwrap_or_else(|e| panic!("{}", e.0))
    }
}
