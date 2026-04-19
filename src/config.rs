use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG_TEMPLATE: &str = include_str!("../cli-bot.toml");

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub environment: EnvironmentConfig,
    pub safety: SafetyConfig,
    pub ui: UiConfig,
    pub execution: ExecutionConfig,
    #[serde(default)]
    pub session_memory: SessionMemoryConfig,
    #[serde(default)]
    pub models_benchmark: ModelsBenchmarkConfig,
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file `{}`", path.display()))?;

        toml::from_str(&raw)
            .with_context(|| format!("failed to parse config file `{}`", path.display()))
    }
}

pub fn resolve_config_path(explicit: Option<PathBuf>) -> Result<PathBuf> {
    let home = env::var_os("HOME").map(PathBuf::from);
    resolve_config_path_for_home(explicit, home.as_deref())
}

fn resolve_config_path_for_home(explicit: Option<PathBuf>, home: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }

    let candidates = default_config_paths_for_home(home);

    if let Some(path) = candidates.iter().find(|path| path.is_file()) {
        return Ok(path.clone());
    }

    if let Some(home) = home {
        return create_default_user_config(home);
    }

    let searched = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    bail!(
        "no config file found; looked for cli-bot.toml in: {searched}. Use --config to specify a path"
    )
}

fn default_config_paths_for_home(home: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = home {
        paths.push(home.join(".config/cli-bot/cli-bot.toml"));
    }

    paths.push(PathBuf::from("/etc/cli-bot.toml"));
    paths
}

fn create_default_user_config(home: &Path) -> Result<PathBuf> {
    let path = home.join(".config/cli-bot/cli-bot.toml");

    if path.exists() {
        if path.is_file() {
            return Ok(path);
        }

        bail!(
            "default config path `{}` exists but is not a file",
            path.display()
        )
    }

    let parent = path
        .parent()
        .context("default config path should have a parent directory")?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create config directory `{}` for default config",
            parent.display()
        )
    })?;
    fs::write(&path, DEFAULT_CONFIG_TEMPLATE)
        .with_context(|| format!("failed to write default config file `{}`", path.display()))?;

    Ok(path)
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub system_prompt: String,
    #[serde(default)]
    pub use_chat_api: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvironmentConfig {
    #[serde(default = "default_auto_setting")]
    pub os: String,
    #[serde(default = "default_auto_setting")]
    pub distro: String,
    #[serde(default = "default_auto_setting")]
    pub preferred_package_manager: String,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            os: default_auto_setting(),
            distro: default_auto_setting(),
            preferred_package_manager: default_auto_setting(),
        }
    }
}

fn default_auto_setting() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct SafetyConfig {
    pub require_confirmation: bool,
    pub destructive_substrings: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UiConfig {
    pub selection_prompt: String,
    pub approval_prompt: String,
    pub show_command_before_execution: bool,
    #[serde(default)]
    pub auto_select_recommended: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionConfig {
    pub shell: String,
    pub shell_arg: String,
    #[serde(default)]
    pub preferred_editor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionMemoryConfig {
    #[serde(default = "default_session_memory_enabled")]
    pub enabled: bool,
    #[serde(default = "default_session_memory_default_name")]
    pub default_name: String,
    #[serde(default)]
    pub scope: SessionScope,
    #[serde(default = "default_session_memory_storage_dir")]
    pub storage_dir: String,
    #[serde(default = "default_session_memory_max_turns")]
    pub max_turns: usize,
    #[serde(default = "default_session_memory_include_working_directory")]
    pub include_working_directory: bool,
    #[serde(default = "default_session_memory_save_text_responses")]
    pub save_text_responses: bool,
    #[serde(default = "default_session_memory_save_selected_commands")]
    pub save_selected_commands: bool,
    #[serde(default)]
    pub capture_command_output: bool,
    #[serde(default)]
    pub include_command_output_in_prompt: bool,
    #[serde(default = "default_session_memory_max_output_bytes")]
    pub max_output_bytes: usize,
    #[serde(default)]
    pub retention_days: Option<u64>,
}

impl Default for SessionMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: default_session_memory_enabled(),
            default_name: default_session_memory_default_name(),
            scope: SessionScope::default(),
            storage_dir: default_session_memory_storage_dir(),
            max_turns: default_session_memory_max_turns(),
            include_working_directory: default_session_memory_include_working_directory(),
            save_text_responses: default_session_memory_save_text_responses(),
            save_selected_commands: default_session_memory_save_selected_commands(),
            capture_command_output: false,
            include_command_output_in_prompt: false,
            max_output_bytes: default_session_memory_max_output_bytes(),
            retention_days: None,
        }
    }
}

impl SessionMemoryConfig {
    pub fn effective_max_turns(&self) -> usize {
        self.max_turns.max(1)
    }
}

fn default_session_memory_enabled() -> bool {
    true
}

fn default_session_memory_default_name() -> String {
    "default".to_string()
}

