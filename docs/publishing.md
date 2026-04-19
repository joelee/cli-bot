# Publishing

## Important Note

Publishing to crates.io uploads Rust source code, not a prebuilt executable.

After publishing, users install the CLI with:

```bash
cargo install cli-bot
```

That command compiles `cli-bot` on the user's machine.

If you want prebuilt binaries, publish release artifacts separately, for example through GitHub Releases.

## Before First Publish

Fill in the remaining package metadata in `Cargo.toml` before publishing publicly:

- `license` or `license-file`
- `repository`
- optionally `homepage`
- optionally `documentation`

The crate already includes some useful metadata:

- `description`
- `readme`
- `keywords`
- `categories`

## Create a crates.io Account Token

1. Sign in at `https://crates.io/`.
2. Create an API token in your account settings.
3. Authenticate locally:

```bash
cargo login
```

## Verify the Package Locally

Run the standard checks:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Then build the publishable archive locally:

```bash
cargo package
```

You can inspect the generated package contents with:

```bash
cargo package --list
```

## GitHub Actions

The repository includes these workflows:

1. `.github/workflows/release-checks.yml`
2. `.github/workflows/unit-coverage.yml`
3. `.github/workflows/coverage-pages.yml`

### Release Checks

`release-checks.yml` does two things:

1. Runs release checks on pull requests and pushes to `main`
2. Prepares a publishable `.crate` artifact on version tags like `v0.1.0` or manual workflow dispatch

The verification job runs:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo package
cargo package --list
```

On matching tags or manual runs, the workflow also uploads the packaged crate from `target/package/*.crate` as a GitHub Actions artifact.

### Unit Coverage

`unit-coverage.yml` runs `bash ./scripts/coverage-unit.sh` and uploads:

- `target/coverage/html/`
- `target/coverage/unit-report.txt`
- `target/coverage/unit-tests.profdata`
- `target/coverage/lcov.info`

It also uploads the LCOV report to Codecov.

### Coverage Pages

`coverage-pages.yml` rebuilds the unit-test HTML coverage report on pushes to `main` and deploys it to GitHub Pages.

Before the first deployment succeeds, enable Pages in the repository settings:

1. Open `Settings -> Pages`
2. Set the source to `GitHub Actions`
3. Allow the workflow deployment to publish the site

Expected coverage site URL:

```text
https://joelee.github.io/cli-bot/
```

## Publish

```bash
cargo publish
```

After the first publish, users can install it with:

```bash
cargo install cli-bot
```

## Publish Updates

For each new release:

1. Update `version` in `Cargo.toml`.
2. Re-run the local verification steps.
3. Run `cargo publish`.

Users can then upgrade with:

```bash
cargo install cli-bot --force
```

## Optional: Prebuilt Binaries

If you want users to avoid a local Rust build, publish binaries separately.

Common approach:

1. Build release artifacts in CI for each target platform.
2. Upload them to a GitHub Release.
3. Optionally support tools like `cargo-binstall`.
