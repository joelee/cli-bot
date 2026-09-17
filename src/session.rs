use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::config::{SessionMemoryConfig, SessionScope};
use crate::planner::CommandPlan;

const STDOUT_PREVIEW_LABEL: &str = "stdout";
const STDERR_PREVIEW_LABEL: &str = "stderr";

pub struct SessionStore {
    config: SessionMemoryConfig,
    cwd: PathBuf,
}

impl SessionStore {
    pub fn new(config: SessionMemoryConfig) -> Result<Self> {
        let cwd = env::current_dir().context("failed to resolve current working directory")?;
        Ok(Self { config, cwd })
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn default_name(&self) -> &str {
        &self.config.default_name
    }

    pub fn load(&self, requested_name: Option<&str>) -> Result<SessionRecord> {
        let session_name = requested_name.unwrap_or(self.default_name());
        let session_name = normalize_session_name(session_name)?;
        let session_path = self.path_for_session(&session_name)?;

        if !session_path.is_file() {
            return Ok(SessionRecord::new(
                session_name,
                self.config.scope,
                self.cwd.clone(),
            ));
        }

        let raw = fs::read_to_string(&session_path)
            .with_context(|| format!("failed to read session file `{}`", session_path.display()))?;
        let mut record = serde_json::from_str::<SessionRecord>(&raw).with_context(|| {
            format!("failed to parse session file `{}`", session_path.display())
        })?;
        let max_turns = self.config.effective_max_turns();
        if record.turns.len() > max_turns {
            let drain_count = record.turns.len() - max_turns;
            record.turns.drain(0..drain_count);
        }

        Ok(record)
    }

    pub fn list(&self) -> Result<Vec<SessionListEntry>> {
        let sessions_dir = resolve_storage_root(&self.config)?.join("sessions");

        if !sessions_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for directory_entry in fs::read_dir(&sessions_dir).with_context(|| {
            format!(
                "failed to read session directory `{}`",
                sessions_dir.display()
            )
        })? {
            let directory_entry = directory_entry.with_context(|| {
                format!("failed to read an entry from `{}`", sessions_dir.display())
            })?;
            let path = directory_entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read session file `{}`", path.display()))?;
            let record = serde_json::from_str::<SessionRecord>(&raw)
                .with_context(|| format!("failed to parse session file `{}`", path.display()))?;
            let last_updated_epoch_ms = record
                .turns
                .last()
                .map(|turn| turn.timestamp_epoch_ms)
                .unwrap_or(0);
            let scoped_name = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();

            entries.push(SessionListEntry {
                name: record.name,
                scoped_name,
                scope: record.scope,
                turns: record.turns.len(),
                last_updated_epoch_ms,
                path,
            });
        }

        entries.sort_by(|left, right| {
            right
                .last_updated_epoch_ms
                .cmp(&left.last_updated_epoch_ms)
                .then_with(|| left.scoped_name.cmp(&right.scoped_name))
        });

        Ok(entries)
    }

    pub fn save(&self, record: &mut SessionRecord) -> Result<()> {
        record.scope = self.config.scope;
        record.base_working_directory = self.cwd.clone();
        let max_turns = self.config.effective_max_turns();
        if record.turns.len() > max_turns {
            let drain_count = record.turns.len() - max_turns;
            record.turns.drain(0..drain_count);
        }

        let session_path = self.path_for_session(&record.name)?;
        let parent = session_path
            .parent()
            .context("session path should have a parent directory")?;
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create session directory `{}`", parent.display())
        })?;
        let payload =
            serde_json::to_string_pretty(record).context("failed to serialize session")?;
        fs::write(&session_path, payload)
            .with_context(|| format!("failed to write session file `{}`", session_path.display()))
    }

    pub fn clear(&self, requested_name: Option<&str>) -> Result<bool> {
        let session_name = normalize_session_name(requested_name.unwrap_or(self.default_name()))?;
        let session_path = self.path_for_session(&session_name)?;

        if !session_path.exists() {
            return Ok(false);
        }

        fs::remove_file(&session_path).with_context(|| {
            format!("failed to remove session file `{}`", session_path.display())
        })?;

        Ok(true)
    }

    pub fn prune_expired(&self) -> Result<usize> {
        let retention_days = match self.config.retention_days {
            Some(retention_days) => retention_days,
            None => return Ok(0),
        };

        let cutoff = current_timestamp_epoch_ms().saturating_sub(
            Duration::from_secs(retention_days.saturating_mul(24 * 60 * 60)).as_millis(),
        );
        let sessions_dir = resolve_storage_root(&self.config)?.join("sessions");
        if !sessions_dir.is_dir() {
            return Ok(0);
        }

        let mut removed = 0;
        for directory_entry in fs::read_dir(&sessions_dir).with_context(|| {
            format!(
                "failed to read session directory `{}`",
                sessions_dir.display()
            )
        })? {
            let directory_entry = directory_entry.with_context(|| {
                format!("failed to read an entry from `{}`", sessions_dir.display())
            })?;
            let path = directory_entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read session file `{}`", path.display()))?;
            let record = serde_json::from_str::<SessionRecord>(&raw)
                .with_context(|| format!("failed to parse session file `{}`", path.display()))?;
            let last_updated_epoch_ms = record
                .turns
                .last()
                .map(|turn| turn.timestamp_epoch_ms)
                .unwrap_or(0);

            if last_updated_epoch_ms < cutoff {
                fs::remove_file(&path).with_context(|| {
                    format!("failed to remove expired session file `{}`", path.display())
                })?;
                removed += 1;
            }
        }

        Ok(removed)
    }

    pub fn render_prompt_context(&self, record: &SessionRecord) -> Option<String> {
        if record.turns.is_empty() {
            return None;
        }

        let mut lines = vec![
            "Session context is provided below. Use it only to resolve short follow-up references such as \"it\", \"that\", \"again\", or \"the first one\". If the reference is still ambiguous, set unresolved=true instead of guessing.".to_string(),
            String::new(),
            format!(
                "Session name: {}. Session scope: {}.",
                record.name,
                record.scope.as_str()
            ),
            "Recent session turns:".to_string(),
        ];

        for (index, turn) in record.turns.iter().enumerate() {
            lines.push(format!("{}. request: {}", index + 1, turn.request));

            if let Some(working_directory) = turn.working_directory.as_deref() {
                lines.push(format!("   cwd: {working_directory}"));
            }

            if let Some(summary) = turn.plan_summary.as_deref() {
                lines.push(format!("   plan_summary: {summary}"));
            }

            if turn.unresolved {
                lines.push("   planner: unresolved".to_string());
            }

            if let Some(selected_command) = turn.selected_command.as_deref() {
                lines.push(format!("   selected_command: {selected_command}"));
            }

            if let Some(text_response) = turn.text_response.as_deref() {
                lines.push(format!(
                    "   text_response: {}",
                    summarize_text(text_response, 240)
                ));
            }

            if let Some(execution) = turn.execution.as_ref() {
                let status = if execution.executed {
                    match execution.exit_status {
                        Some(code) => format!("executed with exit status {code}"),
                        None => "executed".to_string(),
                    }
                } else {
                    "not executed".to_string()
                };
                lines.push(format!("   result: {status}"));

                if self.config.include_command_output_in_prompt {
                    if let Some(stdout) = execution.stdout.as_deref() {
                        lines.push(format!(
                            "   {STDOUT_PREVIEW_LABEL}: {}",
                            summarize_text(stdout, self.config.max_output_bytes)
                        ));
                    }
                    if let Some(stderr) = execution.stderr.as_deref() {
                        lines.push(format!(
                            "   {STDERR_PREVIEW_LABEL}: {}",
                            summarize_text(stderr, self.config.max_output_bytes)
                        ));
                    }
                }
            }
        }

        Some(lines.join("\n"))
    }

    pub fn should_capture_command_output(&self) -> bool {
        self.config.capture_command_output
    }

    fn path_for_session(&self, session_name: &str) -> Result<PathBuf> {
        let storage_root = resolve_storage_root(&self.config)?;
        let effective_name = match self.config.scope {
            SessionScope::Global => session_name.to_string(),
            SessionScope::WorkingDirectory => {
                format!("{}-{}", session_name, stable_path_hash(&self.cwd))
            }
        };

        Ok(storage_root
            .join("sessions")
            .join(format!("{effective_name}.json")))
    }
}

