//! The web sandbox, slice 5 — **THE BURNER** (`docs/plans/sandbox-plan.md` § 13). NOT a rung.
//!
//! Slices 1–4 treat the burner as a box that turns `Tt3` into `Tt4`. This view opens it: the NOx
//! diagnostics of rungs 7–24 as knobs — how rich the front (primary) zone burns and how long the gas
//! stays there, the fast-O-atom and prompt-NO switches (rung 19), and how the dilution air mixes in:
//! instantly (rungs 8/9), over a set time (10), or through dilution jets (11) with, optionally, ONE of
//! the eight mixing models the model allows (12, 13, 15, 16, 18, 22, 23, 24).
//!
//! **Which burner** (plan § 13.1, the user's answer § 13.8): the *Design* view's design, always run
//! on the EQUILIBRIUM gas — rungs 7–24's own gas, and the only one whose burner inlet lands the
//! diagnostics' mix-out check (the perfect gas's constant `cp` needs too little fuel). The
//! diagnostics read four numbers off that run (`Tt3`, `Tt4`, the fuel-air ratio, `pt4`), exactly as
//! the CLI's panels do (`panels::nox::design_quad`).
//!
//! **Every request is self-contained**: it re-runs the design (~2 ms) and calls the model once. No
//! cache, no state between requests, and no model code edited (§ 13.8 Q4) — `zoned_nox` is called as
//! it stands. Every rung from 7 up is a diagnostic BESIDE the cycle, so nothing here moves a cycle
//! number.
//!
//! **The grids are fixed** (plan § 13.2's level L1): within ~0.6 % of the model's own defaults at
//! every jet strength with every minimum in place, where the next coarser level moved a minimum and
//! crashed the cross-plane models on their own quadrature check. They are constants, never knobs.

use crate::engine::{build_turbojet, Engine, EngineResult};
use crate::gas::{equilibrium_composition, f_stoich};
use crate::jobj;
use crate::nox::{
    quench_trajectory, JetMixing, MixingPdf, PocketQuenchPdf, PromptNo, QuenchPdf, SpatialDwellPdf,
    SpatialLocalPdf, SpatialPdf, ThermalNoxOpts, TransportedPdf, Unmixedness, ZonedNoxOpts,
    ZonedNoxState,
};
use crate::sandbox::{self, DoesNotRun, GasModel, Settings};
use crate::visuals::Json;

// ---- the fixed grids (plan § 13.2, level L1) ------------------------------------------------------

/// RK4 steps of the front zone's Zeldovich integrator — the model's own default.
pub const ZELDOVICH_STEPS: usize = 4000;
/// Points on a finite quench's mixing path. Its RK4 steps stay at the model's own default (2000):
/// the charts page's 400 went UNSTABLE on a long core dwell at high pressure (rung 12's core EI came
/// back NaN where 2000, 8000 and 32 000 steps agree to 1e-9 — the crash map, plan § 13.9), and the
/// steps cost almost nothing beside the path's equilibrium solves.
pub const QUENCH_NGRID: usize = 60;
/// Points on the "ideal bell" (rungs 13, 15, 18, 22) — and on the per-pocket models' pocket grid
/// (16, 23, 24), where each point is a whole quench.
pub const N_BELL: usize = 80;
pub const N_BELL_POCKET: usize = 40;
// The β-PDF quadrature stays at each model's own default (200 on rungs 13/15/18/22, 160 on 16/23/24):
// the nodes cost little beside the curve, and the plan's 160 on the first four failed the model's own
// mean check on 6 cases the default passes (crash map, plan § 13.9).
/// Cross-plane grid (rungs 22–24) and the plane-in-time slices (23).
pub const N_PLANE: usize = 32;
pub const N_TIME: usize = 24;

/// The richest front zone the model takes: its 5-species products stop at soot onset.
pub const PHI_MAX: f64 = 2.0;

/// Points in a richness sweep; points in a jet-strength sweep (log-spaced over [`J_SWEEP`]), before
/// the model's optimum is added.
pub const PHI_SWEEP_POINTS: usize = 21;
pub const J_SWEEP_POINTS: usize = 14;
pub const J_SWEEP: (f64, f64) = (1.0, 600.0);

// ---- the knobs ----------------------------------------------------------------------------------

/// How the dilution air meets the front zone's gas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quench {
    /// Rungs 8/9: all at once — the NO is frozen at the front zone's value.
    Instant,
    /// Rung 10: over a set time `τ_q`, along a straight mixing schedule.
    Time,
    /// Rung 11: through dilution jets — the jets' strength sets the time and the schedule.
    Jets,
}

impl Quench {
    pub const ALL: [Quench; 3] = [Quench::Instant, Quench::Time, Quench::Jets];
    pub fn key(self) -> &'static str {
        match self { Quench::Instant => "instant", Quench::Time => "time", Quench::Jets => "jets" }
    }
    fn from_key(k: &str) -> Result<Self, String> {
        Quench::ALL.into_iter().find(|q| q.key() == k).ok_or_else(|| format!("unknown quench {k:?}"))
    }
}

/// The mixing models that ride on the jets (`zoned_nox` takes at most one). `None` is rung 11's
/// mean field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Closure {
    None,
    TwoStream,
    Pdf,
    PdfQuench,
    Pocket,
    Transported,
    Spatial,
    SpatialDwell,
    SpatialLocal,
}

impl Closure {
    pub const ALL: [Closure; 9] = [Closure::None, Closure::TwoStream, Closure::Pdf, Closure::PdfQuench,
                                   Closure::Pocket, Closure::Transported, Closure::Spatial,
                                   Closure::SpatialDwell, Closure::SpatialLocal];

