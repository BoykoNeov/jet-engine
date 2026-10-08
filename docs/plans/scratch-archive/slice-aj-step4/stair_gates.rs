// ---------------------------------------------------------------------------- the gates

/// Rung 83 § 3.2's jump pair (`r = 0.25`) — `test_p2`'s second half — and its `classify`,
/// forward and REVERSED.
#[test]
fn jump_pair_and_its_classification_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let a = staircase_law::edge_read(&m, JUMP_LO, &kw(&f, 0.25, DS));
    one("edge_jump_lo", Some(&m), &a, edge);
    let b = staircase_law::edge_read(&m, JUMP_HI, &kw(&f, 0.25, DS));
    one("edge_jump_hi", Some(&m), &b, edge);
    one("classify_jump", None, &staircase_law::classify(&a, &b), classified);
    one("classify_rev", None, &staircase_law::classify(&b, &a), classified);
}

/// Rung 83 § 3.4's crossing pair (`r = 0.35`) — `test_p2`'s first half.
#[test]
fn crossing_pair_and_its_classification_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let a = staircase_law::edge_read(&m, CROSS_LO, &kw(&f, 0.35, DS));
    one("edge_cross_lo", Some(&m), &a, edge);
    let b = staircase_law::edge_read(&m, CROSS_HI, &kw(&f, 0.35, DS));
    one("edge_cross_hi", Some(&m), &b, edge);
    one("classify_cross", None, &staircase_law::classify(&a, &b), classified);
}

/// The EMPTY window, and `classify` of it with itself — every Option `None`.
#[test]
fn empty_window_read_and_classification_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let a = staircase_law::edge_read(&m, 0.05, &kw(&f, 1.0, DS));
    one("edge_v1", Some(&m), &a, edge);
    one("classify_v1", None, &staircase_law::classify(&a, &a), classified);
}

/// `test_p4`'s ladder, verbatim.
#[test]
fn p4_ladder_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let s = staircase_law::staircase_scan(&m, 0.0190, 0.0206, 9, &kw(&f, 0.25, DS));
    one("scan_p4", Some(&m), &s, ladder);
}

/// `test_p3`'s count at the shipped step.
#[test]
fn p3_count_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let c = staircase_law::lattice_count(&m, 0.016, 0.024, &kw(&f, 0.25, 0.005));
    one("lattice_p3", Some(&m), &c, lattice);
}

/// `test_p3`'s count at the halved step.
#[test]
fn p3_count_at_the_halved_step_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let c = staircase_law::lattice_count(&m, 0.016, 0.024, &kw(&f, 0.25, 0.0025));
    one("lattice_fine", Some(&m), &c, lattice);
}

/// Python's misnamed `V3: an edge off the march grid`, at `r = 1.0`.
#[test]
fn count_void_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let c = staircase_law::lattice_count(&m, 0.004, 0.05, &kw(&f, 1.0, DS));
    one("lattice_v3", Some(&m), &c, lattice);
}

/// ONE edge move with a spacing — a `lam`.
#[test]
fn staircase_number_with_a_spacing_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, JUMP_LO, JUMP_HI, Some(SPACING), &kw(&f, 0.25, DS));
    one("sn_ok", Some(&m), &n, number);
}

/// No spacing — the FACTORS and no `lam`.
#[test]
fn staircase_number_bare_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, JUMP_LO, JUMP_HI, None, &kw(&f, 0.25, DS));
    one("sn_bare", Some(&m), &n, number);
}

/// `spacing = 0.0` — Python's truthiness: no tread, but the spacing reported.
#[test]
fn staircase_number_zero_spacing_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, JUMP_LO, JUMP_HI, Some(0.0), &kw(&f, 0.25, DS));
    one("sn_zero", Some(&m), &n, number);
}

/// V5 on a degenerate bracket, its message through `%g`.
#[test]
fn staircase_number_v5_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(
        &m, 0.0198123456, 0.0198123456, None, &kw(&f, 0.25, DS));
    one("sn_v5", Some(&m), &n, number);
}

/// V2 — an end of the bracket in the empty window.
#[test]
fn staircase_number_v2_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let n = staircase_law::staircase_number(&m, 0.05, 0.30, None, &kw(&f, 1.0, DS));
    one("sn_v2", Some(&m), &n, number);
}

/// `test_p7`'s coarse call — a JUMP, no root.
#[test]
fn root_class_jump_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, N_BISECT, EPS, &kw(&f, 0.25, 0.005));
    one("root_jump", Some(&m), &r, root);
}

/// A CROSSING at the shipped step (`r = 0.35`).
#[test]
fn root_class_crossing_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, N_BISECT, EPS, &kw(&f, 0.35, 0.005));
    one("root_cross", Some(&m), &r, root);
}

/// The bisection's V1, carried.
#[test]
fn root_class_v1_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, N_BISECT, EPS, &kw(&f, 1.0, DS));
    one("root_v1", Some(&m), &r, root);
}

/// V6 with an `eps` whose `%g` (`0.123457`) is not Rust's `{}`.
#[test]
fn root_class_v6_bit_for_bit() {
    let (m, f) = (rig(), flight());
    let r = staircase_law::root_class(&m, BRACKET, 2, 0.123456789, &kw(&f, 0.35, DS));
    one("root_v6", Some(&m), &r, root);
}
