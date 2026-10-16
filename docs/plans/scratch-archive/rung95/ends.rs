//! Givens: the ends' HP and LP speeds and fuel (steady points, existing code) on each shape.
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES};
fn main() {
    for shape in 0..4 {
        let sv = build(&ControlsSettings { shape, ..ControlsSettings::defaults() }).unwrap();
        println!("{:10} start nu_H {:.5} nu_L {:.5} mf {:.6e} | end nu_H {:.5} nu_L {:.5} mf {:.6e} | (nu_H end/start)^2 {:.4}",
                 CONTROL_SHAPES[shape], sv.start.nu_hp, sv.start.nu_lp, sv.mf_lo, sv.end.nu_hp, sv.end.nu_lp, sv.mf_hi,
                 (sv.end.nu_hp / sv.start.nu_hp).powi(2));
    }
}
