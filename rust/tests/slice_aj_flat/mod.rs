//! SLICE AJ's FLATTENERS — one per returned shape, shared by every `slice_aj_*.rs` binary.
//!
//! Each writes Python's `flat` (`rust/oracle/probe_slice_aj_step{2,3,4}.py`,
//! `rust/oracle/dump_slice_aj.py`): one `(path, token)` per value IN PYTHON's DICT ORDER — a dict
//! `keys:N` then its items, a list `len:N` then its items, a float its IEEE bits (`f:nan` for a
//! NaN), `None` as `n`. A field missing, extra, renamed or out of order fails at its path.
//!
//! Written at steps 2–4, one copy per binary; LIFTED here at step 6 (plan § 5.34.6) so the
//! oracle (`slice_aj_oracle.rs`) reuses the converters those binaries VERIFY against their own
//! goldens, rather than describing every struct a second time. The two helpers that differed
//! between the copies are reconciled here: `f` spells a NaN `f:nan` (steps 3/4's; step 2's
//! raw-bits `f` never met one), and `Flat` carries every helper any copy had.
//!
//! Not a test target: a subdirectory `mod.rs`, pulled in by `mod slice_aj_flat;`.

#![allow(dead_code, unused_imports)]

use turbojet::authority_clock::{
    AuthStats, AuthorityClock, AuthorityMask, ClipControlRow, ClockRow, Criterion, MaskArm,
    MaskCell, TauFInert,
};
use turbojet::corrector_law::{
    self, Change, CorrectorRead, CorrectorSecant, CorrectorStep, ResidualShape, SecantStep,
};
use turbojet::fuel_transient::Authority;
use turbojet::staircase_law::{
    self, Classified, EdgeRead, LatticeCount, RootClass, StaircaseNumber, StaircaseScan,
};
use turbojet::stator_transient::ScheduledStatorCore;
use turbojet::threshold_law::{
    self, AtOk, Bisect, RefFull, RefRow, RowOk, RowVoid, TermsAt, ThresholdLaw,
    ThresholdReference, ThresholdRow, ThresholdScan, ThresholdTerms, P4, P5,
};

#[derive(Default)]
pub struct Flat(pub Vec<(String, String)>);

impl Flat {
    pub fn put(&mut self, p: &str, t: String) {
        self.0.push((p.to_string(), t));
    }
    pub fn f(&mut self, p: &str, x: f64) {
        if x.is_nan() {
            self.put(p, "f:nan".into());
        } else {
            self.put(p, format!("f:{:016x}", x.to_bits()));
        }
    }
    pub fn of(&mut self, p: &str, x: Option<f64>) {
        match x {
            Some(x) => self.f(p, x),
            None => self.none(p),
        }
    }
    pub fn i(&mut self, p: &str, x: usize) {
        self.put(p, format!("i:{x}"));
    }
    pub fn oi(&mut self, p: &str, x: Option<i64>) {
        match x {
            Some(x) => self.put(p, format!("i:{x}")),
            None => self.none(p),
        }
    }
    pub fn b(&mut self, p: &str, x: bool) {
        self.put(p, format!("b:{}", x as u8));
    }
    pub fn ob(&mut self, p: &str, x: Option<bool>) {
        match x {
            Some(x) => self.b(p, x),
            None => self.none(p),
        }
    }
    pub fn s(&mut self, p: &str, x: &str) {
        self.put(p, format!("s:{x}"));
    }
    pub fn os(&mut self, p: &str, x: Option<&str>) {
        match x {
            Some(x) => self.s(p, x),
            None => self.none(p),
        }
    }
    pub fn none(&mut self, p: &str) {
        self.put(p, "n".into());
    }
    pub fn lab(&mut self, p: &str, x: Option<Authority>) {
        self.os(p, x.map(|a| a.as_str()));
    }
    pub fn kind(&mut self, p: &str, k: Option<staircase_law::Kind>) {
        self.os(p, k.map(|k| k.as_str()));
    }
    pub fn len(&mut self, p: &str, n: usize) {
        self.put(p, format!("len:{n}"));
    }
    pub fn keys(&mut self, p: &str, n: usize) {
        self.put(p, format!("keys:{n}"));
    }
    pub fn fs(&mut self, p: &str, xs: &[f64]) {
        self.len(p, xs.len());
        for (k, &x) in xs.iter().enumerate() {
            self.f(&format!("{p}.{k}"), x);
        }
    }
    pub fn ofs(&mut self, p: &str, xs: &[Option<f64>]) {
        self.len(p, xs.len());
        for (k, &x) in xs.iter().enumerate() {
            self.of(&format!("{p}.{k}"), x);
        }
    }
    pub fn is(&mut self, p: &str, xs: &[usize]) {
        self.len(p, xs.len());
        for (k, &x) in xs.iter().enumerate() {
            self.i(&format!("{p}.{k}"), x);
        }
    }
    pub fn pair(&mut self, p: &str, a: f64, b: f64) {
        self.len(p, 2);
        self.f(&format!("{p}.0"), a);
        self.f(&format!("{p}.1"), b);
    }
    pub fn pairs(&mut self, p: &str, xs: &[(f64, f64)]) {
        self.len(p, xs.len());
        for (k, &(a, b)) in xs.iter().enumerate() {
            self.pair(&format!("{p}.{k}"), a, b);
        }
    }
    pub fn census(&mut self, p: &str, c: &[(Authority, usize)]) {
        self.keys(p, c.len());
        for (a, n) in c {
            self.i(&format!("{p}.{}", a.as_str()), *n);
        }
    }
}

// ======================================== from `slice_aj_clock.rs` (step 2)

//
// The probe's `flat`, mirrored: one `(path, token)` per value, in Python's dict order.


