//! RUNG 62 — **THE BLEED SCHEDULE beside the STATOR SCHEDULE, on the transient plant.**
//! `tests/test_rung62.py`, ported one-to-one.
//!
//! **58 `#[test]` against Python's 58 COLLECTED** (from 23 `def test_` — a 2.1× `parametrize`
//! expansion, the largest in the port so far). Both counts are EMITTED, not typed:
//! `pytest --collect-only -q tests/test_rung62.py` reports 58, and `cargo test --release` reports
//! 58 run, 0 ignored. **12 of the 23 carry `@pytest.mark.slow` and not one becomes `#[ignore]`** —
//! slice M's `0 ignored` line holds.
//!
//! THE HEADLINE: a state-fed schedule closes a FEEDBACK LOOP on itself through the shaft speed it
//! reads, and the loop's SIGN is the sign of the lever's own `dn/d(setting)`. Rung 57 found the
//! stator schedule SELF-CANCELS (`FULL/RAMP` = 0.77–0.83) because closing stators raises `n` and
//! the schedule opens back up: `(dn/dv)(dv/dn) = (+)(-) < 0`. A handling bleed flips one factor —
//! rung 61 § 2's own −9.77 % demand term is `dn_L/db < 0` — so the SAME instrument on the SAME
//! plant returns `FULL/RAMP` = 1.09–1.10: **the bleed schedule AMPLIFIES itself.** Both signs were
//! derivable from published tables before either was measured.
//!
//! THE SECOND FINDING: the two loops close through ONE state and they do not compose. A bleed
//! SCHEDULE beside a stator schedule TRIPLES the stator's own surrender while the stator leaves
//! the bleed's amplification alone to within 0.7 % — a **one-way arrow** running from the
//! amplifying lever to the cancelling one. And it is the LOOP, not the LEVEL: a CONSTANT valve at
//! the schedule's own commanded value reaches a fraction of it.
//!
//! **WHAT 88/88 GREEN DOES NOT ESTABLISH.** Every gate in this file is RELATIONAL — it
//! asserts a relation among values THIS CRATE computed, so a Rust/Python arithmetic
//! divergence moves both sides of every one of them and leaves the whole suite green.
//! Agreement with Python is step 4's oracle, not this file. Step 3 MEASURED how far
//! that reaches, by running 6 INJECTIONS over 5 distinct defects (I2 and I2b are two
//! spellings of one) into `bleed_transient.rs`, counting both
//! how many of a 871-key probe moved and how many of the 88 fired:
//!
//! * **I1** — `mdot_face` returning the TRIAL face flow rather than the imposed `mdot_imp/(1-b)`: **312 of 871** gate-visible keys move, 2 witness — **0 of 88 catch it**.
//! * **I2** — `_powers`/`_instant_tail` re-reading `b_of` instead of the closure's own `bleed` key: **0 of 871** gate-visible keys move, 7 witness — **0 of 88 catch it**.
//! * **I2b** — the same re-read confined to the branch PREDICATE (P4's own injection): **0 of 871** gate-visible keys move, 7 witness — **0 of 88 catch it**.
//! * **I3** — `R62_FUEL` spread from `..R43`, which drops rung 57's floor-RESOLVING `_surge_fuel`: **0 of 871** gate-visible keys move, 1 witness, 3 key(s) never emitted (its section PANICS) — **0 of 88 catch it**.
//! * **I4** — `at_stator` left un-overridden, the shape slice V shipped: **4 of 871** gate-visible keys move, 1 witness, 1 key(s) never emitted (its section PANICS) — **2 of 88 catch it**.
//! * **I5** — the `1/(1-b)` dropped from the fuel bracket walls: **151 of 871** gate-visible keys move, 2 witness — **0 of 88 catch it**.
//!
//! So **5 of the 6 injections are invisible to every gate in both suites**, and the one
//! that is
//! caught is caught by the two written for it. The zeros are not probe blindness:
//! `caught > 0` implies `moved_G > 0` on every row, and the two that move NOTHING
//! gate-visible each leave a witness the gates never read — `b_of`'s call count for
//! I2/I2b, and for I3 a deliberately un-ported `Floor::Incidence` cell that PANICS,
//! which is what shows the channel is live at all. Both suites build only
//! `Floor::Phi`, and that is precisely why I3 reaches nothing here.

