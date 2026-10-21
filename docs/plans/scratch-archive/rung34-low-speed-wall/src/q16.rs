// Round-number Fly-it requests that land on a negative-thrust running point (for the below-idle words' test).
use turbojet::afterburner::AbMode;
use turbojet::atmosphere::standard;
use turbojet::sandbox::{fly, FlySettings, GasModel, MapShape};

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    for (h, m0, tt4, ab) in [(9000.0, 0.6, 340.0, None), (9000.0, 0.6, 380.0, None), (5000.0, 0.7, 400.0, None),
                              (0.0, 0.6, 450.0, None), (9000.0, 1.4, 600.0, None), (0.0, 1.0, 500.0, None),
                              (5000.0, 0.7, 450.0, Some(AbMode::Unlit)), (9000.0, 1.4, 650.0, Some(AbMode::Unlit)), (0.0, 1.0, 550.0, Some(AbMode::Unlit))] {
        for map in MapShape::ALL {
            let (t0, p0) = standard(h);
            let s = FlySettings { gas: GasModel::Perfect, map, t0, p0, m0, tt4, ab, ..FlySettings::defaults() };
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fly(&s).map(|o| o.point.thrust)));
            let txt = match r { Ok(Ok(t)) => format!("runs {t:.0} N"), Ok(Err(e)) => format!("refused: {:.110}", e.0), Err(_) => "PANIC".into() };
            println!("h {h} M0 {m0} Tt4 {tt4} {ab:?} {map:?}: {txt}");
        }
    }
}
