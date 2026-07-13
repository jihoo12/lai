use super::{LlmBackend, Message, Role};
use std::io::{self, Write};

pub struct StdinBackend;

impl LlmBackend for StdinBackend {
    fn complete(&mut self, messages: &[Message]) -> Result<String, String> {
        let last = messages.last().ok_or("no messages")?;
        if last.role == Role::User {
            eprintln!("\n--- user ---\n{}", last.content);
        } else if last.role == Role::Tool {
            eprintln!("\n--- tool result ---\n{}", last.content);
        }

        eprint!("\n--- assistant ---\n> ");
        io::stderr().flush().ok();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("read: {}", e))?;
        Ok(input.trim().to_string())
    }
}
