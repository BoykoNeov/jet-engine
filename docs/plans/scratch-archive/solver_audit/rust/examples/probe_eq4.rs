// Item 1: at the 200th iteration of a FAILING solve, are the non-floored species converged?
//         Compared against the same solve with the floor lowered to -300 (the unfloored answer).
// Item 2: does any SUCCESSFUL solve ever touch the floor along the way?
use turbojet::gas::{try_equilibrium_composition, EQFLOOR, EQLAST, EQPROBE2, SP_REACT};

fn main() {
    let fs = [1e-14f64, 1e-12, 1e-10, 1e-8, 1e-6, 1e-5, 1e-4, 1e-3, 0.002, 0.005, 0.01, 0.02, 0.03, 0.034, 0.05, 0.06, 0.0675];
    let ps = [3.0e4f64, 1.0e5, 3.0e5, 7.47441e5, 2.0e6];
    let (mut n_ok, mut n_ok_hit, mut n_fail) = (0usize, 0usize, 0usize);
    println!("f\tT\tp\tits\tfree_step@end\tres_free@end\tscale@end\tref_ok\tmax_rel_dev_major\tworst_species");
    for &f in &fs { for &p in &ps {
        let mut t = 300.0f64;
        while t <= 900.0 + 1e-9 {
            EQFLOOR.with(|c| c.set(-80.0));
            let r = try_equilibrium_composition(f, t, p);
            let h = EQPROBE2.with(|c| c.borrow().clone());
            if r.is_ok() {
                n_ok += 1;
                if h.iter().any(|x| x.2) { n_ok_hit += 1; eprintln!("OK-BUT-HIT f={f} T={t} p={p} its={}", h.len()); }
            } else {
                n_fail += 1;
                let y80 = EQLAST.with(|c| c.get());
                let last = *h.last().unwrap();
                EQFLOOR.with(|c| c.set(-300.0));
                let rr = try_equilibrium_composition(f, t, p);
                let yref = EQLAST.with(|c| c.get());
                EQFLOOR.with(|c| c.set(-80.0));
                // relative deviation of every species whose reference amount is > 1e-20 mol
                let (mut dev, mut ws) = (0.0f64, "-");
                if let Ok(_) = rr {
                    for j in 0..8 {
                        if yref[j] > (1e-20f64).ln() {
                            let d = ((y80[j] - yref[j]).exp() - 1.0).abs();
                            if d > dev { dev = d; ws = SP_REACT[j]; }
                        }
                    }
                }
                println!("{f}\t{t}\t{p}\t{}\t{:.3e}\t{:.3e}\t{:.3e}\t{}\t{:.3e}\t{ws}",
                         h.len(), last.0, last.1, last.3, rr.is_ok(), dev);
            }
            t += 20.0;
        }
    }}
    eprintln!("SUMMARY ok={n_ok} ok_but_touched_floor={n_ok_hit} fail={n_fail}");
}