pub struct SessionListEntry {
    pub name: String,
    pub scoped_name: String,
    pub scope: SessionScope,
    pub turns: usize,
    pub last_updated_epoch_ms: u128,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub name: String,
    pub scope: SessionScope,
    pub base_working_directory: PathBuf,
    #[serde(default)]
    pub turns: Vec<SessionTurn>,
}

impl SessionRecord {
    pub fn new(name: String, scope: SessionScope, base_working_directory: PathBuf) -> Self {
        Self {
            name,
            scope,
            base_working_directory,
            turns: Vec::new(),
        }
    }

    pub fn push_turn(&mut self, turn: SessionTurn) {
        self.turns.push(turn);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTurn {
    pub timestamp_epoch_ms: u128,
    pub request: String,
    #[serde(default)]
    pub working_directory: Option<String>,
    #[serde(default)]
    pub plan_summary: Option<String>,
    #[serde(default)]
    pub unresolved: bool,
    #[serde(default)]
    pub command_choices: Vec<SessionCommandChoice>,
    #[serde(default)]
    pub selected_command: Option<String>,
    #[serde(default)]
    pub selected_command_rationale: Option<String>,
    #[serde(default)]
    pub confirmation_required: bool,
    #[serde(default)]
    pub text_response: Option<String>,
    #[serde(default)]
    pub execution: Option<SessionExecution>,
}

impl SessionTurn {
    pub fn from_plan(
        request: &str,
        plan: &CommandPlan,
        include_working_directory: bool,
        cwd: &Path,
    ) -> Self {
        Self {
            timestamp_epoch_ms: current_timestamp_epoch_ms(),
            request: request.to_string(),
            working_directory: include_working_directory.then(|| cwd.display().to_string()),
            plan_summary: plan.summary.clone(),
            unresolved: plan.unresolved,
            command_choices: plan
                .commands
                .iter()
                .map(|command| SessionCommandChoice {
                    description: command.description.clone(),
                    command: command.command.clone(),
                    recommended: command.recommended,
                    rationale: command.rationale.clone(),
                })
                .collect(),
            selected_command: None,
            selected_command_rationale: None,
            confirmation_required: false,
            text_response: None,
            execution: None,
        }
    }

    pub fn unresolved_text(
        request: &str,
        response: &str,
        include_working_directory: bool,
        cwd: &Path,
    ) -> Self {
        Self {
            timestamp_epoch_ms: current_timestamp_epoch_ms(),
            request: request.to_string(),
            working_directory: include_working_directory.then(|| cwd.display().to_string()),
            plan_summary: None,
            unresolved: true,
            command_choices: Vec::new(),
            selected_command: None,
            selected_command_rationale: None,
            confirmation_required: false,
            text_response: Some(response.to_string()),
            execution: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCommandChoice {
    pub description: String,
    pub command: String,
    pub recommended: bool,
    #[serde(default)]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionExecution {
    pub executed: bool,
    #[serde(default)]
    pub exit_status: Option<i32>,
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    pub stderr: Option<String>,
}

pub fn normalize_session_name(name: &str) -> Result<String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        bail!("session name must not be empty")
    }

    if trimmed.len() > 64 {
        bail!("session name must be 64 characters or fewer")
    }

    if trimmed
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        Ok(trimmed.to_string())
    } else {
        bail!("session name must contain only ASCII letters, digits, '.', '-', or '_'")
    }
}

fn resolve_storage_root(config: &SessionMemoryConfig) -> Result<PathBuf> {
    if config.storage_dir != "auto" {
        return Ok(PathBuf::from(&config.storage_dir));
    }

    #[cfg(target_os = "macos")]
    {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set for automatic session storage path")?;
        return Ok(home.join("Library/Application Support/cli-bot"));
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(state_home) = env::var_os("XDG_STATE_HOME") {
            return Ok(PathBuf::from(state_home).join("cli-bot"));
        }

        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set for automatic session storage path")?;
        Ok(home.join(".local/state/cli-bot"))
    }
}

fn stable_path_hash(path: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn current_timestamp_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn summarize_text(text: &str, max_bytes: usize) -> String {
    let single_line = text.replace('\n', " ").trim().to_string();

    if max_bytes == 0 {
        return single_line;
    }

    truncate_to_bytes(&single_line, max_bytes)
}

fn truncate_to_bytes(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }

    let mut end = 0;
    for (index, _) in value.char_indices() {
        if index > max_bytes {
            break;
        }
        end = index;
    }

    if end == 0 {
        return "[truncated]".to_string();
    }

    format!("{} [truncated]", &value[..end])
}

#[cfg(test)]
mod tests {
    use super::{
        SessionCommandChoice, SessionExecution, SessionRecord, SessionStore, SessionTurn,
        current_timestamp_epoch_ms, normalize_session_name, summarize_text, truncate_to_bytes,
    };
    use crate::config::{SessionMemoryConfig, SessionScope};
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn accepts_valid_session_names() {
        assert_eq!(
            normalize_session_name("release-notes_1.2").expect("name should pass"),
            "release-notes_1.2"
        );
    }

    #[test]
    fn rejects_invalid_session_names() {
        let error = normalize_session_name("release notes").expect_err("name should fail");
        assert!(error.to_string().contains("session name"));
    }

    #[test]
    fn renders_prompt_context_from_recent_turns() {
        let store = sample_store();
        let mut record = SessionRecord::new(
            "default".into(),
            SessionScope::WorkingDirectory,
            PathBuf::from("/tmp/project"),
        );
        record.push_turn(SessionTurn {
            timestamp_epoch_ms: 1,
            request: "find the git config file".into(),
            working_directory: Some("/tmp/project".into()),
            plan_summary: Some("Locate the config file".into()),
            unresolved: false,
            command_choices: vec![SessionCommandChoice {
                description: "Search for config".into(),
                command: "fd gitconfig ~".into(),
                recommended: true,
                rationale: Some("Fast local search".into()),
            }],
            selected_command: Some("fd gitconfig ~".into()),
            selected_command_rationale: Some("Fast local search".into()),
            confirmation_required: false,
            text_response: None,
            execution: Some(SessionExecution {
                executed: true,
                exit_status: Some(0),
                stdout: None,
                stderr: None,
            }),
        });

        let context = store
            .render_prompt_context(&record)
            .expect("context should exist");

        assert!(context.contains("Session name: default"));
        assert!(context.contains("selected_command: fd gitconfig ~"));
        assert!(context.contains("result: executed with exit status 0"));
    }

    #[test]
    fn saves_and_loads_session_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let temp_root = env::temp_dir().join(format!("cli-bot-session-test-{unique}"));
        fs::create_dir_all(&temp_root).expect("temp root should exist");

        let config = SessionMemoryConfig {
            storage_dir: temp_root.display().to_string(),
            ..SessionMemoryConfig::default()
        };
        let store = SessionStore::new(config).expect("store should initialize");
        let mut record = SessionRecord::new(
            "default".into(),
            SessionScope::WorkingDirectory,
            PathBuf::from("/tmp/project"),
        );
        record.push_turn(SessionTurn {
            timestamp_epoch_ms: 1,
            request: "ping google five times".into(),
            working_directory: Some("/tmp/project".into()),
            plan_summary: Some("Ping google".into()),
            unresolved: false,
            command_choices: Vec::new(),
            selected_command: Some("ping -c 5 google.com".into()),
            selected_command_rationale: None,
            confirmation_required: false,
            text_response: None,
            execution: None,
        });

        store.save(&mut record).expect("session should save");
        let loaded = store.load(Some("default")).expect("session should load");

        assert_eq!(loaded.turns.len(), 1);
        assert_eq!(
            loaded.turns[0].selected_command.as_deref(),
            Some("ping -c 5 google.com")
        );

        fs::remove_dir_all(&temp_root).expect("temp root should be removed");
    }

    #[test]
    fn lists_saved_sessions() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let temp_root = env::temp_dir().join(format!("cli-bot-session-list-test-{unique}"));
        fs::create_dir_all(&temp_root).expect("temp root should exist");

        let config = SessionMemoryConfig {
            storage_dir: temp_root.display().to_string(),
            ..SessionMemoryConfig::default()
        };
        let store = SessionStore::new(config).expect("store should initialize");
        let mut record = SessionRecord::new(
            "default".into(),
            SessionScope::WorkingDirectory,
            PathBuf::from("/tmp/project"),
        );
        record.push_turn(SessionTurn {
            timestamp_epoch_ms: current_timestamp_epoch_ms(),
            request: "ping google five times".into(),
            working_directory: None,
            plan_summary: None,
            unresolved: false,
            command_choices: Vec::new(),
            selected_command: Some("ping -c 5 google.com".into()),
            selected_command_rationale: None,
            confirmation_required: false,
            text_response: None,
            execution: None,
        });
        store.save(&mut record).expect("session should save");

        let entries = store.list().expect("sessions should list");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "default");
        assert_eq!(entries[0].turns, 1);

