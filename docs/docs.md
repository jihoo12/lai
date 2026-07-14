# lai Documentation

lai is an AI agent that uses alisp (a Lisp dialect) as its tool execution layer. The LLM generates alisp code, the agent executes it, and results are fed back as context.

## Architecture

```
User → Agent → LLM → alisp code block → execute → result → LLM → ...
```

### Core Loop

1. User sends a message
2. Message is added to conversation history
3. LLM generates a response (with streaming)
4. If response contains `` ```alisp `` blocks, they are extracted and executed
5. Execution results are added to conversation as tool output
6. Loop continues until LLM produces a final answer (no code blocks)

### Source Structure

```
src/
  main.rs        CLI entry point, backend selection, REPL loop
  agent.rs       Agent loop, context management, skill integration
  tools.rs       alisp evaluator wrapper with security policy
  security.rs    Security policy, pre-flight checks, audit logging
  skills.rs      Skill loading from directories (.alisp/.json)
  config.rs      Configuration parser (alisp-based config)
  memory.rs      Per-project SQLite memory management
  hotreload.rs   File watcher for skill hot-reloading
  llm/
    mod.rs       LlmBackend trait, shared SSE streaming parser
    stdin.rs     Interactive stdin backend (for manual testing)
    llamacpp.rs  llama.cpp /v1/chat/completions backend
    openai.rs    OpenAI API with SSE streaming backend
```

## Backends

### LlmBackend Trait

```rust
pub trait LlmBackend {
    fn complete(&mut self, messages: &[Message]) -> Result<String, String>;

    fn complete_streaming(
        &mut self,
        messages: &[Message],
        on_token: &mut dyn FnMut(&str),
    ) -> Result<String, String> {
        self.complete(messages)
    }
}
```

### Supported Backends

| Backend | Flag | Description |
|---------|------|-------------|
| llama.cpp | `--llama <url> <model>` | OpenAI-compatible endpoint |
| OpenAI | `--openai <url> <model> <api_key>` | OpenAI or compatible APIs |
| stdin | (default) | Interactive manual input |

### Adding a Backend

1. Create `src/llm/mybackend.rs`
2. Implement `LlmBackend` for your struct
3. Register in `src/llm/mod.rs`
4. Add CLI flag handling in `main.rs`

## Configuration

lai uses alisp for configuration. Config is searched in order:
1. `./lai.alisp` (current directory)
2. `../lai.alisp` (parent directories, up to `/`)
3. `~/.lai/config.alisp` (global fallback)

### Config Options

```lisp
;; Backend settings
(def backend-type "openai")          ;; "llama" or "openai"
(def backend-url "https://api.openai.com/v1")
(def backend-model "gpt-4o")
(def backend-temperature 0.7)
(def backend-max-tokens 4096)

;; Agent settings
(def agent-max-turns 20)             ;; max tool-use iterations per request
(def agent-max-context-tokens 8192)  ;; context window limit

;; Security settings
(def security-mode "Confirm")        ;; "Off", "Confirm", or "Strict"
(def security-allow-network true)
(def security-blocked-commands (quote ("rm -rf /")))
(def security-blocked-paths (quote ("/etc" "/boot")))
(def security-sandbox-paths (quote ("/home/user/projects")))
(def security-max-ops-per-turn 50)
(def security-audit-log "/tmp/lai-audit.log")
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | API key for OpenAI/OpenRouter |
| `OPENAI_API_BASE` | Custom API base URL |
| `OPENAI_MODEL` | Model name |

## Skills

Skills extend the agent with custom functions and instructions. They are loaded from:
- `~/.lai/skills/` (global)
- `./skills/` (project-local)

### alisp Format

```lisp
; name: git
; description: Git repository operations
; prompt: You are a git expert. Use (git-status), (git-diff), etc.

(defn git-status ()
  (exec "git status"))

(defn git-diff ()
  (exec "git diff"))

(defn git-commit (message)
  (exec (str "git commit -m \"" message "\"")))
```

### JSON Format

```json
{
  "name": "docker",
  "description": "Docker management",
  "prompt": "You are a Docker expert...",
  "init": "(def docker-ps-cmd \"docker ps -a\")",
  "commands": {
    "docker-ps": "exec \"docker ps -a\"",
    "docker-logs": "exec \"docker logs --tail 50\"",
    "docker-stats": "exec \"docker stats --no-stream\""
  }
}
```

### Skill Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique skill identifier |
| `description` | No | Short description |
| `prompt` | No | Instructions added to system prompt |
| `init` / `init_code` | No | alisp code executed on load |
| `commands` | No | Named commands (JSON) |

### Built-in Skills

| Skill | Description |
|-------|-------------|
| `git` | Git operations — status, log, diff, commit, branch, stash |
| `docker` | Container management — ps, images, logs, stats |
| `project` | Code analysis — tree, language stats, TODOs, dependencies |
| `research` | Web research — fetch pages, JSON, links |

### Hot Reload

Skills are watched for changes. When a `.alisp` or `.json` file is modified in a skills directory, the new skill is automatically loaded on the next conversation turn.

## Memory (SQL Database)

Each project gets its own SQLite database at `./memory.db`. This keeps memories scoped to the current project.

### How It Works

1. On startup, lai creates/opens `memory.db` in the current directory
2. If the directory is a git repo, prompts to add `memory.db` to `.gitignore`
3. Creates default tables if they don't exist
4. The agent can use SQL to store and query memories

### Database Schema

#### memories

Store facts, preferences, and key-value data.

