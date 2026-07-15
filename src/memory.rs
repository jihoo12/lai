use std::io::{self, Write};
use std::path::PathBuf;

pub struct MemoryManager {
    pub db_path: PathBuf,
    pub project_dir: PathBuf,
}

impl MemoryManager {
    pub fn new() -> Self {
        let project_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let db_path = project_dir.join("memory.db");
        Self {
            db_path,
            project_dir,
        }
    }

    /// Check if memory.db should be added to .gitignore and prompt user
    pub fn check_gitignore(&self) {
        let is_git_repo = self.project_dir.join(".git").is_dir();
        if !is_git_repo {
            return;
        }

        let gitignore_path = self.project_dir.join(".gitignore");
        let should_ignore = if gitignore_path.is_file() {
            let content = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
            content.lines().any(|l| l.trim() == "memory.db")
        } else {
            false
        };

        if should_ignore {
            return;
        }

        eprint!("\nmemory.db is not in .gitignore. Add it? [Y/n] ");
        io::stderr().flush().ok();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .ok();

        let answer = input.trim().to_lowercase();
        if answer.is_empty() || answer == "y" || answer == "yes" {
            let entry = if gitignore_path.is_file() {
                "\nmemory.db\n"
            } else {
                "memory.db\n"
            };
            if let Err(e) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&gitignore_path)
                .and_then(|mut f| f.write_all(entry.as_bytes()))
            {
                eprintln!("warning: failed to update .gitignore: {}", e);
            } else {
                eprintln!("memory: added memory.db to .gitignore");
            }
        }
    }

    /// Returns alisp code to initialize the memory database.
    pub fn init_code(&self) -> String {
        let path = self.db_path.to_string_lossy();
        let project = self.project_dir.to_string_lossy();
        format!(
            r#"
(do
  ;; Open the project memory database
  (sql-open "{path}" "default")

  ;; Create core memory tables
  (sql-execute "
    CREATE TABLE IF NOT EXISTS memories (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      category TEXT NOT NULL DEFAULT 'fact',
      key TEXT NOT NULL,
      value TEXT NOT NULL,
      context TEXT,
      importance INTEGER DEFAULT 5,
      created_at TEXT DEFAULT (datetime('now')),
      accessed_at TEXT DEFAULT (datetime('now')),
      access_count INTEGER DEFAULT 0
    )
  ")

  (sql-execute "
    CREATE TABLE IF NOT EXISTS conversations (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      role TEXT NOT NULL,
      content TEXT NOT NULL,
      topic TEXT,
      timestamp TEXT DEFAULT (datetime('now'))
    )
  ")

  (sql-execute "
    CREATE TABLE IF NOT EXISTS entities (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL UNIQUE,
      entity_type TEXT NOT NULL DEFAULT 'unknown',
      attributes TEXT,
      created_at TEXT DEFAULT (datetime('now')),
      updated_at TEXT DEFAULT (datetime('now'))
    )
  ")

  (sql-execute "
    CREATE TABLE IF NOT EXISTS knowledge (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      domain TEXT NOT NULL DEFAULT 'general',
      topic TEXT NOT NULL,
      fact TEXT NOT NULL,
      source TEXT,
      confidence REAL DEFAULT 1.0,
      created_at TEXT DEFAULT (datetime('now'))
    )
  ")

  ;; Create indexes for fast lookup
  (sql-execute "CREATE INDEX IF NOT EXISTS idx_memories_category ON memories(category)")
  (sql-execute "CREATE INDEX IF NOT EXISTS idx_memories_key ON memories(key)")
  (sql-execute "CREATE INDEX IF NOT EXISTS idx_conversations_topic ON conversations(topic)")
  (sql-execute "CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name)")
  (sql-execute "CREATE INDEX IF NOT EXISTS idx_knowledge_domain ON knowledge(domain)")

  ;; Code of conduct for self-improvement
  (sql-execute "
    CREATE TABLE IF NOT EXISTS code_of_conduct (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      version INTEGER NOT NULL DEFAULT 1,
      content TEXT NOT NULL,
      reason TEXT,
      created_at TEXT DEFAULT (datetime('now'))
    )
  ")

  ;; Store project context
  (sql-execute "INSERT OR IGNORE INTO memories (category, key, value) VALUES ('context', 'project_dir', '{project}')")

  (println "memory: project database initialized at {path}")
)
"#,
            path = path,
            project = project
        )
    }
}