use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::stator_transient::{
    Ramp, ScheduledStatorCore, ScheduledStatorTransient, Shape, StatorArm, StatorLeg,
    StatorSchedule,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::Instant2;

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.01;
const SETTLE: f64 = 1.2;
/// `n_lo` is placed BELOW both levers' armed idle speeds (stator 0.799, bleed 0.737) so neither
/// schedule is measured SATURATED. Rung 57's own 0.75574 leaves the bleed clipped at `b_max`,
/// where `db/dn = 0` and there is no loop to measure — the artifact gate 3's last test pins.
const N_LO: f64 = 0.65;
const V: f64 = 0.20;
const B: f64 = 0.10;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// `test_rung62.py`'s `_cpg`, spelled character for character — `R_c` is DERIVED, and
/// `1.4 - 1.0` is NOT `0.4` in IEEE-754.
fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn tilt_map() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

/// Python's `_bt` — a rung-62 machine on the shipped hardware.
fn bt(arm: &LeverArm) -> ScheduledStatorCore {
    bt_on(design(), lp_map(), hp_map(), arm)
}

fn bt_on(de: TwoSpoolEngine, lp: ComponentMap, hp: ComponentMap, arm: &LeverArm)
    -> ScheduledStatorCore {
    match build_scheduled_bleed(de, flight(), 1.0, Some(lp), Some(hp), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// A rung-57 machine on the same hardware — the reduce's other side.
fn st_on(de: TwoSpoolEngine, arm: StatorArm) -> ScheduledStatorCore {
    match ScheduledStatorTransient::new(de, flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                                        arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn stat_arm() -> LeverArm {
    LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(V, N_LO)))
}

fn bled_arm() -> LeverArm {
    LeverArm::scheduled(BleedSchedule::new(B, N_LO))
}

fn ramp(r: f64, ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds }
}

/// Python's `KEYS` — the fifteen the reduce gates compare.
fn keys(i: &Instant2) -> [(&'static str, f64); 15] {
    [("nu_lp", i.nu_lp), ("nu_hp", i.nu_hp), ("phi_lp", i.close.phi_lp),
     ("phi_hp", i.close.phi_hp), ("Tt4", i.tt4), ("f", i.close.f),
     ("pi_lpc", i.close.pi_lpc), ("pi_hpc", i.close.pi_hpc), ("Phi_lp", i.phi_lp_dot),
     ("Phi_hp", i.phi_hp_dot), ("sp_thrust", i.sp_thrust), ("m_lp", i.close.m_lp),
     ("m_hp", i.close.m_hp), ("Tt25", i.close.tt25), ("Tt3", i.close.tt3)]
}

fn same(a: &Instant2, b: &Instant2, label: &str) {
    for ((k, va), (_, vb)) in keys(a).into_iter().zip(keys(b)) {
        assert!(va.to_bits() == vb.to_bits(), "{label} key {k}: {va:?} != {vb:?}");
    }
}

// =============================================================================================
// GATE 1 — THE REDUCE, TWO-AXIS and per CALL
// =============================================================================================

/// `b == 0` dispatches to rung 57's own body VERBATIM at every state, so an unbled machine is
/// rung 57 (hence rungs 43–52) bit-for-bit on every recorded key.
fn reduce_valve_shut(kw57: StatorArm, kw62: LeverArm, label: &str, tt4: f64) {
    let de = design();
    let a = st_on(de.clone(), kw57);
    let c = bt_on(de, lp_map(), hp_map(), &kw62);
    same(&a.fuel.inner.equilibrium(&flight(), tt4),
         &c.fuel.inner.equilibrium(&flight(), tt4),
         &format!("{label} Tt4={tt4}"));
}

#[test]
fn reduce_valve_shut_is_rung57_bit_for_bit_v0_b0_1400() {
    reduce_valve_shut(StatorArm::default(), LeverArm::default(), "(v=0, b=0)", 1400.0);
}
#[test]
fn reduce_valve_shut_is_rung57_bit_for_bit_v0_b0_1200() {
    reduce_valve_shut(StatorArm::default(), LeverArm::default(), "(v=0, b=0)", 1200.0);
}
#[test]
fn reduce_valve_shut_is_rung57_bit_for_bit_v0_b0_1000() {
    reduce_valve_shut(StatorArm::default(), LeverArm::default(), "(v=0, b=0)", 1000.0);
}
#[test]
fn reduce_valve_shut_is_rung57_bit_for_bit_vconst_b0_1400() {
    reduce_valve_shut(StatorArm::constant(V, 0.0),
                      LeverArm::stator(StatorArm::constant(V, 0.0)), "(v const, b=0)", 1400.0);
}
#[test]
fn reduce_valve_shut_is_rung57_bit_for_bit_vconst_b0_1200() {
    reduce_valve_shut(StatorArm::constant(V, 0.0),
                      LeverArm::stator(StatorArm::constant(V, 0.0)), "(v const, b=0)", 1200.0);
}
#[test]
fn reduce_valve_shut_is_rung57_bit_for_bit_vconst_b0_1000() {
    reduce_valve_shut(StatorArm::constant(V, 0.0),
                      LeverArm::stator(StatorArm::constant(V, 0.0)), "(v const, b=0)", 1000.0);
}
#[test]
fn reduce_valve_shut_is_rung57_bit_for_bit_vsched_b0_1400() {
    reduce_valve_shut(StatorArm::scheduled_lp(StatorSchedule::new(V, N_LO)),
                      stat_arm(), "(v sched, b=0)", 1400.0);
}
#[test]
fn reduce_valve_shut_is_rung57_bit_for_bit_vsched_b0_1200() {
    reduce_valve_shut(StatorArm::scheduled_lp(StatorSchedule::new(V, N_LO)),
                      stat_arm(), "(v sched, b=0)", 1200.0);
}
#[test]
fn reduce_valve_shut_is_rung57_bit_for_bit_vsched_b0_1000() {
    reduce_valve_shut(StatorArm::scheduled_lp(StatorSchedule::new(V, N_LO)),
                      stat_arm(), "(v sched, b=0)", 1000.0);
}

/// A `BleedSchedule` with `b_max = 0.0` returns `0.0` at every `n`, at which point `_close`
/// RETURNS TO ITS PARENT rather than multiplying by `(1 - 0.0)`. The machinery is witnessed
/// inert, not merely arithmetically neutral (rung 57's `is`-not-`==` discipline).
///
/// **AND THE `in`-TEST PORTS EXACTLY.** Python asserts `"bleed" not in ez` — an ABSENT dict key,
/// which is why [`CloseState::bleed`] is an `Option<f64>` and not a zero. `is_none()` IS
/// `not in`.
#[test]
fn reduce_zero_schedule_dispatches_rather_than_computing_unit_factors() {
    let de = design();
    let a = st_on(de.clone(), StatorArm::default());
    let z = bt_on(de.clone(), lp_map(), hp_map(),
                  &LeverArm { bleed_sched: Some(BleedSchedule::new(0.0, N_LO)),
                              ..Default::default() });
    let ea = a.fuel.inner.equilibrium(&flight(), 1200.0);
    let ez = z.fuel.inner.equilibrium(&flight(), 1200.0);
    same(&ea, &ez, "b_max=0");
    assert!(ez.close.bleed.is_none(),
            "the b=0 path must reach rung 57's dict, not a bled one");
    let armed = bt_on(de, lp_map(), hp_map(), &LeverArm::constant(0.05));
    assert!(armed.fuel.inner.equilibrium(&flight(), 1200.0).close.bleed.is_some());
}

/// The FUEL closure has its own dispatch (and its own bracket), so it needs its own witness —
/// rungs 43–52 all live on `_close_fuel`.
#[test]
fn reduce_fuel_path_valve_shut_is_rung57_bit_for_bit() {
    let de = design();
    let a = st_on(de.clone(), StatorArm::default());
    let c = bt_on(de, lp_map(), hp_map(), &LeverArm::default());
    let mf = a.fuel.fuel_for_tt4(&flight(), 1200.0);
    assert!(mf.to_bits() == c.fuel.fuel_for_tt4(&flight(), 1200.0).to_bits());
    let ia = a.fuel.instant_fuel(&flight(), 0.85, 0.88, mf);
    let ic = c.fuel.instant_fuel(&flight(), 0.85, 0.88, mf);
    same(&ia.base, &ic.base, "fuel path");
}

// =============================================================================================
// GATE 2 — THE PLANT GATE. The (v=0, b!=0) corner has NO transient ancestor.
// =============================================================================================

/// **THE GATE THAT CAUGHT THE SILENT WRONG NUMBER.** Validated the way rung 40 validated itself:
/// through the FORWARD closure only, never by calling the steady matcher.
///
/// Rung 40 factored `(Phi_L, Phi_H)` out of `_instant_tail` into `_powers` for the Newton's inner
/// loop. With `_powers` left bleed-free the Newton converges to 1e-12 on a residual the plant
/// does not use and returns `n_L = 0.8720` against a true `0.8282` — **5.3 % wrong, with `phi_L`
/// still agreeing to 1e-3 and NO exception anywhere.** Nothing internal to the transient ladder
/// can see that; only this cross-object comparison can.
fn plant_forward_vs_rung42(tt4: f64, b: f64) {
    let de = design();
    let eq = bt_on(de.clone(), lp_map(), hp_map(), &LeverArm::constant(b))
        .fuel.inner.equilibrium(&flight(), tt4);
    let od = TwoSpoolBleedMatcher::new(de, flight(), 1.0, lp_map(), hp_map(), b)
        .match_point(&flight(), tt4);
    for (name, got, want) in [("n_lp", eq.close.n_lp, od.base.n_lp),
                              ("phi_lp", eq.close.phi_lp, od.base.phi_lp),
                              ("phi_hp", eq.close.phi_hp, od.base.phi_hp),
                              ("pi_lpc", eq.close.pi_lpc, od.base.base.pi_lpc),
                              ("pi_hpc", eq.close.pi_hpc, od.base.base.pi_hpc)] {
        assert!((got / want - 1.0).abs() < 1e-9,
                "Tt4={tt4} b={b} {name}: forward {got:?} vs rung-42 steady {want:?}");
    }
}

#[test] fn plant_forward_closure_reproduces_rung42_steady_match_1500_005() {
    plant_forward_vs_rung42(1500.0, 0.05);
}
#[test] fn plant_forward_closure_reproduces_rung42_steady_match_1200_005() {
    plant_forward_vs_rung42(1200.0, 0.05);
}
#[test] fn plant_forward_closure_reproduces_rung42_steady_match_1000_005() {
    plant_forward_vs_rung42(1000.0, 0.05);
}
#[test] fn plant_forward_closure_reproduces_rung42_steady_match_1500_010() {
    plant_forward_vs_rung42(1500.0, 0.10);
}
#[test] fn plant_forward_closure_reproduces_rung42_steady_match_1200_010() {
    plant_forward_vs_rung42(1200.0, 0.10);
}
#[test] fn plant_forward_closure_reproduces_rung42_steady_match_1000_010() {
    plant_forward_vs_rung42(1000.0, 0.10);
}

