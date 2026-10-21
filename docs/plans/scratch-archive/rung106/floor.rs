//! The floor sweep as a scratch binary (FLOOR_HOLD as in floor_probe.rs, plus n1_01: the fan-speed rate law at 0.1).
use turbojet::burner_hold::Hold;
use turbojet::governor_lag::{self as gl, ClipLag, Demand};
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, DS};
fn d() -> ControlsSettings { ControlsSettings::defaults() }
fn gl_on(s: ControlsSettings) -> ControlsSettings { ControlsSettings { govern: true, redline_on: true, gov_lag_on: true, ..s } }
fn main() {
    // The advisor's check of the 0.1 floor: the fastest speed law (TAU_MIN, density-scaled as the page does) at lags
    // 0.1–1.0, at the design flight and a thin and a dense one; step 0.02 against 0.005, flow coefficients.
    use turbojet::sandbox_controls::TAU_MIN;
    let which = std::env::var("FLOOR_HOLD").unwrap_or_default();
    for (name, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
        let base = ControlsSettings { t0, p0, m0, ..d() };
        let tau_min = if which == "rate01" { 0.1 / base.density_ratio().min(1.0) } else { TAU_MIN / base.density_ratio().min(1.0) };
        let hold = if which == "thrust" || which == "n1" || which == "n1_01" { Hold::Thrust } else { Hold::HpSpeed };
        let base = match which.as_str() {
            "n1" | "n1_01" => ControlsSettings { thrust_by: ThrustBy::N1, ..base },
            "prop" => ControlsSettings { speed_law: SpeedLaw::Prop,
                                         gov_gain: turbojet::sandbox_controls::GAIN_MAX / base.density_ratio().max(1.0), ..base },
            _ => base,
        };
        let tau_min = if which == "n1" { turbojet::sandbox_controls::TAU_MIN_LP / base.density_ratio().min(1.0) } else if which == "n1_01" { 0.1 / base.density_ratio().min(1.0) } else { tau_min };
        let mut worst = (0.0f64, 0.0f64);
        let mut row = Vec::new();
        let mut lag = 0.1;
        while lag <= 1.0 + 1e-9 {
            let s = gl_on(ControlsSettings { hold, gov_tau: tau_min, gov_lag: lag, ..base });
            let sv = build(&ControlsSettings { gov_lag_on: false, ..s }).unwrap();
            let g = sv.governor.as_ref().unwrap();
            let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
            let run = |ds: f64| gl::march(&sv.core.fuel, &sv.flight, Demand::Governor(g), ClipLag { tt4_max: s.redline, tau: lag },
                                          &sv.lag_limiters(), None, nu0, s.s_end(), ds).0;
            let (a, b) = (run(DS), run(DS / 4.0));
            let n = a.len().min(b.len() / 4 + 1);
            let dm = (0..n).map(|k| (a[k].phi_lp - b[4 * k].phi_lp).abs().max((a[k].phi_hp - b[4 * k].phi_hp).abs())).fold(0.0, f64::max);
            if dm > worst.0 { worst = (dm, lag); }
            row.push(format!("{lag:.2}:{dm:.1e}{}", if a.len() < s.expected_points() { "(stop)" } else { "" }));
            lag += if which.is_empty() { 0.02 } else { 0.04 };
        }
        println!("[{which}] {name} (density {:.2}, tau_N {tau_min:.3}): worst {:.2e} at lag {:.2}\n  {}", base.density_ratio(), worst.0, worst.1, row.join(" "));
    }
}
