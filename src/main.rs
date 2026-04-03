use mcdbg::{DapServer, DebugSession, PythonHookManager, PythonDebugState, ProtocolMessage};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("mcdbg=debug"));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "run" => run_standalone().await?,
        "attach" => {
            if args.len() < 3 {
                eprintln!("Error: attach requires PID argument");
                print_usage();
                return Ok(());
            }
            let pid: u32 = args[2].parse().map_err(|_| "Invalid PID")?;
            attach_to_process(pid).await?;
        }
        "version" => {
            println!("mcdbg v0.1.0");
            println!("Minecraft China Bedrock Python Debugger");
        }
        "help" => print_usage(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("MCDBG - Minecraft China Python Debugger");
    println!();
    println!("Usage:");
    println!("  mcdbg run              - Run as DAP server (listens on port 5632)");
    println!("  mcdbg attach <pid>     - Attach to process");
    println!("  mcdbg version          - Show version");
    println!("  mcdbg help             - Show this help");
    println!();
    println!("VS Code launch.json configuration:");
    println!(r#"{{
  "name": "Minecraft Modpc Debugger",
  "type": "debugpy",
  "request": "attach",
  "connect": {{
    "host": "localhost",
    "port": 5632
  }},
  "pathMappings": [{{
    "localRoot": "${{workspaceFolder}}",
    "remoteRoot": "${{workspaceFolder}}"
  }}]
}}"#);
}

async fn run_standalone() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "127.0.0.1:5632".parse()?;
    let listener = TcpListener::bind(addr).await?;
    
    tracing::info!("MCDBG DAP Server listening on {}", addr);
    println!("MCDBG listening on {} (DAP protocol)", addr);

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        tracing::info!("Client connected: {}", addr);
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, addr).await {
                                tracing::error!("Client error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Accept error: {}", e);
                    }
                }
            }
        }
    }
}

