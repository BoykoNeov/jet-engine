//! Rung 33 — THE SUBSONIC-NOZZLE MATCHING BRANCH (below the nozzle-unchoke boundary).
//!
//! Port of `tests/test_rung33.py` (phase 5 slice I).
//!
//! Rung 31 pinned the turbine by TWO choked throats, and (★) `pi_t/sqrt(tau_t) =
//! A4·MFP4/(A8·pi_n·MFP9)` is pure GEOMETRY — so `tau_t`, `pi_t` are constant on a
//! calorically-perfect gas, *"the turbine does not know the operating condition changed"*.
//! Below the nozzle-unchoke boundary that decoupling BREAKS: only the NGV stays choked and the
//! nozzle passes a SUBSONIC flow whose corrected throughput is `MFP(M9)` with `M9` set by the
//! ACTUAL ratio `pt9/p0` — which moves with `pi_c` as you throttle. So `pi_t` is no longer
//! geometry-pinned; it equilibrates the NGV-choked supply against the subsonic-nozzle demand.
//!
//! Gates (`docs/rung33-spec.md` § Verification gates):
//!
//! 1. **REDUCE / CHOKED BIT-FOR-BIT** — the choked path is left literally unchanged.
//! 2. **DISPATCH + BOUNDARY CONTINUITY** — choked above unchoke, subsonic below; `M9` passes
//!    through 1 continuously and `pi_c`, `tau_t` do not jump across it.
//! 3. **THE RUNG (CPG `tau_t` VARIES)** — on a calorically-perfect gas the subsonic `tau_t`
//!    varies with throttle (structural coupling through `pi_c`), the INVERSION of rung 31's
//!    machine-constant CPG `tau_t`.
//! 4. **NON-TAUTOLOGICAL ANCHOR** — the matched subsonic point satisfies textbook compressible
//!    flow to machine precision, solved a second and entirely separate way.
//! 5. **ENVELOPE** — monotone, bounded ABOVE by nozzle-unchoke and BELOW by thrust-neutral
//!    idle (SUB-IDLE raised, not force-fit).
//! 6. **HOMOGENEITY (the framing)** — scaling `p0` leaves the subsonic ratios invariant: the
//!    coupling is to `pi_c` via `pt9/p0`, NOT to the ambient pressure.
//! 7. **CYCLE UNTOUCHED** — the default design run is bit-for-bit rung 6.
//!
//! **GATE 7'S SECOND HALF IS DEFERRED TO SLICE J, IN WRITING.** The Python also asserts that
//! `MapMatcher` (rung 32) does NOT inherit the subsonic dispatch — that it flags
//! `nozzle_choked=False` and stays on its choked-only path, because subsonic + map is out of
//! scope. That is a claim ABOUT RUNG 32, and rung 32 does not exist in the Rust yet, so there
//! is nothing here for it to be true or false of. It is recorded at `slice_j_deferrals` below
//! rather than left to be noticed later — the port has already been bitten once by a
//! documented gate that did not exist (§ 4.16), and once by a census that swept only the phase
//! it was standing in.

use turbojet::components::ram_recovery;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{powp, Gas, GasSpec};
use turbojet::matcher::{Branch, OffDesignMatcher};

const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;
const ETA_C: f64 = 0.88;
const ETA_T: f64 = 0.90;
const ETA_B: f64 = 0.99;
const ETA_M: f64 = 0.99;
const PI_B: f64 = 0.96;
const PI_N: f64 = 0.98;
const PI_D: f64 = 0.97;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> Losses {
    Losses {
        pi_d: PI_D, eta_c: ETA_C, eta_b: ETA_B, pi_b: PI_B,
        eta_t: ETA_T, eta_m: ETA_M, pi_n: PI_N,
        nozzle_convergent: true,
        ..Losses::default()
    }
}

