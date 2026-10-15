//! RUNG 90 — THE CUSTOMER BLEED: air the aircraft takes off the HPC delivery (station 3).
//!
//! Anchor (the derivation, the linear model and every pre-registered bar, committed before any of
//! this code existed): `docs/plans/rung90-anchor-customer-bleed.md`. Spec: `docs/rung90-spec.md`.
//!
//! Not a lever: a LOAD. The aircraft draws a share `c` of the HPC delivery (or a fixed amount of
//! air, so a larger share at low power) for its cabin and anti-ice, and it never comes back. The
//! hardware is captured with it OFF ([`crate::two_spool::TwoSpoolEngine::customer`]).
//!
//! # Where `c` enters — and the two places it breaks rung 42's structure
//!
//! Per unit inlet air: the LPC pumps `1`, the HPC `(1-b)`, the burner and both turbines
//! `(1-b)(1-c)` (times `1+f` after the burner).
//!
//! 1. **The HP shaft balance** carries `(1-c)` — rung 42's `(1-b)` cancelled there, this does
//!    NOT: the HP turbine drives its compressor with less gas than the compressor pumps.
//! 2. **The HP face referral (†)** carries `1/(1-c)`, so rung 39's HP efficiency loop can no
//!    longer be called verbatim (rung 42 could): [`hp_eta_loop_customer`] is its copy with (†-c).
//!    The triangle survives — the HP still closes on itself, given `(Tt25, Tt3, c)`.
//! 3. The LP shaft balance carries `(1-b)(1-c)`; 4. the LP face referral (‡) `1/[(1-b)(1-c)]`;
//! 5. the forward rebuild books the extraction at station 3, before the burner; 6. the customer
//!    air was captured, so it pays full ram drag and returns no momentum.
//!
//! # Reduce — by DISPATCH
//!
//! An INACTIVE customer bleed (`None`, a zero share, a zero amount) never enters this module's
//! arithmetic: the steady cell hands to rung 42's (hence rung 39's) body, the transient cells to
//! rung 62's (hence rung 57's). Every body that cannot model it refuses an ACTIVE one
//! ([`crate::two_spool::TwoSpoolCore::refuse_customer`]).
//!
//! # The bodies are COPIES, not parameterisations
//!
//! Rung 42's rule ([[rust-port-copy-vs-rederivation]]): the steady rebuild, the efficiency loops and
//! rung 62's four transient cells are copied here with `c` inserted, so the earlier gated bodies
//! keep running unchanged and are what an inactive customer bleed reaches.

use std::cell::Cell;

use crate::components::{try_choked_mfp, ram_recovery, Burner, Compressor, Inlet, Nozzle, Turbine};
use crate::engine::{try_score, FlightCondition};
use crate::fuel_transient::{FuelCloseState, FuelTransientCore};
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::map::ComponentMap;
use crate::matcher::Branch;
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::two_spool::{
    secant, Cascade, CascadeMap, EtaLoop, TwoSpoolCore, TwoSpoolEngine, TwoSpoolHooks,
    TwoSpoolMapCore, TwoSpoolMapResult, TwoSpoolResult, R39,
};
use crate::two_spool_transient::{CloseState, Instant2, TwoSpoolTransientCore};

// =========================================================================================
// THE DEMAND
// =========================================================================================

/// What the aircraft draws off station 3.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CustomerBleed {
    /// A fixed SHARE `c` of the HPC delivery, at every power.
    Share(f64),
    /// A fixed AMOUNT of air, `w` × the design inlet flow (kg/s at the design flow of 1 kg/s) —
    /// as a cabin system draws it, so a LARGER share of a smaller flow at low power.
    Amount(f64),
}

impl CustomerBleed {
    /// The largest share any closure may reach — rung 42's bound: half the core gone and the
    /// choked branch is long lost.
    pub const C_MAX: f64 = 0.5;

    /// A fixed share, range-checked (`[0, 0.5)`, rung 42's bound).
    pub fn share(c: f64) -> Self {
        assert!((0.0..Self::C_MAX).contains(&c),
                "rung-90 customer share must be in [0, 0.5): got {c}");
        CustomerBleed::Share(c)
    }

    /// A fixed amount, `w` × the design inlet flow, range-checked (`[0, 0.5)`).
    pub fn amount(w: f64) -> Self {
        assert!((0.0..Self::C_MAX).contains(&w),
                "rung-90 customer amount must be in [0, 0.5) of the design flow: got {w}");
        CustomerBleed::Amount(w)
    }

    /// Does it take any air? A zero share or a zero amount is INACTIVE, and dispatches away.
    pub fn is_active(&self) -> bool {
        match *self {
            CustomerBleed::Share(c) => c != 0.0,
            CustomerBleed::Amount(w) => w != 0.0,
        }
    }

