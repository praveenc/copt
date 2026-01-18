---
allowed-tools: Bash(cargo:*), Bash(git:*), Bash(gh:*), Bash(docker compose:*), Bash(docker exec:*), Bash(docker ps:*), Read, Edit, Write, AskUserQuestion
argument-hint: [major|minor|patch|version]
description: Create a new release with changelog, tag, and GitHub release
---

You are executing the release workflow for the copt project.

## Container Requirement

**CRITICAL**: All git operations (commit, tag, push) and GitHub CLI commands MUST run inside the git-workspace container.

### Container Setup
1. **Check if running**: `docker ps --filter name=my-git-workspace --format '{{.Names}}'`
2. **Start only if not running**: `docker compose run -d --rm --name my-git-workspace git-workspace`

### Verify Git Identity
After container is running, verify git config (must run from repo directory):
```bash
docker exec -w /workspace/repos/copt my-git-workspace git-test
```
Expected: username `praveenc`, email `1090396+praveenc@users.noreply.github.com`

**Note**: Git user config is set per-repo, not globally. Running `git-test` from `/workspace` will show "not set".

If incorrect, stop and alert user.

### Running Commands in Container
```bash
docker exec my-git-workspace git -C /workspace/repos/copt <command>
docker exec my-git-workspace gh <command>
```

Repo path inside container: `/workspace/repos/copt`

## Current State
- Cargo.toml version: !`grep '^version' Cargo.toml | head -1`
- Latest git tag: !`git describe --tags --abbrev=0 2>/dev/null || echo "no tags"`
- Git status: !`git status --porcelain`

## Release Workflow

Execute these steps IN ORDER. Stop and report errors immediately.

### Step 0: Container Setup
1. Check if running: `docker ps --filter name=my-git-workspace --format '{{.Names}}'`
2. If empty, start: `docker compose run -d --rm --name my-git-workspace git-workspace`
3. Verify git identity: `docker exec -w /workspace/repos/copt my-git-workspace git-test`
4. Confirm username is `praveenc` and email is `1090396+praveenc@users.noreply.github.com`
5. If verification fails, STOP and alert user

### Step 1: Pre-flight Validation
Run all validation commands. ALL must pass before proceeding:
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. `cargo build --release`

If any fail, report the error and stop.

### Step 2: Determine Version
The user requested: $ARGUMENTS

If empty or ambiguous, ask the user which version bump they want:
- major (breaking changes)
- minor (new features)
- patch (bug fixes)
- Or an explicit version number

Calculate the new version from current Cargo.toml version.

### Step 3: Generate Changelog
1. Get commits since last tag: `git -P log $(git describe --tags --abbrev=0)..HEAD --oneline`
2. Categorize commits by conventional commit type:
   - `feat:` -> Added
   - `fix:` -> Fixed
   - `docs:` -> Documentation (only if significant)
   - `refactor:` -> Changed
   - `perf:` -> Changed (performance)
   - `chore:` -> Skip unless significant
3. Generate a changelog entry in Keep a Changelog format
4. Show the draft to user and ask for approval/edits

### Step 4: Update Files
1. Update version in `Cargo.toml`
2. Add new version section to `CHANGELOG.md`:
   - Insert after `## [Unreleased]` line
   - Add `## [X.Y.Z] - YYYY-MM-DD` header
   - Add categorized changes
3. Update comparison links at bottom of CHANGELOG.md

### Step 5: Commit and Tag (in container)
Run these commands inside the container:
1. Stage changes: `docker exec my-git-workspace git -C /workspace/repos/copt add Cargo.toml CHANGELOG.md`
2. Commit: `docker exec my-git-workspace git -C /workspace/repos/copt commit -m "chore: release v{version}"`
3. Create tag: `docker exec my-git-workspace git -C /workspace/repos/copt tag v{version}`

### Step 6: Push and Create Release (in container)
Run these commands inside the container:
1. Push with tags: `docker exec my-git-workspace git -C /workspace/repos/copt push origin main --tags`
2. Create GitHub release:
   ```bash
   docker exec -w /workspace/repos/copt my-git-workspace gh release create v{version} \
     --title "copt v{version}" \
     --notes "$(extract changelog section for this version)"
   ```

### Step 7: Report Success
Display summary:
- Version released
- Changelog entries
- GitHub release URL
- Next steps (if any)

## Important Rules
- **All git/gh commands must run via `docker exec my-git-workspace`**
- Use `-P` flag for git commands that paginate (log, diff, show)
- Never use `git push --force`
- Never use `git commit --amend`
- Stop immediately on any error
- Always show what you're about to do before destructive operations
- Verify container is running before any git operations
