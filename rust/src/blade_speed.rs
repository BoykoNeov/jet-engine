//! RUNG 85 — **THE BLADE-SPEED WALLS.** The stack (rungs 55/56) gets a physical blade speed.
//!
//! `docs/rung85-spec.md`; the anchor, with every derivation, source and pre-registered bar, is
//! `docs/plans/rung85-anchor-blade-speed.md` (§ refs below are to it).
//!
//! # What it adds — KNOBS, every result shown (the user's sandbox decision, § 4.4 A1)
//!
//! Rungs 55/56 never named a blade speed: it was IMPLIED twice, by the stage count `K` and by the
//! capacity level `C`, and the two disagreed by ~30 % on the LP spool (§ 0 P-A). Here the blade
//! speed is SIZED from two walls, per spool, and `K` and `C` become OUTPUTS:
//!
//! * **the AIRFLOW level** `M_rel,lim` — the front-row relative tip Mach a designer targets for
//!   efficiency (Biollo & Benini 2011: ≈ 1.3 civil). It SIZES the design; off design the tip Mach
//!   is a READING, never a wall (§ 3, the user's roles decision);
//! * **the STRENGTH wall** — the untapered blade-root pull `σ = ρ·U_tip²(1−h²)/2` at the material's
//!   yield `σ_y/ρ` (Ti-6Al-4V, AMS 4911 minimum), with 14 CFR 33.27's 120 % overspeed between the
//!   capability and the REDLINE `R = N_red/N_d`. The only OFF-design wall: a lever whose physical
//!   shaft speed passes `R` CROSSES it.
//!
//! The design flow coefficient `Φ_d` (Rotor 37's mean line, 0.537) is a design input (§ 6); it
//! enters ONLY this module's sizing and readouts — never a shipped schedule residual (§ 4.4 A2).
//!
//! # The rounding knob `λ` (the user's, § 4.5)
//!
//! A real machine has whole rows, so `K = ceil(K*)` and the design sits below the wall by a
//! rounding gap. `λ` chooses who absorbs it: `0` slows the blades (rungs 55/56's machine), `1`
//! keeps them AT the wall and loads every row lighter through a design pre-swirl
//! `v_d = (1 − r)/(1 + l)`. That pre-swirl is EXACTLY a reshaped map of the shipped family
//! ([`reshape`]: `σ' = σ/r`, `l' = (1+l)/r − 1`, § 4.5 A9), so no solver changes — the lever's
//! `vsv` becomes TRAVEL from design, and this module adds `v_d` back wherever a physical angle is
//! read. Rung 54's throat law is never called (it reads `vsv` as a physical tangent at `Φ_d = 1`;
//! § 4.5 A9 (ii)).
//!
//! # Reduce
//!
//! The walls do not touch a shipped plant: at `r == 1.0` (`λ = 0`, or an exactly-integer `K*`)
//! [`reshape`] returns the SHIPPED map object, so every plant run here is rung 53/55's own,
//! bit for bit (`tests/rung85.rs`).
//!
//! # Born in Rust
//!
//! The first rung with no Python ancestor, so the porting rules that keep a port bit-equal to
//! PyPy (`powp` for `** 0.5`, Python's operation order) bind nothing here: a square root is
//! `sqrt`. Its numbers are held to the anchor's independently-computed § 6.2 table, never to a
//! capture of themselves.

use crate::engine::FlightCondition;
use crate::map::{mfp_frac, ComponentMap};
use crate::stage::{StageStackCore, StageStackCoreSpec};
use crate::stator::VariableStatorCore;
use crate::two_spool::{Spool, TwoSpoolEngine, TwoSpoolMapResult};

/// Ti-6Al-4V annealed, AMS 4911 plate > 0.1874 in: specified MINIMUM 0.2 % yield, Pa (Rolled
/// Alloys bulletin 1052USe 09/16, 120 ksi). Plate stands in for the forging (§ 1, disclosed).
pub const TI64_YIELD: f64 = 827.4e6;
/// Ti-6Al-4V density, kg/m³ (same sheet, 0.160 lb/in³).
pub const TI64_DENSITY: f64 = 4429.0;
/// 14 CFR 33.27: every rotor survives 120 % of its maximum permissible speed.
pub const OVERSPEED_33_27: f64 = 1.2;
/// NASA TP 1659 Table II(a): Rotor 37's mean-line design flow coefficient, 210.2/391.7.
pub const PHI_D_ROTOR37: f64 = 0.537;