pub fn criterion(o: &mut Flat, p: &str, c: &Criterion) {
    o.keys(p, 10);
    o.f(&format!("{p}.s"), c.s);
    o.f(&format!("{p}.setpoint_gap"), c.setpoint_gap);
    o.f(&format!("{p}.lag_gap"), c.lag_gap);
    o.f(&format!("{p}.tau_f"), c.tau_f);
    o.f(&format!("{p}.tau_gov"), c.tau_gov);
    o.f(&format!("{p}.slope_f"), c.slope_f);
    o.f(&format!("{p}.slope_r"), c.slope_r);
    o.s(&format!("{p}.predicted"), c.predicted.as_str());
    o.s(&format!("{p}.measured"), c.measured.as_str());
    o.f(&format!("{p}.margin"), c.margin);
}

pub fn clock_row(o: &mut Flat, p: &str, x: &ClockRow) {
    o.keys(p, 20);
    o.s(&format!("{p}.coord"), x.coord);
    o.f(&format!("{p}.tau_f"), x.tau_f);
    o.f(&format!("{p}.tau_gov"), x.tau_gov);
    o.f(&format!("{p}.tau_q"), x.tau_q);
    o.f(&format!("{p}.tau_s"), x.tau_s);
    o.f(&format!("{p}.phi_lim"), x.phi_lim);
    o.of(&format!("{p}.phi_air"), x.phi_air);
    o.f(&format!("{p}.max_Tt4"), x.max_tt4);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.census(&format!("{p}.census"), &x.census);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.i(&format!("{p}.n_gov"), x.n_gov);
    o.i(&format!("{p}.n_scored"), x.n_scored);
    o.i(&format!("{p}.n_edge"), x.n_edge);
    o.i(&format!("{p}.n_agree"), x.n_agree);
    o.of(&format!("{p}.agreement"), x.agreement);
    o.of(&format!("{p}.worst_miss"), x.worst_miss);
    o.of(&format!("{p}.min_margin"), x.min_margin);
    o.len(&format!("{p}.cells"), x.cells.len());
    for (k, c) in x.cells.iter().enumerate() {
        criterion(o, &format!("{p}.cells.{k}"), c);
    }
}

pub fn ctl_row(o: &mut Flat, p: &str, x: &ClipControlRow) {
    o.keys(p, 5);
    o.f(&format!("{p}.tau_f"), x.tau_f);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.census(&format!("{p}.census"), &x.census);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.f(&format!("{p}.max_Tt4"), x.max_tt4);
}

pub fn inert(o: &mut Flat, p: &str, v: &Option<TauFInert>) {
    let Some(v) = v else {
        o.put(p, "n".into());
        return;
    };
    o.keys(p, 8);
    o.i(&format!("{p}.n_tau_f"), v.n_tau_f);
    o.i(&format!("{p}.n_floats"), v.n_floats);
    o.i(&format!("{p}.n_differing"), v.n_differing);
    o.b(&format!("{p}.march_identical"), v.march_identical);
    o.b(&format!("{p}.Tt4_identical"), v.tt4_identical);
    o.b(&format!("{p}.riding4_identical"), v.riding4_identical);
    o.is(&format!("{p}.n_riding4"), &v.n_riding4);
    o.b(&format!("{p}.all_valid"), v.all_valid);
}

pub fn clock(o: &mut Flat, p: &str, r: &AuthorityClock) {
    o.keys(p, 23);
    o.f(&format!("{p}.phi_lim"), r.phi_lim);
    o.of(&format!("{p}.phi_air"), r.phi_air);
    o.fs(&format!("{p}.tau_fs"), &r.tau_fs);
    o.fs(&format!("{p}.tau_govs"), &r.tau_govs);
    o.f(&format!("{p}.tau_q"), r.tau_q);
    o.f(&format!("{p}.tau_s"), r.tau_s);
    o.len(&format!("{p}.coords"), r.coords.len());
    for (k, c) in r.coords.iter().enumerate() {
        o.s(&format!("{p}.coords.{k}"), c);
    }
    o.f(&format!("{p}.ds"), r.ds);
    o.len(&format!("{p}.rows"), r.rows.len());
    for (k, x) in r.rows.iter().enumerate() {
        clock_row(o, &format!("{p}.rows.{k}"), x);
    }
    o.b(&format!("{p}.control_all_gov"), r.control_all_gov);
    o.is(&format!("{p}.control_n_riding4"), &r.control_n_riding4);
    o.census(&format!("{p}.control_clip_shared"), &r.control_clip_shared);
    o.i(&format!("{p}.control_clip_fuel"), r.control_clip_fuel);
    o.len(&format!("{p}.control_clip_rows"), r.control_clip_rows.len());
    for (k, x) in r.control_clip_rows.iter().enumerate() {
        ctl_row(o, &format!("{p}.control_clip_rows.{k}"), x);
    }
    o.b(&format!("{p}.control_clip_tau_f_live"), r.control_clip_tau_f_live);
    o.b(&format!("{p}.all_fuel"), r.all_fuel);
    o.of(&format!("{p}.agreement"), r.agreement);
    o.i(&format!("{p}.n_scored"), r.n_scored);
    o.of(&format!("{p}.worst_miss"), r.worst_miss);
    o.keys(&format!("{p}.fuel_cells"), r.fuel_cells.len());
    for (c, cells) in &r.fuel_cells {
        let q = format!("{p}.fuel_cells.{c}");
        o.len(&q, cells.len());
        for (k, (tf, tg, n)) in cells.iter().enumerate() {
            o.len(&format!("{q}.{k}"), 3);
            o.f(&format!("{q}.{k}.0"), *tf);
            o.f(&format!("{q}.{k}.1"), *tg);
            o.i(&format!("{q}.{k}.2"), *n);
        }
    }
    o.keys(&format!("{p}.fuel_side"), r.fuel_side.len());
    for (c, side) in &r.fuel_side {
        let q = format!("{p}.fuel_side.{c}");
        o.len(&q, side.len());
        for (k, s) in side.iter().enumerate() {
            o.s(&format!("{q}.{k}"), s);
        }
    }
    o.keys(&format!("{p}.tau_f_inert"), r.tau_f_inert.len());
    for (key, v) in &r.tau_f_inert {
        inert(o, &format!("{p}.tau_f_inert.{key}"), v);
    }
    o.i(&format!("{p}.n_invalid"), r.n_invalid);
}

