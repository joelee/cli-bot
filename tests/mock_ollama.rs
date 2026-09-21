use std::collections::VecDeque;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use cli_bot::{Cli, CliBotError, ColorMode, Prompter, run, run_with_prompter};

static TEMP_DIR_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn plans_commands_via_generate_endpoint() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server = MockOllamaServer::start(
        capture.clone(),
        vec![MockResponse::json(
            r#"{"response":"{\"summary\":\"Ping google\",\"unresolved\":false,\"commands\":[{\"command\":\"ping -c 5 google.com\",\"description\":\"Ping five times\",\"potentially_destructive\":false,\"recommended\":true,\"rationale\":\"Matches the request\"}]}"}"#,
        )],
    );

    let config_path = write_config(server.base_url(), false, false, None, None, false);
    let cli = sample_cli(config_path, vec!["Ping google five times"]);

    run(cli).expect("generate backend should succeed");

    let requests = capture.lock().expect("capture should lock");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/api/generate");
    assert!(requests[0].body.contains("Ping google five times"));
    assert!(requests[0].body.contains("Session context"));
}

#[test]
fn plans_commands_via_chat_endpoint() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server = MockOllamaServer::start(
        capture.clone(),
        vec![MockResponse::json(
            r#"{"message":{"content":"{\"summary\":\"Ping google\",\"unresolved\":false,\"commands\":[{\"command\":\"ping -c 5 google.com\",\"description\":\"Ping five times\",\"potentially_destructive\":false,\"recommended\":true,\"rationale\":\"Matches the request\"}]}"}}"#,
        )],
    );

    let config_path = write_config(server.base_url(), true, false, None, None, false);
    let cli = sample_cli(config_path, vec!["Ping google five times"]);

    run(cli).expect("chat backend should succeed");

    let requests = capture.lock().expect("capture should lock");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/api/chat");
    assert!(requests[0].body.contains("\"role\":\"system\""));
    assert!(requests[0].body.contains("\"role\":\"user\""));
    assert!(requests[0].body.contains("Ping google five times"));
    assert!(requests[0].body.contains("\"format\":\"json\""));
}

#[test]
fn unresolved_fallback_uses_generate_endpoint_twice() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server = MockOllamaServer::start(
        capture.clone(),
        vec![
            MockResponse::json(
                r#"{"response":"{\"summary\":\"Need clarification\",\"unresolved\":true,\"commands\":[]}"}"#,
            ),
            MockResponse::json(r#"{"response":"maintenance"}"#),
        ],
    );

    let config_path = write_config(server.base_url(), false, false, None, None, false);
    let cli = sample_cli(config_path, vec!["spell mantainence"]);

    run(cli).expect("generate unresolved fallback should succeed");

    let requests = capture.lock().expect("capture should lock");
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].path, "/api/generate");
    assert_eq!(requests[1].path, "/api/generate");
    assert!(requests[1].body.contains("Respond with plain text only"));
}

#[test]
fn unresolved_fallback_uses_chat_endpoint_then_plain_text_chat() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server = MockOllamaServer::start(
        capture.clone(),
        vec![
            MockResponse::json(
                r#"{"message":{"content":"{\"summary\":\"Need clarification\",\"unresolved\":true,\"commands\":[]}"}}"#,
            ),
            MockResponse::json(r#"{"message":{"content":"maintenance"}}"#),
        ],
    );

    let config_path = write_config(server.base_url(), true, false, None, None, false);
    let cli = sample_cli(config_path, vec!["spell mantainence"]);

    run(cli).expect("chat unresolved fallback should succeed");

    let requests = capture.lock().expect("capture should lock");
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].path, "/api/chat");
    assert_eq!(requests[1].path, "/api/chat");
    assert!(requests[0].body.contains("\"format\":\"json\""));
    assert!(!requests[1].body.contains("\"format\":\"json\""));
    assert!(requests[1].body.contains("Respond with plain text only"));
}

#[test]
fn check_hits_ollama_version_and_tags_endpoints() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server = MockOllamaServer::start(
        capture.clone(),
        vec![
            MockResponse::json(r#"{"version":"0.6.0"}"#),
            MockResponse::json(
                r#"{"models":[{"name":"lfm2:latest","details":{"parameter_size":"12B"}}]}"#,
            ),
        ],
    );

    let config_path = write_config(server.base_url(), true, false, None, None, false);
    let mut cli = sample_cli(config_path, vec![]);
    cli.check = true;

    run(cli).expect("check should succeed against mock Ollama");

    let requests = capture.lock().expect("capture should lock");
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].path, "/api/version");
    assert_eq!(requests[1].path, "/api/tags");
}

#[test]
fn session_commands_list_show_and_prune() {
    let temp_root = unique_temp_dir("cli-bot-session-integration");
    let storage_dir = temp_root.join("session-state");
    fs::create_dir_all(storage_dir.join("sessions")).expect("session dir should exist");

    fs::write(
        storage_dir.join("sessions/default-global.json"),
        r#"
{
  "name": "default",
  "scope": "global",
  "base_working_directory": "/tmp/project",
  "turns": [
    {
      "timestamp_epoch_ms": 1,
      "request": "old request",
      "working_directory": "/tmp/project",
      "plan_summary": null,
      "unresolved": false,
      "command_choices": [],
      "selected_command": "true",
      "selected_command_rationale": null,
      "confirmation_required": false,
      "text_response": null,
      "execution": null
    }
  ]
}
"#,
    )
    .expect("expired session should write");
    // Pruning reads the file's modification time, not its newest turn.
    set_modified(
        &storage_dir.join("sessions/default-global.json"),
        SystemTime::now() - Duration::from_secs(3 * 24 * 60 * 60),
    );

    let config_path = write_config(
        "http://127.0.0.1:1",
        false,
        false,
        Some(storage_dir.clone()),
        Some(1),
        true,
    );

    let mut list_cli = sample_cli(config_path.clone(), vec![]);
    list_cli.session_list = true;
    list_cli.no_session = false;
    run(list_cli).expect("session list should succeed");

    assert!(
        !storage_dir.join("sessions/default-global.json").exists(),
        "expired session should be pruned before listing"
    );

    fs::write(
        storage_dir.join("sessions/default.json"),
        format!(
            r#"
{{
  "name": "default",
  "scope": "global",
  "base_working_directory": "{}",
  "turns": [
    {{
      "timestamp_epoch_ms": {},
      "request": "find my git config file",
      "working_directory": "{}",
      "plan_summary": "Locate the file",
      "unresolved": false,
      "command_choices": [],
      "selected_command": "fd gitconfig ~",
      "selected_command_rationale": null,
      "confirmation_required": false,
      "text_response": null,
      "execution": null
    }}
  ]
}}
"#,
            temp_root.display(),
            current_epoch_ms(),
            temp_root.display(),
        ),
    )
    .expect("current session should write");

    let mut show_cli = sample_cli(config_path, vec![]);
    show_cli.session_show = true;
    show_cli.no_session = false;
    run(show_cli).expect("session show should succeed");
}

/// What the flow asked the user, in order.
#[derive(Debug, Clone, PartialEq)]
enum Asked {
    Request {
        error_state: bool,
    },
    Select {
        prompt: String,
        items: Vec<String>,
        default: usize,
    },
    Confirm {
        prompt: String,
        default: bool,
    },
}

