# Rungs 90–94 — What the aircraft takes from the engine

*Plain-language notes. The formal derivations, tests and every number's source are in `docs/rung90-spec.md`
to `docs/rung94-spec.md`. Numbers below are copied from those specs and belong to the cases the specs ran; the
sandbox page shows its own.*

An engine on an aircraft does more than push. The aircraft takes **air** from it (for the cabin, for de-icing)
and **power** from it (a generator on a shaft). The engine also feeds part of its own air back to its hottest
part, to **cool the turbine**. These five rungs put those loads on the two-shaft engine and ask what each
costs: in thrust, in fuel, and in distance from stall.

**What you need from earlier rungs.** The engine has two shafts (rung 38): a front, low-pressure (LP)
compressor and a high-pressure (HP) compressor behind it, each driven by its own turbine. Each compressor has its
own distance from stall, read as its **flow coefficient** (rung 36: airflow measured against blade speed; lower
is closer to stall). Rung 41 found the LP compressor the more exposed of the two, and it stays so through all
five rungs. Rung 42 had already looked at one kind of bleed, air let out between the two compressors to protect
the LP one, and rung 64 made that a **protection valve** that opens as the LP compressor nears stall. Several
results below are about how hard that valve has to work.

Each of these rungs wrote its predictions down before any of its code existed, and then scored them.

---

## Rung 90 — Air for the cabin

**In one sentence:** air taken from behind the HP compressor moves the HP compressor away from stall at every
power, but moves the LP compressor toward stall at high power, so the protection valve has to bleed more.