pub fn mask_cell(o: &mut Flat, p: &str, c: &MaskCell) {
    o.keys(p, 9);
    o.f(&format!("{p}.s"), c.s);
    o.f(&format!("{p}.phi"), c.phi);
    o.lab(&format!("{p}.authority"), c.authority);
    o.lab(&format!("{p}.masked"), c.masked);
    o.of(&format!("{p}.mask_leak"), c.mask_leak);
    o.f(&format!("{p}.c1"), c.c1);
    o.f(&format!("{p}.c0"), c.c0);
    o.i(&format!("{p}.zeros"), c.zeros);
    o.f(&format!("{p}.cyc"), c.cyc);
}

pub fn stats(o: &mut Flat, p: &str, d: &AuthStats) {
    o.keys(p, 4);
    o.i(&format!("{p}.n"), d.n);
    o.f(&format!("{p}.max_cyc"), d.max_cyc);
    o.f(&format!("{p}.max_leak"), d.max_leak);
    o.is(&format!("{p}.zeros"), &d.zeros);
}

pub fn taus4(o: &mut Flat, p: &str, t: (f64, f64, f64, f64)) {
    o.fs(p, &[t.0, t.1, t.2, t.3]);
}

pub fn mask_arm(o: &mut Flat, p: &str, a: &MaskArm) {
    o.keys(p, 9);
    taus4(o, &format!("{p}.taus"), a.taus);
    o.b(&format!("{p}.riding4_valid"), a.riding4_valid);
    o.f(&format!("{p}.max_Tt4"), a.max_tt4);
    o.i(&format!("{p}.n_riding"), a.n_riding);
    o.i(&format!("{p}.n_sampled"), a.n_sampled);
    o.i(&format!("{p}.n_interior"), a.n_interior);
    o.keys(&format!("{p}.skipped"), 2);
    o.i(&format!("{p}.skipped.switch"), a.skipped.0);
    o.i(&format!("{p}.skipped.regime"), a.skipped.1);
    o.keys(&format!("{p}.by_authority"), a.by_authority.len());
    for (k, d) in &a.by_authority {
        stats(o, &format!("{p}.by_authority.{}", k.map_or("None", |a| a.as_str())), d);
    }
    o.len(&format!("{p}.cells"), a.cells.len());
    for (k, c) in a.cells.iter().enumerate() {
        mask_cell(o, &format!("{p}.cells.{k}"), c);
    }
}

pub fn mask(o: &mut Flat, p: &str, r: &AuthorityMask) {
    o.keys(p, 16);
    o.s(&format!("{p}.coord"), r.coord);
    o.len(&format!("{p}.clocks"), r.clocks.len());
    for (k, t) in r.clocks.iter().enumerate() {
        taus4(o, &format!("{p}.clocks.{k}"), *t);
    }
    o.f(&format!("{p}.phi_lim"), r.phi_lim);
    o.of(&format!("{p}.phi_air"), r.phi_air);
    o.f(&format!("{p}.ds"), r.ds);
    o.len(&format!("{p}.arms"), r.arms.len());
    for (k, a) in r.arms.iter().enumerate() {
        mask_arm(o, &format!("{p}.arms.{k}"), a);
    }
    o.b(&format!("{p}.vacuous"), r.vacuous);
    o.i(&format!("{p}.n_fuel_interior"), r.n_fuel_interior);
    o.i(&format!("{p}.n_gov_interior"), r.n_gov_interior);
    o.b(&format!("{p}.all_differenced"), r.all_differenced);
    o.b(&format!("{p}.ever_two_authorities"), r.ever_two_authorities);
    o.of(&format!("{p}.max_mask_leak"), r.max_mask_leak);
    o.of(&format!("{p}.cyc_fuel_auth"), r.cyc_fuel_auth);
    o.of(&format!("{p}.cyc_gov_auth"), r.cyc_gov_auth);
    o.is(&format!("{p}.zeros_fuel_auth"), &r.zeros_fuel_auth);
    o.is(&format!("{p}.zeros_gov_auth"), &r.zeros_gov_auth);
}

/// The rig after the reader — the probe's `name@rig` dict.
pub fn rig_after(o: &mut Flat, name: &str, m: &ScheduledStatorCore) {
    let p = format!("{name}@rig");
    o.keys(&p, 2);
    o.s(&format!("{p}.lag_coord"), m.fuel.inner.lag_coord.get());
    o.of(&format!("{p}.sm_air"), m.fuel.inner.sm_air.get());
}

// ======================================== from `slice_aj_threshold.rs` (step 3)

//
// The probe's `flat`, mirrored: one `(path, token)` per value, in Python's dict order.


pub fn scan(o: &mut Flat, p: &str, s: &ThresholdScan) {
    o.keys(p, 28);
    o.f(&format!("{p}.tau_f"), s.tau_f);
    o.f(&format!("{p}.tau_gov"), s.tau_gov);
    o.f(&format!("{p}.r"), s.r);
    o.f(&format!("{p}.ds"), s.ds);
    o.f(&format!("{p}.phi_lim"), s.phi_lim);
    o.of(&format!("{p}.phi_air"), s.phi_air);
    o.i(&format!("{p}.npts"), s.npts);
    o.i(&format!("{p}.n_riding4"), s.n_riding4);
    o.f(&format!("{p}.max_Tt4"), s.max_tt4);
    o.f(&format!("{p}.min_phi"), s.min_phi);
    o.b(&format!("{p}.riding4_valid"), s.riding4_valid);
    o.b(&format!("{p}.window_open"), s.window_open);
    o.i(&format!("{p}.n_scored"), s.n_scored);
    o.i(&format!("{p}.n_slope_excluded"), s.n_slope_excluded);
    o.i(&format!("{p}.n_fuel"), s.n_fuel);
    o.i(&format!("{p}.n_gov"), s.n_gov);
    o.i(&format!("{p}.n_pred_fuel"), s.n_pred_fuel);
    o.i(&format!("{p}.n_agree"), s.n_agree);
    o.of(&format!("{p}.h"), s.h);
    o.of(&format!("{p}.tau_hat_min"), s.tau_hat_min);
    o.of(&format!("{p}.s_bind"), s.s_bind);
    o.of(&format!("{p}.tau_eff_bind"), s.tau_eff_bind);
    o.fs(&format!("{p}.kappa"), &s.kappa);
    o.b(&format!("{p}.kappa_pure"), s.kappa_pure);
    o.of(&format!("{p}.gap_bind"), s.gap_bind);
    o.of(&format!("{p}.ratio_bind"), s.ratio_bind);
    o.of(&format!("{p}.slope_f_bind"), s.slope_f_bind);
    o.of(&format!("{p}.slope_r_bind"), s.slope_r_bind);
}