```sql
CREATE TABLE memories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  category TEXT NOT NULL DEFAULT 'fact',  -- fact, preference, context, task, decision
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  context TEXT,
  importance INTEGER DEFAULT 5,          -- 1-10
  created_at TEXT DEFAULT (datetime('now')),
  accessed_at TEXT DEFAULT (datetime('now')),
  access_count INTEGER DEFAULT 0
)
```

#### conversations

Track conversation history.

```sql
CREATE TABLE conversations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  role TEXT NOT NULL,       -- user, assistant, system
  content TEXT NOT NULL,
  topic TEXT,
  timestamp TEXT DEFAULT (datetime('now'))
)
```

#### entities

Track people, places, things.

```sql
CREATE TABLE entities (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  entity_type TEXT NOT NULL DEFAULT 'unknown',  -- person, project, concept
  attributes TEXT,
  created_at TEXT DEFAULT (datetime('now')),
  updated_at TEXT DEFAULT (datetime('now'))
)
```

#### knowledge

Store learned knowledge.

```sql
CREATE TABLE knowledge (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  domain TEXT NOT NULL DEFAULT 'general',
  topic TEXT NOT NULL,
  fact TEXT NOT NULL,
  source TEXT,
  confidence REAL DEFAULT 1.0,
  created_at TEXT DEFAULT (datetime('now'))
)
```

### SQL Functions

| Function | Description |
|----------|-------------|
| `(sql-execute "SQL" [params...])` | Execute INSERT/UPDATE/DELETE/CREATE, returns affected rows |
| `(sql-query "SELECT..." [params...])` | Query, returns `((columns...) (row...) ...)` |
| `(sql-tables)` | List all tables |
| `(sql-schema "table")` | Get CREATE TABLE statement |

### Example Usage

```alisp
;; Store a project fact
(sql-execute "INSERT INTO memories (category, key, value) VALUES ('fact', 'framework', 'React 18')")

;; Recall it
(sql-query "SELECT value FROM memories WHERE key = 'framework'")

;; Store knowledge about architecture
(sql-execute "INSERT INTO knowledge (domain, topic, fact) VALUES ('architecture', 'frontend', 'Uses Vite for bundling')")

;; Search by importance
(sql-query "SELECT key, value FROM memories WHERE importance >= 7 ORDER BY importance DESC")

;; Track a decision
(sql-execute "INSERT INTO memories (category, key, value, importance) VALUES ('decision', 'state_management', 'Zustand over Redux', 8)")
```

## Security

lai includes a security layer that checks alisp code before execution.

### Modes

| Mode | Behavior |
|------|----------|
| `Off` | No restrictions |
| `Confirm` | Prompts before dangerous operations (default) |
| `Strict` | Blocks dangerous operations entirely |

### Security Checks

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

### Configuration

```lisp
(def security-mode "confirm")

;; Network control
(def security-allow-network true)
(def security-blocked-domains (quote ("malicious.com")))
(def security-allowed-domains (quote ("api.github.com")))

;; Dangerous operations
(def security-require-confirm-rm true)
(def security-require-confirm-sudo true)
(def security-require-confirm-write-system true)
(def security-require-confirm-eval true)

;; Blocked patterns
(def security-blocked-commands (quote ("rm -rf /" "mkfs")))
(def security-blocked-functions (quote ("exit" "setenv")))
(def security-blocked-paths (quote ("/etc" "/boot" "/sys" "/proc")))

;; Sandbox
(def security-sandbox-paths (quote ("/home/user/projects")))

;; Limits
(def security-max-ops-per-turn 50)
(def security-max-output-bytes 1048576)

;; Audit log
(def security-audit-log "/tmp/lai-audit.log")
```

### Example

```
⚠ security: file deletion (rm) detected in: (exec "rm -rf build/")
  allow? [y/N]
```

## alisp Reference

alisp is a Lisp dialect designed for AI agents. See the full reference in the [alisp repository](https://github.com/jihoo12/alisp).

### Special Forms

```lisp
(def name value)                          ; define global
(set! name value)                         ; mutate
(fn (params...) body...)                  ; lambda
(defn name (params...) body...)           ; named function
(if cond then else?)                      ; conditional
(when cond body...)                       ; if-true block
(cond (test expr)... (expr))              ; multi-branch
(do body...)                              ; sequential
(let ((name val)...) body...)             ; local bindings
(while cond body...)                      ; loop
(try body... (catch var handler...))      ; error handling
```

### Built-in Functions

**Shell:** `exec`, `shell`, `sh`, `exec-result`

**File I/O:** `read`, `write`, `append`, `exists`, `mkdir`, `cp`, `mv`, `rm`, `ls`, `glob`, `cwd`, `cd`

**Strings:** `str`, `split`, `join`, `trim`, `contains`, `replace`, `upper`, `lower`, `format`

**Lists:** `list`, `car`, `cdr`, `cons`, `len`, `push`, `nth`, `map`, `filter`, `reduce`, `each`, `range`

**Arithmetic:** `+`, `-`, `*`, `/`, `%`, `pow`, `sqrt`, `abs`, `min`, `max`

**Comparison:** `=`, `!=`, `<`, `>`, `<=`, `>=`, `not`

**HTTP:** `http-get`, `http-post`, `http-put`, `http-delete`

**JSON:** `json-parse`, `json-stringify`, `json-get`, `json-set`, `json-keys`

**SQL:** `sql-execute`, `sql-query`, `sql-tables`, `sql-schema`

**Misc:** `sleep`, `time`, `timestamp`, `exit`
