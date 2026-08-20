//! RUNG 42 — INTERSTAGE BLEED: the project's first STEADY mass extraction.
//!
//! A handling-bleed valve at station 25, between the two compressors. The hardware (`A4`, `A45`,
//! `A8`, both maps' design references) is captured from a **bleed-free** design run, exactly as a
//! real valve is shut at the design condition and opened off design.
//!
//! # What the extraction actually changes — and what it deliberately does not
//!
//! Rung 39's triangular cascade survives intact. Only **two** places see `b`:
//!
//! * the **LP shaft balance**, where the LP turbine drives `mdot_2` while passing only
//!   `(1-b)*mdot_2*(1+f)` of gas — so `Tt25` falls;
//! * the **LP face's flow referral** (‡), which picks up `1/(1-b)`.
//!
//! The HP balance is bleed-INVARIANT because `(1-b)` cancels (both sides are core flow), and
//! (†) carries no `b` at all — so [`crate::two_spool::hp_eta_loop_closed`] is called **verbatim**,
//! through the hook table. Both turbine pins are untouched for the same reason. The finding is
//! that bleed is a new degree of freedom **on the LP spool and NOT the HP** one; the "penalises
//! the HP spool" hypothesis is refuted.
//!
//! # Three structural decisions, each of which could be got wrong silently
//!
//! **1. THE REBUILD IS A DELIBERATE DUPLICATION, NOT A PARAMETERISATION.** [`try_match_bleed`]
//! carries its own copy of rungs 38/39's forward rebuild rather than calling `try_rebuild` with a
//! `b` argument, and its own result assembly rather than `try_into_result`. That is the port's
//! *COPY vs REDERIVATION* rule: an "exactly" claim survives a copied instruction sequence and
//! dies on a second derivation, and the Python says so in its own docstring. Factoring the
//! duplication away would look like a cleanup and break P7's bit-for-bit reduce. The result
//! assembly has a second reason: rung 39's computes `thrust = mdot_air * specific_thrust`, and
//! rung 42's needs `mdot_air * st_inlet` — reusing it would compile and silently drop the dumped
//! air's ram drag.
//!
//! **2. `(1-b)` IS INSERTED WHERE PYTHON PUTS IT, TO THE PARENTHESIS.** All three insertions
//! change an association order and not merely a factor: `/((1+f)*(1-b))` is ONE denominator with
//! the product formed inside it, and `eta_m * (1-b) * (1+f)` puts the new factor BETWEEN two
//! existing ones. Left-to-right, neither is rung 39's product with a factor appended. This is the
//! *power-spelling-is-split* class of defect — one bit, amplified by a fixed point.
//!
//! **3. THE UNCHOKED GUARD IS AN `Err`, WHERE PYTHON SPELLS IT `assert`.** Python spells rung
//! 42's scope guard exactly as it spells rung 39's, and rung 39's is already an [`Abort`] here
//! because rung 41's schedule methods SKIP such a point. Rung 42's `match` is reached through the
//! SAME hook from those same methods, so a panic would kill a `surge_margin_schedule` on a bleed
//! matcher where Python skips. It is also the one guard the bleed axis MOVES: 23 cells of the
//! 147-cell dump grid at `b = 0`, rising to 25 at `b = 0.10` — rung 42's own gate 6, *opening the
//! valve shrinks the choked envelope*, expressed as a count that measures physics.
//!
//! Every other guard stays an `assert!` on the zero-firing rule: the constructor's range check
//! (which DOES fire), the `unphysical` closure check, the LP secant and the turbine loop (all
//! measured **0** over 147 cells × 4 bleed levels).
//!
//! **SLICE M MADE THE HP HOOK FALLIBLE, AND LEFT THIS MODULE'S OWN `solve_n` PANICKING — WITH AN
//! EXPIRY DATE.** Rung 54's `_scan` marches the stator closed until the solve gives out, and the
//! frame it lands on is [`ComponentMap::solve_n`](crate::map::ComponentMap::solve_n)'s bracket,
//! so the two rung-39 efficiency loops now return `Result` and [`try_cascade_bleed`] propagates
//! the HP one with `?`. [`lp_eta_loop_bleed`]'s own `solve_n` is NOT converted: no rung-53/54
//! walk reaches it, because rung 53's `at_setting` builds a valve-shut sibling. **Rung 61's
//! `StatorBleedMatcher` overrides `at_setting` precisely so the valve stays OPEN through every
//! sweep** — which puts this call site inside `_scan`'s catch for the first time. Slice O must
//! MEASURE that site, not inherit this paragraph: a zero-firing verdict is a claim about the grid
//! that measured it.
//!
//! **THE EXPIRY CAME DUE AT SLICE O, AND THE MEASUREMENT IS THE POINT.** It fires. Rung 61's
//! `authority_with_bleed` runs rung 54's ceiling walk on a valve-OPEN machine, and the walk ends
//! on THIS bracket — so [`lp_eta_loop_bleed`] now returns `Result` and [`try_cascade_bleed`]
//! propagates it with `?`. The firing count on slice O's 640-cell grid is emitted by the census
//! and gated in `slice_o_oracle.rs`, never restated from here.
//!
//! Two things about how it was found are worth keeping. **The gate found it before the paragraph
//! was read** — the oracle panicked, the backtrace named the frame, and only then did this note
//! turn out to have called it. A deferral written down is worth exactly the grep that finds it,
//! and an empirical failure is what actually reaches you. And **no rung-42 gate could ever have
//! seen it**: rung 42's own readers never walk until refusal, and rung 54's walk never ran on an
//! open valve. The defect lives in neither rung — it is created by the COMPOSITION, which is what
//! rung 61 is.
//!
//! # The narrowing, named rather than left implicit
//!
//! Python's `TwoSpoolBleedMatcher` accepts `lp_disabled`, and with the valve shut it forwards to
//! rung 32's delegate. [`TwoSpoolBleedMatcher`] holds a [`TwoSpoolMapCore`] and so has no
//! degenerate arm at all — rung 41's precedent, one paragraph up the same file: a rung-42 method
//! on an `lp_disabled` Python matcher raises `AttributeError` from the rung-32 delegate, never
//! `AssertionError`, so it was never catchable by the schedule methods either.
//!
//! [`Abort`]: crate::gas::Abort

