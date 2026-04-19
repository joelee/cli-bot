use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use cli_bot::{Cli, ColorMode, run};

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

    let config_path = write_config(server.base_url(), true, true, None, None, false);
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

fn sample_cli(config_path: PathBuf, request: Vec<&str>) -> Cli {
    Cli {
        request: request.into_iter().map(ToString::to_string).collect(),
        config: Some(config_path),
        model: None,
        check: false,
        models_benchmark: None,
        auto_select_best: false,
        color: ColorMode::Never,
        dry_run: true,
        print_plan: false,
        benchmark: false,
        verbose: false,
        quiet: true,
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
    let temp_dir = unique_temp_dir("cli-bot-integration-test");
    fs::create_dir_all(&temp_dir).expect("temp dir should exist");
    let config_path = temp_dir.join("cli-bot.toml");
    let preferred_editor = if require_editor { "nvim" } else { "sh" };
    let storage_dir = storage_dir
        .as_deref()
        .map(path_to_string)
        .unwrap_or_else(|| "auto".to_string());
    let retention_days_line = retention_days
        .map(|value| format!("retention_days = {value}\n"))
        .unwrap_or_default();
    let config = format!(
        r#"
[ollama]
base_url = "{base_url}"
model = "lfm2:latest"
temperature = 0.0
use_chat_api = {use_chat_api}
system_prompt = "Return JSON only"

[environment]
os = "auto"
distro = "auto"
preferred_package_manager = "auto"

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
capture_command_output = false
include_command_output_in_prompt = false
max_output_bytes = 8192
{retention_days_line}

[models_benchmark]
models = []
queries = []
"#,
        base_url = base_url,
        use_chat_api = use_chat_api,
        preferred_editor = preferred_editor,
        session_enabled = session_enabled,
        storage_dir = storage_dir,
        retention_days_line = retention_days_line,
    );
    fs::write(&config_path, config).expect("config should write");
    config_path
}

struct MockOllamaServer {
    base_url: String,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockOllamaServer {
    fn start(capture: Arc<Mutex<Vec<CapturedRequest>>>, responses: Vec<MockResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        listener
            .set_nonblocking(true)
            .expect("listener should become nonblocking");
        let address = listener.local_addr().expect("address should resolve");
        let base_url = format!("http://{}", address);

        let handle = thread::spawn(move || {
            for response in responses {
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(error) => panic!("request should arrive: {error}"),
                    }
                };
                let request = read_http_request(&mut stream);
                capture.lock().expect("capture should lock").push(request);
                let payload = response.body;
                let http = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    payload.len(),
                    payload
                );
                stream
                    .write_all(http.as_bytes())
                    .expect("response should write");
                stream.flush().expect("response should flush");
            }
        });

        Self {
            base_url,
            handle: Some(handle),
        }
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for MockOllamaServer {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = TcpStream::connect(self.base_url.trim_start_matches("http://"))
                .and_then(|stream| stream.shutdown(Shutdown::Both));
            handle.join().expect("server thread should join");
        }
    }
}

struct MockResponse {
    body: String,
}

impl MockResponse {
    fn json(body: &str) -> Self {
        Self {
            body: body.to_string(),
        }
    }
}

#[derive(Debug)]
struct CapturedRequest {
    path: String,
    body: String,
}

fn read_http_request(stream: &mut std::net::TcpStream) -> CapturedRequest {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut header_end = None;
    let mut content_length = 0_usize;

    loop {
        let bytes_read = stream.read(&mut chunk).expect("request should read");
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

    let header_end = header_end.expect("headers should exist");
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let first_line = headers.lines().next().expect("request line should exist");
    let path = first_line
        .split_whitespace()
        .nth(1)
        .expect("path should exist")
        .to_string();
    let body =
        String::from_utf8_lossy(&buffer[header_end..header_end + content_length]).to_string();

    CapturedRequest { path, body }
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}"))
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
