# Granary

A CLI context hub for agentic work. Granary helps LLM agents and orchestrators manage projects, tasks, sessions, and context in a structured, machine-readable way.

## Features

- **Session-centric**: Explicit "what's in context" for each agent run
- **LLM-first I/O**: Every command has `--json` and `--format prompt` for machine consumption
- **Local-first**: All state stored locally (SQLite), no network dependency
- **Concurrency-tolerant**: Task claiming with leases for multi-agent safety
- **Context packs**: Generate summaries and handoffs optimized for LLM context windows

## Installation

### macOS / Linux

```sh
curl -sSfL https://raw.githubusercontent.com/danielkov/granary/main/scripts/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/danielkov/granary/main/scripts/install.ps1 | iex
```

### From source

Requires [Rust](https://rustup.rs/):

```sh
cargo install --git https://github.com/danielkov/granary.git
```

## Quick Start

```sh
# Initialize a workspace
granary init

# Create a project
granary projects create "My Project" --description "Building something great"

# Start a session
granary session start "feature-work" --owner "Claude Code"

# Add tasks
granary project my-project-xxxx tasks create "Implement login" --priority P0
granary project my-project-xxxx tasks create "Add tests" --priority P1

# Get the next actionable task
granary next

# Start working on a task
granary start my-project-xxxx-task-1

# Mark it done
granary task my-project-xxxx-task-1 done

# Get a summary for your LLM
granary summary --format prompt
```

## Why Granary?

Granary is designed for the agentic loop pattern:

1. **Plan**: Create projects and tasks, set dependencies
2. **Execute**: Agents claim tasks, work on them, report progress
3. **Coordinate**: Multiple agents can work safely in parallel with leases
4. **Handoff**: Generate context packs for sub-agents or human review

### Key Concepts

- **Workspace**: A directory (typically a repo) containing `.granary/`
- **Project**: Long-lived initiative with tasks and steering references
- **Task**: Unit of work with status, priority, dependencies, and claiming
- **Session**: Container for "what's in context" for a run
- **Checkpoint**: Snapshot of state for pause/resume or rollback

## Commands

```
granary init          # Initialize workspace
granary projects      # List/create projects
granary tasks         # List tasks in session scope
granary next          # Get next actionable task
granary start <id>    # Start working on a task
granary summary       # Generate work summary
granary context       # Export context pack for LLM
granary handoff       # Generate handoff for sub-agent
granary checkpoint    # Create/restore checkpoints
```

Use `granary --help` or `granary <command> --help` for detailed usage.

## Output Formats

Every command supports multiple output formats:

```sh
granary tasks                    # Human-readable table
granary tasks --json             # JSON for parsing
granary tasks --format yaml      # YAML
granary tasks --format md        # Markdown
granary tasks --format prompt    # Optimized for LLM context
```

## Integration with Claude Code

Granary works seamlessly with Claude Code and other LLM coding assistants:

```sh
# Set session for sub-agents
eval $(granary session env)

# Generate context for prompts
granary context --format prompt --token-budget 2000

# Handoff to a review agent
granary handoff --to "Code Review Agent" --tasks task-1,task-2
```

## License

MIT
