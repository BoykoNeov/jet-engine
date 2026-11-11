---
name: rung108-decisions
description: "User scope decisions 2026-11-11 for rung 108 (rung 107's next seam: the two-speed temperature limiter — rung 52's fast-cut / slow-give-back lag on the REDLINE clip): BOTH with and without the governor; ALL holds; page = a tick box with a REALISTIC default (fast cut, today's 0.2 lag as the give-back)"
metadata:
  node_type: memory
  type: project
  originSessionId: 7bd67372-25f9-448c-8397-ca17317f3231
  modified: 2026-11-11T10:49:29.049Z
---

User asked "what is next" → picked #1 (rung 107 spec's next seam) on 2026-11-11. Asked (AskUserQuestion) after the
advisor's framing:

- **Where: BOTH** — beside the governor AND on the plain throttle ramp (rung 47's open route gets two speeds too), so
  the page never has a switch that works in one mode only and governed-vs-open is comparable on the page.
- **Holds: ALL** (thrust, HP speed both laws, Tt4, inferred thrust), each speed law's edge measured, as 106/107.
- **Page: REALISTIC DEFAULT** — a tick box "…cuts fast, gives back slowly"; ticking it = a fast cut-in, today's 0.2
  lag kept as the give-back. Fast cut ⇒ finer step ⇒ slower runs: promised to MEASURE and REPORT the cost before
  shipping (and the default pays — the § 45 lesson).

Stated, not asked (carry over): hot-start + hot-request refusals, the minimum-fuel hold, two-lags-at-once refusals
(beside rung 52's release / 88's slow valve / 101's lagged sensor). The step floor for the fast cut waits for a measured
sweep (§ 45's method), then is asked.

Advisor framing: new module = rung 106's `try_der` with `dg = (r−g)/lag.tau(r,g)`; reduce gates (a) equal constants ≡
106 bit for bit (`AsymmetricLag::tau` only selects; tie → release), (b) pinned trigger: before r first falls below g
the run ≡ 106 at τ = attack for ANY release; the ledger becomes ∫(r−g)/τ(r,g) ds = Δg; the RISK to predict: a fast
attack may store MORE cut by the move's end ⇒ a deeper sag as the overshoot shrinks; "fast attack ends the overshoot"
is construction, not a finding. Related: [[rung106-decisions]], [[rung107-decisions]], [[gov-fast-attack-decisions]].

**Probe (`W:\temp\claude\rung108-probe`, sweep.txt / base.txt / timing.txt):** prototype at equal constants 0.2 ≡ the
page's rung 106 / rung 47 march bit for bit (72/72 settings); one attack + one release in every engaged run, 0 re-attacks,
0 min-fuel holds, 0 stops. Governed at one step a fast cut is rough (att 0.1: 14/126 > 1e-4 … 0.01: 91/126); at the
0.005 step clean everywhere (worst 5.8e-5, att 0.01–0.1). Open: one step clean for att ≥ 0.02 (9.6e-5). Cost governed
fine step 0.5–3.0 s natively (4–8× rung 106); open unchanged.

**Floor + step (asked 2026-11-11): attack floor 0.02, governed two-speed ALWAYS on the 0.005 step** (open at 0.02).

**Found: rung 106's SHIPPED route is rough at one step with the floor/schedule beside the lag** (to 6.2e-4, floor dense
EPR; its own step check never combined them; no lag ⇒ clean). Asked: **RECORD NOW, FIX AFTER this rung** (keeps
"nothing that runs today moves").

**PAUSED 2026-11-11 (usage limit):** anchor DRAFT `docs/plans/rung108-anchor-two-speed-limiter.md` committed as wip —
§ 0 givens, § 1 device, § 2 gates (ledger bars), § 3 H1–H8 written but NOT yet advisor-reviewed, so NOT frozen. Next:
advisor review of § 3 → fix → commit as the frozen anchor → archive the probe (`W:\temp\claude\rung108-probe`, sources +
sweep/base/timing/open2/ledger .txt) into `docs/plans/scratch-archive/rung108/` → build `two_speed_clip.rs`. Rung 106's
step-roughness still to be written into the sandbox plan as an open item (it is in anchor G3).

**After the build (2026-11-11):** anchor § 3 frozen (`e14a723`), built, scored 144 HIT / 4 MISS / 12 VACUOUS. The
pinned-trigger gate restated (open route, coarse step: RK4 stages read the give-back early) — the advisor's call, not
asked (the scored overshoot moved by 2.8e−5 K). **Cost reported before shipping** (page: governed 3.3 s against rung
106's 0.6 s; open 0.20 against 0.16 s; worst estimated 9–10 s) — the user: **SHIP AS IS**. Then a side agent caught that
the box had been built UNTICKED although the picked option said the realistic case is the default — re-asked: **TICKED
ON THE PAGE** (the defaults reply), the model's own default OFF so no earlier request/gate/golden moves. Next (the
user's earlier choice): fix rung 106's step roughness (sandbox plan § 43's OPEN item). Lessons: [[rung108-two-speed-limiter]].
