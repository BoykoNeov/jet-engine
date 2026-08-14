//! RUNG 42 — INTERSTAGE BLEED: the valve is a degree of freedom on ONE spool.
//!
//! Port of `tests/test_rung42.py`. **The Python file has TWELVE `def test_` functions under EIGHT
//! documented gates**, and § 5.8's step-4 line called it "10 gates". The roster is written out
//! below and re-stated as data in [`slice_l_roster`] — same reason as `rung41.rs`: a count nobody
//! can re-derive is how a port drops one silently (`docs`' *slice K* and *an oracle cannot see a
//! MISSING GATE*).
//!
//! **All twelve port.** Unlike rung 41's, this file's `test_cycle_untouched_rung6_bit_for_bit`
//! reaches only `build_turbojet` and the bleed matcher — no `SpoolTransient` — so nothing here
//! waits for phase 6.
//!
//! | # | `tests/test_rung42.py` | here |
//! |---|------------------------|------|
//! | 1 | `test_reduce_bleed_zero_is_rung39_bit_for_bit` | [`gate1_reduce_valve_shut_is_rung39`] |
//! | 2 | `test_reduce_bleed_zero_bit_for_bit_on_reacting_gas` | [`gate1b_reduce_holds_on_the_reacting_gas`] |
//! | 3 | `test_x_lp_is_exactly_bleed_invariant_and_phi_lp_moves` | [`gate2_x_lp_is_exactly_bleed_invariant`] |
//! | 4 | `test_hp_running_line_is_bleed_invariant_as_a_curve` | [`gate2b_hp_stays_on_one_running_line`] |
//! | 5 | `test_mass_extraction_identity` | [`gate2c_mass_extraction_identity`] |
//! | 6 | `test_bleed_derived_s_H_matches_rung41_closed_form` | [`gate3_perturbation_independence`] |
//! | 7 | `test_bleed_hp_response_reverses_sign_at_pi_star` | [`gate4_the_response_reverses_at_pi_star`] |
//! | 8 | `test_self_targeting_is_a_phi_space_statement` | [`gate5_self_targeting_in_phi_space`] |
//! | 9 | `test_trade_thrust_falls_tsfc_rises_and_the_cost_grows_with_throttle_down` | [`gate6_the_trade`] |
//! |10 | `test_opening_the_valve_shrinks_the_choked_envelope` | [`gate6b_the_valve_shrinks_the_envelope`] |
//! |11 | `test_bleed_does_not_penalise_the_hp_spool_at_design` | [`gate7_the_refuted_hypothesis`] |
//! |12 | `test_cycle_untouched_rung6_bit_for_bit` | [`gate8_cycle_untouched_rung6`] |
//!
//! **Two gates here have no Python counterpart, and both exist because the PORT can fail in a way
//! the source cannot.**
//!
//! * [`the_dispatch_is_live`] — § 5.8's owed gate. Rung 42 is the port's first override of a live
//!   virtual slot. Writing `R39.try_match_point` in `R42`'s table compiles, returns numbers, and
//!   silently gives rung 42 rung 39's physics. Python cannot get this wrong: `self.match` IS the
//!   override.
//! * [`p7_the_absence_is_a_type`] — Python never CONSTRUCTS a `TwoSpoolBleedResult` at `b = 0`,
//!   so its four booking attributes are ABSENT and `bleed_trade` reads that through `getattr`. A
//!   port that always built the struct would write `0.0` where Python writes the core specific
//!   thrust, and a float dump compares that equal.

use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses,
                          TwoSpoolMapMatcher};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const THROTTLE: [f64; 4] = [1500.0, 1300.0, 1100.0, 900.0];

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

fn cpg_gas() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn fast_gas() -> Gas {
    Gas::thermally_perfect()
}

/// Python shares ONE `design_engine` object between the reference and the bleed matcher; Rust's
/// takes ownership, so it is rebuilt. The design cycle is a pure function of its arguments, so
/// the two builds are bit-identical — and gate 1 would fail loudly if they were not, since it
/// compares the two matchers' output with `==` on the bits.
fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

fn bm(gas: Gas, ml: ComponentMap, mh: ComponentMap, bleed: f64) -> TwoSpoolBleedMatcher {
    TwoSpoolBleedMatcher::new(design(gas), flight(), 1.0, ml, mh, bleed)
}

