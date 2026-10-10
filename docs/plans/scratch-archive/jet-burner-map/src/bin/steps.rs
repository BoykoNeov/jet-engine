//! Per-pocket cost at the default design, ALONE: quench steps 400 vs 2000, C_e 0.15 vs 0.20.
use std::time::Instant;
use turbojet::sandbox::Settings;
use turbojet::sandbox_burner::*;
use turbojet::visuals::Json;

fn main() {
    let d = Settings::defaults();
    let (engine, _, i) = design_run(&d).unwrap();
    for model in ["pocket", "spatial_dwell", "spatial_local", "pdf_quench", "spatial"] {
        for ce in [0.15, 0.20] {
            let b = BurnerSettings::from_json(&Json::parse(&format!(
                "{{\"quench\":\"jets\",\"closure\":\"{model}\",\"phi_p\":1.5,\"J\":25,\"C_e\":{ce}}}"))).unwrap();
            let mut line = format!("{model:<14} C_e {ce}: ");
            for n in [400usize, 2000] {
                let mut o = b.opts();
                o.quench_nsteps = n;
                let t = Instant::now();
                let z = engine.gas.zoned_nox(i.far, i.tt3, i.tt4, i.pt4, b.phi_p, o);
                let ms = t.elapsed().as_secs_f64() * 1e3;
                line += &format!(" n{n}: {:.6} ({ms:.0} ms) ", headline_thermal(&b, &z));
            }
            println!("{line}");
        }
    }
}
