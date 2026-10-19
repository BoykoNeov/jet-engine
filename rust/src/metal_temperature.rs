//! RUNG 93 — METAL TEMPERATURE: what the turbine's metal allows, and what cooling buys.
//!
//! Anchor (the derivation, the model and every pre-registered bar, committed before any of this code
//! existed): `docs/plans/rung93-anchor-metal-temperature.md`. Spec: `docs/rung93-spec.md`.
//!
//! # The correlation — Gauntner (1980), NASA TM-81453, used as published
//!
//! Per row of airfoils, the coolant over the gas entering the row is
//! `W/WG = FACTOR · 0.022 · (φ/(1−φ))^1.25`, `φ = (T − T_metal)/(T − T_coolant)`, and Gauntner's
//! subroutine COOLIT fixes what `T` is: the gas the row sees — the turbine inlet temperature for the
//! first vane row, `0.92 ×` the gas after the vane coolant has mixed in for the rotor (the relative
//! total temperature) — raised by a 150 °R safety margin and by the combustor's hot spot (pattern
//! factor 0.30 for the first stator, 0.13 for every other row), with the row's flow `×4/3` for the
//! endwall, shroud, disk and leakage. Rung 91's two streams ARE those two rows: vane air cools the
//! nozzle guide vanes, blade air the rotor.
//!
//! # It moves no gas
//!
//! A metal temperature is read off a point that already exists — `Tt4`, `Tt3` (the coolant), `Tt41`,
//! the burner gas and the two streams — so [`read`] is pure algebra beside rung 91's cooled point.
//! What is NEW is what the metal decides:
//!
//! * **Framing B — the same engine run hotter** ([`FixedEngine`]): on rung 91's fixed hardware, the
//!   hottest `Tt4` the metal allows at given shares ([`FixedEngine::try_max_tt4`]), and the shares
//!   that put BOTH metals at their limits at a given `Tt4` ([`FixedEngine::try_limited`]) — a fixed
//!   point through the plant, because on fixed hardware cooling slows the HP spool and lowers its own
//!   coolant's temperature.
//! * **Framing A — a fresh design at each setting** ([`try_cooled_design_run`],
//!   [`try_limited_design`]): rung 91's two mixers inside the design run. There `Tt3` is fixed by the
//!   pressure ratio and `f` by `(Tt3, Tt4)`, so the closed form is exact.
//!
//! # Reduce — by DISPATCH
//!
//! Nothing here runs unless asked: no metal limit, no change anywhere. The cooled design run with
//! no active cooling IS [`TwoSpoolEngine::run`] (dispatched, counted). **The perfect gas only**
//! (rung 91's mixers).

use std::cell::Cell;

use crate::components::Component;
use crate::engine::{Engine, FlightCondition, Performance};
use crate::gas::{Abort, FlowState, Gas};
use crate::map::ComponentMap;
use crate::turbine_cooling::{is_perfect_gas, mix, mixed_tt, try_score_nozzle, CooledResult, TurbineCooling,
                             TurbineCoolingMatcher};
use crate::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// =========================================================================================
// GAUNTNER'S CONSTANTS
// =========================================================================================

/// The safety margin added to the gas a row sees: 150 °R, in K.
pub const SAFETY: f64 = 150.0 * 5.0 / 9.0;
/// The hot-spot pattern factor of the first stator (the vane row).
pub const PATTERN_VANE: f64 = 0.30;
/// The pattern factor of every later row (the rotor).
pub const PATTERN_BLADE: f64 = 0.13;
/// The rotor sees `0.92 ×` the absolute total temperature — Gauntner's approximation of the
/// relative total temperature.
pub const ROTOR_RELATIVE: f64 = 0.92;
/// Endwall, shroud, disk and leakage: a row's stream is `4/3` of what its airfoils need.
pub const ENDWALL: f64 = 4.0 / 3.0;
/// The correlation's constant (a full-coverage film-cooled vane, FACTOR 1).
pub const COEFF: f64 = 0.022;
/// The correlation's exponent.
pub const EXPONENT: f64 = 1.25;

/// Gauntner's cooling technologies (his table I's relative cooling flow, FACTOR).
pub const TECHNOLOGIES: [(&str, f64); 4] = [
    ("convection", 2.0), ("coated convection", 1.5), ("full-coverage film", 1.0),
    ("transpiration", 0.8),
];

