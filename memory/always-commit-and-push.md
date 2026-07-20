---
name: always-commit-and-push
description: "User wants work committed and pushed to main automatically, without asking"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c26fd4b-a96a-48b1-b4d0-d6638ba4a998
---

When work reaches a green, complete state, commit it and push to `main`
without waiting for an explicit "commit" request.

**Why:** The user stated "always commit and push" — they don't want to be
asked each time; the default should be to persist finished work.

**How to apply:** After a coherent unit of work passes its checks (tests green,
build clean), stage everything, write a descriptive commit, and push to origin
main. Still respect the [[session-end-routine]] (also refresh memory + docs at
session end) and [[git-remote-setup]] (origin = github.com/BoykoNeov/jet-engine
over SSH).