/// The SELF-CONSISTENT CPG dual gas — `R_t = (γ−1)/γ·cp_t` EXACTLY, which gate 4 depends on:
/// its independent solve is closed-form algebra, and the shipped solver's sonic throat equals
/// that closed form only when the constants satisfy the perfect-gas relation exactly.
fn cpg_gas() -> Gas {
    let (g, cp) = (1.3, 1239.0);
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: g, cp_t: cp, r_t: (g - 1.0) / g * cp,
        hpr: 42.8e6,
        ..GasSpec::default()
    })
}

fn matcher_at(gas: Gas, fl: FlightCondition, p0: f64) -> OffDesignMatcher {
    OffDesignMatcher::new(build_turbojet(gas, PI_C, TT4, p0, real()), fl, 1.0)
}

fn cpg_matcher() -> OffDesignMatcher {
    matcher_at(cpg_gas(), flight(), 50_000.0)
}

fn reacting_matcher() -> OffDesignMatcher {
    matcher_at(Gas::reacting_equilibrium(), flight(), 50_000.0)
}

// ------------------------------------------------------------------------------- gate 1
/// GATE 1 — the choked path is untouched: design reduces, choked points stay choked.
#[test]
fn gate1_reduce_choked_bitforbit() {
    let m = reacting_matcher();
    let od = m.match_point(&flight(), TT4);
    assert_eq!(od.branch, Branch::Choked);
    assert!(od.nozzle_choked);
    assert!((od.pi_c - PI_C).abs() < 1e-8, "pi_c did not reduce to design: {}", od.pi_c);
    // A mid-throttle choked point is still choked — dispatch only fires below unchoke.
    let mid = m.match_point(&flight(), 1000.0);
    assert_eq!(mid.branch, Branch::Choked);
    assert!(mid.nozzle_choked && mid.m9 > 1.0 - 1e-9);
}

// ------------------------------------------------------------------------------- gate 2
/// GATE 2 — choked above unchoke, subsonic below; `M9`/`pi_c`/`tau_t` continuous across it.
#[test]
fn gate2_dispatch_and_boundary_continuity() {
    let m = cpg_matcher();
    let mut prev: Option<turbojet::matcher::OffDesignResult> = None;
    let mut crossed = false;
    for tt4 in [700.0, 650.0, 620.0, 610.0, 600.0, 590.0, 580.0, 560.0] {
        let od = m.match_point(&flight(), tt4);
        if od.branch == Branch::Choked {
            assert!(od.nozzle_choked && (od.m9 - 1.0).abs() < 1e-6);
        } else {
            assert!(!od.nozzle_choked && od.m9 < 1.0);
            if let Some(p) = &prev {
                if p.branch == Branch::Choked {
                    crossed = true;
                    // Continuity: no jump in pi_c / tau_t across the branch change, M9 just
                    // below 1. The physical adjacent-step jump is only ~1.6-3 % (a ~10-20 K
                    // throttle step), so a tight bound still catches a real discontinuity —
                    // gate 4 pins the VALUES rigorously.
                    assert!((od.pi_c - p.pi_c).abs() < 0.05 * p.pi_c);
                    assert!((od.tau_t - p.tau_t).abs() < 1e-3 * p.tau_t);
                    assert!(0.90 < od.m9 && od.m9 < 1.0);
                }
            }
        }
        prev = Some(od);
    }
    assert!(crossed, "the scan must cross the nozzle-unchoke boundary");
}

