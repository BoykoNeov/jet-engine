//! RUNG 36 — THE SURGE LINE: the excursion finally gets a boundary to be measured against.
//!
//! Rungs 32/34/35 reported the excursion as a distance above the RUNNING LINE and deliberately
//! drew no surge line, because a representative efficiency island is not a stability boundary and
//! any margin number rides on where the line is drawn. This rung imposes **one disclosed
//! constant** — a stall flow coefficient `phi_surge` — because the map's own loading-law peak
//! `1 - l/(2*sigma)` lands at `phi < 0` for the surge-realistic shapes, so there is no free
//! in-range stall point to inherit.
//!
//! **Every margin MAGNITUDE is therefore disclaimed, and the rung is careful to say so twice.**
//! What survives as load-bearing is a SIGN: the schedule is thin at LOW power, inherited from the
//! running-line `phi_op(Tt4)` that the choked hardware determines (rungs 31/32) rather than from
//! the imposed floor. Gate 6 exists to prove the *crossing* is NOT claimed — it flips with the
//! floor, on purpose.
//!
//! The seven gates of `tests/test_rung36.py`, in file order. All seven port.
//!
//! | # | `tests/test_rung36.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_surge_off_is_bit_for_bit` | [`gate1_the_surge_floor_is_a_pure_diagnostic`] |
//! | 2 | `test_pi_c_reproduction_non_tautological` | [`gate2_the_margin_is_measured_on_the_running_lines_own_map`] |
//! | 3 | `test_the_schedule_thin_at_low_power_sign_robust` | [`gate3_the_schedule_is_thin_at_low_power`] |
//! | 4 | `test_the_compounding_confirmation_and_sharpening` | [`gate4_the_compounding_confirms_and_sharpens`] |
//! | 5 | `test_currency_equivalence_airtight` | [`gate5_the_two_crossings_are_one_statement`] |
//! | 6 | `test_crossing_is_disclaimed_flips_with_floor` | [`gate6_the_crossing_is_disclaimed_and_flips_with_the_floor`] |
//! | 7 | `test_cycle_untouched_bit_for_bit_rung6` | [`gate7_the_design_cycle_is_untouched`] |
//!
//! **PLUS THE GATE `rung41.rs` DEFERRED TO PHASE 6.** `test_rung36_verdict_survives_but_its_
//! mechanism_is_corrected` is rung 41's, built on the SINGLE-spool transient, and slice L booked
//! it here (§ 5.12's inbox, § 5.13 prediction 9). It lands as
//! [`rung41_deferred_the_verdict_survives_while_its_mechanism_is_corrected`].
//!
//! **Python marks one of the seven `slow`; the Rust does not** — slice M's rule, *port the gate,
//! drop the marker, re-introduce `#[ignore]` only against a MEASURED cost.*

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::map::ComponentMap;
use turbojet::spool::SpoolTransient;

const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;
const PHI_SURGE: [f64; 3] = [0.55, 0.65, 0.75];
const SWEEP: [f64; 6] = [1500.0, 1300.0, 1100.0, 900.0, 800.0, 700.0];

fn flight() -> FlightCondition {
    FlightCondition { t0: 250.0, p0: 50_000.0, m0: 0.85 }
}

fn real() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
    }
}

fn design() -> Engine {
    build_turbojet(Gas::thermally_perfect(), PI_C, TT4, 50_000.0, real())
}

fn st(cmap: ComponentMap) -> SpoolTransient {
    SpoolTransient::new(design(), flight(), 1.0, cmap)
}

/// Python's `SHAPES` — the three surge-realistic map shapes, with their names for the messages.
fn shapes() -> [(&'static str, ComponentMap); 3] {
    [
        ("surge_flow", ComponentMap::surge_flow()),
        ("surge_pressure", ComponentMap::surge_pressure()),
        ("surge_tilted", ComponentMap::surge_tilted()),
    ]
}

fn catch<T>(f: impl FnOnce() -> T + std::panic::UnwindSafe) -> Result<T, String> {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = std::panic::catch_unwind(f);
    std::panic::set_hook(prev);
    out.map_err(|e| {
        e.downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "<non-string panic>".to_string())
    })
}

