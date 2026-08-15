//! RUNG 54 — THE STATOR-ROW THROAT: a constraint's SEVERITY is coordinate-dependent too.
//!
//! Port of `tests/test_rung54.py`, gate for gate. Its ten gate groups:
//!
//!   1. REDUCE — an INVARIANCE OVER `C`, not merely an identity at `C = 0`: the throat enters
//!      NO solver, so the capacity constant cannot move ANY matched number at ANY setting, on
//!      both gases. Plus the flatness rule read for the throat, and `C = 0` ≡ no model.
//!   2. THE DERIVED AREA LAW — `A_th(v)/A_th(0) = cos(atan v)`, exactly EVEN, unity at design.
//!   3. THE ONE CONSTANT, DISCLOSED — bounded, readable as a design throat Mach, and ESCAPED by
//!      `c_min = 1/X`, which needs no constant at all.
//!   4. P1: BIND, NEVER RELIEVE — a choked row leaves every matched number untouched.
//!   5. THE HEADLINE — the throat cuts the SETTING far more than the MARGIN; and gate 5b, an
//!      EXACT ZERO: the measured asymmetry is the efficiency island and nothing else.
//!   6. THE ARTIFACT IS NEVER THE CEILING — `binds != Edge` on every shape.
//!   7. P-C2 — the incidence peak IS interior on some shapes, correcting rung 53's concession;
//!      and where it falls short of design incidence, rung 53's schedule ceases to EXIST.
//!   8. P-C3 — rung 54's bracketed root is immune to the doubling ladder rung 53 relies on,
//!      and agrees with it wherever that ladder succeeds.
//!   9. THE RACE — `X(v*)` crosses the design loading inside the envelope; the ceiling is a
//!      pure-LP object.
//!  10. CYCLE UNTOUCHED.
//!
//! Python marks seven of these `@pytest.mark.slow`; the markers are not carried over, for the
//! reason `rung53.rs`'s header gives — the marker records a COST that did not survive the port.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator::{Binds, VariableStatorCore};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const THROTTLE: [f64; 6] = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0];

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
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn shape(name: &str) -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    match name {
        "flow/press" => (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                         ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
        "press/flow" => (ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f },
                         ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        "tilted"     => (ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f },
                         ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        "steep"      => (ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f },
                         ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }),
        "flat-eta"   => (ComponentMap { sigma: 0.1, l: 0.7, ..f },
                         ComponentMap { sigma: 0.1, l: 1.0, ..f }),
        _ => panic!("unknown shape {name}"),
    }
}

const ALL_SHAPES: [&str; 5] = ["flow/press", "press/flow", "tilted", "steep", "flat-eta"];

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

fn vm(d: TwoSpoolEngine, ml: ComponentMap, mh: ComponentMap, vl: f64, vh: f64)
    -> VariableStatorCore
{
    VariableStatorCore::new(d, flight(), 1.0, ml, mh, vl, vh)
}

/// Python's `_shaped`.
fn shaped(d: TwoSpoolEngine, name: &str, c: f64, vl: f64, vh: f64) -> VariableStatorCore {
    let (ml, mh) = shape(name);
    let (mut ml, mut mh) = (ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR));
    if c > 0.0 {
        ml = ml.with_capacity(c);
        mh = mh.with_capacity(c);
    }
    vm(d, ml, mh, vl, vh)
}

/// The 19 matched fields the invariance is asserted over, as raw bits.
fn fields(m: &VariableStatorCore, tt4: f64) -> Vec<(&'static str, u64)> {
    let o = m.core.match_point(&flight(), tt4);
    vec![
        ("pi_lpc", o.base.pi_lpc.to_bits()), ("pi_hpc", o.base.pi_hpc.to_bits()),
        ("n_lp", o.n_lp.to_bits()), ("n_hp", o.n_hp.to_bits()),
        ("phi_lp", o.phi_lp.to_bits()), ("phi_hp", o.phi_hp.to_bits()),
        ("slip", o.slip.to_bits()),
        ("eta_lpc", o.eta_lpc.to_bits()), ("eta_hpc", o.eta_hpc.to_bits()),
        ("eta_hpt", o.eta_hpt.to_bits()), ("eta_lpt", o.eta_lpt.to_bits()),
        ("tau_lpc", o.base.tau_lpc.to_bits()), ("tau_hpc", o.base.tau_hpc.to_bits()),
        ("tau_hpt", o.base.tau_hpt.to_bits()), ("tau_lpt", o.base.tau_lpt.to_bits()),
        ("mdot_air", o.base.mdot_air.to_bits()), ("thrust", o.base.thrust.to_bits()),
        ("N_lp_ratio", o.n_lp_ratio.to_bits()), ("N_hp_ratio", o.n_hp_ratio.to_bits()),
    ]
}

