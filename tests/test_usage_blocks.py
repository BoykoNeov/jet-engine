"""Guard: every call written in a class's `Usage:` block must BIND against the real signature.

A `Usage:` block is the one kind of documentation in this project that is
executable in principle and executed by nothing. It decays with exactly the
signature it documents, and no gate anywhere notices — so it decays silently,
and a reader who copies the documented call gets a `TypeError` or an
`AttributeError` from a line the repo presents as the worked example.

WHAT THE DECAY LOOKED LIKE WHEN IT WAS FINALLY MEASURED (rung 74's port,
slice AF steps 5(a) and 6):

  * Rung 72's block called `t.shared_modes(...)`, a method with ZERO definitions
    anywhere in `engine.py`. Two more classes had the same defect (rung 65's
    `restored_plant`, rung 66's `cascade_modes`) — every one a reader RENAMED
    while its own class docstring kept the old name.
  * Rung 74's block passed `sm=` to three readers that take `phi_lim=`; rung 63's
    three lever sweeps were documented WITHOUT the lever they exist to sweep;
    rung 65's `bandwidth_ceiling` without its `phi_lim`.
  * Six statements were not Python AT ALL — five wrote a builder call in the
    shape `build_turbojet(gas, pi_c=10, Tt4=1500, p0, ...)`, which is a
    positional argument after a keyword one (and names a parameter, `p0`, that
    the builder does not have: it is `p_ambient`), and one wrote `nu0=(.., ..)`,
    where `..` is not an expression. Those six were the WORSE half and were
    invisible to the first census, because a block that does not parse gets
    skipped whole.

THE LESSON THAT MADE THE LAST ONE FINDABLE, and the one to apply when this test
fails: **look for one construct spelled two ways in the same file.** Both
not-Python spellings had a correct twin a few hundred lines away — rung 40's
block writes `build_two_spool_turbojet(gas, 3, 6, 1500, p0, **losses, ...)`
positionally and rung 34's writes `nu0=..., s_end=..., ds=...` with real
Ellipsis. The right spelling was never in doubt; nothing had ever compared them.

IF THIS TEST FAILS:
  * a BIND FAIL means a documented call cannot run. Fix the DOCSTRING, not the
    signature — unless the docstring is right and the code drifted, which is a
    different and more interesting bug. For a method that no longer exists,
    decide explicitly: RETARGET the line to the renamed reader (check the spec's
    own reader list, which is where all three renames were settled), or DELETE
    the line, and say which in the commit.
  * a NEW SKIP means a documented call became uncheckable — a `**kwargs` in the
    written call, or a line that is not Python. That is not a lesser failure than
    a bind fail; it is how the six worst entries hid. The frozen list below is
    the whole of what this project has agreed to leave uncheckable.
  * a COUNT change means blocks or calls were added or removed. Re-measure by
    running this file directly (it prints the census) and type the new numbers
    from that run. Do not transcribe them from a plan or a commit message.

WHAT THIS GUARD DOES NOT DO: it binds, it does not RUN. A call that binds can
still raise inside the body, and a value written in a block is never checked
against anything. Binding is what decays silently; a wrong value is a different
guard's job and there isn't one.
"""
import ast
import inspect
import os
import sys
import textwrap
import types

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import turbojet.engine as ENGINE      # noqa: E402


# --- the census, MEASURED (run this file directly; it prints all four) --------------------
# Taken from the run in the commit that fixed the last seven bind fails and the six
# not-Python statements — read off THAT run, on the fixed tree, never transcribed.
# `CALLS` counts every call this checker can resolve to a class, a method or a
# module-level function of `turbojet.engine`; calls to anything else (`dict`, a
# name the block never binds) are not counted and not checked.
CLASSES = 58
BLOCKS = 34
CALLS = 166

# The frozen uncheckable set: (class the block belongs to, what was written, why).
# A `**kwargs` in the WRITTEN call makes a static bind meaningless — the missing
# arguments may be in the dict. Nothing else is allowed to be uncheckable, and
# growing this list is a decision, not a fix.
SKIPS = frozenset({
    ("CombustorTransient", "build_turbojet", "star-args"),
    ("MapMatcher", "build_turbojet", "star-args"),
    ("OffDesignMatcher", "build_turbojet", "star-args"),
    ("SpoolTransient", "build_turbojet", "star-args"),
    ("StaircaseLawTransient", "StaircaseLawTransient.edge_read", "star-args"),
    ("StaircaseLawTransient", "StaircaseLawTransient.staircase_scan", "star-args"),
    ("StaircaseLawTransient", "StaircaseLawTransient.lattice_count", "star-args"),
    ("StaircaseLawTransient", "StaircaseLawTransient.root_class", "star-args"),
    ("TwoSpoolFuelTransient", "build_two_spool_turbojet", "star-args"),
    ("TwoSpoolMapMatcher", "build_two_spool_turbojet", "star-args"),
    ("TwoSpoolMatcher", "build_two_spool_turbojet", "star-args"),
    ("TwoSpoolTransient", "build_two_spool_turbojet", "star-args"),
})


