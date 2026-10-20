//! RUNG 92 — SHAFT-POWER OFFTAKE: the aircraft's generator load on a shaft.
//!
//! Anchor (the derivation, the linear model and every pre-registered bar, committed before any of
//! this code existed): `docs/plans/rung92-anchor-shaft-offtake.md`. Spec: `docs/rung92-spec.md`.
//!
//! A generator (and pumps) draw power off the HP shaft, the LP shaft, or both, after the bearings
//! (`η_m` already taken). Each shaft's demand is a [`Demand`]: a SHARE `s` of that shaft's turbine
//! shaft power, or a fixed POWER `w · P_ref` with `P_ref` the design HP compressor power — ONE unit
//! for both shafts, so an HP and an LP load of the same `w` are the same kilowatts.
//!
//! # What it touches — and what it does not
//!
//! The load moves no gas. Both turbine pins, the face referrals, the burner's air and the thrust
//! bookkeeping are untouched; only the two shaft balances change:
//! `h(Tt3) − h(Tt25) = η_m (gas per HPC air) Δh_HPT − q_H` (likewise the LP, per face air), with
//! `q = P_x / ṁ_compressor`. On the transient plant the CLOSURE never sees a shaft balance, so rung
//! 62's closure cells run unchanged and only MARK the result ([`r92_try_close`]); the power and
//! tail cells WRAP rung 62's ([`r92_powers`], [`r92_try_instant_tail`]): the unloaded shaft powers,
//! less the load.
//!
//! # Two exact identities (flat maps, perfect gas) — gated, not assumed
//!
//! An LP-shaft share `s` IS rung 42's interstage bleed `b = s` minus the dumped air; an HP plus an
//! LP share `s` IS rung 90's customer bleed `c = s` minus the dumped air. A bleed is a power
//! offtake plus air thrown away (anchor § 1; `tests/rung92.rs`).
//!
//! # Reduce — by DISPATCH
//!
//! An inactive load (`None`, zero demands) never enters this module's arithmetic: the steady cell
//! hands to rung 91's body (hence 90's / 42's / 39's); rung 62's transient cells run their own
//! bodies unmarked. Every body that cannot model a load refuses an ACTIVE one
//! ([`crate::two_spool::TwoSpoolCore::refuse_offtake`]) — rung 40's power and tail only when the
//! closure is UNMARKED, i.e. did not come through rung 62's dispatch.
//!
//! # The steady body is a COPY
//!
//! Rung 91's cascade and match, copied with the load inserted in both shaft balances
//! ([[rust-port-copy-vs-rederivation]]); with no cooling the mixers add nothing and the HP pin is
//! rung 91's bisection on the uncooled throat ratio. Rung 90's efficiency loops are CALLED, as rung
//! 91 calls them.

use std::cell::Cell;

use crate::bleed_transient::{r62_close_body, r62_close_fuel_body, r62_powers_body, r62_tail_body};
use crate::components::{try_choked_mfp, ram_recovery, Burner, Compressor, Inlet, Nozzle, Turbine};
use crate::customer_bleed::{hp_eta_loop_customer, lp_eta_loop_customer, CustomerBleed};
use crate::engine::{FlightCondition, Performance};
use crate::fuel_transient::{FuelCloseState, FuelTransientCore};
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::turbine_cooling::{check_root, customer_raw, hpc_from_burner, mix,
                             CooledHpt, TurbineCooling};
use crate::two_spool::{
    Cascade, CascadeMap, TwoSpoolCore, TwoSpoolEngine, TwoSpoolHooks, TwoSpoolMapCore,
    TwoSpoolMapResult, TwoSpoolResult, R39,
};
use crate::two_spool_transient::{CloseState, Instant2, TwoSpoolTransientCore};

// =========================================================================================
// THE LOAD
// =========================================================================================

/// One shaft's demand.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Demand {
    /// A fixed SHARE `s` of that shaft's turbine shaft power: the compressor gets `(1−s)` of it.
    Share(f64),
    /// A fixed POWER `w · P_ref`, `P_ref` the design HP compressor power (the same unit on both
    /// shafts) — as a generator draws it, so a LARGER share of a smaller turbine at low power.
    Power(f64),
}

impl Demand {
    /// The largest share either demand may be, and the largest share a power load may reach at a
    /// converged point (refused there in words).
    pub const SHARE_MAX: f64 = 0.5;

