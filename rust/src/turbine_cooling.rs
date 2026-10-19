//! RUNG 91 — TURBINE COOLING AIR: HP-compressor air that rejoins the gas at the turbines.
//!
//! Anchor (the derivation, the linear model and every pre-registered bar, committed before any of
//! this code existed): `docs/plans/rung91-anchor-turbine-cooling.md`. Spec: `docs/rung91-spec.md`.
//!
//! Two streams, each a share of the HPC delivery (station 3), beside rung 90's customer share `c`:
//! **vane** cooling `ε₁` rejoins after the HPT nozzle throat (A4) and before the rotor, so it works
//! in the HP turbine; **blade** cooling `ε₂` rejoins after the rotor and before the LPT nozzle
//! throat (A45), so it works only in the LP turbine. The burner gets `1 − c − ε₁ − ε₂`.
//!
//! # The one place it breaks every earlier two-spool rung — the HPT pin
//!
//! Rung 31's `(★)` between the A4 and A45 throats held the HP turbine at ONE point on the CPG gas.
//! The A45 throat now passes the coolant as well, at the mixed temperature, so
//! `π_HPT √(Tt4/Tt45) = (A4·MFP4)/(A45·MFP45) · (G+ε)/G`: **the HP turbine moves**
//! ([`try_solve_cooled_hpt`]). The LPT pin keeps its form, fed with the mixed `Tt45`.
//!
//! The steady cascade loses its triangle with it: both mixers need `Tt3`, which the HP balance
//! makes from the HPT — closed by a fixed point on `Tt3` ([`try_cascade_cooled`]). The transient
//! closure has no such loop: there `Tt3` comes from the compressor maps before any turbine.
//!
//! # The mixers
//!
//! Adiabatic and at constant total pressure (Mattingly's coolant mixers), on the SAME enthalpy zero
//! as the burner's own balance; the fuel/air ratio is diluted at each. A mixer takes TWO streams, so
//! it is not an `apply(state, gas)` component — the same declared divergence as the Turbine and the
//! Nozzle — and [`mix`] asserts its own mass, fuel and energy balances. **The perfect gas only**: a
//! mixer on a reacting gas would re-react the diluted mixture
//! ([`crate::two_spool::TwoSpoolEngine::with_turbine_cooling`] refuses the others).
//!
//! # Reduce — by DISPATCH
//!
//! Inactive cooling (`None`, both shares 0) never enters this module's arithmetic: the steady cell
//! hands to rung 90's body (hence rung 42's / 39's), the transient cells to rung 90's or rung 62's
//! (hence rung 57's). Every body that cannot model cooling refuses an ACTIVE one
//! ([`crate::two_spool::TwoSpoolCore::refuse_cooling`]).
//!
//! # The bodies are COPIES, not parameterisations
//!
//! Rung 90's steady rebuild and rung 62's four transient cells (as rung 90 carried them) are copied
//! here with the cooling inserted ([[rust-port-copy-vs-rederivation]]); rung 90's two efficiency
//! loops are CALLED, with the extraction `c + ε` — their bodies carry the extraction only in the
//! face referral, where the cooling air is exactly as absent as the customer air.

use std::cell::Cell;

use crate::components::{try_choked_mfp, ram_recovery, Burner, Compressor, Inlet, Nozzle, Turbine};
use crate::customer_bleed::{hp_eta_loop_customer, lp_eta_loop_customer, CustomerBleed};
use crate::engine::{FlightCondition, Performance};
use crate::fuel_transient::{FuelCloseState, FuelTransientCore};
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::matcher::Branch;
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::two_spool::{
    Cascade, CascadeMap, TwoSpoolCore, TwoSpoolEngine, TwoSpoolHooks, TwoSpoolMapCore,
    TwoSpoolMapResult, TwoSpoolResult, R39,
};
use crate::two_spool_transient::{CloseState, Instant2, TwoSpoolTransientCore};

// =========================================================================================
// THE COOLING
// =========================================================================================

/// The two cooling streams, each a SHARE of the HPC delivery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurbineCooling {
    /// Vane (NGV) cooling: rejoins after the A4 throat, before the HPT rotor — it works there.
    pub vane: f64,
    /// Blade (rotor) cooling: rejoins after the HPT rotor, before the A45 throat — LPT work only.
    pub blade: f64,
}

impl TurbineCooling {
    /// The largest share either stream may take.
    pub const SHARE_MAX: f64 = 0.25;
    /// The largest share the customer and both streams may take TOGETHER at a converged point —
    /// rung 42's bound, as rung 90's.
    pub const TOTAL_MAX: f64 = 0.5;

    /// Range-checked: each share in `[0, 0.25)`.
    pub fn new(vane: f64, blade: f64) -> Self {
        for (name, x) in [("vane", vane), ("blade", blade)] {
            assert!((0.0..Self::SHARE_MAX).contains(&x),
                    "rung-91 {name} cooling share must be in [0, 0.25) of the HP compressor's air: \
                     got {x}");
        }
        TurbineCooling { vane, blade }
    }

    /// Does any air go to the turbines? Both shares 0 is INACTIVE, and dispatches away.
    pub fn is_active(&self) -> bool { self.vane != 0.0 || self.blade != 0.0 }

    /// `ε = ε₁ + ε₂`.
    pub fn total(&self) -> f64 { self.vane + self.blade }
}

/// Can the mixers run on this gas? The perfect gas only (see the module note).
pub fn is_perfect_gas(gas: &Gas) -> bool {
    let s = &gas.spec;
    s.cp_c_coeffs.is_none() && s.cp_t_coeffs.is_none() && !s.reacting_hot && !s.fork_b
        && !s.equilibrium
}

/// The customer share at an HPC flow, UNCHECKED (a bracket trial may sit anywhere — rung 90's
/// lesson 2). No customer bleed is a zero share.
pub fn customer_raw(cb: Option<CustomerBleed>, hpc: f64, mdot_design: f64) -> f64 {
    match cb {
        None => 0.0,
        Some(c) => c.share_raw(hpc, mdot_design),
    }
}

/// The HPC air that feeds `burner` kg/s to the burner with the customer bleed and cooling `eps`
/// taken off: `burner/(1−c−ε)`, or `(burner + W)/(1−ε)` for a fixed customer amount.
pub fn hpc_from_burner(cb: Option<CustomerBleed>, eps: f64, burner: f64, mdot_design: f64) -> f64 {
    match cb {
        None => burner / (1.0 - eps),
        Some(CustomerBleed::Share(c)) => burner / (1.0 - c - eps),
        Some(CustomerBleed::Amount(w)) => (burner + w * mdot_design) / (1.0 - eps),
    }
}

