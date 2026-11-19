//! RUNG 109 — THE TURBOFAN: a fan whose air mostly goes around the core, on the design point.
//!
//! **The rung** (`docs/rung109-spec.md`, anchor `docs/plans/rung109-anchor-turbofan.md`). A turbojet makes its thrust
//! from a small, fast, hot jet, and the faster the jet runs past the flight speed the more of the fuel's work it throws
//! away as jet energy left behind in the air. A fan takes some of the turbine's work and hands it to a much larger
//! stream of air that goes AROUND the core: more air, more slowly — less energy wasted for the same push.
//!
//! Stations (Mattingly, *Elements of Gas Turbine Propulsion*, Fig. 7-11): the core 0 → 2 → 3 → 4 → 5 → 9 as the
//! turbojet's; the bypass air 2 → **13** (fan exit) → **19** (fan-nozzle exit). The core air passes the fan root
//! inside the compressor's pressure ratio (2 → 3 is the WHOLE core compression, the book's convention).
//!
//! **The shaft — ONE lumped turbine** pays for the compressor AND the fan (per unit core air):
//!
//! ```text
//! Δh_turbine = [ h_c(Tt3) − h_c(Tt2)  +  α·( h_c(Tt13) − h_c(Tt2) ) ] / ( η_m·(1+f) )
//! ```
//!
//! — the turbojet's balance plus the fan's work on `α` kilograms of bypass air per kilogram of core air. On the perfect
//! gas this is Mattingly's (7-52o) exactly.
//!
//! **The fan nozzle is COLD.** The shipped nozzle reads the hot section at the burned fuel/air ratio; the bypass air
//! never meets fuel, so [`try_cold_nozzle`] is the same three laws on the cold section, in the core nozzle's mode
//! (fully expanded, a set exit pressure, or convergent — the user's choice: the fan nozzle follows the core's).
//!
//! **Thrust per TOTAL air** `ṁ0 = (1+α)·ṁ_core`: `F/ṁ0 = [F_core + α·F_fan]/(1+α)`, `TSFC = f/[(1+α)·F/ṁ0]`; thermal
//! efficiency = the kinetic energy added to BOTH jets per fuel energy; overall = thrust power per fuel power
//! (thrust-based, so a jet leaving off the outside pressure is counted); propulsive = overall ÷ thermal.
//!
//! Reduces: **α = 0 is `build_turbojet(…).run(…)` itself, by DISPATCH** — no fan arithmetic is entered, so every number
//! is the turbojet's bit for bit.

use crate::components::{Component, Compressor, Nozzle, NozzleExit};
use crate::engine::{build_turbojet, Engine, EngineResult, FlightCondition, Losses, Performance};
use crate::gas::{powp, Abort, FlowState, Gas};

/// The fan's knobs (`docs/rung109-spec.md` § The device). `alpha = 0` (the default) is the turbojet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fan {
    /// Bypass ratio: kilograms of air around the core per kilogram through it.
    pub alpha: f64,
    /// Fan total-pressure ratio (station 2 → 13).
    pub pi_f: f64,
    /// Fan ISENTROPIC efficiency — used when `e_f` is `None`.
    pub eta_f: f64,
    /// Fan POLYTROPIC efficiency. Mutually exclusive with `eta_f`, as the compressor's two are.
    pub e_f: Option<f64>,
    /// Bypass duct + fan nozzle total-pressure ratio (13 → 19).
    pub pi_fn: f64,
}

impl Default for Fan {
    fn default() -> Self { Fan { alpha: 0.0, pi_f: 1.6, eta_f: 0.88, e_f: None, pi_fn: 0.98 } }
}

/// The bypass stream's exit (station 19): totals plus the jet's statics.
#[derive(Clone, Copy, Debug)]
pub struct FanStream {
    /// Fan exit totals (station 13), `mdot = α·ṁ_core`.
    pub s13: FlowState,
    /// The fan nozzle: station-19 totals and the jet's exit statics.
    pub exit: NozzleExit,
}

