//! Rung-20 verification: super-equilibrium O THROUGH the quench — lifting the finite-quench
//! lower bound.
//!
//! Rung 19 lifted the equilibrium-[O] lower bound only on the PRIMARY diagnostic. The
//! finite-quench fields still RE-MADE NO on equilibrium O, so every one was still a lower
//! bound. Rung 20 threads the same Westenberg `m(T)` lift INSIDE the `quench_no` re-making,
//! closing that seam (`docs/rung20-spec.md`).
//!
//! The load-bearing result INVERTS the naive "the lift bites hardest on the slow cooling
//! pocket": the Zeldovich re-making peaks at the HOTTEST stoich crossing, where `m(T)` is at its
//! MINIMUM, so the effective lift is MODEST & PEAK-CONCENTRATED (`≈ m(T_peak)`) — even SMALLER
//! than the rung-19 primary lift, because the quench crossing is hotter than the flame.
//!
//! Gates (priority order), and **what this slice can and cannot see**:
//!
//! 1. **REDUCE (LOAD-BEARING)** — `super_eq_o: false` ⇒ bit-for-bit the prior rung, as a direct
//!    `quench_no` reduce and through the public entry point.
//! 2. **THE MODEST PEAK-CONCENTRATED LIFT** — the bulk-quench lift ∈ (1.10, 1.25),
//!    `≥ m(T_peak)`, and STRICTLY LESS than the rung-19 primary lift at the same φ_p.
//! 3. **CLAMP DORMANT** — `max_a_quench` stays < 1 with the lift: super-eq O is NOT the
//!    burner-clamp lever.
//! 7. **THE FLOOR is load-bearing** — raw `m(T)` DIVERGES below the flame band, and the T-floor
//!    is what keeps the lifted quench inside the standing `1 ≤ m ≤ 2` trajectory assert.
//!
//! **DELIBERATELY NOT PORTED HERE** — gates 4, 5 and 6 read `exhaust_no_clamp` (rung 17), the
//! prompt-through-the-dilution invariant, and the rung-13/15/18 ideal-bell closures. None of
//! those exist yet: the nozzle strand and the PDF family are later slices. Shipping tests whose
//! subject is absent would be shipping untested code, which is the objection phase 2 raised
//! against porting rung 30's choked nozzle into a phase whose gates could not see it. They land
//! WITH their machinery. What gate 6's quench half asserts — that the lift composes with the
//! `quench_no`-based closures — IS portable now, and is [`super_eq_o_combines_with_unmixedness`].

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, f_stoich, hf_fuel_default, Gas};
use turbojet::nox::{
    primary_aft, quench_no, super_eq_o_multiplier, thermal_no, JetMixing, QuenchOpts,
    Unmixedness, ZonedNoxOpts, SUPER_EQ_T_FLOOR,
};

const TAU: f64 = 3e-3;
const NG: usize = 33;
const NSTEPS: usize = 800;
const PHI_P: f64 = 1.5;
const J: f64 = 25.0;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}
fn losses() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}
fn mix() -> JetMixing {
    JetMixing { j: J, c_e: 0.20, shape_n: 2.0, ..JetMixing::default() }
}
fn opts() -> ZonedNoxOpts {
    ZonedNoxOpts { tau: TAU, quench_ngrid: NG, quench_nsteps: NSTEPS, ..ZonedNoxOpts::default() }
}

/// `(gas, far, Tt3, Tt4, pt4)` — the rung-17 design point. NO is trace ⇒ cycle bit-for-bit rung 6.
fn dp() -> (Gas, f64, f64, f64, f64) {
    let g = Gas::reacting_equilibrium();
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, flight().p0, losses())
        .run(&flight(), 50.0);
    (g, r.station("4").far, r.station("3").tt, r.station("4").tt, r.station("4").pt)
}

