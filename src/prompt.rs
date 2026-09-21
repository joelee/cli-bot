//! The terminal prompts, behind a trait.
//!
//! Every question cli-bot asks a user goes through [`Prompter`], so the
//! request flow can be driven by a test without a terminal. Production uses
//! [`DialoguerPrompter`], which is the behaviour the CLI had before the trait
//! existed.

use std::env;
use std::io::{self, IsTerminal};

use anyhow::{Context, Result};
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};

use crate::error::CliBotError;
use crate::normalize_request;

/// Asks the user the three questions the request flow needs.
pub trait Prompter {
    /// Reads the next natural-language request. `error_state` colours the
    /// interactive prompt after a failed request.
    fn read_request(&self, error_state: bool) -> Result<String>;

    /// Chooses one of `items`, returning its index.
    fn select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize>;

    /// Asks a yes/no question. `default` is the answer an empty reply gives.
    fn confirm(&self, prompt: &str, default: bool) -> Result<bool>;

    /// Whether a dialog can be shown at all. When this is false the caller
    /// must fail rather than assume an answer.
    fn supports_dialogs(&self) -> bool;
}

/// The production prompter: `dialoguer` on the real terminal.
#[derive(Debug, Default, Clone, Copy)]
pub struct DialoguerPrompter;

impl DialoguerPrompter {
    pub fn new() -> Self {
        Self
    }
}

impl Prompter for DialoguerPrompter {
    fn read_request(&self, error_state: bool) -> Result<String> {
        if !io::stdin().is_terminal() {
            let mut request = String::new();
            let read = io::stdin()
                .read_line(&mut request)
                .context("failed to read request from stdin")?;
            // Nothing left to read: the pipe ended, or the file did.
            if read == 0 {
                return Err(
                    anyhow::Error::new(CliBotError::Cancelled).context("no request was provided")
                );
            }

            return normalize_request(request);
        }

        let theme = interactive_prompt_theme(error_state);

        Input::<String>::with_theme(&theme)
            .with_prompt("")
            .validate_with(|input: &String| -> std::result::Result<(), &str> {
                if input.trim().is_empty() {
                    Err("request must not be empty")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            // `dialoguer` reports Ctrl-C and a lost terminal the same way,
            // and both mean the same thing here: the user is done.
            .map_err(|_| {
                anyhow::Error::new(CliBotError::Cancelled)
                    .context("failed to capture request from terminal")
            })
            .and_then(normalize_request)
    }

    fn select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize> {
        Select::new()
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .context("failed to capture command selection from terminal")
    }

    fn confirm(&self, prompt: &str, default: bool) -> Result<bool> {
        Confirm::new()
            .with_prompt(prompt)
            .default(default)
            .interact()
            .context("failed to capture confirmation from terminal")
    }

    fn supports_dialogs(&self) -> bool {
        terminal_environment_status().interactive_dialogs_supported()
    }
}

fn interactive_prompt_theme(error_state: bool) -> ColorfulTheme {
    ColorfulTheme {
        prompt_prefix: style("".to_string()).for_stderr().dim(),
        prompt_suffix: style(format!(
            "{}{}",
            if error_state {
                console::Style::new().for_stderr().red().apply_to("cli-bot")
            } else {
                console::Style::new()
                    .for_stderr()
                    .cyan()
                    .apply_to("cli-bot")
            },
            console::Style::new().for_stderr().dim().apply_to(">")
        )),
        prompt_style: console::Style::new().dim(),
        values_style: console::Style::new().for_stderr(),
        ..ColorfulTheme::default()
    }
}

/// The current terminal's state, for an error message that tells the user
/// which part is missing.
pub fn describe_terminal() -> String {
    terminal_environment_status().describe()
}

fn terminal_environment_status() -> TerminalEnvironmentStatus {
    TerminalEnvironmentStatus {
        stdin_tty: io::stdin().is_terminal(),
        stdout_tty: io::stdout().is_terminal(),
        stderr_tty: io::stderr().is_terminal(),
        term: env::var("TERM").ok().filter(|term| !term.trim().is_empty()),
    }
}

struct TerminalEnvironmentStatus {
    stdin_tty: bool,
    stdout_tty: bool,
    stderr_tty: bool,
    term: Option<String>,
}

impl TerminalEnvironmentStatus {
    fn interactive_dialogs_supported(&self) -> bool {
        self.stdin_tty
            && self.stdout_tty
            && self.stderr_tty
            && self
                .term
                .as_deref()
                .is_some_and(|term| !term.is_empty() && term != "dumb")
    }

    fn describe(&self) -> String {
        format!(
            "stdin_tty={}, stdout_tty={}, stderr_tty={}, TERM={}",
            self.stdin_tty,
            self.stdout_tty,
            self.stderr_tty,
            self.term.as_deref().unwrap_or("unset")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{DialoguerPrompter, Prompter, TerminalEnvironmentStatus, interactive_prompt_theme};

    #[test]
    fn terminal_status_accepts_interactive_terminal() {
        let status = TerminalEnvironmentStatus {
            stdin_tty: true,
            stdout_tty: true,
            stderr_tty: true,
            term: Some("xterm-ghostty".into()),
        };

        assert!(status.interactive_dialogs_supported());
        assert!(status.describe().contains("TERM=xterm-ghostty"));
    }

    #[test]
    fn terminal_status_rejects_non_interactive_terminal() {
        let status = TerminalEnvironmentStatus {
            stdin_tty: false,
            stdout_tty: false,
            stderr_tty: false,
            term: Some("xterm-ghostty".into()),
        };

        assert!(!status.interactive_dialogs_supported());
    }

    #[test]
    fn terminal_status_rejects_dumb_and_unset_term() {
        for term in [Some("dumb".to_string()), None] {
            let status = TerminalEnvironmentStatus {
                stdin_tty: true,
                stdout_tty: true,
                stderr_tty: true,
                term,
            };

            assert!(!status.interactive_dialogs_supported());
        }

        let unset = TerminalEnvironmentStatus {
            stdin_tty: true,
            stdout_tty: true,
            stderr_tty: true,
            term: None,
        };

        assert!(unset.describe().contains("TERM=unset"));
    }

    #[test]
    fn dialoguer_prompter_reports_no_dialogs_without_a_terminal() {
        // `cargo test` captures stdout, so this process has no usable
        // terminal and the production prompter must say so.
        assert!(!DialoguerPrompter::new().supports_dialogs());
    }

    #[test]
    fn interactive_prompt_theme_switches_prompt_color_on_error() {
        let normal_rendered = format!("{}", interactive_prompt_theme(false).prompt_suffix);
        let error_rendered = format!("{}", interactive_prompt_theme(true).prompt_suffix);
        let normal = console::strip_ansi_codes(&normal_rendered);
        let error = console::strip_ansi_codes(&error_rendered);

        assert_eq!(normal, "cli-bot>");
        assert_eq!(error, "cli-bot>");
    }
}