/// The bound at a converged point: the customer and the cooling together take less than half.
pub fn check_root(c: f64, eps: f64, hpc: f64) -> Result<(), Abort> {
    if !(c + eps < TurbineCooling::TOTAL_MAX) || !(hpc > 0.0) {
        return Err(Abort(format!(
            "rung-91: the aircraft ({:.1} %) and turbine cooling ({:.1} %) together take {:.1} % of \
             the HP compressor's air here (HPC flow {hpc:.4} kg/s): more than half, so the burner is \
             starved.", 100.0 * c, 100.0 * eps, 100.0 * (c + eps))));
    }
    Ok(())
}

// =========================================================================================
// THE MIXERS
// =========================================================================================

/// The mixed total temperature of `gas_flow` kg/s of hot gas at `(tt, far)` plus `air` kg/s of
/// compressor air at `tt_cool`, landing at the diluted `far_out` — the adiabatic balance on the
/// burner's own enthalpy zero. Zero air returns `tt` EXACTLY.
pub fn mixed_tt(
    gas: &Gas, gas_flow: f64, tt: f64, far: f64, air: f64, tt_cool: f64, far_out: f64,
) -> f64 {
    if air == 0.0 {
        return tt;
    }
    let h = (gas_flow * gas.h_t(tt, far) + air * gas.h_c(tt_cool)) / (gas_flow + air);
    gas.t_from_h_t(h, far_out)
}

/// A coolant MIXER on full stations: `main` (hot gas, `mdot` the GAS flow, `far` carried) plus
/// `air` kg/s of compressor air at `(tt_cool, pt_cool)`. Total pressure is the gas path's (the
/// coolant must arrive above it, or it could not flow in). Asserts its own mass, fuel and energy
/// balances — the conservation checks the working contract makes assertions.
pub fn mix(main: &FlowState, air: f64, tt_cool: f64, pt_cool: f64, gas: &Gas) -> FlowState {
    assert!(air >= 0.0, "rung-91 mixer: negative coolant flow {air}");
    assert!(pt_cool >= main.pt,
            "rung-91 mixer: coolant at {pt_cool:.0} Pa is below the gas path's {:.0} Pa — it \
             cannot flow in", main.pt);
    let air_main = main.mdot / (1.0 + main.far);
    let fuel = main.mdot - air_main;
    let far = fuel / (air_main + air);
    let tt = mixed_tt(gas, main.mdot, main.tt, main.far, air, tt_cool, far);
    let out = FlowState { tt, pt: main.pt, mdot: main.mdot + air, far };
    // Mass (gas = air + fuel), fuel, and energy.
    assert!((out.mdot - (air_main + air) * (1.0 + out.far)).abs() <= 1e-12 * out.mdot,
            "rung-91 mixer: mass not conserved");
    assert!((out.far * (air_main + air) - fuel).abs() <= 1e-12 * out.mdot,
            "rung-91 mixer: fuel not conserved");
    let e_in = main.mdot * gas.h_t(main.tt, main.far) + air * gas.h_c(tt_cool);
    let e_out = out.mdot * gas.h_t(out.tt, out.far);
    assert!((e_out - e_in).abs() <= 1e-9 * e_in.abs(), "rung-91 mixer: energy not conserved");
    out
}

// =========================================================================================
// THE COOLED HP TURBINE PIN
// =========================================================================================

/// The HP turbine between the A4 and A45 throats with both coolant streams.
#[derive(Clone, Copy, Debug)]
pub struct CooledHpt {
    /// `pt45/pt4` — the rotor's expansion (the mixers keep total pressure).
    pub pi_hpt: f64,
    /// The ROTOR's temperature ratio `Tt44/Tt41`.
    pub tau_rotor: f64,
    pub tt41: f64,
    pub tt44: f64,
    pub tt45: f64,
    /// The diluted fuel/air ratios after mixer 1 and mixer 2.
    pub f41: f64,
    pub f45: f64,
}

/// Bisect `π_HPT` so the A45 throat passes the burner gas AND both coolants:
/// `π √(Tt4/Tt45) = A4·MFP(Tt4)/(A45·MFP(Tt45)) · (G+ε)/G`. Per unit HPC air: `burner` the burner's
/// share, `G = burner(1+f)`. The rotor's efficiency `eta` is given (the map reads it at `Tt41`).
#[allow(clippy::too_many_arguments)]
pub fn try_solve_cooled_hpt(
    base: &TwoSpoolCore, gas: &Gas, tt4: f64, f: f64, burner: f64, e1: f64, e2: f64, tt3: f64,
    eta: f64,
) -> Result<CooledHpt, Abort> {
    let g = burner * (1.0 + f);
    let fuel = burner * f;
    let f41 = fuel / (burner + e1);
    let f45 = fuel / (burner + e1 + e2);
    let tt41 = mixed_tt(gas, g, tt4, f, e1, tt3, f41);
    let mfp4 = try_choked_mfp(gas, tt4, f)?;
    let rhs0 = base.a4 * mfp4 * (g + e1 + e2) / (base.a45 * g);
    let at = |pi: f64| -> Result<(f64, f64, f64, f64), Abort> {
        let (tau, tt44) = base.try_tau_of(gas, tt41, f41, pi, eta)?;
        let tt45 = mixed_tt(gas, g + e1, tt44, f41, e2, tt3, f45);
        let mfp45 = try_choked_mfp(gas, tt45, f45)?;
        Ok((pi * powp(tt4 / tt45, 0.5) - rhs0 / mfp45, tau, tt44, tt45))
    };
    let (mut lo, mut hi) = (0.02, 0.999);
    let (mut flo, fhi) = (at(lo)?.0, at(hi)?.0);
    if !(flo < 0.0 && 0.0 < fhi) {
        return Err(Abort(format!(
            "rung-91 cooled HP turbine pin does not bracket at Tt4={tt4:.0}, vane={e1:.3}, \
             blade={e2:.3}: the LP turbine's throat cannot pass the coolant at any HP expansion.")));
    }
    for _ in 0..TwoSpoolCore::MAX {
        let mid = 0.5 * (lo + hi);
        let fm = at(mid)?.0;
        if flo * fm <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
            flo = fm;
        }
        if hi - lo <= TwoSpoolCore::TOL {
            break;
        }
    }
    let pi_hpt = 0.5 * (lo + hi);
    let (_, tau_rotor, tt44, tt45) = at(pi_hpt)?;
    Ok(CooledHpt { pi_hpt, tau_rotor, tt41, tt44, tt45, f41, f45 })
}

// =========================================================================================
// DISPATCH COUNTERS
// =========================================================================================

thread_local! {
    static STEADY_ACTIVE: Cell<u64> = const { Cell::new(0) };
    static STEADY_INACTIVE: Cell<u64> = const { Cell::new(0) };
    static CLOSE_R91: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_R91: Cell<u64> = const { Cell::new(0) };
    static POWERS_R91: Cell<u64> = const { Cell::new(0) };
    static TAIL_R91: Cell<u64> = const { Cell::new(0) };
}

