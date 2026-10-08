//! The web sandbox, slice 4 (B) — **CONTROLS** (`docs/plans/sandbox-plan.md` § 12). NOT a rung.
//!
//! The throttle slam of slice 4 (A), on a TWO-shaft engine, with the fuel controls and one airflow
//! lever as switches. Every fuel limiter (rungs 46–52) and every airflow lever on the transient
//! (57–63) is two-shaft by assertion — its finding is a split BETWEEN spools — so this view has its
//! own engine (plan § 12.1): the *Size the blades* view's, at its opening settings (rung 43's rig:
//! `π_LPC` 3 × `π_HPC` 6, `Tt4` 1500, a convergent nozzle, the panels' two-spool losses, at 250 K /
//! 50 kPa / Mach 0.85, the perfect gas), on rungs 43/45's fuel-metered march.
//!
//! **The switches** (plan § 12.3, the user's answers § 12.8): the turbine-temperature limiter
//! (rung 46) and its response lag (47); the acceleration schedule (48); the stall floor (49), read
//! as a flow coefficient or — with the stators moving — as an incidence (60); the realistic
//! fast-attack / slow-release lag (52); and ONE airflow lever: the stator schedule (57) or the bleed
//! schedule (62). The fuel follows a ramp between the two endpoints' steady fuel flows, exactly as
//! rung 57's own march (`r57_stator_march`) spells it, and the limiters cut it.
//!
//! **Time is in the HP spool's time constant** (plan § 12.8 Q4); `ρ` is how much slower the LP
//! shaft responds.
//!
//! **A run that stops early is a RESULT** (slice 4 (A)'s rule). The marchers `break` and drop the
//! error, so [`stop_reason`] re-runs the failed step through the PUBLIC pieces the march itself
//! calls (`try_instant_fuel` and the legs' set-point solves), in its order and arithmetic — the plain
//! min-select march and rung 52's lagged one, which records its third state. Rung 47's lagged
//! governor does NOT record its third state, so a stop on that route is reported by where it
//! happened, never by a guessed cause. `tests/sandbox_controls.rs` holds the re-run to every
//! recorded step, bit for bit.

use crate::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use crate::blade_speed::shape_maps;
use crate::engine::FlightCondition;
use crate::fuel_transient::{release_weight, AccelSchedule, AsymmetricLag, FuelInstant, FuelLimiters,
                            FuelPoint, FuelTransientCore, PointExtra, SurgeLimiter};
use crate::gas::Abort;
use crate::jobj;
use crate::map::ComponentMap;
use crate::matcher::Branch;
use crate::sandbox::DoesNotRun;
use crate::sandbox_blades::BladeSettings;
use crate::stator_transient::{IncidenceLimiter, ScheduledStatorCore, ScheduledStatorTransient, StatorArm,
                              StatorSchedule};
use crate::two_spool::Spool;
use crate::two_spool_transient::Instant2;
use crate::visuals::Json;

/// The march step, in HP spool time constants — rung 43's grid. (Rungs 57/62 ran at 0.01; the
/// step is checked against it in the crash map, plan § 12.5.)
pub const DS: f64 = 0.02;

/// The longest run the page asks for, in spool time constants — a bound on one request's cost (the
/// crash map's slowest requests, all long runs with limiters armed, took 1–3.7 s natively).
pub const MAX_RUN: f64 = 10.0;

/// The range of `ρ` the step was checked over (plan § 12.10: at 0.2, 1 and 5 the trajectory at step
/// 0.02 agrees with 0.005 to ~1e-4 in flow coefficient).
pub const RHO_MIN: f64 = 0.2;
pub const RHO_MAX: f64 = 5.0;

/// How many rows rung 48's derived acceleration schedule reads off the running line — rung 48's own.
pub const ACCEL_ROWS: usize = 13;

/// The design airflow the hardware is captured at, kg/s — every two-spool rung's 1.0. Thrust is
/// therefore read per kg/s of design airflow.
pub const MDOT_DESIGN: f64 = 1.0;

/// Rung 43's three map shapes (`tests/rung43.rs` `shapes()`, which are rung 55's first three —
/// [`shape_maps`]) and rung 49's `flat-lp`: the LP compressor's map flat, the HP's as `flow/press`.
pub const CONTROL_SHAPES: [&str; 4] = ["flow/press", "press/flow", "tilted", "flat-lp"];

/// The `(LP, HP)` maps of one of [`CONTROL_SHAPES`], each with rung 41's stall line at 0.55.
pub fn control_maps(name: &str) -> Option<(ComponentMap, ComponentMap)> {
    match name {
        "flat-lp" => {
            let (_, hp) = shape_maps("flow/press")?;
            Some((ComponentMap::flat().with_phi_surge(hp.phi_surge), hp))
        }
        n if CONTROL_SHAPES.contains(&n) => shape_maps(n),
        _ => None,
    }
}

/// The ONE airflow lever (plan § 12.3: stator + bleed + a fuel limiter on one plant is open physics).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeverChoice {
    None,
    /// Rung 57: the variable stators close at low corrected speed on ONE spool.
    Stator,
    /// Rung 62: the handling-bleed valve opens at low LP corrected speed.
    Bleed,
}

impl LeverChoice {
    pub const ALL: [LeverChoice; 3] = [LeverChoice::None, LeverChoice::Stator, LeverChoice::Bleed];
    pub fn key(self) -> &'static str {
        match self { LeverChoice::None => "none", LeverChoice::Stator => "stator", LeverChoice::Bleed => "bleed" }
    }
}

/// What the stall floor watches (plan § 12.3): the flow coefficient (rung 49), or the blade
/// incidence (rung 60) — the coordinate whose wall the stators do not move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRef {
    Phi,
    Incidence,
}

impl FloorRef {
    pub const ALL: [FloorRef; 2] = [FloorRef::Phi, FloorRef::Incidence];
    pub fn key(self) -> &'static str {
        match self { FloorRef::Phi => "phi", FloorRef::Incidence => "incidence" }
    }
}

fn spool_key(s: Spool) -> &'static str {
    match s { Spool::Lp => "lp", Spool::Hp => "hp" }
}

/// Every knob of the Controls view. A switch keeps its numbers while it is off, as the page does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlsSettings {
    /// Index into [`CONTROL_SHAPES`].
    pub shape: usize,
    /// `τ_LP / τ_HP` — how much slower the LP shaft responds.
    pub rho: f64,
    /// The throttle before and after the move: the turbine-inlet temperature each steady point
    /// holds, K. The FUEL moves between their steady fuel flows.
    pub from: f64,
    pub to: f64,
    /// How long the fuel move takes, and how long the run continues after it, in HP spool τ.
    pub ramp: f64,
    pub settle: f64,
    /// Rung 46: the turbine-temperature limiter's redline, K.
    pub redline_on: bool,
    pub redline: f64,
    /// Rung 47: that limiter's response lag, τ.
    pub gov_lag_on: bool,
    pub gov_lag: f64,
    /// Rung 48: the acceleration schedule's margin above the steady fuel-per-pressure line.
    pub accel_on: bool,
    pub accel_margin: f64,
    /// Rungs 49/60: the stall floor — which spool, the flow coefficient it holds (at the design stator
    /// setting), and what it watches.
    pub floor_on: bool,
    pub floor_spool: Spool,
    pub floor_phi: f64,
    pub floor_ref: FloorRef,
    /// Rung 52: the fast-attack / slow-release lag on the floor and the schedule, τ.
    pub release_on: bool,
    pub tau_att: f64,
    pub tau_rel: f64,
    /// The airflow lever, and its schedule: closed/open fully below `n_lo`, back to design at speed 1.
    pub lever: LeverChoice,
    pub stator_spool: Spool,
    pub v_max: f64,
    pub v_n_lo: f64,
    pub b_max: f64,
    pub b_n_lo: f64,
}

