//! RUNG 86 — FUEL + BLEED + STATOR on one plant: the CUBE. `docs/rung86-spec.md`; anchor
//! `docs/plans/rung86-anchor-three-levers.md` (§ refs are to it; § 3's bars were committed in
//! `abe7c5a` before any of them was measured).
//!
//! Gate groups:
//!
//!   1. THE REDUCES (R1–R5) — every two-lever face of the cube IS the earlier rung's own
//!      instrument, bit for bit: rung 63's `lever_composite` (stator off, and with the stator as
//!      its `neighbour`), rung 58's `composite_credit` (bleed off), the four leg-free siblings
//!      (leg off); and the cube's own march with a plain leg IS `stator_march`.
//!   2. THE ARITHMETIC — `T`'s three spellings agree (one alternating sum, not a finding), and a
//!      cell that is not the machine it names is refused.
//!   3. THE EXPERIMENT — § 3's bars, each gate naming its HIT / MISS / no-credit.

use turbojet::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use turbojet::engine::{FlightCondition};
use turbojet::fuel_transient::{AccelSchedule, AsymmetricLag};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    CellRead, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg, StatorSchedule,
};
use turbojet::three_levers::{cube, march, ArmedLimiter, Cube, Limiter};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------------------- rung 63's rig
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const N_LO: f64 = 0.65;
const V: f64 = 0.20;
const B: f64 = 0.10;
const MARGIN: f64 = 0.25;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn tilt_map() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn bt_on(lp: ComponentMap, hp: ComponentMap) -> ScheduledStatorCore {
    match build_scheduled_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn bt() -> ScheduledStatorCore {
    bt_on(lp_map(), hp_map())
}

fn bled() -> LeverArm {
    LeverArm::scheduled(BleedSchedule::new(B, N_LO))
}

fn stat() -> LeverArm {
    LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(V, N_LO)))
}

fn ramp(r: f64, ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds }
}

fn cube_on(m: &ScheduledStatorCore, r: f64, ds: f64, lim: Limiter) -> Cube {
    cube(m, &flight(), &ramp(r, ds), Spool::Lp, &stat(), &bled(), lim)
}

const ACCEL: Limiter = Limiter::Accel { margin: MARGIN };

/// Two cell reads are the SAME march read the same way — bit for bit, `s_eng`'s NaN included.
fn same(a: &CellRead, b: &CellRead, what: &str) {
    let bits = |x: f64| x.to_bits();
    assert_eq!(a.npts, b.npts, "{what}: npts");
    assert_eq!(bits(a.m_i), bits(b.m_i), "{what}: m_i {} vs {}", a.m_i, b.m_i);
    assert_eq!(bits(a.s), bits(b.s), "{what}: s");
    assert_eq!(bits(a.v), bits(b.v), "{what}: v");
    assert_eq!(bits(a.min_phi), bits(b.min_phi), "{what}: min_phi");
    assert_eq!(bits(a.fuel_removed), bits(b.fuel_removed), "{what}: removed");
    assert_eq!(bits(a.s_eng), bits(b.s_eng), "{what}: s_eng");
    assert_eq!(bits(a.tt4_peak), bits(b.tt4_peak), "{what}: tt4_peak");
    assert!(a.prof.len() == b.prof.len()
                && a.prof.iter().zip(&b.prof).all(|(x, y)| bits(x.0) == bits(y.0) && bits(x.1) == bits(y.1)),
            "{what}: profile");
}

// =============================================================================================
// GATE 1 — THE REDUCES
// =============================================================================================

/// R1 + R2: the cube's `S = 0` and `S = 1` faces ARE rung 63's `lever_composite`, without and with
/// the stator as its `neighbour` — the same four cells, bit for bit, and so `I_BF|S` is rung 63's
/// `interaction` on each face (rung 63's published +0.007658 on the first).
#[test]
fn r1_r2_the_stator_faces_are_rung63s_lever_composite() {
    let m = bt();
    let c = cube_on(&m, 0.5, DS, ACCEL);
    let leg: AccelSchedule = m.fuel.accel_schedule(&flight(), LO, HI, MARGIN, 13);
    let sl = StatorLeg { accel: Some(&leg), surge: None, tt4_max: None };
    for (s, nb) in [(0, None), (1, Some(&stat()))] {
        let lc = m.lever_composite(&flight(), &ramp(0.5, DS), &bled(), &sl, Spool::Lp, nb);
        same(&c.at(s, 0, 0).read, &lc.neither, "neither");
        same(&c.at(s, 1, 0).read, &lc.lever, "bleed");
        same(&c.at(s, 0, 1).read, &lc.fuel, "fuel");
        same(&c.at(s, 1, 1).read, &lc.both, "both");
        assert_eq!(c.i_bf[s].to_bits(), lc.interaction.to_bits(), "S={s}");
    }
    assert!((c.i_bf[0] - 0.007658).abs() < 5e-7, "rung 63's published interaction: {}", c.i_bf[0]);
}