fn bump(k: &'static std::thread::LocalKey<Cell<u64>>) {
    k.with(|x| x.set(x.get() + 1));
}

/// How many times each rung-91 body ran on this thread — `(steady active, steady inactive,
/// close, close_fuel, powers, tail)`. Reset by [`reset_counts`].
pub fn counts() -> [u64; 6] {
    [&STEADY_ACTIVE, &STEADY_INACTIVE, &CLOSE_R91, &CLOSE_FUEL_R91, &POWERS_R91, &TAIL_R91]
        .map(|k| k.with(|x| x.get()))
}

pub fn reset_counts() {
    for k in [&STEADY_ACTIVE, &STEADY_INACTIVE, &CLOSE_R91, &CLOSE_FUEL_R91, &POWERS_R91,
              &TAIL_R91] {
        k.with(|x| x.set(0));
    }
}

// =========================================================================================
// STEADY — THE HOOK TABLE AND THE CASCADE
// =========================================================================================

/// RUNG 91's steady table. Only `try_match_point` changes (rung 90's reason).
pub const R91: TwoSpoolHooks = TwoSpoolHooks {
    try_match_point: r91_try_match_point,
    hp_eta_loop: R39.hp_eta_loop,
    lp_eta_loop: R39.lp_eta_loop,
};

fn r91_try_match_point(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<TwoSpoolMapResult, Abort> {
    try_match_cooled(core, flight, tt4).map(|r| r.base)
}

/// The cooled cascade's answer: rung 39's map cascade, the cooled HPT and the `Tt3` it closed on.
#[derive(Clone, Debug)]
pub struct CooledCascade {
    pub map: CascadeMap,
    pub hpt: CooledHpt,
}

/// Rung 90's cascade with the two cooling streams. **No longer triangular**: the mixers need
/// `Tt3`, the HP balance makes `Tt3` from the HPT — closed here by a fixed point on `Tt3` (seeded
/// with `tt3_seed`), inside the turbine-efficiency loop.
#[allow(clippy::too_many_arguments)]
pub fn try_cascade_cooled(
    core: &TwoSpoolMapCore, wgas: &Gas, tt2: f64, tt4: f64, f: f64, c: f64, cool: TurbineCooling,
    tt3_seed: f64,
) -> Result<CooledCascade, Abort> {
    let b = core.bleed;
    let base = &core.base;
    let (e1, e2) = (cool.vane, cool.blade);
    let eps = e1 + e2;
    let burner = 1.0 - c - eps;
    let mfp4 = try_choked_mfp(wgas, tt4, f)?;
    let (mut eta_hpt, mut eta_lpt, mut tt3) = (base.eta_hpt, base.eta_lpt, tt3_seed);
    for _ in 0..TwoSpoolCore::MAX {
        let hpt = try_solve_cooled_hpt(base, wgas, tt4, f, burner, e1, e2, tt3, eta_hpt)?;
        let (pi_lpt, tau_lpt, tt5) = crate::steady_unchoked::try_lp_steady(
            base, wgas, hpt.tt45, hpt.f45, eta_lpt, hpt.pi_hpt)?;

        // The LP balance, per FACE air: the LPT passes (1-b)(G + eps) of gas.
        let g = burner * (1.0 + f);
        let dh_lpt = base.eta_m * (1.0 - b) * (g + eps)
            * (wgas.h_t(hpt.tt45, hpt.f45) - wgas.h_t(tt5, hpt.f45));
        let tt25 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_lpt);
        // The HP balance, per HPC air: the ROTOR passes G + eps1 from Tt41 to Tt44.
        let dh_hpt = base.eta_m * (g + e1)
            * (wgas.h_t(hpt.tt41, hpt.f41) - wgas.h_t(hpt.tt44, hpt.f41));
        let tt3_new = wgas.t_from_h_c(wgas.h_c(tt25) + dh_hpt);

        // Both face referrals miss the cooling air exactly as they miss the customer air.
        let hp = hp_eta_loop_customer(wgas, tt4, f, tt25, tt3_new, mfp4, &core.map_hp(), c + eps,
                                      base.eta_hpc, base.a4, base.pi_b, core.mcorr_hp_d,
                                      core.tau_hpc_d)?;
        let lp = lp_eta_loop_customer(wgas, tt2, tt4, f, tt25, mfp4, hp.pi, &core.map_lp(), b,
                                      c + eps, base.eta_lpc, base.a4, base.pi_b, core.mcorr_lp_d,
                                      core.tau_lpc_d)?;

        let nl = lp.n * powp(tt2 / core.tt2_d, 0.5);
        let nh = hp.n * powp(tt25 / core.tt25_d, 0.5);
        // The HPT map's corrected speed is the ROTOR's: referred to Tt41 (= Tt4 with no vane air).
        let nu_hpt = nh * powp(core.tt4_d / hpt.tt41, 0.5);
        let nu_lpt = nl * powp(core.tt45_d / hpt.tt45, 0.5);
        let map = CascadeMap {
            c: Cascade { pi_hpt: hpt.pi_hpt, tau_hpt: hpt.tau_rotor, tt45: hpt.tt45, pi_lpt,
                         tau_lpt, tt5, pi_lpc: lp.pi, tt25, pi_hpc: hp.pi, tt3: tt3_new },
            eta_lpc: lp.eta, eta_hpc: hp.eta, eta_hpt, eta_lpt,
            m_l: lp.m, m_h: hp.m, n_l: lp.n, n_h: hp.n, nl, nh,
            phi_l: lp.m / lp.n, phi_h: hp.m / hp.n, nu_hpt, nu_lpt, slip: nl / nh,
        };
        let t_hpt = core.map_hp().eta_t_at(base.eta_hpt, nu_hpt);
        let t_lpt = core.map_lp().eta_t_at(base.eta_lpt, nu_lpt);
        if (t_hpt - eta_hpt).abs() <= TwoSpoolMapCore::ETA_TOL
            && (t_lpt - eta_lpt).abs() <= TwoSpoolMapCore::ETA_TOL
            && (tt3_new - tt3).abs() <= 1e-12 * tt3_new {
            return Ok(CooledCascade { map, hpt });
        }
        eta_hpt = t_hpt;
        eta_lpt = t_lpt;
        tt3 = tt3_new;
    }
    Err(Abort(format!("rung-91 cooled cascade (Tt3 fixed point / turbine efficiencies) did not \
                       converge at Tt4={tt4}")))
}

// =========================================================================================
// STEADY — MATCH ONE POINT
// =========================================================================================

