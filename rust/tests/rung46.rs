//! RUNG 46 — THE TIT TOPPING GOVERNOR: relief SPLITS by spool, because the two accel limits are
//! coupled in CAUSE but SEQUENCED in time.
//!
//! Port of `tests/test_rung46.py`, gate for gate. That file names **8 gates**, defines **6 test
//! functions** and collects **6 items** — no `parametrize` — because its gates 3+4+5 share one
//! function and so do 1 and 2's halves. § 5.17 counted the slice as **31 items, 6 + 9 + 16** with
//! `--collect-only` rather than off a header, and this file is the 6.
//!
//! | # | `tests/test_rung46.py` | here |
//! |---|---|---|
//! | 1 | `test_reduce_dormant_bit_for_bit_rung45` | [`gate1_dormant_governor_is_bit_for_bit_rung45`] |
//! | 2 | `test_reduce_lp_disabled_asserts_the_split_is_two_shaft` | [`gate2_lp_disabled_asserts_the_split_is_two_shaft`] |
//! | 3+4+5 | `test_governor_holds_and_the_surge_relief_split` | [`gates345_governor_holds_and_the_surge_relief_split`] |
//! | 6 | `test_the_lever_fast_ramp_switches_on_lp_relief` | [`gate6_the_lever_fast_ramp_switches_on_lp_relief`] |
//! | 7 | `test_decel_bit_for_bit_rung45` | [`gate7_decel_is_bit_for_bit_rung45`] |
//! | 8 | `test_cycle_untouched_by_topping_governor_bit_for_bit_rung6` | [`gate8_cycle_untouched_bit_for_bit_rung6`] |
//!
//! # Where Rust has to say out loud what Python leaves implicit
//!
//! * **`SETTLE = 2.0`, not rung 45's `6.0`.** `test_rung46.py:80` shortens the settle because the
//!   surge minimum and the `Tt4` peak both live inside the ramp. Every call here writes it out;
//!   `topping_relief` also defaults `r = 0.5` and `ds = 0.02` (`engine.py:5438`), which the suite
//!   leans on at gate 2 and names everywhere else.
//! * **TWO GASES, AND THE SPLIT IS NOT COSMETIC.** Gates 1, 7 and 8 run the CPG gas; gates 3-6 run
//!   `Gas::thermally_perfect()`. § 5.17's probes measured the reader grid on CPG, so the gate cells
//!   were re-measured on TPG before this file was written — same shape (`held` slack 4.1e-12 to
//!   8.6e-12 against a `1e-6` bar, `relief_lp` exactly `0.0` at every shape) — rather than assumed
//!   from the CPG numbers. *A census is a property of the grid*, which this port has paid for
//!   before.
//! * **`==` on a returned record.** Gates 1 and 7 compare whole [`PhiExcursionFuel`] dicts with
//!   Python's `==`; the struct has no `PartialEq`, so the comparison goes through
//!   [`phi_exc_bits`], which destructures EXHAUSTIVELY — a tenth field breaks the build instead of
//!   silently narrowing a nine-key `==`. Bit comparison is STRICTER than `==` (it separates `-0.0`
//!   from `0.0`).
//! * **`pytest.raises(AssertionError)` becomes a `catch_unwind` that reads the MESSAGE.** Gate 2
//!   expects *a* raise from two different entry points, and rung 45's precedent is that the gate
//!   must assert WHICH refusal escaped — measured to matter there, so it is spelled the same way
//!   here.
//! * **The governor's refusal is hoisted one level.** Python reaches gate 2's first assert inside
//!   `phi_excursion_fuel`, two calls below `topping_relief`; Rust refuses in
//!   [`TwoSpoolFuelTransient::topping_relief`] itself so the message names the method the caller
//!   invoked. The gate asserts the two-shaft wording, which both spellings carry.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::{FuelLimiters, PhiExcursionFuel, TwoSpoolFuelTransient};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

/// The accel band, and a redline in the GAP — above the 1400 endpoint, below the bare peak.
///
/// **THE SUITE'S OWN COMMENT PUTS THAT PEAK AT `~1645` AND RUNG 47's PUTS IT AT `~1670`, ON A
/// BYTE-IDENTICAL GRID.** § 5.17 finding 6 measured it: 1690.5 / 1695.4 / 1702.4 / 1703.0 over the
/// four shapes. No gate reads the figure — 1480 clears every peak by more than 200 K — so this is
/// a doc correction carried here and booked to step 4, not a bar.
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const REDLINE: f64 = 1480.0;
const R: f64 = 0.5;
/// `test_rung46.py:80`. Rung 45 settles for `6.0`; this suite needs only `2.0`.
const SETTLE: f64 = 2.0;
/// `topping_relief` / `phi_excursion_fuel`'s silent `ds` default (`engine.py:5439`, `:5346`).
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

