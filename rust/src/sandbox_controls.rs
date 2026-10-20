//! The web sandbox, slice 4 (B) — **CONTROLS** (`docs/plans/sandbox-plan.md` § 12). NOT a rung.
//!
//! The throttle slam of slice 4 (A), on a TWO-shaft engine, with the fuel controls and one airflow
//! lever as switches. Every fuel limiter (rungs 46–52) and every airflow lever on the transient
//! (57–63) is two-shaft by assertion — its finding is a split BETWEEN spools — so this view has its
//! own engine (plan § 12.1): the *Size the blades* view's, at its opening settings (rung 43's rig:
//! `π_LPC` 3 × `π_HPC` 6, `Tt4` 1500, a convergent nozzle, the panels' two-spool losses, at 250 K /
//! 50 kPa / Mach 0.85, the perfect gas), on rungs 43/45's fuel-metered march. Since plan § 27 it is FLOWN
//! at the knobs' air temperature and pressure (Mach kept), its hardware still captured at that design flight.
//!
//! **The switches** (plan § 12.3, the user's answers § 12.8): the turbine-temperature limiter
//! (rung 46) and its response lag (47); the acceleration schedule (48); the stall floor (49), read
//! as a flow coefficient or — with the stators moving — as an incidence (60); the realistic
//! fast-attack / slow-release lag (52); and the airflow levers: the stator schedule (57), the bleed
//! schedule (62), or BOTH beside any fuel limiter (rung 86, which measured that plant); and the bleed valve
//! either on its schedule or opening on its own to hold a stall margin read as a flow coefficient or as an
//! incidence (rungs 64 and 87). The fuel follows a ramp between the two endpoints' steady fuel flows, exactly as
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
use crate::lagged_bleed::build_lagged_bleed;
use crate::limited_bleed::{build_limited_bleed, r64_solve_b_at, r87_target, BleedLimiter};
use crate::blade_speed::shape_maps;
use crate::customer_bleed::CustomerBleed;
use crate::turbine_cooling::TurbineCooling;
use crate::shaft_offtake::{self, Demand, ShaftOfftake, ShaftOfftakeMatcher};
use crate::metal_temperature::{self as mt, MetalInputs, MetalLimit, MetalReading, Row, TECHNOLOGIES};
use crate::sandbox_cooling::{LIFE_MAX, LIFE_MIN, TECH_KEYS, YEAR_MAX, YEAR_MIN};
use crate::burner_hold::{self as bh, Hold};
use crate::governor::{self as gv, GovCall, GovMark, Governor, Law, Leg};
use crate::inferred_thrust as inf;
use crate::corrected_tables::{Carry, TableAt};
use crate::mach_axis::{self as ma, SpeedCorr, Tables};
use crate::unchoked_formula::Formula;
use crate::airflow_source::{self as af, AirflowFrom, SensorLag};
use crate::engine::FlightCondition;
use crate::fuel_transient::{r43_cut_below_ceiling, release_weight, AccelSchedule, AsymmetricLag, FuelInstant, FuelLimiters,
                            FuelPoint, FuelTransientCore, PointExtra, SurgeLimiter};
use crate::components::ram_recovery;
use crate::gas::{powp, Abort};
use crate::jobj;
use crate::map::ComponentMap;
use crate::matcher::Branch;
use crate::sandbox::{ab_precheck, DoesNotRun};
use crate::afterburner::{AbMode, Afterburner, Throat, TwoShaftFit};
use crate::sandbox_blades::BladeSettings;
use crate::stator_transient::{IncidenceLimiter, ScheduledStatorCore, ScheduledStatorTransient, StatorArm,
                              StatorSchedule};
use crate::two_spool::{IdleRule, Spool, TwoSpoolEngine};
use crate::bleed_transient::r62_try_close_fuel;
use crate::two_spool_transient::{ForcedBleed, Instant2, MarchedBleed};
use crate::hp_vane::HpVane;
use crate::turbine_law::TurbineLaw;
use crate::visuals::Json;

/// The march step, in HP spool time constants — rung 43's grid. (Rungs 57/62 ran at 0.01; the
/// step is checked against it in the crash map, plan § 12.5.)
pub const DS: f64 = 0.02;

/// RUNG 102's defaults on this view: the afterburner's temperature (Fly it's), and the light-off at the ramp's end of
/// the page's opening run (full throttle reached — where a pilot lights it).
pub const AB_TT7: f64 = 2000.0;
pub const AB_LIGHT_AT: f64 = 0.5;

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

/// The airflow lever(s). `Both` is rung 86's plant — rung 62's machine with both schedules armed —
/// and every fuel limiter runs beside it (plan § 15; until rung 86 it was refused as open physics).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeverChoice {
    None,
    /// Rung 57: the variable stators close at low corrected speed on ONE spool.
    Stator,
    /// Rung 62: the handling-bleed valve opens at low LP corrected speed.
    Bleed,
    /// Rung 86: the stator schedule AND the bleed schedule, each with its own numbers.
    Both,
}

impl LeverChoice {
    pub const ALL: [LeverChoice; 4] = [LeverChoice::None, LeverChoice::Stator, LeverChoice::Bleed, LeverChoice::Both];
    pub fn key(self) -> &'static str {
        match self {
            LeverChoice::None => "none",
            LeverChoice::Stator => "stator",
            LeverChoice::Bleed => "bleed",
            LeverChoice::Both => "both",
        }
    }
    /// The stators move (alone or with the bleed).
    pub fn stator(self) -> bool {
        matches!(self, LeverChoice::Stator | LeverChoice::Both)
    }
    /// The bleed valve opens (alone or with the stators).
    pub fn bleed(self) -> bool {
        matches!(self, LeverChoice::Bleed | LeverChoice::Both)
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

/// How the bleed valve moves: on its speed schedule (rung 62), or on its own, opening just enough to hold
/// a stall margin on the LP compressor (rung 64's valve; rung 87 gave it the incidence reference).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BleedMode {
    Schedule,
    Valve,
}

impl BleedMode {
    pub const ALL: [BleedMode; 2] = [BleedMode::Schedule, BleedMode::Valve];
    pub fn key(self) -> &'static str {
        match self { BleedMode::Schedule => "schedule", BleedMode::Valve => "valve" }
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
    /// Rungs 64/87: the bleed valve on its schedule or holding a margin; the LP flow coefficient it holds
    /// (at the design stator setting, as the floor's), and what it watches. `b_max` is its largest opening.
    pub bleed_mode: BleedMode,
    pub valve_phi: f64,
    pub valve_ref: FloorRef,
    /// Rung 88: the valve's speed — instant (rungs 64/87), or a first-order lag of time constant
    /// `valve_tau` (rung 65's, which rung 88 gave the incidence reference).
    pub valve_lag_on: bool,
    pub valve_tau: f64,
    /// Rung 89: a slow valve CLOSING at its own time constant `valve_tau_close` (`valve_tau` is then its
    /// opening one); off, it closes at `valve_tau` — rung 88's valve, bit for bit.
    pub valve_close_own: bool,
    pub valve_tau_close: f64,
    /// Rung 90: the aircraft's customer bleed off station 3 — off (the default: every saved setting
    /// stays valid), a fixed SHARE of the HP compressor's air, or a fixed AMOUNT (× the design airflow,
    /// so a larger share at low power).
    pub customer: CustomerMode,
    pub customer_share: f64,
    pub customer_amount: f64,
    /// Rung 91: turbine cooling — the VANE and BLADE shares of the HP compressor's air. Both 0 (the
    /// default) is no cooling, so every saved setting stays valid.
    pub cool_vane: f64,
    pub cool_blade: f64,
    /// Rung 92: the aircraft's generator load — off (the default: every saved setting stays valid), on
    /// the HP or the LP shaft, set as a SHARE of that turbine's work or a fixed POWER (× the design HP
    /// compressor power, the same unit on both shafts, so a larger share at low power).
    pub generator: GeneratorMode,
    pub generator_by: GeneratorBy,
    pub generator_share: f64,
    pub generator_power: f64,
    /// Rung 93: read both turbine metals along the run (off by default: uncooled, the metal is over its
    /// limit at every opening throttle), with Gauntner's cooling technology (an index into
    /// [`TECHNOLOGIES`]) and the limits from a material year and a part life.
    pub metal_on: bool,
    pub metal_tech: usize,
    pub metal_year: f64,
    pub metal_life: f64,
    /// Rung 94: what each throttle end holds while the generator is on — the burner temperature (rung 92's throttle,
    /// the default), or the thrust or HP speed this engine gives at that temperature with the generator off, the burner
    /// temperature floating to carry the load; and whether the metal limit (rung 93) BINDS the floating burner.
    pub hold: Hold,
    pub hold_cap: bool,
    /// Rung 95: GOVERN the held quantity all along the move — the governor's fuel replaces the ramp as the demand the
    /// limiters cut. Holding HP speed, `speed_law` picks the commanded rate (`gov_tau`, HP spool τ) or the proportional
    /// law (`gov_gain`, fuel fraction per unit HP speed short).
    pub govern: bool,
    pub speed_law: SpeedLaw,
    pub gov_tau: f64,
    pub gov_gain: f64,
    /// Rung 96: a governed thrust is READ from the thrust itself (rung 95), the engine pressure ratio, or the LP speed
    /// (`speed_law` and its knobs then on the LP shaft), turned into thrust through the control's table — built on the
    /// generator-off engine (`table_clean`, inert with the generator off) or on this one.
    pub thrust_by: ThrustBy,
    pub table_clean: bool,
    /// Rung 98: where the control's tables (rung 96's thrust table, rung 48's schedule, rung 95's steady table) are
    /// made — once at the DESIGN flight (the default, the user's choice) and carried to the flight flown, correcting for
    /// the air's pressure (`corr_p`) and temperature (`corr_t`); or at THIS flight (rungs 48/95/96, the reduce setting).
    pub table_at: TableAt,
    pub corr_p: bool,
    pub corr_t: bool,
    /// Rung 99: how a design-flight table is corrected for flight SPEED — tables at eight grid speeds blended at the one
    /// flown (the default, the user's choice), the thrust recomputed for it, or not at all (rung 98).
    pub speed_corr: SpeedCorr,
    /// RUNG 100's two reduce settings, hidden (request keys, not on the page): `low_grid` — the grid's two slow slices
    /// (off: rung 99's eight); `formula_choked` — rung 99's formula column (off: the formula read at the reading).
    /// Written to a reply only when not at their defaults, so every default reply is rung 99's in shape.
    pub low_grid: bool,
    pub formula_choked: bool,
    /// What a steady end below idle gets (sandbox plan §§ 38–39), hidden: `Accept` by default (the user's choice — the
    /// run starts from it and the page warns, whichever start found it); `Refuse` (§ 38's floor); `Old` (§ 37's rule, the
    /// reduce setting rungs 98/99's tests read their anchored cells with). Written to a reply only off the default.
    pub idle: IdleRule,
    /// The fuel closures' lean wall (sandbox plan § 40), hidden: `false` by default (the user's choice — the wall drops to
    /// half the leaner steady end's mixture when that is under rung 43's 0.004, so a slam can leave a lean start); `true`
    /// keeps 0.004 (the old code, request key `lean: old`). Written to a reply only off the default.
    pub lean_old: bool,
    /// RUNG 101: where the formula (`speed_corr: formula`) gets the air swallowed — rung 100's table (the default),
    /// the speed grid's, or the exhaust-temperature sensor; and that sensor's lag (off by default; HP-shaft τ). Written to
    /// a reply only off their defaults, so every default reply is rung 100's in shape.
    pub airflow: AirflowFrom,
    pub exhaust_lag: bool,
    pub exhaust_tau: f64,
    /// The flight FLOWN (plan § 27): ambient static temperature, K, and pressure, Pa — the page's altitude and
    /// deviation from the standard day, linked to these through the standard atmosphere as on *Fly it*. The hardware
    /// stays captured at the DESIGN flight ([`ControlsSettings::design_flight`]). The defaults ARE the design flight,
    /// so every saved setting stays valid.
    pub t0: f64,
    pub p0: f64,
    /// The flight Mach number FLOWN (plan § 29); the default is the design's 0.85.
    pub m0: f64,
    /// RUNG 102 (Part C): the afterburner — not fitted (`None`, the default: every saved setting stays valid), fitted
    /// and never lit (`Unlit`), or `Lit` at `light_at` (HP spool τ into the run, on the march grid) over
    /// [`crate::afterburner::LIGHT_RAMP`]; its temperature `tt7`, and its throat as on *Fly it* (`a8_ratio` × the dry
    /// throat, or opened by the rule with `a8_match`). Written to a reply only off their defaults.
    pub ab: Option<AbMode>,
    pub tt7: f64,
    pub light_at: f64,
    pub a8_ratio: f64,
    pub a8_match: bool,
    /// RUNG 104: what sets the LP turbine's flow — ellipse (the default), two rows, one row, or the old pin. Written to a
    /// reply only off its default.
    pub turbine: TurbineLaw,
    /// RUNG 105: what sets the HP turbine vane's flow — the old choked vane (the default) or the reaction law. Written to a
    /// reply only off its default.
    pub hp_vane: HpVane,
}

/// Rung 96's switch: where a governed thrust is read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThrustBy {
    /// The thrust itself — rung 95.
    True,
    /// The engine pressure ratio, through the table.
    Epr,
    /// The LP shaft speed, through the table.
    N1,
}

impl ThrustBy {
    pub const ALL: [ThrustBy; 3] = [ThrustBy::True, ThrustBy::Epr, ThrustBy::N1];
    pub fn key(self) -> &'static str {
        match self { ThrustBy::True => "true", ThrustBy::Epr => "epr", ThrustBy::N1 => "n1" }
    }
}

/// Rung 95's toggle: how a governor holding HP speed closes the gap (the user's "both").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedLaw {
    /// The commanded rate: the shaft asked to close its gap in `gov_tau`.
    Rate,
    /// The proportional law: the steady fuel at the asked speed, plus `gov_gain` × the gap.
    Prop,
}

impl SpeedLaw {
    pub const ALL: [SpeedLaw; 2] = [SpeedLaw::Rate, SpeedLaw::Prop];
    pub fn key(self) -> &'static str {
        match self { SpeedLaw::Rate => "rate", SpeedLaw::Prop => "prop" }
    }
}

/// Rung 95: the largest proportional gain the page runs — set where the march at its step stops agreeing with a
/// finer one (`docs/rung95-spec.md`).
pub const GAIN_MAX: f64 = 50.0;

/// Rung 95: the shortest commanded-rate time constant — two steps (at one step the march at 0.02 is ~1e-3 off a finer
/// one in flow coefficient on the default move; at two, under 1e-4 on every map's acceleration).
pub const TAU_MIN: f64 = 2.0 * DS;

/// Rung 96: the LP-speed laws' limits — their OWN step check (`docs/rung96-spec.md`), not the HP shaft's.
pub const TAU_MIN_LP: f64 = 2.0 * DS;
pub const GAIN_MAX_LP: f64 = 50.0;

/// Rung 92's switch: which shaft the generator loads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratorMode {
    Off,
    Hp,
    Lp,
}

impl GeneratorMode {
    pub const ALL: [GeneratorMode; 3] = [GeneratorMode::Off, GeneratorMode::Hp, GeneratorMode::Lp];
    pub fn key(self) -> &'static str {
        match self { GeneratorMode::Off => "off", GeneratorMode::Hp => "hp", GeneratorMode::Lp => "lp" }
    }
}

/// Rung 92's switch: how the generator load is set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratorBy {
    Share,
    Power,
}

impl GeneratorBy {
    pub const ALL: [GeneratorBy; 2] = [GeneratorBy::Share, GeneratorBy::Power];
    pub fn key(self) -> &'static str {
        match self { GeneratorBy::Share => "share", GeneratorBy::Power => "power" }
    }
}

/// Rung 90's switch: how the aircraft's customer bleed is set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomerMode {
    Off,
    Share,
    Amount,
}

impl CustomerMode {
    pub const ALL: [CustomerMode; 3] = [CustomerMode::Off, CustomerMode::Share, CustomerMode::Amount];
    pub fn key(self) -> &'static str {
        match self { CustomerMode::Off => "off", CustomerMode::Share => "share", CustomerMode::Amount => "amount" }
    }
}

impl ControlsSettings {
    /// Rung 48/49's ramp — 1000 → 1400 K over half a spool time — every switch off, the numbers each
    /// rung used when it was switched on: redline 1480 K (rungs 46–49), lag 0.2 τ (47), margin 0.25
    /// (48), LP floor 0.75 (49), attack/release 0.02/0.10 τ (52), stator 0.20 below 0.65 and bleed
    /// 0.10 below 0.65 (62); the valve, when chosen, holds 0.78 (rung 87's middle set point, `sm` ≈ 0.42).
    pub fn defaults() -> Self {
        let fd = BladeSettings::defaults().flight();
        ControlsSettings {
            shape: 0, rho: 1.0, from: 1000.0, to: 1400.0, ramp: 0.5, settle: 2.0,
            redline_on: false, redline: 1480.0, gov_lag_on: false, gov_lag: 0.2,
            accel_on: false, accel_margin: 0.25,
            floor_on: false, floor_spool: Spool::Lp, floor_phi: 0.75, floor_ref: FloorRef::Phi,
            release_on: false, tau_att: 0.02, tau_rel: 0.10,
            lever: LeverChoice::None, stator_spool: Spool::Lp, v_max: 0.20, v_n_lo: 0.65, b_max: 0.10, b_n_lo: 0.65,
            bleed_mode: BleedMode::Schedule, valve_phi: 0.78, valve_ref: FloorRef::Phi,
            valve_lag_on: false, valve_tau: 0.05, valve_close_own: false, valve_tau_close: 0.2,
            customer: CustomerMode::Off, customer_share: 0.05, customer_amount: 0.05,
            cool_vane: 0.0, cool_blade: 0.0,
            generator: GeneratorMode::Off, generator_by: GeneratorBy::Power, generator_share: 0.05,
            generator_power: 0.02,
            metal_on: false, metal_tech: 2, metal_year: 1985.0, metal_life: 1.0e4,
            hold: Hold::BurnerTemp, hold_cap: false,
            govern: false, speed_law: SpeedLaw::Rate, gov_tau: 0.1, gov_gain: 20.0,
            thrust_by: ThrustBy::True, table_clean: true,
            table_at: TableAt::Design, corr_p: true, corr_t: true, speed_corr: SpeedCorr::Grid,
            low_grid: true, formula_choked: false, idle: IdleRule::Accept, lean_old: false,
            airflow: AirflowFrom::Table, exhaust_lag: false, exhaust_tau: 1.0,
            t0: fd.t0, p0: fd.p0, m0: fd.m0,
            ab: None, tt7: AB_TT7, light_at: AB_LIGHT_AT, a8_ratio: 1.0, a8_match: false,
            turbine: TurbineLaw::default(),
            hp_vane: HpVane::default(),
        }
    }