// ------------------------------------------------------------------------------------ gate 1
/// GATE 1 — the surge floor is a PURE diagnostic: attaching it to a map perturbs NOTHING on the
/// running line, to the BIT. And with no floor the surge methods refuse to invent a boundary, so
/// nothing silently runs off a zero surge line.
#[test]
fn gate1_the_surge_floor_is_a_pure_diagnostic() {
    for (name, shape) in shapes() {
        let s = st(ComponentMap::flow_dominated());
        let bare = shape;
        let withsurge = shape.with_phi_surge(0.65);
        for tt4 in [1400.0f64, 1000.0] {
            let eb = s.equilibrium(&flight(), tt4, Some(&bare));
            let es = s.equilibrium(&flight(), tt4, Some(&withsurge));
            assert_eq!(eb.pi_c.to_bits(), es.pi_c.to_bits(), "{name}: floor perturbed pi_c");
            assert_eq!(eb.nu.to_bits(), es.nu.to_bits(), "{name}: floor perturbed nu");
            assert_eq!(
                eb.flowcoef.to_bits(), es.flowcoef.to_bits(),
                "{name}: floor perturbed the running line"
            );
        }
        // `is_flat` ignores `phi_surge` — a flat map WITH a floor still reduces rung 32 to rung 31.
        assert!(ComponentMap::flat().with_phi_surge(0.7).is_flat());
    }

    // No floor => the surge margin refuses to invent a boundary.
    let err = catch(|| {
        st(ComponentMap::flow_dominated())
            .surge_margin(&flight(), 1200.0, Some(&ComponentMap::surge_flow()))
    })
    .expect_err("surge_margin must require a surge line (phi_surge > 0)");
    assert!(err.contains("surge line"), "unexpected panic message: {err}");
}

// ------------------------------------------------------------------------------------ gate 2
/// GATE 2 — the margin is measured on the SAME forward map that sets the running line.
///
/// `pi_c_map` evaluated at the OPERATING point `(n, phi_op)` reproduces the shipped equilibrium
/// `pi_c` to machine zero. Two code paths, one `pi_c` — so the surge margin is not a parallel
/// re-derivation that could drift away from the running line it is measured against.
#[test]
fn gate2_the_margin_is_measured_on_the_running_lines_own_map() {
    for (name, shape) in [shapes()[0], shapes()[1]] {
        let s = st(ComponentMap::flow_dominated());
        let cm = shape.with_phi_surge(0.65);
        for tt4 in [1500.0f64, 1200.0, 900.0, 700.0] {
            let eq = s.equilibrium(&flight(), tt4, Some(&cm));
            let pc = s
                .pi_c_map(&cm, eq.n, eq.flowcoef, eq.tt2)
                .expect("the operating point does positive work by construction");
            assert!(
                (pc - eq.pi_c).abs() <= 1e-12 * eq.pi_c,
                "{name}: pi_c_map != shipped pi_c at Tt4={tt4} ({pc} vs {})", eq.pi_c
            );
        }
    }
}