impl ControlsSettings {
    /// Rung 48/49's ramp — 1000 → 1400 K over half a spool time — every switch off, the numbers each
    /// rung used when it was switched on: redline 1480 K (rungs 46–49), lag 0.2 τ (47), margin 0.25
    /// (48), LP floor 0.75 (49), attack/release 0.02/0.10 τ (52), stator 0.20 below 0.65 and bleed
    /// 0.10 below 0.65 (62).
    pub fn defaults() -> Self {
        ControlsSettings {
            shape: 0, rho: 1.0, from: 1000.0, to: 1400.0, ramp: 0.5, settle: 2.0,
            redline_on: false, redline: 1480.0, gov_lag_on: false, gov_lag: 0.2,
            accel_on: false, accel_margin: 0.25,
            floor_on: false, floor_spool: Spool::Lp, floor_phi: 0.75, floor_ref: FloorRef::Phi,
            release_on: false, tau_att: 0.02, tau_rel: 0.10,
            lever: LeverChoice::None, stator_spool: Spool::Lp, v_max: 0.20, v_n_lo: 0.65, b_max: 0.10, b_n_lo: 0.65,
        }
    }

    pub fn flight(&self) -> FlightCondition { BladeSettings::defaults().flight() }

    pub fn s_end(&self) -> f64 { self.ramp + self.settle }

    /// How many points a march that runs to the end records — the marcher's own step count.
    pub fn expected_points(&self) -> usize { (self.s_end() / DS).round_ties_even() as usize + 1 }

    pub fn to_json(&self) -> Json {
        let b = |x: bool| Json::Int(x as i64);
        jobj! {
            "shape" => CONTROL_SHAPES[self.shape], "rho" => self.rho, "from" => self.from, "to" => self.to,
            "ramp" => self.ramp, "settle" => self.settle,
            "redline_on" => b(self.redline_on), "redline" => self.redline,
            "gov_lag_on" => b(self.gov_lag_on), "gov_lag" => self.gov_lag,
            "accel_on" => b(self.accel_on), "accel_margin" => self.accel_margin,
            "floor_on" => b(self.floor_on), "floor_spool" => spool_key(self.floor_spool),
            "floor_phi" => self.floor_phi, "floor_ref" => self.floor_ref.key(),
            "release_on" => b(self.release_on), "tau_att" => self.tau_att, "tau_rel" => self.tau_rel,
            "lever" => self.lever.key(), "stator_spool" => spool_key(self.stator_spool),
            "v_max" => self.v_max, "v_n_lo" => self.v_n_lo, "b_max" => self.b_max, "b_n_lo" => self.b_n_lo,
        }
    }

    /// Read settings; a missing key keeps its default.
    pub fn from_json(j: &Json) -> Result<Self, String> {
        let mut s = ControlsSettings::defaults();
        let num = |k: &str, into: &mut f64| -> Result<(), String> {
            match j.get(k) {
                None => Ok(()),
                Some(Json::Float(x)) => { *into = *x; Ok(()) }
                Some(Json::Int(n)) => { *into = *n as f64; Ok(()) }
                Some(o) => Err(format!("setting {k:?} must be a number, got {o:?}")),
            }
        };
        let flag = |k: &str, into: &mut bool| -> Result<(), String> {
            match j.get(k) {
                None => Ok(()),
                Some(Json::Int(n)) if *n == 0 || *n == 1 => { *into = *n == 1; Ok(()) }
                Some(o) => Err(format!("switch {k:?} must be 0 or 1, got {o:?}")),
            }
        };
        let text = |k: &str| -> Result<Option<String>, String> {
            match j.get(k) {
                None => Ok(None),
                Some(Json::Str(t)) => Ok(Some(t.clone())),
                Some(o) => Err(format!("setting {k:?} must be text, got {o:?}")),
            }
        };
        let spool = |t: &str| -> Result<Spool, String> {
            [Spool::Lp, Spool::Hp].into_iter().find(|&v| spool_key(v) == t)
                .ok_or_else(|| format!("unknown spool {t:?} (\"lp\" or \"hp\")"))
        };
        if let Some(t) = text("shape")? {
            s.shape = CONTROL_SHAPES.iter().position(|&k| k == t).ok_or_else(|| format!("unknown map shape {t:?}"))?;
        }
        for (k, into) in [("rho", &mut s.rho), ("from", &mut s.from), ("to", &mut s.to), ("ramp", &mut s.ramp),
                          ("settle", &mut s.settle), ("redline", &mut s.redline), ("gov_lag", &mut s.gov_lag),
                          ("accel_margin", &mut s.accel_margin), ("floor_phi", &mut s.floor_phi),
                          ("tau_att", &mut s.tau_att), ("tau_rel", &mut s.tau_rel), ("v_max", &mut s.v_max),
                          ("v_n_lo", &mut s.v_n_lo), ("b_max", &mut s.b_max), ("b_n_lo", &mut s.b_n_lo)] {
            num(k, into)?;
        }
        for (k, into) in [("redline_on", &mut s.redline_on), ("gov_lag_on", &mut s.gov_lag_on),
                          ("accel_on", &mut s.accel_on), ("floor_on", &mut s.floor_on),
                          ("release_on", &mut s.release_on)] {
            flag(k, into)?;
        }
        if let Some(t) = text("floor_spool")? { s.floor_spool = spool(&t)?; }
        if let Some(t) = text("stator_spool")? { s.stator_spool = spool(&t)?; }
        if let Some(t) = text("floor_ref")? {
            s.floor_ref = FloorRef::ALL.into_iter().find(|r| r.key() == t)
                .ok_or_else(|| format!("unknown floor reference {t:?} (\"phi\" or \"incidence\")"))?;
        }
        if let Some(t) = text("lever")? {
            s.lever = LeverChoice::ALL.into_iter().find(|l| l.key() == t)
                .ok_or_else(|| format!("unknown lever {t:?} (\"none\", \"stator\" or \"bleed\")"))?;
        }
        Ok(s)
    }

