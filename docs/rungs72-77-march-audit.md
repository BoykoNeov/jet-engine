# RUNGS 72–77's MARCHES, AUDITED — the arrest is a CELL, and rung 74 already owned it

**Status: CHECKED — CONFIRMATION for rungs 72–77, CORRECTION for rungs 78/79 and for the
document that opened this seam.** Not a rung: no new knob, no new constant, no production code.

The seam, in `docs/rung79-gap-margin.md` § 6's own words:

> **Rungs 72–77's § 5 sections.** Only rungs 78 and 79 were checked here. They share the rig —
> and that is a reason to look, **not** a result. **NOT MEASURED; this document makes no claim
> about them, and no later reader should quote one.**

It is now measured. **Rungs 72–77's marches do NOT stand still.** The arrest belongs to one
cell — `(demand coordinate, φ_lim = 0.80)` — and that cell was found, named, mechanised and
gated at **rung 74 § 2.2**, which rungs 78, 79 and the gap document all re-entered without
citing.

---

## 0. THE SEAM HAD TO BE RETARGETED FIRST, AND THAT IS NOT A DETAIL

"§ 5" is rung 78/79's numbering. In the rungs the seam names it is a different section every
time — § 5 is **THE LEDGER** in rungs 72/73 and **WHAT THIS DOES TO THE RUNGS BEFORE IT** in
rungs 74–77. Auditing "§ 5" literally would have read six sections, five of which contain no
trajectory at all, and reported nothing while looking thorough.

So the scope is redefined as **every trajectory-based reading in rungs 72–77, wherever it
lives**, and the work-list is built from the **march call sites**, which are one per rung:

| rung | march | coordinate | wall its findings march at |
|---|---|---|---|
| 72 | `_shared_march` | `clip` | `PHI = 0.80` |
| 73 | `_shared_march` (applied reference) | `clip` | `PHI = 0.80` |
| 74 | `_coord_march` | **both** | `0.80` / `0.76` / `0.70` |
| 75 | `_windup_march` | `demand` | `PHI_BOTH = 0.76` |
| 76 | `_cap_march` | `demand` | `PHI_BOTH = 0.76` |
| 77 | `_ledger_march` | `clip` | `PHI_JAC = 0.80` |
| 78 | `_cap_march` | `demand` | `PHI_JAC = 0.80` ← arrested |
| 79 | `coord_march` | `demand` | `PHI_JAC = 0.80` ← arrested |

**That table is most of the answer.** Rungs 78 and 79 are the only two whose marches sit at
`(demand, 0.80)`, and rungs 75/76 are the only others in the `demand` coordinate at all —
at a **different wall**.

**AND THE LAST COLUMN IS NOT A FORMALITY.** Two of its entries were wrong on the first pass and
had to be read out of the readers rather than off the test files' constants: rung 77's
`leg_slopes` derives its `sm` from `phi_lim = PHI_JAC = 0.80` (not from the `PHI_BOTH` its
reduce arms use), and rung 75's `windup_bill` marches `("demand", "applied", "track")` and
`("demand-latched", …)` — never the `("demand", "sched", "none")` cell an audit would reach for
by default. **A march measured at settings its own rung does not use is a number about nothing**
(rung 63's lesson, one document over). **Both errors were in this document's first draft and
were caught in review, not before it** — the wall came off each test file's most prominent
constant instead of out of the reader that consumes it. The rows below are the re-measured
ones, and rung 77's correction moved it from the audit's third-liveliest march to its
liveliest.

---

## 1. TWO VACUITY MODES, AND THE INSTRUMENT FOR ONE SCORES THE OTHER GREEN

Registered before the sweep, because a one-sided instrument was the failure this audit exists
to look for:

* **MODE A — STATIC PLANT.** The states do not move. This is rungs 78/79's mode, and its
  signature is `nu_lp`/`nu_hp` relative spread **exactly 0.0**.
* **MODE B — DORMANT LEG.** The plant moves, but the loop the section is *about* never acts. A
  trajectory in which the subject does nothing is exactly as vacuous as a frozen one, and a
  spread-only instrument passes it.

So every row below records **both**: state spreads **and** per-loop liveness, using the markers
this family's own readers already use — `required > 0` (rung 47's **governor**),
`0 < b_cmd < b_max` (rung 65's **valve** strictly interior), `v_regime == "riding"` (rung 68's
**stator**), and for rung 49's **φ fuel leg** its observable signature, `min φ_lp` against the
wall.

**THE VERDICT SHAPE IS PRE-COMMITTED**, on rung 79 gap-margin § 2.3's precedent: if a march
stands still or its leg is dormant, the response is **RECORD + GATE + rescore that section's
scope** — never re-tune the wall. A wall placed where a rung's own Jacobian sections require it
is not a bug to fix.

