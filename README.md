# lai

AI agent that uses alisp

The model thinks, generates alisp code, executes it, and feeds results back. No JSON schemas, no function definitions — just Lisp.

## Quick Start

```bash
cargo build --release
```

### Interactive mode (stdin)

```bash
cargo run
```

### With llama.cpp

```bash
# Start llama.cpp with an OpenAI-compatible endpoint
llama-server -m model.gguf --port 8080

# Connect lai
cargo run -- --llama http://localhost:8080 local
```

### With OpenAI

```bash
export OPENAI_API_KEY=sk-...
cargo run -- --openai https://api.openai.com/v1 gpt-4o
```

### With OpenRouter

```bash
export OPENAI_API_KEY=your-openrouter-key
cargo run -- --openai https://openrouter.ai/api/v1 anthropic/claude-3.5-sonnet
```

Or in `~/.lai/config.toml`:

```toml
[backend]
type = "openai"
url = "https://openrouter.ai/api/v1"
model = "anthropic/claude-3.5-sonnet"
```

## Configuration

Create `~/.lai/config.toml`:

```toml
[backend]
type = "openai"        # "llama" or "openai"
url = "https://api.openai.com/v1"  # or https://openrouter.ai/api/v1
model = "gpt-4o"
temperature = 0.7
max_tokens = 4096

[agent]
max_turns = 20
max_context_tokens = 8192
```

Environment variables:
- `OPENAI_API_KEY` — API key for OpenAI
- `OPENAI_API_BASE` — Custom API base URL
- `OPENAI_MODEL` — Model name

## Skills

Skills extend the agent with custom functions and instructions. Place `.alisp` or `.json` files in `~/.lai/skills/` or `./skills/`.

### alisp format

```lisp
; name: git
; description: Git repository operations
; prompt: You are a git expert. Use (git-status), (git-diff), etc.

(defn git-status ()
  (exec "git status"))

(defn git-diff ()
  (exec "git diff"))
```

### JSON format

```json
{
  "name": "docker",
  "description": "Docker management",
  "prompt": "You are a Docker expert...",
  "commands": {
    "docker-ps": "exec \"docker ps -a\"",
    "docker-logs": "exec \"docker logs --tail 50\""
  }
}
```

### Built-in skills

| Skill | Description |
|-------|-------------|
| `git` | Git operations — status, log, diff, commit, branch, stash |
| `docker` | Container management — ps, images, logs, stats |
| `project` | Code analysis — tree, language stats, TODOs, dependencies |
| `research` | Web research — fetch pages, JSON, links |

## Security

lai includes a minimal security layer that checks code before execution.

### Modes

| Mode | Behavior |
|------|----------|
| `off` | No restrictions |
| `confirm` | Prompts before dangerous operations (default) |
| `strict` | Blocks dangerous operations entirely |

### Configuration

```toml
[security]
mode = "confirm"                    # "off", "confirm", or "strict"
allow_network = true                # allow HTTP requests

require_confirm_rm = true           # confirm before rm
require_confirm_sudo = true         # confirm before sudo
require_confirm_write_system = true # confirm before writing to system paths

blocked_commands = ["rm -rf /", "mkfs"]
blocked_paths = ["/etc", "/boot", "/sys", "/proc"]
```

### What it checks

- **Dangerous shell commands** — `rm -rf /`, `mkfs`, fork bombs
- **System paths** — writes to `/etc`, `/boot`, `/sys`, `/proc`
- **sudo usage** — requires confirmation or blocked in strict mode
- **Network access** — can be disabled entirely
- **File deletion** — confirms before `rm` commands

In `confirm` mode, the agent asks before proceeding:

```
⚠ security: file deletion (rm) detected in: (exec "rm -rf build/")
  allow? [y/N]
```

## How It Works

```
User → Agent → LLM → alisp code block → execute → result → LLM → ...
```

1. User sends a message
2. LLM responds, optionally with ```` ```alisp ```` blocks
3. Agent extracts and executes the code via alisp
4. Results fed back to LLM as context
5. Loop until the model produces a final answer

### Features

- **Streaming** — Tokens displayed in real-time as the model generates them
- **State persistence** — Variables defined with `(def ...)` survive across conversation turns
- **Context management** — Automatic truncation when conversation exceeds token limit
- **Skills** — Extensible with custom alisp functions and instructions
- **Security** — Pre-flight checks and confirmation prompts for dangerous operations
- **Multiple backends** — llama.cpp, OpenAI, or interactive stdin

## Example Session

```
you> what files are in the current directory?

> (exec "ls")
src/  Cargo.toml  README.md  ...

> The project contains src/, Cargo.toml, and README.md.
```

## Architecture

```
src/
  main.rs        CLI entry point, backend selection, REPL
  agent.rs       Agent loop (think → alisp → observe)
  tools.rs       alisp evaluator wrapper
  security.rs    Security policy and pre-flight checks
  skills.rs      Skill loading from directories
  config.rs      ~/.lai/config.toml parser
  llm/
    mod.rs       LlmBackend trait with streaming support
    stdin.rs     Interactive stdin backend
    llamacpp.rs  llama.cpp /v1/chat/completions
    openai.rs    OpenAI API with SSE streaming
```

## Adding a Backend

Implement the `LlmBackend` trait:

```rust
struct MyBackend;

impl LlmBackend for MyBackend {
    fn complete(&mut self, messages: &[Message]) -> Result<String, String> {
        // call your LLM here
    }

    fn complete_streaming(
        &mut self,
        messages: &[Message],
        on_token: &mut dyn FnMut(&str),
    ) -> Result<String, String> {
        // stream tokens, call on_token for each chunk
    }
}
```

## Creating a Skill

Create a `.alisp` or `.json` file in `~/.lai/skills/` or `./skills/`:

```bash
# Simple skill
cat > ~/.lai/skills/my-skill.alisp << 'EOF'
; name: my-skill
; description: My custom skill
; prompt: You have access to (my-command).

(defn my-command ()
  (exec "echo hello from my skill"))
EOF
```

Restart lai to load the new skill.

## Dependencies

- [alisp](https://github.com/jihoo12/alisp) — Lisp interpreter for AI agents
- [ureq](https://github.com/algesten/ureq) — HTTP client
- [serde](https://serde.rs/) — Serialization
- [toml](https://github.com/toml-rs/toml) — Config parsing
- [regex](https://docs.rs/regex) — Pattern matching for security checks
