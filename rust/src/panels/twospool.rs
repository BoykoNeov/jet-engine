//! Rungs 38–45's panels — the two-spool family (slice AN).
//!
//! Each function is the `main.py` function of the same name (less the `print_` prefix), line for
//! line. Python's `None` map arguments are `ComponentMap::flat()`, which is what its constructors
//! substitute; a `TwoSpoolTransient` built without `rho` gets Python's default `1.0`.

use std::panic::{catch_unwind, AssertUnwindSafe};

use super::nox::dashes;
use super::offdesign::cpg;
use super::{Design, TT4};
use crate::bleed::TwoSpoolBleedMatcher;
use crate::components::ram_recovery;
use crate::engine::{build_turbojet, FlightCondition, Losses};
use crate::fuel_transient::{FuelTransientCore, TwoSpoolFuelTransient};
use crate::gas::{powp, Gas, GasSpec};
use crate::map::ComponentMap;
use crate::pyf;
use crate::pyfmt::{Printer, PyFormat};
use crate::spool::SpoolTransient;
use crate::two_spool::{
    build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses, TwoSpoolMapMatcher,
    TwoSpoolMatcher,
};
use crate::two_spool_transient::TwoSpoolTransient;

/// The two-spool loss set every rung-38+ panel spells out:
/// `dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96, eta_hpt=0.92,
/// eta_lpt=0.90, eta_m=0.99, pi_n=0.98)` with `nozzle_convergent=True`.
pub(crate) fn ts_losses() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
                     eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
                     nozzle_convergent: true, ..TwoSpoolLosses::default() }
}

/// `build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, flight.p0, nozzle_convergent=True, **losses)`.
pub(crate) fn ts_design(gas: Gas, pi_lpc: f64, pi_hpc: f64, d: &Design) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, d.flight.p0, ts_losses())
}

/// The panels' local `cpg()`: `g, cp = 1.3, 1239.0`, `R_t = (g - 1.0) / g * cp`.
pub(crate) fn cpg13() -> Gas {
    let (g, cp) = (1.3, 1239.0);
    cpg(g, cp, (g - 1.0) / g * cp)
}

/// `print_two_spool_matching_table(flight)` — rung 38.
pub fn two_spool_matching_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTwo-spool matching (rung 38): a THIRD choked throat (the LP-turbine NGV, A45) appears --");
    p.print("rung 31's (*) mass-flow trick chains TWICE: tau_HPT from (A4,A45), tau_LPT from (A45,A8),");
    p.print("both independent of either compressor. The two compressor RATIOS are not a 2x2 solve.");

    let (pi_lpc, pi_hpc) = (3.0, 6.0);
    let mut m = TwoSpoolMatcher::new(ts_design(Gas::reacting_equilibrium(), pi_lpc, pi_hpc, d), *flight, 1.0);

    p.print(pyf!("\n  Running line (M0={}), design pi_LPC={}, pi_HPC={}:", flight.m0, pi_lpc, pi_hpc));
    p.print(pyf!("  {:>7} {:>8} {:>8} {:>9} {:>9} {:>11} {:>8}",
                 "Tt4 [K]", "pi_LPC", "pi_HPC", "tau_HPT", "tau_LPT", "mdot/mdot_R", "F/mdot"));
    p.print(format!("  {}", dashes(68)));
    for tt4 in [1500.0, 1300.0, 1100.0, 900.0, 700.0] {
        let od = m.match_point(flight, tt4).two();
        p.print(pyf!("  {:>7.0f} {:>8.4f} {:>8.4f} {:>9.6f} {:>9.6f} {:>11.4f} {:>8.1f}",
                     tt4, od.pi_lpc, od.pi_hpc, od.tau_hpt, od.tau_lpt, od.mdot_ratio,
                     od.performance.specific_thrust));
    }
    // Python's `try: m.match(flight, 600.0) except AssertionError:` — the panic hook is left
    // alone, as in rung 33's panel.
    if catch_unwind(AssertUnwindSafe(|| m.match_point(flight, 600.0))).is_err() {
        p.print("  600      nozzle UNCHOKES here -- OUT OF SCOPE (flagged, not solved; rung-33-shaped follow-on)");
    }

    // THE FINDING — measured directly on the cascade, at a FIXED (Tt2, Tt4, f).
    let core = m.core();
    let (state0, _) = core.freestream_for(flight);
    let (tt2, pt2) = (state0.tt, core.pi_d_max * state0.pt);
    let f = 0.02;
    let pt4 = core.pi_b * core.pi_hpc_design * core.pi_lpc_design * pt2;
    let wgas = core.working_gas(f, TT4, pt4).unwrap_or_else(|| core.gas().clone());
    let base = core.cascade(&wgas, tt2, TT4, f);

    let mut perturbed = |attr: &str, value: f64| -> (bool, bool) {
        let c = m.core_mut();
        let slot = match attr {
            "eta_hpc" => &mut c.eta_hpc,
            "eta_lpc" => &mut c.eta_lpc,
            "eta_hpt" => &mut c.eta_hpt,
            _ => &mut c.eta_lpt,
        };
        let saved = *slot;
        *slot = value;
        let r = m.core().cascade(&wgas, tt2, TT4, f);
        let c = m.core_mut();
        match attr {
            "eta_hpc" => c.eta_hpc = saved,
            "eta_lpc" => c.eta_lpc = saved,
            "eta_hpt" => c.eta_hpt = saved,
            _ => c.eta_lpt = saved,
        }
        (r.pi_lpc != base.pi_lpc, r.pi_hpc != base.pi_hpc)
    };

    p.print("\n  THE FINDING — perturb ONE component parameter at fixed (Tt2, Tt4, f); does pi_LPC / pi_HPC move?");
    p.print(pyf!("  {:>10} {:>14} {:>14}  role", "parameter", "moves pi_LPC?", "moves pi_HPC?"));
    p.print(format!("  {}", dashes(62)));
    for (attr, label, role) in [
        ("eta_hpc", "eta_HPC", "HP compressor's OWN pressure-inversion leaf"),
        ("eta_lpc", "eta_LPC", "LP compressor's OWN pressure-inversion leaf"),
        ("eta_hpt", "eta_HPT", "energy-path (shapes the shared Tt45)"),
        ("eta_lpt", "eta_LPT", "energy-path (shapes the shared Tt25 via Tt5)"),
    ] {
        let (moves_lpc, moves_hpc) = perturbed(attr, 0.55);
        p.print(pyf!("  {:>10} {:>14} {:>14}  {}", label, moves_lpc.py_str(), moves_hpc.py_str(), role));
    }
    p.print("  Each compressor's OWN efficiency is a dead end for the OTHER spool -- so the two");
    p.print("  compressor ratios are never a joint (2x2) solve. This is narrower than 'the spools");
    p.print("  don't talk' (eta_HPT/eta_LPT legitimately move BOTH -- an initial over-claim, corrected).");
    p.print("  Reduce: lp_disabled=True dispatches (not limits) to rung 31's OffDesignMatcher bit-for-bit.");
    p.print("  Scope: isentropic knobs only, no compressor maps yet; cycle stays rung-6 exact.");
}

