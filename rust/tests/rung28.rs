//! Rung-28 verification: THE COUPLED NO MARCH — rung 27's clock on rung 26's relaxing pool.
//!
//! Rung 27 read its NO clock on the FROZEN station-4 pool and deferred the coupled march with the
//! note that it "can ONLY slow NO further (radical-poorer ⇒ larger τ_NO)". Rung 28 builds it and
//! finds that "only" was ONE-SIDED: coupling to rung 26 couples to ALL of rung 26, including its
//! exothermic heat release, which lifts `T(s)` above the frozen isentrope — and because the NO
//! clock is Arrhenius, that SPEEDS destruction. Two opposing channels.
//!
//! THE VERDICT is a CONFIRMATION with a MECHANISTIC CORRECTION: the conclusion holds
//! (`net_factor < 1`), the mechanism was incomplete (`heat_release_factor > 1` everywhere), the
//! win is STRUCTURAL rather than incidental (channel 1 is unbounded, channel 2 saturates), and the
//! HEADLINE IS UNTOUCHED — the entry state is path-independent, so `da_entry` is rung 27's
//! bit-for-bit.
//!
//! **THE TWO GATES HERE THE ORACLE CANNOT SUPPLY.**
//!
//! 1. **The STRUCTURAL reduce** — feeding the frozen trajectory to `coupled_no_march` reproduces
//!    `no_freeze_out_expand` bit-for-bit. A Python↔Rust dump compares values and is blind to a
//!    loop-shape error transcribed identically into both copies; this is not. § 4.13 prediction 2
//!    registered it as COPY-class and therefore predicted to SURVIVE *before* it was measured, on
//!    slice F's discriminator — and it did, 10/10.
//! 2. **The β repair's fallback, from the REFUSING side.** The shipped code ends its β sweep with
//!    "if the ratio is not finite, call it 1.0", and that branch is DORMANT at every shipped
//!    condition (0 of 55 sampled cells). A guard gated only from the accepting side proves
//!    nothing — the rung-20 gate-5 lesson — so this suite reaches it directly.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, Gas};
use turbojet::march::{
    coupled_no_march, frozen_no_trajectory, no_freeze_out_expand, tau_no_destroy, tau_no_exact,
    CoupledNoFreezeOut, CoupledNoFreezeOutState, NoFreezeOut,
};
use turbojet::nox::ZonedNoxOpts;

const PI_C: f64 = 10.0;
const PHI_P: f64 = 1.0;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses() -> Losses {
    Losses {
        pi_d: 0.97,
        eta_c: 0.88,
        eta_b: 0.99,
        pi_b: 0.96,
        eta_t: 0.90,
        eta_m: 0.99,
        pi_n: 0.98,
        ..Losses::default()
    }
}

struct Dp {
    gas: Gas,
    far: f64,
    tt3: f64,
    tt4: f64,
    pt4: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
}

fn dp(tt4: f64) -> Dp {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, 50_000.0, losses());
    let r = eng.run(&flight(), 1.0);
    let (s3, s4, s9) = (r.station("3"), r.station("4"), r.station("9"));
    Dp {
        far: s4.far,
        tt3: s3.tt,
        tt4: s4.tt,
        pt4: s4.pt,
        tt9: s9.tt,
        pt9: s9.pt,
        p9: r.p9,
        gas: eng.gas,
    }
}

fn cpl(d: &Dp, cfg: CoupledNoFreezeOut, couple: bool) -> CoupledNoFreezeOutState {
    d.gas.coupled_no_freeze_out_nozzle(
        d.far, d.tt3, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, PHI_P, cfg, couple,
    )
}

fn entry(d: &Dp) -> Vec<(&'static str, f64)> {
    equilibrium_composition(d.far, d.tt4, d.pt4)
}

// --- GATE 1: THE STRUCTURAL REDUCE — a RUST-vs-RUST gate the oracle cannot give --------------- //