// --------------------------------------------------------------------- gate 3 (THE RUNG)
/// GATE 3 — on CPG the SUBSONIC `tau_t` VARIES with throttle: the inversion of rung 31.
///
/// Rung 31's choked branch holds `tau_t` machine-constant on CPG (its gate 2). Here the
/// coupling runs through `pi_c` — structural — rather than through `gamma(T)`/composition, so
/// it SURVIVES CPG and `tau_t` moves measurably. First-order structural coupling against rung
/// 31's second-order variable-`cp` drift.
#[test]
fn gate3_the_rung_cpg_tau_t_varies() {
    let m = cpg_matcher();
    let mut taus = Vec::new();
    for tt4 in [580.0, 560.0, 540.0, 520.0, 500.0, 480.0, 460.0] {
        let od = m.match_point(&flight(), tt4);
        assert_eq!(od.branch, Branch::Subsonic);
        taus.push(od.tau_t);
    }
    let spread = taus.iter().cloned().fold(f64::MIN, f64::max)
        - taus.iter().cloned().fold(f64::MAX, f64::min);
    assert!(spread > 1e-3, "CPG subsonic tau_t must VARY (structural), got {spread:.2e}");
    // And it is monotone — rising toward 1 as the turbine expands less.
    assert!(taus.windows(2).all(|w| w[1] > w[0]), "subsonic tau_t should rise as Tt4 falls");

    // Contrast: the CHOKED branch on the SAME CPG gas holds tau_t machine-constant (rung 31).
    let (hot, warm) = (m.match_point(&flight(), 1200.0).tau_t,
                       m.match_point(&flight(), 800.0).tau_t);
    assert!((hot - warm).abs() < 1e-9,
            "CPG choked tau_t must stay constant, got {:.2e}", (hot - warm).abs());
}

// ------------------------------------------------------------------- gate 4 (the anchor)
/// An INDEPENDENT closed-form CPG solve of the subsonic match (★★) — a SECOND code path.
///
/// No sonic-throat solver, no `Nozzle::apply`, no equilibrium: pure calorically-perfect
/// algebra (Mattingly's dual-mode ratio method on the gas the textbook assumes). For a trial
/// `pi_t`: CPG isentropic turbine -> `tau_t`; shaft (nested one-shot `f`) -> `Tt3` -> `pi_c`;
/// `pt4`, `pt9`; `M9` from the isentropic `pt9/p0`; `MFP(M9)` and `MFP*` in closed form. Then
/// root-find `pi_t` on `mdot_NGV = mdot_noz`.
///
/// **THIS IS THE DENSEST POWER-SPELLING SITE IN THE SLICE — twelve of them — AND IT IS IN A
/// TEST.** Under the port's split rule (`lib.rs`) the two SQUARES are products and the other
/// ten go through [`powp`]. Getting one wrong here does not look like a spelling bug: it
/// surfaces as "the two independent paths disagree at 1e-9", which reads as a solver artefact
/// and would be chased in the wrong place.
fn indep_cpg_subsonic(m: &OffDesignMatcher, gas: &Gas, tt4: f64, p0: f64)
    -> (f64, f64, f64, f64)
{
    let (gc, cpc) = (gas.gamma_c(), gas.spec.cp_c);
    let (gt, cpt, rt) = (gas.gamma_t(), gas.spec.cp_t, gas.r_t());
    let hpr = gas.hpr();
    let m0 = flight().m0;
    let tau_r = 1.0 + 0.5 * (gc - 1.0) * (m0 * m0);                     // SQUARE -> product
    let tt2 = flight().t0 * tau_r;
    let pt2 = PI_D * ram_recovery(m0) * p0 * powp(tau_r, gc / (gc - 1.0));
    let eps = 0.5 * (gt - 1.0);
    let exp_mfp = (gt + 1.0) / (2.0 * (gt - 1.0));
    let mfp_star = powp(gt / rt, 0.5) * powp(2.0 / (gt + 1.0), exp_mfp);

    let op = |pi_t: f64| -> (f64, f64, f64, f64) {
        let tau_t = 1.0 - ETA_T * (1.0 - powp(pi_t, (gt - 1.0) / gt));
        let tt5 = tau_t * tt4;
        let mut f = 0.0f64;
        let mut tt3 = 0.0f64;
        for _ in 0..80 {                       // CPG one-shot burner, the nested f fixed point
            tt3 = tt2 + ETA_M * (1.0 + f) * cpt * (tt4 - tt5) / cpc;
            let f_new = (cpt * tt4 - cpc * tt3) / (ETA_B * hpr - cpt * tt4);
            if (f_new - f).abs() <= 1e-14 * (f_new + 1e-30) {
                break;
            }
            f = f_new;
        }
        let tt3s = tt2 + ETA_C * (tt3 - tt2);
        let pi_c = powp(tt3s / tt2, gc / (gc - 1.0));
        let pt4 = PI_B * pi_c * pt2;
        let pt9 = PI_N * pi_t * pt4;
        let m9 = powp((powp(pt9 / p0, (gt - 1.0) / gt) - 1.0) / eps, 0.5);
        let mfp_m9 = powp(gt / rt, 0.5) * m9 * powp(1.0 + eps * (m9 * m9), -exp_mfp);
        let mdot_ngv = m.a4 * pt4 * mfp_star / powp(tt4, 0.5);
        let mdot_noz = m.a8 * pt9 * mfp_m9 / powp(tau_t * tt4, 0.5);
        (pi_c, tau_t, m9, mdot_ngv - mdot_noz)
    };

    let (mut lo, mut hi) = (0.15f64, 0.9995f64);
    let mut rlo = op(lo).3;
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let rm = op(mid).3;
        if rlo * rm <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
            rlo = rm;
        }
        if hi - lo <= 1e-13 {
            break;
        }
    }
    let pi_t = 0.5 * (lo + hi);
    let (pi_c, tau_t, m9, _) = op(pi_t);
    (pi_t, pi_c, tau_t, m9)
}