/// `print_two_spool_map_table(flight)` — rung 39.
pub fn two_spool_map_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTwo-spool + component maps (rung 39): rung 38 predicted a map would force a joint 2x2");
    p.print("solve. It does not. pi_LPC CANCELS out of the HP compressor's corrected flow, so the map");
    p.print("opens EXACTLY ONE arrow (HP -> LP): the cascade acquires a DIRECTION instead of dissolving.");

    let (pi_lpc, pi_hpc) = (3.0, 6.0);
    let matcher = |gas: Gas, ml: ComponentMap, mh: ComponentMap| {
        TwoSpoolMapMatcher::new(ts_design(gas, pi_lpc, pi_hpc, d), *flight, 1.0, ml, mh)
    };
    let map_lp = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, a_t: 0.0, ..ComponentMap::default() };
    let map_hp = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, a_t: 0.0, ..ComponentMap::default() };
    let flat = ComponentMap::flat();

    // --- THE FINDING: the asymmetry, at a FIXED (Tt2, pt2, Tt4, f).
    let mut m = matcher(cpg13(), map_lp, map_hp);
    let tt4_p = 1200.0;
    let od0 = m.match_point(flight, tt4_p).two();
    let core = m.core();
    let (state0, _) = core.base.freestream_for(flight);
    let tt2 = state0.tt;
    let pt2 = core.base.pi_d_max * ram_recovery(flight.m0) * state0.pt;
    let f = od0.base.station("4").far;
    let pt4 = core.base.pi_b * od0.base.pi_hpc * od0.base.pi_lpc * pt2;
    let wgas = core.base.working_gas(f, tt4_p, pt4).unwrap_or_else(|| core.gas().clone());
    let base = core.cascade_map(&wgas, tt2, pt2, tt4_p, f);

    let mut perturb = |attr: &str| -> (f64, f64) {
        let delta = 0.01;
        let c = &mut m.core_mut().base;
        let slot = match attr {
            "eta_lpc" => &mut c.eta_lpc,
            "eta_hpc" => &mut c.eta_hpc,
            "eta_hpt" => &mut c.eta_hpt,
            _ => &mut c.eta_lpt,
        };
        let saved = *slot;
        *slot = saved - delta;
        let r = m.core().cascade_map(&wgas, tt2, pt2, tt4_p, f);
        let c = &mut m.core_mut().base;
        match attr {
            "eta_lpc" => c.eta_lpc = saved,
            "eta_hpc" => c.eta_hpc = saved,
            "eta_hpt" => c.eta_hpt = saved,
            _ => c.eta_lpt = saved,
        }
        (r.c.pi_lpc / base.c.pi_lpc - 1.0, r.c.pi_hpc / base.c.pi_hpc - 1.0)
    };

    p.print(pyf!("\n  THE FINDING — perturb one efficiency by -0.01 at fixed (Tt2, pt2, Tt4={:.0f}, f);", tt4_p));
    p.print("  compressor maps only (a_t = 0), so the reading is the pure structural one:");
    p.print(pyf!("  {:>10} {:>15} {:>15}  channel", "parameter", "d pi_LPC / pi", "d pi_HPC / pi"));
    p.print(format!("  {}", dashes(74)));
    for (attr, label, role) in [
        ("eta_lpc", "eta_LPC", "own ratio only -- CANNOT reach pi_HPC (pi_LPC cancels)"),
        ("eta_hpc", "eta_HPC", "own ratio AND pi_LPC -- THE ONE ARROW the map opens"),
        ("eta_hpt", "eta_HPT", "energy path: moves BOTH (shapes the shared Tt45)"),
        ("eta_lpt", "eta_LPT", "energy path: moves BOTH (shapes Tt25 via Tt5)"),
    ] {
        let (dl, dh) = perturb(attr);
        let sl = if dl == 0.0 { "EXACTLY 0".to_string() } else { pyf!("{:+.3e}", dl) };
        let sh = if dh == 0.0 { "EXACTLY 0".to_string() } else { pyf!("{:+.3e}", dh) };
        p.print(pyf!("  {:>10} {:>15} {:>15}  {}", label, sl, sh, role));
    }
    p.print("  eta_LPC -> pi_HPC is EXACTLY zero (bit-for-bit), by the (dagger) cancellation -- while");
    p.print("  eta_HPC -> pi_LPC is real and negative. ONE arrow, not two: still strictly triangular.");

    // --- the structural novelty: two speeds, hence the slip.
    p.print("\n  THE STRUCTURAL NOVELTY — two shaft speeds (rung 38 computes none) => the SLIP N_L/N_H:");
    p.print(pyf!("  {:>7} {:>8} {:>8} {:>8} {:>8} {:>9} {:>9} {:>9}",
                 "Tt4 [K]", "pi_LPC", "pi_HPC", "eta_LPC", "eta_HPC", "N_L/N_Ld", "N_H/N_Hd", "slip"));
    p.print(format!("  {}", dashes(78)));
    let ms = matcher(cpg13(), map_lp, map_hp);
    for tt4 in [1500.0, 1300.0, 1100.0, 900.0] {
        let od = ms.match_point(flight, tt4).two();
        p.print(pyf!("  {:>7.0f} {:>8.4f} {:>8.4f} {:>8.5f} {:>8.5f} {:>9.5f} {:>9.5f} {:>9.6f}",
                     tt4, od.base.pi_lpc, od.base.pi_hpc, od.eta_lpc, od.eta_hpc,
                     od.n_lp_ratio, od.n_hp_ratio, od.slip));
    }
    p.print("  The LP spool falls AWAY from the HP spool as the engine is throttled back -- the");
    p.print("  textbook twin-spool behaviour (at idle a twin-spool runs high N_H, much lower N_L).");

    // --- B1/B2: the slip identity and what breaks it.
    p.print("\n  WHERE THE SLIP COMES FROM — slip == 1 is a STRUCTURAL identity, broken by two channels:");
    p.print(pyf!("  {:>26} {:>10} {:>10} {:>10} {:>10}", "gas / map", "Tt4=1500", "1300", "1100", "900"));
    p.print(format!("  {}", dashes(70)));
    let rows: [(&str, fn() -> Gas, ComponentMap, ComponentMap); 4] = [
        ("CPG, FLAT maps", cpg13, flat, flat),
        ("thermally-perfect, FLAT", Gas::thermally_perfect, flat, flat),
        ("reacting, FLAT", Gas::reacting_equilibrium, flat, flat),
        ("CPG, SHAPED maps", cpg13, map_lp, map_hp),
    ];
    for (label, gas_fn, ml, mh) in rows {
        let mm = matcher(gas_fn(), ml, mh);
        let row: String = [1500.0, 1300.0, 1100.0, 900.0].iter()
            .map(|&t| pyf!("{:>10.6f}", mm.match_point(flight, t).two().slip))
            .collect();
        p.print(pyf!("  {:>26} {}", label, row));
    }
    p.print("  Both shaft works are eta_m*(1+f)*cp_t*Tt4*[pure geometry], so (1+f) AND Tt4 cancel in");
    p.print("  N_L/N_H: on CPG + flat maps the slip is EXACTLY 1 at every throttle. The cp(T) gas curve");
    p.print("  breaks it ~1.5% (the rung-31-gate-5 mirror); the MAP breaks it ~5% -- the larger channel.");
    p.print("  ON CPG that INVERTS rung 32 (where the map only re-labelled map-free work): the deviation is");
    p.print("  identically zero without a map, so there the map is the SOLE channel and CREATES the object.");
    p.print("  NOT unconditional -- on the reacting gas the same FLAT maps already give 0.9835 at Tt4=900,");
    p.print("  so on the real gas the map is the DOMINANT channel (~3.4x), not the only one.");
    p.print("  Reduce: FLAT maps => rung 38 bit-for-bit;");
    p.print("  lp_disabled dispatches to rung 32 (shaped) / rung 31 (flat). Cycle stays rung-6 exact.");
    p.print("  Disclaimed: representative maps -- every magnitude (arrow strength, slip depth) rides on");
    p.print("  the shapes; only the asymmetry, the identity and the slip SIGN are load-bearing.");
}

