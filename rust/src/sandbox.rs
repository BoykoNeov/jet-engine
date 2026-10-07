//! The web sandbox's model side (`docs/plans/sandbox-plan.md`) — NOT a rung.
//!
//! The user changes the engine's design and watches it respond (the project's direction since
//! 2026-10-06). This module is the whole bridge between a page's knobs and the model, written as
//! ordinary Rust so the normal gate tests it: settings arrive as JSON text, the model runs, the
//! result leaves as JSON text. The browser build (`rust/sandbox-wasm/`) is only a thin export
//! shell around [`call`].
//!
//! Three rules shape it:
//! - **Full precision out.** No `r6`: the result is the model's own `f64`s, so a check that the
//!   browser's numbers match the native ones measures the model, not a rounding.
//! - **A design that does not run is a result.** The cheap, common failure (a turbine-inlet
//!   temperature not above the compressor exit) is caught BEFORE the solve and named in plain
//!   words. Anything else panics inside the model, as it always has — the conservation asserts are
//!   the contract — and the page shows [`explain`]'s plain words beside the model's own message.
//! - **It edits no model code.** It calls `build_turbojet(…).run(…)` exactly as the CLI does, so
//!   every golden and oracle is unchanged.

use crate::atmosphere::{self, Ambient};
use crate::components::{ram_recovery, Component};
use crate::engine::{build_turbojet, score, EngineResult, FlightCondition, Losses};
use crate::gas::{FlowState, Gas};
use crate::jobj;
use crate::map::ComponentMap;
use crate::matcher::{Branch, Rebuilt};
use crate::spool::SpoolTransient;
use crate::panels::{flight, real_losses, PI_C, TT4};
use crate::visuals::Json;

/// The five gas models of rungs 1–6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GasModel {
    /// Rungs 1–2: constant cp (the textbook cold-air standard).
    Perfect,
    /// Rung 3: cp(T) from the NASA tables.
    ThermallyPerfect,
    /// Rung 4: the products' composition follows the fuel-air ratio.
    Reacting,
    /// Rung 5: the heat release derived from formation enthalpies.
    ForkB,
    /// Rung 6: chemical equilibrium — the production cycle.
    Equilibrium,
}

impl GasModel {
    pub const ALL: [GasModel; 5] = [GasModel::Perfect, GasModel::ThermallyPerfect,
                                    GasModel::Reacting, GasModel::ForkB, GasModel::Equilibrium];

    pub fn key(self) -> &'static str {
        match self {
            GasModel::Perfect => "perfect",
            GasModel::ThermallyPerfect => "thermally_perfect",
            GasModel::Reacting => "reacting",
            GasModel::ForkB => "fork_b",
            GasModel::Equilibrium => "equilibrium",
        }
    }

    fn from_key(k: &str) -> Result<Self, String> {
        GasModel::ALL.into_iter().find(|g| g.key() == k)
            .ok_or_else(|| format!("unknown gas model {k:?}"))
    }

    pub fn gas(self) -> Gas {
        match self {
            GasModel::Perfect => Gas::default(),
            GasModel::ThermallyPerfect => Gas::thermally_perfect(),
            GasModel::Reacting => Gas::reacting(),
            GasModel::ForkB => Gas::reacting_forkb(),
            GasModel::Equilibrium => Gas::reacting_equilibrium(),
        }
    }
}

/// How the nozzle's exit pressure is decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NozzleMode {
    /// Exit pressure = ambient (an ideally-shaped nozzle).
    Expanded,
    /// Exit pressure set by the `p_exit` knob (under/over-expanded; thrust gains a pressure term).
    ExitPressure,
    /// Rung 30: a fixed convergent nozzle — the flow picks the exit pressure (sonic if choked).
    Convergent,
}

impl NozzleMode {
    pub const ALL: [NozzleMode; 3] = [NozzleMode::Expanded, NozzleMode::ExitPressure, NozzleMode::Convergent];

    pub fn key(self) -> &'static str {
        match self {
            NozzleMode::Expanded => "expanded",
            NozzleMode::ExitPressure => "exit_pressure",
            NozzleMode::Convergent => "convergent",
        }
    }

    fn from_key(k: &str) -> Result<Self, String> {
        NozzleMode::ALL.into_iter().find(|n| n.key() == k)
            .ok_or_else(|| format!("unknown nozzle mode {k:?}"))
    }
}

/// Every knob of the design-point sandbox (slice 1).
///
/// Both efficiency spellings are kept for each machine and a mode picks one (the model forbids
/// setting both — rung 2b), so toggling the mode does not lose the other value. Likewise
/// `p_exit` is kept while the nozzle is in another mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Ambient static temperature, K, and pressure, Pa — what the model runs on. The altitude
    /// knobs are a view of these ([`Ambient`]).
    pub t0: f64,
    pub p0: f64,
    pub m0: f64,
    pub pi_c: f64,
    pub tt4: f64,
    /// Air mass flow, kg/s — scales thrust and fuel flow, nothing per-kilogram.
    pub mdot: f64,
    /// The inlet's own recovery; the model is fed `pi_d_max · ram_recovery(M0)`, so the inlet
    /// keeps answering to flight speed above Mach 1 (the plan's "inlet trap").
    pub pi_d_max: f64,
    pub compressor_polytropic: bool,
    pub eta_c: f64,
    pub e_c: f64,
    pub turbine_polytropic: bool,
    pub eta_t: f64,
    pub e_t: f64,
    pub eta_b: f64,
    pub pi_b: f64,
    pub eta_m: f64,
    pub pi_n: f64,
    pub nozzle: NozzleMode,
    pub p_exit: f64,
    pub gas: GasModel,
}

impl Settings {
    /// What the page opens on: the CLI's design point and loss set (`panels`), on the
    /// production gas (user decision, plan § 9.2).
    pub fn defaults() -> Self {
        let f = flight();
        let l = real_losses();
        Settings {
            t0: f.t0, p0: f.p0, m0: f.m0, pi_c: PI_C, tt4: TT4, mdot: 20.0,
            pi_d_max: l.pi_d, compressor_polytropic: false, eta_c: l.eta_c, e_c: 0.90,
            turbine_polytropic: false, eta_t: l.eta_t, e_t: 0.90,
            eta_b: l.eta_b, pi_b: l.pi_b, eta_m: l.eta_m, pi_n: l.pi_n,
            nozzle: NozzleMode::Expanded, p_exit: f.p0, gas: GasModel::Equilibrium,
        }
    }

    pub fn flight(&self) -> FlightCondition { FlightCondition::new(self.t0, self.p0, self.m0) }

    /// The inlet recovery the model actually runs with.
    pub fn pi_d(&self) -> f64 { self.pi_d_max * ram_recovery(self.m0) }

    pub fn losses(&self) -> Losses {
        let (eta_c, e_c) = if self.compressor_polytropic { (1.0, Some(self.e_c)) } else { (self.eta_c, None) };
        let (eta_t, e_t) = if self.turbine_polytropic { (1.0, Some(self.e_t)) } else { (self.eta_t, None) };
        Losses {
            pi_d: self.pi_d(), eta_c, e_c, eta_b: self.eta_b, pi_b: self.pi_b, eta_t, e_t,
            eta_m: self.eta_m, pi_n: self.pi_n,
            p_exit: if self.nozzle == NozzleMode::ExitPressure { Some(self.p_exit) } else { None },
            nozzle_convergent: self.nozzle == NozzleMode::Convergent,
        }
    }

