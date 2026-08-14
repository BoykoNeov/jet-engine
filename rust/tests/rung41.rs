//! RUNG 41 — THE TWO-SPOOL SURGE LINE: the exposure SPLITS onto the LP spool.
//!
//! Port of `tests/test_rung41.py`. **The Python file has TWELVE `def test_` functions under
//! EIGHT documented gates**, and § 5.8's step-4 line called it "9 gates" — neither number is the
//! one to port against. So the enumeration is written out below and re-stated as data in
//! [`slice_l_deferrals`], where it is auditable rather than asserted. (`docs`' *slice K* entry:
//! the phase table's scope list had never been enumerated, and it dropped one rung and
//! double-counted another. *An oracle cannot see a MISSING GATE*: grep the source's gates and
//! diff — never port from a header.)
//!
//! | # | `tests/test_rung41.py` | here |
//! |---|------------------------|------|
//! | 1 | `test_reduce_surge_line_is_pure_diagnostic_bit_for_bit` | [`gate1_surge_line_is_a_pure_diagnostic`] |
//! | 2 | `test_reduce_transient_untouched_by_surge_line_bit_for_bit` | **DEFER → phase 6** (`TwoSpoolTransient`); its closing `is_flat` line **WITHDRAWN → slice M** |
//! | 3 | `test_cycle_untouched_rung6_bit_for_bit` | **SPLIT** — [`gate1c_cycle_untouched_rung6`] ports the bit-for-bit halves; the interleaved `SpoolTransient` construction defers |
//! | 4 | `test_pi_c_spool_reproduces_shipped_pi_both_spools` | [`gate2_pi_c_spool_reproduces_the_shipped_pi`] |
//! | 5 | `test_split_lp_takes_the_excursion` | [`gate3_the_split_lp_takes_the_excursion`] |
//! | 6 | `test_shielding_hp_sensitivity_needs_no_lp_pressure_ratio` | [`gate4_shielding_hp_sensitivity_reads_no_lp_ratio`] |
//! | 7 | `test_flight_condition_enters_only_through_Tt2` | [`gate4b_flight_enters_only_through_tt2`] |
//! | 8 | `test_closed_form_flow_turn_depends_on_gamma_c_alone` | [`gate5_closed_form_depends_on_gamma_c_alone`] |
//! | 9 | `test_closed_form_residual_is_the_fuel_fraction_kill_test` | [`gate5_kill_test_the_residual_is_the_fuel_fraction`] |
//! |10 | `test_margin_ordering_lp_is_the_exposed_spool` | [`gate6_margin_ordering_lp_is_the_exposed_spool`] |
//! |11 | `test_flow_turn_does_not_propagate_into_the_margin` | [`gate7_the_turn_does_not_propagate_into_the_margin`] |
//! |12 | `test_rung36_verdict_survives_but_its_mechanism_is_corrected` | **DEFER → phase 6** (`SpoolTransient`, **single**-spool rungs 34/36) |
//!
//! **Three claims here are NOT comparisons, and could not be.** A Rust-vs-Python oracle is blind
//! to an assumption both sides share (`docs`' *measure before registering*), so each of these
//! carries an absolute bar of its own — the `rung39.rs::the_efficiency_loops_test_before_they_step`
//! precedent:
//!
//! * [`p5_the_refinement_count_is_33_and_predictable`] — the count is *predictable from the
//!   arithmetic*, which is why it is gated against the arithmetic and kept OUT of the value dump.
//! * [`p3_tt4_lo_is_dead_the_envelope_ends_the_band`] — the coarse scan's low end.
//! * [`p4_round6_is_pythons_round_not_the_naive_spelling`] — on the tie inputs that separate them.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, counters, round6, Spool, TurnKind,
                          TwoSpoolLosses, TwoSpoolMapMatcher, TwoSpoolMapResult};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const THROTTLE: [f64; 5] = [1500.0, 1300.0, 1100.0, 900.0, 800.0];

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

/// Self-consistent CPG dual gas (rungs 31/38/39/40/41's recipe): `R = (g-1)/g*cp` EXACTLY.
fn cpg_gas_with(gamma_c: f64, cp_c: f64, gamma_t: f64, cp_t: f64, hpr: f64) -> Gas {
    Gas::new(GasSpec {
        gamma_c, cp_c, r_c: (gamma_c - 1.0) / gamma_c * cp_c,
        gamma_t, cp_t, r_t: (gamma_t - 1.0) / gamma_t * cp_t,
        hpr, ..GasSpec::default()
    })
}

fn cpg_gas() -> Gas {
    cpg_gas_with(1.4, 1004.0, 1.3, 1239.0, 42.8e6)
}

/// Python's `_fast_gas()` — the thermally-perfect gas the shape sweeps run on.
fn fast_gas() -> Gas {
    Gas::thermally_perfect()
}