/// `test_rung46.py`'s `SINGLE` — `eta_c = 0.90`, and NO `nozzle_convergent`, which is admissible
/// for the rung-6 cycle gate that is its only consumer here.
fn single() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.90, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: false,
    }
}

/// [`single`] plus the one constant this file adds, so gate 2 has a degenerate object at all.
fn single_matchable() -> Losses {
    Losses { nozzle_convergent: true, ..single() }
}

/// `test_rung46.py`'s `_cpg_gas` — `R_c` DERIVED, as in `test_rung45.py` and unlike
/// `test_rung43.py`'s literal `286.9`.
fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

/// `test_rung46.py`'s `_tpg`. Gates 3-6 run on THIS, not on [`cpg_gas`] — see the header.
fn tpg() -> Gas {
    Gas::thermally_perfect()
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

/// `SHAPES`, in Python's dict order — the same four as rung 45's, `hp-only` last.
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

fn ft(d: &TwoSpoolEngine, map_lp: ComponentMap, map_hp: ComponentMap, rho: f64)
    -> TwoSpoolFuelTransient
{
    TwoSpoolFuelTransient::new(d.clone(), flight(), 1.0, map_lp, map_hp, rho)
}

/// The nine fields of [`PhiExcursionFuel`], as raw bits, by EXHAUSTIVE destructure — rung 45's
/// helper, repeated rather than shared because integration-test crates do not link to each other.
fn phi_exc_bits(e: &PhiExcursionFuel) -> [u64; 9] {
    let PhiExcursionFuel { ext_lp, ext_hp, s_lp, s_hp, min_phi_lp, min_phi_hp, tt4_peak, ratio,
                           npts } = *e;
    [ext_lp.to_bits(), ext_hp.to_bits(), s_lp.to_bits(), s_hp.to_bits(), min_phi_lp.to_bits(),
     min_phi_hp.to_bits(), tt4_peak.to_bits(), ratio.to_bits(), npts as u64]
}

/// The message of an `assert!` that fired, or `None` if the call returned. Rung 45's helper, and
/// its caveat travels with it: this swaps the GLOBAL panic hook, so the restore can race a
/// parallel test's backtrace output — it cannot change a `catch_unwind` RESULT.
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

// ================================================================================== gate 1
/// GATE 1 — REDUCE: a DORMANT governor is bit-for-bit rung 45/43.
///
/// A redline above the bare peak leaves the clip un-consulted, so the topped march is the bare
/// rung-43 march float-for-float and the rung-45 referenced excursion is identical armed-vs-bare
/// (it never reads the redline). `Tt4_max = None` is the same claim by a different route.
///
/// **The second half compares SIX keys per point, not the whole point**, because that is what
/// Python compares; `mf` and `mf_sched` are deliberately outside the tuple there and stay outside
/// here.
#[test]
fn gate1_dormant_governor_is_bit_for_bit_rung45() {
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let f = flight();
    let bare = t.phi_excursion_fuel(&f, LO, HI, R, SETTLE, DS, None, None, None, None);
    let huge = bare.tt4_peak + 500.0;
    let armed = t.phi_excursion_fuel(&f, LO, HI, R, SETTLE, DS, Some(huge), None, None, None);
    assert_eq!(phi_exc_bits(&armed), phi_exc_bits(&bare),
               "a redline above the bare peak must leave the excursion untouched");

    let core = t.core();
    let (mf0, mf1) = (core.fuel_for_tt4(&f, LO), core.fuel_for_tt4(&f, HI));
    let eq0 = core.inner.equilibrium(&f, LO);
    let nu0 = (eq0.nu_lp, eq0.nu_hp);
    let sched = |s: f64| mf0 + (mf1 - mf0) * (s / R).min(1.0);

    let bare_lim = FuelLimiters::default();
    let dormant = FuelLimiters { tt4_max: Some(huge), ..Default::default() };
    let none = FuelLimiters { tt4_max: None, ..Default::default() };
    let pa = core.integrate_fuel(&f, sched, nu0, R + SETTLE, DS, &bare_lim);
    let pb = core.integrate_fuel(&f, sched, nu0, R + SETTLE, DS, &dormant);
    let pc = core.integrate_fuel(&f, sched, nu0, R + SETTLE, DS, &none);
    assert_eq!(pa.len(), pb.len());
    assert_eq!(pa.len(), pc.len());
    // Python's six-key tuple, in Python's order.
    let keys = |p: &turbojet::fuel_transient::FuelPoint| {
        [p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(), p.phi_hp.to_bits(),
         p.tt4.to_bits(), p.f.to_bits()]
    };
    for ((a, b), c) in pa.iter().zip(pb.iter()).zip(pc.iter()) {
        assert_eq!(keys(a), keys(b), "dormant redline at s={}", a.s);
        assert_eq!(keys(a), keys(c), "Tt4_max=None at s={}", a.s);
    }
}

// ================================================================================== gate 2
/// GATE 2 — REDUCE: `lp_disabled` ASSERTS, because the SPLIT is inherently two-shaft.
///
/// A relief that lands on one spool and not the other is not a claim a single-shaft engine can
/// even state, so `lp_disabled` is not a reduce axis for it. Python raises from two entry points —
/// through the reader and directly in `integrate_fuel` — and both are exercised.
///
/// **Python calls `topping_relief` on its DEFAULTS here** (`r = 0.5, s_settle = 6.0, ds = 0.02`),
/// not on this suite's `SETTLE = 2.0` — the only call in the file that does. The refusal fires
/// before any of the three is read, so nothing numeric turns on it; they are written out as the
/// defaults anyway, because a reader comparing the two files should not have to prove that.
#[test]
fn gate2_lp_disabled_asserts_the_split_is_two_shaft() {
    let se: Engine = build_turbojet(cpg_gas(), 10.0, TT4, 50_000.0, single_matchable());
    let deg = TwoSpoolFuelTransient::lp_disabled(se, flight(), 1.0, hp_shaped());
    let f = flight();

    let m1 = refusal(|| {
        deg.topping_relief(&f, LO, HI, REDLINE, 0.5, 6.0, 0.02, None);
    })
    .expect("topping_relief on an lp_disabled object must refuse");
    assert!(m1.contains("two-shaft"), "the refusal must name the reason: {m1}");

    let governed = FuelLimiters { tt4_max: Some(REDLINE), ..Default::default() };
    let m2 = refusal(|| {
        deg.integrate_fuel_lp_disabled(&f, |_s| 0.5, 1.0, 1.0, 0.05, &governed);
    })
    .expect("integrate_fuel with a redline on an lp_disabled object must refuse");
    assert!(m2.contains("two-shaft"), "the refusal must name the reason: {m2}");
}

// =========================================================== a DISCLOSED PORT DIVERGENCE
/// **NOT A PORT OF ANY PYTHON TEST — a DISCLOSED DIVERGENCE, found by adding a cell no suite has.**
///
/// Gate 2 above arms the governor and checks the refusal. Running the SAME degenerate object with
/// the governor DISARMED — the rung-35 dispatch Python leaves open — reaches a second refusal
/// entirely, and the two languages handle it differently:
///
/// | | `integrate_fuel(..., Tt4_max=None)` on the degenerate object at `mf = 0.5` |
/// |---|---|
/// | **Python** | returns an **empty trajectory**: `_sonic_throat`'s bracket `assert` fires at the very first point, and `integrate_fuel`'s `except AssertionError` catches it and `break`s |
/// | **Rust** | **PANICS** — `components::sonic_throat`'s `assert!` is a panic, and nothing between it and the marcher converts it to an [`Abort`](turbojet::gas::Abort) the `march` loop can `break` on |
///
/// `mf = 0.5` is ~25x the design fuel flow, so this is not a physical operating point; Python's own
/// gate passes it only to trigger a refusal and never reads the result. But the divergence is a
/// CLASS, not this cell: **an `assert` that Python code catches, Rust cannot.** Measured census —
/// `choked_mfp` / `sonic_throat` have **28 call sites** in the crate and **at least 10 sit inside
/// functions already returning `Result<_, Abort>`**, every one of them one line from being
/// faithful once a fallible twin exists (`eval_m_fuel`, the route this test takes, is
/// `spool.rs:1041`).
///
/// **NOT FIXED IN SLICE T, DELIBERATELY.** The repair is contained and mechanical — add
/// `try_sonic_throat` / `try_choked_mfp` and convert the call sites already in fallible chains,
/// which cannot change behaviour on any path where the assert does not fire — but it edits shipped
/// phase-2/4/5/6 code across six files inside a slice whose whole content is gates. It is booked
/// in § 5.17 and at [`turbojet::components::sonic_throat`] instead. Slice S step 3 finding 4's
/// precedent: a divergence gets a gate on BOTH sides, never a comment on one.
///
/// **BOTH BRANCHES BELOW ARE ASSERTIONS, AND THAT IS THE POINT.** An earlier draft made the
/// panic-free branch an `expect` carrying prose — which fires as a bare test failure the next
/// reader repairs by deleting the assert rather than the test, and which would also have been
/// satisfied by a clean return arriving for some unrelated reason. So the second branch asserts
/// **Python's own measured answer at this cell — a trajectory of length 0**, and the test keeps
/// passing when the repair lands. Delete it then; until then it is the gate.
#[test]
fn disclosed_divergence_a_python_catchable_assert_panics_in_rust() {
    let se: Engine = build_turbojet(cpg_gas(), 10.0, TT4, 50_000.0, single_matchable());
    let deg = TwoSpoolFuelTransient::lp_disabled(se, flight(), 1.0, hp_shaped());
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = catch_unwind(AssertUnwindSafe(|| {
        deg.integrate_fuel_lp_disabled(&flight(), |_s| 0.5, 1.0, 1.0, 0.05,
                                       &FuelLimiters::default())
    }));
    std::panic::set_hook(hook);
    match r {
        // TODAY: the bracket assert escapes as a panic. Asserted by MESSAGE so an unrelated panic
        // arriving here fails rather than passing as "the known divergence".
        Err(e) => {
            let m = e.downcast_ref::<String>().cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "<non-string panic>".to_string());
            assert!(m.contains("CPG sonic-throat root outside the physical bracket"),
                    "the escaping panic must still be the bracket assert, not something new: {m}");
        }
        // AFTER THE REPAIR: the assert becomes an `Abort` the marcher breaks on, and the answer
        // must be Python's — measured with PyPy at this exact cell, `len(pts) == 0`, because the
        // refusal fires at the FIRST point and nothing is ever pushed.
        Ok(pts) => assert_eq!(pts.len(), 0,
                              "the fallible-twin repair has landed; Python returns an EMPTY \
                               trajectory here, so anything else is a new divergence"),
    }
}

