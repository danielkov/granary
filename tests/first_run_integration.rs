use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use tempfile::TempDir;

fn granary_bin() -> &'static PathBuf {
    static BIN: OnceLock<PathBuf> = OnceLock::new();
    BIN.get_or_init(|| {
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            let path = dir.join("granary");
            if path.exists() {
                return path;
            }
        }

        let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
        for profile in ["debug", "release"] {
            let path = target_dir.join(profile).join("granary");
            if path.exists() {
                return path;
            }
        }

        panic!(
            "granary binary not found. Run 'cargo build' first. Searched in: {:?}",
            target_dir
        );
    })
}

fn run(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(granary_bin())
        .args(args)
        .env("HOME", home)
        .current_dir(cwd)
        .output()
        .expect("failed to execute granary")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn mkdirp(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

// --- Global agent injection tests ---

#[test]
fn test_first_run_injects_into_global_dirs() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();

    mkdirp(&home.join(".claude"));
    mkdirp(&home.join(".codex"));
    let workspace = home.join("workspace");
    mkdirp(&workspace);

    assert_success(&run(home, &workspace, &["init"]));

    let claude_md = home.join(".claude/CLAUDE.md");
    assert!(claude_md.exists());
    assert!(
        fs::read_to_string(&claude_md)
            .unwrap()
            .contains("use granary")
    );

    let codex_md = home.join(".codex/AGENTS.md");
    assert!(codex_md.exists());
    assert!(
        fs::read_to_string(&codex_md)
            .unwrap()
            .contains("use granary")
    );
}

#[test]
fn test_subsequent_run_skips_global_injection() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();

    mkdirp(&home.join(".granary"));
    mkdirp(&home.join(".claude"));
    let workspace = home.join("workspace");
    mkdirp(&workspace);

    assert_success(&run(home, &workspace, &["init"]));
    assert!(!home.join(".claude/CLAUDE.md").exists());
}

#[test]
fn test_first_run_with_no_global_dirs() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let workspace = home.join("workspace");
    mkdirp(&workspace);

    assert_success(&run(home, &workspace, &["init"]));
}

#[test]
fn test_first_run_injects_into_existing_files() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();

    let claude_md = home.join(".claude/CLAUDE.md");
    mkdirp(&home.join(".claude"));
    let original = "# My Custom Instructions\n\nDo something important.\n";
    fs::write(&claude_md, original).unwrap();

    let workspace = home.join("workspace");
    mkdirp(&workspace);

    assert_success(&run(home, &workspace, &["init"]));

    let updated = fs::read_to_string(&claude_md).unwrap();
    assert!(updated.contains("# My Custom Instructions"));
    assert!(updated.contains("Do something important"));
    assert!(updated.contains("use granary"));
}

#[test]
fn test_first_run_skips_files_with_existing_instruction() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();

    let claude_md = home.join(".claude/CLAUDE.md");
    mkdirp(&home.join(".claude"));
    let existing = "# Instructions\n\nUse granary to plan your work.\n";
    fs::write(&claude_md, existing).unwrap();

    let workspace = home.join("workspace");
    mkdirp(&workspace);

    assert_success(&run(home, &workspace, &["init"]));
    assert_eq!(fs::read_to_string(&claude_md).unwrap(), existing);
}

// --- Workspace resolution tests ---

#[test]
fn test_init_creates_workspace_in_cwd_not_ancestor() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();

    mkdirp(&home.join(".granary"));
    let project = home.join("Code/my-project");
    mkdirp(&project);

    assert_success(&run(home, &project, &["init"]));

    assert!(project.join(".granary").is_dir());
    assert!(project.join(".granary/granary.db").exists());
}

#[test]
fn test_init_idempotent() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let workspace = home.join("workspace");
    mkdirp(&workspace);

    for _ in 0..2 {
        assert_success(&run(home, &workspace, &["init"]));
    }
    assert!(workspace.join(".granary").is_dir());
}

#[test]
fn test_commands_find_workspace_in_parent_directory() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let project = home.join("project");
    mkdirp(&project);

    assert_success(&run(home, &project, &["init"]));

    let nested = project.join("src/deep/nested");
    mkdirp(&nested);

    assert_success(&run(home, &nested, &["tasks"]));
}

#[test]
fn test_init_in_subdirectory_creates_independent_workspace() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let parent = home.join("parent");
    let child = parent.join("child");
    mkdirp(&child);

    for dir in [&parent, &child] {
        assert_success(&run(home, dir, &["init"]));
    }

    assert!(parent.join(".granary").is_dir());
    assert!(child.join(".granary").is_dir());

    let output = run(home, &child, &["doctor"]);
    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&child.display().to_string()),
        "doctor should use child workspace, got: {stdout}",
    );
}
