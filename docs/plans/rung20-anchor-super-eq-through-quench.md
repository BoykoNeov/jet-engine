# Rung-20 anchor — super-equilibrium O *through the quench*

The numbers-before-code record for rung 20: the effective super-eq-O lift on the **finite-quench** NO
fields, instrumented **before** the lesson was written (project discipline; the advisor's explicit gate
was "don't write the lesson until the effective-lift number is on screen, because it probably inverts
your headline"). Instrumentation: `M:\claud_projects\temp\rung20\instrument_lift.py` and `smoke.py`.

No new external constants — rung 20 reuses the rung-19 Westenberg `m(T)` ratio (`docs/plans/rung19-
anchor-superequilibrium-prompt.md §1a`). What is new is the **trajectory-weighted** behaviour of that
ratio through the cooling quench, which had to be measured, not assumed.

## 1. The design point (the rung-17 rich RQL primary)
`M0=0.85, T0=250 K, p0=50 kPa; πc=10, Tt4=1500 K`; equilibrium engine ⇒ `far=0.02718, Tt3=584 K,
p=747 kPa`. Zoned rich primary `φ_p=1.5 ⇒ T_p=2110 K`, `α=0.268`. Jet `J=225, C_e=0.20, shape_n=2 ⇒
τ_q=4.44e-4 s`. Per-pocket `PocketQuenchPDF(S=0.0625, C_opt=2.5, k_g=0.3, g_max=0.3, τ_res=2.5e-3,
b_u=3.0)`. Quench grids `ngrid=24, nsteps=200` (direction, not digits).

## 2. Where the NO is made vs where `m` is large (the inversion, measured)
The quench trajectory (β = dilution fraction), `m` raw vs floored at 1500 K, and the formation proxy
`R1∝[O]·[N2]`:

| β | T (K) | m(T) | [O] (mol/m³) | R1∝[O]·[N2] |
|---|---|---|---|---|
| 0.087 | 2351 | 1.173 | 7.55e-4 | 2.0e-2 |
| **0.174** | **2448** | **1.142** | 1.10e-2 | **2.9e-1** ← peak formation |
| 0.261 | 2327 | 1.181 | 1.31e-2 | 3.7e-1 |
| 0.348 | 2178 | 1.242 | 7.57e-3 | 2.3e-1 |
| 0.522 | 1928 | 1.393 | 1.77e-3 | 6.2e-2 |
| 0.696 | 1742 | 1.567 | 4.08e-4 | 1.6e-2 |
| 0.870 | 1602 | 1.760 | 1.02e-4 | 4.3e-3 |
| 0.957 | 1544 | 1.861 | 5.29e-5 | 2.3e-3 |

The NO mass is made at **β≈0.17–0.44, T≈2180–2448 K** (the hot stoich crossing), where `m` sits at its
**minimum ~1.14–1.24**. The cool tail (β>0.6) carries large `m` (→1.86) but formation ~100× smaller —
and there NO is super-eq (`a>1`, `(1−a²)<0`, i.e. destruction). So the effective lift is a
**formation-weighted average of `m`, floored by `m(T_peak)`**. The trajectory bottoms at `T≈1517 K`, so
the 1500 K floor **does not bind at this design point** — it is a safety rail for colder ones.

## 3. The measured lift (the headline number)
| field | eq-O | super-eq-O | factor |
|---|---|---|---|
| `ei_no_quenched` (bulk) | 0.2712 | 0.3176 | **×1.171** |
| `ei_no_pocket_quench` | 0.917 | 1.067 | **×1.164** |
| primary `ei_no` (rung 19) | 1.311e-3 | 1.673e-3 | **×1.276** = `m(2110)` |
| `m(T_peak=2448)` | — | — | 1.142 |

**The quench lift (×1.17) < the primary lift (×1.28)** — the quench crossing (2448 K) is hotter than the
flame (2110 K), and `m` is smallest where hottest. The naive "strongest on the cooling pocket" headline
is **inverted**: modest, peak-concentrated, and *below* the primary.

## 4. The rung-17 clamp (the certified spine)
| margin | eq-O | super-eq-O |
|---|---|---|
| `a_mixed_out` | 0.0158 | 0.0202 (stays ≪1, dormant) |
| `a_bulk_quench` | 3.27 | 3.83 |
| `a_pocket` | 11.06 | 12.87 |
| denominator `x_no_e_exit` | 2.1147e-06 | **2.1147e-06 (bit-identical)** |
| per-pocket `max_a_quench` | 0.72 | 0.81 (still <1 — clamp dormant at station 4) |

Numerators rise ~×1.17; the thermodynamic denominator `x_no_e(T9)=Kp_NO·√(x_N2·x_O2)` is **untouched**
by the O-atom closure ⇒ every margin rises by a bounded factor, the rung-17 ordering + `a_mixed<1`
survive, and the clamp still does **not** fire at the burner (`max_a<1`).

## 5. The floor hazard (the advisor's implementation catch)
`m(T)=A·T·exp(B/T)`, `B=θ1−θ2≈3967 K`, **diverges** as `T→0`: raw `m(1200 K)=3.02 > 2`. The T-floor maps
it onto `m(1500 K)=1.95 < 2`, keeping the lifted quench inside the standing `1≤m≤2` trajectory assert.
Without the floor a colder design point would silently inject a divergent lift.

## 6. Honest scope (carried from rung 19 §5)
- The super-eq **ratio** `m(T)` is Westenberg's fitted partial-eq/eq ratio — semi-empirical (a
  full-equilibrium pool cannot self-yield super-eq O). So the lifted `a` is **better-justified but not
  pinned**.
- **Prompt** rides the quench as an **invariant** per-kg-fuel EI (`ei_no_quenched_total`), kept **out**
  of the clamp `a` because its magnitude is imposed. Injecting prompt moles into the cooling chemistry
  would be false precision.
- The **ideal-bell composition integrals** (rung 13/15-term2/18) are **deliberately left on
  equilibrium O** (forbidden to combine), staying documented lower bounds.
- The **"per-pocket clamp fires AT the burner"** seam is **not** discharged: super-eq O speeds
  formation but does not raise `[NO]_e`, so `max_a<1` here — the lever is a slow-enough freeze, a
  separate seam.
