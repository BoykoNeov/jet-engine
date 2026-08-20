//! RUNG 56 — PER-ROW CAPACITY: two constraints on one machine, at opposite ends.
//!
//! Port of `tests/test_rung56.py`, gate for gate. Its ten gate groups:
//!
//!   1. REDUCE — an INVARIANCE over BOTH the constant and the profile, on a stack that DOES
//!      enter the solver; plus `K = 1` reproducing rung 54's own `throat_margin` to the last bit.
//!   2. THE DERIVED PROFILE — `C_0` is the disclosed constant EXACTLY, `C_k` falls monotonically,
//!      and it is the total-referenced Mach `nu` that scales as `1/sqrt(theta_k,d)`.
//!   3. THE PER-ROW CURRENCY — `m_k = phi_k*n_k` exactly, `X_k` is rung 54's law at the row's OWN
//!      setting, and the design tie is a TOLERANCE (~2e-14), not an identity.
//!   4. THE NON-TAUTOLOGY GATE — the amplification is EXACTLY 1.0 at `K = 1` and grows with
//!      throttle depth; a resolution gap, not a feedback one.
//!   5. P1 — the binding row MIGRATES: front near design, rear at part power, one-way.
//!   6. P3 — `K` is a RESOLUTION: the increments shrink.
//!   7. P4 — the disclosed SPLIT is LOAD-BEARING here (contrast rung 55 P6, where it was not).
//!   8. P5 — THE HEADLINE: the two constraints land at opposite ENDS and on opposite SPOOLS;
//!      and rung 54's "the HP never approaches its throat" is CORRECTED by resolution.
//!   9. P6 — the positional lever DEBITS the row it does not move, and the advantage is
//!      CURRENCY-DEPENDENT.
//!  10. CYCLE UNTOUCHED.
//!
//! Python marks one of these `@pytest.mark.slow`; the marker is not carried over — see
//! `rung53.rs`'s header.
//!
//! # Where this file is not a transcription
//!
//! `test_uniform_profile_is_the_disclosed_alternative` SPLITS: `#[should_panic]` is per-test and
//! one of its two refusals (`cap_profile="quadratic"`) is unrepresentable against an enum. And
//! one gate is ADDED — the `nu` range guard, which Python never reaches. Both directions of the
//! name diff are booked in `rung55.rs::slice_n_deferrals`.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::{mach_of_nu, mfp_frac, nu_of_mach, ComponentMap};
use turbojet::stage::{CapProfile, Split, StageStack, StageStackCore, StageStackCoreSpec,
                      StageStackSpec};
use turbojet::stator::VariableStatorCore;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const CAP: f64 = 0.90;
const THROTTLE: [f64; 4] = [1500.0, 1200.0, 1000.0, 800.0];
const WALK: [f64; 8] = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 800.0];
const ALL_SHAPES: [&str; 5] = ["flow/press", "press/flow", "tilted", "steep", "flat-eta"];

/// B1, MEASURED not guessed: the `K`-stage march does not reproduce `X_k = 1` at design to the
/// bit — `max|X_k − 1|` runs 7.8e-15 … 1.9e-14 over `K = 2..16` on both spools. So the design tie
/// is a tolerance, and binding-row identity at design under the UNIFORM profile is noise.
const DESIGN_DRIFT: f64 = 1e-12;

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

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

/// Rung 53/54/55's five disclosed shapes — armed with BOTH walls, which is what makes this
/// rung 56's file and not rung 55's.
fn bare_shape(name: &str) -> (ComponentMap, ComponentMap) {
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
        other => panic!("unknown shape {other}"),
    }
}

/// Python's `_maps`.
fn maps(name: &str, c: f64) -> (ComponentMap, ComponentMap) {
    let (l, h) = bare_shape(name);
    (l.with_phi_surge(FLOOR).with_capacity(c), h.with_phi_surge(FLOOR).with_capacity(c))
}

/// Python's `_sm`, with its nine defaults.
#[allow(clippy::too_many_arguments)]
fn sm(d: TwoSpoolEngine, shape: &str, c: f64, k: usize, prof: CapProfile, split: Split,
      vl: f64, vh: f64, vs_lp: Option<usize>) -> StageStackCore {
    let (ml, mh) = maps(shape, c);
    StageStackCore::new(StageStackCoreSpec {
        vsv_lp: vl, vsv_hp: vh, k_lp: k, k_hp: k, split, cap_profile: prof,
        vsv_stages_lp: vs_lp, vsv_stages_hp: None,
        ..StageStackCoreSpec::new(d, flight(), 1.0, ml, mh)
    })
}

/// The default call: `flow/press`, `C = 0.90`, `K = 8`, derived, `dT`, stator at design.
fn sm8(d: TwoSpoolEngine) -> StageStackCore {
    sm(d, "flow/press", CAP, 8, CapProfile::Derived, Split::DT, 0.0, 0.0, None)
}

/// The 19 matched fields, as raw bits.
fn fields(o: turbojet::two_spool::TwoSpoolMapResult) -> Vec<(&'static str, u64)> {
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

fn rel_close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * a.abs().max(b.abs())
}

