//! RUNG 45 — THE TRANSIENT TWO-SPOOL SURGE LINE ON THE FUEL PATH: a `rho`-monotone overshoot
//! that NEVER reaches the reference-free surge object.
//!
//! Port of `tests/test_rung45.py`, gate for gate. That file names **6 gates**, defines **9 test
//! functions** and collects **9 items** — no `parametrize` anywhere, so for the second file
//! running the three counts reduce to two. § 5.16 counted the pair as **20 items, 11 + 9** with
//! `--collect-only` rather than off a header, and this file is the 9.
//!
//! | # | `tests/test_rung45.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_read_only_integrate_fuel_bit_for_bit_rung43` | [`gate1_read_only_integrate_fuel_bit_for_bit_rung43`] |
//! | 1 | `test_reduce_lp_disabled_asserts_the_split_is_two_shaft` | [`gate1_lp_disabled_asserts_the_split_is_two_shaft`] |
//! | 1 | `test_cycle_untouched_by_fuel_surge_call_bit_for_bit_rung6` | [`gate1_cycle_untouched_by_fuel_surge_call_bit_for_bit_rung6`] |
//! | 2 | `test_split_survives_dominance_compresses` | [`gate2_split_survives_dominance_compresses`] |
//! | 3 | `test_headline_currency_trap_rho_monotone_plant_rho_invariant_surge` | [`gate3_currency_trap_rho_monotone_plant_rho_invariant_surge`] |
//! | 3 | `test_headline_the_trap_is_a_reference_artifact` | [`gate3_the_trap_is_a_reference_artifact`] |
//! | 4 | `test_fuel_enlarges_the_surge_approach_vs_tt4_control` | [`gate4_fuel_enlarges_the_surge_approach_vs_tt4_control`] |
//! | 5 | `test_ramp_rate_governs_faster_is_deeper` | [`gate5_ramp_rate_governs_faster_is_deeper`] |
//! | 6 | `test_report_the_crossing_gate_the_flip_fuel` | [`gate6_report_the_crossing_gate_the_flip_fuel`] |
//!
//! **This file carries TEN `#[test]` fns, not nine.** The tenth is `rung55.rs`'s roster item 5,
//! `test_cycle_untouched_transient_ladder_is_bit_for_bit_unstacked` — the last slice-M/N deferral
//! whose stated blocker was that *"`TwoSpoolFuelTransient` does not exist in Rust yet"*. It exists
//! as of slice S step 1, so the item comes due here; it lands in THIS file rather than back in
//! `rung55.rs` for slice P's reason (put the discharged item where its object lives), and it is
//! **not** a straight port — see its own doc comment.
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **`R_c` is DERIVED here and a LITERAL in `rung43.rs`.** `test_rung45.py:83` writes
//!   `(gamma_c-1)/gamma_c*cp_c = 286.8571428571428`; `test_rung43.py:62` hard-codes `286.9`. The
//!   two suites one rung apart therefore run DIFFERENT cold sections, and § 5.16 probe 1 measured
//!   that the whole 400-key fuel-path dump is bit-identical across the pair — only a THRUST key
//!   witnesses the difference at all. Slice R step 3 shipped rung 40's constant into `rung44.rs`
//!   and no gate in that file could see it, so each suite's gas is built from its OWN expression.
//! * **Six silent defaults.** `phi_excursion_fuel` and `transient_surge_margin_fuel` both default
//!   to `r = 0.5, s_settle = 6.0, ds = 0.02` (`engine.py:5345`, `:5391`), and the suite leans on
//!   the last two at every one of its call sites while naming `r` at most of them. Rust has no
//!   defaults, so every call below writes all three out.
//! * **`==` on a returned record.** Python compares whole dicts; [`PhiExcursionFuel`] has no
//!   `PartialEq`, so gate 1 compares it through [`phi_exc_bits`], which destructures it
//!   EXHAUSTIVELY — a field added to the struct stops this file compiling rather than silently
//!   narrowing the source's nine-key `==`. Bit comparison is STRICTER than Python's `==` (it
//!   separates `-0.0` from `0.0`).
//! * **`pytest.raises(AssertionError)` becomes a `catch_unwind` that reads the MESSAGE.** Both
//!   halves of gate 1's `lp_disabled` test expect *a* raise, and § 5.16's port decisions registered
//!   that the gate must assert WHICH refusal escapes — measured to matter, see that gate.
//! * **The march bound, checked before it was written.** `_fuel_ramp_march` marches to
//!   `r + s_settle`, so this suite's four ramp rates give `s_end/ds` = `7.0/0.02`, `6.5/0.02`,
//!   `6.3/0.02`, `6.1/0.02` = **350.0, 325.0, 315.0, 305.0, every one exact**. Step 1's prediction
//!   4 measured `8.25/0.02 = 412.5` splitting `round_ties_even` (412) from `f64::round` (413), so
//!   the four were computed BEFORE this file's loop bounds were written rather than after a
//!   mismatch. No tie here; the hazard is registered and does not bite.
//!
//! # The one place this file's OBJECT differs from Python's
//!
//! `test_rung45.py` builds its `lp_disabled` object from a **two-spool** design engine where
//! `test_rung43.py` builds one from `build_turbojet`, and § 5.16 step 1 booked that difference
//! here. Measured (`probe_s7`/`probe_s7c`) rather than reasoned, and the reason is sharper than
//! duck typing: **rung 45's own `SINGLE` dict has no `nozzle_convergent`**, so
//! `build_turbojet(**SINGLE)` is REFUSED by the rung-31 matcher outright — the source could not
//! have fed a single-spool engine without inventing a recipe it does not contain. Feeding the
//! two-spool one instead produces a HYBRID: `OffDesignMatcher.__init__` takes the LAST `Compressor`
//! and the LAST `Turbine` off the roster, so the held rung-35 object pairs the HPC's `pi_c = 6.0`
//! with the LPT's `eta_t = 0.9`, over a `ref` whose `2 -> 3` span covers BOTH compressors
//! (`tau_c_d` 2.48 against a single-spool 1.76, `A4`/`A8`/`P_ref` all different, `pi_c` at
//! `Tt4 = 1400` **15.4 against 5.4**).
//!
//! None of that is READ by either gated method — `phi_excursion_fuel` refuses in the eighth assert
//! opening `_fuel_ramp_march`, and `transient_surge_margin_fuel` reads only `self.map_lp/map_hp`,
//! which on the `lp_disabled` branch come from CONSTRUCTOR KWARGS and not from the design engine.
//! Measured: all three admissible feeds (two-spool, rung 45's `SINGLE` + `nozzle_convergent`, rung
//! 43's `SINGLE`) raise **byte-identical** messages at both methods. So this file feeds rung 45's
//! own `SINGLE` with `nozzle_convergent: true` added, and the added flag is disclosed HERE rather
//! than passed off as the suite's — it is the one constant in this file that is not the source's.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::{FuelLimiters, PhiExcursionFuel, TwoSpoolFuelTransient};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stage::{Split, StageStack, StageStackCore, StageStackCoreSpec, StageStackSpec};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::TwoSpoolTransientCore;

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