/// Feed [`frozen_no_trajectory`] to [`coupled_no_march`] and it reproduces
/// [`no_freeze_out_expand`] BIT-FOR-BIT — the two march the identical expression sequence.
///
/// § 4.13 registered this as COPY-class and therefore predicted it would SURVIVE, using slice F's
/// discriminator (an "exactly" claim survives a copy and dies on a rederivation). It survives
/// despite an asymmetry that looks like it should break it: rung 27 computes
/// `equilibrium_no_fraction` ONCE per step and uses the value twice, while rung 28 calls it TWICE.
/// Same function, same arguments, same bits — **a COPY is about the arithmetic performed, not the
/// syntax.**
///
/// 20 cells against the Python suite's 4.
#[test]
fn the_structural_reduce_is_bit_for_bit() {
    let mut cells = 0usize;
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let ce = entry(&d);
        let zn = d.gas.zoned_nox(d.far, d.tt3, d.tt4, d.pt4, PHI_P, ZonedNoxOpts::default());
        let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, Some(zn.x_no_mix));
        let tau_res = 0.5 / (0.6 * nf.v9_frozen);
        let da_no = move |comp: &[(&'static str, f64)], t: f64, p: f64| {
            tau_res / tau_no_destroy(comp, t, p, None, None)
        };
        for nstep in [100usize, 400] {
            let a = no_freeze_out_expand(&ce, d.tt9, d.pt9, d.p9, zn.x_no_mix, &da_no, nstep);
            let traj = frozen_no_trajectory(&ce, d.tt9, d.pt9, d.p9, nstep);
            let b = coupled_no_march(&traj, &traj, zn.x_no_mix, &da_no);
            let names = ["T9", "x_no", "x_no_e_exit", "max_a", "Da_entry", "Da_exit"];
            let av = [a.0, a.1, a.2, a.3, a.4, a.5];
            let bv = [b.0, b.1, b.2, b.3, b.4, b.5];
            for i in 0..6 {
                assert_eq!(
                    av[i].to_bits(),
                    bv[i].to_bits(),
                    "{} not bit-for-bit at Tt4={tt4}, nstep={nstep}: {} vs {}",
                    names[i],
                    av[i],
                    bv[i]
                );
            }
            cells += 1;
        }
    }
    assert_eq!(cells, 10, "the reduce swept {cells} cells, expected 10");
}

/// …and the same through the PUBLIC method: `couple = false` IS rung 27, so its entry clock and
/// clamp must equal `no_freeze_out_nozzle`'s exactly.
#[test]
fn uncoupled_is_rung27_through_the_public_method() {
    for tt4 in [1500.0, 2200.0] {
        let d = dp(tt4);
        let r27 = d.gas.no_freeze_out_nozzle(
            d.far,
            d.tt3,
            d.tt4,
            d.pt4,
            d.tt9,
            d.pt9,
            d.p9,
            PHI_P,
            NoFreezeOut::default(),
        );
        let un = cpl(&d, CoupledNoFreezeOut::default(), false);
        assert_eq!(un.da_entry.to_bits(), r27.da_entry.to_bits());
        assert_eq!(un.max_a.to_bits(), r27.max_a.to_bits());
        assert_eq!(un.max_a_frozen.to_bits(), r27.max_a_frozen.to_bits());
        assert_eq!(un.x_no_relaxed.to_bits(), r27.x_no_relaxed.to_bits());
    }
}

// --- GATE 2: THE HEADLINE IS UNTOUCHED — the entry is PATH-INDEPENDENT ------------------------ //

/// The nozzle-entry state does not depend on which trajectory the clock reads, so `da_entry` must
/// be identical coupled and uncoupled — bit-for-bit, not to a tolerance. That is what keeps rung
/// 27's "frozen from entry" verdict intact under the whole rung-28 correction.
#[test]
fn the_entry_clock_is_path_independent() {
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let c = cpl(&d, CoupledNoFreezeOut::default(), true);
        let u = cpl(&d, CoupledNoFreezeOut::default(), false);
        assert_eq!(c.da_entry.to_bits(), u.da_entry.to_bits(), "entry moved at Tt4={tt4}");
        assert!(c.frozen_from_entry(), "NO not frozen from entry at Tt4={tt4}");
        assert!(c.da_entry < 1.0);
    }
}

