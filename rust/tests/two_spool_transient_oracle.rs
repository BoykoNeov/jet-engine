//! SLICE R step 4 — the rungs 40 + 44 ORACLE. Every value `oracle/dump_two_spool_transient.py`
//! produces, recomputed here and compared BIT for bit, on BOTH Python suites' full grids.
//!
//! # Why an oracle, when 17 rung gates already pass
//!
//! Steps 1 and 3 measured it rather than argued it. Nineteen defects were injected into the
//! shipped port across the two steps, and the ones the rung suites could not see are exactly what
//! this file's grid is aimed at:
//!
//! | injected defect | rung gates that fail (of 17) | seen by |
//! |---|---|---|
//! | `best` keeps the LATEST tie (`<=` for `<`) | 0 | **nothing** — step 1 |
//! | the march-in ladder as `0.02*(k+1)` | 0 | **nothing** — step 1 |
//! | the high wall drops the literal `2.5` arm | 0 | the CENSUS only (two `illinois_evals`) |
//! | `min_phi_lp`/`min_phi_hp` tracked as a MAX | 0 | the smoke only — step 3 |
//! | `s_lp`/`s_hp` (WHERE the extremum sits) frozen | 0 | the smoke only — step 3 |
//! | **delete `rho` from the SHIPPED marcher** | 0 in `rung44.rs` | caught a rung away |
//!
//! So: the census is emitted PER SECTION on the full grid (that is the only thing that can see the
//! high wall), every excursion cell carries all EIGHT returned keys including the four no Python
//! gate reads, and the Newton's tie-break gets a counter of its own — step 1 registered *"step 4's
//! larger reacting grid is where it could be, and that is registered rather than assumed covered"*.
//!
//! # The two suites run two DIFFERENT CPG gases
//!
//! `test_rung40.py` hard-codes `R_c = 286.9`; `test_rung44.py` derives it as
//! `(gamma_c-1)/gamma_c*cp_c = 286.8571428571428`. Section A dumps both. This is not decoration:
//! `rust/tests/rung44.rs` shipped at step 3 with `286.9`, copied from its neighbour `rung40.rs`,
//! so every rung-44 gate ran rung 40's gas — invisible, because every assertion in that file is a
//! sign, an ordering or a spread, and step 3's own value probe used `286.9` on BOTH sides so it
//! could not see it either. Found here by enumerating each suite's grid instead of reading the
//! constant off its neighbour.
//!
//! # The three arms
//!
//! * **main** (`two_spool_transient_pypy.tsv`, 6 853 keys) — the CPG grids of both suites.
//! * **equil** (`two_spool_transient_eq_pypy.tsv`, 1 120 keys) — the REACTING cells. § 5.15
//!   prediction 1 registered these as the slice's one genuine exposure: probe 4 measured the
//!   Newton's exit CLASSIFICATION flipping in 5 of 12 cells between CPython and PyPy, because the
//!   `1e-12` acceptance bar is ABSOLUTE and sits below the gas sub-solve's own ~1e-10 noise. Step 4
//!   SPIKED those 12 cells against the shipped Rust before this file was written: all 12 agree with
//!   PyPy on the exit kind, the pass count and both converged speeds, so the arm ships at
//!   bit-equality and § 9 Decision 1's Option B is not invoked.
//! * **cpython** (`two_spool_transient_cpython.tsv`) — main + equil under CPython 3.14. Read as a
//!   DETECTOR with a measured sensitivity, never as coverage; see `oracle_matches_cpython`.
//!
//! Regenerate with:
//!     .venv\Scripts\python.exe rust/oracle/dump_two_spool_transient.py main    rust/oracle/two_spool_transient_pypy.tsv
//!     .venv\Scripts\python.exe rust/oracle/dump_two_spool_transient.py equil   rust/oracle/two_spool_transient_eq_pypy.tsv
//!     C:\Python314\python.exe  rust/oracle/dump_two_spool_transient.py cpython rust/oracle/two_spool_transient_cpython.tsv

use std::collections::{BTreeMap, BTreeSet};

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::spool::{counters as scount, SpoolTransient};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{
    counters as tcount, CloseState, EqExit, Instant2, PhiExcursion, TransientSurgeMargin,
    TwoSpoolTransientCore, TwoSpoolTransientPoint,
};

const ORACLE_MAIN: &str = include_str!("../oracle/two_spool_transient_pypy.tsv");
const ORACLE_EQ: &str = include_str!("../oracle/two_spool_transient_eq_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/two_spool_transient_cpython.tsv");

/// How a key is compared on the CPython arm — MEASURED per class, never chosen.
///
/// The PyPy-vs-CPython diff over all 7 973 keys splits by MECHANISM, and every bar below is the
/// measured worst with one decade of margin:
///
/// | class | keys | moved | worst | bar here |
/// |---|---|---|---|---|
/// | CPG (closed-form arithmetic) | 6 410 | **0** | — | **BITS** |
/// | reacting DIRECT (`nu`, `pi`, `mdot`, `phi`) | 629 | 523 | 1.06e-10 rel | 1e-9 rel |
/// | reacting RESIDUAL (`Phi`, `Phi_lp`, `Phi_hp`) | 30 | 30 | 8.17e-11 **abs** | 1e-9 abs |
/// | reacting DERIVATIVE (section R's `J`/`bc`/`eig`) | 357 | 357 | 6.37e-3 rel | 5e-2 rel |
/// | thermally-perfect finite difference (`E/channel/gas`) | 1 | 1 | 1.40e-9 rel | 1e-8 rel |
///
/// The classes are not cosmetic. The reacting equilibrium sub-solve leaves ~1e-10 of noise in
/// `Phi`, which is the direct class's bar exactly. The residual class is that same noise on a
/// quantity whose true value is ZERO, so relative is the wrong currency — `L/1500/Phi` reads
/// `1.3e+4` relative and `2.9e-11` absolute, and only the second is a statement about anything.
/// The derivative class is that same noise divided by the Jacobian's `h = 1e-6`, i.e. amplified a
/// millionfold; `bc` is worst because it is a product of the two SMALL off-diagonals. And
/// `E/channel/gas` is the one key in the whole main dump built on a THERMALLY-PERFECT gas, whose
/// table integrals go through `log`/`exp` — so it drifts where every closed-form CPG key does not.
#[derive(Clone, Copy)]
enum Tier {
    Bits,
    Rel(f64),
    Abs(f64),
}

fn tier(key: &str) -> Tier {
    if key == "E/channel/gas" {
        return Tier::Rel(1e-8);
    }
    if !is_reacting(key) {
        return Tier::Bits;
    }
    match key.rsplit('/').next().unwrap_or("") {
        "Phi" | "Phi_lp" | "Phi_hp" => Tier::Abs(1e-9),
        _ if key.starts_with("R/") => Tier::Rel(5e-2),
        _ => Tier::Rel(1e-9),
    }
}

