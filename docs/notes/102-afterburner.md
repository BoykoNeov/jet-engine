# Rung 102 — The afterburner

*Plain-language notes. The formal derivation, tests and every number's source are in `docs/rung102-spec.md`.
Numbers below are copied from it and belong to the cases it ran; the sandbox page shows its own.*

**In one sentence:** a second burner behind the turbine adds a lot of thrust for a lot of fuel, but only if the
exhaust nozzle opens to let the hotter gas out; whether lighting it raises or lowers thrust depends on what the
throttle holds.

**How it works.** The gas leaving the turbine still holds most of its oxygen: a jet engine burns far less fuel
than the air could burn, to spare the turbine. An **afterburner** sprays fuel into that gas behind the turbine and
burns it. No turbine has to survive it, so it can run much hotter. The catch is the nozzle. Hotter gas is thinner,
so the same mass of it needs a bigger hole. If the throat does not open, the gas backs up into the turbine. The
textbook **square-root rule** opens the throat by the square root of the temperature rise.

---

## Part A — On the drawing board

- **It pays most flown fast.** Lit to the same temperature, the same engine gains more thrust at a higher flight
  speed, for a smaller fuel penalty: in the spec's design at 1900 K, ×1.35 thrust for ×1.39 fuel per unit thrust
  at Mach 0.85, but ×1.64 for ×1.36 at Mach 2. That is why afterburners belong to supersonic aircraft.
- **The rule is not quite enough.** The throat must open by the square-root rule *and* by another **1.1–5.9 %**:
  the added fuel is extra mass to pass, and very hot gas passes a little less per unit of area.
- **The ceiling is chemistry.** Burning all the oxygen left, the gas reaches only **2211–2244 K** once it is
  allowed to dissociate (hot molecules breaking apart and soaking up heat, rung 6): 60–79 K short of a gas that
  does not. Near that ceiling, dissociation costs 1.7–3.1 % more afterburner fuel.
- A textbook reference engine (Mattingly's) runs through the model and lands on his own equations to twelve
  digits.

**On the page.** *Design → Light an afterburner behind the turbine*. *Try this → Light the afterburner*. In
`docs/sandbox/EXPERIMENTS.md` (experiment 8) thrust rises 39 % and fuel per thrust 46 %, and the throat opens to
1.33 times its unlit size.

## Part B — Flown, behind a nozzle that does not open

**Lit behind a held nozzle, an afterburner makes LESS thrust than not lighting it**, and walks the compressor
toward stall. At four flights, two throttles and two temperatures, a held throat gave ×0.62–0.90 of the unlit
thrust for ×1.07–1.22 of its fuel; the shaft slowed to 72–86 % of its unlit speed and the stall margin fell by
64–99 %. The backed-up gas leaves the turbine less pressure drop to work with, so it does less work and the
shaft slows.

Opening the throat buys both back, step by step, and the lit thrust passes the unlit one about halfway to the
rule's size. At the rule's size the core runs exactly where it runs unlit, and the throat has opened **1.19–1.40×**,
more than the plain square-root rule's 1.16–1.32.

**On the page.** *Fly it → Afterburner*: *Lit*, then *Open the nozzle by the rule*. *Try this → Afterburner lit,
nozzle held at its dry size* and *…nozzle opened by the rule* (experiment 11: 14.92 kN unlit, 10.97 kN lit behind
the held throat, 20.71 kN opened).

## Part C — Lit during a slam

**What the throttle holds decides the sign.** Part B held the turbine temperature. On the *Controls* slam the
throttle meters *fuel* instead, and there a light-off behind a held nozzle gives **more** thrust than not lighting.
It takes that thrust from the core: the backed-up gas slows the shafts, and the main burner, getting the same fuel
in less air, runs about 200 K hotter.

Opened by the rule at every moment, no shaft moves at all and thrust rises ×1.48–1.54.

One combination is refused: a governor holding *thrust* with the afterburner lit. While it is lit, more main fuel
*lowers* thrust (less air gets through the turbines, so the afterburner burns less), so the governor would push the
wrong way. The page says so in words.

**Later correction.** As first built, the held light-off made the HP shaft *speed up* by 8–13 %. Rung 104 found
that came from the old assumption that the LP turbine's entrance is choked. With its real flow law the HP shaft
*slows* 3–6 %, the LP shaft slows 10–16 %, the burner still runs about 200 K hotter and thrust still rises
(+22 to +27 %). And a held light-off now ends with every stall margin below the unlit run's. The sign the throttle
decides survives.

**On the page.** *Controls → Afterburner*, lit at a moment in the run.
