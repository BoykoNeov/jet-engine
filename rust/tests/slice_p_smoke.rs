//! SLICE P STEP 1 — the go/no-go smoke check: does the ported `SpoolTransient` reproduce Python
//! AT ALL?
//!
//! **This is not the gate** — `spool_oracle.rs` is. It exists so a structural mistake is caught
//! before an oracle is built around it: a wrong hook table (rung 31's bisection where rung 34's
//! Illinois belongs, which § 5.13 probe 2 says moves `pi_t` by ~9e-12), a mis-threaded design
//! reference, a reordered Illinois convergence test. Slice N's and slice O's precedent.
//!
//! 132 values dumped from PyPy as RAW BITS at `M:\claud_projects\temp\rust-phase6`, over eight
//! cells chosen so every path in the slice is touched at least once: a choked equilibrium, a
//! subsonic one, an off-equilibrium instant, a marched RK4 trajectory, the rung-35 fuel control,
//! rung 36's margin and its compounding, rung 41's channels, all three `phi_max` arms driven
//! directly, and the forward/backward map inverse.

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::Gas;
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::spool::SpoolTransient;

fn flight() -> FlightCondition {
    FlightCondition { t0: 250.0, p0: 50_000.0, m0: 0.85 }
}

fn real() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, e_t: None,
        eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
    }
}

fn design() -> Engine {
    build_turbojet(Gas::thermally_perfect(), 10.0, 1500.0, 50_000.0, real())
}

fn st(cmap: ComponentMap) -> SpoolTransient {
    SpoolTransient::new(design(), flight(), 1.0, cmap)
}

/// Compare against PyPy's bits. `key` is the dump's own key, so a failure names the quantity.
fn eq_bits(key: &str, got: f64, want: u64) {
    assert_eq!(
        got.to_bits(),
        want,
        "{key}: Rust {got:?} (bits {}) != PyPy bits {want}",
        got.to_bits()
    );
}

