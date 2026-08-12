//! Rung-30 verification: THE CHOKED CONVERGENT NOZZLE — is FULL EXPANSION earned?
//!
//! Every rung since 1 has expanded the nozzle to a SPECIFIED exit pressure, ambient by default —
//! i.e. assumed a perfectly-expanded (convergent-divergent) nozzle. A fixed CONVERGENT nozzle has
//! no diverging section, so its exit IS its throat and the flow can only reach Mach 1 there. Above
//! the critical pressure ratio it CHOKES: the exit pins at `p* > p0` and the nozzle runs
//! underexpanded.
//!
//! **THE VERDICT: full expansion is NOT earned at the design point** — the convergent nozzle
//! chokes — but the momentum loss is far smaller than the halved velocity implies, because the
//! PRESSURE term is strictly positive and cancels most of it.
//!
//! **WHERE THIS SUITE SAYS MORE THAN THE PYTHON'S.**
//!
//! 1. **The two-path gate is protected from going vacuous, and the protection is checked.** Rung
//!    30's gate 2a justifies itself as "two genuinely different code paths onto the same M=1
//!    condition" — but it runs on a CPG gas, where [`sonic_throat`] takes a CLOSED FORM. Without
//!    an explicit `sonic_throat_bisect` entry point it would compare the closed form against
//!    itself. This suite asserts the two paths DISAGREE somewhere (they must, to different
//!    stopping rules) as well as agreeing to the bisection's band — the same shape
//!    `porting_rules.rs` uses to prove `powp` is a real call.
//! 2. **The choke/unchoke boundary is swept**, so both branches are exercised at every design
//!    point rather than only where the shipped ambient happens to land.

use turbojet::components::{sonic_throat, sonic_throat_bisect, Nozzle};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{FlowState, Gas, GasSpec};

const PI_C: f64 = 10.0;

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

/// A SELF-CONSISTENT CPG gas: `cp = γR/(γ−1)` exactly, so the textbook critical ratios apply.
///
/// The shipped Mattingly constants are ROUNDED (`cp_t = 1239`, `γ_t = 1.3`, `R_t = 285.9` do not
/// satisfy that identity to better than ~0.1 %), which is the trap gate 2a exists to avoid: on
/// those constants the residual's own root and the textbook `2/(γ+1)` are different numbers, and a
/// test comparing them would be measuring the rounding rather than the solver.
fn cpg_consistent() -> Gas {
    let (gamma, r) = (1.3f64, 285.9f64);
    Gas::new(GasSpec {
        gamma_c: 1.4,
        cp_c: 1004.0,
        r_c: 286.9,
        gamma_t: gamma,
        cp_t: gamma * r / (gamma - 1.0),
        r_t: r,
        hpr: 42.8e6,
        ..GasSpec::default()
    })
}

struct Dp {
    gas: Gas,
    far: f64,
    tt5: f64,
    pt5: f64,
    v9: f64,
    p9: f64,
    t9: f64,
}

fn dp(tt4: f64) -> Dp {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, 50_000.0, losses());
    let r = eng.run(&flight(), 1.0);
    let (s4, s5) = (r.station("4"), r.station("5"));
    Dp { far: s4.far, tt5: s5.tt, pt5: s5.pt, v9: r.v9, p9: r.p9, t9: r.t9, gas: eng.gas }
}

fn st5(d: &Dp) -> FlowState {
    FlowState { tt: d.tt5, pt: d.pt5, mdot: 1.0, far: d.far }
}

// --- GATE 1: REDUCE — subcritical, the convergent nozzle IS the shipped one ------------------- //

/// Below the critical pressure ratio a convergent nozzle reaches `p9 = p0` with `M9 < 1`, and must
/// then be BIT-FOR-BIT the shipped specified-exit-pressure nozzle at the same condition. That is
/// the spine: rung 30 adds a branch, it does not move the default path.
#[test]
fn subcritical_convergent_is_the_default_nozzle_bit_for_bit() {
    let mut checked = 0usize;
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let pt9 = 0.98 * d.pt5;
        // A back-pressure comfortably above critical (p*/pt ~ 0.54), so it cannot choke.
        for frac in [0.70, 0.80, 0.95] {
            let p0 = frac * pt9;
            let conv = Nozzle::convergent(p0, 0.98).apply(&st5(&d), &d.gas);
            let plain = Nozzle::new(p0, 0.98, Some(p0)).apply(&st5(&d), &d.gas);
            assert!(conv.m9 < 1.0, "should not choke at p0/pt9={frac} (Tt4={tt4})");
            assert_eq!(conv.m9.to_bits(), plain.m9.to_bits());
            assert_eq!(conv.t9.to_bits(), plain.t9.to_bits());
            assert_eq!(conv.v9.to_bits(), plain.v9.to_bits());
            assert_eq!(conv.p9.to_bits(), plain.p9.to_bits());
            checked += 1;
        }
    }
    assert_eq!(checked, 15);
}

