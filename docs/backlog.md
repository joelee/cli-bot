# Backlog

Future work not covered by an active plan. Completed items are removed.
Items here are candidates, not release promises. Shell command generation
stays the primary product behaviour.

## Candidate features

- **External dictionary and language tools.** Use dictionary, spelling, or
  thesaurus tools when available, to improve non-command requests such as
  spelling, definitions, and synonyms.
- **Better non-command result types.** Go beyond plain text and
  clarification: concise help, explanations, or command comparisons, without
  executing anything.
- **Platform-aware package helpers.** Richer prompt examples per platform,
  and stronger handling of package search, upgrade, and uninstall.
- **Session storage.** Move from JSON files to SQLite if captured command
  output becomes heavily used; finer-grained pruning than `retention_days`.
  See [Session Memory](session-memory.md).
- **Windows support.** The release workflow ships a Windows binary that
  cannot start (REV-00001-MED-08): config resolution needs `HOME`, the
  session store needs `HOME` or `XDG_STATE_HOME`, the default shell is
  `/bin/sh`, and `OperatingSystem` has no Windows value. Supporting it
  needs `USERPROFILE`/`APPDATA` paths, a Windows shell, an OS value, and a
  CI job.

## Agent suggested next steps

### Rules the code does not meet yet

These are the `Known deviations` of `AGENTS.md`. Each changes behaviour or
is large, so each needs its own plan.

- **Config lookup order.** Read `--config`, then `CLI_BOT_CONFIG_FILE`,
  `$XDG_CONFIG_HOME/cli-bot/config.toml`, `~/.config/cli-bot/config.toml`,
  `/etc/cli-bot/config.toml`, and `./config.toml`. Today only `--config`,
  `~/.config/cli-bot/cli-bot.toml`, and `/etc/cli-bot.toml` are read. Keep
  reading the old `cli-bot.toml` paths so existing installs still work.
- **Structured logging.** Replace the `[verbose]` `eprintln!` lines with a
  logger that has the levels Error, Warning, Info, Verbose, and Debug,
  timestamps, and a component name. Do not log full prompts or command
  output above Debug.
- **Rustdoc for the public API.** `Cli`, `run`, `OutputStyler`, and
  `ColorMode` are public; most items have no rustdoc.

### Open findings of Review 00001 (2026-09-20)

From [Review 00001](reviews/00001-Main_Current_Code_State.md), which
replaces the handover-review list kept here before. `REV-00001-MAJ-01`
(destructive commands run without confirmation) and `REV-00001-MED-06`
(the approval and selection paths have no test) are not listed, because
PLAN-00002 covers them. The "middle tier" for state-changing commands,
formerly a candidate feature here, is part of MAJ-01.

Medium, in the order the review recommends:

- **`REV-00001-MED-02` One unreadable session file stops every
  invocation.** `prune_expired` runs before any flag is read and fails on
  the first unparsable file, so `--check`, `--no-session`, and
  `--session-clear` all stop. Skip and report bad files, do not prune when
  the invocation does not use sessions, and write atomically through a
  temporary file and `rename`.
- **`REV-00001-MED-05` Session history is readable by other local users.**
  Files are written `644` and the folder `755`. Create them `600` and
  `700` on Unix, and tighten existing ones when next written. Do it with
  MED-02, since both change how sessions are written.
- **`REV-00001-MED-01` Planner calls fail after a fixed 30 seconds.**
  Add `[ollama] request_timeout_seconds` (default about 300, `0` for none)
  and a short connect timeout. Then regenerate
  `docs/models-benchmark-report-v0.3.2.md`: 11 of its 35 calls failed on
  this timeout, so its ranking partly measures speed, not answer quality.
- **`REV-00001-MED-03` Interactive mode ends on the first planner error.**
  Error kinds are told apart by matching message text, so a timeout or an
  unparsable plan ends the session. Introduce typed errors, keep the
  prompt open after a planner error, and exit 0 at end of input. Do it
  with LOW-01.
- **`REV-00001-MED-04` Output capture breaks interactive commands.** With
  `capture_command_output = true` the command runs through
  `Command::output()`, so `nvim`, `sudo`, `less`, and `ssh` cannot work
  and nothing streams. Inherit stdin and copy output while collecting it,
  or skip capture for known full-screen programs and document the limit.