/// **THE ONE PLACE THE BLEED CHANGES THE CONTROL** and not just the flow. Every finding in this
/// rung runs through `_close_fuel`'s bleed branch, but the reduce gates above only exercise its
/// `b == 0` dispatch — so without this the branch executes constantly and is never asserted.
#[test]
fn plant_the_burner_sees_core_air_only() {
    let t0 = bt(&LeverArm::default());
    let tb = bt(&LeverArm::constant(0.10));
    let (tt2, pt2, _) = t0.fuel.inner.inlet(&flight());
    let mf = t0.fuel.fuel_for_tt4(&flight(), 1200.0);
    let a = t0.fuel.close_fuel(0.85, 0.88, mf, tt2, pt2);
    let b = tb.fuel.close_fuel(0.85, 0.88, mf, tt2, pt2);
    // `f` × CORE air recovers the metered fuel. This closes only AT the root, so it is asserted
    // at the closure's own tolerance rather than bit-exactly.
    assert!((b.base.f * b.base.mdot_air / mf - 1.0).abs() < 1e-9, "f is CORE-referenced");
    assert!((a.base.f * a.base.mdot_air / mf - 1.0).abs() < 1e-9, "and so is the b=0 path");
    let face = b.base.mdot_face.expect("the bled closure sets it");
    assert!((face / b.base.mdot_air - 1.0 / 0.9).abs() < 1e-12);   // the extraction, exact
    assert!(b.base.f > a.base.f && b.tt4 > a.tt4, "same fuel, less air => richer, hotter");
}