async fn handle_client(
    mut stream: tokio::net::TcpStream,
    _addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut dap_server = DapServer::new();
    let mut debug_state = PythonDebugState::new();
    let mut hook_manager = PythonHookManager::new();
    let mut session: Option<DebugSession> = None;
    let mut buffer = String::new();

    loop {
        buffer.clear();
        match stream.read_to_string(&mut buffer).await? {
            0 => break,
            _ => {}
        }

        if buffer.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<ProtocolMessage>(&buffer) {
            Ok(msg) => {
                let response = process_message(
                    &mut dap_server,
                    &mut debug_state,
                    &mut hook_manager,
                    &mut session,
                    msg,
                ).await;

                if let Some(resp) = response {
                    let mut response_json = serde_json::to_string(&resp)?;
                    response_json.push('\n');
                    stream.write_all(response_json.as_bytes()).await?;
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse message: {}", e);
            }
        }
    }

    if let Some(s) = session.take() {
        let _ = s.detach();
    }

    Ok(())
}

async fn process_message(
    dap_server: &mut DapServer,
    debug_state: &mut PythonDebugState,
    _hook_manager: &mut PythonHookManager,
    session: &mut Option<DebugSession>,
    msg: ProtocolMessage,
) -> Option<ProtocolMessage> {
    let seq = dap_server.next_seq();

    match msg.command.as_deref() {
        Some("initialize") => {
            let request: serde_json::Value = msg.body.unwrap_or_default();
            let _init_req: Result<mcdbg::InitializeRequest, _> = 
                serde_json::from_value(request);
            
            let response = dap_server.handle_initialize(mcdbg::InitializeRequest {
                adapter_id: "mcdbg".to_string(),
                client_id: None,
                client_name: None,
                locale: None,
                lines_start_at1: Some(true),
                columns_start_at1: Some(true),
                path_format: None,
                supports_variable_type: None,
                supports_variable_paging: None,
                supports_run_in_terminal_request: None,
            });
            
            let body = serde_json::to_value(response).ok();
            Some(ProtocolMessage::new_response(seq, msg.seq, "initialize", true, body))
        }
        Some("launch") => {
            let _ = dap_server.handle_launch(&msg.body.unwrap_or_default());
            Some(ProtocolMessage::new_response(seq, msg.seq, "launch", true, None))
        }
        Some("attach") => {
            let body = msg.body.unwrap_or_default();
            let pid = body.get("pid").and_then(|v| v.as_u64()).map(|v| v as u32);
            
            if let Some(pid) = pid {
                match DebugSession::new(pid) {
                    Ok(s) => *session = Some(s),
                    Err(e) => {
                        tracing::error!("Failed to attach: {}", e);
                    }
                }
            }
            
            Some(ProtocolMessage::new_response(seq, msg.seq, "attach", true, None))
        }
        Some("configurationDone") => {
            let _ = dap_server.handle_configuration_done();
            debug_state.resume();
            
            Some(ProtocolMessage::new_response(seq, msg.seq, "configurationDone", true, None))
        }
        Some("threads") => {
            let threads = dap_server.handle_threads();
            let body = serde_json::to_value(serde_json::json!({ "threads": threads })).ok();
            Some(ProtocolMessage::new_response(seq, msg.seq, "threads", true, body))
        }
        Some("stackTrace") => {
            let body = msg.body.unwrap_or_default();
            let thread_id = body.get("threadId").and_then(|v| v.as_u64()).unwrap_or(1) as u64;
            let frames = dap_server.handle_stack_trace(thread_id);
            let body = serde_json::to_value(serde_json::json!({ "stackFrames": frames })).ok();
            Some(ProtocolMessage::new_response(seq, msg.seq, "stackTrace", true, body))
        }
        Some("scopes") => {
            let body = msg.body.unwrap_or_default();
            let frame_id = body.get("frameId").and_then(|v| v.as_u64()).unwrap_or(1) as u64;
            let scopes = dap_server.handle_scopes(frame_id);
            let body = serde_json::to_value(serde_json::json!({ "scopes": scopes })).ok();
            Some(ProtocolMessage::new_response(seq, msg.seq, "scopes", true, body))
        }
        Some("variables") => {
            let body = msg.body.unwrap_or_default();
            let var_ref = body.get("variablesReference").and_then(|v| v.as_u64()).unwrap_or(0) as u64;
            let vars = dap_server.handle_variables(var_ref);
            let body = serde_json::to_value(serde_json::json!({ "variables": vars })).ok();
            Some(ProtocolMessage::new_response(seq, msg.seq, "variables", true, body))
        }
        Some("evaluate") => {
            let body = msg.body.unwrap_or_default();
            let expr = body.get("expression").and_then(|v| v.as_str()).unwrap_or("");
            let frame_id = body.get("frameId").and_then(|v| v.as_u64());
            
            match dap_server.handle_evaluate(expr, frame_id.map(|v| v as u64)) {
                Ok(result) => {
                    let body = serde_json::to_value(serde_json::json!({
                        "result": result,
                        "variablesReference": 0
                    })).ok();
                    Some(ProtocolMessage::new_response(seq, msg.seq, "evaluate", true, body))
                }
                Err(e) => {
                    Some(ProtocolMessage::new_response(seq, msg.seq, "evaluate", false, 
                        serde_json::to_value(serde_json::json!({ "message": e })).ok()))
                }
            }
        }
        Some("disconnect") => {
            if let Some(s) = session.take() {
                let _ = s.detach();
            }
            Some(ProtocolMessage::new_response(seq, msg.seq, "disconnect", true, None))
        }
        _ => None,
    }
}

async fn attach_to_process(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Attaching to process {}", pid);
    
    let session = DebugSession::new(pid)?;
    
    tracing::info!("Successfully attached to process {}", pid);
    println!("Attached to process {}", pid);
    println!("Python interpreter: {:?}", session.python_path);
    println!("Use VS Code with debugpy to connect to port 5632");
    
    std::future::pending().await
}