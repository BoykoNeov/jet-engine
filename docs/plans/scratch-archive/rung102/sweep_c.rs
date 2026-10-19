// Part C probe: thrust against MAIN fuel at frozen shaft speeds, unlit vs lit (held / matched) — the governor's handle.
use turbojet::afterburner::{self as abm, Afterburner, LpAb, Throat};
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver};
use turbojet::governor as gv;

fn main() {
    let d = ControlsSettings::defaults();
    let sv0 = build(&d).unwrap();
    for &(a, b) in &[(0.738, 0.836), (0.9495, 0.9591), (0.80, 0.83)] {
        println!("== frozen speeds nu_lp {a}, nu_hp {b}");
        for (label, th, w) in [("unlit (fitted)", Throat::Held(1.0), 0.0), ("lit held", Throat::Held(1.0), 1.0),
                               ("lit x1.3", Throat::Held(1.3), 1.0), ("lit matched", Throat::Match, 1.0)] {
            let mut x = LpAb::capture(&sv0.core.fuel.inner.inner.base, Afterburner { tt7: 2000.0, ..Afterburner::default() }, th);
            x.w = w;
            abm::set_lp_ab(Some(x));
            print!("  {label:15}");
            for k in 0..=14 {
                let mf = 0.004 + 0.0025 * k as f64;
                match sv0.core.fuel.try_instant_fuel(&sv0.flight, a, b, mf) {
                    Ok(i) => print!(" {:.4}:{:.0}K/{:.0}K/{:.2}/{:.0}", mf, i.base.tt4, i.base.tt5, i.base.close.mdot_air, ControlsSolver::thrust(&i.base)),
                    Err(e) => print!(" {:.4}:x[{}]", mf, &e.0[..e.0.len().min(28)]),
                }
                if k % 5 == 4 { print!("\n  {:15}", ""); }
            }
            println!();
        }
        abm::set_lp_ab(None);
    }
    let _ = gv::thrust;
}
