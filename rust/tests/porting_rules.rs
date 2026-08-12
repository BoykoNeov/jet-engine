//! THE ARITHMETIC-SPELLING RULES the port depends on, kept honest.
//!
//! `gas.rs`'s header states two porting rules — float multiplication is not associative, and
//! never spell an integer power as a product. Both were learned the expensive way, and the
//! second one was learned TWICE: phase 1 believed it had followed it, its own oracle agreed at
//! 100 % on enthalpy, and phase 2 still found `antideriv_h` spelling Python's `T ** 3`..`T **
//! 5` as multiplications. That defect cost the port ~1.1 % of its bit-exactness and was
//! misdiagnosed for a whole phase as a solver stopping-rule artefact.
//!
//! A rule stated in a comment is a rule that drifts. These tests make each one a MEASUREMENT
//! that fails if the compiler, the platform libm, or a future edit quietly changes the answer.
//!
//! WHAT IS AND IS NOT HERE. The value agreement between `powp` and Python's `**` is NOT
//! re-measured here — `gas_oracle.rs` and `cycle_oracle.rs` already carry it end-to-end at
//! 3232/3232 and 676/676 bit-identical, which is a far stronger statement than any synthetic
//! grid. What is here is the half those cannot see: that the CALL Rust emits is a real `pow`
//! and not something the optimiser swapped in. That question has no Python side, so it needs
//! no oracle — and keeping it oracle-free is what makes it a detector rather than a duplicate.

use turbojet::gas::powp;

/// The grid: a geometric sweep across the magnitudes the cycle actually produces — pressure
/// ratios near 1, temperatures in the hundreds, enthalpies in the millions.
fn grid() -> Vec<f64> {
    let mut xs = Vec::new();
    let mut x = 1.0000000001f64;
    for _ in 0..4000 {
        xs.push(x);
        x *= 1.0017;
    }
    for v in [0.5, 2.0, 3.0, 287.0, 1004.0, 1e5, 4.31e5, 2.2e6, 1.4e6, 1e-3, 7.0, 1234.5678] {
        xs.push(v);
    }
    xs
}

/// RULE 1 — `powp(x, 0.5)` must be a REAL `pow` call, not a strength-reduced `sqrt`.
///
/// This is the detector for the trap `todo-rust-port.md` § 4 records as one of three bug hunts
/// in the earlier cffi experiment. Written as the obvious `x.powf(0.5)`, LLVM rewrites the call
/// to `sqrt` — and measurably so: on this grid the two are byte-identical at all 4012 points,
/// while Python's `x ** 0.5` differs from `sqrt` at 6 of them. `powp`'s `black_box` on the
/// exponent is what stops the rewrite.
///
/// The assertion is that `powp` and `sqrt` DISAGREE somewhere. That reads backwards until you
/// notice what a passing "they agree" test would prove: nothing, because the folded version
/// agrees with itself. Disagreement is the only observable that distinguishes a real call from
/// a substituted one.
#[test]
fn powp_is_a_real_pow_and_not_a_folded_sqrt() {
    let xs = grid();
    let mut differs = 0usize;
    let mut folded = 0usize;
    for &x in &xs {
        if powp(x, 0.5).to_bits() != x.sqrt().to_bits() {
            differs += 1;
        }
        // The unguarded spelling, for contrast — this is what the naive port would have used.
        if x.powf(0.5).to_bits() == x.sqrt().to_bits() {
            folded += 1;
        }
    }
    println!("powp(x,0.5) != sqrt(x) at {differs} / {} points", xs.len());
    println!("powf(0.5)   == sqrt(x) at {folded} / {} points  <- the strength reduction", xs.len());
    assert!(differs > 0,
            "powp(x, 0.5) agreed with sqrt at EVERY one of {} points, which means the pow call \
             was folded away. Python's `x ** 0.5` is libm pow and differs from sqrt about 1 \
             point in 670, so this test is the only thing standing between the port and a \
             silent last-bit drift in every velocity in the cycle.", xs.len());
    assert_eq!(folded, xs.len(),
               "the UNGUARDED x.powf(0.5) no longer folds to sqrt on this toolchain. That is \
                good news, but `powp`'s black_box was justified by this measurement — re-check \
                the justification before simplifying it away.");
}