/// The endpoints every rung-45 gate ramps between.
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
/// The Tt4 DELTA the rung-44 comparison must be spelled with. `phi_excursion(flight, Tt4_lo,
/// dTt4, ...)` takes a STEP where `phi_excursion_fuel` takes an ENDPOINT — `test_rung45.py` writes
/// `1000.0, 400.0` against `1000.0, 1400.0`, and the two calls therefore span the SAME range.
/// Porting the `400.0` as an endpoint would give a wrong-but-plausible rung-44 ratio that every
/// sign assertion in gates 2 and 4 still passes.
const DTT4: f64 = HI - LO;

/// `phi_excursion_fuel` / `transient_surge_margin_fuel`'s silent defaults (`engine.py:5346`).
const S_SETTLE: f64 = 6.0;
const DS: f64 = 0.02;

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

/// `test_rung45.py`'s `SINGLE` — **`eta_c = 0.90`**, where `test_rung43.py`'s is `0.88`. It has NO
/// `nozzle_convergent`, which is admissible for the rung-6 cycle gate that uses it and REFUSED by
/// the matcher; see [`single_matchable`].
fn single() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.90, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: false,
    }
}

/// [`single`] with the ONE constant this file adds to the source's: `nozzle_convergent`.
///
/// Needed only to give the `lp_disabled` gate an object at all — see the header. Every other
/// number is `test_rung45.py`'s own.
fn single_matchable() -> Losses {
    Losses { nozzle_convergent: true, ..single() }
}

/// `test_rung45.py`'s `_cpg_gas` — `R_c` **DERIVED**, unlike `rung43.rs`'s literal. See the header.
fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

