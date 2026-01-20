# Granary workers

Workers are an extension to `granary`'s core context management capabilities. They allow the user to spawn arbitrary number of agentic tools, reacting to `granary` events in the given workspace.

## Example use case

Start a worker that spawns an agent each time a new task is unblocked. The agent receives instructions to implement the task.

## Granary global config

A global configuration file is to be added. This can be modified with `granary config` commands or directly. Config file is in TOML format and is at `~/.granary/config.toml`.

```toml
[runners.claude-code]
command = "claude"
args = [
  "--prompt",
  "{output}" # the output of each iteration gets substituted here
]

# More runners can be defined here
```

## Interface

```sh
granary workers # or granary worker list
granary worker
granary worker start --runner claude-code
granary worker start --runner claude-code --on "task.unblocked"
granary worker start --runner claude-code --on "task.updated" --filters="project=abc-123" --filters="status!=draft" # allow filters - scoped to the event
granary worker start --detached # or -d - start the worker as a deamon
granary worker start --detached # or -d - start the worker as a deamon (default to attaching worker to stdout)
granary worker start --concurrency # amount of runners this worker can spawn at once
granary worker <worker_id> status
granary worker <worker_id> logs
granary worker <worker_id> stop
granary worker <worker_id> stop --runs # also stops all runs spawned by this worker
granary worker prune

# runs
granary runs # or granary run list
granary run <run_id> status
granary run <run_id> logs
granary run <run_id> stop
granary run <run_id> pause
granary run <run_id> resume

# runners (config)
granary config edit runners.claude-code --command=claude
granary config add runners claude-code --command=claude --args="--prompt" --args="{output}"
granary config edit # spawns $EDITOR
```

# Events

- `task.created` - When a new task is created
- `task.updated` - When a task is updated
- `task.unblocked` - When a task is unblocked (when it becomes eligible to be shown for `granary next --all`)
- `task.complete` - When a task is marked as complete
- `project.created` - When a new project is created
- `project.updated` - When a project is updated
- `project.unblocked` - When a project becomes unblocked (all dependencies are resolved)
- `project.archived` - When a project is archived
- `initiative.created` - When a new initiative is created
- `initiative.updated` - When an initiative is updated

## Filters

Filters are scoped to the event, e.g.: for `task.xxx` events each filter will match on `task.property`, e.g.: `status!=draft` will check the project's status. Filters that don't resolve are ignored and a warning is logged in the worker's logs.

## Config

Each "runner config" can be used to define the startup parameters of the runner, launched by workers, e.g.:

```toml
[runners.claude-sonnet-task]
command = "claude"
args = [
  "--model",
  "sonnet",
  "--prompt",
  "\'Execute granary task {task.id}. Use granary handoff --to agent --tasks {task.id} to get context, then start the task with granary task {task.id} start, implement it, and mark it done when complete.\'"
]
concurrency = 5

[runners.claude-opus-task]
command = "claude"
args = [
  "--model",
  "opus",
  "--prompt",
  "\'Execute granary task {task.id}. Use granary handoff --to agent --tasks {task.id} to get context, then start the task with granary task {task.id} start, implement it, and mark it done when complete.\'"
]
concurrency = 2

[runners.claude-planner]
command = "claude"
args = [
  "--model",
  "opus",
  "--prompt",
  "\'Use granary to plan project: {project.id}\'"
]
concurrency = 2

# a runner can also be used to trigger other tasks, e.g.: post a message to slack
[runners.slack-notifier]
command = "curl"
args = [
  "-sS",
  "-X",
  "POST",
  "https://slack.com/api/chat.postMessage",
  "-H",
  '"Authorization: Bearer ${SLACK_TOKEN}"',
  "-H",
  '"Content-Type: application/json; charset=utf-8"',
  "--data",
  '"{\"channel\":\"abc123\",\"text\":\"Task {task.id} is finished\"}"'
]
on = "task.complete"
```

All of the runner's configuration options can also be defined as command line arguments. E.g.:

```sh
granary worker start --command=claude --args="--model" --args="opus" --args="--prompt" --args="{output}" --concurrency 3 --on=task.unblocked
```

## When runners error

Runners may occasionally fail. We may have intermittent or persistent failures. For each "event" for which a runner spawns, granary tracks return codes. If the run exits non-zero, granary attempts to retry the task. We use a configurable exponential backoff strategy with additional jitter.

## What happens if the workspace directory is deleted?

Worker detects `instance_path` missing / unreadable:

- transitions to `error: instance missing`
- stops consuming events
- keeps itself controllable from global CLI
- `granary worker stop <id>` still works (it’s global).
- `granary worker prune` removes dead workers from the registry.

## Prerequisites

- Add a `draft` state to tasks
- All tasks need to be created in `draft` state
- Tasks should only go from `draft` to `todo` state when their descriptions are complete and dependencies are set
- Tasks in `draft` state do not show up in `granary next`
- Tasks in `draft` state cannot be started
- Need to update task `plan-work` skill to only put tasks in `todo` state once their description is fully complete and all of their dependencies are connected.
