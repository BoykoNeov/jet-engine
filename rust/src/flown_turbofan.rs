//! RUNG 110 — THE TURBOFAN FLOWN: rung 109's fixed turbofan off design, on the two-shaft engine's own solver.
//!
//! **The rung** (`docs/rung110-spec.md`, anchor `docs/plans/rung110-anchor-flown-turbofan.md`). Rung 109 designed a
//! turbofan at one flight and one throttle. Freeze its hardware — the turbine and nozzle throats — and fly it: the
//! throttle and the flight move, and the engine finds its own pressure ratios, shaft speeds and, new here, its own
//! BYPASS RATIO. Two fixed nozzles share the fan's air, so how much goes around the core is no longer a knob.
//!
//! **One solver (the user's decision, twice).** There is no turbofan loop here. The bypass stream lives INSIDE the
//! two-shaft engine (`two_spool.rs`: [`Bypass`] on the design engine, [`TwoSpoolCore::try_fan_split`] in rung 39's
//! cascade, the fan nozzle in its rebuild and scoring), so every engine flies through rung 39's matcher on flat maps
//! (≡ rung 38, the textbook method — efficiencies held at design):
//!
//! * **no fan, π_f > 1** — the two-shaft engine, line for line;
//! * **no fan, π_f = 1** — one shaft, on rung 31's matcher (the anchor's `TwoSpoolMatcher::lp_disabled` → rung 31 route;
//!   not rung 32 on a flat map, which has no subsonic branch below choke — [`Shafts::One`]);
//! * **a fan** — the LP turbine pays for the core air through the fan root and the bypass air through the tip, and the
//!   bypass ratio is the fan nozzle's flow over the core's.
//!
//! What this module adds is the bridge from rung 109: its one compressor and one lumped turbine split onto two shafts
//! (anchor § 1 D1 — POLYTROPICALLY, so stations 3 and 5 are rung 109's and no derived efficiency exceeds 1), and the
//! readout. Shaft speeds are the anchor's frozen D8 (Mattingly's 8-52ag/ah on the table gases, `√(Δh_s/Δh_s,design)`);
//! the matcher's own flat-map speeds are kept beside them, printed, never scored.
//!
//! [`Bypass`]: crate::two_spool::Bypass
//! [`TwoSpoolCore::try_fan_split`]: crate::two_spool::TwoSpoolCore::try_fan_split

use crate::components::{Component, NozzleExit};
use crate::engine::{EngineResult, FlightCondition, Performance};
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::map::ComponentMap;
use crate::matcher::OffDesignMatcher;
use crate::turbofan::Turbofan;
use crate::two_spool::{build_two_spool_turbojet, Bypass, FanOut, IdleRule, TwoSpoolCore, TwoSpoolEngine,
                       TwoSpoolLosses, TwoSpoolMapMatcher};

/// The flown turbofan's components, every efficiency ISENTROPIC (the two-shaft engine takes no other).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FanComponents {
    pub pi_d: f64,
    /// Fan pressure ratio — both streams; the two-shaft engine's LP compressor ratio.
    pub pi_f: f64,
    /// The fan ROOT's efficiency on the core air — the LP compressor's (D1: the compressor's own, split polytropically).
    pub eta_root: f64,
    /// The fan TIP's efficiency on the bypass air.
    pub eta_f: f64,
    pub pi_hpc: f64,
    pub eta_hpc: f64,
    pub eta_b: f64,
    pub pi_b: f64,
    pub eta_hpt: f64,
    pub eta_lpt: f64,
    /// Mechanical efficiency of each shaft (rung 109 has one `η_m` on both; Mattingly's Example 8-8 has two).
    pub eta_m_hp: f64,
    pub eta_m_lp: f64,
    pub pi_n: f64,
    /// Bypass duct + fan nozzle total-pressure ratio (13 → 19).
    pub pi_fn: f64,
    /// DESIGN bypass ratio (off design an output).
    pub alpha: f64,
    /// DESIGN turbine-inlet temperature.
    pub tt4: f64,
}

