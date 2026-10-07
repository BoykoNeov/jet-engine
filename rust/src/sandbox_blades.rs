//! The web sandbox, slice 2 — **SIZE THE BLADES** (`docs/plans/sandbox-plan.md` § 11). NOT a rung.
//!
//! Rung 85 (`crate::blade_speed`) as knobs: the user picks how the compressor blades are designed and
//! watches the stage counts, blade speeds and the REDLINE come out, then moves a stator lever and
//! sees whether a shaft passes its redline. Every result is shown; nothing is a pass/fail verdict
//! (the project's sandbox direction, user 2026-10-06).
//!
//! **A second engine, said plainly** (user decision, plan § 11.7 Q1). `blade_speed::build` sizes a
//! TWO-spool engine; the page's *Design* view is single-spool. So this view has its own engine —
//! rung 85's rig (rung 55's): `π_LPC` 3 × `π_HPC` 6, `Tt4` 1500, a convergent nozzle, the panels'
//! two-spool losses, at the panels' flight (250 K, 50 kPa, Mach 0.85) — with its pressure ratios and
//! design `Tt4` as knobs.
//!
//! **Two gases** (Q2): the rig's perfect gas (cold 1.4 / 1004, hot 1.3 / 1239 — `tests/rung85.rs`'s,
//! bit for bit) and the thermally-perfect one, on which the sizing reads `γ, cp` at each face
//! (`blade_speed::build`'s `face_props`). The row-by-row stack still splits the pressure rise with the
//! scalar `γ` (rung 55, `stage.rs`); the page says so.
//!
//! **Requests.** `blade_size` sizes both spools (microseconds) and sweeps the airflow level for the
//! staircase chart; `blade_lever` reads ONE throttle of one lever's schedule (milliseconds on the perfect gas, ~1 s in the browser
//! on the thermally perfect one), so the page
//! streams a grid ([`lever_grid`]) as slice 3 streams its running line. A lever point whose
//! lever-at-design match fails (the nozzle unchokes — the two-spool model's open seam) is refused in
//! plain words BEFORE the schedule runs, through the matcher's own fallible twin; anything else
//! panics as the model always has, and the page shows [`explain_blades`]'s words.

use crate::blade_speed::{airflow_wall, build, shape_maps, size, BladeKnobs, Binding, Droop, Lever,
                         Machine, Sizing, SizingError, Verdict, SHAPES, TI64_DENSITY, TI64_YIELD};
use crate::components::Component;
use crate::engine::{Engine, FlightCondition};
use crate::gas::{Gas, GasSpec};
use crate::jobj;
use crate::panels::flight;
use crate::sandbox::{explain, DoesNotRun};
use crate::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use crate::visuals::Json;

/// The two gases this view offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BladeGas {
    /// Rung 85's rig: calorically perfect, cold 1.4 / 1004, hot 1.3 / 1239.
    Perfect,
    /// NASA-polynomial air and lean products (rung 3).
    ThermallyPerfect,
}

impl BladeGas {
    pub const ALL: [BladeGas; 2] = [BladeGas::Perfect, BladeGas::ThermallyPerfect];

    pub fn key(self) -> &'static str {
        match self { BladeGas::Perfect => "perfect", BladeGas::ThermallyPerfect => "thermally_perfect" }
    }

    fn from_key(k: &str) -> Result<Self, String> {
        BladeGas::ALL.into_iter().find(|g| g.key() == k)
            .ok_or_else(|| format!("unknown gas {k:?} (the blade view offers \"perfect\" and \"thermally_perfect\")"))
    }

    pub fn gas(self) -> Gas {
        match self {
            BladeGas::Perfect => {
                let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
                Gas::new(GasSpec { gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
                                   gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
                                   hpr: 42.8e6, ..GasSpec::default() })
            }
            BladeGas::ThermallyPerfect => Gas::thermally_perfect(),
        }
    }
}

