# The φ-RATE LIMITER — investigated, NEGATIVE (not shipped, not a rung)

Rung 60's named next seam:

> *"**A limiter that RELOCATES rather than FLOORS the protected variable.** Rung 60 says the
> composable legs are the ones that move a minimum's location; rung 49's family all set its
> value. The object that would settle it is a `φ`-*rate* limiter — a leg that caps `dφ/ds`
> rather than `φ` — which arrests the descent without fixing where it stops, and so should
> compose where every floor cannot. It is new plant (a derivative of the protected variable is
> not available inside a derivative call without a state), and it is the first leg that would
> test the relocate-vs-pin law rather than illustrate it."*

**Attacked, and it fails — but not for the reason rung 60 gave.** The derivative *is* available
inside the derivative call, with no state and no new constant (§1), so the plant objection
falls in one line. What kills the leg is a **sign**: no fuel-side leg can arrest a `φ` descent
at all, because fuel's authority over `φ` **inverts between the level and the derivative**.

**VERDICT: NEGATIVE, and the seam is discharged by a proof rather than by a lever.** The
relocating limiter rung 60 asked for is not hard to build on the fuel path — it is
**unrealisable** there. Cutting fuel drives the descent rate monotonically **away** from any
arresting target, on both spools, at every point on the descent, over the entire range in which
the model is defined (§3).

One result survives independently and is recorded here because it lives nowhere else:
**§4's bound on rung 49's monotonicity**, which six shipped rungs rest on. It is gated by
`tests/test_phi_rate_limiter_negative.py` — the only negative in the project that carries a
test, because it is the only one that BOUNDS a load-bearing claim rather than merely failing to
extend one.

---

## 1. THE PLANT OBJECTION FALLS — the derivative is already in the derivative call

Along the march `φ` is evaluated at `(ν_L, ν_H, m_f)`, so the total derivative is a chain rule

```
    dφ/ds  =  φ_νL·ν̇_L  +  φ_νH·ν̇_H  +  φ_mf·ṁ_f                              ...(1)
              └──── the STATE term ────┘   └─ the FUEL term ─┘
```

and `ν̇_L, ν̇_H` are **exactly what `der(...)` already computes** — they are its return value.
The partials come from the same finite differences `equilibrium_fuel` and `jacobian` already
take. So rung 60's "needs a state" is **conditional, not structural**; it depends only on where
`ṁ_f` comes from:

| | source of `ṁ_f` | state? | status |
|---|---|---|---|
| **A** | the **schedule** — a pure function of `s` | **none** | one equation in one unknown `m_f`, rung 49's `_surge_fuel` shape |
| **B** | the **applied** fuel | a third state | rung 60's stated plant |

**B is refused on prior work, not re-measured.** Taking `ṁ_f` from the applied fuel makes the
leg a fuel-command rate limit, and `docs/both-edges-limiter-negative.md` § already established
that a fuel-command rate limit is **rung 44's ramp-rate lever BY IDENTITY** — not a new
instrument. So the entire seam rests on whether **A**'s bracket has a root.

This is the same shape as rung 52 refuting rung 51's deferral reason: the stated obstacle was
a property of one formulation, not of the object.

## 2. IT DOES NOT — cutting fuel STEEPENS the descent

Bare fuel ramp, the rungs 57–60 machine (`r = 0.5`, `ds = 0.005`, `LO/HI = 1000/1400 K`).
First, (1) is validated against a finite difference of the marched trajectory, so the analytic
rate is not itself the artifact:

| `s` | `φ_lp` | state_L | state_H | fuel | **total** | fd check |
|---|---|---|---|---|---|---|
| 0.000 | 0.773116 | −0.00000 | −0.00000 | −0.33346 | **−0.33346** | −0.32967 |
| 0.100 | 0.747332 | +0.00426 | +0.11196 | −0.29925 | **−0.18303** | −0.17936 |
| 0.200 | 0.736157 | +0.01283 | +0.21130 | −0.26718 | **−0.04305** | −0.03977 |
| 0.300 | 0.738085 | +0.02400 | +0.29178 | −0.23780 | **+0.07798** | +0.08071 |

(Agreement to ~3 dp inside the ramp. The `s = 0.5` row is omitted: it sits on the ramp corner,
where the schedule slope is discontinuous and a centred check is meaningless.)

Now the controllability question the bracket actually asks — sweep the **applied** fuel down at
frozen state, exactly as an Illinois solve would, at `s = 0.100`:

| `w`/`m_f` | `Tt4` | state_L | state_H | fuel | **dφ/ds** |
|---|---|---|---|---|---|
| 1.0000 | 1182.9 | +0.00426 | +0.11196 | −0.29925 | **−0.18303** |
| 0.9127 | 1108.8 | +0.00194 | +0.06382 | −0.31016 | **−0.24440** |
| 0.7837 | 1001.9 | −0.00008 | −0.00375 | −0.32824 | **−0.33207** |

**Monotone the wrong way.** The mechanism is one line: cutting fuel cools `Tt4`, which collapses
shaft acceleration, which kills the **state** term — and the state term is the only term that
LIFTS `φ`. The fuel term worsens slightly too (`φ_mf` steepens at lower fuel), so both
contributions push the same direction. To arrest the rate the leg would have to **add** fuel,
which is the opposite of what a surge-protection leg does.

## 3. THE TWO CHECKS

**CHECK 1 — no spool split.** Rungs 49 and 56 both found per-spool splits where none was
expected, so this was measured rather than assumed:

| spool | `s`=0.100 | `s`=0.200 | `s`=0.300 | verdict |
|---|---|---|---|---|
| `φ_lp` | −0.183 → −0.332 | −0.043 → −0.229 | +0.078 → −0.142 | STEEPENS |
| `φ_hp` | −0.324 → −0.463 | −0.192 → −0.360 | −0.080 → −0.272 | STEEPENS |

(each cell: `dφ/ds` at full scheduled fuel → at `0.97⁸` of it.) The claim is therefore **"no
fuel-side leg"**, not "not on this spool."

**CHECK 2 — the leg, actually armed and marched.** Formulation A built as a real min-select leg
in a local subclass **outside the repo** (rung 49's precedent — `engine.py` is untouched by this
investigation), using rung 49's exact bracket structure with the floor moved onto the
derivative, and demanding merely **half** the current descent rate:

| `s` | rate@sched  LP / HP | `rate_min`  LP / HP | LP | HP |
|---|---|---|---|---|
| 0.050 | −0.258 / −0.396 | −0.129 / −0.198 | **NO ROOT** | **NO ROOT** |
| 0.100 | −0.183 / −0.324 | −0.092 / −0.162 | **NO ROOT** | **NO ROOT** |
| 0.150 | −0.111 / −0.256 | −0.056 / −0.128 | **NO ROOT** | **NO ROOT** |
| 0.200 | −0.043 / −0.192 | −0.022 / −0.096 | **NO ROOT** | **NO ROOT** |
| 0.250 | +0.020 / −0.133 | — / −0.067 | dormant | **NO ROOT** |
| 0.300 | +0.078 / −0.080 | — / −0.040 | dormant | **NO ROOT** |

`NO ROOT` = the bracket search ran its full 60 steps without the descent ever becoming
shallower. LP's dormant rows are past its own `φ` minimum, where the rate is already positive
and the leg is not consulted.

**How deep the search actually reaches, stated honestly.** `0.9⁶⁰` is 0.18 % of scheduled flow,
but that number is meaningless here: most of those trials throw, because `_instant_fuel` leaves
the modeled speed-line region long before it. The evaluable walk is much shorter — and over it
the rate moves monotonically **AWAY** from any arresting target, which is a stronger statement
than the mere absence of a crossing:

| sample | evaluable cuts | deepest `w`/`m_f` | `dφ/ds`  first → deepest | factor |
|---|---|---|---|---|
| `φ_lp`, s=0.05 | 13 / 60 | 0.2542 | −0.25763 → −0.66702 | 2.6× |
| `φ_lp`, s=0.10 | 14 / 60 | 0.2288 | −0.18303 → −0.66443 | 3.6× |
| `φ_lp`, s=0.20 | 16 / 60 | 0.1853 | −0.04305 → −0.66262 | 15.4× |
| `φ_hp`, s=0.05 | 13 / 60 | 0.2542 | −0.39565 → −0.80393 | 2.0× |
| `φ_hp`, s=0.10 | 14 / 60 | 0.2288 | −0.32449 → −0.79670 | 2.5× |
| `φ_hp`, s=0.20 | 16 / 60 | 0.1853 | −0.19203 → −0.77848 | 4.1× |

So the claim is **not** "no root down to flame-out" — it is "**the rate diverges from the target
across the whole domain in which the plant is defined**," roughly 19–25 % of scheduled flow.
Below that the machine is off its modeled speed lines and the question is not well-posed. This
correction was forced by the gate's own coverage assertion (§7), which is the reason that
assertion exists.