impl FanComponents {
    /// The two-shaft design engine these components make, with its fan fitted when `alpha > 0`, against `p0`.
    pub fn engine(&self, gas: &Gas, p0: f64) -> TwoSpoolEngine {
        let e = build_two_spool_turbojet(gas.clone(), self.pi_f, self.pi_hpc, self.tt4, p0, TwoSpoolLosses {
            pi_d: self.pi_d, eta_lpc: self.eta_root, eta_hpc: self.eta_hpc, eta_b: self.eta_b, pi_b: self.pi_b,
            eta_hpt: self.eta_hpt, eta_lpt: self.eta_lpt, eta_m: self.eta_m_hp, pi_n: self.pi_n, p_exit: None,
            nozzle_convergent: true,
        });
        if self.alpha > 0.0 {
            e.with_fan(Bypass { alpha: self.alpha, eta_f: self.eta_f, pi_fn: self.pi_fn, eta_m_lp: self.eta_m_lp })
        } else {
            assert!(self.eta_m_lp == self.eta_m_hp, "with no fan the two-shaft engine carries one mechanical efficiency");
            e
        }
    }
}

/// D1 — rung 109's design split onto two shafts: the compressor at `π_f`, the lumped turbine at the HP shaft's work, both
/// on their POLYTROPIC paths (`e_c = ln π_c / ln(pr3/pr2)`, `e_t = ln(pr5/pr4) / ln(pt5/pt4)`, exact on the tables), then
/// each part's isentropic efficiency read off its states. Rung 109's design must run and end in the fixed convergent
/// nozzle (rung 30) — the throat the flown engine keeps.
pub fn split_rung109(tf: &Turbofan, flight: &FlightCondition, mdot: f64) -> Result<FanComponents, Abort> {
    let eng = &tf.engine;
    let gas = &eng.gas;
    let by = |label: &str| eng.components.iter().find(|(l, _)| *l == label).map(|&(_, c)| c)
        .unwrap_or_else(|| panic!("a turbojet has a component at station {label}"));
    let (pi_d, cmp, burner, nozzle) = match (by("2"), by("3"), by("4"), by("9")) {
        (Component::Inlet(i), Component::Compressor(c), Component::Burner(b), Component::Nozzle(n)) => (i.pi_d, c, b, n),
        _ => panic!("a turbojet is inlet, compressor, burner, turbine, nozzle"),
    };
    assert!(nozzle.convergent, "rung 110 flies the FIXED CONVERGENT nozzle (rung 30): design the turbofan with it");
    let r = tf.try_run(flight, mdot)?;
    let (s2, s3, s4, s5) = (*r.core.station("2"), *r.core.station("3"), *r.core.station("4"), *r.core.station("5"));
    let f = s4.far;
    let pi_f = tf.fan.pi_f;
    let pi_hpc = cmp.pi_c / pi_f;
    let dhs = |tt: f64, pi: f64| gas.h_c(gas.t_from_pr_c(gas.pr_c(tt) * pi)) - gas.h_c(tt);
    // The compressor, split at pi_f on its polytropic path.
    let e_c = match cmp.e_c {
        Some(e) => e,
        None if cmp.eta_c == 1.0 => 1.0,
        None => cmp.pi_c.ln() / (gas.pr_c(s3.tt) / gas.pr_c(s2.tt)).ln(),
    };
    let tt25 = gas.t_from_pr_c(gas.pr_c(s2.tt) * powp(pi_f, 1.0 / e_c));
    let (h2, h25, h3) = (gas.h_c(s2.tt), gas.h_c(tt25), gas.h_c(s3.tt));
    let eta_root = if pi_f == 1.0 { 1.0 } else { dhs(s2.tt, pi_f) / (h25 - h2) };
    let eta_hpc = dhs(tt25, pi_hpc) / (h3 - h25);
    // The fan tip on the bypass air: rung 109's own fan knob, read as isentropic.
    let eta_f = match (tf.fan.alpha > 0.0, r.fan) {
        (true, Some(fs)) => dhs(s2.tt, pi_f) / (gas.h_c(fs.s13.tt) - h2),
        _ => match tf.fan.e_f {
            None => tf.fan.eta_f,
            Some(e) => dhs(s2.tt, pi_f) / (gas.h_c(gas.t_from_pr_c(gas.pr_c(s2.tt) * powp(pi_f, 1.0 / e))) - h2),
        },
    };
    // The lumped turbine, split at the HP shaft's work on its polytropic path.
    let e_t = (gas.pr_t(s5.tt, f) / gas.pr_t(s4.tt, f)).ln() / (s5.pt / s4.pt).ln();
    let h4 = gas.h_t(s4.tt, f);
    let h45 = h4 - (h3 - h25) / (eng.eta_m * (1.0 + f));
    let tt45 = gas.t_from_h_t(h45, f);
    let pt45 = s4.pt * powp(gas.pr_t(tt45, f) / gas.pr_t(s4.tt, f), 1.0 / e_t);
    let eta_hpt = (h4 - h45) / (h4 - gas.h_t(gas.t_from_pr_t(gas.pr_t(s4.tt, f) * (pt45 / s4.pt), f), f));
    let h5 = gas.h_t(s5.tt, f);
    let eta_lpt = (h45 - h5) / (h45 - gas.h_t(gas.t_from_pr_t(gas.pr_t(tt45, f) * (s5.pt / pt45), f), f));
    Ok(FanComponents {
        pi_d, pi_f, eta_root, eta_f, pi_hpc, eta_hpc, eta_b: burner.eta_b, pi_b: burner.pi_b, eta_hpt, eta_lpt,
        eta_m_hp: eng.eta_m, eta_m_lp: eng.eta_m, pi_n: nozzle.pi_n, pi_fn: tf.fan.pi_fn, alpha: tf.fan.alpha,
        tt4: burner.tt4,
    })
}

