// Against the repaired worktree: does the afterburner path (its own speed search) keep the low-speed wall?
// Fly it at 45 °C sea level, Mach 0.3, throttles across the old/new floor, afterburner none / unlit / lit.
use turbojet::afterburner::AbMode;
use turbojet::sandbox::{explain_fly, fly, FlySettings, GasModel};

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    for ab in [None, Some(AbMode::Unlit), Some(AbMode::Lit)] {
        let mut row = format!("{ab:?}:");
        for tt4 in [680.0, 700.0, 725.0, 750.0, 800.0] {
            let s = FlySettings { gas: GasModel::ThermallyPerfect, t0: 318.15, p0: 101_325.0, m0: 0.3, tt4, ab, ..FlySettings::defaults() };
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fly(&s).map(|_| ())));
            row += &match r {
                Ok(Ok(())) => format!(" {tt4}=runs"),
                Ok(Err(e)) => format!(" {tt4}=precheck({:.300})", format!("{e:?}")),
                Err(e) => { let m = e.downcast_ref::<String>().cloned().unwrap_or_default(); format!(" {tt4}=REFUSED[{:.70}|{:.30}]", m, explain_fly(&m)) }
            };
        }
        println!("{row}");
    }
}