// ============================================================================== gates 3+4+5
/// GATES 3+4+5 — THE GOVERNOR HOLDS, THE SPLIT, AND THE MECHANISM THAT GIVES IT ITS SIGN.
///
/// At every shape including the mode-free `hp-only` (LP map FLAT, so no rung-40 complex
/// inter-spool mode):
///
/// * **(3)** the governor HOLDS `Tt4` at the redline;
/// * **(4)** the SPLIT — `relief_lp` machine-zero, `relief_hp` strictly positive. A two-shaft
///   differential no single shaft can show;
/// * **(5)** the MECHANISM — `LP-min-Tt4 < redline < HP-min-Tt4`, so the clip window EXCLUDES the
///   early LP minimum and INCLUDES the late HP one.
///
/// `hp-only` witnesses that the split is the WINDOW mechanism and not a mode artifact. Magnitudes
/// are disclaimed; the differential's SIGN is gated.
///
/// **This is one of the two gates `test_rung46.py` marks `slow`.** No `#[ignore]` here: slice M's
/// rule wants a MEASURED cost, and this file's is recorded in § 5.17 step 1 rather than guessed.
#[test]
fn gates345_governor_holds_and_the_surge_relief_split() {
    let d = design(tpg());
    let f = flight();
    for (name, ml, mh) in shapes() {
        let t = ft(&d, ml, mh, 1.0);
        let o = t.topping_relief(&f, LO, HI, REDLINE, R, SETTLE, DS, None);
        assert!(o.held && o.tt4_peak_top <= REDLINE + 1e-6,
                "{name}: the governor must hold Tt4 at the redline ({})", o.tt4_peak_top);
        assert!(o.relief_lp.abs() < 1e-9,
                "{name}: LP (binding) surge relief must be machine-zero ({})", o.relief_lp);
        assert!(o.relief_hp > 1e-6,
                "{name}: HP (late) surge relief must be strictly positive ({})", o.relief_hp);
    }

    // The mechanism, on one shape — the ordering that gives the split its sign.
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let (traj, _) = t.core().fuel_ramp_march(&f, LO, HI, R, SETTLE, DS, &FuelLimiters::default());
    // Python's `min(traj, key=...)`: FIRST minimum on ties, so both folds are STRICT.
    let arg_tt4 = |key: fn(&turbojet::fuel_transient::FuelPoint) -> f64| {
        let mut best = key(&traj[0]);
        let mut tt4 = traj[0].tt4;
        for p in &traj[1..] {
            if key(p) < best {
                best = key(p);
                tt4 = p.tt4;
            }
        }
        tt4
    };
    let lp_min_tt4 = arg_tt4(|p| p.phi_lp);
    let hp_min_tt4 = arg_tt4(|p| p.phi_hp);
    assert!(lp_min_tt4 < REDLINE && REDLINE < hp_min_tt4,
            "window: LP-min-Tt4 < redline < HP-min-Tt4 ({lp_min_tt4}, {REDLINE}, {hp_min_tt4})");
}

