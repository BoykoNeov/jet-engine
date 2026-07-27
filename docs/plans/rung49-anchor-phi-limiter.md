# Rung 49 anchor — the φ / surge-margin feedback limiter

Verified data behind `docs/rung49-spec.md`. Probes: `M:\claud_projects\temp\rung49-probe\` —
`phi_leg.py` (the leg built as a LOCAL subclass, outside the repo, while the signs were still
unknown), `probe1_bare.py`, `probe2_sweep.py` (the three signs), `probe3_release.py` (engagement
vs release edge), `probe4_robust.py` (`ds` / shape / `ρ` / `r`), `probe5_discriminator.py` (**the
deciding experiment**), `probe6_farside.py`, `verify_shipped.py` (the shipped leg reproduces the
probe **exactly**), `verify_test_config.py` (the numbers at the test file's own config).

## Plant

CPG gas (`γ_c`=1.4, `cp_c`=1004, `γ_t`=1.3, `cp_t`=1239, `hPR`=42.8 MJ/kg); two-spool
`π_LPC`=3, `π_HPC`=6, `Tt4`=1500 K design; `FLIGHT` = (250 K, 50 kPa, M0 0.85); maps
`LP_SHAPED(a=.20,b=.05,σ=.1,l=.7)`, `HP_SHAPED(a=.08,b=.15,σ=.1,l=1.0)`; accel 1000→1400 K.

## The bare march (the reference every relief is differenced against)

| | `ds`=0.02 | `ds`=0.01 | `ds`=0.005 |
|---|---|---|---|
| LP min φ @ `s_lp*` | 0.735466 @ 0.240 | 0.735448 @ 0.230 | 0.735442 @ 0.235 |
| HP min φ @ `s_hp*` | 0.861199 @ 0.400 | 0.861169 @ 0.390 | 0.861169 @ 0.390 |

LP start φ = 0.773116; HP start φ = 0.943343; `nu_hp` at settle (`s_settle`=4) = 0.959060.

`r`=0.15: `s_lp*` = `s_hp*` = 0.150 (both at the ramp end). `r`=2.0: LP 0.760927 @ 0.310,
HP 0.910688 @ 0.650 — `s_hp*` and the ramp end **3.1× apart**, which is what makes probe 5 work.

## LP-watching, `r`=0.5, `ds`=0.01, `s_settle`=4 (spec § 1, 2)

| `φ_lim` | `s_eng` | `s_rel` | hold err | `relief_lp` | `relief_hp` | `s@min φ_hp` | fuel removed | `nu_hp_end` |
|---|---|---|---|---|---|---|---|---|
| 0.7650 | 0.030 | 0.910 | 7.8e-16 | +0.029552 | −0.000231 | 0.920 | 0.005602 | 0.959048 |
| 0.7550 | 0.070 | 0.600 | 6.7e-16 | +0.019552 | −0.007977 | 0.610 | 0.001879 | 0.959057 |
| 0.7500 | 0.090 | 0.520 | 1.0e-15 | +0.014552 | −0.011317 | 0.530 | 0.000983 | 0.959059 |
| 0.7450 | 0.120 | 0.440 | 1.6e-15 | +0.009552 | −0.009536 | 0.450 | 0.000395 | 0.959059 |
| 0.7400 | 0.150 | 0.350 | 1.0e-15 | +0.004552 | −0.002741 | 0.380 | 0.000095 | 0.959060 |
| 0.7370 | 0.190 | 0.290 | 7.8e-16 | +0.001552 | −0.000414 | 0.390 | 0.000015 | 0.959060 |

`hold err` = max |φ_watched − φ_lim| over the engaged window: **1e-15, the sliding mode**.
`s@min φ_hp` is **one grid step after `s_rel`** in all six rows — the mechanism.
Bare `nu_hp_end` = 0.959060: the endpoint is unmoved to ≤ 1.2e-5 everywhere.

## HP-watching, same config (spec § 5) — rung 48's crossing, reproduced

| `φ_lim` | `s_eng` | vs `s_lp*`=0.230 | `relief_hp` (watched) | `relief_lp` (other) | forecast `min_{s≤s_eng} φ_bare − min φ_bare` |
|---|---|---|---|---|---|
| 0.9200 | 0.060 | upstream | +0.058831 | +0.021921 | +0.020392 |
| 0.9000 | 0.120 | upstream | +0.038831 | +0.010127 | +0.008516 |
| 0.8800 | 0.200 | upstream | +0.018831 | +0.001205 | +0.000709 |
| 0.8700 | 0.250 | **downstream** | +0.008831 | **0.000000** | **0** |
| 0.8650 | 0.300 | downstream | +0.003831 | **0.000000** | **0** |
| 0.8620 | 0.350 | downstream | +0.000831 | **0.000000** | **0** |

The watched column is the definitional identity `φ_lim − 0.861169` (0.9200 → 0.058831 ✓).

## THE DISCRIMINATOR — `r`=2.0, LP-watching, `ds`=0.01 (spec § 3)

`s_hp*` = 0.650, ramp end = 2.0.

| `φ_lim` | `s_eng` | `s_rel` | `s_rel/s_hp*` | `s_rel/r` | `relief_lp` | `relief_hp` |
|---|---|---|---|---|---|---|
| 0.7615 | 0.240 | 0.390 | 0.60 | 0.20 | +0.000573 | −0.000134 |
| 0.7630 | 0.180 | 0.510 | 0.78 | 0.26 | +0.002073 | −0.001412 |
| 0.7650 | 0.130 | 0.670 | **1.03** | 0.34 | +0.004073 | **−0.005810** |
| 0.7670 | 0.090 | 0.840 | 1.29 | 0.42 | +0.006073 | −0.011389 |
| 0.7690 | 0.060 | 1.070 | 1.65 | 0.54 | +0.008073 | −0.018629 |
| 0.7710 | 0.030 | 1.450 | 2.23 | 0.73 | +0.010073 | −0.029802 |
| 0.7725 | 0.010 | 2.110 | 3.25 | **1.05** | +0.011573 | **−0.045059** |

**8× larger at `s_rel≈r` than at `s_rel≈s_hp*`**, monotone in `s_rel` straight through `s_hp*`.
Far side (same `r`): `s_rel/r` = 1.05 / 1.21 / 1.37 ⇒ −0.045059 / −0.044150 / −0.043825 — the
debit peaks at the ramp end and decays past it. (At `r`=0.5 the decay is complete by
`s_rel/r`=1.82: −0.000231.)

## The sign flip — `r`=0.15, LP-watching, `ds`=0.01 (spec § 4)

Releases at `s_rel` = 0.37 … 0.58, i.e. `s_rel/r` = 2.5 … 3.9 — far past the ramp.

| `φ_lim` | 0.7550 | 0.7500 | 0.7450 | 0.7400 |
|---|---|---|---|---|
| `relief_lp` | +0.066564 | +0.061564 | +0.056564 | +0.051564 |
| `relief_hp` | **+0.051699** | **+0.047331** | **+0.043668** | **+0.039740** |

## Robustness (spec § 6, gate 12)

`relief_hp` at `φ_lim`=0.7500, `r`=0.5: **`ds`** 0.02 / 0.01 / 0.005 ⇒ −0.010403 / −0.011317 /
−0.011744. **`ρ`** 0.25 / 1 / 4 ⇒ −0.015259 / −0.011317 / −0.010271 (`relief_lp` +0.011505 /
+0.014552 / +0.015343 — the split holds on every one).

## The basin asymmetry (spec § 5, gate 9b)

Within 0.005 of its own bare minimum: **LP over `s ∈ [0.150, 0.320]`, HP over `[0.290, 0.500]`.**
A release edge is structurally late, so it lands inside the HP basin and past the LP one.

## The honest boundary (spec § 7)

FLAT LP map (`hp-only` shape), floors 0.7400–0.7550: `s_eng` = 0.000 for every one, and
`nu_hp_end` collapses to **0.7366 against bare 0.9589**. The accel does not complete; the leg
has degenerated into rung 44's ramp-rate lever. Structurally rung 48's `m → 0` corner.

## Test-config numbers (`ds`=0.02, `s_settle`=2.0 — what `tests/test_rung49.py` reads)

LP-watching: (0.7550, 0.7500, 0.7450, 0.7400) ⇒ `s_eng` 0.080/0.100/0.120/0.160,
`s_rel` 0.600/0.520/0.440/0.340, `relief_hp` −0.006490/−0.010403/−0.008887/−0.002765.
HP-watching: (0.9000, 0.8800, 0.8700, 0.8650) ⇒ `s_eng` 0.120/0.200/0.260/0.300,
`relief_lp` +0.010275/+0.001818/**0.000000**/**0.000000** (`s_lp*` = 0.240 here).
`r`=0.15: `relief_hp` +0.042844 / +0.035302. FLAT-LP: `nu_hp_end` 0.736745 vs bare 0.956543.
