//! The sandbox's standard atmosphere, held to the PUBLISHED 1976 table
//! (`docs/plans/sandbox-anchor-atmosphere.md`). Every bar here is set by how the table is
//! printed — none was read off this code.

use turbojet::atmosphere::{pressure_altitude, standard, Ambient, Z_MAX, Z_MIN};

/// Table 1 rows: geometric Z (m), T (K, 3 decimals), p (Pa, 5 significant figures).
const TABLE: [(f64, f64, f64); 7] = [
    (0.0, 288.150, 1.0132e5),
    (5_000.0, 255.676, 5.4048e4),
    (10_000.0, 223.252, 2.6500e4),
    (15_000.0, 216.650, 1.2112e4),
    (20_000.0, 216.650, 5.5293e3),
    (25_000.0, 221.552, 2.5492e3),
    (30_000.0, 226.509, 1.1970e3),
];

/// Half a unit in the fifth significant figure of `x`.
fn half_unit_5sf(x: f64) -> f64 { 0.5 * 10f64.powi(x.abs().log10().floor() as i32 - 4) }

#[test]
fn matches_the_published_table() {
    for &(z, t_tab, p_tab) in &TABLE {
        let (t, p) = standard(z);
        assert!((t - t_tab).abs() <= 0.0005, "Z={z}: T {t} vs table {t_tab}");
        assert!((p - p_tab).abs() <= half_unit_5sf(p_tab), "Z={z}: p {p} vs table {p_tab}");
    }
}

#[test]
fn sea_level_is_exact() {
    assert_eq!(standard(0.0), (288.15, 101_325.0));
}

#[test]
fn pressure_altitude_inverts_the_standard_pressure() {
    let mut z = Z_MIN;
    while z <= Z_MAX {
        let back = pressure_altitude(standard(z).1);
        assert!((back - z).abs() <= 1e-6, "Z={z}: round trip gave {back}");
        z += 250.0;
    }
}

#[test]
fn the_connected_knobs_round_trip() {
    for &(z, dt) in &[(0.0, 0.0), (5_574.0, -2.4), (11_000.0, 15.0), (-1_000.0, 30.0), (20_000.0, -10.0)] {
        let a = Ambient::from_altitude(z, dt);
        let b = Ambient::from_ambient(a.t0, a.p0);
        assert!((b.altitude - z).abs() <= 1e-6, "{z}/{dt}: altitude {}", b.altitude);
        assert!((b.delta_t - dt).abs() <= 1e-9, "{z}/{dt}: deviation {}", b.delta_t);
    }
}

#[test]
fn the_panels_flight_point_reads_as_a_cold_day_near_5_6_km() {
    // The anchor's § "What the sandbox opens on": 250 K / 50 kPa is not a standard day.
    let a = Ambient::from_ambient(250.0, 50_000.0);
    assert!((5_500.0..5_700.0).contains(&a.altitude), "pressure altitude {}", a.altitude);
    assert!((-3.0..-1.5).contains(&a.delta_t), "deviation {}", a.delta_t);
}

#[test]
fn layers_are_continuous_at_every_base() {
    // Approach each geopotential base from both sides, in geometric metres.
    for hb in [11_000.0, 20_000.0, 32_000.0] {
        let zb = turbojet::atmosphere::geometric(hb);
        let (tl, pl) = standard(zb - 1e-6);
        let (tr, pr) = standard(zb + 1e-6);
        assert!((tl - tr).abs() < 1e-6 && ((pl - pr) / pl).abs() < 1e-9, "jump at H={hb}");
    }
}
