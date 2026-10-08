"""Slice AI step 4 injection sweep. Each injection: apply (exact-string, must match once), run the
slice-AI split + cells test files with --no-fail-fast, record per-test verdicts and panic
messages, restore. Originals restored from memory AND re-verified byte-for-byte at the end."""
import subprocess
import sys

ROOT = r"W:\Claude_projects\jet engine"
SW = ROOT + r"\rust\src\split_wall.rs"
DC = ROOT + r"\rust\src\demand_coordinate.rs"
LOG = r"W:\temp\claude\slice-ai-step4\inject.log"

INJ = {
    "K1": (DC, [("    m.fuel.inner.lag_coord.set(coord);\n    m.fuel.inner.ref_law.set(ref_law);",
                 "    std::mem::forget(CoordScope::set(&m.fuel.inner, coord));\n"
                 "    m.fuel.inner.ref_law.set(ref_law);")]),
    "K2": (SW, [("    fn drop(&mut self) {\n        self.cell.set(self.prev);\n    }",
                 "    fn drop(&mut self) {\n        let _ = self.prev;\n    }")]),
    "K3": (SW, [("let sm_air = phi_air.map(|pa| pa / ps - 1.0);",
                 "let sm_air = phi_air.map(|pa| pa * (1.0 / ps) - 1.0);")]),
    "K4": (SW, [("arrested: max_t <= tt4_lo * (1.0 + 1e-9),",
                 "arrested: max_t < tt4_lo * (1.0 + 1e-9),")]),
    "K5": (SW, [("valve_moved: st.iter().filter(|x| (x.0 - b0).abs() > tol).count(),",
                 "valve_moved: st.iter().filter(|x| (x.0 - b0).abs() >= tol).count(),")]),
    "K6": (SW, [("b_max_hit: st.iter().any(|x| x.0 >= b_max * (1.0 - 1e-12)),",
                 "b_max_hit: st.iter().any(|x| x.0 > b_max * (1.0 - 1e-12)),")]),
    "K7": (SW, [("cyc_fwd: gg.f_q * gg.c_v * gg.v_f,", "cyc_fwd: gg.f_q * (gg.c_v * gg.v_f),")]),
    "K8": (SW, [("let rate = 1.0 / tt.0 + 1.0 / tt.1 + 1.0 / tt.2 + 1.0 / tt.3;\n            cells",
                 "let rate = 1.0 / tt.3 + 1.0 / tt.2 + 1.0 / tt.1 + 1.0 / tt.0;\n            cells")]),
    "K9": (SW, [("pub const GAINS_EVERY: usize = 5;", "pub const GAINS_EVERY: usize = 4;")]),
    "K10": (SW, [("    v.sort_by_key(|x| x.map(|a| a.as_str()));\n    v\n}", "    v\n}")]),
    "K11": (SW, [("        let mut zeros: Vec<usize> = cells.iter().map(|c| c.zeros).collect();",
                  "        auth.sort_by_key(|x| x.0.map(|a| a.as_str()));\n"
                  "        let mut zeros: Vec<usize> = cells.iter().map(|c| c.zeros).collect();")]),
    "K13": (SW, [("        \"sched\", None)\n}", "        \"applied\", None)\n}")]),
    "K14": (SW, [("                _ => (w, Some(phi_air_hi)),", "                _ => (w, None),")]),
    "K16": (SW, [("phi_air: bl.map(|l| l.phi_lim).or(stator),", "phi_air: stator.or(bl.map(|l| l.phi_lim)),")]),
    "K17": (SW, [("                let _sh = ShareScope::set(&m, \"max\");\n"
                  "                (m.triple_hooks().quad_gains_at)",
                  "                (m.triple_hooks().quad_gains_at)")]),
    "K19": (SW, [(".filter(|c| c.masked == Some(Authority::Gov))",
                  ".filter(|c| c.masked == Some(Authority::Fuel))")]),
}

orig = {p: open(p, "rb").read() for p in (SW, DC)}
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
             "--no-fail-fast", "--test", "slice_ai_split", "--test", "slice_ai_cells"],
            capture_output=True, text=True, encoding="utf-8", errors="replace")
        text = r.stdout + r.stderr
        if "error[" in text or "could not compile" in text:
            verdict = "COMPILE-ERROR"
            detail = [l for l in text.splitlines() if l.startswith("error")][:3]
            msgs = []
        else:
            failed = [l.split()[1] for l in text.splitlines()
                      if l.startswith("test ") and l.rstrip().endswith("FAILED")]
            verdict = "KILLED" if failed else "SURVIVED"
            detail = failed
            lines = text.splitlines()
            msgs = []
            for i, l in enumerate(lines):
                if "panicked at" in l:
                    msgs.append(l.strip()[:160])
                    if i + 1 < len(lines):
                        msgs.append("  " + lines[i + 1].strip()[:200])
            msgs = msgs[:16]
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
