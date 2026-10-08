# Re-run docs/visuals/extract_data.py WITHOUT touching the repo: exec its source with the
# output path redirected here. Prints the interpreter so the log says which one ran.
import sys
print(sys.version, flush=True)
SRC = r"W:\Claude_projects\jet engine\docs\visuals\extract_data.py"
OUT = r"W:\temp\claude\phase8-ak\data_pypy.json"
code = open(SRC, encoding="utf-8").read()
needle = 'with open(HERE / "data.json", "w") as fh:'
assert code.count(needle) == 1
code = code.replace(needle, f'with open(r"{OUT}", "w") as fh:')
sys.argv = [SRC]
g = {"__name__": "__main__", "__file__": SRC}
exec(compile(code, SRC, "exec"), g)
