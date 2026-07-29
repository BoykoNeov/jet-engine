"""Rung 62 — THE BLEED SCHEDULE beside the STATOR SCHEDULE, on the transient plant.

Rung 61 put rung 42's valve and rung 53's stator on one STEADY machine and closed with the
seam this rung answers: a b(n_L) schedule beside a v(n) schedule on the TRANSIENT plant,
"sharp now, because § 2 says the two devices' costs do not share and § 1 says their credits
do not stack."

THE HEADLINE: a state-fed schedule closes a FEEDBACK LOOP on itself through the shaft speed
it reads, and the loop's SIGN is the sign of the lever's own dn/d(setting). Rung 57 found
the stator schedule SELF-CANCELS (FULL/RAMP = 0.77-0.83) because closing stators raises n
and the schedule opens back up: (dn/dv)(dv/dn) = (+)(-) < 0. A handling bleed flips one
factor -- rung 61 § 2's own -9.77 % demand term is dn_L/db < 0 -- so the SAME instrument on
the SAME plant returns FULL/RAMP = 1.09-1.10: the bleed schedule AMPLIFIES itself. Both
signs were derivable from published tables before either was measured.

THE SECOND FINDING: the two loops close through ONE state and they do not compose. A bleed
SCHEDULE beside a stator schedule TRIPLES the stator's own surrender (0.169-0.229 ->
0.633-0.724) while the stator leaves the bleed's amplification alone to within 0.7 % -- a
one-way arrow running from the amplifying lever to the cancelling one. It is the LOOP and
not the LEVEL: a CONSTANT valve at the schedule's own commanded value (b = 0.0709) reaches
only 0.2265, and even an over-matched constant b = 0.10 reaches 0.2471.

CORRECTS RUNG 61. The same two devices are additive to <= 2.3 % on the steady matcher and
sub-additive by 9-29 % here. The near-additivity was the SHAFT BALANCE's doing -- the same
shape in which rung 57 corrected rung 53's exact zeros.

Reduces TWO-AXIS and per CALL: b = 0 dispatches to rung 57's own body verbatim, so
(v, b=0) is rung 57 bit-for-bit and (v=0, b=0) is rungs 43-52. The corner with NO transient
ancestor, (v=0, b!=0), is validated against rung 42's STEADY match through the forward
closure only -- which is what caught the `_powers` silent wrong number (n_L 5.3 % off with
phi_L still right to 1e-3 and no exception anywhere).
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    ScheduledStatorTransient, StatorSchedule, ScheduledBleedTransient, BleedSchedule,
    TwoSpoolBleedMatcher,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE = 1000.0, 1400.0, 0.01, 1.2

# n_lo is placed BELOW both levers' armed idle speeds (stator 0.799, bleed 0.737) so
# neither schedule is measured SATURATED. Rung 57's own 0.75574 leaves the bleed clipped at
# b_max, where db/dn = 0 and there is no loop to measure — the artifact the anchor doc
# publishes and this constant fixes.
N_LO, V, B = 0.65, 0.20, 0.10

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
TILT_LP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
TILT_HP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
SHAPES = {"shaped": (LP, HP), "tilted": (TILT_LP, TILT_HP)}

KEYS = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "pi_lpc", "pi_hpc",
        "Phi_lp", "Phi_hp", "sp_thrust", "m_lp", "m_hp", "Tt25", "Tt3")

STAT = dict(vsv_sched_lp=StatorSchedule(V, N_LO))
BLED = dict(bleed_sched=BleedSchedule(B, N_LO))


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _bt(lp=LP, hp=HP, design=None, **kw):
    return ScheduledBleedTransient(design if design is not None else _design(), FLIGHT,
                                   1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


# =============================================================================
# GATE 1 — THE REDUCE, TWO-AXIS and per CALL
# =============================================================================

@pytest.mark.parametrize("kw57,kw62,label", [
    ({}, {}, "(v=0, b=0)"),
    (dict(vsv_lp=V), dict(vsv_lp=V), "(v const, b=0)"),
    (dict(vsv_sched_lp=StatorSchedule(V, N_LO)),
     dict(vsv_sched_lp=StatorSchedule(V, N_LO)), "(v sched, b=0)"),
])
@pytest.mark.parametrize("Tt4", [1400.0, 1200.0, 1000.0])
def test_reduce_valve_shut_is_rung57_bit_for_bit(kw57, kw62, label, Tt4):
    """b == 0 dispatches to rung 57's own body VERBATIM at every state, so an unbled
    machine is rung 57 (hence rungs 43-52) bit-for-bit on every recorded key."""
    de = _design()
    a = ScheduledStatorTransient(de, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0, **kw57)
    c = _bt(design=de, **kw62)
    ea, ec = a.equilibrium(FLIGHT, Tt4), c.equilibrium(FLIGHT, Tt4)
    for k in KEYS:
        assert ea[k] == ec[k], f"{label} Tt4={Tt4} key {k}: {ea[k]!r} != {ec[k]!r}"


def test_reduce_zero_schedule_dispatches_rather_than_computing_unit_factors():
    """A BleedSchedule with b_max = 0.0 returns 0.0 at every n, at which point `_close`
    RETURNS TO ITS PARENT rather than multiplying by (1-0.0). The machinery is witnessed
    inert, not merely arithmetically neutral (rung 57's `is`-not-`==` discipline)."""
    de = _design()
    a = ScheduledStatorTransient(de, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
    z = _bt(design=de, bleed_sched=BleedSchedule(0.0, N_LO))
    ea, ez = a.equilibrium(FLIGHT, 1200.0), z.equilibrium(FLIGHT, 1200.0)
    for k in KEYS:
        assert ea[k] == ez[k], k
    # and the dispatch is real: a bled closure would have written a `bleed` key.
    assert "bleed" not in ez, "the b=0 path must reach rung 57's dict, not a bled one"
    assert "bleed" in _bt(design=de, bleed=0.05).equilibrium(FLIGHT, 1200.0)


def test_reduce_fuel_path_valve_shut_is_rung57_bit_for_bit():
    """The FUEL closure has its own dispatch (and its own bracket), so it needs its own
    witness — rungs 43-52 all live on `_close_fuel`."""
    de = _design()
    a = ScheduledStatorTransient(de, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
    c = _bt(design=de)
    mf = a.fuel_for_Tt4(FLIGHT, 1200.0)
    assert mf == c.fuel_for_Tt4(FLIGHT, 1200.0)
    ia = a._instant_fuel(FLIGHT, 0.85, 0.88, mf)
    ic = c._instant_fuel(FLIGHT, 0.85, 0.88, mf)
    for k in KEYS:
        assert ia[k] == ic[k], k


# =============================================================================
# GATE 2 — THE PLANT GATE. The (v=0, b!=0) corner has NO transient ancestor.
# =============================================================================

@pytest.mark.parametrize("Tt4", [1500.0, 1200.0, 1000.0])
@pytest.mark.parametrize("b", [0.05, 0.10])
def test_plant_forward_closure_reproduces_rung42_steady_match(Tt4, b):
    """THE GATE THAT CAUGHT THE SILENT WRONG NUMBER. Validated the way rung 40 validated
    itself: through the FORWARD closure only, never by calling the steady matcher.

    Rung 40 factored (Phi_L, Phi_H) out of `_instant_tail` into `_powers` for the Newton's
    inner loop. With `_powers` left bleed-free the Newton converges to 1e-12 on a residual
    the plant does not use and returns n_L = 0.8720 against a true 0.8282 — 5.3 % wrong,
    with phi_L still agreeing to 1e-3 and NO exception anywhere. Nothing internal to the
    transient ladder can see that; only this cross-object comparison can."""
    de = _design()
    eq = _bt(design=de, bleed=b).equilibrium(FLIGHT, Tt4)
    od = TwoSpoolBleedMatcher(de, FLIGHT, 1.0, map_lp=LP, map_hp=HP, bleed=b).match(
        FLIGHT, Tt4)
    for name, got, want in (("n_lp", eq["n_lp"], od.n_lp),
                            ("phi_lp", eq["phi_lp"], od.phi_lp),
                            ("phi_hp", eq["phi_hp"], od.phi_hp),
                            ("pi_lpc", eq["pi_lpc"], od.pi_lpc),
                            ("pi_hpc", eq["pi_hpc"], od.pi_hpc)):
        assert abs(got / want - 1.0) < 1e-9, (
            f"Tt4={Tt4} b={b} {name}: forward {got!r} vs rung-42 steady {want!r}")


def test_plant_the_burner_sees_CORE_air_only():
    """THE ONE PLACE THE BLEED CHANGES THE CONTROL and not just the flow. Every finding in
    this rung runs through `_close_fuel`'s bleed branch, but the reduce gates above only
    exercise its b == 0 dispatch — so without this the branch executes constantly and is
    never asserted.

    At a FIXED state and a FIXED metered fuel flow, the burner never sees the dumped air:
    f is CORE-referenced (an exact identity), the face carries 1/(1-b) more, and the
    mixture is therefore RICHER and the turbine entry HOTTER than with the valve shut."""
    t0, tb = _bt(), _bt(bleed=0.10)
    Tt2, pt2, _ = t0._inlet(FLIGHT)
    mf = t0.fuel_for_Tt4(FLIGHT, 1200.0)
    a = t0._close_fuel(0.85, 0.88, mf, Tt2, pt2)
    b = tb._close_fuel(0.85, 0.88, mf, Tt2, pt2)
    # f x CORE air recovers the metered fuel. This closes only AT the root (the returned
    # `mdot_air` is the choke-IMPLIED core flow, and `f` was formed from the trial FACE
    # flow), so it is asserted at the closure's own tolerance rather than bit-exactly.
    assert abs(b["f"] * b["mdot_air"] / mf - 1.0) < 1e-9, "f is CORE-referenced"
    assert abs(a["f"] * a["mdot_air"] / mf - 1.0) < 1e-9, "and so is the b=0 path"
    assert abs(b["mdot_face"] / b["mdot_air"] - 1.0 / 0.9) < 1e-12   # the extraction, exact
    assert b["f"] > a["f"] and b["Tt4"] > a["Tt4"], "same fuel, less air => richer, hotter"


def test_plant_the_powers_touch_point_is_not_optional():
    """A DIRECT witness that `_powers` and `_instant_tail` agree under bleed — the two
    sites rung 40 split apart. If a future edit restores one and not the other, the
    equilibrium Newton silently converges on the wrong residual again."""
    t = _bt(bleed=0.10)
    Tt2, pt2, _ = t._inlet(FLIGHT)
    c = t._close(0.85, 0.88, 1200.0, Tt2, pt2)
    p_lp, p_hp = t._powers(c, FLIGHT, 0.85, 0.88, 1200.0)
    inst = t._instant(FLIGHT, 0.85, 0.88, 1200.0)
    assert p_lp == inst["Phi_lp"] and p_hp == inst["Phi_hp"]
    assert c["bleed"] == 0.10 and c["mdot_face"] > c["mdot_air"]


def test_plant_schedule_is_shut_at_the_design_speed():
    """Rung 42 captures A4/A45/A8 and both maps' references with the valve SHUT, so a
    schedule holding it open at n_ref would contradict every design reference."""
    assert BleedSchedule(B, N_LO)(1.0) == 0.0
    assert BleedSchedule(B, N_LO, shape="linear")(1.0) == 0.0
    with pytest.raises(AssertionError):
        BleedSchedule(B, 1.2)                       # n_lo >= n_ref
    with pytest.raises(AssertionError):
        BleedSchedule(0.6, N_LO)                    # rung 42's own b < 0.5 bound
    with pytest.raises(AssertionError):             # a position OR a schedule, not both
        _bt(bleed=0.05, bleed_sched=BleedSchedule(B, N_LO))


def test_plant_stays_in_the_choked_scope_at_the_idle_end():
    """A b(n_L) schedule is MOST open at Tt4_lo, and rung 42 warns its choked guard "bites
    SOONER" with the valve open. Checked rather than assumed."""
    de = _design()
    for Tt4 in (900.0, LO):
        for b in (0.10, 0.20, 0.30):
            assert _bt(design=de, bleed=b).equilibrium(FLIGHT, Tt4)["branch"] == "choked"


def test_trap_at_stator_carries_the_valve():
    """RUNG 61's `at_setting` TRAP, one ladder over. Rung 57 hard-constructs
    `ScheduledStatorTransient` in `at_stator`, and `stator_credit` / `credit_decomposition`
    / `arrow_toggle` all route their BARE leg through it. Un-overridden, every one would
    difference an armed machine against a VALVE-SHUT bare one and attribute the valve's
    whole effect to the stator — plausible numbers, no exception."""
    t = _bt(bleed=0.10)
    sib = t.at_stator(vsv_lp=V)
    assert isinstance(sib, ScheduledBleedTransient)
    assert sib.bleed == 0.10 and sib.vsv_lp == V
    s2 = _bt(**BLED).at_stator(vsv_sched_lp=StatorSchedule(V, N_LO))
    assert s2.bleed_sched is not None and s2.vsv_sched_lp is not None


# =============================================================================
# GATE 3 — THE HEADLINE: the loop gain's SIGN
# =============================================================================

@pytest.mark.parametrize("Tt4", [1500.0, 1300.0, 1100.0, 900.0])
def test_headline_both_loop_factors_have_their_sign_and_neither_reverses(Tt4):
    """The sign argument rests on two derivatives, and rung 42's own dphi_H/db REVERSES at
    pi* = 3.24674 — so they are measured, not quoted."""
    row = _bt().loop_factors(FLIGHT, [Tt4])[0]
    assert row["dn_db"] < 0.0, f"Tt4={Tt4}: dn_L/db = {row['dn_db']}"
    assert row["dn_dv"] > 0.0, f"Tt4={Tt4}: dn_L/dv = {row['dn_dv']}"


@pytest.mark.slow
@pytest.mark.parametrize("r", [0.25, 0.50, 1.00])
def test_headline_the_two_schedules_land_on_OPPOSITE_SIDES_of_one(r):
    """THE RUNG. Same instrument, same plant, same n_lo, same ramp: the stator schedule
    surrenders authority to its own loop and the bleed schedule GAINS it."""
    de = _design()
    s = _bt(design=de, **STAT).loop_decomposition(FLIGHT, LO, HI, r=r)
    b = _bt(design=de, **BLED).loop_decomposition(FLIGHT, LO, HI, r=r)
    assert s["self_cancel"] < 1.0, f"stator r={r}: {s['self_cancel']}"
    assert b["self_cancel"] > 1.0, f"bleed  r={r}: {b['self_cancel']}"
    # and the sizes rung 62 publishes, as bands rather than points
    assert 0.75 < s["self_cancel"] < 0.85
    assert 1.08 < b["self_cancel"] < 1.11


@pytest.mark.slow
@pytest.mark.parametrize("r", [0.25, 1.00])
def test_headline_the_loop_is_witnessed_in_the_COMMANDED_SETTING(r):
    """Not a ratio of credits: between the RAMP and FULL legs the two schedules move their
    own commanded setting in OPPOSITE directions. The stator backs off; the bleed leans in.
    This is the loop itself, and it needs no normalisation to read."""
    de = _design()
    s = _bt(design=de, **STAT).loop_decomposition(FLIGHT, LO, HI, r=r)
    b = _bt(design=de, **BLED).loop_decomposition(FLIGHT, LO, HI, r=r)
    assert s["cmd_full"] < s["cmd_ramp"], f"stator: {s['cmd_ramp']} -> {s['cmd_full']}"
    assert b["cmd_full"] > b["cmd_ramp"], f"bleed:  {b['cmd_ramp']} -> {b['cmd_full']}"
    # the head start's own sign, which is what drives both: the stator raises the armed
    # idle, the bleed lowers it.
    assert s["nu0_armed"] > s["nu0_ref"] and b["nu0_armed"] < b["nu0_ref"]


@pytest.mark.slow
@pytest.mark.parametrize("shape", ["smooth", "linear"])
def test_headline_survives_the_schedule_SHAPE(shape):
    """`smooth` is C1 at both corners (S' = 0 there); `linear` is not. The sign must not
    be a property of the flat spot."""
    de = _design()
    s = _bt(design=de, vsv_sched_lp=StatorSchedule(V, N_LO, shape=shape)
            ).loop_decomposition(FLIGHT, LO, HI, r=0.25)
    b = _bt(design=de, bleed_sched=BleedSchedule(B, N_LO, shape=shape)
            ).loop_decomposition(FLIGHT, LO, HI, r=0.25)
    assert s["self_cancel"] < 1.0 < b["self_cancel"]


@pytest.mark.slow
def test_headline_is_grid_converged():
    """The composite ratios below are differences of small marginal credits, so the RK4
    grid is checked rather than trusted."""
    de = _design()
    vals = {}
    for ds in (0.02, 0.01, 0.005):
        vals[ds] = (_bt(design=de, **STAT).loop_decomposition(FLIGHT, LO, HI, r=0.25,
                                                              ds=ds)["self_cancel"],
                    _bt(design=de, **BLED).loop_decomposition(FLIGHT, LO, HI, r=0.25,
                                                              ds=ds)["self_cancel"])
    for i in (0, 1):
        lo = min(v[i] for v in vals.values())
        hi = max(v[i] for v in vals.values())
        assert (hi - lo) / lo < 0.02, f"leg {i} moves {(hi - lo) / lo:.4f} across the grid"


@pytest.mark.slow
def test_headline_is_NOT_a_saturated_schedule_artifact():
    """THE ARTIFACT THIS RUNG PUBLISHES. At rung 57's own n_lo = 0.75574 the bleed's head
    start pushes nu0 BELOW n_lo, where S clips to 1, b == b_max and db/dn = 0 — there is no
    loop left to measure. The SIGN survives it, but the magnitude halves, so the placement
    is load-bearing and is asserted rather than left to the reader.

    The saturation itself is gated EXACTLY (the schedule's own clip), not through an
    arithmetic proxy: at the bad placement the armed machine idles below n_lo and therefore
    commands b_max identically, while at the placed one it commands strictly less."""
    de = _design()
    sat_sched, free_sched = BleedSchedule(B, 0.75574), BleedSchedule(B, N_LO)
    sat = _bt(design=de, bleed_sched=sat_sched).loop_decomposition(FLIGHT, LO, HI, r=0.25)
    free = _bt(design=de, bleed_sched=free_sched).loop_decomposition(FLIGHT, LO, HI, r=0.25)
    # THE SATURATION, exactly: S clipped to 1 => the schedule is pinned at b_max.
    assert sat_sched(sat["nu0_armed"]) == B, "the bad placement must really be clipped"
    assert free_sched(free["nu0_armed"]) < B, "the good placement must be off the clip"
    # the sign survives, the magnitude does not
    assert sat["self_cancel"] > 1.0 and free["self_cancel"] > 1.0
    assert free["self_cancel"] - 1.0 > 1.8 * (sat["self_cancel"] - 1.0)


# =============================================================================
# GATE 4 — THE SECOND FINDING: the loops do NOT compose, and the arrow is ONE-WAY
# =============================================================================

@pytest.mark.slow
@pytest.mark.parametrize("r", [0.25, 0.50, 1.00])
def test_second_finding_a_bleed_SCHEDULE_triples_the_stators_surrender(r):
    """P3 predicted the bleed's positive loop would RESTORE part of what the stator's
    negative loop surrenders. Refuted with the opposite sign — it triples it. The
    neighbour is carried on BOTH sides of the difference, so what is measured is the
    stator schedule's own loop and not the pair's composite."""
    t = _bt()
    alone = t.marginal_loop(FLIGHT, LO, HI, lever=STAT, r=r)
    beside = t.marginal_loop(FLIGHT, LO, HI, lever=STAT, neighbour=BLED, r=r)
    assert 0.15 < alone["surrendered"] < 0.25
    assert beside["surrendered"] > 2.5 * alone["surrendered"]
    assert beside["surrendered"] > 0.60


@pytest.mark.slow
@pytest.mark.parametrize("r", [0.25, 1.00])
def test_second_finding_the_arrow_is_ONE_WAY(r):
    """The mirror: a stator schedule barely touches the bleed schedule's amplification."""
    t = _bt()
    alone = t.marginal_loop(FLIGHT, LO, HI, lever=BLED, r=r)
    beside = t.marginal_loop(FLIGHT, LO, HI, lever=BLED, neighbour=STAT, r=r)
    assert alone["self_cancel"] > 1.0 and beside["self_cancel"] > 1.0
    assert abs(beside["self_cancel"] / alone["self_cancel"] - 1.0) < 0.02


@pytest.mark.slow
@pytest.mark.parametrize("r", [0.25, 0.50, 1.00])
def test_second_finding_is_the_LOOP_and_not_the_LEVEL(r):
    """THE CONTROL THAT MAKES IT MEAN ANYTHING. A CONSTANT valve has no loop of its own.
    Matched at the value the schedule actually commands at its own surge minimum — and
    even OVER-matched at b_max, which is strictly more lever than the schedule ever
    applies — a constant moves the stator's surrender a fraction as far as the schedule
    does. Without this leg the finding would be indistinguishable from "more bleed"."""
    t = _bt()
    cmd = _bt(**BLED).commanded_level(FLIGHT, LO, HI, r=r)["at_min"]
    assert cmd < B, "the schedule must command LESS than b_max for this to be a control"
    alone = t.marginal_loop(FLIGHT, LO, HI, lever=STAT, r=r)["surrendered"]
    matched = t.marginal_loop(FLIGHT, LO, HI, lever=STAT,
                              neighbour=dict(bleed=cmd), r=r)["surrendered"]
    over = t.marginal_loop(FLIGHT, LO, HI, lever=STAT,
                           neighbour=dict(bleed=B), r=r)["surrendered"]
    sched = t.marginal_loop(FLIGHT, LO, HI, lever=STAT,
                            neighbour=BLED, r=r)["surrendered"]
    # a constant does move it a little, in the same direction
    assert alone < matched < over < sched
    # but the schedule does 2.3-2.9x what the strictly LARGER constant does
    assert sched > 2.2 * over


@pytest.mark.slow
def test_second_finding_mechanism_the_head_start_is_ENLARGED():
    """The measured mechanism: as the stator raises n the bleed schedule CLOSES, which
    raises n further, so the stator's own head start is larger in the pair than alone.
    The small-signal "two loops multiply" algebra is NOT asserted — only this."""
    t = _bt()
    n_bare = t.at_lever().equilibrium(FLIGHT, LO)["nu_lp"]
    n_stat = t.at_lever(**STAT).equilibrium(FLIGHT, LO)["nu_lp"]
    n_bled = t.at_lever(**BLED).equilibrium(FLIGHT, LO)["nu_lp"]
    n_pair = t.at_lever(**{**STAT, **BLED}).equilibrium(FLIGHT, LO)["nu_lp"]
    assert n_stat > n_bare > n_bled                      # the two head starts' signs
    assert n_pair - n_bare > (n_stat - n_bare) + (n_bled - n_bare)   # super-additive
    assert n_pair - n_bled > 1.15 * (n_stat - n_bare)    # the stator's is ENLARGED


# =============================================================================
# GATE 5 — CORRECTS RUNG 61: the steady near-additivity was the SHAFT BALANCE's
# =============================================================================

@pytest.mark.slow
@pytest.mark.parametrize("name", list(SHAPES))
@pytest.mark.parametrize("r", [0.25, 0.50, 1.00])
def test_corrects_rung61_credits_are_sub_additive_on_the_transient(name, r):
    """Rung 61 measured these two devices additive to <= 2.3 % on the STEADY matcher. Rung
    40 removed the shaft balance; the same pair is sub-additive by an order more here."""
    lp, hp = SHAPES[name]
    d = _bt(lp, hp).pair_interaction(FLIGHT, LO, HI, lever_a=STAT, lever_b=BLED, r=r)
    assert d["interaction"] < 0.0, "sub-additive, not synergistic"
    assert 0.08 < -d["interaction_frac"] < 0.32, d["interaction_frac"]
    assert -d["interaction_frac"] > 3.0 * 0.023, "must clear rung 61's steady 2.3 %"


@pytest.mark.slow
@pytest.mark.parametrize("name", list(SHAPES))
@pytest.mark.parametrize("r", [0.25, 0.50, 1.00])
def test_corrects_rung61_adverse_SPEED_cost_interaction_survives(name, r):
    """Rung 61's cost interaction was positive in all 30 steady rows. It survives the
    transplant. Asserted RAW: `cost_b` is negative while `cost_a` is positive, so a
    normalised interaction would put a difference of opposite-signed terms in its
    denominator — rung 43's currency-circularity trap."""
    lp, hp = SHAPES[name]
    d = _bt(lp, hp).pair_interaction(FLIGHT, LO, HI, lever_a=STAT, lever_b=BLED, r=r)
    assert d["cost_a"] > 0.0 > d["cost_b"], "the two levers pay in opposite speed signs"
    assert d["cost_interaction"] > 0.0, d["cost_interaction"]


# =============================================================================
# GATE 6 — THE CONTROL that is explicitly NOT a finding (rung 57 already said it)
# =============================================================================

@pytest.mark.slow
def test_control_ramp_invariance_is_a_WALL_MOVER_property():
    """Rung 57 § 2 ALREADY names the mechanism (both its channels are algebraic in the
    instantaneous state), so "the bleed has no clock" is a CONFIRMATION and is gated as a
    control. What is new is the complementary case, and its signature is MONOTONICITY:
    a wall-mover's floor channel contributes exactly `v` whatever the trajectory does, so
    its credit/setting wobbles non-monotonically at the 0.4 % level; a point-mover's whole
    credit runs through phi and decays strictly monotonically with ramp rate."""
    t = _bt()
    bl = [x["per_setting"] for x in t.clock_sweep(FLIGHT, LO, HI, dict(bleed=B), B)]
    st = [x["per_setting"] for x in t.clock_sweep(FLIGHT, LO, HI, dict(vsv_lp=V), V)]
    assert all(bl[i] > bl[i + 1] for i in range(len(bl) - 1)), bl
    assert not all(st[i] > st[i + 1] for i in range(len(st) - 1)), st
    assert (max(st) - min(st)) / min(st) < 0.04
    assert (max(bl) - min(bl)) / min(bl) > 3.0 * (max(st) - min(st)) / min(st)


# =============================================================================
# GATE 7 — CYCLE UNTOUCHED
# =============================================================================

def test_cycle_untouched_design_run_is_bit_for_bit_rung6():
    """Rung 62 adds a transient plant and reads on it. The default single-spool design run
    must be untouched — the project's spine since rung 7."""
    kw = {k: v for k, v in REAL.items() if k in ("pi_d", "eta_b", "pi_b", "eta_m", "pi_n")}
    res = build_turbojet(gas=Gas.reacting_equilibrium(), pi_c=PI_LPC * PI_HPC, Tt4=TT4,
                         p_ambient=FLIGHT.p0, **kw).run(FLIGHT, 1.0)
    ref = build_turbojet(gas=Gas.reacting_equilibrium(), pi_c=PI_LPC * PI_HPC, Tt4=TT4,
                         p_ambient=FLIGHT.p0, **kw).run(FLIGHT, 1.0)
    assert res.performance.specific_thrust > 0.0 and res.performance.tsfc > 0.0
    for st in ("2", "3", "4", "5", "9"):
        assert res.stations[st].Tt == ref.stations[st].Tt
        assert res.stations[st].pt == ref.stations[st].pt
    assert res.performance.specific_thrust == ref.performance.specific_thrust


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
