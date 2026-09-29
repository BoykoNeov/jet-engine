//! RUNG 83 — **THE CORRECTOR'S OWN BAR.** `CorrectorLawTransient` (`engine.py:22645`), slice AJ.
//!
//! Rung 82 asked whether ONE Newton step off its residual `h` reaches what thirteen bisection
//! marches reach. The answer is no, and the reason is not accuracy. Headline: **a bracketing
//! solve locates a SIGN CHANGE; a corrector needs a ROOT, and on a residual built as a `min` those
//! are different objects** — `h` jumps at every argmin handover, bisection reads only `sign(h)`,
//! and at `r = 0.25` on the shipped grid there is no root at all. Rung 78 found a residual's
//! slope is a gauge and its root's UNIQUENESS is not; this rung finds its root's EXISTENCE is not
//! either.
//!
//! # WHAT STEP 1 ADDS HERE — **THIS FILE, AND NOTHING IN IT YET**
//!
//! Reader-only, as [`authority_clock`](crate::authority_clock) says for all four rungs: no `R83`
//! table, no builder. Every march it runs is rung 82's `_scan`. `corrector_read`,
//! `corrector_step`, `residual_shape` and `corrector_secant` are **step 4**, with rung 84.
//!
//! # WHAT THE PRE-FLIGHT FIXED FOR STEP 4
//!
//! * **`corrector_secant`'s S2 message** (`engine.py:22784`) is built with `%g` →
//!   [`py_g`](crate::demand_coordinate::py_g).
//! * **Every void is a RETURNED string, byte for byte** (§ 5.34 (vi)) — including
//!   `V4: kappa impure` and `V4: kappa impure at a start point`, which at `r = 1.0` MISNAME their
//!   cause: there is no four-loop window, so κ is EMPTY, not impure. The port reproduces
//!   Python's words and step 5 pins them; the misnaming is recorded in the plan, not repaired
//!   here.
