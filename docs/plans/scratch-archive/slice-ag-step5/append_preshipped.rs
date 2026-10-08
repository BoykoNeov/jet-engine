
// ---------------------------------------------------------------------------------------------
// STEP 5 — THE READERS: § 1's AUTHORITATIVE DIAGONAL, § 2's BILL, § 3's GAIN
// ---------------------------------------------------------------------------------------------
//
// Four readers, and every one of them CONSUMES step 4's four methods rather than re-deriving
// anything: `cap_rows` and `solve_gain` each open with a [`cap_march`], all three public readers
// open with an [`accel_for`], and `cap_rows`/`solve_gain` both close over [`c_at`]. So this step
// re-DRIVES step 4 rather than re-proving it, which is what § 5.31.4 (j) booked here.
//
// THE ONE CALL-SITE DIFFERENCE FROM RUNG 75's READERS IS THE ARMED SCHEDULE, AND IT IS THE RUNG.
// [`windup_rows`](crate::anti_windup::windup_rows) passes `accel = None` into `rhs_gains_at`;
// `cap_rows` passes the real one (`engine.py:19353`). Copying the parent's call site there is the
// single highest-value defect this step can ship, because the resulting reader still returns
// well-formed Jacobians at every point — the accel branch of `_cap_fuel` simply never runs, so
// `sensed` and `solve` agree and § 1's whole headline reads as REFUTED.

/// One row of [`cap_rows`] — the SAME state read under BOTH cap laws.
///
/// Every field ending `0` is the `"solve"` arm's reading at the identical state, which is what
/// makes each claim in § 1 a DIFFERENCE between two laws rather than a property of a trajectory —
/// [`WindupRow`](crate::anti_windup::WindupRow)'s discipline, one knob over.
///
/// # IT CARRIES THREE CAPS WHERE THE PARENT CARRIED NONE, AND THAT IS THE MIN-SELECT GUARD
///
/// `_cap_fuel` is `min(accel, phi)` — min-select ONE LEVEL DOWN — and the phi leg has no sensed
/// form. So wherever the phi cap is the lower one the knob is INERT, and an aggregate that pooled
/// those points would report a real law as broken. Worse, the two laws being differenced sit on
/// OPPOSITE SIDES of that `min` near a crossover, so a point can be accel-bound under `solve` and
/// phi-bound under `sensed`; differencing it measures a LEG CHANGE and reports it as a law that
/// broke. [`accel_binds`](CapRow::accel_binds) is the filter that answers both, and it is rung 72's
/// `switch_guard` one min-select level down.
#[derive(Clone, Copy, Debug)]
pub struct CapRow {
    pub s: f64,
    pub auth: Authority,
    pub masked: Authority,
    /// `d(cap_sensed)/dw` at this point, from [`c_at`] at the APPLIED demand `mf_app`.
    pub c: f64,
    /// `max(cap_a, cap_a2) < cap_s` — the accel leg is the lower cap under **BOTH** laws.
    ///
    /// Python's `max` here has TWO EXPRESSIONS and no literal, which is a third category beside
    /// step 1's literal-first census and [`c_at`]'s single expression-first fold. Both arguments
    /// can be `f64::INFINITY` (`_cap_fuel` returns the empty `min`'s identity when neither leg is
    /// armed), so the fold is spelled out rather than taken as `f64::max`: Python replaces only on
    /// a strict `>` and keeps argument 0 on a tie, and `f64::max` additionally discards a NaN that
    /// Python would propagate.
    pub accel_binds: bool,
    /// The accel cap under `solve` — Python's `cap_a`. `cap_a2`, the sensed one, is NOT recorded:
    /// it exists only inside [`accel_binds`](CapRow::accel_binds), exactly as in the source.
    pub cap_accel: f64,
    /// The PHI cap — `_cap_fuel(…, accel = None, surge)`, read under whatever law the receiver
    /// carries, because the source's third call is a BARE one with no scope around it.
    pub cap_phi: f64,
    pub tau_auth: f64,
    pub tau_masked: f64,
    pub auth_diag: f64,
    pub auth_diag0: f64,
    pub masked_diag: f64,
    pub masked_diag0: f64,
    pub row_auth: f64,
    pub row_auth0: f64,
    pub mask_leak: f64,
    pub mask_leak0: f64,
    pub det: f64,
    pub det0: f64,
    /// **P9** — the GOVERNOR's row cannot move in any cell, because `_cap_gov` has no branch.
    /// `max |J[1][j] - J0[1][j]|` over the four columns.
    pub gov_row: f64,
    pub zeros: usize,
    pub zeros0: usize,
}

/// Python's `max(seq)` — replace only on a strict `>`, keep argument 0 on a tie, propagate a NaN
/// that arrived FIRST.
///
/// `f64::max` does none of those three the same way, and this rung is the first in the slice where
/// a NaN is reachable in an aggregate at all ([`SolveGainRow::gain`] is `nan` when `dS == 0`). The
/// crate's older aggregates spell `fold(f64::NEG_INFINITY, f64::max)`, which is bit-identical on
/// every NaN-free population and silently different on this one; step 5's folds are all written
/// this way instead, and the difference is named rather than inherited.
fn py_max<T, F: Fn(&T) -> f64>(xs: &[T], f: F) -> f64 {
    xs.iter().map(|x| f(x)).reduce(|a, b| if b > a { b } else { a })
      .expect("Python's `max()` raises on an empty sequence; every caller filters first")
}