    pub fn to_json(&self) -> Json {
        jobj! {
            "T0" => self.t0, "p0" => self.p0, "M0" => self.m0, "pi_c" => self.pi_c,
            "Tt4" => self.tt4, "mdot" => self.mdot, "pi_d_max" => self.pi_d_max,
            "compressor_eff" => if self.compressor_polytropic { "polytropic" } else { "isentropic" },
            "eta_c" => self.eta_c, "e_c" => self.e_c,
            "turbine_eff" => if self.turbine_polytropic { "polytropic" } else { "isentropic" },
            "eta_t" => self.eta_t, "e_t" => self.e_t,
            "eta_b" => self.eta_b, "pi_b" => self.pi_b, "eta_m" => self.eta_m, "pi_n" => self.pi_n,
            "nozzle" => self.nozzle.key(), "p_exit" => self.p_exit, "gas" => self.gas.key(),
        }
    }

    /// Read settings; a missing key keeps its default, so a page may send only what it changed.
    pub fn from_json(j: &Json) -> Result<Self, String> {
        let mut s = Settings::defaults();
        let num = |k: &str, into: &mut f64| -> Result<(), String> {
            match j.get(k) {
                None => Ok(()),
                Some(Json::Float(x)) => { *into = *x; Ok(()) }
                Some(Json::Int(n)) => { *into = *n as f64; Ok(()) }
                Some(other) => Err(format!("setting {k:?} must be a number, got {other:?}")),
            }
        };
        let text = |k: &str| -> Result<Option<String>, String> {
            match j.get(k) {
                None => Ok(None),
                Some(Json::Str(t)) => Ok(Some(t.clone())),
                Some(other) => Err(format!("setting {k:?} must be text, got {other:?}")),
            }
        };
        num("T0", &mut s.t0)?;
        num("p0", &mut s.p0)?;
        num("M0", &mut s.m0)?;
        num("pi_c", &mut s.pi_c)?;
        num("Tt4", &mut s.tt4)?;
        num("mdot", &mut s.mdot)?;
        num("pi_d_max", &mut s.pi_d_max)?;
        num("eta_c", &mut s.eta_c)?;
        num("e_c", &mut s.e_c)?;
        num("eta_t", &mut s.eta_t)?;
        num("e_t", &mut s.e_t)?;
        num("eta_b", &mut s.eta_b)?;
        num("pi_b", &mut s.pi_b)?;
        num("eta_m", &mut s.eta_m)?;
        num("pi_n", &mut s.pi_n)?;
        num("p_exit", &mut s.p_exit)?;
        let mode = |k: &str, cur: bool| -> Result<bool, String> {
            match text(k)?.as_deref() {
                None => Ok(cur),
                Some("isentropic") => Ok(false),
                Some("polytropic") => Ok(true),
                Some(o) => Err(format!("setting {k:?} must be \"isentropic\" or \"polytropic\", got {o:?}")),
            }
        };
        s.compressor_polytropic = mode("compressor_eff", s.compressor_polytropic)?;
        s.turbine_polytropic = mode("turbine_eff", s.turbine_polytropic)?;
        if let Some(n) = text("nozzle")? { s.nozzle = NozzleMode::from_key(&n)?; }
        if let Some(g) = text("gas")? { s.gas = GasModel::from_key(&g)?; }
        Ok(s)
    }
}

/// A design that does not run, in plain words — what the page shows instead of numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct DoesNotRun(pub String);

/// The checks made BEFORE the solve. Each names the knob and the physical reason.
pub fn precheck(s: &Settings) -> Result<(), DoesNotRun> {
    let no = |m: String| Err(DoesNotRun(m));
    let named = [("T0", s.t0), ("p0", s.p0), ("M0", s.m0), ("pi_c", s.pi_c), ("Tt4", s.tt4),
                 ("mdot", s.mdot), ("pi_d_max", s.pi_d_max), ("eta_c", s.eta_c), ("e_c", s.e_c),
                 ("eta_t", s.eta_t), ("e_t", s.e_t), ("eta_b", s.eta_b), ("pi_b", s.pi_b),
                 ("eta_m", s.eta_m), ("pi_n", s.pi_n), ("p_exit", s.p_exit)];
    if let Some((k, _)) = named.iter().find(|(_, v)| !v.is_finite()) {
        return no(format!("Setting {k} is not a number."));
    }
    if s.t0 <= 0.0 || s.p0 <= 0.0 {
        return no("Ambient temperature and pressure must be above zero.".into());
    }
    if s.m0 <= 0.0 {
        return no("Flight Mach number must be above zero. The model scores efficiency per unit of \
                   flight speed, so a standing engine (Mach 0) is not supported yet.".into());
    }
    if s.mdot <= 0.0 {
        return no("Air mass flow must be above zero.".into());
    }
    if s.pi_c <= 1.0 {
        return no("Compressor pressure ratio must be above 1. At 1 the compressor does no work, so \
                   the turbine has nothing to drive; below 1 it would be a turbine itself.".into());
    }
    let eff = [("compressor efficiency", if s.compressor_polytropic { s.e_c } else { s.eta_c }),
               ("turbine efficiency", if s.turbine_polytropic { s.e_t } else { s.eta_t }),
               ("combustion efficiency", s.eta_b), ("shaft mechanical efficiency", s.eta_m)];
    if let Some((k, _)) = eff.iter().find(|(_, v)| !(*v > 0.0 && *v <= 1.0)) {
        return no(format!("The {k} must be above 0 and at most 1 (100 %)."));
    }
    let rec = [("inlet pressure recovery", s.pi_d_max), ("burner pressure ratio", s.pi_b),
               ("nozzle pressure ratio", s.pi_n)];
    if let Some((k, _)) = rec.iter().find(|(_, v)| !(*v > 0.0 && *v <= 1.0)) {
        return no(format!("The {k} must be above 0 and at most 1 — a duct can only lose pressure."));
    }
    if s.nozzle == NozzleMode::ExitPressure && s.p_exit <= 0.0 {
        return no("The nozzle exit pressure must be above zero.".into());
    }
    // The one solve-dependent check: the burner can only ADD heat. Run the inlet and compressor
    // (cheap, and pure) to find the compressor-exit temperature.
    let engine = build_turbojet(s.gas.gas(), s.pi_c, s.tt4, s.p0, s.losses());
    let (mut st, _) = engine.try_freestream(&s.flight(), s.mdot)
        .map_err(|e| DoesNotRun(format!("The freestream could not be formed: {}", e.0)))?;
    for (label, c) in &engine.components {
        if *label == "4" { break; }
        if let Component::Inlet(_) | Component::Compressor(_) = c {
            st = c.apply(&st, &engine.gas);
        }
    }
    if s.tt4 <= st.tt {
        return no(format!(
            "The turbine-inlet temperature ({:.0} K) is not above the compressor-exit temperature \
             ({:.0} K), so the burner would have to cool the air. Raise Tt4 or lower the pressure ratio.",
            s.tt4, st.tt));
    }
    Ok(())
}

