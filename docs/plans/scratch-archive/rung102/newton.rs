use turbojet::gas::{f_stoich, try_equilibrium_composition};
fn main() {
    let fs = f_stoich();
    for t in [1300.0, 1500.0, 1700.0, 1900.0, 2100.0] {
        let mut row = format!("T {t}:");
        for phi in [0.4, 0.6, 0.8, 0.9, 0.95, 0.99, 0.999, 0.999999] {
            let ok = try_equilibrium_composition(phi * fs, t, 315_657.0).is_ok();
            row += &format!("  {phi}:{}", if ok { "ok" } else { "FAIL" });
        }
        println!("{row}");
    }
}