    /// The airflow lever as the model's constructor takes it.
    pub fn lever_arm(&self) -> LeverArm {
        match self.lever {
            LeverChoice::None => LeverArm::default(),
            LeverChoice::Stator => {
                let sch = StatorSchedule::new(self.v_max, self.v_n_lo);
                LeverArm::stator(match self.stator_spool {
                    Spool::Lp => StatorArm::scheduled_lp(sch),
                    Spool::Hp => StatorArm::scheduled_hp(sch),
                })
            }
            LeverChoice::Bleed => LeverArm::scheduled(BleedSchedule::new(self.b_max, self.b_n_lo)),
        }
    }

    /// The fuel the throttle schedules at march time `s` — rung 57's `_stator_march` ramp, spelled
    /// the same way, between the two endpoints' steady fuel flows.
    pub fn schedule(&self, mf_lo: f64, mf_hi: f64) -> impl Fn(f64) -> f64 {
        let r = self.ramp;
        move |s: f64| if s <= 0.0 { mf_lo } else if s >= r { mf_hi } else { mf_lo + (mf_hi - mf_lo) * (s / r) }
    }
}

/// The checks made before the engine is built — every one the model would otherwise ASSERT
/// (`r43_integrate_fuel`'s, the constructors'), in plain words.
pub fn controls_precheck(s: &ControlsSettings) -> Result<(), DoesNotRun> {
    let no = |m: &str| Err(DoesNotRun(m.to_string()));
    let named = [("rho", s.rho), ("from", s.from), ("to", s.to), ("ramp", s.ramp), ("settle", s.settle),
                 ("redline", s.redline), ("gov_lag", s.gov_lag), ("accel_margin", s.accel_margin),
                 ("floor_phi", s.floor_phi), ("tau_att", s.tau_att), ("tau_rel", s.tau_rel), ("v_max", s.v_max),
                 ("v_n_lo", s.v_n_lo), ("b_max", s.b_max), ("b_n_lo", s.b_n_lo)];
    if let Some((k, _)) = named.iter().find(|(_, v)| !v.is_finite()) {
        return Err(DoesNotRun(format!("Setting {k} is not a number.")));
    }
    if !(RHO_MIN..=RHO_MAX).contains(&s.rho) {
        return Err(DoesNotRun(format!(
            "The LP shaft's response time (ρ) must be between {RHO_MIN} and {RHO_MAX} times the HP shaft's: the \
             march's step was checked over that range (rungs 40–45 ran there too).")));
    }
    // Measured on THIS engine too (plan § 12.10): the bare slam's peak Tt4 moves 7.5 K with the step at a
    // ramp of 0.51, 0.0 K at 0.5.
    let steps = s.ramp / DS;
    if !(s.ramp >= DS - 1e-12 && (steps - steps.round()).abs() < 1e-9) {
        return Err(DoesNotRun(format!(
            "The ramp must last at least {DS} spool time constants and a whole number of {DS} steps: the run is \
             worked out in steps of {DS}, and a ramp that ends between two steps makes the peaks depend on the \
             step size.")));
    }
    if s.settle <= 0.0 {
        return no("The run must continue for some time after the move.");
    }
    if s.s_end() > MAX_RUN {
        return Err(DoesNotRun(format!("The whole run (ramp + settling) may be at most {MAX_RUN} spool time constants.")));
    }
    if s.gov_lag_on && !s.redline_on {
        return no("The temperature limiter's lag delays the limiter's cut, so it needs the limiter switched on.");
    }
    if s.gov_lag_on && s.gov_lag < DS - 1e-12 {
        return Err(DoesNotRun(format!(
            "The temperature limiter's lag must be at least one march step ({DS} spool time constants); switch the \
             lag off for an instant limiter.")));
    }
    if s.release_on && !(s.accel_on || s.floor_on) {
        return no("The realistic release lags the cut of the acceleration schedule or the stall floor, so switch \
                   one of them on.");
    }
    if s.release_on && s.gov_lag_on {
        return no("The realistic release and the temperature limiter's lag cannot run together: each carries a \
                   cut as a lagging state on a different limiter, and two such lags at once is a different model \
                   (rung 66's two-lag cascade), not this one.");
    }
    if s.release_on && !(s.tau_att >= DS - 1e-12 && s.tau_rel >= DS - 1e-12) {
        return Err(DoesNotRun(format!(
            "The realistic release's attack and release times must each be at least one march step ({DS} spool time \
             constants): measured, an attack of a quarter step moves the peak temperature by ~2.5 K with the step \
             size.")));
    }
    if s.accel_on && s.accel_margin < 0.0 {
        return no("The acceleration schedule's margin is measured above the steady line, so it cannot be negative.");
    }
    if s.floor_on && s.floor_phi <= 0.0 {
        return no("The stall floor is a flow coefficient, so it must be above zero.");
    }
    match s.lever {
        LeverChoice::None => {}
        LeverChoice::Stator => {
            if !(s.v_n_lo < 1.0) {
                return no("The stators must be back at their design setting by the design speed (1.0), so the \
                           speed they start opening from must be below 1.");
            }
        }
        LeverChoice::Bleed => {
            if !(s.b_n_lo < 1.0) {
                return no("The bleed valve must be shut by the design speed (1.0), so the speed it starts closing \
                           from must be below 1.");
            }
            if !(0.0..0.5).contains(&s.b_max) {
                return no("The bleed fraction must be at least 0 and below 0.5: bleeding half the air starves the \
                           core.");
            }
        }
    }
    Ok(())
}

/// Everything one Controls run needs, built once.
pub struct ControlsSolver {
    pub settings: ControlsSettings,
    pub core: ScheduledStatorCore,
    pub flight: FlightCondition,
    /// The steady points at the two throttles (rung 40's equilibrium on THIS machine — the lever
    /// armed, so the run starts on its own running line).
    pub start: Instant2,
    pub end: Instant2,
    /// Their steady fuel flows — rung 43's `fuel_for_tt4`, read off the same solves.
    pub mf_lo: f64,
    pub mf_hi: f64,
    /// Rung 48's table, derived on THIS machine's running line over the throttle band.
    pub accel: Option<AccelSchedule>,
}

/// Build the machine and solve both endpoints — or say why an endpoint has no steady point.
pub fn build(s: &ControlsSettings) -> Result<ControlsSolver, DoesNotRun> {
    controls_precheck(s)?;
    let (ml, mh) = control_maps(CONTROL_SHAPES[s.shape]).expect("CONTROL_SHAPES lists only buildable shapes");
    let fl = s.flight();
    let core = match build_scheduled_bleed(BladeSettings::defaults().design(), fl, MDOT_DESIGN, Some(ml), Some(mh),
                                           s.rho, &s.lever_arm()) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("the Controls engine has both shafts"),
    };
    let steady = |which: &str, tt4: f64| -> Result<Instant2, DoesNotRun> {
        core.fuel.inner.try_equilibrium(&fl, tt4, None).map(|(i, _, _)| i)
            .map_err(|e| DoesNotRun(format!("At the {which} throttle ({tt4:.0} K): {}", endpoint_words(&e.0))))
    };
    let start = steady("starting", s.from)?;
    let end = steady("final", s.to)?;
    let (mf_lo, mf_hi) = (start.close.f * start.close.mdot_air, end.close.f * end.close.mdot_air);
    let accel = match s.accel_on {
        false => None,
        true => {
            let (lo, hi) = if s.from <= s.to { (s.from, s.to) } else { (s.to, s.from) };
            Some(accel_table(&core, &fl, lo, hi, s.accel_margin)?)
        }
    };
    Ok(ControlsSolver { settings: *s, core, flight: fl, start, end, mf_lo, mf_hi, accel })
}