pub fn bisect(o: &mut Flat, p: &str, b: &Bisect) {
    match b {
        Bisect::VoidEnd { void, lo, hi, at_lo, at_hi } => {
            o.keys(p, 5);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.lo"), *lo);
            o.f(&format!("{p}.hi"), *hi);
            scan(o, &format!("{p}.at_lo"), at_lo);
            scan(o, &format!("{p}.at_hi"), at_hi);
        }
        Bisect::VoidStraddle { void, lo, hi, at_lo, at_hi, below, above } => {
            o.keys(p, 7);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.lo"), *lo);
            o.f(&format!("{p}.hi"), *hi);
            scan(o, &format!("{p}.at_lo"), at_lo);
            scan(o, &format!("{p}.at_hi"), at_hi);
            o.b(&format!("{p}.below"), *below);
            o.b(&format!("{p}.above"), *above);
        }
        Bisect::VoidMid { void, lo, hi } => {
            o.keys(p, 3);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.lo"), *lo);
            o.f(&format!("{p}.hi"), *hi);
        }
        Bisect::Ok(b) => {
            o.keys(p, 6);
            o.f(&format!("{p}.lo"), b.lo);
            o.f(&format!("{p}.hi"), b.hi);
            o.f(&format!("{p}.mid"), b.mid);
            o.f(&format!("{p}.width"), b.width);
            o.none(&format!("{p}.void"));
            scan(o, &format!("{p}.at_hi"), &b.at_hi);
        }
    }
}

pub fn row_void(o: &mut Flat, p: &str, x: &RowVoid) {
    o.keys(p, 8);
    o.f(&format!("{p}.r"), x.r);
    o.b(&format!("{p}.ok"), false);
    o.s(&format!("{p}.void"), &x.void);
    o.fs(&format!("{p}.kappa"), &x.kappa);
    o.b(&format!("{p}.kappa_pure"), x.kappa_pure);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.b(&format!("{p}.window_open"), x.window_open);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
}

pub fn row_ok(o: &mut Flat, p: &str, x: &RowOk) {
    o.keys(p, 32);
    o.f(&format!("{p}.r"), x.r);
    o.b(&format!("{p}.ok"), true);
    o.none(&format!("{p}.void"));
    o.f(&format!("{p}.tau_star"), x.tau_star);
    o.f(&format!("{p}.lo"), x.lo);
    o.f(&format!("{p}.hi"), x.hi);
    o.f(&format!("{p}.width"), x.width);
    o.f(&format!("{p}.tau_star_eff"), x.tau_star_eff);
    o.f(&format!("{p}.fixed"), x.fixed);
    o.of(&format!("{p}.fwd"), x.fwd);
    o.f(&format!("{p}.tau_ref"), x.tau_ref);
    o.of(&format!("{p}.tau_hat_min_ref"), x.tau_hat_min_ref);
    o.b(&format!("{p}.fixed_in_bracket"), x.fixed_in_bracket);
    o.b(&format!("{p}.fwd_in_bracket"), x.fwd_in_bracket);
    o.b(&format!("{p}.fixed_early"), x.fixed_early);
    o.b(&format!("{p}.fwd_early"), x.fwd_early);
    o.f(&format!("{p}.err_fixed"), x.err_fixed);
    o.of(&format!("{p}.err_fwd"), x.err_fwd);
    o.f(&format!("{p}.dist_ref"), x.dist_ref);
    o.of(&format!("{p}.s_bind"), x.s_bind);
    o.of(&format!("{p}.gap_bind"), x.gap_bind);
    o.of(&format!("{p}.ratio_bind"), x.ratio_bind);
    o.of(&format!("{p}.tau_eff_bind"), x.tau_eff_bind);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.of(&format!("{p}.agreement"), x.agreement);
    o.fs(&format!("{p}.kappa"), &x.kappa);
    o.b(&format!("{p}.kappa_pure"), x.kappa_pure);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
    o.b(&format!("{p}.window_open"), x.window_open);
    o.ob(&format!("{p}.ds_stable"), x.ds_stable);
    o.of(&format!("{p}.tau_star_fine"), x.tau_star_fine);
}

pub fn law(o: &mut Flat, p: &str, r: &ThresholdLaw) {
    o.keys(p, 25);
    o.fs(&format!("{p}.rs"), &r.rs);
    o.f(&format!("{p}.tau_gov"), r.tau_gov);
    o.f(&format!("{p}.tau_ref"), r.tau_ref);
    o.f(&format!("{p}.phi_lim"), r.phi_lim);
    o.of(&format!("{p}.phi_air"), r.phi_air);
    o.pair(&format!("{p}.bracket"), r.bracket.0, r.bracket.1);
    o.f(&format!("{p}.ds"), r.ds);
    o.of(&format!("{p}.ds_fine"), r.ds_fine);
    o.len(&format!("{p}.rows"), r.rows.len());
    for (k, x) in r.rows.iter().enumerate() {
        match x {
            ThresholdRow::Void(v) => row_void(o, &format!("{p}.rows.{k}"), v),
            ThresholdRow::Ok(v) => row_ok(o, &format!("{p}.rows.{k}"), v),
        }
    }
    o.i(&format!("{p}.n_void"), r.n_void);
    o.fs(&format!("{p}.p1_fixed_in"), &r.p1_fixed_in);
    o.fs(&format!("{p}.p1_fixed_out"), &r.p1_fixed_out);
    o.fs(&format!("{p}.p1_fwd_in"), &r.p1_fwd_in);
    o.fs(&format!("{p}.p1_fwd_out"), &r.p1_fwd_out);
    o.fs(&format!("{p}.p2_fixed_early"), &r.p2_fixed_early);
    o.fs(&format!("{p}.p2_fwd_early"), &r.p2_fwd_early);
    o.b(&format!("{p}.p2_all_early"), r.p2_all_early);
    o.b(&format!("{p}.p3_fwd_never_better"), r.p3_fwd_never_better);
    o.fs(&format!("{p}.kappa_seen"), &r.kappa_seen);
    o.b(&format!("{p}.all_kappa_pure"), r.all_kappa_pure);
    o.fs(&format!("{p}.ds_stable"), &r.ds_stable);
    o.fs(&format!("{p}.ds_unstable"), &r.ds_unstable);
    o.fs(&format!("{p}.ds_unrun"), &r.ds_unrun);
    o.pairs(&format!("{p}.thresholds"), &r.thresholds);
    o.b(&format!("{p}.monotone_in_r"), r.monotone_in_r);
}

