---
name: git-remote-setup
description: Where this repo lives on GitHub and which branch the session-end routine pushes
metadata: 
  node_type: memory
  type: project
  originSessionId: d6ed48ca-d67d-48f6-b698-b30b3ba4c4b9
---

The repo is hosted at **https://github.com/BoykoNeov/jet-engine** —
**public**, default branch **`main`**, remote **`origin`** over SSH
(`git@github.com:BoykoNeov/jet-engine.git`). The local branch was renamed
`master` → `main` on 2026-06-22 when the remote was created.

**Why:** the [[session-end-routine]] commits and pushes at every batch
boundary; it needs a settled target. Originally local-only (no remote, branch
`master`); the user chose to create the GitHub repo and then make it public.

**How to apply:** push `main` to `origin` as step 4 of the wrap-up routine.
`gh` is authenticated as BoykoNeov with `repo` scope. The repo is public, so
treat anything committed as published — don't commit secrets or unshareable
notes.
