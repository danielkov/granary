---
name: granary-plan-work
description: Plan and organize work into granary projects and tasks. Use when breaking down a feature, creating tasks, or setting up dependencies.
---

# Planning Work in Granary

Use this skill when you need to break down work into projects and tasks, or when an orchestrator determines a project needs planning before implementation.

## Before You Plan: Do Thorough Research

**Critical**: Before creating any tasks, conduct comprehensive research of the codebase. Each task description is the ONLY context a sub-agent receives—they cannot ask follow-up questions or explore further. Your research directly determines whether tasks succeed or fail.

### Check for Existing Work First

Before creating a new project or task, search for similar existing work:

```bash
# Search for similar projects and tasks
granary search "user profile"
granary search "authentication"
granary search "api endpoint"
```

**Decision tree based on search results:**

| Search Result | Action |
|---------------|--------|
| Exact match found (same feature) | Don't duplicate—add tasks to existing project or update existing tasks |
| Similar project exists | Review it—maybe extend it instead of creating a new project |
| Related tasks exist in other projects | Consider dependencies or consolidation |
| No matches | Safe to create new project |

**Example: Avoiding duplication**

```bash
$ granary search "profile"
Projects:
  user-profile-abc1: "User Profile" (3 tasks, 1 completed)

Tasks:
  settings-xyz9-task-2: "Add profile settings page"
```

In this case, don't create a new "User Profile" project—add your tasks to `user-profile-abc1` or coordinate with the existing work.

### Research Checklist

- [ ] **Search granary** - Check for existing similar projects and tasks
- [ ] **Locate all relevant files** - Find existing implementations, tests, configs
- [ ] **Understand existing patterns** - How does the codebase handle similar features?
- [ ] **Identify dependencies** - What modules/services will be affected?
- [ ] **Note file paths and line numbers** - Sub-agents need exact locations
- [ ] **Document function signatures** - What interfaces exist?
- [ ] **Find test patterns** - How are similar features tested?

### Research Tools

```bash
# Check for existing planned work
granary search "feature keywords"
granary projects                    # List all projects
granary project <id> tasks          # See tasks in a specific project

# Find relevant files in codebase
glob "**/*.rs" | grep -i auth
grep -r "UserService" src/

# Understand existing patterns
cat src/services/user.rs | head -100
```

Document your findings in steering files (see Section 5) before creating tasks.

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

**The task description is the ONLY context a sub-agent receives.** Sub-agents cannot explore the codebase or ask questions—they rely entirely on what you provide.

Include:

- **Goal**: One sentence on what this achieves
- **Context**: Why this matters, background info
- **Requirements**: Specific deliverables
- **Files to Modify**: Exact paths with line numbers where relevant
- **Implementation Details**: Suggested approach, function signatures, patterns to follow
- **Acceptance Criteria**: How to verify completion

### Bad vs Good Task Descriptions

**Bad**: "Fix the auth bug"

**Good**:

````
**Goal:** Fix null pointer in UserService.getById when user not found.

**Context:** Currently throws NullPointerException when querying non-existent user IDs. Should gracefully handle missing users.

**Files to Modify:**
- `src/services/user_service.rs:142-156` - The `get_by_id` function
- `src/errors/mod.rs:23` - Add UserNotFoundException if not present
- `tests/services/user_service_test.rs` - Add test case

**Implementation Details:**
The existing pattern in `src/services/post_service.rs:89` handles this well:
```rust
pub fn get_by_id(id: Uuid) -> Result<Option<Post>, ServiceError> {
    // Returns Ok(None) for missing records
}
````

Follow this pattern—return `Result<Option<User>, ServiceError>` instead of `Result<User, ServiceError>`.

**Acceptance Criteria:**

- [ ] `get_by_id` returns `Ok(None)` for missing users
- [ ] Existing callers updated to handle `Option`
- [ ] Test added for missing user case

```

### Linking Files in Descriptions

Always provide exact file paths and line numbers:

| Instead of...                | Write...                                          |
| ---------------------------- | ------------------------------------------------- |
| "the user service"           | `src/services/user_service.rs`                    |
| "the auth middleware"        | `src/middleware/auth.rs:45-67`                    |
| "follow the existing pattern"| "follow pattern in `src/handlers/posts.rs:23-45`" |
| "add tests"                  | "add tests in `tests/unit/user_test.rs`"          |