    /// The HPC air that feeds `burner` kg/s to the burner — `burner/(1-c)`, or `burner + W`.
    pub fn hpc_from_burner(&self, burner: f64, mdot_design: f64) -> f64 {
        match *self {
            CustomerBleed::Share(c) => burner / (1.0 - c),
            CustomerBleed::Amount(w) => burner + w * mdot_design,
        }
    }

    /// The share at a given HPC flow, UNCHECKED — for a bracket trial, where an off-root flow may
    /// sit anywhere (the Tt4-pinned closure's high wall imposes a tiny flow, so a check there
    /// would stop a solve whose root is fine).
    pub fn share_raw(&self, hpc: f64, mdot_design: f64) -> f64 {
        match *self {
            CustomerBleed::Share(c) => c,
            CustomerBleed::Amount(w) => w * mdot_design / hpc,
        }
    }

    /// The share `c` this demand is at a given HPC flow — `Err` past [`C_MAX`](Self::C_MAX)
    /// (an amount the compressor cannot deliver and keep the burner alight).
    pub fn share_at(&self, hpc: f64, mdot_design: f64) -> Result<f64, Abort> {
        let c = self.share_raw(hpc, mdot_design);
        if !(c < Self::C_MAX) || !(hpc > 0.0) {
            return Err(Abort(format!(
                "rung-90 customer bleed takes {:.1} % of the compressor's air here (HPC flow \
                 {hpc:.4} kg/s): more than half, so the burner is starved — the aircraft is asking \
                 for more air than the engine can give at this power.", 100.0 * c)));
        }
        Ok(c)
    }
}

// =========================================================================================
// DISPATCH COUNTERS — the gates that a value key cannot replace (rung 62's lesson)
// =========================================================================================

thread_local! {
    static STEADY_ACTIVE: Cell<u64> = const { Cell::new(0) };
    static STEADY_INACTIVE: Cell<u64> = const { Cell::new(0) };
    static CLOSE_R90: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_R90: Cell<u64> = const { Cell::new(0) };
    static POWERS_R90: Cell<u64> = const { Cell::new(0) };
    static TAIL_R90: Cell<u64> = const { Cell::new(0) };
}

fn bump(k: &'static std::thread::LocalKey<Cell<u64>>) {
    k.with(|x| x.set(x.get() + 1));
}

/// How many times each rung-90 body ran on this thread — `(steady active, steady inactive,
/// close, close_fuel, powers, tail)`. Reset by [`reset_counts`].
pub fn counts() -> [u64; 6] {
    [&STEADY_ACTIVE, &STEADY_INACTIVE, &CLOSE_R90, &CLOSE_FUEL_R90, &POWERS_R90, &TAIL_R90]
        .map(|k| k.with(|x| x.get()))
}

pub fn reset_counts() {
    for k in [&STEADY_ACTIVE, &STEADY_INACTIVE, &CLOSE_R90, &CLOSE_FUEL_R90, &POWERS_R90,
              &TAIL_R90] {
        k.with(|x| x.set(0));
    }
}

// =========================================================================================
// STEADY — THE HOOK TABLE
// =========================================================================================

/// RUNG 90's steady table. Only `try_match_point` changes; the efficiency-loop slots stay rung
/// 39's, because an INACTIVE customer bleed must reach them through rung 42's body unchanged, and
/// the ACTIVE body calls its own copies by name.
pub const R90: TwoSpoolHooks = TwoSpoolHooks {
    try_match_point: r90_try_match_point,
    hp_eta_loop: R39.hp_eta_loop,
    lp_eta_loop: R39.lp_eta_loop,
};

