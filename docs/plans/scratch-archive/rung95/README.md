# Rung 95 — scratch scripts and their outputs

Run from a scratch crate depending on `turbojet` by path (`W:\temp\claude\jet-rung95-probe`, deleted after the rung);
`lib.rs` is a PROTOTYPE governor built from public pieces only (the instant inversion with its clamp, the rate law, the
proportional law on a 13-row steady table) on the Controls engine. Every script printed REACHABILITY only.

- `reach.rs` → `reach.txt` — anchor § 0 G1–G3: monotonicity of thrust / `Tt4` / HP acceleration in fuel at frozen
  speeds and the walls' fuel-air ratios; the prototype-governed marches' clamp counts and which leg bound.
- `reach2.rs` → `reach2.txt` — G3: rich clamps with no leg cutting (0 of 8); the proportional law's clamps by gain.
- `ends.rs` → `ends.txt` — G4: the ends' HP and LP speeds and fuel flows.
