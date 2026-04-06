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
| Ping google five times | 4878 | 18357 | 3080 | 7840 | 13629 |
| Print the last git log message | 1162 | 12116 | 1650 | 3160 | 11485 |
| What is my external IP address? | 3135 | 19476 | 1450 | 1628 | 26356 |
| Is neovim installed? | 2518 | 25991 | 1112 | 3999 | 13374 |
| 230*(123-22) | 2078 | 15378 | failed | 1721 | failed |
| List all markdown files recursively in my home directory | 2310 | 11209 | 1752 | 3518 | 10062 |
| What is the purpose of life? | 1546 | 12709 | 2375 | 4577 | 18867 |

## Model Summary

| Model | Parameter Size | Success Rate | Avg Total ms (ok) | Successful Queries | Failed Queries |
| --- | --- | ---: | ---: | ---: | ---: |
| lfm2:latest | 23.8B | 100.0% | 2518 | 7 | 0 |
| qwen3.5:latest | 9.7B | 100.0% | 16462 | 7 | 0 |
| llama3.2:latest | 3.2B | 85.7% | 1903 | 6 | 1 |
| gemma4:latest | 8.0B | 100.0% | 3777 | 7 | 0 |
| gemma4:26b | 25.8B | 85.7% | 15628 | 6 | 1 |

## Ranking

Ranked by success rate first, then by average total milliseconds across successful queries.

| Rank | Model | Parameter Size | Success Rate | Avg Total ms (ok) |
| ---: | --- | --- | ---: | ---: |
| 1 | lfm2:latest | 23.8B | 100.0% | 2518 |
| 2 | gemma4:latest | 8.0B | 100.0% | 3777 |
| 3 | qwen3.5:latest | 9.7B | 100.0% | 16462 |
| 4 | llama3.2:latest | 3.2B | 85.7% | 1903 |
| 5 | gemma4:26b | 25.8B | 85.7% | 15628 |

| Model | Query | Planner ms | Fallback ms | Total ms | Kind | Unresolved | Status |
| --- | --- | ---: | ---: | ---: | --- | --- | --- |
| lfm2:latest | Ping google five times | 4878 | - | 4878 | command_plan | no | ok |
| lfm2:latest | Print the last git log message | 1162 | - | 1162 | command_plan | no | ok |
| lfm2:latest | What is my external IP address? | 3135 | - | 3135 | command_plan | no | ok |
| lfm2:latest | Is neovim installed? | 2518 | - | 2518 | command_plan | no | ok |
| lfm2:latest | 230*(123-22) | 2078 | - | 2078 | command_plan | no | ok |
| lfm2:latest | List all markdown files recursively in my home directory | 2310 | - | 2310 | command_plan | no | ok |
| lfm2:latest | What is the purpose of life? | 980 | 566 | 1546 | text_response | yes | ok |
| qwen3.5:latest | Ping google five times | 18357 | - | 18357 | command_plan | no | ok |
| qwen3.5:latest | Print the last git log message | 12116 | - | 12116 | command_plan | no | ok |
| qwen3.5:latest | What is my external IP address? | 19476 | - | 19476 | command_plan | no | ok |
| qwen3.5:latest | Is neovim installed? | 25991 | - | 25991 | command_plan | no | ok |
| qwen3.5:latest | 230*(123-22) | 15378 | - | 15378 | command_plan | no | ok |
| qwen3.5:latest | List all markdown files recursively in my home directory | 11209 | - | 11209 | command_plan | no | ok |
| qwen3.5:latest | What is the purpose of life? | 4495 | 8213 | 12709 | text_response | yes | ok |
| llama3.2:latest | Ping google five times | 3080 | - | 3080 | command_plan | no | ok |
| llama3.2:latest | Print the last git log message | 1650 | - | 1650 | command_plan | no | ok |
| llama3.2:latest | What is my external IP address? | 1450 | - | 1450 | command_plan | no | ok |
| llama3.2:latest | Is neovim installed? | 1112 | - | 1112 | command_plan | no | ok |
| llama3.2:latest | 230*(123-22) | 1791 | - | 1791 | error | no | error |
| llama3.2:latest | List all markdown files recursively in my home directory | 1752 | - | 1752 | command_plan | no | ok |
| llama3.2:latest | What is the purpose of life? | 539 | 1835 | 2375 | text_response | yes | ok |
| gemma4:latest | Ping google five times | 7840 | - | 7840 | command_plan | no | ok |
| gemma4:latest | Print the last git log message | 3160 | - | 3160 | command_plan | no | ok |
| gemma4:latest | What is my external IP address? | 1110 | 518 | 1628 | text_response | yes | ok |
| gemma4:latest | Is neovim installed? | 3999 | - | 3999 | command_plan | no | ok |
| gemma4:latest | 230*(123-22) | 1006 | 714 | 1721 | text_response | yes | ok |
| gemma4:latest | List all markdown files recursively in my home directory | 3518 | - | 3518 | command_plan | no | ok |
| gemma4:latest | What is the purpose of life? | 821 | 3755 | 4577 | text_response | yes | ok |
| gemma4:26b | Ping google five times | 13629 | - | 13629 | command_plan | no | ok |
| gemma4:26b | Print the last git log message | 11485 | - | 11485 | command_plan | no | ok |
| gemma4:26b | What is my external IP address? | 26356 | - | 26356 | command_plan | no | ok |
| gemma4:26b | Is neovim installed? | 13374 | - | 13374 | command_plan | no | ok |
| gemma4:26b | 230*(123-22) | 4206 | 30000 | 34206 | error | yes | error |
| gemma4:26b | List all markdown files recursively in my home directory | 10062 | - | 10062 | command_plan | no | ok |
| gemma4:26b | What is the purpose of life? | 4200 | 14666 | 18867 | text_response | yes | ok |

