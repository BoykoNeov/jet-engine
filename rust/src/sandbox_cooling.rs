//! The web sandbox — **COOL THE TURBINE** (`docs/plans/sandbox-plan.md` § 22): rung 93 as a view. NOT
//! one of the plan's five slices; it is its own view because the trade it shows is steady, not a slam
//! (the user's choice, 2026-10-16).
//!
//! The engine is the *Controls* engine (the *Size the blades* engine at its opening settings, the
//! perfect gas, rig A of rungs 42/90–93): `π_LPC` 3 × `π_HPC` 6, design `Tt4` 1500 K, at 250 K / 50 kPa /
//! Mach 0.85. **Two framings, a switch** (the user's): *this engine, run hotter* — the hardware fixed,
//! sized uncooled at 1500 K ([`FixedEngine`]) — or *redesigned at each temperature* — a fresh design
//! whose turbines are cooled from the start ([`mt::try_limited_design`]).
//!
//! **The limits come from a material YEAR and a part LIFE** (the user's): Gauntner's relation,
//! [`mt::allowable`] — 1985 and 10 000 hours give 1227.8 K (vane) and 1172.2 K (blade).
//!
//! **Requests.** `cooling_point` answers ONE burner temperature of the sweep (milliseconds: a fixed
//! point through rung 91's matcher, 8–40 passes), so the page streams [`sweep_grid`] as the other
//! views stream theirs; `cooling_knob` answers the knob form on the fixed engine — the user's cooling
//! shares in, the hottest burner temperature the metal allows out.

use crate::jobj;
use crate::metal_temperature::{self as mt, FixedEngine, MetalLimit, Row, TECHNOLOGIES};
use crate::sandbox::DoesNotRun;
use crate::sandbox_blades::{rig_losses, BladeSettings};
use crate::sandbox_controls::{control_maps, CONTROL_SHAPES};
use crate::turbine_cooling::TurbineCooling;
use crate::visuals::Json;

/// The sweep's burner temperatures, K: 1300 → 2200 by 50 (rung 93's scored band).
pub const SWEEP_LO: f64 = 1300.0;
pub const SWEEP_HI: f64 = 2200.0;
pub const SWEEP_STEP: f64 = 50.0;

pub fn sweep_grid() -> Vec<f64> {
    let n = ((SWEEP_HI - SWEEP_LO) / SWEEP_STEP).round() as usize;
    (0..=n).map(|k| SWEEP_LO + SWEEP_STEP * k as f64).collect()
}

/// The knob form's search range, K.
pub const KNOB_LO: f64 = 700.0;
pub const KNOB_HI: f64 = 2400.0;
/// The grid the search's low end steps up on, K.
pub const KNOB_STEP: f64 = 50.0;

/// The material years and part lives the sliders offer.
pub const YEAR_MIN: f64 = 1950.0;
pub const YEAR_MAX: f64 = 2030.0;
pub const LIFE_MIN: f64 = 100.0;
pub const LIFE_MAX: f64 = 1.0e6;

/// The two framings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Framing {
    /// This engine, run hotter: rung 91's fixed hardware.
    Fixed,
    /// Redesigned at each temperature: a fresh cooled design.
    Fresh,
}

impl Framing {
    pub const ALL: [Framing; 2] = [Framing::Fixed, Framing::Fresh];
    pub fn key(self) -> &'static str {
        match self { Framing::Fixed => "fixed", Framing::Fresh => "fresh" }
    }
}

/// The technology keys the page sends (Gauntner's FACTORs, [`TECHNOLOGIES`]).
pub const TECH_KEYS: [&str; 4] = ["convection", "coated", "film", "transpiration"];

/// Every knob of the view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoolingSettings {
    pub framing: Framing,
    /// Index into [`TECHNOLOGIES`].
    pub tech: usize,
    pub year: f64,
    pub life: f64,
    /// Index into [`CONTROL_SHAPES`] — the fixed engine's compressor maps (a fresh design runs at its own
    /// design point, where the map does not enter).
    pub shape: usize,
    /// The knob form's cooling shares on the fixed engine (of the HP compressor's air).
    pub vane: f64,
    pub blade: f64,
}

