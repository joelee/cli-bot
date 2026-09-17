use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::OllamaConfig;
use crate::environment::ResolvedEnvironment;
use crate::output::OutputStyler;
use crate::planner::CommandPlan;

pub struct OllamaClient {
    client: Client,
    config: OllamaConfig,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn plan_commands(
        &self,
        request: &str,
        destructive_substrings: &[String],
        preferred_editor: Option<&str>,
        environment: &ResolvedEnvironment,
        session_context: Option<&str>,
        verbose: bool,
        output: &OutputStyler,
    ) -> Result<CommandPlan> {
        let request_body = GenerateRequest {
            model: &self.config.model,
            prompt: build_command_prompt(
                request,
                destructive_substrings,
                preferred_editor,
                environment,
                session_context,
            ),
            system: &self.config.system_prompt,
            stream: false,
            options: GenerateOptions {
                temperature: self.config.temperature,
            },
            expects_json: true,
        };

        let generated = self.generate_text(&request_body, verbose, output)?;
        let json = extract_json_document(&generated).with_context(|| {
            if verbose {
                format!(
                    "Ollama response did not contain a JSON object\nGenerated text:\n{generated}"
                )
            } else {
                "Ollama response did not contain a JSON object".to_string()
            }
        })?;

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!("[verbose] extracted planner json:\n{json}"))
            );
        }

        let plan = serde_json::from_str::<CommandPlan>(json).with_context(|| {
            if verbose {
                format!(
                    "failed to parse planner JSON returned by Ollama\nExtracted planner JSON:\n{json}\nFull generated text:\n{generated}"
                )
            } else {
                "failed to parse planner JSON returned by Ollama".to_string()
            }
        })?;

        validate_plan(&plan)?;

        Ok(plan)
    }

    pub fn answer_unresolved(
        &self,
        request: &str,
        preferred_editor: Option<&str>,
        environment: &ResolvedEnvironment,
        session_context: Option<&str>,
        verbose: bool,
        output: &OutputStyler,
    ) -> Result<String> {
        let request_body = GenerateRequest {
            model: &self.config.model,
            prompt: build_text_response_prompt(
                request,
                preferred_editor,
                environment,
                session_context,
            ),
            system: &self.config.system_prompt,
            stream: false,
            options: GenerateOptions {
                temperature: self.config.temperature,
            },
            expects_json: false,
        };

        let generated = self.generate_text(&request_body, verbose, output)?;
        let answer = generated.trim();

        if answer.is_empty() {
            bail!("ollama returned an empty text response")
        }

        Ok(answer.to_string())
    }

    fn generate_text(
        &self,
        request_body: &GenerateRequest<'_>,
        verbose: bool,
        output: &OutputStyler,
    ) -> Result<String> {
        if self.config.use_chat_api {
            return self.chat_text(request_body, verbose, output);
        }

        self.generate_text_via_generate(request_body, verbose, output)
    }

    fn generate_text_via_generate(
        &self,
        request_body: &GenerateRequest<'_>,
        verbose: bool,
        output: &OutputStyler,
    ) -> Result<String> {
        let url = format!(
            "{}/api/generate",
            self.config.base_url.trim_end_matches('/')
        );

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!(
                    "[verbose] ollama generate url: {url}\n[verbose] ollama generate request:\n{}",
                    serde_json::to_string_pretty(request_body)
                        .unwrap_or_else(|_| "<failed to serialize request>".to_string())
                ))
            );
        }

        let response = self
            .client
            .post(url)
            .json(request_body)
            .send()
            .context("failed to call Ollama")?
            .error_for_status()
            .context("Ollama returned an unsuccessful response")?
            .text()
            .context("failed to read Ollama response body")?;

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!("[verbose] ollama raw response:\n{response}"))
            );
        }

        let response = serde_json::from_str::<GenerateResponse>(&response).with_context(|| {
            if verbose {
                format!("failed to decode Ollama response\nFull Ollama output:\n{response}")
            } else {
                "failed to decode Ollama response".to_string()
            }
        })?;

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!(
                    "[verbose] ollama generated text:\n{}",
                    response.response
                ))
            );
        }

        Ok(response.response)
    }

    fn chat_text(
        &self,
        request_body: &GenerateRequest<'_>,
        verbose: bool,
        output: &OutputStyler,
    ) -> Result<String> {
        let url = format!("{}/api/chat", self.config.base_url.trim_end_matches('/'));
        let chat_request = ChatRequest {
            model: request_body.model,
            messages: vec![
                ChatMessageRequest {
                    role: "system",
                    content: request_body.system.to_string(),
                },
                ChatMessageRequest {
                    role: "user",
                    content: request_body.prompt.clone(),
                },
            ],
            stream: false,
            options: request_body.options.clone(),
            format: request_body.expects_json.then_some("json"),
        };

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!(
                    "[verbose] ollama chat url: {url}\n[verbose] ollama chat request:\n{}",
                    serde_json::to_string_pretty(&chat_request)
                        .unwrap_or_else(|_| "<failed to serialize request>".to_string())
                ))
            );
        }

        let response = self
            .client
            .post(url)
            .json(&chat_request)
            .send()
            .context("failed to call Ollama")?
            .error_for_status()
            .context("Ollama returned an unsuccessful response")?
            .text()
            .context("failed to read Ollama response body")?;

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!("[verbose] ollama raw chat response:\n{response}"))
            );
        }

        let response = serde_json::from_str::<ChatResponse>(&response).with_context(|| {
            if verbose {
                format!("failed to decode Ollama chat response\nFull Ollama output:\n{response}")
            } else {
                "failed to decode Ollama chat response".to_string()
            }
        })?;

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!(
                    "[verbose] ollama chat generated text:\n{}",
                    response.message.content
                ))
            );
        }

        Ok(response.message.content)
    }

    pub fn check_service(&self, verbose: bool, output: &OutputStyler) -> Result<OllamaStatus> {
        let version = self.fetch_version(verbose, output).ok();
        let response = self.fetch_tags(verbose, output)?;

        Ok(OllamaStatus {
            model_available: response
                .models
                .iter()
                .any(|model| model.name == self.config.model),
            version,
        })
    }

    pub fn benchmark_metadata(
        &self,
        verbose: bool,
        output: &OutputStyler,
    ) -> Result<OllamaBenchmarkMetadata> {
        let version = self.fetch_version(verbose, output).ok();
        let tags = self.fetch_tags(verbose, output)?;
        let model_parameter_sizes = tags
            .models
            .into_iter()
            .filter_map(|model| {
                model
                    .details
                    .and_then(|details| details.parameter_size)
                    .map(|parameter_size| (model.name, parameter_size))
            })
            .collect::<BTreeMap<_, _>>();

        Ok(OllamaBenchmarkMetadata {
            version,
            model_parameter_sizes,
        })
    }

    fn fetch_version(&self, verbose: bool, output: &OutputStyler) -> Result<String> {
        let url = format!("{}/api/version", self.config.base_url.trim_end_matches('/'));

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!("[verbose] ollama version url: {url}"))
            );
        }

        let response = self
            .client
            .get(url)
            .send()
            .context("failed to reach Ollama version endpoint")?
            .error_for_status()
            .context("Ollama version endpoint returned an unsuccessful response")?
            .text()
            .context("failed to read Ollama version response body")?;

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!(
                    "[verbose] ollama version raw response:\n{response}"
                ))
            );
        }

        let response = serde_json::from_str::<VersionResponse>(&response).with_context(|| {
            if verbose {
                format!("failed to decode Ollama version response\nFull Ollama output:\n{response}")
            } else {
                "failed to decode Ollama version response".to_string()
            }
        })?;

        Ok(response.version)
    }

    fn fetch_tags(&self, verbose: bool, output: &OutputStyler) -> Result<TagsResponse> {
        let url = format!("{}/api/tags", self.config.base_url.trim_end_matches('/'));

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!("[verbose] ollama tags url: {url}"))
            );
        }

        let response = self
            .client
            .get(url)
            .send()
            .context("failed to reach Ollama service")?
            .error_for_status()
            .context("Ollama service returned an unsuccessful response")?
            .text()
            .context("failed to read Ollama tags response body")?;

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!("[verbose] ollama tags raw response:\n{response}"))
            );
        }

        serde_json::from_str::<TagsResponse>(&response).with_context(|| {
            if verbose {
                format!("failed to decode Ollama tags response\nFull Ollama output:\n{response}")
            } else {
                "failed to decode Ollama tags response".to_string()
            }
        })
    }
}

