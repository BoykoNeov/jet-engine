//! The web sandbox — **THE ENGINE** (roadmap A9, `docs/plans/a9-cutaway-view-plan.md`). NOT a rung.
//!
//! The animated cutaway, as a view that draws the *Design* view's engine: every number it draws is read
//! from this module's reply, which reads each one off a model the project already has. Nothing here
//! moves a cycle number.
//!
//! * **The stations, the jet, the performance**: the Design view's own run ([`sandbox::run`]).
//! * **The front-zone flame**: the Burner view's design run and pre-check ([`sandbox_burner::design_run`],
//!   [`sandbox_burner::precheck`]) at ITS opening knobs — a stoichiometric front zone (rung 8's design
//!   case) — so on the equilibrium gas, whatever gas the design runs on (that view's rule), then the very
//!   flame solve `zoned_nox` runs, in its fallible spelling ([`nox::try_primary_aft`]; the burner's own
//!   number, `tests/sandbox_cutaway.rs`). Where there is no flame to draw — the whole burner is already
//!   at or past stoichiometric, or the flame is past the model's 3200 K search — the reply says why and
//!   the view draws no separate flame.
//! * **The compressor**: rung 85's sizing ([`blade_speed::size`]) of the ONE compressor, at rung 85's
//!   own default blade knobs, on the map slope of the Fly view's opening map: its integer stage count,
//!   blade speeds and the front row's axial speed.
//! * **The shaft speed**: NEW ARITHMETIC (rung 85 never names a radius). Continuity at the compressor
//!   face, `A = ṁ·√Tt2 / (pt2·MFP(M))` with `M` the front row's design (axial) Mach and the face's `γ, R`;
//!   the annulus `A = π·r_tip²·(1 − h²)`; `N = U_tip / r_tip`. Anchored to NASA Rotor 37
//!   (`tests/sandbox_cutaway.rs`): its published airflow, tip speed and hub-to-tip ratio through this
//!   formula bracket its published 17 188.7 rpm between its own tip and hub inlet Mach numbers.
//! * **The nozzle throat**: the model's sonic throat ([`components::try_sonic_throat`]) on station 9's
//!   total state when the jet reaches Mach 1 inside the nozzle (`p* ≥ p9`); otherwise the throat is the
//!   exit and its temperature `T9`.
//!
//! The turbine's stage count, all geometry and every colour between the computed points stay DRAWN;
//! the page lists them as such. A turbofan or a lit afterburner is refused in plain words: the cutaway
//! draws a plain turbojet.

use crate::blade_speed::{face_props, size, BladeKnobs, Binding, Sizing, SpoolDuty};
use crate::components::try_sonic_throat;
use crate::engine::build_turbojet;
use crate::gas::{f_stoich, hf_fuel_default};
use crate::jobj;
use crate::nox::try_primary_aft;
use crate::sandbox::{self, outcome_json, DoesNotRun, MapShape, Outcome, Settings};
use crate::sandbox_burner::{self, BurnerSettings};
use crate::visuals::Json;

/// The turbine's stage count as DRAWN: no rung sizes a turbine stage.
pub const TURBINE_STAGES_DRAWN: usize = 2;

/// The design with every efficiency and every pressure recovery at 1, on the same gas: the view's
/// "ideal" case. (Rung 1's ideal cycle is also a frozen perfect gas; this one keeps the design's gas,
/// and the inlet keeps its shock loss above Mach 1, which is no knob.)
pub fn ideal(s: &Settings) -> Settings {
    Settings { pi_d_max: 1.0, eta_c: 1.0, e_c: 1.0, eta_t: 1.0, e_t: 1.0, eta_b: 1.0, pi_b: 1.0, eta_m: 1.0, pi_n: 1.0, ..*s }
}

/// Mass-flow parameter `ṁ·√Tt/(A·pt) = M·√(γ/R)·(1 + (γ−1)/2·M²)^(−(γ+1)/(2(γ−1)))`, perfect gas at the face.
pub fn mfp(m: f64, gamma: f64, r: f64) -> f64 {
    m * (gamma / r).sqrt() * (1.0 + 0.5 * (gamma - 1.0) * m * m).powf(-(gamma + 1.0) / (2.0 * (gamma - 1.0)))
}

/// The compressor's tip radius (m) from continuity at its face: annulus `A = ṁ·√Tt/(pt·MFP(M))`,
/// `A = π·r_tip²·(1 − h²)`.
pub fn tip_radius(mdot: f64, tt: f64, pt: f64, m: f64, gamma: f64, r: f64, h: f64) -> f64 {
    let area = mdot * tt.sqrt() / (pt * mfp(m, gamma, r));
    (area / (std::f64::consts::PI * (1.0 - h * h))).sqrt()
}

/// Shaft speed, rpm, from a tip speed (m/s) and a tip radius (m).
pub fn rpm(u_tip: f64, r_tip: f64) -> f64 {
    u_tip / r_tip * 60.0 / (2.0 * std::f64::consts::PI)
}

