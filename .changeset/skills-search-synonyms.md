---
"@googleworkspace/cli": minor
---

feat(skills): add synonym expansion to `gws skills search`

Searching for "email" now finds Gmail, "spreadsheet" finds Sheets,
"schedule" finds Calendar, and so on. A static synonym table maps
30+ common natural-language terms to their canonical service names,
expanding each query token before matching so agents don't need to
know the exact API names to discover the right skill.