/// A DIRECT witness that `_powers` and `_instant_tail` agree under bleed — the two sites rung 40
/// split apart. If a future edit restores one and not the other, the equilibrium Newton silently
/// converges on the wrong residual again.
#[test]
fn plant_the_powers_touch_point_is_not_optional() {
    let t = bt(&LeverArm::constant(0.10));
    let (tt2, pt2, _) = t.fuel.inner.inlet(&flight());
    let c = t.fuel.inner.close(0.85, 0.88, 1200.0, tt2, pt2);
    let (p_lp, p_hp) = t.fuel.inner.powers(&c, &flight(), 0.85, 0.88, 1200.0).expect("powers");
    let inst = t.fuel.inner.instant(&flight(), 0.85, 0.88, 1200.0);
    assert!(p_lp.to_bits() == inst.phi_lp_dot.to_bits());
    assert!(p_hp.to_bits() == inst.phi_hp_dot.to_bits());
    assert!(c.bleed == Some(0.10) && c.mdot_face.expect("bled") > c.mdot_air);
}

/// Rung 42 captures `A4`/`A45`/`A8` and both maps' references with the valve SHUT, so a schedule
/// holding it open at `n_ref` would contradict every design reference.
#[test]
fn plant_schedule_is_shut_at_the_design_speed() {
    assert!(BleedSchedule::new(B, N_LO).at(1.0) == 0.0);
    assert!(BleedSchedule::with_shape(B, N_LO, BleedSchedule::N_REF, Shape::Linear).at(1.0)
            == 0.0);
    // n_lo >= n_ref
    assert!(std::panic::catch_unwind(|| BleedSchedule::new(B, 1.2)).is_err());
    // rung 42's own b < 0.5 bound
    assert!(std::panic::catch_unwind(|| BleedSchedule::new(0.6, N_LO)).is_err());
    // a position OR a schedule, not both
    assert!(std::panic::catch_unwind(|| bt(&LeverArm {
        bleed: 0.05, bleed_sched: Some(BleedSchedule::new(B, N_LO)), ..Default::default()
    })).is_err());
}

/// A `b(n_L)` schedule is MOST open at `Tt4_lo`, and rung 42 warns its choked guard *"bites
/// SOONER"* with the valve open. Checked rather than assumed.
#[test]
fn plant_stays_in_the_choked_scope_at_the_idle_end() {
    let de = design();
    for tt4 in [900.0, LO] {
        for b in [0.10, 0.20, 0.30] {
            let m = bt_on(de.clone(), lp_map(), hp_map(), &LeverArm::constant(b));
            assert!(m.fuel.inner.equilibrium(&flight(), tt4).branch == Branch::Choked);
        }
    }
}