/// The fixed turbofan, ready to fly — the two-shaft matcher on flat maps.
/// The solver a flown design runs on.
pub enum Shafts {
    /// A fan (`π_f > 1`, bypass or not): rung 39's map matcher on flat maps (≡ rung 38), the bypass inside it.
    Two(TwoSpoolMapMatcher),
    /// No fan rise and no bypass: ONE shaft, on rung 31's matcher — the anchor's route (`TwoSpoolMatcher::lp_disabled`
    /// → rung 31), which dispatches to rung 33's subsonic branch below choke. NOT rung 32 on a flat map: rung 32 never
    /// gained that dispatch (`map.rs` § 3, kept by its own gate), so below choke it would solve a choked nozzle.
    One(OffDesignMatcher),
}

pub struct FlownTurbofan {
    pub matcher: Shafts,
    pub components: FanComponents,
    pub flight_design: FlightCondition,
    pub mdot_design: f64,
    /// The cold gas the shaft speeds are read on (D8).
    pub gas: Gas,
    /// D8's design isentropic rises, J/kg: the fan on station 2 at `pi_f`, the HP compressor on station 25 at `pi_hpc`
    /// (on one shaft: NaN, and the compressor on station 2 at `pi_c`).
    pub dhs_design: [f64; 2],
}

/// D8 — the isentropic rise `Δh_s` of a compressor at ratio `pi` from inlet total temperature `tt`, J/kg (cold gas).
pub fn dhs(gas: &Gas, tt: f64, pi: f64) -> f64 { gas.h_c(gas.t_from_pr_c(gas.pr_c(tt) * pi)) - gas.h_c(tt) }

