---
name: rung37-combustor-dynamics
description: "SHIPPED rung 37 = the two internal clocks (combustor volume-filling + heat-soak); they SPLIT — volume-filling CONFIRMS rung 34's concession (peak==E0), heat-soak CORRECTS it (E(r,theta0) history-dependent); contrast rung, advisor probe-first discipline"
metadata: 
  node_type: memory
  type: project
  originSessionId: 1c258a79-b4c7-4891-ab3d-022937f8d1a3
  modified: 2026-07-21T17:05:15.781Z
---

SHIPPED rung 37 (`CombustorTransient` subclasses `SpoolTransient`) closes rung 34's **bundled
internal-clock concession** ("no combustor volume-filling, no heat soak … faster clocks below
`τ_spool`, they do not change the `r` framing"). It builds BOTH omitted dynamic states and they
**SPLIT** along the physics — one CONFIRMS the concession, one CORRECTS it (the rung-28/29/32
cross-rung move, done twice in one contrast).

**Volume-filling (a combustor plenum `pt4`, `τ_fill ≪ τ_spool`) CONFIRMS.** At `r→0` the plenum fills
to full quasi-steady `pt4` before `ν` moves ⇒ the peak surge excursion lands on rung-35's `E0` to
**machine zero, INDEPENDENT of the fill clock `r_v`**. The anti-tautology content is STRUCTURAL (not
"fast relaxes fast"): the **FIRST rung where compressor mass flow ≠ NGV mass flow** (`ṁ_c≠ṁ_NGV` by
`~22%` during the fill; rung 34 tied them via `pt4=π_b·π_c·pt2`). Genuinely new closure: the
compressor run from **back-pressure** — invert `π_c(m)` for `m` on the STABLE branch (`_PHI_FLOOR=0.3`,
past the η-island peak where `π_c(m)` turns non-monotone — a real gotcha in the build).

**Heat-soak (a metal state `Tm`, `τ_soak ~ τ_spool`) CORRECTS.** NOT a fast clock ⇒ a second STATE
makes `E = E(r, θ₀)` **history-dependent** (refutes rung 34's blanket claim). The modeled combustor
gas-path sink is surge-**PROTECTED**: `cold < hot-reslam < adiabatic` (cold metal depresses `Tt4_turb`
→ colder NGV passes more corrected flow → `φ` away from surge; channel a beats the rung-36 "slow spool
parks at thin low-ν margin" counter-channel, which doesn't flip the EARLY peak). So rung 34/35's
adiabatic no-soak case is the **ceiling**; a hot reslam is merely the least-protected case. The cost is
the **accel-time LAG** (cold ~2.5–3× slower — the primary CRS/Walsh-Fletcher effect). **ADVISOR CAUGHT
AN OVERCLAIM (honesty amendment, follow-up commit):** I first called the hot reslam "the bodie / why
engines carry a reslam schedule" — WRONG. My channel is surge-protective (never worse than no-soak), but
the real bodie hazard is the OPPOSITE sign (heat soakage moving the working line TOWARD surge) and lives
in an UNMODELED compressor-side channel (tip-clearance / compressor soak). The overlap with the real
bodie is the history-dependence, NOT the sign. (A rung-29-flavored "the obvious channel has the wrong
sign" result — worth the honest framing.) Heat-soak equilibrium == rung 35 because `Tm=Tt4_burner ⇒ Q=0`
at steady (**transient-only — never moves the running line**).

Modeled SEPARATELY (the contrast IS the rung; combined 3-state deferred). **Reduce = EXACT DISPATCH,
not a stiff limit** (advisor's requirement): both clocks off ⇒ inherited `equilibrium_fuel`/
`integrate_fuel` are literally rung 34/35 (rung 31–36 suites pass unchanged); the two equilibria
reproduce rung 35 via independent closures (back-pressure invert; `Q=0`). Built on rung 35's
**fuel-control** forward-burner path (advisor: dodges the "what does commanded Tt4 mean" ambiguity).

**Method matters (advisor):** I wrote a THROWAWAY PROBE first and got both load-bearing SIGNS on the
table before choosing any framing — the advisor caught that my initial guesses (volume-filling gives a
"peak cushion"; heat-soak "makes surge worse") were BOTH likely wrong. Probe showed: peak==E0
(confirmation, not cushion) and heat-soak PROTECTS surge (channel a wins, robust across G×r_m). Ship
negatives; don't build the elegant narrative ahead of the numbers. See
[[rung34-spool-transient]] [[rung35-fuel-metering]] [[rung36-surge-line]].
