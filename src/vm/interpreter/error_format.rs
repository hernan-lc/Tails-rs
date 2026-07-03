use super::*;
use crate::objects::Value;

impl Interpreter {
    pub(crate) fn build_stack_trace(&self, error_name: &str, message: &str) -> String {
        let mut trace = format!(
            "{}{}",
            error_name,
            if message.is_empty() {
                String::new()
            } else {
                format!(": {}", message)
            }
        );

        for frame in self.call_stack.iter().rev() {
            let func_name = frame
                .func_heap_idx
                .and_then(|idx| {
                    if let HeapValue::Function(f) = &self.heap[idx] {
                        f.name.clone()
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "<anonymous>".to_string());

            let location = match (&frame.source_name, frame.source_line, frame.source_col) {
                (Some(name), Some(line), Some(col)) => format!(" ({}:{}:{})", name, line, col),
                (Some(name), Some(line), None) => format!(" ({}:{})", name, line),
                (Some(name), None, _) => format!(" ({})", name),
                (None, Some(line), Some(col)) => format!(" (line {}:{})", line, col),
                (None, Some(line), None) => format!(" (line {})", line),
                (None, None, _) => String::new(),
            };

            trace.push_str(&format!("\n    at {}{}", func_name, location));
        }

        trace
    }

    pub(crate) fn format_rejection_reason(&self, reason: &Value) -> String {
        if let Value::Object(obj_idx) = reason {
            if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                let name = obj
                    .properties
                    .get("name")
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Cons(c) => c.flatten(),
                        _ => "Error".to_string(),
                    })
                    .unwrap_or_else(|| "Error".to_string());
                let message = obj
                    .properties
                    .get("message")
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Cons(c) => c.flatten(),
                        _ => String::new(),
                    })
                    .unwrap_or_default();
                let stack = obj
                    .properties
                    .get("stack")
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Cons(c) => c.flatten(),
                        _ => String::new(),
                    });
                if let Some(stack) = stack {
                    if !stack.is_empty() {
                        return stack;
                    }
                }
                if message.is_empty() {
                    return name;
                }
                return format!("{}: {}", name, message);
            }
        }
        self.value_to_string(reason)
    }

    pub(crate) fn call_stack_backtrace(&self) -> String {
        let mut frames: Vec<String> = Vec::new();

        for frame in self.call_stack.iter().rev() {
            let func_name = frame
                .func_heap_idx
                .and_then(|idx| {
                    if let HeapValue::Function(f) = &self.heap[idx] {
                        f.name.clone()
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "<anonymous>".to_string());

            let location = match (&frame.source_name, frame.source_line, frame.source_col) {
                (Some(name), Some(line), Some(col)) => format!("{}:{}:{}", name, line, col),
                (Some(name), Some(line), None) => format!("{}:{}", name, line),
                (Some(name), None, _) => name.clone(),
                (None, Some(line), Some(col)) => format!("line {}:{}", line, col),
                (None, Some(line), None) => format!("line {}", line),
                (None, None, _) => "<script>".to_string(),
            };

            frames.push(format!("    at {} ({})", func_name, location));
        }

        if frames.is_empty() {
            String::new()
        } else {
            format!("Call stack:\n{}", frames.join("\n"))
        }
    }
}