/// Degrees Rankine to kelvin.
pub fn rankine(r: f64) -> f64 { r * 5.0 / 9.0 }

/// A row of airfoils.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// The first nozzle guide vanes — cooled by rung 91's VANE air.
    Vane,
    /// The HP rotor blades — cooled by rung 91's BLADE air.
    Blade,
}

impl Row {
    pub fn pattern(self) -> f64 {
        match self { Row::Vane => PATTERN_VANE, Row::Blade => PATTERN_BLADE }
    }
    pub fn name(self) -> &'static str {
        match self { Row::Vane => "vane", Row::Blade => "blade" }
    }
}

/// Gauntner's allowable bulk metal temperature, K: `(10·YEAR − 17640) − (100·log₁₀ LIFE − 400)` °R
/// for the vane, the blade 100 °R lower (17740) — "the year of material technology" and the life in
/// hours (his default 10 000).
pub fn allowable(row: Row, year: f64, life_hours: f64) -> f64 {
    let off = match row { Row::Vane => 17640.0, Row::Blade => 17740.0 };
    rankine((10.0 * year - off) - (100.0 * life_hours.log10() - 400.0))
}

/// The metal limits and the cooling technology.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetalLimit {
    /// Gauntner's FACTOR — the coolant a technology needs relative to full-coverage film.
    pub factor: f64,
    /// The allowable vane and blade metal temperatures, K.
    pub vane: f64,
    pub blade: f64,
}

impl MetalLimit {
    pub fn new(factor: f64, vane: f64, blade: f64) -> Self {
        assert!(factor.is_finite() && factor > 0.0, "rung-93 cooling technology FACTOR must be > 0: got {factor}");
        assert!(vane > 0.0 && blade > 0.0, "rung-93 metal limits must be positive temperatures: got {vane}, {blade}");
        MetalLimit { factor, vane, blade }
    }

    /// Gauntner's 1985 materials at 10 000 hours: 1227.8 K (vane), 1172.2 K (blade).
    pub fn gauntner_1985(factor: f64) -> Self {
        MetalLimit::new(factor, allowable(Row::Vane, 1985.0, 1.0e4), allowable(Row::Blade, 1985.0, 1.0e4))
    }

    pub fn of(&self, row: Row) -> f64 {
        match row { Row::Vane => self.vane, Row::Blade => self.blade }
    }
}

// =========================================================================================
// THE CORRELATION, FORWARD AND INVERSE
// =========================================================================================

/// Gauntner's `W/WG` for the AIRFOIL at effectiveness `phi` (before the endwall `4/3`).
pub fn coolant_flow(factor: f64, phi: f64) -> f64 {
    factor * COEFF * (phi / (1.0 - phi)).powf(EXPONENT)
}

/// The inverse: the effectiveness an airfoil coolant/gas `w_airfoil` buys. Zero coolant is zero.
pub fn effectiveness(factor: f64, w_airfoil: f64) -> f64 {
    if w_airfoil <= 0.0 {
        return 0.0;
    }
    let y = (w_airfoil / (COEFF * factor)).powf(1.0 / EXPONENT);
    y / (1.0 + y)
}

/// The gas a row sees, before the hot spot: `t_row` is `Tt4` for the vane, the rotor-inlet `Tt41`
/// for the blade (scaled to the relative frame), plus the safety margin.
pub fn gas_seen(row: Row, t_row: f64) -> f64 {
    match row { Row::Vane => t_row + SAFETY, Row::Blade => ROTOR_RELATIVE * t_row + SAFETY }
}

/// The hot spot: `T + P(T − Tc)` — Gauntner's `φ_hot = (P + φ)/(P + 1)` written as a temperature.
pub fn hot_spot(row: Row, t_row: f64, tc: f64) -> f64 {
    let t = gas_seen(row, t_row);
    t + row.pattern() * (t - tc)
}

/// The row's metal temperature: the hot spot cooled to the effectiveness its stream buys. `w_row`
/// is the row's whole stream over the gas entering it (endwall included). No coolant: the hot spot.
pub fn metal(row: Row, t_row: f64, tc: f64, w_row: f64, factor: f64) -> f64 {
    let hot = hot_spot(row, t_row, tc);
    hot - effectiveness(factor, w_row / ENDWALL) * (hot - tc)
}