fn mm_with(gas: Gas, map_lp: ComponentMap, map_hp: ComponentMap, pi_lpc: f64, pi_hpc: f64,
           losses: TwoSpoolLosses) -> TwoSpoolMapMatcher {
    let d = build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, 50_000.0, losses);
    TwoSpoolMapMatcher::new(d, flight(), 1.0, map_lp, map_hp)
}

fn mm(gas: Gas, map_lp: ComponentMap, map_hp: ComponentMap) -> TwoSpoolMapMatcher {
    mm_with(gas, map_lp, map_hp, PI_LPC, PI_HPC, real())
}

fn flat_mm(gas: Gas) -> TwoSpoolMapMatcher {
    mm(gas, ComponentMap::flat(), ComponentMap::flat())
}

fn floor(cmap: ComponentMap, phi_surge: f64) -> ComponentMap {
    ComponentMap { phi_surge, ..cmap }
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

/// Rung 41's disclosed shape pairs (`a_t = 0` — compressor islands only).
fn shapes() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    vec![
        ("flow/press", lp_shaped(), hp_shaped()),
        ("press/flow", ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0,
                                      ..ComponentMap::flat() },
                       ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7,
                                      ..ComponentMap::flat() }),
        ("tilted", tilted(), tilted()),
        ("steep", steep(), steep()),
    ]
}

/// MATCHED pairs for the margin ordering — the SAME island/loading shape on both spools. NOT a
/// fully controlled comparison: the two compressors still carry different DESIGN pressure ratios
/// (3 vs 6), which alone makes `SM_L < SM_H` at the design point. The gated content is therefore
/// the RATIO's collapse, not the level.
fn matched_shapes() -> Vec<(&'static str, ComponentMap)> {
    vec![("tilted", tilted()), ("steep", steep()), ("flow", lp_shaped())]
}

// ============================================================================== gate 1
/// **GATE 1 — REDUCE.** `phi_surge` is read by the rung-41 surge methods ALONE, so a map carrying
/// a surge floor must leave rung 39's matched point bit-for-bit identical.
///
/// **This is where P8's content lives, and it is a VALUE test on purpose.** § 5.8.1's deferral
/// table said Python's closing `ComponentMap.flat().with_phi_surge(0.6).is_flat()` line ports
/// here; step 2 withdrew that (§ 5.8.2 (b)) for two reasons. A Rust `is_flat` would be Python's
/// predicate MINUS the `vsv` conjunct — rung 53's field, slice M's — which is the `l` mistake of
/// slice J → K repeated on a predicate. And there is no flat-reduce BRANCH for it to guard: here
/// the reduce is STRUCTURAL (`psi` returns `1.0`, `eta_c_at` returns its base), so `is_flat()`
/// could return `true` while the reduce is broken, and the other way about. The predicate and the
/// property are not the same object in this port, so the property is what is gated.
#[test]
fn gate1_surge_line_is_a_pure_diagnostic() {
    for (name, ml, mh) in shapes() {
        let bare = mm(fast_gas(), ml, mh);
        let armed = mm(fast_gas(), floor(ml, 0.55), floor(mh, 0.55));
        for tt4 in [1500.0, 1100.0, 850.0] {
            let a = bare.match_point(&flight(), tt4).two();
            let b = armed.match_point(&flight(), tt4).two();
            assert_eq!(a.base.pi_lpc.to_bits(), b.base.pi_lpc.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.base.pi_hpc.to_bits(), b.base.pi_hpc.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.eta_lpc.to_bits(), b.eta_lpc.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.eta_hpc.to_bits(), b.eta_hpc.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.n_lp.to_bits(), b.n_lp.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.n_hp.to_bits(), b.n_hp.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.slip.to_bits(), b.slip.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.base.mdot_air.to_bits(), b.base.mdot_air.to_bits(), "{name} @ {tt4}");
            assert_eq!(a.base.thrust.to_bits(), b.base.thrust.to_bits(), "{name} @ {tt4}");
        }
    }
}

