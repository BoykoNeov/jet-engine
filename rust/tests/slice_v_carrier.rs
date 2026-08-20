//! SLICE V step 1b — **THE CARRIER'S OWN WITNESS**, because the phase gate cannot be one.
//!
//! Step 1b turned `TwoSpoolMapCore`'s two map fields into `Cell` and rewrote 56 read sites.
//! `cargo test --release` coming back green after that says the rewrite was behaviour-neutral —
//! which is what it is for — and says **nothing at all** about whether the carrier WORKS, because
//! until slice V step 2 nothing in the tree writes through it during a march. `set_map_lp` could
//! be `{}` and the only thing that would break is rung 53's constructor.
//!
//! That is [`slice U step 1`]'s finding on this slice's own step: *bit-exact and green says
//! nothing about GATE POWER.* So the property the carrier exists for is asserted directly here:
//!
//! > **a write made through a shared `&` reference, from a context that does NOT own the core,
//! > persists — and a downstream reader that never saw the write sees the moved map.**
//!
//! Both halves matter. The first is what `_arm` needs (it is reached with `&self` inside a hook
//! cell). The second is what § 5.20 (ii) measured at **15.4 %** on `margin_min_lp`: Python's
//! readers see the arming because the field IS the object's, and a port that scoped the mutation
//! would silently disagree with all 59 ported gates passing.
//!
//! **This file is NOT the slice's P5 gate.** P5 manufactures the local-armed-core bug against
//! rung 57's real march and asserts a value key breaks; it needs step 2's `r57_try_close` to
//! exist. This is the weaker, earlier statement — *the mechanism is live* — and it is written now
//! because a carrier installed and unwitnessed reads exactly like a carrier that works.
//!
//! [`slice U step 1`]: https://example.invalid

use turbojet::engine::FlightCondition;
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses, TwoSpoolMapCore};
use turbojet::gas::{Gas, GasSpec};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
/// Rung 57's own suite value, so the wall this moves is the one slice V is measured against.
const FLOOR: f64 = 0.55;
const V: f64 = 0.20;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

fn cpg_gas() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 0.4 / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: 0.3 / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn core() -> TwoSpoolMapCore {
    // `rung41.rs`'s spelling exactly, so the hardware is the one rungs 41/44/57 are measured on.
    let lp = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, phi_surge: FLOOR,
                            ..ComponentMap::flat() };
    let hp = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, phi_surge: FLOOR,
                            ..ComponentMap::flat() };
    let d = build_two_spool_turbojet(cpg_gas(), PI_LPC, PI_HPC, TT4, 50_000.0, real());
    TwoSpoolMapCore::new(d, flight(), 1.0, lp, hp)
}

/// Nothing here takes `&mut`. The binding is deliberately handed out as a SHARED reference and
/// the write is made through that, which is the only shape `_arm` can use.
fn arm_through_shared_ref(c: &TwoSpoolMapCore, v: f64) {
    c.set_map_lp(c.map_lp().with_vsv(v));
}

#[test]
fn the_carrier_is_live_a_shared_ref_write_persists_and_a_reader_sees_it() {
    let c = core();
    let fl = flight();

    // --- 1. the design state, captured before anything is armed ------------------------------
    let before_vsv = c.map_lp().vsv;
    let before_sm = c.surge_margin(&fl, TT4);
    assert_eq!(before_vsv, 0.0, "the core must start at the design setting");

    // --- 2. THE WRITE, through a shared `&` -- `_arm`'s shape --------------------------------
    arm_through_shared_ref(&c, V);

    // --- 3. it PERSISTED. A no-op `set_map_lp` dies here. ------------------------------------
    assert_eq!(c.map_lp().vsv.to_bits(), V.to_bits(),
               "the write through `&` did not persist -- the carrier is not live");

    // --- 4. and a DOWNSTREAM reader, which never saw the write, sees the moved map -----------
    //     This is the half that matters for § 5.20 (ii): `surge_margin` reads the field, not a
    //     parameter, so a scoped mutation would leave it at the design value.
    let after_sm = c.surge_margin(&fl, TT4);
    assert_ne!(before_sm.sm_lp.to_bits(), after_sm.sm_lp.to_bits(),
               "`surge_margin` did not see the arming -- a reader is reading a stale copy, \
                which is exactly the divergence the carrier exists to prevent");

    // --- 5. THE TWO CELLS ARE NOT ONE, and the bars are MEASURED rather than guessed --------
    //     The first writing of this test asserted the HP margin bit-for-bit ("rung 53's P5 zero,
    //     one ladder early") and it moved by **2 ULPs**. That was a guessed bar: rung 53's zero
    //     is a claim about the STEADY matcher's own lever, and here the LP arming shifts the LP
    //     work, which moves the HP operating point through the cascade. Measured instead:
    //
    //         d_lp_rel = 5.632730e-1     d_hp_rel = 3.220528e-16
    //
    //     Fifteen orders of magnitude, and THAT SEPARATION IS THE ASSERTION — if the two `Cell`s
    //     were aliased, or if `set_map_lp` wrote both, the HP margin would move like the LP one.
    //     A bare `assert_ne!` on the LP side cannot tell those apart; this can.
    let d_lp = (after_sm.sm_lp - before_sm.sm_lp).abs() / before_sm.sm_lp.abs();
    let d_hp = (after_sm.sm_hp - before_sm.sm_hp).abs() / before_sm.sm_hp.abs();
    assert!(d_lp > 1e-1, "the LP margin barely moved ({d_lp:.3e}) -- the arming is not reaching                           the reader");
    assert!(d_hp < 1e-12, "the HP margin moved {d_hp:.3e} -- that is a SOLVE-coupling size, not                            a rounding one, so the two map Cells are not independent");
    assert_eq!(c.map_hp().vsv, 0.0, "arming LP moved the HP map itself");
}

/// The write is NOT restored — Python's `_arm` has no `finally`, and `v_of` exists precisely
/// because the field is left wherever the last call put it. A carrier that restored would pass
/// the test above and still be wrong, so the persistence is asserted ACROSS a second reader.
#[test]
fn the_arming_is_never_restored() {
    let c = core();
    let fl = flight();
    arm_through_shared_ref(&c, V);
    let _ = c.surge_margin(&fl, TT4);          // a full solve between the write and the read
    let _ = c.surge_margin(&fl, 1300.0);
    assert_eq!(c.map_lp().vsv.to_bits(), V.to_bits(),
               "the arming was restored by something -- Python's `_arm` never restores");
}