/// Plain words for an endpoint's failed steady solve — the four messages the crash map found
/// (plan § 12.10), each driven by a test.
pub fn endpoint_words(message: &str) -> String {
    let why = if message.contains("did not converge") {
        "the search for the steady shaft speeds does not settle. Measured only with a lever whose schedule does \
         nearly all its travel just below the design speed; start the lever's schedule lower"
    } else if message.contains("does not bracket") || message.contains("off-map compressor trial") {
        "the compressors would have to run off the part of their maps the model covers. This happens at a low \
         throttle (on the flat-LP map, below about 700–900 K) or with a lever whose schedule does nearly all its \
         travel just below the design speed. Raise the throttle, or start the lever's schedule lower"
    } else {
        "the model's steady solve fails"
    };
    format!("the engine has no steady operating point: {why}. (The model says: {message})")
}

impl ControlsSolver {
    /// The floor as the model's one min-select slot takes it.
    pub fn floor(&self) -> (Option<SurgeLimiter>, Option<IncidenceLimiter>) {
        let s = &self.settings;
        if !s.floor_on {
            return (None, None);
        }
        match s.floor_ref {
            FloorRef::Phi => (Some(SurgeLimiter::new(s.floor_spool, s.floor_phi)), None),
            FloorRef::Incidence => {
                let cmap = self.core.design_map(s.floor_spool);
                (None, Some(IncidenceLimiter::from_phi(&cmap, s.floor_spool, s.floor_phi, 0.0)))
            }
        }
    }

    pub fn limiters(&self) -> FuelLimiters<'_> {
        let s = &self.settings;
        let (surge, incidence) = self.floor();
        FuelLimiters {
            freeze: None,
            tt4_max: s.redline_on.then_some(s.redline),
            tau_gov: s.gov_lag_on.then_some(s.gov_lag),
            accel: self.accel.as_ref(),
            surge,
            incidence,
            s_off: None,
            tau_rel: None,
            lag: s.release_on.then(|| AsymmetricLag::new(s.tau_att, s.tau_rel)),
        }
    }

    /// The march — `integrate_fuel` itself, from the starting equilibrium.
    pub fn march(&self) -> Vec<FuelPoint> {
        let sched = self.settings.schedule(self.mf_lo, self.mf_hi);
        self.core.fuel.integrate_fuel(&self.flight, sched, (self.start.nu_lp, self.start.nu_hp),
                                      self.settings.s_end(), DS, &self.limiters())
    }

    /// Which march the model dispatches to (`r43_integrate_fuel`'s order).
    pub fn route(&self) -> Route {
        let s = &self.settings;
        if s.release_on { Route::Release } else if s.redline_on && s.gov_lag_on { Route::GovLag } else { Route::Plain }
    }

    /// The instant at a recorded point, re-read: the closure at the point's speeds and APPLIED fuel —
    /// the call the march's derivative made last there. `tests/sandbox_controls.rs` holds it to the
    /// recorded point bit for bit.
    pub fn reread(&self, p: &FuelPoint) -> Result<FuelInstant, Abort> {
        self.core.fuel.try_instant_fuel(&self.flight, p.nu_lp, p.nu_hp, p.mf)
    }
}

/// The marcher `r43_integrate_fuel` dispatches to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// Rungs 43–49's min-select march.
    Plain,
    /// Rung 47's lagged governor — its third state is not recorded, so a failed step cannot be re-run.
    GovLag,
    /// Rung 52's asymmetric lag — its third state IS recorded.
    Release,
}

/// Which call inside one derivative evaluation failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    /// The engine worked out at the SCHEDULED fuel — the plain march's first call, made to see
    /// whether any limiter is needed.
    AtSchedule,
    /// The engine worked out at the fuel actually applied (after the limiters, or the lag's cut).
    AtApplied,
    /// A limiter's own set-point solve: the temperature limiter, the acceleration schedule, the floor.
    Redline,
    Accel,
    Floor,
}

/// Where a re-run evaluation failed: the model's message, the call, the state and the fuel it was
/// evaluated at.
#[derive(Clone, Debug, PartialEq)]
pub struct Failure {
    pub message: String,
    pub call: Call,
    /// The fuel flow handed to the failing call, kg/s (a limiter's solve: the scheduled fuel it starts from).
    pub mf: f64,
    pub nu_lp: f64,
    pub nu_hp: f64,
    pub s: f64,
}

/// One derivative evaluation of the PLAIN march — `r43_integrate_fuel`'s `der`, copied in its order:
/// the bare instant, then each armed leg off the SCHEDULED fuel (the redline only when the bare
/// instant is over it), the min of the caps below the scheduled fuel, the instant re-made there.
/// Returns `(dν_L/ds, dν_H/ds)`.
fn der_plain(sv: &ControlsSolver, a: f64, b: f64, mf_in: f64, s: f64) -> Result<(f64, f64), Failure> {
    let fail = |call: Call, mf: f64| move |e: Abort| Failure { message: e.0, call, mf, nu_lp: a, nu_hp: b, s };
    let ft = &sv.core.fuel;
    let fl = &sv.flight;
    let lim = sv.limiters();
    let floor = lim.floor();
    let mut mf = mf_in;
    let mut i = ft.try_instant_fuel(fl, a, b, mf).map_err(fail(Call::AtSchedule, mf))?;
    let mut caps: Vec<f64> = Vec::new();
    let w = release_weight(s, lim.s_off, lim.tau_rel);
    let faded = |c: f64| if w >= 1.0 { c } else { mf + w * (c - mf) };
    if let Some(tt4_max) = lim.tt4_max {
        if i.base.tt4 > tt4_max {
            caps.push(ft.try_topping_fuel(fl, a, b, tt4_max, mf).map_err(fail(Call::Redline, mf))?);
        }
    }
    if let Some(accel) = lim.accel {
        if w > 0.0 {
            caps.push(faded(ft.try_sched_fuel(fl, a, b, mf, accel).map_err(fail(Call::Accel, mf))?));
        }
    }
    if let Some(surge) = floor.as_ref() {
        if w > 0.0 {
            caps.push(faded(ft.try_surge_fuel(fl, a, b, mf, surge).map_err(fail(Call::Floor, mf))?));
        }
    }
    caps.retain(|&c| c < mf);
    if !caps.is_empty() {
        let mut m = caps[0];
        for &c in &caps[1..] {
            if c < m {
                m = c;
            }
        }
        mf = m;
        i = ft.try_instant_fuel(fl, a, b, mf).map_err(fail(Call::AtApplied, mf))?;
    }
    Ok((i.base.phi_lp_dot / ft.rho(), i.base.phi_hp_dot))
}