/// GATE 4 — an INDEPENDENT CPG closed-form solve of (★★) reproduces the shipped solver.
///
/// The rigorous, non-tautological anchor, mirroring rung 31's gate 2. The subsonic-branch code
/// has **no reduce-to-prior of its own** — gate 1 is a CHOKED point, which returns before the
/// dispatch ever fires — so the only way to pin its values is to solve the same operating point
/// a second, entirely separate way. Two code paths, one operating point. A 1 % `pi_c` drift in
/// the shipped solver would be caught here, where gates 1 and 2 both miss it.
#[test]
fn gate4_nontautological_independent_solve() {
    let gas = cpg_gas();
    let m = cpg_matcher();
    for tt4 in [580.0, 540.0, 500.0, 460.0] {
        let od = m.match_point(&flight(), tt4);
        assert_eq!(od.branch, Branch::Subsonic);
        let (pi_t_i, pi_c_i, tau_t_i, m9_i) = indep_cpg_subsonic(&m, &gas, tt4, 50_000.0);
        assert!((od.pi_t - pi_t_i).abs() < 1e-9,
                "Tt4={tt4}: pi_t {} vs indep {pi_t_i}", od.pi_t);
        assert!((od.pi_c - pi_c_i).abs() < 1e-9 * pi_c_i,
                "Tt4={tt4}: pi_c {} vs indep {pi_c_i}", od.pi_c);
        assert!((od.tau_t - tau_t_i).abs() < 1e-9,
                "Tt4={tt4}: tau_t {} vs indep {tau_t_i}", od.tau_t);
        assert!((od.m9 - m9_i).abs() < 1e-9, "Tt4={tt4}: M9 {} vs indep {m9_i}", od.m9);
    }
}

// -------------------------------------------------------------------- gate 5 (envelope)
/// GATE 5 — the subsonic branch is monotone, and bounded above (unchoke) and below (idle).
#[test]
fn gate5_envelope_monotone() {
    let m = cpg_matcher();
    let ods: Vec<_> = [580.0, 540.0, 500.0, 460.0]
        .iter().map(|&t| m.match_point(&flight(), t)).collect();
    for w in ods.windows(2) {
        assert!(w[0].pi_c > w[1].pi_c, "pi_c falls with Tt4");
        assert!(w[0].m9 > w[1].m9, "M9 falls with Tt4");
        assert!(w[0].performance.specific_thrust > w[1].performance.specific_thrust,
                "thrust falls with Tt4");
    }
}

