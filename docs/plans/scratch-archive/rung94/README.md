# Rung 94 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\jet-rung94-probe`, deleted after the rung); `lib.rs` is its shared rig-A setup.

- `givens.rs` → `givens.txt` — anchor § 0 G1/G2: the unloaded line's slopes in `Tt4`, and the responses to a fixed POWER load on either shaft at a fixed `Tt4` (Richardson 0.01/0.02). Existing code only (rung 92's loaded matcher).
- `curv.rs` → `curv.txt` — G3: finite loads (curvature in `w`), the line's second derivatives, and the largest fixed power carried at a fixed `Tt4` (the fold).
- `model.rs` → `model.txt` — § 1's model: the hold's linear `ΔT`, its quadratic correction (the bar), the flow-coefficient shifts, and reachability on the existing matcher, both holds, both shafts, both maps.
- `check2.rs` → `check2.txt` — the advisor's pre-freeze checks: G4 (the 700 K HP-compressor shift at `T0 + ΔT_lin`) and G5 (the lowest `Tt4` that carries each load, and the thrust there).