/// `print_two_shaft_transient_table(flight)` — rung 40.
pub fn two_shaft_transient_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTwo-shaft transient (rung 40): both shaft speeds become STATES. Nondimensionalizing");
    p.print("leaves ONE parameter -- the clock RATIO rho = tau_L/tau_H -- and its power SPLITS:");
    p.print("it can never destabilize the pair, but it decides whether the mode is real or COMPLEX.");

    let flat = ComponentMap::flat();
    let lp_s = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::default() };
    let hp_s = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::default() };
    let tt = |gas: Gas, ml: ComponentMap, mh: ComponentMap| {
        TwoSpoolTransient::new(ts_design(gas, 3.0, 6.0, d), *flight, 1.0, ml, mh, 1.0)
    };

    // --- the reduce: the 2-D root lands on rung 39, through the FORWARD closure only.
    let t = tt(cpg13(), lp_s, hp_s);
    let tc = t.core();
    p.print("\n  REDUCE — the 2-D equilibrium (Phi_L = Phi_H = 0) vs rung 39's matched point");
    p.print("  (via the forward closure ONLY -- it never calls the matcher, so this is non-circular):");
    p.print(pyf!("  {:>6} {:>11} {:>11} {:>11} {:>11}", "Tt4", "d nu_L", "d nu_H", "d pi_LPC", "d pi_HPC"));
    p.print(format!("  {}", dashes(56)));
    for tt4 in [1500.0, 1300.0, 1200.0] {
        let od = tc.match_point(flight, tt4);
        let eq = tc.equilibrium(flight, tt4);
        p.print(pyf!("  {:6.0f} {:11.2e} {:11.2e} {:11.2e} {:11.2e}",
                     tt4, eq.nu_lp / od.n_lp_ratio - 1.0, eq.nu_hp / od.n_hp_ratio - 1.0,
                     eq.close.pi_lpc / od.base.pi_lpc - 1.0, eq.close.pi_hpc / od.base.pi_hpc - 1.0));
    }

    // --- sigma_crit: the lead threshold. The ==1 identity is INHERITED from rung 39 B1.
    p.print("\n  sigma_crit (the LEAD THRESHOLD: HP leads iff rho > sigma_crit), at Tt4=1100.");
    p.print("  == 1 on flat+CPG is rung 39's B1 slip identity restated for the transient");
    p.print("  (on the running line sigma_crit reduces to the steady slip) -- INHERITED, the");
    p.print("  reduce spine, NOT this rung's finding:");
    let rows = [("CPG + flat maps", cpg13(), flat, flat),
                ("thermally_perfect + flat", Gas::thermally_perfect(), flat, flat),
                ("reacting + flat", Gas::reacting_equilibrium(), flat, flat),
                ("CPG + shaped maps", cpg13(), lp_s, hp_s)];
    p.print(pyf!("  {:>26} {:>16}   channel", "configuration", "sigma_crit - 1"));
    p.print(format!("  {}", dashes(66)));
    for (label, gas, ml, mh) in rows {
        let dev = tt(gas, ml, mh).core().lead_threshold(flight, 1100.0, 25.0, None) - 1.0;
        let ch = if dev.abs() < 1e-11 { "the IDENTITY" }
                 else if ml.is_flat() { "the cp(T) gas curve" } else { "the MAP" };
        p.print(pyf!("  {:>26} {:16.4e}   {}", label, dev, ch));
    }
    p.print("  => the map channel is ~5.8x the gas channel: DOMINANT, not sole (rung 39 B2's shape).");

    let lp_only = tt(cpg13(), lp_s, flat).core().lead_threshold(flight, 1100.0, 5.0, None);
    let hp_only = tt(cpg13(), flat, hp_s).core().lead_threshold(flight, 1100.0, 5.0, None);
    p.print("\n  A REFUTED hypothesis, kept visible: 'the map favours the LP spool' is FALSE --");
    p.print(pyf!("  shaping only the LP map gives sigma_crit = {:.4f} (< 1), only the HP map {:.4f} (> 1).",
                 lp_only, hp_only));
    p.print("  Both signs are reachable, so only the EXISTENCE of a material shift is claimed.");

    // --- THE FINDING: stability is rho-free; the complex mode is created by the LP map.
    p.print("\n  THE FINDING — J(rho) = [[a/rho, b/rho], [c, d]] on the running line, Tt4=1200:");
    p.print("    tr = a/rho + d,  det = (ad-bc)/rho,  disc = (a/rho - d)^2 + 4bc/rho");
    p.print("  STABILITY needs a<0, d<0, ad>bc -- three conditions with NO rho in them, so the");
    p.print("  clock ratio can NEVER destabilize the pair. OSCILLATION is different: disc kills");
    p.print("  its first term at rho = a/d, so bc<0 => a COMPLEX band exists.");
    p.print(pyf!("\n  {:>12} {:>10} {:>11} {:>9} {:>20} {:>11}", "shapes", "b", "b*c", "ad-bc", "band (rho)", "|Im/Re|max"));
    p.print(format!("  {}", dashes(78)));
    for (name, ml, mh) in [("flat", flat, flat), ("hp-only", flat, hp_s),
                           ("lp-only", lp_s, flat), ("flow/press", lp_s, hp_s)] {
        let mut tx = tt(cpg13(), ml, mh);
        let od = tx.core().match_point(flight, 1200.0);
        let nu = (od.n_lp_ratio, od.n_hp_ratio);
        tx.core_mut().rho = 1.0;
        let tx = tx.core();
        let j = tx.jacobian(flight, 1200.0, Some(nu), 1e-6);
        let bc = j[0][1] * j[1][0];
        let band = tx.oscillatory_band(flight, 1200.0, Some(nu));
        let bs = match band { None => "none".to_string(), Some((lo, hi)) => pyf!("[{:.3f}, {:.3f}]", lo, hi) };
        p.print(pyf!("  {:>12} {:10.4f} {:11.4e} {:9.4f} {:>20} {:11.3f}",
                     name, j[0][1], bc, j[0][0] * j[1][1] - bc, bs,
                     tx.damping_ratio_max(flight, 1200.0, Some(nu))));
    }
    p.print("\n  hp-only is the DISCRIMINATOR: its HP map IS shaped, yet no band appears -- so the");
    p.print("  mechanism is the LP map SPECIFICALLY, not shaping in general. A shaped LP map flips");
    p.print("  b = dPhi_L/dnu_H from small-negative to large-positive; with c<0 always, that");
    p.print("  antisymmetric cross-coupling is what makes the pair complex. The mode is MAP-CREATED");
    p.print("  -- rung 39's slip pattern a third time.");
    p.print("  |Im/Re| <= 0.25 here (no visible ringing) is a DISCLAIMED MAGNITUDE, not a verdict:");
    p.print("  existence + sign + mechanism are gated, the number rides on the representative maps.");
    p.print("\n  SCOPE (a negative, stated plainly): sigma_crit's authority is FIRST-INSTANT only.");
    p.print("  The finite-ramp slip excursion is SCHEDULE-SLAVED -- dominated by slip_ss(Tt4) moving");
    p.print("  while the speeds lag -- so the marched threshold is NOT sigma_crit (0.60x / 1.40x).");
    p.print("  Oscillation claim scoped to INTER-SPOOL (rung 37's shaft+metal pair is not audited).");
    p.print("  Reduce: lp_disabled => rung 34 SpoolTransient bit-for-bit. Cycle stays rung-6 exact.");
}

