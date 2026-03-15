---
"@googleworkspace/cli": patch
---

Fix critical security vulnerability (TOCTOU/Symlink race) in atomic file writes.

The atomic_write and atomic_write_async utilities now use:
- Randomized temporary filenames to prevent predictability.
- O_EXCL creation flags to prevent following pre-existing symlinks.
- Strict 0600 permissions from the moment of file creation on Unix systems.
- Redundant post-write permission calls have been removed to close race windows.