    pub fn key(self) -> &'static str {
        match self {
            Closure::None => "none",
            Closure::TwoStream => "two_stream",
            Closure::Pdf => "pdf",
            Closure::PdfQuench => "pdf_quench",
            Closure::Pocket => "pocket",
            Closure::Transported => "transported",
            Closure::Spatial => "spatial",
            Closure::SpatialDwell => "spatial_dwell",
            Closure::SpatialLocal => "spatial_local",
        }
    }

    fn from_key(k: &str) -> Result<Self, String> {
        Closure::ALL.into_iter().find(|c| c.key() == k).ok_or_else(|| format!("unknown mixing model {k:?}"))
    }

    pub fn rung(self) -> i64 {
        match self {
            Closure::None => 11,
            Closure::TwoStream => 12,
            Closure::Pdf => 13,
            Closure::PdfQuench => 15,
            Closure::Pocket => 16,
            Closure::Transported => 18,
            Closure::Spatial => 22,
            Closure::SpatialDwell => 23,
            Closure::SpatialLocal => 24,
        }
    }

    /// How long one point takes (plan § 13.2): `"fast"` (tenths of a second), `"curve"` (a
    /// precomputed curve each call, up to ~2 s) or `"pocket"` (each pocket its own quench: 4–10 s
    /// a point at the opening design in the browser, up to ~20 s at the box's edges, § 13.9).
    pub fn cost(self) -> &'static str {
        match self {
            Closure::None | Closure::TwoStream => "fast",
            Closure::Pdf | Closure::PdfQuench | Closure::Transported | Closure::Spatial => "curve",
            Closure::Pocket | Closure::SpatialDwell | Closure::SpatialLocal => "pocket",
        }
    }

    /// Whether the model is GIVEN the best mixing (`C_opt`, rungs 12–18) or WORKS IT OUT from the
    /// plume spread `k_p` (rungs 22–24: `C_opt = 1/(4k_p²)`).
    pub fn derives_optimum(self) -> bool {
        matches!(self, Closure::Spatial | Closure::SpatialDwell | Closure::SpatialLocal)
    }

    /// The model's own knobs for this mixing model, at the model's own defaults — read off each
    /// config's `Default`, never typed here. The grid sizes are not knobs (they are this module's
    /// fixed L1 constants).
    pub fn knob_defaults(self) -> Vec<(&'static str, f64)> {
        match self {
            Closure::None => vec![],
            Closure::TwoStream => {
                let d = Unmixedness::default();
                vec![("s", d.s), ("c_opt", d.c_opt), ("tau_res", d.tau_res), ("k_u", d.k_u),
                     ("b_u", d.b_u), ("w_max", d.w_max)]
            }
            Closure::Pdf => {
                let d = MixingPdf::default();
                vec![("s", d.s), ("c_opt", d.c_opt), ("k_g", d.k_g), ("g_max", d.g_max)]
            }
            Closure::PdfQuench => {
                let d = QuenchPdf::default();
                vec![("s", d.s), ("c_opt", d.c_opt), ("k_g", d.k_g), ("g_max", d.g_max),
                     ("tau_res", d.tau_res), ("b_u", d.b_u)]
            }
            Closure::Pocket => {
                let d = PocketQuenchPdf::default();
                vec![("s", d.s), ("c_opt", d.c_opt), ("k_g", d.k_g), ("g_max", d.g_max),
                     ("tau_res", d.tau_res), ("b_u", d.b_u)]
            }
            Closure::Transported => {
                let d = TransportedPdf::default();
                vec![("s", d.s), ("c_opt", d.c_opt), ("c_phi", d.c_phi), ("da_opt", d.da_opt),
                     ("w_cov", d.w_cov), ("tau_mix", d.tau_mix)]
            }
            Closure::Spatial => {
                let d = SpatialPdf::default();
                vec![("s", d.s), ("k_p", d.k_p), ("k_y", d.k_y), ("k_z", d.k_z)]
            }
            Closure::SpatialDwell => {
                let d = SpatialDwellPdf::default();
                vec![("s", d.s), ("k_p", d.k_p), ("k_y", d.k_y), ("k_z", d.k_z)]
            }
            Closure::SpatialLocal => {
                let d = SpatialLocalPdf::default();
                vec![("s", d.s), ("k_p", d.k_p), ("k_y", d.k_y), ("k_z", d.k_z)]
            }
        }
    }
}

/// Every knob of the burner view. The design comes separately (slice 1's [`Settings`]).
#[derive(Clone, Debug, PartialEq)]
pub struct BurnerSettings {
    /// Front-zone equivalence ratio (1 = stoichiometric; above 1 rich).
    pub phi_p: f64,
    /// Front-zone residence time, s.
    pub tau: f64,
    /// Rung 19: faster-than-equilibrium O atoms (and, with a finite quench, along the quench — 20).
    pub super_eq_o: bool,
    /// Rung 19: prompt NO, at its imposed reference level `prompt_peak` (g/kg).
    pub prompt: bool,
    pub prompt_peak: f64,
    pub quench: Quench,
    /// Rung 10's quench time, s.
    pub tau_q: f64,
    /// Rung 11's jets: strength `J`, duct height `H` (m), crossflow speed `U_c` (m/s), entrainment
    /// constant `C_e`, schedule shape.
    pub j: f64,
    pub h: f64,
    pub u_c: f64,
    pub c_e: f64,
    pub shape_n: f64,
    pub closure: Closure,
    /// The mixing model's own knobs (always exactly [`Closure::knob_defaults`]'s names, in order).
    pub knobs: Vec<(&'static str, f64)>,
}

impl BurnerSettings {
    /// What the view opens on. The front zone at stoichiometric — rung 8's design case
    /// (`panels::nox`, `phi_primary = 1.0`), the one whose emission index rung 8 anchors to the
    /// ICAO band — with the instant quench; every other value is the model's own default (the jets'
    /// `C_e` is 0.15, `JetMixing`'s and rung 12's stated default — plan § 13.8). `J` opens at 25,
    /// plan § 13.2's measured point, off the default spacing's optimum (16).
    pub fn defaults() -> Self {
        let z = ZonedNoxOpts::default();
        let jm = JetMixing::default();
        BurnerSettings {
            phi_p: 1.0, tau: z.tau, super_eq_o: false, prompt: false,
            prompt_peak: PromptNo::default().peak_ei, quench: Quench::Instant, tau_q: 1e-3,
            j: 25.0, h: jm.h, u_c: jm.u_c, c_e: jm.c_e, shape_n: jm.shape_n,
            closure: Closure::None, knobs: Vec::new(),
        }
    }