// ==========================================================================================
// GATE 1 — REDUCE: an INVARIANCE over the constant AND over the profile
// ==========================================================================================

/// THE SPINE, in rung 54's stronger form. Rung 54 earned an invariance over `C` on a channel
/// that entered no solver at all; rung 55's STACK does enter the solver, so this is no longer
/// free — `capacity` and `cap_profile` ride on objects (`ComponentMap`, `StageStack`) the
/// speed-line inversion consumes. Every matched field must still be bit-identical for every `C`
/// and both profiles, at a MOVED stator.
#[test]
fn test_reduce_invariance_over_capacity_and_profile() {
    for (vl, vh) in [(0.0, 0.0), (0.30, 0.0), (0.20, 0.10)] {
        let d = design(cpg_gas());
        // a throat model that is ~off
        let r = sm(d.clone(), "flow/press", 1e-9, 8, CapProfile::Derived, Split::DT, vl, vh, None);
        // …and one with NO throat model at all
        let (bl, bh) = bare_shape("flow/press");
        let nothroat = StageStackCore::new(StageStackCoreSpec {
            vsv_lp: vl, vsv_hp: vh, k_lp: 8, k_hp: 8,
            ..StageStackCoreSpec::new(d.clone(), flight(), 1.0,
                                      bl.with_phi_surge(FLOOR), bh.with_phi_surge(FLOOR))
        });
        let mut cases = vec![nothroat];
        for c in [0.30, 0.70, 0.90, 0.99] {
            for p in [CapProfile::Derived, CapProfile::Uniform] {
                cases.push(sm(d.clone(), "flow/press", c, 8, p, Split::DT, vl, vh, None));
            }
        }
        assert_eq!(cases.len(), 9, "one throat-free case and four constants on two profiles");
        for t in THROTTLE {
            let a = fields(r.match_point(&flight(), t));
            for other in &cases {
                for (got, want) in fields(other.match_point(&flight(), t)).iter().zip(a.iter()) {
                    assert_eq!(got.1, want.1,
                               "rung-56 invariance broken on {} at Tt4={t}, C={}, profile={:?}",
                               got.0, other.core.core.map_lp().capacity, other.cap_profile());
                }
            }
        }
    }
}

/// At `K = 1` there is no stack, so `stage_throat_margin`'s single row must BE rung 54's face
/// read — the same `X`, the same margin, the same `c_min` — to the last bit, on both profiles
/// (which cannot differ when there is only one row) and at a moved stator.
///
/// **The `K = 1` branch calls `cmap.throat_ratio()`/`throat_loading`/`capacity_margin` VERBATIM**
/// rather than re-deriving them through the per-row law, which is what makes this an identity
/// and not an algebraic agreement — slice D/E's *an "exactly" claim survives a copied
/// instruction sequence and dies on a second derivation.*
#[test]
fn test_reduce_k1_is_rung54_throat_margin_bit_for_bit() {
    let d = design(cpg_gas());
    for vl in [0.0, 0.30] {
        let (ml, mh) = maps("flow/press", CAP);
        let r54 = VariableStatorCore::new(d.clone(), flight(), 1.0, ml, mh, vl, 0.0);
        for prof in [CapProfile::Derived, CapProfile::Uniform] {
            let st = sm(d.clone(), "flow/press", CAP, 1, prof, Split::DT, vl, 0.0, None);
            for t in THROTTLE {
                let a = r54.throat_margin(&flight(), t);
                let b = st.stage_throat_margin(&flight(), t);
                for spool in [Spool::Lp, Spool::Hp] {
                    let row = &b.spool(spool).stages[0];
                    let face = a.spool(spool).throat.expect("rung 54's throat read");
                    assert_eq!(row.throat_loading.to_bits(), face.throat_loading.to_bits());
                    assert_eq!(row.m_c.to_bits(),
                               face.choke.expect("C > 0 gives the choke read").m_c.to_bits());
                    assert_eq!(row.c_min.to_bits(), face.c_min.to_bits());
                    assert_eq!(b.spool(spool).amplification, 1.0,
                               "at K = 1 the binding row IS the face: amplification is exactly 1");
                }
            }
        }
    }
}

/// A hand-built one-stage stack carries exactly the disclosed constant, on both profiles and for
/// any `gamma` — `theta_d[0] == 1`, so the derived ladder cannot bite.
#[test]
fn test_reduce_stack_capacities_at_k1() {
    let m = sm8(design(cpg_gas()));
    let cmap = maps("flow/press", CAP).0;
    let (tau_d, pi_d, eta_d) = (m.core.core.tau_lpc_d, m.core.core.base.pi_lpc_design,
                                m.core.core.base.eta_lpc);
    for prof in [CapProfile::Derived, CapProfile::Uniform] {
        for g in [1.3, 1.4, 1.667] {
            let st = StageStack::new(StageStackSpec {
                cap_profile: prof, gamma_th: g,
                ..StageStackSpec::new(1, cmap, tau_d, pi_d, eta_d) });
            assert_eq!(st.capacities(), &[CAP][..],
                       "K=1 must carry the disclosed level alone (profile={prof:?}, gamma={g})");
            assert_eq!(st.stage_capacity_margin(0, 0.7).to_bits(),
                       cmap.capacity_margin(0.7).to_bits());
        }
    }
}

