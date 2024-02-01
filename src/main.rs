//use std::process::Stdio;
use anyhow::{Context, Result};
use std::{env, fs, os::unix::fs::chroot, path::Path};
use tempfile::TempDir;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let temp_dir = TempDir::new()?;
    fs::create_dir_all(temp_dir.path().join("usr/local/bin/"))?;
    fs::create_dir_all(temp_dir.path().join("dev/null"))?;
    let source_path = Path::new("/usr/local/bin/docker-explorer");
    let dest_path = temp_dir.path().join("usr/local/bin/docker-explorer");
    fs::copy(source_path, &dest_path)?;
    chroot(temp_dir.path())?;
    env::set_current_dir("/")?;

    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let std_out = std::str::from_utf8(&output.stdout)?;
    print!("{}", std_out);
    let std_err = std::str::from_utf8(&output.stderr)?;
    eprint!("{}", std_err);
    std::process::exit(output.status.code().unwrap_or(1));
}
