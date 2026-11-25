---
name: cutaway-artifact
description: "The old cutaway artifact (claude.ai/artifact/KbCcwpVMiJWJm1sN7kYdZm) is now a POINTER page to the sandbox's Engine view (2026-11-25); the stand-alone cutaway page is deleted from the repo"
metadata:
  node_type: memory
  type: reference
  originSessionId: 19af6582-8bb4-4a85-824c-d17edc6db008
  modified: 2026-11-25T09:11:01.974Z
---

**Since 2026-11-25 (roadmap A9, [[a9-engine-view]]):** the stand-alone cutaway page
(`docs/visuals/turbojet-cutaway.html` + `cutaway-template.html`) is deleted; the animated cutaway is the sandbox's
**Engine** view (`docs/sandbox/`). At the user's choice its published link,
https://claude.ai/artifact/KbCcwpVMiJWJm1sN7kYdZm (title "Turbojet Cutaway"), was republished as a short pointer
page: where the cutaway moved, how to open it, what is now computed. Its source is not in the repo (a one-off
page); if it needs changing, read the artifact and republish to that URL. Nothing rebuilds it.

The old page lives in git history (before the A9 commit) and its publish history on the artifact.

Lessons from building the old page that still apply to the Engine view:
- Chrome's MCP tools refuse `file://` URLs; serve the folder with a local server (record the PID, stop by PID — the
  global kill-by-PID rule).
- Rotor "spin" in a side cutaway is a projection, not a scroll: blade at angle θ spans y = r·cosθ, visible only on
  the far half (sinθ<0), shear from the stagger — this reads as real rotation with blades bunching at the rims.
