// Part C gate: the PROBE's hooked fitted-unlit march (default slam; a below-choke start), every speed to 17 digits.
use turbojet::afterburner::{self as abm, Afterburner, LpAb, Throat};
use turbojet::sandbox_controls::{build, ControlsSettings};

fn main() {
    let d = ControlsSettings::defaults();
    for (lab, s) in [("default", d), ("from600", ControlsSettings { from: 600.0, to: 1400.0, settle: 3.0, ..d })] {
        abm::set_lp_ab(None);
        abm::set_light(None);
        let sv0 = build(&s).unwrap();
        abm::set_lp_ab(Some(LpAb::capture(&sv0.core.fuel.inner.inner.base, Afterburner::default(), Throat::Held(1.0))));
        abm::set_light(Some((99.0, 0.1)));
        let sv = build(&s).unwrap();
        for p in sv.march_marked().0 {
            println!("{lab} {:.4} {:.17e} {:.17e} {:?}", p.s, p.nu_lp, p.nu_hp, p.branch);
        }
    }
}
