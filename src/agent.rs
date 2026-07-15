use crate::config::AgentConfig;
use crate::llm::{LlmBackend, Message, Role};
use crate::memory::MemoryManager;
use crate::security::{SecurityConfig, SecurityPolicy};
use crate::skills::Skill;
use crate::tools::AlispHost;
use std::collections::HashSet;

const SYSTEM_PROMPT: &str = include_str!("../prompt.md");

const SELF_IMPROVE_PROMPT: &str = r#"You are reviewing your own conversation history to improve your behavior.

## Your Task

Analyze the recent conversation and write/update your Code of Conduct. This is a set of guidelines that will be injected into your system prompt for future conversations.

## Rules for Code of Conduct

1. Keep it concise (under 500 words)
2. Focus on behavioral patterns, not technical details
3. Include lessons learned from mistakes
4. Include what worked well
5. Be specific and actionable

## Output Format

Write ONLY the code of conduct text. Do not include any preamble or explanation.

Example:
```
## Code of Conduct

### Communication
- Be concise and direct
- Ask clarifying questions when requirements are ambiguous

### Code Quality
- Always check existing code before adding new features
- Run tests after making changes

### Problem Solving
- Break complex tasks into smaller steps
- Verify assumptions before implementing
```"#;

/// Rough token estimation: ~4 chars per token for English text.
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

pub struct Agent {
    messages: Vec<Message>,
    tools: AlispHost,
    policy: SecurityPolicy,
    max_turns: u32,
    max_context_tokens: usize,
    loaded_skills: HashSet<String>,
    self_improve: bool,
    user_turn_count: usize,
}

impl Agent {
    pub fn new(
        config: AgentConfig,
        security: SecurityConfig,
        skills: &[Skill],
        memory: &MemoryManager,
    ) -> Self {
        let policy = SecurityPolicy::new(security.clone());
        let mut tools = AlispHost::with_policy(policy.clone());

        let mut system_prompt = SYSTEM_PROMPT.to_string();
        let mut loaded_skills = HashSet::new();

        // Initialize memory database
        let mem_init = memory.init_code();
        if let Err(e) = tools.execute(&mem_init) {
            eprintln!("warning: memory init failed: {}", e);
        }

        for skill in skills {
            loaded_skills.insert(skill.name.clone());
            if !skill.prompt.is_empty() {
                system_prompt.push_str(&format!("\n\n{}", skill.prompt));
            }
            if !skill.init_code.is_empty() {
                if let Err(e) = tools.execute(&skill.init_code) {
                    eprintln!("warning: skill '{}' init failed: {}", skill.name, e);
                }
            }
        }

        system_prompt.push_str(&Skill::skill_index(skills));

        // Load existing code of conduct from memory
        if config.self_improve {
            if let Ok(result) = tools.execute(
                r#"(sql-query "SELECT content FROM code_of_conduct ORDER BY version DESC LIMIT 1")"#
            ) {
                if let Some(content) = extract_first_cell(&result) {
                    if !content.is_empty() && content != "nil" {
                        system_prompt.push_str(&format!("\n\n{}", content));
                        eprintln!("self-improve: loaded existing code of conduct");
                    }
                }
            }
        }

        Self {
            messages: vec![Message {
                role: Role::System,
                content: system_prompt,
            }],
            tools,
            policy,
            max_turns: config.max_turns,
            max_context_tokens: config.max_context_tokens,
            loaded_skills,
            self_improve: config.self_improve,
            user_turn_count: 0,
        }
    }

    /// Refresh skills: initialize any new skills and update the system prompt.
    pub fn refresh_skills(&mut self, skills: &[Skill]) {
        let mut new_count = 0;
        let mut new_skill_text = String::new();

        for skill in skills {
            if self.loaded_skills.contains(&skill.name) {
                continue;
            }
            self.loaded_skills.insert(skill.name.clone());
            new_count += 1;

            eprintln!("hotreload: loaded skill '{}'", skill.name);

            if !skill.prompt.is_empty() {
                new_skill_text.push_str(&format!("\n\n{}", skill.prompt));
            }
            if !skill.init_code.is_empty() {
                if let Err(e) = self.tools.execute(&skill.init_code) {
                    eprintln!("warning: skill '{}' init failed: {}", skill.name, e);
                }
            }
        }

        if new_count > 0 {
            let index = Skill::skill_index(skills);
            let sys_msg = &mut self.messages[0].content;
            if let Some(pos) = sys_msg.find("\n## Available Skills") {
                sys_msg.truncate(pos);
            }
            sys_msg.push_str(&new_skill_text);
            sys_msg.push_str(&index);

            eprintln!(
                "hotreload: {} new skill(s) available (total: {})",
                new_count,
                self.loaded_skills.len()
            );
        }
    }

    fn total_tokens(&self) -> usize {
        self.messages.iter().map(|m| estimate_tokens(&m.content)).sum()
    }