    pub fn knob(&self, k: &str) -> f64 {
        self.knobs.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
            .unwrap_or_else(|| panic!("mixing model {:?} has no knob {k:?}", self.closure.key()))
    }

    pub fn to_json(&self) -> Json {
        let knobs = Json::Obj(self.knobs.iter().map(|(k, v)| (k.to_string(), Json::Float(*v))).collect());
        jobj! {
            "phi_p" => self.phi_p, "tau" => self.tau,
            "super_eq_o" => Json::Int(self.super_eq_o as i64), "prompt" => Json::Int(self.prompt as i64),
            "prompt_peak" => self.prompt_peak, "quench" => self.quench.key(), "tau_q" => self.tau_q,
            "J" => self.j, "H" => self.h, "U_c" => self.u_c, "C_e" => self.c_e, "shape_n" => self.shape_n,
            "closure" => self.closure.key(), "knobs" => knobs,
        }
    }

    /// Read the burner knobs; a missing key keeps its default. The mixing model's knobs start at that
    /// model's defaults and take any of ITS names from `"knobs"` — another model's name is an error.
    pub fn from_json(j: &Json) -> Result<Self, String> {
        let mut s = BurnerSettings::defaults();
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
        for (k, into) in [("phi_p", &mut s.phi_p), ("tau", &mut s.tau), ("prompt_peak", &mut s.prompt_peak),
                          ("tau_q", &mut s.tau_q), ("J", &mut s.j), ("H", &mut s.h), ("U_c", &mut s.u_c),
                          ("C_e", &mut s.c_e), ("shape_n", &mut s.shape_n)] {
            num(k, into)?;
        }
        flag("super_eq_o", &mut s.super_eq_o)?;
        flag("prompt", &mut s.prompt)?;
        if let Some(t) = text("quench")? { s.quench = Quench::from_key(&t)?; }
        if let Some(t) = text("closure")? { s.closure = Closure::from_key(&t)?; }
        s.knobs = s.closure.knob_defaults();
        match j.get("knobs") {
            None => {}
            Some(Json::Obj(kv)) => {
                for (k, v) in kv {
                    let x = match v {
                        Json::Float(x) => *x,
                        Json::Int(n) => *n as f64,
                        o => return Err(format!("knob {k:?} must be a number, got {o:?}")),
                    };
                    let slot = s.knobs.iter_mut().find(|(n, _)| n == k).ok_or_else(|| {
                        format!("mixing model {:?} has no knob {k:?}", s.closure.key())
                    })?;
                    slot.1 = x;
                }
            }
            Some(o) => return Err(format!("\"knobs\" must be an object, got {o:?}")),
        }
        Ok(s)
    }

    /// The model's options for these knobs, on the fixed L1 grids.
    pub fn opts(&self) -> ZonedNoxOpts {
        let mut o = ZonedNoxOpts {
            tau: self.tau,
            super_eq_o: self.super_eq_o,
            prompt: if self.prompt { Some(PromptNo { peak_ei: self.prompt_peak, ..PromptNo::default() }) } else { None },
            nsteps: ZELDOVICH_STEPS,
            quench_ngrid: QUENCH_NGRID,
            ..ZonedNoxOpts::default()
        };
        match self.quench {
            Quench::Instant => return o,
            Quench::Time => { o.tau_q = Some(self.tau_q); return o; }
            Quench::Jets => o.mixing = Some(self.jet()),
        }
        let k = |n: &str| self.knob(n);
        match self.closure {
            Closure::None => {}
            Closure::TwoStream => o.unmixedness = Some(Unmixedness {
                s: k("s"), c_opt: k("c_opt"), tau_res: k("tau_res"), k_u: k("k_u"), b_u: k("b_u"), w_max: k("w_max"),
            }),
            Closure::Pdf => o.pdf = Some(MixingPdf {
                s: k("s"), c_opt: k("c_opt"), k_g: k("k_g"), g_max: k("g_max"), n_bell: N_BELL, ..MixingPdf::default()
            }),
            Closure::PdfQuench => o.pdf_quench = Some(QuenchPdf {
                s: k("s"), c_opt: k("c_opt"), k_g: k("k_g"), g_max: k("g_max"), tau_res: k("tau_res"), b_u: k("b_u"),
                n_bell: N_BELL, ..QuenchPdf::default()
            }),
            Closure::Pocket => o.pocket_quench = Some(PocketQuenchPdf {
                s: k("s"), c_opt: k("c_opt"), k_g: k("k_g"), g_max: k("g_max"), tau_res: k("tau_res"), b_u: k("b_u"),
                n_bell: N_BELL_POCKET, ..PocketQuenchPdf::default()
            }),
            Closure::Transported => o.transported = Some(TransportedPdf {
                s: k("s"), c_opt: k("c_opt"), c_phi: k("c_phi"), da_opt: k("da_opt"), w_cov: k("w_cov"),
                tau_mix: k("tau_mix"), n_bell: N_BELL, ..TransportedPdf::default()
            }),
            Closure::Spatial => o.spatial = Some(SpatialPdf {
                s: k("s"), k_p: k("k_p"), k_y: k("k_y"), k_z: k("k_z"), ny: N_PLANE, nz: N_PLANE,
                n_bell: N_BELL, ..SpatialPdf::default()
            }),
            Closure::SpatialDwell => o.spatial_dwell = Some(SpatialDwellPdf {
                s: k("s"), k_p: k("k_p"), k_y: k("k_y"), k_z: k("k_z"), ny: N_PLANE, nz: N_PLANE, nt: N_TIME,
                n_bell: N_BELL_POCKET, ..SpatialDwellPdf::default()
            }),
            Closure::SpatialLocal => o.spatial_local = Some(SpatialLocalPdf {
                s: k("s"), k_p: k("k_p"), k_y: k("k_y"), k_z: k("k_z"), ny: N_PLANE, nz: N_PLANE,
                n_bell: N_BELL_POCKET, ..SpatialLocalPdf::default()
            }),
        }
        o
    }

