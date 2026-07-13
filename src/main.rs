mod agent;
mod config;
mod llm;
mod security;
mod skills;
mod tools;

use agent::Agent;
use config::Config;
use llm::LlmBackend;
use skills::Skill;
use std::io::{self, Write};
use std::path::PathBuf;

fn skill_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".lai").join("skills"));
    }

    dirs.push(PathBuf::from("skills"));

    dirs
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = Config::load();

    let mut backend: Box<dyn LlmBackend> = match args.get(1).map(|s| s.as_str()) {
        Some("--llama") => {
            let url = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| config.backend.url.clone());
            let model = args
                .get(3)
                .cloned()
                .unwrap_or_else(|| config.backend.model.clone());
            eprintln!("using llama.cpp at {} (model: {})", url, model);
            Box::new(llm::llamacpp::LlamaCppBackend::with_params(
                &url,
                &model,
                config.backend.temperature,
                config.backend.max_tokens,
            ))
        }
        Some("--openai") => {
            let url = args.get(2).cloned().unwrap_or_else(|| {
                std::env::var("OPENAI_API_BASE")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".to_string())
            });
            let model = args.get(3).cloned().unwrap_or_else(|| {
                std::env::var("OPENAI_MODEL")
                    .unwrap_or_else(|_| "gpt-4o".to_string())
            });
            let api_key = args.get(4).cloned().unwrap_or_else(|| {
                std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
                    eprintln!("error: OPENAI_API_KEY not set");
                    std::process::exit(1);
                })
            });
            eprintln!("using OpenAI at {} (model: {})", url, model);
            Box::new(llm::openai::OpenAIBackend::with_params(
                &url,
                &model,
                &api_key,
                config.backend.temperature,
                config.backend.max_tokens,
            ))
        }
        _ => {
            let url = config.backend.url.clone();
            let model = config.backend.model.clone();
            match config.backend.r#type.as_str() {
                "openai" => {
                    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
                        eprintln!("error: OPENAI_API_KEY not set");
                        std::process::exit(1);
                    });
                    eprintln!("using OpenAI at {} (model: {})", url, model);
                    Box::new(llm::openai::OpenAIBackend::with_params(
                        &url,
                        &model,
                        &api_key,
                        config.backend.temperature,
                        config.backend.max_tokens,
                    ))
                }
                _ => {
                    eprintln!("lai agent (type your message, Ctrl+D to exit)");
                    eprintln!("usage: lai [--llama <url> <model> | --openai <url> <model> <api_key>]");
                    eprintln!("or configure ~/.lai/config.toml");
                    Box::new(llm::stdin::StdinBackend)
                }
            }
        }
    };

    let skill_dirs = skill_dirs();
    let skills = Skill::load_dirs(&skill_dirs);
    if !skills.is_empty() {
        eprintln!("loaded {} skill(s): {}", skills.len(), skills.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "));
    }

    let mut agent = Agent::new(config.agent, config.security, &skills);

    loop {
        eprint!("\nyou> ");
        io::stderr().flush().ok();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("error: {}", e);
                break;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match agent.run_streaming(&mut *backend, input, &mut |token| {
            print!("{}", token);
            io::stdout().flush().ok();
        }) {
            Ok(_) => println!(),
            Err(e) => eprintln!("\nagent error: {}", e),
        }
    }
}
