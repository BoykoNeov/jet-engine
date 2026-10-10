//! The web sandbox, slice 4 (A) — **SLAM THE THROTTLE** (`docs/plans/sandbox-plan.md` § 12). NOT a rung.
//!
//! Slices 1–3 show STEADY points: the engine has already settled. Here the throttle moves and the
//! engine takes its time — the shaft speed is a STATE that lags the fuel (rung 34) — on the user's own
//! *Fly it* engine: its frozen hardware, flight, map shape and stall line ([`crate::sandbox::fly_solver`]).
//!
//! **Two ways to move the throttle, side by side, because the difference IS rung 35's finding**
//! (user decision, plan § 12.8 Q1):
//! - *temperature commanded* (rung 34, `SpoolTransient::integrate`): the turbine-inlet temperature
//!   follows the ramp by fiat — the textbook idealisation;
//! - *fuel metered* (rung 35, `SpoolTransient::integrate_fuel`): the FUEL follows the ramp, between the
//!   two endpoints' steady fuel flows, and the temperature is an OUTPUT. On a slam the spool cannot
//!   pump the extra air yet, so the temperature OVERSHOOTS.
//!
//! **Time is in spool time constants** (`τ_spool = I·ω_d²/P_ref`, rung 34) — the model has no rotor
//! inertia, and rung 34's finding is the RATIO of the ramp to that time (plan § 12.8 Q4). The march
//! step is the rungs' own, [`DS`].
//!
//! **A run that stops early is a RESULT.** Rung 34's marcher `break`s when an evaluation leaves the
//! region the model covers, and drops the error; the page needs the reason. [`stop_reason`] replays the
//! WHOLE failed step — `k1` to `k4`, then the new state's `k1` — through the PUBLIC instants, in the
//! marcher's own order and arithmetic (`spool.rs` `march`), so the first error it meets is the one the
//! march met (`tests/sandbox_transient.rs` holds the replay to the march's own trajectory, bit for bit).
//!
//! **The equilibrium gas is refused** (plan § 12.4): fuel metering does not exist on it (the forward
//! burner asserts — rung 35's open seam), and a temperature-commanded march on it takes ~9 s natively.

use crate::components::ram_recovery;
use crate::gas::powp;
use crate::jobj;
use crate::matcher::Branch;
use crate::sandbox::{fly_precheck, fly_solver, DoesNotRun, FlySettings, GasModel};
use crate::spool::{Instant, SpoolTransient, TransientPoint};
use crate::visuals::Json;

/// The march step, in spool time constants — rung 34's and rung 35's own grid.
pub const DS: f64 = 0.02;

/// How the throttle is moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThrottleMode {
    /// Rung 34: the turbine-inlet temperature follows the ramp.
    Temperature,
    /// Rung 35: the fuel flow follows the ramp; the temperature is an output.
    Fuel,
}

impl ThrottleMode {
    pub const ALL: [ThrottleMode; 2] = [ThrottleMode::Temperature, ThrottleMode::Fuel];

    pub fn key(self) -> &'static str {
        match self { ThrottleMode::Temperature => "temperature", ThrottleMode::Fuel => "fuel" }
    }

    fn from_key(k: &str) -> Result<Self, String> {
        ThrottleMode::ALL.into_iter().find(|m| m.key() == k)
            .ok_or_else(|| format!("unknown throttle mode {k:?} (\"temperature\" or \"fuel\")"))
    }
}

/// Every knob of the slam: the *Fly it* view's (hardware, gas, flight, map, stall line) plus the move.
/// The fly settings' own throttle is not read — the slam has two.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlamSettings {
    pub fly: FlySettings,
    /// The throttle before and after the move: turbine-inlet temperature, K.
    pub from: f64,
    pub to: f64,
    /// How long the move takes, in spool time constants (0 = a step).
    pub ramp: f64,
    /// How long the run continues AFTER the move, in spool time constants.
    pub settle: f64,
    pub mode: ThrottleMode,
}

