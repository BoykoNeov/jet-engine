//! Rungs 31–37's panels — single-spool off-design matching and the transient (slice AM).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line for
//! line. Where Python shares one `eng` between several matchers, the Rust clones it: the matchers
//! only read their design engine, and the gas here is the stateless thermally-perfect one.

use super::marches::conv_losses;
use super::mixing::py_min;
use super::nox::{dashes, py_max};
use super::{Design, PI_C, TT4};
use crate::combustor::{CombustorTransient, Theta0};
use crate::engine::build_turbojet;
use crate::gas::{Gas, GasSpec};
use crate::map::{ComponentMap, MapMatcher};
use crate::matcher::{OffDesignMatcher, OffDesignResult};
use crate::spool::SpoolTransient;
use crate::pyf;
use crate::pyfmt::Printer;

/// `build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, flight.p0, nozzle_convergent=True,
/// **REAL_LOSSES)` wrapped in rung 31's matcher.
fn eq_matcher(d: &Design) -> OffDesignMatcher {
    OffDesignMatcher::new(build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, d.flight.p0, conv_losses()),
                          d.flight, 1.0)
}

/// `Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=…, cp_t=…, R_t=…, hPR=42.8e6)`.
pub(crate) fn cpg(gamma_t: f64, cp_t: f64, r_t: f64) -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t, cp_t, r_t, hpr: 42.8e6,
                       ..GasSpec::default() })
}

/// `print_offdesign_table(flight)` — rung 31.
pub fn offdesign_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nOff-design matching (rung 31): with the turbine NGV + convergent nozzle both CHOKED,");
    p.print("the compressor has NO freedom — pi_c is an OUTPUT slaved to Tt4/(tau_r·T0), not a knob.");

    let m = eq_matcher(d);
    let od0 = m.match_point(flight, TT4);
    p.print(pyf!("\n  Reduce-to-design: matching at the design point returns pi_c = {:.6f} (input 10), mdot/mdot_R = {:.6f} — the spine holds by construction.",
                 od0.pi_c, od0.mdot_ratio));

    p.print(pyf!("\n  Running line — throttle sweep (M0={}); pi_c is the OUTPUT:", flight.m0));
    p.print(pyf!("  {:>7} {:>7} {:>9} {:>11} {:>8} {:>8} {:>7}",
                 "Tt4 [K]", "pi_c", "tau_t", "mdot/mdotR", "F/mdot", "thrust", "nozzle"));
    p.print(format!("  {}", dashes(62)));
    for tt4 in [1500.0, 1300.0, 1100.0, 900.0, 700.0, 600.0] {
        let od = m.match_point(flight, tt4);
        let tag = if od.nozzle_choked { "choked" } else { "UNCHOKE" };
        p.print(pyf!("  {:>7.0f} {:>7.3f} {:>9.6f} {:>11.4f} {:>8.1f} {:>8.1f} {:>7}",
                     tt4, od.pi_c, od.tau_t, od.mdot_ratio, od.performance.specific_thrust, od.thrust, tag));
    }

    let drift = |matcher: &OffDesignMatcher| -> f64 {
        let (h, c) = (matcher.match_point(flight, 1500.0).tau_t, matcher.match_point(flight, 800.0).tau_t);
        100.0 * (h - c) / h
    };
    let gas_cpg = cpg(1.3, (1.3 - 1.0) / 1.3 * 1239.0 * 1.3 / (1.3 - 1.0), (1.3 - 1.0) / 1.3 * 1239.0);
    let d_cpg = drift(&OffDesignMatcher::new(build_turbojet(gas_cpg, PI_C, TT4, flight.p0, conv_losses()), *flight, 1.0));
    let d_tpg = drift(&OffDesignMatcher::new(
        build_turbojet(Gas::thermally_perfect(), PI_C, TT4, flight.p0, conv_losses()), *flight, 1.0));
    let d_react = drift(&m);

    p.print("\n  THE VERDICT — the choked hardware strips the compressor of freedom: pi_c and mdot");
    p.print("  ride one fixed running line (a pumping characteristic WITHOUT a compressor map).");
    p.print("  THE FINDING — the textbook says tau_t is EXACTLY constant, but that is a CPG statement.");
    p.print(pyf!("  On the real gas tau_t DRIFTS {:.1f}% across the choked throttle range (1500→800 K).", d_react));
    p.print(pyf!("  Kill-test (3-gas ladder, drift over the SAME range): CPG {:+.3f}%  |  variable-cp", d_cpg));
    p.print(pyf!("  frozen-composition {:+.2f}%  |  reacting {:+.2f}%. So the gamma_t(T) CURVE", d_tpg, d_react));
    p.print(pyf!("  drives {:.0f}% of it (R cancels between the two throats), composition the rest —", 100.0 * d_tpg / d_react));
    p.print("  same species as rung 30's '0.03% is the physics, not error'. Below Tt4≈600 the nozzle");
    p.print("  UNCHOKES (pt9/p0 < ~1.85) and this two-choke pin is lost — the SUBSONIC-nozzle matching");
    p.print("  branch (rung 33) takes over there; rung 32 earns the eta curvature the running line holds.");
}

