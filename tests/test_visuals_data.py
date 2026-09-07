"""Guard: the `docs/visuals/` pages are the MODEL's, and they are what the build
scripts actually produce.

`docs/visuals/` holds two published HTML pages — the rung-1..23 chart page
(`turbojet-visuals.html`) and the animated engine cutaway (`turbojet-cutaway.html`).
Both are *built*: a `template.html` with a placeholder, spliced with `data.json`,
which `extract_data.py` dumps by running the real `turbojet` package at the
design point. The pages' whole claim is "every number here is the model's."

Nothing enforced that claim. Four ways it could rot silently, all of them
invisible to the rest of the suite because no test had ever opened these files:

  1. `extract_data.py` runs for ~10 minutes and is therefore run by hand, rarely.
     A cycle change moves the model and leaves `data.json` on yesterday's numbers.
  2. The built `.html` is committed beside its template, so a hand-edit to the
     built file — or a template edit with the rebuild forgotten — desynchronises
     the two with no symptom.
  3. `build_cutaway.py` TRIMS `data.json` to a `KEEP` list. A field added to the
     template's readouts but not to `KEEP` renders `undefined` in the browser and
     is invisible to every Python test in the repo. (`eta_brayton` is in
     `data.json` and deliberately not in `KEEP` — that is the shape of the trap.)
  4. `extract_data.py` used to re-declare the design point (`FLIGHT` / `PI_C` /
     `TT4` / `REAL_LOSSES`) rather than importing `main.py`'s, so if `main.py`'s
     moved, the pages would describe a different engine under the same name. It
     imports them now; this file pins both the import and the committed dump.

So this file gates the JOINTS, not the physics. The physics gates live in the
per-rung files; the sweep blocks here are ILLUSTRATION curves on deliberately
reduced grids (see `docs/visuals/README.md`) and are checked only for shape.

WHAT IS AND IS NOT RECOMPUTED. The `ideal` / `real` CPG cycle is two
`build_turbojet(...).run(...)` calls — milliseconds — so it is recomputed and
compared. Everything else in `data.json` (the bell, quench, J-sweep, spatial,
ladder and dwell blocks) costs ~10 minutes and is NEVER recomputed here; a
structural check is all this file will ever do for those.

IF A TEST HERE FAILS, the fix is almost always to re-run the build, not to
change this file:

    python docs/visuals/extract_data.py    # only if the MODEL moved (~10 min)
    python docs/visuals/build.py
    python docs/visuals/build_cutaway.py

then republish the two artifacts (their URLs are in `memory/visuals-artifact.md`
and `memory/cutaway-artifact.md`).
"""
import json
import math
import os
import re
import sys
import importlib.util

_HERE = os.path.dirname(os.path.abspath(__file__))
_ROOT = os.path.abspath(os.path.join(_HERE, os.pardir))
_VIS = os.path.join(_ROOT, "docs", "visuals")


def _text_path(path):
    with open(path, encoding="utf-8") as fh:
        return fh.read()


def _text(name):
    # read_text, NOT bytes: `build.py` writes without newline="\n" while
    # `build_cutaway.py` pins it, so the two built files need not agree on line
    # endings and a byte comparison would report a difference that is not one.
    # (CLAUDE.md was bitten by exactly this twice — see test_claude_md_reference.)
    with open(os.path.join(_VIS, name), encoding="utf-8") as fh:
        return fh.read()


def _data():
    return json.loads(_text("data.json"))


