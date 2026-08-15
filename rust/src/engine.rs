//! Assemble components into an engine, solve the shaft balance, score performance.
//!
//! Port of `turbojet/engine.py`'s DESIGN POINT (phase 2 of `docs/plans/todo-rust-port.md`).
//! The off-design and transient matcher ladders — rungs 31 to 84, which is the bulk of the
//! Python file — arrive in phases 5 to 7 and leave this design run untouched, exactly as they
//! do in the Python.
//!
//! RUNG 2 — the engine owns the dual-cp, mechanical-efficiency shaft balance and the
//! pressure-thrust term, and reports TWO thermal efficiencies (`docs/rung2-spec.md`
//! § Performance). The shaft coupling is solved EXPLICITLY here: the engine computes the
//! compressor's work and hands the turbine a `delta_h` at call time.
//!
//! RUNG 4 — the two hot-section reads the engine still owns (the shaft-closure enthalpies and
//! the pressure-thrust `R_t`) pass the burned-gas fuel/air ratio f, so a reacting gas uses the
//! products composition; a CPG/frozen-TPG gas ignores it.

use crate::components::{Burner, Component, Compressor, Inlet, Nozzle, Turbine};
use crate::gas::{powp, Abort, FlowState, Gas};

/// Freestream / flight inputs (station 0).
#[derive(Clone, Copy, Debug)]
pub struct FlightCondition {
    /// Ambient STATIC temperature, K.
    pub t0: f64,
    /// Ambient STATIC pressure, Pa.
    pub p0: f64,
    /// Flight Mach number.
    pub m0: f64,
}

impl FlightCondition {
    pub fn new(t0: f64, p0: f64, m0: f64) -> Self { FlightCondition { t0, p0, m0 } }
}

/// Top-level cycle outputs (`docs/rung2-spec.md` § Performance).
///
/// TWO thermal efficiencies are reported because rung 2 splits a definitional knot the rung-1
/// notes had flagged:
/// - `eta_brayton = 1 - Tt2/Tt3`: the cold-Brayton identity (`= 1 - 1/pi_c^gc`). This is the
///   rung-1 number (0.4821) and the primary hand-check. Once the legs tilt it is no longer the
///   true thermal efficiency — kept for the hand-check and table continuity.
/// - `eta_thermal = [(1+f)V9² - V0²]/(2 f hPR)`: the REAL thermal efficiency (kinetic energy
///   added to the jet per unit fuel power). Anchors Mattingly's `eta_T`; = 0.5477 in the ideal
///   limit. Under THIS definition the textbook cascade `eta_o = eta_thermal * eta_p` holds
///   exactly — which is why it is a free consistency check rather than a claim.
#[derive(Clone, Copy, Debug)]
pub struct Performance {
    /// `F / mdot`, N·s/kg.
    pub specific_thrust: f64,
    /// kg/(N·s).
    pub tsfc: f64,
    /// `1 - Tt2/Tt3` — the Brayton identity, the rung-1 hand-check.
    pub eta_brayton: f64,
    /// KE per unit fuel power — the real thermal efficiency.
    pub eta_thermal: f64,
    pub eta_propulsive: f64,
    pub eta_overall: f64,
}