// ============================================================================== gate 1c
/// **GATE 1c (the ported half of a SPLIT gate).** The default single-spool design run is
/// untouched by rung 41 — building AND exercising the rung-41 diagnostics must not perturb it.
///
/// Python's version interleaves a `SpoolTransient.surge_margin_channels` call; that half waits
/// for phase 6, and the rung-41 diagnostic exercised here is a two-spool one instead. The gate's
/// content — *the rungs-7+ invariant survives a diagnostic being run* — is intact; what is not
/// yet witnessed is the SINGLE-spool channel split doing the perturbing.
#[test]
fn gate1c_cycle_untouched_rung6() {
    let plain = || build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, ..Losses::default()
    });
    let before = plain().run(&flight(), 1.0);

    let armed = mm(fast_gas(), floor(lp_shaped(), 0.55), floor(hp_shaped(), 0.55));
    let _ = armed.core().surge_margin(&flight(), 1200.0);
    let _ = armed.core().running_line_map(&flight(), &THROTTLE);

    let after = plain().run(&flight(), 1.0);
    assert_eq!(before.performance.specific_thrust.to_bits(),
               after.performance.specific_thrust.to_bits());
    assert_eq!(before.station("4").far.to_bits(), after.station("4").far.to_bits());
    for (label, s) in &before.stations {
        let t = after.station(label);
        assert_eq!(s.tt.to_bits(), t.tt.to_bits(), "station {label} Tt");
        assert_eq!(s.pt.to_bits(), t.pt.to_bits(), "station {label} pt");
    }
}

// ============================================================================== gate 2
/// **GATE 2 — the pi REPRODUCTION, non-tautological.** `pi_c_spool` at the OPERATING `(n, phi)`
/// equals the shipped `pi` on BOTH spools: two code paths, one `pi`, per spool. A margin computed
/// off some other map would be measuring a different machine from the one that set the running
/// line.
#[test]
fn gate2_pi_c_spool_reproduces_the_shipped_pi() {
    for (name, ml, mh) in shapes() {
        let m = mm(fast_gas(), floor(ml, 0.55), floor(mh, 0.55));
        for tt4 in THROTTLE {
            let od = m.match_point(&flight(), tt4).two();
            let lp = m.core().pi_c_spool_shipped(&od, Spool::Lp);
            let hp = m.core().pi_c_spool_shipped(&od, Spool::Hp);
            assert!((lp / od.base.pi_lpc - 1.0).abs() < 1e-9, "{name} @ {tt4}: lp");
            assert!((hp / od.base.pi_hpc - 1.0).abs() < 1e-9, "{name} @ {tt4}: hp");
        }
    }
}

// ============================================================================== gate 3
/// **GATE 3 — THE SPLIT.** `phi_L` falls far more than `phi_H` over the same throttle, and
/// `phi_L < phi_H` at every part-power point. Sign/ordering only — the magnitudes ride on the
/// maps.
#[test]
fn gate3_the_split_lp_takes_the_excursion() {
    for (name, ml, mh) in shapes() {
        let m = mm(fast_gas(), ml, mh);
        let rows = m.core().running_line_map(&flight(), &THROTTLE);
        assert_eq!(rows.len(), THROTTLE.len(), "{name}");
        let (d, lo) = (rows[0], rows[rows.len() - 1]);
        assert!((d.phi_lp - 1.0).abs() < 1e-9 && (d.phi_hp - 1.0).abs() < 1e-9, "{name}");
        let drop_l = 1.0 - lo.phi_lp;
        let drop_h = 1.0 - lo.phi_hp;
        assert!(drop_l > 3.0 * drop_h && drop_h > 0.0, "{name}: {drop_l} vs {drop_h}");
        for r in &rows[1..] {
            assert!(r.phi_lp < r.phi_hp, "{name} @ {}", r.tt4);
        }
        // the HP's OWN ratio spans a NARROWER range than the LP's — the mechanism.
        assert!((rows[0].x_hp / lo.x_hp) < (rows[0].x_lp / lo.x_lp), "{name}");
    }
}

// ============================================================================== gate 4
/// `(s_H, s_L, matched point)` by central difference on the SHIPPED matched points.
fn log_sensitivities(m: &TwoSpoolMapMatcher, fl: &FlightCondition, tt4: f64)
    -> Option<(f64, f64, TwoSpoolMapResult)> {
    let h = 4.0;
    let a = m.core().try_match_point(fl, tt4 - h).ok()?;
    let b = m.core().try_match_point(fl, tt4 + h).ok()?;
    let mid = m.core().try_match_point(fl, tt4).ok()?;
    let xh = |o: &TwoSpoolMapResult| o.base.tt4 / o.base.station("25").tt;
    let xl = |o: &TwoSpoolMapResult| o.base.tt4 / o.base.station("2").tt;
    let s_h = (b.phi_hp.ln() - a.phi_hp.ln()) / (xh(&b).ln() - xh(&a).ln());
    let s_l = (b.phi_lp.ln() - a.phi_lp.ln()) / (xl(&b).ln() - xl(&a).ln());
    Some((s_h, s_l, mid))
}