/// `SHAPES`, in Python's dict order. The fourth — `hp-only`, an LP map that is FLAT — is rung 40's
/// DISCRIMINATOR (no LP-map complex mode), and gate 2 sweeps all four where `rung43.rs`'s
/// `shapes()` carries only three.
fn shapes() -> [(&'static str, ComponentMap, ComponentMap); 4] {
    let f = ComponentMap::flat();
    let m = |a: f64, b: f64, c: f64, sigma: f64, l: f64| ComponentMap { a, b, c, sigma, l, ..f };
    let tilted = m(0.14, 0.10, 0.06, 0.2, 0.85);
    [
        ("flow/press", lp_shaped(), hp_shaped()),
        ("press/flow", m(0.05, 0.20, 0.0, 0.1, 1.0), m(0.20, 0.05, 0.0, 0.1, 0.7)),
        ("tilted", tilted, tilted),
        ("hp-only", f, hp_shaped()),
    ]
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

/// `test_rung45.py`'s `_ft`, non-degenerate arm. Python rebuilds `_design(gas)` per call; the build
/// is a pure function of its arguments, so a clone carries the same numbers — `rung43.rs`'s shape,
/// and the same honest caveat: where the source hands ONE design object to two objects so a
/// MUTATION would surface, cloning severs that channel and leaves "same inputs, same numbers".
fn ft(d: &TwoSpoolEngine, map_lp: ComponentMap, map_hp: ComponentMap, rho: f64)
    -> TwoSpoolFuelTransient
{
    TwoSpoolFuelTransient::new(d.clone(), flight(), 1.0, map_lp, map_hp, rho)
}

/// `_floor(cm, phi_surge)`.
fn floor(cm: ComponentMap, phi_surge: f64) -> ComponentMap {
    cm.with_phi_surge(phi_surge)
}

/// The nine fields of [`PhiExcursionFuel`], as raw bits, by EXHAUSTIVE destructure.
///
/// Python's gate 1 compares two whole dicts with `==`. A hand-written six-field comparison would
/// narrow that silently; destructuring without `..` means a tenth field breaks the build instead.
fn phi_exc_bits(e: &PhiExcursionFuel) -> [u64; 9] {
    let PhiExcursionFuel { ext_lp, ext_hp, s_lp, s_hp, min_phi_lp, min_phi_hp, tt4_peak, ratio,
                           npts } = *e;
    [ext_lp.to_bits(), ext_hp.to_bits(), s_lp.to_bits(), s_hp.to_bits(), min_phi_lp.to_bits(),
     min_phi_hp.to_bits(), tt4_peak.to_bits(), ratio.to_bits(), npts as u64]
}

/// The message of an `assert!` that fired, or `None` if the call returned.
///
/// **This swaps the GLOBAL panic hook**, where `rung44.rs` calls `catch_unwind` and leaves the
/// hook alone — a divergence recorded rather than repaired. Two tests in this file call it and
/// cargo runs them on parallel threads, so the restore can race; it cannot change a `catch_unwind`
/// RESULT, only interleave the suppressed backtrace output of an unrelated failing test. The
/// silencing is what buys the ability to assert WHICH refusal escaped, which § 5.16 registered as
/// a port decision and finding 1 measured to matter.
fn refusal<F: FnOnce()>(f: F) -> Option<String> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(hook);
    match r {
        Ok(()) => None,
        Err(e) => Some(
            e.downcast_ref::<String>().cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "<non-string panic>".to_string()),
        ),
    }
}

fn spread(v: &[f64]) -> f64 {
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
     - v.iter().cloned().fold(f64::INFINITY, f64::min)) / mean.abs()
}

// ================================================================================== gate 1
/// GATE 1/read-only — arming `phi_surge` leaves rung 43's `integrate_fuel` / `equilibrium_fuel` /
/// `phi_excursion_fuel` bit-for-bit. Rung 45 adds NO state; it only READS a surge line that the
/// rung-41 reduce, two rungs on, says the march must never touch.
#[test]
fn gate1_read_only_integrate_fuel_bit_for_bit_rung43() {
    let d = design(cpg_gas());
    let [fp, _, ti, _] = shapes();
    for (name, ml, mh) in [fp, ti] {
        let bare = ft(&d, ml, mh, 1.5);
        let armed = ft(&d, floor(ml, 0.60), floor(mh, 0.55), 1.5);
        let (bc, ac) = (bare.core(), armed.core());
        let mf0 = bc.fuel_for_tt4(&flight(), 1000.0);
        let mf1 = bc.fuel_for_tt4(&flight(), 1200.0);
        let eq0 = bc.inner.equilibrium(&flight(), 1000.0);
        let nu0 = (eq0.nu_lp, eq0.nu_hp);
        let sched = |s: f64| mf0 + (mf1 - mf0) * (s / 0.5).min(1.0);

        // s_end = 2.0, ds = 0.05 -> 40.0 exactly; see the header's march-bound note.
        let lim = FuelLimiters::default();
        let pa = bc.integrate_fuel(&flight(), sched, nu0, 2.0, 0.05, &lim);
        let pb = ac.integrate_fuel(&flight(), sched, nu0, 2.0, 0.05, &lim);
        assert_eq!(pa.len(), pb.len(), "{name}: point counts");
        for (a, b) in pa.iter().zip(pb.iter()) {
            assert_eq!(
                (a.nu_lp.to_bits(), a.nu_hp.to_bits(), a.phi_lp.to_bits(), a.phi_hp.to_bits(),
                 a.tt4.to_bits(), a.f.to_bits()),
                (b.nu_lp.to_bits(), b.nu_hp.to_bits(), b.phi_lp.to_bits(), b.phi_hp.to_bits(),
                 b.tt4.to_bits(), b.f.to_bits()),
                "{name}: armed march diverged at s = {}", a.s);
        }
        for mf in [mf0, mf1] {
            // `FuelInstant` has no `PartialEq`; `Instant2` does, and the one extra field is
            // compared on its bits.
            let (a, _) = bc.equilibrium_fuel(&flight(), mf, None);
            let (b, _) = ac.equilibrium_fuel(&flight(), mf, None);
            assert!(a.base == b.base && a.mdot_air_face.to_bits() == b.mdot_air_face.to_bits(),
                    "{name}: equilibrium_fuel moved when the surge line was armed");
        }
        // the referenced excursion never reads phi_surge => identical armed vs bare
        let ea = bare.phi_excursion_fuel(&flight(), 1000.0, 1300.0, 0.5, S_SETTLE, DS,
                                         None, None, None, None);
        let eb = armed.phi_excursion_fuel(&flight(), 1000.0, 1300.0, 0.5, S_SETTLE, DS,
                                          None, None, None, None);
        assert_eq!(phi_exc_bits(&ea), phi_exc_bits(&eb),
                   "{name}: phi_excursion_fuel read the surge line");
    }
}

