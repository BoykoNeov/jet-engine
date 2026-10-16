//! RUNG 96 — A GOVERNOR THAT INFERS THRUST. Anchor: `docs/plans/rung96-anchor-inferred-thrust.md`; spec:
//! `docs/rung96-spec.md`.
//!
//! The CONSTRUCTION gates (anchor § 2, "no credit") first: thrust read directly is rung 95, the clean table is inert
//! with no generator, the loaded table's ends are rung 94's, the pressure-ratio gap IS the ram drag it cannot see, the
//! hold identity and the fixed point, the LP laws' step check. Then the scored claims H1–H4, pinned at a subset of the
//! spec's grid (the full grid is `docs/plans/scratch-archive/rung96/score.rs`).

use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor::{self as gv, Clamp, Leg};
use turbojet::inferred_thrust as inf;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ControlsSolver, CustomerMode, GeneratorMode, LeverChoice,
                                 SpeedLaw, ThrustBy, DS, GAIN_MAX_LP, TAU_MIN_LP};

fn d() -> ControlsSettings { ControlsSettings { govern: true, hold: Hold::Thrust, ..ControlsSettings::defaults() } }
fn epr() -> ControlsSettings { ControlsSettings { thrust_by: ThrustBy::Epr, ..d() } }
fn n1() -> ControlsSettings { ControlsSettings { thrust_by: ThrustBy::N1, ..d() } }
const FLOW_PRESS: usize = 0;
const FLAT_LP: usize = 3;
fn bits(p: &[FuelPoint]) -> Vec<[u64; 4]> { p.iter().map(|q| [q.nu_lp, q.nu_hp, q.mf, q.tt4].map(f64::to_bits)).collect() }
/// `F_true − F̂` over the ramp's points `0 < s ≤ ramp`.
fn gaps(sv: &ControlsSolver, p: &[FuelPoint]) -> Vec<f64> {
    p.iter().filter(|x| x.s > 0.0 && x.s <= sv.settings.ramp + 1e-9).map(|x| {
        let r = sv.read(x).unwrap();
        r.thrust - r.inferred.unwrap()
    }).collect()
}

// =============================================================================================
// CONSTRUCTION
// =============================================================================================

#[test]
fn thrust_read_directly_is_rung95_and_the_new_knobs_move_nothing_there() {
    // Read from the thrust itself, the table switch and the reading's own knobs are inert, bit for bit; off the thrust
    // hold the reading is ignored, as rung 95's speed knobs are off the speed hold.
    let gen = ControlsSettings { generator: GeneratorMode::Hp, ..d() };
    let speed = ControlsSettings { hold: Hold::HpSpeed, ..d() };
    for (a, b) in [(gen, ControlsSettings { table_clean: false, ..gen }),
                   (speed, ControlsSettings { thrust_by: ThrustBy::Epr, table_clean: false, ..speed })] {
        let (sva, x) = controls(&a).unwrap();
        let (_, y) = controls(&b).unwrap();
        assert_eq!(bits(&x.points), bits(&y.points));
        assert!(sva.governor.as_ref().unwrap().infer.is_none(), "no table without an inferred reading");
    }
}

#[test]
fn with_the_generator_off_the_clean_table_is_inert_and_the_loaded_tables_ends_are_rung94s() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        for base in [epr(), n1()] {
            let (_, a) = controls(&ControlsSettings { shape, ..base }).unwrap();
            let (_, b) = controls(&ControlsSettings { shape, table_clean: false, ..base }).unwrap();
            assert_eq!(bits(&a.points), bits(&b.points));
            // the loaded table: the ends are rung 94's hold ends, the same instants
            let r94 = build(&ControlsSettings { shape, generator: GeneratorMode::Hp, govern: false, ..d() }).unwrap();
            let lt = build(&ControlsSettings { shape, generator: GeneratorMode::Hp, table_clean: false, ..base }).unwrap();
            assert!(r94.start == lt.start && r94.end == lt.end);
        }
    }
}

