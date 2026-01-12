---
name: granary-execute-task
description: Execute an assigned granary task as a sub-agent. Use when you receive a task ID to complete.
---

# Executing a Granary Task

Use this skill when you're a sub-agent assigned to complete a specific task.

## 1. View Task Details

First, understand what you need to do:

```bash
granary show <task-id>
# or
granary task <task-id>
```

Read the task description carefully - it contains all the context you need.

## 2. Start the Task

Mark the task as in-progress and optionally claim it with a lease:

```bash
# Simple start
granary task <task-id> start --owner "Worker"

# With lease (prevents conflicts in multi-agent scenarios)
granary task <task-id> start --owner "Worker" --lease 30
```

The lease (in minutes) ensures no other agent claims this task while you work.

## 3. Do the Work

Perform the actual implementation described in the task.

### Record Progress

Add comments as you work to maintain visibility:

```bash
granary task <task-id> comments create "Started implementing the login form"
granary task <task-id> comments create "Completed form validation, now adding API integration" --kind progress
```

Comment kinds: `note`, `progress`, `decision`, `blocker`

### If You Get Blocked

If you cannot continue:

```bash
granary task <task-id> block --reason "Waiting for API credentials from ops team"
```

## 4. Complete the Task

When finished:

```bash
granary task <task-id> done --comment "Implemented login form with validation and API integration"
```

## 5. Handle Failures

If you cannot complete the task, release it so others can pick it up:

```bash
granary task <task-id> release
```

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