/// …and the DEFAULT nozzle is untouched, so the whole rungs 1–6 cycle is inert to this rung.
#[test]
fn the_default_nozzle_path_is_untouched() {
    let d = dp(1500.0);
    let n = Nozzle::new(50_000.0, 0.98, Some(50_000.0));
    assert!(!n.convergent, "the default must not be convergent");
    let ex = n.apply(&st5(&d), &d.gas);
    assert_eq!(ex.v9.to_bits(), d.v9.to_bits());
    assert_eq!(ex.p9.to_bits(), d.p9.to_bits());
    assert_eq!(ex.t9.to_bits(), d.t9.to_bits());
}

// --- GATE 2: THE SOLVER IS RIGHT — two genuinely different paths, and they DIFFER -------------- //

/// **The bisection and the closed form must AGREE to the bisection's band and DISAGREE somewhere.**
///
/// The agreement is the physics check. The disagreement is what proves the gate is not comparing
/// one path with itself: on a CPG gas `sonic_throat` returns the closed-form root, so if
/// `sonic_throat_bisect` were not a separate entry point this whole test would be an identity.
/// The bisection stops at `1e-13·Tt`, so it CANNOT land on the exact root every time — and a run
/// where the two agreed at every point would mean the dispatch had collapsed.
#[test]
fn the_closed_form_and_the_bisection_are_two_paths_onto_one_condition() {
    let g = cpg_consistent();
    let (mut agree, mut differ) = (0usize, 0usize);
    for tt9 in [900.0, 1100.0, 1262.0, 1500.0, 1800.0, 2000.0] {
        let (tstar, _, _) = sonic_throat(&g, tt9, 3.4e5, 0.0);
        let tb = sonic_throat_bisect(&g, tt9, 0.0, g.h_t(tt9, 0.0), g.r_t_at(0.0));
        assert!(
            (tb - tstar).abs() <= 1e-12 * tt9,
            "the two paths disagree beyond the bisection's band at Tt9={tt9}: {tb} vs {tstar}"
        );
        if tb.to_bits() == tstar.to_bits() {
            agree += 1;
        } else {
            differ += 1;
        }
    }
    assert!(
        differ > 0,
        "closed form and bisection agreed BIT-FOR-BIT at all {agree} points — that is the \
         signature of one path being compared with itself, not of a correct solver"
    );
}

/// On a SELF-CONSISTENT CPG gas the solved throat must reproduce the textbook critical ratios.
/// This is the non-tautological physics check — a different formula, not a different code path.
#[test]
fn the_cpg_throat_reproduces_the_textbook_critical_ratios() {
    let g = cpg_consistent();
    let gamma = 1.3f64;
    for tt9 in [900.0, 1500.0, 2000.0] {
        let (tstar, pstar, _) = sonic_throat(&g, tt9, 3.4e5, 0.0);
        let t_ratio = 2.0 / (gamma + 1.0);
        let p_ratio = t_ratio.powf(gamma / (gamma - 1.0));
        assert!(
            (tstar / tt9 - t_ratio).abs() < 1e-9,
            "T*/Tt = {} vs textbook {t_ratio} at Tt9={tt9}",
            tstar / tt9
        );
        assert!(
            (pstar / 3.4e5 - p_ratio).abs() < 1e-9,
            "p*/pt = {} vs textbook {p_ratio} at Tt9={tt9}",
            pstar / 3.4e5
        );
    }
}

