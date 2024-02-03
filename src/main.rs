use anyhow::{Context, Result};

mod registry;
mod sandbox;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let (image, command, command_args) = parse_args()?;

    let layers = registry::pull_image(&image)?;

    sandbox::create(layers)?;

    let output = std::process::Command::new(&command)
        .args(&command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                &command, &command_args
            )
        })?;

    let std_out = std::str::from_utf8(&output.stdout)?;
    print!("{}", std_out);
    let std_err = std::str::from_utf8(&output.stderr)?;
    eprint!("{}", std_err);
    std::process::exit(output.status.code().unwrap_or(1));
}

fn parse_args() -> Result<(String, String, Vec<String>)> {
    let mut args = std::env::args();
    let _ = args.next();
    let _ = args.next();

    Ok((
        args.next().context("Missing image argument")?,
        args.next().context("Missing command argument")?,
        args.collect::<Vec<_>>(),
    ))
}