/// **RUNG 61's `at_setting` TRAP, one ladder over.** Rung 57 hard-constructs a bare sibling in
/// `at_stator`, and `stator_credit` / `credit_decomposition` / `arrow_toggle` all route their
/// BARE leg through it. Un-overridden, every one would difference an armed machine against a
/// VALVE-SHUT bare one and attribute the valve's whole effect to the stator — plausible numbers,
/// no exception.
///
/// **Python's `isinstance(sib, ScheduledBleedTransient)` has no port and does not need one.**
/// § 5.21 (iii): one type carries rungs 57–84 and the RUNG is the table, so the claim
/// `isinstance` was making — *this sibling is a rung-62 object* — is exactly the two assertions
/// below, which read the state and the arming rather than the class.
#[test]
fn trap_at_stator_carries_the_valve() {
    let t = bt(&LeverArm::constant(0.10));
    let sib = t.at_stator(StatorArm::constant(V, 0.0));
    assert!(sib.armed_bleed(), "the sibling must be a rung-62 object, valve and all");
    assert!(sib.fuel.inner.lever.bleed == 0.10 && sib.arming().vsv_lp == V);
    let s2 = bt(&bled_arm()).at_stator(StatorArm::scheduled_lp(StatorSchedule::new(V, N_LO)));
    assert!(s2.fuel.inner.lever.sched.is_some() && s2.arming().sched_lp.is_some());
}

// =============================================================================================
// GATE 3 — THE HEADLINE: the loop gain's SIGN
// =============================================================================================

/// The sign argument rests on two derivatives, and rung 42's own `dphi_H/db` REVERSES at
/// `pi* = 3.24674` — so they are measured, not quoted.
fn loop_factor_signs(tt4: f64) {
    let row = bt(&LeverArm::default()).loop_factors(&flight(), &[tt4], 0.10, 0.20)[0];
    assert!(row.dn_db < 0.0, "Tt4={tt4}: dn_L/db = {}", row.dn_db);
    assert!(row.dn_dv > 0.0, "Tt4={tt4}: dn_L/dv = {}", row.dn_dv);
}

#[test] fn headline_both_loop_factors_1500() { loop_factor_signs(1500.0); }
#[test] fn headline_both_loop_factors_1300() { loop_factor_signs(1300.0); }
#[test] fn headline_both_loop_factors_1100() { loop_factor_signs(1100.0); }
#[test] fn headline_both_loop_factors_0900() { loop_factor_signs(900.0); }

/// **THE RUNG.** Same instrument, same plant, same `n_lo`, same ramp: the stator schedule
/// surrenders authority to its own loop and the bleed schedule GAINS it.
fn opposite_sides_of_one(r: f64) {
    let de = design();
    let s = bt_on(de.clone(), lp_map(), hp_map(), &stat_arm())
        .loop_decomposition(&flight(), &ramp(r, DS), Spool::Lp);
    let b = bt_on(de, lp_map(), hp_map(), &bled_arm())
        .loop_decomposition(&flight(), &ramp(r, DS), Spool::Lp);
    assert!(s.self_cancel < 1.0, "stator r={r}: {}", s.self_cancel);
    assert!(b.self_cancel > 1.0, "bleed  r={r}: {}", b.self_cancel);
    // and the sizes rung 62 publishes, as bands rather than points
    assert!(0.75 < s.self_cancel && s.self_cancel < 0.85, "{}", s.self_cancel);
    assert!(1.08 < b.self_cancel && b.self_cancel < 1.11, "{}", b.self_cancel);
}

#[test] fn headline_two_schedules_opposite_sides_r025() { opposite_sides_of_one(0.25); }
#[test] fn headline_two_schedules_opposite_sides_r050() { opposite_sides_of_one(0.50); }
#[test] fn headline_two_schedules_opposite_sides_r100() { opposite_sides_of_one(1.00); }

/// Not a ratio of credits: between the RAMP and FULL legs the two schedules move their own
/// COMMANDED setting in OPPOSITE directions. The stator backs off; the bleed leans in. **This is
/// the loop itself, and it needs no normalisation to read.**
fn commanded_setting_moves_opposite(r: f64) {
    let de = design();
    let s = bt_on(de.clone(), lp_map(), hp_map(), &stat_arm())
        .loop_decomposition(&flight(), &ramp(r, DS), Spool::Lp);
    let b = bt_on(de, lp_map(), hp_map(), &bled_arm())
        .loop_decomposition(&flight(), &ramp(r, DS), Spool::Lp);
    assert!(s.cmd_full < s.cmd_ramp, "stator: {} -> {}", s.cmd_ramp, s.cmd_full);
    assert!(b.cmd_full > b.cmd_ramp, "bleed:  {} -> {}", b.cmd_ramp, b.cmd_full);
    // the head start's own sign, which is what drives both.
    assert!(s.nu0_armed > s.nu0_ref && b.nu0_armed < b.nu0_ref);
}

#[test] fn headline_loop_witnessed_in_commanded_setting_r025() {
    commanded_setting_moves_opposite(0.25);
}
#[test] fn headline_loop_witnessed_in_commanded_setting_r100() {
    commanded_setting_moves_opposite(1.00);
}

