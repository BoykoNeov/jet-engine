
// =============================================================================================
// § 5 — THE PLANT, AND THE ONE QUESTION THAT CAN ACTUALLY FAIL (step 3)
// =============================================================================================

/// Restores § 5's flag and log on the way out — the `finally` of Python's `_with_probe`
/// (`engine.py:20983`). On an unwind it ONLY restores; the log is handed back on success alone,
/// as Python's `return out, log` sits after the `finally`.
struct ProbeGuard {
    prev_p: bool,
    prev_l: Option<Vec<CoordLogRow>>,
}

impl Drop for ProbeGuard {
    fn drop(&mut self) {
        COORD_PROBE.with(|c| c.set(self.prev_p));
        COORD_LOG.with(|l| *l.borrow_mut() = self.prev_l.take());
    }
}

/// Arm § 5's instrument around `f` and hand back its log, always cleared and disarmed — Python's
/// `_with_probe`.
///
/// **THE FLAG IS PER-THREAD STATE, NOT A CORE FIELD**, which is the Rust spelling of Python's
/// fix: the first version wrote `self._coord_probe = True`, and because `_cap_march` builds a NEW
/// machine through `at_lever`, the marching object read the class default and the log came back
/// EMPTY while `hits`/`binds` read a flawless 1366/1366. A core field here would be that instance
/// attribute again (module header).
///
/// **RESTORE-PREVIOUS, both halves**: the displaced flag and the displaced LOG, so a nested probe
/// leaves the outer one's rows where it found them. The log is TAKEN before the guard restores —
/// Python reads `_coord_log` inside its `finally`, before it writes `prev_l` back.
pub fn with_probe<T>(f: impl FnOnce() -> T) -> (T, Vec<CoordLogRow>) {
    let guard = ProbeGuard {
        prev_p: COORD_PROBE.with(Cell::get),
        prev_l: COORD_LOG.with(|l| l.borrow_mut().replace(Vec::new())),
    };
    COORD_PROBE.with(|c| c.set(true));
    let out = f();
    let log = COORD_LOG.with(|l| l.borrow_mut().take()).expect(
        "rung-79: the probe's log vanished inside its own scope -- only `ProbeGuard` writes the \
         slot, and a nested guard restores what it found");
    drop(guard);
    (out, log)
}