## 4. WHAT SURVIVES — the bound on rung 49's monotonicity

> **Fuel's authority over `φ` INVERTS between the level and the derivative.**

Rung 49's bracket — *"`φ` falls monotonically with fuel at fixed spool speeds, so cutting fuel
RAISES `φ`"* — is the load-bearing monotonicity under **six shipped rungs** (49, 50, 51, 52, 58,
60). It is **sound, and it is a LEVEL property.** It reverses one derivative up, because the
level and the rate answer through *different channels*:

```
    level:  more fuel -> hotter Tt4 -> less choked-NGV corrected capacity -> less φ   [DIRECT]
    rate:   less fuel -> cooler Tt4 -> less shaft acceleration -> the STATE term dies [INDIRECT]
```

The direct channel is instantaneous; the indirect one runs through the shaft ODE. Nothing in
rungs 49–60 is wrong — every one of them floors a level — but the bracket **must not be
extended to a derivative**, and that boundary was not previously stated anywhere.

Three consequences:

1. **Rung 60's seam is discharged.** The relocating limiter it asked for is unreachable on the
   fuel path.
2. **Rung 60's stated reason is corrected.** The blocker is the SIGN, not the plant (§1).
3. **Rung 60's composability law keeps its positive half only.** Location-movers compose, and
   the location-mover that works is the **STATOR** (rungs 53/57), already shipped. No fuel-side
   leg can join it — which is a sharper statement of rung 58's "two levers do not superpose"
   than rung 58 could make, since it now has a reason rather than a measurement.

## 5. WHY THE OBVIOUS THIRD OBJECT DOES NOT RESCUE IT

A rate-**TRIGGERED** leg — engage while `|dφ/ds| > R`, clip by an independently-set depth —
dodges §2 entirely, since the rate is only *read* and never *held*. It was considered and
refused for two reasons, either of which is sufficient:

- **The clip law is under-determined, and every candidate is walked ground.** A fixed
  fractional clip mints a **new unanchored constant** carrying the entire credit magnitude —
  against this family's habit of inheriting disclaimed constants (`phi_surge`, `m`) rather than
  minting fresh ones. Clipping to a `φ` floor while the trigger is armed is **rung 49 with a
  rate-gated engagement edge**, which PINS `φ` when it binds and therefore inherits rung 60's
  tautology verbatim. Clipping to `dφ/ds = −R` **is** formulation A. There is no fourth option,
  so the leg cannot answer rung 60's own question: *when it binds, what does the composite's
  second difference reduce to?*
- **The trigger releases AT the minimum, by construction.** `dφ/ds = 0` **is** the `φ` minimum.
  §2's table crosses zero between `s = 0.2` and `s = 0.3`, and the bare `min φ_lp` is at
  `s = 0.235`. So the release edge coincides with the minimum it was meant to relocate — rung
  50's release-edge-relocates-the-minimum result arriving through a different door, and the
  same trap `docs/both-edges-limiter-negative.md` closed for the whole `pt3`-filter family.

## 6. SCOPE / CONCESSIONS

- **One machine, one ramp rate, one map pair.** The sign is large and monotone at every point on
  both spools, so the VERDICT is robust; the MAGNITUDES ride on the disclaimed rung-36
  `phi_surge` and the representative maps, as everywhere in this family.
- **Formulation B is refused on prior work, not re-measured** (§1). If
  `both-edges-limiter-negative.md`'s identity is ever revisited, B revives with it.
- **§4's inversion is a statement about THIS plant's channels**, not a theorem about limiters.
  It needs the `Tt4` → shaft-acceleration path to dominate, which is exactly what rungs 34–43
  built; a plant where fuel reaches `φ` without moving shaft power would not show it.
- The investigation touched **no production code**. `engine.py` is byte-unchanged; the armed leg
  lived in a local subclass outside the repo, and the gate in
  `tests/test_phi_rate_limiter_negative.py` builds its own.

## 7. WHY THIS ONE CARRIES A TEST

The other six negatives record an attack that did not extend the ladder, and nothing downstream
depends on them. This one **bounds a claim six shipped rungs rest on**. A future change to the
fuel → `Tt4` → shaft-acceleration channel could silently flip §2's sign and make the bound wrong
without any per-rung gate noticing — none of them looks at a derivative. The test pins the level
monotonicity (rung 49's, which must HOLD) and the rate inversion (which must also hold) in the
same file, so the pair cannot drift apart unobserved.