fn r90_try_match_point(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<TwoSpoolMapResult, Abort> {
    try_match_customer(core, flight, tt4).map(|r| r.base)
}

// =========================================================================================
// STEADY — THE EFFICIENCY LOOPS WITH THE EXTRACTION
// =========================================================================================

/// Rung 39's [`hp_eta_loop_closed`](crate::two_spool::hp_eta_loop_closed) with (†-c): the HP face
/// passes `1/(1-c)` of the burner flow the HPT nozzle chokes on. **The one difference is that
/// factor**; rung 42 could call rung 39's loop verbatim, this rung cannot.
#[allow(clippy::too_many_arguments)]
pub fn hp_eta_loop_customer(
    wgas: &Gas, tt4: f64, f: f64, tt25: f64, tt3: f64, mfp4: f64, cmap: &ComponentMap, c: f64,
    eta_hpc_base: f64, a4: f64, pi_b: f64, mcorr_hp_d: f64, tau_hpc_d: f64,
) -> Result<EtaLoop, Abort> {
    let (h25, h3, pr25) = (wgas.h_c(tt25), wgas.h_c(tt3), wgas.pr_c(tt25));
    let tau_hpc = tt3 / tt25;
    let (mut eta, mut eta_prev, mut r_prev) = (eta_hpc_base, None, f64::NAN);
    for _ in 0..TwoSpoolMapCore::ETA_MAX {
        let pi = wgas.pr_c(wgas.t_from_h_c(h25 + eta * (h3 - h25))) / pr25;
        // (†-c): pi_LPC-free, and the extraction 1/(1-c).
        let m = (a4 * pi_b * pi * mfp4 * powp(tt25 / tt4, 0.5) / ((1.0 + f) * (1.0 - c)))
            / mcorr_hp_d;
        let n = cmap.try_solve_n(m, tau_hpc, tau_hpc_d)?;
        let tgt = cmap.eta_c_at(eta_hpc_base, m / n, n);
        let r = tgt - eta;
        if r.abs() <= TwoSpoolMapCore::ETA_TOL {
            return Ok(EtaLoop { eta, pi, m, n });
        }
        let nxt = secant(eta, eta_prev, r, r_prev, tgt);
        eta_prev = Some(eta);
        r_prev = r;
        eta = nxt;
    }
    Err(Abort(format!("rung-90 HP efficiency secant did not converge at Tt4={tt4}, c={c}")))
}

/// Rung 42's [`lp_eta_loop_bleed`](crate::bleed::lp_eta_loop_bleed) with (‡-bc): the LP face
/// passes `1/[(1-b)(1-c)]` of the burner flow.
#[allow(clippy::too_many_arguments)]
pub fn lp_eta_loop_customer(
    wgas: &Gas, tt2: f64, tt4: f64, f: f64, tt25: f64, mfp4: f64, pi_hpc: f64,
    cmap: &ComponentMap, b: f64, c: f64, eta_lpc_base: f64, a4: f64, pi_b: f64,
    mcorr_lp_d: f64, tau_lpc_d: f64,
) -> Result<EtaLoop, Abort> {
    let (h2, h25, pr2) = (wgas.h_c(tt2), wgas.h_c(tt25), wgas.pr_c(tt2));
    let tau_lpc = tt25 / tt2;
    let (mut eta, mut eta_prev, mut r_prev) = (eta_lpc_base, None, f64::NAN);
    for _ in 0..TwoSpoolMapCore::ETA_MAX {
        let pi = wgas.pr_c(wgas.t_from_h_c(h2 + eta * (h25 - h2))) / pr2;
        let m = (a4 * pi_b * pi_hpc * pi * mfp4 * powp(tt2 / tt4, 0.5)
                 / ((1.0 + f) * (1.0 - b) * (1.0 - c))) / mcorr_lp_d;
        let n = cmap.try_solve_n(m, tau_lpc, tau_lpc_d)?;
        let tgt = cmap.eta_c_at(eta_lpc_base, m / n, n);
        let r = tgt - eta;
        if r.abs() <= TwoSpoolMapCore::ETA_TOL {
            return Ok(EtaLoop { eta, pi, m, n });
        }
        let nxt = secant(eta, eta_prev, r, r_prev, tgt);
        eta_prev = Some(eta);
        r_prev = r;
        eta = nxt;
    }
    Err(Abort(format!("rung-90 LP efficiency secant did not converge at Tt4={tt4}, c={c}")))
}

/// Rung 42's cascade with the station-3 extraction `c` beside the station-25 one `b`.
pub fn try_cascade_customer(
    core: &TwoSpoolMapCore, wgas: &Gas, tt2: f64, tt4: f64, f: f64, c: f64,
) -> Result<CascadeMap, Abort> {
    let b = core.bleed;
    let base = &core.base;
    let mfp4 = try_choked_mfp(wgas, tt4, f)?;
    let (mut eta_hpt, mut eta_lpt) = (base.eta_hpt, base.eta_lpt);
    for _ in 0..TwoSpoolMapCore::TURB_MAX {
        let (pi_hpt, tau_hpt, tt45) = base.try_solve_choked_turbine(
            wgas, tt4, f, base.a4, base.a45, 1.0, eta_hpt)?;
        let (pi_lpt, tau_lpt, tt5) = base.try_solve_choked_turbine(
            wgas, tt45, f, base.a45, base.a8, base.pi_n, eta_lpt)?;

        // (3) the LP balance: the LPT drives the face air with (1-b)(1-c)(1+f) of gas.
        let dh_lpt = base.eta_m * (1.0 - b) * (1.0 - c) * (1.0 + f)
            * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
        let tt25 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_lpt);
        // (1) the HP balance — NO LONGER bleed-invariant: the HPT passes (1-c)(1+f) per HPC air.
        let dh_hpt = base.eta_m * (1.0 - c) * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
        let tt3 = wgas.t_from_h_c(wgas.h_c(tt25) + dh_hpt);

        let hp = hp_eta_loop_customer(wgas, tt4, f, tt25, tt3, mfp4, &core.map_hp(), c,
                                      base.eta_hpc, base.a4, base.pi_b, core.mcorr_hp_d,
                                      core.tau_hpc_d)?;
        let lp = lp_eta_loop_customer(wgas, tt2, tt4, f, tt25, mfp4, hp.pi, &core.map_lp(), b, c,
                                      base.eta_lpc, base.a4, base.pi_b, core.mcorr_lp_d,
                                      core.tau_lpc_d)?;

        let nl = lp.n * powp(tt2 / core.tt2_d, 0.5);
        let nh = hp.n * powp(tt25 / core.tt25_d, 0.5);
        let nu_hpt = nh * powp(core.tt4_d / tt4, 0.5);
        let nu_lpt = nl * powp(core.tt45_d / tt45, 0.5);
        let out = CascadeMap {
            c: Cascade { pi_hpt, tau_hpt, tt45, pi_lpt, tau_lpt, tt5,
                         pi_lpc: lp.pi, tt25, pi_hpc: hp.pi, tt3 },
            eta_lpc: lp.eta, eta_hpc: hp.eta, eta_hpt, eta_lpt,
            m_l: lp.m, m_h: hp.m, n_l: lp.n, n_h: hp.n, nl, nh,
            phi_l: lp.m / lp.n, phi_h: hp.m / hp.n, nu_hpt, nu_lpt, slip: nl / nh,
        };
        let t_hpt = core.map_hp().eta_t_at(base.eta_hpt, nu_hpt);
        let t_lpt = core.map_lp().eta_t_at(base.eta_lpt, nu_lpt);
        if (t_hpt - eta_hpt).abs() <= TwoSpoolMapCore::ETA_TOL
            && (t_lpt - eta_lpt).abs() <= TwoSpoolMapCore::ETA_TOL {
            return Ok(out);
        }
        eta_hpt = t_hpt;
        eta_lpt = t_lpt;
    }
    Err(Abort(format!("rung-90 turbine-efficiency loop did not converge at Tt4={tt4}")))
}

