import pathlib
M = pathlib.Path(r"C:\Users\boiko\.claude\projects\W--Claude-projects-jet-engine\memory")
s = M/"rust-port-status.md"; t = s.read_text(encoding="utf-8")
old = "* **PHASE 7** authorised 2026-08-20, **COMPLETE 2026-10-01**"
assert t.count(old) == 1
t = t.replace(old, "* **PHASE 8 AUTHORISED 2026-10-01** (\"start phase 8\"). **PRE-FLIGHT DONE same day** (plan § 8.1): row's 2–3 sessions re-sized to **11–14**, `main.py` (7 031 lines, 85 panels, 30.8 min under PyPy) is 7–8 of them; slices AK…AU proposed; the CLI scope (byte-exact port of every panel vs a slimmer CLI ≈ 4–5 sessions) is put to the USER before any slice starts. The delete is the last, separately-gated step.\n" + old, 1)
s.write_text(t, encoding="utf-8")
i = M/"MEMORY.md"; u = i.read_text(encoding="utf-8")
old2 = "phase 8 needs authorisation, its sonic_throat blocker CLOSED 2026-10-01)"
assert u.count(old2) == 1
u = u.replace(old2, "phase 8 AUTHORISED 2026-10-01, pre-flight § 8.1 done, CLI scope awaiting the user)")
i.write_text(u, encoding="utf-8")
h = M/"windows-tooling-file-hazards.md"; w = h.read_text(encoding="utf-8")
w = w.rstrip("\n") + "\n\n**A backslash in a Bash-tool heredoc or `python -c` reaches Python as an ESCAPE.** Phase 8's pre-flight (2026-10-01) wrote `W:\temp\...` into the plan through a heredoc'd Python script and got `W:<TAB>emp` — and THREE repair attempts failed the same way, because the repair's own `\"W:\\temp\"` literal was collapsed too, replacing the tab with a tab while printing `fixed 1`. A pre-existing instance (plan line 22909, slice AG) had the same origin. **Fix: write the script with the Write tool, or build the bytes as `bytes([92])`; and check with `grep -c $'\t'`, not with the script's own count.**\n"
h.write_text(w, encoding="utf-8")
print("ok")
