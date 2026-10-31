// Dependency-free, fail-closed *structural* gate. Does not compile or run Oreslang.
// Intentionally pinned to the bootstrap manifest; promotion must replace this gate.
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

const BOOTSTRAP_MANIFEST: &str = concat!(
    "{\n",
    "  \"schema_version\": 1,\n",
    "  \"kind\": \"websocket-server\",\n",
    "  \"status\": \"bootstrap\",\n",
    "  \"native_guest_present\": false,\n",
    "  \"compiler_execution_verified\": false,\n",
    "  \"exact_head_ci_verified\": false,\n",
    "  \"network_listener_verified\": false,\n",
    "  \"frame_protocol_verified\": false,\n",
    "  \"first_executable_milestone\": \"native echo actor plus bounded frame adapter and localhost handshake/fragmentation/negative tests\"\n",
    "}\n",
);

fn valid_sha(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn forbidden_path(path: &str) -> bool {
    let p = path.to_ascii_lowercase();
    p.ends_with(".ores") || p.ends_with(".py") || p.ends_with(".pyi")
        || p.ends_with(".ipynb") || p.ends_with("/pom.xml") || p == "pom.xml"
        || p.ends_with("/cargo.toml") || p == "cargo.toml"
        || p.ends_with("/package.json") || p == "package.json"
        || p.ends_with("/mix.exs") || p == "mix.exs"
        || p.ends_with("/go.mod") || p == "go.mod"
}

fn forbidden_trigger(text: &str) -> bool {
    text.lines().any(|line| {
        let s = line.trim_start();
        ["pull_request_target:", "workflow_run:", "repository_dispatch:", "workflow_call:"]
            .iter()
            .any(|event| s.starts_with(event))
    })
}

fn checked_git(args: &[&str]) -> Result<Vec<u8>, Box<dyn Error>> {
    let output = Command::new("git").args(args).output()?;
    if !output.status.success() {
        return Err(format!("git {:?} failed", args).into());
    }
    Ok(output.stdout)
}

fn check() -> Result<usize, Box<dyn Error>> {
    let status = checked_git(&["status", "--porcelain", "--untracked-files=all"])?;
    if !status.is_empty() {
        return Err("working tree is not clean; inspect staged and untracked content".into());
    }
    if let Ok(expected) = env::var("EXPECTED_DEMO_SHA") {
        if !valid_sha(&expected) {
            return Err("EXPECTED_DEMO_SHA must be a lowercase 40-hex commit".into());
        }
        let head = String::from_utf8(checked_git(&["rev-parse", "HEAD"])?)?;
        if head.trim() != expected {
            return Err("checked-out commit differs from expected PR/push head".into());
        }
    }
    if fs::read_to_string("demo-status.json")? != BOOTSTRAP_MANIFEST {
        return Err("bootstrap manifest changed: explicit promotion and replacement of this gate required".into());
    }
    let readme = fs::read_to_string("README.md")?;
    if !readme.contains("**Status: empty-demo bootstrap.**")
        || !readme.contains("No actual WebSocket listener")
    {
        return Err("README must retain an accurate empty-demo bootstrap declaration".into());
    }
    if !Path::new("docs/DEMO_RELEASE_GATES.md").is_file() {
        return Err("release gate documentation is required".into());
    }

    let index = checked_git(&["ls-files", "--stage", "-z"])?;
    let mut count = 0usize;
    for item in index.split(|byte| *byte == 0).filter(|part| !part.is_empty()) {
        count += 1;
        if count > 50_000 {
            return Err("tracked-file budget exceeded".into());
        }
        let record = String::from_utf8(item.to_vec())?;
        let (metadata, path) = record.split_once('\t').ok_or("invalid Git index entry")?;
        let mode = metadata.split_whitespace().next().ok_or("missing Git mode")?;
        if !matches!(mode, "100644" | "100755") {
            return Err(format!("symlink/submodule/non-file entry is forbidden: {path}").into());
        }
        if forbidden_path(path) {
            return Err(format!("executable source or Python tooling in bootstrap: {path}").into());
        }
        if !Path::new(path).is_file() || Path::new(path).is_symlink() {
            return Err(format!("tracked file is missing or became a symlink: {path}").into());
        }
        let bytes = fs::read(path)?;
        if bytes.starts_with(b"#!") {
            let first_line = bytes.split(|b| *b == b'\n').next().unwrap_or(&[]);
            let shebang = String::from_utf8_lossy(first_line).to_ascii_lowercase();
            if shebang.contains("python") || shebang.contains("pypy") {
                return Err(format!("Python interpreter shebang forbidden: {path}").into());
            }
        }
        if (path.starts_with(".github/workflows/")) && (path.ends_with(".yml") || path.ends_with(".yaml")) {
            let content = String::from_utf8(bytes)?;
            if forbidden_trigger(&content) || content.contains("python3 ") || content.contains("pypy ") {
                return Err(format!("privileged workflow trigger or Python interpreter forbidden: {path}").into());
            }
        }
    }
    if count == 0 {
        return Err("empty index".into());
    }
    Ok(count)
}

fn main() {
    match check() {
        Ok(count) => {
            println!("BOOTSTRAP_STRUCTURAL_PASS tracked_files={count}");
            println!("NOT_EXECUTED: no Oreslang compiler, guest, listener, frame or runtime test ran");
        }
        Err(error) => {
            eprintln!("BOOTSTRAP_POLICY_FAILURE: {error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_manifest_keeps_all_evidence_false() {
        assert!(BOOTSTRAP_MANIFEST.contains("\"status\": \"bootstrap\""));
        for field in ["native_guest_present", "compiler_execution_verified", "exact_head_ci_verified", "network_listener_verified", "frame_protocol_verified"] {
            assert!(BOOTSTRAP_MANIFEST.contains(&format!("\"{field}\": false")));
        }
        assert_ne!(BOOTSTRAP_MANIFEST.replace("false", "true"), BOOTSTRAP_MANIFEST);
    }

    #[test]
    fn python_and_case_variant_guest_source_denied() {
        for p in ["src/hello.ores", "src/HELLO.ORES", "tests/tool.PY", "tool.PYI", "a.ipynb", "pom.xml", "demo/Cargo.toml", "go.mod"] {
            assert!(forbidden_path(p), "must deny {p}");
        }
        for p in ["ci/bootstrap_guard.rs", "ci/source-policy.sh", "README.md"] {
            assert!(!forbidden_path(p), "must allow {p}");
        }
    }

    #[test]
    fn sha_must_be_exact() {
        assert!(valid_sha(&"a".repeat(40)));
        for bad in ["a".repeat(39), "A".repeat(40), "g".repeat(40), "a".repeat(41)] {
            assert!(!valid_sha(&bad));
        }
    }

    #[test]
    fn privileged_actions_triggers_are_denied() {
        for event in ["pull_request_target:", "workflow_run:", "repository_dispatch:", "workflow_call:"] {
            assert!(forbidden_trigger(&format!("on:\n  {event}\n")));
        }
        assert!(!forbidden_trigger("on:\n  pull_request:\n  workflow_dispatch:\n"));
    }
}