use crate::components::{choked_mfp, ram_recovery, Burner, Compressor, Inlet, Nozzle, Turbine};
use crate::engine::{try_score, FlightCondition};
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::map::ComponentMap;
use crate::two_spool::{
    counters, secant, Cascade, CascadeMap, EtaLoop, TwoSpoolCore, TwoSpoolEngine, TwoSpoolHooks,
    TwoSpoolMapCore, TwoSpoolMapResult, TwoSpoolResult, R39,
};

// =========================================================================================
// THE RESULT — and the absence a float dump cannot see
// =========================================================================================

/// What the extraction books on a matched point, and **only ever present when the valve was
/// open.**
///
/// Python builds a `TwoSpoolBleedResult` *only* on the `b > 0` path; at `b == 0` it returns rung
/// 39's own object, which has no such attributes at all, and `bleed_trade` reads that absence
/// through `getattr(od, "st_inlet", od.performance.specific_thrust)`. So the dataclass's
/// `st_inlet = 0.0` default is unreachable, and a port that always built this struct would write
/// `0.0` into the `b = 0` row where Python writes the core specific thrust.
///
/// [`Option`] is what makes that unwritable — § 5.8.1's P9 applied a second time in the same
/// slice, to a missing OBJECT rather than a missing value.
#[derive(Clone, Copy, Debug)]
pub struct BleedBooking {
    /// The extraction fraction this point was matched at.
    pub bleed: f64,
    /// Air through HPC / burner / turbines — `(1-b) * mdot_air`.
    pub mdot_core: f64,
    /// `F / mdot_INLET`. The dumped air was captured, so it carries FULL ram drag and returns no
    /// exhaust momentum — an overboard dump with no recovery, the conservative reading:
    /// `(1-b)*specific_thrust - b*V0`.
    pub st_inlet: f64,
    /// `mdot_fuel / F`, with `F` carrying the bleed drag.
    pub tsfc_inlet: f64,
}