# --- the instrument ----------------------------------------------------------------------

def _usage_block(doc):
    """The lines under a `Usage:` line, to the first blank line or dedent."""
    if not doc:
        return None
    lines = doc.splitlines()
    for i, line in enumerate(lines):
        if line.strip() != "Usage:":
            continue
        base = len(line) - len(line.lstrip())
        body = []
        for nxt in lines[i + 1:]:
            if not nxt.strip():
                if body:
                    break
                continue
            if len(nxt) - len(nxt.lstrip()) <= base:
                break
            body.append(nxt)
        return "\n".join(body)
    return None


def _statements(block):
    """Group a block's lines into logical statements, PER STATEMENT.

    A line that does not parse is joined with the ones after it until it does (a
    wrapped call). If NO extension of it ever parses, that ONE line is yielded
    with `None` and the scan resumes at the next one — so an unparseable line
    costs one line and not the whole block. That is the difference that found six
    of the sixteen original defects: the first census parsed each block whole, so
    a block with one untypable line took its other calls down with it.
    """
    lines = [ln for ln in textwrap.dedent(block).splitlines() if ln.strip()]
    out, i = [], 0
    while i < len(lines):
        for j in range(i, len(lines)):
            src = textwrap.dedent("\n".join(lines[i:j + 1]))
            try:
                tree = ast.parse(src)
            except SyntaxError:
                continue
            out.append((src, tree))
            i = j + 1
            break
        else:
            out.append((lines[i].strip(), None))
            i += 1
    return out


def _implicit_self(owner, name, receiver_is_class):
    """How many positional arguments the CALLER does not write.

    A plain method reached through an instance gets `self` for free; reached
    through the class it does not, because the caller writes it. A `staticmethod`
    and a `classmethod` never get one.
    """
    static = inspect.getattr_static(owner, name, None)
    if isinstance(static, (staticmethod, classmethod)):
        return 0
    return 0 if receiver_is_class else 1


def census(module):
    """Bind every documented call in `module`'s class `Usage:` blocks.

    Returns `(classes, blocks, calls, skips, oks, fails)`; `skips` and `fails`
    are lists of `(class name, what was written, why)`.
    """
    classes = sorted(
        (c for _, c in inspect.getmembers(module, inspect.isclass)
         if c.__module__ == module.__name__),
        key=lambda c: c.__name__)
    n_blocks = n_calls = n_ok = 0
    skips, fails = [], []

    for cls in classes:
        block = _usage_block(cls.__doc__)
        if block is None:
            continue
        n_blocks += 1
        env = {}                        # local name -> the class it is an instance of
        for src, tree in _statements(block):
            if tree is None:
                n_calls += 1
                fails.append((cls.__name__, src.splitlines()[0].strip(), "NOT PYTHON"))
                continue
            for stmt in tree.body:
                if (isinstance(stmt, ast.Assign) and len(stmt.targets) == 1
                        and isinstance(stmt.targets[0], ast.Name)
                        and isinstance(stmt.value, ast.Call)):
                    func = stmt.value.func
                    root = func.value if isinstance(func, ast.Attribute) else func
                    made = getattr(module, root.id, None) if isinstance(root, ast.Name) else None
                    if isinstance(made, type):
                        env[stmt.targets[0].id] = made
            for node in ast.walk(tree):
                if not isinstance(node, ast.Call):
                    continue
                func = node.func
                if isinstance(func, ast.Name):
                    obj = getattr(module, func.id, None)
                    if isinstance(obj, type):
                        written, target, implicit = func.id, obj.__init__, 1
                    elif inspect.isfunction(obj):
                        written, target, implicit = func.id, obj, 0
                    else:
                        continue
                elif isinstance(func, ast.Attribute) and isinstance(func.value, ast.Name):
                    recv = func.value.id
                    named = getattr(module, recv, None)
                    receiver_is_class = isinstance(named, type)
                    owner = named if receiver_is_class else env.get(recv)
                    if owner is None:
                        continue
                    written = f"{owner.__name__}.{func.attr}"
                    target = getattr(owner, func.attr, None)
                    if target is None:
                        n_calls += 1
                        fails.append((cls.__name__, written, "NO SUCH METHOD"))
                        continue
                    implicit = _implicit_self(owner, func.attr, receiver_is_class)
                else:
                    continue
                n_calls += 1
                if (any(isinstance(a, ast.Starred) for a in node.args)
                        or any(k.arg is None for k in node.keywords)):
                    skips.append((cls.__name__, written, "star-args"))
                    continue
                try:
                    sig = inspect.signature(target)
                except (TypeError, ValueError) as exc:     # pragma: no cover - defensive
                    skips.append((cls.__name__, written, f"no signature: {exc}"))
                    continue
                try:
                    sig.bind(*([object()] * (implicit + len(node.args))),
                             **{k.arg: object() for k in node.keywords})
                except TypeError as exc:
                    fails.append((cls.__name__, written, str(exc)))
                else:
                    n_ok += 1

    return len(classes), n_blocks, n_calls, skips, n_ok, fails


