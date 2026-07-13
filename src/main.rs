mod llm;
mod tools;
mod agent;

use agent::Agent;
use llm::LlmBackend;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut backend: Box<dyn LlmBackend> = match args.get(1).map(|s| s.as_str()) {
        Some("--llama") => {
            let url = args.get(2).cloned().unwrap_or_else(|| "http://localhost:8080".to_string());
            let model = args.get(3).cloned().unwrap_or_else(|| "local".to_string());
            eprintln!("using llama.cpp at {} (model: {})", url, model);
            Box::new(llm::llamacpp::LlamaCppBackend::new(&url, &model))
        }
        _ => {
            eprintln!("elf agent (type your message, Ctrl+D to exit)");
            eprintln!("usage: elf [--llama <url> <model>]");
            Box::new(llm::stdin::StdinBackend)
        }
    };

    let mut agent = Agent::new();

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

        match agent.run(&mut *backend, input) {
            Ok(response) => println!("{}", response),
            Err(e) => eprintln!("agent error: {}", e),
        }
    }
}