/// GATE 1/`lp_disabled` — the fuel-surge SPLIT is inherently two-shaft (rung 44's contract), so
/// `lp_disabled` is not a reduce axis for it and BOTH methods refuse on the degenerate engine.
///
/// **THE GATE ASSERTS WHICH REFUSAL ESCAPES, AND THE SOURCE'S OWN SECOND OBJECT IS WHY.**
/// `test_rung45.py` builds the second object with `_floor(...)` on BOTH maps — armed — where the
/// first is bare, and only `pytest.raises(AssertionError)` is asserted either way. Measured
/// (`probe_s7b`): `transient_surge_margin_fuel` reads its surge-line assert FIRST, so on an
/// UNARMED degenerate object it raises *"needs a surge line on BOTH maps"* and the two-shaft
/// refusal is never reached. The source arms deliberately to get past it; had it not, that half of
/// the gate would have passed having tested a completely different assert. *The project's own
/// "a gate whose expected result is a raise passes when everything raises" landing in the SOURCE.*
///
/// **AND RUST REVERSES THAT ORDER, IN THE ONE CASE NO GATE REACHES.** The refusal lives on the
/// enum here (`matches!(self, Full)`) because the `Degenerate` variant holds a `SpoolTransient`
/// and no `map_lp`/`map_hp` at all — so a BARE-map `transient_surge_margin_fuel` raises the
/// TWO-SHAFT refusal in Rust where Python raises the SURGE-LINE one. Both inputs the source
/// exercises agree; the divergence is disclosed rather than repaired, because repairing it means
/// carrying two maps on a variant that has no use for them.
#[test]
fn gate1_lp_disabled_asserts_the_split_is_two_shaft() {
    let gas = cpg_gas();
    let single_design = build_turbojet(gas.clone(), PI_HPC, TT4, 50_000.0, single_matchable());
    let deg = TwoSpoolFuelTransient::lp_disabled(single_design.clone(), flight(), 1.0, hp_shaped());

    let m = refusal(|| {
        deg.phi_excursion_fuel(&flight(), LO, HI, 0.5, S_SETTLE, DS, None, None, None, None);
    })
    .expect("phi_excursion_fuel must refuse the degenerate engine");
    assert!(m.contains("inherently two-shaft") && m.contains("not a reduce axis"),
            "the TWO-SHAFT refusal must be the one that escapes, got: {m}");

    // Python's second object arms BOTH maps; here the maps live on `Full` only, so the armed
    // half is spelled by arming the map the degenerate constructor does take.
    let deg2 = TwoSpoolFuelTransient::lp_disabled(
        single_design, flight(), 1.0, floor(hp_shaped(), 0.55));
    let m2 = refusal(|| {
        deg2.transient_surge_margin_fuel(&flight(), LO, HI, 0.5, S_SETTLE, DS,
                                         None, None, None, None);
    })
    .expect("transient_surge_margin_fuel must refuse the degenerate engine");
    assert!(m2.contains("inherently two-shaft") && m2.contains("not a reduce axis"),
            "the TWO-SHAFT refusal must be the one that escapes, got: {m2}");

    // and the refusal is a REFUSAL, not everything raising: the same call on a FULL object with
    // an armed surge line returns.
    let d = design(cpg_gas());
    let full = ft(&d, floor(lp_shaped(), 0.60), floor(hp_shaped(), 0.55), 1.0);
    assert!(refusal(|| {
        full.transient_surge_margin_fuel(&flight(), LO, HI, 0.5, S_SETTLE, DS,
                                         None, None, None, None);
    })
    .is_none(), "a FULL armed object must be ADMITTED");
}