// ================================================================================== gate 6
/// GATE 6 — THE LEVER: `relief_lp` switches ON in the fast-ramp limit.
///
/// `relief_lp` is machine-zero at MODERATE `r` — the LP surge minimum sits below the redline,
/// outside the clip window — but goes strictly POSITIVE at `r <= 0.15`, where that minimum
/// migrates ABOVE the redline and into the window. So the governor becomes a modest LP-surge lever
/// precisely where surge is most dangerous.
///
/// The gated claim is *"zero at moderate `r`, positive fast"*, not an unconditional zero — which
/// is why gates 3-5's machine-zero is not the whole finding.
///
/// The second `slow` gate of the two.
#[test]
fn gate6_the_lever_fast_ramp_switches_on_lp_relief() {
    let d = design(tpg());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    let f = flight();
    let tt4_max = 1440.0;
    let slow = t.topping_relief(&f, LO, HI, tt4_max, 0.5, SETTLE, DS, None);
    let fast = t.topping_relief(&f, LO, HI, tt4_max, 0.15, SETTLE, DS, None);
    assert!(slow.relief_lp.abs() < 1e-9, "moderate r: LP relief zero ({})", slow.relief_lp);
    assert!(fast.relief_lp > 1e-4, "fast r: LP relief positive ({})", fast.relief_lp);
    assert!(fast.relief_hp > slow.relief_hp, "faster ramp => more HP rebate too");
}