/// The panels' two-spool losses (`panels::twospool::ts_losses`), with the convergent nozzle every
/// two-spool matcher needs.
pub fn rig_losses() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
                     eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
                     p_exit: None, nozzle_convergent: true }
}

/// Every knob of the blade view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BladeSettings {
    pub gas: BladeGas,
    pub pi_lpc: f64,
    pub pi_hpc: f64,
    /// The DESIGN turbine-inlet temperature, K — the engine the blades are sized on.
    pub tt4: f64,
    /// Index into [`SHAPES`].
    pub shape: usize,
    /// Each spool's blade knobs. The page links them by default ("same for both", Q4).
    pub lp: BladeKnobs,
    pub hp: BladeKnobs,
    pub lever: Lever,
    /// The spool whose stators the lever moves.
    pub spool: Spool,
}

impl BladeSettings {
    /// Rung 85's rig, at the default knobs.
    pub fn defaults() -> Self {
        BladeSettings { gas: BladeGas::Perfect, pi_lpc: 3.0, pi_hpc: 6.0, tt4: 1500.0, shape: 0,
                        lp: BladeKnobs::default(), hp: BladeKnobs::default(),
                        lever: Lever::Lumped, spool: Spool::Lp }
    }

    pub fn flight(&self) -> FlightCondition { flight() }

    pub fn design(&self) -> TwoSpoolEngine {
        build_two_spool_turbojet(self.gas.gas(), self.pi_lpc, self.pi_hpc, self.tt4, self.flight().p0, rig_losses())
    }

    pub fn to_json(&self) -> Json {
        jobj! {
            "gas" => self.gas.key(), "pi_lpc" => self.pi_lpc, "pi_hpc" => self.pi_hpc, "Tt4" => self.tt4,
            "shape" => SHAPES[self.shape], "lp" => knobs_json(&self.lp), "hp" => knobs_json(&self.hp),
            "lever" => lever_key(self.lever), "spool" => spool_key(self.spool),
        }
    }

    /// Read blade settings; a missing key keeps its default.
    pub fn from_json(j: &Json) -> Result<Self, String> {
        let mut s = BladeSettings::defaults();
        num(j, "pi_lpc", &mut s.pi_lpc)?;
        num(j, "pi_hpc", &mut s.pi_hpc)?;
        num(j, "Tt4", &mut s.tt4)?;
        if let Some(g) = text(j, "gas")? { s.gas = BladeGas::from_key(&g)?; }
        if let Some(sh) = text(j, "shape")? {
            s.shape = SHAPES.iter().position(|&k| k == sh).ok_or_else(|| format!("unknown map shape {sh:?}"))?;
        }
        if let Some(l) = text(j, "lever")? {
            s.lever = [Lever::Lumped, Lever::AllRows, Lever::FrontRow].into_iter().find(|&v| lever_key(v) == l)
                .ok_or_else(|| format!("unknown lever {l:?}"))?;
        }
        if let Some(sp) = text(j, "spool")? {
            s.spool = [Spool::Lp, Spool::Hp].into_iter().find(|&v| spool_key(v) == sp)
                .ok_or_else(|| format!("unknown spool {sp:?}"))?;
        }
        if let Some(k) = j.get("lp") { s.lp = knobs_from_json(k, s.lp)?; }
        if let Some(k) = j.get("hp") { s.hp = knobs_from_json(k, s.hp)?; }
        Ok(s)
    }
}

fn num(j: &Json, k: &str, into: &mut f64) -> Result<(), String> {
    match j.get(k) {
        None => Ok(()),
        Some(Json::Float(x)) => { *into = *x; Ok(()) }
        Some(Json::Int(n)) => { *into = *n as f64; Ok(()) }
        Some(other) => Err(format!("setting {k:?} must be a number, got {other:?}")),
    }
}

fn text(j: &Json, k: &str) -> Result<Option<String>, String> {
    match j.get(k) {
        None => Ok(None),
        Some(Json::Str(t)) => Ok(Some(t.clone())),
        Some(other) => Err(format!("setting {k:?} must be text, got {other:?}")),
    }
}

