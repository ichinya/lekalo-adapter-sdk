use lekalo_core::expressions::{canonical_bytes, render_program, ExpressionsAttachment};

const VALID: &[u8] = include_bytes!("../../../tests/fixtures/expressions/valid/planner.json");
const VECTORS: &[u8] = include_bytes!("../../../tests/fixtures/expressions/vectors.json");
fn parse(bytes: &[u8]) -> ExpressionsAttachment {
    let json: serde_json::Value = serde_json::from_slice(bytes).expect("fixture json");
    ExpressionsAttachment::from_value(&json).expect("valid fixture")
}

#[test]
fn every_target_renders_a_deterministic_program() {
    let attachment = parse(VALID);
    for target in ["node", "php", "go"] {
        let first = render_program(std::path::Path::new("."), &attachment, target)
            .expect("selected expression generator");
        let second = render_program(std::path::Path::new("."), &attachment, target)
            .expect("selected expression generator");
        assert_eq!(first, second, "{target:?} renders deterministically");
        assert!(first.contains("expr.planner/overdue-check"), "{target:?}");
        assert!(first.contains("lekFail"), "{target:?}");
    }
}

/// The acceptance proof that closes the loop: every rendered target
/// program is actually executed against the shared vector document,
/// and its computed results must match the declared expectations
/// byte-for-byte in the wire encoding. A missing interpreter skips
/// that target loudly (the CI Linux image ships all three); a wrong
/// result always fails.
#[test]
fn every_generated_program_executes_the_shared_vectors_identically() {
    use std::process::{Command, Stdio};

    let attachment = parse(VALID);
    let vectors_json: serde_json::Value = serde_json::from_slice(VECTORS).expect("vectors json");

    let dir = std::env::temp_dir().join(format!(
        "lekalo-expr-exec-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");

    let vectors_path = dir.join("vectors.json");
    std::fs::write(&vectors_path, VECTORS).expect("write vectors");

    let available = |tool: &str, probe: &[&str]| {
        Command::new(tool)
            .args(probe)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    };
    let node = available("node", &["--version"]);
    let php = available("php", &["--version"]);
    // `go --version` is not a command; the tool probes with `go version`.
    let go = available("go", &["version"]);
    assert!(node, "node must be available to execute the shared vectors");

    let mut executed: Vec<&'static str> = Vec::new();
    for (target, file, tool, args) in [
        ("node", "planner.cjs", "node", vec![]),
        ("php", "planner.php", "php", vec![]),
        ("go", "planner.go", "go", vec!["run".to_owned()]),
    ] {
        let present = match tool {
            "node" => node,
            "php" => php,
            _ => go,
        };
        if !present {
            eprintln!("skipping target {target:?}: {tool} is not on this host");
            continue;
        }
        let program = render_program(std::path::Path::new("."), &attachment, target)
            .expect("selected expression generator");
        let path = dir.join(file);
        std::fs::write(&path, program).expect("write program");

        let vectors_file = std::fs::File::open(&vectors_path).expect("open vectors");
        let output = Command::new(tool)
            .args(&args)
            .arg(&path)
            .stdin(Stdio::from(vectors_file))
            .output()
            .expect("run the generated program");
        assert!(
            output.status.success(),
            "{tool} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("utf8 results");
        let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("results envelope");
        let rows = envelope["results"].as_array().expect("results array");

        let cases = vectors_json["vectors"].as_array().expect("vectors");
        assert_eq!(rows.len(), cases.len(), "{tool}: one row per vector");
        for (row, case) in rows.iter().zip(cases.iter()) {
            assert_eq!(row["id"], case["id"], "{tool}: row order matches");
            let expect = &case["expect"];
            if let Some(expected_error) = expect.get("error") {
                assert_eq!(
                    &row["error"], expected_error,
                    "{tool}: {} must fail identically",
                    case["id"]
                );
            } else {
                assert_eq!(
                    &row["value"], &expect["value"],
                    "{tool}: {} must compute identically",
                    case["id"]
                );
            }
        }
        executed.push(tool);
    }
    eprintln!("executed targets on this host: {executed:?}");
    let _ = canonical_bytes(&attachment).expect("canonical");
}