/// One sandbox run: the model's own result plus what the page needs beside it.
pub struct Outcome {
    pub settings: Settings,
    pub result: EngineResult,
    /// `(label, s, T)` for the T–s diagram — see [`ts_points`].
    pub ts: Vec<(&'static str, f64, f64)>,
    /// The burner and exhaust-cooling legs as `(s, T)` curves — see [`ts_curves`].
    pub curves: [Vec<(f64, f64)>; 2],
}

/// Run one design. The solve itself is `build_turbojet(…).run(…)`, the CLI's own call.
pub fn run(s: &Settings) -> Result<Outcome, DoesNotRun> {
    precheck(s)?;
    let engine = build_turbojet(s.gas.gas(), s.pi_c, s.tt4, s.p0, s.losses());
    let result = engine.run(&s.flight(), s.mdot);
    let ts = ts_points(&engine.gas, &result, &s.flight());
    let curves = ts_curves(&engine.gas, &result, &ts);
    Ok(Outcome { settings: *s, result, ts, curves })
}

/// Entropy at each station for the T–s diagram, ON THE RUN'S OWN GAS and from the run's OWN
/// ambient datum — `visuals::cycle_points` uses the perfect gas and the panels' flight whatever it
/// is handed, so it is right only there (and this reduces to it there, bit for bit:
/// `tests/sandbox.rs`).
///
/// Station 0 and 9 are STATIC states (the ambient air, the jet), 2–5 TOTAL ones — the existing
/// chart's convention. Stations 0–3 use the cold (air) section; 4–9 the hot section at the run's
/// fuel-air ratio, each referenced to ITS OWN mixture at the ambient datum — so the burner's jump
/// mixes heat addition with a change of reference mixture (the page says so).
pub fn ts_points(gas: &Gas, r: &EngineResult, fl: &FlightCondition) -> Vec<(&'static str, f64, f64)> {
    let (tref, pref) = (fl.t0, fl.p0);
    let far = r.station("4").far;
    let s_cold = |t: f64, p: f64| {
        if gas.cold_is_cpg() {
            gas.spec.cp_c * (t / tref).ln() - gas.spec.r_c * (p / pref).ln()
        } else {
            gas.r_c() * (gas.pr_c(t).ln() - gas.pr_c(tref).ln()) - gas.r_c() * (p / pref).ln()
        }
    };
    let s_hot = |t: f64, p: f64| {
        if gas.hot_is_cpg() {
            gas.spec.cp_t * (t / tref).ln() - gas.spec.r_t * (p / pref).ln()
        } else {
            let rt = gas.r_t_at(far);
            rt * (gas.pr_t(t, far).ln() - gas.pr_t(tref, far).ln()) - rt * (p / pref).ln()
        }
    };
    let st = |l: &str| -> FlowState { *r.station(l) };
    vec![
        ("0", s_cold(fl.t0, fl.p0), fl.t0),
        ("2", s_cold(st("2").tt, st("2").pt), st("2").tt),
        ("3", s_cold(st("3").tt, st("3").pt), st("3").tt),
        ("4", s_hot(st("4").tt, st("4").pt), st("4").tt),
        ("5", s_hot(st("5").tt, st("5").pt), st("5").tt),
        ("9", s_hot(r.t9, r.p9), r.t9),
    ]
}

/// The two heat-exchange legs of the T–s diagram as CURVES, `TS_CURVE_POINTS` each: the burner
/// (3 → 4) and the exhaust cooling back to the outside air (9 → 0). The compression and expansion
/// legs are drawn straight between stations; these two are not, because heat added or removed at
/// near-constant pressure follows a curve, and a straight 3 → 4 line misdraws the cycle's biggest leg.
///
/// The SHAPE is the burned gas's own constant-pressure line on the run's gas (`cp ln T` on the
/// perfect gas, `R ln pr(T)` from the tables), from the leg's first station; whatever that misses
/// the second station by (the pressure loss, the change of mixture) is spread linearly in `T` —
/// the charts page's `ts_legs` method, there on the perfect gas only. Both ends land on the
/// stations, so the curve and the points agree (`tests/sandbox.rs`).
pub fn ts_curves(gas: &Gas, r: &EngineResult, pts: &[(&'static str, f64, f64)]) -> [Vec<(f64, f64)>; 2] {
    let far = r.station("4").far;
    let at = |l: &str| { let p = pts.iter().find(|p| p.0 == l).unwrap(); (p.1, p.2) };
    let shape = |t: f64, ta: f64| {
        if gas.hot_is_cpg() {
            gas.spec.cp_t * (t / ta).ln()
        } else {
            gas.r_t_at(far) * (gas.pr_t(t, far).ln() - gas.pr_t(ta, far).ln())
        }
    };
    let leg = |a: &str, b: &str| {
        let ((sa, ta), (sb, tb)) = (at(a), at(b));
        let residual = sb - (sa + shape(tb, ta));
        (0..TS_CURVE_POINTS)
            .map(|i| {
                let t = ta + (tb - ta) * i as f64 / (TS_CURVE_POINTS - 1) as f64;
                (sa + shape(t, ta) + residual * (t - ta) / (tb - ta), t)
            })
            .collect()
    };
    [leg("3", "4"), leg("9", "0")]
}

/// Points per curved T–s leg.
pub const TS_CURVE_POINTS: usize = 32;

/// Plain words for a message the MODEL panicked with — shown above the message itself, which the
/// page always prints verbatim. Unrecognised messages get a general line; a wrong guess here
/// would mislead, so each entry matches a message the model was MEASURED to raise, and
/// `tests/sandbox.rs` drives each one with a design that raises it.
pub fn explain(message: &str) -> String {
    // The seven messages a 13 000-design sweep over every gas model actually produced
    // (2026-10-06, pressure ratio 1.02–60, Tt4 600–2800 K, Mach 0.05–3.5, efficiencies 0.6–1).
    const TURBINE_STARVED: &str =
        "The turbine has to take out more work than the hot gas can give: its exit temperature falls \
         off the bottom of the gas tables. Raise Tt4, lower the pressure ratio, or improve the \
         compressor and turbine efficiencies.";
    const BURNER_OVERREACH: &str =
        "The burner's energy balance did not close. This shows up when the burner is asked for a very \
         large temperature rise — a mixture at or past the point where all the oxygen is used. Lower \
         Tt4, or raise the pressure ratio so the air arrives hotter.";
    const KNOWN: &[(&str, &str)] = &[
        ("rich mixture",
         "Reaching this turbine-inlet temperature needs more fuel than the air can burn (a rich \
          mixture), and this gas model handles only lean burning. Lower Tt4, or raise the pressure \
          ratio so the air arrives hotter."),
        ("Fork B absolute-enthalpy balance", BURNER_OVERREACH),
        ("equilibrium burner balance", BURNER_OVERREACH),
        ("nozzle back-pressure",
         "The gas reaching the nozzle has less pressure than the air it must push into, so the nozzle \
          cannot expel it. The turbine has used up too much of the pressure (poor efficiencies or a low \
          Tt4), or the nozzle exit pressure asked for is too high."),
        ("root not bracketed", TURBINE_STARVED),
        ("turbine substate not isentropic", TURBINE_STARVED),
        ("efficiency cascade",
         "The efficiency bookkeeping failed. This happens at very low thrust, where the jet is barely \
          faster than the flight speed."),
    ];
    KNOWN.iter().find(|(k, _)| message.contains(k)).map(|(_, v)| v.to_string()).unwrap_or_else(|| {
        "The model stopped on one of its internal checks for this design.".to_string()
    })
}

/// Plain words for a message the model panicked with WHILE FLYING (slice 3). The fly solve's own
/// failure is rung 34's "equilibrium does not bracket": no shaft speed balances turbine against
/// compressor. Its message carries the ends of the speed search — `Phi[Some(ν)]=Some(Φ)` or
/// `Phi[None]=None` — and the plain words read them, because the same message means opposite things:
/// a 27 610-point sweep (2026-10-07, plan § 10.4) found the shaft slowing at every workable speed
/// (below idle), speeding up all the way to the search's top (overspeed), and speeding up where only
/// the SLOWEST speeds were workable — fast flight, low throttle, the compressor heating the air past
/// the throttle setting above them. Each branch is driven by a test (`tests/sandbox_fly.rs`).
/// Anything else falls back to [`explain`].
pub fn explain_fly(message: &str) -> String {
    if !message.contains("equilibrium does not bracket") {
        return explain(message);
    }
    // Each end of the search: `None` (no workable speed found from that side) or `(ν, Φ)`.
    let ends: Vec<Option<(f64, f64)>> = message.split("Phi[").skip(1).map(|part| {
        let inner = |s: &str| s.strip_prefix("Some(").and_then(|r| r.split(')').next()).and_then(|v| v.parse::<f64>().ok());
        let nu = inner(part);
        let phi = part.split("]=").nth(1).and_then(inner);
        nu.zip(phi)
    }).collect();
    const SPOOL_DOWN: &str =
        "At this throttle and flight the turbine cannot keep the compressor turning at any shaft speed: the \
         engine would spool down. This is below idle. Raise the throttle (the turbine-inlet temperature).";
    const OVERSPEED: &str =
        "At this throttle the turbine gives more power than the compressor can absorb at every shaft speed up \
         to 160 % of design: the shaft would overspeed. Lower the throttle.";
    const HOT_AIR: &str =
        "Only the slowest shaft speeds can be worked out here, and at each the turbine still out-pulls the \
         compressor. At any higher speed the compressor heats the air past the turbine-inlet temperature you \
         set, so the burner would have to cool it. This happens at a low throttle in fast flight, where the \
         air arrives already hot. Raise the throttle.";
    const NO_SPEED: &str =
        "No shaft speed gives a workable operating point at this throttle and flight: at every speed the \
         model tries, the burner, the turbine or the nozzle leaves its range. Usually the throttle is too low \
         for the flight; try raising it.";
    match ends.as_slice() {
        [Some((_, a)), Some((_, b))] if *a < 0.0 && *b < 0.0 => SPOOL_DOWN,
        [Some((_, a)), Some((nu_hi, b))] if *a > 0.0 && *b > 0.0 && *nu_hi >= SPEED_LINE_MAX - 0.05 => OVERSPEED,
        [Some((_, a)), Some((_, b))] if *a > 0.0 && *b > 0.0 => HOT_AIR,
        _ => NO_SPEED,
    }.to_string()
}

fn finite(x: f64, what: &str) -> Result<f64, DoesNotRun> {
    if x.is_finite() { Ok(x) } else { Err(DoesNotRun(format!("The model gave a non-finite {what}."))) }
}

/// The page's view of a run, FULL precision.
pub fn outcome_json(o: &Outcome) -> Result<Json, DoesNotRun> {
    let r = &o.result;
    let p = &r.performance;
    let s = &o.settings;
    let mut stations = Vec::new();
    for (l, st) in &r.stations {
        stations.push(jobj! { "label" => *l, "Tt" => finite(st.tt, "temperature")?,
                              "pt" => finite(st.pt, "pressure")?, "far" => finite(st.far, "fuel-air ratio")? });
    }
    let mut ts = Vec::new();
    for &(l, sv, t) in &o.ts {
        ts.push(jobj! { "label" => l, "s" => finite(sv, "entropy")?, "T" => finite(t, "temperature")? });
    }
    let mut curves = Vec::new();
    for c in &o.curves {
        let mut line = Vec::new();
        for &(sv, t) in c {
            line.push(Json::List(vec![Json::Float(finite(sv, "entropy")?), Json::Float(finite(t, "temperature")?)]));
        }
        curves.push(Json::List(line));
    }
    let f = r.station("4").far;
    let thrust = finite(p.specific_thrust, "specific thrust")? * s.mdot;
    Ok(jobj! {
        "ok" => Json::Int(1),
        "settings" => s.to_json(),
        "ambient" => ambient_json(s.t0, s.p0),
        "pi_d" => s.pi_d(),
        "stations" => Json::List(stations),
        "V0" => finite(r.v0, "flight speed")?, "V9" => finite(r.v9, "jet speed")?,
        "M9" => finite(r.m9, "jet Mach number")?, "T9" => finite(r.t9, "jet temperature")?,
        "p9" => finite(r.p9, "jet pressure")?,
        "specific_thrust" => p.specific_thrust, "thrust" => thrust,
        "fuel_flow" => f * s.mdot,
        "tsfc" => finite(p.tsfc, "TSFC")?,
        "eta_brayton" => finite(p.eta_brayton, "efficiency")?,
        "eta_thermal" => finite(p.eta_thermal, "efficiency")?,
        "eta_propulsive" => finite(p.eta_propulsive, "efficiency")?,
        "eta_overall" => finite(p.eta_overall, "efficiency")?,
        "ts" => Json::List(ts),
        "ts_burner" => curves[0].clone(),
        "ts_reject" => curves[1].clone(),
    })
}

/// The four connected flight knobs for raw `T0`/`p0` — `Null` when `p0` lies outside the
/// standard atmosphere used (the page then shows only the raw values).
pub fn ambient_json(t0: f64, p0: f64) -> Json {
    let (_, p_lo) = atmosphere::standard(atmosphere::Z_MAX);
    let (_, p_hi) = atmosphere::standard(atmosphere::Z_MIN);
    if !(p0 >= p_lo && p0 <= p_hi && t0.is_finite()) {
        return Json::Null;
    }
    let a = Ambient::from_ambient(t0, p0);
    jobj! { "altitude" => a.altitude, "delta_t" => a.delta_t, "T0" => a.t0, "p0" => a.p0 }
}

fn refusal(reason: &str) -> Json {
    jobj! { "ok" => Json::Int(0), "reason" => reason }
}

/// The ONE entry point the browser shell exports. `request` is a JSON object with an `"op"`:
///
/// - `{"op":"defaults"}` → the opening settings.
/// - `{"op":"run","settings":{…}}` → a run, or `{"ok":0,"reason":…}` for a design that does not
///   run by [`precheck`]. A design that fails INSIDE the model panics (the browser shell turns
///   that into a message; see `explain`).
/// - `{"op":"atmosphere","altitude":…,"delta_t":…}` → `T0`/`p0` for those knobs;
///   `{"op":"atmosphere","T0":…,"p0":…}` → the altitude and deviation they read as.
/// - `{"op":"explain","message":…}` → [`explain`]'s plain words; with `"view":"fly"`, [`explain_fly`]'s.
/// - `{"op":"range"}` → the atmosphere's altitude range, for the page's slider.
///
/// Slice 3, flying the design's frozen hardware (`fly` settings: [`FlySettings::from_json`]):
/// - `{"op":"fly_defaults"}` → the opening fly settings.
/// - `{"op":"fly","fly":{…}}` → one operating point ([`fly_json`]), or a plain-words refusal.
/// - `{"op":"running_point","fly":{…}}` → the spool equilibrium alone at the fly throttle (no
///   station table) — the page streams the running line point by point with it.
/// - `{"op":"running_grid","fly":{…}}` → the throttle grid of the running line.
/// - `{"op":"map_lines","fly":{…}}` → the compressor map's speed lines and stall line.
/// - `blade_defaults`, `blade_size`, `blade_grid`, `blade_lever` (each with `"blades":{…}`; the last
///   also `"Tt4"`) → slice 2's blade view, [`crate::sandbox_blades::call_op`]. `explain` with
///   `"view":"blades"` → [`crate::sandbox_blades::explain_blades`].
pub fn call(request: &str) -> String {
    let req = Json::parse(request);
    let op = match req.get("op") { Some(Json::Str(s)) => s.as_str(), _ => "" };
    let num = |k: &str| match req.get(k) {
        Some(Json::Float(x)) => Some(*x),
        Some(Json::Int(n)) => Some(*n as f64),
        _ => None,
    };
    let out = match op {
        "defaults" => Settings::defaults().to_json(),
        "run" => {
            let empty = Json::Obj(Vec::new());
            match Settings::from_json(req.get("settings").unwrap_or(&empty)) {
                Err(e) => refusal(&e),
                Ok(s) => match run(&s).and_then(|o| outcome_json(&o)) {
                    Ok(j) => j,
                    Err(DoesNotRun(m)) => refusal(&m),
                },
            }
        }
        "atmosphere" => match (num("altitude"), num("delta_t"), num("T0"), num("p0")) {
            (Some(z), Some(dt), _, _) => {
                if !(atmosphere::Z_MIN..=atmosphere::Z_MAX).contains(&z) || !dt.is_finite() {
                    refusal("altitude outside the standard atmosphere used")
                } else {
                    let a = Ambient::from_altitude(z, dt);
                    jobj! { "altitude" => a.altitude, "delta_t" => a.delta_t, "T0" => a.t0, "p0" => a.p0 }
                }
            }
            (_, _, Some(t0), Some(p0)) => match ambient_json(t0, p0) {
                Json::Null => refusal("pressure outside the standard atmosphere used"),
                j => j,
            },
            _ => refusal("atmosphere needs altitude + delta_t, or T0 + p0"),
        },
        "explain" => match req.get("message") {
            Some(Json::Str(m)) if matches!(req.get("view"), Some(Json::Str(v)) if v == "fly") => jobj! { "plain" => explain_fly(m) },
            Some(Json::Str(m)) if matches!(req.get("view"), Some(Json::Str(v)) if v == "blades") =>
                jobj! { "plain" => crate::sandbox_blades::explain_blades(m) },
            Some(Json::Str(m)) if matches!(req.get("view"), Some(Json::Str(v)) if v == "blades_lever") =>
                jobj! { "plain" => crate::sandbox_blades::lever_words(m) },
            Some(Json::Str(m)) => jobj! { "plain" => explain(m) },
            _ => refusal("explain needs a message"),
        },
        "range" => jobj! { "z_min" => atmosphere::Z_MIN, "z_max" => atmosphere::Z_MAX },
        "fly_defaults" => FlySettings::defaults().to_json(),
        "fly" | "running_point" | "running_grid" | "map_lines" => {
            let empty = Json::Obj(Vec::new());
            match FlySettings::from_json(req.get("fly").unwrap_or(&empty)) {
                Err(e) => refusal(&e),
                Ok(s) => match fly_precheck(&s) {
                    Err(DoesNotRun(m)) => refusal(&m),
                    Ok(()) => match op {
                        "fly" => match fly(&s).and_then(|o| fly_json(&o)) {
                            Ok(j) => j,
                            Err(DoesNotRun(m)) => refusal(&m),
                        },
                        "running_point" => running_point_json(&s),
                        "running_grid" => Json::List(running_line_grid(&s).into_iter().map(Json::Float).collect()),
                        _ => map_lines(&s),
                    },
                },
            }
        }
        other => crate::sandbox_blades::call_op(other, &req).unwrap_or_else(|| refusal(&format!("unknown op {other:?}"))),
    };
    out.dump_compact()
}

// ---------------------------------------------------------------------------------------------
// Slice 3 — FLY THE ENGINE YOU DESIGNED (`docs/plans/sandbox-plan.md` § 10).
//
// The design's hardware is frozen — the turbine and nozzle throat areas, the design flow and speed
// references — and the throttle and the flight move. The engine finds its own operating point.
// ONE solver, chosen by measurement (§ 10.1): rung 34's spool equilibrium on a rung-32 map, with
// rung 36's stall margin read off the solved point. It handles both nozzle branches; rung 32's map
// matcher does not, and stays off the page.

/// The compressor maps the fly view offers: rung 34's three surge-realistic shapes, as equals. The
/// model has no measured map; each is a representative shape, disclosed as such on the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapShape {
    /// Curvature concentrated in flow (`ComponentMap::surge_flow`).
    Flow,
    /// Curvature concentrated in pressure (`ComponentMap::surge_pressure`).
    Pressure,
    /// A tilted efficiency island (`ComponentMap::surge_tilted`).
    Tilted,
}

impl MapShape {
    pub const ALL: [MapShape; 3] = [MapShape::Flow, MapShape::Pressure, MapShape::Tilted];

    pub fn key(self) -> &'static str {
        match self { MapShape::Flow => "flow", MapShape::Pressure => "pressure", MapShape::Tilted => "tilted" }
    }

    fn from_key(k: &str) -> Result<Self, String> {
        MapShape::ALL.into_iter().find(|m| m.key() == k).ok_or_else(|| format!("unknown map shape {k:?}"))
    }

    pub fn map(self) -> ComponentMap {
        match self {
            MapShape::Flow => ComponentMap::surge_flow(),
            MapShape::Pressure => ComponentMap::surge_pressure(),
            MapShape::Tilted => ComponentMap::surge_tilted(),
        }
    }
}

/// Every knob of the fly view: the design whose hardware is frozen, then what moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlySettings {
    /// The design (slice 1's knobs). Its nozzle and gas are overridden when the hardware is
    /// captured — see [`FlySettings::capture`].
    pub design: Settings,
    /// The gas the engine is flown on. Opens on the thermally-perfect gas (user decision,
    /// plan § 10.7: live at 10–45 ms a point, where equilibrium takes 0.5–2.5 s).
    pub gas: GasModel,
    /// The throttle: turbine-inlet temperature, K.
    pub tt4: f64,
    /// The flight: ambient static temperature (K), pressure (Pa), Mach number.
    pub t0: f64,
    pub p0: f64,
    pub m0: f64,
    pub map: MapShape,
    /// The stall line's flow coefficient — rung 36's ONE chosen constant, a knob here because the
    /// model has no data for it (user decision, plan § 10.7). Only the margin's TREND is load-bearing.
    pub phi_surge: f64,
}

