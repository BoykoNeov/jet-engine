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

use std::borrow::Cow;
use std::cell::RefCell;

use crate::components::{ram_recovery, try_choked_mfp, Component, Nozzle, NozzleExit};
use crate::engine::{build_turbojet, Engine, EngineResult, FlightCondition, Losses, Performance};
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::map::ComponentMap;
use crate::matcher::Branch;
use crate::spool::{illinois, try_illinois, Instant, SpoolTransient, ILLINOIS_MAXIT};

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

/// [`bisect_t`]'s root to the same width (1e−14 of the bracket's larger end), by the Illinois false position — about
/// ten residuals where bisection takes about fifty (Part B: the duct is solved inside the Fly-it speed search, ~500
/// times a point, and bisection made a lit point cost 6–12× a dry one against the anchor's 3×). No sign change at the
/// ends ⇒ [`bisect_t`] itself (its end-point answer); no shared counters (rung 34's Illinois census stays its own).
fn root(lo: f64, hi: f64, mut r: impl FnMut(f64) -> Result<f64, Abort>) -> Result<f64, Abort> {
    let (mut a, mut b) = (lo, hi);
    let (mut fa, mut fb) = (r(a)?, r(b)?);
    if fa == 0.0 { return Ok(a); }
    if fb == 0.0 { return Ok(b); }
    if !(fa * fb < 0.0) {
        return bisect_t(lo, hi, r);
    }
    for _ in 0..200 {
        let c = (a * fb - b * fa) / (fb - fa);
        let fc = r(c)?;
        if fc == 0.0 { return Ok(c); }
        if fc * fb < 0.0 { a = b; fa = fb; } else { fa *= 0.5; }
        b = c;
        fb = fc;
        if (b - a).abs() <= 1e-14 * a.abs().max(b.abs()) { break; }
    }
    Ok(b)
}

