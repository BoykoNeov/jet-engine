# Rung 33's double root — what the repair's numbers were measured with (2026-10-21)

Scratch crate (`Cargo.toml` points at the repo; `q6` was also built against a throwaway worktree carrying the trial
fix). Spec: `docs/rung33-spec.md` § The double root.

* `q1.rs` — Fly it (rung 34) vs rung 31/33's matcher at 45 °C / sea level / Mach 0.3, 600–1000 K: Fly it never uses
  rung 33; it refuses 600–725 K for its own reason. (Output read in session.)
* `q2.rs` / `q2.txt` — the candidate walk vs rung 34's flat-map equilibrium and the root count, six flights, two gases.
* `q3.rs` / `q3.txt` — both roots' physics (`π_c`, `M9`, thrust) and the merge throttle, bisected.
* `q4.rs` / `q4.txt` — no silent wrong root: 69 subsonic points the old code returned, none with two roots.
* `q5.rs` / `q5.txt` — rung 34's wall: no instant evaluates below ~0.56–0.60 of design speed at 700/750 K (the next
  repair's seam).
* `q6.rs` / `q6-old.txt` / `q6-new.txt` — the offdesign oracle's grid, old code vs the trial fix: exactly six cells move.