// ==========================================================================================
// GATE 1 — REDUCE: an INVARIANCE OVER C, not merely an identity at C = 0
// ==========================================================================================

/// THE RUNG'S STRONGEST CLAIM (P1). `v` enters the solve through `solve_n` alone (rung 53) and
/// the throat enters NO solver, so `X` is a post-hoc functional of the SOLVED state. Then the
/// capacity constant cannot move ANY matched number — for every `C`, not just `C = 0`, and at a
/// MOVED stator. Rung 53 earned an identity at one setting; rung 54 earns invariance over a
/// whole parameter.
#[test]
fn test_reduce_every_matched_field_is_bit_identical_for_every_capacity() {
    for (vl, vh) in [(0.0, 0.0), (0.30, 0.0), (0.0, 0.15), (0.20, 0.10)] {
        let d = design(cpg_gas());
        let base = vm(d.clone(), lp_map(), hp_map(), vl, vh);
        let refs: Vec<Vec<(&str, u64)>> =
            THROTTLE.iter().map(|&t| fields(&base, t)).collect();
        for c in [0.05, 0.30, 0.55, 0.80, 0.95, 0.999] {
            let m = vm(d.clone(), lp_map().with_capacity(c), hp_map().with_capacity(c), vl, vh);
            for (i, &t) in THROTTLE.iter().enumerate() {
                for (got, want) in fields(&m, t).iter().zip(refs[i].iter()) {
                    assert_eq!(got.1, want.1,
                               "capacity C={c} moved {} at Tt4={t} (vsv={vl},{vh}) — \
                                the throat entered a solver", got.0);
                }
            }
        }
    }
}

/// The invariance is structural, so it must not be a CPG accident.
#[test]
fn test_reduce_holds_on_the_reacting_gas_too() {
    let d = design(Gas::reacting_equilibrium());
    let base = vm(d.clone(), lp_map(), hp_map(), 0.25, 0.0);
    for c in [0.40, 0.85] {
        let m = vm(d.clone(), lp_map().with_capacity(c), hp_map().with_capacity(c), 0.25, 0.0);
        for t in [1500.0, 1200.0] {
            for (got, want) in fields(&m, t).iter().zip(fields(&base, t).iter()) {
                assert_eq!(got.1, want.1, "C={c} moved {} at Tt4={t}", got.0);
            }
        }
    }
}

/// The `phi_surge` rule, read for the throat: a PURE DIAGNOSTIC that never touches
/// `psi`/`eta`/the running line is not part of flatness, so a flat map WITH a throat model still
/// reduces `MapMatcher` to rung 31. Rung 53's `vsv` is the opposite case and stays so.
#[test]
fn test_reduce_capacity_is_not_part_of_flatness_but_vsv_still_is() {
    assert!(ComponentMap::flat().with_capacity(0.8).is_flat());
    assert!(ComponentMap::flat().with_phi_surge(0.6).with_capacity(0.8).is_flat());
    assert!(!ComponentMap::flat().with_vsv(0.1).is_flat());
    assert!(!ComponentMap::flat().with_capacity(0.8).with_vsv(0.1).is_flat());
}