/// GATE 1/cycle — the default single-spool design path is bit-for-bit rung 6: constructing and
/// exercising the rung-45 diagnostics must not perturb it.
///
/// This is the one gate that uses `test_rung45.py`'s `SINGLE` VERBATIM — no matcher is built from
/// it, so the missing `nozzle_convergent` is admissible exactly here.
///
/// **AND IT IS THE ONE TEST IN THIS FILE THAT STEP 3's INJECTIONS DO NOT REACH.** Thirteen
/// injections gave the other nine teeth; not one of them could fire this gate, because its channel
/// is `engine.rs`'s design run and nothing in `fuel_transient.rs` can perturb a single-spool cycle.
/// Said out loud rather than left to a 14-row table beside 10 tests: step 1's own finding is that
/// covering SOME of a set is the same defect as a partition sum covering an arm. What this gate
/// carries is the project-wide rung-6 invariant, and its teeth are the same as every other file's
/// copy of it.
#[test]
fn gate1_cycle_untouched_by_fuel_surge_call_bit_for_bit_rung6() {
    let eng: Engine = build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, single());
    let a = eng.run(&flight(), 1.0);
    let d = design(cpg_gas());
    let t = ft(&d, floor(lp_shaped(), 0.60), floor(hp_shaped(), 0.55), 1.0);
    t.phi_excursion_fuel(&flight(), 1000.0, 1300.0, 0.5, S_SETTLE, DS, None, None, None, None);
    t.transient_surge_margin_fuel(&flight(), 1000.0, 1300.0, 0.5, S_SETTLE, DS,
                                  None, None, None, None);
    let b = eng.run(&flight(), 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
}

// ================================================================================== gate 2
/// GATE 2 — THE SPLIT SURVIVES, THE DOMINANCE COMPRESSES.
///
/// Accel drives both spools TOWARD surge (`ext < 0`), decel is the mirror (`ext > 0`), and the LP
/// LEADS at every shape including the mode-free `hp-only`. The dominance COMPRESSES against rung
/// 44: at every shape the fuel-path ratio is BELOW the Tt4-path ratio on the SAME maps, because
/// the Tt4 overshoot loads the HP transient lag. A shape-matched RELATIVE comparison, not a bare
/// magnitude threshold — the strong LP asymmetry lives on the raw margin (gate 6).
#[test]
fn gate2_split_survives_dominance_compresses() {
    let d = design(cpg_gas());
    for (name, ml, mh) in shapes() {
        let t = ft(&d, ml, mh, 1.0);
        let tt = TwoSpoolTransientCore::new(d.clone(), flight(), 1.0, ml, mh, 1.0);
        let acc = t.phi_excursion_fuel(&flight(), LO, HI, 0.5, S_SETTLE, DS, None, None, None, None);
        let dec = t.phi_excursion_fuel(&flight(), HI, LO, 0.5, S_SETTLE, DS, None, None, None, None);
        // RUNG 44, same maps: a DELTA, and rung 44's own defaults `s_end = 3.0`.
        let tt4 = tt.phi_excursion(&flight(), LO, DTT4, 0.5, 3.0, DS);

        assert!(acc.ext_lp < 0.0 && acc.ext_hp < 0.0, "{name}: accel toward surge, {acc:?}");
        assert!(dec.ext_lp > 0.0 && dec.ext_hp > 0.0, "{name}: decel away (mirror), {dec:?}");
        assert!(acc.ext_lp.abs() > acc.ext_hp.abs(), "{name}: LP leads, {acc:?}");
        let fuel_ratio = acc.ext_lp.abs() / acc.ext_hp.abs();
        let tt4_ratio = tt4.ext_lp.abs() / tt4.ext_hp.abs();
        assert!(fuel_ratio < tt4_ratio,
                "{name}: fuel-path dominance must COMPRESS vs rung 44 — {fuel_ratio} vs {tt4_ratio}");
    }
}

// ================================================================================== gate 3
/// GATE 3/(b) — THE LOAD-BEARING FINDING. Over `rho` in `[0.2, 5.0]` (25x) the Tt4 OVERSHOOT (the
/// PLANT) is strongly `rho`-MONOTONE (> 5 %, rung 43), yet the reference-free surge object (the raw
/// transient min `phi`) is `rho`-INVARIANT (< 2 %, rung 44's own bar). The plant's `rho` signal does
/// not reach the surge margin — NOT decoupled, an order weaker — so rung 44's "`rho` powerless over
/// surge" SURVIVES the control swap on the reference-free object.
#[test]
fn gate3_currency_trap_rho_monotone_plant_rho_invariant_surge() {
    let d = design(cpg_gas());
    let (mut mins, mut peaks) = (Vec::new(), Vec::new());
    for rho in [0.2f64, 0.5, 1.0, 2.0, 5.0] {
        let t = ft(&d, lp_shaped(), hp_shaped(), rho);
        let e = t.phi_excursion_fuel(&flight(), LO, HI, 0.5, S_SETTLE, DS, None, None, None, None);
        mins.push(e.min_phi_lp);
        peaks.push(e.tt4_peak);
    }
    let (ms, ps) = (spread(&mins), spread(&peaks));
    assert!(ms < 0.02, "surge object must be rho-invariant: {mins:?} spread {ms}");
    assert!(ps > 0.05, "the plant (Tt4 overshoot) IS rho-monotone: {peaks:?} spread {ps}");
    assert!(ps > 5.0 * ms, "the plant signal must not reach the surge object: {ps} vs {ms}");
}