pub fn lever_key(l: Lever) -> &'static str {
    match l { Lever::Lumped => "lumped", Lever::AllRows => "all_rows", Lever::FrontRow => "front_row" }
}

pub fn spool_key(s: Spool) -> &'static str {
    match s { Spool::Lp => "lp", Spool::Hp => "hp" }
}

fn droop_key(d: Droop) -> &'static str {
    match d { Droop::WithBladeSpeed => "blade_speed", Droop::WithRowWork => "row_work" }
}

fn knobs_json(k: &BladeKnobs) -> Json {
    jobj! {
        "h" => k.h, "m_rel_lim" => k.m_rel_lim, "sigma_over_rho" => k.sigma_over_rho,
        "overspeed" => k.overspeed, "phi_d" => k.phi_d, "lambda" => k.lambda, "droop" => droop_key(k.droop),
    }
}

fn knobs_from_json(j: &Json, mut k: BladeKnobs) -> Result<BladeKnobs, String> {
    num(j, "h", &mut k.h)?;
    num(j, "m_rel_lim", &mut k.m_rel_lim)?;
    num(j, "sigma_over_rho", &mut k.sigma_over_rho)?;
    num(j, "overspeed", &mut k.overspeed)?;
    num(j, "phi_d", &mut k.phi_d)?;
    num(j, "lambda", &mut k.lambda)?;
    if let Some(d) = text(j, "droop")? {
        k.droop = [Droop::WithBladeSpeed, Droop::WithRowWork].into_iter().find(|&v| droop_key(v) == d)
            .ok_or_else(|| format!("unknown droop switch {d:?}"))?;
    }
    Ok(k)
}

/// Ti-6Al-4V's specific strength (AMS 4911 minimum yield over density), m²/s² — the one cited
/// material preset (Q3).
pub const TI64_SPECIFIC: f64 = TI64_YIELD / TI64_DENSITY;

/// The checks made before the engine is built: numbers, pressure ratios above 1, and the burner
/// able to ADD heat (the inlet and both compressors run cheaply first, as slice 1's [`precheck`]
/// does for its one compressor).
///
/// [`precheck`]: crate::sandbox::precheck
pub fn blade_precheck(s: &BladeSettings) -> Result<(), DoesNotRun> {
    let no = |m: String| Err(DoesNotRun(m));
    let named = [("pi_lpc", s.pi_lpc), ("pi_hpc", s.pi_hpc), ("Tt4", s.tt4)];
    if let Some((k, _)) = named.iter().find(|(_, v)| !v.is_finite()) {
        return no(format!("Setting {k} is not a number."));
    }
    if s.pi_lpc <= 1.0 || s.pi_hpc <= 1.0 {
        return no("Both compressor pressure ratios must be above 1: at 1 a compressor does no work, \
                   so it has no blades to size.".into());
    }
    let d = s.design();
    let fs = Engine::new(d.gas.clone(), Vec::new(), d.eta_m);
    let (mut st, _) = fs.try_freestream(&s.flight(), 1.0)
        .map_err(|e| DoesNotRun(format!("The freestream could not be formed: {}", e.0)))?;
    for (label, c) in &d.components {
        if *label == "4" { break; }
        if let Component::Inlet(_) | Component::Compressor(_) = c {
            st = c.apply(&st, &d.gas);
        }
    }
    if s.tt4 <= st.tt {
        return no(format!(
            "The turbine-inlet temperature ({:.0} K) is not above the compressor-exit temperature ({:.0} K), \
             so the burner would have to cool the air. Raise Tt4 or lower the pressure ratios.", s.tt4, st.tt));
    }
    Ok(())
}

