---
name: rung28-coupled-no-march
description: "SHIPPED rung 28 = the rung-26-coupled NO march; confirmed rung 27's verdict but corrected BOTH its stated reasons (heat-release channel + the beta repair)"
metadata: 
  node_type: memory
  type: project
  originSessionId: d9dabcb6-4b28-4275-939f-00fcb7c6a7c3
  modified: 2026-07-20T08:29:07.264Z
---

**Rung 28 (shipped 2026-07-20)** is the rung-26-coupled NO march — rung 27's NO clock
read on rung-26's *relaxing* pool instead of the frozen station-4 pool. `CoupledNOFreezeOut` /
`Gas.coupled_no_freeze_out_nozzle`, spec `docs/rung28-spec.md`, anchor
`docs/plans/rung28-anchor-coupled-no-march.md`.

**The character: rung 27's VERDICT confirmed, BOTH its stated REASONS corrected.** This was
picked as a low-payoff "confirmation of a confirmation" and turned into a two-correction rung.

1. **"Can ONLY slow NO further" was one-sided.** Coupling to rung 26 couples to *all* of rung 26,
   including its **exothermic heat release**, which lifts T and — the NO clock being Arrhenius —
   **speeds** NO destruction. Two opposing channels (depletion vs heat release), decomposed by
   running one clock on two **hybrid trajectories**. Conclusion survives (`net<1` everywhere) but
   `|ln ch2/ln ch1|` rises monotonically 0.003→0.48 — the opposing channel cancels ~half the
   depletion hot. The win is **structural**: depletion **unbounded**, heat release **saturating**.
2. **The β repair.** Rung 27 justified its `a≫1` clock with "NO arrives super-equilibrium" — **false
   at the entry** (`a`=0.31–0.61 hot; NO arrives SUB-eq and initially FORMS), exactly where
   freeze-from-entry is decided. What holds is `β=R1/(R2+R3)<1` ⇒ the surrogate bounds the rate in
   **both** regimes. **Rung 27's numbers are untouched; only its reasoning was repaired** — errata
   added to `_tau_no_destroy` and `docs/rung27-spec.md`.

**Precedent worth remembering: I edited a shipped, committed rung.** Adding errata to rung 27
rather than silently building on a false premise is the behaviour this project wants — but such
edits must be *stated plainly* to the user, not buried in a green test count. See
[[always-commit-and-push]] and [[session-end-routine]].

**Method notes that paid off** (worth reusing on the next rung):
- The advisor's "if channel 2 turns out comparable or dominant, stop and surface" tripwire fired and
  was worth honouring — it led to the β discovery.
- Making the NO relaxer consume a `[(s,p,T,comp)]` **trajectory** bought the bit-for-bit reduce
  *structurally* (identical expression sequence) AND made the channel decomposition first-class
  instead of a probe artifact. One design choice, two payoffs.
- Rung 26's marcher gained a **pure-observer `record=`** hook rather than a third verbatim copy of
  the loop; gated by asserting every returned float is identical with and without it.

**Disclosed weak point:** β reaches ~0.51 hot — *half* the β=1 threshold, a factor 2 not orders.
Worth re-checking at higher `π_c` / hotter cycles. Also found rung 27's "3–9 order" entry margin is
band-specific (2.1e-2, ~1.7 orders, at Tt4=2400 K).

Related: [[rung27-no-freeze-out]], [[rung26-freeze-out]], [[rung25-finite-rate-nozzle]].