/// GATE 3/(a) — THE TRAP ITSELF, same-currency (`phi`). Over the SAME `rho` sweep the REFERENCE-FREE
/// object is `rho`-invariant (< 2 %) while the OUTPUT-Tt4-referenced excursion — the naive choice,
/// which folds rung 43's `rho`-monotone overshoot into a MOVING baseline — swings > 20 %. The more
/// the reference tracks the overshoot, the more `rho` leaks in: reference-free quietest, the
/// shipped COMMANDED-ramp excursion intermediate (NOT claimed `rho`-flat), the output loudest.
///
/// Python's test writes its OWN `interp` inline rather than calling `_interp`, and so does this —
/// the duplication is kept rather than factored onto `FuelTransientCore::interp`, because a
/// reference that calls the shipped helper cannot witness a defect in the shipped helper.
#[test]
fn gate3_the_trap_is_a_reference_artifact() {
    let d = design(cpg_gas());
    // A wide running-line grid spanning the OUTPUT Tt4 range (the overshoot reaches ~1780 K).
    let tg = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let grid: Vec<f64> = (0..19).map(|k| 1000.0 + 50.0 * k as f64).collect();
    let ys_l: Vec<f64> =
        grid.iter().map(|&t| tg.core().inner.equilibrium(&flight(), t).close.phi_lp).collect();

    let interp = |x: f64| -> f64 {
        if x <= grid[0] {
            return ys_l[0];
        }
        if x >= grid[grid.len() - 1] {
            return ys_l[ys_l.len() - 1];
        }
        for i in 0..grid.len() - 1 {
            if grid[i] <= x && x <= grid[i + 1] {
                let t = (x - grid[i]) / (grid[i + 1] - grid[i]);
                return ys_l[i] + t * (ys_l[i + 1] - ys_l[i]);
            }
        }
        ys_l[ys_l.len() - 1]
    };

    let (mut cmd_ext, mut out_ext, mut raw_min) = (Vec::new(), Vec::new(), Vec::new());
    for rho in [0.2f64, 1.0, 5.0] {
        let t = ft(&d, lp_shaped(), hp_shaped(), rho);
        let c = t.core();
        let e = t.phi_excursion_fuel(&flight(), LO, HI, 0.5, S_SETTLE, DS, None, None, None, None);
        cmd_ext.push(e.ext_lp); // COMMANDED-referenced (shipped)
        raw_min.push(e.min_phi_lp);

        // OUTPUT-referenced excursion, from an equivalent march: reference to phi_steady(Tt4_OUT).
        let mf_lo = c.fuel_for_tt4(&flight(), LO);
        let mf_hi = c.fuel_for_tt4(&flight(), HI);
        let eq0 = c.inner.equilibrium(&flight(), LO);
        let sched = |s: f64| mf_lo + (mf_hi - mf_lo) * (s / 0.5).min(1.0);
        // s_end = 6.5, ds = 0.02 -> 325.0 exactly; see the header's march-bound note.
        let mut oe = 0.0f64;
        for p in c.integrate_fuel(&flight(), sched, (eq0.nu_lp, eq0.nu_hp), 6.5, 0.02,
                                  &FuelLimiters::default())
        {
            let e_lp = p.phi_lp - interp(p.tt4);
            if e_lp.abs() > oe.abs() {
                oe = e_lp;
            }
        }
        out_ext.push(oe);
    }

    let (sr, sc, so) = (spread(&raw_min), spread(&cmd_ext), spread(&out_ext));
    assert!(sr < 0.02, "raw min phi must be rho-invariant: {raw_min:?} spread {sr}");
    assert!(so > 0.20, "output-ref must swing (THE TRAP): {out_ext:?} spread {so}");
    assert!(so > sc && sc > sr,
            "reference ordering: output > commanded > reference-free, got {so} {sc} {sr}");
}

// ================================================================================== gate 4
/// GATE 4 — FUEL ENLARGES the surge approach (rung 35 on two shafts). At the SAME endpoints and the
/// SAME ramp rate, fuel control drives the raw transient min `phi` DEEPER toward surge than Tt4
/// control does: the overshoot amplifies the approach. Rung 35's "the two accel limits are
/// coupled", now on two shafts. SIGN only.
#[test]
fn gate4_fuel_enlarges_the_surge_approach_vs_tt4_control() {
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let tt = TwoSpoolTransientCore::new(d.clone(), flight(), 1.0, lp_shaped(), hp_shaped(), 1.0);
    for r in [1.0f64, 0.5, 0.3] {
        let fuel_min = t
            .phi_excursion_fuel(&flight(), LO, HI, r, S_SETTLE, DS, None, None, None, None)
            .min_phi_lp;
        // RUNG 44: a DELTA again, and rung 44's own `s_end = 3.0` default.
        let tt4_min = tt.phi_excursion(&flight(), LO, DTT4, r, 3.0, DS).min_phi_lp;
        assert!(fuel_min < tt4_min,
                "fuel must dip deeper toward surge at r = {r}: {fuel_min} vs {tt4_min}");
    }
}