/// Whether a key comes off the REACTING gas, and is therefore in the CPython arm's tolerant tiers.
///
/// Sections P/Q/R/S are the reacting arm; section **L** is the `lp_disabled` reduce, whose
/// single-spool design is ALSO built on `Gas::reacting_equilibrium()` — reading the arm off the
/// FILE name would have put `L` in the bit-exact tier and failed. It is a gas test, not a file test.
///
/// **THE SECTION LETTERS ARE LOAD-BEARING.** This is a prefix test, so a later step adding a CPG
/// section under `P`, `Q`, `R`, `S` or `L` would silently inherit a tolerant tier and its drift
/// would stop being a failure. Those five letters are reserved for reacting sections; a new CPG
/// section takes a new one.
fn is_reacting(key: &str) -> bool {
    let k = key.strip_prefix("census/").unwrap_or(key);
    k.starts_with("P/") || k.starts_with("Q/") || k.starts_with("R/") || k.starts_with("S/")
        || k.starts_with("L/")
}

/// The one DISCRETE key class that moves on the CPG half, and the only one exempted there.
///
/// `census/I/shapes/illinois_evals` reads 54 323 on PyPy and 54 322 on CPython; `census/K` reads
/// 38 513 against 38 512. **Every value those root finds produce is bit-identical** — the CPG half
/// moves zero floats — so what differs is one convergence test landing on the other side of its
/// bracket, and the root comes back the same double from a different last step. That is slice N's
/// rule (*"iteration counts are not interpreter-invariant"*) holding, in the same arm where its
/// other half (*"the branch verdicts ARE, because a verdict is a comparison"*) is INVERTED by the
/// reacting Newton's five flipped exits. Both halves of one precedent, contradicted in one dump.
///
/// **THIS EXEMPTS THE CLASS, WHICH IS WIDER THAN THE MEASUREMENT, DELIBERATELY.** The CPG half
/// carries **16** `illinois_evals` keys (17 census blocks in the main dump, of which `L` is
/// reacting) and exactly **2** of them move. Exempting the class rather
/// than the two is the right width because what makes a key exempt is BEING an iteration count, not
/// being one of these two — pinning the pair would turn a CPython point release into a port
/// regression. The width is not hidden: `oracle_matches_cpython` prints how many actually
/// moved on EACH half (2 of 16 CPG, 5 of 6 reacting), and asserts every CPG-half mover is INSIDE this class, so a new drifter
/// fails.
fn is_iteration_count(key: &str) -> bool {
    key.ends_with("/illinois_evals")
}

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        assert!(m.insert(k.to_string(), b.parse::<u64>().expect("u64")).is_none(), "dup {k}");
    }
    m
}

/// Accumulates `(key, got, want)` so ONE run reports every disagreement, and reports every golden
/// key the Rust never asked for.
///
/// **BOTH HALVES PANIC TOGETHER.** Step 1 found this comparator's ancestor asserting the VALUE
/// diffs BEFORE the never-compared ones, which made the half that exists to catch a field missing
/// from the PORT unreachable whenever any value also moved — exactly what a short march does.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
    /// Set on the CPython arm, where a REACTING-gas disagreement is content rather than a failure.
    /// The CPG keys are held to bit-equality there exactly as they are against PyPy.
    cpython: bool,
    /// `(key, message)` — the KEY kept separately, never re-parsed out of the message.
    discrete_flips: Vec<(String, String)>,
    float_drifts: Vec<(String, f64)>,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython: false,
              discrete_flips: Vec::new(), float_drifts: Vec::new() }
    }

    /// A float key. Bit-exact against PyPy; on the CPython arm, at the class bar [`tier`] gives it.
    fn f(&mut self, key: &str, got: f64) {
        assert!(got.is_finite(), "{key} is not finite: {got}");
        if !self.cpython {
            return self.cmp_bits(key, got.to_bits(), false);
        }
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        let Some(&want) = self.py.get(key) else {
            self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)"));
            return;
        };
        let exp = f64::from_bits(want);
        let d = (got - exp).abs();
        let (over, scale) = match tier(key) {
            Tier::Bits => (got.to_bits() != want, 0.0),
            Tier::Abs(bar) => (d > bar, d),
            Tier::Rel(bar) => {
                let rel = if exp == 0.0 { d } else { d / exp.abs() };
                (rel > bar, rel)
            }
        };
        if d > 0.0 {
            self.float_drifts.push((key.to_string(), scale));
        }
        if over {
            self.bad.push(format!("{key}: rust {got:e} vs cpython {exp:e} (dev {scale:e})"));
        }
    }

    /// A discrete key (a count, a flag, a branch label), compared as an integer.
    fn d(&mut self, key: &str, got: u64) {
        self.cmp_bits(key, got, true);
    }

    fn cmp_bits(&mut self, key: &str, got: u64, discrete: bool) {
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        match self.py.get(key) {
            None => self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)")),
            Some(&want) if want != got => {
                let tolerated = discrete && self.cpython
                    && (is_reacting(key) || is_iteration_count(key));
                if tolerated {
                    self.discrete_flips.push(
                        (key.to_string(), format!("{key}: rust {got} vs cpython {want}")));
                } else {
                    self.bad.push(format!(
                        "{key}: rust {} vs python {}",
                        if discrete { got as f64 } else { f64::from_bits(got) },
                        if discrete { want as f64 } else { f64::from_bits(want) }));
                }
            }
            Some(_) => {}
        }
    }

    fn finish(&self) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        if self.bad.is_empty() && missed.is_empty() {
            return;
        }
        panic!(
            "{} of {} compared keys differ:\n  {}\n\n{} golden keys the Rust never asked for:\n  \
             {:?}",
            self.bad.len(), self.seen.len(), self.bad.join("\n  "), missed.len(),
            missed.iter().take(24).collect::<Vec<_>>()
        );
    }
}

// ------------------------------------------------------------------------------ the grid
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// `test_rung40.py`'s CPG dual gas — `R_c` HARD-CODED at 286.9.
fn cpg40() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

/// `test_rung44.py`'s CPG dual gas — `R_c` DERIVED, so `286.8571428571428`, not `286.9`.
fn cpg44() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

fn core(gas: Gas, ml: ComponentMap, mh: ComponentMap, rho: f64) -> TwoSpoolTransientCore {
    TwoSpoolTransientCore::new(design(gas), flight(), 1.0, ml, mh, rho)
}

fn flat() -> ComponentMap { ComponentMap::flat() }
fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}
fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}
fn tilted() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
}
fn steep() -> ComponentMap {
    ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..ComponentMap::flat() }
}
fn press_lp() -> ComponentMap {
    ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}
fn press_hp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