impl SlamSettings {
    /// A slam from 400 K below the design throttle up to it, over half a spool time — rung 35's own
    /// `r` = 0.5 — then three spool times to settle.
    pub fn defaults() -> Self {
        let fly = FlySettings::defaults();
        SlamSettings { fly, from: fly.design.tt4 - 400.0, to: fly.design.tt4, ramp: 0.5, settle: 3.0,
                       mode: ThrottleMode::Temperature }
    }

    /// The fly settings at one endpoint's throttle — what the steady solves and slice 3's pre-checks read.
    pub fn at(&self, tt4: f64) -> FlySettings { FlySettings { tt4, ..self.fly } }

    pub fn to_json(&self) -> Json {
        jobj! {
            "fly" => self.fly.to_json(), "from" => self.from, "to" => self.to, "ramp" => self.ramp,
            "settle" => self.settle, "mode" => self.mode.key(),
        }
    }

    /// Read slam settings; a missing key keeps its default (a missing `fly` is the opening one).
    pub fn from_json(j: &Json) -> Result<Self, String> {
        let mut s = SlamSettings::defaults();
        if let Some(f) = j.get("fly") {
            s.fly = FlySettings::from_json(f)?;
            s.from = s.fly.design.tt4 - 400.0;
            s.to = s.fly.design.tt4;
        }
        let num = |k: &str, into: &mut f64| -> Result<(), String> {
            match j.get(k) {
                None => Ok(()),
                Some(Json::Float(x)) => { *into = *x; Ok(()) }
                Some(Json::Int(n)) => { *into = *n as f64; Ok(()) }
                Some(other) => Err(format!("setting {k:?} must be a number, got {other:?}")),
            }
        };
        num("from", &mut s.from)?;
        num("to", &mut s.to)?;
        num("ramp", &mut s.ramp)?;
        num("settle", &mut s.settle)?;
        match j.get("mode") {
            None => {}
            Some(Json::Str(m)) => s.mode = ThrottleMode::from_key(m)?,
            Some(o) => return Err(format!("setting \"mode\" must be text, got {o:?}")),
        }
        Ok(s)
    }

    /// The march's end, in spool time constants.
    pub fn s_end(&self) -> f64 { self.ramp + self.settle }

    /// How many points a march that runs to the end records — rung 34's own step count.
    pub fn expected_points(&self) -> usize { (self.s_end() / DS).round_ties_even() as usize + 1 }
}

/// The longest run the page asks for, in spool time constants — a bound on the cost of one request.
pub const MAX_RUN: f64 = 20.0;

/// The checks made before the slam: slice 3's fly pre-checks at BOTH throttles, then the slam's own.
pub fn slam_precheck(s: &SlamSettings) -> Result<(), DoesNotRun> {
    let no = |m: &str| Err(DoesNotRun(m.to_string()));
    if s.fly.gas == GasModel::Equilibrium {
        return no("The throttle slam does not run on the equilibrium gas. Metering fuel needs the burner \
                   worked forwards (fuel in, temperature out), which the model does not have for that gas, so \
                   the two ways of moving the throttle cannot be compared; and one run would take about \
                   20 seconds. Pick another gas — the reacting gas is the closest.");
    }
    let named = [("from", s.from), ("to", s.to), ("ramp", s.ramp), ("settle", s.settle)];
    if let Some((k, _)) = named.iter().find(|(_, v)| !v.is_finite()) {
        return Err(DoesNotRun(format!("Setting {k} is not a number.")));
    }
    // Measured (plan § 12.9): a ramp that ends ON a march step gives the same peaks at half and a quarter
    // of the step (to 0.1 K); one that ends BETWEEN steps, or an instant step, moves the peak temperature
    // by 20–70 K with the step size, because the schedule's corner falls inside an RK4 step.
    let steps = s.ramp / DS;
    if !(s.ramp >= DS - 1e-12 && (steps - steps.round()).abs() < 1e-9) {
        return Err(DoesNotRun(format!(
            "The ramp must last at least {DS} spool time constants and a whole number of {DS} steps. The run is \
             worked out in steps of {DS}; a ramp that ends between two steps (or an instant step) makes the \
             peak temperature depend on the step size by tens of kelvin.")));
    }
    if s.settle <= 0.0 {
        return no("The run must continue for some time after the move.");
    }
    if s.s_end() > MAX_RUN {
        return Err(DoesNotRun(format!(
            "The whole run (ramp + settling) may be at most {MAX_RUN} spool time constants.")));
    }
    for (which, tt4) in [("starting", s.from), ("final", s.to)] {
        fly_precheck(&s.at(tt4)).map_err(|DoesNotRun(m)| DoesNotRun(format!("At the {which} throttle: {m}")))?;
    }
    Ok(())
}