// ==========================================================================================
// GATE 2 — THE DERIVED PROFILE: shape derived, level disclosed
// ==========================================================================================

/// `C_0` is rung 54's constant EXACTLY (not a bisection round-trip), the profile falls
/// monotonically rearward, and the object that actually scales as `1/sqrt(theta_k,d)` is the
/// TOTAL-referenced Mach `nu` — which is what a common design throat VELOCITY at rising `Tt`
/// means. Zero new constants beyond rung 54's one.
#[test]
fn test_derived_profile_is_the_ladder_and_the_level_is_the_front_row() {
    let m = sm8(design(cpg_gas()));
    for spool in [Spool::Lp, Spool::Hp] {
        let st = m.stack_of(spool).expect("K = 8 builds a stack");
        let cs = st.capacities();
        assert_eq!(cs[0].to_bits(), CAP.to_bits(),
                   "the disclosed level IS the front row's C, exactly");
        assert_eq!(cs.len(), st.k);
        for k in 0..st.k - 1 {
            assert!(cs[k + 1] < cs[k],
                    "rung-56 derived profile must FALL rearward on {spool:?} (rising Tt at a \
                     common throat velocity), got {cs:?}");
        }
        // the derivation itself: nu_k * sqrt(theta_k,d) is invariant
        let g = st.gamma_th;
        let nu1 = nu_of_mach(st.cmap.design_throat_mach(g), g);
        let nu: Vec<f64> = (0..st.k)
            .map(|k| {
                let s = st.theta_d[k].powf(0.5);
                nu_of_mach(mach_of_nu(nu1 / s, g), g) * s
            })
            .collect();
        for x in &nu {
            assert!(rel_close(*x, nu[0], 1e-12),
                    "nu*sqrt(theta) must be invariant down the stack: {nu:?}");
        }
        // and each C_k IS the MFP fraction of a Mach BELOW the front row's. Python defaults
        // `gamma` to 1.4 on all three helpers HERE, so the call sites spell 1.4 and not
        // `st.gamma_th` — the same value on this stack, and a different spelling on purpose.
        let last = mfp_frac(
            mach_of_nu(nu_of_mach(st.cmap.design_throat_mach(1.4), 1.4)
                           / st.theta_d[st.k - 1].powf(0.5), 1.4), 1.4);
        assert!(rel_close(cs[st.k - 1], last, 1e-12), "{} vs {last}", cs[st.k - 1]);
    }
}

/// The uniform profile is rung 54's constant on every row — the disclosed alternative the level
/// claims are refused against.
#[test]
fn test_uniform_profile_is_the_disclosed_alternative() {
    let m = sm(design(cpg_gas()), "flow/press", CAP, 8, CapProfile::Uniform, Split::DT,
               0.0, 0.0, None);
    for spool in [Spool::Lp, Spool::Hp] {
        let st = m.stack_of(spool).expect("K = 8 builds a stack");
        assert_eq!(st.capacities(), &vec![CAP; st.k][..]);
    }
}

/// The other half of Python's `test_uniform_profile_is_the_disclosed_alternative`: no throat
/// model ⇒ no per-row capacity. Split out because `#[should_panic]` is per-test.
///
/// **AND IT IS CALLED ON THE STACK DIRECTLY, NOT THROUGH THE MATCHER, DELIBERATELY.** § 5.10's
/// laziness note measured that through `StageStackMatcher` the same capacity-free maps give
/// `match`, `stage_margin`, `work_gap` and `stage_incidence_schedule` all OK, and
/// `stage_throat_margin` raises from the MATCHER's own `cmap` assert — a second assert carrying
/// the same sentence, which always wins. A matcher-driven version of this gate would read as
/// though it gated the stack's guard while gating the matcher's.
#[test]
#[should_panic(expected = "needs rung 54's throat model")]
fn test_per_row_capacity_needs_a_throat_model() {
    let m = sm8(design(cpg_gas()));
    let (tau_d, pi_d, eta_d) = (m.core.core.tau_lpc_d, m.core.core.base.pi_lpc_design,
                                m.core.core.base.eta_lpc);
    let bare = bare_shape("flow/press").0.with_phi_surge(FLOOR);
    let st = StageStack::new(StageStackSpec::new(4, bare, tau_d, pi_d, eta_d));
    // Construction is FINE — § 5.10 measured it — and the raise lands at FIRST READ.
    let _ = st.capacities();
}

/// The profile is a functional of the ladder, so the spool with the larger design temperature
/// rise has the steeper Mach fall. Not fitted — read off `tau_d`.
#[test]
fn test_hp_profile_falls_harder_than_lp() {
    let m = sm8(design(cpg_gas()));
    let (lp, hp) = (m.stack_of(Spool::Lp).unwrap(), m.stack_of(Spool::Hp).unwrap());
    let (cl, ch) = (lp.capacities(), hp.capacities());
    assert!(hp.tau_d > lp.tau_d);
    assert!(ch[ch.len() - 1] / ch[0] < cl[cl.len() - 1] / cl[0]);
}