/// Everything one run produces: the station table plus performance.
///
/// Station states are totals only. The nozzle-exit STATIC quantities (`m9`, `t9`, `v9`, `p9`)
/// and the flight velocity `v0` are surfaced here because they are not total quantities and so
/// do not live on a [`FlowState`].
///
/// The table is an ordered `Vec` rather than a map: Python's dict is insertion-ordered and at
/// least one test walks it, so the order is part of the interface, not an implementation
/// detail.
#[derive(Clone, Debug)]
pub struct EngineResult {
    /// Keyed "0", "2", "3", "4", "5", "9", in flow order.
    pub stations: Vec<(&'static str, FlowState)>,
    pub performance: Performance,
    /// Flight velocity, m/s.
    pub v0: f64,
    /// Exhaust velocity, m/s.
    pub v9: f64,
    /// Exhaust Mach number.
    pub m9: f64,
    /// Exhaust static temperature, K.
    pub t9: f64,
    /// Exhaust static pressure, Pa (`== p0` when fully expanded).
    pub p9: f64,
}

impl EngineResult {
    /// Python's `result.stations["3"]`. Panics on an unknown label — a typo'd station is a
    /// programming error, and a silent `None` would let a test assert nothing.
    pub fn station(&self, label: &str) -> &FlowState {
        self.stations.iter().find(|&&(l, _)| l == label)
            .map(|(_, s)| s)
            .unwrap_or_else(|| panic!("no station {label:?} in the table"))
    }
}

/// An ordered list of components that transform a [`FlowState`] 0 -> 9.
///
/// [`run`](Self::run) chains the components and owns the shaft balance: it computes the
/// turbine's required enthalpy drop from the compressor/inlet states it already holds and
/// passes it in. Performance scoring uses the resulting station table plus the freestream/exit
/// velocities and the exit pressure.
///
/// `Clone` because rung 53's `at_setting` REBUILDS a sibling matcher from the design engine, as
/// Python's does — see [`crate::stator::VariableStatorCore::at_setting`]. Nothing here is
/// mutated, so a clone is a deep copy of read-only design data.
#[derive(Clone)]
pub struct Engine {
    pub gas: Gas,
    /// Ordered `(station_label, component)` pairs.
    pub components: Vec<(&'static str, Component)>,
    /// Shaft mechanical efficiency, <= 1.
    pub eta_m: f64,
}

impl Engine {
    pub fn new(gas: Gas, components: Vec<(&'static str, Component)>, eta_m: f64) -> Self {
        Engine { gas, components, eta_m }
    }

    /// Station 0: freestream totals + flight velocity `V0`.
    ///
    /// COLD-section properties (the freestream is fresh air). Station 0 is ONE of the two
    /// velocity<->enthalpy coupling stations — the other is the nozzle — so it is one of the
    /// only two places the rounded-R trap forces a CPG/TPG branch:
    ///
    /// ```text
    ///   CPG (bit-for-bit rung 1; gamma-only, so the rounded R never enters):
    /// Tt0 = T0 * (1 + (gamma_c-1)/2 * M0^2)
    /// pt0 = p0 * (1 + (gamma_c-1)/2 * M0^2) ** (1/gc)
    /// V0  = M0 * sqrt(gamma_c * R_c * T0)
    ///   TPG (variable cp): stagnation ENTHALPY + the pr ratio set the totals:
    /// V0  = M0 * sqrt(gamma_c(T0) * R_c * T0)
    /// Tt0 = T_from_h_c( h_c(T0) + V0^2/2 )
    /// pt0 = p0 * pr_c(Tt0)/pr_c(T0)
    /// ```
    ///
    /// Physical justification: a TOTAL quantity is what the gas reaches if brought to rest
    /// isentropically, so it already folds in the flow's kinetic energy. Standing on the
    /// engine, air arrives at `V0` and is stopped; that KE reappears as the ram rise. The
    /// general statement is that stopping the flow conserves stagnation enthalpy
    /// (`h(Tt0) = h(T0) + V0²/2`) and entropy (the pr ratio sets `pt0`); at constant cp these
    /// collapse to the gamma-only closed form — EXCEPT that rung 1's rounded `R = 287` makes
    /// `gamma*R/cp` differ from `gamma-1` by ~0.05 %, which the `pt0` exponent `1/gc = 3.5`
    /// amplifies to ~0.18 %. So the CPG branch keeps the closed form, to stay exact.
    pub fn freestream(&self, flight: &FlightCondition, mdot: f64) -> (FlowState, f64) {
        self.try_freestream(flight, mdot).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin of [`freestream`](Self::freestream) — see [`Abort`].
    ///
    /// The ram check is a SANITY check, not a conservation law, and slice L is where it becomes
    /// control flow: rung 41's `surge_margin_schedule` skips a point whose `match` raises, and
    /// this is what raises on the whole `M0 = 0` column of the dump grid — **28 of the 294 cells
    /// measured**, on the two integral gases only, because `t_from_h_c(h_c(250))` round-trips
    /// three ulps low while the pressure clause is exact (§ 5.7 (f), now inside a caught scope).
    pub fn try_freestream(
        &self, flight: &FlightCondition, mdot: f64,
    ) -> Result<(FlowState, f64), Abort> {
        let gas = &self.gas;
        let (tt0, pt0, v0);
        if gas.cold_is_cpg() {
            // CPG: gamma-only closed form. The compressibility factor appears in BOTH the Tt
            // and the pt relation.
            let stag = 1.0 + 0.5 * (gas.gamma_c() - 1.0) * (flight.m0 * flight.m0);
            tt0 = flight.t0 * stag;
            pt0 = flight.p0 * powp(stag, 1.0 / gas.g_c());     // 1/gc = gamma_c/(gamma_c-1)
            let a0 = powp(gas.gamma_c() * gas.r_c() * flight.t0, 0.5);   // sound speed, m/s
            v0 = flight.m0 * a0;
        } else {
            // TPG: stagnation enthalpy + pr ratio. gamma at the LOCAL static T0.
            let a0 = powp(gas.gamma_c_at(flight.t0) * gas.r_c() * flight.t0, 0.5);
            v0 = flight.m0 * a0;
            tt0 = gas.t_from_h_c(gas.h_c(flight.t0) + 0.5 * (v0 * v0));
            pt0 = flight.p0 * gas.pr_c(tt0) / gas.pr_c(flight.t0);
        }
        let state0 = FlowState { tt: tt0, pt: pt0, mdot, far: 0.0 };

        // Sanity check, every call. NOT a conservation law: station 0 manufactures totals from
        // statics, so stopping the flow can only raise T and p.
        if !(tt0 >= flight.t0 && pt0 >= flight.p0) {
            return Err(Abort("ram must not cool/depressurize".to_string()));
        }
        Ok((state0, v0))
    }

    /// Propagate the flow 0 -> 9 and compute performance.
    ///
    /// Chains each component (a pure transform), collecting the station table. The turbine is
    /// the one coupled step: the dual-cp + `eta_m` shaft balance is solved HERE and the result
    /// passed in, then closed from the turbine's OUTPUT as an independent check. Then the flow
    /// converts to static at the nozzle exit and performance is scored.
    pub fn run(&self, flight: &FlightCondition, mdot: f64) -> EngineResult {
        let gas = &self.gas;

        // Station 0: manufacture the freestream totals + flight velocity.
        let (mut state, v0) = self.freestream(flight, mdot);
        let mut stations: Vec<(&'static str, FlowState)> = vec![("0", state)];

        // Nozzle-exit statics, filled when the flow reaches station 9.
        let (mut m9, mut t9, mut v9, mut p9) = (f64::NAN, f64::NAN, f64::NAN, f64::NAN);

        // Walk the components in flow order. Turbine and Nozzle diverge from the bare
        // apply(state, gas) and are handled explicitly — the engine owns the awkward bits
        // (SPEC.md § Architecture).
        for &(label, component) in &self.components {
            match component {
                Component::Turbine(turbine) => {
                    // THE SHAFT BALANCE (enthalpy + mechanical efficiency), in the open.
                    // `state` here is station 4, which carries f as its far. Rung 2's
                    // cp*delta_Tt is promoted to delta_h.
                    let f = state.far;
                    let s4 = state;
                    let tt3 = station_of(&stations, "3").tt;
                    let tt2 = station_of(&stations, "2").tt;
                    let delta_h =
                        (gas.h_c(tt3) - gas.h_c(tt2)) / (self.eta_m * (1.0 + f));
                    state = turbine.apply(&state, gas, delta_h);
                    // Shaft CLOSURE check (engine-owned — it alone holds Tt2/Tt3). Computed
                    // two INDEPENDENT ways: turbine power from the turbine's OUTPUT Tt5
                    // (re-applying eta_m, 1+f, h_t), compressor power from the cold states. A
                    // dropped factor in delta_h fires this.
                    let compressor_power = gas.h_c(tt3) - gas.h_c(tt2);
                    let turbine_power = self.eta_m * (1.0 + state.far)
                        * (gas.h_t(s4.tt, state.far) - gas.h_t(state.tt, state.far));
                    assert!((turbine_power - compressor_power).abs()
                                < 1e-6 * compressor_power,
                            "shaft does not close: turbine {turbine_power} != compressor \
                             {compressor_power}");
                }
                Component::Nozzle(nozzle) => {
                    let exit = nozzle.apply(&state, gas);
                    state = exit.state;                 // station-9 TOTALS go on the table
                    m9 = exit.m9;                       // statics ride out
                    t9 = exit.t9;
                    v9 = exit.v9;
                    p9 = exit.p9;
                }
                other => {
                    state = other.apply(&state, gas);
                }
            }
            stations.push((label, state));
        }

        let performance = score(gas, &stations, v0, t9, v9, p9, flight.p0, gas.hpr());

        EngineResult { stations, performance, v0, v9, m9, t9, p9 }
    }
}

fn station_of<'a>(stations: &'a [(&'static str, FlowState)], label: &str) -> &'a FlowState {
    stations.iter().find(|&&(l, _)| l == label)
        .map(|(_, s)| s)
        .unwrap_or_else(|| panic!("station {label:?} not reached yet"))
}

/// Score a station table into a [`Performance`] (`docs/rung2-spec.md` § Performance).
///
/// Kept separate from [`Engine::run`] so the rung-31 off-design path (phase 5) scores
/// IDENTICALLY rather than growing a second copy of these formulae.
///
/// The PRESSURE-THRUST term `(1+f)*Rt*T9*(1-p0/p9)/V9` vanishes when `p9 == p0` (fully
/// expanded), recovering rung 1's `(1+f)*V9 - V0`. It is the static-pressure imbalance
/// `A9*(p9-p0)/mdot` rewritten via the ideal gas law — and it is exactly what carries rung
/// 30's choked-nozzle finding.
pub fn score(
    gas: &Gas, stations: &[(&'static str, FlowState)], v0: f64,
    t9: f64, v9: f64, p9: f64, p0: f64, hpr: f64,
) -> Performance {
    try_score(gas, stations, v0, t9, v9, p9, p0, hpr).unwrap_or_else(|e| panic!("{}", e.0))
}

/// The FALLIBLE twin of [`score`] — see [`Abort`].
///
/// The cascade closure is a free consistency check on a converged cycle, so it reads as a
/// conservation assert; it becomes control flow at slice L for one structural reason. It is
/// `0/0` at `M0 = 0` and near-`0/0` at very low thrust, and rung 41's schedule methods march
/// straight through both. Measured: **27 raises of 294 cells** on the dump grid, every one on a
/// cell rung 41 SKIPS.
#[allow(clippy::too_many_arguments)]
pub fn try_score(
    gas: &Gas, stations: &[(&'static str, FlowState)], v0: f64,
    t9: f64, v9: f64, p9: f64, p0: f64, hpr: f64,
) -> Result<Performance, Abort> {
    let f = station_of(stations, "4").far;
    let pressure_thrust = (1.0 + f) * gas.r_t_at(f) * t9 * (1.0 - p0 / p9) / v9;
    let specific_thrust = (1.0 + f) * v9 - v0 + pressure_thrust;
    let tsfc = f / specific_thrust;
    // eta_brayton: the cold-Brayton identity 1 - Tt2/Tt3 (the rung-1 hand-check).
    let eta_brayton = 1.0 - station_of(stations, "2").tt / station_of(stations, "3").tt;
    let ke_net = (1.0 + f) * (v9 * v9) - (v0 * v0);
    let eta_thermal = ke_net / (2.0 * f * hpr);
    let eta_propulsive = (specific_thrust * v0) / (0.5 * ke_net);
    let eta_overall = (specific_thrust * v0) / (f * hpr);
    // CASCADE CLOSURE (free consistency check): the KE-based cascade holds EXACTLY, which is
    // the whole reason eta_thermal is defined on kinetic energy rather than on Tt2/Tt3.
    if !((eta_overall - eta_thermal * eta_propulsive).abs() < 1e-9 * eta_overall) {
        return Err(Abort(
            "efficiency cascade eta_o == eta_thermal*eta_p must hold under the KE definition"
                .to_string()));
    }
    Ok(Performance {
        specific_thrust, tsfc, eta_brayton, eta_thermal, eta_propulsive, eta_overall,
    })
}

/// The rung-2 loss parameters, all defaulting to IDEAL.
///
/// Python spells these as keyword-only arguments with ideal defaults, so that the no-keyword
/// call is the rung-1 ideal engine — the reduce-to-ideal gate. `Losses::default()` is that
/// same statement, and `Losses { eta_c: 0.88, ..Default::default() }` is that same call.
#[derive(Clone, Copy, Debug)]
pub struct Losses {
    /// Inlet net total-pressure recovery (`= pi_d_max * ram_recovery(M0)`, folded in at the
    /// design Mach — use [`crate::components::ram_recovery`]).
    pub pi_d: f64,
    /// Compressor ISENTROPIC efficiency (rung 2).
    pub eta_c: f64,
    /// Compressor POLYTROPIC efficiency (rung 2b). Mutually exclusive with `eta_c`.
    pub e_c: Option<f64>,
    /// Burner combustion efficiency.
    pub eta_b: f64,
    /// Burner total-pressure ratio.
    pub pi_b: f64,
    /// Turbine ISENTROPIC efficiency (rung 2).
    pub eta_t: f64,
    /// Turbine POLYTROPIC efficiency (rung 2b). Mutually exclusive with `eta_t`.
    pub e_t: Option<f64>,
    /// Shaft mechanical efficiency — lives on the [`Engine`], which owns the shaft.
    pub eta_m: f64,
    /// Nozzle total-pressure ratio.
    pub pi_n: f64,
    /// Specified nozzle exit static pressure. `None` => `p_ambient` (fully expanded). Set it
    /// away from ambient for an under/over-expanded nozzle — specific thrust then carries the
    /// pressure term.
    pub p_exit: Option<f64>,
    /// RUNG 30. `true` makes the nozzle a fixed CONVERGENT one that choke-detects and ignores
    /// `p_exit`: the FLOW decides `p9` (the sonic `p*` if choked, else `p_ambient`).
    ///
    /// The default `false` keeps the specified-`p_exit` nozzle, so every rungs 1-6 number is
    /// untouched by construction. **Rung 31 REQUIRES it** — the matcher's `A8` is the throat
    /// area of a convergent nozzle, and there is no such area without one.
    pub nozzle_convergent: bool,
}

impl Default for Losses {
    fn default() -> Self {
        Losses {
            pi_d: 1.0, eta_c: 1.0, e_c: None, eta_b: 1.0, pi_b: 1.0,
            eta_t: 1.0, e_t: None, eta_m: 1.0, pi_n: 1.0, p_exit: None,
            nozzle_convergent: false,
        }
    }
}

/// Factory: wire the five components into a single-spool turbojet.
///
/// Order: Inlet -> Compressor -> Burner -> Turbine -> Nozzle. With `Losses::default()` this is
/// the rung-1 ideal engine — the reduce-to-ideal gate (`docs/rung2-spec.md` § Verification
/// gates).
///
/// The gas is MOVED in: an equilibrium gas carries a frozen station-4 mixture and a burn-config
/// guard, so "which engine owns which gas" is load-bearing state, not a borrow detail. A caller
/// that needs the gas afterwards reads `engine.gas`.
pub fn build_turbojet(
    gas: Gas, pi_c: f64, tt4: f64, p_ambient: f64, losses: Losses,
) -> Engine {
    let components: Vec<(&'static str, Component)> = vec![
        ("2", Component::Inlet(Inlet::new(losses.pi_d))),
        ("3", Component::Compressor(Compressor::new(pi_c, losses.eta_c, losses.e_c))),
        ("4", Component::Burner(Burner::new(tt4, losses.eta_b, losses.pi_b))),
        ("5", Component::Turbine(Turbine::new(losses.eta_t, losses.e_t))),
        ("9", Component::Nozzle(if losses.nozzle_convergent {
            Nozzle::convergent(p_ambient, losses.pi_n)
        } else {
            Nozzle::new(p_ambient, losses.pi_n, losses.p_exit)
        })),
    ];
    Engine::new(gas, components, losses.eta_m)
}