pub fn ref_row(o: &mut Flat, p: &str, x: &RefRow) {
    o.keys(p, 12);
    o.f(&format!("{p}.tau_ref"), x.tau_ref);
    o.of(&format!("{p}.fwd"), x.fwd);
    o.b(&format!("{p}.ref_above"), x.ref_above);
    o.b(&format!("{p}.early"), x.early);
    o.of(&format!("{p}.err"), x.err);
    o.f(&format!("{p}.dist"), x.dist);
    o.of(&format!("{p}.s_bind"), x.s_bind);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.b(&format!("{p}.window_open"), x.window_open);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
    o.b(&format!("{p}.kappa_pure"), x.kappa_pure);
}

pub fn ref_full(o: &mut Flat, p: &str, r: &RefFull) {
    o.keys(p, 14);
    o.f(&format!("{p}.r"), r.r);
    o.f(&format!("{p}.tau_star"), r.tau_star);
    o.pair(&format!("{p}.bracket"), r.bracket[0], r.bracket[1]);
    o.f(&format!("{p}.width"), r.width);
    o.fs(&format!("{p}.tau_refs"), &r.tau_refs);
    o.len(&format!("{p}.rows"), r.rows.len());
    for (k, x) in r.rows.iter().enumerate() {
        ref_row(o, &format!("{p}.rows.{k}"), x);
    }
    o.none(&format!("{p}.void"));
    o.i(&format!("{p}.n_live"), r.n_live);
    o.b(&format!("{p}.sign_follows_reference"), r.sign_follows_reference);
    o.i(&format!("{p}.n_agree"), r.n_agree);
    o.len(&format!("{p}.crossing"), r.crossing.len());
    for (k, (t, e)) in r.crossing.iter().enumerate() {
        o.len(&format!("{p}.crossing.{k}"), 2);
        o.f(&format!("{p}.crossing.{k}.0"), *t);
        o.b(&format!("{p}.crossing.{k}.1"), *e);
    }
    o.ob(&format!("{p}.grows_above"), r.grows_above);
    o.ob(&format!("{p}.grows_below"), r.grows_below);
    o.len(&format!("{p}.s_binds"), r.s_binds.len());
    for (k, (t, s)) in r.s_binds.iter().enumerate() {
        o.len(&format!("{p}.s_binds.{k}"), 2);
        o.f(&format!("{p}.s_binds.{k}.0"), *t);
        o.of(&format!("{p}.s_binds.{k}.1"), *s);
    }
}

pub fn reference(o: &mut Flat, p: &str, r: &ThresholdReference) {
    match r {
        ThresholdReference::Void { r, void } => {
            o.keys(p, 3);
            o.f(&format!("{p}.r"), *r);
            o.s(&format!("{p}.void"), void);
            o.len(&format!("{p}.rows"), 0);
        }
        ThresholdReference::Full(f) => ref_full(o, p, f),
    }
}

pub fn at_ok(o: &mut Flat, p: &str, x: &AtOk) {
    o.keys(p, 15);
    o.f(&format!("{p}.phi_lim"), x.phi_lim);
    o.f(&format!("{p}.tau_gov"), x.tau_gov);
    o.b(&format!("{p}.ok"), true);
    o.none(&format!("{p}.void"));
    o.f(&format!("{p}.tau_star"), x.tau_star);
    o.f(&format!("{p}.width"), x.width);
    o.of(&format!("{p}.gap_bind"), x.gap_bind);
    o.of(&format!("{p}.ratio_bind"), x.ratio_bind);
    o.of(&format!("{p}.s_bind"), x.s_bind);
    o.i(&format!("{p}.n_riding4"), x.n_riding4);
    o.i(&format!("{p}.n_fuel"), x.n_fuel);
    o.b(&format!("{p}.riding4_valid"), x.riding4_valid);
    o.of(&format!("{p}.slope_f"), x.slope_f);
    o.of(&format!("{p}.slope_r"), x.slope_r);
    o.of(&format!("{p}.kappa"), x.kappa);
}

pub fn ats(o: &mut Flat, p: &str, xs: &[TermsAt]) {
    o.len(p, xs.len());
    for (k, x) in xs.iter().enumerate() {
        let q = format!("{p}.{k}");
        match x {
            TermsAt::Void { phi_lim, tau_gov, void } => {
                o.keys(&q, 4);
                o.f(&format!("{q}.phi_lim"), *phi_lim);
                o.f(&format!("{q}.tau_gov"), *tau_gov);
                o.b(&format!("{q}.ok"), false);
                o.s(&format!("{q}.void"), void);
            }
            TermsAt::Ok(a) => at_ok(o, &q, a),
        }
    }
}

