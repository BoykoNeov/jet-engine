// Solver-tolerance audit: the rung-6 equilibrium Newton at LOW temperature.
// For each (f, T, p): converged? iterations; and on failure the tail of (step, species, y, dy, |F|).
use turbojet::gas::{try_equilibrium_composition, EQPROBE, SP_REACT};

fn main() {
    println!("f\tT\tp\tok\tits\tlast_step\tlast_species\tlast_y\tlast_dy\tlast_resid\tmin_step\tmin_resid");
    for f in [1e-14f64, 1e-12, 1e-10, 1e-8, 1e-7, 1e-6] {
        for p in [3.0e4f64, 1.0e5, 3.0e5, 7.47441e5, 2.0e6] {
            let mut t = 460.0f64;
            while t <= 900.0 + 1e-9 {
                let r = try_equilibrium_composition(f, t, p);
                let h = EQPROBE.with(|c| c.borrow().clone());
                let last = *h.last().unwrap();
                let min_step = h.iter().fold(f64::INFINITY, |a, x| a.min(x.0));
                let min_res = h.iter().fold(f64::INFINITY, |a, x| a.min(x.4));
                println!("{f}\t{t}\t{p}\t{}\t{}\t{:.3e}\t{}\t{:.2}\t{:.3e}\t{:.3e}\t{:.3e}\t{:.3e}",
                         r.is_ok(), h.len(), last.0, SP_REACT[last.1], last.2, last.3, last.4,
                         min_step, min_res);
                t += 10.0;
            }
        }
    }
    // Full history of one failing case, for the record.
    let _ = try_equilibrium_composition(0.01, 440.0, 7.47441e5);
    EQPROBE.with(|c| for (i, x) in c.borrow().iter().enumerate() {
        if i < 25 || i % 20 == 0 || i > 190 {
            eprintln!("it {i}\tstep {:.3e}\t{}\ty {:.3}\tdy {:.3e}\t|F| {:.3e}", x.0, SP_REACT[x.1], x.2, x.3, x.4);
        }
    });
}
