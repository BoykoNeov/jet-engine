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
//! * **G — rungs 27/28** and **H — rungs 29/30**: next.
//!
//! The remaining off-design and transient ladders arrive in phases 5–7 and leave this design run
//! untouched, as they do in the Python.
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

pub mod components;
pub mod engine;
pub mod gas;
pub mod march;
pub mod nox;