/// A prompter whose answers are fixed in advance, so a test drives the real
/// request flow without a terminal. An unscripted question is an error, not a
/// guess, so a test that expects no prompt fails loudly when one appears.
struct ScriptedPrompter {
    requests: Mutex<VecDeque<String>>,
    selections: Mutex<VecDeque<usize>>,
    confirmations: Mutex<VecDeque<bool>>,
    supports_dialogs: bool,
    asked: Mutex<Vec<Asked>>,
}

impl ScriptedPrompter {
    /// Answers nothing: any prompt fails the test.
    fn silent() -> Self {
        Self {
            requests: Mutex::new(VecDeque::new()),
            selections: Mutex::new(VecDeque::new()),
            confirmations: Mutex::new(VecDeque::new()),
            supports_dialogs: true,
            asked: Mutex::new(Vec::new()),
        }
    }

    /// No terminal: the flow must fail rather than assume an answer.
    fn without_dialogs(mut self) -> Self {
        self.supports_dialogs = false;
        self
    }

    fn with_selections(mut self, answers: impl IntoIterator<Item = usize>) -> Self {
        self.selections = Mutex::new(answers.into_iter().collect());
        self
    }

    fn with_requests(mut self, answers: impl IntoIterator<Item = &'static str>) -> Self {
        self.requests = Mutex::new(answers.into_iter().map(ToString::to_string).collect());
        self
    }

    fn with_confirmations(mut self, answers: impl IntoIterator<Item = bool>) -> Self {
        self.confirmations = Mutex::new(answers.into_iter().collect());
        self
    }

    fn asked(&self) -> Vec<Asked> {
        self.asked.lock().expect("asked should lock").clone()
    }

    /// The confirmation prompts shown, as (text, default answer).
    fn confirmations_shown(&self) -> Vec<(String, bool)> {
        self.asked()
            .into_iter()
            .filter_map(|entry| match entry {
                Asked::Confirm { prompt, default } => Some((prompt, default)),
                _ => None,
            })
            .collect()
    }
}

impl Prompter for ScriptedPrompter {
    fn read_request(&self, error_state: bool) -> anyhow::Result<String> {
        self.asked
            .lock()
            .expect("asked should lock")
            .push(Asked::Request { error_state });
        self.requests
            .lock()
            .expect("requests should lock")
            .pop_front()
            // A script that has run out is this harness's end of input.
            .ok_or_else(|| {
                anyhow::Error::new(CliBotError::Cancelled).context("the script has run out")
            })
    }

    fn select(&self, prompt: &str, items: &[String], default: usize) -> anyhow::Result<usize> {
        self.asked
            .lock()
            .expect("asked should lock")
            .push(Asked::Select {
                prompt: prompt.to_string(),
                items: items.to_vec(),
                default,
            });
        self.selections
            .lock()
            .expect("selections should lock")
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("unscripted selection: {prompt}"))
    }

    fn confirm(&self, prompt: &str, default: bool) -> anyhow::Result<bool> {
        self.asked
            .lock()
            .expect("asked should lock")
            .push(Asked::Confirm {
                prompt: prompt.to_string(),
                default,
            });
        self.confirmations
            .lock()
            .expect("confirmations should lock")
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("unscripted confirmation: {prompt}"))
    }

    fn supports_dialogs(&self) -> bool {
        self.supports_dialogs
    }
}

#[test]
fn run_with_prompter_drives_the_same_flow_as_run() {
    let storage_dir = unique_temp_dir("cli-bot-prompter-seam");
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server =
        MockOllamaServer::start(capture.clone(), vec![MockResponse::generated(PRINTF_PLAN)]);
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent();

    run_with_prompter(session_cli(config_path, vec!["print ok"]), &prompter)
        .expect("a single planned command should run without any prompt");

    assert_eq!(prompter.asked(), Vec::new());
    assert_eq!(
        read_default_session(&storage_dir)["turns"][0]["selected_command"],
        "printf ok"
    );
}

/// A planner reply offering each of `commands`, the first recommended.
fn plan_with_commands(commands: &[&str]) -> MockResponse {
    let entries = commands
        .iter()
        .enumerate()
        .map(|(index, command)| {
            serde_json::json!({
                "command": command,
                "description": format!("choice {index}"),
                "potentially_destructive": false,
                "recommended": index == 0,
            })
        })
        .collect::<Vec<_>>();
    MockResponse::generated(
        &serde_json::json!({"summary": "x", "unresolved": false, "commands": entries}).to_string(),
    )
}

/// A planner reply carrying the timings Ollama reports alongside it.
fn plan_for_with_timings(command: &str) -> MockResponse {
    let plan = serde_json::json!({
        "summary": "x",
        "unresolved": false,
        "commands": [{
            "command": command,
            "description": "d",
            "potentially_destructive": false,
            "recommended": true,
        }],
    })
    .to_string();
    MockResponse::json(
        &serde_json::json!({
            "response": plan,
            "eval_count": 10,
            "eval_duration": 255_555_000_u64,
            "prompt_eval_count": 5,
            "prompt_eval_duration": 86_884_000_u64,
            "load_duration": 8_614_861_799_u64,
            "total_duration": 8_962_347_492_u64,
        })
        .to_string(),
    )
}

/// A planner reply whose single command is `command`, never flagged by the
/// model, so only cli-bot's own classification decides what happens.
fn plan_for(command: &str) -> MockResponse {
    MockResponse::generated(
        &serde_json::json!({
            "summary": "x",
            "unresolved": false,
            "commands": [{
                "command": command,
                "description": "d",
                "potentially_destructive": false,
                "recommended": true,
            }],
        })
        .to_string(),
    )
}

const PRINTF_PLAN: &str = r#"{"summary":"Print ok","unresolved":false,"commands":[{"command":"printf ok","description":"Print ok","potentially_destructive":false,"recommended":true,"rationale":"Harmless"}]}"#;
const UNRESOLVED_PLAN: &str = r#"{"summary":"Need clarification","unresolved":true,"commands":[]}"#;

#[test]
fn executes_command_and_saves_session_turn_with_captured_output() {
    let storage_dir = unique_temp_dir("cli-bot-exec-session");
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server =
        MockOllamaServer::start(capture.clone(), vec![MockResponse::generated(PRINTF_PLAN)]);
    let config_path = ConfigOptions {
        capture_command_output: true,
        ..ConfigOptions::with_session(&storage_dir)
    }
    .write(server.base_url());
    let mut cli = session_cli(config_path, vec!["print", "ok"]);
    cli.benchmark = true;
    cli.print_plan = true;

    run(cli).expect("harmless command should execute");

    let session = read_default_session(&storage_dir);
    let turn = &session["turns"][0];
    assert_eq!(turn["request"], "print ok");
    assert_eq!(turn["selected_command"], "printf ok");
    assert_eq!(turn["confirmation_required"], false);
    assert_eq!(turn["execution"]["executed"], true);
    assert_eq!(turn["execution"]["exit_status"], 0);
    assert_eq!(turn["execution"]["stdout"], "ok");
}