/// One spool's blade knobs. Every field is a disclosed design choice; the defaults are the
/// sourced values and the middle of § 6.2's grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BladeKnobs {
    /// Front-row hub-to-tip ratio `h` — shared by both walls.
    pub h: f64,
    /// The airflow design level: front-row relative tip Mach at design.
    pub m_rel_lim: f64,
    /// Material specific strength `σ_y/ρ`, m²/s².
    pub sigma_over_rho: f64,
    /// Capability-to-redline factor (1.2 = 14 CFR 33.27).
    pub overspeed: f64,
    /// Design flow coefficient `Φ_d = Vx/U_m` (1 = rungs 55/56's sizing).
    pub phi_d: f64,
    /// The rounding knob, `0` = blades slow down, `1` = blades at the wall, rows load lighter.
    pub lambda: f64,
}

impl Default for BladeKnobs {
    fn default() -> Self {
        BladeKnobs { h: 0.5, m_rel_lim: 1.4, sigma_over_rho: TI64_YIELD / TI64_DENSITY,
                     overspeed: OVERSPEED_33_27, phi_d: PHI_D_ROTOR37, lambda: 0.0 }
    }
}

/// What the cycle hands one spool's sizing: its face, its design work and its map slope.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpoolDuty {
    /// Design face total temperature, K (station 2 LP / 25 HP).
    pub tt: f64,
    /// Design spool work `Δh`, J/kg.
    pub dh: f64,
    /// The map's design loading slope `l` (per-row work coefficient `1/(1+l)`, § 6).
    pub l: f64,
    pub gamma: f64,
    pub cp: f64,
}

/// Which wall set the design blade speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    /// The airflow level — every cell at `M0` = 0.85 (§ 0 P-B).
    Airflow,
    /// The strength wall at design: D3's vacuity condition. The design then sits AT the redline
    /// (`R` = 1 at `K*`), so every overspeed crosses by construction.
    Strength,
}

/// A spool's sized machine — every number the sandbox shows for it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizing {
    pub duty: SpoolDuty,
    pub knobs: BladeKnobs,
    /// The binding wall's design mean blade speed (zero swirl), m/s.
    pub u_wall: f64,
    pub binding: Binding,
    /// Continuous stage count at the wall, and the machine's integer one.
    pub k_star: f64,
    pub k: usize,
    /// Mean blade speed if the blades absorb the rounding (`λ = 0`), m/s.
    pub u0: f64,
    /// The design mean blade speed at this `λ`, and its tip speed, m/s.
    pub u_m: f64,
    pub u_tip: f64,
    /// Design row work relative to the unswirled row, `(u0/u_m)²`.
    pub r: f64,
    /// Design pre-swirl in map units (`v = Φ_d·tan α₁`) and as a physical angle, degrees.
    pub v_d: f64,
    pub alpha_d_deg: f64,
    /// The front row's design absolute Mach and the capacity level it implies,
    /// `C = MFP(M_abs)/MFP(1)` (§ 4.5 A9 (ii)).
    pub m_abs: f64,
    pub capacity: f64,
    /// The front row's design relative tip Mach (below `m_rel_lim` when `λ > 0`, § 4.5 A11).
    pub m_rel_d: f64,
    /// Strength capability tip speed, m/s, and the redline `R = N_red/N_d`.
    pub u_cap: f64,
    pub redline: f64,
}

/// Why a knob setting builds no machine. Reported, never a panic: these sit behind sandbox knobs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SizingError {
    /// A knob outside its physical range (named, with its value).
    Knob(&'static str, f64),
    /// The front row's design absolute Mach reaches 1: the row passage chokes at design.
    Chokes { m_abs: f64 },
}

impl std::fmt::Display for SizingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SizingError::Knob(name, v) => write!(f, "knob {name} = {v} is out of range"),
            SizingError::Chokes { m_abs } =>
                write!(f, "the front row chokes at design (absolute Mach {m_abs:.3} >= 1)"),
        }
    }
}