pub fn p4(o: &mut Flat, p: &str, x: &Option<P4>) {
    let Some(x) = x else {
        o.none(p);
        return;
    };
    o.keys(p, 9);
    o.f(&format!("{p}.measured"), x.measured);
    o.of(&format!("{p}.predicted"), x.predicted);
    o.b(&format!("{p}.rises"), x.rises);
    o.of(&format!("{p}.rel_err"), x.rel_err);
    o.of(&format!("{p}.transfer"), x.transfer);
    o.len(&format!("{p}.secants"), x.secants.len());
    for (k, s) in x.secants.iter().enumerate() {
        let q = format!("{p}.secants.{k}");
        o.keys(&q, 4);
        o.f(&format!("{q}.lo"), s.lo);
        o.f(&format!("{q}.hi"), s.hi);
        o.f(&format!("{q}.measured"), s.measured);
        o.of(&format!("{q}.predicted"), s.predicted);
    }
    o.ofs(&format!("{p}.kappa"), &x.kappa);
    o.ofs(&format!("{p}.ratio_bind"), &x.ratio_bind);
    o.pairs(&format!("{p}.taus"), &x.taus);
}

pub fn p5(o: &mut Flat, p: &str, x: &Option<P5>) {
    let Some(x) = x else {
        o.none(p);
        return;
    };
    o.keys(p, 15);
    o.of(&format!("{p}.d_gap"), x.d_gap);
    o.of(&format!("{p}.d_lag"), x.d_lag);
    o.of(&format!("{p}.d_slope_f"), x.d_slope_f);
    o.of(&format!("{p}.d_ratio"), x.d_ratio);
    o.b(&format!("{p}.rises"), x.rises);
    o.b(&format!("{p}.separates"), x.separates);
    o.b(&format!("{p}.v7_withdrawn"), x.v7_withdrawn);
    o.b(&format!("{p}.gap_falls"), x.gap_falls);
    o.b(&format!("{p}.ratio_rises"), x.ratio_rises);
    o.b(&format!("{p}.monotone"), x.monotone);
    o.pairs(&format!("{p}.taus"), &x.taus);
    o.ofs(&format!("{p}.gaps"), &x.gaps);
    o.ofs(&format!("{p}.ratios"), &x.ratios);
    o.ofs(&format!("{p}.slope_f"), &x.slope_f);
    o.ofs(&format!("{p}.slope_r"), &x.slope_r);
}

pub fn terms(o: &mut Flat, p: &str, r: &ThresholdTerms) {
    o.keys(p, 12);
    o.f(&format!("{p}.r"), r.r);
    o.f(&format!("{p}.phi_lim"), r.phi_lim);
    o.fs(&format!("{p}.phi_lims"), &r.phi_lims);
    o.of(&format!("{p}.phi_air"), r.phi_air);
    o.f(&format!("{p}.ds"), r.ds);
    o.fs(&format!("{p}.tau_govs"), &r.tau_govs);
    o.pair(&format!("{p}.bracket"), r.bracket.0, r.bracket.1);
    ats(o, &format!("{p}.govs"), &r.govs);
    ats(o, &format!("{p}.walls"), &r.walls);
    o.i(&format!("{p}.n_void"), r.n_void);
    p4(o, &format!("{p}.p4"), &r.p4);
    p5(o, &format!("{p}.p5"), &r.p5);
}

// ======================================== from `slice_aj_corrector.rs` (step 4)

pub fn read(o: &mut Flat, p: &str, r: &CorrectorRead) {
    o.keys(p, 16);
    o.f(&format!("{p}.tau_f"), r.tau_f);
    o.of(&format!("{p}.F"), r.f);
    o.of(&format!("{p}.g"), r.g);
    o.of(&format!("{p}.h"), r.h);
    o.of(&format!("{p}.kappa"), r.kappa);
    o.b(&format!("{p}.kappa_pure"), r.kappa_pure);
    o.b(&format!("{p}.exact"), r.exact);
    o.of(&format!("{p}.identity_pred"), r.identity_pred);
    o.ob(&format!("{p}.below_root"), r.below_root);
    o.of(&format!("{p}.s_bind"), r.s_bind);
    o.i(&format!("{p}.n_fuel"), r.n_fuel);
    o.i(&format!("{p}.n_scored"), r.n_scored);
    o.b(&format!("{p}.window_open"), r.window_open);
    o.b(&format!("{p}.riding4_valid"), r.riding4_valid);
    o.of(&format!("{p}.tau_hat_min"), r.tau_hat_min);
    scan(o, &format!("{p}.scan"), &r.scan);
}

pub fn step(o: &mut Flat, p: &str, s: &CorrectorStep) {
    o.keys(p, 5);
    match s {
        CorrectorStep::Void { c, forward, void } => {
            o.none(&format!("{p}.tau_hat"));
            o.f(&format!("{p}.c"), *c);
            o.of(&format!("{p}.forward"), *forward);
            o.none(&format!("{p}.correction"));
            o.s(&format!("{p}.void"), void);
        }
        CorrectorStep::Ok { tau_hat, c, forward, correction } => {
            o.f(&format!("{p}.tau_hat"), *tau_hat);
            o.f(&format!("{p}.c"), *c);
            o.none(&format!("{p}.void"));
            o.f(&format!("{p}.forward"), *forward);
            o.f(&format!("{p}.correction"), *correction);
        }
    }
}

pub fn change(o: &mut Flat, p: &str, c: &Change) {
    o.keys(p, 11);
    o.f(&format!("{p}.tau_lo"), c.tau_lo);
    o.f(&format!("{p}.tau_hi"), c.tau_hi);
    o.f(&format!("{p}.g_lo"), c.g_lo);
    o.f(&format!("{p}.g_hi"), c.g_hi);
    o.of(&format!("{p}.s_lo"), c.s_lo);
    o.of(&format!("{p}.s_hi"), c.s_hi);
    o.b(&format!("{p}.argmin_moved"), c.argmin_moved);
    o.f(&format!("{p}.smallest_g"), c.smallest_g);
    o.f(&format!("{p}.step"), c.step);
    o.f(&format!("{p}.ratio"), c.ratio);
    o.of(&format!("{p}.jump_in_F"), c.jump_in_f);
}

