---
name: sandbox-build-path-embedded
description: "FIXED 2026-11-24 (roadmap A10): the sandbox page now builds byte-identically from any folder — a path remap cleaned the strings but NOT the code layout; the cure was workspace membership"
metadata:
  node_type: memory
  type: feedback
  originSessionId: fb9bd4c6-cd0f-4126-8ce1-588b676faa23
  modified: 2026-11-24T19:10:13.105Z
---

Until 2026-11-24 the sandbox page's wasm held 42 absolute source paths (panic locations), so a page built in a
worktree or re-clone failed `check.ps1`'s freshness check. **Fixed (roadmap A10):** `rust/sandbox-wasm` is a
member (NOT a default member) of a workspace rooted at `rust/Cargo.toml`; it builds into `rust\target\`;
`check.ps1` builds through `build.ps1`; `tests/sandbox_page.rs` fails on any drive-letter path in the page.

**The lesson:** `--remap-path-prefix` removed every path STRING, and a grep said "clean" — yet a second-folder
build still differed in ~49 000 bytes of code. Cargo hashes a path dependency that lies OUTSIDE the build's
workspace by its ABSOLUTE folder (inside: relative), and the crate hash moves the code layout. Only a
byte-for-byte comparison of two builds from two folders showed it; a two-crate throwaway probe confirmed the
cause in seconds. (`trim-paths` is not stable in cargo 1.100; stripping symbols did not help either.)

**How to apply:** to prove "same from any folder", build in a fresh `git worktree` (apply the change with
`git diff HEAD --binary | git -C <wt> apply --index`) and compare hashes — and check both files EXIST
(`Get-FileHash` on a missing path silently compares False). The old `rust\sandbox-wasm\target\` is dead output.