fn bm_floor(gas: Gas, ml: ComponentMap, mh: ComponentMap, bleed: f64, floor: f64)
    -> TwoSpoolBleedMatcher {
    TwoSpoolBleedMatcher::new(design(gas), flight(), 1.0,
                              ComponentMap { phi_surge: floor, ..ml },
                              ComponentMap { phi_surge: floor, ..mh }, bleed)
}

fn flat() -> ComponentMap {
    ComponentMap::flat()
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

fn tilted() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
}

fn steep() -> ComponentMap {
    ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..ComponentMap::flat() }
}

fn shapes() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    vec![
        ("flow/press", lp_shaped(), hp_shaped()),
        ("press/flow", ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..flat() },
                       ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..flat() }),
        ("tilted", tilted(), tilted()),
        ("steep", steep(), steep()),
    ]
}

/// Rung 41's closed-form running-line flow-coefficient sensitivity.
fn s_h_closed(pi: f64, gamma_c: f64) -> f64 {
    let k = gamma_c / (gamma_c - 1.0);
    k * (1.0 - pi.powf(-1.0 / k)) - 1.0
}

// ============================================================================== gate 1
/// **GATE 1 — REDUCE, exact dispatch.** `bleed == 0` never enters the bleed cascade: `match`
/// forwards to rung 39's body VERBATIM, so a bleed matcher with the valve shut is rung 39
/// bit-for-bit.
///
/// The Rust spells that forward as `R39.try_match_point` and NOT `core.try_match_point`, which is
/// load-bearing: this is Python's `super().match(...)`, a NON-virtual call, and routing it back
/// through the hook table would recurse forever.
#[test]
fn gate1_reduce_valve_shut_is_rung39() {
    for (name, ml, mh) in shapes() {
        let reference = TwoSpoolMapMatcher::new(design(fast_gas()), flight(), 1.0, ml, mh);
        let shut = bm(fast_gas(), ml, mh, 0.0);
        for tt4 in THROTTLE {
            let a = reference.match_point(&flight(), tt4).two();
            let b = shut.match_point(&flight(), tt4);
            assert_eq!(a.base.pi_lpc.to_bits(), b.base.base.pi_lpc.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.base.pi_hpc.to_bits(), b.base.base.pi_hpc.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.eta_lpc.to_bits(), b.base.eta_lpc.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.eta_hpc.to_bits(), b.base.eta_hpc.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.phi_lp.to_bits(), b.base.phi_lp.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.phi_hp.to_bits(), b.base.phi_hp.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.n_lp.to_bits(), b.base.n_lp.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.n_hp.to_bits(), b.base.n_hp.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.slip.to_bits(), b.base.slip.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.base.mdot_air.to_bits(), b.base.base.mdot_air.to_bits(),
                       "{name} @ {tt4}");
            assert_eq!(a.base.thrust.to_bits(), b.base.base.thrust.to_bits(), "{name} @ {tt4}");
        }
    }
}

/// **GATE 1b.** The reduce is not a fast-gas artefact: it holds on the shipped reacting gas too.
#[test]
fn gate1b_reduce_holds_on_the_reacting_gas() {
    let reference = TwoSpoolMapMatcher::new(design(Gas::reacting_equilibrium()), flight(), 1.0,
                                            lp_shaped(), hp_shaped());
    let shut = bm(Gas::reacting_equilibrium(), lp_shaped(), hp_shaped(), 0.0);
    for tt4 in [1500.0, 1200.0] {
        let a = reference.match_point(&flight(), tt4).two();
        let b = shut.match_point(&flight(), tt4);
        assert_eq!(a.base.pi_lpc.to_bits(), b.base.base.pi_lpc.to_bits(), "@ {tt4}");
        assert_eq!(a.base.pi_hpc.to_bits(), b.base.base.pi_hpc.to_bits(), "@ {tt4}");
        assert_eq!(a.phi_lp.to_bits(), b.base.phi_lp.to_bits(), "@ {tt4}");
        assert_eq!(a.phi_hp.to_bits(), b.base.phi_hp.to_bits(), "@ {tt4}");
        assert_eq!(a.base.thrust.to_bits(), b.base.base.thrust.to_bits(), "@ {tt4}");
        assert_eq!(a.base.mdot_air.to_bits(), b.base.base.mdot_air.to_bits(), "@ {tt4}");
    }
}

