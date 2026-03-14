---
name: gws-modelarmor-sanitize-response
version: 1.0.0
description: "Google Model Armor: Sanitize a model response through a Model Armor template."
metadata:
  openclaw:
    category: "security"
    requires:
      bins: ["gws"]
    cliHelp: "gws modelarmor +sanitize-response --help"
---

# modelarmor +sanitize-response

Sanitize a model response through a Model Armor template

## Usage

```bash
gws modelarmor +sanitize-response --template <NAME>
```

## Flags

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--template` | ✓ | — | Full template resource name (projects/PROJECT/locations/LOCATION/templates/TEMPLATE) |
| `--text` | — | — | Text content to sanitize |
| `--json` | — | — | Full JSON request body (overrides --text) |

## Examples

```bash
gws modelarmor +sanitize-response --template projects/P/locations/L/templates/T --text 'model output'
model_cmd | gws modelarmor +sanitize-response --template ...
```

## Tips

- Use for outbound safety (model -> user).
- For inbound safety (user -> model), use +sanitize-prompt.

