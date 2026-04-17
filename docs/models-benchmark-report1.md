# Model Benchmark Report

## Host

- Hostname: megamind
- OS: CachyOS
- Kernel: 6.19.11-1-cachyos
- CPU: AMD RYZEN AI MAX+ PRO 395 w/ Radeon 8060S
- GPU: 00.0 Display controller: Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics] (rev d1)
- GPU VRAM: 64.0 GiB
- Memory: 62.6 GiB
- Ollama Version: 0.20.0

## Models

| Model | Parameter Size |
| --- | --- |
| lfm2:latest | 23.8B |
| qwen3.5:latest | 9.7B |
| llama3.2:latest | 3.2B |
| gemma4:latest | 8.0B |
| gemma4:26b | 25.8B |

## Summary Table

| Query | lfm2:latest | qwen3.5:latest | llama3.2:latest | gemma4:latest | gemma4:26b |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ping google five times | 2996 | 15048 | 2725 | 7049 | 12259 |
| Print the last git log message | 1016 | 7072 | 1605 | 3006 | 11146 |
| What is my external IP address? | 2814 | 15713 | 1420 | 1517 | 25703 |
| Is neovim installed? | 2308 | 15962 | 1102 | 3719 | 13024 |
| 230*(123-22) | 1924 | 14146 | failed | 1652 | failed |
| List all markdown files recursively in my home directory | 2165 | 8823 | 1691 | 3257 | 9670 |
| What is the purpose of life? | 1311 | 11818 | 2271 | 4278 | 18108 |

## Model Summary

| Model | Parameter Size | Success Rate | Avg Total ms (ok) | Successful Queries | Failed Queries |
| --- | --- | ---: | ---: | ---: | ---: |
| lfm2:latest | 23.8B | 100.0% | 2076 | 7 | 0 |
| qwen3.5:latest | 9.7B | 100.0% | 12654 | 7 | 0 |
| llama3.2:latest | 3.2B | 85.7% | 1802 | 6 | 1 |
| gemma4:latest | 8.0B | 100.0% | 3496 | 7 | 0 |
| gemma4:26b | 25.8B | 85.7% | 14985 | 6 | 1 |

## Ranking

Ranked by success rate first, then by average total milliseconds across successful queries.

| Rank | Model | Parameter Size | Success Rate | Avg Total ms (ok) |
| ---: | --- | --- | ---: | ---: |
| 1 | lfm2:latest | 23.8B | 100.0% | 2076 |
| 2 | gemma4:latest | 8.0B | 100.0% | 3496 |
| 3 | qwen3.5:latest | 9.7B | 100.0% | 12654 |
| 4 | llama3.2:latest | 3.2B | 85.7% | 1802 |
| 5 | gemma4:26b | 25.8B | 85.7% | 14985 |

## Detailed Results

