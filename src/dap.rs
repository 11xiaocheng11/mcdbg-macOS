use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolMessageType {
    Request,
    Response,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    #[serde(rename = "type")]
    pub message_type: ProtocolMessageType,
    pub seq: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

impl ProtocolMessage {
    pub fn new_request(seq: u64, command: &str, body: Option<serde_json::Value>) -> Self {
        Self {
            message_type: ProtocolMessageType::Request,
            seq,
            request_seq: None,
            success: None,
            command: Some(command.to_string()),
            event: None,
            body,
        }
    }

    pub fn new_response(seq: u64, request_seq: u64, command: &str, success: bool, body: Option<serde_json::Value>) -> Self {
        Self {
            message_type: ProtocolMessageType::Response,
            seq,
            request_seq: Some(request_seq),
            success: Some(success),
            command: Some(command.to_string()),
            event: None,
            body,
        }
    }

    pub fn new_event(seq: u64, event: &str, body: Option<serde_json::Value>) -> Self {
        Self {
            message_type: ProtocolMessageType::Event,
            seq,
            request_seq: None,
            success: None,
            command: None,
            event: Some(event.to_string()),
            body,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    #[serde(rename = "clientID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(rename = "clientName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    #[serde(rename = "adapterID")]
    pub adapter_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_start_at1: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns_start_at1: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_variable_type: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_variable_paging: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_run_in_terminal_request: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    #[serde(rename = "serverID")]
    pub server_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_invalidated_event: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_memory_event: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_args_can_be_interpreted_by_shell: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_configuration_done_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_support_terminate_debuggee: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_delayed_stack_trace_response: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_modules_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_loaded_sources_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_progress_reporting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_memory_read_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_memory_write_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_evaluate_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_step_in_targets_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_goto_targets_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_data_breakpoint_request: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_business: Option<bool>,
    pub completion: Option<bool>,
    pub columns_start_at1: Option<bool>,
    pub lines_start_at1: Option<bool>,
}

impl Default for InitializeResponse {
    fn default() -> Self {
        Self {
            server_id: "mcdbg".to_string(),
            cache_timeout: None,
            supports_invalidated_event: Some(true),
            supports_memory_event: Some(false),
            supports_args_can_be_interpreted_by_shell: Some(false),
            supports_configuration_done_request: Some(true),
            supports_support_terminate_debuggee: Some(false),
            supports_delayed_stack_trace_response: Some(false),
            supports_modules_request: Some(false),
            supports_loaded_sources_request: Some(false),
            supports_progress_reporting: Some(false),
            supports_memory_read_request: Some(true),
            supports_memory_write_request: Some(true),
            supports_evaluate_request: Some(true),
            supports_step_in_targets_request: Some(false),
            supports_goto_targets_request: Some(false),
            supports_data_breakpoint_request: Some(false),
            supports_business: None,
            completion: Some(false),
            columns_start_at1: Some(true),
            lines_start_at1: Some(true),
        }
    }
}

pub struct DapServer {
    sequence: u64,
    capabilities: HashMap<String, bool>,
}

impl DapServer {
    pub fn new() -> Self {
        let mut capabilities = HashMap::new();
        capabilities.insert("supportsStepIn".to_string(), true);
        capabilities.insert("supportsStepOut".to_string(), true);
        capabilities.insert("supportsStepBack".to_string(), false);
        capabilities.insert("supportsRestartFrame".to_string(), false);
        capabilities.insert("supportsGotoTargetsRequest".to_string(), false);
        capabilities.insert("supportsStepInTargets".to_string(), false);
        capabilities.insert("supportsCompletionsRequest".to_string(), false);
        capabilities.insert("supportsModulesRequest".to_string(), false);
        capabilities.insert("supportsConfigurationDoneRequest".to_string(), true);
        capabilities.insert("supportsFunctionBreakpoints".to_string(), false);
        capabilities.insert("supportsConditionalBreakpoints".to_string(), false);
        capabilities.insert("supportsHitConditionalBreakpoints".to_string(), false);
        capabilities.insert("supportsEvaluateForHovers".to_string(), true);
        capabilities.insert("supportsLoadedSourcesRequest".to_string(), false);
        capabilities.insert("supportsProgressReporting".to_string(), false);
        capabilities.insert("supportsRunInTerminalRequest".to_string(), false);
        capabilities.insert("supportsMemoryReferences".to_string(), false);
        capabilities.insert("supportsInvalidatedEvent".to_string(), false);
        
        Self {
            sequence: 1,
            capabilities,
        }
    }

    pub fn next_seq(&mut self) -> u64 {
        let seq = self.sequence;
        self.sequence += 1;
        seq
    }

    pub fn handle_initialize(&mut self, _request: InitializeRequest) -> InitializeResponse {
        InitializeResponse::default()
    }

    pub fn handle_launch(&mut self, args: &serde_json::Value) -> Result<(), String> {
        tracing::info!("Launch request: {:?}", args);
        Ok(())
    }

    pub fn handle_attach(&mut self, args: &serde_json::Value) -> Result<(), String> {
        tracing::info!("Attach request: {:?}", args);
        Ok(())
    }

    pub fn handle_configuration_done(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub fn handle_threads(&mut self) -> Vec<ThreadInfo> {
        vec![ThreadInfo {
            id: 1,
            name: "Main Thread".to_string(),
        }]
    }

    pub fn handle_stack_trace(&mut self, thread_id: u64) -> Vec<StackFrame> {
        vec![StackFrame {
            id: 1,
            name: "main".to_string(),
            source: None,
            line: 1,
            column: 1,
        }]
    }

    pub fn handle_scopes(&mut self, frame_id: u64) -> Vec<Scope> {
        vec![Scope {
            name: "Locals".to_string(),
            variables_reference: frame_id,
            expensive: false,
        }]
    }

    pub fn handle_variables(&mut self, variables_reference: u64) -> Vec<Variable> {
        vec![Variable {
            name: "test".to_string(),
            value: "value".to_string(),
            variables_reference: 0,
            r#type: Some("str".to_string()),
        }]
    }

    pub fn handle_evaluate(&mut self, expression: &str, _frame_id: Option<u64>) -> Result<String, String> {
        Ok(format!("Result: {}", expression))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub id: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    pub line: u64,
    pub column: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub name: String,
    pub variables_reference: u64,
    pub expensive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: String,
    #[serde(rename = "variablesReference")]
    pub variables_reference: u64,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}