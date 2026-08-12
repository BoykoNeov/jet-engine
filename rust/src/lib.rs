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
//! **Phases 0–2 are complete and green**: the gas core ([`gas`], rungs 1–6), the five
//! components ([`components`]) and the design-point cycle ([`engine`]). All three oracles hold
//! the port to **bit-equality** against PyPy — 3232/3232 gas values, 1481/1481 cycle values and
//! 1790/1790 NOx values — rather than to a tolerance, because phase 2 measured that bar
//! achievable and found that a tolerance had let a real transcription defect ride for a whole
//! phase (§ 4.2 of the plan).
//!
//! **Phase 3 is in progress**, in slices grouped by DEPENDENCY rather than by rung number —
//! because rungs 20 and 21 thread rung 19's lift through machinery (rung 10's quench, rung 13's
//! bell) that arrives well after it. Three slices are complete and green, all in [`nox`]:
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
//!
//! Remaining: the nozzle strand (rungs 14/17, independent of the mixing closures and portable at
//! any point) and the resolved cross-plane (22/23/24). Off-design and transient ladders arrive in
//! phases 5–7 and leave this design run untouched, as they do in the Python.
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
pub mod nox;
