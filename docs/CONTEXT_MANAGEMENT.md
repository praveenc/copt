# MANAGING CONTEXT

This guide covers context management strategies for Kiro IDE to keep the main agent's context window clean and focused while handling complex, multi-faceted tasks.

---

## SUBAGENTS

Subagents run with **isolated context windows**, keeping the main agent context clean. Use them liberally for any task that would consume significant context.

### Built-in Subagents

| Subagent | Purpose | Use When |
|----------|---------|----------|
| **Context Gatherer** | Exploring projects, reading files | Investigating unfamiliar codebases |
| **General-Purpose** | Parallelizing any task | Running multiple operations simultaneously |

### Custom Subagents (v0.9+)

Define specialized agents in `~/.kiro/agents/` with custom system prompts and model selection:

```yaml
# ~/.kiro/agents/code-reviewer.yaml
name: code-reviewer
description: Security-focused code review specialist
model: claude-opus-4-6
system_prompt: |
  You are a security-focused code reviewer. Analyze code for:
  - OWASP Top 10 vulnerabilities
  - Input validation issues
  - Authentication/authorization flaws
```

### When to Use Subagents

- **Parallel investigation**: Analyze multiple files, issues, or data sources simultaneously
- **Context-heavy operations**: Git history analysis, large file reads, codebase exploration
- **Specialized tasks**: Delegate to domain-specific custom agents
- **Extending limits**: Bypass context window constraints without triggering summarization

Reference: https://kiro.dev/changelog/ide/0-8/

---

## AUTOMATIC SUMMARIZATION (v0.7+)

Kiro automatically summarizes conversations when context usage reaches **80%** of the window limit. This preserves important information while freeing space.

### Working with Summarization

- **Proactive offloading**: Use subagents for exploratory work *before* hitting 80%
- **Critical context**: Keep essential information (requirements, constraints) in the main thread
- **Checkpoints**: Create checkpoints (v0.6+) before major context shifts to enable rollback

---

## STEERING FILES & POWERS

### Steering Files (v0.9+)

Steering files auto-include when their descriptions match the current request. Place project-specific guidance in:

- `CLAUDE.md` / `AGENTS.md` - Project-level instructions
- `~/.kiro/steering/` - Global steering rules across workspaces

### Powers (v0.7+)

Powers bundle MCP servers and steering files for **dynamic, context-aware loading**:

```yaml
# ~/.kiro/powers/aws-dev.yaml
name: aws-dev
description: AWS development context
mcp_servers:
  - aws-documentation-mcp-server
steering_files:
  - ~/.kiro/steering/aws-patterns.md
```

Activate with slash commands: `/power aws-dev`

---

## HOOKS

Hooks intercept agent actions to inject context, block operations, or provide guardrails.

### Hook Triggers (v0.8+)

| Trigger | Fires When | Use For |
|---------|------------|---------|
| `prompt_submit` | User submits a prompt | Pre-processing, context injection |
| `agent_stop` | Agent completes | Post-processing, cleanup |
| `pre_tool_use` | Before any tool call | Blocking risky operations, adding context |
| `post_tool_use` | After any tool call | Logging, validation |

### Example: Git Safety Hook

```yaml
# ~/.kiro/hooks/git-safety.yaml
trigger: pre_tool_use
tool: Bash
match: "git push|git reset --hard"
action: block
message: "Blocked: Destructive git operation. Use explicit confirmation."
```

---

## TARGETED FILE CONTEXT (v0.5+)

Use line ranges to include only relevant portions of files:

```
#file:src/analyzer/mod.rs:150-200
#file:src/llm/bedrock.rs:1-50
```

This drastically reduces context consumption for large files.

---

## CHECKPOINTING (v0.6+)

Create conversation checkpoints before:
- Major refactoring discussions
- Switching between unrelated tasks
- Experimental approaches that might fail

Revert to checkpoints to restore previous context state without losing work.

---

## SLASH COMMANDS (v0.7+)

Quick access to steering and hooks:

| Command | Action |
|---------|--------|
| `/power <name>` | Activate a power bundle |
| `/hook <name>` | Trigger a hook manually |
| `/steer <file>` | Load a steering file |

---

## CONTEXT MANAGEMENT TIPS

### Do

- Use Makefile targets instead of raw commands (reduces token count)
- Delegate exploratory work to subagents
- Use targeted file context with line ranges
- Create checkpoints before context-heavy operations
- Define custom subagents for repeated specialized tasks

### Avoid

- Reading entire large files when only sections are needed
- Running multiple exploratory searches in the main thread
- Letting context hit 80% before offloading to subagents

### Project-Specific

- Use `@container-git` subagent for git operations inside the docker-workspace container
- Use `@rust-dev` subagent for cargo builds, clippy, and test execution
- Offload codebase exploration to the Context Gatherer subagent
- Use Powers to load AWS documentation MCP server only when needed

---

## CONFIGURED .kiro/ INVENTORY

### Global (`~/.kiro/`)

| Type | Name | Purpose |
|------|------|---------|
| Agent | `rust-dev` | Rust builds, clippy, tests — isolates cargo output from main context |
| Agent | `container-git` | Docker-workspace git operations (haiku model for cost efficiency) |
| Steering | `git-commits.md` | Git safety, conventional commits (manual inclusion) |
| Steering | `rust-cargo.md` | Global Rust/Cargo conventions (manual inclusion) |

### Project-level (`.kiro/`)

| Type | Name | Auto-Include Trigger |
|------|------|---------------------|
| Steering | `copt-rules.md` | Rule taxonomy, analyzer, EXP/STY/TUL/FMT patterns |
| Steering | `tui-architecture.md` | TUI module, MVU pattern, ratatui, widgets |
| Steering | `rust-idioms.md` | .rs file edits, Rust patterns |
| Hook | `container-git-guard.md` | Pre-tool: warns on bare git commands |

---

## REFERENCES

- [v0.9 - Custom Subagents, Skills, Hooks](https://kiro.dev/changelog/ide/0-9/)
- [v0.8 - Subagents, Web Tools](https://kiro.dev/changelog/ide/0-8/)
- [v0.7 - Powers, Summarization, Slash Commands](https://kiro.dev/changelog/ide/0-7/)
- [v0.6 - Checkpointing, Multi-root Workspaces](https://kiro.dev/changelog/ide/0-6/)
- [v0.5 - AGENTS.md, Targeted File Context](https://kiro.dev/changelog/ide/0-5/)
