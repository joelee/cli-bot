mod config;
mod llm;
mod output;
mod planner;
mod shell;

use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{io, io::IsTerminal};

use anyhow::{Context, Result, bail};
use clap::Parser;
use dialoguer::{Confirm, Input, Select};

use crate::config::{AppConfig, is_known_editor, resolve_config_path};
use crate::llm::OllamaClient;
pub use crate::output::{ColorMode, OutputStyler};
use crate::planner::{
    CommandPlan, PlannedCommand, command_requires_confirmation, recommended_command,
};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Translate natural language into shell commands"
)]
pub struct Cli {
    /// Natural-language request to translate into a shell command.
    pub request: Option<String>,

    /// Path to the cli-bot TOML configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Override the Ollama model from the configuration file.
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Verify config, Ollama connectivity, and editor availability.
    #[arg(short = 'C', long)]
    pub check: bool,

    /// Automatically use the LLM-recommended command when multiple choices are returned.
    #[arg(short = 'a', long)]
    pub auto_select_best: bool,

    /// Control ANSI color output: auto, always, or never.
    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorMode,

    /// Print the selected command without executing it.
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Print the raw structured plan returned by the planner.
    #[arg(short = 'p', long)]
    pub print_plan: bool,

    /// Print planner, execution, and total elapsed time in milliseconds.
    #[arg(short = 'b', long)]
    pub benchmark: bool,

