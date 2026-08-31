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
use turbojet::reference_split::{c_div, c_neg, csqrt, py_two, C64};

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

/// RULE 4 — `csqrt` is CPython's GENERAL `c_sqrt`, and it must not have moved the REAL branch it
/// replaced.
///
/// Added at slice AC step 5, and the reason is a defect this file exists to catch. Step 3 shipped
/// `csqrt_gated`, whose body was the real-argument spelling under an `assert!(p.im == 0.0)` —
/// registered in § 5.27 (iv) as a *gated condition* on the strength of `p` being measured
/// positive-real on **18 of 18 calls of the shipped rung-70 READERS**. Rung 71's own damping-reader
/// gate then drove rung 70's `_zeta_pair` on a CONSTRUCTED spectrum where `p = 4462 + 4947i`, and
/// the assertion refused a call Python answers with `1.278` — a number the port had already
/// written into `zeta_ring`'s doc comment at the same step. **The measurement was of the readers;
/// the claim was about the rung, and the shipped test suite is in the second set.**
///
/// So the general algorithm went in, and this rule is what keeps the replacement honest. It has
/// TWO halves and the second is the one an argument would have skipped:
///
/// * on a real argument with `im = +0.0` the general spelling is **bit-for-bit** the real one
///   (`ax/8` is a power-of-two scaling, `hypot(x, 0) = |x|`, and `2*sqrt(ax/4) = sqrt(ax)`
///   exactly), so nothing the port already ships can move;
/// * at `im = -0.0` it deliberately **does not** — CPython's `copysign(d, z.imag)` returns
///   `-0.0` where the real-only spelling returns `+0.0`. That divergence is invisible to every
///   value gate in the slice, so it is ASSERTED here rather than left latent.
///
/// **AND THE SECOND HALF IS THE COMMON CASE, NOT THE CORNER — the first writing of this comment
/// had it backwards.** Intercepting all 96 `cmath.sqrt(p)` calls the two shipped suites make:
/// **90 carry `im == -0.0`**, 5 carry `+0.0`, one `p` is genuinely complex, the `sqrt` differs
/// bit-wise on **91**, and the RETURNED `zeta` differs on **1** — the complex one. The 90 are
/// harmless only because `p.re < 0` on **zero** of them, which keeps `copysign` on the imaginary
/// component's zero instead of flipping the sign of a non-zero one. That count is a measurement,
/// and it is the reason this rule asserts the divergence rather than describing it.
///
/// **AND `p.re < 0` ON ZERO OF 90 IS A PROPERTY OF THIS PLANT's SPECTRA, NOT A THEOREM.** `p` is a
/// product of two cubic roots, and nothing forbids its real part being negative — it simply is not
/// on the trajectories rungs 70 and 71 march today. If the plant's roots move, that count is the
/// one to re-measure, because it is the only thing standing between the signed-zero divergence and
/// a sign flip on a non-zero component.
#[test]
fn csqrt_matches_the_real_branch_it_replaced_and_diverges_only_at_negative_zero() {
    // The real-only spelling `csqrt` replaced, transcribed here so the comparison has two sides.
    fn real_only(d: f64) -> (f64, f64) {
        if d == 0.0 {
            (0.0, 0.0)
        } else if d > 0.0 {
            (d.sqrt(), 0.0)
        } else {
            (0.0, (-d).sqrt())
        }
    }
    let mut n = 0usize;
    for &d in &grid() {
        for sign in [1.0f64, -1.0] {
            let x = sign * d;
            let g = csqrt(C64 { re: x, im: 0.0 });
            let r = real_only(x);
            assert_eq!((g.re.to_bits(), g.im.to_bits()), (r.0.to_bits(), r.1.to_bits()),
                       "csqrt moved the real branch at {x}");
            n += 1;
        }
    }
    println!("csqrt == the real-only spelling, bit for bit, at {n} real arguments");

    // …and the ONE place it is different, on purpose.
    let neg0 = csqrt(C64 { re: 4.0, im: -0.0 });
    assert_eq!(neg0.re, 2.0);
    assert!(neg0.im.is_sign_negative() && neg0.im == 0.0,
            "CPython's copysign carries the sign of a zero imaginary part; a real-only spelling \
             cannot, and that is the whole reason this half is measured");
    assert!(csqrt(C64 { re: 4.0, im: 0.0 }).im.is_sign_positive());

    // THE CALL THAT FALSIFIED THE ASSERTION, at its own value — `cmath.sqrt(4462 + 4947j)`.
    let rt = csqrt(C64 { re: 4462.0, im: 4947.0 });
    assert_eq!(rt.re, 74.578819632228_f64);
    assert_eq!(rt.im, 33.16625299512139_f64);

    // …AND THE HALF THE FIRST WRITING OF THIS RULE LEFT AS AN ARGUMENT.
    //
    // The sweep above drives `im: +0.0`, which the census says **5** of 96 shipped calls carry.
    // The other **90** carry `-0.0`, and for those the claim was *"`c_div` washes the signed zero
    // out of the `.real` the reader takes"* — a Python measurement transferred to Rust by
    // reasoning, invisible to every value gate in the slice. It is measured here instead, on the
    // composition `zeta_pair` actually evaluates: `(-s / (2*rt)).real`.
    //
    // `p.re >= 0` on all 90, so this drives that arm; the `p.re < 0` arm is the one the doc note
    // above says would flip the sign of a NON-zero component, and it is asserted to do so, since
    // a silent agreement there would mean this half had stopped measuring anything.
    for (sr, si, pr) in [(-42.0_f64, 0.0_f64, 1225.0_f64),
                         (-217.0, -25.5, 4462.0),
                         (-3.5, 12.25, 0.75)] {
        let s_ = C64 { re: sr, im: si };
        let pos = c_div(c_neg(s_), py_two(csqrt(C64 { re: pr, im: 0.0 }))).re;
        let neg = c_div(c_neg(s_), py_two(csqrt(C64 { re: pr, im: -0.0 }))).re;
        assert_eq!(pos.to_bits(), neg.to_bits(),
                   "the signed zero reached the returned `.real` at p.re = {pr}");
    }
    // THE NEGATIVE-real arm, where the same `copysign` lands on a NON-zero component: the two
    // signs must NOT agree, or the divergence this rule exists for has quietly disappeared.
    //
    // **AND `s` HAS TO BE COMPLEX FOR THAT, WHICH THE FIRST WRITING OF THIS CONTROL GOT WRONG.**
    // With a purely real `s` the quotient's `.real` comes out `+0.0` under BOTH signs — `a.re`
    // multiplies a zero ratio and `a.im` is a signed zero, so the result is zero either way and
    // the flip cannot show. The control failed, and it was the CONTROL that was wrong: the
    // divergence needs a negative `p.re` **and** an `s` with a non-zero imaginary part. Measured
    // here, the two differ in SIGN and not merely in bits (±0.19087).
    let s_ = C64 { re: -217.0, im: -25.5 };
    let pos = c_div(c_neg(s_), py_two(csqrt(C64 { re: -4462.0, im: 0.0 }))).re;
    let neg = c_div(c_neg(s_), py_two(csqrt(C64 { re: -4462.0, im: -0.0 }))).re;
    assert_ne!(pos.to_bits(), neg.to_bits(),
               "at p.re < 0 with a complex s, the sign of the zero moves a NON-zero component —                 if these agree, this rule has stopped measuring the thing it was written for");
    assert!(pos > 0.0 && neg < 0.0, "and the difference is a SIGN: {pos} vs {neg}");
}
