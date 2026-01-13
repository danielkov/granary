---
name: granary-orchestrate
description: Orchestrate sub-agents and coordinate multi-agent workflows with granary. Use when delegating tasks, spawning workers, or managing parallel execution.
---

# Orchestrating Implementation with Granary

This skill is for **implementation-time orchestration**. Your job is to coordinate sub-agents to implement planned work. You do NOT plan or design—you delegate and track.

## Primary Use-Case

You are prompted to implement a project. Follow these steps:

## Step 1: Get Granary Overview

```bash
granary summary
```

This shows all projects, their status, and task counts. Understand what exists before proceeding.

## Step 2: Find Your Target Project

If given a project name, find it:

```bash
granary projects
granary project <project-id>
```

## Step 3: Assess Project Readiness

Examine the project structure:

```bash
granary project <project-id> tasks
```

**Decision tree:**

| Situation                                    | Action                                       |
| -------------------------------------------- | -------------------------------------------- |
| Project has no tasks                         | Use `/granary:plan-work` skill to plan first |
| Project has tasks but they lack detail       | Use `/granary:plan-work` skill to refine     |
| Single simple project with clear description | Implement directly as one task               |
| Project has tasks with dependencies          | **Happy path** - proceed to Step 4           |

## Step 4: Start Orchestration Session

```bash
granary session start "implementing-<project-name>" --mode execute
granary session add <project-id>
```

## Step 5: The Orchestration Loop

Your sole focus: **hand off tasks to sub-agents**. You do NOT implement tasks yourself.

```bash
# Get next actionable task (respects dependencies)
granary next --json
```

If no task is returned, all tasks are complete.

### For Each Task:

1. **Prepare the handoff context**

   ```bash
   granary handoff --to "Implementation Agent" --tasks <task-id>
   ```

2. **Spawn a sub-agent**
   Use Claude Code's Task tool to spawn a sub-agent with:

   - The handoff context
   - Instruction to use `/granary:execute-task` skill
   - The task ID

3. **Wait for sub-agent completion**
   The sub-agent is responsible for:

   - Starting the task (`granary task <id> start`)
   - Doing the actual implementation
   - Marking the task done (`granary task <id> done`) or blocked

4. **Repeat** - get next task with `granary next --json`

### Example Sub-Agent Spawn

```
Use Task tool with:
  prompt: "Execute granary task {TASK_ID}. Use /granary:execute-task skill."
  subagent_type: "general-purpose"
```

## Step 6: Handle Completion

When all tasks are done:

```bash
granary session close --summary "Completed implementation of <project-name>"
```

## Handling Edge Cases

### Sub-Agent Gets Blocked

If a sub-agent cannot complete:

```bash
# Sub-agent should have blocked the task:
granary task <task-id> block --reason "..."

# You see it when checking next task
granary next --json  # Will skip blocked tasks
```

### Parallel Execution

For independent tasks, spawn multiple sub-agents:

```bash
# Export session for sub-agents
eval $(granary session env)

# Spawn multiple sub-agents with their own task IDs
# They all share the same session via GRANARY_SESSION env var
```

### Conflict Prevention

Sub-agents should claim tasks with leases:

```bash
granary task <task-id> claim --owner "Worker-1" --lease 30
# Exit code 4 = conflict (task claimed by another)
```

## Checkpointing

Before risky operations:

```bash
granary checkpoint create "before-major-change"

# If things go wrong
granary checkpoint restore before-major-change
```

## Summary

1. **Get overview** → `granary summary`
2. **Find project** → `granary projects`
3. **Assess readiness** → Are tasks planned? If not, use plan-work skill
4. **Start session** → `granary session start`
5. **Loop: hand off tasks** → Spawn sub-agents, do NOT implement yourself
6. **Close session** → `granary session close`

Your job is **coordination**, not implementation. Sub-agents do the actual work.
