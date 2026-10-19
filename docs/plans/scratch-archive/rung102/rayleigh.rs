// Constant-area heat addition (Rayleigh), perfect gas: total-pressure ratio and thermal choke.
fn tt_ratio_star(m: f64, g: f64) -> f64 { (g + 1.0) * m * m * (2.0 + (g - 1.0) * m * m) / (1.0 + g * m * m).powi(2) }
fn pt_ratio_star(m: f64, g: f64) -> f64 {
    (g + 1.0) / (1.0 + g * m * m) * ((2.0 + (g - 1.0) * m * m) / (g + 1.0)).powf(g / (g - 1.0))
}
fn m_from_tt_star(r: f64, g: f64) -> Option<f64> {
    if r > 1.0 { return None; }
    let (mut lo, mut hi) = (1e-9, 1.0);
    for _ in 0..200 { let mid = 0.5 * (lo + hi); if tt_ratio_star(mid, g) < r { lo = mid } else { hi = mid } }
    Some(0.5 * (lo + hi))
}
fn main() {
    let g = 1.3;
    println!("thermal choke Tt7/Tt6 max at entry Mach (gamma {g}):");
    for m in [0.1, 0.15, 0.2, 0.25, 0.3, 0.4] { println!("  M6 {m:.2}: {:.3}", 1.0 / tt_ratio_star(m, g)); }
    println!("Rayleigh pt7/pt6 (heat-addition loss alone):");
    for m in [0.1, 0.15, 0.2, 0.25, 0.3] {
        let r6 = tt_ratio_star(m, g);
        let mut row = format!("  M6 {m:.2}:");
        for tr in [1.2, 1.5, 1.8, 2.0, 2.2] {
            match m_from_tt_star(r6 * tr, g) {
                Some(m7) => row += &format!("  x{tr}: {:.4} (M7 {:.3})", pt_ratio_star(m7, g) / pt_ratio_star(m, g), m7),
                None => row += &format!("  x{tr}: CHOKED"),
            }
        }
        println!("{row}");
    }
}