def _build_cutaway_module():
    """Import `docs/visuals/build_cutaway.py` for its `KEEP` list.

    Imported rather than copied so the trim this file checks IS the trim the
    build performs. Its `main()` is `__main__`-guarded, so importing writes
    nothing.
    """
    path = os.path.join(_VIS, "build_cutaway.py")
    spec = importlib.util.spec_from_file_location("_visuals_build_cutaway", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _trimmed_payload():
    """Reproduce build_cutaway.py's trimmed payload (the JSON the cutaway sees)."""
    keep = _build_cutaway_module().KEEP
    d = _data()
    return {"design": d["design"],
            "ideal": {k: d["ideal"][k] for k in keep},
            "real": {k: d["real"][k] for k in keep}}


# --------------------------------------------------------------- 1. the splices

def test_cutaway_html_is_the_current_splice():
    """turbojet-cutaway.html == cutaway-template.html spliced with data.json."""
    tpl = _text("cutaway-template.html")
    assert tpl.count("/*__DATA__*/") == 1, "cutaway template placeholder missing/duplicated"
    expected = tpl.replace("/*__DATA__*/",
                           json.dumps(_trimmed_payload(), separators=(",", ":")))
    assert expected == _text("turbojet-cutaway.html"), (
        "docs/visuals/turbojet-cutaway.html is not the splice of its template and "
        "data.json — either the built page was hand-edited, or a template/data change "
        "was committed without `python docs/visuals/build_cutaway.py`. Rebuild, then "
        "republish the artifact (memory/cutaway-artifact.md)."
    )


def test_visuals_html_is_the_current_splice():
    """turbojet-visuals.html == template.html spliced with data.json.

    build.py's splice is two lines and is reproduced here rather than imported:
    that module has no `__main__` guard, so importing it would REWRITE the built
    page — a test must not mutate the repo it is checking.
    """
    tpl = _text("template.html")
    assert tpl.count("/*__DATA_JSON__*/") == 1, "visuals template placeholder missing/duplicated"
    expected = tpl.replace("/*__DATA_JSON__*/", _text("data.json"), 1)
    assert expected == _text("turbojet-visuals.html"), (
        "docs/visuals/turbojet-visuals.html is not the splice of its template and "
        "data.json — hand-edited, built without `python docs/visuals/build.py`, or "
        "data.json was REFORMATTED: build.py splices it verbatim (build_cutaway.py "
        "re-serializes a trim, so a whitespace-only change breaks this one and not "
        "that one). Rebuild, then republish the artifact (memory/visuals-artifact.md)."
    )


# --------------------------------------------------- 2. the design point is ONE

def test_extract_data_imports_the_design_point_rather_than_copying_it():
    """The generator must not carry its own copy of the design point.

    `extract_data.py` used to re-declare `FLIGHT` / `PI_C` / `TT4` /
    `REAL_LOSSES` verbatim from `main.py`. It now imports them, which is the
    structural fix; this pins that, because the duplication is the kind an editor
    reintroduces without noticing (it is four short lines and looks harmless).
    """
    src = _text_path(os.path.join(_VIS, "extract_data.py"))
    assert re.search(r"^from main import .*\bFLIGHT\b", src, re.M), (
        "docs/visuals/extract_data.py no longer imports FLIGHT from main.py. If the "
        "design point is declared locally again, the pages can describe a different "
        "engine than the project does under the same name."
    )
    for name in ("PI_C", "TT4", "REAL_LOSSES"):
        assert re.search(rf"^from main import .*\b{name}\b", src, re.M), (
            f"docs/visuals/extract_data.py no longer imports {name} from main.py."
        )
        assert not re.search(rf"^{name}\s*(,|=)", src, re.M), (
            f"docs/visuals/extract_data.py assigns {name} locally as well as importing "
            "it — one of the two is dead, and which one wins is not obvious to a reader."
        )


def test_data_json_design_point_is_main_pys():
    """The COMMITTED dump was taken at the project's design point.

    The import above stops the two declarations diverging; this catches the other
    half, which an import cannot: `data.json` is committed and `extract_data.py`
    runs for ~10 minutes, so the dump can sit at yesterday's design point long
    after the source of it moved. `data.json`'s `design` block records what the
    script actually ran at, which makes the DATA the thing compared.
    """
    sys.path.insert(0, _ROOT)
    try:
        import main  # ~0.4 s warm (matplotlib); imported here, not at module scope,
    finally:          # so xdist collection on every worker does not pay for it.
        sys.path.remove(_ROOT)

    d = _data()["design"]
    assert (d["T0"], d["p0"], d["M0"]) == (main.FLIGHT.T0, main.FLIGHT.p0, main.FLIGHT.M0), (
        f"docs/visuals/data.json flight condition {d['T0'], d['p0'], d['M0']} != "
        f"main.py's {main.FLIGHT}. Re-run docs/visuals/extract_data.py after "
        "syncing its FLIGHT to main.py's."
    )
    assert d["pi_c"] == main.PI_C and d["Tt4"] == main.TT4, (
        f"docs/visuals/data.json design point (pi_c={d['pi_c']}, Tt4={d['Tt4']}) != "
        f"main.py's (pi_c={main.PI_C}, Tt4={main.TT4})."
    )
    assert d["losses"] == main.REAL_LOSSES, (
        f"docs/visuals/data.json losses {d['losses']} != main.py's REAL_LOSSES "
        f"{main.REAL_LOSSES}. The 'real' case on the pages is then a different "
        "engine from the one every main.py panel prints."
    )


# ------------------------------------------- 3. the cycle blocks are the model's

# Relative bar. `extract_data.py` rounds every dumped number to 6 significant
# figures (`r(x, sig=6)`), so a live-model comparison CANNOT be tighter than
# ~5e-6 no matter how exact the code is; the worst deviation measured when this
# gate was written was 3.3e-6, which is that rounding and nothing else. Do NOT
# tighten this to the measured value — the next number to round unluckily trips
# it for no reason. An ABSOLUTE bar is meaningless here: pt is ~7e5 Pa and far
# is ~0.02.
_REL = 1e-5


def _close(a, b):
    return abs(a - b) <= _REL * max(abs(b), 1e-30)


def test_data_json_cycle_blocks_match_the_live_model():
    """Recompute the ideal/real CPG cycle and compare against the committed dump.

    This is the load-bearing one: it is what makes "every number on the page is
    the model's" a checked claim rather than a promise. Cheap (two design-point
    runs), so it can live in the fast subset and run on every iteration.
    """
    sys.path.insert(0, _ROOT)
    try:
        from turbojet.engine import FlightCondition, build_turbojet
        from turbojet.gas import Gas
    finally:
        sys.path.remove(_ROOT)

    d = _data()
    dp = d["design"]
    flight = FlightCondition(T0=dp["T0"], p0=dp["p0"], M0=dp["M0"])
    gas = Gas()
    cases = (("ideal", {}), ("real", dp["losses"]))

    for name, losses in cases:
        res = build_turbojet(gas, dp["pi_c"], dp["Tt4"], flight.p0, **losses).run(flight, 1.0)
        blk = d[name]

        assert set(blk["stations"]) == set(res.stations), (
            f"data.json['{name}'] stations {sorted(blk['stations'])} != the model's "
            f"{sorted(res.stations)} — the cycle gained or lost a station; re-run "
            "docs/visuals/extract_data.py."
        )
        for label, s in res.stations.items():
            j = blk["stations"][label]
            for field, live in (("Tt", s.Tt), ("pt", s.pt), ("far", s.far)):
                dumped = j[field]
                # Every one of these is a number at every station. `far` in
                # particular: FlowState declares `far: float = 0.0`, so it is 0.0
                # upstream of the burner, never None — the cutaway's hover probe
                # showing 0 there is the PAGE's choice, not missing data. Stated
                # because extract_data.py's rounder does pass None through, so a
                # None here would be a real change and is worth failing on rather
                # than skipping past.
                assert isinstance(dumped, (int, float)) and isinstance(live, (int, float)), (
                    f"data.json['{name}'].stations['{label}'].{field}: dumped "
                    f"{dumped!r}, model {live!r} — one of them is not a number."
                )
                assert _close(live, dumped), (
                    f"data.json['{name}'].stations['{label}'].{field} = {dumped!r} but "
                    f"the model now gives {live!r} (rel "
                    f"{abs(live - dumped) / max(abs(dumped), 1e-30):.2e} > {_REL:.0e}). "
                    "The visuals are stale: re-run docs/visuals/extract_data.py, then "
                    "build.py and build_cutaway.py, then republish both artifacts."
                )

        perf = res.performance
        for field, live in (("V0", res.V0), ("V9", res.V9), ("M9", res.M9), ("T9", res.T9),
                            ("specific_thrust", perf.specific_thrust),
                            ("tsfc", perf.tsfc),
                            ("eta_brayton", perf.eta_brayton),
                            ("eta_thermal", perf.eta_thermal),
                            ("eta_propulsive", perf.eta_propulsive),
                            ("eta_overall", perf.eta_overall)):
            dumped = blk[field]
            assert _close(live, dumped), (
                f"data.json['{name}'].{field} = {dumped!r} but the model now gives "
                f"{live!r} (rel {abs(live - dumped) / max(abs(dumped), 1e-30):.2e} > "
                f"{_REL:.0e}). Re-run docs/visuals/extract_data.py, then the two build "
                "scripts, then republish both artifacts."
            )


# ------------------------------------- 4. what the page READS survives the trim

# Aliases the cutaway template binds to a data block, and the shapes it reads a
# field through:  DATA.ideal -> I,  DATA.real -> R,  DATA[CASE] -> D() and d.
_READ_RE = re.compile(r"\b(?:D\(\)|DATA|[IRd])\.([A-Za-z_][A-Za-z_0-9]*)")

# Field names the regex will find that are NOT data fields: `DATA.design` is a
# top-level block, and it is carried by the trim unconditionally.
_NOT_A_CYCLE_FIELD = {"design", "ideal", "real"}

# A field present in data.json's cycle blocks that build_cutaway.py deliberately
# DROPS. Naming it here is the point: if extract_data.py grows a new field, this
# test fails and forces a decision — carry it in KEEP, or record it here as
# deliberately dropped — instead of the field going missing unnoticed.
_DELIBERATELY_DROPPED = {"eta_brayton"}   # the cutaway's perf table shows the
                                          # three efficiencies a reader can act
                                          # on; the Brayton bound is the chart
                                          # page's story, not the cutaway's.


def _fields_the_cutaway_reads():
    found = {m for m in _READ_RE.findall(_text("cutaway-template.html"))}
    return found - _NOT_A_CYCLE_FIELD


def test_the_read_census_can_actually_see():
    """The instrument, before its verdict.

    A regex census that matches nothing passes every test built on it. These
    five names are read by the cutaway's own perf table and T-s diagram today;
    if the census cannot find them it is broken, and its silence below means
    nothing.
    """
    found = _fields_the_cutaway_reads()
    for probe in ("stations", "specific_thrust", "eta_overall", "points", "legs"):
        assert probe in found, (
            f"the template read-census found no `{probe}` — the regex no longer "
            "matches how the page reads its data, so every check built on it is "
            "vacuous. Fix _READ_RE before trusting the assertions below."
        )


def test_every_field_the_cutaway_reads_survives_the_trim():
    """No field the page reads is trimmed away by build_cutaway.py's KEEP list.

    This is the failure with no Python symptom: a dropped field is `undefined` in
    the browser and renders as `NaN`, and no other test in the repo opens these
    files.
    """
    payload = _trimmed_payload()
    for field in sorted(_fields_the_cutaway_reads()):
        for case in ("ideal", "real"):
            assert field in payload[case], (
                f"cutaway-template.html reads `{field}`, but build_cutaway.py's KEEP "
                f"list drops it from the '{case}' block — the page renders undefined/NaN "
                "there. Add it to KEEP in docs/visuals/build_cutaway.py and rebuild."
            )


def test_the_trim_drops_nothing_unnamed():
    """Every dumped cycle field is either KEPT or named as deliberately dropped.

    The complement of the test above: it catches a field ADDED to extract_data.py
    that nobody decided about, before a template edit starts reading it.
    """
    keep = set(_build_cutaway_module().KEEP)
    # The two declarations must not contradict each other. Without this, the
    # natural repair for a newly-read field (add it to KEEP) leaves it ALSO named
    # as deliberately dropped, and the difference below passes either way — a lie
    # in the source with no symptom.
    assert not (keep & _DELIBERATELY_DROPPED), (
        f"{sorted(keep & _DELIBERATELY_DROPPED)} is both KEPT by build_cutaway.py and "
        "listed in _DELIBERATELY_DROPPED here. Remove it from _DELIBERATELY_DROPPED."
    )
    d = _data()
    for case in ("ideal", "real"):
        unaccounted = set(d[case]) - keep - _DELIBERATELY_DROPPED
        assert not unaccounted, (
            f"data.json['{case}'] carries {sorted(unaccounted)}, which build_cutaway.py "
            "neither KEEPs nor is recorded as deliberately dropping. Decide: add to KEEP "
            "in docs/visuals/build_cutaway.py, or add to _DELIBERATELY_DROPPED here with "
            "the reason."
        )
        missing = keep - set(d[case])
        assert not missing, (
            f"build_cutaway.py KEEPs {sorted(missing)}, which data.json['{case}'] does not "
            "have — the trim would raise KeyError on the next build."
        )


def test_the_cutaway_renders_its_design_point_from_the_data():
    """The chips and the losses footer are RENDERED, not typed.

    They used to be literal markup — `<span class="chip"><b>π_c</b>10</span>` and a
    typed list of the seven loss factors — which meant the page could go on
    announcing a design point the model had left. `paintChrome()` now reads them
    from `DATA.design`, so this test pins that: the fields it reads must exist,
    and every loss must have a label to render under.
    """
    tpl = _text("cutaway-template.html")
    assert "function paintChrome()" in tpl and "paintChrome();" in tpl, (
        "cutaway-template.html no longer defines/calls paintChrome() — if the design "
        "point went back to being typed into the markup, it can drift from the model "
        "again with no symptom."
    )
    design = _data()["design"]
    for field in ("M0", "T0", "p0", "pi_c", "Tt4", "losses"):
        assert field in design, f"data.json['design'] has no `{field}`, which paintChrome() reads"

    labels = re.search(r"const LOSS_LABEL = \{(.*?)\};", tpl, re.S)
    assert labels, "cutaway-template.html no longer declares LOSS_LABEL — the label census cannot run."
    known = set(re.findall(r"([A-Za-z_][A-Za-z_0-9]*):", labels.group(1)))
    # Instrument self-check by COUNT, not by naming one key: probing for a
    # specific name means removing exactly that name reports "census broken"
    # instead of the real finding. (Caught by mutating this test.)
    assert len(known) >= 5, (
        f"the LOSS_LABEL census parsed only {sorted(known)} — it no longer matches how "
        "the labels are declared, so the check below is vacuous."
    )
    unlabelled = set(design["losses"]) - known
    assert not unlabelled, (
        f"data.json['design'].losses carries {sorted(unlabelled)}, which cutaway-template.html's "
        "LOSS_LABEL does not name — the footer would print the raw Python key. Add the symbol."
    )


# The design-point spellings that used to be typed into template.html's prose.
# This is a NAMED-FORM gate, not a general one: a future session could re-type the
# design point in a spelling this list does not carry. It catches the regression that
# actually happened, which is the one a splice test structurally cannot see.
_TYPED_DESIGN_POINT = (
    "M<sub>0</sub>=0.85",
    "π<sub>c</sub>=10",
    "T<sub>t4</sub>=1500",
    "η<sub>c</sub>=0.88",
    "η<sub>t</sub>=0.90",
)


def test_the_charts_page_renders_its_design_point_from_the_data():
    """The charts page states WHICH ENGINE it shows in prose — three times.

    The figure subtitle, the footer's provenance paragraph and the T-s lede all
    named `M0=0.85, pi_c=10, Tt4=1500 K` (and the lede two efficiencies) as literal
    markup. That is invisible to `test_visuals_html_is_the_current_splice`: a
    constant typed into the template lands in the built page too, so the splice
    matches perfectly while the sentence is false. `paintDesignPoint()` now fills
    three spans from `DATA.design`, and this pins that it stays that way.
    """
    tpl = _text("template.html")
    assert "function paintDesignPoint()" in tpl and "\npaintDesignPoint();" in tpl, (
        "template.html no longer defines/calls paintDesignPoint() — if the design point "
        "went back to being typed into the prose, the page can announce an engine the "
        "model has left, and the splice test will not notice."
    )
    for span in ("ts-design", "footer-design", "lede-eta"):
        assert f'id="{span}"' in tpl, (
            f"template.html has no `id={span}` for paintDesignPoint() to fill — the "
            "sentence it belongs to is either gone or back to being typed."
        )
    typed = [t for t in _TYPED_DESIGN_POINT if t in tpl]
    assert not typed, (
        f"template.html types the design point again as {typed}. Render it from "
        "DATA.design through paintDesignPoint() instead — a typed constant is copied "
        "into the built page by build.py, so no splice or data gate can see it drift."
    )
    design = _data()["design"]
    for field in ("M0", "pi_c", "Tt4", "losses"):
        assert field in design, f"data.json['design'] has no `{field}`, which paintDesignPoint() reads"
    for loss in ("eta_c", "eta_t"):
        assert loss in design["losses"], (
            f"data.json['design'].losses has no `{loss}`, which the T-s lede renders"
        )


def test_every_element_the_pages_look_up_actually_exists():
    """Every `getElementById(...)` target must be declared in the same template.

    This is the DOM half of the typed-design-point defect, and it is the wider
    class: `paintChrome()` does `document.getElementById('chips').innerHTML = ...`
    with no null guard, so deleting that div throws and kills the REST of boot —
    the page renders half-drawn while every test here stays green. The guarded
    spellings (`paintDesignPoint()` uses `if (n)`) fail silently instead, which is
    worse to notice and no better to have.

    Measured when written: 15 lookups in the cutaway, 21 in the charts page, all
    resolving. So this gate has no current failure to paper over — it exists to
    catch the next rename that touches only one side.
    """
    for name in ("cutaway-template.html", "template.html"):
        src = _text(name)
        looked_up = sorted(set(
            m.group(2) for m in re.finditer(r"getElementById\((['\"])([A-Za-z0-9_-]+)\1\)", src)
        ))
        # Instrument self-check: if the census parses nothing, the assert below is
        # vacuous. Both pages drive their whole DOM this way, so a handful is a bug
        # in this regex, not a page that stopped using ids.
        assert len(looked_up) >= 10, (
            f"the getElementById census parsed only {looked_up} out of {name} — it no "
            "longer matches how the page looks elements up, so this check is vacuous."
        )
        declared = set(re.findall(r"""\bid=["']([A-Za-z0-9_-]+)["']""", src))
        missing = [i for i in looked_up if i not in declared]
        assert not missing, (
            f"{name} looks up {missing} but declares no such id. An UNGUARDED lookup "
            "(`getElementById(x).innerHTML = ...`) throws and stops the rest of the "
            "script; a guarded one renders nothing. Either way the page is wrong and "
            "the splice gate cannot see it — the built page has the same defect."
        )


def test_the_cutaway_station_labels_all_exist():
    """The station tiles are a literal list in the template; every one must resolve."""
    tpl = _text("cutaway-template.html")
    m = re.search(r"const STN = \[(.*?)\];", tpl, re.S)
    assert m, "cutaway-template.html no longer declares `const STN = [...]` — the station-tile census cannot run."
    labels = re.findall(r"\['([0-9]+)'", m.group(1))
    assert len(labels) >= 6, f"only {len(labels)} station labels parsed out of STN — census broken"
    payload = _trimmed_payload()
    for case in ("ideal", "real"):
        for label in labels:
            s = payload[case]["stations"].get(label)
            assert s is not None, (
                f"the cutaway draws a tile for station {label}, but data.json['{case}'] has "
                f"no such station — the tile renders undefined."
            )
            for field in ("Tt", "pt", "far"):
                assert field in s, f"station {label} in '{case}' has no {field}"


# --------------------------------------- 5. the expensive blocks, shape only

# The blocks extract_data.py produces from the ~10-minute sweeps. NEVER
# recomputed here — see this file's docstring. Checked only for existence and
# for carrying finite numbers, which is what catches a truncated or half-written
# dump.
_SWEEP_BLOCKS = ("eq_design", "bell", "quench", "jsweep", "m_of_T",
                 "prompt_shape", "spatial", "ladder", "dwell")


def _finite_leaves(obj, path, out):
    if isinstance(obj, dict):
        for k, v in obj.items():
            _finite_leaves(v, f"{path}.{k}", out)
    elif isinstance(obj, list):
        assert obj, f"{path} is an empty list"
        for i, v in enumerate(obj):
            _finite_leaves(v, f"{path}[{i}]", out)
    elif isinstance(obj, (int, float)) and not isinstance(obj, bool):
        assert math.isfinite(obj), f"{path} is {obj!r} — a non-finite number reached data.json"
        out.append(path)


def test_sweep_blocks_present_and_finite():
    d = _data()
    for block in _SWEEP_BLOCKS:
        assert block in d, (
            f"data.json has no '{block}' block — the chart page's panels read it, so "
            "the dump is incomplete. Re-run docs/visuals/extract_data.py (~10 min)."
        )
        leaves = []
        _finite_leaves(d[block], block, leaves)
        assert leaves, f"data.json['{block}'] carries no numbers at all"


def test_data_json_has_no_unexpected_top_level_blocks():
    """A new block means a new panel; make that a decision, not a surprise."""
    expected = {"design", "ideal", "real"} | set(_SWEEP_BLOCKS)
    actual = set(_data())
    assert actual == expected, (
        f"data.json's top-level blocks changed: added {sorted(actual - expected)}, "
        f"removed {sorted(expected - actual)}. Update _SWEEP_BLOCKS in this file in the "
        "same commit that changes docs/visuals/extract_data.py."
    )


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok:", name)