/// One steady endpoint, read off rung 34's equilibrium.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Steady {
    pub tt4: f64,
    pub nu: f64,
    pub pi_c: f64,
    pub m_corr: f64,
    pub phi: f64,
    pub thrust: f64,
    pub fuel: f64,
    /// The fuel-air ratio there.
    pub far: f64,
    pub choked: bool,
}

impl Steady {
    fn of(eq: &Instant) -> Self {
        Steady { tt4: eq.tt4, nu: eq.nu, pi_c: eq.pi_c, m_corr: eq.m, phi: eq.flowcoef, thrust: eq.thrust,
                 fuel: eq.f * eq.mdot_air, far: eq.f, choked: eq.branch == Branch::Choked }
    }

    fn to_json(self) -> Json {
        jobj! { "Tt4" => self.tt4, "nu" => self.nu, "pi_c" => self.pi_c, "m_corr" => self.m_corr,
                "phi" => self.phi, "thrust" => self.thrust, "fuel" => self.fuel, "far" => self.far,
                "choked" => Json::Int(self.choked as i64) }
    }
}

/// One slam: the two steady endpoints and the march between them.
pub struct SlamOutcome {
    pub settings: SlamSettings,
    pub start: Steady,
    pub end: Steady,
    pub points: Vec<TransientPoint>,
    /// Why the march stopped before its end, or `None` if it ran to the end.
    pub stop: Option<Stop>,
    /// The compressor face this flight gives (constant through a march): `Tt2`, `pt2`.
    pub tt2: f64,
    pub pt2: f64,
    /// The design references the corrected speed and flow are formed against.
    pub tt2_d: f64,
    pub mdot_corr_d: f64,
}

/// The temperature schedule — rung 34's `ramp_excursion` ramp, spelled the same way.
fn ramp(lo: f64, hi: f64, r: f64) -> impl Fn(f64) -> f64 {
    move |s: f64| if s <= 0.0 { lo } else if s >= r { hi } else { lo + (hi - lo) * (s / r) }
}

/// One evaluation of the march's right-hand side, in the given mode — the call `march` makes.
fn instant_at(st: &SlamSolver, nu: f64, s: f64) -> Result<Instant, String> {
    let fl = st.settings.fly.flight();
    let cmap = st.cmap;
    match st.settings.mode {
        ThrottleMode::Temperature => st.st.try_instant_marched(&fl, nu, (st.schedule)(s), Some(&cmap)),
        ThrottleMode::Fuel => st.st.try_instant_fuel(&fl, nu, (st.schedule)(s), Some(&cmap)),
    }.map_err(|e| e.0)
}

/// Everything one slam needs, built once.
pub struct SlamSolver {
    pub settings: SlamSettings,
    pub st: SpoolTransient,
    pub cmap: crate::map::ComponentMap,
    pub schedule: Box<dyn Fn(f64) -> f64>,
    pub start: Steady,
    pub end: Steady,
    /// The compressor face this flight gives (constant through a march): `Tt2`, `pt2`.
    pub tt2: f64,
    pub pt2: f64,
}

