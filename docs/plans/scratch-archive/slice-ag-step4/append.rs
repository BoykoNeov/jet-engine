
// ---------------------------------------------------------------------------------------------
// STEP 4 — THE CAP: the scope guard, the ACCEL-ARMED march, the schedule's OWN plant, and the
// one derivative this rung rests on
// ---------------------------------------------------------------------------------------------

/// Python's `accel_schedule(…, n = 13)` — **the DEFAULT this rung's [`accel_for`] relies on, and
/// the only site in the crate that does.**
///
/// The Rust `accel_schedule` takes `n` positionally because Rust has no defaults, so every other
/// caller in `rust/src` passes a value its own Python line spells out. [`accel_for`] is the one
/// caller whose Python line does NOT — `m.accel_schedule(flight, Tt4_lo, Tt4_hi, margin)` at
/// `engine.py:19301` — so the default is a fact about `engine.py:4694` that has to be carried here
/// by hand. Named rather than typed inline: a bare `13` at the call site is a number nobody can
/// check against a signature, and a wrong `n` is a silently different schedule on a rung whose
/// entire subject is WHICH schedule is being read.
pub const ACCEL_SCHEDULE_N: usize = 13;

/// Python's `_c_at(…, rel = 1e-6)` — the central-difference step, RELATIVE to `w`.
///
/// Carried for [`ACCEL_SCHEDULE_N`]'s reason: both of this rung's callers of [`c_at`] (`_cap_rows`
/// and `solve_gain`, step 5) take the default, so the value is invisible at either call site in
/// Python and would be invisible at neither in Rust.
pub const C_AT_REL: f64 = 1e-6;

/// RUNG 76's `_with_cap` — **a named CAP LAW for the length of one reader, restored in a
/// `finally`.** Rung 62's reason, TENTH reload.
///
/// # IT WRITES THE FIELD DIRECTLY, ON THE SAME EVIDENCE AS ITS PREDECESSOR
///
/// [`WindupScope`] took a direct write because `_with_windup` has exactly one definer over all 58
/// classes; `_with_cap` has exactly one too (`engine.py:19283`, and no other `def _with_cap`
/// anywhere), so there is no later body for a cell to reach and a slot would be dead —
/// [`ShareScope`](crate::shared_actuator::ShareScope)'s decision, third instance.
///
/// # IT RESTORES THE **PREVIOUS** VALUE, WHICH PUTS IT ON THE OTHER SIDE OF THE CRATE'S SPLIT
///
/// § 5.19 (iv) measured 72 reload guards over the nine STATE-kind fields: 68 restore to `None` and
/// 4 restore what they displaced. This is a LAW-kind field, not a state-kind one, and every law
/// guard in the family restores the previous value — Python's `prev = self._cap_law` … `finally:
/// self._cap_law = prev`. **The distinction bites at this rung's own call sites**: step 5's
/// `_cap_rows` nests `_with_windup(…, m._with_cap, …)`, so a `None`-restoring guard here would
/// leave the receiver carrying no cap law at all, and [`r76_integrate_fuel`]'s first refusal —
/// which admits exactly two strings — would fire on the NEXT reader rather than on this one.
///
/// # THE CURRIED FORM IS NOT PORTED, AND THAT IS A SHAPE DIFFERENCE WORTH NAMING
///
/// Python's `_with_cap(law, fn, *a, **kw)` CALLS `fn` inside the `try`. The Rust guard is an RAII
/// value the caller holds, so the callee is spelled at the call site instead of passed to it. The
/// two are equivalent for every use in `engine.py` because no caller passes a `fn` that captures
/// the guard's own lifetime — checked at all five call sites (`engine.py:19351`, `19373`, `19375`,
/// `19591`, `19598`) — and the RAII form additionally survives an unwind, which Python's `finally`
/// also does. Same reason [`WindupScope`] is a guard and not a combinator.
///
/// [`WindupScope`]: crate::anti_windup::WindupScope
pub struct CapScope<'a> {
    core: &'a TwoSpoolTransientCore,
    prev: &'static str,
}

impl<'a> CapScope<'a> {
    /// Arm the named cap law for as long as the returned guard lives.
    pub fn set(core: &'a TwoSpoolTransientCore, law: &'static str) -> Self {
        let prev = core.cap_law.get();
        core.cap_law.set(law);
        CapScope { core, prev }
    }

    /// What this scope displaced — Python's `prev`, exposed so a gate can read the restore POLICY
    /// rather than only its effect.
    /// [`WindupScope::displaced`](crate::anti_windup::WindupScope::displaced)'s precedent, back to
    /// a single value.
    pub fn displaced(&self) -> &'static str {
        self.prev
    }
}

impl Drop for CapScope<'_> {
    fn drop(&mut self) {
        self.core.cap_law.set(self.prev);
    }
}

