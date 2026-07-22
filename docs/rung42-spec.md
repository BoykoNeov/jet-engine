# Rung 42 — Interstage bleed: the valve is a degree of freedom on ONE spool

Rungs 36 **and** 41 closed with the same standing concession, in almost the same words: *"no
bleed valve / variable stator — the devices that raise the margin at low speed; this rung
exhibits the margin they protect, it does not model them."* Rung 41 additionally **located**
the exposure: over a 2:1 throttle `φ_L` falls ~29 % while `φ_H` falls ~7 % and is bounded —
**the LP compressor carries the surge problem**. A handling-bleed valve at station 25 is
exactly the device that is supposed to act there. Rung 42 fits it.

> **THE FINDING — bleed is a genuinely NEW degree of freedom on the LP spool, and NOT on the
> HP spool.** At a fixed throttle and flight condition the LP map coordinate `x_L = Tt4/Tt2`
> is **exactly** bleed-invariant (both are *inputs*), so the entire `+8 – 12 %` change in
> `φ_L` is **displacement OFF the running line**: the LP running line becomes a **family**
> indexed by `b`. The HP compressor stays on **one curve** — its `φ_H(x_H)` is bleed-invariant
> to **0.01 – 0.016 %**, a **~700–1000×** contrast — because rung 39's `(†)` carries no `b`
> and the HP shaft balance cancels it. Opening the valve does not give the HP spool a new
> freedom; it only **slides it along the same running line the throttle slides it along**.

Two consequences follow, and the spec is careful about which is new:

* **INHERITED (rung 41).** Because the HP point only slides along its own curve, its whole
  response to the valve is rung 41's closed-form sensitivity
  `s_H = k(1 − π_HPC^(−1/k)) − 1`, `k = γ_c/(γ_c−1)` — including the turn at
  **`π* = γ_c^(γ_c/(γ_c−1))` = 3.2467**, where `s_H = 0`. So bleed has **exactly zero**
  first-order HP effect at `π*` and **REVERSES SIGN** below it. The *location* of that
  reversal is rung 41's; what is new is that a **second, independent perturbation sweeps
  through it** — `π*` surfacing a **third** time (rung 41's throttle turn, rung 42's bleed
  sign-reversal), the shape rung 40 used for the slip pattern.
* **NEW — perturbation-independence, and it could have failed.** "Throttle-derived `s_H` ==
  bleed-derived `s_H`" says the HP response depends on `x_H` **alone, regardless of how that
  `x_H` was reached**. It is algebra only on a CPG gas at frozen `f`; on the real gas the HP
  loop reads `(Tt4, Tt25, f)` **separately**, so the collapse is a *measurement*. It holds to
  **≤0.004 absolute** in `s_H` and **0.012 %** in the curve test. That is this rung's
  canonical "two paths, one number".

Like rungs 38–41 this is a **separate entry point** (`TwoSpoolBleedMatcher`); `bleed = 0`
dispatches to rung 39's `match` **bit-for-bit**, and the default `build_turbojet(…).run(…)`
design path is untouched (**bit-for-bit rung 6**).

---

## The model, and where `b` enters

A fraction `b` of the LPC exit flow is extracted between the LPC and the HPC and dumped
overboard. Per unit **inlet** air `ṁ₂` the core carries `(1−b)`:

```
LPC pumps  ṁ₂             LPT expands  ṁ₂(1−b)(1+f)
HPC pumps  ṁ₂(1−b)        HPT expands  ṁ₂(1−b)(1+f)
```

This is the project's first **steady mass EXTRACTION** — the first time mass *leaves* the
flowpath, so the two **compressors** pass different air (`ṁ_LPC = ṁ₂`, `ṁ_HPC = (1−b)ṁ₂`).
Every prior flow change was fuel **addition**, and rung 37's `ṁ_c ≠ ṁ_NGV` was *transient
storage*.

**Stated precisely, because the obvious gloss is wrong.** It is tempting to say "the first
shaft whose compressor and turbine pass different air" — that is **false**: `(1+f)` has made
the LPC pass `ṁ₂` and the LPT pass `ṁ₂(1+f)` since the two-spool engine was built. The novelty
is not that a flow *changes* along the path but that mass *leaves* it, and that it leaves
**between the two compressors**, so the split is on the **LP shaft alone**. That asymmetry is
the whole rung.