    /// Rung 91's cooling as the model takes it (`None` when both shares are 0). Range-checked by the
    /// precheck first.
    pub fn turbine_cooling(&self) -> Option<TurbineCooling> {
        (self.cool_vane != 0.0 || self.cool_blade != 0.0)
            .then(|| TurbineCooling::new(self.cool_vane, self.cool_blade))
    }

    /// Rung 92's load as the model takes it (`None` when off). Range-checked by the precheck first.
    pub fn shaft_offtake(&self) -> Option<ShaftOfftake> {
        let d = match self.generator_by {
            GeneratorBy::Share => Demand::share(self.generator_share),
            GeneratorBy::Power => Demand::power(self.generator_power),
        };
        match self.generator {
            GeneratorMode::Off => None,
            GeneratorMode::Hp => Some(ShaftOfftake::hp(d)),
            GeneratorMode::Lp => Some(ShaftOfftake::lp(d)),
        }
    }

    /// Rung 90's demand as the model takes it (`None` when off). Range-checked by the precheck first.
    pub fn customer_bleed(&self) -> Option<CustomerBleed> {
        match self.customer {
            CustomerMode::Off => None,
            CustomerMode::Share => Some(CustomerBleed::share(self.customer_share)),
            CustomerMode::Amount => Some(CustomerBleed::amount(self.customer_amount)),
        }
    }

    /// Rung 93's limits (`None` when the metal readout is off).
    pub fn metal_limit(&self) -> Option<MetalLimit> {
        self.metal_on.then(|| MetalLimit::new(TECHNOLOGIES[self.metal_tech].1,
                                              mt::allowable(Row::Vane, self.metal_year, self.metal_life),
                                              mt::allowable(Row::Blade, self.metal_year, self.metal_life)))
    }

    /// Both metals at an instant — rung 93's reading from the instant's temperatures and this run's
    /// shares (the transient carries no station table: [`MetalInputs::of_shares`]).
    pub fn metal_at(&self, i: &Instant2) -> Option<MetalReading> {
        let lim = self.metal_limit()?;
        let cool = TurbineCooling { vane: self.cool_vane, blade: self.cool_blade };
        let inp = MetalInputs::of_shares(&BladeSettings::defaults().gas.gas(), i.tt4, i.close.tt3, i.close.f,
                                         i.close.customer.unwrap_or(0.0), cool);
        Some(mt::read(&inp, &lim))
    }

    /// Rung 95: the governor's law, from what the throttle holds and the speed toggle.
    pub fn law(&self) -> Law {
        match (self.hold, self.speed_law) {
            (Hold::BurnerTemp, _) => Law::BurnerTemp,
            (Hold::Thrust, law) => match (self.thrust_by, law) {
                (ThrustBy::True, _) => Law::Thrust,
                (ThrustBy::Epr, _) => Law::Epr,
                (ThrustBy::N1, SpeedLaw::Rate) => Law::LpRate { tau: self.gov_tau },
                (ThrustBy::N1, SpeedLaw::Prop) => Law::LpProp { gain: self.gov_gain },
            },
            (Hold::HpSpeed, SpeedLaw::Rate) => Law::SpeedRate { tau: self.gov_tau },
            (Hold::HpSpeed, SpeedLaw::Prop) => Law::SpeedProp { gain: self.gov_gain },
        }
    }

    /// Rung 95: the metal leg runs — the governor is on and the cap switched on (its prerequisites checked first).
    pub fn metal_leg(&self) -> bool { self.govern && self.hold_cap }

    /// Rung 96: the governed thrust is inferred from a reading (the control's table is built).
    pub fn inferred(&self) -> bool { self.govern && self.hold == Hold::Thrust && self.thrust_by != ThrustBy::True }

    /// Rung 96: the ends are CALIBRATED — inferred, the table on the generator-off engine, and a generator to differ.
    pub fn calibrated(&self) -> bool { self.inferred() && self.table_clean && self.generator != GeneratorMode::Off }

    /// Rung 98: the run reads a table — rung 96's (an inferred thrust), rung 48's schedule, or rung 95's steady table
    /// (the proportional HP-speed law).
    pub fn uses_table(&self) -> bool {
        self.inferred() || self.accel_on
            || (self.govern && self.hold == Hold::HpSpeed && self.speed_law == SpeedLaw::Prop)
    }

    /// Rung 98: flown at the design flight itself (bitwise), where a design-flight table IS the run's own.
    pub fn at_design_flight(&self) -> bool {
        let d = self.design_flight();
        self.t0 == d.t0 && self.p0 == d.p0 && self.m0 == d.m0
    }

    /// Rung 98: the tables are made at the design flight and CARRIED to this one — otherwise rungs 48/95/96's own.
    /// Rung 99: under the speed grid, flown at a grid slice's own flight, the slice IS a table made at this flight.
    pub fn carried(&self) -> bool {
        self.table_at == TableAt::Design && self.uses_table() && !self.at_design_flight()
            && !(self.speed_corr == SpeedCorr::Grid
                 && ma::at_slice_flight(ma::grid(self.low_grid), &self.design_flight(), &self.flight()))
    }

    /// The flight the hardware was captured at — the *Size the blades* rig's (250 K, 50 kPa, Mach 0.85). Every
    /// CONSTRUCTOR takes this one; the flight flown is [`ControlsSettings::flight`].
    pub fn design_flight(&self) -> FlightCondition { BladeSettings::defaults().flight() }

    /// The flight FLOWN: the knobs' `T0`/`p0`/`M0`. Every solve and the march take this one.
    pub fn flight(&self) -> FlightCondition { FlightCondition::new(self.t0, self.p0, self.m0) }

    /// The density of the air the engine SWALLOWS over the design flight's — the inlet's total density,
    /// `(p0/p0d)·(T0d/T0)·(k(M0)/k(M0d))` with `k = π_ram·(1 + (γ−1)/2·M²)^(1/(γ−1))` — exactly 1 at the design flight.
    /// The governor's step-checked knob limits scale by it (plan § 27): the shafts answer fuel harder in denser air.
    /// Plan § 29: flight speed rams the air denser, so the Mach factor joins; at the design's Mach it is `x/x`, exactly
    /// 1, and the ratio is § 27's bit for bit.
    pub fn density_ratio(&self) -> f64 {
        let d = self.design_flight();
        let g = BladeSettings::defaults().gas.gas().gamma_c();
        let k = |m: f64| ram_recovery(m) * powp(1.0 + 0.5 * (g - 1.0) * m * m, 1.0 / (g - 1.0));
        (self.p0 / d.p0) * (d.t0 / self.t0) * (k(self.m0) / k(d.m0))
    }

    pub fn s_end(&self) -> f64 { self.ramp + self.settle }

    /// How many points a march that runs to the end records — the marcher's own step count.
    pub fn expected_points(&self) -> usize { (self.s_end() / DS).round_ties_even() as usize + 1 }

    pub fn to_json(&self) -> Json {
        let b = |x: bool| Json::Int(x as i64);
        let mut j = jobj! {
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
            "bleed_mode" => self.bleed_mode.key(), "valve_phi" => self.valve_phi, "valve_ref" => self.valve_ref.key(),
            "valve_speed" => if self.valve_lag_on { "slow" } else { "instant" }, "valve_tau" => self.valve_tau,
            "valve_close" => if self.valve_close_own { "own" } else { "same" }, "valve_tau_close" => self.valve_tau_close,
            "customer" => self.customer.key(), "customer_share" => self.customer_share,
            "customer_amount" => self.customer_amount, "cool_vane" => self.cool_vane,
            "cool_blade" => self.cool_blade,
            "generator" => self.generator.key(), "generator_by" => self.generator_by.key(),
            "generator_share" => self.generator_share, "generator_power" => self.generator_power,
            "metal_on" => b(self.metal_on), "metal_tech" => TECH_KEYS[self.metal_tech], "metal_year" => self.metal_year,
            "metal_life" => self.metal_life,
            "hold" => self.hold.key(), "hold_cap" => b(self.hold_cap),
            "govern" => b(self.govern), "speed_law" => self.speed_law.key(), "gov_tau" => self.gov_tau,
            "gov_gain" => self.gov_gain,
            "thrust_by" => self.thrust_by.key(), "table_clean" => b(self.table_clean),
            "table_at" => self.table_at.key(), "corr_p" => b(self.corr_p), "corr_t" => b(self.corr_t),
            "speed_corr" => self.speed_corr.key(),
            "T0" => self.t0, "p0" => self.p0, "M0" => self.m0,
        };
        // Rung 100's hidden reduce settings, only off their defaults.
        if let Json::Obj(kv) = &mut j {
            if !self.low_grid { kv.push(("low_grid".into(), b(false))); }
            if self.formula_choked { kv.push(("formula_choked".into(), b(true))); }
            // The below-idle floor's, likewise (plan § 38).
            if self.idle != IdleRule::Accept { kv.push(("idle".into(), Json::from(idle_key(self.idle)))); }
            // The lean wall's (plan § 40), likewise.
            if self.lean_old { kv.push(("lean".into(), Json::from("old"))); }
            // Rung 101's three, likewise.
            if self.airflow != AirflowFrom::Table { kv.push(("airflow".into(), Json::from(self.airflow.key()))); }
            if self.exhaust_lag { kv.push(("exhaust_lag".into(), b(true))); }
            if self.exhaust_tau != 1.0 { kv.push(("exhaust_tau".into(), Json::Float(self.exhaust_tau))); }
            // Rung 102's five, likewise.
            if let Some(m) = self.ab {
                kv.push(("ab".into(), Json::from(if m == AbMode::Lit { "lit" } else { "unlit" })));
            }
            if self.tt7 != AB_TT7 { kv.push(("Tt7".into(), Json::Float(self.tt7))); }
            if self.light_at != AB_LIGHT_AT { kv.push(("light_at".into(), Json::Float(self.light_at))); }
            if self.a8_ratio != 1.0 { kv.push(("a8_ratio".into(), Json::Float(self.a8_ratio))); }
            if self.a8_match { kv.push(("a8_match".into(), b(true))); }
        }
        // Rung 104's law, likewise; and rung 105's.
        self.turbine.push_json(&mut j);
        self.hp_vane.push_json(&mut j);
        j
    }

    /// Rung 102: the afterburner as the model takes it (`None` when not fitted). Range-checked by the precheck first.
    pub fn afterburner_fit(&self) -> Option<TwoShaftFit> {
        let throat = if self.a8_match { Throat::Match } else { Throat::Held(self.a8_ratio) };
        self.ab.map(|m| TwoShaftFit {
            ab: Afterburner { tt7: self.tt7, ..Afterburner::default() },
            throat,
            light_at: (m == AbMode::Lit).then_some(self.light_at),
        })
    }

    /// The Controls engine's design — rung 85's rig at its defaults, with this run's LP-turbine law (rung 104) and HP vane
    /// law (rung 105).
    pub fn blades(&self) -> BladeSettings {
        BladeSettings { turbine: self.turbine, hp_vane: self.hp_vane, ..BladeSettings::defaults() }
    }

    /// Rung 102: does this run light the afterburner?
    pub fn lit(&self) -> bool { self.ab == Some(AbMode::Lit) }

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
                          ("v_n_lo", &mut s.v_n_lo), ("b_max", &mut s.b_max), ("b_n_lo", &mut s.b_n_lo),
                          ("valve_phi", &mut s.valve_phi), ("valve_tau", &mut s.valve_tau),
                          ("valve_tau_close", &mut s.valve_tau_close), ("customer_share", &mut s.customer_share),
                          ("customer_amount", &mut s.customer_amount), ("cool_vane", &mut s.cool_vane),
                          ("cool_blade", &mut s.cool_blade), ("generator_share", &mut s.generator_share),
                          ("generator_power", &mut s.generator_power), ("metal_year", &mut s.metal_year),
                          ("metal_life", &mut s.metal_life), ("gov_tau", &mut s.gov_tau), ("gov_gain", &mut s.gov_gain),
                          ("exhaust_tau", &mut s.exhaust_tau),
                          ("T0", &mut s.t0), ("p0", &mut s.p0), ("M0", &mut s.m0),
                          ("Tt7", &mut s.tt7), ("light_at", &mut s.light_at), ("a8_ratio", &mut s.a8_ratio)] {
            num(k, into)?;
        }
        if let Some(t) = text("customer")? {
            s.customer = CustomerMode::ALL.into_iter().find(|m| m.key() == t)
                .ok_or_else(|| format!("unknown customer bleed {t:?} (\"off\", \"share\" or \"amount\")"))?;
        }
        if let Some(t) = text("generator")? {
            s.generator = GeneratorMode::ALL.into_iter().find(|m| m.key() == t)
                .ok_or_else(|| format!("unknown generator shaft {t:?} (\"off\", \"hp\" or \"lp\")"))?;
        }
        if let Some(t) = text("idle")? {
            s.idle = [IdleRule::Accept, IdleRule::Refuse, IdleRule::Old].into_iter().find(|r| idle_key(*r) == t)
                .ok_or_else(|| format!("unknown below-idle rule {t:?} (\"warn\", \"refuse\" or \"old\")"))?;
        }
        if let Some(t) = text("lean")? {
            s.lean_old = match t.as_str() {
                "run" => false,
                "old" => true,
                _ => return Err(format!("unknown lean wall {t:?} (\"run\" or \"old\")")),
            };
        }
        if let Some(t) = text("generator_by")? {
            s.generator_by = GeneratorBy::ALL.into_iter().find(|m| m.key() == t)
                .ok_or_else(|| format!("unknown generator setting {t:?} (\"share\" or \"power\")"))?;
        }
        for (k, into) in [("redline_on", &mut s.redline_on), ("gov_lag_on", &mut s.gov_lag_on),
                          ("accel_on", &mut s.accel_on), ("floor_on", &mut s.floor_on),
                          ("release_on", &mut s.release_on), ("metal_on", &mut s.metal_on), ("hold_cap", &mut s.hold_cap),
                          ("govern", &mut s.govern), ("table_clean", &mut s.table_clean),
                          ("corr_p", &mut s.corr_p), ("corr_t", &mut s.corr_t), ("low_grid", &mut s.low_grid),
                          ("formula_choked", &mut s.formula_choked),
                          ("exhaust_lag", &mut s.exhaust_lag),
                          ("a8_match", &mut s.a8_match)] {
            flag(k, into)?;
        }
        if let Some(t) = text("ab")? {
            s.ab = match t.as_str() {
                "none" => None,
                "unlit" => Some(AbMode::Unlit),
                "lit" => Some(AbMode::Lit),
                _ => return Err(format!("setting \"ab\" must be \"none\", \"unlit\" or \"lit\", got {t:?}")),
            };
        }
        if let Some(t) = text("floor_spool")? { s.floor_spool = spool(&t)?; }
        if let Some(t) = text("stator_spool")? { s.stator_spool = spool(&t)?; }
        if let Some(t) = text("floor_ref")? {
            s.floor_ref = FloorRef::ALL.into_iter().find(|r| r.key() == t)
                .ok_or_else(|| format!("unknown floor reference {t:?} (\"phi\" or \"incidence\")"))?;
        }
        if let Some(t) = text("bleed_mode")? {
            s.bleed_mode = BleedMode::ALL.into_iter().find(|m| m.key() == t)
                .ok_or_else(|| format!("unknown bleed mode {t:?} (\"schedule\" or \"valve\")"))?;
        }
        if let Some(t) = text("valve_ref")? {
            s.valve_ref = FloorRef::ALL.into_iter().find(|r| r.key() == t)
                .ok_or_else(|| format!("unknown valve reference {t:?} (\"phi\" or \"incidence\")"))?;
        }
        if let Some(t) = text("valve_speed")? {
            s.valve_lag_on = match t.as_str() {
                "instant" => false,
                "slow" => true,
                _ => return Err(format!("unknown valve speed {t:?} (\"instant\" or \"slow\")")),
            };
        }
        if let Some(t) = text("valve_close")? {
            s.valve_close_own = match t.as_str() {
                "same" => false,
                "own" => true,
                _ => return Err(format!("unknown valve closing {t:?} (\"same\" or \"own\")")),
            };
        }
        if let Some(t) = text("speed_law")? {
            s.speed_law = SpeedLaw::ALL.into_iter().find(|l| l.key() == t)
                .ok_or_else(|| format!("unknown speed law {t:?} (\"rate\" or \"prop\")"))?;
        }
        if let Some(t) = text("thrust_by")? {
            s.thrust_by = ThrustBy::ALL.into_iter().find(|r| r.key() == t)
                .ok_or_else(|| format!("unknown thrust reading {t:?} (\"true\", \"epr\" or \"n1\")"))?;
        }
        if let Some(t) = text("table_at")? {
            s.table_at = TableAt::ALL.into_iter().find(|a| a.key() == t)
                .ok_or_else(|| format!("unknown table place {t:?} (\"design\" or \"flight\")"))?;
        }
        if let Some(t) = text("speed_corr")? {
            s.speed_corr = SpeedCorr::ALL.into_iter().find(|a| a.key() == t)
                .ok_or_else(|| format!("unknown speed correction {t:?} (\"grid\", \"formula\" or \"none\")"))?;
        }
        if let Some(t) = text("airflow")? {
            s.airflow = AirflowFrom::ALL.into_iter().find(|a| a.key() == t)
                .ok_or_else(|| format!("unknown airflow source {t:?} (\"table\", \"grid\" or \"exhaust\")"))?;
        }
        if let Some(t) = text("turbine")? { s.turbine = TurbineLaw::from_key(&t)?; }
        s.hp_vane = HpVane::from_json(j, s.hp_vane)?;
        if let Some(t) = text("hold")? {
            s.hold = Hold::ALL.into_iter().find(|h| h.key() == t)
                .ok_or_else(|| format!("unknown hold {t:?} (\"tt4\", \"thrust\" or \"speed\")"))?;
        }
        if let Some(t) = text("metal_tech")? {
            s.metal_tech = TECH_KEYS.iter().position(|&k| k == t).ok_or_else(|| format!(
                "unknown cooling technology {t:?} (\"convection\", \"coated\", \"film\" or \"transpiration\")"))?;
        }
        if let Some(t) = text("lever")? {
            s.lever = LeverChoice::ALL.into_iter().find(|l| l.key() == t)
                .ok_or_else(|| format!("unknown lever {t:?} (\"none\", \"stator\", \"bleed\" or \"both\")"))?;
        }
        Ok(s)
    }

    /// The airflow lever(s) as the model's constructor takes them — for `Both`, rung 63's
    /// `{**stator, **bleed}` merge, which is how rung 86 builds its all-on cells.
    pub fn lever_arm(&self) -> LeverArm {
        let stator = || {
            let sch = StatorSchedule::new(self.v_max, self.v_n_lo);
            LeverArm::stator(match self.stator_spool {
                Spool::Lp => StatorArm::scheduled_lp(sch),
                Spool::Hp => StatorArm::scheduled_hp(sch),
            })
        };
        let bleed = || match self.bleed_mode {
            BleedMode::Schedule => LeverArm::scheduled(BleedSchedule::new(self.b_max, self.b_n_lo)),
            BleedMode::Valve => LeverArm::floored(self.valve()),
        };
        match self.lever {
            LeverChoice::None => LeverArm::default(),
            LeverChoice::Stator => stator(),
            LeverChoice::Bleed => bleed(),
            LeverChoice::Both => LeverArm::merged(&stator(), &bleed()),
        }
    }

    /// Rung 64's valve as the settings name it: `valve_phi` at the design stator setting, `b_max` its largest
    /// opening, watching the flow coefficient or (rung 87) the incidence.
    pub fn valve(&self) -> BleedLimiter {
        let v = BleedLimiter::new(self.valve_phi, self.b_max);
        let v = match self.valve_ref { FloorRef::Phi => v, FloorRef::Incidence => v.on_incidence() };
        match (self.valve_lag_on, self.valve_close_own) {
            (false, _) => v,
            (true, false) => v.lagged(self.valve_tau),
            (true, true) => v.lagged(self.valve_tau).closing(self.valve_tau_close),
        }
    }

    /// The bleed is the controlled valve (rung 64's machine is needed: rung 62's drops the valve silently).
    pub fn valve_on(&self) -> bool { self.lever.bleed() && self.bleed_mode == BleedMode::Valve }

    /// The controlled valve is on and SLOW (rung 88): rung 65's machine and march.
    pub fn valve_slow(&self) -> bool { self.valve_on() && self.valve_lag_on }

    /// The fuel the throttle schedules at march time `s` — rung 57's `_stator_march` ramp, spelled
    /// the same way, between the two endpoints' steady fuel flows.
    pub fn schedule(&self, mf_lo: f64, mf_hi: f64) -> impl Fn(f64) -> f64 {
        let r = self.ramp;
        move |s: f64| if s <= 0.0 { mf_lo } else if s >= r { mf_hi } else { mf_lo + (mf_hi - mf_lo) * (s / r) }
    }
}