/// Mean blade speed at which the front row's relative tip Mach equals `m` with NO inlet swirl —
/// § 6.2's airflow wall, in closed form. With `Vx = Φ_d·U`, `U_tip = k·U`, `k = 2/(1+h)`,
/// `T = Tt − Vx²/2cp`: `(Φ_d² + k²)·U² = m²·γR·(Tt − Φ_d²U²/2cp)`.
fn airflow_wall(duty: &SpoolDuty, kn: &BladeKnobs) -> f64 {
    let rg = duty.cp * (duty.gamma - 1.0) / duty.gamma;
    let kt = 2.0 / (1.0 + kn.h);
    let a2 = kn.m_rel_lim * kn.m_rel_lim * duty.gamma * rg;
    let p2 = kn.phi_d * kn.phi_d;
    (a2 * duty.tt / (p2 + kt * kt + a2 * p2 / (2.0 * duty.cp))).sqrt()
}

/// The front row's (absolute, relative-tip) Mach at mean blade speed `u`, normalised flow
/// coefficient `phi` and total swirl `v` in map units. Uniform absolute inlet angle across the
/// span (`Vθ = Vx·tan α₁ = u·phi·v`, disclosed § 4.5 A11); static `T = Tt − (Vx² + Vθ²)/2cp`.
/// Corrected quantities: `u` is the CORRECTED mean speed referred to the design face `Tt`.
fn front_row_mach(duty: &SpoolDuty, kn: &BladeKnobs, u: f64, phi: f64, v: f64) -> (f64, f64) {
    let rg = duty.cp * (duty.gamma - 1.0) / duty.gamma;
    let vx = kn.phi_d * phi * u;
    let vt = u * phi * v;
    let t = duty.tt - (vx * vx + vt * vt) / (2.0 * duty.cp);
    let a = (duty.gamma * rg * t).sqrt();
    let wt = 2.0 / (1.0 + kn.h) * u - vt;
    ((vx * vx + vt * vt).sqrt() / a, (vx * vx + wt * wt).sqrt() / a)
}

/// Size one spool: both walls, the integer stage count, the rounding knob, the redline.
pub fn size(duty: SpoolDuty, kn: BladeKnobs) -> Result<Sizing, SizingError> {
    let in_range = |name, v: f64, lo: f64, hi: f64| {
        if v.is_finite() && v > lo && v < hi { Ok(()) } else { Err(SizingError::Knob(name, v)) }
    };
    in_range("h", kn.h, 0.0, 1.0)?;
    in_range("m_rel_lim", kn.m_rel_lim, 0.0, 3.0)?;
    in_range("sigma_over_rho", kn.sigma_over_rho, 0.0, f64::INFINITY)?;
    in_range("overspeed", kn.overspeed, 1.0 - 1e-12, f64::INFINITY)?;
    in_range("phi_d", kn.phi_d, 0.0, 2.0)?;
    if !(0.0..=1.0).contains(&kn.lambda) {
        return Err(SizingError::Knob("lambda", kn.lambda));
    }
    let kt = 2.0 / (1.0 + kn.h);
    let u_cap = (2.0 * kn.sigma_over_rho / (1.0 - kn.h * kn.h)).sqrt();
    // The strength wall at design is the capability brought down by the overspeed factor: a
    // design faster than that would start life above its own redline (D3).
    let u_air = airflow_wall(&duty, &kn);
    let u_str = u_cap / kn.overspeed / kt;
    let (u_wall, binding) = if u_air <= u_str { (u_air, Binding::Airflow) } else { (u_str, Binding::Strength) };
    let k_star = duty.dh * (1.0 + duty.l) / (u_wall * u_wall);
    let k = (k_star.ceil() as usize).max(1);
    let u0 = (duty.dh * (1.0 + duty.l) / k as f64).sqrt();
    let u_m = if kn.lambda == 0.0 { u0 } else { u0 + kn.lambda * (u_wall - u0) };
    let r = if u_m == u0 { 1.0 } else { (u0 / u_m) * (u0 / u_m) };
    let v_d = (1.0 - r) / (1.0 + duty.l);
    let (m_abs, m_rel_d) = front_row_mach(&duty, &kn, u_m, 1.0, v_d);
    if !(m_abs < 1.0) {
        return Err(SizingError::Chokes { m_abs });
    }
    let u_tip = kt * u_m;
    Ok(Sizing {
        duty, knobs: kn, u_wall, binding, k_star, k, u0, u_m, u_tip, r, v_d,
        alpha_d_deg: (v_d / kn.phi_d).atan().to_degrees(),
        m_abs, capacity: mfp_frac(m_abs, duty.gamma), m_rel_d,
        u_cap, redline: u_cap / (kn.overspeed * u_tip),
    })
}