// ------------------------------------------------------------------------------------ gate 3
/// GATE 3 — THE SCHEDULE, and the rung's load-bearing claim. `SM_N` decreases monotonically as
/// `Tt4` falls (tightest at part power), with the SAME sign across three shapes × three imposed
/// floors, and the constant-FLOW definition agrees.
///
/// The sign is inherited from the running-line `phi_op(Tt4)`, which the choked hardware
/// determines — **not from the floor** — which is why sweeping the floor cannot change it. The
/// magnitude is disclaimed, and gate 6 makes that disclaimer testable.
#[test]
fn gate3_the_schedule_is_thin_at_low_power() {
    for (name, shape) in shapes() {
        let s = st(ComponentMap::flow_dominated());
        for phi_s in PHI_SURGE {
            let cm = shape.with_phi_surge(phi_s);
            let sched = s.surge_margin_schedule(&flight(), &SWEEP, Some(&cm));
            let sm_n: Vec<f64> = sched.iter().map(|x| x.sm_n).collect();
            let sm_f: Vec<f64> = sched.iter().map(|x| x.sm_flow).collect();
            for w in sm_n.windows(2) {
                assert!(w[1] < w[0], "SM_N not monotone-thinning ({name}, phi_s={phi_s}): {sm_n:?}");
            }
            assert!(
                sm_n[sm_n.len() - 1] < sm_n[0],
                "SM_N must be thinnest at the low-power end"
            );
            for w in sm_f.windows(2) {
                assert!(w[1] < w[0], "SM_flow sign disagrees with SM_N ({name}, phi_s={phi_s})");
            }
            // The MECHANISM: phi_op itself walks DOWN toward the fixed floor.
            let phis: Vec<f64> = sched.iter().map(|x| x.phi_op).collect();
            assert!(phis[phis.len() - 1] < phis[0]);
            for w in phis.windows(2) {
                assert!(
                    w[1] <= w[0] + 1e-12,
                    "running-line phi_op must walk down toward the stall floor as throttled"
                );
            }
        }
    }
}

// ------------------------------------------------------------------------------------ gate 4
/// GATE 4 — CONFIRMATION and SHARPENING, not relocation.
///
/// For a full-throttle burst, the consumed-margin ratio `E0/SM_N` rises monotonically as the start
/// power falls, because BOTH ingredients point low: `E0` rises AND `SM_N` falls. So the low-power
/// burst is most surge-critical on BOTH axes.
///
/// Nothing relocates — rung 34's `E0` was already largest at low power, so the two schedules are
/// parallel. The surge line's unique contribution is `SM_N`, the margin the excursion consumes,
/// and it is new information rather than a rescale of `E`.
#[test]
fn gate4_the_compounding_confirms_and_sharpens() {
    let lows = [1400.0f64, 1200.0, 1000.0, 900.0, 800.0, 700.0];
    for (name, shape) in shapes() {
        let s = st(ComponentMap::flow_dominated());
        let cm = shape.with_phi_surge(0.65);
        let rows: Vec<_> =
            lows.iter().map(|&lo| s.acceleration_binding(&flight(), lo, 1500.0, Some(&cm))).collect();
        let ratios: Vec<f64> = rows.iter().map(|r| r.ratio).collect();
        for w in ratios.windows(2) {
            assert!(w[1] > w[0], "E0/SM_N not monotone-rising toward low power ({name}): {ratios:?}");
        }
        assert!(
            rows[rows.len() - 1].e0 > rows[0].e0,
            "E0 must rise as start power falls (a bigger burst from a lower spool)"
        );
        assert!(
            rows[rows.len() - 1].sm_n < rows[0].sm_n,
            "SM_N must fall as start power falls (the running line nears surge)"
        );
    }
}

// ------------------------------------------------------------------------------------ gate 5
/// GATE 5 — `SM_N` is EXACTLY the currency the constant-speed excursion consumes.
///
/// `reaches_surge` (`E0 >= SM_N`, a pressure-ratio crossing) equals `phi_step_le_surge`
/// (`phi_step <= phi_surge`, a flow-coefficient crossing) at every tested point. The two are the
/// SAME statement, which is what makes the margin the right currency rather than a plausible one.
#[test]
fn gate5_the_two_crossings_are_one_statement() {
    for (name, shape) in shapes() {
        let s = st(ComponentMap::flow_dominated());
        let cm = shape.with_phi_surge(0.65);
        for lo in [1400.0f64, 1000.0, 800.0, 700.0] {
            let b = s.acceleration_binding(&flight(), lo, 1500.0, Some(&cm));
            assert_eq!(
                b.reaches_surge, b.phi_step_le_surge,
                "currency equivalence broken at Tt4_lo={lo} ({name}): E0>=SM_N is {} but \
                 phi_step<=phi_surge is {}", b.reaches_surge, b.phi_step_le_surge
            );
        }
    }
}

