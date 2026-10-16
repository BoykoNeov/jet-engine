//! Reachability on the LOADED / modified engines (the advisor's blocker). Per config x shape, default slam:
//! (a) EPR and LP acceleration rise with fuel at frozen speeds (violations); (b) equal EPR => equal gross thrust
//! (worst rel diff); (c) the steady table on THIS machine rises in EPR, nu_L, fuel; (d) gross thrust AFFINE in EPR:
//! the residual of interp(gross | EPR) along the open march's points (relative) — the decomposition gate's bar;
//! (e) with the generator, the CLEAN machine's table too (rising?). Nothing a claim is about.
use jet_rung96_probe::*;
use turbojet::sandbox_controls::{build, ControlsSettings, GeneratorMode, LeverChoice, CustomerMode, CONTROL_SHAPES};

fn main() {
    let b = ControlsSettings::defaults();
    let cfgs: Vec<(&str, ControlsSettings)> = vec![
        ("plain", b),
        ("gen HP 0.02", ControlsSettings { generator: GeneratorMode::Hp, ..b }),
        ("gen LP 0.02", ControlsSettings { generator: GeneratorMode::Lp, ..b }),
        ("cooling .05/.03", ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..b }),
        ("customer 0.05", ControlsSettings { customer: CustomerMode::Share, ..b }),
        ("lever bleed", ControlsSettings { lever: LeverChoice::Bleed, ..b }),
        ("lever stator", ControlsSettings { lever: LeverChoice::Stator, ..b }),
    ];
    for (name, c) in cfgs {
        for shape in 0..4 {
            let s = ControlsSettings { shape, ..c };
            let sv = match build(&s) { Ok(v) => v, Err(e) => { println!("{name:16} {:10} BUILD REFUSED: {}", CONTROL_SHAPES[shape], e.0); continue } };
            let pts = sv.march();
            let (mut viol, mut checked) = ([0usize; 2], 0usize);
            let mut samples: Vec<(f64, f64, f64, f64)> = Vec::new();
            for p in pts.iter().step_by(5) {
                let Some((lo, hi)) = range(&sv, p.nu_lp, p.nu_hp, p.mf) else { continue };
                let mut prev: Option<[f64; 2]> = None;
                for k in 0..=40 {
                    let mf = lo * (hi / lo).powf(k as f64 / 40.0);
                    let Ok(i) = inst(&sv, p.nu_lp, p.nu_hp, mf) else { continue };
                    let v = [epr(&sv, &i.base), i.base.phi_lp_dot];
                    if let Some(pv) = prev { for j in 0..2 { if !(v[j] > pv[j]) { viol[j] += 1 } } }
                    prev = Some(v);
                    checked += 1;
                    if k % 10 == 5 { samples.push((p.nu_lp, p.nu_hp, mf, v[0])) }
                }
            }
            let (mut worst, mut pairs) = (0.0f64, 0usize);
            for (x, y) in samples.iter().zip(samples.iter().rev()).take(samples.len() / 2) {
                let ia = inst(&sv, x.0, x.1, x.2).unwrap();
                let (a, bb) = (y.0, y.1);
                let Some((lo, hi)) = range(&sv, a, bb, y.2) else { continue };
                let r = |mf: f64| epr(&sv, &inst(&sv, a, bb, mf).unwrap().base) - x.3;
                if r(lo) > 0.0 || r(hi) < 0.0 { continue }
                let (mut u, mut w) = (lo, hi);
                for _ in 0..200 { if w - u <= 1e-15 * w { break } let m = 0.5 * (u + w); if r(m) < 0.0 { u = m } else { w = m } }
                let ib = inst(&sv, a, bb, 0.5 * (u + w)).unwrap();
                let (ga, gb) = (gross(&sv, &ia.base), gross(&sv, &ib.base));
                worst = worst.max(((ga - gb) / ga).abs());
                pairs += 1;
            }
            let rows = table(&sv, 13);
            let rising = |rows: &[[f64; 6]], j: usize| rows.windows(2).all(|w| w[1][j] > w[0][j]);
            // (d) affine residual along the open march
            let mut aff = 0.0f64;
            for p in &pts {
                let Ok(i) = sv.reread(p) else { continue };
                let e = epr(&sv, &i.base);
                let g = gross(&sv, &i.base);
                let (r0, r1) = (rows[0], rows[rows.len() - 1]);
                let line = r0[4] + (r1[4] - r0[4]) * (e - r0[1]) / (r1[1] - r0[1]);
                aff = aff.max(((g - line) / g).abs());
            }
            // only inside the table's EPR span is the residual an interpolation error; report span coverage
            println!("{name:16} {:10} pts {}: monotone {checked} (non-rising EPR {} / LP accel {}); equal-EPR gross {pairs} pairs worst {worst:.1e}; \
                      table rising EPR {} nuL {} fuel {}; affine residual (march) {aff:.1e}",
                     CONTROL_SHAPES[shape], pts.len(), viol[0], viol[1], rising(&rows, 1), rising(&rows, 2), rising(&rows, 3));
            if s.generator != GeneratorMode::Off {
                let clean = build(&ControlsSettings { generator: GeneratorMode::Off, ..s }).unwrap();
                let cr = table(&clean, 13);
                println!("    clean-machine table rising EPR {} nuL {} fuel {}", rising(&cr, 1), rising(&cr, 2), rising(&cr, 3));
            }
        }
    }
}
