---
name: GitHub push authentication
description: The authentication form that successfully pushes this workspace to its GitHub origin.
---

Use Git's HTTP Basic authorization header with the username `x-access-token` and the workspace's GitHub personal access token when pushing to the configured GitHub origin. A Bearer extraheader may authenticate the API but still be rejected by the Git remote.

**Why:** In this workspace, the same valid token returned an API success response but GitHub rejected the Bearer-form push; the Basic-form push succeeded.

**How to apply:** Keep the token out of remotes, files, logs, and user-facing output. Supply the header only for the push process.