// =========================================================================================
// STEADY — MATCH ONE POINT
// =========================================================================================

/// What the customer bleed books on a matched point — present only when it was ACTIVE.
#[derive(Clone, Copy, Debug)]
pub struct CustomerBooking {
    /// The interstage (rung 42) share at this point.
    pub bleed: f64,
    /// The customer SHARE of the HPC delivery at this point (the amount mode's share floats).
    pub customer: f64,
    /// Air through the HPC, kg/s.
    pub mdot_hpc: f64,
    /// Air through the burner and turbines, kg/s.
    pub mdot_burner: f64,
    /// What the aircraft gets, kg/s — and at `(Tt3, pt3)`.
    pub mdot_customer: f64,
    pub tt3: f64,
    pub pt3: f64,
    /// `F / ṁ_INLET`: the burner-referenced specific thrust scaled by the burner's share of the
    /// inlet air, minus full ram drag on BOTH extractions.
    pub st_inlet: f64,
    /// Fuel per unit thrust, the thrust carrying both extractions' drag.
    pub tsfc_inlet: f64,
}

/// A matched two-spool point with the customer bleed.
#[derive(Clone, Debug)]
pub struct CustomerBleedResult {
    /// `base.performance` is BURNER-referenced; `base.thrust` is the honest inlet-referenced one.
    pub base: TwoSpoolMapResult,
    /// `None` exactly when the customer bleed was INACTIVE and rung 42's body produced this point.
    pub booking: Option<CustomerBooking>,
}