impl CoolingSettings {
    /// Full-coverage film, Gauntner's 1985 materials at 10 000 hours, 5 % vane and 3 % blade air.
    pub fn defaults() -> Self {
        CoolingSettings { framing: Framing::Fixed, tech: 2, year: 1985.0, life: 1.0e4, shape: 0, vane: 0.05, blade: 0.03 }
    }

    pub fn limit(&self) -> MetalLimit {
        MetalLimit::new(TECHNOLOGIES[self.tech].1, mt::allowable(Row::Vane, self.year, self.life),
                        mt::allowable(Row::Blade, self.year, self.life))
    }

    pub fn to_json(&self) -> Json {
        jobj! {
            "framing" => self.framing.key(), "tech" => TECH_KEYS[self.tech], "year" => self.year, "life" => self.life,
            "shape" => CONTROL_SHAPES[self.shape], "vane" => self.vane, "blade" => self.blade,
        }
    }

    /// Read settings; a missing key keeps its default.
    pub fn from_json(j: &Json) -> Result<Self, String> {
        let mut s = CoolingSettings::defaults();
        let text = |k: &str| -> Result<Option<String>, String> {
            match j.get(k) {
                None => Ok(None),
                Some(Json::Str(t)) => Ok(Some(t.clone())),
                Some(o) => Err(format!("setting {k:?} must be text, got {o:?}")),
            }
        };
        for (k, into) in [("year", &mut s.year), ("life", &mut s.life), ("vane", &mut s.vane), ("blade", &mut s.blade)] {
            match j.get(k) {
                None => {}
                Some(Json::Float(x)) => *into = *x,
                Some(Json::Int(n)) => *into = *n as f64,
                Some(o) => return Err(format!("setting {k:?} must be a number, got {o:?}")),
            }
        }
        if let Some(t) = text("framing")? {
            s.framing = Framing::ALL.into_iter().find(|f| f.key() == t)
                .ok_or_else(|| format!("unknown framing {t:?} (\"fixed\" or \"fresh\")"))?;
        }
        if let Some(t) = text("tech")? {
            s.tech = TECH_KEYS.iter().position(|&k| k == t).ok_or_else(|| format!(
                "unknown cooling technology {t:?} (\"convection\", \"coated\", \"film\" or \"transpiration\")"))?;
        }
        if let Some(t) = text("shape")? {
            s.shape = CONTROL_SHAPES.iter().position(|&k| k == t).ok_or_else(|| format!("unknown map shape {t:?}"))?;
        }
        Ok(s)
    }

    /// The fixed engine — the Controls engine's design, with this view's maps.
    pub fn fixed(&self) -> FixedEngine {
        let b = BladeSettings::defaults();
        let (ml, mh) = control_maps(CONTROL_SHAPES[self.shape]).expect("CONTROL_SHAPES lists only buildable shapes");
        FixedEngine::new(b.design(), b.flight(), 1.0, ml, mh)
    }
}

/// The checks made before the model runs, in plain words.
pub fn cooling_precheck(s: &CoolingSettings) -> Result<(), DoesNotRun> {
    let no = |m: &str| Err(DoesNotRun(m.to_string()));
    for (k, v) in [("year", s.year), ("life", s.life), ("vane", s.vane), ("blade", s.blade)] {
        if !v.is_finite() {
            return Err(DoesNotRun(format!("Setting {k} is not a number.")));
        }
    }
    if !(YEAR_MIN..=YEAR_MAX).contains(&s.year) {
        return Err(DoesNotRun(format!(
            "The material year must be between {YEAR_MIN:.0} and {YEAR_MAX:.0}: Gauntner's relation is a straight-line \
             trend (about 5.6 K a year) fitted to the materials of his time, not a law to extrapolate far.")));
    }
    if !(LIFE_MIN..=LIFE_MAX).contains(&s.life) {
        return no("The part life must be between 100 and 1 000 000 hours.");
    }
    let quarter = |x: f64| (0.0..TurbineCooling::SHARE_MAX).contains(&x);
    if !quarter(s.vane) || !quarter(s.blade) {
        return no("Each turbine-cooling share must be at least 0 and below 0.25 of the HP compressor's air.");
    }
    Ok(())
}

