//! Slice 5 measurements — the combustor diagnostics (rungs 7–24), the same calls natively and in
//! the browser build. Not in the repo.
use turbojet::engine::build_turbojet;
use turbojet::nox::*;
use turbojet::gas::Gas;
use turbojet::sandbox::{GasModel, Settings};

/// The burner inlet the diagnostics read: (Tt3, Tt4, far, pt4) off a design run on `gas`.
pub fn burner_inlet(gas: usize, pi_c: f64, tt4: f64) -> (f64, f64, f64, f64) {
    let mut s = Settings::defaults();
    s.gas = GasModel::ALL[gas];
    s.pi_c = pi_c;
    s.tt4 = tt4;
    let r = build_turbojet(s.gas.gas(), s.pi_c, s.tt4, s.p0, s.losses()).run(&s.flight(), s.mdot);
    let (s3, s4) = (r.station("3"), r.station("4"));
    (s3.tt, s4.tt, s4.far, s4.pt)
}

pub const CLOSURES: [&str; 12] = [
    "ideal", "ideal+O+prompt", "tau_q 1ms", "jet(11)", "two-stream(12)", "pdf(13)", "pdf_quench(15)",
    "pocket(16)", "transported(18)", "spatial(22)", "spatial_dwell(23)", "spatial_local(24)",
];

/// One zoned_nox call. `level`: 0 = the model's own defaults, 1 = the charts page's grids,
/// 2 = coarser. Returns (headline EI, t_mix).
pub fn zn(inlet: (f64, f64, f64, f64), closure: usize, phi: f64, j: f64, level: usize) -> (f64, f64) {
    let (tt3, tt4, far, p) = inlet;
    let eq = Gas::reacting_equilibrium();
    let (qg, qn, nb, nq, nyz, nt, zs) = match level {
        0 => (240, 2000, 0, 0, 0, 0, 4000),
        1 => (60, 400, 80, 160, 32, 24, 4000),
        _ => (32, 200, 40, 80, 24, 16, 1000),
    };
    let lv = |dflt: usize, coarse: usize| if level == 0 { dflt } else { coarse };
    let mut o = ZonedNoxOpts { tau: 3e-3, nsteps: zs, ..ZonedNoxOpts::default() };
    if level > 0 {
        o.quench_ngrid = qg;
        o.quench_nsteps = qn;
    }
    let mix = JetMixing { j, c_e: 0.20, shape_n: 2.0, ..JetMixing::default() };
    match closure {
        0 => {}
        1 => { o.super_eq_o = true; o.prompt = Some(PromptNo::default()); }
        2 => o.tau_q = Some(1e-3),
        3 => o.mixing = Some(mix),
        4 => { o.mixing = Some(mix); o.unmixedness = Some(Unmixedness::default()); }
        5 => { let d = MixingPdf::default(); o.mixing = Some(mix);
               o.pdf = Some(MixingPdf { n_bell: lv(d.n_bell, nb), n_quad: lv(d.n_quad, nq), ..d }); }
        6 => { let d = QuenchPdf::default(); o.mixing = Some(mix);
               o.pdf_quench = Some(QuenchPdf { n_bell: lv(d.n_bell, nb), n_quad: lv(d.n_quad, nq), ..d }); }
        7 => { let d = PocketQuenchPdf::default(); o.mixing = Some(mix);
               o.pocket_quench = Some(PocketQuenchPdf { n_bell: lv(d.n_bell, nb / 2), n_quad: lv(d.n_quad, nq), ..d }); }
        8 => { let d = TransportedPdf::default(); o.mixing = Some(mix);
               o.transported = Some(TransportedPdf { n_bell: lv(d.n_bell, nb), n_quad: lv(d.n_quad, nq), ..d }); }
        9 => { let d = SpatialPdf::default(); o.mixing = Some(mix);
               o.spatial = Some(SpatialPdf { n_bell: lv(d.n_bell, nb), n_quad: lv(d.n_quad, nq),
                                             ny: lv(d.ny, nyz), nz: lv(d.nz, nyz), ..d }); }
        10 => { let d = SpatialDwellPdf::default(); o.mixing = Some(mix);
                o.spatial_dwell = Some(SpatialDwellPdf { n_bell: lv(d.n_bell, nb / 2), n_quad: lv(d.n_quad, nq),
                                                         ny: lv(d.ny, nyz), nz: lv(d.nz, nyz), nt: lv(d.nt, nt), ..d }); }
        _ => { let d = SpatialLocalPdf::default(); o.mixing = Some(mix);
               o.spatial_local = Some(SpatialLocalPdf { n_bell: lv(d.n_bell, nb / 2), n_quad: lv(d.n_quad, nq),
                                                        ny: lv(d.ny, nyz), nz: lv(d.nz, nyz), ..d }); }
    }
    let z = eq.zoned_nox(far, tt3, tt4, p, phi, o);
    let ei = match closure {
        0 | 1 => z.ei_no_total(),
        2 | 3 => z.ei_no_quenched.unwrap(),
        4 => z.ei_no_unmixed.unwrap(),
        5 => z.ei_no_pdf.unwrap(),
        6 => z.ei_no_pdf_quench.unwrap(),
        7 => z.ei_no_pocket_quench.unwrap(),
        8 => z.ei_no_transported.unwrap(),
        9 => z.ei_no_spatial.unwrap(),
        10 => z.ei_no_spatial_dwell.unwrap(),
        _ => z.ei_no_spatial_local.unwrap(),
    };
    (ei, z.t_mix)
}

// ---- browser exports (the default design's inlet is cached per gas/pi_c/tt4 by the caller) ----
#[no_mangle]
pub extern "C" fn inlet_part(gas: u32, pi_c: f64, tt4: f64, which: u32) -> f64 {
    let i = burner_inlet(gas as usize, pi_c, tt4);
    [i.0, i.1, i.2, i.3][which as usize]
}

#[no_mangle]
pub extern "C" fn zn_ei(tt3: f64, tt4: f64, far: f64, p: f64, closure: u32, phi: f64, j: f64, level: u32) -> f64 {
    zn((tt3, tt4, far, p), closure as usize, phi, j, level as usize).0
}