/// Python's `min(seq)` — [`py_max`]'s mirror.
fn py_min<T, F: Fn(&T) -> f64>(xs: &[T], f: F) -> f64 {
    xs.iter().map(|x| f(x)).reduce(|a, b| if b < a { b } else { a })
      .expect("Python's `min()` raises on an empty sequence; every caller filters first")
}

/// RUNG 76's `_cap_rows` — **`sensed` against `solve` AT THE SAME STATES, both through
/// [`rhs_laws`](crate::anti_windup::rhs_laws).**
///
/// # THE STATES ARE THE CLIP PLANT's AT THE INHERITED FLOOR, AND THEY ARE READ UNDER `solve`
///
/// Rung 74 § 1.3 / rung 75 § 1.2's disclosure inherited word for word — and here it acquires a
/// second, harder reason: `clip × sensed` is REFUSED ([`r76_integrate_fuel`]'s second assert), so
/// the base march CANNOT carry this rung's law. The difference measured is therefore PURELY the
/// law at ONE state, which is what a Jacobian comparison has to be. The march runs
/// `clip × none × solve`; both devices are then applied IN THE READER.
///
/// # `_lag_coord` IS FLIPPED BY A PLAIN ASSIGNMENT, AND IT IS THE READER's ADMISSION TICKET
///
/// `engine.py:19341` writes `m._lag_coord = "demand"` after the march and before the filter, by
/// plain attribute assignment — a direct `set`, no scope and no hook, for
/// [`windup_rows`](crate::anti_windup::windup_rows)'s reason. § 5.31.3 (a) measured what dropping
/// the identical line one rung down costs: not a value break but a PANIC, because rung 75's
/// `windup_tau` refuses `track` outside the plain demand coordinate. That refusal is live HERE too
/// — this reader's `laws` grid arms `track` — so the line is load-bearing at this rung for the rung
/// below's reason, which is the shape § 5.31.4 (a) named from the other side.
///
/// # THE FOUR DIFFERENCE STEPS ARE `_rhs_gains_at`'s **DEFAULTS**, NOT THIS READER's CHOICE
///
/// `engine.py:18789-18790` declares `dg = 1e-7, dq = 1e-5, dv = 1e-4, switch_guard = 4.0` and both
/// callers take them; Rust has no defaults, so they are spelled at the call site. They coincide
/// with the numbers [`windup_rows`](crate::anti_windup::windup_rows) writes — which is a fact about
/// the DEFAULTS and not a value copied from the sibling, and the distinction is written down
/// because the next reader to add a call site can only get it right from the signature.
///
/// # AND THE `accel` ARGUMENT IS THE ONE THAT MAKES THIS READER RUNG 76 AT ALL
///
/// See this section's header: `Some(accel)`, where the parent passes `None`.
#[allow(clippy::too_many_arguments)]
pub fn cap_rows(
    core: &ScheduledStatorCore, flight: &FlightCondition, sm: f64, ref_law: &'static str,
    law: &'static str, tau_t: Option<f64>, taus: (f64, f64, f64, f64), inc: bool, tt4_lo: f64,
    tt4_hi: f64, tt4_max: f64, r: f64, s_settle: f64, ds: f64, v_max: f64,
    accel: &AccelSchedule, every: usize,
) -> (Vec<CapRow>, usize) {
    // THE BASE MARCH CARRIES NEITHER DEVICE — `clip` × `none` × `solve`.
    let (m, surge, lag, traj) = cap_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
        LAG_COORD_CLIP, ref_law, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE, accel, None);
    // `m._lag_coord = "demand"` — PLAIN, see the header.
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let lag = lag.expect("`_shared_rig` arms the fuel leg, so it carries the lag");
    let mut rows: Vec<CapRow> = Vec::new();
    for p in pts.iter().step_by(every) {
        let (required_fuel, g_fuel, g_gov, b, v, wf_opt, wr_opt) = match p.extra {
            PointExtra::Demand { required_fuel, g_fuel, g_gov, b, v, w_fuel, w_gov, .. } =>
                (required_fuel, g_fuel, g_gov, b, v, Some(w_fuel), Some(w_gov)),
            PointExtra::Shared { required_fuel, g_fuel, g_gov, b, v, .. } =>
                (required_fuel, g_fuel, g_gov, b, v, None, None),
            _ => panic!("rung-76's rows read `required_fuel`/`g_fuel`/`b`/`v` off every filtered \
                         point, and project `w_fuel`/`w_gov` when the point does not carry them."),
        };
        let tau_f = lag.tau(required_fuel, g_fuel);
        let mf_sched = p.mf_sched;
        let wf = wf_opt.unwrap_or(mf_sched - g_fuel);
        let wr = wr_opt.unwrap_or(mf_sched - g_gov);
        // Python's `(wf, wr, mf_sched)` against this crate's `(mf_sched, wf, wr)` — a plain
        // minimum of all three, so the permutation is unobservable. Named for
        // [`rhs_gains_at`](crate::anti_windup::rhs_gains_at)'s reason: a gate here would score
        // SURVIVED structurally and read as coverage.
        let ma = applied_demand(mf_sched, wf, wr);
        // `_with_windup(law, tau_t, m._with_cap, cl, m._rhs_gains_at, …)` — the windup scope
        // OUTSIDE, the cap scope INSIDE. Rust drops in reverse declaration order, which is
        // Python's inner-`finally`-first unwind exactly.
        let read = |cl: &'static str| {
            let _ws = WindupScope::set(&m.fuel.inner, law, tau_t);
            let _cs = CapScope::set(&m.fuel.inner, cl);
            rhs_gains_at(&m, flight, p, Some(accel), surge.as_ref(), tt4_max, tau_f, taus.1,
                         1e-7, 1e-5, 1e-4, 4.0).unwrap_or_else(|e| panic!("{}", e.0))
        };
        let g = read(CAP_LAW_SENSED);
        if !g.interior || g.masked.is_none() {
            continue;
        }
        let g0 = read(CAP_LAW_SOLVE);
        if !g0.interior {
            continue;
        }
        let c = c_at(&m, flight, p.nu_lp, p.nu_hp, accel, ma, b, v, C_AT_REL)
            .unwrap_or_else(|e| panic!("{}", e.0));
        // THE THREE CAPS, all inside BOTH state guards — and they differ in ALL THREE of `accel`,
        // `surge` and `mf_app`, so unlike `applied_demand` above a transposition here IS
        // observable. Python's `finally` restores both states to `None`, which is what
        // [`MarchedBleed`]/[`MarchedStator`] do on drop.
        let (cap_a, cap_a2, cap_s) = {
            let _sb = MarchedBleed::set(&m.fuel.inner, b);
            let _sv = MarchedStator::set(&m.fuel.inner, v);
            let cf = |accel: Option<&AccelSchedule>, surge: Option<&Floor>, mf_app: Option<f64>| {
                (m.fuel.inner.triple_hooks.cap_fuel)(
                    &m.fuel, flight, p.nu_lp, p.nu_hp, mf_sched, accel, surge, mf_app)
                    .unwrap_or_else(|e| panic!("{}", e.0))
            };
            let a1 = { let _cs = CapScope::set(&m.fuel.inner, CAP_LAW_SOLVE);
                       cf(Some(accel), None, None) };
            let a2 = { let _cs = CapScope::set(&m.fuel.inner, CAP_LAW_SENSED);
                       cf(Some(accel), None, Some(ma)) };
            // THE THIRD IS A **BARE** CALL — no scope. It reads whatever law the receiver carries,
            // which the march above left at `"solve"`; `accel = None` means the cell is never
            // reached anyway, so the law is unobservable here and the source's shape is kept.
            let s3 = cf(None, surge.as_ref(), None);
            (a1, a2, s3)
        };
        let rate = 1.0 / tau_f + 1.0 / taus.1 + 1.0 / taus.2 + 1.0 / taus.3;
        let jm = g.j.expect("an interior reading carries J");
        let jm0 = g0.j.expect("an interior reading carries J");
        let ch = charpoly4(&jm);
        let ch0 = charpoly4(&jm0);
        let rt = quartic_roots_c(&ch);
        let rt0 = quartic_roots_c(&ch0);
        let auth = g.authority.expect("an interior reading carries an authority");
        let masked = g.masked.expect("filtered above");
        rows.push(CapRow {
            s: p.s,
            auth,
            masked,
            c,
            // Python's two-expression `max`, spelled as the fold — see [`CapRow::accel_binds`].
            accel_binds: (if cap_a2 > cap_a { cap_a2 } else { cap_a }) < cap_s,
            cap_accel: cap_a,
            cap_phi: cap_s,
            tau_auth: if auth == Authority::Fuel { tau_f } else { taus.1 },
            tau_masked: if masked == Authority::Fuel { tau_f } else { taus.1 },
            auth_diag: g.auth_diag.expect("a masked leg implies an authoritative one"),
            auth_diag0: g0.auth_diag.expect("the same state, the same mask"),
            masked_diag: g.masked_diag.expect("a masked leg has a diagonal"),
            masked_diag0: g0.masked_diag.expect("the same state, the same mask"),
            row_auth: g.masked_row_auth.expect("both indices exist here"),
            row_auth0: g0.masked_row_auth.expect("both indices exist here"),
            mask_leak: g.mask_leak.expect("both indices exist here"),
            mask_leak0: g0.mask_leak.expect("both indices exist here"),
            det: ch[4],
            det0: ch0[4],
            gov_row: (0..4).map(|j| (jm[1][j] - jm0[1][j]).abs())
                           .reduce(|x, y| if y > x { y } else { x })
                           .expect("four columns"),
            zeros: rt.iter().filter(|z| z.abs() < 1e-4 * rate).count(),
            zeros0: rt0.iter().filter(|z| z.abs() < 1e-4 * rate).count(),
        });
    }
    (rows, pts.len())
}

