use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

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

    pub fn plan_commands(
        &self,
        request: &str,
        destructive_substrings: &[String],
        preferred_editor: Option<&str>,
        environment: &ResolvedEnvironment,
        verbose: bool,
        output: &OutputStyler,
    ) -> Result<CommandPlan> {
        let url = format!(
            "{}/api/generate",
            self.config.base_url.trim_end_matches('/')
        );
        let request_body = GenerateRequest {
            model: &self.config.model,
            prompt: build_prompt(
                request,
                destructive_substrings,
                preferred_editor,
                environment,
            ),
            system: &self.config.system_prompt,
            stream: false,
            options: GenerateOptions {
                temperature: self.config.temperature,
            },
        };

        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim(&format!(
                    "[verbose] ollama generate url: {url}\n[verbose] ollama generate request:\n{}",
                    serde_json::to_string_pretty(&request_body)
                        .unwrap_or_else(|_| "<failed to serialize request>".to_string())
                ))
            );
        }

        let response = self
            .client
            .post(url)
            .json(&request_body)
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

        let json = extract_json_document(&response.response).with_context(|| {
            if verbose {
                format!(
                    "Ollama response did not contain a JSON object\nGenerated text:\n{}",
                    response.response
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
                    "failed to parse planner JSON returned by Ollama\nExtracted planner JSON:\n{json}\nFull generated text:\n{}",
                    response.response
                )
            } else {
                "failed to parse planner JSON returned by Ollama".to_string()
            }
        })?;

        if plan.commands.is_empty() {
            bail!("planner returned an empty command list")
        }

        Ok(plan)
    }

    pub fn check_service(&self, verbose: bool, output: &OutputStyler) -> Result<OllamaStatus> {
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

        let response = serde_json::from_str::<TagsResponse>(&response).with_context(|| {
            if verbose {
                format!("failed to decode Ollama tags response\nFull Ollama output:\n{response}")
            } else {
                "failed to decode Ollama tags response".to_string()
            }
        })?;

        Ok(OllamaStatus {
            model_available: response
                .models
                .iter()
                .any(|model| model.name == self.config.model),
        })
    }
}

pub struct OllamaStatus {
    pub model_available: bool,
}

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: String,
    system: &'a str,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Debug, Serialize)]
struct GenerateOptions {
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<TaggedModel>,
}

#[derive(Debug, Deserialize)]
struct TaggedModel {
    name: String,
}

fn build_prompt(
    request: &str,
    destructive_substrings: &[String],
    preferred_editor: Option<&str>,
    environment: &ResolvedEnvironment,
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

    format!(
        concat!(
            "Translate the user request into one or more shell commands. ",
            "Return JSON only with the schema ",
            r#"{{"summary":"short summary","commands":[{{"command":"...","description":"...","potentially_destructive":false,"recommended":true,"rationale":"..."}}]}}"#,
            ". If there are multiple plausible commands, include each one in the commands array. ",
            "Set potentially_destructive to true when the command could delete, overwrite, stop, or reconfigure something important. ",
            "When there are multiple command choices, mark the single best choice with recommended=true and set recommended=false for the others. ",
            "When there is only one command, set recommended=true. ",
            "Operating system: {os}. Linux distribution: {distro}. Detected package manager: {detected_package_manager}. Effective package manager: {effective_package_manager}. ",
            "For package-related requests such as listing installed packages, installing software, removing software, or searching package repositories, use commands appropriate for this environment and prefer the effective package manager. ",
            "Use the package manager's canonical syntax and only pass the package name as the package argument. Examples: `brew install btop`, `paru -S btop`, `pacman -Q`, `apt list --installed`. ",
            "Preferred terminal editor: {preferred_editor}. If the user asks to edit a file, prefer commands that open that editor. ",
            "Known destructive patterns: {destructive_examples}. ",
            "User request: {request}"
        ),
        destructive_examples = destructive_examples,
        os = environment.os.as_str(),
        distro = distro,
        detected_package_manager = detected_package_manager,
        effective_package_manager = effective_package_manager,
        preferred_editor = preferred_editor,
        request = request,
    )
}

fn extract_json_document(response: &str) -> Option<&str> {
    let start = response.find('{')?;
    let end = response.rfind('}')?;

    response.get(start..=end)
}

#[cfg(test)]
mod tests {
    use super::{GenerateRequest, build_prompt, extract_json_document};
    use crate::environment::{
        OperatingSystem, PackageManager, PackageManagerSource, ResolvedEnvironment,
    };

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
    fn prompt_requires_potentially_destructive_boolean() {
        let prompt = build_prompt(
            "delete the target directory",
            &["rm -rf".into()],
            None,
            &sample_environment(),
        );

        assert!(prompt.contains("\"potentially_destructive\":false"));
        assert!(prompt.contains("\"recommended\":true"));
        assert!(prompt.contains("Set potentially_destructive to true"));
        assert!(prompt.contains("mark the single best choice with recommended=true"));
    }

    #[test]
    fn prompt_includes_preferred_editor() {
        let prompt = build_prompt(
            "edit my git config file",
            &[],
            Some("nvim"),
            &sample_environment(),
        );

        assert!(prompt.contains("Preferred terminal editor: nvim"));
        assert!(prompt.contains("prefer commands that open that editor"));
    }

    #[test]
    fn prompt_includes_environment_context() {
        let prompt = build_prompt("install btop", &[], None, &sample_environment());

        assert!(prompt.contains("Operating system: linux"));
        assert!(prompt.contains("Linux distribution: arch"));
        assert!(prompt.contains("Detected package manager: paru"));
        assert!(prompt.contains("Effective package manager: paru"));
        assert!(prompt.contains("For package-related requests"));
    }

    #[test]
    fn generate_request_serializes_prompt_for_verbose_logging() {
        let request = GenerateRequest {
            model: "lfm2:latest",
            prompt: build_prompt("ping google", &[], Some("nvim"), &sample_environment()),
            system: "Return JSON only",
            stream: false,
            options: super::GenerateOptions { temperature: 0.0 },
        };

        let json = serde_json::to_string(&request).expect("request should serialize");

        assert!(json.contains("lfm2:latest"));
        assert!(json.contains("ping google"));
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