/// `C = 0` is "no throat model" exactly as `phi_surge = 0` is "no surge line".
///
/// **NARROWED:** Python also round-trips `phi_max`, which does not exist in Rust — see
/// `rung53.rs::slice_m_deferrals` item 1, the same debt, not a second one.
#[test]
fn test_reduce_capacity_zero_leaves_rung_53_expressions_bit_for_bit() {
    let m = lp_map().with_vsv(0.3);
    assert_eq!(m.capacity, 0.0);
    assert_eq!(m.with_capacity(0.0), m);
    assert_eq!(m.psi(0.8).to_bits(), m.with_capacity(0.7).psi(0.8).to_bits());
    assert_eq!(m.phi_surge_at().to_bits(), m.with_capacity(0.7).phi_surge_at().to_bits());
    assert_eq!(m.tan_beta1(0.8).to_bits(), m.with_capacity(0.7).tan_beta1(0.8).to_bits());
}

/// No throat model ⇒ the margin is not defined.
/// `expected` is not decoration: a bare `should_panic` catches ANY panic, so it would pass if
/// `lp_map()` or `with_vsv` blew up and the guard under test never ran at all.
#[test]
#[should_panic(expected = "needs a throat model")]
fn test_capacity_margin_undefined_without_a_model() {
    let _ = lp_map().with_vsv(0.3).capacity_margin(1.0);
}

// ==========================================================================================
// GATE 2 — THE DERIVED AREA LAW (shape: zero new constants)
// ==========================================================================================

/// `A_th(v)/A_th(0) = cos(alpha_1)` with `v = tan(alpha_1)`: `o/s = cos(alpha)` is the standard
/// cascade throat relation, so the area law rides on rung 53's OWN coordinate.
#[test]
fn test_throat_ratio_is_the_cascade_cosine_rule() {
    for v in [-1.5, -0.6, -0.2, 0.0, 0.2, 0.6, 1.5, 3.0] {
        let got = ComponentMap { l: 0.7, ..ComponentMap::flat() }.with_vsv(v).throat_ratio();
        assert!((got - v.atan().cos()).abs() <= 1e-15, "v={v}: {got}");
        assert!((got - 1.0 / (1.0 + v * v).sqrt()).abs() <= 1e-15, "v={v}: {got}");
    }
}

/// The GEOMETRIC cost is two-sided: `cos` is even, so the throat is maximal AT the design setting
/// and closes whichever way the vane turns. (That the peak coincides with the design setting is
/// INHERITED from rung 53's coordinate origin, not derived — see the spec's Concessions.) Any
/// measured asymmetry must therefore come from elsewhere: gate 5b.
#[test]
fn test_throat_area_law_is_exactly_even_and_unity_at_design() {
    let m = ComponentMap { l: 0.7, ..ComponentMap::flat() };
    for v in [0.2, 0.6, 1.5, 3.0] {
        assert_eq!(m.with_vsv(v).throat_ratio().to_bits(),
                   m.with_vsv(-v).throat_ratio().to_bits(), "EXACTLY even at v={v}");
        assert!(m.with_vsv(v).throat_ratio() < 1.0, "v={v}");
    }
    assert_eq!(m.throat_ratio(), 1.0);
    assert_eq!(m.throat_loading(0.83), 0.83, "X == m at the design setting");
}

// ==========================================================================================
// GATE 3 — THE ONE CONSTANT, DISCLOSED (and the escape from it)
// ==========================================================================================

/// `C >= 1` would mean the row is past choke at its own design point. And `C` is disclosed in
/// units an engineer can judge: the design throat Mach, by inverting `MFP(M)/MFP(1)`.
#[test]
fn test_capacity_constant_is_bounded_and_reads_as_a_design_throat_mach() {
    for (c, m) in [(0.70, 0.4583), (0.80, 0.5533), (0.90, 0.6782)] {
        let got = lp_map().with_capacity(c).design_throat_mach(1.4);
        assert!((got - m).abs() < 5e-4, "C={c}: {got} vs {m}");
    }
    // strictly increasing, so the inverse is well posed
    let machs: Vec<f64> = [0.5, 0.6, 0.7, 0.8, 0.9].iter()
        .map(|&c| lp_map().with_capacity(c).design_throat_mach(1.4)).collect();
    assert!(machs.windows(2).all(|w| w[0] <= w[1]), "{machs:?}");
}

