# Model Benchmark Report

## Host

- Hostname: megamind
- OS: Omarchy
- Kernel: 7.2.5-3-omarchy
- CPU: AMD RYZEN AI MAX+ PRO 395 w/ Radeon 8060S
- GPU: 00.0 Display controller: Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics] (rev d1)
- GPU VRAM: 0.5 GiB
- Memory: 125.1 GiB
- Ollama Version: 0.32.13

## Models

| Model | Parameter Size |
| --- | --- |
| ornith-1.5:9b | 9.0B |
| qwen3.8:27b | 27.3B |
| muse-glimmer:latest | 27.9B |
| nemotron-3.5-lightning:30b | 32.9B |
| nemotron-3-super:120b | 123.6B |

## Summary Table

| Query | ornith-1.5:9b | qwen3.8:27b | muse-glimmer:latest | nemotron-3.5-lightning:30b | nemotron-3-super:120b |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ping google five times | 10057 | 18581 | failed | 24084 | failed |
| Print the last git log message | 11703 | 15780 | failed | 16796 | 20848 |
| What is my external IP address? | 7854 | 13963 | failed | failed | 21813 |
| Is neovim installed? | 12127 | 15855 | failed | failed | 21468 |
| 230*(123-22) | 7986 | 18270 | failed | 28650 | 16575 |
| List all markdown files recursively in my home directory | 7034 | 18826 | failed | failed | 16824 |
| What is the purpose of life? | 10540 | 11318 | 51319 | failed | 25530 |

## Model Summary

| Model | Parameter Size | Success Rate | Avg Total ms (ok) | Successful Queries | Failed Queries |
| --- | --- | ---: | ---: | ---: | ---: |
| ornith-1.5:9b | 9.0B | 100.0% | 9614 | 7 | 0 |
| qwen3.8:27b | 27.3B | 100.0% | 16084 | 7 | 0 |
| muse-glimmer:latest | 27.9B | 14.3% | 51319 | 1 | 6 |
| nemotron-3.5-lightning:30b | 32.9B | 42.9% | 23176 | 3 | 4 |
| nemotron-3-super:120b | 123.6B | 85.7% | 20509 | 6 | 1 |

## Ranking

Ranked by success rate first, then by average total milliseconds across successful queries.

| Rank | Model | Parameter Size | Success Rate | Avg Total ms (ok) |
| ---: | --- | --- | ---: | ---: |
| 1 | ornith-1.5:9b | 9.0B | 100.0% | 9614 |
| 2 | qwen3.8:27b | 27.3B | 100.0% | 16084 |
| 3 | nemotron-3-super:120b | 123.6B | 85.7% | 20509 |
| 4 | nemotron-3.5-lightning:30b | 32.9B | 42.9% | 23176 |
| 5 | muse-glimmer:latest | 27.9B | 14.3% | 51319 |

## Detailed Results

| Model | Query | Planner ms | Fallback ms | Total ms | Kind | Unresolved | Status |
| --- | --- | ---: | ---: | ---: | --- | --- | --- |
| ornith-1.5:9b | Ping google five times | 10057 | - | 10057 | command_plan | no | ok |
| qwen3.8:27b | Ping google five times | 18581 | - | 18581 | command_plan | no | ok |
| muse-glimmer:latest | Ping google five times | 30000 | - | 30000 | error | no | error |
| nemotron-3.5-lightning:30b | Ping google five times | 24084 | - | 24084 | command_plan | no | ok |
| nemotron-3-super:120b | Ping google five times | 30000 | - | 30000 | error | no | error |
| ornith-1.5:9b | Print the last git log message | 11703 | - | 11703 | command_plan | no | ok |
| qwen3.8:27b | Print the last git log message | 15780 | - | 15780 | command_plan | no | ok |
| muse-glimmer:latest | Print the last git log message | 30000 | - | 30000 | error | no | error |
| nemotron-3.5-lightning:30b | Print the last git log message | 16796 | - | 16796 | command_plan | no | ok |
| nemotron-3-super:120b | Print the last git log message | 20848 | - | 20848 | command_plan | no | ok |
| ornith-1.5:9b | What is my external IP address? | 7854 | - | 7854 | command_plan | no | ok |
| qwen3.8:27b | What is my external IP address? | 13963 | - | 13963 | command_plan | no | ok |
| muse-glimmer:latest | What is my external IP address? | 30000 | - | 30000 | error | no | error |
| nemotron-3.5-lightning:30b | What is my external IP address? | 30000 | - | 30000 | error | no | error |
| nemotron-3-super:120b | What is my external IP address? | 21813 | - | 21813 | command_plan | no | ok |
| ornith-1.5:9b | Is neovim installed? | 12127 | - | 12127 | command_plan | no | ok |
| qwen3.8:27b | Is neovim installed? | 15855 | - | 15855 | command_plan | no | ok |
| muse-glimmer:latest | Is neovim installed? | 30000 | - | 30000 | error | no | error |
| nemotron-3.5-lightning:30b | Is neovim installed? | 30000 | - | 30000 | error | no | error |
| nemotron-3-super:120b | Is neovim installed? | 21468 | - | 21468 | command_plan | no | ok |
| ornith-1.5:9b | 230*(123-22) | 7986 | - | 7986 | command_plan | no | ok |
| qwen3.8:27b | 230*(123-22) | 12976 | 5294 | 18270 | text_response | yes | ok |
| muse-glimmer:latest | 230*(123-22) | 30000 | - | 30000 | error | no | error |
| nemotron-3.5-lightning:30b | 230*(123-22) | 14948 | 13702 | 28650 | text_response | yes | ok |
| nemotron-3-super:120b | 230*(123-22) | 16575 | - | 16575 | command_plan | no | ok |
| ornith-1.5:9b | List all markdown files recursively in my home directory | 7034 | - | 7034 | command_plan | no | ok |
| qwen3.8:27b | List all markdown files recursively in my home directory | 18826 | - | 18826 | command_plan | no | ok |
| muse-glimmer:latest | List all markdown files recursively in my home directory | 30000 | - | 30000 | error | no | error |
| nemotron-3.5-lightning:30b | List all markdown files recursively in my home directory | 30000 | - | 30000 | error | no | error |
| nemotron-3-super:120b | List all markdown files recursively in my home directory | 16824 | - | 16824 | command_plan | no | ok |
| ornith-1.5:9b | What is the purpose of life? | 2834 | 7706 | 10540 | text_response | yes | ok |
| qwen3.8:27b | What is the purpose of life? | 5907 | 5410 | 11318 | text_response | yes | ok |
| muse-glimmer:latest | What is the purpose of life? | 22200 | 29118 | 51319 | text_response | yes | ok |
| nemotron-3.5-lightning:30b | What is the purpose of life? | 15914 | 30000 | 45914 | error | yes | error |
| nemotron-3-super:120b | What is the purpose of life? | 17687 | 7843 | 25530 | text_response | yes | ok |