impl Sizing {
    /// The front row's relative TIP Mach off design — a READING (§ 3). `n` the corrected speed
    /// (design 1), `phi` the face flow coefficient (design 1), `v` the lever's TRAVEL from design;
    /// the design pre-swirl is added here, so a physical angle never reads travel (§ 4.5 A9 (iii)).
    pub fn tip_rel_mach(&self, n: f64, phi: f64, v: f64) -> f64 {
        front_row_mach(&self.duty, &self.knobs, n * self.u_m, phi, self.v_d + v).1
    }
}

/// The shipped map family with a design pre-swirl folded in (§ 4.5 A9): `σ' = σ/r`,
/// `l' = (1+l)/r − 1`. At `r == 1.0` it returns the map UNTOUCHED — the bit-for-bit reduce
/// (A10); `(1+l)/1 − 1` is not `l` in the last bit for every `l`.
pub fn reshape(map: ComponentMap, r: f64) -> ComponentMap {
    if r == 1.0 {
        return map;
    }
    ComponentMap { sigma: map.sigma / r, l: (1.0 + map.l) / r - 1.0, ..map }
}

/// Both spools sized on one design engine and map pair.
#[derive(Clone)]
pub struct Machine {
    pub design: TwoSpoolEngine,
    pub flight: FlightCondition,
    /// The maps the plants run on — [`reshape`]d by each spool's own `r`.
    pub map_lp: ComponentMap,
    pub map_hp: ComponentMap,
    pub lp: Sizing,
    pub hp: Sizing,
}

/// Size both spools on `design` (its design run supplies each face's `Tt` and work), with each
/// spool's own knobs (§ 4.5 A12), and reshape the two maps.
pub fn build(
    design: &TwoSpoolEngine, flight: FlightCondition, map_lp: ComponentMap, map_hp: ComponentMap,
    knobs_lp: BladeKnobs, knobs_hp: BladeKnobs,
) -> Result<Machine, SizingError> {
    let core = VariableStatorCore::new(design.clone(), flight, 1.0, map_lp, map_hp, 0.0, 0.0);
    let c = &core.core;
    let gas = c.gas();
    let gamma = gas.gamma_c();
    let cp = gas.r_c() * gamma / (gamma - 1.0);
    let tt3 = c.tt25_d * c.tau_hpc_d;
    let duty_lp = SpoolDuty { tt: c.tt2_d, dh: gas.h_c(c.tt25_d) - gas.h_c(c.tt2_d), l: map_lp.l, gamma, cp };
    let duty_hp = SpoolDuty { tt: c.tt25_d, dh: gas.h_c(tt3) - gas.h_c(c.tt25_d), l: map_hp.l, gamma, cp };
    let lp = size(duty_lp, knobs_lp)?;
    let hp = size(duty_hp, knobs_hp)?;
    Ok(Machine { design: design.clone(), flight, map_lp: reshape(map_lp, lp.r),
                 map_hp: reshape(map_hp, hp.r), lp, hp })
}

/// Which stator lever moves, and over which rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lever {
    /// Rung 53's lumped machine: ONE block per spool (rung 55's `K` = 1, its proven identity).
    Lumped,
    /// Rung 55's stack at the walls' `K`, the lever on EVERY row.
    AllRows,
    /// Rung 55's stack at the walls' `K`, the lever on the FRONT row only.
    FrontRow,
}

