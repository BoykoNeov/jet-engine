// Floor -80 (shipped) vs -300 (fix (b)), lean AND rich, 300-3200 K.
// Checks: (1) every solve that passes at -80 is BIT-identical at -300; (2) iteration headroom at -300;
// (3) what still fails at -300, and how deep the converged trace species sit.
use turbojet::gas::{try_equilibrium_composition, EQFLOOR, EQLAST, EQPROBE2, SP_REACT};

fn run(f: f64, t: f64, p: f64, floor: f64) -> (bool, [f64; 8], usize, Vec<u64>) {
    EQFLOOR.with(|c| c.set(floor));
    let r = try_equilibrium_composition(f, t, p);
    let its = EQPROBE2.with(|c| c.borrow().len());
    let y = EQLAST.with(|c| c.get());
    let bits = match &r { Ok(v) => v.iter().map(|x| x.1.to_bits()).collect(), Err(_) => vec![] };
    (r.is_ok(), y, its, bits)
}

fn main() {
    let fs = [1e-14f64, 1e-10, 1e-6, 1e-4, 1e-3, 0.005, 0.01, 0.02, 0.034, 0.05, 0.0675, 0.08, 0.1, 0.135, 0.2];
    let ps = [3.0e4f64, 1.0e5, 7.47441e5, 2.0e6, 4.0e6];
    let (mut n, mut pass80, mut bitdiff, mut fixed, mut still, mut maxits, mut its_hi) = (0, 0, 0, 0, 0, 0usize, 0);
    let mut min_y = 0.0f64; let mut min_y_at = String::new();
    for &f in &fs { for &p in &ps {
        let mut t = 300.0f64;
        while t <= 3200.0 + 1e-9 {
            n += 1;
            let a = run(f, t, p, -80.0);
            let b = run(f, t, p, -300.0);
            if a.0 { pass80 += 1; if !b.0 || a.3 != b.3 || a.2 != b.2 { bitdiff += 1; println!("BITDIFF f={f} T={t} p={p} its {} vs {}", a.2, b.2); } }
            else if b.0 { fixed += 1; } else { still += 1; println!("STILL-FAILS f={f} T={t} p={p} its={} ymin={:.1} ({})", b.2,
                b.1.iter().cloned().fold(f64::INFINITY, f64::min), SP_REACT[(0..8).min_by(|&i, &j| b.1[i].partial_cmp(&b.1[j]).unwrap()).unwrap()]); }
            if b.0 {
                if b.2 > maxits { maxits = b.2; }
                if b.2 > 160 { its_hi += 1; println!("NEAR-CAP f={f} T={t} p={p} its={}", b.2); }
                for j in 0..8 { if b.1[j] < min_y { min_y = b.1[j]; min_y_at = format!("{} f={f} T={t} p={p}", SP_REACT[j]); } }
            }
            t += 50.0;
        }
    }}
    println!("SUMMARY cases={n} pass@-80={pass80} bitdiff_on_passing={bitdiff} fixed_by_-300={fixed} still_fail={still} max_its@-300={maxits} its>160={its_hi} deepest_y={min_y:.1} at {min_y_at}");
}