    pub fn jet(&self) -> JetMixing {
        JetMixing { j: self.j, h: self.h, u_c: self.u_c, c_e: self.c_e, shape_n: self.shape_n }
    }

    /// The best mixing's jet strength for the current model and jets — `J_opt = (C_opt·H/S)²` with
    /// `C_opt` given (rungs 12–18) or worked out from the plume spread (22–24). `None` without a
    /// mixing model (rung 11 has no optimum).
    pub fn j_opt(&self) -> Option<f64> {
        let c_opt = match self.closure {
            Closure::None => return None,
            c if c.derives_optimum() => 1.0 / (4.0 * self.knob("k_p") * self.knob("k_p")),
            _ => self.knob("c_opt"),
        };
        let r = c_opt * self.h / self.knob("s");
        Some(r * r)
    }
}

// ---- the burner inlet: the design, on the equilibrium gas ----------------------------------------

/// What the diagnostics read off the design run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inlet {
    pub tt3: f64,
    pub tt4: f64,
    pub far: f64,
    pub pt4: f64,
    /// Overall equivalence ratio of the whole burner.
    pub phi: f64,
    pub mdot: f64,
    /// Nozzle entry (`Tt9`, `pt9`) and exit static pressure — rung 14's inputs.
    pub tt9: f64,
    pub pt9: f64,
    pub p9: f64,
}

/// The design run the burner reads: slice 1's settings with the gas set to equilibrium, through
/// slice 1's own pre-check and the CLI's own `build_turbojet(…).run(…)`.
pub fn design_run(design: &Settings) -> Result<(Engine, EngineResult, Inlet), DoesNotRun> {
    let mut s = *design;
    s.gas = GasModel::Equilibrium;
    sandbox::precheck(&s)?;
    let engine = build_turbojet(s.gas.gas(), s.pi_c, s.tt4, s.p0, s.losses());
    let r = engine.run(&s.flight(), s.mdot);
    let (st3, st4, st9) = (r.station("3"), r.station("4"), r.station("9"));
    let inlet = Inlet {
        tt3: st3.tt, tt4: st4.tt, far: st4.far, pt4: st4.pt, phi: st4.far / f_stoich(), mdot: s.mdot,
        tt9: st9.tt, pt9: st9.pt, p9: r.p9,
    };
    Ok((engine, r, inlet))
}

/// The checks made before the model runs, in plain words.
pub fn precheck(b: &BurnerSettings, inlet: &Inlet) -> Result<(), DoesNotRun> {
    let no = |m: String| Err(DoesNotRun(m));
    let mut named = vec![("front-zone richness", b.phi_p), ("residence time", b.tau),
                         ("prompt reference level", b.prompt_peak), ("quench time", b.tau_q),
                         ("jet strength J", b.j), ("duct height", b.h), ("crossflow speed", b.u_c),
                         ("entrainment constant", b.c_e), ("schedule shape", b.shape_n)];
    named.extend(b.knobs.iter().map(|&(k, v)| (k, v)));
    if let Some((k, _)) = named.iter().find(|(_, v)| !v.is_finite()) {
        return no(format!("Setting {k} is not a number."));
    }
    if !(b.phi_p > 0.0 && b.phi_p <= PHI_MAX) {
        return no(format!("The front zone's richness must be above 0 and at most {PHI_MAX}: richer than that, \
                           the flame makes soot, which this model does not include."));
    }
    // The model's own bar (`zoned_nox`: α ≤ 1 + 1e-9), read as a richness.
    if inlet.far / (b.phi_p * f_stoich()) > 1.0 + 1e-9 {
        return no(format!(
            "The front zone ({:.3}) cannot be leaner than the burner as a whole ({:.3}). All the fuel burns in \
             the front zone, so it gets at most all of the air. Raise the front zone's richness, or lower the \
             design's turbine-inlet temperature.", b.phi_p, inlet.phi));
    }
    Ok(())
}

/// The emission index (g NO per kg fuel) the chosen route reports, THERMAL only: the instant
/// quench's front-zone value, a finite quench's, or the mixing model's own total.
pub fn headline_thermal(b: &BurnerSettings, z: &ZonedNoxState) -> f64 {
    let pick = |x: Option<f64>| x.expect("the chosen route set its field");
    match (b.quench, b.closure) {
        (Quench::Instant, _) => z.ei_no(),
        (_, Closure::None) | (Quench::Time, _) => pick(z.ei_no_quenched),
        (_, Closure::TwoStream) => pick(z.ei_no_unmixed),
        (_, Closure::Pdf) => pick(z.ei_no_pdf),
        (_, Closure::PdfQuench) => pick(z.ei_no_pdf_quench),
        (_, Closure::Pocket) => pick(z.ei_no_pocket_quench),
        (_, Closure::Transported) => pick(z.ei_no_transported),
        (_, Closure::Spatial) => pick(z.ei_no_spatial),
        (_, Closure::SpatialDwell) => pick(z.ei_no_spatial_dwell),
        (_, Closure::SpatialLocal) => pick(z.ei_no_spatial_local),
    }
}