/// The stream (over the gas entering the row, endwall included) that puts the row's metal AT `lim`:
/// `y = (T_hot − Tc)/(lim − Tc) − 1`, `w = (4/3)·0.022·F·y^1.25`; 0 when the hot spot is already
/// within the limit. Refused when the limit is at or below the coolant: no air can hold it.
pub fn try_need(row: Row, t_row: f64, tc: f64, lim: f64, factor: f64) -> Result<f64, Abort> {
    if !(lim > tc) {
        return Err(Abort(format!(
            "rung-93: the {} metal limit ({lim:.0} K) is at or below the cooling air's temperature \
             ({tc:.0} K) - no amount of that air can hold the metal there.", row.name())));
    }
    let y = (hot_spot(row, t_row, tc) - tc) / (lim - tc) - 1.0;
    Ok(if y <= 0.0 { 0.0 } else { ENDWALL * COEFF * factor * y.powf(EXPONENT) })
}

// =========================================================================================
// READING A POINT
// =========================================================================================

/// What the metal model reads off a point: the burner exit, the coolant, the rotor inlet, and each
/// row's stream over the gas entering it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetalInputs {
    pub tt4: f64,
    pub tt3: f64,
    pub tt41: f64,
    pub w_vane: f64,
    pub w_blade: f64,
}