// ------------------------------------------------------------------------------------ gate 6
/// GATE 6 — the ANTI-OVERCLAIM gate, and the reason rung 36 is honest about its one constant.
///
/// `E0` is independent of `phi_surge` (a pure map displacement); only `SM_N` moves with the floor.
/// So for a FIXED burst there is a floor that surges and one that does not. **The test asserts the
/// flip EXISTS**, which certifies that the rung claims the trend (gate 3) and never the crossing
/// location — rung 32's warning, made testable.
#[test]
fn gate6_the_crossing_is_disclaimed_and_flips_with_the_floor() {
    let s = st(ComponentMap::flow_dominated());
    let cm_lo = ComponentMap::surge_flow().with_phi_surge(0.55); // wide floor
    let cm_hi = ComponentMap::surge_flow().with_phi_surge(0.65); // tight floor
    let (lo, hi) = (700.0, 1500.0);
    let b_lo = s.acceleration_binding(&flight(), lo, hi, Some(&cm_lo));
    let b_hi = s.acceleration_binding(&flight(), lo, hi, Some(&cm_hi));
    assert!((b_lo.e0 - b_hi.e0).abs() <= 1e-12, "E0 must not depend on phi_surge");
    assert!(b_lo.sm_n != b_hi.sm_n, "SM_N must move with phi_surge");
    assert!(
        b_lo.reaches_surge != b_hi.reaches_surge,
        "the crossing must flip with phi_surge — it is disclaimed, only the trend is claimed"
    );
}

// ------------------------------------------------------------------------------------ gate 7
/// GATE 7 — the default design run is bit-for-bit rung 6; the surge line is read-only.
#[test]
fn gate7_the_design_cycle_is_untouched() {
    let eng = design();
    let before = eng.run(&flight(), 1.0).performance.specific_thrust;
    let s = st(ComponentMap::surge_flow().with_phi_surge(0.65));
    let _ = s.surge_margin_schedule(&flight(), &[1400.0, 1000.0], None);
    let after = eng.run(&flight(), 1.0).performance.specific_thrust;
    assert!(
        (after - before).abs() < 1e-12,
        "using the surge line must not perturb the design run"
    );
}