/// `print_two_spool_surge_table(flight)` — rung 41.
pub fn two_spool_surge_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTwo-spool surge line (rung 41): drawing rung 36's line on BOTH compressors. The");
    p.print("two-spool running line does not halve the low-power surge problem -- it CONCENTRATES");
    p.print("it on the LP spool, and the reason is closed-form.");

    let single = Losses { pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92,
                          eta_m: 0.99, pi_n: 0.98, nozzle_convergent: true, ..Losses::default() };
    let tilted = ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::default() };
    let flat = ComponentMap::flat();
    let mm = |gas: Gas, ml: ComponentMap, mh: ComponentMap, pl: f64, ph: f64| {
        TwoSpoolMapMatcher::new(ts_design(gas, pl, ph, d), *flight, 1.0, ml, mh)
    };

    // --- THE SPLIT: the LP takes the excursion, the HP is shielded and BOUNDED.
    let m = mm(Gas::thermally_perfect(), tilted, tilted, 3.0, 6.0);
    p.print("\n  THE SPLIT — the two running lines in map coordinates (matched `tilted` shape):");
    p.print(pyf!("  {:>6} {:>8} {:>8} {:>8} {:>8}", "Tt4", "phi_L", "phi_H", "pi_LPC", "pi_HPC"));
    p.print(format!("  {}", dashes(42)));
    for r in m.core().running_line_map(flight, &[1500.0, 1300.0, 1100.0, 900.0, 800.0, 750.0]) {
        p.print(pyf!("  {:6.0f} {:8.4f} {:8.4f} {:8.4f} {:8.4f}", r.tt4, r.phi_lp, r.phi_hp, r.pi_lpc, r.pi_hpc));
    }
    p.print("  => phi_L falls ~29%, phi_H ~7% and TURNS BACK UP. The LP takes the excursion.");

    // --- WHY: the sensitivity of each face is closed-form in the pressure ratios it SEES.
    p.print("\n  WHY — each face's flow-coefficient sensitivity, closed form (CPG, flat maps):");
    p.print("    s_H = k(1 - pi_HPC^(-1/k)) - 1                     <- pi_HPC ALONE (rung 39 (dagger))");
    p.print("    s_L = k(1 - pi_LPC^(-1/k)) + k(1 - pi_HPC^(-1/k))/tau_LPC - 1   <- the PRODUCT");
    let mc = mm(cpg13(), flat, flat, 3.0, 6.0);
    let mcc = mc.core();
    let k = 1.4 / 0.4;
    p.print(pyf!("  {:>6} {:>8} {:>8} {:>8} {:>8} {:>16}", "Tt4", "s_H", "pred", "s_L", "pred", "pred w/o pi_HPC"));
    p.print(format!("  {}", dashes(60)));
    for tt4 in [1400.0, 1200.0, 1000.0, 850.0, 750.0] {
        let (a, b) = (mcc.match_point(flight, tt4 - 4.0), mcc.match_point(flight, tt4 + 4.0));
        let c = mcc.match_point(flight, tt4);
        let lg = |o: &crate::two_spool::TwoSpoolMapResult, hp: bool| -> (f64, f64) {
            let x = o.base.tt4 / if hp { o.base.station("25").tt } else { o.base.station("2").tt };
            ((if hp { o.phi_hp } else { o.phi_lp }).ln(), x.ln())
        };
        let ((ya, xa), (yb, xb)) = (lg(&a, true), lg(&b, true));
        let s_h = (yb - ya) / (xb - xa);
        let ((ya, xa), (yb, xb)) = (lg(&a, false), lg(&b, false));
        let s_l = (yb - ya) / (xb - xa);
        let tau_l = c.base.station("25").tt / c.base.station("2").tt;
        let s_hp = k * (1.0 - powp(c.base.pi_hpc, -1.0 / k)) - 1.0;
        let s_lp = k * (1.0 - powp(c.base.pi_lpc, -1.0 / k))
            + k * (1.0 - powp(c.base.pi_hpc, -1.0 / k)) / tau_l - 1.0;
        let s_ln = k * (1.0 - powp(c.base.pi_lpc, -1.0 / k)) - 1.0;
        p.print(pyf!("  {:6.0f} {:8.4f} {:8.4f} {:8.4f} {:8.4f} {:16.4f}", tt4, s_h, s_hp, s_l, s_lp, s_ln));
    }
    p.print("  => both predictions land within ~0.013. DROPPING pi_HPC from s_L misses by ~0.8-1.0");
    p.print("     AND gets the SIGN wrong: the LP face cannot be written without the HP's ratio,");
    p.print("     while the HP's needs no LP quantity at all. The shielding, quantified.");

    // --- (star): the closed form, and its fuel-fraction kill test.
    p.print("\n  THE CLOSED FORM (star) — s_H = 0 gives 1 + eta_c(tau_c-1) = gamma_c, i.e.");
    p.print(pyf!("    pi_c* = gamma_c^(gamma_c/(gamma_c-1)) = {:.5f}  (gamma_c ALONE).", mcc.critical_flow_turn_pi()));
    p.print("  The turn's location in Tt4 moves by ~76%; its location in PRESSURE RATIO does not:");
    p.print(pyf!("  {:>16} {:>8} {:>9} {:>14} {:>12}", "case", "Tt4*", "pi*", "1+eta(tau-1)", "vs gamma_c"));
    p.print(format!("  {}", dashes(64)));
    for (label, pl, ph) in [("split 3x6", 3.0, 6.0), ("split 4.5x4", 4.5, 4.0), ("split 2.25x8", 2.25, 8.0)] {
        let t = mm(cpg13(), flat, flat, pl, ph).core().flow_coefficient_turn(flight, Spool::Hp);
        let sf = t.star_form.unwrap();
        p.print(pyf!("  {:>16} {:8.1f} {:9.4f} {:14.5f} {:11.3f}%",
                     label, t.tt4_star, t.pi_star.unwrap(), sf, 100.0 * (sf / 1.4 - 1.0)));
    }
    let t = mm(cpg13(), flat, flat, 3.0, 6.0).core()
        .flow_coefficient_turn(&FlightCondition::new(250.0, 50_000.0, 1.60), Spool::Hp);
    let sf = t.star_form.unwrap();
    p.print(pyf!("  {:>16} {:8.1f} {:9.4f} {:14.5f} {:11.3f}%",
                 "M0=1.60", t.tt4_star, t.pi_star.unwrap(), sf, 100.0 * (sf / 1.4 - 1.0)));
    p.print("  eta_HPC (0.80/0.95), eta_HPT, gamma_t and cp_t all drop out -- verified in the gates.");
    p.print("\n  KILL TEST — the whole +0.44% residual is the FUEL FRACTION (f enters K and the");
    p.print("  choked flow; (star) is exact with f frozen). Raise hPR so f -> 0:");
    p.print(pyf!("  {:>10} {:>9} {:>14} {:>10}", "hPR", "f", "1+eta(tau-1)", "residual"));
    p.print(format!("  {}", dashes(46)));
    for hpr in [42.8e6, 4.28e8, 4.28e10] {
        let (g, cp) = (1.3, 1239.0);
        let gas = Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: g, cp_t: cp,
                                     r_t: (g - 1.0) / g * cp, hpr, ..GasSpec::default() });
        let t = mm(gas, flat, flat, 3.0, 6.0).core().flow_coefficient_turn(flight, Spool::Hp);
        let sf = t.star_form.unwrap();
        p.print(pyf!("  {:10.3g} {:9.5f} {:14.5f} {:9.3f}%", hpr, t.far.unwrap(), sf, 100.0 * (sf / 1.4 - 1.0)));
    }

    // --- the margins, and the deliberate divergence.
    p.print("\n  THE MARGINS — matched shape on both spools, COMMON imposed floor phi_surge=0.55");
    p.print("  (rung 36's one disclosed constant, DOUBLED -- every magnitude disclaimed):");
    let ms = mm(Gas::thermally_perfect(), tilted.with_phi_surge(0.55), tilted.with_phi_surge(0.55), 3.0, 6.0);
    p.print(pyf!("  {:>6} {:>8} {:>8} {:>11} {:>8}", "Tt4", "SM_L", "SM_H", "SM_L/SM_H", "phi_H"));
    p.print(format!("  {}", dashes(46)));
    for r in ms.core().surge_margin_schedule(flight, &[1500.0, 1300.0, 1100.0, 900.0, 800.0, 750.0]) {
        p.print(pyf!("  {:6.0f} {:8.4f} {:8.4f} {:11.4f} {:8.4f}", r.tt4, r.sm_lp, r.sm_hp, r.sm_lp / r.sm_hp, r.phi_hp));
    }
    p.print("  => the RATIO collapses (0.61 -> 0.13): THAT is the running-line divergence, and it");
    p.print("     is what is gated. The ORDERING's LEVEL is partly a DESIGN-SPLIT artifact --");
    p.print("     SM_L < SM_H already at Tt4=1500 where phi_L = phi_H = 1, purely because");
    p.print("     pi_LPC=3 < pi_HPC=6 (a smaller design pressure ratio gives a smaller margin at");
    p.print("     the same flow-coefficient gap). Not over-attributed to exposure.");
    p.print("     NOTE also the deliberate DIVERGENCE:");
    p.print("     phi_H TURNS UP past pi* while SM_H keeps FALLING -- so (star) is an INCIDENCE");
    p.print("     fact, NOT a margin extremum. The worst margin is still at idle, on both spools.");

    // --- the cross-rung correction of rung 36.
    p.print("\n  THE CORRECTION OF RUNG 36 (the rung-28 shape: verdict confirmed, reason corrected).");
    p.print("  (star) is SURFACED by this rung, not created by it -- the same turn sits INSIDE rung");
    p.print("  36's OWN choked envelope (pi_c=10 single spool). Freezing one coordinate at a time:");
    let d1 = build_turbojet(Gas::thermally_perfect(), 10.0, TT4, flight.p0, single);
    let cm = ComponentMap::surge_flow().with_phi_surge(0.55);
    let st = SpoolTransient::new(d1, *flight, 1.0, cm);
    p.print(pyf!("  {:>6} {:>8} {:>9} {:>8} {:>13} {:>15}", "Tt4", "pi_c", "phi_op", "SM_N", "SM(phi-walk)", "SM(speed-line)"));
    p.print(format!("  {}", dashes(64)));
    for tt4 in [1500.0, 1100.0, 900.0, 800.0, 700.0, 650.0, 600.0] {
        // Python's `try: … except AssertionError: break`.
        let c = match catch_unwind(AssertUnwindSafe(|| st.surge_margin_channels(flight, tt4, Some(&cm), None))) {
            Ok(c) => c,
            Err(_) => break,
        };
        p.print(pyf!("  {:6.0f} {:8.4f} {:9.5f} {:8.4f} {:13.4f} {:15.4f}",
                     tt4, c.pi_c, c.phi_op, c.sm_n, c.sm_phi_walk, c.sm_speed_line));
    }
    p.print("  => phi_op TURNS UP at the bottom (crossing pi*), and the phi-walk channel turns with");
    p.print("     it -- yet SM_N keeps falling, because the SPEED LINE FLATTENS (tau_c-1 ~ n^2) and");
    p.print("     that channel does not reverse. Rung 36's VERDICT survives (SM_N still monotone,");
    p.print("     no rung-36 test changes); its stated MECHANISM ('the trend is set by phi_op') was");
    p.print("     SINGLE-CHANNEL -- the two are comparable (~56%/48% of the log decay). Both are");
    p.print("     choke-determined, so rung 36's floor-robustness conclusion is untouched.");
    p.print("\n  NOT claimed: 'the slip protects the LP spool' (the rigid-shaft counterfactual is");
    p.print("  not run); which spool binds at UNMATCHED shapes/floors; any margin magnitude.");
    p.print("  Reduce: a phi_surge-carrying map leaves rung 39/40 bit-for-bit. Cycle stays rung-6 exact.");
}