/// One cell of [`cap_gains`] — Python's `cells[f"{ref}|{law}|{auth}"]`, a **SIX-key empty dict OR a
/// twenty-six-key reading**.
///
/// An enum for [`WindupCell`](crate::anti_windup::WindupCell)'s reason, and the empty arm is WIDER
/// than the parent's: rung 75's carries `n` and `n_riding` alone, where this one adds `ref`, `law`,
/// `auth` **and `n_inert`** — the count of points at which the PHI cap was the lower one, so the
/// knob was inert by construction. A cell that reports zero live rows is not the same fact as a
/// cell that had none to begin with, and the source keeps the two distinguishable even where it
/// keeps nothing else.
#[derive(Clone, Debug)]
pub enum CapCell {
    /// `dict(n=0, n_riding=n, ref=…, law=…, auth=…, n_inert=…)`.
    Empty { n_riding: usize, ref_law: &'static str, law: &'static str, auth: Authority,
            n_inert: usize },
    Read(Box<CapCellRead>),
}

/// The twenty-six keys a populated [`CapCell`] carries.
#[derive(Clone, Debug)]
pub struct CapCellRead {
    pub n: usize,
    pub n_riding: usize,
    pub ref_law: &'static str,
    pub law: &'static str,
    pub auth: Authority,
    /// The points where the OTHER cap was the lower one. **REPORTED, NEVER POOLED IN.**
    pub n_inert: usize,
    pub c: (f64, f64),
    /// **P2/P4** — the AUTHORITATIVE diagonal, which nothing in rungs 73/74/75 could move.
    pub auth_diag: (f64, f64),
    pub auth_diag0: (f64, f64),
    pub auth_moved: f64,
    /// `max |auth_diag - (c-1)/tau_auth| * tau_auth` — the LAW, scored per point against THAT
    /// point's own `c`. Pooling a single `c` across the cell would be rung 73 § 4's failure.
    pub auth_err: f64,
    pub masked_diag: (f64, f64),
    pub masked_moved: f64,
    /// **P6** — the masked ROW's coupling to the leg that holds.
    pub row_auth: (f64, f64),
    pub row_auth0: (f64, f64),
    /// `None` on the FUEL-authoritative cell: the target is only stated where the governor holds.
    pub row_err: Option<f64>,
    /// **P5** — the masked COLUMN, untouched: `n_live <= 3` a FIFTH time.
    pub mask_leak: f64,
    pub mask_leak0: f64,
    /// **P7** — `det J` scales by `1 - c`, per point.
    pub det: (f64, f64),
    pub det0: (f64, f64),
    pub det_ratio: Option<(f64, f64)>,
    pub det_err: Option<f64>,
    /// **P8** — the spectrum's count.
    pub zeros: (usize, usize),
    pub zeros0: (usize, usize),
    pub zeros_moved: usize,
    /// **P9** — the governor's row is bit-identical.
    pub gov_row: f64,
}