### 1.1 THIS INSTRUMENT'S OWN VACUITY CONDITION

A liveness counter reads the trajectory it is given. **On a frozen plant it therefore reports
the initial condition, at full count** — and § 2's table shows exactly that: the two arrested
rows carry `valve = 341/341`, the highest valve count in the audit, because a plant pinned at
an interior valve position satisfies `0 < b_cmd < b_max` at every step of a march that never
moves. **The two modes are not independent, and mode B's instrument fails hardest precisely
where mode A holds.** Any row is therefore read spreads-first; liveness counts are meaningless
until the plant is known to move.

---

## 2. THE MEASUREMENT — every march, at its own rung's settings

341 steps each, driven through each rung's own march at the settings its own findings use —
taken from the READERS, per § 0: rungs 75/76's `windup_bill`/`cap_bill` march in `demand` at
`PHI_BOTH`, rung 75 in both of its two real cells; rung 77's `leg_slopes` marches `clip` at
`PHI_JAC` and only then switches `_lag_coord` for its frozen readers.

| march | wall | coord | `nu_lp` spread | `Tt4` | `min φ_lp` (vs wall) | gov | valve | stator |
|---|---|---|---|---|---|---|---|---|
| r72 `_shared_march` | 0.80 | clip | 1.144e−01 | 1000→1283 | 0.795155 (−0.61%) | 340 | 123 | 87 |
| r72 `_shared_march` inc | 0.80 | clip | 1.141e−01 | 1000→1283 | 0.791380 (−1.08%) | 340 | 130 | 49 |
| r73 `_shared_march` | 0.80 | clip | 1.148e−01 | 1000→1315 | 0.795155 (−0.61%) | 340 | 121 | 85 |
| r73 `_shared_march` inc | 0.80 | clip | 1.150e−01 | 1000→1354 | 0.791380 (−1.08%) | 340 | 124 | 59 |
| r74 `_coord_march` | 0.80 | clip | 1.144e−01 | 1000→1283 | 0.795155 (−0.61%) | 340 | 123 | 87 |
| **r74 THE ARREST ARM** | **0.80** | **demand** | **0.0** | **1000→1000** | **0.800000 (0.00%)** | 340 | *341* | *66* |
| r74 `_coord_march` | 0.76 | clip | 1.056e−01 | 1000→1281 | 0.756432 (−0.47%) | 332 | 52 | 53 |
| r74 `_coord_march` | 0.76 | demand | 9.382e−02 | 1000→1198 | 0.760992 (+0.13%) | 332 | **0** | **0** |
| r74 `_coord_march` | 0.70 | clip | 1.062e−01 | 1000→1279 | 0.742994 (+6.14%) | 319 | **0** | **0** |
| r74 `_coord_march` | 0.70 | demand | 1.004e−01 | 1000→1198 | 0.746435 (+6.63%) | 319 | **0** | **0** |
| r75 `_windup_march` **the rows** | 0.76 | demand | 9.138e−02 | 1000→1198 | 0.765754 (+0.76%) | 332 | **0** | **0** |
| r75 `_windup_march` **the accident** | 0.76 | demand-latched | 9.914e−02 | 1000→1360 | 0.765911 (+0.78%) | 332 | **0** | **0** |
| r76 `_cap_march` solve | 0.76 | demand | 6.954e−02 | 1000→1179 | 0.760992 (+0.13%) | 332 | **0** | **0** |
| r76 `_cap_march` sensed | 0.76 | demand | 6.560e−02 | 1000→1169 | 0.762375 (+0.31%) | 332 | **0** | **0** |
| r77 `_ledger_march` | **0.80** | clip | 1.054e−01 | 1000→1233 | 0.795155 (−0.61%) | 340 | 161 | 97 |
| **r78 `_cap_march`** (reference) | **0.80** | **demand** | **0.0** | **1000→1000** | **0.800000 (0.00%)** | 340 | *341* | *66* |

*Italicised counts are on a frozen plant and mean nothing (§ 1.1).*

**MODE A: CLEARED for rungs 72–77.** Every one of their marches moves — `nu_lp` by 6.6% to
11.5%, `Tt4` by 169 K to 360 K. The only frozen rows are the two at `(demand, 0.80)`.

### 2.1 TWO INTERNAL CONTROLS

* **The rig replication is faithful.** Driven off-table at rung 75's *reduce* cell
  `("demand", "sched", "none")`, `_windup_march` reproduces `r74 demand @ 0.76` to every digit
  printed (9.382e−02 / 7.921e−02 / 0.760992). It must: rung 75 at `law = "none"` **is** rung 74,
  and that is its own shipped reduce arm. Reproducing it from outside the test suite is the
  check that these marches are being driven the way their rungs drive them. (That cell is not a
  table row precisely because `windup_bill` never marches it — § 0.)
