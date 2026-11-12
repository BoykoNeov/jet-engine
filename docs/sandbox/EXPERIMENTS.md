# Experiments to try in the sandbox

Open `turbojet-sandbox.html` (or double-click `Start Jet Engine.cmd` at the repo root). Each experiment
below says which knob to move, what you should see, and why it happens. Start each one from the page as
it opens (reload the page to get back there).

Every number here was read off the page itself on 2026-11-12, at its opening settings unless the
experiment says otherwise. They are there so you can tell you are looking at the right thing; the
*direction* of each change is the lesson, not the last digit. These numbers are not yet checked by the
test suite, so after a model change they may drift (`docs/plans/roadmap.md`, item A2).

Units used on the page: thrust in kN; "fuel per thrust" (TSFC) in grams of fuel per kN of thrust per
second — lower is better; τ is the shaft's own response time (the model has no rotor weight, so it
cannot give seconds).

---

## Design the engine

The opening design: pressure ratio 10, turbine inlet 1500 K, Mach 0.85 at about 5.6 km.
Thrust 15.97 kN, fuel per thrust 34.04.

### 1. More compression: thrust peaks, fuel economy keeps improving

Move **Compressor pressure ratio**.

| Pressure ratio | 3 | 5 | 10 | 15 | 20 | 30 | 40 | 60 |
|---|---|---|---|---|---|---|---|---|
| Thrust, kN | 13.38 | 14.94 | 15.97 | 16.05 | 15.88 | 15.30 | 14.63 | 13.24 |
| Fuel per thrust | 47.74 | 40.31 | 34.04 | 31.38 | 29.76 | 27.74 | 26.46 | 24.87 |

**Why.** Squeezing the air harder before burning turns more of the fuel's heat into work, so fuel per
thrust keeps falling. But the compressed air arrives at the burner already hot, and the turbine inlet
temperature is fixed, so less fuel can be added per kilogram of air: beyond a pressure ratio of about 15
each kilogram of air gives less thrust. An engine built for thrust and one built for economy want
different pressure ratios.

### 2. A hotter turbine: more thrust, worse economy

Put the pressure ratio back to 10 and move **Turbine inlet temperature**.

| Turbine inlet, K | 1100 | 1300 | 1500 | 1700 | 1900 |
|---|---|---|---|---|---|
| Thrust, kN | 10.11 | 13.22 | 15.97 | 18.49 | 20.86 |
| Fuel per thrust | 28.37 | 31.12 | 34.04 | 36.99 | 39.96 |
| Propulsive efficiency, % | 52.5 | 46.0 | 41.5 | 38.2 | 35.6 |

**Why.** More heat makes a faster jet, so more thrust from the same airflow. But a faster jet wastes
more energy in its own wake (the propulsive efficiency falls), so each newton costs more fuel. This is
the reason airliners moved from turbojets to turbofans: move more air, more slowly.

### 3. What the textbook gas gets wrong

Under *Nozzle and gas*, switch **Gas model** to the perfect gas, then back.

| Gas model | Thrust, kN | Fuel per thrust |
|---|---|---|
| Perfect gas (constant heat capacity) | 14.78 | 30.29 |
| Thermally perfect (heat capacity grows with temperature) | 15.97 | 34.14 |
| Reacting products / equilibrium chemistry | 15.97 | 34.04 |

**Why.** Hot gas takes more heat to warm by one kelvin than cold gas does. The constant-property
textbook gas misses that, so it under-counts the fuel by about 11 % and the thrust by about 7 %. Once
the heat capacity follows temperature, the finer chemistry barely moves the cycle at this temperature —
it matters for what comes out of the burner (experiment 12), not for thrust.

### 4. Real parts against ideal ones

Set every loss to ideal: both efficiencies, the combustion efficiency, the burner and nozzle pressure
ratios, the inlet recovery and the shaft efficiency all to 1. Thrust rises from 15.97 to 17.52 kN and
fuel per thrust falls from 34.04 to 31.75. Use **pin & compare** before you start and the page shows
each change. Then restore them one at a time to see which loss costs the most.

### 5. Flight speed: thrust falls, efficiency rises

Move **Flight Mach number**.

| Mach | 0 | 0.5 | 0.85 | 1.5 | 2.0 | 2.5 |
|---|---|---|---|---|---|---|
| Thrust, kN | 19.86 | 17.29 | 15.97 | 13.29 | 10.45 | 6.76 |
| Overall efficiency, % | 0 | 11.3 | 18.5 | 32.0 | 41.8 | 49.2 |

