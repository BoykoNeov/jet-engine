// Shared probe helpers: variants, the corrected quantities, the flights.
#![allow(dead_code)]
use turbojet::engine::FlightCondition;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face};
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, CustomerMode, GeneratorBy, GeneratorMode,
                                 LeverChoice};
use turbojet::two_spool_transient::Instant2;

pub fn variants() -> Vec<(&'static str, ControlsSettings)> {
    let d = ControlsSettings::defaults();
    vec![
        ("plain", d),
        ("gen HP share .05", ControlsSettings { generator: GeneratorMode::Hp, generator_by: GeneratorBy::Share, ..d }),
        ("gen LP share .05", ControlsSettings { generator: GeneratorMode::Lp, generator_by: GeneratorBy::Share, ..d }),
        ("gen HP power .02", ControlsSettings { generator: GeneratorMode::Hp, ..d }),
        ("cust share .05", ControlsSettings { customer: CustomerMode::Share, ..d }),
        ("cust amount .05", ControlsSettings { customer: CustomerMode::Amount, ..d }),
        ("cool .05/.05", ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..d }),
        ("lever stator", ControlsSettings { lever: LeverChoice::Stator, ..d }),
        ("lever bleed", ControlsSettings { lever: LeverChoice::Bleed, ..d }),
        ("lever both", ControlsSettings { lever: LeverChoice::Both, ..d }),
    ]
}

/// The machine of `s` (built at the design flight — the defaults' flight).
pub fn machine(s: &ControlsSettings) -> ControlsSolver { build(s).unwrap_or_else(|e| panic!("build: {}", e.0)) }

/// Station 2 of a flight on this machine: (Tt2, pt2, V0).
pub fn st2(sv: &ControlsSolver, fl: &FlightCondition) -> (f64, f64, f64) { sv.core.fuel.inner.inlet(fl) }

/// Corrected quantities of a steady point: [thrust/δ, EPR, νL/√θ, νH/√θ, fuel/(δ√θ), face·√θ/δ, (Wf/pt3)/√θ, n_hp].
/// θ, δ against `d2` (the design flight's station 2).
pub fn corrected(sv: &ControlsSolver, fl: &FlightCondition, d2: (f64, f64), i: &Instant2) -> [f64; 8] {
    let (tt2, pt2, _) = st2(sv, fl);
    let (th, de) = (tt2 / d2.0, pt2 / d2.1);
    let fuel = i.close.f * i.close.mdot_air;
    let pt3 = i.close.pt4 / sv.core.fuel.inner.inner.base.pi_b;
    [thrust(i) / de, epr(i, pt2), i.nu_lp / th.sqrt(), i.nu_hp / th.sqrt(), fuel / (de * th.sqrt()),
     face(i) * th.sqrt() / de, fuel / pt3 / th.sqrt(), i.close.n_hp]
}

pub fn choked(i: &Instant2) -> bool { i.branch == Branch::Choked }

pub fn rel(a: f64, b: f64) -> f64 { ((a - b) / b).abs() }

pub const NAMES: [&str; 8] = ["thrust/δ", "EPR", "νL/√θ", "νH/√θ", "fuel/δ√θ", "face√θ/δ", "Wf/pt3/√θ", "n_hp"];