/// The three refused capacities, one test each — Rust's `#[should_panic]` is per-test, so a
/// loop over the bad values would stop at the first and the other two would go unmeasured.
/// Each names the guard's own message, so "a panic happened" cannot stand in for "the guard
/// fired": `C = 1.0` is the boundary the half-open range must exclude, and it is the one a
/// `<=` slip would let through.
#[test]
#[should_panic(expected = "C in [0,1)")]
fn test_capacity_one_is_refused() { let _ = lp_map().with_capacity(1.0); }

#[test]
#[should_panic(expected = "C in [0,1)")]
fn test_capacity_above_one_is_refused() { let _ = lp_map().with_capacity(1.2); }

#[test]
#[should_panic(expected = "C in [0,1)")]
fn test_capacity_negative_is_refused() { let _ = lp_map().with_capacity(-0.1); }

/// The escape from the disclosed constant: `c_min = 1/X` is a DERIVED threshold, present whether
/// or not a throat model is attached, and the row chokes iff `C >= c_min`.
#[test]
fn test_c_min_is_reported_without_any_constant_and_the_margin_uses_it() {
    let d = design(cpg_gas());
    let bare = vm(d.clone(), lp_map(), hp_map(), 0.4, 0.0)
        .throat_margin(&flight(), 1200.0).lp;
    let bt = bare.throat.expect("throat_margin always carries a throat read");
    assert_eq!(bt.capacity, 0.0);
    assert!(bt.choke.is_none(), "no model ⇒ the three choke keys are ABSENT, not zeroed");
    let x = bt.throat_loading;
    assert!((bt.c_min / (1.0 / x) - 1.0).abs() < 1e-14);
    for c in [0.5, 0.8] {
        let r = vm(d.clone(), lp_map().with_capacity(c), hp_map().with_capacity(c), 0.4, 0.0)
            .throat_margin(&flight(), 1200.0).lp;
        let t = r.throat.expect("throat read");
        assert!((t.throat_loading / x - 1.0).abs() < 1e-14, "X is constant-free");
        let ch = t.choke.expect("a model ⇒ the choke keys are PRESENT");
        assert!((ch.m_c / (1.0 - c * x) - 1.0).abs() < 1e-14);
        assert_eq!(ch.choked, c >= t.c_min);
    }
}

/// `X = m*sqrt(1+v^2)`: the face-referred corrected flow is NOT divided by the throat (annulus
/// continuity keeps `Vx` independent of `alpha_1`), so `phi` is untouched and only the throat
/// referral changes. That is the whole reason the channel is diagnostic-only.
#[test]
fn test_throat_loading_equals_face_flow_times_secant() {
    let d = design(cpg_gas());
    for v in [0.0, 0.35, 0.9] {
        let r = vm(d.clone(), lp_map(), hp_map(), v, 0.0).throat_margin(&flight(), 1200.0).lp;
        let t = r.throat.expect("throat read");
        let want = r.m * (1.0 + v * v).sqrt();
        assert!((t.throat_loading / want - 1.0).abs() < 1e-13, "v={v}");
        assert!((r.m / (r.phi_op * r.n) - 1.0).abs() < 1e-13, "v={v}");
    }
}

// ==========================================================================================
// GATE 4 — P1: BIND, NEVER RELIEVE
// ==========================================================================================

/// The theorem's operational face. Pick a setting the row cannot pass (`C*X > 1`) and check the
/// solve does not notice: the throat REMOVES SETTINGS FROM THE FEASIBLE SET, it does not change
/// the map from setting to incidence. So no area law could buy back rung 53's overspeed — which
/// REFUTES the expectation rung 53's own seam recorded.
#[test]
fn test_a_choked_row_still_leaves_every_matched_number_untouched() {
    let d = design(cpg_gas());
    let c = 0.95;
    let m = vm(d.clone(), lp_map().with_capacity(c), hp_map().with_capacity(c), 1.1, 0.0);
    let r = m.throat_margin(&flight(), 1500.0).lp;
    let ch = r.throat.expect("throat").choke.expect("choke keys");
    assert!(ch.choked && ch.m_c < 0.0, "pick a setting that actually chokes the row");
    let bare = vm(d, lp_map(), hp_map(), 1.1, 0.0);
    for (got, want) in fields(&m, 1500.0).iter().zip(fields(&bare, 1500.0).iter()) {
        assert_eq!(got.1, want.1, "a CHOKED row moved {}", got.0);
    }
}