/// Plain words for a [`SizingError`] — the sizing's own refusals, which never panic.
pub fn sizing_words(e: &SizingError, spool: &str) -> String {
    match e {
        SizingError::Knob(name, v) => {
            let what = match *name {
                "h" => "hub-to-tip ratio must be above 0 and below 1",
                "m_rel_lim" => "airflow level (design tip Mach) must be above 0 and below 3",
                "sigma_over_rho" => "material strength must be above 0",
                "overspeed" => "overspeed factor must be at least 1",
                "phi_d" => "design flow coefficient must be above 0 and below 2",
                "lambda" => "rounding knob must be between 0 and 1",
                _ => "knob is out of range",
            };
            format!("The {spool} spool's {what} (it is {v}).")
        }
        SizingError::Chokes { m_abs } => format!(
            "The {spool} spool's front row would meet its air at the speed of sound (absolute Mach {m_abs:.3}): \
             the blade passage chokes at design. Lower the design flow coefficient or the airflow level, or \
             lower the rounding knob (blades at the wall push the air faster)."),
    }
}

/// Build the machine: the precheck, then rung 85's `build` on this view's engine and shape.
pub fn machine(s: &BladeSettings) -> Result<Machine, DoesNotRun> {
    blade_precheck(s)?;
    let (ml, mh) = shape_maps(SHAPES[s.shape]).expect("SHAPES lists only buildable shapes");
    let design = s.design();
    build(&design, s.flight(), ml, mh, s.lp, s.hp).map_err(|e| {
        // `build` sizes the LP spool first, so the refusal is the LP's iff the LP's knobs alone
        // (on both spools) refuse the same way — a second sizing, microseconds.
        let lp_alone = build(&design, s.flight(), ml, mh, s.lp, s.lp).err();
        DoesNotRun(sizing_words(&e, if lp_alone == Some(e) { "low-pressure" } else { "high-pressure" }))
    })
}

fn binding_key(b: Binding) -> &'static str {
    match b { Binding::Airflow => "airflow", Binding::Strength => "strength" }
}

/// One spool's sizing, every number.
pub fn sizing_json(z: &Sizing) -> Json {
    let kt = 2.0 / (1.0 + z.knobs.h);
    jobj! {
        "binding" => binding_key(z.binding),
        "Tt_face" => z.duty.tt, "work" => z.duty.dh, "slope_l" => z.duty.l,
        "gamma" => z.duty.gamma, "cp" => z.duty.cp,
        // Both walls as design MEAN blade speeds, whichever binds.
        "u_airflow" => airflow_wall(&z.duty, &z.knobs),
        "u_strength" => z.u_cap / z.knobs.overspeed / kt,
        "u_wall" => z.u_wall, "k_star" => z.k_star, "k" => Json::Int(z.k as i64),
        "u0" => z.u0, "u_mean" => z.u_m, "u_tip" => z.u_tip, "r" => z.r,
        "preswirl" => z.v_d, "preswirl_deg" => z.alpha_d_deg,
        "m_abs" => z.m_abs, "capacity" => z.capacity, "m_rel_tip" => z.m_rel_d,
        "u_cap_tip" => z.u_cap, "u_redline_tip" => z.u_cap / z.knobs.overspeed, "redline" => z.redline,
    }
}

/// The airflow levels the staircase chart sweeps.
pub fn staircase_levels() -> Vec<f64> {
    (0..=70).map(|i| 1.0 + 0.01 * i as f64).collect()
}

/// `K` and `R` against the airflow level, one spool, at the rounding knob `lambda` (everything
/// else as set). A level whose front row chokes is `null` — the chart leaves a gap.
fn stair(z: &Sizing, lambda: f64) -> Json {
    let (mut ks, mut rs) = (Vec::new(), Vec::new());
    for m in staircase_levels() {
        match size(z.duty, BladeKnobs { m_rel_lim: m, lambda, ..z.knobs }) {
            Ok(s) => { ks.push(Json::Int(s.k as i64)); rs.push(Json::Float(s.redline)); }
            Err(_) => { ks.push(Json::Null); rs.push(Json::Null); }
        }
    }
    jobj! { "lambda" => lambda, "k" => Json::List(ks), "redline" => Json::List(rs) }
}