    /// A fixed share, range-checked (`[0, 0.5)`).
    pub fn share(s: f64) -> Self {
        assert!((0.0..Self::SHARE_MAX).contains(&s),
                "rung-92 offtake share must be in [0, 0.5) of the shaft's turbine power: got {s}");
        Demand::Share(s)
    }

    /// A fixed power, `w` × the design HP compressor power, range-checked (`[0, 0.5)`).
    pub fn power(w: f64) -> Self {
        assert!((0.0..Self::SHARE_MAX).contains(&w),
                "rung-92 offtake power must be in [0, 0.5) of the design HP compressor power: got \
                 {w}");
        Demand::Power(w)
    }

    pub fn is_active(&self) -> bool {
        match *self {
            Demand::Share(s) => s != 0.0,
            Demand::Power(w) => w != 0.0,
        }
    }

    /// The load in watts on a shaft whose turbine delivers `p_turbine` W (after the bearings).
    pub fn watts(&self, p_turbine: f64, p_ref: f64) -> f64 {
        match *self {
            Demand::Share(s) => s * p_turbine,
            Demand::Power(w) => w * p_ref,
        }
    }
}

/// The aircraft's shaft loads: HP, LP, or both (the sandbox offers one at a time).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShaftOfftake {
    pub hp: Option<Demand>,
    pub lp: Option<Demand>,
}

impl ShaftOfftake {
    pub fn hp(d: Demand) -> Self { ShaftOfftake { hp: Some(d), lp: None } }
    pub fn lp(d: Demand) -> Self { ShaftOfftake { hp: None, lp: Some(d) } }
    pub fn both(hp: Demand, lp: Demand) -> Self { ShaftOfftake { hp: Some(hp), lp: Some(lp) } }

    /// Does either shaft carry a load? Zero demands on both are INACTIVE, and dispatch away.
    pub fn is_active(&self) -> bool {
        self.hp.is_some_and(|d| d.is_active()) || self.lp.is_some_and(|d| d.is_active())
    }
}

fn watts(d: Option<Demand>, p_turbine: f64, p_ref: f64) -> f64 {
    d.map_or(0.0, |d| d.watts(p_turbine, p_ref))
}

/// The design HP compressor power — the power unit of [`Demand::Power`]. The SAME arithmetic as
/// the transient's `p_ref_hp` (rung 40's constructor), so the steady and transient loads agree.
pub fn p_ref(base: &TwoSpoolCore) -> f64 {
    let gas = base.gas();
    base.mdot_air_design
        * (gas.h_c(base.reference.station("3").tt) - gas.h_c(base.reference.station("25").tt))
}

/// The bound at a converged point: neither load takes half its turbine's work.
pub fn check_shares(share_hp: f64, share_lp: f64) -> Result<(), Abort> {
    for (name, s) in [("HP", share_hp), ("LP", share_lp)] {
        if !(s < Demand::SHARE_MAX) {
            return Err(Abort(format!(
                "rung-92: the generator takes {:.1} % of the {name} turbine's work here: half or \
                 more, so the {name} compressor is starved — the aircraft is asking for more power \
                 than the engine can give at this throttle.", 100.0 * s)));
        }
    }
    Ok(())
}

// =========================================================================================
// DISPATCH COUNTERS
// =========================================================================================

thread_local! {
    static STEADY_ACTIVE: Cell<u64> = const { Cell::new(0) };
    static STEADY_INACTIVE: Cell<u64> = const { Cell::new(0) };
    static CLOSE_R92: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_R92: Cell<u64> = const { Cell::new(0) };
    static POWERS_R92: Cell<u64> = const { Cell::new(0) };
    static TAIL_R92: Cell<u64> = const { Cell::new(0) };
}

fn bump(k: &'static std::thread::LocalKey<Cell<u64>>) {
    k.with(|x| x.set(x.get() + 1));
}

/// How many times each rung-92 body ran on this thread — `(steady active, steady inactive,
/// close, close_fuel, powers, tail)`. Reset by [`reset_counts`].
pub fn counts() -> [u64; 6] {
    [&STEADY_ACTIVE, &STEADY_INACTIVE, &CLOSE_R92, &CLOSE_FUEL_R92, &POWERS_R92, &TAIL_R92]
        .map(|k| k.with(|x| x.get()))
}

pub fn reset_counts() {
    for k in [&STEADY_ACTIVE, &STEADY_INACTIVE, &CLOSE_R92, &CLOSE_FUEL_R92, &POWERS_R92,
              &TAIL_R92] {
        k.with(|x| x.set(0));
    }
}

// =========================================================================================
// STEADY — THE HOOK TABLE AND THE CASCADE
// =========================================================================================