/// `smooth` is C1 at both corners (`S' = 0` there); `linear` is not. The sign must not be a
/// property of the flat spot.
fn survives_shape(shape: Shape) {
    let de = design();
    let s = bt_on(de.clone(), lp_map(), hp_map(), &LeverArm::stator(
        StatorArm::scheduled_lp(StatorSchedule::with_shape(V, N_LO, 1.0, shape))))
        .loop_decomposition(&flight(), &ramp(0.25, DS), Spool::Lp);
    let b = bt_on(de, lp_map(), hp_map(), &LeverArm::scheduled(
        BleedSchedule::with_shape(B, N_LO, BleedSchedule::N_REF, shape)))
        .loop_decomposition(&flight(), &ramp(0.25, DS), Spool::Lp);
    assert!(s.self_cancel < 1.0 && 1.0 < b.self_cancel);
}

#[test] fn headline_survives_shape_smooth() { survives_shape(Shape::Smooth); }
#[test] fn headline_survives_shape_linear() { survives_shape(Shape::Linear); }

/// The composite ratios are differences of small marginal credits, so the RK4 grid is CHECKED
/// rather than trusted.
#[test]
fn headline_is_grid_converged() {
    let de = design();
    let mut vals: Vec<(f64, f64)> = Vec::new();
    for ds in [0.02, 0.01, 0.005] {
        vals.push((
            bt_on(de.clone(), lp_map(), hp_map(), &stat_arm())
                .loop_decomposition(&flight(), &ramp(0.25, ds), Spool::Lp).self_cancel,
            bt_on(de.clone(), lp_map(), hp_map(), &bled_arm())
                .loop_decomposition(&flight(), &ramp(0.25, ds), Spool::Lp).self_cancel));
    }
    for i in 0..2 {
        let g = |v: &(f64, f64)| if i == 0 { v.0 } else { v.1 };
        let lo = vals.iter().map(g).fold(f64::INFINITY, f64::min);
        let hi = vals.iter().map(g).fold(f64::NEG_INFINITY, f64::max);
        assert!((hi - lo) / lo < 0.02, "leg {i} moves {:.4} across the grid", (hi - lo) / lo);
    }
}

/// **THE ARTIFACT THIS RUNG PUBLISHES.** At rung 57's own `n_lo = 0.75574` the bleed's head start
/// pushes `nu0` BELOW `n_lo`, where `S` clips to 1, `b == b_max` and `db/dn = 0` — there is no
/// loop left to measure. The SIGN survives it, but the magnitude halves, so the placement is
/// load-bearing and is asserted rather than left to the reader.
///
/// The saturation itself is gated EXACTLY — the schedule's own clip — not through an arithmetic
/// proxy.
#[test]
fn headline_is_not_a_saturated_schedule_artifact() {
    let de = design();
    let sat_sched = BleedSchedule::new(B, 0.75574);
    let free_sched = BleedSchedule::new(B, N_LO);
    let sat = bt_on(de.clone(), lp_map(), hp_map(), &LeverArm::scheduled(sat_sched))
        .loop_decomposition(&flight(), &ramp(0.25, DS), Spool::Lp);
    let free = bt_on(de, lp_map(), hp_map(), &LeverArm::scheduled(free_sched))
        .loop_decomposition(&flight(), &ramp(0.25, DS), Spool::Lp);
    assert!(sat_sched.at(sat.nu0_armed) == B, "the bad placement must really be clipped");
    assert!(free_sched.at(free.nu0_armed) < B, "the good placement must be off the clip");
    assert!(sat.self_cancel > 1.0 && free.self_cancel > 1.0);
    assert!(free.self_cancel - 1.0 > 1.8 * (sat.self_cancel - 1.0));
}

// =============================================================================================
// GATE 4 — THE SECOND FINDING: the loops do NOT compose, and the arrow is ONE-WAY
// =============================================================================================

/// P3 predicted the bleed's positive loop would RESTORE part of what the stator's negative loop
/// surrenders. **Refuted with the opposite sign — it triples it.** The neighbour is carried on
/// BOTH sides of the difference, so what is measured is the stator schedule's own loop and not
/// the pair's composite.
fn bleed_triples_the_surrender(r: f64) {
    let t = bt(&LeverArm::default());
    let free = StatorLeg::default();
    let alone = t.marginal_loop(&flight(), &ramp(r, DS), &stat_arm(), None, Spool::Lp, &free);
    let beside = t.marginal_loop(&flight(), &ramp(r, DS), &stat_arm(), Some(&bled_arm()),
                                 Spool::Lp, &free);
    assert!(0.15 < alone.surrendered && alone.surrendered < 0.25, "{}", alone.surrendered);
    assert!(beside.surrendered > 2.5 * alone.surrendered);
    assert!(beside.surrendered > 0.60);
}

#[test] fn second_finding_bleed_triples_surrender_r025() { bleed_triples_the_surrender(0.25); }
#[test] fn second_finding_bleed_triples_surrender_r050() { bleed_triples_the_surrender(0.50); }
#[test] fn second_finding_bleed_triples_surrender_r100() { bleed_triples_the_surrender(1.00); }