impl SlamSolver {
    /// Capture the hardware, solve both endpoints (a below-idle endpoint PANICS, as slice 3's `fly`
    /// does — the page reads it with `explain_fly`), and build the schedule for the mode.
    pub fn new(s: &SlamSettings) -> Self {
        let st = fly_solver(&s.at(s.to));
        let cmap = s.fly.map.map().with_phi_surge(s.fly.phi_surge);
        let fl = s.fly.flight();
        let eq0 = st.equilibrium(&fl, s.from, Some(&cmap));
        let eq1 = st.equilibrium(&fl, s.to, Some(&cmap));
        let (start, end) = (Steady::of(&eq0), Steady::of(&eq1));
        let schedule: Box<dyn Fn(f64) -> f64> = match s.mode {
            ThrottleMode::Temperature => Box::new(ramp(s.from, s.to, s.ramp)),
            // Rung 35's `ramp_excursion_fuel`: between the endpoints' steady fuel flows
            // (`fuel_for_tt4` = the equilibrium's `f · mdot_air`, read here off the same solves).
            ThrottleMode::Fuel => Box::new(ramp(eq0.f * eq0.mdot_air, eq1.f * eq1.mdot_air, s.ramp)),
        };
        let m = &st.inner.inner;
        let (state0, _) = m.freestream_for(&fl);
        let (tt2, pt2) = (state0.tt, m.pi_d_max * ram_recovery(fl.m0) * state0.pt);
        SlamSolver { settings: *s, st, cmap, schedule, start, end, tt2, pt2 }
    }

    /// The march — `integrate` or `integrate_fuel` itself, from the starting equilibrium's speed.
    pub fn march(&self) -> Vec<TransientPoint> {
        let fl = self.settings.fly.flight();
        let (nu0, s_end) = (self.start.nu, self.settings.s_end());
        match self.settings.mode {
            ThrottleMode::Temperature =>
                self.st.integrate(&fl, |s| (self.schedule)(s), nu0, s_end, DS, Some(&self.cmap)),
            ThrottleMode::Fuel =>
                self.st.integrate_fuel(&fl, |s| (self.schedule)(s), nu0, s_end, DS, Some(&self.cmap)),
        }
    }
}

/// Where a replayed evaluation failed: the model's message and the state it was evaluated at.
#[derive(Clone, Debug, PartialEq)]
pub struct Failure {
    pub message: String,
    pub nu: f64,
    pub s: f64,
}

fn at(sv: &SlamSolver, nu: f64, s: f64) -> Result<Instant, Failure> {
    instant_at(sv, nu, s).map_err(|message| Failure { message, nu, s })
}

/// One RK4 step of the march from `(nu, s)`, replayed: `Ok(next ν)` or the first failure met, in
/// `spool.rs` `march`'s order and arithmetic (`k1`, then `k2`/`k3` at the half step, `k4` at the full
/// one, the `0.2` speed floor).
pub fn replay_step(sv: &SlamSolver, nu: f64, s: f64) -> Result<f64, Failure> {
    let k1 = at(sv, nu, s)?.phi;
    let k2 = at(sv, nu + 0.5 * DS * k1, s + 0.5 * DS)?.phi;
    let k3 = at(sv, nu + 0.5 * DS * k2, s + 0.5 * DS)?.phi;
    let k4 = at(sv, nu + DS * k3, s + DS)?.phi;
    Ok(0.2f64.max(nu + DS / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4)))
}