#[test]
fn a_failed_command_keeps_its_status_and_is_remembered() {
    let storage_dir = unique_temp_dir("cli-bot-exec-failure");
    let server =
        MockOllamaServer::start(Arc::new(Mutex::new(Vec::new())), vec![plan_for("exit 3")]);
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent().with_confirmations([true]);

    let error = run_with_prompter(session_cli(config_path, vec!["fail"]), &prompter)
        .expect_err("exit 3 should fail");

    assert_eq!(
        error.downcast_ref::<CliBotError>(),
        Some(&CliBotError::CommandFailed { status: Some(3) })
    );
    assert_eq!(
        error.to_string(),
        "command exited with status 3",
        "no doubled word, and the status is the command's own"
    );
    // The attempt is remembered, so "why did that fail" has context.
    let turn = &read_default_session(&storage_dir)["turns"][0];
    assert_eq!(turn["selected_command"], "exit 3");
    assert_eq!(turn["execution"]["executed"], true);
    assert_eq!(turn["execution"]["exit_status"], 3);
}

/// A command in the state-changing tier that is harmless when it runs.
fn touch_command(directory: &Path) -> (String, PathBuf) {
    let target = directory.join("created-by-the-test");
    (format!("touch {}", path_to_string(&target)), target)
}

/// A command in the destructive tier that is harmless when it runs: the
/// path does not exist, and `rm -f` succeeds on a missing path.
fn absent_rm_command(directory: &Path) -> String {
    format!("rm -fr {}", path_to_string(&directory.join("absent")))
}

#[test]
fn interactive_mode_survives_a_planner_error_and_keeps_asking() {
    let storage_dir = unique_temp_dir("cli-bot-interactive-recovery");
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server = MockOllamaServer::start(
        capture.clone(),
        vec![
            // The first request gets a reply with no JSON in it at all.
            MockResponse::generated("I have no idea what you mean"),
            plan_for("ls -la"),
        ],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent().with_requests(["first request", "second request"]);
    let mut cli = session_cli(config_path, vec![]);
    cli.interactive = true;

    run_with_prompter(cli, &prompter).expect("end of input ends the session cleanly");

    assert_eq!(
        capture.lock().expect("capture should lock").len(),
        2,
        "the second request must still reach the planner"
    );
    // Three prompts: two requests, then the one that ends the session.
    let requests = prompter
        .asked()
        .into_iter()
        .filter(|entry| matches!(entry, Asked::Request { .. }))
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[1], Asked::Request { error_state: true });
    assert_eq!(
        read_default_session(&storage_dir)["turns"]
            .as_array()
            .expect("turns should be an array")
            .len(),
        1,
        "only the request that produced a command is remembered"
    );
}

#[test]
fn a_planner_error_outside_interactive_mode_is_still_fatal() {
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![MockResponse::generated("no json here")],
    );
    let config_path = ConfigOptions::default().write(server.base_url());

    let error = run_with_prompter(
        sample_cli(config_path, vec!["do something"]),
        &ScriptedPrompter::silent(),
    )
    .expect_err("a single request has nowhere to recover to");

    assert!(format!("{error:#}").contains("did not contain a JSON object"));
}

#[test]
fn benchmark_reports_the_rate_only_when_the_server_supplies_it() {
    for (reply, expected) in [
        (plan_for_with_timings("ls -la"), true),
        (plan_for("ls -la"), false),
    ] {
        let server = MockOllamaServer::start(Arc::new(Mutex::new(Vec::new())), vec![reply]);
        let config_path = ConfigOptions::default().write(server.base_url());
        let mut cli = sample_cli(config_path, vec!["list files"]);
        cli.quiet = false;
        cli.benchmark = true;

        // The report goes to stdout; that it runs and that the figures are
        // derived is covered by the unit tests. Here we only prove the flow
        // survives both shapes of reply.
        run_with_prompter(cli, &ScriptedPrompter::silent())
            .unwrap_or_else(|error| panic!("timings present={expected}: {error:#}"));
    }
}

#[test]
fn a_slow_planner_reports_the_timeout_rather_than_a_decoding_failure() {
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![plan_for("ls -la").after(Duration::from_secs(3))],
    );
    let config_path = ConfigOptions {
        request_timeout_seconds: Some(1),
        ..ConfigOptions::default()
    }
    .write(server.base_url());
    let mut cli = sample_cli(config_path, vec!["list files"]);
    cli.quiet = false;
    cli.verbose = true;

    let error = run(cli).expect_err("the request should time out");
    let message = format!("{error:#}");

    assert!(message.contains("failed to call Ollama"), "{message}");
    assert!(
        message.to_lowercase().contains("timed out") || message.to_lowercase().contains("timeout"),
        "the cause should name the timeout: {message}"
    );
}

#[test]
fn an_unreadable_session_file_stops_nothing() {
    let storage_dir = unique_temp_dir("cli-bot-corrupt-session");
    let sessions = storage_dir.join("sessions");
    fs::create_dir_all(&sessions).expect("sessions dir should exist");
    // A hand-edited or half-written file, which used to abort every run.
    fs::write(sessions.join("broken.json"), "{\"name\": \"bro").expect("corrupt file should write");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            // In the order the test asks for them: the seeding plan, then
            // the version and tags that `--check` reads.
            plan_for("ls -la"),
            MockResponse::json(r#"{"version":"0.6.0"}"#),
            MockResponse::json(r#"{"models":[{"name":"lfm2:latest"}]}"#),
        ],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent();

    // Seed one good session beside the broken one.
    run_with_prompter(
        session_cli(config_path.clone(), vec!["list files"]),
        &prompter,
    )
    .expect("a read-only command should run");

    let mut check = sample_cli(config_path.clone(), vec![]);
    check.check = true;
    check.no_session = false;
    run_with_prompter(check, &prompter).expect("--check must not read sessions");

    let mut list = session_cli(config_path.clone(), vec![]);
    list.session_list = true;
    run_with_prompter(list, &prompter).expect("--session-list should skip the bad file");

    let mut clear = session_cli(config_path.clone(), vec![]);
    clear.session_clear = true;
    run_with_prompter(clear, &prompter).expect("--session-clear should succeed");

    let mut disabled = sample_cli(config_path, vec![]);
    disabled.session_show = true;
    let error = run_with_prompter(disabled, &prompter)
        .expect_err("--no-session with a session command is still rejected");
    assert!(error.to_string().contains("session memory is disabled"));

    assert!(
        sessions.join("broken.json").is_file(),
        "a file the user wrote is theirs; it is skipped, not deleted"
    );
}

#[test]
fn without_session_memory_the_sessions_folder_is_never_touched() {
    // A storage directory that does not exist: any read or prune would
    // have to create or walk it.
    let storage_dir = unique_temp_dir("cli-bot-untouched-sessions");
    let server =
        MockOllamaServer::start(Arc::new(Mutex::new(Vec::new())), vec![plan_for("ls -la")]);
    let config_path = ConfigOptions {
        retention_days: Some(1),
        ..ConfigOptions::with_session(&storage_dir)
    }
    .write(server.base_url());
    let mut cli = session_cli(config_path, vec!["list files"]);
    cli.no_session = true;

    run_with_prompter(cli, &ScriptedPrompter::silent()).expect("the command should run");

    assert!(
        !storage_dir.exists(),
        "nothing may create the sessions folder"
    );
}

#[test]
fn the_selected_alternative_is_the_command_that_runs() {
    let storage_dir = unique_temp_dir("cli-bot-selection");
    fs::create_dir_all(&storage_dir).expect("storage dir should exist");
    let chosen = storage_dir.join("second");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![plan_with_commands(&[
            "ls -la",
            &format!("touch {}", path_to_string(&chosen)),
        ])],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    // Choose the second command, then approve it.
    let prompter = ScriptedPrompter::silent()
        .with_selections([1])
        .with_confirmations([true]);

    run_with_prompter(session_cli(config_path, vec!["do something"]), &prompter)
        .expect("the chosen command should run");

    let selects = prompter
        .asked()
        .into_iter()
        .filter(|entry| matches!(entry, Asked::Select { .. }))
        .collect::<Vec<_>>();
    assert_eq!(selects.len(), 1);
    let Asked::Select { items, default, .. } = &selects[0] else {
        unreachable!()
    };
    assert_eq!(items.len(), 2);
    assert_eq!(*default, 0);
    assert!(chosen.is_file(), "the second command should have run");
    assert_eq!(
        read_default_session(&storage_dir)["turns"][0]["selected_command"],
        format!("touch {}", path_to_string(&chosen))
    );
}

#[test]
fn require_confirmation_false_runs_every_tier_without_asking() {
    let storage_dir = unique_temp_dir("cli-bot-no-confirmation");
    fs::create_dir_all(&storage_dir).expect("storage dir should exist");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![plan_for(&absent_rm_command(&storage_dir))],
    );
    let config_path = ConfigOptions {
        no_confirmation: true,
        ..ConfigOptions::with_session(&storage_dir)
    }
    .write(server.base_url());
    let prompter = ScriptedPrompter::silent().without_dialogs();

    run_with_prompter(session_cli(config_path, vec!["delete it"]), &prompter)
        .expect("the master switch turns every prompt off");

    assert_eq!(prompter.asked(), Vec::new());
    let turn = &read_default_session(&storage_dir)["turns"][0];
    assert_eq!(turn["risk"], "destructive");
    assert_eq!(turn["confirmation_required"], false);
    assert_eq!(turn["execution"]["executed"], true);
}

