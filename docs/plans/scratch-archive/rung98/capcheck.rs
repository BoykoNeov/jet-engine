// Rung 98 (final review): does rung 48's schedule read beyond its rows (where `cap` clamps) in H3's runs? Per run: the
// table's corrected-HP-speed span, how many points read above / below it, and how many of those the schedule held.
use turbojet::sandbox_controls::{controls, ControlsSettings, Holder, CONTROL_SHAPES};
fn main() {
    for shape in 0..4 {
        for (fname, t0) in [("hot", 290.0), ("cold", 216.65)] {
            for ct in [false, true] {
                let s = ControlsSettings { shape, accel_on: true, corr_t: ct, t0, ..ControlsSettings::defaults() };
                let (sv, o) = controls(&s).unwrap();
                let a = sv.accel.as_ref().unwrap();
                let (lo, hi) = (a.n_h[0], a.n_h[a.n_h.len() - 1]);
                let (mut above, mut below, mut held_out) = (0, 0, 0);
                for p in &o.points {
                    let n = sv.core.fuel.try_instant_fuel(&sv.flight, p.nu_lp, p.nu_hp, p.mf).unwrap().base.close.n_hp;
                    let out = n > hi || n < lo;
                    if n > hi { above += 1 } else if n < lo { below += 1 }
                    if out && sv.read(p).unwrap().holder == Holder::Accel { held_out += 1 }
                }
                println!("{:10} {fname:4} θ-corr {ct:5}: rows n_H {lo:.4}–{hi:.4}; points above {above}, below {below}; held by the schedule while outside {held_out}",
                         CONTROL_SHAPES[shape]);
            }
        }
    }
}
