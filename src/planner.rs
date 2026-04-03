use serde::{Deserialize, Serialize};

use crate::config::SafetyConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPlan {
    #[serde(default)]
    pub summary: Option<String>,
    pub commands: Vec<PlannedCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedCommand {
    pub command: String,
    pub description: String,
    #[serde(default)]
    pub potentially_destructive: bool,
    #[serde(default)]
    pub recommended: bool,
    #[serde(default)]
    pub rationale: Option<String>,
}

impl PlannedCommand {
    pub fn label(&self) -> String {
        match self.rationale.as_deref() {
            Some(rationale) => format!("{} [{}] - {}", self.description, self.command, rationale),
            None => format!("{} [{}]", self.description, self.command),
        }
    }
}

pub fn recommended_command(plan: &CommandPlan) -> Option<&PlannedCommand> {
    plan.commands.iter().find(|command| command.recommended)
}

pub fn command_requires_confirmation(command: &PlannedCommand, safety: &SafetyConfig) -> bool {
    if !safety.require_confirmation {
        return false;
    }

    if command.potentially_destructive {
        return true;
    }

    let command_text = command.command.to_ascii_lowercase();

    safety
        .destructive_substrings
        .iter()
        .any(|pattern| command_text.contains(&pattern.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::{CommandPlan, PlannedCommand, command_requires_confirmation, recommended_command};
    use crate::config::SafetyConfig;

    #[test]
    fn detects_destructive_command_from_llm_flag() {
        let safety = SafetyConfig {
            require_confirmation: true,
            destructive_substrings: vec![],
        };
        let command = PlannedCommand {
            command: "git clean -fd".into(),
            description: "Clean repo".into(),
            potentially_destructive: true,
            recommended: false,
            rationale: None,
        };

        assert!(command_requires_confirmation(&command, &safety));
    }

    #[test]
    fn detects_destructive_command_from_configured_pattern() {
        let safety = SafetyConfig {
            require_confirmation: true,
            destructive_substrings: vec!["rm -rf".into()],
        };
        let command = PlannedCommand {
            command: "RM -RF ./target".into(),
            description: "Delete build directory".into(),
            potentially_destructive: false,
            recommended: false,
            rationale: None,
        };

        assert!(command_requires_confirmation(&command, &safety));
    }

    #[test]
    fn skips_confirmation_when_disabled() {
        let safety = SafetyConfig {
            require_confirmation: false,
            destructive_substrings: vec!["rm -rf".into()],
        };
        let command = PlannedCommand {
            command: "rm -rf ./target".into(),
            description: "Delete build directory".into(),
            potentially_destructive: true,
            recommended: false,
            rationale: None,
        };

        assert!(!command_requires_confirmation(&command, &safety));
    }

    #[test]
    fn returns_recommended_command_when_present() {
        let plan = CommandPlan {
            summary: None,
            commands: vec![
                PlannedCommand {
                    command: "ping google.com".into(),
                    description: "Ping once".into(),
                    potentially_destructive: false,
                    recommended: false,
                    rationale: None,
                },
                PlannedCommand {
                    command: "ping -c 5 google.com".into(),
                    description: "Ping five times".into(),
                    potentially_destructive: false,
                    recommended: true,
                    rationale: None,
                },
            ],
        };

        let command = recommended_command(&plan).expect("recommended command should exist");

        assert_eq!(command.command, "ping -c 5 google.com");
    }
}