`b` enters exactly **three** places:

1. **The LP shaft balance** — `h_c(Tt25) − h_c(Tt2) = η_m(1−b)(1+f)·Δh_LPT`. The LP turbine
   drives its compressor with less air than the compressor pumps, so **`Tt25` falls**. This is
   the one place `b` touches the energy cascade.
2. **The LP face flow referral** — `ṁ₂ = ṁ_core/(1−b)`, so rung 39's `(‡)` picks up an
   explicit `1/(1−b)`:
   ```
   (‡-b)   ṁ_corr,2 = A4·π_b·π_HPC·π_LPC·MFP*·√(Tt2/Tt4) / [(1+f)(1−b)]
   ```
3. **The thrust bookkeeping** — the dumped air was still *captured*, so it carries full ram
   drag and returns no exhaust momentum (see below).

And **not** the fourth. Rung 39's `(†)` refers the HPT-NGV choke to station 25 through
`pt4/pt25 = π_b·π_HPC`, and **both sides of that referral are core flow**:

```
(†)   ṁ_corr,25 = A4·π_b·π_HPC·MFP*·√(Tt25/Tt4)/(1+f)          — carries NO b
```

Nor does the **HP shaft balance** (HPC and HPT both see `(1−b)ṁ₂`, so it cancels and
`Tt3 − Tt25` is bleed-invariant). Nor do the two **turbine pins** `(★-HP)`/`(★-LP)`: they are
ratios of choked MFPs passing the *same* core flow, and bleed is upstream of station 4, so
`τ_HPT`, `τ_LPT`, `Tt45/Tt4`, `Tt5/Tt45` are **untouched**.

**The structural claim, in rung 39's register:** `b` reaches the HP spool **only through the
shared `Tt25`** — never through the HP face's own flow referral. That is the exact analogue of
rung 39's leaf (*"`η_HPC` is a leaf; everything geometric reaches both"*), and it is the reason
`_hp_eta_loop` is reused **verbatim** by `_cascade_bleed`: **its body is `b`-free; its
arguments are not.** A code-level guarantee, not a numerical coincidence.

**What the algebra does NOT settle — both faces carry competing channels.** LP: the explicit
`1/(1−b)` pushes `m_L` **up** while the falling `Tt25` lowers `π_LPC` and pushes it back
**down**. HP: `Tt3 − Tt25` being bleed-invariant makes `τ_HPC = 1 + K/Tt25` **rise**, so `(†)`
has `π_HPC` **up** against `√Tt25` **down**. Every sign below is **measured**.

### Thrust

The bleed air is dumped overboard with no momentum recovery (the conservative reading; a real
duct into the nacelle recovers some). Per unit **inlet** air:

```
F/ṁ₂ = (1−b)·[(1+f)V9 + pressure − V0] − b·V0     TSFC = (1−b)·f / (F/ṁ₂)
```

`performance` on the result is **core**-referenced, so at `b = 0` it is bit-for-bit rung 39's.

---

## A hypothesis, written down and REFUTED

The rung was proposed with the prediction *"bleed protects the LP spool **at the HP spool's
expense**"* — the textbook trade. **It is false**, and the truth is more interesting:

| | `dlnφ_L` | `dlnφ_H` | ratio |
|---|---|---|---|
| design (`π_HPC` = 6.00) | +0.0196 | **+0.0024** | 8.4 |
| `Tt4` = 1100 (4.41) | +0.0211 | +0.0010 | 22 |
| `Tt4` = 900 (3.66) | +0.0220 | +0.0003 | 69 |
| `Tt4` = 800 (3.30) | +0.0225 | +0.00003 | 659 |
| `Tt4` = 750 (3.13) | +0.0228 | **−0.0001** | −232 |

(CPG + flat maps, `db = 0.02`.) The HP spool is **helped** above `π*`, **negligibly hurt**
below it, and in either direction by **one to three orders of magnitude less** than the LP.
The growth of the ratio is the **HP denominator passing through zero** — `dlnφ_L` is nearly
constant (~0.022) across the whole band — not a large numerator. Stated as "infinite
selectivity" it would be an artifact; stated as "**the HP response passes through zero at
`π*`**" it is the inherited closed form doing exactly what it must.