/// Where the page's stall-line knob opens: rung 36's own panel value.
pub const PHI_SURGE_DEFAULT: f64 = 0.65;

impl FlySettings {
    /// Flying the opening design at its own design point.
    pub fn defaults() -> Self {
        Self::at_design(Settings::defaults())
    }

    /// Flying `design` at its own design point (the reduce: shaft speed 1, `π_c` = design).
    pub fn at_design(design: Settings) -> Self {
        FlySettings { design, gas: GasModel::ThermallyPerfect, tt4: design.tt4, t0: design.t0,
                      p0: design.p0, m0: design.m0, map: MapShape::Flow, phi_surge: PHI_SURGE_DEFAULT }
    }

    /// The design the hardware is captured from: the user's, on the fly gas, with a FIXED
    /// CONVERGENT nozzle — capturing hardware needs a throat area (rung 30/31), so the design
    /// numbers here can differ from slice 1's "fully expanded" default. The page shows both.
    pub fn capture(&self) -> Settings {
        Settings { gas: self.gas, nozzle: NozzleMode::Convergent, ..self.design }
    }

    pub fn flight(&self) -> FlightCondition { FlightCondition::new(self.t0, self.p0, self.m0) }

    pub fn to_json(&self) -> Json {
        jobj! {
            "design" => self.design.to_json(), "gas" => self.gas.key(), "Tt4" => self.tt4,
            "T0" => self.t0, "p0" => self.p0, "M0" => self.m0, "map" => self.map.key(),
            "phi_surge" => self.phi_surge,
        }
    }