/// What the cooling books on a matched point — present only when it was ACTIVE.
#[derive(Clone, Copy, Debug)]
pub struct CoolingBooking {
    /// The interstage (rung 42) share, the customer share (rung 90; 0 with none) and the two
    /// cooling shares, all of the HPC delivery except `bleed` (of the face air).
    pub bleed: f64,
    pub customer: f64,
    pub vane: f64,
    pub blade: f64,
    /// Air through the HPC, the burner, the vanes, the blades and the nozzle, kg/s.
    pub mdot_hpc: f64,
    pub mdot_burner: f64,
    pub mdot_vane: f64,
    pub mdot_blade: f64,
    pub mdot_nozzle_air: f64,
    /// The rotor-inlet, rotor-exit and LPT-inlet temperatures (mixer 1, rotor, mixer 2).
    pub tt41: f64,
    pub tt44: f64,
    pub tt45: f64,
    /// `F / ṁ_INLET`: the cooling air leaves through the nozzle; only the interstage and customer
    /// air carry full ram drag with no momentum back.
    pub st_inlet: f64,
    /// Fuel per unit thrust, per inlet air.
    pub tsfc_inlet: f64,
}

/// A matched two-spool point with turbine cooling.
#[derive(Clone, Debug)]
pub struct CooledResult {
    /// `base.performance` is NOZZLE-AIR-referenced (fuel per nozzle air); `base.thrust` is the
    /// honest one.
    pub base: TwoSpoolMapResult,
    /// `None` exactly when the cooling was INACTIVE and rung 90's body produced this point.
    pub booking: Option<CoolingBooking>,
}

