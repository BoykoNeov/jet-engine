// Rung 99 final review: the gates' measured margins (the worst value each non-bitwise gate sees), two maps.
use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::{self as ct, TableAt};
use turbojet::inferred_thrust::Reading;
use turbojet::mach_axis::{self as ma, SpeedCorr, Tables};
use turbojet::sandbox_controls::{build, controls, ControlsSettings, CustomerMode, LeverChoice, SpeedLaw, ThrustBy};
fn rel(a: f64, b: f64) -> f64 { ((a - b) / b).abs() }
fn main() {
    let gov = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..ControlsSettings::defaults() };
    let prop = ControlsSettings { govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, accel_on: true, gov_gain: 5.0, ..ControlsSettings::defaults() };
    // Pressure gate per slice.
    let mut worst = (0.0f64, String::new());
    for shape in [0usize, 3] {
        for j in 0..ma::GRID.len() {
            for v in [ControlsSettings::defaults(), ControlsSettings { customer: CustomerMode::Share, ..ControlsSettings::defaults() },
                      ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..ControlsSettings::defaults() },
                      ControlsSettings { lever: LeverChoice::Bleed, ..ControlsSettings::defaults() }] {
                for b in [gov, prop] {
                    let s = ControlsSettings { shape, govern: b.govern, hold: b.hold, thrust_by: b.thrust_by, speed_law: b.speed_law,
                                               accel_on: b.accel_on, gov_gain: b.gov_gain, t0: 250.0, p0: 35_000.0, m0: ma::GRID[j], ..v };
                    let (Ok(d), Ok(f)) = (build(&s), build(&ControlsSettings { table_at: TableAt::Flight, ..s })) else { continue };
                    let (gd, gf) = (d.governor.as_ref().unwrap(), f.governor.as_ref().unwrap());
                    let mut w = 0.0f64;
                    if let (Some(td), Some(tf)) = (gd.infer.as_ref(), gf.infer.as_ref()) {
                        for (a, b) in td.rows.iter().zip(&tf.rows) { for (x, y) in [(a.thrust, b.thrust), (a.epr, b.epr), (a.nu_lp, b.nu_lp), (a.fuel, b.fuel), (a.face, b.face)] { w = w.max(rel(x, y)); } }
                    }
                    for (a, b) in gd.table.iter().zip(&gf.table) { w = w.max(rel(a.0, b.0)).max(rel(a.1, b.1)); }
                    if let (Some(ad), Some(af)) = (d.accel.as_ref(), f.accel.as_ref()) { for k in 0..ad.n_h.len() { w = w.max(rel(ad.n_h[k], af.n_h[k])).max(rel(ad.kappa[k], af.kappa[k])); } }
                    if w > worst.0 { worst = (w, format!("shape {shape} Mach {}", ma::GRID[j])); }
                }
            }
        }
    }
    println!("pressure gate per slice: worst {:.2e} ({})", worst.0, worst.1);
    // Blend gate.
    let mut wb = 0.0f64;
    for shape in [0usize, 3] {
        for m in [0.5, 1.4, 0.15] {
            let s = ControlsSettings { shape, accel_on: true, t0: 250.0, p0: 50_000.0, m0: m, ..gov };
            let sv = build(&s).unwrap();
            let Some(Tables::Grid { a, b: Some(b), w }) = sv.tables else { continue };
            let ft = &sv.core.fuel;
            for r in [Reading::Epr, Reading::LpSpeed] {
                let one = |c: &ct::Carry| c.thrust_table(&ct::try_design_table(ft, c, s.from, s.to).unwrap());
                let (ta, tb) = (one(&a.carry), one(&b.carry));
                let t = ma::blend_thrust(&ta, &tb, w, r);
                let x = |row: &turbojet::inferred_thrust::Row| if r == Reading::Epr { row.epr } else { row.nu_lp };
                let (x0, x1) = (x(&ta.rows[0]).max(x(&tb.rows[0])), x(ta.rows.last().unwrap()).min(x(tb.rows.last().unwrap())));
                for k in 0..=200 { let v = x0 + (x1 - x0) * k as f64 / 200.0; wb = wb.max(rel(t.inferred(r, v), (1.0 - w) * ta.inferred(r, v) + w * tb.inferred(r, v))); }
            }
        }
    }
    println!("blend gate: worst {wb:.2e}");
    // Formula gate.
    let mut wf = 0.0f64;
    for shape in [0usize, 3] {
        let s = ControlsSettings { shape, speed_corr: SpeedCorr::Formula, t0: 250.0, p0: 50_000.0, m0: 1.4, ..gov };
        let (sv, o) = controls(&s).unwrap();
        for p in &o.points { let r = sv.read(p).unwrap(); wf = wf.max(r.gap_nozzle.unwrap().abs() / r.thrust.abs()); }
    }
    println!("formula gate (rest / thrust): worst {wf:.2e}");
}