/// A time-constant floor as the refusal states it: rounded UP to 0.001, so the value named is one the page accepts.
/// The below-idle rule's request key (plan § 39): `warn` — the run starts from the point and the page says so.
pub fn idle_key(r: IdleRule) -> &'static str {
    match r {
        IdleRule::Accept => "warn",
        IdleRule::Refuse => "refuse",
        IdleRule::Old => "old",
    }
}

fn tau_text(t: f64) -> String { format!("{}", ((t * 1000.0) - 1e-6).ceil() / 1000.0) }

/// A gain ceiling as the refusal states it: rounded DOWN to 0.1, for the same reason.
fn gain_text(g: f64) -> String { format!("{}", ((g * 10.0) + 1e-6).floor() / 10.0) }

/// The flight clause of a time-constant refusal: the design-flight text (`at_design`) unless the air is thinner.
fn thin_words(s: &ControlsSettings, at_design: &str) -> String {
    match s.density_ratio() < 1.0 {
        false => at_design.to_string(),
        true => format!(" here: the air the engine takes in is {:.0} % as dense as at the design flight, so the shafts answer the fuel more \
                         slowly and a fast commanded rate rides the rich edge", 100.0 * s.density_ratio()),
    }
}

/// The flight clause of a gain refusal: empty at or below the design flight's density.
fn dense_words(s: &ControlsSettings) -> String {
    match s.density_ratio() > 1.0 {
        false => String::new(),
        true => format!(" here (the air the engine takes in is {:.2} times as dense as at the design flight, so the shafts answer the fuel \
                         harder)", s.density_ratio()),
    }
}

