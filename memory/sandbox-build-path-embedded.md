---
name: sandbox-build-path-embedded
description: "The sandbox page's wasm embeds the build folder's absolute path — a page built in a worktree fails the real repo's freshness check"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 765d0165-d7cd-430a-aa2f-7daab84dc6af
  modified: 2026-11-20T10:00:01.297Z
---

`rust\sandbox-wasm\build.ps1` compiles the model with panic-location strings holding the ABSOLUTE source path
(42 occurrences of `W:\Claude_projects\jet engine…` in the blob, measured 2026-11-20). Built in a git worktree
under `W:\temp\claude\…`, the blob differs, so `check.ps1`'s `the_committed_page_is_current` fails in the real
repo even with identical source.

**Why:** found 2026-11-20 when a template-only change had to be built from a clean worktree because uncommitted
rung-110 `src/` changes sat on `main`.

**How to apply:** for a TEMPLATE-only change while `src/` holds uncommitted work: build in a worktree to get the
fresh page, then apply the same HTML lines to the committed page (keep its blob), and prove equality with the
fresh build after swapping the blob back (a small node script). Run check.mjs/browser.mjs in the worktree —
only the freshness check may fail, and only for the path. A `src/` change cannot use this trick.
