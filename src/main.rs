use anyhow::{Context, Result};
use libc::chroot;
use tempfile::TempDir;

use std::{
    env,
    ffi::CString,
    fs,
    os::unix::prelude::OsStrExt,
    path::Path,
    process::{self, exit, Stdio},
};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let temp_dir = TempDir::new()?;
    fs::create_dir_all(temp_dir.path().join("usr/local/bin/"))?;
    fs::create_dir_all(temp_dir.path().join("dev/null"))?;

    let source_path = Path::new("/usr/local/bin/docker-explorer");
    let dest_path = temp_dir.path().join("usr/local/bin/docker-explorer");
    fs::copy(source_path, &dest_path)?;

    let c_path = CString::new(temp_dir.path().as_os_str().as_bytes()).unwrap();
    unsafe { chroot(c_path.as_ptr()) };

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
