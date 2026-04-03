# Install Ollama

`cli-bot` depends on a locally running Ollama endpoint.

Before using `cli-bot`, make sure Ollama is installed, running, and has the configured model available.

## Install Ollama

Follow the official installation instructions for your platform:

- `https://ollama.com/download`

## Start the Ollama Service

After installation, start Ollama so the local HTTP endpoint is available.

Typical default endpoint:

```text
http://127.0.0.1:11434
```

You can verify the service is responding with:

```bash
ollama list
```

## Pull the Default Model

`cli-bot` currently defaults to `lfm2:latest`.

Pull it with:

```bash
ollama pull lfm2:latest
```

## Use a Different Model

If you want a different model, pull it first:

```bash
ollama pull qwen3.5:latest
```

Then either:

- update `cli-bot.toml`
- or override it at runtime with `--model`

Example:

```bash
cli-bot --model qwen3.5:latest "Ping google five times"
```

## Verify with cli-bot

Once Ollama is running and the model is installed, verify the setup with:

```bash
cli-bot --config ./cli-bot.toml --check
```

This confirms:

- the config file can be loaded
- the Ollama service is reachable
- the configured model exists
- the preferred editor is available
