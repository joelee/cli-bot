# crates.io Release

This document explains how to manually publish a new `cli-bot` release to crates.io.

## Release Order

Recommended order for a new release:

1. Update the code and docs
2. Bump the version in `Cargo.toml`
3. Run local verification
4. Package and publish to crates.io
5. Update the Homebrew formula in `homebrew-oss`

## Update The Version

Edit `Cargo.toml`:

```toml
[package]
version = "0.1.2"
```

Use the next semver version you want to release.

## Run Local Verification

Before publishing, run:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Build The Publishable Package

Create the crate archive locally:

```bash
cargo package
```

Inspect the packaged contents if needed:

```bash
cargo package --list
```

## Authenticate With crates.io

If this machine is not already authenticated:

```bash
cargo login
```

You will need a crates.io API token from:

- `https://crates.io/settings/tokens`

## Publish

Publish the new version:

```bash
cargo publish
```

Or use the local release helper script after pushing the GitHub tag/release:

```bash
scripts/release.sh v0.1.2
```

The script:

- loads `.env` if present
- reads `HOMEBREW_FORMULA_FILE` from the environment or `.env`
- verifies the requested tag matches `Cargo.toml`
- runs `fmt`, `clippy`, `test`, and `package`
- publishes to crates.io
- waits for the crate URL to become available
- computes the checksum from the published crate artifact
- updates the Homebrew formula file with the new crate URL and published checksum

Important:

- do not use the SHA256 of the local `target/package/*.crate` file for Homebrew
- Homebrew should use the checksum of the crate downloaded from crates.io
- `scripts/release.sh` now computes the checksum from the published crates.io artifact automatically

Example `.env`:

```bash
HOMEBREW_FORMULA_FILE="$HOME/Projects/MyOSS/homebrew-oss/Formula/cli-bot.rb"
```

## Verify The Release

Check the published crate metadata:

```bash
curl -s https://crates.io/api/v1/crates/cli-bot | jq .
```

Or open the crate page:

- `https://crates.io/crates/cli-bot`

## Get The Published Checksum

After publishing, the Homebrew formula will need the new checksum.

You can fetch it with:

```bash
curl -s https://crates.io/api/v1/crates/cli-bot | jq -r '.versions[0].checksum'
```

## After Publishing

After the new crate version is live:

1. Update `https://github.com/joelee/homebrew-oss/Formula/cli-bot.rb`
2. Change the crate `url` to the new version
3. Change the `sha256` to the published checksum
4. Push the Homebrew tap update

See [Homebrew](homebrew.md) for the Homebrew side of the release.

## Typical Manual Release Session

Example for version `0.1.2`:

```bash
# edit Cargo.toml
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo package
cargo publish
curl -s https://crates.io/api/v1/crates/cli-bot | jq -r '.versions[0].checksum'
```

Or with the helper script:

```bash
scripts/release.sh v0.1.2
```

## Notes

- crates.io publishes source code, not prebuilt binaries
- users install from crates.io with `cargo install cli-bot`
- publish to crates.io before updating Homebrew