/// What stopped a march, read off the failure (plan § 12.2, § 12.9 — each kind measured in the crash
/// map and driven by a test).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopKind {
    /// Fuel metered: the burner would need a fuel-air ratio above rung 35's closure search edge
    /// (`f_cap` = 0.05, `spool.rs` `try_close_compressor_fuel`) — the overshoot outran the model.
    FuelCap,
    /// The nozzle is near unchoking and the model's unchoked-nozzle turbine solve has a gap there
    /// (rung 34's own "a real subsonic-solve gap" escalation).
    SubsonicGap,
    /// Temperature commanded, on a fast power cut: the commanded temperature fell to the air the
    /// still-fast compressor delivers, so the burner would need zero or almost zero fuel — the engine
    /// would flame out. Rung 34's marched closure says so (`spool.rs`); until 2026-10-10 these
    /// stops were mislabelled as the airflow search's first-trial artefact (plan § 12.9).
    FlameOut,
    /// The shaft speed changed so fast that an RK stage carried it to zero or below: the fixed step is too
    /// coarse for this engine at this flight (measured at Mach 3.3, where the shaft's own response is far
    /// faster than its design time constant).
    Overstep,
    /// Any other burner fuel solve that did not converge.
    Burner,
    Other,
}

impl StopKind {
    /// What the page says when a run stops this way — above the model's own message, which it always
    /// prints too. Each kind was measured in the crash map and is driven by a test.
    pub fn words(self) -> &'static str {
        match self {
            StopKind::FuelCap =>
                "The fuel arrived faster than the shaft could bring in air to burn it, so the turbine-inlet                  temperature shot past its target. The next step would need more than 0.05 kg of fuel per kg of air,                  the most the model's fuel-metered solver searches (burning every bit of the oxygen takes about                  0.068). A real engine's fuel control exists to prevent exactly this. Make the move slower, or start                  from a higher throttle.",
            StopKind::SubsonicGap =>
                "The nozzle is close to the point where it stops being choked (the jet just under the speed of                  sound), and the model's solver for that case has a known gap there: it cannot find the turbine's                  operating point. This is a limit of the model, not of the engine.",
            StopKind::FlameOut =>
                "The temperature you are commanding has fallen to the temperature of the air already leaving the                  compressor, which is still spinning fast: to hold it the burner would have to burn next to no                  fuel at all. A real engine would flame out here. Cut the throttle more slowly or not                  so far, or meter the fuel instead (what a real engine does).",
            StopKind::Overstep =>
                "The shaft's speed was changing so fast here that one of the model's fixed time steps carried it                  past zero. The time step (0.02 τ) is too coarse for this engine at this flight, so the last points                  before the stop are not to be trusted either. This is a limit of the model's stepping, not of the                  engine.",
            StopKind::Burner =>
                "The burner's fuel balance did not converge at this point of the run.",
            StopKind::Other =>
                "The model stopped on one of its internal checks at this point of the run.",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            StopKind::FuelCap => "fuel_cap",
            StopKind::SubsonicGap => "subsonic_gap",
            StopKind::FlameOut => "flame_out",
            StopKind::Overstep => "overstep",
            StopKind::Burner => "burner",
            StopKind::Other => "other",
        }
    }
}

/// Why and where a march stopped early.
#[derive(Clone, Debug, PartialEq)]
pub struct Stop {
    pub failure: Failure,
    pub kind: StopKind,
}

/// Classify a failure by where it happened (a stage at zero speed or below) and by its message.
///
/// A fourth kind lived here until 2026-10-10: a commanded power cut stopped when the airflow search's
/// FIRST trial (its lowest flow) asked the burner to cool the air. Rung 34's march now walks that wall
/// in (`SpoolTransient::try_close_compressor_marched`), so the cut runs through (plan § 12.9).
pub fn classify(_sv: &SlamSolver, f: &Failure) -> StopKind {
    let m = &f.message;
    if f.nu <= 0.0 {
        StopKind::Overstep
    } else if m.contains("fuel compressor closure does not bracket") {
        StopKind::FuelCap
    } else if m.contains("subsonic turbine failed to bracket AWAY") {
        StopKind::SubsonicGap
    } else if m.contains("the burner would need zero or almost zero fuel") {
        StopKind::FlameOut
    } else if m.contains("burner f did not converge") {
        StopKind::Burner
    } else {
        StopKind::Other
    }
}