    /// Read fly settings; a missing key keeps its default. A missing `design` is the opening
    /// design, and a missing flight/throttle is that design's own point.
    pub fn from_json(j: &Json) -> Result<Self, String> {
        let design = match j.get("design") { Some(d) => Settings::from_json(d)?, None => Settings::defaults() };
        let mut s = FlySettings::at_design(design);
        let num = |k: &str, into: &mut f64| -> Result<(), String> {
            match j.get(k) {
                None => Ok(()),
                Some(Json::Float(x)) => { *into = *x; Ok(()) }
                Some(Json::Int(n)) => { *into = *n as f64; Ok(()) }
                Some(other) => Err(format!("setting {k:?} must be a number, got {other:?}")),
            }
        };
        num("Tt4", &mut s.tt4)?;
        num("T0", &mut s.t0)?;
        num("p0", &mut s.p0)?;
        num("M0", &mut s.m0)?;
        num("phi_surge", &mut s.phi_surge)?;
        match j.get("gas") {
            None => {}
            Some(Json::Str(g)) => s.gas = GasModel::from_key(g)?,
            Some(o) => return Err(format!("setting \"gas\" must be text, got {o:?}")),
        }
        match j.get("map") {
            None => {}
            Some(Json::Str(m)) => s.map = MapShape::from_key(m)?,
            Some(o) => return Err(format!("setting \"map\" must be text, got {o:?}")),
        }
        Ok(s)
    }
}