/// R3: the `B = 0` face IS rung 58's `composite_credit` on the stator-armed sibling.
#[test]
fn r3_the_bleed_off_face_is_rung58s_composite_credit() {
    let m = bt();
    let c = cube_on(&m, 0.5, DS, ACCEL);
    let leg: AccelSchedule = m.fuel.accel_schedule(&flight(), LO, HI, MARGIN, 13);
    let sl = StatorLeg { accel: Some(&leg), surge: None, tt4_max: None };
    let cc = m.at_lever(&stat()).composite_credit(&flight(), &ramp(0.5, DS), Spool::Lp, &sl);
    same(&c.at(0, 0, 0).read, &cc.cells.neither, "neither");
    same(&c.at(1, 0, 0).read, &cc.cells.stator, "stator");
    same(&c.at(0, 0, 1).read, &cc.cells.fuel, "fuel");
    same(&c.at(1, 0, 1).read, &cc.cells.both, "both");
    assert_eq!(c.i_sf[0].to_bits(), cc.interaction.to_bits());
}

/// R4: the `F = 0` face is the four `at_lever` siblings' own leg-free cells — rung 62's pair.
#[test]
fn r4_the_leg_free_face_is_rung62s_pair() {
    let m = bt();
    let c = cube_on(&m, 0.5, DS, ACCEL);
    let free = StatorLeg::default();
    let arms = [LeverArm::default(), bled(), stat(), LeverArm::merged(&stat(), &bled())];
    for (k, (s, b)) in [(0, 0), (0, 1), (1, 0), (1, 1)].into_iter().enumerate() {
        let cell = m.at_lever(&arms[k]).cell(&flight(), &ramp(0.5, DS), Spool::Lp, &free);
        same(&c.at(s, b, 0).read, &cell, &format!("S{s}B{b}"));
    }
}

/// R5: the cube's own march (rung 57's body, reached through `ramp_march` so the LAGGED limiters
/// can ride it) IS `stator_march` for every plain limiter, on the most-armed sibling — the one
/// where the map reset and the ceiling switch both matter.
#[test]
fn r5_the_cubes_march_is_stator_march_for_every_plain_limiter() {
    let m = bt();
    let both = m.at_lever(&LeverArm::merged(&stat(), &bled()));
    let rp = ramp(0.5, DS);
    for lim in [ACCEL, Limiter::Topping { tt4_max: 1480.0 }, Limiter::IncidenceFloor { sm: 0.42 },
                Limiter::PhiFloor { sm: 0.42 }] {
        let a = ArmedLimiter::on(&m.at_lever(&LeverArm::default()), &flight(), &rp, Spool::Lp, lim);
        let (mine, n0) = march(&both, &flight(), &rp, Some(&a));
        let (theirs, n1) = both.stator_march(&flight(), &rp, None, &a.leg());
        assert_eq!((n0.0.to_bits(), n0.1.to_bits()), (n1.0.to_bits(), n1.1.to_bits()), "{lim:?}");
        assert_eq!(mine.len(), theirs.len(), "{lim:?}");
        for (p, q) in mine.iter().zip(&theirs) {
            assert!(p.s.to_bits() == q.s.to_bits() && p.mf.to_bits() == q.mf.to_bits()
                        && p.nu_lp.to_bits() == q.nu_lp.to_bits() && p.nu_hp.to_bits() == q.nu_hp.to_bits()
                        && p.phi_lp.to_bits() == q.phi_lp.to_bits() && p.tt4.to_bits() == q.tt4.to_bits(),
                    "{lim:?} at s={}", p.s);
        }
    }
}

// =============================================================================================
// GATE 2 — THE ARITHMETIC
// =============================================================================================

/// `T` is ONE alternating sum over the cube; its three spellings agree to rounding. Arithmetic,
/// not a finding (§ 1).
#[test]
fn the_three_spellings_of_t_are_one_sum() {
    let c = cube_on(&bt(), 0.5, DS, ACCEL);
    assert!((c.t_bf - c.t).abs() < 1e-15 && (c.t_sf - c.t).abs() < 1e-15, "{} {} {}", c.t, c.t_bf, c.t_sf);
    assert!(c.full_length());
}