/// **GATE 4 — THE SHIELDING, made QUANTITATIVE (the two-spool non-tautological gate).**
///
/// NOT a "the HP collapses across flight conditions and the LP does not" gate — that framing was
/// probed and WITHDRAWN: on the choked branch `x_L` and `x_H` are in BIJECTION, so BOTH running
/// lines collapse and the contrast is vacuous. What is not vacuous is WHICH pressure ratios each
/// face's sensitivity contains:
///
/// ```text
///   s_H = k*(1 - pi_HPC^(-1/k)) - 1                              -- pi_HPC ALONE
///   s_L = k*(1 - pi_LPC^(-1/k)) + k*(1 - pi_HPC^(-1/k))/tau_LPC - 1   -- the PRODUCT
/// ```
///
/// `s_H` reads NO LP quantity (rung 39's (†) cancellation); `s_L` cannot be written without
/// `pi_HPC` (rung 39's (‡)). Dropping the HP term from `s_L` must FAIL — and it fails by an order
/// of magnitude, with the wrong SIGN.
#[test]
fn gate4_shielding_hp_sensitivity_reads_no_lp_ratio() {
    let m16 = FlightCondition::new(250.0, 50_000.0, 1.60);
    let cases: Vec<(&str, Gas, f64, f64, FlightCondition)> = vec![
        ("split 3x6", cpg_gas(), PI_LPC, PI_HPC, flight()),
        ("split 4.5x4", cpg_gas(), 4.5, 4.0, flight()),
        ("M0=1.6", cpg_gas(), PI_LPC, PI_HPC, m16),
        ("gamma_c=1.35", cpg_gas_with(1.35, 1004.0, 1.3, 1239.0, 42.8e6), PI_LPC, PI_HPC,
         flight()),
    ];
    let mut worst_lp_drop: f64 = 0.0;
    for (name, gas, pl, ph, fl) in cases {
        let k = gas.gamma_c() / (gas.gamma_c() - 1.0);
        let m = mm_with(gas, ComponentMap::flat(), ComponentMap::flat(), pl, ph, real());
        for tt4 in [1400.0, 1200.0, 1000.0, 850.0, 750.0] {
            let Some((s_h, s_l, o)) = log_sensitivities(&m, &fl, tt4) else { continue };
            let (pi_h, pi_l) = (o.base.pi_hpc, o.base.pi_lpc);
            let tau_l = o.base.station("25").tt / o.base.station("2").tt;
            let s_h_p = k * (1.0 - pi_h.powf(-1.0 / k)) - 1.0;
            let s_l_p = k * (1.0 - pi_l.powf(-1.0 / k))
                + k * (1.0 - pi_h.powf(-1.0 / k)) / tau_l - 1.0;
            let s_l_no = k * (1.0 - pi_l.powf(-1.0 / k)) - 1.0;       // the HP term DROPPED
            assert!((s_h - s_h_p).abs() < 0.05, "{name} @ {tt4}: {s_h} vs {s_h_p}");
            assert!((s_l - s_l_p).abs() < 0.05, "{name} @ {tt4}: {s_l} vs {s_l_p}");
            assert!((s_l - s_l_no).abs() > 10.0 * (s_l - s_l_p).abs(),
                    "{name} @ {tt4}: dropping pi_HPC from s_L did not break it");
            worst_lp_drop = worst_lp_drop.max((s_l - s_l_no).abs());
        }
    }
    assert!(worst_lp_drop > 0.5, "the HP term is no small correction: {worst_lp_drop}");
}

/// **GATE 4b — the WITHDRAWN framing, recorded as its true (weaker) statement.** The flight
/// condition enters the matched state ONLY through `Tt2`, so `p0` is pure scale and the map point
/// is IDENTICAL (pressure-homogeneous — rung 33 gate 6, on two spools).
#[test]
fn gate4b_flight_enters_only_through_tt2() {
    let m = flat_mm(cpg_gas());
    let a = m.match_point(&flight(), 1100.0).two();
    let b = m.match_point(&FlightCondition::new(250.0, 101_325.0, 0.85), 1100.0).two();
    assert!((a.phi_hp / b.phi_hp - 1.0).abs() < 1e-12);
    assert!((a.phi_lp / b.phi_lp - 1.0).abs() < 1e-12);
    assert!((a.base.pi_hpc / b.base.pi_hpc - 1.0).abs() < 1e-12);
    assert!((a.slip / b.slip - 1.0).abs() < 1e-12);
}