/// The checks made before the engine is built — every one the model would otherwise ASSERT
/// (`r43_integrate_fuel`'s, the constructors'), in plain words.
pub fn controls_precheck(s: &ControlsSettings) -> Result<(), DoesNotRun> {
    let no = |m: &str| Err(DoesNotRun(m.to_string()));
    let named = [("rho", s.rho), ("from", s.from), ("to", s.to), ("ramp", s.ramp), ("settle", s.settle),
                 ("redline", s.redline), ("gov_lag", s.gov_lag), ("accel_margin", s.accel_margin),
                 ("floor_phi", s.floor_phi), ("tau_att", s.tau_att), ("tau_rel", s.tau_rel), ("v_max", s.v_max),
                 ("v_n_lo", s.v_n_lo), ("b_max", s.b_max), ("b_n_lo", s.b_n_lo), ("valve_phi", s.valve_phi),
                 ("valve_tau", s.valve_tau), ("valve_tau_close", s.valve_tau_close),
                 ("customer_share", s.customer_share), ("customer_amount", s.customer_amount),
                 ("cool_vane", s.cool_vane), ("cool_blade", s.cool_blade),
                 ("generator_share", s.generator_share), ("generator_power", s.generator_power),
                 ("gov_tau", s.gov_tau), ("gov_gain", s.gov_gain), ("exhaust_tau", s.exhaust_tau), ("T0", s.t0),
                 ("p0", s.p0), ("M0", s.m0)];
    if let Some((k, _)) = named.iter().find(|(_, v)| !v.is_finite()) {
        return Err(DoesNotRun(format!("Setting {k} is not a number.")));
    }
    if s.t0 <= 0.0 || s.p0 <= 0.0 {
        return no("The air's temperature and pressure must be above zero.");
    }
    s.hp_vane.check().map_err(DoesNotRun)?;
    if s.m0 <= 0.0 {
        return no("The flight Mach number must be above zero.");
    }
    // Rung 102: the afterburner's knobs (Fly it's ranges), its light-off on the march's grid, and the one control that
    // cannot run with it lit.
    if let Some(fit) = s.afterburner_fit() {
        ab_precheck(&fit.ab)?;
        let named = [("Tt7", s.tt7), ("light_at", s.light_at), ("a8_ratio", s.a8_ratio)];
        if let Some((k, _)) = named.iter().find(|(_, v)| !v.is_finite()) {
            return Err(DoesNotRun(format!("Setting {k} is not a number.")));
        }
        if !s.a8_match && !(1.0..=2.0).contains(&s.a8_ratio) {
            return no("The nozzle throat opens from its dry size (1) to twice it (2).");
        }
        if s.lit() {
            let k = s.light_at / DS;
            if !(s.light_at >= 0.0 && s.light_at < s.s_end()) || (k - k.round()).abs() > 1e-6 {
                return Err(DoesNotRun(format!(
                    "The afterburner lights at a moment of the run, on the march's {DS} step: from 0 to before the run \
                     ends ({:.2}).", s.s_end())));
            }
            if s.govern && s.hold == Hold::Thrust {
                return no("With the afterburner lit, the control cannot hold thrust with the main fuel: the afterburner \
                           holds the exhaust temperature, so more main fuel squeezes the airflow down while the \
                           afterburner simply burns less, and the thrust FALLS at that moment instead of rising. A \
                           control that pushes the fuel the way it expects would push it the wrong way. Hold the HP \
                           speed, or switch the afterburner to not lit.");
            }
        }
    }
    // Rung 101: the exhaust sensor's lag, where it acts — its reading is a march state, stepped by the march's RK4,
    // which is stable above 0.0072 τ at this step; the floor keeps a seventh of that edge.
    if s.exhaust_lag && !(s.exhaust_tau >= af::TAU_MIN && s.exhaust_tau <= af::TAU_MAX) {
        return Err(DoesNotRun(format!(
            "The exhaust sensor's lag must be between {} and {} shaft time constants: below that the march's step is \
             too coarse to follow the probe, and a probe slower than that misses the whole move.",
            af::TAU_MIN, af::TAU_MAX)));
    }
    // Rung 94: the metal cap binds a FLOATING burner, so it needs a hold, the metal read, and cooling (uncooled, the
    // 1985 metals allow about 990 K — below every opening throttle, so the cap would clamp both ends).
    if s.hold_cap {
        if s.hold == Hold::BurnerTemp {
            return no("The metal cap stops a burner that floats to hold thrust or speed. With the burner temperature itself \
                       held, lower the throttle instead.");
        }
        // Rung 95: a governed thrust or speed floats the burner along the move with no load at all, so the governor
        // lifts the generator's requirement (the anchor's § 1).
        if s.generator == GeneratorMode::Off && !s.govern {
            return no("The metal cap stops a burner that floats to carry the generator; with the generator off nothing \
                       floats. Switch the generator on, or govern the thrust or speed during the move.");
        }
        if !s.metal_on {
            return no("The metal cap needs the metal read: switch on Read the turbine metal.");
        }
        if s.cool_vane == 0.0 && s.cool_blade == 0.0 {
            return no("With no turbine cooling the metal allows only about 990 K (1985 metals), below these throttles, so \
                       the cap would clamp both ends of the run. Cool the turbine first.");
        }
    }
    // Rung 93: the metal readout's year and life, inside Gauntner's trend (the Cool the turbine view's own).
    if s.metal_on {
        if !(YEAR_MIN..=YEAR_MAX).contains(&s.metal_year) {
            return Err(DoesNotRun(format!(
                "The material year must be between {YEAR_MIN:.0} and {YEAR_MAX:.0}: Gauntner's relation is a straight-line \
                 trend fitted to the materials of his time, not a law to extrapolate far.")));
        }
        if !(LIFE_MIN..=LIFE_MAX).contains(&s.metal_life) {
            return no("The part life must be between 100 and 1 000 000 hours.");
        }
    }
    // Rung 92: the model's own bounds, refused in words before it asserts.
    let gen_half = |x: f64| (0.0..Demand::SHARE_MAX).contains(&x);
    if s.generator != GeneratorMode::Off && s.generator_by == GeneratorBy::Share && !gen_half(s.generator_share) {
        return no("The generator's share of the turbine's work must be at least 0 and below 0.5: half or more \
                   starves the compressor that turbine drives.");
    }
    if s.generator != GeneratorMode::Off && s.generator_by == GeneratorBy::Power && !gen_half(s.generator_power) {
        return no("The generator's power must be at least 0 and below 0.5 of the HP compressor's design power.");
    }
    // Rung 91: each cooling stream below a quarter of the air, and with a customer SHARE the two
    // together below half (a customer AMOUNT is checked by the model at every point it reaches).
    let quarter = |x: f64| (0.0..TurbineCooling::SHARE_MAX).contains(&x);
    if !quarter(s.cool_vane) || !quarter(s.cool_blade) {
        return no("Each turbine-cooling share must be at least 0 and below 0.25 of the HP compressor's air.");
    }
    if s.customer == CustomerMode::Share && !(s.customer_share + s.cool_vane + s.cool_blade < TurbineCooling::TOTAL_MAX) {
        return no("The aircraft's share and the turbine cooling together must take less than half of the HP \
                   compressor's air: half starves the burner.");
    }
    // Rung 90: the model's own bound, half the air (rung 42's) — refused in words before it asserts.
    let half = |x: f64| (0.0..CustomerBleed::C_MAX).contains(&x);
    if s.customer == CustomerMode::Share && !half(s.customer_share) {
        return no("The aircraft's share of the compressor's air must be at least 0 and below 0.5: taking half the \
                   air starves the burner.");
    }
    if s.customer == CustomerMode::Amount && !half(s.customer_amount) {
        return no("The aircraft's amount of air must be at least 0 and below 0.5 of the design airflow: half the \
                   design air is more than the burner can spare even at full power.");
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
    // Rung 95: the governor rides rung 43's plain march; each lagged route is its own marcher (another model).
    if s.govern {
        if s.gov_lag_on {
            return no("The governor runs with the instant limiters only: the temperature limiter's lag carries its cut as \
                       a state on its own march, and a governor on that march is a different model. Switch the lag off.");
        }
        if s.release_on {
            return no("The governor runs with the instant limiters only: the realistic release carries its cut as a \
                       state on its own march, and a governor on that march is a different model. Switch the release off.");
        }
        if s.valve_slow() {
            return no("The governor runs with the instant valve only: a slow valve is a state on its own march, and a \
                       governor on that march is a different model. Make the valve instant.");
        }
        // Plan § 27: both limits were set by the step check AT the design flight. The shafts answer the governor's fuel
        // harder in denser air, so the gain limit falls by the density ratio there; in thinner air the shafts are slower,
        // a fast commanded rate rides the rich edge, and the time constant's floor rises by it. Both 1 at the design flight.
        let (tau_min, gain_max) = (TAU_MIN / s.density_ratio().min(1.0), GAIN_MAX / s.density_ratio().max(1.0));
        if s.hold == Hold::HpSpeed && s.speed_law == SpeedLaw::Rate && !(s.gov_tau >= tau_min - 1e-12) {
            return Err(DoesNotRun(format!(
                "The speed governor's time constant must be at least {} spool time constants{}: measured, one step \
                 moves the flow coefficients by about 0.001 against a finer step.", tau_text(tau_min),
                thin_words(s, " (two march steps)"))));
        }
        if s.hold == Hold::HpSpeed && s.speed_law == SpeedLaw::Prop && !(s.gov_gain > 0.0 && s.gov_gain <= gain_max) {
            return Err(DoesNotRun(format!(
                "The speed governor's gain must be above 0 and at most {}{}: a larger gain is worked out too \
                 coarsely at this march's step.", gain_text(gain_max), dense_words(s))));
        }
        // Rung 96: an inferred thrust — the cap would saturate the control's table (a capped end lies below its span),
        // and the LP laws' knobs have their own step check.
        if s.inferred() {
            if s.hold_cap {
                return no("The metal cap is not built beside a control that infers thrust: a capped end's thrust falls \
                           below the control's table, which then cannot turn it back into a reading. Read the thrust \
                           itself, or switch the cap off.");
            }
            let (tau_min, gain_max) = (TAU_MIN_LP / s.density_ratio().min(1.0), GAIN_MAX_LP / s.density_ratio().max(1.0));
            if s.thrust_by == ThrustBy::N1 && s.speed_law == SpeedLaw::Rate && !(s.gov_tau >= tau_min - 1e-12) {
                return Err(DoesNotRun(format!(
                    "The fan-speed governor's time constant must be at least {} spool time constants{}: a faster one is \
                     worked out too coarsely at this march's step.", tau_text(tau_min), thin_words(s, ""))));
            }
            if s.thrust_by == ThrustBy::N1 && s.speed_law == SpeedLaw::Prop && !(s.gov_gain > 0.0 && s.gov_gain <= gain_max) {
                return Err(DoesNotRun(format!(
                    "The fan-speed governor's gain must be above 0 and at most {}{}: a larger gain is worked out too \
                     coarsely at this march's step.", gain_text(gain_max), dense_words(s))));
            }
        }
    }
    if s.accel_on && s.accel_margin < 0.0 {
        return no("The acceleration schedule's margin is measured above the steady line, so it cannot be negative.");
    }
    if s.floor_on && s.floor_phi <= 0.0 {
        return no("The stall floor is a flow coefficient, so it must be above zero.");
    }
    // Both levers on runs BOTH lists of checks.
    if s.lever.stator() && !(s.v_n_lo < 1.0) {
        return no("The stators must be back at their design setting by the design speed (1.0), so the \
                   speed they start opening from must be below 1.");
    }
    if s.valve_on() {
        if !(s.b_max > 0.0 && s.b_max < 0.5) {
            return no("The valve's largest opening must be above 0 (a valve that cannot open holds nothing) and below \
                       0.5 (bleeding half the air starves the core).");
        }
        if s.valve_phi <= 0.0 {
            return no("The valve holds a flow coefficient, so its target must be above zero.");
        }
        if s.valve_lag_on {
            // Rung 65's march asserts `ds/τ ≤ 2` (beyond it the explicit step is unstable and the result
            // LOOKS physical — rung 65 § 0). Measured on this view's engine (plan § 17): at half a step the
            // LP flow coefficient is 7e-4–1.1e-3 off a 4× finer step; at one step ≤ 9e-5 on the default move,
            // the other routes' ~1e-4 (§ 12.10). So one step, as the view's other lags.
            if !(s.valve_tau >= DS - 1e-12) {
                return Err(DoesNotRun(format!(
                    "The slow valve's time constant must be at least one march step ({DS} spool time constants): \
                     a faster valve than that is worked out too coarsely at this step — measured, half a step moves \
                     the flow coefficient by up to 0.001 against a finer step. Switch the valve to instant instead.")));
            }
            // Rung 89: the same floor on the CLOSING constant — the march's bound reads the smaller one.
            if s.valve_close_own && !(s.valve_tau_close >= DS - 1e-12) {
                return Err(DoesNotRun(format!(
                    "The valve's closing time constant must be at least one march step ({DS} spool time constants), \
                     as its opening one: a faster close than that is worked out too coarsely at this step. Let it close \
                     at the same speed instead.")));
            }
            if s.gov_lag_on || s.release_on {
                return no("A slow valve cannot run beside the temperature limiter's lag or the realistic release: a \
                           lagging valve beside a lagging fuel cut is two lags on two levers (rung 66's two-lag \
                           cascade), a different model from this one. Make the valve instant, or switch the lag off.");
            }
        }
    } else if s.lever.bleed() {
        if !(s.b_n_lo < 1.0) {
            return no("The bleed valve must be shut by the design speed (1.0), so the speed it starts closing \
                       from must be below 1.");
        }
        if !(0.0..0.5).contains(&s.b_max) {
            return no("The bleed fraction must be at least 0 and below 0.5: bleeding half the air starves the \
                       core.");
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
    /// Rung 94: what each end held, when it held thrust or HP speed with the generator on.
    pub hold_start: Option<HoldInfo>,
    pub hold_end: Option<HoldInfo>,
    /// Rung 95: the governor, when the move is governed.
    pub governor: Option<Governor>,
    /// Rung 98: the carry from the design flight, and rung 96's thrust table as made THERE (before the carry) — when
    /// the tables are made at the design flight and flown elsewhere.
    pub carry: Option<Carry>,
    pub design_table: Option<inf::Table>,
    /// Rung 99: where the tables came from under a carry — rung 98's, two grid slices, or the formula.
    pub tables: Option<Tables>,
}

/// Rung 94: one end's hold — the throttle set, the target (thrust N per kg/s of design airflow, or HP speed), what
/// the end achieves, and whether the metal cap stopped the burner short of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoldInfo {
    pub set: f64,
    pub target: f64,
    pub achieved: f64,
    pub capped: bool,
}

/// Rung 94: the burner temperatures a hold searches, K.
pub const HOLD_LO: f64 = 600.0;
pub const HOLD_HI: f64 = 2000.0;

/// Build the machine and solve both endpoints — or say why an endpoint has no steady point.
pub fn build(s: &ControlsSettings) -> Result<ControlsSolver, DoesNotRun> {
    controls_precheck(s)?;
    let (ml, mh) = control_maps(CONTROL_SHAPES[s.shape]).expect("CONTROL_SHAPES lists only buildable shapes");
    // Plan § 27: the hardware is captured at the DESIGN flight (every constructor below takes `fd`); every solve and
    // the march run at the flight FLOWN (`fl`). Rung 96's control table is built at `fl` too — a table made at one
    // flight and flown at another is that rung's open seam, not this knob.
    let (fd, fl) = (s.design_flight(), s.flight());
    // A slow valve needs rung 65's machine (rung 64's refuses a lag); an instant one rung 64's (rung 62's
    // drops the valve silently).
    let builder = if s.valve_slow() { build_lagged_bleed } else if s.valve_on() { build_limited_bleed }
                  else { build_scheduled_bleed };
    // Rung 90: the aircraft's demand rides on the design engine, so every sibling the model builds keeps it.
    let design = match s.customer_bleed() {
        Some(cb) => s.blades().design().with_customer_bleed(cb),
        None => s.blades().design(),
    };
    // Rung 91: the turbine cooling rides there too, for the same reason.
    let design = match s.turbine_cooling() {
        Some(tc) => design.with_turbine_cooling(tc),
        None => design,
    };
    // Rung 102: the afterburner is hardware on the design engine too — every sibling built below carries it.
    let design = match s.afterburner_fit() {
        Some(fit) => design.with_afterburner(fit),
        None => design,
    };
    // Rung 92: and the generator load. A load the engine cannot carry slows a spool until its compressor
    // leaves the map, and the equilibrium search then reports only the map — so when an endpoint fails with
    // the generator on, the page asks the model two questions before naming a cause: (i) with no airflow
    // lever the steady engine IS this machine's equilibrium (rung 92's plant gate), and its own refusal is
    // used when it gives one; (ii) otherwise the same machine WITHOUT the generator is solved at that
    // throttle, and only if it has a steady point is the load named as what removes it. Crash map: 76 of
    // 129 such refusals ran with the generator off (plan § 21).
    let unloaded = s.shaft_offtake().is_some().then(|| design.clone());
    let design = match s.shaft_offtake() {
        Some(so) => design.with_shaft_offtake(so),
        None => design,
    };
    // The customer bleed's twin of question (ii): the same machine — cooling, afterburner and generator kept — with
    // only the aircraft's air left in the engine. Asked only if the generator's question did not name the load.
    let unbled = s.customer_bleed().is_some().then(|| {
        let d = s.blades().design();
        let d = match s.turbine_cooling() { Some(tc) => d.with_turbine_cooling(tc), None => d };
        let d = match s.afterburner_fit() { Some(fit) => d.with_afterburner(fit), None => d };
        match s.shaft_offtake() { Some(so) => d.with_shaft_offtake(so), None => d }
    });
    let gen_check = (s.shaft_offtake().is_some() && s.lever == LeverChoice::None).then(|| design.clone());
    // Every machine this view builds — the flown one and each sibling a refusal or a hold re-solves — holds the view's
    // below-idle rule (plan §§ 38–39): by default it keeps a point below idle, whichever start found it, and warns.
    let make = |d: TwoSpoolEngine| -> ScheduledStatorCore {
        let c = match builder(d, fd, MDOT_DESIGN, Some(ml), Some(mh), s.rho, &s.lever_arm()) {
            ScheduledStatorTransient::Full(c) => c,
            ScheduledStatorTransient::Degenerate(_) => unreachable!("the Controls engine has both shafts"),
        };
        c.fuel.inner.inner.base.idle_rule.set(s.idle);
        c
    };
    let core = make(design);
    let steady = |which: &str, tt4: f64| -> Result<Instant2, DoesNotRun> {
        core.fuel.inner.try_equilibrium(&fl, tt4, None).map(|(i, _, _)| i).map_err(|e| {
            let mut m = e.0;
            if let Some(de) = gen_check.as_ref().filter(|_| !m.contains("rung-92:")) {
                let st = ShaftOfftakeMatcher::new(de.clone(), fd, MDOT_DESIGN, ml, mh, 0.0);
                // Rung 103: the steady check re-solves an unchoked throttle, as the march's own equilibrium does
                // (rung 97) — so its answer is no longer the pin's there.
                st.core.base.below_choke.set(true);
                if let Err(e2) = st.try_match_point(&fl, tt4) {
                    if e2.0.contains("rung-92:") { m = e2.0; }
                }
            }
            let mut words = format!("At the {which} throttle ({tt4:.0} K): {}", endpoint_words(&m));
            if let Some(de) = unloaded.as_ref().filter(|_| !m.contains("rung-92:")) {
                let bare = make(de.clone());
                if bare.fuel.inner.try_equilibrium(&fl, tt4, None).is_ok() {
                    words += " With the generator off this throttle does have a steady point, so the generator's \
                              load is what removes it: ask for less power, or raise the throttle.";
                }
            }
            // Only on the "off the maps" refusal: the others already name their cause (the bleed's own "takes more
            // than half" among them). Crash map: the one such refusal with the customer bleed on (plan § 38).
            let named = words.contains("With the generator off");
            let off_maps = m.contains("does not bracket") || m.contains("off-map compressor trial");
            if let Some(de) = unbled.as_ref().filter(|_| !named && off_maps) {
                let bare = make(de.clone());
                if bare.fuel.inner.try_equilibrium(&fl, tt4, None).is_ok() {
                    words += " With the customer bleed off this throttle does have a steady point, so the air the \
                              aircraft takes is what removes it: ask for less, or raise the throttle.";
                }
            }
            DoesNotRun(words)
        })
    };
    // Rung 94: with the generator on, an end that holds thrust or HP speed lands where the loaded machine gives what
    // the same machine WITHOUT the generator gives at the set throttle; the burner temperature floats, and the metal
    // cap (on its switch) stops it. Holding the burner temperature, or with no load, is rung 92's end exactly.
    let held = |i: &Instant2| match s.hold {
        Hold::Thrust => ControlsSolver::thrust(i),
        Hold::HpSpeed => i.nu_hp,
        Hold::BurnerTemp => i.tt4,
    };
    // Rung 96: calibrated on the generator-off engine, the control's table is that engine's, between the two set
    // throttles (rung 94's target ends — its end rows give the targets exactly); each end is then where THIS engine's
    // steady reading equals the table's.
    // Rung 98: with the tables made at the design flight and flown elsewhere, every table is built THERE and carried
    // here (below), and an inferred thrust's ends are calibrated on the carried table instead.
    // Rung 99: and where they come from — rung 98's one design-flight table, two grid slices blended, or the design
    // table with its thrust recomputed for this flight speed. `carry` stays rung 98's (the air against the design's).
    // Rung 100: two slow slices in the grid, the formula read at the reading (each with its hidden reduce setting).
    let tables = s.carried().then(|| Tables::new(&core.fuel, &fd, &fl, s.speed_corr, s.corr_p, s.corr_t, s.low_grid,
                                                 s.formula_choked, s.airflow));
    let carry = s.carried().then(|| Carry::new(&core.fuel, &fd, &fl, s.corr_p, s.corr_t));
    let clean = match (s.calibrated() && tables.is_none(), unloaded.as_ref()) {
        (true, Some(de)) => {
            let bare = make(de.clone());
            let end_of = |which: &str, t: f64| bare.fuel.inner.try_equilibrium(&fl, t, None).map(|(i, _, _)| i)
                .map_err(|e| DoesNotRun(format!(
                    "At the {which} throttle ({t:.0} K): with the generator off this throttle has no steady point, so the \
                     control has no table to be calibrated on ({}).", endpoint_words(&e.0))));
            let (b0, b1) = (end_of("starting", s.from)?, end_of("final", s.to)?);
            let table = inf::try_table(&bare.fuel, &fl, &b0, &b1).map_err(|e| DoesNotRun(format!(
                "The control's table, calibrated on the engine with the generator off: {}", endpoint_words(&e.0))))?;
            Some((table, b0, b1))
        }
        _ => None,
    };
    let hold_end = |which: &str, t_set: f64| -> Result<(Instant2, Option<HoldInfo>), DoesNotRun> {
        // Rung 95: governed with the cap and no load, the end is `T_set` itself, capped by rung 94's bisection if its
        // metal is over — so the run starts on a point the metal leg agrees with.
        if s.metal_leg() && unloaded.is_none() {
            let i = steady(which, t_set)?;
            let target = held(&i);
            if s.metal_at(&i).is_none_or(|m| m.over() <= 0.0) {
                return Ok((i, Some(HoldInfo { set: t_set, target, achieved: target, capped: false })));
            }
            let ok = |t: f64| core.fuel.inner.try_equilibrium(&fl, t, None).map(|_| 0.0);
            let (floor, _) = bh::floor_of(&ok, HOLD_LO, t_set).ok_or_else(|| DoesNotRun(format!(
                "At the {which} throttle ({t_set:.0} K): the engine has no steady point below it to cap the metal at.")))?;
            let excess = |t: f64| core.fuel.inner.try_equilibrium(&fl, t, None).map(|(i, _, _)| {
                s.metal_at(&i).map_or(f64::NEG_INFINITY, |m| m.over())
            });
            let t = bh::try_metal_cap(&excess, floor, t_set).map_err(|m| DoesNotRun(format!(
                "At the {which} throttle ({t_set:.0} K): {m}")))?.unwrap_or(t_set);
            let i = steady(which, t)?;
            let achieved = held(&i);
            return Ok((i, Some(HoldInfo { set: t_set, target, achieved, capped: true })));
        }
        if let Some((table, b0, b1)) = clean.as_ref() {
            let r = s.law().reading().expect("a calibrated end infers thrust");
            let bi = if which == "starting" { b0 } else { b1 };
            let (target, want) = (ControlsSolver::thrust(bi), table.read(r, bi));
            let q_at = |t: f64| core.fuel.inner.try_equilibrium(&fl, t, None).map(|(i, _, _)| table.read(r, &i));
            let root = bh::try_hold_root(&q_at, want, HOLD_LO, HOLD_HI).map_err(|e| DoesNotRun(format!(
                "At the {which} throttle ({t_set:.0} K), the control holding the {} its table gives for the asked thrust: {}",
                inf::reading_words(r), inf::refusal_words(&e, r, want))))?;
            let i = steady(which, root.t)?;
            let achieved = held(&i);
            return Ok((i, Some(HoldInfo { set: t_set, target, achieved, capped: false })));
        }
        let Some(de) = unloaded.as_ref().filter(|_| s.hold != Hold::BurnerTemp) else {
            return Ok((steady(which, t_set)?, None));
        };
        let bare = make(de.clone());
        let target = held(&bare.fuel.inner.try_equilibrium(&fl, t_set, None).map_err(|e| DoesNotRun(format!(
            "At the {which} throttle ({t_set:.0} K): with the generator off this throttle has no steady point, so it \
             sets no {} to hold ({})", if s.hold == Hold::Thrust { "thrust" } else { "speed" }, endpoint_words(&e.0))))?.0);
        let q_at = |t: f64| core.fuel.inner.try_equilibrium(&fl, t, None).map(|(i, _, _)| held(&i));
        let root = bh::try_hold_root(&q_at, target, HOLD_LO, HOLD_HI).map_err(|r| DoesNotRun(format!(
            "At the {which} throttle ({t_set:.0} K), holding {}: {}", if s.hold == Hold::Thrust { "thrust" } else { "HP speed" },
            bh::refusal_words(&r, s.hold, target))))?;
        let (mut t, mut capped) = (root.t, false);
        if s.hold_cap {
            let excess = |t: f64| core.fuel.inner.try_equilibrium(&fl, t, None).map(|(i, _, _)| {
                s.metal_at(&i).map_or(f64::NEG_INFINITY, |m| m.over())
            });
            match bh::try_metal_cap(&excess, root.t_floor, root.t).map_err(|m| DoesNotRun(format!(
                "At the {which} throttle ({t_set:.0} K): {m}")))? {
                Some(tc) => { t = tc; capped = true; }
                None => {}
            }
        }
        let i = steady(which, t)?;
        let achieved = held(&i);
        Ok((i, Some(HoldInfo { set: t_set, target, achieved, capped })))
    };
    // Rung 98: with the clean table under a carry, each end's target is the generator-off engine's thrust at the set
    // throttle (rung 96's `b0`/`b1`), and rung 94's thrust-hold root is NOT run: the ends are calibrated below.
    let carried_clean = tables.is_some() && s.inferred() && s.calibrated() && unloaded.is_some();
    let nominal = match carried_clean {
        true => None,
        false => Some((hold_end("starting", s.from)?, hold_end("final", s.to)?)),
    };
    // Rung 98: an inferred thrust under a carried table — rung 96's table made at the DESIGN flight on its own machine
    // (the generator-off one between the set throttles, else this one between the ends just found), carried here, and
    // each end CALIBRATED on it: where this engine's steady reading equals the carried table's reading at the end's
    // target (rung 96's target, unchanged). The thrust there minus the target is the steady miss.
    let (start, end, hold_start, hold_end_info, design_table, carried_infer) = match (tables.as_ref(), s.inferred()) {
        (Some(tb), true) => {
            let r = s.law().reading().expect("an inferred law has a reading");
            let made = |m: &ScheduledStatorCore, lo: f64, hi: f64| tb.thrust_table(&m.fuel, lo, hi, r).map_err(DoesNotRun);
            let ((table, dt), targets) = match (carried_clean, unloaded.as_ref()) {
                (true, Some(de)) => {
                    let bare = make(de.clone());
                    let target_of = |which: &str, t: f64| bare.fuel.inner.try_equilibrium(&fl, t, None)
                        .map(|(i, _, _)| ControlsSolver::thrust(&i)).map_err(|e| DoesNotRun(format!(
                            "At the {which} throttle ({t:.0} K): with the generator off this throttle has no steady point, so \
                             the control has no table to be calibrated on ({}).", endpoint_words(&e.0))));
                    let targets = (target_of("starting", s.from)?, target_of("final", s.to)?);
                    (made(&bare, s.from, s.to)?, targets)
                }
                _ => {
                    let ((a, ha), (b, hb)) = nominal.as_ref().expect("the nominal ends exist unless the clean table is carried");
                    let target = |i: &Instant2, h: Option<HoldInfo>| h.map_or_else(|| held(i), |h| h.target);
                    (made(&core, a.tt4, b.tt4)?, (target(a, *ha), target(b, *hb)))
                }
            };
            let calibrate = |which: &str, t_set: f64, target: f64| -> Result<(Instant2, Option<HoldInfo>), DoesNotRun> {
                // Rung 101: the exhaust sensor's belief depends on the steady point itself (its exhaust temperature and
                // fuel), so the end is where the steady BELIEF equals the target — not a reading worked out beforehand.
                // The search starts AT the set throttle and steps out from it: rung 94's root walks up from its floor
                // (600 K), and the fan-speed formula cannot be read that far below its rows (the first score refused 19
                // cells so, `docs/rung101-spec.md`), where the reading alone, read without the sensor, always can.
                if table.sensed() {
                    let f_at = |t: f64| core.fuel.inner.try_equilibrium(&fl, t, None)
                        .and_then(|(i, _, _)| table.try_belief(&core.fuel, r, &i, None));
                    let t = sensed_end_root(&f_at, target, t_set).map_err(|m| DoesNotRun(format!(
                        "At the {which} throttle ({t_set:.0} K), the control holding the thrust its exhaust sensor \
                         believes: {m}")))?;
                    let i = steady(which, t)?;
                    let achieved = held(&i);
                    return Ok((i, Some(HoldInfo { set: t_set, target, achieved, capped: false })));
                }
                let want = table.try_reading_at(&core.fuel, r, target).map_err(|e| DoesNotRun(format!(
                    "At the {which} throttle ({t_set:.0} K): {}", e.0)))?;
                let q_at = |t: f64| core.fuel.inner.try_equilibrium(&fl, t, None).map(|(i, _, _)| table.read(r, &i));
                let root = bh::try_hold_root(&q_at, want, HOLD_LO, HOLD_HI).map_err(|e| DoesNotRun(format!(
                    "At the {which} throttle ({t_set:.0} K), the control holding the {} its design-flight table gives for \
                     the asked thrust: {}", inf::reading_words(r), inf::refusal_words(&e, r, want))))?;
                let i = steady(which, root.t)?;
                let achieved = held(&i);
                Ok((i, Some(HoldInfo { set: t_set, target, achieved, capped: false })))
            };
            let (s0, h0) = calibrate("starting", s.from, targets.0)?;
            let (s1, h1) = calibrate("final", s.to, targets.1)?;
            (s0, s1, h0, h1, dt, Some(table))
        }
        _ => {
            let ((a, ha), (b, hb)) = nominal.expect("the nominal ends exist unless the clean table is carried");
            (a, b, ha, hb, None, None)
        }
    };
    let (mf_lo, mf_hi) = (start.close.f * start.close.mdot_air, end.close.f * end.close.mdot_air);
    // Plan § 40: rung 43's lean wall (0.004) is a search bound, and a steady end can burn leaner (down to ~0.0001 at
    // Mach 1.6–2 near the lowest throttle that builds): the march's first closure could not reach its own start. The
    // wall drops to half the leaner end's mixture there; a run whose ends both burn at 0.008 or richer keeps 0.004.
    if !s.lean_old {
        let wall = 0.5 * start.close.f.min(end.close.f);
        if wall < FuelTransientCore::F_FLOOR {
            core.fuel.inner.inner.base.lean_wall.set(Some(wall));
        }
    }
    let accel = match s.accel_on {
        false => None,
        true => {
            let (lo, hi) = if s.from <= s.to { (s.from, s.to) } else { (s.to, s.from) };
            match tables.as_ref() {
                None => Some(accel_table(&core, &fl, lo, hi, s.accel_margin)?),
                // Rung 98: made at the design flight, carried (`Wf/pt3 × √θ`); rung 99: or at two grid speeds, blended.
                Some(tb) => Some(tb.accel(&core.fuel, lo, hi, ACCEL_ROWS, s.accel_margin).map_err(DoesNotRun)?),
            }
        }
    };
    // Rung 95: the governor's targets are rung 94's — `T_set` for the burner temperature; the held thrust or speed the
    // end was asked for (with the generator or the cap), else the end's own.
    let governor = match s.govern {
        false => None,
        true => {
            let target = |i: &Instant2, h: Option<HoldInfo>, t_set: f64| match s.hold {
                Hold::BurnerTemp => t_set,
                _ => h.map_or_else(|| held(i), |h| h.target),
            };
            let law = s.law();
            let table = match (law, tables.as_ref()) {
                (Law::SpeedProp { .. }, None) => gv::try_steady_table(&core.fuel, &fl, &start, &end).map_err(|e| DoesNotRun(
                    format!("The speed governor's steady table has a throttle with no steady point between the ends: {}",
                            endpoint_words(&e.0))))?,
                // Rung 98: made at the design flight between the ends' burner temperatures, carried.
                // Rung 99: or at two grid speeds, blended.
                (Law::SpeedProp { .. }, Some(tb)) => tb.steady_table(&core.fuel, start.tt4, end.tt4).map_err(DoesNotRun)?,
                _ => Vec::new(),
            };
            // Rung 96: the control's table — calibrated (above), or on this machine between the run's own ends; rung 98's
            // carried one when the tables are made at the design flight.
            let infer = match (law.reading(), clean, carried_infer) {
                (None, _, _) => None,
                (Some(_), _, Some(t)) => Some(t),
                (Some(_), Some((t, _, _)), None) => Some(t),
                (Some(_), None, None) => Some(inf::try_table(&core.fuel, &fl, &start, &end).map_err(|e| DoesNotRun(format!(
                    "The control's table has a throttle with no steady point between the ends: {}",
                    endpoint_words(&e.0))))?),
            };
            // Rung 101: a lagged exhaust sensor's reading is the march's third state, started at the steady start's.
            let lag = (s.exhaust_lag && infer.as_ref().is_some_and(inf::Table::sensed))
                .then(|| SensorLag { tau: s.exhaust_tau, t0: start.tt5 });
            Some(Governor { law, lo: target(&start, hold_start, s.from), hi: target(&end, hold_end_info, s.to),
                            ramp: s.ramp, mf_lo, mf_hi, table, infer, lag })
        }
    };
    Ok(ControlsSolver { settings: *s, core, flight: fl, start, end, mf_lo, mf_hi, accel, hold_start, hold_end: hold_end_info,
                        governor, carry, design_table, tables })
}

/// Rung 99: how the tables were corrected for flight speed — the setting, the grid slices read (Mach), the weight on
/// the second, whether the trend is carried past the grid, and how the pressure-ratio gap is split.
fn speed_json(s: &ControlsSettings, t: Option<&Tables>) -> Json {
    let epr = s.thrust_by == ThrustBy::Epr;
    let split = match t {
        Some(Tables::Design(_)) if epr && s.corr_p => "ram_nozzle",
        Some(Tables::Formula { .. }) if epr && (s.corr_p || !s.formula_choked) => "airflow_rest",
        _ => "whole",
    };
    let (slices, w) = match t {
        Some(Tables::Grid { a, b, w }) => match b {
            Some(b) => (vec![Json::Float(a.m), Json::Float(b.m)], Json::Float(*w)),
            None => (vec![Json::Float(a.m)], Json::Null),
        },
        Some(Tables::Design(_)) if s.speed_corr == SpeedCorr::Grid => (vec![Json::Float(ma::DESIGN_MACH)], Json::Null),
        _ => (Vec::new(), Json::Null),
    };
    let extended = matches!(t, Some(Tables::Grid { b: Some(_), w, .. }) if *w < 0.0 || *w > 1.0);
    let mut j = jobj! { "corr" => s.speed_corr.key(), "slices" => Json::List(slices), "weight" => w,
                        "extended" => Json::Int(extended as i64), "split" => split };
    // Rung 101: where the formula got the air swallowed, written only off rung 100's table (and the lag with it).
    if let (Some(Tables::Formula { air, choked: false, .. }), Json::Obj(kv)) = (t, &mut j) {
        if *air != AirflowFrom::Table {
            kv.push(("airflow".into(), Json::from(air.key())));
            if *air == AirflowFrom::Exhaust && s.exhaust_lag {
                kv.push(("exhaust_tau".into(), Json::Float(s.exhaust_tau)));
            }
        }
    }
    j
}

/// Plain words for an endpoint's failed steady solve — the four messages the crash map found
/// (plan § 12.10), each driven by a test.
pub fn endpoint_words(message: &str) -> String {
    // Plan § 37: the equilibrium's low start found the one root, and it is below idle (first: the
    // full-speed refusal's own words come before it in the message).
    let why = if message.contains("below idle") {
        "at this throttle the engine is below idle: it makes more drag than thrust. Raise the throttle (at the \
         design flight the first one that runs is about 480 K on every map; flying faster raises it — about 580 K \
         at Mach 1.6, 690 K at Mach 2, 960 K at Mach 2.6 — and so does warmer air (on a standard sea-level day about \
         550–560 K) or air taken off the engine: the bleed lever adds about 40–120 K, more than 140 K at Mach 2.6, \
         and a 5 % customer bleed about 30–90 K)"
    } else if message.contains("customer bleed takes") {
        "the aircraft's customer bleed would take more than half of the HP compressor's air here, which starves the \
         burner. A fixed amount of air is a larger share at low power: raise the throttle, or ask for less air"
    } else if message.contains("turbine cooling") && message.contains("together take") {
        "the aircraft's air and the turbine cooling together would take more than half of the HP compressor's air \
         here, which starves the burner. A fixed amount of air is a larger share at low power: raise the throttle, \
         ask for less air, or cool the turbines less"
    } else if message.contains("rung-92:") {
        "the aircraft's generator would take half or more of its turbine's work here, which starves the compressor \
         that turbine drives. A fixed power is a larger share at low power, and the less work the compressor gets \
         the less air the engine takes: raise the throttle, or ask for less power"
    } else if message.contains("did not converge") {
        // Plan § 27: off the design flight, all 14 such refusals in the crash map had a lever AND a generator load.
        // Plan § 37: the design-flight case (a schedule doing all its travel just below design speed) runs since the
        // low start; the 6 left in its crash map are all a lever with a generator on the LP shaft, in other air.
        "the search for the steady shaft speeds does not settle. Measured only with an airflow lever on and a \
         generator load, away from the design flight: ask for less power, or switch the lever off"
    } else if message.contains("rung-97:") {
        // Rung 97: the unchoked nozzle cannot pass the flow even with the LP turbine doing almost no work.
        "at this low a throttle the gas leaving the turbines has too little pressure to be pushed out against the air \
         outside, even with the low-pressure turbine taking almost no work from it. Raise the throttle"
    } else if message.contains("nozzle back-pressure") {
        // Plan § 27: met only off the design flight (crash map: 3 of 1 400 requests, 600–800 K, every one in air above 250 K).
        "at this low a throttle the gas leaving the turbine has less pressure than the air outside, so the nozzle \
         cannot push it out (met only at very low throttles, 600–800 K, in air warmer than at the design flight, 250 K). Raise the \
         throttle"
    } else if message.contains("does not bracket") || message.contains("off-map compressor trial") {
        // Plan § 27: away from the design flight the TOP of the range can be off the maps too, so the words do not say
        // which way to move. Plan § 37: a throttle below idle has its own words now (above); in the crash map the low
        // ones left here (3 of 1 200, 575–598 K) all carried a load or the bleed, the search slowing the LP shaft to ~0.
        "the compressors would have to run off the part of their maps the model covers. Mostly this is a low throttle \
         with a fixed load or the bleed valve open (the search slows the low-pressure shaft almost to a stop), or a \
         large fixed load or amount of air in thin air, or a lever whose schedule does nearly all its travel just \
         below the design speed. Move this throttle toward the middle of its range, ask for less load, or start the \
         lever's schedule lower"
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
            // Where the scheduled fuel has no operating point, decide the limiters from the most
            // fuel that has one, instead of stopping (2026-10-10). The model's default since
            // 2026-10-14 (plan § 14.6); set explicitly so the view does not depend on a thread's
            // `CeilingDefault`.
            below_ceiling: true,
        }
    }

    /// The march — `integrate_fuel` itself, from the starting equilibrium.
    pub fn march(&self) -> Vec<FuelPoint> { self.march_marked().0 }

    /// The march and, when governed (rung 95), what the governor did at each point (empty otherwise).
    pub fn march_marked(&self) -> (Vec<FuelPoint>, Vec<GovMark>) {
        let nu0 = (self.start.nu_lp, self.start.nu_hp);
        let out = if let Some(g) = &self.governor {
            let over = |i: &Instant2| self.metal_over(i);
            let metal: Option<&dyn Fn(&Instant2) -> f64> = if self.settings.metal_leg() { Some(&over) } else { None };
            gv::march(&self.core.fuel, &self.flight, g, &self.limiters(), metal, nu0, self.settings.s_end(), DS)
        } else {
            let sched = self.settings.schedule(self.mf_lo, self.mf_hi);
            (self.core.fuel.integrate_fuel(&self.flight, sched, nu0, self.settings.s_end(), DS, &self.limiters()),
             Vec::new())
        };
        // RUNG 102: the march leaves the afterburner as lit at its last evaluation; every later reader sets its own.
        self.core.fuel.inner.inner.base.ab_w.set(0.0);
        out
    }

    /// Rung 95's metal leg reads this: the hotter metal's excess over its limit at an instant, K (−∞ unread).
    pub fn metal_over(&self, i: &Instant2) -> f64 {
        self.settings.metal_at(i).map_or(f64::NEG_INFINITY, |m| m.over())
    }

    /// Rung 95: one governed derivative evaluation — the model's own [`gv::try_der`], its failure in this view's terms.
    pub fn gov_der(&self, a: f64, b: f64, s: f64) -> Result<gv::GovDer, Failure> { self.gov_der_at(a, b, s, None) }

    /// Rung 101: [`ControlsSolver::gov_der`] with the lagged exhaust sensor's reading `tm`.
    pub fn gov_der_at(&self, a: f64, b: f64, s: f64, tm: Option<f64>) -> Result<gv::GovDer, Failure> {
        let g = self.governor.as_ref().expect("the governed route has its governor");
        let over = |i: &Instant2| self.metal_over(i);
        let metal: Option<&dyn Fn(&Instant2) -> f64> = if self.settings.metal_leg() { Some(&over) } else { None };
        gv::try_der_at(&self.core.fuel, &self.flight, g, &self.limiters(), metal, a, b, s, tm)
            .map_err(|f| gov_failure(f, a, b, s))
    }

    /// Rung 101: the lagged exhaust sensor, when the governor carries one.
    pub fn sensor_lag(&self) -> Option<SensorLag> { self.governor.as_ref().and_then(|g| g.lag) }

    /// Rung 101: the rest of one three-state governed RK4 step — the model's own [`gv::try_rk_step_sensed`].
    pub fn gov_step_sensed(&self, a: f64, b: f64, m: f64, s: f64, k1: (f64, f64, f64)) -> Result<(f64, f64, f64), Failure> {
        let g = self.governor.as_ref().expect("the governed route has its governor");
        let over = |i: &Instant2| self.metal_over(i);
        let metal: Option<&dyn Fn(&Instant2) -> f64> = if self.settings.metal_leg() { Some(&over) } else { None };
        gv::try_rk_step_sensed(&self.core.fuel, &self.flight, g, &self.limiters(), metal, a, b, m, s, DS, k1)
            .map_err(|f| gov_failure(f, a, b, s))
    }

    /// Rung 95: the rest of one governed RK4 step — the model's own [`gv::try_rk_step`].
    pub fn gov_step(&self, a: f64, b: f64, s: f64, k1: (f64, f64)) -> Result<(f64, f64), Failure> {
        let g = self.governor.as_ref().expect("the governed route has its governor");
        let over = |i: &Instant2| self.metal_over(i);
        let metal: Option<&dyn Fn(&Instant2) -> f64> = if self.settings.metal_leg() { Some(&over) } else { None };
        gv::try_rk_step(&self.core.fuel, &self.flight, g, &self.limiters(), metal, a, b, s, DS, k1)
            .map_err(|f| gov_failure(f, a, b, s))
    }

    /// Which march the model dispatches to (`r43_integrate_fuel`'s order).
    pub fn route(&self) -> Route {
        let s = &self.settings;
        // Rung 95: the governed march first; the prechecks refuse it beside every lagged route.
        if s.govern {
            return Route::Governor;
        }
        // A slow valve's machine dispatches to rung 65's march first (its fuel table); the prechecks
        // refuse it beside either fuel-side lag.
        if s.valve_slow() { Route::ValveLag }
        else if s.release_on { Route::Release } else if s.redline_on && s.gov_lag_on { Route::GovLag } else { Route::Plain }
    }

    /// The instant at a recorded point, re-read: the closure at the point's speeds and APPLIED fuel —
    /// the call the march's derivative made last there. `tests/sandbox_controls.rs` holds it to the
    /// recorded point bit for bit. A SLOW valve's position is a march state, so it is re-read at the
    /// RECORDED position (rung 65: re-solving it would hand back the command, not the valve).
    pub fn reread(&self, p: &FuelPoint) -> Result<FuelInstant, Abort> {
        let _st = match (self.route(), p.extra) {
            (Route::ValveLag, PointExtra::Valve { b, .. }) => Some(MarchedBleed::set(&self.core.fuel.inner, b)),
            (Route::ValveLag, _) => unreachable!("a slow valve's march records the valve at every point"),
            _ => None,
        };
        // RUNG 102: and the afterburner as lit at the point's time (it stays set for the rest of a [`Self::read`]).
        let base = &self.core.fuel.inner.inner.base;
        base.light_at(p.s);
        base.ab_last.set(None);
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
    /// Rung 65's slow valve (rung 88 gave it the incidence reference) — its position IS recorded.
    ValveLag,
    /// Rung 95's governed march — stateless, so every step can be re-run.
    Governor,
}

/// A failed governed evaluation in this view's terms.
fn gov_failure(f: gv::GovFailure, a: f64, b: f64, s: f64) -> Failure {
    let call = match f.call {
        GovCall::Governor => Call::Governor,
        GovCall::AtDemand => Call::AtSchedule,
        GovCall::Redline => Call::Redline,
        GovCall::Accel => Call::Accel,
        GovCall::Floor => Call::Floor,
        GovCall::Metal => Call::Metal,
        GovCall::AtApplied => Call::AtApplied,
    };
    Failure { message: f.message, call, mf: f.mf, nu_lp: a, nu_hp: b, s }
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
    /// The slow valve's command: the opening the valve would hold if it were instant (rung 65).
    Valve,
    /// Rung 95: the governor's own solve, and the metal leg's.
    Governor,
    Metal,
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
    sv.core.fuel.inner.inner.base.light_at(s);   // RUNG 102: the afterburner as lit at this march time
    let ft = &sv.core.fuel;
    let fl = &sv.flight;
    let lim = sv.limiters();
    let floor = lim.floor();
    let mut mf = mf_in;
    let mut i = match ft.try_instant_fuel(fl, a, b, mf) {
        Ok(i) => i,
        // No operating point at the schedule: the model decides the min-select from the solvable
        // ceiling below it (2026-10-10) — called, not copied, so the replay is the march's own.
        Err(e) => {
            let (_, i) = r43_cut_below_ceiling(ft, fl, a, b, mf, s, &lim, floor.as_ref(), e)
                .map_err(fail(Call::AtSchedule, mf))?;
            return Ok((i.base.phi_lp_dot / ft.rho(), i.base.phi_hp_dot));
        }
    };
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
    sv.core.fuel.inner.inner.base.light_at(s);   // RUNG 102: the afterburner as lit at this march time
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
    // Each leg off the schedule, or off the solvable ceiling where the schedule has no operating
    // point — `integrate_fuel_asym`'s `required`, through the same model helper (2026-10-10).
    let top = std::cell::OnceCell::new();
    let mut undecided = false;
    let mut caps: Vec<f64> = Vec::new();
    if let Some(accel) = lim.accel {
        let solve = |x: f64| ft.try_sched_fuel(fl, a, b, x, accel);
        match ft.try_leg_below(fl, a, b, mf_sched, &top, &solve).map_err(fail(Call::Accel, mf_sched))? {
            Some(c) => caps.push(c),
            None => undecided = true,
        }
    }
    if let Some(surge) = floor.as_ref() {
        let solve = |x: f64| ft.try_surge_fuel(fl, a, b, x, surge);
        match ft.try_leg_below(fl, a, b, mf_sched, &top, &solve).map_err(fail(Call::Floor, mf_sched))? {
            Some(c) => caps.push(c),
            None => undecided = true,
        }
    }
    if undecided && caps.is_empty() {
        let e = ft.try_instant_fuel(fl, a, b, mf_sched).err().expect("an undecided leg means the scheduled instant failed");
        return Err(fail(Call::AtSchedule, mf_sched)(e));
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

/// One derivative evaluation of rung 65's slow-valve march — `r65_integrate_fuel_valve_lag`'s `der`,
/// copied in its order: the min-select inside the valve's recorded position (rung 43's cut below the
/// ceiling, then each armed leg off the SCHEDULED fuel), then — outside it — the valve's command, rung
/// 64's root against the valve's own target (rung 88). Returns `(dν_L/ds, dν_H/ds, db/ds)`.
fn der_valve_lag(sv: &ControlsSolver, a: f64, h: f64, q: f64, s: f64) -> Result<(f64, f64, f64), Failure> {
    let fail = |call: Call, mf: f64| move |e: Abort| Failure { message: e.0, call, mf, nu_lp: a, nu_hp: h, s };
    sv.core.fuel.inner.inner.base.light_at(s);   // RUNG 102: the afterburner as lit at this march time
    let ft = &sv.core.fuel;
    let fl = &sv.flight;
    let lim = sv.limiters();
    let floor = lim.floor();
    let mf_sched = sv.settings.schedule(sv.mf_lo, sv.mf_hi)(s);
    let (mf, inst) = 'cut: {
        let _st = MarchedBleed::set(&ft.inner, q);
        let mut i = match ft.try_instant_fuel(fl, a, h, mf_sched) {
            Ok(i) => i,
            Err(e) => break 'cut r43_cut_below_ceiling(
                ft, fl, a, h, mf_sched, s,
                &FuelLimiters { tt4_max: lim.tt4_max, accel: lim.accel, ..FuelLimiters::default() }, floor.as_ref(), e)
                .map_err(fail(Call::AtSchedule, mf_sched))?,
        };
        let mut caps: Vec<f64> = Vec::new();
        if let Some(t4) = lim.tt4_max {
            if i.base.tt4 > t4 {
                caps.push(ft.try_topping_fuel(fl, a, h, t4, mf_sched).map_err(fail(Call::Redline, mf_sched))?);
            }
        }
        if let Some(sch) = lim.accel {
            caps.push(ft.try_sched_fuel(fl, a, h, mf_sched, sch).map_err(fail(Call::Accel, mf_sched))?);
        }
        if let Some(f) = floor.as_ref() {
            caps.push(ft.try_surge_fuel(fl, a, h, mf_sched, f).map_err(fail(Call::Floor, mf_sched))?);
        }
        caps.retain(|&c| c < mf_sched);
        let mf = if caps.is_empty() {
            mf_sched
        } else {
            let mut m = caps[0];
            for &c in &caps[1..] {
                if c < m {
                    m = c;
                }
            }
            m
        };
        if !caps.is_empty() {
            i = ft.try_instant_fuel(fl, a, h, mf).map_err(fail(Call::AtApplied, mf))?;
        }
        (mf, i)
    };
    let cmd = valve_command(sv, a, h, mf).map_err(|f| Failure { s, ..f })?;
    // Rung 89: the edge's own time constant — `(cmd - q) / tau` on a symmetric valve.
    let vl = ft.inner.lever.lim.expect("the slow-valve route's valve has its lag");
    Ok((inst.base.phi_lp_dot / ft.rho(), inst.base.phi_hp_dot, vl.lag_rate(cmd, q)))
}

/// The slow valve's command at a state and fuel — the march's own `command` (rung 64's root against the
/// valve's target, rung 88).
fn valve_command(sv: &ControlsSolver, a: f64, h: f64, mf: f64) -> Result<f64, Failure> {
    let ft = &sv.core.fuel;
    let vl = ft.inner.lever.lim.expect("the slow-valve route carries its valve");
    let (tt2, pt2, _) = ft.inner.inlet(&sv.flight);
    r64_solve_b_at(&vl, r87_target(&vl, &ft.inner, a, h, tt2), |b| {
        let _g = ForcedBleed::set(&ft.inner, b);
        r62_try_close_fuel(ft, a, h, mf, tt2, pt2)
    }).map(|r| r.1).map_err(|e| Failure { message: e.0, call: Call::Valve, mf, nu_lp: a, nu_hp: h, s: 0.0 })
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
        // Rung 101: the lagged sensor's reading is a march state, re-run from the recorded one (the third value).
        Route::Governor => match (p.extra, sv.sensor_lag()) {
            (PointExtra::Sensor { tm }, Some(lag)) => sv.gov_der_at(a, b, s, Some(tm))
                .and_then(|d| sv.gov_step_sensed(a, b, tm, s, (d.da, d.db, (d.inst.base.tt5 - tm) / lag.tau))),
            _ => sv.gov_der(a, b, s).and_then(|d| sv.gov_step(a, b, s, (d.da, d.db))).map(|(a, b)| (a, b, 0.0)),
        },
        Route::Plain => (|| {
            let (k1a, k1b) = der_plain(sv, a, b, sched(s), s)?;
            let mfm = sched(s + ds / 2.0);
            let (k2a, k2b) = der_plain(sv, a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, mfm, s + ds / 2.0)?;
            let (k3a, k3b) = der_plain(sv, a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, mfm, s + ds / 2.0)?;
            let (k4a, k4b) = der_plain(sv, a + ds * k3a, b + ds * k3b, sched(s + ds), s + ds)?;
            Ok((a + ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a),
                b + ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b), 0.0))
        })(),
        Route::ValveLag => (|| {
            let PointExtra::Valve { b: q, .. } = p.extra else {
                unreachable!("the slow-valve route records the valve");
            };
            let b_max = sv.core.fuel.inner.lever.lim.expect("the slow valve").b_max;
            let (k1a, k1b, k1q) = der_valve_lag(sv, a, b, q, s)?;
            let (k2a, k2b, k2q) = der_valve_lag(sv, a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, q + ds / 2.0 * k1q, s + ds / 2.0)?;
            let (k3a, k3b, k3q) = der_valve_lag(sv, a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, q + ds / 2.0 * k2q, s + ds / 2.0)?;
            let (k4a, k4b, k4q) = der_valve_lag(sv, a + ds * k3a, b + ds * k3b, q + ds * k3q, s + ds)?;
            let q = q + ds / 6.0 * (k1q + 2.0 * k2q + 2.0 * k3q + k4q);
            // the valve's stops, applied to the state as the march applies them (`min(b_max, max(0, q))`)
            Ok((a + ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a),
                b + ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b),
                b_max.min(0.0f64.max(q))))
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
    Some(stop_at(sv, points.last()))
}