/// `print_component_map_table(flight)` — rung 32.
pub fn component_map_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nComponent-map matching (rung 32): rung 31 said the running line needs NO compressor map.");
    p.print("That over-claimed — it held eta at design. A real map droops eta_c off-design, so pi_c and");
    p.print("mdot fall BELOW rung 31's line. The choke pins the WORK tau_c (map-free); the map moves pi_c.");

    let mm = MapMatcher::new(build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, flight.p0, conv_losses()),
                             *flight, 1.0, ComponentMap::flat());
    let base = eq_matcher(d);

    let flat = ComponentMap::flat();
    let r0 = mm.match_with(flight, TT4, &flat);
    p.print(pyf!("\n  Reduce-to-rung-31: flat map at design returns pi_c = {:.6f}, N/N_d = {:.6f} — rung 31 bit-for-bit (the spine).",
                 r0.base.pi_c, r0.n_ratio));

    let cmap = ComponentMap::flow_dominated();
    p.print("\n  Running line with a peaked compressor map (flow-dominated shape); pi_c is the OUTPUT:");
    p.print(pyf!("  {:>6} {:>9} {:>9} {:>7} {:>7} {:>7} {:>9} {:>6}",
                 "Tt4", "pi_c(r31)", "pi_c(map)", "dpi_c", "dmdot", "eta_c", "tau_c rel", "N/Nd"));
    p.print(format!("  {}", dashes(66)));
    for tt4 in [1500.0, 1300.0, 1100.0, 900.0] {
        let mo = mm.match_with(flight, tt4, &cmap);
        let ro = base.match_point(flight, tt4);
        let dpc = 100.0 * (mo.base.pi_c - ro.pi_c) / ro.pi_c;
        let dmd = 100.0 * (mo.base.mdot_air - ro.mdot_air) / ro.mdot_air;
        let tcr = (mo.base.tau_c - ro.tau_c).abs() / ro.tau_c;
        p.print(pyf!("  {:>6.0f} {:>9.4f} {:>9.4f} {:>6.2f}% {:>6.2f}% {:>7.4f} {:>9.1e} {:>6.3f}",
                     tt4, ro.pi_c, mo.base.pi_c, dpc, dmd, mo.eta_c, tcr, mo.n_ratio));
    }

    let mut droops = Vec::new();
    for shape in [ComponentMap::flow_dominated(), ComponentMap::pressure_dominated(), ComponentMap::tilted()] {
        let mo = mm.match_with(flight, 900.0, &shape);
        let ro = base.match_point(flight, 900.0);
        droops.push(100.0 * (mo.base.pi_c - ro.pi_c) / ro.pi_c);
    }
    let steep = mm.match_with(flight, 900.0,
                              &ComponentMap { a: 0.25, b: 0.05, sigma: 0.3, a_t: 0.5, ..ComponentMap::default() });

    p.print("\n  THE FINDING — rung 31's 'without a map' over-claimed. The work tau_c is choke-pinned");
    p.print("  (map-free to ~1e-6), but the map droops eta_c so pi_c/mdot fall below rung 31's line —");
    p.print(pyf!("  SAME SIGN across 3 map shapes at Tt4=900: dpi_c = {:.1f}% / {:.1f}% / {:.1f}% (magnitude shape-dependent, DISCLAIMED).",
                 droops[0], droops[1], droops[2]));
    p.print(pyf!("  SUB-FINDING — the turbine barely moves: its corrected speed nu_t = {:.4f} stays", steep.nu_t));
    p.print(pyf!("  ~1% from design (single-spool N/sqrt(Tt4)), so |d eta_t| = {:.1e} even for a", (steep.eta_t - 0.90).abs()));
    p.print("  STEEP turbine map — the compressor is where the map bites. (No surge line modeled.)");
}