/// The checks made before the fly solve. The design itself must run (slice 1's [`precheck`], on
/// the captured design), and the off-design solver's own limits are named in plain words.
pub fn fly_precheck(s: &FlySettings) -> Result<(), DoesNotRun> {
    let no = |m: &str| Err(DoesNotRun(m.to_string()));
    if s.design.compressor_polytropic || s.design.turbine_polytropic {
        return no("Flying the engine uses the isentropic efficiencies: the off-design solver reads the \
                   compressor and turbine through them. Switch both efficiency knobs to isentropic.");
    }
    precheck(&s.capture()).map_err(|DoesNotRun(m)| DoesNotRun(format!("The design does not run: {m}")))?;
    let named = [("Tt4", s.tt4), ("T0", s.t0), ("p0", s.p0), ("M0", s.m0), ("phi_surge", s.phi_surge)];
    if let Some((k, _)) = named.iter().find(|(_, v)| !v.is_finite()) {
        return Err(DoesNotRun(format!("Setting {k} is not a number.")));
    }
    if s.t0 <= 0.0 || s.p0 <= 0.0 {
        return no("Ambient temperature and pressure must be above zero.");
    }
    if s.m0 <= 0.0 {
        return no("Flight Mach number must be above zero. The model scores efficiency per unit of \
                   flight speed, so a standing engine (Mach 0) is not supported yet.");
    }
    if s.tt4 <= 0.0 {
        return no("The throttle (turbine-inlet temperature) must be above zero.");
    }
    if !(s.phi_surge > 0.0 && s.phi_surge < 1.0) {
        return no("The stall line must sit between 0 and 1 in flow coefficient: below the design \
                   flow (1), above no flow (0).");
    }
    // The throttle must at least beat the air's own temperature at the compressor face: below it no
    // shaft speed can work, and the solver could only say "no workable speed" (measured, plan § 10.4).
    let d = s.capture();
    let (st0, _) = build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses()).try_freestream(&s.flight(), d.mdot)
        .map_err(|e| DoesNotRun(format!("The freestream could not be formed: {}", e.0)))?;
    if s.tt4 <= st0.tt {
        return Err(DoesNotRun(format!(
            "The throttle ({:.0} K) is not above the temperature the air already has entering the compressor \
             ({:.0} K, heated by the flight speed), so the burner would have to cool it before it is even \
             compressed. Raise the throttle.", s.tt4, st0.tt)));
    }
    Ok(())
}

/// The stall margin at a fly point — or why there is none.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stall {
    /// Rung 36's two definitions: the pressure-ratio headroom at constant SPEED and at constant FLOW.
    /// The constant-flow one reads a DIFFERENT speed line, `n·φ_op/φ_surge` — with a low stall line,
    /// a compressor spun at several times design, whose exit the gas tables may refuse. Then it is
    /// `None`: a reading the model cannot make, not a crash.
    Margin { sm_n: f64, sm_flow: Option<f64> },
    /// The nozzle is unchoked: rung 36's surge margin is a choked-branch reading only.
    Unchoked,
    /// The stall line the user set is at or beyond this operating point's flow coefficient.
    PastStallLine,
    /// The stall line sits where the map's speed line does no work (rung 36's `tau_c <= 1` edge).
    OffMap,
}

/// One fly run.
pub struct FlyOutcome {
    pub settings: FlySettings,
    /// The captured design's own run (convergent nozzle, fly gas) — the "before" the page compares.
    pub design: EngineResult,
    /// The operating point, rebuilt forward through the real components.
    pub result: EngineResult,
    /// The spool equilibrium it was rebuilt from.
    pub point: SpoolPoint,
    pub stall: Stall,
    pub ts: Vec<(&'static str, f64, f64)>,
    pub curves: [Vec<(f64, f64)>; 2],
}

/// The fields of rung 34's equilibrium the page reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpoolPoint {
    /// Shaft speed over design.
    pub nu: f64,
    /// Corrected speed and corrected-flow ratio (design = 1), and the flow coefficient `m/n`.
    pub n: f64,
    pub m: f64,
    pub phi: f64,
    pub pi_c: f64,
    pub eta_c: f64,
    pub eta_t: f64,
    pub mdot_air: f64,
    pub thrust: f64,
    pub choked: bool,
}

/// Capture the hardware and build the solver for this flight. A FRESH solver every call: the
/// reacting/equilibrium gases memoise per fuel-air ratio and never forget, so a long-lived solver
/// carries a growing memo (plan § 10.5); a capture costs ~1.5 ms.
///
/// **The nozzle's back-pressure is set to THIS flight's ambient pressure.** The matcher keeps the
/// design run's ambient as `p_ambient` and uses it ONLY as the nozzle's back-pressure (every use
/// in `matcher.rs` / `spool.rs`), while the thrust's pressure term reads the flight's `p0`. Every
/// shipped caller flies off design at the design `p0`, where the two agree; the sandbox is the
/// first to change altitude with the hardware frozen. At the design `p0` this is the shipped solver
/// exactly (`tests/sandbox.rs` pins both facts).
pub fn fly_solver(s: &FlySettings) -> SpoolTransient {
    let d = s.capture();
    let engine = build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses());
    let mut st = SpoolTransient::new(engine, d.flight(), d.mdot, s.map.map());
    st.inner.inner.p_ambient = s.p0;
    st
}

