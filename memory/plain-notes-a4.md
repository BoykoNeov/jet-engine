---
name: plain-notes-a4
description: "Roadmap A4 (plain-language notes, docs/notes/) started 2026-11-12, ten families written; lessons: my first number check searched all specs for substrings and could not fail; my first correction search grepped only expected rungs and missed eleven cross-rung links"
metadata:
  node_type: memory
  type: project
  originSessionId: 688f24b4-7dc4-4660-8fee-113193ab630d
  modified: 2026-11-12T15:24:23.266Z
---

**State, 2026-11-12:** `docs/notes/` has the families 31–33, 34–37, 38–45, 46–52, 90–94, 95–101, 102, 103–105 and
106–108; `docs/notes/README.md` is the index. Still to write: 25–30, 53–68, 85–89, and 69–84 last (one short
page). The roadmap's A4 "Progress" lists the working practice.

**Lessons.**
- **A check that agrees with itself.** My first number check asked "does this number appear anywhere in all the
  specs and the whole page?" Short numbers like 0.2 or 12 always do, so it could not fail. The advisor caught it.
  The kept version, `docs/notes/check-numbers.ps1`, checks each rung section against its own spec as WHOLE numbers.
  It was proven by planting two wrong numbers in a copy: the substring version had missed 2.7 (found inside a
  longer number). This is the [[instrument-fed-by-what-it-certifies]] trap again.
- **Corrections hide in later specs.** Grepping only the rungs I expected to correct a family missed eleven links:
  53/57/64 bounding the whole fuel-limiter family, 49 inverting 41, and others. Read the whole rung table in
  `CLAUDE.md` for CORRECTS / BOUNDS / INVERTS / EXTENDS / EXPLAINS / DISCHARGES, and confirm each in that spec's
  "What it does to its neighbours" section.
- **Windows hazards met again.** The editing tool writes a typed `–` as a literal en dash, and Windows
  PowerShell misreads a non-ASCII script saved without a byte-order mark: keep `.ps1` regexes ASCII (`\p{Pd}`).
  `printf` turned `docs\notes` into a line break. GNU `sed` treats `\u` in a replacement as "uppercase the next
  letter". See [[windows-tooling-file-hazards]].

Related: [[try-this-menu-a1]], [[roadmap-and-standing-engine]].