/// The mirror: a stator schedule barely touches the bleed schedule's amplification.
fn arrow_is_one_way(r: f64) {
    let t = bt(&LeverArm::default());
    let free = StatorLeg::default();
    let alone = t.marginal_loop(&flight(), &ramp(r, DS), &bled_arm(), None, Spool::Lp, &free);
    let beside = t.marginal_loop(&flight(), &ramp(r, DS), &bled_arm(), Some(&stat_arm()),
                                 Spool::Lp, &free);
    assert!(alone.self_cancel > 1.0 && beside.self_cancel > 1.0);
    assert!((beside.self_cancel / alone.self_cancel - 1.0).abs() < 0.02);
}

#[test] fn second_finding_the_arrow_is_one_way_r025() { arrow_is_one_way(0.25); }
#[test] fn second_finding_the_arrow_is_one_way_r100() { arrow_is_one_way(1.00); }

/// **THE CONTROL THAT MAKES IT MEAN ANYTHING.** A CONSTANT valve has no loop of its own. Matched
/// at the value the schedule actually commands at its own surge minimum — and even OVER-matched
/// at `b_max`, which is strictly more lever than the schedule ever applies — a constant moves the
/// stator's surrender a fraction as far as the schedule does. Without this leg the finding would
/// be indistinguishable from *"more bleed"*.
fn is_the_loop_and_not_the_level(r: f64) {
    let t = bt(&LeverArm::default());
    let free = StatorLeg::default();
    let cmd = bt(&bled_arm()).commanded_level(&flight(), &ramp(r, DS), Spool::Lp).at_min;
    assert!(cmd < B, "the schedule must command LESS than b_max for this to be a control");
    let surr = |nb: Option<&LeverArm>| {
        t.marginal_loop(&flight(), &ramp(r, DS), &stat_arm(), nb, Spool::Lp, &free).surrendered
    };
    let alone = surr(None);
    let matched = surr(Some(&LeverArm::constant(cmd)));
    let over = surr(Some(&LeverArm::constant(B)));
    let sched = surr(Some(&bled_arm()));
    assert!(alone < matched && matched < over && over < sched,
            "{alone} {matched} {over} {sched}");
    assert!(sched > 2.2 * over);
}

#[test] fn second_finding_loop_not_level_r025() { is_the_loop_and_not_the_level(0.25); }
#[test] fn second_finding_loop_not_level_r050() { is_the_loop_and_not_the_level(0.50); }
#[test] fn second_finding_loop_not_level_r100() { is_the_loop_and_not_the_level(1.00); }

/// The measured mechanism: as the stator raises `n` the bleed schedule CLOSES, which raises `n`
/// further, so the stator's own head start is larger in the pair than alone. The small-signal
/// *"two loops multiply"* algebra is NOT asserted — only this.
#[test]
fn second_finding_mechanism_the_head_start_is_enlarged() {
    let t = bt(&LeverArm::default());
    let nu = |arm: &LeverArm| t.at_lever(arm).fuel.inner.equilibrium(&flight(), LO).nu_lp;
    let n_bare = nu(&LeverArm::default());
    let n_stat = nu(&stat_arm());
    let n_bled = nu(&bled_arm());
    let n_pair = nu(&LeverArm::merged(&stat_arm(), &bled_arm()));
    assert!(n_stat > n_bare && n_bare > n_bled, "the two head starts' signs");
    assert!(n_pair - n_bare > (n_stat - n_bare) + (n_bled - n_bare), "super-additive");
    assert!(n_pair - n_bled > 1.15 * (n_stat - n_bare), "the stator's is ENLARGED");
}

// =============================================================================================
// GATE 5 — CORRECTS RUNG 61: the steady near-additivity was the SHAFT BALANCE's
// =============================================================================================

/// Rung 61 measured these two devices additive to ≤ 2.3 % on the STEADY matcher. Rung 40 removed
/// the shaft balance; the same pair is sub-additive by an order more here.
fn credits_sub_additive(lp: ComponentMap, hp: ComponentMap, r: f64) {
    let d = bt_on(design(), lp, hp, &LeverArm::default())
        .pair_interaction(&flight(), &ramp(r, DS), &stat_arm(), &bled_arm(), Spool::Lp);
    assert!(d.interaction < 0.0, "sub-additive, not synergistic");
    assert!(0.08 < -d.interaction_frac && -d.interaction_frac < 0.32, "{}",
            d.interaction_frac);
    assert!(-d.interaction_frac > 3.0 * 0.023, "must clear rung 61's steady 2.3 %");
}

#[test] fn corrects_rung61_credits_sub_additive_shaped_r025() {
    credits_sub_additive(lp_map(), hp_map(), 0.25);
}
#[test] fn corrects_rung61_credits_sub_additive_shaped_r050() {
    credits_sub_additive(lp_map(), hp_map(), 0.50);
}
#[test] fn corrects_rung61_credits_sub_additive_shaped_r100() {
    credits_sub_additive(lp_map(), hp_map(), 1.00);
}
#[test] fn corrects_rung61_credits_sub_additive_tilted_r025() {
    credits_sub_additive(tilt_map(), tilt_map(), 0.25);
}
#[test] fn corrects_rung61_credits_sub_additive_tilted_r050() {
    credits_sub_additive(tilt_map(), tilt_map(), 0.50);
}
#[test] fn corrects_rung61_credits_sub_additive_tilted_r100() {
    credits_sub_additive(tilt_map(), tilt_map(), 1.00);
}