/// RUNG 76's `cap_gains` return.
#[derive(Clone, Debug)]
pub struct CapGains {
    pub phi_lim: f64,
    pub margin: f64,
    pub taus: (f64, f64, f64, f64),
    pub tau_t: f64,
    pub inc: bool,
    pub ds: f64,
    /// Python's `cells`, keyed `f"{ref}|{law}|{auth}"`. **The key is built HERE, not by the
    /// oracle**, because all three components are strings —
    /// [`WindupGains::cells`](crate::anti_windup::WindupGains::cells) carries a tuple only because
    /// one of ITS two components is an `f64` with no shortest-repr spelling in Rust. Inheriting
    /// that workaround would be a mechanism with no reason.
    pub cells: Vec<(String, CapCell)>,
}

/// RUNG 76 § 1 — **a device in a leg's LAW reaches only the MASKED leg; a device in the PLANT THE
/// LEGS READ reaches only the AUTHORITATIVE one.**
///
/// Rung 75's back-calculation is a term in the masked leg's own law, and min-select masks a LAW —
/// so it wrote `-1/tau_t` on the MASKED diagonal and left the authoritative one *moved 0.0
/// relative*, as rungs 73 and 74 each also report. A sensed cap is in NO leg's law: it is in the
/// plant BOTH legs read through `mf_app`, and min-select cannot mask a PLANT. So it writes
/// `c/tau_f` on the AUTHORITATIVE fuel diagonal — the one entry this whole family has measured at
/// exactly zero, four rungs running.
///
/// # THE TWO AUTHORITY CELLS CARRY DIFFERENT HALVES OF IT AND ARE NEVER POOLED
///
/// * **FUEL authoritative** — the fuel row IS the authoritative row: `d(cap)/dw_f = c`, diagonal
///   `(c-1)/tau_f`, and `det J` scales by `1 - c`. The masked GOVERNOR's cap has no sensed form, so
///   its row does not move at all ([`gov_row`](CapCellRead::gov_row), P9).
/// * **GOV authoritative** — the fuel leg is MASKED and reads the governor through `mf_app`: its
///   diagonal is UNMOVED (`min` is flat in what the masked leg holds) and its CROSS moves. `det J`
///   is then unmoved EXACTLY.
///
/// That split is rung 73 § 4's pooling failure and rung 75 § 1.4's, inherited rather than
/// rediscovered — and it is why [`row_err`](CapCellRead::row_err) exists only on the `gov` cell.
///
/// # `c` IS PER POINT, AND EVERY RATIO IS SCORED AGAINST THAT POINT's OWN `c`
///
/// [`c_at`] is called once per row and the errors difference against it there. A cell-wide `c`
/// would be a fitted constant on a rung whose whole claim is that the number is MEASURED.
#[allow(clippy::too_many_arguments)]
pub fn cap_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), tau_t: f64, refs: &[&'static str],
    laws: &[&'static str], inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64, every: usize,
) -> CapGains {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let accel = accel_for(core, flight, tt4_lo, tt4_hi, sm, tt4_max, taus, v_max, inc, margin);
    let mut cells: Vec<(String, CapCell)> = Vec::new();
    for ref_law in refs {
        for law in laws {
            let (rows, n) = cap_rows(
                core, flight, sm, ref_law, law,
                if *law == crate::anti_windup::WINDUP_LAW_TRACK { Some(tau_t) } else { None },
                taus, inc, tt4_lo, tt4_hi, tt4_max, r, s_settle, ds, v_max, &accel, every);
            for auth in [Authority::Fuel, Authority::Gov] {
                let n_inert = rows.iter().filter(|x| x.auth == auth && !x.accel_binds).count();
                let rr: Vec<CapRow> =
                    rows.iter().filter(|x| x.auth == auth && x.accel_binds).copied().collect();
                let key = format!("{}|{}|{}", ref_law, law, auth.as_str());
                if rr.is_empty() {
                    cells.push((key, CapCell::Empty {
                        n_riding: n, ref_law, law, auth, n_inert }));
                    continue;
                }
                // Python's `det_rat` — the PAIR, so every residual is scored against the `c` of
                // the point its ratio came from.
                let det_rat: Vec<(f64, f64)> = rr.iter().filter(|x| x.det0.abs() > 1e-30)
                                                 .map(|x| (x.det / x.det0, x.c)).collect();
                let sched = *ref_law == REF_SCHED;
                cells.push((key, CapCell::Read(Box::new(CapCellRead {
                    n: rr.len(),
                    n_riding: n,
                    ref_law,
                    law,
                    auth,
                    n_inert,
                    c: (py_min(&rr, |x| x.c), py_max(&rr, |x| x.c)),
                    auth_diag: (py_min(&rr, |x| x.auth_diag), py_max(&rr, |x| x.auth_diag)),
                    auth_diag0: (py_min(&rr, |x| x.auth_diag0), py_max(&rr, |x| x.auth_diag0)),
                    auth_moved: py_max(&rr, |x| (x.auth_diag - x.auth_diag0).abs()
                                                / 1e-30f64.max(x.auth_diag0.abs())),
                    auth_err: py_max(&rr, |x| (x.auth_diag - (x.c - 1.0) / x.tau_auth).abs()
                                              * x.tau_auth),
                    masked_diag: (py_min(&rr, |x| x.masked_diag), py_max(&rr, |x| x.masked_diag)),
                    // Python's conditional expression INSIDE the generator: the RELATIVE move
                    // where the base is nonzero, the ABSOLUTE one where it is not. A single
                    // `/max(1e-30, .)` would agree numerically on every live point and lose the
                    // source's shape at the one it was written for.
                    masked_moved: py_max(&rr, |x| if x.masked_diag0.abs() > 1e-30 {
                        (x.masked_diag - x.masked_diag0).abs()
                            / 1e-30f64.max(x.masked_diag0.abs())
                    } else {
                        x.masked_diag.abs()
                    }),
                    row_auth: (py_min(&rr, |x| x.row_auth), py_max(&rr, |x| x.row_auth)),
                    row_auth0: (py_min(&rr, |x| x.row_auth0), py_max(&rr, |x| x.row_auth0)),
                    row_err: if auth == Authority::Gov {
                        Some(py_max(&rr, |x| {
                            let tgt = if sched { x.c } else { x.c - 1.0 };
                            (x.row_auth - tgt / x.tau_masked).abs() * x.tau_masked
                        }))
                    } else {
                        None
                    },
                    mask_leak: py_max(&rr, |x| x.mask_leak.abs()),
                    mask_leak0: py_max(&rr, |x| x.mask_leak0.abs()),
                    det: (py_min(&rr, |x| x.det), py_max(&rr, |x| x.det)),
                    det0: (py_min(&rr, |x| x.det0), py_max(&rr, |x| x.det0)),
                    det_ratio: if det_rat.is_empty() { None } else {
                        Some((py_min(&det_rat, |x| x.0), py_max(&det_rat, |x| x.0)))
                    },
                    det_err: if det_rat.is_empty() { None } else {
                        Some(py_max(&det_rat, |x| (x.0 - (1.0 - x.1)).abs()))
                    },
                    zeros: (rr.iter().map(|x| x.zeros).min().expect("non-empty"),
                            rr.iter().map(|x| x.zeros).max().expect("non-empty")),
                    zeros0: (rr.iter().map(|x| x.zeros0).min().expect("non-empty"),
                             rr.iter().map(|x| x.zeros0).max().expect("non-empty")),
                    zeros_moved: rr.iter()
                                   .map(|x| (x.zeros as i64 - x.zeros0 as i64).unsigned_abs()
                                            as usize)
                                   .max().expect("non-empty"),
                    gov_row: py_max(&rr, |x| x.gov_row),
                }))));
            }
        }
    }
    CapGains { phi_lim, margin, taus, tau_t, inc, ds, cells }
}

// ---------------------------------------------------------------------------------------------
// § 2 — `cap_bill`: THE PATH MOVES AND THE DESTINATION DOES NOT
// ---------------------------------------------------------------------------------------------

/// RUNG 76's `cap_bill` return.
#[derive(Clone, Debug)]
pub struct CapBill {
    pub phi_lim: f64,
    pub margin: f64,
    pub ref_law: &'static str,
    pub law: &'static str,
    pub inc: bool,
    pub ds: f64,
    pub n: usize,
    pub s_tail: f64,
    /// **P11** — the sign of the BILL, `(solve, sensed)` in Python's tuple order.
    pub max_tt4: (f64, f64),
    pub min_phi: (f64, f64),
    /// `sum(mf) * ds` on each arm — a LEFT-TO-RIGHT fold seeded at `0.0`, which is Python's `sum`
    /// exactly. A pairwise or SIMD reduction would be a different float, and § 5.31 (vii)'s **P2**
    /// names precisely these two keys as the CPython arm's only exemption in 83 273: they are the
    /// two `sum()` calls in either class that add a 341-long trajectory rather than a literal `1`.
    /// Changing the summation order here would make that prediction untestable at step 6.
    pub fuel_int: (f64, f64),
    /// **P10** — the destination is the SAME, the path is not. `None` when the window is empty.
    pub wf_tail: Option<f64>,
    pub wf_ramp: Option<f64>,
    pub cuts_harder: bool,
    pub traj_solve: Vec<FuelPoint>,
    pub traj_sensed: Vec<FuelPoint>,
}

/// RUNG 76 § 2 — **the knob is a PURE TRANSIENT DEVICE on the leg that holds.**
///
/// Setting `dw_f/ds = 0` with the fuel leg authoritative (`mf_app = w_f`) gives
/// `w_f* = cap_sensed(w_f*) = cap_solve` EXACTLY, because `cap_solve` is BY CONSTRUCTION the fixed
/// point of `cap_sensed` — and under `applied` identically, since `_demand_reference` returns `cap`
/// itself when `mf_app == w_own`. So the two laws have the SAME equilibrium and different paths:
/// the marching lag performs the set-point solve's fixed-point iteration IN TIME rather than at a
/// POINT, at contraction ratio `c` per lag time constant.
///
/// The BILL's sign is the claim and the magnitude is reported with its `ds` band: during the ramp
/// `mf_app < cap_solve`, so `cap_sensed < cap_solve` (the droop identity, D1) and the sensed leg
/// cuts HARDER.
///
/// # THE GRID-EQUALITY ASSERT IS THE ONLY MESSAGE IN EITHER CLASS THAT DOES NOT NAME ITS RUNG
///
/// `engine.py:19523` — *"the two cap laws marched different grids"*. Every other assert in
/// `AntiWindupTransient` and `SensedCapTransient` opens `rung-75:` or `rung-76:`; this one opens
/// with nothing. § 5.31 (v) counted them (4 of 4 tagged at rung 75, **4 of 5** here) and § 5.31
/// (vii)'s **P6** names this message as the candidate for a port defect that every ported gate
/// passes, because a suite needle checking for a `rung-76:` prefix cannot see it. **So it is ported
/// VERBATIM and UNTAGGED.** Adding a prefix would be an improvement that silently settles P6 in the
/// port's favour before step 7 can test it.
///
/// # THE TAIL WINDOW IS `r + tail * max(taus)`, AND `max` OVER A TUPLE IS PYTHON's FOLD
///
/// [`py_max`]'s rule, on four constants rather than on a row field.
#[allow(clippy::too_many_arguments)]
pub fn cap_bill(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), tau_t: f64, ref_law: &'static str,
    law: &'static str, inc: bool, r: f64, s_settle: f64, ds: f64, v_max: f64, tail: f64,
) -> CapBill {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let accel = accel_for(core, flight, tt4_lo, tt4_hi, sm, tt4_max, taus, v_max, inc, margin);
    let mut out: Vec<Vec<FuelPoint>> = Vec::with_capacity(2);
    for cl in [CAP_LAW_SOLVE, CAP_LAW_SENSED] {
        out.push(cap_march(
            core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
            LAG_COORD_DEMAND, ref_law, law,
            if law == crate::anti_windup::WINDUP_LAW_TRACK { Some(tau_t) } else { None },
            cl, &accel, None).3);
    }
    let b = out.pop().expect("two arms");
    let a = out.pop().expect("two arms");
    // VERBATIM AND UNTAGGED — see this function's doc.
    assert!(a.len() == b.len(), "the two cap laws marched different grids");
    let s_tail = r + tail * [taus.0, taus.1, taus.2, taus.3].into_iter()
                                .reduce(|x, y| if y > x { y } else { x }).expect("four clocks");
    let ramp: Vec<usize> = a.iter().enumerate().filter(|(_, p)| p.s <= r).map(|(i, _)| i).collect();
    let tl: Vec<usize> = a.iter().enumerate().filter(|(_, p)| p.s >= s_tail).map(|(i, _)| i)
                          .collect();
    let w_fuel_of = |p: &FuelPoint| match p.extra {
        PointExtra::Demand { w_fuel, .. } => w_fuel,
        _ => panic!("rung-76's bill reads `w_fuel` off every marched point of a DEMAND march."),
    };
    let wf: Vec<f64> = (0..a.len())
        .map(|i| (w_fuel_of(&b[i]) - w_fuel_of(&a[i])).abs() / 1e-30f64.max(w_fuel_of(&a[i]).abs()))
        .collect();
    // Python's `sum(key(t, "mf")) * ds` — left to right from `0.0`. See [`CapBill::fuel_int`].
    let fold_mf = |t: &[FuelPoint]| t.iter().fold(0.0f64, |acc, p| acc + p.mf) * ds;
    CapBill {
        phi_lim, margin, ref_law, law, inc, ds,
        n: a.len(),
        s_tail,
        max_tt4: (py_max(&a, |p| p.tt4), py_max(&b, |p| p.tt4)),
        min_phi: (py_min(&a, |p| p.phi_lp), py_min(&b, |p| p.phi_lp)),
        fuel_int: (fold_mf(&a), fold_mf(&b)),
        wf_tail: if tl.is_empty() { None } else { Some(py_max(&tl, |i| wf[*i])) },
        wf_ramp: if ramp.is_empty() { None } else { Some(py_max(&ramp, |i| wf[*i])) },
        cuts_harder: ramp.iter().all(|&i| b[i].mf <= a[i].mf + 1e-15),
        traj_solve: a,
        traj_sensed: b,
    }
}