/// The cube refuses a machine that already carries a lever — every cell must be a fresh sibling
/// of the BARE machine, or the alternating sum isolates nothing.
#[test]
#[should_panic(expected = "BARE machine")]
fn the_cube_refuses_an_armed_machine() {
    let m = bt();
    cube_on(&m.at_lever(&stat()), 0.5, DS, ACCEL);
}

// =============================================================================================
// GATE 3 — THE EXPERIMENT (§ 3). Six cells: shaped / tilted × r ∈ {0.25, 0.5, 1.0}.
// =============================================================================================

fn grid() -> Vec<(&'static str, ScheduledStatorCore, f64)> {
    let mut v = Vec::new();
    for r in [0.25, 0.5, 1.0] {
        v.push(("shaped", bt(), r));
        v.push(("tilted", bt_on(tilt_map(), tilt_map()), r));
    }
    v
}

const PAIRS: [(usize, usize); 4] = [(0, 0), (0, 1), (1, 0), (1, 1)];

/// A1 — HIT (5 scored cells; P1 had seen shaped r = 0.5, gated here but not scored). The leg
/// reads the stator–bleed pair EARLIER, before its interaction has built up: `T` carries the sign
/// of `−I_SB|F=0` and never exceeds it. A3 — HIT: rung 58's predictor, built from the LEG-FREE
/// runs on each stator face, recovers `T` at 71–93 %. The step-size bar holds: sign, order and
/// both ratios at `ds` = 0.0025 too.
#[test]
fn a1_a3_the_accel_leg_rereads_the_pair_earlier() {
    for (shape, m, r) in grid() {
        for ds in [DS, DS / 2.0] {
            let c = cube_on(&m, r, ds, ACCEL);
            assert!(c.full_length(), "{shape} r={r}");
            let what = format!("{shape} r={r} ds={ds}: T={} I_SB={:?} pred={}", c.t, c.i_sb, c.predicted);
            assert!(c.t * c.i_sb[0] < 0.0 && c.t.abs() < c.i_sb[0].abs(), "A1 {what}");
            let rec = c.predicted / c.t;
            assert!((0.5..=1.5).contains(&rec), "A3 {what}: {rec}");
            // measured: T is 7–16 % of the pair, and the predictor recovers 0.71–0.93
            assert!((0.06..0.17).contains(&(c.t / -c.i_sb[0])), "{what}");
            assert!((0.70..0.94).contains(&rec), "{what}: {rec}");
        }
    }
}

/// A2 — HIT: at `r` = 1.00 the leg engages after the BARE minimum, yet `T` ≠ 0, because every
/// lever moves the minimum later (§ 0 P2) and the leg reaches those cells. The bare cell's own
/// equality is no credit (rung 63 gates it) — and is NOT grid-robust on the tilted map: at
/// `ds` = 0.005 the leg's engagement falls inside the refine stencil and moves the reading.
#[test]
fn a2_at_r1_the_leg_misses_the_bare_cell_but_reaches_the_levers() {
    for (shape, m, r) in grid().into_iter().filter(|g| g.2 == 1.0) {
        let fine = cube_on(&m, r, DS / 2.0, ACCEL);
        assert_eq!(fine.reached, [false, true, true, true], "{shape}");
        assert!(fine.t > 1.5e-3, "{shape}: {}", fine.t);
        let coarse = cube_on(&m, r, DS, ACCEL);
        assert_eq!(coarse.reached[0], shape == "tilted", "{shape}: the stencil artefact, as measured");
    }
}

/// A4 — HIT: rung 52's attack/release lag keeps `T`'s sign, at 71–90 % of the plain leg's.
#[test]
fn a4_the_release_lag_keeps_the_sign() {
    let lag = Limiter::AccelLag { margin: MARGIN, lag: AsymmetricLag::new(0.02, 0.10) };
    for (shape, m, r) in grid() {
        let (a, b) = (cube_on(&m, r, DS, ACCEL), cube_on(&m, r, DS, lag));
        let q = b.t / a.t;
        assert!((0.70..0.91).contains(&q), "{shape} r={r}: {q}");
    }
}