/// **P7's OTHER HALF — THE ABSENCE IS A TYPE.**
///
/// Python never *constructs* a `TwoSpoolBleedResult` at `b = 0`; it returns rung 39's object,
/// whose four booking attributes do not exist, and `bleed_trade` reads that ABSENCE through
/// `getattr(od, "st_inlet", od.performance.specific_thrust)`. So the dataclass's `st_inlet = 0.0`
/// default is UNREACHABLE, and a port that always built the struct would write `0.0` into the
/// `b = 0` row where Python writes the core specific thrust — which a float dump compares equal.
///
/// **The `b = 0` row is vacuous for the thing it looks like it tests**, which is why this is a
/// TYPE assertion and the value sweeps above are at `b > 0`: at `b = 0`,
/// `st_inlet == specific_thrust` and `mdot_core == mdot_air` numerically, so every spelling
/// agrees there — a wrongly-built booking, a swapped `st_inlet`/`tsfc_inlet` pair and a defaulted
/// field are all invisible.
#[test]
fn p7_the_absence_is_a_type() {
    let shut = bm(fast_gas(), lp_shaped(), hp_shaped(), 0.0);
    for tt4 in THROTTLE {
        let od = shut.match_point(&flight(), tt4);
        assert!(od.booking.is_none(),
                "the valve is SHUT at Tt4={tt4}, so rung 39's body produced this point and there \
                 is no booking to read");
    }
    let mut open = bm(fast_gas(), lp_shaped(), hp_shaped(), 0.10);
    for tt4 in THROTTLE {
        let od = open.match_point(&flight(), tt4);
        let k = od.booking.expect("the valve is OPEN, so rung 42's body produced this point");
        assert_eq!(k.bleed.to_bits(), 0.10f64.to_bits());
        // ...and the numbers a `b = 0` row could not discriminate really do separate here.
        assert!(k.st_inlet < od.base.base.performance.specific_thrust,
                "the dumped air carries FULL ram drag, so the per-INLET specific thrust must be \
                 BELOW the core-referenced one");
        assert!(k.mdot_core < od.base.base.mdot_air);
    }
    open.set_bleed(0.0);
    assert!(open.match_point(&flight(), 1200.0).booking.is_none(),
            "moving the valve back must restore the rung-39 dispatch");
}

// ============================================================================== gate 2
/// **GATE 2 — THE ASYMMETRY.** `x_L = Tt4/Tt2` is built from two INPUTS, so bleed cannot move it
/// — hence the whole `dphi_L` is displacement OFF the LP running line: a new degree of freedom.
#[test]
fn gate2_x_lp_is_exactly_bleed_invariant() {
    for (name, ml, mh) in shapes() {
        let shut = bm(fast_gas(), ml, mh, 0.0);
        let open = bm(fast_gas(), ml, mh, 0.10);
        for tt4 in THROTTLE {
            let a = shut.match_point(&flight(), tt4);
            let c = open.match_point(&flight(), tt4);
            let x_l_a = tt4 / a.base.base.station("2").tt;
            let x_l_c = tt4 / c.base.base.station("2").tt;
            assert_eq!(x_l_a.to_bits(), x_l_c.to_bits(),
                       "{name} @ {tt4}: x_L moved under bleed");
            assert!(c.base.phi_lp / a.base.phi_lp - 1.0 > 0.05,
                    "{name} @ {tt4}: phi_L displacement too small to be the rung");
        }
    }
}