/// `print_subsonic_matching_table(flight)` — rung 33.
pub fn subsonic_matching_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nSubsonic-nozzle matching (rung 33): below Tt4~600 the nozzle UNCHOKES and rung 31's two-choke");
    p.print("pin (star) is void — only the NGV chokes. pi_t becomes the unknown that matches the NGV-choked");
    p.print("supply to the subsonic-nozzle demand MFP(M9). The clean rung-31 decoupling BREAKS.");

    let m = eq_matcher(d);
    p.print(pyf!("\n  Running line across the boundary (M0={}); branch is auto-dispatched:", flight.m0));
    p.print(pyf!("  {:>7} {:>9} {:>7} {:>9} {:>7} {:>8} {:>7}", "Tt4 [K]", "branch", "pi_c", "tau_t", "M9", "F/mdot", "pt9/p0"));
    p.print(format!("  {}", dashes(60)));
    for tt4 in [700.0, 600.0, 560.0, 520.0, 480.0, 440.0, 420.0] {
        // Python's `try: … except AssertionError` — the sub-idle `assert!` unwinds here, as in
        // `anti_windup::try_windup_march` (the panic hook is left alone: stderr noise only).
        let od: Option<OffDesignResult> =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| m.match_point(flight, tt4))).ok();
        match od {
            Some(od) => p.print(pyf!("  {:>7.0f} {:>9} {:>7.3f} {:>9.6f} {:>7.4f} {:>8.1f} {:>7.3f}",
                                     tt4, od.branch.label(), od.pi_c, od.tau_t, od.m9,
                                     od.performance.specific_thrust, od.station("9").pt / flight.p0)),
            None => p.print(pyf!("  {:>7.0f} {:>9}  (net thrust <= 0: below thrust-neutral idle)", tt4, "SUB-IDLE")),
        }
    }

    let (g, cp) = (1.3, 1239.0);
    let gas_cpg = cpg(g, cp, (g - 1.0) / g * cp);
    let mc = OffDesignMatcher::new(build_turbojet(gas_cpg, PI_C, TT4, flight.p0, conv_losses()), *flight, 1.0);
    let ch: Vec<f64> = [1200.0, 800.0].iter().map(|&t| mc.match_point(flight, t).tau_t).collect();
    let sub: Vec<f64> = [580.0, 540.0, 500.0, 460.0, 440.0].iter().map(|&t| mc.match_point(flight, t).tau_t).collect();
    let (smax, smin) = (py_max(sub.iter().copied()), py_min(&sub));
    p.print("\n  THE RUNG — rung 31: 'the turbine does not know the operating condition changed' holds");
    p.print("  ONLY while both throats choke. On a CPG gas the CHOKED tau_t is constant to machine zero");
    p.print(pyf!("  (spread {:.1e}, rung 31 gate 2), but the SUBSONIC tau_t VARIES {:.2f}%",
                 (ch[0] - ch[1]).abs(), 100.0 * (smax - smin) / smax));
    p.print("  — the coupling runs through pi_c (STRUCTURAL, 1st order), not the gamma_t(T) curve that");
    p.print("  drove rung 31's 2nd-order drift. So it SURVIVES CPG: the exact inversion of rung 31.");
    p.print("  (Coupling is to pi_c via pt9/p0, NOT ambient p0 — the cycle is pressure-homogeneous.)");
    p.print("  Envelope: bounded ABOVE by nozzle-unchoke, BELOW by thrust-neutral idle. Cycle: rung-6 exact.");
}