    /// Print detailed actions and full Ollama responses for debugging.
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

pub fn run(cli: Cli) -> Result<()> {
    let total_start = Instant::now();
    let output = OutputStyler::new(cli.color.clone());
    let config_path = resolve_config_path(cli.config)?;
    let mut config = AppConfig::load(&config_path)?;
    apply_model_override(&mut config, cli.model.as_deref())?;

    if cli.verbose {
        eprintln!(
            "{}",
            output.stderr_dim(&format!("[verbose] config path: {}", config_path.display()))
        );
        eprintln!(
            "{}",
            output.stderr_dim(&format!(
                "[verbose] ollama base_url: {}",
                config.ollama.base_url
            ))
        );
        eprintln!(
            "{}",
            output.stderr_dim(&format!("[verbose] ollama model: {}", config.ollama.model))
        );
    }

    if cli.check {
        return run_check(config_path, config, cli.verbose, &output);
    }

    let planner = OllamaClient::new(config.ollama.clone());
    let preferred_editor = config.execution.resolved_preferred_editor();
    if cli.verbose {
        eprintln!(
            "{}",
            output.stderr_dim(&format!(
                "[verbose] preferred editor: {}",
                preferred_editor.as_deref().unwrap_or("<none>")
            ))
        );
    }
    let request = cli
        .request
        .as_deref()
        .map(str::to_owned)
        .map(normalize_request)
        .transpose()?
        .map_or_else(prompt_for_request, Ok)?;
    if cli.verbose {
        eprintln!(
            "{}",
            output.stderr_dim(&format!("[verbose] natural language request: {request}"))
        );
    }
    let planning_start = Instant::now();
    let plan = planner.plan_commands(
        &request,
        &config.safety.destructive_substrings,
        preferred_editor.as_deref(),
        cli.verbose,
        &output,
    )?;
    let planning_elapsed = planning_start.elapsed();

    if cli.print_plan {
        let plan_json = serde_json::to_string_pretty(&plan)?;
        println!("{plan_json}");
    }

    if cli.verbose
        && let Some(summary) = plan.summary.as_deref()
    {
        eprintln!(
            "{}",
            output.stderr_dim(&format!("[verbose] planner summary: {summary}"))
        );
    }

    let auto_select_best = config.ui.auto_select_recommended || cli.auto_select_best;
    let selected = select_command(&plan, &config.ui.selection_prompt, auto_select_best)?;

    if auto_select_best && plan.commands.len() > 1 && selected.recommended {
        println!(
            "{} {}",
            output.key("Recommended command:"),
            output.accent(&selected.command)
        );
    }

    if config.ui.show_command_before_execution {
        println!(
            "{} {}",
            output.key("Selected command:"),
            output.accent(&selected.command)
        );
    }

    if let Some(rationale) = selected.rationale.as_deref() {
        println!("{} {rationale}", output.dim("Why:"));
    }

    if cli.dry_run {
        if cli.benchmark {
            print_benchmark_report(
                &output,
                &config.ollama.model,
                planning_elapsed,
                None,
                total_start.elapsed(),
            );
        }
        return Ok(());
    }

    if command_requires_confirmation(selected, &config.safety) {
        let approved = Confirm::new()
            .with_prompt(format!(
                "{}\n{}",
                config.ui.approval_prompt, selected.command
            ))
            .default(false)
            .interact()
            .context("failed to capture confirmation from terminal")?;

        if !approved {
            println!("{}", output.warn("Command execution cancelled."));
            if cli.benchmark {
                print_benchmark_report(
                    &output,
                    &config.ollama.model,
                    planning_elapsed,
                    None,
                    total_start.elapsed(),
                );
            }
            return Ok(());
        }
    }

    let execution_elapsed = shell::execute(&selected.command, &config.execution)?;

    if cli.benchmark {
        print_benchmark_report(
            &output,
            &config.ollama.model,
            planning_elapsed,
            Some(execution_elapsed),
            total_start.elapsed(),
        );
    }

    Ok(())
}

fn run_check(
    config_path: PathBuf,
    config: AppConfig,
    verbose: bool,
    output: &OutputStyler,
) -> Result<()> {
    let mut failures = Vec::new();

    println!("{}", output.heading("Check results:"));
    println!(
        "{} {} ({})",
        output.key("config:"),
        output.ok("ok"),
        config_path.display()
    );

    let planner = OllamaClient::new(config.ollama.clone());
    match planner.check_service(verbose, output) {
        Ok(status) if status.model_available => {
            println!(
                "{} {} ({}; model `{}` available)",
                output.key("ollama:"),
                output.ok("ok"),
                config.ollama.base_url,
                config.ollama.model
            );
        }
        Ok(_) => {
            let message = format!(
                "service reachable at {}, but model `{}` is not available",
                config.ollama.base_url, config.ollama.model
            );
            println!(
                "{} {} ({message})",
                output.key("ollama:"),
                output.error("error")
            );
            failures.push(message);
        }
        Err(error) => {
            let message = format!("failed to reach {}: {error:#}", config.ollama.base_url);
            println!(
                "{} {} ({message})",
                output.key("ollama:"),
                output.error("error")
            );
            failures.push(message);
        }
    }

    let resolved_editor = config.execution.resolved_preferred_editor();
    if verbose {
        eprintln!(
            "{}",
            output.stderr_dim(&format!(
                "[verbose] preferred editor for check: {}",
                resolved_editor.as_deref().unwrap_or("<none>")
            ))
        );
    }

    match resolved_editor {
        Some(editor) if is_known_editor(&editor) => {
            println!("{} {} ({editor})", output.key("editor:"), output.ok("ok"));
        }
        Some(editor) => {
            let message = format!("`{editor}` is not available on PATH");
            println!(
                "{} {} ({message})",
                output.key("editor:"),
                output.error("error")
            );
            failures.push(message);
        }
        None => {
            let message = "no preferred editor configured and $EDITOR is not set".to_string();
            println!(
                "{} {} ({message})",
                output.key("editor:"),
                output.error("error")
            );
            failures.push(message);
        }
    }

    if failures.is_empty() {
        println!("{} {}", output.key("check:"), output.ok("ok"));
        Ok(())
    } else {
        bail!("environment check failed")
    }
}

fn apply_model_override(config: &mut AppConfig, model_override: Option<&str>) -> Result<()> {
    if let Some(model_override) = model_override {
        let model_override = model_override.trim();

        if model_override.is_empty() {
            bail!("--model must not be empty")
        }

        config.ollama.model = model_override.to_string();
    }

    Ok(())
}

fn prompt_for_request() -> Result<String> {
    if !io::stdin().is_terminal() {
        let mut request = String::new();
        io::stdin()
            .read_line(&mut request)
            .context("failed to read request from stdin")?;

        return normalize_request(request);
    }

    Input::<String>::new()
        .with_prompt("What would you like cli-bot to do?")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.trim().is_empty() {
                Err("request must not be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .context("failed to capture request from terminal")
        .and_then(normalize_request)
}

fn normalize_request(request: String) -> Result<String> {
    let request = request.trim();

    if request.is_empty() {
        bail!("request must not be empty")
    }

    Ok(request.to_string())
}

fn select_command<'a>(
    plan: &'a CommandPlan,
    prompt: &str,
    auto_select_best: bool,
) -> Result<&'a PlannedCommand> {
    if plan.commands.is_empty() {
        bail!("planner returned no commands")
    }

    if plan.commands.len() == 1 {
        return Ok(&plan.commands[0]);
    }

    if auto_select_best && let Some(command) = recommended_command(plan) {
        return Ok(command);
    }

    let items = plan
        .commands
        .iter()
        .map(PlannedCommand::label)
        .collect::<Vec<_>>();
    let selection = Select::new()
        .with_prompt(prompt)
        .items(&items)
        .default(0)
        .interact()
        .context("failed to capture command selection from terminal")?;

    Ok(&plan.commands[selection])
}

fn print_benchmark_report(
    output: &OutputStyler,
    model: &str,
    planning: Duration,
    execution: Option<Duration>,
    total: Duration,
) {
    println!("{}", output.heading("Benchmark:"));
    println!("{} {}", output.key("model:"), output.accent(model));
    println!(
        "{} {}",
        output.key("planning_ms:"),
        duration_to_ms(planning)
    );

    match execution {
        Some(execution) => println!(
            "{} {}",
            output.key("execution_ms:"),
            duration_to_ms(execution)
        ),
        None => println!("{} skipped", output.key("execution_ms:")),
    }

    println!("{} {}", output.key("total_ms:"), duration_to_ms(total));
}

fn duration_to_ms(duration: Duration) -> u128 {
    duration.as_millis()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{apply_model_override, duration_to_ms, normalize_request, select_command};
    use crate::config::{AppConfig, ExecutionConfig, OllamaConfig, SafetyConfig, UiConfig};
    use crate::planner::{CommandPlan, PlannedCommand};

    #[test]
    fn converts_duration_to_milliseconds() {
        assert_eq!(duration_to_ms(Duration::from_millis(42)), 42);
        assert_eq!(duration_to_ms(Duration::from_micros(999)), 0);
    }

    #[test]
    fn applies_model_override() {
        let mut config = sample_config();

        apply_model_override(&mut config, Some("lfm2:latest")).expect("override should apply");

        assert_eq!(config.ollama.model, "lfm2:latest");
    }

    #[test]
    fn rejects_blank_model_override() {
        let mut config = sample_config();

        let error =
            apply_model_override(&mut config, Some("  ")).expect_err("blank override should fail");

        assert!(error.to_string().contains("--model must not be empty"));
    }

    #[test]
    fn keeps_existing_model_when_override_missing() {
        let mut config = sample_config();

        apply_model_override(&mut config, None).expect("missing override should be ignored");

        assert_eq!(config.ollama.model, "default-model");
    }

    #[test]
    fn auto_selects_recommended_command_when_enabled() {
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

        let selected = select_command(&plan, "Choose", true).expect("selection should succeed");

        assert_eq!(selected.command, "ping -c 5 google.com");
    }

    #[test]
    fn normalizes_request_by_trimming_whitespace() {
        let request = normalize_request("  Ping google five times  ".into())
            .expect("request should normalize");

        assert_eq!(request, "Ping google five times");
    }

    #[test]
    fn rejects_blank_request() {
        let error = normalize_request("   ".into()).expect_err("blank request should fail");

        assert!(error.to_string().contains("request must not be empty"));
    }

    fn sample_config() -> AppConfig {
        AppConfig {
            ollama: OllamaConfig {
                base_url: "http://127.0.0.1:11434".into(),
                model: "default-model".into(),
                temperature: 0.0,
                system_prompt: "Return JSON only".into(),
            },
            safety: SafetyConfig {
                require_confirmation: true,
                destructive_substrings: vec![],
            },
            ui: UiConfig {
                selection_prompt: "Choose".into(),
                approval_prompt: "Approve?".into(),
                show_command_before_execution: true,
                auto_select_recommended: false,
            },
            execution: ExecutionConfig {
                shell: "/bin/sh".into(),
                shell_arg: "-c".into(),
                preferred_editor: Some("nvim".into()),
            },
        }
    }
}