// ================================================================================== gate 5
/// GATE 5 — RAMP-RATE GOVERNS. The raw transient min `phi_lp` dips monotonically DEEPER as the fuel
/// ramp gets faster: the schedule against the shaft clock is the governing variable, surviving the
/// control swap. Reference-free, so immune to the currency trap.
#[test]
fn gate5_ramp_rate_governs_faster_is_deeper() {
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let mut prev: Option<f64> = None;
    // s_end = r + 6.0 over these four -> 350.0 / 325.0 / 315.0 / 305.0 steps, all exact.
    for r in [1.0f64, 0.5, 0.3, 0.1] {
        let m = t
            .phi_excursion_fuel(&flight(), LO, HI, r, S_SETTLE, DS, None, None, None, None)
            .min_phi_lp;
        if let Some(p) = prev {
            assert!(m < p, "faster ramp must go deeper toward surge at r = {r}: {m} vs {p}");
        }
        prev = Some(m);
    }
}

// ================================================================================== gate 6
/// GATE 6 — REPORT THE CROSSING, GATE THE FLIP (rung 36's discipline), on the ACCEL only.
///
/// `transient_surge_margin_fuel` ALLOWS `phi < phi_surge` and RECORDS it. On an accel the raw
/// transient min LP margin sits BELOW the commanded steady min LP margin (the flip); with a floor
/// placed in the gap the LP crosses while every steady point clears, and it lands on the LP spool.
/// The flip's SIGN is gated; the crossing DEPTH is disclaimed.
///
/// Only the accel: a decel moves AWAY from surge, so the raw min `phi` relaxes onto the low-power
/// steady point and the raw margin is degenerate there. The decel MIRROR lives on the referenced
/// excursion (gate 2's `dec`).
///
/// **The floor 0.746 is HAND-TUNED between the transient min (~0.719) and the steady min (~0.773)**,
/// which makes this the gate in the file most sensitive to any drift — so it was the first one run
/// against the port rather than the last.
#[test]
fn gate6_report_the_crossing_gate_the_flip_fuel() {
    let d = design(cpg_gas());
    let (ml, mh) = (floor(lp_shaped(), 0.746), floor(hp_shaped(), 0.55));
    let t = ft(&d, ml, mh, 1.0);

    let acc = t.transient_surge_margin_fuel(&flight(), LO, HI, 0.3, S_SETTLE, DS,
                                            None, None, None, None);
    assert!(acc.margin_min_lp < acc.steady_min_lp, "accel flip (LP toward surge): {acc:?}");
    assert!(acc.steady_min_lp > 0.0, "steady must CLEAR the floor: {acc:?}");
    assert!(acc.crossed_lp && !acc.crossed_hp,
            "the transient crossing must land on the LP spool: {acc:?}");

    // unarmed maps => the method refuses (the surge line is genuinely off when absent)
    let bare = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let m = refusal(|| {
        bare.transient_surge_margin_fuel(&flight(), LO, HI, 0.5, S_SETTLE, DS,
                                         None, None, None, None);
    })
    .expect("an unarmed FULL object must refuse");
    assert!(m.contains("needs a surge line on BOTH maps"),
            "the SURGE-LINE refusal must be the one that escapes, got: {m}");
}