// ==========================================================================================
// GATE 3 — THE PER-ROW CURRENCY
// ==========================================================================================

/// `m_k = phi_k * n_k` is an IDENTITY at every station (the face relation `m = phi*n`, per row),
/// and `X_k` applies rung 54's derived area law at the setting THAT row carries — the design
/// setting for every row a front-block stator does not move.
#[test]
fn test_per_row_corrected_flow_is_phi_times_n_and_x_is_rung54s_law() {
    let m = sm(design(cpg_gas()), "flow/press", CAP, 8, CapProfile::Derived, Split::DT,
               0.40, 0.0, Some(3));
    let r = m.stage_throat_margin(&flight(), 1000.0);
    for (k, s) in r.lp.stages.iter().enumerate() {
        assert_eq!(s.m_k.to_bits(), (s.phi * s.n).to_bits());
        assert_eq!(s.vsv, if k < 3 { 0.40 } else { 0.0 },
                   "only the front block carries the setting — that positional split is the rung");
        assert!(rel_close(s.throat_loading, s.m_k * (1.0 + s.vsv * s.vsv).sqrt(), 1e-15));
        assert!(rel_close(s.m_c, 1.0 - s.capacity * s.throat_loading, 1e-15));
        assert!(rel_close(s.c_min, 1.0 / s.throat_loading, 1e-15));
    }
}