/// `SpoolTransient(build_turbojet(Gas.thermally_perfect(), PI_C, TT4, flight.p0,
/// nozzle_convergent=True, **REAL_LOSSES), flight, 1.0, comp_map=ComponentMap.surge_flow())`.
fn tpg_spool(d: &Design) -> SpoolTransient {
    SpoolTransient::new(build_turbojet(Gas::thermally_perfect(), PI_C, TT4, d.flight.p0, conv_losses()),
                        d.flight, 1.0, ComponentMap::surge_flow())
}

/// `print_spool_transient_table(flight)` — rung 34.
pub fn spool_transient_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nSpool transient (rung 34): the shaft gets INERTIA, so N becomes a STATE that N-lags a fuel");
    p.print("change. The compressor map runs FORWARD + NGV choke closes the flow with NO shaft balance;");
    p.print("the leftover power drives dN/ds. Equilibrium (dN/ds=0) reduces to the rung 31/32 running line.");

    let shape = ComponentMap::surge_flow();
    let st = tpg_spool(d);
    // Python also builds an `OffDesignMatcher` `base` here and never reads it — not ported.

    let eq = st.equilibrium(flight, TT4, Some(&ComponentMap::flat()));
    p.print(pyf!("\n  Reduce-to-rung-31: flat-map equilibrium at design returns pi_c = {:.6f}, N/N_d = {:.6f} — the running line is the transient's stable attractor.",
                 eq.pi_c, eq.nu));

    let e0 = st.constant_speed_excursion(flight, 1100.0, 1400.0, Some(&shape));
    p.print("\n  THE FINDING — acceleration Tt4 1100->1400, peak excursion ABOVE the running line");
    p.print("  (toward lower surge margin) vs the fuel/spool time ratio r = tau_fuel/tau_spool:");
    p.print(pyf!("  {:>22} {:>15} {:>7}", "r = tau_fuel/tau_spool", "peak excursion", "E/E0"));
    p.print(format!("  {}", dashes(48)));
    p.print(pyf!("  {:>22} {:>13.2f}% {:>7.3f}", "0 (algebraic limit)", e0 * 100.0, 1.000));
    let mut e_r5 = f64::NAN;
    for r in [0.2, 0.5, 1.0, 2.0, 5.0] {
        let e = st.ramp_excursion(flight, 1100.0, 1400.0, r, Some(&shape), 5.0, 0.1).e;
        p.print(pyf!("  {:>22.1f} {:>13.2f}% {:>7.3f}", r, e * 100.0, e / e0));
        e_r5 = e;
    }

    let sched = |s: f64| -> f64 {
        if s <= 0.0 { 900.0 } else if s >= 6.0 { 460.0 } else { 900.0 - (900.0 - 460.0) * (s / 6.0) }
    };
    let nu0 = st.equilibrium(flight, 900.0, Some(&shape)).nu;
    let traj = st.integrate(flight, sched, nu0, 21.0, 0.1, Some(&shape));
    let flip = (1..traj.len()).find(|&i| traj[i].branch != traj[i - 1].branch).map(|i| &traj[i])
        .expect("the spool-down never unchokes");
    let last = &traj[traj.len() - 1];

    p.print("\n  Direction is shape-robust (accel + / decel - across 3 surge-realistic maps); the r->0");
    p.print("  step excursion is a MAP property (E/E0->1), the DYNAMICAL content is the ratio — a slow");
    // Python re-runs the r=5 ramp here; the same deterministic call, so the loop's value is reused.
    p.print(pyf!("  ramp (r=5) nearly stays on the line (E/E0={:.2f}). tau_spool=I·w_d^2/P_ref is the ONE disclaimed clock.",
                 e_r5 / e0));
    p.print(pyf!("  SPOOL-DOWN — fuel cut 900->460: N coasts {:.3f} -> {:.3f}, and at s={:.1f} the", nu0, last.nu, flip.s));
    p.print(pyf!("  nozzle UNCHOKES (M9={:.3f}), flipping onto rung 33's SUBSONIC branch toward thrust-neutral", flip.m9));
    p.print(pyf!("  idle (sp. thrust {:.0f} -> {:.0f} N·s/kg). Cycle: rung-6 exact.", traj[0].sp_thrust, last.sp_thrust));
}

