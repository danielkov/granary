---
name: granary-orchestrate
description: Orchestrate sub-agents and coordinate multi-agent workflows with granary. Use when delegating tasks, spawning workers, or managing parallel execution.
---

# Orchestrating Sub-Agents with Granary

Use this skill when you need to delegate tasks to sub-agents or coordinate parallel work.

## 1. Start an Orchestration Session

```bash
granary session start "feature-impl" --owner "Orchestrator" --mode execute
granary session add <project-id>
```

## 2. The Orchestrator Loop

The core pattern for processing tasks:

```bash
# Get next actionable task (respects dependencies)
granary next --json

# Returns the highest priority task with all dependencies satisfied
# Returns null when no more tasks
```

### Basic Loop

```bash
while TASK=$(granary next --json) && [ "$(echo $TASK | jq -r '.task')" != "null" ]; do
  TASK_ID=$(echo $TASK | jq -r '.task.id')

  # Start the task
  granary task $TASK_ID start --owner "Orchestrator"

  # Get context for sub-agent
  CONTEXT=$(granary context --format prompt)

  # Spawn sub-agent with task context
  # ... your agent spawning logic here ...

  # When sub-agent completes, mark done
  granary task $TASK_ID done
done
```

## 3. Preparing Context for Sub-Agents

### Quick Summary

```bash
granary summary
```

### Detailed Context Pack

```bash
# Full context for LLM consumption
granary context --format prompt

# With specific includes
granary context --include tasks,decisions,blockers --format prompt
```

### Task-Specific Context

```bash
# Just the task details
granary show <task-id> --format prompt
```

## 4. Handoff Documents

Generate structured handoffs for sub-agents:

```bash
granary handoff --to "Implementation Agent" \
  --tasks task-1,task-2 \
  --constraints "Do not modify production code" \
  --acceptance-criteria "All tests pass"
```

## 5. Parallel Execution

For parallel workers, pass the session ID:

```bash
# Export session for sub-agents
eval $(granary session env)
# Sets GRANARY_SESSION environment variable

# Each sub-agent can then use the same session
GRANARY_SESSION=sess-xxx granary task <id> start --owner "Worker-1" --lease 30
```

### Preventing Conflicts

Sub-agents should claim tasks with leases:

```bash
# Sub-agent claims task (fails if already claimed)
granary task <task-id> claim --owner "Worker-1" --lease 30

# Exit code 4 means conflict - task claimed by another
```

## 6. Checkpointing

Before risky operations, create a checkpoint:

```bash
granary checkpoint create "before-major-refactor"

# If things go wrong
granary checkpoint restore before-major-refactor
```

## 7. Close Session When Done

```bash
granary session close --summary "Completed feature implementation"
```

## Example: Orchestrating a Feature Build

```bash
# Setup
granary session start "auth-feature" --owner "Orchestrator" --mode execute
granary session add auth-proj-abc1

# Process tasks
while TASK=$(granary next --json) && [ "$(echo $TASK | jq -r '.task')" != "null" ]; do
  TASK_ID=$(echo $TASK | jq -r '.task.id')
  TITLE=$(echo $TASK | jq -r '.task.title')

  echo "Processing: $TITLE"
  granary task $TASK_ID start --owner "Orchestrator"

  # Spawn sub-agent (implementation depends on your setup)
  # Pass: TASK_ID, task description, relevant context

  # Wait for completion, then:
  granary task $TASK_ID done
done

# Cleanup
granary session close --summary "Auth feature complete"
```