/// GATE 5 (lower bound) — below thrust-neutral idle the match self-reports SUB-IDLE.
///
/// The message is asserted, not merely the panic: three different guards can abort a cell down
/// here (the bracket, the thrust floor, and the equilibrium Newton), and "it panicked" would
/// pass for any of them.
#[test]
#[should_panic(expected = "SUB-IDLE")]
fn gate5_subidle_is_reported_not_force_fit() {
    let m = cpg_matcher();
    let _ = m.match_point(&flight(), 400.0);
}

// --------------------------------------------------------------------- gate 6 (framing)
/// GATE 6 — the coupling is to `pi_c` via `pt9/p0`, NOT to ambient `p0`: scale `p0`, and the
/// ratios are invariant.
#[test]
fn gate6_homogeneity_coupling_through_pi_c() {
    let mut ratios = Vec::new();
    for p0 in [25_000.0, 50_000.0, 100_000.0] {
        let fl = FlightCondition::new(250.0, p0, 0.85);
        let m = matcher_at(cpg_gas(), fl, p0);
        let od = m.match_point(&fl, 500.0);
        assert_eq!(od.branch, Branch::Subsonic);
        ratios.push((od.pi_c, od.tau_t, od.m9));
    }
    let first = ratios[0];
    for &(pc, tt, m9) in &ratios[1..] {
        assert!((pc - first.0).abs() < 1e-9 * first.0, "pi_c must be p0-invariant");
        assert!((tt - first.1).abs() < 1e-9, "tau_t must be p0-invariant");
        assert!((m9 - first.2).abs() < 1e-9, "M9 must be p0-invariant");
    }
}

// ------------------------------------------------------------------------------- gate 7
/// GATE 7 — the default design run is bit-for-bit rung 6.
#[test]
fn gate7_cycle_untouched() {
    let plain = Losses { nozzle_convergent: false, ..real() };
    let r = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, 50_000.0, plain)
        .run(&flight(), 1.0);
    assert!((r.performance.specific_thrust - 798.37).abs() < 0.5);   // the rung-6 anchor
    assert!(r.m9 > 1.8 && (r.p9 - 50_000.0).abs() < 1e-6);
}

/// WHAT SLICE J OWED — **DISCHARGED**, at
/// `rung32.rs::rung33_gate7_second_half_map_does_not_inherit_subsonic`.
///
/// `test_rung33.py`'s gate 7 has a second half this module could not express when slice I
/// shipped: `MapMatcher` (rung 32) overrides `match` and stays on its choked-only path below
/// unchoke, flagging `nozzle_choked = False` WITHOUT re-solving — because subsonic + map is out
/// of scope. It documents that rung 33's dispatch is **not inherited** by the map matcher.
///
/// Rung 32 did not exist in the Rust then, so there was nothing for that claim to be true or
/// false of. It was written down rather than left implicit because the port has been bitten
/// twice by exactly this shape: once by a documented gate that did not exist (§ 4.16), and once
/// by a census that answered only for the phase it was standing in (§ 5.3 finding 4).
///
/// **This test is KEPT rather than deleted**, and it now carries the half the rung-32 file
/// cannot: that dispatching is a property of `match_point`'s OWN body, so the only thing rung 32
/// had to do was decline to call it. Delete it and the rung-32 assertion becomes a statement
/// about one matcher with nothing to contrast against.
#[test]
fn slice_j_deferrals() {
    let m = cpg_matcher();
    let deep = m.match_point(&flight(), 560.0);
    assert_eq!(deep.branch, Branch::Subsonic,
               "rung 31/33's own matcher DOES dispatch — slice J's job is to show rung 32 \
                does not inherit that, which needs rung 32 to exist");
    assert!(!deep.nozzle_choked);
}
