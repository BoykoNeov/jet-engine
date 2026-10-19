// Write N random Design-view requests (seeded), afterburner OFF or ON.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (seed, n, on): (u64, usize, bool) = (a[1].parse().unwrap(), a[2].parse().unwrap(), a[3] == "on");
    let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut r = || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; (x >> 11) as f64 / (1u64 << 53) as f64 };
    let gases = ["perfect", "thermally_perfect", "reacting", "fork_b", "equilibrium"];
    let nozzles = ["expanded", "exit_pressure", "convergent"];
    for _ in 0..n {
        let g = gases[(r() * 5.0) as usize];
        let nz = nozzles[(r() * 3.0) as usize];
        let (m0, pi_c, tt4) = (0.05 + 2.95 * r(), 2.0 + 38.0 * r(), 900.0 + 1300.0 * r());
        let z = r(); let (t0, p0) = (216.65 + 71.5 * (1.0 - z), 15_000.0 + 86_000.0 * (1.0 - z));
        let mut s = format!(r#"{{"op":"run","settings":{{"gas":"{g}","nozzle":"{nz}","M0":{m0},"pi_c":{pi_c},"Tt4":{tt4},"T0":{t0},"p0":{p0},"p_exit":{}"#, 0.9 * p0);
        if on {
            s += &format!(r#","afterburner":1,"Tt7":{},"eta_ab":{},"pi_fh":{},"M6":{}"#, 1000.0 + 1600.0 * r(), 0.5 + 0.5 * r(), 0.85 + 0.15 * r(), 0.35 * r());
        }
        s += "}}";
        println!("{s}");
    }
}