fn num_or_null(x: f64) -> Json { if x.is_finite() { Json::Float(x) } else { Json::Null } }

/// One burner temperature of the sweep, in the settings' framing: the cooling that holds both metals
/// at their limits, what it costs, and the same engine uncooled at the same temperature.
pub fn cooling_point(s: &CoolingSettings, tt4: f64) -> Result<Json, DoesNotRun> {
    cooling_precheck(s)?;
    if !tt4.is_finite() {
        return Err(DoesNotRun("The burner temperature is not a number.".into()));
    }
    let lim = s.limit();
    let b = BladeSettings::defaults();
    let fl = b.flight();
    let words = |e: crate::gas::Abort| DoesNotRun(point_words(&e.0, tt4));
    match s.framing {
        Framing::Fixed => {
            let fe = s.fixed();
            let l = fe.try_limited(&lim, &fl, tt4).map_err(words)?;
            let k = l.point.booking.expect("a limited point is cooled");
            let r = &l.point.base;
            // The same engine uncooled at the same burner temperature (its metal far past the limit).
            let bare = fe.try_point(TurbineCooling::new(0.0, 0.0), &fl, tt4).ok();
            let (st0, tsfc0) = bare.as_ref()
                .map(|p| (p.base.base.thrust / p.base.base.mdot_air, p.base.base.performance.tsfc))
                .unwrap_or((f64::NAN, f64::NAN));
            let (ml, mh) = control_maps(CONTROL_SHAPES[s.shape]).expect("buildable");
            Ok(jobj! {
                "ok" => Json::Int(1), "framing" => "fixed", "Tt4" => tt4, "vane" => l.vane, "blade" => l.blade,
                "st" => k.st_inlet, "tsfc" => k.tsfc_inlet, "st_bare" => num_or_null(st0), "tsfc_bare" => num_or_null(tsfc0),
                "Tt3" => l.inputs.tt3, "Tt41" => l.inputs.tt41, "metal_vane" => l.reading.vane,
                "metal_blade" => l.reading.blade, "passes" => Json::Int(l.passes as i64),
                "n_lp" => r.n_lp_ratio, "n_hp" => r.n_hp_ratio, "phi_lp" => r.phi_lp, "phi_hp" => r.phi_hp,
                "stall_lp" => ml.phi_surge_at(), "stall_hp" => mh.phi_surge_at(),
            })
        }
        Framing::Fresh => {
            let l = mt::try_limited_design(&b.gas.gas(), b.pi_lpc, b.pi_hpc, tt4, fl.p0, rig_losses(), &lim, &fl, 1.0)
                .map_err(words)?;
            let bare = mt::try_cooled_design_run(&b.gas.gas(), b.pi_lpc, b.pi_hpc, tt4, fl.p0, rig_losses(), None, &fl, 1.0)
                .map_err(words)?;
            Ok(jobj! {
                "ok" => Json::Int(1), "framing" => "fresh", "Tt4" => tt4, "vane" => l.vane, "blade" => l.blade,
                "st" => l.run.st_inlet, "tsfc" => l.run.tsfc_inlet, "st_bare" => bare.st_inlet, "tsfc_bare" => bare.tsfc_inlet,
                "Tt3" => l.inputs.tt3, "Tt41" => l.inputs.tt41, "metal_vane" => l.reading.vane,
                "metal_blade" => l.reading.blade, "passes" => Json::Int(l.passes as i64),
            })
        }
    }
}