/// [`stop_reason`]'s re-run from the last recorded point (`None`: the very first evaluation failed).
fn stop_at(sv: &ControlsSolver, last: Option<&FuelPoint>) -> Stop {
    let Some(last) = last else {
        // The very first evaluation failed: re-make it.
        let sched = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
        let (a, b) = (sv.start.nu_lp, sv.start.nu_hp);
        let failure = match sv.route() {
            Route::GovLag => None,
            Route::Governor => sv.gov_der_at(a, b, 0.0, sv.sensor_lag().map(|l| l.t0)).err(),
            Route::Plain => der_plain(sv, a, b, sched(0.0), 0.0).err(),
            Route::Release => der_release(sv, a, b, 0.0, 0.0).err(),
            // The march starts the valve at its command (rung 65's equilibrium start), solved before
            // its loop — a failure THERE panics in the model, so an empty run is the first `der`'s.
            Route::ValveLag => valve_command(sv, a, b, sched(0.0))
                .and_then(|q| der_valve_lag(sv, a, b, q, 0.0).map(|_| ())).err(),
        };
        return Stop { s_last: f64::NAN, failure };
    };
    let failure = match replay_step(sv, last) {
        None => None,
        Some(Err(f)) => Some(f),
        Some(Ok((a, b, g))) => {
            let s = last.s + DS;
            let sched = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
            match sv.route() {
                Route::Release => der_release(sv, a, b, g, s).err(),
                Route::ValveLag => der_valve_lag(sv, a, b, g, s).err(),
                Route::Governor => sv.gov_der_at(a, b, s, sv.sensor_lag().map(|_| g)).err(),
                _ => der_plain(sv, a, b, sched(s), s).err(),
            }.or_else(|| Some(Failure { message: "the march stopped, but re-running its last step does not fail".into(),
                                        call: Call::AtApplied, mf: f64::NAN, nu_lp: f64::NAN, nu_hp: f64::NAN,
                                        s: f64::NAN }))
        }
    };
    Stop { s_last: last.s, failure }
}

