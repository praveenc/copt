---
trigger: pre_tool_use
tools:
  - execute_bash
match_pattern: "^\\s*git\\s+(commit|push|tag|reset|rebase|filter-branch)"
action: warn
---

# Container Git Guard

**Warning**: Bare git commands detected. This project requires all git operations to run inside the docker-workspace container.

## Required Pattern

```bash
# All git commands must use:
docker exec my-git-workspace git -C /workspace/repos/copt <command>

# All gh commands must use:
docker exec my-git-workspace gh <command>
```

## Examples

```bash
# WRONG - bare git
git commit -m "feat: add feature"
git tag -a v0.3.0 -m "Release"

# CORRECT - container git
docker exec my-git-workspace git -C /workspace/repos/copt commit -m "feat: add feature"
docker exec my-git-workspace git -C /workspace/repos/copt tag -a v0.3.0 -m "Release"
```

## Setup

Verify container is running first:
```bash
docker ps --filter name=my-git-workspace --format '{{.Names}}'
```

If not running:
```bash
docker compose run -d --rm --name my-git-workspace git-workspace
```