// ============================================================================== gate 5
/// **GATE 5 — THE CLOSED FORM (★).** `1 + eta_c(tau_c-1) = gamma_c`, i.e.
/// `pi* = gamma_c^(gamma_c/(gamma_c-1))`: invariant to `eta_HPC`/`eta_HPT`/`gamma_t`/`cp_t`/the
/// design split/the flight condition, tracking `gamma_c` ALONE. The turn's location in `Tt4`
/// moves by hundreds of kelvin across these cases; its location in pressure ratio does not.
#[test]
fn gate5_closed_form_depends_on_gamma_c_alone() {
    let flat = ComponentMap::flat();
    let mut tt4_stars: Vec<f64> = Vec::new();
    let cases: Vec<(&str, f64, f64, TwoSpoolLosses)> = vec![
        ("base", PI_LPC, PI_HPC, real()),
        ("split 4.5x4", 4.5, 4.0, real()),
        ("split 2.25x8", 2.25, 8.0, real()),
        ("eta_hpc .80", PI_LPC, PI_HPC, TwoSpoolLosses { eta_hpc: 0.80, ..real() }),
        ("eta_hpc .95", PI_LPC, PI_HPC, TwoSpoolLosses { eta_hpc: 0.95, ..real() }),
        ("eta_hpt .85", PI_LPC, PI_HPC, TwoSpoolLosses { eta_hpt: 0.85, ..real() }),
        ("eta_lpc .80", PI_LPC, PI_HPC, TwoSpoolLosses { eta_lpc: 0.80, ..real() }),
    ];
    for (name, pl, ph, lo) in cases {
        let m = mm_with(cpg_gas(), flat, flat, pl, ph, lo);
        let t = m.core().flow_coefficient_turn(&flight(), Spool::Hp);
        assert_eq!(t.kind, TurnKind::Min, "{name}");
        let (sf, gc) = (t.star_form.expect("MIN"), t.gamma_c.expect("MIN"));
        assert!((sf / gc - 1.0).abs() < 0.01, "{name}: star_form {sf}");
        tt4_stars.push(t.tt4_star);
    }
    // the Tt4 location is NOT the invariant — it moves a lot, and that IS the point.
    let (hi, lo) = (tt4_stars.iter().cloned().fold(f64::MIN, f64::max),
                    tt4_stars.iter().cloned().fold(f64::MAX, f64::min));
    assert!(hi / lo > 1.4, "{tt4_stars:?}");

    // gamma_t / cp_t are HOT-section knobs: they cannot enter a COLD-section closed form.
    for gas in [cpg_gas_with(1.4, 1004.0, 1.25, 1239.0, 42.8e6),
                cpg_gas_with(1.4, 1004.0, 1.3, 1300.0, 42.8e6)] {
        let m = mm(gas, flat, flat);
        let t = m.core().flow_coefficient_turn(&flight(), Spool::Hp);
        let (sf, gc) = (t.star_form.expect("MIN"), t.gamma_c.expect("MIN"));
        assert!((sf / gc - 1.0).abs() < 0.01, "star_form {sf}");
    }

    // the closed form must TRACK gamma_c — the only parameter in it.
    for gc in [1.30, 1.35, 1.40, 1.45] {
        let m = mm(cpg_gas_with(gc, 1004.0, 1.3, 1239.0, 42.8e6), flat, flat);
        let t = m.core().flow_coefficient_turn(&flight(), Spool::Hp);
        assert!((m.core().critical_flow_turn_pi() - gc.powf(gc / (gc - 1.0))).abs() < 1e-12);
        assert!((t.star_form.expect("MIN") / gc - 1.0).abs() < 0.01, "gamma_c {gc}");
    }

    // flight condition: the same closed form at a very different Tt4.
    let t_d = flat_mm(cpg_gas()).core().flow_coefficient_turn(&flight(), Spool::Hp);
    let t_m = flat_mm(cpg_gas())
        .core().flow_coefficient_turn(&FlightCondition::new(250.0, 50_000.0, 1.60), Spool::Hp);
    assert_eq!(t_m.kind, TurnKind::Min);
    assert!((t_m.star_form.expect("MIN") / t_m.gamma_c.expect("MIN") - 1.0).abs() < 0.01);
    assert!(t_m.tt4_star / t_d.tt4_star > 1.2, "{} vs {}", t_d.tt4_star, t_m.tt4_star);
}

/// **GATE 5's KILL TEST.** (★) is exact with `f` FROZEN — the burner's `(1+f)` is the ONLY
/// impurity. Raise `hPR` so `f → 0`: the residual must fall MONOTONICALLY toward zero, tracking
/// `f`.
#[test]
fn gate5_kill_test_the_residual_is_the_fuel_fraction() {
    let flat = ComponentMap::flat();
    let (mut prev_err, mut prev_f) = (f64::NAN, f64::NAN);
    for (i, hpr) in [42.8e6, 4.28e8, 4.28e9, 4.28e10].into_iter().enumerate() {
        let m = mm(cpg_gas_with(1.4, 1004.0, 1.3, 1239.0, hpr), flat, flat);
        let t = m.core().flow_coefficient_turn(&flight(), Spool::Hp);
        assert_eq!(t.kind, TurnKind::Min, "hPR {hpr}");
        let err = (t.star_form.expect("MIN") / t.gamma_c.expect("MIN") - 1.0).abs();
        let f = t.far.expect("MIN");
        if i > 0 {
            assert!(err < prev_err && f < prev_f, "hPR {hpr}: {err} vs {prev_err}");
        }
        prev_err = err;
        prev_f = f;
    }
    assert!(prev_err < 1e-4, "f ~ 1e-5 should make the closed form EXACT: {prev_err}");
}

