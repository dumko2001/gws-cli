---
name: gws-meet
version: 1.0.0
description: "Manage Google Meet conferences."
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["gws"]
    cliHelp: "gws meet --help"
---

# meet (v2)

```bash
gws meet <resource> <method> [flags]
```

## API Resources

### conferenceRecords

  - `get` — Gets a conference record by conference ID.
  - `list` — Lists the conference records. By default, ordered by start time and in descending order.
  - `participants` — Operations on the 'participants' resource
  - `recordings` — Operations on the 'recordings' resource
  - `transcripts` — Operations on the 'transcripts' resource

### spaces

  - `create` — Creates a space.
  - `endActiveConference` — Ends an active conference (if there's one). For an example, see [End active conference](https://developers.google.com/workspace/meet/api/guides/meeting-spaces#end-active-conference).
  - `get` — Gets details about a meeting space. For an example, see [Get a meeting space](https://developers.google.com/workspace/meet/api/guides/meeting-spaces#get-meeting-space).
  - `patch` — Updates details about a meeting space. For an example, see [Update a meeting space](https://developers.google.com/workspace/meet/api/guides/meeting-spaces#update-meeting-space).

## Reference

Use `gws meet --help` to list resources, and `gws schema meet.<resource>.<method>` to inspect parameters.