#[test]
fn the_pressure_ratio_gap_is_the_ram_drag_it_cannot_see() {
    // Anchor G2/G3: equal pressure ratio gives equal gross thrust and gross thrust is affine in it, so the table carries
    // the gross part exactly and F_true − F̂ = V0·(ṁ̂_face − ṁ_face). Bar frozen at 1e−12 × thrust (measured 1.2e−13).
    let mut worst = 0.0f64;
    for s in [epr(), ControlsSettings { from: 1400.0, to: 1000.0, ..epr() },
              ControlsSettings { generator: GeneratorMode::Hp, ..epr() },
              ControlsSettings { generator: GeneratorMode::Lp, table_clean: false, ..epr() },
              ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..epr() },
              ControlsSettings { redline_on: true, accel_on: true, ..epr() },
              ControlsSettings { lever: LeverChoice::Bleed, ..epr() },
              ControlsSettings { customer: CustomerMode::Share, ..epr() }] {
        for shape in [FLOW_PRESS, FLAT_LP] {
            let (sv, o) = controls(&ControlsSettings { shape, ..s }).unwrap();
            assert_eq!(o.points.len(), s.expected_points());
            let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
            for p in &o.points {
                let r = sv.read(p).unwrap();
                let i = sv.reread(p).unwrap().base;
                let ram = t.v0 * (t.face_at_epr(inf::epr(&i, t.pt2)) - inf::face(&i));
                worst = worst.max(((r.thrust - r.inferred.unwrap() - ram) / r.thrust).abs());
            }
        }
    }
    assert!(worst <= 1e-12, "{worst:e}");
}

#[test]
fn the_inferred_run_settles_on_its_end_and_holds_its_reading() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        for s in [epr(), n1(), ControlsSettings { generator: GeneratorMode::Hp, ..epr() },
                  ControlsSettings { generator: GeneratorMode::Lp, ..n1() }] {
            let s = ControlsSettings { shape, settle: 8.0, ..s };
            let (sv, o) = controls(&s).unwrap();
            assert_eq!(o.points.len(), s.expected_points());
            let l = o.points.last().unwrap();
            // Holding the fan speed pins the LP shaft at once and leaves the HP shaft to creep to its end on its own,
            // slowly (the spec's § 4: ~3e-6 short after 8 τ on flat-LP) — so its bar is looser than rung 95's 1e-6.
            let bar = if s.thrust_by == ThrustBy::N1 { 1e-5 } else { 1e-6 };
            assert!((l.nu_lp - sv.end.nu_lp).abs() < bar && (l.nu_hp - sv.end.nu_hp).abs() < bar, "{s:?}");
            let g = sv.governor.as_ref().unwrap();
            for (p, m) in o.points.iter().zip(&o.marks) {
                assert_eq!((m.clamp, m.leg), (Clamp::Root, Leg::None));
                let i = sv.reread(p).unwrap().base;
                let q = gv::held_by(g, &sv.core.fuel, &i);
                match g.law {
                    gv::Law::Epr => assert!((q / g.demand(p.s) - 1.0).abs() < 1e-12),
                    gv::Law::LpRate { tau } => assert!((q - (g.demand(p.s) - p.nu_lp) / tau).abs() < 1e-10),
                    _ => unreachable!(),
                }
            }
            // on its root the pressure-ratio control believes exactly the demanded thrust (the table inverts itself)
            if g.law == gv::Law::Epr {
                for p in &o.points {
                    assert!((sv.read(p).unwrap().inferred.unwrap() / g.thrust_demand(p.s) - 1.0).abs() < 1e-12);
                }
            }
        }
    }
}