/// RUNG 92's steady table. Only `try_match_point` changes (rung 90's reason).
pub const R92: TwoSpoolHooks = TwoSpoolHooks {
    try_match_point: r92_try_match_point,
    hp_eta_loop: R39.hp_eta_loop,
    lp_eta_loop: R39.lp_eta_loop,
};

fn r92_try_match_point(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<TwoSpoolMapResult, Abort> {
    try_match_loaded(core, flight, tt4).map(|r| r.base)
}

/// A shaft's load inside the cascade, per unit of that shaft's COMPRESSOR air.
#[derive(Clone, Copy, Debug)]
enum PerAir {
    /// A share of the turbine's work per compressor air (known only inside the cascade).
    Share(f64),
    /// J per kg of compressor air — a power load over the joint loop's current flow.
    Specific(f64),
}

impl PerAir {
    fn of(d: Option<Demand>, p_ref: f64, mdot_compressor: f64) -> Option<PerAir> {
        match d {
            None => None,
            Some(Demand::Share(s)) => Some(PerAir::Share(s)),
            Some(Demand::Power(w)) => Some(PerAir::Specific(w * p_ref / mdot_compressor)),
        }
    }
}

/// The load per compressor air on a shaft whose turbine gives `dh` per compressor air.
fn q_of(p: Option<PerAir>, dh: f64) -> f64 {
    match p {
        None => 0.0,
        Some(PerAir::Share(s)) => s * dh,
        Some(PerAir::Specific(q)) => q,
    }
}

/// The loaded cascade's answer: rung 91's, plus each turbine's shaft work per compressor air and
/// each load per compressor air.
#[derive(Clone, Debug)]
pub struct LoadedCascade {
    pub map: CascadeMap,
    pub hpt: CooledHpt,
    /// `η_m × gas × Δh` of each turbine, per kg of ITS compressor's air (HPC air, face air).
    pub dh_hpt: f64,
    pub dh_lpt: f64,
    /// The loads, per kg of the same air.
    pub q_hp: f64,
    pub q_lp: f64,
}

/// A load that would take ALL of a turbine's work leaves its compressor nothing to do — no trial
/// past it can lead to a root, so this one is refused at the trial, in words (the half bound is
/// [`check_shares`]'s, at the root). Measured: a POWER load reaches it by running away — less
/// work, less air, a larger share — past a fold where the steady point ceases to exist.
fn all_of(q: f64, dh: f64, shaft: &str) -> Result<(), Abort> {
    if q >= dh {
        return Err(Abort(format!(
            "rung-92: no steady point — carrying this load, the {shaft} turbine's work falls below \
             the load itself ({q:.0} of {dh:.0} J per kg of compressor air), and less work means \
             less air: the aircraft is asking for more power than the engine can give at this \
             throttle.")));
    }
    Ok(())
}

/// Rung 91's cascade with the loads in both shaft balances (the `Tt3` fixed point kept).
#[allow(clippy::too_many_arguments)]
fn try_cascade_loaded(
    core: &TwoSpoolMapCore, wgas: &Gas, tt2: f64, tt4: f64, f: f64, c: f64, cool: TurbineCooling,
    tt3_seed: f64, load_hp: Option<PerAir>, load_lp: Option<PerAir>,
) -> Result<LoadedCascade, Abort> {
    let b = core.bleed;
    let base = &core.base;
    let (e1, e2) = (cool.vane, cool.blade);
    let eps = e1 + e2;
    let burner = 1.0 - c - eps;
    let mfp4_pin = try_choked_mfp(wgas, tt4, f)?;
    let (mut eta_hpt, mut eta_lpt, mut tt3) = (base.eta_hpt, base.eta_lpt, tt3_seed);
    for _ in 0..TwoSpoolCore::MAX {
        let (hpt, (pi_lpt, tau_lpt, tt5)) = crate::turbine_cooling::try_cooled_pair(
            base, wgas, tt4, f, burner, e1, e2, tt3, eta_hpt,
            |tt45, f45, pi_hpt| Ok((crate::steady_unchoked::try_lp_steady(base, wgas, tt45, f45, eta_lpt, pi_hpt)?,
                                    eta_lpt)))?;
        // RUNG 105: the vane's law reaches (†)/(‡) as the A4 flow per pinned area (`MFP*` itself on the pin).
        let mfp4 = mfp4_pin * (hpt.a4 / base.a4);

        // The LP balance, per FACE air: the LPT passes (1-b)(G + eps) of gas; the LP load comes off.
        let g = burner * (1.0 + f);
        let dh_lpt = base.eta_m * (1.0 - b) * (g + eps)
            * (wgas.h_t(hpt.tt45, hpt.f45) - wgas.h_t(tt5, hpt.f45));
        let q_lp = q_of(load_lp, dh_lpt);
        all_of(q_lp, dh_lpt, "LP")?;
        let tt25 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_lpt - q_lp);
        // The HP balance, per HPC air: the ROTOR passes G + eps1; the HP load comes off.
        let dh_hpt = base.eta_m * (g + e1)
            * (wgas.h_t(hpt.tt41, hpt.f41) - wgas.h_t(hpt.tt44, hpt.f41));
        let q_hp = q_of(load_hp, dh_hpt);
        all_of(q_hp, dh_hpt, "HP")?;
        let tt3_new = wgas.t_from_h_c(wgas.h_c(tt25) + dh_hpt - q_hp);

        let hp = hp_eta_loop_customer(wgas, tt4, f, tt25, tt3_new, mfp4, &core.map_hp(), c + eps,
                                      base.eta_hpc, base.a4, base.pi_b, core.mcorr_hp_d,
                                      core.tau_hpc_d)?;
        let lp = lp_eta_loop_customer(wgas, tt2, tt4, f, tt25, mfp4, hp.pi, &core.map_lp(), b,
                                      c + eps, base.eta_lpc, base.a4, base.pi_b, core.mcorr_lp_d,
                                      core.tau_lpc_d)?;

        let nl = lp.n * powp(tt2 / core.tt2_d, 0.5);
        let nh = hp.n * powp(tt25 / core.tt25_d, 0.5);
        let nu_hpt = nh * powp(core.tt4_d / hpt.tt41, 0.5);
        let nu_lpt = nl * powp(core.tt45_d / hpt.tt45, 0.5);
        let map = CascadeMap {
            c: Cascade { pi_hpt: hpt.pi_hpt, tau_hpt: hpt.tau_rotor, tt45: hpt.tt45, pi_lpt,
                         tau_lpt, tt5, pi_lpc: lp.pi, tt25, pi_hpc: hp.pi, tt3: tt3_new, a4: hpt.a4 },
            eta_lpc: lp.eta, eta_hpc: hp.eta, eta_hpt, eta_lpt,
            m_l: lp.m, m_h: hp.m, n_l: lp.n, n_h: hp.n, nl, nh,
            phi_l: lp.m / lp.n, phi_h: hp.m / hp.n, nu_hpt, nu_lpt, slip: nl / nh,
        };
        let t_hpt = core.map_hp().eta_t_at(base.eta_hpt, nu_hpt);
        let t_lpt = core.map_lp().eta_t_at(base.eta_lpt, nu_lpt);
        if (t_hpt - eta_hpt).abs() <= TwoSpoolMapCore::ETA_TOL
            && (t_lpt - eta_lpt).abs() <= TwoSpoolMapCore::ETA_TOL
            && (tt3_new - tt3).abs() <= 1e-12 * tt3_new {
            return Ok(LoadedCascade { map, hpt, dh_hpt, dh_lpt, q_hp, q_lp });
        }
        eta_hpt = t_hpt;
        eta_lpt = t_lpt;
        tt3 = tt3_new;
    }
    Err(Abort(format!("rung-92 loaded cascade (Tt3 fixed point / turbine efficiencies) did not \
                       converge at Tt4={tt4}")))
}