/// **GATE 2b — THE CONTRAST.** Take the bled HP point's `x_H`, find the `b = 0` THROTTLE setting
/// with the same `x_H`, and compare `phi_H`: the HP compressor is on ONE curve (bleed only slides
/// it along), while the LP at the SAME `x_L` is displaced by 100× more.
#[test]
fn gate2b_hp_stays_on_one_running_line() {
    for (name, ml, mh) in [("flat", flat(), flat()), ("flow/press", lp_shaped(), hp_shaped())] {
        let shut = bm(cpg_gas(), ml, mh, 0.0);
        let open = bm(cpg_gas(), ml, mh, 0.10);
        for tt4 in [1400.0, 1100.0, 900.0] {
            let c = open.match_point(&flight(), tt4);
            let x_h_target = tt4 / c.base.base.station("25").tt;
            let resid = |t: f64| {
                let o = shut.match_point(&flight(), t);
                t / o.base.base.station("25").tt - x_h_target
            };
            let (mut lo, mut hi) = (tt4, 1500.0f64.min(tt4 * 1.3));
            let (mut flo, fhi) = (resid(lo), resid(hi));
            assert!(flo * fhi <= 0.0, "{name} @ {tt4}: no bracket on the b=0 running line");
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                let fm = resid(mid);
                if flo * fm <= 0.0 { hi = mid; } else { lo = mid; flo = fm; }
            }
            let o = shut.match_point(&flight(), 0.5 * (lo + hi));
            let d_hp = (c.base.phi_hp / o.base.phi_hp - 1.0).abs();
            let a = shut.match_point(&flight(), tt4);          // same x_L as the bled point
            let d_lp = (c.base.phi_lp / a.base.phi_lp - 1.0).abs();
            assert!(d_hp < 5e-4, "{name} @ {tt4}: HP left its running line by {d_hp:.2e}");
            assert!(d_lp > 100.0 * d_hp,
                    "{name} @ {tt4}: contrast only {:.0}x (LP {d_lp:.3e}, HP {d_hp:.3e})",
                    d_lp / d_hp);
        }
    }
}

/// **GATE 2c — the mass-extraction identity.** The first STEADY mass extraction: the core carries
/// exactly `(1-b)` of the inlet air, and the station-25 flowpath split is booked explicitly.
///
/// `mdot` is the ONLY place the extraction is visible at all — `try_score` never touches mass
/// flow, so no downstream number would reveal a wrong split.
#[test]
fn gate2c_mass_extraction_identity() {
    for b in [0.05, 0.10] {
        let od = bm(fast_gas(), lp_shaped(), hp_shaped(), b).match_point(&flight(), 1200.0);
        let k = od.booking.expect("the valve is open");
        let (s2, s25) = (od.base.base.station("2"), od.base.base.station("25"));
        let s3 = od.base.base.station("3");
        assert!((k.mdot_core - (1.0 - b) * od.base.base.mdot_air).abs()
                    < 1e-12 * od.base.base.mdot_air, "b={b}");
        assert!((s3.mdot - (1.0 - b) * s25.mdot).abs() < 1e-12 * s25.mdot, "b={b}");
        assert_eq!(s2.mdot.to_bits(), s25.mdot.to_bits(),
                   "b={b}: nothing leaves the flowpath before station 25");
    }
}

// ============================================================================== gate 3
/// **GATE 3 — PERTURBATION-INDEPENDENCE (non-tautological).** `s_H` measured by opening the VALVE
/// equals rung 41's closed form, which was measured on the THROTTLE. Two perturbations, one
/// sensitivity.
///
/// NOT a tautology: only on a CPG gas at frozen `f` is the HP subsystem exactly one-parameter in
/// `x_H`. On the shipped gas the HP loop reads `(Tt4, Tt25, f)` SEPARATELY, so the collapse is a
/// measurement — which is why this is gated on CPG + flat, where it should be sharp.
#[test]
fn gate3_perturbation_independence() {
    let gas = cpg_gas();
    let shut = bm(gas.clone(), flat(), flat(), 0.0);
    let open = bm(gas.clone(), flat(), flat(), 0.02);
    let mut worst: f64 = 0.0;
    for tt4 in [1500.0, 1300.0, 1100.0, 1000.0, 900.0, 800.0, 750.0, 700.0] {
        let a = shut.match_point(&flight(), tt4);
        let c = open.match_point(&flight(), tt4);
        let x_h_a = tt4 / a.base.base.station("25").tt;
        let x_h_c = tt4 / c.base.base.station("25").tt;
        let s_meas = (c.base.phi_hp / a.base.phi_hp).ln() / (x_h_c / x_h_a).ln();
        let s_closed = s_h_closed(a.base.base.pi_hpc, gas.gamma_c());
        worst = worst.max((s_meas - s_closed).abs());
        assert!((s_meas - s_closed).abs() < 0.01,
                "Tt4={tt4}: bleed-derived s_H={s_meas:.4} vs closed form {s_closed:.4}");
    }
    assert!(worst > 1e-6,
            "suspiciously exact — check the two paths are really independent, not the same \
             arithmetic reached twice");
}

