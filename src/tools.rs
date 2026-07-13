use alisp::Evaluator;
use alisp::expr_to_string;

pub struct AlispHost {
    eval: Evaluator,
}

impl AlispHost {
    pub fn new() -> Self {
        Self {
            eval: Evaluator::new(),
        }
    }

    pub fn execute(&mut self, code: &str) -> Result<String, String> {
        match self.eval.eval_str(code) {
            Ok(Some(val)) => Ok(expr_to_string(&val)),
            Ok(None) => Ok("nil".to_string()),
            Err(e) => Err(e),
        }
    }
}
