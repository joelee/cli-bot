use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use crate::config::ExecutionConfig;

pub fn execute(command: &str, config: &ExecutionConfig) -> Result<Duration> {
    let start = Instant::now();
    let status = Command::new(&config.shell)
        .arg(&config.shell_arg)
        .arg(command)
        .status()
        .with_context(|| format!("failed to launch shell `{}`", config.shell))?;

    if !status.success() {
        bail!("command exited with status {status}")
    }

    Ok(start.elapsed())
}