/// PyPy's dump. **Never retyped from a decimal** — the crate's standing rule, and slice O's
/// reminder that an oracle's INPUTS have to be built with the source's own expression too.
const PY: &[(&str, u64)] = &[
    ("Pc_spec_d", 4688959784859220217),
    ("P_ref", 4688959784859220217),
    ("A/nu", 4607182418799998932),
    ("A/n", 4607182418799998932),
    ("A/pi_c", 4621819117588931366),
    ("A/tau_c", 4611770577926465715),
    ("A/mdot_air", 4607182418799952887),
    ("A/f", 4583515894746806934),
    ("A/pi_t", 4601396903998884109),
    ("A/tau_t", 4605762360027409860),
    ("A/Tt3", 4648343779432371110),
    ("A/Tt5", 4653271803104164453),
    ("A/flowcoef", 4607182418799971363),
    ("A/Phi", 13595136816295274673),
    ("A/sp_thrust", 4649771419385843406),
    ("A/M9", 4607182418800016934),
    ("A/pt9_over_p0", 4618759410625772371),
    ("A/eta_c", 4606101554889448489),
    ("A/eta_t", 4606281698874543309),
    ("A/nu_t", 4607182418799998932),
    ("A/p_net_spec", 13677432068324196352),
    ("A/m", 4607182418799952887),
    ("A/Tt2", 4643745586963694955),
    ("A/pt2", 4680086807681088516),
    ("A/V0", 4643453651052699634),
    ("A/thrust", 4649771419385796413),
    ("B/nu", 4603399355079703454),
    ("B/n", 4603399355079703454),
    ("B/pi_c", 4612945806269462122),
    ("B/mdot_air", 4601850032835219385),
    ("B/f", 4570208698880457116),
    ("B/pi_t", 4601419780084211770),
    ("B/tau_t", 4605529695880442415),
    ("B/Phi", 4365571704325590355),
    ("B/M9", 4605923845966850433),
    ("B/sp_thrust", 4633899305753487272),
    ("C/pi_c", 4621030851400889938),
    ("C/tau_c", 4611457900730922224),
    ("C/mdot_air", 4606562431404400136),
    ("C/f", 4581871330415195802),
    ("C/pi_t", 4601376922777175295),
    ("C/tau_t", 4605728371772606127),
    ("C/Phi", 13810699089317957064),
    ("C/flowcoef", 4607003863607511911),
    ("C/M9", 4607182418800016853),
    ("C/n_pts", 4626604192193052672),
    ("C/traj0/s", 0),
    ("C/traj0/nu", 4606281698874543309),
    ("C/traj0/Tt4", 4652552666608566272),
    ("C/traj0/pi_c", 4619939910941121218),
    ("C/traj0/f", 4580292374658948575),
    ("C/traj0/Phi", 13814911891307700561),
    ("C/traj0/sp_thrust", 4647933999224991664),
    ("C/traj5/s", 4598175219545276416),
    ("C/traj5/nu", 4606178831061525851),
    ("C/traj5/Tt4", 4653212373585231872),
    ("C/traj5/pi_c", 4620000925778522506),
    ("C/traj5/f", 4581600022014200378),
    ("C/traj5/Phi", 13788622244586802687),
    ("C/traj5/sp_thrust", 4648481800047435362),
    ("C/traj10/s", 4602678819172646911),
    ("C/traj10/nu", 4606254839196274897),
    ("C/traj10/Tt4", 4653872080561897472),
    ("C/traj10/pi_c", 4620423510946882930),
    ("C/traj10/f", 4582897045319138401),
    ("C/traj10/Phi", 4589412571898068485),
    ("C/traj10/sp_thrust", 4649073477077304629),
    ("C/traj20/s", 4607182418800017409),
    ("C/traj20/nu", 4606483296598667084),
    ("C/traj20/Tt4", 4653872080561897472),
    ("C/traj20/pi_c", 4620831571947182062),
    ("C/traj20/f", 4582827507967487224),
    ("C/traj20/Phi", 4585617321932557884),
    ("C/traj20/sp_thrust", 4649166131971024786),
    ("D/mf", 4578102469382629501),
    ("D/nu", 4605566620785509837),
    ("D/Tt4", 4652552666608565604),
    ("D/pi_c", 4618494000952540457),
    ("D/mdot_air", 4604636101219965747),
    ("D/f", 4580500960861463387),
    ("D/Phi", 4380642661649774697),
    ("D/tt4_from_f", 4654416836170749834),
    ("E/n", 4606367321277454572),
    ("E/phi_op", 4606581433007227040),
    ("E/pi_c", 4620515462362358122),
    ("E/SM_N", 4599539030279503088),
    ("E/SM_flow", 4624172932768431805),
    ("E/nu", 4606367321277454572),
    ("E/ab_E0", 4589023567422847232),
    ("E/ab_SM_N", 4597063541342154376),
    ("E/ab_ratio", 4598726924192982978),
    ("E/ab_nu0", 4605566620785508860),
    ("E/ab_phi_step", 4605343639097301457),
    ("E/ab_reaches", 0),
    ("F/SM_N", 4598482601969735012),
    ("F/SM_phi_walk", 4599525020006383452),
    ("F/SM_speed_line", 4600726327100490856),
    ("F/SM_ref", 4602344746300529708),
    ("F/n", 4605965663708545124),
    ("F/phi_op", 4606303235332028792),
    ("G/phi_max/flat", 4617315517961601024),
    ("G/phi_max_floor02/flat", 4617315517961601024),
    ("G/phi_max/quad", 4611933203511401781),
    ("G/phi_max_floor02/quad", 4611686018427387904),
    ("G/phi_max/quad2", 4611134791512078494),
    ("G/phi_max_floor02/quad2", 4610754468581411466),
    ("G/phi_max/linear", 4612329389802726546),
    ("G/phi_max_floor02/linear", 4612007704115057226),
    ("G/phi_max/swirl", 4609493408265433748),
    ("G/phi_max_floor02/swirl", 4609096198440435003),
    ("G/phi_max/swirl_lin", 4610961301245971962),
    ("G/phi_max_floor02/swirl_lin", 4610443646116389146),
    ("H/tau_c/0.8/0.7", 4610429958244771384),
    ("H/solve_n/0.8/0.7", 4605380978949069236),
    ("H/tau_c/0.8/1.0", 4609630923359803140),
    ("H/solve_n/0.8/1.0", 4605380978949069236),
    ("H/tau_c/0.8/1.3", 4608747779539575080),
    ("H/solve_n/0.8/1.3", 4605380978949069236),
    ("H/tau_c/1.0/0.7", 4612240186148342602),
    ("H/solve_n/1.0/0.7", 4607182418800017418),
    ("H/tau_c/1.0/1.0", 4611770577926475300),
    ("H/solve_n/1.0/1.0", 4607182418800017418),
    ("H/tau_c/1.0/1.3", 4610831812046568278),
    ("H/solve_n/1.0/1.3", 4607182418800017418),
    ("H/tau_c/1.15/0.7", 4613323088689812722),
    ("H/solve_n/1.15/0.7", 4607857958744122981),
    ("H/tau_c/1.15/1.0", 4612800912383408035),
    ("H/solve_n/1.15/1.0", 4607857958744122981),
    ("H/tau_c/1.15/1.3", 4612236681609373441),
    ("H/solve_n/1.15/1.3", 4607857958744122981),
];