pub struct BurnerOutcome {
    pub inlet: Inlet,
    pub z: ZonedNoxState,
    /// Rung 7: the NO a perfectly mixed burner would make at `Tt4` (same residence, no lift).
    pub ei_mixed: f64,
}

/// One burner: the design run, then ONE `zoned_nox` — and rung 7's perfectly-mixed number beside it.
pub fn burner(design: &Settings, b: &BurnerSettings) -> Result<BurnerOutcome, DoesNotRun> {
    burner_on(design, b).map(|(_, o)| o)
}

/// [`burner`], keeping the design's engine (its equilibrium gas) for the nozzle readout.
fn burner_on(design: &Settings, b: &BurnerSettings) -> Result<(Engine, BurnerOutcome), DoesNotRun> {
    let (engine, _, inlet) = design_run(design)?;
    precheck(b, &inlet)?;
    let z = engine.gas.zoned_nox(inlet.far, inlet.tt3, inlet.tt4, inlet.pt4, b.phi_p, b.opts());
    let mixed = engine.gas.thermal_nox(inlet.far, inlet.tt4, inlet.pt4,
                                       ThermalNoxOpts { tau: b.tau, nsteps: ZELDOVICH_STEPS, ..ThermalNoxOpts::default() });
    Ok((engine, BurnerOutcome { inlet, z, ei_mixed: mixed.ei_no }))
}

fn fin(x: f64, what: &str) -> Result<Json, DoesNotRun> {
    if x.is_finite() { Ok(Json::Float(x)) } else { Err(DoesNotRun(format!("The model gave a non-finite {what}."))) }
}

fn fin_opt(x: Option<f64>, what: &str) -> Result<Json, DoesNotRun> {
    x.map_or(Ok(Json::Null), |v| fin(v, what))
}

pub fn inlet_json(i: &Inlet) -> Json {
    jobj! {
        "Tt3" => i.tt3, "Tt4" => i.tt4, "far" => i.far, "pt4" => i.pt4, "phi" => i.phi,
        "mdot" => i.mdot, "fuel_flow" => i.far * i.mdot,
    }
}

/// The page's view of one burner: every number the model returned, full precision.
pub fn burner_json(b: &BurnerSettings, o: &BurnerOutcome) -> Result<Json, DoesNotRun> {
    let z = &o.z;
    let thermal = headline_thermal(b, z);
    let total = thermal + z.ei_no_prompt;
    let fuel = o.inlet.far * o.inlet.mdot;
    let mut fields = vec![
        ("ok".to_string(), Json::Int(1)),
        ("burner".to_string(), b.to_json()),
        ("inlet".to_string(), inlet_json(&o.inlet)),
        ("ei_thermal".to_string(), fin(thermal, "emission index")?),
        ("ei_prompt".to_string(), fin(z.ei_no_prompt, "prompt emission index")?),
        ("ei_total".to_string(), fin(total, "emission index")?),
        ("nox_flow".to_string(), fin(total * fuel, "NOx flow")?),
        ("ei_mixed".to_string(), fin(o.ei_mixed, "perfectly-mixed emission index")?),
        ("j_opt".to_string(), fin_opt(if b.quench == Quench::Jets { b.j_opt() } else { None }, "optimum")?),
        ("tau_q_jet".to_string(), fin(b.jet().tau_q(), "quench time")?),
    ];
    let scalars: [(&str, Option<f64>); 46] = [
        ("phi_primary", Some(z.phi_primary)), ("far_primary", Some(z.far_primary)), ("alpha", Some(z.alpha)),
        ("t_primary", Some(z.t_primary)), ("t_mix", Some(z.t_mix)), ("ei_no", Some(z.ei_no())),
        ("x_no_primary", Some(z.primary.x_no)), ("ppm_primary", Some(z.ppm_primary())),
        ("fraction_of_equil", Some(z.primary.fraction_of_equil())), ("x_no_mix", Some(z.x_no_mix)),
        ("ppm_mix", Some(z.ppm_mix())), ("o_multiplier", Some(z.o_multiplier)),
        ("ei_no_prompt", Some(z.ei_no_prompt)), ("tau_q", z.tau_q), ("ei_no_quenched", z.ei_no_quenched),
        ("x_no_quenched", z.x_no_quenched), ("t_peak", z.t_peak), ("max_a_quench", z.max_a_quench),
        ("c_holdeman", z.c_holdeman), ("w_core", z.w_core), ("ei_no_unmixed", z.ei_no_unmixed),
        ("ei_no_core", z.ei_no_core), ("g_seg", z.g_seg), ("ei_no_pdf", z.ei_no_pdf),
        ("ei_no_pdf_excess", z.ei_no_pdf_excess), ("ei_no_pdf_quench", z.ei_no_pdf_quench),
        ("ei_no_pocket_excess", z.ei_no_pocket_excess), ("ei_no_pocket_quench", z.ei_no_pocket_quench),
        ("g_ceiling", z.g_ceiling), ("g_transported", z.g_transported), ("ei_no_transported", z.ei_no_transported),
        ("g_spatial", z.g_spatial), ("ei_no_spatial", z.ei_no_spatial), ("g_spatial_dwell", z.g_spatial_dwell),
        ("tau_mean_dwell", z.tau_mean_dwell), ("ei_no_spatial_dwell_excess", z.ei_no_spatial_dwell_excess),
        ("ei_no_spatial_dwell", z.ei_no_spatial_dwell), ("ei_no_spatial_dwell_meanfield", z.ei_no_spatial_dwell_meanfield),
        ("corr_ratio", z.corr_ratio), ("g_spatial_local", z.g_spatial_local), ("f_shape", z.f_shape),
        ("tau_mean_local", z.tau_mean_local), ("ei_no_spatial_local_excess", z.ei_no_spatial_local_excess),
        ("ei_no_spatial_local", z.ei_no_spatial_local), ("ei_no_spatial_local_meanfield", z.ei_no_spatial_local_meanfield),
        ("corr_ratio_local", z.corr_ratio_local),
    ];
    let mut state = Vec::new();
    for (k, v) in scalars {
        state.push((k.to_string(), fin_opt(v, k)?));
    }
    fields.push(("state".to_string(), Json::Obj(state)));
    Ok(Json::Obj(fields))
}

