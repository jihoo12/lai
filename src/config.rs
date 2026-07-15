use crate::security::{SecurityConfig, SecurityMode};
use alisp::Evaluator;
use std::path::PathBuf;

pub struct Config {
    pub backend: BackendConfig,
    pub agent: AgentConfig,
    pub security: SecurityConfig,
}

pub struct BackendConfig {
    pub r#type: String,
    pub url: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

pub struct AgentConfig {
    pub max_turns: u32,
    pub max_context_tokens: usize,
    pub self_improve: bool,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            r#type: "llama".to_string(),
            url: "http://localhost:8080/v1".to_string(),
            model: "local".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_turns: 20,
            max_context_tokens: 8192,
            self_improve: false,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = find_config();
        match std::fs::read_to_string(&config_path) {
            Ok(content) => parse_alisp_config(&content).unwrap_or_else(|e| {
                eprintln!("warning: failed to parse {}: {}", config_path.display(), e);
                Config::default()
            }),
            Err(_) => Config::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: BackendConfig::default(),
            agent: AgentConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

fn find_config() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let mut dir = Some(cwd.as_path());
    while let Some(current) = dir {
        let candidate = current.join("lai.alisp");
        if candidate.is_file() {
            return candidate;
        }
        dir = current.parent();
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".lai").join("config.alisp")
}

fn parse_alisp_config(content: &str) -> Result<Config, String> {
    let mut eval = Evaluator::new();
    eval.eval_str(content)?;

    let backend = BackendConfig {
        r#type: get_str(&eval, "backend-type", "llama"),
        url: get_str(&eval, "backend-url", "http://localhost:8080/v1"),
        model: get_str(&eval, "backend-model", "local"),
        temperature: get_f64(&eval, "backend-temperature", 0.7),
        max_tokens: get_u32(&eval, "backend-max-tokens", 4096),
    };

    let agent = AgentConfig {
        max_turns: get_u32(&eval, "agent-max-turns", 20),
        max_context_tokens: get_usize(&eval, "agent-max-context-tokens", 8192),
        self_improve: get_bool(&eval, "agent-self-improve", false),
    };

    let security = SecurityConfig {
        mode: SecurityMode::from(get_str(&eval, "security-mode", "Confirm").as_str()),
        allow_network: get_bool(&eval, "security-allow-network", true),
        require_confirm_rm: get_bool(&eval, "security-require-confirm-rm", true),
        require_confirm_sudo: get_bool(&eval, "security-require-confirm-sudo", true),
        require_confirm_write_system: get_bool(&eval, "security-require-confirm-write-system", true),
        require_confirm_eval: get_bool(&eval, "security-require-confirm-eval", true),
        blocked_commands: get_str_list(&eval, "security-blocked-commands"),
        blocked_functions: get_str_list(&eval, "security-blocked-functions"),
        blocked_paths: get_str_list(&eval, "security-blocked-paths"),
        blocked_domains: get_str_list(&eval, "security-blocked-domains"),
        allowed_domains: get_str_list(&eval, "security-allowed-domains"),
        sandbox_paths: get_str_list(&eval, "security-sandbox-paths"),
        max_ops_per_turn: get_usize(&eval, "security-max-ops-per-turn", 50),
        max_output_bytes: get_usize(&eval, "security-max-output-bytes", 1048576),
        exec_timeout_secs: get_u64(&eval, "security-exec-timeout-secs", 60),
        audit_log: get_optional_str(&eval, "security-audit-log"),
    };

    Ok(Config {
        backend,
        agent,
        security,
    })
}

fn get_str(eval: &Evaluator, name: &str, default: &str) -> String {
    match eval.get(name) {
        Some(alisp::Expr::Str(s)) => s,
        _ => default.to_string(),
    }
}

fn get_optional_str(eval: &Evaluator, name: &str) -> Option<String> {
    match eval.get(name) {
        Some(alisp::Expr::Str(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

fn get_f64(eval: &Evaluator, name: &str, default: f64) -> f64 {
    match eval.get(name) {
        Some(alisp::Expr::Num(n)) => n,
        _ => default,
    }
}

fn get_u32(eval: &Evaluator, name: &str, default: u32) -> u32 {
    match eval.get(name) {
        Some(alisp::Expr::Num(n)) => n as u32,
        _ => default,
    }
}

fn get_usize(eval: &Evaluator, name: &str, default: usize) -> usize {
    match eval.get(name) {
        Some(alisp::Expr::Num(n)) => n as usize,
        _ => default,
    }
}

fn get_u64(eval: &Evaluator, name: &str, default: u64) -> u64 {
    match eval.get(name) {
        Some(alisp::Expr::Num(n)) => n as u64,
        _ => default,
    }
}

fn get_bool(eval: &Evaluator, name: &str, default: bool) -> bool {
    match eval.get(name) {
        Some(alisp::Expr::Bool(b)) => b,
        _ => default,
    }
}

fn get_str_list(eval: &Evaluator, name: &str) -> Vec<String> {
    match eval.get(name) {
        Some(alisp::Expr::List(v)) => v
            .into_iter()
            .filter_map(|e| match e {
                alisp::Expr::Str(s) => Some(s),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}
