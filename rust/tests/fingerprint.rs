//! SLICE AS — THE NUMERIC FINGERPRINT, RE-ANCHORED ON RUST (plan § 8.1 (vi)).
//!
//! `tests/test_numeric_fingerprint.py` is the project's only ABSOLUTE-value gate: 45 kernels,
//! each a fixed computation whose every value is compared against a committed CPython golden
//! (`tests/golden/numeric_fingerprint.json`) under a per-kernel tolerance measured as one round
//! decade above the CPython-vs-PyPy drift. Every OTHER gate in the crate compares Rust with PyPy
//! bit for bit; this is the one that ties the numbers to something outside the two
//! implementations' agreement with each other.
//!
//! Each kernel here is the Python kernel of the same name, ported call for call, emitting the
//! SAME keys (`_floats_of`'s float attributes, `_flat`'s `.k` / `#n` / `[i]` / `.re` / `.im`).
//! Each is checked TWICE:
//!
//! 1. **bit-exact against PyPy** — `rust/oracle/fingerprint_pypy.tsv`, written by
//!    `rust/oracle/dump_fingerprint.py` from the Python module's own `KERNELS` table, so the key
//!    set is the gate's own and not a hand reconstruction. A missing / extra / renamed key or a
//!    last-bit difference fails here, at its name.
//! 2. **within tolerance against CPython** — the committed golden, under the module's own `TOL` /
//!    `ABS_TOL` (transcribed, and the transcription checked against the values the dump wrote
//!    from the module itself) and its `_close`, verbatim.
//!
//! Python's `_UNSTABLE` keys (rung 76's three `0/0` readings) are dropped by the Python kernel
//! before it returns, so they are in neither golden and the port never emits them.

mod fingerprint_support;
mod slice_aj_flat;

use fingerprint_support::*;

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{hf_fuel_default, Gas, GasSpec};
use turbojet::march::{CoupledNoFreezeOut, FiniteRate, FreezeOut, NoFreezeOut};
use turbojet::matcher::{OffDesignMatcher, OffDesignResult};
use turbojet::nox::{
    spatial_local_field, ExhaustClampOpts, JetMixing, MixingPdf, NoxState, PocketQuenchPdf,
    PromptNo, QuenchPdf, SpatialDwellPdf, SpatialLocalPdf, SpatialPdf, ThermalNoxOpts,
    TransportedPdf, Unmixedness, ZonedNoxOpts, ZonedNoxState,
};
use turbojet::pyfmt::{fstring, repr_f64};
use turbojet::bleed_transient::LeverArm;
use turbojet::fuel_transient::{
    AsymmetricLag, Authority, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{MarchScope, Ramp, ScheduledStatorTransient, StatorLeg};
use turbojet::two_lag::build_two_lag_cascade;
use turbojet::cross_loop::{build_cross_loop_cascade, OscRow, Window};
use turbojet::reference_split::{
    build_reference_split_cascade, reference_gains, reference_modes, RefModesArm,
    StatorIncidenceLimiter, C64,
};
use turbojet::cross_split::{build_cross_split_cascade, split_gains, split_modes, StateBoundary};
use turbojet::full_split::{build_full_split_cascade, full_gains, full_modes};
use turbojet::shared_actuator::{
    build_shared_actuator_cascade, charpoly_selftest, shared_gains, BoundaryCheck, QuadGains,
};
use turbojet::applied_reference::{applied_gains, build_applied_reference_cascade};
use turbojet::demand_coordinate::{build_demand_coordinate_cascade, demand_gains};
use turbojet::anti_windup::{build_anti_windup_cascade, windup_gains, WindupCell};
use turbojet::sensed_cap::{build_sensed_cap_cascade, cap_gains, CapCell};
use turbojet::split_wall::{
    build_split_wall_cascade, split_arrest, split_gains as wall_gains, split_liveness,
    split_saturation, SplitGains, SplitRow,
};
use turbojet::state_coordinate::{
    build_state_coordinate_cascade, coord_census, coord_forced, coord_march, coord_scan,
};
use turbojet::residual_gauge::{
    build_residual_gauge_cascade, gauge_scan, gauge_vs_device, root_census, GAUGE_SCAN_MULTS,
    ROOT_CENSUS_MULTS,
};
use turbojet::authority_clock::{authority_clock, authority_mask};
use turbojet::threshold_law;
use turbojet::stiffness_ledger::{
    build_stiffness_ledger_cascade, leg_slopes, set_point_gains, singular_limit, Leg,
};
use turbojet::three_loop::{
    build_three_loop_cascade, triple_gains, triple_modes, StatorLimiter, TripleGains, TripleRigArm,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ------------------------------------------------------------------------- shared conditions

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;

fn losses() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}

/// `_cpg_gas()`: the self-consistent CPG dual gas, `R_t = (g-1)/g*cp` exactly.
fn cpg_gas() -> Gas {
    let (g, cp) = (1.3, 1239.0);
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: g, cp_t: cp, r_t: (g - 1.0) / g * cp,
        hpr: 42.8e6,
        ..GasSpec::default()
    })
}

/// Python's `f"{x:g}"`.
fn g(x: f64) -> String { fstring("{:g}", &[&x]) }

fn od_floats(out: &mut Kernel, p: &str, r: &OffDesignResult) {
    for (k, v) in [
        ("M0", r.m0), ("M9", r.m9), ("T9", r.t9), ("Tt4", r.tt4), ("V0", r.v0), ("V9", r.v9),
        ("mdot_air", r.mdot_air), ("mdot_ratio", r.mdot_ratio), ("p9", r.p9), ("pi_c", r.pi_c),
        ("pi_t", r.pi_t), ("tau_c", r.tau_c), ("tau_t", r.tau_t), ("thrust", r.thrust),
    ] {
        out.f(format!("{p}.{k}"), v);
    }
}

// ------------------------------------------------------------------------- slice 1

fn kernel_cpg() -> Kernel {
    let mut out = Kernel::default();
    let fl = flight();
    let design = build_turbojet(
        cpg_gas(), PI_C, TT4, fl.p0, Losses { nozzle_convergent: true, ..losses() },
    );
    let r = design.run(&fl, 1.0);
    for (k, st) in &r.stations {
        out.f(format!("design.st{k}.Tt"), st.tt);
        out.f(format!("design.st{k}.pt"), st.pt);
    }
    let m = OffDesignMatcher::new(design, fl, 1.0);
    for tt4 in [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0] {
        od_floats(&mut out, &format!("od{}", tt4 as i64), &m.match_point(&fl, tt4));
    }
    out
}

/// `_equilibrium_design()` / `_dp()`: the shared equilibrium design run. The engine is RETURNED
/// because Python's kernels read the gas the run froze (`gas` is the object `build_turbojet` was
/// handed), and in Rust that gas lives on the engine.
struct Dp {
    eng: turbojet::engine::Engine,
    res: turbojet::engine::EngineResult,
}

impl Dp {
    fn new() -> Dp {
        let fl = flight();
        let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, fl.p0, losses());
        let res = eng.run(&fl, 50.0);
        Dp { eng, res }
    }
    fn gas(&self) -> &Gas { &self.eng.gas }
    fn far(&self) -> f64 { self.res.station("4").far }
    fn tt3(&self) -> f64 { self.res.station("3").tt }
    fn tt4(&self) -> f64 { self.res.station("4").tt }
    fn pt4(&self) -> f64 { self.res.station("4").pt }
    fn tt5(&self) -> f64 { self.res.station("5").tt }
    fn pt5(&self) -> f64 { self.res.station("5").pt }
}

fn kernel_a() -> Kernel {
    let dp = Dp::new();
    let mut out = Kernel::default();
    for (k, st) in &dp.res.stations {
        out.f(format!("st{k}.Tt"), st.tt);
        out.f(format!("st{k}.pt"), st.pt);
        out.f(format!("st{k}.far"), st.far);
    }
    // Python also reads `thrust` / `tsfc` / `f` / `eta_*` / `v9` IF `hasattr(res, …)` — and the
    // golden carries none of them: `EngineResult` has none of those names. Nothing to port.
    out
}

fn mix(j: f64) -> JetMixing { JetMixing { j, c_e: 0.20, u_c: 75.0, shape_n: 2.0, ..JetMixing::default() } }

fn kernel_b() -> Kernel {
    let mut out = Kernel::default();
    let m = mix(16.0);
    let (g_field, tau_of_xi, f) =
        spatial_local_field(0.02718, 1.5, 0.0625, 0.10, 16.0, m.tau_q(), 0.316, 0.28, 0.28, 32, 32);
    out.f("g_spatial", g_field);
    out.f("F_shape", f);
    out.f("tau_q", m.tau_q());
    for k in 0..21 {
        out.f(format!("tau_of_xi[{k}]"), tau_of_xi.at(0.001 + k as f64 * (0.070 - 0.001) / 20.0));
    }
    for jj in [4.0, 9.0, 36.0, 64.0] {
        let (gj, _, fj) = spatial_local_field(
            0.02718, 1.5, 0.0625, 0.10, jj, mix(jj).tau_q(), 0.316, 0.28, 0.28, 32, 32,
        );
        out.f(format!("J{}.g", jj as i64), gj);
        out.f(format!("J{}.F", jj as i64), fj);
    }
    out
}

/// `_floats_of` on a `ZonedNOxState`: every float field that is SET, plus the five properties.
fn zoned_floats(out: &mut Kernel, p: &str, z: &ZonedNoxState) {
    for (k, v) in [
        ("phi_primary", z.phi_primary), ("far_primary", z.far_primary), ("alpha", z.alpha),
        ("T_primary", z.t_primary), ("T_mix", z.t_mix), ("x_no_mix", z.x_no_mix),
        ("o_multiplier", z.o_multiplier), ("ei_no_prompt", z.ei_no_prompt),
        ("ei_no", z.ei_no()), ("ei_no_total", z.ei_no_total()),
        ("ppm_primary", z.ppm_primary()), ("ppm_mix", z.ppm_mix()),
    ] {
        out.f(format!("{p}.{k}"), v);
    }
    for (k, v) in [
        ("tau_q", z.tau_q), ("ei_no_quenched", z.ei_no_quenched),
        ("x_no_quenched", z.x_no_quenched), ("T_peak", z.t_peak),
        ("max_a_quench", z.max_a_quench), ("C_holdeman", z.c_holdeman), ("w_core", z.w_core),
        ("ei_no_unmixed", z.ei_no_unmixed), ("ei_no_core", z.ei_no_core), ("g_seg", z.g_seg),
        ("ei_no_pdf", z.ei_no_pdf), ("ei_no_pdf_excess", z.ei_no_pdf_excess),
        ("ei_no_pdf_quench", z.ei_no_pdf_quench), ("ei_no_pocket_excess", z.ei_no_pocket_excess),
        ("ei_no_pocket_quench", z.ei_no_pocket_quench), ("g_ceiling", z.g_ceiling),
        ("g_transported", z.g_transported), ("ei_no_transported", z.ei_no_transported),
        ("g_spatial", z.g_spatial), ("ei_no_spatial", z.ei_no_spatial),
        ("g_spatial_dwell", z.g_spatial_dwell), ("tau_mean_dwell", z.tau_mean_dwell),
        ("ei_no_spatial_dwell_excess", z.ei_no_spatial_dwell_excess),
        ("ei_no_spatial_dwell", z.ei_no_spatial_dwell),
        ("ei_no_spatial_dwell_meanfield", z.ei_no_spatial_dwell_meanfield),
        ("corr_ratio", z.corr_ratio), ("g_spatial_local", z.g_spatial_local),
        ("f_shape", z.f_shape), ("tau_mean_local", z.tau_mean_local),
        ("ei_no_spatial_local_excess", z.ei_no_spatial_local_excess),
        ("ei_no_spatial_local", z.ei_no_spatial_local),
        ("ei_no_spatial_local_meanfield", z.ei_no_spatial_local_meanfield),
        ("corr_ratio_local", z.corr_ratio_local),
        ("ei_no_quenched_total", z.ei_no_quenched_total()),
    ] {
        out.of(format!("{p}.{k}"), v);
    }
}

fn kernel_c() -> Kernel {
    let dp = Dp::new();
    let z = dp.gas().zoned_nox(
        dp.far(), dp.tt3(), dp.tt4(), dp.pt4(), 1.5,
        ZonedNoxOpts {
            tau: 3e-3, mixing: Some(mix(16.0)), quench_ngrid: 12, quench_nsteps: 40,
            ..ZonedNoxOpts::default()
        },
    );
    let mut out = Kernel::default();
    zoned_floats(&mut out, "z", &z);
    out
}

fn kernel_d() -> Kernel {
    let dp = Dp::new();
    let gas = dp.gas();
    let far = dp.far();
    let delta_h = gas.h_t(dp.tt4(), far) - gas.h_t(dp.tt5(), far);
    let s = gas.shifting_turbine(far, dp.tt4(), dp.pt4(), delta_h);
    let mut out = Kernel::default();
    for (k, v) in [
        ("T5_frozen", s.t5_frozen), ("T5_shifting", s.t5_shifting), ("dT5", s.dt5()),
        ("dT5_fraction", s.dt5_fraction()), ("delta_h", s.delta_h),
        ("dp5_fraction", s.dp5_fraction()), ("p5_frozen", s.p5_frozen),
        ("p5_shifting", s.p5_shifting), ("radical_inventory", s.radical_inventory),
        ("super_eq_ratio_max", s.super_eq_ratio_max),
    ] {
        out.f(format!("sh.{k}"), v);
    }
    out
}

fn kernel_e() -> Kernel {
    let fl = flight();
    let design = build_turbojet(
        Gas::reacting_equilibrium(), PI_C, TT4, fl.p0,
        Losses { nozzle_convergent: true, ..losses() },
    );
    let m = OffDesignMatcher::new(design, fl, 1.0);
    let mut out = Kernel::default();
    for tt4 in [1500.0, 1300.0, 1100.0] {
        od_floats(&mut out, &format!("{}", tt4 as i64), &m.match_point(&fl, tt4));
    }
    out
}

fn kernel_f() -> Kernel {
    let dp = Dp::new();
    let fo = dp.gas().freeze_out_nozzle(
        dp.far(), dp.tt4(), dp.pt4(), dp.tt5(), dp.pt5(), flight().p0,
        FreezeOut { l: 0.3, ..FreezeOut::default() },
    );
    let mut out = Kernel::default();
    for (k, v) in [
        ("Da_entry", fo.da_entry), ("Da_exit", fo.da_exit), ("T9_freeze", fo.t9_freeze),
        ("T9_frozen", fo.t9_frozen), ("T9_irrev_fast", fo.t9_irrev_fast),
        ("V9_freeze", fo.v9_freeze), ("V9_frozen", fo.v9_frozen),
        ("V9_irrev_fast", fo.v9_irrev_fast), ("bracket_filled", fo.bracket_filled()),
        ("co_fraction_entry", fo.co_fraction_entry),
        ("co_fraction_freeze_exit", fo.co_fraction_freeze_exit), ("dS_freeze", fo.ds_freeze),
        ("s_freeze", fo.s_freeze),
    ] {
        out.f(format!("fo.{k}"), v);
    }
    out
}

// ------------------------------------------------------------------------- slice 2

/// The slice-2 reduced resolution, verbatim (`_NG, _NS, _NB, _NQ, _NXY, _NT`).
const NG: usize = 24;
const NS: usize = 200;
const NB: usize = 24;
const NQ: usize = 48;
const NXY: usize = 16;
const NT: usize = 12;

