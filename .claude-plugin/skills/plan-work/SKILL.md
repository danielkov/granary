---
name: granary-plan-work
description: Plan and organize work into granary projects and tasks. Use when breaking down a feature, creating tasks, or setting up dependencies.
---

# Planning Work in Granary

Use this skill when you need to break down work into projects and tasks.

## 1. Create a Project

Group related tasks into a project:

```bash
granary projects create "Feature Name" \
  --description "Clear description of what this project achieves" \
  --owner "Team or Person"
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

Only add dependencies when truly required - over-constraining reduces parallelism.

## 4. Start a Planning Session (Optional)

For complex planning:

```bash
granary session start "planning-feature-x" --owner "Planner" --mode plan
granary session add <project-id>
```

## Example: Planning a User Profile Feature

```bash
# Create project
granary projects create "User Profile" --description "Add user profile page with edit capability"
# Output: user-profile-abc1

# Create tasks
granary project user-profile-abc1 tasks create "Design profile API schema" --priority P0
# Output: user-profile-abc1-task-1

granary project user-profile-abc1 tasks create "Implement profile API endpoints" --priority P1
# Output: user-profile-abc1-task-2

granary project user-profile-abc1 tasks create "Create profile UI components" --priority P1
# Output: user-profile-abc1-task-3

# Add dependencies
granary task user-profile-abc1-task-2 deps add user-profile-abc1-task-1
granary task user-profile-abc1-task-3 deps add user-profile-abc1-task-2
```