* **`r74 clip @ 0.80` reproduces `r72` exactly** (1.144e−01, 1000→1283, valve 123, stator 87) —
  the coordinate is the *only* difference between the arrested row and the moving one. Same
  rig, same wall, same 341 steps.

---

## 3. MODE B — WHAT IS DORMANT, AND IT IS NOT NOTHING

Reading the `min φ_lp` column against the **free droop**, which the table supplies at the 0.70
arms where the φ leg is provably below the plant's own excursion: **≈ 0.7430 (clip) / 0.7464
(demand)**.

* **The φ fuel leg is LIVE in every row except the two 0.70 arms.** At 0.76 it holds `φ` in
  0.7610–0.7659 — **1.5 to 1.9 percentage points above the free droop** it would otherwise
  reach. That is the leg doing work, not a coincidence of where the excursion lands.
* **At 0.70 the φ leg is DORMANT by 6.1–6.6%, and rung 74 SAYS SO** — its redline test's own
  docstring reads *"the surge leg is below the clip plant's own droop so only the GOVERNOR is
  live — which is what makes this a statement about rung 47's leg and not about rung 49's."*
  Disclosed, deliberate, and load-bearing for the claim it carries.
* **NEW, AND RECORDED HERE FOR THE FIRST TIME: at `(demand, 0.76)` the valve and the stator are
  BOTH inert, 0/341** — in **every** cell rungs **75 and 76** march, including rung 75's
  `demand-latched` accident arm, which is the one its § 4 bill is scored against. Their claims
  are about the fuel-side legs — rung 75's anti-windup device and hand-over, rung 76's `sensed`
  vs `solve` cap — so **no shipped claim is voided**; but those two rungs' trajectories are a
  **two-loop** plant (governor + φ fuel leg), not the four-loop one their surrounding sections
  describe, and neither spec says so.
* Rungs **72, 73** (four loops live) and **77** (governor 340, valve 161, stator 97 — its
  richest march in the family) are the fully-exercised ones.

**So liveness is not a property of a rung. It is a property of the `(coordinate, wall)` pair —
and at every wall in this family at least one of the four loops is inert.** Which one changes
with the wall: at 0.80 all four run, at 0.76 the airflow levers drop out, at 0.70 the φ leg
does too. **Rung 77 is the sharpest case of the pair mattering more than the rung**: it marches
at the *arrested rows' own wall*, 0.80, and is the liveliest march in the audit — because its
coordinate is `clip`.

---

## 4. THE CORRECTION — RUNG 74 FOUND THE ARREST, AND THREE LATER DOCUMENTS RE-ENTERED IT BLIND

Rung 74 § 2.2, shipped, titled **"THE ARREST — disclosed, not tuned away"**:

> At the **inherited** floor (`φ_lim = 0.80`) the surge cap equals the scheduled fuel at `s = 0`
> (§ 0.2). A leg that *tracks* its cap therefore pins `φ` on the floor and the acceleration
> never starts: `max Tt4 = Tt4_lo` **exactly**, `min φ_lp = 0.800000` **exactly**.

It is gated (`test_at_the_inherited_floor_the_demand_plant_does_not_accelerate`), it explains
why rung 74's own comparison arms sit at 0.76 and 0.70, and rung 74 § 10 lists **"THE ARREST,
AS A PLANT"** as an open seam.

**Rungs 75, 76, 77, 78, 79 and `docs/rung79-gap-margin.md` contain the string zero times.**

* **Rungs 75/76 escaped it by the wall and rung 77 by the coordinate** — never by citation.
  Rung 77's `leg_slopes` marches at `PHI_JAC = 0.80`, the arrest's own wall, and is untouched
  only because `_ledger_march` is `clip`. **Two rungs after rung 74 disclosed the arrest, one
  was already running one knob away from it.**
* **Rungs 78 and 79 marched inside it.** Rung 79's `coord_march` docstring argues at length that
  the coordinate **must** be `demand` (`clip` dispatches out of the ladder before `_cap_fuel`);
  combined with `PHI_JAC = 0.80`, inherited from rung 76, that lands the section in rung 74's
  arrest arm exactly. The requirement was real and so was the wall — **what is missing is that
  either spec knew the intersection had already been characterised.** Both now carry the
  pointer: `docs/rung78-spec.md` § 5.3 and `docs/rung79-spec.md` § 9. Rung 78 § 5 already
  declared itself *NOT ESTABLISHED*, so this **sharpens** it rather than correcting a number —
  its `hits = 1366` / `binds = 0` are **one operating point read 341 times**, and its § 9 seam
  needed two things, not one (a binding accel leg **and** a plant that moves).