// --- GATE 3: THE TWO CHANNELS — depletion slows, heat release SPEEDS -------------------------- //

/// Rung 27's deferred prediction said coupling can "ONLY" slow NO. Both channels are measured
/// separately here, and the second one goes the other way at every design point — which is the
/// correction. The NET still lands below 1, which is the confirmation.
#[test]
fn the_two_channels_oppose_and_the_net_is_still_deeper_frozen() {
    for tt4 in [1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let c = cpl(&d, CoupledNoFreezeOut::default(), true);
        assert!(
            c.depletion_factor() < 1.0,
            "channel 1 must SLOW the clock at Tt4={tt4}: {}",
            c.depletion_factor()
        );
        assert!(
            c.heat_release_factor() > 1.0,
            "channel 2 must SPEED it — the correction — at Tt4={tt4}: {}",
            c.heat_release_factor()
        );
        assert!(c.net_factor() < 1.0, "the net must stay deeper frozen at Tt4={tt4}");
        assert!(c.deeper_frozen());
        // …and the radicals really are depleted, which is what channel 1 rides on.
        assert!(c.x_radical_exit_pool < c.x_radical_entry);
        // …while the pool exit is WARMER than the frozen one, which is what channel 2 rides on.
        assert!(c.t9_pool > c.t9_frozen);
    }
}

/// The `channel_ratio` — how much of the depletion the heat release cancels — RISES with `Tt4`.
/// That monotone trend is the certified claim; the net's turnaround is explicitly not.
#[test]
fn the_channel_ratio_rises_with_tt4() {
    let ratios: Vec<f64> = [1500.0, 1800.0, 2200.0, 2300.0]
        .iter()
        .map(|&tt4| cpl(&dp(tt4), CoupledNoFreezeOut::default(), true).channel_ratio())
        .collect();
    assert!(
        ratios.windows(2).all(|w| w[0] < w[1]),
        "channel_ratio not rising with Tt4: {ratios:?}"
    );
}

/// The win is STRUCTURAL: channel 1 is UNBOUNDED (it keeps growing as the pool equilibrates) while
/// channel 2 SATURATES (it is capped by the finite frozen-in chemical enthalpy). So at any
/// chemistry faster than anchored, depletion wins decisively — driven by `pool_rate_scale`.
#[test]
fn depletion_wins_decisively_at_a_faster_pool() {
    let d = dp(2200.0);
    let base = cpl(&d, CoupledNoFreezeOut::default(), true);
    let fast = cpl(&d, CoupledNoFreezeOut { pool_rate_scale: 1e6, ..Default::default() }, true);
    assert!(
        fast.depletion_factor() < base.depletion_factor(),
        "a faster pool must deplete harder: {} vs {}",
        fast.depletion_factor(),
        base.depletion_factor()
    );
    assert!(fast.net_factor() < base.net_factor(), "the net must go deeper frozen");
}

// --- GATE 4: THE β REPAIR — and its DORMANT fallback, from the refusing side ------------------ //