/// `print_interstage_bleed_table(flight)` — rung 42.
pub fn interstage_bleed_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nInterstage bleed (rung 42): the device rungs 36 and 41 both deferred, fitted to the");
    p.print("spool rung 41 showed is exposed. A fraction b is extracted at station 25 and dumped --");
    p.print("the project's FIRST steady mass EXTRACTION -- the first time mass LEAVES the flowpath,");
    p.print("so the two COMPRESSORS pass different air. (NOT 'compressor and turbine differ': (1+f)");
    p.print("has done that since rung 2. What is new is mass leaving, not mass changing.)");
    p.print("Comparison held at FIXED Tt4: the valve sets b, not the throttle.");

    let lp = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::default() };
    let hp = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::default() };
    let flat = ComponentMap::default();
    let bm = |gas: Gas, ml: ComponentMap, mh: ComponentMap, b: f64, floor: Option<f64>| {
        let (ml, mh) = match floor {
            Some(fl) => (ml.with_phi_surge(fl), mh.with_phi_surge(fl)),
            None => (ml, mh),
        };
        TwoSpoolBleedMatcher::new(ts_design(gas, 3.0, 6.0, d), *flight, 1.0, ml, mh, b)
    };

    // --- WHERE b ENTERS: three places, and not the fourth.
    p.print("\n  WHERE b ENTERS -- exactly three places:");
    p.print("    (1) the LP shaft balance   h_c(Tt25)-h_c(Tt2) = eta_m(1-b)(1+f)dh_LPT   => Tt25 FALLS");
    p.print("    (2) the LP face referral   (ddagger-b): mdot_corr,2 picks up an explicit 1/(1-b)");
    p.print("    (3) the thrust books       the dumped air keeps full ram drag, returns no momentum");
    p.print("  and NOT the HP face: rung 39's (dagger) mdot_corr,25 = A4 pi_b pi_HPC MFP* sqrt(Tt25/Tt4)/(1+f)");
    p.print("  is core flow on BOTH sides, so it carries NO b -- which is why _hp_eta_loop is reused");
    p.print("  VERBATIM. Its BODY is b-free; its ARGUMENTS are not (rung 39's leaf, one rung on).");

    // --- THE ASYMMETRY: LP displaced OFF its line, HP only slides ALONG its own.
    p.print("\n  THE ASYMMETRY -- open the valve at a FIXED Tt4 (shapes flow/press, b = 0.10):");
    p.print(pyf!("  {:>6} {:>8} {:>9} {:>8} {:>9} {:>8} {:>8} {:>8}",
                 "Tt4", "x_L", "dphi_L", "x_H", "dphi_H", "ratio", "dF", "dTSFC"));
    p.print(format!("  {}", dashes(70)));
    let (shut, opn) = (bm(Gas::thermally_perfect(), lp, hp, 0.0, None),
                       bm(Gas::thermally_perfect(), lp, hp, 0.10, None));
    for tt4 in [1500.0, 1300.0, 1100.0, 900.0] {
        let (a, c) = (shut.match_point(flight, tt4), opn.match_point(flight, tt4));
        let (xla, xlc) = (tt4 / a.base.base.station("2").tt, tt4 / c.base.base.station("2").tt);
        let (dl, dh) = (c.base.phi_lp / a.base.phi_lp - 1.0, c.base.phi_hp / a.base.phi_hp - 1.0);
        let tsfc_inlet = c.booking.as_ref().expect("the valve is open").tsfc_inlet;
        p.print(pyf!("  {:6.0f} {:8.4f} {:+8.3f}% {:8.4f} {:+8.3f}% {:8.1f} {:+7.2f}% {:+7.2f}%",
                     tt4, xla, 100.0 * dl, tt4 / a.base.base.station("25").tt, 100.0 * dh, dl / dh,
                     100.0 * (c.base.base.thrust / a.base.base.thrust - 1.0),
                     100.0 * (tsfc_inlet / a.base.base.performance.tsfc - 1.0)));
        assert_eq!(xla, xlc);   // x_L is built from two INPUTS: bleed cannot move it
    }
    p.print("  => x_L is EXACTLY bleed-invariant (both Tt4 and Tt2 are inputs), so the whole dphi_L");
    p.print("     is displacement OFF the LP running line: the LP line becomes a FAMILY indexed by b.");

    p.print("\n  ...and the HP stays on ONE curve. Take the bled point's x_H, find the b=0 THROTTLE");
    p.print("  setting with the SAME x_H, compare phi_H (CPG, flat maps, b = 0.10):");
    let (cshut, copn) = (bm(cpg13(), flat, flat, 0.0, None), bm(cpg13(), flat, flat, 0.10, None));
    p.print(pyf!("  {:>6} {:>11} {:>9} {:>10} {:>20}", "Tt4", "Tt4* (b=0)", "x_H", "HP dphi", "LP dphi (same x_L)"));
    p.print(format!("  {}", dashes(62)));
    for tt4 in [1400.0, 1100.0, 900.0] {
        let c = copn.match_point(flight, tt4);
        let target = tt4 / c.base.base.station("25").tt;
        let (mut lo, mut hi) = (tt4, tt4 * 1.3);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            let o = cshut.match_point(flight, mid);
            if mid / o.base.base.station("25").tt - target <= 0.0 { lo = mid; } else { hi = mid; }
        }
        let o = cshut.match_point(flight, 0.5 * (lo + hi));
        let a = cshut.match_point(flight, tt4);
        p.print(pyf!("  {:6.0f} {:11.2f} {:9.5f} {:+9.4f}% {:+19.2f}%",
                     tt4, 0.5 * (lo + hi), target, 100.0 * (c.base.phi_hp / o.base.phi_hp - 1.0),
                     100.0 * (c.base.phi_lp / a.base.phi_lp - 1.0)));
    }
    p.print("  => the HP leaves its running line by ~0.01% while the LP is displaced ~11% -- a");
    p.print("     ~1000x contrast. Bleed gives the LP spool a new freedom; the HP it merely slides.");

    // --- INHERITED: the HP response IS rung 41's s_H, and it reverses sign at pi*.
    p.print("\n  SO THE HP RESPONSE IS RUNG 41's (INHERITED, and the spec says so): s_H measured by");
    p.print("  opening the VALVE vs rung 41's closed form measured on the THROTTLE --");
    p.print("  perturbation-independence, which could have failed (the HP loop reads Tt4, Tt25 and f");
    p.print("  SEPARATELY on the real gas; only CPG at frozen f makes it one-parameter in x_H):");
    let k = 1.4 / 0.4;
    let pi_star = powp(1.4, k);
    let cdb = bm(cpg13(), flat, flat, 0.02, None);
    p.print(pyf!("  {:>6} {:>8} {:>12} {:>11} {:>9} {:>10}", "Tt4", "pi_HPC", "s_H (valve)", "s_H closed", "diff", "dln phi_L"));
    p.print(format!("  {}", dashes(62)));
    for tt4 in [1500.0, 1300.0, 1100.0, 900.0, 800.0, 790.0, 780.0, 750.0, 700.0] {
        let (a, c) = (cshut.match_point(flight, tt4), cdb.match_point(flight, tt4));
        let (xa, xc) = (tt4 / a.base.base.station("25").tt, tt4 / c.base.base.station("25").tt);
        let sm = (c.base.phi_hp / a.base.phi_hp).ln() / (xc / xa).ln();
        let sc = k * (1.0 - powp(a.base.base.pi_hpc, -1.0 / k)) - 1.0;
        let mark = if (780.0..=790.0).contains(&tt4) { "  <- pi*" } else { "" };
        p.print(pyf!("  {:6.0f} {:8.5f} {:12.4f} {:11.4f} {:+9.4f} {:+10.5f}{}",
                     tt4, a.base.base.pi_hpc, sm, sc, sm - sc, (c.base.phi_lp / a.base.phi_lp).ln(), mark));
    }
    p.print("  => agreement to <=0.004 over a 2.4:1 throttle. And since s_H = 0 at");
    p.print(pyf!("     pi* = gamma_c^(gamma_c/(gamma_c-1)) = {:.5f}, the bleed response passes", pi_star));
    p.print("     through ZERO there and REVERSES SIGN below it -- bracketed above between");
    p.print("     Tt4 = 790 (pi_HPC = 3.2688, +) and 780 (3.2339, -). The crossing interpolates to");
    p.print("     pi_HPC ~ 3.260, i.e. +0.40%: the SAME fuel-fraction residual rung 41's own kill");
    p.print("     test isolated (+0.44%). pi* SURFACES A THIRD TIME -- its LOCATION is inherited,");
    p.print("     that a second, independent perturbation sweeps through it is new.");
    p.print("     The growing ratio is the HP denominator passing through zero (dln phi_L is nearly");
    p.print("     constant ~0.022 throughout) -- NOT 'infinite selectivity'.");

    // --- SELF-TARGETING, stated in phi-space (NOT in relative margin).
    p.print("\n  SELF-TARGETING -- stated in phi-SPACE (rung 41's surge-proximity currency), because");
    p.print("  the relative-margin version is CONFOUNDED (absolute dSM_L SHRINKS 0.056 -> 0.018 pp;");
    p.print("  only its collapsing base makes the relative gain 'grow' -- this project's own rung-41");
    p.print("  lesson).  b = 0.10, matched imposed floor phi_surge = 0.55:");
    let (fshut, fopn) = (bm(cpg13(), lp, hp, 0.0, Some(0.55)), bm(cpg13(), lp, hp, 0.10, Some(0.55)));
    p.print(pyf!("  {:>6} {:>8} {:>7} {:>8} {:>7} | {:>8} {:>7} {:>9} {:>7}",
                 "Tt4", "phi_L", "gap_L", "dphi_L", "frac", "phi_H", "gap_H", "dphi_H", "frac"));
    p.print(format!("  {}", dashes(76)));
    for tt4 in [1500.0, 1300.0, 1100.0, 950.0, 900.0] {
        let (a, c) = (fshut.match_point(flight, tt4), fopn.match_point(flight, tt4));
        let (g_l, g_h) = (a.base.phi_lp - 0.55, a.base.phi_hp - 0.55);
        let (dl, dh) = (c.base.phi_lp - a.base.phi_lp, c.base.phi_hp - a.base.phi_hp);
        p.print(pyf!("  {:6.0f} {:8.4f} {:7.4f} {:+8.4f} {:6.1f}% | {:8.4f} {:7.4f} {:+9.5f} {:6.2f}%",
                     tt4, a.base.phi_lp, g_l, dl, 100.0 * dl / g_l, a.base.phi_hp, g_h, dh, 100.0 * dh / g_h));
    }
    p.print("  => dphi_L is nearly CONSTANT (+-1%) while dphi_H collapses ~8x. A fixed absolute");
    p.print("     increment into a SHRINKING LP gap => the fraction closed RISES on the LP spool");
    p.print("     (17% -> 42%) and FALLS on the HP (1.8% -> 0.4%). That is the honest sense in");
    p.print("     which the device is SELF-TARGETING.");

    // --- the trade and the envelope.
    p.print("\n  THE TRADE. Thrust falls 10.0% -> 14.7% and TSFC rises 6.3% -> 14.6% (b = 0.10) as");
    p.print("  the throttle comes back: the valve gets MORE SELECTIVE and MORE EXPENSIVE together --");
    p.print("  which is why real bleed is SCHEDULED, not simply left open. And bleed lowers pi_LPC");
    p.print("  hence pt4, so it SHRINKS the choked envelope (lowest runnable Tt4 605 -> 630 K over");
    p.print("  b = 0 -> 0.15): the inherited nozzle-choke guard bites sooner. It flags, it never lies.");

    p.print("\n  A HYPOTHESIS, REFUTED and kept visible (rung 40's convention): this rung was");
    p.print("  proposed as 'bleed protects the LP AT THE HP SPOOL'S EXPENSE'. FALSE -- above pi* the");
    p.print("  HP flow coefficient RISES too, just 10-100x less; below pi* it falls, by ~1e-4. The");
    p.print("  textbook trade is not what the choked two-spool hardware does.");
    p.print("\n  NOT claimed: any magnitude (all ride on b, the representative maps and the two");
    p.print("  imposed floors); a surge-SURVIVAL claim (E0 vs SM_N needs the transient, deferred);");
    p.print("  a bleed SCHEDULE b(n_L); variable stators (they move phi_surge -- still open).");
    p.print("  Reduce: bleed=0 => rung 39 bit-for-bit by exact dispatch. Cycle stays rung-6 exact.");
}

