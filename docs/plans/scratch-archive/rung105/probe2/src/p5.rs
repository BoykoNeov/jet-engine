// Rung 105 probe P5 — COST on the pages' own entries: Fly it points, its slam (both throttle modes), the Controls slam
// (default and below choke). Built twice: default (the reaction law 0.3) and `--features pin` (the old pin).
use std::time::Instant as T;
use turbojet::sandbox::{fly, FlySettings};
use turbojet::sandbox_transient::{slam, SlamSettings, ThrottleMode};
use turbojet::sandbox_controls::{controls, ControlsSettings};

fn best<R>(n: usize, mut f: impl FnMut() -> R) -> (f64, R) {
    let mut b = f64::INFINITY;
    let mut out = None;
    for _ in 0..n { let t = T::now(); let r = f(); b = b.min(t.elapsed().as_secs_f64()); out = Some(r); }
    (b, out.unwrap())
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let law = if cfg!(feature = "pin") { "PIN" } else { "R0.3" };
    let fd = FlySettings::defaults();
    for (name, s) in [("fly design", fd), ("fly 1100", FlySettings { tt4: 1100.0, ..fd }), ("fly 800", FlySettings { tt4: 800.0, ..fd }),
                      ("fly hot M.3 700", FlySettings { tt4: 700.0, t0: 318.15, p0: 101_325.0, m0: 0.3, ..fd }),
                      ("fly hot M.3 1000", FlySettings { tt4: 1000.0, t0: 318.15, p0: 101_325.0, m0: 0.3, ..fd })] {
        let (t, r) = best(5, || std::panic::catch_unwind(|| fly(&s)).unwrap_or_else(|_| Err(turbojet::sandbox::DoesNotRun("PANIC".into()))));
        match r { Ok(o) => println!("{law} {name:<18} {:8.1} ms  pi_c {:.6} mdot {:.6} nu {:.6} choked {}", 1e3 * t, o.point.pi_c, o.point.mdot_air, o.point.nu, o.point.choked),
                  Err(e) => println!("{law} {name:<18} {:8.1} ms  refused: {}", 1e3 * t, &e.0[..e.0.len().min(90)]) }
    }
    let sd = SlamSettings::defaults();
    for (name, s) in [("slam default", sd), ("slam fuel", SlamSettings { mode: ThrottleMode::Fuel, ..sd }),
                      ("slam low 700->1200", SlamSettings { from: 700.0, to: 1200.0, ..sd })] {
        let (t, r) = best(3, || std::panic::catch_unwind(|| slam(&s)).unwrap_or_else(|_| Err(turbojet::sandbox::DoesNotRun("PANIC".into()))));
        match r { Ok(o) => { let p = o.points.last().unwrap(); println!("{law} {name:<18} {:8.1} ms  points {} end nu {:.6} stop {}", 1e3 * t, o.points.len(), p.nu, o.stop.is_some()) }
                  Err(e) => println!("{law} {name:<18} {:8.1} ms  refused: {}", 1e3 * t, &e.0[..e.0.len().min(90)]) }
    }
    let cd = ControlsSettings::defaults();
    for (name, s) in [("controls default", cd),
                      ("controls unchoked", ControlsSettings { t0: 288.15 + 40.0, p0: 101_325.0, m0: 0.85, from: 700.0, to: 1400.0, ..cd })] {
        turbojet::hp_vane::PROTO_P0.with(|c| c.set(s.p0));
        let (t, r) = best(3, || std::panic::catch_unwind(|| controls(&s)).unwrap_or_else(|_| Err(turbojet::sandbox::DoesNotRun("PANIC".into()))));
        match r { Ok((_, o)) => { let p = o.points.last().unwrap(); println!("{law} {name:<18} {:8.1} ms  points {} end nuL {:.6} nuH {:.6} stop {}", 1e3 * t, o.points.len(), p.nu_lp, p.nu_hp, o.stop.is_some()) }
                  Err(e) => println!("{law} {name:<18} {:8.1} ms  refused: {}", 1e3 * t, &e.0[..e.0.len().min(90)]) }
    }
}
