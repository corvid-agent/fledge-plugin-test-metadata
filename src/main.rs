#[link(wasm_import_module = "fledge")]
extern "C" {
    fn recv(ptr: *mut u8, max_len: i32) -> i32;
    fn send(ptr: *const u8, len: i32);
    fn exit(code: i32);
    fn metadata(ptr: *const u8, len: i32) -> i32;
}

static mut PASS: u32 = 0;
static mut FAIL: u32 = 0;

fn fledge_recv() -> Vec<u8> {
    let mut buf = vec![0u8; 65536];
    let len = unsafe { recv(buf.as_mut_ptr(), buf.len() as i32) };
    buf.truncate(len.max(0) as usize);
    buf
}

fn fledge_send(msg: &str) {
    unsafe { send(msg.as_ptr(), msg.len() as i32) };
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn output(text: &str) {
    fledge_send(&format!(
        r#"{{"type":"output","text":"{}"}}"#,
        json_escape(text)
    ));
}

fn pass(msg: &str) {
    unsafe { PASS += 1 };
    output(&format!("  \u{2713} PASS: {msg}\n"));
}

fn fail(msg: &str) {
    unsafe { FAIL += 1 };
    output(&format!("  \u{2717} FAIL: {msg}\n"));
}

fn header(title: &str) {
    output(&format!("\n=== {title} ===\n"));
}

fn fledge_metadata(keys_json: &str) -> String {
    let _resp_len = unsafe { metadata(keys_json.as_ptr(), keys_json.len() as i32) };
    let resp = fledge_recv();
    String::from_utf8_lossy(&resp).to_string()
}

fn test_fledge_config() {
    header("FLEDGE CONFIG");
    let resp = fledge_metadata(r#"["fledge_config"]"#);
    if resp.contains("fledge_config") {
        pass("fledge_config key present in response");
        if resp.len() > 20 {
            pass(&format!("fledge_config has content ({} bytes)", resp.len()));
        } else {
            output(&format!("  (response: {resp})\n"));
        }
    } else {
        fail(&format!("missing fledge_config in response: {resp}"));
    }
}

fn test_git_status() {
    header("GIT STATUS");
    let resp = fledge_metadata(r#"["git_status"]"#);
    if resp.contains("git_status") {
        pass("git_status key present");
    } else {
        fail(&format!("missing git_status: {resp}"));
    }
}

fn test_git_tags() {
    header("GIT TAGS");
    let resp = fledge_metadata(r#"["git_tags"]"#);
    if resp.contains("git_tags") {
        pass("git_tags key present");
    } else {
        fail(&format!("missing git_tags: {resp}"));
    }
}

fn test_git_log() {
    header("GIT LOG");
    let resp = fledge_metadata(r#"["git_log"]"#);
    if resp.contains("git_log") {
        pass("git_log key present");
    } else {
        fail(&format!("missing git_log: {resp}"));
    }
}

fn test_env() {
    header("ENVIRONMENT (FILTERED)");
    let resp = fledge_metadata(r#"["env"]"#);
    if resp.contains("env") {
        pass("env key present");
        // Should NOT contain sensitive vars (they're filtered)
        if resp.contains("GITHUB_TOKEN")
            || resp.contains("GH_TOKEN")
            || resp.contains("ANTHROPIC_API_KEY")
        {
            fail("env contains sensitive tokens (should be filtered)");
        } else {
            pass("no sensitive tokens in env response");
        }
    } else {
        fail(&format!("missing env: {resp}"));
    }
}

fn test_multiple_keys() {
    header("MULTIPLE KEYS IN ONE REQUEST");
    let resp = fledge_metadata(r#"["fledge_config","git_status","git_tags","git_log"]"#);
    let mut found = 0;
    for key in &["fledge_config", "git_status", "git_tags", "git_log"] {
        if resp.contains(key) {
            found += 1;
        } else {
            fail(&format!("missing '{key}' in multi-key response"));
        }
    }
    if found == 4 {
        pass("all 4 keys present in single request");
    }
}

fn test_unknown_key() {
    header("UNKNOWN KEY");
    let resp = fledge_metadata(r#"["nonexistent_key_xyz"]"#);
    // Should return the key with null or empty value, not crash
    if !resp.is_empty() {
        pass("unknown key handled gracefully (no crash)");
    } else {
        fail("empty response for unknown key");
    }
}

fn test_empty_request() {
    header("EMPTY KEY LIST");
    let resp = fledge_metadata(r#"[]"#);
    if !resp.is_empty() {
        pass("empty key list returns response (no crash)");
    } else {
        fail("empty response for empty key list");
    }
}

fn test_negative_no_filesystem() {
    header("NEGATIVE — OTHER CAPABILITIES BLOCKED");
    match std::fs::read_to_string("/project/Cargo.toml") {
        Ok(_) => fail("filesystem accessible without capability"),
        Err(_) => pass("filesystem blocked (no capability granted)"),
    }
}

fn test_negative_no_network() {
    match std::net::TcpStream::connect("8.8.8.8:53") {
        Ok(_) => fail("network accessible without capability"),
        Err(_) => pass("network blocked (no capability granted)"),
    }
}

fn test_negative_no_process_spawn() {
    match std::process::Command::new("echo").arg("test").output() {
        Ok(_) => fail("process spawn succeeded"),
        Err(_) => pass("process spawn blocked (WASI p1)"),
    }
}

fn main() {
    let _init = fledge_recv();

    output("fledge-plugin-test-metadata v0.1.0\n");
    output("Capability: metadata=true (all others denied)\n");
    output("Tests that WASM plugins can query project metadata via fledge::metadata\n");

    test_fledge_config();
    test_git_status();
    test_git_tags();
    test_git_log();
    test_env();
    test_multiple_keys();
    test_unknown_key();
    test_empty_request();
    test_negative_no_filesystem();
    test_negative_no_network();
    test_negative_no_process_spawn();

    let (p, f) = unsafe { (PASS, FAIL) };
    let total = p + f;
    header("SUMMARY");
    output(&format!("  {total} tests: {p} passed, {f} failed\n\n"));

    if f == 0 {
        output("  RESULT: metadata capability works correctly.\n\n");
    } else {
        output(&format!("  WARNING: {f} test(s) failed!\n\n"));
    }

    unsafe { exit(if f == 0 { 0 } else { 1 }) };
    unreachable!();
}