// ================================================================================== gate 7
/// GATE 7 — DECEL: the clip never fires, so the topped decel is bit-for-bit rung 45.
///
/// The topping governor is an ACCELERATION-schedule limiter. On a decel `Tt4` undershoots and
/// never exceeds a redline above the endpoint, so the min-select never selects the clip and the
/// topped march equals the bare one float-for-float, at every shape.
#[test]
fn gate7_decel_is_bit_for_bit_rung45() {
    let d = design(cpg_gas());
    let f = flight();
    for (name, ml, mh) in shapes() {
        let t = ft(&d, ml, mh, 1.0);
        let bare = t.phi_excursion_fuel(&f, HI, LO, R, SETTLE, DS, None, None, None, None);
        let top = t.phi_excursion_fuel(&f, HI, LO, R, SETTLE, DS, Some(REDLINE), None, None, None);
        assert_eq!(phi_exc_bits(&top), phi_exc_bits(&bare), "{name}: decel must never fire the clip");
    }
}

// ================================================================================== gate 8
/// GATE 8 — CYCLE UNTOUCHED: exercising the governor does not perturb the rung-6 design run.
#[test]
fn gate8_cycle_untouched_bit_for_bit_rung6() {
    let eng: Engine = build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, single());
    let f = flight();
    let a = eng.run(&f, 1.0);
    let d = design(cpg_gas());
    let t = ft(&d, lp_shaped(), hp_shaped(), 1.0);
    t.topping_relief(&f, LO, HI, REDLINE, R, 1.5, DS, None);
    let b = eng.run(&f, 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
    assert_eq!(a.station("4").far.to_bits(), b.station("4").far.to_bits());
}
