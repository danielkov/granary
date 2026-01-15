---
name: granary-initiative-planner
description: Plan and organize multi-project initiatives. Use when work spans multiple services, repositories, or has natural project boundaries.
---

# Planning Multi-Project Initiatives

Use this skill when work naturally spans multiple projects or services. **Do NOT use for single-feature work**—use `/granary:plan-work` instead.

**Your role:** High-level architecture, separation of concerns, dependency analysis, and spawning sub-agents for project planning. You do NOT plan individual project tasks yourself.

## When to Use Initiatives

**Use initiatives when:**

- Work spans multiple services (API + frontend + workers)
- Implementation has clear phases with dependencies (schema -> API -> UI)
- Multiple teams or agents will work in parallel on different components
- You need to track cross-project dependencies

**Do NOT use initiatives when:**

- Work fits in a single project
- There are no cross-project dependencies
- It's a single feature in one service

## Step 1: Research and Assess

Before creating an initiative, do high-level research to understand the scope:

```bash
# Search for related existing work
granary search "feature keywords"
granary initiatives  # Check existing initiatives
```

Research the codebase to understand:

- What services/modules exist that will be affected
- What boundaries already exist in the architecture
- What shared dependencies or interfaces might be needed

**Decision tree:**

| Situation                                         | Action                              |
| ------------------------------------------------- | ----------------------------------- |
| Work fits in one project                          | Use `/granary:plan-work` instead    |
| Related initiative exists                         | Add projects to existing initiative |
| Work spans 2+ distinct projects with dependencies | Create new initiative               |

## Step 2: Create the Initiative

```bash
granary initiatives create "Initiative Name" \
  --description "High-level goal spanning multiple projects"
```

Example:

```bash
granary initiatives create "User Authentication System" \
  --description "Implement auth across API, web app, and mobile app with shared token service"
```

## Step 3: Design Separation of Concerns

Break the initiative into logical projects. Each project should:

- Be independently implementable (given its dependencies)
- Have a clear boundary (service, module, or phase)
- Be completable by a single agent

Think carefully about:

- **What depends on what?** Which components need to exist before others can start?
- **What can run in parallel?** Which projects have no dependencies on each other?
- **Where are the interfaces?** What contracts need to be defined between projects?

Example breakdown:

```
Initiative: User Authentication System
├── Project: Auth Token Service (shared backend) — no deps, can start first
├── Project: API Auth Integration — depends on token service
├── Project: Web App Login — depends on API auth
└── Project: Mobile App Login — depends on API auth (parallel with Web App)
```

## Step 4: Create Projects and Add to Initiative

```bash
# Create each project with clear descriptions for sub-agents
granary projects create "Auth Token Service" \
  --description "JWT token generation and validation service"
# Output: auth-token-service-abc1

granary projects create "API Auth Integration" \
  --description "Add authentication middleware to API endpoints"
# Output: api-auth-integration-def2

# Add projects to initiative
granary initiative <initiative-id> add-project auth-token-service-abc1
granary initiative <initiative-id> add-project api-auth-integration-def2
```

## Step 5: Analyze and Set Up Project Dependencies

**This is critical.** Review each project pair and ask:

- Does project A need anything from project B to start?
- Does project B produce interfaces/APIs that project A consumes?
- Are there shared schemas or contracts that must exist first?

```bash
# API Auth depends on Token Service being complete
granary project api-auth-integration-def2 deps add auth-token-service-abc1

# Web App depends on API Auth
granary project web-app-login-ghi3 deps add api-auth-integration-def2
```

**Key principle:** A project is blocked until ALL its dependency projects have ALL tasks done.

## Step 6: Verify the Dependency Structure

Before spawning sub-agents, verify the structure makes sense:

```bash
# View the initiative dependency graph
granary initiative <initiative-id> graph

# View as mermaid diagram (paste into GitHub/VSCode)
granary initiative <initiative-id> graph --format mermaid

# Check overall status
granary initiative <initiative-id> summary
```

Review the graph for:

- Circular dependencies (error)
- Missing dependencies (projects that should depend on each other but don't)
- Overly sequential structure (could more projects run in parallel?)

## Step 7: Spawn Sub-Agents for Project Planning

**Do NOT plan project tasks yourself.** Spawn sub-agents to handle detailed project planning.

For each unblocked project, spawn a sub-agent with `/granary:plan-work`:

```bash
# Get unblocked projects
granary initiative <initiative-id> next --all
```

Then spawn sub-agents (using the Task tool) with prompts like:

> "Use /granary:plan-work to plan the project `auth-token-service-abc1`.
> This project is part of the User Authentication System initiative.
> It should implement JWT token generation and validation.
> Create detailed tasks for implementation."

Spawn agents for all unblocked projects in parallel when possible.

## Step 8: Hand Off to Orchestration

Once all projects have been planned by sub-agents, the initiative is ready for `/granary:orchestrate`:

```bash
# Verify all projects have tasks
granary initiative <initiative-id> summary

# Get the next actionable task across the entire initiative
granary initiative <initiative-id> next

# Get ALL unblocked tasks (for parallel execution)
granary initiative <initiative-id> next --all
```

## Example: Full Initiative Planning Flow

```bash
# 1. Research the codebase (understand existing architecture)

# 2. Create initiative
granary initiatives create "Payment Processing" \
  --description "Add payment support: Stripe integration, checkout flow, order management"

# 3. Design separation of concerns and create projects
granary projects create "Stripe Service" --description "Stripe API integration wrapper"
granary projects create "Checkout API" --description "Checkout endpoints and cart management"
granary projects create "Checkout UI" --description "Frontend checkout flow"
granary projects create "Order Service" --description "Order persistence and status tracking"

# 4. Add to initiative
granary initiative payment-processing-xyz1 add-project stripe-service-abc1
granary initiative payment-processing-xyz1 add-project checkout-api-def2
granary initiative payment-processing-xyz1 add-project checkout-ui-ghi3
granary initiative payment-processing-xyz1 add-project order-service-jkl4

# 5. Analyze dependencies and set them up
granary project checkout-api-def2 deps add stripe-service-abc1
granary project checkout-ui-ghi3 deps add checkout-api-def2
granary project order-service-jkl4 deps add checkout-api-def2

# 6. Verify structure
granary initiative payment-processing-xyz1 graph --format mermaid

# 7. Spawn sub-agents to plan unblocked projects
# stripe-service-abc1 is unblocked -> spawn agent with /granary:plan-work

# 8. Once planned, check readiness
granary initiative payment-processing-xyz1 summary
```

## Summary

1. **Research** -> Understand the codebase and architecture
2. **Assess** -> Is this truly multi-project work?
3. **Create initiative** -> High-level container
4. **Design separation** -> Clear boundaries, identify interfaces
5. **Create projects** -> Add to initiative with good descriptions
6. **Analyze dependencies** -> Review all project pairs, set dependencies
7. **Verify structure** -> Check graph for issues
8. **Spawn sub-agents** -> Each unblocked project gets a planning agent
9. **Hand off** -> Orchestrator uses initiative-level commands

**Your output:** A well-structured initiative with projects and dependencies. Sub-agents handle detailed task planning within each project.
