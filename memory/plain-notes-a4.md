---
name: plain-notes-a4
description: "Roadmap A4 (plain-language notes, docs/notes/) DONE 2026-11-14, every family 25-108; lessons: a number check that agreed with itself; corrections hide in later specs; a scraped headline is the wrong paragraph; forward pointers must be added to notes already written"
metadata:
  node_type: memory
  type: project
  originSessionId: 688f24b4-7dc4-4660-8fee-113193ab630d
  modified: 2026-11-14T14:22:51.045Z
---

**State, 2026-11-14: DONE.** `docs/notes/` holds every family from 25 to 108 (index `docs/notes/README.md`); each
held by `docs/notes/check-numbers.ps1` (misses 0). 2026-11-14 added 25–30, 53–56+61, 57–63, 64–68, 85, 86–89 and
69–84 (one short page). **Open gap, outside A4, left for the user to choose:** `NOTES.md` has no section of its own
for rungs 17, 20, 21, 23, 24 (recorded in the notes index).

**Lessons.**
- **A check that agrees with itself.** My first number check searched every spec for any substring, so it could
  not fail. The kept `check-numbers.ps1` checks each rung section against its own spec as WHOLE numbers; prove it
  by planting a wrong number in a copy (`-Notes <folder>`). [[instrument-fed-by-what-it-certifies]]
- **Corrections hide in later specs.** Read the whole rung table for CORRECTS / BOUNDS / INVERTS / … and each later
  spec's "neighbours" section; also negative/margin docs (e.g. rung 29's two margin checks) and rungs whose row does
  NOT name the rung they undercut (rung 105's unchoked vane undercuts 59/63's premise).
- **Read the headline; do not scrape it.** An awk "HEADLINE" scrape returned a preamble line for rungs 81/82, so
  those bullets were written from the one-line table row and said the wrong thing (advisor caught it).
- **Write forward pointers into notes already written** when a later family bounds them (46–52 needed 64's
  "location" bound and 89; 90–94 needed 64's ceiling). The roadmap's practice line said so; I first skipped it.
- **Write dependency order, not page order:** 85–89 lean on 53–68's ideas, so 53–68 went first.
- **Windows:** `python3` is the Store stub and hangs (cost a 2-minute timeout); keep `.ps1` regexes ASCII. See
  [[windows-tooling-file-hazards]].

Related: [[try-this-menu-a1]], [[roadmap-and-standing-engine]].