// ==========================================================================================
// GATE 5 — THE HEADLINE: severity is coordinate-dependent
// ==========================================================================================

/// RUNG 54's HEADLINE, and rung 53's law read one level up. Rung 53: a MARGIN is a distance, so
/// it is coordinate-dependent. Rung 54: so is a CONSTRAINT'S SEVERITY. The throat truncates the
/// stator hard in the lever's own coordinate and nearly not at all in the protected variable,
/// because the coordinate's returns have already flattened.
#[test]
fn test_headline_the_throat_cuts_the_setting_far_more_than_the_margin() {
    let d = design(cpg_gas());
    for name in ALL_SHAPES {
        let m = shaped(d.clone(), name, 0.90, 0.0, 0.0);
        let mut seen = false;
        for t in [1200.0, 1000.0, 800.0] {
            let a = m.authority_ceiling(&flight(), t, Spool::Lp, None);
            if a.v_ch.is_none() {
                continue;
            }
            seen = true;
            assert!(a.setting_cut > 0.10,
                    "{name} @{t}: the throat should bite the SETTING appreciably, got {:.3}",
                    a.setting_cut);
            assert!(a.retained > a.setting_cut, "{name} @{t}: severity not inverted");
            assert!(a.retained >= 0.78,
                    "{name} @{t}: retention {:.3} — the margin cost should stay small even \
                     where the setting cost is large", a.retained);
        }
        assert!(seen, "{name}: the throat never bound anywhere — gate vacuous");
    }
}

/// The spec's quoted case, pinned: at `Tt4` = 1000, `C = 0.90` the setting is cut ~30% and the
/// margin ~4%.
#[test]
fn test_headline_the_default_shape_numbers() {
    let m = shaped(design(cpg_gas()), "flow/press", 0.90, 0.0, 0.0);
    let a = m.authority_ceiling(&flight(), 1000.0, Spool::Lp, None);
    assert!((a.setting_cut - 0.304).abs() < 0.02, "setting_cut {}", a.setting_cut);
    assert!((a.retained - 0.905).abs() < 0.02, "retained {}", a.retained);
    assert!((a.m_i_usable / a.m_i_peak - 0.960).abs() < 0.02);
}

/// GATE 5b, an EXACT ZERO. The geometric cost is exactly even (gate 2), so any asymmetry in the
/// MEASURED cost `X(v)` must enter through `m` — which moves only via the efficiency island. On
/// a FLAT island rung 53's P5 pins `m` exactly, so `X` must be even BIT-FOR-BIT; on a shaped
/// island it must not be (or the zero is vacuous).
#[test]
fn test_the_measured_asymmetry_is_the_efficiency_islands() {
    let d = design(cpg_gas());
    for name in ["flat-eta", "flow/press", "steep"] {
        let grid = [-0.6, -0.4, -0.2, 0.2, 0.4, 0.6];
        let rows = shaped(d.clone(), name, 0.0, 0.0, 0.0)
            .throat_sweep(&flight(), 1500.0, &grid, Spool::Lp);
        let at = |v: f64| -> &turbojet::stator::SpoolMargin {
            rows.iter().find(|r| r.vsv == v).unwrap_or_else(|| panic!("no row at {v}"))
        };
        // **THE INSTRUMENT, BEFORE THE CLAIM.** The flat-eta half asserts an EXACT ZERO, and
        // the two ways to get one for free are both closed here: `find` returning the SAME row
        // for `+a` and `-a` (six distinct settings, so it cannot), and a difference taken over
        // an empty set (`all()` on an empty iterator is `true`). Rung 54's zero IS the finding,
        // so a weak instrument would loosen the claim, not the tolerance.
        assert_eq!(rows.len(), grid.len(), "one row per swept setting");
        for (i, r) in rows.iter().enumerate() {
            assert_eq!(r.vsv.to_bits(), grid[i].to_bits(), "row {i} is not its own setting");
        }
        let load = |v: f64| at(v).throat.expect("throat read").throat_loading;
        let diffs: Vec<f64> = [0.2, 0.4, 0.6].iter().map(|&a| load(a) - load(-a)).collect();
        assert_eq!(diffs.len(), 3, "the difference set must not be empty");
        if name == "flat-eta" {
            assert!(diffs.iter().all(|&x| x == 0.0),
                    "flat island must be EXACTLY even, got {diffs:?}");
            for a in [0.2, 0.4, 0.6] {
                assert_eq!(at(a).m.to_bits(), at(-a).m.to_bits(), "m even at {a}");
            }
        } else {
            assert!(diffs.iter().all(|&x| x > 0.0),
                    "{name}: shaped island should NOT be even, got {diffs:?}");
            let mx = diffs.iter().cloned().fold(f64::MIN, f64::max);
            assert!(mx > 1e-3, "{name}: asymmetry too small to be a real contrast");
        }
    }
}

