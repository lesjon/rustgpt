use std::error::Error;
use std::process::{Child, Command, Stdio};
use std::str;

use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

pub fn get_instance() -> std::io::Result<Child> {
    Command::new("powershell.exe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub fn run_command(command: &str) -> String {
    let cmd = Command::new("powershell.exe")
        .arg(command)
        .output()
        .expect("failed to execute command");
    String::from_utf8(cmd.stdout).expect("Could not parse string from command output")
}
pub fn run_command_on_child(child: &mut Child, command: &str) -> String {
    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(command.as_bytes()).expect("failed to write to stdin");
    child_stdin.write_all("\n".as_bytes()).expect("failed to write newline to stdin");

    let mut output = String::new();
    if let Some(stdout) = &mut child.stdout {
        if let Ok(stdout) = read(stdout) {
            output = stdout;
        }
    }
    let mut error = String::new();
    if let Some(stderr) = &mut child.stderr {
        if let Ok(stderr) = read(stderr) {
            error = stderr;
        }
    }

    format!("{}\n{}", output, error)
}

fn read(stream: &mut dyn Read) -> Result<String, Box<dyn Error>> {
    let mut wait_max_ms = 100;  // wait at most N ms
    let mut output = String::new();
    let mut buffer = Vec::with_capacity(1024);
    'WAIT_FOR_OUTPUT_LOOP: loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                thread::sleep(Duration::from_millis(1));
                wait_max_ms -= 1;
                if wait_max_ms < 0 {
                    eprintln!("maximum wait time passed without ever getting output from the stream '{:?}'", buffer);
                    break 'WAIT_FOR_OUTPUT_LOOP;
                }
            }
            Ok(s) => {
                output.push_str(str::from_utf8(&buffer[..s])?);
                break 'WAIT_FOR_OUTPUT_LOOP;
            }
            Err(e) => {
                eprintln!("error ({}), while reading from stream after having read:'{:?}'", e, buffer);
                break 'WAIT_FOR_OUTPUT_LOOP;
            }
        }
    }
    'WAIT_FOR_END_LOOP: loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                break 'WAIT_FOR_END_LOOP;
            }
            Ok(s) => {
                output.push_str(str::from_utf8(&buffer[..s])?);
            }
            Err(e) => {
                eprintln!("error ({}), while reading from stream after having read:'{:?}'", e, buffer);
                break 'WAIT_FOR_END_LOOP;
            }
        }
    }
    Ok(output)
}