fn want(key: &str) -> u64 {
    PY.iter().find(|(k, _)| *k == key).unwrap_or_else(|| panic!("no PyPy bits for {key}")).1
}

#[test]
fn design_references_and_the_choked_equilibrium_reproduce_bit_for_bit() {
    let s = st(ComponentMap::flat());
    eq_bits("Pc_spec_d", s.pc_spec_d, want("Pc_spec_d"));
    eq_bits("P_ref", s.p_ref, want("P_ref"));

    let eq = s.equilibrium(&flight(), 1500.0, None);
    assert_eq!(eq.branch, Branch::Choked);
    for (k, v) in [
        ("A/nu", eq.nu), ("A/n", eq.n), ("A/pi_c", eq.pi_c), ("A/tau_c", eq.tau_c),
        ("A/mdot_air", eq.mdot_air), ("A/f", eq.f), ("A/pi_t", eq.pi_t), ("A/tau_t", eq.tau_t),
        ("A/Tt3", eq.tt3), ("A/Tt5", eq.tt5), ("A/flowcoef", eq.flowcoef), ("A/Phi", eq.phi),
        ("A/sp_thrust", eq.sp_thrust), ("A/M9", eq.m9), ("A/pt9_over_p0", eq.pt9_over_p0),
        ("A/eta_c", eq.eta_c), ("A/eta_t", eq.eta_t), ("A/nu_t", eq.nu_t),
        ("A/p_net_spec", eq.p_net_spec), ("A/m", eq.m), ("A/Tt2", eq.tt2), ("A/pt2", eq.pt2),
        ("A/V0", eq.v0), ("A/thrust", eq.thrust),
    ] {
        eq_bits(k, v, want(k));
    }
}

#[test]
fn the_subsonic_equilibrium_reproduces_bit_for_bit() {
    // Tt4 = 520 sits below nozzle unchoke, so this is rung 33's branch reached through rung 34's
    // forward closure — and it is the cell `_turbine_subsonic` exists for.
    let s = st(ComponentMap::flat());
    let eq = s.equilibrium(&flight(), 520.0, None);
    assert_eq!(eq.branch, Branch::Subsonic, "Tt4 = 520 must dispatch subsonic");
    for (k, v) in [
        ("B/nu", eq.nu), ("B/n", eq.n), ("B/pi_c", eq.pi_c), ("B/mdot_air", eq.mdot_air),
        ("B/f", eq.f), ("B/pi_t", eq.pi_t), ("B/tau_t", eq.tau_t), ("B/Phi", eq.phi),
        ("B/M9", eq.m9), ("B/sp_thrust", eq.sp_thrust),
    ] {
        eq_bits(k, v, want(k));
    }
}

#[test]
fn the_shaped_map_instant_and_a_marched_rk4_trajectory_reproduce_bit_for_bit() {
    // The march is the point of the slice, and it is where a reordered Illinois or a
    // mis-threaded RK4 stage shows up first — accumulated over 21 steps rather than at one point.
    let s = st(ComponentMap::surge_flow());
    let inst = s.instant(&flight(), 0.95, 1300.0, None);
    for (k, v) in [
        ("C/pi_c", inst.pi_c), ("C/tau_c", inst.tau_c), ("C/mdot_air", inst.mdot_air),
        ("C/f", inst.f), ("C/pi_t", inst.pi_t), ("C/tau_t", inst.tau_t), ("C/Phi", inst.phi),
        ("C/flowcoef", inst.flowcoef), ("C/M9", inst.m9),
    ] {
        eq_bits(k, v, want(k));
    }

    let traj = s.integrate(
        &flight(), |x| 1100.0 + 300.0 * 1.0f64.min(x / 0.5), 0.90, 1.0, 0.05, None,
    );
    eq_bits("C/n_pts", traj.len() as f64, want("C/n_pts"));
    for i in [0usize, 5, 10, 20] {
        let p = &traj[i];
        for (k, v) in [
            (format!("C/traj{i}/s"), p.s), (format!("C/traj{i}/nu"), p.nu),
            (format!("C/traj{i}/Tt4"), p.tt4), (format!("C/traj{i}/pi_c"), p.pi_c),
            (format!("C/traj{i}/f"), p.f), (format!("C/traj{i}/Phi"), p.phi),
            (format!("C/traj{i}/sp_thrust"), p.sp_thrust),
        ] {
            eq_bits(&k, v, want(&k));
        }
    }
}