/// One derivative evaluation of rung 52's lagged march — `integrate_fuel_asym`'s `der`, copied in its
/// order. Returns `(dν_L/ds, dν_H/ds, dg/ds)`.
fn der_release(sv: &ControlsSolver, a: f64, b: f64, g: f64, s: f64) -> Result<(f64, f64, f64), Failure> {
    let fail = |call: Call, mf: f64| move |e: Abort| Failure { message: e.0, call, mf, nu_lp: a, nu_hp: b, s };
    let ft = &sv.core.fuel;
    let fl = &sv.flight;
    let lim = sv.limiters();
    let floor = lim.floor();
    let lag = lim.lag.expect("the release route has its lag");
    let mf_sched = sv.settings.schedule(sv.mf_lo, sv.mf_hi)(s);
    let mut mf = mf_sched - g;
    if 1e-9 > mf {
        mf = 1e-9;
    }
    if let Some(tt4_max) = lim.tt4_max {
        if ft.try_instant_fuel(fl, a, b, mf).map_err(fail(Call::AtApplied, mf))?.base.tt4 > tt4_max {
            let c = ft.try_topping_fuel(fl, a, b, tt4_max, mf).map_err(fail(Call::Redline, mf))?;
            if c < mf {
                mf = c;
            }
        }
    }
    let i = ft.try_instant_fuel(fl, a, b, mf).map_err(fail(Call::AtApplied, mf))?;
    let mut caps: Vec<f64> = Vec::new();
    if let Some(accel) = lim.accel {
        caps.push(ft.try_sched_fuel(fl, a, b, mf_sched, accel).map_err(fail(Call::Accel, mf_sched))?);
    }
    if let Some(surge) = floor.as_ref() {
        caps.push(ft.try_surge_fuel(fl, a, b, mf_sched, surge).map_err(fail(Call::Floor, mf_sched))?);
    }
    let req = if caps.is_empty() {
        0.0
    } else {
        let mut m = caps[0];
        for &c in &caps[1..] {
            if c < m {
                m = c;
            }
        }
        0.0f64.max(mf_sched - m)
    };
    let dg = (req - g) / lag.tau(req, g);
    Ok((i.base.phi_lp_dot / ft.rho(), i.base.phi_hp_dot, dg))
}

/// One RK4 step from a recorded point, re-run: `Ok((ν_L, ν_H, g))` after the step, or the first
/// failure met — the marcher's order (`k1`, `k2`/`k3` at the half step, `k4` at the full one) and its
/// arithmetic (`a + ds/2*k`, then `a += ds/6*(k1 + 2k2 + 2k3 + k4)`). `None` on the governor-lag route.
pub fn replay_step(sv: &ControlsSolver, p: &FuelPoint) -> Option<Result<(f64, f64, f64), Failure>> {
    let sched = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
    let (a, b, s) = (p.nu_lp, p.nu_hp, p.s);
    let ds = DS;
    Some(match sv.route() {
        Route::GovLag => return None,
        Route::Plain => (|| {
            let (k1a, k1b) = der_plain(sv, a, b, sched(s), s)?;
            let mfm = sched(s + ds / 2.0);
            let (k2a, k2b) = der_plain(sv, a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, mfm, s + ds / 2.0)?;
            let (k3a, k3b) = der_plain(sv, a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, mfm, s + ds / 2.0)?;
            let (k4a, k4b) = der_plain(sv, a + ds * k3a, b + ds * k3b, sched(s + ds), s + ds)?;
            Ok((a + ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a),
                b + ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b), 0.0))
        })(),
        Route::Release => (|| {
            let PointExtra::Asym { g, .. } = p.extra else {
                unreachable!("the release route records its lag state");
            };
            let (k1a, k1b, k1g) = der_release(sv, a, b, g, s)?;
            let (k2a, k2b, k2g) = der_release(sv, a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, g + ds / 2.0 * k1g, s + ds / 2.0)?;
            let (k3a, k3b, k3g) = der_release(sv, a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, g + ds / 2.0 * k2g, s + ds / 2.0)?;
            let (k4a, k4b, k4g) = der_release(sv, a + ds * k3a, b + ds * k3b, g + ds * k3g, s + ds)?;
            Ok((a + ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a),
                b + ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b),
                g + ds / 6.0 * (k1g + 2.0 * k2g + 2.0 * k3g + k4g)))
        })(),
    })
}

/// Why and where a march stopped early. `failure` is `None` on the governor-lag route, whose failed
/// step cannot be re-run.
#[derive(Clone, Debug, PartialEq)]
pub struct Stop {
    pub s_last: f64,
    pub failure: Option<Failure>,
}

/// Why a march that stopped early stopped: re-run the step from its last recorded point, then the
/// next step's opening evaluation (`k1` at a state the march never records). `None` if it ran to the end.
pub fn stop_reason(sv: &ControlsSolver, points: &[FuelPoint]) -> Option<Stop> {
    if points.len() >= sv.settings.expected_points() {
        return None;
    }
    let Some(last) = points.last() else {
        // The very first evaluation failed: re-make it.
        let sched = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
        let (a, b) = (sv.start.nu_lp, sv.start.nu_hp);
        let failure = match sv.route() {
            Route::GovLag => None,
            Route::Plain => der_plain(sv, a, b, sched(0.0), 0.0).err(),
            Route::Release => der_release(sv, a, b, 0.0, 0.0).err(),
        };
        return Some(Stop { s_last: f64::NAN, failure });
    };
    let failure = match replay_step(sv, last) {
        None => None,
        Some(Err(f)) => Some(f),
        Some(Ok((a, b, g))) => {
            let s = last.s + DS;
            let sched = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
            match sv.route() {
                Route::Release => der_release(sv, a, b, g, s).err(),
                _ => der_plain(sv, a, b, sched(s), s).err(),
            }.or_else(|| Some(Failure { message: "the march stopped, but re-running its last step does not fail".into(),
                                        call: Call::AtApplied, mf: f64::NAN, nu_lp: f64::NAN, nu_hp: f64::NAN,
                                        s: f64::NAN }))
        }
    };
    Some(Stop { s_last: last.s, failure })
}

/// The geometric middle of rung 43's two fuel-air walls (`F_FLOOR` 0.004, `F_CAP` 0.065): a failed
/// closure whose trial mixture is under this is on the LEAN side, over it on the RICH side. The
/// crash map's lean stops sat at 0.0024–0.0041 and its rich ones at 0.055–0.070 (plan § 12.10), each
/// within a factor of two of its own wall; `tests/sandbox_controls.rs` holds that.
pub fn lean_rich_split() -> f64 { (FuelTransientCore::F_FLOOR * FuelTransientCore::F_CAP).sqrt() }