pub fn shape(o: &mut Flat, p: &str, s: &ResidualShape) {
    o.keys(p, 9);
    o.f(&format!("{p}.lo"), s.lo);
    o.f(&format!("{p}.hi"), s.hi);
    o.i(&format!("{p}.n"), s.n);
    o.f(&format!("{p}.step"), s.step);
    o.len(&format!("{p}.points"), s.points.len());
    for (k, x) in s.points.iter().enumerate() {
        read(o, &format!("{p}.points.{k}"), x);
    }
    o.len(&format!("{p}.changes"), s.changes.len());
    for (k, x) in s.changes.iter().enumerate() {
        change(o, &format!("{p}.changes.{k}"), x);
    }
    o.i(&format!("{p}.n_changes"), s.n_changes);
    o.i(&format!("{p}.n_kappa_impure"), s.n_kappa_impure);
    o.i(&format!("{p}.n_argmin_switches"), s.n_argmin_switches);
}

pub fn trace_step(o: &mut Flat, p: &str, x: &SecantStep) {
    o.keys(p, 5);
    o.f(&format!("{p}.tau"), x.tau);
    o.f(&format!("{p}.F"), x.f);
    o.f(&format!("{p}.g"), x.g);
    o.of(&format!("{p}.s_bind"), x.s_bind);
    o.b(&format!("{p}.clamped"), x.clamped);
}

pub fn secant(o: &mut Flat, p: &str, s: &CorrectorSecant) {
    o.keys(p, 11);
    o.f(&format!("{p}.t0"), s.t0);
    o.f(&format!("{p}.t1"), s.t1);
    o.i(&format!("{p}.cap"), s.cap);
    o.len(&format!("{p}.trace"), s.trace.len());
    for (k, x) in s.trace.iter().enumerate() {
        trace_step(o, &format!("{p}.trace.{k}"), x);
    }
    o.i(&format!("{p}.clamps"), s.clamps);
    o.os(&format!("{p}.abort"), s.abort.as_deref());
    o.i(&format!("{p}.marches"), s.marches);
    o.of(&format!("{p}.tau"), s.tau);
    o.of(&format!("{p}.final_g"), s.final_g);
    o.len(&format!("{p}.marches_vs_bisect"), 2);
    o.i(&format!("{p}.marches_vs_bisect.0"), s.marches_vs_bisect.0);
    o.i(&format!("{p}.marches_vs_bisect.1"), s.marches_vs_bisect.1);
    o.b(&format!("{p}.converged"), s.converged);
}

// ======================================== from `slice_aj_staircase.rs` (step 4)

pub fn edge(o: &mut Flat, p: &str, e: &EdgeRead) {
    o.keys(p, 19);
    o.f(&format!("{p}.tau_f"), e.tau_f);
    o.f(&format!("{p}.r"), e.r);
    o.f(&format!("{p}.ds"), e.ds);
    o.of(&format!("{p}.h"), e.h);
    o.of(&format!("{p}.kappa"), e.kappa);
    o.b(&format!("{p}.kappa_pure"), e.kappa_pure);
    o.of(&format!("{p}.F"), e.f);
    o.of(&format!("{p}.g"), e.g);
    o.len(&format!("{p}.summands"), e.summands.len());
    for (k, &(s, v)) in e.summands.iter().enumerate() {
        o.len(&format!("{p}.summands.{k}"), 2);
        o.f(&format!("{p}.summands.{k}.0"), s);
        o.f(&format!("{p}.summands.{k}.1"), v);
    }
    o.of(&format!("{p}.s_bind"), e.s_bind);
    o.of(&format!("{p}.edge"), e.edge);
    o.b(&format!("{p}.at_edge"), e.at_edge);
    o.i(&format!("{p}.n_ride"), e.n_ride);
    o.i(&format!("{p}.n_scored"), e.n_scored);
    o.i(&format!("{p}.n_slope_excluded"), e.n_slope_excluded);
    o.b(&format!("{p}.window_open"), e.window_open);
    o.b(&format!("{p}.riding4_valid"), e.riding4_valid);
    o.b(&format!("{p}.edge_on_grid"), e.edge_on_grid);
    o.oi(&format!("{p}.edge_index"), e.edge_index);
}

pub fn classified(o: &mut Flat, p: &str, c: &Classified) {
    o.keys(p, 17);
    o.f(&format!("{p}.tau_lo"), c.tau_lo);
    o.f(&format!("{p}.tau_hi"), c.tau_hi);
    o.fs(&format!("{p}.entered"), &c.entered);
    o.fs(&format!("{p}.left"), &c.left);
    o.b(&format!("{p}.set_changed"), c.set_changed);
    o.b(&format!("{p}.argmin_moved"), c.argmin_moved);
    o.b(&format!("{p}.edge_moved"), c.edge_moved);
    o.of(&format!("{p}.h_lo"), c.h_lo);
    o.of(&format!("{p}.h_hi"), c.h_hi);
    o.of(&format!("{p}.h_common_lo"), c.h_common_lo);
    o.of(&format!("{p}.h_common_hi"), c.h_common_hi);
    o.of(&format!("{p}.d_full"), c.d_full);
    o.of(&format!("{p}.d_smooth"), c.d_smooth);
    o.of(&format!("{p}.d_membership"), c.d_membership);
    o.b(&format!("{p}.sign_change"), c.sign_change);
    o.b(&format!("{p}.sign_change_common"), c.sign_change_common);
    o.kind(&format!("{p}.kind"), c.kind);
}

