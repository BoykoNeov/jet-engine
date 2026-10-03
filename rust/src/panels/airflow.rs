//! Rungs 53–56's panels — the airflow levers on the steady matcher (slice AP, first half).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line for
//! line. Python's keyword defaults are spelled positionally at each call, read off the Python
//! SIGNATURE (slice AO's rule): `currency_split`'s `dv = None`, `authority_ceiling`'s
//! `capacity = None`, `design_throat_mach`'s `gamma = 1.4`. A flat Python key that lives in a
//! nested Rust `Option` (rung 54's throat and choke reads) is unwrapped: Python would raise
//! `KeyError`/`TypeError` where it is absent, so the golden proves it present. Python bools printed
//! through `str()` go through `py_str()` — `format(True, '>8')` is `'       1'`, not `'True'`.

use super::twospool::{cpg13, ts_design};
use super::{Design, TT4};
use crate::map::ComponentMap;
use crate::pyf;
use crate::pyfmt::{Printer, PyFormat};
use crate::stage::{CapProfile, StageStackCore, StageStackCoreSpec};
use crate::stator::{Binds, VariableStatorCore};
use crate::two_spool::{Spool, TwoSpoolEngine};

/// `FLOOR = 0.55` — rung 36/41's imposed `phi_surge`, read as an incidence anchor.
const FLOOR: f64 = 0.55;

/// `LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)`.
pub(crate) fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::default() }
        .with_phi_surge(FLOOR)
}

/// `HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)`.
pub(crate) fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::default() }
        .with_phi_surge(FLOOR)
}

/// `build_two_spool_turbojet(cpg(), 3.0, 6.0, TT4, flight.p0, nozzle_convergent=True, **losses)`.
pub(crate) fn design13(d: &Design) -> TwoSpoolEngine {
    ts_design(cpg13(), 3.0, 6.0, d)
}

/// Python's `binds` string (`engine.py`'s `authority_ceiling`).
fn binds_str(b: Binds) -> &'static str {
    match b {
        Binds::Throat => "throat",
        Binds::Peak => "peak",
        Binds::Edge => "edge",
    }
}