## lfm2:latest - Ping google five times

- Planner ms: 4878
- Fallback ms: not used
- Total ms: 4878
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google five times
ping -c 5 google.com
```

## lfm2:latest - Print the last git log message

- Planner ms: 1162
- Fallback ms: not used
- Total ms: 1162
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Use git log to display the last commit message
git log -1 --pretty=%B
git rev-parse --abbrev-ref HEAD
```

## lfm2:latest - What is my external IP address?

- Planner ms: 3135
- Fallback ms: not used
- Total ms: 3135
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

## lfm2:latest - Is neovim installed?

- Planner ms: 2518
- Fallback ms: not used
- Total ms: 2518
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

## lfm2:latest - 230*(123-22)

- Planner ms: 2078
- Fallback ms: not used
- Total ms: 2078
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Calculate the expression 230*(123-22)
echo $((230*(123-22))) [recommended]
why: The command performs a simple arithmetic calculation using shell arithmetic expansion.
```

## lfm2:latest - List all markdown files recursively in my home directory

- Planner ms: 2310
- Fallback ms: not used
- Total ms: 2310
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: List all markdown files recursively in the home directory
find ~ -type f -name '*.md' [recommended]
why: The find command is appropriate for searching files by type and name without side effects.
```

## lfm2:latest - What is the purpose of life?

- Planner ms: 980
- Fallback ms: 566
- Total ms: 1546
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
What do you mean by that?
```

## qwen3.5:latest - Ping google five times

- Planner ms: 18357
- Fallback ms: not used
- Total ms: 18357
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google.com five times
ping -c 5 google.com [recommended]
why: The -c flag limits ping to 5 counts, which matches the user's request exactly
```

## qwen3.5:latest - Print the last git log message

- Planner ms: 12116
- Fallback ms: not used
- Total ms: 12116
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Print the last git log message
git log -1 --oneline [recommended]
why: This command safely displays the most recent commit message without modifying anything. The --oneline flag provides a concise output format.
```

## qwen3.5:latest - What is my external IP address?

- Planner ms: 19476
- Fallback ms: not used
- Total ms: 19476
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Query external IP address using curl
curl -s ifconfig.me [recommended]
why: Most reliable and simple method to get external IP address
curl -s api.ipify.org
why: Alternative service for external IP lookup
curl -s icanhazip.com
why: Another alternative service for external IP lookup
```

## qwen3.5:latest - Is neovim installed?

- Planner ms: 25991
- Fallback ms: not used
- Total ms: 25991
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if neovim is installed on the system
pacman -Q neovim [recommended]
why: Standard Arch Linux way to check if a package is installed. Will output package info if installed, nothing if not.
which nvim
why: Alternative method to verify neovim executable exists in PATH, useful if package is installed but binary not found.
```

## qwen3.5:latest - 230*(123-22)

- Planner ms: 15378
- Fallback ms: not used
- Total ms: 15378
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Calculate the mathematical expression 230*(123-22)
echo $((230*(123-22))) [recommended]
why: Shell arithmetic expansion is POSIX-compliant and doesn't require external tools like bc or awk
```

## qwen3.5:latest - List all markdown files recursively in my home directory

- Planner ms: 11209
- Fallback ms: not used
- Total ms: 11209
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: List all markdown files recursively in home directory
find ~ -type f -name "*.md" [recommended]
why: Uses find command with -type f for files, -name "*.md" for markdown extension, and ~ for home directory
```

## qwen3.5:latest - What is the purpose of life?

- Planner ms: 4495
- Fallback ms: 8213
- Total ms: 12709
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

## llama3.2:latest - Ping google five times

- Planner ms: 3080
- Fallback ms: not used
- Total ms: 3080
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Run ping command on google.com five times
ping -c 5 google.com [recommended]
why: This is a standard way to test network connectivity on Linux systems
```

## llama3.2:latest - Print the last git log message