// ============================================================================== gate 4
/// **GATE 4 — `pi*` A THIRD TIME.** `dphi_H/db` passes through ZERO, and the crossing BRACKETS
/// `pi* = gamma_c^(gamma_c/(gamma_c-1))`.
///
/// Gated: the EXISTENCE of the sign reversal and that the bracket contains `pi*` within the
/// fuel-fraction residual rung 41 already isolated. NOT gated: the exact crossing — it rides on
/// `f`, on the map shape and on the gas, as rung 41's own turn does.
#[test]
fn gate4_the_response_reverses_at_pi_star() {
    let gas = cpg_gas();
    let shut = bm(gas.clone(), flat(), flat(), 0.0);
    let open = bm(gas.clone(), flat(), flat(), 0.02);
    let gc = gas.gamma_c();
    let pi_star = gc.powf(gc / (gc - 1.0));
    let mut rows: Vec<(f64, f64, f64)> = Vec::new();
    for tt4 in [900.0, 850.0, 820.0, 800.0, 790.0, 780.0, 770.0, 750.0, 700.0] {
        let a = shut.match_point(&flight(), tt4);
        let c = open.match_point(&flight(), tt4);
        rows.push((tt4, a.base.base.pi_hpc, (c.base.phi_hp / a.base.phi_hp).ln()));
    }
    let signs: Vec<bool> = rows.iter().map(|r| r.2 > 0.0).collect();
    assert!(signs[0] && !signs[signs.len() - 1],
            "no sign reversal in dphi_H/db across the band");
    let i = (1..rows.len()).find(|&j| signs[j] != signs[j - 1]).expect("a crossing");
    let (pi_hi, pi_lo) = (rows[i - 1].1, rows[i].1);        // pi falls with Tt4
    assert!(pi_lo < pi_star && pi_star < pi_hi,
            "crossing bracket ({pi_lo:.5}, {pi_hi:.5}) does not contain pi*={pi_star:.5}");
    // ...and the LP response does NOT reverse anywhere in that band — the contrast.
    for tt4 in [900.0, 800.0, 750.0, 700.0] {
        let a = shut.match_point(&flight(), tt4);
        let c = open.match_point(&flight(), tt4);
        assert!(c.base.phi_lp > a.base.phi_lp, "LP response reversed at Tt4={tt4}");
    }
}

// ============================================================================== gate 5
/// **GATE 5 — SELF-TARGETING, in phi-SPACE.** `dphi_L` is near-CONSTANT while `dphi_H` collapses,
/// so the FRACTION of the shrinking `(phi_op - phi_surge)` gap that the valve closes RISES on LP
/// and FALLS on HP.
///
/// Deliberately gated in phi-space. The relative-surge-margin version is CONFOUNDED: the ABSOLUTE
/// `dSM_L` shrinks, and only its collapsing base makes the ratio grow. Gating that would repeat
/// this project's own rung-41 lesson.
#[test]
fn gate5_self_targeting_in_phi_space() {
    let grid = [1500.0, 1300.0, 1100.0, 950.0, 900.0];
    for (name, ml, mh) in [("flow/press", lp_shaped(), hp_shaped()),
                           ("tilted", tilted(), tilted())] {
        for fl in [0.50, 0.55, 0.60] {
            let shut = bm_floor(cpg_gas(), ml, mh, 0.0, fl);
            let open = bm_floor(cpg_gas(), ml, mh, 0.10, fl);
            let (mut dphi_l, mut dphi_h) = (Vec::new(), Vec::new());
            let (mut frac_l, mut frac_h) = (Vec::new(), Vec::new());
            for tt4 in grid {
                let a = shut.match_point(&flight(), tt4);
                let c = open.match_point(&flight(), tt4);
                dphi_l.push(c.base.phi_lp - a.base.phi_lp);
                dphi_h.push(c.base.phi_hp - a.base.phi_hp);
                frac_l.push((c.base.phi_lp - a.base.phi_lp) / (a.base.phi_lp - fl));
                frac_h.push((c.base.phi_hp - a.base.phi_hp) / (a.base.phi_hp - fl));
            }
            let tag = format!("{name}/{fl}");
            let hi = dphi_l.iter().cloned().fold(f64::MIN, f64::max);
            let lo = dphi_l.iter().cloned().fold(f64::MAX, f64::min);
            assert!(hi / lo - 1.0 < 0.10, "{tag}: dphi_L not near-constant ({:.3})", hi / lo - 1.0);
            assert!(dphi_h[0] / dphi_h[dphi_h.len() - 1] > 5.0, "{tag}: dphi_H did not collapse");
            for i in 0..frac_l.len() - 1 {
                assert!(frac_l[i] < frac_l[i + 1],
                        "{tag}: LP fraction-closed not monotone rising toward low power");
                assert!(frac_h[i] > frac_h[i + 1],
                        "{tag}: HP fraction-closed not monotone falling");
            }
            assert!(frac_l[frac_l.len() - 1] > 2.0 * frac_l[0],
                    "{tag}: LP concentration too weak");
        }
    }
}

