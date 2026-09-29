//! RUNG 81 — **THE AUTHORITY CLOCK.** `AuthorityClockTransient` (`engine.py:21795`), slice AJ.
//!
//! Rungs 72–80 read `n_live <= 3` six times off ONE side of a switch: under `min` exactly one
//! fuel-side leg reaches the actuator, and in every cell measured it was the `Tt4` governor. This
//! rung throws the switch. Headline: **authority is decided by the LAG, not by the set point** —
//! in `demand` each leg's state sits below its cap by the ramp-tracking error `tau·dc/ds`, so
//! `min` hands the actuator to the SLOWER leg, which need not be the one demanding the deeper
//! cut; and **a leg that never holds the actuator has no clock** (a 10× sweep of a masked leg's
//! `tau` moves the march by nothing, bit for bit).
//!
//! # WHAT STEP 1 ADDS HERE — **THIS FILE, AND NOTHING IN IT YET**
//!
//! Plan § 5.34 (ii) measured rungs 81–84 as **reader-only**: 24 methods over the four classes,
//! every one single-definer in the MRO, and no class after rung 80 defines `at_lever`,
//! `_shared_rig`, `_cap_fuel`, `_with_coord` or any other cell. So there is **no `R81` table** and
//! no builder: a Python `AuthorityClockTransient` is a rung-80 core — built by
//! [`build_split_wall_cascade`](crate::split_wall::build_split_wall_cascade) — plus this module's
//! free functions. A `build_authority_clock_cascade` would name a machine that does not exist,
//! and the only check it could invite (*is it `R81`'s?*) is vacuous by construction.
//!
//! The readers — `_central`, `_criterion_at`, `authority_clock`, `_tau_f_inert`,
//! `authority_mask` — are **step 2**.
//!
//! # WHAT THE PRE-FLIGHT FIXED FOR STEP 2
//!
//! * **`id(p)` → indices.** `authority_clock` maps `_riding4`'s points back to trajectory
//!   positions through `id(p)` (`engine.py:21941`, `:21944`), because `_criterion_at` differences
//!   `traj[i-1]` and `traj[i+1]`. The crate's [`riding4`](crate::shared_actuator::riding4)
//!   returns copies, so step 1 adds
//!   [`riding4_idx`](crate::shared_actuator::riding4_idx), whose doc says why the two sets are
//!   equal without a no-aliasing premise.
//! * **`authority_mask` dispatches ON THE RIG.** Its five calls (`engine.py:22096–22103` —
//!   `_with_share`, `_quad_gains_at`, `_jac4`, `_charpoly4`, `_quartic_roots_c`) are made on the
//!   machine `_split_march` RETURNS, not on `self`. With no swaps the two carry the same pointers,
//!   so no value gate can tell a port that reads the caller's table from one that reads the
//!   rig's; step 7 owes the injection that does (P4).
//! * **One float sum** — `rate = sum(1.0 / t for t in tt)` (`engine.py:22104`), four terms, used
//!   only as `zeros`' bar. Ported as the four precedents at rungs 72/73/75/76 port it. The
//!   pre-registered CPython exemption is `_charpoly4`'s `c0`/`c1`, not this sum (§ 5.34 (v)).
//! * **No `%g`, no carriers, no `assert`, no class state** (§ 5.34 (iv)).