**How it works.** The aircraft's air is taken where the HP compressor delivers it, and never comes back. So the
HP compressor pumps more air than the burner and turbines pass. That extra air through its front face moves it
away from stall. The LP compressor sees the other side: the HP compressor's pressure ratio falls, so it swallows less
of what the LP compressor delivers, the LP airflow falls against its speed, and above a crossing at about **910 K** (the spec's case) this
moves the LP compressor toward stall. Below it the missing air helps.

**On a slam.** The LP compressor comes closest to stall on the high-power side of that crossing. So with 5 % of
the HP compressor's air going to the aircraft, the protection valve has to bleed more to hold the same margin
(**17 to 760 %** more over the spec's cases). A fixed *amount* of air (what a cabin actually asks for) costs it
more again, because at the low-power start of a slam the same amount is a bigger share of a smaller flow.

**Later correction.** Rung 92 corrected the *reason* for the HP compressor's relief. Rung 90 had credited the
HP turbine having less gas to drive its compressor with. In fact the lost work pushes the HP compressor *toward*
stall; the relief comes from the extra air through its face, and that wins. The verdict, away from stall, stands.

**On the page.** *Controls → Aircraft takes a share / a fixed amount*. *Try this → Air for the cabin*. In the
page's slam (`docs/sandbox/EXPERIMENTS.md`, experiment 17) 5 % of the HP compressor's air costs the most thrust of
all the loads: 537 against 615 with none. That air had already been compressed at full cost.

---

## Rung 91 — Air that cools the turbine and comes back

**In one sentence:** cooling air that rejoins the gas at the turbines costs little thrust but a lot of stall
margin, the reverse of the cabin's air.

**How it works.** Turbine blades run in gas hotter than their metal can stand. They survive on air taken from
the HP compressor and blown through them. The model has two streams: **vane air**, which cools the fixed vanes
and rejoins before the HP turbine's rotor (so it still does work there), and **blade air**, which cools the
rotor blades and rejoins after it.

Either way the air comes back *in front of* the LP turbine's entrance, a choked throat (rung 31). That throat
must now pass more gas, so the pressure in front of it rises and the HP turbine cannot expand as far. The HP
compressor loses *more* work than if the air had been thrown away. So:
- the HP compressor is pushed toward stall at high power, above about **1116 K** for vane air and **795 K** for
  blade air: rung 90's HP relief is reversed;
- the LP compressor is pushed toward stall **at every power**: the cabin air's low-power relief is gone;
- blade air is worse than vane air everywhere: it chokes the throat just as much and does no work in the HP
  turbine.

**The other side of the bill.** The air goes out through the nozzle, so its thrust cost is small: 5 % of vane
air costs **−2.9 %** of thrust against **−8.3 %** for 5 % of cabin air (at 1500 K). But on a slam the protection
valve bleeds **2–12×** (vane) and **3–29×** (blade) what it bleeds for the cabin share.

**What it does to rung 31.** Rung 31's "the HP turbine sits at one fixed point" needs both choked throats to
pass the same gas. Cooling air between them moves it.

**On the page.** *Controls → Turbine cooling, vane air / blade air*. On its own, at a fixed burner temperature,
cooling can only look like a cost. Rung 93 shows what it buys.

---

## Rung 92 — A generator on a shaft

**In one sentence:** a bleed is exactly a power offtake plus air thrown away, and that shows what the bleeds
hid: a generator on the HP shaft pushes **both** compressors toward stall at every power, while the same
generator on the LP shaft costs them almost nothing.

**The exact identity.** Take a share of the LP shaft's power, and the engine runs exactly as with rung 42's bleed
of the same share, except the air is kept: every speed and every pressure ratio identical, only the flow
coefficients differ by that share. Take it from the HP shaft too, and it is rung 90's cabin bleed with the air
kept. So a generator is the half of a bleed that never leaves.

**What that reveals.**
- **On the HP shaft** the HP compressor gets less work, so its pressure ratio falls, and that starves the LP
  compressor too. Both move toward stall at every power, no crossing. Rung 90's HP compressor was helped only
  because its air *left*.
- **On the LP shaft** the LP compressor's own loss is almost offset by the HP compressor's pressure rising. In
  the steady state it is nearly free. That is why more-electric aircraft studies take power off the LP shaft.
- The thrust and fuel bill is the same on either shaft.

**On a slam.** With the load on the HP shaft the protection valve bleeds more, and the HP shaft's speed-up comes
**1.3–1.6 %** later. **The one prediction that missed:** the small steady HP relief of an LP-shaft load does not
survive a fast slam.

**On the page.** *Controls → On the HP shaft / On the LP shaft*, as a fixed power or a share. *Try this → A
generator on the HP shaft*. In experiment 17 the same generator costs the same thrust on either shaft (591 against
615).

---

## Rung 93 — The metal's temperature: what cooling buys

**In one sentence:** cooling air lets the burner run hotter, by more than enough to pay for itself, so there
is no best amount of cooling; and on a built engine, cooling cools its own coolant.

**How it works.** The model reads the vane and blade metal temperatures with a published NASA relation
(Gauntner, 1980): from the gas temperature, the cooling air's temperature, how much cooling air there is, and the
cooling **technology**. There are four, roughly in historical order: **convection** (air through internal
passages), **coated** (the same plus a heat-shield coating), **film** (air bled out through holes to form a cool
layer on the surface) and **transpiration** (air seeping through a porous wall). The metal's limit comes from
the material's year and the part's design life.

**What it found.**
- **Cooling never stops the engine.** Each percent of vane air buys about **100 K** of burner temperature and
  costs about half a percent of thrust. So thrust at the metal limit keeps rising with burner temperature through
  2200 K, for every technology. The limits on a hot engine lie elsewhere: in shaft speed, in airflow.
- **A built engine cools its own coolant.** On fixed hardware, more cooling air slows the HP shaft, so the
  compressor delivers cooler air, which is better coolant. The built engine needs **5 %** less cooling than a
  naive sum at 1500 K and **22 %** less at 2000 K (film cooling).
- **Built and freshly designed engines trade places.** Below a crossing the built engine needs less cooling air
  than one redesigned for each temperature, above it more. The crossing **rises as the technology worsens**:
  1624 K (transpiration), 1672 K (film), 1875 K (coated), above 2200 K (convection).
- **On a fresh design, cooling lowers the fuel bill** at a fixed burner temperature: by 0.18 to 2.9 mg of fuel
  per newton-second (film, 1300 → 2200 K).

**On the page.** The *Cool the turbine* view; and *Controls → Read the turbine metal*, which reads the metal along
a slam. In experiment 19, with the page's opening cooling air, the hottest burner the metal allows is 1422 K
(convection), 1512 K (coated), 1638 K (film), 1709 K (transpiration); with no cooling air, 991 K. *Try this* on
that view switches technology and air.

---

## Rung 94 — A burner that carries the load

**In one sentence:** if the throttle holds *thrust* instead of burner temperature, the burner runs hotter to
carry the load, and that **relieves** the compressors; holding HP shaft speed instead is not the same as holding
thrust.

**How it works.** Rungs 90–92 kept the burner temperature fixed and let thrust fall. A real control would not:
it burns hotter to keep giving the thrust asked for. Here each end of a throttle move asks for what the engine
gives *without* the load, either its thrust or its HP shaft speed, and the burner temperature floats to deliver
it. The metal limit of rung 93 is reported, or can be switched on to stop the burner.

**What it found.**
- **Holding thrust,** the burner runs hotter (in the spec's case about **49 K** for a generator taking 5 % of the
  HP compressor's design power, at 1500 K; **84 K** at 1000 K) and pushes more air. The LP compressor sits
  further from stall than at a fixed burner temperature, at every power. An LP-shaft generator carried this way
  leaves *both* compressors further from stall than no generator at all (1000–1500 K).
- **Holding HP speed is not holding thrust.** With the generator on the HP shaft the speed hold runs much hotter
  than the thrust hold and gives *more* thrust (**+4 %** at 1500 K, **+15 %** at 700 K). With it on the LP shaft
  the speed hold can even run the burner cooler and lose more thrust.
- **A load can set a floor.** On the spec's test engine a fixed 5 % generator on the LP shaft cannot run below
  **143 N** of thrust, against **58 N** on the HP shaft. Below that, no throttle can hold it.

**What it does to rung 92.** Rung 92 conceded that a fixed burner temperature "understates" a generator's stall
effects. Wrong way round: it *overstates* them, for the LP compressor at every power and for the HP compressor
above idle.

**On the page.** *Controls → Throttle holds burner temperature / Holds thrust / Holds HP speed*, and *Stop the
burner at the metal limit*. The same switch drives rung 95's governor, in the next family.
