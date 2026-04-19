mod config;
mod environment;
mod llm;
mod output;
mod planner;
mod session;
mod shell;

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{env, io, io::IsTerminal};
use std::{fmt::Write as _, fs};

use anyhow::{Context, Result, bail};
use clap::Parser;
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};

use crate::config::{AppConfig, ModelsBenchmarkConfig, is_known_editor, resolve_config_path};
use crate::environment::{PackageManagerSource, resolve_environment};
use crate::llm::{OllamaBenchmarkMetadata, OllamaClient};
pub use crate::output::{ColorMode, OutputStyler};
use crate::planner::{
    CommandPlan, PlannedCommand, command_requires_confirmation, recommended_command,
};
use crate::session::{SessionExecution, SessionStore, SessionTurn};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Translate natural language into shell commands"
)]
pub struct Cli {
    /// Natural-language request to translate into a shell command.
    pub request: Vec<String>,

    /// Path to the cli-bot TOML configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Override the Ollama model from the configuration file.
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Verify config, Ollama connectivity, and editor availability.
    #[arg(short = 'C', long)]
    pub check: bool,

    /// Benchmark configured models across configured queries and print Markdown to stdout or an optional file.
    #[arg(long, value_name = "FILE", num_args = 0..=1, default_missing_value = "-")]
    pub models_benchmark: Option<PathBuf>,

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

    /// Hide cli-bot informational output and only show the selected command's output.
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Keep prompting for new requests until Ctrl-C or /quit.
    #[arg(short = 'i', long)]
    pub interactive: bool,

    /// Use a named session, or the configured default when omitted.
    #[arg(long, value_name = "NAME", num_args = 0..=1, default_missing_value = "")]
    pub session: Option<String>,

    /// Disable session memory for this invocation.
    #[arg(long)]
    pub no_session: bool,

    /// Print the current session turns and exit.
    #[arg(long)]
    pub session_show: bool,

    /// List locally stored sessions and exit.
    #[arg(long)]
    pub session_list: bool,

    /// Clear the current session and exit.
    #[arg(long)]
    pub session_clear: bool,
}

