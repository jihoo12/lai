use crate::security::SecurityConfig;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub backend: BackendConfig,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub security: SecurityConfig,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    #[serde(default = "default_backend_type")]
    pub r#type: String,

    #[serde(default = "default_llama_url")]
    pub url: String,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_temperature")]
    pub temperature: f64,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,

    #[serde(default = "default_max_context_tokens")]
    pub max_context_tokens: usize,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            r#type: default_backend_type(),
            url: default_llama_url(),
            model: default_model(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_turns: default_max_turns(),
            max_context_tokens: default_max_context_tokens(),
        }
    }
}

fn default_backend_type() -> String {
    "llama".to_string()
}
fn default_llama_url() -> String {
    "http://localhost:8080/v1".to_string()
}
fn default_model() -> String {
    "local".to_string()
}
fn default_temperature() -> f64 {
    0.7
}
fn default_max_tokens() -> u32 {
    4096
}
fn default_max_turns() -> u32 {
    20
}
fn default_max_context_tokens() -> usize {
    8192
}

impl Config {
    pub fn load() -> Self {
        let config_path = config_path();
        match std::fs::read_to_string(&config_path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("warning: failed to parse {}: {}", config_path.display(), e);
                Config::default()
            }),
            Err(_) => Config::default(),
        }
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".lai").join("config.toml")
}