/// B1 — HIT in every cell: the turbine limiter reaches a lever cell IFF that cell's leg-free run
/// passes the redline by the last point of its minimum's stencil, and an unreached cell is its
/// leg-free self bit for bit. B2 — HIT: none reached at `r` = 1 (`T` = 0.0 exactly); all four at
/// 0.25; at 0.5 the SB cell is reached — and so is B, so the clause "each lever alone shows
/// nothing" is NOT met. B3 — HIT: the lag reaches exactly the cells the instant limiter does.
#[test]
fn b_the_turbine_limiter_is_a_ledger_of_hot_minima() {
    for (shape, m, r) in grid() {
        let inst = cube_on(&m, r, DS, Limiter::Topping { tt4_max: 1480.0 });
        let lag = cube_on(&m, r, DS, Limiter::ToppingLag { tt4_max: 1480.0, tau_gov: 0.2 });
        for (k, (s, b)) in PAIRS.into_iter().enumerate() {
            let hot = inst.at(s, b, 0).tt4_to_stencil > 1480.0;
            assert_eq!(inst.reached[k], hot, "B1 {shape} r={r} S{s}B{b}");
            if !hot {
                // The MINIMUM's reading is the leg-free one bit for bit — not the whole run: the
                // governor still trims the later, hotter fuel (`removed` > 0, a lower peak).
                let (l, f) = (&inst.at(s, b, 1).read, &inst.at(s, b, 0).read);
                assert!(l.m_i.to_bits() == f.m_i.to_bits() && l.s.to_bits() == f.s.to_bits()
                            && l.v.to_bits() == f.v.to_bits(), "B1 {shape} r={r} S{s}B{b}");
            }
        }
        assert_eq!(lag.reached, inst.reached, "B3 {shape} r={r}");
        let want = if r == 1.0 { [false; 4] } else if r == 0.5 { [false, true, false, true] } else { [true; 4] };
        assert_eq!(inst.reached, want, "B2 {shape} r={r}");
        if r == 1.0 {
            assert_eq!(inst.t, 0.0);
        }
    }
}

/// B's step-size reading, measured: the LAGGED limiter's `T` is grid-stable (<1 %); the INSTANT
/// limiter's moves 25–34 % between `ds` 0.005 and 0.0025 where it acts — same sign, same order,
/// so the bar is met, but its SIZE is reported as grid-sensitive.
#[test]
fn b_the_instant_limiters_size_is_grid_sensitive_the_lags_is_not() {
    let m = bt();
    let inst = Limiter::Topping { tt4_max: 1480.0 };
    let lag = Limiter::ToppingLag { tt4_max: 1480.0, tau_gov: 0.2 };
    let (a, b) = (cube_on(&m, 0.5, DS, inst).t, cube_on(&m, 0.5, DS / 2.0, inst).t);
    assert!(a > 0.0 && b > 0.0 && (b / a - 1.0).abs() > 0.2, "{a} {b}");
    let (a, b) = (cube_on(&m, 0.5, DS, lag).t, cube_on(&m, 0.5, DS / 2.0, lag).t);
    assert!((b / a - 1.0).abs() < 0.01, "{a} {b}");
}

const SMS: [f64; 6] = [0.30, 0.34, 0.38, 0.42, 0.46, 0.50];

/// C1 — HIT: the incidence floor's whole legged face is `max(c_xy0, m_lim)` — from the four
/// leg-free cells and the set point alone — in every cell at every set point, and so is `T`.
/// (Above all four minima it erases the pair: rung 60's pinning — no credit.)
#[test]
fn c1_the_incidence_floor_is_a_ledger() {
    for (shape, m, r) in grid() {
        for sm in SMS {
            let c = cube_on(&m, r, DS, Limiter::IncidenceFloor { sm });
            let (led, t_led) = c.incidence_ledger().unwrap();
            for (k, (s, b)) in PAIRS.into_iter().enumerate() {
                assert!((c.m(s, b, 1) - led[k]).abs() < 1e-6, "{shape} r={r} sm={sm} S{s}B{b}");
                // DORMANT over the WHOLE run, not by the reading's own definition: a floor below
                // the run's minimum never binds, so it withholds no fuel at all.
                if c.m(s, b, 0) > c.set_point.unwrap() {
                    assert_eq!(c.at(s, b, 1).read.fuel_removed, 0.0, "{shape} r={r} sm={sm} S{s}B{b}");
                }
            }
            assert!((c.t - t_led).abs() < 2e-6, "{shape} r={r} sm={sm}");
        }
    }
}