/// The mean-field bulk quench with / without the lift, through the PUBLIC entry point.
fn bulk(super_eq_o: bool) -> turbojet::nox::ZonedNoxState {
    let (g, far, tt3, tt4, p) = dp();
    g.zoned_nox(far, tt3, tt4, p, PHI_P,
                ZonedNoxOpts { mixing: Some(mix()), super_eq_o, ..opts() })
}

// --------------------------------------------------------------------------------------
// GATE 1 — REDUCE: super_eq_o = false is bit-for-bit the prior rung.
// --------------------------------------------------------------------------------------

/// A direct `quench_no` reduce: the flag off is the exact prior integrator call.
///
/// This one is worth its cost even though the Rust branch is a visible `if o.super_eq_o`,
/// because the lift is applied to a LOCAL `c_o` inside the derivative closure. A port that
/// hoisted the multiply out of the branch, or applied it to the trajectory table instead of the
/// interpolated value, would leave the flag-on numbers plausible and this comparison unequal.
#[test]
fn reduce_quench_no_flag_off_is_byte_identical() {
    let (_g, far, tt3, _tt4, p) = dp();
    let far_p = PHI_P * f_stoich();
    let alpha = far / far_p;
    let t_p = primary_aft(far_p, p, tt3, hf_fuel_default());
    let comp_p = equilibrium_composition(far_p, t_p, p);
    let ntot: f64 = comp_p.iter().map(|&(_, v)| v).sum();
    let n0 = alpha * thermal_no(&comp_p, t_p, p, TAU, far_p, 4000, 1.0).x_no * ntot;
    let m = mix();
    let sched = |x: f64| m.schedule(x);
    let run = |super_eq_o: bool| {
        quench_no(&comp_p, t_p, alpha, far, tt3, p, n0, m.tau_q(), QuenchOpts {
            nsteps: NSTEPS, ngrid: NG, tab: None, schedule: Some(&sched), super_eq_o,
        })
    };
    let (a, b) = (run(false), run(false));
    assert_eq!(a.ei.to_bits(), b.ei.to_bits());
    assert_eq!(a.x_no_mix.to_bits(), b.x_no_mix.to_bits());
    // and the lift, when ON, must actually MOVE the answer — otherwise the reduce above is
    // measuring nothing at all, which is the trap rung 75's inherited instrument fell into.
    let lifted = run(true);
    assert_ne!(lifted.ei.to_bits(), a.ei.to_bits(),
               "super_eq_o=true did not move the quench EI — the reduce test would then be \
                vacuous, since both arms would agree for the wrong reason");
}

/// The same reduce through the public entry point.
#[test]
fn reduce_zoned_flag_off_is_identical() {
    let (g, far, tt3, tt4, p) = dp();
    let base = g.zoned_nox(far, tt3, tt4, p, PHI_P, ZonedNoxOpts { mixing: Some(mix()), ..opts() });
    let off = bulk(false);
    assert_eq!(base.ei_no_quenched.unwrap().to_bits(), off.ei_no_quenched.unwrap().to_bits(),
               "bulk quench not bit-for-bit with super_eq_o=false");
    assert_eq!(base.max_a_quench.unwrap().to_bits(), off.max_a_quench.unwrap().to_bits());
    assert_eq!(base.t_peak.unwrap().to_bits(), off.t_peak.unwrap().to_bits(),
               "the trajectory is equilibrium-O throughout — only the integrator lifts, so \
                T_peak must be untouched by the flag in EITHER direction");
}

// --------------------------------------------------------------------------------------
// GATE 2 — the MODEST, PEAK-CONCENTRATED lift (the corrected headline).
// --------------------------------------------------------------------------------------

