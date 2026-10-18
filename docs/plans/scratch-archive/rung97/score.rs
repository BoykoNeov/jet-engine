//! Rung 97 scoring (anchor § 3): H1 at the 700 K start end and H2 over the ramp of 700 -> 1400 K / 0.5 tau, new branch
//! vs "the old model" (lp_pinned), sea level +40 K and ISA, M 0.85, four maps; floor 1e-9. Plus the named risks (HP
//! difference at the ends; whether the LP trajectories cross over the whole run) and the reported quantities.
use turbojet::atmosphere::standard;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, DS};
use turbojet::fuel_transient::FuelPoint;

const FLOOR: f64 = 1e-9;

fn main() {
    let names = ["flow/press", "press/flow", "tilted", "flat-lp"];
    for (lab, dt) in [("SL +40 K", 40.0), ("SL ISA", 0.0)] {
        let (t, p) = standard(0.0);
        for shape in 0..4 {
            let s = ControlsSettings { shape, t0: t + dt, p0: p, from: 700.0, to: 1400.0, ..ControlsSettings::defaults() };
            let sv = match build(&s) { Ok(b) => b, Err(e) => { println!("{lab} {:<10}: VACUOUS ({})", names[shape], e.0.chars().take(70).collect::<String>()); continue } };
            let (tc, fl) = (&sv.core.fuel.inner, sv.flight);
            let new_march = sv.march();
            tc.inner.base.lp_pinned.set(true);
            let old_start = tc.try_equilibrium(&fl, 700.0, None).unwrap().0;
            let old_end = tc.try_equilibrium(&fl, 1400.0, None).unwrap().0;
            let (lo, hi) = (old_start.close.f * old_start.close.mdot_air, old_end.close.f * old_end.close.mdot_air);
            let old_march: Vec<FuelPoint> = sv.core.fuel.integrate_fuel(&fl, sv.settings.schedule(lo, hi),
                (old_start.nu_lp, old_start.nu_hp), sv.settings.s_end(), DS, &sv.limiters());
            tc.inner.base.lp_pinned.set(false);
            let n = &sv.start;
            let (dl, dh) = (n.nu_lp - old_start.nu_lp, n.nu_hp - old_start.nu_hp);
            let h1 = if dl.abs() < FLOOR { "VACUOUS" } else if dl < 0.0 { "HIT" } else { "MISS" };
            let m = new_march.len().min(old_march.len());
            let (mut live, mut behind, mut ahead, mut cross, mut sign0, mut max_gap) = (0, 0, 0, 0, 0i32, 0.0f64);
            for k in 0..m {
                let (a, b) = (&new_march[k], &old_march[k]);
                let d = a.nu_lp - b.nu_lp;
                if d.abs() >= FLOOR {
                    let sg = if d < 0.0 { -1 } else { 1 };
                    if sign0 != 0 && sg != sign0 { cross += 1; println!("    crossing at s {:.2}: d {:+.2e}", a.s, d); }
                    sign0 = sg;
                    if a.s > 0.0 && a.s <= sv.settings.ramp + 1e-12 { live += 1; if d < 0.0 { behind += 1 } else { ahead += 1 } }
                }
                if a.s > 0.0 && a.s <= sv.settings.ramp + 1e-12 { max_gap = max_gap.max(d.abs()); }
            }
            let h2 = if live == 0 { "VACUOUS" } else if ahead == 0 { "HIT" } else { "MISS" };
            let unch = |v: &[FuelPoint]| v.iter().filter(|x| x.branch != turbojet::matcher::Branch::Choked).count();
            let pk = |v: &[FuelPoint]| v.iter().map(|x| x.tt4).fold(0.0, f64::max);
            let mn = |v: &[FuelPoint], f: fn(&FuelPoint) -> f64| v.iter().map(f).fold(f64::INFINITY, f64::min);
            println!("{lab} {:<10}: H1 {h1} dnuL {dl:+.3e} | risk dnuH {dh:+.3e} | thrust {:.2}->{:.2} (old) | H2 {h2} ramp pts behind {behind} ahead {ahead} (max |d| {max_gap:.2e}) crossings whole run {cross} | unchoked pts new {} old {} | peak Tt4 {:.1} vs {:.1} | min phiLP {:.4} vs {:.4} phiHP {:.4} vs {:.4} | start phiLP {:.4} vs {:.4} phiHP {:.4} vs {:.4} | pi_LPT {:.4} vs {:.4}",
                names[shape], ControlsSolver::thrust(n), ControlsSolver::thrust(&old_start), unch(&new_march), unch(&old_march),
                pk(&new_march), pk(&old_march), mn(&new_march, |x| x.phi_lp), mn(&old_march, |x| x.phi_lp),
                mn(&new_march, |x| x.phi_hp), mn(&old_march, |x| x.phi_hp), new_march[0].phi_lp, old_march[0].phi_lp,
                new_march[0].phi_hp, old_march[0].phi_hp, n.pi_lpt, old_start.pi_lpt);
        }
    }
}
