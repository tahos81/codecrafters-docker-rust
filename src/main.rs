use anyhow::{Context, Result};
use std::env;
use std::process::{self, exit, Stdio};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.

    let args: Vec<_> = env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let mut child = process::Command::new(command)
        .args(command_args)
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let exit_code = child.wait()?.code();

    if let Some(code) = exit_code {
        exit(code);
    }

    Ok(())
}
