use anyhow::Result;
use bytes::Bytes;
use std::{env, fs, os::unix::fs::chroot, path::Path};
use tempfile::TempDir;

pub fn create(layers: Vec<Bytes>) -> Result<()> {
    let sandbox = TempDir::new()?;
    create_dev_null(&sandbox)?;
    copy_docker_explorer(&sandbox)?;
    isolate_filesystem(&sandbox)?;
    extract_layers(layers)?;
    isolate_process();

    Ok(())
}

fn create_dev_null(sandbox: &TempDir) -> Result<()> {
    fs::create_dir_all(sandbox.path().join("dev/null"))?;
    Ok(())
}

fn copy_docker_explorer(sandbox: &TempDir) -> Result<()> {
    fs::create_dir_all(sandbox.path().join("usr/local/bin/"))?;
    let source_path = Path::new("/usr/local/bin/docker-explorer");
    let dest_path = sandbox.path().join("usr/local/bin/docker-explorer");
    fs::copy(source_path, &dest_path)?;
    Ok(())
}

fn isolate_filesystem(sandbox: &TempDir) -> Result<()> {
    chroot(sandbox.path())?;
    env::set_current_dir("/")?;
    Ok(())
}

fn extract_layers(layers: Vec<Bytes>) -> Result<()> {
    for layer in layers {
        let layer = layer.to_vec();
        let layer = flate2::read::GzDecoder::new(&layer[..]);
        tar::Archive::new(layer).unpack("/")?;
    }
    Ok(())
}

fn isolate_process() {
    #[cfg(target_os = "linux")]
    unsafe {
        libc::unshare(libc::CLONE_NEWPID);
    }
}