/// One turbofan design run.
#[derive(Clone, Debug)]
pub struct TurbofanResult {
    pub alpha: f64,
    /// The core: stations 0 2 3 4 5 9 and its exit; `performance` is per TOTAL air (at α = 0, the turbojet's own).
    pub core: EngineResult,
    /// The bypass stream; `None` at α = 0.
    pub fan: Option<FanStream>,
    /// Core thrust per kilogram of CORE air, N·s/kg (the turbojet's specific thrust at α = 0).
    pub core_thrust: f64,
    /// Fan thrust per kilogram of BYPASS air, N·s/kg (0 at α = 0).
    pub fan_thrust: f64,
}

impl TurbofanResult {
    /// Thrust per kilogram of CORE air — what a fan adds to the same core.
    pub fn thrust_per_core(&self) -> f64 { self.core.performance.specific_thrust * (1.0 + self.alpha) }
    /// The core's fuel/air ratio.
    pub fn f(&self) -> f64 { self.core.station("4").far }
}

/// The core run up to the burner exit (stations 0 2 3 4) — the part a fan does not change, computed once and shared by
/// every fan setting a search or a chart tries.
#[derive(Clone, Debug)]
pub struct CoreTo4 {
    pub stations: Vec<(&'static str, FlowState)>,
    pub v0: f64,
    pub flight: FlightCondition,
}

/// The engine: the turbojet's five components (unchanged) plus a fan.
#[derive(Clone)]
pub struct Turbofan {
    pub engine: Engine,
    pub fan: Fan,
}

/// Factory: the turbojet of [`build_turbojet`] with a fan beside its compressor.
pub fn build_turbofan(gas: Gas, pi_c: f64, tt4: f64, p_ambient: f64, losses: Losses, fan: Fan) -> Turbofan {
    Turbofan { engine: build_turbojet(gas, pi_c, tt4, p_ambient, losses), fan }
}

/// The refusal when the turbine cannot pay for the fan (the core jet left with no pressure, or the turbine exit
/// colder than the outside air).
pub const NO_CORE_PRESSURE: &str = "The turbine cannot pay for this fan: after driving the compressor and the fan, the \
    core jet has no pressure left to leave the nozzle. Lower the bypass ratio or the fan pressure ratio.";

/// Is this one of the rung's own refusals (plain words for the page)?
pub fn is_refusal(e: &Abort) -> bool {
    e.0 == NO_CORE_PRESSURE || e.0.starts_with("The fan's pressure rise")
}

fn station<'a>(stations: &'a [(&'static str, FlowState)], label: &str) -> &'a FlowState {
    stations.iter().find(|&&(l, _)| l == label).map(|(_, s)| s)
        .unwrap_or_else(|| panic!("station {label:?} not reached yet"))
}

impl Turbofan {
    /// The core nozzle as built (its mode is the fan nozzle's too).
    fn core_nozzle(&self) -> Nozzle {
        match self.engine.components.last() {
            Some(&(_, Component::Nozzle(n))) => n,
            _ => panic!("a turbojet ends in its nozzle"),
        }
    }

    /// Run the design point. `mdot` is the CORE airflow; the bypass flow is `α·mdot`.
    pub fn run(&self, flight: &FlightCondition, mdot: f64) -> TurbofanResult {
        self.try_run(flight, mdot).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The fallible [`run`](Self::run): the fan's walls come back as [`Abort`]s in plain words.
    pub fn try_run(&self, flight: &FlightCondition, mdot: f64) -> Result<TurbofanResult, Abort> {
        if self.fan.alpha == 0.0 {
            // THE REDUCE, by dispatch: the turbojet's own run, untouched.
            let core = self.engine.run(flight, mdot);
            let core_thrust = core.performance.specific_thrust;
            return Ok(TurbofanResult { alpha: 0.0, core, fan: None, core_thrust, fan_thrust: 0.0 });
        }
        let c = self.core_to_4(flight, mdot);
        self.try_finish(&c, self.fan)
    }

    /// Stations 0 2 3 4: the freestream, inlet, compressor and burner — the turbojet's own components, in its order.
    pub fn core_to_4(&self, flight: &FlightCondition, mdot: f64) -> CoreTo4 {
        let gas = &self.engine.gas;
        let (mut s, v0) = self.engine.freestream_or_static(flight, mdot);
        let mut stations = vec![("0", s)];
        for &(label, comp) in &self.engine.components {
            match comp {
                Component::Turbine(_) | Component::Nozzle(_) => break,
                other => { s = other.apply(&s, gas); stations.push((label, s)); }
            }
        }
        CoreTo4 { stations, v0, flight: *flight }
    }

    /// Everything a fan changes, from station 4 on: the fan, the turbine (paying for it), both nozzles, the score.
    /// `fan.alpha` must be > 0 (α = 0 is the turbojet's run — [`try_run`](Self::try_run) dispatches it).
    pub fn try_finish(&self, c: &CoreTo4, fan: Fan) -> Result<TurbofanResult, Abort> {
        assert!(fan.alpha > 0.0, "try_finish is the fan's path; α = 0 is the turbojet's run");
        let gas = &self.engine.gas;
        let fl = &c.flight;
        let s2 = *station(&c.stations, "2");
        let s3 = *station(&c.stations, "3");
        let s4 = *station(&c.stations, "4");
        let f = s4.far;

        // Station 13: the fan on the bypass air — the shipped compressor, cold air, alpha kilograms per core kilogram.
        let s13 = Compressor::new(fan.pi_f, fan.eta_f, fan.e_f)
            .apply(&FlowState { mdot: fan.alpha * s2.mdot, ..s2 }, gas);

        // THE SHAFT: one turbine pays for the compressor and the fan.
        let compressor_work = gas.h_c(s3.tt) - gas.h_c(s2.tt);
        let fan_work = fan.alpha * (gas.h_c(s13.tt) - gas.h_c(s2.tt));
        let delta_h = (compressor_work + fan_work) / (self.engine.eta_m * (1.0 + f));
        let turbine = match self.engine.components.iter().find(|(l, _)| *l == "5") {
            Some(&(_, Component::Turbine(t))) => t,
            _ => panic!("a turbojet has its turbine at station 5"),
        };
        // THE WALL, before the gas tables are asked to invert an enthalpy below their range. If even the IDEAL turbine
        // exit `Tt5s` (the isentropic state at the turbine's exit pressure) is not above the outside air's temperature
        // `T0`, then `pt5 < p0`: `pt5/pt4 = pr(Tt5s)/pr(Tt4)` and `pt4 = pt0·π_d·π_c·π_b`, while
        // `pr(Tt4)/pr(Tt0) > pr(Tt3)/pr(Tt2) ≥ π_c ≥ π_d·π_c·π_b` (the compressor's pr ratio is at least its pressure
        // ratio; the burner only heats) — so `pt5 > p0` needs `pr(Tt5s) > pr(T0)`. A jet at `pt9 ≤ pt5 < p0` cannot leave
        // a nozzle exhausting to `p0`: the same refusal the core nozzle gives, reached without the out-of-range lookup.
        let h4 = gas.h_t(s4.tt, f);
        let h0 = gas.h_t(fl.t0, f);
        let ideal_below_ambient = match turbine.e_t {
            None => h4 - delta_h / turbine.eta_t <= h0,
            Some(e_t) => {
                // Tt5 first (h5 ≥ h5s, so h5 ≤ h0 already settles it); then pr(Tt5s) = pr(Tt4)·pt5/pt4 without inverting.
                h4 - delta_h <= h0 || {
                    let tt5 = gas.t_from_h_t(h4 - delta_h, f);
                    let pt_ratio = powp(gas.pr_t(tt5, f) / gas.pr_t(s4.tt, f), 1.0 / e_t);
                    gas.pr_t(s4.tt, f) * pt_ratio <= gas.pr_t(fl.t0, f)
                }
            }
        };
        if ideal_below_ambient {
            return Err(Abort(NO_CORE_PRESSURE.to_string()));
        }
        let s5 = turbine.apply(&s4, gas, delta_h);
        // Shaft CLOSURE, from the turbine's OUTPUT (the engine's check, with the fan's term): every run.
        let turbine_power = self.engine.eta_m * (1.0 + s5.far) * (gas.h_t(s4.tt, f) - gas.h_t(s5.tt, f));
        let demand = compressor_work + fan_work;
        assert!((turbine_power - demand).abs() < 1e-6 * demand,
                "turbofan shaft does not close: turbine {turbine_power} != compressor + fan {demand}");

        // Station 9: the core nozzle, as shipped. A jet with no pressure left is the fan's wall, in words.
        let nozzle = self.core_nozzle();
        let core_exit = nozzle.try_apply(&s5, gas).map_err(|e| {
            if e.0.starts_with("nozzle back-pressure") { Abort(NO_CORE_PRESSURE.to_string()) } else { e }
        })?;

        // Station 19: the fan nozzle, cold, in the core nozzle's mode.
        let fan_nozzle = Nozzle { pi_n: fan.pi_fn, ..nozzle };
        let fan_exit = try_cold_nozzle(&fan_nozzle, &s13, gas).map_err(|e| {
            if e.0.starts_with("nozzle back-pressure") {
                Abort(format!("The fan's pressure rise ({:.3}) does not cover the inlet and duct losses here: the \
                               bypass air would arrive at the nozzle below the outside pressure.", fan.pi_f))
            } else { e }
        })?;

        let mut stations = c.stations.clone();
        stations.push(("5", s5));
        stations.push(("9", core_exit.state));
        let sc = score_turbofan(gas, &stations, fan.alpha, c.v0, &core_exit, &fan_exit, fl.p0, gas.hpr());
        let core = EngineResult { stations, performance: sc.performance, v0: c.v0, v9: core_exit.v9,
                                  m9: core_exit.m9, t9: core_exit.t9, p9: core_exit.p9 };
        Ok(TurbofanResult { alpha: fan.alpha, core, fan: Some(FanStream { s13, exit: fan_exit }),
                            core_thrust: sc.core_thrust, fan_thrust: sc.fan_thrust })
    }
}

/// [`score_turbofan`]'s result: the performance per TOTAL air and each jet's own thrust.
#[derive(Clone, Copy, Debug)]
pub struct FanScore {
    pub performance: Performance,
    /// N·s per kilogram of core air.
    pub core_thrust: f64,
    /// N·s per kilogram of bypass air.
    pub fan_thrust: f64,
}

/// Score two jets (`docs/rung109-spec.md` § Thrust): per TOTAL air, the fuel the core's.
///
/// ```text
/// F_core = (1+f)·V9 − V0 + (1+f)·R_t(f)·T9·(1 − p0/p9)/V9          # the turbojet's, per core kg (engine::score)
/// F_fan  = V19 − V0 + R_c·T19·(1 − p0/p19)/V19                      # cold air, no fuel, per bypass kg
/// F/ṁ0   = (F_core + α·F_fan)/(1+α);   TSFC = f/((1+α)·F/ṁ0)
/// η_th   = [(1+f)·V9² + α·V19² − (1+α)·V0²] / (2·f·hPR)            # jet KE gained per fuel energy
/// η_o    = (1+α)·(F/ṁ0)·V0 / (f·hPR)                               # thrust power per fuel power
/// η_p    = η_o / η_th
/// ```
///
/// Standing still (`V0 = 0`) `η_o` and `η_p` are exactly 0, the standing-engine rule of [`crate::engine::score_or_static`].
#[allow(clippy::too_many_arguments)]
pub fn score_turbofan(
    gas: &Gas, stations: &[(&'static str, FlowState)], alpha: f64, v0: f64,
    core: &NozzleExit, fan: &NozzleExit, p0: f64, hpr: f64,
) -> FanScore {
    let f = station(stations, "4").far;
    let (t9, v9, p9) = (core.t9, core.v9, core.p9);
    let pressure_thrust = (1.0 + f) * gas.r_t_at(f) * t9 * (1.0 - p0 / p9) / v9;
    let core_thrust = (1.0 + f) * v9 - v0 + pressure_thrust;
    let (t19, v19, p19) = (fan.t9, fan.v9, fan.p9);
    let fan_thrust = v19 - v0 + gas.r_c() * t19 * (1.0 - p0 / p19) / v19;
    let specific_thrust = (core_thrust + alpha * fan_thrust) / (1.0 + alpha);
    let tsfc = f / ((1.0 + alpha) * specific_thrust);
    let eta_brayton = 1.0 - station(stations, "2").tt / station(stations, "3").tt;
    let ke_net = (1.0 + f) * (v9 * v9) + alpha * (v19 * v19) - (1.0 + alpha) * (v0 * v0);
    let eta_thermal = ke_net / (2.0 * f * hpr);
    let eta_overall = (1.0 + alpha) * specific_thrust * v0 / (f * hpr);
    let eta_propulsive = eta_overall / eta_thermal;
    if v0 == 0.0 {
        assert!(eta_overall == 0.0 && eta_propulsive == 0.0,
                "a standing engine does no propulsive work: eta_p {eta_propulsive}, eta_o {eta_overall}");
    }
    FanScore {
        performance: Performance { specific_thrust, tsfc, eta_brayton, eta_thermal, eta_propulsive, eta_overall },
        core_thrust, fan_thrust,
    }
}

/// The COLD-section nozzle — [`Nozzle::try_apply`]'s three laws on fresh air (the bypass stream never meets fuel):
///
/// ```text
/// Tt19 = Tt13;   pt19 = π_fn·pt13
///   perfect gas:  M19 = sqrt(((pt19/p19)^gc − 1)/((γc−1)/2));  T19 = Tt19/(1 + (γc−1)/2·M19²);  V19 = M19·sqrt(γc·Rc·T19)
///   tables:       T19 = T_from_pr_c(pr_c(Tt19)·p19/pt19);  V19 = sqrt(2·(h_c(Tt19) − h_c(T19)))
///   convergent:   choked when the sonic p* > p0 — h_c(Tt19) − h_c(T*) = ½·γc(T*)·Rc·T*, p19 = p*; else p19 = p0
/// ```
///
/// The same arithmetic, in the same order, as the hot nozzle with the cold section's functions, so on a single
/// perfect gas (cold = hot) the two agree bit for bit (`tests/rung109.rs`). Every check is an [`Abort`].
pub fn try_cold_nozzle(nz: &Nozzle, s: &FlowState, gas: &Gas) -> Result<NozzleExit, Abort> {
    let r = gas.r_c();
    let tt9 = s.tt;
    let pt9 = nz.pi_n * s.pt;
    let p9 = if nz.convergent {
        let (tstar, pstar, vstar) = try_cold_sonic_throat(gas, tt9, pt9)?;
        if pstar > nz.p_ambient {
            let (p9, t9, v9) = (pstar, tstar, vstar);
            let a9 = powp(gas.gamma_c_at(t9) * r * t9, 0.5);
            let m9 = v9 / a9;
            let out = FlowState { tt: tt9, pt: pt9, mdot: s.mdot, far: s.far };
            check((gas.pr_c(tt9) / gas.pr_c(t9) - pt9 / p9).abs() < 1e-9 * (pt9 / p9),
                  || "fan nozzle static drop not isentropic".to_string())?;
            check((m9 - 1.0).abs() < 1e-9, || "choked fan nozzle must exit at M19 = 1".to_string())?;
            let split_tol = if gas.cold_is_cpg() { 1e-3 } else { 1e-9 };
            let enthalpy_total = gas.h_c(tt9);
            check((gas.h_c(t9) + 0.5 * (v9 * v9) - enthalpy_total).abs() <= split_tol * enthalpy_total,
                  || "choked fan nozzle energy split off by more than the constant mismatch".to_string())?;
            return Ok(NozzleExit { state: out, m9, t9, v9, p9 });
        }
        nz.p_ambient
    } else {
        nz.p_exit
    };
    check(p9 <= pt9, || format!(
        "nozzle back-pressure p19={p9:.0} Pa exceeds total pressure pt19={pt9:.0} Pa — the fan nozzle cannot expand to it"))?;
    let (m9, t9, v9);
    if gas.cold_is_cpg() {
        let (gc, gamma) = (gas.g_c(), gas.gamma_c());
        let half_gm1 = 0.5 * (gamma - 1.0);
        m9 = powp((powp(pt9 / p9, gc) - 1.0) / half_gm1, 0.5);
        t9 = tt9 / (1.0 + half_gm1 * (m9 * m9));
        let a9 = powp(gamma * r * t9, 0.5);
        v9 = m9 * a9;
    } else {
        t9 = gas.t_from_pr_c(gas.pr_c(tt9) * (p9 / pt9));
        v9 = powp(2.0 * (gas.h_c(tt9) - gas.h_c(t9)), 0.5);
        let a9 = powp(gas.gamma_c_at(t9) * r * t9, 0.5);
        m9 = v9 / a9;
    }
    let out = FlowState { tt: tt9, pt: pt9, mdot: s.mdot, far: s.far };
    check((gas.pr_c(tt9) / gas.pr_c(t9) - pt9 / p9).abs() < 1e-9 * (pt9 / p9),
          || "fan nozzle static drop not isentropic".to_string())?;
    let split_tol = if gas.cold_is_cpg() { 1e-3 } else { 1e-9 };
    let enthalpy_total = gas.h_c(tt9);
    let enthalpy_static_plus_ke = gas.h_c(t9) + 0.5 * (v9 * v9);
    check((enthalpy_static_plus_ke - enthalpy_total).abs() <= split_tol * enthalpy_total,
          || format!("fan nozzle energy split off by more than the constant mismatch: {enthalpy_static_plus_ke} vs \
                      {enthalpy_total}"))?;
    Ok(NozzleExit { state: out, m9, t9, v9, p9 })
}

/// The cold air's sonic throat `(T*, p*, V*)` — [`crate::components::try_sonic_throat`] on the cold section: closed
/// form on the perfect gas, bisection on the tables (the same residual, monotone in `T*`).
pub fn try_cold_sonic_throat(gas: &Gas, tt: f64, pt: f64) -> Result<(f64, f64, f64), Abort> {
    let r = gas.r_c();
    let h_tt = gas.h_c(tt);
    let tstar = if gas.cold_is_cpg() {
        let t = h_tt / (gas.cp_c_at(tt) + 0.5 * gas.gamma_c_at(tt) * r);
        check(0.5 * tt < t && t < tt, || "cold sonic-throat root outside the physical bracket".into())?;
        t
    } else {
        let resid = |t: f64| -> f64 { (h_tt - gas.h_c(t)) - 0.5 * gas.gamma_c_at(t) * r * t };
        let (mut lo, mut hi) = (0.5 * tt, tt);
        let mut flo = resid(lo);
        check(flo > 0.0 && resid(hi) <= 0.0, || "cold sonic-throat bracket does not straddle M=1".into())?;
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            let fm = resid(mid);
            if flo * fm <= 0.0 { hi = mid; } else { lo = mid; flo = fm; }
            if hi - lo <= 1e-13 * tt { break; }
        }
        0.5 * (lo + hi)
    };
    let pstar = pt * gas.pr_c(tstar) / gas.pr_c(tt);
    let vstar = powp(2.0 * (h_tt - gas.h_c(tstar)), 0.5);
    Ok((tstar, pstar, vstar))
}

fn check(ok: bool, msg: impl FnOnce() -> String) -> Result<(), Abort> {
    if ok { Ok(()) } else { Err(Abort(msg())) }
}

/// The fan pressure ratio with the lowest fuel per thrust, everything else held.
#[derive(Clone, Copy, Debug)]
pub struct BestFan {
    pub pi_f: f64,
    pub tsfc: f64,
    /// The lowest TSFC sits at the top of the searched range: the best is there or above.
    pub at_top: bool,
    /// The lowest TSFC sits against the TURBINE's wall ([`NO_CORE_PRESSURE`]): a stronger fan would still be better,
    /// but the turbine cannot pay for it.
    pub at_wall: bool,
}

/// Fan pressure ratios the search may try (the page's knob range).
pub const PI_F_RANGE: (f64, f64) = (1.0, 4.0);

/// The best fan pressure ratio at `fan`'s bypass ratio (`docs/rung109-spec.md` § The best fan): a 31-point scan of
/// [`PI_F_RANGE`] over the settings that RUN; where the best point's neighbour does NOT run, the edge of the runnable
/// range between them is found by bisection (to `1e−6`) and becomes the bracket's end; then a golden-section refinement
/// to `1e−4` in `π_f`. Standing still the fuel is fixed by the core, so the lowest TSFC is the highest thrust — the
/// same search. `None` when no scanned fan pressure ratio runs.
pub fn best_pi_f(tf: &Turbofan, c: &CoreTo4, fan: Fan) -> Option<BestFan> {
    assert!(fan.alpha > 0.0);
    let try_at = |pf: f64| tf.try_finish(c, Fan { pi_f: pf, ..fan });
    let eval = |pf: f64| -> Option<f64> {
        try_at(pf).ok().map(|r| r.core.performance.tsfc).filter(|t| t.is_finite() && *t > 0.0)
    };
    let (lo, hi) = PI_F_RANGE;
    const N: usize = 31;
    let xs: Vec<f64> = (0..N).map(|i| lo + (hi - lo) * i as f64 / (N - 1) as f64).collect();
    // pi_f = 1.0 exactly is a fan that does nothing; start the scan just above it.
    let xs: Vec<f64> = xs.into_iter().map(|x| if x == lo { lo + 1e-3 } else { x }).collect();
    let ys: Vec<Option<f64>> = xs.iter().map(|&x| eval(x)).collect();
    let (ib, yb) = ys.iter().enumerate().filter_map(|(i, y)| y.map(|y| (i, y)))
        .min_by(|a, b| a.1.total_cmp(&b.1))?;
    // The runnable edge between a runnable `inside` and a refused `outside` point.
    let edge = |mut inside: f64, mut outside: f64| -> f64 {
        while (outside - inside).abs() > 1e-6 {
            let mid = 0.5 * (inside + outside);
            if eval(mid).is_some() { inside = mid; } else { outside = mid; }
        }
        inside
    };
    let mut right_wall = false;
    let mut a = if ib == 0 { xs[0] } else if ys[ib - 1].is_some() { xs[ib - 1] } else { edge(xs[ib], xs[ib - 1]) };
    let mut b = if ib + 1 == N { xs[N - 1] } else if ys[ib + 1].is_some() { xs[ib + 1] } else {
        right_wall = true;
        edge(xs[ib], xs[ib + 1])
    };
    let b_edge = b;
    let (mut best_x, mut best_y) = (xs[ib], yb);
    for x in [a, b] {
        if let Some(y) = eval(x) { if y < best_y { best_x = x; best_y = y; } }
    }
    if b > a {
        let g = 0.5 * (5f64.sqrt() - 1.0);
        let (mut x1, mut x2) = (b - g * (b - a), a + g * (b - a));
        let (mut f1, mut f2) = (eval(x1).unwrap_or(f64::INFINITY), eval(x2).unwrap_or(f64::INFINITY));
        while b - a > 1e-4 {
            if f1 <= f2 { b = x2; x2 = x1; f2 = f1; x1 = b - g * (b - a); f1 = eval(x1).unwrap_or(f64::INFINITY); }
            else { a = x1; x1 = x2; f1 = f2; x2 = a + g * (b - a); f2 = eval(x2).unwrap_or(f64::INFINITY); }
        }
        for (x, y) in [(x1, f1), (x2, f2)] {
            if y < best_y { best_x = x; best_y = y; }
        }
    }
    // Against the turbine's wall: the best is the right-hand runnable edge, and what lies past it is the turbine's
    // refusal (not the fan's own, which bounds the range from below).
    let at_wall = right_wall && (b_edge - best_x) <= 1e-4
        && try_at(b_edge + 2e-6).err().is_some_and(|e| e.0 == NO_CORE_PRESSURE);
    Some(BestFan { pi_f: best_x, tsfc: best_y, at_top: ib == N - 1, at_wall })
}
