"""Slice AG step 5 mutation sweep. Patch, rebuild, drive, diff, restore, verify SHA."""
import hashlib
import io
import subprocess

SRC = r"M:\claud_projects\jet engine\rust\src\sensed_cap.rs"
MANI = r"M:\claud_projects\jet engine\rust\Cargo.toml"
PRISTINE = io.open(SRC, encoding="utf-8", newline="").read()
SHA = hashlib.sha256(PRISTINE.encode("utf-8")).hexdigest()
print("pristine sha256 %s" % SHA)

PY = dict(l.rstrip("\n").split("\t")
          for l in io.open(r"W:\temp\claude\slice-ag-step5\py.tsv") if "\t" in l)
print("golden keys %d" % len(PY))

INJ = [
    (1, "cap_rows passes None to rhs_gains_at",
     "            rhs_gains_at(&m, flight, p, Some(accel), surge.as_ref(), tt4_max, tau_f,"
     " taus.1,",
     "            rhs_gains_at(&m, flight, p, None, surge.as_ref(), tt4_max, tau_f, taus.1,"),
    (2, "cap_rows drops lag_coord.set(DEMAND)",
     "    m.fuel.inner.lag_coord.set(LAG_COORD_DEMAND);\n"
     "    let b_max = m.fuel.inner.lever.lim.expect(\"the rig arms a valve\").b_max;\n"
     "    let pts = riding4(&traj, b_max);\n"
     "    let lag = lag.expect",
     "    let b_max = m.fuel.inner.lever.lim.expect(\"the rig arms a valve\").b_max;\n"
     "    let pts = riding4(&traj, b_max);\n"
     "    let lag = lag.expect"),
    (3, "accel_binds folds min instead of max",
     "            accel_binds: (if cap_a2 > cap_a { cap_a2 } else { cap_a }) < cap_s,",
     "            accel_binds: (if cap_a2 < cap_a { cap_a2 } else { cap_a }) < cap_s,"),
    (4, "the SENSED cap call drops mf_app",
     "                       cf(Some(accel), None, Some(ma)) };",
     "                       cf(Some(accel), None, None) };"),
    (5, "the PHI cap call passes Some(accel)",
     "            let s3 = cf(None, surge.as_ref(), None);",
     "            let s3 = cf(Some(accel), surge.as_ref(), None);"),
    (6, "tau_auth / tau_masked SWAPPED",
     "            tau_auth: if auth == Authority::Fuel { tau_f } else { taus.1 },\n"
     "            tau_masked: if masked == Authority::Fuel { tau_f } else { taus.1 },",
     "            tau_auth: if masked == Authority::Fuel { tau_f } else { taus.1 },\n"
     "            tau_masked: if auth == Authority::Fuel { tau_f } else { taus.1 },"),
    (7, "the two scopes NEST the other way round",
     "            let _ws = WindupScope::set(&m.fuel.inner, law, tau_t);\n"
     "            let _cs = CapScope::set(&m.fuel.inner, cl);",
     "            let _cs = CapScope::set(&m.fuel.inner, cl);\n"
     "            let _ws = WindupScope::set(&m.fuel.inner, law, tau_t);"),
    (8, "auth_err targets (c+1)/tau_auth",
     "                    auth_err: py_max(&rr, |x| (x.auth_diag - (x.c - 1.0) / x.tau_auth)"
     ".abs()",
     "                    auth_err: py_max(&rr, |x| (x.auth_diag - (x.c + 1.0) / x.tau_auth)"
     ".abs()"),
    (9, "masked_moved loses its conditional",
     "                    masked_moved: py_max(&rr, |x| if x.masked_diag0.abs() > 1e-30 {\n"
     "                        (x.masked_diag - x.masked_diag0).abs()\n"
     "                            / 1e-30f64.max(x.masked_diag0.abs())\n"
     "                    } else {\n"
     "                        x.masked_diag.abs()\n"
     "                    }),",
     "                    masked_moved: py_max(&rr, |x| (x.masked_diag - x.masked_diag0).abs()\n"
     "                                                  / 1e-30f64.max(x.masked_diag0.abs())),"),
    (10, "row_err's two targets INVERTED",
     "                            let tgt = if sched { x.c } else { x.c - 1.0 };",
     "                            let tgt = if sched { x.c - 1.0 } else { x.c };"),
    (11, "det_err scores against (1 + c)",
     "                        Some(py_max(&det_rat, |x| (x.0 - (1.0 - x.1)).abs()))",
     "                        Some(py_max(&det_rat, |x| (x.0 - (1.0 + x.1)).abs()))"),
    (12, "n_inert counts the BINDING rows",
     "                let n_inert = rows.iter().filter(|x| x.auth == auth && !x.accel_binds)"
     ".count();",
     "                let n_inert = rows.iter().filter(|x| x.auth == auth && x.accel_binds)"
     ".count();"),
    (13, "s_tail folds min(taus)",
     "    let s_tail = r + tail * [taus.0, taus.1, taus.2, taus.3].into_iter()\n"
     "                                .reduce(|x, y| if y > x { y } else { x })"
     ".expect(\"four clocks\");",
     "    let s_tail = r + tail * [taus.0, taus.1, taus.2, taus.3].into_iter()\n"
     "                                .reduce(|x, y| if y < x { y } else { x })"
     ".expect(\"four clocks\");"),
    (14, "fuel_int folds RIGHT to LEFT",
     "    let fold_mf = |t: &[FuelPoint]| t.iter().fold(0.0f64, |acc, p| acc + p.mf) * ds;",
     "    let fold_mf = |t: &[FuelPoint]| t.iter().rev().fold(0.0f64, |acc, p| acc + p.mf) * ds;"),
    (15, "fixed_point reads solve(q), not sensed(q, w0)",
     "            fixed_point: (sensed(q, w0) - w0).abs(),",
     "            fixed_point: (solve(q) - w0).abs(),"),
    (16, "predicted = 1/(1 + c)",
     "            predicted: 1.0 / (1.0 - c),",
     "            predicted: 1.0 / (1.0 + c),"),
]


