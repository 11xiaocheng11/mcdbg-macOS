use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonBreakpoint {
    pub id: u64,
    pub line: u64,
    pub condition: Option<String>,
    pub hit_condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonVariable {
    pub name: String,
    pub value: String,
    pub type_name: String,
    pub reference: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonFrame {
    pub id: u64,
    pub name: String,
    pub file: String,
    pub line: u64,
    pub variables: Vec<PythonVariable>,
}

pub struct PythonHookManager {
    breakpoints: Vec<PythonBreakpoint>,
    eval_callbacks: Vec<Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>>,
}

impl PythonHookManager {
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            eval_callbacks: Vec::new(),
        }
    }

    pub fn add_breakpoint(&mut self, breakpoint: PythonBreakpoint) {
        self.breakpoints.push(breakpoint);
    }

    pub fn remove_breakpoint(&mut self, id: u64) {
        self.breakpoints.retain(|b| b.id != id);
    }

    pub fn get_breakpoint(&self, id: u64) -> Option<&PythonBreakpoint> {
        self.breakpoints.iter().find(|b| b.id == id)
    }

    pub fn should_break_at_line(&self, line: u64) -> bool {
        self.breakpoints.iter().any(|b| b.line == line)
    }

    pub fn register_eval_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) -> Result<String, String> + Send + Sync + 'static,
    {
        self.eval_callbacks.push(Box::new(callback));
    }

    pub fn evaluate(&self, expression: &str) -> Result<String, String> {
        for callback in &self.eval_callbacks {
            return callback(expression);
        }
        Err("No evaluation callback registered".to_string())
    }

    pub fn on_breakpoint_hit(&self, line: u64) -> bool {
        for bp in &self.breakpoints {
            if bp.line == line {
                if let Some(ref condition) = bp.condition {
                    if condition.is_empty() {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for PythonHookManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PythonExecutionState {
    Running,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonDebugState {
    pub state: PythonExecutionState,
    pub current_line: Option<u64>,
    pub current_file: Option<String>,
    pub stack: Vec<PythonFrame>,
}

impl PythonDebugState {
    pub fn new() -> Self {
        Self {
            state: PythonExecutionState::Running,
            current_line: None,
            current_file: None,
            stack: Vec::new(),
        }
    }

    pub fn pause(&mut self, line: u64, file: String) {
        self.state = PythonExecutionState::Paused;
        self.current_line = Some(line);
        self.current_file = Some(file);
    }

    pub fn resume(&mut self) {
        self.state = PythonExecutionState::Running;
        self.current_line = None;
        self.current_file = None;
    }

    pub fn stop(&mut self) {
        self.state = PythonExecutionState::Stopped;
    }
}

impl Default for PythonDebugState {
    fn default() -> Self {
        Self::new()
    }
}