/// The top of rung 34's shaft-speed search (`SpoolTransient::find_equilibrium_nu` marches 0.30–1.60
/// of design) — where [`explain_fly`] tells overspeed from a search cut short.
pub const SPEED_LINE_MAX: f64 = 1.6;

/// `SpoolTransient::pi_c_map`, COPIED with the gas tables' fallible inverse: the original panics when
/// a far speed line's exit leaves the tables, and a panic in the browser kills the whole fly point
/// for one secondary reading. Same arithmetic, same order — `tests/sandbox_fly.rs` holds it equal to
/// the original bit for bit wherever the original answers.
pub fn try_pi_c_map(st: &SpoolTransient, cmap: &ComponentMap, n: f64, phi: f64, tt2: f64) -> Option<f64> {
    let mm = &st.inner.inner;
    let gas = mm.gas();
    let tau_c = 1.0 + (st.inner.tau_c_d - 1.0) * cmap.psi(phi) * n * n;
    if !(tau_c > 1.0) {
        return None;
    }
    let tt3 = tt2 * tau_c;
    let eta_c = cmap.eta_c_at(mm.eta_c, phi, n);
    let (h2, h3) = (gas.h_c(tt2), gas.h_c(tt3));
    let tt3s = gas.try_t_from_h_c(h2 + eta_c * (h3 - h2)).ok()?;
    Some(gas.pr_c(tt3s) / gas.pr_c(tt2))
}

/// Rung 36's margin read off an ALREADY-SOLVED equilibrium — `SpoolTransient::surge_margin`'s own
/// arithmetic without its second equilibrium solve (which doubled the cost), and with its two
/// asserts returned as readings. `tests/sandbox.rs` holds it equal to `surge_margin`.
pub fn stall_reading(st: &SpoolTransient, eq: &crate::spool::Instant, cmap: &ComponentMap) -> Stall {
    if eq.branch != Branch::Choked {
        return Stall::Unchoked;
    }
    let phi_s = cmap.phi_surge;
    if !(phi_s < eq.flowcoef) {
        return Stall::PastStallLine;
    }
    let n_s = eq.flowcoef * eq.n / phi_s;
    let Ok(pn) = st.pi_c_map(cmap, eq.n, phi_s, eq.tt2) else { return Stall::OffMap };
    let sm_flow = try_pi_c_map(st, cmap, n_s, phi_s, eq.tt2).map(|pf| pf / eq.pi_c - 1.0);
    Stall::Margin { sm_n: pn / eq.pi_c - 1.0, sm_flow }
}

/// Fly the design's hardware at `s`'s throttle and flight.
pub fn fly(s: &FlySettings) -> Result<FlyOutcome, DoesNotRun> {
    fly_precheck(s)?;
    let st = fly_solver(s);
    let design = st.inner.inner.reference.clone();
    let fl = s.flight();
    let cmap = s.map.map().with_phi_surge(s.phi_surge);
    let eq = st.equilibrium(&fl, s.tt4, Some(&cmap));
    let stall = stall_reading(&st, &eq, &cmap);
    // The station table: the forward rebuild every matcher ends with, at the equilibrium's
    // (pi_c, mdot, eta_c, eta_t) — it fires every conservation assert on the operating point.
    let m = &st.inner.inner;
    let pi_d = m.pi_d_max * ram_recovery(fl.m0);
    let Rebuilt { state0, v0, s2, s3, s4, s5, exit, gas } =
        m.rebuild(&fl, pi_d, eq.pi_c, s.tt4, eq.mdot_air, eq.eta_c, eq.eta_t);
    let stations = vec![("0", state0), ("2", s2), ("3", s3), ("4", s4), ("5", s5), ("9", exit.state)];
    let performance = score(&gas, &stations, v0, exit.t9, exit.v9, exit.p9, fl.p0, gas.hpr());
    let result = EngineResult { stations, performance, v0, v9: exit.v9, m9: exit.m9, t9: exit.t9, p9: exit.p9 };
    let ts = ts_points(&gas, &result, &fl);
    let curves = ts_curves(&gas, &result, &ts);
    let point = SpoolPoint {
        nu: eq.nu, n: eq.n, m: eq.m, phi: eq.flowcoef, pi_c: eq.pi_c, eta_c: eq.eta_c, eta_t: eq.eta_t,
        mdot_air: eq.mdot_air, thrust: eq.thrust, choked: eq.branch == Branch::Choked,
    };
    Ok(FlyOutcome { settings: *s, design, result, point, stall, ts, curves })
}

/// The page's view of a fly run, FULL precision. The operating point carries slice 1's keys (so
/// the page's station table, T–s diagram and pin & compare read it unchanged) plus the spool's.
pub fn fly_json(o: &FlyOutcome) -> Result<Json, DoesNotRun> {
    let s = &o.settings;
    let p = &o.point;
    // Reuse slice 1's view: an Outcome carrying the operating point, with the air mass flow the
    // ENGINE chose (an output here) in place of the design's.
    let op = Outcome {
        settings: Settings { t0: s.t0, p0: s.p0, m0: s.m0, tt4: s.tt4, pi_c: p.pi_c, mdot: p.mdot_air,
                             gas: s.gas, nozzle: NozzleMode::Convergent, ..s.design },
        result: o.result.clone(), ts: o.ts.clone(), curves: o.curves.clone(),
    };
    let mut j = outcome_json(&op)?;
    let d = &o.design;
    let stall = match o.stall {
        Stall::Margin { sm_n, sm_flow } => jobj! { "kind" => "margin", "sm_n" => sm_n,
                                                    "sm_flow" => sm_flow.map(Json::Float).unwrap_or(Json::Null) },
        Stall::Unchoked => jobj! { "kind" => "unchoked" },
        Stall::PastStallLine => jobj! { "kind" => "past_stall_line" },
        Stall::OffMap => jobj! { "kind" => "off_map" },
    };
    let extra = jobj! {
        "fly" => s.to_json(),
        "nu" => finite(p.nu, "shaft speed")?, "n_corr" => p.n, "m_corr" => p.m, "phi" => p.phi,
        "phi_surge" => s.phi_surge, "eta_c_op" => p.eta_c, "eta_t_op" => p.eta_t,
        "mdot_air" => p.mdot_air, "mdot_ratio" => p.mdot_air / s.design.mdot,
        "spool_thrust" => p.thrust, "choked" => Json::Int(p.choked as i64),
        "stall" => stall,
        "design_point" => jobj! {
            "pi_c" => s.design.pi_c, "Tt4" => s.design.tt4, "mdot" => s.design.mdot,
            "thrust" => d.performance.specific_thrust * s.design.mdot,
            "tsfc" => d.performance.tsfc, "M9" => d.m9,
        },
    };
    if let (Json::Obj(a), Json::Obj(b)) = (&mut j, extra) { a.extend(b); }
    Ok(j)
}