// ==========================================================================================
// GATE 6 — THE ARTIFACT IS NEVER THE CEILING (P-A2, the shape-robust SIGN claim)
// ==========================================================================================

/// Rung 53 conceded its authority ceiling was `solve_n`'s speed-line bracket — "a map-validity
/// edge", i.e. an ARTIFACT. Once the throat is modelled that artifact is never what stops the
/// stator: `v_ch < v_edge` everywhere at `C >= 0.80`. The LEVEL of `C_edge` is disclaimed (P-A1
/// measured 0.63..0.78 across shapes — it is a threshold on an artifact and has no reason to be
/// robust); the SIGN is the claim.
///
/// THE LOAD-BEARING ASSERTION IS THE NEGATIVE ONE, `binds != Edge`. `throat_before_edge` also
/// holds 20/20 but is weaker and says something different: on `steep` the incidence peak is
/// INSIDE the throat (`v_peak < v_ch < v_edge`), so there the PEAK binds and the throat is
/// merely also present. The spec states it that way.
#[test]
fn test_the_throat_binds_before_solve_n_bracket_on_every_shape() {
    // Pin the scan resolution: the published 20/20 is measured at this step, and its tightest
    // cell (steep @1500, v_ch 0.993 vs v_edge 1.12) is the first that would flip if it changed.
    assert_eq!(VariableStatorCore::V_STEP, 0.04,
               "the 20/20 claim in docs/rung54-spec.md is measured at V_STEP = 0.04; \
                re-measure it before changing the scan resolution");
    let d = design(cpg_gas());
    for name in ALL_SHAPES {
        let m = shaped(d.clone(), name, 0.80, 0.0, 0.0);
        for t in THROTTLE {
            let a = m.authority_ceiling(&flight(), t, Spool::Lp, None);
            assert!(a.v_ch.is_some(), "{name} @{t}: throat unreachable within the scan");
            assert!(a.binds != Binds::Edge,
                    "{name} @{t}: the ARTIFACT bound (v_edge={:.3})", a.v_edge);
            assert!(a.throat_before_edge,
                    "{name} @{t}: v_ch={:.3} did not beat v_edge={:.3}",
                    a.v_ch.unwrap(), a.v_edge);
            assert!(a.c_edge < 0.90, "{name} @{t}: C_edge={:.4} (P-A2)", a.c_edge);
        }
    }
}

// ==========================================================================================
// GATE 7 — P-C2: THE TURNING POINT IS REACHED (rung 53's concession, corrected)
// ==========================================================================================

/// Rung 53 § Concessions: "The incidence benefit SATURATES in v and does not turn back ... (The
/// apparent turning point that this algebra suggests is *not* reached.)" TRUE on the shape rung
/// 53 measured, FALSE on others — the rung-28 shape, verdict kept and reason corrected. Asserted
/// as a CONTRAST so neither half can be vacuous.
#[test]
fn test_the_incidence_peak_is_interior_on_some_shapes_and_not_others() {
    let d = design(cpg_gas());
    let flat = shaped(d.clone(), "flow/press", 0.0, 0.0, 0.0)
        .authority_ceiling(&flight(), 1000.0, Spool::Lp, None);
    assert!(!flat.peak_interior,
            "flow/press is where rung 53 measured: its walk must still run to the edge");
    for (name, min_drop) in [("tilted", 5e-3), ("steep", 3e-2)] {
        let a = shaped(d.clone(), name, 0.0, 0.0, 0.0)
            .authority_ceiling(&flight(), 1000.0, Spool::Lp, None);
        assert!(a.peak_interior, "{name}: expected an INTERIOR incidence peak");
        assert!(a.v_peak < a.v_edge, "{name}: peak not strictly inside the band");
        assert!(a.m_i_peak - a.m_i_edge > min_drop,
                "{name}: the turn-back is immaterial ({:.5}) — rung 53's concession would \
                 effectively stand", a.m_i_peak - a.m_i_edge);
    }
}