// ---------------------------------------------------------------------------------------------
// § 3 — `solve_gain`: WHAT THE SOLVE WAS BUYING — a GAIN, not a relocation
// ---------------------------------------------------------------------------------------------

/// One row of [`solve_gain`] — one marched state, both laws differenced in the VALVE.
#[derive(Clone, Copy, Debug)]
pub struct SolveGainRow {
    pub s: f64,
    pub cap_solve: f64,
    pub c: f64,
    /// `|cap_sensed(cap_solve) - cap_solve|` — identity (1), and it is EXACT, not small.
    pub fixed_point: f64,
    /// `d(cap_sensed)/dq` and `d(cap_solve)/dq` at the SAME `w = cap_solve`.
    pub d_s: f64,
    pub d_d: f64,
    /// `dD/dS`, or **`nan` when `dS` is exactly zero** — Python's own conditional. This is the one
    /// NaN reachable by construction anywhere in slice AG, and it is why [`py_max`] exists.
    pub gain: f64,
    pub predicted: f64,
}

/// RUNG 76's `solve_gain` return.
#[derive(Clone, Debug)]
pub struct SolveGain {
    pub phi_lim: f64,
    pub margin: f64,
    pub ref_law: &'static str,
    pub inc: bool,
    pub n: usize,
    pub rows: Vec<SolveGainRow>,
    /// `None` on an empty row set — Python's `if rows else None`, three times.
    pub fixed_point: Option<f64>,
    pub gain: Option<(f64, f64)>,
    pub gain_err: Option<f64>,
}

