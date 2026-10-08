# Slice AJ step 2 — injection predictions (written BEFORE the sweep ran)

Gate: `rust/tests/slice_aj_clock.rs` (5 tests: clock, mask, clock_latched, clock_single, mask_clip).

| # | injection (in `authority_clock.rs`) | prediction | why |
|---|---|---|---|
| J1 | `_criterion_at` branches on the POINT VARIANT (`Demand`) instead of `coord == "demand"` | KILLED — `clock_latched` ONLY | latched points are `Demand`-shaped; `demand`/`clip` marches give `Demand`/`Shared`, so the two tests agree there |
| J2a | `authority_mask` uses `demand_tau(lag, cap_fuel, w_fuel)` when `coord == "demand"` | **SURVIVED** | on a `demand` point `required_fuel = mf_sched - cap_fuel` and `g_fuel = mf_sched - w_fuel`, so `required > g` ⇔ `w > cap` — the swap IS the identity; only a rounding collapse at `cap ≈ w` could split them |
| J2b | `_criterion_at`'s demand branch uses `lag.tau(required_fuel, g_fuel)` | **SURVIVED** | same identity, other direction |
| J3 | the mask ignores `every` (stride 1 always) | KILLED — `mask_clip` ONLY | the only call at stride 2 |
| J4 | the mask's aggregates over ALL arms, not `alive` | SURVIVED | every arm is `riding4_valid` on this rig |
| J5 | clip `lag_gap` written in the DEMAND form (`tau_f·sf − tau_gov·sr`) | KILLED — clock, clock_latched, clock_single | every clip-form cell moves |
| J6 | `census` over interior points only (ends dropped) | SURVIVED | `n_edge = 0` on every row |
| J7 | the control loop passes the caller's `phi_air` instead of `None` | KILLED — clock, clock_latched, clock_single | the split wall has no fuel point in `clip` (rung 81's headline), so every control census moves |
| J8 | `fuel_side` left in insertion order | KILLED — clock | three labels in `demand`; insertion order is by `tau_f` ascending, not string order |
| J9 | `n_differing` summed over later rows, not maxed | KILLED — clock (and latched? only 2 rows there ⇒ sum = max: SURVIVES in latched) | demand columns have 5 later rows |
| J10 | `ctl` drops the chained `tau_gov == 0.05` (only `tau_f == tau_gov`) | KILLED — clock | the diagonal's 0.02 and 0.20 cells join the control |
| J11 | the mask reads the CALLER's table (`core.triple_hooks()`) | SURVIVED | no swaps: same pointers — step 7's P4 |
| J12 | `rate` summed in reverse | SURVIVED | every quartic has one zero, far from the bar |
| J13 | `authority_mask`'s `lag.tau` arguments swapped (`g_fuel, required_fuel`) | KILLED — mask, mask_clip | the other clock on every non-tie point ⇒ `jac4` moves `c0`/`c1` |
| J14 | `predicted` with `<=` | SURVIVED | no exact tie between the two gaps expected |
| J15 | the control's `ccensus` = FIRST `tau_f == 0.05` row, not the last | SURVIVED | 0.05 appears once |