| Model | Query | Planner ms | Fallback ms | Total ms | Kind | Unresolved | Status |
| --- | --- | ---: | ---: | ---: | --- | --- | --- |
| lfm2:latest | Ping google five times | 2996 | - | 2996 | command_plan | no | ok |
| qwen3.5:latest | Ping google five times | 15048 | - | 15048 | command_plan | no | ok |
| llama3.2:latest | Ping google five times | 2725 | - | 2725 | command_plan | no | ok |
| gemma4:latest | Ping google five times | 7049 | - | 7049 | command_plan | no | ok |
| gemma4:26b | Ping google five times | 12259 | - | 12259 | command_plan | no | ok |
| lfm2:latest | Print the last git log message | 1016 | - | 1016 | command_plan | no | ok |
| qwen3.5:latest | Print the last git log message | 7072 | - | 7072 | command_plan | no | ok |
| llama3.2:latest | Print the last git log message | 1605 | - | 1605 | command_plan | no | ok |
| gemma4:latest | Print the last git log message | 3006 | - | 3006 | command_plan | no | ok |
| gemma4:26b | Print the last git log message | 11146 | - | 11146 | command_plan | no | ok |
| lfm2:latest | What is my external IP address? | 2814 | - | 2814 | command_plan | no | ok |
| qwen3.5:latest | What is my external IP address? | 15713 | - | 15713 | command_plan | no | ok |
| llama3.2:latest | What is my external IP address? | 1420 | - | 1420 | command_plan | no | ok |
| gemma4:latest | What is my external IP address? | 990 | 526 | 1517 | text_response | yes | ok |
| gemma4:26b | What is my external IP address? | 25703 | - | 25703 | command_plan | no | ok |
| lfm2:latest | Is neovim installed? | 2308 | - | 2308 | command_plan | no | ok |
| qwen3.5:latest | Is neovim installed? | 15962 | - | 15962 | command_plan | no | ok |
| llama3.2:latest | Is neovim installed? | 1102 | - | 1102 | command_plan | no | ok |
| gemma4:latest | Is neovim installed? | 3719 | - | 3719 | command_plan | no | ok |
| gemma4:26b | Is neovim installed? | 13024 | - | 13024 | command_plan | no | ok |
| lfm2:latest | 230*(123-22) | 1924 | - | 1924 | command_plan | no | ok |
| qwen3.5:latest | 230*(123-22) | 14146 | - | 14146 | command_plan | no | ok |
| llama3.2:latest | 230*(123-22) | 1713 | - | 1713 | error | no | error |
| gemma4:latest | 230*(123-22) | 980 | 671 | 1652 | text_response | yes | ok |
| gemma4:26b | 230*(123-22) | 4091 | 30000 | 34091 | error | yes | error |
| lfm2:latest | List all markdown files recursively in my home directory | 2165 | - | 2165 | command_plan | no | ok |
| qwen3.5:latest | List all markdown files recursively in my home directory | 8823 | - | 8823 | command_plan | no | ok |
| llama3.2:latest | List all markdown files recursively in my home directory | 1691 | - | 1691 | command_plan | no | ok |
| gemma4:latest | List all markdown files recursively in my home directory | 3257 | - | 3257 | command_plan | no | ok |
| gemma4:26b | List all markdown files recursively in my home directory | 9670 | - | 9670 | command_plan | no | ok |
| lfm2:latest | What is the purpose of life? | 874 | 437 | 1311 | text_response | yes | ok |
| qwen3.5:latest | What is the purpose of life? | 4188 | 7630 | 11818 | text_response | yes | ok |
| llama3.2:latest | What is the purpose of life? | 533 | 1737 | 2271 | text_response | yes | ok |
| gemma4:latest | What is the purpose of life? | 793 | 3484 | 4278 | text_response | yes | ok |
| gemma4:26b | What is the purpose of life? | 4042 | 14065 | 18108 | text_response | yes | ok |

### Ping google five times

#### lfm2:latest

- Planner ms: 2996
- Fallback ms: not used
- Total ms: 2996
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google five times
ping -c 5 google.com
```

#### qwen3.5:latest

- Planner ms: 15048
- Fallback ms: not used
- Total ms: 15048
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google.com five times
ping -c 5 google.com [recommended]
why: The -c flag limits ping to 5 counts, which matches the user's request exactly
```

#### llama3.2:latest

- Planner ms: 2725
- Fallback ms: not used
- Total ms: 2725
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Run ping command on google.com five times
ping -c 5 google.com [recommended]
why: This is a standard way to test network connectivity on Linux systems
```

#### gemma4:latest

- Planner ms: 7049
- Fallback ms: not used
- Total ms: 7049
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google five times
ping -c 5 google.com [recommended]
why: The user explicitly asks to ping a host five times, which translates directly to the 'ping' command with the '-c' flag for count.
```

#### gemma4:26b

- Planner ms: 12259
- Fallback ms: not used
- Total ms: 12259
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google.com five times
ping -c 5 google.com [recommended]
why: The -c flag specifies the number of packets to send.
```


### Print the last git log message

#### lfm2:latest

- Planner ms: 1016
- Fallback ms: not used
- Total ms: 1016
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Use git log to display the last commit message
git log -1 --pretty=%B
git rev-parse --abbrev-ref HEAD
```

#### qwen3.5:latest