/// The consequence for rung 53's P7 payoff object. Where the incidence peak falls short of the
/// DESIGN incidence there is no schedule at all — the stator cannot restore design incidence at
/// any feasible setting. Rung 53 disclosed finite authority (verdict kept) but attributed it to
/// the map-validity edge (reason corrected).
#[test]
fn test_rung_53s_schedule_ceases_to_exist_inside_the_envelope() {
    let d = design(cpg_gas());
    let rows = shaped(d.clone(), "steep", 0.0, 0.0, 0.0)
        .schedule_throat(&flight(), &[1200.0, 1000.0], Spool::Lp);
    assert!(rows[0].exists, "steep still has a schedule at Tt4=1200");
    assert!(!rows[1].exists, "steep must lose its schedule by Tt4=1000");
    assert!(rows[1].found.is_none(), "the 9 schedule keys are DROPPED, not nulled");
    assert!(rows[1].tan_b1_min > rows[1].tan_b1_design,
            "the reason must be that design incidence is UNREACHABLE, not a solver failure");
    // and it survives where rung 53 measured, so the finding is a contrast not a breakage
    let ok = shaped(d, "flow/press", 0.0, 0.0, 0.0)
        .schedule_throat(&flight(), &[1200.0, 1000.0], Spool::Lp);
    assert!(ok.iter().all(|r| r.exists));
}

// ==========================================================================================
// GATE 8 — P-C3: rung 54's root is immune to the ladder rung 53 relies on
// ==========================================================================================

/// Rung 53's `incidence_schedule` justifies its doubling ladder with "the residual is monotone
/// decreasing in v". Where the peak is interior that premise fails and the ladder cannot bracket
/// the root. Rung 54 brackets off a scan and is immune. (Rung 53's own published table is the
/// flow/press shape, where the premise holds — so its numbers stand.)
#[test]
fn test_rung54_root_finds_a_schedule_rung53s_doubling_ladder_walks_over() {
    let d = design(cpg_gas());
    let m = shaped(d, "steep", 0.0, 0.0, 0.0);
    let row = m.schedule_throat(&flight(), &[1200.0], Spool::Lp)[0];
    assert!(row.exists);
    let f = row.found.expect("the schedule exists here");
    assert!((f.vsv_star - 0.909).abs() < 0.01, "vsv_star {}", f.vsv_star);
    assert!((f.tan_b1 - row.tan_b1_design).abs() < 1e-9,
            "the root must actually satisfy the design-incidence condition");
    // the ladder, on the same point, gives up — asserted in a child so the panic is catchable
    // without a separate #[should_panic] test (which could not reach `row.v_edge`).
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        m.incidence_schedule(&flight(), &[1200.0], Spool::Lp, 0.98 * row.v_edge)
    }));
    let err = caught.expect_err("rung 53's ladder must FAIL here");
    let msg = err.downcast_ref::<String>().map(|s| s.as_str())
        .or_else(|| err.downcast_ref::<&str>().copied()).unwrap_or("");
    assert!(msg.contains("does not bracket"), "wrong panic: {msg}");
}

/// The other half of the contrast: no silent divergence from the shipped rung.
#[test]
fn test_rung54_root_agrees_with_rung53_wherever_the_ladder_succeeds() {
    let d = design(cpg_gas());
    let m = shaped(d, "flow/press", 0.0, 0.0, 0.0);
    for t in [1300.0, 1100.0] {
        let mine = m.schedule_throat(&flight(), &[t], Spool::Lp)[0]
            .found.expect("schedule exists").vsv_star;
        let theirs = m.incidence_schedule(&flight(), &[t], Spool::Lp, 1.0)[0].vsv_star;
        assert!((mine - theirs).abs() < 1e-9, "Tt4={t}: {mine} vs {theirs}");
    }
}