/// One flown operating point.
#[derive(Clone, Debug)]
pub struct FlownPoint {
    pub tt4: f64,
    pub m0: f64,
    /// The bypass ratio — an OUTPUT (0 with no fan).
    pub alpha: f64,
    /// Fan (first compressor) and HP compressor pressure ratios — OUTPUTS (`pi_f` is 1 on a single shaft).
    pub pi_f: f64,
    pub pi_hpc: f64,
    /// Core stations 0 2 (25) 3 4 (45) 5 9.
    pub stations: Vec<(&'static str, FlowState)>,
    /// The bypass stream (`None` with no fan).
    pub fan: Option<FanOut>,
    /// Per TOTAL air.
    pub performance: Performance,
    /// Net thrust (both jets), N.
    pub thrust: f64,
    /// Core airflow, kg/s.
    pub mdot_core: f64,
    /// Core net thrust per kilogram of CORE air, N·s/kg (it can be negative: the fan then carries the engine).
    pub core_thrust: f64,
    pub core_exit: NozzleExit,
    /// Shaft speeds as fractions of design, the anchor's FROZEN definition D8 (Mattingly 8-52ag/ah on the table gases):
    /// `N/N_design = √(Δh_s/Δh_s,design)`, the fan on station 2, the HP compressor on station 25 (`n_fan` NaN on one
    /// shaft). The predictions are scored on these.
    pub n_fan: f64,
    pub n_hp: f64,
    /// The matcher's own speeds (rung 39's corrected-speed lines on a flat map: `√((τ−1)/(τ_d−1))·√(Tt_in/Tt_in,d)`),
    /// printed beside D8, never scored.
    pub n_fan_map: f64,
    pub n_hp_map: f64,
}

impl FlownPoint {
    pub fn station(&self, label: &str) -> &FlowState {
        self.stations.iter().find(|&&(l, _)| l == label).map(|(_, s)| s)
            .unwrap_or_else(|| panic!("station {label:?} not reached"))
    }
    /// Total airflow (core + bypass), kg/s.
    pub fn mdot_total(&self) -> f64 { self.mdot_core * (1.0 + self.alpha) }
    /// Is the core nozzle choked (the two-shaft tails' own test)?
    pub fn core_choked(&self, p0: f64) -> bool { self.core_exit.p9 > p0 + 1e-6 }
    /// Is the fan nozzle choked? `None` with no fan.
    pub fn fan_choked(&self, p0: f64) -> Option<bool> { self.fan.map(|f| f.exit.p9 > p0 + 1e-6) }
}

/// Fly rung 109's turbofan: its design split onto two shafts (D1), then the hardware frozen.
pub fn fly_rung109(tf: &Turbofan, flight: &FlightCondition, mdot: f64) -> Result<FlownTurbofan, Abort> {
    if tf.fan.alpha > 0.0 && tf.fan.pi_f <= 1.0 {
        return Err(Abort(format!("rung-110: a bypass stream needs a fan pressure rise — a fan pressure ratio of {:.3} \
                                  with bypass ratio {} has nothing to drive air around the core; set the fan pressure \
                                  ratio above 1 or the bypass ratio to 0", tf.fan.pi_f, tf.fan.alpha)));
    }
    let c = split_rung109(tf, flight, mdot)?;
    if c.alpha == 0.0 && c.pi_f == 1.0 {
        // One shaft: rung 31's matcher on rung 109's engine (the turbojet), the standing engine on as the two-shaft
        // route has it.
        let m = OffDesignMatcher::new(tf.engine.clone(), *flight, mdot);
        m.standing.set(true);
        let matcher = Shafts::One(m);
        let gas = tf.engine.gas.clone();
        let tt2 = tf.try_run(flight, mdot)?.core.station("2").tt;
        let dhs_design = [f64::NAN, dhs(&gas, tt2, c.pi_hpc)];
        return Ok(FlownTurbofan { matcher, components: c, flight_design: *flight, mdot_design: mdot, gas, dhs_design });
    }
    Ok(FlownTurbofan::new(&tf.engine.gas, c, flight, mdot))
}

impl FlownTurbofan {
    /// Freeze the hardware of the two-shaft turbofan `c` designed at `flight` with CORE airflow `mdot` (`π_f > 1`). The
    /// page switches are on: the free LP turbine below choke (rung 103), the standing engine (A3), below idle refused.
    pub fn new(gas: &Gas, c: FanComponents, flight: &FlightCondition, mdot: f64) -> Self {
        assert!(c.pi_f > 1.0, "a fan with no pressure rise is the single-shaft engine: build it through fly_rung109");
        let matcher = TwoSpoolMapMatcher::new(c.engine(gas, flight.p0), *flight, mdot, ComponentMap::flat(),
                                              ComponentMap::flat());
        let b = &matcher.core().base;
        b.below_choke.set(true);
        b.standing.set(true);
        b.idle_rule.set(IdleRule::Refuse);
        let dhs_design = [dhs(gas, b.reference.station("2").tt, c.pi_f),
                          dhs(gas, b.reference.station("25").tt, c.pi_hpc)];
        FlownTurbofan { matcher: Shafts::Two(matcher), components: c, flight_design: *flight, mdot_design: mdot,
                        gas: gas.clone(), dhs_design }
    }

