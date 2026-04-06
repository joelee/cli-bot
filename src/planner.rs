use serde::{Deserialize, Deserializer, Serialize};

use crate::config::SafetyConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPlan {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub unresolved: bool,
    #[serde(default, deserialize_with = "deserialize_commands")]
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

fn deserialize_commands<'de, D>(deserializer: D) -> Result<Vec<PlannedCommand>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum CommandInput {
        Structured(PlannedCommand),
        Simple(String),
    }

    let inputs = Vec::<CommandInput>::deserialize(deserializer)?;

    Ok(inputs
        .into_iter()
        .map(|input| match input {
            CommandInput::Structured(command) => command,
            CommandInput::Simple(command) => PlannedCommand {
                description: command.clone(),
                command,
                potentially_destructive: false,
                recommended: false,
                rationale: None,
            },
        })
        .collect())
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
            unresolved: false,
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

    #[test]
    fn parses_simple_string_command_entries() {
        let plan = serde_json::from_str::<CommandPlan>(
            r#"{"summary":"Ping","unresolved":false,"commands":["ping -c 5 google.com"]}"#,
        )
        .expect("plan should parse");

        assert_eq!(plan.commands.len(), 1);
        assert_eq!(plan.commands[0].command, "ping -c 5 google.com");
        assert_eq!(plan.commands[0].description, "ping -c 5 google.com");
        assert!(!plan.commands[0].recommended);
    }
}