#[test]
fn a_configured_substring_still_forces_the_destructive_tier() {
    let storage_dir = unique_temp_dir("cli-bot-substring");
    fs::create_dir_all(&storage_dir).expect("storage dir should exist");
    // `rm -rf` is in the shipped substring list; the built-in rules would
    // reach the same tier, so the quoted argument proves the list is read.
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![plan_for("echo 'rm -rf /'")],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent().with_confirmations([false]);

    run_with_prompter(session_cli(config_path, vec!["print it"]), &prompter)
        .expect("a declined command is not an error");

    assert_eq!(prompter.confirmations_shown().len(), 1);
    assert_eq!(
        read_default_session(&storage_dir)["turns"][0]["risk"],
        "destructive"
    );
}

#[test]
fn a_command_that_cannot_be_parsed_is_confirmed_rather_than_run() {
    let storage_dir = unique_temp_dir("cli-bot-unparsable");
    fs::create_dir_all(&storage_dir).expect("storage dir should exist");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![plan_for("echo 'unterminated")],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent().with_confirmations([false]);

    run_with_prompter(session_cli(config_path, vec!["print it"]), &prompter)
        .expect("a declined command is not an error");

    assert_eq!(
        prompter.confirmations_shown().len(),
        1,
        "an unparsable command must never be waved through"
    );
    assert_eq!(
        read_default_session(&storage_dir)["turns"][0]["risk"],
        "state-changing"
    );
}

#[test]
fn yes_approves_the_ordinary_tier_but_not_the_destructive_one() {
    let storage_dir = unique_temp_dir("cli-bot-flag-yes");
    fs::create_dir_all(&storage_dir).expect("storage dir should exist");
    let (command, target) = touch_command(&storage_dir);
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            plan_for(&command),
            plan_for(&absent_rm_command(&storage_dir)),
        ],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());

    let mut ordinary = session_cli(config_path.clone(), vec!["create the file"]);
    ordinary.yes = true;
    let quiet = ScriptedPrompter::silent();
    run_with_prompter(ordinary, &quiet).expect("--yes should approve the ordinary tier");
    assert_eq!(quiet.asked(), Vec::new());
    assert!(target.is_file());

    let mut destructive = session_cli(config_path, vec!["delete it"]);
    destructive.yes = true;
    let asked = ScriptedPrompter::silent().with_confirmations([false]);
    run_with_prompter(destructive, &asked).expect("a declined command is not an error");
    assert_eq!(
        asked.confirmations_shown().len(),
        1,
        "--yes must not silence the destructive tier"
    );
}

#[test]
fn the_destructive_flag_approves_both_tiers() {
    let storage_dir = unique_temp_dir("cli-bot-flag-destructive");
    fs::create_dir_all(&storage_dir).expect("storage dir should exist");
    let (command, target) = touch_command(&storage_dir);
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            plan_for(&absent_rm_command(&storage_dir)),
            plan_for(&command),
        ],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent();

    for request in ["delete it", "create the file"] {
        let mut cli = session_cli(config_path.clone(), vec![request]);
        cli.i_approve_destructive_commands = true;
        run_with_prompter(cli, &prompter).expect("both tiers should run unattended");
    }

    assert_eq!(prompter.asked(), Vec::new());
    assert!(target.is_file(), "the flag implies --yes");
}

#[test]
fn assume_yes_in_the_config_behaves_as_the_yes_flag() {
    let storage_dir = unique_temp_dir("cli-bot-assume-yes");
    fs::create_dir_all(&storage_dir).expect("storage dir should exist");
    let (command, target) = touch_command(&storage_dir);
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            plan_for(&command),
            plan_for(&absent_rm_command(&storage_dir)),
        ],
    );
    let config_path = ConfigOptions {
        assume_yes: true,
        ..ConfigOptions::with_session(&storage_dir)
    }
    .write(server.base_url());

    let quiet = ScriptedPrompter::silent();
    run_with_prompter(
        session_cli(config_path.clone(), vec!["create the file"]),
        &quiet,
    )
    .expect("assume_yes should approve the ordinary tier");
    assert_eq!(quiet.asked(), Vec::new());
    assert!(target.is_file());

    let asked = ScriptedPrompter::silent().with_confirmations([false]);
    run_with_prompter(session_cli(config_path, vec!["delete it"]), &asked)
        .expect("a declined command is not an error");
    assert_eq!(
        asked.confirmations_shown().len(),
        1,
        "no config key may pre-approve the destructive tier"
    );
}

#[test]
fn without_a_terminal_an_unapproved_command_fails_and_runs_nothing() {
    let storage_dir = unique_temp_dir("cli-bot-no-terminal");
    fs::create_dir_all(&storage_dir).expect("storage dir should exist");
    let (command, target) = touch_command(&storage_dir);
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            plan_for(&command),
            plan_for(&absent_rm_command(&storage_dir)),
        ],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent().without_dialogs();

    let error = run_with_prompter(
        session_cli(config_path.clone(), vec!["create the file"]),
        &prompter,
    )
    .expect_err("a state-changing command needs approval");
    let message = format!("{error:#}");
    assert!(message.contains("state-changing"), "{message}");
    assert!(message.contains("--yes"), "{message}");
    assert!(!target.exists(), "nothing may run without approval");

    let error = run_with_prompter(session_cli(config_path, vec!["delete it"]), &prompter)
        .expect_err("a destructive command needs approval");
    let message = format!("{error:#}");
    assert!(message.contains("destructive"), "{message}");
    assert!(
        message.contains("--i-approve-destructive-commands"),
        "{message}"
    );
    assert_eq!(prompter.asked(), Vec::new(), "no prompt may be attempted");
}

