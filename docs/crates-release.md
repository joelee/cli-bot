# crates.io Release

Releases are tag-driven and published by the `Release` GitHub workflow, never
by hand from a developer machine. This document walks the flow defined in
`AGENTS.md` § Release workflow; see also [Homebrew](homebrew.md).

## One-time setup

In the GitHub repository settings:

1. Create a `release` environment with a required reviewer (the project
   owner). The workflow's publish job waits for that approval.
2. Add a `CARGO_REGISTRY_TOKEN` secret to the `release` environment, holding
   an API token from `https://crates.io/settings/tokens`.

## Release order

1. The delivering plan bumps the version in `Cargo.toml` and `Cargo.lock`
   and writes the `docs/release/vX.Y.Z.md` draft
2. Local verification: `just ci`
3. Finalise the release records in one `release: vX.Y.Z - <top feature>`
   commit
4. Pull request into `main`; checks must pass
5. The owner tags the merge commit `vX.Y.Z` and pushes the tag
6. The Release workflow verifies, builds, and publishes after the owner
   approves the pending `release` deployment
7. Update the Homebrew tap formula with `scripts/update-homebrew-formula.sh`

## Update the version

Edit `Cargo.toml`:

```toml
[package]
version = "0.3.3"
```

## Run local verification

```bash
just ci
```

This runs the format check, clippy, the script syntax check, the Markdown
link check, the tests, the 80% coverage gate, a locked build, the
supply-chain audit, a publish dry run, and the workflow lint. See the
[Developer Guide](developer-guide.md).

## Finalise the release records

In one commit, `release: vX.Y.Z - <top feature>`:

- `CHANGELOG.md`: rename `Unreleased` to `vX.Y.Z - <UTC timestamp of this
  commit>` and add a fresh `Unreleased` above it.
- `docs/release/vX.Y.Z.md`: remove the draft line; name the date, plan, and
  PR, not a commit hash; use absolute links pinned to the tag, because the
  GitHub release page cannot resolve relative ones.
- `README.md` and other docs: remove pre-release wording.

Then check the tag and records locally:

```bash
scripts/check-release-tag.sh vX.Y.Z
```

## Publish

The owner pushes the tag:

```bash
git tag vX.Y.Z && git push origin vX.Y.Z
```

The Release workflow then:

- verifies the tag, the version, and the release records
  (`scripts/check-release-tag.sh`)
- verifies the package builds as a crates.io dependency
  (`cargo publish --dry-run --locked`)
- builds binaries for Linux, macOS, and Windows
- waits for the owner to approve the pending `release` deployment
- publishes to crates.io with the `CARGO_REGISTRY_TOKEN` secret
- creates the GitHub release from `docs/release/vX.Y.Z.md` and attaches the
  binaries

A published crate version can only be yanked, never replaced. Never create
the GitHub release by hand.

## Verify the release

- `https://crates.io/crates/cli-bot`
- `https://github.com/joelee/cli-bot/releases`

## After publishing

Once crates.io has the version, run from this repository:

```bash
scripts/update-homebrew-formula.sh vX.Y.Z
```

The script points `Formula/cli-bot.rb` in the tap (`../homebrew-oss`) at the
published crate and its crates.io checksum. Commit the change on a branch in
the tap, never on its `main`; the owner pushes it and merges once the tap's
macOS formula test passes.

## Notes

- crates.io publishes source code, not prebuilt binaries; the GitHub release
  carries those
- users install from crates.io with `cargo install cli-bot`
- publish to crates.io before updating Homebrew