/// What stopped a march, read off the re-run failure (plan § 12.10 — each kind measured in the
/// crash map and driven by a test).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopKind {
    /// Rung 49's own refusal: no fuel cut restores the flow coefficient to the floor.
    FloorUnreachable,
    /// A limiter held the fuel below the schedule, and the march's check AT THE SCHEDULED FUEL (made
    /// every step, to see whether a limiter is needed) has no solution at these shaft speeds, while the
    /// cut fuel does ([`classify`] asks the model) — the method's limit, not the engine's.
    ScheduleCheck,
    /// The run's own fuel left the fuel-metered solver's range on the lean side (a fast power cut).
    Lean,
    /// ...on the rich side (a fast slam: the fuel outran the air).
    Rich,
    /// The temperature limiter's lag is on: that march does not record its lag state, so the failed
    /// step cannot be re-run and the cause is not known.
    Unknown,
    /// Anything else the model refused at this point of the run.
    Other,
}

impl StopKind {
    pub fn key(self) -> &'static str {
        match self {
            StopKind::FloorUnreachable => "floor_unreachable",
            StopKind::ScheduleCheck => "schedule_check",
            StopKind::Lean => "lean",
            StopKind::Rich => "rich",
            StopKind::Unknown => "unknown",
            StopKind::Other => "other",
        }
    }

    /// What the page says when a run stops this way, above the model's own message.
    pub fn words(self) -> &'static str {
        match self {
            StopKind::FloorUnreachable =>
                "The stall floor could no longer be held. The floor cuts the fuel to keep the compressor's flow \
                 coefficient up; here even cutting the fuel to almost nothing would not lift it to the floor, so the \
                 model stops. This happens when the floor sits above where the compressor runs: the floor starves \
                 the engine, the shafts slow, and the flow coefficient falls further. Lower the floor (with the \
                 stator schedule on, try the floor that watches blade incidence).",
            StopKind::ScheduleCheck =>
                "This stop is the model's, not the engine's. A limiter is holding the fuel below the schedule. At \
                 every step the model first works the engine out at the FULL scheduled fuel, to see whether a \
                 limiter is needed at all; at these shaft speeds it finds no operating point for that much fuel, \
                 so the run stops, although the engine does solve at the fuel actually burning (checked at the \
                 failing state). Shorten the gap: move the throttle less, or loosen the limiter that is holding.",
            StopKind::Lean =>
                "The fuel fell faster than the air: on this power cut the mixture got leaner than 0.004 kg of fuel \
                 per kg of air, the leanest the model's fuel-metered solver searches, so the run stops. (A real \
                 burner has a lean blow-out limit too; this one is the solver's search bound, not a combustion \
                 model.) Cut the throttle more slowly, or not so far.",
            StopKind::Rich =>
                "The fuel arrived faster than the shafts could bring in air to burn it: the mixture would need more \
                 than 0.065 kg of fuel per kg of air, the richest the model's fuel-metered solver searches (burning \
                 every bit of the oxygen takes about 0.068). A real engine's fuel controls exist to prevent exactly \
                 this — try the acceleration schedule or the temperature limiter, or a slower move.",
            StopKind::Unknown =>
                "With the temperature limiter's lag switched on, the model's march does not record the lag's own \
                 state, so the sandbox cannot re-run the failed step to say why the run stopped here. The points \
                 before the stop are the model's own.",
            StopKind::Other =>
                "The model stopped on one of its internal checks at this point of the run.",
        }
    }
}

/// The mixture the failing call was asked for, estimated with the last recorded point's LP-face air
/// flow (`mf / f` there): the fuel handed to the failing call over that flow.
pub fn trial_mixture(f: &Failure, last: Option<&FuelPoint>) -> Option<f64> {
    let l = last?;
    (f.mf.is_finite() && l.mf > 0.0).then(|| f.mf * l.f / l.mf)
}

/// The fuel actually burning at a failing state, estimated: the scheduled fuel the failing call was
/// handed, cut by the fraction the limiters held at the last recorded point. `None` unless a limiter was
/// cutting there and the failing call was one made at the scheduled fuel.
pub fn cut_fuel(f: &Failure, last: Option<&FuelPoint>) -> Option<f64> {
    let l = last?;
    (l.mf < l.mf_sched && f.call != Call::AtApplied).then(|| f.mf * (l.mf / l.mf_sched))
}

/// Classify a re-run failure by its message, by WHICH call failed, and — for the every-step check —
/// by asking the model itself: a stop is the METHOD's only if the engine DOES solve at the cut fuel at
/// the failing state while it fails at the scheduled one (plan § 12.10: 89 of 90 such stops in the
/// slider box; the one that did not, a 4 % cut on a fast slam, is a rich stop and is classed so).
pub fn classify(sv: &ControlsSolver, stop: &Stop, last: Option<&FuelPoint>) -> StopKind {
    let Some(f) = &stop.failure else { return StopKind::Unknown };
    let m = &f.message;
    if m.contains("UNREACHABLE") {
        return StopKind::FloorUnreachable;
    }
    if !m.contains("fuel closure does not bracket") {
        return StopKind::Other;
    }
    if let Some(w) = cut_fuel(f, last) {
        if sv.core.fuel.try_instant_fuel(&sv.flight, f.nu_lp, f.nu_hp, w).is_ok() {
            return StopKind::ScheduleCheck;
        }
    }
    match trial_mixture(f, last) {
        Some(x) if x < lean_rich_split() => StopKind::Lean,
        Some(_) => StopKind::Rich,
        None => StopKind::Other,
    }
}

/// One Controls run.
pub struct ControlsOutcome {
    pub settings: ControlsSettings,
    pub points: Vec<FuelPoint>,
    pub stop: Option<Stop>,
}

/// Rung 48's table, derived on this machine's running line — `FuelTransientCore::accel_schedule`'s
/// rows, made through the FALLIBLE equilibrium so a band with no steady point is refused in words
/// instead of crashing (the crash map's four panics, plan § 12.10). The arithmetic is the model's,
/// line for line; `tests/sandbox_controls.rs` holds the table to `accel_schedule` bit for bit.
pub fn accel_table(core: &ScheduledStatorCore, fl: &FlightCondition, tt4_lo: f64, tt4_hi: f64, margin: f64)
    -> Result<AccelSchedule, DoesNotRun> {
    let n = ACCEL_ROWS;
    let ft = &core.fuel;
    let mut rows: Vec<(f64, f64)> = Vec::with_capacity(n);
    for k in 0..n {
        let tt4 = tt4_lo + (tt4_hi - tt4_lo) * k as f64 / (n as f64 - 1.0);
        let eq = ft.inner.try_equilibrium(fl, tt4, None).map_err(|e| DoesNotRun(format!(
            "The acceleration schedule is read off this engine's own steady running line between the two \
             throttles, and at {tt4:.0} K that line has no steady point ({}). Narrow the throttle range, start \
             the lever's schedule lower, or switch the schedule off.", e.0)))?.0;
        let pt3 = eq.close.pt4 / ft.inner.inner.base.pi_b;
        rows.push((eq.close.n_hp, eq.close.f * eq.close.mdot_air / pt3));
    }
    rows.sort_by(|a, b| a.partial_cmp(b).expect("running-line rows are finite"));
    Ok(AccelSchedule { margin, n_h: rows.iter().map(|&(a, _)| a).collect(), kappa: rows.iter().map(|&(_, b)| b).collect() })
}

