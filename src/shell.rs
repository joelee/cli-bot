use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use crate::config::ExecutionConfig;

pub struct ExecutionResult {
    pub duration: Duration,
    pub exit_status: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

pub fn execute(
    command: &str,
    config: &ExecutionConfig,
    capture_output: bool,
    max_output_bytes: usize,
) -> Result<ExecutionResult> {
    let start = Instant::now();
    if capture_output {
        let output = Command::new(&config.shell)
            .arg(&config.shell_arg)
            .arg(command)
            .output()
            .with_context(|| format!("failed to launch shell `{}`", config.shell))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !stdout.is_empty() {
            print!("{stdout}");
        }
        if !stderr.is_empty() {
            eprint!("{stderr}");
        }

        if !output.status.success() {
            bail!("command exited with status {}", output.status)
        }

        return Ok(ExecutionResult {
            duration: start.elapsed(),
            exit_status: output.status.code(),
            stdout: Some(truncate_output(stdout, max_output_bytes)),
            stderr: Some(truncate_output(stderr, max_output_bytes)),
        });
    }

    let status = Command::new(&config.shell)
        .arg(&config.shell_arg)
        .arg(command)
        .status()
        .with_context(|| format!("failed to launch shell `{}`", config.shell))?;

    if !status.success() {
        bail!("command exited with status {status}")
    }

    Ok(ExecutionResult {
        duration: start.elapsed(),
        exit_status: status.code(),
        stdout: None,
        stderr: None,
    })
}

fn truncate_output(output: String, max_output_bytes: usize) -> String {
    let trimmed = output.trim().to_string();
    if max_output_bytes == 0 || trimmed.len() <= max_output_bytes {
        return trimmed;
    }

    let mut end = 0;
    for (index, _) in trimmed.char_indices() {
        if index > max_output_bytes {
            break;
        }
        end = index;
    }

    if end == 0 {
        return "[truncated]".to_string();
    }

    format!("{} [truncated]", &trimmed[..end])
}

#[cfg(test)]
mod tests {
    use super::truncate_output;

    #[test]
    fn truncates_large_output() {
        assert_eq!(truncate_output("abcdef".into(), 4), "abcd [truncated]");
    }

    #[test]
    fn keeps_small_output() {
        assert_eq!(truncate_output("abc".into(), 4), "abc");
    }
}
