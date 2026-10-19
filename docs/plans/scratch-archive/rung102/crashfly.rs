// Write N random Fly-it requests (seeded) with the afterburner keys randomized; with "off" the same requests with
// the afterburner removed (the base each panic is classified against).
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (seed, n, off): (u64, usize, bool) = (a[1].parse().unwrap(), a[2].parse().unwrap(), a[3] == "off");
    let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut r = || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; (x >> 11) as f64 / (1u64 << 53) as f64 };
    let gases = ["perfect", "thermally_perfect", "reacting", "fork_b", "thermally_perfect", "equilibrium"];
    let maps = ["flow", "pressure", "tilted"];
    let abs = ["none", "unlit", "lit", "lit", "lit"];
    for _ in 0..n {
        let g = gases[(r() * 6.0) as usize];
        let mp = maps[(r() * 3.0) as usize];
        let (pi_c_d, tt4_d) = (4.0 + 26.0 * r(), 1200.0 + 600.0 * r());
        let (eta_ab, pi_fh, m6) = (0.5 + 0.5 * r(), 0.85 + 0.15 * r(), 0.35 * r());
        let (m0, tt4) = (0.05 + 2.2 * r(), 600.0 + 1500.0 * r());
        let z = r(); let (t0, p0) = (216.65 + 71.5 * (1.0 - z), 15_000.0 + 86_000.0 * (1.0 - z));
        let ab = abs[(r() * 5.0) as usize];
        let (tt7, a8, mt) = (1000.0 + 1600.0 * r(), 1.0 + r(), r() < 0.4);
        let mut s = format!(r#"{{"op":"fly","fly":{{"design":{{"pi_c":{pi_c_d},"Tt4":{tt4_d},"eta_ab":{eta_ab},"pi_fh":{pi_fh},"M6":{m6}}},"gas":"{g}","map":"{mp}","M0":{m0},"Tt4":{tt4},"T0":{t0},"p0":{p0}"#);
        if !off {
            s += &format!(r#","ab":"{ab}","Tt7":{tt7},"a8_ratio":{a8},"a8_match":{}"#, mt as i32);
        }
        s += "}}";
        println!("{s}");
    }
}
