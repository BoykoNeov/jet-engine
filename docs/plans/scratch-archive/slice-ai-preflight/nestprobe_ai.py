"""pytest plugin: count SAME-FIELD NESTED sets of `_b_state` / `_v_state`.

Slice AC booked a `_b_state`/`_v_state` same-field nest as a live CANDIDATE at slice AH,
found by a NAME-BASED reachability graph across classes -- an UPPER bound. This is the
runtime form that booking owes: it patches the two owning classes' `__setattr__` and
records every set of the field while it is ALREADY non-None (that, and only that, is what
a `Cell<Option<f64>>` + RAII guard whose Drop restores to None would get wrong).
"""
import atexit, json, os

TALLY = {"b_sets": 0, "b_nests": 0, "v_sets": 0, "v_nests": 0,
         "b_nest_where": {}, "v_nest_where": {}}


def pytest_configure(config):
    import traceback
    from turbojet import engine as E

    def make(cls, field, skey, nkey, wkey):
        base = cls.__setattr__

        def setter(self, name, value):
            if name == field:
                TALLY[skey] += 1
                if value is not None and getattr(self, field, None) is not None:
                    TALLY[nkey] += 1
                    # PER-SITE TALLY, not a capped LIST. The first writing kept the first
                    # twelve frames; all twelve were the SAME site, so it could not say
                    # whether the other three STATIC nest sites ever run. That is slice AG
                    # step 7's own lesson (a capped/one-bit reading needs a COUNTER) applied
                    # to this probe.
                    st = [f for f in traceback.extract_stack()[:-1]
                          if "engine.py" in f.filename][-3:]
                    site = (f"{type(self).__name__} <- " +
                            " <- ".join(f"{f.name}:{f.lineno}" for f in reversed(st)))
                    TALLY[wkey][site] = TALLY[wkey].get(site, 0) + 1
            base(self, name, value)
        cls.__setattr__ = setter

    make(E.LimitedBleedTransient, "_b_state", "b_sets", "b_nests", "b_nest_where")
    make(E.ThreeLoopCascadeTransient, "_v_state", "v_sets", "v_nests", "v_nest_where")

    # ONE FILE PER PROCESS. Under `pytest.ini`s `-n auto` the tests run in WORKER
    # subprocesses and the CONTROLLER sees nothing; a single shared path lets the
    # controllers empty tally clobber theirs at exit, which is how the first run of
    # this probe reported 0 sets and 0 nests -- a BLIND instrument, not a zero.
    # POSITIVE CONTROL, run BEFORE any test: prove the patched `__setattr__` is installed
    # and that the nest predicate FIRES. Without this a `0 nests` reading is indistinguishable
    # from a probe that was never wired in -- which is exactly what run 1 produced.
    probe = object.__new__(E.ThreeLoopCascadeTransient)
    object.__setattr__(probe, "_b_state", None)
    object.__setattr__(probe, "_v_state", None)
    probe._b_state = 1.0          # None -> 1.0 : a set, NOT a nest
    probe._b_state = 2.0          # 1.0  -> 2.0 : a NEST
    probe._v_state = 3.0
    probe._v_state = 4.0
    TALLY["control"] = (f"b_sets={TALLY['b_sets']} b_nests={TALLY['b_nests']} "
                        f"v_sets={TALLY['v_sets']} v_nests={TALLY['v_nests']}")
    assert TALLY["b_sets"] == 2 and TALLY["b_nests"] == 1, TALLY
    assert TALLY["v_sets"] == 2 and TALLY["v_nests"] == 1, TALLY
    for k in ("b_sets", "b_nests", "v_sets", "v_nests"):
        TALLY[k] = 0
    TALLY["b_nest_where"].clear(); TALLY["v_nest_where"].clear()

    base_out = os.environ.get("NEST_OUT", r"W:/temp/claude/slice-ai-preflight/nest")
    out = "%s.%d.json" % (base_out, os.getpid())

    def dump():
        TALLY["pid"] = os.getpid()
        open(out, "w", encoding="utf-8").write(json.dumps(TALLY, indent=1))
    atexit.register(dump)


# --- AI ADDITION: which `_with_coord` BODY runs, on which CLASS, from which SITE -----------------
def _wrap_with_coord():
    import traceback
    from turbojet import engine as E
    TALLY["with_coord"] = {}
    for cls in (E.DemandCoordinateTransient, E.StateCoordinateTransient):
        body = cls.__dict__["_with_coord"]

        def make(body, owner):
            def w(self, *a, **kw):
                st = [f for f in traceback.extract_stack()[:-1] if "engine.py" in f.filename][-1:]
                site = f"{type(self).__name__} body={owner} from " + \
                       ",".join(f"{f.name}:{f.lineno}" for f in st)
                TALLY["with_coord"][site] = TALLY["with_coord"].get(site, 0) + 1
                return body(self, *a, **kw)
            return w
        setattr(cls, "_with_coord", make(body, cls.__name__))


_orig_configure = pytest_configure


def pytest_configure(config):          # noqa: F811 -- chain the AH plugin's configure
    _orig_configure(config)
    _wrap_with_coord()
