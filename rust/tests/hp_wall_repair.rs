//! **THE HP FACE'S OWN WALL** (2026-10-19). Not a rung — a solver repair (sandbox plan § 36,
//! CLAUDE.md "Solver repairs").
//!
//! Rung 40's two-shaft closure (and its copies for rungs 62/90/91) searches the LP face flow up to
//! the LP map's own wall. On the FLAT LP map there is none, so the search ran to the bare 2.5, where
//! the HP face is far past its map: `ψ_H < 0`, `Tt3` below zero yet finite, and the residual turns
//! back negative. The bracket's sign test then refused a closure whose physical root sat well
//! inside — and rung 40's equilibrium, which the Controls page starts and ends every slam from,
//! refused engines that run: every flat-LP throttle at the design flight below 750 K, every one
//! tried on a standard sea-level day (rung 103's probe; `docs/rung103-spec.md` § "The next seam").
//! Seeding the Newton at the true root did not help — the closure failed there too — so the
//! repair is the search's upper end, not the start.
//!
//! The repair caps the search where the HP face reaches its map's wall (`ψ_H = 0.1`, the floor the
//! LP wall already uses), and ONLY where the old bracket failed — so every closure that solved
//! before is untouched, bit for bit (the oracles hold that).
//!
//! **The gate is independent:** at every flat-LP point the steady matcher solves (rung 39, with
//! rung 103's below-choke switch on — no closure code in common), the repaired equilibrium must
//! land on the same two shaft speeds. Removed (`git stash push -- rust/src`), this file went red
//! (recorded in the commit).

use turbojet::atmosphere::standard;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{build, control_maps, ControlsSettings, CustomerMode, LeverChoice, CONTROL_SHAPES};
use turbojet::two_spool_transient::{take_hp_wall_rescues, TwoSpoolTransientCore};

/// The two solves share no code; measured agreement ≤ 6e-13 over the grid.
const SAME_SPEED: f64 = 1e-10;

