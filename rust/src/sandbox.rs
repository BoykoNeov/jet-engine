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
use crate::engine::{build_turbojet, EngineResult, FlightCondition, Losses};
use crate::gas::{FlowState, Gas};
use crate::jobj;
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
}

/// Run one design. The solve itself is `build_turbojet(…).run(…)`, the CLI's own call.
pub fn run(s: &Settings) -> Result<Outcome, DoesNotRun> {
    precheck(s)?;
    let engine = build_turbojet(s.gas.gas(), s.pi_c, s.tt4, s.p0, s.losses());
    let result = engine.run(&s.flight(), s.mdot);
    let ts = ts_points(&engine.gas, &result, &s.flight());
    Ok(Outcome { settings: *s, result, ts })
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
/// - `{"op":"explain","message":…}` → [`explain`]'s plain words.
/// - `{"op":"range"}` → the atmosphere's altitude range, for the page's slider.
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
            Some(Json::Str(m)) => jobj! { "plain" => explain(m) },
            _ => refusal("explain needs a message"),
        },
        "range" => jobj! { "z_min" => atmosphere::Z_MIN, "z_max" => atmosphere::Z_MAX },
        other => refusal(&format!("unknown op {other:?}")),
    };
    out.dump_compact()
}