        fs::remove_dir_all(&temp_root).expect("temp root should be removed");
    }

    #[test]
    fn prunes_expired_sessions() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let temp_root = env::temp_dir().join(format!("cli-bot-session-prune-test-{unique}"));
        fs::create_dir_all(&temp_root).expect("temp root should exist");

        let config = SessionMemoryConfig {
            storage_dir: temp_root.display().to_string(),
            retention_days: Some(1),
            ..SessionMemoryConfig::default()
        };
        let store = SessionStore::new(config).expect("store should initialize");
        let mut record = SessionRecord::new(
            "default".into(),
            SessionScope::WorkingDirectory,
            PathBuf::from("/tmp/project"),
        );
        record.push_turn(SessionTurn {
            timestamp_epoch_ms: current_timestamp_epoch_ms()
                .saturating_sub(Duration::from_secs(3 * 24 * 60 * 60).as_millis()),
            request: "old request".into(),
            working_directory: None,
            plan_summary: None,
            unresolved: false,
            command_choices: Vec::new(),
            selected_command: Some("true".into()),
            selected_command_rationale: None,
            confirmation_required: false,
            text_response: None,
            execution: None,
        });
        store.save(&mut record).expect("session should save");

        let removed = store.prune_expired().expect("prune should succeed");

        assert_eq!(removed, 1);
        assert!(store.list().expect("sessions should list").is_empty());

        fs::remove_dir_all(&temp_root).expect("temp root should be removed");
    }

    #[test]
    fn clear_returns_false_when_session_missing() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let temp_root = env::temp_dir().join(format!("cli-bot-session-clear-test-{unique}"));
        fs::create_dir_all(&temp_root).expect("temp root should exist");

        let config = SessionMemoryConfig {
            storage_dir: temp_root.display().to_string(),
            ..SessionMemoryConfig::default()
        };
        let store = SessionStore::new(config).expect("store should initialize");

        assert!(!store.clear(Some("default")).expect("clear should succeed"));

        fs::remove_dir_all(&temp_root).expect("temp root should be removed");
    }

    #[test]
    fn render_prompt_context_includes_output_when_enabled() {
        let config = SessionMemoryConfig {
            include_command_output_in_prompt: true,
            max_output_bytes: 12,
            ..SessionMemoryConfig::default()
        };
        let store = SessionStore::new(config).expect("store should initialize");
        let mut record = SessionRecord::new(
            "default".into(),
            SessionScope::Global,
            PathBuf::from("/tmp/project"),
        );
        record.push_turn(SessionTurn {
            timestamp_epoch_ms: 1,
            request: "run that again".into(),
            working_directory: Some("/tmp/project".into()),
            plan_summary: None,
            unresolved: false,
            command_choices: Vec::new(),
            selected_command: Some("make test".into()),
            selected_command_rationale: None,
            confirmation_required: false,
            text_response: Some("done".into()),
            execution: Some(SessionExecution {
                executed: true,
                exit_status: None,
                stdout: Some("line one\nline two".into()),
                stderr: Some("warning line".into()),
            }),
        });

        let context = store
            .render_prompt_context(&record)
            .expect("context should exist");

        assert!(context.contains("stdout:"));
        assert!(context.contains("stderr:"));
        assert!(context.contains("line one"));
    }

    #[test]
    fn should_capture_command_output_tracks_config() {
        let config = SessionMemoryConfig {
            capture_command_output: true,
            ..SessionMemoryConfig::default()
        };
        let store = SessionStore::new(config).expect("store should initialize");

        assert!(store.should_capture_command_output());
    }

    #[test]
    fn unresolved_text_turn_marks_request_unresolved() {
        let path = PathBuf::from("/tmp/project");
        let turn =
            SessionTurn::unresolved_text("spell mantainence", "maintenance", true, path.as_path());

        assert!(turn.unresolved);
        assert_eq!(turn.text_response.as_deref(), Some("maintenance"));
        assert_eq!(turn.working_directory.as_deref(), Some("/tmp/project"));
    }

    #[test]
    fn summarize_text_flattens_newlines() {
        assert_eq!(summarize_text("line one\nline two", 0), "line one line two");
    }

    #[test]
    fn truncate_to_bytes_handles_tiny_limits() {
        assert_eq!(truncate_to_bytes("abcdef", 0), "[truncated]");
    }

    fn sample_store() -> SessionStore {
        SessionStore::new(SessionMemoryConfig::default()).expect("store should initialize")
    }
}