// ============================================================================== gate 6
/// **GATE 6 — THE MARGIN ORDERING.** With the SAME map shape on both spools and a COMMON imposed
/// floor, `SM_L < SM_H` at every point and the LP's RELATIVE share of the margin collapses as the
/// engine throttles.
///
/// Two deliberate choices, both about not over-attributing. **(a)** The gated content is the
/// RATIO's COLLAPSE, not the ordering's level: `SM_L < SM_H` already holds AT DESIGN (where
/// `phi_L = phi_H = 1`, so there is no exposure difference) purely because `pi_LPC = 3 < 6`.
/// Matching the map SHAPE does not match the design split. **(b)** The measure is the RATIO, not
/// the absolute gap: both margins tend to zero at deep throttle, so the gap must eventually
/// shrink too.
#[test]
fn gate6_margin_ordering_lp_is_the_exposed_spool() {
    for (name, shape) in matched_shapes() {
        for phi_s in [0.50, 0.55, 0.60] {
            let m = mm(fast_gas(), floor(shape, phi_s), floor(shape, phi_s));
            let sched = m.core().surge_margin_schedule(&flight(), &THROTTLE);
            assert_eq!(sched.len(), THROTTLE.len(), "{name} / {phi_s}");
            for r in &sched {
                assert!(r.sm_lp < r.sm_hp, "{name} / {phi_s} @ {}", r.tt4);
                assert_eq!(r.binding, Spool::Lp, "{name} / {phi_s} @ {}", r.tt4);
            }
            let ratio: Vec<f64> = sched.iter().map(|r| r.sm_lp / r.sm_hp).collect();
            for i in 0..ratio.len() - 1 {
                assert!(ratio[i] > ratio[i + 1], "{name} / {phi_s}: {ratio:?}");
            }
            assert!(ratio[ratio.len() - 1] < 0.5 * ratio[0], "{name} / {phi_s}: {ratio:?}");
            // both schedules inherit rung 36's sign: thin at low power.
            for (k, v) in [("SM_lp", sched.iter().map(|r| r.sm_lp).collect::<Vec<_>>()),
                           ("SM_hp", sched.iter().map(|r| r.sm_hp).collect::<Vec<_>>())] {
                for i in 0..v.len() - 1 {
                    assert!(v[i] > v[i + 1], "{name} / {phi_s} / {k}: {v:?}");
                }
            }
        }
    }
}

// ============================================================================== gate 7
/// **GATE 7 — THE DIVERGENCE.** The withdrawn claim, asserted as a DELIBERATE divergence: `phi_H`
/// turns UP past `pi*` while `SM_H` keeps FALLING. Flow-coefficient proximity and pressure-ratio
/// margin are different schedules — (★) is an incidence fact, NOT a margin extremum.
#[test]
fn gate7_the_turn_does_not_propagate_into_the_margin() {
    let grid = [1500.0, 1300.0, 1100.0, 950.0, 850.0, 800.0, 750.0, 700.0];
    for (name, shape) in matched_shapes() {
        let m = mm(fast_gas(), floor(shape, 0.50), floor(shape, 0.50));
        let sched = m.core().surge_margin_schedule(&flight(), &grid);
        let phis: Vec<f64> = sched.iter().map(|r| r.phi_hp).collect();
        let sms: Vec<f64> = sched.iter().map(|r| r.sm_hp).collect();
        assert!((0..phis.len() - 1).any(|i| phis[i] < phis[i + 1]),
                "{name}: phi_H never turned up: {phis:?}");
        for i in 0..sms.len() - 1 {
            assert!(sms[i] > sms[i + 1], "{name}: SM_H not monotone: {sms:?}");
        }
    }
}

