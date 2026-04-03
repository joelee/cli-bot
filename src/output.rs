use std::env;
use std::io::{self, IsTerminal};

use clap::ValueEnum;
use owo_colors::OwoColorize;

#[derive(Clone, Debug, ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

pub struct OutputStyler {
    stdout_enabled: bool,
    stderr_enabled: bool,
}

impl OutputStyler {
    pub fn new(mode: ColorMode) -> Self {
        if env::var_os("NO_COLOR").is_some() {
            return Self {
                stdout_enabled: false,
                stderr_enabled: false,
            };
        }

        match mode {
            ColorMode::Auto => Self {
                stdout_enabled: io::stdout().is_terminal(),
                stderr_enabled: io::stderr().is_terminal(),
            },
            ColorMode::Always => Self {
                stdout_enabled: true,
                stderr_enabled: true,
            },
            ColorMode::Never => Self {
                stdout_enabled: false,
                stderr_enabled: false,
            },
        }
    }

    pub fn heading(&self, text: &str) -> String {
        self.paint_stdout(text, Tone::Heading)
    }

    pub fn key(&self, text: &str) -> String {
        self.paint_stdout(text, Tone::Key)
    }

    pub fn ok(&self, text: &str) -> String {
        self.paint_stdout(text, Tone::Ok)
    }

    pub fn warn(&self, text: &str) -> String {
        self.paint_stdout(text, Tone::Warn)
    }

    pub fn error(&self, text: &str) -> String {
        self.paint_stdout(text, Tone::Error)
    }

    pub fn accent(&self, text: &str) -> String {
        self.paint_stdout(text, Tone::Accent)
    }

    pub fn dim(&self, text: &str) -> String {
        self.paint_stdout(text, Tone::Dim)
    }

    pub fn stderr_error(&self, text: &str) -> String {
        self.paint_stderr(text, Tone::Error)
    }

    pub fn stderr_dim(&self, text: &str) -> String {
        self.paint_stderr(text, Tone::Dim)
    }

    fn paint_stdout(&self, text: &str, tone: Tone) -> String {
        self.paint(text, tone, self.stdout_enabled)
    }

    fn paint_stderr(&self, text: &str, tone: Tone) -> String {
        self.paint(text, tone, self.stderr_enabled)
    }

    fn paint(&self, text: &str, tone: Tone, enabled: bool) -> String {
        if !enabled {
            return text.to_string();
        }

        match tone {
            Tone::Heading => text.bold().bright_cyan().to_string(),
            Tone::Key => text.bold().yellow().to_string(),
            Tone::Ok => text.bold().green().to_string(),
            Tone::Warn => text.bold().yellow().to_string(),
            Tone::Error => text.bold().red().to_string(),
            Tone::Accent => text.bold().cyan().to_string(),
            Tone::Dim => text.dimmed().to_string(),
        }
    }

    #[cfg(test)]
    fn for_test(stdout_enabled: bool, stderr_enabled: bool) -> Self {
        Self {
            stdout_enabled,
            stderr_enabled,
        }
    }
}

#[derive(Clone, Copy)]
enum Tone {
    Heading,
    Key,
    Ok,
    Warn,
    Error,
    Accent,
    Dim,
}

#[cfg(test)]
mod tests {
    use super::OutputStyler;

    #[test]
    fn returns_plain_text_when_colors_disabled() {
        let styler = OutputStyler::for_test(false, false);

        assert_eq!(styler.ok("ok"), "ok");
        assert_eq!(styler.stderr_error("Error:"), "Error:");
        assert_eq!(styler.stderr_dim("[verbose] test"), "[verbose] test");
    }

    #[test]
    fn returns_ansi_sequences_when_colors_enabled() {
        let styler = OutputStyler::for_test(true, true);

        assert!(styler.ok("ok").contains("\u{1b}["));
        assert!(styler.stderr_error("Error:").contains("\u{1b}["));
        assert!(styler.stderr_dim("[verbose] test").contains("\u{1b}["));
    }
}