fn kernel_prop() -> Kernel {
    const TS: [f64; 8] = [250.0, 500.0, 800.0, 1200.0, 1500.0, 1800.0, 2200.0, 2800.0];
    const FARS: [f64; 4] = [0.0, 0.010, 0.02718, 0.040];
    let mut out = Kernel::default();
    let eq = Gas::reacting_equilibrium_with(hf_fuel_default(), 0.02718);
    eq.freeze_equilibrium(0.02718, 1500.0, 1.4e6);
    let gases: Vec<(&str, Gas)> = vec![
        ("tpg", Gas::thermally_perfect()),
        ("r4", Gas::reacting_with(0.02718, 42.8e6)),
        ("r5", Gas::reacting_forkb_with(hf_fuel_default(), 0.02718)),
        ("r6", eq),
    ];
    for (gname, gs) in &gases {
        for t in TS {
            let tg = g(t);
            out.f(format!("{gname}.cp_c({tg})"), gs.cp_c_at(t));
            out.f(format!("{gname}.h_c({tg})"), gs.h_c(t));
            out.f(format!("{gname}.pr_c({tg})"), gs.pr_c(t));
            out.f(format!("{gname}.gam_c({tg})"), gs.gamma_c_at(t));
            out.f(format!("{gname}.T_of_h_c({tg})"), gs.t_from_h_c(gs.h_c(t)));
            out.f(format!("{gname}.T_of_pr_c({tg})"), gs.t_from_pr_c(gs.pr_c(t)));
        }
        let fars: &[f64] = if *gname == "r6" { &[0.02718] } else { &FARS };
        for &far in fars {
            let fg = g(far);
            out.f(format!("{gname}.R_t({fg})"), gs.r_t_at(far));
            for t in TS {
                let tg = g(t);
                out.f(format!("{gname}.cp_t({tg},{fg})"), gs.cp_t_at(t, far));
                out.f(format!("{gname}.h_t({tg},{fg})"), gs.h_t(t, far));
                out.f(format!("{gname}.pr_t({tg},{fg})"), gs.pr_t(t, far));
                out.f(format!("{gname}.gam_t({tg},{fg})"), gs.gamma_t_at(t, far));
                out.f(format!("{gname}.T_of_h_t({tg},{fg})"), gs.t_from_h_t(gs.h_t(t, far), far));
                out.f(format!("{gname}.T_of_pr_t({tg},{fg})"), gs.t_from_pr_t(gs.pr_t(t, far), far));
            }
        }
    }
    let fb = &gases[2].1;
    out.f("r5.lhv", fb.lhv());
    out.f("r5.f_stoich", fb.f_stoich_lean());
    out.f("r5.hf_fuel_mass", fb.hf_fuel_mass());
    for far in FARS {
        out.f(format!("r5.hf_products({})", g(far)), fb.hf_products_mass(far));
        for t in [1200.0, 1800.0, 2400.0] {
            out.f(format!("r5.h_t_abs({},{})", g(t), g(far)), fb.h_t_abs(t, far));
        }
    }
    let eq = &gases[3].1;
    for t in [1200.0, 1800.0, 2400.0] {
        out.f(format!("r6.h_air_B({})", g(t)), eq.h_air_abs_b(t));
    }
    for (far, t, p) in [(0.010, 1800.0, 1.4e6), (0.02718, 2400.0, 1.4e6), (0.02718, 2400.0, 3.0e5)] {
        let comp = eq.equilibrium_composition(far, t, p);
        let tag = format!("{},{},{}", g(far), g(t), g(p));
        let mut sorted: Vec<&(&str, f64)> = comp.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));
        for (s, x) in sorted {
            out.f(format!("r6.comp({tag}).{s}"), *x);
        }
        out.f(format!("r6.h_prod_B({tag})"), eq.h_products_abs_b(&comp, t));
    }
    out
}

fn nox_floats(out: &mut Kernel, p: &str, n: &NoxState) {
    for (k, v) in [
        ("char_time", n.char_time), ("ei_no", n.ei_no), ("ei_no_prompt", n.ei_no_prompt),
        ("ei_no_total", n.ei_no_total()), ("fraction_of_equil", n.fraction_of_equil()),
        ("initial_rate", n.initial_rate), ("o_multiplier", n.o_multiplier), ("ppm", n.ppm()),
        ("ppm_eq", n.ppm_eq()), ("x_no", n.x_no), ("x_no_eq", n.x_no_eq),
    ] {
        out.f(format!("{p}.{k}"), v);
    }
}

fn kernel_r7() -> Kernel {
    let dp = Dp::new();
    let (gas, far, pt4) = (dp.gas(), dp.far(), dp.pt4());
    let mut out = Kernel::default();
    let base = ThermalNoxOpts { tau: 3e-3, ..ThermalNoxOpts::default() };
    for (tag, o) in [
        ("plain", base),
        ("supereq", ThermalNoxOpts { super_eq_o: true, ..base }),
        ("prompt", ThermalNoxOpts { prompt: Some(PromptNo::default()), phi: Some(1.0), ..base }),
        ("both", ThermalNoxOpts {
            super_eq_o: true, prompt: Some(PromptNo::default()), phi: Some(1.2), ..base
        }),
    ] {
        nox_floats(&mut out, tag, &gas.thermal_nox(far, 2200.0, pt4, o));
    }
    for t in [1800.0, 2000.0, 2400.0] {
        nox_floats(&mut out, &format!("T{}", t as i64), &gas.thermal_nox(far, t, pt4, base));
    }
    out
}

/// `_zoned(cases)`: `zoned_nox` at the shared design point, `phi_primary = 1.5` unless a case
/// overrides it, one closure per case.
fn zoned(cases: Vec<(String, f64, ZonedNoxOpts)>) -> Kernel {
    let dp = Dp::new();
    let mut out = Kernel::default();
    for (tag, phi, o) in cases {
        let z = dp.gas().zoned_nox(dp.far(), dp.tt3(), dp.tt4(), dp.pt4(), phi, o);
        zoned_floats(&mut out, &tag, &z);
    }
    out
}

fn zo() -> ZonedNoxOpts {
    ZonedNoxOpts { tau: 3e-3, quench_ngrid: NG, quench_nsteps: NS, ..ZonedNoxOpts::default() }
}

fn kernel_r8() -> Kernel {
    zoned([0.8, 1.0, 1.2, 1.5, 2.0].iter().map(|&phi| (format!("phi{}", g(phi)), phi, zo())).collect())
}

fn kernel_r10() -> Kernel {
    zoned([1e-3, 3e-3].iter()
        .map(|&tq| (format!("tq{}", g(tq)), 1.5, ZonedNoxOpts { tau_q: Some(tq), ..zo() }))
        .collect())
}

fn j_cases(js: &[f64], f: impl Fn(f64) -> ZonedNoxOpts) -> Vec<(String, f64, ZonedNoxOpts)> {
    js.iter().map(|&j| (format!("J{}", j as i64), 1.5, f(j))).collect()
}

fn kernel_r11() -> Kernel {
    zoned(j_cases(&[4.0, 16.0, 64.0], |j| ZonedNoxOpts { mixing: Some(mix(j)), ..zo() }))
}

fn kernel_r12() -> Kernel {
    zoned(j_cases(&[4.0, 16.0, 64.0], |j| ZonedNoxOpts {
        mixing: Some(mix(j)),
        unmixedness: Some(Unmixedness { s: 0.0625, ..Unmixedness::default() }),
        ..zo()
    }))
}

fn kernel_r13() -> Kernel {
    zoned(j_cases(&[4.0, 16.0], |j| ZonedNoxOpts {
        mixing: Some(mix(j)),
        pdf: Some(MixingPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..MixingPdf::default() }),
        ..zo()
    }))
}

fn kernel_r14() -> Kernel {
    let dp = Dp::new();
    let nf = dp.gas().nozzle_flow(dp.far(), dp.tt4(), dp.pt4(), dp.tt5(), dp.pt5(), flight().p0, None);
    let mut out = Kernel::default();
    for (k, v) in [
        ("T9_equilibrium", nf.t9_equilibrium), ("T9_frozen", nf.t9_frozen),
        ("V9_equilibrium", nf.v9_equilibrium), ("V9_frozen", nf.v9_frozen),
        ("co_fraction_entry", nf.co_fraction_entry), ("dV9", nf.dv9()),
        ("dV9_frac", nf.dv9_frac()), ("no_collapse_ratio", nf.no_collapse_ratio),
        ("x_no_e_entry", nf.x_no_e_entry), ("x_no_e_exit", nf.x_no_e_exit),
    ] {
        out.f(format!("nf.{k}"), v);
    }
    out.of("nf.max_a", nf.max_a);
    out.of("nf.x_no_frozen", nf.x_no_frozen);
    out
}

fn kernel_r15() -> Kernel {
    zoned(j_cases(&[4.0, 16.0], |j| ZonedNoxOpts {
        mixing: Some(mix(j)),
        pdf_quench: Some(QuenchPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..QuenchPdf::default() }),
        ..zo()
    }))
}

fn pq() -> PocketQuenchPdf {
    PocketQuenchPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..PocketQuenchPdf::default() }
}

fn kernel_r16() -> Kernel {
    zoned(vec![
        ("J4".into(), 1.5, ZonedNoxOpts { mixing: Some(mix(4.0)), pocket_quench: Some(pq()), ..zo() }),
        ("J16".into(), 1.5, ZonedNoxOpts { mixing: Some(mix(16.0)), pocket_quench: Some(pq()), ..zo() }),
        ("J16.seq".into(), 1.5, ZonedNoxOpts {
            mixing: Some(mix(16.0)), super_eq_o: true, pocket_quench: Some(pq()), ..zo()
        }),
    ])
}

fn kernel_r17() -> Kernel {
    let dp = Dp::new();
    let c = dp.gas().exhaust_no_clamp(
        dp.far(), dp.tt3(), dp.tt4(), dp.pt4(), dp.tt5(), dp.pt5(), flight().p0, 1.5, mix(16.0),
        pq(),
        ExhaustClampOpts { quench_ngrid: NG, quench_nsteps: NS, ..ExhaustClampOpts::default() },
    );
    let mut out = Kernel::default();
    for (k, v) in [
        ("T9", c.t9), ("a_bulk_quench", c.a_bulk_quench), ("a_mixed_out", c.a_mixed_out),
        ("a_pocket", c.a_pocket), ("ei_no_pocket_quench", c.ei_no_pocket_quench),
        ("ei_no_quenched", c.ei_no_quenched), ("gap_pocket_over_bulk", c.gap_pocket_over_bulk),
        ("max_a_quench", c.max_a_quench), ("no_collapse_ratio", c.no_collapse_ratio),
        ("phi_primary", c.phi_primary), ("x_no_bulk_quench", c.x_no_bulk_quench),
        ("x_no_e_exit", c.x_no_e_exit), ("x_no_mixed_out", c.x_no_mixed_out),
        ("x_no_pocket", c.x_no_pocket),
    ] {
        out.f(format!("clamp.{k}"), v);
    }
    out
}

fn kernel_r18() -> Kernel {
    zoned(j_cases(&[4.0, 16.0], |j| ZonedNoxOpts {
        mixing: Some(mix(j)),
        transported: Some(TransportedPdf { s: 0.0625, n_bell: NB, n_quad: NQ, ..TransportedPdf::default() }),
        ..zo()
    }))
}

fn kernel_r22() -> Kernel {
    zoned(j_cases(&[4.0, 16.0], |j| ZonedNoxOpts {
        mixing: Some(mix(j)),
        spatial: Some(SpatialPdf {
            s: 0.0625, n_bell: NB, n_quad: NQ, ny: NXY, nz: NXY, ..SpatialPdf::default()
        }),
        ..zo()
    }))
}

fn kernel_r23() -> Kernel {
    zoned(j_cases(&[4.0, 16.0], |j| ZonedNoxOpts {
        mixing: Some(mix(j)),
        spatial_dwell: Some(SpatialDwellPdf {
            s: 0.0625, nt: NT, n_bell: NB, n_quad: NQ, ny: NXY, nz: NXY,
            ..SpatialDwellPdf::default()
        }),
        ..zo()
    }))
}

fn kernel_r24() -> Kernel {
    zoned(j_cases(&[4.0, 16.0], |j| ZonedNoxOpts {
        mixing: Some(mix(j)),
        spatial_local: Some(SpatialLocalPdf {
            s: 0.0625, n_bell: NB, n_quad: NQ, ny: NXY, nz: NXY, ..SpatialLocalPdf::default()
        }),
        ..zo()
    }))
}

fn kernel_r25() -> Kernel {
    let dp = Dp::new();
    let mut out = Kernel::default();
    for da in [0.1, 1.0, 10.0] {
        let s = dp.gas().finite_rate_nozzle(
            dp.far(), dp.tt4(), dp.pt4(), dp.tt5(), dp.pt5(), flight().p0,
            FiniteRate { da, ..FiniteRate::default() },
        );
        let p = format!("Da{}", g(da));
        for (k, v) in [
            ("Da", s.da), ("T9_finite", s.t9_finite), ("T9_frozen", s.t9_frozen),
            ("T9_irrev_fast", s.t9_irrev_fast), ("T9_reversible", s.t9_reversible),
            ("T_star_entry", s.t_star_entry), ("V9_finite", s.v9_finite),
            ("V9_frozen", s.v9_frozen), ("V9_irrev_fast", s.v9_irrev_fast),
            ("V9_reversible", s.v9_reversible), ("attainable_gap", s.attainable_gap()),
            ("co_fraction_entry", s.co_fraction_entry),
            ("co_fraction_finite_exit", s.co_fraction_finite_exit), ("dS_finite", s.ds_finite),
            ("finite_filled", s.finite_filled()), ("unreachable_gap", s.unreachable_gap()),
        ] {
            out.f(format!("{p}.{k}"), v);
        }
    }
    out
}

fn kernel_r27() -> Kernel {
    let dp = Dp::new();
    let s = dp.gas().no_freeze_out_nozzle(
        dp.far(), dp.tt3(), dp.tt4(), dp.pt4(), dp.tt5(), dp.pt5(), flight().p0, 1.5,
        NoFreezeOut { l: 0.3, ..NoFreezeOut::default() },
    );
    let mut out = Kernel::default();
    for (k, v) in [
        ("Da_entry", s.da_entry), ("Da_exit", s.da_exit), ("T9_frozen", s.t9_frozen),
        ("max_a", s.max_a), ("max_a_frozen", s.max_a_frozen),
        ("relaxed_fraction", s.relaxed_fraction()), ("x_no_e_entry", s.x_no_e_entry),
        ("x_no_e_exit", s.x_no_e_exit), ("x_no_frozen", s.x_no_frozen),
        ("x_no_relaxed", s.x_no_relaxed),
    ] {
        out.f(format!("nfo.{k}"), v);
    }
    out
}

fn kernel_r28() -> Kernel {
    let dp = Dp::new();
    let mut out = Kernel::default();
    for couple in [true, false] {
        let s = dp.gas().coupled_no_freeze_out_nozzle(
            dp.far(), dp.tt3(), dp.tt4(), dp.pt4(), dp.tt5(), dp.pt5(), flight().p0, 1.5,
            CoupledNoFreezeOut { l: 0.3, ..CoupledNoFreezeOut::default() }, couple,
        );
        let p = if couple { "on" } else { "off" };
        for (k, v) in [
            ("Da_entry", s.da_entry), ("Da_exit_coupled", s.da_exit_coupled),
            ("Da_exit_depletion", s.da_exit_depletion), ("Da_exit_frozen", s.da_exit_frozen),
            ("Da_exit_heat", s.da_exit_heat), ("T9_frozen", s.t9_frozen), ("T9_pool", s.t9_pool),
            ("a_entry", s.a_entry), ("a_exit", s.a_exit), ("beta_max", s.beta_max),
            ("channel_ratio", s.channel_ratio()), ("depletion_factor", s.depletion_factor()),
            ("heat_release_factor", s.heat_release_factor()), ("max_a", s.max_a),
            ("max_a_frozen", s.max_a_frozen), ("net_factor", s.net_factor()),
            ("relaxed_fraction", s.relaxed_fraction()), ("s_freeze_pool", s.s_freeze_pool),
            ("tau_ratio_min", s.tau_ratio_min), ("x_no_e_exit", s.x_no_e_exit),
            ("x_no_frozen", s.x_no_frozen), ("x_no_relaxed", s.x_no_relaxed),
            ("x_radical_entry", s.x_radical_entry), ("x_radical_exit_pool", s.x_radical_exit_pool),
        ] {
            out.f(format!("{p}.{k}"), v);
        }
    }
    out
}