/// A matched two-spool point with the interstage bleed valve at `self.bleed`.
///
/// `base.performance` is CORE-referenced (specific thrust per unit air through the burner), so at
/// `b = 0` it is bit-for-bit rung 39's. The honest per-INLET-air numbers are in [`BleedBooking`].
#[derive(Clone, Debug)]
pub struct TwoSpoolBleedResult {
    pub base: TwoSpoolMapResult,
    /// `None` **exactly when the valve was shut** and rung 39's body produced this point.
    pub booking: Option<BleedBooking>,
}

// =========================================================================================
// THE HOOK TABLE
// =========================================================================================

/// RUNG 42's table — **the port's first override of a live virtual slot.**
///
/// Only `try_match_point` changes. The other two are rung 39's, *by name*, and that is
/// load-bearing in both directions:
///
/// * `_hp_eta_loop` is called **VERBATIM** by rung 42's cascade (its body is `b`-free), so the
///   slot must keep rung 39's function — and [`try_cascade_bleed`] must reach it THROUGH the
///   table, since rung 55 overrides that same slot in phase 7.
/// * `_lp_eta_loop_bleed` is a **NEW METHOD NAME in Python, not an override**. Putting
///   [`lp_eta_loop_bleed`] in the `lp_eta_loop` slot would stop rungs 39/41's suites witnessing
///   the unchanged body — the very thing the Python's docstring says it is preserving.
pub const R42: TwoSpoolHooks = TwoSpoolHooks {
    try_match_point: r42_try_match_point,
    hp_eta_loop: R39.hp_eta_loop,
    lp_eta_loop: R39.lp_eta_loop,
};

// =========================================================================================
// THE LP EFFICIENCY FIXED POINT, WITH THE EXTRACTION IN ITS FLOW REFERRAL
// =========================================================================================

/// Rung 39's [`lp_eta_loop_arrow`] with (‡-b): the LP face passes `mdot_core/(1-b)`.
///
/// **The ONLY difference from the rung-39 body is the `/(1-bleed)` on `m`** — and it is spelled as
/// Python spells it, `/ ((1.0 + f) * (1.0 - bleed))`: ONE division by a product formed first.
/// `/(1+f)/(1-b)` is a different double. Rung 39's own function is left untouched so its gates
/// keep witnessing it bit-for-bit, which is why this is a copy and not a `bleed: Option<f64>`
/// parameter on that one.
///
/// [`lp_eta_loop_arrow`]: crate::two_spool::lp_eta_loop_arrow
#[allow(clippy::too_many_arguments)]
pub fn lp_eta_loop_bleed(
    wgas: &Gas, tt2: f64, tt4: f64, f: f64, tt25: f64, mfp4: f64, pi_hpc: f64,
    cmap: &ComponentMap, bleed: f64, eta_lpc_base: f64, a4: f64, pi_b: f64, mcorr_lp_d: f64,
    tau_lpc_d: f64,
) -> Result<EtaLoop, Abort> {
    let (h2, h25, pr2) = (wgas.h_c(tt2), wgas.h_c(tt25), wgas.pr_c(tt2));
    let tau_lpc = tt25 / tt2;
    let (mut eta, mut eta_prev, mut r_prev) = (eta_lpc_base, None, f64::NAN);
    for pass in 0..TwoSpoolMapCore::ETA_MAX {
        let pi = wgas.pr_c(wgas.t_from_h_c(h2 + eta * (h25 - h2))) / pr2;
        // (‡-b): carries pi_hpc (rung 39's ONE arrow) AND the extraction 1/(1-b).
        let m = (a4 * pi_b * pi_hpc * pi * mfp4 * powp(tt2 / tt4, 0.5)
                 / ((1.0 + f) * (1.0 - bleed))) / mcorr_lp_d;
        // SLICE O CONVERTED THIS, AND THE MODULE NOTE ABOVE SAID IT WOULD. The abort is
        // TALLIED rather than merely propagated: slice L's verdict here was a zero-firing
        // COUNT, so what retires it has to be a count too.
        let n = match cmap.try_solve_n(m, tau_lpc, tau_lpc_d) {
            Ok(n) => n,
            Err(e) => {
                counters::bump_lp_bleed_abort();
                return Err(e);
            }
        };
        let tgt = cmap.eta_c_at(eta_lpc_base, m / n, n);
        let r = tgt - eta;
        if r.abs() <= TwoSpoolMapCore::ETA_TOL {
            counters::note_lp(pass as u64);
            return Ok(EtaLoop { eta, pi, m, n });
        }
        let nxt = secant(eta, eta_prev, r, r_prev, tgt);
        eta_prev = Some(eta);
        r_prev = r;
        eta = nxt;
    }
    panic!("rung-42 LP efficiency secant did not converge at Tt4={tt4}, b={bleed}; moderate the \
            LP map coefficients, the bleed or the throttle.");
}