// ============================================================== rung 55's roster item 5
/// `rung55.rs`'s roster item **5**, DISCHARGED — and **not** as a straight port.
///
/// Python's `test_cycle_untouched_transient_ladder_is_bit_for_bit_unstacked` runs a rung-43 fuel
/// transient, builds a K=8 stacked matcher on the SAME design object, matches it, and demands the
/// second march equal the first. Its content is *absence of leakage from the stack into the
/// transient closures*. Its MECHANISM is shared mutable state: one Python design object reaching
/// both, so a write by the matcher would be visible to the transient.
///
/// **THAT MECHANISM DOES NOT EXIST HERE, so the straight port would be VACUOUS** —
/// `StageStackCoreSpec::new` and `TwoSpoolFuelTransient::new` each take a `TwoSpoolEngine` BY
/// VALUE, so neither can see the other's copy, and `before == after` could not fail whatever the
/// closures read. That is `rust-port-ported-test-vacuity` exactly, and step 2's finding 5 is the
/// precedent for saying so rather than letting a green tick imply the Python claim.
///
/// So the gate is REBUILT to assert the content directly and MORE strongly than Python's: a live
/// K=8 [`StageStack`] is INJECTED into the fuel transient's own map core — the very slot rung 55
/// writes, `TwoSpoolMapCore::stack_lp`/`stack_hp` — and the march is demanded bit-identical. Python
/// asks *"did the matcher write on the transient's object?"*; this asks *"would the transient's
/// closures READ a stack if one were sitting in the slot?"*, which is the question the scope
/// boundary is actually about.
///
/// **THE INJECTION IS SHOWN TO EXPRESS ITSELF** before its zero is read — slice R step 3's rule,
/// and step 2's finding 3 one step on. Two ways: the slot is asserted non-`None` with `K == 8`
/// after the write, AND the same K=8 stack is shown to MOVE a result that does read it (a stacked
/// matcher's matched point against the K=1 one on the same hardware). Without the second, "nothing
/// moved" would be consistent with the stack being inert everywhere.
#[test]
fn rung55_item5_transient_ladder_is_bit_for_bit_unstacked() {
    let gas = cpg_gas();
    let d = design(gas);
    // Rung 55's own armed shapes for this gate — `_maps()` = `flow/press` at `test_rung55.py`'s
    // own `FLOOR = 0.55`, not this file's floors. The value cannot matter (gate 1 above is what
    // establishes that the march never reads `phi_surge`), but a ported gate takes the source's
    // constant rather than a neighbouring one — slice R step 3's `R_c = 286.9`, one slice on.
    let (ml, mh) = (floor(lp_shaped(), 0.55), floor(hp_shaped(), 0.55));

    let march = |t: &TwoSpoolFuelTransient| -> Vec<(u64, u64, u64, u64)> {
        let c = t.core();
        let mf0 = c.fuel_for_tt4(&flight(), 1000.0);
        let mf1 = c.fuel_for_tt4(&flight(), 1400.0);
        let eq0 = c.inner.equilibrium(&flight(), 1000.0);
        let sched = |s: f64| mf0 + (mf1 - mf0) * (s / 0.5).min(1.0);
        // s_end = 2.0, ds = 0.01 -> 200.0 exactly.
        c.integrate_fuel(&flight(), sched, (eq0.nu_lp, eq0.nu_hp), 2.0, 0.01,
                         &FuelLimiters::default())
            .iter()
            .map(|p| (p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.tt4.to_bits()))
            .collect()
    };

    let plain = ft(&d, ml, mh, 1.0);
    let before = march(&plain);

    // --- the injection EXPRESSES ITSELF: a K=8 stack moves a reader that reads it -------------
    let spec = |k: usize| StageStackCoreSpec {
        k_lp: k, k_hp: k, split: Split::DT,
        ..StageStackCoreSpec::new(d.clone(), flight(), 1.0, ml, mh)
    };
    let m1 = StageStackCore::new(spec(1));
    let m8 = StageStackCore::new(spec(8));
    assert!(m8.core.core.stack_lp.is_some(), "the K=8 matcher must actually carry a stack");
    let (r1, r8) = (m1.match_point(&flight(), 1000.0), m8.match_point(&flight(), 1000.0));
    assert!(r1.eta_lpc.to_bits() != r8.eta_lpc.to_bits(),
            "a live K=8 stack must MOVE a reader that reads it, else every zero below is vacuous");

    // --- now inject that same live stack into the fuel transient's OWN map core ---------------
    let mut stacked = ft(&d, ml, mh, 1.0);
    {
        let c = stacked.core_mut();
        let g = c.inner.inner.gas().gamma_c();
        let kc = g / (g - 1.0);
        let (tau_lpc_d, tau_hpc_d) = (c.inner.inner.tau_lpc_d, c.inner.inner.tau_hpc_d);
        let (pi_lpc_d, pi_hpc_d) =
            (c.inner.inner.base.pi_lpc_design, c.inner.inner.base.pi_hpc_design);
        let (eta_lpc, eta_hpc) = (c.inner.inner.base.eta_lpc, c.inner.inner.base.eta_hpc);
        c.inner.inner.stack_lp = Some(StageStack::new(StageStackSpec {
            kc, split: Split::DT,
            ..StageStackSpec::new(8, ml, tau_lpc_d, pi_lpc_d, eta_lpc)
        }));
        c.inner.inner.stack_hp = Some(StageStack::new(StageStackSpec {
            kc, split: Split::DT,
            ..StageStackSpec::new(8, mh, tau_hpc_d, pi_hpc_d, eta_hpc)
        }));
    }
    assert!(stacked.core().inner.inner.stack_lp.as_ref().is_some_and(|s| s.k == 8)
            && stacked.core().inner.inner.stack_hp.as_ref().is_some_and(|s| s.k == 8),
            "the stack must actually be live on the transient's own core");

    let after = march(&stacked);
    assert_eq!(before, after,
               "rung-55 SCOPE VIOLATION: a live stage stack changed a rung-43 transient result. \
                The transient closures must stay on the lumped loading law \
                (docs/rung55-spec.md § Scope).");
}