* **`docs/rung79-gap-margin.md` rediscovered it from scratch**, mechanised it correctly and
  independently (the stator lifting the free `φ = 0.7731` onto the wall, so rung 49's leg reads
  a state already on its floor), and reported it as new. Its § 2.1 and rung 74 § 2.2 are the
  same phenomenon from two ends: rung 74's *"the surge cap equals the scheduled fuel at `s = 0`"*
  is the gap doc's *"the cap therefore equals the current fuel at every step"*, derived twice.

**What this corrects is not a number — every measurement in both documents stands.** It is the
attribution: the gap document's finding is a **rediscovery**, and the honest ledger entry is
that rung 74 predicted rungs 78/79's standstill before either was built, in a section written
to warn about exactly that.

### 4.2 AND THIS DOCUMENT'S OWN "CELL" IS UNDER-STATED — `docs/rung74-arrest-interval.md`

*"The arrest is the cell `(demand, 0.80)`"* was true of the cells **measured**. It is an
**INTERVAL** — every wall in `(0.7731162133, ≈0.852)`, both edges derived. This document's two
`demand` points, 0.76 and 0.80, happen to sit either side of the lower edge (the **free
operating point**), which is why one marches and one does not; a third point anywhere between
0.7732 and 0.85 would have shown it. **The localisation to the coordinate stands** — `clip` has
no arrest at any wall — and so does § 3's *"liveness is a property of the `(coordinate, wall)`
pair"*, now with a mechanism and made **exhaustive**: in `demand` that pair has **no** live
cell at any wall, so §§ 2–3's `(demand, 0.76)` dormancy is not one wall's accident but the only
outcome available. The "NEW, AND RECORDED HERE FOR THE FIRST TIME" bullet in § 3 keeps its
credit; what it lacked was the reason.

### 4.1 AND THE GAP DOC'S HEDGE WAS RIGHT TO BE A HEDGE

Its § 6 refused to infer from "they share the rig" and said so in bold. **Inferring would have
been wrong**: the shared rig implies nothing, because the arrest needs the *coordinate* as well,
and five of the six rungs it declined to claim about are in the clear. A project whose
discipline is *count, don't infer* got the payoff for once in the direction of the refusal.

---

## 5. THE GATES

Per this repo's convention (a margin sweep gates inside the rungs it is about), in each affected
rung's own test file, each paired with the control that makes it non-vacuous:

* **`tests/test_rung72.py`, `tests/test_rung73.py`** — the march MOVES at `φ_lim = 0.80` and all
  four loops are live. The control is the wall: these run at the same wall, same rig and same
  341 steps as the arrested rows, so the gate is a **counter-example that localises the arrest
  to the coordinate**, not a bare liveness assertion.
* **`tests/test_rung75.py`, `tests/test_rung76.py`** — the march MOVES at 0.76, the φ leg HOLDS
  (`min φ_lp` above the wall and far above the free droop), and the valve and stator are **both
  0/341**. The dormancy is the finding, so it is pinned as an equality, not a bound. Rung 75's
  gate marches `("demand", "applied", "track")` — `windup_bill`'s own cell, not the
  `("demand", "sched", "none")` one, which is this rung's REDUCE to rung 74 and would have
  gated the parent (§ 0).
* **`tests/test_rung77.py`** — at `PHI_JAC = 0.80`, the arrested rows' own wall: the march MOVES
  and, being in `clip`, keeps three loops live. Gated against the `demand` rows' zeros so the
  two coordinates cannot be conflated later.

No gate is added for rung 74: it already gates the arrest, and this document's job there was to
find that out.

---

## 6. WHAT IS STILL OPEN

* ~~**A φ-floor march with NON-ZERO initial margin**~~ — **CLOSED by REFUTING its premise,
  `docs/rung74-arrest-interval.md`.** In `clip` it needs no new rig: `(clip, 0.76)` — rungs
  75/76's own wall — already has all four loops live with +1.3% initial margin. In `demand` it
  is unreachable at **every** wall, which is that document § 5 and the reason the
  `(demand, 0.76)` dormancy this audit found is not an accident of one wall.
* **THE ARREST, AS A PLANT** — rung 74 § 10's own seam, still open, still uncited by the five
  rungs that followed it.
* **Rungs 62–71's marches.** NOT MEASURED. They predate the `demand` coordinate entirely, so
  the arrest cannot reach them — but that is an argument, not a measurement, and this document
  makes no claim about their leg liveness.
* Whether rung 75's and rung 76's trajectory claims **would survive at a wall where all four
  loops are live** — the dormancy voids nothing here, but it has never been checked the other
  way, and it is a cheaper question than the new rig.
