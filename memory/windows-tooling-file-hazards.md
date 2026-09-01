---
name: windows-tooling-file-hazards
description: "Six silent file-tooling hazards on this box: PyPy unflushed writes, PowerShell UTF-8 double-encoding, backticks in a -m message, a status read off the runner, a log still being written, and a text-mode rewrite that flips every line ending behind git's normalisation"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 454e5108-5b41-4abd-b607-eac9932757b5
  modified: 2026-08-12T06:22:47.728Z
---

Two file-writing hazards hit in one session on this box, both silent.

**1. PyPy does not refcount.** `open(p, "w").write(s)` leaves the handle unflushed, so the file
is **truncated at a buffer boundary** — a small file lands at 0 bytes while a large one looks
plausible. CPython closes it immediately, so the same script is correct there and broken under
the repo venv. Always `with open(p, "w") as fh:`.

**2. PowerShell 5.1 `Get-Content -Raw` + `Set-Content -Encoding utf8` destroys UTF-8.** It reads
as the system ANSI codepage and writes back as UTF-8, so every non-ASCII character is
double-encoded (`∫` → `âˆ«`, `§` → `Â§`) and a BOM is prepended. The build still succeeds, so
nothing fails — the damage is only visible by reading the file. Recovery is a byte round-trip:
strip `﻿`, `UTF8.GetString` the bytes, re-encode with codepage 1252, write raw bytes.

**3. BACKTICKS IN A DOUBLE-QUOTED `git commit -m` ARE COMMAND SUBSTITUTION.** Writing
`-m "the algebra is sound (\`with_vsv\` sets only \`vsv\`)"` makes bash run `with_vsv` as a
command and splice its (empty) output in, so the message ships with **every backtick-quoted
identifier silently deleted** — `bash: with_vsv: command not found` scrolls past on stderr while
the commit succeeds. This project's messages are dense with `code_names`, so the loss is
invariably load-bearing. **Always `git commit -F -` with a quoted heredoc** (`<<'EOF'`), which is
what the long messages already use — the hazard is only in the short `-m` path, which is exactly
where it feels safe to skip the heredoc.

**And the same class, twice, in the same session:** `cargo test | tail -45` reports **tail's**
exit status, and `cmd; echo "X=$?" >> log` makes the *echo* the last command, so the harness
reports the echo's success. **A status read off the runner is not the command's status — write it
into the artefact and read it back.**

**5. AND A LOG THAT IS STILL BEING WRITTEN IS A VALID PREFIX OF A GOOD ONE.** On 2026-09-01 the
full Rust gate's log held 125 `test result: ok` blocks summing to 1 335 passed, 0 failed — both
numbers plausible, both within a couple of targets of the real answer, and **no line anywhere
saying it was incomplete**. The run had not finished; twelve minutes later it exited 0 at
**137 blocks / 1 393 passed**. Read at the wrong minute the gate row would have said *down two
targets and thirty tests*, which reads as a regression and sends you chasing nothing. A truncated
`cargo test` or `pytest` log carries no error text, because every line in it is true. **Never sum a
log you did not watch exit.** The check is structural, not a sum: a sum over result blocks cannot
detect a *missing* result block, so count the lines that ANNOUNCE a target (`     Running `, plus
`Doc-tests`) and require them to equal the result blocks — and take the exit status from the
process object, not from a tail.

**6. AND A PYTHON TEXT-MODE REWRITE FLIPS EVERY LINE ENDING, WHICH `git diff` THEN HIDES.**
`io.open(p, encoding=...).read()` collapses CRLF to LF and `io.open(p, "w", encoding=...).write(s)`
translates every LF back to `os.linesep`, so on Windows a file that was LF comes back **CRLF in
full**. With git's `text=auto` the endings are normalised on read, so `git diff`, the diffstat and
`git show` all report only the lines you meant to change — on 2026-09-01 a **3-line** diff had
rewritten **1 569** line endings and nothing in the review path could see it. It surfaced only in a
gate that reads raw source bytes at compile time (`include_str!`) and scopes its search with a
newline-brace-newline pattern: that separator does not exist in a CRLF file, so the scope ran to
the end of the file and the gate fired. **Read AND write with `newline=""`**, and have any
script that restores a file assert the restore is **byte for byte**, not merely that the file is
back. When a gate over raw source fails on a step that added three lines, suspect the ENCODING
before the lines.

**Why:** all of these corrupt output while reporting success, and this project's deliverable is prose —
20,000+ lines of derivation comments full of `∫`, `§`, `Δ`, `φ`, `≈`. A mangling that survives
a green build is exactly the kind of damage that gets committed.

**How to apply:** use the **Edit/Write tools** for source files, never PowerShell text
round-trips. Reserve PowerShell for running commands. If a bulk edit really needs scripting,
operate on bytes, or verify afterwards by grepping for `âˆ|Â§|Ã|ï»¿`. Related:
[[rust-port-decided]], [[pypy-switch-shipped]].