#[test]
fn a_read_only_command_runs_without_any_prompt() {
    let storage_dir = unique_temp_dir("cli-bot-tier-read-only");
    let server =
        MockOllamaServer::start(Arc::new(Mutex::new(Vec::new())), vec![plan_for("ls -la")]);
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent();

    run_with_prompter(session_cli(config_path, vec!["list files"]), &prompter)
        .expect("a read-only command should run");

    assert_eq!(prompter.asked(), Vec::new());
    let turn = &read_default_session(&storage_dir)["turns"][0];
    assert_eq!(turn["risk"], "read-only");
    assert_eq!(turn["confirmation_required"], false);
    assert_eq!(turn["execution"]["executed"], true);
}

#[test]
fn a_state_changing_command_is_confirmed_with_yes_as_the_default() {
    let storage_dir = unique_temp_dir("cli-bot-tier-state-changing");
    fs::create_dir_all(&storage_dir).expect("storage dir should exist");
    let target = storage_dir.join("created-by-the-test");
    let command = format!("touch {}", path_to_string(&target));
    let server =
        MockOllamaServer::start(Arc::new(Mutex::new(Vec::new())), vec![plan_for(&command)]);
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent().with_confirmations([true]);

    run_with_prompter(session_cli(config_path, vec!["create the file"]), &prompter)
        .expect("an approved command should run");

    let shown = prompter.confirmations_shown();
    assert_eq!(shown.len(), 1);
    assert!(shown[0].0.starts_with("Approve?"), "{:?}", shown[0].0);
    assert!(shown[0].1, "the ordinary tier defaults to yes");
    assert!(target.is_file(), "the approved command should have run");
    let turn = &read_default_session(&storage_dir)["turns"][0];
    assert_eq!(turn["risk"], "state-changing");
    assert_eq!(turn["confirmation_required"], true);
}

#[test]
fn a_destructive_command_is_confirmed_with_no_as_the_default_and_obeys_a_refusal() {
    let storage_dir = unique_temp_dir("cli-bot-tier-destructive");
    // Harmless even if the refusal were ignored: the path does not exist.
    let command = format!("rm -fr {}", path_to_string(&storage_dir.join("absent")));
    let server =
        MockOllamaServer::start(Arc::new(Mutex::new(Vec::new())), vec![plan_for(&command)]);
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let prompter = ScriptedPrompter::silent().with_confirmations([false]);

    run_with_prompter(session_cli(config_path, vec!["delete it"]), &prompter)
        .expect("a declined command is not an error");

    let shown = prompter.confirmations_shown();
    assert_eq!(shown.len(), 1);
    assert!(
        shown[0].0.starts_with("This command may be destructive"),
        "{:?}",
        shown[0].0
    );
    assert!(!shown[0].1, "the destructive tier defaults to no");
    let turn = &read_default_session(&storage_dir)["turns"][0];
    assert_eq!(turn["risk"], "destructive");
    assert_eq!(turn["execution"]["executed"], false);
}

#[test]
fn dry_run_saves_turn_as_not_executed_and_flags_destructive_command() {
    let storage_dir = unique_temp_dir("cli-bot-dry-run-session");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![MockResponse::generated(
            r#"{"summary":"Delete build output","unresolved":false,"commands":[{"command":"rm -rf ./target","description":"Delete target","potentially_destructive":false,"recommended":true,"rationale":"Removes build output"}]}"#,
        )],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let mut cli = session_cli(config_path, vec!["delete the build output"]);
    cli.dry_run = true;
    cli.benchmark = true;

    run(cli).expect("dry run should succeed without executing");

    let session = read_default_session(&storage_dir);
    let turn = &session["turns"][0];
    assert_eq!(turn["selected_command"], "rm -rf ./target");
    assert_eq!(turn["confirmation_required"], true);
    assert_eq!(turn["execution"]["executed"], false);
    assert!(turn["execution"]["exit_status"].is_null());
}

#[test]
fn unresolved_request_saves_text_response_in_session() {
    let storage_dir = unique_temp_dir("cli-bot-unresolved-session");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            MockResponse::generated(UNRESOLVED_PLAN),
            MockResponse::generated("maintenance"),
        ],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let mut cli = session_cli(config_path, vec!["spell mantainence"]);
    cli.benchmark = true;
    cli.verbose = true;

    run(cli).expect("unresolved fallback should succeed");

    let session = read_default_session(&storage_dir);
    let turn = &session["turns"][0];
    assert_eq!(turn["unresolved"], true);
    assert_eq!(turn["text_response"], "maintenance");
    assert!(turn["selected_command"].is_null());
}

#[test]
fn follow_up_request_carries_previous_turn_as_session_context() {
    let storage_dir = unique_temp_dir("cli-bot-follow-up-session");
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server = MockOllamaServer::start(
        capture.clone(),
        vec![
            MockResponse::generated(PRINTF_PLAN),
            MockResponse::generated(PRINTF_PLAN),
        ],
    );
    let config_path = ConfigOptions {
        capture_command_output: true,
        include_command_output_in_prompt: true,
        ..ConfigOptions::with_session(&storage_dir)
    }
    .write(server.base_url());

    run(session_cli(config_path.clone(), vec!["print ok"])).expect("first request should run");
    let mut follow_up = session_cli(config_path, vec!["do it again"]);
    follow_up.verbose = true;
    run(follow_up).expect("follow-up request should run");

    let requests = capture.lock().expect("capture should lock");
    assert_eq!(requests.len(), 2);
    assert!(!requests[0].body.contains("selected_command: printf ok"));
    assert!(requests[1].body.contains("1. request: print ok"));
    assert!(requests[1].body.contains("selected_command: printf ok"));
    assert!(requests[1].body.contains("executed with exit status 0"));
    assert!(requests[1].body.contains("stdout: ok"));
    assert_eq!(
        read_default_session(&storage_dir)["turns"]
            .as_array()
            .expect("turns should be an array")
            .len(),
        2
    );
}

#[test]
fn named_session_is_stored_separately_from_default() {
    let storage_dir = unique_temp_dir("cli-bot-named-session");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![MockResponse::generated(PRINTF_PLAN)],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let mut cli = session_cli(config_path, vec!["print ok"]);
    cli.session = Some("work".to_string());
    cli.verbose = true;
    cli.dry_run = true;

    run(cli).expect("named session request should succeed");

    assert!(storage_dir.join("sessions/work.json").is_file());
    assert!(!storage_dir.join("sessions/default.json").exists());
}

#[test]
fn auto_select_best_picks_recommended_command_from_several() {
    let storage_dir = unique_temp_dir("cli-bot-auto-select");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![MockResponse::generated(
            r#"{"summary":"Ping","unresolved":false,"commands":[{"command":"ping google.com","description":"Ping forever","recommended":false},{"command":"ping -c 5 google.com","description":"Ping five times","recommended":true,"rationale":"Bounded"}]}"#,
        )],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let mut cli = session_cli(config_path, vec!["ping google"]);
    cli.auto_select_best = true;
    cli.dry_run = true;

    run(cli).expect("auto selection should not need a terminal");

    let session = read_default_session(&storage_dir);
    let turn = &session["turns"][0];
    assert_eq!(turn["selected_command"], "ping -c 5 google.com");
    assert_eq!(turn["selected_command_rationale"], "Bounded");
    assert_eq!(
        turn["command_choices"]
            .as_array()
            .expect("choices should be an array")
            .len(),
        2
    );
}