/// B1, gated as measured. At design every `X_k` should be 1; in floating point the `K`-stage
/// march drifts by ~1e-14. So this is a tolerance — and consequently no binding-row claim may be
/// gated at design under the UNIFORM profile, where the rows are otherwise tied.
///
/// This is § 5.10 (iv)'s degenerate argmin from the other side: the same 1–2 ULP spread that
/// makes 13 of 1 280 half-rows report an INTERIOR binding row is what this gate refuses to let
/// any verdict ride on.
#[test]
fn test_design_tie_is_a_tolerance_not_an_identity() {
    for k in [2usize, 4, 8, 16] {
        let m = sm(design(cpg_gas()), "flow/press", CAP, k, CapProfile::Uniform, Split::DT,
                   0.0, 0.0, None);
        for spool in [Spool::Lp, Spool::Hp] {
            let r = m.stage_throat_margin(&flight(), TT4);
            let xs: Vec<f64> = r.spool(spool).stages.iter().map(|s| s.throat_loading).collect();
            let worst = xs.iter().map(|x| (x - 1.0).abs()).fold(0.0f64, f64::max);
            let (lo, hi) = (xs.iter().cloned().fold(f64::INFINITY, f64::min),
                            xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
            assert!(worst < DESIGN_DRIFT, "K={k} {spool:?}: {worst:e}");
            assert!(hi - lo < DESIGN_DRIFT);
            assert!(hi - lo > 0.0,
                    "the drift is REAL — if this ever becomes an exact tie the noise warning \
                     can be dropped, but until then binding-row gates must avoid design+uniform");
        }
    }
}

// ==========================================================================================
// GATE 4 — THE NON-TAUTOLOGY GATE: a RESOLUTION gap, not a feedback one
// ==========================================================================================

/// P2. The channel enters no solver (rung 54 P1, inherited), so what makes rung 56 content is
/// RESOLUTION: at the SAME solved state the binding row's throat deficit exceeds the face
/// deficit rung 54 could read. Exactly 1.0 at `K = 1`, and growing with throttle depth.
#[test]
fn test_amplification_is_the_non_tautology_gate() {
    for shape in ALL_SHAPES {
        for split in [Split::DT, Split::Tau] {
            let d = design(cpg_gas());
            let one = sm(d.clone(), shape, CAP, 1, CapProfile::Derived, split, 0.0, 0.0, None);
            let eight = sm(d, shape, CAP, 8, CapProfile::Derived, split, 0.0, 0.0, None);
            for spool in [Spool::Lp, Spool::Hp] {
                assert_eq!(one.stage_throat_margin(&flight(), 800.0).spool(spool).amplification,
                           1.0);
                let vals: Vec<f64> = [1200.0, 1000.0, 800.0].iter()
                    .map(|&t| eight.stage_throat_margin(&flight(), t).spool(spool).amplification)
                    .collect();
                assert!(vals[0] <= vals[1] && vals[1] < vals[2],
                        "the amplification must grow with throttle depth on {spool:?}/{shape}: \
                         {vals:?}");
                assert!(vals[2] >= 1.15,
                        "P2's band: >= 1.15x at Tt4 = 800 on {spool:?}/{shape}/{split:?}, got {}",
                        vals[2]);
            }
        }
    }
}

/// P2's second half. The derived profile is PROTECTIVE — it designs the rear rows with more
/// capacity — so the naive uniform read overstates the rear's exposure. This is why the profile
/// is disclosed and no LEVEL claim is made robust to it.
#[test]
fn test_uniform_profile_amplifies_harder_than_derived() {
    let d = design(cpg_gas());
    let dd = sm8(d.clone());
    let u = sm(d, "flow/press", CAP, 8, CapProfile::Uniform, Split::DT, 0.0, 0.0, None);
    for t in [1000.0, 800.0] {
        for spool in [Spool::Lp, Spool::Hp] {
            assert!(u.stage_throat_margin(&flight(), t).spool(spool).amplification
                    > dd.stage_throat_margin(&flight(), t).spool(spool).amplification);
        }
    }
}

// ==========================================================================================
// GATE 5 — P1: the binding row MIGRATES (HIT at part power, REFUTED near design)
// ==========================================================================================

/// P1. The derived profile designs the rear rows with MORE capacity exactly where the off-design
/// march loads them hardest, so the two fight: the profile wins near design (the FRONT binds)
/// and the loading wins at part power (the REAR binds). Rung 55's seam predicted only the rear —
/// it is HIT at part power and REFUTED near design, for a DERIVED reason.
///
/// RUN UNDER BOTH SPLITS DELIBERATELY. What is pinned is the migration's EXISTENCE and
/// ONE-WAYNESS plus the two EXTREME cells; the interior crossover cell is genuinely fragile to
/// the split (`press/flow` HP at `Tt4 = 1200` binds row 2 on `dT` and row 3 on `tau`) and is
/// never asserted.
#[test]
fn test_binding_row_migrates_front_to_rear() {
    for shape in ALL_SHAPES {
        for split in [Split::DT, Split::Tau] {
            let m = sm(design(cpg_gas()), shape, CAP, 8, CapProfile::Derived, split,
                       0.0, 0.0, None);
            for spool in [Spool::Lp, Spool::Hp] {
                let w = m.throat_walk(&flight(), &WALK, spool);
                assert_eq!(w[0].binds, 0,
                           "near design the derived PROFILE binds (front row, highest Mach) on \
                            {spool:?}/{shape}, got row {}", w[0].binds);
                let last = &w[w.len() - 1];
                assert_eq!(last.binds, last.margins.len() - 1,
                           "at part power the LOADING binds (rear row) on {spool:?}/{shape}");
                let first_rear = w.iter().position(|r| r.binds == r.margins.len() - 1)
                    .expect("the rear binds somewhere on this walk");
                assert!(w[first_rear..].iter().all(|r| r.binds == r.margins.len() - 1),
                        "the migration must be one-way: once the loading wins it does not \
                         hand back");
                for r in [&w[0], last] {         // the two pinned cells are decided by a WIDE gap
                    let mut g = r.margins.clone();
                    g.sort_by(f64::total_cmp);
                    assert!(g[1] - g[0] > 1e-3,
                            "a pinned binding-row cell must not be near-degenerate \
                             ({spool:?}/{shape}/{split:?}, Tt4={}): gap {:.2e}",
                            r.tt4, g[1] - g[0]);
                }
            }
        }
    }
}

/// The control. Strip the derived profile and the contest disappears — `X_k` alone decides, and
/// it rises rearward monotonically. DESIGN IS EXCLUDED: there the rows tie to ~1e-14 and the
/// binding row is float noise (gate 3).
#[test]
fn test_uniform_profile_binds_at_the_rear_at_every_off_design_throttle() {
    for shape in ALL_SHAPES {
        let m = sm(design(cpg_gas()), shape, CAP, 8, CapProfile::Uniform, Split::DT,
                   0.0, 0.0, None);
        for spool in [Spool::Lp, Spool::Hp] {
            for r in m.throat_walk(&flight(), &WALK[1..], spool) {
                assert_eq!(r.binds, r.margins.len() - 1,
                           "uniform C must bind at the rear at Tt4={} on {spool:?}/{shape}",
                           r.tt4);
            }
        }
    }
}

// ==========================================================================================
// GATE 6 — P3: K is a RESOLUTION
// ==========================================================================================

/// P3. The amplification grows with `K` but its increments shrink monotonically, so the
/// disclosed integer is a resolution coordinate and no claim rides on a particular `K`.
/// (Scored HONESTLY: the LP increments halve; the HP's shrink by ~0.53 per doubling, which
/// MISSES the pre-registered "at least halves" band while confirming what it encoded.)
#[test]
fn test_k_is_a_resolution_increments_shrink() {
    let d = design(cpg_gas());
    for spool in [Spool::Lp, Spool::Hp] {
        let vals: Vec<f64> = [1usize, 2, 4, 8, 16, 32].iter()
            .map(|&k| sm(d.clone(), "flow/press", CAP, k, CapProfile::Derived, Split::DT,
                         0.0, 0.0, None)
                    .stage_throat_margin(&flight(), 800.0).spool(spool).amplification)
            .collect();
        let inc: Vec<f64> = vals.windows(2).map(|w| w[1] - w[0]).collect();
        assert!(inc.iter().all(|&x| x > 0.0),
                "amplification must grow with K on {spool:?}: {vals:?}");
        for w in inc.windows(2) {
            assert!(w[1] < w[0], "increments must SHRINK on {spool:?}: {inc:?}");
            assert!(w[1] / w[0] < 0.60,
                    "and shrink geometrically (first order) on {spool:?}: {inc:?}");
        }
    }
}

// ==========================================================================================
// GATE 7 — P4: the disclosed SPLIT is LOAD-BEARING here (contrast rung 55 P6)
// ==========================================================================================

/// P4, and an honest inversion of rung 55 P6. The amplification rides on the internal
/// theta/varpi ladder, which is exactly what the disclosed work split moves — so unlike rung 55
/// (where the split moved `d_phi` by 0.01 %) it moves this by 2–5 %. The LEVELS are therefore
/// disclaimed on the split; the SIGNS and the part-power binding row are not.
#[test]
fn test_split_is_load_bearing_but_carries_no_sign() {
    for shape in ALL_SHAPES {
        let d = design(cpg_gas());
        let a = sm(d.clone(), shape, CAP, 8, CapProfile::Derived, Split::DT, 0.0, 0.0, None);
        let b = sm(d, shape, CAP, 8, CapProfile::Derived, Split::Tau, 0.0, 0.0, None);
        for spool in [Spool::Lp, Spool::Hp] {
            for t in [1000.0, 800.0] {
                let x = a.stage_throat_margin(&flight(), t);
                let y = b.stage_throat_margin(&flight(), t);
                let (x, y) = (x.spool(spool), y.spool(spool));
                let rel = (y.amplification - x.amplification).abs() / (x.amplification - 1.0);
                // The PRE-REGISTERED band is HP at Tt4 = 800 (> 2 %); it holds there on every
                // shape, 3.7–4.6 %. Off that cell the sweep is asserted at its MEASURED floor —
                // the tightest cell in the grid is flat-eta LP at 1000, 1.98 %. Reported, not
                // rounded up to the prediction: the band the prediction named is the one scored.
                let floor = if spool == Spool::Hp && t == 800.0 { 0.02 } else { 0.017 };
                assert!(rel > floor,
                        "P4 says the split MOVES this ({spool:?}/{shape}/{t}): rel = {rel:.4}");
                assert!(x.binds == y.binds && x.binds == x.stages.len() - 1,
                        "but it must not move the part-power binding row");
                assert_eq!(x.m_c_worst > 0.0, y.m_c_worst > 0.0);
            }
        }
        // …and two orders of magnitude above rung 55 P6's 0.01 %, which is the contrast.
        assert_ne!(a.stage_throat_margin(&flight(), 800.0).hp.amplification.to_bits(),
                   b.stage_throat_margin(&flight(), 800.0).hp.amplification.to_bits());
    }
}

// ==========================================================================================
// GATE 8 — P5: THE HEADLINE. Opposite ENDS, opposite SPOOLS; and rung 54 CORRECTED
// ==========================================================================================

/// P5 — THE HEADLINE, in its strong form. Rung 55's seam predicted front-vs-back on one machine.
/// Measured, it is more than that: at part power the worst INCIDENCE margin in the whole machine
/// is the LP's FRONT row and the worst CAPACITY margin is the HP's REAR row. Opposite end AND
/// opposite spool — a lumped block has one `phi` and one face, and cannot express either
/// statement, let alone their separation.
#[test]
fn test_two_constraints_opposite_ends_and_opposite_spools() {
    for shape in ALL_SHAPES {
        let m = sm(design(cpg_gas()), shape, CAP, 8, CapProfile::Derived, Split::DT,
                   0.0, 0.0, None);
        for t in [1000.0, 800.0] {
            let r = m.stage_throat_margin(&flight(), t);
            let (lp, hp) = (&r.lp, &r.hp);
            assert!(lp.inc_worst == 0 && hp.inc_worst == 0,
                    "incidence binds at the FRONT of each spool (rung 55 P4, inherited)");
            assert!(lp.binds == lp.stages.len() - 1 && hp.binds == hp.stages.len() - 1,
                    "capacity binds at the REAR of each spool");
            assert!(lp.m_i_worst < hp.m_i_worst,
                    "the machine's INCIDENCE exposure is the LP's (rungs 41/44/45/53's split)");
            assert!(hp.m_c_worst < lp.m_c_worst,
                    "but the machine's CAPACITY exposure is the HP's — the opposite spool");
        }
    }
}

/// Rung 54 § The exposure split wrote: *"The HP schedule's demand falls monotonically and never
/// approaches its throat at any throttle."* At the FACE that is true and stays true. Resolved
/// into rows it is nearly false: the HP REAR row's margin FALLS with throttle while the face's
/// RISES, and the threshold on the constant reaches `C* ~ 0.91`.
///
/// The rung-28 shape: the face-level reasoning survives as a face-level statement, and the
/// verdict it supported is corrected by resolution. Stated as a THRESHOLD ON the constant
/// (rung 54's discipline), never as a level.
#[test]
fn test_rung54s_hp_throat_claim_is_corrected_by_resolution() {
    let d = design(cpg_gas());
    let m = sm8(d.clone());
    let (mut face, mut rear, mut cstar) = (Vec::new(), Vec::new(), Vec::new());
    for t in [1200.0, 1000.0, 800.0] {
        let r = m.stage_throat_margin(&flight(), t);
        let hp = &r.hp;
        face.push(hp.m_c_face);
        rear.push(hp.stages[hp.stages.len() - 1].m_c);
        cstar.push(hp.stages[hp.stages.len() - 1].c_min);
    }
    assert!(face[0] < face[1] && face[1] < face[2],
            "at the FACE the HP relaxes with throttle (rung 54)");
    assert!(rear[0] > rear[1] && rear[1] > rear[2],
            "at the REAR ROW it TIGHTENS — the opposite sign");
    assert!(cstar[2] < 0.92,
            "and the constant-free threshold reaches C* = {:.4}: any HP row whose design \
             capacity fraction exceeds it is CHOKED at Tt4 = 800", cstar[2]);
    let u = sm(d, "flow/press", CAP, 8, CapProfile::Uniform, Split::DT, 0.0, 0.0, None);
    let ur = u.stage_throat_margin(&flight(), 800.0);
    let last = &ur.hp.stages[ur.hp.stages.len() - 1];
    assert!(0.0 < last.m_c && last.m_c < 0.02,
            "and on the naive UNIFORM profile that row is a hair from choking ({:.4}) — which is \
             what makes the derived profile a finding and not furniture", last.m_c);
}

/// Rung 54's refusal, inherited EXPLICITLY. The rear row at `C = 0.90` is close to the wall and
/// it is tempting to let it bind; rung 54 already priced that as inverting rung 31's `(*)`, the
/// flow being set at the first choked throat DOWNSTREAM. So a choked row must change nothing
/// that is solved.
#[test]
fn test_capacity_channel_stays_diagnostic_only() {
    let d = design(cpg_gas());
    let a = sm(d.clone(), "flow/press", 0.99, 8, CapProfile::Uniform, Split::DT, 0.0, 0.0, None);
    assert!(a.stage_throat_margin(&flight(), 800.0).hp.chokes,
            "pick a C that provably chokes the binding row, or this gate is vacuous");
    let b = sm(d, "flow/press", 0.30, 8, CapProfile::Derived, Split::DT, 0.0, 0.0, None);
    assert!(!b.stage_throat_margin(&flight(), 800.0).hp.chokes);
    for (got, want) in fields(a.match_point(&flight(), 800.0)).iter()
        .zip(fields(b.match_point(&flight(), 800.0)).iter())
    {
        assert_eq!(got.1, want.1, "a CHOKED row moved {} — the channel entered a solver", got.0);
    }
}

// ==========================================================================================
// GATE 9 — P6: the positional lever's DEBIT, and its currency dependence
// ==========================================================================================

/// P6's sign, measured before the headline was fixed. The front-row stator reaches an unmoved
/// rear row only through the solved `(m, n)` — and the sign is a DEBIT: closing it costs the rear
/// row throat margin, monotonically. Rung 55's honest half (the shaft speed is the one thing
/// every stage shares) in a second currency.
#[test]
fn test_front_row_lever_debits_the_row_it_does_not_move() {
    let d = design(cpg_gas());
    let rear_mc = |v: f64, t: f64| {
        let m = sm(d.clone(), "flow/press", CAP, 8, CapProfile::Derived, Split::DT,
                   v, 0.0, Some(1));
        let r = m.stage_throat_margin(&flight(), t);
        r.lp.stages[r.lp.stages.len() - 1].m_c
    };
    let front_mc = |v: f64, t: f64| {
        let m = sm(d.clone(), "flow/press", CAP, 8, CapProfile::Derived, Split::DT,
                   v, 0.0, Some(1));
        m.stage_throat_margin(&flight(), t).lp.stages[0].m_c
    };
    for t in [1000.0, 800.0] {
        let rear: Vec<f64> = [0.0, 0.20, 0.3536, 0.60].iter().map(|&v| rear_mc(v, t)).collect();
        assert!(rear.windows(2).all(|w| w[1] < w[0]),
                "the debit must be monotone in v: {rear:?}");
        let front = [front_mc(0.0, t), front_mc(0.60, t)];
        assert!(front[0] - front[1] > 10.0 * (rear[0] - rear[3]),
                "and the lever's throat cost must land overwhelmingly on the row it MOVES");
    }
}

/// P6 scored a MISS, and the miss is the content. The rear-row debit ratio (front-only / lumped)
/// was predicted to track rung 55's `dN` ratio within 25 %. It does not: the SPEED ratio is
/// nearly `v`-invariant (~0.11–0.13) while the THROAT ratio COLLAPSES with the setting
/// (0.18 → 0.03). The lumped lever spends every row's throat directly, by `sqrt(1+v^2)`, on top
/// of the speed rise — so the positional lever's advantage is larger, and grows, in exactly the
/// currency rung 54 introduced. Rung 53's law a fourth time: the LEVER'S COST is
/// coordinate-dependent too.
#[test]
fn test_positional_advantage_is_currency_dependent() {
    let d = design(cpg_gas());
    let t = 1000.0;
    let read = |v: f64, vs: Option<usize>| {
        sm(d.clone(), "flow/press", CAP, 8, CapProfile::Derived, Split::DT, v, 0.0, vs)
            .stage_throat_margin(&flight(), t)
    };
    let base = read(0.0, None);
    let (base_rear, base_n) = (base.lp.stages[base.lp.stages.len() - 1].m_c, base.lp.n);
    let (mut thr, mut spd) = (Vec::new(), Vec::new());
    for v in [0.20, 0.3536, 0.60] {
        let fr = read(v, Some(1));
        let lu = read(v, None);
        let (fr_rear, lu_rear) = (fr.lp.stages[fr.lp.stages.len() - 1].m_c,
                                  lu.lp.stages[lu.lp.stages.len() - 1].m_c);
        thr.push((base_rear - fr_rear) / (base_rear - lu_rear));
        spd.push(((fr.lp.n - base_n) / base_n) / ((lu.lp.n - base_n) / base_n));
    }
    assert!(thr.iter().all(|&x| x < 1.0) && spd.iter().all(|&x| x < 1.0),
            "the positional lever must be cheaper in BOTH currencies");
    assert!(thr[0] > thr[1] && thr[1] > thr[2], "the THROAT ratio collapses with v: {thr:?}");
    let (lo, hi) = (spd.iter().cloned().fold(f64::INFINITY, f64::min),
                    spd.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!(hi - lo < 0.10 * lo, "the SPEED ratio does not: {spd:?}");
    assert!(thr[2] < 0.5 * spd[2],
            "so at the larger setting the two currencies disagree by >2x: {:.4} vs {:.4} — the \
             pre-registered 'within 25 %' is REFUTED", thr[2], spd[2]);
}

/// B3, rung 50's shape in a currency rung 50 never saw. Push the front-row setting far enough
/// and the row's OWN throat cost `sqrt(1+v^2)` overwhelms the rear's loading, so the binding
/// capacity row relocates to the moved row. The threshold sits well ABOVE rung 55's own
/// front-row schedule (`v* ~ 0.35`), so rung 55's published lever does not trip it.
#[test]
fn test_lever_relocates_the_binding_row_to_itself_at_large_setting() {
    let d = design(cpg_gas());
    for t in [1000.0, 800.0] {
        let binds: Vec<usize> = [0.0, 0.3536, 1.2].iter()
            .map(|&v| sm(d.clone(), "flow/press", CAP, 8, CapProfile::Derived, Split::DT,
                         v, 0.0, Some(1))
                    .stage_throat_margin(&flight(), t).lp.binds)
            .collect();
        assert!(binds[0] == 7 && binds[1] == 7,
                "at and below rung 55's schedule setting the REAR still binds: {binds:?}");
        assert_eq!(binds[2], 0, "far enough closed, the moved row binds ITSELF");
    }
}

// ==========================================================================================
// THE ONE GATE PYTHON DOES NOT HAVE — the latent guard
// ==========================================================================================

/// `_M_of_nu`'s range guard, which § 5.10 (iii) measured as **LATENT-ONLY**: the worst `nu^2` on
/// the whole slice-N grid is 2.7 % of the limit, and no shipped path can reach it because
/// `nu_1 < nu(M=1)` for any `C < 1` and the ladder only divides it DOWN.
///
/// Python has no gate for it at all. Rust ships one anyway — rung 54 P-C3's *gate the latent
/// defect, not just the exercised path* — because `gamma_th` is a free constructor argument, and
/// because the two languages fail DIFFERENTLY there: Python's `** 0.5` on a negative radicand
/// returns a COMPLEX number, Rust's `powf` returns `NaN`. Two different silent wrong answers is
/// exactly the case a `#[should_panic]` is for, and it is a value gate in neither language.
///
/// Booked in `rung55.rs::slice_n_deferrals` item 7 so the name diff is symmetric in BOTH
/// directions — a gate the port ADDS is as invisible to a one-way diff as one it drops.
#[test]
#[should_panic(expected = "total-referenced Mach out of range")]
fn test_total_referenced_mach_guard_is_latent_not_absent() {
    // nu(M=1) at gamma = 1.4 is 1/sqrt(1.2) ~ 0.913, and the limit is sqrt(2/0.4) ~ 2.236.
    // Nothing on the grid comes close; a hand-built profile can.
    let _ = mach_of_nu(2.5, 1.4);
}

// ==========================================================================================
// GATE 10 — CYCLE UNTOUCHED
// ==========================================================================================

/// The project's standing gate: rung 56 is reached through a separate entry point, so the
/// default single-spool design run must be bit-for-bit what it was.
#[test]
fn test_cycle_untouched_default_design_run_is_bit_for_bit_rung6() {
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1600.0, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, ..Losses::default()
    });
    let a = eng.run(&flight(), 1.0);
    let b = eng.run(&flight(), 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(),
               b.performance.specific_thrust.to_bits());
    for st in ["2", "3", "4", "5", "9"] {
        assert_eq!(a.station(st).tt.to_bits(), b.station(st).tt.to_bits());
        assert_eq!(a.station(st).pt.to_bits(), b.station(st).pt.to_bits());
    }
}