/// Run one Controls request.
pub fn controls(s: &ControlsSettings) -> Result<(ControlsSolver, ControlsOutcome), DoesNotRun> {
    let sv = build(s)?;
    let points = sv.march();
    let stop = stop_reason(&sv, &points);
    Ok((sv, ControlsOutcome { settings: *s, points, stop }))
}

/// Which limiter holds the fuel at a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holder {
    /// The applied fuel IS the scheduled fuel.
    None,
    Redline,
    Accel,
    Floor,
    /// A lag's cut (rung 47's or rung 52's) — between its leg's set point and the schedule.
    Lag,
}

impl Holder {
    pub fn key(self) -> &'static str {
        match self {
            Holder::None => "none",
            Holder::Redline => "redline",
            Holder::Accel => "accel",
            Holder::Floor => "floor",
            Holder::Lag => "lag",
        }
    }
}

/// How close a limiter's own equation must hold at the applied state for it to be the one holding
/// the fuel — relative. Measured over 3 000 requests in the slider box (plan § 12.10), counting only
/// the limiters that act WITHOUT a lag on their route: a holding leg's residual is at most 4.8e-15, a
/// non-holding armed leg's at least 3.2e-6. The bar sits ~4 decades from each.
pub const HOLD_TOL: f64 = 1e-10;

/// Everything the page draws at one recorded point, beyond the fourteen fields the march records:
/// re-read through the closure the march's derivative made last there ([`ControlsSolver::reread`]).
#[derive(Clone, Copy, Debug)]
pub struct PointRead {
    /// Thrust, N per kg/s of design airflow — with the bleed open, the honest inlet figure (rung 42's
    /// booking: the dumped air pays full ram drag and returns nothing).
    pub thrust: f64,
    /// The stall line on each compressor's map at this point's stator setting (rung 53: closing the
    /// stators moves the wall).
    pub stall_lp: f64,
    pub stall_hp: f64,
    /// The stator setting on the lever's spool, and the bleed fraction.
    pub v: f64,
    pub bleed: f64,
    /// The floor's flow coefficient at this point (an incidence floor moves with the stators).
    pub floor: Option<f64>,
    pub holder: Holder,
    /// Each armed limiter's relative residual at the applied state (`None` = not armed).
    pub res_redline: Option<f64>,
    pub res_accel: Option<f64>,
    pub res_floor: Option<f64>,
}

impl ControlsSolver {
    /// The thrust of an instant, by the rule [`PointRead::thrust`] states.
    pub fn thrust(i: &Instant2) -> f64 {
        match (i.close.bleed, i.sp_thrust_inlet, i.close.mdot_face) {
            (Some(_), Some(spi), Some(face)) => spi * face,
            _ => i.sp_thrust * i.close.mdot_air,
        }
    }

    /// The stator setting on a spool at a state, and that compressor's stall line there.
    pub fn stall(&self, spool: Spool, nu_lp: f64, nu_hp: f64) -> (f64, f64) {
        let v = self.core.v_of(spool, nu_lp, nu_hp, None);
        (v, self.core.design_map(spool).with_vsv(v).phi_surge_at())
    }

    /// Read one recorded point.
    pub fn read(&self, p: &FuelPoint) -> Result<PointRead, Abort> {
        let s = &self.settings;
        let inst = self.reread(p)?;
        let i = &inst.base;
        let (v_lp, stall_lp) = self.stall(Spool::Lp, p.nu_lp, p.nu_hp);
        let (v_hp, stall_hp) = self.stall(Spool::Hp, p.nu_lp, p.nu_hp);
        let floor = match self.limiters().floor() {
            None => None,
            Some(fl) => Some(self.core.fuel.resolve_floor(&fl, p.nu_lp, p.nu_hp)?.phi_lim),
        };
        let res_redline = s.redline_on.then(|| (i.tt4 - s.redline).abs() / s.redline);
        let res_accel = self.accel.as_ref().map(|a| {
            let cap = a.cap(i.close.n_hp, i.close.pt4 / self.core.fuel.inner.inner.base.pi_b);
            (p.mf - cap).abs() / p.mf
        });
        let res_floor = floor.map(|lim| {
            let phi = match s.floor_spool { Spool::Lp => i.close.phi_lp, Spool::Hp => i.close.phi_hp };
            (phi - lim).abs() / lim
        });
        // Only a limiter that acts WITHOUT a lag on this route can meet its own equation exactly; a
        // lagged cut converges onto its set point and is reported as the lag's.
        let unlagged: &[Holder] = match self.route() {
            Route::Plain => &[Holder::Redline, Holder::Accel, Holder::Floor],
            Route::GovLag => &[Holder::Accel, Holder::Floor],
            Route::Release => &[Holder::Redline],
        };
        let holder = if p.mf == p.mf_sched {
            Holder::None
        } else {
            let mut best: Option<(f64, Holder)> = None;
            for (r, h) in [(res_redline, Holder::Redline), (res_accel, Holder::Accel), (res_floor, Holder::Floor)] {
                if let Some(r) = r {
                    if unlagged.contains(&h) && r < HOLD_TOL && best.is_none_or(|(b, _)| r < b) {
                        best = Some((r, h));
                    }
                }
            }
            best.map(|(_, h)| h).unwrap_or(Holder::Lag)
        };
        Ok(PointRead {
            thrust: Self::thrust(i), stall_lp, stall_hp,
            v: match s.stator_spool { Spool::Lp => v_lp, Spool::Hp => v_hp },
            bleed: i.close.bleed.unwrap_or(0.0), floor, holder, res_redline, res_accel, res_floor,
        })
    }
}

fn num_or_null(x: f64) -> Json { if x.is_finite() { Json::Float(x) } else { Json::Null } }

/// One steady endpoint, for the page.
fn steady_json(sv: &ControlsSolver, i: &Instant2) -> Json {
    let (_, stall_lp) = sv.stall(Spool::Lp, i.nu_lp, i.nu_hp);
    let (_, stall_hp) = sv.stall(Spool::Hp, i.nu_lp, i.nu_hp);
    jobj! {
        "Tt4" => i.tt4, "nu_lp" => i.nu_lp, "nu_hp" => i.nu_hp, "fuel" => i.close.f * i.close.mdot_air,
        "far" => i.close.f, "thrust" => ControlsSolver::thrust(i), "phi_lp" => i.close.phi_lp, "phi_hp" => i.close.phi_hp,
        "pi_lpc" => i.close.pi_lpc, "pi_hpc" => i.close.pi_hpc, "stall_lp" => stall_lp, "stall_hp" => stall_hp,
        "choked" => Json::Int((i.branch == Branch::Choked) as i64),
    }
}

