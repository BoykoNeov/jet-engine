//! Rung 97 — the unchoked nozzle on two shafts (`docs/rung97-spec.md`; device and bars frozen in
//! `docs/plans/rung97-anchor-unchoked-lp.md` before any code). Below the nozzle's unchoke boundary the LP turbine's
//! expansion is the root that matches the choked `A45` supply to the subsonic nozzle's demand; choked instants keep
//! rung 38's pin bit for bit.

use turbojet::atmosphere::standard;
use turbojet::components::try_choked_mfp;
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, CustomerMode, GeneratorMode, LeverChoice, DS};
use turbojet::two_spool_transient::{EqExit, Instant2, TwoSpoolTransientCore};

/// Sea level, 40 K warmer than standard, Mach 0.85: the 700 K end is unchoked on three maps (anchor G1, `h1_cells.txt`).
fn warm(shape: usize, from: f64) -> ControlsSettings {
    let (t, p) = standard(0.0);
    ControlsSettings { shape, t0: t + 40.0, p0: p, from, to: 1400.0, ..ControlsSettings::defaults() }
}
fn tr(sv: &ControlsSolver) -> &TwoSpoolTransientCore { &sv.core.fuel.inner }
/// The same instant with rung 38's pin forced — "the old model".
fn pinned(sv: &ControlsSolver, i: &Instant2) -> Instant2 {
    let base = &tr(sv).inner.base;
    base.lp_pinned.set(true);
    let p = tr(sv).try_instant(&sv.flight, i.nu_lp, i.nu_hp, i.tt4);
    base.lp_pinned.set(false);
    p.unwrap()
}

#[test]
fn choked_instants_keep_the_pin_bit_for_bit_and_the_switch_restores_it_where_unchoked() {
    // the default move never unchokes (anchor G1): pinned or not, the march is the same, bit for bit
    let sv = build(&ControlsSettings::defaults()).unwrap();
    let free = sv.march();
    tr(&sv).inner.base.lp_pinned.set(true);
    let pin = sv.march();
    tr(&sv).inner.base.lp_pinned.set(false);
    assert_eq!(free.len(), pin.len());
    assert!(free.iter().all(|p| p.branch == Branch::Choked));
    assert!(free.iter().zip(&pin).all(|(a, b)| a.nu_lp.to_bits() == b.nu_lp.to_bits()
        && a.nu_hp.to_bits() == b.nu_hp.to_bits() && a.tt4.to_bits() == b.tt4.to_bits()));
    // an unchoked end: the switch gives rung 38's pin exactly; the branch expands the LP turbine less
    let sv = build(&warm(0, 700.0)).unwrap();
    let i = &sv.start;
    assert_eq!(i.branch, Branch::Subsonic);
    let p = pinned(&sv, i);
    let base = &tr(&sv).inner.base;
    let g = i.close.gas(&tr(&sv).inner);
    let pin = base.try_solve_choked_turbine(g, i.tt45, i.close.f, base.a45, base.a8, base.pi_n, i.eta_lpt).unwrap();
    assert_eq!(p.pi_lpt.to_bits(), pin.0.to_bits());
    assert_eq!((p.pi_hpt.to_bits(), p.tt45.to_bits()), (i.pi_hpt.to_bits(), i.tt45.to_bits()), "the HP turbine is untouched");
    assert!(i.pi_lpt / p.pi_lpt - 1.0 > 1e-3, "{} vs {}", i.pi_lpt, p.pi_lpt);
    assert!(i.phi_lp_dot < p.phi_lp_dot && i.phi_hp_dot.to_bits() == p.phi_hp_dot.to_bits(),
            "the change takes power off the LP shaft only");
}