/// RUNG 102: did the lit afterburner stop this run? Lit at the failing call, and either the failure is the
/// afterburner's own (its refusal in words, or the lean wall), or the same step runs with the afterburner put out.
pub fn afterburner_stopped(sv: &ControlsSolver, stop: &Stop, last: Option<&FuelPoint>) -> bool {
    let base = &sv.core.fuel.inner.inner.base;
    let Some(f) = &stop.failure else { return false };
    if !base.afterburner.is_some_and(|ab| ab.w_at(f.s) > 0.0) {
        return false;
    }
    if crate::afterburner::is_lit_failure(&Abort(f.message.clone())) {
        return true;
    }
    base.ab_out.set(true);
    let without = stop_at(sv, last);
    base.ab_out.set(false);
    without.failure.is_none_or(|g| g.message.starts_with("the march stopped, but re-running"))
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
    /// The run's own fuel left the fuel-metered solver's range on the lean side (a fast power cut).
    Lean,
    /// ...on the rich side (a fast slam: the fuel outran the air).
    Rich,
    /// The temperature limiter's lag is on: that march does not record its lag state, so the failed
    /// step cannot be re-run and the cause is not known.
    Unknown,
    /// Rung 90: the aircraft's fixed amount of air grew past half the HP compressor's delivery as the
    /// engine slowed.
    Customer,
    /// Rung 92: the aircraft's generator took half or more of its turbine's work as the engine slowed.
    Generator,
    /// Rung 95: the governed metal leg met a point where the metal is over its limit even at the leanest fuel.
    MetalCap,
    /// Rung 46: the temperature limiter met a point where even the least fuel that solves burns hotter than the redline
    /// (plan § 29: met flown fast, at a run's first step).
    RedlineFloor,
    /// Rung 102: the lit afterburner — its own refusal, the lean wall, or a step that runs with it put out.
    Afterburner,
    /// Plan § 39: the march's fuel closure found no burner state at all at these shaft speeds — no trial mixture to
    /// read, neither wall — met where a slam starts low (near or below idle), mostly at its first step.
    LowSpeed,
    /// Anything else the model refused at this point of the run.
    Other,
}

impl StopKind {
    pub fn key(self) -> &'static str {
        match self {
            StopKind::FloorUnreachable => "floor_unreachable",
            StopKind::Lean => "lean",
            StopKind::Rich => "rich",
            StopKind::Unknown => "unknown",
            StopKind::Customer => "customer",
            StopKind::Generator => "generator",
            StopKind::MetalCap => "metal_cap",
            StopKind::RedlineFloor => "redline_floor",
            StopKind::Afterburner => "afterburner",
            StopKind::LowSpeed => "low_speed",
            StopKind::Other => "other",
        }
    }

    /// What the page says when a run stops this way, above the model's own message.
    pub fn words(self) -> &'static str {
        match self {
            StopKind::MetalCap =>
                "The turbine metal went over its limit even at the leanest fuel the engine runs on, so the metal cap had \
                 nothing left to burn and the model stops. Cool the turbine more, pick a newer material, or switch the \
                 metal cap off.",
            StopKind::RedlineFloor =>
                "The temperature limiter could not hold its redline: at these shaft speeds even the least fuel the \
                 engine can run on burns hotter than the redline, so the limiter has nothing left to cut and the model \
                 stops. Flying fast heats the air before it reaches the burner, which is where this was met. Raise the \
                 redline, or switch the limiter off.",
            StopKind::Customer =>
                "The aircraft asked for more air than the engine could give. Its customer bleed is a fixed amount of \
                 air, so as the engine slowed it became a bigger and bigger share of what the HP compressor delivers; \
                 past half, the burner is starved and the model stops. Ask for less air, or do not cut the power so \
                 far.",
            StopKind::Generator =>
                "The aircraft asked for more power than the engine could give. A fixed generator power is a bigger \
                 and bigger share of its turbine's work as the engine slows; past half, the compressor that turbine \
                 drives is starved and the model stops. Ask for less power, or do not cut the power so far.",
            StopKind::FloorUnreachable =>
                "The stall floor could no longer be held. The floor cuts the fuel to keep the compressor's flow \
                 coefficient up; here even cutting the fuel to almost nothing would not lift it to the floor, so the \
                 model stops. This happens when the floor sits above where the compressor runs: the floor starves \
                 the engine, the shafts slow, and the flow coefficient falls further. Lower the floor (with the \
                 stator schedule on, try the floor that watches blade incidence).",
            StopKind::Lean =>
                "The fuel fell faster than the air: on this power cut the mixture got leaner than the leanest the \
                 model's fuel-metered solver searches (0.004 kg of fuel per kg of air, or half the leaner steady end's \
                 mixture when that is lower), so the run stops. (A real burner has a lean blow-out limit too; this one \
                 is the solver's search bound, not a combustion model.) Cut the throttle more slowly, or not so far.",
            StopKind::Rich =>
                "The fuel arrived faster than the shafts could bring in air to burn it: the mixture would need more \
                 than 0.065 kg of fuel per kg of air, the richest the model's fuel-metered solver searches (burning \
                 every bit of the oxygen takes about 0.068). A real engine's fuel controls exist to prevent exactly \
                 this — try the acceleration schedule or the temperature limiter, or a slower move.",
            StopKind::Unknown =>
                "With the temperature limiter's lag switched on, the model's march does not record the lag's own \
                 state, so the sandbox cannot re-run the failed step to say why the run stopped here. The points \
                 before the stop are the model's own.",
            StopKind::Afterburner =>
                "The lit afterburner stopped the run: either it refused here in its own words, or this step runs with \
                 it put out. Behind a held nozzle the \
                 afterburner's hot gas needs engine pressure to get out, and a limiter cutting the main fuel, or the \
                 shafts slowing, can take the engine below the point where it can — the model keeps both turbine \
                 throats choked, so there it has no running point (a real engine would stall). Or the afterburner \
                 cannot reach its temperature here (the model's words below say which). Open the nozzle, or let it \
                 open by the rule; light the afterburner later in the run; or lower its temperature.",
            StopKind::LowSpeed =>
                "The shafts are too slow here for the model's step-by-step fuel search: at these speeds it finds no \
                 state of the burner to work from, so the run stops. This happens when a move starts at a low throttle \
                 (near or below idle). The starting point itself is a steady one, so this is a limit of the model's \
                 search, not proof the engine could not move from there. Start from a higher throttle.",
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

/// [`trial_mixture`], or — for a stop at the first step, before any point was recorded — the same estimate off the
/// run's steady start (its fuel over its mixture is its LP-face air flow). Plan § 40: § 39 named a first-step stop "no
/// burner state" for want of a point to read; 1 553 of its 1 691 were starts leaner than the lean wall.
pub fn run_mixture(sv: &ControlsSolver, f: &Failure, last: Option<&FuelPoint>) -> Option<f64> {
    match last {
        Some(_) => trial_mixture(f, last),
        None => (f.mf.is_finite() && sv.mf_lo > 0.0).then(|| f.mf * sv.start.close.f / sv.mf_lo),
    }
}

/// The lean wall this run's fuel closures scan to (plan § 40): rung 43's 0.004, or the lower one the view set.
pub fn lean_wall(sv: &ControlsSolver) -> f64 {
    sv.core.fuel.inner.inner.base.lean_wall.get().unwrap_or(FuelTransientCore::F_FLOOR)
}

/// Classify a re-run failure by its message and, for a failed fuel closure, by its mixture.
///
/// A fifth kind lived here until 2026-10-10: the plain march's every-step check AT THE SCHEDULED FUEL
/// stopped a run whose cut fuel solved (plan § 12.10). The Controls view now switches on
/// `FuelLimiters::below_ceiling`, which decides the limiters from the most fuel that solves. On a
/// 1 000-request sweep of the slider box (2026-10-10, same requests before and after) the 24 such stops
/// went to none; overall 23 more runs completed and 4 more ended at a real floor-unreachable stop (one
/// rich stop and two cause-unknown stops on the governor-lag route also ran on).
pub fn classify(sv: &ControlsSolver, stop: &Stop, last: Option<&FuelPoint>) -> StopKind {
    // Rung 102 first: a lit afterburner names its own stops, whatever solve met them.
    if afterburner_stopped(sv, stop, last) {
        return StopKind::Afterburner;
    }
    let Some(f) = &stop.failure else { return StopKind::Unknown };
    let m = &f.message;
    if m.contains("UNREACHABLE") {
        return StopKind::FloorUnreachable;
    }
    // Rung 91's refusal names the customer and the cooling together; only the customer's amount grows.
    if m.contains("customer bleed takes") || (m.contains("turbine cooling") && m.contains("together take")) {
        return StopKind::Customer;
    }
    if m.contains("rung-95: the turbine metal is over") {
        return StopKind::MetalCap;
    }
    if m.contains("rung-92:") {
        return StopKind::Generator;
    }
    if m.contains("redline below the flow's floor") {
        return StopKind::RedlineFloor;
    }
    if !m.contains("fuel closure does not bracket") {
        return StopKind::Other;
    }
    // Plan § 39: a wall is named only near it (the crash map's lean and rich stops sit at 0.71–1.05 of their walls);
    // a closure that failed between them found no burner state at all. Plan § 40: the lean wall is the run's own, and a
    // stop at the first step reads its mixture off the start.
    match run_mixture(sv, f, last) {
        Some(x) if x < lean_rich_split() && x < 2.0 * lean_wall(sv) => StopKind::Lean,
        Some(x) if x >= lean_rich_split() && x > 0.5 * FuelTransientCore::F_CAP => StopKind::Rich,
        _ => StopKind::LowSpeed,
    }
}

/// One Controls run.
pub struct ControlsOutcome {
    pub settings: ControlsSettings,
    pub points: Vec<FuelPoint>,
    pub stop: Option<Stop>,
    /// Rung 95: what the governor did at each point (empty when the move is not governed).
    pub marks: Vec<GovMark>,
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
    let (points, marks) = sv.march_marked();
    let stop = stop_reason(&sv, &points);
    Ok((sv, ControlsOutcome { settings: *s, points, stop, marks }))
}

/// Which limiter holds the fuel at a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holder {
    /// The applied fuel IS the scheduled fuel.
    None,
    Redline,
    Accel,
    Floor,
    /// Rung 95: the metal leg (the governed march's cap at the turbine metal's limit).
    Metal,
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
            Holder::Metal => "metal",
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
    /// The LP flow coefficient the controlled valve holds at this point (rung 87: an incidence valve's
    /// moves with the LP stators), when the valve is on.
    pub valve: Option<f64>,
    pub holder: Holder,
    /// Rung 90: the aircraft's share of the HP compressor's air here (0 with no customer bleed).
    pub customer: f64,
    /// Rung 93: both turbine metals here, when the readout is on.
    pub metal: Option<MetalReading>,
    /// Each armed limiter's relative residual at the applied state (`None` = not armed).
    pub res_redline: Option<f64>,
    pub res_accel: Option<f64>,
    pub res_floor: Option<f64>,
    /// Rung 96: the control's reading here and the thrust its table makes of it (`None` unless thrust is inferred).
    pub reading: Option<f64>,
    pub inferred: Option<f64>,
    /// Rung 98: a pressure-ratio control on a table made at the design flight (pressure corrected), at a choked point —
    /// its gap `F_true − F̂` split into the RAM-DRAG part and the NOZZLE part (anchor § 1).
    pub gap_ram: Option<f64>,
    pub gap_nozzle: Option<f64>,
    /// Rung 101: the gas's temperature at the nozzle inlet, and a lagged exhaust sensor's reading of it (`None` unless
    /// the sensor lags).
    pub tt5: f64,
    pub sensor: Option<f64>,
    /// Rung 101: under the exhaust sensor, the reading the control demands HERE (its belief depends on the instant).
    pub demand: Option<f64>,
    /// Rung 102: what the lit afterburner did here (`None` unlit or not fitted).
    pub ab: Option<crate::afterburner::AbAt>,
}