// =========================================================================================
// THE CASCADE WITH THE EXTRACTION
// =========================================================================================

/// Rung 39's triangular cascade with the station-25 extraction — see [`try_cascade_bleed`].
pub fn cascade_bleed(
    core: &TwoSpoolMapCore, wgas: &Gas, tt2: f64, pt2: f64, tt4: f64, f: f64,
) -> CascadeMap {
    try_cascade_bleed(core, wgas, tt2, pt2, tt4, f).unwrap_or_else(|e| panic!("{}", e.0))
}

/// The FALLIBLE twin — see [`Abort`]. Differences from
/// [`try_cascade_map`](TwoSpoolMapCore::try_cascade_map), and ONLY these:
///
/// * the **LP shaft balance** carries `(1-b)` ⇒ `Tt25` falls;
/// * the **LP eta loop** uses (‡-b) ⇒ `m_L` picks up `1/(1-b)`.
///
/// `hp_eta_loop` is called VERBATIM — its body is `b`-free, since (†) carries no `b`. Both turbine
/// pins and the HP shaft balance are untouched for the same reason: `(1-b)` cancels out of the HP
/// balance because both of its sides are core flow.
///
/// `pt2` is UNUSED here as it is in rung 39 — see that method's note.
///
/// [`Abort`]: crate::gas::Abort
pub fn try_cascade_bleed(
    core: &TwoSpoolMapCore, wgas: &Gas, tt2: f64, _pt2: f64, tt4: f64, f: f64,
) -> Result<CascadeMap, Abort> {
    counters::bump_cascade();
    let b = core.bleed;
    let base = &core.base;
    let mfp4 = choked_mfp(wgas, tt4, f);
    let (mut eta_hpt, mut eta_lpt) = (base.eta_hpt, base.eta_lpt);
    for turb_pass in 0..TwoSpoolMapCore::TURB_MAX {
        let (pi_hpt, tau_hpt, tt45) = base.try_solve_choked_turbine(
            wgas, tt4, f, base.a4, base.a45, 1.0, eta_hpt)?;
        let (pi_lpt, tau_lpt, tt5) = base.try_solve_choked_turbine(
            wgas, tt45, f, base.a45, base.a8, base.pi_n, eta_lpt)?;

        // ENERGY. (1) the LP balance: the LPT drives mdot_2 with (1-b)*mdot_2*(1+f) of gas.
        // `(1-b)` sits BETWEEN eta_m and (1+f), which is not rung 39's product with a factor
        // appended — see the module note.
        let dh_lpt =
            base.eta_m * (1.0 - b) * (1.0 + f) * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
        let tt25 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_lpt);
        // The HP balance: (1-b) cancels (both sides are core flow) -> bleed-INVARIANT form.
        let dh_hpt = base.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
        let tt3 = wgas.t_from_h_c(wgas.h_c(tt25) + dh_hpt);

        // THE TRIANGLE, unchanged in shape: HP closes on itself (VERBATIM rung 39, THROUGH the
        // hook — the structural claim), THEN LP closes onto pi_HPC with the extraction in its
        // flow.
        let hp = (core.hooks.hp_eta_loop)(core, wgas, tt4, f, tt25, tt3, mfp4, &core.map_hp())?;
        let lp = lp_eta_loop_bleed(wgas, tt2, tt4, f, tt25, mfp4, hp.pi, &core.map_lp(), b,
                                   base.eta_lpc, base.a4, base.pi_b, core.mcorr_lp_d,
                                   core.tau_lpc_d)?;

        let nl = lp.n * powp(tt2 / core.tt2_d, 0.5);
        let nh = hp.n * powp(tt25 / core.tt25_d, 0.5);
        let nu_hpt = nh * powp(core.tt4_d / tt4, 0.5);
        let nu_lpt = nl * powp(core.tt45_d / tt45, 0.5);

        let out = CascadeMap {
            c: Cascade {
                pi_hpt, tau_hpt, tt45, pi_lpt, tau_lpt, tt5,
                pi_lpc: lp.pi, tt25, pi_hpc: hp.pi, tt3,
            },
            eta_lpc: lp.eta, eta_hpc: hp.eta, eta_hpt, eta_lpt,
            m_l: lp.m, m_h: hp.m, n_l: lp.n, n_h: hp.n, nl, nh,
            phi_l: lp.m / lp.n, phi_h: hp.m / hp.n, nu_hpt, nu_lpt, slip: nl / nh,
        };

        let t_hpt = core.map_hp().eta_t_at(base.eta_hpt, nu_hpt);
        let t_lpt = core.map_lp().eta_t_at(base.eta_lpt, nu_lpt);
        if (t_hpt - eta_hpt).abs() <= TwoSpoolMapCore::ETA_TOL
            && (t_lpt - eta_lpt).abs() <= TwoSpoolMapCore::ETA_TOL {
            counters::note_turb(turb_pass as u64 + 1);
            return Ok(out);
        }
        eta_hpt = t_hpt;
        eta_lpt = t_lpt;
    }
    panic!("rung-42 turbine-efficiency loop did not converge at Tt4={tt4}; moderate a_t.");
}