/// `print_fuel_metering_table(flight)` — rung 35.
pub fn fuel_metering_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nFuel metering (rung 35): rung 34 commanded Tt4; a real engine meters FUEL and Tt4 is an");
    p.print("OUTPUT. At a frozen spool a fuel step starves the airflow, so f = mdot_fuel/mdot_air SPIKES");
    p.print("and Tt4 OVERSHOOTS — a second acceleration limit (turbine life) that commanding Tt4 hid.");

    let shape = ComponentMap::surge_flow();
    let st = tpg_spool(d);
    let (lo, hi) = (1100.0, 1400.0);

    let eq_t = st.equilibrium(flight, hi, Some(&shape));
    let eq_f = st.equilibrium_fuel(flight, eq_t.f * eq_t.mdot_air, Some(&shape));
    p.print(pyf!("\n  Reduce (control-invariance): fuel = f_eq*mdot_air of the Tt4={:.0f} point returns the", hi));
    p.print(pyf!("  SAME running-line instant (nu {:.6f} vs {:.6f}, pi_c {:.4f} vs {:.4f}, Tt4_out={:.3f}) — a different closure onto one point.",
                 eq_t.nu, eq_f.nu, eq_t.pi_c, eq_f.pi_c, eq_f.tt4));

    let (e_surge0, e_temp0, tt4_peak, _) = st.constant_speed_excursion_fuel(flight, lo, hi, Some(&shape));
    let e0t = st.constant_speed_excursion(flight, lo, hi, Some(&shape));
    p.print(pyf!("\n  THE FINDING — acceleration Tt4 {:.0f}->{:.0f}, excursions vs r = tau_fuel/tau_spool.", lo, hi));
    p.print("  E_surge/E_temp are referenced to the running line at the CURRENT speed (E_temp is the");
    p.print("  E_surge analogue); Tt4_pk is the ABSOLUTE peak turbine-inlet temperature (a redline is absolute):");
    p.print(pyf!("  {:>6} {:>13} {:>13} {:>7} {:>8} {:>11}", "r", "E_surge Tt4", "E_surge fuel", "gap", "E_temp", "Tt4_pk (K)"));
    p.print(format!("  {}", dashes(62)));
    p.print(pyf!("  {:>6} {:>12.2f}% {:>12.2f}% {:>6.2f}% {:>7.1f}% {:>11.0f}",
                 "0*", e0t * 100.0, e_surge0 * 100.0, (e_surge0 - e0t) * 100.0, e_temp0 * 100.0, tt4_peak));
    for r in [0.3, 1.0, 3.0] {
        let e_t = st.ramp_excursion(flight, lo, hi, r, Some(&shape), 4.0, 0.1).e;
        let ef = st.ramp_excursion_fuel(flight, lo, hi, r, Some(&shape), 4.0, 0.1);
        p.print(pyf!("  {:>6.1f} {:>12.2f}% {:>12.2f}% {:>6.2f}% {:>7.1f}% {:>11.0f}",
                     r, e_t * 100.0, ef.e_surge * 100.0, (ef.e_surge - e_t) * 100.0, ef.e_temp * 100.0, ef.tt4_peak));
    }
    p.print(pyf!("  (* r->0 algebraic limit, no integration.) The r->0 peak Tt4={:.0f} K is +{:.0f}% OVER the {:.0f} K target — a TIT excursion commanding",
                 tt4_peak, (tt4_peak / hi - 1.0) * 100.0, hi));
    p.print("  Tt4 hid. Fuel control also lifts the surge excursion ABOVE rung 34's: the two limits are");
    p.print("  COUPLED. Magnitude claim rests on the r->0 STEP (both are steps, unconfounded); the r->inf");
    p.print("  vanishing is the trend. Sign shape-robust across surge maps; magnitude disclaimed. Cycle: rung-6 exact.");
}