#[test]
fn the_lp_laws_step_check_holds_at_their_limits() {
    // The LP laws' OWN step check (the spec's § 4): at the page's step against four times finer, flow coefficients to
    // 1e-4 on the acceleration, at the fastest rate and the largest gain the page runs.
    for s in [ControlsSettings { gov_tau: TAU_MIN_LP, ..n1() },
              ControlsSettings { speed_law: SpeedLaw::Prop, gov_gain: GAIN_MAX_LP, ..n1() }, epr()] {
        for shape in [FLOW_PRESS, FLAT_LP] {
            let sv = build(&ControlsSettings { shape, ..s }).unwrap();
            let g = sv.governor.as_ref().unwrap();
            let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
            let (a, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), DS);
            let (b, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), DS / 4.0);
            let dm = a.iter().enumerate().map(|(k, p)| (p.phi_lp - b[4 * k].phi_lp).abs().max((p.phi_hp - b[4 * k].phi_hp).abs()))
                .fold(0.0, f64::max);
            assert!(dm < 1e-4, "{:?}: {dm:e}", g.law);
        }
    }
}

#[test]
fn every_inferred_step_reruns_bit_for_bit() {
    use turbojet::sandbox_controls::{replay_step, stop_reason};
    for s in [ControlsSettings { redline_on: true, accel_on: true, ..epr() },
              ControlsSettings { floor_on: true, ..n1() },
              ControlsSettings { generator: GeneratorMode::Lp, redline_on: true, ..epr() },
              ControlsSettings { speed_law: SpeedLaw::Prop, from: 1400.0, to: 1000.0, ..n1() }] {
        let (sv, o) = controls(&s).unwrap();
        for w in o.points.windows(2) {
            let (a, b, _) = replay_step(&sv, &w[0]).unwrap().unwrap();
            assert_eq!((a.to_bits(), b.to_bits()), (w[1].nu_lp.to_bits(), w[1].nu_hp.to_bits()));
        }
        assert!(stop_reason(&sv, &o.points).is_none());
    }
}

// =============================================================================================
// SCORED — H1–H4 (anchor § 3), all HIT; pinned on two maps
// =============================================================================================

#[test]
fn h1_h3_both_readings_understate_the_thrust_through_an_acceleration_and_overstate_it_through_a_slowdown() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        for base in [epr(), n1()] {
            for (from, to, above) in [(1000.0, 1400.0, true), (1400.0, 1000.0, false)] {
                let (sv, o) = controls(&ControlsSettings { shape, from, to, ..base }).unwrap();
                let g = gaps(&sv, &o.points);
                assert_eq!(g.len(), 25);
                assert!(g.iter().all(|&x| (x > 0.0) == above), "{:?} shape {shape} {from}->{to}: {g:?}", base.thrust_by);
            }
        }
    }
}

#[test]
fn h2_the_pressure_ratio_acceleration_is_hotter_and_nearer_stall_than_rung95s() {
    let peak = |o: &[FuelPoint]| o.iter().map(|p| p.tt4).fold(f64::NEG_INFINITY, f64::max);
    let low = |o: &[FuelPoint], f: fn(&FuelPoint) -> f64| o.iter().enumerate()
        .fold((0, f64::INFINITY), |m, (k, p)| if f(p) < m.1 { (k, f(p)) } else { m });
    for shape in [FLOW_PRESS, FLAT_LP] {
        let (_, e) = controls(&ControlsSettings { shape, ..epr() }).unwrap();
        let (_, t) = controls(&ControlsSettings { shape, ..d() }).unwrap();
        assert!(peak(&e.points) > peak(&t.points));
        for f in [(|p: &FuelPoint| p.phi_lp) as fn(&FuelPoint) -> f64, |p: &FuelPoint| p.phi_hp] {
            let (a, b) = (low(&e.points, f), low(&t.points, f));
            assert!(a.0 > 0 && b.0 > 0, "interior minima (else VACUOUS)");
            assert!(a.1 < b.1);
        }
    }
}

#[test]
fn h4_calibrated_on_the_clean_engine_the_pressure_ratio_gives_more_thrust_than_asked_with_a_generator() {
    for shape in [FLOW_PRESS, FLAT_LP] {
        for g in [GeneratorMode::Hp, GeneratorMode::Lp] {
            let sv = build(&ControlsSettings { shape, generator: g, ..epr() }).unwrap();
            for h in [sv.hold_start.unwrap(), sv.hold_end.unwrap()] {
                assert!(h.achieved > h.target, "{g:?} shape {shape}: {h:?}");
            }
        }
    }
}
