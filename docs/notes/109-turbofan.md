# Rung 109 — The turbofan

*Plain-language notes. The formal derivation, tests and every number's source are in `docs/rung109-spec.md`.
Numbers below are copied from it and belong to the cases it ran; the sandbox page shows its own.*

**In one sentence:** a fan driven by the same turbine pushes a second stream of air around the core, so the
engine throws more air out more slowly and burns less fuel for its thrust; but the best fan does not make the two
jets equally fast.

**How it works.** A jet engine makes thrust by speeding air up. Thrust grows with the speed added, but the energy
it costs grows with the square of that speed, so a fast jet wastes much of its fuel as jet energy left behind in
the air. A **fan** at the front takes some of the turbine's work and uses it to push extra air, the **bypass
air**, around the core and out of its own nozzle. The **bypass ratio** is how many kilograms go around for each
kilogram through the core. The **fan pressure ratio** sets how hard the fan pushes, and so how fast its jet leaves.
On this page the turbine that drives the compressor also drives the fan, and the engine is looked at only on the
drawing board (its design point), not flown.

- **More bypass, less fuel per thrust, on the same core.** Each bypass ratio at its own best fan, bypass ratio 12
  needs 0.55–0.65 of the turbojet's fuel per unit thrust at 11 km and Mach 0.8, and only 0.33–0.36 standing still.
  Standing still the fan keeps paying long after the gain has flattened in flight, because there the whole jet
  speed is waste.
- **The best fan stops short of equal jets.** A loss-free engine does best when both jets leave at the same speed.
  A real one does not: work moved from the core jet to the fan jet passes through the turbine and then the fan,
  and each loses a share on the way. So at the best fan the fan jet leaves at **0.70–0.81** of the core jet's
  speed, close to the product of the turbine's and the fan's efficiencies.
- **A simple nozzle is almost free on a fan.** A fixed convergent nozzle, which cannot expand the gas fully, costs
  the turbojet 8.7–15 % of its thrust at 11 km, but a bypass-ratio-8 turbofan only 0.4–1.3 %: the fan's low
  pressure has little to waste.
- A textbook turbofan (Mattingly's Example 7-6) runs through the model and lands on his own equations to twelve
  digits. At bypass ratio 0 the page is the turbojet, number for number.

**On the page.** *Design → Fan (turbofan)*: set the bypass ratio above 0, or pick *Try this → Add a fan*. The fan's
tiles show the best fan pressure ratio for your settings and how fast the fan jet leaves beside the core jet; a chart
plots fuel per thrust against bypass ratio, at your fan pressure ratio and at each one's best; stations 13 and 19
join the table and the T–s diagram.