/// Rung 1's scoring ([`crate::engine::try_score`]) with the fuel ratio passed EXPLICITLY: there it
/// is read off station 4, which is the burner's — the nozzle passes the coolant too, so the ratio
/// that scores the exhaust is the diluted `f45`. A copy, not a parameterisation of the shipped one.
/// Rung 93's cooled DESIGN run scores its exhaust with it too.
pub(crate) fn try_score_nozzle(
    stations: &[(&'static str, FlowState)], f: f64, gas: &Gas, v0: f64, t9: f64, v9: f64,
    p9: f64, p0: f64, hpr: f64,
) -> Result<Performance, Abort> {
    let st = |l: &str| stations.iter().find(|(x, _)| *x == l).expect("station").1;
    let pressure_thrust = (1.0 + f) * gas.r_t_at(f) * t9 * (1.0 - p0 / p9) / v9;
    let specific_thrust = (1.0 + f) * v9 - v0 + pressure_thrust;
    let tsfc = f / specific_thrust;
    let eta_brayton = 1.0 - st("2").tt / st("3").tt;
    let ke_net = (1.0 + f) * (v9 * v9) - (v0 * v0);
    let eta_thermal = ke_net / (2.0 * f * hpr);
    let eta_propulsive = (specific_thrust * v0) / (0.5 * ke_net);
    let eta_overall = (specific_thrust * v0) / (f * hpr);
    if !((eta_overall - eta_thermal * eta_propulsive).abs() < 1e-9 * eta_overall) {
        return Err(Abort("efficiency cascade eta_o == eta_thermal*eta_p must hold under the KE \
                          definition".to_string()));
    }
    Ok(Performance { specific_thrust, tsfc, eta_brayton, eta_thermal, eta_propulsive, eta_overall })
}

/// Match at `(flight, Tt4)` with the interstage valve at `core.bleed`, the customer bleed and the
/// turbine cooling off the design engine. **Inactive cooling ⇒ rung 90's body verbatim.**
pub fn try_match_cooled(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<CooledResult, Abort> {
    // RUNG 103: behind the switch, a point the pinned body cannot finish is re-solved with the LP turbine free.
    crate::steady_unchoked::dispatch(&core.base, flight.p0, tt4, || cooled_body(core, flight, tt4),
                                     |x| x.base.base.thrust)
}

fn cooled_body(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<CooledResult, Abort> {
    core.base.refuse_offtake("rung 91's turbine-cooling match");
    if !core.base.cooling_active() {
        bump(&STEADY_INACTIVE);
        let r = crate::customer_bleed::try_match_customer(core, flight, tt4)?;
        return Ok(CooledResult { base: r.base, booking: None });
    }
    bump(&STEADY_ACTIVE);
    assert!(core.stack_lp.is_none() && core.stack_hp.is_none(),
            "rung-91's steady body lumps both compressors; rung 55's stage stacks are out of scope");
    let cool = core.base.cooling.expect("active");
    let (e1, e2) = (cool.vane, cool.blade);
    let eps = cool.total();
    let cb = core.base.customer.filter(|c| c.is_active());
    let b = core.bleed;
    let base = &core.base;
    let mdot_d = base.mdot_air_design;
    let pi_d = base.pi_d_max * ram_recovery(flight.m0);
    let (state0, _v0) = base.try_freestream_for(flight)?;
    let (tt2, pt2) = (state0.tt, pi_d * state0.pt);

    // Rung 90's joint (f, pt4, c) loop, the cascade now carrying the Tt3 fixed point, seeded from
    // the design run and then from the previous pass.
    let (mut f, mut pt4) =
        (base.f_design, base.pi_b * base.pi_hpc_design * base.pi_lpc_design * pt2);
    let mut c = match cb {
        None => 0.0,
        Some(CustomerBleed::Share(c)) => c,
        Some(CustomerBleed::Amount(w)) => w,
    };
    let mut tt3_seed = base.reference.station("3").tt;
    let mut cc: Option<CooledCascade> = None;
    for _ in 0..TwoSpoolCore::MAX {
        crate::steady_unchoked::track_pt4(base, pt4);
        let owned = base.try_working_gas(f, tt4, pt4)?;
        let wgas = owned.as_ref().unwrap_or(base.gas());
        let x = try_cascade_cooled(core, wgas, tt2, tt4, f, c, cool, tt3_seed)?;
        let pt4_new = base.pi_b * x.map.c.pi_hpc * x.map.c.pi_lpc * pt2;
        let f_new = base.try_solve_f(x.map.c.tt3, pt4_new, tt4)?;
        let c_new = match cb {
            Some(CustomerBleed::Amount(_)) => {
                let burner = base.a4 * pt4_new * try_choked_mfp(wgas, tt4, f_new)?
                    / powp(tt4, 0.5) / (1.0 + f_new);
                customer_raw(cb, hpc_from_burner(cb, eps, burner, mdot_d), mdot_d)
            }
            _ => c,
        };
        let done = (f_new - f).abs() <= TwoSpoolCore::TOL * (f_new + 1e-30)
            && (pt4_new - pt4).abs() <= TwoSpoolCore::TOL * pt4_new
            && (c_new - c).abs() <= TwoSpoolCore::TOL * (c_new + 1e-30);
        tt3_seed = x.map.c.tt3;
        cc = Some(x);
        f = f_new;
        pt4 = pt4_new;
        c = c_new;
        if done {
            break;
        }
    }
    let x = cc.expect("the joint loop runs at least once");
    let m = &x.map;
    if !(m.c.pi_lpc > 1.0 && m.c.pi_hpc > 1.0 && 0.0 < m.c.tau_hpt && m.c.tau_hpt < 1.0
         && 0.0 < m.c.tau_lpt && m.c.tau_lpt < 1.0) {
        return Err(Abort(format!("rung-91 cooled match unphysical at Tt4={tt4:.0}")));
    }

    let owned = base.try_working_gas(f, tt4, pt4)?;
    let wgas = owned.as_ref().unwrap_or(base.gas());
    let mdot_burner = base.a4 * pt4 * try_choked_mfp(wgas, tt4, f)? / powp(tt4, 0.5) / (1.0 + f);
    let mdot_hpc = hpc_from_burner(cb, eps, mdot_burner, mdot_d);
    // The bound is a property of the ROOT (rung 90's lesson 2).
    check_root(c, eps, mdot_hpc)?;
    let mdot_air = mdot_hpc / (1.0 - b);           // what the INLET ingests

    // --- the forward rebuild: both extractions at station 3, both returns at their mixers ------
    let rgas = base.gas().clone();
    let (state0, v0) = base.try_freestream_at(flight, mdot_air)?;
    let s2 = Inlet::new(pi_d).apply(&state0, &rgas);
    let s25 = Compressor::new(m.c.pi_lpc, m.eta_lpc, None).apply(&s2, &rgas);
    let s25c = FlowState { mdot: (1.0 - b) * s25.mdot, ..s25 };      // rung 42's extraction
    let s3 = Compressor::new(m.c.pi_hpc, m.eta_hpc, None).apply(&s25c, &rgas);
    let (mdot_vane, mdot_blade) = (e1 * s3.mdot, e2 * s3.mdot);
    // Rung 90's customer air and both cooling streams leave before the burner.
    let s3c = FlowState { mdot: mdot_burner, ..s3 };
    let s4 = Burner::new(tt4, base.eta_b, base.pi_b).apply(&s3c, &rgas);
    let s41 = mix(&s4, mdot_vane, s3.tt, s3.pt, &rgas);                 // <- MIXER 1 (vane)
    // (3): the rotor drives the HPC air with the gas it passes, G + eps1 per HPC air.
    let dh_hpt_reb = (rgas.h_c(s3.tt) - rgas.h_c(s25.tt)) * s3.mdot / (base.eta_m * s41.mdot);
    let s44 = Turbine::new(m.eta_hpt, None).apply(&s41, &rgas, dh_hpt_reb);
    let s45 = mix(&s44, mdot_blade, s3.tt, s3.pt, &rgas);               // <- MIXER 2 (blade)
    let dh_lpt_reb = (rgas.h_c(s25.tt) - rgas.h_c(s2.tt)) * s2.mdot / (base.eta_m * s45.mdot);
    let s5 = Turbine::new(m.eta_lpt, None).apply(&s45, &rgas, dh_lpt_reb);
    let exit = Nozzle::convergent(flight.p0, base.pi_n).try_apply(&s5, &rgas)?;
    if exit.p9 <= flight.p0 + 1e-6
        && crate::steady_unchoked::refuse_unchoked(base, tt4, (1.0 + s45.far) * exit.v9 - v0)? {
        return Err(Abort(format!(
            "rung-91 cooled match at Tt4={tt4:.0}, vane={e1:.3}, blade={e2:.3}, M0={:.2}: nozzle \
             UNCHOKED -- OUT OF SCOPE (docs/rung38-spec.md 'Scope').", flight.m0)));
    }

    let stations = vec![
        ("0", state0), ("2", s2), ("25", s25), ("3", s3), ("4", s4), ("41", s41), ("44", s44),
        ("45", s45), ("5", s5), ("9", exit.state),
    ];
    let perf = try_score_nozzle(&stations, s45.far, &rgas, v0, exit.t9, exit.v9, exit.p9,
                                flight.p0, rgas.hpr())?;
    // (7) THRUST per inlet air: the nozzle passes the burner air AND the cooling air.
    let mdot_nozzle_air = mdot_burner + mdot_vane + mdot_blade;
    let nozzle_share = mdot_nozzle_air / mdot_air;
    let st_inlet = nozzle_share * perf.specific_thrust - (1.0 - nozzle_share) * v0;
    let thrust = mdot_air * st_inlet;
    let base_result = TwoSpoolResult {
        stations, performance: perf, v0, v9: exit.v9, m9: exit.m9, t9: exit.t9, p9: exit.p9,
        thrust, tt4, m0: flight.m0, pi_lpc: m.c.pi_lpc, pi_hpc: m.c.pi_hpc,
        tau_lpc: s25.tt / s2.tt, tau_hpc: s3.tt / s25.tt,
        tau_hpt: m.c.tau_hpt, pi_hpt: m.c.pi_hpt, tau_lpt: m.c.tau_lpt, pi_lpt: m.c.pi_lpt,
        mdot_air, mdot_ratio: mdot_air / base.mdot_air_design,
    };
    Ok(CooledResult {
        base: TwoSpoolMapResult {
            base: base_result,
            eta_lpc: m.eta_lpc, eta_hpc: m.eta_hpc, eta_hpt: m.eta_hpt, eta_lpt: m.eta_lpt,
            n_lp: m.n_l, n_hp: m.n_h, n_lp_ratio: m.nl, n_hp_ratio: m.nh, slip: m.slip,
            phi_lp: m.phi_l, phi_hp: m.phi_h, nu_hpt: m.nu_hpt, nu_lpt: m.nu_lpt,
        },
        booking: Some(CoolingBooking {
            bleed: b, customer: c, vane: e1, blade: e2, mdot_hpc, mdot_burner, mdot_vane,
            mdot_blade, mdot_nozzle_air, tt41: s41.tt, tt44: s44.tt, tt45: s45.tt, st_inlet,
            tsfc_inlet: (mdot_burner * s4.far / mdot_air) / st_inlet,
        }),
    })
}

/// RUNG 91. Two-spool map matching with turbine cooling (off the design engine), beside rung 90's
/// customer bleed and rung 42's interstage valve.
///
/// ```text
/// let de = build_two_spool_turbojet(..).with_turbine_cooling(TurbineCooling::new(0.05, 0.03));
/// let m = TurbineCoolingMatcher::new(de, flight, 1.0, map_lp, map_hp, 0.0);
/// let od = m.match_point(&flight, 1200.0);
/// ```
pub struct TurbineCoolingMatcher {
    /// The rung-39 core carrying [`R91`]; rung 41's surge readers reach rung 91's body through it.
    pub core: TwoSpoolMapCore,
}

impl TurbineCoolingMatcher {
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: crate::map::ComponentMap, map_hp: crate::map::ComponentMap, bleed: f64,
    ) -> Self {
        let mut core = TwoSpoolMapCore::with_hooks(
            design_engine, flight_design, mdot_design, map_lp, map_hp, &R91);
        assert!((0.0..0.5).contains(&bleed), "rung-42 bleed fraction must be in [0, 0.5)");
        core.bleed = bleed;
        TurbineCoolingMatcher { core }
    }

    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> CooledResult {
        self.try_match_point(flight, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    pub fn try_match_point(
        &self, flight: &FlightCondition, tt4: f64,
    ) -> Result<CooledResult, Abort> {
        try_match_cooled(&self.core, flight, tt4)
    }
}

// =========================================================================================
// TRANSIENT — rung 62's four cells (as rung 90 carried them) with the cooling
// =========================================================================================

fn cooling_of(t: &TwoSpoolTransientCore) -> TurbineCooling {
    t.inner.base.cooling.expect("rung-91 transient body reached with no turbine cooling")
}

fn customer_of(t: &TwoSpoolTransientCore) -> Option<CustomerBleed> {
    t.inner.base.customer.filter(|c| c.is_active())
}

/// RUNG 91's `_close`. Reached from [`crate::bleed_transient::r62_try_close`] whenever cooling is
/// ACTIVE (at any `b`, with or without a customer bleed). The closure itself sees the cooling only
/// as burner air the A4 choke does not pass: `Tt3` comes from the maps, before any turbine.
pub fn r91_try_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    bump(&CLOSE_R91);
    let b = t.b_of(nu_lp, Some(tt2));
    t.arm(nu_lp, nu_hp, tt2);
    let cool = cooling_of(t);
    let eps = cool.total();
    let cb = customer_of(t);
    let c = &t.inner;
    let mdot_d = c.base.mdot_air_design;
    let gas = c.gas();
    let n_lp = nu_lp * powp(c.tt2_d / tt2, 0.5);
    let (h2, pr2) = (gas.h_c(tt2), gas.pr_c(tt2));

    let ev = |m_lp: f64| -> Result<CloseState, Abort> {
        let phi_lp = m_lp / n_lp;
        let tau_lpc = 1.0 + (c.tau_lpc_d - 1.0) * c.map_lp().psi(phi_lp) * n_lp * n_lp;
        let tt25 = tt2 * tau_lpc;
        let eta_lpc = c.map_lp().eta_c_at(c.base.eta_lpc, phi_lp, n_lp);
        let h25 = gas.h_c(tt25);
        let pi_lpc = gas.pr_c(gas.t_from_h_c(h2 + eta_lpc * (h25 - h2))) / pr2;
        let pt25 = pi_lpc * pt2;
        let mdot_face_trial = m_lp * c.mcorr_lp_d * pt2 / powp(tt2, 0.5);
        let mdot_hpc = (1.0 - b) * mdot_face_trial;

        let m_hp = (mdot_hpc * powp(tt25, 0.5) / pt25) / c.mcorr_hp_d;
        let n_hp = nu_hp * powp(c.tt25_d / tt25, 0.5);
        let phi_hp = m_hp / n_hp;
        let tau_hpc = 1.0 + (c.tau_hpc_d - 1.0) * c.map_hp().psi(phi_hp) * n_hp * n_hp;
        let tt3 = tt25 * tau_hpc;
        let eta_hpc = c.map_hp().eta_c_at(c.base.eta_hpc, phi_hp, n_hp);
        let h3 = gas.h_c(tt3);
        let pi_hpc = gas.pr_c(gas.t_from_h_c(h25 + eta_hpc * (h3 - h25))) / gas.pr_c(tt25);
        let pt4 = c.base.pi_b * pi_hpc * pt25;

        let f = c.base.try_solve_f(tt3, pt4, tt4)?;
        let wgas = c.base.try_working_gas(f, tt4, pt4)?;
        let wg = wgas.as_ref().unwrap_or(gas);
        let mdot4 = c.base.a4 * pt4 * try_choked_mfp(wg, tt4, f)? / powp(tt4, 0.5);
        // The BURNER air the A4 choke imposes; the HPC air that implies (customer and cooling
        // added back); the face air that implies.
        let mdot_imp = mdot4 / (1.0 + f);
        let hpc_imp = hpc_from_burner(cb, eps, mdot_imp, mdot_d);
        let share = customer_raw(cb, hpc_imp, mdot_d);
        let face_imp = hpc_imp / (1.0 - b);
        let m_imp = (face_imp * powp(tt2, 0.5) / pt2) / c.mcorr_lp_d;
        Ok(CloseState {
            m_lp, m_imp, m_hp, phi_lp, phi_hp, tt2, n_lp, n_hp, tau_lpc, tau_hpc, tt25, tt3,
            pi_lpc, pi_hpc, pt4, f, wgas, eta_lpc, eta_hpc, mdot_air: mdot_imp, mdot4,
            bleed: Some(b), mdot_face: Some(face_imp), customer: Some(share),
            cooling: Some(cool), offtake: None,
        })
    };

    let g = |m: f64| -> Result<f64, Abort> {
        let r = m - ev(m)?.m_imp;
        if r.is_nan() {
            return Err(Abort(format!(
                "off-map compressor trial at m_lp={m:.4}: the loading law has gone non-physical \
                 (Tt3 < 0 => a complex pressure ratio).")));
        }
        Ok(r)
    };
    let wall_map = c.map_lp().phi_max(0.1) * n_lp;
    let hi = 2.5f64.min(wall_map);
    let ghi = g(hi)?;
    let (mut lo, mut glo, mut m) = (None, 0.0f64, 0.02f64);
    while m < hi {
        match g(m) {
            Ok(v) => { glo = v; lo = Some(m); break; }
            Err(_) => { m += 0.02; }
        }
    }
    let Some(lo) = lo.filter(|_| glo < 0.0 && 0.0 < ghi) else {
        return Err(Abort(format!(
            "rung-91 two-shaft closure with turbine cooling does not bracket at \
             nu=({nu_lp:.4},{nu_hp:.4}), Tt4={tt4:.0}, b={b:.4} — off the modeled speed-line \
             region.")));
    };
    let root = try_illinois(g, lo, hi, glo, ghi, TwoSpoolTransientCore::CLOSE_TOL,
                            ILLINOIS_MAXIT)?;
    let out = ev(root)?;
    let hpc = (1.0 - b) * out.mdot_face.expect("set above");
    check_root(out.customer.expect("set above"), eps, hpc)?;
    Ok(out)
}

/// RUNG 91's `_close_fuel`. The burner never sees the customer air or the cooling air, so
/// `f = ṁ_fuel / burner air` with the burner air `(1−c−ε)` of the HPC air (or the HPC air less
/// the cooling and the amount), and the rich/lean bracket walls carry the same factor.
pub fn r91_try_close_fuel(
    ft: &FuelTransientCore, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
) -> Result<FuelCloseState, Abort> {
    bump(&CLOSE_FUEL_R91);
    let b = ft.inner.b_of(nu_lp, Some(tt2));
    ft.inner.arm(nu_lp, nu_hp, tt2);
    let cool = cooling_of(&ft.inner);
    let eps = cool.total();
    let cb = customer_of(&ft.inner);
    let c = &ft.inner.inner;
    let mdot_d = c.base.mdot_air_design;
    let gas = c.gas();
    let n_lp = nu_lp * powp(c.tt2_d / tt2, 0.5);
    let (h2, pr2) = (gas.h_c(tt2), gas.pr_c(tt2));

    let ev = |m_lp: f64| -> Result<FuelCloseState, Abort> {
        let phi_lp = m_lp / n_lp;
        let tau_lpc = 1.0 + (c.tau_lpc_d - 1.0) * c.map_lp().psi(phi_lp) * n_lp * n_lp;
        let tt25 = tt2 * tau_lpc;
        let eta_lpc = c.map_lp().eta_c_at(c.base.eta_lpc, phi_lp, n_lp);
        let h25 = gas.h_c(tt25);
        let pi_lpc = gas.pr_c(gas.t_from_h_c(h2 + eta_lpc * (h25 - h2))) / pr2;
        let pt25 = pi_lpc * pt2;
        let mdot_face_trial = m_lp * c.mcorr_lp_d * pt2 / powp(tt2, 0.5);
        let mdot_hpc = (1.0 - b) * mdot_face_trial;

        let m_hp = (mdot_hpc * powp(tt25, 0.5) / pt25) / c.mcorr_hp_d;
        let n_hp = nu_hp * powp(c.tt25_d / tt25, 0.5);
        let phi_hp = m_hp / n_hp;
        let tau_hpc = 1.0 + (c.tau_hpc_d - 1.0) * c.map_hp().psi(phi_hp) * n_hp * n_hp;
        let tt3 = tt25 * tau_hpc;
        let eta_hpc = c.map_hp().eta_c_at(c.base.eta_hpc, phi_hp, n_hp);
        let h3 = gas.h_c(tt3);
        let pi_hpc = gas.pr_c(gas.t_from_h_c(h25 + eta_hpc * (h3 - h25))) / gas.pr_c(tt25);
        let pt4 = c.base.pi_b * pi_hpc * pt25;

        // THE CUSTOMER AND COOLING AIR LEAVE BEFORE THE BURNER.
        let share = customer_raw(cb, mdot_hpc, mdot_d);
        let burner_trial = (1.0 - share - eps) * mdot_hpc;
        if !(burner_trial > 0.0) {
            return Err(Abort(format!(
                "rung-91 fuel-closure trial at m_lp={m_lp:.4}: the customer and cooling air take \
                 all the HP compressor's air.")));
        }
        let f = mdot_fuel / burner_trial;
        let tt4 = ft.try_tt4_from_f(tt3, f)?;
        let wgas = c.base.try_working_gas(f, tt4, pt4)?;
        let wg = wgas.as_ref().unwrap_or(gas);
        let mdot4 = c.base.a4 * pt4 * try_choked_mfp(wg, tt4, f)? / powp(tt4, 0.5);
        let mdot_imp = mdot4 / (1.0 + f);
        let face_imp = hpc_from_burner(cb, eps, mdot_imp, mdot_d) / (1.0 - b);
        let m_imp = (face_imp * powp(tt2, 0.5) / pt2) / c.mcorr_lp_d;
        Ok(FuelCloseState {
            base: CloseState {
                m_lp, m_imp, m_hp, phi_lp, phi_hp, tt2, n_lp, n_hp, tau_lpc, tau_hpc, tt25, tt3,
                pi_lpc, pi_hpc, pt4, f, wgas, eta_lpc, eta_hpc, mdot_air: mdot_imp, mdot4,
                bleed: Some(b), mdot_face: Some(face_imp), customer: Some(share),
                cooling: Some(cool), offtake: None,
            },
            tt4,
            mdot_air_face: mdot_face_trial,
        })
    };

    let g = |m: f64| -> Result<f64, Abort> {
        let r = m - ev(m)?.base.m_imp;
        if r.is_nan() {
            return Err(Abort(format!(
                "off-map compressor trial at m_lp={m:.4}: the loading law has gone non-physical \
                 (Tt3 < 0 => a complex pressure ratio).")));
        }
        Ok(r)
    };

    // The f caps are BURNER-referenced, so the FACE-flow walls they imply carry the customer and
    // the cooling extraction as well as rung 62's 1/(1-b).
    let face_at = |f_wall: f64| hpc_from_burner(cb, eps, mdot_fuel / f_wall, mdot_d) / (1.0 - b);
    let lo0 = face_at(FuelTransientCore::F_CAP) * powp(tt2, 0.5) / (c.mcorr_lp_d * pt2);
    let hi0 = face_at(FuelTransientCore::F_FLOOR) * powp(tt2, 0.5) / (c.mcorr_lp_d * pt2);
    let wall_map = c.map_lp().phi_max(0.1) * n_lp;
    let cap = 2.5f64.min(wall_map).min(hi0);
    let mut m = lo0.max(0.02);
    let (mut lo, mut glo, mut hi, mut ghi) = (None, 0.0f64, None, 0.0f64);
    while m < cap {
        let gm = match g(m) {
            Ok(v) => v,
            Err(_) => { m += FuelTransientCore::MARCH_IN_STEP; continue; }
        };
        if gm < 0.0 {
            lo = Some(m);
            glo = gm;
        } else if lo.is_some() {
            hi = Some(m);
            ghi = gm;
            break;
        }
        m += FuelTransientCore::MARCH_IN_STEP;
    }
    let (Some(lo), Some(hi)) = (lo, hi) else {
        return Err(Abort(format!(
            "rung-91 fuel closure does not bracket (turbine cooling on) at \
             nu=({nu_lp:.4},{nu_hp:.4}), mdot_fuel={mdot_fuel:.5}, b={b:.4} — off the modeled \
             speed-line region.")));
    };
    let root = try_illinois(g, lo, hi, glo, ghi, TwoSpoolTransientCore::CLOSE_TOL,
                            ILLINOIS_MAXIT)?;
    let out = ev(root)?;
    let hpc = (1.0 - b) * out.base.mdot_face.expect("set above");
    check_root(out.base.customer.expect("set above"), eps, hpc)?;
    Ok(out)
}

/// The two turbines at a closure, with both mixers. Per unit HPC air the burner share is the
/// closure's own `burner / hpc`.
struct Turbines {
    hpt: CooledHpt,
    pi_lpt: f64,
    tau_lpt: f64,
    tt5: f64,
    eta_hpt: f64,
    eta_lpt: f64,
    nu_hpt: f64,
    nu_lpt: f64,
}

#[allow(clippy::too_many_arguments)]
fn turbines(
    t: &TwoSpoolTransientCore, c: &CloseState, wgas: &Gas, cool: TurbineCooling, nu_lp: f64,
    nu_hp: f64, tt4: f64, p0: f64,
) -> Result<Turbines, Abort> {
    let core = &t.inner;
    let f = c.f;
    let hpc = (1.0 - c.bleed.unwrap_or(0.0)) * c.mdot_face.expect("a rung-91 closure sets it");
    let burner = c.mdot_air / hpc;
    // Mixer 1 is upstream of the rotor, so the rotor's corrected speed is known before its pin.
    let fuel = burner * f;
    let tt41 = mixed_tt(wgas, burner * (1.0 + f), tt4, f, cool.vane, c.tt3,
                        fuel / (burner + cool.vane));
    let nu_hpt = nu_hp * powp(core.tt4_d / tt41, 0.5);
    let eta_hpt = core.map_hp().eta_t_at(core.base.eta_hpt, nu_hpt);
    let hpt = try_solve_cooled_hpt(&core.base, wgas, tt4, f, burner, cool.vane, cool.blade, c.tt3,
                                   eta_hpt)?;
    let nu_lpt = nu_lp * powp(core.tt45_d / hpt.tt45, 0.5);
    let eta_lpt = core.map_lp().eta_t_at(core.base.eta_lpt, nu_lpt);
    // RUNG 97: the pin while the nozzle stays choked, else the subsonic root — at the cooled `Tt45`/`f45`.
    let (pi_lpt, tau_lpt, tt5) = crate::unchoked_lp::try_lp_turbine(
        &core.base, wgas, hpt.tt45, hpt.f45, eta_lpt, hpt.pi_hpt * c.pt4, p0)?;
    Ok(Turbines { hpt, pi_lpt, tau_lpt, tt5, eta_hpt, eta_lpt, nu_hpt, nu_lpt })
}

/// The shaft powers in rung 62's bookkeeping (each specific power per unit of that SHAFT'S
/// compressor air). The ROTOR passes `G + ε₁` per HPC air; the LPT `G + ε`.
/// Returns `(pt_lp, pt_hp, pc_lp, pc_hp, Phi_L, Phi_H)`.
fn shaft_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, wgas: &Gas, cool: TurbineCooling, tb: &Turbines,
    nu_lp: f64, nu_hp: f64,
) -> (f64, f64, f64, f64, f64, f64) {
    let core = &t.inner;
    let f = c.f;
    let b = c.bleed.unwrap_or(0.0);
    let face = c.mdot_face.expect("a rung-91 closure always sets mdot_face");
    let hpc = (1.0 - b) * face;
    let g = (c.mdot_air / hpc) * (1.0 + f);       // burner gas per HPC air
    let h = &tb.hpt;
    let pt_hp = core.base.eta_m * (g + cool.vane)
        * (wgas.h_t(h.tt41, h.f41) - wgas.h_t(h.tt44, h.f41));
    let pt_lp = (hpc / face) * core.base.eta_m * (g + cool.total())
        * (wgas.h_t(h.tt45, h.f45) - wgas.h_t(tb.tt5, h.f45));
    let pc_hp = wgas.h_c(c.tt3) - wgas.h_c(c.tt25);
    let pc_lp = wgas.h_c(c.tt25) - wgas.h_c(c.tt2);
    let phi_hp_dot = (hpc * (pt_hp - pc_hp)) / (t.p_ref_hp * nu_hp);
    let phi_lp_dot = (face * (pt_lp - pc_lp)) / (t.p_ref_lp * nu_lp);
    (pt_lp, pt_hp, pc_lp, pc_hp, phi_lp_dot, phi_hp_dot)
}

/// RUNG 91's `_powers`. Dispatched on the CLOSURE's own `cooling` field (rung 62 § 5.21 (v)).
pub fn r91_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, tt4: f64,
) -> Result<(f64, f64), Abort> {
    bump(&POWERS_R91);
    let cool = c.cooling.expect("a rung-91 closure carries its cooling");
    let wgas = c.gas(&t.inner);
    let tb = turbines(t, c, wgas, cool, nu_lp, nu_hp, tt4, flight.p0)?;
    let (_, _, _, _, pl, ph) = shaft_powers(t, c, wgas, cool, &tb, nu_lp, nu_hp);
    Ok((pl, ph))
}

/// RUNG 91's turbine / power / thrust tail. `sp_thrust` is per NOZZLE air (burner + cooling air,
/// at the diluted `f45`); `sp_thrust_inlet` per inlet air.
pub fn r91_try_instant_tail(
    t: &TwoSpoolTransientCore, flight: &FlightCondition, c: &CloseState, nu_lp: f64, nu_hp: f64,
    tt4: f64, v0: f64,
) -> Result<Instant2, Abort> {
    bump(&TAIL_R91);
    let core = &t.inner;
    let cool = c.cooling.expect("a rung-91 closure carries its cooling");
    let wgas = c.gas(core);
    let tb = turbines(t, c, wgas, cool, nu_lp, nu_hp, tt4, flight.p0)?;
    let (pt_lp, pt_hp, pc_lp, pc_hp, phi_lp_dot, phi_hp_dot) =
        shaft_powers(t, c, wgas, cool, &tb, nu_lp, nu_hp);

    let face = c.mdot_face.expect("a rung-91 closure always sets mdot_face");
    let hpc = (1.0 - c.bleed.unwrap_or(0.0)) * face;
    let nozzle_air = c.mdot_air + cool.total() * hpc;
    let f45 = tb.hpt.f45;
    let s5 = FlowState { tt: tb.tt5, pt: tb.pi_lpt * tb.hpt.pi_hpt * c.pt4, mdot: nozzle_air,
                         far: f45 };
    // RUNG 102: behind a lit afterburner the exit is station 7's, on the total fuel (the shipped nozzle otherwise).
    let (exit, f9) = crate::afterburner::lp_exit(&core.base, wgas, &s5, flight.p0)?;
    let press = (1.0 + f9) * wgas.r_t_at(f9) * exit.t9 * (1.0 - flight.p0 / exit.p9) / exit.v9;
    let sp_thrust = (1.0 + f9) * exit.v9 - v0 + press;
    let share = nozzle_air / face;
    Ok(Instant2 {
        close: c.clone(),
        nu_lp, nu_hp, tt4, slip: nu_lp / nu_hp, phi_lp_dot, phi_hp_dot, pt_lp, pt_hp, pc_lp, pc_hp,
        tt45: tb.hpt.tt45, tt5: tb.tt5, tau_hpt: tb.hpt.tau_rotor, tau_lpt: tb.tau_lpt,
        pi_hpt: tb.hpt.pi_hpt, pi_lpt: tb.pi_lpt, eta_hpt: tb.eta_hpt, eta_lpt: tb.eta_lpt,
        nu_hpt: tb.nu_hpt, nu_lpt: tb.nu_lpt, sp_thrust,
        sp_thrust_inlet: Some(share * sp_thrust - (1.0 - share) * v0),
        m9: exit.m9,
        branch: if exit.p9 > flight.p0 + 1e-6 { Branch::Choked } else { Branch::Subsonic },
    })
}