// ==================================================================== the ABSOLUTE bars
/// **P5 — THE REFINEMENT COUNT IS EXACTLY 33 ON EVERY `MIN` RUN, AND THE GATE NAMES ITS
/// INSTRUMENT.**
///
/// **Why this is not in the value dump.** Python cannot instrument the shipped body's two phases
/// apart from outside — its arm would be a transcription of the same loop, so an oracle
/// comparison would be self-confirming (rung 83's *identity round-trip sold as verification*).
/// The load-bearing leg is the ARITHMETIC, and it is interpreter-independent: the bracket is
/// `2 * coarse = 20` wide, the stop is `b - a < 1e-5`, so the golden section takes
/// `ceil(ln(1e-5/20)/ln(0.618…)) = 31` passes that call `phi`, plus the 2 initial.
///
/// **The instrument is named because the memo makes three counts differ**: refinement `phi` calls
/// = 33, total `match` calls = 116–140 (the coarse scan's length rides the envelope), and loop
/// passes = 32, of which 31 call `phi`. A gate comparing 33 against a loop-pass counter would be
/// measuring the wrong number and would still pass most of the time (§ 5.7 (d) corrected exactly
/// that bar one slice ago).
///
/// The second half is the vacuity guard: a `RAIL` run never enters the refinement at all, so the
/// same counter must read ZERO there. Without it, a counter wedged at 33 would satisfy the first
/// half for the wrong reason.
///
/// # WHAT THIS GATE CATCHES — measured by injecting the defect, not assumed
///
/// A first draft of this doc claimed the count witnesses the golden section's CHECK-FIRST shape,
/// copying a claim the shipped source carried ("a `do`-while makes the refinement count 34
/// instead of 33"). **Both were wrong, and the gate passed the defect.** Rewritten `do`-while the
/// loop makes the same 33 calls, lands on the same `tt4_star`, and moves no bit of any oracle —
/// because the bracket is ALWAYS 20 wide on entry, so the stopping rule cannot be met before the
/// first pass, which is the only thing separating the two shapes. (Contrast `rung39.rs`'s
/// efficiency-loop gate, where a flat map DOES meet the residual on entry.) Both claims are now
/// corrected in `two_spool.rs`.
///
/// What it does catch, confirmed the same way: a changed stopping rule — `1e-6` makes it 37.
/// So this is a gate on the SCAN PARAMETERS reaching the loop, not on the loop's shape.
#[test]
fn p5_the_refinement_count_is_33_and_predictable() {
    let gr = (5.0f64.sqrt() - 1.0) / 2.0;
    let predicted = 2 + ((1e-5f64 / 20.0).ln() / gr.ln()).ceil() as u64;
    assert_eq!(predicted, 33, "the arithmetic itself moved");

    counters::reset();
    let t = flat_mm(cpg_gas()).core().flow_coefficient_turn(&flight(), Spool::Hp);
    assert_eq!(t.kind, TurnKind::Min);
    assert_eq!(counters::refine_calls(), predicted,
               "the golden section made {} refinement phi calls, not the {predicted} the \
                bracket width and the stopping rule predict", counters::refine_calls());

    // ...and the counter is not simply wedged: a RAIL run never refines.
    counters::reset();
    let r = flat_mm(cpg_gas()).core().flow_coefficient_turn(&flight(), Spool::Lp);
    assert_eq!(r.kind, TurnKind::Rail, "the LP spool on flat maps is the RAIL branch");
    assert_eq!(counters::refine_calls(), 0,
               "a RAIL run never enters the golden section, so a nonzero count here means the \
                counter is not measuring the refinement at all");
}

/// **P3 — `Tt4_lo` IS DEAD: the CHOKED ENVELOPE ends the coarse scan, not the parameter.**
///
/// A comparison cannot see this — both sides would agree while both read the parameter. The scan
/// is `while T > Tt4_lo`, stepping by `coarse` and breaking on an abort, so if the PARAMETER ended
/// it the last appended throttle would land in `(Tt4_lo, Tt4_lo + coarse]`. Measured over 118
/// runs the scan always terminated on the abort, so `band_lo` sits far above that window — which
/// is what makes "ported as written, recorded as dead" a gate rather than a note.
///
/// It matters because a reader who believes `Tt4_lo = 350` sets the band's low end would tune it
/// expecting the band to move.
#[test]
fn p3_tt4_lo_is_dead_the_envelope_ends_the_band() {
    let (tt4_lo, coarse) = (350.0, 10.0);
    let mut n = 0;
    for (name, ml, mh) in shapes() {
        for phi_s in [0.0, 0.55] {
            for spool in [Spool::Hp, Spool::Lp] {
                let m = mm(cpg_gas(), floor(ml, phi_s), floor(mh, phi_s));
                let t = m.core().flow_coefficient_turn(&flight(), spool);
                assert!(t.band.0 > tt4_lo + coarse,
                        "{name} / {phi_s} / {spool:?}: the coarse scan ended at {} — inside one \
                         step of Tt4_lo = {tt4_lo}, which means the PARAMETER ended the band and \
                         not the choked envelope", t.band.0);
                n += 1;
            }
        }
    }
    assert_eq!(n, 16, "the sweep this bar was measured on is 4 shapes x 2 floors x 2 spools");
}