/// The knob form on the fixed engine: the hottest burner temperature the metal allows with the
/// settings' shares, and — for scale — with none.
pub fn cooling_knob(s: &CoolingSettings) -> Result<Json, DoesNotRun> {
    cooling_precheck(s)?;
    let lim = s.limit();
    let fl = BladeSettings::defaults().flight();
    let fe = s.fixed();
    let one = |cool: TurbineCooling| -> Json {
        // The search starts at the coolest burner temperature (on a 50 K grid from KNOB_LO) where this engine solves
        // with this cooling: much cooling unchokes the nozzle at a low throttle (the two-spool model's open seam), and a
        // search that started there would refuse a hottest temperature far above it (the crash map, plan § 22).
        let mut lo = KNOB_LO;
        while lo < KNOB_HI && fe.try_point(cool, &fl, lo).is_err() {
            lo += KNOB_STEP;
        }
        if lo >= KNOB_HI {
            return jobj! { "ok" => Json::Int(0), "reason" => format!(
                "With this cooling the engine has no burner temperature from {KNOB_LO:.0} to {KNOB_HI:.0} K that the model \
                 solves (its nozzle unchokes, a case the two-shaft model does not handle).") };
        }
        match fe.try_max_tt4(cool, &lim, &fl, lo, KNOB_HI) {
            Ok(m) => {
                let (st, tsfc) = match m.point.booking {
                    Some(k) => (k.st_inlet, k.tsfc_inlet),
                    None => (m.point.base.base.thrust / m.point.base.base.mdot_air, m.point.base.base.performance.tsfc),
                };
                jobj! {
                    "ok" => Json::Int(1), "Tt4" => m.tt4, "binding" => m.binding.name(), "metal_vane" => m.reading.vane,
                    "metal_blade" => m.reading.blade, "st" => st, "tsfc" => tsfc, "n_lp" => m.point.base.n_lp_ratio,
                    "n_hp" => m.point.base.n_hp_ratio,
                }
            }
            Err(e) => jobj! { "ok" => Json::Int(0), "reason" => knob_words(&e.0, lo) },
        }
    };
    Ok(jobj! {
        "ok" => Json::Int(1), "yours" => one(TurbineCooling::new(s.vane, s.blade)),
        "bare" => one(TurbineCooling::new(0.0, 0.0)),
        "limit_vane" => lim.vane, "limit_blade" => lim.blade, "factor" => lim.factor,
    })
}

/// Plain words for a sweep point the model refuses. The model's own sentence is kept after them.
pub fn point_words(message: &str, tt4: f64) -> String {
    if message.contains("cannot hold the metal") {
        format!("At {tt4:.0} K this cooling technology would need more than a quarter of the compressor's air in one \
                 stream to hold the metal - more than the model lets a stream take. Pick a better technology or a newer \
                 material, or stay cooler. The model says: {message}")
    } else if message.contains("no amount of that air") {
        format!("At {tt4:.0} K the metal limit is below the cooling air's own temperature, so no air can hold it. A newer \
                 material raises the limit. The model says: {message}")
    } else if message.contains("UNCHOKED") || message.contains("unchoked") {
        format!("At {tt4:.0} K, with the cooling this needs, the engine's nozzle stops choking - a case the two-shaft \
                 model does not solve. The model says: {message}")
    } else {
        format!("At {tt4:.0} K the model found no answer. The model says: {message}")
    }
}

/// Plain words for the knob form's refusals; `lo` is where the search started.
pub fn knob_words(message: &str, lo: f64) -> String {
    if message.contains("over its limit") {
        let start = if lo > KNOB_LO { " (the coolest the engine solves at with this cooling)".to_string() } else { String::new() };
        format!("Even at {lo:.0} K{start} the metal is over its limit with this cooling. {message}")
    } else if message.contains("holds all the way") {
        format!("With this cooling the metal holds all the way to {KNOB_HI:.0} K, the top of the range the view searches.")
    } else {
        format!("The model found no answer. The model says: {message}")
    }
}

