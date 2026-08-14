//! Turbojet cycle simulator.
//!
//! A station-by-station model of a single-spool turbojet (Brayton cycle), built in cumulative
//! **rungs** — each adds one physical effect and is anchored to a published case.
//!
//! **The deliverable is understanding, not the tool.** The code is the medium that forces every
//! thermodynamic assumption into the open. See `CLAUDE.md` for the rung index and
//! `docs/rungN-spec.md` for each rung's derivation, assumptions and verification gates.
//!
//! # Port status
//!
//! This crate is the Rust port of `turbojet/`, following `docs/plans/todo-rust-port.md`. The
//! Python remains the **oracle** the Rust is validated against, and is deleted at phase 8.
//!
//! **Phases 0–3 are complete and green**: the gas core ([`gas`], rungs 1–6), the five
//! components ([`components`]), the design-point cycle ([`engine`]) and the whole NOx / mixing /
//! nozzle strand ([`nox`], rungs 7–24). Every oracle holds the port to **bit-equality** against
//! PyPy rather than to a tolerance, because phase 2 measured that bar achievable and found that
//! a tolerance had let a real transcription defect ride for a whole phase (§ 4.2 of the plan).
//!
//! **Phase 3 ran in slices grouped by DEPENDENCY rather than by rung number** — because rungs 20
//! and 21 thread rung 19's lift through machinery (rung 10's quench, rung 13's bell) that arrives
//! well after it. All five are complete and green, all in [`nox`]:
//!
//! * **A — rungs 7/8/9/19**: the extended-Zeldovich integrator, the two-zone primary/dilution
//!   split, the rich-primary bell, and rung 19's two channels for lifting the equilibrium-[O]
//!   lower bound. 1806/1806 bit-exact.
//! * **B — rungs 10/11/12/20**: the finite-rate quench, the jet-entrainment model that derives
//!   its time, the two-stream variance layer, and the lift threaded through the quench.
//!   2507/2507 bit-exact — and its own shape key NARROWED a shipped claim.
//! * **C — rungs 13/15/16/18/21**: the mixture-fraction PDF family — the β-PDF closure on the
//!   ideal bell, through the quench, per pocket, from a transported variance ODE, and the lift
//!   threaded through all of them. 2448/2448 bit-exact.
//! * **D — rungs 22/23/24**: the resolved y-z cross-plane, that plane developed in TIME so each
//!   pocket carries its own dwell, and the same plane with each cell relaxing at its own
//!   gradient-derived rate. 462/462 bit-exact — and it CORRECTED two of the source's own claims
//!   of exactness, both caused by applying an operation inside an accumulation and removing it
//!   outside (see `nox`'s cross-plane section note).
//! * **E — rungs 14/17**: the nozzle strand — the frozen↔shifting thrust bracket and the
//!   combustor-mixing-fidelity ladder of the dropped-NO-clamp margin. 513/513 bit-exact. It
//!   corrected a THIRD claim of exactness (the frozen reduce is exact in algebra only, and its
//!   floor is the entropy ROUTE rather than the bisection's stopping rule) and LOCATED a band
//!   edge the source states without measuring: past J ≈ 2460 the bulk margin goes dormant while
//!   the per-pocket one keeps RISING, so the rung's headline predicate is about `a_bulk` and not
//!   about the ladder.
//!
//! **Phase 4 — the nozzle & turbine marches (rungs 25–30) — is under way in [`march`]**, in three
//! dependency slices:
//!
//! * **F — rungs 25/26 (SHIPPED)**: the Damköhler flow between rung-14's bounds, its closed-form
//!   irreversible-fast ceiling, and the anchored GRI-Mech clock that lets the relaxation freeze
//!   partway down the nozzle. 912/912 bit-exact — and against CPython the same dump is only 54 %
//!   identical (velocity 3/88), because a 400-step march carries a last-bit difference all the way
//!   to the exit. It also found a shipped test bar that holds only at the three temperatures it
//!   was evaluated at (§ 4.12 of the plan).
//! * **G — rungs 27/28 (SHIPPED)**: the trace-NO relaxation that DERIVES the frozen-NO assumption
//!   every NO number has carried since rung 7, and that clock re-read on slice F's relaxing pool.
//!   776/776 bit-exact — and only **8.0 %** CPython-identical, the sharpest dump in the port,
//!   because every quantity is a ratio of Arrhenius rates read off a marched trajectory. Slice F's
//!   copy-vs-rederivation discriminator made two pre-registered predictions here and both held.
//! * **H — rungs 29/30 (SHIPPED)**: the work-limited turbine bracket that asks whether FREEZING
//!   the turbine is earned, and the choked convergent nozzle that asks whether FULL EXPANSION is.
//!   270/270 bit-exact. Rung 29's "ratio ≠ energy" came out stronger than the source states it —
//!   the super-equilibrium ratio is ANTI-correlated with the energy, largest (993×) exactly where
//!   the bracket is worth least.
//!
//! **PHASE 4 IS COMPLETE.** Note that three different spellings of "raise to a power" live in
//! phase 4 alone — a libm `pow` for `** 0.5`, the real `sqrt` instruction for `math.sqrt`, and a
//! product for a small integer exponent — and each site says which rule applies, because applying
//! any one by habit is a silent one-bit defect.
//!
//! **Phase 5 — the steady matchers — is under way in [`matcher`] and [`map`]**, in dependency
//! slices:
//!
//! * **I — rungs 31/33 (SHIPPED)**: off-design matching, where the operating point stops being an
//!   input. Fixed hardware (two throat areas from one design run) pins the turbine by geometry
//!   and the shaft balance hands back the compressor, so `pi_c` and `mdot_air` become OUTPUTS;
//!   below the nozzle-unchoke boundary that pin is void and a second branch re-solves it.
//!   3951/3951 bit-exact, including 961 discrete keys and every bracket endpoint.
//! * **J — rung 32 (SHIPPED)**: the component map, in [`map`]. Slice I's fixed hardware IS a
//!   map for the *work*, but not for the pressure ratio, the mass flow or the shaft speed — so
//!   rung 32 hangs an analytic efficiency island and a family of speed lines off the same solve,
//!   making `eta_c` and `eta_t` OUTPUTS of an outer secant around slice I's inner loop and
//!   attaching `N` by inverting a speed line. 7252/7252 bit-exact. **It found the SECOND live
//!   site of point 2's hook below** — `MapMatcher::operating_point` calls `solve_turbine`, and
//!   phase 6's rung 34 subclasses `MapMatcher` overriding that method and neither of the two
//!   that call it — a site the phase-5 census structurally could not name, because it enumerated
//!   method triples rather than call sites inside a class that did not yet exist in Rust. It is
//!   also the port's first THREE-DEEP nest of solves, and the outer one turns out to be the
//!   STABLE one: slice I's 7↔200 inner-pass flip reaches 5 of 144 cells and the outer secant
//!   count NONE of them.
//!
//! Slice I is where the port first had to do three things it had never done, and each one left a
//! mark on the crate rather than only on the module:
//!
//! 1. **MARCH PAST A FAILURE.** Rung 33 steps a bracket inward while catching what Python raises
//!    as `AssertionError`, so some of the crate's `assert!`s are now [`gas::Abort`]s instead —
//!    under one rule, applied per site, with both of its edges measured. Every conversion is an
//!    additive `try_` twin whose panicking original delegates to it, so no earlier gate moved.
//! 2. **DISPATCH VIRTUALLY.** `solve_turbine` is called on `self` inside rung 31's own body and
//!    is overridden by rung 34 — in the NEXT phase. It ships hookable ([`matcher::MatcherHooks`])
//!    rather than being refactored later under a gate.
//! 3. **REPRODUCE A LOOP THAT DOES NOT CONVERGE.** The joint `(f, pt4)` fixed point exhausts its
//!    200-pass cap with no assert on the production gas, so its answer is the 200th iterate of a
//!    fixed count and every pass has to reproduce exactly.
//!
//! The remaining transient ladders arrive in phases 6–7 and leave the design run untouched, as
//! they do in the Python.
//!
//! # Porting rules that are NOT optional
//!
//! Python's `**` is a libm `pow` call. The faithful Rust spelling is SPLIT — `t * t` for the
//! square, [`gas::powp`] for every higher power and for `** 0.5` — and both halves were
//! measured, not guessed. `tests/porting_rules.rs` keeps the measurement honest. Getting this
//! wrong is silent: it costs one bit in a high-order polynomial term, which a safeguarded
//! Newton then amplifies into something that looks like a solver artefact.
//!
//! # Conventions
//!
//! - **SI throughout** (K, Pa, kg/s, m/s, J/kg).
//! - The cycle runs in **total (stagnation)** quantities `Tt`, `pt`; static conversion happens
//!   only at the nozzle exit, for exhaust velocity.
//! - **Conservation checks are `assert!`**, not `debug_assert!` — they run on every execution,
//!   in every profile. That is the working contract, not a debugging convenience.

pub mod bleed;
pub mod components;
pub mod engine;
pub mod gas;
pub mod map;
pub mod march;
pub mod matcher;
pub mod nox;
pub mod two_spool;