/// RULE 2 — the integer-power rule is SPLIT, and both halves are load-bearing.
///
/// The tempting simplifications are "always multiply" and "always call pow". Phase 2 measured
/// both against the reference interpreter, and BOTH LOSE:
///
/// ```text
///   spelling of the SQUARE in poly/antideriv_h/antideriv_phi   gas oracle vs PyPy
///     t * t                                                     3232 / 3232   <- shipped
///     powp(t, 2.0)                                              3230 / 3232
///   spelling of the CUBE and above
///     product chains                                            3196 / 3232   (phase 1)
///     powp(t, 3.0), powp(t, 4.0), powp(t, 5.0)                   3232 / 3232   <- shipped
/// ```
///
/// The reason is PyPy, not Rust: PyPy's JIT rewrites `x ** 2` into a multiply and does NOT
/// rewrite higher powers, so reproducing PyPy means doing exactly the same. CPython does not
/// (it calls libm `pow` for the square too, and disagrees with itself-as-PyPy at 2 points in
/// 6013) — which is one more instance of the standing finding that CPython is the outlier.
///
/// So the rule is: MULTIPLY THE SQUARE, CALL `pow` ABOVE IT. This test pins the observable
/// each half rests on. The value agreement itself belongs to the oracles; what is checked here
/// is that the three spellings are still genuinely DIFFERENT operations — because the day they
/// stop being different, the reasoning above silently becomes vacuous.
#[test]
fn the_integer_power_rule_is_split_and_both_halves_are_real() {
    let xs = grid();
    let (mut sq_diff, mut cube_diff, mut quart_diff) = (0usize, 0usize, 0usize);
    for &x in &xs {
        if powp(x, 2.0).to_bits() != (x * x).to_bits() {
            sq_diff += 1;
        }
        if powp(x, 3.0).to_bits() != (x * x * x).to_bits() {
            cube_diff += 1;
        }
        if powp(x, 4.0).to_bits() != (x * x * x * x).to_bits() {
            quart_diff += 1;
        }
    }
    let n = xs.len();
    println!("pow(x,2) != x*x        at {sq_diff} / {n}   <- why 'always pow' loses the square");
    println!("pow(x,3) != x*x*x      at {cube_diff} / {n}   <- why 'always multiply' loses the cube");
    println!("pow(x,4) != x*x*x*x    at {quart_diff} / {n}");

    // The square: pow and multiply must still be DISTINGUISHABLE, or the measurement that
    // chose `t * t` over `powp(t, 2.0)` no longer means anything. One point is enough — this
    // is a discrimination check, not a rate.
    assert!(sq_diff > 0,
            "pow(x,2) and x*x agreed at all {n} points, so the gas oracle's 3232-vs-3230 split \
             between the two spellings can no longer be reproduced. Re-measure before trusting \
             either half of the rule.");
    // The cube and quartic: ~25 % and ~36 % of the grid. Assert an order of magnitude below
    // that, so the test measures the EFFECT rather than pinning a platform-specific count.
    assert!(cube_diff > n / 20,
            "x**3 differed from x*x*x at only {cube_diff} of {n} points. Either the platform \
             pow changed or powp folded — either way the premise for spelling cubes as pow \
             calls needs re-measuring, not assuming.");
    assert!(quart_diff > n / 20,
            "x**4 differed from x*x*x*x at only {quart_diff} of {n} points — see above.");
}

/// RULE 3 — float multiplication is not associative, so a transcribed expression must keep
/// Python's grouping.
///
/// `gas.rs`'s header cites `A[2] * T ** 2`, which Python evaluates as `A2 * (T*T)` and which
/// the natural Rust `a[2] * t * t` evaluates as `(A2 * T) * T`. The rule is stated as a
/// warning; here it is a number, so nobody has to take it on faith.
#[test]
fn multiplication_is_not_associative() {
    let mut differs = 0usize;
    let mut worst = 0.0f64;
    // Coefficient magnitudes straight out of the NASA table, times realistic temperatures.
    for &a in &[3.298677, 1.4082404e-3, -3.963222e-6, 5.641515e-9, -2.444854e-12] {
        for &t in &grid() {
            let python_grouping = a * (t * t);
            let naive_rust = a * t * t;
            if python_grouping.to_bits() != naive_rust.to_bits() {
                differs += 1;
                let scale = python_grouping.abs().max(naive_rust.abs());
                if scale > 0.0 {
                    worst = worst.max((python_grouping - naive_rust).abs() / scale);
                }
            }
        }
    }
    println!("a*(t*t) != (a*t)*t at {differs} products, worst relative gap {worst:.2e}");
    assert!(differs > 0,
            "the two groupings agreed everywhere, which would make gas.rs's porting rule 1 \
             vacuous — it is not, so this test has stopped measuring what it claims to.");
}