/// The surrogate bounds the true rate along the whole path: `β < 1` and `τ_exact/τ_surrogate ≥ 1`
/// pointwise. That is the justification rung 27's (entry-false) super-equilibrium premise owed.
#[test]
fn the_surrogate_bounds_the_rate_along_the_path() {
    let mut betas: Vec<f64> = Vec::new();
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let c = cpl(&dp(tt4), CoupledNoFreezeOut::default(), true);
        assert!(c.surrogate_bounds_rate(), "the bound fails at Tt4={tt4}");
        assert!(c.beta_max < 1.0, "beta_max={} at Tt4={tt4}", c.beta_max);
        assert!(c.tau_ratio_min >= 1.0, "tau_ratio_min={} at Tt4={tt4}", c.tau_ratio_min);
        betas.push(c.beta_max);
    }
    // β RISES with Tt4 and reaches about half the β = 1 threshold — a factor, not orders, which is
    // the honest weak point the source discloses rather than a margin it claims.
    assert!(betas.windows(2).all(|w| w[0] < w[1]), "beta_max not rising with Tt4: {betas:?}");
    assert!(betas[4] > 0.5 && betas[4] < 0.6, "hot beta_max out of the disclosed band: {}", betas[4]);
}

/// NO arrives SUB-equilibrium at the entry and leaves SUPER-equilibrium — the erratum's point,
/// and the reason rung 27's stated premise needed replacing even though its numbers held.
#[test]
fn no_arrives_sub_equilibrium_and_leaves_super_equilibrium() {
    for tt4 in [1800.0, 2200.0, 2300.0] {
        let c = cpl(&dp(tt4), CoupledNoFreezeOut::default(), true);
        assert!(c.sub_equilibrium_entry(), "a_entry={} at Tt4={tt4} is not < 1", c.a_entry);
        assert!(c.a_exit > 1.0, "a_exit={} at Tt4={tt4} is not > 1", c.a_exit);
        assert!(c.clamp_fires());
    }
}