// =========================================================================================
// STEADY — MATCH ONE POINT
// =========================================================================================

/// What the load books on a matched point — present only when it was ACTIVE.
#[derive(Clone, Copy, Debug)]
pub struct OfftakeBooking {
    /// The load on each shaft, W, and the share of that turbine's shaft power it is.
    pub power_hp: f64,
    pub power_lp: f64,
    pub share_hp: f64,
    pub share_lp: f64,
    /// Each turbine's shaft power (after the bearings), W.
    pub turbine_hp: f64,
    pub turbine_lp: f64,
    /// The interstage (rung 42), customer (rung 90) and cooling (rung 91) shares carried with it.
    pub bleed: f64,
    pub customer: f64,
    pub vane: f64,
    pub blade: f64,
    /// Air through the HPC and the burner, kg/s.
    pub mdot_hpc: f64,
    pub mdot_burner: f64,
    /// The rotor-inlet, rotor-exit and LPT-inlet temperatures (rung 91's; `Tt4`, `Tt45`, `Tt45`
    /// with no cooling).
    pub tt41: f64,
    pub tt44: f64,
    pub tt45: f64,
    /// `F / ṁ_INLET` and fuel per unit thrust per inlet air (rung 91's bookkeeping).
    pub st_inlet: f64,
    pub tsfc_inlet: f64,
}