/// `print_two_shaft_fuel_table(flight)` — rung 43.
pub fn two_shaft_fuel_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTwo-shaft fuel metering (rung 43): rung 35's control on rung 40's plant. Fuel is");
    p.print("metered and Tt4 FLOATS against the airflow two lagging spools can currently pump.");
    p.print("Rung 35's TIT-overshoot finding re-measures unchanged and is INHERITED, not this");
    p.print("rung's finding. What is new is a question ONE shaft cannot ask:");
    p.print("    f   = mdot_fuel / mdot_air        <- the LP FACE sets the airflow");
    p.print("    Tt4 = burner(Tt3, f)                 (Tt4 floats up as the LP lag spikes f)");
    p.print("    md4 = A4 pt4 MFP*(Tt4)/sqrt(Tt4)  <- the HP-FED NGV CHOKE meters it back");
    p.print("The two spools sit at DIFFERENT points in the ONE overshoot loop, so with two clocks");
    p.print("there is a RATIO rho = tau_L/tau_H and the question is: which spool's lag governs it?");

    let lp = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::default() };
    let hp = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::default() };
    let (lo, hi) = (1250.0, 1450.0);   // rung 35's own step -- apples-to-apples
    let ft = |rho: f64| TwoSpoolFuelTransient::new(ts_design(cpg13(), 3.0, 6.0, d), *flight, 1.0, lp, hp, rho);
    // `ramp_excursion_fuel(flight, LO, HI, r, freeze=…)` with Python's `s_settle=8.0, ds=0.02`.
    let rx = |f: &TwoSpoolFuelTransient, r: f64, freeze: Option<Spool>| {
        f.core().ramp_excursion_fuel(flight, lo, hi, r, freeze, 8.0, 0.02)
    };

    // --- the reduce, first: a steady point is the same however it is NAMED.
    p.print("\n  REDUCE -- CONTROL-INVARIANCE (the non-tautological gate). Feed the fuel of a rung-40");
    p.print("  Tt4-control point to the FUEL solver: it must return that point, via the forward");
    p.print("  BURNER (Tt4 an OUTPUT) -- a genuinely different code path. Two closures, one point:");
    let f0 = ft(1.0);
    p.print(pyf!("  {:>6} {:>10} {:>10} {:>10} {:>10}", "Tt4", "d nu_L", "d nu_H", "d Tt4", "d pi_LPC"));
    p.print(format!("  {}", dashes(50)));
    for tt4 in [1500.0, 1300.0, 1100.0] {
        let eq = f0.core().inner.equilibrium(flight, tt4);
        let fq = f0.core().equilibrium_fuel(flight, eq.close.f * eq.close.mdot_air, None).0.base;
        p.print(pyf!("  {:6.0f} {:+10.2e} {:+10.2e} {:+10.2e} {:+10.2e}",
                     tt4, fq.nu_lp / eq.nu_lp - 1.0, fq.nu_hp / eq.nu_hp - 1.0, fq.tt4 / tt4 - 1.0,
                     fq.close.pi_lpc / eq.close.pi_lpc - 1.0));
    }
    p.print("  => machine zero. This also KILLS, empirically, the framing this rung was proposed");
    p.print("     with -- 'fuel metering breaks rung 39's (dagger) and re-couples LP into the HP");
    p.print("     core'. If both controls land on the SAME manifold, the knob cannot change the");
    p.print("     coupling. (It was a category error anyway: (dagger) is a STEADY eta-fixed-point");
    p.print("     artifact that does not arise in the transient closure at all.)");

    // --- THE FINDING: channel isolation. Freeze one spool at a time.
    p.print("\n  THE FINDING -- CHANNEL ISOLATION (rung 41's move, applied to the transient): march");
    p.print("  the fuel ramp with ONE spool's speed HELD at its initial value. Tt4_peak [K]:");
    p.print(pyf!("  {:>5} {:>5} {:>10} {:>10} {:>10} {:>8} {:>8}",
                 "rho", "r", "both free", "LP frozen", "HP frozen", "d_LP", "d_HP"));
    p.print(format!("  {}", dashes(60)));
    for r in [0.25, 1.0] {
        for rho in [0.5, 1.0, 2.0] {
            let fc = ft(rho).core().freeze_channels(flight, lo, hi, r, 8.0, 0.02);
            p.print(pyf!("  {:5.1f} {:5.2f} {:10.1f} {:10.1f} {:10.1f} {:+8.1f} {:+8.1f}",
                         rho, r, fc.both, fc.lp, fc.hp, fc.d_lp, fc.d_hp));
        }
    }
    p.print("  => (1) freezing EITHER spool makes the overshoot WORSE, 6/6 -- both spools' motion");
    p.print("         RELIEVES it. Neither is a bystander.");
    p.print("     (2) the SHARE of the relief TRADES with rho: as the LP spool slows, the LP");
    p.print("         channel weakens and the HP channel strengthens.");
    p.print("     THAT is why no single spool's clock can govern the overshoot -- the responsibility");
    p.print("     for quenching it is SHARED and rho-DEPENDENT. Direction only: d_LP and d_HP do");
    p.print("     NOT sum to the total and are not calibrated weights.");

    // --- the bounded positive: monotone in rho, ceilinged by the LP-frozen march.
    p.print("\n  THE POSITIVE, AND ITS CEILING. X = Tt4_peak - Tt4_target rises monotonically with");
    p.print("  rho (a heavier LP spool worsens the TIT excursion -- the LP-face lag is what spikes");
    p.print("  f). It is BOUNDED, and the bound is STRUCTURAL: rho multiplies ONLY the LP ODE");
    p.print("  (dnu_L/ds = Phi_L/rho), so rho -> infinity IS the LP-frozen system:");
    let head: String = ["rho=0.25", "rho=1", "rho=4", "rho=8", "rho=32", "rho=128"].iter()
        .map(|h| pyf!("{:>9}", h)).collect();
    p.print(pyf!("  {:>5} {}{:>11}", "r", head, "LP-frozen"));
    p.print(format!("  {}", dashes(72)));
    for r in [0.25, 1.0] {
        let row: String = [0.25, 1.0, 4.0, 8.0, 32.0, 128.0].iter()
            .map(|&rho| pyf!("{:9.1f}", rx(&ft(rho), r, None).x)).collect();
        let ceil = rx(&ft(1.0), r, Some(Spool::Lp)).x;
        p.print(pyf!("  {:5.2f} {}{:11.1f}", r, row, ceil));
    }
    let a = rx(&ft(0.25), 0.25, Some(Spool::Lp)).x;
    let b = rx(&ft(50.0), 0.25, Some(Spool::Lp)).x;
    p.print("  => X(rho) converges UPWARD onto the LP-frozen march, which is rho-independent");
    p.print(pyf!("     BIT-FOR-BIT ({} == {} at rho = 0.25 vs 50: {}). So the worst TIT",
                 a.py_str(), b.py_str(), (a == b).py_str()));
    p.print("     excursion a heavy LP spool can produce is computable WITHOUT marching it.");

    // --- THE NEGATIVE: the currencies are circular.
    p.print("\n  THE NEGATIVE, STATED PLAINLY -- there is NO effective clock ratio r_eff = r/rho^q");
    p.print("  (q=0 => 'the HP clock governs', q=1 => 'the slow spool rate-limits'). The reason it");
    p.print("  APPEARED to exist is a trap worth recording: THE CURRENCIES ARE CIRCULAR -- the");
    p.print("  fitted exponent reads back whichever spool sits in the excursion's DENOMINATOR:");
    let mut pts = Vec::new();
    for rho in [0.25, 1.0, 4.0, 8.0] {
        let fx = ft(rho);
        for r in [0.25, 0.5, 1.0, 2.0] {
            let e = rx(&fx, r, None);
            if e.complete {
                pts.push((r, rho, e));
            }
        }
    }
    let col = |get: fn(&crate::fuel_transient::RampExcursionFuel) -> f64| -> Vec<(f64, f64, f64)> {
        pts.iter().map(|(r, rho, e)| (*r, *rho, get(e))).collect()
    };
    p.print(pyf!("  {:>10} {:>21} {:>8} {:>10}", "currency", "denominator", "best q", "residual"));
    p.print(format!("  {}", dashes(53)));
    let mut qs = std::collections::HashMap::new();
    let cols: [(&str, &str, fn(&crate::fuel_transient::RampExcursionFuel) -> f64); 3] = [
        ("E_temp_H", "nu_H running line", |e| e.e_temp_h),
        ("X", "none (spool-neutral)", |e| e.x),
        ("E_temp_L", "nu_L running line", |e| e.e_temp_l),
    ];
    for (key, den, get) in cols {
        let (q, s) = FuelTransientCore::collapse_exponent(&col(get), 6, None);
        qs.insert(key, (q, s));
        p.print(pyf!("  {:>10} {:>21} {:8.2f} {:10.3f}", key, den, q, s));
    }
    let xs = col(|e| e.x);
    let r0 = FuelTransientCore::collapse_exponent(&xs, 6, Some(0.0)).1;
    let r1 = FuelTransientCore::collapse_exponent(&xs, 6, Some(1.0)).1;
    p.print("  => the HP-REFERENCED currency reads far below the spool-neutral one. So E_temp's");
    p.print("     q ~ 0 was NEVER evidence that 'the HP clock governs' -- it was the reference");
    p.print("     reading itself back. Only X is spool-neutral, which is why every magnitude");
    p.print("     above is quoted in X: THE DATA SELECTED THE INSTRUMENT, not the answer it");
    p.print("     gave. And even on X there is NO collapse: the best exponent cuts the spread");
    p.print(pyf!("     ~{:.1f}x vs q=0 but bottoms out at {:.0f}% -- points a real clock would put",
                 r0 / qs["X"].1, 100.0 * qs["X"].1));
    p.print("     on ONE curve still differ by about a seventh. On the other shape pairs q*(X)");
    p.print("     and q*(E_temp_L) can TIE (press/flow: 0.45 = 0.45), so only the HP-vs-neutral");
    p.print("     separation is claimed -- not a strict three-way ordering.");

    p.print("\n  DELIBERATELY NOT CLAIMED (each was written, probed, and withdrawn):");
    p.print("    - 'it rides on the geometric-mean composite clock sqrt(det) ~ rho^(-1/2)'. DROPPED:");
    p.print("      sqrt(det)*sqrt(rho) = const IS a true rung-40 Jacobian identity, but it is not");
    p.print(pyf!("      connected to the overshoot -- q*(X)={:.2f} is the MIDPOINT of the two", qs["X"].0));
    p.print(pyf!("      circular currencies ({:.2f}, {:.2f}), an averaging artifact, not evidence for 1/2.",
                 qs["E_temp_H"].0, qs["E_temp_L"].0));
    p.print(pyf!("    - 'q=1 is refuted in every currency'. FALSE: on X, q=0 ({:.2f}) fits WORSE than", r0));
    p.print(pyf!("      q=1 ({:.2f}). Nothing about the exponent is currency-independent.", r1));
    p.print("    - 'the overshoot is irreducibly two-dimensional'. OVERCLAIM: only POWER-LAW");
    p.print("      collapses were tested. The honest statement is that rung 35's single-clock r");
    p.print("      framing does not extend to two shafts via any effective clock ratio.");
    p.print("\n  NOT claimed: any magnitude (all ride on rho -- a disclaimed clock group, DOUBLED --");
    p.print("  on the two representative maps and on the fuel step); a surge-SURVIVAL claim (no");
    p.print("  surge line on either spool in transient); a TIT redline. Reduce: lp_disabled => rung");
    p.print("  35 bit-for-bit; Tt4-control untouched => rung 40 bit-for-bit. Cycle stays rung-6 exact.");
}