/// The lift is a small `O(m)` factor, floored by `m` at the HOTTEST point, and SMALLER than the
/// rung-19 primary lift.
///
/// The peak-floor argument is the interesting one: `m(T)` DECREASES in T, and with `max_a < 1`
/// every rate contribution along the path is positive, so the delivered lift is a clean
/// formation-weighted average of `m` — which is therefore ≥ its value at the hottest point.
/// And because the quench crossing is HOTTER than the flame, threading the lift through the
/// quench gives LESS lift than rung 19 got on the primary, not more. That is the inversion.
#[test]
fn lift_is_modest_peak_concentrated_and_below_primary() {
    let (b0, b_l) = (bulk(false), bulk(true));
    let lift = b_l.ei_no_quenched.unwrap() / b0.ei_no_quenched.unwrap();
    let m_peak = super_eq_o_multiplier(b_l.t_peak.unwrap()); // m at the hottest point = m's MIN
    assert!(1.10 < lift && lift < 1.25,
            "bulk-quench lift {lift:.4} outside the modest band (1.10, 1.25)");
    assert!(lift >= m_peak - 1e-6,
            "lift {lift:.4} below m(T_peak)={m_peak:.4} — not peak-floored");
    assert!(lift < 1.5 * m_peak,
            "lift {lift:.4} far above m(T_peak)={m_peak:.4} — not peak-concentrated");

    let (g, far, tt3, tt4, p) = dp();
    let zn0 = g.zoned_nox(far, tt3, tt4, p, PHI_P, ZonedNoxOpts { tau: TAU, ..Default::default() });
    let zn_l = g.zoned_nox(far, tt3, tt4, p, PHI_P,
                           ZonedNoxOpts { tau: TAU, super_eq_o: true, ..Default::default() });
    let primary_lift = zn_l.ei_no() / zn0.ei_no();
    assert!(b_l.t_peak.unwrap() > zn0.t_primary,
            "quench peak {:.0} not hotter than flame {:.0}", b_l.t_peak.unwrap(), zn0.t_primary);
    assert!(lift < primary_lift,
            "quench lift {lift:.4} not < primary lift {primary_lift:.4} (the hotter peak is why)");
}

// --------------------------------------------------------------------------------------
// GATE 3 — the clamp stays dormant with the lift.
// --------------------------------------------------------------------------------------

/// Super-eq O speeds FORMATION; it does not raise `[NO]_e`, which is a thermodynamic ceiling
/// untouched by the O-atom closure. So the lift moves the numerator of `a = [NO]/[NO]_e` and
/// not the denominator, and `max_a` rises — but not through 1. The burner-clamp seam wants a
/// SLOW-FREEZE lever, and this is not one.
#[test]
fn clamp_stays_dormant_with_the_lift() {
    let (a0, a_l) = (bulk(false).max_a_quench.unwrap(), bulk(true).max_a_quench.unwrap());
    assert!(a_l < 1.0,
            "max_a={a_l:.4} crossed 1 with the lift — the burner-clamp seam would then be a \
             super-eq-O lever, which the rung says it is not");
    assert!(a_l > a0,
            "max_a must RISE with the lift ({a_l:.4} vs {a0:.4}): the numerator lifts and the \
             thermodynamic denominator does not. If it did not rise, the lift is not reaching \
             the ratio and gate 2's factor is coming from somewhere else");
}

// --------------------------------------------------------------------------------------
// GATE 6 (the portable half) — the lift composes with the quench-based closures.
// --------------------------------------------------------------------------------------

/// The `quench_no`-based closures accept the lift. Rung 12's core is a second integration on
/// the same trajectory, so the lift must reach BOTH streams — and `max_a_quench` spanning the
/// pair is what would catch it reaching only one.
#[test]
fn super_eq_o_combines_with_unmixedness() {
    let (g, far, tt3, tt4, p) = dp();
    let um = Unmixedness {
        s: 0.0625, c_opt: 2.5, k_u: 0.3, b_u: 3.0, tau_res: 2.5e-3, ..Unmixedness::default()
    };
    let base = g.zoned_nox(far, tt3, tt4, p, PHI_P, ZonedNoxOpts {
        mixing: Some(mix()), unmixedness: Some(um), ..opts()
    });
    let lifted = g.zoned_nox(far, tt3, tt4, p, PHI_P, ZonedNoxOpts {
        mixing: Some(mix()), unmixedness: Some(um), super_eq_o: true, ..opts()
    });
    assert!(lifted.ei_no_core.unwrap() > base.ei_no_core.unwrap(),
            "the lift must reach the lingering CORE too, not just the bulk");
    assert!(lifted.ei_no_quenched.unwrap() > base.ei_no_quenched.unwrap(),
            "the lift must reach the mean-field BULK");
    assert!(lifted.ei_no_unmixed.unwrap() > base.ei_no_unmixed.unwrap(),
            "and therefore the two-stream total");
}

