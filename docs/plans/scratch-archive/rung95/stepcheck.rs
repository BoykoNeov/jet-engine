use turbojet::burner_hold::Hold;
use turbojet::governor as gv;
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES, DS};
fn main() {
    for shape in 0..4 { for (from, to) in [(1000.0, 1400.0), (1400.0, 1000.0)] { for gain in [5.0, 20.0, 50.0] { let tau = gain;
        let s = ControlsSettings { shape, from, to, govern: true, hold: Hold::HpSpeed, speed_law: turbojet::sandbox_controls::SpeedLaw::Prop, gov_gain: gain, ..ControlsSettings::defaults() };
        let sv = build(&s).unwrap();
        let g = sv.governor.as_ref().unwrap();
        let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
        let (a, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), DS);
        let (b, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), DS / 4.0);
        let dm = a.iter().enumerate().filter_map(|(k, p)| b.get(4 * k).map(|q| (p.phi_lp - q.phi_lp).abs().max((p.phi_hp - q.phi_hp).abs()))).fold(0.0, f64::max);
        println!("{:10} {from}->{to} tau {tau}: {}/{} pts, max d phi {dm:.2e}", CONTROL_SHAPES[shape], a.len(), b.len());
    }}}
}