// ============================================================================== gate 6
/// **GATE 6 — THE TRADE.** Thrust falls / TSFC rises monotonically in `b`, and the thrust penalty
/// GROWS with throttle-down.
#[test]
fn gate6_the_trade() {
    let mut m = bm(fast_gas(), lp_shaped(), hp_shaped(), 0.0);
    let mut penalties: Vec<f64> = Vec::new();
    for tt4 in [1500.0, 1100.0, 900.0] {
        let rows = m.bleed_trade(&flight(), tt4, &[0.0, 0.05, 0.10]);
        let f: Vec<f64> = rows.iter().map(|r| r.thrust).collect();
        let s: Vec<f64> = rows.iter().map(|r| r.tsfc).collect();
        assert!(f[0] > f[1] && f[1] > f[2], "thrust not monotone in b at Tt4={tt4}: {f:?}");
        assert!(s[0] < s[1] && s[1] < s[2], "TSFC not monotone in b at Tt4={tt4}: {s:?}");
        penalties.push(1.0 - f[2] / f[0]);
    }
    for i in 0..penalties.len() - 1 {
        assert!(penalties[i] < penalties[i + 1],
                "the thrust penalty should GROW with throttle-down: {penalties:?}");
    }
    assert_eq!(m.bleed().to_bits(), 0.0f64.to_bits(),
               "bleed_trade must restore the valve — Python does it in a `finally`, and a port \
                that forgets leaves every later reading on the wrong machine");
}

/// **GATE 6b — THE ENVELOPE.** Bleed lowers `pi_LPC` hence `pt4`, so the inherited nozzle-choked
/// guard bites SOONER. This is the physics behind the dump's UNCHOKED census column.
#[test]
fn gate6b_the_valve_shrinks_the_envelope() {
    let mut lows: Vec<f64> = Vec::new();
    for b in [0.0, 0.10] {
        let m = bm(cpg_gas(), flat(), flat(), b);
        let (mut low, mut t) = (None, 900.0);
        while t > 400.0 {
            if m.try_match_point(&flight(), t).is_err() {
                break;
            }
            low = Some(t);
            t -= 5.0;
        }
        lows.push(low.expect("the top of the sweep must match"));
    }
    assert!(lows[1] > lows[0], "envelope did not shrink with bleed: {lows:?}");
}

// ============================================================================== gate 7
/// **GATE 7 — THE REFUTED HYPOTHESIS, kept visible.** The rung was proposed as "bleed protects LP
/// AT THE HP SPOOL'S EXPENSE". FALSE above `pi*`: the HP flow coefficient RISES too — just
/// 10–100× less. Asserted, not quietly dropped (rung 40's convention).
#[test]
fn gate7_the_refuted_hypothesis() {
    for (gname, gas) in [("cpg", cpg_gas()), ("tpg", fast_gas())] {
        for (name, ml, mh) in shapes() {
            let shut = bm(gas.clone(), ml, mh, 0.0);
            let open = bm(gas.clone(), ml, mh, 0.10);
            let a = shut.match_point(&flight(), TT4);
            let c = open.match_point(&flight(), TT4);
            assert!(c.base.phi_hp > a.base.phi_hp,
                    "{gname}/{name}: HP penalised at design — check pi_HPC");
            let gain_l = c.base.phi_lp / a.base.phi_lp - 1.0;
            let gain_h = c.base.phi_hp / a.base.phi_hp - 1.0;
            assert!(gain_l > 5.0 * gain_h,
                    "{gname}/{name}: selectivity only {:.1}x at design", gain_l / gain_h);
        }
    }
}

