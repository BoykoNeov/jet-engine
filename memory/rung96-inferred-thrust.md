---
name: rung96-inferred-thrust
description: "Rung 96 (governor infers thrust from EPR or fan speed) process lessons — a prediction's NAMED RISK is a claim too: measure it even when the prediction hits (it failed on 2 maps, H3 hit anyway); prove an identity on every engine variant before calling it construction; a crash-map time under load is not a cost"
metadata:
  node_type: memory
  type: project
  originSessionId: ceea2c90-bfc3-4df3-b26e-915b5fb41fa3
  modified: 2026-10-16T13:50:46.000Z
---

Rung 96 (2026-10-16): `inferred_thrust.rs` + `governor.rs` laws `Epr`/`LpRate`/`LpProp`; anchor `67075f1`, model
`0621531`; H1–H4 all HIT (44 cells). Setup proposed to the user, who answered "go": both readings as a toggle, thrust line
through a 13-row steady table, rung 95's speed laws on the LP shaft, table clean (generator off) or loaded. Follows
[[rung95-governor]], [[rung95-decisions]].

**Lesson 1 — the condition a prediction names as its RISK is a claim too; measure it when the prediction hits.** I
wrote in the spec "the HP shaft leads (it held on all four maps)" — never measured. Measured: it LAGGED on two maps and
H3 hit anyway (the fuel excess outweighs it) — a finding, not a confirmation. Second time running (rung 95 lesson 3):
before commit, grep the spec for "held", "on all", "every" and trace each to a printed number.

**Lesson 2 — an identity that makes a prediction "construction" must be probed on every engine variant first** (the
advisor's blocker): EPR ⇒ gross thrust held to 1e−15 on plain/generator/customer/levers but only 1e−13 cooled — the
gate's bar had to come from the worst variant, not the default.

**Lesson 3 — a crash map's slowest time under machine load is not a cost.** 10 s in the map, 1.6 s re-timed alone,
and rung 95's route took 4.4 s on the same request: re-time alone AND against the predecessor before blaming the rung.

Tooling: `python3` is the Store stub too (hung a sed chain) — see [[windows-tooling-file-hazards]]; a PowerShell
`-File` script with `ValueFromRemainingArguments` cannot take `cargo run --release -- …` (the `--` breaks binding) —
build, then run `target\release\turbojet.exe` directly.