- Planner ms: 7072
- Fallback ms: not used
- Total ms: 7072
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Print the last git log message
git log -1 --oneline [recommended]
why: The -1 flag limits output to the last commit, and --oneline provides a compact view with just the hash and message
```

#### llama3.2:latest

- Planner ms: 1605
- Fallback ms: not used
- Total ms: 1605
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Print the last git log message
git log -1 --format=%s [recommended]
why: This command prints the most recent commit message. The \u2011 symbol is used to indicate the last commit.
```

#### gemma4:latest

- Planner ms: 3006
- Fallback ms: not used
- Total ms: 3006
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Show the last git log message
git log -1 [recommended]
why: The user wants to see the last git log message, which is best achieved using 'git log -1'.
```

#### gemma4:26b

- Planner ms: 11146
- Fallback ms: not used
- Total ms: 11146
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Print the last git commit message
git log -1 --pretty=%B [recommended]
why: The --pretty=%B flag ensures only the raw body of the commit message is printed for the last commit (-1).
```


### What is my external IP address?

#### lfm2:latest

- Planner ms: 2814
- Fallback ms: not used
- Total ms: 2814
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Determine external IP address
curl -s ifconfig.me [recommended]
why: This command retrieves the public IP without modifying any system configuration.
wget -qO- https://api.ipify.org
why: Also retrieves the public IP but uses a different service.
```

#### qwen3.5:latest

- Planner ms: 15713
- Fallback ms: not used
- Total ms: 15713
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Query external IP address using curl
curl -s https://api.ipify.org [recommended]
why: Most reliable method to get external IP address
curl -s https://ifconfig.me
why: Alternative service for external IP lookup
curl -s https://icanhazip.com
why: Another alternative service for external IP lookup
```

#### llama3.2:latest

- Planner ms: 1420
- Fallback ms: not used
- Total ms: 1420
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Get external IP address
curl -sS ifconfig.me [recommended]
why: A simple and effective way to get your external IP address.
```

#### gemma4:latest

- Planner ms: 990
- Fallback ms: 526
- Total ms: 1517
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
curl ifconfig.me
```

#### gemma4:26b

- Planner ms: 25703
- Fallback ms: not used
- Total ms: 25703
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Retrieve external IP address
curl ifconfig.me [recommended]
why: Uses curl to query a widely used and simple service that returns only the IP address.
curl icanhazip.com
why: An alternative reliable service that returns the IP address.
```


### Is neovim installed?

#### lfm2:latest

- Planner ms: 2308
- Fallback ms: not used
- Total ms: 2308
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if neovim is installed
which neovim [recommended]
why: This command checks if neovim is installed and accessible.
file neovim
why: This command verifies the file type of neovim if found.
```

#### qwen3.5:latest

- Planner ms: 15962
- Fallback ms: not used
- Total ms: 15962
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if neovim is installed on the system
which nvim [recommended]
why: The most direct way to check if neovim is installed - if nvim is in PATH, it's installed
paru -Qm neovim
why: Alternative method using paru to check if the neovim package is installed in the main repository
```

#### llama3.2:latest

- Planner ms: 1102
- Fallback ms: not used
- Total ms: 1102
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if Neovim is installed
which nvim [recommended]
```

#### gemma4:latest

- Planner ms: 3719
- Fallback ms: not used
- Total ms: 3719
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if neovim is installed
nvim --version [recommended]
why: The most direct way to check if a command-line tool like neovim is installed and available is to try running it with a version flag.
```

#### gemma4:26b

- Planner ms: 13024
- Fallback ms: not used
- Total ms: 13024
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if neovim is installed
pacman -Q neovim [recommended]
why: Using pacman -Q is the standard way to query the local database for a specific package's presence.
```


### 230*(123-22)

#### lfm2:latest

- Planner ms: 1924
- Fallback ms: not used
- Total ms: 1924
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Calculate the expression 230*(123-22)
echo $((230*(123-22))) [recommended]
why: The command performs a simple arithmetic calculation using shell arithmetic expansion.
```

#### qwen3.5:latest

- Planner ms: 14146
- Fallback ms: not used
- Total ms: 14146
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Calculate the mathematical expression 230*(123-22)
echo $((230*(123-22))) [recommended]
why: Shell arithmetic expansion is POSIX-compliant and doesn't require external tools like bc or awk
```

