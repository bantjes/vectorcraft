//! `vectorcraft --help` prints the usage and exits before the app starts (#2110): run from a
//! terminal, SSH or an agent's shell it used to open the window (or hang without a display).

use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_vectorcraft");

/// Runs the desktop binary without a display, so a regression fails to open a window (or is
/// stopped at the deadline) instead of printing the usage.
fn run(args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut child = Command::new(BIN)
        .args(args)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("WAYLAND_SOCKET")
        .env("VECTORCRAFT_NO_PREFS", "1")
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Both pipes end when the binary exits, however long a loaded machine takes to start it. The
    // deadline only stops a binary that is still running.
    let deadline = Instant::now() + Duration::from_secs(120);
    let stdout = read_to_end(child.stdout.take().unwrap());
    let stderr = read_to_end(child.stderr.take().unwrap());
    let recv = |rx: &mpsc::Receiver<Vec<u8>>| rx.recv_timeout(deadline.saturating_duration_since(Instant::now()));
    let (Ok(stdout), Ok(stderr)) = (recv(&stdout), recv(&stderr)) else {
        child.kill().unwrap();
        panic!("{args:?}: the app started instead of printing its usage");
    };
    Output { status: child.wait().unwrap(), stdout, stderr }
}

/// Reads `pipe` to its end on another thread.
fn read_to_end(mut pipe: impl Read + Send + 'static) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    rx
}

fn prints_usage(args: &[&str], env: &[(&str, &str)]) {
    let out = run(args, env);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    assert!(out.stderr.is_empty(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    assert!(stdout.starts_with("Usage: vectorcraft "), "{args:?}: {stdout}");
    for flag in ["--in-window-menus", "--version", "--help"] {
        assert!(stdout.contains(flag), "{args:?}: usage doesn't mention {flag}: {stdout}");
    }
}

#[test]
fn help_prints_usage_and_exits_before_startup() {
    prints_usage(&["--help"], &[]);
    prints_usage(&["-h"], &[]);
    // After a file or a flag, too: nothing is opened.
    prints_usage(&["drawing.svg", "--help"], &[]);
    prints_usage(&["--in-window-menus", "-h"], &[]);
}