/// RUNG 76 § 3 — **THE SET-POINT SOLVE IS NOT A RELOCATION OF THE CAP, IT IS A GAIN ON IT.**
///
/// **This section is DERIVATION AFTER MEASUREMENT and is labelled so** — it was written to explain
/// why § 1's `det J` ratio misses `1 - c` by ~0.7 % when the whole fuel row was derived to scale by
/// exactly `1 - c`. It does, at ONE `w`; the two laws are read at DIFFERENT `w` (`mf_app` against
/// `cap_solve`), and that is the entire residual.
///
/// TWO IDENTITIES, both with zero fitted constants:
///
/// 1. `cap_sensed(cap_solve) = cap_solve` EXACTLY — `cap_solve` is by construction the fixed point
///    of `cap_sensed`, so the two laws AGREE at the solve's own answer. This is D2 as a property of
///    the LAWS, and it is why the equilibrium of a leg that HOLDS does not move. The march's tail
///    is NOT that equilibrium — the spools are still spinning up there — which is why the
///    trajectory form of the claim is scored REFUTED and gated as such.
/// 2. `d(cap_solve)/dq = ( d(cap_sensed)/dq ) / ( 1 - c )` at the SAME `w = cap_solve`.
///    Differentiating the fixed point `cap = cap_sensed(cap, q)` gives it in one line. **So the
///    solve AMPLIFIES the cap's sensitivity to every other state by `1/(1-c)`** — a limiter written
///    as a solve is a STIFFER limiter than the schedule it claims to implement, and nothing in
///    rungs 48–75 could see that, because none of them had a second reading of the same cap to
///    difference against.
///
/// `q` is the VALVE state, chosen because it is the one other state the cap depends on through the
/// PLANT and not through any leg's law.
///
/// # THE `nan` GUARD IS A DEFENCE WHOSE READER IS MEASURED, NOT ASSUMED
///
/// [`SolveGainRow::gain`] is `nan` when `dS == 0.0`, and that NaN then flows into
/// [`gain`](SolveGain::gain) and [`gain_err`](SolveGain::gain_err), where Python's `min`/`max`
/// propagate it only if it arrives FIRST and the crate's older `fold(±INF, f64::min)` would discard
/// it always. Step 5's drive COUNTS the exact-zero `dS` occurrences rather than reasoning about
/// them; see § 5.31.5. The faithful fold is [`py_max`]/[`py_min`] and it is used throughout this
/// step.
#[allow(clippy::too_many_arguments)]
pub fn solve_gain(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), ref_law: &'static str, inc: bool,
    r: f64, s_settle: f64, ds: f64, v_max: f64, dq: f64, every: usize,
) -> SolveGain {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let accel = accel_for(core, flight, tt4_lo, tt4_hi, sm, tt4_max, taus, v_max, inc, margin);
    let (m, _surge, _lag, traj) = cap_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
        LAG_COORD_CLIP, ref_law, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE, &accel, None);
    // `m._lag_coord = "demand"` — PLAIN, [`cap_rows`]'s reason.
    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding4(&traj, b_max);
    let mut rows: Vec<SolveGainRow> = Vec::new();
    for p in pts.iter().step_by(every) {
        let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
        let (q, v) = match p.extra {
            PointExtra::Demand { b, v, .. } | PointExtra::Shared { b, v, .. } => (b, v),
            _ => panic!("rung-76's § 3 reads `b`/`v` off every filtered point."),
        };
        // Python's two closures, each setting BOTH state fields and restoring both to `None` in a
        // `finally` — [`c_at`]'s third case of the `b_state`/`v_state` boundary, twice more. The
        // cap law is armed INSIDE those guards, matching `_with_cap`'s position in the source.
        let cap_at = |law: &'static str, qq: f64, mf_app: Option<f64>| -> f64 {
            let _sb = MarchedBleed::set(&m.fuel.inner, qq);
            let _sv = MarchedStator::set(&m.fuel.inner, v);
            let _cs = CapScope::set(&m.fuel.inner, law);
            (m.fuel.inner.triple_hooks.cap_fuel)(
                &m.fuel, flight, a, h, ms, Some(&accel), None, mf_app)
                .unwrap_or_else(|e| panic!("{}", e.0))
        };
        let solve = |qq: f64| cap_at(CAP_LAW_SOLVE, qq, None);
        let sensed = |qq: f64, w: f64| cap_at(CAP_LAW_SENSED, qq, Some(w));
        let w0 = solve(q);
        let c = c_at(&m, flight, a, h, &accel, w0, q, v, C_AT_REL)
            .unwrap_or_else(|e| panic!("{}", e.0));
        let big_s = (sensed(q + dq, w0) - sensed(q - dq, w0)) / (2.0 * dq);
        let big_d = (solve(q + dq) - solve(q - dq)) / (2.0 * dq);
        rows.push(SolveGainRow {
            s: p.s,
            cap_solve: w0,
            c,
            fixed_point: (sensed(q, w0) - w0).abs(),
            d_s: big_s,
            d_d: big_d,
            // Python's `D / S if S != 0.0 else float("nan")` — a VALUE comparison against `+0.0`,
            // which `-0.0` also satisfies (`-0.0 != 0.0` is `False` in both languages), so a
            // `to_bits`-style test here would be stricter than the source.
            gain: if big_s != 0.0 { big_d / big_s } else { f64::NAN },
            predicted: 1.0 / (1.0 - c),
        });
    }
    SolveGain {
        phi_lim, margin, ref_law, inc,
        n: rows.len(),
        fixed_point: if rows.is_empty() { None } else { Some(py_max(&rows, |x| x.fixed_point)) },
        gain: if rows.is_empty() { None }
              else { Some((py_min(&rows, |x| x.gain), py_max(&rows, |x| x.gain))) },
        gain_err: if rows.is_empty() { None }
                  else { Some(py_max(&rows, |x| (x.gain - x.predicted).abs())) },
        rows,
    }
}