/// RUNG 76's `_cap_march` — **one rig, one march, under FIVE named knobs AND AN ARMED ACCEL LEG.**
///
/// # THE ARMED LEG IS THE WHOLE STRUCTURAL DIFFERENCE, AND DROPPING IT IS INVISIBLE TO EVERY
/// REDUCE GATE IN THE CRATE
///
/// [`windup_march`](crate::anti_windup::windup_march) builds its [`StatorLeg`] with `accel: None`
/// — no march in this family had ever carried a schedule, which the Python docstring states as
/// *the one thing this family's marches have never done*. Here it is `Some(accel)`, and that is
/// what makes `_cap_fuel`'s accel branch run at all. **Drop it and the failure is silent in the
/// worst available way**: [`r76_sensed_cap`] is never dispatched, the cap is rung 48's set-point
/// solve, and the trajectory is rung 75's — which IS this rung's reduce-arm answer, so every
/// reduce comparison the crate owns goes on passing. That is § 5.31 (ii)'s `0 ADD` blindness one
/// level down, and no reduce can catch it; it needs a POSITIVE count. The pre-flight supplies the
/// number to assert against: on both marched arms the cell is dispatched at **1 366 of 1 366**
/// `_cap_fuel` calls, and under `sensed` the branch is taken at 1 366 of 1 366.
///
/// # IT IS A COPY OF `_windup_march` PLUS TWO LINES, AND IT IS PORTED AS A COPY
///
/// The delegating spelling is not merely unfaithful here, it is WRONG, for the reason step 2 wrote
/// down one rung earlier: the parent RUNS THE MARCH before it returns, so a `cap_march` that called
/// [`windup_march`](crate::anti_windup::windup_march) and then set `cap_law` would set it on a
/// machine that had already marched. Every trajectory would be rung 75's, every reduce gate would
/// pass, and the cap would be reported by a reader that never saw it — the sibling-constructor trap
/// one level up, for the third time in this slice.
///
/// # `_ic_cap` IS STILL CARRIED FROM THE CALLER, AND `_cap_law` COMES FROM THE ARGUMENT
///
/// `self._ic_cap` — the RECEIVER's — because
/// [`contraction_law`](crate::anti_windup::contraction_law) raises the cap inside a `try/finally`
/// and every march it drives has to see the raised value. `cap_law` is the opposite: it comes from
/// the PARAMETER, declared at the call site rather than inherited from whatever `self` happens to
/// carry, which is the trap this family has now been through for five knobs in a row.
#[allow(clippy::too_many_arguments)]
pub fn cap_march(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), r: f64, s_settle: f64, ds: f64, v_max: f64, inc: bool,
    coord: &'static str, ref_law: &'static str, law: &'static str, tau_t: Option<f64>,
    cap_law: &'static str, accel: &AccelSchedule, nu0: Option<(f64, f64)>,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>, Vec<FuelPoint>) {
    let (tau_f, tau_gov, tau_q, tau_s) = taus;
    let (m, surge, lag) = (core.triple_hooks().shared_rig)(core, &SharedRigArm {
        sm,
        tau: tau_q,
        tau_s,
        v_max,
        tt4_max,
        tau_att: tau_f,
        tau_rel: 3.0 * tau_f,
        inc,
        ..Default::default()
    });
    // PLAIN ASSIGNMENTS, all six, exactly as Python spells them — see
    // [`coord_march`](crate::demand_coordinate::coord_march)'s note for why a dispatch here is the
    // defect slice AF step 6 had to repair at four sites.
    m.fuel.inner.lag_coord.set(coord);
    m.fuel.inner.ref_law.set(ref_law);
    m.fuel.inner.windup_law.set(law);
    m.fuel.inner.tau_t.set(tau_t);
    m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());
    m.fuel.inner.cap_law.set(cap_law);
    let leg = StatorLeg { accel: Some(accel), surge, tt4_max: Some(tt4_max) };
    let ramp = Ramp { tt4_lo, tt4_hi, r, s_settle, ds };
    let traj = m.stator_march_scoped(
        flight, &ramp, nu0, &leg,
        &MarchScope { tau_gov: Some(tau_gov), lag, ..MarchScope::DEFAULT }).0;
    (m, surge, lag, traj)
}