fn flights() -> [(&'static str, FlightCondition); 4] {
    let (t, p) = standard(0.0);
    [("SL+40 M0.85", FlightCondition::new(t + 40.0, p, 0.85)),
     ("SL std M0.85", FlightCondition::new(t, p, 0.85)),
     ("SL std M0.30", FlightCondition::new(t, p, 0.30)),
     ("design flight", BladeSettings::defaults().flight())]
}

fn plant(shape: &str) -> TwoSpoolTransientCore {
    let bs = BladeSettings::defaults();
    let (ml, mh) = control_maps(shape).expect("a control shape");
    let tr = TwoSpoolTransientCore::new(bs.design(), bs.flight(), 1.0, ml, mh, 1.0);
    tr.inner.base.below_choke.set(true);
    tr
}

/// Every flat-LP point the steady matcher solves, the equilibrium solves too, at the same speeds —
/// except the one cell whose trouble is the Newton's full-speed START (see the next test).
#[test]
fn flat_lp_equilibrium_lands_on_the_steady_point() {
    let tr = plant("flat-lp");
    take_hp_wall_rescues();
    let mut checked = 0;
    for (name, fl) in flights() {
        let mut tt4 = 550.0;
        while tt4 <= 1500.0 + 1e-9 {
            if let Ok(m) = tr.inner.try_match_point(&fl, tt4) {
                if !(name == "SL std M0.85" && tt4 == 550.0) {
                    let (i, _, _) = tr.try_equilibrium(&fl, tt4, None)
                        .unwrap_or_else(|e| panic!("{name} {tt4} K: steady point exists, equilibrium refused: {}", e.0));
                    let (dl, dh) = (i.nu_lp - m.n_lp_ratio, i.nu_hp - m.n_hp_ratio);
                    assert!(dl.abs() < SAME_SPEED && dh.abs() < SAME_SPEED,
                            "{name} {tt4} K: equilibrium ({}, {}) vs steady ({}, {})", i.nu_lp, i.nu_hp, m.n_lp_ratio, m.n_hp_ratio);
                    checked += 1;
                }
            }
            tt4 += 50.0;
        }
    }
    // 21 of these refused before the repair; the rest were choked points it never touched.
    assert!(checked >= 50, "only {checked} steady points on the grid");
    assert!(take_hp_wall_rescues() > 0, "the repair was never exercised");
}

/// What the repair does NOT reach, stated so it cannot drift silently: the Newton starts at full
/// speed, and at 550 K on a standard sea-level day the compressor exit there is already hotter than
/// 550 K — the burner cannot cool the gas, so the closure has no root AT THE START, while the
/// running point (54 % / 59 % speed, barely above idle) has one. A start problem, still open.
#[test]
fn the_full_speed_start_stays_an_open_seam() {
    let tr = plant("flat-lp");
    let fl = flights()[1].1;
    let m = tr.inner.try_match_point(&fl, 550.0).expect("the steady point exists");
    let e = tr.try_equilibrium(&fl, 550.0, None).expect_err("still refused from full speed");
    assert!(e.0.contains("does not bracket"), "{}", e.0);
    let (i, _, _) = tr.try_equilibrium(&fl, 550.0, Some((m.n_lp_ratio, m.n_hp_ratio))).expect("solves from the running point");
    assert!((i.nu_lp - m.n_lp_ratio).abs() < SAME_SPEED && (i.nu_hp - m.n_hp_ratio).abs() < SAME_SPEED);
}

/// On the three curved LP maps the LP wall binds first over this grid, so the new path is never
/// entered — their every number here is the old code's by construction, not by tolerance. (Not
/// everywhere: at 11 km and high power the HP face still runs off its map — next test.)
#[test]
fn curved_maps_never_enter_the_repair() {
    for shape in CONTROL_SHAPES.iter().filter(|&&s| s != "flat-lp") {
        let tr = plant(shape);
        take_hp_wall_rescues();
        for (_, fl) in flights() {
            let mut tt4 = 550.0;
            while tt4 <= 1500.0 + 1e-9 {
                let _ = tr.try_equilibrium(&fl, tt4, None);
                tt4 += 50.0;
            }
        }
        assert_eq!(take_hp_wall_rescues(), 0, "{shape}");
    }
}

/// The TOP of the range, on a curved map: at 11 km the `flow/press` engine refused above 1690 K
/// (the Controls page's "top of the range" refusal, plan § 27) — the same HP face off its map, at
/// high power. Now it solves to 2000 K, at the steady matcher's speeds.
#[test]
fn the_top_of_the_range_at_altitude_solves() {
    let tr = plant("flow/press");
    let (t, p) = standard(11_000.0);
    let fl = FlightCondition::new(t, p, 0.85);
    take_hp_wall_rescues();
    for tt4 in [1700.0, 1800.0, 1900.0, 2000.0] {
        let m = tr.inner.try_match_point(&fl, tt4).unwrap_or_else(|e| panic!("{tt4} K: steady {}", e.0));
        let (i, _, _) = tr.try_equilibrium(&fl, tt4, None).unwrap_or_else(|e| panic!("{tt4} K: {}", e.0));
        assert!((i.nu_lp - m.n_lp_ratio).abs() < SAME_SPEED && (i.nu_hp - m.n_hp_ratio).abs() < SAME_SPEED,
                "{tt4} K: equilibrium ({}, {}) vs steady ({}, {})", i.nu_lp, i.nu_hp, m.n_lp_ratio, m.n_hp_ratio);
    }
    assert!(take_hp_wall_rescues() > 0, "the repair was never exercised");
}

/// The product's own path: the Controls page on the flat-LP map, through each of the four closures
/// (plain = rung 40, the bleed lever = 62, the customer bleed = 90, turbine cooling = 91). Before the
/// repair 15 of these 16 starts were refused ("no steady operating point").
#[test]
fn the_controls_page_runs_flat_lp_from_low_throttles() {
    let flat = CONTROL_SHAPES.iter().position(|&k| k == "flat-lp").expect("flat-lp");
    let (t_sl, p_sl) = standard(0.0);
    let d = ControlsSettings::defaults();
    for (t0, p0) in [(d.t0, d.p0), (t_sl, p_sl)] {
        for variant in ["plain", "bleed lever", "customer", "cooling"] {
            for from in [600.0, 700.0] {
                let mut s = ControlsSettings::defaults();
                s.shape = flat;
                s.from = from;
                s.to = 1300.0;
                s.t0 = t0;
                s.p0 = p0;
                match variant {
                    "bleed lever" => s.lever = LeverChoice::Bleed,
                    "customer" => s.customer = CustomerMode::Share,
                    "cooling" => { s.cool_vane = 0.03; s.cool_blade = 0.02; }
                    _ => {}
                }
                let sv = build(&s).unwrap_or_else(|e| panic!("{variant} from {from} K at T0={t0}: {}", e.0));
                let n = sv.march().len();
                assert_eq!(n, s.expected_points(), "{variant} from {from} K at T0={t0}: the march stopped early");
            }
        }
    }
}