// ------------------------------------------------------------------------- r66

/// `ComponentMap(a, b, sigma=0.1, l).with_phi_surge(0.55)` — rungs 66-82's two maps.
fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(0.55)
}
fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(0.55)
}

/// The two-spool CPG design rungs 66-82 march — on `_cpg_gas()`, i.e. `R_c = 286.9` and NOT the
/// `(g-1)/g*cp` the rung suites' own `_cpg` uses. The fingerprint module chose the shared gas and
/// recorded that it moved no root (its § SLICE 7); this port follows the module, not the suites.
fn s3_design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg_gas(), 3.0, 6.0, TT4, flight().p0, TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
        eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
    })
}

fn kernel_r66() -> Kernel {
    let (floor, phi, b) = (0.55, 0.80, 0.10);
    let fl = flight();
    let core = match build_two_lag_cascade(
        s3_design(), fl, 1.0, Some(lp_map()), Some(hp_map()), 1.0,
        &LeverArm::floored(BleedLimiter::with_tau(phi, b, Some(0.05))),
    ) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("rung 66 builds a full core"),
    };
    let surge = SurgeLimiter::from_margin(&lp_map(), Spool::Lp, phi / floor - 1.0);
    let (traj, _) = core.stator_march_scoped(
        &fl,
        &Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.005 },
        None,
        &StatorLeg { accel: None, surge: Some(Floor::Phi(surge)), tt4_max: None },
        &MarchScope { lag: Some(AsymmetricLag::new(0.05, 0.15)), ..MarchScope::DEFAULT },
    );

    // `traj[i]` as Python's dict: the FLOAT entries only, which is what `keys` filters to.
    let row = |p: &FuelPoint| -> Vec<(&'static str, f64)> {
        let PointExtra::Cascade { g, required, b, b_cmd, ic_res, .. } = p.extra else {
            panic!("rung 66's march returns cascade points")
        };
        let mut v = vec![
            ("Tt4", p.tt4), ("b", b), ("b_cmd", b_cmd), ("f", p.f), ("g", g), ("ic_res", ic_res),
            ("mdot_air", p.mdot_air), ("mf", p.mf), ("mf_sched", p.mf_sched),
            ("nu_hp", p.nu_hp), ("nu_lp", p.nu_lp), ("phi_hp", p.phi_hp), ("phi_lp", p.phi_lp),
            ("pi_hpc", p.pi_hpc), ("pi_lpc", p.pi_lpc), ("required", required), ("s", p.s),
            ("sp_thrust", p.sp_thrust),
        ];
        v.sort_by(|a, b| a.0.cmp(b.0));
        v
    };
    let rows: Vec<Vec<(&str, f64)>> = traj.iter().map(row).collect();
    let col = |k: &str| -> Vec<f64> {
        rows.iter().map(|r| r.iter().find(|(n, _)| *n == k).unwrap().1).collect()
    };

    let mut out = Kernel::default();
    out.put("rows", V::I(traj.len() as i64));
    let keys: Vec<&str> = rows[0].iter().map(|(k, _)| *k).collect();
    out.put("keys", V::J(keys.iter().map(|k| JItem::S(k.to_string())).collect()));
    for (i, r) in rows.iter().enumerate() {
        for (k, v) in r {
            out.f(format!("t[{i}].{k}"), *v);
        }
    }
    // Python's `min(range(n), key=…)` / `max(…)`: the FIRST index attaining the extremum.
    for k in ["phi_lp", "phi_hp"] {
        let c = col(k);
        let j = (0..c.len()).fold(0, |j, i| if c[i] < c[j] { i } else { j });
        out.put(format!("argmin.{k}"), V::I(j as i64));
        out.f(format!("min.{k}"), c[j]);
    }
    for k in ["g", "b", "mf", "Tt4"] {
        let c = col(k);
        let edges = (1..c.len()).filter(|&i| (c[i] > 0.0) != (c[i - 1] > 0.0));
        out.put(format!("edges.{k}"), V::J(edges.map(|i| JItem::I(i as i64)).collect()));
        let j = (0..c.len()).fold(0, |j, i| if c[i] > c[j] { i } else { j });
        out.put(format!("argmax.{k}"), V::I(j as i64));
    }
    out
}

// ------------------------------------------------------------------------- slice 3 — rungs 67-77

const S3_LO: f64 = 1000.0;
const S3_HI: f64 = 1400.0;
const S3_TT4MAX: f64 = 1200.0;
const S3_PHI: f64 = 0.80;
const S3_B: f64 = 0.10;
const S3_VMAX: f64 = 0.20;
const S3_TAU: f64 = 0.05;
const S3_MARGIN: f64 = 0.10;
const EVERY_G: usize = 4;
const EVERY_M: usize = 4;
fn s3_sm() -> f64 { S3_PHI / 0.55 - 1.0 }
fn s3_ramp(ds: f64) -> Ramp { Ramp { tt4_lo: S3_LO, tt4_hi: S3_HI, r: 0.5, s_settle: 1.2, ds } }
const S3_CLOCKS: [(f64, f64, f64); 2] = [(0.05, 0.05, 0.05), (0.05, 0.005, 0.05)];

fn full_core(b: ScheduledStatorTransient) -> turbojet::stator_transient::ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("a slice-3 rig builds a full core"),
    }
}

/// `_window(P)`'s dict: a CLOSED window carries no `reciprocal` key at all.
fn window_tree(w: &Window) -> Tree {
    let mut v = vec![
        ("P", Tree::f(w.p)), ("k", Tree::f(w.k)), ("zeta", Tree::f(w.zeta)),
        ("T_over_tau", Tree::f(w.t_over_tau)), ("rho_lo", Tree::of(w.rho_lo)),
        ("rho_hi", Tree::of(w.rho_hi)), ("opens", Tree::b(w.opens)),
    ];
    if w.opens {
        v.push(("reciprocal", Tree::of(w.reciprocal)));
    }
    d(v)
}

fn kernel_r67() -> Kernel {
    let fl = flight();
    let m = full_core(build_cross_loop_cascade(
        s3_design(), fl, 1.0, Some(lp_map()), Some(hp_map()), 1.0,
        &LeverArm::floored(BleedLimiter::with_tau(S3_PHI, S3_B, Some(S3_TAU))),
    ));
    let x = m.cross_identity(&fl, &s3_ramp(0.005), S3_TT4MAX, S3_TAU, &[0.005, 0.05, 0.5], 4);
    let idt = d(vec![
        ("Tt4_max", Tree::f(x.tt4_max)), ("tau", Tree::f(x.tau)), ("tau_govs", Tree::fl(&x.tau_govs)),
        ("ds", Tree::f(x.ds)), ("r", Tree::f(x.r)), ("phi_lim", Tree::f(x.phi_lim)),
        ("b_max", Tree::f(x.b_max)),
        ("rows", Tree::L(x.rows.iter().map(|r| d(vec![
            ("tau_gov", Tree::f(r.tau_gov)), ("tau_v", Tree::f(r.tau_v)),
            ("rho_clock", Tree::f(r.rho_clock)), ("n_ride", Tree::u(r.n_ride)),
            ("n_sample", Tree::u(r.n_sample)), ("n_complex", Tree::u(r.n_complex)),
            ("n_saturated", Tree::u(r.n_saturated)), ("prod_lo", Tree::f(r.prod_lo)),
            ("prod_hi", Tree::f(r.prod_hi)), ("P_mid", Tree::f(r.p_mid)),
            ("R_q_lo", Tree::f(r.r_q_lo)), ("R_q_hi", Tree::f(r.r_q_hi)),
            ("C_g_lo", Tree::f(r.c_g_lo)), ("C_g_hi", Tree::f(r.c_g_hi)),
            ("gain_span_R", Tree::f(r.gain_span_r)), ("gain_span_C", Tree::f(r.gain_span_c)),
            ("rho_max", Tree::f(r.rho_max)), ("sum_bound", Tree::f(r.sum_bound)),
            ("sum_conservative", Tree::f(r.sum_conservative)), ("rho_lo", Tree::of(r.rho_lo)),
            ("rho_hi", Tree::of(r.rho_hi)), ("zeta", Tree::of(r.zeta)),
            ("T_over_tau", Tree::of(r.t_over_tau)), ("opens", Tree::ob(r.opens)),
            ("reciprocal", Tree::of(r.reciprocal)),
        ])).collect())),
        ("all_negative", Tree::b(x.all_negative)), ("prod_lo", Tree::f(x.prod_lo)),
        ("prod_hi", Tree::f(x.prod_hi)), ("R_q_min_abs", Tree::f(x.r_q_min_abs)),
        ("sum_always_safe", Tree::b(x.sum_always_safe)),
    ]);
    let o = m.oscillation_window(&fl, &s3_ramp(0.005), S3_TT4MAX, S3_TAU, &[0.5, 1.0, 2.0], 0.005);
    let osc = d(vec![
        ("Tt4_max", Tree::f(o.tt4_max)), ("tau", Tree::f(o.tau)), ("ds", Tree::f(o.ds)),
        ("r", Tree::f(o.r)), ("d_b0", Tree::f(o.d_b0)), ("P", Tree::f(o.p)),
        ("window", window_tree(&o.window)), ("rhos", Tree::fl(&o.rhos)),
        ("rows", Tree::L(o.rows.iter().map(|r| match r {
            OscRow::Skipped { rho, tau_gov } => d(vec![
                ("rho", Tree::f(*rho)), ("tau_gov", Tree::f(*tau_gov)), ("skipped", Tree::s("ds floor")),
            ]),
            OscRow::Live(l) => d(vec![
                ("rho", Tree::f(l.rho)), ("tau_gov", Tree::f(l.tau_gov)), ("npts", Tree::u(l.npts)),
                ("complex_predicted", Tree::b(l.complex_predicted)),
                ("sign_changes_q", Tree::u(l.sign_changes_q)),
                ("sign_changes_g", Tree::u(l.sign_changes_g)), ("rings", Tree::b(l.rings)),
                ("d0", Tree::f(l.d0)), ("d_end", Tree::f(l.d_end)),
                ("survives", Tree::f(l.survives)), ("d_peak", Tree::f(l.d_peak)),
            ]),
        }).collect())),
        ("n_complex", Tree::u(o.n_complex)), ("n_real", Tree::u(o.n_real)),
        ("max_sign_changes", Tree::u(o.max_sign_changes)),
        ("rings_anywhere", Tree::b(o.rings_anywhere)), ("survives_max", Tree::f(o.survives_max)),
    ]);
    let mut out = Kernel::default();
    flat(&idt, "idt", &mut out);
    flat(&osc, "osc", &mut out);
    out
}

/// `_triple_gains_at`'s dict. An OFF-REGIME point returns early with four keys
/// (`interior`, `off_regime`, `s`, `v_base`); an interior one returns the full set, without `s`.
fn gains_tree(g: &TripleGains) -> Tree {
    if !g.interior {
        return d(vec![
            ("interior", Tree::b(false)), ("off_regime", Tree::sl(&g.off_regime)),
            ("s", Tree::f(g.s)), ("v_base", Tree::f(g.v_base)),
        ]);
    }
    d(vec![
        ("interior", Tree::b(true)), ("off_regime", Tree::sl(&g.off_regime)),
        ("R_q", Tree::f(g.r_q)), ("R_v", Tree::f(g.r_v)), ("C_g", Tree::f(g.c_g)),
        ("C_v", Tree::f(g.c_v)), ("V_g", Tree::f(g.v_g)), ("V_q", Tree::f(g.v_q)),
        ("v_base", Tree::f(g.v_base)), ("cyclic", Tree::f(g.cyclic)),
        ("pair_RC", Tree::f(g.pair_rc)), ("pair_RV", Tree::f(g.pair_rv)),
        ("pair_CV", Tree::f(g.pair_cv)),
    ])
}

fn pair(p: Option<(f64, f64)>) -> Tree { p.map_or(Tree::n(), |(a, b)| Tree::fl(&[a, b])) }
fn taus3(t: (f64, f64, f64)) -> Tree { Tree::fl(&[t.0, t.1, t.2]) }
fn clocks_tree() -> Tree { Tree::L(S3_CLOCKS.iter().map(|&c| taus3(c)).collect()) }

fn plain_arm() -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::with_tau(S3_PHI, S3_B, Some(S3_TAU))),
        stator_lim: Some(StatorLimiter::new(S3_PHI, S3_VMAX, Some(S3_TAU))),
        ..Default::default()
    }
}

fn kernel_r68() -> Kernel {
    let fl = flight();
    let m = full_core(build_three_loop_cascade(
        s3_design(), fl, 1.0, Some(lp_map()), Some(hp_map()), 1.0, &plain_arm(),
    ));
    let rig = TripleRigArm { sm: s3_sm(), ..TripleRigArm::default() };
    let g = triple_gains(&m, &fl, &s3_ramp(0.005), s3_sm(), &rig, EVERY_G);
    let gt = d(vec![
        ("n_riding", Tree::u(g.n_riding)), ("n_sampled", Tree::u(g.n_sampled)),
        ("rows", Tree::L(g.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("on", gains_tree(&r.on)), ("live", gains_tree(&r.live)),
        ])).collect())),
        ("skipped", Tree::L(g.skipped.iter().map(|(s, on, live)| d(vec![
            ("s", Tree::f(*s)), ("on", Tree::sl(on)), ("live", Tree::sl(live)),
        ])).collect())),
        ("s_window", pair(g.s_window)), ("cyclic_on", Tree::fl(&g.cyclic_on)),
        ("cyclic_live", Tree::fl(&g.cyclic_live)), ("worst_on", Tree::of(g.worst_on)),
        ("worst_live", Tree::of(g.worst_live)),
    ]);
    let arms = triple_modes(&m, &fl, &s3_ramp(0.005), s3_sm(), &S3_CLOCKS, S3_VMAX, 3.0, EVERY_M);
    let mo = d(vec![
        ("clocks", clocks_tree()), ("ds", Tree::f(0.005)),
        ("arms", Tree::L(arms.iter().map(|a| d(vec![
            ("taus", taus3(a.taus)), ("rate_sum", Tree::f(a.rate_sum)), ("n", Tree::u(a.n)),
            ("rows", Tree::L(a.rows.iter().map(|x| d(vec![
                ("s", Tree::f(x.s)), ("c2", Tree::f(x.c2)), ("c1", Tree::f(x.c1)),
                ("c0", Tree::f(x.c0)), ("roots", Tree::fl(&x.roots)), ("cyclic", Tree::f(x.cyclic)),
                ("zeros", Tree::fl(&x.zeros)), ("dom", Tree::f(x.dom)),
            ])).collect())),
            ("n_sampled", Tree::u(a.n_sampled)), ("skipped", Tree::u(a.skipped)),
            ("dom_range", pair(a.dom_range)), ("worst_zero", Tree::of(a.worst_zero)),
        ])).collect())),
    ]);
    let mut out = Kernel::default();
    flat(&gt, "g", &mut out);
    flat(&mo, "mo", &mut out);
    out
}

fn opair(p: (Option<f64>, Option<f64>)) -> Tree { Tree::L(vec![Tree::of(p.0), Tree::of(p.1)]) }
fn ul(xs: &[usize]) -> Tree { Tree::L(xs.iter().map(|&x| Tree::u(x)).collect()) }
fn skipped_tree(sk: &[(f64, Vec<&'static str>, Vec<&'static str>)], names: (&str, &str)) -> Tree {
    Tree::L(sk.iter().map(|(s, a, b)| d(vec![
        ("s", Tree::f(*s)), (names.0, Tree::sl(a)), (names.1, Tree::sl(b)),
    ])).collect())
}