/// Plain words for a model CRASH in this view (`explain` with `"view":"cooling"`).
pub fn explain_cooling(message: &str) -> String {
    format!("The model failed in a solve this view does not guard. The model says: {message}")
}

/// The requests the browser check compares with the native model.
pub fn check_requests() -> Vec<String> {
    let d = CoolingSettings::defaults();
    let pt = |s: CoolingSettings, t: f64| jobj! { "op" => "cooling_point", "cooling" => s.to_json(), "Tt4" => t }.dump_compact();
    let kb = |s: CoolingSettings| jobj! { "op" => "cooling_knob", "cooling" => s.to_json() }.dump_compact();
    vec![
        r#"{"op":"cooling_defaults"}"#.to_string(),
        // The page's opening requests, in the form the browser check reads them back by.
        r#"{"op":"cooling_knob","cooling":{}}"#.to_string(),
        r#"{"op":"cooling_point","cooling":{},"Tt4":1300}"#.to_string(),
        r#"{"op":"cooling_point","cooling":{},"Tt4":1750}"#.to_string(),
        r#"{"op":"cooling_point","cooling":{},"Tt4":2200}"#.to_string(),
        pt(d, 1300.0), pt(d, 1750.0), pt(d, 2200.0),
        pt(CoolingSettings { framing: Framing::Fresh, ..d }, 1750.0),
        pt(CoolingSettings { tech: 0, shape: 1, ..d }, 2200.0),
        pt(CoolingSettings { tech: 3, framing: Framing::Fresh, year: 2005.0, ..d }, 1500.0),
        pt(CoolingSettings { year: 1950.0, life: 1.0e6, tech: 0, ..d }, 2200.0),       // refused: the stream cap
        pt(CoolingSettings { year: 1900.0, ..d }, 1500.0),                               // refused: year
        kb(d), kb(CoolingSettings { vane: 0.0, blade: 0.0, ..d }), kb(CoolingSettings { vane: 0.2, blade: 0.2, ..d }),
    ]
}

/// `cooling_defaults`, `cooling_point`, `cooling_knob` — the dispatcher `sandbox::call` falls to.
pub fn call_op(op: &str, req: &Json) -> Option<Json> {
    let refusal = |m: &str| jobj! { "ok" => Json::Int(0), "reason" => m };
    let empty = Json::Obj(Vec::new());
    let settings = || CoolingSettings::from_json(req.get("cooling").unwrap_or(&empty));
    Some(match op {
        "cooling_defaults" => jobj! {
            "cooling" => CoolingSettings::defaults().to_json(),
            "grid" => Json::List(sweep_grid().into_iter().map(Json::Float).collect()),
            "technologies" => Json::List(TECH_KEYS.iter().zip(TECHNOLOGIES).map(|(k, (name, f))|
                jobj! { "key" => *k, "name" => name, "factor" => f }).collect()),
            "shapes" => Json::List(CONTROL_SHAPES.iter().map(|&s| Json::from(s)).collect()),
            "year_min" => YEAR_MIN, "year_max" => YEAR_MAX, "life_min" => LIFE_MIN, "life_max" => LIFE_MAX,
        },
        "cooling_point" => match settings() {
            Err(e) => refusal(&e),
            Ok(s) => {
                let t = match req.get("Tt4") { Some(Json::Float(x)) => *x, Some(Json::Int(n)) => *n as f64, _ => f64::NAN };
                cooling_point(&s, t).unwrap_or_else(|DoesNotRun(m)| refusal(&m))
            }
        },
        "cooling_knob" => match settings() {
            Err(e) => refusal(&e),
            Ok(s) => cooling_knob(&s).unwrap_or_else(|DoesNotRun(m)| refusal(&m)),
        },
        _ => return None,
    })
}
