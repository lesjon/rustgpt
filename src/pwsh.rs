use std::process::{Child, Command, Stdio};

use std::io::{Read, Write};

pub fn get_instance() -> std::io::Result<Child> {
    Command::new("powershell.exe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub fn run_command(child: &mut Child, command: &str) -> String {
    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(command.as_bytes()).expect("failed to write to stdin");

    let mut output = String::new();
    if let Some(stdout) = &mut child.stdout {
        stdout.read_to_string(&mut output).expect("failed to read stdout");
    }
    let mut error = String::new();
    if let Some(stderr) = &mut child.stderr {
        stderr.read_to_string(&mut error).expect("failed to read stderr");
    }

    format!("{}\n{}", output, error)
}