impl ControlsSolver {
    /// The thrust of an instant, by the rule [`PointRead::thrust`] states.
    pub fn thrust(i: &Instant2) -> f64 { gv::thrust(i) }

    /// The stator setting on a spool at a state, and that compressor's stall line there.
    pub fn stall(&self, spool: Spool, nu_lp: f64, nu_hp: f64) -> (f64, f64) {
        let v = self.core.v_of(spool, nu_lp, nu_hp, None);
        (v, self.core.design_map(spool).with_vsv(v).phi_surge_at())
    }

    /// Read one recorded point.
    pub fn read(&self, p: &FuelPoint) -> Result<PointRead, Abort> {
        let s = &self.settings;
        let inst = self.reread(p)?;
        // Rung 102: what the lit afterburner did in that instant (read now: the readouts below solve more instants).
        let ab = self.core.fuel.inner.inner.base.ab_last.get();
        let i = &inst.base;
        let (v_lp, stall_lp) = self.stall(Spool::Lp, p.nu_lp, p.nu_hp);
        let (v_hp, stall_hp) = self.stall(Spool::Hp, p.nu_lp, p.nu_hp);
        let floor = match self.limiters().floor() {
            None => None,
            Some(fl) => Some(self.core.fuel.resolve_floor(&fl, p.nu_lp, p.nu_hp)?.phi_lim),
        };
        // The valve's target at the LP stator setting the model's own valve solve reads (at the run's inlet Tt2).
        let valve = self.core.fuel.inner.lever.lim.map(|l| {
            let (tt2, _, _) = self.core.fuel.inner.inlet(&self.flight);
            l.target(self.core.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2)))
        });
        // Rung 101: a lagged exhaust sensor's reading at this point is the march's recorded state.
        let tm = match p.extra { PointExtra::Sensor { tm } => Some(tm), _ => None };
        let demand = match self.governor.as_ref().and_then(|g| Some((g, g.infer.as_ref().filter(|t| t.sensed())?, g.law.reading()?))) {
            Some((g, t, r)) => Some(t.try_demand_at(&self.core.fuel, r, g.thrust_demand(p.s), i, tm)?),
            None => None,
        };
        let (reading, inferred) = match self.governor.as_ref().and_then(|g| Some((g.infer.as_ref()?, g.law.reading()?))) {
            Some((t, r)) => {
                let v = t.read(r, i);
                (Some(v), Some(match t.sensed() {
                    true => t.try_belief(&self.core.fuel, r, i, tm)?,
                    false => t.try_inferred(&self.core.fuel, r, v)?,
                }))
            }
            None => (None, None),
        };
        let (gap_ram, gap_nozzle) = match (self.tables.as_ref(), self.carry.as_ref(), self.design_table.as_ref(), inferred) {
            (Some(Tables::Design(_)), Some(c), Some(dt), Some(f_hat))
                if s.corr_p && s.thrust_by == ThrustBy::Epr && i.branch == Branch::Choked =>
                (Some(c.ram_part(dt, i)), Some(c.nozzle_part(dt, i, Self::thrust(i), f_hat))),
            // Rung 99: under the formula, the airflow guess and the rest — zero while the nozzle is choked. Rung 100:
            // the push worked out from the reading, the rest is zero at every point, whatever the pressure switch.
            (Some(tb @ Tables::Formula { .. }), _, _, Some(f_hat))
                if (s.corr_p || !s.formula_choked) && s.thrust_by == ThrustBy::Epr => {
                let t = self.governor.as_ref().and_then(|g| g.infer.as_ref()).expect("an inferred run reads its table");
                tb.formula_split(t, i, Self::thrust(i), f_hat, tm).map_or((None, None), |(a, b)| (Some(a), Some(b)))
            }
            _ => (None, None),
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
            Route::ValveLag => &[Holder::Redline, Holder::Accel, Holder::Floor],
            Route::Governor => &[Holder::Redline, Holder::Accel, Holder::Floor],
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
            bleed: i.close.bleed.unwrap_or(0.0), floor, valve, holder, customer: i.close.customer.unwrap_or(0.0),
            metal: s.metal_at(i),
            res_redline, res_accel, res_floor,
            reading, inferred, gap_ram, gap_nozzle, tt5: i.tt5, sensor: tm, demand, ab,
        })
    }
}

fn num_or_null(x: f64) -> Json { if x.is_finite() { Json::Float(x) } else { Json::Null } }

/// One steady endpoint, for the page.
fn steady_json(sv: &ControlsSolver, i: &Instant2, hold: Option<HoldInfo>) -> Json {
    let (_, stall_lp) = sv.stall(Spool::Lp, i.nu_lp, i.nu_hp);
    let (_, stall_hp) = sv.stall(Spool::Hp, i.nu_lp, i.nu_hp);
    let mut j = jobj! {
        "Tt4" => i.tt4, "nu_lp" => i.nu_lp, "nu_hp" => i.nu_hp, "fuel" => i.close.f * i.close.mdot_air,
        "far" => i.close.f, "thrust" => ControlsSolver::thrust(i), "phi_lp" => i.close.phi_lp, "phi_hp" => i.close.phi_hp,
        "pi_lpc" => i.close.pi_lpc, "pi_hpc" => i.close.pi_hpc, "stall_lp" => stall_lp, "stall_hp" => stall_hp,
        "choked" => Json::Int((i.branch == Branch::Choked) as i64),
        // Rung 90: what the aircraft gets — its share of the HP compressor's air, and kg/s.
        "customer_share" => i.close.customer.unwrap_or(0.0),
        "customer_air" => match (i.close.customer, i.close.mdot_face) {
            (Some(c), Some(face)) => c * (1.0 - i.close.bleed.unwrap_or(0.0)) * face,
            _ => 0.0,
        },
        // Rung 92: the generator's share of each turbine's work.
        "generator_share_hp" => shaft_offtake::shares(&sv.core.fuel.inner, i).0,
        "generator_share_lp" => shaft_offtake::shares(&sv.core.fuel.inner, i).1,
        // Rung 93: both metals at the steady point (null with the readout off).
        "metal_vane" => sv.settings.metal_at(i).map(|m| Json::Float(m.vane)).unwrap_or(Json::Null),
        "metal_blade" => sv.settings.metal_at(i).map(|m| Json::Float(m.blade)).unwrap_or(Json::Null),
        // Rung 94: what the end held (null when it held the burner temperature or carried no load).
        "hold_set" => hold.map_or(Json::Null, |h| Json::Float(h.set)),
        "hold_target" => hold.map_or(Json::Null, |h| Json::Float(h.target)),
        "hold_achieved" => hold.map_or(Json::Null, |h| Json::Float(h.achieved)),
        "hold_capped" => hold.map_or(Json::Null, |h| Json::Int(h.capped as i64)),
    };
    // Plan § 39: an end below idle — no positive thrust per kg/s of INLET air, the quantity the rule judges — carries
    // that number, and the page warns. Absent otherwise, so every other reply is unchanged.
    let sp = i.sp_thrust_inlet.unwrap_or(i.sp_thrust);
    if !(sp > 0.0) {
        if let Json::Obj(kv) = &mut j { kv.push(("below_idle".into(), num_or_null(sp))); }
    }
    j
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
    let mut j = jobj! {
        "ok" => Json::Int(1),
        "controls" => o.settings.to_json(),
        "ambient" => crate::sandbox::ambient_json(o.settings.t0, o.settings.p0),
        "density_ratio" => o.settings.density_ratio(),
        "start" => steady_json(sv, &sv.start, sv.hold_start), "end" => steady_json(sv, &sv.end, sv.hold_end),
        "expected_points" => Json::Int(o.settings.expected_points() as i64),
        "route" => match sv.route() { Route::Plain => "plain", Route::GovLag => "gov_lag", Route::Release => "release",
                                      Route::ValveLag => "valve_lag", Route::Governor => "governor" },
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
                    "mixture" => f.and_then(|f| run_mixture(sv, f, last)).map(num_or_null).unwrap_or(Json::Null),
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
        "valve" => rcol(&|r| r.valve.unwrap_or(f64::NAN)), "customer" => rcol(&|r| r.customer),
        // Rung 95: on the governed route the march recorded which leg set the fuel — read it, not a residual.
        "holder" => match o.marks.is_empty() {
            true => Json::List(reads.iter().map(|r| Json::from(r.as_ref().map(|r| r.holder.key()).unwrap_or("none"))).collect()),
            false => Json::List(o.marks.iter().map(|m| Json::from(match m.leg {
                Leg::None => Holder::None, Leg::Redline => Holder::Redline, Leg::Accel => Holder::Accel,
                Leg::Floor => Holder::Floor, Leg::Metal => Holder::Metal,
            }.key())).collect()),
        },
        // Rung 95: where the governor's own fuel sat (root / rich / lean), and the held quantity it asked for.
        "governor" => match o.marks.is_empty() {
            true => Json::Null,
            false => Json::List(o.marks.iter().map(|m| Json::from(m.clamp.key())).collect()),
        },
        "demand" => match &sv.governor {
            None => Json::Null,
            // Rung 100: through the formula when the table carries one — it ran at every one of these points.
            // Rung 101: under the exhaust sensor, worked out at each point's own instant.
            Some(g) if g.infer.as_ref().is_some_and(inf::Table::sensed) => rcol(&|r| r.demand.unwrap_or(f64::NAN)),
            Some(g) => col(&|x| g.try_demand(&sv.core.fuel, x.s).unwrap_or(f64::NAN)),
        },
        // Rung 96: an inferred thrust — what the control reads, the thrust it believes, the thrust the throttle asks,
        // and the table it reads them through.
        "reading" => match o.settings.inferred() {
            false => Json::Null,
            true => Json::from(o.settings.thrust_by.key()),
        },
        "calibrated" => Json::Int(o.settings.calibrated() as i64),
        "reading_value" => rcol(&|r| r.reading.unwrap_or(f64::NAN)),
        "thrust_inferred" => rcol(&|r| r.inferred.unwrap_or(f64::NAN)),
        "thrust_demand" => match sv.governor.as_ref().filter(|g| g.infer.is_some()) {
            None => Json::Null,
            Some(g) => col(&|x| g.thrust_demand(x.s)),
        },
        "table" => match sv.governor.as_ref().and_then(|g| g.infer.as_ref()) {
            None => Json::Null,
            Some(t) => Json::List(t.rows.iter().map(|w| jobj! {
                "thrust" => w.thrust, "epr" => w.epr, "nu_lp" => w.nu_lp, "fuel" => w.fuel,
            }).collect()),
        },
        // Rung 98: where the tables were made, the carry, and the pressure-ratio gap's two parts.
        "tables" => match sv.carry.as_ref() {
            None => jobj! { "at" => "flight", "carried" => Json::Int(0) },
            Some(c) => jobj! { "at" => "design", "carried" => Json::Int(1), "theta" => c.theta, "delta" => c.delta,
                               "corr_p" => Json::Int(o.settings.corr_p as i64), "corr_t" => Json::Int(o.settings.corr_t as i64),
                               "speed" => {
                                   let mut j = speed_json(&o.settings, sv.tables.as_ref());
                                   // Rung 100: under the fan-speed formula, the burner temperature of the table row it
                                   // settles on at each end, beside the end's own (written only then).
                                   let fan = sv.governor.as_ref().and_then(|g| g.infer.as_ref())
                                       .and_then(|t| t.formula.as_ref().filter(|f| matches!(f, Formula::Fan { .. })).map(|f| (t, f)));
                                   if let (Some((t, f)), Json::Obj(kv)) = (fan, &mut j) {
                                       let row = |i: &Instant2| f.try_fan_root(&sv.core.fuel, t, i.nu_lp)
                                           .map_or(Json::Null, |(x, _)| Json::List(vec![Json::Float(f.flight_tt4(x)), Json::Float(i.tt4)]));
                                       kv.push(("fan_rows".into(), Json::List(vec![row(&sv.start), row(&sv.end)])));
                                   }
                                   j
                               } },
        },
        "gap_ram" => rcol(&|r| r.gap_ram.unwrap_or(f64::NAN)),
        "gap_nozzle" => rcol(&|r| r.gap_nozzle.unwrap_or(f64::NAN)),
        "choked" => Json::List(p.iter().map(|x| Json::Int((x.branch == Branch::Choked) as i64)).collect()),
        // Rung 93: both metals along the run and their limits (null columns with the readout off).
        "metal_vane" => rcol(&|r| r.metal.map_or(f64::NAN, |m| m.vane)),
        "metal_blade" => rcol(&|r| r.metal.map_or(f64::NAN, |m| m.blade)),
        "metal_limit" => match o.settings.metal_limit() {
            None => Json::Null,
            Some(l) => jobj! { "vane" => l.vane, "blade" => l.blade, "factor" => l.factor },
        },
    };
    // Rung 101: with a lagged exhaust sensor, the gas's exhaust temperature and the sensor's reading of it (written only
    // then, so every other reply keeps rung 100's shape).
    if let (Some(_), Json::Obj(kv)) = (sv.sensor_lag(), &mut j) {
        kv.push(("exhaust_tt5".into(), rcol(&|r| r.tt5)));
        kv.push(("exhaust_sensor".into(), rcol(&|r| r.sensor.unwrap_or(f64::NAN))));
    }
    // Rung 102: with the afterburner fitted, what it is, and through the run how far lit, its throat over the dry one,
    // its own fuel-air ratio and its exit temperature (written only then).
    if let (Some(fit), Json::Obj(kv)) = (o.settings.afterburner_fit(), &mut j) {
        kv.push(("afterburner".into(), jobj! {
            "mode" => if fit.light_at.is_some() { "lit" } else { "unlit" }, "Tt7" => fit.ab.tt7,
            "light_at" => fit.light_at.map_or(Json::Null, Json::Float), "ramp" => crate::afterburner::LIGHT_RAMP,
            "match" => Json::Int(o.settings.a8_match as i64), "a8_ratio" => o.settings.a8_ratio,
        }));
        kv.push(("ab_w".into(), rcol(&|r| r.ab.map_or(0.0, |a| a.w))));
        kv.push(("ab_throat".into(), rcol(&|r| r.ab.map_or(1.0, |a| a.s))));
        kv.push(("ab_far".into(), rcol(&|r| r.ab.map_or(0.0, |a| a.detail.f7 - a.detail.f5))));
        kv.push(("Tt7".into(), rcol(&|r| r.ab.map_or(f64::NAN, |a| a.s7.tt))));
    }
    j
}