// ============================================================================== gate 8
/// **GATE 8 — CYCLE UNTOUCHED.** The default single-spool design run is bit-for-bit rung 6:
/// building AND exercising the bleed matcher must not perturb it.
#[test]
fn gate8_cycle_untouched_rung6() {
    let plain = || build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, ..Losses::default()
    });
    let before = plain().run(&flight(), 1.0);

    let mut m = bm(fast_gas(), lp_shaped(), hp_shaped(), 0.10);
    let _ = m.match_point(&flight(), 1200.0);
    let _ = m.bleed_trade(&flight(), 1200.0, &[0.0, 0.05]);

    let after = plain().run(&flight(), 1.0);
    assert_eq!(before.performance.specific_thrust.to_bits(),
               after.performance.specific_thrust.to_bits());
    assert_eq!(before.station("4").far.to_bits(), after.station("4").far.to_bits());
    assert_eq!(before.station("9").pt.to_bits(), after.station("9").pt.to_bits());
}

// ================================================================= the PORT-ONLY gates
/// **THE DISPATCH IS LIVE — § 5.8's owed gate, and the one Python cannot fail.**
///
/// Rung 42 is the port's first override of a live virtual slot. Rung 41's three schedule methods
/// call `self.match`, which on a rung-42 object must reach rung 42's body. Naming
/// `R39.try_match_point` in `R42`'s table compiles, returns plausible numbers, and silently gives
/// rung 42 rung 39's physics — so the gate is a SIGN, not a value: no value key can fake a margin
/// that is computed *from* a match only the override supplies.
///
/// **All THREE methods, not just the one `bleed_trade` happens to call — and each in its OWN
/// test.** The first pass of the step-3 smoke check exercised `surge_margin` alone, because that
/// is the only one on `bleed_trade`'s path — 1 of the 3 the slice's own headline names. The other
/// two are the ones with output nothing else covers: `flow_coefficient_turn` carries the nullable
/// columns and its `MIN`/`RAIL` branch can FLIP under bleed (bleed moves `phi`, hence the argmin
/// index), and `running_line_map` feeds nothing downstream at all.
///
/// **They are three tests and not three legs of one for a calibration reason.** Pointed
/// deliberately at `R39.try_match_point`, a single test fires on its FIRST leg and the other two
/// never run — so "all three witness the dispatch" would be a claim the calibration had not
/// touched. Split, each was confirmed to fail on its own. Measured: of this file's 15 tests,
/// **these three are the only ones that catch the defect at all** — every value gate reaches
/// `try_match_bleed` directly rather than through the hook, so rung 42's physics can be replaced
/// wholesale by rung 39's and they all still pass.
#[test]
fn the_dispatch_is_live_through_surge_margin() {
    let mut m = bm_floor(cpg_gas(), lp_shaped(), hp_shaped(), 0.0, 0.55);
    let rows = m.bleed_trade(&flight(), 1200.0, &[0.0, 0.05, 0.10]);
    let sm_lp: Vec<f64> = rows.iter().map(|r| r.sm_lp.expect("both maps armed")).collect();
    let sm_hp: Vec<f64> = rows.iter().map(|r| r.sm_hp.expect("both maps armed")).collect();
    for i in 0..rows.len() - 1 {
        assert!(sm_lp[i] < sm_lp[i + 1] && sm_hp[i] < sm_hp[i + 1],
                "a margin did not rise with b: {sm_lp:?} / {sm_hp:?} — margins INVARIANT in b \
                 would mean R42's hook slot never got rung 42's body, which compiles and returns \
                 numbers");
    }
}

