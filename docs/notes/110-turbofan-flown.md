# Rung 110 — The turbofan flown

*Plain-language notes. The formal derivation, tests and every number's source are in `docs/rung110-spec.md`.
Numbers below are copied from it and belong to the cases it ran.*

**In one sentence:** once a turbofan is built and its throttle moved, the share of air going around the core stops
being a setting and becomes a result, which ties the fan's shaft to the core's compressor and weakens the core's own
jet at part throttle.

**How it works.** Rung 109 drew a turbofan on the drawing board. Here the hardware is frozen at that design and the
engine is flown: the throttle sets the turbine temperature and everything else settles where it must. The engine is
split onto two shafts, as a real turbofan is: a high-pressure shaft (the core's compressor and its turbine) and a
low-pressure shaft (the fan and its turbine). Two fixed nozzles share the fan's air: the core's and the fan's. How much
air goes around the core (the **bypass ratio**) is whatever those two nozzles let through. Without maps of the
compressors, each part keeps its design efficiency, and a shaft's speed is read from the work its compressor does.

- **The fan shaft hears the core's compressor.** In a two-shaft turbojet (rung 38) the front compressor does not notice
  a change in the rear one. With a fan it does: the bypass ratio depends on how much air the core swallows, and that
  depends on the rear compressor. Make the rear compressor 0.01 less efficient and the fan's pressure ratio drops at
  every fan point that runs (162 of 168).
- **The core's jet weakens at part throttle.** The fan takes most of the low-pressure turbine's work, so the core's
  own nozzle stops running full (unchokes) as the throttle comes back. Then the core shaft slows more than the
  textbook rule "its speed depends on the turbine temperature alone" says: up to 0.16 of its design speed lower,
  standing still at 900 K, against 0.005 wherever the core's nozzle stays full.
- **The fan carries the engine.** In cruise, at high bypass, the core's own jet ends up slower than the flight at part
  throttle, so it pulls back instead of pushing: its thrust crosses zero near 1070–1135 K, and below that the fan
  makes all the thrust.
- A textbook example (Mattingly's Example 8-8, the same engine flown from cruise to standing still) runs through the
  model and lands on the book's own equations to eleven digits. With no bypass air the engine is rung 38's two-shaft
  turbojet, number for number.

**On the page.** Not yet: this rung is the model and a command-line table (`-- panel print_flown_turbofan_table`).
The sandbox page gets the flown turbofan with the compressor maps, in the next rung.