/// The compressor as sized.
pub struct Compressor {
    pub sizing: Sizing,
    pub vx: f64,
    pub r_tip: f64,
    pub rpm: f64,
}

/// The nozzle throat.
pub struct Throat {
    /// The jet reaches Mach 1 inside the nozzle.
    pub choked: bool,
    pub t: f64,
}

pub struct CutawayOutcome {
    pub run: Outcome,
    pub compressor: Compressor,
    pub throat: Throat,
}

/// The compressor's duty, read off the run: face `Tt2`, work `h(Tt3) − h(Tt2)` on the run's gas, `γ, cp` at the face,
/// the Fly view's opening map slope.
pub fn duty(s: &Settings, run: &Outcome) -> SpoolDuty {
    let gas = s.gas.gas();
    let (tt2, tt3) = (run.result.station("2").tt, run.result.station("3").tt);
    let (gamma, cp) = face_props(&gas, tt2);
    SpoolDuty { tt: tt2, dh: gas.h_c(tt3) - gas.h_c(tt2), l: MapShape::Flow.map().l, gamma, cp }
}

/// Size the compressor and its shaft. A sizing error is a refusal in words.
pub fn compressor(s: &Settings, run: &Outcome) -> Result<Compressor, DoesNotRun> {
    let d = duty(s, run);
    let kn = BladeKnobs::default();
    let sizing = size(d, kn).map_err(|e| DoesNotRun(format!(
        "The compressor cannot be sized at rung 85's blade settings: {e}. The cutaway cannot draw it.")))?;
    let st2 = run.result.station("2");
    let rg = d.cp * (d.gamma - 1.0) / d.gamma;
    let r_tip = tip_radius(s.mdot, st2.tt, st2.pt, sizing.m_abs, d.gamma, rg, kn.h);
    Ok(Compressor { vx: kn.phi_d * sizing.u_m, r_tip, rpm: rpm(sizing.u_tip, r_tip), sizing })
}

/// The front zone's flame temperature at the Burner view's opening knobs, or why there is none.
pub fn flame(s: &Settings) -> Result<f64, String> {
    let (engine, _, inlet) = sandbox_burner::design_run(s).map_err(|e| e.0)?;
    let b = BurnerSettings::defaults();
    sandbox_burner::precheck(&b, &inlet).map_err(|e| e.0)?;
    let hf = engine.gas.spec.hf_fuel_molar.unwrap_or_else(hf_fuel_default);
    try_primary_aft(b.phi_p * f_stoich(), inlet.pt4, inlet.tt3, hf).ok_or_else(|| {
        "The front zone's flame would be past 3200 K, the top of the model's temperature search: the air leaves \
         the compressor too hot. The cutaway draws the burner without a separate flame.".to_string()
    })
}

/// The throat: sonic inside the nozzle when the sonic pressure is at or above the exit's. On the run's own engine
/// gas (an equilibrium gas carries the mixture its burner froze; a fresh one has none).
pub fn throat(s: &Settings, run: &Outcome) -> Result<Throat, DoesNotRun> {
    let engine = build_turbojet(s.gas.gas(), s.pi_c, s.tt4, s.p0, s.losses());
    let again = engine.run(&s.flight(), s.mdot);
    let st9 = again.station("9");
    debug_assert!(st9 == run.result.station("9"));
    let (tstar, pstar, _) = try_sonic_throat(&engine.gas, st9.tt, st9.pt, st9.far)
        .map_err(|e| DoesNotRun(format!("The nozzle's sonic throat could not be found: {}", e.0)))?;
    Ok(if pstar >= run.result.p9 { Throat { choked: true, t: tstar } } else { Throat { choked: false, t: run.result.t9 } })
}

/// One cutaway: the design run, the compressor, the throat. The flame is its OWN request ([`flame`]): it re-runs the
/// design on the equilibrium gas, which can fail inside the model where the design's own gas runs (a hot `Tt4` on
/// another gas) — the page then draws the engine without a flame and says why, instead of drawing nothing.
pub fn cutaway(s: &Settings) -> Result<CutawayOutcome, DoesNotRun> {
    if s.alpha > 0.0 {
        return Err(DoesNotRun("The cutaway draws a plain turbojet. Set the bypass ratio to 0 to see this design drawn \
                               (the Design view shows the turbofan's numbers).".into()));
    }
    if s.afterburner {
        return Err(DoesNotRun("The cutaway draws a plain turbojet. Switch the afterburner off to see this design drawn \
                               (the Design view shows the lit engine's numbers).".into()));
    }
    let run = sandbox::run(s)?;
    let compressor = compressor(s, &run)?;
    let throat = throat(s, &run)?;
    Ok(CutawayOutcome { run, compressor, throat })
}

fn fin(x: f64, what: &str) -> Result<Json, DoesNotRun> {
    if x.is_finite() { Ok(Json::Float(x)) } else { Err(DoesNotRun(format!("The model gave a non-finite {what}."))) }
}