/// `print_transient_surge_table(flight)` — rung 44.
pub fn transient_surge_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTransient two-spool surge line (rung 44): rungs 40 and 41 both deferred this in the");
    p.print("same words -- march rung 40's trajectory against rung 41's line. On an ACCEL the fuel/Tt4");
    p.print("step outruns the shaft inertia, so phi dips BELOW the steady running line -- toward surge.");

    let lp = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::default() };
    let hp = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::default() };
    let flat = ComponentMap::flat();
    let tt = |ml: ComponentMap, mh: ComponentMap, rho: f64| {
        TwoSpoolTransient::new(ts_design(cpg13(), 3.0, 6.0, d), *flight, 1.0, ml, mh, rho)
    };
    // `phi_excursion(flight, Tt4_lo, dTt4, r_ramp=0.5, s_end=3.0, ds=0.02)`'s defaults.
    let px = |t: &TwoSpoolTransient, r_ramp: f64, s_end: f64| {
        t.core().phi_excursion(flight, 1000.0, 400.0, r_ramp, s_end, 0.02)
    };

    // --- THE SPLIT SURVIVES DYNAMICALLY, and the mode is IRRELEVANT (hp-only is the tell).
    p.print("\n  THE SPLIT, DYNAMIC (accel Tt4 1000->1400). ext = extremum of phi(s)-phi_steady(Tt4),");
    p.print("  referenced to the running line. NEGATIVE = toward surge. hp-only (LP map FLAT) has NO");
    p.print("  complex mode -- yet the LARGEST LP/HP ratio, so the asymmetry is NOT the mode:");
    p.print(pyf!("  {:>12} {:>9} {:>9} {:>7} {:>7} {:>8}", "shape pair", "ext_lp", "ext_hp", "|L/H|", "band?", "|Im/Re|"));
    p.print(format!("  {}", dashes(56)));
    for (name, ml, mh) in [("flow/press", lp, hp), ("hp-only", flat, hp)] {
        let t = tt(ml, mh, 1.0);
        let e = px(&t, 0.5, 3.0);
        let band = t.core().oscillatory_band(flight, 1200.0, None);
        let dr = t.core().damping_ratio_max(flight, 1200.0, None);
        p.print(pyf!("  {:>12} {:+9.4f} {:+9.4f} {:7.2f} {:>7} {:8.4f}",
                     name, e.ext_lp, e.ext_hp, e.ratio, if band.is_some() { "yes" } else { "NONE" }, dr));
    }
    p.print("  => both spools toward surge, LP eats ~1.9-2.2x. The MODE-IRRELEVANCE claim rests on the");
    p.print("     DAMPING RATIO: every |Im/Re| < 0.25 (e-folds before a quarter cycle) -> the ring");
    p.print("     cannot cross a line the steady point clears. The mode-free pair eating the MOST is");
    p.print("     CORROBORATION (mode not necessary), not proof -- it also swaps LP shaped->flat.");

    // --- SCHEDULE-SLAVED: rho-invariant but ramp-rate-driven.
    p.print("\n  SCHEDULE-SLAVED (flow/press). Over a 25x rho range the excursion barely moves; over");
    p.print("  the ramp rate it moves ~5x. rho (which spool LEADS) is powerless; the SLAM RATE governs:");
    let head = |hs: [&str; 3]| -> String { hs.iter().map(|h| pyf!("{:>9}", h)).collect() };
    p.print(pyf!("  {:>14}{}", "", head(["rho=0.2", "rho=1.0", "rho=5.0"])));
    let row: String = [0.2, 1.0, 5.0].iter()
        .map(|&r| pyf!("{:9.4f}", px(&tt(lp, hp, r), 0.5, 3.0).ext_lp)).collect();
    p.print(pyf!("  {:>14}{}   <- <2% spread", "ext_lp:", row));
    let t = tt(lp, hp, 1.0);
    p.print(pyf!("  {:>14}{}", "", head(["r=0.1", "r=0.5", "r=2.0"])));
    let row: String = [0.1, 0.5, 2.0].iter()
        .map(|&r| pyf!("{:9.4f}", px(&t, r, 6.0).ext_lp)).collect();
    p.print(pyf!("  {:>14}{}   <- faster => deeper", "ext_lp:", row));

    // --- REPORT THE CROSSING, GATE THE FLIP.
    p.print("\n  REPORT THE CROSSING, GATE THE FLIP (rung 36's discipline). Arm phi_surge and place a");
    p.print("  floor in the gap: the steady point CLEARS it, the transient CROSSES -- on the LP spool:");
    let ta = tt(lp.with_phi_surge(0.76), hp.with_phi_surge(0.55), 1.0);
    let m = ta.core().transient_surge_margin(flight, 1000.0, 400.0, 0.3, 3.0, 0.02);
    p.print(pyf!("    steady min LP margin = {:+.4f}  (clears the phi_surge=0.76 floor)", m.steady_min_lp));
    p.print(pyf!("    transient min LP     = {:+.4f}  crossed_lp={} crossed_hp={}",
                 m.margin_min_lp, m.crossed_lp.py_str(), m.crossed_hp.py_str()));
    p.print("  The crossing DEPTH rides on the imposed floor + the ramp (disclaimed); the flip's SIGN");
    p.print("  (transient below steady, on the LP spool) is the gated object. NO survival claim.");
    p.print("\n  NOT claimed: any magnitude (phi_surge imposed, DOUBLED; excursion depths ride on the");
    p.print("  maps + the ramp); the mode's irrelevance is only at |Im/Re|<=0.164, not universal.");
    p.print("  Reduce: the methods only READ -> rung 40 integrate/equilibrium/jacobian bit-for-bit;");
    p.print("  Tt4-control (fuel path is the extension). Cycle stays rung-6 exact.");
}

