---
name: rust-port-slice-ai-step2
description: "Slice AI step 2 (rung 79's coord_at/coord_scan/coord_census) — a mechanism a spec records for ONE reader was never checked at its OTHER readers; the scan's two exact zeros were the fallback compared with itself"
metadata:
  node_type: memory
  type: project
  originSessionId: 65a3b767-e1f4-4dbf-a92c-2813854b3f78
  modified: 2026-09-28T20:32:57.691Z
---

Slice AI step 2 shipped 2026-09-26 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.33.2): `coord_at`, `coord_scan`, `coord_census` in `rust/src/state_coordinate.rs`, gated by
`rust/tests/slice_ai_scan.rs` (3 gates, every value bit for bit against a Python probe, embedded as
`f64::from_bits` literals). Green on the first run.

**THE LESSON: A MECHANISM A SPEC RECORDS AT ONE READER IS NOT CHECKED AT ITS OTHER READERS.** Rung
79 § 5.1 already says the plant's `d_set = 0` is `_surge_fuel`'s fallback, not coordinate
invariance — for the MARCH. Its § 1 table still reads D3's `dwdq_err = 0` as "exactly zero", and
`test_rung79.py` gates it. Counters snapshotted around `coord_scan` in Python: `[0,0,30,30,30,30]` —
every incidence solve in the scan fell back too, so `w_inc == w_phi` bit for bit and both zeros are
the fallback compared with itself. Found only because the counters were probed BEFORE writing the
port ([[rust-port-slice-ai-preflight]]: a zero value-diff is two findings, only a counter splits them).

**Why:** a recorded mechanism feels "handled", so its other sites go unexamined — and the site that
reads it as a positive result is the one nobody re-derives.

**How to apply:** when a spec says "reading X is vacuous because of mechanism M", grep for every
other reader that could route through M and count its path there too. Pin the counter vector in the
SAME test that runs the reader (reset, run, read — never through a cache, per
[[rust-port-slice-ah-step7]]): here two injections (incidence leg solved in `phi`) moved NO value
and were caught by the counters alone.

Also measured: the census walks `Gi` for real but reads only SIGNS, so a census walking `Gs` twice
is bit-identical — D1's theorem is exactly what blinds it; only D2 (the slope ratio) sees the
coordinate. `predicted_ratio` is `0x3ff8ffffffffffff`, not 1.5625. Sweep: 12 injections, predicted
in writing first, 12 of 12 right (6 killed, 6 survived for stated reasons). The spec's D3 row was
NOT edited in the port commit — offered to the user as a separate docs-only change. **Since done, at
the user's word (2026-09-28)**: the spec's D3 row in `ea43d9c`; `main.py`'s rung-79 panel and the two
`test_rung79.py` docstrings in `9f1d11a`.
