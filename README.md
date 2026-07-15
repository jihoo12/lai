# lai

AI agent powered by Lisp. No JSON schemas, no function definitions — just alisp.

## Install

```bash
cargo build --release
```

## Usage

```bash
# Interactive (paste your own LLM responses)
cargo run

# llama.cpp
cargo run -- --llama http://localhost:8080 local

# OpenAI
OPENAI_API_KEY=sk-... cargo run -- --openai https://api.openai.com/v1 gpt-4o

# OpenRouter
OPENAI_API_KEY=your-key cargo run -- --openai https://openrouter.ai/api/v1 anthropic/claude-3.5-sonnet
```

## How It Works

```
you> list files in this directory

> (exec "ls")
src/  Cargo.toml  README.md  ...

> The project contains src/, Cargo.toml, and README.md.
```

1. You send a message
2. LLM responds with text + optional `` ```alisp `` code blocks
3. Agent executes the code and feeds results back
4. Loop until final answer

## Features

- **Streaming** — Real-time token display
- **Per-project memory** — SQLite database in each project directory
- **Self-improvement** — Agent reflects on its behavior and evolves over time
- **Skills** — Extensible with `.alisp` or `.json` files
- **Security** — Pre-flight checks and confirmation prompts
- **Multiple backends** — llama.cpp, OpenAI, OpenRouter, or stdin

## Configuration

lai looks for config in this order:
1. `lai.alisp` in current directory
2. `lai.alisp` in parent directories
3. `~/.lai/config.alisp`

```lisp
(def backend-type "openai")
(def backend-url "https://openrouter.ai/api/v1")
(def backend-model "anthropic/claude-3.5-sonnet")

;; Self-improvement (agent reflects on its behavior)
(def agent-self-improve true)
```

## Skills

Place `.alisp` or `.json` files in `~/.lai/skills/` or `./skills/`:

```lisp
; name: git
; description: Git operations
; prompt: You are a git expert.

(defn git-status ()
  (exec "git status"))
```

## Memory

Each project gets its own `memory.db` SQLite database. The agent can store and query facts, knowledge, and conversation history using SQL through alisp.

## Security

Three modes: `off`, `confirm` (default), `strict`. Configurable via `lai.alisp`.

## Documentation

See [docs/docs.md](docs/docs.md) for detailed documentation.

## Dependencies

- [alisp](https://github.com/jihoo12/alisp) — Lisp interpreter
- [rusqlite](https://github.com/rusqlite/rusqlite) — SQLite
- [ureq](https://github.com/algesten/ureq) — HTTP client