/// `test_rung40.py`'s seven disclosed pairs, in ITS order. `test_rung44.py`'s five are the SUBSET
/// listed in [`RUNG44_SHAPES`] — the same maps, so one table serves both suites.
fn shapes() -> [(&'static str, ComponentMap, ComponentMap); 7] {
    [
        ("flat", flat(), flat()),
        ("flow_press", lp_shaped(), hp_shaped()),
        ("press_flow", press_lp(), press_hp()),
        ("tilted", tilted(), tilted()),
        ("steep", steep(), steep()),
        ("lp_only", lp_shaped(), flat()),
        ("hp_only", flat(), hp_shaped()),
    ]
}

const RUNG44_SHAPES: [&str; 5] = ["flow_press", "press_flow", "tilted", "steep", "hp_only"];

fn shape(name: &str) -> (ComponentMap, ComponentMap) {
    for (n, ml, mh) in shapes() {
        if n == name {
            return (ml, mh);
        }
    }
    panic!("no shape {name}")
}

/// Gate 5's `rho` spot-check sweep.
const RHO_EIG: [f64; 6] = [0.05, 0.2, 1.0, 5.0, 20.0, 100.0];

/// Every numeric label is a LITERAL string, never a formatted `f64`. Python's `f"{1.0}"` is `"1.0"`
/// and Rust's `{}` is `"1"`, so formatting the value would produce a DISJOINT key set — which the
/// comparator would catch, but as a coverage failure rather than as what it is.
const RHO_EXC: [(&str, f64); 5] =
    [("0.2", 0.2), ("0.5", 0.5), ("1.0", 1.0), ("2.0", 2.0), ("5.0", 5.0)];
const RAMPS: [(&str, f64); 6] =
    [("5.0", 5.0), ("2.0", 2.0), ("1.0", 1.0), ("0.5", 0.5), ("0.3", 0.3), ("0.1", 0.1)];

/// `{tt4:.0}` — Python's `f"{Tt4:.0f}"`.
fn t4(tt4: f64) -> String {
    format!("{tt4:.0}")
}

// ------------------------------------------------------------------------------ the emitters
/// The 20 float keys of `_close`'s dict, under PYTHON's names. `wgas` is the 21st and is not a
/// number; the dump asserts its type and skips it.
fn put_close(c: &mut Cmp, p: &str, s: &CloseState) {
    c.f(&format!("{p}/Tt2"), s.tt2);
    c.f(&format!("{p}/Tt25"), s.tt25);
    c.f(&format!("{p}/Tt3"), s.tt3);
    c.f(&format!("{p}/eta_hpc"), s.eta_hpc);
    c.f(&format!("{p}/eta_lpc"), s.eta_lpc);
    c.f(&format!("{p}/f"), s.f);
    c.f(&format!("{p}/m_hp"), s.m_hp);
    c.f(&format!("{p}/m_imp"), s.m_imp);
    c.f(&format!("{p}/m_lp"), s.m_lp);
    c.f(&format!("{p}/mdot4"), s.mdot4);
    c.f(&format!("{p}/mdot_air"), s.mdot_air);
    c.f(&format!("{p}/n_hp"), s.n_hp);
    c.f(&format!("{p}/n_lp"), s.n_lp);
    c.f(&format!("{p}/phi_hp"), s.phi_hp);
    c.f(&format!("{p}/phi_lp"), s.phi_lp);
    c.f(&format!("{p}/pi_hpc"), s.pi_hpc);
    c.f(&format!("{p}/pi_lpc"), s.pi_lpc);
    c.f(&format!("{p}/pt4"), s.pt4);
    c.f(&format!("{p}/tau_hpc"), s.tau_hpc);
    c.f(&format!("{p}/tau_lpc"), s.tau_lpc);
}

/// The 23 keys `_instant_tail` adds, beside the 20 above.
fn put_instant(c: &mut Cmp, p: &str, i: &Instant2) {
    put_close(c, p, &i.close);
    c.f(&format!("{p}/M9"), i.m9);
    c.f(&format!("{p}/Pc_hp"), i.pc_hp);
    c.f(&format!("{p}/Pc_lp"), i.pc_lp);
    c.f(&format!("{p}/Phi_hp"), i.phi_hp_dot);
    c.f(&format!("{p}/Phi_lp"), i.phi_lp_dot);
    c.f(&format!("{p}/Pt_hp"), i.pt_hp);
    c.f(&format!("{p}/Pt_lp"), i.pt_lp);
    c.f(&format!("{p}/Tt4"), i.tt4);
    c.f(&format!("{p}/Tt45"), i.tt45);
    c.f(&format!("{p}/Tt5"), i.tt5);
    c.d(&format!("{p}/branch_choked"), u64::from(i.branch == Branch::Choked));
    c.f(&format!("{p}/eta_hpt"), i.eta_hpt);
    c.f(&format!("{p}/eta_lpt"), i.eta_lpt);
    c.f(&format!("{p}/nu_hp"), i.nu_hp);
    c.f(&format!("{p}/nu_hpt"), i.nu_hpt);
    c.f(&format!("{p}/nu_lp"), i.nu_lp);
    c.f(&format!("{p}/nu_lpt"), i.nu_lpt);
    c.f(&format!("{p}/pi_hpt"), i.pi_hpt);
    c.f(&format!("{p}/pi_lpt"), i.pi_lpt);
    c.f(&format!("{p}/slip"), i.slip);
    c.f(&format!("{p}/sp_thrust"), i.sp_thrust);
    c.f(&format!("{p}/tau_hpt"), i.tau_hpt);
    c.f(&format!("{p}/tau_lpt"), i.tau_lpt);
}

/// All EIGHT `phi_excursion` keys, including the four NO Python gate reads.
fn put_exc(c: &mut Cmp, p: &str, e: &PhiExcursion) {
    c.f(&format!("{p}/ext_lp"), e.ext_lp);
    c.f(&format!("{p}/ext_hp"), e.ext_hp);
    c.f(&format!("{p}/s_lp"), e.s_lp);
    c.f(&format!("{p}/s_hp"), e.s_hp);
    c.f(&format!("{p}/min_phi_lp"), e.min_phi_lp);
    c.f(&format!("{p}/min_phi_hp"), e.min_phi_hp);
    c.f(&format!("{p}/ratio"), e.ratio);
    c.d(&format!("{p}/npts"), e.npts as u64);
}

fn put_sm(c: &mut Cmp, p: &str, m: &TransientSurgeMargin) {
    c.f(&format!("{p}/margin_min_lp"), m.margin_min_lp);
    c.f(&format!("{p}/margin_min_hp"), m.margin_min_hp);
    c.f(&format!("{p}/steady_min_lp"), m.steady_min_lp);
    c.f(&format!("{p}/steady_min_hp"), m.steady_min_hp);
    c.f(&format!("{p}/phi_surge_lp"), m.phi_surge_lp);
    c.f(&format!("{p}/phi_surge_hp"), m.phi_surge_hp);
    c.d(&format!("{p}/crossed_lp"), u64::from(m.crossed_lp));
    c.d(&format!("{p}/crossed_hp"), u64::from(m.crossed_hp));
    c.d(&format!("{p}/npts"), m.npts as u64);
}

fn put_point(c: &mut Cmp, p: &str, x: &TwoSpoolTransientPoint) {
    c.f(&format!("{p}/s"), x.s);
    c.f(&format!("{p}/nu_lp"), x.nu_lp);
    c.f(&format!("{p}/nu_hp"), x.nu_hp);
    c.f(&format!("{p}/Tt4"), x.tt4);
    c.f(&format!("{p}/slip"), x.slip);
    c.f(&format!("{p}/pi_lpc"), x.pi_lpc);
    c.f(&format!("{p}/pi_hpc"), x.pi_hpc);
    c.f(&format!("{p}/phi_lp"), x.phi_lp);
    c.f(&format!("{p}/phi_hp"), x.phi_hp);
    c.f(&format!("{p}/mdot_air"), x.mdot_air);
    c.f(&format!("{p}/f"), x.f);
    c.f(&format!("{p}/Phi_lp"), x.phi_lp_dot);
    c.f(&format!("{p}/Phi_hp"), x.phi_hp_dot);
    c.f(&format!("{p}/sp_thrust"), x.sp_thrust);
}

/// The counts the Python dump CAN see, plus the six Rust-only arms asserted against ZERO.
///
/// `march_in_advances`, `close_nonreal`, both march truncation arms, the speed floor and the
/// Newton's damper have no Python column — the first two are swallowed inside the shipped body and
/// the rest never fire. § 5.15 predictions 6 and 7 are these two blocks: the dead arms stay at 0
/// across the whole dump, and `illinois_exhausted` comes back 0 at THIS call site against slice Q's
/// 103 of 109 at `_plenum_pt4_at` — the same counter on the opposite population, reported with its
/// grid and never summed with slice Q's.
///
/// Returns the census so a caller that needs the memo's key sequence can read it before the reset.
fn census(c: &mut Cmp, p: &str) -> tcount::Census {
    let t = tcount::take();
    census_from(c, p, &t);
    t
}

fn census_from(c: &mut Cmp, p: &str, t: &tcount::Census) {
    let s = scount::take();
    c.d(&format!("census/{p}/close_calls"), t.close_calls);
    c.d(&format!("census/{p}/close_bracket_fails"), t.close_bracket_fails);
    c.d(&format!("census/{p}/close_nonreal_propagated"), t.close_nonreal);
    c.d(&format!("census/{p}/powers_calls"), t.powers_calls);
    c.d(&format!("census/{p}/instant_calls"), t.instant_calls);
    c.d(&format!("census/{p}/integrate_calls"), t.march_calls);
    c.d(&format!("census/{p}/hi_wall_literal"), t.hi_wall_literal);
    c.d(&format!("census/{p}/hi_wall_map"), t.hi_wall_map);
    c.d(&format!("census/{p}/eig_real"), t.eig_real);
    c.d(&format!("census/{p}/eig_complex"), t.eig_complex);
    c.d(&format!("census/{p}/illinois_calls"), s.illinois_calls);
    c.d(&format!("census/{p}/illinois_evals"), s.illinois_evals);
    c.d(&format!("census/{p}/illinois_exhausted"), s.illinois_exhausted);
    c.d(&format!("census/{p}/match_calls"), t.match_calls);
    assert_eq!((t.march_in_advances, t.close_nonreal), (0, 0),
               "{p}: the low-wall march-in and the non-real guard are measured DEAD");
    assert_eq!((t.march_break_k1, t.march_break_rk), (0, 0),
               "{p}: neither march truncation arm fires on any shipped grid");
    assert_eq!(t.nu_floor_hits, 0, "{p}: the max(0.2, .) speed floor is measured DEAD");
    assert_eq!((t.eq_damped, t.eq_damp_floor), (0, 0),
               "{p}: the Newton's damper and its 1e-30 floor are measured DEAD");
    assert_eq!(s.illinois_exhausted, 0,
               "{p}: PREDICTION 7 — the exhaustion arm is 0 at rung 40's call site");
}

/// The handful of counts the gates below read, summed across a sweep's sections.
///
/// [`tcount::Census`] carries two `Vec`s and is not summable; these are the scalars, added by hand
/// so a gate reads a total it can name rather than one section's slice of it.
#[derive(Default, Clone, Copy)]
struct Totals {
    eig_real: u64,
    eig_complex: u64,
    eq_calls: u64,
    eq_ties: u64,
    eq_damped: u64,
    eq_damp_floor: u64,
    steady_misses: u64,
}

impl Totals {
    fn add(&mut self, t: &tcount::Census) {
        self.eig_real += t.eig_real;
        self.eig_complex += t.eig_complex;
        self.eq_calls += t.eq_calls;
        self.eq_ties += t.eq_ties;
        self.eq_damped += t.eq_damped;
        self.eq_damp_floor += t.eq_damp_floor;
        self.steady_misses += t.steady_misses;
    }
}

// ==============================================================================================
// THE MAIN SWEEP — sections A…L, the CPG grids of both suites
// ==============================================================================================
fn sweep_main(c: &mut Cmp) -> Totals {
    let fl = flight();
    let mut tot = Totals::default();

    // --- A: the constants, and BOTH gases ------------------------------------------------------
    c.f("A/cpg40/R_c", cpg40().r_c());
    c.f("A/cpg44/R_c", cpg44().r_c());
    c.f("A/cpg40/R_t", cpg40().r_t());
    c.f("A/cpg44/R_t", cpg44().r_t());
    let t40 = core(cpg40(), lp_shaped(), hp_shaped(), 1.0);
    let t44 = core(cpg44(), lp_shaped(), hp_shaped(), 1.0);
    for (nm, t) in [("cpg40", &t40), ("cpg44", &t44)] {
        let (tt2, pt2, v0) = t.inlet(&fl);
        c.f(&format!("A/{nm}/tt2"), tt2);
        c.f(&format!("A/{nm}/pt2"), pt2);
        c.f(&format!("A/{nm}/v0"), v0);
    }
    tot.add(&census(c, "A"));

    // --- B: `_close`, driven DIRECTLY ----------------------------------------------------------
    // Seven pairs x two speed pairs x two throttles. The HIGH WALL's contested `min` — the one
    // defect visible in the census ALONE — gets its population here and in every section below.
    let (tt2, pt2, _) = t40.inlet(&fl);
    for (name, ml, mh) in shapes() {
        let t = core(cpg40(), ml, mh, 1.0);
        for (inu, (a, b)) in [(1.0, 1.0), (0.92, 0.96)].iter().enumerate() {
            for tt4 in [1500.0, 1200.0] {
                put_close(c, &format!("B/{name}/{inu}/{}", t4(tt4)),
                          &t.close(*a, *b, tt4, tt2, pt2));
            }
        }
    }
    tot.add(&census(c, "B"));

    // --- C: `_instant` at the MATCHED point ----------------------------------------------------
    for (name, ml, mh) in shapes() {
        let t = core(cpg40(), ml, mh, 1.0);
        for tt4 in [1500.0, 1200.0, 950.0] {
            let od = t.match_point(&fl, tt4);
            let p = format!("C/{name}/{}", t4(tt4));
            c.f(&format!("{p}/nu_lp"), od.n_lp_ratio);
            c.f(&format!("{p}/nu_hp"), od.n_hp_ratio);
            c.f(&format!("{p}/slip"), od.slip);
            c.f(&format!("{p}/phi_lp"), od.phi_lp);
            c.f(&format!("{p}/phi_hp"), od.phi_hp);
            put_instant(c, &format!("{p}/i"),
                        &t.instant(&fl, od.n_lp_ratio, od.n_hp_ratio, tt4));
        }
    }
    tot.add(&census(c, "C"));

    // --- D: `equilibrium` on CPG ---------------------------------------------------------------
    // The exit kind and the pass count are DISCRETE keys. On CPG the primary return always fires
    // (the residual's noise floor is ~1e-14 against the ABSOLUTE 1e-12 bar) — asserted by dumping
    // it, not assumed.
    for (name, ml, mh) in shapes() {
        let t = core(cpg40(), ml, mh, 1.0);
        for tt4 in [1500.0, 1300.0, 1200.0] {
            let (eq, kind, passes) = t.try_equilibrium(&fl, tt4, None).expect("converged");
            let p = format!("D/{name}/{}", t4(tt4));
            put_instant(c, &p, &eq);
            c.d(&format!("{p}/exit_noise"), u64::from(kind == EqExit::Noise));
            c.d(&format!("{p}/passes"), passes as u64);
            c.d(&format!("{p}/powers_calls"), powers_for(kind, passes));
        }
    }
    let (eq, kind, passes) = t40.try_equilibrium(&fl, 1200.0, Some((0.90, 0.95))).expect("start");
    put_instant(c, "D/start", &eq);
    c.d("D/start/exit_noise", u64::from(kind == EqExit::Noise));
    c.d("D/start/passes", passes as u64);
    tot.add(&census(c, "D"));

    // --- E: `lead_threshold` -------------------------------------------------------------------
    // Gate 4's grid PLUS every pair on the DEFAULT `d = 5.0`, which gate 4 never exercises: step 2
    // found that non-uniformity by reading `engine.py:3644`, not the call.
    let t_flat = core(cpg40(), flat(), flat(), 1.0);
    for tt4 in [900.0, 1100.0, 1300.0, 1500.0] {
        c.f(&format!("E/identity/{}", t4(tt4)), t_flat.lead_threshold(&fl, tt4, 25.0, None));
    }
    c.f("E/channel/gas",
        core(Gas::thermally_perfect(), flat(), flat(), 1.0).lead_threshold(&fl, 1100.0, 25.0, None));
    c.f("E/channel/map", t40.lead_threshold(&fl, 1100.0, 25.0, None));
    c.f("E/refute/lp_only",
        core(cpg40(), lp_shaped(), flat(), 1.0).lead_threshold(&fl, 1100.0, 5.0, None));
    c.f("E/refute/hp_only",
        core(cpg40(), flat(), hp_shaped(), 1.0).lead_threshold(&fl, 1100.0, 5.0, None));
    for (name, ml, mh) in shapes() {
        let t = core(cpg40(), ml, mh, 1.0);
        for tt4 in [1500.0, 1300.0, 1100.0] {
            c.f(&format!("E/default_d/{name}/{}", t4(tt4)),
                t.lead_threshold(&fl, tt4, 5.0, None));
        }
    }
    tot.add(&census(c, "E"));

    // --- F: the 2x2 and its two arms -----------------------------------------------------------
    // `rho` enters the way gate 5 does it — `J` built at `rho = 1` (here through `jacobian_at_rho`,
    // which is the port's spelling of Python's save/restore) and the LP row divided afterwards.
    for (name, ml, mh) in shapes() {
        let t = core(cpg40(), ml, mh, 1.0);
        for tt4 in [1500.0, 1200.0, 950.0] {
            let od = t.match_point(&fl, tt4);
            let nu = Some((od.n_lp_ratio, od.n_hp_ratio));
            let j = t.jacobian_at_rho(&fl, tt4, nu, 1e-6, 1.0);
            let p = format!("F/{name}/{}", t4(tt4));
            for r in 0..2 {
                for cc in 0..2 {
                    c.f(&format!("{p}/J/{r}{cc}"), j[r][cc]);
                }
            }
            c.f(&format!("{p}/bc"), j[0][1] * j[1][0]);
            for (ir, rho) in RHO_EIG.iter().enumerate() {
                let jr = [[j[0][0] / rho, j[0][1] / rho], [j[1][0], j[1][1]]];
                let (lo, hi) = TwoSpoolTransientCore::eigenvalues(jr);
                c.f(&format!("{p}/eig/{ir}/lo"), lo);
                c.f(&format!("{p}/eig/{ir}/hi"), hi);
            }
            let band = t.oscillatory_band(&fl, tt4, nu);
            c.d(&format!("{p}/band_is_none"), u64::from(band.is_none()));
            if let Some((blo, bhi)) = band {
                c.f(&format!("{p}/band_lo"), blo);
                c.f(&format!("{p}/band_hi"), bhi);
                let (a, b, cc, d) = (j[0][0], j[0][1], j[1][0], j[1][1]);
                let mid = (blo * bhi).sqrt();
                for (tag, rr) in [("mid", mid), ("lo2", 0.5 * blo), ("hi2", 2.0 * bhi)] {
                    c.f(&format!("{p}/disc/{tag}"),
                        (a / rr - d) * (a / rr - d) + 4.0 * b * cc / rr);
                }
            }
            c.f(&format!("{p}/damping"), t.damping_ratio_max(&fl, tt4, nu));
        }
    }
    tot.add(&census(c, "F"));

    // --- G: the march, EVERY point -------------------------------------------------------------
    // Two cells at `s_end = 1.2, ds = 0.05` (gate 7's pair, the only one of the four in use where
    // `int(round(s_end/ds))` is not exact) and one at rung 44's own default ramp, 151 points.
    for (tag, gas, sname, tt4_lo, dt, r_ramp, s_end, ds) in [
        ("g7", cpg40(), "flow_press", 1100.0, 50.0, 0.5, 1.2, 0.05),
        ("steep", cpg40(), "steep", 1100.0, 50.0, 0.5, 1.2, 0.05),
        ("r44", cpg44(), "hp_only", 1000.0, 400.0, 0.5, 3.0, 0.02),
    ] {
        let (ml, mh) = shape(sname);
        let t = core(gas, ml, mh, 1.0);
        let od_lo = t.match_point(&fl, tt4_lo);
        let nu0 = (od_lo.n_lp_ratio, od_lo.n_hp_ratio);
        let sched = |x: f64| tt4_lo + dt * 1.0f64.min(x / r_ramp);
        let pts = t.integrate(&fl, sched, nu0, s_end, ds);
        c.d(&format!("G/{tag}/npts"), pts.len() as u64);
        for (ip, x) in pts.iter().enumerate() {
            put_point(c, &format!("G/{tag}/{ip}"), x);
        }
        tot.add(&census(c, &format!("G/{tag}")));
    }

    // --- H: `slip_excursion` + gate 7's bisection ----------------------------------------------
    // Reproduced in full, INCLUDING `elo * ehi < 0.0` — the bracket check four lines ahead of the
    // headline 0.2 margin, which step 1 measured to be what a truncated step count actually breaks.
    let mut t = core(cpg40(), lp_shaped(), hp_shaped(), 1.0);
    let (tt4_lo, dt) = (1100.0, 50.0);
    let sc = t.lead_threshold(&fl, tt4_lo, 5.0, None);
    c.f("H/sigma_crit", sc);
    let exc = |t: &mut TwoSpoolTransientCore, rho: f64| {
        t.rho = rho;
        t.slip_excursion(&fl, tt4_lo, dt, 0.5, 1.2, 0.05)
    };
    let (mut lo, mut hi) = (0.6 * sc, 1.6 * sc);
    let (elo, ehi) = (exc(&mut t, lo), exc(&mut t, hi));
    c.f("H/bisect/elo", elo);
    c.f("H/bisect/ehi", ehi);
    for ib in 0..18 {
        let mid = 0.5 * (lo + hi);
        let em = exc(&mut t, mid);
        c.f(&format!("H/bisect/{ib}/mid"), mid);
        c.f(&format!("H/bisect/{ib}/exc"), em);
        if em * elo > 0.0 { lo = mid } else { hi = mid }
    }
    c.f("H/bisect/rho_star", 0.5 * (lo + hi));
    t.rho = 1.0;
    // THE TWO RUNNING-LINE REFERENCES, POINTWISE. At `r_ramp = 0.5` the extremum lands exactly
    // where the ramp SATURATES (`u == 1`), so the linear reference IS the endpoint match bit for
    // bit and unifying the two moves nothing; `r_ramp = 3.0` never saturates and they differ 2.4 %.
    for (tag, rr) in [("0.5", 0.5), ("3.0", 3.0)] {
        c.f(&format!("H/ref/{tag}/slip_excursion"),
            t.slip_excursion(&fl, tt4_lo, dt, rr, 1.2, 0.05));
        let sched = |x: f64| tt4_lo + dt * 1.0f64.min(x / rr);
        let od_lo = t.match_point(&fl, tt4_lo);
        let od_hi = t.match_point(&fl, tt4_lo + dt);
        let nu0 = (od_lo.n_lp_ratio, od_lo.n_hp_ratio);
        let pts = t.integrate(&fl, sched, nu0, 1.2, 0.05);
        for (ip, x) in pts.iter().enumerate() {
            let u = (x.tt4 - tt4_lo) / dt;
            let linear = od_lo.slip + u * (od_hi.slip - od_lo.slip);
            let instant = t.match_point(&fl, x.tt4).slip;
            c.f(&format!("H/ref/{tag}/{ip}/err_linear"), x.slip - linear);
            c.f(&format!("H/ref/{tag}/{ip}/err_instant"), x.slip - instant);
        }
    }
    for (tag, rho) in [("0.5", 0.5), ("2.0", 2.0)] {
        t.rho = rho;
        c.f(&format!("H/rho/{tag}/slip_excursion"),
            t.slip_excursion(&fl, tt4_lo, dt, 0.5, 1.2, 0.05));
    }
    t.rho = 1.0;
    tot.add(&census(c, "H"));

    // --- I: `phi_excursion`, rung 44's grid ----------------------------------------------------
    for name in RUNG44_SHAPES {
        let (ml, mh) = shape(name);
        let t = core(cpg44(), ml, mh, 1.0);
        put_exc(c, &format!("I/acc/{name}"), &t.phi_excursion(&fl, 1000.0, 400.0, 0.5, 3.0, 0.02));
        put_exc(c, &format!("I/dec/{name}"),
                &t.phi_excursion(&fl, 1400.0, -400.0, 0.5, 3.0, 0.02));
        let band = t.oscillatory_band(&fl, 1200.0, None);
        c.d(&format!("I/band_is_none/{name}"), u64::from(band.is_none()));
        c.f(&format!("I/damping/{name}"), t.damping_ratio_max(&fl, 1200.0, None));
    }
    tot.add(&census(c, "I/shapes"));

    for (tag, rho) in RHO_EXC {
        let t = core(cpg44(), lp_shaped(), hp_shaped(), rho);
        put_exc(c, &format!("I/rho/{tag}"), &t.phi_excursion(&fl, 1000.0, 400.0, 0.5, 3.0, 0.02));
    }
    tot.add(&census(c, "I/rho"));

    let t = core(cpg44(), lp_shaped(), hp_shaped(), 1.0);
    for (tag, r) in RAMPS {
        put_exc(c, &format!("I/ramp/{tag}"), &t.phi_excursion(&fl, 1000.0, 400.0, r, 6.0, 0.02));
    }
    tot.add(&census(c, "I/ramp"));

    // --- J: the memo's KEY SEQUENCE ------------------------------------------------------------
    // The equivalence relation `round(Tt4, 3)` is the one thing NO value key can see: probe 1
    // measured the single collision that exists moving 0 reported values, and confirmed it FIRES
    // inside the measured set. `collide` is the case where it does.
    for (tag, r_ramp, s_end) in [("collide", 5.0, 6.0), ("default", 0.5, 3.0)] {
        let e = t.phi_excursion(&fl, 1000.0, 400.0, r_ramp, s_end, 0.02);
        put_exc(c, &format!("J/{tag}"), &e);
        let cen = tcount::take();
        c.d(&format!("J/{tag}/match_calls"), cen.match_calls);
        c.d(&format!("J/{tag}/steady_misses"), cen.steady_misses);
        c.d(&format!("J/{tag}/steady_calls"), cen.steady_calls);
        for (ik, k) in cen.steady_keys.iter().enumerate() {
            c.f(&format!("J/{tag}/key/{ik}"), *k);
        }
        // THE TWO KEYING SCHEMES, off the SAME trajectory. `steady_misses` is the rounded
        // scheme's cardinality by construction; the exact scheme's is the distinct raw `Tt4` over
        // EVERY lookup, which is why `steady_tt4_all` exists — counting distinct values among the
        // MISSES would measure the rounded relation against itself and always report 0 collisions.
        // A rounded HIT whose raw `Tt4` is new IS the collision.
        let exact: BTreeSet<u64> = cen.steady_tt4_all.iter().map(|x| x.to_bits()).collect();
        c.d(&format!("J/{tag}/keys_rounded"), cen.steady_misses);
        c.d(&format!("J/{tag}/keys_exact"), exact.len() as u64);
        c.d(&format!("J/{tag}/collisions"), exact.len() as u64 - cen.steady_misses);
        census_from(c, &format!("J/{tag}"), &cen);
        tot.add(&cen);
    }

    // --- K: `transient_surge_margin` -----------------------------------------------------------
    for name in RUNG44_SHAPES {
        let (ml, mh) = shape(name);
        let t = core(cpg44(), ml.with_phi_surge(0.86), mh.with_phi_surge(0.90), 1.0);
        put_sm(c, &format!("K/def/{name}"),
               &t.transient_surge_margin(&fl, 1000.0, 400.0, 0.5, 3.0, 0.02));
    }
    let t = core(cpg44(), lp_shaped().with_phi_surge(0.86), hp_shaped().with_phi_surge(0.90), 1.0);
    put_sm(c, "K/acc", &t.transient_surge_margin(&fl, 1000.0, 400.0, 0.3, 3.0, 0.02));
    put_sm(c, "K/dec", &t.transient_surge_margin(&fl, 1400.0, -400.0, 0.3, 3.0, 0.02));
    tot.add(&census(c, "K"));

    // --- L: the `lp_disabled` REDUCE -----------------------------------------------------------
    // Gate 2 at its OWN gas — `_single_design` is built on `Gas.reacting_equilibrium()`, neither
    // CPG one. Asserted bit-for-bit on BOTH sides, so the dump would fail if the dispatch stopped
    // being exact.
    let deg = SpoolTransient::new(single_design(), fl, 1.0, hp_shaped());
    let refr = SpoolTransient::new(single_design(), fl, 1.0, hp_shaped());
    for tt4 in [1500.0, 1200.0] {
        let a = deg.equilibrium(&fl, tt4, None);
        let b = refr.equilibrium(&fl, tt4, None);
        assert_eq!((a.nu, a.pi_c, a.tau_c), (b.nu, b.pi_c, b.tau_c), "lp_disabled reduce at {tt4}");
        let p = format!("L/{}", t4(tt4));
        c.f(&format!("{p}/nu"), a.nu);
        c.f(&format!("{p}/pi_c"), a.pi_c);
        c.f(&format!("{p}/tau_c"), a.tau_c);
        c.f(&format!("{p}/tau_t"), a.tau_t);
        c.f(&format!("{p}/mdot_air"), a.mdot_air);
        c.f(&format!("{p}/f"), a.f);
        c.f(&format!("{p}/Phi"), a.phi);
        c.f(&format!("{p}/sp_thrust"), a.sp_thrust);
    }
    tot.add(&census(c, "L"));
    tot
}

// ==============================================================================================
// THE REACTING SWEEP — sections P…S
// ==============================================================================================
fn sweep_equil(c: &mut Cmp) -> Totals {
    let fl = flight();
    let mut tot = Totals::default();

    // --- P: probe 4's TWELVE cells, exactly ----------------------------------------------------
    // Taken from `probe_r4.py` rather than reconstructed from § 5.15's prose table, because the
    // arm's whole value is that it can reproduce probe 4's 5-of-12 and 10-of-12 under CPython.
    for (name, ml, mh) in [("shaped", lp_shaped(), hp_shaped()), ("flat", flat(), flat())] {
        let t = core(Gas::reacting_equilibrium(), ml, mh, 1.0);
        for tt4 in [1500.0, 1450.0, 1400.0, 1300.0, 1200.0, 1100.0] {
            let (eq, kind, passes) = t.try_equilibrium(&fl, tt4, None).expect("converged");
            let p = format!("P/{name}/{}", t4(tt4));
            put_instant(c, &p, &eq);
            c.d(&format!("{p}/exit_noise"), u64::from(kind == EqExit::Noise));
            c.d(&format!("{p}/passes"), passes as u64);
            c.d(&format!("{p}/powers_calls"), powers_for(kind, passes));
        }
        tot.add(&census(c, &format!("P/{name}")));
    }

    // --- Q: gate 1's reacting REDUCE -----------------------------------------------------------
    let t = core(Gas::reacting_equilibrium(), lp_shaped(), hp_shaped(), 1.0);
    for tt4 in [1500.0, 1200.0] {
        let od = t.match_point(&fl, tt4);
        let p = format!("Q/{}", t4(tt4));
        c.f(&format!("{p}/nu_lp"), od.n_lp_ratio);
        c.f(&format!("{p}/nu_hp"), od.n_hp_ratio);
        c.f(&format!("{p}/pi_lpc"), od.base.pi_lpc);
        c.f(&format!("{p}/pi_hpc"), od.base.pi_hpc);
        c.f(&format!("{p}/mdot_air"), od.base.mdot_air);
        c.f(&format!("{p}/slip"), od.slip);
    }
    tot.add(&census(c, "Q"));

    // --- R: gate 5's REACTING Jacobians --------------------------------------------------------
    for (name, ml, mh) in shapes() {
        let t = core(Gas::reacting_equilibrium(), ml, mh, 1.0);
        for tt4 in [1500.0, 1200.0, 950.0] {
            let od = t.match_point(&fl, tt4);
            let nu = Some((od.n_lp_ratio, od.n_hp_ratio));
            let j = t.jacobian_at_rho(&fl, tt4, nu, 1e-6, 1.0);
            let p = format!("R/{name}/{}", t4(tt4));
            for r in 0..2 {
                for cc in 0..2 {
                    c.f(&format!("{p}/J/{r}{cc}"), j[r][cc]);
                }
            }
            c.f(&format!("{p}/bc"), j[0][1] * j[1][0]);
            for (ir, rho) in RHO_EIG.iter().enumerate() {
                let jr = [[j[0][0] / rho, j[0][1] / rho], [j[1][0], j[1][1]]];
                let (lo, hi) = TwoSpoolTransientCore::eigenvalues(jr);
                c.f(&format!("{p}/eig/{ir}/lo"), lo);
                c.f(&format!("{p}/eig/{ir}/hi"), hi);
            }
        }
    }
    tot.add(&census(c, "R"));

    // --- S: the reacting forward closure -------------------------------------------------------
    let t = core(Gas::reacting_equilibrium(), lp_shaped(), hp_shaped(), 1.0);
    let (tt2, pt2, v0) = t.inlet(&fl);
    c.f("S/tt2", tt2);
    c.f("S/pt2", pt2);
    c.f("S/v0", v0);
    for (inu, (a, b)) in [(1.0, 1.0), (0.92, 0.96)].iter().enumerate() {
        put_close(c, &format!("S/close/{inu}"), &t.close(*a, *b, 1350.0, tt2, pt2));
        put_instant(c, &format!("S/inst/{inu}"), &t.instant(&fl, *a, *b, 1350.0));
    }
    tot.add(&census(c, "S"));
    tot
}

/// `_powers` calls implied by an exit — the SAME derivation the Python dump uses to recover the
/// exit kind, run in the OPPOSITE direction so the two instruments disagree if either is wrong.
fn powers_for(kind: EqExit, passes: usize) -> u64 {
    match kind {
        EqExit::Noise => 3 * TwoSpoolTransientCore::EQ_MAX as u64,
        EqExit::Primary => 3 * passes as u64 + 1,
    }
}

fn single_design() -> Engine {
    build_turbojet(
        Gas::reacting_equilibrium(), PI_HPC, TT4, 50_000.0,
        Losses { pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
                 pi_n: 0.98, nozzle_convergent: true, ..Losses::default() },
    )
}

// ==============================================================================================
// THE GATES
// ==============================================================================================

/// **PREDICTION 1, THE CPG HALF.** Every one of the 6 853 main-arm keys, bit for bit against PyPy.
///
/// The census assertions inside [`census_from`] carry predictions 6 and 7 with it: the six dead
/// arms stay at zero across the whole dump, and `illinois_exhausted` is 0 at rung 40's call site.
#[test]
fn oracle_main_is_bit_exact_against_pypy() {
    let mut c = Cmp::new(load(ORACLE_MAIN));
    let tot = sweep_main(&mut c);
    c.finish();
    // PREDICTION 9's grid-local twin. `rung40.rs` gates gate 5's OWN grid at 124 real / 2 complex
    // (CPG) and 121/5 (reacting); section F is an INDEPENDENT construction of the same 7x3x6 sweep
    // and lands on the same CPG pair, which is what makes the two a corroboration rather than one
    // number quoted twice. The per-section counts are compared as dump keys above; this is the
    // total, read against the grid size so a silently shorter sweep cannot satisfy it.
    assert_eq!((tot.eig_real, tot.eig_complex), (124, 2),
               "section F's CPG eigenvalue split — gate 5's own grid, reached independently");
    assert_eq!(tot.eig_real + tot.eig_complex, 7 * 3 * 6, "the grid size itself");
    assert_eq!(tot.eq_damped + tot.eq_damp_floor, 0,
               "PREDICTION 6 — the Newton's damper and its 1e-30 floor stay dead at scale");
}

/// **PREDICTION 1, THE EXPOSED HALF.** The reacting-gas `equilibrium` keys, bit for bit.
///
/// § 5.15 registered this as the one place 100 % could fail, and pre-committed § 9 Decision 1's
/// Option B — an individually-adjudicated fragile set with a published deviation distribution — as
/// the route if it did. It did not: the 12 cells were spiked against PyPy before this file existed
/// and agree on the exit kind, the pass count and both converged speeds, so the arm ships at
/// bit-equality and Option B stays unused. That the prediction named the failure mode in advance is
/// what makes this a measurement rather than a lucky pass.
#[test]
fn oracle_equil_is_bit_exact_against_pypy() {
    let mut c = Cmp::new(load(ORACLE_EQ));
    let tot = sweep_equil(&mut c);
    c.finish();
    assert_eq!((tot.eig_real, tot.eig_complex), (121, 5),
               "section R's REACTING eigenvalue split — the other half of prediction 9's 245/7,                 which is the two gases SUMMED and never a per-gas bar");
    assert_eq!(tot.eig_real + tot.eig_complex, 7 * 3 * 6, "the grid size itself");
    assert_eq!(tot.eq_calls, 12, "probe 4's twelve cells, and no more");
}

/// **THE `best` TIE-BREAK — REGISTERED AS UNWITNESSED, MEASURED HERE AS REACHED.**
///
/// `try_equilibrium` keeps its best iterate under a STRICT `<`, so a tie keeps the EARLIEST one; a
/// `<=` keeps the latest, and `best` is READ only by the noise-floor exit. Step 1 measured that
/// spelling invisible to all 1 174 smoke values — on that grid no two Newton passes leave exactly
/// equal residuals — and registered *"step 4's larger reacting grid is where it could be, and that
/// is registered rather than assumed covered"*.
///
/// **It is reached, and the split is the whole content:**
///
/// | arm | `equilibrium` calls | ties | where |
/// |---|---|---|---|
/// | CPG (section D) | 22 | **0** | — |
/// | reacting (section P) | 12 | **22** | `flat/1450` (17), `flow_press/1100` (4), `flat/1300` (1) |
///
/// All three tying cells take the NOISE exit, which is the only exit that reads `best`; no CPG cell
/// ties at all, because the residual there falls under the absolute `1e-12` bar in four or five
/// passes instead of plateauing for eighty. Injecting `<=` then moves **9 of the 1 120** reacting
/// keys — every one in `P/flat/1300`, the single-tie cell, at the last bit. The two 4-and-17-tie
/// cells do not move, because a tie only reaches the return when the tied residual is also the
/// minimum. So step 1's injection row *"`best` keeps the LATEST tie -> 0 keys -> INVISIBLE"* is
/// CORRECTED by this grid rather than confirmed, and the count is asserted PER ARM: a sum would
/// have let the reacting 22 hide behind a CPG zero, which is what a registered sum instead of a
/// gated split costs.
#[test]
fn the_newtons_tie_break_is_reached_only_on_the_reacting_arm() {
    let mut c = Cmp::new(load(ORACLE_MAIN));
    let a = sweep_main(&mut c);
    let mut c2 = Cmp::new(load(ORACLE_EQ));
    let b = sweep_equil(&mut c2);
    assert_eq!((a.eq_calls, a.eq_ties), (22, 0),
               "the CPG arm: 21 cells + the explicit start, and no residual ever repeats");
    assert_eq!((b.eq_calls, b.eq_ties), (12, 22),
               "the reacting arm: probe 4's twelve cells, 22 ties, all in noise-exit cells");
}

/// **THE CPython ARM — a DETECTOR with a measured sensitivity, not coverage.**
///
/// `slice_n_oracle.rs` states the rule this arm has to INVERT, and the reason is measured. There:
/// *"iteration counts are not interpreter-invariant… the branch verdicts ARE, because a verdict is
/// a comparison and not an iteration."* Here the verdicts are exactly what moves. Probe 4 measured
/// the reacting Newton's exit CLASSIFICATION flipping in **5 of 12** cells between CPython and PyPy
/// and its iteration count in **10 of 12**, because the `1e-12` acceptance bar is ABSOLUTE while
/// the equilibrium sub-solve inside `_close` leaves ~1e-10 of noise in `Phi`: whether a pass ever
/// squeaks under the bar is decided BELOW the solver's own floor, so one last-bit difference in
/// `exp`/`log` re-rolls it. Copying slice N's precedent would have shipped a gate that fails.
///
/// So the arm is TIERED, and the tier is chosen by GAS rather than by file:
///
/// * **CPG keys** — bit-equal, the same bar as against PyPy. This is most of the dump and it is
///   where the arm functions as coverage.
/// * **reacting FLOAT keys** — a published relative bar. They differ at the solver's noise floor.
/// * **reacting DISCRETE keys** (`exit_noise`, `passes`, `powers_calls`) — NOT asserted equal. They
///   are counted, and the count is asserted NON-ZERO: the port's claim is that it matches PyPy, and
///   that CPython disagrees is a property of CPython, which a libm change could move. Gating the
///   exact 5-of-12 would make an interpreter upgrade look like a port regression.
///
/// [`is_reacting`] is what routes a key, and section **L** is why it is a gas test and not a file
/// test: `L` lives in the MAIN dump but its single-spool design is built on
/// `Gas::reacting_equilibrium()`.
#[test]
fn oracle_matches_cpython() {
    let mut c = Cmp::new(load(ORACLE_CPYTHON));
    c.cpython = true;
    sweep_main(&mut c);
    sweep_equil(&mut c);
    let flips = c.discrete_flips.clone();
    let drifts = c.float_drifts.len();
    // Everything that is NOT a tolerated reacting/iteration disagreement, or is over its class bar,
    // is a hard failure — and `finish` reports the never-compared half in the same panic.
    c.finish();

    // THE DETECTOR, read against probe 4's own numbers. Five exit-kind flips and ten differing pass
    // counts were measured on this exact grid BEFORE the port existed; the dump reproduces both.
    let exits = flips.iter().filter(|(k, _)| k.ends_with("/exit_noise")).count();
    let passes = flips.iter().filter(|(k, _)| k.ends_with("/passes")).count();
    assert!(exits > 0 && passes > 0,
            "PROBE 4's detector reported NOTHING — the reacting Newton's exit branch is supposed              to be interpreter-unstable here, and an arm that finds no flip has stopped measuring");

    // PUBLISHED, NOT GATED. A libm change can move these, and gating them would make an interpreter
    // upgrade read as a port regression: the port's claim is bit-equality against PYPY, and says
    // nothing about CPython. Measured 2026-08-18 on CPython 3.14.3: 5 of 12 and 10 of 12 — probe
    // 4's numbers exactly, from an independently built grid.
    let counts_cpg = flips.iter().filter(|(k, _)| is_iteration_count(k) && !is_reacting(k)).count();
    let counts_re = flips.iter().filter(|(k, _)| is_iteration_count(k) && is_reacting(k)).count();
    println!("[cpython arm] exit-kind flips {exits}/12, pass-count differences {passes}/12,               {} tolerated discrete keys, of which iteration counts {counts_cpg}/16 CPG and               {counts_re}/6 reacting, and {drifts} drifting floats", flips.len());

    // THE CPG HALF IS THE COVERAGE HALF, AND IT IS THE ONE THAT MUST NOT MOVE. Every non-reacting
    // float is bit-equal (6 410 of them, asserted above by the tier table), and the only CPG-half
    // DISCRETE keys allowed to differ are iteration counts. Asserted as a SUBSET, not a count: a
    // third `illinois_evals` drifting is a CPython property and must not fail, while anything that
    // is not an iteration count drifting is a real disagreement and must.
    let rogue: Vec<&String> = flips.iter()
        .filter(|(k, _)| !is_reacting(k) && !is_iteration_count(k))
        .map(|(_, m)| m).collect();
    assert!(rogue.is_empty(),
            "a CPG-half discrete key drifted that is not an iteration count: {rogue:?}");
}
