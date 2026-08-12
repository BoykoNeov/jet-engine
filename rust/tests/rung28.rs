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
