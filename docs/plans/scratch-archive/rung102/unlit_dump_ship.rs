// Part C gate: the SHIPPED fitted-unlit march (the swapped core) and the unfitted one, every speed to 17 digits.
use turbojet::afterburner::AbMode;
use turbojet::sandbox_controls::{controls, ControlsSettings};

fn main() {
    let d = ControlsSettings::defaults();
    for (lab, s) in [("default", d), ("from600", ControlsSettings { from: 600.0, to: 1400.0, settle: 3.0, ..d })] {
        for (tag, ab) in [("fitted", Some(AbMode::Unlit)), ("none", None)] {
            let (_, o) = controls(&ControlsSettings { ab, ..s }).unwrap();
            for p in o.points {
                println!("{tag} {lab} {:.4} {:.17e} {:.17e} {:?}", p.s, p.nu_lp, p.nu_hp, p.branch);
            }
        }
    }
}