/// `op: blade_size` — both spools sized, the design run's faces, the staircase sweeps.
pub fn size_json(s: &BladeSettings) -> Result<Json, DoesNotRun> {
    let z = machine(s)?;
    let st = |sz: &Sizing| Json::List(vec![stair(sz, 0.0), stair(sz, 1.0), stair(sz, sz.knobs.lambda)]);
    Ok(jobj! {
        "ok" => Json::Int(1), "blades" => s.to_json(),
        "lp" => sizing_json(&z.lp), "hp" => sizing_json(&z.hp),
        "levels" => Json::List(staircase_levels().into_iter().map(Json::Float).collect()),
        "stair_lp" => st(&z.lp), "stair_hp" => st(&z.hp),
        "flight" => jobj! { "T0" => s.flight().t0, "p0" => s.flight().p0, "M0" => s.flight().m0 },
    })
}

/// The throttle grid a lever is read on: the design `Tt4` down to 60 % of it.
pub fn lever_grid(s: &BladeSettings) -> Vec<f64> {
    (0..=LEVER_POINTS).map(|i| s.tt4 * (1.0 - 0.4 * i as f64 / LEVER_POINTS as f64)).collect()
}

/// Intervals on the lever grid (so `LEVER_POINTS + 1` throttles).
pub const LEVER_POINTS: usize = 8;

fn verdict_key(v: Verdict) -> &'static str {
    match v { Verdict::Crosses => "crosses", Verdict::Under => "under", Verdict::Void => "void" }
}

/// `op: blade_lever` — ONE throttle of the chosen lever's constant-incidence schedule, read against
/// both redlines (`Machine::schedule`, bit for bit). Where the lever-at-design point itself does not
/// match, the reply is a refusal (`"ok":0`) with plain words and the model's `"message"`, made
/// BEFORE the schedule runs — the common case is the unchoked nozzle, 5–6 % of a random sweep.
pub fn lever_json(s: &BladeSettings, tt4: f64) -> Result<Json, DoesNotRun> {
    if !(tt4.is_finite() && tt4 > 0.0) {
        return Err(DoesNotRun("The throttle (turbine-inlet temperature) must be a number above zero.".into()));
    }
    let z = machine(s)?;
    let fl = s.flight();
    let plant = z.plant(s.lever, s.spool);
    if let Err(e) = plant.core.core.try_match_point(&fl, tt4) {
        // A refusal carrying plain words AND the model's own message, as the trap path gives them.
        return Ok(jobj! { "ok" => Json::Int(0), "reason" => lever_words(&e.0), "message" => e.0 });
    }
    let r = z.schedule(s.lever, s.spool, &[tt4])[0];
    let moved = match s.spool { Spool::Lp => &z.lp, Spool::Hp => &z.hp };
    let deg = |v: f64| ((moved.v_d + v) / moved.knobs.phi_d).atan().to_degrees();
    Ok(jobj! {
        "ok" => Json::Int(1), "Tt4" => r.tt4, "reached" => Json::Int(r.reached as i64),
        "travel" => r.vsv_star, "vane_deg" => deg(r.vsv_star), "vane_deg_design" => deg(0.0),
        "n_lp" => r.n_lp, "n_hp" => r.n_hp, "n_lp_bare" => r.n_lp_bare, "n_hp_bare" => r.n_hp_bare,
        "tip_mach_lp" => r.tip_mach_lp, "tip_mach_hp" => r.tip_mach_hp,
        "tip_mach_lp_bare" => r.tip_mach_lp_bare, "tip_mach_hp_bare" => r.tip_mach_hp_bare,
        "redline_lp" => z.lp.redline, "redline_hp" => z.hp.redline,
        "verdict_lp" => verdict_key(r.verdict(&z, Spool::Lp)), "verdict_hp" => verdict_key(r.verdict(&z, Spool::Hp)),
    })
}