/// Why a march that stopped early stopped: replay the step from its last recorded point, then the next
/// step's opening evaluation (the march's `k1` at a state it never records). `None` if it ran to the
/// end. A march that recorded no point failed at its very first evaluation, which is re-made.
pub fn stop_reason(sv: &SlamSolver, points: &[TransientPoint]) -> Option<Stop> {
    if points.len() >= sv.settings.expected_points() {
        return None;
    }
    let failure = match points.last() {
        None => at(sv, sv.start.nu, 0.0).err(),
        Some(last) => match replay_step(sv, last.nu, last.s) {
            Err(f) => Some(f),
            Ok(nu) => at(sv, nu, last.s + DS).err(),
        },
    }.unwrap_or_else(|| Failure { message: "the march stopped, but replaying its last step does not fail".into(),
                                  nu: f64::NAN, s: f64::NAN });
    let kind = classify(sv, &failure);
    Some(Stop { failure, kind })
}

/// The highest fuel-air ratio rung 35's fuel-metered closure searches — `f_cap` in
/// `SpoolTransient::try_close_compressor_fuel`, copied (the model spells it inline). The closure's low
/// flow wall IS this ratio at the metered fuel, so a point needing more is unreachable on that path;
/// `tests/sandbox_transient.rs` pins it against the closure.
pub const FUEL_CAP: f64 = 0.05;

/// Fuel metered: a steady endpoint whose own fuel-air ratio is above [`FUEL_CAP`] cannot be reached
/// by ANY ramp — refused in words, because the overshoot's words ("make the move slower") would be wrong
/// there (plan § 12.9: the crash map's fuel-cap stops include starts that never record a point).
pub fn fuel_reach_check(sv: &SlamSolver) -> Result<(), DoesNotRun> {
    if sv.settings.mode != ThrottleMode::Fuel {
        return Ok(());
    }
    for (which, e) in [("starting", sv.start), ("final", sv.end)] {
        if e.far > FUEL_CAP {
            return Err(DoesNotRun(format!(
                "Metering the fuel cannot reach the {which} throttle ({:.0} K): holding it steady takes {:.4} kg of fuel                  per kg of air, above the 0.05 the model's fuel-metered solver searches. Command the temperature                  instead, or pick a cooler throttle.", e.tt4, e.far)));
        }
    }
    Ok(())
}

/// Run one slam.
pub fn slam(s: &SlamSettings) -> Result<SlamOutcome, DoesNotRun> {
    slam_precheck(s)?;
    let sv = SlamSolver::new(s);
    fuel_reach_check(&sv)?;
    let points = sv.march();
    let stop = stop_reason(&sv, &points);
    let m = &sv.st.inner;
    Ok(SlamOutcome { settings: *s, start: sv.start, end: sv.end, points, stop, tt2: sv.tt2, pt2: sv.pt2,
                     tt2_d: m.tt2_d, mdot_corr_d: m.mdot_corr_d })
}

