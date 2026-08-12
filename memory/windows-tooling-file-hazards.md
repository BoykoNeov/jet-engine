---
name: windows-tooling-file-hazards
description: "PyPy leaves open().write() unflushed, and PowerShell Get/Set-Content silently destroys UTF-8 source files"
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

**Why:** both corrupt files while reporting success, and this project's deliverable is prose —
20,000+ lines of derivation comments full of `∫`, `§`, `Δ`, `φ`, `≈`. A mangling that survives
a green build is exactly the kind of damage that gets committed.

**How to apply:** use the **Edit/Write tools** for source files, never PowerShell text
round-trips. Reserve PowerShell for running commands. If a bulk edit really needs scripting,
operate on bytes, or verify afterwards by grepping for `âˆ|Â§|Ã|ï»¿`. Related:
[[rust-port-decided]], [[pypy-switch-shipped]].
