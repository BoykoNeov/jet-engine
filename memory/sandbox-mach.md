---
name: sandbox-mach
description: "Sandbox plan § 29 (Mach knob on Controls, 2026-10-18) process lessons — an archived probe source that was a copy of another probe; identical output sizes were the tell"
metadata:
  node_type: memory
  type: feedback
  originSessionId: f1e86d85-6f2a-4ab4-9593-9fd477e71319
  modified: 2026-10-18T05:56:07.854Z
---

The § 29 rerun of the density-rule check rebuilt the probe from its ARCHIVED source, which had been archived as a copy
of the choke probe by mistake. The "rerun" printed the choke table a second time, and the summary carried it forward as
"the rulecheck output, not yet read". Caught only because `mach_rulecheck.txt` and `mach_choke.txt` had the same byte
size (`cmp` said IDENTICAL). The real source was recovered from the session transcript (the heredoc that wrote it).

**Why:** an archive is trusted as the instrument of record; a wrong copy silently re-measures something else.
**How to apply:** when archiving a probe, check its output header against the archived output it claims to produce;
when rerunning from an archive, check the new output has the expected FORMAT before reading numbers; two different
probes giving outputs of equal size is a red flag — `cmp` them.

Also: the bash `$side`-in-a-Windows-path hazard bit again (see [[windows-tooling-file-hazards]]) — build per-side loops
in PowerShell. Related: [[sandbox-altitude]], [[rung97-decisions]].