def run(cmd):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True)


rows = []
for n, name, old, new in INJ:
    assert PRISTINE.count(old) == 1, (n, name, PRISTINE.count(old))
    io.open(SRC, "w", encoding="utf-8", newline="").write(PRISTINE.replace(old, new))
    bres = run('cargo test --release --manifest-path "%s" --test slice_ag_step5_drive' % MANI)
    if "test result: ok" not in bres.stdout:
        out = bres.stdout + bres.stderr
        kind = ("KILLED (compile)" if "error[E" in out or "error: " in out
                else "KILLED (panic)")
        why = [l for l in out.splitlines() if "panicked" in l or "error[E" in l]
        rows.append((n, name, kind, (why[0] if why else "")[:150]))
    else:
        rs = dict(l.rstrip("\n").split("\t")
                  for l in io.open(r"W:\temp\claude\slice-ag-step5\rs.tsv") if "\t" in l)
        d = [k for k in PY if k not in rs or PY[k] != rs[k]]
        extra = [k for k in rs if k not in PY]
        # WHICH GRID caught it -- the whole reason the second margin was added.
        m10 = len([k for k in d if k.startswith(("E/0.1/", "F/0.1/"))])
        m20 = len([k for k in d if k.startswith(("E/0.2/", "F/0.2/"))])
        rows.append((n, name, "KILLED" if (d or extra) else "SURVIVED",
                     "%d differ/absent (%d @0.10, %d @0.20), %d extra"
                     % (len(d), m10, m20, len(extra))))
    io.open(SRC, "w", encoding="utf-8", newline="").write(PRISTINE)
    assert hashlib.sha256(
        io.open(SRC, encoding="utf-8", newline="").read().encode("utf-8")).hexdigest() == SHA
    print("%2d  %-45s %-16s %s" % rows[-1])

print()
for r in rows:
    print("| %d | %s | %s | %s |" % r)