/// `print_surge_line_table(flight)` — rung 36.
pub fn surge_line_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nSurge line (rung 36): rungs 32/34/35 measured the excursion ABOVE the running line but drew");
    p.print("NO surge line. Rung 36 imposes a stall flow coeff phi_surge (disclaimed) and finds the SIGN");
    p.print("that survives it: surge margin is THIN AT LOW POWER, so the binding accel is the low-power burst.");

    let shape = ComponentMap::surge_flow();
    let st = tpg_spool(d);
    let phi_s = 0.65;
    let cm = shape.with_phi_surge(phi_s);

    p.print(pyf!("\n  THE SCHEDULE (phi_surge={}, DISCLAIMED level; the falling SIGN is the claim):", phi_s));
    p.print(pyf!("  {:>5} {:>8} {:>7} {:>8} {:>9}", "Tt4", "phi_op", "pi_c", "SM_N", "SM_flow"));
    p.print(format!("  {}", dashes(40)));
    for sm in st.surge_margin_schedule(flight, &[1500.0, 1300.0, 1100.0, 900.0, 800.0, 700.0], Some(&cm)) {
        p.print(pyf!("  {:>5.0f} {:>8.4f} {:>7.3f} {:>7.1f}% {:>8.0f}%",
                     sm.tt4, sm.phi_op, sm.pi_c, sm.sm_n * 100.0, sm.sm_flow * 100.0));
    }

    p.print("\n  THE COMPOUNDING (confirm+sharpen) - full-throttle burst to Tt4=1500. E0 = rung-34 constant-N excursion;");
    p.print("  SM_N = steady margin at the START. Surge iff E0>=SM_N (== phi_step<=phi_surge, airtight):");
    p.print(pyf!("  {:>6} {:>7} {:>7} {:>8} {:>8}", "Tt4_lo", "E0", "SM_N", "E0/SM_N", "verdict"));
    p.print(format!("  {}", dashes(40)));
    for lo in [1400.0, 1200.0, 1000.0, 900.0, 800.0, 700.0] {
        let b = st.acceleration_binding(flight, lo, 1500.0, Some(&cm));
        assert_eq!(b.reaches_surge, b.phi_step_le_surge, "currency equivalence, live");
        p.print(pyf!("  {:>6.0f} {:>6.1f}% {:>6.1f}% {:>8.3f} {:>8}",
                     lo, b.e0 * 100.0, b.sm_n * 100.0, b.ratio, if b.reaches_surge { "SURGE" } else { "ok" }));
    }
    p.print("  E0/SM_N rises as start power falls (E0 UP and SM_N DOWN): low-power burst worst on BOTH axes.");
    p.print("  Rung 34's E0 was ALREADY largest here (no relocation); SM_N is the new info (the margin consumed).");
    p.print("  The CROSSING into SURGE rides on the disclaimed phi_surge (E0 is floor-independent) and is NOT");
    p.print("  claimed; only the trend is. Constant-flow SM is a weak sign-check only. Cycle: rung-6 exact.");
}

