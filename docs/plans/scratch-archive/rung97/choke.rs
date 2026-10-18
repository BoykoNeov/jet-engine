//! Is the nozzle choked across Mach (and altitude)? Steady ends + every march point, default move, four maps.
//! (Needs the Mach field of sandbox plan § 28, then held in git stash "sandbox-28-mach-wip".)
use turbojet::atmosphere::standard;
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{controls, ControlsSettings};

fn main() {
    let d = ControlsSettings::defaults();
    for (z, dt) in [(5579.3, -1.92), (0.0, 0.0), (0.0, 40.0), (11000.0, 0.0)] {
        let (t, p) = standard(z);
        for m0 in [0.05, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.85] {
            for shape in [0usize, 3] {
                for (from, to) in [(1000.0, 1400.0), (700.0, 1400.0)] {
                    let s = ControlsSettings { shape, m0, t0: t + dt, p0: p, from, to, ..d };
                    match controls(&s) {
                        Err(e) => println!("z {z:>6} dT {dt:+} M {m0:>4} shape {shape} {from}->{to}: REFUSED {}", e.0.chars().take(120).collect::<String>()),
                        Ok((sv, o)) => {
                            let un = o.points.iter().filter(|x| x.branch != Branch::Choked).count();
                            println!("z {z:>6} dT {dt:+} M {m0:>4} shape {shape} {from}->{to}: start {:?} end {:?}  unchoked points {un}/{}",
                                sv.start.branch, sv.end.branch, o.points.len());
                        }
                    }
                }
            }
        }
    }
}
