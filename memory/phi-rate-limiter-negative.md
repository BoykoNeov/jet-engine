---
name: phi-rate-limiter-negative
description: "rung 60's seam (a dφ/ds cap) is NEGATIVE — fuel's authority over φ INVERTS between level and derivative, so no fuel-side leg can arrest a descent; the ONE negative carrying a test"
metadata: 
  node_type: memory
  type: project
  originSessionId: c9489eb8-7d46-4b45-9f03-377ff2bc71f9
  modified: 2026-07-29T01:33:01.963Z
---

Rung 60's named seam — a **φ-RATE limiter** (`dφ/ds` cap) that would "relocate rather than
floor" the protected variable and so test its composability law — is **NEGATIVE**, shipped as
`docs/phi-rate-limiter-negative.md` + `tests/test_phi_rate_limiter_negative.py` (2026-07-29).
Not a rung: no code survives, `engine.py` byte-unchanged.

**THE HEADLINE: fuel's authority over φ INVERTS between the LEVEL and the DERIVATIVE.**
Rung 49's bracket ("cutting fuel raises φ") is load-bearing under six shipped rungs
(49/50/51/52/58/60) and is a *level* property. One derivative up the sign reverses: cutting
fuel cools Tt4 ⇒ collapses shaft acceleration ⇒ kills the STATE term, the only term that lifts
φ. So the arresting bracket has no root and the leg is **unrealisable on fuel**, both spools.

**Rung 60's stated reason was wrong — the blocker is the SIGN, not the plant.** `dφ/ds` IS
available inside the derivative call with no state (chain rule; `ν̇` is what `der` returns), so
"needs a state" was a property of one formulation. Same shape as rung 52 refuting rung 51's
deferral reason. See [[rung60-matched-floor]], [[rung52-asymmetric-lag]].

**Three method lessons worth keeping:**
1. **Probe the CONTROLLABILITY SIGN before designing the leg.** One frozen-state fuel sweep
   killed the whole rung in ~2 minutes, before any `engine.py` edit.
2. **The advisor killed the tempting third object.** A rate-TRIGGERED leg (read the rate, clip
   independently) dodges the sign problem but is under-determined — every clip law is rung 49
   re-gated, a new unanchored constant, or the dead formulation. And `dφ/ds = 0` IS the φ
   minimum, so its release edge sits exactly where the minimum is.
3. **A coverage assertion caught my own over-claim.** I wrote "bracket rootless to 0.9^60 =
   0.18% of flow"; the gate's `seen >= 12` check revealed only 13-16 of the trials are
   evaluable (~19-25% of flow) — the rest leave the modeled speed-line region. The corrected,
   stronger claim is DIVERGENCE (2.0-15.4×) across the domain where the plant is defined.
   **Never quote a search's nominal depth without checking how much of it evaluated.**

**Why this negative has a test and the other six do not:** it BOUNDS a claim six rungs rest on,
and no per-rung gate looks at a derivative. It pins the level monotonicity (must hold) and the
rate inversion (must also hold) in one file so they cannot drift apart.
