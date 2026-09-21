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

From [Review 00001](reviews/00001-Main_Current_Code_State.md). Fixed and no
longer listed: `MAJ-01` and `MED-06` in v0.4.0 (PLAN-00002), and `MED-01` to
`MED-05`, `LOW-01`, and `LOW-02` in v0.4.1 (PLAN-00003).

- **`REV-00001-MED-07` The rules point at a number-allocation script that
  is not in the repository.** Five tracked documents tell the writer to run
  `.agents/skills/allocating-report-numbers/allocate-report.sh`, which
  commit `7c41d82` removed; `.agents` and `.opencode` are now ignored
  through `.git/info/exclude` and hold symlinks into
  `../opencode-template`. Track the script again, or have `just setup`
  create the links and say so in the documents.
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
  reports, not two.
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

Now that timeouts are configurable (`MED-01`), regenerating
`docs/models-benchmark-report-v0.3.2.md` is worthwhile: 11 of its 35 calls
failed on the old fixed 30-second limit, so its ranking partly measured
speed rather than answer quality. It needs the owner's hardware and a long
run.

### Tooling

- **A pseudo-terminal test harness.** `src/prompt.rs` sits at 71.65% line
  coverage because the bodies of `read_request`, `select`, and `confirm`
  call `dialoguer` and need a real terminal; a test that reached them would
  open a prompt and hang. A pty-backed harness, which needs a
  dev-dependency, would close the last uncovered part of the request flow.
  See PLAN-00002 STEP-08.
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