/// The exhaust NO mole fraction to carry through the nozzle (rung 14), THERMAL only — prompt is
/// kept out, as rung 17 keeps it out of its margin (`ZonedNoxState::ei_no_quenched_total`). The
/// instant quench has its own (`x_no_mix`), a finite quench too (`x_no_quenched`); a mixing model
/// gives only an emission index, so it is converted the way rung 17's `exhaust_no_clamp` converts
/// its per-pocket mean: `x = κ·EI`, `κ` = the bulk quench's `x_no/EI` (the same fuel-air ratio, so
/// the same moles per kilogram of fuel).
pub fn exhaust_x_no(b: &BurnerSettings, z: &ZonedNoxState) -> Option<f64> {
    match (b.quench, b.closure) {
        (Quench::Instant, _) => Some(z.x_no_mix),
        (_, Closure::None) | (Quench::Time, _) => z.x_no_quenched,
        _ => {
            let (x, ei) = (z.x_no_quenched?, z.ei_no_quenched?);
            if ei > 0.0 { Some(x / ei * headline_thermal(b, z)) } else { None }
        }
    }
}

/// Rung 14 on this burner: the jet expanded frozen and in equilibrium, and how the exhaust NO sits
/// against its own equilibrium at the nozzle exit. A separate request, so a failure here leaves the
/// burner's numbers standing.
pub fn nozzle_json(design: &Settings, b: &BurnerSettings) -> Result<Json, DoesNotRun> {
    let (engine, o) = burner_on(design, b)?;
    let i = o.inlet;
    let x = exhaust_x_no(b, &o.z);
    let nf = engine.gas.nozzle_flow(i.far, i.tt4, i.pt4, i.tt9, i.pt9, i.p9, x);
    Ok(jobj! {
        "ok" => Json::Int(1),
        "x_no_exhaust" => fin_opt(x, "exhaust NO")?,
        "t9_frozen" => fin(nf.t9_frozen, "jet temperature")?, "t9_equilibrium" => fin(nf.t9_equilibrium, "jet temperature")?,
        "v9_frozen" => fin(nf.v9_frozen, "jet speed")?, "v9_equilibrium" => fin(nf.v9_equilibrium, "jet speed")?,
        "dv9" => fin(nf.dv9(), "jet speed gain")?, "dv9_frac" => fin(nf.dv9_frac(), "jet speed gain")?,
        "co_fraction_entry" => fin(nf.co_fraction_entry, "CO share")?,
        "x_no_e_entry" => fin(nf.x_no_e_entry, "equilibrium NO")?, "x_no_e_exit" => fin(nf.x_no_e_exit, "equilibrium NO")?,
        "no_collapse_ratio" => fin(nf.no_collapse_ratio, "NO collapse")?,
        "max_a" => fin_opt(nf.max_a, "NO over its equilibrium")?,
    })
}

/// The quench path — temperature against mixing progress `β` — for the front zone the knobs make
/// (`quench_trajectory`, rung 10), on the fixed grid. It exists for every quench: the instant one
/// takes it in no time, a finite one along a schedule, so the same path is the backdrop for all.
pub fn path_json(design: &Settings, b: &BurnerSettings) -> Result<Json, DoesNotRun> {
    let (engine, _, i) = design_run(design)?;
    precheck(b, &i)?;
    let z = engine.gas.zoned_nox(i.far, i.tt3, i.tt4, i.pt4, b.phi_p,
                                 ZonedNoxOpts { tau: b.tau, nsteps: ZELDOVICH_STEPS, ..ZonedNoxOpts::default() });
    let comp = equilibrium_composition(z.far_primary, z.t_primary, i.pt4);
    let path = quench_trajectory(&comp, z.t_primary, z.alpha, i.far, i.tt3, i.pt4, QUENCH_NGRID);
    let n = path.len();
    let mut pts = Vec::new();
    for (k, q) in path.iter().enumerate() {
        let beta = k as f64 / (n - 1) as f64;
        // The local equivalence ratio at this point: `far/a(β)` over stoichiometric.
        pts.push(Json::List(vec![Json::Float(beta), fin(q.t, "path temperature")?,
                                 fin(i.far / q.a / f_stoich(), "path richness")?]));
    }
    Ok(jobj! { "ok" => Json::Int(1), "t_primary" => z.t_primary, "t_mix" => z.t_mix, "path" => Json::List(pts) })
}

/// The values a sweep steps through: front-zone richness from the burner's own richness to the soot
/// limit, or jet strength log-spaced with the mixing model's optimum added (a sweep that steps over
/// `J_opt` misses rungs 12/13's notch — plan § 13.2).
pub fn sweep_grid(b: &BurnerSettings, inlet: &Inlet, axis: &str) -> Result<Vec<f64>, String> {
    match axis {
        "phi" => {
            // From just above the burner's own richness (there the front zone IS the burner, and the
            // cross-plane models refuse a front zone that is not richer than the mean) to the soot limit.
            let lo = inlet.phi;
            let n = PHI_SWEEP_POINTS;
            Ok((1..=n).map(|k| if k == n { PHI_MAX } else { lo + (PHI_MAX - lo) * k as f64 / n as f64 }).collect())
        }
        "J" => {
            let (a, z) = J_SWEEP;
            let n = J_SWEEP_POINTS;
            let mut g: Vec<f64> = (0..n).map(|k| a * (z / a).powf(k as f64 / (n - 1) as f64)).collect();
            if let Some(jo) = b.j_opt() {
                if jo > a && jo < z && !g.contains(&jo) {
                    g.push(jo);
                    g.sort_by(|x, y| x.partial_cmp(y).unwrap());
                }
            }
            Ok(g)
        }
        other => Err(format!("unknown sweep axis {other:?} (\"phi\" or \"J\")")),
    }
}

