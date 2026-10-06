//! The 1976 US Standard Atmosphere — the sandbox's flight knobs (`docs/plans/sandbox-plan.md`).
//!
//! NOT a rung: no cycle number moves. It exists so the sandbox can offer altitude + deviation from
//! the standard day, wired both ways to raw ambient `T0`/`p0` (the user's choice, § 9.1). It is
//! still NEW PHYSICS, so it is derived and anchored before it is coded:
//! `docs/plans/sandbox-anchor-atmosphere.md`, held to the published Table 1 by
//! `tests/atmosphere.rs`.
//!
//! **Derivation (the anchor's § Derivation, in one line each).**
//! - Hydrostatics + ideal gas: `dp/p = −(g0 M0/R*) dH/T` — in GEOPOTENTIAL `H`, which folds the
//!   fall of `g` with height into the coordinate, so `g0` is a constant.
//! - `T` is piecewise linear in `H`; integrating across a layer gives a power law (`L ≠ 0`) or an
//!   exponential (`L = 0`), and each layer's base pressure is the one below evaluated at its top —
//!   chained from 101 325 Pa, so no base pressure is typed in.
//! - `H = r0 Z/(r0 + Z)` converts the GEOMETRIC altitude `Z` the knob and the table use.
//! - Pressure altitude inverts the layer law in closed form.

use crate::gas::powp;

/// Standard gravity, m/s².
pub const G0: f64 = 9.80665;
/// The 1976 universal gas constant, J/(kmol·K) — its own value, not CODATA's.
pub const R_STAR: f64 = 8314.32;
/// Sea-level mean molecular weight, kg/kmol.
pub const M0: f64 = 28.9644;
/// Effective Earth radius for the geopotential conversion, m.
pub const R0: f64 = 6_356_766.0;
/// Sea-level temperature, K, and pressure, Pa.
pub const T_SL: f64 = 288.15;
pub const P_SL: f64 = 101_325.0;

/// `(H_b, L_b)` — geopotential base altitude (m) and lapse rate (K/m) of each layer used.
pub const LAYERS: [(f64, f64); 5] =
    [(0.0, -0.0065), (11_000.0, 0.0), (20_000.0, 0.001), (32_000.0, 0.0028), (47_000.0, 0.0)];
/// Top of the last layer used, geopotential m.
pub const H_TOP: f64 = 51_000.0;
/// The sandbox's geometric range, m: the table's lower edge to the stratopause.
pub const Z_MIN: f64 = -5_000.0;
pub const Z_MAX: f64 = 47_000.0;

/// `g0 M0 / R*`, K/m — the hydrostatic constant every layer law carries.
fn k() -> f64 { G0 * M0 / R_STAR }

/// Geometric → geopotential altitude, m.
pub fn geopotential(z: f64) -> f64 { R0 * z / (R0 + z) }

/// Geopotential → geometric altitude, m.
pub fn geometric(h: f64) -> f64 { R0 * h / (R0 - h) }

/// `(T_b, p_b)` at each layer's base, chained upward from sea level.
fn bases() -> [(f64, f64); 5] {
    let mut out = [(T_SL, P_SL); 5];
    for b in 1..LAYERS.len() {
        let (h0, l) = LAYERS[b - 1];
        let (t0, p0) = out[b - 1];
        out[b] = layer(h0, l, t0, p0, LAYERS[b].0);
    }
    out
}

/// One layer's law from its base `(h0, t0, p0)` up (or down) to `h`.
fn layer(h0: f64, l: f64, t0: f64, p0: f64, h: f64) -> (f64, f64) {
    if l == 0.0 {
        (t0, p0 * (-k() * (h - h0) / t0).exp())
    } else {
        let t = t0 + l * (h - h0);
        (t, p0 * powp(t0 / t, k() / l))
    }
}

/// Index of the layer holding geopotential `h` (below 0 the first layer extends down).
fn layer_of(h: f64) -> usize {
    (1..LAYERS.len()).rev().find(|&b| h >= LAYERS[b].0).unwrap_or(0)
}

/// Standard-day static `(T, p)` at GEOMETRIC altitude `z`, K and Pa.
pub fn standard(z: f64) -> (f64, f64) {
    assert!((Z_MIN..=Z_MAX).contains(&z), "altitude {z} m is outside the standard atmosphere used ({Z_MIN}..{Z_MAX} m)");
    let h = geopotential(z);
    let b = layer_of(h);
    let (tb, pb) = bases()[b];
    layer(LAYERS[b].0, LAYERS[b].1, tb, pb, h)
}

/// The GEOMETRIC pressure altitude of static pressure `p`, m — the inverse of [`standard`]'s `p`.
pub fn pressure_altitude(p: f64) -> f64 {
    let (_, p_lo) = standard(Z_MAX);
    let (_, p_hi) = standard(Z_MIN);
    assert!(p >= p_lo && p <= p_hi, "pressure {p} Pa is outside the standard atmosphere used ({p_lo:.1}..{p_hi:.0} Pa)");
    let bs = bases();
    // The layer whose base pressure is the first at or below... i.e. the highest base still >= p.
    let b = (1..LAYERS.len()).rev().find(|&b| p <= bs[b].1).unwrap_or(0);
    let (hb, l) = LAYERS[b];
    let (tb, pb) = bs[b];
    let h = if l == 0.0 {
        hb + (tb / k()) * (pb / p).ln()
    } else {
        let t = tb * powp(pb / p, l / k());
        hb + (t - tb) / l
    };
    geometric(h)
}

/// The four connected flight knobs: geometric altitude (m), deviation from the standard day (K),
/// and the ambient static `T0` (K) and `p0` (Pa) the model actually runs on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ambient {
    pub altitude: f64,
    pub delta_t: f64,
    pub t0: f64,
    pub p0: f64,
}

impl Ambient {
    /// Altitude + deviation ⇒ `T0 = T_std + ΔT`, `p0 = p_std`.
    pub fn from_altitude(altitude: f64, delta_t: f64) -> Self {
        let (t, p) = standard(altitude);
        Ambient { altitude, delta_t, t0: t + delta_t, p0: p }
    }

    /// Raw `T0`/`p0` ⇒ the pressure altitude and the deviation that reproduce them.
    pub fn from_ambient(t0: f64, p0: f64) -> Self {
        let altitude = pressure_altitude(p0);
        Ambient { altitude, delta_t: t0 - standard(altitude).0, t0, p0 }
    }
}