/// One running-line point: the spool equilibrium at the fly throttle, without the forward rebuild.
/// A throttle where the engine does not run panics, as `fly` does — the page reads that as "the
/// running line ends here", not as a broken page.
pub fn running_point_json(s: &FlySettings) -> Json {
    let st = fly_solver(s);
    let cmap = s.map.map().with_phi_surge(s.phi_surge);
    let eq = st.equilibrium(&s.flight(), s.tt4, Some(&cmap));
    let sm = match stall_reading(&st, &eq, &cmap) { Stall::Margin { sm_n, .. } => Json::Float(sm_n), _ => Json::Null };
    jobj! {
        "ok" => Json::Int(1), "Tt4" => s.tt4, "m_corr" => eq.m, "pi_c" => eq.pi_c, "nu" => eq.nu,
        "phi" => eq.flowcoef, "thrust" => eq.thrust, "choked" => Json::Int((eq.branch == Branch::Choked) as i64),
        "sm_n" => sm,
    }
}

/// The throttle grid the running line is drawn on — from well below idle to well above design,
/// so the chart shows where the engine stops running at each flight.
pub fn running_line_grid(s: &FlySettings) -> Vec<f64> {
    let hi = s.design.tt4.max(1000.0) * 1.2;
    (0..=RUNNING_LINE_POINTS).map(|i| 600.0 + (hi - 600.0) * i as f64 / RUNNING_LINE_POINTS as f64).collect()
}

/// Intervals on the running-line grid (so `RUNNING_LINE_POINTS + 1` points).
pub const RUNNING_LINE_POINTS: usize = 16;

/// The compressor map for the chart: speed lines `(n, [(m, π_c)])` and the stall line `[(m, π_c)]`,
/// read through the SAME map arithmetic the operating point uses (`pi_c_map`), at the design inlet.
pub fn map_lines(s: &FlySettings) -> Json {
    let st = fly_solver(s);
    let cmap = s.map.map().with_phi_surge(s.phi_surge);
    let tt2 = st.inner.tt2_d;
    let mut speed = Vec::new();
    let mut stall = Vec::new();
    for k in 0..=7 {
        let n = 0.5 + 0.1 * k as f64;
        let mut line = Vec::new();
        for i in 0..=24 {
            let phi = 0.4 + 1.0 * i as f64 / 24.0;
            if let Some(pc) = try_pi_c_map(&st, &cmap, n, phi, tt2).filter(|pc| pc.is_finite()) {
                line.push(Json::List(vec![Json::Float(phi * n), Json::Float(pc)]));
            }
        }
        speed.push(jobj! { "n" => n, "points" => Json::List(line) });
        if let Some(pc) = try_pi_c_map(&st, &cmap, n, s.phi_surge, tt2).filter(|pc| pc.is_finite()) {
            stall.push(Json::List(vec![Json::Float(s.phi_surge * n), Json::Float(pc)]));
        }
    }
    jobj! { "speed_lines" => Json::List(speed), "stall_line" => Json::List(stall) }
}

// ---------------------------------------------------------------------------------------------
// The page: the browser build is spliced into `docs/sandbox/template.html` as base64, so the page
// is ONE self-contained file that runs from a double-click (`file://` refuses a fetch of a
// sibling `.wasm`) and stays fully offline, like the charts and cutaway pages.

/// Where the template takes the base64 of the browser build.
pub const WASM_PLACEHOLDER: &str = "/*__SANDBOX_WASM_B64__*/";

/// Standard base64 (RFC 4648, with padding) — the crate has no dependency to borrow one from.
pub fn base64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            out.push(if i <= c.len() { A[(n >> shift & 63) as usize] as char } else { '=' });
        }
    }
    out
}

/// The inverse of [`base64`] — the page gate reads the shipped build back out of the page.
pub fn unbase64(text: &str) -> Vec<u8> {
    let val = |c: u8| -> u32 {
        match c {
            b'A'..=b'Z' => (c - b'A') as u32,
            b'a'..=b'z' => (c - b'a' + 26) as u32,
            b'0'..=b'9' => (c - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("base64: bad byte {c:?}"),
        }
    };
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    for q in text.as_bytes().chunks(4) {
        let pad = q.iter().filter(|&&c| c == b'=').count();
        let n = q.iter().take(4 - pad).enumerate().fold(0u32, |n, (i, &c)| n | val(c) << (18 - 6 * i));
        out.extend_from_slice(&[(n >> 16) as u8, (n >> 8) as u8, n as u8][..3 - pad]);
    }
    out
}

/// The built page: the template with the browser build in place of [`WASM_PLACEHOLDER`].
pub fn splice_page(template: &str, wasm: &[u8]) -> String {
    assert_eq!(template.matches(WASM_PLACEHOLDER).count(), 1, "the template must hold the placeholder exactly once");
    template.replacen(WASM_PLACEHOLDER, &base64(wasm), 1)
}

/// The designs the browser check runs natively AND in the browser build, and compares. Every gas,
/// every nozzle, both efficiency spellings, subsonic and supersonic, a high and a low altitude.
pub fn check_grid() -> Vec<Settings> {
    let mut out = Vec::new();
    for g in GasModel::ALL {
        for n in NozzleMode::ALL {
            for &(pi_c, tt4, m0) in &[(10.0, 1500.0, 0.85), (25.0, 1700.0, 0.6), (4.0, 1200.0, 2.0)] {
                for &(t0, p0) in &[(250.0, 50_000.0), (216.65, 19_330.4)] {
                    out.push(Settings { gas: g, nozzle: n, pi_c, tt4, m0, t0, p0, p_exit: 0.9 * p0,
                                        compressor_polytropic: out.len() % 2 == 1,
                                        turbine_polytropic: out.len() % 3 == 1, ..Settings::defaults() });
                }
            }
        }
    }
    out
}

/// The fly points the browser check runs natively AND in the browser build: every gas, three flights
/// (the design's own, cruise at 11 km, supersonic at 11 km), the three map shapes in turn. Every one
/// runs (a native panic would stop the check's native side).
pub fn check_fly_grid() -> Vec<FlySettings> {
    let mut out = Vec::new();
    for g in GasModel::ALL {
        for (i, &(tt4, m0, t0, p0)) in [(1500.0, 0.85, 250.0, 50_000.0), (1200.0, 0.85, 216.65, 22_632.1),
                                        (1700.0, 1.6, 216.65, 22_632.1)].iter().enumerate() {
            out.push(FlySettings { gas: g, tt4, m0, t0, p0, map: MapShape::ALL[(out.len() + i) % 3], ..FlySettings::defaults() });
        }
    }
    out
}

/// One request line per grid design, as the page sends it — led by the page's opening request
/// (empty settings = the defaults), which the browser test also reads back off the page — then the
/// fly grid, its map lines, and two running-line points.
pub fn check_requests() -> Vec<String> {
    let mut out = vec![r#"{"op":"run","settings":{}}"#.to_string()];
    out.extend(check_grid().iter().map(|s| jobj! { "op" => "run", "settings" => s.to_json() }.dump_compact()));
    out.push(r#"{"op":"fly","fly":{}}"#.to_string());       // the page's opening fly point
    let fly = check_fly_grid();
    out.extend(fly.iter().map(|s| jobj! { "op" => "fly", "fly" => s.to_json() }.dump_compact()));
    for m in MapShape::ALL {
        out.push(jobj! { "op" => "map_lines", "fly" => FlySettings { map: m, ..FlySettings::defaults() }.to_json() }.dump_compact());
    }
    for tt4 in [900.0, 1300.0] {
        out.push(jobj! { "op" => "running_point", "fly" => FlySettings { tt4, ..FlySettings::defaults() }.to_json() }.dump_compact());
    }
    out.extend(crate::sandbox_blades::check_requests());
    out
}