fn ref_arm_tree(a: &RefModesArm) -> Tree {
    d(vec![
        ("rate_sum", Tree::f(a.rate_sum)), ("n", Tree::u(a.n)), ("n_sampled", Tree::u(a.n_sampled)),
        ("skipped", Tree::u(a.skipped)),
        ("rows", Tree::L(a.rows.iter().map(|x| d(vec![
            ("s", Tree::f(x.s)), ("c1", Tree::f(x.c1)), ("c0", Tree::f(x.c0)), ("c2", Tree::f(x.c2)),
            ("k", Tree::f(x.k)), ("pair_RC", Tree::f(x.pair_rc)), ("cyclic", Tree::f(x.cyclic)),
            ("roots", Tree::L(x.roots.iter().map(|r| Tree::C(r.re, r.im)).collect())),
            ("zeta", Tree::of(x.zeta)), ("complex_pair", Tree::b(x.complex_pair)),
            ("n_zero", Tree::u(x.n_zero)), ("worst_zero", Tree::f(x.worst_zero)),
            ("c1_rel", Tree::f(x.c1_rel)), ("c0_rel", Tree::f(x.c0_rel)),
        ])).collect())),
        ("zeros", ul(&a.zeros)), ("max_c0_rel", Tree::of(a.max_c0_rel)),
        ("min_c1_rel", Tree::of(a.min_c1_rel)), ("all_complex", Tree::ob(a.all_complex)),
        ("zeta_range", opair(a.zeta_range)),
    ])
}

fn kernel_r69() -> Kernel {
    let fl = flight();
    let m = full_core(build_reference_split_cascade(
        s3_design(), fl, 1.0, Some(lp_map()), Some(hp_map()), 1.0, &plain_arm(),
    ));
    let rig = TripleRigArm { sm: s3_sm(), ..TripleRigArm::default() };
    let g = reference_gains(&m, &fl, &s3_ramp(0.005), s3_sm(), &rig, EVERY_G);
    let gt = d(vec![
        ("n_riding", Tree::u(g.n_riding)), ("n_sampled", Tree::u(g.n_sampled)),
        ("rows", Tree::L(g.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("inc", gains_tree(&r.inc)),
            ("phi", gains_tree(&r.phi)), ("own", gains_tree(&r.own)),
            ("k", Tree::f(r.k)), ("pair_gap", Tree::f(r.pair_gap)), ("v_base", Tree::f(r.v_base)),
        ])).collect())),
        ("skipped", skipped_tree(&g.skipped, ("inc", "phi"))),
        ("s_window", pair(g.s_window)), ("k_range", opair(g.k_range)),
        ("worst_RC_inc", Tree::of(g.worst_rc_inc)), ("worst_RC_phi", Tree::of(g.worst_rc_phi)),
        ("worst_pair_gap", Tree::of(g.worst_pair_gap)), ("worst_RC_own", Tree::of(g.worst_rc_own)),
    ]);
    let mo = reference_modes(&m, &fl, &s3_ramp(0.005), s3_sm(), &S3_CLOCKS, S3_VMAX, 3.0, EVERY_M);
    let mt = d(vec![
        ("clocks", Tree::L(mo.clocks.iter().map(|&c| taus3(c)).collect())), ("ds", Tree::f(mo.ds)),
        ("arms", Tree::L(mo.arms.iter().map(|a| d(vec![
            ("taus", taus3(a.taus)),
            ("refs", d(vec![("inc", ref_arm_tree(&a.inc)), ("phi", ref_arm_tree(&a.phi))])),
        ])).collect())),
    ]);
    let mut out = Kernel::default();
    flat(&gt, "g", &mut out);
    flat(&mt, "mo", &mut out);
    out
}

/// The rung-70+ rig: every floor `from_margin`, the stator arm either `phi` or incidence.
fn margin_arm(inc: bool) -> LeverArm {
    let sm = s3_sm();
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), S3_B, sm, Some(S3_TAU))),
        stator_inc: inc.then(|| StatorIncidenceLimiter::from_margin(&lp_map(), S3_VMAX, sm, Some(S3_TAU))),
        stator_lim: (!inc).then(|| StatorLimiter::from_margin(&lp_map(), S3_VMAX, sm, Some(S3_TAU))),
        ..Default::default()
    }
}

fn boundary_tree(b: &[StateBoundary]) -> Tree {
    Tree::L(b.iter().map(|x| d(vec![
        ("s", Tree::f(x.s)),
        ("live", d(vec![("R_q", Tree::f(x.live_r_q)), ("R_v", Tree::f(x.live_r_v))])),
        ("dead", d(vec![("R_q", Tree::f(x.dead_r_q)), ("R_v", Tree::f(x.dead_r_v))])),
    ])).collect())
}

fn skipped1(sk: &[(f64, Vec<&'static str>)]) -> Tree {
    Tree::L(sk.iter().map(|(s, o)| d(vec![("s", Tree::f(*s)), ("off_regime", Tree::sl(o))])).collect())
}

fn roots_tree(r: &[C64; 3]) -> Tree { Tree::L(r.iter().map(|x| Tree::C(x.re, x.im)).collect()) }

fn kernel_r70() -> Kernel {
    let fl = flight();
    let m = full_core(build_cross_split_cascade(
        s3_design(), fl, 1.0, Some(lp_map()), Some(hp_map()), 1.0, &margin_arm(false),
    ));
    let g = split_gains(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, s3_sm(), 0.5, 1.2, 0.005, S3_TAU, S3_TAU,
                        S3_TAU, S3_VMAX, EVERY_G);
    let gt = d(vec![
        ("n_riding", Tree::u(g.n_riding)), ("n_sampled", Tree::u(g.n_sampled)),
        ("rows", Tree::L(g.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("gov", gains_tree(&r.gov)), ("fuel", gains_tree(&r.fuel)),
            ("pair_gap", Tree::f(r.pair_gap)), ("cyclic_is_RC", Tree::f(r.cyclic_is_rc)),
        ])).collect())),
        ("skipped", skipped1(&g.skipped)), ("boundary", boundary_tree(&g.boundary)),
        ("s_window", pair(g.s_window)), ("worst_CV", Tree::of(g.worst_cv)),
        ("worst_RC_is_1", Tree::of(g.worst_rc_is_1)), ("worst_RV_is_1", Tree::of(g.worst_rv_is_1)),
        ("min_pair_gap", Tree::of(g.min_pair_gap)), ("max_pair_gap", Tree::of(g.max_pair_gap)),
        ("worst_cyclic_is_RC", Tree::of(g.worst_cyclic_is_rc)),
        ("worst_RC_fuel", Tree::of(g.worst_rc_fuel)), ("pair_RC", Tree::fl(&g.pair_rc)),
        ("pair_RV", Tree::fl(&g.pair_rv)), ("worse_pair", Tree::of(g.worse_pair)),
    ]);
    let mo = split_modes(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, s3_sm(), &S3_CLOCKS, 0.5, 1.2, 0.005,
                         S3_VMAX, EVERY_M);
    let mt = d(vec![
        ("clocks", Tree::L(mo.clocks.iter().map(|&c| taus3(c)).collect())), ("ds", Tree::f(mo.ds)),
        ("arms", Tree::L(mo.arms.iter().map(|a| d(vec![
            ("taus", taus3(a.taus)), ("rate_sum", Tree::f(a.rate_sum)), ("n", Tree::u(a.n)),
            ("n_sampled", Tree::u(a.n_sampled)), ("skipped", Tree::u(a.skipped)),
            ("rows", Tree::L(a.rows.iter().map(|x| d(vec![
                ("s", Tree::f(x.s)), ("c2", Tree::f(x.c2)), ("c1", Tree::f(x.c1)),
                ("c0", Tree::f(x.c0)), ("roots", roots_tree(&x.roots)),
                ("c1_pred", Tree::f(x.c1_pred)), ("c1_err", Tree::of(x.c1_err)),
                ("pair_RC", Tree::f(x.pair_rc)), ("pair_RV", Tree::f(x.pair_rv)),
                ("pair_CV", Tree::f(x.pair_cv)), ("cyclic", Tree::f(x.cyclic)),
                ("zeta", Tree::of(x.zeta)), ("complex_pair", Tree::b(x.complex_pair)),
                ("n_zero", Tree::u(x.n_zero)), ("worst_zero", Tree::f(x.worst_zero)),
                ("c1_rel", Tree::f(x.c1_rel)), ("c0_rel", Tree::f(x.c0_rel)),
            ])).collect())),
            ("zeros", ul(&a.zeros)), ("max_c0_rel", Tree::of(a.max_c0_rel)),
            ("min_c1_rel", Tree::of(a.min_c1_rel)), ("max_c1_err", Tree::of(a.max_c1_err)),
            ("any_complex", Tree::ob(a.any_complex)), ("zeta_range", opair(a.zeta_range)),
        ])).collect())),
    ]);
    let mut out = Kernel::default();
    flat(&gt, "g", &mut out);
    flat(&mt, "mo", &mut out);
    out
}

fn kernel_r71() -> Kernel {
    let fl = flight();
    let m = full_core(build_full_split_cascade(
        s3_design(), fl, 1.0, Some(lp_map()), Some(hp_map()), 1.0, &margin_arm(true),
    ));
    let g = full_gains(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, s3_sm(), 0.5, 1.2, 0.005, S3_TAU, S3_TAU,
                       S3_TAU, S3_VMAX, EVERY_G);
    let gt = d(vec![
        ("n_riding", Tree::u(g.n_riding)), ("n_sampled", Tree::u(g.n_sampled)),
        ("rows", Tree::L(g.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("gains", gains_tree(&r.gains)),
            ("phi_rig", gains_tree(&r.phi_rig)), ("x", Tree::f(r.x)), ("y", Tree::f(r.y)),
            ("det", Tree::f(r.det)), ("det_pred", Tree::f(r.det_pred)),
            ("y_is_RV", Tree::f(r.y_is_rv)), ("x_is_product", Tree::f(r.x_is_product)),
            ("det_err", Tree::f(r.det_err)), ("cross_rung", Tree::of(r.cross_rung)),
        ])).collect())),
        ("skipped", skipped1(&g.skipped)), ("boundary", boundary_tree(&g.boundary)),
        ("ds", Tree::f(g.ds)), ("s_window", pair(g.s_window)),
        ("closest_to_1", Tree::of(g.closest_to_1)), ("worst_y_is_RV", Tree::of(g.worst_y_is_rv)),
        ("worst_x_is_product", Tree::of(g.worst_x_is_product)),
        ("worst_det_err", Tree::of(g.worst_det_err)), ("det_scale", Tree::of(g.det_scale)),
        ("worst_cross_rung", Tree::of(g.worst_cross_rung)), ("pair_RC", Tree::fl(&g.pair_rc)),
        ("pair_RV", Tree::fl(&g.pair_rv)), ("pair_CV", Tree::fl(&g.pair_cv)),
    ]);
    // `ds = 0.002` HERE, not 0.005 — rung 71's own modes resolution (the Python kernel's note).
    let mo = full_modes(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, s3_sm(), &S3_CLOCKS, 0.5, 1.2, 0.002,
                        S3_VMAX, EVERY_M);
    let mt = d(vec![
        ("clocks", Tree::L(mo.clocks.iter().map(|&c| taus3(c)).collect())), ("ds", Tree::f(mo.ds)),
        ("arms", Tree::L(mo.arms.iter().map(|a| d(vec![
            ("taus", taus3(a.taus)), ("rate_sum", Tree::f(a.rate_sum)), ("n", Tree::u(a.n)),
            ("n_sampled", Tree::u(a.n_sampled)), ("skipped", Tree::u(a.skipped)),
            ("rows", Tree::L(a.rows.iter().map(|x| d(vec![
                ("s", Tree::f(x.s)), ("c2", Tree::f(x.c2)), ("c1", Tree::f(x.c1)),
                ("c0", Tree::f(x.c0)), ("roots", roots_tree(&x.roots)),
                ("c0_pred", Tree::f(x.c0_pred)), ("c0_err", Tree::of(x.c0_err)),
                ("u", Tree::f(x.u)), ("w", Tree::f(x.w)), ("z", Tree::f(x.z)),
                ("routh", Tree::f(x.routh)), ("pair_RC", Tree::f(x.pair_rc)),
                ("pair_RV", Tree::f(x.pair_rv)), ("pair_CV", Tree::f(x.pair_cv)),
                ("zeta", Tree::of(x.zeta)), ("r69_floor", Tree::of(x.r69_floor)),
                ("below_r69", Tree::b(x.below_r69)), ("complex_pair", Tree::b(x.complex_pair)),
                ("n_zero", Tree::u(x.n_zero)), ("min_root", Tree::f(x.min_root)),
                ("max_root", Tree::f(x.max_root)), ("stable", Tree::b(x.stable)),
                ("ds_lambda", Tree::f(x.ds_lambda)), ("mod_ratio", Tree::f(x.mod_ratio)),
            ])).collect())),
            ("zeros", ul(&a.zeros)), ("min_root_rel", Tree::of(a.min_root_rel)),
            ("max_c0_err", Tree::of(a.max_c0_err)), ("min_routh", Tree::of(a.min_routh)),
            ("all_stable", Tree::ob(a.all_stable)), ("any_complex", Tree::ob(a.any_complex)),
            ("any_below_r69", Tree::ob(a.any_below_r69)),
            ("max_mod_ratio", Tree::of(a.max_mod_ratio)), ("zeta_range", opair(a.zeta_range)),
        ])).collect())),
        ("zeros_everywhere", ul(&mo.zeros_everywhere)),
        ("arms_with_ring", Tree::u(mo.arms_with_ring)), ("arms_real", Tree::u(mo.arms_real)),
        ("arms_below_r69", Tree::u(mo.arms_below_r69)), ("max_c0_err", Tree::of(mo.max_c0_err)),
        ("min_routh", Tree::of(mo.min_routh)), ("max_mod_ratio", Tree::of(mo.max_mod_ratio)),
        ("all_stable", Tree::b(mo.all_stable)),
    ]);
    let mut out = Kernel::default();
    flat(&gt, "g", &mut out);
    flat(&mt, "mo", &mut out);
    out
}

fn auth(a: Option<Authority>) -> Tree { a.map_or(Tree::n(), |x| Tree::s(x.as_str())) }
fn taus4(t: (f64, f64, f64, f64)) -> Tree { Tree::fl(&[t.0, t.1, t.2, t.3]) }
const S3_TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);

/// `_quad_gains_at`'s dict. Rung 72's spells twelve gains; rung 73's re-definition adds the two
/// self-gains `F_f` / `R_r` and the three branch indicators — `v73` says which. The two early
/// returns (the switch guard, an off-regime point) carry five keys either way.
fn quad_tree(g: &QuadGains, v73: bool) -> Tree {
    if !g.interior {
        return d(vec![
            ("interior", Tree::b(false)), ("off_regime", Tree::sl(&g.off_regime)),
            ("s", Tree::f(g.s)), ("v_base", Tree::f(g.v_base)), ("near_switch", Tree::b(g.near_switch)),
        ]);
    }
    let mut v = vec![
        ("interior", Tree::b(true)), ("off_regime", Tree::sl(&g.off_regime)),
        ("near_switch", Tree::b(g.near_switch)), ("v_base", Tree::f(g.v_base)),
        ("authority", auth(g.authority)),
        ("F_r", Tree::f(g.f_r)), ("F_q", Tree::f(g.f_q)), ("F_v", Tree::f(g.f_v)),
        ("R_f", Tree::f(g.r_f)), ("R_q", Tree::f(g.r_q)), ("R_v", Tree::f(g.r_v)),
        ("C_f", Tree::f(g.c_f)), ("C_r", Tree::f(g.c_r)), ("C_v", Tree::f(g.c_v)),
        ("V_f", Tree::f(g.v_f)), ("V_r", Tree::f(g.v_r)), ("V_q", Tree::f(g.v_q)),
        ("pair_FR", Tree::f(g.pair_fr)), ("pair_RC", Tree::f(g.pair_rc)),
        ("pair_CV", Tree::f(g.pair_cv)), ("pair_RV", Tree::f(g.pair_rv)),
        ("masked", auth(g.masked)), ("mask_leak", Tree::of(g.mask_leak)),
    ];
    if v73 {
        v.extend([
            ("F_f", Tree::f(g.f_f)), ("R_r", Tree::f(g.r_r)),
            ("self_masked", Tree::of(g.self_masked)), ("cross_masked", Tree::of(g.cross_masked)),
            ("self_live", Tree::of(g.self_live)),
        ]);
    }
    d(v)
}

