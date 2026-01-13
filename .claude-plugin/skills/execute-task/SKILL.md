---
name: granary-execute-task
description: Execute an assigned granary task as a sub-agent. Use when you receive a task ID to complete.
---

# Executing a Granary Task

You are a **sub-agent** spawned by an orchestrator to complete a specific task. Your responsibility is to:

1. Implement the task as described
2. Report your progress
3. Signal completion or blockers back to the orchestrator

**You do NOT coordinate other agents. You do the actual work.**

## Step 1: Understand Your Task

View the task details:

```bash
granary show <task-id>
# or
granary task <task-id>
```

Read the task description carefully - it contains all the context you need. If the orchestrator provided handoff context, that also contains relevant information.

## Step 2: Start the Task

Mark the task as in-progress:

```bash
granary task <task-id> start
```

If working in a multi-agent environment, claim with a lease to prevent conflicts:

```bash
granary task <task-id> start --lease 30
```

The lease (in minutes) ensures no other agent claims this task while you work.

## Step 3: Do the Work

**This is your main responsibility.** Implement whatever the task description specifies:

- Write code
- Create files
- Run tests
- Fix bugs
- Whatever the task requires

### Record Progress

Add comments as you work to maintain visibility for the orchestrator:

```bash
granary task <task-id> comments create "Started implementing the login form"
granary task <task-id> comments create "Completed form validation, now adding API integration" --kind progress
```

Comment kinds: `note`, `progress`, `decision`, `blocker`

## Step 4: Report Completion

When you have successfully completed the task:

```bash
granary task <task-id> done --comment "Implemented login form with validation and API integration"
```

**Important**: Only mark done when the task is truly complete according to its acceptance criteria.

## Handling Problems

### If You Get Blocked

If you cannot continue due to external factors:

```bash
granary task <task-id> block --reason "Waiting for API credentials from ops team"
```

This signals to the orchestrator that this task needs attention.

### If You Cannot Complete

If you cannot complete the task for any reason, release it so others can pick it up:

```bash
granary task <task-id> release
```

Then report back to the orchestrator with what went wrong.

## Multi-Agent Safety

If working in parallel with other agents:

```bash
# Extend lease for long-running tasks
granary task <task-id> heartbeat --lease 30

# Check if task is still yours
granary task <task-id>
```

Exit codes to watch for:

- `4` = Conflict (task claimed by another agent)
- `5` = Blocked (dependencies not met)

## Summary

As a sub-agent, your workflow is:

1. **Read task** → Understand what to do
2. **Start task** → `granary task <id> start`
3. **Do the work** → Actually implement the task
4. **Record progress** → `granary task <id> comments create "..."`
5. **Complete or block** → `granary task <id> done` or `granary task <id> block`

You are the implementer. Stay focused on your assigned task.