/// **THE DORMANT FALLBACK, REACHED.**
///
/// The shipped β sweep ends with "if the ratio is not finite, call it 1.0", and that branch is
/// dormant at every shipped condition — 0 of 55 sampled cells, and `tau_no_exact` returns finite
/// even at 400 K, which was the obvious candidate. A guard gated only from the ACCEPTING side
/// proves nothing (the rung-20 gate-5 lesson), so this reaches the degenerate return directly by
/// removing the radicals the rate needs.
#[test]
fn the_degenerate_clock_branch_is_reachable_and_returns_the_sentinel() {
    let d = dp(2200.0);
    let ce = entry(&d);

    // Live, at a real path state: finite, with a real β and a real `a`.
    let (tau, beta, a) = tau_no_exact(&ce, 1400.0, 5.0e4, 1e-3);
    assert!(tau.is_finite() && beta > 0.0 && a > 0.0);

    // Degenerate: no radicals ⇒ R1 = 0 and R2+R3 = 0 ⇒ the sentinel.
    let no_rad: Vec<(&'static str, f64)> =
        ce.iter().map(|&(sp, n)| (sp, if sp == "O" || sp == "H" { 0.0 } else { n })).collect();
    let (tau_d, beta_d, a_d) = tau_no_exact(&no_rad, 1400.0, 5.0e4, 1e-3);
    assert!(tau_d.is_infinite(), "the degenerate branch must return +inf, got {tau_d}");
    assert_eq!((beta_d, a_d), (0.0, 0.0));

    // …and the empty mixture takes the OTHER degenerate exit (`ntot <= 0`), which is a different
    // branch of the same function and would otherwise never be reached.
    let empty: Vec<(&'static str, f64)> = ce.iter().map(|&(sp, _)| (sp, 0.0)).collect();
    let (tau_e, _, _) = tau_no_exact(&empty, 1400.0, 5.0e4, 1e-3);
    assert!(tau_e.is_infinite());

    // The surrogate degenerates on the same input, which is what makes `Da_NO → 0` (frozen) the
    // physical limit rather than a division by zero.
    assert!(tau_no_destroy(&no_rad, 1400.0, 5.0e4, None, None).is_infinite());
}

// --- GATE 5: THE INTERLOCK, CYCLE UNTOUCHED, and GUARDS --------------------------------------- //

/// The coupling is gated by rung 26's own pool freeze point: where the pool never relaxes, the
/// coupled and uncoupled clocks must agree exactly, because there is nothing to couple TO.
#[test]
fn the_pool_freeze_point_gates_the_coupling() {
    // Lean: rung 26's pool is frozen from entry (`s_freeze` = 0), so coupling changes nothing
    // about the ENTRY, and the exit factors stay within a hair of 1.
    let cold = cpl(&dp(1300.0), CoupledNoFreezeOut::default(), true);
    assert_eq!(cold.s_freeze_pool, 0.0);
    assert!((cold.net_factor() - 1.0).abs() < 0.05, "net={} on a frozen pool", cold.net_factor());
    // Hot: the pool relaxes partway, and the coupling then bites.
    let hot = cpl(&dp(2200.0), CoupledNoFreezeOut::default(), true);
    assert!(hot.s_freeze_pool > 0.0);
    assert!(hot.net_factor() < 0.95, "net={} on a relaxing pool", hot.net_factor());
}

// ============================================================================================= //
// THE β-MARGIN FAMILY — rung 28's own seam, re-checked, and the gates the first pass of this port
// missed.
//
// The same coverage lesson slice H's rung 29 taught: the oracle read 776/776 and this file still
// held 14 of the source's 20 gates. **An oracle gates VALUES; a missing gate is a missing CLAIM.**
// Enumerated with `grep "^def test" tests/test_rung28.py` and diffed, which is the only detector.
// ============================================================================================= //

const BAND: [f64; 6] = [1500.0, 1650.0, 1800.0, 2000.0, 2200.0, 2400.0];

/// `beta_max` at an arbitrary `(Tt4, π_c)`, for the plane sweeps.
fn beta_at(tt4: f64, pi_c: f64) -> f64 {
    let eng = build_turbojet(Gas::reacting_equilibrium(), pi_c, tt4, 50_000.0, losses());
    let r = eng.run(&flight(), 1.0);
    let (s3, s4, s9) = (r.station("3"), r.station("4"), r.station("9"));
    eng.gas
        .coupled_no_freeze_out_nozzle(
            s4.far,
            s3.tt,
            s4.tt,
            s4.pt,
            s9.tt,
            s9.pt,
            r.p9,
            PHI_P,
            CoupledNoFreezeOut::default(),
            true,
        )
        .beta_max
}

/// The frozen trajectory must BE rung-27's own path: `k = 0` is `Tt9` exactly (no bisection at the
/// entry), the composition is the same object at every station, and the path cools monotonically.
#[test]
fn the_frozen_trajectory_matches_the_rung27_path() {
    let d = dp(2200.0);
    let ce = entry(&d);
    let traj = frozen_no_trajectory(&ce, d.tt9, d.pt9, d.p9, 400);
    assert_eq!(traj.len(), 401);
    assert_eq!(traj[0].p.to_bits(), d.pt9.to_bits(), "entry pressure must be pt9 exactly");
    assert_eq!(traj[0].t.to_bits(), d.tt9.to_bits(), "entry T must be Tt9 exactly, not a bisection");
    assert_eq!(traj[400].p.to_bits(), d.p9.to_bits());
    for station in &traj {
        assert_eq!(station.comp.len(), ce.len());
        for (&(s1, n1), &(s2, n2)) in station.comp.iter().zip(ce.iter()) {
            assert_eq!(s1, s2);
            assert_eq!(n1.to_bits(), n2.to_bits(), "frozen: the composition must not move");
        }
    }
    assert!(
        traj.windows(2).all(|w| w[0].t > w[1].t),
        "the frozen path must cool monotonically"
    );
}

/// `rate_scale → 0` leaves NO where it entered, so the clamp is the rung-14/17 number bit-for-bit.
#[test]
fn no_rate_off_recovers_the_clamp() {
    for tt4 in [1500.0, 2200.0] {
        let off =
            cpl(&dp(tt4), CoupledNoFreezeOut { rate_scale: 1e-30, ..Default::default() }, true);
        assert_eq!(off.x_no_relaxed.to_bits(), off.x_no_frozen.to_bits());
        assert_eq!(off.max_a.to_bits(), off.max_a_frozen.to_bits());
        assert_eq!(off.relaxed_fraction(), 0.0);
    }
}

/// **The algebra the whole bound rests on**, checked pointwise rather than asserted:
/// `τ_exact/τ_surrogate = (1+u)²/[(1+u)² − (1−β²)]` with `u = βa`.
///
/// This is the non-tautological arm of the β repair — a closed form derived by hand against two
/// independently computed relaxation times. Also the one place rung 28's `(1+βa)²` integer power
/// is checked against a formula rather than against the Python.
#[test]
fn the_exact_tau_ratio_matches_the_closed_form() {
    let d = dp(2200.0);
    let ce = entry(&d);
    let zn = d.gas.zoned_nox(d.far, d.tt3, d.tt4, d.pt4, PHI_P, ZonedNoxOpts::default());
    let traj = frozen_no_trajectory(&ce, d.tt9, d.pt9, d.p9, 400);
    let mut sampled = 0usize;
    for station in traj.iter().step_by(40) {
        let (tau_e, beta, a_loc) = tau_no_exact(&station.comp, station.t, station.p, zn.x_no_mix);
        let tau_s = tau_no_destroy(&station.comp, station.t, station.p, None, None);
        let u = beta * a_loc;
        let one_plus_u = 1.0 + u;
        let expect = (one_plus_u * one_plus_u) / ((one_plus_u * one_plus_u) - (1.0 - beta * beta));
        let got = tau_e / tau_s;
        assert!(
            (got - expect).abs() <= 1e-9 * expect.abs(),
            "closed form mismatch at T={:.0}: {got} vs {expect}",
            station.t
        );
        assert!(tau_e >= tau_s, "the bound must hold POINTWISE at T={:.0}", station.t);
        sampled += 1;
    }
    assert!(sampled >= 10, "only {sampled} path samples");
}

/// β RISES with `Tt4` and reaches about half the `β = 1` threshold — the honest weak point, gated
/// so it cannot silently drift into a violation.
///
/// **And it FORBIDS the false comfort that β plateaus.** On a fixed mixture β climbs without limit
/// and crosses 1 off-cycle near 3200 K, so the margin is a TEMPERATURE HEADROOM rather than a
/// ceiling — which is a much weaker and more honest claim than "it saturates around 0.5".
#[test]
fn the_beta_margin_is_disclosed_not_comfortable() {
    let betas: Vec<f64> =
        BAND.iter().map(|&tt4| cpl(&dp(tt4), CoupledNoFreezeOut::default(), true).beta_max).collect();
    assert!(betas[0] < 0.15, "beta small lean, got {}", betas[0]);
    assert!(betas[5] > 0.3, "beta must be materially larger hot: {}", betas[5]);
    assert!(
        betas.iter().cloned().fold(f64::MIN, f64::max) < 0.6,
        "beta must stay under the measured plane bound: {betas:?}"
    );

    // NOT a plateau: on a FIXED mixture beta climbs monotonically and passes 1 off-cycle.
    let d = dp(2200.0);
    let ce = entry(&d);
    let seq: Vec<f64> = [1600.0, 2000.0, 2400.0, 2800.0, 3200.0]
        .iter()
        .map(|&t| tau_no_exact(&ce, t, d.pt9, 1e-4).1)
        .collect();
    assert!(seq.windows(2).all(|w| w[0] < w[1]), "beta must climb monotonically in T: {seq:?}");
    assert!(seq[4] > 1.0, "beta must exceed 1 off-cycle (~3200 K) — not a plateau: {}", seq[4]);
}

/// The whole-runnable-plane bound on β, and that its maximum is INTERIOR rather than a scan edge.
///
/// β is non-monotone in BOTH axes and turns over below `π_c ≈ 8` — as `π_c` falls, `far` rises
/// (pushing β down) while `Tt9` rises (pushing it up), and the composition channel wins at low
/// `π_c`. So the ridge must strictly beat BOTH flanks, which is what makes the quoted bound a
/// maximum rather than wherever the scan happened to stop.
#[test]
fn the_beta_plane_maximum_is_interior() {
    let ridge: Vec<f64> = [(2300.0, 8.0), (2325.0, 10.0)].iter().map(|&(t, p)| beta_at(t, p)).collect();
    let low: Vec<f64> = [(2300.0, 4.0), (2200.0, 4.0), (2300.0, 6.0)]
        .iter()
        .map(|&(t, p)| beta_at(t, p))
        .collect();
    let high: Vec<f64> = [(2300.0, 13.0), (2400.0, 20.0), (2450.0, 25.0)]
        .iter()
        .map(|&(t, p)| beta_at(t, p))
        .collect();
    let mx = |v: &[f64]| v.iter().cloned().fold(f64::MIN, f64::max);
    let mn = |v: &[f64]| v.iter().cloned().fold(f64::MAX, f64::min);
    let peak = mx(&ridge).max(mx(&low)).max(mx(&high));
    assert!(peak < 0.6, "beta must stay under the quoted plane bound: {peak}");
    assert!(peak > 0.5, "the ridge must actually be sampled: {peak}");
    assert_eq!(peak.to_bits(), mx(&ridge).to_bits(), "the max must sit ON the ridge");
    assert!(mx(&low) < mn(&ridge), "the ridge must beat the LOW-pi_c flank: {ridge:?} vs {low:?}");
    assert!(mx(&high) < mn(&ridge), "the ridge must beat the high flank: {ridge:?} vs {high:?}");
}

/// **β is EXACTLY pressure-invariant**, so `π_c` has no DIRECT channel into the bound at all.
///
/// `R1`, `R2` and `R3` are each a product of two concentrations, so `c_tot²` cancels top and
/// bottom and β reduces to mole fractions and T-only rate constants. Gated over a **640× pressure
/// span** at `rel_tol = 1e-12` — a claim of exactness that a tolerance three orders looser would
/// not distinguish from "roughly flat".
#[test]
fn beta_is_exactly_pressure_invariant() {
    let d = dp(2200.0);
    let ce = entry(&d);
    let reference = tau_no_exact(&ce, d.tt9, d.pt9, 1e-4).1;
    assert!(reference > 0.0 && reference < 1.0);
    for scale in [0.25, 4.0, 40.0, 160.0] {
        let beta = tau_no_exact(&ce, d.tt9, d.pt9 * scale, 1e-4).1;
        assert!(
            (beta - reference).abs() <= 1e-12 * reference,
            "beta must be pressure-invariant; at {scale}x p got {beta} vs {reference}"
        );
    }
}

/// The seam rung 28 filed was "β at higher `π_c`", and the answer INVERTS the worry: both of
/// `π_c`'s indirect channels push β DOWN, so a higher-pressure cycle is PROTECTIVE. Entry `Da_NO`
/// falls on the same axis, so rung 27's verdict hardens there too.
#[test]
fn beta_falls_with_pressure_ratio() {
    let mut out: Vec<(f64, f64)> = Vec::new();
    for pi_c in [10.0, 40.0] {
        let eng = build_turbojet(Gas::reacting_equilibrium(), pi_c, 2200.0, 50_000.0, losses());
        let r = eng.run(&flight(), 1.0);
        let (s3, s4, s9) = (r.station("3"), r.station("4"), r.station("9"));
        let cp = eng.gas.coupled_no_freeze_out_nozzle(
            s4.far,
            s3.tt,
            s4.tt,
            s4.pt,
            s9.tt,
            s9.pt,
            r.p9,
            PHI_P,
            CoupledNoFreezeOut::default(),
            true,
        );
        out.push((cp.beta_max, cp.da_entry));
    }
    assert!(out[1].0 < out[0].0, "beta must FALL with pi_c: {} → {}", out[0].0, out[1].0);
    assert!(out[1].0 < 0.8 * out[0].0, "…and materially so: {} → {}", out[0].0, out[1].0);
    assert!(out[1].1 < out[0].1, "entry Da_NO must fall with pi_c too");
    assert!(out[1].1 < 1.0, "still frozen from entry at high pi_c");
}

/// **The Da ratios are the CLOCK's depth, not NO's motion.** `relaxed_fraction` stays ≈ 0 across
/// the band — slightly NEGATIVE hot, which is the sub-equilibrium entry causing a tiny FORMATION
/// drift — and the clamp is unmoved from rung 14/17's number.
#[test]
fn no_barely_moves_despite_the_da_ratios() {
    for tt4 in BAND {
        let cp = cpl(&dp(tt4), CoupledNoFreezeOut::default(), true);
        assert!(
            cp.relaxed_fraction().abs() < 1e-2,
            "NO must stay frozen at Tt4={tt4}: {}",
            cp.relaxed_fraction()
        );
        assert!(
            (cp.max_a - cp.max_a_frozen).abs() <= 1e-2 * cp.max_a_frozen,
            "the clamp must be unmoved at Tt4={tt4}"
        );
        assert!(cp.clamp_fires(), "the clamp must still fire at Tt4={tt4}");
    }
}

/// Depletion wins at EVERY `Tt4` once the pool chemistry runs faster than anchored — the
/// structural claim, not a single-point one.
#[test]
fn depletion_wins_at_every_tt4_in_the_limit() {
    for tt4 in [1800.0, 2000.0, 2200.0, 2400.0] {
        let d = dp(tt4);
        let fast =
            cpl(&d, CoupledNoFreezeOut { pool_rate_scale: 1e6, ..Default::default() }, true);
        assert!(
            fast.net_factor() < 1.0,
            "a fast pool must drive the clock DEEPER frozen at Tt4={tt4}: {}",
            fast.net_factor()
        );
        assert!(fast.depletion_factor() < 1.0);
    }
}

#[test]
fn cycle_untouched() {
    let d = dp(2200.0);
    let far_before = d.far;
    let _ = cpl(&d, CoupledNoFreezeOut::default(), true);
    let r = build_turbojet(Gas::reacting_equilibrium(), PI_C, 2200.0, 50_000.0, losses())
        .run(&flight(), 1.0);
    assert_eq!(r.station("4").far.to_bits(), far_before.to_bits());
}

#[test]
#[should_panic(expected = "too coarse")]
fn guard_nstep_below_100_is_refused() {
    CoupledNoFreezeOut { nstep: 99, ..Default::default() }.validate();
}

#[test]
#[should_panic(expected = "pool_rate_scale=0 must be positive")]
fn guard_pool_rate_scale_must_be_positive() {
    CoupledNoFreezeOut { pool_rate_scale: 0.0, ..Default::default() }.validate();
}

#[test]
#[should_panic(expected = "needs the rung-6 equilibrium gas")]
fn guard_requires_the_equilibrium_gas() {
    let d = dp(2200.0);
    Gas::thermally_perfect().coupled_no_freeze_out_nozzle(
        d.far,
        d.tt3,
        d.tt4,
        d.pt4,
        d.tt9,
        d.pt9,
        d.p9,
        PHI_P,
        CoupledNoFreezeOut::default(),
        true,
    );
}

#[test]
#[should_panic(expected = "exceeds pt9")]
fn guard_rejects_back_pressure_above_total() {
    let d = dp(2200.0);
    cpl_bad(&d);
}

fn cpl_bad(d: &Dp) {
    d.gas.coupled_no_freeze_out_nozzle(
        d.far,
        d.tt3,
        d.tt4,
        d.pt4,
        d.tt9,
        d.pt9,
        d.pt9 * 1.5,
        PHI_P,
        CoupledNoFreezeOut::default(),
        true,
    );
}