// ==========================================================================================
// GATE 9 — THE RACE, and the exposure split it inherits
// ==========================================================================================

/// A CONSTANT-FREE boundary. As power falls the schedule's demand `v*` RISES while the flow `m`
/// FALLS, so `X(v*)` is a race. Above the crossing the schedule asks LESS of the throat than the
/// DESIGN point, so it is feasible for EVERY row whatever its `C`; below, feasibility becomes
/// `C`-dependent. Bracketed, not pinned (the level rides on the disclosed shape).
#[test]
fn test_the_schedule_crosses_the_design_throat_loading_inside_the_envelope() {
    let m = shaped(design(cpg_gas()), "flow/press", 0.0, 0.0, 0.0);
    let grid = [1200.0, 1000.0, 900.0, 870.0, 860.0, 800.0];
    let rows = m.schedule_throat(&flight(), &grid, Spool::Lp);
    assert!(rows.iter().all(|r| r.exists));
    let f = |i: usize| rows[i].found.expect("schedule exists");
    assert!(f(3).throat_loading < 1.0 && 1.0 < f(4).throat_loading,
            "the design-loading crossing must be bracketed by Tt4 = 870 / 860");
    // rung 53's ENTIRE published band sits above the crossing: inert there for any row
    for i in [0usize, 1] {
        assert!(f(i).c_min > 1.0,
                "Tt4={}: c_min={:.4} — must exceed 1, i.e. no row can choke",
                grid[i], f(i).c_min);
    }
    // and the race has an interior minimum: the throttle wins, then the schedule does
    assert!(f(0).throat_loading < f(1).throat_loading);
    assert!(f(0).throat_loading < 1.0);
}

/// The exposure split, INHERITED not new: rung 53's P7 needs `v*_LP >> v*_HP`, and the throat
/// cost goes as `sqrt(1+v^2)`, so the LP eats it quadratically faster. The HP's demand FALLS
/// monotonically and never approaches its throat.
#[test]
fn test_the_capacity_ceiling_is_a_pure_lp_object() {
    let m = shaped(design(cpg_gas()), "flow/press", 0.0, 0.0, 0.0);
    let hp = m.schedule_throat(&flight(), &[1400.0, 1200.0, 1000.0, 800.0], Spool::Hp);
    assert!(hp.iter().all(|r| r.exists));
    let loads: Vec<f64> = hp.iter().map(|r| r.found.unwrap().throat_loading).collect();
    assert!(loads.windows(2).all(|w| w[0] >= w[1]), "HP demand should FALL, got {loads:?}");
    let mx = loads.iter().cloned().fold(f64::MIN, f64::max);
    let mn = loads.iter().cloned().fold(f64::MAX, f64::min);
    assert!(mx < 1.0 && mn < 0.75);
    let lp = m.schedule_throat(&flight(), &[1400.0, 800.0], Spool::Lp);
    let (lp_hi, lp_lo) = (lp[0].found.unwrap(), lp[1].found.unwrap());
    assert!(lp_lo.throat_loading > lp_hi.throat_loading, "LP must turn back up");
    assert!(lp_lo.vsv_star > 3.0 * hp[3].found.unwrap().vsv_star);
}

// ==========================================================================================
// GATE 10 — CYCLE UNTOUCHED
// ==========================================================================================

/// Rung 54 adds a diagnostic field and pure read methods; the default single-spool design path
/// must be untouched, as at every rung since 7.
#[test]
fn test_cycle_untouched_design_run_is_bit_for_bit_rung6() {
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1600.0, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, ..Losses::default()
    });
    let r = eng.run(&flight(), 1.0);
    assert!(r.performance.specific_thrust > 0.0);
    // the new field is inert by default, and does not disturb rung 53's flatness rule
    assert_eq!(ComponentMap::flat().capacity, 0.0);
    assert!(ComponentMap::flat().is_flat());
}