/// Match at `(flight, Tt4)` with the interstage valve at `core.bleed` and the customer bleed off
/// the design engine. **Inactive ⇒ rung 42's body verbatim** (hence rung 39's at `b = 0`).
pub fn try_match_customer(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<CustomerBleedResult, Abort> {
    if !core.base.customer_active() {
        bump(&STEADY_INACTIVE);
        let base = (crate::bleed::R42.try_match_point)(core, flight, tt4)?;
        return Ok(CustomerBleedResult { base, booking: None });
    }
    bump(&STEADY_ACTIVE);
    core.base.refuse_cooling("rung 90's customer-bleed match");
    core.base.refuse_offtake("rung 90's customer-bleed match");

    assert!(core.stack_lp.is_none() && core.stack_hp.is_none(),
            "rung-90's steady body lumps both compressors; rung 55's stage stacks are out of scope");
    let cb = core.base.customer.expect("active");
    let b = core.bleed;
    let base = &core.base;
    let mdot_d = base.mdot_air_design;
    let pi_d = base.pi_d_max * ram_recovery(flight.m0);
    let (state0, _v0) = base.try_freestream_for(flight)?;
    let (tt2, pt2) = (state0.tt, pi_d * state0.pt);

    // The joint (f, pt4) loop of rungs 38/39/42, with the share c iterated inside it: the amount
    // mode's share depends on the burner flow the HPT nozzle imposes, which depends on pt4.
    let (mut f, mut pt4) =
        (base.f_design, base.pi_b * base.pi_hpc_design * base.pi_lpc_design * pt2);
    let mut c = match cb {
        CustomerBleed::Share(c) => c,
        CustomerBleed::Amount(w) => w,
    };
    let mut cm: Option<CascadeMap> = None;
    for _ in 0..TwoSpoolCore::MAX {
        let owned = base.try_working_gas(f, tt4, pt4)?;
        let wgas = owned.as_ref().unwrap_or(base.gas());
        let x = try_cascade_customer(core, wgas, tt2, tt4, f, c)?;
        let pt4_new = base.pi_b * x.c.pi_hpc * x.c.pi_lpc * pt2;
        let f_new = base.try_solve_f(x.c.tt3, pt4_new, tt4)?;
        let c_new = match cb {
            CustomerBleed::Share(c) => c,
            CustomerBleed::Amount(_) => {
                let owned_n = base.try_working_gas(f_new, tt4, pt4_new)?;
                let wg = owned_n.as_ref().unwrap_or(base.gas());
                let burner = base.a4 * pt4_new * try_choked_mfp(wg, tt4, f_new)?
                    / powp(tt4, 0.5) / (1.0 + f_new);
                cb.share_at(cb.hpc_from_burner(burner, mdot_d), mdot_d)?
            }
        };
        let done = (f_new - f).abs() <= TwoSpoolCore::TOL * (f_new + 1e-30)
            && (pt4_new - pt4).abs() <= TwoSpoolCore::TOL * pt4_new
            && (c_new - c).abs() <= TwoSpoolCore::TOL * c_new;
        cm = Some(x);
        f = f_new;
        pt4 = pt4_new;
        c = c_new;
        if done {
            break;
        }
    }
    let x = cm.expect("the joint loop runs at least once");
    if !(x.c.pi_lpc > 1.0 && x.c.pi_hpc > 1.0 && 0.0 < x.c.tau_hpt && x.c.tau_hpt < 1.0
         && 0.0 < x.c.tau_lpt && x.c.tau_lpt < 1.0) {
        return Err(Abort(format!("rung-90 customer-bleed match unphysical at Tt4={tt4:.0}")));
    }

    let owned = base.try_working_gas(f, tt4, pt4)?;
    let wgas = owned.as_ref().unwrap_or(base.gas());
    let mdot_burner = base.a4 * pt4 * try_choked_mfp(wgas, tt4, f)? / powp(tt4, 0.5) / (1.0 + f);
    let mdot_hpc = mdot_burner / (1.0 - c);
    let mdot_air = mdot_hpc / (1.0 - b);           // what the INLET ingests

    // --- the forward rebuild, the extraction booked at station 3 -----------------------------
    let rgas = if base.gas().is_equilibrium() {
        Gas::reacting_equilibrium_with(
            base.hf_fuel_molar.expect("an equilibrium gas carries hf_fuel_molar"), 0.0)
    } else {
        base.gas().clone()
    };
    let (state0, v0) = base.try_freestream_at(flight, mdot_air)?;
    let s2 = Inlet::new(pi_d).apply(&state0, &rgas);
    let s25 = Compressor::new(x.c.pi_lpc, x.eta_lpc, None).apply(&s2, &rgas);
    let s25c = FlowState { mdot: (1.0 - b) * s25.mdot, ..s25 };      // rung 42's extraction
    let s3 = Compressor::new(x.c.pi_hpc, x.eta_hpc, None).apply(&s25c, &rgas);
    let s3c = FlowState { mdot: (1.0 - c) * s3.mdot, ..s3 };         // <- THE CUSTOMER BLEED
    let s4 = Burner::new(tt4, base.eta_b, base.pi_b).apply(&s3c, &rgas);
    // (1) in the rebuild: the HPT drives the HPC air while passing (1-c)(1+f) of it.
    let dh_hpt_reb = (rgas.h_c(s3.tt) - rgas.h_c(s25.tt))
        / (base.eta_m * (1.0 - c) * (1.0 + s4.far));
    let s45 = Turbine::new(x.eta_hpt, None).apply(&s4, &rgas, dh_hpt_reb);
    let dh_lpt_reb = (rgas.h_c(s25.tt) - rgas.h_c(s2.tt))
        / (base.eta_m * (1.0 - b) * (1.0 - c) * (1.0 + s4.far));
    let s5 = Turbine::new(x.eta_lpt, None).apply(&s45, &rgas, dh_lpt_reb);
    let exit = Nozzle::convergent(base.p_ambient, base.pi_n).try_apply(&s5, &rgas)?;
    if exit.p9 <= base.p_ambient + 1e-6 {
        return Err(Abort(format!(
            "rung-90 customer-bleed match at Tt4={tt4:.0}, c={c:.3}, M0={:.2}: nozzle UNCHOKED -- \
             OUT OF SCOPE (docs/rung38-spec.md 'Scope').", flight.m0)));
    }

    let stations = vec![
        ("0", state0), ("2", s2), ("25", s25), ("3", s3), ("4", s4),
        ("45", s45), ("5", s5), ("9", exit.state),
    ];
    let perf = try_score(&rgas, &stations, v0, exit.t9, exit.v9, exit.p9, flight.p0, rgas.hpr())?;
    // (6) THRUST per inlet air: both extractions were captured, so both carry full ram drag.
    let burner_share = mdot_burner / mdot_air;
    let st_inlet = burner_share * perf.specific_thrust - (1.0 - burner_share) * v0;
    let thrust = mdot_air * st_inlet;
    let base_result = TwoSpoolResult {
        stations, performance: perf, v0, v9: exit.v9, m9: exit.m9, t9: exit.t9, p9: exit.p9,
        thrust, tt4, m0: flight.m0, pi_lpc: x.c.pi_lpc, pi_hpc: x.c.pi_hpc,
        tau_lpc: s25.tt / s2.tt, tau_hpc: s3.tt / s25.tt,
        tau_hpt: x.c.tau_hpt, pi_hpt: x.c.pi_hpt, tau_lpt: x.c.tau_lpt, pi_lpt: x.c.pi_lpt,
        mdot_air, mdot_ratio: mdot_air / base.mdot_air_design,
    };
    Ok(CustomerBleedResult {
        base: TwoSpoolMapResult {
            base: base_result,
            eta_lpc: x.eta_lpc, eta_hpc: x.eta_hpc, eta_hpt: x.eta_hpt, eta_lpt: x.eta_lpt,
            n_lp: x.n_l, n_hp: x.n_h, n_lp_ratio: x.nl, n_hp_ratio: x.nh, slip: x.slip,
            phi_lp: x.phi_l, phi_hp: x.phi_h, nu_hpt: x.nu_hpt, nu_lpt: x.nu_lpt,
        },
        booking: Some(CustomerBooking {
            bleed: b, customer: c, mdot_hpc, mdot_burner, mdot_customer: mdot_hpc - mdot_burner,
            tt3: s3.tt, pt3: s3.pt, st_inlet,
            tsfc_inlet: burner_share * s4.far / st_inlet,
        }),
    })
}

/// RUNG 90. Two-spool map matching with the aircraft's customer bleed (off the design engine)
/// beside rung 42's interstage valve.
///
/// ```text
/// let de = build_two_spool_turbojet(..).with_customer_bleed(CustomerBleed::share(0.05));
/// let m = CustomerBleedMatcher::new(de, flight, 1.0, map_lp, map_hp, 0.0);
/// let od = m.match_point(&flight, 1200.0);
/// ```
pub struct CustomerBleedMatcher {
    /// The rung-39 core carrying [`R90`]; rung 41's surge readers live on it and reach rung 90's
    /// body through the table.
    pub core: TwoSpoolMapCore,
}

impl CustomerBleedMatcher {
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, bleed: f64,
    ) -> Self {
        let mut core = TwoSpoolMapCore::with_hooks(
            design_engine, flight_design, mdot_design, map_lp, map_hp, &R90);
        assert!((0.0..0.5).contains(&bleed), "rung-42 bleed fraction must be in [0, 0.5)");
        core.bleed = bleed;
        CustomerBleedMatcher { core }
    }

    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> CustomerBleedResult {
        self.try_match_point(flight, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    pub fn try_match_point(
        &self, flight: &FlightCondition, tt4: f64,
    ) -> Result<CustomerBleedResult, Abort> {
        try_match_customer(&self.core, flight, tt4)
    }
}

// =========================================================================================
// TRANSIENT — rung 62's four cells with the extraction at station 3
// =========================================================================================

/// The share `c` at this closure, or `None` when the closure came from a body without one.
fn customer_of(t: &TwoSpoolTransientCore) -> CustomerBleed {
    t.inner.base.customer.expect("rung-90 transient body reached with no customer bleed")
}

/// RUNG 90's `_close` — rung 62's bled closure with the station-3 extraction. Reached from
/// [`crate::bleed_transient::r62_try_close`] whenever the customer bleed is ACTIVE (at any `b`,
/// including 0).
pub fn r90_try_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    bump(&CLOSE_R90);
    t.inner.base.refuse_cooling("rung 90's closure");

    let b = t.b_of(nu_lp, Some(tt2));
    t.arm(nu_lp, nu_hp, tt2);
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
        // The BURNER air the NGV choke imposes, the HPC air that implies, the face air that implies.
        let mdot_imp = mdot4 / (1.0 + f);
        let hpc_imp = cb.hpc_from_burner(mdot_imp, mdot_d);
        let share = cb.share_raw(hpc_imp, mdot_d);
        let face_imp = hpc_imp / (1.0 - b);
        let m_imp = (face_imp * powp(tt2, 0.5) / pt2) / c.mcorr_lp_d;
        Ok(CloseState {
            m_lp, m_imp, m_hp, phi_lp, phi_hp, tt2, n_lp, n_hp, tau_lpc, tau_hpc, tt25, tt3,
            pi_lpc, pi_hpc, pt4, f, wgas, eta_lpc, eta_hpc, mdot_air: mdot_imp, mdot4,
            bleed: Some(b), mdot_face: Some(face_imp), customer: Some(share),
            cooling: None, offtake: None,
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
            "rung-90 two-shaft closure with the customer bleed does not bracket at \
             nu=({nu_lp:.4},{nu_hp:.4}), Tt4={tt4:.0}, b={b:.4} — off the modeled speed-line \
             region.")));
    };
    let root = try_illinois(g, lo, hi, glo, ghi, TwoSpoolTransientCore::CLOSE_TOL,
                            ILLINOIS_MAXIT)?;
    let out = ev(root)?;
    // The bound is a property of the ROOT, checked there (see `share_raw`).
    cb.share_at((1.0 - b) * out.mdot_face.expect("set above"), mdot_d)?;
    Ok(out)
}