/// Rung 61's cost interaction was positive in all 30 steady rows. It survives the transplant.
/// **Asserted RAW**: `cost_b` is negative while `cost_a` is positive, so a normalised interaction
/// would put a difference of opposite-signed terms in its denominator — rung 43's
/// currency-circularity trap.
fn adverse_speed_cost(lp: ComponentMap, hp: ComponentMap, r: f64) {
    let d = bt_on(design(), lp, hp, &LeverArm::default())
        .pair_interaction(&flight(), &ramp(r, DS), &stat_arm(), &bled_arm(), Spool::Lp);
    assert!(d.cost_a > 0.0 && 0.0 > d.cost_b, "the two levers pay in opposite speed signs");
    assert!(d.cost_interaction > 0.0, "{}", d.cost_interaction);
}

#[test] fn corrects_rung61_adverse_speed_shaped_r025() {
    adverse_speed_cost(lp_map(), hp_map(), 0.25);
}
#[test] fn corrects_rung61_adverse_speed_shaped_r050() {
    adverse_speed_cost(lp_map(), hp_map(), 0.50);
}
#[test] fn corrects_rung61_adverse_speed_shaped_r100() {
    adverse_speed_cost(lp_map(), hp_map(), 1.00);
}
#[test] fn corrects_rung61_adverse_speed_tilted_r025() {
    adverse_speed_cost(tilt_map(), tilt_map(), 0.25);
}
#[test] fn corrects_rung61_adverse_speed_tilted_r050() {
    adverse_speed_cost(tilt_map(), tilt_map(), 0.50);
}
#[test] fn corrects_rung61_adverse_speed_tilted_r100() {
    adverse_speed_cost(tilt_map(), tilt_map(), 1.00);
}

// =============================================================================================
// GATE 6 — THE CONTROL that is explicitly NOT a finding (rung 57 already said it)
// =============================================================================================

/// Rung 57 § 2 ALREADY names the mechanism (both its channels are algebraic in the instantaneous
/// state), so *"the bleed has no clock"* is a CONFIRMATION and is gated as a control. What is new
/// is the complementary case, and its signature is **MONOTONICITY**: a wall-mover's floor channel
/// contributes exactly `v` whatever the trajectory does, so its credit/setting wobbles
/// non-monotonically at the 0.4 % level; a point-mover's whole credit runs through `phi` and
/// decays strictly monotonically with ramp rate.
#[test]
fn control_ramp_invariance_is_a_wall_mover_property() {
    const RATES: [f64; 5] = [0.10, 0.25, 0.50, 1.00, 2.00];
    let t = bt(&LeverArm::default());
    let per = |arm: &LeverArm, setting: f64| -> Vec<f64> {
        t.clock_sweep(&flight(), &ramp(0.5, DS), arm, setting, &RATES, Spool::Lp)
            .into_iter().map(|x| x.per_setting).collect()
    };
    let bl = per(&LeverArm::constant(B), B);
    let st = per(&LeverArm::stator(StatorArm::constant(V, 0.0)), V);
    assert!((0..bl.len() - 1).all(|i| bl[i] > bl[i + 1]), "{bl:?}");
    assert!(!(0..st.len() - 1).all(|i| st[i] > st[i + 1]), "{st:?}");
    let spread = |v: &[f64]| {
        let lo = v.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        (hi - lo) / lo
    };
    assert!(spread(&st) < 0.04);
    assert!(spread(&bl) > 3.0 * spread(&st));
}

// =============================================================================================
// GATE 7 — CYCLE UNTOUCHED
// =============================================================================================

/// Rung 62 adds a transient plant and reads on it. The default single-spool design run must be
/// untouched — the project's spine since rung 7.
#[test]
fn cycle_untouched_design_run_is_bit_for_bit_rung6() {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_LPC * PI_HPC, TT4, 50_000.0,
                             Losses { pi_d: 0.97, eta_b: 0.99, pi_b: 0.96, eta_m: 0.99,
                                      pi_n: 0.98, ..Losses::default() });
    let res = eng.run(&flight(), 1.0);
    let reference = eng.run(&flight(), 1.0);
    assert!(res.performance.specific_thrust > 0.0 && res.performance.tsfc > 0.0);
    for s in ["2", "3", "4", "5", "9"] {
        assert!(res.station(s).tt.to_bits() == reference.station(s).tt.to_bits());
        assert!(res.station(s).pt.to_bits() == reference.station(s).pt.to_bits());
    }
    assert!(res.performance.specific_thrust.to_bits()
            == reference.performance.specific_thrust.to_bits());
}