// --------------------------------------------------------------------------------------
// GATE 7 — the T-floor is load-bearing.
// --------------------------------------------------------------------------------------

/// Raw `m(T) = A·T·exp(B/T)` DIVERGES as T falls, and a quench trajectory cools to `≈ Tt4`.
/// Without the floor the lift would inject an out-of-band multiplier on the cool tail and trip
/// the standing `1 ≤ m ≤ 2` assert — so the floor is what makes rung 20 expressible at all.
///
/// **MEASURED, and it is not what the design point suggests.** At the shipped point
/// (Tt4 = 1500 K) the trajectory bottoms out at 1517 K — ABOVE the 1500 K floor, so the floor is
/// DORMANT there, by 17 K. It binds below roughly Tt4 = 1480 K. Asserting the floor at the
/// design point alone would therefore have been a gate on a branch nothing takes, so this runs
/// a COOLER design point too, where the clip genuinely fires and the standing trajectory assert
/// is what the floor is holding up.
#[test]
fn super_eq_o_floor_keeps_m_in_band() {
    let m_cold_raw = super_eq_o_multiplier(1200.0); // below the flame band
    let m_floored = super_eq_o_multiplier(1200.0f64.max(SUPER_EQ_T_FLOOR));
    assert!(m_cold_raw > 2.0,
            "raw m(1200 K)={m_cold_raw:.3} should DIVERGE past 2 — the hazard the floor guards");
    assert!((1.0..=2.0).contains(&m_floored),
            "floored m={m_floored:.3} must sit in the flame-band bound [1,2]");
    assert!(super_eq_o_multiplier(SUPER_EQ_T_FLOOR) <= 2.0,
            "the floor must map onto an in-band multiplier");

    // (a) at the SHIPPED design point the floor is dormant — thinly.
    let t_cold_design = bulk(true).t_mix;
    assert!(t_cold_design > SUPER_EQ_T_FLOOR,
            "expected the 1500 K design point to stay ABOVE the floor; got {t_cold_design:.1} K");
    assert!(t_cold_design - SUPER_EQ_T_FLOOR < 50.0,
            "the design point's margin over the floor is {:.0} K — measured at 17 K, so a big \
             move means the mix-out or the floor changed",
            t_cold_design - SUPER_EQ_T_FLOOR);

    // (b) a COOLER design point takes the clip. Without the floor, `m` at this trajectory's
    // cold end would be out of band and the lifted quench would trip its own assert; that it
    // completes IS the gate, and the margin below the floor is what makes the branch live.
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1400.0, flight().p0, losses())
        .run(&flight(), 50.0);
    let (far, tt3, tt4, p) =
        (r.station("4").far, r.station("3").tt, r.station("4").tt, r.station("4").pt);
    let g = Gas::reacting_equilibrium();
    let cold = g.zoned_nox(far, tt3, tt4, p, PHI_P,
                           ZonedNoxOpts { mixing: Some(mix()), super_eq_o: true, ..opts() });
    assert!(cold.t_mix < SUPER_EQ_T_FLOOR,
            "the Tt4=1400 K point must cool BELOW the floor for the clip to be exercised; got \
             {:.1} K", cold.t_mix);
    assert!(super_eq_o_multiplier(cold.t_mix) > 2.0,
            "…and the UNFLOORED multiplier there must be out of band ({:.3}), which is exactly \
             what the floor is preventing", super_eq_o_multiplier(cold.t_mix));
    assert!(cold.ei_no_quenched.unwrap() > 0.0);
}
