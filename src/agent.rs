use crate::llm::{LlmBackend, Message, Role};
use crate::tools::AlispHost;

const SYSTEM_PROMPT: &str = r#"You are an AI agent. You can use alisp to interact with the system.

When you need to run code, output an alisp code block:

```alisp
(exec "ls -la")
```

You can chain multiple expressions. The last expression's value is returned as the result.

Available functions:
- Shell: (exec "cmd"), (exec-result "cmd")
- Files: (read "path"), (write "path" "content"), (ls "path"), (glob "pattern"), (exists "path"), (mkdir "path"), (rm "path"), (cp "src" "dst"), (mv "src" "dst")
- Strings: (str a b...), (split "s" "delim"), (join list "delim"), (trim "s"), (contains "s" "sub"), (format "{}" val)
- HTTP: (http-get "url"), (http-post "url" "body")
- JSON: (json-parse "str"), (json-stringify expr), (json-get obj key)
- Lists: (list a b...), (car list), (cdr list), (len list), (map fn list), (filter fn list)
- IO: (print a...), (println a...)
- Misc: (sleep N), (cwd), (cd "path"), (getenv "NAME")

When you have completed the task, respond with your final answer directly (no code block needed).
Always explain what you are doing before and after running code."#;

pub struct Agent {
    messages: Vec<Message>,
}

impl Agent {
    pub fn new() -> Self {
        Self {
            messages: vec![Message {
                role: Role::System,
                content: SYSTEM_PROMPT.to_string(),
            }],
        }
    }

    pub fn run(&mut self, backend: &mut dyn LlmBackend, user_input: &str) -> Result<String, String> {
        self.messages.push(Message {
            role: Role::User,
            content: user_input.to_string(),
        });

        let mut tools = AlispHost::new();
        let max_turns = 20;

        for _ in 0..max_turns {
            let response = backend.complete(&self.messages)?;

            if response.trim().is_empty() {
                return Ok(String::new());
            }

            let blocks = extract_alisp_blocks(&response);

            if blocks.is_empty() {
                self.messages.push(Message {
                    role: Role::Assistant,
                    content: response.clone(),
                });
                return Ok(response);
            }

            self.messages.push(Message {
                role: Role::Assistant,
                content: response.clone(),
            });

            let mut tool_output = String::new();
            for code in &blocks {
                let result = tools.execute(code);
                let output = match result {
                    Ok(val) => val,
                    Err(e) => format!("error: {}", e),
                };
                tool_output.push_str(&format!("```\n{}\n```\n", output));
            }

            self.messages.push(Message {
                role: Role::Tool,
                content: tool_output,
            });
        }

        Err("max turns exceeded".to_string())
    }
}

fn extract_alisp_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("```alisp") {
        let after_tag = start + 8;
        if let Some(end) = remaining[after_tag..].find("```") {
            let code = remaining[after_tag..after_tag + end].trim().to_string();
            if !code.is_empty() {
                blocks.push(code);
            }
            remaining = &remaining[after_tag + end + 3..];
        } else {
            break;
        }
    }

    blocks
}