/// `print_variable_stator_table(flight)` — rung 53.
pub fn variable_stator_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nThe VARIABLE STATOR (rung 53): the first lever that moves the SURGE FLOOR itself.");
    p.print("Two channels, ONE setting v = tan(alpha_1), both derived from the map's own l and");
    p.print("rung 36/41's own phi_surge:  psi -= v(1+l)phi   and   phi_surge(v) = phi_s0/(1+v phi_s0).");

    let (lp, hp) = (lp_map(), hp_map());
    let vs = VariableStatorCore::new(design13(d), *flight, 1.0, lp, hp, 0.0, 0.0);

    p.print("\n  THE SPLIT (LP stators, fixed throttle Tt4=1500). Both margins are reference-free");
    p.print("  and vanish TOGETHER; they move in OPPOSITE directions:");
    p.print(pyf!("  {:>7}{:>9}{:>10}{:>9}{:>9}{:>9}{:>9}{:>8}",
                 "v", "phi_op", "floor(v)", "M_phi", "tan_b1", "M_i", "SM_N", "n_LP"));
    for row in vs.stator_sweep(flight, TT4, &[-0.2, -0.1, 0.0, 0.1, 0.2, 0.3], Spool::Lp) {
        let q = row.lp;
        p.print(pyf!("  {:+7.2f}{:9.5f}{:10.5f}{:9.5f}{:9.5f}{:9.5f}{:9.5f}{:8.5f}",
                     row.vsv, q.phi_op, q.phi_surge, q.m_phi, q.tan_b1, q.m_i, q.sm_n, q.n));
    }
    let cs = vs.currency_split(flight, TT4, Spool::Lp, None);
    p.print(pyf!("  => dM_phi/dv = {:+.5f}  (closed form -(1+l)/(2+l)+phi_s0^2 = {:+.5f})",
                 cs.d_m_phi, -(1.0 + lp.l) / (2.0 + lp.l) + FLOOR * FLOOR));
    p.print(pyf!("     dM_i/dv   = {:+.5f}  (closed form 1/(2+l) = {:+.5f})   SPLIT = {}",
                 cs.d_m_i, 1.0 / (2.0 + lp.l), cs.split.py_str()));

    p.print("\n  WHY it is the FLOOR's fault, not a metric quirk. For any lever x,");
    p.print("    sign(dM_phi/dx) = sign(phi' + v' phi_surge^2)   sign(dM_i/dx) = sign(phi' + v' phi_op^2)");
    p.print("  so at v'=0 they differ only by the STRICTLY POSITIVE Jacobian 1/phi_op^2 and CANNOT");
    p.print("  split. The control -- throttle alone, stators at design (all three currencies):");
    p.print(pyf!("  {:>7}{:>11}{:>11}{:>11}{:>8}{:>9}{:>10}",
                 "Tt4", "dM_phi", "dM_i", "dSM_N", "agree", "ratio", "1/phi^2"));
    for r in vs.throttle_currency(flight, &[1500.0, 1300.0, 1100.0, 1000.0], Spool::Lp) {
        p.print(pyf!("  {:7.0f}{:+11.6f}{:+11.6f}{:+11.6f}{:>8}{:9.5f}{:10.5f}",
                     r.tt4, r.d_m_phi, r.d_m_i, r.d_sm_n, r.all_three_agree.py_str(), r.ratio,
                     r.jacobian));
    }
    p.print("  => a FLOOR-FIXED lever can never split them: rungs 36-52's phi-currency is BOUNDED,");
    p.print("     not refuted -- and now licensed by derivation rather than by assumption.");

    p.print("\n  THE PAYOFF the correct currency makes derivable: the schedule v*(Tt4) that holds");
    p.print("  the rotor incidence AT ITS DESIGN VALUE (what a real VSV schedule is FOR).");
    p.print(pyf!("  {:>7}{:>8}{:>9}{:>11}{:>9}{:>12}{:>9}{:>10}",
                 "Tt4", "v*", "phi_op", "M_i", "M_phi", "M_phi,bare", "SM_N", "N_L/N_Ld"));
    let rows = vs.incidence_schedule(flight, &[1500.0, 1300.0, 1100.0, 1000.0], Spool::Lp, 1.6);
    for r in &rows {
        p.print(pyf!("  {:7.0f}{:8.4f}{:9.5f}{:11.7f}{:9.5f}{:12.5f}{:9.5f}{:10.5f}",
                     r.tt4, r.vsv_star, r.phi_op, r.m_i, r.m_phi, r.m_phi_bare, r.sm_n, r.n));
    }
    // Python's `max(generator)`: the first value, replaced only on a strict `>`.
    let res_max = rows[1..].iter()
        .fold(rows[0].residual, |a, r| if r.residual > a { r.residual } else { a });
    let (first, last) = (&rows[0], &rows[rows.len() - 1]);
    p.print(pyf!("  => M_i EXACTLY constant (to {:.0e}) while M_phi", res_max.abs()));
    p.print(pyf!("     falls {:.0f}% -- BELOW its own unscheduled value at every point. One trajectory,",
                 (1.0 - last.m_phi / first.m_phi) * 100.0));
    p.print("     one boundary, three reference-free distances, three verdicts.");

    p.print("\n  HONEST SCOPE, two confidence levels. The CURRENCY result is coordinate algebra and");
    p.print("  rides on no magnitude. The SCHEDULE's numbers do not: holding design incidence at");
    p.print(pyf!("  Tt4=1000 costs N_L +{:.0f}% here, because this lumped one-stage map has neither a",
                 (last.n - 1.0) * 100.0));
    p.print("  stator-row FLOW CAPACITY nor a stage stack to rematch -- the channels that carry the");
    p.print("  real multistage benefit. That capacity channel needs a NEW constant: refused, and");
    p.print("  named as this rung's seam. Read v* as the swirl a cartoon needs, not a VSV schedule.");
}