#[test]
fn auto_select_best_falls_back_to_first_command_without_recommendation() {
    let storage_dir = unique_temp_dir("cli-bot-auto-select-first");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![MockResponse::generated(
            r#"{"unresolved":false,"commands":["uname -a","uname -r"]}"#,
        )],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let mut cli = session_cli(config_path, vec!["kernel"]);
    cli.auto_select_best = true;
    cli.dry_run = true;

    run(cli).expect("auto selection should fall back to the first command");

    assert_eq!(
        read_default_session(&storage_dir)["turns"][0]["selected_command"],
        "uname -a"
    );
}

#[test]
fn verbose_chat_request_succeeds_and_model_override_is_sent() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server = MockOllamaServer::start(capture.clone(), vec![MockResponse::chat(PRINTF_PLAN)]);
    let config_path = ConfigOptions {
        use_chat_api: true,
        ..ConfigOptions::default()
    }
    .write(server.base_url());
    let mut cli = sample_cli(config_path, vec!["print ok"]);
    cli.quiet = false;
    cli.verbose = true;
    cli.model = Some(" other-model:latest ".to_string());

    run(cli).expect("verbose chat request should succeed");

    let requests = capture.lock().expect("capture should lock");
    assert_eq!(requests[0].path, "/api/chat");
    assert!(requests[0].body.contains(r#""model":"other-model:latest""#));
}

#[test]
fn empty_model_override_is_rejected_before_any_request() {
    let config_path = ConfigOptions::default().write("http://127.0.0.1:1");
    let mut cli = sample_cli(config_path, vec!["print ok"]);
    cli.model = Some("   ".to_string());

    let error = run(cli).expect_err("blank model should be rejected");

    assert!(error.to_string().contains("--model must not be empty"));
}

#[test]
fn malformed_planner_output_is_reported_with_generated_text_when_verbose() {
    for use_chat_api in [false, true] {
        let reply = if use_chat_api {
            MockResponse::chat("I cannot help with that")
        } else {
            MockResponse::generated("I cannot help with that")
        };
        let server = MockOllamaServer::start(Arc::new(Mutex::new(Vec::new())), vec![reply]);
        let config_path = ConfigOptions {
            use_chat_api,
            ..ConfigOptions::default()
        }
        .write(server.base_url());
        let mut cli = sample_cli(config_path, vec!["print ok"]);
        cli.quiet = false;
        cli.verbose = true;

        let error = run(cli).expect_err("text without JSON should fail");
        let message = format!("{error:#}");

        assert!(message.contains("did not contain a JSON object"));
        assert!(message.contains("I cannot help with that"));
    }
}

#[test]
fn invalid_planner_json_and_empty_command_list_are_errors() {
    let cases = [
        (
            r#"{"commands":[{"description":"no command field"}]}"#,
            "failed to parse planner JSON",
        ),
        (
            r#"{"summary":"Nothing","unresolved":false,"commands":[]}"#,
            "planner returned an empty command list",
        ),
    ];

    for (plan, expected) in cases {
        let server = MockOllamaServer::start(
            Arc::new(Mutex::new(Vec::new())),
            vec![MockResponse::generated(plan)],
        );
        let config_path = ConfigOptions::default().write(server.base_url());

        let error = run(sample_cli(config_path, vec!["print ok"])).expect_err("plan should fail");

        assert!(
            format!("{error:#}").contains(expected),
            "expected `{expected}` in `{error:#}`"
        );
    }
}

#[test]
fn ollama_error_status_and_undecodable_body_are_errors() {
    let cases = [
        (
            MockResponse::status(500, r#"{"error":"boom"}"#),
            "unsuccessful response",
        ),
        (
            MockResponse::json("not json"),
            "failed to decode Ollama response",
        ),
    ];

    for (reply, expected) in cases {
        let server = MockOllamaServer::start(Arc::new(Mutex::new(Vec::new())), vec![reply]);
        let config_path = ConfigOptions::default().write(server.base_url());
        let mut cli = sample_cli(config_path, vec!["print ok"]);
        cli.quiet = false;
        cli.verbose = true;

        let error = run(cli).expect_err("bad Ollama reply should fail");

        assert!(
            format!("{error:#}").contains(expected),
            "expected `{expected}` in `{error:#}`"
        );
    }
}

#[test]
fn check_fails_when_model_is_missing() {
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            MockResponse::json(r#"{"version":"0.6.0"}"#),
            MockResponse::json(r#"{"models":[{"name":"another:latest"}]}"#),
        ],
    );
    let config_path = ConfigOptions::default().write(server.base_url());
    let mut cli = sample_cli(config_path, vec![]);
    cli.check = true;
    cli.quiet = false;
    cli.verbose = true;

    let error = run(cli).expect_err("missing model should fail the check");

    assert!(error.to_string().contains("environment check failed"));
}

#[test]
fn check_fails_when_service_returns_an_error() {
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            MockResponse::status(500, "{}"),
            MockResponse::status(500, "{}"),
        ],
    );
    let config_path = ConfigOptions::default().write(server.base_url());
    let mut cli = sample_cli(config_path, vec![]);
    cli.check = true;
    cli.quiet = false;

    let error = run(cli).expect_err("failing service should fail the check");

    assert!(error.to_string().contains("environment check failed"));
}

#[test]
fn check_fails_when_editor_is_not_installed() {
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            MockResponse::json(r#"{"version":"0.6.0"}"#),
            MockResponse::json(r#"{"models":[{"name":"lfm2:latest"}]}"#),
        ],
    );
    let config_path = temp_config_with_editor(server.base_url(), "cli-bot-no-such-editor");
    let mut cli = sample_cli(config_path, vec![]);
    cli.check = true;
    cli.quiet = false;

    let error = run(cli).expect_err("unknown editor should fail the check");

    assert!(error.to_string().contains("environment check failed"));
}