pub fn ladder(o: &mut Flat, p: &str, s: &StaircaseScan) {
    o.keys(p, 22);
    o.f(&format!("{p}.lo"), s.lo);
    o.f(&format!("{p}.hi"), s.hi);
    o.i(&format!("{p}.n"), s.n);
    o.len(&format!("{p}.points"), s.points.len());
    for (k, x) in s.points.iter().enumerate() {
        edge(o, &format!("{p}.points.{k}"), x);
    }
    o.len(&format!("{p}.pairs"), s.pairs.len());
    for (k, x) in s.pairs.iter().enumerate() {
        classified(o, &format!("{p}.pairs.{k}"), x);
    }
    o.len(&format!("{p}.changes"), s.changes.len());
    for (k, x) in s.changes.iter().enumerate() {
        classified(o, &format!("{p}.changes.{k}"), x);
    }
    o.i(&format!("{p}.n_at_edge"), s.n_at_edge);
    o.i(&format!("{p}.n_points"), s.n_points);
    o.b(&format!("{p}.all_at_edge"), s.all_at_edge);
    o.i(&format!("{p}.n_sign_changes"), s.n_sign_changes);
    o.i(&format!("{p}.n_crossings"), s.n_crossings);
    o.i(&format!("{p}.n_jumps"), s.n_jumps);
    o.b(&format!("{p}.exact_zero_when_set_equal"), s.exact_zero_when_set_equal);
    o.b(&format!("{p}.nonzero_when_set_differs"), s.nonzero_when_set_differs);
    o.i(&format!("{p}.n_argmin_only"), s.n_argmin_only);
    o.i(&format!("{p}.n_set_only"), s.n_set_only);
    o.i(&format!("{p}.n_edge_moves"), s.n_edge_moves);
    o.b(&format!("{p}.all_on_grid"), s.all_on_grid);
    o.b(&format!("{p}.edge_monotone"), s.edge_monotone);
    o.len(&format!("{p}.edge_indices"), s.edge_indices.len());
    for (k, &x) in s.edge_indices.iter().enumerate() {
        o.oi(&format!("{p}.edge_indices.{k}"), x);
    }
    o.b(&format!("{p}.all_open"), s.all_open);
    o.b(&format!("{p}.all_kappa_pure"), s.all_kappa_pure);
}

pub fn lattice(o: &mut Flat, p: &str, c: &LatticeCount) {
    o.keys(p, 14);
    o.f(&format!("{p}.lo"), c.lo);
    o.f(&format!("{p}.hi"), c.hi);
    o.f(&format!("{p}.ds"), c.ds);
    o.f(&format!("{p}.r"), c.r);
    o.of(&format!("{p}.edge_lo"), c.edge_lo);
    o.of(&format!("{p}.edge_hi"), c.edge_hi);
    o.oi(&format!("{p}.index_lo"), c.index_lo);
    o.oi(&format!("{p}.index_hi"), c.index_hi);
    o.oi(&format!("{p}.n_jumps"), c.n_jumps);
    o.of(&format!("{p}.ds_star"), c.ds_star);
    o.of(&format!("{p}.slope_s_star"), c.slope_s_star);
    o.b(&format!("{p}.on_grid"), c.on_grid);
    o.len(&format!("{p}.at_edge"), 2);
    o.b(&format!("{p}.at_edge.0"), c.at_edge.0);
    o.b(&format!("{p}.at_edge.1"), c.at_edge.1);
    o.os(&format!("{p}.void"), c.void.as_deref());
}

pub fn number(o: &mut Flat, p: &str, n: &StaircaseNumber) {
    match n {
        StaircaseNumber::V2 { void, tau_lo, tau_hi } => {
            o.keys(p, 5);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.tau_lo"), *tau_lo);
            o.f(&format!("{p}.tau_hi"), *tau_hi);
            o.none(&format!("{p}.rise"));
            o.none(&format!("{p}.dg_dtau"));
        }
        StaircaseNumber::V5 { void, tau_lo, tau_hi, edge_moved } => {
            o.keys(p, 6);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.tau_lo"), *tau_lo);
            o.f(&format!("{p}.tau_hi"), *tau_hi);
            o.b(&format!("{p}.edge_moved"), *edge_moved);
            o.none(&format!("{p}.rise"));
            o.none(&format!("{p}.dg_dtau"));
        }
        StaircaseNumber::Ok(x) => {
            o.keys(p, 15);
            o.f(&format!("{p}.tau_lo"), x.tau_lo);
            o.f(&format!("{p}.tau_hi"), x.tau_hi);
            o.f(&format!("{p}.ds"), x.ds);
            o.f(&format!("{p}.r"), x.r);
            o.none(&format!("{p}.void"));
            o.f(&format!("{p}.rise"), x.rise);
            o.f(&format!("{p}.dg_dtau"), x.dg_dtau);
            o.f(&format!("{p}.dtau"), x.dtau);
            o.of(&format!("{p}.spacing"), x.spacing);
            o.of(&format!("{p}.tread"), x.tread);
            o.of(&format!("{p}.lam"), x.lam);
            o.fs(&format!("{p}.entered"), &x.entered);
            o.fs(&format!("{p}.left"), &x.left);
            o.f(&format!("{p}.d_membership"), x.d_membership);
            o.f(&format!("{p}.d_smooth"), x.d_smooth);
        }
    }
}

pub fn root(o: &mut Flat, p: &str, r: &RootClass) {
    match r {
        RootClass::Void { void, r, ds } => {
            o.keys(p, 5);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.r"), *r);
            o.f(&format!("{p}.ds"), *ds);
            o.none(&format!("{p}.kind"));
            o.none(&format!("{p}.root_exists"));
        }
        RootClass::Ok(x) => {
            o.keys(p, 21);
            o.f(&format!("{p}.r"), x.r);
            o.f(&format!("{p}.ds"), x.ds);
            o.none(&format!("{p}.void"));
            o.f(&format!("{p}.lo"), x.lo);
            o.f(&format!("{p}.hi"), x.hi);
            o.f(&format!("{p}.mid"), x.mid);
            o.f(&format!("{p}.width"), x.width);
            o.kind(&format!("{p}.kind"), x.kind);
            o.b(&format!("{p}.root_exists"), x.root_exists);
            o.b(&format!("{p}.set_changed"), x.set_changed);
            o.b(&format!("{p}.argmin_moved"), x.argmin_moved);
            o.b(&format!("{p}.edge_moved"), x.edge_moved);
            o.fs(&format!("{p}.entered"), &x.entered);
            o.fs(&format!("{p}.left"), &x.left);
            o.of(&format!("{p}.d_full"), x.d_full);
            o.of(&format!("{p}.d_smooth"), x.d_smooth);
            o.of(&format!("{p}.d_membership"), x.d_membership);
            o.of(&format!("{p}.h_lo"), x.h_lo);
            o.of(&format!("{p}.h_hi"), x.h_hi);
            o.of(&format!("{p}.h_common_lo"), x.h_common_lo);
            o.of(&format!("{p}.h_common_hi"), x.h_common_hi);
        }
    }
}