    /// Perform self-improvement: analyze conversation and update code of conduct.
    fn self_improve(&mut self, backend: &mut dyn LlmBackend) {
        eprintln!("self-improve: analyzing conversation...");

        // Gather recent conversation history (last 10 user/assistant exchanges)
        let history = self.messages[1..]
            .iter()
            .rev()
            .take(20)
            .map(|m| {
                let role = match m.role {
                    Role::User => "User",
                    Role::Assistant => "Assistant",
                    Role::Tool => "Tool",
                    _ => "System",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n\n");

        // Get existing code of conduct
        let existing_coc = if let Ok(result) = self.tools.execute(
            r#"(sql-query "SELECT content FROM code_of_conduct ORDER BY version DESC LIMIT 1")"#
        ) {
            extract_first_cell(&result).unwrap_or_default()
        } else {
            String::new()
        };

        // Build the reflection prompt
        let reflection_prompt = format!(
            "{}\n\n## Existing Code of Conduct\n{}\n\n## Recent Conversation\n{}\n\nBased on this conversation, write an improved Code of Conduct. Focus on lessons learned and behavioral patterns.",
            SELF_IMPROVE_PROMPT,
            if existing_coc.is_empty() || existing_coc == "nil" {
                "(No existing code of conduct)"
            } else {
                &existing_coc
            },
            history
        );

        // Create a temporary message list for the LLM call
        let reflection_messages = vec![
            Message {
                role: Role::System,
                content: "You are an AI agent reviewing your own behavior to improve. Be concise and actionable.".to_string(),
            },
            Message {
                role: Role::User,
                content: reflection_prompt,
            },
        ];

        // Get LLM response (non-streaming)
        match backend.complete(&reflection_messages) {
            Ok(new_coc) => {
                let new_coc = new_coc.trim().to_string();
                if new_coc.is_empty() {
                    eprintln!("self-improve: LLM returned empty response");
                    return;
                }

                // Store in memory
                let escaped_coc = new_coc.replace('\'', "''");
                let sql = format!(
                    "INSERT INTO code_of_conduct (version, content, reason) VALUES ((SELECT COALESCE(MAX(version), 0) + 1 FROM code_of_conduct), '{}', 'auto-improved from conversation')",
                    escaped_coc
                );

                if let Err(e) = self.tools.execute(&sql) {
                    eprintln!("self-improve: failed to store code of conduct: {}", e);
                    return;
                }

                // Update system prompt
                let sys_msg = &mut self.messages[0].content;
                if let Some(pos) = sys_msg.find("\n## Code of Conduct") {
                    sys_msg.truncate(pos);
                }
                sys_msg.push_str("\n\n");
                sys_msg.push_str(&new_coc);

                eprintln!("self-improve: code of conduct updated");
            }
            Err(e) => {
                eprintln!("self-improve: LLM error: {}", e);
            }
        }
    }

    fn truncate_context(&mut self) {
        while self.total_tokens() > self.max_context_tokens && self.messages.len() > 2 {
            let second = &self.messages[1];
            if second.role == Role::User {
                let removed = self.messages.remove(1);
                let removed_tokens = estimate_tokens(&removed.content);

                self.messages.insert(
                    1,
                    Message {
                        role: Role::User,
                        content: format!(
                            "[Earlier message truncated ({} tokens)]",
                            removed_tokens
                        ),
                    },
                );
            } else {
                break;
            }
        }

        if self.total_tokens() > self.max_context_tokens && self.messages.len() > 3 {
            let removed = self.messages.remove(1);
            let removed_tokens = estimate_tokens(&removed.content);
            self.messages.insert(
                1,
                Message {
                    role: Role::User,
                    content: format!(
                        "[Earlier messages truncated ({} tokens)]",
                        removed_tokens
                    ),
                },
            );
        }
    }

    #[allow(dead_code)]
    pub fn run(&mut self, backend: &mut dyn LlmBackend, user_input: &str) -> Result<String, String> {
        self.messages.push(Message {
            role: Role::User,
            content: user_input.to_string(),
        });

        self.maybe_self_improve(backend);
        self.truncate_context();

        self.run_loop(backend, None)
    }

    pub fn run_streaming(
        &mut self,
        backend: &mut dyn LlmBackend,
        user_input: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<String, String> {
        self.messages.push(Message {
            role: Role::User,
            content: user_input.to_string(),
        });

        self.maybe_self_improve(backend);
        self.truncate_context();

        self.run_loop(backend, Some(on_token))
    }

    /// Check if self-improvement should be triggered (every 5 user turns).
    fn maybe_self_improve(&mut self, backend: &mut dyn LlmBackend) {
        if !self.self_improve {
            return;
        }

        self.user_turn_count += 1;

        // Trigger every 5 user turns, and only if we have enough history
        if self.user_turn_count % 5 == 0 && self.messages.len() > 10 {
            self.self_improve(backend);
        }
    }

    fn run_loop(
        &mut self,
        backend: &mut dyn LlmBackend,
        mut on_token: Option<&mut dyn FnMut(&str)>,
    ) -> Result<String, String> {
        for _ in 0..self.max_turns {
            self.policy.start_turn();

            let response = if let Some(ref mut callback) = on_token {
                backend.complete_streaming(&self.messages, callback)?
            } else {
                backend.complete(&self.messages)?
            };

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
                let result = self.tools.execute(code);
                let output = match result {
                    Ok(val) => val,
                    Err(e) => format!("error: {}", e),
                };
                let output = self.policy.check_output(&output);
                tool_output.push_str(&format!("```\n{}\n```\n", output));
            }

            self.messages.push(Message {
                role: Role::Tool,
                content: tool_output,
            });
        }

        Err("max turns exceeded".to_string())
    }

    #[allow(dead_code)]
    pub fn clear_history(&mut self) {
        self.messages.truncate(1);
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

/// Extract the first cell value from a SQL query result string.
/// Result format: ((columns...) (row1...) (row2...) ...)
fn extract_first_cell(result: &str) -> Option<String> {
    // Find the first row after the header
    // Result looks like: ((col1 col2) (val1 val2) ...)
    let trimmed = result.trim();

    // Find first ( after the header
    let header_end = trimmed.find(") (");
    if header_end == None {
        return None;
    }
    let row_start = header_end.unwrap() + 3;

    // Extract the row content
    let row = &trimmed[row_start..];
    let row_end = row.find(')');
    if row_end == None {
        return None;
    }

    let cell = &row[..row_end.unwrap()];
    // Remove leading/trailing quotes if present
    let cell = cell.trim();
    let cell = cell.trim_matches('"');
    Some(cell.to_string())
}