/// A hand-written perfect-gas solve of the subsonic root — closed-form turbine, sonic throat and subsonic nozzle, its
/// own bisection; it calls neither the helper, `Nozzle`, nor `try_tau_of` (`try_choked_mfp` only for the gas's own
/// constants' sonic flow, checked against the closed form first).
#[test]
fn the_root_matches_a_hand_written_perfect_gas_solve() {
    for shape in [0usize, 1, 2] {
        let sv = build(&warm(shape, 700.0)).unwrap();
        let i = &sv.start;
        let base = &tr(&sv).inner.base;
        let g = i.close.gas(&tr(&sv).inner);
        assert!(g.hot_is_cpg());
        let f = i.close.f;
        let (cp, gam, r) = (g.cp_t_at(i.tt45, f), g.gamma_t(), g.r_t_at(f));
        let ge = (gam - 1.0) / gam;
        let mfp_star = |tt: f64| {
            let ts = cp * tt / (cp + 0.5 * gam * r);
            let ps_over_pt = (ts / tt).powf(1.0 / ge);
            ps_over_pt / (r * ts) * (2.0 * cp * (tt - ts)).sqrt() * tt.sqrt()
        };
        assert!((mfp_star(i.tt45) / try_choked_mfp(g, i.tt45, f).unwrap() - 1.0).abs() < 1e-12);
        let (pt45, p0) = (i.pi_hpt * i.close.pt4, sv.flight.p0);
        let supply = base.a45 * pt45 * mfp_star(i.tt45) / i.tt45.sqrt();
        let resid = |pi: f64| {
            let tt5 = i.tt45 - i.eta_lpt * (i.tt45 - i.tt45 * pi.powf(ge));
            let pt9 = base.pi_n * pi * pt45;
            let m9 = (((pt9 / p0).powf(ge) - 1.0) / (0.5 * (gam - 1.0))).sqrt();
            let t9 = tt5 / (1.0 + 0.5 * (gam - 1.0) * m9 * m9);
            supply - base.a8 * (p0 / (r * t9)) * m9 * (gam * r * t9).sqrt()
        };
        let (mut lo, mut hi) = (0.5, 0.9995);
        for _ in 0..200 { let m = 0.5 * (lo + hi); if resid(m) > 0.0 { lo = m } else { hi = m } }
        let pi = 0.5 * (lo + hi);
        assert!((i.pi_lpt / pi - 1.0).abs() <= 1e-10, "shape {shape}: {} vs {pi}", i.pi_lpt);
    }
}

/// The variants every transient tail and power hook serves (anchor § 2).
fn variants() -> Vec<(&'static str, ControlsSettings)> {
    let w = warm(0, 700.0);
    vec![
        ("plain", w),
        ("generator HP", ControlsSettings { generator: GeneratorMode::Hp, ..w }),
        ("generator LP", ControlsSettings { generator: GeneratorMode::Lp, ..w }),
        ("customer air", ControlsSettings { customer: CustomerMode::Share, ..w }),
        ("cooling", ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..w }),
        ("bleed lever", ControlsSettings { lever: LeverChoice::Bleed, ..w }),
        ("stator lever", ControlsSettings { lever: LeverChoice::Stator, ..w }),
    ]
}

/// Bisect the steady end's throttle onto the unchoke boundary: there the root meets the pin (≤ 1e−9, frozen; over it
/// means that variant's bookkeeping is wrong — investigate, not relax), and the equilibrium's Newton converges on the
/// primary bar on both sides of the kink.
#[test]
fn the_root_meets_the_pin_at_the_unchoke_boundary_on_every_variant() {
    for (name, s) in variants() {
        let sv = build(&s).unwrap();
        let (t, fl) = (tr(&sv), sv.flight);
        let unchoked = |tt4: f64| t.try_equilibrium(&fl, tt4, None).unwrap().0.branch == Branch::Subsonic;
        let (mut lo, mut hi) = (700.0, 1000.0);
        assert!(unchoked(lo) && !unchoked(hi), "{name}: the boundary is not between 700 and 1000 K");
        while hi - lo > 1e-9 { let m = 0.5 * (lo + hi); if unchoked(m) { lo = m } else { hi = m } }
        let i = t.try_equilibrium(&fl, lo, None).unwrap().0;
        let p = pinned(&sv, &i);
        assert!((i.pi_lpt / p.pi_lpt - 1.0).abs() <= 1e-9, "{name}: root {} vs pin {} at {lo} K", i.pi_lpt, p.pi_lpt);
        for tt4 in [lo - 1.0, lo - 1e-3, hi + 1e-3, hi + 1.0] {
            let (_, exit, it) = t.try_equilibrium(&fl, tt4, None).unwrap();
            assert!(exit == EqExit::Primary && it < 80, "{name}: at {tt4} K the Newton ended {exit:?} after {it}");
        }
    }
}

