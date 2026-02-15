# MANAGING CONTEXT

The Kiro IDE that you are working on allows subagents

## SUBAGENTS

Introducing subagents for parallel task execution. Kiro can now run multiple tasks simultaneously or delegate to specialized subagents. Two built-in subagents are available: a context gatherer for exploring projects and a general-purpose agent for parallelizing tasks. Each subagent has its own context window, keeping the main agent context clean. Use subagents to investigate multiple data sources in parallel, analyze GitHub issues across repositories, or extend your context window limits without requiring summarization.
Reference: https://kiro.dev/changelog/ide/0-8/

## TIPS FOR MANAGING CONTEXT

- Use Makefile whenever possible.
- Use subagents for parallel tasks
- Use subagent for git-commits inside the container
