# Rung 93 anchor — METAL TEMPERATURE: what the turbine's metal allows, and what cooling buys (pre-registered 2026-10-16 — § 3)

**Seam:** rung 92 § *The next seam*: *"A metal-temperature limit that lets cooling buy a hotter burner
(rung 91's), and with it a burner that carries the load."* Rung 91's concession: *"No metal temperature:
at a fixed burner temperature cooling is pure loss here; the rotor-inlet temperature it buys is
reported, not used."*

**The user's choices (2026-10-16):** the seam is TWO rungs — this one (the metal limit, and what
cooling buys) and rung 94 (the burner that carries the generator load: each throttle end holds thrust
or HP speed, a switch). Here: a metal temperature for the **vane AND the rotor blade**, each cooled by
its own rung-91 stream, each with its own limit; and **both framings as a switch** — (B) *the same
engine run hotter* (the sandbox's fixed hardware: the cooling and the metal decide how hot the burner
may run) and (A) *a fresh design at each setting* (the textbook trade).

Shipped as KNOBS (the sandbox direction): every prediction below is scored HIT/MISS, and a miss is a
finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung93-probe`, not in the repo; its
source and output archived in `docs/plans/scratch-archive/rung93/`), on EXISTING code only — no metal
code existed when it ran. § 1–§ 3 written BEFORE any metal number was computed by metal code. What
§ 0 saw is a GIVEN and scores no credit.

---

## 0. GIVENS — measured before anything was written

**G0 — the published correlation (Gauntner 1980, NASA TM-81453, "Algorithm for calculating turbine
cooling flow and the resulting decrease in turbine efficiency"; read from the NTRS scan).** Per row of
airfoils, `W/WG = FACTOR · 0.022 · (φ/(1−φ))^1.25`, `φ = (T − TM)/(T − TC)`: `WG` the gas entering the
row, `TC` the coolant's extraction temperature, `TM` the allowable bulk metal temperature. Gauntner's
listing (subroutine COOLIT) fixes the rest:

* **the gas a row sees**: the first vane row sees the turbine inlet temperature; the rotor sees the
  gas after the vane coolant has mixed in (a mass-averaged enthalpy), times **0.92** (the relative
  total temperature); both raised by a **150 °R safety margin** (83.33 K);
* **the hot spot**: `φ_hot = (P + φ)/(P + 1)`, `P` = **0.30** for the first stator, **0.13** for every
  other row — algebraically the gas temperature `T + P(T − TC)`;
* **endwall, shroud, disk and leakage**: the row's flow `×4/3` (so the airfoil itself gets ¾ of it);
* **FACTOR** (cooling technology): 2.0 convection, 1.5 coated convection, 1.4–1.1 film + convection,
  1.0 full-coverage film, 0.8 transpiration;
* **allowable metal**: `TMVANE = (10·YEAR − 17640) − (100·log₁₀ LIFE − 400)` °R, `TMBLAD` the same
  with 17740 — at 1985 and 10 000 h: **2210 °R = 1227.8 K (vane), 2110 °R = 1172.2 K (blade)**.

Reproduced from the bare formula (the probe): Gauntner's own example `FACTOR 1, φ 0.5 → 0.0220`;
his worked limits (YEAR 2000 → 2360 °R, YEARB 1977.8 → 2038 °R, 1985 blade 2110 °R) exactly; and
Kandiyoor (TU Delft MSc thesis, 2024, *Cooled Turbine Model*) — Fig. 6.5 (`TM` 1100 K, `TC` 800 K,
gas 2100 K): FACTOR 2.0 → **0.198** (plotted ≈ 0.20), FACTOR 1.0 → **0.099** (≈ 0.10); § 6.2's Young &
Wilcock case (1700 / 1100 / 867 K) needs FACTOR **2.016** to give 0.1447 (thesis: "2.01").

**G1 — rig A's cooled running lines** (rung 91's matcher; rig A = rungs 42/90/91/92's: CPG γ 1.4/1.3,
`π_LPC` 3 × `π_HPC` 6, design `Tt4` 1500 K, 250 K / 50 kPa / M 0.85, flat and `flow/press` maps), `Tt4`
1300 → 2200 K, shares (vane, blade) ∈ {0, (2,0), (5,0), (10,0), (0,2), (5,2), (10,5)} %. The matcher
solves the whole range (to 2200 K on every share); above ~1600 K the speeds run 5–38 % over design
(the invented maps extrapolated). Uncooled `Tt3`: 651.6 / 709.8 / 857.7 / 917.9 K at 1300 / 1500 /
2000 / 2200 K.

**G2 — cooling cools its own coolant on fixed hardware.** At a fixed `Tt4`, cooling slows the HP spool
(rung 91) and `Tt3` FALLS: relative slopes `a₁ = d ln Tt3/dε₁` = −0.648 (1300 K) → −0.696 (1500) →
−0.813 (2200), `a₂` (blade) = −0.804 → −0.852 → −0.968. **Linear and additive**: the two-slope
prediction of `Tt3` at (5, 2) % matches to 0.01 K, at (10, 5) % to 0.18–0.20 K, at every `Tt4`; and
**map-independent** (identical on flat and `flow/press` to 0.01 K — on the perfect gas the cascade's
temperatures do not see the map's flow shape). Thrust per inlet air at fixed `Tt4`: `d ln(F/ṁ)/dε` ≈
−0.64 → −0.47 (vane), −0.67 → −0.44 (blade), 1300 → 2200 K.

**G3 — a fresh design's numbers** (`build_two_spool_turbojet`'s design run at `π` 3 × 6, uncooled):
`Tt3` 709.77 K at every `Tt4` (fixed by `π`); `F/ṁ` 608.8 → 1081.2 N·s/kg and TSFC 36.2 → 47.0 mg/N·s
from 1300 to 2200 K — **TSFC rises with `Tt4` even uncooled** (at fixed flight speed the propulsive
efficiency falls faster than anything else gains). So the published *"efficiency peaks at a moderate
temperature once cooling is counted"* (Horlock, Watson & Jones 2001 — a power-plant cycle, abstract
only) does NOT transfer to a turbojet: there is no efficiency peak to find. (Rung 1's KE-based
`η_thermal` reads ~0.13 with `η_p` > 1 on this choked nozzle — the pressure thrust is not kinetic
energy; it is not used.)

**G4 — the sandbox slam** (the Controls view's march, rung 43's fuel-metered path, defaults 1000 →
1400 K over 0.5 τ, four maps × `ρ` {0.5, 1, 2} × ramps {0.5, 0.1} τ, cooling off and (5, 3) %): `Tt4`
**overshoots** its end value by 232–581 K (larger for faster ramps and slower LP spools), and at the
peak `Tt3` LAGS its end value by 33–72 K. With (5, 3) % cooling the overshoot is 7–19 K larger and the
lag a little smaller.

**G5 — every scored B cell is reachable** (the advisor's pre-freeze check): rung 91's matcher, run at
Model 1's shares (§ 1) for every `Tt4` 1300–2200 K × FACTOR × both maps, SOLVES at all 80 (solve /
refuse and the speeds read; no `Tt3`, no metal). No share reaches rung 91's 0.25 cap (largest ≈ 0.17
vane, FACTOR 2.0 at 2200 K). The speeds there: with FACTOR 2.0 the HP spool stays at 0.92–0.94 of
design speed from 1500 to 2200 K (the cooling slows it as fast as the heat speeds it); with FACTOR 0.8
it reaches 1.05–1.06 and the LP 1.17–1.21.

## 1. THE DEVICE — derived before coding

**The metal temperature of a row** (Gauntner inverted): with `w` the row's stream over the gas
entering it, the airfoil's share is `w_a = ¾ w`, `y = (w_a/(0.022 F))^0.8`, `φ = y/(1+y)`;

```
T_seen  = T_row + S                      (S = 83.33 K;  vane T_row = Tt4,  blade T_row = 0.92·Tt41)
T_hot   = T_seen + P (T_seen − Tc)       (P = 0.30 vane, 0.13 blade;  Tc = Tt3, the coolant)
T_metal = T_hot − φ (T_hot − Tc)
w_vane  = ε₁ / G,   w_blade = ε₂ / (G + ε₁)  (per HPC air; G = (1 − c − ε)(1+f), the burner gas)
```

No cooling (`φ = 0`) is the gas hot spot itself. The inverse — the stream a row needs to sit AT its
limit — is closed form: `y = (1+P)(T_seen − Tc)/(T_lim − Tc) − 1` (`≤ 0`: no cooling needed),
`w = (4/3)·0.022 F · y^1.25`. And so is the hottest gas a cooling buys:
`T_seen,max = Tc + (T_lim − Tc)(1+y)/(1+P)`. **Everything the metal model reads already exists** on a
rung-91 point: `Tt4`, `Tt3`, `Tt41`, `f`, the shares. It moves no gas.

**Framing B — the same engine run hotter** (rung 91's cooled matcher, hardware sized uncooled at
1500 K, unchanged): (B-knob) given the shares, `Tt4_max` = the root in `Tt4` of
`max(T_metal,v − T_lim,v, T_metal,b − T_lim,b) = 0`, with which row binds; (B-limited) given `Tt4`, the
shares `(ε₁*, ε₂*)` that put BOTH metals exactly at their limits — a 2×2 root on the matcher.

**Framing A — a fresh design at each setting**: a COOLED DESIGN RUN — `TwoSpoolEngine::run` copied
with rung 91's two mixers (vane air after the A4 throat, blade air after the HP rotor) and the turbine
work balances per HPC air (`η_m (G+ε₁)(h41 − h44) = h3 − h25`, `η_m (G+ε)(h45 − h5) = h25 − h2`);
scored per inlet air with rung 91's nozzle bookkeeping. (A-limited): at each `Tt4`, the shares that put
both metals at their limits. On a fresh design `Tt3` is fixed by `π` and `f` by `(Tt3, Tt4)` — neither
sees the cooling — so the closed form IS exact there: **A's cooling requirement is a construction gate,
not a prediction.**

**The question both framings answer** (so they compare directly): at each `Tt4`, the cooling that puts
both metals exactly at their limits, and what it costs (thrust per inlet air, TSFC). The knob form
(shares in, `Tt4_max` out) is the page's readout and is scored once (M6).

**Where it lives.** A reader BESIDE rung 91's cooled point, plus two roots and the cooled design run;
the default `build_turbojet(…).run(…)` design run, every matcher and every march are untouched. **The
perfect gas only** (rung 91's mixers). Bounds in words: shares inside rung 91's `[0, 0.25)` and
together below half (checked at the root); a limit above the gas the row sees needs no cooling (`y ≤ 0`
→ 0, not an error); a limit at or below `Tc` is refused (no coolant can hold it).
`TurbineCooling::new` PANICS at a share ≥ 0.25, so the B-limited root never builds a trial past the
cap: a step that would is held inside it, and a root that needs the cap is refused in words (rung 90's
lesson: trials may sit anywhere, bounds are checked at the root).

**Reduce (the spine).** No metal limit asked for ⇒ nothing runs and nothing changes: no oracle, no CLI
golden, no earlier gate moves. The cooled design run at zero cooling IS `TwoSpoolEngine::run` bit for
bit (gated).

### The model (the prediction instrument)

Framing B with `Tt3` and `f` taken from the UNCOOLED line at the same `Tt4` — **Model 0** (frozen) — or
with `Tt3 = Tt3,line(Tt4)·(1 + a₁ε₁ + a₂ε₂)` from G2's measured slopes — **Model 1**; `G` from the shares
and the line's `f`; `Tt41` by rung 91's mixer on those inputs; both rows solved together by fixed
point. Framing A: `Tt3` = 709.77 K and the design run's `f` (exact).

Total cooling at the limits (`ε₁* + ε₂*`), flat; Model 1 (B) and A — the probe's table:

| FACTOR | | 1300 | 1500 | 1700 | 1900 | 2000 | 2200 |
|---|---|---|---|---|---|---|---|
| 2.0 | B | 0.0461 | 0.0961 | 0.1494 | 0.2018 | 0.2269 | 0.2741 |
| | A | 0.0508 | 0.1044 | 0.1581 | 0.2094 | 0.2339 | 0.2802 |
| 1.5 | B | 0.0356 | 0.0761 | 0.1216 | 0.1685 | 0.1916 | 0.2362 |
| | A | 0.0391 | 0.0815 | 0.1252 | 0.1683 | 0.1892 | 0.2296 |
| 1.0 | B | 0.0244 | 0.0538 | 0.0891 | 0.1279 | 0.1479 | 0.1880 |
| | A | 0.0267 | 0.0567 | 0.0885 | 0.1208 | 0.1369 | 0.1687 |
| 0.8 | B | 0.0198 | 0.0442 | 0.0744 | 0.1088 | 0.1270 | 0.1644 |
| | A | 0.0216 | 0.0461 | 0.0725 | 0.0997 | 0.1134 | 0.1407 |

Model 0 overstates B everywhere (FACTOR 1.0: 0.0567 / 0.0998 / 0.1905 at 1500 / 1700 / 2000 K against
Model 1's 0.0538 / 0.0891 / 0.1479).

**What it says, before any metal code.**
1. **Cooling cools its own coolant on fixed hardware.** Every share of cooling slows the HP spool and
   lowers `Tt3`, so the remaining coolant is colder: the fixed engine needs LESS cooling than the
   frozen sum says — 5 % less at 1500 K, 22 % less at 2000 K (FACTOR 1.0).
2. **Fixed hardware and a fresh design trade places.** Below a crossing the fixed engine needs less
   cooling than a fresh design (its `Tt3` sits below the design's 709.8 K); above it, more (its `π`
   and `Tt3` rise with `Tt4`). The crossing **rises as the technology worsens** — 1626 K (FACTOR 0.8),
   1676 K (1.0), 1889 K (1.5), above 2200 K (2.0): more coolant cools its own source more. (On the
   model's own terms the FACTOR-2 fixed point is lost near 2350 K, past the measured slopes' range;
   it is not used above 2200 K.)
3. **No best cooling share — on either engine.** Thrust per inlet air at the metal-limited cooling
   rises monotonically with `Tt4` through 2200 K for every technology (FACTOR 1.0, flat: B 615 → 954,
   A 598 → 998 N·s/kg). Cooling costs ~0.5 % of thrust per percent; a percent of vane air buys ~100 K.
   So the metal never stops the fixed engine — what stops it is elsewhere (speed, flow capacity), which
   this rung reports and does not model.
4. **Vane air relieves the blade.** With 2 % blade air (FACTOR 1.0, flat), the vane binds below
   **3.38 %** vane air (bisected; `Tt4_max` 1500 K there) and the blade above; the blade's own limit
   rises 1464 → 1663 K as vane air goes 0 → 16 % — the vane coolant dilutes the gas the rotor sees.
   Uncooled, the engine is metal-limited at **989 K** (vane).
5. **On a fresh design, cooling LOWERS the fuel bill at a fixed `Tt4`.** The burner gets `(1 − ε)` of
   the air at an unchanged fuel ratio (`Tt3`, `Tt4` fixed), so fuel per inlet air falls by the whole
   share, while thrust per inlet air falls only `|d|` ≈ 0.44–0.67 of it (G2's slopes, borrowed from the
   fixed engine): TSFC ≈ TSFC_uncooled `(1 − ε)/(1 + d₁ε₁ + d₂ε₂)`. FACTOR 1.0: −0.35 mg/N·s at 1300 K
   → −4.66 at 2200 K (36.19 → 35.84, 46.96 → 42.30); FACTOR 2.0: −0.67 → −8.19, so its metal-limited
   TSFC is nearly flat above 2000 K (38.61 / 38.72 / 38.77). The metal-limited engine is, in fuel, a
   cooler engine. (Rung 91's fixed-engine "+0.79 / +2.0 % at 1100 / 800 K" is a different bill: there
   cooling also lowers `Tt3`, so each kilogram of burner air needs more fuel.)

## 2. THE READINGS

**Steady (rig A, both maps, FACTOR ∈ {2.0, 1.5, 1.0, 0.8}, Gauntner's 1985 limits, `Tt4` 1300 → 2200
K by 100):** B-limited and A-limited shares, their total, thrust per inlet air and TSFC there; on B also
both speeds and flow coefficients (reported). B-knob at FACTOR 1.0 (flat): `Tt4_max` and the binding
row for blade 0.02 and vane 0 → 0.16 by 0.02.

**The slam (reported, NOT scored — § 0 G4):** the metal is an algebraic function of `(Tt4, Tt3, f,
shares)` at each point, so a slam reading has no model content beyond arithmetic. The page reads both
metals along the march (each recorded point re-read through the march's own closure) and shows the
peak against the limit; the spec reports the default slam's peak. Quasi-steady metal: no blade
thermal lag (rung 37's heat-soak clock is a different, lumped metal) — an UPPER bound on the peak.

## 3. PREDICTIONS (frozen here; scored in the spec)

**Construction (gated, no credit):** G0's published numbers from the device's own functions; the
forward and inverse correlation round-trip to 1e-12; at zero cooling the metal is the hot spot; the
cooled design run at zero cooling ≡ `TwoSpoolEngine::run` bit for bit, and its mixers and both shaft
balances close; **A-limited ≡ the closed form** (to 1e-9); at every B root both metals sit at their
limits to 1e-6 K; the knob round trip `Tt4_max(ε*(T)) = T`; refusals (non-CPG gas, a limit at or below
`Tc`, shares out of range); the reduce (no metal ⇒ nothing changes; design run rung 6).

**The bars are derived.** Model 1's only approximations on B are the `Tt3` linearisation (residual
≤ 0.2 K at (10, 5) %, growing roughly with the square of the share, so ≲ 0.6 K at the largest FACTOR-2
shares) and `f` frozen at the uncooled line (`G` off by ≲ 0.1 %). Through the sensitivity
`d(ε₁+ε₂)/dTc` ≈ 4e−4 per K that is ≲ 5e−4 on a share: **bar 1.5e−3** (3×). A crossing's bar is that
over the local slope of B − A in `Tt4` (≈ 2.2e−5 /K): **± 70 K**.

- **M1 (B magnitude).** The device's `ε₁*` and `ε₂*` within 1.5e−3 of Model 1 at every `Tt4` 1300–2200
  K, every FACTOR, both maps.
- **M2 (cooling cools its own coolant).** The device's total B cooling is BELOW Model 0's at every
  `Tt4`, every FACTOR, both maps.
- **M3 (the crossing).** B − A changes sign ONCE in 1300–2200 K, negative below, at **1626 ± 70 K**
  (FACTOR 0.8), **1676 ± 70 K** (1.0), **1889 ± 70 K** (1.5); at FACTOR 2.0 B < A throughout. (A is the
  construction above; the maps do not enter either cooling — scored on flat, checked equal on
  `flow/press`.)
- **M4 (no best share).** Thrust per inlet air at the metal-limited cooling rises monotonically with
  `Tt4` 1300–2200 K on B (both maps) and on A, for every FACTOR.
- **M5 (cooling lowers the fuel bill — § 1 item 5).** On A, TSFC at the metal-limited cooling is
  BELOW the uncooled design's at every `Tt4` 1300–2200 K, and the gap WIDENS monotonically with `Tt4`,
  for every FACTOR. Sign and trend only: the thrust slopes are borrowed from the fixed engine, so the
  magnitude has no derived bar; the sign holds unless a fresh design's thrust penalty per share
  exceeds 1 (the fixed engine's is 0.44–0.67).
- **M6 (the knob form).** FACTOR 1.0, blade 0.02, flat: the binding row switches ONCE, vane → blade,
  as vane air rises, at **3.38 ± 0.10 %**; the blade's limit rises monotonically with vane air
  (0 → 16 %). Bar: the model's error on either row's limit is ≲ 3 K (the `Tt3` linearisation ≤ 0.2 K at
  these shares, `f` frozen), the gap between the two limits moves 9 064 K per unit share there
  (10 163 − 1 099), and 3 × 3 K / 9 064 ≈ 0.001.

**The likeliest miss, named in advance:** M1 at FACTOR 2.0 above ~2000 K — the shares there (≈ 0.14 /
0.09 and up) run past the largest measured slope row (10, 5) %, so the `Tt3` linearisation is
extrapolated. Then M3 at FACTOR 1.5, whose crossing sits where B − A is flattest.
