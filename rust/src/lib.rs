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
//! This crate is the Rust port of `turbojet/`, following `docs/plans/todo-rust-port.md`.
//! Phase 1 (the gas core, rungs 1–6) is in progress; the Python remains the **oracle** the
//! Rust is validated against, and is deleted at phase 8.
//!
//! # Conventions
//!
//! - **SI throughout** (K, Pa, kg/s, m/s, J/kg).
//! - The cycle runs in **total (stagnation)** quantities `Tt`, `pt`; static conversion happens
//!   only at the nozzle exit, for exhaust velocity.
//! - **Conservation checks are `assert!`**, not `debug_assert!` — they run on every execution,
//!   in every profile. That is the working contract, not a debugging convenience.

pub mod gas;
