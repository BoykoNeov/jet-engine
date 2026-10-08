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