#[test]
fn rung35_fuel_control_reproduces_bit_for_bit() {
    let s = st(ComponentMap::surge_flow());
    let mf = s.fuel_for_tt4(&flight(), 1100.0, None);
    eq_bits("D/mf", mf, want("D/mf"));
    let eq = s.equilibrium_fuel(&flight(), mf, None);
    for (k, v) in [
        ("D/nu", eq.nu), ("D/Tt4", eq.tt4), ("D/pi_c", eq.pi_c), ("D/mdot_air", eq.mdot_air),
        ("D/f", eq.f), ("D/Phi", eq.phi),
    ] {
        eq_bits(k, v, want(k));
    }
    eq_bits("D/tt4_from_f", s.tt4_from_f(700.0, 0.025), want("D/tt4_from_f"));
}

#[test]
fn rung36_margin_and_its_compounding_reproduce_bit_for_bit() {
    let s = st(ComponentMap::surge_flow());
    let m36 = ComponentMap::surge_flow().with_phi_surge(0.55);
    let sm = s.surge_margin(&flight(), 1300.0, Some(&m36));
    for (k, v) in [
        ("E/n", sm.n), ("E/phi_op", sm.phi_op), ("E/pi_c", sm.pi_c), ("E/SM_N", sm.sm_n),
        ("E/SM_flow", sm.sm_flow), ("E/nu", sm.nu),
    ] {
        eq_bits(k, v, want(k));
    }
    let ab = s.acceleration_binding(&flight(), 1100.0, 1450.0, Some(&m36));
    for (k, v) in [
        ("E/ab_E0", ab.e0), ("E/ab_SM_N", ab.sm_n), ("E/ab_ratio", ab.ratio),
        ("E/ab_nu0", ab.nu0), ("E/ab_phi_step", ab.phi_step),
    ] {
        eq_bits(k, v, want(k));
    }
    assert!(!ab.reaches_surge, "PyPy says this burst does not reach surge (E/ab_reaches = 0)");
}

#[test]
fn rung41_channels_reproduce_bit_for_bit() {
    // The gate `rung41.rs`'s roster deferred to phase 6, because it is built on the SINGLE-spool
    // transient. § 5.13 prediction 9.
    let s = st(ComponentMap::surge_flow());
    let m36 = ComponentMap::surge_flow().with_phi_surge(0.55);
    let ch = s.surge_margin_channels(&flight(), 1200.0, Some(&m36), None);
    for (k, v) in [
        ("F/SM_N", ch.sm_n), ("F/SM_phi_walk", ch.sm_phi_walk),
        ("F/SM_speed_line", ch.sm_speed_line), ("F/SM_ref", ch.sm_ref), ("F/n", ch.n),
        ("F/phi_op", ch.phi_op),
    ] {
        eq_bits(k, v, want(k));
    }
}

#[test]
fn phi_max_reproduces_all_three_arms_including_the_two_this_slice_never_reaches() {
    // § 5.13 probe 1: of `phi_max`'s three arithmetic arms only `flat5` and `quadratic` are
    // reachable through a rung-34 march, and `vsv == 0.0` at all 16 508 measured calls. So the
    // LINEAR arm and every `A != 0` path are driven DIRECTLY here — a constant measured dead
    // still has to be spelled right (slice N step 3), and this is the only gate that can say so.
    let flat = ComponentMap::flat();
    let cases: [(&str, ComponentMap); 6] = [
        ("flat", flat),
        ("quad", ComponentMap::surge_flow()),
        ("quad2", ComponentMap { sigma: 0.2, l: 0.85, ..flat }),
        ("linear", ComponentMap { sigma: 0.0, l: 0.7, ..flat }),
        ("swirl", ComponentMap { sigma: 0.1, l: 0.7, ..flat }.with_vsv(0.20)),
        ("swirl_lin", ComponentMap { sigma: 0.0, l: 0.7, ..flat }.with_vsv(0.10)),
    ];
    for (label, cm) in cases {
        eq_bits(&format!("G/phi_max/{label}"), cm.phi_max(0.1), want(&format!("G/phi_max/{label}")));
        eq_bits(
            &format!("G/phi_max_floor02/{label}"),
            cm.phi_max(0.2),
            want(&format!("G/phi_max_floor02/{label}")),
        );
    }
    // The arms must not be the same number, or the gate above is comparing one branch six times.
    let vals: Vec<u64> = cases.iter().map(|(_, c)| c.phi_max(0.1).to_bits()).collect();
    let distinct: std::collections::HashSet<_> = vals.iter().collect();
    assert_eq!(distinct.len(), 6, "the six phi_max shapes must give six DISTINCT values");
}