fn quad_boundary(b: &[BoundaryCheck]) -> Tree {
    Tree::L(b.iter().map(|x| d(vec![
        ("s", Tree::f(x.s)),
        ("live", d(vec![("F_q", Tree::f(x.live_f_q)), ("F_v", Tree::f(x.live_f_v)),
                        ("R_q", Tree::f(x.live_r_q)), ("R_v", Tree::f(x.live_r_v))])),
        ("dead", d(vec![("F_q", Tree::f(x.dead_f_q)), ("F_v", Tree::f(x.dead_f_v)),
                        ("R_q", Tree::f(x.dead_r_q)), ("R_v", Tree::f(x.dead_r_v))])),
    ])).collect())
}

fn kernel_r72() -> Kernel {
    let fl = flight();
    let cp = d(charpoly_selftest().iter().map(|(name, a)| {
        let mut v = vec![
            ("trace_err", Tree::f(a.trace_err)), ("det_err", Tree::f(a.det_err)),
            ("det_vs_a0", Tree::f(a.det_vs_a0)), ("resid", Tree::f(a.resid)),
        ];
        if let Some(x) = a.diag_err { v.push(("diag_err", Tree::f(x))); }
        if let Some(x) = a.max_imag { v.push(("max_imag", Tree::f(x))); }
        (*name, d(v))
    }).collect());
    let m = full_core(build_shared_actuator_cascade(
        s3_design(), fl, 1.0, Some(lp_map()), Some(hp_map()), 1.0, &margin_arm(false),
    ));
    let g = shared_gains(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, s3_sm(), S3_TAUS, false, 0.5, 1.2,
                         0.005, S3_VMAX, EVERY_G).expect("rung 72's gains do not abort");
    let gt = d(vec![
        ("inc", Tree::b(g.inc)), ("taus", taus4(g.taus)), ("ds", Tree::f(g.ds)),
        ("rows", Tree::L(g.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("gains", quad_tree(&r.gains, false)),
            ("authority", auth(r.authority)), ("masked", auth(r.masked)),
            ("mask_leak", Tree::of(r.mask_leak)), ("det", Tree::f(r.det)), ("taus", taus4(r.taus)),
        ])).collect())),
        ("skipped", d(vec![("switch", Tree::u(g.skipped_switch)), ("regime", Tree::u(g.skipped_regime))])),
        ("n_riding", Tree::u(g.n_riding)), ("n_sampled", Tree::u(g.n_sampled)),
        ("boundary", quad_boundary(&g.boundary)), ("s_window", pair(g.s_window)),
        ("by_authority", d(vec![("fuel", Tree::u(g.by_authority_fuel)), ("gov", Tree::u(g.by_authority_gov))])),
        ("worst_F_r", Tree::of(g.worst_f_r)), ("worst_R_f", Tree::of(g.worst_r_f)),
        ("worst_pair_FR", Tree::of(g.worst_pair_fr)), ("worst_mask_leak", Tree::of(g.worst_mask_leak)),
        ("min_live_gain", Tree::of(g.min_live_gain)), ("det_range", pair(g.det_range)),
    ]);
    let mut out = Kernel::default();
    flat(&cp, "cp", &mut out);
    flat(&gt, "g", &mut out);
    out
}

fn kernel_r73() -> Kernel {
    let fl = flight();
    let m = full_core(build_applied_reference_cascade(
        s3_design(), fl, 1.0, Some(lp_map()), Some(hp_map()), 1.0, &margin_arm(false),
    ));
    let g = applied_gains(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, s3_sm(), S3_TAUS, false, 0.5, 1.2,
                          0.005, S3_VMAX, EVERY_G).expect("rung 73's gains do not abort");
    let gt = d(vec![
        ("inc", Tree::b(g.inc)), ("taus", taus4(g.taus)), ("ds", Tree::f(g.ds)),
        ("rows", Tree::L(g.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("authority", auth(r.authority)), ("masked", auth(r.masked)),
            ("gains", quad_tree(&r.gains, true)), ("taus", taus4(r.taus)),
            ("self_masked", Tree::of(r.self_masked)), ("cross_masked", Tree::of(r.cross_masked)),
            ("self_live", Tree::of(r.self_live)), ("mask_leak", Tree::of(r.mask_leak)),
            ("delta_moved", Tree::fl(&[r.delta_moved.0, r.delta_moved.1])),
            ("delta_rest", Tree::f(r.delta_rest)), ("det", Tree::f(r.det)),
        ])).collect())),
        ("skipped", d(vec![("switch", Tree::u(g.skipped_switch)), ("regime", Tree::u(g.skipped_regime))])),
        ("boundary", quad_boundary(&g.boundary)),
        ("n_riding", Tree::u(g.n_riding)), ("n_sampled", Tree::u(g.n_sampled)),
        ("by_authority", d(vec![("fuel", Tree::u(g.by_authority_fuel)), ("gov", Tree::u(g.by_authority_gov))])),
        ("self_masked", Tree::fl(&g.self_masked)), ("cross_masked", Tree::fl(&g.cross_masked)),
        ("self_live", Tree::fl(&g.self_live)), ("worst_mask_leak", Tree::of(g.worst_mask_leak)),
        ("worst_delta_rest", Tree::of(g.worst_delta_rest)), ("moved_scaled", Tree::fl(&g.moved_scaled)),
        ("min_live_gain", Tree::of(g.min_live_gain)), ("det_range", pair(g.det_range)),
    ]);
    let mut out = Kernel::default();
    flat(&gt, "g", &mut out);
    out
}

/// A rung-74+ rig with Python's `_s3_rig(cls, **attrs)` attributes set AFTER construction.
fn demand_rig(build: fn(TwoSpoolEngine, FlightCondition, f64, Option<ComponentMap>,
                        Option<ComponentMap>, f64, &LeverArm) -> ScheduledStatorTransient)
    -> turbojet::stator_transient::ScheduledStatorCore {
    let m = full_core(build(s3_design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                            &margin_arm(false)));
    m.fuel.inner.lag_coord.set("demand");
    m.fuel.inner.ref_law.set("sched");
    m
}

fn kernel_r74() -> Kernel {
    let fl = flight();
    let m = demand_rig(build_demand_coordinate_cascade);
    let g = demand_gains(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_TAUS, false, 0.5, 1.2,
                         0.005, S3_VMAX, EVERY_G);
    let gt = d(vec![
        ("inc", Tree::b(g.inc)), ("phi_lim", Tree::f(g.phi_lim)), ("taus", taus4(g.taus)),
        ("ds", Tree::f(g.ds)), ("n", Tree::u(g.n)),
        ("rows", Tree::L(g.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("authority", Tree::s(r.authority.as_str())),
            ("poly_gap", Tree::f(r.poly_gap)), ("poly_scale", Tree::f(r.poly_scale)),
            ("worst_flip", Tree::f(r.worst_flip)), ("worst_keep", Tree::f(r.worst_keep)),
            ("n_sign_changed", Tree::u(r.n_sign_changed)), ("biggest_moved", Tree::f(r.biggest_moved)),
            ("mask_leak_w", Tree::of(r.mask_leak_w)), ("mask_leak_g", Tree::of(r.mask_leak_g)),
            ("pairs_gap", Tree::f(r.pairs_gap)),
        ])).collect())),
        ("skipped", d(vec![("regime", Tree::u(g.skipped.0)), ("switch", Tree::u(g.skipped.1))])),
        ("worst_poly_gap", Tree::of(g.worst_poly_gap)), ("worst_poly_rel", Tree::of(g.worst_poly_rel)),
        ("worst_flip", Tree::of(g.worst_flip)), ("worst_keep", Tree::of(g.worst_keep)),
        ("worst_pairs_gap", Tree::of(g.worst_pairs_gap)),
        ("worst_mask_leak", Tree::of(g.worst_mask_leak)),
        ("min_sign_changed", g.min_sign_changed.map_or(Tree::n(), Tree::u)),
        ("biggest_moved", Tree::of(g.biggest_moved)),
    ]);
    let mut out = Kernel::default();
    flat(&gt, "g", &mut out);
    out
}

fn fpair(p: (f64, f64)) -> Tree { Tree::fl(&[p.0, p.1]) }

fn kernel_r75() -> Kernel {
    let fl = flight();
    let m = demand_rig(build_anti_windup_cascade);
    m.fuel.inner.windup_law.set("track");
    m.fuel.inner.tau_t.set(Some(0.05));
    let g = windup_gains(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_TAUS, &[0.05, 0.0125],
                         &["applied", "sched"], false, 0.5, 1.2, 0.005, S3_VMAX, EVERY_G);
    // Python keys the cells by the STRING `f"{ref}|{tau_t}"`, so `tau_t` is spelt by `repr`.
    let cells: Vec<(String, Tree)> = g.cells.iter().map(|((r, t), c)| {
        let body = match c {
            WindupCell::Empty { n_riding } => d(vec![("n", Tree::u(0)), ("n_riding", Tree::u(*n_riding))]),
            WindupCell::Read(x) => d(vec![
                ("n", Tree::u(x.n)), ("n_riding", Tree::u(x.n_riding)), ("tau_t", Tree::f(x.tau_t)),
                ("ref", Tree::s(x.ref_law)), ("masked_diag", fpair(x.masked_diag)),
                ("masked_diag0", fpair(x.masked_diag0)), ("diag_err", Tree::f(x.diag_err)),
                ("auth_diag_moved", Tree::f(x.auth_diag_moved)), ("track_leak", Tree::f(x.track_leak)),
                ("mask_leak", Tree::f(x.mask_leak)), ("mask_leak0", Tree::f(x.mask_leak0)),
                ("det", fpair(x.det)), ("det0", fpair(x.det0)), ("det_alive", Tree::f(x.det_alive)),
                ("det0_alive", Tree::f(x.det0_alive)), ("zeros", ul(&x.zeros)),
                ("zeros0", ul(&x.zeros0)), ("row_err", Tree::f(x.row_err)),
                ("row_auth0", fpair(x.row_auth0)),
                ("rows", Tree::L(x.rows.iter().map(|w| d(vec![
                    ("s", Tree::f(w.s)), ("auth", Tree::s(w.auth.as_str())),
                    ("masked", Tree::s(w.masked.as_str())), ("tau_masked", Tree::f(w.tau_masked)),
                    ("masked_diag", Tree::f(w.masked_diag)), ("masked_diag0", Tree::f(w.masked_diag0)),
                    ("auth_diag", Tree::f(w.auth_diag)), ("auth_diag0", Tree::f(w.auth_diag0)),
                    ("row_auth", Tree::f(w.row_auth)), ("row_auth0", Tree::f(w.row_auth0)),
                    ("mask_leak", Tree::f(w.mask_leak)), ("mask_leak0", Tree::f(w.mask_leak0)),
                    ("track_leak", Tree::f(w.track_leak)), ("det", Tree::f(w.det)),
                    ("det0", Tree::f(w.det0)), ("zeros", Tree::u(w.zeros)), ("zeros0", Tree::u(w.zeros0)),
                ])).collect())),
            ]),
        };
        (format!("{r}|{}", repr_f64(*t)), body)
    }).collect();
    let gt = d(vec![
        ("phi_lim", Tree::f(g.phi_lim)), ("taus", taus4(g.taus)), ("tau_ts", Tree::fl(&g.tau_ts)),
        ("inc", Tree::b(g.inc)), ("ds", Tree::f(g.ds)), ("cells", Tree::D(cells)),
        ("ratios", d(g.ratios.iter().map(|x| (x.ref_law, d(vec![
            ("n", Tree::u(x.n)), ("diag", fpair(x.diag)), ("det", fpair(x.det)),
        ]))).collect())),
    ]);
    let mut out = Kernel::default();
    flat(&gt, "g", &mut out);
    out
}

/// Python's `_UNSTABLE["r76"]`: rung 76's `applied|none|fuel` masked determinant is the DEAD one,
/// so `det_err` / `det_ratio` are `0/0` and the model does not determine them. Dropped after
/// flattening, exactly as `_s3` drops them.
const UNSTABLE_R76: [&str; 3] = [
    "g.cells.applied|none|fuel.det_err",
    "g.cells.applied|none|fuel.det_ratio[0]",
    "g.cells.applied|none|fuel.det_ratio[1]",
];

fn upair(p: (usize, usize)) -> Tree { ul(&[p.0, p.1]) }

fn kernel_r76() -> Kernel {
    let fl = flight();
    let m = demand_rig(build_sensed_cap_cascade);
    m.fuel.inner.windup_law.set("none");
    m.fuel.inner.tau_t.set(None);
    m.fuel.inner.cap_law.set("sensed");
    let g = cap_gains(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS, 0.05,
                      &["sched", "applied"], &["none", "track"], false, 0.5, 1.2, 0.005, S3_VMAX,
                      EVERY_G);
    let cells: Vec<(String, Tree)> = g.cells.iter().map(|(k, c)| {
        let body = match c {
            CapCell::Empty { n_riding, ref_law, law, auth, n_inert } => d(vec![
                ("n", Tree::u(0)), ("n_riding", Tree::u(*n_riding)), ("ref", Tree::s(ref_law)),
                ("law", Tree::s(law)), ("auth", Tree::s(auth.as_str())), ("n_inert", Tree::u(*n_inert)),
            ]),
            CapCell::Read(x) => d(vec![
                ("n", Tree::u(x.n)), ("n_riding", Tree::u(x.n_riding)), ("ref", Tree::s(x.ref_law)),
                ("law", Tree::s(x.law)), ("auth", Tree::s(x.auth.as_str())),
                ("n_inert", Tree::u(x.n_inert)), ("c", fpair(x.c)), ("auth_diag", fpair(x.auth_diag)),
                ("auth_diag0", fpair(x.auth_diag0)), ("auth_moved", Tree::f(x.auth_moved)),
                ("auth_err", Tree::f(x.auth_err)), ("masked_diag", fpair(x.masked_diag)),
                ("masked_moved", Tree::f(x.masked_moved)), ("row_auth", fpair(x.row_auth)),
                ("row_auth0", fpair(x.row_auth0)), ("row_err", Tree::of(x.row_err)),
                ("mask_leak", Tree::f(x.mask_leak)), ("mask_leak0", Tree::f(x.mask_leak0)),
                ("det", fpair(x.det)), ("det0", fpair(x.det0)),
                ("det_ratio", x.det_ratio.map_or(Tree::n(), fpair)), ("det_err", Tree::of(x.det_err)),
                ("zeros", upair(x.zeros)), ("zeros0", upair(x.zeros0)),
                ("zeros_moved", Tree::u(x.zeros_moved)), ("gov_row", Tree::f(x.gov_row)),
            ]),
        };
        (k.clone(), body)
    }).collect();
    let gt = d(vec![
        ("phi_lim", Tree::f(g.phi_lim)), ("margin", Tree::f(g.margin)), ("taus", taus4(g.taus)),
        ("tau_t", Tree::f(g.tau_t)), ("inc", Tree::b(g.inc)), ("ds", Tree::f(g.ds)),
        ("cells", Tree::D(cells)),
    ]);
    let mut out = Kernel::default();
    flat(&gt, "g", &mut out);
    for k in UNSTABLE_R76 {
        out.0.remove(k);
    }
    out
}