/// § 5's whole reading — Python's `coord_march` return dict.
#[derive(Clone, Debug)]
pub struct CoordMarch {
    pub phi_lim: f64,
    pub margin: f64,
    pub inc: bool,
    /// Steps compared: `min(len(traj0), len(traj1))`.
    pub n: usize,
    pub same_len: bool,
    /// **P4**: worst relative difference over `nu_lp, nu_hp, mf, b, v`, incidence against `phi`.
    pub worst: f64,
    /// Where it sits — `None` if nothing differed (Python's strict `>` from `0.0`).
    pub where_: Option<(&'static str, usize)>,
    /// **P3**: the re-coordinated branch RAN …
    pub hits: u64,
    /// … and its value WON the min.
    pub binds: u64,
    /// Log rows whose accel cap is finite.
    pub n_armed: usize,
    pub n_log: usize,
    /// § 5.1's counters, split by coordinate — the `phi` column is the PROBE's, the incidence
    /// column the plant's own.
    pub calls_phi: u64,
    pub calls_inc: u64,
    pub fb_phi: u64,
    pub fb_inc: u64,
    /// `calls − fb`: solves that actually BRACKETED the coordinated residual. **`i64`, not
    /// `u64`**: a fallback bumps before `_surge_fuel` can raise, and a call bumps only after the
    /// solve returns, so the difference is not a count by construction.
    pub br_phi: i64,
    /// … and the number that decides whether § 5 exercised the KNOB at all.
    pub br_inc: i64,
    /// Distinct `p_phi` FLOATS in the log — **not distinct STATES** (`docs/rung79-gap-margin.md`
    /// § 4.1: the plant never leaves its initial state).
    pub n_distinct: usize,
    /// Distinct accel caps.
    pub n_distinct_accel: usize,
    /// § 5.3: calls whose residual WAS bracketed …
    pub n_live: usize,
    /// … calls whose cap beat the schedule …
    pub n_reach: usize,
    /// … and both. MUST be `0` — an identity of `_cap_free`'s branch condition.
    pub n_both: usize,
    /// **P2**: armed rows where the two coordinates' caps sit on different sides of the accel cap.
    pub flips: usize,
    /// **P2n**: the closest the two legs ever came, relative.
    pub gap_min: Option<f64>,
    /// The UPPER median — `sorted(gaps)[len // 2]`, an index, not an average.
    pub gap_med: Option<f64>,
    pub gap_max: Option<f64>,
    pub n_distinct_gap: usize,
    /// The largest relative float move the coordinate made on the plant's cap.
    pub d_max: Option<f64>,
    /// Upper median, as `gap_med`.
    pub d_med: Option<f64>,
    /// **THE GUARD**: `Some(true)` when `d_max == 0.0` — the STRONGEST vacuity, which Python's
    /// first version sent to `None` — else `gap_min > 1e3·d_max`, else `None` on an empty log.
    pub vacuous: Option<bool>,
    /// The schedule is NOT a function of the coordinate. **Not an `Option`**: Python's
    /// `max([…] + […])` has no `default` and raises on empty.
    pub sched_moved: f64,
}

/// Python's `len(set(xs))` over floats: `-0.0` and `0.0` are one element (they compare equal and
/// hash alike) and every `NaN` is its own. Sorted by `total_cmp`, which places the two zeros side
/// by side, then deduplicated on `==`, which never merges a `NaN`.
fn py_set_len(xs: impl Iterator<Item = f64>) -> usize {
    let mut v: Vec<f64> = xs.collect();
    v.sort_by(f64::total_cmp);
    v.dedup_by(|a, b| a == b);
    v.len()
}

/// Python's `sorted(xs)[len(xs) // 2]` — the UPPER median. `sorted` is stable and compares with
/// `<`; every argument here is an `abs` over a positive floor, so no `NaN` and no `-0.0` arrives,
/// and a stable `partial_cmp` sort is the same permutation.
fn py_upper_median(xs: &[f64]) -> f64 {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).expect("rung-79 § 5: a NaN gap or delta"));
    v[v.len() / 2]
}

/// Python's two-argument `min(x, y)` — `x` unless `y` is STRICTLY smaller.
fn py_min2(x: f64, y: f64) -> f64 {
    if y < x { y } else { x }
}

/// § 5: **the whole march, in each coordinate, with the min-select watched.** Python's
/// `coord_march` (`engine.py:21125`); the log is dropped — see [`coord_march_logged`].
#[allow(clippy::too_many_arguments)]
pub fn coord_march(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64,
    ds: f64, v_max: f64,
) -> CoordMarch {
    coord_march_logged(core, flight, tt4_lo, tt4_hi, tt4_max, phi_lim, margin, taus, inc, r,
                       s_settle, ds, v_max).0
}

