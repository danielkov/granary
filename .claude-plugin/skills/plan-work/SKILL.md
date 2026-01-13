---
name: granary-plan-work
description: Plan and organize work into granary projects and tasks. Use when breaking down a feature, creating tasks, or setting up dependencies.
---

# Planning Work in Granary

Use this skill when you need to break down work into projects and tasks, or when an orchestrator determines a project needs planning before implementation.

## 1. Create a Project

Group related tasks into a project:

```bash
granary projects create "Feature Name" --description "Clear description of what this project achieves"
```

Output gives you a project ID like `feature-name-abc1`.

## 2. Break Down into Tasks

Create focused, actionable tasks:

```bash
granary project <project-id> tasks create "Task title" \
  --description "**Goal:** What this task accomplishes

**Context:** Why this task exists

**Requirements:**
- Specific deliverable 1
- Specific deliverable 2

**Acceptance Criteria:**
- [ ] Criterion 1
- [ ] Criterion 2" \
  --priority P1
```

### Priority Levels

| Priority | When to Use                   |
| -------- | ----------------------------- |
| P0       | Critical, blocking other work |
| P1       | Important, should do soon     |
| P2       | Normal priority (default)     |
| P3       | Nice to have                  |

### Writing Good Task Descriptions

**The task description is the ONLY context a sub-agent receives.**

Include:

- **Goal**: One sentence on what this achieves
- **Context**: Why this matters, background info
- **Requirements**: Specific deliverables
- **Technical Notes**: File locations, patterns to follow
- **Acceptance Criteria**: How to verify completion

**Bad**: "Fix the auth bug"
**Good**: "Fix null pointer in UserService.getById when user not found. Should return null or throw UserNotFoundException. See error in logs/app.log line 142."

## 3. Add Dependencies

If tasks must be done in order:

```bash
# task-2 depends on task-1 (task-2 cannot start until task-1 is done)
granary task <project-id>-task-2 deps add <project-id>-task-1

# View dependencies
granary task <project-id>-task-2 deps graph
```

You can also specify dependencies at task creation time:

```bash
granary project <project-id> tasks create "Dependent task" \
  --dependencies <project-id>-task-1
```

Only add dependencies when truly required - over-constraining reduces parallelism.

## 4. Start a Planning Session (Optional)

For complex planning:

```bash
granary session start "planning-feature-x" --mode plan
granary session add <project-id>
```

## 5. Steering Files

Steering files provide standards, conventions, and context that sub-agents should follow during implementation. Set these up during planning so orchestrators and sub-agents have the guidance they need.

### Steering Scopes

| Scope            | When Included                         | Use Case                  |
| ---------------- | ------------------------------------- | ------------------------- |
| Global (default) | Always in context/handoffs            | Project-wide standards    |
| `--project <id>` | When project is in session scope      | Project-specific patterns |
| `--task <id>`    | When handing off that specific task   | Task-specific research    |
| `--for-session`  | During session, auto-deleted on close | Temporary research notes  |

### Adding Steering Files (optional)

Steering files are useful for:

- Transient files, produced during deep research, e.g.:
  - Summary of a technical article
  - Explanation of how a system works
- Task or project specific guidelines

```bash
# Global steering (always included)
granary steering add docs/coding-standards.md

# Project-attached (only when this project is in context)
granary steering add docs/auth-patterns.md --project auth-proj-abc1

# Task-attached (only in handoffs for this specific task)
granary steering add .granary/task-research.md --task auth-proj-abc1-task-3

# Session-attached (temporary, auto-deleted on session close)
granary steering add .granary/temp-notes.md --for-session

# List current steering files
granary steering list

# Remove steering (specify scope to match)
granary steering rm docs/auth-patterns.md --project auth-proj-abc1
```

### When to Use Each Scope

- **Global**: Project-wide coding standards, architecture decisions
- **Project-attached**: Module-specific patterns (e.g., auth module conventions)
- **Task-attached**: Research specific to one task (avoid polluting other handoffs)
- **Session-attached**: Temporary research during planning that shouldn't persist

### Example: Adding Steering During Planning

```bash
# During planning, document patterns you discover
cat > docs/auth-patterns.md << 'EOF'
# Authentication Patterns

## Existing Conventions
- Auth middleware in src/middleware/auth.rs uses JWT tokens
- User model in src/models/user.rs with bcrypt password hashing
- Session storage uses Redis (see src/services/session.rs)

## Key Conventions
- All API endpoints return JSON with {data, error, meta} structure
- Use `ApiError` type for error handling
- Tests go in tests/ directory, not inline
EOF

# Attach to the project so sub-agents get this context
granary steering add docs/auth-patterns.md --project auth-proj-abc1
```

## Example: Planning a User Profile Feature

```bash
# Create project
granary projects create "User Profile" --description "Add user profile page with edit capability"
# Output: user-profile-abc1

# Create tasks with detailed descriptions
granary project user-profile-abc1 tasks create "Design profile API schema" \
  --description "**Goal:** Define the API schema for user profile endpoints.

**Requirements:**
- GET /api/profile/:id endpoint
- PUT /api/profile/:id endpoint for updates
- Schema should include: name, email, avatar_url, bio

**Acceptance Criteria:**
- [ ] OpenAPI spec created in docs/api/
- [ ] Types defined in src/types/" \
  --priority P0
# Output: user-profile-abc1-task-1

granary project user-profile-abc1 tasks create "Implement profile API endpoints" \
  --description "**Goal:** Implement the profile API based on the schema.

**Context:** Schema defined in task-1

**Requirements:**
- Implement GET and PUT handlers
- Add validation for profile updates
- Add auth middleware to protect endpoints

**Acceptance Criteria:**
- [ ] Endpoints return correct responses
- [ ] Tests pass" \
  --priority P1 \
  --dependencies user-profile-abc1-task-1
# Output: user-profile-abc1-task-2

granary project user-profile-abc1 tasks create "Create profile UI components" \
  --description "**Goal:** Build React components for profile page.

**Requirements:**
- ProfileView component (read-only display)
- ProfileEdit component (form with validation)
- Connect to API endpoints

**Acceptance Criteria:**
- [ ] Components render correctly
- [ ] Form validation works
- [ ] API integration complete" \
  --priority P1 \
  --dependencies user-profile-abc1-task-2
# Output: user-profile-abc1-task-3

# Add project-specific steering
granary steering add docs/profile-patterns.md --project user-profile-abc1
```

## Summary

1. **Create project** with clear description
2. **Break into tasks** with detailed descriptions (this is what sub-agents see)
3. **Add dependencies** only where truly required
4. **Set up steering** to provide context and conventions
5. Hand off to `/granary:orchestrate` for implementation
