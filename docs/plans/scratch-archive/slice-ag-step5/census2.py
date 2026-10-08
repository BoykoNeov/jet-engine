import io

p = r"M:\claud_projects\jet engine\tests\test_rust_line_citations.py"
s = io.open(p, encoding="utf-8", newline="").read()
old = "# RE-BLESSED AT SLICE AG STEP 5 (10/59/47 -> 10/63/51):"
assert s.count(old) == 1
new = (
"# RE-BLESSED AGAIN IN STEP 5's OWN ADDENDUM COMMIT (10/63/51 -> 10/70/56): five more, every one\n"
"# a SIGNATURE line for a reader default that no caller in the repository spells (`19403` is\n"
"# `cap_gains`'s `refs`/`laws`, `19497`/`19500` `cap_bill`'s `tau_t`/`tail`, `19550`/`19552`\n"
"# `solve_gain`'s `ref` and its `dq`/`every`). Step 4 named two constants because the CRATE relied\n"
"# on them; these are named because step 6's ported gates are the only call sites and the suite\n"
"# they are transcribed from does not write the numbers down either.\n"
"#\n"
"# RE-BLESSED AT SLICE AG STEP 5 (10/59/47 -> 10/63/51):")
s = s.replace(old, new)
s = s.replace("FILES = 10\nSITES = 63\nLINES = 51", "FILES = 10\nSITES = 70\nLINES = 56")
assert "SITES = 70" in s
io.open(p, "w", encoding="utf-8", newline="").write(s)
print("census updated")