/// Plain words for a message the model panicked with while running the burner. Each entry matches
/// a message the crash map (plan § 13.9: designs × every burner knob × every mixing model, through
/// [`crate::sandbox::call`]) actually produced, and `tests/sandbox_burner.rs` drives each one. A
/// design that does not run on the equilibrium gas at all keeps slice 1's words, said as such.
pub fn explain_burner(message: &str) -> String {
    const NOT_TRACE: &str =
        "The front zone burns so hot that NO stops being a trace gas: at equilibrium it would be more than 2 % \
         of the gas. The model treats NO as a trace that does not change the flame, and that stops being true \
         here. Move the front zone's richness away from 1, or lower the pressure ratio or the flight speed so \
         the air arrives cooler.";
    const NOT_TRACE_PATH: &str =
        "On the way through the dilution, the gas passes a mixture so hot that NO stops being a trace gas (more \
         than 2 % at equilibrium). The model treats NO as a trace that does not change the flame, and that stops \
         being true there. Lower the pressure ratio or the flight speed so the air arrives cooler.";
    const FAST_O: &str =
        "The fast-O-atom correction is fitted to flames hotter than about 1500 K, and this front zone burns \
         cooler. Switch it off, or move the front zone's richness towards 1 so it burns hotter.";
    const NOZZLE_COLD: &str =
        "The nozzle readout cannot follow this jet: the jet leaves the nozzle colder than 500 K, below where the \
         readout's expansion search starts. The burner's numbers above are unaffected.";
    const NOZZLE_NEWTON: &str =
        "The nozzle readout's equilibrium solve did not settle on the expanding jet at this state. The burner's \
         numbers above are unaffected.";
    const WIDE_PDF: &str =
        "The mixing spread is too wide for this mixing model: with the burner's mixture this lean, the model's \
         spread-of-mixtures curve cannot be summed accurately (its own check fails). Lower the spread cap (the \
         model's default, 0.3, is the largest the sandbox offers on its slider).";
    const PLANE_SHORT: &str =
        "The dilution jets cannot spread their air evenly enough. To bring the rich front-zone gas down to the burner's \
         overall mixture, most of the cross-section has to take in jet air, and with plumes this narrow some of it never \
         gets any. Make the front zone less rich, or widen the plumes (raise the spread lengths).";
    const KNOWN: &[(&str, &str)] = &[
        ("β-PDF shape", WIDE_PDF),
        ("β-PDF quadrature drifted the mean", WIDE_PDF),
        ("field drifted the mean", PLANE_SHORT),
        ("must be RICHER than the overall mean",
         "This mixing model needs a front zone richer than the burner as a whole: the jets dilute the front zone's gas \
          down to the burner's mixture. Raise the front zone's richness above the burner's."),
        ("primary_aft: flame temp",
         "The front zone's flame temperature falls outside the model's search (800 to 3200 K). Move the front zone's \
          richness, or the design's pressure ratio or flight speed."),
        ("super-eq O multiplier", FAST_O),
        ("summed primary NO not trace", NOT_TRACE),
        ("NO not trace on quench path", NOT_TRACE_PATH),
        ("NO not trace (", NOT_TRACE),
        ("nozzle exit T=", NOZZLE_COLD),
        ("equilibrium Newton did not converge", NOZZLE_NEWTON),
    ];
    if let Some((_, v)) = KNOWN.iter().find(|(k, _)| message.contains(k)) {
        return v.to_string();
    }
    // The dilution path's temperature search ([700, 3200] K): which edge it hit says which way.
    if message.contains("mixed_out_t: mix temp") {
        let hot = message.split("mix temp ").nth(1).and_then(|r| r.split(' ').next())
            .and_then(|v| v.parse::<f64>().ok()).is_some_and(|t| t > 2000.0);
        return if hot {
            "On the way through the dilution, the rich front-zone gas passes stoichiometric with air this hot, and \
             the mixture would go above 3200 K — past the top of the model's temperature search. Lower the pressure \
             ratio or the flight speed so the air arrives cooler, or make the front zone less rich."
        } else {
            "On the way through the dilution, the mixture would fall below 700 K — past the bottom of the model's \
             temperature search."
        }.to_string();
    }
    let general = sandbox::explain("");
    let design = sandbox::explain(message);
    if design != general {
        return format!("This design does not run on the equilibrium gas, which the burner always reads (rung 6's \
                        chemistry): {design}");
    }
    "The model stopped on one of its internal checks for this burner.".to_string()
}

fn refusal(m: &str) -> Json { jobj! { "ok" => Json::Int(0), "reason" => m } }