---

## `s_H`, measured under the VALVE (the non-tautological gate)

`s_H ≡ dlnφ_H/dlnx_H` measured by opening the **bleed valve**, against rung 41's closed form
evaluated at the *same* operating `π_HPC` (CPG + flat maps):

| `Tt4` | `π_HPC` | `s_H` measured (bleed) | `s_H` closed form | Δ |
|---|---|---|---|---|
| 1500 | 6.000 | +0.4039 | +0.4023 | +0.0016 |
| 1300 | 5.188 | +0.3138 | +0.3134 | +0.0004 |
| 1100 | 4.406 | +0.2081 | +0.2088 | −0.0007 |
| 1000 | 4.028 | +0.1480 | +0.1494 | −0.0014 |
| 900 | 3.660 | +0.0821 | +0.0842 | −0.0021 |
| 800 | 3.304 | +0.0097 | +0.0124 | −0.0027 |
| 750 | 3.130 | −0.0294 | −0.0263 | −0.0031 |
| 620 | 2.694 | −0.1410 | −0.1369 | −0.0041 |

Two different perturbations — the **throttle** (rung 41) and the **valve** (rung 42) — one
sensitivity, over a 2.4:1 throttle range. On shaped maps and on the variable-`cp` gas the
agreement loosens (rung 41's own disclaimer: `(★)` is a CPG + flat-map statement); the sign
and the ordering survive.

### `π*` a THIRD time

Fine bracket of the `dφ_H/db` sign change (CPG + flat):

| `Tt4` | `π_HPC` | `π_HPC − π*` | `dlnφ_H` |
|---|---|---|---|
| 790 | 3.26878 | **+0.0220** | **+7.0e-6** |
| 780 | 3.23391 | **−0.0128** | **−2.0e-5** |

The crossing interpolates to `π_HPC ≈ 3.260` against `π* = 3.24675` — **+0.40 %**, which is
the *same* residual rung 41's own kill test isolated as the **fuel fraction** (+0.44 %, killed
by driving `f → 0`). Same number, same cause, reached by a different perturbation.

---

## Self-targeting — stated in φ-space, NOT in relative margin

The tempting statement is "the relative surge-margin gain grows as you throttle down"
(`SM_L`: +23 % at design → +53 % at `Tt4` = 850). **That is confounded**: the *absolute*
`ΔSM_L` **shrinks** (0.056 → 0.018 pp); the relative figure grows only because the `SM_L`
base is collapsing. Resting the claim there would repeat this project's own rung-41 lesson
(*matching a denominator is not a controlled comparison*).

The defensible statement is in rung 41's **surge-proximity** currency, the flow coefficient:

| `Tt4` | `φ_L` | gap `φ_L − φ_surge` | `Δφ_L` (b=0.10) | fraction closed | `Δφ_H` | fraction (HP) |
|---|---|---|---|---|---|---|
| 1500 | 1.0000 | 0.4500 | **+0.0776** | 17.2 % | +0.00806 | 1.79 % |
| 1300 | 0.9059 | 0.3559 | **+0.0787** | 22.1 % | +0.00591 | 1.39 % |
| 1100 | 0.8153 | 0.2653 | **+0.0785** | 29.6 % | +0.00367 | 0.91 % |
| 950 | 0.7532 | 0.2032 | **+0.0776** | 38.2 % | +0.00202 | 0.52 % |
| 900 | 0.7342 | 0.1842 | **+0.0773** | 42.0 % | +0.00149 | 0.39 % |
| 850 | 0.7163 | 0.1663 | **+0.0769** | 46.3 % | +0.00097 | 0.25 % |