/// A rung-77+ rig: `_lag_coord="demand"`, `_ref_law="sched"`, `_windup_law="none"`,
/// `_tau_t=None`, `_cap_law="solve"` — every reader-only rung's own attribute set.
fn solve_rig(build: fn(TwoSpoolEngine, FlightCondition, f64, Option<ComponentMap>,
                       Option<ComponentMap>, f64, &LeverArm) -> ScheduledStatorTransient)
    -> turbojet::stator_transient::ScheduledStatorCore {
    let m = demand_rig(build);
    m.fuel.inner.windup_law.set("none");
    m.fuel.inner.tau_t.set(None);
    m.fuel.inner.cap_law.set("solve");
    m
}

/// `{leg: (lo, hi)}` over the three legs, in Python's `("accel", "gov", "phi")` order.
fn by_leg(x: Option<[(f64, f64); 3]>) -> Tree {
    x.map_or(Tree::n(), |a| d(Leg::ORDER.iter().zip(a).map(|(l, p)| (l.name(), fpair(p))).collect()))
}
fn legs_list(xs: &[Leg]) -> Tree { Tree::L(xs.iter().map(|l| Tree::s(l.name())).collect()) }

fn kernel_r77() -> Kernel {
    let fl = flight();
    let m = solve_rig(build_stiffness_ledger_cascade);
    let sl = leg_slopes(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS, false, 0.5,
                        1.2, 0.005, S3_VMAX, 8);
    let slt = d(vec![
        ("phi_lim", Tree::f(sl.phi_lim)), ("margin", Tree::f(sl.margin)), ("inc", Tree::b(sl.inc)),
        ("n", Tree::u(sl.n)),
        ("rows", Tree::L(sl.rows.iter().map(|r| {
            let mut v: Vec<(&str, Tree)> = Leg::ORDER.iter().zip(&r.legs).map(|(l, x)| (l.name(), d(vec![
                ("w", Tree::f(x.w)), ("Gw", Tree::f(x.gw)), ("norm", Tree::f(x.norm)),
                ("stiff", Tree::f(x.stiff)),
            ]))).collect();
            v.extend([("s", Tree::f(r.s)), ("c", Tree::f(r.c)), ("c_err", Tree::f(r.c_err))]);
            d(v)
        }).collect())),
        ("c_err", Tree::of(sl.c_err)), ("c", pair(sl.c)), ("Gw", by_leg(sl.gw)),
        ("norm", by_leg(sl.norm)), ("stiff", by_leg(sl.stiff)), ("sep", Tree::of(sl.sep)),
    ]);
    let sg = set_point_gains(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS, false,
                             0.5, 1.2, 0.005, S3_VMAX, 1e-5, 8);
    let sgt = d(vec![
        ("phi_lim", Tree::f(sg.phi_lim)), ("margin", Tree::f(sg.margin)), ("inc", Tree::b(sg.inc)),
        ("n", Tree::u(sg.n)),
        ("rows", Tree::L(sg.rows.iter().map(|r| {
            let mut v: Vec<(&str, Tree)> = Leg::ORDER.iter().zip(&r.legs).map(|(l, x)| (l.name(), d(vec![
                ("w", Tree::f(x.w)), ("direct", Tree::f(x.direct)), ("ift", Tree::f(x.ift)),
                ("Gq", Tree::f(x.gq)), ("Gw", Tree::f(x.gw)), ("live", Tree::b(x.live)),
                ("err", Tree::f(x.err)),
            ]))).collect();
            v.extend([("s", Tree::f(r.s)), ("live", legs_list(&r.live))]);
            d(v)
        }).collect())),
        ("ift_err", Tree::of(sg.ift_err)), ("gain", by_leg(sg.gain)),
        ("order", sg.order.map_or(Tree::n(), |o| legs_list(&o))),
        ("order_stable", Tree::ob(sg.order_stable)), ("n_guarded", Tree::u(sg.n_guarded)),
        ("guarded", sg.guarded.as_deref().map_or(Tree::n(), legs_list)),
        ("guarded_stable", Tree::ob(sg.guarded_stable)),
        ("guarded_orders", Tree::L(sg.guarded_orders.iter().map(|o| legs_list(o)).collect())),
        ("phi_top", Tree::b(sg.phi_top)),
    ]);
    let sq = singular_limit(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS, false,
                            0.5, 1.2, 0.005, S3_VMAX, 0.10, 8);
    let sqt = d(vec![
        ("phi_lim", Tree::f(sq.phi_lim)), ("margin", Tree::f(sq.margin)), ("inc", Tree::b(sq.inc)),
        ("spread", Tree::f(sq.spread)), ("n", Tree::u(sq.n)),
        ("rows", Tree::L(sq.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("w_phi", Tree::f(r.w_phi)), ("w_gov", Tree::f(r.w_gov)),
            ("phi_open", Tree::f(r.phi_open)), ("phi_closed", Tree::f(r.phi_closed)),
            ("gov_open", Tree::f(r.gov_open)), ("gov_closed", Tree::f(r.gov_closed)),
            ("gov_rel", Tree::f(r.gov_rel)), ("phi_spread", Tree::f(r.phi_spread)),
            ("phi_off", Tree::f(r.phi_off)),
        ])).collect())),
        ("phi_open", Tree::of(sq.phi_open)), ("phi_closed", Tree::of(sq.phi_closed)),
        ("phi_off", Tree::of(sq.phi_off)), ("phi_spread", Tree::of(sq.phi_spread)),
        ("gov_rel", Tree::of(sq.gov_rel)), ("gov_open", Tree::of(sq.gov_open)),
    ]);
    let mut out = Kernel::default();
    flat(&slt, "sl", &mut out);
    flat(&sgt, "sg", &mut out);
    flat(&sqt, "sq", &mut out);
    out
}

// ------------------------------------------------------------------------- slice 4 — rungs 78/79

/// A float-keyed dict: Python's `_flat` spells the key `f"{k}"`, i.e. `repr`.
fn by_float(items: Vec<(f64, Tree)>) -> Tree {
    Tree::D(items.into_iter().map(|(k, t)| (repr_f64(k), t)).collect())
}

fn r78_rig() -> turbojet::stator_transient::ScheduledStatorCore {
    let m = solve_rig(build_residual_gauge_cascade);
    m.fuel.inner.gauge_k.set(1.0);
    m
}

fn kernel_r78() -> Kernel {
    let fl = flight();
    let g = gauge_scan(&r78_rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS, false,
                       0.5, 1.2, 0.005, S3_VMAX, 1e-5, 8, &GAUGE_SCAN_MULTS);
    let gt = d(vec![
        ("phi_lim", Tree::f(g.phi_lim)), ("margin", Tree::f(g.margin)), ("inc", Tree::b(g.inc)),
        ("n", Tree::u(g.n)),
        ("rows", Tree::L(g.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("c", Tree::f(r.c)), ("k_crit", Tree::f(r.k_crit)),
            ("w1", Tree::f(r.w1)), ("Gw1", Tree::f(r.gw1)), ("c_err", Tree::f(r.c_err)),
            ("ks", by_float(r.ks.iter().map(|x| (x.mult, d(vec![
                ("mult", Tree::f(x.mult)), ("k", Tree::f(x.k)), ("w", Tree::f(x.w)),
                ("ok", Tree::b(x.ok)), ("Gw", Tree::f(x.gw)), ("Gw_pred", Tree::f(x.gw_pred)),
                ("anchor", Tree::f(x.anchor)), ("n_roots", Tree::u(x.n_roots)),
                ("w_move", Tree::f(x.w_move)), ("Gw_err", Tree::f(x.gw_err)),
                ("direct", Tree::f(x.direct)), ("excluded", Tree::b(x.excluded)),
                ("gain_move", Tree::f(x.gain_move)),
            ]))).collect())),
            ("base", Tree::f(r.base)),
        ])).collect())),
        ("n_excluded", Tree::u(g.n_excluded)), ("n_kept", Tree::u(g.n_kept)),
        ("excluded_mults", Tree::fl(&g.excluded_mults)), ("excluded_worst", Tree::of(g.excluded_worst)),
        ("c_err", Tree::of(g.c_err)), ("w_move", Tree::of(g.w_move)), ("Gw_err", Tree::of(g.gw_err)),
        ("Gw_span", pair(g.gw_span)), ("sign_change", Tree::ob(g.sign_change)),
        ("gain_move", Tree::of(g.gain_move)), ("n_bad", Tree::u(g.n_bad)), ("c", pair(g.c)),
    ]);
    let c = root_census(&r78_rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS,
                        false, 0.5, 1.2, 0.005, S3_VMAX, 8, &ROOT_CENSUS_MULTS, 0.2, 3.0, 400);
    let ct = d(vec![
        ("phi_lim", Tree::f(c.phi_lim)), ("margin", Tree::f(c.margin)), ("inc", Tree::b(c.inc)),
        ("n", Tree::u(c.n)),
        ("rows", Tree::L(c.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("w0", Tree::f(r.w0)), ("c", Tree::f(r.c)),
            ("k_crit", Tree::f(r.k_crit)),
            ("cells", by_float(r.cells.iter().map(|x| (x.mult, d(vec![
                ("mult", Tree::f(x.mult)), ("k", Tree::f(x.k)), ("G_at_w0", Tree::f(x.g_at_w0)),
                ("n_roots", Tree::u(x.n_roots)), ("roots", Tree::fl(&x.roots)),
                ("spurious", Tree::fl(&x.spurious)),
            ]))).collect())),
        ])).collect())),
        ("G_at_w0", Tree::of(c.g_at_w0)), ("true_found", Tree::b(c.true_found)),
        ("n_roots", ul(&c.n_roots)), ("multi_mults", Tree::fl(&c.multi_mults)),
        ("band", pair(c.band)), ("brackets", Tree::ob(c.brackets)), ("approach", Tree::of(c.approach)),
    ]);
    let v = gauge_vs_device(&r78_rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS,
                            false, 0.5, 1.2, 0.005, S3_VMAX, 8, 1e-5, 0.10);
    let vt = d(vec![
        ("phi_lim", Tree::f(v.phi_lim)), ("margin", Tree::f(v.margin)), ("inc", Tree::b(v.inc)),
        ("n", Tree::u(v.n)),
        ("rows", Tree::L(v.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("w_solve", Tree::f(r.w_solve)), ("w_sensed", Tree::f(r.w_sensed)),
            ("device", Tree::f(r.device)), ("w_phi", Tree::f(r.w_phi)),
            ("phi_open_w", Tree::f(r.phi_open_w)), ("phi_open_q", Tree::f(r.phi_open_q)),
            ("phi_closed_w", Tree::f(r.phi_closed_w)), ("kill_w", Tree::f(r.kill_w)),
            ("phi_spread", Tree::f(r.phi_spread)),
        ])).collect())),
        ("device", Tree::of(v.device)), ("device_max", Tree::of(v.device_max)),
        ("kill_w", Tree::of(v.kill_w)), ("phi_open_w", Tree::of(v.phi_open_w)),
        ("phi_closed_w", Tree::of(v.phi_closed_w)), ("phi_open_q", Tree::of(v.phi_open_q)),
        ("phi_spread", Tree::of(v.phi_spread)),
    ]);
    let mut out = Kernel::default();
    flat(&gt, "g", &mut out);
    flat(&ct, "c", &mut out);
    flat(&vt, "d", &mut out);
    out
}

fn kernel_r79() -> Kernel {
    let fl = flight();
    let rig = || solve_rig(build_state_coordinate_cascade);
    let f = coord_forced(&rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS, false,
                         0.5, 1.2, 0.005, S3_VMAX, 8);
    let ft = d(vec![
        ("phi_lim", Tree::f(f.phi_lim)), ("margin", Tree::f(f.margin)), ("inc", Tree::b(f.inc)),
        ("n", Tree::u(f.n)),
        ("rows", Tree::L(f.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("binding", Tree::b(r.binding)), ("w_phi", Tree::f(r.w_phi)),
            ("w_inc", Tree::f(r.w_inc)), ("w_shipped", Tree::f(r.w_shipped)),
            ("d_forced", Tree::f(r.d_forced)), ("d_shipped", Tree::f(r.d_shipped)),
            ("same_float", Tree::b(r.same_float)),
        ])).collect())),
        ("n_binding", Tree::u(f.n_binding)), ("d_forced", Tree::of(f.d_forced)),
        ("d_forced_med", Tree::of(f.d_forced_med)), ("n_same_float", Tree::u(f.n_same_float)),
        ("d_shipped", Tree::of(f.d_shipped)),
    ]);
    let s = coord_scan(&rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS, false,
                       0.5, 1.2, 0.005, S3_VMAX, 1e-5, 8);
    let st = d(vec![
        ("phi_lim", Tree::f(s.phi_lim)), ("margin", Tree::f(s.margin)), ("inc", Tree::b(s.inc)),
        ("n", Tree::u(s.n)),
        ("rows", Tree::L(s.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("w_phi", Tree::f(r.w_phi)), ("w_inc", Tree::f(r.w_inc)),
            ("d_set", Tree::f(r.d_set)), ("same_float", Tree::b(r.same_float)),
            ("slope_phi", Tree::f(r.slope_phi)), ("slope_inc", Tree::f(r.slope_inc)),
            ("ratio", Tree::f(r.ratio)), ("dwdq_phi", Tree::f(r.dwdq_phi)),
            ("dwdq_inc", Tree::f(r.dwdq_inc)),
        ])).collect())),
        ("predicted_ratio", Tree::f(s.predicted_ratio)), ("ratio_err", Tree::of(s.ratio_err)),
        ("dwdq_err", Tree::of(s.dwdq_err)), ("d_set", Tree::of(s.d_set)),
        ("d_set_min", Tree::of(s.d_set_min)), ("n_same_float", Tree::u(s.n_same_float)),
    ]);
    let c = coord_census(&rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS, false,
                         0.5, 1.2, 0.005, S3_VMAX, 8, 0.2, 3.0, 400);
    let ct = d(vec![
        ("phi_lim", Tree::f(c.phi_lim)), ("margin", Tree::f(c.margin)), ("inc", Tree::b(c.inc)),
        ("n", Tree::u(c.n)),
        ("rows", Tree::L(c.rows.iter().map(|r| d(vec![
            ("s", Tree::f(r.s)), ("w0", Tree::f(r.w0)), ("n_phi", Tree::u(r.n_phi)),
            ("n_inc", Tree::u(r.n_inc)), ("roots_phi", Tree::fl(&r.roots_phi)),
            ("roots_inc", Tree::fl(&r.roots_inc)), ("worst", Tree::f(r.worst)),
        ])).collect())),
        ("counts_equal", Tree::b(c.counts_equal)), ("n_roots", ul(&c.n_roots)),
        ("worst", Tree::of(c.worst)),
    ]);
    let m = coord_march(&rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S3_PHI, S3_MARGIN, S3_TAUS, false,
                        0.5, 1.2, 0.005, S3_VMAX);
    let u64t = |x: u64| Tree::i(x as i64);
    let mt = d(vec![
        ("phi_lim", Tree::f(m.phi_lim)), ("margin", Tree::f(m.margin)), ("inc", Tree::b(m.inc)),
        ("n", Tree::u(m.n)), ("same_len", Tree::b(m.same_len)), ("worst", Tree::f(m.worst)),
        ("where", m.where_.map_or(Tree::n(), |(k, i)| Tree::L(vec![Tree::s(k), Tree::u(i)]))),
        ("hits", u64t(m.hits)), ("binds", u64t(m.binds)), ("n_armed", Tree::u(m.n_armed)),
        ("n_log", Tree::u(m.n_log)), ("calls_phi", u64t(m.calls_phi)),
        ("calls_inc", u64t(m.calls_inc)), ("fb_phi", u64t(m.fb_phi)), ("fb_inc", u64t(m.fb_inc)),
        ("br_phi", Tree::i(m.br_phi)), ("br_inc", Tree::i(m.br_inc)),
        ("n_distinct", Tree::u(m.n_distinct)), ("n_distinct_accel", Tree::u(m.n_distinct_accel)),
        ("n_live", Tree::u(m.n_live)), ("n_reach", Tree::u(m.n_reach)), ("n_both", Tree::u(m.n_both)),
        ("flips", Tree::u(m.flips)), ("gap_min", Tree::of(m.gap_min)),
        ("gap_med", Tree::of(m.gap_med)), ("gap_max", Tree::of(m.gap_max)),
        ("n_distinct_gap", Tree::u(m.n_distinct_gap)), ("d_max", Tree::of(m.d_max)),
        ("d_med", Tree::of(m.d_med)), ("vacuous", Tree::ob(m.vacuous)),
        ("sched_moved", Tree::f(m.sched_moved)),
    ]);
    let mut out = Kernel::default();
    flat(&ft, "f", &mut out);
    flat(&st, "s", &mut out);
    flat(&ct, "c", &mut out);
    flat(&mt, "m", &mut out);
    out
}