- **`REV-00001-MED-07` The rules point at a number-allocation script that
  is not in the repository.** Five tracked documents tell the writer to run
  `.agents/skills/allocating-report-numbers/allocate-report.sh`, which
  commit `7c41d82` removed; `.agents` and `.opencode` are now ignored
  through `.git/info/exclude` and hold symlinks into
  `../opencode-template`. Track the script again, or have `just setup`
  create the links and say so in the documents.

Low and Info:

- **`REV-00001-LOW-01` A failed command loses its exit status, reads
  oddly, and is not remembered.** Return the result for a non-zero exit,
  save the turn, print `command exited with status 3`, and exit with the
  child's code.
- **`REV-00001-LOW-02` Session file names depend on an unspecified hash.**
  `stable_path_hash` uses `DefaultHasher`, whose algorithm may change
  between Rust releases and then orphans working-directory sessions. Use a
  fixed algorithm and fall back to the old name once.
- **`REV-00001-LOW-03` The default config template carries the
  maintainer's own settings.** The file written on first run sets
  `preferred_editor = "nvim"`, which overrides `$EDITOR` and fails
  `--check` without Neovim, and lists eight benchmark models up to 120B.
  Ship a separate template, and match only the first word of an editor
  value such as `code --wait`.
- **`REV-00001-LOW-04` `--check` needs the exact model tag.** A config of
  `model = "gemma4"` works for requests but is reported missing. Compare
  against `<name>:latest` when the name has no tag.
- **`REV-00001-LOW-05` The release workflow gives every job write access
  and uses movable action references.** Set `contents: read` at the top and
  `contents: write` on the `release` job only, pin third-party actions in
  `release.yml` to commit hashes, and drop the cache from the `crates` job
  that holds the crates.io token.
- **`REV-00001-LOW-06` Documents that no longer match the code state.**
  `docs/usage.md` and `docs/session-memory.md` credit session memory to
  `0.3.1`, a version never published; there are now three benchmark
  reports, not two; `CHANGELOG.md` mixes the old `## [0.3.2] - <date>`
  heading with the `## vX.Y.Z - <UTC timestamp>` form that
  `scripts/check-release-tag.sh` requires from v0.3.3.
- **`REV-00001-LOW-07` Integration tests leave their temporary folders
  behind.** Each `just check` leaves about 60 folders under the system
  temporary directory. Return a guard that removes the folder on `Drop`,
  or take `tempfile` as a dev-dependency.
- **`REV-00001-INFO-01` The published crate carries repository records.**
  `cargo package --list` shows 53 files, 26 of them documents, workflows,
  and scripts. An `include` list in `Cargo.toml` would keep future records
  out of the crate.
- **`REV-00001-INFO-02` The benchmark host section misreports
  unified-memory GPUs.** `detect_linux_gpu_vram` reads
  `mem_info_vram_total`, which on an APU is the small fixed carve-out, so
  the v0.3.2 report shows `GPU VRAM: 0.5 GiB` on a 125 GiB machine. Read
  `mem_info_gtt_total` as well, or label the value "dedicated VRAM".

Open questions the review could not answer without more evidence are in its
`Open questions` section: whether a thinking model breaks
`extract_json_document` through `/api/generate`, whether the `release`
environment is restricted to `v*` tags, and the mixed action versions
across the four workflows.

### Tooling

- **Duplicated helpers.** `is_executable_file` exists in `src/config.rs`
  and `src/environment.rs`; byte-limited truncation exists in
  `src/shell.rs` and `src/session.rs`.
- **macOS CI job.** cli-bot supports macOS, but CI runs on Linux only.
- **Release environment setup.** Create the `release` GitHub environment
  with a required reviewer and the `CARGO_REGISTRY_TOKEN` secret; the
  Release workflow needs both before the next version can publish.
- **Stale benchmark reports.** `docs/models-benchmark-report.md`,
  `docs/models-benchmark-report1.md`, and
  `docs/models-benchmark-report-v0.3.2.md` are three generated reports;
  keep one, or move them under `docs/benchmarks/` with dated names. See
  `REV-00001-LOW-06`.
