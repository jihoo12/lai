# elf

AI agent that uses alisp instead of JSON for tool calling.

The model thinks, generates alisp code, executes it, and feeds results back. No JSON schemas, no function definitions — just Lisp.

## Quick Start

```bash
cargo build --release
```

### Interactive mode (stdin)

```bash
cargo run
```

### With llama.cpp server

```bash
# Start llama.cpp with an OpenAI-compatible endpoint
llama-server -m model.gguf --port 8080

# Connect elf
cargo run -- --llama http://localhost:8080 local
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
  main.rs        CLI entry point
  agent.rs       Agent loop (think → alisp → observe)
  tools.rs       alisp evaluator wrapper
  llm/
    mod.rs       LlmBackend trait
    stdin.rs     Interactive stdin backend
    llamacpp.rs  OpenAI-compatible /v1/chat/completions
```

## Adding a Backend

Implement the `LlmBackend` trait:

```rust
struct MyBackend;

impl LlmBackend for MyBackend {
    fn complete(&mut self, messages: &[Message]) -> Result<String, String> {
        // call your LLM here
    }
}
```

## Dependencies

- [alisp](https://github.com/jihoo12/alisp) — Lisp interpreter for AI agents
- [ureq](https://github.com/algesten/ureq) — HTTP client
- [serde](https://serde.rs/) — Serialization