/// The page's view of a slam, FULL precision. The trajectory leaves as COLUMNS (one list per
/// quantity), the shape the page's charts read. Corrected speed and flow are formed from the recorded
/// shaft speed and air flow at this flight's (constant) compressor face — the same definitions the
/// closure uses (`spool.rs` `try_instant`, `eval_m_fuel`).
pub fn slam_json(o: &SlamOutcome) -> Json {
    let p = &o.points;
    let col = |f: &dyn Fn(&TransientPoint) -> f64| Json::List(p.iter().map(|x| Json::Float(f(x))).collect());
    let n_of = |x: &TransientPoint| x.nu * powp(o.tt2_d / o.tt2, 0.5);
    let m_of = |x: &TransientPoint| (x.mdot_air * powp(o.tt2, 0.5) / o.pt2) / o.mdot_corr_d;
    jobj! {
        "ok" => Json::Int(1),
        "slam" => o.settings.to_json(),
        "start" => o.start.to_json(), "end" => o.end.to_json(),
        "phi_surge" => o.settings.fly.phi_surge,
        "expected_points" => Json::Int(o.settings.expected_points() as i64),
        "stopped" => Json::Int(o.stop.is_some() as i64),
        "stop" => match &o.stop {
            None => Json::Null,
            Some(st) => jobj! { "kind" => st.kind.key(), "words" => st.kind.words(), "message" => st.failure.message.clone(),
                                "s" => num_or_null(st.failure.s), "nu" => num_or_null(st.failure.nu) },
        },
        "s" => col(&|x| x.s), "nu" => col(&|x| x.nu), "Tt4" => col(&|x| x.tt4),
        "pi_c" => col(&|x| x.pi_c), "fuel" => col(&|x| x.f * x.mdot_air), "mdot_air" => col(&|x| x.mdot_air),
        "thrust" => col(&|x| x.sp_thrust * x.mdot_air), "M9" => col(&|x| x.m9),
        "n_corr" => col(&n_of), "m_corr" => col(&m_of), "phi" => col(&|x| m_of(x) / n_of(x)),
        "choked" => Json::List(p.iter().map(|x| Json::Int((x.branch == Branch::Choked) as i64)).collect()),
    }
}

fn num_or_null(x: f64) -> Json { if x.is_finite() { Json::Float(x) } else { Json::Null } }

/// `{"op":"slam_defaults"}` and `{"op":"slam","slam":{…}}` — the dispatcher `sandbox::call` falls to.
pub fn call_op(op: &str, req: &Json) -> Option<Json> {
    let refusal = |m: &str| jobj! { "ok" => Json::Int(0), "reason" => m };
    Some(match op {
        "slam_defaults" => jobj! { "slam" => SlamSettings::defaults().to_json(), "ds" => DS, "max_run" => MAX_RUN },
        "slam" => {
            let empty = Json::Obj(Vec::new());
            match SlamSettings::from_json(req.get("slam").unwrap_or(&empty)) {
                Err(e) => refusal(&e),
                Ok(s) => slam(&s).map(|o| slam_json(&o)).unwrap_or_else(|DoesNotRun(m)| refusal(&m)),
            }
        }
        _ => return None,
    })
}

/// The slam requests the browser check (`rust/sandbox-wasm/check.mjs`) compares with the native model:
/// the page's opening slam, then every offered gas in both modes on a slam and a chop, the measured
/// early stop (the fuel cap on the thermally perfect gas — so a stop's position and words are compared
/// too), and the fast commanded chop that stopped on the airflow search's first trial until rung 34's
/// march walked that wall in (it now runs through the marched closure).
pub fn check_requests() -> Vec<String> {
    let mut out = vec![r#"{"op":"slam","slam":{}}"#.to_string()];
    let d = SlamSettings::defaults();
    for gas in [GasModel::Perfect, GasModel::ThermallyPerfect, GasModel::Reacting, GasModel::ForkB] {
        for mode in ThrottleMode::ALL {
            for (from, to, ramp) in [(1100.0, 1500.0, 0.5), (1500.0, 1000.0, 0.1)] {
                let s = SlamSettings { fly: FlySettings { gas, ..d.fly }, from, to, ramp, settle: 1.5, mode };
                out.push(jobj! { "op" => "slam", "slam" => s.to_json() }.dump_compact());
            }
        }
    }
    let stops = [(GasModel::ThermallyPerfect, ThrottleMode::Fuel, 1000.0, 1500.0, 0.1),
                 (GasModel::Perfect, ThrottleMode::Temperature, 1500.0, 640.0, 0.06)];
    for (gas, mode, from, to, ramp) in stops {
        let s = SlamSettings { fly: FlySettings { gas, ..d.fly }, from, to, ramp, settle: 3.0, mode };
        out.push(jobj! { "op" => "slam", "slam" => s.to_json() }.dump_compact());
    }
    out
}