/// RUNG 90's `_close_fuel` — the fuel-metered closure. **The burner never sees the customer air
/// either**, so `f = ṁ_fuel / burner air` with the burner air `(1-c)` of the HPC air (or the HPC
/// air less the amount), and the rich/lean bracket walls carry the same factor (rung 62's I5).
pub fn r90_try_close_fuel(
    ft: &FuelTransientCore, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
) -> Result<FuelCloseState, Abort> {
    bump(&CLOSE_FUEL_R90);
    ft.inner.inner.base.refuse_cooling("rung 90's fuel closure");

    let b = ft.inner.b_of(nu_lp, Some(tt2));
    ft.inner.arm(nu_lp, nu_hp, tt2);
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

        // THE CUSTOMER AIR LEAVES BEFORE THE BURNER: the fuel meets the trial HPC air less it.
        let share = cb.share_at(mdot_hpc, mdot_d)?;
        let burner_trial = (1.0 - share) * mdot_hpc;
        let f = mdot_fuel / burner_trial;
        let tt4 = ft.try_tt4_from_f(tt3, f)?;
        let wgas = c.base.try_working_gas(f, tt4, pt4)?;
        let wg = wgas.as_ref().unwrap_or(gas);
        let mdot4 = c.base.a4 * pt4 * try_choked_mfp(wg, tt4, f)? / powp(tt4, 0.5);
        let mdot_imp = mdot4 / (1.0 + f);
        let face_imp = cb.hpc_from_burner(mdot_imp, mdot_d) / (1.0 - b);
        let m_imp = (face_imp * powp(tt2, 0.5) / pt2) / c.mcorr_lp_d;
        Ok(FuelCloseState {
            base: CloseState {
                m_lp, m_imp, m_hp, phi_lp, phi_hp, tt2, n_lp, n_hp, tau_lpc, tau_hpc, tt25, tt3,
                pi_lpc, pi_hpc, pt4, f, wgas, eta_lpc, eta_hpc, mdot_air: mdot_imp, mdot4,
                bleed: Some(b), mdot_face: Some(face_imp), customer: Some(share),
            cooling: None, offtake: None,
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

    // The f caps are BURNER-referenced, so the FACE-flow walls they imply carry the customer
    // extraction as well as rung 62's 1/(1-b) — without it the scan starts inside the root.
    let face_at = |f_wall: f64| cb.hpc_from_burner(mdot_fuel / f_wall, mdot_d) / (1.0 - b);
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
            "rung-90 bled fuel closure does not bracket (customer bleed on) at \
             nu=({nu_lp:.4},{nu_hp:.4}), mdot_fuel={mdot_fuel:.5}, b={b:.4} — off the modeled \
             speed-line region.")));
    };
    let root = try_illinois(g, lo, hi, glo, ghi, TwoSpoolTransientCore::CLOSE_TOL,
                            ILLINOIS_MAXIT)?;
    ev(root)
}

