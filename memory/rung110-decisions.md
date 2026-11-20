---
name: rung110-decisions
description: "User scope decisions 2026-11-20 for rung 110 (the turbofan FLOWN off design, roadmap B1 next step): textbook method (no maps) first; ONE solver for every design, built by EXTENDING the two-shaft code with a bypass stream (no parallel module); Fly it switches over only when charts arrive (rung 111); anchor = Mattingly 1996 Example 8-8 from the user's own PDF"
metadata:
  node_type: memory
  type: project
  originSessionId: eb011473-7f0f-453f-88f5-c065f3698fd3
  modified: 2026-11-20T09:07:34.859Z
---

User said "what is next" → I offered roadmap B1–B5 → "go with your recommendation" → I picked B1's next step (fly the
turbofan off design), 2026-11-20.

**Anchor:** Mattingly, *Elements of Gas Turbine Propulsion* (McGraw-Hill 1996) § 8-5 (book pp. 518–541), **Example
8-8** (pp. 526–529): the Example 7-6 engine (rung 109's anchor) referenced at 40 kft M0.8, flown to sea-level static at
Tt4 3000 °R. Printed: α 9.103, πf 1.4973, πcH 16.555, ṁ0 1638, f 0.02769, F/ṁ0 29.04, S 0.3398, F 47 570 lbf,
N_fan 0.938, N_HP 1.00, Table 8-7 iterations. **Source: the user's own PDF at `docs/Elements of Gas Turbine
Propulsion.pdf`** (Internet Archive scan, 1032 pages; book page = PDF page − 68). It is UNTRACKED and listed in
`.git/info/exclude` — never commit it, never put page images in the repo. (The user's first copy held only pp. 1–113;
they replaced it.) Example 8-9 is curves on a max-thrust schedule: trend context only, never scored.

**User answers (AskUserQuestion, 2026-11-20):**
- **Textbook method first:** component efficiencies held at design, both HP vanes choked, no maps. Shaft speeds still
  come from Mattingly's 8-52ag/ah; stall margin, map, slam and the Fly-it afterburner wait for rung 111's charts.
- **One solver for all designs.** First the user chose "B: one solver"; when I asked about the bypass-0 knob they said
  *"if you think two solvers are better - do it that way. my idea is not to have the need to develop extra modules
  that are practically the same. Also - about a sandbox 'design your own engine' i think a single solver would be
  better."* I recommended: ONE solver, built by EXTENDING the existing two-shaft code (rungs 38/39/103/104) with a
  bypass stream — at α 0 it IS the two-shaft engine; the fan knob ALWAYS sets the first compressor (π_f 1 ⇒ one shaft).
- **Fly it switches over "when charts arrive" (rung 111),** not in rung 110 — nothing on the page goes dark, the guide
  is rewritten once. Rung 110 = model + Example 8-8 gate + CLI panel only.

**My calls, told to the user, not objected:** the core air through the fan root is compressed at the COMPRESSOR's
efficiency (rung 109's convention; split polytropically) so the design reduce to rung 109 is exact and no derived
efficiency can exceed 1; the book's own convention lives only in the Example 8-8 gate. The LP turbine ships on rung
104's law (as every two-shaft page engine); the book gate runs on the pin (its own assumption).

**ONE LOOP, not a copy (user, 2026-11-20, mid-build).** My first module ran the bypass case through its OWN copy of
rung 38's loop (dispatching α 0 to rung 39). A side-agent note flagged it; asked, the user chose **"One shared loop"**:
the bypass stream goes INSIDE rung 39's loop in `two_spool.rs` (`r39_body` / `try_cascade_map` / the rebuild), the
no-fan lines unchanged and held by the bit-for-bit oracles, so rung 111's charts land in one place. Lesson: "extend
the code" means the SAME loop — a new loop that calls the same helpers is still a parallel module.

**Correction I owed the user:** my first option text said B switches off Controls — false; Controls, Cooling, Blades,
Burner run their OWN engines. B touches Fly it only (experiments 9–11, ~12 browser checks).

**Session 2 decisions (AskUserQuestion, 2026-11-20):**
- **One nozzle rule for every engine** (two-shaft and one-shaft, fan or not): A8 sized at the design's actual exit
  when unchoked at design. Condition: run the full gate; if any recorded result moves, stop and show the user first.
- **Gate 5 tests the explanation**: keep the exact-equations check; the 0.3 %-of-printed claim is recorded as a MISS
  (thrust 0.367 %), replaced by asserting the model's reference is the book's components with a consistent HP turbine.

Related: [[rung109-decisions]], [[roadmap-and-standing-engine]], [[sandbox-direction]].
