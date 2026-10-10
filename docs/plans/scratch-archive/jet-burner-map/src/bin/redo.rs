//! Re-run failing crash-map rows (a TSV + a message fragment) at the model's own default grids (L0),
//! and at L1: does the failure belong to the sandbox's coarser grid or to the model?
use std::cell::RefCell;
use turbojet::nox::*;
use turbojet::sandbox::Settings;
use turbojet::sandbox_burner::*;
use turbojet::visuals::Json;

thread_local! { static MSG: RefCell<String> = RefCell::new(String::new()); }

fn l0(b: &BurnerSettings) -> ZonedNoxOpts {
    let mut o = b.opts();
    let d = ZonedNoxOpts::default();
    o.quench_ngrid = d.quench_ngrid;
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
    std::panic::set_hook(Box::new(|info| {
        let t = info.payload().downcast_ref::<String>().cloned()
            .or_else(|| info.payload().downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
        MSG.with(|m| *m.borrow_mut() = t);
    }));
    let a: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&a[1]).unwrap();
    let (mut n, mut l0ok, mut l1ok) = (0, 0, 0);
    for line in text.lines().filter(|l| l.contains(a[2].as_str())) {
        let req = Json::parse(line.split('\t').nth(5).unwrap());
        let d = Settings::from_json(req.get("settings").unwrap()).unwrap();
        let b = BurnerSettings::from_json(req.get("burner").unwrap()).unwrap();
        let (engine, _, i) = design_run(&d).unwrap();
        let run = |o: ZonedNoxOpts| std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine.gas.zoned_nox(i.far, i.tt3, i.tt4, i.pt4, b.phi_p, o)
        })).map(|z| headline_thermal(&b, &z)).map_err(|_| MSG.with(|m| m.borrow().chars().take(90).collect::<String>()));
        let r0 = run(l0(&b));
        let r1 = run(b.opts());
        n += 1;
        if r0.is_ok() { l0ok += 1; }
        if r1.is_ok() { l1ok += 1; }
        println!("L0 {:?}\n   L1 {:?}", r0, r1);
    }
    println!("{n} rows: L0 ran {l0ok}, L1 ran {l1ok}");
}