/// `print_throat_capacity_table(flight)` — rung 54.
pub fn throat_capacity_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nThe STATOR-ROW THROAT (rung 54): the half rung 53 refused. The rotation that buys");
    p.print("incidence is the rotation that SPENDS THE THROAT -- one coordinate, now three");
    p.print("channels:  A_th(v)/A_th(0) = cos(alpha_1) = 1/sqrt(1+v^2)   [DERIVED, no constant].");

    let c = 0.90;
    let lp = lp_map().with_capacity(c);
    let hp = hp_map().with_capacity(c);
    let st = ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..ComponentMap::default() }
        .with_phi_surge(FLOOR).with_capacity(c);
    let design = design13(d);
    let vs = VariableStatorCore::new(design.clone(), *flight, 1.0, lp, hp, 0.0, 0.0);
    let steep = VariableStatorCore::new(design.clone(), *flight, 1.0, st, st, 0.0, 0.0);

    p.print(pyf!("\n  THE ONE CONSTANT, DISCLOSED: C = MFP(M_th0)/MFP(1) = {:.2f}, i.e. a design throat Mach",
                 c));
    p.print(pyf!("  M_th0 = {:.3f}. C=0 means no throat model, as phi_surge=0 means no surge line.",
                 lp.design_throat_mach(1.4)));
    p.print("  Everything load-bearing is reported as c_min = 1/X -- a threshold ON C, needing no C.");

    p.print("\n  THE THROAT COST is TWO-SIDED (cos is even) while the incidence benefit is ONE-sided.");
    p.print(pyf!("  {:>7}{:>9}{:>9}{:>10}{:>9}{:>9}{:>9}",
                 "v", "area", "m", "X=m/area", "c_min", "M_c", "M_i"));
    for row in vs.throat_sweep(flight, TT4, &[-0.4, -0.2, 0.0, 0.2, 0.4, 0.8], Spool::Lp) {
        let th = row.throat.expect("throat_sweep rows carry rung 54's extension");
        let ch = th.choke.expect("C > 0, so the choke keys are present");
        p.print(pyf!("  {:+7.2f}{:9.5f}{:9.5f}{:10.5f}{:9.5f}{:9.5f}{:9.5f}",
                     row.vsv, th.area, row.m, th.throat_loading, th.c_min, ch.m_c, row.m_i));
    }
    p.print("  => the throat is MAXIMAL at the design setting and closes whichever way the vane");
    p.print("     turns; only closing buys incidence. (That the peak sits AT design is inherited");
    p.print("     from rung 53's coordinate origin, not derived -- see the spec's Concessions.)");

    p.print("\n  THE HEADLINE: the throat cuts the SETTING hard and the MARGIN barely. Rung 53's");
    p.print("  own saturation is what makes the limit cheap -- the amputated tail was worth little.");
    p.print(pyf!("  {:>7}{:>9}{:>8}{:>7}{:>9}{:>10}{:>9}{:>7}{:>8}",
                 "Tt4", "v_edge", "v_ch", "cut%", "M_i(0)", "M_i_peak", "M_i_use", "kept%",
                 "binds"));
    for t in [1200.0, 1000.0, 800.0] {
        let a = vs.authority_ceiling(flight, t, Spool::Lp, None);
        p.print(pyf!("  {:7.0f}{:9.2f}{:8.3f}{:7.1f}{:9.5f}{:10.5f}{:9.5f}{:7.1f}{:>8}",
                     t, a.v_edge, a.v_ch.expect("the throat binds inside the scan here"),
                     a.setting_cut * 100.0, a.m_i_0, a.m_i_peak, a.m_i_usable,
                     a.retained * 100.0, binds_str(a.binds)));
    }
    p.print("  => rung 53 conceded its ceiling was solve_n's bracket, 'a map-validity edge' -- an");
    p.print("     ARTIFACT. Modelled, the throat beats it on every shape and throttle (20/20).");

    p.print("\n  BIND, NEVER RELIEVE (a theorem, not a measurement). v enters the solve through");
    p.print("  solve_n ALONE (rung 53 P1) and the throat enters NO solver, so X is a post-hoc read.");
    let bare = VariableStatorCore::new(design.clone(), *flight, 1.0, lp.with_capacity(0.0),
                                       hp.with_capacity(0.0), 1.1, 0.0);
    let chk = VariableStatorCore::new(design, *flight, 1.0, lp, hp, 1.1, 0.0);
    let r = chk.throat_margin(flight, TT4).lp;
    let (a1, a0) = (chk.core.match_point(flight, TT4), bare.core.match_point(flight, TT4));
    let m_c = r.throat.and_then(|t| t.choke).expect("C > 0, so the choke keys are present").m_c;
    p.print(pyf!("  At v=1.10, Tt4=1500 the row is CHOKED (M_c = {:+.5f}), and yet every matched", m_c));
    p.print(pyf!("  number is bit-identical to the no-throat run: thrust {}, n_LP {}, pi_LPC {}.",
                 (a1.base.thrust == a0.base.thrust).py_str(), (a1.n_lp == a0.n_lp).py_str(),
                 (a1.base.pi_lpc == a0.base.pi_lpc).py_str()));
    p.print("  => an upstream throat REMOVES SETTINGS FROM THE FEASIBLE SET; it cannot change the");
    p.print("     map from setting to incidence. Rung 53's seam expected capacity to buy back its");
    p.print("     +26% overspeed: REFUTED, structurally. The real mechanism is STAGE REMATCHING.");

    p.print("\n  THE RACE, and a CONSTANT-FREE boundary. As power falls the schedule's demand v*");
    p.print("  RISES while the flow m FALLS. c_min > 1 means the schedule asks LESS of the throat");
    p.print("  than the DESIGN point -- feasible for EVERY row, whatever its C.");
    p.print(pyf!("  {:>7}{:>8}{:>9}{:>9}{:>9}{:>10}", "Tt4", "v*", "m", "X(v*)", "c_min", "feasible"));
    for r in vs.schedule_throat(flight, &[1200.0, 1000.0, 900.0, 870.0, 860.0, 800.0], Spool::Lp) {
        let f = r.found.expect("the schedule exists on every row of this grid");
        let ch = f.choke.expect("C > 0, so the choke keys are present");
        p.print(pyf!("  {:7.0f}{:8.3f}{:9.5f}{:9.5f}{:9.5f}{:>10}",
                     r.tt4, f.vsv_star, f.m, f.throat_loading, f.c_min, ch.feasible.py_str()));
    }
    p.print("  => the crossing sits between Tt4 = 870 and 860: rung 53's ENTIRE published band");
    p.print("     (1000..1500) is above it, so the channel it refused is inert over all of it.");

    p.print("\n  AND A CORRECTION TO RUNG 53. Its Concessions say the incidence benefit 'does not");
    p.print("  turn back ... the apparent turning point is NOT reached'. True where it measured;");
    p.print("  false elsewhere -- on the steep shape the peak is INTERIOR and the schedule DIES:");
    for (tag, m) in [("flow/press", &vs), ("steep", &steep)] {
        let a = m.authority_ceiling(flight, 1000.0, Spool::Lp, None);
        let sch = m.schedule_throat(flight, &[1000.0], Spool::Lp)[0];
        let star = match sch.found {
            Some(f) if sch.exists => pyf!("{:.3f}", f.vsv_star),
            _ => "NONE".to_string(),
        };
        p.print(pyf!("    {:>11}: peak_interior={:>5}  v_peak={:.3f}  v_edge={:.2f}  M_i_peak-M_i_edge={:+.5f}  v*(1000)={}",
                     tag, a.peak_interior.py_str(), a.v_peak, a.v_edge, a.m_i_peak - a.m_i_edge,
                     star));
    }
    p.print("  => the rung-28 shape: rung 53's VERDICT (finite authority) survives, its REASON is");
    p.print("     corrected -- the ceiling is the incidence PEAK, not a map-validity edge, and it");
    p.print("     is reached INSIDE the envelope. It also defeats rung 53's monotone root ladder.");
}