/// The burner requests the browser check runs natively AND in the browser build (plan § 13.6): the
/// page's opening burner, then four designs (plan § 13.2's) × every route but the per-pocket three,
/// rich (`φ_p` 1.5) at `J` 25 with the fast-O and prompt switches on half the time; each design's inlet,
/// quench path, nozzle readout and both sweep grids; the β-PDF model AT its notch; and ONE per-pocket
/// point (rung 16 at the default design — seconds per build, so one). Every one runs natively.
pub fn check_requests() -> Vec<String> {
    let req = |op: &str, design: &Json, b: &str| format!("{{\"op\":\"{op}\",\"settings\":{},\"burner\":{b}}}", design.dump_compact());
    let mut out = vec![r#"{"op":"burner","settings":{},"burner":{}}"#.to_string()];
    let designs = [(10.0, 1500.0), (4.0, 1000.0), (20.0, 1900.0), (40.0, 1600.0)];
    let routes = ["\"quench\":\"instant\"", "\"quench\":\"time\"", "\"quench\":\"jets\"",
                  "\"quench\":\"jets\",\"closure\":\"two_stream\"", "\"quench\":\"jets\",\"closure\":\"pdf\"",
                  "\"quench\":\"jets\",\"closure\":\"pdf_quench\"", "\"quench\":\"jets\",\"closure\":\"transported\"",
                  "\"quench\":\"jets\",\"closure\":\"spatial\""];
    for (k, &(pi_c, tt4)) in designs.iter().enumerate() {
        let d = jobj! { "pi_c" => pi_c, "Tt4" => tt4 };
        out.push(req("burner_inlet", &d, "{}"));
        for (r, route) in routes.iter().enumerate() {
            let sw = if (k + r) % 2 == 1 { ",\"super_eq_o\":1,\"prompt\":1" } else { "" };
            out.push(req("burner", &d, &format!("{{\"phi_p\":1.5,\"J\":25,{route}{sw}}}")));
        }
        out.push(req("burner_path", &d, "{\"phi_p\":1.5}"));
        out.push(req("burner_nozzle", &d, "{\"phi_p\":1.5,\"quench\":\"jets\",\"closure\":\"two_stream\"}"));
        out.push(req("burner_grid", &d, "{}").replace("\"burner\":{}", "\"burner\":{},\"axis\":\"phi\""));
        out.push(req("burner_grid", &d, "{\"quench\":\"jets\",\"closure\":\"spatial\"}")
            .replace("\"burner\":{", "\"axis\":\"J\",\"burner\":{"));
    }
    let empty = Json::Obj(Vec::new());
    out.push(req("burner", &empty, "{\"phi_p\":1.5,\"quench\":\"jets\",\"closure\":\"pdf\",\"J\":16}"));
    out.push(req("burner", &empty, "{\"phi_p\":1.5,\"J\":25,\"quench\":\"jets\",\"closure\":\"pocket\"}"));
    out
}

/// Slice 5's ops, reached through [`crate::sandbox::call`]:
/// - `burner_defaults` → the opening knobs, the mixing models' table, the fixed grids.
/// - `burner_inlet` (`"settings"`: the design) → the inlet the burner reads.
/// - `burner` (`"settings"`, `"burner"`) → one burner, every field.
/// - `burner_path` → the quench path; `burner_nozzle` → rung 14's nozzle readout.
/// - `burner_grid` (+ `"axis"`: `"phi"` / `"J"`) → a sweep's values (the page asks `burner` at each).
pub fn call_op(op: &str, req: &Json) -> Option<Json> {
    if !op.starts_with("burner") {
        return None;
    }
    if op == "burner_defaults" {
        let models: Vec<Json> = Closure::ALL.iter().map(|&c| {
            let knobs = Json::Obj(c.knob_defaults().into_iter().map(|(k, v)| (k.to_string(), Json::Float(v))).collect());
            jobj! { "key" => c.key(), "rung" => Json::Int(c.rung()), "cost" => c.cost(),
                    "derives_optimum" => Json::Int(c.derives_optimum() as i64), "knobs" => knobs }
        }).collect();
        return Some(jobj! {
            "burner" => BurnerSettings::defaults().to_json(), "models" => Json::List(models),
            "phi_max" => PHI_MAX, "j_sweep" => vec![J_SWEEP.0, J_SWEEP.1],
            "grid" => jobj! { "zeldovich_steps" => Json::Int(ZELDOVICH_STEPS as i64),
                              "quench_ngrid" => Json::Int(QUENCH_NGRID as i64),
                              "quench_nsteps" => Json::Int(ZonedNoxOpts::default().quench_nsteps as i64),
                              "n_bell" => Json::Int(N_BELL as i64), "n_bell_pocket" => Json::Int(N_BELL_POCKET as i64),
"n_plane" => Json::Int(N_PLANE as i64),
                              "n_time" => Json::Int(N_TIME as i64) },
        });
    }
    let empty = Json::Obj(Vec::new());
    let design = match Settings::from_json(req.get("settings").unwrap_or(&empty)) {
        Ok(s) => s,
        Err(e) => return Some(refusal(&e)),
    };
    let b = match BurnerSettings::from_json(req.get("burner").unwrap_or(&empty)) {
        Ok(b) => b,
        Err(e) => return Some(refusal(&e)),
    };
    let done = |r: Result<Json, DoesNotRun>| r.unwrap_or_else(|DoesNotRun(m)| refusal(&m));
    Some(match op {
        "burner_inlet" => done(design_run(&design).map(|(_, _, i)| {
            let mut j = inlet_json(&i);
            if let Json::Obj(kv) = &mut j { kv.insert(0, ("ok".to_string(), Json::Int(1))); }
            j
        })),
        "burner" => done(burner(&design, &b).and_then(|o| burner_json(&b, &o))),
        "burner_path" => done(path_json(&design, &b)),
        "burner_nozzle" => done(nozzle_json(&design, &b)),
        "burner_grid" => {
            let axis = match req.get("axis") { Some(Json::Str(a)) => a.clone(), _ => String::new() };
            done(design_run(&design).and_then(|(_, _, i)| {
                sweep_grid(&b, &i, &axis).map(Json::from).map_err(DoesNotRun)
            }))
        }
        other => refusal(&format!("unknown op {other:?}")),
    })
}
