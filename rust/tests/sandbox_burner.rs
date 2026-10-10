//! The sandbox's slice 5 — the BURNER view (`src/sandbox_burner.rs`, `docs/plans/sandbox-plan.md`
//! § 13, what the build found § 13.9).
//!
//! What these hold it to, and where each value comes from:
//! - the inlet IS slice 1's design run on the equilibrium gas (`sandbox::run`), bit for bit;
//! - every route — instant, set time, jets, and each of the eight mixing models — IS one direct
//!   `zoned_nox` on options built HERE from literal numbers, never through the module's own builder,
//!   at NON-default knob values throughout (a knob parsed and dropped could not pass a bit-equality);
//! - the readouts beside it (prompt, NOx flow, rung 7's perfectly-mixed number, the nozzle, the quench
//!   path) are the model's own calls, bit for bit;
//! - the sweep grids hold each mixing model's OWN optimum (imposed `C_opt` or derived from `k_p`);
//! - the crash map (plan § 13.9): each pre-check fires on its case and only there, and every plain-words
//!   entry is driven by a request that raises its message.

use std::panic::{catch_unwind, AssertUnwindSafe};
use turbojet::engine::EngineResult;
use turbojet::gas::{f_stoich, Gas};
use turbojet::nox::*;
use turbojet::sandbox::{call, run, GasModel, Settings};
use turbojet::sandbox_burner::*;
use turbojet::visuals::Json;

fn quiet<T>(g: impl FnOnce() -> T) -> Result<T, String> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = catch_unwind(AssertUnwindSafe(g)).map_err(|e| {
        e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|m| m.to_string())).unwrap_or_default()
    });
    std::panic::set_hook(hook);
    r
}

fn num(j: &Json, k: &str) -> f64 {
    match j.get(k) {
        Some(Json::Float(x)) => *x,
        Some(Json::Int(n)) => *n as f64,
        other => panic!("{k}: not a number: {other:?}"),
    }
}

fn opt(j: &Json, k: &str) -> Option<f64> {
    match j.get(k) {
        Some(Json::Null) => None,
        Some(Json::Float(x)) => Some(*x),
        Some(Json::Int(n)) => Some(*n as f64),
        other => panic!("{k}: not a number or null: {other:?}"),
    }
}

fn ask(op: &str, design: &str, burner: &str) -> Json {
    Json::parse(&call(&format!("{{\"op\":\"{op}\",\"settings\":{design},\"burner\":{burner}}}")))
}

/// Slice 1's own design run, on the equilibrium gas whatever the design's gas.
fn slice1(design: &str) -> EngineResult {
    let mut s = Settings::from_json(&Json::parse(design)).unwrap();
    s.gas = GasModel::Equilibrium;
    run(&s).unwrap().result
}

/// `(far, Tt3, Tt4, pt4)` off a run — `panels::nox::design_quad`'s reading.
fn quad(r: &EngineResult) -> (f64, f64, f64, f64) {
    let (s3, s4) = (r.station("3"), r.station("4"));
    (s4.far, s3.tt, s4.tt, s4.pt)
}

// ==========================================================================================
// The inlet
// ==========================================================================================

#[test]
fn the_inlet_is_slice_1s_design_run_on_the_equilibrium_gas() {
    for design in ["{}", "{\"gas\":\"perfect\"}", "{\"pi_c\":20,\"Tt4\":1800}", "{\"nozzle\":\"convergent\",\"M0\":1.6}",
                   "{\"gas\":\"thermally_perfect\",\"T0\":288.15,\"p0\":101325,\"M0\":0.3}"] {
        let j = ask("burner_inlet", design, "{}");
        assert_eq!(num(&j, "ok"), 1.0, "{design}: {j:?}");
        let r = slice1(design);
        let (far, tt3, tt4, pt4) = quad(&r);
        assert_eq!(num(&j, "Tt3").to_bits(), tt3.to_bits(), "{design}: Tt3");
        assert_eq!(num(&j, "Tt4").to_bits(), tt4.to_bits(), "{design}: Tt4");
        assert_eq!(num(&j, "far").to_bits(), far.to_bits(), "{design}: far");
        assert_eq!(num(&j, "pt4").to_bits(), pt4.to_bits(), "{design}: pt4");
        assert_eq!(num(&j, "phi").to_bits(), (far / f_stoich()).to_bits(), "{design}: overall richness");
        let mdot = Settings::from_json(&Json::parse(design)).unwrap().mdot;
        assert_eq!(num(&j, "fuel_flow").to_bits(), (far * mdot).to_bits(), "{design}: fuel flow");
    }
}

// ==========================================================================================
// Every route IS one zoned_nox — options built here, from literals, at non-default knobs
// ==========================================================================================

/// The non-default knobs every route test runs at (the jets' and the front zone's).
const BURNER: &str = "\"phi_p\":1.4,\"tau\":0.002,\"super_eq_o\":1,\"prompt\":1,\"prompt_peak\":3.0,\
                      \"tau_q\":0.0007,\"J\":30,\"H\":0.12,\"U_c\":60,\"C_e\":0.18,\"shape_n\":1.5";

