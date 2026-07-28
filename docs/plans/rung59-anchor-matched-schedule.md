# Rung 59 anchor — the MATCHED schedule

The probes that fixed the mechanism and cleared the blocker **before** any prediction was
written, then the five predictions as registered, then their scoring. Ordering matters and is
recorded honestly: probes A–G are exploration, P1–P5 were written down before being run.

## The probes (exploration — these FIXED the rung, they did not test it)

**Probe A — `Δκ` and the CLAMP.** Built `L_B` (bare) and `L_A` (LP-scheduled) and printed both
tables. **Surprise: identical to every printed digit, including the abscissa.** Either the
running line is stator-invariant in `(n_H, Wf/pt3)` — a finding — or `accel_schedule` never
sees the stator — an artifact. Probe A could not tell them apart, so nothing was concluded.

**Probe B — is the equilibrium ACTUALLY armed?** It is: `map_lp.vsv = 0.16890367` after an
`equilibrium` call, and `nu_lp` moves by **+6.09e−02** (8 %). So the machine really is
statored and the table really is invariant. The columns showed why: `n_hp`, `Tt25` and `f` at
machine precision, while `mdot_air` and `pt4` **both** move −0.373 % — the *ratio* is
preserved. This is where the mechanism was first visible.

**Probe C — is it exact, and WHICH SPOOL breaks it?** `κ_ss` invariant to `≤ 3e−13` for LP
const (0.05…0.35), LP schedules (two `v_max`, two knees) **and** HP const. But the HP rows
showed `d n_H = +3.2e−02` with `d f = d Tt25 = 0`: **the ordinate is invariant, the abscissa
moves.** `v_hp = 0.20` raised off-map — the authority limit.

**Probe D — tuple level, and `Δ_match`.** LP: `Δ_match = −1.8e−15`, with `s*`, `s_eng` and the
fuel removed bit-identical. HP `v=0.10`: `Δ_match = +1.48e−02` (LP spool), `s_eng` **halved**
0.2469 → 0.1247, fuel removed **×6.6**. The matched table's cap is 10–11 % lower at fixed
`n_H`.

**Probe E — the PROOF chain, and the BLOCKER.** `MFP = ṁ(1+f)√Tt4/pt4 = 2.962907072632e−05`
— **the same number at `Tt4` = 1000, 1200, 1400 and on every stator**, i.e. the choked-`A4`
hardware group. `Tt3` invariant to `1e−13`. And the blocker the advisor flagged: every
engagement and **all 201 / 86 / 234 cutting points strictly inside** their brackets. Cleared.

**Probe F — the DISCRIMINATOR (the one that made it a mechanism).** Spliced the tables:
`L_synth` = armed index + bare values, `L_ctrl` = bare index + armed values.

    ABSCISSA carries  +1.482594e-02  =  100.00 %
    ORDINATE carries  +3.330669e-15  =    0.00 %

`L_synth` reproduces the real matched leg to every digit of `M_i`, `s*`, `s_eng`, removed.
**Also run, also reported: rung 58's own profile-credit recovery returns 3.6 %** — its channel
is the schedule's *state-feed* and the HP branch is a *constant* setting, which has none. Rung
58's predictor does **not** extend, and the spec says so.

**Probe G — the main table.** Four stator legs × two spools. LP rows: `Δ_match` ∈ ±1.3e−14.
HP rows: abscissa share **100.00 %** on all four, and the `|ΔI|` ratio (unmatched / matched)
**95.7× / 10.2× / 48.3× / 10.0×**, with the SIGN flipping on the statored spool. Authority
limit re-checked: `v_hp` = 0.125 and 0.150 OK, 0.200 off-map.

**A correction found while gate-writing, not by a probe.** The advisor proposed asserting
*tuple equality* of the two tables for nonzero LP settings. Measured: they differ by
`1.2e−13 … 7.8e−13` — `equilibrium`'s Newton converges to a tolerance, not an exact float.
Tuple equality holds **only at `v = 0`**. The invariance gate is therefore a tolerance gate,
and the bit-level gate is the `v = 0` reduce. Asserting more would claim more than the solver
can deliver.

## The predictions, as registered (before running)

- **P1** — LP stator: `Δ_match` stays at machine zero (`< 1e−12`) at a different knee
  (`n_lo = 0.85`) and a different `v_max` (0.35). *expect HIT*
- **P2** — HP `v = 0.10`, LP spool: `Δ_match` survives `ds`-halving (0.005 → 0.0025) within
  5 %. *expect HIT*
- **P3** — the abscissa share stays `≥ 99 %` at `r = 0.25` and `r = 1.00`; the mechanism is
  not ramp-rate. *expect HIT*
- **P4** — the `|ΔI|` ratio stays `> 10×` at `r = 0.25` and `r = 1.00`. **UNCERTAIN** — rung
  58 showed the interaction is clocked, so this may fail at one end.
- **P5** — at `v_hp = 0.15` (the authority edge) the abscissa share is still ~100 % and
  `Δ_match` grows monotonically with `v_hp`. *expect HIT*

## Scoring

<!-- SCORING -->

## Shas

- rung 58 (parent): `e9e7123`
- rung 59: recorded on commit.