# --- the gates ---------------------------------------------------------------------------

def test_every_documented_call_binds():
    _, _, _, _, _, fails = census(ENGINE)
    assert not fails, (
        "a shipped `Usage:` block documents a call that cannot run:\n  "
        + "\n  ".join(f"{c}: {w} -- {why}" for c, w, why in fails)
        + "\nFix the DOCSTRING (see this file's module docstring); for a method that no "
          "longer exists, retarget it to the renamed reader or delete the line, explicitly.")


def test_the_uncheckable_set_is_frozen():
    """A NEW skip is a failure. This is how the six worst entries originally hid."""
    _, _, _, skips, _, _ = census(ENGINE)
    got = frozenset(skips)
    assert got == SKIPS, (
        f"the uncheckable set moved.\n  added:   {sorted(got - SKIPS)}\n"
        f"  removed: {sorted(SKIPS - got)}\n"
        "A call this checker cannot bind is not a lesser failure than one that fails to "
        "bind — it is an unchecked line. Remove the `**kwargs` from the written call, or "
        "add it here deliberately.")


def test_the_census_is_the_size_it_was_measured():
    """A guard that only asserts `0 fails` passes when the parser finds nothing at all."""
    n_classes, n_blocks, n_calls, _, _, _ = census(ENGINE)
    assert (n_classes, n_blocks, n_calls) == (CLASSES, BLOCKS, CALLS), (
        f"census moved: {n_classes} classes / {n_blocks} blocks / {n_calls} calls, "
        f"expected {CLASSES} / {BLOCKS} / {CALLS}. If that is real growth, run this file "
        "directly and type the numbers off ITS OWN run.")


def _synthetic():
    """A module carrying every defect this guard exists to catch, and one clean call."""
    mod = types.ModuleType("synthetic_usage")

    class Plant:
        """A plant.

        Usage:
            t = Plant(1.0)
            t.reader(1.0, phi_lim=0.8)       # binds
            t.reader(1.0, sm=0.8)            # an argument the reader does not take
            t.renamed(1.0)                   # a method that does not exist
            t.reader(1.0, x=..., 2.0)        # not Python at all
            t.reader(1.0, **kw)              # uncheckable, on purpose
        """

        def __init__(self, scale):
            self.scale = scale

        def reader(self, x, phi_lim=0.8):
            return x

    mod.Plant = Plant
    Plant.__module__ = mod.__name__
    return mod


def test_the_instrument_can_see_a_break():
    """A guard reporting zero that has never been shown to report one is not a guard."""
    _, blocks, calls, skips, oks, fails = census(_synthetic())
    kinds = sorted(why if why in ("NOT PYTHON", "NO SUCH METHOD") else "BAD ARGUMENTS"
                   for _, _, why in fails)
    assert blocks == 1 and calls == 6 and oks == 2, (blocks, calls, oks)
    assert [w for _, w, _ in skips] == ["Plant.reader"], skips
    assert kinds == ["BAD ARGUMENTS", "NO SUCH METHOD", "NOT PYTHON"], kinds
    assert any("'sm'" in why for _, _, why in fails), fails


if __name__ == "__main__":
    n_classes, n_blocks, n_calls, skips, n_ok, fails = census(ENGINE)
    print(f"classes in engine.py    : {n_classes}")
    print(f"with a `Usage:` block   : {n_blocks}")
    print(f"calls written in them   : {n_calls}")
    print(f"uncheckable (star-args) : {len(skips)}")
    print(f"bind OK                 : {n_ok}")
    print(f"BIND FAILS              : {len(fails)}")
    for cls, written, why in skips:
        print(f"  SKIP {cls:28s} {written:44s} {why}")
    for cls, written, why in fails:
        print(f"  FAIL {cls:28s} {written:44s} {why}")
