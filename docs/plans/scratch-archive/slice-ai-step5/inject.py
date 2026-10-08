"""Slice AI step 5 injection sweep. Each injection: apply (exact string, must match once), run
rung79 + rung80 with --no-fail-fast, record which tests fail and their first panic lines, restore.
Originals are restored from memory AND re-verified byte-for-byte at the end."""
import subprocess
import sys

ROOT = r"W:\Claude_projects\jet engine"
SC = ROOT + r"\rust\src\state_coordinate.rs"
SW = ROOT + r"\rust\src\split_wall.rs"
LB = ROOT + r"\rust\src\limited_bleed.rs"
T79 = ROOT + r"\rust\tests\rung79.rs"
LOG = r"W:\temp\claude\slice-ai-step5\inject.log"

INJ = {
    "N1": (T79, [("    let _sb = MarchedBleed::set(&m.fuel.inner, q);\n"
                  "    let _sv = MarchedStator::set(&m.fuel.inner, v);\n"
                  "    let g = |x: f64|",
                  "    let _ = (MarchedBleed::set_opt, MarchedStator::set_opt);\n"
                  "    let g = |x: f64|")]),
    "N2": (SC, [("pub const R79: LeverHooks = LeverHooks {\n    at_lever: r79_at_lever,\n    ..R78\n};",
                 "pub const R79: LeverHooks = LeverHooks {\n    ..R78\n};\n"
                 "#[allow(dead_code)]\nconst _N2: fn(&ScheduledStatorCore, &LeverArm) -> "
                 "ScheduledStatorCore = r79_at_lever;")]),
    "N3": (SC, [("            hh *= grow;", "            hh *= shrink;")]),
    "N4": (LB, [("Self::with_tau((1.0 + sm) * cmap.phi_surge, b_max, tau)",
                 "Self::with_tau(cmap.phi_surge + sm * cmap.phi_surge, b_max, tau)")]),
    "N5": (SW, [("assert!(phi_air_hi > py_max_of(walls) && phi_lim_lo < py_min_of(walls),",
                 "assert!(phi_air_hi > py_max_of(walls),")]),
    "N6": (SW, [("    let (b0, v0, _, _) = bv_req(&traj[0]);",
                 "    assert!(m.fuel.inner.lever.lim.is_some(), \"rung-80: `n_riding4` counts points "
                 "with the valve STRICTLY INTERIOR (hoisted)\");\n"
                 "    let (b0, v0, _, _) = bv_req(&traj[0]);")]),
    "N7": (SW, [("    fn drop(&mut self) {\n        self.cell.set(self.prev);\n    }",
                 "    fn drop(&mut self) {\n        let _ = self.prev;\n    }")]),
    "N8": (SC, [("    if r == PHI_REF_PHI {\n"
                 "        return (R78_TRIPLE.cap_fuel)(ft, flight, a, h, mf_sched, accel, surge, mf_app);\n"
                 "    }\n", "")]),
}

paths = sorted({p for p, _ in INJ.values()})
orig = {p: open(p, "rb").read() for p in paths}
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
             "--no-fail-fast", "--test", "rung79", "--test", "rung80"],
            capture_output=True, text=True, encoding="utf-8", errors="replace")
        text = r.stdout + r.stderr
        if "error[" in text or "could not compile" in text:
            verdict = "COMPILE-ERROR"
            detail = [l for l in text.splitlines() if l.startswith("error")][:5]
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
                        msgs.append("  " + lines[i + 1].strip()[:220])
            msgs = msgs[:20]
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
