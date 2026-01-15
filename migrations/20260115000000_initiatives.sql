-- Initiatives table
-- Initiatives are top-level entities that group related projects

CREATE TABLE IF NOT EXISTS initiatives (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    owner TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    tags TEXT,  -- JSON array
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_initiatives_status ON initiatives(status);
CREATE INDEX IF NOT EXISTS idx_initiatives_slug ON initiatives(slug);

-- Project dependencies table
-- Allows projects to depend on other projects with cycle detection
CREATE TABLE IF NOT EXISTS project_dependencies (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    depends_on_project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    PRIMARY KEY (project_id, depends_on_project_id),
    CHECK (project_id != depends_on_project_id)
);

CREATE INDEX IF NOT EXISTS idx_project_deps_project ON project_dependencies(project_id);
CREATE INDEX IF NOT EXISTS idx_project_deps_depends_on ON project_dependencies(depends_on_project_id);

-- Initiative-Project many-to-many relationship
-- A project can belong to multiple initiatives, and an initiative can contain multiple projects
CREATE TABLE IF NOT EXISTS initiative_projects (
    initiative_id TEXT NOT NULL REFERENCES initiatives(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    added_at TEXT NOT NULL,
    PRIMARY KEY (initiative_id, project_id)
);

CREATE INDEX IF NOT EXISTS idx_init_proj_initiative ON initiative_projects(initiative_id);
CREATE INDEX IF NOT EXISTS idx_init_proj_project ON initiative_projects(project_id);