- Planner ms: 1650
- Fallback ms: not used
- Total ms: 1650
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Print the last git log message
git log -1 --format=%s [recommended]
why: This command prints the most recent commit message. The \u2011 symbol is used to indicate the last commit.
```

## llama3.2:latest - What is my external IP address?

- Planner ms: 1450
- Fallback ms: not used
- Total ms: 1450
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Get external IP address
curl -sS ifconfig.me [recommended]
why: A simple and effective way to get your external IP address.
```

## llama3.2:latest - Is neovim installed?

- Planner ms: 1112
- Fallback ms: not used
- Total ms: 1112
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if Neovim is installed
which nvim [recommended]
```

## llama3.2:latest - 230*(123-22)

- Planner ms: 1791
- Fallback ms: not used
- Total ms: 1791
- Kind: error
- Unresolved: no
- Status: error

```text
failed to parse planner JSON returned by Ollama: expected `,` or `}` at line 1 column 146
```

## llama3.2:latest - List all markdown files recursively in my home directory

- Planner ms: 1752
- Fallback ms: not used
- Total ms: 1752
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: List markdown files recursively in home directory
find ~ -type f -name '*.md' -print0 | xargs -0 nvim [recommended]
why: Lists markdown files recursively and opens them in nvim for editing
```

## llama3.2:latest - What is the purpose of life?

- Planner ms: 539
- Fallback ms: 1835
- Total ms: 2375
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

## gemma4:latest - Ping google five times

- Planner ms: 7840
- Fallback ms: not used
- Total ms: 7840
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google five times
ping -c 5 google.com [recommended]
why: The user explicitly asks to ping a host five times, which translates directly to the 'ping' command with the '-c' flag for count.
```

## gemma4:latest - Print the last git log message

- Planner ms: 3160
- Fallback ms: not used
- Total ms: 3160
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Show the last git log message
git log -1 [recommended]
why: The user wants to see the last git log message, which is best achieved using 'git log -1'.
```

## gemma4:latest - What is my external IP address?

- Planner ms: 1110
- Fallback ms: 518
- Total ms: 1628
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
curl ifconfig.me
```

## gemma4:latest - Is neovim installed?

- Planner ms: 3999
- Fallback ms: not used
- Total ms: 3999
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if neovim is installed
nvim --version [recommended]
why: The most direct way to check if a command-line tool like neovim is installed and available is to try running it with a version flag.
```

## gemma4:latest - 230*(123-22)

- Planner ms: 1006
- Fallback ms: 714
- Total ms: 1721
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
230*(123-22)
```

## gemma4:latest - List all markdown files recursively in my home directory

- Planner ms: 3518
- Fallback ms: not used
- Total ms: 3518
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Recursively list markdown files in the home directory
find ~ -type f -name "*.md" [recommended]
why: The find command is the standard and most robust way to recursively search for files matching a pattern in a directory structure.
```

## gemma4:latest - What is the purpose of life?

- Planner ms: 821
- Fallback ms: 3755
- Total ms: 4577
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
The purpose of life is a question that has been contemplated by philosophers, theologians, and scientists for millennia, and there is no single, universally accepted answer. It is often considered a deeply personal question whose meaning is found through individual experience, belief, and purpose-setting.
```

## gemma4:26b - Ping google five times

- Planner ms: 13629
- Fallback ms: not used
- Total ms: 13629
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Ping google.com five times
ping -c 5 google.com [recommended]
why: The -c flag specifies the number of packets to send.
```

## gemma4:26b - Print the last git log message

- Planner ms: 11485
- Fallback ms: not used
- Total ms: 11485
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Print the last git commit message
git log -1 --pretty=%B [recommended]
why: The --pretty=%B flag ensures only the raw body of the commit message is printed for the last commit (-1).
```

## gemma4:26b - What is my external IP address?

- Planner ms: 26356
- Fallback ms: not used
- Total ms: 26356
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

## gemma4:26b - Is neovim installed?

- Planner ms: 13374
- Fallback ms: not used
- Total ms: 13374
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: Check if neovim is installed
pacman -Q neovim [recommended]
why: Using pacman -Q is the standard way to query the local database for a specific package's presence.
```

## gemma4:26b - 230*(123-22)

- Planner ms: 4206
- Fallback ms: 30000
- Total ms: 34206
- Kind: error
- Unresolved: yes
- Status: error

```text
failed to call Ollama: error sending request for url (http://127.0.0.1:11434/api/generate): operation timed out
```

## gemma4:26b - List all markdown files recursively in my home directory

- Planner ms: 10062
- Fallback ms: not used
- Total ms: 10062
- Kind: command_plan
- Unresolved: no
- Status: ok

```text
summary: List all markdown files recursively in the home directory
find ~ -name "*.md" [recommended]
why: The find command with the -name pattern is the standard way to recursively search for files by extension.
```

## gemma4:26b - What is the purpose of life?

- Planner ms: 4200
- Fallback ms: 14666
- Total ms: 18867
- Kind: text_response
- Unresolved: yes
- Status: ok

```text
The purpose of life is a subjective concept that varies depending on individual, philosophical, or biological perspectives.
```

