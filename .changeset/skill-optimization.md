---
"gws": patch
---

feat(skills): implement hierarchical skill discovery and search command

- Restructure skills/ into hierarchical references/ subdirectory to avoid agent context pollution.
- Add `gws skills search <query>` command for semantic/keyword discovery of 40+ API services.
- Restore essential safety tips (zsh ! expansion, JSON quoting) in the gws-shared skill.
- Refactor generate-skills logic to automate artifact generation and link validation.
