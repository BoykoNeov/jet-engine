"""Slice AG step 4 mutation sweep. Patch, rebuild, drive, diff, restore, verify SHA."""
import hashlib
import io
import subprocess
import sys

SRC = r"M:\claud_projects\jet engine\rust\src\sensed_cap.rs"
MANI = r"M:\claud_projects\jet engine\rust\Cargo.toml"
PRISTINE = io.open(SRC, encoding="utf-8", newline="").read()
SHA = hashlib.sha256(PRISTINE.encode("utf-8")).hexdigest()
print("pristine sha256 %s" % SHA)

PY = dict(l.rstrip("\n").split("\t")
          for l in io.open(r"W:\temp\claude\slice-ag-step4\py.tsv") if "\t" in l)

INJ = [
    (1, "accel None in cap_march",
     "let leg = StatorLeg { accel: Some(accel), surge, tt4_max: Some(tt4_max) };",
     "let leg = StatorLeg { accel: None, surge, tt4_max: Some(tt4_max) };"),
    (2, "cap_law not set on the rig",
     "    m.fuel.inner.cap_law.set(cap_law);\n    let leg = StatorLeg",
     "    let leg = StatorLeg"),
    (3, "ic_cap not carried",
     "    m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());\n    m.fuel.inner.cap_law.set(cap_law);",
     "    m.fuel.inner.cap_law.set(cap_law);"),
    (4, "accel_for reaches R75_TRIPLE directly",
     "    let m = (core.triple_hooks().shared_rig)(core, &SharedRigArm {",
     "    let m = (crate::anti_windup::R75_TRIPLE.shared_rig)(core, &SharedRigArm {"),
    (5, "accel_schedule n = 12",
     "pub const ACCEL_SCHEDULE_N: usize = 13;",
     "pub const ACCEL_SCHEDULE_N: usize = 12;"),
    (6, "accel_for tau_rel = tau_f",
     "        tau_att: tau_f,\n        tau_rel: 3.0 * tau_f,\n        inc,\n"
     "        ..Default::default()\n    }).0;",
     "        tau_att: tau_f,\n        tau_rel: tau_f,\n        inc,\n"
     "        ..Default::default()\n    }).0;"),
    (7, "accel_for tau_att = the governor's clock",
     "    let (tau_f, _tau_gov, tau_q, tau_s) = taus;",
     "    let (tau_f, _tau_gov, tau_q, tau_s) = taus;\n    let tau_f = taus.1; let _ = tau_f;"),
    (8, "c_at fold becomes w.max(1e-9)",
     "let dw = rel * if 1e-9 > w { 1e-9 } else { w };",
     "let dw = rel * w.max(1e-9);"),
    (9, "c_at one-sided difference",
     "    Ok((cap(w + dw)? - cap(w - dw)?) / (2.0 * dw))",
     "    Ok((cap(w + dw)? - cap(w)?) / dw)"),
    (10, "c_at state guards SWAPPED",
     "    let _sb = MarchedBleed::set(&core.fuel.inner, q);\n"
     "    let _sv = MarchedStator::set(&core.fuel.inner, v);",
     "    let _sb = MarchedBleed::set(&core.fuel.inner, v);\n"
     "    let _sv = MarchedStator::set(&core.fuel.inner, q);"),
    (11, "CapScope::drop restores SOLVE",
     "        self.core.cap_law.set(self.prev);",
     "        self.core.cap_law.set(CAP_LAW_SOLVE);"),
    (12, "CapScope::drop restores nothing",
     "        self.core.cap_law.set(self.prev);",
     "        let _ = self.prev;"),
    (13, "cap_march tau_rel = tau_f",
     "        tau_att: tau_f,\n        tau_rel: 3.0 * tau_f,\n        inc,\n"
     "        ..Default::default()\n    });",
     "        tau_att: tau_f,\n        tau_rel: tau_f,\n        inc,\n"
     "        ..Default::default()\n    });"),
    (14, "c_at drops the /pi_b",
     "        Ok(accel.cap(i.base.close.n_hp, i.base.close.pt4 / core.fuel.inner.inner.base.pi_b))",
     "        Ok(accel.cap(i.base.close.n_hp, i.base.close.pt4))"),
]


def run(cmd):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True)


rows = []
for n, name, old, new in INJ:
    assert PRISTINE.count(old) == 1, (n, name, PRISTINE.count(old))
    io.open(SRC, "w", encoding="utf-8", newline="").write(PRISTINE.replace(old, new))
    b = run('cargo test --release --manifest-path "%s" --test slice_ag_step4_drive' % MANI)
    if "test result: ok" not in b.stdout:
        out = b.stdout + b.stderr
        kind = ("KILLED (compile)" if "error[E" in out or "error:" in out
                else "KILLED (panic)")
        why = [l for l in out.splitlines() if "panicked" in l or "error[E" in l]
        rows.append((n, name, kind, (why[0] if why else "")[:170]))
    else:
        rs = dict(l.rstrip("\n").split("\t")
                  for l in io.open(r"W:\temp\claude\slice-ag-step4\rs.tsv") if "\t" in l)
        d = [k for k in PY if k not in rs or PY[k] != rs[k]]
        extra = [k for k in rs if k not in PY]
        rows.append((n, name, "KILLED" if (d or extra) else "SURVIVED",
                     "%d keys differ/absent, %d extra" % (len(d), len(extra))))
    io.open(SRC, "w", encoding="utf-8", newline="").write(PRISTINE)
    assert hashlib.sha256(
        io.open(SRC, encoding="utf-8", newline="").read().encode("utf-8")).hexdigest() == SHA
    print("%2d  %-45s %-14s %s" % rows[-1])

print()
for r in rows:
    print("| %d | %s | %s | %s |" % r)
