//! The fixed L1 grid against the model's own defaults (L0), at the jets' default C_e (0.15), on the
//! Design view's default design, phi_p 1.5 (rich, as rungs 12–24 ran). Per model: the J sweep's
//! relative difference and both argmins. Per-pocket models at 5 J only (L0 is ~1 min a point).
use std::sync::Mutex;
use turbojet::nox::*;
use turbojet::sandbox::Settings;
use turbojet::sandbox_burner::*;
use turbojet::visuals::Json;

fn l0(b: &BurnerSettings) -> ZonedNoxOpts {
    let mut o = b.opts();
    let d = ZonedNoxOpts::default();
    o.quench_ngrid = d.quench_ngrid;
    o.quench_nsteps = d.quench_nsteps;
    o.nsteps = d.nsteps;
    if let Some(x) = o.pdf.as_mut() { let e = MixingPdf::default(); x.n_bell = e.n_bell; x.n_quad = e.n_quad; }
    if let Some(x) = o.pdf_quench.as_mut() { let e = QuenchPdf::default(); x.n_bell = e.n_bell; x.n_quad = e.n_quad; }
    if let Some(x) = o.pocket_quench.as_mut() { let e = PocketQuenchPdf::default(); x.n_bell = e.n_bell; x.n_quad = e.n_quad; }
    if let Some(x) = o.transported.as_mut() { let e = TransportedPdf::default(); x.n_bell = e.n_bell; x.n_quad = e.n_quad; }
    if let Some(x) = o.spatial.as_mut() { let e = SpatialPdf::default(); x.n_bell = e.n_bell; x.n_quad = e.n_quad; x.ny = e.ny; x.nz = e.nz; }
    if let Some(x) = o.spatial_dwell.as_mut() { let e = SpatialDwellPdf::default(); x.n_bell = e.n_bell; x.n_quad = e.n_quad; x.ny = e.ny; x.nz = e.nz; x.nt = e.nt; }
    if let Some(x) = o.spatial_local.as_mut() { let e = SpatialLocalPdf::default(); x.n_bell = e.n_bell; x.n_quad = e.n_quad; x.ny = e.ny; x.nz = e.nz; }
    o
}

fn main() {
    let models: Vec<String> = std::env::args().skip(1).collect();
    let d = Settings::defaults();
    let (_, _, i) = design_run(&d).unwrap();
    let mut jobs = Vec::new();
    for m in &models {
        let b = BurnerSettings::from_json(&Json::parse(&format!("{{\"quench\":\"jets\",\"closure\":\"{m}\",\"phi_p\":1.5}}"))).unwrap();
        let mut js = sweep_grid(&b, &i, "J").unwrap();
        if b.closure.cost() == "pocket" {
            let jo = b.j_opt().unwrap();
            js = vec![jo / 9.0, jo / 3.0, jo, jo * 3.0, jo * 9.0];
        }
        for j in js { jobs.push((m.clone(), j)); }
    }
    let out = Mutex::new(Vec::new());
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..12 {
            sc.spawn(|| { let gas = turbojet::gas::Gas::reacting_equilibrium(); loop {
                let k = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if k >= jobs.len() { break; }
                let (m, j) = &jobs[k];
                let b = BurnerSettings::from_json(&Json::parse(&format!("{{\"quench\":\"jets\",\"closure\":\"{m}\",\"phi_p\":1.5,\"J\":{j}}}"))).unwrap();
                let z1 = gas.zoned_nox(i.far, i.tt3, i.tt4, i.pt4, b.phi_p, b.opts());
                let z0 = gas.zoned_nox(i.far, i.tt3, i.tt4, i.pt4, b.phi_p, l0(&b));
                out.lock().unwrap().push((m.clone(), *j, headline_thermal(&b, &z1), headline_thermal(&b, &z0)));
            }});
        }
    });
    let mut out = out.into_inner().unwrap();
    out.sort_by(|a, b| (a.0.clone(), a.1).partial_cmp(&(b.0.clone(), b.1)).unwrap());
    for m in &models {
        let rows: Vec<_> = out.iter().filter(|r| &r.0 == m).collect();
        let worst = rows.iter().map(|r| if r.3 == 0.0 { (r.2 - r.3).abs() } else { ((r.2 - r.3) / r.3).abs() }).fold(0.0, f64::max);
        let am = |f: &dyn Fn(&&(String, f64, f64, f64)) -> f64| rows.iter().min_by(|a, b| f(a).partial_cmp(&f(b)).unwrap()).unwrap().1;
        println!("{m:<14} n={} worst rel {worst:.3e}  argmin L1 J={:.2}  L0 J={:.2}", rows.len(), am(&|r| r.2), am(&|r| r.3));
        for r in &rows { println!("    J {:9.3}  L1 {:.6}  L0 {:.6}", r.1, r.2, r.3); }
    }
}