/// A matched two-spool point with a shaft load.
#[derive(Clone, Debug)]
pub struct OfftakeResult {
    /// `base.performance` is NOZZLE-AIR-referenced; `base.thrust` is the honest one.
    pub base: TwoSpoolMapResult,
    /// `None` exactly when the load was INACTIVE and rung 91's body produced this point.
    pub booking: Option<OfftakeBooking>,
}

/// Rung 91's copy of rung 1's scoring with the fuel ratio passed explicitly (the nozzle passes
/// any coolant too, at the diluted `f45`). A copy, as rung 91's is.
#[allow(clippy::too_many_arguments)]
fn try_score_nozzle(
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

/// Match at `(flight, Tt4)` with the interstage valve at `core.bleed` and the customer bleed,
/// cooling and shaft load off the design engine. **Inactive load ⇒ rung 91's body verbatim.**
pub fn try_match_loaded(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<OfftakeResult, Abort> {
    // RUNG 103: behind the switch, a point the pinned body cannot finish is re-solved with the LP turbine free.
    crate::steady_unchoked::dispatch(&core.base, flight.p0, tt4, || loaded_body(core, flight, tt4),
                                     |x| x.base.base.thrust)
}

fn loaded_body(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<OfftakeResult, Abort> {
    if !core.base.offtake_active() {
        bump(&STEADY_INACTIVE);
        let r = crate::turbine_cooling::try_match_cooled(core, flight, tt4)?;
        return Ok(OfftakeResult { base: r.base, booking: None });
    }
    bump(&STEADY_ACTIVE);
    assert!(core.stack_lp.is_none() && core.stack_hp.is_none(),
            "rung-92's steady body lumps both compressors; rung 55's stage stacks are out of scope");
    let load = core.base.offtake.expect("active");
    let cool = core.base.cooling.unwrap_or(TurbineCooling { vane: 0.0, blade: 0.0 });
    let (e1, e2) = (cool.vane, cool.blade);
    let eps = cool.total();
    let cb = core.base.customer.filter(|c| c.is_active());
    let b = core.bleed;
    let base = &core.base;
    let mdot_d = base.mdot_air_design;
    let p_ref = p_ref(base);
    let pi_d = base.pi_d_max * ram_recovery(flight.m0);
    let (state0, _v0) = base.try_freestream_for(flight)?;
    let (tt2, pt2) = (state0.tt, pi_d * state0.pt);

    // Rung 91's joint (f, pt4, c) loop, now also carrying the compressor flows a POWER load is
    // spread over — seeded at the design flow, then from the previous pass's burner flow.
    let (mut f, mut pt4) =
        (base.f_design, base.pi_b * base.pi_hpc_design * base.pi_lpc_design * pt2);
    let mut c = match cb {
        None => 0.0,
        Some(CustomerBleed::Share(c)) => c,
        Some(CustomerBleed::Amount(w)) => w,
    };
    let (mut face, mut hpc) = (mdot_d, (1.0 - b) * mdot_d);
    let mut tt3_seed = base.reference.station("3").tt;
    let mut lc: Option<LoadedCascade> = None;
    for _ in 0..TwoSpoolCore::MAX {
        crate::steady_unchoked::track_pt4(base, pt4);
        let owned = base.try_working_gas(f, tt4, pt4)?;
        let wgas = owned.as_ref().unwrap_or(base.gas());
        let x = try_cascade_loaded(core, wgas, tt2, tt4, f, c, cool, tt3_seed,
                                   PerAir::of(load.hp, p_ref, hpc),
                                   PerAir::of(load.lp, p_ref, face))?;
        let pt4_new = base.pi_b * x.map.c.pi_hpc * x.map.c.pi_lpc * pt2;
        let f_new = base.try_solve_f(x.map.c.tt3, pt4_new, tt4)?;
        let burner = x.map.c.a4 * pt4_new * try_choked_mfp(wgas, tt4, f_new)?
            / powp(tt4, 0.5) / (1.0 + f_new);
        let c_new = match cb {
            Some(CustomerBleed::Amount(_)) => {
                customer_raw(cb, hpc_from_burner(cb, eps, burner, mdot_d), mdot_d)
            }
            _ => c,
        };
        let hpc_new = hpc_from_burner(cb, eps, burner, mdot_d);
        let face_new = hpc_new / (1.0 - b);
        let done = (f_new - f).abs() <= TwoSpoolCore::TOL * (f_new + 1e-30)
            && (pt4_new - pt4).abs() <= TwoSpoolCore::TOL * pt4_new
            && (c_new - c).abs() <= TwoSpoolCore::TOL * (c_new + 1e-30)
            && (hpc_new - hpc).abs() <= TwoSpoolCore::TOL * hpc_new;
        tt3_seed = x.map.c.tt3;
        lc = Some(x);
        f = f_new;
        pt4 = pt4_new;
        c = c_new;
        hpc = hpc_new;
        face = face_new;
        if done {
            break;
        }
    }
    let x = lc.expect("the joint loop runs at least once");
    let m = &x.map;
    if !(m.c.pi_lpc > 1.0 && m.c.pi_hpc > 1.0 && 0.0 < m.c.tau_hpt && m.c.tau_hpt < 1.0
         && 0.0 < m.c.tau_lpt && m.c.tau_lpt < 1.0) {
        return Err(Abort(format!("rung-92 loaded match unphysical at Tt4={tt4:.0}")));
    }

    let owned = base.try_working_gas(f, tt4, pt4)?;
    let wgas = owned.as_ref().unwrap_or(base.gas());
    let mdot_burner = m.c.a4 * pt4 * try_choked_mfp(wgas, tt4, f)? / powp(tt4, 0.5) / (1.0 + f);
    let mdot_hpc = hpc_from_burner(cb, eps, mdot_burner, mdot_d);
    check_root(c, eps, mdot_hpc)?;
    let mdot_air = mdot_hpc / (1.0 - b);           // what the INLET ingests

    // The loads at the ROOT, in watts and as shares of each turbine's shaft power.
    let (turbine_hp, turbine_lp) = (x.dh_hpt * mdot_hpc, x.dh_lpt * mdot_air);
    let (power_hp, power_lp) = (x.q_hp * mdot_hpc, x.q_lp * mdot_air);
    let (share_hp, share_lp) = (power_hp / turbine_hp, power_lp / turbine_lp);
    check_shares(share_hp, share_lp)?;

    // --- the forward rebuild: rung 91's, each turbine now driving its compressor AND its load ---
    let rgas = base.gas().clone();
    let (state0, v0) = base.try_freestream_at(flight, mdot_air)?;
    let s2 = Inlet::new(pi_d).apply(&state0, &rgas);
    let s25 = Compressor::new(m.c.pi_lpc, m.eta_lpc, None).apply(&s2, &rgas);
    let s25c = FlowState { mdot: (1.0 - b) * s25.mdot, ..s25 };      // rung 42's extraction
    let s3 = Compressor::new(m.c.pi_hpc, m.eta_hpc, None).apply(&s25c, &rgas);
    let (mdot_vane, mdot_blade) = (e1 * s3.mdot, e2 * s3.mdot);
    let s3c = FlowState { mdot: mdot_burner, ..s3 };
    let s4 = Burner::new(tt4, base.eta_b, base.pi_b).apply(&s3c, &rgas);
    let s41 = mix(&s4, mdot_vane, s3.tt, s3.pt, &rgas);
    // THE HP BALANCE WITH THE LOAD: eta_m * gas * dh_t = HPC air * dh_c + P_hp.
    let dh_hpt_reb = ((rgas.h_c(s3.tt) - rgas.h_c(s25.tt)) * s3.mdot + power_hp)
        / (base.eta_m * s41.mdot);
    let s44 = Turbine::new(m.eta_hpt, None).apply(&s41, &rgas, dh_hpt_reb);
    let s45 = mix(&s44, mdot_blade, s3.tt, s3.pt, &rgas);
    // THE LP BALANCE WITH THE LOAD, per face air.
    let dh_lpt_reb = ((rgas.h_c(s25.tt) - rgas.h_c(s2.tt)) * s2.mdot + power_lp)
        / (base.eta_m * s45.mdot);
    let s5 = Turbine::new(m.eta_lpt, None).apply(&s45, &rgas, dh_lpt_reb);
    let exit = Nozzle::convergent(flight.p0, base.pi_n).try_apply(&s5, &rgas)?;
    if exit.p9 <= flight.p0 + 1e-6
        && crate::steady_unchoked::refuse_unchoked(base, tt4, (1.0 + s45.far) * exit.v9 - v0)? {
        return Err(Abort(format!(
            "rung-92 loaded match at Tt4={tt4:.0}, M0={:.2}: nozzle UNCHOKED -- OUT OF SCOPE \
             (docs/rung38-spec.md 'Scope').", flight.m0)));
    }

    let stations = vec![
        ("0", state0), ("2", s2), ("25", s25), ("3", s3), ("4", s4), ("41", s41), ("44", s44),
        ("45", s45), ("5", s5), ("9", exit.state),
    ];
    let perf = try_score_nozzle(&stations, s45.far, &rgas, v0, exit.t9, exit.v9, exit.p9,
                                flight.p0, rgas.hpr())?;
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
    Ok(OfftakeResult {
        base: TwoSpoolMapResult {
            base: base_result,
            eta_lpc: m.eta_lpc, eta_hpc: m.eta_hpc, eta_hpt: m.eta_hpt, eta_lpt: m.eta_lpt,
            n_lp: m.n_l, n_hp: m.n_h, n_lp_ratio: m.nl, n_hp_ratio: m.nh, slip: m.slip,
            phi_lp: m.phi_l, phi_hp: m.phi_h, nu_hpt: m.nu_hpt, nu_lpt: m.nu_lpt,
        },
        booking: Some(OfftakeBooking {
            power_hp, power_lp, share_hp, share_lp, turbine_hp, turbine_lp,
            bleed: b, customer: c, vane: e1, blade: e2, mdot_hpc, mdot_burner,
            tt41: s41.tt, tt44: s44.tt, tt45: s45.tt, st_inlet,
            tsfc_inlet: (mdot_burner * s4.far / mdot_air) / st_inlet,
        }),
    })
}

/// RUNG 92. Two-spool map matching with a shaft-power offtake (off the design engine), beside
/// rung 91's cooling, rung 90's customer bleed and rung 42's interstage valve.
///
/// ```text
/// let de = build_two_spool_turbojet(..).with_shaft_offtake(ShaftOfftake::hp(Demand::power(0.02)));
/// let m = ShaftOfftakeMatcher::new(de, flight, 1.0, map_lp, map_hp, 0.0);
/// let od = m.match_point(&flight, 1200.0);
/// ```
pub struct ShaftOfftakeMatcher {
    /// The rung-39 core carrying [`R92`]; rung 41's surge readers reach rung 92's body through it.
    pub core: TwoSpoolMapCore,
}

impl ShaftOfftakeMatcher {
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: crate::map::ComponentMap, map_hp: crate::map::ComponentMap, bleed: f64,
    ) -> Self {
        let mut core = TwoSpoolMapCore::with_hooks(
            design_engine, flight_design, mdot_design, map_lp, map_hp, &R92);
        assert!((0.0..0.5).contains(&bleed), "rung-42 bleed fraction must be in [0, 0.5)");
        core.bleed = bleed;
        ShaftOfftakeMatcher { core }
    }

    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> OfftakeResult {
        self.try_match_point(flight, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    pub fn try_match_point(
        &self, flight: &FlightCondition, tt4: f64,
    ) -> Result<OfftakeResult, Abort> {
        try_match_loaded(&self.core, flight, tt4)
    }
}

// =========================================================================================
// TRANSIENT — rung 62's closure cells MARKED, its power and tail cells WRAPPED
// =========================================================================================

/// RUNG 92's `_close`: rung 62's closure, unchanged, marked with the load.
pub fn r92_try_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    bump(&CLOSE_R92);
    let mut c = r62_close_body(t, nu_lp, nu_hp, tt4, tt2, pt2)?;
    c.offtake = t.inner.base.offtake;
    Ok(c)
}

/// RUNG 92's `_close_fuel`: rung 62's fuel closure, unchanged, marked with the load.
pub fn r92_try_close_fuel(
    ft: &FuelTransientCore, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
) -> Result<FuelCloseState, Abort> {
    bump(&CLOSE_FUEL_R92);
    let mut c = r62_close_fuel_body(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2)?;
    c.base.offtake = ft.inner.inner.base.offtake;
    Ok(c)
}

/// Each shaft's compressor air at a closure, on every lower path: `(HPC air, face air)`.
fn airs(c: &CloseState) -> (f64, f64) {
    let face = c.mdot_face.unwrap_or(c.mdot_air);
    let hpc = if c.customer.is_some() { (1.0 - c.bleed.unwrap_or(0.0)) * face } else { c.mdot_air };
    (hpc, face)
}

/// The load's share of each turbine's shaft power at an instant, `(HP, LP)` — 0 on an unloaded
/// shaft or an unmarked closure. Each lower body's `p_t` is per kg of that shaft's compressor air
/// ([`airs`]), so the turbine's power is `air · p_t` (the plant gate holds this to the steady
/// booking on every lower path).
pub fn shares(t: &TwoSpoolTransientCore, i: &Instant2) -> (f64, f64) {
    let Some(load) = i.close.offtake else { return (0.0, 0.0) };
    let (hpc, face) = airs(&i.close);
    let (pt_hp, pt_lp) = (hpc * i.pt_hp, face * i.pt_lp);
    (watts(load.hp, pt_hp, t.p_ref_hp) / pt_hp, watts(load.lp, pt_lp, t.p_ref_hp) / pt_lp)
}

/// The load at a closure, from the UNLOADED rates `(Φ̇_L, Φ̇_H)` the earlier bodies return.
///
/// Each body writes `Φ̇ = air·(p_t − p_c)/(P_ref,shaft·ν)`, with `air` that SHAFT'S compressor air
/// — on rung 57's path the closure's `mdot_air` for both, on rung 62's bled path the face air on
/// the LP and `mdot_air` (the HPC air) on the HP, on rungs 90/91's the face and `(1−b)·face`.
/// So the turbine's shaft power is `Φ̇·P_ref,shaft·ν + air·p_c`, and the load comes off `Φ̇` as
/// `P_x/(P_ref,shaft·ν)`. Returns `(Φ̇_L, Φ̇_H, P_x,L, P_x,H, P_T,L, P_T,H)`.
fn loaded(
    t: &TwoSpoolTransientCore, c: &CloseState, phi_lp_dot: f64, phi_hp_dot: f64, nu_lp: f64,
    nu_hp: f64,
) -> (f64, f64, f64, f64, f64, f64) {
    let load = c.offtake.expect("a rung-92 cell is reached only on a marked closure");
    let core = &t.inner;
    let wgas = c.gas(core);
    let (hpc, face) = airs(c);
    let pc_hp = wgas.h_c(c.tt3) - wgas.h_c(c.tt25);
    let pc_lp = wgas.h_c(c.tt25) - wgas.h_c(c.tt2);
    let pt_hp = phi_hp_dot * t.p_ref_hp * nu_hp + hpc * pc_hp;
    let pt_lp = phi_lp_dot * t.p_ref_lp * nu_lp + face * pc_lp;
    // ONE power unit on both shafts: the design HP compressor power.
    let px_hp = watts(load.hp, pt_hp, t.p_ref_hp);
    let px_lp = watts(load.lp, pt_lp, t.p_ref_hp);
    (phi_lp_dot - px_lp / (t.p_ref_lp * nu_lp), phi_hp_dot - px_hp / (t.p_ref_hp * nu_hp),
     px_lp, px_hp, pt_lp, pt_hp)
}

/// RUNG 92's `_powers`: rung 62's unloaded powers, less the load.
pub fn r92_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, flight: &FlightCondition, nu_lp: f64, nu_hp: f64,
    tt4: f64,
) -> Result<(f64, f64), Abort> {
    bump(&POWERS_R92);
    let (pl, ph) = r62_powers_body(t, c, flight, nu_lp, nu_hp, tt4)?;
    let (pl, ph, ..) = loaded(t, c, pl, ph, nu_lp, nu_hp);
    Ok((pl, ph))
}

/// RUNG 92's tail: rung 62's, its two rates loaded. The thrust is untouched (the load moves no
/// gas); the bound is checked here, on a marched state, and never inside the Newton's powers.
/// `pt_lp`/`pt_hp`/`pc_lp`/`pc_hp` stay the UNLOADED turbine and compressor works.
pub fn r92_try_instant_tail(
    t: &TwoSpoolTransientCore, flight: &FlightCondition, c: &CloseState, nu_lp: f64, nu_hp: f64,
    tt4: f64, v0: f64,
) -> Result<Instant2, Abort> {
    bump(&TAIL_R92);
    let mut i = r62_tail_body(t, flight, c, nu_lp, nu_hp, tt4, v0)?;
    let (pl, ph, px_lp, px_hp, pt_lp, pt_hp) =
        loaded(t, c, i.phi_lp_dot, i.phi_hp_dot, nu_lp, nu_hp);
    check_shares(px_hp / pt_hp, px_lp / pt_lp)?;
    i.phi_lp_dot = pl;
    i.phi_hp_dot = ph;
    Ok(i)
}