pub fn run(cli: Cli) -> Result<()> {
    let total_start = Instant::now();
    let output = OutputStyler::new(cli.color.clone());
    let show_output = !cli.quiet;
    let verbose = cli.verbose && show_output;
    let config_path = resolve_config_path(cli.config.clone())?;
    let mut config = AppConfig::load(&config_path)?;
    apply_model_override(&mut config, cli.model.as_deref())?;
    let resolved_environment = resolve_environment(&config.environment)?;
    let session_store = SessionStore::new(config.session_memory.clone())?;
    let pruned_sessions = session_store.prune_expired()?;
    let session_requested = !cli.no_session && session_store.enabled();
    let session_name = if session_requested {
        cli.session.as_deref().and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
    } else {
        None
    };

    if verbose {
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
        eprintln!(
            "{}",
            output.stderr_dim(&format!(
                "[verbose] environment os: {}",
                resolved_environment.os.as_str()
            ))
        );
        eprintln!(
            "{}",
            output.stderr_dim(&format!(
                "[verbose] environment distro: {}",
                resolved_environment
                    .distro
                    .map(|distro| distro.as_str())
                    .unwrap_or("not applicable")
            ))
        );
        eprintln!(
            "{}",
            output.stderr_dim(&format!(
                "[verbose] detected package manager: {}",
                resolved_environment
                    .detected_package_manager
                    .map(|package_manager| package_manager.as_str())
                    .unwrap_or("unknown")
            ))
        );
        eprintln!(
            "{}",
            output.stderr_dim(&format!(
                "[verbose] effective package manager: {}",
                resolved_environment
                    .effective_package_manager
                    .map(|package_manager| package_manager.as_str())
                    .unwrap_or("unknown")
            ))
        );
        eprintln!(
            "{}",
            output.stderr_dim(&format!(
                "[verbose] session memory: {}",
                if session_requested {
                    "enabled"
                } else {
                    "disabled"
                }
            ))
        );
        if session_requested {
            eprintln!(
                "{}",
                output.stderr_dim(&format!(
                    "[verbose] session name: {}",
                    session_name.unwrap_or(session_store.default_name())
                ))
            );
        }
        if pruned_sessions > 0 {
            eprintln!(
                "{}",
                output.stderr_dim(&format!(
                    "[verbose] pruned {pruned_sessions} expired sessions"
                ))
            );
        }
    }

    if cli.check {
        return run_check(
            config_path,
            config,
            resolved_environment,
            verbose,
            show_output,
            &output,
        );
    }

    if cli.session_show || cli.session_clear || cli.session_list {
        return handle_session_command(
            &cli,
            session_requested,
            session_name,
            &session_store,
            show_output,
            &output,
        );
    }

    if let Some(benchmark_output) = cli.models_benchmark.clone() {
        let benchmark_output = if benchmark_output_is_stdout(&benchmark_output) {
            resolve_request(&cli.request)?
                .map(PathBuf::from)
                .unwrap_or(benchmark_output)
        } else {
            benchmark_output
        };

        return run_models_benchmark(
            config,
            resolved_environment,
            verbose,
            &output,
            benchmark_output,
        );
    }

    let planner = OllamaClient::new(config.ollama.clone());
    let preferred_editor = config.execution.resolved_preferred_editor();
    if verbose {
        eprintln!(
            "{}",
            output.stderr_dim(&format!(
                "[verbose] preferred editor: {}",
                preferred_editor.as_deref().unwrap_or("<none>")
            ))
        );
    }
    if cli.interactive {
        let mut next_request = resolve_request(&cli.request)?;
        let mut prompt_in_error_state = false;

        if show_output {
            println!(
                "Enter {} or press {} to exit Interactive Mode.",
                output.accent("'/quit'"),
                output.accent("'Ctrl-C'")
            );
            println!();
        }

        loop {
            let request = match next_request.take() {
                Some(request) => request,
                None => match prompt_for_request(prompt_in_error_state) {
                    Ok(request) => request,
                    Err(error) if interactive_prompt_cancelled(&error) => return Ok(()),
                    Err(error) => return Err(error),
                },
            };

            if should_quit_interactive(&request) {
                return Ok(());
            }

            let request_result = run_single_request(
                &cli,
                &config,
                &resolved_environment,
                &session_store,
                session_requested,
                session_name,
                &planner,
                preferred_editor.as_deref(),
                &request,
                total_start,
                show_output,
                verbose,
                &output,
            );

            match request_result {
                Ok(()) => {
                    prompt_in_error_state = false;
                }
                Err(error) => {
                    handle_interactive_request_error(&error, show_output, &output)?;
                    prompt_in_error_state = true;
                }
            }

            if show_output {
                println!();
            }
        }
    }

    let request = if cli.request.is_empty() {
        prompt_for_request(false)?
    } else {
        resolve_request(&cli.request)?.expect("request parts should resolve when non-empty")
    };

    run_single_request(
        &cli,
        &config,
        &resolved_environment,
        &session_store,
        session_requested,
        session_name,
        &planner,
        preferred_editor.as_deref(),
        &request,
        total_start,
        show_output,
        verbose,
        &output,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_single_request(
    cli: &Cli,
    config: &AppConfig,
    resolved_environment: &crate::environment::ResolvedEnvironment,
    session_store: &SessionStore,
    session_requested: bool,
    session_name: Option<&str>,
    planner: &OllamaClient,
    preferred_editor: Option<&str>,
    request: &str,
    total_start: Instant,
    show_output: bool,
    verbose: bool,
    output: &OutputStyler,
) -> Result<()> {
    let mut session_record = if session_requested {
        Some(session_store.load(session_name)?)
    } else {
        None
    };
    let session_context = session_record
        .as_ref()
        .and_then(|record| session_store.render_prompt_context(record));
    if verbose {
        eprintln!(
            "{}",
            output.stderr_dim(&format!("[verbose] natural language request: {request}"))
        );
        if let Some(session_context) = session_context.as_deref() {
            eprintln!(
                "{}",
                output.stderr_dim(&format!("[verbose] session context:\n{session_context}"))
            );
        }
    }

    let planning_start = Instant::now();
    let plan = planner.plan_commands(
        request,
        &config.safety.destructive_substrings,
        preferred_editor,
        resolved_environment,
        session_context.as_deref(),
        verbose,
        output,
    )?;
    let mut planning_elapsed = planning_start.elapsed();

    if cli.print_plan && show_output {
        let plan_json = serde_json::to_string_pretty(&plan)?;
        println!("{plan_json}");
    }

    if verbose && let Some(summary) = plan.summary.as_deref() {
        eprintln!(
            "{}",
            output.stderr_dim(&format!("[verbose] planner summary: {summary}"))
        );
    }

    if plan.unresolved {
        if verbose {
            eprintln!(
                "{}",
                output.stderr_dim("[verbose] planner marked request as unresolved; requesting text response fallback")
            );
        }
        let response_start = Instant::now();
        let text_response = planner.answer_unresolved(
            request,
            preferred_editor,
            resolved_environment,
            session_context.as_deref(),
            verbose,
            output,
        )?;
        planning_elapsed += response_start.elapsed();

        if let Some(record) = session_record.as_mut()
            && config.session_memory.save_text_responses
        {
            let turn = SessionTurn::unresolved_text(
                request,
                &text_response,
                config.session_memory.include_working_directory,
                &std::env::current_dir().context("failed to resolve current working directory")?,
            );
            record.push_turn(turn);
            session_store.save(record)?;
        }

        println!("{text_response}");

        if cli.benchmark && show_output {
            print_benchmark_report(
                output,
                &config.ollama.model,
                planning_elapsed,
                None,
                total_start.elapsed(),
            );
        }

        return Ok(());
    }

    let auto_select_best = config.ui.auto_select_recommended || cli.auto_select_best;
    let selected = select_command(&plan, &config.ui.selection_prompt, auto_select_best)?;
    let mut session_turn = session_record.as_ref().map(|_| {
        SessionTurn::from_plan(
            request,
            &plan,
            config.session_memory.include_working_directory,
            &std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        )
    });

    if show_output && auto_select_best && plan.commands.len() > 1 && selected.recommended {
        println!(
            "{} {}",
            output.key("Recommended command:"),
            output.accent(&selected.command)
        );
    }

    if show_output && config.ui.show_command_before_execution {
        println!(
            "{} {}",
            output.key("Selected command:"),
            output.accent(&selected.command)
        );
    }

    if show_output && let Some(rationale) = selected.rationale.as_deref() {
        println!("{} {rationale}", output.dim("Why:"));
    }

    if cli.dry_run {
        if let (Some(record), Some(turn)) = (session_record.as_mut(), session_turn.as_mut())
            && config.session_memory.save_selected_commands
        {
            turn.selected_command = Some(selected.command.clone());
            turn.selected_command_rationale = selected.rationale.clone();
            turn.confirmation_required = command_requires_confirmation(selected, &config.safety);
            turn.execution = Some(SessionExecution {
                executed: false,
                exit_status: None,
                stdout: None,
                stderr: None,
            });
            record.push_turn(turn.clone());
            session_store.save(record)?;
        }
        if cli.benchmark && show_output {
            print_benchmark_report(
                output,
                &config.ollama.model,
                planning_elapsed,
                None,
                total_start.elapsed(),
            );
        }
        return Ok(());
    }

    if command_requires_confirmation(selected, &config.safety) {
        if let Some(turn) = session_turn.as_mut() {
            turn.confirmation_required = true;
        }
        let approved = Confirm::new()
            .with_prompt(format!(
                "{}\n{}",
                config.ui.approval_prompt, selected.command
            ))
            .default(false)
            .interact()
            .context("failed to capture confirmation from terminal")?;

        if !approved {
            if show_output {
                println!("{}", output.warn("Command execution cancelled."));
            }
            if cli.benchmark && show_output {
                print_benchmark_report(
                    output,
                    &config.ollama.model,
                    planning_elapsed,
                    None,
                    total_start.elapsed(),
                );
            }
            return Ok(());
        }
    }

    let execution_result = shell::execute(
        &selected.command,
        &config.execution,
        session_store.should_capture_command_output(),
        config.session_memory.max_output_bytes,
    )?;
    let execution_elapsed = execution_result.duration;

    if let (Some(record), Some(turn)) = (session_record.as_mut(), session_turn.as_mut())
        && config.session_memory.save_selected_commands
    {
        turn.selected_command = Some(selected.command.clone());
        turn.selected_command_rationale = selected.rationale.clone();
        turn.execution = Some(SessionExecution {
            executed: true,
            exit_status: execution_result.exit_status,
            stdout: execution_result.stdout,
            stderr: execution_result.stderr,
        });
        record.push_turn(turn.clone());
        session_store.save(record)?;
    }

    if cli.benchmark && show_output {
        print_benchmark_report(
            output,
            &config.ollama.model,
            planning_elapsed,
            Some(execution_elapsed),
            total_start.elapsed(),
        );
    }

    Ok(())
}

fn handle_session_command(
    cli: &Cli,
    session_requested: bool,
    session_name: Option<&str>,
    session_store: &SessionStore,
    show_output: bool,
    output: &OutputStyler,
) -> Result<()> {
    if !session_requested {
        bail!(
            "session memory is disabled for this invocation; remove --no-session or enable [session_memory].enabled"
        )
    }

    if cli.session_show && cli.session_clear {
        bail!("--session-show and --session-clear cannot be used together")
    }

    if cli.session_list {
        let entries = session_store.list()?;
        if show_output {
            if entries.is_empty() {
                println!("{}", output.dim("No stored sessions."));
            } else {
                for entry in entries {
                    println!("{} {}", output.key("Session:"), output.accent(&entry.name));
                    println!("{} {}", output.dim("scoped_name:"), entry.scoped_name);
                    println!("{} {}", output.dim("scope:"), entry.scope.as_str());
                    println!("{} {}", output.dim("turns:"), entry.turns);
                    println!("{} {}", output.dim("path:"), entry.path.display());
                }
            }
        }
        return Ok(());
    }

    if cli.session_show {
        let record = session_store.load(session_name)?;
        if show_output {
            println!("{} {}", output.key("Session:"), output.accent(&record.name));
            println!("{} {}", output.key("Turns:"), record.turns.len());
            println!("{} {}", output.key("Scope:"), record.scope.as_str());
            if record.turns.is_empty() {
                println!("{}", output.dim("No stored turns."));
            } else {
                for (index, turn) in record.turns.iter().enumerate() {
                    println!("{} {}", output.key("Turn"), index + 1);
                    println!("{} {}", output.dim("request:"), turn.request);
                    if let Some(selected_command) = turn.selected_command.as_deref() {
                        println!("{} {}", output.dim("selected_command:"), selected_command);
                    }
                    if let Some(text_response) = turn.text_response.as_deref() {
                        println!("{} {}", output.dim("text_response:"), text_response);
                    }
                }
            }
        }
        return Ok(());
    }

    if cli.session_clear {
        let removed = session_store.clear(session_name)?;
        if show_output {
            if removed {
                println!("{}", output.ok("Session cleared."));
            } else {
                println!("{}", output.warn("Session did not exist."));
            }
        }
    }

    Ok(())
}

fn run_check(
    config_path: PathBuf,
    config: AppConfig,
    resolved_environment: crate::environment::ResolvedEnvironment,
    verbose: bool,
    show_output: bool,
    output: &OutputStyler,
) -> Result<()> {
    let mut failures = Vec::new();
    let terminal = terminal_environment_status();

    if show_output {
        println!("{}", output.heading("Check results:"));
        println!(
            "{} {} ({})",
            output.key("config:"),
            output.ok("ok"),
            config_path.display()
        );
    }

    let planner = OllamaClient::new(config.ollama.clone());
    match planner.check_service(verbose, output) {
        Ok(status) if status.model_available => {
            if show_output {
                let version = status.version.as_deref().unwrap_or("unknown version");
                println!(
                    "{} {} ({}; model `{}` available; Ollama {version})",
                    output.key("ollama:"),
                    output.ok("ok"),
                    config.ollama.base_url,
                    config.ollama.model
                );
            }
        }
        Ok(_) => {
            let message = format!(
                "service reachable at {}, but model `{}` is not available",
                config.ollama.base_url, config.ollama.model
            );
            if show_output {
                println!(
                    "{} {} ({message})",
                    output.key("ollama:"),
                    output.error("error")
                );
            }
            failures.push(message);
        }
        Err(error) => {
            let message = format!("failed to reach {}: {error:#}", config.ollama.base_url);
            if show_output {
                println!(
                    "{} {} ({message})",
                    output.key("ollama:"),
                    output.error("error")
                );
            }
            failures.push(message);
        }
    }

    if show_output {
        println!(
            "{} {} ({})",
            output.key("os:"),
            output.ok("ok"),
            resolved_environment.os.as_str()
        );

        match resolved_environment.distro {
            Some(distro) => println!(
                "{} {} ({})",
                output.key("distro:"),
                if distro.as_str() == "unknown" {
                    output.warn("warn")
                } else {
                    output.ok("ok")
                },
                distro.as_str()
            ),
            None => println!(
                "{} {} (not applicable)",
                output.key("distro:"),
                output.ok("ok")
            ),
        }

        match resolved_environment.detected_package_manager {
            Some(package_manager) if package_manager.as_str() != "unknown" => println!(
                "{} {} ({})",
                output.key("package_manager_detected:"),
                output.ok("ok"),
                package_manager.as_str()
            ),
            _ => println!(
                "{} {} (unknown)",
                output.key("package_manager_detected:"),
                output.warn("warn")
            ),
        }
    }

    match resolved_environment.effective_package_manager {
        Some(package_manager)
            if resolved_environment.effective_package_manager_available
                || package_manager.as_str() == "unknown" =>
        {
            if show_output {
                let source = resolved_environment
                    .package_manager_source
                    .map(PackageManagerSource::as_str)
                    .unwrap_or("unknown");
                let tone = if package_manager.as_str() == "unknown" {
                    output.warn("warn")
                } else {
                    output.ok("ok")
                };

                println!(
                    "{} {} ({}; {})",
                    output.key("package_manager_effective:"),
                    tone,
                    package_manager.as_str(),
                    source
                );
            }
        }
        Some(package_manager) => {
            let message = format!(
                "`{}` selected as the effective package manager but it is not available on PATH",
                package_manager.as_str()
            );
            if show_output {
                println!(
                    "{} {} ({message})",
                    output.key("package_manager_effective:"),
                    output.error("error")
                );
            }
            failures.push(message);
        }
        None => {
            if show_output {
                println!(
                    "{} {} (unknown)",
                    output.key("package_manager_effective:"),
                    output.warn("warn")
                );
            }
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
            if show_output {
                println!("{} {} ({editor})", output.key("editor:"), output.ok("ok"));
            }
        }
        Some(editor) => {
            let message = format!("`{editor}` is not available on PATH");
            if show_output {
                println!(
                    "{} {} ({message})",
                    output.key("editor:"),
                    output.error("error")
                );
            }
            failures.push(message);
        }
        None => {
            let message = "no preferred editor configured and $EDITOR is not set".to_string();
            if show_output {
                println!(
                    "{} {} ({message})",
                    output.key("editor:"),
                    output.error("error")
                );
            }
            failures.push(message);
        }
    }

    if show_output {
        let tone = if terminal.interactive_dialogs_supported() {
            output.ok("ok")
        } else {
            output.warn("warn")
        };
        println!(
            "{} {} ({})",
            output.key("terminal:"),
            tone,
            terminal.describe()
        );
    }

    if failures.is_empty() {
        if show_output {
            println!("{} {}", output.key("check:"), output.ok("ok"));
        }
        Ok(())
    } else {
        bail!("environment check failed")
    }
}

fn run_models_benchmark(
    config: AppConfig,
    resolved_environment: crate::environment::ResolvedEnvironment,
    verbose: bool,
    output: &OutputStyler,
    output_path: PathBuf,
) -> Result<()> {
    if config.models_benchmark.models.is_empty() {
        bail!("models_benchmark.models is empty in cli-bot.toml")
    }

    if config.models_benchmark.queries.is_empty() {
        bail!("models_benchmark.queries is empty in cli-bot.toml")
    }

    let preferred_editor = config.execution.resolved_preferred_editor();
    let host_info = collect_benchmark_host_info();
    let metadata_client = OllamaClient::new(config.ollama.clone());
    let ollama_metadata = metadata_client.benchmark_metadata(verbose, output)?;
    let mut results = Vec::new();

    for model in &config.models_benchmark.models {
        let mut ollama_config = config.ollama.clone();
        ollama_config.model = model.clone();
        let client = OllamaClient::new(ollama_config);

        for query in &config.models_benchmark.queries {
            let planner_start = Instant::now();
            let result = client.plan_commands(
                query,
                &config.safety.destructive_substrings,
                preferred_editor.as_deref(),
                &resolved_environment,
                None,
                verbose,
                output,
            );
            let planner_elapsed = planner_start.elapsed();

            let entry = match result {
                Ok(plan) if plan.unresolved => {
                    let fallback_start = Instant::now();
                    let response = client.answer_unresolved(
                        query,
                        preferred_editor.as_deref(),
                        &resolved_environment,
                        None,
                        verbose,
                        output,
                    );
                    let fallback_elapsed = fallback_start.elapsed();

                    match response {
                        Ok(response) => ModelBenchmarkResult {
                            model: model.clone(),
                            query: query.clone(),
                            planner_ms: duration_to_ms(planner_elapsed),
                            fallback_ms: Some(duration_to_ms(fallback_elapsed)),
                            total_ms: duration_to_ms(planner_elapsed + fallback_elapsed),
                            unresolved: true,
                            response_kind: "text_response".to_string(),
                            response: response.trim().to_string(),
                            error: None,
                        },
                        Err(error) => ModelBenchmarkResult {
                            model: model.clone(),
                            query: query.clone(),
                            planner_ms: duration_to_ms(planner_elapsed),
                            fallback_ms: Some(duration_to_ms(fallback_elapsed)),
                            total_ms: duration_to_ms(planner_elapsed + fallback_elapsed),
                            unresolved: true,
                            response_kind: "error".to_string(),
                            response: String::new(),
                            error: Some(format!("{error:#}")),
                        },
                    }
                }
                Ok(plan) => ModelBenchmarkResult {
                    model: model.clone(),
                    query: query.clone(),
                    planner_ms: duration_to_ms(planner_elapsed),
                    fallback_ms: None,
                    total_ms: duration_to_ms(planner_elapsed),
                    unresolved: false,
                    response_kind: "command_plan".to_string(),
                    response: format_command_plan_response(&plan),
                    error: None,
                },
                Err(error) => ModelBenchmarkResult {
                    model: model.clone(),
                    query: query.clone(),
                    planner_ms: duration_to_ms(planner_elapsed),
                    fallback_ms: None,
                    total_ms: duration_to_ms(planner_elapsed),
                    unresolved: false,
                    response_kind: "error".to_string(),
                    response: String::new(),
                    error: Some(format!("{error:#}")),
                },
            };

            results.push(entry);
        }
    }

    let report = render_models_benchmark_markdown(
        &host_info,
        &ollama_metadata,
        &config.models_benchmark,
        &results,
    );

    if benchmark_output_is_stdout(&output_path) {
        print!("{report}");
    } else {
        fs::write(&output_path, report).with_context(|| {
            format!(
                "failed to write models benchmark report to `{}`",
                output_path.display()
            )
        })?;
    }

    Ok(())
}

fn format_command_plan_response(plan: &CommandPlan) -> String {
    let mut lines = Vec::new();

    if let Some(summary) = plan.summary.as_deref() {
        lines.push(format!("summary: {summary}"));
    }

    for command in &plan.commands {
        let recommended = if command.recommended {
            " [recommended]"
        } else {
            ""
        };
        lines.push(format!("{}{}", command.command, recommended));

        if let Some(rationale) = command.rationale.as_deref() {
            lines.push(format!("why: {rationale}"));
        }
    }

    lines.join("\n")
}

fn render_models_benchmark_markdown(
    host_info: &BenchmarkHostInfo,
    ollama_metadata: &OllamaBenchmarkMetadata,
    benchmark_config: &ModelsBenchmarkConfig,
    results: &[ModelBenchmarkResult],
) -> String {
    let mut report = String::new();
    writeln!(report, "# Model Benchmark Report\n").ok();
    writeln!(report, "## Host\n").ok();
    writeln!(report, "- Hostname: {}", host_info.hostname).ok();
    writeln!(report, "- OS: {}", host_info.os).ok();
    writeln!(report, "- Kernel: {}", host_info.kernel).ok();
    writeln!(report, "- CPU: {}", host_info.cpu).ok();
    writeln!(report, "- GPU: {}", host_info.gpu).ok();
    writeln!(report, "- GPU VRAM: {}", host_info.gpu_vram).ok();
    writeln!(report, "- Memory: {}", host_info.memory).ok();
    writeln!(
        report,
        "- Ollama Version: {}\n",
        ollama_metadata.version.as_deref().unwrap_or("unknown")
    )
    .ok();

    writeln!(report, "## Models\n").ok();
    writeln!(report, "| Model | Parameter Size |").ok();
    writeln!(report, "| --- | --- |").ok();
    for model in &benchmark_config.models {
        writeln!(
            report,
            "| {} | {} |",
            escape_markdown_cell(model),
            ollama_metadata
                .model_parameter_sizes
                .get(model)
                .map(String::as_str)
                .unwrap_or("unknown")
        )
        .ok();
    }
    writeln!(report).ok();

    render_models_benchmark_summary_table(&mut report, benchmark_config, results);
    render_models_benchmark_model_summary(&mut report, ollama_metadata, benchmark_config, results);
    render_models_benchmark_ranking(&mut report, ollama_metadata, benchmark_config, results);

    writeln!(report, "## Detailed Results\n").ok();
    writeln!(
        report,
        "| Model | Query | Planner ms | Fallback ms | Total ms | Kind | Unresolved | Status |"
    )
    .ok();
    writeln!(
        report,
        "| --- | --- | ---: | ---: | ---: | --- | --- | --- |"
    )
    .ok();

    for query in &benchmark_config.queries {
        for result in results.iter().filter(|result| result.query == *query) {
            writeln!(
                report,
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                escape_markdown_cell(&result.model),
                escape_markdown_cell(&result.query),
                result.planner_ms,
                result
                    .fallback_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                result.total_ms,
                result.response_kind,
                if result.unresolved { "yes" } else { "no" },
                if result.error.is_some() {
                    "error"
                } else {
                    "ok"
                },
            )
            .ok();
        }
    }

    writeln!(report).ok();

    for query in &benchmark_config.queries {
        writeln!(report, "### {}\n", query).ok();

        for result in results.iter().filter(|result| result.query == *query) {
            writeln!(report, "#### {}\n", result.model).ok();
            writeln!(report, "- Planner ms: {}", result.planner_ms).ok();
            match result.fallback_ms {
                Some(fallback_ms) => writeln!(report, "- Fallback ms: {fallback_ms}").ok(),
                None => writeln!(report, "- Fallback ms: not used").ok(),
            };
            writeln!(report, "- Total ms: {}", result.total_ms).ok();
            writeln!(report, "- Kind: {}", result.response_kind).ok();
            writeln!(
                report,
                "- Unresolved: {}",
                if result.unresolved { "yes" } else { "no" }
            )
            .ok();

            if let Some(error) = result.error.as_deref() {
                writeln!(report, "- Status: error\n").ok();
                writeln!(report, "```text\n{}\n```\n", error.trim()).ok();
            } else {
                writeln!(report, "- Status: ok\n").ok();
                writeln!(report, "```text\n{}\n```\n", result.response.trim()).ok();
            }
        }

        writeln!(report).ok();
    }

    report
}

fn render_models_benchmark_summary_table(
    report: &mut String,
    benchmark_config: &ModelsBenchmarkConfig,
    results: &[ModelBenchmarkResult],
) {
    writeln!(report, "## Summary Table\n").ok();
    let mut header = String::from("| Query |");
    let mut separator = String::from("| --- |");

    for model in &benchmark_config.models {
        header.push_str(&format!(" {} |", escape_markdown_cell(model)));
        separator.push_str(" ---: |");
    }

    writeln!(report, "{header}").ok();
    writeln!(report, "{separator}").ok();

    for query in &benchmark_config.queries {
        let mut row = format!("| {} |", escape_markdown_cell(query));

        for model in &benchmark_config.models {
            let cell = results
                .iter()
                .find(|result| result.query == *query && result.model == *model)
                .map(|result| {
                    if result.error.is_some() {
                        "failed".to_string()
                    } else {
                        result.total_ms.to_string()
                    }
                })
                .unwrap_or_else(|| "failed".to_string());

            row.push_str(&format!(" {} |", cell));
        }

        writeln!(report, "{row}").ok();
    }

    writeln!(report).ok();
}

fn render_models_benchmark_model_summary(
    report: &mut String,
    ollama_metadata: &OllamaBenchmarkMetadata,
    benchmark_config: &ModelsBenchmarkConfig,
    results: &[ModelBenchmarkResult],
) {
    writeln!(report, "## Model Summary\n").ok();
    writeln!(
        report,
        "| Model | Parameter Size | Success Rate | Avg Total ms (ok) | Successful Queries | Failed Queries |"
    )
    .ok();
    writeln!(report, "| --- | --- | ---: | ---: | ---: | ---: |").ok();

    for model in &benchmark_config.models {
        let stats = compute_model_benchmark_stats(model, results);
        writeln!(
            report,
            "| {} | {} | {} | {} | {} | {} |",
            escape_markdown_cell(model),
            ollama_metadata
                .model_parameter_sizes
                .get(model)
                .map(String::as_str)
                .unwrap_or("unknown"),
            format_success_rate(stats.success_count, stats.total_count),
            stats
                .avg_total_ms_ok
                .map(|value| value.to_string())
                .unwrap_or_else(|| "failed".to_string()),
            stats.success_count,
            stats.failure_count,
        )
        .ok();
    }

    writeln!(report).ok();
}

fn render_models_benchmark_ranking(
    report: &mut String,
    ollama_metadata: &OllamaBenchmarkMetadata,
    benchmark_config: &ModelsBenchmarkConfig,
    results: &[ModelBenchmarkResult],
) {
    writeln!(report, "## Ranking\n").ok();
    writeln!(
        report,
        "Ranked by success rate first, then by average total milliseconds across successful queries.\n"
    )
    .ok();
    writeln!(
        report,
        "| Rank | Model | Parameter Size | Success Rate | Avg Total ms (ok) |"
    )
    .ok();
    writeln!(report, "| ---: | --- | --- | ---: | ---: |").ok();

    let mut ranked = benchmark_config
        .models
        .iter()
        .map(|model| (model.clone(), compute_model_benchmark_stats(model, results)))
        .collect::<Vec<_>>();

    ranked.sort_by(|(left_model, left_stats), (right_model, right_stats)| {
        right_stats
            .success_count
            .cmp(&left_stats.success_count)
            .then_with(|| left_stats.failure_count.cmp(&right_stats.failure_count))
            .then_with(
                || match (left_stats.avg_total_ms_ok, right_stats.avg_total_ms_ok) {
                    (Some(left), Some(right)) => left.cmp(&right),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                },
            )
            .then_with(|| left_model.cmp(right_model))
    });

    for (index, (model, stats)) in ranked.iter().enumerate() {
        writeln!(
            report,
            "| {} | {} | {} | {} | {} |",
            index + 1,
            escape_markdown_cell(model),
            ollama_metadata
                .model_parameter_sizes
                .get(model)
                .map(String::as_str)
                .unwrap_or("unknown"),
            format_success_rate(stats.success_count, stats.total_count),
            stats
                .avg_total_ms_ok
                .map(|value| value.to_string())
                .unwrap_or_else(|| "failed".to_string()),
        )
        .ok();
    }

    writeln!(report).ok();
}

fn benchmark_output_is_stdout(path: &std::path::Path) -> bool {
    path.as_os_str() == "-"
}

fn format_success_rate(success_count: usize, total_count: usize) -> String {
    if total_count == 0 {
        return "0.0%".to_string();
    }

    format!(
        "{:.1}%",
        (success_count as f64 / total_count as f64) * 100.0
    )
}

fn compute_model_benchmark_stats(
    model: &str,
    results: &[ModelBenchmarkResult],
) -> ModelBenchmarkStats {
    let model_results = results
        .iter()
        .filter(|result| result.model == model)
        .collect::<Vec<_>>();
    let total_count = model_results.len();
    let success_count = model_results
        .iter()
        .filter(|result| result.error.is_none())
        .count();
    let failure_count = total_count.saturating_sub(success_count);
    let successful_total_ms = model_results
        .iter()
        .filter(|result| result.error.is_none())
        .map(|result| result.total_ms)
        .collect::<Vec<_>>();
    let avg_total_ms_ok = if successful_total_ms.is_empty() {
        None
    } else {
        Some(successful_total_ms.iter().sum::<u128>() / successful_total_ms.len() as u128)
    };

    ModelBenchmarkStats {
        total_count,
        success_count,
        failure_count,
        avg_total_ms_ok,
    }
}

fn escape_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

struct ModelBenchmarkResult {
    model: String,
    query: String,
    planner_ms: u128,
    fallback_ms: Option<u128>,
    total_ms: u128,
    unresolved: bool,
    response_kind: String,
    response: String,
    error: Option<String>,
}

struct ModelBenchmarkStats {
    total_count: usize,
    success_count: usize,
    failure_count: usize,
    avg_total_ms_ok: Option<u128>,
}

struct BenchmarkHostInfo {
    hostname: String,
    os: String,
    kernel: String,
    cpu: String,
    gpu: String,
    gpu_vram: String,
    memory: String,
}

fn collect_benchmark_host_info() -> BenchmarkHostInfo {
    let hostname = detect_hostname();
    let os = detect_host_os();
    let kernel = command_output("uname", &["-r"]).unwrap_or_else(|| "unknown".to_string());
    let cpu = detect_cpu_model().unwrap_or_else(|| "unknown".to_string());
    let gpu = detect_gpu_model().unwrap_or_else(|| "unknown".to_string());
    let gpu_vram = detect_gpu_vram().unwrap_or_else(|| "unknown".to_string());
    let memory = detect_memory_total().unwrap_or_else(|| "unknown".to_string());

    BenchmarkHostInfo {
        hostname,
        os,
        kernel,
        cpu,
        gpu,
        gpu_vram,
        memory,
    }
}

fn detect_hostname() -> String {
    command_output("hostname", &[]).unwrap_or_else(|| "unknown".to_string())
}

fn detect_host_os() -> String {
    match env::consts::OS {
        "linux" => read_linux_pretty_name().unwrap_or_else(|| "Linux".to_string()),
        "macos" => command_output("sw_vers", &["-productVersion"])
            .map(|version| format!("macOS {version}"))
            .unwrap_or_else(|| "macOS".to_string()),
        other => other.to_string(),
    }
}

fn read_linux_pretty_name() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;

    content.lines().find_map(|line| {
        line.strip_prefix("PRETTY_NAME=")
            .map(|value| value.trim_matches('"').to_string())
    })
}

fn detect_cpu_model() -> Option<String> {
    match env::consts::OS {
        "linux" => {
            let content = std::fs::read_to_string("/proc/cpuinfo").ok()?;
            content.lines().find_map(|line| {
                line.split_once(':').and_then(|(key, value)| {
                    (key.trim() == "model name").then(|| value.trim().to_string())
                })
            })
        }
        "macos" => command_output("sysctl", &["-n", "machdep.cpu.brand_string"]),
        _ => None,
    }
}

fn detect_gpu_model() -> Option<String> {
    match env::consts::OS {
        "linux" => command_output(
            "sh",
            &["-c", "lspci | grep -Ei 'vga|3d|display' | head -n 1"],
        )
        .and_then(|line| {
            line.split_once(':')
                .map(|(_, value)| value.trim().to_string())
                .or(Some(line))
        }),
        "macos" => command_output(
            "sh",
            &[
                "-c",
                "system_profiler SPDisplaysDataType 2>/dev/null | grep 'Chipset Model' | head -n 1 | sed 's/.*: //'",
            ],
        ),
        _ => None,
    }
}

fn detect_memory_total() -> Option<String> {
    match env::consts::OS {
        "linux" => {
            let content = std::fs::read_to_string("/proc/meminfo").ok()?;
            let mem_total_kib = content.lines().find_map(parse_mem_total_kib)?;
            Some(format_bytes_from_kib(mem_total_kib))
        }
        "macos" => command_output("sysctl", &["-n", "hw.memsize"])
            .and_then(|value| value.parse::<u64>().ok())
            .map(format_bytes),
        _ => None,
    }
}

fn detect_gpu_vram() -> Option<String> {
    match env::consts::OS {
        "linux" => detect_linux_gpu_vram(),
        "macos" => command_output(
            "sh",
            &[
                "-c",
                "system_profiler SPDisplaysDataType 2>/dev/null | grep -E 'VRAM|Video Memory' | head -n 1 | sed 's/.*: //'",
            ],
        ),
        _ => None,
    }
}

fn detect_linux_gpu_vram() -> Option<String> {
    if let Some(value) = command_output(
        "nvidia-smi",
        &["--query-gpu=memory.total", "--format=csv,noheader,nounits"],
    ) {
        let first_value = value.lines().next()?.trim().parse::<u64>().ok()?;
        return Some(format_bytes_from_mib(first_value));
    }

    let paths = std::fs::read_dir("/sys/class/drm").ok()?;
    for entry in paths.flatten() {
        let vram_path = entry.path().join("device/mem_info_vram_total");
        if let Ok(content) = std::fs::read_to_string(vram_path)
            && let Ok(bytes) = content.trim().parse::<u64>()
        {
            return Some(format_bytes(bytes));
        }
    }

    None
}

fn parse_mem_total_kib(line: &str) -> Option<u64> {
    let (key, value) = line.split_once(':')?;
    if key.trim() != "MemTotal" {
        return None;
    }

    value.split_whitespace().next()?.parse::<u64>().ok()
}

fn format_bytes_from_kib(kib: u64) -> String {
    format_bytes(kib.saturating_mul(1024))
}

fn format_bytes_from_mib(mib: u64) -> String {
    format_bytes(mib.saturating_mul(1024 * 1024))
}

fn format_bytes(bytes: u64) -> String {
    let gib = bytes as f64 / 1024_f64.powi(3);
    format!("{gib:.1} GiB")
}

fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
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

fn prompt_for_request(error_state: bool) -> Result<String> {
    if !io::stdin().is_terminal() {
        let mut request = String::new();
        io::stdin()
            .read_line(&mut request)
            .context("failed to read request from stdin")?;

        return normalize_request(request);
    }

    let theme = ColorfulTheme {
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
    };

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
        .context("failed to capture request from terminal")
        .and_then(normalize_request)
}

fn should_quit_interactive(request: &str) -> bool {
    request.trim() == "/quit"
}

fn interactive_prompt_cancelled(error: &anyhow::Error) -> bool {
    error
        .to_string()
        .contains("failed to capture request from terminal")
}

fn handle_interactive_request_error(
    error: &anyhow::Error,
    show_output: bool,
    output: &OutputStyler,
) -> Result<()> {
    if error.to_string().contains("command exited with status") {
        if show_output {
            println!("{} {error}", output.error("Error:"));
        }
        return Ok(());
    }

    Err(anyhow::Error::msg(error.to_string()))
}

fn normalize_request(request: String) -> Result<String> {
    let request = request.trim();

    if request.is_empty() {
        bail!("request must not be empty")
    }

    Ok(request.to_string())
}

fn resolve_request(parts: &[String]) -> Result<Option<String>> {
    if parts.is_empty() {
        return Ok(None);
    }

    normalize_request(parts.join(" ")).map(Some)
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

    if auto_select_best {
        if let Some(command) = recommended_command(plan) {
            return Ok(command);
        }

        return Ok(&plan.commands[0]);
    }

    let terminal = terminal_environment_status();
    if !terminal.interactive_dialogs_supported() {
        bail!(
            "interactive selection requires TTY stdin/stdout and a usable TERM; current terminal status: {}. Use --auto-select-best or run cli-bot in an interactive terminal",
            terminal.describe()
        )
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
    use std::collections::BTreeMap;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{
        BenchmarkHostInfo, Cli, ModelBenchmarkResult, TerminalEnvironmentStatus,
        apply_model_override, benchmark_output_is_stdout, collect_benchmark_host_info,
        command_output, compute_model_benchmark_stats, duration_to_ms, escape_markdown_cell,
        format_bytes_from_kib, format_bytes_from_mib, format_command_plan_response,
        format_success_rate, handle_interactive_request_error, handle_session_command,
        interactive_prompt_cancelled, normalize_request, parse_mem_total_kib,
        print_benchmark_report, render_models_benchmark_markdown, resolve_request,
        run_models_benchmark, select_command, should_quit_interactive,
    };
    use crate::config::{
        AppConfig, EnvironmentConfig, ExecutionConfig, ModelsBenchmarkConfig, OllamaConfig,
        SafetyConfig, SessionMemoryConfig, SessionScope, UiConfig,
    };
    use crate::environment::{OperatingSystem, ResolvedEnvironment};
    use crate::llm::OllamaBenchmarkMetadata;
    use crate::output::{ColorMode, OutputStyler};
    use crate::planner::{CommandPlan, PlannedCommand};
    use crate::session::{SessionRecord, SessionStore, SessionTurn};

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

        let selected = select_command(&plan, "Choose", true).expect("selection should succeed");

        assert_eq!(selected.command, "ping -c 5 google.com");
    }

    #[test]
    fn auto_selects_first_command_when_recommendation_missing() {
        let plan = CommandPlan {
            summary: None,
            unresolved: false,
            commands: vec![
                PlannedCommand {
                    command: "git log -1 --pretty=%B".into(),
                    description: "git log -1 --pretty=%B".into(),
                    potentially_destructive: false,
                    recommended: false,
                    rationale: None,
                },
                PlannedCommand {
                    command: "git rev-parse --short HEAD".into(),
                    description: "git rev-parse --short HEAD".into(),
                    potentially_destructive: false,
                    recommended: false,
                    rationale: None,
                },
            ],
        };

        let selected = select_command(&plan, "Choose", true).expect("selection should succeed");

        assert_eq!(selected.command, "git log -1 --pretty=%B");
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

    #[test]
    fn resolves_multi_part_request() {
        let request = resolve_request(&[
            "ping".to_string(),
            "google".to_string(),
            "five".to_string(),
            "times".to_string(),
        ])
        .expect("request parts should exist")
        .expect("request should normalize");

        assert_eq!(request, "ping google five times");
    }

    #[test]
    fn does_not_resolve_empty_request_parts() {
        assert_eq!(resolve_request(&[]).expect("request should resolve"), None);
    }

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
    fn select_command_rejects_empty_command_list() {
        let plan = CommandPlan {
            summary: None,
            unresolved: false,
            commands: vec![],
        };

        let error = select_command(&plan, "Choose", true).expect_err("selection should fail");

        assert!(error.to_string().contains("planner returned no commands"));
    }

    #[test]
    fn format_command_plan_response_includes_summary_and_rationale() {
        let plan = CommandPlan {
            summary: Some("Ping google".into()),
            unresolved: false,
            commands: vec![PlannedCommand {
                command: "ping -c 5 google.com".into(),
                description: "Ping five times".into(),
                potentially_destructive: false,
                recommended: true,
                rationale: Some("Matches the request".into()),
            }],
        };

        let response = format_command_plan_response(&plan);

        assert!(response.contains("summary: Ping google"));
        assert!(response.contains("ping -c 5 google.com [recommended]"));
        assert!(response.contains("why: Matches the request"));
    }

    #[test]
    fn escapes_markdown_cells() {
        assert_eq!(escape_markdown_cell("a|b\nc"), "a\\|b c");
    }

    #[test]
    fn benchmark_output_detects_stdout_marker() {
        assert!(benchmark_output_is_stdout(PathBuf::from("-").as_path()));
        assert!(!benchmark_output_is_stdout(
            PathBuf::from("report.md").as_path()
        ));
    }

    #[test]
    fn interactive_mode_quits_on_quit_command() {
        assert!(should_quit_interactive("/quit"));
        assert!(should_quit_interactive("  /quit  "));
        assert!(!should_quit_interactive("quit"));
    }

    #[test]
    fn interactive_prompt_cancelled_matches_prompt_error() {
        let error = anyhow::anyhow!("failed to capture request from terminal: interrupted");
        assert!(interactive_prompt_cancelled(&error));

        let other_error = anyhow::anyhow!("failed to read request from stdin");
        assert!(!interactive_prompt_cancelled(&other_error));
    }

    #[test]
    fn interactive_request_handler_swallows_command_exit_errors() {
        let output = OutputStyler::new(ColorMode::Never);
        let error = anyhow::anyhow!("command exited with status 7");

        handle_interactive_request_error(&error, false, &output)
            .expect("command exit errors should be recoverable in interactive mode");
    }

    #[test]
    fn interactive_request_handler_propagates_non_command_errors() {
        let output = OutputStyler::new(ColorMode::Never);
        let error = anyhow::anyhow!("failed to contact Ollama");

        let propagated = handle_interactive_request_error(&error, false, &output)
            .expect_err("non-command failures should still be fatal");

        assert!(propagated.to_string().contains("failed to contact Ollama"));
    }

    #[test]
    fn collect_benchmark_host_info_returns_non_empty_fields() {
        let host_info = collect_benchmark_host_info();

        assert!(!host_info.hostname.is_empty());
        assert!(!host_info.os.is_empty());
        assert!(!host_info.kernel.is_empty());
        assert!(!host_info.cpu.is_empty());
        assert!(!host_info.gpu.is_empty());
        assert!(!host_info.gpu_vram.is_empty());
        assert!(!host_info.memory.is_empty());
    }

    #[test]
    fn command_output_returns_none_for_failing_commands() {
        assert!(command_output("sh", &["-c", "exit 1"]).is_none());
        assert!(command_output("sh", &["-c", "printf ''"]).is_none());
    }

    #[test]
    fn render_models_benchmark_markdown_includes_summary_sections() {
        let mut parameter_sizes = BTreeMap::new();
        parameter_sizes.insert("lfm2:latest".to_string(), "12B".to_string());
        let host_info = BenchmarkHostInfo {
            hostname: "devbox".into(),
            os: "Linux".into(),
            kernel: "6.8.0".into(),
            cpu: "CPU".into(),
            gpu: "GPU".into(),
            gpu_vram: "8.0 GiB".into(),
            memory: "32.0 GiB".into(),
        };
        let metadata = OllamaBenchmarkMetadata {
            version: Some("0.6.0".into()),
            model_parameter_sizes: parameter_sizes,
        };
        let benchmark_config = ModelsBenchmarkConfig {
            models: vec!["lfm2:latest".into(), "qwen3.5:latest".into()],
            queries: vec!["Ping google five times".into(), "spell mantainence".into()],
        };
        let results = vec![
            ModelBenchmarkResult {
                model: "lfm2:latest".into(),
                query: "Ping google five times".into(),
                planner_ms: 10,
                fallback_ms: None,
                total_ms: 10,
                unresolved: false,
                response_kind: "command_plan".into(),
                response: "ping -c 5 google.com".into(),
                error: None,
            },
            ModelBenchmarkResult {
                model: "qwen3.5:latest".into(),
                query: "Ping google five times".into(),
                planner_ms: 20,
                fallback_ms: None,
                total_ms: 20,
                unresolved: false,
                response_kind: "error".into(),
                response: String::new(),
                error: Some("planner failed".into()),
            },
            ModelBenchmarkResult {
                model: "lfm2:latest".into(),
                query: "spell mantainence".into(),
                planner_ms: 11,
                fallback_ms: Some(4),
                total_ms: 15,
                unresolved: true,
                response_kind: "text_response".into(),
                response: "maintenance".into(),
                error: None,
            },
        ];

        let markdown =
            render_models_benchmark_markdown(&host_info, &metadata, &benchmark_config, &results);

        assert!(markdown.contains("# Model Benchmark Report"));
        assert!(markdown.contains("## Summary Table"));
        assert!(markdown.contains("## Model Summary"));
        assert!(markdown.contains("## Ranking"));
        assert!(markdown.contains("## Detailed Results"));
        assert!(markdown.contains("failed"));
        assert!(markdown.contains("maintenance"));
        assert!(markdown.contains("lfm2:latest"));
    }

    #[test]
    fn handle_session_command_requires_enabled_session() {
        let (store, temp_root) = temp_session_store();
        let cli = sample_cli();
        let output = OutputStyler::new(ColorMode::Never);

        let error = handle_session_command(&cli, false, None, &store, false, &output)
            .expect_err("disabled session should fail");

        assert!(error.to_string().contains("session memory is disabled"));
        fs::remove_dir_all(temp_root).expect("temp root should be removed");
    }

    #[test]
    fn handle_session_command_rejects_conflicting_flags() {
        let (store, temp_root) = temp_session_store();
        let mut cli = sample_cli();
        cli.session_show = true;
        cli.session_clear = true;
        let output = OutputStyler::new(ColorMode::Never);

        let error = handle_session_command(&cli, true, None, &store, false, &output)
            .expect_err("conflicting session flags should fail");

        assert!(error.to_string().contains("cannot be used together"));
        fs::remove_dir_all(temp_root).expect("temp root should be removed");
    }

    #[test]
    fn handle_session_command_lists_shows_and_clears_sessions() {
        let (store, temp_root) = temp_session_store();
        let output = OutputStyler::new(ColorMode::Never);
        let mut record = SessionRecord::new(
            "default".into(),
            SessionScope::Global,
            PathBuf::from("/tmp/project"),
        );
        record.push_turn(SessionTurn {
            timestamp_epoch_ms: 1,
            request: "find git config".into(),
            working_directory: None,
            plan_summary: Some("Locate config".into()),
            unresolved: false,
            command_choices: Vec::new(),
            selected_command: Some("fd gitconfig ~".into()),
            selected_command_rationale: None,
            confirmation_required: false,
            text_response: None,
            execution: None,
        });
        store.save(&mut record).expect("session should save");

        let mut list_cli = sample_cli();
        list_cli.session_list = true;
        handle_session_command(&list_cli, true, None, &store, false, &output)
            .expect("session list should succeed");

        let mut show_cli = sample_cli();
        show_cli.session_show = true;
        handle_session_command(&show_cli, true, None, &store, false, &output)
            .expect("session show should succeed");

        let mut clear_cli = sample_cli();
        clear_cli.session_clear = true;
        handle_session_command(&clear_cli, true, None, &store, false, &output)
            .expect("session clear should succeed");

        assert!(!store.clear(None).expect("clear should be idempotent"));
        fs::remove_dir_all(temp_root).expect("temp root should be removed");
    }

    #[test]
    fn run_models_benchmark_rejects_missing_models() {
        let config = sample_config();
        let output = OutputStyler::new(ColorMode::Never);

        let error = run_models_benchmark(
            config,
            sample_resolved_environment(),
            false,
            &output,
            PathBuf::from("-"),
        )
        .expect_err("missing models should fail");

        assert!(
            error
                .to_string()
                .contains("models_benchmark.models is empty")
        );
    }

    #[test]
    fn run_models_benchmark_rejects_missing_queries() {
        let mut config = sample_config();
        config.models_benchmark.models = vec!["lfm2:latest".into()];
        let output = OutputStyler::new(ColorMode::Never);

        let error = run_models_benchmark(
            config,
            sample_resolved_environment(),
            false,
            &output,
            PathBuf::from("-"),
        )
        .expect_err("missing queries should fail");

        assert!(
            error
                .to_string()
                .contains("models_benchmark.queries is empty")
        );
    }

    #[test]
    fn print_benchmark_report_executes_without_error() {
        let output = OutputStyler::new(ColorMode::Never);

        print_benchmark_report(
            &output,
            "lfm2:latest",
            Duration::from_millis(12),
            Some(Duration::from_millis(3)),
            Duration::from_millis(15),
        );
    }

    fn sample_config() -> AppConfig {
        AppConfig {
            ollama: OllamaConfig {
                base_url: "http://127.0.0.1:11434".into(),
                model: "default-model".into(),
                temperature: 0.0,
                use_chat_api: true,
                system_prompt: "Return JSON only".into(),
            },
            environment: EnvironmentConfig::default(),
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
            session_memory: SessionMemoryConfig::default(),
            models_benchmark: ModelsBenchmarkConfig::default(),
        }
    }

    fn sample_resolved_environment() -> ResolvedEnvironment {
        ResolvedEnvironment {
            os: OperatingSystem::Linux,
            distro: None,
            detected_package_manager: None,
            effective_package_manager: None,
            package_manager_source: None,
            effective_package_manager_available: false,
        }
    }

    fn sample_cli() -> Cli {
        Cli {
            request: Vec::new(),
            config: None,
            model: None,
            check: false,
            models_benchmark: None,
            auto_select_best: false,
            color: ColorMode::Never,
            dry_run: false,
            print_plan: false,
            benchmark: false,
            verbose: false,
            quiet: false,
            interactive: false,
            session: None,
            no_session: false,
            session_show: false,
            session_list: false,
            session_clear: false,
        }
    }

    fn temp_session_store() -> (SessionStore, PathBuf) {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let temp_root = env::temp_dir().join(format!("cli-bot-lib-test-{unique}"));
        fs::create_dir_all(&temp_root).expect("temp root should exist");
        let config = SessionMemoryConfig {
            storage_dir: temp_root.display().to_string(),
            scope: SessionScope::Global,
            ..SessionMemoryConfig::default()
        };
        let store = SessionStore::new(config).expect("store should initialize");

        (store, temp_root)
    }

    #[test]
    fn parses_mem_total_from_proc_meminfo_line() {
        assert_eq!(
            parse_mem_total_kib("MemTotal:       32768000 kB"),
            Some(32_768_000)
        );
        assert_eq!(format_bytes_from_kib(1_048_576), "1.0 GiB");
        assert_eq!(format_bytes_from_mib(16_384), "16.0 GiB");
    }

    #[test]
    fn formats_success_rate_percentage() {
        assert_eq!(format_success_rate(7, 7), "100.0%");
        assert_eq!(format_success_rate(5, 7), "71.4%");
        assert_eq!(format_success_rate(0, 0), "0.0%");
    }

    #[test]
    fn computes_model_benchmark_stats() {
        let results = vec![
            ModelBenchmarkResult {
                model: "lfm2:latest".into(),
                query: "Ping google five times".into(),
                planner_ms: 10,
                fallback_ms: None,
                total_ms: 10,
                unresolved: false,
                response_kind: "command_plan".into(),
                response: "ping -c 5 google.com".into(),
                error: None,
            },
            ModelBenchmarkResult {
                model: "lfm2:latest".into(),
                query: "What is the purpose of life?".into(),
                planner_ms: 10,
                fallback_ms: Some(5),
                total_ms: 15,
                unresolved: true,
                response_kind: "text_response".into(),
                response: "42".into(),
                error: None,
            },
            ModelBenchmarkResult {
                model: "lfm2:latest".into(),
                query: "broken".into(),
                planner_ms: 30,
                fallback_ms: None,
                total_ms: 30,
                unresolved: false,
                response_kind: "error".into(),
                response: String::new(),
                error: Some("boom".into()),
            },
        ];

        let stats = compute_model_benchmark_stats("lfm2:latest", &results);

        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.avg_total_ms_ok, Some(12));
    }
}