### Ping google five times

#### ornith-1.5:9b

- Planner ms: 10057
- Fallback ms: not used
- Total ms: 10057
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google five times
ping -c 5 google.com [recommended]
why: The user asked to ping google five times; -c 5 sets the count to 5.
```

#### qwen3.8:27b

- Planner ms: 18581
- Fallback ms: not used
- Total ms: 18581
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google.com five times
ping -c 5 google.com [recommended]
why: Standard ping with -c 5 limits to exactly five packets, matching the user's request.
```

#### muse-glimmer:latest

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3.5-lightning:30b

- Planner ms: 24084
- Fallback ms: not used
- Total ms: 24084
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google.com five times
ping -c 5 google.com [recommended]
why: The -c flag is the POSIX-standard way to limit ping iterations on Linux
```

#### nemotron-3-super:120b

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```


### Print the last git log message

#### ornith-1.5:9b

- Planner ms: 11703
- Fallback ms: not used
- Total ms: 11703
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Print the subject of the most recent git commit
git log -1 --format=%s [recommended]
why: The most common interpretation of 'last git log message' is the subject of the most recent commit; -1 limits output to one commit and %s prints only the subject line.
git log -1 --format=%B
why: If the user wants the entire commit message including the body, %B prints the raw body; otherwise it may include unwanted trailing blank lines.
```

#### qwen3.8:27b

- Planner ms: 15780
- Fallback ms: not used
- Total ms: 15780
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Print the full commit message of the most recent git commit
git log -1 --pretty=%B [recommended]
why: git log -1 limits output to the most recent commit and --pretty=%B prints the full commit message including subject and body
```

#### muse-glimmer:latest

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3.5-lightning:30b

- Planner ms: 16796
- Fallback ms: not used
- Total ms: 16796
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Print the last git commit message
git log -1 --pretty=%B [recommended]
why: Standard git command to retrieve the latest commit message body
```

#### nemotron-3-super:120b

- Planner ms: 20848
- Fallback ms: not used
- Total ms: 20848
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Show the commit message of the most recent Git commit
git log -1 --pretty=%B [recommended]
why: The user asked to print the last git log message, which is the commit message of the most recent commit. Using git log -1 limits output to one commit and --pretty=%B shows only the commit message.
```


### What is my external IP address?

#### ornith-1.5:9b

- Planner ms: 7854
- Fallback ms: not used
- Total ms: 7854
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Fetch the external IP address via a public service
curl -s https://api.ipify.org [recommended]
why: curl to ipify.org returns the public IP address as plain text.
curl -s https://ifconfig.me
why: Alternative service that returns the public IP address.
```

#### qwen3.8:27b

- Planner ms: 13963
- Fallback ms: not used
- Total ms: 13963
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Get external IP address using curl
curl ifconfig.me [recommended]
why: curl ifconfig.me is the simplest and most common way to retrieve the public IP address; it returns just the IP on stdout.
```

#### muse-glimmer:latest

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3.5-lightning:30b

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3-super:120b