/// `StageStackMatcher(design, flight, 1.0, map_lp=…, map_hp=…, K_lp=…, K_hp=…, …)`.
#[allow(clippy::too_many_arguments)]
fn stack(design: &TwoSpoolEngine, d: &Design, lp: ComponentMap, hp: ComponentMap, k_lp: usize,
         k_hp: usize, vsv_lp: f64, vs_lp: Option<usize>, prof: CapProfile) -> StageStackCore {
    StageStackCore::new(StageStackCoreSpec {
        vsv_lp, k_lp, k_hp, vsv_stages_lp: vs_lp, cap_profile: prof,
        ..StageStackCoreSpec::new(design.clone(), d.flight, 1.0, lp, hp)
    })
}

fn spool_upper(sp: Spool) -> &'static str {
    match sp {
        Spool::Lp => "LP",
        Spool::Hp => "HP",
    }
}

/// `print_stage_stack_table(flight)` — rung 55.
pub fn stage_stack_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nThe STAGE STACK (rung 55): the compressor stops being ONE block. With all annuli");
    p.print("sized so phi_k = 1 at design, the kinematics are DERIVED with no new constant:");
    p.print("   phi_k = phi_1 (theta_k/theta_k,d)/(varpi_k/varpi_k,d),  n_k = n sqrt(theta_k,d/theta_k)");
    p.print("and phi_1 = m/n EXACTLY -- the face phi every rung since 32 reads IS the FRONT stage's.");

    let (lp, hp) = (lp_map(), hp_map());
    let design = design13(d);
    let mk = |k_lp: usize, k_hp: usize, vs_lp: Option<usize>| {
        stack(&design, d, lp, hp, k_lp, k_hp, 0.0, vs_lp, CapProfile::Derived)
    };

    let m8 = mk(8, 8, None);

    p.print("\n  ONE MACHINE, TWO OPPOSITE FAILURES (K=8) -- the front stalls while the rear chokes.");
    p.print("  A lumped block has ONE phi and can represent neither end of it:");
    for tt4 in [1500.0, 800.0] {
        let r = m8.stage_margin(flight, tt4);
        for sp in [Spool::Lp, Spool::Hp] {
            let s = r.spool(sp);
            let phis: Vec<String> = s.stages.iter().map(|st| pyf!("{:.3f}", st.phi)).collect();
            p.print(pyf!("   Tt4={:.0f} {}  phi_k = ", tt4, spool_upper(sp)) + &phis.join(" ")
                    + &pyf!("   rear/front {:+5.1f}%", s.rear_excess * 100.0));
        }
    }
    p.print("  The LP FRONT stage is the worst incidence in the machine; the HP REAR runs ABOVE");
    p.print("  design phi -- toward choke and negative incidence.");

    p.print("\n  WHY THIS IS CONTENT, NOT A RE-READ (the non-tautology gate). The spread above is a");
    p.print("  functional of the (tau_c, pi_c) rung 39 already solves. The RUNG is the feedback:");
    p.print("  with a per-stage psi(phi_k) the work is no longer psi(phi_face)*n^2, so the stack");
    p.print("  MOVES the running line. Marched vs lumped work at the SAME (m, n):");
    p.print(pyf!("  {:>7}{:>10}{:>10}   (fraction of tau_c-1; EXACTLY 0 at K=1)",
                 "Tt4", "LP gap", "HP gap"));
    for tt4 in [1500.0, 1200.0, 1000.0, 800.0] {
        let g = m8.work_gap(flight, tt4);
        p.print(pyf!("  {:7.0f}{:9.2f}%{:9.2f}%", tt4, g.lp.gap_frac * 100.0, g.hp.gap_frac * 100.0));
    }

    p.print("\n  SO n RISES AND phi FALLS -- and rungs 36-53 are BOUNDED, not refuted: they read the");
    p.print("  right object (the front stage) but the lumped solve placed it OPTIMISTICALLY.");
    p.print(pyf!("  {:>7}{:>9}{:>10}{:>9}{:>10}{:>10}",
                 "Tt4", "d_n LP", "d_phi LP", "d_n HP", "d_phi HP", "d_thrust"));
    for r in m8.running_line_shift(flight, &[1500.0, 1200.0, 1000.0, 800.0]) {
        p.print(pyf!("  {:7.0f}{:8.3f}%{:9.3f}%{:8.3f}%{:9.3f}%{:9.3f}%",
                     r.tt4, r.lp.d_n * 100.0, r.lp.d_phi * 100.0, r.hp.d_n * 100.0,
                     r.hp.d_phi * 100.0, r.d_thrust * 100.0));
    }
    p.print("  Thrust barely moves: like rung 53's stator, the stack is paid in SHAFT SPEED.");

    p.print("\n  THE HEADLINE -- rung 54's seam discharged. Hold the FRONT stage's design incidence");
    p.print("  at Tt4=1000 with a FRONT-ROW-ONLY stator (what a real VSV is) instead of rung 53's");
    p.print("  whole-machine lever. The cost FACTORISES into a positional leg and a setting leg:");
    let t = 1000.0;
    let r53 = VariableStatorCore::new(design.clone(), *flight, 1.0, lp, hp, 0.0, 0.0);
    let row53 = r53.incidence_schedule(flight, &[t], Spool::Lp, 4.0)[0];
    let b53 = r53.at_setting(0.0, 0.0).core.match_point(flight, t);
    let s53 = r53.at_setting(row53.vsv_star, 0.0).core.match_point(flight, t);
    let dn53 = (s53.n_lp_ratio - b53.n_lp_ratio) / b53.n_lp_ratio;
    p.print(pyf!("   rung 53 (one lumped block, EVERY row moves): v* = {:.4f}, dN_L = {:+.2f}%",
                 row53.vsv_star, dn53 * 100.0));
    p.print(pyf!("  {:>4}{:>10}{:>9}{:>9}{:>9}{:>8}{:>13}",
                 "K", "v*front", "dN_L", "ratio", "v*ratio", "1/K", "(v*ratio)/K"));
    for k in [2usize, 4, 8, 16] {
        let m = mk(k, 8, Some(1));
        let row = m.stage_incidence_schedule(flight, &[t], Spool::Lp, 0, 4.0)[0];
        let (bare, sib) = (m.at_setting(0.0, 0.0), m.at_setting(row.vsv_star, 0.0));
        let (b, s) = (bare.match_point(flight, t), sib.match_point(flight, t));
        let dn = (s.n_lp_ratio - b.n_lp_ratio) / b.n_lp_ratio;
        let vr = row.vsv_star / row53.vsv_star;
        let kf = k as f64;
        p.print(pyf!("  {:4d}{:10.4f}{:8.2f}%{:9.4f}{:9.4f}{:8.4f}{:13.4f}",
                     k, row.vsv_star, dn * 100.0, dn / dn53, vr, 1.0 / kf, vr / kf));
    }
    p.print("  The product law holds to 3 % over an 8x range in K. The 1/K leg is positional; the");
    p.print("  SETTING leg is that a front-only lever does not FIGHT ITS OWN SPEED RISE.");

    p.print("\n  AND THE HONEST HALF -- the same law read backwards. Shaft speed is the one thing");
    p.print("  every stage shares, so relief taken at the front is PAID BY THE ROWS LEFT BEHIND.");
    p.print("  Sweep how many of the 8 rows the stator moves (target unchanged):");
    let m0 = mk(8, 8, Some(1));
    let bare0 = m0.at_setting(0.0, 0.0);
    let b0 = bare0.match_point(flight, t);
    let mi0 = bare0.stage_margin(flight, t).lp.m_i_worst;
    p.print(pyf!("  {:>6}{:>9}{:>9}{:>7}{:>11}{:>9}{:>9}",
                 "rows", "v*", "dN_L", "worst", "M_i worst", "relief", "per dN"));
    p.print(pyf!("  {:>6}{:>9}{:>9}{:7d}{:11.4f}{:>9}{:>9}", "0", "--", "--", 0usize, mi0, "--",
                 "--"));
    for rows in [1usize, 2, 3, 4, 5, 6] {
        // Python's `m._V_SCAN = 0.01` — on THIS matcher; its siblings keep the class default.
        let m = mk(8, 8, Some(rows)).with_v_scan(0.01);
        let row = m.stage_incidence_schedule(flight, &[t], Spool::Lp, 0, 4.0)[0];
        let sib = m.at_setting(row.vsv_star, 0.0);
        let sm = sib.stage_margin(flight, t).lp;
        let dn = (sib.match_point(flight, t).n_lp_ratio - b0.n_lp_ratio) / b0.n_lp_ratio;
        let rel = (sm.m_i_worst - mi0) / mi0;
        p.print(pyf!("  {:6d}{:9.4f}{:8.2f}%{:7d}{:11.4f}{:8.2f}%{:9.2f}",
                     rows, row.vsv_star, dn * 100.0, sm.worst, sm.m_i_worst, rel * 100.0,
                     rel / dn));
    }
    p.print("  THE OPTIMUM IS A COUNT -- and it is INTERIOR: relief peaks at 3-4 rows and then");
    p.print("  REVERSES, ending worse than bare, as the worst stage migrates into the rows the");
    p.print("  stator does NOT move. Two currencies, two optima: most relief at 4 rows, most");
    p.print("  relief-per-speed at 1. Rung 53's coordinate law, a third time.");

    p.print("\n  SCOPE: steady and two-spool. The stack ENTERS THE SOLVER, so unlike rung 54's");
    p.print("  throat there is no free invariance -- the transient ladders (34/40/43 and the whole");
    p.print("  limiter family) stay on the lumped loading law, and a test asserts it.");
    p.print("  REDUCE: an IDENTITY at K = 1 -- no stack object is built and both efficiency loops");
    p.print("  are the INHERITED rung-39 ones. Measured 0.000e+00 on every matched field.");
}