#[test]
fn models_benchmark_writes_markdown_report_with_failures_and_fallbacks() {
    let capture = Arc::new(Mutex::new(Vec::new()));
    let server = MockOllamaServer::start(
        capture.clone(),
        vec![
            MockResponse::json(r#"{"version":"0.6.0"}"#),
            MockResponse::json(
                r#"{"models":[{"name":"good:latest","details":{"parameter_size":"1.2B"}},{"name":"bad:latest"}]}"#,
            ),
            // good:latest — one plan, one unresolved request with its fallback
            MockResponse::generated(PRINTF_PLAN),
            MockResponse::generated(UNRESOLVED_PLAN),
            MockResponse::generated("forty-two"),
            // bad:latest — no JSON, then a fallback that the service rejects
            MockResponse::generated("no json here"),
            MockResponse::generated(UNRESOLVED_PLAN),
            MockResponse::status(500, "{}"),
        ],
    );
    let config_path = ConfigOptions {
        benchmark_models: vec!["good:latest", "bad:latest"],
        benchmark_queries: vec!["print ok", "meaning | of life"],
        ..ConfigOptions::default()
    }
    .write(server.base_url());
    let report_path = unique_temp_dir("cli-bot-benchmark-report").with_extension("md");
    let mut cli = sample_cli(config_path, vec![]);
    cli.models_benchmark = Some(report_path.clone());

    run(cli).expect("models benchmark should write a report");

    let report = fs::read_to_string(&report_path).expect("report should exist");
    assert!(report.starts_with("# Model Benchmark Report"));
    assert!(report.contains("- Ollama Version: 0.6.0"));
    assert!(report.contains("| good:latest | 1.2B |"));
    assert!(report.contains("| bad:latest | unknown |"));
    assert!(report.contains("meaning \\| of life"));
    assert!(report.contains("| 1 | good:latest | 1.2B | 100.0% |"));
    // The ranking gained a tokens-per-second column; a model that reported
    // no rate shows `n/a` there.
    assert!(
        report.contains("| 2 | bad:latest | unknown | 0.0% | n/a | failed |"),
        "{report}"
    );
    assert!(report.contains("printf ok [recommended]"));
    assert!(report.contains("forty-two"));
    assert!(report.contains("did not contain a JSON object"));

    let requests = capture.lock().expect("capture should lock");
    assert_eq!(requests.len(), 8);
    assert!(requests[2].body.contains(r#""model":"good:latest""#));
    assert!(requests[5].body.contains(r#""model":"bad:latest""#));
}

#[test]
fn models_benchmark_takes_report_path_from_trailing_argument() {
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![
            MockResponse::json(r#"{"version":"0.6.0"}"#),
            MockResponse::json(r#"{"models":[]}"#),
            MockResponse::generated(PRINTF_PLAN),
        ],
    );
    let config_path = ConfigOptions {
        benchmark_models: vec!["good:latest"],
        benchmark_queries: vec!["print ok"],
        ..ConfigOptions::default()
    }
    .write(server.base_url());
    let report_path = unique_temp_dir("cli-bot-benchmark-arg").with_extension("md");
    // `--models-benchmark report.md` parses the path as the request.
    let mut cli = sample_cli(config_path, vec![&path_to_string(&report_path)]);
    cli.models_benchmark = Some(PathBuf::from("-"));

    run(cli).expect("models benchmark should accept a trailing path");

    assert!(report_path.is_file());
}

#[test]
fn models_benchmark_requires_models_and_queries() {
    let cases = [
        (vec![], vec!["print ok"], "models_benchmark.models is empty"),
        (
            vec!["good:latest"],
            vec![],
            "models_benchmark.queries is empty",
        ),
    ];

    for (benchmark_models, benchmark_queries, expected) in cases {
        let config_path = ConfigOptions {
            benchmark_models,
            benchmark_queries,
            ..ConfigOptions::default()
        }
        .write("http://127.0.0.1:1");
        let mut cli = sample_cli(config_path, vec![]);
        cli.models_benchmark = Some(PathBuf::from("-"));

        let error = run(cli).expect_err("empty benchmark config should fail");

        assert!(error.to_string().contains(expected));
    }
}

#[test]
fn session_clear_removes_the_session_file_and_reports_missing_one() {
    let storage_dir = unique_temp_dir("cli-bot-session-clear");
    let server = MockOllamaServer::start(
        Arc::new(Mutex::new(Vec::new())),
        vec![MockResponse::generated(PRINTF_PLAN)],
    );
    let config_path = ConfigOptions::with_session(&storage_dir).write(server.base_url());
    let mut seed = session_cli(config_path.clone(), vec!["print ok"]);
    seed.dry_run = true;
    run(seed).expect("seeding request should succeed");
    assert!(storage_dir.join("sessions/default.json").is_file());

    for _ in 0..2 {
        let mut clear = session_cli(config_path.clone(), vec![]);
        clear.session_clear = true;
        run(clear).expect("session clear should succeed, also when nothing is stored");
        assert!(!storage_dir.join("sessions/default.json").exists());
    }

    let mut list = session_cli(config_path, vec![]);
    list.session_list = true;
    run(list).expect("listing no sessions should succeed");
}

#[test]
fn session_commands_reject_conflicting_or_disabled_use() {
    let storage_dir = unique_temp_dir("cli-bot-session-conflict");
    let config_path = ConfigOptions::with_session(&storage_dir).write("http://127.0.0.1:1");

    let mut both = session_cli(config_path.clone(), vec![]);
    both.session_show = true;
    both.session_clear = true;
    let error = run(both).expect_err("show and clear together should fail");
    assert!(error.to_string().contains("cannot be used together"));

    let mut disabled = session_cli(config_path.clone(), vec![]);
    disabled.session_show = true;
    disabled.no_session = true;
    let error = run(disabled).expect_err("session command without session memory should fail");
    assert!(error.to_string().contains("session memory is disabled"));

    let mut bad_name = session_cli(config_path, vec![]);
    bad_name.session_show = true;
    bad_name.session = Some("../escape".to_string());
    let error = run(bad_name).expect_err("path-like session name should fail");
    assert!(error.to_string().contains("session name must contain only"));
}

/// A CLI that prints its normal output, keeps session memory on, and executes
/// the selected command unless the test sets `dry_run`.
fn session_cli(config_path: PathBuf, request: Vec<&str>) -> Cli {
    let mut cli = sample_cli(config_path, request);
    cli.quiet = false;
    cli.dry_run = false;
    cli.no_session = false;
    cli
}

fn read_default_session(storage_dir: &Path) -> serde_json::Value {
    let raw = fs::read_to_string(storage_dir.join("sessions/default.json"))
        .expect("default session file should exist");
    serde_json::from_str(&raw).expect("session file should be JSON")
}

/// Writes a default config, then points `preferred_editor` at `editor`.
fn temp_config_with_editor(base_url: &str, editor: &str) -> PathBuf {
    let config_path = ConfigOptions::default().write(base_url);
    let config = fs::read_to_string(&config_path).expect("config should read");
    fs::write(
        &config_path,
        config.replace(
            r#"preferred_editor = "/bin/sh""#,
            &format!(r#"preferred_editor = "{editor}""#),
        ),
    )
    .expect("config should write");
    config_path
}

fn sample_cli(config_path: PathBuf, request: Vec<&str>) -> Cli {
    Cli {
        request: request.into_iter().map(ToString::to_string).collect(),
        config: Some(config_path),
        model: None,
        check: false,
        models_benchmark: None,
        auto_select_best: false,
        color: ColorMode::Never,
        yes: false,
        i_approve_destructive_commands: false,
        dry_run: true,
        print_plan: false,
        benchmark: false,
        verbose: false,
        quiet: true,
        interactive: false,
        session: None,
        no_session: true,
        session_show: false,
        session_list: false,
        session_clear: false,
    }
}

fn write_config(
    base_url: &str,
    use_chat_api: bool,
    require_editor: bool,
    storage_dir: Option<PathBuf>,
    retention_days: Option<u64>,
    session_enabled: bool,
) -> PathBuf {
    ConfigOptions {
        use_chat_api,
        require_editor,
        storage_dir,
        retention_days,
        session_enabled,
        ..ConfigOptions::default()
    }
    .write(base_url)
}

/// Settings a test may vary in the generated `cli-bot.toml`.
#[derive(Default)]
struct ConfigOptions {
    use_chat_api: bool,
    require_editor: bool,
    storage_dir: Option<PathBuf>,
    retention_days: Option<u64>,
    session_enabled: bool,
    capture_command_output: bool,
    include_command_output_in_prompt: bool,
    benchmark_models: Vec<&'static str>,
    benchmark_queries: Vec<&'static str>,
    assume_yes: bool,
    /// `false` restores the behaviour of versions before v0.4.0: nothing is
    /// ever confirmed.
    no_confirmation: bool,
    /// Seconds; `None` leaves the key out so the default applies.
    request_timeout_seconds: Option<u64>,
}

impl ConfigOptions {
    /// Session memory enabled and stored below `storage_dir`, so a test never
    /// touches the user's real session files.
    fn with_session(storage_dir: &Path) -> Self {
        Self {
            storage_dir: Some(storage_dir.to_path_buf()),
            session_enabled: true,
            ..Self::default()
        }
    }

    fn write(&self, base_url: &str) -> PathBuf {
        let temp_dir = unique_temp_dir("cli-bot-integration-test");
        fs::create_dir_all(&temp_dir).expect("temp dir should exist");
        let config_path = temp_dir.join("cli-bot.toml");
        let preferred_editor = if self.require_editor {
            "nvim"
        } else {
            "/bin/sh"
        };
        let storage_dir = self
            .storage_dir
            .as_deref()
            .map(path_to_string)
            .unwrap_or_else(|| "auto".to_string());
        let retention_days_line = self
            .retention_days
            .map(|value| format!("retention_days = {value}\n"))
            .unwrap_or_default();
        let config = format!(
            r#"
[ollama]
base_url = "{base_url}"
model = "lfm2:latest"
temperature = 0.0
use_chat_api = {use_chat_api}
{timeout_line}system_prompt = "Return JSON only"

[environment]
os = "auto"
distro = "auto"
preferred_package_manager = "auto"

[safety]
require_confirmation = {require_confirmation}
assume_yes = {assume_yes}
destructive_substrings = ["rm -rf"]

[ui]
selection_prompt = "Choose"
approval_prompt = "This command may be destructive. Approve execution?"
confirmation_prompt = "Approve?"
show_command_before_execution = true
auto_select_recommended = false

[execution]
shell = "/bin/sh"
shell_arg = "-c"
preferred_editor = "{preferred_editor}"

[session_memory]
enabled = {session_enabled}
default_name = "default"
scope = "global"
storage_dir = "{storage_dir}"
max_turns = 6
include_working_directory = true
save_text_responses = true
save_selected_commands = true
capture_command_output = {capture_command_output}
include_command_output_in_prompt = {include_command_output_in_prompt}
max_output_bytes = 8192
{retention_days_line}

[models_benchmark]
models = {benchmark_models:?}
queries = {benchmark_queries:?}
"#,
            base_url = base_url,
            use_chat_api = self.use_chat_api,
            timeout_line = self
                .request_timeout_seconds
                .map(|seconds| format!("request_timeout_seconds = {seconds}\n"))
                .unwrap_or_default(),
            assume_yes = self.assume_yes,
            require_confirmation = !self.no_confirmation,
            preferred_editor = preferred_editor,
            session_enabled = self.session_enabled,
            storage_dir = storage_dir,
            capture_command_output = self.capture_command_output,
            include_command_output_in_prompt = self.include_command_output_in_prompt,
            retention_days_line = retention_days_line,
            benchmark_models = self.benchmark_models,
            benchmark_queries = self.benchmark_queries,
        );
        fs::write(&config_path, config).expect("config should write");
        config_path
    }
}

struct MockOllamaServer {
    base_url: String,
    handle: Option<thread::JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl MockOllamaServer {
    fn start(capture: Arc<Mutex<Vec<CapturedRequest>>>, responses: Vec<MockResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        listener
            .set_nonblocking(true)
            .expect("listener should become nonblocking");
        let address = listener.local_addr().expect("address should resolve");
        let base_url = format!("http://{}", address);

        let shutdown = Arc::new(AtomicBool::new(false));
        let stopping = shutdown.clone();
        let handle = thread::spawn(move || {
            // Only a real request consumes a queued response, and the
            // shutdown flag is checked between polls, so the thread ends on
            // its own when the server is dropped. Nothing here may panic
            // once a test is already failing: a panic while unwinding
            // aborts the process and writes a 70 MB core file instead of
            // reporting the test failure.
            let mut index = 0;
            while index < responses.len() {
                if stopping.load(Ordering::Relaxed) {
                    return;
                }
                let mut stream = match listener.accept() {
                    Ok((stream, _)) => stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(_) => return,
                };
                let Some(request) = read_http_request(&mut stream) else {
                    continue;
                };
                capture.lock().expect("capture should lock").push(request);
                // Slept in slices, so dropping the server does not wait
                // out a delay the client has already given up on.
                let deadline = std::time::Instant::now() + responses[index].delay;
                while std::time::Instant::now() < deadline {
                    if stopping.load(Ordering::Relaxed) {
                        return;
                    }
                    thread::sleep(Duration::from_millis(25));
                }
                let payload = responses[index].body.clone();
                let http = format!(
                    "HTTP/1.1 {} Mock\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    responses[index].status,
                    payload.len(),
                    payload
                );
                let _ = stream
                    .write_all(http.as_bytes())
                    .and_then(|()| stream.flush());
                index += 1;
            }
        });

        Self {
            base_url,
            handle: Some(handle),
            shutdown,
        }
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for MockOllamaServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            // Never `expect` here: this runs while a failing test unwinds,
            // and a panic during unwinding aborts the process.
            let _ = handle.join();
        }
    }
}

struct MockResponse {
    status: u16,
    body: String,
    /// How long the server waits before replying, for timeout tests.
    delay: Duration,
}

impl MockResponse {
    fn json(body: &str) -> Self {
        Self::status(200, body)
    }

    fn status(status: u16, body: &str) -> Self {
        Self {
            status,
            body: body.to_string(),
            delay: Duration::ZERO,
        }
    }

    /// Replies only after `delay`, so a test can drive a client timeout.
    fn after(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// A `/api/generate` reply whose generated text is `text`.
    fn generated(text: &str) -> Self {
        Self::json(&serde_json::json!({ "response": text }).to_string())
    }

    /// A `/api/chat` reply whose message content is `text`.
    fn chat(text: &str) -> Self {
        Self::json(&serde_json::json!({ "message": { "content": text } }).to_string())
    }
}

#[derive(Debug)]
struct CapturedRequest {
    path: String,
    body: String,
}

/// Reads one HTTP request, or `None` when the peer sent nothing, which is
/// what a connection opened only to wake the server looks like.
fn read_http_request(stream: &mut std::net::TcpStream) -> Option<CapturedRequest> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut header_end = None;
    let mut content_length = 0_usize;

    loop {
        let bytes_read = stream.read(&mut chunk).ok()?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);

        if header_end.is_none()
            && let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n")
        {
            header_end = Some(position + 4);
            let headers = String::from_utf8_lossy(&buffer[..position + 4]);
            for line in headers.lines() {
                let lower = line.to_ascii_lowercase();
                if let Some((_, value)) = lower.split_once("content-length:") {
                    content_length = value.trim().parse::<usize>().expect("content length");
                }
            }
        }

        if let Some(header_end) = header_end
            && buffer.len() >= header_end + content_length
        {
            break;
        }
    }

    let header_end = header_end?;
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let path = headers
        .lines()
        .next()?
        .split_whitespace()
        .nth(1)?
        .to_string();
    let body =
        String::from_utf8_lossy(&buffer[header_end..header_end + content_length]).to_string();

    Some(CapturedRequest { path, body })
}

/// Sets a file's modification time, which is what pruning looks at.
fn set_modified(path: &Path, time: SystemTime) {
    fs::File::options()
        .write(true)
        .open(path)
        .expect("session file should open")
        .set_modified(time)
        .expect("modification time should be settable");
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    // Tests run in parallel, so the clock alone could repeat.
    let sequence = TEMP_DIR_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{unique}-{sequence}"))
}

fn current_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_millis()
}

fn path_to_string(path: &Path) -> String {
    path.display().to_string()
}