/// See [`the_dispatch_is_live_through_surge_margin`]. `running_line_map` on a rung-42 core: same
/// design, same maps, same throttles — the ONLY difference is the valve, so any difference is the
/// dispatch. This method's output feeds nothing downstream, so no other number in the port would
/// reveal it reading the wrong body.
#[test]
fn the_dispatch_is_live_through_running_line_map() {
    let shut = bm(cpg_gas(), lp_shaped(), hp_shaped(), 0.0);
    let open = bm(cpg_gas(), lp_shaped(), hp_shaped(), 0.10);
    let grid = [1500.0, 1300.0, 1100.0, 950.0, 900.0];
    let a = shut.core.running_line_map(&flight(), &grid);
    let c = open.core.running_line_map(&flight(), &grid);
    assert_eq!(a.len(), c.len(), "the two sweeps must be comparable row for row");
    for (ra, rc) in a.iter().zip(c.iter()) {
        assert!(rc.phi_lp > ra.phi_lp,
                "running_line_map read rung 39's body on a rung-42 core at Tt4={}", ra.tt4);
        assert_eq!((ra.tt4 / ra.x_lp).to_bits(), (rc.tt4 / rc.x_lp).to_bits(),
                   "Tt2 is an INPUT, so x_lp's denominator cannot move under bleed");
    }
}

/// See [`the_dispatch_is_live_through_surge_margin`]. `flow_coefficient_turn` on a rung-42 core —
/// the method carrying P9's nullable columns, whose `MIN`/`RAIL` branch can FLIP under bleed
/// because bleed moves `phi` and therefore the argmin index.
#[test]
fn the_dispatch_is_live_through_flow_coefficient_turn() {
    let shut = bm(cpg_gas(), lp_shaped(), hp_shaped(), 0.0);
    let open = bm(cpg_gas(), lp_shaped(), hp_shaped(), 0.10);
    for spool in [Spool::Hp, Spool::Lp] {
        let ta = shut.core.flow_coefficient_turn(&flight(), spool);
        let tc = open.core.flow_coefficient_turn(&flight(), spool);
        assert!(ta.phi_star != tc.phi_star || ta.tt4_star != tc.tt4_star,
                "{spool:?}: flow_coefficient_turn located the SAME point with the valve open and \
                 shut — it is reading rung 39's body");
        // the nullable columns travel with the branch, on BOTH sides.
        for t in [&ta, &tc] {
            match t.kind {
                turbojet::two_spool::TurnKind::Min =>
                    assert!(t.pi_star.is_some() && t.star_form.is_some()
                            && t.gamma_c.is_some() && t.far.is_some()),
                turbojet::two_spool::TurnKind::Rail =>
                    assert!(t.pi_star.is_none() && t.star_form.is_none()
                            && t.gamma_c.is_none() && t.far.is_none()),
            }
        }
    }
}

/// **THE ROSTER.** Every `def test_` in `tests/test_rung42.py`, in file order — stated as data so
/// the count is auditable. All twelve port; see the module note for why this file has no
/// deferrals where `rung41.rs` has two.
#[test]
fn slice_l_roster() {
    let roster: [(&str, bool); 12] = [
        ("test_reduce_bleed_zero_is_rung39_bit_for_bit", true),
        ("test_reduce_bleed_zero_bit_for_bit_on_reacting_gas", true),
        ("test_x_lp_is_exactly_bleed_invariant_and_phi_lp_moves", true),
        ("test_hp_running_line_is_bleed_invariant_as_a_curve", true),
        ("test_mass_extraction_identity", true),
        ("test_bleed_derived_s_H_matches_rung41_closed_form", true),
        ("test_bleed_hp_response_reverses_sign_at_pi_star", true),
        ("test_self_targeting_is_a_phi_space_statement", true),
        ("test_trade_thrust_falls_tsfc_rises_and_the_cost_grows_with_throttle_down", true),
        ("test_opening_the_valve_shrinks_the_choked_envelope", true),
        ("test_bleed_does_not_penalise_the_hp_spool_at_design", true),
        ("test_cycle_untouched_rung6_bit_for_bit", true),
    ];
    assert_eq!(roster.len(), 12,
               "tests/test_rung42.py has 12 test functions — if that changed, this roster is \
                stale and the port is gating against a file that no longer exists");
    assert_eq!(roster.iter().filter(|(_, p)| *p).count(), 12,
               "all twelve port: nothing in rung 42's suite reaches a transient");
}
