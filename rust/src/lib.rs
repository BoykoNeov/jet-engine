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
//! components ([`components`]) and the design-point cycle ([`engine`]). Both oracles hold the
//! port to **bit-equality** against PyPy — 3232/3232 gas values and 1481/1481 cycle values —
//! rather than to a tolerance, because phase 2 measured that bar achievable and found that a
//! tolerance had let a real transcription defect ride for a whole phase (§ 4.2 of the plan).
//!
//! Phase 3 (NOx and mixing, rungs 7–24) is **not yet authorised**. Off-design and transient
//! ladders arrive in phases 5–7 and leave this design run untouched, as they do in the Python.
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