#### llama3.2:latest

- Planner ms: 1713
- Fallback ms: not used
- Total ms: 1713
- Kind: error
- Unresolved: no
- Status: error

```text
failed to parse planner JSON returned by Ollama: expected `,` or `}` at line 1 column 146
```

#### gemma4:latest

- Planner ms: 980
- Fallback ms: 671
- Total ms: 1652
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
230*(123-22)
```

#### gemma4:26b

- Planner ms: 4091
- Fallback ms: 30000
- Total ms: 34091
- Kind: error
- Unresolved: yes
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/generate): operation timed out
```


### List all markdown files recursively in my home directory

#### lfm2:latest

- Planner ms: 2165
- Fallback ms: not used
- Total ms: 2165
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: List all markdown files recursively in the home directory
find ~ -type f -name '*.md' [recommended]
why: The find command is appropriate for searching files by type and name without side effects.
```

#### qwen3.5:latest

- Planner ms: 8823
- Fallback ms: not used
- Total ms: 8823
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: List markdown files recursively in home directory
find ~ -type f -name "*.md" [recommended]
why: Uses find command with -type f for files, -name "*.md" for markdown extension, and ~ for home directory
```

#### llama3.2:latest

- Planner ms: 1691
- Fallback ms: not used
- Total ms: 1691
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: List markdown files recursively in home directory
find ~ -type f -name '*.md' -print0 | xargs -0 nvim [recommended]
why: Lists markdown files recursively and opens them in nvim for editing
```

#### gemma4:latest

- Planner ms: 3257
- Fallback ms: not used
- Total ms: 3257
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Recursively list markdown files in the home directory
find ~ -type f -name "*.md" [recommended]
why: The find command is the standard and most robust way to recursively search for files matching a pattern in a directory structure.
```

#### gemma4:26b

- Planner ms: 9670
- Fallback ms: not used
- Total ms: 9670
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: List all markdown files recursively in the home directory
find ~ -name "*.md" [recommended]
why: The find command with the -name pattern is the standard way to recursively search for files by extension.
```


### What is the purpose of life?

#### lfm2:latest

- Planner ms: 874
- Fallback ms: 437
- Total ms: 1311
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
What do you mean by that?
```

#### qwen3.5:latest

- Planner ms: 4188
- Fallback ms: 7630
- Total ms: 11818
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
The purpose of life is a philosophical question without a single definitive answer. Different perspectives offer various interpretations:

- **Existentialist view**: You create your own meaning through choices and actions
- **Biological view**: Survival, reproduction, and passing on genetic information
- **Religious/Spiritual views**: Vary widely across traditions (serving a deity, achieving enlightenment, etc.)
- **Humanist view**: Contributing to others, pursuing knowledge, creating beauty, and experiencing life

Ultimately, many find meaning in relationships, personal growth, creativity, and making a positive impact on the world. What matters most is what gives *you* purpose.
```

#### llama3.2:latest

- Planner ms: 533
- Fallback ms: 1737
- Total ms: 2271
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
A philosophical question that has puzzled humans for centuries.

There isn't a definitive answer, but various perspectives offer insights:

* Many believe life's purpose is to seek happiness, fulfillment, and personal growth.
* Some argue it's to find meaning through relationships, contributions to society, or spiritual pursuits.
* Others propose that life's purpose is simply to exist, experience, and enjoy the present moment.

What do you think is a fundamental aspect of human existence that gives life its purpose?
```

#### gemma4:latest

- Planner ms: 793
- Fallback ms: 3484
- Total ms: 4278
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
The purpose of life is a question that has been contemplated by philosophers, theologians, and scientists for millennia, and there is no single, universally accepted answer. It is often considered a deeply personal question whose meaning is found through individual experience, belief, and purpose-setting.
```

#### gemma4:26b

- Planner ms: 4042
- Fallback ms: 14065
- Total ms: 18108
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
The purpose of life is a subjective concept that varies depending on individual, philosophical, or biological perspectives.
```


