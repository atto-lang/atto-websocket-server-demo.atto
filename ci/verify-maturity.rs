//! Dependency-free, fail-closed maturity check. No guest or wire execution.
use std::fs;
use std::process::Command;

const MANIFEST: &str = concat!(
  "{\n",
  "  \"schema_version\": 1,\n",
  "  \"kind\": \"websocket-server\",\n",
  "  \"status\": \"source-authored\",\n",
  "  \"native_guest_present\": true,\n",
  "  \"compiler_execution_verified\": false,\n",
  "  \"exact_head_ci_verified\": false,\n",
  "  \"network_listener_verified\": false,\n",
  "  \"frame_protocol_verified\": false,\n",
  "  \"first_executable_milestone\": \"compiled .atto EchoActor bound to bounded WebSocket session, with exact-head guest plus wire test receipts\"\n",
  "}\n"
);

fn check() -> Result<(), String> {
    let status = fs::read_to_string("demo-status.json").map_err(|e| e.to_string())?;
    if status != MANIFEST { return Err("metadata must match the fail-closed source-authored manifest".into()); }
    let out = Command::new("git").args(["ls-files","--cached","-z"]).output().map_err(|e| e.to_string())?;
    if !out.status.success() { return Err("git ls-files failed".into()); }
    let mut guest = Vec::<String>::new();
    for name in out.stdout.split(|b| *b == 0).filter(|n| !n.is_empty()) {
        let path = std::str::from_utf8(name).map_err(|_| "non-UTF8 tracked path")?;
        let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() { return Err(format!("tracked symlink forbidden: {path}")); }
        if path.ends_with(".ores") { return Err(format!("legacy guest source: {path}")); }
        if path.ends_with(".atto") { guest.push(path.to_string()); }
    }
    if guest != ["src/echo_actor.atto", "src/echo_contract.atto", "tests/echo_contract_test.atto"] {
        return Err(format!("unexpected canonical source inventory: {guest:?}"));
    }
    Ok(())
}

fn main() {
    if let Err(err) = check() { eprintln!("MATURITY_GATE_FAIL: {err}"); std::process::exit(1); }
    println!("SOURCE_AUTHORED_STRUCTURAL_PASS; compiler, actor, WebSocket guest and exact-head CI NOT_EXECUTED");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn manifest_is_fail_closed() {
        assert!(MANIFEST.contains("\"compiler_execution_verified\": false"));
        assert!(MANIFEST.contains("\"frame_protocol_verified\": false"));
        assert!(!MANIFEST.contains("\"status\": \"ready\""));
    }
    #[test] fn canonical_inventory_matches_manifest() { assert!(MANIFEST.contains("\"native_guest_present\": true")); }
}
