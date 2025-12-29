# 📋 Optimization Rules Reference

> Complete documentation of all optimization rules in `copt`

---

## Table of Contents

- [Overview](#overview)
- [Rule Categories](#rule-categories)
- [Explicitness Rules (EXP)](#explicitness-rules-exp)
- [Style Rules (STY)](#style-rules-sty)
- [Tool Usage Rules (TUL)](#tool-usage-rules-tul)
- [Formatting Rules (FMT)](#formatting-rules-fmt)
- [Verbosity Rules (VRB)](#verbosity-rules-vrb)
- [Agentic Coding Rules (AGT)](#agentic-coding-rules-agt)
- [Long-Horizon Task Rules (LHT)](#long-horizon-task-rules-lht)
- [Frontend Design Rules (FED)](#frontend-design-rules-fed)
- [Severity Levels](#severity-levels)
- [Customization](#customization)

---

## Overview

The `copt` uses a rule-based engine to detect common anti-patterns in prompts and suggest improvements aligned with Claude 4.5 best practices.

### How Rules Work

1. **Detection**: Each rule has a pattern matcher that identifies specific issues
2. **Analysis**: Detected issues are categorized and scored by severity
3. **Transformation**: Static transformations are applied where possible
4. **Enhancement**: LLM-powered rewriting improves complex cases

### Rule Naming Convention

```
[CATEGORY][NUMBER] - Rule Name
    │        │
    │        └── Sequential number (001, 002, etc.)
    └────────── Category prefix (3 letters)
```

---

## Rule Categories

| Category        | Prefix | Description                  | Rule Count |
| --------------- | ------ | ---------------------------- | ---------- |
| Explicitness    | EXP    | Clear, specific instructions | 4          |
| Style           | STY    | Instruction tone and wording | 4          |
| Tool Usage      | TUL    | Tool and action directives   | 3          |
| Formatting      | FMT    | Output format specifications | 3          |
| Verbosity       | VRB    | Response length and detail   | 2          |
| Agentic Coding  | AGT    | Code exploration and safety  | 4          |
| Long-Horizon    | LHT    | Multi-step task management   | 3          |
| Frontend Design | FED    | UI/UX aesthetic guidance     | 2          |

**Total: 25 rules**

---

## Explicitness Rules (EXP)

These rules ensure prompts are specific and actionable.

### EXP001 — Vague Instructions

**Severity**: Warning

**Description**: Detects overly general or vague instructions that don't provide enough specificity for Claude 4.5's precise instruction following.

**Detection Patterns**:

- Short imperative phrases without detail (< 10 words)
- Generic verbs without objects: "Create", "Build", "Make", "Write"
- Missing success criteria or completion indicators

**Examples**:

❌ **Before**:

```
Create a dashboard
```

✅ **After**:

```
Create an analytics dashboard. Include as many relevant features and
interactions as possible. Go beyond the basics to create a fully-featured
implementation with charts, filters, and real-time data visualization.
```

---

### EXP002 — Missing Context/Motivation

**Severity**: Info

**Description**: Instructions that would benefit from explaining "why" to help Claude understand the underlying intent.

**Detection Patterns**:

- Bare prohibitions without reasoning
- Instructions that could apply to multiple scenarios
- Rules without explanation of consequences

**Examples**:

❌ **Before**:

```
Always use snake_case for variable names
```

✅ **After**:

```
Always use snake_case for variable names to maintain consistency with our
Python codebase and follow PEP 8 conventions. This makes it easier for
the team to read and maintain the code.
```

---

### EXP003 — Indirect Commands

**Severity**: Warning

**Description**: Detects polite but indirect phrasing that may cause Claude to suggest rather than act.

**Detection Patterns**:

- "Can you..." / "Could you..."
- "Would you mind..."
- "Is it possible to..."
- "I was wondering if..."

**Examples**:

❌ **Before**:

```
Can you refactor this function to be more efficient?
```

✅ **After**:

```
Refactor this function to improve its performance. Implement the changes
directly using the available tools.
```

---

### EXP004 — Missing Success Criteria

**Severity**: Info

**Description**: Complex tasks that lack clear definition of what constitutes success.

**Detection Patterns**:

- Multi-step instructions without checkpoints
- Open-ended tasks without boundaries
- Research tasks without scope limits

**Examples**:

❌ **Before**:

```
Research the best practices for API design
```

✅ **After**:

```
Research REST API design best practices. Focus on:
1. URL naming conventions
2. HTTP method usage
3. Error response formats
4. Pagination strategies

Summarize your findings with 3-5 key recommendations for each area.
```

---

## Style Rules (STY)

These rules improve the tone and effectiveness of instructions.

### STY001 — Negative Instructions

**Severity**: Warning

**Description**: Instructions framed as prohibitions ("Don't...", "Never...") which can be less effective than positive guidance.

**Detection Patterns**:

- "Don't..." / "Do not..."
- "Never..."
- "Avoid..."
- "Stop..."
- Negations without alternatives

**Examples**:

❌ **Before**:

```
Don't use markdown in your response
```

✅ **After**:

```
Write your response in smoothly flowing prose paragraphs without any
special formatting. Use complete sentences that connect naturally.
```

---

### STY002 — Aggressive Emphasis

**Severity**: Info

**Description**: Claude 4.5 is more responsive than previous models; aggressive emphasis may cause overtriggering.

**Detection Patterns**:

- ALL CAPS text (except acronyms)
- Multiple exclamation marks
- "CRITICAL:", "IMPORTANT:", "MUST", "ALWAYS" in caps
- Excessive asterisks/bold markers

**Examples**:

❌ **Before**:

```
CRITICAL: You MUST ALWAYS check for null values!!! This is EXTREMELY
IMPORTANT and should NEVER be forgotten!
```

✅ **After**:

```
Check for null values before processing. This validation step is important
for preventing runtime errors.
```

---

### STY003 — Sensitive Word "Think"

**Severity**: Warning (when extended thinking is disabled)

**Description**: Claude Opus 4.5 without extended thinking is sensitive to the word "think" and its variants.

**Detection Patterns**:

- "think about"
- "think through"
- "I think"
- "thinking"

**Suggested Replacements**:
| Original | Replacement |
|----------|-------------|
| think about | consider |
| think through | work through |
| I think | I believe |
| thinking | considering / evaluating |

**Examples**:

❌ **Before**:

```
Think about the edge cases before implementing
```

✅ **After**:

```
Consider the edge cases before implementing
```

---

### STY004 — Over-Triggering Language

**Severity**: Info

**Description**: Language designed to force action in Claude 3.x may overtrigger in Claude 4.5.

**Detection Patterns**:

- Multiple emphatic triggers in one prompt
- Redundant urgency markers
- Stacked requirements with different phrasings

**Examples**:

❌ **Before**:

```
CRITICAL: You MUST use this tool when searching. This is MANDATORY and
REQUIRED. Always remember to NEVER skip this step. It's ESSENTIAL.
```

✅ **After**:

```
Use this tool when searching for files in the codebase.
```

---

## Tool Usage Rules (TUL)

These rules ensure proper tool invocation and action directives.

### TUL001 — Suggestion Without Action

**Severity**: Warning

**Description**: Phrases that ask for suggestions when the intent is to make changes.

**Detection Patterns**:

- "suggest changes"
- "recommend improvements"
- "what do you think about"
- "how would you improve"

**Examples**:

❌ **Before**:

```
Can you suggest some changes to improve this function?
```

✅ **After**:

```
Improve this function's performance. Implement the changes directly using
the edit tool.
```

---

### TUL002 — Missing Parallel Tool Guidance

**Severity**: Info

**Description**: Tasks involving multiple independent operations that could benefit from parallel execution guidance.

**Detection Patterns**:

- Multiple file operations mentioned
- Batch processing tasks
- Independent API calls
- No parallel/sequential specification

**Examples**:

❌ **Before**:

```
Read the config files in the settings directory
```

✅ **After**:

```
Read all config files in the settings directory. If there are multiple
files that can be read independently, read them in parallel to improve
efficiency. Only sequence operations that have dependencies.
```

---

### TUL003 — Missing Cleanup Instructions

**Severity**: Info

**Description**: Tasks that may create temporary files without cleanup guidance.

**Detection Patterns**:

- Testing/iteration workflows
- Script generation tasks
- Temporary file mentions
- No cleanup specification

**Examples**:

❌ **Before**:

```
Create some test scripts to verify the changes work correctly
```

✅ **After**:

```
Create test scripts to verify the changes work correctly. After testing
is complete, clean up any temporary scripts or helper files that were
created during the process.
```

---

## Formatting Rules (FMT)

These rules help control output formatting.

### FMT001 — Missing Format Specification

**Severity**: Info

**Description**: Prompts that would benefit from explicit format guidance.

**Detection Patterns**:

- Tasks with implicit format assumptions
- Mixed content types without structure
- Long-form content without organization hints

**Examples**:

❌ **Before**:

```
Explain how authentication works in this system
```

✅ **After**:

```
Explain how authentication works in this system. Structure your
explanation with clear headings for each component (e.g., ## Token
Generation, ## Validation Flow). Use code blocks for any example code.
```

---

### FMT002 — Negative Format Instructions

**Severity**: Warning

**Description**: Format restrictions phrased negatively rather than as positive guidance.

**Detection Patterns**:

- "No markdown"
- "Don't use bullet points"
- "Avoid code blocks"
- Format prohibitions without alternatives

**Examples**:

❌ **Before**:

```
Don't use any markdown formatting in your response
```

✅ **After**:

```
Write your response in flowing prose paragraphs. Incorporate information
naturally into sentences rather than fragmenting it into lists.
```

---

### FMT003 — Missing XML Structure Suggestion

**Severity**: Info

**Description**: Complex prompts that could benefit from XML tag organization.

**Detection Patterns**:

- Multiple distinct sections
- Conditional instructions
- Rules with examples
- Input/output pairs

**Examples**:

❌ **Before**:

```
Here are the rules: always be concise, use formal tone.
Here's an example of good output: "The analysis reveals..."
Here's the input to process: [data]
```

✅ **After**:

```
<rules>
- Always be concise
- Use formal tone
</rules>

<example>
Good output: "The analysis reveals..."
</example>

<input>
[data]
</input>
```

---

## Verbosity Rules (VRB)

These rules manage response detail and length.

### VRB001 — Missing Verbosity Guidance

**Severity**: Info

**Description**: Complex tasks without guidance on expected response length or detail level.

**Detection Patterns**:

- Multi-step tasks without progress expectations
- Tool-using tasks without summary requests
- Ambiguous scope on explanations

**Examples**:

❌ **Before**:

```
Refactor the authentication module
```

✅ **After**:

```
Refactor the authentication module. After completing the changes, provide
a brief summary of what was modified and why.
```

---

### VRB002 — Missing Progress Reporting

**Severity**: Info

**Description**: Long-running tasks that would benefit from progress updates.

**Detection Patterns**:

- Multi-file operations
- Iterative processes
- Testing workflows
- No visibility instructions

**Examples**:

❌ **Before**:

```
Update all API endpoints to use the new response format
```

✅ **After**:

```
Update all API endpoints to use the new response format. After completing
each endpoint, provide a quick summary of the changes made before moving
to the next one.
```

---

## Agentic Coding Rules (AGT)

These rules improve code exploration and reduce errors.

### AGT001 — Missing Exploration Directive

**Severity**: Warning

**Description**: Code modification tasks without instruction to read code first.

**Detection Patterns**:

- "Fix the bug in..."
- "Update the function..."
- "Change the implementation..."
- No read/explore instruction

**Examples**:

❌ **Before**:

```
Fix the bug in the user authentication flow
```

✅ **After**:

```
Fix the bug in the user authentication flow. First, read and understand
the relevant files before proposing changes. Inspect the current
implementation to understand the existing patterns and conventions.
```

---

### AGT002 — Missing Hallucination Prevention

**Severity**: Warning

**Description**: Tasks that risk speculation about code without verification.

**Detection Patterns**:

- Questions about code behavior
- Requests for explanations without file references
- Assumptions about implementation details

**Examples**:

❌ **Before**:

```
Why is the login failing for admin users?
```

✅ **After**:

```
Investigate why login is failing for admin users. Read the relevant
authentication and user role files before answering. Do not speculate
about code you haven't inspected.
```

---

### AGT003 — Missing State Management Guidance

**Severity**: Info

**Description**: Complex tasks that would benefit from structured state tracking.

**Detection Patterns**:

- Multi-step implementations
- Test-driven workflows
- Iterative development tasks
- No state persistence mention

**Examples**:

❌ **Before**:

```
Implement the full CRUD API for the products resource
```

✅ **After**:

```
Implement the full CRUD API for the products resource. Track your progress
in a structured format (e.g., a progress.txt file) noting which endpoints
are complete and any remaining work. Use git commits to checkpoint your
progress.
```

---

### AGT004 — Missing Anti-Overengineering Directive

**Severity**: Info

**Description**: Tasks where Claude might add unnecessary complexity.

**Detection Patterns**:

- Open-ended implementation requests
- "Build a system for..."
- Feature additions without scope limits
- No simplicity constraints

**Examples**:

❌ **Before**:

```
Build a caching system for the API responses
```

✅ **After**:

```
Build a simple caching system for the API responses. Avoid over-engineering:
only implement what's directly needed without adding extra features,
abstractions, or configurability beyond the current requirements.
```

---

## Long-Horizon Task Rules (LHT)

These rules improve multi-step and extended task management.

### LHT001 — Missing State Persistence

**Severity**: Warning

**Description**: Long tasks that may span context windows without persistence guidance.

**Detection Patterns**:

- Complex multi-phase projects
- Research tasks
- Large refactoring efforts
- No persistence strategy

**Examples**:

❌ **Before**:

```
Refactor the entire backend to use async/await
```

✅ **After**:

```
Refactor the entire backend to use async/await. This is a large task, so:
1. Start by creating a todo list of files to modify
2. Track progress in a progress.txt file
3. Commit changes incrementally with descriptive messages
4. If context runs low, save your state before continuing
```

---

### LHT002 — Missing Incremental Progress Emphasis

**Severity**: Info

**Description**: Large tasks without guidance to work incrementally.

**Detection Patterns**:

- "Complete" or "full" implementation requests
- Large scope without breakdown
- No iteration strategy

**Examples**:

❌ **Before**:

```
Implement all the missing features in the dashboard
```

✅ **After**:

```
Implement the missing features in the dashboard. Work on one feature at a
time, ensuring each is complete and tested before moving to the next.
Focus on incremental progress rather than attempting everything at once.
```

---

### LHT003 — Missing Context Window Awareness

**Severity**: Info

**Description**: Extended tasks without context budget management.

**Detection Patterns**:

- Very long task descriptions
- Open-ended explorations
- No stopping criteria
- Research without limits

**Examples**:

❌ **Before**:

```
Explore the codebase and document everything you find
```

✅ **After**:

```
Explore the codebase and document the main components. Work systematically
and efficiently use your context window. If you approach the context limit,
summarize your findings and save your state before the window refreshes.
```

---

## Frontend Design Rules (FED)

These rules improve UI/UX output quality.

### FED001 — Generic UI Request

**Severity**: Info

**Description**: UI creation requests that may result in "AI slop" aesthetics.

**Detection Patterns**:

- "Create a UI for..."
- "Build a form..."
- "Design a page..."
- No aesthetic guidance

**Examples**:

❌ **Before**:

```
Create a login page
```

✅ **After**:

```
Create a login page with distinctive, creative design. Avoid generic "AI
slop" aesthetics:
- Choose unique typography (not Inter, Roboto, or system fonts)
- Use a bold, cohesive color palette with sharp accents
- Add thoughtful micro-interactions and animations
- Create atmosphere with layered backgrounds or subtle patterns
```

---

### FED002 — Missing Design Specificity

**Severity**: Info

**Description**: Frontend tasks without typography, color, or motion guidance.

**Detection Patterns**:

- Dashboard/UI requests without style guide
- Component creation without design context
- No mention of animations or interactions

**Examples**:

❌ **Before**:

```
Build a data visualization component
```

✅ **After**:

```
Build a data visualization component with polished design:
- Typography: Use a distinctive, modern font family
- Colors: Apply a cohesive palette with clear data encoding
- Motion: Add smooth transitions when data updates
- Interactions: Include hover states and tooltips for data points
```

---

## Severity Levels

| Level       | Icon | Description                                 | Action     |
| ----------- | ---- | ------------------------------------------- | ---------- |
| **Error**   | 🔴   | Critical issue likely to cause poor results | Must fix   |
| **Warning** | 🟡   | Issue that may degrade output quality       | Should fix |
| **Info**    | 🔵   | Suggestion for improvement                  | Optional   |

### Severity Distribution

```
Error:   0 rules  (0%)
Warning: 11 rules (44%)
Info:    14 rules (56%)
```

---

## Customization

### Disabling Specific Rules

```bash
# Disable specific rules
c45-optimize -f prompt.txt --disable-rules EXP002,STY003

# Disable entire categories
c45-optimize -f prompt.txt --disable-categories FED,VRB
```

### Configuration File

```toml
# ~/.c45-optimizer.toml

[rules]
# Disable specific rules
disabled = ["EXP002", "STY003"]

# Disable categories
disabled_categories = ["FED"]

# Adjust severity (promote info to warning)
[rules.severity_overrides]
AGT004 = "warning"
FMT003 = "warning"
```

### Custom Rules (Future)

Custom rules can be defined in TOML format:

```toml
# custom-rules.toml

[[rule]]
id = "CUSTOM001"
name = "Company-specific terminology"
category = "style"
severity = "warning"
pattern = "(?i)\\buser\\b"
suggestion = "Replace 'user' with 'customer' per company style guide"
replacement = "customer"
```

---

## Contributing New Rules

1. Identify the anti-pattern from Claude 4.5 best practices
2. Determine the category and appropriate severity
3. Write detection patterns (regex or semantic)
4. Create before/after examples
5. Implement transformation logic
6. Add unit tests
7. Update this documentation

See [CONTRIBUTING.md](CONTRIBUTING.md) for full guidelines.

---

_Document Version: 1.0.0_
_Rules Version: 25 rules across 8 categories_