/// Plain words for the lever-at-design match's own refusals — at a THROTTLE, so each says what the
/// throttle does. The three a 3 300-design sweep through these requests produced (2026-10-07, both
/// gases, plan § 11.5), each driven by a test:
/// - the unchoked nozzle (5 % of points, both gases);
/// - the nozzle's gas below the outside pressure (1 %, perfect gas, high pressure ratios at a low
///   throttle) — the design itself ran;
/// - `inverse: root not bracketed` (3 %, thermally perfect only). Inside the two-spool match the
///   only fallible gas inverse is the turbines' exit temperature from their pressure ratio
///   (`TwoSpoolMatcher::try_tau_of`, the one chain `gas::try_solve` documents): a turbine expanding
///   past the bottom of the gas tables.
pub fn lever_words(message: &str) -> String {
    if message.contains("nozzle back-pressure") {
        return "At this throttle the gas reaching the nozzle has less pressure than the air outside: the two \
                turbines use it all up driving the compressors. This happens at a low throttle with high \
                pressure ratios. Raise the throttle.".to_string();
    }
    if message.contains("root not bracketed") {
        return "At this throttle a turbine would have to expand the gas colder than the gas tables reach: it \
                is asked for more work than the cooled gas can give. This happens at a low throttle with high \
                pressure ratios. Raise the throttle.".to_string();
    }
    explain_blades(message)
}

/// Plain words for a message the model raised while SIZING (`blade_size` — a trap in a lever
/// request gets [`lever_words`], through the `explain` op's `"blades_lever"` view). The design run
/// itself fails two ways over the page's slider box (sweep 2026-10-07, 45 000 designs, plan § 11.8),
/// both only at an overall pressure ratio ≳ 49 with a low design `Tt4`: the jet below outside
/// pressure (thermally perfect, 1.7 %) and slice 1's efficiency-bookkeeping check ([`explain`]'s
/// words). The two-spool matchers' own failure, the UNCHOKED nozzle (rung 38's scope, an open seam),
/// is kept for completeness. Each entry is driven by a test (`tests/sandbox_blades.rs`).
pub fn explain_blades(message: &str) -> String {
    if message.contains("nozzle UNCHOKED") {
        return "At this throttle the nozzle unchokes: the jet no longer reaches the speed of sound. The \
                two-spool model only handles a choked nozzle (the unchoked case is an open gap in the \
                project), so this throttle is not modelled. Raise the throttle.".to_string();
    }
    if message.contains("nozzle back-pressure") {
        return "The design engine does not run: the gas reaching the nozzle has less pressure than the air \
                outside, because the two turbines use it all up driving the compressors. This happens with \
                high pressure ratios and a low design turbine-inlet temperature. Raise Tt4 or lower the \
                pressure ratios.".to_string();
    }
    explain(message)
}