/// One schedule point, read against both spools' redlines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeverRead {
    pub tt4: f64,
    pub spool: Spool,
    pub lever: Lever,
    /// Did the schedule reach its target incidence? If not, everything below is read at the LAST
    /// valid scan point (§ 4.2 V1).
    pub reached: bool,
    /// The lever's travel from design, map units.
    pub vsv_star: f64,
    /// Physical shaft speed against design, both spools — what the strength wall reads (V2).
    pub n_lp: f64,
    pub n_hp: f64,
    /// The same throttle with the lever at design.
    pub n_lp_bare: f64,
    pub n_hp_bare: f64,
    /// The front rows' relative tip Mach — readings — at the lever's setting and bare.
    pub tip_mach_lp: f64,
    pub tip_mach_hp: f64,
    pub tip_mach_lp_bare: f64,
    pub tip_mach_hp_bare: f64,
}

/// What a read says about one spool's redline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Physical speed above the redline, at the target (or before it, under V1).
    Crosses,
    /// Under the redline at the reached target — one-sided: the strength wall is optimistic (§ 5).
    Under,
    /// The MOVED spool's schedule never reached its target and never passed the redline along
    /// its scan: no verdict (V1). The unmoved spool is never `Void` — its speed is a valid
    /// reading at whatever setting the scan stopped on.
    Void,
}

impl LeverRead {
    pub fn verdict(&self, m: &Machine, spool: Spool) -> Verdict {
        let (n, r) = match spool {
            Spool::Lp => (self.n_lp, m.lp.redline),
            Spool::Hp => (self.n_hp, m.hp.redline),
        };
        if n > r {
            Verdict::Crosses
        } else if self.reached || spool != self.spool {
            Verdict::Under
        } else {
            Verdict::Void
        }
    }
}

impl Machine {
    /// The rung-55 plant for `lever` with `spool`'s lever moving.
    pub fn plant(&self, lever: Lever, spool: Spool) -> StageStackCore {
        let base = StageStackCoreSpec::new(self.design.clone(), self.flight, 1.0, self.map_lp, self.map_hp);
        let front = |s: Spool| if lever == Lever::FrontRow && s == spool { Some(1) } else { None };
        let (k_lp, k_hp) = match lever {
            Lever::Lumped => (1, 1),
            Lever::AllRows | Lever::FrontRow => (self.lp.k, self.hp.k),
        };
        StageStackCore::new(StageStackCoreSpec {
            k_lp, k_hp, vsv_stages_lp: front(Spool::Lp), vsv_stages_hp: front(Spool::Hp), ..base
        })
    }

    fn read_at(&self, plant: &StageStackCore, spool: Spool, v: f64, tt4: f64) -> TwoSpoolMapResult {
        match spool {
            Spool::Lp => plant.at_setting(v, 0.0),
            Spool::Hp => plant.at_setting(0.0, v),
        }.match_point(&self.flight, tt4)
    }

    /// The constant-incidence schedule (rung 53's target: the front stage's design incidence,
    /// read off the matcher) on `spool`, at each `Tt4`, read against both redlines. Rung 55's
    /// scan-and-bisect, which reports an unreached target instead of panicking (V1).
    pub fn schedule(&self, lever: Lever, spool: Spool, tt4_grid: &[f64]) -> Vec<LeverRead> {
        let plant = self.plant(lever, spool);
        plant.stage_incidence_schedule(&self.flight, tt4_grid, spool, 0, 4.0).iter().map(|row| {
            let at = self.read_at(&plant, spool, row.vsv_star, row.tt4);
            let bare = self.read_at(&plant, spool, 0.0, row.tt4);
            let (vl, vh) = match spool { Spool::Lp => (row.vsv_star, 0.0), Spool::Hp => (0.0, row.vsv_star) };
            LeverRead {
                tt4: row.tt4, spool, lever, reached: row.reached, vsv_star: row.vsv_star,
                n_lp: at.n_lp_ratio, n_hp: at.n_hp_ratio,
                n_lp_bare: bare.n_lp_ratio, n_hp_bare: bare.n_hp_ratio,
                tip_mach_lp: self.lp.tip_rel_mach(at.n_lp, at.phi_lp, vl),
                tip_mach_hp: self.hp.tip_rel_mach(at.n_hp, at.phi_hp, vh),
                tip_mach_lp_bare: self.lp.tip_rel_mach(bare.n_lp, bare.phi_lp, 0.0),
                tip_mach_hp_bare: self.hp.tip_rel_mach(bare.n_hp, bare.phi_hp, 0.0),
            }
        }).collect()
    }
}