/// `print_transient_fuel_surge_table(flight)` — rung 45.
pub fn transient_fuel_surge_table(p: &mut Printer, d: &Design) {
    let flight = &d.flight;
    p.print("\nTransient surge on the FUEL path (rung 45): rung 44's diagnostic on rung 43's plant,");
    p.print("where Tt4 FLOATS and OVERSHOOTS. The overshoot is rho-loud; the surge object is rho-quiet.");

    let lp = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::default() };
    let hp = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::default() };
    // Python builds `design` ONCE and hands it to every constructor; each only reads it.
    let design = ts_design(cpg13(), 3.0, 6.0, d);
    let ft = |ml: ComponentMap, mh: ComponentMap, rho: f64| {
        TwoSpoolFuelTransient::new(design.clone(), *flight, 1.0, ml, mh, rho)
    };
    // `phi_excursion_fuel(flight, Tt4_lo, Tt4_hi, r, s_settle=6.0, ds=0.02)` and no limiters.
    let pxf = |f: &TwoSpoolFuelTransient, r: f64| {
        f.phi_excursion_fuel(flight, 1000.0, 1400.0, r, 6.0, 0.02, None, None, None, None)
    };
    let head = |hs: [&str; 3]| -> String { hs.iter().map(|h| pyf!("{:>10}", h)).collect() };

    // --- THE HEADLINE: the currency trap.
    p.print("\n  THE CURRENCY TRAP (flow/press, accel Tt4 1000->1400, r=0.5). Sweep rho: the Tt4 PEAK");
    p.print("  (the plant) swings hard, the RAW min phi (the surge object) barely moves:");
    p.print(pyf!("  {:>12}{}", "", head(["rho=0.2", "rho=1.0", "rho=5.0"])));
    let (mut peaks, mut mins) = (Vec::new(), Vec::new());
    for r in [0.2, 1.0, 5.0] {
        let e = pxf(&ft(lp, hp, r), 0.5);
        peaks.push(e.tt4_peak);
        mins.push(e.min_phi_lp);
    }
    let peaks: String = peaks.iter().map(|x| pyf!("{:10.1f}", x)).collect();
    let mins: String = mins.iter().map(|x| pyf!("{:10.4f}", x)).collect();
    p.print(pyf!("  {:>12}{}   <- ~12% (rho-LOUD)", "Tt4_peak:", peaks));
    p.print(pyf!("  {:>12}{}   <- <1% (rho-QUIET)", "min_phi_lp:", mins));
    p.print("  => rung 43's rho-monotone overshoot NEVER reaches the surge object. rung 44's 'rho");
    p.print("     powerless over surge' SURVIVES on the reference-free object -- the currency you pick");
    p.print("     (output-referenced excursion would read ~40%!) decides whether rho appears to matter.");

    // --- FUEL ENLARGES the approach vs Tt4 control (rung 35 on two shafts).
    p.print("\n  FUEL ENLARGES the approach (rung 35, two shafts). Same endpoints + ramp, raw min phi_lp:");
    let tt4 = TwoSpoolTransient::new(design.clone(), *flight, 1.0, lp, hp, 1.0);
    p.print(pyf!("  {:>12}{}", "", head(["r=1.0", "r=0.5", "r=0.3"])));
    let frow: String = [1.0, 0.5, 0.3].iter()
        .map(|&r| pyf!("{:10.4f}", pxf(&ft(lp, hp, 1.0), r).min_phi_lp)).collect();
    let trow: String = [1.0, 0.5, 0.3].iter()
        .map(|&r| pyf!("{:10.4f}", tt4.core().phi_excursion(flight, 1000.0, 400.0, r, 3.0, 0.02).min_phi_lp))
        .collect();
    p.print(pyf!("  {:>12}{}   <- deeper toward surge (Tt4 overshoot amplifies)", "fuel:", frow));
    p.print(pyf!("  {:>12}{}", "Tt4-ctrl:", trow));

    // --- REPORT THE CROSSING, GATE THE FLIP (accel; the raw object is degenerate on a decel).
    p.print("\n  REPORT THE CROSSING, GATE THE FLIP (rung 36, on the ACCEL). Arm phi_surge, floor in gap:");
    let fa = ft(lp.with_phi_surge(0.746), hp.with_phi_surge(0.55), 1.0);
    let m = fa.transient_surge_margin_fuel(flight, 1000.0, 1400.0, 0.3, 6.0, 0.02, None, None, None, None);
    p.print(pyf!("    steady min LP margin = {:+.4f}  (clears the phi_surge=0.746 floor)", m.steady_min_lp));
    p.print(pyf!("    transient min LP     = {:+.4f}  crossed_lp={} crossed_hp={}",
                 m.margin_min_lp, m.crossed_lp.py_str(), m.crossed_hp.py_str()));
    p.print("  The LP crosses while the HP clears wide (the strong asymmetry; the excursion RATIO");
    p.print("  compresses to ~1.2-1.7 vs rung 44's 1.6-2.2). NO survival claim -- phi_surge imposed,");
    p.print("  tripled. Reduce: reads integrate_fuel -> rung 43 bit-for-bit; cycle stays rung-6 exact.");
}