/// The two flows a station table carries for the rows: the gas leaving the burner (station 4) and
/// the gas entering the rotor (station 41, absent when no vane air mixes in).
fn inputs_from(stations: &[(&'static str, FlowState)], mdot_vane: f64, mdot_blade: f64) -> MetalInputs {
    let st = |l: &str| stations.iter().find(|(x, _)| *x == l).map(|(_, s)| *s);
    let s4 = st("4").expect("station 4");
    let s41 = st("41").unwrap_or(s4);
    MetalInputs {
        tt4: s4.tt, tt3: st("3").expect("station 3").tt, tt41: s41.tt,
        w_vane: mdot_vane / s4.mdot, w_blade: mdot_blade / s41.mdot,
    }
}

impl MetalInputs {
    /// Off a rung-91 point (an inactive one has no streams: `Tt41 = Tt4`).
    pub fn of_cooled(r: &CooledResult) -> Self {
        match r.booking {
            Some(k) => inputs_from(&r.base.base.stations, k.mdot_vane, k.mdot_blade),
            None => inputs_from(&r.base.base.stations, 0.0, 0.0),
        }
    }

    /// Off a cooled design run.
    pub fn of_design(d: &CooledDesign) -> Self { inputs_from(&d.stations, d.mdot_vane, d.mdot_blade) }

    /// From the temperatures and SHARES alone — for a point that carries no station table (a transient
    /// instant): per HPC air the burner gets `1 − c − ε`, its gas is `G = (1 − c − ε)(1+f)`, and the
    /// rotor's inlet is rung 91's mixer 1 on those flows. On a steady point it agrees with
    /// [`of_cooled`](Self::of_cooled) to round-off (`tests/sandbox_cooling.rs`).
    pub fn of_shares(gas: &Gas, tt4: f64, tt3: f64, f: f64, customer: f64, cool: TurbineCooling) -> Self {
        let burner = 1.0 - customer - cool.vane - cool.blade;
        let g = burner * (1.0 + f);
        let f41 = f * burner / (burner + cool.vane);
        MetalInputs {
            tt4, tt3, tt41: mixed_tt(gas, g, tt4, f, cool.vane, tt3, f41),
            w_vane: cool.vane / g, w_blade: cool.blade / (g + cool.vane),
        }
    }
}

/// Both rows' metal temperatures at a point, and the limits they are held to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetalReading {
    pub vane: f64,
    pub blade: f64,
    pub limit: MetalLimit,
}

impl MetalReading {
    /// How far the hotter row (relative to its own limit) sits above its limit; negative when both
    /// are within them.
    pub fn over(&self) -> f64 { (self.vane - self.limit.vane).max(self.blade - self.limit.blade) }

    /// The row closest to (or furthest past) its limit.
    pub fn binding(&self) -> Row {
        if self.vane - self.limit.vane >= self.blade - self.limit.blade { Row::Vane } else { Row::Blade }
    }
}

pub fn read(inp: &MetalInputs, lim: &MetalLimit) -> MetalReading {
    MetalReading {
        vane: metal(Row::Vane, inp.tt4, inp.tt3, inp.w_vane, lim.factor),
        blade: metal(Row::Blade, inp.tt41, inp.tt3, inp.w_blade, lim.factor),
        limit: *lim,
    }
}

/// The two streams each row needs AT THIS POINT's temperatures, as shares of the HP compressor's
/// air: with `w` the needed stream over each row's gas and `A = 1 + f`, the gas per HPC air is
/// `G = (1 − ε₁ − ε₂)A`, so `ε₁ = w_v G` and `ε₂ = w_b (G + ε₁)` — linear, solved exactly.
fn shares_for(w_v: f64, w_b: f64, f: f64) -> (f64, f64) {
    let a = 1.0 + f;
    let det = (1.0 + w_v * a) * (1.0 + w_b * a) - w_v * a * (w_b * a - w_b);
    (w_v * a / det, w_b * a * (1.0 + w_v) / det)
}

/// The needed streams at a point's temperatures.
fn try_needs(inp: &MetalInputs, lim: &MetalLimit) -> Result<(f64, f64), Abort> {
    Ok((try_need(Row::Vane, inp.tt4, inp.tt3, lim.vane, lim.factor)?,
        try_need(Row::Blade, inp.tt41, inp.tt3, lim.blade, lim.factor)?))
}

/// The bound at a converged point, in words (rung 91's caps).
fn check_shares(e1: f64, e2: f64, tt4: f64) -> Result<(), Abort> {
    if !(e1 < TurbineCooling::SHARE_MAX && e2 < TurbineCooling::SHARE_MAX
         && e1 + e2 < TurbineCooling::TOTAL_MAX) {
        return Err(Abort(format!(
            "rung-93: at {tt4:.0} K the metal needs {:.1} % vane air and {:.1} % blade air - past the \
             {:.0} % a stream may take: this technology cannot hold the metal there.",
            100.0 * e1, 100.0 * e2, 100.0 * TurbineCooling::SHARE_MAX)));
    }
    Ok(())
}

/// The largest share a TRIAL may build (`TurbineCooling::new` panics at the cap; the bound is
/// checked at the root — rung 90's lesson).
const TRIAL_MAX: f64 = TurbineCooling::SHARE_MAX * (1.0 - 1e-9);

/// How many passes a fixed point may take, and its tolerance on a share.
pub const MAX_PASSES: usize = 200;
pub const SHARE_TOL: f64 = 1e-12;

// =========================================================================================
// FRAMING B — THE SAME ENGINE RUN HOTTER
// =========================================================================================

/// Rung 91's fixed hardware, with the cooling left for this rung to set.
pub struct FixedEngine {
    /// The design engine, with NO turbine cooling of its own (this rung sets it).
    pub design: TwoSpoolEngine,
    pub flight_design: FlightCondition,
    pub mdot_design: f64,
    pub map_lp: ComponentMap,
    pub map_hp: ComponentMap,
    /// RUNG 103: its matchers re-solve an unchoked throttle instead of refusing (the Cooling view sets it; `false`
    /// from [`FixedEngine::new`], so rungs 93/94 read exactly as before).
    pub below_choke: bool,
    /// The below-idle floor (sandbox plan § 38) on its matchers — `false` from [`FixedEngine::new`]; set with
    /// `below_choke` by [`FixedEngine::for_pages`].
    pub idle_floor: bool,
}

/// The hottest `Tt4` the metal allows at given shares.
#[derive(Clone, Debug)]
pub struct MaxTt4 {
    pub tt4: f64,
    pub binding: Row,
    pub reading: MetalReading,
    pub point: CooledResult,
}

/// The cooling that puts both metals at their limits, and the point it lands on.
#[derive(Clone, Debug)]
pub struct Limited {
    pub vane: f64,
    pub blade: f64,
    pub passes: usize,
    pub reading: MetalReading,
    pub inputs: MetalInputs,
    pub point: CooledResult,
}

impl FixedEngine {
    pub fn new(
        design: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64, map_lp: ComponentMap,
        map_hp: ComponentMap,
    ) -> Self {
        assert!(is_perfect_gas(&design.gas),
                "rung-93's metal model rides on rung 91's cooled engine: the perfect gas only.");
        assert!(design.cooling.is_none(), "rung-93's fixed engine sets the turbine cooling itself: pass \
                                           the design engine without it.");
        assert!(design.customer.is_none() && design.offtake.is_none(),
                "rung-93's fixed engine is scoped without the customer bleed or a shaft load.");
        FixedEngine { design, flight_design, mdot_design, map_lp, map_hp, below_choke: false, idle_floor: false }
    }

    /// The sandbox views' engine: rung 103's below-choke re-solve and the below-idle floor (plan § 38), together.
    pub fn for_pages(self) -> Self { FixedEngine { below_choke: true, idle_floor: true, ..self } }

    pub fn matcher(&self, cool: TurbineCooling) -> TurbineCoolingMatcher {
        let m = TurbineCoolingMatcher::new(self.design.clone().with_turbine_cooling(cool), self.flight_design,
                                           self.mdot_design, self.map_lp, self.map_hp, 0.0);
        m.core.base.below_choke.set(self.below_choke);
        m.core.base.idle_floor.set(self.idle_floor);
        m
    }

    pub fn try_point(&self, cool: TurbineCooling, flight: &FlightCondition, tt4: f64)
                     -> Result<CooledResult, Abort> {
        self.matcher(cool).try_match_point(flight, tt4)
    }

    /// The hottest `Tt4` in `[lo, hi]` at which both metals hold: bisected on the hotter row's
    /// excess. Refused in words when the metal holds across the whole range, or nowhere in it.
    pub fn try_max_tt4(
        &self, cool: TurbineCooling, lim: &MetalLimit, flight: &FlightCondition, lo: f64, hi: f64,
    ) -> Result<MaxTt4, Abort> {
        let m = self.matcher(cool);
        let at = |t: f64| -> Result<(f64, CooledResult, MetalReading), Abort> {
            let p = m.try_match_point(flight, t)?;
            let r = read(&MetalInputs::of_cooled(&p), lim);
            Ok((r.over(), p, r))
        };
        let (glo, ghi) = (at(lo)?.0, at(hi)?.0);
        if !(glo < 0.0) {
            return Err(Abort(format!(
                "rung-93: even at {lo:.0} K the metal is {glo:.1} K over its limit with this cooling.")));
        }
        if !(ghi > 0.0) {
            return Err(Abort(format!(
                "rung-93: the metal holds all the way to {hi:.0} K with this cooling.")));
        }
        let (mut a, mut b) = (lo, hi);
        for _ in 0..MAX_PASSES {
            let mid = 0.5 * (a + b);
            if at(mid)?.0 < 0.0 { a = mid; } else { b = mid; }
            if b - a <= 1e-9 {
                break;
            }
        }
        let (_, point, reading) = at(a)?;
        Ok(MaxTt4 { tt4: a, binding: reading.binding(), reading, point })
    }

    /// The shares that put BOTH metals at their limits at `tt4`: a fixed point through the plant
    /// (each pass matches the engine at the current shares, reads its temperatures, and solves for
    /// the shares those temperatures need). It contracts because a share of cooling moves its own
    /// coolant's temperature only weakly (anchor § 0 G2).
    pub fn try_limited(&self, lim: &MetalLimit, flight: &FlightCondition, tt4: f64) -> Result<Limited, Abort> {
        let (mut e1, mut e2) = (0.0_f64, 0.0_f64);
        for pass in 1..=MAX_PASSES {
            let cool = TurbineCooling::new(e1.min(TRIAL_MAX), e2.min(TRIAL_MAX));
            let point = self.try_point(cool, flight, tt4)?;
            let inputs = MetalInputs::of_cooled(&point);
            let (w_v, w_b) = try_needs(&inputs, lim)?;
            let f = point.base.base.station("4").far;
            let (n1, n2) = shares_for(w_v, w_b, f);
            if (n1 - e1).abs() <= SHARE_TOL && (n2 - e2).abs() <= SHARE_TOL {
                check_shares(n1, n2, tt4)?;
                let reading = read(&inputs, lim);
                return Ok(Limited { vane: cool.vane, blade: cool.blade, passes: pass, reading, inputs, point });
            }
            e1 = n1;
            e2 = n2;
        }
        Err(Abort(format!("rung-93: the cooling that holds both metals at {tt4:.0} K did not settle.")))
    }
}

// =========================================================================================
// FRAMING A — A FRESH DESIGN AT EACH SETTING
// =========================================================================================

thread_local! {
    static DESIGN_ACTIVE: Cell<u64> = const { Cell::new(0) };
    static DESIGN_INACTIVE: Cell<u64> = const { Cell::new(0) };
}

/// How many cooled design runs ran on this thread — `(active, dispatched to the plain design run)`.
pub fn counts() -> [u64; 2] { [&DESIGN_ACTIVE, &DESIGN_INACTIVE].map(|k| k.with(|x| x.get())) }

pub fn reset_counts() {
    for k in [&DESIGN_ACTIVE, &DESIGN_INACTIVE] {
        k.with(|x| x.set(0));
    }
}

/// A design-point run with turbine cooling: the stations (with 41 and 44 when cooled), the exhaust
/// scored per NOZZLE air (`performance`, at the diluted `f45`), and per INLET air.
#[derive(Clone, Debug)]
pub struct CooledDesign {
    pub stations: Vec<(&'static str, FlowState)>,
    pub performance: Performance,
    pub v0: f64,
    /// Thrust per inlet air, N·s/kg — the cooling air leaves through the nozzle, so the nozzle passes
    /// exactly the inlet air.
    pub st_inlet: f64,
    /// Fuel per unit thrust, kg/(N·s).
    pub tsfc_inlet: f64,
    pub mdot_air: f64,
    pub mdot_burner: f64,
    pub mdot_vane: f64,
    pub mdot_blade: f64,
}

impl CooledDesign {
    pub fn station(&self, label: &str) -> &FlowState {
        &self.stations.iter().find(|(l, _)| *l == label)
            .unwrap_or_else(|| panic!("no station {label}")).1
    }
}

/// The fresh design `build_two_spool_turbojet(gas, π_LPC, π_HPC, Tt4, p_ambient, losses)` with its
/// turbines cooled by `cool`: [`TwoSpoolEngine::run`] with rung 91's two mixers inside (vane air
/// after the A4 throat, blade air after the HP rotor) and each turbine's work per kg of the gas it
/// passes. **Inactive cooling ⇒ [`TwoSpoolEngine::run`] itself** (dispatched).
#[allow(clippy::too_many_arguments)]
pub fn try_cooled_design_run(
    gas: &Gas, pi_lpc: f64, pi_hpc: f64, tt4: f64, p_ambient: f64, losses: TwoSpoolLosses,
    cool: Option<TurbineCooling>, flight: &FlightCondition, mdot: f64,
) -> Result<CooledDesign, Abort> {
    let eng = build_two_spool_turbojet(gas.clone(), pi_lpc, pi_hpc, tt4, p_ambient, losses);
    let Some(cool) = cool.filter(|c| c.is_active()) else {
        DESIGN_INACTIVE.with(|x| x.set(x.get() + 1));
        let r = eng.run(flight, mdot);
        return Ok(CooledDesign {
            performance: r.performance, v0: r.v0, st_inlet: r.performance.specific_thrust,
            tsfc_inlet: r.performance.tsfc, mdot_air: mdot, mdot_burner: mdot, mdot_vane: 0.0,
            mdot_blade: 0.0, stations: r.stations,
        });
    };
    DESIGN_ACTIVE.with(|x| x.set(x.get() + 1));
    assert!(is_perfect_gas(gas), "rung-93's cooled design run uses rung 91's mixers: the perfect gas only.");
    check_shares(cool.vane, cool.blade, tt4)?;
    let by = |label: &str| -> Component {
        eng.components.iter().find(|(l, _)| *l == label)
            .unwrap_or_else(|| panic!("no component at station {label}")).1
    };
    let (state0, v0) = Engine::new(gas.clone(), Vec::new(), losses.eta_m).freestream(flight, mdot);
    let s2 = by("2").apply(&state0, gas);
    let s25 = by("25").apply(&s2, gas);
    let s3 = by("3").apply(&s25, gas);
    let (mdot_vane, mdot_blade) = (cool.vane * s3.mdot, cool.blade * s3.mdot);
    let mdot_burner = s3.mdot - mdot_vane - mdot_blade;
    let s4 = by("4").apply(&FlowState { mdot: mdot_burner, ..s3 }, gas);
    let s41 = mix(&s4, mdot_vane, s3.tt, s3.pt, gas);                    // <- MIXER 1 (vane)
    let turbine = |label: &str| match by(label) {
        Component::Turbine(t) => t,
        _ => panic!("station {label} must be a Turbine"),
    };
    // HP shaft: the rotor drives the HPC air with the gas it passes, G + eps1 per HPC air.
    let dh_hpc = gas.h_c(s3.tt) - gas.h_c(s25.tt);
    let s44 = turbine("45").apply(&s41, gas, dh_hpc * s3.mdot / (losses.eta_m * s41.mdot));
    let p_hp = losses.eta_m * s41.mdot * (gas.h_t(s41.tt, s41.far) - gas.h_t(s44.tt, s44.far));
    assert!((p_hp - s3.mdot * dh_hpc).abs() < 1e-6 * s3.mdot * dh_hpc, "rung-93 cooled design: HP shaft does not close");
    let s45 = mix(&s44, mdot_blade, s3.tt, s3.pt, gas);                  // <- MIXER 2 (blade)
    // LP shaft: the LPT drives the LPC with the burner gas and both coolants.
    let dh_lpc = gas.h_c(s25.tt) - gas.h_c(s2.tt);
    let s5 = turbine("5").apply(&s45, gas, dh_lpc * s2.mdot / (losses.eta_m * s45.mdot));
    let p_lp = losses.eta_m * s45.mdot * (gas.h_t(s45.tt, s45.far) - gas.h_t(s5.tt, s5.far));
    assert!((p_lp - s2.mdot * dh_lpc).abs() < 1e-6 * s2.mdot * dh_lpc, "rung-93 cooled design: LP shaft does not close");
    let exit = match by("9") {
        Component::Nozzle(n) => n.try_apply(&s5, gas)?,
        _ => panic!("station 9 must be a Nozzle"),
    };
    let stations = vec![
        ("0", state0), ("2", s2), ("25", s25), ("3", s3), ("4", s4), ("41", s41), ("44", s44),
        ("45", s45), ("5", s5), ("9", exit.state),
    ];
    let performance = try_score_nozzle(&stations, s45.far, gas, v0, exit.t9, exit.v9, exit.p9,
                                       flight.p0, gas.hpr())?;
    // The nozzle passes the burner air AND both coolants: exactly the inlet air at design.
    let st_inlet = performance.specific_thrust;
    let tsfc_inlet = (mdot_burner * s4.far / mdot) / st_inlet;
    Ok(CooledDesign { stations, performance, v0, st_inlet, tsfc_inlet, mdot_air: mdot, mdot_burner,
                      mdot_vane, mdot_blade })
}

/// A fresh design whose cooling puts both metals at their limits.
#[derive(Clone, Debug)]
pub struct LimitedDesign {
    pub vane: f64,
    pub blade: f64,
    pub passes: usize,
    pub reading: MetalReading,
    pub inputs: MetalInputs,
    pub run: CooledDesign,
}

/// The cooling that puts both metals at their limits on a FRESH design at `tt4` — the same fixed
/// point as [`FixedEngine::try_limited`], on the cooled design run (where only `Tt41` moves).
#[allow(clippy::too_many_arguments)]
pub fn try_limited_design(
    gas: &Gas, pi_lpc: f64, pi_hpc: f64, tt4: f64, p_ambient: f64, losses: TwoSpoolLosses,
    lim: &MetalLimit, flight: &FlightCondition, mdot: f64,
) -> Result<LimitedDesign, Abort> {
    let (mut e1, mut e2) = (0.0_f64, 0.0_f64);
    for pass in 1..=MAX_PASSES {
        let cool = TurbineCooling::new(e1.min(TRIAL_MAX), e2.min(TRIAL_MAX));
        let run = try_cooled_design_run(gas, pi_lpc, pi_hpc, tt4, p_ambient, losses, Some(cool), flight, mdot)?;
        let inputs = MetalInputs::of_design(&run);
        let (w_v, w_b) = try_needs(&inputs, lim)?;
        let (n1, n2) = shares_for(w_v, w_b, run.station("4").far);
        if (n1 - e1).abs() <= SHARE_TOL && (n2 - e2).abs() <= SHARE_TOL {
            check_shares(n1, n2, tt4)?;
            let reading = read(&inputs, lim);
            return Ok(LimitedDesign { vane: cool.vane, blade: cool.blade, passes: pass, reading, inputs, run });
        }
        e1 = n1;
        e2 = n2;
    }
    Err(Abort(format!("rung-93: the cooling that holds both metals on a fresh design at {tt4:.0} K did not settle.")))
}