/// `print_per_row_capacity_table(flight)` — rung 56.
pub fn per_row_capacity_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nPER-ROW CAPACITY (rung 56): rung 54's throat, per stage. The seam asked for a C per");
    p.print("row; the ladder supplies all but one. At design phi_k = 1 => Vx_k = U, so every row");
    p.print("has the SAME throat velocity while Tt_k rises, and it is the total-referenced Mach");
    p.print("   nu = M/sqrt(1+(g-1)/2 M^2)   that scales:  nu_k = nu_1/sqrt(theta_k,d)");
    p.print("=> ONE disclosed LEVEL (the front row's C, rung 54's constant) and K-1 rows DERIVED.");

    let cap = 0.90;
    let lp = lp_map().with_capacity(cap);
    let hp = hp_map().with_capacity(cap);
    let design = design13(d);
    let mk = |prof: CapProfile, vl: f64, vs_lp: Option<usize>| {
        stack(&design, d, lp, hp, 8, 8, vl, vs_lp, prof)
    };

    let m8 = mk(CapProfile::Derived, 0.0, None);
    p.print(pyf!("\n  the DERIVED profile (C_front = {:.2f} <=> design throat Mach {:.3f}):",
                 cap, lp.design_throat_mach(1.4)));
    for (sp, s) in [("LP", Spool::Lp), ("HP", Spool::Hp)] {
        let cs = m8.stack_of(s).expect("K = 8 builds both stacks").capacities();
        let txt: Vec<String> = cs.iter().map(|c| pyf!("{:.3f}", c)).collect();
        p.print(pyf!("   {}  C_k = ", sp) + &txt.join(" ")
                + &pyf!("   C_K/C_1 = {:.4f}   (the HP falls harder: bigger tau_d)",
                        cs[cs.len() - 1] / cs[0]));
    }

    p.print("\n  THE CONTEST -- the rear rows are DESIGNED with more capacity exactly where the");
    p.print("  off-design march loads them hardest, so which end binds MIGRATES with throttle:");
    let grid = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 800.0];
    let head: Vec<String> = grid.iter().map(|t| pyf!("{:5.0f}", t)).collect();
    p.print("     Tt4:      ".to_string() + &head.join(" "));
    for (name, prof) in [("derived", CapProfile::Derived), ("uniform", CapProfile::Uniform)] {
        let mm = mk(prof, 0.0, None);
        for sp in [Spool::Lp, Spool::Hp] {
            let w = mm.throat_walk(flight, &grid, sp);
            let cols: Vec<String> = w.iter().map(|r| pyf!("{:5d}", r.binds)).collect();
            p.print(pyf!("   {:7s} {} binds row ", name, spool_upper(sp)) + &cols.join(" "));
        }
    }
    p.print("  Strip the derived profile ('uniform') and the contest disappears: X_k alone decides");
    p.print("  and the rear binds everywhere. The migration is the machine's own design Mach law.");

    p.print("\n  THE HEADLINE -- the machine's two BINDING rows are different rows: different END");
    p.print("  and different SPOOL. Minima taken over all 16 rows:");
    for tt4 in [1000.0, 800.0] {
        let r = m8.stage_throat_margin(flight, tt4);
        let (l, h) = (&r.lp, &r.hp);
        p.print(pyf!("   Tt4={:.0f}  INCIDENCE worst: LP row {} (M_i={:.4f}, vs HP's {:.4f})  |  CAPACITY worst: HP row {} (M_c={:.4f}, vs LP's {:.4f})",
                     tt4, l.inc_worst, l.m_i_worst, h.m_i_worst, h.binds, h.m_c_worst,
                     l.m_c_worst));
    }
    p.print("  Per spool the ends agree (each front stalls first, each rear chokes first); the");
    p.print("  CROSS-SPOOL half is the comparison above -- rungs 41/44/45/53 put the surge");
    p.print("  exposure on the LP, and the CAPACITY exposure is the HP's. A lumped block has one");
    p.print("  phi and one face and cannot express either half, let alone their separation.");

    p.print("\n  RUNG 54 CORRECTED BY RESOLUTION -- it wrote 'the HP never approaches its throat at");
    p.print("  any throttle'. True at the FACE -- and down the columns the face RELAXES with");
    p.print("  throttle while the rear row TIGHTENS. Opposite signs on the same machine:");
    p.print("     Tt4      face M_c ->    rear-row M_c ->      C*      [uniform rear]");
    for tt4 in [1200.0, 1000.0, 800.0] {
        let r = m8.stage_throat_margin(flight, tt4).hp;
        let u = mk(CapProfile::Uniform, 0.0, None).stage_throat_margin(flight, tt4).hp;
        let (rr, ur) = (&r.stages[r.stages.len() - 1], &u.stages[u.stages.len() - 1]);
        p.print(pyf!("    {:6.0f}      {:+.4f}          {:+.4f}          {:.4f}      {:+.4f}",
                     tt4, r.m_c_face, rr.m_c, rr.c_min, ur.m_c));
    }
    p.print("  Stated as a THRESHOLD ON the constant (rung 54's discipline): any HP row whose");
    p.print("  design capacity fraction exceeds 0.913 is CHOKED at Tt4 = 800. A machine pays for");
    p.print("  its rear rows' off-design capacity AT DESIGN, in the front row's Mach.");

    p.print("\n  THE LEVER DEBITS THE ROW IT DOES NOT MOVE (front-row stator, LP, Tt4 = 1000) --");
    p.print("  it reaches the rear only through the shaft speed they share:");
    let base = m8.stage_throat_margin(flight, 1000.0).lp;
    let rear = |s: &crate::stage::SpoolThroatMargin| s.stages[s.stages.len() - 1].m_c;
    p.print("     v      rear M_c    debit    front-only/lumped   dN ratio");
    for v in [0.20, 0.3536, 0.60] {
        let fr = mk(CapProfile::Derived, v, Some(1)).stage_throat_margin(flight, 1000.0).lp;
        let lu = mk(CapProfile::Derived, v, None).stage_throat_margin(flight, 1000.0).lp;
        let d_fr = rear(&base) - rear(&fr);
        let d_lu = rear(&base) - rear(&lu);
        p.print(pyf!("   {:6.4f}   {:+.5f}   {:+.5f}        {:.4f}         {:.4f}",
                     v, rear(&fr), d_fr, d_fr / d_lu,
                     ((fr.n - base.n) / base.n) / ((lu.n - base.n) / base.n)));
    }
    p.print("  The SPEED ratio is nearly v-invariant; the THROAT ratio COLLAPSES. So a positional");
    p.print("  lever's advantage is CURRENCY-DEPENDENT -- rung 53's law a fourth time, now about");
    p.print("  the LEVER'S COST. (Pre-registered 'within 25 % of the dN ratio': REFUTED.)");

    p.print("\n  SCOPE: DIAGNOSTIC ONLY, by rung 54's theorem -- the throat enters no solver, so a");
    p.print("  choked row changes nothing that is solved (making it BIND would invert rung 31's");
    p.print("  (*) and is a different rung). REDUCE: an INVARIANCE over the constant AND the");
    p.print("  profile on a stack that DOES enter the solver, plus K=1 == rung 54 bit-for-bit.");
}