pub fn cutaway_json(o: &CutawayOutcome) -> Result<Json, DoesNotRun> {
    let c = &o.compressor;
    let z = &c.sizing;
    Ok(jobj! {
        "ok" => Json::Int(1),
        "run" => outcome_json(&o.run)?,
        "compressor" => jobj! {
            "stages" => Json::Int(z.k as i64), "k_star" => fin(z.k_star, "stage count")?,
            "u_m" => fin(z.u_m, "blade speed")?, "u_tip" => fin(z.u_tip, "tip speed")?,
            "vx" => fin(c.vx, "axial speed")?, "m_abs" => fin(z.m_abs, "face Mach")?,
            "h" => z.knobs.h, "r_tip" => fin(c.r_tip, "tip radius")?, "rpm" => fin(c.rpm, "shaft speed")?,
            "binding" => match z.binding { Binding::Airflow => "airflow", Binding::Strength => "strength" },
        },
        "turbine_stages" => Json::Int(TURBINE_STAGES_DRAWN as i64),
        "throat" => jobj! { "choked" => Json::Int(o.throat.choked as i64), "t" => fin(o.throat.t, "throat temperature")? },
        "speeds" => jobj! { "v0" => fin(o.run.result.v0, "flight speed")?, "vx" => fin(c.vx, "axial speed")?,
                            "v9" => fin(o.run.result.v9, "jet speed")? },
    })
}

fn refusal(m: &str) -> Json { jobj! { "ok" => Json::Int(0), "reason" => m } }

/// The designs the gates draw: the opening design and two that move every computed number — the stage count, the
/// shaft speed, the flame, the three anchor speeds, the throat (measured: `tests/sandbox_cutaway.rs`).
pub fn check_designs() -> Vec<Settings> {
    let d = Settings::defaults();
    vec![d, Settings { pi_c: 25.0, mdot: 60.0, m0: 0.6, ..d }, Settings { pi_c: 4.0, mdot: 5.0, m0: 1.6, tt4: 1300.0, ..d }]
}

/// The *Engine* view's requests for the browser check, appended LAST in [`crate::sandbox::check_requests`] (every
/// earlier request keeps its number): the page's opening pair, each check design real and ideal with its flame, every
/// gas and nozzle at the opening design, standing still, and the refusals — a turbofan, the afterburner, a flame past
/// the model's search.
pub fn check_requests() -> Vec<String> {
    let req = |op: &str, s: &Settings, ideal: bool| {
        let mut j = jobj! { "op" => op, "settings" => s.to_json() };
        if ideal { if let Json::Obj(kv) = &mut j { kv.push(("ideal".into(), Json::Int(1))); } }
        j.dump_compact()
    };
    let mut out = vec![r#"{"op":"cutaway","settings":{}}"#.to_string(), r#"{"op":"cutaway_flame","settings":{}}"#.to_string()];
    for s in check_designs() {
        for ideal in [false, true] {
            out.push(req("cutaway", &s, ideal));
            out.push(req("cutaway_flame", &s, ideal));
        }
    }
    let d = Settings::defaults();
    for g in sandbox::GasModel::ALL {
        for n in sandbox::NozzleMode::ALL {
            out.push(req("cutaway", &Settings { gas: g, nozzle: n, p_exit: 0.9 * d.p0, ..d }, false));
        }
    }
    out.push(req("cutaway", &Settings { m0: 0.0, ..d }, false));
    out.push(req("cutaway", &Settings { alpha: 1.0, ..d }, false));
    out.push(req("cutaway", &Settings { afterburner: true, ..d }, false));
    out.push(req("cutaway_flame", &Settings { pi_c: 60.0, m0: 3.0, tt4: 2400.0, ..d }, false));
    out
}

/// `cutaway` (`"settings"`: the design; `"ideal"`: 1 for every loss knob at 1) → one cutaway, or a refusal;
/// `cutaway_flame` (the same two keys) → `{"ok":1,"t":…}`, or a refusal naming why there is no flame to draw.
pub fn call_op(op: &str, req: &Json) -> Option<Json> {
    if op != "cutaway" && op != "cutaway_flame" {
        return None;
    }
    let empty = Json::Obj(Vec::new());
    let s = match Settings::from_json(req.get("settings").unwrap_or(&empty)) {
        Ok(s) => s,
        Err(e) => return Some(refusal(&e)),
    };
    let s = if matches!(req.get("ideal"), Some(Json::Int(1))) { ideal(&s) } else { s };
    Some(if op == "cutaway_flame" {
        match flame(&s) {
            Ok(t) if t.is_finite() => jobj! { "ok" => Json::Int(1), "t" => t },
            Ok(_) => refusal("The model gave a non-finite flame temperature."),
            Err(m) => refusal(&m),
        }
    } else {
        cutaway(&s).and_then(|o| cutaway_json(&o)).unwrap_or_else(|DoesNotRun(m)| refusal(&m))
    })
}
