//! Fail-closed, dependency-free structural status checker for the WebSocket demo.
//! This intentionally never invokes an Oreslang compiler or guest.
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
enum Value {
    Text(String),
    Flag(bool),
    Number(u64),
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self { bytes: text.as_bytes(), pos: 0 }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        self.skip_ws();
        if self.peek() != Some(byte) {
            return Err(format!("expected JSON byte 0x{byte:02x} at {}", self.pos));
        }
        self.pos += 1;
        Ok(())
    }

    // Intentionally narrower than general JSON: state metadata must use
    // canonical, unescaped UTF-8 strings and a flat object only.
    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let start = self.pos;
        loop {
            match self.peek() {
                Some(b'"') => {
                    let value = std::str::from_utf8(&self.bytes[start..self.pos])
                        .map_err(|_| "invalid UTF-8 JSON string".to_string())?
                        .to_owned();
                    self.pos += 1;
                    return Ok(value);
                }
                Some(b'\\') | Some(0..=31) => {
                    return Err("escaped/control JSON strings are not supported in status metadata".into());
                }
                None => return Err("unterminated JSON string".into()),
                Some(_) => self.pos += 1,
            }
        }
    }

    fn keyword(&mut self, literal: &[u8]) -> Result<(), String> {
        if self.bytes.get(self.pos..self.pos + literal.len()) != Some(literal) {
            return Err("invalid JSON literal".into());
        }
        self.pos += literal.len();
        Ok(())
    }

    fn value(&mut self) -> Result<Value, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'"') => Ok(Value::Text(self.string()?)),
            Some(b't') => {
                self.keyword(b"true")?;
                Ok(Value::Flag(true))
            }
            Some(b'f') => {
                self.keyword(b"false")?;
                Ok(Value::Flag(false))
            }
            Some(b'0'..=b'9') => {
                let start = self.pos;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
                let digits = std::str::from_utf8(&self.bytes[start..self.pos])
                    .map_err(|_| "invalid JSON number".to_string())?;
                if digits.len() > 1 && digits.starts_with('0') {
                    return Err("non-canonical JSON integer".into());
                }
                Ok(Value::Number(digits.parse::<u64>()
                    .map_err(|_| "JSON integer overflow".to_string())?))
            }
            _ => Err("only flat strings, booleans, and unsigned integers are allowed".into()),
        }
    }
}

fn parse_status(text: &str) -> Result<BTreeMap<String, Value>, String> {
    if text.is_empty() || text.len() > 16 * 1024 {
        return Err("demo-status.json must be 1..16384 bytes".into());
    }
    let mut p = Parser::new(text);
    let mut fields = BTreeMap::new();
    p.expect(b'{')?;
    p.skip_ws();
    if p.peek() != Some(b'}') {
        loop {
            let key = p.string()?;
            p.expect(b':')?;
            let value = p.value()?;
            if fields.insert(key.clone(), value).is_some() {
                return Err(format!("duplicate status key: {key}"));
            }
            p.skip_ws();
            if p.peek() != Some(b',') {
                break;
            }
            p.pos += 1;
        }
    }
    p.expect(b'}')?;
    p.skip_ws();
    if p.pos != p.bytes.len() {
        return Err("trailing data after demo-status.json".into());
    }
    Ok(fields)
}

const REQUIRED: [&str; 9] = [
    "schema_version", "kind", "status", "native_guest_present",
    "compiler_execution_verified", "exact_head_ci_verified",
    "network_listener_verified", "frame_protocol_verified",
    "first_executable_milestone",
];

fn validate_claims(fields: &BTreeMap<String, Value>, guest_count: usize) -> Result<(), String> {
    if fields.len() != REQUIRED.len() ||
        REQUIRED.iter().any(|key| !fields.contains_key(*key)) {
        return Err("status metadata has missing or unexpected keys".into());
    }
    if fields.get("schema_version") != Some(&Value::Number(1)) ||
        fields.get("kind") != Some(&Value::Text("websocket-server".into())) {
        return Err("unsupported demo status schema or kind".into());
    }
    let milestone = match fields.get("first_executable_milestone") {
        Some(Value::Text(s)) if !s.trim().is_empty() => s,
        _ => return Err("first executable milestone must be nonempty".into()),
    };
    let _ = milestone;
    let flag = |name: &str| -> Result<bool, String> {
        match fields.get(name) {
            Some(Value::Flag(value)) => Ok(*value),
            _ => Err(format!("status key {name} must be boolean")),
        }
    };
    let has_guest = flag("native_guest_present")?;
    let compiler = flag("compiler_execution_verified")?;
    let ci = flag("exact_head_ci_verified")?;
    let listener = flag("network_listener_verified")?;
    let frames = flag("frame_protocol_verified")?;
    if has_guest != (guest_count > 0) {
        return Err("native guest claim disagrees with tracked .ores source".into());
    }
    if compiler && !has_guest || ci && !compiler {
        return Err("compiler and exact-head CI claim dependency violated".into());
    }
    if compiler || ci || listener || frames {
        return Err("unverified WebSocket compilation, CI or wire claims are forbidden".into());
    }
    match fields.get("status") {
        Some(Value::Text(status)) if status == "bootstrap" && !has_guest => Ok(()),
        Some(Value::Text(status)) if status == "source-authored" && has_guest
            && guest_count >= 2 => Ok(()),
        _ => Err("status requires an exact matching unexecuted source maturity tier".into()),
    }
}

