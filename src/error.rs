//! The error kinds the request flow tells apart.
//!
//! Interactive mode has to know whether an error ends the session, returns
//! the prompt, or is fatal, and the process has to know what to exit with.
//! Both used to decide by matching the text of a message, so rewording one
//! silently changed behaviour. These kinds travel inside an
//! [`anyhow::Error`] and are recognised with `downcast_ref`.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliBotError {
    /// The user ended the prompt, with Ctrl-C or end of input.
    Cancelled,
    /// The command ran and exited non-zero. The status is the command's own,
    /// and `None` when a signal ended it.
    CommandFailed { status: Option<i32> },
    /// The planner could not produce a usable plan: transport, timeout,
    /// decoding, or a plan with no commands in it.
    Planner,
}

impl fmt::Display for CliBotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(formatter, "cancelled"),
            Self::CommandFailed { status: Some(code) } => {
                write!(formatter, "command exited with status {code}")
            }
            Self::CommandFailed { status: None } => {
                write!(formatter, "command was ended by a signal")
            }
            Self::Planner => write!(formatter, "the planner could not produce a command"),
        }
    }
}

impl std::error::Error for CliBotError {}

/// The kind carried by `error`, if any. An `anyhow::Error` built from a
/// [`CliBotError`] keeps it however much context is added on top.
pub fn kind_of(error: &anyhow::Error) -> Option<CliBotError> {
    error.downcast_ref::<CliBotError>().copied()
}

#[cfg(test)]
mod tests {
    use super::{CliBotError, kind_of};

    #[test]
    fn each_kind_survives_a_round_trip_through_anyhow() {
        for kind in [
            CliBotError::Cancelled,
            CliBotError::CommandFailed { status: Some(3) },
            CliBotError::CommandFailed { status: None },
            CliBotError::Planner,
        ] {
            let error = anyhow::Error::new(kind).context("while doing something");

            assert_eq!(kind_of(&error), Some(kind), "{kind}");
        }
    }

    #[test]
    fn an_ordinary_error_has_no_kind() {
        assert_eq!(kind_of(&anyhow::anyhow!("no config file found")), None);
    }

    #[test]
    fn a_failed_command_reads_without_a_doubled_word() {
        assert_eq!(
            CliBotError::CommandFailed { status: Some(3) }.to_string(),
            "command exited with status 3"
        );
        assert_eq!(
            CliBotError::CommandFailed { status: None }.to_string(),
            "command was ended by a signal"
        );
        assert_eq!(CliBotError::Cancelled.to_string(), "cancelled");
        assert_eq!(
            CliBotError::Planner.to_string(),
            "the planner could not produce a command"
        );
    }
}