### Implementation Detail Examples

**For new endpoints:**
```

**Implementation Details:**
Add handler in `src/handlers/profile.rs`. Follow the pattern from `src/handlers/user.rs:34-56`:

1. Create `GetProfileHandler` struct implementing `Handler` trait
2. Use `ProfileService::get_by_user_id()` (already exists at `src/services/profile.rs:12`)
3. Return `ApiResponse<ProfileDto>` matching existing response format
4. Register route in `src/routes/mod.rs:45` following existing pattern

```

**For refactoring:**
```

**Implementation Details:**
Extract validation logic from `src/handlers/user.rs:78-112` into `src/validators/user_validator.rs`.

Existing validator pattern at `src/validators/post_validator.rs`:

- Implement `Validator<T>` trait
- Return `ValidationResult` with field-level errors
- Use `#[derive(Validate)]` macro for struct validation

The three validation functions to extract:

- `validate_email()` (lines 82-89)
- `validate_password()` (lines 91-98)
- `validate_username()` (lines 100-110)

```

**For bug fixes:**
```

**Implementation Details:**
Root cause: `calculate_total()` at `src/cart/pricing.rs:45` doesn't account for null discounts.

Fix approach:

1. Add null check before discount application (line 52)
2. Use `discount.unwrap_or(Decimal::ZERO)` pattern (see `src/cart/tax.rs:23`)
3. Add debug logging using existing `tracing::debug!` macro

````

## 3. Add Dependencies

If tasks must be done in order:

```bash
# task-2 depends on task-1 (task-2 cannot start until task-1 is done)
granary task <project-id>-task-2 deps add <project-id>-task-1

# View dependencies
granary task <project-id>-task-2 deps graph
````

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

### Step 1: Check for Existing Work

First, search granary for similar projects or tasks:

```bash
$ granary search "profile"
No matching projects or tasks found.

$ granary search "user"
Projects:
  user-auth-def2: "User Authentication" (4 tasks, 4 completed)

Tasks:
  user-auth-def2-task-3: "Implement user login endpoint"
```

**Analysis:** No existing profile work, but there's a completed auth project. We can reference patterns from `user-auth-def2` when building our profile feature. Safe to create a new project.

### Step 2: Research the Codebase

Now thoroughly research the codebase:

```bash
# Find existing user-related code
glob "**/*user*"
grep -r "struct User" src/

# Understand the API patterns
cat src/handlers/mod.rs
cat src/routes/mod.rs | head -50

# Check existing types
cat src/types/user.rs

# Find test patterns
ls tests/
cat tests/handlers/user_test.rs | head -30
```

**Research findings to document:**

- User model at `src/models/user.rs:12-45` has `id`, `email`, `name`, `created_at`
- Missing: `bio`, `avatar_url` fields needed for profile
- Handler pattern at `src/handlers/user.rs:23-67` - uses `axum::Json<T>`
- Routes registered in `src/routes/api.rs:34`
- Tests use `mockall` for service mocking (see `tests/handlers/user_test.rs:5`)

### Step 3: Create Project and Tasks

```bash
# Create project
granary projects create "User Profile" --description "Add user profile page with edit capability. Extends existing User model with profile fields."
# Output: user-profile-abc1

# Task 1: Extend the data model
granary project user-profile-abc1 tasks create "Extend User model with profile fields" \
  --description "**Goal:** Add profile fields to the User model and create database migration.

**Context:** User model exists but lacks profile-specific fields.