#[test]
fn the_forward_speed_line_and_solve_n_are_exact_inverses() {
    // Rung 34's gate 6, and § 5.13 prediction 7: `solve_n(m, tau_c_forward(n, m)) == n` should be
    // EXACT rather than merely tight, because slice J ported the inverse of this very equation.
    let s = st(ComponentMap::surge_flow());
    let cmap = ComponentMap::surge_flow();
    // Labels are the DUMP's own key text, not Rust's float formatting — `{1.0f64}` prints "1"
    // where Python's f-string prints "1.0", and a key that misses is a panic, not a pass.
    for (nl, n) in [("0.8", 0.8f64), ("1.0", 1.0), ("1.15", 1.15)] {
        for (ml, m) in [("0.7", 0.7f64), ("1.0", 1.0), ("1.3", 1.3)] {
            let tc = s.tau_c_forward(&cmap, n, m);
            let back = cmap.solve_n(m, tc, s.inner.tau_c_d);
            let kt = format!("H/tau_c/{nl}/{ml}");
            let ks = format!("H/solve_n/{nl}/{ml}");
            eq_bits(&kt, tc, want(&kt));
            eq_bits(&ks, back, want(&ks));
            assert!(
                (back - n).abs() <= 1e-12 * n,
                "forward/backward map inverse at (n={n}, m={m}): got {back}, want {n}"
            );
        }
    }
}

/// The COUNT keys — what the 132 value keys are structurally blind to.
///
/// Two changes injected into [`try_illinois`] left every one of those 132 values bit-exact:
/// moving the convergence test ahead of `f(c)`, and changing what the exhaustion arm returns.
/// Neither is a value property. The first alters only how many residual evaluations happen; the
/// second is unreachable. So both are gated here, against PyPy counts measured the same way
/// Python measures its own — by replacing `_illinois` with a counting copy, which is legitimate
/// precisely because the copy is the INSTRUMENT and the shipped loop is what it wraps.
///
/// Rust's harness runs each `#[test]` on its own thread and the counters are thread-local, so
/// this test's tallies are its own. It still calls `take()` first, because a future gate added
/// to this file must not silently inherit them (`stage.rs`'s `take_census` fragility, noted).
#[test]
fn the_illinois_call_and_evaluation_counts_reproduce_pypys() {
    use turbojet::spool::counters;

    // (label, PyPy calls, PyPy residual evaluations)
    let expect = |label: &str, evals: u64| {
        let c = counters::take();
        assert_eq!(
            c.illinois_evals, evals,
            "{label}: Rust ran {} Illinois residual evaluations, PyPy ran {evals}",
            c.illinois_evals
        );
        assert_eq!(
            c.illinois_exhausted, 0,
            "{label}: no Illinois call should exhaust maxit — if one does, the `Ok(b)` arm is \
             live and this gate's second clause has become the only thing watching it"
        );
    };

    counters::take(); // discard anything this thread accumulated while building
    st(ComponentMap::flat()).equilibrium(&flight(), 1500.0, None);
    expect("eq_flat_1500", 227);

    st(ComponentMap::flat()).equilibrium(&flight(), 520.0, None);
    expect("eq_flat_520", 403);

    let s = st(ComponentMap::surge_flow());
    s.integrate(&flight(), |x| 1100.0 + 300.0 * 1.0f64.min(x / 0.5), 0.90, 1.0, 0.05, None);
    expect("march_shaped", 1344);

    let s = st(ComponentMap::surge_flow());
    s.equilibrium_fuel(&flight(), 0.0197, None);
    expect("eq_fuel", 199);
}
