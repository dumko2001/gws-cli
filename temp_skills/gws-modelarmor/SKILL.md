---
name: gws-modelarmor
version: 1.0.0
description: "Google Model Armor: Filter user-generated content for safety."
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["gws"]
    cliHelp: "gws modelarmor --help"
---

# modelarmor (v1)

```bash
gws modelarmor <resource> <method> [flags]
```

## Helper Commands

| Command | Description |
|---------|-------------|
| [`+sanitize-prompt`](../gws-modelarmor-sanitize-prompt/SKILL.md) | Sanitize a user prompt through a Model Armor template |
| [`+sanitize-response`](../gws-modelarmor-sanitize-response/SKILL.md) | Sanitize a model response through a Model Armor template |
| [`+create-template`](../gws-modelarmor-create-template/SKILL.md) | Create a new Model Armor template |

## Reference

Use `gws modelarmor --help` to list resources, and `gws schema modelarmor.<resource>.<method>` to inspect parameters.