/// C2 — HIT on the stator-OFF face, MISS on the stator-ON face. The `φ` floor binds IFF the
/// leg-free `min φ` is below `φ_lim`, and an unbound cell is its leg-free self, everywhere. With
/// the stator OFF a binding cell reads `T_c − 1/φ_lim` (to 6e-8). With it ON the incidence minimum
/// is NOT held: it slides to the end of the held stretch, where the stator setting is smallest,
/// and at `r` = 1 on the tilted map it sits off it altogether — `φ` there 1.2e-3 above the floor
/// at BOTH step sizes, so a property of the march, not of the grid.
#[test]
fn c2_the_phi_floor_holds_phi_but_not_the_incidence_minimum_with_the_stator_on() {
    for (shape, m, r) in grid() {
        for sm in SMS {
            let c = cube_on(&m, r, DS, Limiter::PhiFloor { sm });
            let phi_lim = c.set_point.unwrap();
            for (k, (s, b)) in PAIRS.into_iter().enumerate() {
                let (free, leg) = (c.at(s, b, 0), c.at(s, b, 1));
                if leg.read.npts == 0 {
                    continue; // the run does not exist -- gated by the starvation test
                }
                let binds = free.read.min_phi < phi_lim;
                assert_eq!(c.reached[k], binds, "{shape} r={r} sm={sm} S{s}B{b}");
                if !binds {
                    assert_eq!(leg.read.fuel_removed, 0.0, "dormant over the whole run: {shape} r={r} sm={sm} S{s}B{b}");
                }
                if binds && s == 0 {
                    assert!((leg.phi_at_min - phi_lim).abs() < 1e-6
                                && (leg.read.m_i - (c.t_c - 1.0 / phi_lim)).abs() < 1e-6,
                            "{shape} r={r} sm={sm} B{b}");
                }
            }
        }
    }
    let tilted = bt_on(tilt_map(), tilt_map());
    for ds in [DS, DS / 2.0] {
        let c = cube_on(&tilted, 1.0, ds, Limiter::PhiFloor { sm: 0.30 });
        let leg = c.at(1, 0, 1);
        assert!(leg.phi_at_min - c.set_point.unwrap() > 1e-3, "ds={ds}: {}", leg.phi_at_min);
    }
}

/// C3 — HIT, 64 of 64 comparisons (equal-length cells that both bind, withheld fuel as a fraction
/// of each cell's own scheduled fuel): each lever lets the incidence floor withhold LESS; the
/// stator makes the `φ` floor withhold MORE.
#[test]
fn c3_withheld_fuel_answers_to_the_floors_coordinate() {
    let mut n = 0;
    for (shape, m, r) in grid() {
        for ds in [DS, DS / 2.0] {
            for sm in SMS {
                for (inc, lim) in [(true, Limiter::IncidenceFloor { sm }), (false, Limiter::PhiFloor { sm })] {
                    let c = cube_on(&m, r, ds, lim);
                    let pairs: &[((usize, usize), (usize, usize))] = if inc {
                        &[((0, 0), (0, 1)), ((1, 0), (1, 1)), ((0, 0), (1, 0)), ((0, 1), (1, 1))]
                    } else {
                        &[((0, 0), (1, 0)), ((0, 1), (1, 1))]
                    };
                    for &(off, on) in pairs {
                        let (a, b) = (c.at(off.0, off.1, 1), c.at(on.0, on.1, 1));
                        if a.read.npts != b.read.npts || !(a.read.fuel_removed > 0.0 && b.read.fuel_removed > 0.0) {
                            continue;
                        }
                        let (fa, fb) = (c.withheld_fraction(off.0, off.1), c.withheld_fraction(on.0, on.1));
                        assert!(if inc { fb < fa } else { fb > fa }, "{shape} r={r} ds={ds} sm={sm} {off:?}->{on:?}");
                        n += 1;
                    }
                }
            }
        }
    }
    assert_eq!(n, 64, "the comparison count, as measured");
}

/// THE KNOB FINDING the floors add: with the stator on, a `φ` floor can STARVE the engine — it
/// is violated from the start (the stator lowers `φ`), cuts fuel until no fuel holds it, and the
/// run ends early; at the top set point it does not start. The bleed lifts `φ` and rescues it.
/// A run that does not exist is a result: `T` is NaN, nothing panics.
#[test]
fn a_phi_floor_beside_the_stator_starves_the_engine_and_the_bleed_rescues_it() {
    let m = bt();
    let c = cube_on(&m, 0.5, DS, Limiter::PhiFloor { sm: 0.38 });
    let full = c.at(0, 0, 0).read.npts;
    assert!(c.at(1, 0, 1).read.npts < full / 3, "{}", c.at(1, 0, 1).read.npts);
    assert_eq!(c.at(1, 1, 1).read.npts, full);
    assert_eq!(c.at(0, 0, 1).read.npts, full);
    let top = cube_on(&m, 0.5, DS, Limiter::PhiFloor { sm: 0.50 });
    assert_eq!(top.at(1, 0, 1).read.npts, 0);
    assert!(top.t.is_nan() && !top.full_length());
}
