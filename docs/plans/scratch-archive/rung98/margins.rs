// Rung 98 — the gates' measured margins on the SHIPPED code: the pressure gate (tables and a run), the thrust split's
// spread, over all four maps and the share/off variants (the gate tests pin a subset).
use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::TableAt;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, CustomerMode, GeneratorBy, GeneratorMode, LeverChoice,
                                 SpeedLaw, ThrustBy, CONTROL_SHAPES};

fn rel(a: f64, b: f64) -> f64 { ((a - b) / b).abs() }

fn main() {
    let d = ControlsSettings::defaults();
    let variants = [("plain", d), ("cust share", ControlsSettings { customer: CustomerMode::Share, ..d }),
                    ("cool", ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..d }),
                    ("bleed", ControlsSettings { lever: LeverChoice::Bleed, ..d }),
                    ("stator", ControlsSettings { lever: LeverChoice::Stator, ..d }),
                    ("gen HP share", ControlsSettings { generator: GeneratorMode::Hp, generator_by: GeneratorBy::Share, ..d }),
                    ("gen LP share", ControlsSettings { generator: GeneratorMode::Lp, generator_by: GeneratorBy::Share, ..d })];
    let (mut wt, mut wr, mut ws) = (0.0f64, 0.0f64, 0.0f64);
    for shape in 0..4 {
        for (vn, v) in variants {
            // pressure gate, tables: p0 35 kPa
            let s = ControlsSettings { shape, t0: 250.0, p0: 35_000.0, m0: 0.85, govern: true, hold: Hold::Thrust,
                                       thrust_by: ThrustBy::Epr, accel_on: true, table_clean: false, ..v };
            let mut t = 0.0f64;
            if let (Ok(a), Ok(b)) = (build(&s), build(&ControlsSettings { table_at: TableAt::Flight, ..s })) {
                let (ta, tb) = (a.governor.as_ref().unwrap().infer.as_ref().unwrap(), b.governor.as_ref().unwrap().infer.as_ref().unwrap());
                for (x, y) in ta.rows.iter().zip(&tb.rows) {
                    for (p, q) in [(x.thrust, y.thrust), (x.epr, y.epr), (x.nu_lp, y.nu_lp), (x.fuel, y.fuel), (x.face, y.face)] { t = t.max(rel(p, q)); }
                }
                let (aa, ab) = (a.accel.as_ref().unwrap(), b.accel.as_ref().unwrap());
                for k in 0..aa.n_h.len() { t = t.max(rel(aa.n_h[k], ab.n_h[k])).max(rel(aa.kappa[k], ab.kappa[k])); }
                let sp = ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, accel_on: false, ..s };
                if let (Ok(c), Ok(e)) = (build(&sp), build(&ControlsSettings { table_at: TableAt::Flight, ..sp })) {
                    for (x, y) in c.governor.as_ref().unwrap().table.iter().zip(&e.governor.as_ref().unwrap().table) {
                        t = t.max(rel(x.0, y.0)).max(rel(x.1, y.1));
                    }
                }
            }
            // pressure gate, a run
            let g = ControlsSettings { accel_on: false, ..s };
            let mut rr = f64::NAN;
            if let (Ok((_, a)), Ok((_, b))) = (controls(&g), controls(&ControlsSettings { table_at: TableAt::Flight, ..g })) {
                rr = a.points.iter().zip(&b.points).map(|(x, y)| rel(x.tt4, y.tt4).max(rel(x.sp_thrust * x.mdot_air, y.sp_thrust * y.mdot_air)))
                    .fold(0.0f64, f64::max);
                if a.points.len() != b.points.len() { rr = f64::INFINITY; }
            }
            // split spread at Mach 1.4 and hot
            let mut sp = 0.0f64;
            for (t0, m0) in [(290.0, 0.85), (250.0, 1.4)] {
                let q = ControlsSettings { t0, p0: 50_000.0, m0, accel_on: false, ..s };
                if let Ok((sv, o)) = controls(&q) {
                    let (mut lo, mut hi, mut fm) = (f64::INFINITY, f64::NEG_INFINITY, 0.0f64);
                    for p in &o.points {
                        let r = sv.read(p).unwrap();
                        if let Some(gn) = r.gap_nozzle { lo = lo.min(gn); hi = hi.max(gn); fm = fm.max(r.thrust.abs()); }
                    }
                    if fm > 0.0 { sp = sp.max((hi - lo) / fm); }
                }
            }
            println!("{:10} {vn:13}: tables {t:.1e}  run {rr:.1e}  split spread {sp:.1e}", CONTROL_SHAPES[shape]);
            wt = wt.max(t); if rr.is_finite() { wr = wr.max(rr); } ws = ws.max(sp);
        }
    }
    println!("WORST tables {wt:.2e} (bar 1e-9), run {wr:.2e} (bar 1e-7), split spread {ws:.2e} (bar 1e-12)");
}
