use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn run_aurora(args: &[&str], input: &str, cwd: &Path) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_aurora"))
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn aurora binary");

    if !input.is_empty() {
        let mut stdin = child.stdin.take().expect("stdin should be piped");
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write stdin");
    }

    child.wait_with_output().expect("failed to wait for aurora")
}

#[test]
fn exits_with_unified_error_when_workspace_is_invalid() {
    let cwd = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = run_aurora(&["./definitely-missing-workspace"], "", cwd);

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("错误：workspace not found:"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn clears_session_and_exits_through_cli_commands() {
    let cwd = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = run_aurora(&["."], "/clear\nquit\n", cwd);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("A U R O R A"),
        "unexpected stdout: {stdout}"
    );
    assert!(stdout.contains("助手> 已清空当前会话。"));
    assert!(stdout.contains("助手> 下次见。"));
}

#[test]
fn local_slash_commands_do_not_print_thinking_line() {
    let cwd = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = run_aurora(
        &["."],
        "我是谁？\n你现在的模型是谁\n我\n/model\n你是谁啊？\nquit\n",
        cwd,
    );

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Provider:"), "unexpected stdout: {stdout}");
    assert!(stdout.contains("Model:"), "unexpected stdout: {stdout}");
    assert!(
        !stdout.contains("正在思考"),
        "local commands should not show thinking: {stdout}"
    );
}