/// RUNG 76's `accel_for` — **rung 48's schedule built on THE RIG THAT WILL MARCH IT.**
///
/// # THE SCHEDULE IS A TABLE READ OFF A PLANT, SO THE PLANT IS THE ARGUMENT
///
/// `kappa_ss` is read off the machine's OWN equilibria, so a schedule built on `self` and marched
/// on [`cap_march`]'s rig would be a schedule for a different engine. Python's docstring calls this
/// the fourteenth instance of rungs 61–75's carried-knob trap *wearing its other face* — the thing
/// not carried is here a TABLE rather than a scalar, and the failure is quieter for it: a
/// mismatched schedule still produces a perfectly well-formed cap at every point of the march.
///
/// # THE RIG COMES THROUGH THE HOOK TABLE, AND AT THIS RUNG THAT IS A DISPATCH, NOT A HABIT
///
/// `self._shared_rig(…)` has **eight** definers (rungs 72…80) and rung 76 re-aims the cell
/// precisely so the rig carries `_cap_law` ([`r76_shared_rig`]). Reaching
/// `crate::anti_windup::R75_TRIPLE.shared_rig` directly would build the schedule on a machine at
/// the class default — and since a schedule is read off EQUILIBRIA, which no cap law touches, the
/// table would come back identical and the defect would surface only through whatever else that
/// machine were later used for. So it is called through `core.triple_hooks()` for the RULE rather
/// than for a measured difference, and this comment says which of the two it is.
///
/// # `n` IS PYTHON's DEFAULT AND THE ONLY ONE IN THE CRATE — see [`ACCEL_SCHEDULE_N`]
#[allow(clippy::too_many_arguments)]
pub fn accel_for(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, sm: f64,
    tt4_max: f64, taus: (f64, f64, f64, f64), v_max: f64, inc: bool, margin: f64,
) -> AccelSchedule {
    let (tau_f, _tau_gov, tau_q, tau_s) = taus;
    let m = (core.triple_hooks().shared_rig)(core, &SharedRigArm {
        sm,
        tau: tau_q,
        tau_s,
        v_max,
        tt4_max,
        tau_att: tau_f,
        tau_rel: 3.0 * tau_f,
        inc,
        ..Default::default()
    }).0;
    m.fuel.accel_schedule(flight, tt4_lo, tt4_hi, margin, ACCEL_SCHEDULE_N)
}

/// RUNG 76's `_c_at` — **`c = d(cap_sensed)/dw` AT A POINT, and it is MEASURED.**
///
/// The one number this whole rung rests on. **It is NOT implied by the shipped bracket working**:
/// a bracketing root-finder converges on a sign change whether or not `G = w − cap(w)` is monotone,
/// so `_sched_fuel` bracketing buys *a root exists*, never `G' > 0` (anchor § 0.3). That
/// distinction is rung 83's whole subject, arriving seven rungs early as a measurement.
///
/// # IT RE-SPELLS [`r76_sensed_cap`]'s FORMULA WITHOUT THE LAW CHECK, AND THAT IS DELIBERATE
///
/// The inner `cap` closure is `(1 + margin)·kappa(n_H(x))·pt3(x)` — the SENSED law — with no
/// `_cap_law` consultation, because this reader measures the sensed cap's slope whatever law is
/// currently armed, and every one of its callers is COMPARING the two laws. Routing it through the
/// cell would be a factoring the source refuses, and it would silently return `None` on the arm the
/// reader exists to characterise.
///
/// # BOTH STATE FIELDS ARE SET, WHICH IS THE `b_state`/`v_state` BOUNDARY'S THIRD CASE
///
/// [`MarchedStator`]'s own doc states the rule: a law that TRIALS an actuator must not see that
/// actuator's state and must see the other two, so the valve law sets `v_state` alone and the
/// stator law `b_state` alone — **and a law that trials NEITHER sets BOTH**. This trials fuel, so
/// it is that third case, and Python spells it as one tuple assignment. Both guards restore `None`
/// on drop, which is Python's `finally: self._b_state, self._v_state = None, None` exactly; note
/// that this is the OPPOSITE policy from [`CapScope`] three items up, and the two sit in one file
/// because the fields are of different KINDS.
///
/// # THE `max` IS EXPRESSION-FIRST, AND IT IS THE ONLY ONE IN THE PACKAGE
///
/// `dw = rel * max(w, 1e-9)`. Slice AG step 1 censused all 268 n-ary `max`/`min` calls in
/// `engine.py`: 103 put a literal first — every one faithful under `lit.max(x)`, because Python
/// seeds its fold at argument 0 and replaces only on a strict comparison, so a NaN in argument 1 is
/// discarded exactly as Rust's `f64::max` discards it. **This call is the single expression-first
/// `1e-9` fold in the whole package**, and there the two spellings DISAGREE: `max(w, 1e-9)` is
/// `nan` for a NaN `w` where `w.max(1e-9)` is `1e-9`. So it is written as the explicit fold. The
/// obligation was written down at [`demand_coordinate`](crate::demand_coordinate)'s C-law note at
/// step 1, naming this method and this step, and it is discharged here rather than re-derived.
pub fn c_at(
    core: &ScheduledStatorCore, flight: &FlightCondition, a: f64, h: f64, accel: &AccelSchedule,
    w: f64, q: f64, v: f64, rel: f64,
) -> Result<f64, Abort> {
    // Python's `max(w, 1e-9)` — argument 0 is the EXPRESSION, so `w.max(1e-9)` is the wrong
    // spelling on a NaN `w`. See this function's doc for the census that decides it.
    let dw = rel * if 1e-9 > w { 1e-9 } else { w };
    let _sb = MarchedBleed::set(&core.fuel.inner, q);
    let _sv = MarchedStator::set(&core.fuel.inner, v);
    let cap = |x: f64| -> Result<f64, Abort> {
        let i = core.fuel.try_instant_fuel(flight, a, h, x)?;
        Ok(accel.cap(i.base.close.n_hp, i.base.close.pt4 / core.fuel.inner.inner.base.pi_b))
    };
    Ok((cap(w + dw)? - cap(w - dw)?) / (2.0 * dw))
}