    /// The two-shaft core (`None` on a single shaft).
    pub fn core(&self) -> Option<&TwoSpoolCore> {
        match &self.matcher {
            Shafts::Two(TwoSpoolMapMatcher::Full(m)) => Some(&m.base),
            _ => None,
        }
    }

    /// The two-shaft core, mutably — to perturb a component ON THE FROZEN HARDWARE (P4 and gate 7 lower `eta_hpc` this
    /// way, as rung 38's gate 3 does; rebuilding the engine would re-capture every throat at a new design).
    pub fn core_mut(&mut self) -> Option<&mut TwoSpoolCore> {
        match &mut self.matcher {
            Shafts::Two(TwoSpoolMapMatcher::Full(m)) => Some(&mut m.base),
            _ => None,
        }
    }

    /// The design run the hardware was captured from (`None` on a single shaft).
    pub fn design(&self) -> Option<&EngineResult> { self.core().map(|b| &b.reference) }

    /// Switch the LP turbine's flow law (rung 104) — the book gate flies the pin, the book's own assumption.
    pub fn set_turbine_law(&self, law: crate::turbine_law::TurbineLaw) {
        if let Some(b) = self.core() { b.turbine_law.set(law); }
    }

    /// Fly one point.
    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> FlownPoint {
        self.try_match(flight, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// Fly one point, fallibly (the single-shaft route is rung 31's infallible matcher: its failures panic in its words).
    pub fn try_match(&self, flight: &FlightCondition, tt4: f64) -> Result<FlownPoint, Abort> {
        match &self.matcher {
            Shafts::One(m) => {
                let b = &m.match_point(flight, tt4);
                let s9 = *b.stations.iter().find(|(l, _)| *l == "9").map(|(_, s)| s).expect("station 9");
                Ok(FlownPoint {
                    tt4, m0: flight.m0, alpha: 0.0, pi_f: 1.0, pi_hpc: b.pi_c, stations: b.stations.clone(), fan: None,
                    performance: b.performance, thrust: b.thrust, mdot_core: b.mdot_air,
                    core_thrust: b.performance.specific_thrust,
                    core_exit: NozzleExit { state: s9, m9: b.m9, t9: b.t9, v9: b.v9, p9: b.p9 },
                    n_fan: f64::NAN,
                    n_hp: powp(dhs(&self.gas, b.stations.iter().find(|(l, _)| *l == "2").expect("station 2").1.tt, b.pi_c)
                               / self.dhs_design[1], 0.5),
                    n_fan_map: f64::NAN, n_hp_map: f64::NAN, // rung 31 computes no speed (no map)
                })
            }
            Shafts::Two(TwoSpoolMapMatcher::Degenerate(_)) => unreachable!("a two-shaft flown engine is never degenerate"),
            Shafts::Two(TwoSpoolMapMatcher::Full(m)) => {
                let r = m.try_match_point(flight, tt4)?;
                let b = r.base;
                let s9 = *b.stations.iter().find(|(l, _)| *l == "9").map(|(_, s)| s).expect("station 9");
                let tt_at = |l: &str| b.stations.iter().find(|(x, _)| *x == l).map(|(_, s)| s.tt).expect("station");
                let n_fan = powp(dhs(&self.gas, tt_at("2"), b.pi_lpc) / self.dhs_design[0], 0.5);
                let n_hp = powp(dhs(&self.gas, tt_at("25"), b.pi_hpc) / self.dhs_design[1], 0.5);
                Ok(FlownPoint {
                    tt4, m0: flight.m0, alpha: b.fan.map_or(0.0, |f| f.alpha), pi_f: b.pi_lpc, pi_hpc: b.pi_hpc,
                    core_thrust: b.fan.map_or(b.performance.specific_thrust, |f| f.core_thrust),
                    core_exit: NozzleExit { state: s9, m9: b.m9, t9: b.t9, v9: b.v9, p9: b.p9 },
                    stations: b.stations, fan: b.fan, performance: b.performance, thrust: b.thrust,
                    mdot_core: b.mdot_air, n_fan, n_hp, n_fan_map: r.n_lp_ratio, n_hp_map: r.n_hp_ratio,
                })
            }
        }
    }
}