/// The M=1 sonic identity on the REACTING gas, where the throat is SEARCHED rather than solved:
/// `V* == a(T*)` by construction, so the nozzle exits at Mach 1 exactly.
#[test]
fn the_reacting_throat_exits_at_mach_one() {
    for tt4 in [1300.0, 1500.0, 1800.0, 2200.0, 2300.0] {
        let d = dp(tt4);
        let pt9 = 0.98 * d.pt5;
        let (tstar, _, vstar) = sonic_throat(&d.gas, d.tt5, pt9, d.far);
        let a = (d.gas.gamma_t_at(tstar, d.far) * d.gas.r_t_at(d.far) * tstar).sqrt();
        assert!((vstar / a - 1.0).abs() < 1e-9, "M9={} at Tt4={tt4}", vstar / a);
        // …and the root sits in the physical band the bracket assumes.
        assert!(tstar / d.tt5 > 0.80 && tstar / d.tt5 < 0.92, "T*/Tt={}", tstar / d.tt5);
    }
}

// --- GATE 3: THE VERDICT — it CHOKES at design, and the PRESSURE TERM rescues most of it ------- //

/// At the design point the convergent nozzle chokes, so full expansion is NOT earned. But the
/// momentum deficit is largely cancelled by the pressure term, which is the rung's actual finding:
/// the halved exit velocity does NOT mean a halved thrust.
///
/// Specific thrust per unit mass flow is `V9 − V0 + (p9 − p0)/(ρ9·V9)`, and the pressure term is
/// strictly zero for a fully-expanded nozzle and strictly positive for a choked one.
#[test]
fn it_chokes_at_design_and_the_pressure_term_rescues_most_of_the_deficit() {
    let d = dp(1500.0);
    let p0 = 50_000.0;
    let conv = Nozzle::convergent(p0, 0.98).apply(&st5(&d), &d.gas);
    let full = Nozzle::new(p0, 0.98, Some(p0)).apply(&st5(&d), &d.gas);

    assert!((conv.m9 - 1.0).abs() < 1e-9, "the design point must CHOKE: M9={}", conv.m9);
    assert!(conv.p9 > p0, "a choked nozzle is UNDEREXPANDED: p9={} vs p0={p0}", conv.p9);
    assert!(conv.v9 < full.v9, "the choked exit velocity must be lower");

    // The two thrust terms, per unit mass flow. `rho9 = p9/(R_t*T9)`.
    let v0 = flight().m0 * (1.4 * 286.9 * 250.0f64).sqrt();
    let r_t = d.gas.r_t_at(d.far);
    let rho9 = conv.p9 / (r_t * conv.t9);
    let momentum = conv.v9 - v0;
    let pressure = (conv.p9 - p0) / (rho9 * conv.v9);
    let full_thrust = full.v9 - v0; // fully expanded: the pressure term is exactly zero
    assert!(pressure > 0.0, "the pressure term must be strictly positive when choked");

    // The momentum deficit ALONE would be a large loss; the pressure term cancels most of it.
    let momentum_deficit = full_thrust - momentum;
    let net_deficit = full_thrust - (momentum + pressure);
    assert!(momentum_deficit > 0.0 && net_deficit > 0.0, "the convergent nozzle must lose thrust");
    let rescued = 1.0 - net_deficit / momentum_deficit;
    assert!(
        rescued > 0.5,
        "the pressure term should rescue most of the momentum deficit, got {rescued}"
    );
    // …and the NET loss is a few percent, not the tens of percent the velocity drop implies.
    assert!(
        net_deficit / full_thrust < 0.15,
        "net specific-thrust loss {} is larger than the rung claims",
        net_deficit / full_thrust
    );
}

/// The choke boundary MOVES with the back-pressure, so both branches are live at every design
/// point — the census that keeps the reduce from being tested on only one side.
#[test]
fn the_choke_boundary_is_where_the_critical_ratio_puts_it() {
    for tt4 in [1300.0, 1500.0, 2200.0] {
        let d = dp(tt4);
        let pt9 = 0.98 * d.pt5;
        let (_, pstar, _) = sonic_throat(&d.gas, d.tt5, pt9, d.far);
        // Just below p*: choked. Just above: not.
        let below = Nozzle::convergent(pstar * 0.999, 0.98).apply(&st5(&d), &d.gas);
        let above = Nozzle::convergent(pstar * 1.001, 0.98).apply(&st5(&d), &d.gas);
        assert!((below.m9 - 1.0).abs() < 1e-9, "should choke just below p* at Tt4={tt4}");
        assert!(above.m9 < 1.0, "should NOT choke just above p* at Tt4={tt4}");
    }
}