/// A march held at an unchoked steady end's fuel stays on it (≤ 1e−9 in both speeds after 2 τ), on every variant: a
/// tail that branched without its power hook would drift.
#[test]
fn a_march_from_an_unchoked_steady_end_stays_there_on_every_variant() {
    for (name, s) in variants() {
        let sv = build(&s).unwrap();
        assert_eq!(sv.start.branch, Branch::Subsonic, "{name}");
        let mf = sv.mf_lo;
        let pts = sv.core.fuel.integrate_fuel(&sv.flight, move |_| mf, (sv.start.nu_lp, sv.start.nu_hp), 2.0, DS,
                                              &sv.limiters());
        let l = pts.last().unwrap();
        assert!((l.s - 2.0).abs() < 1e-9, "{name}: the march stopped at {}", l.s);
        assert!((l.nu_lp - sv.start.nu_lp).abs() <= 1e-9 && (l.nu_hp - sv.start.nu_hp).abs() <= 1e-9,
                "{name}: drifted by {:.2e} / {:.2e}", l.nu_lp - sv.start.nu_lp, l.nu_hp - sv.start.nu_hp);
    }
}

/// Mass conservation at the nozzle, at every unchoked point of the move on every variant: `A8·ρ9·V9` at the tail's own
/// `pt5` equals the gas the engine delivers (burner gas, plus the cooling air when cooled) — a quantity the helper never
/// reads. Measured ≤ 7.6e−14 (`gate_margins` § of the spec); bar 1e−11. Added beyond the anchor: the boundary gate cannot
/// see a wrong `pt45` (it cancels from the pin and the root alike), this one can. On rung 38's pin it fails.
#[test]
fn the_unchoked_nozzle_passes_exactly_the_gas_the_engine_delivers_on_every_variant() {
    for (name, s) in variants() {
        let sv = build(&s).unwrap();
        let (t, fl) = (tr(&sv), sv.flight);
        let base = &t.inner.base;
        let mut n = 0;
        for pt in sv.march().iter().filter(|x| x.branch != Branch::Choked) {
            let i = sv.core.fuel.try_instant_fuel(&fl, pt.nu_lp, pt.nu_hp, pt.mf).unwrap().base;
            let c = &i.close;
            let g = c.gas(&t.inner);
            let hpc = (1.0 - c.bleed.unwrap_or(0.0)) * c.mdot_face.unwrap_or(c.mdot_air);
            let (cool, far) = match c.cooling {
                Some(cl) => (cl.total() * hpc, c.f * c.mdot_air / (c.mdot_air + cl.total() * hpc)),
                None => (0.0, c.f),
            };
            let s5 = turbojet::gas::FlowState { tt: i.tt5, pt: i.pi_lpt * i.pi_hpt * c.pt4, mdot: 1.0, far };
            let e = turbojet::components::Nozzle::convergent(fl.p0, base.pi_n).try_apply(&s5, g).unwrap();
            let flux = base.a8 * e.p9 / (g.r_t_at(far) * e.t9) * e.v9;
            let rel = flux / (c.mdot_air * (1.0 + c.f) + cool) - 1.0;
            assert!(rel.abs() <= 1e-11, "{name}: the nozzle passes {rel:+.2e} of the engine's gas at s = {}", pt.s);
            n += 1;
        }
        assert!(n >= 5, "{name}: only {n} unchoked points");
    }
}