(`flow/press` shapes, `φ_surge` = 0.55.) **`Δφ_L` is nearly constant** — ±1 % over a 1.76:1
throttle — while `Δφ_H` **collapses ×8** toward its zero at `π*`. The valve delivers a fixed
absolute increment into a **shrinking** LP gap, so the fraction of the surge-proximity gap it
closes **rises 17 % → 46 %** on the LP spool and **falls 1.8 % → 0.25 %** on the HP. That is
the honest sense in which the device is **self-targeting**: it concentrates on the spool
rung 41 showed is exposed, and does so more completely exactly where that spool is thin.
Robust across shapes × three imposed floors; magnitudes disclaimed (they ride on the maps and
on `φ_surge`, rung 36's doubled cost, inherited).

**A surge-SURVIVAL claim is deliberately NOT made.** That is `E0`-vs-`SM_N` territory
(rungs 36/41) and requires the transient, which is out of scope here — see Concessions.

---

## The trade, and the envelope

At `b = 0.10`, fixed `Tt4` (`flow/press` shapes, CPG gas; the `thermally_perfect` gas gives
−10.1 %/−11.4 %/−14.0 % over the same points — same signs, same growth):

| `Tt4` | thrust | TSFC |
|---|---|---|
| 1500 | −10.0 % | +6.3 % |
| 1100 | −11.2 % | +9.2 % |
| 900 | −13.6 % | +12.9 % |
| 850 | −14.7 % | +14.6 % |

The device gets **more selective AND more expensive** as you throttle down — the two move
together, which is precisely why bleed is *scheduled* and not simply left open.

And bleed lowers `π_LPC`, hence `pt4`, so it **shrinks the choked envelope**: the lowest
runnable `Tt4` rises `605 → 610 → 620 → 630 K` for `b = 0 → 0.05 → 0.10 → 0.15` (CPG, flat).
The inherited nozzle-choked guard bites sooner with the valve open; it **flags**, it does not
lie.

---

## Reduce-to-prior contract (the spine)

**Exact dispatch** (rungs 38/39/40's contract): `bleed == 0.0` forwards `match` to rung 39's
`TwoSpoolMapMatcher.match` **verbatim** — the bleed cascade is never entered, so a bleed
matcher with the valve shut is rung 39 **bit-for-bit** (`==`, verified on the fast gas *and*
the reacting gas). Rung 39's `_cascade_map` and `_lp_eta_loop` are left **literally
unchanged** (the rung-33/39/40 discipline), so the rung-39, rung-40 and rung-41 suites still
witness them. The default single-spool design run is untouched ⇒ **bit-for-bit rung 6**.

### Why rung 42 carries no independent bare-math gate (a deliberate break in the streak)

Rungs 38, 39 and 40 each ship an **independent bare-math CPG cascade**, for a stated reason:
their reduce path never enters the new code, so nothing else ties the new numbers down. Rung
42 is in exactly that position — `bleed = 0` dispatches away, so the `b > 0` cascade has **no
reduce anchor of its own** — and it deliberately ships **without** one. The reason is that
the load-bearing content is already anchored by other means:

* **The HP side is anchored transitively, twice.** Gate 2 lands the bled point **on the
  `b = 0` HP running line** (to 0.01–0.016 %), and that line is exactly what rung 39's own
  bare-math gate ties down; gate 3 then pins the HP *response* to rung 41's **closed form**,
  which is independent arithmetic, not this solver.
* **Every LP-side `b > 0` magnitude is disclaimed** — the `π_LPC` drop, the thrust penalty,
  the `+8–12 %` displacement all ride on `b` and on the representative maps.
* **The one load-bearing LP claim is a SHAPE, not a magnitude.** "`Δφ_L` is near-constant
  while `Δφ_H` collapses" and "`x_L` is exactly invariant" survive a uniform magnitude error;
  the second is an **identity in the inputs**, checkable by inspection.

Recorded here rather than left implicit, because it *is* a break in a deliberate streak. A
~30-line bare-math `b > 0` cascade reproducing `(π_LPC, π_HPC, φ_L, φ_H)` would close it, and
is the natural first addition if any `b > 0` **magnitude** is ever promoted to load-bearing.

---

## Verification gates (`tests/test_rung42.py`)

1. **REDUCE — exact dispatch.** `bleed=0` ⇒ rung 39's `match` bit-for-bit (`==` on `π_LPC`,
   `π_HPC`, both `φ`, both `η`, `ṁ`, thrust) across shapes × throttles, on the fast gas and
   the reacting gas.
2. **THE ASYMMETRY (the rung).** At fixed `(flight, Tt4)`: `x_L` is **exactly** bleed-invariant
   (`==`) while `φ_L` moves > 5 % — pure off-line displacement; and the bled HP point,
   compared to the `b = 0` point at the **same `x_H`**, matches in `φ_H` to < 0.05 % — a
   > 100× contrast. Plus the mass-extraction identity `ṁ_core == (1−b)·ṁ_air`.
3. **PERTURBATION-INDEPENDENCE (non-tautological).** Bleed-derived `s_H` equals rung 41's
   closed form to < 0.01 absolute at every point of the CPG + flat throttle band — two
   perturbations, one sensitivity. (The reacting/TPG drift is measured and disclaimed, not
   gated.)
4. **`π*` A THIRD TIME.** `dφ_H/db` **changes sign** along the choked band, and the crossing
   **brackets** `π* = γ_c^(γ_c/(γ_c−1))` within the fuel-fraction residual. Existence + sign +
   bracket only; the exact crossing is **disclaimed** (it rides on `f`, on the map shape and on
   the gas, exactly as rung 41's turn does).
5. **SELF-TARGETING in φ-space.** `Δφ_L` is near-constant (spread < 10 %) while `Δφ_H` falls
   ≥ 5× over the same band; hence the fraction of `(φ_op − φ_surge)` closed is **monotone
   rising** on LP and **monotone falling** on HP, across ≥ 2 shapes × ≥ 3 floors. The gate
   asserts this **in φ-space** and deliberately does **not** gate the relative-`SM` version.
6. **THE TRADE + THE ENVELOPE.** Thrust falls and TSFC rises monotonically in `b`, the thrust
   penalty **grows** with throttle-down, and the lowest choked `Tt4` **rises** with `b`.
7. **THE REFUTED HYPOTHESIS, kept visible.** `dφ_H/db > 0` at the design point — "bleed
   penalizes the HP spool" is **false** there (rung 40's convention: a refuted hypothesis is
   asserted, not quietly dropped).
8. **CYCLE UNTOUCHED.** The default `build_turbojet(…).run(…)` design point is bit-for-bit
   rung 6.

---

## Concessions

* **`b` is an imposed device setting.** It is a *valve position*, not a fudge factor, and the
  reduce at `b = 0` is exact — but every **magnitude** here rides on it, and on the two
  representative maps and (for the margin numbers) the two imposed `φ_surge` floors inherited
  from rungs 36/41. Only the **asymmetry**, the **perturbation-independence**, the **sign
  reversal at `π*`**, the **near-constancy of `Δφ_L`** and the **directions** of the trade are
  load-bearing.
* **A fixed `b`, not a schedule.** A real handling-bleed valve is scheduled on corrected
  speed; rung 42 opens it to a constant fraction and reads the steady point. The schedule
  `b(n_L)` is a control-design question this rung supplies the plant for.
* **Bleed moves `φ_op`, not `φ_surge`.** The variable-stator half of the rung-36/41 seam —
  the device that moves the **stall line itself** — is untouched and still open.
* **Overboard dump, zero recovery.** The conservative thrust reading; a duct into the
  nacelle/bypass would recover part of it. No bleed-duct pressure loss is modelled (the valve
  is a mass sink, not a flow network).
* **Steady only.** Rung 40's two-shaft transient does not read `bleed`, so no surge-*survival*
  claim (`E0` vs `SM_N`, rung 36's currency) is made — bleed *during* an acceleration is the
  natural follow-on, and it needs the transient two-spool surge line rung 41 already deferred.
* **Fully-choked branch, both NGVs choked, one `η_m`, no bypass, isentropic knobs** —
  all inherited from rungs 38–41. The envelope shrinkage is reported, the LP subsonic branch
  is still a rung-33-shaped follow-on.
* **Customer/cooling bleed is NOT modelled.** Only the interstage handling bleed at station 25.
  HPC-exit bleed (station 3) returns to the flowpath downstream and is a different sink with a
  different signature.

---

## Anchor

`docs/plans/rung42-anchor-interstage-bleed.md` — the measured tables above.