// ------------------------------------------------------------------------- slice 5 — rung 80

const S5_SAT: [f64; 4] = [0.84, 0.85, 0.855, 0.88];

/// `_split_row`'s dict; `wall` is the extra key `split_arrest` merges into each of its rows.
fn split_row(r: &SplitRow, wall: Option<f64>) -> Tree {
    let mut v = vec![
        ("coord", Tree::s(r.coord)), ("phi_lim", Tree::f(r.phi_lim)), ("phi_air", Tree::of(r.phi_air)),
        ("npts", Tree::u(r.npts)), ("phi_lim_built", Tree::of(r.phi_lim_built)),
        ("phi_air_built", Tree::of(r.phi_air_built)), ("phi0", Tree::f(r.phi0)), ("b0", Tree::f(r.b0)),
        ("v0", Tree::f(r.v0)), ("b0_frac", Tree::f(r.b0_frac)), ("min_phi", Tree::f(r.min_phi)),
        ("max_Tt4", Tree::f(r.max_tt4)), ("arrested", Tree::b(r.arrested)),
        ("riding4_valid", Tree::b(r.riding4_valid)), ("n_cut_fuel", Tree::u(r.n_cut_fuel)),
        ("n_cut_gov", Tree::u(r.n_cut_gov)), ("max_req_fuel", Tree::f(r.max_req_fuel)),
        ("max_req_gov", Tree::f(r.max_req_gov)), ("valve_moved", Tree::u(r.valve_moved)),
        ("stator_moved", Tree::u(r.stator_moved)), ("n_riding4", Tree::u(r.n_riding4)),
        ("b_max_hit", Tree::b(r.b_max_hit)),
    ];
    if let Some(w) = wall {
        v.push(("wall", Tree::f(w)));
    }
    d(v)
}

fn split_gains_tree(g: &SplitGains) -> Tree {
    let key = |a: Option<Authority>| a.map_or("None", |x| x.as_str());
    d(vec![
        ("phi_lim", Tree::f(g.phi_lim)), ("coord", Tree::s(g.coord)), ("taus", taus4(g.taus)),
        ("ds", Tree::f(g.ds)),
        ("arms", Tree::L(g.arms.iter().map(|a| d(vec![
            ("phi_air", Tree::of(a.phi_air)), ("n_riding", Tree::u(a.n_riding)),
            ("n_sampled", Tree::u(a.n_sampled)), ("n_interior", Tree::u(a.n_interior)),
            // `GainsArm.skipped` is `(switch, regime)` — the OPPOSITE order to rung 74's tuple.
            ("skipped", d(vec![("switch", Tree::u(a.skipped.0)), ("regime", Tree::u(a.skipped.1))])),
            ("cells", Tree::L(a.cells.iter().map(|c| d(vec![
                ("s", Tree::f(c.s)), ("phi", Tree::f(c.phi)), ("authority", auth(c.authority)),
                ("masked", auth(c.masked)), ("mask_leak", Tree::of(c.mask_leak)),
                ("zeros", Tree::u(c.zeros)), ("c1", Tree::f(c.c1)), ("c0", Tree::f(c.c0)),
                ("cyc_fwd", Tree::f(c.cyc_fwd)), ("cyc_rev", Tree::f(c.cyc_rev)),
                ("pair_RC", Tree::f(c.pair_rc)), ("pair_CV", Tree::f(c.pair_cv)),
            ])).collect())),
            ("authority", Tree::D(a.authority.iter().map(|(x, n)| (key(*x).to_string(), Tree::u(*n))).collect())),
            ("masked", Tree::L(a.masked.iter().map(|x| auth(*x)).collect())),
            ("max_mask_leak", Tree::of(a.max_mask_leak)), ("zeros", ul(&a.zeros)),
            ("max_cyc", Tree::of(a.max_cyc)),
        ])).collect())),
        ("vacuous", Tree::b(g.vacuous)), ("n_interior", ul(&g.n_interior)),
        ("all_differenced", Tree::b(g.all_differenced)),
        ("ever_two_authorities", Tree::b(g.ever_two_authorities)),
        ("control_nonzero", Tree::of(g.control_nonzero)), ("max_mask_leak", Tree::of(g.max_mask_leak)),
    ])
}

fn kernel_r80() -> Kernel {
    let fl = flight();
    let rig = || solve_rig(build_split_wall_cascade);
    let a = split_arrest(&rig(), &fl, S3_LO, S3_HI, S3_TT4MAX,
                         &[0.7700, 0.7725, 0.7731, 0.7732, 0.7740, 0.7800], S5_PHI_FUEL, 0.80,
                         "demand", S3_TAUS, false, 0.5, 1.2, 0.005, S3_VMAX);
    let at = d(vec![
        ("walls", Tree::fl(&a.walls)), ("phi_lim_lo", Tree::f(a.phi_lim_lo)),
        ("phi_air_hi", Tree::f(a.phi_air_hi)), ("coord", Tree::s(a.coord)), ("taus", taus4(a.taus)),
        ("ds", Tree::f(a.ds)),
        ("arms", d(a.arms.iter().map(|(name, arm)| (*name, d(vec![
            ("rows", Tree::L(arm.rows.iter().map(|(w, r)| split_row(r, Some(*w))).collect())),
            ("marched", Tree::fl(&arm.marched)), ("arrested", Tree::fl(&arm.arrested)),
            ("last_march", Tree::of(arm.last_march)), ("first_arrest", Tree::of(arm.first_arrest)),
            ("monotone", Tree::b(arm.monotone)),
        ]))).collect())),
        ("control_bracket", opair(a.control_bracket)), ("owner", Tree::sl(&a.owner)),
    ]);
    let l = split_liveness(&rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S5_PHI_FUEL,
                           &[None, Some(0.76), Some(0.77)], &["demand", "clip"], S3_TAUS, false,
                           0.5, 1.2, 0.005, S3_VMAX);
    let lt = d(vec![
        ("phi_lim", Tree::f(l.phi_lim)),
        ("phi_airs", Tree::L(l.phi_airs.iter().map(|&x| Tree::of(x)).collect())),
        ("coords", Tree::sl(&l.coords)), ("taus", taus4(l.taus)), ("ds", Tree::f(l.ds)),
        ("rows", Tree::L(l.rows.iter().map(|r| split_row(r, None)).collect())),
        ("control_ok", Tree::b(l.control_ok)), ("levers_woke", Tree::b(l.levers_woke)),
        ("fuel_off", Tree::b(l.fuel_off)), ("four_live", Tree::u(l.four_live)),
        ("n_split", Tree::u(l.n_split)),
    ]);
    let airs = [None, Some(0.77), Some(0.80)];
    let gc = wall_gains(&rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S5_PHI_FUEL, &airs, "clip", S3_TAUS,
                         false, 0.5, 1.2, 0.005, S3_VMAX, 5).expect("rung 80's clip gains");
    let gd = wall_gains(&rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S5_PHI_FUEL, &airs, "demand",
                         S3_TAUS, false, 0.5, 1.2, 0.005, S3_VMAX, 5).expect("rung 80's demand gains");
    let s = split_saturation(&rig(), &fl, S3_LO, S3_HI, S3_TT4MAX, S5_PHI_FUEL, &S5_SAT, "demand",
                             S3_TAUS, false, 0.5, 1.2, 0.005, S3_VMAX);
    let st = d(vec![
        ("phi_lim", Tree::f(s.phi_lim)), ("coord", Tree::s(s.coord)),
        ("rows", Tree::L(s.rows.iter().map(|r| split_row(r, None)).collect())),
        ("ds", Tree::f(s.ds)), ("first_sat", Tree::of(s.first_sat)),
        ("last_march", Tree::of(s.last_march)), ("cell", Tree::fl(&s.cell)),
        ("impossible", Tree::b(s.impossible)),
    ]);
    let mut out = Kernel::default();
    flat(&at, "a", &mut out);
    flat(&lt, "l", &mut out);
    flat(&split_gains_tree(&gc), "gc", &mut out);
    flat(&split_gains_tree(&gd), "gd", &mut out);
    flat(&st, "s", &mut out);
    out
}

// ------------------------------------------------------------------------- slices 6/7 — rungs 81/82
//
// Rungs 81-84 are READERS on a rung-80 machine (no table of their own), so `_s6_rig()` /
// `_s7_rig()` are `build_split_wall_cascade` with the reader rungs' attributes. Their converters
// are slice AJ's (`tests/slice_aj_flat/mod.rs`), re-read into Python's tree by `tree_from_flat`.

const S5_PHI_FUEL: f64 = 0.75;
const S6_PHI_AIR: Option<f64> = Some(0.77);
const S7_NBISECT: usize = 10;
const BRACKET: (f64, f64) = (0.004, 0.30);

fn aj_tree(conv: impl FnOnce(&mut slice_aj_flat::Flat)) -> Tree {
    let mut f = slice_aj_flat::Flat::default();
    conv(&mut f);
    tree_from_flat(&f.0)
}

fn kernel_r81() -> Kernel {
    let fl = flight();
    let m = solve_rig(build_split_wall_cascade);
    let c = authority_clock(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, S5_PHI_FUEL, S6_PHI_AIR,
                            &[0.02, 0.05, 0.08, 0.10, 0.12, 0.20], &[0.02, 0.05, 0.20], 0.05, 0.05,
                            &["demand", "clip"], 0.5, 1.2, 0.005, S3_VMAX, false);
    let mut out = Kernel::default();
    flat(&aj_tree(|f| slice_aj_flat::clock(f, "c", &c)), "c", &mut out);
    out
}

fn kernel_r81m() -> Kernel {
    let fl = flight();
    let m = solve_rig(build_split_wall_cascade);
    let r = authority_mask(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, S5_PHI_FUEL, S6_PHI_AIR,
                           &[(0.20, 0.05, 0.05, 0.05), (0.05, 0.05, 0.05, 0.05)], "demand", 0.5,
                           1.2, 0.005, S3_VMAX, false, 1)
        .expect("the mask's gains do not abort on this rig");
    let mut out = Kernel::default();
    flat(&aj_tree(|f| slice_aj_flat::mask(f, "m", &r)), "m", &mut out);
    out
}

fn kernel_r82() -> Kernel {
    let fl = flight();
    let m = solve_rig(build_split_wall_cascade);
    // `ds_fine=None` SKIPS the step control — the Python kernel's recorded choice.
    let r = threshold_law::threshold_law(&m, &fl, S3_LO, S3_HI, S3_TT4MAX, S5_PHI_FUEL, S6_PHI_AIR,
                                         &[0.25, 0.35, 0.50], 0.05, 0.05, 0.05, 0.05, BRACKET,
                                         S7_NBISECT, 1.2, 0.005, None, S3_VMAX, false);
    let mut out = Kernel::default();
    flat(&aj_tree(|f| slice_aj_flat::law(f, "l", &r)), "l", &mut out);
    out
}

fn kernel_r82r() -> Kernel {
    let fl = flight();
    let m = solve_rig(build_split_wall_cascade);
    let r = threshold_law::threshold_reference(
        &m, &fl, S3_LO, S3_HI, S3_TT4MAX, S5_PHI_FUEL, S6_PHI_AIR, 0.35,
        &[0.02, 0.03, 0.05, 0.08, 0.12], 0.05, 0.05, 0.05, BRACKET, S7_NBISECT, 1.2, 0.005,
        S3_VMAX, false);
    let mut out = Kernel::default();
    flat(&aj_tree(|f| slice_aj_flat::reference(f, "f", &r)), "f", &mut out);
    out
}

fn kernel_r82t() -> Kernel {
    let fl = flight();
    let m = solve_rig(build_split_wall_cascade);
    // The test's three walls, not the reader's five — the Python kernel's recorded cut.
    let r = threshold_law::threshold_terms(
        &m, &fl, S3_LO, S3_HI, S3_TT4MAX, S5_PHI_FUEL, S6_PHI_AIR, &[0.745, 0.750, 0.755], 0.35,
        &[0.02, 0.05, 0.20], 0.05, 0.05, BRACKET, S7_NBISECT, 1.2, 0.005, S3_VMAX, false);
    let mut out = Kernel::default();
    flat(&aj_tree(|f| slice_aj_flat::terms(f, "t", &r)), "t", &mut out);
    out
}

// ------------------------------------------------------------------------- the table

type KernelFn = fn() -> Kernel;

/// `KERNELS`, in the Python module's order. A kernel not yet ported is absent; the coverage
/// gate below says how many and which.
const KERNELS: &[(&str, KernelFn)] = &[
    ("cpg", kernel_cpg), ("r66", kernel_r66),
    ("A", kernel_a), ("B", kernel_b), ("C", kernel_c), ("D", kernel_d), ("E", kernel_e),
    ("F", kernel_f),
    ("prop", kernel_prop), ("r7", kernel_r7), ("r8", kernel_r8), ("r10", kernel_r10),
    ("r11", kernel_r11), ("r12", kernel_r12), ("r13", kernel_r13), ("r14", kernel_r14),
    ("r15", kernel_r15), ("r16", kernel_r16), ("r17", kernel_r17), ("r18", kernel_r18),
    ("r22", kernel_r22), ("r23", kernel_r23), ("r24", kernel_r24), ("r25", kernel_r25),
    ("r27", kernel_r27), ("r28", kernel_r28),
    ("r67", kernel_r67), ("r68", kernel_r68), ("r69", kernel_r69),
    ("r70", kernel_r70), ("r71", kernel_r71), ("r72", kernel_r72), ("r73", kernel_r73),
    ("r74", kernel_r74), ("r75", kernel_r75), ("r76", kernel_r76), ("r77", kernel_r77),
    ("r78", kernel_r78), ("r79", kernel_r79), ("r80", kernel_r80),
    ("r81", kernel_r81), ("r81m", kernel_r81m), ("r82", kernel_r82), ("r82r", kernel_r82r),
    ("r82t", kernel_r82t),
];

// ------------------------------------------------------------------------- the gates