/// [`coord_march`] with the probe's LOG handed back beside the reading.
///
/// Python's reader consumes the log and returns only aggregates of it; the port's gate digests the
/// log itself (every row's bits, in order) against the Python probe, because six of the reading's
/// keys are ORDER STATISTICS or SET SIZES of it and none of them can see a row out of place.
///
/// # THE FOUR COORDINATE SCOPES DISPATCH, AND THAT IS THE RULE'S OWN CASE
///
/// `self._with_coord(…)` four times (`engine.py:21151`, `:21157`, `:21165`, `:21172` — the last a
/// bound-method reference handed to `_with_probe`), each through
/// [`CoordScope`](crate::demand_coordinate::CoordScope) and so through the table: at rung 79 the
/// cell writes `phi_ref`, and [`r79_shared_rig`] carries it onto every rig `accel_for` and
/// `cap_march` build.
///
/// # THE COUNTERS ARE RESET AFTER THE `phi` MARCH, NOT AT THE TOP, AS PYTHON RESETS THEM
///
/// Measured in Python: neither `accel_for` call moves a counter and the `phi` march dispatches to
/// rung 78's body, which bumps none, so a reset at the top returns the same numbers here. It is
/// kept where the source puts it — a reading on this rig cannot tell the two apart.
///
/// # THE ONE SECTION ON THE PLANT, AND ON THE SHIPPED RIG IT IS VACUOUS BY ITS OWN GUARD
///
/// Measured in Python and gated in `tests/slice_ai_march.rs`: 1 366 calls, 1 363 fell back, 3
/// bracketed the incidence residual (`br_inc = 3`), `d_max = 0.0` and so `vacuous = Some(true)`.
/// `tests/test_rung79.py` asserts that vacuity as a disclosure, and the three live calls are the
/// whole of what § 5 ever asked the knob.
#[allow(clippy::too_many_arguments)]
pub fn coord_march_logged(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64,
    ds: f64, v_max: f64,
) -> (CoordMarch, Vec<CoordLogRow>) {
    let t = &core.fuel.inner;
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let accel0 = {
        let _cs = CoordScope::set(t, PHI_REF_PHI);
        accel_for(core, flight, tt4_lo, tt4_hi, sm, tt4_max, taus, v_max, inc, margin)
    };
    // THE SCHEDULE MUST NOT MOVE WITH THE COORDINATE -- checked, not assumed.
    let sched1 = {
        let _cs = CoordScope::set(t, PHI_REF_INCIDENCE);
        accel_for(core, flight, tt4_lo, tt4_hi, sm, tt4_max, taus, v_max, inc, margin)
    };
    let moved: Vec<f64> = sched1.kappa.iter().zip(accel0.kappa.iter())
        .chain(sched1.n_h.iter().zip(accel0.n_h.iter()))
        .map(|(&x, &y)| rel_err(x, y, 1e-30))
        .collect();
    let sched_moved = py_max_of(&moved);
    let march = || cap_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
        "demand", "sched", "none", None, "solve", &accel0, None).3;
    let traj0 = {
        let _cs = CoordScope::set(t, PHI_REF_PHI);
        march()
    };
    reset_coord_counters();
    // `self._with_probe(self._with_coord, "incidence", self._cap_march, *args)` -- the probe
    // OUTSIDE, the coordinate inside, so the coordinate is restored first.
    let (traj1, log) = with_probe(|| {
        let _cs = CoordScope::set(t, PHI_REF_INCIDENCE);
        march()
    });
    let c = coord_counters();
    let armed: Vec<&CoordLogRow> = log.iter().filter(|x| x.a_cap != f64::INFINITY).collect();
    let flips = armed.iter()
        .filter(|x| (x.p_phi <= x.a_cap) != (x.p_cap <= x.a_cap))
        .count();
    let gaps: Vec<f64> = armed.iter().map(|x| rel_err(x.a_cap, x.p_phi, 1e-30)).collect();
    let dels: Vec<f64> = log.iter().map(|x| rel_err(x.p_cap, x.p_phi, 1e-30)).collect();
    // § 5.3 -- THE COMPLEMENTARITY, counted so the identity is checked and not merely asserted.
    let n_live = log.iter().filter(|x| !x.used_fb).count();
    let n_reach = log.iter().filter(|x| py_min2(x.a_cap, x.p_cap) < x.mf_sched).count();
    let n_both = log.iter()
        .filter(|x| !x.used_fb && py_min2(x.a_cap, x.p_cap) < x.mf_sched)
        .count();
    let n = traj0.len().min(traj1.len());
    let (mut worst, mut where_): (f64, Option<(&'static str, usize)>) = (0.0, None);
    for key in MARCH_KEYS {
        for i in 0..n {
            let (x, y) = match (march_key(&traj1[i], key), march_key(&traj0[i], key)) {
                (Some(x), Some(y)) => (x, y),
                _ => continue,
            };
            let e = rel_err(x, y, 1e-12);
            if e > worst {
                worst = e;
                where_ = Some((key, i));
            }
        }
    }
    let opt = |xs: &[f64], f: fn(&[f64]) -> f64| -> Option<f64> {
        if xs.is_empty() { None } else { Some(f(xs)) }
    };
    let d_max = opt(&dels, py_max_of);
    let vacuous = if d_max == Some(0.0) {
        Some(true)
    } else if !gaps.is_empty() && !dels.is_empty() {
        Some(py_min_of(&gaps) > 1e3 * py_max_of(&dels))
    } else {
        None
    };
    let out = CoordMarch {
        phi_lim,
        margin,
        inc,
        n,
        same_len: traj0.len() == traj1.len(),
        worst,
        where_,
        hits: c.hits,
        binds: c.binds,
        n_armed: armed.len(),
        n_log: log.len(),
        calls_phi: c.calls_phi,
        calls_inc: c.calls_inc,
        fb_phi: c.fb_phi,
        fb_inc: c.fb_inc,
        br_phi: c.calls_phi as i64 - c.fb_phi as i64,
        br_inc: c.calls_inc as i64 - c.fb_inc as i64,
        n_distinct: py_set_len(log.iter().map(|x| x.p_phi)),
        n_distinct_accel: py_set_len(log.iter().map(|x| x.a_cap)),
        n_live,
        n_reach,
        n_both,
        flips,
        gap_min: opt(&gaps, py_min_of),
        gap_med: opt(&gaps, py_upper_median),
        gap_max: opt(&gaps, py_max_of),
        n_distinct_gap: py_set_len(gaps.iter().copied()),
        d_max,
        d_med: opt(&dels, py_upper_median),
        vacuous,
        sched_moved,
    };
    (out, log)
}

// =============================================================================================
// § 5.2 — THE SHORT-CIRCUIT BYPASSED, WHICH IS WHAT MAKES § 5 READABLE (step 3)
// =============================================================================================

/// [`forced_cap`]'s bracket growth — Python's `grow = 1.0 / 0.9`, rung 74's own direction.
pub const FORCED_GROW: f64 = 1.0 / 0.9;

/// [`forced_cap`]'s bracket shrink — Python's `shrink = 0.9`, `_surge_fuel`'s.
pub const FORCED_SHRINK: f64 = 0.9;

/// [`forced_cap`]'s bracket search length — Python's `n = 60`.
pub const FORCED_N: usize = 60;

/// [`forced_cap`]'s Illinois tolerance — Python's literal `tol=1e-13`. **Its OWN constant**, not
/// [`LEG_TOL`](FuelTransientCore::LEG_TOL): the two agree today, and `d_shipped == 0.0` holds
/// only while this body and `_surge_fuel`'s are the same instruction sequence, so neither should
/// move when the other does.
pub const FORCED_TOL: f64 = 1e-13;

/// The phi leg's set point with `_cap_free`'s BINDING SHORT-CIRCUIT BYPASSED — Python's
/// `_forced_cap` (`engine.py:21256`). **An instrument, never the plant.**
///
/// # IN THE `phi` COORDINATE IT IS A COPY OF `_surge_fuel`, AND § 5.2's `d_shipped == 0` IS THAT
///
/// Same residual, same `0.9` walk from `mf_sched`, same `(lo, hi, glo, ghi)` order, same `1e-13`
/// Illinois — so the shipped suite's exact `d_shipped == 0.0` is a statement about two runs of
/// one arithmetic (slice F's copy-versus-rederivation discriminator). Two differences from
/// [`try_surge_fuel`](FuelTransientCore::try_surge_fuel) are Python's and are kept: this walk
/// does NOT reset `glo` after a non-negative reading (the check after it re-tests the sign
/// instead), and it bumps no rung-49 counter.
///
/// # THE TWO WALKS HANDLE A FAILED EVALUATION DIFFERENTLY, AS PYTHON's DO
///
/// Binding (walk DOWN): a failure is skipped (`continue`) and `glo` keeps its STALE previous
/// value. Slack (walk UP): a failure ends the walk with `ghi = None`, and a non-positive reading
/// resets `ghi` to `None` — `cap_free`'s shape. Every `Abort` stands for Python's
/// `AssertionError`, [`cap_free`]'s precedent. `g0` and the Illinois solve are unguarded, so their
/// failures propagate.
///
/// Both refusals are `Abort`s. Python formats `mf_sched` with `:.6e`, which pads the exponent to
/// two digits; Rust's `{:.6e}` does not, [`cap_free`]'s recorded divergence — the gates match the
/// phrase, never the number.
#[allow(clippy::too_many_arguments)]
pub fn forced_cap(
    ft: &FuelTransientCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64,
    surge: &Floor, coord: &'static str, grow: f64, shrink: f64, n: usize,
) -> Result<f64, Abort> {
    let big_g = phi_residual(ft, flight, a, h, surge, Some(coord));
    let g0 = big_g(mf_sched)?;
    let (lo, hi, glo, ghi);
    if g0 > 0.0 {
        // BINDING: walk DOWN to the sign change.
        let mut l = mf_sched;
        let mut gl: Option<f64> = None;
        for _ in 0..n {
            l *= shrink;
            match big_g(l) {
                Err(_) => continue,
                Ok(v) => gl = Some(v),
            }
            if gl.expect("just assigned") < 0.0 {
                break;
            }
        }
        let Some(gl) = gl.filter(|&g| g < 0.0) else {
            return Err(Abort(format!(
                "rung-79: the forced bracket found no sign change below mf_sched = \
                 {mf_sched:.6e} (searched to {l:.6e}) in the '{coord}' coordinate.")));
        };
        (lo, hi, glo, ghi) = (l, mf_sched, gl, g0);
    } else {
        // SLACK: walk UP, rung 74's own direction.
        let mut hh = mf_sched;
        let mut gh: Option<f64> = None;
        for _ in 0..n {
            hh *= grow;
            match big_g(hh) {
                Err(_) => {
                    gh = None;
                    break;
                }
                Ok(v) => gh = Some(v),
            }
            if gh.expect("just assigned") > 0.0 {
                break;
            }
            gh = None;
        }
        let Some(gh) = gh else {
            return Err(Abort(format!(
                "rung-79: the forced bracket found no sign change above mf_sched = \
                 {mf_sched:.6e} (searched to {hh:.6e}) in the '{coord}' coordinate.")));
        };
        (lo, hi, glo, ghi) = (mf_sched, hh, g0, gh);
    }
    try_illinois(|w| big_g(w), lo, hi, glo, ghi, FORCED_TOL, ILLINOIS_MAXIT)
}

/// One riding point of [`coord_forced`].
#[derive(Clone, Copy, Debug)]
pub struct CoordForcedRow {
    pub s: f64,
    /// `Gs(mf_sched) > 0` — the regime the plant is in here.
    pub binding: bool,
    /// The forced solve in `phi` …
    pub w_phi: f64,
    /// … and in incidence.
    pub w_inc: f64,
    /// The SHIPPED `_surge_fuel` at the same frozen state.
    pub w_shipped: f64,
    /// `|w_inc − w_phi| / max(|w_phi|, 1e-30)` — the coordinate's true float footprint.
    pub d_forced: f64,
    /// `|w_phi − w_shipped| / max(|w_shipped|, 1e-30)` — the bypass measures the SAME set point.
    pub d_shipped: f64,
    pub same_float: bool,
}

/// § 5.2's whole reading — Python's `coord_forced` return dict.
#[derive(Clone, Debug)]
pub struct CoordForced {
    pub phi_lim: f64,
    pub margin: f64,
    pub inc: bool,
    pub n: usize,
    pub rows: Vec<CoordForcedRow>,
    /// Python's `sum(…)` — a plain count, no `if rows else None`.
    pub n_binding: usize,
    /// THE NUMBER § 5 CANNOT PRODUCE.
    pub d_forced: Option<f64>,
    /// The UPPER median of the per-row `d_forced`.
    pub d_forced_med: Option<f64>,
    pub n_same_float: usize,
    pub d_shipped: Option<f64>,
}

/// § 5.2: **the same set point, both coordinates, with the short-circuit bypassed.** Python's
/// `coord_forced` (`engine.py:21306`).
///
/// On rung 78's own points (through [`gauge_points`]), one freeze per point — set before the
/// binding test and cleared after the shipped solve, Python's `try/finally`, which CLOBBERS to
/// `None`. Measured in Python and gated in `tests/slice_ai_march.rs`: every point binds, the
/// coordinate moves the root by up to `6.1e-15` (1 of 10 rows the same float), and the forced `phi`
/// solve reproduces the shipped one to the bit. **It moves no rung-79 counter** — it calls
/// [`phi_residual`] and [`forced_cap`], never [`phi_cap`] — so, unlike §§ 1–3, the incidence
/// numbers here are the incidence residual's own.
#[allow(clippy::too_many_arguments)]
pub fn coord_forced(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64,
    ds: f64, v_max: f64, every: usize,
) -> CoordForced {
    let (m, surge, _accel, pts) = gauge_points(
        core, flight, tt4_lo, tt4_hi, tt4_max, margin, taus, r, s_settle, ds, v_max, inc,
        phi_lim, every);
    let mut rows: Vec<CoordForcedRow> = Vec::with_capacity(pts.len());
    for p in pts.iter() {
        let surge = surge.as_ref().expect("rung-79 § 5.2: `_gauge_points` armed no phi floor");
        let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
        let (q, v) = crate::stiffness_ledger::bv_of(p);
        let (binding, w_p, w_i, w_s) = {
            let _sb = MarchedBleed::set(&m.fuel.inner, q);
            let _sv = MarchedStator::set(&m.fuel.inner, v);
            let big_g = phi_residual(&m.fuel, flight, a, h, surge, Some(PHI_REF_PHI));
            let binding = big_g(ms).unwrap_or_else(boom) > 0.0;
            let forced = |coord: &'static str| -> f64 {
                forced_cap(&m.fuel, flight, a, h, ms, surge, coord, FORCED_GROW, FORCED_SHRINK,
                           FORCED_N).unwrap_or_else(boom)
            };
            let w_p = forced(PHI_REF_PHI);
            let w_i = forced(PHI_REF_INCIDENCE);
            let w_s = m.fuel.try_surge_fuel(flight, a, h, ms, surge).unwrap_or_else(boom);
            (binding, w_p, w_i, w_s)
        };
        rows.push(CoordForcedRow {
            s: p.s,
            binding,
            w_phi: w_p,
            w_inc: w_i,
            w_shipped: w_s,
            d_forced: rel_err(w_i, w_p, 1e-30),
            d_shipped: rel_err(w_p, w_s, 1e-30),
            same_float: w_i == w_p,
        });
    }
    let d_forced: Vec<f64> = rows.iter().map(|x| x.d_forced).collect();
    let d_shipped: Vec<f64> = rows.iter().map(|x| x.d_shipped).collect();
    CoordForced {
        phi_lim,
        margin,
        inc,
        n: rows.len(),
        n_binding: rows.iter().filter(|x| x.binding).count(),
        d_forced: if rows.is_empty() { None } else { Some(py_max_of(&d_forced)) },
        d_forced_med: if rows.is_empty() { None } else { Some(py_upper_median(&d_forced)) },
        n_same_float: rows.iter().filter(|x| x.same_float).count(),
        d_shipped: if rows.is_empty() { None } else { Some(py_max_of(&d_shipped)) },
        rows,
    }
}
