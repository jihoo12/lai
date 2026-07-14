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

lai looks for config in this order:
1. `lai.toml` in the current directory
2. `lai.toml` in parent directories (walks up to `/`)
3. `~/.lai/config.toml` (global fallback)

Create a `lai.toml` in your project root:

```toml
[backend]
type = "openai"
url = "https://openrouter.ai/api/v1"
model = "anthropic/claude-3.5-sonnet"
temperature = 0.7
max_tokens = 4096

[agent]
max_turns = 20
max_context_tokens = 8192

[security]
mode = "Confirm"
allow_network = true
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

lai includes a security layer that checks code before execution.

### Modes

| Mode | Behavior |
|------|----------|
| `off` | No restrictions |
| `confirm` | Prompts before dangerous operations (default) |
| `strict` | Blocks dangerous operations entirely |

### Configuration

```toml
[security]
mode = "confirm"

# Network control
allow_network = true
blocked_domains = ["malicious.com"]
allowed_domains = ["api.github.com", "httpbin.org"]

# Dangerous operations
require_confirm_rm = true
require_confirm_sudo = true
require_confirm_write_system = true
require_confirm_eval = true

# Blocked commands (always blocked in strict mode)
blocked_commands = ["rm -rf /", "mkfs", ":(){ :|:& };:"]

# Blocked alisp functions (always blocked in strict mode)
blocked_functions = ["exit", "setenv"]

# System paths (blocked from writes)
blocked_paths = ["/etc", "/boot", "/sys", "/proc"]

# Sandbox: restrict file writes to these paths
sandbox_paths = ["/home/user/projects"]

# Rate limiting
max_ops_per_turn = 50

# Output size limit
max_output_bytes = 1048576  # 1MB

# Audit logging
audit_log = "/tmp/lai-audit.log"
```

### What it checks

| Check | Confirm | Strict |
|-------|---------|--------|
| `rm` commands | prompt | prompt |
| `sudo` | prompt | blocked |
| `eval` | prompt | blocked |
| System path writes | prompt | prompt |
| Blocked commands | blocked | blocked |
| Blocked functions | prompt | blocked |
| Sandbox violations | prompt | prompt |
| Domain blocklist | blocked | blocked |
| Domain allowlist | blocked | blocked |
| Rate limit (ops/turn) | blocked | blocked |
| Output size limit | truncated | truncated |

### Example

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
