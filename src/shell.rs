use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::config::ExecutionConfig;
use crate::error::CliBotError;

pub struct ExecutionResult {
    pub duration: Duration,
    pub exit_status: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl ExecutionResult {
    /// Whether the command reported failure. A non-zero exit is not an
    /// error here: the caller has to record the attempt in session memory
    /// before it can report one, and it needs this result to do that.
    pub fn failed(&self) -> bool {
        self.exit_status != Some(0)
    }

    /// The error a failed command becomes.
    pub fn failure(&self) -> anyhow::Error {
        CliBotError::CommandFailed {
            status: self.exit_status,
        }
        .into()
    }
}

/// Runs `command`, optionally keeping a copy of what it printed.
///
/// Standard input is always the terminal's, so a command that asks for a
/// password or opens an editor still works. When output is captured it is
/// still written out as it arrives, rather than held back until the command
/// ends. A command that drives the terminal itself, such as an editor or a
/// pager, is run without capture: a pipe would leave it with nowhere to
/// draw.
pub fn execute(
    command: &str,
    config: &ExecutionConfig,
    capture_output: bool,
    max_output_bytes: usize,
) -> Result<ExecutionResult> {
    let start = Instant::now();
    let capture = capture_output && !needs_terminal(command, config.preferred_editor.as_deref());
    let pipe = || {
        if capture {
            Stdio::piped()
        } else {
            Stdio::inherit()
        }
    };

    let mut child = Command::new(&config.shell)
        .arg(&config.shell_arg)
        .arg(command)
        .stdin(Stdio::inherit())
        .stdout(pipe())
        .stderr(pipe())
        .spawn()
        .with_context(|| format!("failed to launch shell `{}`", config.shell))?;

    if !capture {
        let status = child
            .wait()
            .with_context(|| format!("failed to run `{command}`"))?;

        return Ok(ExecutionResult {
            duration: start.elapsed(),
            exit_status: status.code(),
            stdout: None,
            stderr: None,
        });
    }

    // One reader per stream, so neither can fill its pipe and stall the
    // command while the other is being drained.
    let out_pipe = child.stdout.take().context("stdout should be piped")?;
    let err_pipe = child.stderr.take().context("stderr should be piped")?;
    let keep = capture_limit(max_output_bytes);
    let out_reader = thread::spawn(move || tee(out_pipe, &mut std::io::stdout(), keep));
    let err_reader = thread::spawn(move || tee(err_pipe, &mut std::io::stderr(), keep));

    let status = child
        .wait()
        .with_context(|| format!("failed to run `{command}`"))?;
    // Joined before the status is reported, so nothing printed is lost.
    let stdout = out_reader.join().unwrap_or_default();
    let stderr = err_reader.join().unwrap_or_default();

    Ok(ExecutionResult {
        duration: start.elapsed(),
        exit_status: status.code(),
        stdout: Some(truncate_output(stdout, max_output_bytes)),
        stderr: Some(truncate_output(stderr, max_output_bytes)),
    })
}

/// Programs that draw on the terminal themselves, and so cannot have their
/// output taken away from them. The configured editor is added to these.
const TERMINAL_PROGRAMS: &[&str] = &[
    "vi", "vim", "nvim", "emacs", "nano", "pico", "helix", "hx", "kak", "micro", "less", "more",
    "most", "top", "htop", "btop", "man", "ssh", "sftp", "telnet", "watch", "tmux", "screen",
    "fzf", "vimdiff", "crontab", "visudo",
];

/// Whether any part of `command` needs the terminal to itself.
fn needs_terminal(command: &str, preferred_editor: Option<&str>) -> bool {
    let editor = preferred_editor
        .and_then(|value| value.split_whitespace().next())
        .map(|value| value.rsplit('/').next().unwrap_or(value));

    crate::safety::program_names(command).iter().any(|program| {
        TERMINAL_PROGRAMS.contains(&program.as_str()) || editor == Some(program.as_str())
    })
}

/// How much of a stream to keep. One byte past the limit is enough for
/// `truncate_output` to tell a full stream from a trimmed one, and keeps a
/// command that prints for ever from filling memory.
fn capture_limit(max_output_bytes: usize) -> usize {
    if max_output_bytes == 0 {
        usize::MAX
    } else {
        max_output_bytes.saturating_add(1)
    }
}

/// Copies `source` to `sink` as it arrives, keeping the first `keep` bytes.
fn tee(mut source: impl Read, sink: &mut impl Write, keep: usize) -> String {
    let mut kept: Vec<u8> = Vec::new();
    let mut chunk = [0_u8; 4096];

    loop {
        match source.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                let _ = sink.write_all(&chunk[..read]).and_then(|()| sink.flush());
                if kept.len() < keep {
                    let room = keep - kept.len();
                    kept.extend_from_slice(&chunk[..read.min(room)]);
                }
            }
        }
    }

    String::from_utf8_lossy(&kept).to_string()
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
    use super::{capture_limit, execute, needs_terminal, tee, truncate_output};
    use crate::config::ExecutionConfig;
    use crate::error::CliBotError;
    #[cfg(target_os = "linux")]
    use std::fs;

    #[test]
    fn truncates_large_output() {
        assert_eq!(truncate_output("abcdef".into(), 4), "abcd [truncated]");
    }

    #[test]
    fn keeps_small_output() {
        assert_eq!(truncate_output("abc".into(), 4), "abc");
    }

    fn sample_config(preferred_editor: Option<&str>) -> ExecutionConfig {
        ExecutionConfig {
            shell: "/bin/sh".into(),
            shell_arg: "-c".into(),
            preferred_editor: preferred_editor.map(ToString::to_string),
        }
    }

    #[test]
    fn capture_is_skipped_for_programs_that_need_the_terminal() {
        let config = sample_config(Some("nvim"));

        // Nothing here is run; only the decision is under test.
        assert!(needs_terminal(
            "nvim notes.md",
            config.preferred_editor.as_deref()
        ));
        assert!(needs_terminal("less big.log", None));
        assert!(needs_terminal("cat f | less", None));
        assert!(needs_terminal("sudo /usr/bin/vim /etc/hosts", None));
        assert!(!needs_terminal("ls -la", None));
        assert!(!needs_terminal("cat notes.md", None));
        // The editor is matched by its first word and its file name.
        assert!(needs_terminal("code f", Some("/usr/local/bin/code --wait")));
    }

    #[test]
    fn a_terminal_program_runs_without_its_output_being_taken_away() {
        // `true` stands in for an editor: named on the skip list, harmless.
        let config = sample_config(Some("true"));

        let result = execute("true", &config, true, 100).expect("the command should run");

        assert_eq!(result.exit_status, Some(0));
        assert!(
            result.stdout.is_none(),
            "a skipped command reports no captured output"
        );
    }

    #[test]
    fn capture_keeps_only_the_configured_number_of_bytes() {
        let config = sample_config(None);

        let result =
            execute("printf '0123456789'", &config, true, 4).expect("the command should run");

        assert_eq!(result.stdout.as_deref(), Some("0123 [truncated]"));
    }

    #[test]
    fn a_zero_limit_keeps_everything() {
        assert_eq!(capture_limit(0), usize::MAX);
        assert_eq!(capture_limit(8), 9);

        let config = sample_config(None);
        let result =
            execute("printf '0123456789'", &config, true, 0).expect("the command should run");

        assert_eq!(result.stdout.as_deref(), Some("0123456789"));
    }

    #[test]
    fn tee_writes_everything_through_and_keeps_the_start() {
        let mut sink = Vec::new();
        let kept = tee(&b"hello world"[..], &mut sink, 5);

        assert_eq!(sink, b"hello world", "everything reaches the terminal");
        assert_eq!(kept, "hello", "only the start is kept");
    }

    /// Proves the command inherits this process's standard input rather
    /// than a pipe or `/dev/null`: both see the same open file. Reading
    /// standard input in a test could block on a terminal, so this compares
    /// what the descriptor points at instead.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_command_inherits_standard_input() {
        let config = sample_config(None);
        let ours = fs::read_link("/proc/self/fd/0").expect("our stdin should resolve");

        let result =
            execute("readlink /proc/self/fd/0", &config, true, 0).expect("the command should run");

        assert_eq!(
            result.stdout.as_deref(),
            Some(ours.display().to_string().as_str())
        );
    }

    #[test]
    fn execute_returns_output_when_capture_enabled() {
        let config = ExecutionConfig {
            shell: "/bin/sh".into(),
            shell_arg: "-c".into(),
            preferred_editor: None,
        };

        let result = execute("printf 'hello'", &config, true, 10)
            .expect("captured execution should succeed");

        assert_eq!(result.exit_status, Some(0));
        assert_eq!(result.stdout.as_deref(), Some("hello"));
        assert_eq!(result.stderr.as_deref(), Some(""));
    }

    #[test]
    fn execute_returns_error_for_failed_command() {
        let config = ExecutionConfig {
            shell: "/bin/sh".into(),
            shell_arg: "-c".into(),
            preferred_editor: None,
        };

        let result = execute("exit 7", &config, false, 0).expect("the shell should launch");

        assert!(result.failed());
        assert_eq!(result.exit_status, Some(7));
        assert_eq!(
            crate::error::kind_of(&result.failure()),
            Some(CliBotError::CommandFailed { status: Some(7) })
        );
    }

    #[test]
    fn execute_returns_error_for_failed_captured_command() {
        let config = ExecutionConfig {
            shell: "/bin/sh".into(),
            shell_arg: "-c".into(),
            preferred_editor: None,
        };

        let result =
            execute("printf 'oops' >&2; exit 2", &config, true, 100).expect("shell should launch");

        assert!(result.failed());
        assert_eq!(result.exit_status, Some(2));
        assert_eq!(result.stderr.as_deref(), Some("oops"));
    }
}