/// The page's view of a Controls run, FULL precision; the trajectory leaves as COLUMNS.
pub fn controls_json(sv: &ControlsSolver, o: &ControlsOutcome) -> Json {
    let p = &o.points;
    let reads: Vec<Option<PointRead>> = p.iter().map(|x| sv.read(x).ok()).collect();
    let col = |f: &dyn Fn(&FuelPoint) -> f64| Json::List(p.iter().map(|x| Json::Float(f(x))).collect());
    let rcol = |f: &dyn Fn(&PointRead) -> f64| Json::List(reads.iter().map(|r| r.as_ref().map(|r| num_or_null(f(r))).unwrap_or(Json::Null)).collect());
    let last = p.last();
    let (settled_lp, settled_hp) = match last {
        Some(l) => ((l.nu_lp - sv.end.nu_lp).abs(), (l.nu_hp - sv.end.nu_hp).abs()),
        None => (f64::NAN, f64::NAN),
    };
    jobj! {
        "ok" => Json::Int(1),
        "controls" => o.settings.to_json(),
        "start" => steady_json(sv, &sv.start), "end" => steady_json(sv, &sv.end),
        "expected_points" => Json::Int(o.settings.expected_points() as i64),
        "route" => match sv.route() { Route::Plain => "plain", Route::GovLag => "gov_lag", Route::Release => "release" },
        "settled_lp" => num_or_null(settled_lp), "settled_hp" => num_or_null(settled_hp),
        "stopped" => Json::Int(o.stop.is_some() as i64),
        "stop" => match &o.stop {
            None => Json::Null,
            Some(st) => {
                let kind = classify(sv, st, last);
                let f = st.failure.as_ref();
                jobj! {
                    "kind" => kind.key(), "words" => kind.words(),
                    "message" => f.map(|f| Json::Str(f.message.clone())).unwrap_or(Json::Null),
                    "s_last" => num_or_null(st.s_last),
                    "s" => f.map(|f| num_or_null(f.s)).unwrap_or(Json::Null),
                    "mixture" => f.and_then(|f| trial_mixture(f, last)).map(num_or_null).unwrap_or(Json::Null),
                    "min_fuel_fraction" => num_or_null(p.iter().map(|x| x.mf / x.mf_sched).fold(f64::INFINITY, f64::min)),
                }
            }
        },
        "s" => col(&|x| x.s), "nu_lp" => col(&|x| x.nu_lp), "nu_hp" => col(&|x| x.nu_hp), "Tt4" => col(&|x| x.tt4),
        "fuel" => col(&|x| x.mf), "fuel_sched" => col(&|x| x.mf_sched), "far" => col(&|x| x.f),
        "phi_lp" => col(&|x| x.phi_lp), "phi_hp" => col(&|x| x.phi_hp),
        "pi_lpc" => col(&|x| x.pi_lpc), "pi_hpc" => col(&|x| x.pi_hpc),
        "thrust" => rcol(&|r| r.thrust), "stall_lp" => rcol(&|r| r.stall_lp), "stall_hp" => rcol(&|r| r.stall_hp),
        "v" => rcol(&|r| r.v), "bleed" => rcol(&|r| r.bleed), "floor" => rcol(&|r| r.floor.unwrap_or(f64::NAN)),
        "holder" => Json::List(reads.iter().map(|r| Json::from(r.as_ref().map(|r| r.holder.key()).unwrap_or("none"))).collect()),
        "choked" => Json::List(p.iter().map(|x| Json::Int((x.branch == Branch::Choked) as i64)).collect()),
    }
}

/// Plain words for a model CRASH in this view (`explain` with `"view":"controls"`). The crash map's
/// crashes (the acceleration schedule's steady rows) are now refused in words before the run
/// ([`accel_table`]); none was left in 3 000 requests over the slider box, so this is the fallback.
pub fn explain_controls(message: &str) -> String {
    format!("The model failed in a solve this view does not guard (no such case was found in the 3 000 requests the \
             crash map ran over these sliders). The model says: {message}")
}

/// The Controls requests the browser check (`rust/sandbox-wasm/check.mjs`) compares with the native
/// model: the page's opening run, each switch where it binds (plan § 12.10's default table), the two
/// levers, a chop, a refused combination, and one stop of each kind the crash map found.
pub fn check_requests() -> Vec<String> {
    let d = ControlsSettings::defaults();
    let flat = CONTROL_SHAPES.iter().position(|&k| k == "flat-lp").expect("flat-lp is a shape");
    let req = |s: ControlsSettings| jobj! { "op" => "controls", "controls" => s.to_json() }.dump_compact();
    let mut out = vec![r#"{"op":"controls_defaults"}"#.to_string(), r#"{"op":"controls","controls":{}}"#.to_string()];
    for s in [
        ControlsSettings { redline_on: true, ..d },
        ControlsSettings { redline_on: true, gov_lag_on: true, ..d },
        ControlsSettings { accel_on: true, ..d },
        ControlsSettings { floor_on: true, ..d },
        ControlsSettings { floor_on: true, release_on: true, ..d },
        ControlsSettings { redline_on: true, accel_on: true, floor_on: true, floor_phi: 0.69, lever: LeverChoice::Stator, ..d },
        ControlsSettings { shape: flat, floor_on: true, floor_ref: FloorRef::Incidence, lever: LeverChoice::Stator, ..d },
        ControlsSettings { lever: LeverChoice::Stator, stator_spool: Spool::Hp, ..d },
        ControlsSettings { lever: LeverChoice::Bleed, redline_on: true, rho: 3.0, ..d },
        ControlsSettings { from: 1400.0, to: 1000.0, ..d },
        ControlsSettings { release_on: true, ..d },                                  // refused: nothing to lag
        ControlsSettings { floor_on: true, lever: LeverChoice::Stator, ..d },        // stop: the floor unreachable
        ControlsSettings { shape: 2, from: 1485.0, to: 614.0, ramp: 0.66, settle: 1.0, ..d },   // stop: lean
    ] {
        out.push(req(s));
    }
    out
}

/// `{"op":"controls_defaults"}` and `{"op":"controls","controls":{…}}` — the dispatcher
/// `sandbox::call` falls to.
pub fn call_op(op: &str, req: &Json) -> Option<Json> {
    let refusal = |m: &str| jobj! { "ok" => Json::Int(0), "reason" => m };
    Some(match op {
        "controls_defaults" => jobj! {
            "controls" => ControlsSettings::defaults().to_json(), "ds" => DS, "max_run" => MAX_RUN,
            "shapes" => Json::List(CONTROL_SHAPES.iter().map(|&s| Json::from(s)).collect()),
        },
        "controls" => {
            let empty = Json::Obj(Vec::new());
            match ControlsSettings::from_json(req.get("controls").unwrap_or(&empty)) {
                Err(e) => refusal(&e),
                Ok(s) => controls(&s).map(|(sv, o)| controls_json(&sv, &o)).unwrap_or_else(|DoesNotRun(m)| refusal(&m)),
            }
        }
        _ => return None,
    })
}
