---
name: gws-shared
version: 1.0.0
description: "gws CLI: Shared patterns for authentication, global flags, and output formatting."
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["gws"]
---

# gws — Shared Reference

## Installation
The `gws` binary must be on `$PATH`.

## Authentication
```bash
gws auth login  # Interactive OAuth
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/key.json  # Service Account
```

## Global Flags
| Flag | Description |
|------|-------------|
| `--format <json|table|yaml|csv>` | Output format |
| `--dry-run` | Local validation only |
| `--sanitize <TPL>` | Screen through Model Armor |

## CLI Syntax
`gws <service> <resource> [sub-resource] <method> [flags]`

### Method Flags
| Flag | Description |
|------|-------------|
| `--params '{"k": "v"}'` | URL/query parameters |
| `--json '{"k": "v"}'` | Request body |
| `-o, --output <PATH>` | Save binary response |
| `--upload <PATH>` | Upload file |
| `--page-all` | NDJSON pagination |

## Security Rules
- Confirm write/delete commands with user.
- Use `--dry-run` for destructive operations.
- Use `--sanitize` for PII safety.

## Shell Tips
- **zsh `!`:** Use double quotes for sheet ranges: `"Sheet1!A1"`.
- **JSON:** Wrap `--params` and `--json` in single quotes: `'{"key": "val"}'`.