/// The two turbines at this closure — rung 62's pins, unchanged (the extraction is upstream of
/// station 4).
fn turbines(
    t: &TwoSpoolTransientCore, wgas: &Gas, f: f64, nu_lp: f64, nu_hp: f64, tt4: f64,
) -> Result<[f64; 10], Abort> {
    let core = &t.inner;
    let nu_hpt = nu_hp * powp(core.tt4_d / tt4, 0.5);
    let eta_hpt = core.map_hp().eta_t_at(core.base.eta_hpt, nu_hpt);
    let (pi_hpt, tau_hpt, tt45) = core.base.try_solve_choked_turbine(
        wgas, tt4, f, core.base.a4, core.base.a45, 1.0, eta_hpt)?;
    let nu_lpt = nu_lp * powp(core.tt45_d / tt45, 0.5);
    let eta_lpt = core.map_lp().eta_t_at(core.base.eta_lpt, nu_lpt);
    let (pi_lpt, tau_lpt, tt5) = core.base.try_solve_choked_turbine(
        wgas, tt45, f, core.base.a45, core.base.a8, core.base.pi_n, eta_lpt)?;
    Ok([pi_hpt, tau_hpt, tt45, pi_lpt, tau_lpt, tt5, eta_hpt, eta_lpt, nu_hpt, nu_lpt])
}