/// RUNG 101: the burner temperature at which the steady belief `f_at` equals `target`, searched from the set throttle
/// `t0` — stepped by rung 94's grid toward the target (the belief rises with the burner temperature) between rung 94's
/// bounds, then bisected to its tolerance.
fn sensed_end_root(f_at: &dyn Fn(f64) -> Result<f64, Abort>, target: f64, t0: f64) -> Result<f64, String> {
    let at = |t: f64| f_at(t).map_err(|e| format!("at {t:.0} K the engine or its sensor's belief has no answer ({})", e.0));
    // A loaded engine (a generator, a valve) can have no steady point at the set throttle itself — the reason its end
    // is calibrated at all: start from the nearest grid temperature that answers, upward first (the crash map's four).
    let (t0, q0) = match at(t0) {
        Ok(q) => (t0, q),
        Err(first) => {
            let n = ((HOLD_HI - HOLD_LO) / bh::GRID).ceil() as i32;
            (1..=n).flat_map(|k| [t0 + k as f64 * bh::GRID, t0 - k as f64 * bh::GRID])
                .filter(|t| (HOLD_LO..=HOLD_HI).contains(t))
                .find_map(|t| at(t).ok().map(|q| (t, q)))
                .ok_or(first)?
        }
    };
    if q0 == target {
        return Ok(t0);
    }
    let step = if q0 > target { -bh::GRID } else { bh::GRID };
    let (mut a, mut qa) = (t0, q0);
    let (mut b, mut qb);
    loop {
        b = (a + step).clamp(HOLD_LO, HOLD_HI);
        qb = at(b)?;
        if (qb > target) != (qa > target) || qb == target {
            break;
        }
        if b == HOLD_LO || b == HOLD_HI {
            return Err(format!("the belief does not reach {target:.1} N per kg/s between {HOLD_LO:.0} and {HOLD_HI:.0} K"));
        }
        (a, qa) = (b, qb);
    }
    let (mut lo, mut hi) = if a < b { (a, b) } else { (b, a) };
    let rising = |q: f64| q > target;
    let lo_above = rising(if a < b { qa } else { qb });
    for _ in 0..bh::MAX_HALVINGS {
        if hi - lo <= bh::T_TOL {
            break;
        }
        let mid = 0.5 * (lo + hi);
        if rising(at(mid)?) == lo_above { lo = mid } else { hi = mid }
    }
    let (ql, qh) = (at(lo)?, at(hi)?);
    Ok(if (ql - target).abs() <= (qh - target).abs() { lo } else { hi })
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
        ControlsSettings { lever: LeverChoice::Both, accel_on: true, ..d },                   // rung 86's plant
        ControlsSettings { lever: LeverChoice::Both, redline_on: true, gov_lag_on: true, ..d },
        ControlsSettings { lever: LeverChoice::Both, floor_on: true, floor_phi: 0.759, ..d },  // the bleed rescues it
        ControlsSettings { lever: LeverChoice::Stator, floor_on: true, floor_phi: 0.825, ..d }, // starved at once
        // rung 87: the controlled valve, watching each wall, with the stators and beside a floor
        ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, ..d },
        ControlsSettings { lever: LeverChoice::Both, bleed_mode: BleedMode::Valve, valve_ref: FloorRef::Incidence, ..d },
        ControlsSettings { lever: LeverChoice::Both, bleed_mode: BleedMode::Valve, valve_ref: FloorRef::Incidence,
                           floor_on: true, floor_phi: 0.78, ..d },                                  // mismatched: starves
        ControlsSettings { lever: LeverChoice::Both, bleed_mode: BleedMode::Valve, valve_ref: FloorRef::Incidence,
                           floor_on: true, floor_phi: 0.77, floor_ref: FloorRef::Incidence, ..d }, // matched, below: runs
        // rung 88: the SLOW valve (rung 65's march), watching each wall; and refused beside a fuel-side lag
        ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, valve_lag_on: true, ..d },
        ControlsSettings { lever: LeverChoice::Both, bleed_mode: BleedMode::Valve, valve_ref: FloorRef::Incidence,
                           valve_lag_on: true, valve_tau: 0.2, accel_on: true, ..d },
        // rung 89: the slow valve closing at its own speed — slower, and faster, than it opens
        ControlsSettings { lever: LeverChoice::Both, bleed_mode: BleedMode::Valve, valve_lag_on: true, valve_tau: 0.02,
                           valve_close_own: true, valve_tau_close: 0.2, ..d },
        ControlsSettings { lever: LeverChoice::Both, bleed_mode: BleedMode::Valve, valve_ref: FloorRef::Incidence,
                           valve_lag_on: true, valve_tau: 0.2, valve_close_own: true, valve_tau_close: 0.02, ..d },
        ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, valve_lag_on: true,
                           valve_close_own: true, valve_tau_close: 0.01, ..d },                     // refused: one step
        ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, valve_lag_on: true,
                           floor_on: true, release_on: true, ..d },                                 // refused: two lags
        // rung 90: the aircraft's customer bleed — a share, an amount beside the valve, beside the slow
        // valve and the schedule, and refused at half the air
        ControlsSettings { customer: CustomerMode::Share, ..d },
        ControlsSettings { customer: CustomerMode::Amount, lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, ..d },
        ControlsSettings { customer: CustomerMode::Share, customer_share: 0.1, lever: LeverChoice::Both,
                           bleed_mode: BleedMode::Valve, valve_lag_on: true, accel_on: true, ..d },
        ControlsSettings { customer: CustomerMode::Share, customer_share: 0.5, ..d },             // refused: half
        // rung 91: turbine cooling — vane air alone, both streams beside the valve and a customer amount,
        // and refused past a quarter / with a customer share past half together
        ControlsSettings { cool_vane: 0.05, ..d },
        ControlsSettings { cool_vane: 0.04, cool_blade: 0.03, customer: CustomerMode::Amount, lever: LeverChoice::Bleed,
                           bleed_mode: BleedMode::Valve, ..d },
        ControlsSettings { cool_blade: 0.25, ..d },                                                // refused: quarter
        ControlsSettings { cool_vane: 0.2, cool_blade: 0.2, customer: CustomerMode::Share, customer_share: 0.15, ..d }, // refused
        // rung 92: the generator on each shaft, by power and by share, beside the valve and the cooling, and
        // refused at half
        ControlsSettings { generator: GeneratorMode::Hp, ..d },
        ControlsSettings { generator: GeneratorMode::Lp, lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, ..d },
        ControlsSettings { generator: GeneratorMode::Hp, generator_by: GeneratorBy::Share, cool_vane: 0.04,
                           customer: CustomerMode::Share, ..d },
        ControlsSettings { generator: GeneratorMode::Lp, generator_by: GeneratorBy::Share, generator_share: 0.5, ..d }, // refused
        // rung 93: the metal readout — uncooled, cooled with a customer amount, a newer material, and refused
        ControlsSettings { metal_on: true, ..d },
        ControlsSettings { metal_on: true, cool_vane: 0.05, cool_blade: 0.03, customer: CustomerMode::Amount, ..d },
        ControlsSettings { metal_on: true, metal_tech: 3, metal_year: 2010.0, cool_vane: 0.04, cool_blade: 0.02,
                           redline_on: true, ..d },
        ControlsSettings { metal_on: true, metal_year: 1900.0, ..d },                              // refused: year
        // rung 94: the ends hold thrust or HP speed with the generator on; the metal cap; and refused
        ControlsSettings { generator: GeneratorMode::Hp, hold: Hold::Thrust, ..d },
        ControlsSettings { generator: GeneratorMode::Lp, hold: Hold::HpSpeed, lever: LeverChoice::Bleed,
                           bleed_mode: BleedMode::Valve, ..d },
        ControlsSettings { generator: GeneratorMode::Hp, hold: Hold::Thrust, metal_on: true, hold_cap: true,
                           cool_vane: 0.04, cool_blade: 0.02, ..d },
        ControlsSettings { generator: GeneratorMode::Hp, hold: Hold::Thrust, hold_cap: true, ..d },  // refused: no metal
        ControlsSettings { hold: Hold::Thrust, hold_cap: true, metal_on: true, cool_vane: 0.04, ..d }, // refused: no generator
        // rung 95: the governor — thrust alone and under the redline and schedule, HP speed by each law (the commanded
        // rate on a slowdown, on the lean edge), the metal leg with no generator, and refused beside a lag
        ControlsSettings { govern: true, hold: Hold::Thrust, ..d },
        ControlsSettings { govern: true, hold: Hold::Thrust, redline_on: true, accel_on: true, ..d },
        ControlsSettings { govern: true, hold: Hold::HpSpeed, gov_tau: 0.04, from: 1400.0, to: 1000.0, ..d },
        ControlsSettings { govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, generator: GeneratorMode::Lp, ..d },
        ControlsSettings { govern: true, hold: Hold::Thrust, metal_on: true, hold_cap: true, cool_vane: 0.04,
                           cool_blade: 0.02, ..d },
        ControlsSettings { govern: true, redline_on: true, gov_lag_on: true, ..d },                // refused: a lag
        // rung 96: thrust read from the pressure ratio (beside the limiters; calibrated on the clean engine with the
        // generator) and the fan speed by each law (on a slowdown), and refused beside the cap
        ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, redline_on: true, accel_on: true, ..d },
        ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, generator: GeneratorMode::Hp, ..d },
        ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, generator: GeneratorMode::Lp,
                           table_clean: false, ..d },
        ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop,
                           from: 1400.0, to: 1000.0, ..d },
        ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, metal_on: true, hold_cap: true,
                           cool_vane: 0.04, ..d },                                                  // refused: the cap
        ControlsSettings { from: 1400.0, to: 1000.0, ..d },
        ControlsSettings { release_on: true, ..d },                                  // refused: nothing to lag
        ControlsSettings { floor_on: true, lever: LeverChoice::Stator, ..d },        // stop: the floor unreachable
        ControlsSettings { shape: 2, from: 1485.0, to: 614.0, ramp: 0.66, settle: 1.0, ..d },   // stop: lean
    ] {
        out.push(req(s));
    }
    // Plan § 27: the altitude knob — sea level, a hot sea-level day with the floor, 15 km governed beside the redline,
    // the gain refused in dense air, and the crash map's one new end failure (the nozzle).
    let at = |z: f64, dt: f64, s: ControlsSettings| {
        let (t, p) = crate::atmosphere::standard(z);
        ControlsSettings { t0: t + dt, p0: p, ..s }
    };
    for s in [
        at(0.0, 0.0, d),
        at(0.0, 30.0, ControlsSettings { floor_on: true, lever: LeverChoice::Bleed, ..d }),
        at(15_000.0, 0.0, ControlsSettings { govern: true, hold: Hold::Thrust, redline_on: true, ..d }),
        at(0.0, -20.0, ControlsSettings { govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 50.0, ..d }),
        ControlsSettings { t0: 281.025575242416, p0: 110666.3721625535, redline_on: true, generator: GeneratorMode::Hp,
                           generator_power: 0.05, customer: CustomerMode::Amount, customer_amount: 0.13, cool_vane: 0.04,
                           cool_blade: 0.12, from: 600.0, to: 870.0, ramp: 0.02, settle: 3.3, rho: 2.3, ..d },
    ] {
        out.push(req(s));
    }
    // Rung 97: the nozzle unchoked at the start — a +40 K sea-level day from 700 K, bare, with the generator on the LP
    // shaft, with cooling, and governed on the pressure ratio.
    for s in [
        at(0.0, 40.0, ControlsSettings { from: 700.0, ..d }),
        at(0.0, 40.0, ControlsSettings { from: 700.0, generator: GeneratorMode::Lp, ..d }),
        at(0.0, 40.0, ControlsSettings { from: 700.0, cool_vane: 0.05, cool_blade: 0.03, ..d }),
        // Pinned to tables made at this flight (rung 98 made the design flight the default) so it keeps testing rung 97.
        at(0.0, 40.0, ControlsSettings { from: 700.0, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr,
                                         table_at: TableAt::Flight, ..d }),
    ] {
        out.push(req(s));
    }
    // Plan § 29: the Mach knob — slow, fast and governed beside the redline, slow from 700 K (the nozzle unchoked at the
    // start), high and fast in thin air, the gain refused in rammed-dense air, Mach zero, and off the maps at Mach 3.5.
    for s in [
        ControlsSettings { m0: 0.3, ..d },
        ControlsSettings { m0: 1.6, govern: true, hold: Hold::HpSpeed, redline_on: true, ..d },
        ControlsSettings { m0: 0.05, from: 700.0, ..d },
        at(20_000.0, 0.0, ControlsSettings { m0: 2.5, govern: true, hold: Hold::Thrust, ..d }),
        ControlsSettings { m0: 1.6, govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: GAIN_MAX, ..d },
        ControlsSettings { m0: 0.0, ..d },
        ControlsSettings { m0: 3.5, ..d },
    ] {
        out.push(req(s));
    }
    // Rung 98: the tables made at the design flight -- the pressure ratio at Mach 1.4 (less thrust than asked) and the
    // same with the tables made there, the fan speed uncorrected for temperature on a warm day, the schedule uncorrected
    // in cold air, the proportional speed law uncorrected on a warm day (it reads off its table), and Mach 3 at sea level,
    // beyond the design-flight table's reach.
    for s in [
        // Pinned to no speed correction (rung 99 made the speed grid the default) so these keep testing rung 98.
        ControlsSettings { m0: 1.4, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, speed_corr: SpeedCorr::None, ..d },
        ControlsSettings { m0: 1.4, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, table_at: TableAt::Flight, ..d },
        ControlsSettings { t0: 290.0, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, corr_t: false, ..d },
        ControlsSettings { t0: 216.65, accel_on: true, corr_t: false, ..d },
        ControlsSettings { t0: 290.0, govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, corr_t: false, ..d },
        at(0.0, 0.0, ControlsSettings { m0: 3.0, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr,
                                        speed_corr: SpeedCorr::None, ..d }),
    ] {
        out.push(req(s));
    }
    // Rung 99: the speed axis -- the pressure ratio at Mach 1.4 and 0.5 on the speed grid (a few tenths of a per cent
    // off), the same recomputed, the fan speed at Mach 0.15 (the grid's trend carried below it), the formula at Mach 0.05
    // from 700 K (the nozzle unchoked at the start), the schedule and the proportional law at Mach 1.4 on the grid, and
    // Mach 3 at sea level, beyond the Mach-3 table's reach.
    for s in [
        ControlsSettings { m0: 1.4, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d },
        ControlsSettings { m0: 0.5, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d },
        // These three moved under rung 100's defaults: pinned to rung 99's grid and formula column.
        ControlsSettings { m0: 1.4, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, speed_corr: SpeedCorr::Formula,
                           formula_choked: true, ..d },
        ControlsSettings { m0: 0.15, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, low_grid: false, ..d },
        ControlsSettings { m0: 0.05, from: 700.0, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr,
                           speed_corr: SpeedCorr::Formula, formula_choked: true, ..d },
        ControlsSettings { m0: 1.4, accel_on: true, ..d },
        ControlsSettings { m0: 1.4, govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, ..d },
        at(0.0, 0.0, ControlsSettings { m0: 3.0, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d }),
    ] {
        out.push(req(s));
    }
    // Rung 100: below choke -- the slow slices at Mach 0.1 from 700 K on both readings and the schedule there, the
    // pressure ratio's push worked out from the reading (Mach 0.05 from 700 K, unchoked at the start; Mach 1.4, choked;
    // Mach 1.4 from 700 K, the airflow guess left alone), the fan-speed formula at Mach 0.05 from 700 K (its table rows
    // named) and at Mach 1.4 (choked: rung 99's column), and its refusal at sea level from 600 K.
    let slow = |m0: f64, by: ThrustBy| ControlsSettings { m0, from: 700.0, govern: true, hold: Hold::Thrust, thrust_by: by, ..d };
    let fx = |s: ControlsSettings| ControlsSettings { speed_corr: SpeedCorr::Formula, ..s };
    for s in [
        slow(0.1, ThrustBy::Epr),
        slow(0.1, ThrustBy::N1),
        ControlsSettings { m0: 0.1, from: 700.0, accel_on: true, ..d },
        fx(slow(0.05, ThrustBy::Epr)),
        fx(ControlsSettings { m0: 1.4, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d }),
        fx(slow(1.4, ThrustBy::Epr)),
        fx(slow(0.05, ThrustBy::N1)),
        fx(ControlsSettings { m0: 1.4, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, ..d }),
        at(0.0, 0.0, fx(ControlsSettings { m0: 0.1, from: 600.0, ..slow(0.1, ThrustBy::N1) })),
    ] {
        out.push(req(s));
    }
    // Rung 101: the formula's airflow from the speed tables and from the exhaust sensor, on both readings — the sensor
    // instant (the pressure ratio's belief the true thrust) and slow (the third state), the fan-speed proportional law
    // under it (its own root), the customer bleed beside it (the overboard term), and the lag refused below its floor.
    let src = |s: ControlsSettings, a: AirflowFrom| ControlsSettings { airflow: a, ..fx(s) };
    let lag = |s: ControlsSettings, tau: f64| ControlsSettings { exhaust_lag: true, exhaust_tau: tau, ..src(s, AirflowFrom::Exhaust) };
    for s in [
        src(slow(1.4, ThrustBy::Epr), AirflowFrom::Grid),
        src(slow(0.1, ThrustBy::N1), AirflowFrom::Grid),
        src(slow(1.4, ThrustBy::Epr), AirflowFrom::Exhaust),
        src(ControlsSettings { customer: CustomerMode::Share, ..slow(0.5, ThrustBy::Epr) }, AirflowFrom::Exhaust),
        src(slow(1.4, ThrustBy::N1), AirflowFrom::Exhaust),
        src(ControlsSettings { speed_law: SpeedLaw::Prop, ..slow(1.4, ThrustBy::N1) }, AirflowFrom::Exhaust),
        lag(slow(1.4, ThrustBy::Epr), 1.0),
        lag(slow(0.5, ThrustBy::N1), 0.5),
        lag(slow(0.5, ThrustBy::Epr), 0.04),
    ] {
        out.push(req(s));
    }
    out
}

/// RUNG 102 (Part C): the Controls requests with the afterburner fitted — appended LAST to the whole list
/// ([`crate::sandbox::check_requests`]), so every earlier request keeps its index. Fitted and never lit; lit held, at
/// ×1.15 and by the rule; mid-slam with the redline (the stop the afterburner causes); the slow valve and the HP-speed
/// governor lit; and two refusals ("hold thrust" lit; a light-off off the march's grid).
pub fn check_ab_requests() -> Vec<String> {
    let d = ControlsSettings::defaults();
    let req = |s: ControlsSettings| jobj! { "op" => "controls", "controls" => s.to_json() }.dump_compact();
    let lit = |s: ControlsSettings| ControlsSettings { ab: Some(AbMode::Lit), ..s };
    [
        ControlsSettings { ab: Some(AbMode::Unlit), ..d },
        lit(d),
        lit(ControlsSettings { a8_ratio: 1.15, tt7: 1900.0, ..d }),
        lit(ControlsSettings { a8_match: true, ..d }),
        lit(ControlsSettings { light_at: 0.24, redline_on: true, ..d }),
        lit(ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, valve_lag_on: true, ..d }),
        lit(ControlsSettings { govern: true, hold: Hold::HpSpeed, a8_ratio: 1.3, ..d }),
        lit(ControlsSettings { govern: true, hold: Hold::Thrust, ..d }),
        lit(ControlsSettings { light_at: 0.25, ..d }),
    ].into_iter().map(req).collect()
}

/// `{"op":"controls_defaults"}` and `{"op":"controls","controls":{…}}` — the dispatcher
/// `sandbox::call` falls to.
pub fn call_op(op: &str, req: &Json) -> Option<Json> {
    let refusal = |m: &str| jobj! { "ok" => Json::Int(0), "reason" => m };
    Some(match op {
        "controls_defaults" => jobj! {
            "controls" => {
                // Rung 102's five keys are written to a reply only off their defaults; the page's knobs need them here.
                let mut j = ControlsSettings::defaults().to_json();
                if let Json::Obj(kv) = &mut j {
                    kv.extend([("ab".into(), Json::from("none")), ("Tt7".into(), Json::Float(AB_TT7)),
                               ("light_at".into(), Json::Float(AB_LIGHT_AT)), ("a8_ratio".into(), Json::Float(1.0)),
                               ("a8_match".into(), Json::Int(0))]);
                }
                j
            },
            "ds" => DS, "max_run" => MAX_RUN,
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
