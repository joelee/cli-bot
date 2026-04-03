# Homebrew

This document explains how to manually update the Homebrew formula for `cli-bot` after publishing a new release.

The Homebrew tap for this project lives at:

- GitHub: `https://github.com/joelee/homebrew-oss`
- For Local path check `$HOMEBREW_TAP_REPO_PATH`

The formula file to update is:

- `https://github.com/joelee/homebrew-oss/blob/main/Formula/cli-bot.rb`
- for local file: `$HOMEBREW_TAP_REPO_PATH/Formula/cli-bot.rb`

## When To Update It

Update the Homebrew formula after publishing a new `cli-bot` version to crates.io.

Example release flow:

1. Bump `version` in `Cargo.toml`
2. Publish the new crate to crates.io
3. Update the Homebrew formula to point to the new crate version and checksum

## What To Change

In `Formula/cli-bot.rb`, update these fields:

- `url`
- `sha256`

Example:

```ruby
url "https://static.crates.io/crates/cli-bot/cli-bot-0.1.1.crate"
sha256 "9a2f13aad6ad135793b0ae7b29e6ffba5efbf67dc97d813947da32907f0501af"
```

For a new release like `0.1.2`, the `url` should become:

```ruby
url "https://static.crates.io/crates/cli-bot/cli-bot-0.1.2.crate"
```

## How To Get The New Checksum

Option 1: get it from crates.io metadata:

```bash
curl -s https://crates.io/api/v1/crates/cli-bot | jq -r '.versions[0].checksum'
```

Option 2: compute it directly from the crate download:

```bash
curl -L https://static.crates.io/crates/cli-bot/cli-bot-0.1.2.crate | shasum -a 256
```

## Manual Update Steps

From the tap repository:

```bash
cd /home/joel/Projects/MyOSS/homebrew-oss
```

Edit `Formula/cli-bot.rb` and update the `url` and `sha256`.

Then commit and push the tap update:

```bash
git add Formula/cli-bot.rb
git commit -m "cli-bot 0.1.2"
git push
```

## Recommended Validation On macOS

After updating the formula, validate it on a macOS machine:

```bash
brew install --build-from-source joelee/oss/cli-bot
brew test joelee/oss/cli-bot
brew audit --strict joelee/oss/cli-bot
```

## User Install Command

Users install `cli-bot` from the tap with:

```bash
brew install joelee/oss/cli-bot
```

## Notes

- The Homebrew formula currently installs from the crates.io `.crate` archive.
- If the tap is updated before crates.io has the new version available, the Homebrew install will fail.
- Publish the crate first, then update the tap.