**Files to Modify:**
- \`src/models/user.rs:12-45\` - Add new fields to User struct
- \`src/schema.rs\` - Update diesel schema (auto-generated after migration)
- \`migrations/\` - Create new migration file

**Implementation Details:**
Add these fields to the User struct at \`src/models/user.rs:12\`:
\`\`\`rust
pub struct User {
    // existing fields...
    pub bio: Option<String>,        // max 500 chars
    pub avatar_url: Option<String>, // validated URL
}
\`\`\`

Create migration following pattern in \`migrations/2024_01_15_create_users/up.sql\`:
\`\`\`sql
ALTER TABLE users ADD COLUMN bio TEXT;
ALTER TABLE users ADD COLUMN avatar_url VARCHAR(2048);
\`\`\`

**Acceptance Criteria:**
- [ ] Migration runs successfully (\`diesel migration run\`)
- [ ] User struct compiles with new fields
- [ ] Existing user queries still work" \
  --priority P0
# Output: user-profile-abc1-task-1

# Task 2: Create Profile DTOs and validation
granary project user-profile-abc1 tasks create "Create Profile DTOs with validation" \
  --description "**Goal:** Create request/response DTOs for profile endpoints.

**Files to Modify:**
- \`src/types/mod.rs:1\` - Add mod declaration
- \`src/types/profile.rs\` - New file for DTOs

**Implementation Details:**
Follow existing DTO pattern at \`src/types/user.rs:8-25\`:
\`\`\`rust
#[derive(Serialize, Deserialize)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub bio: Option<String>,
    #[validate(url)]
    pub avatar_url: Option<String>,
}
\`\`\`

Use \`validator\` crate (already in Cargo.toml) for validation.

**Acceptance Criteria:**
- [ ] DTOs serialize/deserialize correctly
- [ ] Validation rejects invalid inputs
- [ ] Unit tests in \`tests/types/profile_test.rs\`" \
  --priority P0 \
  --dependencies user-profile-abc1-task-1
# Output: user-profile-abc1-task-2

# Task 3: Implement API handlers
granary project user-profile-abc1 tasks create "Implement profile API handlers" \
  --description "**Goal:** Create GET and PUT handlers for profile endpoints.

**Context:** DTOs defined in task-2. User model extended in task-1.

**Files to Modify:**
- \`src/handlers/mod.rs:5\` - Add mod declaration
- \`src/handlers/profile.rs\` - New handler file
- \`src/routes/api.rs:34\` - Register new routes
- \`tests/handlers/profile_test.rs\` - New test file

**Implementation Details:**
Follow handler pattern at \`src/handlers/user.rs:23-67\`:

\`\`\`rust
// GET /api/profile/:id
pub async fn get_profile(
    State(pool): State<DbPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let user = UserService::get_by_id(&pool, user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(ProfileResponse::from(user)))
}

// PUT /api/profile/:id (requires auth)
pub async fn update_profile(
    State(pool): State<DbPool>,
    Extension(current_user): Extension<AuthUser>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    // Verify user can only update own profile
    if current_user.id != user_id {
        return Err(ApiError::Forbidden);
    }
    req.validate()?;
    // ... update logic
}
\`\`\`

Register routes in \`src/routes/api.rs:34\` following existing pattern:
\`\`\`rust
.route(\"/profile/:id\", get(get_profile))
.route(\"/profile/:id\", put(update_profile).layer(auth_middleware()))
\`\`\`

**Acceptance Criteria:**
- [ ] GET returns profile data
- [ ] PUT requires authentication
- [ ] PUT validates ownership (users can only edit own profile)
- [ ] Integration tests pass" \
  --priority P1 \
  --dependencies user-profile-abc1-task-2
# Output: user-profile-abc1-task-3

# Add steering file with research findings
cat > docs/profile-patterns.md << 'EOF'
# Profile Feature Patterns

## Existing Codebase Patterns
- Handlers use `axum` extractors: `State`, `Path`, `Json`, `Extension`
- Auth middleware at `src/middleware/auth.rs:12` adds `AuthUser` to extensions
- Error handling uses `ApiError` enum at `src/errors/api_error.rs`
- All responses wrapped in `Json<T>`

## Database Access
- Use `UserService` at `src/services/user.rs` for queries
- Connection pool passed via `State<DbPool>`
- Async queries with `sqlx`

## Testing
- Integration tests in `tests/handlers/`
- Use `TestApp::new()` helper at `tests/common/mod.rs:8`
- Mock services with `mockall` when needed
EOF

granary steering add docs/profile-patterns.md --project user-profile-abc1
```

## Summary

1. **Search for existing work** - Use `granary search` to find similar projects/tasks before creating duplicates
2. **Research thoroughly** - Explore the codebase, find patterns, note file paths and line numbers
3. **Create project** with clear description (or add to existing project if found)
4. **Break into tasks** with detailed descriptions including:
   - Exact file paths with line numbers
   - Implementation details with code examples
   - References to existing patterns to follow
5. **Add dependencies** only where truly required
6. **Set up steering** to document research findings and conventions
7. Hand off to `/granary:orchestrate` for implementation

**Remember:** Sub-agents only see task descriptions. Every detail they need must be in the task or steering files—they cannot explore or ask questions.