// ======================================================== the gate rung 41 deferred to phase 6
/// **DEFERRED TO PHASE 6 BY SLICE L, DISCHARGED HERE** — `rung41.rs`'s roster item 12,
/// `test_rung36_verdict_survives_but_its_mechanism_is_corrected`. Slice L's own ledger called it
/// a `TwoSpoolTransient` gate; it is a SINGLE-spool one (rungs 34/36), which is why it waited for
/// this file rather than for slice R. § 5.12's inbox, § 5.13 prediction 9.
///
/// **RUNG 41's CORRECTION OF RUNG 36's STATED MECHANISM.** Rung 36 shipped the right verdict with
/// a single-channel attribution: *"the trend is set by `phi_op(Tt4)`"*. Rung 41 finds `phi_op` is
/// NOT monotone — it turns around at `pi* = gamma_c^(gamma_c/(gamma_c-1))`, which for a `pi_c=10`
/// single spool sits INSIDE rung 36's own choked envelope — while the margin keeps thinning. So
/// the attribution cannot be the whole story. Freezing one running-line coordinate at a time
/// separates the channels:
///
/// * **phi-WALK** — `n` frozen at the reference, `phi_op(Tt4)` live: rung 36's stated cause.
/// * **SPEED-LINE** — `phi` frozen, `n(Tt4)` live: `tau_c - 1 ~ n^2`, so the `pi_c` gap between
///   the running line and the floor collapses with `n`. The cause rung 36 omitted.
///
/// Rung 36's CONCLUSION is untouched — both channels are choked-hardware-determined and hence
/// floor-independent, so its sign-robustness argument survives. Only its reason is corrected.
#[test]
fn rung41_deferred_the_verdict_survives_while_its_mechanism_is_corrected() {
    let s = st(ComponentMap::flow_dominated());
    let cm = ComponentMap::surge_flow().with_phi_surge(0.65);
    let sweep = [1500.0f64, 1300.0, 1100.0, 900.0, 800.0];
    let rows: Vec<_> =
        sweep.iter().map(|&t| s.surge_margin_channels(&flight(), t, Some(&cm), None)).collect();

    // The FULL margin thins monotonically — rung 36's verdict, re-measured on rung 41's object.
    for w in rows.windows(2) {
        assert!(
            w[1].sm_n < w[0].sm_n,
            "rung 36's verdict must survive the decomposition: {:?}",
            rows.iter().map(|r| r.sm_n).collect::<Vec<_>>()
        );
    }

    // BOTH channels are live — neither is a rounding artefact of the other.
    let walk: Vec<f64> = rows.iter().map(|r| r.sm_phi_walk).collect();
    let line: Vec<f64> = rows.iter().map(|r| r.sm_speed_line).collect();
    assert!(
        walk.iter().any(|&x| (x - rows[0].sm_ref).abs() > 1e-6),
        "the phi-walk channel must MOVE, or rung 36's stated cause is dead"
    );
    assert!(
        line.iter().any(|&x| (x - rows[0].sm_ref).abs() > 1e-6),
        "the speed-line channel must MOVE, or rung 41 has nothing to correct"
    );

    // The SPEED-LINE channel thins monotonically all the way down — it is the one that never
    // reverses, which is rung 41's point about deep throttle.
    for w in line.windows(2) {
        assert!(w[1] < w[0], "the speed-line channel must thin monotonically: {line:?}");
    }

    // At the reference point every channel collapses onto the shipped margin — the decomposition
    // is anchored, not a parallel arithmetic.
    let at_ref = s.surge_margin_channels(&flight(), TT4, Some(&cm), None);
    for (label, v) in [
        ("phi_walk", at_ref.sm_phi_walk), ("speed_line", at_ref.sm_speed_line),
        ("ref", at_ref.sm_ref),
    ] {
        assert!(
            (v - at_ref.sm_n).abs() <= 1e-12 * at_ref.sm_n.abs(),
            "at the reference Tt4 the {label} channel must BE the shipped margin: {v} vs {}",
            at_ref.sm_n
        );
    }
}

/// The record of what ported.
#[test]
fn rung36_roster() {
    let roster: [(&str, bool); 7] = [
        ("test_reduce_surge_off_is_bit_for_bit", true),
        ("test_pi_c_reproduction_non_tautological", true),
        ("test_the_schedule_thin_at_low_power_sign_robust", true),
        ("test_the_compounding_confirmation_and_sharpening", true),
        ("test_currency_equivalence_airtight", true),
        ("test_crossing_is_disclaimed_flips_with_floor", true),
        ("test_cycle_untouched_bit_for_bit_rung6", true),
    ];
    assert_eq!(roster.len(), 7, "tests/test_rung36.py has 7 test functions");
    assert_eq!(roster.iter().filter(|(_, p)| *p).count(), 7, "all seven port");
    // Plus the one rung 41 deferred here. `rung41.rs`'s roster still says 10 of 12 ported THERE;
    // this is the eleventh, discharged in the file whose object it actually uses.
    println!(
        "rung36.rs: 7 ported + 1 discharged from rung41.rs's deferral ledger, 0 slow markers \
         (Python marks 1)"
    );
}