pub struct OllamaStatus {
    pub model_available: bool,
    pub version: Option<String>,
}

pub struct OllamaBenchmarkMetadata {
    pub version: Option<String>,
    pub model_parameter_sizes: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: String,
    system: &'a str,
    stream: bool,
    options: GenerateOptions,
    #[serde(skip_serializing)]
    expects_json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct GenerateOptions {
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessageRequest>,
    stream: bool,
    options: GenerateOptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct ChatMessageRequest {
    role: &'static str,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<TaggedModel>,
}

#[derive(Debug, Deserialize)]
struct TaggedModel {
    name: String,
    details: Option<TaggedModelDetails>,
}

#[derive(Debug, Deserialize)]
struct TaggedModelDetails {
    parameter_size: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    version: String,
}

fn build_command_prompt(
    request: &str,
    destructive_substrings: &[String],
    preferred_editor: Option<&str>,
    environment: &ResolvedEnvironment,
    session_context: Option<&str>,
) -> String {
    let destructive_examples =
        serde_json::to_string(destructive_substrings).unwrap_or_else(|_| "[]".to_string());
    let preferred_editor = preferred_editor.unwrap_or("not specified");
    let distro = environment
        .distro
        .map(|distro| distro.as_str())
        .unwrap_or("not applicable");
    let detected_package_manager = environment
        .detected_package_manager
        .map(|package_manager| package_manager.as_str())
        .unwrap_or("unknown");
    let effective_package_manager = environment
        .effective_package_manager
        .map(|package_manager| package_manager.as_str())
        .unwrap_or("unknown");
    let session_context = session_context.unwrap_or("No session context available.");

    format!(
        concat!(
            "You are a shell-focused assistant. Translate the user request into one or more shell commands when the user is clearly asking for a terminal action. ",
            "Return JSON only with the schema ",
            r#"{{"summary":"short summary","unresolved":false,"commands":[{{"command":"...","description":"...","potentially_destructive":false,"recommended":true,"rationale":"..."}}]}}"#,
            ". Set `unresolved` to true when you cannot confidently turn the request into a shell command without guessing. ",
            "When `unresolved` is true, return an empty `commands` array. ",
            "If there are multiple plausible shell commands, include each one in the commands array. ",
            "Do not answer factual questions directly in this step. If the request is not clearly a shell command request or needs non-command output, set `unresolved` to true instead of guessing. ",
            "If a request could reasonably refer to multiple targets or contexts, set `unresolved` to true instead of guessing. ",
            "Examples of unresolved requests: `spell mantainence`, `open config`, `what does foo mean`. ",
            "Examples of command requests: `What git branch am I in?` -> `git rev-parse --abbrev-ref HEAD`, `install btop` -> package manager command, `ping google five times` -> `ping -c 5 google.com`. ",
            "Set potentially_destructive to true when the command could delete, overwrite, stop, or reconfigure something important. ",
            "When there are multiple command choices, mark the single best choice with recommended=true and set recommended=false for the others. ",
            "When there is only one command, set recommended=true. ",
            "Operating system: {os}. Linux distribution: {distro}. Detected package manager: {detected_package_manager}. Effective package manager: {effective_package_manager}. ",
            "Only use package-manager commands when the user is clearly asking about packages, installing software, removing software, upgrading packages, searching repositories, or listing installed software. Do not guess with package-manager commands for unrelated requests. ",
            "For explicitly package-related requests, use commands appropriate for this environment and prefer the effective package manager. ",
            "Use the package manager's canonical syntax and only pass the package name as the package argument. Examples: `brew install btop`, `paru -S btop`, `pacman -Q`, `apt list --installed`. ",
            "Preferred terminal editor: {preferred_editor}. If the user asks to edit a file, prefer commands that open that editor. ",
            "Known destructive patterns: {destructive_examples}. ",
            "Session context: {session_context}. ",
            "Treat session context as advisory only. Do not assume prior commands succeeded unless the context says they did. If a follow-up could refer to multiple prior targets, set unresolved=true. ",
            "User request: {request}"
        ),
        destructive_examples = destructive_examples,
        os = environment.os.as_str(),
        distro = distro,
        detected_package_manager = detected_package_manager,
        effective_package_manager = effective_package_manager,
        preferred_editor = preferred_editor,
        session_context = session_context,
        request = request,
    )
}

fn build_text_response_prompt(
    request: &str,
    preferred_editor: Option<&str>,
    environment: &ResolvedEnvironment,
    session_context: Option<&str>,
) -> String {
    let preferred_editor = preferred_editor.unwrap_or("not specified");
    let distro = environment
        .distro
        .map(|distro| distro.as_str())
        .unwrap_or("not applicable");
    let effective_package_manager = environment
        .effective_package_manager
        .map(|package_manager| package_manager.as_str())
        .unwrap_or("unknown");
    let session_context = session_context.unwrap_or("No session context available.");

    format!(
        concat!(
            "The user's request was not resolved as a shell command. Respond with plain text only, not JSON. ",
            "If the request is a spelling, wording, or factual prompt, answer directly and concisely. ",
            "If the request is ambiguous, ask one concise clarification question instead of guessing. ",
            "Do not invent shell command results or system state. ",
            "Session context: {session_context}. Treat it as advisory only and do not assume command output unless explicitly provided. ",
            "Operating system: {os}. Linux distribution: {distro}. Effective package manager: {effective_package_manager}. Preferred terminal editor: {preferred_editor}. ",
            "User request: {request}"
        ),
        os = environment.os.as_str(),
        distro = distro,
        effective_package_manager = effective_package_manager,
        preferred_editor = preferred_editor,
        session_context = session_context,
        request = request,
    )
}

fn validate_plan(plan: &CommandPlan) -> Result<()> {
    if plan.unresolved {
        return Ok(());
    }

    if plan.commands.is_empty() {
        bail!("planner returned an empty command list")
    }

    Ok(())
}

fn extract_json_document(response: &str) -> Option<&str> {
    let start = response.find('{')?;
    let end = response.rfind('}')?;

    response.get(start..=end)
}

#[cfg(test)]
mod tests {
    use super::{
        GenerateRequest, OllamaStatus, build_command_prompt, build_text_response_prompt,
        extract_json_document, validate_plan,
    };
    use crate::environment::{
        OperatingSystem, PackageManager, PackageManagerSource, ResolvedEnvironment,
    };
    use crate::planner::{CommandPlan, PlannedCommand};

    #[test]
    fn extracts_plain_json_document() {
        let json = extract_json_document(r#"{"commands":[]}"#).expect("json should be extracted");

        assert_eq!(json, r#"{"commands":[]}"#);
    }

    #[test]
    fn extracts_json_from_markdown_fence() {
        let payload = "Here is the plan:\n```json\n{\"commands\":[]}\n```";
        let json = extract_json_document(payload).expect("json should be extracted");

        assert_eq!(json, "{\"commands\":[]}");
    }

    #[test]
    fn returns_none_when_json_missing() {
        assert!(extract_json_document("no structured output here").is_none());
    }

    #[test]
    fn prompt_requires_command_plan_schema() {
        let prompt = build_command_prompt(
            "delete the target directory",
            &["rm -rf".into()],
            None,
            &sample_environment(),
            None,
        );

        assert!(prompt.contains("\"unresolved\":false"));
        assert!(prompt.contains("Examples of command requests"));
        assert!(prompt.contains("What git branch am I in?"));
        assert!(prompt.contains("set `unresolved` to true"));
    }

    #[test]
    fn command_prompt_includes_environment_context() {
        let prompt = build_command_prompt("install btop", &[], None, &sample_environment(), None);

        assert!(prompt.contains("Operating system: linux"));
        assert!(prompt.contains("Linux distribution: arch"));
        assert!(prompt.contains("Detected package manager: paru"));
        assert!(prompt.contains("Effective package manager: paru"));
        assert!(prompt.contains("Only use package-manager commands"));
    }

    #[test]
    fn text_response_prompt_mentions_plain_text() {
        let prompt =
            build_text_response_prompt("spell mantainence", None, &sample_environment(), None);

        assert!(prompt.contains("Respond with plain text only"));
        assert!(prompt.contains("spell mantainence"));
    }

    #[test]
    fn text_response_prompt_includes_session_context_when_present() {
        let prompt = build_text_response_prompt(
            "show that again",
            Some("nvim"),
            &sample_environment(),
            Some("Session name: default\n1. request: find the config file"),
        );

        assert!(prompt.contains("Session context: Session name: default"));
        assert!(prompt.contains("do not assume command output unless explicitly provided"));
    }

    #[test]
    fn generate_request_serializes_prompt_for_verbose_logging() {
        let request = GenerateRequest {
            model: "lfm2:latest",
            prompt: build_command_prompt(
                "ping google",
                &[],
                Some("nvim"),
                &sample_environment(),
                Some("Session name: default"),
            ),
            system: "Return JSON only",
            stream: false,
            options: super::GenerateOptions { temperature: 0.0 },
            expects_json: true,
        };

        let json = serde_json::to_string(&request).expect("request should serialize");

        assert!(json.contains("lfm2:latest"));
        assert!(json.contains("ping google"));
        assert!(json.contains("Session name: default"));
    }

    #[test]
    fn command_prompt_includes_session_context_when_present() {
        let prompt = build_command_prompt(
            "open it again",
            &[],
            Some("nvim"),
            &sample_environment(),
            Some("Session name: default\n1. request: find git config"),
        );

        assert!(prompt.contains("Session context: Session name: default"));
        assert!(prompt.contains("Treat session context as advisory only"));
    }

    #[test]
    fn validates_command_plan() {
        let plan = CommandPlan {
            summary: None,
            unresolved: false,
            commands: vec![PlannedCommand {
                command: "ping -c 5 google.com".into(),
                description: "Ping five times".into(),
                potentially_destructive: false,
                recommended: true,
                rationale: None,
            }],
        };

        validate_plan(&plan).expect("command plan should validate");
    }

    #[test]
    fn validates_unresolved_plan_without_commands() {
        let plan = CommandPlan {
            summary: Some("Need clarification".into()),
            unresolved: true,
            commands: vec![],
        };

        validate_plan(&plan).expect("unresolved plan should validate");
    }

    #[test]
    fn rejects_resolved_plan_without_commands() {
        let plan = CommandPlan {
            summary: None,
            unresolved: false,
            commands: vec![],
        };

        let error = validate_plan(&plan).expect_err("plan should fail");

        assert!(
            error
                .to_string()
                .contains("planner returned an empty command list")
        );
    }

    #[test]
    fn check_service_status_fields_are_accessible() {
        let status = OllamaStatus {
            model_available: false,
            version: Some("0.6.0".into()),
        };

        assert!(!status.model_available);
        assert_eq!(status.version.as_deref(), Some("0.6.0"));
    }

    fn sample_environment() -> ResolvedEnvironment {
        ResolvedEnvironment {
            os: OperatingSystem::Linux,
            distro: Some(crate::environment::LinuxDistro::Arch),
            detected_package_manager: Some(PackageManager::Paru),
            effective_package_manager: Some(PackageManager::Paru),
            package_manager_source: Some(PackageManagerSource::Auto),
            effective_package_manager_available: true,
        }
    }
}
