# Rung 93 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\jet-rung93-probe`, deleted after the rung); `lib.rs` is its shared rig-A setup and the bare Gauntner correlation.

- `hot.rs` — probe 1: does rig A's cooled matcher solve above its design `Tt4` (1500 → 2000 K, no cooling and 5 % vane)? It does; speeds run well over design.
- `givens.rs` → `givens.txt` — anchor § 0: G0 (Gauntner's published numbers and Kandiyoor's Fig. 6.5 / Y&W case reproduced), G1 (rig A's cooled running lines), G3 (uncooled design runs over `Tt4`). Existing code only.
- `slam.rs` → `slam.txt` — anchor § 0 G4: the sandbox Controls march's `Tt4` overshoot and lagging `Tt3`, cooling off and (5, 3) %.
- `model.rs` → `model.txt` (part 1: slopes, the `Tt3` linearisation's residual, Model 0 / Model 1 / A per FACTOR) and `model2.txt` (`-- part2`: the A/B crossing on a 10 K grid and the knob form). The knob scan's final panic is its own bracket running past 2400 K at 18 % vane air; the 0–16 % rows above it are the reading.
- `model.rs -- part3` → `model3.txt` — the advisor's pre-freeze checks: (a) A's TSFC from the model (cooling LOWERS it — M5 was re-registered from "grows" to this), (b) reachability of every scored B cell at Model 1's shares (all 80 solve; no share reaches 0.25), (c) the knob-form crossover bisected (3.381 %) and its sensitivity (the M6 bar).