fn tracked_guest_count(root: &Path) -> Result<usize, String> {
    let listing = Command::new("git")
        .args(["ls-files", "--cached", "-z"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cannot list Git-index files: {e}"))?;
    if !listing.status.success() {
        return Err("git ls-files failed".into());
    }
    let mut count = 0usize;
    for raw in listing.stdout.split(|b| *b == 0).filter(|entry| !entry.is_empty()) {
        let path = std::str::from_utf8(raw)
            .map_err(|_| "non-UTF-8 tracked repository path".to_string())?;
        let metadata = fs::symlink_metadata(root.join(path))
            .map_err(|e| format!("tracked path inaccessible ({path}): {e}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("tracked symlink not accepted: {path}"));
        }
        if path.ends_with(".ores") {
            count += 1;
        }
    }
    Ok(count)
}

fn run() -> Result<usize, String> {
    let root = env::current_dir().map_err(|e| e.to_string())?;
    let status = fs::read_to_string(root.join("demo-status.json"))
        .map_err(|e| format!("cannot read demo-status.json: {e}"))?;
    let fields = parse_status(&status)?;
    let guest_count = tracked_guest_count(&root)?;
    validate_claims(&fields, guest_count)?;
    Ok(guest_count)
}

fn main() {
    match run() {
        Ok(count) => {
            println!("BOOTSTRAP_STATUS_GATE_PASS");
            println!("native_guest_files={count}");
            println!("NOT_EXECUTED: structural claim consistency, no Oreslang guest was run");
        }
        Err(reason) => {
            eprintln!("BOOTSTRAP_STATUS_GATE_FAILURE: {reason}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> &'static str {
        r#"{"schema_version":1,"kind":"websocket-server","status":"bootstrap","native_guest_present":false,"compiler_execution_verified":false,"exact_head_ci_verified":false,"network_listener_verified":false,"frame_protocol_verified":false,"first_executable_milestone":"native echo"}"#
    }

    #[test]
    fn accepts_current_bootstrap_claim() {
        validate_claims(&parse_status(valid()).unwrap(), 0).unwrap();
    }

    #[test]
    fn source_authored_requires_tracked_ores_but_cannot_claim_execution() {
        let authored = valid()
            .replace(r#""status":"bootstrap""#, r#""status":"source-authored""#)
            .replace(r#""native_guest_present":false"#,
                     r#""native_guest_present":true"#);
        assert!(validate_claims(&parse_status(&authored).unwrap(), 2).is_ok());
        assert!(validate_claims(&parse_status(&authored).unwrap(), 0).is_err());
        assert!(validate_claims(&parse_status(&authored).unwrap(), 1).is_err());
        let promoted = authored.replace(
            r#""compiler_execution_verified":false"#,
            r#""compiler_execution_verified":true"#,
        );
        assert!(validate_claims(&parse_status(&promoted).unwrap(), 2).is_err());
        let fake_wire = authored.replace(
            r#""network_listener_verified":false"#,
            r#""network_listener_verified":true"#,
        );
        assert!(validate_claims(&parse_status(&fake_wire).unwrap(), 2).is_err());
        let fake_ci = authored.replace(
            r#""exact_head_ci_verified":false"#,
            r#""exact_head_ci_verified":true"#,
        );
        assert!(validate_claims(&parse_status(&fake_ci).unwrap(), 2).is_err());
    }

    #[test]
    fn denies_duplicate_keys_and_unknown_keys() {
        assert!(parse_status(r#"{"status":"bootstrap","status":"promoted"}"#).is_err());
        let unexpected = valid().replace(
            r#""kind":"websocket-server""#,
            r#""kind":"websocket-server","surprise":true"#,
        );
        assert!(validate_claims(&parse_status(&unexpected).unwrap(), 0).is_err());
    }

    #[test]
    fn denies_promoted_claims_and_missing_guest_source() {
        let promoted = valid().replace(r#""status":"bootstrap""#, r#""status":"ready""#);
        assert!(validate_claims(&parse_status(&promoted).unwrap(), 0).is_err());
        let executed = valid().replace(
            r#""compiler_execution_verified":false"#,
            r#""compiler_execution_verified":true"#,
        );
        assert!(validate_claims(&parse_status(&executed).unwrap(), 0).is_err());
        assert!(validate_claims(&parse_status(valid()).unwrap(), 1).is_err());
    }

    #[test]
    fn denies_wrong_types_escape_sequences_and_trailing_data() {
        let wrong_type = valid().replace(
            r#""native_guest_present":false"#,
            r#""native_guest_present":"false""#,
        );
        assert!(validate_claims(&parse_status(&wrong_type).unwrap(), 0).is_err());
        assert!(parse_status(r#"{"status":"boot\u0073trap"}"#).is_err());
        assert!(parse_status(&format!("{} garbage", valid())).is_err());
    }
}