/// Both comparisons for one kernel. THE ANCHOR: every token equal, both key sets equal — the
/// anchor is Rust's own frozen output, byte-identical to the PyPy capture (see
/// `anchor_is_byte_identical_to_the_pypy_capture`), so this is Rust == PyPy. CPYTHON: the module's
/// `_check` — shape first, then `_close` per value at the kernel's (tol, abs_tol).
fn check(name: &str) {
    let f = KERNELS.iter().find(|(n, _)| *n == name).unwrap_or_else(|| panic!("{name} not ported")).1;
    let got = f();

    let anchor = load_anchor();
    let want = anchor.get(name).unwrap_or_else(|| panic!("no anchor values for {name}"));
    let missing: Vec<_> = want.keys().filter(|k| !got.0.contains_key(*k)).take(8).collect();
    let extra: Vec<_> = got.0.keys().filter(|k| !want.contains_key(*k)).take(8).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "kernel {name}: THE KERNEL'S SHAPE CHANGED, NOT ITS ARITHMETIC\n  missing ({}): \
         {missing:?}\n  extra ({}): {extra:?}",
        want.keys().filter(|k| !got.0.contains_key(*k)).count(),
        got.0.keys().filter(|k| !want.contains_key(*k)).count(),
    );
    let bad: Vec<String> = want
        .iter()
        .filter(|(k, t)| got.0[*k].token() != **t)
        .map(|(k, t)| format!("  {k}\n    anchor {t}\n    rust   {}", got.0[k].token()))
        .collect();
    assert!(bad.is_empty(), "kernel {name}: {} of {} values moved off the anchor (bit-equality):\n{}\n\
            Do NOT regenerate reflexively — a moved value is a finding.",
            bad.len(), want.len(), bad.iter().take(6).cloned().collect::<Vec<_>>().join("\n"));

    let cp = load_cpython();
    let golden = &cp.kernels.iter().find(|(n, _)| n == name).expect("CPython kernel").1;
    assert_eq!(
        golden.keys().collect::<Vec<_>>(), got.0.keys().collect::<Vec<_>>(),
        "kernel {name}: key set differs from the CPython golden"
    );
    let (t, a) = (tol(name), abs_tol(name));
    let bad: Vec<String> = golden
        .iter()
        .filter_map(|(k, w)| {
            let (ok, err) = close(&got.0[k], w, t, a);
            (!ok).then(|| format!("  {k}: golden {w:?} rust {:?} rel {err:.3e}", got.0[k]))
        })
        .collect();
    assert!(bad.is_empty(), "kernel {name}: {} values beyond tol={t:e} abs_tol={a:e}:\n{}",
            bad.len(), bad.iter().take(6).cloned().collect::<Vec<_>>().join("\n"));
}

/// The anchor file's bytes for a set of computed kernels: `kernel<TAB>key<TAB>token`, kernels in
/// `KERNELS` order and keys sorted, then `_tol` / `_abs_tol` per kernel — `dump_fingerprint.py`'s
/// layout exactly, so the two producers can be compared byte for byte.
fn anchor_text(computed: &[(&str, Kernel)]) -> String {
    let mut s = String::new();
    for (name, k) in computed {
        for (key, v) in &k.0 {
            s.push_str(&format!("{name}\t{key}\t{}\n", v.token()));
        }
    }
    for (name, _) in computed {
        s.push_str(&format!("_tol\t{name}\t{}\n", V::F(tol(name)).token()));
        s.push_str(&format!("_abs_tol\t{name}\t{}\n", V::F(abs_tol(name)).token()));
    }
    s
}

type Golden = std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>;

/// The published table, one line per kernel, from the ANCHOR against the CPython golden.
fn deviation_text(anchor: &Golden) -> String {
    let cp = load_cpython();
    let mut s = String::from(
        "kernel\tn_values\tn_differ\tmax_rel\tmax_abs\ttol\tabs_tol\tmax_used\tleg\n");
    for (name, golden) in &cp.kernels {
        let got: std::collections::BTreeMap<String, V> =
            anchor[name].iter().map(|(k, t)| (k.clone(), v_from_token(t))).collect();
        let dv = deviation(name, &got, golden);
        s.push_str(&format!("{name}\t{}\t{}\t{:.3e}\t{:.3e}\t{:e}\t{:e}\t{:.4}\t{}\n", dv.n,
                            dv.n_differ, dv.max_rel, dv.max_abs, tol(name), abs_tol(name),
                            dv.max_used, dv.leg));
    }
    s
}

/// THE REGENERATION MODE — the only writer of the anchor and of the deviation table. A no-op
/// unless `FINGERPRINT_REGEN=1`: an anchor that rewrites itself on every run anchors nothing.
#[test]
fn regenerate_anchor() {
    if std::env::var("FINGERPRINT_REGEN").as_deref() != Ok("1") {
        eprintln!("regenerate_anchor: FINGERPRINT_REGEN is not 1 — nothing written (by design)");
        return;
    }
    let computed: Vec<(&str, Kernel)> = KERNELS.iter().map(|(n, f)| (*n, f())).collect();
    let text = anchor_text(&computed);
    std::fs::write(anchor_path(), &text).expect("write the anchor");
    std::fs::write(deviation_path(), deviation_text(&parse_tsv(&text))).expect("write the table");
    eprintln!("regenerate_anchor: wrote {:?} and {:?}", anchor_path(), deviation_path());
}

/// THE HAND-OVER, recorded: Rust's anchor and the PyPy capture are the SAME BYTES — two
/// independent producers (Rust's kernels, and the Python module's own `KERNELS` on PyPy) agree on
/// every one of the 35 075 values and on both tolerance tables. A deliberate future regeneration
/// will break this; that is the moment to retire it, with the reason, not to re-bless it.
#[test]
fn anchor_is_byte_identical_to_the_pypy_capture() {
    let text = std::fs::read_to_string(anchor_path()).expect("the Rust anchor exists");
    assert!(text == PYPY_TSV, "the Rust anchor and the PyPy capture differ");
    let n: usize = load_anchor().iter().filter(|(k, _)| !k.starts_with('_')).map(|(_, v)| v.len()).sum();
    assert_eq!(n, 35_075, "the anchor's value count, measured");
}

/// The PUBLISHED table is the one the anchor implies — recomputed from the anchor and the CPython
/// golden on every run, so a regenerated anchor cannot leave a stale table behind. And no kernel
/// spends its whole band: every value is inside tolerance with room.
#[test]
fn deviation_table_is_the_published_one() {
    let anchor = load_anchor();
    let table = deviation_text(&anchor);
    let published = std::fs::read_to_string(deviation_path()).expect("the published table exists");
    assert!(table == published, "the deviation table no longer matches its published copy");
    for line in table.lines().skip(1) {
        let used: f64 = line.split('\t').nth(7).unwrap().parse().unwrap();
        assert!(used < 1.0, "a kernel spends its whole tolerance: {line}");
    }
}

/// THE MODULE's `test_instrument_arms_are_not_vacuous`, read from the ANCHOR. The regeneration
/// mode is the only writer of the anchor, and nothing else stops it pinning an arm whose
/// instrument sampled no live base point — every row list empty, every derived field `None` — an
/// arm that passes its own gate forever and guards nothing. Same four rules, same thresholds:
/// no EMPTY `rows` / `govs` / `walls` list; at least one non-empty `cells` list where any exist
/// (the WEAK rule — rung 80's shared-wall `demand` arm is empty on purpose); at most 15 % of the
/// pinned values `None`; at least 8 distinct floats.
#[test]
fn instrument_arms_are_not_vacuous() {
    let anchor = load_anchor();
    let instrument = ["r67", "r68", "r69", "r70", "r71", "r72", "r73", "r74", "r75", "r76", "r77",
                      "r78", "r79", "r80", "r81", "r81m", "r82", "r82r", "r82t"];
    for name in instrument {
        if let Err(e) = vacuity(&anchor[name]) {
            panic!("{name}: {e}");
        }
    }
}

/// The four rules, as a function so the detector itself can be shown to fire.
fn vacuity(arm: &std::collections::BTreeMap<String, String>) -> Result<(), String> {
    let empty: Vec<&String> = arm.iter()
        .filter(|(k, t)| (k.ends_with("rows#n") || k.ends_with("govs#n") || k.ends_with("walls#n"))
                         && t.as_str() == "i:0")
        .map(|(k, _)| k).collect();
    if !empty.is_empty() {
        return Err(format!("EMPTY row list(s) {empty:?} — the arm pins nothing"));
    }
    let cells: Vec<&String> = arm.iter().filter(|(k, _)| k.ends_with("cells#n")).map(|(_, t)| t).collect();
    if !(cells.is_empty() || cells.iter().any(|t| t.as_str() != "i:0")) {
        return Err(format!("EVERY one of its {} cell lists is empty", cells.len()));
    }
    let nones = arm.values().filter(|t| t.as_str() == "n").count();
    if nones as f64 > 0.15 * arm.len() as f64 {
        return Err(format!("{nones}/{} pinned values are None — the instrument was idle", arm.len()));
    }
    let floats: std::collections::BTreeSet<&String> = arm.values().filter(|t| t.starts_with("f:")).collect();
    if floats.len() < 8 {
        return Err(format!("only {} distinct floats pinned — vacuous", floats.len()));
    }
    Ok(())
}

/// The detector FIRES: a real arm passes, and the same arm with one row list emptied, or with
/// every float replaced by `None`, is refused — so a green `instrument_arms_are_not_vacuous` is a
/// measurement and not a detector that cannot see.
#[test]
fn the_vacuity_detector_fires() {
    let anchor = load_anchor();
    let real = anchor["r68"].clone();
    assert!(vacuity(&real).is_ok());
    let mut emptied = real.clone();
    emptied.insert("g.rows#n".into(), "i:0".into());
    assert!(vacuity(&emptied).is_err(), "an emptied row list went unseen");
    let idle: std::collections::BTreeMap<String, String> =
        real.iter().map(|(k, t)| (k.clone(), if t.starts_with("f:") { "n".into() } else { t.clone() })).collect();
    assert!(vacuity(&idle).is_err(), "an idle instrument went unseen");
}

/// The gate reads a COPY of the CPython anchor kept under `rust/oracle/`, so it still compiles once
/// AU deletes `tests/`. While the original exists, the copy must be its exact bytes; after the
/// delete the copy IS the record, and this says so instead of failing.
#[test]
fn the_cpython_copy_is_the_audit_record() {
    let original = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..").join("tests").join("golden").join("numeric_fingerprint.json");
    match std::fs::read_to_string(&original) {
        Ok(text) => assert!(text == CPYTHON_JSON, "the rust/oracle copy has drifted from {original:?}"),
        Err(_) => eprintln!("{original:?} is absent (the Python tree is deleted); the copy is the record"),
    }
}

/// THE MODULE's `test_golden_file_declares_its_provenance`: the CPython golden's whole value is
/// being CPython's, so its meta block must say so, completely. The Rust anchor's provenance is
/// `anchor_is_byte_identical_to_the_pypy_capture` — it carries no meta of its own.
#[test]
fn cpython_golden_declares_its_provenance() {
    let cp = load_cpython();
    let get = |k: &str| cp.meta.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    assert!(matches!(get("implementation"), Some(Json::Str(s)) if s == "cpython"),
            "the CPython golden no longer says it was generated by CPython");
    for k in ["version", "generated", "repo_sha"] {
        assert!(matches!(get(k), Some(Json::Str(s)) if !s.is_empty()),
                "golden meta is missing {k:?} — provenance must be complete");
    }
}

macro_rules! gate {
    ($($t:ident => $n:literal),* $(,)?) => { $( #[test] fn $t() { check($n); } )* };
}

gate! {
    k_cpg => "cpg", k_r66 => "r66", k_a => "A", k_b => "B", k_c => "C", k_d => "D", k_e => "E", k_f => "F",
    k_prop => "prop", k_r7 => "r7", k_r8 => "r8", k_r10 => "r10", k_r11 => "r11",
    k_r12 => "r12", k_r13 => "r13", k_r14 => "r14", k_r15 => "r15", k_r16 => "r16",
    k_r17 => "r17", k_r18 => "r18", k_r22 => "r22", k_r23 => "r23", k_r24 => "r24",
    k_r25 => "r25", k_r27 => "r27", k_r28 => "r28",
    k_r67 => "r67", k_r68 => "r68", k_r69 => "r69", k_r70 => "r70", k_r71 => "r71", k_r72 => "r72", k_r73 => "r73",
    k_r74 => "r74", k_r75 => "r75", k_r76 => "r76", k_r77 => "r77",
    k_r78 => "r78", k_r79 => "r79", k_r80 => "r80",
    k_r81 => "r81", k_r81m => "r81m", k_r82 => "r82", k_r82r => "r82r", k_r82t => "r82t",
}

/// The transcribed `TOL` / `ABS_TOL` equal the values the dump wrote FROM the module.
#[test]
fn tolerance_tables_match_the_module() {
    let pypy = load_pypy();
    let (tols, abss) = (&pypy["_tol"], &pypy["_abs_tol"]);
    assert_eq!(tols.len(), 45, "the module's KERNELS has 45 entries");
    for (name, t) in tols {
        assert_eq!(V::F(tol(name)).token(), *t, "TOL[{name}] transcribed wrong");
        assert_eq!(V::F(abs_tol(name)).token(), abss[name], "ABS_TOL[{name}] transcribed wrong");
    }
}

/// The PyPy capture and the CPython golden hold the SAME kernels and the SAME key sets — the
/// capture ran the module's own `KERNELS`, so a difference would mean the module and the golden
/// had drifted apart before this slice started.
#[test]
fn the_two_goldens_have_one_shape() {
    let pypy = load_pypy();
    let cp = load_cpython();
    let names: Vec<&String> = cp.kernels.iter().map(|(n, _)| n).collect();
    let pnames: Vec<&String> = pypy.keys().filter(|k| !k.starts_with('_')).collect();
    let mut a = names.clone();
    a.sort();
    assert_eq!(a, pnames);
    let mut total = 0;
    for (n, kv) in &cp.kernels {
        assert_eq!(kv.keys().collect::<Vec<_>>(), pypy[n].keys().collect::<Vec<_>>(), "{n}");
        total += kv.len();
    }
    assert_eq!(total, 35_075, "the fingerprint's value count, measured");
}

/// `from_hex` reads every float `float.hex` wrote: on every key where the dump's own `_close`
/// measured NO difference, the parsed CPython float carries PyPy's exact bits.
#[test]
fn hex_reader_round_trips() {
    let pypy = load_pypy();
    let cp = load_cpython();
    let mut n = 0;
    for (name, kv) in &cp.kernels {
        for (k, v) in kv {
            if let V::F(x) = v {
                let t = &pypy[name][k];
                if x.is_nan() {
                    assert_eq!(t, "f:nan");
                } else if let Some(bits) = t.strip_prefix("f:") {
                    if u64::from_str_radix(bits, 16).ok() == Some(x.to_bits()) {
                        n += 1;
                    }
                }
            }
        }
    }
    // a broken reader would agree with PyPy on (almost) nothing
    assert!(n > 25_000, "only {n} floats round-tripped");
    assert_eq!(from_hex("-0x0.0p+0").to_bits(), (-0.0f64).to_bits());
    assert_eq!(from_hex("0x0.0000000000001p-1022"), f64::from_bits(1));
    assert_eq!(from_hex("0x1.fffffffffffffp+1023"), f64::MAX);
    assert_eq!(from_hex("0x1.8p+1"), 3.0);
}

/// Every one of the module's 45 kernels is ported, IN THE MODULE's ORDER (the anchor's byte
/// layout depends on it), and each has its own gate — a kernel in the table with no `gate!` line
/// would be computed by nothing but the regeneration run.
#[test]
fn coverage() {
    // The CPython JSON is key-SORTED; the capture's `_tol` lines are in the module's own order.
    let module: Vec<&str> = PYPY_TSV.lines().filter_map(|l| l.strip_prefix("_tol\t"))
        .map(|l| l.split('\t').next().unwrap()).collect();
    let ported: Vec<&str> = KERNELS.iter().map(|(n, _)| *n).collect();
    assert_eq!(ported, module, "KERNELS must list the module's kernels, in its order");
    let src = include_str!("fingerprint.rs");
    for n in ported {
        assert!(src.contains(&format!("=> \"{n}\"")), "kernel {n} has no gate! line");
    }
}