fn base() -> ZonedNoxOpts {
    ZonedNoxOpts {
        tau: 0.002, super_eq_o: true,
        prompt: Some(PromptNo { peak_ei: 3.0, n_carbon: 12.0, ea: 303474.0, t_ref: 2400.0, phi_ref: 1.24, phi_valid_max: 1.6 }),
        nsteps: 4000, quench_ngrid: 60, quench_nsteps: 2000,
        ..ZonedNoxOpts::default()
    }
}

fn jet() -> JetMixing { JetMixing { j: 30.0, h: 0.12, u_c: 60.0, c_e: 0.18, shape_n: 1.5 } }

/// Every `state` field the page gets, against the struct, bit for bit (a `None` is a `null`).
fn same_state(j: &Json, z: &ZonedNoxState, what: &str) {
    let st = j.get("state").unwrap_or_else(|| panic!("{what}: no state: {j:?}"));
    let fields: [(&str, Option<f64>); 46] = [
        ("phi_primary", Some(z.phi_primary)), ("far_primary", Some(z.far_primary)), ("alpha", Some(z.alpha)),
        ("t_primary", Some(z.t_primary)), ("t_mix", Some(z.t_mix)), ("ei_no", Some(z.primary.ei_no)),
        ("x_no_primary", Some(z.primary.x_no)), ("ppm_primary", Some(z.primary.x_no * 1e6)),
        ("fraction_of_equil", Some(z.primary.fraction_of_equil())), ("x_no_mix", Some(z.x_no_mix)),
        ("ppm_mix", Some(z.x_no_mix * 1e6)), ("o_multiplier", Some(z.o_multiplier)),
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
    for (k, want) in fields {
        let got = opt(st, k);
        assert_eq!(got.map(f64::to_bits), want.map(f64::to_bits), "{what}: {k}: page {got:?} vs model {want:?}");
    }
}

/// One route: the page's `burner` against a direct `zoned_nox` with `o`, its headline field, and the
/// readouts beside it.
fn route(quench: &str, closure: &str, knobs: &str, o: ZonedNoxOpts, headline: fn(&ZonedNoxState) -> Option<f64>) {
    let what = format!("{quench}/{closure}");
    let burner = format!("{{{BURNER},\"quench\":\"{quench}\",\"closure\":\"{closure}\",\"knobs\":{{{knobs}}}}}");
    let j = ask("burner", "{}", &burner);
    assert_eq!(num(&j, "ok"), 1.0, "{what}: {j:?}");
    let r = slice1("{}");
    let (far, tt3, tt4, pt4) = quad(&r);
    let eq = Gas::reacting_equilibrium();
    let z = eq.zoned_nox(far, tt3, tt4, pt4, 1.4, o);
    same_state(&j, &z, &what);
    let thermal = headline(&z).unwrap();
    assert_eq!(num(&j, "ei_thermal").to_bits(), thermal.to_bits(), "{what}: headline");
    assert!(z.ei_no_prompt > 0.0, "{what}: the prompt switch must be live here");
    let total = thermal + z.ei_no_prompt;
    assert_eq!(num(&j, "ei_total").to_bits(), total.to_bits(), "{what}: total = thermal + prompt");
    assert_eq!(num(&j, "nox_flow").to_bits(), (total * (far * 20.0)).to_bits(), "{what}: NOx flow = EI × fuel flow");
    let mixed = eq.thermal_nox(far, tt4, pt4, ThermalNoxOpts { tau: 0.002, nsteps: 4000, ..ThermalNoxOpts::default() });
    assert_eq!(num(&j, "ei_mixed").to_bits(), mixed.ei_no.to_bits(), "{what}: rung 7's perfectly mixed burner");
}

#[test]
fn the_instant_and_set_time_quenches_are_zoned_nox() {
    route("instant", "none", "", base(), |z| Some(z.primary.ei_no));
    route("time", "none", "", ZonedNoxOpts { tau_q: Some(0.0007), ..base() }, |z| z.ei_no_quenched);
    // A mixing model set while the quench is not the jets is not passed on (the model would refuse it).
    route("time", "pdf", "", ZonedNoxOpts { tau_q: Some(0.0007), ..base() }, |z| z.ei_no_quenched);
}

#[test]
fn the_jets_and_the_fast_mixing_models_are_zoned_nox() {
    route("jets", "none", "", ZonedNoxOpts { mixing: Some(jet()), ..base() }, |z| z.ei_no_quenched);
    route("jets", "two_stream", "\"s\":0.05,\"c_opt\":2.2,\"tau_res\":0.002,\"k_u\":2.0,\"b_u\":1.5,\"w_max\":0.6",
          ZonedNoxOpts { mixing: Some(jet()), unmixedness: Some(Unmixedness { s: 0.05, c_opt: 2.2, tau_res: 0.002, k_u: 2.0,
                                                                                b_u: 1.5, w_max: 0.6 }), ..base() },
          |z| z.ei_no_unmixed);
}

#[test]
fn the_curve_mixing_models_are_zoned_nox() {
    let b = |n: &str| format!("\"s\":0.05,\"c_opt\":2.2,{n}");
    route("jets", "pdf", &b("\"k_g\":0.35,\"g_max\":0.25"),
          ZonedNoxOpts { mixing: Some(jet()), pdf: Some(MixingPdf { s: 0.05, c_opt: 2.2, k_g: 0.35, g_max: 0.25, n_bell: 80,
                                                                     n_quad: 200 }), ..base() },
          |z| z.ei_no_pdf);
    route("jets", "pdf_quench", &b("\"k_g\":0.35,\"g_max\":0.25,\"tau_res\":0.002,\"b_u\":2.5"),
          ZonedNoxOpts { mixing: Some(jet()), pdf_quench: Some(QuenchPdf { s: 0.05, c_opt: 2.2, k_g: 0.35, g_max: 0.25,
                                                                           tau_res: 0.002, b_u: 2.5, n_bell: 80, n_quad: 200 }),
                         ..base() },
          |z| z.ei_no_pdf_quench);
    route("jets", "transported", &b("\"c_phi\":1.8,\"da_opt\":2.5,\"w_cov\":0.8,\"tau_mix\":0.002"),
          ZonedNoxOpts { mixing: Some(jet()), transported: Some(TransportedPdf { s: 0.05, c_opt: 2.2, c_phi: 1.8, da_opt: 2.5,
                                                                                w_cov: 0.8, tau_mix: 0.002, n_bell: 80,
                                                                                n_quad: 200, n_ode: 400 }), ..base() },
          |z| z.ei_no_transported);
    route("jets", "spatial", "\"s\":0.05,\"k_p\":0.3,\"k_y\":0.3,\"k_z\":0.26",
          ZonedNoxOpts { mixing: Some(jet()), spatial: Some(SpatialPdf { s: 0.05, k_p: 0.3, k_y: 0.3, k_z: 0.26, ny: 32, nz: 32,
                                                                        n_bell: 80, n_quad: 200 }), ..base() },
          |z| z.ei_no_spatial);
}

#[test]
fn the_per_pocket_model_is_zoned_nox() {
    route("jets", "pocket", "\"s\":0.05,\"c_opt\":2.2,\"k_g\":0.35,\"g_max\":0.25,\"tau_res\":0.002,\"b_u\":2.5",
          ZonedNoxOpts { mixing: Some(jet()), pocket_quench: Some(PocketQuenchPdf { s: 0.05, c_opt: 2.2, k_g: 0.35, g_max: 0.25,
                                                                                   tau_res: 0.002, b_u: 2.5, n_bell: 40,
                                                                                   n_quad: 160 }), ..base() },
          |z| z.ei_no_pocket_quench);
}

#[test]
fn the_plane_in_time_model_is_zoned_nox() {
    route("jets", "spatial_dwell", "\"s\":0.05,\"k_p\":0.3,\"k_y\":0.3,\"k_z\":0.26",
          ZonedNoxOpts { mixing: Some(jet()), spatial_dwell: Some(SpatialDwellPdf { s: 0.05, k_p: 0.3, k_y: 0.3, k_z: 0.26, ny: 32,
                                                                                   nz: 32, nt: 24, n_bell: 40, n_quad: 160 }),
                         ..base() },
          |z| z.ei_no_spatial_dwell);
}

#[test]
fn the_local_rate_model_is_zoned_nox() {
    route("jets", "spatial_local", "\"s\":0.05,\"k_p\":0.3,\"k_y\":0.3,\"k_z\":0.26",
          ZonedNoxOpts { mixing: Some(jet()), spatial_local: Some(SpatialLocalPdf { s: 0.05, k_p: 0.3, k_y: 0.3, k_z: 0.26, ny: 32,
                                                                                   nz: 32, n_bell: 40, n_quad: 160 }),
                         ..base() },
          |z| z.ei_no_spatial_local);
}

// ==========================================================================================
// The knobs
// ==========================================================================================

#[test]
fn each_mixing_models_knobs_open_at_the_models_own_defaults() {
    let d = Json::parse(&call("{\"op\":\"burner_defaults\"}"));
    let Some(Json::List(models)) = d.get("models") else { panic!("no models: {d:?}") };
    let want: &[(&str, &[(&str, f64)])] = &[
        ("none", &[]),
        ("two_stream", &[("s", 0.0625), ("c_opt", 2.5), ("tau_res", 2.5e-3), ("k_u", 2.5), ("b_u", 1.0), ("w_max", 0.7)]),
        ("pdf", &[("s", 0.0625), ("c_opt", 2.5), ("k_g", 0.3), ("g_max", 0.3)]),
        ("pdf_quench", &[("s", 0.0625), ("c_opt", 2.5), ("k_g", 0.3), ("g_max", 0.3), ("tau_res", 2.5e-3), ("b_u", 3.0)]),
        ("pocket", &[("s", 0.0625), ("c_opt", 2.5), ("k_g", 0.3), ("g_max", 0.3), ("tau_res", 2.5e-3), ("b_u", 3.0)]),
        ("transported", &[("s", 0.0625), ("c_opt", 2.5), ("c_phi", 2.0), ("da_opt", 2.0), ("w_cov", 1.0), ("tau_mix", 2.5e-3)]),
        ("spatial", &[("s", 0.0625), ("k_p", 0.316), ("k_y", 0.28), ("k_z", 0.28)]),
        ("spatial_dwell", &[("s", 0.0625), ("k_p", 0.316), ("k_y", 0.28), ("k_z", 0.28)]),
        ("spatial_local", &[("s", 0.0625), ("k_p", 0.316), ("k_y", 0.28), ("k_z", 0.28)]),
    ];
    assert_eq!(models.len(), want.len());
    for (m, (key, knobs)) in models.iter().zip(want) {
        assert_eq!(m.get("key"), Some(&Json::from(*key)));
        let Some(Json::Obj(kv)) = m.get("knobs") else { panic!("{key}: knobs") };
        let got: Vec<(&str, f64)> = kv.iter().map(|(k, v)| (k.as_str(), match v { Json::Float(x) => *x, _ => f64::NAN })).collect();
        assert_eq!(got, knobs.to_vec(), "{key}");
    }
    // The view opens on rung 8's stoichiometric front zone, the instant quench, the jets' model defaults.
    let b = d.get("burner").unwrap();
    assert_eq!((num(b, "phi_p"), num(b, "tau"), num(b, "C_e"), num(b, "H"), num(b, "U_c"), num(b, "shape_n")),
               (1.0, 3e-3, 0.15, 0.10, 75.0, 2.0));
    assert_eq!(b.get("quench"), Some(&Json::from("instant")));
}

#[test]
fn a_knob_the_mixing_model_does_not_have_is_refused_and_a_new_model_starts_at_its_defaults() {
    let j = ask("burner", "{}", "{\"quench\":\"jets\",\"closure\":\"pdf\",\"knobs\":{\"k_p\":0.3}}");
    assert_eq!(num(&j, "ok"), 0.0);
    assert!(format!("{j:?}").contains("has no knob \\\"k_p\\\""), "{j:?}");
    let b = BurnerSettings::from_json(&Json::parse("{\"closure\":\"spatial\"}")).unwrap();
    assert_eq!(b.knobs, Closure::Spatial.knob_defaults());
    assert!(BurnerSettings::from_json(&Json::parse("{\"closure\":\"bogus\"}")).is_err());
    assert!(BurnerSettings::from_json(&Json::parse("{\"quench\":\"slow\"}")).is_err());
}

// ==========================================================================================
// The pre-checks — each on its case, and only there
// ==========================================================================================

#[test]
fn the_lean_floor_is_the_models_own_bar() {
    let phi = num(&ask("burner_inlet", "{}", "{}"), "phi");
    // AT the burner's own richness the front zone takes all the air: it runs (rung 7's well-mixed burner).
    let at = ask("burner", "{}", &format!("{{\"phi_p\":{phi}}}"));
    assert_eq!(num(&at, "ok"), 1.0, "{at:?}");
    assert!((num(at.get("state").unwrap(), "alpha") - 1.0).abs() < 1e-12);
    // Just leaner: refused in words — and the model itself would have stopped there.
    let lean = phi * (1.0 - 1e-6);
    let j = ask("burner", "{}", &format!("{{\"phi_p\":{lean}}}"));
    assert_eq!(num(&j, "ok"), 0.0);
    assert!(format!("{j:?}").contains("cannot be leaner than the burner as a whole"), "{j:?}");
    let (far, tt3, tt4, pt4) = quad(&slice1("{}"));
    let m = quiet(|| Gas::reacting_equilibrium().zoned_nox(far, tt3, tt4, pt4, lean, ZonedNoxOpts::default()));
    assert!(m.is_err_and(|e| e.contains("primary air fraction")), "the model must refuse it too");
    // The soot limit: 2 runs, just above is refused.
    assert_eq!(num(&ask("burner", "{}", "{\"phi_p\":2.0}"), "ok"), 1.0);
    let j = ask("burner", "{}", "{\"phi_p\":2.0001}");
    assert!(format!("{j:?}").contains("soot"), "{j:?}");
    // A design the cycle refuses is refused before the burner (slice 1's words).
    let j = ask("burner", "{\"Tt4\":500}", "{}");
    assert!(format!("{j:?}").contains("not above the compressor-exit temperature"), "{j:?}");
}

// ==========================================================================================
// The sweep grids
// ==========================================================================================

#[test]
fn each_sweep_grid_holds_its_models_own_optimum() {
    let (_, _, i) = design_run(&Settings::defaults()).unwrap();
    let parse = |s: &str| BurnerSettings::from_json(&Json::parse(s)).unwrap();
    // Imposed (rungs 12–18): J_opt = (C_opt·H/S)². Derived (22–24): C_opt = 1/(4k_p²).
    let cases = [
        ("{\"quench\":\"jets\",\"closure\":\"pdf\"}", (2.5f64 * 0.10 / 0.0625).powi(2)),
        ("{\"quench\":\"jets\",\"closure\":\"two_stream\",\"H\":0.15,\"knobs\":{\"s\":0.05,\"c_opt\":3}}", (3.0f64 * 0.15 / 0.05).powi(2)),
        ("{\"quench\":\"jets\",\"closure\":\"spatial\"}", (1.0 / (4.0 * 0.316f64 * 0.316) * 0.10 / 0.0625).powi(2)),
        ("{\"quench\":\"jets\",\"closure\":\"spatial_local\",\"knobs\":{\"k_p\":0.4}}", (1.0 / (4.0 * 0.4f64 * 0.4) * 0.10 / 0.0625).powi(2)),
    ];
    for (s, jo) in cases {
        let b = parse(s);
        let g = sweep_grid(&b, &i, "J").unwrap();
        assert!(g.windows(2).all(|w| w[0] < w[1]), "{s}: sorted");
        assert_eq!((g[0], *g.last().unwrap()), J_SWEEP, "{s}: ends");
        assert!(g.iter().any(|&x| ((x - jo) / jo).abs() < 1e-12), "{s}: J_opt {jo} not in {g:?}");
        assert_eq!(g.len(), J_SWEEP_POINTS + 1, "{s}");
    }
    // The mean-field jets have no optimum, so no extra point.
    assert_eq!(sweep_grid(&parse("{\"quench\":\"jets\"}"), &i, "J").unwrap().len(), J_SWEEP_POINTS);
    // Richness: above the burner's own (the cross-plane models refuse it there), up to the soot limit.
    let g = sweep_grid(&parse("{}"), &i, "phi").unwrap();
    assert_eq!(g.len(), PHI_SWEEP_POINTS);
    assert!(g[0] > i.phi && *g.last().unwrap() == PHI_MAX && g.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn the_grid_hits_the_notch_where_the_pdf_model_drops_to_the_well_mixed_value() {
    // At J_opt the segregation width is ~0 and the β-PDF model takes its EXACT well-mixed branch
    // (`pdf_mean_ei` at g ≤ 1e-9) — the notch a sweep that steps over J_opt would miss.
    let (_, _, i) = design_run(&Settings::defaults()).unwrap();
    let b = BurnerSettings::from_json(&Json::parse("{\"quench\":\"jets\",\"closure\":\"pdf\",\"phi_p\":1.5}")).unwrap();
    let jo = b.j_opt().unwrap();
    assert!(sweep_grid(&b, &i, "J").unwrap().contains(&jo));
    let at = ask("burner", "{}", &format!("{{\"quench\":\"jets\",\"closure\":\"pdf\",\"phi_p\":1.5,\"J\":{jo}}}"));
    let off = ask("burner", "{}", "{\"quench\":\"jets\",\"closure\":\"pdf\",\"phi_p\":1.5,\"J\":25}");
    let g = num(at.get("state").unwrap(), "g_seg");
    assert!(g.abs() <= 1e-9, "g at J_opt = {g}");
    let (e_at, e_off) = (num(&at, "ei_thermal"), num(&off, "ei_thermal"));
    assert!(e_at.abs() < 1e-3 && e_off > e_at, "the notch: {e_at} at J_opt vs {e_off} at J 25");
}

// ==========================================================================================
// The readouts beside the burner
// ==========================================================================================

#[test]
fn the_nozzle_readout_is_rung_14s_call_with_the_routes_exhaust_no() {
    let r = slice1("{\"nozzle\":\"convergent\"}");
    let (far, tt3, tt4, pt4) = quad(&r);
    let st9 = r.station("9");
    assert!(r.p9 > 50_000.0 * 1.01, "the convergent nozzle exits above the outside pressure: {}", r.p9);
    let eq = Gas::reacting_equilibrium();
    let base = ZonedNoxOpts { nsteps: 4000, quench_ngrid: 60, quench_nsteps: 2000, ..ZonedNoxOpts::default() };
    let j25 = JetMixing { j: 25.0, h: 0.10, u_c: 75.0, c_e: 0.15, shape_n: 2.0 };
    let ts = Unmixedness { s: 0.0625, c_opt: 2.5, tau_res: 2.5e-3, k_u: 2.5, b_u: 1.0, w_max: 0.7 };
    let cases: [(&str, ZonedNoxOpts, fn(&ZonedNoxState) -> f64); 3] = [
        ("{\"phi_p\":1.5}", base, |z| z.x_no_mix),
        ("{\"phi_p\":1.5,\"quench\":\"jets\"}", ZonedNoxOpts { mixing: Some(j25), ..base }, |z| z.x_no_quenched.unwrap()),
        // A mixing model gives an emission index only: rung 17's κ = x/EI of the bulk quench.
        ("{\"phi_p\":1.5,\"quench\":\"jets\",\"closure\":\"two_stream\"}", ZonedNoxOpts { mixing: Some(j25), unmixedness: Some(ts), ..base },
         |z| z.x_no_quenched.unwrap() / z.ei_no_quenched.unwrap() * z.ei_no_unmixed.unwrap()),
    ];
    for (b, o, x) in cases {
        let j = ask("burner_nozzle", "{\"nozzle\":\"convergent\"}", b);
        assert_eq!(num(&j, "ok"), 1.0, "{b}: {j:?}");
        let z = eq.zoned_nox(far, tt3, tt4, pt4, 1.5, o);
        let xx = x(&z);
        assert_eq!(num(&j, "x_no_exhaust").to_bits(), xx.to_bits(), "{b}: exhaust NO");
        let nf = eq.nozzle_flow(far, tt4, pt4, st9.tt, st9.pt, r.p9, Some(xx));
        for (k, v) in [("t9_frozen", nf.t9_frozen), ("t9_equilibrium", nf.t9_equilibrium), ("v9_frozen", nf.v9_frozen),
                       ("v9_equilibrium", nf.v9_equilibrium), ("x_no_e_exit", nf.x_no_e_exit),
                       ("no_collapse_ratio", nf.no_collapse_ratio), ("max_a", nf.max_a.unwrap())] {
            assert_eq!(num(&j, k).to_bits(), v.to_bits(), "{b}: {k}");
        }
    }
}

#[test]
fn the_quench_path_is_rung_10s_trajectory_for_this_front_zone() {
    let r = slice1("{}");
    let (far, tt3, tt4, pt4) = quad(&r);
    let j = ask("burner_path", "{}", "{\"phi_p\":1.6}");
    assert_eq!(num(&j, "ok"), 1.0, "{j:?}");
    let z = Gas::reacting_equilibrium().zoned_nox(far, tt3, tt4, pt4, 1.6, ZonedNoxOpts::default());
    let comp = turbojet::gas::equilibrium_composition(z.far_primary, z.t_primary, pt4);
    let path = quench_trajectory(&comp, z.t_primary, z.alpha, far, tt3, pt4, 60);
    let Some(Json::List(pts)) = j.get("path") else { panic!("no path") };
    assert_eq!(pts.len(), path.len());
    for (k, (p, q)) in pts.iter().zip(&path).enumerate() {
        let Json::List(v) = p else { panic!() };
        let f = |i: usize| match &v[i] { Json::Float(x) => *x, Json::Int(n) => *n as f64, o => panic!("{o:?}") };
        assert_eq!(f(0), k as f64 / 59.0);
        assert_eq!(f(1).to_bits(), q.t.to_bits(), "point {k}: T");
        assert_eq!(f(2).to_bits(), (far / q.a / f_stoich()).to_bits(), "point {k}: local richness");
    }
    // The path runs from the front zone's richness to the burner's own, crossing stoichiometric.
    let rich = |p: &Json| match p { Json::List(v) => match &v[2] { Json::Float(x) => *x, _ => f64::NAN }, _ => f64::NAN };
    assert!((rich(&pts[0]) - 1.6).abs() < 1e-9 && (rich(&pts[59]) - far / f_stoich()).abs() < 1e-12);
}

// ==========================================================================================
// The crash map's plain words — each entry driven by a request that raises it
// ==========================================================================================

#[test]
fn every_plain_words_entry_is_raised_by_a_request() {
    let generic = explain_burner("something no model raises");
    // (op, design, burner knobs, a fragment of the model's message, a fragment of the plain words)
    let cases: &[(&str, &str, &str, &str, &str)] = &[
        ("burner", "{\"pi_c\":2.2,\"Tt4\":1092,\"T0\":310,\"p0\":101325,\"M0\":0.1}",
         "{\"phi_p\":0.43,\"super_eq_o\":1}", "super-eq O multiplier", "fast-O-atom"),
        ("burner", "{\"pi_c\":34.5,\"Tt4\":2243,\"T0\":288.15,\"p0\":101325,\"M0\":2.5}",
         "{\"phi_p\":0.835}", "NO not trace (", "stops being a trace gas"),
        ("burner", "{\"pi_c\":39.1,\"Tt4\":1993,\"T0\":288.15,\"p0\":101325,\"M0\":2.5}",
         "{\"phi_p\":1.2465,\"super_eq_o\":1,\"prompt\":1,\"prompt_peak\":8.55}", "summed primary NO not trace", "stops being a trace gas"),
        ("burner_path", "{\"pi_c\":23,\"Tt4\":1868,\"T0\":310,\"p0\":101325,\"M0\":2.5}",
         "{\"phi_p\":1.19}", "NO not trace on quench path", "On the way through the dilution"),
        ("burner_path", "{\"pi_c\":36.9,\"Tt4\":2203,\"T0\":310,\"p0\":101325,\"M0\":2.5}",
         "{\"phi_p\":2}", "mixed_out_t: mix temp 3200", "above 3200 K"),
        ("burner_nozzle", "{\"pi_c\":9.23,\"Tt4\":1398,\"T0\":288.15,\"p0\":101325,\"M0\":2.5}",
         "{\"phi_p\":0.457}", "nozzle exit T=", "colder than 500 K"),
        ("burner_nozzle", "{\"pi_c\":8.26,\"Tt4\":2341.6,\"T0\":216.65,\"p0\":22632,\"M0\":0.85}",
         "{\"phi_p\":2}", "equilibrium Newton did not converge", "did not settle"),
        ("burner", "{\"pi_c\":36.47,\"Tt4\":2341.4,\"T0\":310,\"p0\":101325,\"M0\":2.5}",
         "{\"phi_p\":0.99}", "primary_aft: flame temp", "800 to 3200 K"),
        ("burner", "{}", "{\"phi_p\":1.5,\"quench\":\"jets\",\"closure\":\"pdf\",\"J\":62,\"knobs\":{\"k_g\":0.8,\"g_max\":0.8}}",
         "β-PDF shape", "spread is too wide"),
        ("burner", "{\"pi_c\":7.94,\"Tt4\":1150.7,\"T0\":216.65,\"p0\":22632,\"M0\":0.5}",
         "{\"phi_p\":1.014,\"quench\":\"jets\",\"closure\":\"spatial\",\"J\":720,\"H\":0.0904,\"U_c\":59.3,\"C_e\":0.0633,\
          \"shape_n\":3.72,\"knobs\":{\"s\":0.0481,\"k_p\":0.258,\"k_y\":0.179,\"k_z\":0.163}}",
         "field drifted the mean", "cannot spread their air evenly"),
        ("burner", "{\"pi_c\":3.62,\"Tt4\":2344.7,\"T0\":216.65,\"p0\":22632,\"M0\":0.85}",
         "{}", "equilibrium burner balance", "does not run on the equilibrium gas"),
    ];
    for &(op, design, burner, needle, words) in cases {
        let req = format!("{{\"op\":\"{op}\",\"settings\":{design},\"burner\":{burner}}}");
        let msg = match quiet(|| call(&req)) {
            Err(m) => m,
            Ok(reply) => panic!("{needle}: the request ran ({reply:.200})"),
        };
        assert!(msg.contains(needle), "wanted a panic with {needle:?}, got {msg:?}");
        let plain = explain_burner(&msg);
        assert_ne!(plain, generic, "{needle}: no plain words");
        assert!(plain.contains(words), "{needle}: {plain:?}");
        assert!(!plain.contains("   "), "{needle}: a lost line continuation left a run of spaces: {plain:?}");
    }
    // The page asks through `explain` with view "burner".
    let j = Json::parse(&call("{\"op\":\"explain\",\"view\":\"burner\",\"message\":\"super-eq O multiplier m=2.1\"}"));
    assert!(format!("{j:?}").contains("fast-O-atom"), "{j:?}");
}

#[test]
fn a_cross_plane_model_needs_a_front_zone_richer_than_the_burner() {
    // The richness slider's floor is the burner's own richness, which the instant quench takes; the
    // cross-plane models refuse it in their own words, said plainly.
    let phi = num(&ask("burner_inlet", "{}", "{}"), "phi");
    let req = format!("{{\"op\":\"burner\",\"settings\":{{}},\"burner\":{{\"phi_p\":{phi},\"quench\":\"jets\",\"closure\":\"spatial\"}}}}");
    let msg = quiet(|| call(&req)).expect_err("the cross-plane model must refuse a front zone no richer than the burner");
    assert!(msg.contains("must be RICHER than the overall mean"), "{msg}");
    assert!(explain_burner(&msg).contains("richer than the burner as a whole"));
}

#[test]
fn every_burner_request_of_the_browser_check_runs_natively() {
    // The check's native side prints each request's reply; a panic there would stop the whole check.
    let reqs = check_requests();
    assert!(reqs.len() > 40, "{}", reqs.len());
    for r in &reqs {
        let reply = quiet(|| call(r)).unwrap_or_else(|m| panic!("{r}: panicked natively: {m}"));
        let j = Json::parse(&reply);
        match &j {
            Json::List(v) => assert!(v.len() >= J_SWEEP_POINTS.min(PHI_SWEEP_POINTS), "{r}: a short grid"),
            _ => assert_eq!(num(&j, "ok"), 1.0, "{r}: {reply:.200}"),
        }
    }
}

/// The model's own default grids (L0) for a route the page runs on its fixed coarser ones (L1).
fn at_model_defaults(o: ZonedNoxOpts) -> ZonedNoxOpts {
    let d = ZonedNoxOpts::default();
    let mut o = ZonedNoxOpts { quench_ngrid: d.quench_ngrid, quench_nsteps: d.quench_nsteps, nsteps: d.nsteps, ..o };
    if let Some(x) = o.pdf.as_mut() { x.n_bell = MixingPdf::default().n_bell; }
    if let Some(x) = o.pdf_quench.as_mut() { x.n_bell = QuenchPdf::default().n_bell; }
    if let Some(x) = o.transported.as_mut() { x.n_bell = TransportedPdf::default().n_bell; }
    if let Some(x) = o.spatial.as_mut() { let e = SpatialPdf::default(); x.n_bell = e.n_bell; x.ny = e.ny; x.nz = e.nz; }
    o
}

#[test]
fn the_pages_fixed_grids_keep_the_models_numbers_and_its_best_jet() {
    // MEASURED 2026-10-08 (plan § 13.9) at the opening design, φ_p 1.5, the jets' default C_e 0.15, over the
    // J sweep's 15 points: every route within 0.35 % of the model's own grids, every minimum at the same J —
    // the per-pocket three too (5 J each, not gated here: a minute a point at the model's grids). The
    // mean-field jets alone show the same 0.35 %, so it is the quench path's 60 points (against 240), not a
    // mixing model. Bar 1 %, ~3x the worst; and the notch side: the value AT the best jet lies below both
    // neighbours on both grids where the model has its minimum there.
    let r = slice1("{}");
    let (far, tt3, tt4, pt4) = quad(&r);
    let eq = Gas::reacting_equilibrium();
    let base = ZonedNoxOpts { nsteps: 4000, quench_ngrid: 60, quench_nsteps: 2000, ..ZonedNoxOpts::default() };
    let jet = |j: f64| JetMixing { j, h: 0.10, u_c: 75.0, c_e: 0.15, shape_n: 2.0 };
    type Route = (&'static str, fn(f64) -> ZonedNoxOpts, fn(&ZonedNoxState) -> f64, bool);
    let routes: [Route; 4] = [
        ("two_stream", |_| ZonedNoxOpts { unmixedness: Some(Unmixedness { s: 0.0625, c_opt: 2.5, tau_res: 2.5e-3, k_u: 2.5,
                                                                           b_u: 1.0, w_max: 0.7 }), ..ZonedNoxOpts::default() },
         |z| z.ei_no_unmixed.unwrap(), false),
        ("pdf", |_| ZonedNoxOpts { pdf: Some(MixingPdf { s: 0.0625, c_opt: 2.5, k_g: 0.3, g_max: 0.3, n_bell: 80, n_quad: 200 }),
                                   ..ZonedNoxOpts::default() }, |z| z.ei_no_pdf.unwrap(), true),
        ("transported", |_| ZonedNoxOpts { transported: Some(TransportedPdf { s: 0.0625, c_opt: 2.5, c_phi: 2.0, da_opt: 2.0, w_cov: 1.0,
                                                                              tau_mix: 2.5e-3, n_bell: 80, n_quad: 200, n_ode: 400 }),
                                           ..ZonedNoxOpts::default() }, |z| z.ei_no_transported.unwrap(), true),
        ("pdf_quench", |_| ZonedNoxOpts { pdf_quench: Some(QuenchPdf { s: 0.0625, c_opt: 2.5, k_g: 0.3, g_max: 0.3, tau_res: 2.5e-3,
                                                                       b_u: 3.0, n_bell: 80, n_quad: 200 }), ..ZonedNoxOpts::default() },
         |z| z.ei_no_pdf_quench.unwrap(), true),
    ];
    for (name, closure, read, notch) in routes {
        let mut rows = Vec::new();
        for j in [11.709, 16.0, 19.152] {   // the J sweep's neighbours of J_opt = 16, and J_opt itself
            let o = ZonedNoxOpts { mixing: Some(jet(j)), ..closure(j) };
            let l1 = read(&eq.zoned_nox(far, tt3, tt4, pt4, 1.5, merge(base, o)));
            let l0 = read(&eq.zoned_nox(far, tt3, tt4, pt4, 1.5, at_model_defaults(merge(base, o))));
            let rel = ((l1 - l0) / l0).abs();
            assert!(rel < 1e-2, "{name} at J {j}: page grid {l1} vs model grid {l0} ({rel:.2e})");
            rows.push((l1, l0));
        }
        if notch {
            assert!(rows[1].0 < rows[0].0 && rows[1].0 < rows[2].0 && rows[1].1 < rows[0].1 && rows[1].1 < rows[2].1,
                    "{name}: the minimum at J_opt must hold on both grids: {rows:?}");
        }
    }
}

/// `o`'s closure on `base`'s grids and front zone.
fn merge(base: ZonedNoxOpts, o: ZonedNoxOpts) -> ZonedNoxOpts {
    ZonedNoxOpts { mixing: o.mixing, unmixedness: o.unmixedness, pdf: o.pdf, pdf_quench: o.pdf_quench, transported: o.transported,
                   spatial: o.spatial, ..base }
}
