use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn expressions_root() -> PathBuf {
    alias_free_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../")
            .join("tests/fixtures/expressions"),
    )
}

/// The selection policy denies alias-spelled working directories;
/// chdir the child into the resolved spelling.
fn alias_free_path(path: &Path) -> PathBuf {
    let canonical = path.canonicalize().expect("fixture path must exist");
    #[cfg(windows)]
    match canonical.to_string_lossy().strip_prefix(r"\\?\") {
        Some(stripped) => PathBuf::from(stripped),
        None => canonical,
    }
    #[cfg(not(windows))]
    {
        canonical
    }
}

fn lekalo_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lekalo"))
        .args(args)
        .current_dir(alias_free_path(dir))
        .env_remove("LEKALO_PROJECT")
        .output()
        .expect("run the real lekalo binary")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf8")
}

fn exit_code(output: &Output) -> u8 {
    output.status.code().expect("exit code") as u8
}

fn json_of(output: &Output) -> serde_json::Value {
    serde_json::from_str(&stdout_text(output)).expect("json envelope")
}

#[test]
fn render_emits_complete_programs_for_every_target() {
    let root = expressions_root();
    for (target, marker) in [
        ("node", "const LEK = Object.create(null);"),
        ("php", "<?php"),
        ("go", "package main"),
    ] {
        let output = lekalo_in(
            &root,
            &[
                "expressions",
                "render",
                "valid/planner.json",
                "--target",
                target,
            ],
        );
        assert_eq!(exit_code(&output), 0, "{target}: {}", stdout_text(&output));
        let program = stdout_text(&output);
        assert!(program.contains(marker), "{target} missing {marker}");
        assert!(program.contains("expr.planner/overdue-check"), "{target}");
        // The JSON envelope carries the same program.
        let json_output = lekalo_in(
            &root,
            &[
                "--json",
                "expressions",
                "render",
                "valid/planner.json",
                "--target",
                target,
            ],
        );
        assert_eq!(exit_code(&json_output), 0);
        let document = json_of(&json_output);
        assert_eq!(document["expressionRender"]["target"], target);
        assert_eq!(
            document["expressionRender"]["program"]
                .as_str()
                .map(str::len),
            Some(program.len())
        );
    }
}