/// **P4 — `round6` IS PYTHON'S `round`, ON THE INPUTS THAT SEPARATE THE TWO SPELLINGS.**
///
/// **This is a throttle, not a cache key**: `cache[key] = self.match(flight, key)` passes the
/// ROUNDED value on, so a divergent rounding makes the port solve a silently different engine.
///
/// The pre-registration measured 0 disagreements over 4 216 live keys and a 600 000-point
/// synthetic sweep, and said the zero was NOT a proof (≈0.1 expected events at the estimated
/// rate). It can be settled by construction instead. An exact tie at the 6th decimal needs
/// `x = (2j+1)/(2·10^6)`; for a dyadic `x = m/2^k` that forces `k = 7` with `m` odd — so the ties
/// are exactly the ODD MULTIPLES OF 1/128, which are representable and inside the [350, 1500]
/// band the scan sweeps. Python rounds half to EVEN; `(x*1e6).round()/1e6` rounds half AWAY from
/// zero and carries the multiply's own error.
#[test]
fn p4_round6_is_pythons_round_not_the_naive_spelling() {
    let naive = |x: f64| (x * 1e6).round() / 1e6;
    // The witness: an exact tie, reachable inside the scanned band.
    let x = 350.0 + 1.0 / 128.0;
    assert_eq!(x, 350.0078125, "the tie input must be exactly representable");
    assert_eq!(round6(x).to_bits(), 350.007812f64.to_bits(),
               "Python's round is half-to-EVEN: round(350.0078125, 6) = 350.007812");
    assert_ne!(round6(x).to_bits(), naive(x).to_bits(),
               "the naive spelling must DIFFER here — if it agrees, this test has stopped \
                witnessing the divergence class it was written for");

    // ...and the two agree everywhere the ties are not, which is why the defect hid.
    let mut differs = 0;
    for i in 0..20_000 {
        let t = 350.0 + 0.0575 * i as f64;
        if round6(t).to_bits() != naive(t).to_bits() {
            differs += 1;
        }
    }
    assert_eq!(differs, 0,
               "off the tie set the two spellings agree — {differs} disagreements means the \
                port's round6 has a SECOND defect, not the half-to-even one");
}

/// **THE IOU.** What `tests/test_rung41.py` carries that this file does not, and why — the
/// `rung33.rs::slice_j_deferrals` precedent, reused for the third time.
///
/// The count is stated as data rather than prose because § 5.8's step-4 line said "9 gates" for a
/// file with **12** test functions under **8** documented gate headings, and a number nobody can
/// re-derive is how a port drops one silently.
#[test]
fn slice_l_deferrals() {
    // (name, ported?) — every `def test_` in tests/test_rung41.py, in file order.
    let roster: [(&str, bool); 12] = [
        ("test_reduce_surge_line_is_pure_diagnostic_bit_for_bit", true),
        // DEFER -> phase 6: reaches TwoSpoolTransient (rung 40). Its closing
        // `ComponentMap.flat().with_phi_surge(0.6).is_flat()` line was listed by § 5.8.1 as
        // porting NOW and is WITHDRAWN (§ 5.8.2 (b)): a Rust `is_flat` would be Python's
        // predicate minus the `vsv` conjunct (rung 53's, slice M's), and there is no
        // flat-reduce BRANCH here for it to guard. P8's content is gated as a VALUE, in
        // `gate1_surge_line_is_a_pure_diagnostic`.
        ("test_reduce_transient_untouched_by_surge_line_bit_for_bit", false),
        // SPLIT: the bit-for-bit cycle halves are `gate1c_cycle_untouched_rung6`; the
        // interleaved `SpoolTransient.surge_margin_channels` construction defers to phase 6.
        ("test_cycle_untouched_rung6_bit_for_bit", true),
        ("test_pi_c_spool_reproduces_shipped_pi_both_spools", true),
        ("test_split_lp_takes_the_excursion", true),
        ("test_shielding_hp_sensitivity_needs_no_lp_pressure_ratio", true),
        ("test_flight_condition_enters_only_through_Tt2", true),
        ("test_closed_form_flow_turn_depends_on_gamma_c_alone", true),
        ("test_closed_form_residual_is_the_fuel_fraction_kill_test", true),
        ("test_margin_ordering_lp_is_the_exposed_spool", true),
        ("test_flow_turn_does_not_propagate_into_the_margin", true),
        // DEFER -> phase 6: `SpoolTransient.surge_margin_channels` — SINGLE-spool, rungs 34/36.
        // § 5.8's own list called this `TwoSpoolTransient`; phase 6 covers both, so the verdict
        // held while the noun was wrong.
        ("test_rung36_verdict_survives_but_its_mechanism_is_corrected", false),
    ];
    let ported = roster.iter().filter(|(_, p)| *p).count();
    assert_eq!(roster.len(), 12,
               "tests/test_rung41.py has 12 test functions — if that changed, this roster is \
                stale and the port is gating against a file that no longer exists");
    assert_eq!(ported, 10, "10 of rung 41's 12 test functions port in slice L; 2 defer to \
                            phase 6, and one of the 10 ports only its non-transient half");
    for (name, p) in roster {
        if !p {
            println!("DEFERRED -> phase 6: {name}");
        }
    }
}