/// `print_combustor_dynamics_table(flight)` — rung 37.
pub fn combustor_dynamics_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nCombustor dynamics (rung 37): rung 34 bundled two internal clocks into 'faster clocks below");
    p.print("tau_spool, they do not change the r framing'. They SPLIT — volume-filling CONFIRMS it (a fast");
    p.print("clock, peak unmoved), heat-soak CORRECTS it (tau_soak ~ tau_spool: a second STATE, E=E(r,theta0)).");

    let shape = ComponentMap::surge_flow();
    let eng = build_turbojet(Gas::thermally_perfect(), PI_C, TT4, flight.p0, conv_losses());
    let (lo, hi) = (1100.0, 1400.0);

    p.print(pyf!("\n  VOLUME-FILLING — frozen-spool fuel step Tt4 {:.0f}->{:.0f}, plenum clock r_v=tau_fill/tau_spool:", lo, hi));
    p.print(pyf!("  {:>6} {:>12} {:>12} {:>10} {:>11}", "r_v", "E0 (rung35)", "plenum peak", "peak-E0", "mdot split"));
    p.print(format!("  {}", dashes(54)));
    for r_v in [0.03, 0.1] {
        let ct = CombustorTransient::new(eng.clone(), *flight, 1.0, shape, r_v, 0.0, 0.0);
        let r = ct.plenum_frozen_peak(flight, lo, hi, Some(&shape), 1.0 / 15.0);
        p.print(pyf!("  {:>6.2f} {:>11.4f}% {:>11.4f}% {:>+9.5f}% {:>10.1f}%",
                     r_v, r.e0 * 100.0, r.peak * 100.0, r.peak_minus_e0 * 100.0, r.split_max * 100.0));
    }
    p.print("  The peak lands on rung-35's E0 to machine zero, INDEPENDENT of r_v (a frozen-spool map fact) —");
    p.print("  volume-filling CONFIRMS the concession. Its content is STRUCTURAL: the ~22% mdot_c != mdot_NGV");
    p.print("  split is the FIRST rung where the two mass flows differ (rung 34 tied them: pt4 = pi_b*pi_c*pt2).");

    let ct = CombustorTransient::new(eng, *flight, 1.0, shape, 0.0, 0.15, 3.0);
    let ad = ct.adiabatic_excursion(flight, lo, hi, Some(&shape), 0.05, 12.0);
    let cold = ct.soak_excursion(flight, lo, hi, Theta0::Cold, Some(&shape), 0.05, 12.0);
    let hot = ct.soak_excursion(flight, lo, hi, Theta0::Hot, Some(&shape), 0.05, 12.0);
    p.print(pyf!("\n  HEAT-SOAK — accel Tt4 {:.0f}->{:.0f} (G=0.15, r_m=tau_soak/tau_spool=3.0). E = peak surge", lo, hi));
    p.print("  excursion; t_accel = nondim time to 99% of the speed rise (the thrust-response lag):");
    p.print(pyf!("  {:>12} {:>9} {:>9}", "theta0", "E_surge", "t_accel"));
    p.print(format!("  {}", dashes(32)));
    for (tag, x) in [("adiabatic", &ad), ("cold accel", &cold), ("hot reslam", &hot)] {
        let ta = match x.t_accel { Some(t) => pyf!("{:.2f}", t), None => ">s_end".to_string() };
        p.print(pyf!("  {:>12} {:>8.2f}% {:>9}", tag, x.e_surge * 100.0, ta));
    }
    p.print("  cold < hot-reslam < adiabatic: the cold metal's heat sink depresses Tt4_turb -> colder NGV ->");
    p.print("  more airflow -> AWAY from surge, so this modeled combustor sink is surge-PROTECTIVE (rung 34/35's");
    p.print("  adiabatic no-soak case is the CEILING); a hot reslam is just the least-protected case. The primary");
    p.print("  cost is the accel-time LAG (cold ~2.5x slower). E = E(r, theta0), history-dependent — NOT a function");
    p.print("  of r alone. HONEST SCOPE: this is the OPPOSITE sign to the operational bodie/reslam surge hazard");
    p.print("  (heat soak moving the working line TOWARD surge — an UNMODELED compressor-side channel); this rung");
    p.print("  does not reproduce it. Reduce: both OFF => rung 34/35 bit-for-bit (exact dispatch); soak equilibrium");
    p.print("  == rung 35 (Q=0 at steady). Sign shape/knob-robust; magnitudes disclaimed. Cycle: rung-6 exact.");
}