/// The blade requests the browser check runs natively AND in the browser build (appended to
/// [`crate::sandbox::check_requests`]): the page's opening sizing, every shape, both gases, the
/// rounding knob at both ends and both droop switches, split spools, a material weak enough that
/// strength binds; then rung 85's panel lever rows at `Tt4` 1000 (perfect gas — milliseconds), one
/// row-by-row lever at a throttle where its target is never reached, a refused throttle, and ONE
/// thermally-perfect point (one-block lever, ~0.3 s) — the gate's time is shared. Every one runs.
pub fn check_requests() -> Vec<String> {
    let size = |b: BladeSettings| jobj! { "op" => "blade_size", "blades" => b.to_json() }.dump_compact();
    let lever = |b: BladeSettings, tt4: f64| jobj! { "op" => "blade_lever", "blades" => b.to_json(), "Tt4" => tt4 }.dump_compact();
    let d = BladeSettings::defaults();
    let k = |h: f64, m: f64, lambda: f64, droop: Droop| BladeKnobs { h, m_rel_lim: m, lambda, droop, ..BladeKnobs::default() };
    let both = |kn: BladeKnobs| BladeSettings { lp: kn, hp: kn, ..d };
    let mut out = vec![r#"{"op":"blade_defaults"}"#.to_string(), r#"{"op":"blade_size","blades":{}}"#.to_string(),
                       r#"{"op":"blade_grid","blades":{}}"#.to_string()];
    for shape in 0..SHAPES.len() {
        out.push(size(BladeSettings { shape, ..d }));
    }
    out.push(size(BladeSettings { gas: BladeGas::ThermallyPerfect, ..d }));
    out.push(size(both(k(0.7, 1.5, 1.0, Droop::WithBladeSpeed))));
    out.push(size(both(k(0.5, 1.3, 0.5, Droop::WithRowWork))));
    out.push(size(BladeSettings { pi_lpc: 2.0, pi_hpc: 12.0, tt4: 1700.0, hp: k(0.7, 1.6, 1.0, Droop::WithBladeSpeed), ..d }));
    out.push(size(both(BladeKnobs { sigma_over_rho: 3.0e4, ..BladeKnobs::default() })));
    for (lev, spool, kn) in [(Lever::Lumped, Spool::Lp, k(0.5, 1.4, 0.0, Droop::WithBladeSpeed)),
                             (Lever::Lumped, Spool::Hp, k(0.5, 1.4, 0.0, Droop::WithBladeSpeed)),
                             (Lever::AllRows, Spool::Lp, k(0.5, 1.4, 0.0, Droop::WithBladeSpeed)),
                             (Lever::FrontRow, Spool::Lp, k(0.5, 1.4, 0.0, Droop::WithBladeSpeed)),
                             (Lever::AllRows, Spool::Hp, k(0.5, 1.5, 0.0, Droop::WithBladeSpeed)),
                             (Lever::FrontRow, Spool::Hp, k(0.5, 1.5, 0.0, Droop::WithBladeSpeed)),
                             (Lever::Lumped, Spool::Lp, k(0.5, 1.5, 1.0, Droop::WithBladeSpeed)),
                             (Lever::Lumped, Spool::Lp, k(0.7, 1.5, 1.0, Droop::WithRowWork))] {
        out.push(lever(BladeSettings { lever: lev, spool, ..both(kn) }, 1000.0));
    }
    out.push(lever(d, 1500.0));                                                  // the design point: travel 0
    out.push(lever(d, 600.0));                                                   // refused: the nozzle unchokes
    out.push(lever(BladeSettings { gas: BladeGas::ThermallyPerfect, ..d }, 1000.0));
    out
}

/// The blade view's ops, for [`crate::sandbox::call`]. `None` = not a blade op.
pub fn call_op(op: &str, req: &Json) -> Option<Json> {
    let refusal = |m: &str| jobj! { "ok" => Json::Int(0), "reason" => m };
    let settings = || {
        let empty = Json::Obj(Vec::new());
        BladeSettings::from_json(req.get("blades").unwrap_or(&empty))
    };
    Some(match op {
        "blade_defaults" => jobj! { "blades" => BladeSettings::defaults().to_json(),
                                    "ti64" => TI64_SPECIFIC, "shapes" => Json::List(SHAPES.iter().map(|&s| Json::from(s)).collect()) },
        "blade_size" => match settings() {
            Err(e) => refusal(&e),
            Ok(s) => size_json(&s).unwrap_or_else(|DoesNotRun(m)| refusal(&m)),
        },
        "blade_grid" => match settings() {
            Err(e) => refusal(&e),
            Ok(s) => Json::List(lever_grid(&s).into_iter().map(Json::Float).collect()),
        },
        "blade_lever" => match settings() {
            Err(e) => refusal(&e),
            Ok(s) => {
                let tt4 = match req.get("Tt4") { Some(Json::Float(x)) => *x, Some(Json::Int(n)) => *n as f64, _ => s.tt4 };
                lever_json(&s, tt4).unwrap_or_else(|DoesNotRun(m)| refusal(&m))
            }
        },
        _ => return None,
    })
}