**Why.** Thrust is airflow times how much faster the jet leaves than the air arrived. Fly faster and
the arriving air is already fast, so the same jet adds less. But thrust *power* is thrust times flight
speed, so the engine turns more of its fuel into useful work. (The design airflow is held at 20 kg/s
here; a real engine swallows more air as it speeds up.)

### 6. The standing engine — how engines are rated

Set **Altitude** to 0 and **Flight Mach number** to 0: a sea-level test stand. Thrust 18.86 kN, fuel
per thrust 28.72. Overall and propulsive efficiency read exactly 0 — the engine is pushing hard and
moving nothing, so it does no useful work. That is why a standing engine is judged by thrust and fuel
per thrust alone. (Only this view runs a standing engine so far; *Fly it* and *Controls* need a little
flight speed — Mach 0.05 works.)

### 7. A nozzle that cannot finish the job

Under *Nozzle and gas*, switch the nozzle from fully expanded to **fixed convergent**. Thrust falls from
15.97 to 14.91 kN. The gas leaves at the speed of sound but still at higher pressure than the air
outside; the expansion that would have made it faster happens behind the engine, where it pushes on
nothing. Note the efficiency split: "thermal" drops to 15 % and "propulsive" reads 115 %. That split
counts only the jet's speed and misses the pressure push — the page's note on efficiencies explains it;
trust the overall figure.

### 8. Light the afterburner

Tick **Light an afterburner behind the turbine**. Thrust rises from 15.97 to 22.26 kN (+39 %); fuel per
thrust rises from 34.04 to 49.83 (+46 %). The nozzle throat must open to 1.33 times its unlit size.
Burning fuel at low pressure, after the turbine has taken its work, is a poor way to use fuel and a
very good way to get thrust from a small engine for a short time.

---

## Fly it

Press **Fly it**. The design's hardware is now frozen; you move the throttle and the flight.

### 9. Throttle back: the compressor drifts toward stall

Move the **Throttle** down and watch the *Stall margin* tile.

| Throttle, K | 1500 | 1300 | 1100 | 900 | 800 | 700 |
|---|---|---|---|---|---|---|
| Thrust, kN | 14.92 | 10.69 | 7.19 | 4.37 | 3.17 | 2.12 |
| Shaft speed, % of design | 100 | 91.0 | 82.1 | 73.2 | 68.6 | 64.0 |
| Stall margin, % | 39.5 | 25.7 | 16.3 | 10.4 | 8.5 | 7.3 |

**Why.** A compressor is a row of small wings. At low speed the front stages get the air at too steep
an angle. The margin against stall is thinnest at low power, not high — which is why real engines carry
bleed valves and variable stators for starting and idling. (The stall line is a number you choose on
this page, so read the trend, not the size.)

### 10. Slam the throttle: the temperature overshoots

In *Slam the throttle*, keep **Both**: the move from 1100 K to 1500 K in half a shaft response time.

| The move takes | Peak turbine inlet, temperature commanded | Peak, fuel metered |
|---|---|---|
| 0.5 τ | 1500 K | 1756 K |
| 0.1 τ | 1500 K | 1971 K |

**Why.** A real throttle meters *fuel*, not temperature. Fuel arrives at once; the shaft needs time to
spin up and bring more air. For a moment there is full fuel in part-speed airflow, and the mixture burns
hotter than the engine was designed for. The faster the slam, the worse. This overshoot is the reason
the fuel limiters in the next view exist.

### 11. Afterburner against a nozzle that does not open

In the *Afterburner* panel choose **Lit**, leaving the nozzle throat at its dry size: thrust *falls*
from 14.92 kN (14.83 with the afterburner fitted but unlit) to 10.97 kN. Now tick **Open the nozzle by
the rule**: 20.71 kN.

**Why.** Hotter gas is thinner and needs a bigger hole. Behind a throat that stays shut it backs up
into the turbine; the turbine does less work, the shaft slows and the compressor is pushed toward
stall. Every afterburning engine has a variable nozzle for this reason.

---

## Burner

### 12. The nitrogen-oxide bell

Move **the front zone's richness** (1 = exactly enough air to burn the fuel).

| Richness | 0.6 | 0.8 | 1.0 | 1.2 | 1.5 |
|---|---|---|---|---|---|
| Front-zone temperature, K | 1895 | 2225 | 2442 | 2361 | 2110 |
| NOx, g per kg of fuel | 0.05 | 4.78 | 20.74 | 0.83 | ~0 |

