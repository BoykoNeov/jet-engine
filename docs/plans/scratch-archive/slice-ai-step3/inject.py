"""Slice AI step 3 injection sweep. Each injection: apply (exact-string, must match once), run the
three slice-AI test files, record per-test verdicts, restore. Originals are restored from memory
AND re-verified byte-for-byte at the end."""
import subprocess
import sys

ROOT = r"W:\Claude_projects\jet engine"
SC = ROOT + r"\rust\src\state_coordinate.rs"
DC = ROOT + r"\rust\src\demand_coordinate.rs"
LOG = r"W:\temp\claude\slice-ai-step3\inject.log"

INJ = {
    "J1": (SC, [
        ("    let t = &core.fuel.inner;\n    let sm = phi_lim",
         "    reset_coord_counters();\n    let t = &core.fuel.inner;\n    let sm = phi_lim"),
        ("    reset_coord_counters();\n    // `self._with_probe", "    // `self._with_probe")]),
    "J2": (SC, [("gap_med: opt(&gaps, py_upper_median),",
                 "gap_med: opt(&gaps, |xs| { let mut v = xs.to_vec(); "
                 "v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2 - 1] }),")]),
    "J3": (SC, [("let vacuous = if d_max == Some(0.0) {\n        Some(true)",
                 "let vacuous = if d_max == Some(0.0) {\n        None")]),
    "J4": (SC, [("lo, hi, glo, ghi, FORCED_TOL, ILLINOIS_MAXIT)",
                 "lo, hi, glo, ghi, FuelTransientCore::LEG_TOL, ILLINOIS_MAXIT)")]),
    "J5": (SC, [("            if gl.expect(\"just assigned\") < 0.0 {\n                break;\n"
                 "            }\n        }",
                 "            if gl.expect(\"just assigned\") < 0.0 {\n                break;\n"
                 "            }\n            gl = None;\n        }")]),
    "J6": (SC, [("hh *= grow;", "hh *= shrink;")]),
    "J7": (SC, [("*l.borrow_mut() = self.prev_l.take());", "*l.borrow_mut() = None);")]),
    "J8": (SC, [("prev_l: COORD_LOG.with(|l| l.borrow_mut().replace(Vec::new())),",
                 "prev_l: COORD_LOG.with(|l| { let p = l.borrow().clone(); "
                 "if p.is_none() { *l.borrow_mut() = Some(Vec::new()); } p }),")]),
    "J9": (SC, [("    let (traj1, log) = with_probe(|| {\n"
                 "        let _cs = CoordScope::set(t, PHI_REF_INCIDENCE);\n"
                 "        march()\n    });",
                 "    let cs1 = CoordScope::set(t, PHI_REF_INCIDENCE);\n"
                 "    let (traj1, log) = with_probe(&march);\n    drop(cs1);")]),
    "J10": (SC, [("py_min2(x.a_cap, x.p_cap) < x.mf_sched).count();\n    let n_both",
                  "x.a_cap.min(x.p_cap) < x.mf_sched).count();\n    let n_both")]),
    "J11": (SC, [("v.dedup_by(|a, b| a == b);", "v.dedup_by(|a, b| a.to_bits() == b.to_bits());")]),
    "J12": (SC, [("    let traj0 = {\n        let _cs = CoordScope::set(t, PHI_REF_PHI);",
                  "    let traj0 = {\n        let _cs = CoordScope::set(t, PHI_REF_INCIDENCE);")]),
    "J13": (SC, [("let w_i = forced(PHI_REF_INCIDENCE);", "let w_i = forced(PHI_REF_PHI);")]),
    "J14": (SC, [("let w_s = m.fuel.try_surge_fuel(flight, a, h, ms, surge).unwrap_or_else(boom);",
                  "let w_s = forced(PHI_REF_PHI);")]),
    "J15": (SC, [("let p_phi = phi_cap(ft, flight, a, h, mf_sched, surge, Some(PHI_REF_PHI))?;",
                  "let p_phi = phi_cap(ft, flight, a, h, mf_sched, surge, None)?;")]),
    "J16": (DC, [("(self.core.triple_hooks.with_coord)(self.core, self.prev);",
                  "self.core.lag_coord.set(self.prev);")]),
}

orig = {p: open(p, "rb").read() for p in (SC, DC)}
only = sys.argv[1:] or list(INJ)
out = open(LOG, "a", encoding="utf-8")
try:
    for name in only:
        path, reps = INJ[name]
        s = orig[path].decode("utf-8")
        for old, new in reps:
            assert s.count(old) == 1, (name, old[:60], s.count(old))
            s = s.replace(old, new)
        open(path, "wb").write(s.encode("utf-8"))
        r = subprocess.run(
            ["cargo", "test", "--release", "--manifest-path", ROOT + r"\rust\Cargo.toml",
             "--no-fail-fast", "--test", "slice_ai_march", "--test", "slice_ai_scan", "--test", "slice_ai_cells"],
            capture_output=True, text=True, encoding="utf-8", errors="replace")
        text = r.stdout + r.stderr
        if "error[" in text or "could not compile" in text:
            verdict = "COMPILE-ERROR"
            detail = [l for l in text.splitlines() if l.startswith("error")][:3]
        else:
            failed = [l.split()[1] for l in text.splitlines()
                      if l.startswith("test ") and l.rstrip().endswith("FAILED")]
            verdict = "KILLED" if failed else "SURVIVED"
            detail = failed
        msgs = [l for l in text.splitlines() if "panicked" in l or l.strip().startswith(("left:", "right:")) or "assertion" in l][:12]
        line = "%s %s %s\n    %s" % (name, verdict, detail, "\n    ".join(msgs))
        print(line, flush=True)
        out.write(line + "\n")
        out.flush()
        open(path, "wb").write(orig[path])
finally:
    for p, b in orig.items():
        open(p, "wb").write(b)
    for p, b in orig.items():
        assert open(p, "rb").read() == b, p
    out.write("RESTORED-VERIFIED\n")
    out.close()