// =========================================================================================
// MATCH ONE POINT WITH THE VALVE OPEN
// =========================================================================================

/// The hook body. Returns only the rung-39-shaped half, which is all
/// [`TwoSpoolMapCore::try_match_point`]'s signature can carry — and all rung 41's three schedule
/// methods read. [`TwoSpoolBleedMatcher::try_match_point`] is the entry point that keeps the
/// [`BleedBooking`].
fn r42_try_match_point(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<TwoSpoolMapResult, Abort> {
    try_match_bleed(core, flight, tt4).map(|r| r.base)
}

/// Match at `(flight, Tt4)` with the valve at `core.bleed`.
///
/// **REDUCE — exact dispatch (rungs 38/39/40's contract):** `bleed == 0.0` forwards to rung 39's
/// body VERBATIM, so a bleed matcher with the valve shut is rung 39 bit-for-bit and the bleed
/// cascade is never entered. Spelled as `R39.try_match_point` and not as `core.try_match_point`
/// on purpose: this is Python's `super().match(...)`, a NON-virtual call, and routing it back
/// through the table would recurse.
pub fn try_match_bleed(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<TwoSpoolBleedResult, Abort> {
    if core.bleed == 0.0 {
        let base = (R39.try_match_point)(core, flight, tt4)?;
        return Ok(TwoSpoolBleedResult { base, booking: None });
    }

    let b = core.bleed;
    let base = &core.base;
    let pi_d = base.pi_d_max * ram_recovery(flight.m0);
    let (state0, _v0) = base.try_freestream_for(flight)?;
    let (tt2, pt2) = (state0.tt, pi_d * state0.pt);

    let (mut f, mut pt4) =
        (base.f_design, base.pi_b * base.pi_hpc_design * base.pi_lpc_design * pt2);
    let mut c: Option<CascadeMap> = None;
    for _ in 0..TwoSpoolCore::MAX {
        let owned = base.try_working_gas(f, tt4, pt4)?;
        let wgas = owned.as_ref().unwrap_or(base.gas());
        let cm = try_cascade_bleed(core, wgas, tt2, pt2, tt4, f)?;
        let pt4_new = base.pi_b * cm.c.pi_hpc * cm.c.pi_lpc * pt2;
        let f_new = base.try_solve_f(cm.c.tt3, pt4_new, tt4)?;
        let done = (f_new - f).abs() <= TwoSpoolCore::TOL * (f_new + 1e-30)
            && (pt4_new - pt4).abs() <= TwoSpoolCore::TOL * pt4_new;
        c = Some(cm);
        f = f_new;
        pt4 = pt4_new;
        if done {
            break;
        }
    }
    let c = c.expect("the joint loop runs at least once");

    assert!(c.c.pi_lpc > 1.0 && c.c.pi_hpc > 1.0 && 0.0 < c.c.tau_hpt && c.c.tau_hpt < 1.0
                && 0.0 < c.c.tau_lpt && c.c.tau_lpt < 1.0,
            "rung-42 bleed match unphysical");

    let owned = base.try_working_gas(f, tt4, pt4)?;
    let wgas = owned.as_ref().unwrap_or(base.gas());
    let mdot_core = base.a4 * pt4 * choked_mfp(wgas, tt4, f) / powp(tt4, 0.5) / (1.0 + f);
    let mdot_air = mdot_core / (1.0 - b);          // what the INLET ingests

    // --- the SEPARATE rebuild body (module note § 1) --------------------------------------
    //
    // Rebuild FORWARD. The extraction is booked EXPLICITLY at station 25 (the one place mass
    // leaves the flowpath), so every shipped conservation assert downstream still fires — on the
    // core flow, which is what they should see.
    let rgas = if base.gas().is_equilibrium() {
        Gas::reacting_equilibrium_with(
            base.hf_fuel_molar.expect("an equilibrium gas carries hf_fuel_molar"), 0.0)
    } else {
        base.gas().clone()
    };
    let (state0, v0) = base.try_freestream_at(flight, mdot_air)?;
    let s2 = Inlet::new(pi_d).apply(&state0, &rgas);
    let s25 = Compressor::new(c.c.pi_lpc, c.eta_lpc, None).apply(&s2, &rgas);
    let s25c = FlowState { mdot: (1.0 - b) * s25.mdot, ..s25 };   // <- THE BLEED EXTRACTION
    let s3 = Compressor::new(c.c.pi_hpc, c.eta_hpc, None).apply(&s25c, &rgas);
    let s4 = Burner::new(tt4, base.eta_b, base.pi_b).apply(&s3, &rgas);
    let dh_hpt_reb = (rgas.h_c(s3.tt) - rgas.h_c(s25.tt)) / (base.eta_m * (1.0 + s4.far));
    let s45 = Turbine::new(c.eta_hpt, None).apply(&s4, &rgas, dh_hpt_reb);
    // (1) again, in the rebuild: the LPT drives mdot_2 while passing (1-b)*mdot_2*(1+f).
    let dh_lpt_reb =
        (rgas.h_c(s25.tt) - rgas.h_c(s2.tt)) / (base.eta_m * (1.0 - b) * (1.0 + s4.far));
    let s5 = Turbine::new(c.eta_lpt, None).apply(&s45, &rgas, dh_lpt_reb);
    let exit = Nozzle::convergent(base.p_ambient, base.pi_n).try_apply(&s5, &rgas)?;

    // SCOPE GUARD (inherited). Bleed lowers pi_LPC hence pt4, so this bites SOONER — 23 cells of
    // the dump grid at b = 0, 25 at b = 0.10. An `Err` and not a panic: module note § 3.
    if exit.p9 <= base.p_ambient + 1e-6 {
        return Err(Abort(format!(
            "rung-42 bleed match at Tt4={tt4:.0}, b={b:.3}, M0={:.2}: nozzle UNCHOKED -- OUT OF \
             SCOPE (docs/rung38-spec.md 'Scope'). Opening the valve shrinks the choked envelope; \
             the LP spool's own subsonic branch is still a follow-on.", flight.m0)));
    }

    // --- the SEPARATE result assembly (module note § 1) -----------------------------------
    let stations = vec![
        ("0", state0), ("2", s2), ("25", s25), ("3", s3), ("4", s4),
        ("45", s45), ("5", s5), ("9", exit.state),
    ];
    let perf = try_score(&rgas, &stations, v0, exit.t9, exit.v9, exit.p9, flight.p0,
                         rgas.hpr())?;
    // (3) THRUST. The dumped air was captured, so it carries FULL ram drag and returns no exhaust
    // momentum (an overboard dump with no recovery — the conservative reading; a real duct into
    // the nacelle/bypass recovers some). Per unit INLET air:
    //     F/mdot_2 = (1-b)*[(1+f)V9 + pressure - V0] - b*V0
    let st_inlet = (1.0 - b) * perf.specific_thrust - b * v0;
    let thrust = mdot_air * st_inlet;

    let base_result = TwoSpoolResult {
        stations, performance: perf, v0, v9: exit.v9, m9: exit.m9, t9: exit.t9, p9: exit.p9,
        thrust, tt4, m0: flight.m0, pi_lpc: c.c.pi_lpc, pi_hpc: c.c.pi_hpc,
        tau_lpc: s25.tt / s2.tt, tau_hpc: s3.tt / s25.tt,
        tau_hpt: c.c.tau_hpt, pi_hpt: c.c.pi_hpt, tau_lpt: c.c.tau_lpt, pi_lpt: c.c.pi_lpt,
        mdot_air, mdot_ratio: mdot_air / base.mdot_air_design,
    };
    Ok(TwoSpoolBleedResult {
        base: TwoSpoolMapResult {
            base: base_result,
            eta_lpc: c.eta_lpc, eta_hpc: c.eta_hpc, eta_hpt: c.eta_hpt, eta_lpt: c.eta_lpt,
            n_lp: c.n_l, n_hp: c.n_h, n_lp_ratio: c.nl, n_hp_ratio: c.nh, slip: c.slip,
            phi_lp: c.phi_l, phi_hp: c.phi_h, nu_hpt: c.nu_hpt, nu_lpt: c.nu_lpt,
        },
        booking: Some(BleedBooking {
            bleed: b, mdot_core, st_inlet,
            tsfc_inlet: (1.0 - b) * s4.far / st_inlet,
        }),
    })
}

// =========================================================================================
// THE MATCHER
// =========================================================================================

/// RUNG 42. Two-spool map matching WITH an interstage (station-25) bleed valve.
///
/// ```text
/// let m = TwoSpoolBleedMatcher::new(design, flight, 1.0, map_lp, map_hp, 0.08);
/// let od = m.match_point(&flight, 1200.0);        // -> TwoSpoolBleedResult
/// ```
pub struct TwoSpoolBleedMatcher {
    /// The rung-39 core, carrying [`R42`] in its hook slot and the valve setting in
    /// [`TwoSpoolMapCore::bleed`]. `pub` so rung 41's six diagnostics — which live on the core —
    /// are reachable, which is the whole point of the dispatch: `bleed_trade` → `surge_margin`
    /// (rung 41's code) → `self.match` → back here.
    pub core: TwoSpoolMapCore,
}

impl TwoSpoolBleedMatcher {
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, bleed: f64,
    ) -> Self {
        let mut core = TwoSpoolMapCore::with_hooks(
            design_engine, flight_design, mdot_design, map_lp, map_hp, &R42);
        core.bleed = bleed;
        // The one rung-42 guard measured to FIRE, so it stays an assert (the zero-firing rule
        // cuts the other way here).
        assert!((0.0..0.5).contains(&core.bleed),
                "rung-42 bleed fraction must be in [0, 0.5): b>=0.5 starves the core and the \
                 choked branch is long gone by then");
        TwoSpoolBleedMatcher { core }
    }

    pub fn bleed(&self) -> f64 { self.core.bleed }

    /// Move the valve on an EXISTING matcher — Python's plain `m.bleed = b`, which is what
    /// [`bleed_trade`](Self::bleed_trade) does internally.
    ///
    /// **No range re-check, deliberately**: Python's attribute assignment has none either, and
    /// the constructor's assert is the only place the source spends one. Rebuilding a matcher per
    /// bleed level would be the alternative, and it re-runs the whole design cycle — which is the
    /// reason `bleed_trade` mutates in the first place.
    pub fn set_bleed(&mut self, b: f64) { self.core.bleed = b; }

    /// Match one point, keeping the [`BleedBooking`]. Reaches rung 42's body directly rather than
    /// through the hook, because the hook's return type cannot carry the booking — the DISPATCH
    /// is still witnessed, by rung 41's methods on the core.
    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> TwoSpoolBleedResult {
        self.try_match_point(flight, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin of [`match_point`](Self::match_point) — see [`Abort`].
    ///
    /// [`Abort`]: crate::gas::Abort
    pub fn try_match_point(
        &self, flight: &FlightCondition, tt4: f64,
    ) -> Result<TwoSpoolBleedResult, Abort> {
        try_match_bleed(&self.core, flight, tt4)
    }

    /// Open the valve at a FIXED throttle and read what moves — **THE CONTROLLED COMPARISON.**
    ///
    /// `bleed` sets `b`, not the throttle, so the clean "open the valve, nothing else moves"
    /// reading holds `Tt4`. Comparing at fixed THRUST instead folds in a throttle change and mixes
    /// the device's effect with the running line's — a different, and separately reported,
    /// question.
    ///
    /// One row per `b`, with both flow coefficients, both margins (**only** when both maps carry a
    /// surge floor — the same predicate that panics on an unarmed pair, so that divergence does
    /// not widen here) and the thrust/TSFC trade, all at the same `Tt4`. Each armed row costs TWO
    /// matches: one for the row, and one inside `surge_margin`.
    ///
    /// `&mut self` because Python mutates `self.bleed` and restores it in a `finally`. A panic
    /// mid-sweep leaves the field moved, where Python would restore it — the difference is
    /// unobservable, since an uncaught panic ends the run either way.
    pub fn bleed_trade(
        &mut self, flight: &FlightCondition, tt4: f64, bleeds: &[f64],
    ) -> Vec<BleedTradeRow> {
        let b_save = self.core.bleed;
        let mut out = Vec::new();
        for &b in bleeds {
            // Deliberately NOT re-asserting the constructor's range: Python's `bleed_trade`
            // assigns `self.bleed` directly and never re-checks it either.
            self.core.bleed = b;
            let od = self.match_point(flight, tt4);
            // The `getattr` fallback, as a type: at b = 0 there is no booking and the CORE
            // numbers are what Python reads.
            let (st_inlet, tsfc) = match od.booking {
                Some(k) => (k.st_inlet, k.tsfc_inlet),
                None => (od.base.base.performance.specific_thrust,
                         od.base.base.performance.tsfc),
            };
            let (mut sm_lp, mut sm_hp) = (None, None);
            if self.core.map_lp().phi_surge > 0.0 && self.core.map_hp().phi_surge > 0.0 {
                let sm = self.core.surge_margin(flight, tt4);
                sm_lp = Some(sm.sm_lp);
                sm_hp = Some(sm.sm_hp);
            }
            out.push(BleedTradeRow {
                bleed: b, tt4,
                phi_lp: od.base.phi_lp, phi_hp: od.base.phi_hp,
                n_lp: od.base.n_lp, n_hp: od.base.n_hp,
                pi_lpc: od.base.base.pi_lpc, pi_hpc: od.base.base.pi_hpc,
                tt25: od.base.base.station("25").tt, slip: od.base.slip,
                mdot_air: od.base.base.mdot_air, thrust: od.base.base.thrust,
                st_inlet, tsfc, sm_lp, sm_hp,
            });
        }
        self.core.bleed = b_save;
        out
    }
}

/// One row of [`TwoSpoolBleedMatcher::bleed_trade`].
#[derive(Clone, Copy, Debug)]
pub struct BleedTradeRow {
    pub bleed: f64,
    pub tt4: f64,
    pub phi_lp: f64,
    pub phi_hp: f64,
    pub n_lp: f64,
    pub n_hp: f64,
    pub pi_lpc: f64,
    pub pi_hpc: f64,
    pub tt25: f64,
    pub slip: f64,
    pub mdot_air: f64,
    pub thrust: f64,
    /// `F / mdot_INLET` — the booking's when the valve is open, the CORE specific thrust when it
    /// is shut. Python's `getattr(od, "st_inlet", od.performance.specific_thrust)`.
    pub st_inlet: f64,
    /// Likewise `tsfc_inlet` or the core `tsfc`.
    pub tsfc: f64,
    /// `None` unless BOTH maps carry a surge floor — Python omits the key entirely there.
    pub sm_lp: Option<f64>,
    pub sm_hp: Option<f64>,
}