**Why.** Nitrogen oxides need two things at once: very high temperature and spare oxygen. Lean, the
flame is too cool. Rich, it is still hot but there is no oxygen left over. The worst case is right at
the middle — which is exactly where a simple burner wants to run.

### 13. Burning rich is only a loan

Set the richness to 1.5, then switch the dilution from *Instantly* to **Over a set time** and lengthen it.

| Dilution time, ms | 0.2 | 1 | 3 | 10 |
|---|---|---|---|---|
| NOx, g per kg of fuel | 0.22 | 1.09 | 3.24 | 10.52 |

**Why.** To finish burning, the rich gas must be mixed with the rest of the air — and on the way it
passes through the hot, just-right mixture of experiment 12 (it peaks at 2451 K on the way). The
longer it lingers there, the more of the rich zone's advantage is paid back. "Rich burn, quick quench"
only works if the quench really is quick.

---

## Controls

A two-shaft engine, the throttle slammed from 1000 K to 1400 K in 0.5 τ. As it opens (no limiter), the
turbine inlet peaks at **1695 K** before settling at 1400 K, and thrust reaches 90 % of its change
after 1.08 τ.

### 14. How fast you slam decides how hot it gets

| The move takes | 0.1 τ | 0.5 τ | 2 τ |
|---|---|---|---|
| Peak turbine inlet, K | 1943 | 1695 | 1477 |
| Time to 90 % of the thrust change, τ | 0.86 | 1.08 | 2.10 |

### 15. What each limiter costs

Switch each on by itself (the move back at 0.5 τ).

| Fuel control | Peak turbine inlet, K | Time to 90 % thrust, τ |
|---|---|---|
| None | 1695 | 1.08 |
| Temperature limiter at 1480 K | 1480 | 1.28 |
| The same limiter with a reaction lag of 0.2 τ (untick the two-speed box) | 1618 | 1.36 |
| Acceleration schedule | 1546 | 1.46 |

**Why.** Every limiter protects the engine by holding fuel back, and the price is always response time.
A limiter that reacts slowly lets most of the overshoot through *and* still costs time — the worst of
both.

### 16. Asking for thrust instead of temperature

Set the move to 0.1 τ, choose **Holds thrust** and tick **Govern**. The thrust now reaches 90 % of its
change in 0.14 τ instead of 0.86 τ — and the turbine inlet peaks at **2594 K** instead of 1943 K.

**Why.** A controller told to deliver thrust *now*, while the shafts are still slow, has only one way to
do it: pour in fuel. A fast response and a cool turbine are bought with the same fuel, and you cannot
have both. Now switch the temperature limiter on beside the governor and watch the two argue.

### 17. Loads the aircraft hangs on the engine

Back at the opening run, switch on one at a time and read the thrust at the end of the run, in newtons per
kg/s of design airflow (it opens at
615):

| Load | Thrust at the end | Peak turbine inlet, K |
|---|---|---|
| None | 615 | 1695 |
| Generator on the HP shaft | 591 | 1735 |
| Generator on the LP shaft | 591 | 1727 |
| The aircraft takes 5 % of the HP compressor's air | 537 | 1702 |

**Why.** Anything taken from the engine — shaft power for a generator, air for the cabin — is work the
turbine did that never reaches the jet. Air is the dearer of the two: it was compressed at full cost and
then thrown away.

### 18. The same engine, lower down

Set **Altitude** to 0. The same slam peaks at 1587 K instead of 1695 K and the thrust response is
quicker (0.80 τ against 1.08 τ): denser air gives the shafts more to work with.

---

## Cool the turbine

### 19. How cooling technology raised turbine temperatures

Switch the **cooling technology** and read the hottest burner temperature the metal allows (with the
opening cooling air: 5 % to the vanes, 3 % to the blades).

| Technology | Convection | Coated | Film | Transpiration |
|---|---|---|---|---|
| Hottest turbine inlet, K | 1422 | 1512 | 1638 | 1709 |
| What limits it | vanes | vanes | blades | blades |

With no cooling air at all the same metal allows only 991 K. Raise the air to 8 % and 5 % (film) and
the limit climbs to 1886 K.

**Why.** Turbine blades run in gas hotter than their own melting point; they survive on air bled from
the compressor. Each generation of cooling let the same air protect the metal from hotter gas — and by
experiment 2, hotter gas is thrust. But that air skipped the burner, so it is not free: go back to
*Controls*, give the vanes and blades cooling air, and watch what it does to the run.