fn default_session_memory_storage_dir() -> String {
    "auto".to_string()
}

fn default_session_memory_max_turns() -> usize {
    6
}

fn default_session_memory_include_working_directory() -> bool {
    true
}

fn default_session_memory_save_text_responses() -> bool {
    true
}

fn default_session_memory_save_selected_commands() -> bool {
    true
}

fn default_session_memory_max_output_bytes() -> usize {
    8192
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionScope {
    Global,
    #[default]
    WorkingDirectory,
}

impl SessionScope {
    pub fn as_str(self) -> &'static str {
        match self {
            SessionScope::Global => "global",
            SessionScope::WorkingDirectory => "working_directory",
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModelsBenchmarkConfig {
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub queries: Vec<String>,
}

impl ExecutionConfig {
    pub fn resolved_preferred_editor(&self) -> Option<String> {
        resolve_preferred_editor(
            self.preferred_editor.as_deref(),
            env::var("EDITOR").ok().as_deref(),
        )
    }
}

pub fn is_known_editor(editor: &str) -> bool {
    is_known_editor_in_path(editor, env::var_os("PATH").as_deref())
}

fn is_known_editor_in_path(editor: &str, path_env: Option<&OsStr>) -> bool {
    let editor = editor.trim();

    if editor.is_empty() {
        return false;
    }

    let editor_path = Path::new(editor);

    if editor_path.components().count() > 1 || editor_path.is_absolute() {
        return is_executable_file(editor_path);
    }

    path_env
        .map(env::split_paths)
        .into_iter()
        .flatten()
        .map(|directory| directory.join(editor))
        .any(|candidate| is_executable_file(&candidate))
}

fn is_executable_file(path: &Path) -> bool {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };

    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

fn resolve_preferred_editor(configured: Option<&str>, env_editor: Option<&str>) -> Option<String> {
    configured
        .and_then(normalize_optional_string)
        .or_else(|| env_editor.and_then(normalize_optional_string))
}

fn normalize_optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsString,
        fs,
        path::{Path, PathBuf},
    };

    use std::env;
    use std::ffi::OsStr;

    use super::{
        AppConfig, ExecutionConfig, ModelsBenchmarkConfig, SessionMemoryConfig, SessionScope,
        default_config_paths_for_home, is_known_editor_in_path, resolve_config_path,
        resolve_config_path_for_home, resolve_preferred_editor,
    };

    #[test]
    fn parses_full_config() {
        let config = toml::from_str::<AppConfig>(
            r#"
[ollama]
base_url = "http://127.0.0.1:11434"
model = "lfm2:latest"
temperature = 0.0
system_prompt = "Return JSON only"
use_chat_api = true

[environment]
os = "auto"
distro = "auto"
preferred_package_manager = "auto"

[safety]
require_confirmation = true
destructive_substrings = ["rm -rf", "mkfs"]

[ui]
selection_prompt = "Choose"
approval_prompt = "Approve?"
show_command_before_execution = true
auto_select_recommended = false

[execution]
shell = "/bin/sh"
shell_arg = "-c"
preferred_editor = "nvim"

[session_memory]
enabled = true
default_name = "workspace"
scope = "working_directory"
storage_dir = "auto"
max_turns = 8
include_working_directory = true
save_text_responses = true
save_selected_commands = true
capture_command_output = true
include_command_output_in_prompt = false
max_output_bytes = 4096
retention_days = 14

[models_benchmark]
models = ["lfm2:latest", "qwen3.5:latest"]
queries = ["Ping google five times", "Install btop"]
"#,
        )
        .expect("config should parse");

        assert_eq!(config.ollama.model, "lfm2:latest");
        assert!(config.ollama.use_chat_api);
        assert_eq!(config.environment.os, "auto");
        assert_eq!(config.environment.distro, "auto");
        assert_eq!(config.environment.preferred_package_manager, "auto");
        assert!(config.safety.require_confirmation);
        assert_eq!(config.execution.shell_arg, "-c");
        assert_eq!(config.execution.preferred_editor.as_deref(), Some("nvim"));
        assert!(!config.ui.auto_select_recommended);
        assert!(config.session_memory.enabled);
        assert_eq!(config.session_memory.default_name, "workspace");
        assert_eq!(config.session_memory.scope, SessionScope::WorkingDirectory);
        assert!(config.session_memory.capture_command_output);
        assert_eq!(config.session_memory.max_output_bytes, 4096);
        assert_eq!(config.session_memory.retention_days, Some(14));
        assert_eq!(
            config.models_benchmark.models,
            vec!["lfm2:latest", "qwen3.5:latest"]
        );
        assert_eq!(
            config.models_benchmark.queries,
            vec!["Ping google five times", "Install btop"]
        );
    }

    #[test]
    fn defaults_environment_when_section_missing() {
        let config = toml::from_str::<AppConfig>(
            r#"
[ollama]
base_url = "http://127.0.0.1:11434"
model = "lfm2:latest"
temperature = 0.0
system_prompt = "Return JSON only"

[safety]
require_confirmation = true
destructive_substrings = ["rm -rf"]

[ui]
selection_prompt = "Choose"
approval_prompt = "Approve?"
show_command_before_execution = true
auto_select_recommended = false

[execution]
shell = "/bin/sh"
shell_arg = "-c"
preferred_editor = "nvim"
"#,
        )
        .expect("config should parse");

        assert_eq!(config.environment.os, "auto");
        assert_eq!(config.environment.distro, "auto");
        assert_eq!(config.environment.preferred_package_manager, "auto");
        assert_eq!(config.session_memory, SessionMemoryConfig::default());
        assert!(config.models_benchmark.models.is_empty());
        assert!(config.models_benchmark.queries.is_empty());
    }

    #[test]
    fn session_memory_defaults_are_bounded() {
        let config = SessionMemoryConfig::default();

        assert!(config.enabled);
        assert_eq!(config.default_name, "default");
        assert_eq!(config.scope, SessionScope::WorkingDirectory);
        assert_eq!(config.effective_max_turns(), 6);
        assert_eq!(config.max_output_bytes, 8192);
        assert_eq!(config.retention_days, None);
    }

    #[test]
    fn builds_default_config_search_order() {
        let paths = default_config_paths_for_home(Some(Path::new("/home/tester")));

        assert_eq!(
            paths[0],
            Path::new("/home/tester/.config/cli-bot/cli-bot.toml")
        );
        assert_eq!(paths[1], Path::new("/etc/cli-bot.toml"));
    }

    #[test]
    fn keeps_system_config_when_home_missing() {
        let paths = default_config_paths_for_home(None);

        assert_eq!(paths, vec![PathBuf::from("/etc/cli-bot.toml")]);
    }

    #[test]
    fn prefers_explicit_config_path() {
        let path = PathBuf::from("/tmp/custom-cli-bot.toml");
        let resolved = resolve_config_path(Some(path.clone())).expect("path should resolve");

        assert_eq!(resolved, path);
    }

    #[test]
    fn creates_default_config_when_missing() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let temp_home = env::temp_dir().join(format!("cli-bot-config-test-{unique}"));

        fs::create_dir_all(&temp_home).expect("temp home should be created");

        let resolved = resolve_config_path_for_home(None, Some(&temp_home))
            .expect("default config should be created");

        assert_eq!(resolved, temp_home.join(".config/cli-bot/cli-bot.toml"));
        assert!(resolved.is_file());

        let config = AppConfig::load(&resolved).expect("created config should parse");
        assert_eq!(config.ollama.model, "lfm2:latest");

        fs::remove_file(&resolved).expect("config file should be removed");
        fs::remove_dir_all(&temp_home).expect("temp home should be removed");
    }

    #[test]
    fn prefers_configured_editor_over_environment() {
        let editor = resolve_preferred_editor(Some("nvim"), Some("vim"));

        assert_eq!(editor.as_deref(), Some("nvim"));
    }

    #[test]
    fn falls_back_to_environment_editor() {
        let editor = resolve_preferred_editor(None, Some("hx"));

        assert_eq!(editor.as_deref(), Some("hx"));
    }

    #[test]
    fn ignores_blank_editor_values() {
        let editor = resolve_preferred_editor(Some("  "), Some("  "));

        assert!(editor.is_none());
    }

    #[test]
    fn execution_config_resolves_preferred_editor_from_config() {
        let config = ExecutionConfig {
            shell: "/bin/sh".into(),
            shell_arg: "-c".into(),
            preferred_editor: Some("nano".into()),
        };

        assert_eq!(config.resolved_preferred_editor().as_deref(), Some("nano"));
    }

    #[test]
    fn models_benchmark_defaults_empty() {
        let config = ModelsBenchmarkConfig::default();

        assert!(config.models.is_empty());
        assert!(config.queries.is_empty());
    }

    #[test]
    fn rejects_unknown_editor() {
        assert!(!is_known_editor_in_path(
            "missing-editor",
            Some(OsStr::new("/tmp"))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn finds_editor_in_supplied_path() {
        use std::os::unix::fs::PermissionsExt;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("cli-bot-editor-test-{unique}"));
        fs::create_dir_all(&temp_dir).expect("temp dir should be created");

        let editor_path = temp_dir.join("my-editor");
        fs::write(&editor_path, b"#!/bin/sh\n").expect("editor stub should be written");

        let mut permissions = fs::metadata(&editor_path)
            .expect("metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&editor_path, permissions).expect("permissions should be updated");

        let path_env = OsString::from(temp_dir.as_os_str());
        assert!(is_known_editor_in_path(
            "my-editor",
            Some(path_env.as_os_str())
        ));

        fs::remove_file(&editor_path).expect("editor stub should be removed");
        fs::remove_dir(&temp_dir).expect("temp dir should be removed");
    }
}
