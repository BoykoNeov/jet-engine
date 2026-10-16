//! Where the crash map's slowest requests spend their time: build (ends, tables) vs the march.
use turbojet::sandbox_controls::{build, ControlsSettings};
use turbojet::visuals::Json;
fn main() {
    let reqs = [
        r#"{"govern":1,"hold":"thrust","thrust_by":"epr","table_clean":1,"speed_law":"prop","gov_tau":0.75,"gov_gain":26,"generator":"hp","generator_power":0.075,"hold_cap":0,"metal_on":0,"cool_vane":0,"cool_blade":0,"shape":"press/flow","lever":"bleed","bleed_mode":"valve","redline_on":0,"accel_on":0,"floor_on":0,"from":1300,"to":690,"ramp":0.68,"rho":1.8}"#,
        r#"{"govern":1,"hold":"thrust","thrust_by":"epr","table_clean":0,"speed_law":"prop","gov_tau":0.75,"gov_gain":26,"generator":"hp","generator_power":0.075,"hold_cap":0,"metal_on":0,"cool_vane":0,"cool_blade":0,"shape":"press/flow","lever":"bleed","bleed_mode":"valve","redline_on":0,"accel_on":0,"floor_on":0,"from":1300,"to":690,"ramp":0.68,"rho":1.8}"#,
        r#"{"govern":1,"hold":"thrust","thrust_by":"true","table_clean":1,"speed_law":"prop","gov_tau":0.75,"gov_gain":26,"generator":"hp","generator_power":0.075,"hold_cap":0,"metal_on":0,"cool_vane":0,"cool_blade":0,"shape":"press/flow","lever":"bleed","bleed_mode":"valve","redline_on":0,"accel_on":0,"floor_on":0,"from":1300,"to":690,"ramp":0.68,"rho":1.8}"#,
    ];
    for r in reqs {
        let s = ControlsSettings::from_json(&Json::parse(r)).unwrap();
        let t = std::time::Instant::now();
        let sv = match build(&s) { Ok(v) => v, Err(e) => { println!("refused {:.2}s: {}", t.elapsed().as_secs_f64(), e.0); continue } };
        let tb = t.elapsed().as_secs_f64();
        let (p, m) = sv.march_marked();
        let lean = m.iter().filter(|x| x.clamp == turbojet::governor::Clamp::Lean).count();
        println!("{} by={:?} clean={}: build {tb:.2}s, march {:.2}s ({} pts of {}, lean {lean})", "req", s.thrust_by, s.table_clean,
                 t.elapsed().as_secs_f64() - tb, p.len(), s.expected_points());
    }
}