/// The shaft powers at a closure, in rung 62's bookkeeping: each specific power per unit of that
/// SHAFT'S COMPRESSOR air, so `Phi = compressor air × (turbine − compressor)`. Both turbines pass
/// the BURNER gas, so each turbine term carries the burner's share of its compressor's air —
/// `(1-c)` on the HP (anchor § 1, place 1: it no longer cancels) and `(1-b)(1-c)` on the LP.
/// Returns `(pt_lp, pt_hp, pc_lp, pc_hp, Phi_L, Phi_H)`.
fn shaft_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, wgas: &Gas, tt45: f64, tt5: f64, nu_lp: f64,
    nu_hp: f64, tt4: f64,
) -> (f64, f64, f64, f64, f64, f64) {
    let core = &t.inner;
    let f = c.f;
    let b = c.bleed.unwrap_or(0.0);
    let face = c.mdot_face.expect("a rung-90 closure always sets mdot_face");
    let hpc = (1.0 - b) * face;
    let burner = c.mdot_air;
    let pt_hp = (burner / hpc) * core.base.eta_m * (1.0 + f)
        * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
    let pt_lp = (burner / face) * core.base.eta_m * (1.0 + f)
        * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
    let pc_hp = wgas.h_c(c.tt3) - wgas.h_c(c.tt25);
    let pc_lp = wgas.h_c(c.tt25) - wgas.h_c(c.tt2);
    let phi_hp_dot = (hpc * (pt_hp - pc_hp)) / (t.p_ref_hp * nu_hp);
    let phi_lp_dot = (face * (pt_lp - pc_lp)) / (t.p_ref_lp * nu_lp);
    (pt_lp, pt_hp, pc_lp, pc_hp, phi_lp_dot, phi_hp_dot)
}

/// RUNG 90's `_powers`. Dispatched on the CLOSURE's own `customer` field (rung 62 § 5.21 (v)).
pub fn r90_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, nu_lp: f64, nu_hp: f64, tt4: f64,
) -> Result<(f64, f64), Abort> {
    bump(&POWERS_R90);
    t.inner.base.refuse_cooling("rung 90's shaft powers");

    let wgas = c.gas(&t.inner);
    let tb = turbines(t, wgas, c.f, nu_lp, nu_hp, tt4)?;
    let (_, _, _, _, pl, ph) = shaft_powers(t, c, wgas, tb[2], tb[5], nu_lp, nu_hp, tt4);
    Ok((pl, ph))
}

/// RUNG 90's turbine / power / thrust tail.
pub fn r90_try_instant_tail(
    t: &TwoSpoolTransientCore, flight: &FlightCondition, c: &CloseState, nu_lp: f64, nu_hp: f64,
    tt4: f64, v0: f64,
) -> Result<Instant2, Abort> {
    bump(&TAIL_R90);
    t.inner.base.refuse_cooling("rung 90's thrust tail");

    let core = &t.inner;
    let (wgas, f) = (c.gas(core), c.f);
    let [pi_hpt, tau_hpt, tt45, pi_lpt, tau_lpt, tt5, eta_hpt, eta_lpt, nu_hpt, nu_lpt] =
        turbines(t, wgas, f, nu_lp, nu_hp, tt4)?;
    let (pt_lp, pt_hp, pc_lp, pc_hp, phi_lp_dot, phi_hp_dot) =
        shaft_powers(t, c, wgas, tt45, tt5, nu_lp, nu_hp, tt4);

    let burner = c.mdot_air;
    let face = c.mdot_face.expect("a rung-90 closure always sets mdot_face");
    let s5 = FlowState { tt: tt5, pt: pi_lpt * pi_hpt * c.pt4, mdot: burner, far: f };
    let exit = Nozzle::convergent(core.base.p_ambient, core.base.pi_n).try_apply(&s5, wgas)?;
    let press = (1.0 + f) * wgas.r_t_at(f) * exit.t9 * (1.0 - flight.p0 / exit.p9) / exit.v9;
    let sp_thrust = (1.0 + f) * exit.v9 - v0 + press;
    // (6): per INLET air, both extractions carrying full ram drag.
    let share = burner / face;
    Ok(Instant2 {
        close: c.clone(),
        nu_lp, nu_hp, tt4, slip: nu_lp / nu_hp, phi_lp_dot, phi_hp_dot, pt_lp, pt_hp, pc_lp, pc_hp,
        tt45, tt5, tau_hpt, tau_lpt, pi_hpt, pi_lpt, eta_hpt, eta_lpt, nu_hpt, nu_lpt, sp_thrust,
        sp_thrust_inlet: Some(share * sp_thrust - (1.0 - share) * v0),
        m9: exit.m9,
        branch: if exit.p9 > core.base.p_ambient + 1e-6 { Branch::Choked } else { Branch::Subsonic },
    })
}