- Planner ms: 21813
- Fallback ms: not used
- Total ms: 21813
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Retrieve the external (public) IP address of the machine
curl -s ifconfig.me [recommended]
why: A simple HTTP GET to a public IP echo service returns the external IP; it is safe and commonly used.
```


### Is neovim installed?

#### ornith-1.5:9b

- Planner ms: 12127
- Fallback ms: not used
- Total ms: 12127
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check whether neovim is installed
command -v nvim [recommended]
why: command -v returns the executable path and exit code 0 when nvim exists on PATH, and nothing with a non-zero exit when it is not installed.
nvim --version | head -1
why: Running nvim --version prints the version header when installed; it fails with a non-zero exit if nvim is not found.
```

#### qwen3.8:27b

- Planner ms: 15855
- Fallback ms: not used
- Total ms: 15855
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if neovim is installed
command -v nvim [recommended]
why: POSIX-portable way to verify a command exists; prints the path if found, returns non-zero if not
```

#### muse-glimmer:latest

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3.5-lightning:30b

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3-super:120b

- Planner ms: 21468
- Fallback ms: not used
- Total ms: 21468
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if neovim is installed
yay -Q neovim [recommended]
why: Using the effective package manager to query installed packages.
```


### 230*(123-22)

#### ornith-1.5:9b

- Planner ms: 7986
- Fallback ms: not used
- Total ms: 7986
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Compute the arithmetic expression 230*(123-22)
echo $((230*(123-22))) [recommended]
why: The $(( )) syntax performs integer arithmetic in bash, computing 230*(123-22) = 23230.
```

#### qwen3.8:27b

- Planner ms: 12976
- Fallback ms: 5294
- Total ms: 18270
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
230 × (123 − 22) = 230 × 101 = **23,230**
```

#### muse-glimmer:latest

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3.5-lightning:30b

- Planner ms: 14948
- Fallback ms: 13702
- Total ms: 28650
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
23230
```

#### nemotron-3-super:120b

- Planner ms: 16575
- Fallback ms: not used
- Total ms: 16575
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Compute the arithmetic expression 230*(123-22)
echo $((230*(123-22))) [recommended]
why: Bash arithmetic expansion directly computes integer arithmetic without external tools.
```


### List all markdown files recursively in my home directory

#### ornith-1.5:9b

- Planner ms: 7034
- Fallback ms: not used
- Total ms: 7034
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: List all markdown files recursively in the home directory
find ~ -type f -name '*.md' [recommended]
why: find with -type f and -name '*.md' matches regular markdown files recursively under the home directory (~).
```

#### qwen3.8:27b

- Planner ms: 18826
- Fallback ms: not used
- Total ms: 18826
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Recursively list all .md files in the home directory
find ~ -name "*.md" -type f [recommended]
why: `find` with `-name "*.md"` and `-type f` is the standard POSIX way to recursively list markdown files; it avoids following symlinks by default and only matches regular files.
```

#### muse-glimmer:latest

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3.5-lightning:30b

- Planner ms: 30000
- Fallback ms: not used
- Total ms: 30000
- Kind: error
- Unresolved: no
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3-super:120b

- Planner ms: 16824
- Fallback ms: not used
- Total ms: 16824
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Find all markdown files recursively in the home directory
find $HOME -type f -name '*.md' [recommended]
why: Uses find to traverse $HOME, matching files with .md extension; safe and non-destructive.
```


### What is the purpose of life?

#### ornith-1.5:9b

- Planner ms: 2834
- Fallback ms: 7706
- Total ms: 10540
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
The purpose of life is a philosophical question without a single agreed-upon answer. Different perspectives offer various views:

- **Religious/spiritual**: Many traditions hold that life's purpose is to connect with the divine, serve a higher power, or prepare for an afterlife.
- **Existentialist**: Thinkers like Sartre argued that life has no inherent purpose—we create our own meaning through choices and actions.
- **Scientific**: From a biological standpoint, the "purpose" is survival and reproduction to pass on genes.
- **Humanistic**: Many find meaning through relationships, contributing to others, learning, creativity, and personal growth.

Ultimately, the answer depends on your own values and beliefs. Many people find purpose through love, purposeful work, curiosity, and connection to something larger than themselves.
```

#### qwen3.8:27b

- Planner ms: 5907
- Fallback ms: 5410
- Total ms: 11318
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
There's no single agreed-upon answer. Philosophers, religions, and individuals offer different perspectives—common threads include seeking meaning through relationships, creativity, understanding, or contribution. It's ultimately a personal question without a definitive factual answer.
```

#### muse-glimmer:latest

- Planner ms: 22200
- Fallback ms: 29118
- Total ms: 51319
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
There is no single, universally agreed purpose of life. People find meaning in different ways — through relationships, creativity, growth, service, discovery, or personal values and beliefs. The purpose is generally considered something you define for yourself rather than something inherent or externally given.
```

#### nemotron-3.5-lightning:30b

- Planner ms: 15914
- Fallback ms: 30000
- Total ms: 45914
- Kind: error
- Unresolved: yes
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/chat): operation timed out
```

#### nemotron-3-super:120b

- Planner ms: 17687
- Fallback ms: 7843
- Total ms: 25530
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
The purpose of life is a philosophical question with many answers; many people find meaning in relationships, personal growth, contributing to others, or pursuing passions that give their life significance.
```