/// What a lit equilibrium afterburner can be asked: `Ok(f7)`, or the refusal in words (too cold — below the frozen
/// mixture's own recombination heat — or too hot, the ceiling named at this pressure).
pub fn fuel_equilibrium(
    gas: &Gas, comp4: &[(&'static str, f64)], tt5: f64, f5: f64, tt7: f64, p7: f64, eta_ab: f64,
) -> Result<f64, Abort> {
    fuel_equilibrium_named(gas, comp4, tt5, f5, tt7, p7, eta_ab, true)
}

/// [`fuel_equilibrium`], its refusals named in words (`name`) or as a bare [`REFUSED`] marker — the temperature a
/// refusal names costs a bisection of its own, which a search that only needs "refused here" must not pay (Part B).
#[allow(clippy::too_many_arguments)]
fn fuel_equilibrium_named(
    gas: &Gas, comp4: &[(&'static str, f64)], tt5: f64, f5: f64, tt7: f64, p7: f64, eta_ab: f64, name: bool,
) -> Result<f64, Abort> {
    let cap = f_cap(gas);
    let (r_lo, _) = eq_residual(gas, comp4, tt5, f5, f5, tt7, p7, eta_ab)?;
    if r_lo > 0.0 {
        if !name {
            return Err(refused("too cold"));
        }
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
    let (mut r_lo, mut r_hi) = (r_lo, f64::NAN);
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
        if res < 0.0 { (lo, r_lo) = (f, res) } else { (hi, r_hi) = (f, res); found_hi = true; break; }
    }
    // Bracketed (both ends evaluated, the lean one below the cap): the Illinois false position to the same width —
    // about eight equilibrium solves where bisection took about forty, three times over in the `pt7` iteration (Part B:
    // a lit Fly-it point on this gas cost 5.7× a dry one against the anchor's 3×).
    if found_hi && res != 0.0 {
        let (mut a, mut fa, mut b, mut fb) = (lo, r_lo, hi, r_hi);
        for _ in 0..FP_MAX {
            let c = (a * fb - b * fa) / (fb - fa);
            (res, prod) = eq_residual(gas, comp4, tt5, f5, c, tt7, p7, eta_ab)?;
            f = c;
            if res == 0.0 { break; }
            if res * fb < 0.0 { (a, fa) = (b, fb) } else { fa *= 0.5 }
            (b, fb) = (c, res);
            if (b - a).abs() <= FP_TOL * f { break; }
        }
    }
    if !found_hi && eq_residual_at_cap(gas, comp4, tt5, f5, tt7, p7, eta_ab)? < 0.0 {
        if !name {
            return Err(refused("too hot"));
        }
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
    let t = root(0.5 * tt, tt, |t| Ok(resid(t))).expect("infallible residual");
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
    let t_star = root(0.5 * tt7, tt7,
                          |t| Ok((h7t - gas_out.h_t(t, f7)) - 0.5 * gas_out.gamma_t_at(t, f7) * r7 * t))?;
    let u_star = h7t - gas_out.h_t(t_star, f7);
    if momentum(u_star)? > 0.0 {
        return Ok(None);                    // thermal choke
    }
    // The subsonic root, between stagnation and the sonic point, solved for `V7` on the momentum residual times `V7`
    // (`g7·R7·T7 + g7·V7² − impulse·V7`: finite at stagnation, where the residual itself runs to +∞, and convex, so
    // one root on `[0, V*]`); `u = V7²/2` is still what the static temperature is read from, and the stopping rule is
    // relative to `V7` itself, so a slow duct is resolved as finely as a fast one.
    let v7 = root(0.0, powp(2.0 * u_star, 0.5), |v: f64| Ok(g7 * r7 * temp(0.5 * (v * v))? + g7 * (v * v) - impulse * v))?;
    let u7 = 0.5 * (v7 * v7);
    let t7 = temp(u7)?;
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
    afterburn(gas, s4, s5, mdot_air, ab, mode, true)
}

/// The prefix of a refusal left UNNAMED inside a search (Part B): which wall, without the temperature the words name.
pub const REFUSED: &str = "afterburner refused";

fn refused(what: &str) -> Abort { Abort(format!("{REFUSED}: {what}")) }

/// Is this the afterburner (or its duct) saying no — named or not — rather than the core failing?
pub fn is_refusal(e: &Abort) -> bool {
    e.0.starts_with(REFUSED) || e.0.starts_with("The afterburner cannot") || e.0.contains("the afterburner duct")
}

/// [`try_afterburn`], with its refusals named (`name`) or bare ([`REFUSED`]).
fn afterburn(
    gas: &Gas, s4: &FlowState, s5: &FlowState, mdot_air: f64, ab: &Afterburner, mode: AbMode, name: bool,
) -> Result<(FlowState, AbDetail), Abort> {
    let (tt5, f5) = (s5.tt, s5.far);
    if mode == AbMode::Unlit {
        let s7 = FlowState { tt: tt5, pt: ab.pi_fh * s5.pt, mdot: s5.mdot, far: f5 };
        let detail = AbDetail { mode, f5, f7: f5, pi_ab: ab.pi_fh, pi_r: 1.0, m6: ab.m6, m7: ab.m6, passes: 1 };
        return Ok((s7, detail));
    }
    let tt7 = ab.tt7;
    if !(tt7 > tt5) {
        return Err(if name { too_cold(tt7, tt5) } else { refused("too cold") });
    }
    let (f7, pi_r, m7, passes) = if gas.is_equilibrium() {
        let comp4 = gas.try_freeze_equilibrium(s4.far, s4.tt, s4.pt)?;
        let mut p = ab.pi_fh * s5.pt;
        let mut out = None;
        for pass in 1..=AB_PASSES_MAX {
            let f7 = fuel_equilibrium_named(gas, &comp4, tt5, f5, tt7, p, ab.eta_ab, name)?;
            // The station-7 properties at this trial pressure, on a scratch copy (a clone starts with empty memos),
            // so the real gas freezes once, at the converged condition.
            let scratch = gas.clone();
            scratch.try_freeze_equilibrium(s4.far, s4.tt, s4.pt)?;
            scratch.try_freeze_equilibrium(f7, tt7, p)?;
            let Some((pi_r, m7)) = rayleigh(&scratch, f5, &scratch, f7, tt5, ab.m6, tt7, (1.0 + f7) / (1.0 + f5))?
            else {
                if !name { return Err(refused("the duct chokes")); }
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
            if !name { return Err(refused("too hot")); }
            return Err(too_hot(tt7, ceiling_frozen(gas, tt5, f5, ab.eta_ab)?));
        };
        let Some((pi_r, m7)) = rayleigh(gas, f5, gas, f7, tt5, ab.m6, tt7, (1.0 + f7) / (1.0 + f5))? else {
            if !name { return Err(refused("the duct chokes")); }
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

// =============================================================================================
// PART B — the afterburner on the engine you FLY (rung 34's one-shaft plant, `crate::spool`)
// =============================================================================================
//
// The core is rung 34's, unchanged: the compressor closed on its map at `(ν, Tt4)`, the turbine's guide vanes choked,
// the shaft's power imbalance driving `dν/ds`. What the afterburner moves is the NOZZLE relation — the turbine expands
// only as far as the throat lets the gas out, and the throat now sees station 7, not station 5:
//
//     choked    (★AB)   π_t·π_AB·√(Tt4/Tt7) = A4·MFP4·(1+f7) / ((1+f5)·A8·π_n·MFP7)
//     unchoked          ṁ_air·(1+f7) = A8·ρ9·V9                     (p9 = p0)
//
// Unlit, `Tt7 = Tt5` and `π_AB = π_fh`: (★AB) is rung 31's (★) with the throat `A8·π_fh` — so the fitted engine's DRY
// throat is the core's opened by `1/π_fh`, and the unlit engine sits where the engine without an afterburner sits
// wherever the nozzle is choked. The burning zone's area `A6` is hardware, captured at the design at the `M6` knob; off
// design `M6` floats with the turbine-exit corrected flow. The power balance is untouched (the afterburner is behind
// the turbine), and thrust books the TOTAL fuel.

/// The throat: held at `s` × the dry throat (`s` = 1 is *held*), or *matched* — the area that keeps the core where
/// the dry engine sits at this throttle and flight (Mattingly § 8.4's control law).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Throat {
    Held(f64),
    Match,
}

/// The afterburner fitted to rung 34's plant.
pub struct FlyAfterburner<'a> {
    pub st: &'a SpoolTransient,
    pub ab: Afterburner,
    /// The burning zone's flow area, m², captured at the design at `ab.m6` (∞ when `m6` is 0: a stagnant duct).
    pub a6: f64,
    /// The dry throat, m²: the core's opened by `1/π_fh`.
    pub a8_dry: f64,
}

/// One operating point with the afterburner fitted.
#[derive(Clone, Copy, Debug)]
pub struct FlyPoint {
    /// Rung 34's instant — its nozzle fields (`branch`, thrust, `M9`, `pt9/p0`) read at the station-7 nozzle, its
    /// thrust on the total fuel; `f` stays the main burner's.
    pub eq: Instant,
    pub detail: AbDetail,
    pub s5: FlowState,
    pub s7: FlowState,
    /// The throat used, m², and over the dry throat.
    pub a8: f64,
    pub s: f64,
}

/// A scratch copy on the equilibrium gas, where each trial freezes its own `f7` (the per-`far` guard forbids one
/// `f7` frozen at two pressures, which neighbouring trials of a converging search can produce); the gas itself on
/// every other gas, which memoises nothing per trial.
fn scratch<'g>(g: &'g Gas, s4: &FlowState) -> Result<Cow<'g, Gas>, Abort> {
    if !g.is_equilibrium() {
        return Ok(Cow::Borrowed(g));
    }
    // A clone starts with empty memos (as in [`try_afterburn`]): the station-4 mixture is frozen into it again.
    let c = g.clone();
    c.try_freeze_equilibrium(s4.far, s4.tt, s4.pt)?;
    Ok(Cow::Owned(c))
}

/// The flow per unit area `ṁ·√Tt/(A·pt)` at static temperature `t` under the total `tt`, hot section at `far`.
fn mfp_at_t(gas: &Gas, far: f64, tt: f64, t: f64) -> f64 {
    let v = powp(2.0 * (gas.h_t(tt, far) - gas.h_t(t, far)), 0.5);
    gas.pr_t(t, far) / gas.pr_t(tt, far) / (gas.r_t_at(far) * t) * v * powp(tt, 0.5)
}

/// The sonic static temperature under the total `tt`.
fn t_sonic(gas: &Gas, far: f64, tt: f64) -> Result<f64, Abort> {
    let r = gas.r_t_at(far);
    root(0.5 * tt, tt, |t| Ok((gas.h_t(tt, far) - gas.h_t(t, far)) - 0.5 * gas.gamma_t_at(t, far) * r * t))
}

/// The duct area passing `mdot` at `(tt, pt, far)` at Mach `m` — the design capture of `A6`.
fn area_at_mach(gas: &Gas, far: f64, tt: f64, pt: f64, mdot: f64, m: f64) -> f64 {
    let (t, p, v) = static_at_mach(gas, far, tt, 1.0, m);
    mdot * powp(tt, 0.5) / (pt * (p / (gas.r_t_at(far) * t) * v * powp(tt, 0.5)))
}

/// The burning zone's Mach number when the duct `a6` passes `mdot` at `(tt, pt, far)`: the subsonic root, by
/// bisection on the static temperature (one enthalpy and one pressure ratio a step). A flow even Mach 1 cannot pass is
/// a refusal.
#[allow(clippy::too_many_arguments)]
fn m6_in_duct(gas: &Gas, far: f64, tt: f64, pt: f64, mdot: f64, a6: f64, m6_design: f64, name: bool)
    -> Result<f64, Abort>
{
    if a6.is_infinite() {
        return Ok(0.0);
    }
    let target = mdot * powp(tt, 0.5) / (pt * a6);
    let ts = t_sonic(gas, far, tt)?;
    if !(target < mfp_at_t(gas, far, tt, ts)) {
        return Err(if name {
            Abort(format!("The gas leaves the turbine faster than the afterburner duct can pass it: it would choke at the \
                           duct's entry (the duct is sized for Mach {m6_design:.2} at the design point)."))
        } else {
            refused("the duct entry chokes")
        });
    }
    let t = root(ts, tt, |t| Ok(mfp_at_t(gas, far, tt, t) - target))?;
    let v = powp(2.0 * (gas.h_t(tt, far) - gas.h_t(t, far)), 0.5);
    Ok(v / powp(gas.gamma_t_at(t, far) * gas.r_t_at(far) * t, 0.5))
}

/// The bracket for a residual RISING through its root on `[lo, hi]`: the top end marched down by `step` past trials
/// that fail (a too-cold afterburner near the turbine's top), the bottom end found by BISECTION toward the top past
/// trials that fail — a held lit throat's low `π_t` sends the turbine-exit gas into the duct past Mach 1 over a wide
/// band, and marching it up in steps cost ~20 refused trials an instant (the census, 2026-10-19). An evaluable trial
/// above the root tightens the top. `(lo, f(lo), hi, f(hi))`, or the last failure when nothing evaluable is left.
fn bracket_in(f: &impl Fn(f64) -> Result<f64, Abort>, lo: f64, hi: f64, step: f64)
    -> Result<(f64, f64, f64, f64), Abort>
{
    let mut b = hi;
    let mut fb = loop {
        match f(b) {
            Ok(v) => break v,
            Err(e) if b - step <= lo => return Err(e),
            Err(_) => b -= step,
        }
    };
    let mut bad = match f(lo) {
        Ok(v) => return Ok((lo, v, b, fb)),
        Err(e) => (lo, e),
    };
    while b - bad.0 > 1e-6 {
        let x = 0.5 * (bad.0 + b);
        match f(x) {
            Ok(v) if v < 0.0 => return Ok((x, v, b, fb)),
            Ok(v) => (b, fb) = (x, v),
            Err(e) => bad = (x, e),
        }
    }
    Err(bad.1)
}

/// [`bracket_in`] for a residual that may have no root at all because it stays NEGATIVE to the top: the top is found
/// first (marched down past failing trials) and, if not above zero, returned at once as `(top, f, top, f)` — the caller
/// reads the missing sign change — without the bottom end's search. A failure carries the trial that failed.
fn bracket_top_first(f: &impl Fn(f64) -> Result<f64, Abort>, lo: f64, hi: f64, step: f64)
    -> Result<(f64, f64, f64, f64), (f64, Abort)>
{
    let mut b = hi;
    let fb = loop {
        match f(b) {
            Ok(v) => break v,
            Err(e) if b - step <= lo => return Err((b, e)),
            Err(_) => b -= step,
        }
    };
    if !(fb > 0.0) {
        return Ok((b, fb, b, fb));
    }
    let mut bad = match f(lo) {
        Ok(v) => return Ok((lo, v, b, fb)),
        Err(e) => (lo, e),
    };
    let mut fb = fb;
    while b - bad.0 > 1e-6 {
        let x = 0.5 * (bad.0 + b);
        match f(x) {
            Ok(v) if v < 0.0 => return Ok((x, v, b, fb)),
            Ok(v) => (b, fb) = (x, v),
            Err(e) => bad = (x, e),
        }
    }
    Err(bad)
}

/// [`SpoolTransient::find_equilibrium_nu`], COPIED fallible (same march, same sentinel, same Illinois): the original
/// panics when the speed search does not bracket; here that is an `Err` carrying the SAME message, so the page's
/// plain words (`sandbox::explain_fly`) read it unchanged, and the afterburner's own refusal can be reported instead.
pub fn try_find_equilibrium_nu<F>(resid: F) -> Result<f64, Abort>
where
    F: FnMut(f64) -> Result<f64, Abort>,
{
    try_find_equilibrium_nu_from(resid, false)
}

/// [`try_find_equilibrium_nu`], and with `walk_up` (the second pass below rung 34's low-speed wall only) a low end
/// below the unstable equilibrium walked up, by the march's own 0.02, to the first speed whose shaft accelerates —
/// as `SpoolTransient`'s second pass does.
fn try_find_equilibrium_nu_from<F>(mut resid: F, walk_up: bool) -> Result<f64, Abort>
where
    F: FnMut(f64) -> Result<f64, Abort>,
{
    let (mut lo, mut flo) = (None, None);
    let mut nu = 0.30;
    while nu < 1.6 {
        if let Ok(v) = resid(nu) {
            flo = Some(v);
            lo = Some(nu);
            break;
        }
        nu += 0.02;
    }
    let (mut hi, mut fhi) = (None, None);
    nu = 1.60;
    while let Some(l) = lo {
        if nu <= l {
            break;
        }
        if let Ok(v) = resid(nu) {
            fhi = Some(v);
            hi = Some(nu);
            break;
        }
        nu -= 0.02;
    }
    if let (true, Some(a), Some(b), Some(fa), Some(fb)) = (walk_up, lo, hi, flo, fhi) {
        if fa <= 0.0 && fb < 0.0 {
            let mut nu = a + 0.02;
            while nu < b {
                match resid(nu) {
                    Ok(v) if v > 0.0 => {
                        (lo, flo) = (Some(nu), Some(v));
                        break;
                    }
                    Ok(_) => nu += 0.02,
                    Err(_) => break,
                }
            }
        }
    }
    let (lo, hi) = match (lo, hi) {
        (Some(a), Some(b)) if flo.unwrap() > 0.0 && 0.0 > fhi.unwrap() => (a, b),
        _ => return Err(Abort(format!(
            "rung-34 equilibrium does not bracket (Phi[{lo:?}]={flo:?}, Phi[{hi:?}]={fhi:?})"))),
    };
    let mut resid_safe = |nu: f64| -> f64 { resid(nu).unwrap_or(1e9) };
    Ok(illinois(&mut resid_safe, lo, hi, flo.unwrap(), fhi.unwrap(), SpoolTransient::N_TOL, ILLINOIS_MAXIT))
}

/// How far a balanced point's `dν/ds` may sit from zero: the speed root is converged to 1e−12 in `ν`, so a true
/// balance reads ~1e−11; a search that settled on the EDGE of a refusing region reads its finite imbalance.
pub const PHI_BALANCED: f64 = 1e-6;

/// A bracket on `Φ(ν)` (falling) from a nearby speed `h`: `Φ(h)`, a first step of 0.1 toward the sign change, then
/// steps sized by the secant to 1.2× the distance it predicts (0.02–0.3), inside the shipped search's range
/// [0.30, 1.60]. `None` when a step fails or the range runs out (the caller searches fully).
fn bracket_speed(resid: &impl Fn(f64) -> Result<f64, Abort>, h: f64) -> Option<(f64, f64, f64, f64)> {
    let (mut x, mut fx) = (h, resid(h).ok()?);
    let up = fx > 0.0;
    let mut step = 0.1;
    for _ in 0..12 {
        let y = if up { x + step } else { x - step };
        if !(0.30..=1.60).contains(&y) {
            return None;
        }
        let fy = resid(y).ok()?;
        if (fy > 0.0) != up {
            return Some(if up { (x, fx, y, fy) } else { (y, fy, x, fx) });
        }
        let slope = (fy - fx) / (y - x);
        step = if slope < 0.0 { (1.2 * (fy / slope).abs()).clamp(0.02, 0.3) } else { 0.1 };
        (x, fx) = (y, fy);
    }
    None
}

/// Where a search with no nearby point starts: the design shaft speed.
pub const NU_DESIGN: f64 = 1.0;

thread_local! {
    static TRIALS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static INSTANTS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static C_LP: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static C_BURN: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static C_WALL: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Part B's work census — station-5→7 trials and instants run on this thread since the last [`take_census`]; the
/// cost trigger (anchor § 4) is read against it, not guessed.
/// Part C's work census: lit LP-turbine solves, burns (station 5 → 7), and lean-wall answers on this thread since the
/// last call — the cost trigger (addendum § 5.4) is read against it.
pub fn take_census_c() -> (u64, u64, u64) {
    (C_LP.with(|c| c.replace(0)), C_BURN.with(|c| c.replace(0)), C_WALL.with(|c| c.replace(0)))
}

pub fn take_census() -> (u64, u64) {
    (TRIALS.with(|c| c.replace(0)), INSTANTS.with(|c| c.replace(0)))
}

impl<'a> FlyAfterburner<'a> {
    /// Fit the afterburner to the captured hardware: `A6` from the DRY design run's station 5 at the `M6` knob.
    pub fn capture(st: &'a SpoolTransient, ab: Afterburner) -> Self {
        let m = &st.inner.inner;
        let s5 = m.reference.station("5");
        let a6 = if ab.m6 == 0.0 { f64::INFINITY } else { area_at_mach(m.gas(), s5.far, s5.tt, s5.pt, s5.mdot, ab.m6) };
        FlyAfterburner { st, ab, a6, a8_dry: m.a8 / ab.pi_fh }
    }

    /// Station 5 → 7 at a trial `π_t`, on `g` (a scratch copy on the equilibrium gas): the turbine (rung 34's inverse
    /// read), the duct's Mach off its area, the afterburner. `(τ_t, s5, s7, detail)`.
    #[allow(clippy::too_many_arguments)]
    fn trial(&self, g: &Gas, s4: &FlowState, mdot_air: f64, eta_t: f64, pi_t: f64, mode: AbMode, name: bool)
        -> Result<(f64, FlowState, FlowState, AbDetail), Abort>
    {
        TRIALS.with(|c| c.set(c.get() + 1));
        let m = &self.st.inner.inner;
        let (tau_t, tt5) = m.tau_t_of_pi_t(g, s4.tt, s4.far, pi_t, Some(eta_t));
        let s5 = FlowState { tt: tt5, pt: pi_t * s4.pt, mdot: mdot_air * (1.0 + s4.far), far: s4.far };
        // Unlit, the duct heats nothing and its Mach moves no pressure: computed once at the point ([`Self::try_instant`]).
        let m6 = if mode == AbMode::Lit { m6_in_duct(g, s5.far, s5.tt, s5.pt, s5.mdot, self.a6, self.ab.m6, name)? } else { self.ab.m6 };
        let (s7, d) = afterburn(g, s4, &s5, mdot_air, &Afterburner { m6, ..self.ab }, mode, name)?;
        Ok((tau_t, s5, s7, d))
    }

    /// The choked branch: `π_t` from (★AB) — bracketed first within 0.02 of `hint` (a neighbouring instant's root in
    /// the same speed search; about half the trials of the full bracket), else on the turbine's whole range.
    #[allow(clippy::too_many_arguments)]
    fn solve_choked(&self, g: &Gas, s4: &FlowState, mdot_air: f64, eta_t: f64, a8: f64, mode: AbMode,
                    hint: Option<f64>) -> Result<f64, Abort>
    {
        let m = &self.st.inner.inner;
        let rhs = m.a4 * try_choked_mfp(g, s4.tt, s4.far)? / ((1.0 + s4.far) * a8 * m.pi_n);
        let resid = |pi_t: f64| -> Result<f64, Abort> {
            let gs = scratch(g, s4)?;
            let (_, _, s7, d) = self.trial(&gs, s4, mdot_air, eta_t, pi_t, mode, false)?;
            Ok(pi_t * d.pi_ab * powp(s4.tt / s7.tt, 0.5) - rhs * (1.0 + s7.far) / try_choked_mfp(&gs, s7.tt, s7.far)?)
        };
        if let Some(h) = hint {
            let (a, b) = ((h - 0.02).max(0.02), (h + 0.02).min(0.999));
            if let (Ok(fa), Ok(fb)) = (resid(a), resid(b)) {
                if fa < 0.0 && 0.0 < fb {
                    return try_illinois(resid, a, b, fa, fb, SpoolTransient::HOT_TOL, ILLINOIS_MAXIT);
                }
            }
        }
        let (lo, flo, hi, fhi) = bracket_in(&resid, 0.02, 0.999, 0.01)?;
        if !(flo < 0.0 && 0.0 < fhi) {
            return Err(Abort(format!("rung-102 turbine choke-match bracket does not straddle the root \
                                      (pi_t {lo:.2}: {flo:.3e}, {hi:.3}: {fhi:.3e})")));
        }
        try_illinois(resid, lo, hi, flo, fhi, SpoolTransient::HOT_TOL, ILLINOIS_MAXIT)
    }

    /// The nozzle exit at a trial `π_t`, the throat's mass flux `ρ9·V9`, and the mass it must pass, `ṁ_air·(1+f7)`.
    #[allow(clippy::too_many_arguments)]
    fn exit_at(&self, g: &Gas, s4: &FlowState, mdot_air: f64, eta_t: f64, pi_t: f64, mode: AbMode, p0: f64)
        -> Result<(NozzleExit, f64, f64), Abort>
    {
        let (_, _, s7, _) = self.trial(g, s4, mdot_air, eta_t, pi_t, mode, false)?;
        let exit = Nozzle::convergent(p0, self.st.inner.inner.pi_n).try_apply(&s7, g)?;
        let flux = exit.p9 / (g.r_t_at(s7.far) * exit.t9) * exit.v9;
        Ok((exit, flux, mdot_air * (1.0 + s7.far)))
    }

    /// The unchoked branch: `π_t` so the subsonic nozzle (`p9 = p0`) passes the core's flow plus the afterburner's
    /// fuel. `ṁ7 − A8·ρ9·V9` is continuous across the choke boundary (the convergent nozzle's choked flux is its subsonic
    /// flux's limit) and falls as `π_t` rises, and the choked root `pi_t_c` sits on its flow-short side (there the
    /// nozzle read subsonic: it passes less than the sonic formula said). So the root is bracketed by `pi_t_c` and the
    /// top of the turbine's range — NOT `SpoolTransient::try_turbine_subsonic`'s march down from just under the choke
    /// boundary in 0.01 steps, two nozzle solves a step, which made an unchoked lit instant cost 30–40 afterburner trials
    /// (the census, 2026-10-19). The root is unique either way; a trial the afterburner refuses at the top is stepped past.
    #[allow(clippy::too_many_arguments)]
    fn solve_subsonic(&self, g: &Gas, s4: &FlowState, mdot_air: f64, eta_t: f64, a8: f64, mode: AbMode, p0: f64,
                      pi_t_c: f64) -> Result<f64, Abort>
    {
        let resid = |pi_t: f64| -> Result<f64, Abort> {
            let gs = scratch(g, s4)?;
            let (_, flux, mdot7) = self.exit_at(&gs, s4, mdot_air, eta_t, pi_t, mode, p0)?;
            Ok(mdot7 - a8 * flux)
        };
        // Below rung 34's low-speed wall (the instant's second pass) the gas at the choke root cannot reach `p0` at all:
        // step up to the first `π_t` that can — there the nozzle barely flows, the flow-short side. Wherever the choke
        // root's nozzle solved, the first trial runs and the loop never steps.
        let mut lo = pi_t_c;
        let rlo = loop {
            match resid(lo) {
                Ok(r) => break r,
                Err(e) if !e.0.starts_with("nozzle back-pressure") || lo + 0.01 >= 0.9995 => return Err(e),
                Err(_) => lo += 0.01,
            }
        };
        let mut hi = 0.9995;
        let rhi = loop {
            match resid(hi) {
                Ok(r) => break r,
                Err(e) if hi - 0.01 <= lo => return Err(e),
                Err(_) => hi -= 0.01,
            }
        };
        if !(rlo > 0.0 && rhi < 0.0) {
            return Err(Abort(format!("rung-102 subsonic turbine does not bracket at tt4={:.0}", s4.tt)));
        }
        try_illinois(resid, lo, hi, rlo, rhi, SpoolTransient::HOT_TOL, ILLINOIS_MAXIT)
    }

    /// One quasi-steady instant at shaft speed `nu` and throttle `tt4`, the throat `a8` (m²): rung 34's instant with
    /// (★AB) in place of (★) — the compressor closed exactly as `SpoolTransient::try_instant` closes it.
    pub fn try_instant(
        &self, flight: &FlightCondition, nu: f64, tt4: f64, cmap: &ComponentMap, mode: AbMode, a8: f64,
    ) -> Result<FlyPoint, Abort> {
        self.try_instant_hinted(flight, nu, tt4, cmap, mode, a8, None, false)
    }

    /// [`Self::try_instant`] with rung 34's low-speed wall lifted — the instant the equilibrium's second pass
    /// searches on.
    pub fn try_instant_below_choke(
        &self, flight: &FlightCondition, nu: f64, tt4: f64, cmap: &ComponentMap, mode: AbMode, a8: f64,
    ) -> Result<FlyPoint, Abort> {
        self.try_instant_hinted(flight, nu, tt4, cmap, mode, a8, None, true)
    }

    /// [`Self::try_instant`] with the turbine's `π_t` bracketed first near `pi_t_hint` ([`Self::solve_choked`]), and with
    /// `below_choke` rung 34's low-speed wall lifted (`SpoolTransient::try_instant_tail_at`): where the station-7 gas at
    /// the choke `π_t` cannot reach `p0`, the turbine is solved for the subsonic nozzle directly.
    #[allow(clippy::too_many_arguments)]
    fn try_instant_hinted(
        &self, flight: &FlightCondition, nu: f64, tt4: f64, cmap: &ComponentMap, mode: AbMode, a8: f64,
        pi_t_hint: Option<f64>, below_choke: bool,
    ) -> Result<FlyPoint, Abort> {
        let st = self.st;
        let mm = &st.inner.inner;
        let pi_d = mm.pi_d_max * ram_recovery(flight.m0);
        let (state0, v0) = mm.freestream_for(flight);
        let (tt2, pt2) = (state0.tt, pi_d * state0.pt);
        let n = nu * powp(st.inner.tt2_d / tt2, 0.5);
        INSTANTS.with(|c| c.set(c.get() + 1));
        let comp = st.try_close_compressor(tt4, tt2, pt2, cmap, n)?;
        let (f, pt4, mdot_air) = (comp.f, comp.pt4, comp.mdot_air);
        let wgas = comp.gas(mm);
        let s4 = FlowState { tt: tt4, pt: pt4, mdot: comp.mdot4, far: f };
        let nu_t = nu * powp(st.inner.tt4_d / tt4, 0.5);
        let eta_t = cmap.eta_t_at(mm.eta_t, nu_t);

        // Assume choked; solve (★AB), rebuild the nozzle at station 7, and DISPATCH as rung 33 does.
        let p0 = flight.p0;
        let mut pi_t = self.solve_choked(wgas, &s4, mdot_air, eta_t, a8, mode, pi_t_hint)?;
        let mut g = scratch(wgas, &s4)?;
        let (mut tau_t, mut s5, mut s7, mut d) = self.trial(&g, &s4, mdot_air, eta_t, pi_t, mode, false)?;
        let (mut exit, star_refused) = match Nozzle::convergent(p0, mm.pi_n).try_apply(&s7, &g) {
            Ok(e) => (e, false),
            Err(e) if below_choke && e.0.starts_with("nozzle back-pressure") => {
                pi_t = self.solve_subsonic(wgas, &s4, mdot_air, eta_t, a8, mode, p0, pi_t)?;
                g = scratch(wgas, &s4)?;
                (tau_t, s5, s7, d) = self.trial(&g, &s4, mdot_air, eta_t, pi_t, mode, false)?;
                (Nozzle::convergent(p0, mm.pi_n).try_apply(&s7, &g)?, true)
            }
            Err(e) => return Err(e),
        };
        let branch = if !star_refused && exit.p9 > p0 + 1e-6 { Branch::Choked } else { Branch::Subsonic };
        if branch == Branch::Subsonic && !star_refused {
            match self.solve_subsonic(wgas, &s4, mdot_air, eta_t, a8, mode, p0, pi_t) {
                Ok(p) => {
                    pi_t = p;
                    g = scratch(wgas, &s4)?;
                    (tau_t, s5, s7, d) = self.trial(&g, &s4, mdot_air, eta_t, pi_t, mode, false)?;
                    exit = Nozzle::convergent(p0, mm.pi_n).try_apply(&s7, &g)?;
                }
                // The M9 → 1 boundary layer, where the subsonic root meets the choked one (rung 34's fallback).
                Err(_) if exit.m9 > 0.985 => {}
                Err(_) => return Err(Abort(format!(
                    "rung-102 subsonic turbine failed to bracket AWAY from the M9->1 boundary (choked-star M9={:.4}) at \
                     tt4={tt4:.0}, nu={nu:.3}", exit.m9))),
            }
        }

        if mode == AbMode::Unlit {
            let m6 = m6_in_duct(&g, s5.far, s5.tt, s5.pt, s5.mdot, self.a6, self.ab.m6, true)?;
            (d.m6, d.m7) = (m6, m6);
        }

        // The power balance (the afterburner is behind the turbine: unchanged), thrust on the TOTAL fuel.
        let tt5 = s5.tt;
        let pt_spec = mm.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt5, f));
        let pc_spec = wgas.h_c(comp.tt3) - wgas.h_c(tt2);
        let p_net_spec = pt_spec - pc_spec;
        let phi_dot = (mdot_air * p_net_spec) / (st.p_ref * nu);
        let f7 = s7.far;
        let press_thrust = (1.0 + f7) * g.r_t_at(f7) * exit.t9 * (1.0 - p0 / exit.p9) / exit.v9;
        let sp_thrust = (1.0 + f7) * exit.v9 - v0 + press_thrust;
        let eq = Instant {
            nu, tt4, branch, pi_c: comp.pi_c, tau_c: comp.tau_c, eta_c: comp.eta_c, eta_t, m: comp.m, n,
            flowcoef: comp.phi, mdot_air, f, pi_t, tau_t, tt3: comp.tt3, tt5, nu_t, p_net_spec, phi: phi_dot,
            sp_thrust, thrust: mdot_air * sp_thrust, m9: exit.m9, pt9_over_p0: mm.pi_n * s7.pt / p0, tt2, pt2, v0,
        };
        Ok(FlyPoint { eq, detail: d, s5, s7, a8, s: a8 / self.a8_dry })
    }

    /// The balanced point at throttle `tt4` with the throat `a8`: the speed where the shaft's power balances, re-run
    /// at the root and REQUIRED to balance ([`PHI_BALANCED`]) — the search's sentinel treats a refusing trial as "speeds
    /// up", which is right below idle and wrong at a wall the afterburner draws elsewhere. A refusal is named at the
    /// end ([`Self::name_refusal`]). The search starts at the design shaft speed ([`NU_DESIGN`]); the shipped search's
    /// march from both ends of its range (19–37 instants a dry point, the census) is the fallback.
    pub fn try_equilibrium(
        &self, flight: &FlightCondition, tt4: f64, cmap: &ComponentMap, mode: AbMode, a8: f64,
    ) -> Result<FlyPoint, Abort> {
        self.try_equilibrium_from(flight, tt4, cmap, mode, a8, Some(NU_DESIGN))
    }

    /// [`Self::try_equilibrium`] from a known nearby speed `nu_hint` (the dry point's, for a lit one): the root is
    /// bracketed by stepping 0.1 from it toward the sign change and found by the same Illinois with the same sentinel;
    /// where that bracket cannot be made, the full search ([`try_find_equilibrium_nu`]). Within one search, each
    /// instant's turbine is bracketed first near the last instant's root (state local to this call, so the answer
    /// does not depend on what was solved before it). The root is unique (`Φ` falls with speed), so either way lands
    /// on it to the search's tolerance.
    pub fn try_equilibrium_from(
        &self, flight: &FlightCondition, tt4: f64, cmap: &ComponentMap, mode: AbMode, a8: f64, nu_hint: Option<f64>,
    ) -> Result<FlyPoint, Abort> {
        let refusal: RefCell<Option<Abort>> = RefCell::new(None);
        let pi_t_last: std::cell::Cell<Option<f64>> = std::cell::Cell::new(None);
        let resid = |nu: f64| -> Result<f64, Abort> {
            match self.try_instant_hinted(flight, nu, tt4, cmap, mode, a8, pi_t_last.get(), false) {
                Ok(p) => { pi_t_last.set(Some(p.eq.pi_t)); Ok(p.eq.phi) }
                Err(e) => { if is_refusal(&e) { *refusal.borrow_mut() = Some(e.clone()); } Err(e) }
            }
        };
        let near = nu_hint.and_then(|h| bracket_speed(&resid, h));
        let found = match near {
            Some((lo, flo, hi, fhi)) => {
                let mut resid_safe = |nu: f64| -> f64 { resid(nu).unwrap_or(1e9) };
                Ok(illinois(&mut resid_safe, lo, hi, flo, fhi, SpoolTransient::N_TOL, ILLINOIS_MAXIT))
            }
            None => try_find_equilibrium_nu(resid),
        };
        // Rung 34's low-speed wall (`SpoolTransient::equilibrium`): only where the search above refused, and for no
        // refusal of the afterburner's own, a second pass searches on the instant with the wall lifted, walking up past
        // the unstable equilibrium; where it fails too, the first pass's words.
        let (nu, below_choke) = match found {
            Ok(nu) => (nu, false),
            Err(e) => {
                if let Some(r) = refusal.take() {
                    return Err(self.name_refusal(flight, tt4, cmap, &r));
                }
                pi_t_last.set(None);
                let low = |nu: f64| -> Result<f64, Abort> {
                    let p = self.try_instant_hinted(flight, nu, tt4, cmap, mode, a8, pi_t_last.get(), true)?;
                    pi_t_last.set(Some(p.eq.pi_t));
                    Ok(p.eq.phi)
                };
                match try_find_equilibrium_nu_from(low, true) {
                    Ok(nu) => (nu, true),
                    Err(_) => return Err(e),
                }
            }
        };
        match self.try_instant_hinted(flight, nu, tt4, cmap, mode, a8, pi_t_last.get(), below_choke) {
            Ok(p) if p.eq.phi.abs() < PHI_BALANCED => Ok(p),
            Ok(p) => {
                let why = refusal.take().map(|r| self.name_refusal(flight, tt4, cmap, &r).0).unwrap_or_default();
                Err(Abort(format!("rung-102: the speed search settled where the shaft does not balance (dnu/ds \
                                   {:.3e} at nu {nu:.4}). {why}", p.eq.phi)))
            }
            Err(e) if is_refusal(&e) => Err(self.name_refusal(flight, tt4, cmap, &e)),
            Err(e) => Err(e),
        }
    }

    /// An unnamed refusal ([`REFUSED`]) in words, named at THIS throttle and flight's DRY point (the unlit engine,
    /// throat held — its turbine exit is what the afterburner is asked to heat). Where the dry point itself does not
    /// run, or runs the afterburner without refusing (a wall the lit engine moved onto), the words name the wall alone.
    pub fn name_refusal(&self, flight: &FlightCondition, tt4: f64, cmap: &ComponentMap, e: &Abort) -> Abort {
        if !e.0.starts_with(REFUSED) {
            return e.clone();
        }
        if let Ok(dry) = self.try_equilibrium(flight, tt4, cmap, AbMode::Unlit, self.a8_dry) {
            let mm = &self.st.inner.inner;
            let pt4 = dry.s5.pt / dry.eq.pi_t;
            let wgas = mm.try_working_gas(dry.eq.f, tt4, pt4).ok().flatten();
            let g = wgas.as_ref().unwrap_or(mm.gas());
            let s4 = FlowState { tt: tt4, pt: pt4, mdot: 0.0, far: dry.eq.f };
            let named = m6_in_duct(g, dry.s5.far, dry.s5.tt, dry.s5.pt, dry.s5.mdot, self.a6, self.ab.m6, true)
                .and_then(|m6| afterburn(&*scratch(g, &s4)?, &s4, &dry.s5, dry.eq.mdot_air,
                                         &Afterburner { m6, ..self.ab }, AbMode::Lit, true));
            if let Err(w) = named {
                if is_refusal(&w) {
                    return w;
                }
            }
        }
        let wall = e.0.trim_start_matches(REFUSED).trim_start_matches(": ");
        Abort(match wall {
            "too hot" => format!("The afterburner cannot reach {:.0} K at this throttle and flight: burning all the \
                                  oxygen left in the gas falls short.", self.ab.tt7),
            "too cold" => format!("The afterburner cannot cool the gas: it leaves the turbine hotter than the {:.0} K \
                                   asked at this throttle and flight.", self.ab.tt7),
            "the duct entry chokes" => "The gas leaves the turbine faster than the afterburner duct can pass it at \
                                        this throttle and flight: it would choke at the duct's entry.".to_string(),
            _ => format!("Heating the gas to {:.0} K would choke the afterburner duct at this throttle and flight.",
                         self.ab.tt7),
        })
    }

    /// The point at throttle `tt4` with the throat set by `throat`. *Match*: the dry engine first, then the area the
    /// lit exit needs at that core ([`Self::match_area`]), and the lit instant at the dry speed and that area — whose
    /// `π_t` is the dry one (the core cannot move; a gate). Unlit, *match* is the dry throat.
    pub fn try_point(
        &self, flight: &FlightCondition, tt4: f64, cmap: &ComponentMap, mode: AbMode, throat: Throat,
    ) -> Result<FlyPoint, Abort> {
        match (throat, mode) {
            (Throat::Held(s), _) => self.try_equilibrium(flight, tt4, cmap, mode, s * self.a8_dry),
            (Throat::Match, AbMode::Unlit) => self.try_equilibrium(flight, tt4, cmap, mode, self.a8_dry),
            (Throat::Match, AbMode::Lit) => {
                let dry = self.try_equilibrium(flight, tt4, cmap, AbMode::Unlit, self.a8_dry)?;
                self.try_lit_matched(flight, tt4, cmap, &dry)
            }
        }
    }

    /// [`Self::try_point`] when the DRY point is already solved (the page solves it anyway, for its readout): a held
    /// throat's search starts from the dry speed, *match* starts from the dry point itself.
    pub fn try_point_after_dry(
        &self, flight: &FlightCondition, tt4: f64, cmap: &ComponentMap, mode: AbMode, throat: Throat, dry: &FlyPoint,
    ) -> Result<FlyPoint, Abort> {
        match (throat, mode) {
            (Throat::Held(s), _) => self.try_equilibrium_from(flight, tt4, cmap, mode, s * self.a8_dry, Some(dry.eq.nu)),
            (Throat::Match, AbMode::Unlit) => Ok(*dry),
            (Throat::Match, AbMode::Lit) => self.try_lit_matched(flight, tt4, cmap, dry),
        }
    }

    /// *Match* on an already-solved DRY point (the page solves the dry engine anyway, for its readout).
    pub fn try_lit_matched(&self, flight: &FlightCondition, tt4: f64, cmap: &ComponentMap, dry: &FlyPoint)
        -> Result<FlyPoint, Abort>
    {
        let a8 = self.match_area(flight, tt4, cmap, dry)?;
        self.try_instant_hinted(flight, dry.eq.nu, tt4, cmap, AbMode::Lit, a8, Some(dry.eq.pi_t), true)
            .map_err(|e| self.name_refusal(flight, tt4, cmap, &e))
    }

    /// The matched throat at a solved DRY point: the lit afterburner on the dry turbine exit, and the area its nozzle
    /// needs, `ṁ_air·(1+f7)/(ρ9·V9)` — choked or not.
    pub fn match_area(&self, flight: &FlightCondition, tt4: f64, cmap: &ComponentMap, dry: &FlyPoint)
        -> Result<f64, Abort>
    {
        let mm = &self.st.inner.inner;
        let comp = self.st.try_close_compressor(tt4, dry.eq.tt2, dry.eq.pt2, cmap, dry.eq.n)?;
        let s4 = FlowState { tt: tt4, pt: comp.pt4, mdot: comp.mdot4, far: comp.f };
        let g = scratch(comp.gas(mm), &s4)?;
        let (_, flux, mdot7) = self.exit_at(&g, &s4, comp.mdot_air, dry.eq.eta_t, dry.eq.pi_t, AbMode::Lit, flight.p0)
            .map_err(|e| self.name_refusal(flight, tt4, cmap, &e))?;
        Ok(mdot7 / flux)
    }
}

// =============================================================================================
// PART C — the afterburner on the two-shaft engine of the Controls view, lit at a chosen moment of the slam
// =============================================================================================
//
// The afterburner sits behind the LP turbine. FITTED and unlit it is the shipped engine with its nozzle throat opened by
// `1/π_fh` and the flame holders' loss in the nozzle's (`π_n·π_fh`): the core swaps those two numbers at build
// ([`crate::two_spool::TwoSpoolCore::new`]), so every shipped path runs the fitted engine unchanged. LIT, the LP
// turbine's nozzle relation reads station 7 — rung 38's choke chained one throat further:
//
//     choked    π_LPT·π_AB·√(Tt45/Tt7) = A45·MFP45·(1+f7) / ((1+f45)·A8·π_n·MFP7)
//     unchoked  ṁ7 = A8·ρ9·V9                                        (p9 = p0)
//
// with the shipped `π_n` (the flame holders are inside `π_AB`). Lighting takes `LIGHT_RAMP` (HP spool τ): the burning
// temperature `Tt7(w) = Tt5 + w·(Tt7 − Tt5)` and the throat `A8_dry·(1 + w·(s − 1))` follow the light-off fraction `w`.
// *Match* leaves the LP turbine where the unlit engine has it (the core cannot move) and books the lit exit's thrust.

/// How long the afterburner takes to light, HP spool τ: five march steps, so the ramp's two corners land on the grid.
pub const LIGHT_RAMP: f64 = 0.10;

/// What the page sets for the two-shaft engine's afterburner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoShaftFit {
    pub ab: Afterburner,
    pub throat: Throat,
    /// When it lights, HP spool τ into the run (`None`: fitted, never lit).
    pub light_at: Option<f64>,
}

/// The afterburner captured on the two-shaft core.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoShaftAb {
    pub fit: TwoShaftFit,
    /// The burning zone's area, m², from the design run's station 5 at `M6` (∞ at `M6` 0).
    pub a6: f64,
    /// The dry throat, m² (`A8/π_fh` — the core's own after the swap).
    pub a8_dry: f64,
    /// The nozzle's own loss (the shipped `π_n`, before the flame holders joined it).
    pub pi_n: f64,
}

/// The afterburner at an instant of a lit run: what it is, and how far lit (0 < `w` ≤ 1).
#[derive(Clone, Copy, Debug)]
pub struct LitAb {
    pub ab: TwoShaftAb,
    pub w: f64,
}

impl TwoShaftAb {
    /// Capture on the design run: `s5` its station 5, `a8` the CORE throat and `pi_n` the nozzle's loss, both before
    /// the swap.
    pub fn capture(gas: &Gas, s5: &FlowState, a8: f64, pi_n: f64, fit: TwoShaftFit) -> Self {
        let a6 = if fit.ab.m6 == 0.0 { f64::INFINITY } else { area_at_mach(gas, s5.far, s5.tt, s5.pt, s5.mdot, fit.ab.m6) };
        TwoShaftAb { fit, a6, a8_dry: a8 / fit.ab.pi_fh, pi_n }
    }

    /// The light-off fraction at march time `s`: 0 until `light_at`, rising linearly to 1 over [`LIGHT_RAMP`]. Within
    /// 1e−9 of the light-off it is 0 — the march ACCUMULATES `s`, so a grid time can land an ulp past it.
    pub fn w_at(&self, s: f64) -> f64 {
        match self.fit.light_at {
            None => 0.0,
            Some(on) if s <= on + 1e-9 => 0.0,
            Some(on) if s >= on + LIGHT_RAMP => 1.0,
            Some(on) => (s - on) / LIGHT_RAMP,
        }
    }
}

impl LitAb {
    /// The throat over the dry throat now (*match*: `None` — sized at the point).
    pub fn s_now(&self) -> Option<f64> {
        match self.ab.fit.throat {
            Throat::Held(s) => Some(1.0 + self.w * (s - 1.0)),
            Throat::Match => None,
        }
    }

    /// Station 5 → 7 at `s5` (`s5.mdot` the GAS through the turbine): the duct's Mach off its area, the burning
    /// temperature `Tt7(w)`. Gas arriving at or above it is not heated (the duct unlit, at `π_fh`).
    pub fn burn(&self, gas: &Gas, s5: &FlowState, name: bool) -> Result<(FlowState, AbDetail), Abort> {
        C_BURN.with(|c| c.set(c.get() + 1));
        let ab = &self.ab.fit.ab;
        let tt7 = s5.tt + self.w * (ab.tt7 - s5.tt);
        if !(tt7 > s5.tt) {
            let s7 = FlowState { pt: ab.pi_fh * s5.pt, ..*s5 };
            let m6 = m6_in_duct(gas, s5.far, s5.tt, s5.pt, s5.mdot, self.ab.a6, ab.m6, name)?;
            let detail = AbDetail { mode: AbMode::Unlit, f5: s5.far, f7: s5.far, pi_ab: ab.pi_fh, pi_r: 1.0, m6, m7: m6,
                                    passes: 1 };
            return Ok((s7, detail));
        }
        let m6 = m6_in_duct(gas, s5.far, s5.tt, s5.pt, s5.mdot, self.ab.a6, ab.m6, name)?;
        afterburn(gas, s5, s5, s5.mdot / (1.0 + s5.far), &Afterburner { tt7, m6, ..*ab }, AbMode::Lit, name)
    }

    fn lean_wall(&self, s: f64) -> Abort {
        C_WALL.with(|c| c.set(c.get() + 1));
        Abort(format!(
            "With the nozzle at {s:.2}× its dry size and the afterburner lit, the hot gas cannot get out at this engine \
             pressure: there is no running point here (the model keeps both turbine throats choked; a real engine would \
             stall). Open the nozzle further, or light the afterburner at a higher throttle."))
    }

    /// The LP turbine `(π_LPT, τ_LPT, Tt5)` with the afterburner lit and the throat held at `s_now` × dry: the choked
    /// relation above, else (the nozzle unchoked at that root) the subsonic one. A failing search is named: the
    /// afterburner's own refusal in § 1's words, or the lean wall.
    #[allow(clippy::too_many_arguments)]
    pub fn try_lp_turbine(
        &self, base: &crate::two_spool::TwoSpoolCore, gas: &Gas, tt45: f64, f45: f64, eta: f64, pt45: f64, p0: f64,
        s_now: f64,
    ) -> Result<(f64, f64, f64), Abort> {
        C_LP.with(|c| c.set(c.get() + 1));
        let a8 = s_now * self.ab.a8_dry;
        // RUNG 104: the LP turbine's `W45` — the choked vane's `A45·MFP*(Tt45)` under the pin, else the law's at `π`.
        let law = base.turbine_law.get() != crate::turbine_law::TurbineLaw::Pin;
        let mfp45 = if law { f64::NAN } else { try_choked_mfp(gas, tt45, f45)? };
        let w_pin = base.a45_pin() * mfp45;
        let w45 = |pi: f64| -> Result<f64, Abort> { if law { base.w45(gas, pi, tt45, f45, eta) } else { Ok(w_pin) } };
        let trial = |pi: f64, name: bool| -> Result<(f64, f64, FlowState, AbDetail), Abort> {
            let (tau, tt5) = base.try_tau_of(gas, tt45, f45, pi, eta)?;
            // The pin's literal old order (bit for bit), else the law's flow at this `π`.
            let mdot5 = if law { pt45 * w45(pi)? / powp(tt45, 0.5) } else { base.a45_pin() * pt45 * mfp45 / powp(tt45, 0.5) };
            let s5 = FlowState { tt: tt5, pt: pi * pt45, mdot: mdot5, far: f45 };
            let (s7, d) = self.burn(gas, &s5, name)?;
            Ok((tau, tt5, s7, d))
        };
        // A refusal inside the search, put into words at the trial that failed.
        let named = |pi: f64, e: Abort| -> Abort { if is_refusal(&e) { trial(pi, true).err().unwrap_or(e) } else { e } };
        let resid = |pi: f64| -> Result<f64, Abort> {
            let (_, _, s7, d) = trial(pi, false)?;
            let rhs = w45(pi)? / ((1.0 + f45) * a8 * self.ab.pi_n);
            Ok(pi * d.pi_ab * powp(tt45 / s7.tt, 0.5) - rhs * (1.0 + s7.far) / try_choked_mfp(gas, s7.tt, s7.far)?)
        };
        // The residual rises with π_LPT: below zero at the top (stepped down past refusals), the gas cannot leave the
        // throat even with the LP turbine nearly unexpanded — the lean wall, answered without searching below it.
        let (lo, flo, hi, fhi) = bracket_top_first(&resid, 0.02, 0.999, 0.01).map_err(|(x, e)| named(x, e))?;
        if !(flo < 0.0 && 0.0 < fhi) {
            return Err(self.lean_wall(s_now));
        }
        let pi_c = try_illinois(resid, lo, hi, flo, fhi, SpoolTransient::HOT_TOL, ILLINOIS_MAXIT)?;
        let (tau, tt5, s7, _) = trial(pi_c, true)?;
        let nozzle = Nozzle::convergent(p0, self.ab.pi_n);
        if nozzle.try_apply(&s7, gas)?.p9 > p0 + 1e-6 {
            return Ok((pi_c, tau, tt5));
        }
        // Unchoked: `ṁ7 − A8·ρ9·V9` falls as π_LPT rises; the choked root sits on its flow-short side (Part B's bracket).
        let sub = |pi: f64| -> Result<f64, Abort> {
            let (_, _, s7, _) = trial(pi, false)?;
            let e = nozzle.try_apply(&s7, gas)?;
            Ok(s7.mdot - a8 * e.p9 / (gas.r_t_at(s7.far) * e.t9) * e.v9)
        };
        let rlo = sub(pi_c)?;
        let mut top = crate::unchoked_lp::PI_TOP;
        let rhi = loop {
            match sub(top) {
                Ok(r) => break r,
                Err(e) if top - 0.01 <= pi_c => return Err(named(top, e)),
                Err(_) => top -= 0.01,
            }
        };
        if !(rlo > 0.0 && rhi < 0.0) {
            return Err(self.lean_wall(s_now));
        }
        let pi = try_illinois(sub, pi_c, top, rlo, rhi, SpoolTransient::HOT_TOL, ILLINOIS_MAXIT)?;
        let (tau, tt5, _, _) = trial(pi, true)?;
        Ok((pi, tau, tt5))
    }
}

/// The nozzle exit behind the LP turbine and the fuel-air ratio there, for a thrust tail: the shipped nozzle on `s5`
/// when the afterburner is not lit (bit for bit; fitted, the core's swapped `π_n` carries the flame holders), else the
/// station-7 exit (`s5.mdot` the core AIR, as the tails pass it).
pub fn lp_exit(base: &crate::two_spool::TwoSpoolCore, gas: &Gas, s5: &FlowState, p0: f64)
    -> Result<(NozzleExit, f64), Abort>
{
    let Some(x) = base.lit_ab() else {
        return Ok((Nozzle::convergent(p0, base.pi_n).try_apply(s5, gas)?, s5.far));
    };
    let g5 = FlowState { mdot: s5.mdot * (1.0 + s5.far), ..*s5 };
    let (s7, detail) = x.burn(gas, &g5, true)?;
    let exit = Nozzle::convergent(p0, x.ab.pi_n).try_apply(&s7, gas)?;
    // The throat this exit used, over the dry one: held, the setting; matched, the area the lit exit needs.
    let s = x.s_now().unwrap_or_else(|| s7.mdot / (exit.p9 / (gas.r_t_at(s7.far) * exit.t9) * exit.v9) / x.ab.a8_dry);
    base.ab_last.set(Some(AbAt { s7, detail, s, w: x.w }));
    Ok((exit, s7.far))
}

/// What a lit afterburner did at the last instant a thrust tail solved ([`lp_exit`]) — the readout's source.
#[derive(Clone, Copy, Debug)]
pub struct AbAt {
    pub s7: FlowState,
    pub detail: AbDetail,
    /// The throat over the dry throat.
    pub s: f64,
    /// How far lit.
    pub w: f64,
}

/// Is this a lit afterburner's failure (its own refusal, or the lean wall)?
pub fn is_lit_failure(e: &Abort) -> bool {
    is_refusal(e) || e.0.contains("and the afterburner lit, the hot gas cannot get out")
}
